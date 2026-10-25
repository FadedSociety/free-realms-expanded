//! Overworld field enemies and battle markers at runtime: wander and chase AI and the battle start screen.
use super::*;

// Content loading and staged reloads. `#[path]` keeps the `encounter` hub
// untouched; the module is still a descendant of `encounter`.
#[path = "field_content.rs"]
mod field_content;

/// A battle marker's walk (R): four points WANDER_RADIUS around its home at
/// walking pace, pausing at each (the owner: the Troll Summoner "just walks
/// around"). Height stays at home: the server has no terrain heights.
#[derive(Clone, Copy, Debug)]
pub(super) struct Wander {
    pub(super) position: [f32; 3],
    pub(super) waypoint: usize,
    pub(super) pause_until: u64,
}

pub(super) const WANDER_RADIUS: f32 = 5.0;

pub(super) const WANDER_SPEED: f32 = 1.5;

pub(super) const WANDER_PAUSE_MS: u64 = 3000;

/// Only walk while a player could see it (NPC visibility radius is 100 m).
pub(super) const WANDER_SEEN: f32 = 90.0;

/// Closer than this it stops and faces the player (an easy click target).
pub(super) const WANDER_NOTICE: f32 = 8.0;

/// Battle-marker interaction radius: the start screen opens, and its Go
/// (41/108) may start the battle, only within this distance of the marker.
pub(super) const BATTLE_SCREEN_RADIUS: f32 = 8.0;

/// The one level rule for a published battle: the active job level must reach
/// the row's configured floor. Portal admission, the battle marker and
/// `EncounterEngine::begin` all call this; `Some(required)` means refused.
pub(crate) fn level_refusal(level: u32, required: u32) -> Option<u32> {
    (level < required).then_some(required)
}

impl FieldEnemy {
    fn reset_motion(&mut self) {
        self.motion = FieldMotion::Idle;
        self.roam_goal = None;
        self.pause_until = 0;
        self.seed = self.actor.guid;
        self.target = None;
        self.reset_serial = self.reset_serial.wrapping_add(1);
    }

    /// (R) Per-actor deterministic random stream, same LCG as combat crits.
    fn random(&mut self) -> f32 {
        self.seed = self
            .seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.seed >> 40) as f32 / 16_777_216.0
    }
}

impl EncounterEngine {
    /// Authored field level, for client presentation by the owning gameplay
    /// route once its wire carrier is verified. Never infer it from HP.
    pub fn field_level(&self, guid: u64) -> Option<u32> {
        self.field_sim
            .enemies
            .iter()
            .find(|f| f.present && f.actor.guid == guid)
            .map(|f| f.actor.level)
    }

    /// Developer teleport targets for living field enemies (zone 0).
    pub fn field_teleport_targets(&self) -> Vec<crate::dev_teleport::Target> {
        self.field_sim
            .enemies
            .iter()
            .filter(|f| f.present && f.state.health > 0)
            .map(|f| crate::dev_teleport::Target {
                guid: f.actor.guid,
                name: f.actor.name.clone(),
                zone_id: self.content.world.zone_id,
                position: f.state.position,
                heading: None,
                kind: "enemy".into(),
            })
            .chain(self.battles.iter().map(|b| crate::dev_teleport::Target {
                guid: b.spawner.guid,
                name: b.spawner.name.clone(),
                zone_id: 0,
                position: b.spawner.position,
                heading: None,
                kind: "battle".into(),
            }))
            .chain(self.dungeon().into_iter().flat_map(|d| {
                // Inside an arena: the enemies currently standing.
                let states = &self.state.active.as_ref().unwrap().enemies;
                d.enemies
                    .iter()
                    .zip(states)
                    .filter(|(_, s)| s.health > 0 && !s.dormant)
                    .map(|(a, s)| crate::dev_teleport::Target {
                        guid: a.guid,
                        name: a.name.clone(),
                        zone_id: d.zone.zone_id,
                        position: s.position,
                        heading: None,
                        kind: "enemy".into(),
                    })
            }))
            .collect()
    }

