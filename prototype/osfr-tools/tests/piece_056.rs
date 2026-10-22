//! Builder tests for work item 056, "Combat status effects".
//!
//! ORIGINAL evidence the vectors below are pinned to (all outside this test):
//! * `lab/decompiled/server/Gateway/Gateway.Login/ClientPcData.cs` - the recovered
//!   `EffectTag`/`ClientEffectTag` member order (72 + 13 bytes) and the `EffectTags` list inside the
//!   self-to-client payload, written as `count` then `(int TagId, ClientEffectTag)` per row.
//! * `lab/decompiled/server/LibSOE/SOE/BaseClientUpdatePackets.cs` - `AddEffectTag = 16`,
//!   `RemoveEffectTag = 17`; `BasePlayerUpdatePackets.cs` - `AddEffectTagCompositeEffect = 41`,
//!   `RemoveEffectTagCompositeEffect = 42`, `EffectTagCompositeEffectsEnable = 43` and
//!   `UpdateCharacterState = 20`; `BasePackets.cs` - `BasePlayerUpdatePacket = 35`,
//!   `BaseClientUpdatePacket = 38`.
//! * The original client constructor call `FUN_0089fbb0(0x29)` immediately before
//!   `PlayerUpdatePacketAddEffectTagCompositeEffect::vftable` (and `0x2a`/`0x2b`/`0x14` for the
//!   Remove/Enable/UpdateCharacterState classes) proves those numbers are the *sub* opcodes of the
//!   35 family; `0x23` is `PlayerUpdatePacketHitPointModification`, matching enum value 35.
//! * `lab/working/OSFR.Server/OSFR Server/Customize/PacketSendSelfToClient.json` - the captured
//!   retail character carried exactly one effect tag: TagId 2, duration 3600, icon 7334, name 4270,
//!   definition 495, start/stop 142556/3978, composite effect 0. Icon 7334 is the image of original
//!   stat 37 "Coin Boost" (`CharacterStatDefinitions.txt`, NameId 140800) and 4270 resolves to
//!   "Power Hour Coin Boost" in the recovered locale export.
//! * `lab/decompiled/client-ui/ScriptsBase.lua` - `GroupHandler.BuffsDsName =
//!   "BaseClient.PlayerEffectTags"`, buff rows read as instance, icon, duration, timeLeft, name, and
//!   `AddAbilityEffect` returns early when the icon is 0.
//!
//! RECONSTRUCTED policy (labelled, not historical): tag-id allocation, the refresh-on-same
//! (definition, source) stacking rule, the immunity categories, and the use of the effect tag's
//! duration as the remaining time on restore. Nothing here claims native or gameplay acceptance.
//!
//! The module is not registered in `src/lib.rs` (a shared file owned by the lead), so this target
//! compiles the exact source with `#[path]`, as the other piece-NNN targets do.

#[path = "../src/restoration/piece_056.rs"]
#[allow(dead_code)]
mod piece_056;

use osfr_tools::startup_catalogues::{self, EffectTag};
use piece_056::{
    ADD_COMPOSITE_EFFECT_LEN, ADD_EFFECT_TAG_LEN, ActiveEffect, CHARACTER_STATE_LEN,
    CLIENT_UPDATE_OPCODE, CLIENT_UPDATE_SUB_ADD_EFFECT_TAG, CLIENT_UPDATE_SUB_REMOVE_EFFECT_TAG,
    EffectRequest, EffectTagPayload, ImmunitySet, MAX_EFFECT_SECONDS, PLAYER_UPDATE_OPCODE,
    PLAYER_UPDATE_SUB_ADD_COMPOSITE_EFFECT, PLAYER_UPDATE_SUB_REMOVE_COMPOSITE_EFFECT,
    PLAYER_UPDATE_SUB_UPDATE_CHARACTER_STATE, REMOVE_COMPOSITE_EFFECT_LEN, REMOVE_EFFECT_TAG_LEN,
    StatusError, StatusKind, StatusLedger, decode_add_composite_effect, decode_add_effect_tag,
    decode_character_state, decode_remove_composite_effect, decode_remove_effect_tag,
    encode_add_composite_effect, encode_add_effect_tag, encode_character_state,
    encode_profile_rows, encode_remove_composite_effect, encode_remove_effect_tag,
};

