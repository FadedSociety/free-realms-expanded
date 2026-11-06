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
mod battle;
mod captains;

use battle::{Aim, Battery, Hull, Mark, Shot};
pub use captains::{Bot, Captain};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, VecDeque},
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
pub const MARK_TREASURE_PICKUPS: u16 = 30;
pub const RESET_PROXIED_SHIP: u16 = 11;
pub const DESTROY_PROXIED_SHIP: u16 = 10;
pub const SELECT_SHIP: u16 = 21;
pub const UPDATE_SHIP_CANNONS: u16 = 19;
pub const FIRE_AT_LOCATION: u16 = 20;
pub const SPAWN_PROJECTILE: u16 = 26;
pub const DESPAWN_PROJECTILE: u16 = 27;
pub const ANNOUNCER_MESSAGE: u16 = 34;
/// (R) How long a ship stays held at a whirlpool's exit (the loader's ExitFrozenTime).
const WHIRLPOOL_HOLD_MS: u64 = 1_500;
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
    /// The ship every other one is drawn from (R).
    pub ship: Ship,
    /// The ships a base's cave offers (locale 417574-417588), and the one a match starts on.
    #[serde(default)]
    pub fleet: Vec<FleetShip>,
    #[serde(default)]
    pub start_ship: String,
    /// The flags Ship Selection offers: the masts' texture aliases.
    #[serde(default)]
    pub decals: Vec<Decal>,
    /// The ships' colours by dock slot (`COLOURS`; R order).
    pub colours: Vec<String>,
    /// CameraConfig values by the loader's field names (`CAMERA_FIELDS`); a missing key is 0.
    pub camera: BTreeMap<String, f64>,
    /// The treasure round's values (R).
    pub treasure: Treasure,
    pub gunnery: Gunnery,
    /// The AI seats, in dock order after the player's.
    #[serde(default)]
    pub captains: Vec<Captain>,
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
    /// Fleet ship `key` as a full config: `ship` with its hull, health, hold and handling.
    pub fn fleet_ship(&self, key: &str) -> Option<Ship> {
        let entry = self.fleet.iter().find(|ship| ship.key == key)?;
        let mut ship = self.ship.clone();
        let hull = &entry.hull;
        let actor = |part: &str| format!("pirship_hum_{hull}_{part}_bbe.adr");
        let mid = if hull == "lrg" {
            actor("midmast")
        } else {
            String::new()
        };
        ship.actors = [
            actor("mainbody"),
            actor("forwardmast"),
            actor("rearmast"),
            mid,
            String::new(),
        ];
        ship.key.clone_from(&entry.key);
        ship.collision = format!("pirship_hum_{hull}_mainbody_bbe.cdt");
        ship.cannons = self.gunnery.broadsides(entry);
        ship.hit_radius = entry.gun_row;
        // The damage states' thresholds are absolute health: they scale with the hull.
        for state in &mut ship.damage_states {
            state.health = state.health * entry.health / self.ship.health.max(1);
        }
        ship.health = entry.health;
        ship.hold_capacity = entry.hold_capacity;
        ship.physics.insert("MaxSpeed".to_owned(), entry.max_speed);
        ship.physics.insert("TurboSpeed".to_owned(), entry.turbo_speed);
        for gear in &mut ship.gears {
            gear.insert("MaxAngularSpeed".to_owned(), entry.max_turn);
        }
        Some(ship)
    }

    /// The ship a match starts on.
    pub fn opening_ship(&self) -> Ship {
        self.fleet_ship(&self.start_ship)
            .unwrap_or_else(|| self.ship.clone())
    }

    /// The stands as the NPC menu takes them.
    pub fn stand_menus(&self) -> crate::npc_interaction::TableMenus {
        let menu = |stand: &Stand| crate::npc_interaction::TableMenu {
            name: stand.name.clone(),
            position: stand.position,
            games: vec![(stand.arena, stand.label_id)],
            lobby: None,
        };
        self.stands.iter().map(|stand| (stand.npc, menu(stand))).collect()
    }
}

/// The treasure round (R values: the live per-zone files never shipped).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Treasure {
    /// What an X spot can roll instead of a chest when it comes back. Each spot starts on a chest.
    #[serde(default)]
    pub power_ups: Vec<PowerUp>,
    /// How long speed and rapid fire last (R).
    #[serde(default)]
    pub power_seconds: u32,
    /// The chest a pickup shows: `ss_pir_treasure_chest` 1786, whose idle is the bobbing
    /// `ss_pir_treasure_chest_loop_anim` (`Models.txt`).
    pub chest_model: u32,
    /// Its look, one of the actor's texture aliases (pir_gold, pir_silver, pir_bronze).
    pub chest_alias: String,
    /// The effect kept on it. Every effect id on this wire is a composite id
    /// (`ActorCompositeEffectDefinitions.xml`, looked up by `sub_7C4710`), not an emitter's.
    pub chest_effect: u32,
    /// Each dock slot's BankEffectId (the coloured symbol on the pad) and HomeEntranceEffectId (the
    /// cave's coloured wash).
    #[serde(default)]
    pub dock_effects: Vec<[u32; 2]>,
    /// Seconds of warm-up before it shows, and the effect meanwhile.
    pub chest_warmup: f32,
    pub chest_warmup_effect: u32,
    /// The pickup's last effect (+0x28, R: when taken).
    pub chest_end_effect: u32,
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

/// One ship in the cave's list. The card is `sub_A5B510`'s record, which
/// `PiratesShipSelection:PopulateShipSlot` shows (00a60c10); the rest overrides `Content::ship`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FleetShip {
    /// Its key: SelectShip names it.
    pub key: String,
    pub name_id: u32,
    pub size_id: u32,
    pub turning_id: u32,
    pub cannons: u32,
    pub icon: u32,
    /// The `pirship_hum_<hull>_*` actors; the large hull alone has a mid mast.
    pub hull: String,
    pub health: i32,
    pub hold_capacity: i32,
    pub max_speed: f64,
    pub turbo_speed: f64,
    /// The gears' MaxAngularSpeed (degrees a second).
    pub max_turn: f64,
    /// (R) Where its guns sit, in metres from the ship's middle: to either side, up, and how far
    /// along the hull each way the row runs.
    pub gun_beam: f32,
    pub gun_height: f32,
    pub gun_row: f32,
}

/// The guns' shared values (R; the live `.Cannon.txt` files never shipped).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Gunnery {
    /// ProjectileSpeed, along the water. The client's aim reads a shot's landing as the ship's
    /// position plus twice (its speed and this, 00ca7480), and TargetMaxDistance's default is 70,
    /// so 35 metres a second for two seconds.
    pub speed: f32,
    pub flight_seconds: f32,
    /// How long a side takes to reload: one crew loads its guns in turn, so the bar fills.
    pub reload_seconds: f32,
    /// SpawnProjectile's model scale.
    pub ball_scale: f32,
    /// What a ball takes off a hull.
    pub damage: i32,
}

impl Gunnery {
    /// A ship's guns (`entry.cannons`, half to a side), each side's row from bow to stern.
    fn broadsides(&self, entry: &FleetShip) -> Vec<Cannon> {
        let per_side = entry.cannons / 2;
        let gun = |slot: u32, left: bool| {
            let along = (f64::from(slot) + 0.5) / f64::from(per_side);
            Cannon {
                name: format!("Cannon{}", slot * 2 + u32::from(!left)),
                left,
                // Left is the client's +x (00cae5a0 mirrors a right gun's offset).
                offset: [
                    if left { entry.gun_beam } else { -entry.gun_beam },
                    entry.gun_height,
                    entry.gun_row * (1.0 - 2.0 * along as f32),
                ],
                bit: 1 << slot,
                speed: self.speed,
                reload_seconds: self.reload_seconds,
            }
        };
        (0..per_side)
            .flat_map(|slot| [gun(slot, true), gun(slot, false)])
            .collect()
    }
}

/// One gun (`ShipCombatCannonConfig`, loader 01069df0).
#[derive(Clone, Debug, PartialEq)]
pub struct Cannon {
    /// SpawnProjectile names it, and the muzzle flash plays at it (`sub_CAEC40`).
    pub name: String,
    pub left: bool,
    /// LocationOffset, in the ship's frame.
    pub offset: [f32; 3],
    /// Its bit in UpdateShipCannons' mask for its side.
    pub bit: u32,
    pub speed: f32,
    pub reload_seconds: f32,
}

/// One power-up an X spot can roll: its client model and effect, and its chance in 100 (R; the live
/// `PickupSpawnChanceLoad` table never shipped). The firepower and long shot models are missing
/// from the community files, so those two wait.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PowerUp {
    pub kind: PowerKind,
    pub model: u32,
    pub effect: u32,
    pub chance: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PowerKind {
    Repair,
    Speed,
    RapidFire,
}

