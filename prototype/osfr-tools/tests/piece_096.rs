//! Work item 096 (typed world initialization) acceptance tests.
//!
//! Evidence boundary, read before the assertions:
//!
//! ORIGINAL (recovered first-party emulator source, kept outside Git):
//! * `lab/decompiled/server/Gateway/Gateway.Login/LoginManager.cs`:
//!   `HandleLoginRequest` sends, in order, `PlayerCode.SendEncounterOverworldCombat`,
//!   `SendLoginReply`, `SendInitializationParameters`, `SendZoneDetails`,
//!   `SendClientGameSettings`, `PlayerCode.SendPlayerUpdateItemDefinitions`,
//!   `SendAnnouncementData`, `PlayerCode.SendSelfToClient`. `HandlePacketClientIsReady`
//!   ends with `SendPacketLoadWelcomeScreen`, `SendPacketZoneDoneSendingInitialData`,
//!   `PlayerCode.SendClientUpdatePacketDoneSendingPreloadCharacters`,
//!   `UpdateStats.SendUpdatePacketUpdateStat`, `SendPacketMOTD`.
//!   `SendTunneledClientPacket` writes outer opcode 5, boolean `true`, a little-endian
//!   `u32` payload length and the payload.
//! * `lab/decompiled/server/Gateway/Gateway.Player/PlayerCode.cs`: the writer call
//!   order for `SendPlayerUpdateItemDefinitions`, `SendSelfToClient`,
//!   `SendClientUpdatePacketDoneSendingPreloadCharacters` and
//!   `SendEncounterOverworldCombat`.
//! * `lab/decompiled/server/Gateway/Gateway.Player/UpdateStats.cs`:
//!   `SendUpdatePacketUpdateStat` writes sub-opcode 7, the player GUID, an `i32`
//!   count and that many `(i32, i32, f32)` entries.
//! * `lab/references/rust-gateway-reference/` (46 preserved emulator messages and
//!   their manifest) is the unchanged-writer oracle used by the ignored test below.
//!
//! RECONSTRUCTED / NOT CLAIMED: the Rust field names for values the emulator writes as
//! literals, the meaning of the sub-opcode numbers, and the per-entity attribution of
//! the still-captured `op35` entity-add block. Nothing here claims native acceptance;
//! the unchanged-writer oracle needs the private reference or the test reports a skip.
//!
//! This file is a `#[path]` include so it exercises the real module before the lead
//! registers `pub mod piece_096;` under `src/restoration/mod.rs`/`src/lib.rs`.

#[path = "../src/restoration/piece_096.rs"]
#[allow(dead_code)]
mod piece_096;

use piece_096::{
    AnnouncementData, AnnouncementRow, ClientGameSettings, DoneSendingPreloadCharacters,
    EncounterOverworldCombat, INIT_SLOT_COUNT, INIT_SLOTS, InitError, InitMessage,
    InitializationParameters, ItemDefinition, ItemDefinitions, ItemStatDefinition, LoginReply,
    Motd, SelfToClient, TypedWriter, UpdateStat, UpdateStatEntry, ZoneDetails,
    ZoneDoneSendingInitialData, decode, decode_payload, verify_captured, writer_for,
};
use std::fs;

/// The captured login/ready opcode sequence, in order. Only the opcodes are asserted
/// here; the writer attribution is asserted separately per typed slot.
const CAPTURED_OPCODES: [u16; INIT_SLOT_COUNT] = [
    41, 2, 166, 43, 144, 35, 193, 12, // login phase
    67, 57, // ready phase: quick chat, points of interest
    35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 38,
    38, 36, 38, 38, 209, 209, 165, 93, // hitpoints, mana, action bar, effects, literals
    14, 38, 38, 87,
];

const TYPED_SLOTS: [usize; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 42, 43, 44, 45];

fn zone_details() -> ZoneDetails {
    ZoneDetails {
        name: "FabledRealms".to_string(),
        zone_type: 2,
        tutorial: false,
        unknown2: false,
        sky: String::new(),
        is_in_arena: false,
        id: 5,
        geometry_id: 5,
        is_in_starting_social_zone: false,
        unknown6: false,
    }
}

