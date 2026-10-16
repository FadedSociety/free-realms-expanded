//! Piece 077 (warpstones and travel) tests: recovered wire layouts for the
//! point-of-interest destination list, the zone teleport requests and the
//! position/rotation update the server answers a warpstone request with, plus
//! the reconstructed locked-destination gate.
//!
//! Evidence boundary, read before the assertions:
//!
//! ORIGINAL, first-party native client (`lab/decompiled/client/FreeRealms/pseudocode.c`,
//! sha256 `f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3`):
//! * `FUN_00b97a20` installs `PacketPointOfInterestDefinitionRequest::vftable`
//!   and writes base id `0x38` (56); `FUN_00b97ad0` installs the reply vftable
//!   and writes `0x39` (57).
//! * `FUN_00b97b80` installs `PacketZoneTeleportRequest::vftable`, writes `0x5a`
//!   (90) and stores exactly one 32-bit payload argument.
//! * `FUN_00bf7a50` writes `0x7a` (122) for `PacketZoneSafeTeleportRequest`;
//!   `FUN_00c1a5e0` writes `0x3a` (58) for `PacketWorldTeleportRequest` and
//!   stores two 32-bit arguments (a 64-bit guid).
//! * `FUN_0089dd60` calls `FUN_0089d180(0xc)` before installing
//!   `ClientUpdatePacketUpdateLocation::vftable` (base family 38, sub-opcode 12).
//! * `FUN_00b97c40` is the `ClientPointOfInterestManager::InternalDs` column
//!   table: Id, Name, Zone ID, X, Y, Z, Heading, IconId, NotificationType,
//!   SubName, BreadcrumbQuestId, AtlasName.
//! * `FUN_00b991d0` only builds the op-90 request after looking the id up in the
//!   client's own point-of-interest table.
//!
//! ORIGINAL, preserved live captures (`lab/references/OpenSourceFreeRealms/
//! ofrserver/Packets`, hashes pinned in `CAPTURED_TELEPORT_SAMPLES`): eight
//! `PacketZoneTeleportRequest` frames and their `38/12` replies, byte-exact.
//!
//! COMMUNITY comparators (reconstructed servers, not first-party):
//! `lab/references/Sanctuary` (`PacketZoneTeleportRequest`,
//! `ClientUpdatePacketUpdateLocation`) and `lab/decompiled/server`
//! (`LoginManager.HandlePacketZoneTeleportRequest`).
//!
//! RECONSTRUCTED, labelled at the call site: the attunement gate itself. The
//! shipped client locale says "You are now attuned to the Solace Warpstone."
//! (id 41300) and the community wiki says a warpstone must be activated before
//! the Atlas offers it; no captured frame shows a server rejection, so the gate
//! below is server-side policy, not recovered native behavior.
// The module is not registered in `src/lib.rs` (lead-owned). Including the exact
// source file keeps this target compiling against the real artifact. Integration
// step for the lead: register the module and drop this `#[path]` include.
#[path = "../src/restoration/piece_077.rs"]
#[allow(dead_code)]
mod piece_077;

use piece_077::{
    CAPTURED_TELEPORT_SAMPLES, CLIENT_UPDATE_PACKET_OPCODE,
    POINT_OF_INTEREST_DEFINITION_REPLY_OPCODE, POINT_OF_INTEREST_DEFINITION_REQUEST_OPCODE,
    PointOfInterestDefinition, UPDATE_LOCATION_SUB_OPCODE, UpdateLocation,
    WORLD_TELEPORT_REQUEST_OPCODE, WarpCodecError, WarpDestination, WarpRegistry, WarpRejection,
    WorldTeleportRequest, ZONE_SAFE_TELEPORT_REQUEST_OPCODE, ZONE_TELEPORT_REQUEST_OPCODE,
    ZoneSafeTeleportRequest, ZoneTeleportRequest, captured_destinations,
    decode_point_of_interest_reply, encode_point_of_interest_reply,
};

fn unhex(hex: &str) -> Vec<u8> {
    assert!(hex.len() % 2 == 0, "odd hex length");
    (0..hex.len() / 2)
        .map(|i| u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).expect("valid hex"))
        .collect()
}

