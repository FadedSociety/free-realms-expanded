//! W142 warpstone travel: the curated destination set, the `56`/`57` exchange,
//! position-driven attunement and the validated `90`/`122`/`58` travel
//! requests, driven with raw inner packets exactly as the gateway hands them to
//! gameplay.
//!
//! Evidence boundary:
//!
//! ORIGINAL: request layouts and opcodes (native vftable ids, captured frames),
//! the `57` framing (client handler `00b98b70` reads `u16` opcode, `i32` blob
//! length, then the flag-framed stream `00b98990`/`00b98290`), the curated rows
//! (supplied emulator data equal to the captured gateway `57`; checked by the
//! private-capture test below when the lab files exist) and the original spawn
//! of Lakeshore (9) and Lavender Coast Pass (15).
//!
//! RECONSTRUCTED (R), every refusal below: attunement by standing near a
//! warpstone, the cooldowns, the open-world-only rule, the block while knocked
//! out / in an encounter / busy, the safe-teleport target and the friend
//! teleport arrival. No captured frame shows the original server refusing a
//! request, so none of these is claimed as recovered behaviour.
//!
//! The module is not registered in `src/lib.rs`; `#[path]` keeps this target
//! compiling against the real artifact (same pattern as `tests/piece_077.rs`).
#[path = "../src/restoration/piece_077.rs"]
#[allow(dead_code)]
mod piece_077;
// Piece 073's independent reader of the same wire shape.
#[path = "../src/restoration/piece_073.rs"]
#[allow(dead_code)]
mod piece_073;
// One JSON reader for the content gates (`tests/support/json.rs`).
#[path = "support/json.rs"]
mod json;

use osfr_tools::{startup_world, world::NpcVisibility};
use piece_077::{
    Blocker, FriendView, OPEN_WORLD_ZONE, TravelContext, WarpRegistry, WarpRejection, WarpRules,
    WarpState, WarpWorld, WorldTeleportRequest, ZoneTeleportRequest, curated_definitions,
    decode_point_of_interest_message,
};
use std::cell::RefCell;

const SANCTUARY: i32 = 5;
const LAKESHORE: i32 = 9;
const SANCTUARY_POSITION: [f32; 3] = [-1414.636, -27.631, 351.567];
const LAKESHORE_POSITION: [f32; 3] = [-733.952, 9.09, 945.973];

fn ctx_at(position: [f32; 3]) -> TravelContext {
    TravelContext {
        guid: 77,
        zone_id: OPEN_WORLD_ZONE,
        position,
        rotation: [0.0, 0.0, -1.0],
        blocked: None,
    }
}

/// Flat terrain at `height`, no friends.
fn flat_world(height: f32) -> WarpWorld<'static> {
    WarpWorld {
        ground: Box::leak(Box::new(move |_, _| Some(height))),
        friend: &|_| None,
    }
}

fn blind_world() -> WarpWorld<'static> {
    WarpWorld {
        ground: &|_, _| None,
        friend: &|_| None,
    }
}

fn inner(opcode: u16, body: &[u8]) -> Vec<u8> {
    let mut packet = opcode.to_le_bytes().to_vec();
    packet.extend_from_slice(body);
    packet
}

fn travel(id: i32) -> Vec<u8> {
    inner(90, &ZoneTeleportRequest { destination_id: id }.encode())
}

/// A curated registry whose character already stood at Sanctuary.
fn at_sanctuary() -> WarpRegistry {
    let mut registry = WarpRegistry::curated();
    let reply = registry.visit(&ctx_at(SANCTUARY_POSITION));
    assert!(reply.changed);
    registry
}

fn listed_ids(message: &[u8]) -> Vec<i32> {
    decode_point_of_interest_message(message)
        .expect("a well-formed 57")
        .iter()
        .map(|row| row.id)
        .collect()
}

#[test]
fn curated_set_is_small_unique_and_warpstone_shaped() {
    let rows = curated_definitions();
    assert_eq!(rows.len(), 9);
    let mut ids: Vec<i32> = rows.iter().map(|row| row.id).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids, [3, 4, 5, 6, 7, 8, 9, 10, 15]);
    for row in &rows {
        assert_eq!(row.notification_type, 7, "zone hub");
        assert_eq!(row.sub_name_id, 382845, "Press to Teleport...");
        assert!(row.x.is_finite() && row.y.is_finite() && row.z.is_finite());
    }
    // Nothing is attuned yet, so nothing is on offer.
    assert!(listed_ids(&WarpRegistry::curated().definitions_message()).is_empty());
}

