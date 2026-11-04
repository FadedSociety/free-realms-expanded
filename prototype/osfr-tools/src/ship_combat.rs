//! L963: the ship combat packet family (Pirate's Plunder), client family 94. The client's
//! `BaseClient.ShipCombatProcessor` dispatches it at `ShipCombat_DispatchInbound` 00a6d270 after the
//! header `sub_A5A0A0`: u16 family 94, u16 sub-op (little-endian), then the body. Every sub-op id
//! here is a constructor's own (`BaseShipCombatPacket` 00a4ec00) and matches the community enum
//! `BaseShipCombatPackets` (OpenSourceFreeRealms `LibSOE/Core/PacketCodes.cs`). Research and the
//! slice plan: the lead's note `minigames/pirates-plunder.md`.
//!
//! ZoneConfig's wire order is the client reader `sub_A69CF0`; each field's name and type come from
//! the server loader compiled into the client, `ShipZoneConfigLoad` 010680f0, joined by struct
//! offset. The live per-zone values never shipped, so every value is content (R).
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    fs,
    io::{self, Read},
    path::Path,
};

pub const FAMILY: u16 = 94;
/// Pirate's Plunder's MiniGameData row: the content of Games row 1059 (entry type 1, category 20).
pub const GAME_ID: u32 = 54;
pub const UPDATE_GAME_STATE: u16 = 2;
pub const UPDATE_PLAYER_GAME_STATES: u16 = 3;
pub const SET_PLAYER_COUNT: u16 = 7;
pub const CREATE_PROXIED_SHIP: u16 = 9;
pub const ZONE_CONFIG: u16 = 12;
pub const CAMERA_CONFIG: u16 = 13;
/// The longest string these packets carry (R; the readers take any non-negative length).
const MAX_TEXT: usize = 128;
/// (R) Sanity bound on each match timing (countdown, report timeout, `PlayTime`), in seconds.
const MAX_SECONDS: f64 = 3600.0;
/// The client's pirate colours (`PirateColorFromName` 00c9aa40); the ship actors' texture aliases
/// carry the same names.
const COLOURS: [&str; 8] = [
    "pir_red",
    "pir_green",
    "pir_blue",
    "pir_orange",
    "pir_violet",
    "pir_black",
    "pir_white",
    "pir_yellow",
];
/// CreateProxiedShip's two trailing strings are `SoeUtil::StringFixed<14>` (the packet's destructor
/// 00a679a0): 13 bytes and a terminator fit inline (longer ones spill to the heap, 00a55500, so this
/// keeps content to the client's own size, not a crash bound).
const MAX_FIXED: usize = 13;
pub const CONTENT_FILE: &str = "ship_combat.json";
/// Player status (35/20) that turns the client's ship combat processor on: bit 23 of the
/// character status word. `BaseClient::DispatchBasePlayerUpdate` 0092f460 shifts the word (actor
/// +0x6F8) by 0x17 and, while the processor is off, calls its on switch 00a6c630 (which sets the
/// dispatcher's active flag +16). Fishing's bit 22 the same way (`fishing_service`); 1 is the
/// baseline every exit restores.
pub const STATUS_SHIP_COMBAT: u32 = 0x80_0001;
pub const STATUS_NORMAL: u32 = 1;
const MAX_CONTENT: u64 = 64 * 1024;
/// Match states (the client's state machine 00a6b0c0).
pub const WAITING_FOR_CLIENTS: u32 = 3;
pub const WAITING_FOR_ASSETS: u32 = 5;
pub const WAITING_FOR_PHYSICS: u32 = 6;
pub const COUNTDOWN_READY: u32 = 8;
pub const COUNTDOWN: u32 = 9;
pub const PLAYING: u32 = 10;
pub const GAME_OVER: u32 = 12;
pub const EXITING: u32 = 13;
/// (R) How long Exiting shows before the zone back.
const EXIT_DELAY_MS: u64 = 2_000;
/// ZoneConfig keys outside `ZONE_FIELDS`.
const EXTRA_KEYS: &[&str] = &[
    "TreasureAreaPositionX",
    "TreasureAreaPositionZ",
    "UseShipHullTintForDockTint",
];

/// `ship_combat.json`: the arenas and the values ZoneConfig carries (R; the live values never
/// shipped), plus the server's own timings.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Content {
    pub schema_version: u32,
    /// Provenance note; never gameplay input.
    #[serde(default)]
    pub source: Option<String>,
    pub arenas: Vec<Arena>,
    /// (R) Seconds of countdown before Playing.
    pub countdown_seconds: u32,
    /// (R) Seconds a phase waits for the client's report before it moves on anyway.
    pub report_timeout_seconds: u32,
    /// ZoneConfig values by the loader's field names (`ZONE_FIELDS`); a missing key is 0.
    pub zone_config: BTreeMap<String, f64>,
    /// The player's ship (R).
    pub ship: Ship,
    /// The ships' colours by dock slot (`COLOURS`; R order).
    pub colours: Vec<String>,
    /// CameraConfig values by the loader's field names (`CAMERA_FIELDS`); a missing key is 0.
    pub camera: BTreeMap<String, f64>,
}