    /// Battle spawners for the world's proximity visibility:
    /// (guid, name, name id, model id, position).
    /// (guid, name, name id, model, position, overhead icon image id).
    pub fn battle_spawners(&self) -> Vec<(u64, String, i32, i32, [f32; 3], i32)> {
        self.battles
            .iter()
            .map(|b| {
                (
                    b.spawner.guid,
                    b.spawner.name.clone(),
                    b.spawner.name_id,
                    b.spawner.model_id,
                    b.spawner.position,
                    b.nameplate_image_id,
                )
            })
            .collect()
    }

    /// A field battle is playable only when every previewed prize can be
    /// granted: the definition resolves and its quantity fits one stack.
    /// Cave encounters have no `battles` row and are not subject to this check.
    pub fn battle_prizes_available(&self, id: u32, accepts: impl Fn(i32, u32) -> bool) -> bool {
        self.battles
            .iter()
            .find(|battle| battle.id == id)
            .is_none_or(|battle| {
                battle
                    .prizes
                    .iter()
                    .all(|prize| accepts(prize.item_id, prize.quantity))
            })
    }

    /// The return location of a saved battle dropped at load, once.
    pub fn take_dropped_return(&mut self) -> Option<CharacterState> {
        self.dropped_return.take()
    }

    /// An overworld battle's arena is active (a dungeon without a guide).
    pub fn in_battle_arena(&self) -> bool {
        self.dungeon().is_some_and(|d| d.guide.is_none())
    }

    /// Show a battle's start screen again (e.g. after Go was refused: the
    /// screen hid the HUD, so the player needs it back to use Switch Jobs).
    pub fn reopen_battle_screen(&mut self, battle: u32, profile: i32) -> Vec<Vec<u8>> {
        let Some(b) = self.battles.iter().find(|b| b.id == battle) else {
            return Vec::new();
        };
        let instance = self.state.next_instance_id;
        let packets = vec![
            details_packet(
                b.id,
                instance,
                [
                    b.title_id,
                    b.icon_id,
                    b.description_id,
                    b.objective_id,
                    b.difficulty,
                ],
                b.enemies.len(),
                profile,
                false,
                &b.prizes,
            ),
            encounter_header(107, b.id, instance),
        ];
        self.pending_battle = Some((b.id, instance));
        packets
    }

    /// `Some(required)` when `level` is below the battle's configured floor.
    pub(crate) fn battle_level_refusal(&self, battle: u32, level: u32) -> Option<u32> {
        let required = self
            .battles
            .iter()
            .find(|row| row.id == battle)?
            .required_combat_level;
        level_refusal(level, required)
    }

    /// Level refusal for the battle whose start screen is open.
    pub(crate) fn pending_battle_level_refusal(&self, level: u32) -> Option<u32> {
        let (id, _) = self.pending_battle?;
        self.battle_level_refusal(id, level)
    }

    /// Every production `begin` route asks this before the encounter changes.
    pub(super) fn admit_battle(&self, battle: u32) -> io::Result<()> {
        match self.battle_level_refusal(battle, self.job_level) {
            Some(required) => Err(invalid(&format!("combat level {required} required"))),
            None => Ok(()),
        }
    }

    /// T440: this player's live field-battle run as (battle id, run instance, title string id),
    /// the one a grouped member may be invited into. Caves and finished runs are never joinable.
    pub fn joinable_run(&self) -> Option<(u32, i32, i32)> {
        let run = self.state.active.as_ref()?;
        let live = matches!(
            run.phase,
            EncounterPhase::Loading | EncounterPhase::Fighting
        );
        let battle = self.battles.iter().find(|b| b.id == run.encounter_id)?;
        live.then_some((battle.id, run.instance_id, battle.title_id))
    }