#[test]
fn definition_request_lists_only_attuned_warpstones() {
    let mut registry = WarpRegistry::curated();
    let world = blind_world();
    let reply = registry
        .handle(&inner(56, &[]), &ctx_at([0.0; 3]), 0, &world)
        .expect("56 is a warpstone opcode");
    assert_eq!(reply.replies.len(), 1);
    assert!(listed_ids(&reply.replies[0]).is_empty());
    assert!(reply.relocation.is_none() && !reply.changed);

    registry.attune(LAKESHORE);
    let reply = registry
        .handle(&inner(56, &[]), &ctx_at([0.0; 3]), 0, &world)
        .unwrap();
    assert_eq!(listed_ids(&reply.replies[0]), [LAKESHORE]);

    // Native framing: opcode, then the exact length of what follows.
    let message = &reply.replies[0];
    assert_eq!(&message[..2], &57u16.to_le_bytes());
    let declared = i32::from_le_bytes(message[2..6].try_into().unwrap()) as usize;
    assert_eq!(declared, message.len() - 6);
    // Canary: a request with a body is not a 56.
    let forged = registry
        .handle(&inner(56, &[1]), &ctx_at([0.0; 3]), 0, &world)
        .unwrap();
    assert!(forged.replies.is_empty());
    assert!(matches!(
        forged.rejection,
        Some(WarpRejection::MalformedRequest(_))
    ));
}

#[test]
fn piece_073_reads_what_piece_077_writes() {
    let mut registry = WarpRegistry::curated();
    for id in [SANCTUARY, LAKESHORE] {
        registry.attune(id);
    }
    let message = registry.definitions_message();
    let read = piece_073::decode_poi_definitions(&message[6..]).expect("073 decodes the stream");
    let table = piece_073::PoiTable::new(read).expect("unique ids");
    assert_eq!(table.len(), 2);
    let sanctuary = table.by_id(SANCTUARY).unwrap();
    assert_eq!(
        (sanctuary.x, sanctuary.y, sanctuary.z),
        (
            SANCTUARY_POSITION[0],
            SANCTUARY_POSITION[1],
            SANCTUARY_POSITION[2]
        )
    );
    assert_eq!(sanctuary.atlas_name, "sacredgrove");
}

#[test]
fn definition_message_decoder_is_strict_about_the_length_prefix() {
    let mut registry = WarpRegistry::curated();
    registry.attune(LAKESHORE);
    let message = registry.definitions_message();
    assert!(decode_point_of_interest_message(&message).is_ok());
    let mut long = message.clone();
    long.push(0);
    assert!(decode_point_of_interest_message(&long).is_err());
    assert!(decode_point_of_interest_message(&message[..message.len() - 1]).is_err());
    assert!(decode_point_of_interest_message(&message[..5]).is_err());
    let mut wrong_opcode = message;
    wrong_opcode[0] = 56;
    assert!(decode_point_of_interest_message(&wrong_opcode).is_err());
}

#[test]
fn standing_near_a_warpstone_attunes_it_once() {
    let mut registry = WarpRegistry::curated();
    // 13 m away: outside the 12 m radius.
    let near = [SANCTUARY_POSITION[0] + 13.0, 0.0, SANCTUARY_POSITION[2]];
    assert!(!registry.visit(&ctx_at(near)).changed);
    assert!(!registry.is_attuned(SANCTUARY));

    let close = [SANCTUARY_POSITION[0] + 11.0, 0.0, SANCTUARY_POSITION[2]];
    let reply = registry.visit(&ctx_at(close));
    assert!(reply.changed && registry.is_attuned(SANCTUARY));
    assert_eq!(
        reply.notices,
        ["You are now attuned to the Sanctuary Warpstone."]
    );
    assert_eq!(listed_ids(&reply.replies[0]), [SANCTUARY], "57 refresh");

    let again = registry.visit(&ctx_at(close));
    assert!(!again.changed && again.notices.is_empty() && again.replies.is_empty());
}

#[test]
fn attunement_ignores_other_zones_and_bad_positions() {
    let mut registry = WarpRegistry::curated();
    for zone in [411, 9600, 1] {
        let mut ctx = ctx_at(SANCTUARY_POSITION);
        ctx.zone_id = zone;
        assert!(!registry.visit(&ctx).changed, "zone {zone}");
    }
    for bad in [
        [f32::NAN, 0.0, SANCTUARY_POSITION[2]],
        [SANCTUARY_POSITION[0], f32::INFINITY, SANCTUARY_POSITION[2]],
    ] {
        assert!(!registry.visit(&ctx_at(bad)).changed);
    }
    assert!(registry.state().is_empty());
}

#[test]
fn travel_to_an_unlocked_warpstone_relocates_with_server_authority() {
    let mut registry = at_sanctuary();
    registry.attune(LAKESHORE);
    // Flat terrain far below the stored height: the stored height wins.
    let low = registry
        .handle(
            &travel(LAKESHORE),
            &ctx_at(SANCTUARY_POSITION),
            0,
            &flat_world(0.0),
        )
        .unwrap();
    let relocation = low.relocation.expect("relocation");
    // Original captured spawn for Lakeshore (id 9), p2.pcap frame 457.
    assert_eq!(
        relocation.position,
        [-733.9340209960938, 9.086000442504883, 945.948974609375]
    );
    assert_eq!(
        relocation.rotation,
        [0.9876689314842224, 0.0, -0.15655702352523804]
    );
    assert!(low.rejection.is_none() && low.notices.is_empty() && !low.changed);

    // Terrain above the stored height lifts the arrival 0.5 m over the ground.
    let mut registry = at_sanctuary();
    registry.attune(LAKESHORE);
    let high = registry
        .handle(
            &travel(LAKESHORE),
            &ctx_at(SANCTUARY_POSITION),
            0,
            &flat_world(30.0),
        )
        .unwrap();
    assert_eq!(high.relocation.unwrap().position[1], 30.5);
}