/// One ship's config as CreateProxiedShip carries it (the server loader
/// `LoadShipCombatShipProperties` 0106b940 names the fields). Values are (R).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Ship {
    /// HullActor, ForwardMastActor, RearMastActor, MidMastActor and MidMastTwoActor (the client's
    /// `pirship_*` actors).
    pub actors: [String; 5],
    /// CollisionDataAsset: the ship config's sixth string and the physics config's collision asset.
    pub collision: String,
    pub health: i32,
    pub mast_min_health: i32,
    pub mast_max_health: i32,
    pub hull_strength: f32,
    pub hold_capacity: i32,
    /// The ship's states by health (the config's 40-byte list at +71, `sub_A677C0`/`A5C090`): the
    /// client picks the first whose threshold the health is not below (`sub_1087AB0`) and indexes
    /// the list with it (`sub_C9E510`), so an empty list crashes it (native 2026-11-04); highest
    /// threshold first. The mast aliases are the actors' own; thresholds and effect ids are (R).
    pub damage_states: Vec<DamageState>,
    /// The physics config by the loader's names (`PHYSICS_FIELDS`); a missing key is 0.
    pub physics: BTreeMap<String, f64>,
    /// The physics config's gears, lowest first (`GEAR_FIELDS` each): the client's vehicle steps
    /// its current gear every frame (`sub_FF02A0`), so an empty list crashes it (native
    /// 2026-11-04).
    pub gears: Vec<BTreeMap<String, f64>>,
    /// The rear mast's flag, one of its texture aliases (`skullx01`, `anchor`, `octopus`...): the
    /// packet's last string. (R) The live game let the player choose (SelectShip, later).
    pub flag: String,
}

/// One ship state: a health threshold, the sails' look and an effect id (R).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DamageState {
    pub health: i32,
    /// The masts' texture alias for this state (the loader's `MastTextureAlias`; the mast actors
    /// carry `dmg00` to `dmg03`).
    pub mast_alias: String,
    pub effect_id: i32,
}

/// One arena zone and where a player starts in it.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Arena {
    /// The client scene (`ss_pirate_arena`, `ss_pir_arena3`, `ss_pir_arena4`).
    pub scene: String,
    /// (R) The zone id the transition names; one of our own.
    pub zone_id: u32,
    /// The docks (the arena's `Port`/`PortBell` areas); the first is the player's.
    pub docks: Vec<[f32; 3]>,
    /// (R) The spawn rotation (x, y, z, w), as the fishing holes send it.
    pub rotation: [f32; 4],
}

impl Content {
    /// `Ok(None)` without the file.
    pub fn load(directory: &Path) -> io::Result<Option<Self>> {
        let path = directory.join(CONTENT_FILE);
        if !path.is_file() {
            return Ok(None);
        }
        let mut text = String::new();
        fs::File::open(path)?
            .take(MAX_CONTENT + 1)
            .read_to_string(&mut text)?;
        if text.len() as u64 > MAX_CONTENT {
            return Err(invalid("ship combat content too large"));
        }
        let content: Self = serde_json::from_str(&text).map_err(invalid)?;
        // Every packet is written from content checked here, so the writers cannot fail.
        let text_ok = |text: &str| text.len() <= MAX_TEXT;
        let known = |key: &str| {
            ZONE_FIELDS.iter().any(|(name, _)| *name == key) || EXTRA_KEYS.contains(&key)
        };
        let arena_ok = |a: &Arena| {
            !a.scene.is_empty()
                && text_ok(&a.scene)
                && !a.docks.is_empty()
                && i32::try_from(a.zone_id).is_ok_and(|id| id > 0)
        };
        let ship = &content.ship;
        let ship_ok = (ship.actors.iter().chain([&ship.collision])).all(|t| text_ok(t))
            && ship.flag.len() <= MAX_FIXED
            && !ship.damage_states.is_empty()
            && (ship.damage_states.windows(2)).all(|w| w[0].health > w[1].health)
            && ship.damage_states.iter().all(|state| state.health >= 0)
            && ship
                .damage_states
                .iter()
                .all(|state| text_ok(&state.mast_alias))
            && only(&ship.physics, PHYSICS_FIELDS)
            && !ship.gears.is_empty()
            && ship.gears.iter().all(|gear| only(gear, GEAR_FIELDS));
        let play = content.zone_config.get("PlayTime").copied().unwrap_or(0.0);
        let timings = [
            f64::from(content.countdown_seconds),
            f64::from(content.report_timeout_seconds),
            play,
        ];
        if content.schema_version != 1
            || content.arenas.is_empty()
            || !content.arenas.iter().all(arena_ok)
            || !content.zone_config.keys().all(|key| known(key))
            || !only(&content.camera, CAMERA_FIELDS)
            || !ship_ok
            || content.colours.is_empty()
            || !(content.colours.iter()).all(|colour| COLOURS.contains(&colour.as_str()))
            || !timings.iter().all(|s| (0.0..=MAX_SECONDS).contains(s))
        {
            return Err(invalid("invalid ship combat content"));
        }
        Ok(Some(content))
    }
}