    /// The battle whose start screen is open, consumed by Go (41/108). The
    /// player must still stand within `BATTLE_SCREEN_RADIUS` of the marker:
    /// an out-of-reach screen is dropped so Go cannot start a battle after
    /// walking away. `None` covers no open screen and a dropped one.
    pub fn take_pending_battle(&mut self, position: [f32; 3]) -> Option<u32> {
        let (id, _) = self.pending_battle.take()?;
        let near =
            self.battles.iter().find(|b| b.id == id).is_some_and(|b| {
                distance(position, self.marker_position(b)) <= BATTLE_SCREEN_RADIUS
            });
        near.then_some(id)
    }

    /// The marker's current spot: where its walk has taken it, else home.
    fn marker_position(&self, battle: &FieldBattle) -> [f32; 3] {
        self.field_sim
            .wander
            .get(&battle.id)
            .map_or(battle.spawner.position, |w| w.position)
    }

    pub fn is_battle_spawner(&self, guid: u64) -> bool {
        self.battles.iter().any(|b| b.spawner.guid == guid)
    }

    /// Actors whose interactions the encounter owns (overworld enemies and
    /// battle spawners); Legacy also lists some of their GUIDs as plain NPCs.
    pub fn is_field_actor(&self, guid: u64) -> bool {
        self.is_field_enemy(guid) || self.is_battle_spawner(guid)
    }

    /// Spawner click: show the battle's start screen (difficulty, description,
    /// goal) without launching it. Go comes back as 41/108.
    pub(super) fn battle_screen(
        &mut self,
        guid: u64,
        position: [f32; 3],
        profile: i32,
    ) -> EncounterOutcome {
        if self.active() {
            return EncounterOutcome::default();
        }
        let Some(b) = self.battles.iter().find(|b| b.spawner.guid == guid) else {
            return EncounterOutcome::default();
        };
        let at = self.marker_position(b);
        if distance(position, at) > BATTLE_SCREEN_RADIUS {
            return EncounterOutcome::default();
        }
        let instance = self.state.next_instance_id;
        let prizes = b.prizes.clone();
        let (id, icon, title, description, objective, difficulty, count) = (
            b.id,
            b.icon_id,
            b.title_id,
            b.description_id,
            b.objective_id,
            b.difficulty,
            b.enemies.len(),
        );
        self.pending_battle = Some((id, instance));
        EncounterOutcome {
            replies: vec![
                details_packet(
                    id,
                    instance,
                    [title, icon, description, objective, difficulty],
                    count,
                    profile,
                    false,
                    &prizes,
                ),
                // 41/107 ZoneIsReady: the battle is in place, nothing to load.
                // DispatchEncounterPacket case 107 calls SetMiniGameReady
                // (00aa3fdf), turning the start screen's "Loading..." into Go!
                // (native run 023211 stayed on Loading without it).
                encounter_header(107, id, instance),
            ],
            ..Default::default()
        }
    }

    pub fn is_field_enemy(&self, guid: u64) -> bool {
        self.field_sim
            .enemies
            .iter()
            .any(|f| f.present && f.actor.guid == guid)
    }

    /// Show or hide quest-gated field enemies for the player's active quests.
    /// A newly present enemy spawns at full health at home.
    pub fn set_active_quests(&mut self, active: &BTreeSet<u32>) -> EncounterOutcome {
        let mut out = EncounterOutcome::default();
        if self.active() || self.shared_fields.is_some() {
            return out;
        }
        let mut shown = Vec::new();
        for i in 0..self.field_sim.enemies.len() {
            let f = &mut self.field_sim.enemies[i];
            if f.quest_ids.is_empty() {
                continue;
            }
            let present = f.quest_ids.iter().any(|id| active.contains(id));
            if present == f.present {
                continue;
            }
            f.present = present;
            out.changed = true;
            if present {
                f.state = enemy_state(&f.actor);
                f.respawn_at = None;
                f.reset_motion();
                self.field_sim.attacks.remove(&f.actor.guid);
                shown.push(i);
            } else {
                if f.state.health > 0 {
                    out.replies.push(remove(f.actor.guid));
                }
                self.field_sim.moving.remove(&f.actor.guid);
            }
        }
        for i in &shown {
            let f = &self.field_sim.enemies[*i];
            out.replies.push(self.spawn(&f.actor, f.state.position, 0));
            out.replies
                .push(enemy_hp(f.actor.guid, f.state.health, f.actor.max_health));
        }
        if !shown.is_empty() {
            let guids: Vec<u64> = shown
                .iter()
                .map(|i| self.field_sim.enemies[*i].actor.guid)
                .collect();
            out.replies.push(relevance(&guids, false));
        }
        out
    }

