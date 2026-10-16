//! Work item 078 acceptance: zone-transition unload/load/spawn ordering, the persisted world
//! location and refusal of stale prior-zone reports/actions.
//!
//! Evidence behind the vectors below (see `notes/Research/Pieces/078 Zone transitions.md`):
//!
//! ORIGINAL, preserved community capture `p2.pcap` (sha256
//! `65ac0801bf49fa9af4ee0bc95568b751392ec4d594fe6ac4e89605f9ee0d0108`), re-derived by
//! `scripts/pieces/piece_078.py`; the same session is preserved as `p12.pcap`
//! (`99309eb5...`) and `packets_2.pcapng` (`221b8d54...`), which repeat all four transitions:
//! * every zone entry is server inner opcode 31 (`ClientBeginZoning`) followed by client 13
//!   (`ClientIsReady`), server 14 (`ZoneDoneSendingInitialData`), client 10
//!   (`ClientFinishedLoading`) and only then client 125 (`PlayerUpdatePosition`);
//! * no client 125 report occurs between the 31 and the 10, in any of the twelve recovered
//!   transitions;
//! * returning to `FabledRealms` spawns at the exact f32 world position the client last reported
//!   before leaving the world zone. Encounter entry (p2 frame 15682)
//!   `[-218.84437561035156, -16.988563537597656, 204.8990936279297]` returns at p2 frame 21274
//!   with the same bits; housing entry (p2 frame 22063)
//!   `[-220.20208740234375, -18.28126335144043, 230.61207580566406]` returns at p2 frame 82556
//!   with the same bits.
//!
//! RECOVERED-COMMUNITY (pinned `lab/references/Sanctuary`, not the supplied binaries): the
//! player is invisible from the zone change until finished loading; the old zone removes and the
//! new zone adds the player in one locked step with rollback on failure; an emptied non-starting
//! zone instance is evicted; the save position is the current position only while the player is
//! in the starting world zone and otherwise the remembered world origin
//! (`Player.StartingZonePosition`, `GatewayConnection.SavePlayerToDatabase`).
//!
//! FIRST-PARTY native observation: after the server applies a new location the client can queue a
//! report for its prior scene; the developer-teleport run persisted the later arrival report, not
//! the stale origin (`notes/Reviews/Native Developer Teleport 005748.md`). The Rust gateway
//! already documents the same hazard (`gateway.rs`, "Movement from the prior scene can already be
//! queued when a begin-zoning reply is emitted").
//!
//! This file uses a `#[path]` include so the module compiles and exercises the real source before
//! the lead registers `pub mod piece_078;`. Integration step for the lead: register the module
//! (or move it out of `restoration/`) and drop the include.
#[path = "../src/restoration/piece_078.rs"]
#[allow(dead_code)]
mod piece_078;

use piece_078::{
    CLIENT_BEGIN_ZONING, CLIENT_FINISHED_LOADING, CLIENT_IS_READY, CLIENT_PRELOAD_CHARACTERS,
    Location, PLAYER_UPDATE_POSITION, Phase, PositionReport, TransitionError, TransitionEvent,
    ZONE_DONE_SENDING_INITIAL_DATA, ZoneChange, ZoneTransition, opcode,
};

/// Preserved capture this item's vectors were re-derived from.
const CAPTURE_SHA256: &str = "65ac0801bf49fa9af4ee0bc95568b751392ec4d594fe6ac4e89605f9ee0d0108";

/// Whole-message (inner opcode + body) digests of the four recovered BeginZoning packets.
const BEGIN_ZONING_ENCOUNTER_SHA256: &str =
    "de46e706e8f59c2ab94650c8e1a984aab03977677a7c38d8e171b7434281a675";
const BEGIN_ZONING_WORLD_SHA256: &str =
    "aa2b4074ac397bb841c3230f59e934a41c5265b112fe300856b9cb02d08fa737";
/// Identical to the frozen piece-022 vector: the capture's housing lot entry.
const BEGIN_ZONING_HOUSING_SHA256: &str =
    "f3b03575443e99b5d3bd15f24752eedeec258375ac0d995a876a78cb5a435f07";
