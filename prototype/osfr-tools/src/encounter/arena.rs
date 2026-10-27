//! One battle run's enemy simulation: attack clocks, walk set and skittish kiting.
use super::*;

/// (R) The one arena-simulation value. The engine owns exactly one `ArenaSim`;
/// a group's shared run (`SharedArena`) copies its own into the acting member's
/// engine for one action and copies it back (`enter_arena`, `leave_arena`).
/// Never add a second engine field that would have to be copied separately.
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
    /// Shared runs only: enemy GUID to the member who last damaged it (its target).
    pub(super) last_hit: BTreeMap<u64, u64>,
    /// The member whose action holds a shared run's sim; `None` for a solo run (and so for
    /// `ArenaSim::default()` after every `reset_session`, which also detaches a member).
    pub(super) acting: Option<u64>,
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
    /// Summoned minions still stand (the summoner fights beside them instead of kiting);
    /// read off the whole roster, so a driver's per-member subset cannot change it.
    pub(super) backup: bool,
}

/// Whether a summoner has minions standing: `actors` is the whole roster.
pub(super) fn summoner_backup(actors: &[Actor], states: &[EnemyState]) -> bool {
    actors.iter().any(|a| {
        a.summon_below_percent.is_some()
            && states
                .iter()
                .any(|s| s.guid == a.guid && s.health > 0 && !s.dormant)
    })
}