    pub fn field_guids(&self) -> Vec<u64> {
        self.field_sim
            .enemies
            .iter()
            .map(|f| f.actor.guid)
            .collect()
    }

    pub(super) fn field_initialization(&mut self) -> Vec<Vec<u8>> {
        self.field_combat = false;
        let mut packets = Vec::new();
        let mut alive = Vec::new();
        for f in &self.field_sim.enemies {
            if f.present && f.state.health > 0 {
                packets.push(self.spawn(&f.actor, f.state.position, 0));
                packets.push(enemy_hp(f.actor.guid, f.state.health, f.actor.max_health));
                alive.push(f.actor.guid);
            }
        }
        if !alive.is_empty() {
            packets.push(relevance(&alive, false));
        }
        packets
    }

    pub(super) fn tick_wanderers_for(
        &mut self,
        players: &[FieldParticipant],
        now_ms: u64,
        elapsed: u64,
        out: &mut EncounterOutcome,
    ) {
        let ground = self.ground();
        for b in &self.battles {
            let home = b.spawner.position;
            let w = self.field_sim.wander.entry(b.id).or_insert(Wander {
                position: home,
                waypoint: 0,
                pause_until: 0,
            });
            let Some(player) = players
                .iter()
                .min_by(|a, b| {
                    distance(a.position, w.position)
                        .total_cmp(&distance(b.position, w.position))
                        .then(a.guid.cmp(&b.guid))
                })
                .map(|p| p.position)
            else {
                continue;
            };
            let near = distance(player, w.position);
            if near > WANDER_SEEN {
                continue;
            }
            let guid = b.spawner.guid;
            if near < WANDER_NOTICE {
                // It notices the player: stop and face them, so it can be clicked.
                if self.field_sim.moving.contains(&guid) {
                    // Notice exception: this facing divides by the 3-D `near`
                    // clamped at 1 cm, not by `heading`'s true distance. Below
                    // 1 cm the direction is shorter than a unit vector; the
                    // client normalizes it and ignores it under its 0.01
                    // squared-length threshold (`0x0095dae0`), so these bytes
                    // must not change.
                    let d = near.max(0.01);
                    let face = [
                        (player[0] - w.position[0]) / d,
                        0.0,
                        (player[2] - w.position[2]) / d,
                    ];
                    out.replies.push(movement(guid, w.position, face));
                }
                stand(guid, &mut self.field_sim.moving, out);
                continue;
            }
            if now_ms < w.pause_until {
                continue;
            }
            let angle =
                std::f32::consts::FRAC_PI_4 + w.waypoint as f32 * std::f32::consts::FRAC_PI_2;
            // Waypoints sit on the ground, so the walker can reach them exactly.
            let goal = ground.snapped([
                home[0] + WANDER_RADIUS * angle.cos(),
                home[1],
                home[2] + WANDER_RADIUS * angle.sin(),
            ]);
            let left = distance(w.position, goal);
            let step = WANDER_SPEED * elapsed.min(250) as f32 / 1000.0;
            if left <= step {
                w.position = goal;
                w.waypoint = (w.waypoint + 1) % 4;
                w.pause_until = now_ms.saturating_add(WANDER_PAUSE_MS);
                // Waypoint exception: the arrival facing divides the goal
                // offset by WANDER_RADIUS, not by the 3-D distance to it.
                let face = [
                    (goal[0] - home[0]) / WANDER_RADIUS,
                    0.0,
                    (goal[2] - home[2]) / WANDER_RADIUS,
                ];
                out.replies.push(movement(guid, w.position, face));
                stand(guid, &mut self.field_sim.moving, out);
                continue;
            }
            let Some(face) = heading(w.position, goal, left) else {
                continue;
            };
            for (coord, to) in w.position.iter_mut().zip(goal) {
                *coord += (to - *coord) / left * step;
            }
            ground.snap(&mut w.position);
            out.replies.push(movement(guid, w.position, face));
            walk(guid, 2, &mut self.field_sim.moving, out); // loc_walk
        }
    }