fn item_definition(id: i32) -> ItemDefinition {
    ItemDefinition {
        id,
        name_id: 414328,
        description_id: 414330,
        icon_id: 4313,
        icon_tint_id: 262,
        tint: 0,
        unknown5: 0,
        cost: 50,
        class: 1,
        profile_override: 0,
        slot: 4,
        no_trade: true,
        no_sale: false,
        model_name: "freestyle_<race>_<gender>_legs_bootcutpants.adr".to_string(),
        texture_alias: "adventurer-gothpants-P".to_string(),
        gender_usage: 1,
        item_type: 104,
        category_id: 0,
        members_only: false,
        non_minigame: false,
        weapon_trail_effect_id: 0,
        composite_effect_id: 0,
        power_rating: 0,
        min_profile_rank: 1,
        rarity: 2,
        activatable_ability_id: 0,
        passive_ability_id: 0,
        single_use: false,
        max_stack_size: -1,
        is_tintable: false,
        tint_alias: "dyetint".to_string(),
        force_disable_preview: false,
        member_discount: 10,
        vip_rank_required: 0,
        race_set_id: 0,
        client_equip_req_set_id: 0,
        resell_value: 5,
        unknown36: 0,
        unknown37: 0,
        unknown38: 0,
        stats: vec![ItemStatDefinition {
            key: 1,
            unknown: 2,
            unknown2: 3,
            unknown3: 4,
            unknown4: true,
        }],
        abilities: Vec::new(),
    }
}

fn typed_messages() -> Vec<InitMessage> {
    vec![
        InitMessage::EncounterOverworldCombat(EncounterOverworldCombat {
            encounter_id: 132,
            unknown_a: 0,
            unknown_b: 0,
            enabled: true,
        }),
        InitMessage::LoginReply(LoginReply { accepted: true }),
        InitMessage::InitializationParameters(InitializationParameters {
            environment: "live".to_string(),
        }),
        InitMessage::ZoneDetails(zone_details()),
        InitMessage::ClientGameSettings(ClientGameSettings {
            values: [4, 7, 268],
            enabled: true,
            scale: 1.0,
        }),
        InitMessage::ItemDefinitions(ItemDefinitions {
            inner_opcode: 37,
            items: vec![item_definition(68878)],
        }),
        InitMessage::AnnouncementData(AnnouncementData {
            revision: 2,
            rows: vec![AnnouncementRow {
                numbers: [139, 1, 47736, 5103267, 0, 1911],
                action: "Exit".to_string(),
                parameter: "Param".to_string(),
                tail: [0, 0, 0],
                extra: String::new(),
            }],
        }),
        InitMessage::SelfToClient(SelfToClient {
            payload: vec![1, 2, 3, 4],
        }),
        InitMessage::ZoneDoneSendingInitialData(ZoneDoneSendingInitialData),
        InitMessage::DoneSendingPreloadCharacters(DoneSendingPreloadCharacters {
            sub_opcode: 26,
            flag: false,
        }),
        InitMessage::UpdateStat(UpdateStat {
            sub_opcode: 7,
            guid: 1,
            entries: vec![
                UpdateStatEntry {
                    stat_id: 2,
                    unknown: 1,
                    value: 8.0,
                },
                UpdateStatEntry {
                    stat_id: 4,
                    unknown: 0,
                    value: 1.0,
                },
            ],
        }),
        InitMessage::Motd(Motd {
            title: "Welcome".to_string(),
            body: "test build".to_string(),
        }),
    ]
}

#[test]
fn slot_table_matches_the_captured_sequence() {
    assert_eq!(INIT_SLOTS.len(), INIT_SLOT_COUNT);
    for (slot, expected) in CAPTURED_OPCODES.iter().enumerate() {
        assert_eq!(
            INIT_SLOTS[slot].opcode, *expected,
            "slot {slot} opcode drifted from the captured sequence"
        );
        assert_eq!(
            writer_for(slot).is_some(),
            TYPED_SLOTS.contains(&slot),
            "slot {slot} typed/captured classification drifted"
        );
    }
}

#[test]
fn typed_writers_cover_the_login_phase_and_the_ready_tail() {
    for slot in TYPED_SLOTS {
        let writer = writer_for(slot).expect("typed slot");
        assert_eq!(writer.slot(), slot);
        assert_eq!(writer.opcode(), CAPTURED_OPCODES[slot]);
        // Every typed opcode must be unique except for the op38/op35 families, which
        // are disambiguated by slot. Unique opcodes must resolve back to one slot.
        if ![35, 38].contains(&writer.opcode()) {
            assert_eq!(
                TypedWriter::for_opcode(writer.opcode()),
                Some(writer),
                "{} is not reachable by opcode",
                writer.name()
            );
        }
    }
    assert_eq!(
        TypedWriter::for_opcode(35),
        None,
        "opcode 35 needs the slot to disambiguate item definitions from entity adds"
    );
}

