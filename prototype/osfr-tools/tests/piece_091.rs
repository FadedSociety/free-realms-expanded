//! Builder tests for work item 091, "Pets and companions".
//!
//! Evidence boundary, read this before the assertions:
//!
//! ORIGINAL (first-party client inputs plus the recovered managed server serializer;
//! paths and hashes are pinned by the private recovery checker `piece_091.py` and the research note):
//! * Packet family: base opcode 53 (`FUN_00b50670` writes `param_1[1] = 0x35`,
//!   `param_1[2] = subtype`), 35 named subtypes. Sanctuary's reader
//!   (`53 => reader.Read<byte>() switch { ... }`) and the community `PetBasePackets`
//!   enum agree on the names and ids.
//! * Client request senders (Ghidra pseudocode, `lab/decompiled/client/FreeRealms/pseudocode.c`):
//!   `FUN_00b60080`/`FUN_00b601a0` build subtype 4 with a 32-bit pet id and an action
//!   byte (1 = summon, 0 = recall), `FUN_00b60610` builds subtype 4 with the action byte
//!   only, `FUN_00b60110` builds subtype 32 with two 32-bit values, `FUN_00b5fcf0`
//!   builds subtype 26 with four 32-bit floats and a flag byte, `FUN_00b60350`/
//!   `FUN_00b60420` build subtype 28 with one flag byte.
//! * Preserved pet state: the recovered managed serializer `UnknownPetStruct::Serialize`
//!   (community server decompile) plus the recorded `PacketSendSelfToClient.json`
//!   sample (28 pet records; the first is "Captain Zeus").
//! * Client surface: `Pet.Summon`/`Pet.Dismiss`/`Pet.GetActivePetId`/`Pet.RaiseNeed`/
//!   `Pet.ModeSwitch` and `PetsHandler:OnForcedDismiss`/`OnSummonFailed`/
//!   `LockUtilities`/`UnlockUtilities` in `decompiled/client-ui/ScriptsBase.lua`.
//!
//! RECONSTRUCTED (labelled at each call site): the companion session rules - one active
//! pet, ownership checked against the roster, recall limited to the active pet, utility
//! requests gated on a free pet, forced dismiss from the server side.
//!
//! NOT CLAIMED: follow distances, need decay rates, utility timers, trial timers, trick
//! effects, reward values and any native run. Those stay explicit gaps.
//!
//! The module is not registered in `src/lib.rs` (lead-owned). Including the exact source
//! file keeps this target compiling against the real artifact. Integration step for the
//! lead: register the module and drop this `#[path]` include.

#[path = "../src/restoration/piece_091.rs"]
#[allow(dead_code)]
mod piece_091;

use piece_091::{
    CompanionEvent, CompanionSession, PET_PACKET_BASE_OPCODE, PET_PACKET_SUBTYPES, PetError,
    PetProfileItem, PetRecord, PetRequest, PetSummonAction, PetTrickRecord, PetUtilityAction,
    SessionError, pet_packet_name,
};

fn captain_zeus() -> PetRecord {
    // Recorded sample: the first record of the preserved `PacketSendSelfToClient.json`
    // (28 pet records). Every value below is copied from that record; nothing is invented.
    PetRecord {
        unknown: 2,
        unknown_2: true,
        unknown_3: 652,
        food: 53.0,
        groom: 82.37982,
        exercise: 0.0,
        happiness: 2.909777,
        unknown_8: false,
        pet_tricks: vec![
            PetTrickRecord {
                trick_id: 17,
                fields: [17, 7532, 22714, 1, 0, 0, 0, 1],
                enabled: true,
            },
            PetTrickRecord {
                trick_id: 16,
                fields: [16, 7520, 22713, 1, 0, 0, 0, 5],
                enabled: true,
            },
        ],
        item_guids: vec![101, 102, 103, 104, 105],
        profile_items: vec![
            PetProfileItem {
                id: 7,
                unknown: 11,
                unknown_2: 12,
            },
            PetProfileItem {
                id: 8,
                unknown: 13,
                unknown_2: 14,
            },
        ],
        pet_name: "Captain Zeus".to_string(),
        unknown_10: 103980,
        unknown_11: 0,
        texture_alias: "lightmark-chocolate".to_string(),
        icon_id: 3211,
        unknown_14: false,
        unknown_15: 0,
        unknown_16: false,
        unknown_17: [17, 16, 15, 14],
        unknown_18: [0, 688, 0, 0, 352, 354, 129, 0],
    }
}

