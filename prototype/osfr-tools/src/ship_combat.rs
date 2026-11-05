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
/// The Games window's join key for Pirate's Plunder (Games row 1059, entry type 1, category 20).
/// Row 54 is another game's; a match credits its arena's `minigame_id`, TYPE 13 rows 1032-1034.
pub const GAME_ID: u32 = 54;
/// MiniGameData TYPE of the Pirates rows: the client's HUD rule hangs on it (Lua 35774, 30209).
pub const MINIGAME_TYPE: i32 = 13;
/// The rows' description (427743) and the Shipwreck Cove row's icon, the one row that has one (R).
pub const OFFER_DESCRIPTION: i32 = 427_743;
pub const OFFER_ICON: i32 = 30_434;
pub const UPDATE_GAME_STATE: u16 = 2;
pub const UPDATE_PLAYER_GAME_STATES: u16 = 3;
pub const SET_PLAYER_COUNT: u16 = 7;
pub const CREATE_PROXIED_SHIP: u16 = 9;
pub const ZONE_CONFIG: u16 = 12;
pub const CAMERA_CONFIG: u16 = 13;
pub const REFRESH_UI: u16 = 4;
pub const UPDATE_PLAYER: u16 = 8;
pub const UPDATE_SHIP_PHYSICS: u16 = 16;
pub const UPDATE_PLAYER_STATE: u16 = 17;
/// PlayerAwards: what opens the results board (`00a65780`).
pub const PLAYER_AWARDS: u16 = 24;
pub const CREATE_PROXIED_PICKUP: u16 = 28;
pub const DESTROY_PROXIED_PICKUP: u16 = 29;
/// UpdateShipPhysics' body as the transport admits it (`soe::DIRECT_SHAPES`, 117 with the header).
const SHIP_PHYSICS_BODY: usize = 113;
/// (R) Most pickup points an arena carries.
const MAX_PICKUPS: usize = 64;
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
const EXTRA_KEYS: &[&str] = &["UseShipHullTintForDockTint"];

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
    /// (R) Seconds the results board stays up if the player never closes it.
    pub results_timeout_seconds: u32,
    /// (R) What finishing a match pays; no source gives an amount.
    pub win_coins: u32,
    /// ZoneConfig values by the loader's field names (`ZONE_FIELDS`); a missing key is 0.
    pub zone_config: BTreeMap<String, f64>,
    /// The player's ship (R).
    pub ship: Ship,
    /// The ships' colours by dock slot (`COLOURS`; R order).
    pub colours: Vec<String>,
    /// CameraConfig values by the loader's field names (`CAMERA_FIELDS`); a missing key is 0.
    pub camera: BTreeMap<String, f64>,
    /// The treasure round's values (R).
    pub treasure: Treasure,
    /// The dock's book stands (`sg_signinbook_01`), the world's way in.
    #[serde(default)]
    pub stands: Vec<Stand>,
}

/// One stand: its spawn, and the arena its menu sails (R: arena 1 until the others load natively).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Stand {
    pub npc: u64,
    pub name: String,
    /// The menu row's label.
    pub label_id: u32,
    /// Where its spawn stands, for the click's range check.
    pub position: [f32; 3],
    pub arena: u32,
}

impl Content {
    /// The stands as the NPC menu takes them.
    pub fn stand_menus(&self) -> crate::npc_interaction::TableMenus {
        let menu = |stand: &Stand| crate::npc_interaction::TableMenu {
            name: stand.name.clone(),
            position: stand.position,
            games: vec![(stand.arena, stand.label_id)],
        };
        self.stands.iter().map(|stand| (stand.npc, menu(stand))).collect()
    }
}