const SOURCE: u64 = 0x5255_5354_0000_0002;

fn coin_boost_request() -> EffectRequest {
    EffectRequest {
        kind: StatusKind::Status,
        definition_id: 495,
        source_guid: SOURCE,
        duration_seconds: 3600,
        icon_id: 7334,
        name_id: 4270,
        composite_effect_id: 0,
    }
}

fn stun_request(duration_seconds: i32) -> EffectRequest {
    EffectRequest {
        kind: StatusKind::Stun,
        definition_id: 9001,
        source_guid: SOURCE,
        duration_seconds,
        icon_id: 7927,
        name_id: 4252,
        composite_effect_id: 0,
    }
}

#[test]
fn recovered_packet_ids_and_sizes() {
    assert_eq!(CLIENT_UPDATE_OPCODE, 38);
    assert_eq!(CLIENT_UPDATE_SUB_ADD_EFFECT_TAG, 16);
    assert_eq!(CLIENT_UPDATE_SUB_REMOVE_EFFECT_TAG, 17);
    assert_eq!(PLAYER_UPDATE_OPCODE, 35);
    assert_eq!(PLAYER_UPDATE_SUB_ADD_COMPOSITE_EFFECT, 41);
    assert_eq!(PLAYER_UPDATE_SUB_REMOVE_COMPOSITE_EFFECT, 42);
    assert_eq!(PLAYER_UPDATE_SUB_UPDATE_CHARACTER_STATE, 20);

    // 4-byte header + TagId + i32 blob length + the 85-byte ClientEffectTag (72-byte EffectTag +
    // 4 extra fields); client reader 0x008d8870 -> 0x008bfc00.
    assert_eq!(ADD_EFFECT_TAG_LEN, 4 + 4 + 4 + 85);
    assert_eq!(REMOVE_EFFECT_TAG_LEN, 4 + 4);
    assert_eq!(ADD_COMPOSITE_EFFECT_LEN, 4 + 8 + 4 + 4 + 8 + 8);
    assert_eq!(REMOVE_COMPOSITE_EFFECT_LEN, 4 + 8 + 4);
    assert_eq!(CHARACTER_STATE_LEN, 4 + 8 + 4);
}

#[test]
fn captured_retail_coin_boost_round_trips_byte_for_byte() {
    let tag = EffectTagPayload::captured_coin_boost();
    assert_eq!(tag.duration_seconds(), 3600);
    assert_eq!(tag.icon_id(), 7334);
    assert_eq!(tag.name_id(), 4270);
    assert_eq!(tag.unknown_2, 495);
    assert_eq!(tag.unknown_10, 142556);
    assert_eq!(tag.unknown_11, 3978);
    assert_eq!(tag.composite_effect_id, 0);
    assert!(tag.unknown_18 && tag.unknown_19 && !tag.unknown_8);

    let bytes = encode_add_effect_tag(2, &tag);
    assert_eq!(bytes.len(), ADD_EFFECT_TAG_LEN);
    assert_eq!(&bytes[..4], &[38, 0, 16, 0]);
    assert_eq!(&bytes[4..8], &2_i32.to_le_bytes());

    let (tag_id, decoded) = decode_add_effect_tag(&bytes).expect("round trip");
    assert_eq!(tag_id, 2);
    assert_eq!(decoded, tag);

    let remove = encode_remove_effect_tag(2);
    assert_eq!(remove.len(), REMOVE_EFFECT_TAG_LEN);
    assert_eq!(&remove[..4], &[38, 0, 17, 0]);
    assert_eq!(decode_remove_effect_tag(&remove).expect("remove"), 2);
}