    /// (R) Field actors roam, chase within a home leash, then evade home and
    /// heal. Returning actors ignore aggro until they arrive.
    pub(super) fn tick_field(
        &mut self,
        position: [f32; 3],
        now_ms: u64,
        elapsed: u64,
        out: &mut EncounterOutcome,
        total_damage: &mut u32,
        participants: Option<&[FieldParticipant]>,
        hits: &mut Vec<(u64, u32)>,
    ) {
        let ground = self.ground();
        let solo = [self.field_participant(1, position, true)];
        let players = participants.unwrap_or(&solo);
        let shared = participants.is_some();
        let mut engaged = false;
        let mut respawned = Vec::new();
        for (i, f) in self.field_sim.enemies.iter_mut().enumerate() {
            let guid = f.actor.guid;
            if f.state.health == 0 {
                if f.respawn_at.is_some_and(|at| now_ms >= at) {
                    f.respawn_at = None;
                    f.state = enemy_state(&f.actor);
                    f.reset_motion();
                    if f.present {
                        respawned.push(i);
                    }
                }
                continue;
            }
            let b = self.behaviour_of.get(&guid).cloned().unwrap_or_default();
            let home = ground.snapped(f.actor.position);
            let leash = f.leash_radius.unwrap_or(b.leash);
            if !f.present {
                continue;
            }
            let valid = |p: &&FieldParticipant| {
                p.combat
                    && p.known_npcs
                        .as_ref()
                        .is_none_or(|known| known.contains(&guid))
                    && p.health > 0
                    && now_ms >= p.grace_until
                    && valid_position(p.position)
                    && (!shared || horizontal_distance(p.position, home) <= leash + b.reach)
                    && distance(p.position, f.state.position)
                        <= if f.motion == FieldMotion::Chasing {
                            f.aggro_range + 4.0
                        } else {
                            f.aggro_range
                        }
            };
            let player = players
                .iter()
                .filter(valid)
                .find(|p| Some(p.guid) == f.target)
                .or_else(|| {
                    players.iter().filter(valid).min_by(|a, b| {
                        distance(a.position, f.state.position)
                            .total_cmp(&distance(b.position, f.state.position))
                            .then(a.guid.cmp(&b.guid))
                    })
                });
            let position = player.map_or(home, |p| p.position);
            let delta = player.map_or(f32::MAX, |_| distance(position, f.state.position));
            f.target = player.map(|p| p.guid);
            let mitigation = player.map_or(1., |p| p.mitigation);
            // (R) A 4 m hysteresis margin avoids target flicker at aggro range.
            if !shared && now_ms < self.field_grace_until
                || horizontal_distance(f.state.position, home) > leash
                || (f.motion == FieldMotion::Chasing && delta > f.aggro_range + 4.0)
            {
                f.motion = FieldMotion::Returning;
                f.roam_goal = None;
                f.target = None;
            }
            if f.motion == FieldMotion::Idle && delta <= f.aggro_range {
                f.motion = FieldMotion::Chasing;
                f.roam_goal = None;
            }
            if self.skills.stunned(guid, now_ms) && f.motion != FieldMotion::Returning {
                engaged |= f.motion == FieldMotion::Chasing;
                continue;
            }
            if f.motion == FieldMotion::Idle
                && f.roam_goal.is_none()
                && now_ms >= f.pause_until
                && f.roam_radius > 0.0
                && b.mode != Mode::Stationary
            {
                // (R) Uniform disc samples, stable per GUID; no band/level roll.
                let angle = f.random() * std::f32::consts::TAU;
                let radius = f.random().sqrt() * f.roam_radius.min(leash);
                f.roam_goal = ground.field_position([
                    home[0] + radius * angle.cos(),
                    home[1],
                    home[2] + radius * angle.sin(),
                ]);
                if f.roam_goal.is_none() {
                    f.pause_until = now_ms.saturating_add(3000);
                }
            }
            let goal = match f.motion {
                FieldMotion::Returning => Some(home),
                FieldMotion::Chasing if delta > b.reach => Some(position),
                FieldMotion::Idle if now_ms >= f.pause_until => f.roam_goal,
                _ => None,
            };
            if let Some(goal) = goal {
                let left = distance(f.state.position, goal);
                let speed = if f.motion == FieldMotion::Returning {
                    b.run_speed
                } else {
                    b.move_speed
                };
                let stop = if f.motion == FieldMotion::Chasing {
                    (b.reach - 0.1).max(0.0)
                } else {
                    0.0
                };
                let step = (speed * elapsed.min(250) as f32 / 1000.0).min((left - stop).max(0.0));
                if left > 0.001 && step > 0.0 && b.mode != Mode::Stationary {
                    let (next, grounded) =
                        match advance(f.state.position, goal, left, step, &ground) {
                            Ok(next) => (next, true),
                            Err(next) => (next, false),
                        };
                    // Check BEFORE moving. Never take even one step outside home leash.
                    if f.motion == FieldMotion::Chasing && horizontal_distance(next, home) > leash {
                        f.motion = FieldMotion::Returning;
                        f.roam_goal = None;
                    } else if grounded {
                        let face = heading(f.state.position, goal, left).unwrap_or_default();
                        f.state.position = next;
                        out.replies.push(movement(guid, next, face));
                        walk(guid, 3, &mut self.field_sim.moving, out);
                        out.changed = true;
                    } else {
                        f.roam_goal = None;
                        f.pause_until = now_ms.saturating_add(3000);
                        if f.motion == FieldMotion::Chasing {
                            f.motion = FieldMotion::Returning;
                        }
                        stand(guid, &mut self.field_sim.moving, out);
                    }
                }
                if f.motion != FieldMotion::Chasing && distance(f.state.position, goal) <= 0.001 {
                    stand(guid, &mut self.field_sim.moving, out);
                    if f.motion == FieldMotion::Returning {
                        if f.state.health != f.actor.max_health {
                            f.state.health = f.actor.max_health;
                            out.replies
                                .push(enemy_hp(guid, f.state.health, f.actor.max_health));
                            out.changed = true;
                        }
                        self.field_sim.attacks.remove(&guid);
                        self.skills.forget_target(guid);
                        f.motion = FieldMotion::Idle;
                        f.target = None;
                        f.reset_serial = f.reset_serial.wrapping_add(1);
                    }
                    f.roam_goal = None;
                    f.pause_until = now_ms.saturating_add(1500 + (f.random() * 1500.0) as u64);
                    continue;
                }
            }
            if f.motion != FieldMotion::Chasing {
                continue;
            }
            engaged = true;
            if distance(position, f.state.position) <= b.reach {
                stand(guid, &mut self.field_sim.moving, out);
                if self
                    .field_sim
                    .attacks
                    .get(&guid)
                    .is_some_and(|last| now_ms.saturating_sub(*last) < b.attack_ms)
                {
                    continue;
                }
                self.field_sim.attacks.insert(guid, now_ms);
                out.replies.push(animation(guid, 1001, false));
                let victim = player.expect("a chasing actor has a valid target in reach");
                let prior = if shared {
                    hits.iter()
                        .filter(|(guid, _)| *guid == victim.guid)
                        .map(|(_, d)| *d)
                        .fold(0_u32, u32::saturating_add)
                } else {
                    *total_damage
                };
                let left = victim.health.saturating_sub(prior);
                let attack_damage = (f.actor.damage as f32 * mitigation).round() as u32;
                let damage = attack_damage.min(left);
                out.replies.push(attack_processed(
                    guid,
                    victim.guid,
                    damage,
                    left.saturating_sub(damage),
                    victim.max_health,
                    false,
                ));
                if shared {
                    hits.push((victim.guid, damage));
                } else {
                    *total_damage = total_damage.saturating_add(attack_damage);
                }
            }
        }
        for i in respawned {
            let f = &self.field_sim.enemies[i];
            if shared {
                // (R) One observer diff emits AddNpc/full HP, never a second
                // simulation spawn or HP correction for the same respawn.
                out.changed = true;
                continue;
            }
            out.replies.push(self.spawn(&f.actor, f.state.position, 0));
            out.replies
                .push(enemy_hp(f.actor.guid, f.state.health, f.actor.max_health));
            out.replies.push(relevance(&[f.actor.guid], false));
            out.changed = true;
        }
        if !shared && engaged != self.field_combat && !self.world_stage_fighting() {
            self.field_combat = engaged;
            out.replies.extend([
                rules(engaged),
                combat_flag(132, 0, 0, engaged),
                combat_flag(133, 0, 0, engaged),
            ]);
        }
    }
}