/// One match's phases on the server: each waiting state moves on when the client's report word
/// (UpdatePlayerGameStates, sub 3) carries its bit, or after the report timeout (R); the countdown
/// and the play time run on the clock. `advance` returns the states to send, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Match {
    state: u32,
    reported: u32,
    /// When the current state was sent; `None` until the arena has loaded.
    since_ms: Option<u64>,
    countdown_ms: u64,
    play_ms: u64,
    timeout_ms: u64,
}

impl Match {
    pub fn new(content: &Content) -> Self {
        let play = content.zone_config.get("PlayTime").copied().unwrap_or(0.0);
        Self {
            state: WAITING_FOR_CLIENTS,
            reported: 0,
            since_ms: None,
            countdown_ms: u64::from(content.countdown_seconds) * 1000,
            // `load` bounds it to 0..=MAX_SECONDS; `as` floors it.
            play_ms: (play * 1000.0) as u64,
            timeout_ms: u64::from(content.report_timeout_seconds) * 1000,
        }
    }

    pub fn state(&self) -> u32 {
        self.state
    }

    /// The arena loaded and state 3 went out with ZoneConfig.
    pub fn loaded(&mut self, now_ms: u64) {
        self.since_ms.get_or_insert(now_ms);
    }

    /// The client's report word; the states it unlocks.
    pub fn report(&mut self, bits: u32, now_ms: u64) -> Vec<u32> {
        self.reported |= bits;
        self.advance(now_ms)
    }

    /// The states due now (the clock, or a report already in).
    pub fn advance(&mut self, now_ms: u64) -> Vec<u32> {
        let mut sent = Vec::new();
        while let Some(since) = self.since_ms {
            let waited = now_ms.saturating_sub(since);
            let gate = |bit: u32| self.reported & bit != 0 || waited >= self.timeout_ms;
            let next = match self.state {
                WAITING_FOR_CLIENTS if gate(0x2) => WAITING_FOR_ASSETS,
                WAITING_FOR_ASSETS if gate(0x8) => WAITING_FOR_PHYSICS,
                WAITING_FOR_PHYSICS if gate(0x10) => COUNTDOWN_READY,
                COUNTDOWN_READY => COUNTDOWN,
                COUNTDOWN if waited >= self.countdown_ms => PLAYING,
                PLAYING if waited >= self.play_ms => GAME_OVER,
                GAME_OVER if gate(0x400) => EXITING,
                _ => break,
            };
            self.state = next;
            self.since_ms = Some(now_ms);
            sent.push(next);
        }
        sent
    }

    /// Exiting went out a moment ago (the client needs a beat to act on it before the zone back
    /// turns its processor off, R): the visit ends and the player zones back.
    pub fn finished(&self, now_ms: u64) -> bool {
        self.state == EXITING
            && self
                .since_ms
                .is_some_and(|since| now_ms.saturating_sub(since) >= EXIT_DELAY_MS)
    }
}

fn invalid(error: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

/// How one ZoneConfig field is written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Field {
    I32,
    F32,
    Bool,
    /// An angle the loader reads in degrees and the reader takes in radians.
    Degrees,
}