/// One flag: the masts' texture alias, its window icon (image set `icon_ui_pir_flag_*`) and its
/// standings icon (`icon_hud_pir_flag_*`).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Decal {
    pub alias: String,
    pub icon: u32,
    pub hud_icon: u32,
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
    /// Its key in the cave's list; the client finds the ship it is on by this (00a62280).
    #[serde(default)]
    pub key: String,
    /// Its guns, from `Content::fleet_ship`.
    #[serde(skip)]
    pub cannons: Vec<Cannon>,
    /// How near a ball has to pass to strike it (R: half its gun row, with `ProjectileRadius`).
    #[serde(skip)]
    pub hit_radius: f32,
    /// `SHIP_FIELDS` by name; a missing key is 0.
    #[serde(default)]
    pub config: BTreeMap<String, f64>,
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
    /// Whirlpools; each drops a ship on the far side of the map.
    #[serde(default)]
    pub whirlpools: Vec<Whirlpool>,
}

/// One whirlpool (ZoneConfig's +416 list, record `sub_A59140`; loader 01064f80's names). The client
/// draws `WhirlpoolEffectId` at the vortex and `WhirlpoolExitEffectId` at the teleport point
/// (00a602a0); the server moves the ship.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Whirlpool {
    pub vortex: [f32; 3],
    pub radius: f32,
    pub teleport: [f32; 3],
    /// TeleportAngle and TeleportSpeed, as the record carries them.
    pub angle: f32,
    pub speed: f32,
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
                && (a.whirlpools.iter()).all(|w| {
                    (w.vortex.iter().chain(&w.teleport)).all(|v| v.is_finite())
                        && w.radius > 0.0
                        && w.angle.is_finite()
                        && w.speed.is_finite()
                })
                && a.pickups.iter().all(|p| {
                    p.iter().all(|v| v.is_finite())
                        && (a.docks.iter())
                            .all(|d| distance(d.bank, *p) > t.bank_radius + t.pickup_radius)
                })
        };
        let treasure_ok = t.chest_model > 0
            && text_ok(&t.chest_alias)
            && t.chest_warmup.is_finite()
            && t.dock_chest_model > 0
            && t.value > 0
            && [t.pickup_radius, t.bank_radius, t.enter_radius]
                .iter()
                .all(|r| (1.0..=500.0).contains(r))
            && f64::from(t.respawn_seconds) <= MAX_SECONDS
            && (1.0..=1000.0).contains(&t.max_speed);
        let ship = &content.ship;
        let g = &content.gunnery;
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
            && only(&ship.config, SHIP_FIELDS)
            && f64::from(t.max_speed) >= fastest
            && ship.health >= 1
            && (content.fleet.iter()).all(|f| {
                f.health >= 1
                    && f.hold_capacity >= 1
                    && f64::from(t.max_speed) >= f.turbo_speed
                    && text_ok(&f.key)
                    && text_ok(&f.hull)
                    // Two sides, and each side's guns fit its 32-bit mask.
                    && f.cannons % 2 == 0
                    && (2..=64).contains(&f.cannons)
                    && [f.gun_beam, f.gun_height, f.gun_row].iter().all(|v| v.is_finite())
                    && f.gun_beam > 0.0
            })
            && [g.speed, g.flight_seconds, g.reload_seconds, g.ball_scale]
                .iter()
                .all(|v| (0.01..=1000.0).contains(v))
            && g.damage >= 1
            && t.power_ups.iter().map(|power| power.chance).sum::<u32>() <= 100
            && t.power_ups.iter().all(|power| power.model > 0)
            && (content.decals.iter()).all(|d| text_ok(&d.alias))
            && (content.captains.iter()).all(|c| {
                c.name_id > 0
                    && c.flag.len() <= MAX_FIXED
                    && content.fleet.iter().any(|f| f.key == c.ship)
                    && content.decals.iter().any(|d| d.alias == c.flag)
            })
            && content.colours.len() > content.captains.len()
            && content.fleet.iter().all(|f| f.max_turn > 0.0)
            && (content.fleet.is_empty()
                || content.fleet.iter().any(|f| f.key == content.start_ship))
            && (1..=1000).contains(&ship.hold_unload_per_second)
            && !ship.gears.is_empty()
            && ship.gears.iter().all(|gear| only(gear, GEAR_FIELDS));
        let play = content.zone_config.get("PlayTime").copied().unwrap_or(0.0);
        // The hold counts chests: the client divides HoldCapacity by ShipHoldMinValue (00ca10b0,
        // 00ca07c0), so 0 would crash every client in the arena and another value would mismatch.
        let hold_unit = content.zone_config.get("ShipHoldMinValue") == Some(&1.0);
        // The ball needs a model, and the client plays a splash only within this range of the
        // ship (00a5f470, `sub_CA41A0`): at 0 no shot would ever splash.
        let shots_show = [
            "ProjectileMax",
            "ProjectileModelId",
            "ProjectileWaterImpactEffectId",
            "ProjectileDespawnEffectRange",
            "Gravity",
            "WaterLevel",
        ]
        .iter()
        .all(|key| content.zone_config.get(*key).is_some_and(|value| *value > 0.0));
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
            || !shots_show
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
        let effects = treasure.dock_effects.get(slot).copied().unwrap_or_default();
        dock(&mut body, at, tint, effects, treasure);
    }
    body.extend(u32::try_from(arena.whirlpools.len()).unwrap_or(0).to_le_bytes());
    for pool in &arena.whirlpools {
        let [x, y, z] = pool.vortex;
        for value in [x, y, z, 1.0, pool.radius] {
            body.extend(value.to_le_bytes());
        }
        let [x, y, z] = pool.teleport;
        for value in [x, y, z, 1.0, pool.angle, pool.speed] {
            body.extend(value.to_le_bytes());
        }
    }
    for _ in 0..2 {
        body.extend(0_u32.to_le_bytes());
    }
    packet(ZONE_CONFIG, &body)
}