fn untrained_pet(id: i32) -> PetRecord {
    // Shape of the sample's untrained rows (for example record 7, id 12): no tricks,
    // empty name, `unknown_14` set, needs pinned at 100.
    PetRecord {
        unknown: id,
        unknown_2: false,
        unknown_3: 653,
        food: 100.0,
        groom: 100.0,
        exercise: 100.0,
        happiness: 100.0,
        unknown_8: false,
        pet_tricks: Vec::new(),
        item_guids: vec![201, 202],
        profile_items: Vec::new(),
        pet_name: String::new(),
        unknown_10: 4259,
        unknown_11: 0,
        texture_alias: String::new(),
        icon_id: 6088,
        unknown_14: true,
        unknown_15: 0,
        unknown_16: false,
        unknown_17: [0, 0, 0, 0],
        unknown_18: [0, 0, 0, 0, 0, 0, 0, 0],
    }
}

#[test]
fn recovered_packet_family_uses_the_client_base_opcode() {
    assert_eq!(PET_PACKET_BASE_OPCODE, 53);
    assert_eq!(PET_PACKET_SUBTYPES.len(), 35);
    assert_eq!(pet_packet_name(4), Some("PetSummonRecallPacket"));
    assert_eq!(pet_packet_name(5), Some("PetListPacket"));
    assert_eq!(pet_packet_name(9), Some("PetActivePacket"));
    assert_eq!(pet_packet_name(26), Some("PetCanGoLocationPacket"));
    assert_eq!(pet_packet_name(28), Some("PetInWaterPacket"));
    assert_eq!(pet_packet_name(32), Some("PetSummonByItemIdPacket"));
    assert_eq!(pet_packet_name(38), Some("PetPreviewRequestPacket"));
    // Subtype 27 and 30 are absent from both readers; they must not be invented.
    assert_eq!(pet_packet_name(27), None);
    assert_eq!(pet_packet_name(30), None);
}

#[test]
fn summon_and_recall_encode_the_observed_client_fields() {
    let summon = PetRequest::Summon { pet_id: 7 };
    assert_eq!(summon.subtype(), 4);
    assert_eq!(summon.encode(), vec![7, 0, 0, 0, 1]);

    let recall = PetRequest::Recall { pet_id: 7 };
    assert_eq!(recall.subtype(), 4);
    assert_eq!(recall.encode(), vec![7, 0, 0, 0, 0]);

    // `FUN_00b60610` (forced dismiss) writes the action byte and leaves the id field
    // unset, so the body carries the action byte only.
    let forced = PetRequest::ForcedDismiss;
    assert_eq!(forced.subtype(), 4);
    assert_eq!(forced.encode(), vec![0]);
}

#[test]
fn summon_by_item_and_follow_packets_encode_their_recovered_fields() {
    let by_item = PetRequest::SummonByItemId {
        pet_id: 12,
        item_id: 34567,
    };
    assert_eq!(by_item.subtype(), 32);
    assert_eq!(by_item.encode(), vec![12, 0, 0, 0, 0x07, 0x87, 0, 0]);

    let can_go = PetRequest::CanGoLocation {
        position: [1.0, 2.5, -3.0, 0.0],
        permitted: 1,
    };
    assert_eq!(can_go.subtype(), 26);
    let body = can_go.encode();
    assert_eq!(body.len(), 17);
    assert_eq!(&body[..4], &1.0f32.to_le_bytes());
    assert_eq!(&body[4..8], &2.5f32.to_le_bytes());
    assert_eq!(&body[8..12], &(-3.0f32).to_le_bytes());
    assert_eq!(&body[12..16], &0.0f32.to_le_bytes());
    assert_eq!(body[16], 1);

    let in_water = PetRequest::InWater { in_water: 1 };
    assert_eq!(in_water.subtype(), 28);
    assert_eq!(in_water.encode(), vec![1]);
}

#[test]
fn requests_round_trip_through_their_recovered_bodies() {
    let cases = [
        PetRequest::Summon { pet_id: 63 },
        PetRequest::Recall { pet_id: 1 },
        PetRequest::ForcedDismiss,
        PetRequest::SummonByItemId {
            pet_id: 3,
            item_id: 0,
        },
        PetRequest::CanGoLocation {
            position: [-0.5, 0.0, 12.0, 4.0],
            permitted: 0,
        },
        PetRequest::InWater { in_water: 0 },
    ];
    for request in cases {
        let body = request.encode();
        let decoded = PetRequest::decode(request.subtype(), &body).expect("decodes");
        assert_eq!(decoded, request);
    }
}