/// ZoneConfig's scalar fields after its name, in the reader's order (`sub_A69CF0`), named as the
/// loader names them. The debug flags (+188..+191) and the dock tint (+44) are written as bools.
const ZONE_FIELDS: &[(&str, Field)] = &[
    ("MaxPlayers", Field::I32),
    ("MaxScore", Field::I32),
    ("ShipPickupCount", Field::I32),
    ("UseShipTargetHighlight", Field::Bool),
    ("ZoneReadyWaitTime", Field::F32),
    ("PlayTime", Field::F32),
    ("RespawnTime", Field::F32),
    ("InvincibleAfterRespawnTime", Field::F32),
    ("TeleportTime", Field::F32),
    ("InvincibleAfterTeleportTime", Field::F32),
    ("MinDriftingTime", Field::F32),
    ("MaxDriftingTime", Field::F32),
    ("MaxSinkingTime", Field::F32),
    ("ParkingAlignTime", Field::F32),
    ("ParkingSelectTime", Field::F32),
    ("ParkingEnterSpeed", Field::F32),
    ("ParkingExitSpeed", Field::F32),
    ("ParkingAlignSpin", Field::F32),
    ("ParkingAlignPush", Field::F32),
    ("ParkingRepulsorTime", Field::F32),
    ("RepulsorPushSpeed", Field::F32),
    ("RepulsorPushAccel", Field::F32),
    ("RepulsorPushRange", Field::F32),
    ("ShoveOutSpeed", Field::F32),
    ("ShoveOutAccel", Field::F32),
    ("ZoneBubbleRadius", Field::I32),
    ("ZoneBubbleObjectRadius", Field::I32),
    ("ZoneBubbleFloraRadius", Field::I32),
    ("MaxDrawDistance", Field::F32),
    ("PhysicsSimulationRate", Field::F32),
    ("Gravity", Field::F32),
    ("WaterLevel", Field::F32),
    ("AllowablePhysicsDelta", Field::F32),
    ("PickupEmptyTime", Field::F32),
    ("PickupWarmupEffectId", Field::I32),
    ("PickupEmptyDoneEffectId", Field::I32),
    ("FlyingTreasureHeight", Field::F32),
    ("TreasureFillEffectId", Field::I32),
    ("RespawnSplashEffectId", Field::I32),
    ("WhirlpoolEffectId", Field::I32),
    ("WhirlpoolExitEffectId", Field::I32),
    ("TreasureXMarkEffectId", Field::I32),
    ("CameraShakeEffectId", Field::I32),
    ("ProjectileMax", Field::I32),
    ("FireballProjectileModelId", Field::I32),
    ("FireballImpactEffectId", Field::I32),
    ("SnowballProjectileModelId", Field::I32),
    ("SnowballImpactEffectId", Field::I32),
    ("GlueballProjectileModelId", Field::I32),
    ("GlueballImpactEffectId", Field::I32),
    ("GreaseballProjectileModelId", Field::I32),
    ("GreaseballImpactEffectId", Field::I32),
    ("PickupWastedEffectId", Field::I32),
    ("ProjectileModelId", Field::I32),
    ("ProjectileFireEffectId", Field::I32),
    ("ProjectileLandImpactEffectId", Field::I32),
    ("ProjectileWaterImpactEffectId", Field::I32),
    ("ProjectileTrailEffectId", Field::I32),
    ("ProjectileRadius", Field::F32),
    ("ProjectileLifetime", Field::F32),
    ("ProjectileDespawnEffectLifetime", Field::F32),
    ("ProjectileDespawnEffectRange", Field::F32),
    ("ShipSinkEffectId", Field::I32),
    ("ShipWhirlpoolEnterEffectId", Field::I32),
    ("ShipWhirlpoolExitEffectId", Field::I32),
    ("ShipRepairEffectId", Field::I32),
    ("ShipSpeedEffectId", Field::I32),
    ("ShipRapidFireEffectId", Field::I32),
    ("ShipLongShotEffectId", Field::I32),
    ("ShipFirepowerEffectId", Field::I32),
    ("ShipChestFullEffectId", Field::I32),
    ("ShipChestEmptyEffectId", Field::I32),
    ("ShipHoldMinValue", Field::I32),
    ("ShipNameplateVisibleRange", Field::F32),
    ("RamFullSpeed", Field::F32),
    ("RamFullDamage", Field::F32),
    ("RamSpeedThreshold", Field::F32),
    ("RamLowSlope", Field::F32),
    ("RamHighSlope", Field::F32),
    ("RamReturnScale", Field::F32),
    ("DebugDrawBanks", Field::Bool),
    ("DebugDrawHomeEntrance", Field::Bool),
    ("DebugDrawWhirlpools", Field::Bool),
    ("DebugDrawJetStreams", Field::Bool),
];

/// A ship combat packet: the family header, the sub-op, then `body`.
pub fn packet(sub: u16, body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + body.len());
    out.extend(FAMILY.to_le_bytes());
    out.extend(sub.to_le_bytes());
    out.extend(body);
    out
}

/// SetPlayerCount (7): how many human and AI ships the match holds (`sub_A5F030` keeps them at the
/// player list's +0x1c and +0x20). The client reports its clients ready (0x2) once it holds as many
/// human players as the first (`sub_A55310`).
pub fn player_count(humans: u32, ai: u32) -> Vec<u8> {
    packet(
        SET_PLAYER_COUNT,
        &[humans.to_le_bytes(), ai.to_le_bytes()].concat(),
    )
}

/// UpdateGameState (2): one u32 match state; the reader (`sub_A5ED80`) takes 1 to 14, and the
/// server sends only `Match`'s constants (the client's state machine 00a6b0c0 names 3 Waiting For
/// Clients, 5 Waiting For Assets, 6 Waiting For Physics, 7 Tutorial, 8 Countdown Ready,
/// 9 Countdown, 10 Playing, 12 Game Over and 13 Exiting).
pub fn game_state(state: u32) -> Vec<u8> {
    packet(UPDATE_GAME_STATE, &state.to_le_bytes())
}

/// A string as these readers take it: u32 length, then the bytes, cut at `MAX_TEXT` (content
/// checks every string against it at load).
fn string(body: &mut Vec<u8>, text: &str) {
    let bytes = &text.as_bytes()[..text.len().min(MAX_TEXT)];
    // At most MAX_TEXT, so the cast is exact.
    body.extend((bytes.len() as u32).to_le_bytes());
    body.extend(bytes);
}