/// The treasure round (R values: the live per-zone files never shipped).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Treasure {
    /// The chest a pickup shows (`Models.txt`: bronze 1662, silver 1664, gold 1663).
    pub chest_model: u32,
    /// The chest on a dock (`ss_pir_dock_treasure_chest_bbe` 1923), the dock list's TreasureChestId.
    pub dock_chest_model: u32,
    /// Gold for each chest banked.
    pub value: u32,
    /// How near the ship takes a chest (the loader's HoldTreasurePickupRadius, R).
    pub pickup_radius: f32,
    /// How near its own dock the ship banks the hold (the dock list's BankRadius).
    pub bank_radius: f32,
    /// How near its base's cave Ship Selection opens (the dock list's EnterRadius).
    pub enter_radius: f32,
    /// Seconds before a taken chest lies there again.
    pub respawn_seconds: u32,
    /// The fastest a reported ship may move (m/s): a report beyond it is ignored. Load keeps it at
    /// or above the ship's TurboSpeed × SpeedFactor, so an honest ship is never refused.
    pub max_speed: f32,
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
    /// HoldUnloadPerSecond (+148): chests a hold banks a second on its own pad (live guide: stay in
    /// your circle "until all of your loot is offloaded"). The server banks at this pace too.
    pub hold_unload_per_second: i32,
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
    /// The map's locale name, which its offer shows (`IZ4GPcc7vXk_136.00`).
    pub name_id: i32,
    /// (R) The zone id the transition names; one of our own.
    pub zone_id: u32,
    /// The MiniGameData row a finished match credits (TYPE 13; 1034 names Shipwreck Cove, which
    /// is `ss_pir_arena3`; the other two rows carry no name, R).
    pub minigame_id: u32,
    /// The docks by slot, coloured `colours[slot]`; slot 0 is the solo player's (red, as live).
    pub docks: Vec<Dock>,
    /// (R) The spawn rotation (x, y, z, w), as the fishing holes send it.
    pub rotation: [f32; 4],
    /// (R) Where chests lie (on the water between the docks).
    pub pickups: Vec<[f32; 3]>,
}