#[test]
fn captured_profile_row_matches_the_recovered_self_payload_shape() {
    let row = EffectTagPayload::captured_coin_boost();
    let bytes = encode_profile_rows(&[(2, row)]);
    // The original ClientPcData serializes `count` then (int TagId, ClientEffectTag) per row.
    assert_eq!(bytes.len(), 4 + 4 + 85);
    assert_eq!(&bytes[..4], &1_i32.to_le_bytes());
    assert_eq!(&bytes[4..8], &2_i32.to_le_bytes());
    assert_eq!(&bytes[8..], &encode_add_effect_tag(2, &row)[12..]);
}

/// The two 38/16 packets of the verified gateway startup reference
/// (`lab/bundles/playtest-20261016T115811Z/reference/0036-ready.bin`, `0037-ready.bin`, after the
/// 7-byte tunnel prefix). Tag 4 is also the literal the supplied server's `UpdateStats.cs` sends.
const CAPTURED_ADD_EFFECT_TAGS: [&str; 2] = [
    "260010000200000055000000\
     02000000ef0100000200000000000000000000000000c842100e00000091ca73956acc524b0000000000000000\
     000000000000000000000000000000000000000000000000000101a61c000000ae1000000c010000",
    "260010000400000055000000\
     04000000350b000008000000000000000000000000000000100e00000091ca73956acc524b0000000000000000\
     01000000a20f000000000000000000000000000000000000000001272d00000054b80600db070000",
];

#[test]
fn captured_startup_tags_match_the_codec_and_the_startup_writer() {
    for hex in CAPTURED_ADD_EFFECT_TAGS {
        let packet: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).unwrap())
            .collect();
        let (tag_id, tag) = decode_add_effect_tag(&packet).expect("captured row");
        // Both retail rows carry their tag id as the instance id.
        assert_eq!(tag.unknown_1, tag_id);
        assert_eq!(encode_add_effect_tag(tag_id, &tag), packet);
        let startup = startup_catalogues::build_effect_tag(EffectTag {
            id: tag_id,
            definition_blob: &packet[12..],
        });
        assert_eq!(&startup[7..], &packet[..]);
    }
}

#[test]
fn world_effect_packets_round_trip() {
    let add = encode_add_composite_effect(SOURCE, 77, 5030, SOURCE);
    assert_eq!(add.len(), ADD_COMPOSITE_EFFECT_LEN);
    assert_eq!(&add[..4], &[35, 0, 41, 0]);
    let decoded = decode_add_composite_effect(&add).expect("add composite");
    assert_eq!(decoded.guid, SOURCE);
    assert_eq!(decoded.tag_id, 77);
    assert_eq!(decoded.composite_effect_id, 5030);
    assert_eq!(decoded.source_guid, SOURCE);

    let remove = encode_remove_composite_effect(SOURCE, 77);
    assert_eq!(remove.len(), REMOVE_COMPOSITE_EFFECT_LEN);
    assert_eq!(&remove[..4], &[35, 0, 42, 0]);
    assert_eq!(
        decode_remove_composite_effect(&remove).expect("remove composite"),
        (SOURCE, 77)
    );
}

#[test]
fn character_state_flags_carry_the_recorded_bits() {
    let state = encode_character_state(SOURCE, StatusKind::Stun.character_state_bit().unwrap());
    assert_eq!(state.len(), CHARACTER_STATE_LEN);
    assert_eq!(&state[..4], &[35, 0, 20, 0]);
    assert_eq!(
        decode_character_state(&state).expect("state"),
        (SOURCE, 0x40)
    );

    assert_eq!(StatusKind::Sleep.character_state_bit(), Some(0x4));
    assert_eq!(StatusKind::Silence.character_state_bit(), Some(0x8));
    assert_eq!(StatusKind::Root.character_state_bit(), Some(0x20));
    assert_eq!(StatusKind::Stun.character_state_bit(), Some(0x40));
    assert_eq!(StatusKind::Knockback.character_state_bit(), Some(0x200));
    assert_eq!(StatusKind::Confuse.character_state_bit(), Some(0x2000));
    assert_eq!(StatusKind::Fear.character_state_bit(), Some(0x2));
    assert_eq!(StatusKind::Slow.character_state_bit(), None);
    assert_eq!(StatusKind::Poison.character_state_bit(), None);
}

