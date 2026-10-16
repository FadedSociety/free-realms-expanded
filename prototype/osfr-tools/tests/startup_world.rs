use osfr_tools::startup_world::{
    self, JournalDefinitions, JournalHub, JournalHubQuest, JournalRegion, JournalSticker, Motd,
    NpcAttachment, NpcEffectTag, NpcSpawn, PointOfInterest, QuickChatRow, WelcomeClaimCodeRow,
    WelcomeContentRow, WelcomeScreen,
};
use serde::Serialize;
use std::{fs, path::Path};

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn inner(packet: &[u8]) -> &[u8] {
    assert_eq!(&packet[..3], &[5, 0, 1]);
    let length = u32_at(packet, 3) as usize;
    assert_eq!(packet.len(), length + 7);
    &packet[7..]
}

fn captured(reference: &Path, index: usize, phase: &str) -> Vec<u8> {
    fs::read(reference.join(format!("{index:04}-{phase}.bin"))).unwrap()
}

#[derive(Serialize)]
struct WorldContentExport {
    quick_chat: Vec<startup_world::QuickChatRow>,
    points_of_interest: Vec<startup_world::PointOfInterest>,
    npc_spawns: Vec<startup_world::NpcSpawn>,
    journal_definitions: startup_world::JournalDefinitions,
    welcome_screen: startup_world::WelcomeScreen,
    motd: startup_world::Motd,
}