/// One record of ZoneConfig's dock list (+384): reader `sub_A5B660`, named and offset-checked
/// against the server loader `EntryPointConfigLoad` 01066750, whose ShipSpawn and AI fields stay
/// server-side. Bank at the pad, Enter at the cave; the chest and flag sit at the pad (R);
/// TreasureChestId is the dock chest, TintName the slot's colour, the effects `dock_effects`; angles
/// (radians on the wire), the flag texture and image set and the tutorial movie are 0 or empty (R).
fn dock(body: &mut Vec<u8>, at: &Dock, tint: &str, effects: [u32; 2], treasure: &Treasure) {
    let point = |body: &mut Vec<u8>, [x, y, z]: [f32; 3]| {
        for value in [x, y, z, 1.0] {
            body.extend(value.to_le_bytes());
        }
    };
    let word = |body: &mut Vec<u8>, value: u32| body.extend(value.to_le_bytes());
    point(body, at.bank); // Bank
    body.extend(treasure.bank_radius.to_le_bytes());
    word(body, effects[0]); // BankEffectId
    word(body, effects[1]); // HomeEntranceEffectId
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

/// CreateProxiedPickup (0x1c), reader `sub_A5A5F0`, handler 00a5f860: pickup `id` at `at`, showing
/// `treasure`'s chest (`sub_CAC8D0`).
pub fn create_pickup(id: u32, at: [f32; 3], treasure: &Treasure) -> Vec<u8> {
    let (model, effect) = (treasure.chest_model, treasure.chest_effect);
    pickup(id, at, (model, effect, &treasure.chest_alias), treasure)
}

/// What a spot shows: its power-up, or the chest.
fn create_spot(id: u32, at: [f32; 3], treasure: &Treasure, power: Option<&PowerUp>) -> Vec<u8> {
    match power {
        Some(power) => pickup(id, at, (power.model, power.effect, ""), treasure),
        None => create_pickup(id, at, treasure),
    }
}

fn pickup(
    id: u32,
    at: [f32; 3],
    (model, effect, alias): (u32, u32, &str),
    treasure: &Treasure,
) -> Vec<u8> {
    let mut body = id.to_le_bytes().to_vec();
    let [x, y, z] = at;
    // The position, the model's scale (`sub_CAC8D0` hands it to the actor, whose floor of 0.01 made
    // a 0 a speck) and a value the handler needs above 0 (00a5f860, R: a radius).
    for value in [x, y, z, 1.0, 1.0, 1.0] {
        body.extend(value.to_le_bytes());
    }
    body.extend(model.to_le_bytes());
    // The effect `sub_CAC4F0` keeps on the pickup once its actor loads, and an animation (R 0).
    body.extend(effect.to_le_bytes());
    body.extend(0_u32.to_le_bytes());
    // The actor's texture alias: the chest's look (`ss_pir_treasure_chest.adr`: gold, silver, bronze).
    string(&mut body, alias);
    // A warm-up before the actor shows, its effect, and the effect at +0x28 (`sub_CAC8D0`).
    body.extend(treasure.chest_warmup.to_le_bytes());
    body.extend(treasure.chest_warmup_effect.to_le_bytes());
    body.extend(treasure.chest_end_effect.to_le_bytes());
    packet(CREATE_PROXIED_PICKUP, &body)
}

/// UpdatePlayer (8) with bit 2: what the cave's window lists. `sub_A618A0` reads a count, then per
/// ship a key and `sub_A5B510`'s record: name, a value (R 0), size, turning, cannons, cargo, icon,
/// the ship's key. `sub_A61940` the same for flags: a key, then `sub_A5A370` (the standings icon,
/// the window icon, the alias).
pub fn ship_lists(guid: u64, fleet: &[FleetShip], decals: &[Decal]) -> Vec<u8> {
    let mut body = guid.to_le_bytes().to_vec();
    body.extend(4_u32.to_le_bytes());
    body.extend(u32::try_from(fleet.len()).unwrap_or(0).to_le_bytes());
    for (key, ship) in (1_u32..).zip(fleet) {
        let card = [
            key,
            ship.name_id,
            0,
            ship.size_id,
            ship.turning_id,
            ship.cannons,
            ship.hold_capacity.cast_unsigned(),
            ship.icon,
        ];
        for value in card {
            body.extend(value.to_le_bytes());
        }
        string(&mut body, &ship.key);
    }
    body.extend(u32::try_from(decals.len()).unwrap_or(0).to_le_bytes());
    for (key, decal) in (1_u32..).zip(decals) {
        for value in [key, decal.hud_icon, decal.icon] {
            body.extend(value.to_le_bytes());
        }
        string(&mut body, &decal.alias);
    }
    packet(UPDATE_PLAYER, &body)
}

/// AnnouncerMessage (34), reader `sub_A5A9B0`, handler 00a62280, type 5: opens Ship Selection.
pub fn open_ship_selection(guid: u64) -> Vec<u8> {
    announce(guid, 5, 0)
}

/// AnnouncerMessage (34): `kind` 5 opens Ship Selection; 0 is a pickup line, by `sub` (2 speed,
/// 3 rapid fire).
fn announce(guid: u64, kind: u32, sub: u32) -> Vec<u8> {
    let mut body = kind.to_le_bytes().to_vec();
    body.extend(sub.to_le_bytes());
    body.extend(guid.to_le_bytes());
    body.extend(guid.to_le_bytes());
    body.extend([0; 24]);
    packet(ANNOUNCER_MESSAGE, &body)
}

/// DestroyProxiedShip (10), handler 00a5f120: the ship `guid`.
pub fn destroy_ship(guid: u64) -> Vec<u8> {
    packet(DESTROY_PROXIED_SHIP, &guid.to_le_bytes())
}

/// The client's SelectShip (21), class 00a595b0: the ship's key and the flag's alias.
pub fn selected_ship(inner: &[u8]) -> Option<(String, String)> {
    let mut body = inner.strip_prefix(&[94, 0, 21, 0])?;
    let mut text = || -> Option<String> {
        let (len, rest) = body.split_first_chunk::<4>()?;
        let len = usize::try_from(u32::from_le_bytes(*len)).ok()?;
        let (bytes, rest) = rest.split_at_checked(len)?;
        body = rest;
        String::from_utf8(bytes.to_vec()).ok()
    };
    Some((text()?, text()?))
}

/// ResetProxiedShip (11), reader `sub_CA5CB0`, handler 00ca68f0: ship `guid`, its 4x4 transform and a
/// speed. The client holds the ship (flag 0x100) until the next UpdatePlayerState.
pub fn reset_ship(guid: u64, [x, y, z]: [f32; 3], speed: f32) -> Vec<u8> {
    let mut body = guid.to_le_bytes().to_vec();
    let rows = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, x, y, z, 1.0, speed,
    ];
    for value in rows {
        body.extend(value.to_le_bytes());
    }
    packet(RESET_PROXIED_SHIP, &body)
}