#[test]
fn immunity_kinds_match_their_original_locale_ids() {
    assert_eq!(StatusKind::Stun.immune_locale_id(), 70204);
    assert_eq!(StatusKind::Slow.immune_locale_id(), 70209);
    assert_eq!(StatusKind::Poison.immune_locale_id(), 70214);
    assert_eq!(StatusKind::Root.immune_locale_id(), 438557);
    assert_eq!(StatusKind::Knockback.immune_locale_id(), 438016);
    assert_eq!(StatusKind::Status.immune_locale_id(), 438559);
    assert_eq!(StatusKind::Illusion.immune_locale_id(), 436622);
    assert_eq!(StatusKind::Fear.immune_locale_id(), 436675);
    assert_eq!(StatusKind::ForcedEffect.immune_locale_id(), 436620);
}

#[test]
fn a_status_row_is_sent_with_the_recovered_buff_fields() {
    let mut ledger = StatusLedger::new();
    let applied = ledger
        .apply(coin_boost_request(), 0, &ImmunitySet::none())
        .expect("coin boost");
    assert_eq!(applied.tag_id, 1);
    assert!(!applied.refreshed);
    assert_eq!(applied.packets.len(), 1);
    let (tag_id, tag) = decode_add_effect_tag(&applied.packets[0]).expect("buff packet");
    assert_eq!(tag_id, applied.tag_id);
    assert_eq!(tag.duration_seconds(), 3600);
    assert_eq!(tag.icon_id(), 7334);
    assert_eq!(tag.name_id(), 4270);
    assert_eq!(ledger.len(), 1);
}

#[test]
fn a_zero_icon_effect_keeps_state_without_a_buff_row() {
    let mut ledger = StatusLedger::new();
    let mut request = stun_request(4);
    request.icon_id = 0;
    request.composite_effect_id = 5030;
    let applied = ledger
        .apply(request, 0, &ImmunitySet::none())
        .expect("world effect");
    // The client skips a zero icon, so no buff row is written for it; the world effect is separate.
    assert_eq!(applied.packets.len(), 1);
    assert_eq!(&applied.packets[0][..4], &[35, 0, 41, 0]);
    assert_eq!(ledger.len(), 1);
}

#[test]
fn tick_expires_the_row_exactly_once() {
    let mut ledger = StatusLedger::new();
    ledger
        .apply(stun_request(3600), 0, &ImmunitySet::none())
        .expect("stun");

    assert!(ledger.tick(1_000).is_empty());
    assert_eq!(ledger.len(), 1);

    let expired = ledger.tick(3_600_000);
    assert_eq!(expired.len(), 1);
    assert_eq!(expired[0].tag_id, 1);
    assert_eq!(expired[0].kind, StatusKind::Stun);
    assert_eq!(expired[0].packets.len(), 1);
    assert_eq!(&expired[0].packets[0][..4], &[38, 0, 17, 0]);
    assert_eq!(ledger.len(), 0);
    assert!(ledger.tick(9_999_999).is_empty());
}

#[test]
fn an_immune_effect_is_refused_without_spending_a_tag_id() {
    let immune = ImmunitySet::none().with(StatusKind::Stun);
    assert!(immune.contains(StatusKind::Stun));
    assert!(!immune.contains(StatusKind::Root));

    let mut ledger = StatusLedger::new();
    assert_eq!(
        ledger.apply(stun_request(4), 0, &immune),
        Err(StatusError::Immune(StatusKind::Stun))
    );
    assert_eq!(ledger.len(), 0);
    // No id was burned: the next real effect still receives the first id.
    let applied = ledger
        .apply(coin_boost_request(), 0, &immune)
        .expect("coin boost");
    assert_eq!(applied.tag_id, 1);
}