#[test]
#[ignore = "requires private startup reference via OSFR_GATEWAY_REFERENCE"]
fn mapped_world_content_reproduces_capture_and_exports_typed_rows() {
    let reference = std::env::var("OSFR_GATEWAY_REFERENCE")
        .expect("private reference path required for the startup byte oracle");
    let reference = Path::new(&reference);

    let packet = captured(reference, 8, "ready");
    let quick_chat = startup_world::parse_quick_chat(inner(&packet)).unwrap();
    assert_eq!(quick_chat.len(), 2470);
    assert_eq!(startup_world::build_quick_chat(&quick_chat), packet);

    let packet = captured(reference, 9, "ready");
    let points_of_interest = startup_world::parse_points_of_interest(inner(&packet)).unwrap();
    assert!(!points_of_interest.is_empty());
    assert_eq!(
        startup_world::build_points_of_interest(&points_of_interest),
        packet
    );

    let mut npc_spawns = Vec::with_capacity(23);
    for index in 10..=32 {
        let packet = captured(reference, index, "ready");
        let spawn = startup_world::parse_add_npc(inner(&packet)).unwrap();
        assert_eq!(
            startup_world::build_add_npc(&spawn),
            packet,
            "NPC capture {index}"
        );
        npc_spawns.push(spawn);
    }
    assert_eq!(npc_spawns.len(), 23);

    let packet = captured(reference, 38, "ready");
    let journal_definitions = startup_world::parse_journal_definitions(inner(&packet)).unwrap();
    assert_eq!(
        startup_world::build_journal_definitions(&journal_definitions),
        packet
    );

    let packet = captured(reference, 41, "ready");
    let welcome_screen = startup_world::parse_welcome_screen(inner(&packet)).unwrap();
    assert_eq!(startup_world::build_welcome_screen(&welcome_screen), packet);

    let packet = captured(reference, 45, "ready");
    let motd = startup_world::parse_motd(inner(&packet)).unwrap();
    assert_eq!(startup_world::build_motd(&motd), packet);

    if let Some(path) = std::env::var_os("OSFR_WORLD_CONTENT_EXPORT") {
        let path = Path::new(&path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let export = WorldContentExport {
            quick_chat,
            points_of_interest,
            npc_spawns,
            journal_definitions,
            welcome_screen,
            motd,
        };
        fs::write(path, serde_json::to_vec_pretty(&export).unwrap()).unwrap();
    }
}

#[test]
fn synthetic_quick_chat_row_changes() {
    let row = QuickChatRow {
        key: 12,
        id: 12,
        display_string_id: 100,
        say_string_id: 101,
        animation_id: 4,
        parent_id: 8,
        admin_only: 0,
        menu_icon_id: 22,
        item_id: 0,
        item_category: 0,
    };
    let original = startup_world::build_quick_chat(std::slice::from_ref(&row));
    let mut changed_row = row;
    changed_row.display_string_id += 1;
    let changed = startup_world::build_quick_chat(&[changed_row]);
    assert_ne!(original, changed);
    assert_eq!(
        startup_world::parse_quick_chat(inner(&changed)).unwrap()[0].display_string_id,
        101
    );
}

#[test]
fn synthetic_poi_position_and_name_change() {
    let row = PointOfInterest {
        id: 1,
        name_string_id: 2,
        location_id: 3,
        position: [1.0, 2.0, 3.0],
        heading: 0.5,
        icon_id: 4,
        notification_type: 5,
        sub_name_string_id: 6,
        unknown_field: 7,
        breadcrumb_quest_id: 8,
        teleport_location_id: 9,
        atlas_name: "atlas_a".into(),
        unknown_trailing_field: 10,
    };
    let original = startup_world::build_points_of_interest(&[row.clone()]);
    let mut changed_row = row;
    changed_row.name_string_id = 13;
    changed_row.position[0] = 11.0;
    let changed = startup_world::build_points_of_interest(&[changed_row]);
    assert_ne!(original, changed);
    let parsed = startup_world::parse_points_of_interest(inner(&changed)).unwrap();
    assert_eq!(parsed[0].name_string_id, 13);
    assert_eq!(parsed[0].position[0], 11.0);
}

#[test]
fn synthetic_npc_actor_transform_flags_and_lists_change() {
    let mut row = NpcSpawn::default();
    row.actor_id = 0x1122_3344_5566_7788;
    row.name_string_id = 31;
    row.model_id = 42;
    row.position = [1.0, 2.0, 3.0, 1.0];
    row.rotation = [0.0, 0.0, 0.0, 1.0];
    row.npc_name = "npc-fixture".into();
    row.attachments.push(NpcAttachment {
        model_name: "model-fixture".into(),
        texture_alias: "texture-fixture".into(),
        tint_alias: "tint-fixture".into(),
        tint_id: 7,
        composite_effect_id: 8,
        slot: 9,
    });
    row.character_variable_keys = vec![5, 6];
    row.effect_tags.push(NpcEffectTag::default());
    let original = startup_world::build_add_npc(&row);
    let mut changed_row = row;
    changed_row.actor_id += 1;
    changed_row.name_string_id += 1;
    changed_row.model_id += 1;
    changed_row.position[0] = 99.0;
    changed_row.hide_nameplate = 1;
    let changed = startup_world::build_add_npc(&changed_row);
    assert_ne!(original, changed);
    let parsed = startup_world::parse_add_npc(inner(&changed)).unwrap();
    assert_eq!(parsed.actor_id, changed_row.actor_id);
    assert_eq!(parsed.name_string_id, changed_row.name_string_id);
    assert_eq!(parsed.model_id, changed_row.model_id);
    assert_eq!(parsed.position[0], 99.0);
    assert_eq!(parsed.hide_nameplate, 1);
    assert_eq!(parsed.attachments.len(), 1);
    assert_eq!(parsed.effect_tags.len(), 1);
    assert_eq!(parsed.character_variable_keys, vec![5, 6]);
}

#[test]
fn synthetic_journal_rows_change() {
    let mut value = JournalDefinitions {
        regions: vec![JournalRegion {
            key: 1,
            fields: [2; 7],
        }],
        hubs: vec![JournalHub {
            key: 3,
            fields: [4; 10],
        }],
        hub_quests: vec![JournalHubQuest {
            key: 5,
            fields: [6; 3],
        }],
        stickers: vec![JournalSticker {
            key: 7,
            fields: [8; 9],
        }],
    };
    let original = startup_world::build_journal_definitions(&value);
    value.hub_quests[0].fields[1] = 99;
    let changed = startup_world::build_journal_definitions(&value);
    assert_ne!(original, changed);
    let parsed = startup_world::parse_journal_definitions(inner(&changed)).unwrap();
    assert_eq!(parsed.hub_quests[0].fields[1], 99);
}

#[test]
fn synthetic_welcome_rows_and_flag_change() {
    let mut value = WelcomeScreen {
        show_welcome_screen: 1,
        contents: vec![WelcomeContentRow {
            first_string_id: 1,
            second_string_id: 2,
        }],
        claim_codes: vec![WelcomeClaimCodeRow {
            code: "fixture-code".into(),
            title_string_id: 3,
            description_string_id: 4,
            image_set_id: 5,
            tint_alias: "fixture-tint".into(),
        }],
        seconds_since_last_login: 6,
        starting_wallet_balance: 7,
    };
    let original = startup_world::build_welcome_screen(&value);
    value.show_welcome_screen = 0;
    value.contents[0].second_string_id = 10;
    value.claim_codes[0].tint_alias.push_str("-changed");
    let changed = startup_world::build_welcome_screen(&value);
    assert_ne!(original, changed);
    let parsed = startup_world::parse_welcome_screen(inner(&changed)).unwrap();
    assert_eq!(parsed.show_welcome_screen, 0);
    assert_eq!(parsed.contents[0].second_string_id, 10);
    assert_eq!(parsed.claim_codes[0].tint_alias, "fixture-tint-changed");
}

#[test]
fn synthetic_motd_strings_change() {
    let original = startup_world::build_motd(&Motd {
        first_string: "first".into(),
        second_string: "second".into(),
    });
    let changed = startup_world::build_motd(&Motd {
        first_string: "first-longer".into(),
        second_string: "second".into(),
    });
    assert_ne!(original, changed);
    let parsed = startup_world::parse_motd(inner(&changed)).unwrap();
    assert_eq!(parsed.first_string, "first-longer");
}