#[test]
fn request_decode_rejects_bad_bodies() {
    assert_eq!(
        PetRequest::decode(99, &[0, 0, 0, 0]).unwrap_err(),
        PetError::UnknownSubtype(99)
    );
    assert_eq!(
        PetRequest::decode(4, &[1, 0, 0, 0]).unwrap_err(),
        PetError::WrongLength {
            subtype: 4,
            expected: 5,
            got: 4
        }
    );
    assert_eq!(
        PetRequest::decode(4, &[1, 0, 0, 0, 9]).unwrap_err(),
        PetError::UnknownAction(9)
    );
    assert_eq!(
        PetRequest::decode(32, &[0; 4]).unwrap_err(),
        PetError::WrongLength {
            subtype: 32,
            expected: 8,
            got: 4
        }
    );
    assert_eq!(
        PetRequest::decode(26, &[0; 16]).unwrap_err(),
        PetError::WrongLength {
            subtype: 26,
            expected: 17,
            got: 16
        }
    );
    // Subtypes the client constructs are known, but no sender pins a body for them yet.
    assert_eq!(
        PetRequest::decode(12, &[0; 4]).unwrap_err(),
        PetError::LayoutNotRecovered(12)
    );
}

#[test]
fn summon_action_values_come_from_the_two_client_senders() {
    assert_eq!(PetSummonAction::Recall as u8, 0);
    assert_eq!(PetSummonAction::Summon as u8, 1);
}

#[test]
fn recorded_pet_record_round_trips_byte_exactly() {
    let record = captain_zeus();
    let encoded = record.encode();
    let (decoded, consumed) = PetRecord::decode(&encoded).expect("record decodes");
    assert_eq!(consumed, encoded.len());
    assert_eq!(decoded, record);

    // Layout pin against the recovered `UnknownPetStruct::Serialize` order:
    // scalars, trick count + 37-byte trick rows, guid count + guids, utility item
    // count + 12-byte rows, name, two ints, texture alias, icon and the two
    // trailing int arrays.
    assert_eq!(encoded.len(), 261);
    assert_eq!(&encoded[26..30], &2i32.to_le_bytes());
    assert_eq!(&encoded[30..34], &17i32.to_le_bytes());
    assert_eq!(encoded[66], 1); // trick 1 `Unknown9` flag
    assert_eq!(&encoded[104..108], &5i32.to_le_bytes());
    assert_eq!(&encoded[128..132], &2i32.to_le_bytes());
    assert_eq!(&encoded[156..160], &12i32.to_le_bytes()); // UTF-8 name length
    assert_eq!(&encoded[160..172], b"Captain Zeus");
    assert_eq!(&encoded[180..184], &19i32.to_le_bytes()); // texture alias length
    assert_eq!(&encoded[184..203], b"lightmark-chocolate");
}

#[test]
fn pet_record_decode_rejects_truncated_and_negative_counts() {
    let mut encoded = captain_zeus().encode();
    encoded.truncate(encoded.len() - 1);
    assert!(matches!(
        PetRecord::decode(&encoded).unwrap_err(),
        PetError::Truncated
    ));

    let mut negative = captain_zeus().encode();
    // The trick count sits at offset 26, after the scalar block (int, bool, int, four
    // floats, bool); a negative count can never be a valid record.
    negative[26] = 0xff;
    negative[27] = 0xff;
    negative[28] = 0xff;
    negative[29] = 0xff;
    assert!(matches!(
        PetRecord::decode(&negative).unwrap_err(),
        PetError::InvalidCount { .. }
    ));
}

#[test]
fn session_requires_an_owned_pet() {
    let mut session = CompanionSession::new(vec![captain_zeus()]);
    assert_eq!(
        session.summon(77, 0xdead_beef).unwrap_err(),
        SessionError::UnknownPet(77)
    );
    assert_eq!(session.active_pet(), None);
    assert_eq!(
        session.summon(2, 0).unwrap_err(),
        SessionError::InvalidGuid(2)
    );
}

#[test]
fn session_keeps_exactly_one_active_pet() {
    let mut session = CompanionSession::new(vec![captain_zeus(), untrained_pet(12)]);
    assert_eq!(
        session.summon(2, 0x01).unwrap(),
        CompanionEvent::Summoned {
            pet_id: 2,
            guid: 0x01
        }
    );
    assert_eq!(session.active_pet().map(|pet| pet.pet_id), Some(2));

    assert_eq!(
        session.summon(12, 0x02).unwrap(),
        CompanionEvent::Summoned {
            pet_id: 12,
            guid: 0x02
        }
    );
    assert_eq!(session.active_pet().map(|pet| pet.pet_id), Some(12));
    assert_eq!(session.active_pet().map(|pet| pet.guid), Some(0x02));

    // Re-summoning the pet that is already active is not a new summon.
    assert_eq!(
        session.summon(12, 0x02).unwrap(),
        CompanionEvent::AlreadyActive { pet_id: 12 }
    );
    assert_eq!(session.active_pet().map(|pet| pet.guid), Some(0x02));
}