#[test]
fn the_same_definition_and_source_refresh_one_row() {
    let mut ledger = StatusLedger::new();
    let first = ledger
        .apply(coin_boost_request(), 0, &ImmunitySet::none())
        .expect("first");
    let mut again = coin_boost_request();
    again.duration_seconds = 60;
    let second = ledger
        .apply(again, 10_000, &ImmunitySet::none())
        .expect("refresh");

    assert_eq!(second.tag_id, first.tag_id);
    assert!(second.refreshed);
    assert_eq!(ledger.len(), 1);
    // The refresh resends the same instance id, so it replaces its own icon.
    let (_, tag) = decode_add_effect_tag(&second.packets[0]).expect("refreshed row");
    assert_eq!(tag.unknown_1, first.tag_id);
    let effect = ledger.get(second.tag_id).expect("active");
    assert_eq!(effect.remaining_seconds(10_000), 60);
}

#[test]
fn a_different_source_stacks_an_independent_row() {
    let mut ledger = StatusLedger::new();
    let first = ledger
        .apply(stun_request(10), 0, &ImmunitySet::none())
        .expect("first");
    let mut other = stun_request(10);
    other.source_guid = SOURCE + 1;
    let second = ledger
        .apply(other, 0, &ImmunitySet::none())
        .expect("second");

    assert_ne!(second.tag_id, first.tag_id);
    assert_eq!(ledger.len(), 2);
    // Each row carries its own tag id as the client's instance id, so the buff bar draws two icons.
    for applied in [&first, &second] {
        let (tag_id, tag) = decode_add_effect_tag(&applied.packets[0]).expect("buff row");
        assert_eq!((tag_id, tag.unknown_1), (applied.tag_id, applied.tag_id));
    }
    let removed = ledger.remove(first.tag_id).expect("remove first");
    assert_eq!(&removed[0][4..8], &first.tag_id.to_le_bytes());
    assert_eq!(ledger.len(), 1);
    assert_eq!(
        ledger.remove(first.tag_id),
        Err(StatusError::UnknownTag(first.tag_id))
    );
}

#[test]
fn restart_restores_live_rows_and_drops_elapsed_ones() {
    let mut ledger = StatusLedger::new();
    ledger
        .apply(coin_boost_request(), 0, &ImmunitySet::none())
        .expect("coin boost");
    let mut rows = ledger.profile_rows(600_000);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, 1);
    assert_eq!(rows[0].1.duration_seconds(), 3000);
    assert_eq!(rows[0].1.icon_id(), 7334);
    // A row saved before the instance id was written still resends with its tag id.
    rows[0].1.unknown_1 = 0;

    let (restored, packets) = StatusLedger::restore(&rows, 0, 600).expect("restore");
    assert_eq!(restored.len(), 1);
    assert_eq!(restored.get(1).unwrap().kind, StatusKind::Status);
    assert_eq!(restored.get(1).unwrap().definition_id, 495);
    assert_eq!(restored.get(1).unwrap().remaining_seconds(0), 2400);
    assert_eq!(packets.len(), 1);
    let (tag_id, tag) = decode_add_effect_tag(&packets[0]).expect("resend");
    assert_eq!((tag_id, tag.unknown_1), (1, 1));
    assert_eq!(tag.duration_seconds(), 2400);

    let (gone, none) = StatusLedger::restore(&rows, 0, 3000).expect("fully elapsed");
    assert_eq!(gone.len(), 0);
    assert!(none.is_empty());
    // A restored ledger keeps allocating above the ids it reloaded.
    let applied = restored_ledger_probe(&restored);
    assert_eq!(applied, 2);
}