#[test]
fn a_hub_without_a_captured_spawn_keeps_the_players_facing() {
    let mut registry = at_sanctuary();
    let ctx = ctx_at(LAKESHORE_POSITION);
    let reply = registry
        .handle(&travel(SANCTUARY), &ctx, 0, &blind_world())
        .unwrap();
    let relocation = reply.relocation.expect("relocation");
    assert_eq!(relocation.position, SANCTUARY_POSITION);
    assert_eq!(
        relocation.rotation, ctx.rotation,
        "no facing convention invented"
    );
}

#[test]
fn forged_and_locked_requests_do_not_relocate_and_look_alike() {
    let mut registry = at_sanctuary();
    let world = blind_world();
    let ctx = ctx_at(LAKESHORE_POSITION);
    let mut notices = Vec::new();
    // Locked curated id, unknown ids, the hub-range edges, and ids a client
    // would never send.
    for id in [LAKESHORE, 0, -1, 1, 2, 11, 99_999, i32::MAX, i32::MIN] {
        let reply = registry.handle(&travel(id), &ctx, 0, &world).unwrap();
        assert!(reply.relocation.is_none(), "id {id} must not move anyone");
        assert!(reply.rejection.is_some(), "id {id}");
        notices.push(reply.notices);
    }
    assert!(
        notices.windows(2).all(|pair| pair[0] == pair[1]),
        "locked and unknown ids give the same feedback"
    );
    assert_eq!(notices[0], ["You have not attuned to that warpstone yet."]);
    assert!(!registry.is_attuned(LAKESHORE), "a refusal attunes nothing");
}

#[test]
fn requests_far_from_any_warpstone_still_cannot_unlock_one() {
    // Travel works from anywhere once attuned, but a far-away character cannot
    // attune by asking: only the server's position attunes (R).
    let mut registry = WarpRegistry::curated();
    let far = ctx_at([5000.0, 0.0, 5000.0]);
    assert!(!registry.visit(&far).changed);
    let reply = registry
        .handle(&travel(SANCTUARY), &far, 0, &blind_world())
        .unwrap();
    assert_eq!(
        reply.rejection,
        Some(WarpRejection::LockedDestination(SANCTUARY))
    );
    assert!(reply.relocation.is_none());
}

#[test]
fn travel_is_refused_while_knocked_out_in_an_encounter_or_busy() {
    for (blocker, text) in [
        (
            Blocker::KnockedOut,
            "You cannot use a warpstone while knocked out.",
        ),
        (
            Blocker::Encounter,
            "Finish what you are doing before using a warpstone.",
        ),
        (
            Blocker::Busy,
            "Finish what you are doing before using a warpstone.",
        ),
    ] {
        let mut registry = at_sanctuary();
        registry.attune(LAKESHORE);
        let mut ctx = ctx_at(SANCTUARY_POSITION);
        ctx.blocked = Some(blocker);
        let reply = registry
            .handle(&travel(LAKESHORE), &ctx, 0, &blind_world())
            .unwrap();
        assert!(reply.relocation.is_none(), "{blocker:?}");
        assert_eq!(reply.rejection, Some(WarpRejection::Blocked(blocker)));
        assert_eq!(reply.notices, [text]);

        // A refusal costs no cooldown: the same instant, unblocked, works.
        ctx.blocked = None;
        let reply = registry
            .handle(&travel(LAKESHORE), &ctx, 0, &blind_world())
            .unwrap();
        assert!(reply.relocation.is_some(), "{blocker:?} then free");
    }
}

#[test]
fn travel_is_open_world_only() {
    for zone in [411, 9600, 31, u32::MAX] {
        let mut registry = at_sanctuary();
        registry.attune(LAKESHORE);
        let mut ctx = ctx_at(SANCTUARY_POSITION);
        ctx.zone_id = zone;
        let reply = registry
            .handle(&travel(LAKESHORE), &ctx, 0, &blind_world())
            .unwrap();
        assert!(reply.relocation.is_none(), "zone {zone}");
        assert_eq!(reply.rejection, Some(WarpRejection::WrongZone(zone)));
    }
}