/// MarkTreasurePickups (0x1e), handler 00a5fd20: the X spots as (x, z) pairs. The client lays
/// `TreasureXMarkEffectId` on each at WaterLevel, once, when Countdown starts (00a64960, 00a60a90),
/// so this goes out before it.
pub fn mark_treasure(points: &[[f32; 3]]) -> Vec<u8> {
    // `load` bounds the list by MAX_PICKUPS.
    let mut body = (points.len() as u32).to_le_bytes().to_vec();
    for [x, _, z] in points {
        body.extend(x.to_le_bytes());
        body.extend(z.to_le_bytes());
    }
    packet(MARK_TREASURE_PICKUPS, &body)
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
/// key, then `sub_A5A370`: the flag's standings icon (`SetPlayerFlag`), its window icon and its
/// alias (the order native runs settled). Bit 4: the
/// name record (`sub_8E7290`: three values, R 0, the first and last name, which `sub_726400` joins),
/// the AI's locale name (0 for a human), the index 00a67110 checks against the list at processor
/// +0x420 (R: -1) and the row's colour (+0x1d8, `SetPlayerColor`).
pub fn player_card(guid: u64, name: &str, ship: &Ship, decal: Option<&Decal>, colour: u32) -> Vec<u8> {
    card(guid, name, (0, -1), ship, decal, colour)
}

/// A captain's row: no name record, but a locale id (+0x1d0, which `sub_A62FD0` shows), its slot
/// (+0x1d4) and the slot's colour.
fn captain_card(guid: u64, name_id: u32, slot: u32, ship: &Ship, decal: Option<&Decal>) -> Vec<u8> {
    let seat = i32::try_from(slot).unwrap_or(-1);
    card(guid, "", (name_id, seat), ship, decal, slot)
}

fn card(
    guid: u64,
    name: &str,
    (name_id, slot): (u32, i32),
    ship: &Ship,
    decal: Option<&Decal>,
    colour: u32,
) -> Vec<u8> {
    let mut body = guid.to_le_bytes().to_vec();
    body.extend(0x18_u32.to_le_bytes());
    string(&mut body, &ship.key);
    body.extend(decal.map_or(0, |decal| decal.hud_icon).to_le_bytes());
    body.extend(decal.map_or(0, |decal| decal.icon).to_le_bytes());
    string(&mut body, decal.map_or("", |decal| decal.alias.as_str()));
    body.extend([0; 12]);
    string(&mut body, name);
    string(&mut body, "");
    body.extend(name_id.to_le_bytes());
    body.extend(slot.to_le_bytes());
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
    ship_state(guid, 0, health, hold)
}

/// UpdatePlayerState with its flags word (ship +0x1c, `sub_C9E680`).
fn ship_state(guid: u64, flags: u32, health: i32, hold: u32) -> Vec<u8> {
    let mut body = guid.to_le_bytes().to_vec();
    body.extend(flags.to_le_bytes());
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

/// The client's UpdateShipPhysics (16): where the bow points and how fast the ship moves (the third
/// row and the vec4 after the position), or `None` for another packet or a non-finite value.
pub fn ship_motion(packet: &[u8]) -> Option<([f32; 3], [f32; 3])> {
    let body = packet
        .strip_prefix(&FAMILY.to_le_bytes())?
        .strip_prefix(&UPDATE_SHIP_PHYSICS.to_le_bytes())?;
    if body.len() != SHIP_PHYSICS_BODY {
        return None;
    }
    let vector = |at: usize| {
        let word = |at: usize| Some(f32::from_le_bytes(body.get(at..at + 4)?.try_into().ok()?));
        Some([word(at)?, word(at + 4)?, word(at + 8)?])
    };
    let forward = vector(8 + 1 + 8 + 32)?;
    let velocity = vector(8 + 1 + 8 + 64)?;
    (forward.iter().chain(&velocity))
        .all(|v| v.is_finite())
        .then_some((forward, velocity))
}

/// FireAtLocation (20): the side the client fires (`ShipCombatFrameUpdate` 00ca7480 sends it every
/// frame the button is down: 1 left, 2 right, 5 both). Its location stays zero (00ca2dc0).
pub fn fired_side(packet: &[u8]) -> Option<u32> {
    let body = packet
        .strip_prefix(&FAMILY.to_le_bytes())?
        .strip_prefix(&FIRE_AT_LOCATION.to_le_bytes())?;
    Some(u32::from_le_bytes(body.get(16..20)?.try_into().ok()?))
}

/// SpawnProjectile (26; reader `sub_A5A470`, handler 00a5f260): the model, its scale, the gun for
/// the muzzle flash, then a sized blob (`sub_10880C0`): the id, 2 (a ship's shot), the ship's slot,
/// the gravity, the velocity and the transform. The client flies it itself (`sub_1088070`: each
/// frame the gravity slows the climb and the velocity moves the last row) until DespawnProjectile.
fn spawn_projectile(
    id: u32,
    slot: u32,
    gun: &str,
    (model, scale): (u32, f32),
    (at, velocity, gravity): ([f32; 3], [f32; 3], f32),
) -> Vec<u8> {
    let mut blob = Vec::with_capacity(96);
    for word in [id, 2, slot] {
        blob.extend(word.to_le_bytes());
    }
    let [x, y, z] = at;
    let [vx, vy, vz] = velocity;
    let rows = [
        gravity, vx, vy, vz, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, x, y, z,
        1.0,
    ];
    for value in rows {
        blob.extend(value.to_le_bytes());
    }
    let mut body = model.to_le_bytes().to_vec();
    body.extend(scale.to_le_bytes());
    string(&mut body, gun);
    // 96 bytes: the cast is exact.
    body.extend((blob.len() as u32).to_le_bytes());
    body.extend(blob);
    packet(SPAWN_PROJECTILE, &body)
}

/// DespawnProjectile (27; reader `sub_A5A540`, handler 00a5f470): the ball goes and `effect` plays
/// at `at`, at scale 1 (over 1 the effect grows).
fn despawn_projectile(id: u32, effect: u32, at: [f32; 3]) -> Vec<u8> {
    let mut body = id.to_le_bytes().to_vec();
    body.extend(effect.to_le_bytes());
    for value in [at[0], at[1], at[2], 1.0, 1.0] {
        body.extend(value.to_le_bytes());
    }
    packet(DESPAWN_PROJECTILE, &body)
}

/// UpdateShipCannons (19; handler 00a5ee90 keeps eight words at processor +0x348 for the client's
/// own ship): the guns loaded on the right and left (the reload bars, `sub_C9ECF0`: this over half
/// the ship's guns), two more counts, then the left and right masks (00ca7480 aims only those).
fn ship_cannons(guid: u64, (left, right): (u32, u32)) -> Vec<u8> {
    let mut body = guid.to_le_bytes().to_vec();
    for word in [right.count_ones(), left.count_ones(), 0, 0, left, right, 0, 0] {
        body.extend(word.to_le_bytes());
    }
    packet(UPDATE_SHIP_CANNONS, &body)
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
    /// A whirlpool or a new ship holds it until then; its reports count for nothing.
    held_until: Option<u64>,
    /// The last report the speed bound let through: where the ship really is.
    position: Option<[f32; 3]>,
    /// That report's bow, flat on the water and of length 1, and velocity.
    motion: Option<([f32; 3], [f32; 3])>,
    /// The player's guid, for the packets `gunnery` writes.
    guid: u64,
    battery: Battery,
    hull: Hull,
    /// The balls in the air.
    shots: VecDeque<Shot>,
    next_shot: u32,
    /// The left and right masks the client last heard.
    told: Option<(u32, u32)>,
    /// The AI ships at sea with the player.
    crew: Vec<Bot>,
    /// What each spot holds: a power-up by its place in `Treasure::power_ups`, or a chest.
    spots: Vec<Option<usize>>,
    rolls: u64,
    speed_until: u64,
    rapid_until: u64,
    /// The power flags the client last heard.
    flags_told: u32,
}

/// The ship flag that gives it the Turbo speed (`sub_FEEB80` reads it as vehicle +0x2c) with its
/// effect and sound (`sub_C9DEE0`, 00ca6040).
const SPEEDING: u32 = 0x1_0000;
/// The rapid fire flag, with the next effect and sound.
const RAPID_FIRE: u32 = 0x4_0000;

/// (R) Slack on the speed bound for a report's jitter (metres), once per anchor.
/// ponytail: within one anchor window any point in the reach disc counts, so two chests on opposite
/// sides of it can both be taken; currents, whirlpools and Strong Wind raise the bound when they land.
const REPORT_SLACK: f32 = 10.0;
/// The most docks a ZoneConfig carries (the live game: six colours).
const MAX_DOCKS: usize = 6;

/// The pickups to show at the start, ids from 1.
pub fn show_pickups(points: &[[f32; 3]], treasure: &Treasure) -> Vec<Vec<u8>> {
    (points.iter().zip(1..))
        .map(|(at, id)| create_pickup(id, *at, treasure))
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
            held_until: None,
            position: None,
            motion: None,
            guid: 0,
            battery: Battery::default(),
            hull: Hull::default(),
            shots: VecDeque::new(),
            next_shot: 1,
            told: None,
            crew: Vec::new(),
            spots: vec![None; points],
            rolls: 0x9E37_79B9_7F4A_7C15,
            speed_until: 0,
            rapid_until: 0,
            flags_told: 0,
        }
    }

    /// The power flags at `now_ms`.
    fn flags(&self, now_ms: u64) -> u32 {
        let speed = if now_ms < self.speed_until { SPEEDING } else { 0 };
        speed | if now_ms < self.rapid_until { RAPID_FIRE } else { 0 }
    }

    /// The player's state as it stands.
    fn state(&mut self, guid: u64, ship: &Ship, now_ms: u64) -> Vec<u8> {
        self.flags_told = self.flags(now_ms);
        ship_state(guid, self.flags_told, self.hull.health(ship), self.hold)
    }

    /// What a spot rolls as it comes back.
    fn roll(&mut self, treasure: &Treasure) -> Option<usize> {
        self.rolls = (self.rolls)
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let mut pick = u32::try_from((self.rolls >> 33) % 100).unwrap_or(0);
        for (index, power) in treasure.power_ups.iter().enumerate() {
            if pick < power.chance {
                return Some(index);
            }
            pick -= power.chance;
        }
        None
    }

    /// Takes the power-up: the repair lands at once, the others run for `power_seconds`.
    fn power(&mut self, guid: u64, kind: PowerKind, now_ms: u64, seconds: u32) -> Option<Vec<u8>> {
        let until = now_ms.saturating_add(u64::from(seconds) * 1000);
        match kind {
            PowerKind::Repair => {
                self.hull.damage = 0;
                None
            }
            PowerKind::Speed => {
                self.speed_until = until;
                Some(announce(guid, 0, 2))
            }
            PowerKind::RapidFire => {
                self.rapid_until = until;
                Some(announce(guid, 0, 3))
            }
        }
    }

    /// Puts `crew` to sea in this round.
    pub fn man(&mut self, crew: Vec<Bot>) {
        self.crew = crew;
    }

    /// Where the player stands: first, unless a captain has banked more.
    pub fn placement(&self) -> u8 {
        let ahead = self.crew.iter().filter(|bot| bot.gold() > self.gold).count();
        u8::try_from(ahead).unwrap_or(u8::MAX - 1) + 1
    }

    /// What lets the crew's ships go, as Playing starts.
    pub fn launch_crew(&self) -> Vec<Vec<u8>> {
        self.crew.iter().map(Bot::launch).collect()
    }

    /// One tick of the crew, once the round has begun: they share the player's chests, and any
    /// that has the player abeam and in reach fires.
    pub fn sail_crew(&mut self, now_ms: u64, place: &Arena, content: &Content) -> Vec<Vec<u8>> {
        if self.anchor.is_none() {
            return Vec::new();
        }
        let target = self.position.filter(|_| self.held_until.is_none());
        let mut out = Vec::new();
        // Captains want chests: a power-up looks taken to them.
        let spots = &self.spots;
        let mut chests: Vec<_> = (self.taken_until.iter().zip(spots))
            .map(|(until, spot)| spot.map_or(*until, |_| Some(u64::MAX)))
            .collect();
        for bot in &mut self.crew {
            let sink = battle::sinking_ms(content).1;
            out.extend(bot.sail(now_ms, place, &content.treasure, &mut chests, sink));
            if let Some(target) = target {
                let air = (&mut self.shots, &mut self.next_shot);
                out.extend(bot.engage(target, now_ms, content, air));
            }
        }
        for ((until, seen), spot) in self.taken_until.iter_mut().zip(chests).zip(spots) {
            if spot.is_none() {
                *until = seen;
            }
        }
        out
    }

    /// The hold, as UpdatePlayerState carries it.
    pub fn hold(&self) -> u32 {
        self.hold
    }

    /// The last position the speed bound accepted.
    pub fn position(&self) -> Option<[f32; 3]> {
        self.position
    }

    /// A new ship from the cave: it starts on `home`, held for the whirlpool's beat so the client
    /// has built its physics before `release` (a release ahead of that would not hold), and it
    /// keeps what its hold can carry. The speed bound counts from here.
    pub fn restart(&mut self, ship: &Ship, home: [f32; 3], now_ms: u64) {
        self.anchor = Some((home, now_ms));
        self.position = Some(home);
        self.held_until = Some(now_ms.saturating_add(WHIRLPOOL_HOLD_MS));
        self.hold = self.hold.min(u32::try_from(ship.hold_capacity).unwrap_or(0));
        self.battery = Battery::default();
        self.hull = Hull::default();
        self.told = None;
    }

    /// Whether the ship is going down.
    pub fn sunk(&self) -> bool {
        self.hull.sunk
    }

    /// Keeps the heading and speed of the report `sail` just took at `at`; one it turned down
    /// aims nothing. The speed is held to `max_speed`, as the position was.
    pub fn steer(&mut self, at: [f32; 3], motion: ([f32; 3], [f32; 3]), max_speed: f32) {
        let ([x, _, z], [vx, _, vz]) = motion;
        let length = x.hypot(z);
        if self.position != Some(at) || length < 0.01 {
            return;
        }
        let speed = vx.hypot(vz);
        let slow = if speed > max_speed { max_speed / speed } else { 1.0 };
        self.motion = Some(([x / length, 0.0, z / length], [vx * slow, 0.0, vz * slow]));
    }

    /// `side` fires (`Battery::fire`), unless the ship is held. The client empties its bars on every
    /// press (00ca7480), so a press that leaves guns loaded makes `gunnery` tell it again.
    pub fn fire(&mut self, side: u32, now_ms: u64, ship: &Ship, content: &Content) -> Vec<Vec<u8>> {
        let (Some(at), Some((forward, velocity))) = (self.position, self.motion) else {
            self.told = None;
            return Vec::new();
        };
        if !matches!(side, 1 | 2 | 5) {
            return Vec::new();
        }
        if self.held_until.is_some() {
            self.told = None;
            return Vec::new();
        }
        let reload = if now_ms < self.rapid_until { 0.5 } else { 1.0 };
        let aim = Aim { owner: battle::PLAYER, slot: 0, at, forward, velocity, reload };
        let marks: Vec<Mark> = (self.crew.iter())
            .filter(|bot| bot.exposed(now_ms))
            .map(Bot::mark)
            .collect();
        let air = (&mut self.shots, &mut self.next_shot);
        let out = self.battery.fire(side, now_ms, ship, content, (&aim, &marks), air);
        let most = content.zone_config.get("ProjectileMax").copied().unwrap_or(0.0) as usize;
        if self.shots.len() >= most {
            self.told = None;
        }
        out
    }

    /// The balls that struck a ship or came down by `now_ms`, and the guns' state whenever it
    /// changed.
    pub fn gunnery(&mut self, guid: u64, now_ms: u64, ship: &Ship, content: &Content) -> Vec<Vec<u8>> {
        self.guid = guid;
        let mut out = self.strikes(now_ms, ship, content);
        let splash = content.zone_config.get("ProjectileWaterImpactEffectId");
        let splash = splash.copied().unwrap_or(0.0) as u32;
        self.shots.retain(|shot| {
            let flying = shot.lands_ms > now_ms;
            if !flying {
                out.push(despawn_projectile(shot.id, splash, shot.at));
            }
            flying
        });
        // A held ship hears its flags at `release`: a state now would clear the hold or the sinking.
        if self.held_until.is_none() && self.flags(now_ms) != self.flags_told {
            out.push(self.state(guid, ship, now_ms));
        }
        let loaded = self.battery.loaded(ship, now_ms);
        if self.told != Some(loaded) {
            self.told = Some(loaded);
            out.push(ship_cannons(guid, loaded));
        }
        out
    }

    /// Each ball in the air against every ship but its owner's. A hit ends the ball at the hull.
    fn strikes(&mut self, now_ms: u64, ship: &Ship, content: &Content) -> Vec<Vec<u8>> {
        let radius = content.zone_config.get("ProjectileRadius").copied().unwrap_or(0.0) as f32;
        let impact = ship.config.get("ProjectileImpactEffectId").copied().unwrap_or(0.0) as u32;
        let damage = content.gunnery.damage;
        let sink = battle::sinking_ms(content).0;
        let mut out = Vec::new();
        let mut shots = std::mem::take(&mut self.shots);
        shots.retain_mut(|shot| {
            let (from, to) = shot.flown(now_ms);
            let mine = (shot.owner != battle::PLAYER && self.held_until.is_none())
                .then_some(self.position)
                .flatten()
                .filter(|at| now_ms >= self.hull.safe_until
                    && battle::passes(from, to, *at, ship.hit_radius + radius));
            if let Some(at) = mine {
                out.push(despawn_projectile(shot.id, impact, to));
                let forward = self.motion.map_or([0.0, 0.0, 1.0], |(forward, _)| forward);
                out.push(battle::ship_hit(self.guid, damage, battle::side_of(at, forward, to)));
                out.push(self.take_hit(ship, damage, now_ms, sink));
                return false;
            }
            let target = (self.crew.iter_mut()).find(|bot| {
                bot.guid != shot.owner && bot.exposed(now_ms)
                    && battle::passes(from, to, bot.at, bot.ship.hit_radius + radius)
            });
            if let Some(bot) = target {
                out.push(despawn_projectile(shot.id, impact, to));
                out.extend(bot.strike(damage, to, now_ms, sink));
                return false;
            }
            true
        });
        self.shots = shots;
        out
    }

    /// A ball strikes the player: the health shows, and at none the ship goes down with its hold
    /// and stays held until `refloat`.
    fn take_hit(&mut self, ship: &Ship, damage: i32, now_ms: u64, sink_ms: u64) -> Vec<u8> {
        if self.hull.strike(ship, damage) {
            self.hold = 0;
            self.speed_until = 0;
            self.rapid_until = 0;
            self.flags_told = 0;
            self.held_until = Some(now_ms.saturating_add(sink_ms));
            return ship_state(self.guid, battle::SINKING, 0, 0);
        }
        self.state(self.guid, ship, now_ms)
    }

    /// A sunk ship's time is up: whole again on its pad, held for the whirlpool's beat like any
    /// other ship put somewhere, with the grace after it.
    pub fn refloat(
        &mut self,
        guid: u64,
        now_ms: u64,
        place: &Arena,
        content: &Content,
    ) -> Option<Vec<u8>> {
        if !self.hull.sunk || self.held_until.is_some_and(|until| now_ms < until) {
            return None;
        }
        let home = place.docks.first()?.bank;
        let grace = WHIRLPOOL_HOLD_MS + battle::sinking_ms(content).1;
        self.hull.repair(now_ms.saturating_add(grace));
        self.anchor = Some((home, now_ms));
        self.position = Some(home);
        self.held_until = Some(now_ms.saturating_add(WHIRLPOOL_HOLD_MS));
        Some(reset_ship(guid, home, 0.0))
    }

    /// The whirlpool's hold ending: the state that lets the ship go, as Playing's did.
    pub fn release(&mut self, guid: u64, now_ms: u64, ship: &Ship) -> Option<Vec<u8>> {
        let until = self.held_until?;
        if now_ms < until {
            return None;
        }
        self.held_until = None;
        self.anchor = self.anchor.map(|(at, _)| (at, now_ms));
        Some(self.state(guid, ship, now_ms))
    }

    /// The round starts at `start_ms` (when Playing began) with the ship on its own pad (dock slot
    /// 0, where every ship starts): the speed bound counts from there. Once; true that time.
    pub fn begin(&mut self, place: &Arena, start_ms: u64) -> bool {
        let first = self.anchor.is_none();
        if first {
            self.anchor = place.docks.first().map(|dock| (dock.bank, start_ms));
            self.rolls ^= start_ms;
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
        if self.held_until.is_some() {
            return Vec::new();
        }
        // Elapsed time in seconds; `as` is exact well past any match length.
        let seconds = now_ms.saturating_sub(since) as f32 / 1000.0;
        if treasure.max_speed * seconds + REPORT_SLACK < distance(from, at) {
            return Vec::new();
        }
        self.position = Some(at);
        let mut pools = place.whirlpools.iter();
        if let Some(pool) = pools.find(|p| distance(p.vortex, at) <= p.radius) {
            self.anchor = Some((pool.teleport, now_ms));
            self.held_until = Some(now_ms.saturating_add(WHIRLPOOL_HOLD_MS));
            return vec![reset_ship(guid, pool.teleport, pool.speed)];
        }
        let home = home.bank;
        let capacity = u32::try_from(ship.hold_capacity).unwrap_or(0);
        let unload_ms = 1000 / u64::try_from(ship.hold_unload_per_second.max(1)).unwrap_or(1);
        let held = self.hold;
        if (self.anchor).is_none_or(|(_, since)| now_ms.saturating_sub(since) >= 1000) {
            self.anchor = Some((at, now_ms));
        }
        let mut out = Vec::new();
        let mut powered = false;
        for (index, point) in place.pickups.iter().enumerate() {
            let spot = self.spots.get(index).copied().flatten();
            let power = spot.and_then(|at| treasure.power_ups.get(at));
            if self.taken_until.get(index).is_none_or(Option::is_some)
                || (power.is_none() && self.hold >= capacity)
                || distance(*point, at) > treasure.pickup_radius
            {
                continue;
            }
            let back = now_ms.saturating_add(u64::from(treasure.respawn_seconds) * 1000);
            self.taken_until[index] = Some(back);
            let id = u32::try_from(index + 1).unwrap_or(u32::MAX);
            out.push(destroy_pickup(id, true));
            match power {
                Some(power) => {
                    out.extend(self.power(guid, power.kind, now_ms, treasure.power_seconds));
                    powered = true;
                }
                None => self.hold += 1,
            }
        }
        let due = (self.unloaded_ms).is_none_or(|last| now_ms.saturating_sub(last) >= unload_ms);
        if self.hold > 0 && due && distance(home, at) <= treasure.bank_radius {
            self.hold -= 1;
            self.unloaded_ms = Some(now_ms);
            self.gold = self.gold.saturating_add(treasure.value);
            out.extend([player_gold(guid, self.gold), refresh_ui()]);
        }
        // The home pad repairs (HomeHealingPerSecond outruns any hull).
        let healed = self.hull.damage > 0 && distance(home, at) <= treasure.bank_radius;
        if healed {
            self.hull.damage = 0;
        }
        if self.hold != held || healed || powered {
            out.push(self.state(guid, ship, now_ms));
        }
        out
    }

    /// Spots due back at `now_ms`: each rolls what it holds.
    pub fn respawn(
        &mut self,
        now_ms: u64,
        points: &[[f32; 3]],
        treasure: &Treasure,
    ) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        for (index, point) in points.iter().enumerate() {
            if self.taken_until.get(index).copied().flatten().is_some_and(|at| at <= now_ms) {
                self.taken_until[index] = None;
                let spot = self.roll(treasure);
                self.spots[index] = spot;
                let id = u32::try_from(index + 1).unwrap_or(u32::MAX);
                let power = spot.and_then(|at| treasure.power_ups.get(at));
                out.push(create_spot(id, *point, treasure, power));
            }
        }
        out
    }
}