#[test]
fn opcode_constants_match_the_native_vftable_ids() {
    assert_eq!(POINT_OF_INTEREST_DEFINITION_REQUEST_OPCODE, 56);
    assert_eq!(POINT_OF_INTEREST_DEFINITION_REPLY_OPCODE, 57);
    assert_eq!(WORLD_TELEPORT_REQUEST_OPCODE, 58);
    assert_eq!(ZONE_TELEPORT_REQUEST_OPCODE, 90);
    assert_eq!(ZONE_SAFE_TELEPORT_REQUEST_OPCODE, 122);
    assert_eq!(CLIENT_UPDATE_PACKET_OPCODE, 38);
    assert_eq!(UPDATE_LOCATION_SUB_OPCODE, 12);
}

#[test]
fn captured_teleport_requests_decode_and_re_encode_byte_exactly() {
    assert!(
        !CAPTURED_TELEPORT_SAMPLES.is_empty(),
        "capture samples pinned"
    );
    for sample in CAPTURED_TELEPORT_SAMPLES {
        assert_eq!(
            sample.capture_sha256.len(),
            64,
            "{} hash pinned",
            sample.capture
        );
        let body = unhex(sample.request_body_hex);
        let request = ZoneTeleportRequest::decode(&body).unwrap_or_else(|error| {
            panic!(
                "{} frame {}: {error:?}",
                sample.capture, sample.request_frame
            )
        });
        assert_eq!(
            request.destination_id, sample.destination_id,
            "{} frame {}",
            sample.capture, sample.request_frame
        );
        assert_eq!(request.encode().to_vec(), body, "{}", sample.capture);
    }
}

#[test]
fn captured_teleport_replies_round_trip_byte_exactly() {
    for sample in CAPTURED_TELEPORT_SAMPLES {
        let message = unhex(sample.reply_message_hex);
        let reply = UpdateLocation::decode_message(&message).unwrap_or_else(|error| {
            panic!("{} frame {}: {error:?}", sample.capture, sample.reply_frame)
        });
        assert!(
            reply.teleport,
            "{} frame {}",
            sample.capture, sample.reply_frame
        );
        assert_eq!(reply.position[3], 1.0, "captured position W");
        assert_eq!(reply.rotation[1], 0.0, "captured rotation Y lane");
        assert_eq!(reply.rotation[3], 0.0, "captured rotation W lane");
        assert_eq!(reply.encode_message(), message, "{}", sample.capture);
    }
}

#[test]
fn captured_teleport_replies_carry_the_destination_spawn_point() {
    let destinations = captured_destinations();
    assert_eq!(destinations.len(), 6);
    for destination in &destinations {
        let sample = CAPTURED_TELEPORT_SAMPLES
            .iter()
            .find(|sample| sample.destination_id == destination.id)
            .unwrap_or_else(|| panic!("no captured sample for {}", destination.id));
        let reply = UpdateLocation::decode_message(&unhex(sample.reply_message_hex)).unwrap();
        assert_eq!(
            reply.position, destination.position,
            "id {}",
            destination.id
        );
        assert_eq!(
            reply.rotation, destination.rotation,
            "id {}",
            destination.id
        );
    }
}

#[test]
fn malformed_zone_teleport_bodies_are_rejected() {
    for (body, expected) in [
        (
            vec![],
            WarpCodecError::Truncated {
                expected: 4,
                got: 0,
            },
        ),
        (
            vec![1],
            WarpCodecError::Truncated {
                expected: 4,
                got: 1,
            },
        ),
        (
            vec![1, 2, 3],
            WarpCodecError::Truncated {
                expected: 4,
                got: 3,
            },
        ),
        (
            vec![1, 2, 3, 4, 5],
            WarpCodecError::TrailingBytes {
                expected: 4,
                got: 5,
            },
        ),
        (
            vec![0; 8],
            WarpCodecError::TrailingBytes {
                expected: 4,
                got: 8,
            },
        ),
    ] {
        assert_eq!(ZoneTeleportRequest::decode(&body).unwrap_err(), expected);
    }
}

#[test]
fn safe_and_world_teleport_requests_validate_their_own_lengths() {
    assert_eq!(
        ZoneSafeTeleportRequest::decode(&[]).unwrap(),
        ZoneSafeTeleportRequest
    );
    assert_eq!(
        ZoneSafeTeleportRequest::decode(&[0]).unwrap_err(),
        WarpCodecError::TrailingBytes {
            expected: 0,
            got: 1
        }
    );

    let guid = 0x1122_3344_5566_7788u64;
    let encoded = WorldTeleportRequest { player_guid: guid }.encode();
    assert_eq!(encoded, guid.to_le_bytes());
    assert_eq!(
        WorldTeleportRequest::decode(&encoded).unwrap().player_guid,
        guid
    );
    assert_eq!(
        WorldTeleportRequest::decode(&encoded[..7]).unwrap_err(),
        WarpCodecError::Truncated {
            expected: 8,
            got: 7
        }
    );
    let mut long = encoded.to_vec();
    long.push(0);
    assert_eq!(
        WorldTeleportRequest::decode(&long).unwrap_err(),
        WarpCodecError::TrailingBytes {
            expected: 8,
            got: 9
        }
    );
}