#[test]
fn typed_messages_round_trip_byte_identically() {
    for message in typed_messages() {
        let frame = message.encode();
        assert_eq!(frame[0], 5);
        assert_eq!(frame[2], 1);
        assert_eq!(
            u32::from_le_bytes(frame[3..7].try_into().unwrap()) as usize,
            frame.len() - 7,
            "{} declared the wrong payload length",
            message.writer().name()
        );
        let decoded = decode(message.writer().slot(), &frame).expect("decodes");
        assert_eq!(decoded, message, "{} round trip", message.writer().name());
        assert_eq!(decoded.encode(), frame, "re-encode is stable");
        assert_eq!(
            decoded.encode_payload()[..2],
            message.opcode().to_le_bytes()
        );
    }
}

#[test]
fn payload_and_frame_encoders_agree() {
    for message in typed_messages() {
        let frame = message.encode();
        let payload = message.encode_payload();
        assert_eq!(&frame[7..], payload.as_slice());
        assert_eq!(
            decode_payload(message.writer().slot(), &payload).unwrap(),
            message
        );
    }
}

#[test]
fn update_stat_entries_are_count_prefixed() {
    // Recovered order: sub-opcode, GUID, i32 count, then (i32, i32, f32) entries.
    let payload = typed_messages()
        .into_iter()
        .find(|m| matches!(m, InitMessage::UpdateStat(_)))
        .unwrap()
        .encode_payload();
    let entries = 2u32.to_le_bytes();
    let guid = 1i64.to_le_bytes();
    assert_eq!(&payload[..2], &[38, 0]);
    assert_eq!(&payload[2..4], &[7, 0]);
    assert_eq!(&payload[4..12], &guid);
    assert_eq!(&payload[12..16], &entries);
    assert_eq!(payload.len(), 16 + 2 * 12);
}

#[test]
fn item_definition_rows_are_nested_and_counted() {
    let payload = typed_messages()
        .into_iter()
        .find(|m| matches!(m, InitMessage::ItemDefinitions(_)))
        .unwrap()
        .encode_payload();
    assert_eq!(&payload[..2], &[35, 0]);
    assert_eq!(
        &payload[2..4],
        &[37, 0],
        "inner opcode 37 from the recovered writer"
    );
    let inner = u32::from_le_bytes(payload[4..8].try_into().unwrap()) as usize;
    assert_eq!(inner, payload.len() - 8);
    assert_eq!(&payload[8..12], &1i32.to_le_bytes(), "row count");
}

#[test]
fn malformed_frames_are_rejected() {
    let frame = InitMessage::LoginReply(LoginReply { accepted: true }).encode();

    let mut wrong_outer = frame.clone();
    wrong_outer[0] = 6;
    assert_eq!(decode(1, &wrong_outer), Err(InitError::TunnelOuter(6)));

    let mut wrong_length = frame.clone();
    wrong_length[3] += 1;
    assert_eq!(
        decode(1, &wrong_length),
        Err(InitError::LengthMismatch {
            declared: 4,
            actual: 3
        })
    );

    // Extra payload bytes are a body-level error because the tunnel length stays honest.
    assert_eq!(
        decode_payload(1, &[2, 0, 1, 0]),
        Err(InitError::TrailingBytes(1))
    );

    assert_eq!(decode(1, &frame[..6]), Err(InitError::Truncated("tunnel")));
    assert_eq!(
        decode(0, &frame),
        Err(InitError::OpcodeMismatch {
            expected: 41,
            actual: 2
        })
    );
    assert_eq!(decode(8, &frame), Err(InitError::NotTypedSlot(8)));
    assert_eq!(decode(46, &frame), Err(InitError::NotTypedSlot(46)));

    // A non-boolean boolean byte inside a typed body must be rejected, not coerced.
    let mut bad_bool = InitMessage::LoginReply(LoginReply { accepted: true }).encode();
    bad_bool[9] = 2;
    assert_eq!(
        decode(1, &bad_bool),
        Err(InitError::InvalidBoolean {
            field: "login reply",
            value: 2
        })
    );
}