/// ZoneConfig (12) for zone `name`: the fields of `ZONE_FIELDS` from `values` (a missing key is 0),
/// the treasure area (`TreasureAreaPositionX`/`Z`), the dock tint flag, then four empty lists (the
/// docks at +384 and the lists at +416, +432 and +448).
pub fn zone_config(name: &str, values: &BTreeMap<String, f64>) -> Vec<u8> {
    let value = |key: &str| values.get(key).copied().unwrap_or(0.0);
    let mut body = Vec::new();
    string(&mut body, name);
    write_fields(&mut body, ZONE_FIELDS, values);
    for key in ["TreasureAreaPositionX", "TreasureAreaPositionZ"] {
        body.extend((value(key) as f32).to_le_bytes());
    }
    body.push(u8::from(value("UseShipHullTintForDockTint") != 0.0));
    for _ in 0..4 {
        body.extend(0_u32.to_le_bytes());
    }
    packet(ZONE_CONFIG, &body)
}

/// Each field from `values` (a missing key is 0), as its kind is written.
fn write_fields(body: &mut Vec<u8>, fields: &[(&str, Field)], values: &BTreeMap<String, f64>) {
    for (key, field) in fields {
        let value = values.get(*key).copied().unwrap_or(0.0);
        match field {
            Field::I32 => body.extend((value as i32).to_le_bytes()),
            Field::F32 => body.extend((value as f32).to_le_bytes()),
            Field::Bool => body.push(u8::from(value != 0.0)),
            Field::Degrees => body.extend((value as f32).to_radians().to_le_bytes()),
        }
    }
}

/// Whether every key of `values` names one of `fields`.
fn only(values: &BTreeMap<String, f64>, fields: &[(&str, Field)]) -> bool {
    (values.keys()).all(|key| fields.iter().any(|(name, _)| name == key))
}

/// The ship physics config's fields in the reader's order (`sub_A66C00`, struct order), named as
/// the server loader names them (0106a560; defaults: the constructor 0106ae50). The reader stops
/// before the loader's contact and AI steering fields, which the server alone uses.
const PHYSICS_FIELDS: &[(&str, Field)] = &[
    ("SpeedFactor", Field::F32),
    ("Mass", Field::F32),
    ("CenterOfMassZ", Field::F32),
    ("Length", Field::F32),
    ("Width", Field::F32),
    ("Height", Field::F32),
    ("Spinniness", Field::F32),
    ("MaxSpin", Field::F32),
    ("MinSpeed", Field::F32),
    ("MaxSpeed", Field::F32),
    ("TurboSpeed", Field::F32),
    ("ExceedMaxSpeedTime", Field::F32),
    ("ReverseJetStreamSpeed", Field::F32),
    ("StationaryTurnRadius", Field::F32),
    ("StraightenWeight", Field::F32),
    ("StraightenMax", Field::F32),
    ("AutoAimRadius", Field::F32),
    ("DraftConeRadius", Field::F32),
    ("DraftConeAngle", Field::Degrees),
    ("DraftHeadingDiff", Field::Degrees),
    ("DraftSpdMultiplier", Field::F32),
    ("DraftAccel", Field::F32),
    ("CollisionMin", Field::F32),
    ("TerrainBounceDelay", Field::F32),
    ("TerrainBounciness", Field::F32),
    ("TerrainLowBounciness", Field::F32),
    ("TerrainLowBouncePush", Field::F32),
    ("TerrainBounceThreshhold", Field::F32),
    ("TerrainBounceSpin", Field::F32),
    ("ShipBounceDelay", Field::F32),
    ("ShipBounciness", Field::F32),
];

/// One gear's fields in the reader's order (`sub_A5C140`), named as the loader names them
/// (0106a3a0); the vehicle shifts on `ShiftUpSpeed`/`ShiftDownSpeed`.
const GEAR_FIELDS: &[(&str, Field)] = &[
    ("ShiftUpSpeed", Field::F32),
    ("ShiftDownSpeed", Field::F32),
    ("BaseAcceleration", Field::F32),
    ("BaseDeceleration", Field::F32),
    ("TurboAcceleration", Field::F32),
    ("BrakeDeceleration", Field::F32),
    ("SidewaysDeceleration", Field::F32),
    ("AngularAcceleration", Field::Degrees),
    ("TurboAngularAcceleration", Field::Degrees),
    ("AngularDeceleration", Field::Degrees),
    ("MaxAngularSpeed", Field::Degrees),
    ("TurboMaxAngularSpeed", Field::Degrees),
];

/// The ship camera's fields in the reader's order (`sub_A5AF10`), named as the server loader names
/// them (0106b1e0; its defaults are the constructor 0106b090's). The offset's fourth float and the
/// config's trailing name are not loader fields.
const CAMERA_FIELDS: &[(&str, Field)] = &[
    ("OffsetX", Field::F32),
    ("OffsetY", Field::F32),
    ("OffsetZ", Field::F32),
    ("TargetHeight", Field::F32),
    ("Distance", Field::F32),
    ("MinDistance", Field::F32),
    ("MaxDistance", Field::F32),
    ("Pitch", Field::Degrees),
    ("MinPitch", Field::Degrees),
    ("MaxPitch", Field::Degrees),
    ("OverHeadRenderDistance", Field::F32),
    ("OverHeadScale", Field::F32),
    ("OverHeadShipEffectYOffset", Field::F32),
    ("UseShipHeightForOverHead", Field::Bool),
    ("ZoneBubbleRadius", Field::I32),
    ("ZoneBubbleObjectRadius", Field::I32),
    ("ZoneBubbleFloraRadius", Field::I32),
    ("IndicatorScreenRatioX", Field::F32),
    ("IndicatorScreenRatioY", Field::F32),
    ("IndicatorSizeMod", Field::F32),
    ("IndicatorPulseSizeMod", Field::F32),
    ("IndicatorPulseTime", Field::F32),
    ("IndicatorPulseIntervalTime", Field::F32),
];