#[test]
fn session_recall_only_accepts_the_active_pet() {
    let mut session = CompanionSession::new(vec![captain_zeus(), untrained_pet(12)]);
    session.summon(2, 0x01).unwrap();

    assert_eq!(session.recall(12).unwrap_err(), SessionError::NotActive(12));
    assert_eq!(
        session.recall(2).unwrap(),
        CompanionEvent::Recalled { pet_id: 2 }
    );
    assert_eq!(session.active_pet(), None);
    assert_eq!(session.recall(2).unwrap_err(), SessionError::NoActivePet);
}

#[test]
fn forced_dismiss_clears_the_active_pet_without_an_id() {
    let mut session = CompanionSession::new(vec![captain_zeus()]);
    assert_eq!(
        session.forced_dismiss().unwrap_err(),
        SessionError::NoActivePet
    );
    session.summon(2, 0x01).unwrap();
    assert_eq!(
        session.forced_dismiss().unwrap(),
        CompanionEvent::ForcedDismiss
    );
    assert_eq!(session.active_pet(), None);
}

#[test]
fn utility_and_follow_requests_need_an_active_pet() {
    let mut session = CompanionSession::new(vec![captain_zeus()]);
    assert_eq!(
        session
            .apply(PetRequest::InWater { in_water: 1 }, 0x01)
            .unwrap_err(),
        SessionError::NoActivePet
    );
    assert_eq!(
        session
            .apply(
                PetRequest::CanGoLocation {
                    position: [0.0; 4],
                    permitted: 1,
                },
                0x01,
            )
            .unwrap_err(),
        SessionError::NoActivePet
    );
    assert_eq!(
        session.interact(PetUtilityAction::Feed).unwrap_err(),
        SessionError::NoActivePet
    );

    session.summon(2, 0x01).unwrap();
    assert_eq!(
        session
            .apply(PetRequest::InWater { in_water: 1 }, 0x01)
            .unwrap(),
        CompanionEvent::FollowStateChanged { pet_id: 2 }
    );
    assert_eq!(session.follow_state().map(|state| state.in_water), Some(1));

    // A request carrying a different guid than the active pet is refused.
    assert_eq!(
        session
            .apply(PetRequest::InWater { in_water: 1 }, 0x02)
            .unwrap_err(),
        SessionError::GuidMismatch {
            active: 0x01,
            got: 0x02
        }
    );

    session.set_busy(true);
    assert_eq!(
        session.interact(PetUtilityAction::Groom).unwrap_err(),
        SessionError::PetBusy
    );
    session.set_busy(false);
    assert_eq!(
        session.interact(PetUtilityAction::Groom).unwrap(),
        CompanionEvent::UtilityRequested {
            pet_id: 2,
            utility: PetUtilityAction::Groom
        }
    );
}

#[test]
fn summon_by_item_is_accepted_only_for_an_owned_pet() {
    let mut session = CompanionSession::new(vec![untrained_pet(12)]);
    let request = PetRequest::SummonByItemId {
        pet_id: 12,
        item_id: 4242,
    };
    assert_eq!(
        session.apply(request, 0x05).unwrap(),
        CompanionEvent::Summoned {
            pet_id: 12,
            guid: 0x05
        }
    );
    let unknown = PetRequest::SummonByItemId {
        pet_id: 99,
        item_id: 4242,
    };
    assert_eq!(
        session.apply(unknown, 0x06).unwrap_err(),
        SessionError::UnknownPet(99)
    );
}

#[test]
fn roster_snapshot_round_trips_every_owned_pet() {
    let roster = vec![captain_zeus(), untrained_pet(12), untrained_pet(13)];
    let session = CompanionSession::new(roster.clone());
    assert_eq!(session.roster_len(), 3);
    assert_eq!(session.roster(), roster.as_slice());
    assert_eq!(session.pet_ids(), vec![2, 12, 13]);
    // Duplicate ids are refused: the roster is an ownership set.
    assert_eq!(
        CompanionSession::new(vec![captain_zeus(), captain_zeus()]).roster_len(),
        1
    );
}
