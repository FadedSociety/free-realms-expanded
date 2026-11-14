//! Local opening encounters over community-documented native packets.
//!
//! The supplied map, models and story are reference facts. Actor placements,
//! health, waves, damage, timing and rewards are explicitly reconstructed local
//! rules. Callers stage a clone, save it with inventory/progression, then commit.

use crate::{
    abilities::{EffectRecipient, JobEffect},
    character_store::CharacterState,
    npc_notifications::Notification,
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

mod acts;
mod arena;
mod arena_ai;
mod battle_drops;
mod battle_items;
mod card;
mod behaviour;
pub mod brain;
pub mod combat_rules;
mod combat_scale;
pub mod director;
pub mod facts;
mod field;
mod field_marker;
pub mod field_rewards;
pub mod kits;
pub mod pacing;
mod pickups;
mod power_up_use;
mod restore;
mod revive;
pub mod scale;
mod shared;
pub mod sim;
mod skills;
mod threat;
pub mod wire;
pub use self::arena::{MAX_MEMBERS, SharedArena};
use self::arena_ai::ArenaSim;
pub use self::behaviour::{Behaviour, Mode};
use self::combat_rules::CombatRules;
use self::combat_scale::CombatScale;
pub use self::facts::{FightFact, KillFact, Rank};
pub(crate) use self::field::level_refusal;
pub use self::card::MAX_PRIZE_QUANTITY;
use self::card::{Prize, details_packet, prize_bundle, wheel_rows};
use self::field_marker::MarkerSetting;
use self::shared::FieldSim;
pub use self::shared::{SharedFieldAction, SharedFieldState};
use self::skills::{SkillBuff, SkillDot, SkillEffects};
pub use self::wire::{animation, zone_begin_packet};
// The packet writers live in `wire`; the sibling modules reach them through `super::*`.
#[allow(unused_imports)]
use self::wire::{
    attack_processed, boss_display, combat_flag, encounter_header, encounter_state,
    enemy_health, enemy_hp, game_over, header, menu, minigame, movement, objective,
    objective_complete, player_hp, relevance, remove, replace_string, respawn_hide, rules,
    left_unfinished, spawn_actor,
};
use crate::abilities::effect_packets::encode_play_composite_effect as composite_effect;

/// Upper bound for saved player health (level 20 is about 7,135, 8,205 for a Brawler).
const PLAYER_HEALTH_CAP: u32 = 20_000;

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
const BATTLE_JOB_XP: u32 = 50;
/// Job type of combat jobs in the profile definitions (field 3).
const COMBAT_PROFILE_TYPE: i32 = 2;
pub const GUIDE_GUID: u64 = 0x4f50_454e_0000_f001;
/// Power-up pickup actors get `"PWUP"` plus a per-player serial (R). They are
/// personal to the killer, so the serial only has to be unique per engine.
const PICKUP_GUID_BASE: u64 = 0x5057_5550_0000_0000;
/// True for a power-up pickup actor guid; those live in one engine only (never fanned out).
pub fn is_pickup_guid(guid: u64) -> bool {
    guid & 0xFFFF_FFFF_0000_0000 == PICKUP_GUID_BASE
}
/// SFX_PowerUp_Pickup composite (ActorCompositeEffectDefinitions.xml 15509,
/// Combat Power-ups note 1.1); played at the pickup when it is collected.
const POWER_UP_PICKUP_EFFECT: i32 = 15509;
/// The client's own tutorial line, shown when a second stored kind is refused.
const ONE_POWER_UP_TEXT: &str = "You can have one power up active at a time.";
/// (R) The points a finished goal is worth (its 45/3 award), the `per_objective` default of
/// a fight's score; the results card adds it to kills, time and knockouts left
/// (`director::bind::results_card`).
const OBJECTIVE_AWARD_SCORE: i32 = 5000;
const EXIT: i32 = 101;
const RETRY: i32 = 102;
const ATTACK: i32 = 103;
const START: i32 = 104;
/// A field read past a length its branch already checked: unreachable, refused, not a panic.
const SHORT: &str = "encounter request ended early";

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
    /// The director's saved run, so a relog resumes the fight (`director::RunSave`, P5's).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_save: Option<Box<director::RunSave>>,
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

#[derive(Clone, Debug, Default, Deserialize)]
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
    /// Server-side content band, never sent to the client. The field and
    /// world/dungeon loaders scale authored health by it (`CombatScale`); only
    /// the field path scales damage too (R).
    /// Absent means level 1; rows with an explicit level are scaled by it (T737).
    #[serde(default = "one")]
    level: u32,
    boss: bool,
    #[serde(default)]
    stages: Vec<u32>,
    /// Summoned minion: absent until the encounter's summoner drops below this percentage of its
    /// health, then appears in a puff (the owner's Troll Summoner notes). Not a goal count.
    #[serde(default)]
    summon_below_percent: Option<u32>,
    /// Skittish (the Troll Summoner, after the owner's notes and design): fights in melee while
    /// its summoned minions stand; alone, it runs from the player, rests, then runs again (R).
    #[serde(default)]
    skittish: bool,
    /// EV: an `event_calendar.json` key; a field enemy or battle starter is there only while
    /// that event runs (`SharedFieldState::set_active_quests`).
    #[serde(default)]
    live_event: Option<String>,
    /// AddNpc scale; the summoned "small trolls" are shrunken troll models.
    #[serde(default)]
    scale: Option<f32>,
    /// A `behaviours.json` preset name, or `{"preset": ..., overrides}`.
    #[serde(default)]
    behaviour: Option<serde_json::Value>,
    /// Drop table id in `power_ups.json`; absent means this enemy drops nothing (training
    /// dummies and summoned minions stay off the table so they cannot be farmed).
    #[serde(default)]
    power_up_table: Option<String>,
    // The rows below are read by the combat pieces only; the engine itself reads `dormant`.
    /// The catalogue rank (P4); absent follows `boss` (Boss or Normal). A battle row may name
    /// its own when its kind has no catalogue row (`CombatScale::rank_enemy`).
    #[serde(default)]
    rank: Option<Rank>,
    /// Tags that goals, rules and kill facts name (P3, P5, P6).
    #[serde(default)]
    #[allow(dead_code)]
    tags: Vec<String>,
    /// Absent when the fight starts, until woken; unlike `summon_below_percent` it counts to goals.
    #[serde(default)]
    dormant: bool,
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
    /// The wheel's coin slot (the wiki's "N Coins").
    #[serde(default)]
    wheel_coins: u32,
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
    /// Owner 2026-11-12: an `interactable` world object's guid -> (its position, the starter or
    /// entrance guid its click opens), from `npc_spawns_extra.json` and `npcs/*.json`.
    #[serde(skip)]
    object_links: std::collections::BTreeMap<u64, ([f32; 3], u64)>,
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
    /// The enemy exists only while one of these quests is active (The owner's
    /// field notes: overworld enemies belong to active quests, like the
    /// opening's stage-gated wolves). Empty means always present.
    #[serde(default)]
    quest_ids: Vec<u32>,
    /// T879: this enemy's combat marker; absent follows the file's shared row.
    #[serde(default)]
    combat_marker: MarkerSetting,
    #[serde(default)]
    #[allow(dead_code)]
    source: serde_json::Value,
    /// Owner 2026-11-12: clicking this enemy opens `instance`'s start card instead of a fight in
    /// the overworld (the original game fought every battle in an instance).
    #[serde(default)]
    instance_enemy: bool,
    /// Owner 2026-11-12: whether it notices and chases a player in the overworld. Off, it only
    /// answers a player who hurts it.
    #[serde(default = "attacks_in_overworld")]
    attacks_in_overworld: bool,
    /// The starter or entrance guid whose start card an `instance_enemy` click opens.
    #[serde(default)]
    instance: Option<u64>,
}