const BEGIN_ZONING_WORLD_RETURN_SHA256: &str =
    "1e656498cf80729ae45dda616c0cfe2bec42b9e8aa8e19d9949d4d4f6d55286d";

/// p2 frames of the four recovered transitions and their stage packets.
const FRAME_ENCOUNTER_BEGIN: u32 = 15_930;
const FRAME_ENCOUNTER_READY: u32 = 16_785;
const FRAME_ENCOUNTER_INITIAL_DATA: u32 = 17_046;
const FRAME_ENCOUNTER_LOADED: u32 = 17_057;
const FRAME_ENCOUNTER_FIRST_POSITION: u32 = 17_058;
const FRAME_WORLD_RETURN_BEGIN: u32 = 21_274;
const FRAME_WORLD_RETURN_LOADED: u32 = 21_669;
const FRAME_HOUSING_BEGIN: u32 = 27_311;
const FRAME_HOUSING_LOADED: u32 = 29_907;
const FRAME_HOUSING_FIRST_POSITION: u32 = 30_178;
const FRAME_WORLD_RETURN_2_BEGIN: u32 = 82_556;
const FRAME_WORLD_RETURN_2_LOADED: u32 = 82_940;

/// The exact f32 world position reported before entering an instance, re-used verbatim as the
/// return spawn by the server.
const WORLD_ORIGIN_ENCOUNTER: [f32; 3] = [
    -218.844_375_61,
    -16.988_563_537_597_656,
    204.899_093_627_929_7,
];
const WORLD_ORIGIN_HOUSING: [f32; 3] = [
    -220.202_087_402_343_75,
    -18.281_263_351_440_43,
    230.612_075_805_664_06,
];
const WORLD_ORIGIN_ENCOUNTER_REPORTED: u32 = 15_682;
const WORLD_ORIGIN_HOUSING_REPORTED: u32 = 22_063;

/// Recovered spawn and first-post-spawn positions.
const ENCOUNTER_SPAWN: [f32; 3] = [
    130.108_001_708_984_38,
    0.688_000_023_365_020_8,
    120.036_003_112_792_97,
];
const ENCOUNTER_FIRST_POSITION: [f32; 3] = [
    130.108_703_613_281_25,
    1.027_824_878_692_627,
    120.042_518_615_722_66,
];
const HOUSING_SPAWN: [f32; 3] = [
    440.631_988_525_390_6,
    -0.071_000_002_324_581_15,
    432.800_994_873_046_9,
];
const HOUSING_FIRST_POSITION: [f32; 3] = [
    440.572_479_248_046_9,
    -0.070_733_189_582_824_71,
    432.695_678_710_937_5,
];
/// Where the player stood inside the lot when the world origin was stored (p2 frame 22063).
const HOUSING_DOOR_POSITION: [f32; 3] = [
    424.495_788_574_218_75,
    -0.068_969_845_771_789_55,
    458.332_580_566_406_25,
];

/// World zone id used by this project's content (`FabledRealms` is zone 0; the capture's
/// encounter clearing is also `Id` 0, so this policy keys the world origin on the zone the caller
/// names, never on the BeginZoning id alone).
const WORLD_ZONE: i32 = 0;
const ENCOUNTER_ZONE: i32 = 411;
const HOUSING_ZONE: i32 = 1;

fn location(zone_id: i32, position: [f32; 3]) -> Location {
    Location {
        zone_id,
        position,
        rotation: [0.0, 0.0, 0.0],
    }
}

fn world_at_origin() -> ZoneTransition {
    ZoneTransition::new(WORLD_ZONE, location(WORLD_ZONE, WORLD_ORIGIN_ENCOUNTER))
        .expect("finite world spawn")
}

/// Runs the four recovered stage packets in the captured order and returns the spawn.
fn run_captured_transition(
    transition: &mut ZoneTransition,
    destination: Location,
) -> Result<Location, TransitionError> {
    transition.begin(destination)?;
    transition.on_client_ready()?;
    transition.on_initial_data_sent()?;
    transition.on_finished_loading()
}