#[test]
fn oversized_counts_and_text_are_rejected() {
    let mut items = InitMessage::ItemDefinitions(ItemDefinitions {
        inner_opcode: 37,
        items: Vec::new(),
    })
    .encode_payload();
    let declared = (1_000_000i32).to_le_bytes();
    items[8..12].copy_from_slice(&declared);
    assert!(matches!(
        decode_payload(5, &items),
        Err(InitError::TooManyRows { .. })
    ));

    let environment = InitMessage::InitializationParameters(InitializationParameters {
        environment: "live".to_string(),
    })
    .encode_payload();
    // Truncated length prefix: the payload ends in the middle of the environment length.
    assert!(matches!(
        decode_payload(2, &environment[..5]),
        Err(InitError::Truncated("initialization environment"))
    ));

    let reply = InitMessage::Motd(Motd {
        title: "a".repeat(64 * 1024 + 1),
        body: String::new(),
    });
    assert!(matches!(
        decode_payload(45, &reply.encode_payload()),
        Err(InitError::TextTooLong {
            field: "motd title",
            ..
        })
    ));
}

#[test]
fn verify_captured_reports_a_byte_mismatch() {
    let frame = InitMessage::ZoneDetails(zone_details()).encode();
    let vector = verify_captured(3, &frame).expect("unchanged writer vector");
    assert_eq!(vector.slot, 3);
    assert_eq!(vector.writer, TypedWriter::ZoneDetails);
    assert_eq!(vector.length, frame.len());

    // The tunnel flag is a valid boolean either way, but the writer always emits `true`,
    // so a preserved message flagged `false` must fail verification instead of passing.
    let mut tampered = frame.clone();
    tampered[2] = 0;
    assert_eq!(
        verify_captured(3, &tampered),
        Err(InitError::ByteMismatch { offset: 2 })
    );
    assert_eq!(verify_captured(8, &frame), Err(InitError::NotTypedSlot(8)));
}

/// Unchanged-writer oracle. Set `OSFR_GATEWAY_REFERENCE` to the preserved private
/// reference directory and run with `--ignored`; without the variable the test reports
/// a skip instead of inventing evidence.
#[test]
#[ignore = "requires the preserved private gateway reference; set OSFR_GATEWAY_REFERENCE"]
fn captured_templates_re_encode_byte_identically() {
    let Ok(directory) = std::env::var("OSFR_GATEWAY_REFERENCE") else {
        eprintln!("skipped: OSFR_GATEWAY_REFERENCE is not set");
        return;
    };
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(std::path::Path::new(&directory).join("manifest.json")).unwrap(),
    )
    .unwrap();
    let templates = manifest["templates"].as_array().expect("templates");
    assert_eq!(templates.len(), INIT_SLOT_COUNT);

    let mut typed = 0usize;
    let mut captured = 0usize;
    for (slot, entry) in templates.iter().enumerate() {
        let file = std::path::Path::new(&directory).join(entry["file"].as_str().unwrap());
        let bytes = fs::read(&file).unwrap();
        assert_eq!(
            bytes.len(),
            entry["length"].as_u64().unwrap() as usize,
            "slot {slot} length drifted"
        );
        assert_eq!(
            u16::from_le_bytes(bytes[7..9].try_into().unwrap()),
            entry["opcode"].as_u64().unwrap() as u16,
            "slot {slot} opcode drifted from the manifest"
        );
        assert_eq!(
            INIT_SLOTS[slot].opcode,
            entry["opcode"].as_u64().unwrap() as u16
        );
        match writer_for(slot) {
            Some(writer) => {
                let vector = verify_captured(slot, &bytes).unwrap_or_else(|error| {
                    panic!(
                        "slot {slot} ({}) is not an unchanged writer vector: {error}",
                        writer.name()
                    )
                });
                assert_eq!(vector.opcode, entry["opcode"].as_u64().unwrap() as u16);
                typed += 1;
            }
            None => {
                assert_eq!(bytes[0], 5);
                captured += 1;
            }
        }
    }
    assert_eq!(typed, TYPED_SLOTS.len());
    assert_eq!(pending_captured_slots().len(), captured);
    eprintln!("unchanged writer vectors: {typed} typed, {captured} still captured");
}

fn pending_captured_slots() -> Vec<usize> {
    (0..INIT_SLOT_COUNT)
        .filter(|slot| writer_for(*slot).is_none())
        .collect()
}