fn attacks_in_overworld() -> bool {
    true
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
    /// T879 (R): the shared combat marker row; absent sends none (`name_id` 0 = the actor's own).
    #[serde(default)]
    combat_marker: Option<Notification>,
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
    /// The resolved combat marker sent after this enemy's 35/2 (T879).
    marker: Option<Notification>,
    /// L083: who hurt it and by how much (Sanctuary `MobBrain`).
    threat: threat::Threat,
    /// Owner 2026-11-12: the starter or entrance its click opens (`instance_enemy` rows only).
    instance: Option<u64>,
    /// Owner 2026-11-12: whether it notices players in the overworld.
    attacks: bool,
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
    /// T879: this battle's marker; absent follows the file's shared row.
    #[serde(default)]
    combat_marker: MarkerSetting,
    /// The resolved marker sent after the spawner's 35/2.
    #[serde(skip)]
    marker: Option<Notification>,
    /// The instanced arena the battle runs in (e.g. sg_random_encounter_clearing).
    zone: ZoneTransition,
    /// Start-screen picture (MiniGameInfo.IconId); e.g. Images.txt 4543
    /// icon_ui_combatstart_panelgraphic_trollsummonermadness.
    #[serde(default = "no_icon")]
    icon_id: i32,
    #[serde(default)]
    prizes: Vec<Prize>,
    #[serde(default)]
    wheel_coins: u32,
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

fn one() -> u32 {
    1
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
    /// T879 (R): the shared battle marker row; absent sends none.
    #[serde(default)]
    combat_marker: Option<Notification>,
    battles: Vec<FieldBattle>,
}

#[derive(Clone, Debug, Default)]
pub struct EncounterReward {
    pub coins: u32,
    pub items: Vec<(i32, u32)>,
    /// Chance drops: granted only if the bag has room; a full bag loses the drop silently and never the rest. The developer `realm win` (debug_win) clears them, so the dev route never grants one.
    pub bonus_items: Vec<(i32, u32)>,
    /// Experience for the active (combat) job: battles level combat jobs.
    pub job_xp: u32,
    /// L927: the part of `job_xp` that is one field kill's row XP (0 for a row that pays none). The
    /// active job's per-kill `activity_xp` takes its place when the payout is applied.
    pub kill_xp: u32,
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
    /// T789: accepted job-ability casts as (ability definition id, the primary target's
    /// enemy definition id; `None` for a cast with no enemy target). Refusals add nothing.
    pub abilities_used: Vec<(i32, Option<u32>)>,
    /// One fact per entry of `defeated`, in the same order.
    pub kills: Vec<KillFact>,
    /// What happened in the fight other than kills.
    pub facts: Vec<FightFact>,
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
    /// L085: a potion's heal still to come (session only, like energy).
    heal_over_time: Option<restore::HealOverTime>,
    /// Damage, health and tier levels from `combat_scale.json`.
    scale: CombatScale,
    /// Timing and chance rules from `combat_rules.json`.
    rules: Arc<CombatRules>,
    /// The fight run in progress (`director`).
    run: director::RunSlot,
    // One slot per combat piece for its loaded content and session state. Each type is its
    // owner's to fill; they outlive `reset_session` (clear what must not in your own hooks).
    // Boxed: the engine is moved by value on threads near their stack limit, so each slot
    // costs it one pointer however much its owner puts inside.
    #[allow(dead_code)]
    wire_state: Box<wire::WireState>,
    kit_state: Box<kits::KitState>,
    #[allow(dead_code)]
    brain_state: Box<acts::BrainState>,
    #[allow(dead_code)]
    fight_state: Box<director::FightState>,
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
    /// One run's enemy clocks and walks; moved in and out with `enter_arena` / `leave_arena`.
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
    // (rebuilt whenever content or presets load; shared, as the engine is cloned every tick).
    behaviour_presets: BTreeMap<String, Behaviour>,
    behaviour_of: Arc<BTreeMap<u64, Behaviour>>,
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

use crate::terrain::MAX_SNAP;

impl Ground {
    /// Snap a position onto the ground (`Terrain::snapped`, the one rule).
    fn snap(&self, p: &mut [f32; 3]) {
        if let Some((t, zone)) = self.0.as_ref() {
            *p = t.snapped(*zone, *p);
        }
    }
    fn snapped(&self, mut p: [f32; 3]) -> [f32; 3] {
        self.snap(&mut p);
        p
    }
    /// Field motion must not cross missing terrain or snap over a ledge (R); where it may, the
    /// step stands on the one ground rule. Without a terrain grid, retain the authored height.
    fn field_position(&self, p: [f32; 3]) -> Option<[f32; 3]> {
        let Some((terrain, zone)) = &self.0 else {
            return Some(p);
        };
        let height = terrain.height_at(*zone, p[0], p[2])?;
        ((height - p[1]).abs() <= MAX_SNAP).then(|| terrain.snapped(*zone, p))
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
        let rules = CombatRules::load(directory)?;
        let bytes = fs::read(directory.join("encounters.json"))?;
        if bytes.len() > 128 * 1024 {
            return Err(invalid("encounter content too large"));
        }
        let mut content: Content =
            serde_json::from_slice(&bytes).map_err(|e| invalid(&e.to_string()))?;
        // The same local-filename rule the other template descriptors use.
        content.template =
            crate::startup_snapshot::embedded_bytes(directory, &content.template_file, "", 4096)?;
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
        for a in (content.world_enemies.iter_mut())
            .chain(content.dungeons.iter_mut().flat_map(|d| &mut d.enemies))
        {
            scale.scale_world_enemy(a);
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
            energy: rules.energy.max,
            rules,
            run: director::RunSlot::default(),
            wire_state: Box::default(),
            kit_state: Box::default(),
            brain_state: Box::default(),
            fight_state: Box::default(),
            energy_regen_at: 0,
            heal_over_time: None,
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
            behaviour_of: Arc::default(),
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
        engine.load_wire(directory)?;
        engine.load_scale(directory)?;
        engine.load_kits(directory)?;
        engine.load_brains(directory)?;
        engine.load_fights(directory)?;
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
        let (health, crit) = self.rules.job_identity(profile);
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
        self.energy = self.rules.energy.max;
        let mut out = vec![
            player_hp(self.max_health, self.max_health),
            crate::abilities::build_energy(self.energy, self.energy),
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
        for _ in 0..256 {
            let mut standing = self.standing_enemies();
            if standing.is_empty() {
                // L1187 (lead): a paced battle row brings its roster back after a gap (F2's
                // pacing); the debug win plays the waiting rounds out now instead of stopping.
                let Some(due) = self.round_due() else {
                    break;
                };
                let mut round = EncounterOutcome::default();
                self.tick_rounds(due, &mut round);
                skills::merge_skill_outcome(&mut total, round);
                if self.round_due().is_some() {
                    break;
                }
                continue;
            }
            standing.sort_by_key(|(_, _, shielded)| *shielded);
            let (guid, position, _) = standing[0];
            self.last_attack = None;
            let out = self.strike(guid, position, f32::MAX, u32::MAX / 4, now_ms)?;
            skills::merge_skill_outcome(&mut total, out);
        }
        // T658 review: `realm win` is a debug shortcut, not a loot route, so it
        // pays no chance drop (`realm give` grants one for inspection).
        for reward in &mut total.rewards {
            reward.bonus_items.clear();
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
            run_save: None,
        });
        let (zone, guided) = (d.zone.clone(), d.guide.is_some());
        self.state.next_instance_id = next;
        self.state.player_health = self.max_health;
        self.reset_session();
        self.fight_log.clear();
        let feedback = if guided {
            "Entering the opening cave. Combat waves, balance and rewards are locally reconstructed."
        } else {
            "Entering the battle. Enemy behaviour, balance and rewards are locally reconstructed."
        };
        Ok(EncounterOutcome {
            changed: true,
            zoning: Some(zone),
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
                packets.extend(enemy_health(a.guid, s.health, a.max_health));
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

    pub fn on_zone_ready(&mut self, _profile_id: i32, now_ms: u64) -> io::Result<EncounterOutcome> {
        let Some(active) = self.state.active.as_mut() else {
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
        };
        let loading = active.phase == EncounterPhase::Loading;
        if loading {
            active.phase = EncounterPhase::Fighting;
        }
        self.armed = false;
        self.results_presented = false;
        self.last_tick = Some(now_ms);
        let (Some(active), Some(d)) = (self.state.active.as_ref(), self.dungeon()) else {
            return Err(invalid("unknown encounter"));
        };
        let mut out = EncounterOutcome {
            changed: loading,
            ..Default::default()
        };
        out.replies.push(self.details(d, active, true));
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
                out.replies.extend(enemy_health(a.guid, s.health, a.max_health));
                guids.push(a.guid);
                if a.boss {
                    out.replies.push(boss_display(a.guid, true));
                }
            }
        }
        out.replies.push(relevance(&guids, true));
        if !self.run_goals(&mut out) {
            let (count, goal) = goal_progress(d, active);
            out.replies.extend(card::goal_opened(&card::goal_row(d.objective_id, false, goal)));
            out.replies.extend(card::goal_counted(d.objective_id, count));
        }
        if active.phase == EncounterPhase::Won {
            out.replies.extend(card::goal_met(d.objective_id));
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
        let fighting = active.phase == EncounterPhase::Fighting;
        if d.guide.is_none() && fighting {
            self.armed = true;
        }
        if loading {
            self.run_begin(now_ms, &mut out);
        } else if fighting && self.run.run.is_none() {
            // A relog (or any reload) into a running fight whose run is not in memory.
            self.run_resume(now_ms, &mut out);
        }
        self.enemies_spawned(&guids, now_ms, &mut out);
        self.present_results(now_ms, &mut out);
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
        let Some(op) = crate::le::u16(inner, 0) else {
            return Ok(EncounterOutcome::default());
        };
        if !valid_position(position) {
            return Err(invalid("invalid encounter player position"));
        }
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
        let Some(sub) = crate::le::u16(inner, 2) else {
            return if matches!(op, 26 | 36 | 41) {
                Err(invalid("truncated encounter family header"))
            } else {
                Ok(EncounterOutcome::default())
            };
        };
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
                    let encounter = crate::le::u32(inner, 4);
                    if (encounter == Some(0) || encounter == Some(a.encounter_id))
                        && crate::le::i32(inner, 8) == Some(a.instance_id)
                    {
                        return self.leave(now_ms);
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
                let guid = crate::le::u64(inner, 4).ok_or_else(|| invalid(SHORT))?;
                if self.is_battle_spawner(guid) {
                    return Ok(self.battle_screen(guid, position, profile_id));
                }
                if let Some(card) = self.linked_battle_screen(guid, position, profile_id) {
                    return Ok(card);
                }
                Ok(self.interact(guid, position, now_ms))
            }
            (26, 10) => {
                if inner.len() != 16 {
                    return Err(invalid("malformed encounter selection"));
                }
                let guid = crate::le::u64(inner, 4).ok_or_else(|| invalid(SHORT))?;
                let choice = crate::le::i32(inner, 12).ok_or_else(|| invalid(SHORT))?;
                if let (Some(d), Some(active)) = (self.dungeon(), &self.state.active) {
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
                            && active.phase == EncounterPhase::Fighting
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
                let bar = crate::le::i32(inner, 4).ok_or_else(|| invalid(SHORT))?;
                let slot = crate::le::i32(inner, 8).ok_or_else(|| invalid(SHORT))?;
                let target_type = crate::le::i32(inner, 12).ok_or_else(|| invalid(SHORT))?;
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
                let guid = crate::le::u64(inner, 16).ok_or_else(|| invalid(SHORT))?;
                self.basic_cast(guid, player_guid, position, profile_id, slot, now_ms)
            }
            _ => Ok(EncounterOutcome::default()),
        }
    }

    fn interact(&mut self, guid: u64, position: [f32; 3], now_ms: u64) -> EncounterOutcome {
        if let Some(guide) = self.dungeon().and_then(|d| d.guide.as_ref())
            && let Some(active) = &self.state.active
            && guid == guide.guid
            && distance(position, guide.position) <= 12.0
        {
            let phase = &active.phase;
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
        if let Some((a, s)) = self.enemy(guid).filter(|(_, s)| s.health > 0) {
            // T889: a click from farther than the 3 m reach (R) answers a line, not silence.
            return if distance(position, s.position) <= 3.0 {
                EncounterOutcome {
                    replies: vec![menu(guid, &a.name, &[(ATTACK, 22498)], true)],
                    ..Default::default()
                }
            } else {
                EncounterOutcome {
                    feedback: Some("Move closer to attack.".into()),
                    ..Default::default()
                }
            };
        }
        // Not the guide and not a standing enemy: a fight actor (door, captive, talker), if any.
        let mut out = EncounterOutcome::default();
        self.actor_talked(guid, position, now_ms, &mut out);
        out
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
            self.dungeon()?
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
        self.basic_strike(guid, position, profile_id, slot, now_ms)
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
                    .is_some_and(|last| now_ms.saturating_sub(last) < self.rules.basic_gate_ms)
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
                * (self.rules.crit_multiplier() + f64::from(self.skills.crit_damage_bonus())))
                as u32
        } else {
            damage
        };
        let dealt = damage.min(health);
        let hp = health - dealt;
        if let (Some(member), true) = (self.arena.acting, dealt > 0) {
            self.arena.last_hit.insert(guid, member);
        }
        self.add_threat(guid, dealt);
        self.skills.wake(guid, dealt);
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
            let Some(target) = states.iter_mut().find(|s| s.guid == guid) else {
                return Err(invalid("unknown encounter enemy"));
            };
            target.health = hp;
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
            replies: self.player_hit_packets(wire::PlayerHit {
                attacker: self.shared_sender.unwrap_or(1),
                target: guid,
                dealt,
                health: hp,
                max_health,
                critical,
                swing,
                ..Default::default()
            }),
            ..Default::default()
        };
        self.enemy_hit_taken(guid, dealt, hp, max_health, now_ms, &mut out);
        // The brain may keep an enemy at zero health alive (`acts::Death`); then nothing below runs.
        let death = if hp == 0 {
            self.enemy_died(guid, now_ms, &mut out)
        } else {
            acts::Death::Survive
        };
        if death != acts::Death::Survive {
            self.skills.forget_target(guid);
            if self.shared_sender.is_none() && death == acts::Death::Die {
                let delay = self.rules.death_delay(field_index.is_some());
                out.replies.push(wire::death(guid, delay));
            }
            if boss {
                out.replies.push(boss_display(guid, false));
            }
            // T256: the shared field also credits the killer's own quests.
            self.record_kill(definition, guid, boss, false, &mut out);
            self.arena.forget(guid);
            // A director run owns its goals and its win; without one the single legacy goal runs.
            let directed = self.run_kill(guid, now_ms, &mut out);
            if let Some(i) = field_index {
                self.field_sim.moving.remove(&guid);
                self.field_sim.attacks.remove(&guid);
                let f = &mut self.field_sim.enemies[i];
                f.respawn_at = Some(now_ms.saturating_add(f.respawn_ms));
            } else if let (Some(active), Some(d)) = (&self.state.active, self.dungeon()) {
                let (count, goal) = goal_progress(d, active);
                if !directed {
                    out.replies.extend(card::goal_counted(d.objective_id, count));
                }
                if !directed && count >= goal && !self.next_round(now_ms) {
                    self.win_run(now_ms, &mut out);
                }
            } else if !self.world_stage_fighting() && !self.field_combat {
                out.replies.extend([
                    rules(false),
                    combat_flag(132, 0, 0, false),
                    combat_flag(133, 0, 0, false),
                ]);
            }
        }
        if death != acts::Death::Survive {
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
                self.pace_heal_drop(guid, killed_at, now_ms, &mut out);
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
        self.present_results(now_ms, &mut out);
        Ok(out)
    }

    /// Heal `percent` of max health and show the "+N" number. Says nothing when
    /// nothing was healed (full health, or knocked out: healing never revives).
    fn heal_with_number(&mut self, percent: u32, out: &mut EncounterOutcome) {
        let before = self.state.player_health;
        let replies = self.heal_player_percent(percent);
        let healed = self.state.player_health.saturating_sub(before) as i32;
        if healed > 0 {
            out.replies.extend(replies);
            let player = self.shared_sender.unwrap_or(1);
            let (max, current) = (self.max_health as i32, self.state.player_health as i32);
            out.replies.push(wire::hitpoints(
                player, player, true, max, current, healed, false,
            ));
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
            self.energy = self
                .energy
                .saturating_add(energy as i32)
                .min(self.rules.energy.max);
            out.replies.push(crate::abilities::build_energy(
                self.energy,
                self.rules.energy.max,
            ));
        }
        out.replies.push(composite_effect(
            self.shared_sender.unwrap_or(1),
            POWER_UP_PICKUP_EFFECT,
            drop.position,
        ));
        true
    }

    pub fn tick(
        &mut self,
        position: [f32; 3],
        profile_id: i32,
        now_ms: u64,
    ) -> io::Result<EncounterOutcome> {
        let mut out = self.tick_encounter(position, profile_id, now_ms)?;
        self.tick_rounds(now_ms, &mut out);
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
            if now_ms >= at.saturating_add(self.rules.revive.auto_ms) {
                return Ok(self.revive_at_safe_location(now_ms));
            }
        }
        if let Some(exit) = self.lost_run_exit(now_ms) {
            return exit;
        }
        // The run's own clock goes first, on every call: a run keeps time while the player is
        // down, the fight is not armed yet, or less than a tick has passed.
        let mut energy = EncounterOutcome::default();
        self.run_tick(position, now_ms, &mut energy);
        if self
            .state
            .active
            .as_ref()
            .is_some_and(|a| a.phase != EncounterPhase::Fighting)
            || (self.active() && !self.armed)
            || self.state.player_health == 0
        {
            return Ok(energy);
        }
        let elapsed = self.last_tick.map_or(100, |t| now_ms.saturating_sub(t));
        if elapsed < 100 {
            return Ok(energy);
        }
        self.last_tick = Some(now_ms);
        self.tick_job_effects(now_ms, &mut energy)?;
        self.tick_heal_over_time(now_ms, &mut energy);
        if self
            .state
            .active
            .as_ref()
            .is_some_and(|a| a.phase != EncounterPhase::Fighting)
        {
            return Ok(energy);
        }
        self.regenerate_energy(now_ms, &mut energy);
        let mut out = EncounterOutcome::default();
        out.changed = energy.changed;
        out.rewards.append(&mut energy.rewards);
        out.defeated.append(&mut energy.defeated);
        out.kills.append(&mut energy.kills);
        out.facts.append(&mut energy.facts);
        out.replies.append(&mut energy.replies);
        let mut total_damage = self.tick_solo_arena(position, now_ms, elapsed, &mut out);
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
        self.take_enemy_damage(total_damage, now_ms, &mut out);
        self.present_results(now_ms, &mut out);
        Ok(out)
    }

    fn present_results(&mut self, now_ms: u64, out: &mut EncounterOutcome) {
        if self.results_presented {
            return;
        }
        let (Some(active), Some(d)) = (&self.state.active, self.dungeon()) else {
            return;
        };
        if matches!(active.phase, EncounterPhase::Won | EncounterPhase::Lost) {
            if active.phase == EncounterPhase::Lost {
                // The native target listens to the boss registry even over
                // results. Unregister living bosses without killing/removing
                // them; Retry's normal spawn registers their display again.
                for (actor, state) in d.enemies.iter().zip(&active.enemies) {
                    if actor.boss && state.health > 0 {
                        out.replies.push(boss_display(actor.guid, false));
                    }
                }
            }
            // The original Flash screen automatically closes if Lua has no
            // score/stat/reward pages. Populate its score data before Show.
            out.replies.push(self.results_card(active, now_ms));
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
        self.fight_state.lost_since = None;
        let mut zone = self.content.world.clone();
        zone.zone_id = a.return_location.zone_id;
        zone.position = a.return_location.position;
        zone.rotation = a.return_location.rotation;
        let remove_state = left_unfinished(a.instance_id, a.encounter_id);
        let results_presented = self.results_presented;
        let mut replies = vec![
            rules(false),
            combat_flag(132, 0, 0, false),
            combat_flag(133, a.encounter_id, a.instance_id, false),
        ];
        if !results_presented {
            replies.extend(remove_state);
        }
        replies.push(player_hp(self.max_health, self.max_health));
        self.state.player_health = self.max_health;
        self.reset_session();
        self.run_closed();
        Ok(EncounterOutcome {
            changed: true,
            zoning: Some(zone),
            completed: (a.phase == EncounterPhase::Won).then_some(a.encounter_id),
            replies,
            ..Default::default()
        })
    }

    pub fn retry(&mut self, _profile_id: i32, _now_ms: u64) -> io::Result<EncounterOutcome> {
        // A dungeon exists exactly while an encounter is active (`dungeon`).
        let Some(d) = self.dungeon() else {
            return Ok(EncounterOutcome::default());
        };
        let enemies = d.enemies.iter().map(enemy_state).collect();
        let lost = |a: &&mut ActiveEncounter| a.phase == EncounterPhase::Lost;
        let Some(a) = self.state.active.as_mut().filter(lost) else {
            return Ok(EncounterOutcome::default());
        };
        a.enemies = enemies;
        a.phase = EncounterPhase::Loading;
        self.fight_state.lost_since = None;
        self.state.player_health = self.max_health;
        self.reset_session();
        self.run_closed();
        Ok(EncounterOutcome {
            changed: true,
            zoning: self.zone(),
            ..Default::default()
        })
    }

    /// An engine actor; a hostile one (disposition 0) is an enemy, in the open world or an
    /// instance, and draws no press-X prompt.
    fn spawn(&self, a: &Actor, position: [f32; 3], disposition: i32) -> Vec<u8> {
        spawn_actor(&self.content.template, a, position, disposition, disposition == 0)
    }

    /// A hostile-flagged actor that is not an enemy (the exit door, made friendly at once): it
    /// keeps its press-X prompt.
    fn spawn_object(&self, a: &Actor, position: [f32; 3], disposition: i32) -> Vec<u8> {
        spawn_actor(&self.content.template, a, position, disposition, false)
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
        || !(1..=20).contains(&a.level)
        || a.stages.len() > 32
        || a.power_up_table
            .as_ref()
            .is_some_and(|t| t.is_empty() || t.len() > 64)
        || a.tags.len() > 16
        || a.tags.iter().any(|t| t.is_empty() || t.len() > 64)
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
        dormant: a.dormant || a.summon_below_percent.is_some(),
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