/// One base. Every ship, player or AI, starts at its own.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Dock {
    /// The bank pad (the arena's `Port`/`PortBell` area, its coloured ring): the ship starts here,
    /// banks its hold and heals.
    pub bank: [f32; 3],
    /// The base's cave (its `ss_pir_*shipwash*` prop): steering in opens Ship Selection. It lies
    /// apart from the pad, or a ship would start parked.
    pub enter: [f32; 3],
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
        let t = &content.treasure;
        // No chest a ship parked on a pad could take (it would take it again as it respawns), and
        // no cave over a pad.
        let apart = |d: &Dock| {
            distance(d.bank, d.enter) > t.bank_radius + t.enter_radius
                && (d.bank.iter().chain(&d.enter)).all(|v| v.is_finite())
        };
        let arena_ok = |a: &Arena| {
            !a.scene.is_empty()
                && text_ok(&a.scene)
                && !a.docks.is_empty()
                && a.docks.len() <= MAX_DOCKS
                && !a.pickups.is_empty()
                && i32::try_from(a.zone_id).is_ok_and(|id| id > 0)
                && a.pickups.len() <= MAX_PICKUPS
                && a.docks.iter().all(apart)
                && a.pickups.iter().all(|p| {
                    p.iter().all(|v| v.is_finite())
                        && (a.docks.iter())
                            .all(|d| distance(d.bank, *p) > t.bank_radius + t.pickup_radius)
                })
        };
        let treasure_ok = t.chest_model > 0
            && t.dock_chest_model > 0
            && t.value > 0
            && [t.pickup_radius, t.bank_radius, t.enter_radius]
                .iter()
                .all(|r| (1.0..=500.0).contains(r))
            && f64::from(t.respawn_seconds) <= MAX_SECONDS
            && (1.0..=1000.0).contains(&t.max_speed);
        let ship = &content.ship;
        let physics = |key: &str| ship.physics.get(key).copied().unwrap_or(0.0);
        let fastest = physics("TurboSpeed") * physics("SpeedFactor");
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
            && f64::from(t.max_speed) >= fastest
            && ship.health >= 1
            && (1..=1000).contains(&ship.hold_unload_per_second)
            && !ship.gears.is_empty()
            && ship.gears.iter().all(|gear| only(gear, GEAR_FIELDS));
        let play = content.zone_config.get("PlayTime").copied().unwrap_or(0.0);
        // The hold counts chests: the client divides HoldCapacity by ShipHoldMinValue (00ca10b0,
        // 00ca07c0), so 0 would crash every client in the arena and another value would mismatch.
        let hold_unit = content.zone_config.get("ShipHoldMinValue") == Some(&1.0);
        let timings = [
            f64::from(content.countdown_seconds),
            f64::from(content.report_timeout_seconds),
            f64::from(content.results_timeout_seconds),
            play,
        ];
        if content.schema_version != 1
            || content.arenas.is_empty()
            || !content.arenas.iter().all(arena_ok)
            || !content.zone_config.keys().all(|key| known(key))
            || !only(&content.camera, CAMERA_FIELDS)
            || !ship_ok
            || !treasure_ok
            || content.ship.hold_capacity < 1
            || !hold_unit
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
    results_ms: u64,
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
            results_ms: u64::from(content.results_timeout_seconds) * 1000,
        }
    }

    /// When Playing began, while the match is Playing.
    pub fn playing_since(&self) -> Option<u64> {
        self.since_ms.filter(|_| self.state == PLAYING)
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
                // The player's close (`00a4fb50`), or the board's own timeout.
                GAME_OVER if self.reported & 0x800 != 0 || waited >= self.results_ms => EXITING,
                _ => break,
            };
            self.state = next;
            self.since_ms = Some(now_ms);
            sent.push(next);
        }
        sent
    }

    /// The player left the match: straight to Exiting, whatever state it was in.
    pub fn quit(&mut self, now_ms: u64) {
        self.state = EXITING;
        self.since_ms = Some(now_ms);
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

/// ZoneConfig (12) for `arena`: the fields of `ZONE_FIELDS` from `values` (a missing key is 0), the
/// treasure area, the dock tint flag, the docks (+384, tinted `colours` by slot), then three empty
/// lists (+416, +432 and +448). The treasure area, where the HUD's "Treasure N m" marker points, is
/// the centre of the arena's chests (its X spots; `load` keeps at least one).
pub fn zone_config(
    arena: &Arena,
    values: &BTreeMap<String, f64>,
    colours: &[String],
    treasure: &Treasure,
) -> Vec<u8> {
    let value = |key: &str| values.get(key).copied().unwrap_or(0.0);
    let mut body = Vec::new();
    string(&mut body, &arena.scene);
    write_fields(&mut body, ZONE_FIELDS, values);
    let chests = &arena.pickups;
    // At most MAX_PICKUPS, so the count is exact as f32.
    let centre = |axis: usize| chests.iter().map(|p| p[axis]).sum::<f32>() / chests.len() as f32;
    for v in [centre(0), centre(2)] {
        body.extend(v.to_le_bytes());
    }
    body.push(u8::from(value("UseShipHullTintForDockTint") != 0.0));
    let docks = &arena.docks;
    body.extend(u32::try_from(docks.len()).unwrap_or(0).to_le_bytes());
    for (slot, at) in docks.iter().enumerate() {
        let tint = colours.get(slot).map_or("", String::as_str);
        dock(&mut body, at, tint, treasure);
    }
    for _ in 0..3 {
        body.extend(0_u32.to_le_bytes());
    }
    packet(ZONE_CONFIG, &body)
}

/// One record of ZoneConfig's dock list (+384): reader `sub_A5B660`, named and offset-checked
/// against the server loader `EntryPointConfigLoad` 01066750, whose ShipSpawn and AI fields stay
/// server-side. Bank at the pad, Enter at the cave; the chest and flag sit at the pad (R);
/// TreasureChestId is the dock chest, TintName the slot's colour; effects, angles (radians on the
/// wire), the flag texture and image set and the tutorial movie are 0 or empty (R).
fn dock(body: &mut Vec<u8>, at: &Dock, tint: &str, treasure: &Treasure) {
    let point = |body: &mut Vec<u8>, [x, y, z]: [f32; 3]| {
        for value in [x, y, z, 1.0] {
            body.extend(value.to_le_bytes());
        }
    };
    let word = |body: &mut Vec<u8>, value: u32| body.extend(value.to_le_bytes());
    point(body, at.bank); // Bank
    body.extend(treasure.bank_radius.to_le_bytes());
    word(body, 0); // BankEffectId
    word(body, 0); // HomeEntranceEffectId
    word(body, treasure.dock_chest_model); // TreasureChestId
    point(body, at.bank); // Chest
    word(body, 0); // TreasureChestAngle
    point(body, at.bank); // Flag
    word(body, 0); // FlagAngle
    body.extend(1_f32.to_le_bytes()); // FlagScale
    point(body, at.enter); // Enter
    body.extend(treasure.enter_radius.to_le_bytes()); // EnterRadius
    for _ in 0..3 {
        word(body, 0); // EnterAngle, EnterStopDistance, ShoveOutOffest
    }
    string(body, tint); // TintName
    word(body, 0); // TintColor
    string(body, ""); // FlagTextureAlias
    word(body, 0); // FlagImageSetId
    string(body, ""); // TutorialMovieFilename
}

/// CreateProxiedPickup (0x1c), reader `sub_A5A5F0`, handler 00a5f860: pickup `id` at `at` showing
/// `model`. The handler wants an id, a scale above 0 and a model; the reader's other values, the
/// string and the effects are 0 or empty (R).
pub fn create_pickup(id: u32, at: [f32; 3], model: u32) -> Vec<u8> {
    let mut body = id.to_le_bytes().to_vec();
    let [x, y, z] = at;
    // The position, the model's scale (`sub_CAC8D0` hands it to the actor, whose floor of 0.01 made
    // a 0 a speck) and a value the handler needs above 0 (00a5f860, R: a radius).
    for value in [x, y, z, 1.0, 1.0, 1.0] {
        body.extend(value.to_le_bytes());
    }
    body.extend(model.to_le_bytes());
    body.extend([0; 8]); // two values (R: an effect at +44)
    string(&mut body, "");
    body.extend([0; 12]); // three values (R: effects at +92/+96)
    packet(CREATE_PROXIED_PICKUP, &body)
}

/// DestroyProxiedPickup (0x1d), reader `sub_A5A7E0`: pickup `id`, two values (R 0), then whether it
/// was taken (the handler 00a5faa0 plays the pickup's effect) and the taker (R: 0, the one ship).
pub fn destroy_pickup(id: u32, taken: bool) -> Vec<u8> {
    let mut body = id.to_le_bytes().to_vec();
    body.extend([0; 8]);
    body.push(u8::from(taken));
    body.extend([0; 4]);
    packet(DESTROY_PROXIED_PICKUP, &body)
}

/// PlayerAwards (reader `00a65780`): a u32 count of records (20 bytes on the wire, `00a58f50`). A
/// solo match has no trophies to hand out, and trophy icons are unknown (R), so the list is empty;
/// the reader allows that.
pub fn player_awards() -> Vec<u8> {
    let mut packet = FAMILY.to_le_bytes().to_vec();
    packet.extend(PLAYER_AWARDS.to_le_bytes());
    packet.extend(0_u32.to_le_bytes());
    packet
}

/// UpdatePlayer (8), handler 00a67110: `guid`, then the record `sub_A665E0` with bit 0 only: the
/// player's gold (+0x10, which the standings sort and show, `PiratesStandings:SetPlayerGold`) and
/// two values (+0x14/+0x18, R 0).
pub fn player_gold(guid: u64, gold: u32) -> Vec<u8> {
    let mut body = guid.to_le_bytes().to_vec();
    body.extend(1_u32.to_le_bytes());
    body.extend(gold.to_le_bytes());
    body.extend([0; 8]);
    packet(UPDATE_PLAYER, &body)
}

/// UpdatePlayer (8) with bits 3 and 4: what the standings row shows. Bit 3 (`sub_A665E0`): the ship's
/// name, then `sub_A5A370`: a value, the flag icon (+0x230, `SetPlayerFlag`) and a string. Bit 4: the
/// name record (`sub_8E7290`: three values, R 0, the first and last name, which `sub_726400` joins),
/// the AI's locale name (0 for a human), the index 00a67110 checks against the list at processor
/// +0x420 (R: -1) and the row's colour (+0x1d8, `SetPlayerColor`).
pub fn player_card(guid: u64, name: &str, flag: u32, colour: u32) -> Vec<u8> {
    let mut body = guid.to_le_bytes().to_vec();
    body.extend(0x18_u32.to_le_bytes());
    string(&mut body, "");
    body.extend(0_u32.to_le_bytes());
    body.extend(flag.to_le_bytes());
    string(&mut body, "");
    body.extend([0; 12]);
    string(&mut body, name);
    string(&mut body, "");
    body.extend(0_u32.to_le_bytes());
    body.extend((-1_i32).to_le_bytes());
    body.extend(colour.to_le_bytes());
    packet(UPDATE_PLAYER, &body)
}

/// UpdatePlayerState (17), reader `sub_CA5620`, handler `ApplyUpdatePlayerStatePacket` 00ca6040: ship
/// `guid`, its state flags, health, a value, the hold and a flag. The flags (named by the debug HUD
/// 00c9f280: +I, +Ph, +Wh, +Sp, +P, +PZ; the vehicle's locked, turbo, brake, sinking, low, drift) are 0:
/// a sailing ship. The third value is only compared, never stored, so 0 fires nothing. The hold counts
/// in ZoneConfig's ShipHoldMinValue (load keeps 1: chests); the boxes over the ship come from
/// HoldCapacity / ShipHoldMinValue (00ca10b0) and fill per frame from the hold (00ca07c0).
/// ponytail: full health until damage lands; then `Plunder` carries the ship's health.
pub fn player_state(guid: u64, health: i32, hold: u32) -> Vec<u8> {
    let mut body = guid.to_le_bytes().to_vec();
    body.extend(0_u32.to_le_bytes());
    body.extend(health.to_le_bytes());
    body.extend(0_u32.to_le_bytes());
    body.extend(hold.to_le_bytes());
    body.push(0);
    packet(UPDATE_PLAYER_STATE, &body)
}

/// RefreshUI (4): the bare header; the client redraws its standings (00a655a0 → `sub_A62FD0`).
pub fn refresh_ui() -> Vec<u8> {
    packet(REFRESH_UI, &[])
}

/// The client's UpdateShipPhysics (16): its ship's guid and position (the transform's last row,
/// after the guid, a flag, a stamp and three rows), or `None` for another packet or a non-finite
/// position.
pub fn ship_position(packet: &[u8]) -> Option<(u64, [f32; 3])> {
    let body = packet
        .strip_prefix(&FAMILY.to_le_bytes())?
        .strip_prefix(&UPDATE_SHIP_PHYSICS.to_le_bytes())?;
    if body.len() != SHIP_PHYSICS_BODY {
        return None;
    }
    let word = |at: usize| Some(f32::from_le_bytes(body.get(at..at + 4)?.try_into().ok()?));
    let row = 8 + 1 + 8 + 48;
    let position = [word(row)?, word(row + 4)?, word(row + 8)?];
    let guid = u64::from_le_bytes(body.get(..8)?.try_into().ok()?);
    position
        .iter()
        .all(|v| v.is_finite())
        .then_some((guid, position))
}

/// One player's treasure round: the arena's pickup points (each lying there, or taken until a
/// time), the ship's last accepted report, the hold and the banked gold. The server owns it: the
/// client only reports where its ship is.
#[derive(Clone, Debug, PartialEq)]
pub struct Plunder {
    taken_until: Vec<Option<u64>>,
    /// Where and when the speed bound counts from; it moves at most once a second, so the slack
    /// cannot add up report by report.
    anchor: Option<([f32; 3], u64)>,
    hold: u32,
    gold: u32,
    /// When the hold last banked a chest.
    unloaded_ms: Option<u64>,
}

/// (R) Slack on the speed bound for a report's jitter (metres), once per anchor.
/// ponytail: within one anchor window any point in the reach disc counts, so two chests on opposite
/// sides of it can both be taken; currents, whirlpools and Strong Wind raise the bound when they land.
const REPORT_SLACK: f32 = 10.0;
/// The most docks a ZoneConfig carries (the live game: six colours).
const MAX_DOCKS: usize = 6;

/// The pickups to show at the start, ids from 1.
pub fn show_pickups(points: &[[f32; 3]], treasure: &Treasure) -> Vec<Vec<u8>> {
    (points.iter().zip(1..))
        .map(|(at, id)| create_pickup(id, *at, treasure.chest_model))
        .collect()
}

impl Plunder {
    /// The gold banked so far.
    pub fn gold(&self) -> u32 {
        self.gold
    }

    pub fn new(points: usize) -> Self {
        Self {
            taken_until: vec![None; points],
            anchor: None,
            hold: 0,
            gold: 0,
            unloaded_ms: None,
        }
    }

    /// The round starts at `start_ms` (when Playing began) with the ship on its own pad (dock slot
    /// 0, where every ship starts): the speed bound counts from there. Once; true that time.
    pub fn begin(&mut self, place: &Arena, start_ms: u64) -> bool {
        let first = self.anchor.is_none();
        if first {
            self.anchor = place.docks.first().map(|dock| (dock.bank, start_ms));
        }
        first
    }

    /// The ship `guid` reports `at` in `place` (its own dock is the first): before `begin`, or
    /// faster than `max_speed`, a report is ignored; otherwise chests within reach go into the hold
    /// while it has room, and on its own pad the hold banks `hold_unload_per_second` chests a
    /// second. Each change of the hold goes to the client (the boxes over the ship). The packets it
    /// earns.
    pub fn sail(
        &mut self,
        guid: u64,
        at: [f32; 3],
        now_ms: u64,
        place: &Arena,
        treasure: &Treasure,
        ship: &Ship,
    ) -> Vec<Vec<u8>> {
        let (Some((from, since)), Some(home)) = (self.anchor, place.docks.first()) else {
            return Vec::new();
        };
        // Elapsed time in seconds; `as` is exact well past any match length.
        let seconds = now_ms.saturating_sub(since) as f32 / 1000.0;
        if treasure.max_speed * seconds + REPORT_SLACK < distance(from, at) {
            return Vec::new();
        }
        let home = home.bank;
        let capacity = u32::try_from(ship.hold_capacity).unwrap_or(0);
        let unload_ms = 1000 / u64::try_from(ship.hold_unload_per_second.max(1)).unwrap_or(1);
        let held = self.hold;
        if (self.anchor).is_none_or(|(_, since)| now_ms.saturating_sub(since) >= 1000) {
            self.anchor = Some((at, now_ms));
        }
        let mut out = Vec::new();
        for ((point, until), id) in place.pickups.iter().zip(&mut self.taken_until).zip(1..) {
            if until.is_none()
                && self.hold < capacity
                && distance(*point, at) <= treasure.pickup_radius
            {
                *until = Some(now_ms.saturating_add(u64::from(treasure.respawn_seconds) * 1000));
                self.hold += 1;
                out.push(destroy_pickup(id, true));
            }
        }
        let due = (self.unloaded_ms).is_none_or(|last| now_ms.saturating_sub(last) >= unload_ms);
        if self.hold > 0 && due && distance(home, at) <= treasure.bank_radius {
            self.hold -= 1;
            self.unloaded_ms = Some(now_ms);
            self.gold = self.gold.saturating_add(treasure.value);
            out.extend([player_gold(guid, self.gold), refresh_ui()]);
        }
        if self.hold != held {
            out.push(player_state(guid, ship.health, self.hold));
        }
        out
    }

    /// Chests due back at `now_ms`.
    pub fn respawn(
        &mut self,
        now_ms: u64,
        points: &[[f32; 3]],
        treasure: &Treasure,
    ) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        for ((point, until), id) in points.iter().zip(&mut self.taken_until).zip(1..) {
            if until.is_some_and(|at| at <= now_ms) {
                *until = None;
                out.push(create_pickup(id, *point, treasure.chest_model));
            }
        }
        out
    }
}

