//! Housing native placement codec checks.
//! Every fixture below is small and synthetic; no capture literals are embedded here.
use osfr_tools::housing_native::*;

fn place() -> PlaceFixture {
    PlaceFixture {
        first_dword: 7,
        position: [1.0, 2.0, 3.0, 1.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: 1.0,
    }
}

fn empty_update() -> FixtureUpdate {
    FixtureUpdate {
        instance: FixtureInstance {
            guid: 0x0102_0304_0506_0708,
            house_guid: 0x1112_1314_1516_1718,
            fixture_definition_id: 34,
            unknown4: 0,
            position: [1.0, 2.0, 3.0, 1.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            tilt: [0.0, 0.0, 1.0, 0.0],
            npc_guid: 0,
            tint_id: 0,
            unknown10: 0,
            customization: Customization {
                kind: 0,
                texture_alias: String::new(),
                tint_alias: String::new(),
                tint_id: 0,
                texture_override: String::new(),
            },
            unknown11: String::new(),
            unknown12: String::new(),
            unknown13: 0,
            xml_data: String::new(),
            scale: 1.0,
            unknown16: false,
            unknown17: 0,
        },
        info: FixtureInstanceInfo {
            fixture_guid: 0x0102_0304_0506_0708,
            item_definition_id: 7466,
            coupling: CouplingDisplay {
                id: 0,
                npc_guids: Vec::new(),
                composite_effect: 0,
                effect_type: 0,
            },
        },
        definition: FixtureDefinition {
            id: 34,
            item_definition_id: 7466,
            unknown3: 1,
            model_id: 1926,
            flags: [false, false, false, true, false, false],
            category: String::new(),
            lua_call: String::new(),
            actions: Vec::new(),
            composite_effect_id: 0,
            unknown14: 1.0,
            unknown15: 1.0,
            flags2: [false, false, false, false],
        },
        unknown_guids: Vec::new(),
        words: [0, 0, 0],
    }
}

fn populated_update() -> FixtureUpdate {
    let mut update = empty_update();
    update.definition.category = "furniture".to_string();
    update.definition.lua_call = "kick()".to_string();
    update.definition.actions = vec![
        FixtureAction {
            map_key: 7,
            id: 7,
            name: "sit".to_string(),
            requirement_id: 0,
            unknown4: 0,
        },
        // key != id on purpose: the dictionary key is a separate field, not the action id.
        FixtureAction {
            map_key: 11,
            id: 12,
            name: String::new(),
            requirement_id: 5,
            unknown4: 6,
        },
    ];
    update.info.coupling.npc_guids = vec![0x21, 0x22];
    update.unknown_guids = vec![0x31, 0x32, 0x33];
    update
}

/// Reader `00ac1f50`: model dword, selection dword, definition, then the alias strings and tint.
#[test]
fn fixture_asset_bytes_follow_the_client_reader_order() {
    let asset = FixtureAsset {
        model_id: 1926,
        selection_id: 8,
        definition: populated_update().definition,
        texture_alias: "fun-block-L1".to_string(),
        tint_alias: "dyetint".to_string(),
        tint_id: 3,
        unknown_b: 0,
        bytes_c: String::new(),
        guids: Vec::new(),
        unknown_c: 0,
        unknown_d: 0,
        flag: false,
    };
    let bytes = asset.encode().unwrap();
    let mut head = 42u16.to_le_bytes().to_vec();
    head.extend_from_slice(&1926u32.to_le_bytes());
    head.extend_from_slice(&8u32.to_le_bytes());
    assert_eq!(&bytes[..head.len()], head);
    let mut tail = Vec::new();
    for text in ["fun-block-L1", "dyetint"] {
        tail.extend_from_slice(&(text.len() as i32).to_le_bytes());
        tail.extend_from_slice(text.as_bytes());
    }
    tail.extend_from_slice(&3u32.to_le_bytes()); // tint id
    tail.extend_from_slice(&[0; 5 * 4]); // preview tint, override text, guid count, two words
    tail.push(0); // preview flag
    assert!(bytes.ends_with(&tail));
    assert_eq!(FixtureAsset::decode(&bytes).unwrap(), asset);
}

#[test]
fn place_round_trip_length_and_subtype() {
    let encoded = place().encode().unwrap();
    assert_eq!(encoded.len(), 42);
    assert_eq!(u16::from_le_bytes(encoded[0..2].try_into().unwrap()), 2);
    assert_eq!(PlaceFixture::decode(&encoded).unwrap(), place());
    assert_eq!(
        HousingPacket::decode(&encoded).unwrap(),
        HousingPacket::Place(place())
    );
}

#[test]
fn place_rejects_truncation_trailing_and_non_finite() {
    let encoded = place().encode().unwrap();
    for cut in 0..encoded.len() {
        assert!(
            PlaceFixture::decode(&encoded[..cut]).is_err(),
            "cut {cut} accepted"
        );
    }
    let mut trailing = encoded.clone();
    trailing.push(0);
    assert!(PlaceFixture::decode(&trailing).is_err());
    let mut broken = place();
    broken.position[0] = f32::NAN;
    assert!(broken.encode().is_err());
    let mut bytes = encoded;
    bytes[6..10].copy_from_slice(&f32::INFINITY.to_le_bytes());
    assert!(PlaceFixture::decode(&bytes).is_err());
}

#[test]
fn update_empty_round_trip_is_227_bytes() {
    let encoded = empty_update().encode().unwrap();
    assert_eq!(
        encoded.len(),
        2 + 133 + FixtureUpdate::TAIL_BYTES_EMPTY,
        "subtype + 133-byte record + 94-byte tail"
    );
    assert_eq!(FixtureUpdate::decode(&encoded).unwrap(), empty_update());
    let packet = HousingPacket::decode(&encoded).unwrap();
    assert_eq!(packet.subtype(), 40);
    assert_eq!(packet.encode().unwrap(), encoded);
}

#[test]
fn update_populated_round_trip_variable_paths() {
    let update = populated_update();
    let encoded = update.encode().unwrap();
    assert!(encoded.len() > 229);
    assert_eq!(FixtureUpdate::decode(&encoded).unwrap(), update);
}

#[test]
fn update_rejects_oversize_counts_strings_and_lengths() {
    let baseline = empty_update().encode().unwrap();
    // npc count lives in the coupling block: subtype(2)+instance(133)+guid(8)+item(4)+id(4).
    let mut npc = baseline.clone();
    let count_at = 2 + 133 + 8 + 4 + 4;
    npc[count_at..count_at + 4].copy_from_slice(&5000i32.to_le_bytes());
    assert!(FixtureUpdate::decode(&npc).is_err());
    // definition strings: locate the definition category length by encoding a marked string.
    let mut marked = empty_update();
    marked.definition.category = "zz".to_string();
    let bytes = marked.encode().unwrap();
    let string_at = bytes.windows(2).rposition(|w| w == b"zz").unwrap();
    let mut oversize = baseline.clone();
    oversize[string_at - 4..string_at].copy_from_slice(&5000i32.to_le_bytes());
    assert!(FixtureUpdate::decode(&oversize).is_err());
    // an oversize declared string in a hand-built payload must not allocate.
    let mut hand = Vec::new();
    hand.extend_from_slice(&53u16.to_le_bytes());
    hand.extend_from_slice(&i32::MAX.to_le_bytes());
    assert!(PlacementDenied::decode(&hand).is_err());
}

#[test]
fn update_rejects_invalid_utf8_raw_bools_and_trailing() {
    let update = populated_update();
    let encoded = update.encode().unwrap();
    // category "furniture" appears once; flip its first byte to invalid UTF-8.
    let at = encoded.windows(9).position(|w| w == b"furniture").unwrap();
    let mut invalid = encoded.clone();
    invalid[at] = 0xFF;
    assert!(FixtureUpdate::decode(&invalid).is_err());
    // definition flags start right after the four definition ints (all-empty fixture).
    let flags_at = 2 + 133 + 28 + 16;
    let mut raw_bool = empty_update().encode().unwrap();
    raw_bool[flags_at] = 2;
    assert!(FixtureUpdate::decode(&raw_bool).is_err());
    let mut trailing = encoded;
    trailing.push(0);
    assert!(FixtureUpdate::decode(&trailing).is_err());
}

#[test]
fn update_rejects_non_finite_instance_fields() {
    let mut update = empty_update();
    update.instance.scale = f32::NAN;
    assert!(update.encode().is_err());
    let mut bytes = empty_update().encode().unwrap();
    // instance scale sits before the instance's trailing bool + i32 fields.
    let scale_at = 2 + 133 - 9;
    bytes[scale_at..scale_at + 4].copy_from_slice(&f32::NAN.to_le_bytes());
    assert!(FixtureUpdate::decode(&bytes).is_err());
}

#[test]
fn remove_round_trip_and_bounds() {
    let remove = FixtureRemove {
        fixture_guid: 0x0102_0304_0506_0708,
    };
    let encoded = remove.encode().unwrap();
    assert_eq!(encoded.len(), 10);
    assert_eq!(FixtureRemove::decode(&encoded).unwrap(), remove);
    for cut in 0..encoded.len() {
        assert!(FixtureRemove::decode(&encoded[..cut]).is_err(), "cut {cut}");
    }
    let mut trailing = encoded;
    trailing.push(0);
    assert!(FixtureRemove::decode(&trailing).is_err());
}

#[test]
fn house_info_round_trip_and_bool_policy() {
    let info = HouseInfo {
        in_edit_mode: true,
        unused: 10,
        is_locked: false,
        is_flora_allowed: true,
        pet_autospawn: false,
        cur_fixture_count: 2015,
        cur_landmark_count: 0,
        furniture_score: 0,
    };
    let encoded = info.encode().unwrap();
    assert_eq!(encoded.len(), 22);
    assert_eq!(HouseInfo::decode(&encoded).unwrap(), info);
    let mut raw_bool = encoded.clone();
    raw_bool[2] = 2;
    assert!(HouseInfo::decode(&raw_bool).is_err());
    for cut in 0..encoded.len() {
        assert!(HouseInfo::decode(&encoded[..cut]).is_err(), "cut {cut}");
    }
}

/// A `00aafe10` record with distinct values, a non-empty snapshot and a non-zero factory plot.
fn instance_record() -> PlayerHousingInstanceInfo {
    PlayerHousingInstanceInfo {
        owner_guid: 1,
        instance_guid: 2,
        name_id: 3,
        house_name: String::new(),
        owner_name: String::new(),
        icon_id: 101,
        fixture_count: 102,
        furniture_score: 103,
        last_visited: [104, 105],
        is_locked: false,
        is_members_only: true,
        is_flora_allowed: false,
        description: String::new(),
        keyword_list: String::new(),
        rating: 1.25,
        votes: 106,
        has_rating: true,
        can_vote: false,
        factory_plot_id: 107,
        directory_snapshot: "seaside_lot".into(),
        when_created: [108, 109],
    }
}

/// `instance_record()` in client reader `00aafe10` order: the snapshot follows factory_plot_id.
fn instance_record_bytes() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&1u64.to_le_bytes());
    bytes.extend_from_slice(&2u64.to_le_bytes());
    bytes.extend_from_slice(&3i32.to_le_bytes());
    bytes.extend_from_slice(&[0; 8]); // house and owner names
    bytes.extend_from_slice(&101i32.to_le_bytes());
    bytes.extend_from_slice(&102i32.to_le_bytes());
    bytes.extend_from_slice(&104u32.to_le_bytes());
    bytes.extend_from_slice(&105u32.to_le_bytes());
    bytes.extend_from_slice(&103i32.to_le_bytes());
    bytes.extend_from_slice(&[0, 1, 0]);
    bytes.extend_from_slice(&[0; 8]); // description and keyword list
    bytes.extend_from_slice(&1.25f32.to_le_bytes());
    bytes.extend_from_slice(&106i32.to_le_bytes());
    bytes.extend_from_slice(&[1, 0]);
    bytes.extend_from_slice(&107i32.to_le_bytes());
    bytes.extend_from_slice(&11i32.to_le_bytes());
    bytes.extend_from_slice(b"seaside_lot");
    bytes.extend_from_slice(&108u32.to_le_bytes());
    bytes.extend_from_slice(&109u32.to_le_bytes());
    bytes
}

