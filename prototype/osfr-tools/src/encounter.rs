//! Local opening encounters over community-documented native packets.
//!
//! The supplied map, models and story are reference facts. Actor placements,
//! health, waves, damage, timing and rewards are explicitly reconstructed local
//! rules. Callers stage a clone, save it with inventory/progression, then commit.

use crate::{
    abilities::{EffectRecipient, JobEffect},
    character_store::CharacterState,
    power_ups::{self, PowerUps, Rng},
    terrain::Terrain,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::Path,
    sync::Arc,
};

mod arena;
mod battle_drops;
mod combat_scale;
mod field;
pub mod field_rewards;
mod power_up_use;
mod revive;
mod shared;
mod skills;
use self::arena::{ArenaSim, ArenaTick};
use self::combat_scale::CombatScale;
pub(crate) use self::field::level_refusal;
use self::revive::{AUTO_REVIVE_MS, respawn_hide};
use self::shared::FieldSim;
pub use self::shared::{SharedFieldAction, SharedFieldState};
use self::skills::{SkillBuff, SkillDot, SkillEffects};
use crate::abilities::effect_packets::encode_play_composite_effect as composite_effect;

/// Upper bound for saved player health (level 20 is about 7,135, 8,205 for a Brawler).
const PLAYER_HEALTH_CAP: u32 = 20_000;

/// Combat job identity (R; the owner's design, 2026-10-15): Brawler is the
/// berserker tank (video level 20: Brawler 9821 vs Archer 8540 health),
/// Ninja hits quick and crits often, Medic and Wizard have small health pools
/// (Medic heals, Wizard hits hard from range). Profile ids from the profile
/// definitions: 2 Ninja, 11 Medic, 12 Wizard, 32 Warrior, 35 Archer, 43 Brawler.
/// Returns (health multiplier, crit chance %).
fn job_identity(profile: i32) -> (f32, u32) {
    match profile {
        43 => (1.15, 8),
        32 => (1.0, 5),
        35 => (1.0, 10),
        2 => (0.9, 20),
        11 => (0.9, 5),
        12 => (0.85, 5),
        _ => (1.0, 0),
    }
}

/// Critical hits deal x1.5 (video: an 83 hit critted for 124; client "50 Crit Mult %").
const CRIT_NUMERATOR: u32 = 3;
const CRIT_DENOMINATOR: u32 = 2;

/// Lowest job level a battle difficulty admits (wiki: 1 open, 2 recommended
/// 3+, then 5 / 9 / 13); its enemies scale like a player of that level.
fn difficulty_level(difficulty: i32) -> u32 {
    match difficulty {
        ..=1 => 1,
        2 => 3,
        3 => 5,
        4 => 9,
        _ => 13,
    }
}
/// Energy for job abilities (R): the HUD shows 100/100; regeneration of
/// 2 per 0.5 s follows the community reference's 4 per second.
const ENERGY_MAX: i32 = 100;
const BATTLE_JOB_XP: u32 = 50;
/// Job type of combat jobs in the profile definitions (field 3).
const COMBAT_PROFILE_TYPE: i32 = 2;
const ENERGY_STEP: i32 = 2;
const ENERGY_STEP_MS: u64 = 500;
pub const GUIDE_GUID: u64 = 0x4f50_454e_0000_f001;
/// Power-up pickup actors get `"PWUP"` plus a per-player serial (R). They are
/// personal to the killer, so the serial only has to be unique per engine.
const PICKUP_GUID_BASE: u64 = 0x5057_5550_0000_0000;
/// SFX_PowerUp_Pickup composite (ActorCompositeEffectDefinitions.xml 15509,
/// Combat Power-ups note 1.1); played at the pickup when it is collected.
const POWER_UP_PICKUP_EFFECT: i32 = 15509;
/// The client's own tutorial line, shown when a second stored kind is refused.
const ONE_POWER_UP_TEXT: &str = "You can have one power up active at a time.";
/// Local reconstructed objective award. The original encounter scoring table
/// was not recovered, so the completed objective's value doubles as the won
/// total; a loss awards nothing and no per-kill points are invented.
const OBJECTIVE_AWARD_SCORE: i32 = 5000;
const EXIT: i32 = 101;
const RETRY: i32 = 102;
const ATTACK: i32 = 103;
const START: i32 = 104;