impl ArenaSim {
    /// A defeated enemy leaves the walk-away and target books.
    pub(super) fn forget(&mut self, guid: u64) {
        self.kiting.remove(&guid);
        self.last_hit.remove(&guid);
    }

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
                if t.backup {
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

/// Most members one shared run admits (R, a sanity bound; the party's own size limit is the
/// real cap, since only party members are invited). A joiner past it is refused.
pub const MAX_MEMBERS: usize = 4;

/// (R) One group's shared battle run, owned by the World outside any session. The run's enemies,
/// clocks, stuns and target buffs live here once; a member's engine borrows them for one action
/// (`enter_arena`, `leave_arena`, both copies, so a run that ends inside the action - a Close -
/// cannot lose the arena). Player health, energy, cooldowns, results and payouts stay in each
/// member's own engine. Runtime only: never saved, so a reconnect cannot seed one.
#[derive(Clone)]
pub struct SharedArena {
    sim: ArenaSim,
    enemies: Vec<EnemyState>,
    stuns: BTreeMap<u64, u64>,
    target_buffs: BTreeMap<(u64, i32), SkillBuff>,
    /// The arena's own tick clock: one driver tick per interval, whoever drives it.
    last_tick: Option<u64>,
    won: bool,
    lost: bool,
    encounter_id: u32,
    /// Characters whose actions run against this arena; the World keeps it current.
    pub members: BTreeSet<u64>,
    /// A tick of this arena faulted: its enemies stay frozen until the members leave.
    pub faulted: bool,
}

impl SharedArena {
    /// The goal enemy is down (every member still fighting is paid by `finish_shared_win`).
    pub fn won(&self) -> bool {
        self.won
    }
    pub fn lost(&self) -> bool {
        self.lost
    }
    /// Every member is knocked out or gone (a Retry makes one a member again): the enemies stand
    /// still and nobody new may join.
    pub fn set_lost(&mut self, lost: bool) {
        self.lost = lost;
    }
    /// A run that is over, frozen or full takes nobody new.
    pub fn admits(&self) -> bool {
        !self.won && !self.lost && !self.faulted && self.members.len() < MAX_MEMBERS
    }
    /// Milliseconds since the last arena tick, or `None` while one is not yet due (100 ms, the
    /// solo tick's step).
    pub fn due(&mut self, now_ms: u64) -> Option<u64> {
        let elapsed = self
            .last_tick
            .map_or(100, |last| now_ms.saturating_sub(last));
        if elapsed < 100 {
            return None;
        }
        self.last_tick = Some(now_ms);
        Some(elapsed)
    }
    pub fn enemies(&self) -> &[EnemyState] {
        &self.enemies
    }
    pub fn attack_clock(&self, guid: u64) -> Option<u64> {
        self.sim.attacks.get(&guid).copied()
    }
}

/// An enemy's target (R): its last attacker while that member is standing in the fight, else
/// the nearest standing member. `players` holds only members who can be hit.
fn target_of(
    last_hit: &BTreeMap<u64, u64>,
    guid: u64,
    at: [f32; 3],
    players: &[FieldParticipant],
) -> Option<u64> {
    last_hit
        .get(&guid)
        .filter(|hitter| players.iter().any(|p| p.guid == **hitter))
        .copied()
        .or_else(|| {
            players
                .iter()
                .min_by(|a, b| distance(a.position, at).total_cmp(&distance(b.position, at)))
                .map(|p| p.guid)
        })
}

impl EncounterEngine {
    /// Swap `sim` in as the engine's arena for one action, then back (the
    /// `with_shared_fields` single-swap rule). The arena loan (`enter_arena` /
    /// `leave_arena`) keeps the same one-value contract; this test pins it.
    #[cfg(test)]
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

    /// Copy this player's live field-battle run into a new shared arena, `member` first in it.
    /// `None` when the run is not a joinable battle (cave, finished, no run).
    pub fn open_arena(&self, member: u64) -> Option<SharedArena> {
        let (encounter_id, _, _) = self.joinable_run()?;
        Some(SharedArena {
            sim: ArenaSim {
                acting: None,
                ..self.arena.clone()
            },
            enemies: self.state.active.as_ref()?.enemies.clone(),
            stuns: self.skills.stuns.clone(),
            target_buffs: self.skills.target_buffs.clone(),
            last_tick: None,
            won: false,
            lost: false,
            encounter_id,
            members: BTreeSet::from([member]),
            faulted: false,
        })
    }

    /// Lend the arena to this engine for one action by `member`: its enemies, clocks, stuns and
    /// target buffs replace the engine's own copies. `false` (nothing changed) when this engine
    /// is not in that battle's run.
    pub fn enter_arena(&mut self, arena: &SharedArena, member: u64) -> bool {
        let Some(active) = &mut self.state.active else {
            return false;
        };
        if active.encounter_id != arena.encounter_id {
            return false;
        }
        active.enemies.clone_from(&arena.enemies);
        self.arena = ArenaSim {
            acting: Some(member),
            ..arena.sim.clone()
        };
        self.skills.stuns.clone_from(&arena.stuns);
        self.skills.target_buffs.clone_from(&arena.target_buffs);
        true
    }

    /// Take the action's changes back into the arena. Nothing is copied when the action ended
    /// the member's run (`reset_session` detached the sim): the arena stays as it was.
    pub fn leave_arena(&mut self, arena: &mut SharedArena) {
        if self.arena.acting.is_none() {
            return;
        }
        arena.sim = std::mem::take(&mut self.arena);
        arena.sim.acting = None;
        if let Some(active) = &self.state.active {
            arena.enemies.clone_from(&active.enemies);
            arena.won |= active.phase == EncounterPhase::Won;
        }
        arena.stuns.clone_from(&self.skills.stuns);
        arena.target_buffs.clone_from(&self.skills.target_buffs);
    }

    /// This player as a target of the shared run: standing, in the fight, scene loaded.
    pub fn arena_participant(&self, guid: u64, position: [f32; 3]) -> Option<FieldParticipant> {
        let fighting = self.armed
            && self.state.player_health > 0
            && self
                .state
                .active
                .as_ref()
                .is_some_and(|a| a.phase == EncounterPhase::Fighting);
        fighting.then(|| self.field_participant(guid, position, false))
    }

    /// The phase of this player's own run.
    pub fn run_phase(&self) -> Option<EncounterPhase> {
        self.state.active.as_ref().map(|a| a.phase.clone())
    }

    /// One tick of a shared run's enemies, on the arena's enemies (call it between `enter_arena`
    /// and `leave_arena`). Each enemy moves and swings once, at its own target; returns, per
    /// member hit, the damage it takes and the packets that tick made (a member's own damage
    /// popups are among them).
    pub fn tick_arena_enemies(
        &mut self,
        players: &[FieldParticipant],
        now_ms: u64,
        elapsed: u64,
    ) -> Vec<(u64, u32, Vec<Vec<u8>>)> {
        let Some(actors) = self.dungeon().map(|d| d.enemies.clone()) else {
            return Vec::new();
        };
        let ground = self.ground();
        let Some(active) = &mut self.state.active else {
            return Vec::new();
        };
        let states = &mut active.enemies;
        let backup = summoner_backup(&actors, states);
        let targets: BTreeMap<u64, u64> = actors
            .iter()
            .filter_map(|a| {
                let s = states.iter().find(|s| s.guid == a.guid)?;
                if s.health == 0 || s.dormant {
                    return None;
                }
                let target = target_of(&self.arena.last_hit, a.guid, s.position, players)?;
                Some((a.guid, target))
            })
            .collect();
        let mut hits = Vec::new();
        for p in players {
            let mine: Vec<Actor> = actors
                .iter()
                .filter(|a| targets.get(&a.guid) == Some(&p.guid))
                .cloned()
                .collect();
            if mine.is_empty() {
                continue;
            }
            let tick = ArenaTick {
                position: p.position,
                ground: &ground,
                behaviours: &self.behaviour_of,
                skills: &self.skills,
                mitigation: p.mitigation,
                player_health: p.health,
                max_health: p.max_health,
                now_ms,
                elapsed,
                backup,
            };
            let mut out = EncounterOutcome::default();
            let damage = self.arena.tick_enemies(mine, states, &tick, &mut out);
            hits.push((p.guid, damage, out.replies));
        }
        hits
    }

    /// Enemy damage the shared run dealt this member (its own health, knockout and results).
    pub fn receive_arena_damage(&mut self, damage: u32, now_ms: u64) -> EncounterOutcome {
        let mut out = EncounterOutcome::default();
        self.take_enemy_damage(damage, now_ms, &mut out);
        self.present_results(&mut out);
        out
    }

    /// The shared run was won by another member: this member's own results, prize wheel and
    /// payout, exactly once (the phase moves to Won; a member already past Fighting gets none).
    pub fn finish_shared_win(&mut self, now_ms: u64) -> EncounterOutcome {
        let mut out = EncounterOutcome::default();
        let Some(active) = self
            .state
            .active
            .as_ref()
            .filter(|a| matches!(a.phase, EncounterPhase::Loading | EncounterPhase::Fighting))
        else {
            return out;
        };
        let d = self.dungeon().unwrap();
        let (count, _) = goal_progress(d, active);
        out.replies.push(objective(2, d.objective_id, count));
        out.changed = true;
        self.win_run(now_ms, &mut out);
        self.present_results(&mut out);
        out
    }

    /// The goal enemy is down in this member's view of the run: leaving minions, the prize
    /// wheel pick, the member's own reward and quest credit, results and rules off. The solo
    /// hit path and a shared run's other members both end here.
    pub(super) fn win_run(&mut self, now_ms: u64, out: &mut EncounterOutcome) {
        let active = self.state.active.as_ref().unwrap();
        let d = self.dungeon().unwrap();
        // The goal is down: summoned minions still standing leave.
        for (a, s) in d.enemies.iter().zip(&active.enemies) {
            if a.summon_below_percent.is_some() && s.health > 0 && !s.dormant {
                out.replies.push(remove(a.guid));
            }
        }
        // One prize from the battle's wheel (R: uniform pick), granted
        // and shown as the loot wheel's landing slot.
        let prize = (!d.prizes.is_empty())
            .then(|| d.prizes[(now_ms % d.prizes.len() as u64) as usize].clone());
        let (id, instance, objective_id, mut reward) = (
            d.id,
            active.instance_id,
            d.objective_id,
            EncounterReward {
                // (R) 50 job XP per battle win (half a rank on the
                // local 100-XP curve); the cave's XP stays with the
                // opening's own rewards.
                job_xp: if d.guide.is_none() { BATTLE_JOB_XP } else { 0 },
                coins: d.reward_coins,
                items: d
                    .reward_items
                    .iter()
                    .copied()
                    .chain(prize.iter().map(|p| (p.item_id, p.quantity)))
                    .collect(),
                ..Default::default()
            },
        );
        let repeatable = d.guide.is_none();
        reward.bonus_items = self.roll_battle_drops(id);
        self.state.active.as_mut().unwrap().phase = EncounterPhase::Won;
        out.win_completed = Some(id);
        // The cave rewards once; overworld battles every win (R).
        // Repeatable battles are never recorded: their ids are not in the
        // opening content that validates `rewarded` at load.
        if repeatable || self.state.rewarded.insert(id) {
            out.rewards.push(reward);
        }
        out.replies.push(objective_complete(objective_id));
        out.replies.push(game_over(true, instance, id));
        if let Some(prize) = &prize {
            // 39/45 MiniGameLootWheelSetItemToLandOn: the wheel lands
            // on the preview entry with this name id (native
            // StopMiniGameLootWheelAt 009b6da0); the client answers
            // 39/46 when the spin stops, then shows the scores.
            let mut wheel = minigame(45, instance, id);
            wheel.extend(prize_bundle(std::slice::from_ref(prize)));
            out.replies.push(wheel);
        }
        out.replies.push(rules(false));
        out.replies.push(combat_flag(133, id, instance, false));
        out.feedback = Some(if repeatable {
            "Battle won. Close the results to return.".into()
        } else {
            "Darkthorne's defense is complete. Close the results to return. This encounter uses reconstructed combat rules.".into()
        });
    }

    /// Enemy damage to the player: the fight log, god mode, health, and the knockout (results
    /// screen in a run; a shared run's stuns and target buffs are the arena's, so a knockout
    /// clears only the member's own effects).
    pub(super) fn take_enemy_damage(
        &mut self,
        mut total_damage: u32,
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) {
        if total_damage > 0 {
            self.log_fight(now_ms, "taken", "enemies", "you", i64::from(total_damage));
        }
        if self.god && total_damage > 0 {
            total_damage = 0;
            out.replies
                .push(player_hp(self.state.player_health, self.max_health));
        }
        if total_damage > 0 {
            let dealt = total_damage.min(self.state.player_health);
            self.state.player_health -= dealt;
            out.changed = true;
            out.replies
                .push(player_hp(self.state.player_health, self.max_health));
            if self.state.player_health == 0 {
                if self.arena.acting.is_some() {
                    self.skills.dots.clear();
                    self.skills.caster_buffs.clear();
                } else {
                    self.skills.clear();
                }
                if let Some(a) = &mut self.state.active {
                    a.phase = EncounterPhase::Lost;
                    out.replies
                        .push(game_over(false, a.instance_id, a.encounter_id));
                    out.replies.push(rules(false));
                    out.replies
                        .push(combat_flag(133, a.encounter_id, a.instance_id, false));
                    out.feedback = Some("Knocked out. Close the results to leave.".into());
                } else if self.world_stage_fighting() {
                    self.state.player_health = self.max_health;
                    out.replies
                        .push(player_hp(self.max_health, self.max_health));
                    let mut z = self.content.world.clone();
                    z.position = self.content.world.position;
                    out.zoning = Some(z);
                    out.feedback =
                        Some("Knocked out; returning to the local opening checkpoint.".into());
                } else {
                    // (R) Overworld knockout: stay down and show the recovered
                    // respawn window; 41/122 (`resume`) revives the player.
                    out.replies.push(self.respawn_window());
                    out.feedback = Some("Knocked out.".into());
                }
            }
        }
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

    const BATTLE: u32 = 96;
    const SUMMONER: u64 = 100_000_096_001;
    /// The two minions of the 75% tier.
    const TIER_75: [u64; 2] = [100_000_096_002, 100_000_096_003];
    /// Where a player stands to hit the summoner (3 m reach, summoner at 136, 175).
    const NEAR: [f32; 3] = [136., -0.6, 173.];

    /// The real battle 96 row on the minimal engine, and two members' engines in its run.
    fn run_of_two() -> (
        std::path::PathBuf,
        EncounterEngine,
        EncounterEngine,
        SharedArena,
    ) {
        let (dir, mut a) = engine();
        a.load_field_battles(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("data/content/field_battles.json"),
        )
        .unwrap();
        let home = CharacterState {
            guid: 1,
            zone_id: 0,
            position: [0.; 3],
            rotation: [0., 0., 1.],
        };
        let mut b = a.clone();
        for member in [&mut a, &mut b] {
            member.begin(BATTLE, home.clone(), 0).unwrap();
            member.on_zone_ready(0, 1_000).unwrap();
        }
        let arena = a.open_arena(1).expect("a live battle run opens an arena");
        (dir, a, b, arena)
    }

    fn health(arena: &SharedArena, guid: u64) -> u32 {
        arena
            .enemies()
            .iter()
            .find(|e| e.guid == guid)
            .unwrap()
            .health
    }

    /// One action by `member` inside the arena loan, as the gateway stages it.
    fn act<T>(
        engine: &mut EncounterEngine,
        arena: &mut SharedArena,
        member: u64,
        action: impl FnOnce(&mut EncounterEngine) -> T,
    ) -> T {
        assert!(engine.enter_arena(arena, member));
        let result = action(engine);
        engine.leave_arena(arena);
        result
    }

    /// Two members hit one summoner: one canonical health drops by both hits, the 75% threshold
    /// is crossed once, and the one minion set is the same for both.
    #[test]
    fn two_members_share_one_summoner_and_one_minion_set() {
        let (dir, mut a, mut b, mut arena) = run_of_two();
        let full = health(&arena, SUMMONER);
        let hit = |e: &mut EncounterEngine, now| e.attack(SUMMONER, NEAR, 43, 0, now).unwrap();
        let first = act(&mut a, &mut arena, 1, |e| hit(e, 2_000));
        let dealt = full - health(&arena, SUMMONER);
        assert!(dealt > 0 && first.replies.iter().any(|r| r.starts_with(&[35, 0, 5, 0])));
        act(&mut b, &mut arena, 2, |e| hit(e, 2_000));
        assert_eq!(
            health(&arena, SUMMONER),
            full - 2 * dealt,
            "both hits count once each"
        );
        // The next hit crosses 75%: exactly one set of minions appears.
        arena.enemies[0].health = full * 75 / 100 + dealt / 2;
        let summoned = |out: &EncounterOutcome| {
            out.replies
                .iter()
                .filter(|r| r.starts_with(&[35, 0, 2, 0]))
                .map(|r| u64::from_le_bytes(r[4..12].try_into().unwrap()))
                .collect::<Vec<_>>()
        };
        let crossing = act(&mut a, &mut arena, 1, |e| hit(e, 3_000));
        assert_eq!(
            summoned(&crossing),
            TIER_75,
            "one minion set, summoned by the crossing hit"
        );
        let after = act(&mut b, &mut arena, 2, |e| hit(e, 3_000));
        assert!(summoned(&after).is_empty(), "B's hit summons nothing more");
        let standing: Vec<u64> = arena
            .enemies()
            .iter()
            .filter(|e| e.guid != SUMMONER && !e.dormant)
            .map(|e| e.guid)
            .collect();
        assert_eq!(standing, TIER_75);
        // Each engine's own copy was only borrowed: a lone engine entering later sees the same set.
        let mut c = a.clone();
        assert!(c.enter_arena(&arena, 3));
        assert_eq!(c.state().active.as_ref().unwrap().enemies, arena.enemies());
        let _ = fs::remove_dir_all(dir);
    }

    /// The arena ticks once per tick for the whole run: a member's own tick skips the enemies, the
    /// driver swings each enemy once, and the hit lands only on the member it targets.
    #[test]
    fn the_driver_ticks_once_and_a_hit_lands_only_on_the_target() {
        let (dir, mut a, mut b, mut arena) = run_of_two();
        // Health 100 is below every threshold: one hit calls every minion in.
        arena.enemies[0].health = 100 + 254;
        act(&mut a, &mut arena, 1, |e| {
            e.attack(SUMMONER, NEAR, 43, 0, 2_000).unwrap()
        });
        let minion = arena
            .enemies()
            .iter()
            .find(|e| e.guid == TIER_75[0])
            .unwrap()
            .clone();
        assert!(!minion.dormant);
        let at = |engine: &EncounterEngine, position: [f32; 3], guid| {
            let mut p = engine.field_participant(guid, position, false);
            p.health = 500;
            p.max_health = 500;
            p
        };
        // A hit B landed last makes the minion B's, even with A standing closer.
        arena.sim.last_hit.insert(TIER_75[0], 2);
        let spot = minion.position;
        let players = [at(&a, spot, 1), at(&b, [spot[0], spot[1], spot[2] + 1.], 2)];
        // A member's own tick, inside the loan, never swings the shared enemies.
        let before = (arena.attack_clock(TIER_75[0]), a.state().player_health);
        let tick = act(&mut b, &mut arena, 2, |e| e.tick(spot, 0, 10_000).unwrap());
        assert!(tick.replies.iter().all(|r| !r.starts_with(&[32, 0, 7, 0])));
        assert_eq!(
            (arena.attack_clock(TIER_75[0]), a.state().player_health),
            before
        );
        // The driver: each enemy swings once, at its own target.
        assert_eq!(arena.due(20_000), Some(100));
        assert_eq!(
            arena.due(20_050),
            None,
            "a second tick inside the interval is not due"
        );
        let hits = act(&mut a, &mut arena, 1, |e| {
            e.tick_arena_enemies(&players, 20_000, 100)
        });
        let damage = |guid| hits.iter().find(|h| h.0 == guid).map_or(0, |h| h.1);
        assert!(damage(2) > 0, "the minion B last hit swings at B: {hits:?}");
        assert_eq!(arena.attack_clock(TIER_75[0]), Some(20_000));
        let swings = |member: u64, guid: u64| {
            hits.iter()
                .filter(|h| h.0 == member)
                .flat_map(|h| h.2.iter())
                .filter(|r| r.starts_with(&[32, 0, 7, 0]) && r[4..12] == guid.to_le_bytes())
                .count()
        };
        assert_eq!(
            (swings(2, TIER_75[0]), swings(1, TIER_75[0])),
            (1, 0),
            "one swing, only at its target"
        );
        let every = |guid: u64| {
            hits.iter()
                .flat_map(|h| h.2.iter())
                .filter(|r| r.starts_with(&[32, 0, 7, 0]) && r[4..12] == guid.to_le_bytes())
                .count()
        };
        for guid in [SUMMONER, TIER_75[0], TIER_75[1]] {
            assert!(every(guid) <= 1, "{guid} swung once for the whole run");
        }
        // Another driver call inside the attack interval swings nothing more.
        let again = act(&mut a, &mut arena, 1, |e| {
            e.tick_arena_enemies(&players, 20_100, 100)
        });
        assert!(
            again.iter().all(|h| h.1 == 0),
            "attack clocks advanced once: {again:?}"
        );
        assert_eq!(arena.attack_clock(TIER_75[0]), Some(20_000));
        let _ = fs::remove_dir_all(dir);
    }

    /// An enemy with no attacker of its own picks the nearest member standing in the fight.
    #[test]
    fn an_enemy_with_no_attacker_targets_the_nearest_member() {
        let (dir, a, _b, _arena) = run_of_two();
        let near = a.field_participant(1, [0., 0., 1.], false);
        let far = a.field_participant(2, [0., 0., 9.], false);
        let none = BTreeMap::new();
        assert_eq!(
            target_of(&none, 5, [0.; 3], &[far.clone(), near.clone()]),
            Some(1)
        );
        // Its last attacker is gone (disconnected, knocked out): the nearest takes over.
        let gone = BTreeMap::from([(5, 3)]);
        assert_eq!(
            target_of(&gone, 5, [0.; 3], &[far.clone(), near.clone()]),
            Some(1)
        );
        let held = BTreeMap::from([(5, 2)]);
        assert_eq!(target_of(&held, 5, [0.; 3], &[far, near]), Some(2));
        let _ = fs::remove_dir_all(dir);
    }

    /// The goal falls to A's hit: the arena is won, A is paid by its own hit, B by its next
    /// step without acting, each exactly once; A's Close inside the loan leaves the arena alone.
    #[test]
    fn a_win_pays_each_member_once_and_a_close_leaves_the_arena_alone() {
        let (dir, mut a, mut b, mut arena) = run_of_two();
        for e in &mut arena.enemies {
            e.health = if e.guid == SUMMONER { 100 } else { 0 };
            e.dormant = false;
        }
        let win = act(&mut a, &mut arena, 1, |e| {
            e.attack(SUMMONER, NEAR, 43, 0, 2_000).unwrap()
        });
        assert!(arena.won());
        assert_eq!((win.rewards.len(), win.win_completed), (1, Some(BATTLE)));
        // B never acted: its own results, wheel and payout come from the finishing step.
        let paid = act(&mut b, &mut arena, 2, |e| e.finish_shared_win(2_100));
        assert_eq!((paid.rewards.len(), paid.win_completed), (1, Some(BATTLE)));
        assert!(
            paid.replies.iter().any(|r| r.starts_with(&[39, 0, 18])),
            "game over, won"
        );
        assert_eq!(b.run_phase(), Some(EncounterPhase::Won));
        let again = act(&mut b, &mut arena, 2, |e| e.finish_shared_win(2_200));
        assert!(
            again.rewards.is_empty() && again.win_completed.is_none(),
            "once"
        );
        // A closes the results inside its loan: the run ends for A alone.
        let before = arena.enemies().to_vec();
        assert!(a.enter_arena(&arena, 1));
        let closed = a.exit().unwrap();
        a.leave_arena(&mut arena);
        assert!(closed.zoning.is_some() && !a.active());
        assert_eq!(
            arena.enemies(),
            before,
            "A's Close leaves the shared enemies as they were"
        );
        assert!(arena.won() && b.active());
        let _ = fs::remove_dir_all(dir);
    }

    /// A knockout is personal: that member's results show, the arena's stuns survive, and the
    /// other member keeps fighting.
    #[test]
    fn a_knockout_is_personal_and_keeps_the_arena_effects() {
        let (dir, mut a, b, mut arena) = run_of_two();
        arena.stuns.insert(SUMMONER, u64::MAX);
        let out = act(&mut a, &mut arena, 1, |e| {
            e.receive_arena_damage(u32::MAX, 5_000)
        });
        assert_eq!(a.run_phase(), Some(EncounterPhase::Lost));
        assert!(
            out.replies.iter().any(|r| r.starts_with(&[39, 0, 18])),
            "game over, lost"
        );
        assert_eq!(b.run_phase(), Some(EncounterPhase::Fighting));
        assert_eq!(
            arena.stuns.get(&SUMMONER),
            Some(&u64::MAX),
            "a member's knockout keeps the stun"
        );
        assert!(!arena.won() && !arena.lost());
        let _ = fs::remove_dir_all(dir);
    }

    /// A run that is over, frozen or full takes nobody new.
    #[test]
    fn only_a_live_arena_with_room_admits() {
        let (dir, _a, _b, arena) = run_of_two();
        assert!(arena.admits());
        let mut full = arena.clone();
        full.members.extend(10..10 + MAX_MEMBERS as u64);
        assert!(!full.admits(), "full");
        for over in [
            |a: &mut SharedArena| a.won = true,
            |a: &mut SharedArena| a.set_lost(true),
            |a: &mut SharedArena| a.faulted = true,
        ] {
            let mut closed = arena.clone();
            over(&mut closed);
            assert!(!closed.admits());
        }
        let _ = fs::remove_dir_all(dir);
    }

    /// A player with no run in that battle cannot borrow the arena, and leaving changes nothing.
    #[test]
    fn a_member_that_is_not_in_the_run_cannot_borrow_it() {
        let (dir, _a, _b, mut arena) = run_of_two();
        let (_, mut outside) = engine();
        assert!(!outside.enter_arena(&arena, 9), "no run, no loan");
        let before = arena.enemies().to_vec();
        outside.leave_arena(&mut arena);
        assert_eq!(arena.enemies(), before);
        let _ = fs::remove_dir_all(dir);
    }
}