fn restored_ledger_probe(restored: &StatusLedger) -> i32 {
    let mut ledger = restored.clone();
    ledger
        .apply(stun_request(5), 0, &ImmunitySet::none())
        .expect("post-restore")
        .tag_id
}

#[test]
fn a_disconnect_without_persistence_clears_every_row() {
    let mut ledger = StatusLedger::new();
    ledger
        .apply(stun_request(30), 0, &ImmunitySet::none())
        .expect("stun");
    let mut world = stun_request(30);
    world.composite_effect_id = 5030;
    world.source_guid = SOURCE + 7;
    ledger.apply(world, 0, &ImmunitySet::none()).expect("world");
    assert_eq!(ledger.len(), 2);

    let packets = ledger.disconnect_clear();
    assert_eq!(ledger.len(), 0);
    assert_eq!(packets.len(), 3);
    assert_eq!(&packets[0][..4], &[38, 0, 17, 0]);
    assert_eq!(&packets[1][..4], &[38, 0, 17, 0]);
    assert_eq!(&packets[2][..4], &[35, 0, 42, 0]);
}

#[test]
fn malformed_and_invalid_requests_are_rejected() {
    let good = encode_add_effect_tag(2, &EffectTagPayload::captured_coin_boost());
    assert_eq!(
        decode_add_effect_tag(&good[..ADD_EFFECT_TAG_LEN - 1]),
        Err(StatusError::BadLength {
            expected: ADD_EFFECT_TAG_LEN,
            got: ADD_EFFECT_TAG_LEN - 1,
        })
    );
    let mut short_blob = good.clone();
    short_blob[8] = 84;
    assert_eq!(
        decode_add_effect_tag(&short_blob),
        Err(StatusError::BadLength {
            expected: 85,
            got: 84
        })
    );
    let mut wrong = good;
    wrong[2] = 17;
    assert_eq!(
        decode_add_effect_tag(&wrong),
        Err(StatusError::BadOpcode {
            expected: (CLIENT_UPDATE_OPCODE, CLIENT_UPDATE_SUB_ADD_EFFECT_TAG),
            got: (CLIENT_UPDATE_OPCODE, 17),
        })
    );

    let mut ledger = StatusLedger::new();
    let mut zero = stun_request(0);
    assert_eq!(
        ledger.apply(zero, 0, &ImmunitySet::none()),
        Err(StatusError::InvalidDuration(0))
    );
    zero.duration_seconds = MAX_EFFECT_SECONDS + 1;
    assert_eq!(
        ledger.apply(zero, 0, &ImmunitySet::none()),
        Err(StatusError::InvalidDuration(MAX_EFFECT_SECONDS + 1))
    );
    let mut unnamed = stun_request(4);
    unnamed.name_id = 0;
    assert_eq!(
        ledger.apply(unnamed, 0, &ImmunitySet::none()),
        Err(StatusError::UnnamedBuff)
    );
    assert_eq!(ledger.len(), 0);
}

#[test]
fn reference_effects_keep_their_original_fields() {
    // A second captured-shape probe: the client buff row carries a remaining time separately from
    // the duration, and the recorded row must stay byte-identical when nothing changed.
    let tag = EffectTagPayload::captured_coin_boost();
    let mut effect = ActiveEffect {
        tag_id: 2,
        kind: StatusKind::Status,
        definition_id: 495,
        source_guid: 0,
        expires_at_ms: 3_600_000,
        tag,
    };
    assert_eq!(effect.remaining_seconds(0), 3600);
    assert_eq!(effect.remaining_seconds(3_599_000), 1);
    effect.expires_at_ms = 0;
    assert_eq!(effect.remaining_seconds(0), 0);
    assert_eq!(
        encode_add_effect_tag(2, &effect.tag),
        encode_add_effect_tag(2, &tag)
    );
}