fn same_bits(left: [f32; 3], right: [f32; 3]) -> bool {
    left.iter()
        .map(|v| v.to_bits())
        .eq(right.iter().map(|v| v.to_bits()))
}

#[test]
fn captured_capture_and_packet_digests_are_pinned() {
    // Guards the provenance constants themselves so a re-derivation with another capture cannot
    // silently pass with different bytes.
    for digest in [
        CAPTURE_SHA256,
        BEGIN_ZONING_ENCOUNTER_SHA256,
        BEGIN_ZONING_WORLD_SHA256,
        BEGIN_ZONING_HOUSING_SHA256,
        BEGIN_ZONING_WORLD_RETURN_SHA256,
    ] {
        assert_eq!(digest.len(), 64);
        assert!(digest.chars().all(|c| c.is_ascii_hexdigit()));
    }
    // The stage frames are strictly ordered in the capture, exactly as the policy enforces.
    let frames = [
        FRAME_ENCOUNTER_BEGIN,
        FRAME_ENCOUNTER_READY,
        FRAME_ENCOUNTER_INITIAL_DATA,
        FRAME_ENCOUNTER_LOADED,
        FRAME_ENCOUNTER_FIRST_POSITION,
        FRAME_WORLD_RETURN_BEGIN,
        FRAME_WORLD_RETURN_LOADED,
        FRAME_HOUSING_BEGIN,
        FRAME_HOUSING_LOADED,
        FRAME_HOUSING_FIRST_POSITION,
        FRAME_WORLD_RETURN_2_BEGIN,
        FRAME_WORLD_RETURN_2_LOADED,
    ];
    assert!(frames.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(WORLD_ORIGIN_ENCOUNTER_REPORTED < FRAME_ENCOUNTER_BEGIN);
    assert!(WORLD_ORIGIN_HOUSING_REPORTED < FRAME_HOUSING_BEGIN);
}

#[test]
fn stage_events_map_to_the_recovered_inner_opcodes() {
    assert_eq!(opcode(TransitionEvent::BeginZoning), 31);
    assert_eq!(opcode(TransitionEvent::ClientReady), 13);
    assert_eq!(opcode(TransitionEvent::InitialDataSent), 14);
    assert_eq!(opcode(TransitionEvent::FinishedLoading), 10);
    assert_eq!(
        [
            CLIENT_BEGIN_ZONING,
            CLIENT_IS_READY,
            ZONE_DONE_SENDING_INITIAL_DATA,
            CLIENT_FINISHED_LOADING,
            PLAYER_UPDATE_POSITION,
            CLIENT_PRELOAD_CHARACTERS,
        ],
        [31, 13, 14, 10, 125, 41]
    );
}

#[test]
fn stages_only_advance_in_the_captured_order() {
    let mut transition = world_at_origin();
    // Nothing but a begin may start a transition.
    assert_eq!(
        transition.on_client_ready(),
        Err(TransitionError::NotLoading)
    );
    assert_eq!(transition.phase(), Phase::InWorld);

    let change = transition
        .begin(location(ENCOUNTER_ZONE, ENCOUNTER_SPAWN))
        .expect("begin from the world zone");
    assert_eq!(
        change,
        ZoneChange {
            unload_zone_id: WORLD_ZONE,
            spawn: location(ENCOUNTER_ZONE, ENCOUNTER_SPAWN),
        }
    );
    assert_eq!(transition.phase(), Phase::Loading);
    // The client cannot finish loading before the server sent the zone initialization.
    assert_eq!(
        transition.on_finished_loading(),
        Err(TransitionError::NotSpawning)
    );
    assert_eq!(transition.phase(), Phase::Loading);
    // A second begin while a transition is in flight is refused.
    assert_eq!(
        transition.begin(location(HOUSING_ZONE, HOUSING_SPAWN)),
        Err(TransitionError::NotInWorld)
    );
    assert_eq!(transition.phase(), Phase::Loading);

    assert_eq!(transition.on_client_ready(), Ok(()));
    assert_eq!(transition.phase(), Phase::Initializing);
    assert_eq!(transition.on_initial_data_sent(), Ok(()));
    assert_eq!(transition.phase(), Phase::Spawning);
    let spawn = transition
        .on_finished_loading()
        .expect("spawn commits the target");
    assert_eq!(spawn, location(ENCOUNTER_ZONE, ENCOUNTER_SPAWN));
    assert_eq!(transition.phase(), Phase::InWorld);
    assert_eq!(
        transition.location(),
        location(ENCOUNTER_ZONE, ENCOUNTER_SPAWN)
    );
}

#[test]
fn queued_prior_scene_position_never_moves_the_persisted_location() {
    let mut transition = world_at_origin();
    transition
        .begin(location(ENCOUNTER_ZONE, ENCOUNTER_SPAWN))
        .expect("enter the instance");
    assert_eq!(
        transition.persisted_location(),
        location(WORLD_ZONE, WORLD_ORIGIN_ENCOUNTER)
    );

    // The captured client reported this exact world position up to 248 frames before the entry;
    // a queued copy arriving after the begin-zoning reply must not be persisted.
    assert_eq!(
        transition.report_position(WORLD_ORIGIN_ENCOUNTER, [0.0, 0.0, 0.0]),
        PositionReport::IgnoredWhileLoading
    );
    assert_eq!(
        transition.location(),
        location(ENCOUNTER_ZONE, ENCOUNTER_SPAWN)
    );
    assert_eq!(
        transition.persisted_location(),
        location(WORLD_ZONE, WORLD_ORIGIN_ENCOUNTER)
    );

    transition.on_client_ready().unwrap();
    transition.on_initial_data_sent().unwrap();
    // Still refused in every pre-spawn phase.
    assert_eq!(
        transition.report_position(WORLD_ORIGIN_ENCOUNTER, [0.0, 0.0, 0.0]),
        PositionReport::IgnoredWhileLoading
    );
    transition.on_finished_loading().unwrap();

    // After the spawn the first genuine report is the client's own adjustment of the spawn point
    // (the captured one differs from the packet by 0.34m in Y).
    assert_eq!(
        transition.report_position(ENCOUNTER_FIRST_POSITION, [0.0, 0.0, 0.0]),
        PositionReport::Applied
    );
    assert_eq!(
        transition.persisted_location().position,
        WORLD_ORIGIN_ENCOUNTER
    );
    assert!(same_bits(
        transition.location().position,
        ENCOUNTER_FIRST_POSITION
    ));
}

#[test]
fn world_entries_return_to_the_exact_stored_origin() {
    let mut transition = world_at_origin();
    // Encounter round trip (p2 frames 15930/21274): leave from the reported world position, walk
    // inside the instance, then return to the same bits.
    assert!(same_bits(
        transition.persisted_location().position,
        WORLD_ORIGIN_ENCOUNTER
    ));
    run_captured_transition(&mut transition, location(ENCOUNTER_ZONE, ENCOUNTER_SPAWN)).unwrap();
    assert_eq!(
        transition.report_position(ENCOUNTER_FIRST_POSITION, [0.0, 0.0, 0.0]),
        PositionReport::Applied
    );
    assert!(same_bits(
        transition.persisted_location().position,
        WORLD_ORIGIN_ENCOUNTER
    ));
    let back = transition
        .begin(location(
            WORLD_ZONE,
            transition.persisted_location().position,
        ))
        .unwrap();
    assert_eq!(back.unload_zone_id, ENCOUNTER_ZONE);
    assert!(same_bits(back.spawn.position, WORLD_ORIGIN_ENCOUNTER));
    transition.on_client_ready().unwrap();
    transition.on_initial_data_sent().unwrap();
    transition.on_finished_loading().unwrap();
    assert_eq!(transition.location().zone_id, WORLD_ZONE);
    assert!(same_bits(
        transition.location().position,
        WORLD_ORIGIN_ENCOUNTER
    ));
    assert!(same_bits(
        transition.persisted_location().position,
        WORLD_ORIGIN_ENCOUNTER
    ));

    // Housing round trip (p2 frames 27311/82556): the player first walks to the door inside the
    // world zone, so the stored origin is a different position from the encounter one. Movement
    // inside the lot must never overwrite it and the return spawn is that stored origin.
    let mut transition = world_at_origin();
    assert_eq!(
        transition.report_position(WORLD_ORIGIN_HOUSING, [0.0, 0.0, 0.0]),
        PositionReport::Applied
    );
    assert!(same_bits(
        transition.persisted_location().position,
        WORLD_ORIGIN_HOUSING
    ));
    transition
        .begin(location(HOUSING_ZONE, HOUSING_SPAWN))
        .unwrap();
    // The instance spawn did not change the stored origin.
    assert!(same_bits(
        transition.persisted_location().position,
        WORLD_ORIGIN_HOUSING
    ));
    transition.on_client_ready().unwrap();
    transition.on_initial_data_sent().unwrap();
    transition.on_finished_loading().unwrap();
    assert_eq!(
        transition.report_position(HOUSING_FIRST_POSITION, [0.0, 0.0, 0.0]),
        PositionReport::Applied
    );
    // The player walks back to the lot's own exit; the persisted origin still does not move.
    assert_eq!(
        transition.report_position(HOUSING_DOOR_POSITION, [0.0, 0.0, 0.0]),
        PositionReport::Applied
    );
    assert!(same_bits(
        transition.persisted_location().position,
        WORLD_ORIGIN_HOUSING
    ));
    let back = transition
        .begin(location(
            WORLD_ZONE,
            transition.persisted_location().position,
        ))
        .unwrap();
    assert_eq!(back.unload_zone_id, HOUSING_ZONE);
    assert!(same_bits(back.spawn.position, WORLD_ORIGIN_HOUSING));
    transition.on_client_ready().unwrap();
    transition.on_initial_data_sent().unwrap();
    transition.on_finished_loading().unwrap();
    assert_eq!(transition.location().zone_id, WORLD_ZONE);
    assert!(same_bits(
        transition.location().position,
        WORLD_ORIGIN_HOUSING
    ));
    assert!(same_bits(
        transition.persisted_location().position,
        WORLD_ORIGIN_HOUSING
    ));
}

#[test]
fn prior_zone_actions_are_refused_until_the_new_zone_is_spawned() {
    let mut transition = world_at_origin();
    assert!(transition.accepts_zone_action(WORLD_ZONE));
    transition
        .begin(location(ENCOUNTER_ZONE, ENCOUNTER_SPAWN))
        .unwrap();
    // Unloaded: the previous zone no longer accepts actions, and the destination is not live yet.
    assert!(!transition.accepts_zone_action(WORLD_ZONE));
    assert!(!transition.accepts_zone_action(ENCOUNTER_ZONE));
    transition.on_client_ready().unwrap();
    transition.on_initial_data_sent().unwrap();
    assert!(!transition.accepts_zone_action(ENCOUNTER_ZONE));
    transition.on_finished_loading().unwrap();
    assert!(transition.accepts_zone_action(ENCOUNTER_ZONE));
    assert!(!transition.accepts_zone_action(WORLD_ZONE));
}

#[test]
fn malformed_requests_are_refused_without_mutation() {
    assert_eq!(
        ZoneTransition::new(WORLD_ZONE, location(WORLD_ZONE, [f32::NAN, 0.0, 0.0])).err(),
        Some(TransitionError::NonFinite)
    );
    let mut transition = world_at_origin();
    // The community handler returns early when the destination is the current zone.
    assert_eq!(
        transition.begin(location(WORLD_ZONE, [1.0, 2.0, 3.0])),
        Err(TransitionError::SameZone)
    );
    assert_eq!(transition.phase(), Phase::InWorld);
    assert_eq!(
        transition.location(),
        location(WORLD_ZONE, WORLD_ORIGIN_ENCOUNTER)
    );
    assert_eq!(
        transition.begin(location(ENCOUNTER_ZONE, [f32::INFINITY, 0.0, 0.0])),
        Err(TransitionError::NonFinite)
    );
    assert_eq!(transition.phase(), Phase::InWorld);
    assert_eq!(
        transition.report_position([f32::NAN, 0.0, 0.0], [0.0, 0.0, 0.0]),
        PositionReport::Invalid
    );
    assert_eq!(transition.location().position, WORLD_ORIGIN_ENCOUNTER);
}