#[test]
fn zone_data_follows_native_pair_between_the_second_and_third_words() {
    let value = ZoneData {
        flag_a: false,
        flag_b: true,
        head_size_tenths: 10,
        instance: instance_record(),
    };
    let mut expected = vec![45, 0, 0, 1];
    expected.extend_from_slice(&10i32.to_le_bytes());
    expected.extend(instance_record_bytes());

    let payload = value.encode().unwrap();
    assert_eq!(payload, expected);
    let snapshot = payload
        .windows(11)
        .position(|w| w == b"seaside_lot")
        .unwrap();
    assert_eq!(
        payload[snapshot - 8..snapshot - 4],
        107i32.to_le_bytes(),
        "the snapshot's length prefix follows factory_plot_id"
    );
    assert_eq!(ZoneData::decode(&payload).unwrap(), value);
}

#[test]
fn instance_list_rows_carry_the_snapshot_after_factory_plot_id() {
    let list = InstanceList {
        records: vec![instance_record()],
        owner_guid: 1,
    };
    let mut expected = vec![39, 0];
    expected.extend_from_slice(&1u32.to_le_bytes());
    expected.extend(instance_record_bytes());
    expected.extend_from_slice(&1u64.to_le_bytes());

    let payload = list.encode().unwrap();
    assert_eq!(payload, expected);
    assert_eq!(InstanceList::decode(&payload).unwrap(), list);
}

