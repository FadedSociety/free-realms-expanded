//! One battle run's enemy simulation: attack clocks, walk set and skittish kiting.
use super::*;

/// (R) The one arena-simulation value. The engine owns exactly one `ArenaSim`
/// and `with_arena` swaps the whole value for a single action, so a group's
/// shared run can later hand one to each member's engine. Never add a second
/// engine field that would have to be swapped separately.
///
/// Runtime clocks only: none of it is saved (`ActiveEncounter` still holds the
/// enemies and phase), so every load, begin and `reset_session` starts it
/// empty. Keys are arena enemy GUIDs, and the opening stage's `world_enemies`
/// share the same tick loop and so the same maps. Field enemy GUIDs never
/// appear here: the encounter, field-enemy and field-battle loaders refuse an
/// actor GUID that another set already uses (`arena_and_field_guids_are_disjoint`).
#[derive(Clone, Default)]
pub(super) struct ArenaSim {
    /// Enemy GUID to its last attack time.
    pub(super) attacks: BTreeMap<u64, u64>,
    /// Enemy GUIDs whose walk animation is active.
    pub(super) moving: BTreeSet<u64>,
    /// Skittish actors fighting alone: running or resting, until a time.
    pub(super) kiting: BTreeMap<u64, Kite>,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Kite {
    Run(u64),
    Rest(u64),
}

/// What one arena tick reads from the engine; the sim itself is `&mut`.
pub(super) struct ArenaTick<'a> {
    pub(super) position: [f32; 3],
    pub(super) ground: &'a Ground,
    pub(super) behaviours: &'a BTreeMap<u64, Behaviour>,
    pub(super) skills: &'a SkillEffects,
    pub(super) mitigation: f32,
    pub(super) player_health: u32,
    pub(super) max_health: u32,
    pub(super) now_ms: u64,
    pub(super) elapsed: u64,
}

impl ArenaSim {
    /// Move, chase and swing every standing enemy for one tick; returns the
    /// damage the player takes (the caller applies it).
    pub(super) fn tick_enemies(
        &mut self,
        actors: Vec<Actor>,
        states: &mut [EnemyState],
        t: &ArenaTick,
        out: &mut EncounterOutcome,
    ) -> u32 {
        let (position, now_ms, elapsed) = (t.position, t.now_ms, t.elapsed);
        let mut total_damage = 0_u32;
        // Summoned minions still standing: the summoner fights alongside them.
        let backup = actors.iter().any(|a| {
            a.summon_below_percent.is_some()
                && states
                    .iter()
                    .any(|s| s.guid == a.guid && s.health > 0 && !s.dormant)
        });
        for actor in actors {
            let s = states.iter_mut().find(|s| s.guid == actor.guid).unwrap();
            if s.health == 0 || s.dormant {
                continue;
            }
            if t.skills.stunned(actor.guid, now_ms) {
                continue;
            }
            let b = t.behaviours.get(&actor.guid).cloned().unwrap_or_default();
            if b.mode == Mode::Skittish {
                if backup {
                    self.kiting.remove(&actor.guid);
                } else {
                    self.kite_turn(&actor, s, t, &b, out);
                    continue;
                }
            }
            let delta = distance(position, s.position);
            if delta > b.chase_limit {
                if self.moving.remove(&actor.guid) {
                    out.replies.push(animation(actor.guid, 1, true));
                }
                continue;
            }
            if delta > b.reach && b.mode != Mode::Stationary {
                let heading = [
                    (position[0] - s.position[0]) / delta,
                    0.0,
                    (position[2] - s.position[2]) / delta,
                ];
                // Stop inside reach so f32 rounding at world coordinates cannot
                // strand a pursuer just outside the attack threshold.
                let step =
                    (b.move_speed * elapsed.min(250) as f32 / 1000.0).min(delta - (b.reach - 0.1));
                for (coord, to) in s.position.iter_mut().zip(position) {
                    *coord += (to - *coord) / delta * step;
                }
                t.ground.snap(&mut s.position);
                out.replies.push(movement(actor.guid, s.position, heading));
                if self.moving.insert(actor.guid) {
                    out.replies.push(animation(actor.guid, 3, true));
                }
                out.changed = true;
            }
            if distance(position, s.position) <= b.reach {
                if self.moving.remove(&actor.guid) {
                    out.replies.push(animation(actor.guid, 1, true));
                }
                if self
                    .attacks
                    .get(&actor.guid)
                    .is_some_and(|last| now_ms.saturating_sub(*last) < b.attack_ms)
                {
                    continue;
                }
                self.attacks.insert(actor.guid, now_ms);
                out.replies.push(animation(actor.guid, 1001, false));
                let attack_damage = (actor.damage as f32 * t.mitigation).round() as u32;
                let damage = attack_damage.min(t.player_health.saturating_sub(total_damage));
                out.replies.push(attack_processed(
                    actor.guid,
                    1,
                    damage,
                    t.player_health
                        .saturating_sub(total_damage)
                        .saturating_sub(damage),
                    t.max_health,
                    false,
                ));
                total_damage = total_damage.saturating_add(attack_damage);
            }
        }
        total_damage
    }