/// The horizontal distance between two points (the water is flat; heights differ by the hull).
fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    (a[0] - b[0]).hypot(a[2] - b[2])
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
    b.extend(ship.hold_unload_per_second.to_le_bytes());
    zeros(&mut b, 9 + 7);
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
        ]);
        let treasure = content().treasure;
        let empty = Arena {
            scene: "ss_pirate_arena".to_owned(),
            docks: Vec::new(),
            pickups: vec![[800.0, 50.0, 1200.0], [900.0, 50.0, 1240.0]],
            ..content().arenas[0].clone()
        };
        let out = zone_config(&empty, &values, &[], &treasure);
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
        // The treasure area is the chests' centre.
        assert_eq!(&body[scalars..scalars + 4], &850_f32.to_le_bytes());
        assert_eq!(&body[scalars + 4..scalars + 8], &1220_f32.to_le_bytes());
        assert_eq!(
            body.len(),
            scalars + 8 + 1 + 16,
            "the treasure area, the tint and four lists"
        );
        // One dock: four points, thirteen values, three strings (the tint, two empty).
        let dock = Dock {
            bank: [880.0, 52.0, 723.0],
            enter: [813.6, 53.3, 713.3],
        };
        let red = ["pir_red".to_owned()];
        let one = Arena {
            docks: vec![dock],
            ..empty.clone()
        };
        let docked = zone_config(&one, &values, &red, &treasure);
        assert_eq!(docked.len(), out.len() + 4 * 16 + 4 * 13 + 3 * 4 + 7);
        // From the end: three empty lists, then the record after Enter (EnterRadius, three values,
        // the tint, TintColor, an empty string, FlagImageSetId, an empty string).
        let tail = 3 * 4 + 4 + 3 * 4 + (4 + 7) + 4 + 4 + 4 + 4;
        let enter = docked.len() - tail - 16;
        assert_eq!(
            &docked[enter..enter + 4],
            &813.6_f32.to_le_bytes(),
            "Enter at the cave"
        );
        assert!(
            docked.windows(7).any(|w| w == b"pir_red"),
            "the slot's tint"
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
        assert!(game.report(0x400, exit).is_empty(), "only the player's close ends the board");
        assert_eq!(game.report(0x800, exit), [EXITING]);
        assert!(!game.finished(exit + 1_999), "Exiting shows first");
        assert!(game.finished(exit + 2_000));
    }

    /// A player who walks away from the board still gets home.
    #[test]
    fn the_results_board_times_out() {
        let content = content();
        let mut game = Match::new(&content);
        game.loaded(0);
        game.report(0x2 | 0x8 | 0x10, 0);
        let countdown = u64::from(content.countdown_seconds) * 1000;
        game.advance(countdown);
        let over = countdown + (content.zone_config["PlayTime"] * 1000.0) as u64;
        assert_eq!(game.advance(over), [GAME_OVER]);
        let results = u64::from(content.results_timeout_seconds) * 1000;
        assert!(game.advance(over + results - 1).is_empty());
        assert_eq!(game.advance(over + results), [EXITING]);
    }

    #[test]
    fn the_awards_list_opens_with_its_count() {
        assert_eq!(player_awards(), [94, 0, 24, 0, 0, 0, 0, 0]);
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

    fn report(guid: u64, at: [f32; 3]) -> Vec<u8> {
        let mut body = guid.to_le_bytes().to_vec();
        body.extend([0; 9]); // the flag and the stamp
        for value in [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0] {
            body.extend(f32::to_le_bytes(value));
        }
        for value in [at[0], at[1], at[2], 1.0] {
            body.extend(value.to_le_bytes());
        }
        body.extend([0; 32]); // a vec4 and four values
        packet(UPDATE_SHIP_PHYSICS, &body)
    }

    #[test]
    fn ship_reports_carry_the_last_row() {
        let out = report(7, [880.0, 52.0, 723.0]);
        assert_eq!(out.len(), 117, "the transport's admitted size");
        assert_eq!(ship_position(&out), Some((7, [880.0, 52.0, 723.0])));
        assert_eq!(ship_position(&out[..116]), None);
        assert_eq!(ship_position(&report(7, [f32::NAN, 0.0, 0.0])), None);
        assert_eq!(
            create_pickup(1, [0.0; 3], 1662)[4 + 4 + 16..][..4],
            1_f32.to_le_bytes(),
            "the scale"
        );
        assert_eq!(
            create_pickup(1, [0.0; 3], 1662).len(),
            4 + 4 + 16 + 8 + 4 + 8 + 4 + 12
        );
        assert_eq!(destroy_pickup(1, true).len(), 4 + 4 + 8 + 1 + 4);
        // UpdatePlayerState as `sub_CA5620` reads it: guid, four u32, a bool.
        let state = player_state(7, 100, 3);
        assert_eq!(state.len(), 4 + 8 + 4 * 4 + 1);
        assert_eq!(&state[..4], &[94, 0, 17, 0]);
        assert_eq!(&state[16..20], &100_i32.to_le_bytes(), "health");
        assert_eq!(&state[24..28], &3_u32.to_le_bytes(), "the hold");
        assert_eq!(
            player_gold(7, 30)[4 + 8 + 4..4 + 8 + 8],
            30_u32.to_le_bytes()
        );
    }

    fn pad(bank: [f32; 3]) -> Dock {
        Dock {
            bank,
            enter: [bank[0], bank[1], bank[2] + 80.0],
        }
    }

    /// A chest within reach goes into the hold, which banks as gold at the dock; a report beyond
    /// the speed bound counts for nothing; a taken chest comes back after its respawn time.
    #[test]
    fn treasure_is_taken_carried_and_banked() {
        let content = content();
        let treasure = &content.treasure;
        let dock = [0.0, 50.0, 0.0];
        let points = [[100.0, 50.0, 0.0]];
        let place = Arena {
            docks: vec![pad(dock)],
            pickups: points.to_vec(),
            ..content.arenas[0].clone()
        };
        let ship = &content.ship;
        let mut round = Plunder::new(1);
        let sail =
            |round: &mut Plunder, at: [f32; 3], now| round.sail(7, at, now, &place, treasure, ship);
        assert!(
            sail(&mut round, dock, 1_000).is_empty(),
            "nothing before Playing"
        );
        round.begin(&place, 1_000);
        assert!(
            sail(&mut round, [500.0, 50.0, 0.0], 1_000).is_empty(),
            "a first report away from the pad"
        );
        assert!(sail(&mut round, dock, 1_000).is_empty());
        let hops = (100.0 / treasure.max_speed).ceil() as u64 * 1000;
        assert!(
            sail(&mut round, [100.0, 50.0, 0.0], 1_100).is_empty(),
            "too fast to be true"
        );
        let taken = sail(&mut round, [100.0, 50.0, 0.0], 1_000 + hops);
        let health = ship.health;
        assert_eq!(taken, [destroy_pickup(1, true), player_state(7, health, 1)]);
        let banked = sail(&mut round, dock, 1_000 + 2 * hops);
        let gold = player_gold(7, treasure.value);
        assert_eq!(banked, [gold, refresh_ui(), player_state(7, health, 0)]);
        let back = 1_000 + hops + u64::from(treasure.respawn_seconds) * 1000;
        assert!(round.respawn(back - 1, &points, treasure).is_empty());
        assert_eq!(
            round.respawn(back, &points, treasure),
            [create_pickup(1, points[0], treasure.chest_model)]
        );
    }

    /// The hold keeps to its capacity, banks only on the ship's own pad and a chest at a time, and
    /// the speed bound's slack does not add up report by report (Fable audit 2026-11-04).
    #[test]
    fn the_hold_banks_at_its_own_pad_one_chest_at_a_time() {
        let content = content();
        let treasure = &content.treasure;
        let (home, other) = ([0.0, 50.0, 0.0], [0.0, 50.0, 300.0]);
        let points = [[100.0, 50.0, 0.0], [105.0, 50.0, 0.0]];
        let place = Arena {
            docks: vec![pad(home), pad(other)],
            pickups: points.to_vec(),
            ..content.arenas[0].clone()
        };
        let small = |hold_capacity| Ship {
            hold_capacity,
            hold_unload_per_second: 1,
            ..content.ship.clone()
        };
        let (one, two) = (small(1), small(2));
        let mut round = Plunder::new(2);
        round.begin(&place, 0);
        let mut sail =
            |at: [f32; 3], now, ship: &Ship| round.sail(7, at, now, &place, treasure, ship);
        assert!(sail(home, 0, &one).is_empty());
        let first_chest = sail(points[0], 10_000, &one);
        let state = |hold| player_state(7, content.ship.health, hold);
        assert_eq!(
            first_chest,
            [destroy_pickup(1, true), state(1)],
            "a full hold takes no more"
        );
        let second_chest = sail(points[1], 11_000, &two);
        assert_eq!(second_chest, [destroy_pickup(2, true), state(2)]);
        assert!(
            sail(other, 20_000, &two).is_empty(),
            "another base's pad banks nothing"
        );
        let first = sail(home, 30_000, &two);
        assert_eq!(
            first,
            [player_gold(7, treasure.value), refresh_ui(), state(1)]
        );
        assert!(sail(home, 30_999, &two).is_empty(), "one chest a second");
        let second = sail(home, 31_000, &two);
        let gold = player_gold(7, 2 * treasure.value);
        assert_eq!(second, [gold, refresh_ui(), state(0)]);

        // Twenty reports 50 ms apart, each 12 m on, reach a chest 240 m out within a second: past
        // the bound (50 m), so the chest stays; a fresh slack per report would have let it go.
        let ahead = Arena {
            pickups: vec![[240.0, 50.0, 0.0]],
            ..place.clone()
        };
        let mut sly = Plunder::new(1);
        sly.begin(&ahead, 0);
        let mut out = sly.sail(7, home, 0, &ahead, treasure, &one);
        for step in 1..=20_u16 {
            let at = [12.0 * f32::from(step), 50.0, 0.0];
            out.extend(sly.sail(7, at, u64::from(step) * 50, &ahead, treasure, &one));
        }
        assert!(out.is_empty(), "the slack does not compound");

        // The anchor follows the ship: after reports on the pad at 1, 2 and 3 s, a 100 m hop half
        // a second later is past the bound (it would be in reach of the start's anchor).
        let near = Arena {
            pickups: vec![[100.0, 50.0, 0.0]],
            ..place.clone()
        };
        let mut moored = Plunder::new(1);
        moored.begin(&near, 0);
        for now in [1_000, 2_000, 3_000] {
            assert!(moored.sail(7, home, now, &near, treasure, &one).is_empty());
        }
        let hop = moored.sail(7, near.pickups[0], 3_500, &near, treasure, &one);
        assert!(hop.is_empty(), "the anchor moved with the ship");
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
        let refused: [(&str, &dyn Fn(&mut serde_json::Value)); 15] = [
            ("a hold unit other than one chest", &|c| {
                c["zone_config"]["ShipHoldMinValue"] = 0.into()
            }),
            ("an arena without chests", &|c| {
                c["arenas"][0]["pickups"] = serde_json::json!([])
            }),
            ("a speed bound under the fastest ship", &|c| {
                c["treasure"]["max_speed"] = 29.0.into()
            }),
            ("a chest by a pad", &|c| {
                c["arenas"][0]["pickups"][0] = c["arenas"][0]["docks"][1]["bank"].clone()
            }),
            ("a cave over its pad", &|c| {
                c["arenas"][0]["docks"][0]["enter"] = c["arenas"][0]["docks"][0]["bank"].clone()
            }),
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