#[test]
fn placement_denied_is_one_string_and_refuses_trailing() {
    let denied = PlacementDenied {
        message: "no space".to_string(),
    };
    let encoded = denied.encode().unwrap();
    assert_eq!(encoded.len(), 2 + 4 + 8);
    assert_eq!(PlacementDenied::decode(&encoded).unwrap(), denied);
    let mut trailing = encoded.clone();
    trailing.push(0);
    assert!(PlacementDenied::decode(&trailing).is_err());
    let mut invalid = encoded;
    invalid[6] = 0xFF;
    assert!(PlacementDenied::decode(&invalid).is_err());
    let long = PlacementDenied {
        message: "x".repeat(MAX_STRING + 1),
    };
    assert!(long.encode().is_err());
}

#[test]
fn dispatch_rejects_empty_and_unknown_subtype() {
    assert!(HousingPacket::decode(&[]).is_err());
    assert!(HousingPacket::decode(&[0]).is_err());
    let mut unknown = 99u16.to_le_bytes().to_vec();
    unknown.extend_from_slice(&[0; 8]);
    assert!(HousingPacket::decode(&unknown).is_err());
}

#[test]
fn inner_remove_framing_has_one_family_and_one_subtype() {
    let packet = HousingPacket::FixtureRemove(FixtureRemove {
        fixture_guid: 0x0102_0304_0506_0708,
    });
    let mut expected = vec![127, 0, 41, 0];
    expected.extend_from_slice(&0x0102_0304_0506_0708u64.to_le_bytes());
    assert_eq!(packet.encode_inner().unwrap(), expected);
    assert_eq!(HousingPacket::decode_inner(&expected).unwrap(), packet);
    assert!(
        HousingPacket::decode(&expected).is_err(),
        "body API excludes family"
    );
    assert!(
        HousingPacket::decode_inner(&packet.encode().unwrap()).is_err(),
        "inner API requires family"
    );
}