/// Facing from `from` toward `to` for a walk or chase step: the horizontal
/// delta divided by the full 3-D distance. `left` is the caller's
/// `distance(from, to)`, so the hot path computes the square root once.
/// Movement is opcode 125 (`0x008e5940`); its direction is a horizontal
/// vector that the client normalizes before `atan2(X, Z)` (`0x0095dae0`), so
/// the y component is zero. `None` when the spots coincide, in which case the
/// caller sends a zero facing (there is no direction to communicate).
/// Documented exceptions kept inline: the notice facing in
/// `tick_wanderers_for` (divides by 3-D distance clamped at 1 cm) and the
/// waypoint-arrival heading (divides the goal offset by `WANDER_RADIUS`);
/// `kite_turn`'s negated facing in `encounter.rs` is separate.
fn heading(from: [f32; 3], to: [f32; 3], left: f32) -> Option<[f32; 3]> {
    (left > 0.001).then(|| [(to[0] - from[0]) / left, 0.0, (to[2] - from[2]) / left])
}

/// The integrated next spot toward `goal`. `left` is the caller's
/// `distance(position, goal)` and the caller guards it above 1 mm; `Ok` is
/// snapped by `ground`, and `Err` carries the same integrated spot when ground
/// refuses it (`Ground::field_position`), so the caller can still apply its
/// "home leash before moving" rule first, exactly as the old two-step check
/// did. The bare `Ground::snap` walk in `tick_wanderers_for` stays inline
/// because it never refuses a step.
fn advance(
    position: [f32; 3],
    goal: [f32; 3],
    left: f32,
    step: f32,
    ground: &Ground,
) -> Result<[f32; 3], [f32; 3]> {
    let mut next = position;
    for (coord, to) in next.iter_mut().zip(goal) {
        *coord += (to - *coord) / left * step;
    }
    ground.field_position(next).ok_or(next)
}

/// Stop the walk animation if this actor was walking: the `moving` set is the
/// only record of the start/stop pairs, so one helper keeps them in step. The
/// movement frame is pushed by the caller before this one.
fn stand(guid: u64, moving: &mut BTreeSet<u64>, out: &mut EncounterOutcome) {
    if moving.remove(&guid) {
        out.replies.push(animation(guid, 1, true)); // loc_stand
    }
}

/// Start the walk animation if this actor was standing. `id` is 2 (`loc_walk`)
/// for a battle marker and 3 (`loc_run`) for a field actor.
fn walk(guid: u64, id: i32, moving: &mut BTreeSet<u64>, out: &mut EncounterOutcome) {
    if moving.insert(guid) {
        out.replies.push(animation(guid, id, true));
    }
}