#[test]
fn cooldown_spaces_successful_teleports_and_is_tunable_within_bounds() {
    let mut registry = at_sanctuary();
    registry.attune(LAKESHORE);
    let world = blind_world();
    let ctx = ctx_at(SANCTUARY_POSITION);
    assert!(
        registry
            .handle(&travel(LAKESHORE), &ctx, 10_000, &world)
            .unwrap()
            .relocation
            .is_some()
    );
    let soon = registry
        .handle(&travel(SANCTUARY), &ctx, 14_999, &world)
        .unwrap();
    assert!(soon.relocation.is_none());
    assert_eq!(soon.rejection, Some(WarpRejection::Cooldown));
    assert!(
        registry
            .handle(&travel(SANCTUARY), &ctx, 15_000, &world)
            .unwrap()
            .relocation
            .is_some()
    );
    // A clock that went backwards (new session) never underflows or panics.
    let earlier = registry
        .handle(&travel(LAKESHORE), &ctx, 0, &world)
        .unwrap();
    assert_eq!(earlier.rejection, Some(WarpRejection::Cooldown));

    let mut registry = at_sanctuary();
    registry.attune(LAKESHORE);
    registry
        .set_rules(WarpRules {
            cooldown_ms: 0,
            ..WarpRules::default()
        })
        .expect("zero cooldown is a valid tuning");
    for _ in 0..3 {
        assert!(
            registry
                .handle(&travel(LAKESHORE), &ctx, 1, &world)
                .unwrap()
                .relocation
                .is_some()
        );
    }
    for bad in [
        WarpRules {
            attune_radius_m: 0.0,
            ..WarpRules::default()
        },
        WarpRules {
            attune_radius_m: f32::NAN,
            ..WarpRules::default()
        },
        WarpRules {
            attune_radius_m: 1e9,
            ..WarpRules::default()
        },
        WarpRules {
            cooldown_ms: u64::MAX,
            ..WarpRules::default()
        },
        WarpRules {
            stuck_cooldown_ms: 3_600_001,
            ..WarpRules::default()
        },
        WarpRules {
            below_world_m: 0.0,
            ..WarpRules::default()
        },
    ] {
        assert!(registry.set_rules(bad).is_err(), "{bad:?}");
    }
}