#[test]
fn inner_framing_refuses_wrong_family_truncation_double_prefix_and_trailing() {
    let bytes = HousingPacket::Place(place()).encode_inner().unwrap();
    for cut in 0..bytes.len() {
        assert!(
            HousingPacket::decode_inner(&bytes[..cut]).is_err(),
            "cut {cut}"
        );
    }
    let mut wrong = bytes.clone();
    wrong[0] = 126;
    assert!(HousingPacket::decode_inner(&wrong).is_err());
    let mut doubled = vec![127, 0];
    doubled.extend_from_slice(&bytes);
    assert!(HousingPacket::decode_inner(&doubled).is_err());
    let mut trailing = bytes;
    trailing.push(0);
    assert!(HousingPacket::decode_inner(&trailing).is_err());
}

#[test]
fn every_subtype_uses_the_same_explicit_family_boundary() {
    let packets = [
        HousingPacket::Place(place()),
        HousingPacket::InstanceData(InstanceData {
            house_guid: 5,
            owner_guid: 6,
            owner_name: String::new(),
            unknown4: 0,
            name_id: 7,
            name: String::new(),
            unknown7: 0,
            max_fixtures: 8,
            max_landmarks: 0,
            fixtures: vec![(1, empty_update().instance)],
            permissions: vec![HousePermissionRecord {
                key: 1,
                pair: [9, 10],
                level: 3,
            }],
            is_locked: false,
            is_flora_allowed: true,
            cur_fixture_count: 1,
            cur_landmark_count: 0,
            unknown14: false,
            unknown15: 0,
            build_areas: vec![[[1.0, 2.0, 3.0, 0.0], [4.0, 5.0, 6.0, 0.0]]],
            icon_id: 11,
            preview: false,
            unknown18: false,
            furniture_score: 0,
            unknown20: 0,
            is_members_only: false,
            unknown22: String::new(),
            unknown23: String::new(),
            unknown24: false,
        }),
        HousingPacket::FixtureUpdate(populated_update()),
        HousingPacket::FixtureRemove(FixtureRemove { fixture_guid: 7 }),
        HousingPacket::FixtureAsset(FixtureAsset {
            model_id: 1926,
            selection_id: 8,
            definition: populated_update().definition,
            texture_alias: String::new(),
            tint_alias: String::new(),
            tint_id: 0,
            unknown_b: 0,
            bytes_c: String::new(),
            guids: vec![9],
            unknown_c: 0,
            unknown_d: 0,
            flag: false,
        }),
        HousingPacket::FixtureItemList(FixtureItemList {
            items: vec![empty_update().info],
            definitions: vec![populated_update().definition],
            words: vec![12],
        }),
        HousingPacket::HouseInfo(HouseInfo {
            in_edit_mode: true,
            unused: 0,
            is_locked: false,
            is_flora_allowed: true,
            pet_autospawn: false,
            cur_fixture_count: 2,
            cur_landmark_count: 1,
            furniture_score: 3,
        }),
        HousingPacket::ZoneData(ZoneData {
            flag_a: false,
            flag_b: true,
            head_size_tenths: 0,
            instance: PlayerHousingInstanceInfo {
                owner_guid: 2,
                instance_guid: 3,
                name_id: 4,
                house_name: String::new(),
                owner_name: String::new(),
                icon_id: 5,
                fixture_count: 6,
                furniture_score: 7,
                last_visited: [0, 0],
                is_locked: false,
                is_members_only: false,
                is_flora_allowed: true,
                description: String::new(),
                keyword_list: String::new(),
                rating: 0.0,
                votes: 0,
                has_rating: false,
                can_vote: false,
                factory_plot_id: 0,
                directory_snapshot: String::new(),
                when_created: [0, 0],
            },
        }),
        HousingPacket::PositionUpdate(PositionUpdate {
            actor_guid: 13,
            position: [1.0, 2.0, 3.0, 1.0],
            orientation: [0.0, 0.0, 0.0, 1.0],
        }),
        HousingPacket::PlacementDenied(PlacementDenied {
            message: "Denied".into(),
        }),
    ];
    for packet in packets {
        let body = packet.encode().unwrap();
        let inner = packet.encode_inner().unwrap();
        assert_eq!(inner.len(), body.len() + 2);
        assert_eq!(&inner[..2], &127u16.to_le_bytes());
        assert_eq!(&inner[2..], body);
        assert_eq!(HousingPacket::decode_inner(&inner).unwrap(), packet);
    }
}