/// CameraConfig (13): a u32 the handler keeps at processor +0x280, then the camera block. The u32
/// is 1: `ShipCombatFrameUpdate` 00ca7480 sends the client's own ship reports (UpdateShipPhysics 16,
/// FireAtLocation 20 and the special move) only while it is set (0 kept them off, native 133931Z).
/// After it, the camera block
/// (`CAMERA_FIELDS`, the offset's fourth float 0 and an empty name). The handler `HandleCameraConfig`
/// 00a62140 keeps the block at processor +0x290 and points the ship camera (`ControllerShip` +0x140,
/// set by 00af75a0) at it; without it the camera reads a null config each frame (`sub_AF7850`,
/// native 2026-11-04), so it follows the local player's CreateProxiedShip.
pub fn camera_config(values: &BTreeMap<String, f64>) -> Vec<u8> {
    let mut body = 1_u32.to_le_bytes().to_vec();
    let (offset, rest) = CAMERA_FIELDS.split_at(3);
    write_fields(&mut body, offset, values);
    body.extend(0_f32.to_le_bytes());
    write_fields(&mut body, rest, values);
    body.extend(0_u32.to_le_bytes());
    packet(CAMERA_CONFIG, &body)
}

/// CreateProxiedShip (9) for `guid` in dock slot `slot` at `position` (reader 00a69b90; handler
/// 00a6c3b0 makes the ship proxy and, for a guid with no player yet, the player: the local
/// character's own guid becomes the processor's local player +5, whose reports (sub 3) then flow).
/// Layout:
/// - u64 guid, u8 (R), u32 slot (must be >= 0), then the ship's 4x4 transform as four rows (right,
///   up, forward, position with w 1): the client's frame update reads the position from the last
///   row (00ca7480, matrix +48) and echoes the same rows in UpdateShipPhysics (native 141908Z; a
///   position in the first row spawned the ship at the origin);
/// - the ship config (00a683d0): the five actors and the collision asset, 15 values (Health, MastMin/MaxHealth,
///   HullStrength, HoldCapacity, HoldUnloadPerSecond, HomeHealingPerSecond, PainZonePerSecond,
///   LongShotAngle, LongShotSpeed, +168, four effect ids), 7 values, a vec3, 5 values, a u8,
///   4 values, two lists and a string;
/// - the physics config (00a66c00): `PHYSICS_FIELDS`, the collision asset, the gears (each
///   `GEAR_FIELDS` and an empty name) and an empty name;
/// - the ship's colour name (`COLOURS`) and its flag alias (two `StringFixed<14>`).
/// Every other value is 0 and every other list is empty (R).
pub fn create_ship(guid: u64, slot: u32, position: [f32; 3], ship: &Ship, colour: &str) -> Vec<u8> {
    let mut b = Vec::new();
    let count = |n: usize| u32::try_from(n).unwrap_or(u32::MAX).to_le_bytes();
    let zeros = |b: &mut Vec<u8>, words: usize| b.extend(std::iter::repeat_n(0_u8, words * 4));
    b.extend(guid.to_le_bytes());
    b.push(0);
    b.extend(slot.to_le_bytes());
    let [x, y, z] = position;
    // (R) No turn: the identity rotation.
    let rows = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, x, y, z, 1.0,
    ];
    for value in rows {
        b.extend(value.to_le_bytes());
    }
    // The ship config.
    for actor in ship.actors.iter().chain([&ship.collision]) {
        string(&mut b, actor);
    }
    for value in [ship.health, ship.mast_min_health, ship.mast_max_health] {
        b.extend(value.to_le_bytes());
    }
    b.extend(ship.hull_strength.to_le_bytes());
    b.extend(ship.hold_capacity.to_le_bytes());
    zeros(&mut b, 10 + 7);
    zeros(&mut b, 3); // the vec3
    zeros(&mut b, 5);
    b.push(0);
    zeros(&mut b, 4);
    zeros(&mut b, 1); // the list at +67, empty (R)
    b.extend(count(ship.damage_states.len()));
    for state in &ship.damage_states {
        b.extend(state.health.to_le_bytes());
        string(&mut b, &state.mast_alias);
        b.extend(state.effect_id.to_le_bytes());
    }
    string(&mut b, "");
    // The physics config; its collision asset is the ship's own (R).
    write_fields(&mut b, PHYSICS_FIELDS, &ship.physics);
    string(&mut b, &ship.collision);
    b.extend(count(ship.gears.len()));
    for gear in &ship.gears {
        write_fields(&mut b, GEAR_FIELDS, gear);
        string(&mut b, "");
    }
    string(&mut b, "");
    string(&mut b, colour);
    string(&mut b, &ship.flag);
    packet(CREATE_PROXIED_SHIP, &b)
}

