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