#[test]
#[ignore = "private lab vectors: set OSFR_HOUSING_CAPTURE_DIR"]
fn private_housing_corpus_round_trips_through_registered_codec() {
    use sha2::{Digest, Sha256};
    let dir =
        std::path::PathBuf::from(std::env::var("OSFR_HOUSING_CAPTURE_DIR").expect("capture dir"));
    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(dir.join("manifest.json")).expect("private manifest"),
    )
    .expect("manifest JSON");
    let entries = manifest.as_array().expect("manifest entries");
    assert_eq!(entries.len(), 44, "fixed private corpus coverage");
    let mut counts = std::collections::BTreeMap::new();
    let mut names = std::collections::BTreeSet::new();
    for entry in entries {
        let name = entry["file"].as_str().expect("vector name");
        assert_eq!(std::path::Path::new(name).components().count(), 1);
        assert!(matches!(
            std::path::Path::new(name).components().next(),
            Some(std::path::Component::Normal(_))
        ));
        assert!(names.insert(name), "duplicate vector");
        let bytes = std::fs::read(dir.join(name)).expect("vector bytes");
        assert_eq!(bytes.len() as u64, entry["len"].as_u64().expect("length"));
        assert_eq!(
            format!("{:x}", Sha256::digest(&bytes)),
            entry["sha256"].as_str().expect("hash")
        );
        let packet = HousingPacket::decode(&bytes).expect("captured subtype body");
        assert_eq!(
            u64::from(packet.subtype()),
            entry["sub"].as_u64().expect("subtype")
        );
        assert_eq!(packet.encode().unwrap(), bytes, "body {name}");
        let mut inner = 127u16.to_le_bytes().to_vec();
        inner.extend_from_slice(&bytes);
        let decoded_inner = HousingPacket::decode_inner(&inner).unwrap();
        assert_eq!(decoded_inner, packet);
        assert_eq!(decoded_inner.encode_inner().unwrap(), inner, "inner {name}");
        *counts.entry(packet.subtype()).or_insert(0usize) += 1;
    }
    assert_eq!(
        counts.into_iter().collect::<Vec<_>>(),
        vec![(2, 8), (40, 9), (41, 8), (44, 19)]
    );
}