/// The client's UpdatePlayerGameStates (3): its report word (00a69480), or `None` for another packet.
pub fn client_states(packet: &[u8]) -> Option<u32> {
    let rest = packet
        .strip_prefix(&FAMILY.to_le_bytes())?
        .strip_prefix(&UPDATE_PLAYER_GAME_STATES.to_le_bytes())?;
    Some(u32::from_le_bytes(rest.get(..4)?.try_into().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zone_config_writes_every_field_in_the_readers_order() {
        let values = BTreeMap::from([
            ("MaxPlayers".to_owned(), 6.0),
            ("PlayTime".to_owned(), 180.0),
            ("DebugDrawJetStreams".to_owned(), 1.0),
            ("TreasureAreaPositionX".to_owned(), 761.0),
        ]);
        let out = zone_config("ss_pirate_arena", &values);
        assert_eq!(&out[..4], &[94, 0, 12, 0]);
        let name = 15;
        assert_eq!(&out[4..8], &(name as u32).to_le_bytes());
        let body = &out[8 + name..];
        assert_eq!(&body[..4], &6_i32.to_le_bytes(), "MaxPlayers first");
        // MaxPlayers, MaxScore, ShipPickupCount (12 bytes), the highlight flag (1), then
        // ZoneReadyWaitTime and PlayTime.
        assert_eq!(&body[17..21], &180_f32.to_le_bytes());
        let scalars: usize = ZONE_FIELDS
            .iter()
            .map(|(_, f)| if *f == Field::Bool { 1 } else { 4 })
            .sum();
        assert_eq!(body[scalars - 1], 1, "the last debug flag");
        assert_eq!(&body[scalars..scalars + 4], &761_f32.to_le_bytes());
        assert_eq!(
            body.len(),
            scalars + 8 + 1 + 16,
            "the treasure area, the tint and four lists"
        );
    }

    #[test]
    fn game_states_and_client_reports() {
        assert_eq!(game_state(10), [94, 0, 2, 0, 10, 0, 0, 0]);
        assert_eq!(player_count(1, 0), [94, 0, 7, 0, 1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(client_states(&[94, 0, 3, 0, 0x18, 0, 0, 0]), Some(0x18));
        assert_eq!(client_states(&[94, 0, 2, 0, 1, 0, 0, 0]), None);
        assert_eq!(client_states(&[94, 0, 3, 0, 1]), None);
    }

    fn content() -> Content {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
        Content::load(&dir)
            .unwrap()
            .expect("shipped ship_combat.json")
    }

    #[test]
    fn a_match_waits_for_each_report_then_runs_on_the_clock() {
        let content = content();
        let mut game = Match::new(&content);
        assert!(
            game.advance(1_000).is_empty(),
            "nothing before the arena loads"
        );
        game.loaded(1_000);
        assert!(game.advance(1_500).is_empty(), "waiting for the client");
        assert_eq!(game.report(0x2, 2_000), [WAITING_FOR_ASSETS]);
        assert_eq!(
            game.report(0x8 | 0x10, 2_100),
            [WAITING_FOR_PHYSICS, COUNTDOWN_READY, COUNTDOWN]
        );
        let countdown = u64::from(content.countdown_seconds) * 1000;
        assert!(game.advance(2_100 + countdown - 1).is_empty());
        assert_eq!(game.advance(2_100 + countdown), [PLAYING]);
        let play = (content.zone_config["PlayTime"] * 1000.0) as u64;
        assert_eq!(game.advance(2_100 + countdown + play), [GAME_OVER]);
        assert!(!game.finished(2_200 + countdown + play));
        let exit = 2_200 + countdown + play;
        assert_eq!(game.report(0x400, exit), [EXITING]);
        assert!(!game.finished(exit + 1_999), "Exiting shows first");
        assert!(game.finished(exit + 2_000));
    }

    #[test]
    fn a_silent_client_still_moves_on_after_the_timeout() {
        let content = content();
        let mut game = Match::new(&content);
        game.loaded(0);
        let timeout = u64::from(content.report_timeout_seconds) * 1000;
        assert_eq!(game.advance(timeout), [WAITING_FOR_ASSETS]);
        assert_eq!(game.advance(2 * timeout), [WAITING_FOR_PHYSICS]);
        assert_eq!(game.advance(3 * timeout), [COUNTDOWN_READY, COUNTDOWN]);
    }

    #[test]
    fn create_ship_matches_the_readers_shape() {
        let ship = content().ship;
        let out = create_ship(7, 0, [880.0, 52.0, 723.0], &ship, "pir_red");
        assert_eq!(&out[..4], &[94, 0, 9, 0]);
        assert_eq!(&out[4..12], &7_u64.to_le_bytes());
        let actors: usize = (ship.actors.iter().chain([&ship.collision]))
            .map(|a| 4 + a.len())
            .sum();
        // The four rows: the identity rotation, then the position with w 1.
        let rows: Vec<u8> = [
            1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 880., 52., 723., 1.,
        ]
        .iter()
        .flat_map(|v: &f32| v.to_le_bytes())
        .collect();
        let rows_at = 4 + 8 + 1 + 4;
        assert_eq!(&out[rows_at..rows_at + 64], &rows[..]);
        // guid, u8, slot, four vec4; the config: actors, 31 values, a vec3, a u8, two lists and a
        // string; the physics config: 31 values, the collision asset, the gears (12 values and a
        // name each) and a name; the colour and the flag.
        let states: usize = (ship.damage_states.iter())
            .map(|s| 4 + 4 + s.mast_alias.len() + 4)
            .sum();
        let physics = 4 * 31 + 4 + ship.collision.len() + 4 + ship.gears.len() * (48 + 4) + 4;
        let fixed = 8 + 1 + 4 + 64 + actors + 4 * 31 + 12 + 1 + 8 + states + 4 + physics;
        let tail = [
            &7_u32.to_le_bytes()[..],
            b"pir_red",
            &(ship.flag.len() as u32).to_le_bytes(),
            ship.flag.as_bytes(),
        ]
        .concat();
        assert!(out.ends_with(&tail), "the colour, then the flag, last");
        assert_eq!(out.len(), 4 + fixed + tail.len());
        let health_at = 4 + 8 + 1 + 4 + 64 + actors;
        assert_eq!(&out[health_at..health_at + 4], &ship.health.to_le_bytes());
    }

    #[test]
    fn camera_config_matches_the_readers_shape() {
        let values = BTreeMap::from([
            ("OffsetY".to_owned(), 2.0),
            ("Pitch".to_owned(), 22.5),
            ("IndicatorPulseIntervalTime".to_owned(), 0.6),
        ]);
        let out = camera_config(&values);
        assert_eq!(
            &out[..8],
            &[94, 0, 13, 0, 1, 0, 0, 0],
            "the client reports switched on"
        );
        let block = &out[8..];
        // The offset vec4, ten values, a flag, three ints, six floats and an empty name.
        assert_eq!(block.len(), 16 + 40 + 1 + 12 + 24 + 4);
        assert_eq!(&block[4..8], &2_f32.to_le_bytes());
        let pitch = f32::from_le_bytes(block[0x20..0x24].try_into().unwrap());
        assert!((pitch - 0.392_699).abs() < 1e-5, "degrees in, radians out");
        assert_eq!(
            &block[0x59..0x5d],
            &0.6_f32.to_le_bytes(),
            "no padding after the flag"
        );
    }

    /// The shipped content loads; content a packet could not carry, or a match that could not
    /// end, is refused at load (the client crashes on an empty damage-state or gear list).
    #[test]
    fn bad_content_is_refused_at_load() {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let shipped: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(dir.join(CONTENT_FILE)).unwrap()).unwrap();
        let temp = std::env::temp_dir().join(format!("osfr-ship-combat-{}", std::process::id()));
        fs::create_dir_all(&temp).unwrap();
        let loads = |edit: &dyn Fn(&mut serde_json::Value)| {
            let mut content = shipped.clone();
            edit(&mut content);
            fs::write(temp.join(CONTENT_FILE), content.to_string()).unwrap();
            Content::load(&temp).is_ok()
        };
        assert!(loads(&|_| {}), "the shipped content");
        let long = "x".repeat(MAX_TEXT + 1);
        let refused: [(&str, &dyn Fn(&mut serde_json::Value)); 10] = [
            ("no damage state", &|c| {
                c["ship"]["damage_states"] = serde_json::json!([])
            }),
            ("no gear", &|c| c["ship"]["gears"] = serde_json::json!([])),
            ("a flag past StringFixed<14>", &|c| {
                c["ship"]["flag"] = "x".repeat(MAX_FIXED + 1).into()
            }),
            ("no colour", &|c| c["colours"] = serde_json::json!([])),
            ("thresholds out of order", &|c| {
                c["ship"]["damage_states"][0]["health"] = 10.into()
            }),
            ("a colour the client lacks", &|c| {
                c["colours"] = serde_json::json!(["pir_pink"])
            }),
            ("a long arena scene", &|c| {
                c["arenas"][0]["scene"] = long.clone().into()
            }),
            ("zone 0", &|c| c["arenas"][0]["zone_id"] = 0.into()),
            ("zone past i32", &|c| {
                c["arenas"][0]["zone_id"] = (1_u64 << 31).into()
            }),
            ("an endless match", &|c| {
                c["zone_config"]["PlayTime"] = 1e300.into()
            }),
        ];
        for (case, edit) in refused {
            assert!(!loads(edit), "{case}");
        }
        assert!(
            !loads(&|c| c["report_timeout_seconds"] = 3601.into()),
            "a stalled phase"
        );
        fs::remove_dir_all(&temp).ok();
    }
}