#[test]
fn unknown_destination_is_rejected() {
    let registry = WarpRegistry::new(captured_destinations());
    let body = ZoneTeleportRequest {
        destination_id: 99_999,
    }
    .encode();
    assert_eq!(
        registry.handle_zone_teleport(&body, 0).unwrap_err(),
        WarpRejection::UnknownDestination(99_999)
    );
}

#[test]
fn locked_destination_is_rejected_until_attuned() {
    let mut registry = WarpRegistry::new(captured_destinations());
    let body = ZoneTeleportRequest { destination_id: 9 }.encode();
    assert!(!registry.is_attuned(9));
    assert_eq!(
        registry.handle_zone_teleport(&body, 1).unwrap_err(),
        WarpRejection::LockedDestination(9)
    );
    assert!(registry.attune(9), "first attunement reports the change");
    assert!(!registry.attune(9), "repeat attunement is a no-op");

    let reply = registry
        .handle_zone_teleport(&body, 1)
        .expect("attuned warpstone");
    let captured = CAPTURED_TELEPORT_SAMPLES
        .iter()
        .find(|sample| sample.capture == "p2.pcap" && sample.destination_id == 9)
        .expect("captured id 9 sample");
    assert_eq!(reply, unhex(captured.reply_message_hex));
}

#[test]
fn malformed_bodies_reach_the_policy_without_a_reply() {
    let registry = WarpRegistry::new(captured_destinations());
    assert_eq!(
        registry.handle_zone_teleport(&[9, 0, 0], 0).unwrap_err(),
        WarpRejection::MalformedRequest(WarpCodecError::Truncated {
            expected: 4,
            got: 3
        })
    );
}

#[test]
fn point_of_interest_reply_round_trips_and_rejects_a_truncated_record() {
    let records = vec![
        PointOfInterestDefinition {
            id: 9,
            name_id: 18899,
            location_id: 11293,
            x: -733.9340209960938,
            y: 9.086000442504883,
            z: 945.948974609375,
            heading: -2.673,
            icon_id: 1345,
            notification_type: 7,
            sub_name_id: 382845,
            unknown_11: 651,
            breadcrumb_quest_id: 0,
            teleport_location_id: 210025,
            atlas_name: "sacredgrove".to_string(),
            unknown_15: 60,
        },
        PointOfInterestDefinition {
            id: 15,
            name_id: 41393,
            location_id: 11299,
            x: -2124.768,
            y: -23.903,
            z: -292.377,
            heading: 1.694,
            icon_id: 1345,
            notification_type: 7,
            sub_name_id: 382845,
            unknown_11: 651,
            breadcrumb_quest_id: 0,
            teleport_location_id: 210030,
            atlas_name: "sacredgrove".to_string(),
            unknown_15: 30,
        },
    ];
    let body = encode_point_of_interest_reply(&records);
    assert_eq!(
        *body.last().unwrap(),
        0,
        "stream ends with the false terminator"
    );
    assert_eq!(decode_point_of_interest_reply(&body).unwrap(), records);

    let truncated = &body[..body.len() - 1];
    assert!(matches!(
        decode_point_of_interest_reply(truncated).unwrap_err(),
        WarpCodecError::MissingTerminator
    ));
    let cut_record = &body[..9];
    assert!(matches!(
        decode_point_of_interest_reply(cut_record).unwrap_err(),
        WarpCodecError::Truncated { .. }
    ));
}

#[test]
fn registry_rejects_a_destination_that_is_not_a_warpstone() {
    let registry = WarpRegistry::new(vec![WarpDestination {
        id: 131,
        is_warpstone: false,
        position: [128.81799, 22.525, 390.70901, 1.0],
        rotation: [-0.19572824, 0.0, 0.9806582, 0.0],
    }]);
    let body = ZoneTeleportRequest {
        destination_id: 131,
    }
    .encode();
    assert_eq!(
        registry.handle_zone_teleport(&body, 0).unwrap_err(),
        WarpRejection::NotAWarpstone(131)
    );
}