    /// One tick of a skittish actor fighting alone: run from the player, then
    /// rest (no attacks), then run again.
    fn kite_turn(
        &mut self,
        actor: &Actor,
        s: &mut EnemyState,
        t: &ArenaTick,
        b: &Behaviour,
        out: &mut EncounterOutcome,
    ) {
        let (player, now_ms, elapsed) = (t.position, t.now_ms, t.elapsed);
        let delta = distance(player, s.position).max(0.01);
        let away = [
            (s.position[0] - player[0]) / delta,
            0.0,
            (s.position[2] - player[2]) / delta,
        ];
        let phase = *self
            .kiting
            .entry(actor.guid)
            .or_insert(Kite::Run(now_ms.saturating_add(b.run_ms)));
        match phase {
            Kite::Rest(until) if now_ms < until => {}
            Kite::Rest(_) => {
                self.kiting
                    .insert(actor.guid, Kite::Run(now_ms.saturating_add(b.run_ms)));
            }
            Kite::Run(until) if now_ms >= until => {
                self.kiting
                    .insert(actor.guid, Kite::Rest(now_ms.saturating_add(b.rest_ms)));
                self.moving.remove(&actor.guid);
                // Turn to face the player and catch its breath.
                out.replies
                    .push(movement(actor.guid, s.position, [-away[0], 0.0, -away[2]]));
                out.replies.push(animation(actor.guid, 1, true));
                out.replies
                    .push(animation(actor.guid, b.rest_animation, false));
                out.changed = true;
            }
            Kite::Run(until) if delta > b.run_from => {
                self.kiting
                    .insert(actor.guid, Kite::Run(until.saturating_add(elapsed)));
                if self.moving.remove(&actor.guid) {
                    out.replies
                        .push(movement(actor.guid, s.position, [-away[0], 0.0, -away[2]]));
                    out.replies.push(animation(actor.guid, 1, true));
                    out.changed = true;
                }
            }
            Kite::Run(_) => {
                let step = b.run_speed * elapsed.min(250) as f32 / 1000.0;
                // Straight away, or along the leash edge (either tangent) when cornered.
                let directions = [away, [away[2], 0.0, -away[0]], [-away[2], 0.0, away[0]]];
                let next = directions.iter().find_map(|d| {
                    let next = t.ground.snapped([
                        s.position[0] + d[0] * step,
                        player[1],
                        s.position[2] + d[2] * step,
                    ]);
                    let flat = [next[0], actor.position[1], next[2]];
                    (distance(flat, actor.position) <= b.leash).then_some((next, *d))
                });
                if let Some((next, heading)) = next {
                    s.position = next;
                    out.replies.push(movement(actor.guid, s.position, heading));
                    if self.moving.insert(actor.guid) {
                        out.replies.push(animation(actor.guid, 3, true));
                    }
                    out.changed = true;
                }
            }
        }
    }
}