#[test]
fn attunement_survives_a_relog_and_a_save_cannot_unlock_other_ids() {
    let mut registry = at_sanctuary();
    registry.attune(LAKESHORE);
    let json = serde_json::to_string(&registry.state()).unwrap();
    assert_eq!(json, r#"{"attuned":[5,9]}"#);

    let mut relogged = WarpRegistry::curated();
    relogged.restore(&serde_json::from_str::<WarpState>(&json).unwrap());
    assert_eq!(
        listed_ids(&relogged.definitions_message()),
        [SANCTUARY, LAKESHORE]
    );
    let reply = relogged
        .handle(
            &travel(LAKESHORE),
            &ctx_at(SANCTUARY_POSITION),
            0,
            &blind_world(),
        )
        .unwrap();
    assert!(reply.relocation.is_some(), "still valid after relog");

    // Hand-edited or stale saves: ids that are not curated warpstones are dropped.
    let hostile: WarpState =
        serde_json::from_str(r#"{"attuned":[-5,0,20,99999,2147483647,9]}"#).unwrap();
    let mut loaded = WarpRegistry::curated();
    loaded.restore(&hostile);
    assert_eq!(
        loaded.state().attuned.into_iter().collect::<Vec<_>>(),
        [LAKESHORE]
    );
    // Unknown fields are refused instead of silently ignored.
    assert!(serde_json::from_str::<WarpState>(r#"{"attuned":[5],"admin":true}"#).is_err());
    // An empty save is an empty set.
    assert!(serde_json::from_str::<WarpState>("{}").unwrap().is_empty());
}

#[test]
fn safe_teleport_goes_to_the_nearest_warpstone_and_takes_no_position() {
    let world = flat_world(0.0);
    // Standing on the ground and not attuned anywhere: allowed once (R) and it
    // picks the warpstone nearest the SERVER's position.
    let mut registry = WarpRegistry::curated();
    let near_lakeshore = ctx_at([LAKESHORE_POSITION[0] + 200.0, 5.0, LAKESHORE_POSITION[2]]);
    let reply = registry
        .handle(&inner(122, &[]), &near_lakeshore, 0, &world)
        .unwrap();
    let relocation = reply.relocation.expect("relocation");
    assert_eq!(
        relocation.position[0], -733.9340209960938,
        "Lakeshore spawn"
    );

    // Not fallen: the long stuck cooldown stops a second use.
    let again = registry
        .handle(&inner(122, &[]), &near_lakeshore, 60_000, &world)
        .unwrap();
    assert_eq!(again.rejection, Some(WarpRejection::Cooldown));
    assert!(
        registry
            .handle(&inner(122, &[]), &near_lakeshore, 300_000, &world)
            .unwrap()
            .relocation
            .is_some()
    );

    // A verified fall below the terrain only waits out the short cooldown.
    let mut registry = WarpRegistry::curated();
    let fallen = ctx_at([LAKESHORE_POSITION[0], -500.0, LAKESHORE_POSITION[2]]);
    assert!(
        registry
            .handle(&inner(122, &[]), &fallen, 0, &world)
            .unwrap()
            .relocation
            .is_some()
    );
    assert!(
        registry
            .handle(&inner(122, &[]), &fallen, 5_000, &world)
            .unwrap()
            .relocation
            .is_some()
    );
    assert!(
        registry
            .handle(&inner(122, &[]), &fallen, 6_000, &world)
            .unwrap()
            .rejection
            .is_some()
    );

    // Canary: a body (a client-supplied position) makes the request malformed.
    let mut registry = WarpRegistry::curated();
    let forged = registry
        .handle(&inner(122, &[0; 12]), &near_lakeshore, 0, &world)
        .unwrap();
    assert!(forged.relocation.is_none());
    assert!(matches!(
        forged.rejection,
        Some(WarpRejection::MalformedRequest(_))
    ));
}

#[test]
fn safe_teleport_shares_the_travel_gates() {
    let world = flat_world(0.0);
    for (zone, blocked) in [
        (411, None),
        (0, Some(Blocker::Encounter)),
        (0, Some(Blocker::KnockedOut)),
    ] {
        let mut registry = WarpRegistry::curated();
        let mut ctx = ctx_at([-1400.0, -500.0, 350.0]);
        ctx.zone_id = zone;
        ctx.blocked = blocked;
        let reply = registry.handle(&inner(122, &[]), &ctx, 0, &world).unwrap();
        assert!(
            reply.relocation.is_none(),
            "zone {zone} blocked {blocked:?}"
        );
    }
}

#[test]
fn friend_teleport_checks_target_and_context_and_takes_no_position() {
    let friends = RefCell::new(Vec::new());
    let known = |guid: u64| -> Option<FriendView> {
        friends.borrow_mut().push(guid);
        match guid {
            500 => Some(FriendView {
                zone_id: 0,
                position: [10.0, 4.0, 20.0],
                blocked: false,
            }),
            501 => Some(FriendView {
                zone_id: 411,
                position: [1.0, 1.0, 1.0],
                blocked: false,
            }),
            502 => Some(FriendView {
                zone_id: 0,
                position: [3.0, 3.0, 3.0],
                blocked: true,
            }),
            _ => None,
        }
    };
    let world = WarpWorld {
        ground: &|_, _| Some(2.0),
        friend: &known,
    };
    let go = |guid: u64| inner(58, &WorldTeleportRequest { player_guid: guid }.encode());

    let mut registry = WarpRegistry::curated();
    let ctx = ctx_at([0.0; 3]);
    let ok = registry.handle(&go(500), &ctx, 0, &world).unwrap();
    let relocation = ok.relocation.expect("friend teleport");
    assert_eq!(relocation.position, [10.0, 4.0, 20.0]);
    assert_eq!(relocation.rotation, ctx.rotation);

    let mut notices = Vec::new();
    for (guid, now) in [(77, 10_000), (501, 10_000), (502, 10_000), (999, 10_000)] {
        let reply = registry.handle(&go(guid), &ctx, now, &world).unwrap();
        assert!(reply.relocation.is_none(), "guid {guid}");
        notices.push(reply.notices);
    }
    assert!(
        notices.windows(2).all(|pair| pair[0] == pair[1]),
        "no probing signal"
    );
    assert!(
        !friends.borrow().contains(&77),
        "self is refused before any lookup"
    );

    // Cooldown, blocked requester and wrong zone apply to friend teleports too.
    assert_eq!(
        registry
            .handle(&go(500), &ctx, 1_000, &world)
            .unwrap()
            .rejection,
        Some(WarpRejection::Cooldown)
    );
    let mut blocked = ctx;
    blocked.blocked = Some(Blocker::Encounter);
    assert!(
        registry
            .handle(&go(500), &blocked, 99_000, &world)
            .unwrap()
            .relocation
            .is_none()
    );
    let mut elsewhere = ctx;
    elsewhere.zone_id = 9600;
    assert!(
        registry
            .handle(&go(500), &elsewhere, 99_000, &world)
            .unwrap()
            .relocation
            .is_none()
    );
}

#[test]
fn hostile_bodies_never_panic_or_move_anyone() {
    let mut registry = at_sanctuary();
    registry.attune(LAKESHORE);
    let ctx = ctx_at(SANCTUARY_POSITION);
    let world = blind_world();
    for opcode in [56u16, 58, 90, 122] {
        for length in 0..40usize {
            for fill in [0x00u8, 0x7f, 0xff] {
                let reply = registry
                    .handle(&inner(opcode, &vec![fill; length]), &ctx, 0, &world)
                    .expect("warpstone opcode");
                let exact = matches!((opcode, length), (56, 0) | (58, 8) | (90, 4) | (122, 0));
                if !exact {
                    assert!(
                        matches!(reply.rejection, Some(WarpRejection::MalformedRequest(_))),
                        "opcode {opcode} length {length}"
                    );
                    assert!(reply.relocation.is_none() && reply.replies.is_empty());
                }
            }
        }
    }
    // Not ours: truncated headers and other opcodes fall through untouched.
    for packet in [
        &[][..],
        &[90][..],
        &[15, 0, 1, 0][..],
        &[91, 0, 9, 0, 0, 0][..],
    ] {
        assert!(registry.handle(packet, &ctx, 0, &world).is_none());
    }
}

/// Private gate (skips when the lab files are absent): every curated row equals
/// the row the original client accepted in the preserved gateway `57`, bit for
/// bit, and the captured message passes the strict decoder (so the length
/// prefix the native reader needs is the one the original server sent).
#[test]
fn curated_rows_equal_the_captured_original_57() {
    let capture = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../../lab/captures/legacy-gateway-20261008T014620082859Z/messages/0009-ready.bin",
    );
    let Ok(frame) = std::fs::read(&capture) else {
        eprintln!("skipped: {} not present", capture.display());
        return;
    };
    // Tunnel header: u16 family, flag, u32 length; the inner message follows.
    let message = &frame[7..];
    let captured = decode_point_of_interest_message(message).expect("captured 57 decodes strictly");
    assert_eq!(captured.len(), 149);
    for row in curated_definitions() {
        let original = captured
            .iter()
            .find(|candidate| candidate.id == row.id)
            .unwrap_or_else(|| panic!("hub {} missing from the capture", row.id));
        assert_eq!(&row, original, "hub {}", row.id);
    }
}

/// L039: flat synthetic ground at -40 m around Cobblestone Village (warpstone 7)
/// with two 20 m strip holes, x -1960..-1940 and z 420..440, the size of zone
/// 0's holes under runtime objects (the pattern of the `terrain.rs` tests).
fn cobblestone_ground() -> osfr_tools::terrain::HeightGrid {
    let mut grid =
        osfr_tools::terrain::HeightGrid::new([-2000.0, 300.0], [-1800.0, 500.0], 2.0).unwrap();
    let p = |x: f32, z: f32| [x, -40.0, z];
    for (west, east) in [(-2000.0, -1960.0), (-1940.0, -1800.0)] {
        for (south, north) in [(300.0, 420.0), (440.0, 500.0)] {
            grid.add_triangle([p(west, south), p(east, south), p(east, north)]);
            grid.add_triangle([p(west, south), p(east, north), p(west, north)]);
        }
    }
    grid
}

const COBBLESTONE_POSITION: [f32; 3] = [-1904.46, -39.79, 413.316];

fn recover(
    registry: &WarpRegistry,
    zone_id: u32,
    position: [f32; 3],
) -> Option<([f32; 3], &'static str)> {
    let grid = cobblestone_ground();
    let mut ctx = ctx_at(position);
    ctx.zone_id = zone_id;
    let (to, reason) = registry.recover_login_pose(&ctx, &|x, z| grid.height_at(x, z))?;
    assert_eq!(
        to.rotation, ctx.rotation,
        "Cobblestone keeps the saved facing"
    );
    Some((to.position, reason))
}

fn tuned_to_50_m() -> WarpRegistry {
    let mut registry = WarpRegistry::curated();
    registry
        .set_rules(WarpRules {
            login_below_world_m: 50.0,
            ..WarpRules::default()
        })
        .unwrap();
    registry
}

#[test]
fn login_pose_off_the_map_or_deep_under_it_moves_to_the_nearest_warpstone() {
    // Cobblestone's row, raised to the terrain plus the 0.5 m arrival lift.
    let ground = cobblestone_ground()
        .height_at(COBBLESTONE_POSITION[0], COBBLESTONE_POSITION[2])
        .unwrap();
    assert!((ground + 40.0).abs() < 0.01);
    let arrival = [
        COBBLESTONE_POSITION[0],
        ground + 0.5,
        COBBLESTONE_POSITION[2],
    ];
    let registry = WarpRegistry::curated();
    let off_map = [-2600.0, 0.0, 400.0];
    assert_eq!(
        recover(&registry, OPEN_WORLD_ZONE, off_map),
        Some((arrival, "off_map"))
    );
    // 20 m past the terrain edge: ground 40 m east, none west, so not a hole.
    assert_eq!(
        recover(&registry, OPEN_WORLD_ZONE, [-2020.0, 0.0, 400.0]),
        Some((arrival, "off_map"))
    );
    assert_eq!(
        recover(&registry, OPEN_WORLD_ZONE, [-1900.0, -300.0, 400.0]),
        Some((arrival, "below_world"))
    );
    // Tuned to 50 m, a pose 60 m under moves too.
    assert_eq!(
        recover(&tuned_to_50_m(), OPEN_WORLD_ZONE, [-1900.0, -100.0, 400.0]),
        Some((arrival, "below_world"))
    );
    // Idempotent: a recovered pose passes, whatever the tuning.
    assert_eq!(recover(&registry, OPEN_WORLD_ZONE, arrival), None);
    assert_eq!(recover(&tuned_to_50_m(), OPEN_WORLD_ZONE, arrival), None);
    // The same pose the 122 rescue picks: one nearest-warpstone choice.
    let mut rescue = WarpRegistry::curated();
    let grid = cobblestone_ground();
    let world = WarpWorld {
        ground: &|x, z| grid.height_at(x, z),
        friend: &|_| None,
    };
    let reply = rescue
        .handle(&inner(122, &[]), &ctx_at(off_map), 0, &world)
        .unwrap();
    assert_eq!(reply.relocation.unwrap().position, arrival);
}

#[test]
fn login_pose_on_the_ground_in_caves_holes_and_other_zones_stays() {
    for (name, zone, position) in [
        ("on the ground", OPEN_WORLD_ZONE, [-1900.0, -40.0, 400.0]),
        (
            "10 m under: a cave",
            OPEN_WORLD_ZONE,
            [-1900.0, -50.0, 400.0],
        ),
        (
            "in a hole with ground east and west",
            OPEN_WORLD_ZONE,
            [-1950.0, -40.0, 400.0],
        ),
        (
            "in a hole with ground north and south",
            OPEN_WORLD_ZONE,
            [-1850.0, -40.0, 430.0],
        ),
        (
            "deep under a field battle scene",
            9600,
            [-1900.0, -900.0, 400.0],
        ),
        ("off the map in another zone", 411, [-2600.0, 0.0, 400.0]),
    ] {
        assert_eq!(
            recover(&WarpRegistry::curated(), zone, position),
            None,
            "{name}"
        );
        assert_eq!(recover(&tuned_to_50_m(), zone, position), None, "{name}");
    }
    // A mine 190 m under the grid (zone 0 has interiors 188 m under) stays by default.
    assert_eq!(
        recover(
            &WarpRegistry::curated(),
            OPEN_WORLD_ZONE,
            [-1900.0, -230.0, 400.0]
        ),
        None
    );
}

#[test]
fn login_below_world_margin_is_bounded_and_a_refused_rule_changes_nothing() {
    let mut registry = WarpRegistry::curated();
    for bad in [2.0, 600.0, -50.0, f32::NAN, f32::INFINITY] {
        let rules = WarpRules {
            login_below_world_m: bad,
            ..WarpRules::default()
        };
        assert!(registry.set_rules(rules).is_err(), "{bad}");
    }
    // Still the 250 m default: 230 m under stays, 300 m under moves.
    assert_eq!(
        recover(&registry, OPEN_WORLD_ZONE, [-1900.0, -270.0, 400.0]),
        None
    );
    assert!(recover(&registry, OPEN_WORLD_ZONE, [-1900.0, -300.0, 400.0]).is_some());
    for good in [3.0, 500.0] {
        let rules = WarpRules {
            login_below_world_m: good,
            ..WarpRules::default()
        };
        assert!(registry.set_rules(rules).is_ok(), "{good}");
    }
}

// ---------------------------------------------------------------------------
// T546: the three zone warpstone actors.
//
// Sanctuary-main (20261019@10f129e) places three `sg_warpstone_01.adr` NPCs at
// the Sanctuary, Lakeshore and Snowhill hubs; our 11 Legacy stones cover the
// other towns. These are `npc_spawns_extra.json` actors, not travel data:
// `piece_077` is unchanged and attunement still comes from the character's
// position near the curated hub. The spin is the model's own `loc_stand` clip:
// the actor definition's only animation slot maps `loc_stand` (AnimationTypes.xml
// id 1) to `sg_warpstone.gr2`, whose skeleton carries `SPIN_BASE`/`SPIN_EYE`
// bones, and every captured warpstone AddNpc (startup capture, reader
// `0x00921a10`) carries `animation_id` 1. The server sends that same actor;
// there is no separate rotation packet to send.
// ---------------------------------------------------------------------------

/// One new stone: the actor guid the row spawns, the curated hub it belongs to,
/// its client name and locale id, and the Lua placement.
struct Stone {
    guid: u64,
    hub_id: i32,
    name: &'static str,
    name_id: u32,
    position: [f32; 3],
    heading: f32,
}

const NEW_STONES: [Stone; 3] = [
    Stone {
        guid: 100000033167,
        hub_id: 5,
        name: "Sanctuary Warpstone",
        name_id: 20831,
        position: [-1418.3218, -27.002932, 347.82056],
        heading: -0.0622950028,
    },
    Stone {
        guid: 100000033558,
        hub_id: 9,
        name: "Lakeshore Warpstone",
        name_id: 41298,
        position: [-721.7169, 9.560479, 948.71606],
        heading: -1.81613004,
    },
    Stone {
        guid: 100000033602,
        hub_id: 3,
        name: "Snowhill Warpstone",
        name_id: 20830,
        position: [50.92852, 32.425617, 370.30267],
        heading: 1.21874107,
    },
];

fn planar_distance(left: [f32; 3], right: [f32; 3]) -> f32 {
    (left[0] - right[0]).hypot(left[2] - right[2])
}

#[test]
fn the_three_zone_warpstones_are_drawn_at_their_curated_hubs() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let project_root = crate_root.parent().unwrap().parent().unwrap();
    let lab_root = project_root.parent().unwrap().join("lab");
    let content = crate_root.join("data/content");
    let locale: std::collections::BTreeMap<u64, String> =
        json::read(&lab_root.join("reports/locale-id-to-text.json"))
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|row| Some((row["id"].as_u64()?, row["text"].as_str()?.to_owned())))
            .collect();
    let text = |id: u64| -> &str {
        locale
            .get(&id)
            .map(String::as_str)
            .unwrap_or_else(|| panic!("locale id {id} is absent"))
    };

    let extra = json::read(&content.join("npc_spawns_extra.json"));
    let rows = extra["records"].as_array().unwrap();
    let position = |row: &serde_json::Value| -> [f32; 3] {
        let position = row["position"].as_array().unwrap();
        [
            position[0].as_f64().unwrap() as f32,
            position[1].as_f64().unwrap() as f32,
            position[2].as_f64().unwrap() as f32,
        ]
    };
    let hubs = curated_definitions();
    let mut visibility =
        NpcVisibility::load(&content).expect("the production visibility loader accepts it");

    for stone in &NEW_STONES {
        let row = rows
            .iter()
            .find(|row| row["spawn_guid"].as_u64() == Some(stone.guid))
            .unwrap_or_else(|| panic!("{} stone {} is absent", stone.name, stone.guid));
        assert_eq!(
            row["legacy_npc_id"],
            serde_json::Value::Null,
            "{}: a zone placement gets no Legacy NPC id",
            stone.name
        );
        assert!(
            row.get("content_key").is_none(),
            "{}: a stone is drawn, not a menu NPC",
            stone.name
        );
        assert_eq!(row["model_id"].as_u64(), Some(280), "{}", stone.name);
        assert_eq!(
            row["model_file_name"].as_str(),
            Some("sg_warpstone_01.adr"),
            "{}",
            stone.name
        );
        assert_eq!(row["name"].as_str(), Some(stone.name));
        assert_eq!(
            row["name_string_id"].as_u64(),
            Some(u64::from(stone.name_id)),
            "{}: client NameId",
            stone.name
        );
        assert_eq!(text(u64::from(stone.name_id)), stone.name);
        let written = position(row);

        // The stone stands at the curated hub of the same town. The two names come
        // from the client locale, so a renamed or moved row fails here.
        let town = stone
            .name
            .strip_suffix(" Warpstone")
            .expect("a warpstone name ends in Warpstone");
        let matching: Vec<_> = hubs
            .iter()
            .filter(|hub| text(hub.name_id as u64) == town)
            .collect();
        assert_eq!(
            matching.len(),
            1,
            "{}: one curated hub named {town}",
            stone.name
        );
        let hub = matching[0];
        assert_eq!(hub.id, stone.hub_id, "{}: curated hub id", stone.name);
        let distance = planar_distance(written, [hub.x, hub.y, hub.z]);
        assert!(
            distance <= 15.0,
            "{} stands {distance:.2} m from the {town} hub",
            stone.name
        );

        // The row's position and heading are the Lua line's own values.
        for axis in 0..3 {
            assert!(
                (written[axis] - stone.position[axis]).abs() < 0.001,
                "{}: axis {axis} differs from the Lua line",
                stone.name
            );
        }
        assert!(
            (row["heading"].as_f64().unwrap() as f32 - stone.heading).abs() < 0.000_001,
            "{}: heading differs from the Lua line",
            stone.name
        );

        // No second model-280 actor stands at the same town: exactly one (this
        // stone) is within 50 m in the production table, which covers the regional
        // rows as well as the 11 Legacy stones in other towns.
        let neighbours = visibility
            .model_actors()
            .filter(|(zone, model, other)| {
                *zone == OPEN_WORLD_ZONE
                    && *model == 280
                    && planar_distance(written, *other) <= 50.0
            })
            .count();
        assert_eq!(
            neighbours, 1,
            "{}: one model-280 actor within 50 m",
            stone.name
        );

        // The production loader draws this actor, at this position, facing the Lua
        // heading, on the animation every captured warpstone carries.
        let spawn = visibility
            .update(OPEN_WORLD_ZONE, stone.position)
            .expect("visibility update")
            .iter()
            .filter_map(|packet| startup_world::parse_add_npc(packet).ok())
            .find(|spawn| spawn.actor_id == stone.guid)
            .unwrap_or_else(|| {
                panic!(
                    "{} actor {} is not drawn at its position",
                    stone.name, stone.guid
                )
            });
        assert_eq!(spawn.model_id, 280);
        assert_eq!(spawn.name_string_id, stone.name_id as i32);
        // `loc_stand`, the actor definition's only animation slot (the spin).
        assert_eq!(spawn.animation_id, 1);
        let facing = [stone.heading.sin(), 0.0, stone.heading.cos(), 0.0];
        for axis in 0..4 {
            assert!(
                (spawn.rotation[axis] - facing[axis]).abs() < 0.000_001,
                "{}: rotation axis {axis}",
                stone.name
            );
        }
        for axis in 0..3 {
            assert!(
                (spawn.position[axis] - stone.position[axis]).abs() < 0.01,
                "{}: drawn position axis {axis}",
                stone.name
            );
        }
    }
}
