//! Instance-scoped enemies. Client packet layouts reuse the accepted opening encounter and
//! startup-world writers; AI, balance, respawn, GUID scoping, and rewards are reconstructed (R).

use osfr_tools::{
    abilities::{self, AbilityDefinition, AbilityOutcome, AbilityTarget, ActorKind, CombatState},
    quest_engine::QuestEvent,
    rewards::{self, ItemGrant, RewardDisplay, RewardGrant, RewardState},
    startup_world::{self, NpcSpawn},
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::Path,
};

const MAX_CONTENT_BYTES: usize = 1024 * 1024;
const MIN_TICK_MS: u64 = 100;
const MAX_STEP_MS: u64 = 250;

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ZoneInstanceKey {
    pub zone_id: u32,
    pub instance_id: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnemyMode {
    Idle,
    Patrol,
    Chase { target_guid: u64 },
    Return,
    Dead,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnemyView {
    pub guid: u64,
    pub definition: String,
    pub position: [f32; 3],
    pub health: i32,
    pub max_health: i32,
    pub mode: EnemyMode,
    pub respawn_at_ms: Option<u64>,
    pub life: u32,
}

#[derive(Clone, Debug, Default)]
pub struct EnemyOutcome {
    pub messages: Vec<Vec<u8>>,
    pub changed: bool,
}

pub struct EnemyKill {
    pub guid: u64,
    pub grant_key: String,
    pub reward_state: RewardState,
    pub reward_packets: Vec<Vec<u8>>,
    pub messages: Vec<Vec<u8>>,
    pub quest_event: QuestEvent,
}

pub struct AbilityResolution {
    pub ability: AbilityOutcome,
    pub kill: Option<EnemyKill>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnemyFile {
    schema_version: u32,
    sources: Vec<String>,
    definitions: Vec<EnemyDefinition>,
    spawn_groups: Vec<SpawnGroup>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnemyDefinition {
    id: String,
    kind: String,
    actor_kind_id: u32,
    definition_id: i32,
    model_id: i32,
    name_string_id: i32,
    texture_alias: String,
    level: Option<u32>,
    max_health: i32,
    damage: i32,
    move_speed: f32,
    aggro_radius: f32,
    leash_radius: f32,
    attack_range: f32,
    attack_cooldown_ms: u64,
    respawn_ms: u64,
    boss: bool,
    reward: EnemyReward,
    source: String,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnemyReward {
    coins: u32,
    xp: u32,
    items: Vec<EnemyItem>,
    source: String,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnemyItem {
    definition_id: i32,
    count: u32,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct SpawnGroup {
    zone_id: u32,
    stage_id: Option<u32>,
    spawns: Vec<EnemySpawn>,
    source: String,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnemySpawn {
    definition: String,
    guid: u64,
    #[serde(default)]
    actor_kind_id: Option<u32>,
    position: [f32; 3],
    #[serde(default)]
    waypoints: Vec<[f32; 3]>,
    source: String,
}

struct LiveEnemy {
    guid: u64,
    actor_kind_id: u32,
    definition: EnemyDefinition,
    home: [f32; 3],
    waypoints: Vec<[f32; 3]>,
    waypoint: usize,
    combat: CombatState,
    mode: EnemyMode,
    moving: bool,
    last_attack_ms: Option<u64>,
    respawn_at_ms: Option<u64>,
    life: u32,
}

struct LiveInstance {
    last_tick_ms: Option<u64>,
    enemies: BTreeMap<u64, LiveEnemy>,
}

pub struct EnemyEngine {
    definitions: BTreeMap<String, EnemyDefinition>,
    groups: Vec<SpawnGroup>,
    instances: BTreeMap<ZoneInstanceKey, LiveInstance>,
}

impl EnemyEngine {
    pub fn load(directory: impl AsRef<Path>) -> io::Result<Self> {
        let bytes = fs::read(directory.as_ref().join("enemies.json"))?;
        Self::from_json(&bytes)
    }

    pub fn from_json(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() > MAX_CONTENT_BYTES {
            return Err(invalid("enemy content exceeds size limit"));
        }
        let file: EnemyFile =
            serde_json::from_slice(bytes).map_err(|_| invalid("invalid enemy content JSON"))?;
        if file.schema_version != 1
            || file.sources.is_empty()
            || file.definitions.is_empty()
            || file.definitions.len() > 4096
            || file.spawn_groups.len() > 4096
        {
            return Err(invalid("invalid enemy content header"));
        }

        let mut definitions = BTreeMap::new();
        for definition in file.definitions {
            if definition.id.is_empty()
                || definition.kind.is_empty()
                || definition.actor_kind_id == 0
                || definition.definition_id <= 0
                || definition.model_id <= 0
                || definition.name_string_id <= 0
                || definition.level == Some(0)
                || definition.max_health <= 0
                || definition.damage < 0
                || !definition.move_speed.is_finite()
                || definition.move_speed <= 0.0
                || !definition.aggro_radius.is_finite()
                || definition.aggro_radius < 0.0
                || !definition.leash_radius.is_finite()
                || definition.leash_radius <= 0.0
                || !definition.attack_range.is_finite()
                || definition.attack_range < 0.0
                || definition.attack_cooldown_ms == 0
                || definition.respawn_ms == 0
                || definition.source.is_empty()
                || definition.reward.source.is_empty()
                || definition
                    .reward
                    .items
                    .iter()
                    .any(|item| item.definition_id <= 0 || item.count == 0)
            {
                return Err(invalid("invalid enemy definition"));
            }
            let id = definition.id.clone();
            if definitions.insert(id, definition).is_some() {
                return Err(invalid("duplicate enemy definition"));
            }
        }

        for group in &file.spawn_groups {
            if group.source.is_empty() || group.spawns.len() > 4096 {
                return Err(invalid("invalid enemy spawn group"));
            }
            let mut guids = BTreeSet::new();
            for spawn in &group.spawns {
                if spawn.guid == 0
                    || spawn.actor_kind_id == Some(0)
                    || spawn.source.is_empty()
                    || !valid_position(spawn.position)
                    || spawn.waypoints.iter().any(|point| !valid_position(*point))
                    || !definitions.contains_key(&spawn.definition)
                    || !guids.insert(spawn.guid)
                {
                    return Err(invalid("invalid enemy spawn"));
                }
            }
            if let Some(stage_id) = group.stage_id {
                if stage_id == 0 {
                    return Err(invalid("invalid enemy stage id"));
                }
            }
        }

        Ok(Self {
            definitions,
            groups: file.spawn_groups,
            instances: BTreeMap::new(),
        })
    }

    /// Spawn content for one zone instance. Instance-specific GUID derivation is reconstructed (R).
    pub fn enter_instance(
        &mut self,
        key: ZoneInstanceKey,
        active_stage: Option<u32>,
    ) -> io::Result<EnemyOutcome> {
        if self.instances.contains_key(&key) {
            return Ok(EnemyOutcome::default());
        }
        let mut instance = LiveInstance {
            last_tick_ms: None,
            enemies: BTreeMap::new(),
        };
        let mut outcome = EnemyOutcome::default();
        for group in self.groups.iter().filter(|group| {
            group.zone_id == key.zone_id
                && (group.stage_id.is_none() || group.stage_id == active_stage)
        }) {
            for spawn in &group.spawns {
                let definition = self.definitions[&spawn.definition].clone();
                let guid = scoped_guid(spawn.guid, key);
                if instance.enemies.contains_key(&guid) {
                    return Err(invalid("enemy GUID collision in zone instance"));
                }
                let enemy = LiveEnemy {
                    guid,
                    actor_kind_id: spawn.actor_kind_id.unwrap_or(definition.actor_kind_id),
                    combat: CombatState::new(
                        guid,
                        ActorKind::Hostile,
                        0,
                        spawn.position,
                        definition.max_health,
                        definition.max_health,
                        0,
                        0,
                    ),
                    definition,
                    home: spawn.position,
                    waypoints: spawn.waypoints.clone(),
                    waypoint: 0,
                    mode: if spawn.waypoints.is_empty() {
                        EnemyMode::Idle
                    } else {
                        EnemyMode::Patrol
                    },
                    moving: false,
                    last_attack_ms: None,
                    respawn_at_ms: None,
                    life: 0,
                };
                outcome.messages.extend(spawn_packets(&enemy)?);
                instance.enemies.insert(guid, enemy);
                outcome.changed = true;
            }
        }
        self.instances.insert(key, instance);
        Ok(outcome)
    }

    pub fn enemy(&self, key: ZoneInstanceKey, guid: u64) -> Option<EnemyView> {
        let enemy = self.instances.get(&key)?.enemies.get(&guid)?;
        Some(EnemyView {
            guid,
            definition: enemy.definition.id.clone(),
            position: enemy.combat.position,
            health: enemy.combat.health,
            max_health: enemy.combat.max_health,
            mode: enemy.mode,
            respawn_at_ms: enemy.respawn_at_ms,
            life: enemy.life,
        })
    }

    pub fn enemy_guids(&self, key: ZoneInstanceKey) -> Vec<u64> {
        self.instances
            .get(&key)
            .map(|instance| instance.enemies.keys().copied().collect())
            .unwrap_or_default()
    }

    /// Tick server-driven AI for players already filtered to this instance by the caller.
    pub fn tick(
        &mut self,
        key: ZoneInstanceKey,
        players: &mut [CombatState],
        now_ms: u64,
    ) -> io::Result<EnemyOutcome> {
        if players.iter().any(|player| {
            player.kind != ActorKind::Player
                || player.guid == 0
                || !valid_position(player.position)
                || player.max_health <= 0
                || player.health < 0
                || player.health > player.max_health
        }) {
            return Err(invalid("invalid enemy tick player"));
        }
        let instance = self
            .instances
            .get_mut(&key)
            .ok_or_else(|| invalid("unknown enemy zone instance"))?;
        let elapsed = instance
            .last_tick_ms
            .map_or(MIN_TICK_MS, |last| now_ms.saturating_sub(last));
        if elapsed < MIN_TICK_MS {
            return Ok(EnemyOutcome::default());
        }
        instance.last_tick_ms = Some(now_ms);

        let mut outcome = EnemyOutcome::default();
        let mut pending_damage = BTreeMap::<u64, i32>::new();
        let enemy_guids: Vec<_> = instance.enemies.keys().copied().collect();
        for guid in enemy_guids {
            let enemy = instance.enemies.get_mut(&guid).unwrap();
            if enemy.mode == EnemyMode::Dead {
                if enemy.respawn_at_ms.is_some_and(|due| now_ms >= due) {
                    enemy.combat.health = enemy.combat.max_health;
                    enemy.combat.position = enemy.home;
                    enemy.respawn_at_ms = None;
                    enemy.last_attack_ms = None;
                    enemy.moving = false;
                    enemy.mode = if enemy.waypoints.is_empty() {
                        EnemyMode::Idle
                    } else {
                        EnemyMode::Patrol
                    };
                    enemy.life = enemy.life.saturating_add(1);
                    outcome.messages.extend(spawn_packets(enemy)?);
                    outcome.changed = true;
                }
                continue;
            }

            let target_index = match enemy.mode {
                EnemyMode::Chase { target_guid } => players
                    .iter()
                    .position(|player| player.guid == target_guid && player.health > 0)
                    .filter(|index| {
                        let player = &players[*index];
                        distance(enemy.home, player.position) <= enemy.definition.leash_radius
                            && distance(enemy.home, enemy.combat.position)
                                <= enemy.definition.leash_radius
                    }),
                EnemyMode::Return => None,
                _ => players
                    .iter()
                    .enumerate()
                    .filter(|(_, player)| player.health > 0)
                    .filter(|(_, player)| {
                        distance(enemy.combat.position, player.position)
                            <= enemy.definition.aggro_radius
                            && distance(enemy.home, player.position)
                                <= enemy.definition.leash_radius
                    })
                    .min_by(|(_, left), (_, right)| {
                        distance(enemy.combat.position, left.position)
                            .total_cmp(&distance(enemy.combat.position, right.position))
                            .then(left.guid.cmp(&right.guid))
                    })
                    .map(|(index, _)| index),
            };

            if let Some(index) = target_index {
                let target = &players[index];
                enemy.mode = EnemyMode::Chase {
                    target_guid: target.guid,
                };
                let delta = distance(enemy.combat.position, target.position);
                if delta > enemy.definition.attack_range {
                    let (position, heading) = step_toward(
                        enemy.combat.position,
                        target.position,
                        enemy.definition.move_speed,
                        elapsed,
                        (enemy.definition.attack_range - 0.1).max(0.0),
                    );
                    enemy.combat.position = position;
                    outcome
                        .messages
                        .push(movement_packet(enemy.guid, position, heading));
                    if !enemy.moving {
                        outcome.messages.push(animation_packet(enemy.guid, 3, true));
                        enemy.moving = true;
                    }
                    outcome.changed = true;
                }
                if distance(enemy.combat.position, target.position) <= enemy.definition.attack_range
                {
                    if enemy.moving {
                        outcome.messages.push(animation_packet(enemy.guid, 1, true));
                        enemy.moving = false;
                    }
                    if !enemy.last_attack_ms.is_some_and(|last| {
                        now_ms.saturating_sub(last) < enemy.definition.attack_cooldown_ms
                    }) {
                        enemy.last_attack_ms = Some(now_ms);
                        outcome
                            .messages
                            .push(animation_packet(enemy.guid, 1001, false));
                        let already = pending_damage.get(&target.guid).copied().unwrap_or(0);
                        let damage = enemy
                            .definition
                            .damage
                            .min(target.health.saturating_sub(already))
                            .max(0);
                        let after = target.health.saturating_sub(already).saturating_sub(damage);
                        outcome.messages.push(attack_processed_packet(
                            enemy.guid,
                            target.guid,
                            damage,
                            after,
                            target.max_health,
                        ));
                        pending_damage.insert(target.guid, already.saturating_add(damage));
                        outcome.changed = true;
                    }
                }
            } else if matches!(enemy.mode, EnemyMode::Chase { .. }) {
                enemy.mode = EnemyMode::Return;
            }

            if enemy.mode == EnemyMode::Return {
                let delta = distance(enemy.combat.position, enemy.home);
                if delta > 0.001 {
                    let (position, heading) = step_toward(
                        enemy.combat.position,
                        enemy.home,
                        enemy.definition.move_speed,
                        elapsed,
                        0.0,
                    );
                    enemy.combat.position = if distance(position, enemy.home) < 0.001 {
                        enemy.home
                    } else {
                        position
                    };
                    outcome.messages.push(movement_packet(
                        enemy.guid,
                        enemy.combat.position,
                        heading,
                    ));
                    if !enemy.moving {
                        outcome.messages.push(animation_packet(enemy.guid, 3, true));
                        enemy.moving = true;
                    }
                    outcome.changed = true;
                }
                if distance(enemy.combat.position, enemy.home) <= 0.001 {
                    enemy.mode = EnemyMode::Idle;
                    if enemy.moving {
                        outcome.messages.push(animation_packet(enemy.guid, 1, true));
                        enemy.moving = false;
                    }
                }
            } else if matches!(enemy.mode, EnemyMode::Patrol) && target_index.is_none() {
                if let Some(waypoint) = enemy.waypoints.get(enemy.waypoint).copied() {
                    if distance(enemy.combat.position, waypoint) <= 0.1 {
                        enemy.waypoint = (enemy.waypoint + 1) % enemy.waypoints.len();
                    }
                    let destination = enemy.waypoints[enemy.waypoint];
                    let (position, heading) = step_toward(
                        enemy.combat.position,
                        destination,
                        enemy.definition.move_speed,
                        elapsed,
                        0.0,
                    );
                    enemy.combat.position = position;
                    outcome
                        .messages
                        .push(movement_packet(enemy.guid, position, heading));
                    if !enemy.moving {
                        outcome.messages.push(animation_packet(enemy.guid, 3, true));
                        enemy.moving = true;
                    }
                    outcome.changed = true;
                }
            }
        }

        for (guid, damage) in pending_damage {
            if damage <= 0 {
                continue;
            }
            if let Some(player) = players.iter_mut().find(|player| player.guid == guid) {
                player.health = player.health.saturating_sub(damage).max(0);
                outcome
                    .messages
                    .push(player_health_packet(player.health, player.max_health));
            }
        }
        Ok(outcome)
    }

    /// Resolve a real ability against the enemy state, then stage one keyed reward and kill event.
    pub fn use_ability_on_enemy(
        &mut self,
        key: ZoneInstanceKey,
        player: &mut CombatState,
        enemy_guid: u64,
        ability: &AbilityDefinition,
        now_ms: u64,
        reward_state: &RewardState,
    ) -> io::Result<AbilityResolution> {
        if player.kind != ActorKind::Player || player.guid == 0 {
            return Err(invalid("enemy ability caster must be a player"));
        }
        let instance = self
            .instances
            .get_mut(&key)
            .ok_or_else(|| invalid("unknown enemy zone instance"))?;
        let enemy = instance
            .enemies
            .get(&enemy_guid)
            .ok_or_else(|| invalid("unknown enemy"))?;
        let before = enemy.combat.health;
        let definition = enemy.definition.clone();
        let actor_kind_id = enemy.actor_kind_id;
        let life = enemy.life;
        let mut next_player = player.clone();
        let mut next_enemy = enemy.combat.clone();
        let ability_result = abilities::use_ability(
            &mut next_player,
            AbilityTarget::Actor(&mut next_enemy),
            ability,
            now_ms,
        );
        if ability_result.failure.is_some() {
            return Ok(AbilityResolution {
                ability: ability_result,
                kill: None,
            });
        }

        let kill = if before > 0 && next_enemy.health == 0 {
            // (R) Grant keys scope one award to one enemy life in one zone instance.
            let grant_key = format!(
                "enemy:{}:{}:{}:{}",
                key.zone_id, key.instance_id, enemy_guid, life
            );
            let mut job_xp = BTreeMap::new();
            if definition.reward.xp > 0 {
                if next_player.job_profile <= 0 {
                    return Err(invalid("enemy XP reward requires an active job"));
                }
                job_xp.insert(next_player.job_profile, definition.reward.xp);
            }
            let grant = RewardGrant {
                coins: definition.reward.coins,
                items: definition
                    .reward
                    .items
                    .iter()
                    .map(|item| ItemGrant {
                        definition_id: item.definition_id,
                        count: item.count,
                    })
                    .collect(),
                job_xp,
                sc: 0,
                display: RewardDisplay {
                    source_guid: enemy_guid,
                    player_guid: next_player.guid,
                    reward_kind: 0,
                    icon_id: 0,
                    name_id: 0,
                    rows: Vec::new(),
                },
            };
            let (next_rewards, reward_packets) =
                rewards::apply_grant(reward_state, &grant, &grant_key)?;
            let mut messages = vec![despawn_packet(enemy_guid)];
            if definition.boss {
                messages.push(boss_display_packet(enemy_guid, false));
            }
            Some(EnemyKill {
                guid: enemy_guid,
                grant_key,
                reward_state: next_rewards,
                reward_packets,
                messages,
                quest_event: QuestEvent::KillActorKind {
                    actor_kind_id,
                    count: 1,
                },
            })
        } else {
            None
        };

        *player = next_player;
        let enemy = instance.enemies.get_mut(&enemy_guid).unwrap();
        enemy.combat = next_enemy;
        if kill.is_some() {
            enemy.mode = EnemyMode::Dead;
            enemy.respawn_at_ms = Some(now_ms.saturating_add(definition.respawn_ms));
            enemy.moving = false;
        }
        Ok(AbilityResolution {
            ability: ability_result,
            kill,
        })
    }
}

fn valid_position(position: [f32; 3]) -> bool {
    position
        .iter()
        .all(|value| value.is_finite() && value.abs() <= 1_000_000.0)
}

fn distance(left: [f32; 3], right: [f32; 3]) -> f32 {
    left.iter()
        .zip(right)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f32>()
        .sqrt()
}

fn step_toward(
    from: [f32; 3],
    to: [f32; 3],
    speed: f32,
    elapsed_ms: u64,
    stop_distance: f32,
) -> ([f32; 3], [f32; 3]) {
    let delta = distance(from, to);
    if delta <= 0.0 {
        return (from, [0.0; 3]);
    }
    let heading = [(to[0] - from[0]) / delta, 0.0, (to[2] - from[2]) / delta];
    let step =
        (speed * elapsed_ms.min(MAX_STEP_MS) as f32 / 1000.0).min((delta - stop_distance).max(0.0));
    let mut next = from;
    for index in 0..3 {
        next[index] += (to[index] - from[index]) / delta * step;
    }
    (next, heading)
}

fn scoped_guid(template: u64, key: ZoneInstanceKey) -> u64 {
    if key.instance_id == 0 {
        template
    } else {
        // (R) Private instance actor ids are stable within a process and distinct from public ids.
        template ^ key.instance_id.rotate_left(17) ^ ((key.zone_id as u64) << 32)
    }
}

fn spawn_packets(enemy: &LiveEnemy) -> io::Result<Vec<Vec<u8>>> {
    // (R) Unmapped AddNpc fields use the existing neutral defaults; the exact body order is client-read.
    let mut actor = NpcSpawn::default();
    actor.actor_id = enemy.guid;
    actor.name_string_id = enemy.definition.name_string_id;
    actor.model_id = enemy.definition.model_id;
    actor.scale = 3.0;
    actor.position = [
        enemy.combat.position[0],
        enemy.combat.position[1],
        enemy.combat.position[2],
        1.0,
    ];
    actor.rotation = [0.0, 0.0, 0.0, 1.0];
    actor.disposition = 0;
    actor.texture_alias = enemy.definition.texture_alias.clone();
    actor.npc_name = enemy.definition.kind.clone();
    actor.speed = enemy.definition.move_speed;
    actor.walk_animation_id = 1;
    actor.run_animation_id = 3;
    actor.stand_animation_id = 1;
    let framed = startup_world::build_add_npc(&actor);
    if framed.len() < 7
        || framed[..3] != [5, 0, 1]
        || u32::from_le_bytes(framed[3..7].try_into().unwrap()) as usize != framed.len() - 7
    {
        return Err(invalid("AddNpc writer returned an invalid tunnel"));
    }
    let mut packets = vec![
        framed[7..].to_vec(),
        actor_health_packet(enemy.guid, enemy.combat.health, enemy.combat.max_health),
    ];
    if enemy.definition.boss {
        packets.push(boss_display_packet(enemy.guid, true));
    }
    Ok(packets)
}

fn header(family: u16, sub: u16) -> Vec<u8> {
    [family.to_le_bytes(), sub.to_le_bytes()].concat()
}

fn actor_health_packet(guid: u64, health: i32, max_health: i32) -> Vec<u8> {
    let mut bytes = header(35, 5);
    bytes.extend(guid.to_le_bytes());
    for value in [health, max_health, 0] {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

fn player_health_packet(health: i32, max_health: i32) -> Vec<u8> {
    let mut bytes = header(38, 1);
    bytes.extend(health.to_le_bytes());
    bytes.extend(max_health.to_le_bytes());
    bytes
}

fn attack_processed_packet(
    attacker: u64,
    target: u64,
    damage: i32,
    target_health: i32,
    target_max_health: i32,
) -> Vec<u8> {
    // Client reader 00a2a910 consumes three GUID pairs and fixed result fields.
    let mut bytes = header(32, 7);
    for guid in [attacker, attacker, target] {
        bytes.extend(guid.to_le_bytes());
    }
    for value in [damage, target_max_health, 0] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend([0, 0]);
    for value in [0, target_health] {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

/// Encode one 35/8 animation message (native reader `00908a50`). T488/F796: the bytes come from
/// the one writer, `world_events::piece_052::animation_packet`, so the two sites cannot drift.
pub(crate) fn animation_packet(guid: u64, animation_id: i32, base: bool) -> Vec<u8> {
    crate::world_events::piece_052::animation_packet(guid, animation_id, base)
}

pub(crate) fn movement_packet(guid: u64, position: [f32; 3], heading: [f32; 3]) -> Vec<u8> {
    // (R) Opcode 125 layout and flags match the accepted opening EncounterEngine writer.
    let mut bytes = 125_u16.to_le_bytes().to_vec();
    bytes.extend(guid.to_le_bytes());
    for value in position.into_iter().chain(heading) {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend([1, 0]);
    bytes
}

fn despawn_packet(guid: u64) -> Vec<u8> {
    // (R) Keep the byte shape used by the accepted opening encounter's 35/3 actor removal.
    let mut bytes = header(35, 3);
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(guid.to_le_bytes());
    bytes.extend([0; 17]);
    bytes
}

fn boss_display_packet(guid: u64, enabled: bool) -> Vec<u8> {
    let mut bytes = header(32, 9);
    bytes.extend(guid.to_le_bytes());
    bytes.push(u8::from(enabled));
    bytes
}