impl EncounterEngine {
    /// Swap `sim` in as the engine's arena for one action, then back (the
    /// `with_shared_fields` single-swap rule). ponytail: the first non-test
    /// caller is the group arena (T441).
    #[allow(dead_code)]
    pub(super) fn with_arena<T>(
        &mut self,
        sim: &mut ArenaSim,
        action: impl FnOnce(&mut Self) -> T,
    ) -> T {
        std::mem::swap(&mut self.arena, sim);
        let result = action(self);
        std::mem::swap(&mut self.arena, sim);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::super::shared::tests::{TEST_ENEMY, engine};
    use super::*;

    fn fighting() -> (std::path::PathBuf, EncounterEngine) {
        let (dir, mut engine) = engine();
        let home = CharacterState {
            guid: 1,
            zone_id: 0,
            position: [0.; 3],
            rotation: [0., 0., 1.],
        };
        engine.begin(411, home, 0).unwrap();
        engine.on_zone_ready(0, 1_000).unwrap();
        assert!(engine.armed, "a battle with no guide arms on load");
        (dir, engine)
    }

    /// The arena's clocks and walks live in `ArenaSim`; the field's stay in `FieldSim`.
    #[test]
    fn arena_enemy_clocks_live_in_the_arena_sim() {
        let (dir, mut engine) = fighting();
        // Ten metres off: the dungeon enemy (guid 300) walks toward the player.
        engine.tick([10., 0., 0.], 0, 1_100).unwrap();
        assert!(engine.arena.moving.contains(&300), "walk set");
        // Standing on it: the enemy swings and starts its attack clock.
        engine.tick([0.; 3], 0, 20_000).unwrap();
        assert_eq!(
            engine.arena.attacks.get(&300),
            Some(&20_000),
            "attack clock"
        );
        assert!(engine.field_sim.attacks.is_empty() && engine.field_sim.moving.is_empty());
        // Leaving the arena forgets every clock, the field's too.
        engine.field_sim.attacks.insert(TEST_ENEMY, 5);
        engine.reset_session();
        assert!(engine.arena.attacks.is_empty() && engine.arena.moving.is_empty());
        assert!(engine.field_sim.attacks.is_empty());
        let _ = fs::remove_dir_all(dir);
    }

    /// One value swaps in and out: all three maps, nothing else touched.
    #[test]
    fn with_arena_swaps_the_whole_sim_for_one_action() {
        let (dir, mut engine) = engine();
        engine.arena.attacks.insert(7, 70);
        let mut other = ArenaSim::default();
        other.attacks.insert(1, 10);
        other.moving.insert(2);
        other.kiting.insert(3, Kite::Rest(30));
        engine.with_arena(&mut other, |e| {
            assert_eq!(e.arena.attacks.keys().collect::<Vec<_>>(), [&1]);
            assert!(e.arena.moving.contains(&2) && e.arena.kiting.contains_key(&3));
            e.arena.moving.insert(4);
        });
        assert_eq!(engine.arena.attacks.get(&7), Some(&70), "own sim is back");
        assert!(engine.arena.moving.is_empty() && engine.arena.kiting.is_empty());
        assert!(
            other.moving.contains(&4),
            "the action's change stays in the swapped sim"
        );
        assert!(engine.field_sim.attacks.is_empty() && engine.field_sim.moving.is_empty());
        let _ = fs::remove_dir_all(dir);
    }

    /// Arena clocks are keyed by GUID, so the field's GUIDs must never be an arena's.
    #[test]
    fn arena_and_field_guids_are_disjoint() {
        let (dir, mut engine) = engine();
        let arena: BTreeSet<u64> = engine
            .content
            .world_enemies
            .iter()
            .map(|a| a.guid)
            .chain(engine.content.dungeons.iter().flat_map(|d| {
                d.enemies
                    .iter()
                    .map(|a| a.guid)
                    .chain(d.guide.iter().map(|g| g.guid))
            }))
            .collect();
        assert!(arena.contains(&300) && !arena.contains(&TEST_ENEMY));
        assert!(
            engine
                .field_sim
                .enemies
                .iter()
                .all(|f| !arena.contains(&f.actor.guid)),
            "loaded field GUIDs overlap an arena's"
        );
        // A field enemy that reuses an arena GUID is refused, and nothing changes.
        let mut clash: serde_json::Value =
            serde_json::from_slice(&fs::read(dir.join("field_enemies.json")).unwrap()).unwrap();
        clash["enemies"][0]["guid"] = serde_json::json!(300);
        fs::write(dir.join("clash.json"), clash.to_string()).unwrap();
        let error = engine
            .load_field_enemies(&dir.join("clash.json"))
            .unwrap_err();
        assert!(
            error.to_string().contains("duplicate field enemy GUID"),
            "{error}"
        );
        assert_eq!(engine.field_sim.enemies[0].actor.guid, TEST_ENEMY);
        let _ = fs::remove_dir_all(dir);
    }
}