/// Client `00ab8c50` reads a `u32` key and a bare `FixtureInstance` (`00ab6f40`) per 127/28
/// fixture. A retail 127/28 spaces its records 137 bytes apart (4 + 133 with empty strings); a
/// whole 127/40 record per fixture shifted every later field, the build box included.
#[test]
fn instance_data_fixtures_are_bare_instances_so_the_build_box_stays_aligned() {
    let box_ = [[87.2, -100.0, -47.7, 1.0], [138.3, 60.0, -11.6, 1.0]];
    let with = |count: usize| InstanceData {
        house_guid: 5,
        owner_guid: 6,
        owner_name: String::new(),
        unknown4: 0,
        name_id: 7,
        name: String::new(),
        unknown7: 0,
        max_fixtures: 8,
        max_landmarks: 0,
        fixtures: (1..=count as u32)
            .map(|key| (key, empty_update().instance))
            .collect(),
        permissions: Vec::new(),
        is_locked: false,
        is_flora_allowed: true,
        cur_fixture_count: count as u32,
        cur_landmark_count: 0,
        unknown14: false,
        unknown15: 0,
        build_areas: vec![box_],
        icon_id: 11,
        preview: false,
        unknown18: false,
        furniture_score: 0,
        unknown20: 0,
        is_members_only: false,
        unknown22: String::new(),
        unknown23: String::new(),
        unknown24: false,
    };
    let sizes: Vec<usize> = (0..3).map(|n| with(n).encode().unwrap().len()).collect();
    assert_eq!(sizes[1] - sizes[0], 137, "key + 133-byte FixtureInstance");
    assert_eq!(sizes[2] - sizes[1], 137);
    let four = with(4);
    let decoded = InstanceData::decode(&four.encode().unwrap()).unwrap();
    assert_eq!(decoded, four);
    assert_eq!(decoded.build_areas, vec![box_]);
}