/// The horizontal distance between two points (the water is flat; heights differ by the hull).
/// Whether `at` lies within `radius` of `point`.
pub fn near(point: [f32; 3], at: [f32; 3], radius: f32) -> bool {
    distance(point, at) <= radius
}

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

/// The ship config's words after HoldUnloadPerSecond, in the reader's order (`sub_A683D0`), named by
/// the server loader 0106b940. +0xa8 has no loader key.
const SHIP_FIELDS: &[(&str, Field)] = &[
    ("HomeHealingPerSecond", Field::I32),
    ("PainZonePerSecond", Field::I32),
    ("LongShotAngle", Field::F32),
    ("LongShotSpeed", Field::F32),
    ("Unnamed168", Field::F32),
    ("ProjectileImpactEffectId", Field::I32),
    ("ShipImpactEffectId", Field::I32),
    ("FrozenEffectId", Field::I32),
    ("DeathCloudEffectId", Field::I32),
    ("DeathFireEffectId", Field::I32),
    ("LeftWakeEffectId", Field::I32),
    ("RightWakeEffectId", Field::I32),
    ("RestingWakeEffectId", Field::I32),
    ("WakeOffsetX", Field::F32),
    ("WakeOffsetZ", Field::F32),
    ("ProjectileLightEffectId", Field::I32),
    ("ProjectileLightOffsetX", Field::F32),
    ("ProjectileLightOffsetY", Field::F32),
    ("ProjectileLightOffsetZ", Field::F32),
    ("ReloadBarScale", Field::F32),
    ("HudYOffset", Field::F32),
    ("SpecialMoveCooldown", Field::F32),
    ("SpecialMovePushVel", Field::F32),
    ("HackedHullHeading", Field::F32),
];

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
    write_fields(&mut b, SHIP_FIELDS, &ship.config);
    b.push(0);
    zeros(&mut b, 4);
    b.extend(count(ship.cannons.len()));
    for gun in &ship.cannons {
        // Facing (1 left, 2 right), Type (2 `ship`), ReloadTime, RapidReloadTime, ProjectileSpeed.
        b.extend(if gun.left { 1_u32 } else { 2 }.to_le_bytes());
        b.extend(2_u32.to_le_bytes());
        for value in [gun.reload_seconds, gun.reload_seconds, gun.speed] {
            b.extend(value.to_le_bytes());
        }
        // LaunchAngle (0: straight out of the side), then the lob and auto-fire values (R).
        zeros(&mut b, 8);
        for value in [gun.offset[0], gun.offset[1], gun.offset[2], 0.0] {
            b.extend(value.to_le_bytes());
        }
        // Damage (no ship to hit yet), then the gun's bit (00ca7480 tests it against the mask).
        zeros(&mut b, 1);
        b.extend(gun.bit.to_le_bytes());
        string(&mut b, &gun.name);
    }
    b.extend(count(ship.damage_states.len()));
    for state in &ship.damage_states {
        b.extend(state.health.to_le_bytes());
        string(&mut b, &state.mast_alias);
        b.extend(state.effect_id.to_le_bytes());
    }
    string(&mut b, &ship.key);
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
            whirlpools: Vec::new(),
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

    /// A ship in a vortex goes to the exit, held; its reports count for nothing until the hold
    /// ends, and the speed bound then counts from the exit.
    #[test]
    fn a_whirlpool_moves_the_ship_and_lets_it_go() {
        let content = content();
        let (treasure, ship) = (&content.treasure, &content.ship);
        let place = Arena {
            whirlpools: vec![Whirlpool {
                vortex: [100.0, 53.5, 0.0],
                radius: 18.0,
                teleport: [900.0, 52.0, 0.0],
                angle: 0.0,
                speed: 10.0,
            }],
            ..content.arenas[0].clone()
        };
        let mut round = Plunder::new(place.pickups.len());
        round.anchor = Some(([95.0, 52.0, 0.0], 1_000));
        let moved = round.sail(7, [100.0, 52.0, 0.0], 2_000, &place, treasure, ship);
        assert_eq!(moved, [reset_ship(7, [900.0, 52.0, 0.0], 10.0)]);
        assert_eq!(moved[0].len(), 4 + 8 + 64 + 4);
        assert_eq!(moved[0][..4], [94, 0, 11, 0]);
        let held = round.sail(7, [100.0, 52.0, 0.0], 2_100, &place, treasure, ship);
        assert!(held.is_empty(), "held");
        assert_eq!(round.release(7, 2_100, ship), None);
        let free = round.release(7, 2_000 + WHIRLPOOL_HOLD_MS, ship);
        assert_eq!(free, Some(player_state(7, ship.health, 0)));
        assert_eq!(round.release(7, 9_000, ship), None, "once");
        assert_eq!(round.anchor, Some(([900.0, 52.0, 0.0], 2_000 + WHIRLPOOL_HOLD_MS)));
    }

    /// Each fleet ship keeps the base physics but its own hull, health, hold and handling; the
    /// damage states scale so a whole ship shows none.
    #[test]
    fn fleet_ships_differ() {
        let content = content();
        assert_eq!(content.opening_ship().key, "Cutlass");
        let eel = content.fleet_ship("Eel").unwrap();
        let big = content.fleet_ship("Juggernaut").unwrap();
        assert_eq!(eel.actors[0], "pirship_hum_sml_mainbody_bbe.adr");
        assert!(eel.actors[3].is_empty(), "no mid mast");
        assert_eq!(big.actors[3], "pirship_hum_lrg_midmast_bbe.adr");
        assert_eq!((eel.hold_capacity, big.hold_capacity), (4, 8));
        assert!(eel.physics["MaxSpeed"] > big.physics["MaxSpeed"]);
        assert!(eel.gears[0]["MaxAngularSpeed"] > big.gears[0]["MaxAngularSpeed"]);
        for ship in [&eel, &big] {
            assert!(ship.damage_states[0].health < ship.health, "{}", ship.key);
            assert_eq!(ship.damage_states[0].effect_id, 0, "whole: no smoke");
        }
        assert_eq!(content.fleet_ship("Goliath"), None);
    }

    /// The cave's lists as `sub_A618A0` and `sub_A61940` read them, and SelectShip's two strings.
    #[test]
    fn the_caves_packets_keep_their_layout() {
        let content = content();
        let out = ship_lists(7, &content.fleet[..1], &content.decals[..1]);
        assert_eq!(out[..4], [94, 0, 8, 0]);
        assert_eq!(out[12..16], 4_u32.to_le_bytes(), "bit 2");
        assert_eq!(out[16..20], 1_u32.to_le_bytes(), "one ship");
        let card = &content.fleet[0];
        assert_eq!(out[24..28], card.name_id.to_le_bytes());
        assert_eq!(out[44..48], card.hold_capacity.to_le_bytes());
        assert_eq!(open_ship_selection(7)[4..8], 5_u32.to_le_bytes());
        assert_eq!(open_ship_selection(7).len(), 4 + 8 + 16 + 24);
        let mut select = vec![94, 0, 21, 0];
        select.extend(3_u32.to_le_bytes());
        select.extend(b"Eel");
        select.extend(4_u32.to_le_bytes());
        select.extend(b"star");
        assert_eq!(selected_ship(&select), Some(("Eel".to_owned(), "star".to_owned())));
        assert_eq!(selected_ship(&select[..10]), None, "cut short");
        assert_eq!(selected_ship(&[94, 0, 22, 0]), None, "another packet");
    }

    /// Half a ship's guns to a side, the bits per side; the config carries them as `sub_A5BD50`
    /// reads them.
    #[test]
    fn a_ship_carries_its_broadsides() {
        let content = content();
        let ship = content.fleet_ship("Eel").unwrap();
        assert_eq!(ship.cannons.len(), 8);
        let left: Vec<_> = ship.cannons.iter().filter(|gun| gun.left).collect();
        assert_eq!(left.len(), 4);
        assert_eq!(left.iter().map(|gun| gun.bit).sum::<u32>(), 0b1111);
        assert!(left[0].offset[0] > 0.0 && ship.cannons[1].offset[0] < 0.0);
        let bare = Ship { cannons: Vec::new(), ..ship.clone() };
        let gun = 4 * 13 + 16 + 8;
        let names: usize = ship.cannons.iter().map(|gun| 4 + gun.name.len()).sum();
        let grown = create_ship(7, 0, [0.0; 3], &ship, "pir_red").len();
        let plain = create_ship(7, 0, [0.0; 3], &bare, "pir_red").len();
        assert_eq!(grown - plain, 8 * gun + names);
    }

    /// A gun fires once a reload; the side counts, the ball meets the water where it was sent, and
    /// the guns come back one by one.
    #[test]
    fn guns_fire_once_a_reload_and_the_balls_come_down() {
        let content = content();
        let ship = content.fleet_ship("Eel").unwrap();
        let treasure = &content.treasure;
        let mut round = Plunder::new(0);
        let at = [100.0, 53.6, 200.0];
        assert!(round.fire(1, 0, &ship, &content).is_empty(), "no report yet");
        round.position = Some(at);
        round.steer(at, ([0.0, 0.0, 2.0], [0.0, 3.0, 5.0]), treasure.max_speed);
        assert!(round.fire(3, 0, &ship, &content).is_empty(), "no such side");
        let volley = round.fire(1, 0, &ship, &content);
        assert_eq!(volley.len(), 4, "the left side alone");
        assert!(round.fire(1, 100, &ship, &content).is_empty(), "still loading");
        assert_eq!(round.fire(2, 100, &ship, &content).len(), 4, "the right side was loaded");
        let shot = &volley[0];
        assert_eq!(shot[..4], [94, 0, 26, 0]);
        assert_eq!(shot[4..8], 1666_u32.to_le_bytes());
        let name = u32::from_le_bytes(shot[12..16].try_into().unwrap()) as usize;
        assert_eq!(&shot[16..16 + name], b"Cannon0");
        let blob = &shot[16 + name..];
        assert_eq!(blob[..4], 96_u32.to_le_bytes());
        assert_eq!(blob.len(), 100);
        let float = |at: usize| f32::from_le_bytes(blob[at..at + 4].try_into().unwrap());
        assert_eq!(blob[4..8], 1_u32.to_le_bytes(), "the id");
        assert_eq!(blob[8..12], 2_u32.to_le_bytes(), "a ship's shot");
        assert!((float(16) - 9.81).abs() < 0.001, "gravity");
        let speed = content.gunnery.speed;
        assert!((float(20) - speed).abs() < 0.001, "out of the left, +x with the bow on +z");
        assert!((float(28) - 5.0).abs() < 0.001, "the ship's way carries it");
        let (height, climb, flight) = (float(88), float(24), content.gunnery.flight_seconds);
        let landed = height + climb * flight - 9.81 * flight * flight / 2.0;
        assert!((landed - 53.6).abs() < 0.01, "the water, at {landed}");
        let splash = 15968_u32;
        let early = round.gunnery(7, 1_900, &ship, &content);
        assert!(!early.iter().any(|p| p.starts_with(&[94, 0, 27, 0])), "still flying");
        let told = (early.iter()).find(|p| p.starts_with(&[94, 0, 19, 0])).unwrap();
        assert_eq!(told[4..12], 7_u64.to_le_bytes());
        // 1.9 s of a 3 s reload: two of the left's four are back, the right's two (0.1 s behind).
        assert_eq!(told[12..16], 2_u32.to_le_bytes(), "right");
        assert_eq!(told[16..20], 2_u32.to_le_bytes(), "left");
        assert_eq!(told[28..32], 0b0011_u32.to_le_bytes(), "left mask");
        assert!(round.gunnery(7, 1_900, &ship, &content).is_empty(), "said once");
        let down = round.gunnery(7, 2_000, &ship, &content);
        let down: Vec<_> = down.iter().filter(|p| p.starts_with(&[94, 0, 27, 0])).collect();
        assert_eq!(down.len(), 4, "the first volley");
        assert_eq!(down[0][4..8], 1_u32.to_le_bytes());
        assert_eq!(down[0][8..12], splash.to_le_bytes());
        assert_eq!(down[0].len(), 4 + 8 + 16 + 4);
        let y = f32::from_le_bytes(down[0][16..20].try_into().unwrap());
        assert!((y - 53.6).abs() < 0.001);
    }

    /// Fire held down every frame reloads no faster than a tap: a side's guns per reload, and
    /// never more balls in the air than the client's pool.
    #[test]
    fn holding_fire_does_not_beat_the_reload() {
        let content = content();
        let ship = content.fleet_ship("Juggernaut").unwrap();
        let mut round = Plunder::new(0);
        let at = [100.0, 53.6, 200.0];
        round.position = Some(at);
        round.steer(at, ([0.0, 0.0, 1.0], [0.0; 3]), 40.0);
        let (mut fired, mut peak) = (0, 0);
        for now in (0..30_000).step_by(16) {
            fired += round.fire(5, now, &ship, &content).len();
            peak = peak.max(round.shots.len());
            round.gunnery(7, now, &ship, &content);
        }
        // One volley of 24, then 24 a 3 s reload for 30 s; the pool's cap only ever lowers it.
        assert!((200..=264).contains(&fired), "{fired} balls");
        assert!(peak <= 32, "{peak} in the air");
    }

    /// The client empties its bars on any press, so a press the server refuses gets them back.
    #[test]
    fn a_refused_press_refills_the_bars() {
        let content = content();
        let ship = content.fleet_ship("Eel").unwrap();
        let mut round = Plunder::new(0);
        round.position = Some([1.0; 3]);
        round.steer([1.0; 3], ([0.0, 0.0, 1.0], [0.0; 3]), 40.0);
        assert_eq!(round.gunnery(7, 0, &ship, &content).len(), 1, "full bars");
        round.held_until = Some(1_000);
        assert!(round.fire(5, 10, &ship, &content).is_empty());
        assert_eq!(round.gunnery(7, 20, &ship, &content).len(), 1, "told again");
    }

    /// A broadside into a captain: the ball ends at the hull, the health shows, and enough of them
    /// sink it; it comes back whole on its own pad, safe for a moment.
    #[test]
    fn balls_sink_a_captain_and_it_comes_back() {
        let content = content();
        let ship = content.fleet_ship("Eel").unwrap();
        let place = content.arenas[0].clone();
        let mut round = Plunder::new(place.pickups.len());
        let mut crew = content.crew(&place);
        crew.truncate(1);
        let at = [100.0, 53.6, 200.0];
        crew[0].moor([at[0] + 30.0, at[1], at[2]]);
        let guid = crew[0].guid;
        round.man(crew);
        round.position = Some(at);
        round.steer(at, ([0.0, 0.0, 1.0], [0.0; 3]), 40.0);
        round.gunnery(7, 0, &ship, &content);
        let has = |out: &[Vec<u8>], head: [u8; 4]| out.iter().any(|p| p.starts_with(&head));
        let mut now = 0;
        let mut hits = 0;
        while !round.crew[0].down() {
            round.fire(1, now, &ship, &content);
            let out = round.gunnery(7, now, &ship, &content);
            hits += out.iter().filter(|p| p.starts_with(&[94, 0, 23, 0])).count();
            if has(&out, [94, 0, 23, 0]) {
                let hit = out.iter().find(|p| p.starts_with(&[94, 0, 23, 0])).unwrap();
                assert_eq!(hit[4..12], guid.to_le_bytes());
                assert_eq!(hit[12..16], (content.gunnery.damage as u32).to_le_bytes());
                assert!(has(&out, [94, 0, 27, 0]), "the ball ends there");
            }
            now += 50;
            assert!(now < 120_000, "sunk by now");
        }
        let health = round.crew[0].ship.health;
        assert_eq!(hits as i32, (health + content.gunnery.damage - 1) / content.gunnery.damage);
        let mut chests = vec![None; place.pickups.len()];
        let bot = &mut round.crew[0];
        let sink = battle::sinking_ms(&content);
        let quiet = bot.sail(now, &place, &content.treasure, &mut chests, sink.1);
        assert!(!has(&quiet, [94, 0, 16, 0]), "a sinking ship does not sail");
        let reset = bot.sail(now + sink.0, &place, &content.treasure, &mut chests, sink.1);
        assert!(has(&reset, [94, 0, 11, 0]), "back on its pad");
        let later = now + sink.0 + WHIRLPOOL_HOLD_MS;
        let free = bot.sail(later, &place, &content.treasure, &mut chests, sink.1);
        assert!(has(&free, [94, 0, 17, 0]), "let go");
        assert!(!bot.exposed(later), "safe for a moment");
        assert!(bot.exposed(later + sink.1), "then fair game");
    }

    /// A captain's ball sinks the player: its hold goes, it is held, and it comes back on its pad.
    #[test]
    fn a_sunk_player_loses_the_hold_and_comes_back() {
        let content = content();
        let ship = content.fleet_ship("Eel").unwrap();
        let place = content.arenas[0].clone();
        let mut round = Plunder::new(place.pickups.len());
        round.guid = 7;
        round.position = Some([100.0, 53.6, 200.0]);
        round.hold = 2;
        let state = round.take_hit(&ship, 10, 0, 9_000);
        assert_eq!(state, player_state(7, ship.health - 10, 2));
        let state = round.take_hit(&ship, ship.health, 100, 9_000);
        assert_eq!(state[12..16], battle::SINKING.to_le_bytes());
        assert_eq!(round.hold, 0);
        assert_eq!(round.refloat(7, 9_000, &place, &content), None, "still going down");
        let home = place.docks[0].bank;
        assert_eq!(round.refloat(7, 9_100, &place, &content), Some(reset_ship(7, home, 0.0)));
        assert_eq!(round.position, Some(home));
        let free = round.release(7, 9_100 + WHIRLPOOL_HOLD_MS, &ship);
        assert_eq!(free, Some(player_state(7, ship.health, 0)));
    }

    /// The guns swing to a captain in their arc and lead it; one behind the arc they leave alone.
    #[test]
    fn guns_lay_on_a_mark_and_lead_it() {
        let content = content();
        let ship = content.fleet_ship("Eel").unwrap();
        let place = content.arenas[0].clone();
        let mut round = Plunder::new(place.pickups.len());
        let mut crew = content.crew(&place);
        crew.truncate(1);
        crew[0].moor([140.0, 53.6, 200.0]);
        round.man(crew);
        let at = [100.0, 53.6, 200.0];
        round.position = Some(at);
        round.steer(at, ([0.0, 0.0, 1.0], [0.0; 3]), 40.0);
        assert_eq!(round.fire(1, 0, &ship, &content).len(), 4);
        for shot in &round.shots {
            assert!(distance(shot.at, [140.0, 53.6, 200.0]) < 1.0, "{:?}", shot.at);
            assert!(shot.lands_ms < 2_000, "a near mark, a short flight");
        }
    }

    /// A spot that rolls a power-up shows it, the player takes it whatever the hold, the flag runs
    /// its time and a captain leaves it alone.
    #[test]
    fn an_x_spot_can_roll_a_power_up() {
        let mut content = content();
        content.treasure.power_ups.truncate(0);
        content.treasure.power_ups.push(PowerUp {
            kind: PowerKind::Speed,
            model: 1675,
            effect: 16137,
            chance: 100,
        });
        content.treasure.power_seconds = 10;
        let ship = content.fleet_ship("Eel").unwrap();
        let treasure = content.treasure.clone();
        let mut place = content.arenas[0].clone();
        let home = place.docks[0].bank;
        let spot = [home[0] + 100.0, home[1], home[2]];
        place.pickups = vec![spot];
        let mut round = Plunder::new(1);
        round.begin(&place, 0);
        round.taken_until[0] = Some(100);
        let back = round.respawn(100, &place.pickups, &treasure);
        assert_eq!(back.len(), 1);
        assert_eq!(back[0][..4], [94, 0, 28, 0]);
        assert_eq!(back[0][32..36], 1675_u32.to_le_bytes(), "the model after six floats");
        round.hold = u32::try_from(ship.hold_capacity).unwrap();
        let took = round.sail(7, spot, 60_000, &place, &treasure, &ship);
        assert!(took.iter().any(|p| p.starts_with(&[94, 0, 29, 0])), "a full hold still takes it");
        assert!(took.iter().any(|p| p.starts_with(&[94, 0, 34, 0])), "the line");
        let state = took.iter().find(|p| p.starts_with(&[94, 0, 17, 0])).unwrap();
        assert_eq!(state[12..16], SPEEDING.to_le_bytes());
        assert_eq!(round.hold, u32::try_from(ship.hold_capacity).unwrap());
        let over = round.gunnery(7, 70_000, &ship, &content);
        let state = over.iter().find(|p| p.starts_with(&[94, 0, 17, 0])).expect("the time runs out");
        assert_eq!(state[12..16], 0_u32.to_le_bytes());
        // Back on the spot, a captain beside it passes it by.
        round.taken_until[0] = None;
        let mut crew = content.crew(&place);
        crew.truncate(1);
        crew[0].moor(spot);
        round.man(crew);
        round.sail_crew(80_000, &place, &content);
        assert_eq!(round.taken_until[0], None);
        // A timer ending during a hold says nothing: that state would clear the hold.
        round.speed_until = 90_100;
        round.flags_told = SPEEDING;
        round.held_until = Some(95_000);
        let held = round.gunnery(7, 91_000, &ship, &content);
        assert!(!held.iter().any(|p| p.starts_with(&[94, 0, 17, 0])));
    }

    /// A ball flying metres between ticks still strikes what lies between them.
    #[test]
    fn a_ball_cannot_step_over_a_hull() {
        let (a, b) = ([0.0, 0.0, 0.0], [20.0, 0.0, 0.0]);
        assert!(battle::passes(a, b, [10.0, 0.0, 2.0], 3.0));
        assert!(!battle::passes(a, b, [10.0, 0.0, 4.0], 3.0));
        assert!(!battle::passes(a, b, [24.0, 0.0, 0.0], 3.0));
        assert!(battle::passes(a, a, [1.0, 0.0, 0.0], 3.0), "no flight yet");
    }

    /// A report the speed bound refused aims nothing, and a forged speed cannot throw a ball far.
    #[test]
    fn the_guns_trust_only_what_the_bound_let_through() {
        let content = content();
        let ship = content.fleet_ship("Eel").unwrap();
        let mut round = Plunder::new(0);
        round.steer([1.0; 3], ([0.0, 0.0, 1.0], [0.0; 3]), 40.0);
        assert_eq!(round.motion, None, "no accepted report there");
        round.position = Some([1.0; 3]);
        round.steer([1.0; 3], ([0.0; 3], [0.0; 3]), 40.0);
        assert_eq!(round.motion, None, "no bow to read");
        round.steer([1.0; 3], ([0.0, 0.0, 1.0], [0.0, 900.0, 4000.0]), 40.0);
        assert_eq!(round.motion, Some(([0.0, 0.0, 1.0], [0.0, 0.0, 40.0])));
        assert_eq!(round.fire(5, 0, &ship, &content).len(), 8, "both sides");
        round.held_until = Some(1);
        round.battery.reloads.clear();
        assert!(round.fire(5, 0, &ship, &content).is_empty(), "a held ship holds fire");
    }

    #[test]
    fn ship_reports_carry_the_last_row() {
        let out = report(7, [880.0, 52.0, 723.0]);
        assert_eq!(out.len(), 117, "the transport's admitted size");
        assert_eq!(ship_position(&out), Some((7, [880.0, 52.0, 723.0])));
        assert_eq!(ship_position(&out[..116]), None);
        assert_eq!(ship_position(&report(7, [f32::NAN, 0.0, 0.0])), None);
        assert_eq!(ship_motion(&out), Some(([0.0, 0.0, 1.0], [0.0; 3])));
        assert_eq!(ship_motion(&out[..116]), None);
        let mut fire = vec![94, 0, 20, 0];
        fire.extend([0; 16]);
        fire.extend(5_u32.to_le_bytes());
        assert_eq!(fired_side(&fire), Some(5));
        assert_eq!(fired_side(&fire[..20]), None);
        assert_eq!(fired_side(&out), None);
        let marks = mark_treasure(&[[3.0, 50.0, 4.0]]);
        assert_eq!(marks[..8], [94, 0, 30, 0, 1, 0, 0, 0]);
        assert_eq!(marks[8..12], 3_f32.to_le_bytes());
        assert_eq!(marks[12..], 4_f32.to_le_bytes());
        assert_eq!(
            create_pickup(1, [0.0; 3], &content().treasure)[4 + 4 + 16..][..4],
            1_f32.to_le_bytes(),
            "the scale"
        );
        assert_eq!(
            create_pickup(1, [0.0; 3], &content().treasure).len(),
            4 + 4 + 16 + 8 + 4 + 8 + 4 + content().treasure.chest_alias.len() + 12
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
            [create_pickup(1, points[0], treasure)]
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
        let refused: [(&str, &dyn Fn(&mut serde_json::Value)); 24] = [
            ("power-ups past a hundred in a hundred", &|c| {
                c["treasure"]["power_ups"][0]["chance"] = 101.into()
            }),
            ("a ball that does nothing", &|c| c["gunnery"]["damage"] = 0.into()),
            ("a seat with no colour", &|c| {
                c["colours"] = serde_json::json!(["pir_red"]);
            }),
            ("a ship that cannot turn", &|c| c["fleet"][0]["max_turn"] = 0.into()),
            ("no splash effect", &|c| {
                c["zone_config"]["ProjectileWaterImpactEffectId"] = 0.into()
            }),
            ("guns on the wrong sides", &|c| c["fleet"][0]["gun_beam"] = (-1.0).into()),
            ("shots that never splash", &|c| {
                c["zone_config"]["ProjectileDespawnEffectRange"] = 0.into()
            }),
            ("an odd gun count", &|c| c["fleet"][0]["cannons"] = 7.into()),
            ("a ball that never lands", &|c| c["gunnery"]["flight_seconds"] = 0.into()),
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