fn invalid(s: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, s)
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn valid_position(p: [f32; 3]) -> bool {
    p.iter().all(|v| v.is_finite() && v.abs() <= 1_000_000.0)
}
fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}
fn horizontal_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    (a[0] - b[0]).hypot(a[2] - b[2])
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ZoneTransition {
    pub zone_id: u32,
    pub name: String,
    pub sky: String,
    pub geometry_id: i32,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub tutorial: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EncounterPhase {
    Loading,
    Fighting,
    Won,
    Lost,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EnemyState {
    pub guid: u64,
    pub health: u32,
    pub position: [f32; 3],
    /// A summoned minion that has not been called in yet.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub dormant: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ActiveEncounter {
    pub encounter_id: u32,
    pub instance_id: i32,
    pub phase: EncounterPhase,
    pub return_location: CharacterState,
    pub enemies: Vec<EnemyState>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EncounterState {
    pub version: u32,
    pub content_sha256: String,
    pub active: Option<ActiveEncounter>,
    pub world_stage: u32,
    pub world_enemies: Vec<EnemyState>,
    pub player_health: u32,
    pub rewarded: BTreeSet<u32>,
    pub next_instance_id: i32,
}

#[derive(Clone, Debug, Deserialize)]
struct Actor {
    guid: u64,
    definition_id: i32,
    model_id: i32,
    /// Locale id for the client nameplate; 0 shows the literal `name`, so an
    /// actor with no visible name leaves both empty (pickups below).
    name_id: i32,
    name: String,
    texture_alias: String,
    position: [f32; 3],
    max_health: u32,
    damage: u32,
    #[serde(default = "one")]
    level: u32,
    boss: bool,
    #[serde(default)]
    stages: Vec<u32>,
    /// Summoned minion: absent until the encounter's summoner drops below
    /// this percentage of its health, then appears in a puff (the owner's field
    /// notes on the Troll Summoner). Not counted toward the goal.
    #[serde(default)]
    summon_below_percent: Option<u32>,
    /// Skittish (the Troll Summoner, after the owner's notes and design): fights
    /// in melee while its summoned minions stand, and when alone runs from the
    /// player, stops to catch its breath, then runs again (R).
    #[serde(default)]
    skittish: bool,
    /// AddNpc scale; the summoned "small trolls" are shrunken troll models.
    #[serde(default)]
    scale: Option<f32>,
    /// A `behaviours.json` preset name, or `{"preset": ..., overrides}`.
    #[serde(default)]
    behaviour: Option<serde_json::Value>,
    /// Drop table id in `power_ups.json`; absent means this enemy drops nothing
    /// (training dummies and summoned minions stay off the table so they cannot
    /// be farmed).
    #[serde(default)]
    power_up_table: Option<String>,
}

/// Summon presentation (R): the arm-swirl cast and the client's troll spawn
/// puff. troll_shaman.adr has com_cast_01/02 and com_spawn but no
/// com_cast_special_* (the first native run looped on the missing 1131), so
/// the summon uses com_cast_02, new minions play com_spawn, and the puff is
/// PFX_bones_yellow_troll-spawn (ActorCompositeEffectDefinitions.xml 16337).
/// The live game's choices are not recovered.
const SUMMON_ANIMATION: i32 = 1112; // com_cast_02
const SPAWN_ANIMATION: i32 = 1404; // com_spawn
const SUMMON_EFFECT: i32 = 16337;
/// How an enemy moves and fights. Named presets live in `behaviours.json`;
/// an enemy record picks one with `"behaviour": "skittish"` or adjusts one
/// with `{"preset": "melee", "move_speed": 4.0}`. Without either it is
/// `melee` (or `skittish` for the older `skittish: true` flag).
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// Chase and swing.
    Melee,
    /// Fights beside its summoned minions; alone it runs, rests, runs again
    /// (the Troll Summoner, the owner's design).
    Skittish,
    /// Never moves; swings at players within reach.
    Stationary,
}

/// (R) Every default is this build's original constant, so content without
/// presets behaves as before.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Behaviour {
    pub mode: Mode,
    /// Chase speed in m/s.
    pub move_speed: f32,
    /// Melee reach in metres.
    pub reach: f32,
    /// Time between attacks in ms.
    pub attack_ms: u64,
    /// In arenas, stops chasing a player farther than this (m).
    pub chase_limit: f32,
    /// Skittish cycle when alone: run for `run_ms` at `run_speed` (slower
    /// than a running player), then rest `rest_ms` playing `rest_animation`
    /// (com_stunned, AnimationTypes.xml 1401) without attacking - the window
    /// to hit it. Stays within `leash` m of its spawn, sliding along that edge
    /// when cornered, and only runs while the player is within `run_from` m
    /// (farther away it stands and watches, and that time does not tire it).
    /// It follows the terrain grid (`--terrain`); without one it takes the
    /// player's ground height (a fixed height sank it under the slopes at the
    /// clearing's edge in run 053415 - "phasing out of reality").
    pub run_ms: u64,
    pub rest_ms: u64,
    pub run_speed: f32,
    pub leash: f32,
    pub run_from: f32,
    pub rest_animation: i32,
    /// Free text for content authors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl Default for Behaviour {
    fn default() -> Self {
        Self {
            mode: Mode::Melee,
            move_speed: 3.0,
            reach: 2.5,
            attack_ms: 1000,
            chase_limit: 40.0,
            run_ms: 4000,
            rest_ms: 3000,
            run_speed: 3.5,
            leash: 12.0,
            run_from: 10.0,
            rest_animation: 1401,
            note: None,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BehaviourFile {
    #[allow(dead_code)]
    schema_version: u32,
    presets: BTreeMap<String, Behaviour>,
    /// Evidence entries for the content checker.
    #[serde(default)]
    #[allow(dead_code)]
    source: Vec<serde_json::Value>,
}

/// An enemy's behaviour: its record's `behaviour` (a preset name, or an
/// object with an optional `preset` plus overrides) over the default preset.
fn resolve_behaviour(
    actor: &Actor,
    presets: &BTreeMap<String, Behaviour>,
) -> Result<Behaviour, String> {
    let named = |name: &str| {
        presets.get(name).cloned().ok_or(format!(
            "{} ({}): unknown behaviour preset '{name}'",
            actor.name, actor.guid
        ))
    };
    let base = if actor.skittish {
        presets.get("skittish").cloned().unwrap_or(Behaviour {
            mode: Mode::Skittish,
            ..Behaviour::default()
        })
    } else {
        presets.get("melee").cloned().unwrap_or_default()
    };
    match &actor.behaviour {
        None => Ok(base),
        Some(serde_json::Value::String(name)) => named(name),
        Some(serde_json::Value::Object(fields)) => {
            let start = match fields.get("preset") {
                Some(serde_json::Value::String(name)) => named(name)?,
                Some(_) => return Err(format!("{}: behaviour preset must be a name", actor.name)),
                None => base,
            };
            let mut merged = serde_json::to_value(start).map_err(|e| e.to_string())?;
            for (key, value) in fields.iter().filter(|(key, _)| *key != "preset") {
                merged[key] = value.clone();
            }
            serde_json::from_value(merged)
                .map_err(|e| format!("{} ({}): behaviour {e}", actor.name, actor.guid))
        }
        Some(_) => Err(format!(
            "{}: behaviour must be a preset name or an object",
            actor.name
        )),
    }
    .and_then(|b| {
        b.check()
            .map(|()| b)
            .map_err(|e| format!("{} ({}): {e}", actor.name, actor.guid))
    })
}

impl Behaviour {
    /// Any value is tunable, but not nonsense (negative, infinite, or a
    /// reach past the arena's chase limit).
    fn check(&self) -> Result<(), String> {
        let distances = [
            ("move_speed", self.move_speed),
            ("reach", self.reach),
            ("chase_limit", self.chase_limit),
            ("run_speed", self.run_speed),
            ("leash", self.leash),
            ("run_from", self.run_from),
        ];
        for (name, value) in distances {
            if !value.is_finite() || value < 0.0 || value > 1000.0 {
                return Err(format!("behaviour {name} {value} must be 0..1000"));
            }
        }
        if self.reach > self.chase_limit {
            return Err(format!(
                "behaviour reach {} is beyond its chase_limit {}",
                self.reach, self.chase_limit
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Deserialize)]
struct Dungeon {
    id: u32,
    title_id: i32,
    description_id: i32,
    objective_id: i32,
    /// Start-screen picture: MiniGameInfo.IconId (41/114; -1 = default).
    #[serde(default = "no_icon")]
    icon_id: i32,
    zone: ZoneTransition,
    /// The cave's Darkthorne guide (Start/Retry/Exit menu). Overworld battles
    /// have none: they start from the start screen's Go and the first swing.
    #[serde(default)]
    guide: Option<Actor>,
    enemies: Vec<Actor>,
    reward_coins: u32,
    reward_items: Vec<(i32, u32)>,
    #[serde(default)]
    prizes: Vec<Prize>,
}
#[derive(Clone, Deserialize)]
struct Content {
    schema_version: u32,
    template_file: String,
    template_sha256: String,
    world: ZoneTransition,
    world_enemies: Vec<Actor>,
    dungeons: Vec<Dungeon>,
    #[serde(skip)]
    template: Vec<u8>,
    /// (definition, guid, home) per field enemy row: the quest pin's lookup, which must
    /// outlive `take_shared_fields` moving the live rows into the shared field.
    #[serde(skip)]
    field_sites: Vec<(i32, u64, [f32; 3])>,
}

/// Overworld enemy that patrols outside the opening's story stages (e.g. the
/// Hooligan Delinquents south of Jones). Loaded from world content, never saved.
/// A spawn group supplies shared policy and the individual respawn timer; the
/// actor keeps its own GUID, home and roam disc. Spawn, hit, HP and defeat use
/// the same packets as the opening's world enemies.
#[derive(Clone, Debug, Deserialize)]
struct FieldActor {
    #[serde(flatten)]
    actor: Actor,
    #[serde(default)]
    respawn_seconds: Option<u32>,
    /// The enemy chases a player within this range and otherwise walks home.
    #[serde(default)]
    aggro_range: Option<f32>,
    #[serde(default)]
    level_band: Option<[u32; 2]>,
    #[serde(default)]
    spawn_area: Option<SpawnArea>,
    #[serde(default)]
    leash_radius: Option<f32>,
    #[serde(default)]
    region: Option<String>,
    #[serde(default)]
    zone_id: u32,
    /// The enemy exists only while one of these quests is active (the owner's
    /// field notes: overworld enemies belong to active quests, like the
    /// opening's stage-gated wolves). Empty means always present.
    #[serde(default)]
    quest_ids: Vec<u32>,
    #[serde(default)]
    #[allow(dead_code)]
    source: serde_json::Value,
}

/// One always-on population. The group owns the policy shared by its explicit
/// placed actors; each member keeps its own GUID, home position and spawn disc.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FieldSpawnGroup {
    id: String,
    zone_id: u32,
    region: String,
    respawn_seconds: u32,
    aggro_range: f32,
    #[serde(default)]
    level_band: Option<[u32; 2]>,
    leash_radius: f32,
    behaviour: serde_json::Value,
    members: Vec<u64>,
    source: Vec<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize)]
struct SpawnArea {
    id: String,
    centre: [f32; 3],
    radius: f32,
    count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FieldMotion {
    Idle,
    Chasing,
    Returning,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FieldContent {
    schema_version: u32,
    /// (R) Knockout "Revive here" price (`revive.rs`); absent: the retail 1,000.
    #[serde(default)]
    paid_revive: Option<revive::PaidRevive>,
    enemies: Vec<FieldActor>,
    #[serde(default)]
    spawn_groups: Vec<FieldSpawnGroup>,
}

#[derive(Clone, Debug)]
struct FieldEnemy {
    actor: Actor,
    respawn_ms: u64,
    aggro_range: f32,
    quest_ids: Vec<u32>,
    present: bool,
    state: EnemyState,
    respawn_at: Option<u64>,
    leash_radius: Option<f32>,
    roam_radius: f32,
    motion: FieldMotion,
    roam_goal: Option<[f32; 3]>,
    pause_until: u64,
    seed: u64,
    target: Option<u64>,
    reset_serial: u64,
}

/// Overworld battle started from a spawner (red crossed swords): clicking it
/// opens the minigame start screen (41/114 details, not launched); Go sends
/// 41/108 and the player is zoned into the battle's arena. Titles, descriptions and goals are
/// client locale ids (e.g. Encounter 96 "Troll Summoner Madness!" 42056/42059/42071).
#[derive(Clone, Debug, Deserialize)]
struct FieldBattle {
    id: u32,
    title_id: i32,
    description_id: i32,
    objective_id: i32,
    /// Start-screen difficulty 1..5; 3/4/5 show "Requires a level 5/9/13
    /// combat job" (locale 4199-4201).
    difficulty: i32,
    #[serde(default)]
    required_combat_level: u32,
    spawner: Actor,
    /// The instanced arena the battle runs in (e.g. sg_random_encounter_clearing).
    zone: ZoneTransition,
    /// Start-screen picture (MiniGameInfo.IconId); e.g. Images.txt 4543
    /// icon_ui_combatstart_panelgraphic_trollsummonermadness.
    #[serde(default = "no_icon")]
    icon_id: i32,
    #[serde(default)]
    prizes: Vec<Prize>,
    /// Chance items on a win, beside the prize wheel (`battle_drops.rs`).
    #[serde(default)]
    drops: Vec<battle_drops::BattleDrop>,
    /// Overhead crossed-swords icon on the spawner: an Images.txt id sent as
    /// AddNpc NameplateImageId (1345 icon_minigame_combat_encounter_start_64;
    /// 11968-11970 the easy/medium/hard variants). Which one the live game
    /// sent is not known (R).
    #[serde(default = "default_battle_icon")]
    nameplate_image_id: i32,
    enemies: Vec<Actor>,
    #[serde(default)]
    reward_coins: u32,
    #[serde(default)]
    #[allow(dead_code)]
    source: serde_json::Value,
}

/// The largest quantity one battle prize row may preview. The content validator
/// enforces it; the stack preflight additionally caps each row at the item's
/// own `max_stack`.
pub const MAX_PRIZE_QUANTITY: u32 = 100;

/// A prize shown on the battle start screen and won at random on victory
/// (the battle's "prize wheel"; the owner: prizes spin like a loot box). Item
/// definition, name and icon ids come from the client item catalogue.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Prize {
    item_id: i32,
    name_id: i32,
    icon_id: i32,
    #[serde(default = "one")]
    quantity: u32,
}

fn one() -> u32 {
    1
}

/// A MiniGameInfo reward bundle (native reader 008e7930): flag, coin/xp and
/// other scalars (52 bytes, zero), bundle icon and name, the entries, a
/// trailing i32. Each prize is a type-1 Item entry (base reader 0103c460):
/// hidden, icon, tint, name, quantity, item id, param, empty string, text
/// colour, members-only. Empty, the bundle is 69 bytes.
fn prize_bundle(prizes: &[Prize]) -> Vec<u8> {
    let mut b = vec![0_u8; 61];
    b.extend((prizes.len() as i32).to_le_bytes());
    for p in prizes {
        b.extend(1_i32.to_le_bytes()); // entry type: Item
        b.push(0); // hidden
        for v in [
            p.icon_id,
            0,
            p.name_id,
            p.quantity as i32,
            p.item_id,
            0,
            0,
            0,
        ] {
            b.extend(v.to_le_bytes());
        }
        b.push(0); // members only
    }
    b.extend(0_i32.to_le_bytes());
    b
}

fn no_icon() -> i32 {
    -1
}

fn default_battle_icon() -> i32 {
    11970
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FieldBattleFile {
    schema_version: u32,
    battles: Vec<FieldBattle>,
}

#[derive(Clone, Debug, Default)]
pub struct EncounterReward {
    pub coins: u32,
    pub items: Vec<(i32, u32)>,
    /// Chance drops, granted only if the bag has room; never refuse the rest.
    pub bonus_items: Vec<(i32, u32)>,
    /// Experience for the active (combat) job: battles level combat jobs.
    pub job_xp: u32,
}
#[derive(Debug, Default)]
pub struct EncounterOutcome {
    pub replies: Vec<Vec<u8>>,
    pub changed: bool,
    pub zoning: Option<ZoneTransition>,
    pub rewards: Vec<EncounterReward>,
    pub completed: Option<u32>,
    /// The encounter/dungeon id reported on the **win transition** for regional
    /// `CompleteDungeon` quest credit. `completed` still marks the results Close and
    /// drives the opening story destination; do not use one field for both.
    pub win_completed: Option<u32>,
    /// Positive definition ids defeated here; `0` names no quest target.
    pub defeated: Vec<i32>,
    pub feedback: Option<String>,
    /// A group heal power-up was picked up (the engine heals only its own
    /// player; the gateway heals the rest of the group, once per kill).
    pub group_heal: Option<GroupHeal>,
}

/// Percent of max health for the rest of the picker's group, and the kill whose
/// drop it was: the gateway applies at most one group heal per kill (R).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GroupHeal {
    pub percent: u32,
    pub kill: (u64, u64),
}

#[derive(Clone)]
pub struct EncounterEngine {
    content: Arc<Content>,
    state: EncounterState,
    last_attack: Option<u64>,
    last_tick: Option<u64>,
    /// The whole field simulation; swapped with the shared field as one value.
    field_sim: FieldSim,
    // Player energy for job abilities (session only; full on login) and its
    // next regeneration step, plus each ability slot's cooldown end.
    energy: i32,
    energy_regen_at: u64,
    /// Damage, health and tier levels from `combat_scale.json`.
    scale: CombatScale,
    // The player's max health and weapon damage multiplier at the active
    // job's level (set by gameplay; level 1 = the base health and x1).
    max_health: u32,
    damage_scale: f32,
    // Crit chance of the active job (0 until gameplay sets the job) and a
    // small deterministic generator for the rolls.
    crit_percent: u32,
    crit_seed: u64,
    // Developer tools: the player takes no damage; the current fight's log.
    god: bool,
    fight_log: Vec<FightEvent>,
    ability_ready: BTreeMap<i32, u64>,
    last_weapon_refusal: Option<u64>,
    skills: SkillEffects,
    player_profile: i32,
    job_level: u32,
    // Where a dropped saved battle started; the gateway moves the character
    // there on login (its saved location is still inside the arena).
    dropped_return: Option<CharacterState>,
    /// One run's enemy clocks and walks; swapped as one value (`with_arena`).
    arena: ArenaSim,
    // Readiness is local to the connection. Rejoining never resumes attacks
    // while the client is still loading its scene and models.
    armed: bool,
    // Remove presents the original results screen. Keep the finished server
    // encounter until the player closes it; the client's 39/7 is only an ack.
    results_presented: bool,
    // (R) Metadata only after World takes ownership. No local HP copy remains.
    shared_fields: Option<BTreeSet<u64>>,
    shared_sender: Option<u64>,
    // Combat rules are on while a field enemy is engaged.
    field_combat: bool,
    // After an overworld knockout, field enemies ignore the player until then.
    field_grace_until: u64,
    /// When the current overworld knockout was first seen by `tick` (session-only).
    knocked_out_at: Option<u64>,
    /// Coins a paid revive costs (`field_enemies.json` `paid_revive`).
    paid_revive_cost: u32,
    battles: Vec<FieldBattle>,
    /// Start screen shown for (battle id, instance id); Go (41/108) starts it.
    /// Session-local: cleared by a successful `begin`, by a close/leave
    /// packet, and by Go once the player has left the marker; never saved.
    pending_battle: Option<(u32, i32)>,
    // Ground heights (osfr-terrain grids); without them enemies keep the
    // heights they interpolate.
    terrain: Option<Arc<Terrain>>,
    // Named behaviour presets and every enemy's resolved behaviour by GUID
    // (rebuilt whenever content or presets load).
    behaviour_presets: BTreeMap<String, Behaviour>,
    behaviour_of: BTreeMap<u64, Behaviour>,
    // Combat power-ups (session only, never saved: a pickup lasts seconds and
    // the held kind is not a bag item, so it cannot be traded or duplicated).
    power_ups: Option<Arc<PowerUps>>,
    power_up_rng: Option<Rng>,
    pickups: Vec<PowerUpDrop>,
    next_pickup: u64,
    /// The single stored power-up (kind id in `power_ups.json`). Slice 3 shows
    /// it in the key-3 slot and takes it on use; nothing else reads it.
    held_power_up: Option<String>,
    /// (R) Coins and active-job XP for a field kill (`field_kill_rewards.json`).
    field_rewards: Arc<field_rewards::FieldKillRewards>,
    /// Zone the pickups were dropped in; a different zone clears them.
    pickup_zone: Option<u32>,
    /// World instance this session's scene is bound to (`bind_run_instance`).
    /// A begin-zoning reload (opcode 31) drops the ground pickups even into the
    /// same instance; the held power-up stays (player state).
    pickup_instance: Option<i32>,
}

/// A pickup actor on the ground, owned by the player whose kill dropped it.
#[derive(Clone)]
struct PowerUpDrop {
    guid: u64,
    kind: String,
    kill: (u64, u64),
    position: [f32; 3],
    expires_ms: u64,
    /// The one-at-a-time refusal was already shown for this pickup.
    told: bool,
}

#[derive(Clone, Debug)]
pub struct FieldParticipant {
    pub guid: u64,
    pub position: [f32; 3],
    pub health: u32,
    pub max_health: u32,
    pub mitigation: f32,
    pub grace_until: u64,
    pub combat: bool,
    /// (R) Live shared aggro requires an observed NPC. Solo/unit callers use
    /// None; the gateway supplies its final combined NPC set after reconciliation.
    pub known_npcs: Option<BTreeSet<u64>>,
}

/// Terrain heights for the zone the encounter runs in.
#[derive(Clone)]
struct Ground(Option<(Arc<Terrain>, u32)>);

/// (R) Terrain more than this far from the height an enemy would otherwise
/// have is a bridge, building or ledge above or below it (the grid holds only
/// terrain), so the enemy keeps its own height there.
const MAX_SNAP: f32 = 6.0;

impl Ground {
    /// Snap a position onto the ground; off the grid, or where the ground is
    /// far away (a bridge), it keeps its height.
    fn snap(&self, p: &mut [f32; 3]) {
        if let Some(h) = self
            .0
            .as_ref()
            .and_then(|(t, zone)| t.height_at(*zone, p[0], p[2]))
        {
            if (h - p[1]).abs() <= MAX_SNAP {
                p[1] = h;
            }
        }
    }
    fn snapped(&self, mut p: [f32; 3]) -> [f32; 3] {
        self.snap(&mut p);
        p
    }
    /// Field motion must not cross missing terrain or snap over a ledge (R).
    /// Without a terrain grid, retain the authored/interpolated height.
    fn field_position(&self, mut p: [f32; 3]) -> Option<[f32; 3]> {
        if let Some((terrain, zone)) = &self.0 {
            let height = terrain.height_at(*zone, p[0], p[2])?;
            if (height - p[1]).abs() > MAX_SNAP {
                return None;
            }
            p[1] = height;
        }
        Some(p)
    }
}

impl EncounterEngine {
    /// Bind the session to its membership instance. `scene_reload` is the
    /// commit's begin-zoning fact: every opcode 31 begin-zoning (body reader
    /// 0x008e4ae0) makes the client reload its scene, including into the same
    /// world instance (a safe-location revive or a same-instance transfer), so
    /// its ground pickups no longer exist there. A changed instance id is a
    /// reload too. The held power-up is player state and stays.
    pub fn bind_run_instance(&mut self, instance: i32, scene_reload: bool) -> io::Result<bool> {
        if instance < 0 {
            return Err(invalid("negative encounter instance"));
        }
        if scene_reload
            || self
                .pickup_instance
                .is_some_and(|previous| previous != instance)
        {
            self.pickups.clear();
        }
        self.pickup_instance = Some(instance);
        if let Some(active) = &mut self.state.active {
            let changed = active.instance_id != instance;
            active.instance_id = instance;
            self.state.next_instance_id = self.state.next_instance_id.max(
                instance
                    .checked_add(1)
                    .ok_or_else(|| invalid("encounter instance exhausted"))?,
            );
            Ok(changed)
        } else {
            Ok(false)
        }
    }
    pub fn field_participant(
        &self,
        guid: u64,
        position: [f32; 3],
        combat: bool,
    ) -> FieldParticipant {
        FieldParticipant {
            guid,
            position,
            health: self.state.player_health,
            max_health: self.max_health,
            mitigation: if self.god {
                0.
            } else {
                self.skills.mitigation()
            },
            grace_until: self.field_grace_until,
            combat: combat && !self.active(),
            known_npcs: None,
        }
    }
    pub(crate) fn receive_field_damage(
        &mut self,
        damage: u32,
        _now_ms: u64,
        engaged: bool,
    ) -> EncounterOutcome {
        let mut out = EncounterOutcome::default();
        if damage > 0 && self.state.player_health > 0 {
            self.state.player_health = self.state.player_health.saturating_sub(damage);
            out.changed = true;
            out.replies
                .push(player_hp(self.state.player_health, self.max_health));
            if self.state.player_health == 0 {
                // (R) Hold the knockout; 41/122 chooses the way back up.
                out.replies.push(self.respawn_window());
            }
        }
        if self.field_combat != engaged && !self.world_stage_fighting() {
            self.field_combat = engaged;
            out.replies.extend([
                rules(engaged),
                combat_flag(132, 0, 0, engaged),
                combat_flag(133, 0, 0, engaged),
            ]);
        }
        out
    }
    pub fn set_terrain(&mut self, terrain: Option<Arc<Terrain>>) {
        self.terrain = terrain;
    }
    fn ground(&self) -> Ground {
        let zone = self
            .dungeon()
            .map_or(self.content.world.zone_id, |d| d.zone.zone_id);
        Ground(self.terrain.clone().map(|t| (t, zone)))
    }

    pub fn load(directory: impl AsRef<Path>, state: Option<EncounterState>) -> io::Result<Self> {
        let directory = directory.as_ref();
        let scale = CombatScale::load(directory)?;
        let bytes = fs::read(directory.join("encounters.json"))?;
        if bytes.len() > 128 * 1024 {
            return Err(invalid("encounter content too large"));
        }
        let mut content: Content =
            serde_json::from_slice(&bytes).map_err(|e| invalid(&e.to_string()))?;
        let path = Path::new(&content.template_file);
        if path.components().count() != 1
            || !matches!(
                path.components().next(),
                Some(std::path::Component::Normal(_))
            )
        {
            return Err(invalid("encounter template must be a local filename"));
        }
        content.template = fs::read(directory.join(path))?;
        let t = &content.template;
        if t.len() != 334
            || t[..4] != [35, 0, 2, 0]
            || hash(t) != content.template_sha256
            || [73, 81, 85, 106, 163]
                .iter()
                .any(|&o| t[o..o + 4] != [0; 4])
            || t[224] != 1
            || t[133..137] != 100_i32.to_le_bytes()
        {
            return Err(invalid("unsupported encounter NPC template"));
        }
        if content.schema_version != 1
            || content.dungeons.is_empty()
            || content.dungeons.len() > 4
            || content.world_enemies.len() > 32
        {
            return Err(invalid("invalid encounter content bounds"));
        }
        validate_zone(&content.world)?;
        let mut ids = BTreeSet::new();
        let mut guids = BTreeSet::new();
        for a in &content.world_enemies {
            validate_actor(a)?;
            if !guids.insert(a.guid) {
                return Err(invalid("duplicate actor GUID"));
            }
        }
        for d in &content.dungeons {
            if d.id == 0
                || !ids.insert(d.id)
                || d.title_id <= 0
                || d.description_id <= 0
                || d.objective_id <= 0
                || d.enemies.is_empty()
                || d.enemies.len() > 32
                || d.reward_coins > 1_000_000
                || d.reward_items.len() > 16
                || d.reward_items
                    .iter()
                    .any(|(id, n)| *id <= 0 || *n == 0 || *n > 10000)
            {
                return Err(invalid("invalid dungeon content"));
            }
            validate_zone(&d.zone)?;
            if let Some(guide) = &d.guide {
                validate_actor(guide)?;
                if !guids.insert(guide.guid) {
                    return Err(invalid("duplicate actor GUID"));
                }
            }
            for a in &d.enemies {
                validate_actor(a)?;
                if !guids.insert(a.guid) {
                    return Err(invalid("duplicate actor GUID"));
                }
            }
        }
        let content_hash = hash(&bytes);
        // Authored enemy rows are level-one baselines on the old scale.
        for a in (content.world_enemies.iter_mut())
            .chain(content.dungeons.iter_mut().flat_map(|d| &mut d.enemies))
        {
            scale.scale_enemy(a, 1.0, 1.0);
        }
        let mut state = state;
        if let Some(state) = state.as_mut() {
            // Saves from before battles were kept out of `rewarded` can hold
            // an overworld battle id (field_battles.json loads later). The
            // content hash below still pins the opening content, so an unknown
            // id here can only be such a battle: drop it rather than refuse
            // the login.
            state.rewarded.retain(|id| ids.contains(id));
        }
        let mut state = state.unwrap_or_else(|| EncounterState {
            version: 1,
            content_sha256: content_hash.clone(),
            active: None,
            world_stage: 0,
            world_enemies: content.world_enemies.iter().map(enemy_state).collect(),
            player_health: scale.player_base_health(),
            rewarded: BTreeSet::new(),
            next_instance_id: 1,
        });
        // (R) A world knockout is held in memory and saved as zero health; the
        // respawn window is not persisted, so a reload wakes the character.
        if state.active.is_none() && state.player_health == 0 {
            state.player_health = scale.player_base_health();
        }
        if state.version != 1
            || state.content_sha256 != content_hash
            || state.player_health > PLAYER_HEALTH_CAP
            || state.next_instance_id <= 0
            || state.rewarded.iter().any(|id| !ids.contains(id))
        {
            return Err(invalid("incompatible encounter state"));
        }
        restore_enemies(&mut state.world_enemies, &content.world_enemies)?;
        let mut state = state;
        let mut dropped_return = None;
        if state
            .active
            .as_ref()
            .is_some_and(|a| !content.dungeons.iter().any(|d| d.id == a.encounter_id))
        {
            // The character is saved in the battle arena: send it back to
            // where the battle started (see take_dropped_return).
            dropped_return = state.active.as_ref().map(|a| a.return_location.clone());
            // Overworld battles are not in encounters.json (they come from
            // field_battles.json after load). A save taken mid-battle returns
            // the player to the world at full health instead of failing.
            state.active = None;
            state.player_health = scale.player_base_health();
        }
        if let Some(a) = &mut state.active {
            let d = content
                .dungeons
                .iter()
                .find(|d| d.id == a.encounter_id)
                .ok_or_else(|| invalid("unknown saved encounter"))?;
            a.return_location.validate()?;
            if a.return_location.zone_id != content.world.zone_id
                || a.instance_id <= 0
                || a.instance_id >= state.next_instance_id
            {
                return Err(invalid("invalid encounter return/instance"));
            }
            restore_enemies(&mut a.enemies, &d.enemies)?;
            let cleared = a.enemies.iter().all(|e| e.health == 0);
            if (a.phase == EncounterPhase::Won
                && (!cleared || !state.rewarded.contains(&d.id) || state.player_health == 0))
                || (a.phase == EncounterPhase::Lost && state.player_health != 0)
                || (matches!(a.phase, EncounterPhase::Fighting | EncounterPhase::Loading)
                    && (cleared || state.player_health == 0))
            {
                return Err(invalid("inconsistent encounter phase"));
            }
        }
        let mut engine = Self {
            content: Arc::new(content),
            state,
            last_attack: None,
            last_tick: None,
            field_sim: FieldSim::default(),
            arena: ArenaSim::default(),
            dropped_return,
            energy: ENERGY_MAX,
            energy_regen_at: 0,
            max_health: scale.player_base_health(),
            scale,
            damage_scale: 1.0,
            crit_percent: 0,
            crit_seed: 0x5eed_c417_0000_0001,
            god: false,
            fight_log: Vec::new(),
            ability_ready: BTreeMap::new(),
            last_weapon_refusal: None,
            skills: SkillEffects::default(),
            player_profile: 0,
            job_level: 1,
            armed: false,
            results_presented: false,
            shared_fields: None,
            shared_sender: None,
            field_combat: false,
            field_grace_until: 0,
            knocked_out_at: None,
            paid_revive_cost: revive::RETAIL_REVIVE_COST,
            battles: Vec::new(),
            pending_battle: None,
            terrain: None,
            behaviour_presets: BTreeMap::new(),
            behaviour_of: BTreeMap::new(),
            power_ups: None,
            power_up_rng: None,
            pickups: Vec::new(),
            next_pickup: 0,
            held_power_up: None,
            field_rewards: Arc::new(field_rewards::FieldKillRewards::load(directory)?),
            pickup_zone: None,
            pickup_instance: None,
        };
        engine.rebuild_behaviours()?;
        Ok(engine)
    }

    /// Load `power_ups.json` (also a hot reload). A bad file is an `Err` and
    /// the previous tables stay in force.
    pub fn load_power_ups(&mut self, path: &Path) -> io::Result<()> {
        let power_ups = PowerUps::load(path)?;
        if self.power_up_rng.is_none() {
            self.power_up_rng = Some(Rng::from_entropy()?);
        }
        self.power_ups = Some(Arc::new(power_ups));
        Ok(())
    }

    /// Drop nothing from now on. The balance simulator fights on a clone of
    /// this engine and must not be helped by hearts it never earned.
    pub fn unload_power_ups(&mut self) {
        self.power_ups = None;
        self.pickups.clear();
    }

    /// The stored power-up, if any (slice 3 reads it for the key-3 slot).
    pub fn held_power_up(&self) -> Option<&str> {
        self.held_power_up.as_deref()
    }

    /// Load `behaviours.json` presets (also a hot reload).
    pub fn load_behaviours(&mut self, path: &Path) -> io::Result<()> {
        let bytes = fs::read(path)?;
        if bytes.len() > 256 * 1024 {
            return Err(invalid("behaviour presets too large"));
        }
        let file: BehaviourFile = serde_json::from_slice(&bytes)
            .map_err(|e| invalid(&format!("behaviours.json: {e}")))?;
        let previous = std::mem::replace(&mut self.behaviour_presets, file.presets);
        if let Err(error) = self.rebuild_behaviours() {
            self.behaviour_presets = previous;
            self.rebuild_behaviours()?;
            return Err(error);
        }
        Ok(())
    }

    fn rebuild_behaviours(&mut self) -> io::Result<()> {
        let actors = self
            .content
            .dungeons
            .iter()
            .flat_map(|d| d.enemies.iter())
            .chain(self.content.world_enemies.iter())
            .chain(self.field_sim.enemies.iter().map(|f| &f.actor));
        let mut resolved = BTreeMap::new();
        for actor in actors {
            let behaviour =
                resolve_behaviour(actor, &self.behaviour_presets).map_err(|e| invalid(&e))?;
            resolved.insert(actor.guid, behaviour);
        }
        self.behaviour_of = resolved;
        Ok(())
    }

    /// An enemy's resolved behaviour (for Studio and tests).
    pub fn behaviour(&self, guid: u64) -> Option<&Behaviour> {
        self.behaviour_of.get(&guid)
    }

    /// No weapon-specific refusal message is recovered. (R) The gameplay cast
    /// gate refuses silently and logs at most once per five seconds per session.
    pub(crate) fn refuse_weapon(&mut self, profile: i32, slot: i32, now_ms: u64) {
        if self
            .last_weapon_refusal
            .is_none_or(|last| now_ms.saturating_sub(last) >= 5_000)
        {
            eprintln!("combat: weapon requirement refused profile={profile} slot={slot}");
            self.last_weapon_refusal = Some(now_ms);
        }
    }

    /// The active job's level: max health and weapon damage follow it. A
    /// player at full health stays full; health never exceeds the new max.
    pub fn set_player_job(&mut self, profile: i32, level: u32) -> Vec<Vec<u8>> {
        if profile != self.player_profile {
            self.skills.clear();
        }
        self.player_profile = profile;
        self.job_level = level.clamp(1, 20);
        let (health, crit) = job_identity(profile);
        self.crit_percent = crit;
        let max = (self.scale.level_health(level) as f32 * health).round() as u32;
        self.damage_scale = self.scale.tier_factor(level);
        if max == self.max_health && self.state.player_health <= max {
            return Vec::new();
        }
        let was_full = self.state.player_health >= self.max_health;
        self.max_health = max;
        if was_full || self.state.player_health > max {
            self.state.player_health = max;
        }
        vec![player_hp(self.state.player_health, self.max_health)]
    }

    fn log_fight(
        &mut self,
        time_ms: u64,
        kind: &'static str,
        source: &str,
        target: &str,
        amount: i64,
    ) {
        if self.fight_log.len() >= FIGHT_LOG_CAP {
            self.fight_log.remove(0);
        }
        self.fight_log.push(FightEvent {
            time_ms,
            kind,
            source: source.to_owned(),
            target: target.to_owned(),
            amount,
        });
    }

    /// The developer fight log of the current (or last) fight.
    pub fn fight_log(&self) -> &[FightEvent] {
        &self.fight_log
    }

    pub fn player_health(&self) -> (u32, u32) {
        (self.state.player_health, self.max_health)
    }

    /// Developer: toggle invulnerability; returns the new state.
    pub fn toggle_god(&mut self) -> bool {
        self.god = !self.god;
        self.god
    }

    /// W146: heal the player by `percent` of maximum health (R), clamped to the maximum.
    /// A knocked-out player is not healed. Returns the 38/1 health update.
    pub fn heal_player_percent(&mut self, percent: u32) -> Vec<Vec<u8>> {
        if self.state.player_health == 0 {
            return Vec::new();
        }
        let gain = u64::from(self.max_health) * u64::from(percent.min(100)) / 100;
        let health = u64::from(self.state.player_health) + gain;
        self.state.player_health = health.min(u64::from(self.max_health)) as u32;
        vec![player_hp(self.state.player_health, self.max_health)]
    }

    /// Developer: full health and energy.
    pub fn heal_full(&mut self) -> Vec<Vec<u8>> {
        let was_down = self.state.player_health == 0;
        self.state.player_health = self.max_health;
        self.energy = ENERGY_MAX;
        let mut out = vec![
            player_hp(self.max_health, self.max_health),
            crate::abilities::build_energy(ENERGY_MAX, ENERGY_MAX),
        ];
        if was_down {
            out.push(respawn_hide());
        }
        out
    }

    /// Standing enemies of the active encounter: (guid, position, shielded).
    pub fn standing_enemies(&self) -> Vec<(u64, [f32; 3], bool)> {
        let (Some(d), Some(active)) = (self.dungeon(), self.state.active.as_ref()) else {
            return Vec::new();
        };
        d.enemies
            .iter()
            .zip(&active.enemies)
            .filter(|(_, s)| s.health > 0 && !s.dormant)
            .map(|(a, s)| (a.guid, s.position, self.shielded(a.guid)))
            .collect()
    }

    /// Developer: defeat every standing enemy of the active encounter through
    /// the normal hit path (minions first, so shields drop), finishing it.
    pub fn debug_win(&mut self, now_ms: u64) -> io::Result<EncounterOutcome> {
        let mut total = EncounterOutcome::default();
        if !self.active() {
            total.feedback = Some("No encounter is active.".into());
            return Ok(total);
        }
        for _ in 0..64 {
            let mut standing = self.standing_enemies();
            if standing.is_empty() {
                break;
            }
            standing.sort_by_key(|(_, _, shielded)| *shielded);
            let (guid, position, _) = standing[0];
            self.last_attack = None;
            let out = self.strike(guid, position, f32::MAX, u32::MAX / 4, now_ms)?;
            total.changed |= out.changed;
            total.replies.extend(out.replies);
            total.rewards.extend(out.rewards);
            total.defeated.extend(out.defeated);
            total.win_completed = total.win_completed.or(out.win_completed);
            if out.feedback.is_some() {
                total.feedback = out.feedback;
            }
        }
        Ok(total)
    }

    fn world_stage_fighting(&self) -> bool {
        self.content
            .world_enemies
            .iter()
            .zip(&self.state.world_enemies)
            .any(|(a, s)| a.stages.contains(&self.state.world_stage) && s.health > 0)
    }

    pub fn state(&self) -> &EncounterState {
        &self.state
    }
    pub fn world(&self) -> &ZoneTransition {
        &self.content.world
    }
    pub fn active(&self) -> bool {
        self.state.active.is_some()
    }
    pub fn zone(&self) -> Option<ZoneTransition> {
        self.dungeon().map(|d| d.zone.clone())
    }
    pub fn supports_location(&self, location: &CharacterState) -> bool {
        location.validate().is_ok()
            && location.zone_id
                == self
                    .dungeon()
                    .map_or(self.content.world.zone_id, |d| d.zone.zone_id)
    }
    pub fn reset_session(&mut self) {
        self.skills.clear();
        self.armed = false;
        self.results_presented = false;
        self.last_attack = None;
        self.last_tick = None;
        self.field_sim.attacks.clear();
        self.field_sim.moving.clear();
        self.arena = ArenaSim::default();
        // Zoning wipes the client's actors, so the pickups go with them. The
        // held power-up stays: it belongs to the player, not the zone.
        self.pickups.clear();
    }
    fn dungeon(&self) -> Option<&Dungeon> {
        self.state.active.as_ref().and_then(|a| {
            self.content
                .dungeons
                .iter()
                .find(|d| d.id == a.encounter_id)
        })
    }

    pub fn begin(
        &mut self,
        id: u32,
        return_location: CharacterState,
        _profile_id: i32,
    ) -> io::Result<EncounterOutcome> {
        return_location.validate()?;
        if self.active() {
            return Err(invalid("encounter already active"));
        }
        if self.state.player_health == 0 {
            return Err(invalid("cannot begin an encounter while knocked out"));
        }
        if return_location.zone_id != self.content.world.zone_id {
            return Err(invalid("encounter return must be the local world"));
        }
        let d = self
            .content
            .dungeons
            .iter()
            .find(|d| d.id == id)
            .ok_or_else(|| invalid("unknown encounter"))?;
        self.admit_battle(id)?;
        let next = self
            .state
            .next_instance_id
            .checked_add(1)
            .ok_or_else(|| invalid("encounter instance exhausted"))?;
        // Every start (portal, quest, Go or simulator) consumes the open
        // start screen; a later Go must not replay it.
        self.pending_battle = None;
        self.state.active = Some(ActiveEncounter {
            encounter_id: id,
            instance_id: self.state.next_instance_id,
            phase: EncounterPhase::Loading,
            return_location,
            enemies: d.enemies.iter().map(enemy_state).collect(),
        });
        self.state.next_instance_id = next;
        self.state.player_health = self.max_health;
        self.reset_session();
        self.fight_log.clear();
        let d = self.dungeon().unwrap();
        let feedback = if d.guide.is_some() {
            "Entering the opening cave. Combat waves, balance and rewards are locally reconstructed."
        } else {
            "Entering the battle. Enemy behaviour, balance and rewards are locally reconstructed."
        };
        Ok(EncounterOutcome {
            changed: true,
            zoning: Some(d.zone.clone()),
            feedback: Some(feedback.into()),
            ..Default::default()
        })
    }

    pub fn world_initialization(&self) -> Vec<Vec<u8>> {
        if self.active() {
            return Vec::new();
        }
        let fighting = self
            .content
            .world_enemies
            .iter()
            .zip(&self.state.world_enemies)
            .any(|(a, s)| a.stages.contains(&self.state.world_stage) && s.health > 0);
        let mut packets = vec![
            rules(fighting),
            combat_flag(132, 0, 0, fighting),
            combat_flag(133, 0, 0, fighting),
            player_hp(self.state.player_health, self.max_health),
        ];
        let mut active = Vec::new();
        for (a, s) in self
            .content
            .world_enemies
            .iter()
            .zip(&self.state.world_enemies)
        {
            if a.stages.contains(&self.state.world_stage) && s.health > 0 {
                packets.push(self.spawn(a, s.position, 0));
                packets.push(enemy_hp(a.guid, s.health, a.max_health));
                active.push(a.guid);
            }
        }
        if !active.is_empty() {
            packets.push(relevance(&active, false));
        }
        packets
    }

    pub fn set_world_stage(&mut self, stage: u32) -> EncounterOutcome {
        if self.state.world_stage == stage {
            return EncounterOutcome::default();
        }
        let mut outcome = EncounterOutcome {
            changed: true,
            ..Default::default()
        };
        if !self.active() {
            for a in &self.content.world_enemies {
                if a.stages.contains(&self.state.world_stage) && !a.stages.contains(&stage) {
                    outcome.replies.push(remove(a.guid));
                }
            }
        }
        self.state.world_stage = stage;
        outcome.replies.extend(self.world_initialization());
        outcome
    }

    pub fn on_zone_ready(&mut self, profile_id: i32, now_ms: u64) -> io::Result<EncounterOutcome> {
        if !self.active() {
            let mut replies = self.world_initialization();
            replies.extend(self.field_initialization());
            if self.state.player_health == 0 {
                // (R) A knocked-out player whose scene reloads gets the window back.
                replies.push(self.respawn_window());
            }
            return Ok(EncounterOutcome {
                replies,
                ..Default::default()
            });
        }
        let loading = self.state.active.as_ref().unwrap().phase == EncounterPhase::Loading;
        if loading {
            self.state.active.as_mut().unwrap().phase = EncounterPhase::Fighting;
        }
        self.armed = false;
        self.results_presented = false;
        self.last_tick = Some(now_ms);
        let active = self.state.active.as_ref().unwrap();
        let d = self.dungeon().unwrap();
        let mut out = EncounterOutcome {
            changed: loading,
            ..Default::default()
        };
        out.replies.push(self.details(true, profile_id));
        out.replies
            .push(encounter_state(d.id, active.instance_id, 4));
        out.replies
            .push(encounter_header(107, d.id, active.instance_id));
        out.replies
            .push(encounter_state(d.id, active.instance_id, 5));
        let mut enter = encounter_header(2, d.id, active.instance_id);
        enter.extend(1_u64.to_le_bytes());
        enter.push(0);
        out.replies.push(enter);
        out.replies.push(minigame(17, active.instance_id, d.id));
        out.replies
            .push(rules(active.phase == EncounterPhase::Fighting));
        out.replies.push(combat_flag(
            133,
            d.id,
            active.instance_id,
            active.phase == EncounterPhase::Fighting,
        ));
        out.replies
            .push(player_hp(self.state.player_health, self.max_health));
        let mut guids = Vec::new();
        if let Some(guide) = &d.guide {
            out.replies.push(self.spawn(guide, guide.position, 2));
            guids.push(guide.guid);
        }
        for (a, s) in d.enemies.iter().zip(&active.enemies) {
            if s.health > 0 && !s.dormant {
                out.replies.push(self.spawn(a, s.position, 0));
                out.replies.push(enemy_hp(a.guid, s.health, a.max_health));
                guids.push(a.guid);
                if a.boss {
                    out.replies.push(boss_display(a.guid, true));
                }
            }
        }
        out.replies.push(relevance(&guids, true));
        let (count, goal) = goal_progress(d, active);
        out.replies.push(objective(1, d.objective_id, goal));
        out.replies.push(objective(2, d.objective_id, count));
        if active.phase == EncounterPhase::Won {
            out.replies.push(objective_complete(d.objective_id));
            out.replies.push(game_over(true, active.instance_id, d.id));
        } else if active.phase == EncounterPhase::Lost {
            out.replies.push(game_over(false, active.instance_id, d.id));
        } else {
            out.replies
                .push(encounter_state(d.id, active.instance_id, 6));
            if d.guide.is_some() {
                out.feedback =
                    Some("Speak to Darkthorne and choose Start, or attack when ready.".into());
            }
        }
        // A battle has no guide to say Start: it begins once the arena loads.
        if d.guide.is_none() && active.phase == EncounterPhase::Fighting {
            self.armed = true;
        }
        self.present_results(&mut out);
        Ok(out)
    }

    pub fn handle(
        &mut self,
        inner: &[u8],
        player_guid: u64,
        position: [f32; 3],
        profile_id: i32,
        now_ms: u64,
    ) -> io::Result<EncounterOutcome> {
        if inner.len() < 2 {
            return Ok(EncounterOutcome::default());
        }
        if !valid_position(position) {
            return Err(invalid("invalid encounter player position"));
        }
        let op = u16::from_le_bytes(inner[..2].try_into().unwrap());
        if op == 39 {
            if inner.len() < 3 {
                return Err(invalid("truncated minigame request"));
            }
            // Game-start ack, end-screen/exit: only our exact active state is accepted.
            if inner[2] == 5 {
                if inner.len() != 15 {
                    return Err(invalid("malformed minigame ready ack"));
                }
                return Ok(EncounterOutcome::default());
            }
            return Ok(EncounterOutcome::default());
        }
        if inner.len() < 4 {
            return if matches!(op, 26 | 36 | 41) {
                Err(invalid("truncated encounter family header"))
            } else {
                Ok(EncounterOutcome::default())
            };
        }
        let sub = u16::from_le_bytes(inner[2..4].try_into().unwrap());
        match (op, sub) {
            (26, 42) => {
                if inner.len() != 4 {
                    return Err(invalid("malformed minigame end-screen close"));
                }
                if self
                    .state
                    .active
                    .as_ref()
                    .is_some_and(|a| matches!(a.phase, EncounterPhase::Won | EncounterPhase::Lost))
                {
                    self.exit()
                } else {
                    // Closing the start screen releases its pending Go.
                    self.pending_battle = None;
                    Ok(EncounterOutcome::default())
                }
            }

            (41, 122) => self.resume(inner, now_ms),
            (41, 109) => {
                if inner.len() != 12 {
                    return Err(invalid("malformed encounter exit"));
                }

                if let Some(a) = &self.state.active {
                    // Ui.LeaveMiniGameOrEncounter and Ui.QuitMiniGame leave
                    // the first header int zero and send the launch instance
                    // in the second (native 008A31A0 / 009BD240).
                    let encounter = i32::from_le_bytes(inner[4..8].try_into().unwrap());
                    if (encounter == 0 || encounter == a.encounter_id as i32)
                        && i32::from_le_bytes(inner[8..12].try_into().unwrap()) == a.instance_id
                    {
                        return self.exit();
                    }
                }
                if self.state.active.is_none() {
                    // Leaving the start screen releases its pending Go.
                    self.pending_battle = None;
                }
                Ok(EncounterOutcome::default())
            }
            (26, 8) => {
                if inner.len() != 13 || inner[12] > 1 {
                    return Err(invalid("malformed encounter interaction"));
                }
                let guid = u64::from_le_bytes(inner[4..12].try_into().unwrap());
                if self.is_battle_spawner(guid) {
                    return Ok(self.battle_screen(guid, position, profile_id));
                }
                Ok(self.interact(guid, position))
            }
            (26, 10) => {
                if inner.len() != 16 {
                    return Err(invalid("malformed encounter selection"));
                }
                let guid = u64::from_le_bytes(inner[4..12].try_into().unwrap());
                let choice = i32::from_le_bytes(inner[12..16].try_into().unwrap());
                if let Some(d) = self.dungeon() {
                    if d.guide
                        .as_ref()
                        .is_some_and(|g| guid == g.guid && distance(position, g.position) <= 12.0)
                    {
                        if choice == EXIT {
                            return self.exit();
                        }
                        if choice == RETRY {
                            return self.retry(profile_id, now_ms);
                        }
                        if choice == START
                            && !self.armed
                            && self.state.active.as_ref().unwrap().phase == EncounterPhase::Fighting
                        {
                            self.armed = true;
                            self.last_tick = Some(now_ms);
                            return Ok(EncounterOutcome {
                                feedback: Some(
                                    "Defend Darkthorne! Defeat the Gloamlings and Gloam King."
                                        .into(),
                                ),
                                ..Default::default()
                            });
                        }
                    }
                }
                if choice == ATTACK {
                    self.attack(guid, position, profile_id, 0, now_ms)
                } else {
                    Ok(EncounterOutcome::default())
                }
            }
            (36, 10) => {
                if inner.len() < 16 {
                    return Err(invalid("truncated encounter ability"));
                }
                let bar = i32::from_le_bytes(inner[4..8].try_into().unwrap());
                let slot = i32::from_le_bytes(inner[8..12].try_into().unwrap());
                let target_type = i32::from_le_bytes(inner[12..16].try_into().unwrap());
                if target_type == 1 {
                    if inner.len() != 32 {
                        return Err(invalid("malformed ground ability"));
                    }
                    return Ok(EncounterOutcome::default());
                }
                if inner.len() != 24 {
                    return Err(invalid("malformed targeted ability"));
                }
                if bar != 1 || slot != 0 || target_type != 0 {
                    return Ok(EncounterOutcome::default());
                }
                let guid = u64::from_le_bytes(inner[16..24].try_into().unwrap());
                if target_type == 0 && guid == player_guid {
                    // The original basic toolbar requests a self-centered attack.
                    // This supported slice resolves one nearby enemy on the server;
                    // original multi-target sweep effects remain unrecovered.
                    let states: Vec<&EnemyState> = match &self.state.active {
                        Some(a) => a.enemies.iter().collect(),
                        None => self
                            .state
                            .world_enemies
                            .iter()
                            .chain(
                                self.field_sim
                                    .enemies
                                    .iter()
                                    .filter(|f| f.present)
                                    .map(|f| &f.state),
                            )
                            .collect(),
                    };
                    let range = self.scale.profile_stats(profile_id, slot).0;
                    let nearest = states
                        .into_iter()
                        .filter(|s| {
                            self.enemy(s.guid).is_some()
                                && s.health > 0
                                && distance(position, s.position) <= range
                        })
                        // An unshielded enemy in reach first, then the nearest.
                        .min_by(|a, b| {
                            (self.shielded(a.guid), distance(position, a.position))
                                .partial_cmp(&(
                                    self.shielded(b.guid),
                                    distance(position, b.position),
                                ))
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .map(|s| s.guid);
                    return match nearest {
                        Some(target) => self.attack(target, position, profile_id, slot, now_ms),
                        None => Ok(EncounterOutcome::default()),
                    };
                }
                self.attack(guid, position, profile_id, slot, now_ms)
            }
            _ => Ok(EncounterOutcome::default()),
        }
    }

    fn interact(&self, guid: u64, position: [f32; 3]) -> EncounterOutcome {
        if let Some(guide) = self.dungeon().and_then(|d| d.guide.as_ref())
            && guid == guide.guid
            && distance(position, guide.position) <= 12.0
        {
            let phase = &self.state.active.as_ref().unwrap().phase;
            let entries = if *phase == EncounterPhase::Lost {
                vec![(RETRY, 432500), (EXIT, 4826)]
            } else if *phase == EncounterPhase::Fighting && !self.armed {
                vec![(START, 386175), (EXIT, 4826)]
            } else {
                vec![(EXIT, 4826)]
            };
            return EncounterOutcome {
                replies: vec![menu(guid, &guide.name, &entries, false)],
                ..Default::default()
            };
        }
        if let Some((a, s)) = self.enemy(guid) {
            if s.health > 0 && distance(position, s.position) <= 3.0 {
                return EncounterOutcome {
                    replies: vec![menu(guid, &a.name, &[(ATTACK, 22498)], true)],
                    ..Default::default()
                };
            }
        }
        EncounterOutcome::default()
    }

    fn enemy(&self, guid: u64) -> Option<(&Actor, &EnemyState)> {
        if self.shared_sender.is_some() {
            return self
                .field_sim
                .enemies
                .iter()
                .find(|f| f.present && f.actor.guid == guid && f.motion != FieldMotion::Returning)
                .map(|f| (&f.actor, &f.state));
        }
        if let Some(active) = &self.state.active {
            if active.phase != EncounterPhase::Fighting {
                return None;
            }
            self.dungeon()
                .unwrap()
                .enemies
                .iter()
                .zip(&active.enemies)
                .find(|(a, s)| a.guid == guid && !s.dormant)
        } else {
            self.content
                .world_enemies
                .iter()
                .zip(&self.state.world_enemies)
                .find(|(a, _)| a.guid == guid && a.stages.contains(&self.state.world_stage))
                .or_else(|| {
                    self.field_sim
                        .enemies
                        .iter()
                        .find(|f| f.present && f.actor.guid == guid)
                        .map(|f| (&f.actor, &f.state))
                })
        }
    }

    pub fn attack(
        &mut self,
        guid: u64,
        position: [f32; 3],
        profile_id: i32,
        slot: i32,
        now_ms: u64,
    ) -> io::Result<EncounterOutcome> {
        if !valid_position(position) || !(0..=4).contains(&slot) {
            return Err(invalid("invalid encounter attack"));
        }
        if self.state.player_health == 0 {
            return Ok(EncounterOutcome::default());
        }
        let (range, damage) = self.scale.profile_stats(profile_id, slot);
        let damage = (damage as f32 * self.damage_scale).round() as u32;
        self.strike(guid, position, range, damage, now_ms)
    }

    /// One hit on `guid` within `range` for `damage`: the basic attack and the
    /// job abilities share it (summon, shield, defeat and win handling).
    fn strike(
        &mut self,
        guid: u64,
        position: [f32; 3],
        range: f32,
        damage: u32,
        now_ms: u64,
    ) -> io::Result<EncounterOutcome> {
        self.strike_damage(guid, position, range, damage, now_ms, true)
    }

    fn strike_damage(
        &mut self,
        guid: u64,
        position: [f32; 3],
        range: f32,
        damage: u32,
        now_ms: u64,
        swing: bool,
    ) -> io::Result<EncounterOutcome> {
        self.skills.expire(now_ms);
        if self.state.player_health == 0 {
            return Ok(EncounterOutcome::default());
        }
        // (R) Evading field actors cannot be damaged or re-aggroed by attacks
        // (including an old DoT); return-home healing must finish first.
        if self
            .field_sim
            .enemies
            .iter()
            .any(|f| f.actor.guid == guid && f.motion == FieldMotion::Returning)
        {
            return Ok(EncounterOutcome::default());
        }
        let Some((a, s)) = self.enemy(guid) else {
            return Ok(EncounterOutcome::default());
        };
        if s.health == 0
            || distance(position, s.position) > range
            || swing
                && self
                    .last_attack
                    .is_some_and(|last| now_ms.saturating_sub(last) < 600)
        {
            return Ok(EncounterOutcome::default());
        }
        let (definition, max_health, boss) = (a.definition_id, a.max_health, a.boss);
        let level = a.level;
        let target_name = a.name.clone();
        let drop_table = a.power_up_table.clone();
        let health = s.health;
        let killed_at = s.position;
        // A summoner is shielded while its minions stand: clear them first.
        let damage = if self.shielded(guid) {
            0
        } else {
            self.skills.damage(guid, damage)
        };
        let critical = damage > 0 && self.roll_crit();
        let damage = if critical {
            (f64::from(damage)
                * (f64::from(CRIT_NUMERATOR) / f64::from(CRIT_DENOMINATOR)
                    + f64::from(self.skills.crit_damage_bonus()))) as u32
        } else {
            damage
        };
        let dealt = damage.min(health);
        let hp = health - dealt;
        if let (Some(sender), true) = (self.shared_sender, dealt > 0) {
            self.field_sim
                .kills
                .hits
                .entry(guid)
                .or_default()
                .insert(sender, now_ms);
        }
        if self.active() {
            if !self.armed {
                self.last_tick = Some(now_ms);
            }
            self.armed = true;
        }
        if swing {
            self.last_attack = Some(now_ms);
        }
        let field_index = if self.state.active.is_none() {
            self.field_sim
                .enemies
                .iter()
                .position(|f| f.present && f.actor.guid == guid)
        } else {
            None
        };
        if let Some(i) = field_index {
            self.field_sim.enemies[i].state.health = hp;
            self.field_sim.enemies[i].motion = FieldMotion::Chasing;
        } else {
            let states = if let Some(active) = &mut self.state.active {
                &mut active.enemies
            } else {
                &mut self.state.world_enemies
            };
            states.iter_mut().find(|s| s.guid == guid).unwrap().health = hp;
        }
        self.log_fight(
            now_ms,
            if critical { "crit" } else { "hit" },
            "you",
            &target_name,
            i64::from(dealt),
        );
        if hp == 0 {
            self.log_fight(now_ms, "defeat", "you", &target_name, 0);
        }
        let mut out = EncounterOutcome {
            changed: true,
            replies: vec![
                attack_processed(
                    self.shared_sender.unwrap_or(1),
                    guid,
                    dealt,
                    hp,
                    max_health,
                    critical,
                ),
                enemy_hp(guid, hp, max_health),
            ],
            ..Default::default()
        };
        if swing {
            out.replies
                .insert(0, animation(self.shared_sender.unwrap_or(1), 1099, false));
        }
        if hp > 0 && self.active() {
            self.summon(guid, hp, max_health, &mut out);
        }
        if hp == 0 {
            self.skills.forget_target(guid);
            if self.shared_sender.is_none() {
                out.replies.push(remove(guid));
            }
            if boss {
                out.replies.push(boss_display(guid, false));
            }
            // T256: the shared field also credits the killer's own quests.
            if definition > 0 {
                out.defeated.push(definition);
            }
            self.arena.kiting.remove(&guid);
            if let Some(i) = field_index {
                self.field_sim.moving.remove(&guid);
                self.field_sim.attacks.remove(&guid);
                let f = &mut self.field_sim.enemies[i];
                f.respawn_at = Some(now_ms.saturating_add(f.respawn_ms));
            } else if let Some(active) = &self.state.active {
                let d = self.dungeon().unwrap();
                let (count, goal) = goal_progress(d, active);
                out.replies.push(objective(2, d.objective_id, count));
                if count >= goal {
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
            } else if !self.world_stage_fighting() && !self.field_combat {
                out.replies.extend([
                    rules(false),
                    combat_flag(132, 0, 0, false),
                    combat_flag(133, 0, 0, false),
                ]);
            }
        }
        if hp == 0 {
            let hitters = self.field_sim.kills.hits.remove(&guid).unwrap_or_default();
            // A won encounter is over: its zone is about to close, nothing to pick up.
            if self
                .state
                .active
                .as_ref()
                .is_none_or(|a| a.phase == EncounterPhase::Fighting)
            {
                let reward =
                    field_index.and_then(|_| self.field_rewards.reward_for(definition, level));
                let engaged = field_index.and_then(|i| self.field_sim.enemies[i].target);
                self.credit_field_kill(
                    drop_table.as_deref(),
                    reward,
                    definition,
                    guid,
                    killed_at,
                    now_ms,
                    hitters,
                    engaged,
                    &mut out,
                );
            }
        }
        self.present_results(&mut out);
        Ok(out)
    }

    /// Heal `percent` of max health and show the "+N" number. Says nothing when
    /// nothing was healed (full health, or knocked out: healing never revives).
    fn heal_with_number(&mut self, percent: u32, out: &mut EncounterOutcome) {
        let before = self.state.player_health;
        let replies = self.heal_player_percent(percent);
        let healed = (self.state.player_health - before) as i32;
        if healed > 0 {
            out.replies.extend(replies);
            out.replies
                .push(crate::abilities::build_hitpoint_delta(1, 1, healed));
            out.changed = true;
        }
    }

    /// Another player's big heart: heal this player by the same percent.
    pub fn receive_group_heal(&mut self, percent: u32) -> EncounterOutcome {
        let mut out = EncounterOutcome::default();
        self.heal_with_number(percent.min(100), &mut out);
        out
    }

    /// The player is in `zone` now. Pickups left in another zone are gone from
    /// the client already, so forget them (no removal packets).
    pub fn enter_zone(&mut self, zone: u32) {
        if self.pickup_zone.is_some_and(|previous| previous != zone) {
            self.pickups.clear();
        }
        self.pickup_zone = Some(zone);
    }

    /// How many ground pickups this session still holds (tests and diagnostics).
    pub fn pickup_count(&self) -> usize {
        self.pickups.len()
    }

    /// Roll `table` for one participant of the kill and spawn each drop as a
    /// pickup actor. An unknown table, no loaded file or a full ground drops
    /// nothing. With several participants every chance is scaled by the file's
    /// group multiplier.
    fn drop_power_ups(
        &mut self,
        table: &str,
        kill: (u64, u64),
        at: [f32; 3],
        now_ms: u64,
        participants: usize,
        out: &mut EncounterOutcome,
    ) {
        let ground = self.ground();
        let Some(power_ups) = self.power_ups.clone() else {
            return;
        };
        let (Some(table), Some(rng)) = (power_ups.table(table), self.power_up_rng.as_mut()) else {
            return;
        };
        let scale = if participants > 1 {
            power_ups.group_chance_multiplier()
        } else {
            1.0
        };
        let kinds = power_ups::roll_scaled(table, rng, scale);
        for (i, kind_id) in kinds.into_iter().enumerate() {
            let Some(kind) = power_ups.kind(kind_id) else {
                continue;
            };
            if self.pickups.len() >= power_ups.pickup.max_alive_per_instance as usize {
                break;
            }
            self.next_pickup += 1;
            let guid = PICKUP_GUID_BASE | self.next_pickup;
            // (R) Several drops from one kill sit side by side, not stacked.
            let position = ground.snapped([at[0] + i as f32 * 0.8, at[1], at[2]]);
            let actor = Actor {
                guid,
                definition_id: 0,
                model_id: kind.model_id as i32,
                name_id: 0,
                name: String::new(),
                texture_alias: String::new(),
                position,
                max_health: 1,
                damage: 0,
                level: 1,
                boss: false,
                stages: Vec::new(),
                summon_below_percent: None,
                skittish: false,
                scale: kind.scale.map(|s| s as f32),
                behaviour: None,
                power_up_table: None,
            };
            // Disposition 1 (neutral): 0 is hostile and would be Tab-targeted.
            out.replies.push(self.spawn(&actor, position, 1));
            self.pickups.push(PowerUpDrop {
                guid,
                kind: kind_id.to_owned(),
                kill,
                position,
                expires_ms: now_ms.saturating_add(u64::from(power_ups.pickup.ttl_ms)),
                told: false,
            });
        }
    }

    /// Despawn expired pickups and collect the ones the player stands on. The
    /// position is the server's own (never a client claim).
    fn tick_power_ups(&mut self, position: [f32; 3], now_ms: u64, out: &mut EncounterOutcome) {
        let Some(power_ups) = self.power_ups.clone() else {
            return;
        };
        let radius = power_ups.pickup.radius as f32;
        for mut drop in std::mem::take(&mut self.pickups) {
            let collected = now_ms < drop.expires_ms
                && self.state.player_health > 0
                && horizontal_distance(position, drop.position) <= radius
                && self.collect_power_up(&mut drop, &power_ups, out);
            if collected || now_ms >= drop.expires_ms {
                out.replies.push(remove(drop.guid));
            } else {
                self.pickups.push(drop);
            }
        }
    }

    /// Apply one pickup. `false` leaves it on the ground (the player already
    /// holds a power-up and this kind is stored, not automatic).
    fn collect_power_up(
        &mut self,
        drop: &mut PowerUpDrop,
        power_ups: &PowerUps,
        out: &mut EncounterOutcome,
    ) -> bool {
        // A reload that removed the kind just clears the stale pickup.
        let Some(kind) = power_ups.kind(&drop.kind) else {
            return true;
        };
        if !kind.auto {
            if self.held_power_up.is_some() {
                if !drop.told && out.feedback.is_none() {
                    out.feedback = Some(ONE_POWER_UP_TEXT.into());
                    drop.told = true;
                }
                return false;
            }
            self.held_power_up = Some(drop.kind.clone());
        }
        if let Some(percent) = kind.heal_pct_max {
            self.heal_with_number(percent, out);
            if kind.group {
                out.group_heal = Some(GroupHeal {
                    percent: percent.min(100),
                    kill: drop.kill,
                });
            }
        }
        if let Some(energy) = kind.energy {
            self.energy = self.energy.saturating_add(energy as i32).min(ENERGY_MAX);
            out.replies
                .push(crate::abilities::build_energy(self.energy, ENERGY_MAX));
        }
        out.replies.push(composite_effect(
            self.shared_sender.unwrap_or(1),
            POWER_UP_PICKUP_EFFECT,
            drop.position,
        ));
        true
    }

    /// The summoner was hit down to `hp`: every dormant minion whose
    /// threshold is now crossed appears around it in a puff, while the
    /// summoner plays its cast.
    fn summon(&mut self, summoner: u64, hp: u32, max_health: u32, out: &mut EncounterOutcome) {
        let Some(d) = self.dungeon() else {
            return;
        };
        let Some(home) = d
            .enemies
            .iter()
            .find(|a| a.guid == summoner && a.summon_below_percent.is_none())
            .map(|a| a.position)
        else {
            return;
        };
        if !d.enemies.iter().any(|a| a.summon_below_percent.is_some()) {
            return;
        }
        let percent = (u64::from(hp) * 100 / u64::from(max_health.max(1))) as u32;
        let content = self.content.clone();
        let d = content.dungeons.iter().find(|x| x.id == d.id).unwrap();
        let active = self.state.active.as_mut().unwrap();
        let at = active
            .enemies
            .iter()
            .find(|s| s.guid == summoner)
            .map_or(home, |s| s.position);
        let mut called = Vec::new();
        for (a, s) in d.enemies.iter().zip(active.enemies.iter_mut()) {
            if s.dormant && a.summon_below_percent.is_some_and(|below| percent < below) {
                s.dormant = false;
                // Keep the authored offset from the summoner, around where it
                // stands now.
                s.position = [
                    at[0] + (a.position[0] - home[0]),
                    at[1],
                    at[2] + (a.position[2] - home[2]),
                ];
                called.push((a.clone(), s.position));
            }
        }
        if called.is_empty() {
            return;
        }
        out.replies
            .push(animation(summoner, SUMMON_ANIMATION, false));
        let mut guids = Vec::new();
        for (a, position) in &called {
            out.replies
                .push(composite_effect(a.guid, SUMMON_EFFECT, *position));
            out.replies.push(self.spawn(a, *position, 0));
            out.replies
                .push(enemy_hp(a.guid, a.max_health, a.max_health));
            out.replies.push(animation(a.guid, SPAWN_ANIMATION, false));
            guids.push(a.guid);
        }
        out.replies.push(relevance(&guids, false));
    }

    pub fn tick(
        &mut self,
        position: [f32; 3],
        profile_id: i32,
        now_ms: u64,
    ) -> io::Result<EncounterOutcome> {
        let mut out = self.tick_encounter(position, profile_id, now_ms)?;
        self.tick_power_ups(position, now_ms, &mut out);
        Ok(out)
    }

    fn tick_encounter(
        &mut self,
        position: [f32; 3],
        _profile_id: i32,
        now_ms: u64,
    ) -> io::Result<EncounterOutcome> {
        if !valid_position(position) {
            return Err(invalid("invalid encounter player position"));
        }
        if self.state.player_health > 0 {
            self.knocked_out_at = None;
        } else if self.state.active.is_none() {
            let at = *self.knocked_out_at.get_or_insert(now_ms);
            if now_ms >= at.saturating_add(AUTO_REVIVE_MS) {
                return Ok(self.revive_at_safe_location(now_ms));
            }
        }
        if self
            .state
            .active
            .as_ref()
            .is_some_and(|a| a.phase != EncounterPhase::Fighting)
            || (self.active() && !self.armed)
            || self.state.player_health == 0
        {
            return Ok(EncounterOutcome::default());
        }
        let elapsed = self.last_tick.map_or(100, |t| now_ms.saturating_sub(t));
        if elapsed < 100 {
            return Ok(EncounterOutcome::default());
        }
        self.last_tick = Some(now_ms);
        let mut energy = EncounterOutcome::default();
        self.tick_job_effects(now_ms, &mut energy)?;
        if self
            .state
            .active
            .as_ref()
            .is_some_and(|a| a.phase != EncounterPhase::Fighting)
        {
            return Ok(energy);
        }
        self.regenerate_energy(now_ms, &mut energy);
        let mitigation = self.skills.mitigation();
        let ground = self.ground();
        let actors: Vec<Actor> = if let Some(d) = self.dungeon() {
            d.enemies.clone()
        } else {
            self.content
                .world_enemies
                .iter()
                .filter(|a| a.stages.contains(&self.state.world_stage))
                .cloned()
                .collect()
        };
        let states = if let Some(a) = &mut self.state.active {
            &mut a.enemies
        } else {
            &mut self.state.world_enemies
        };
        let mut out = EncounterOutcome::default();
        out.changed = energy.changed;
        out.rewards.append(&mut energy.rewards);
        out.defeated.append(&mut energy.defeated);
        out.replies.append(&mut energy.replies);
        let tick = ArenaTick {
            position,
            ground: &ground,
            behaviours: &self.behaviour_of,
            skills: &self.skills,
            mitigation,
            player_health: self.state.player_health,
            max_health: self.max_health,
            now_ms,
            elapsed,
        };
        let mut total_damage = self.arena.tick_enemies(actors, states, &tick, &mut out);
        if !self.active() && self.shared_fields.is_none() {
            self.tick_field(
                position,
                now_ms,
                elapsed,
                &mut out,
                &mut total_damage,
                None,
                &mut Vec::new(),
            );
            self.tick_wanderers_for(
                &[self.field_participant(1, position, true)],
                now_ms,
                elapsed,
                &mut out,
            );
        }
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
                self.skills.clear();
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
        self.present_results(&mut out);
        Ok(out)
    }

    fn present_results(&mut self, out: &mut EncounterOutcome) {
        if self.results_presented {
            return;
        }
        let Some(active) = &self.state.active else {
            return;
        };
        if matches!(active.phase, EncounterPhase::Won | EncounterPhase::Lost) {
            if active.phase == EncounterPhase::Lost {
                // The native target listens to the boss registry even over
                // results. Unregister living bosses without killing/removing
                // them; Retry's normal spawn registers their display again.
                for (actor, state) in self.dungeon().unwrap().enemies.iter().zip(&active.enemies) {
                    if actor.boss && state.health > 0 {
                        out.replies.push(boss_display(actor.guid, false));
                    }
                }
            }
            // The original Flash screen automatically closes if Lua has no
            // score/stat/reward pages. Populate its score data before Show.
            out.replies.push(end_scores(active));
            // Native OnGameOver only stores results. LeaveMiniGame (39/19)
            // calls the Lua score screen; zoning here would cover that screen.
            out.replies
                .push(minigame(19, active.instance_id, active.encounter_id));
            self.results_presented = true;
        }
    }

    pub fn exit(&mut self) -> io::Result<EncounterOutcome> {
        let Some(a) = self.state.active.take() else {
            return Ok(EncounterOutcome::default());
        };
        let mut zone = self.content.world.clone();
        zone.zone_id = a.return_location.zone_id;
        zone.position = a.return_location.position;
        zone.rotation = a.return_location.rotation;
        let remove_state = minigame(19, a.instance_id, a.encounter_id); // exact header only
        let results_presented = self.results_presented;
        let mut replies = vec![
            rules(false),
            combat_flag(132, 0, 0, false),
            combat_flag(133, a.encounter_id, a.instance_id, false),
        ];
        if !results_presented {
            replies.push(remove_state);
        }
        replies.push(player_hp(self.max_health, self.max_health));
        self.state.player_health = self.max_health;
        self.reset_session();
        Ok(EncounterOutcome {
            changed: true,
            zoning: Some(zone),
            completed: (a.phase == EncounterPhase::Won).then_some(a.encounter_id),
            replies,
            ..Default::default()
        })
    }

    pub fn retry(&mut self, _profile_id: i32, _now_ms: u64) -> io::Result<EncounterOutcome> {
        if self
            .state
            .active
            .as_ref()
            .is_none_or(|a| a.phase != EncounterPhase::Lost)
        {
            return Ok(EncounterOutcome::default());
        }
        let enemies = self
            .dungeon()
            .unwrap()
            .enemies
            .iter()
            .map(enemy_state)
            .collect();
        let a = self.state.active.as_mut().unwrap();
        a.enemies = enemies;
        a.phase = EncounterPhase::Loading;
        self.state.player_health = self.max_health;
        self.reset_session();
        Ok(EncounterOutcome {
            changed: true,
            zoning: self.zone(),
            ..Default::default()
        })
    }

    fn spawn(&self, a: &Actor, position: [f32; 3], disposition: i32) -> Vec<u8> {
        spawn_actor(&self.content.template, a, position, disposition)
    }

    fn details(&self, launch: bool, profile: i32) -> Vec<u8> {
        let a = self.state.active.as_ref().unwrap();
        let d = self.dungeon().unwrap();
        details_packet(
            d.id,
            a.instance_id,
            [d.title_id, d.icon_id, d.description_id, d.objective_id, 1],
            d.enemies.len(),
            profile,
            launch,
            &d.prizes,
        )
    }
}

/// 41/114 EncounterDetailsResponse. `launch = false` opens the minigame start
/// screen (HandleMiniGameStart); `true` only refreshes a running encounter.
/// `text` is [title, icon, description, objective, difficulty] (MiniGameInfo
/// NameId, IconId, DescriptionId, Difficulty per the community layout and
/// the native reader 009bdd70); the cave's difficulty was a constant 1.
fn details_packet(
    id: u32,
    instance: i32,
    text: [i32; 5],
    enemy_count: usize,
    _profile: i32,
    launch: bool,
    prizes: &[Prize],
) -> Vec<u8> {
    let [title_id, icon_id, description_id, objective_id, difficulty] = text;
    {
        let mut b = encounter_header(114, id, instance);
        for v in [0_i32, 0, 0, 0, 8, 0] {
            b.extend(v.to_le_bytes());
        }
        b.extend([1, 0, 1]);
        for v in [0_i32, 0] {
            b.extend(v.to_le_bytes());
        }
        // Original Lua MiniGameTypes.COMBAT_TYPE is 4. SAMPLE_TYPE (1)
        // disables native combat target selection despite combat rules being on.
        // MiniGameInfo.ProfileType is the job TYPE the game needs, not the
        // player's job id: the start screen's Switch Jobs picker lists owned
        // jobs of this type whose rank meets the difficulty (00c8ff30), and
        // only enables at all when it is non-zero. Combat jobs are type 2
        // (profile definitions: Brawler, Medic, Wizard, Ninja, ...).
        for v in [
            title_id,
            icon_id,
            description_id,
            difficulty,
            COMBAT_PROFILE_TYPE,
            4,
        ] {
            b.extend(v.to_le_bytes());
        }
        b.push(0);
        // Reward bundles: base and member empty; the third ("preview") lists
        // the prizes on the start screen and hides its placeholder art.
        b.extend([0; 138]);
        b.extend(prize_bundle(prizes));
        b.extend(1_i32.to_le_bytes());
        for v in [objective_id, 4419610, 4419610] {
            b.extend(v.to_le_bytes());
        }
        b.push(0);
        b.extend([0; 69]);
        for v in [0_i32, 0, enemy_count as i32, 0] {
            b.extend(v.to_le_bytes());
        }
        b.push(0);
        b.extend(0_i32.to_le_bytes());
        b.extend([1; 5]);
        b.extend(0_i32.to_le_bytes());
        b.extend(1_i32.to_le_bytes());
        b.push(1);
        b.extend(0_i32.to_le_bytes());
        b.extend([0; 4]);
        b.extend(0_i32.to_le_bytes());
        b.extend([0, 1, u8::from(launch)]);
        b.extend([0; 8]);
        b
    }
}

fn validate_zone(z: &ZoneTransition) -> io::Result<()> {
    if z.name.is_empty()
        || z.name.len() > 128
        || !z
            .name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        || z.sky.len() > 128
        || !z
            .sky
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'.')
        || !valid_position(z.position)
        || !valid_position(z.rotation)
    {
        return Err(invalid("invalid zone definition"));
    }
    Ok(())
}
fn validate_actor(a: &Actor) -> io::Result<()> {
    if a.guid <= 1
        || a.model_id <= 0
        || a.name_id < 0
        || a.definition_id < 0
        || a.name.is_empty()
        || a.name.len() > 120
        || a.name.chars().any(char::is_control)
        || a.texture_alias.len() > 120
        || !valid_position(a.position)
        || a.max_health == 0
        || a.max_health > 100000
        || a.damage > 1000
        || a.stages.len() > 32
        || a.power_up_table
            .as_ref()
            .is_some_and(|t| t.is_empty() || t.len() > 64)
    {
        return Err(invalid("invalid encounter actor"));
    }
    Ok(())
}
/// Saved enemies are absolute health but the scale file is tunable: clamp each to the loaded max
/// (a full-health enemy stays full when the scale drops), then check the rest.
fn restore_enemies(states: &mut [EnemyState], actors: &[Actor]) -> io::Result<()> {
    for (s, a) in states.iter_mut().zip(actors) {
        s.health = s.health.min(a.max_health);
    }
    if states.len() != actors.len()
        || states.iter().zip(actors).any(|(s, a)| {
            s.guid != a.guid || s.health > a.max_health || !valid_position(s.position)
        })
    {
        return Err(invalid("invalid saved enemies"));
    }
    Ok(())
}
fn enemy_state(a: &Actor) -> EnemyState {
    EnemyState {
        guid: a.guid,
        health: a.max_health,
        position: a.position,
        dormant: a.summon_below_percent.is_some(),
    }
}
/// (defeated goal enemies, goal enemies). Summoned minions are not goals.
fn goal_progress(d: &Dungeon, active: &ActiveEncounter) -> (i32, i32) {
    let goals = d
        .enemies
        .iter()
        .zip(&active.enemies)
        .filter(|(a, _)| a.summon_below_percent.is_none());
    let (mut down, mut total) = (0, 0);
    for (_, s) in goals {
        total += 1;
        if s.health == 0 {
            down += 1;
        }
    }
    (down, total)
}

/// One line of the developer fight log (Studio): who did what to whom.
#[derive(Clone, Debug, Serialize)]
pub struct FightEvent {
    pub time_ms: u64,
    pub kind: &'static str,
    pub source: String,
    pub target: String,
    pub amount: i64,
}
const FIGHT_LOG_CAP: usize = 600;

fn header(op: u16, sub: u16) -> Vec<u8> {
    [op.to_le_bytes(), sub.to_le_bytes()].concat()
}
fn encounter_header(sub: u16, id: u32, instance: i32) -> Vec<u8> {
    let mut b = header(41, sub);
    b.extend((id as i32).to_le_bytes());
    b.extend(instance.to_le_bytes());
    b
}
fn encounter_state(id: u32, instance: i32, state: i32) -> Vec<u8> {
    let mut b = encounter_header(106, id, instance);
    b.extend(state.to_le_bytes());
    b
}
fn combat_flag(sub: u16, id: u32, instance: i32, on: bool) -> Vec<u8> {
    let mut b = encounter_header(sub, id, instance);
    b.push(u8::from(on));
    b
}
fn minigame(sub: u8, _instance: i32, id: u32) -> Vec<u8> {
    let mut b = 39_u16.to_le_bytes().to_vec();
    b.push(sub);
    // Native 41/114 creates its MiniGameState using the FIRST encounter-header
    // integer (encounter id). All 39 lifecycle messages must use that same key.
    for v in [id as i32, -1, id as i32] {
        b.extend(v.to_le_bytes());
    }
    b
}
fn game_over(won: bool, instance: i32, id: u32) -> Vec<u8> {
    let mut b = minigame(18, instance, id);
    b.push(u8::from(won));
    for v in [0_i32, 0, 1] {
        b.extend(v.to_le_bytes());
    }
    b
}
fn end_scores(active: &ActiveEncounter) -> Vec<u8> {
    end_scores_packet(
        active.instance_id,
        active.encounter_id,
        active.enemies.iter().filter(|e| e.health == 0).count() as i32,
        active.phase == EncounterPhase::Won,
    )
}

fn end_scores_packet(instance: i32, id: u32, defeated: i32, won: bool) -> Vec<u8> {
    let mut b = minigame(47, instance, id);
    b.extend(2_i32.to_le_bytes());
    // These are original localization keys and score types. Only the Order-4
    // total row changes with the outcome: a won run reports the local
    // objective award, a loss reports zero. Defeat counts stay actual and no
    // per-kill points or historical score are invented.
    let total = if won { OBJECTIVE_AWARD_SCORE } else { 0 };
    for (name, order, value, points) in [
        ("scoreEnemiesDefeated", 0_i32, defeated, 0_i32),
        ("scoreTotalScore", 4_i32, -1, total),
    ] {
        b.extend((name.len() as i32).to_le_bytes());
        b.extend(name.as_bytes());
        for field in [-1, order, value, -1, points] {
            b.extend(field.to_le_bytes());
        }
    }
    b
}
fn objective(sub: u8, id: i32, value: i32) -> Vec<u8> {
    let mut b = 45_u16.to_le_bytes().to_vec();
    b.push(sub);
    b.extend(id.to_le_bytes());
    b.extend(value.to_le_bytes());
    b
}
fn objective_complete(id: i32) -> Vec<u8> {
    let mut b = objective(3, id, 0);
    b.extend(OBJECTIVE_AWARD_SCORE.to_le_bytes());
    b
}
fn rules(combat: bool) -> Vec<u8> {
    let mut b = 62_u16.to_le_bytes().to_vec();
    b.extend([u8::from(combat); 11]);
    b.extend((-1_i32).to_le_bytes());
    b.extend(1_f32.to_le_bytes());
    for v in [3600_i32, 0, 0, 0, 0, 0] {
        b.extend(v.to_le_bytes());
    }
    b
}
fn player_hp(hp: u32, max: u32) -> Vec<u8> {
    let mut b = header(38, 1);
    b.extend((hp as i32).to_le_bytes());
    b.extend((max as i32).to_le_bytes());
    b
}
fn enemy_hp(guid: u64, hp: u32, max: u32) -> Vec<u8> {
    let mut b = header(35, 5);
    b.extend(guid.to_le_bytes());
    for v in [hp as i32, max as i32, 0] {
        b.extend(v.to_le_bytes());
    }
    b
}
fn boss_display(guid: u64, on: bool) -> Vec<u8> {
    let mut b = header(32, 9);
    b.extend(guid.to_le_bytes());
    b.push(u8::from(on));
    b
}
fn attack_processed(
    attacker: u64,
    target: u64,
    damage: u32,
    hp: u32,
    max: u32,
    critical: bool,
) -> Vec<u8> {
    let mut b = header(32, 7);
    for v in [attacker, attacker, target] {
        b.extend(v.to_le_bytes());
    }
    for v in [damage as i32, max as i32, 0] {
        b.extend(v.to_le_bytes());
    }
    // +0x2c critical-style flag (the client shows "-N!!"), +0x2d suppress reaction.
    b.extend([u8::from(critical), 0]);
    for v in [0_i32, hp as i32] {
        b.extend(v.to_le_bytes());
    }
    b
}
/// 35/8 SetAnimation (client reader `0x008e5dd0`); also world events' writer.
pub(crate) fn animation(guid: u64, id: i32, base: bool) -> Vec<u8> {
    let mut b = header(35, 8);
    b.extend(guid.to_le_bytes());
    b.extend(id.to_le_bytes());
    b.extend(0_i32.to_le_bytes());
    b.push(u8::from(base));
    b
}
fn movement(guid: u64, p: [f32; 3], heading: [f32; 3]) -> Vec<u8> {
    // Movement is standalone opcode125, not a BasePlayerUpdate subpacket.
    let mut b = 125_u16.to_le_bytes().to_vec();
    b.extend(guid.to_le_bytes());
    for v in p.into_iter().chain(heading) {
        b.extend(v.to_le_bytes());
    }
    b.extend([1, 0]); // Community NPC movement state; position/heading are authoritative.
    b
}
fn relevance(guids: &[u64], guide: bool) -> Vec<u8> {
    let mut b = header(35, 12);
    b.extend((guids.len() as i32).to_le_bytes());
    for (i, g) in guids.iter().enumerate() {
        b.extend(g.to_le_bytes());
        b.extend([1, if guide && i == 0 { 5 } else { 1 }, 0]);
    }
    b
}
fn remove(guid: u64) -> Vec<u8> {
    crate::abilities::effect_packets::encode_remove_gracefully(guid, 0)
}
fn replace_string(b: &mut Vec<u8>, offset: usize, s: &str) {
    b.splice(
        offset..offset + 4,
        (s.len() as u32).to_le_bytes().into_iter().chain(s.bytes()),
    );
}
fn menu(guid: u64, name: &str, entries: &[(i32, i32)], auto: bool) -> Vec<u8> {
    let mut b = header(26, 9);
    b.extend(guid.to_le_bytes());
    b.push(u8::from(auto));
    b.extend((entries.len() as i32).to_le_bytes());
    for (id, label) in entries {
        for v in [*id, 0, *label, 0, 0, 0, 0] {
            b.extend(v.to_le_bytes());
        }
    }
    b.extend((name.len() as u32).to_le_bytes());
    b.extend(name.bytes());
    b.extend([1, 0]);
    b
}

/// Pinned Sanctuary PacketClientBeginZoning field order. Fourth components are
/// 1 for a point/identity quaternion; saved XYZ fields are retained on return.
pub fn zone_begin_packet(zone: &ZoneTransition) -> Vec<u8> {
    let mut b = 31_u16.to_le_bytes().to_vec();
    b.extend((zone.name.len() as u32).to_le_bytes());
    b.extend(zone.name.bytes());
    b.extend(2_i32.to_le_bytes());
    for v in zone
        .position
        .into_iter()
        .chain([1_f32])
        .chain(zone.rotation)
        .chain([1_f32])
    {
        b.extend(v.to_le_bytes());
    }
    b.extend((zone.sky.len() as u32).to_le_bytes());
    b.extend(zone.sky.bytes());
    b.extend([u8::from(zone.tutorial), 0]);
    b.extend((zone.zone_id as i32).to_le_bytes());
    b.extend(0_f32.to_le_bytes());
    b.extend(zone.geometry_id.to_le_bytes());
    b.extend([1, 0]);
    b
}

fn spawn_actor(template: &[u8], a: &Actor, position: [f32; 3], disposition: i32) -> Vec<u8> {
    let mut b = template.to_vec();
    b[4..12].copy_from_slice(&a.guid.to_le_bytes());
    b[12..16].copy_from_slice(&a.name_id.to_le_bytes());
    b[16..20].copy_from_slice(&a.model_id.to_le_bytes());
    if let Some(scale) = a.scale {
        // AddNpc: unknown flag (20), bubble colours and size (21..33), scale.
        b[33..37].copy_from_slice(&scale.to_le_bytes());
    }
    for (i, v) in position.into_iter().enumerate() {
        b[37 + i * 4..41 + i * 4].copy_from_slice(&v.to_le_bytes());
    }
    b[69..73].copy_from_slice(&0_i32.to_le_bytes());
    b[77..81].copy_from_slice(&disposition.to_le_bytes());
    b[128..132].copy_from_slice(&3.0_f32.to_le_bytes());
    b[133..137].copy_from_slice(&(if disposition == 2 { 12_i32 } else { 3_i32 }).to_le_bytes());
    b[137..141].copy_from_slice(&2_i32.to_le_bytes());
    b[141..145].copy_from_slice(&3_i32.to_le_bytes());
    b[145..149].copy_from_slice(&1_i32.to_le_bytes());
    b[233..237].copy_from_slice(&2_i32.to_le_bytes());
    replace_string(&mut b, 106, &a.name);
    replace_string(&mut b, 81, &a.texture_alias);
    b
}
