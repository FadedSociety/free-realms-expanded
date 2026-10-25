use osfr_tools::{content, startup_world, world, zone_transition};

use content::ContentDb;
use std::path::Path;
use world::{InstanceKind, World, WorldError, WorldPose};

#[path = "support/json.rs"]
mod json;

fn content() -> ContentDb {
    ContentDb::load(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/world")).unwrap()
}

fn pose(x: f32) -> WorldPose {
    WorldPose {
        position: [x, 2.0, 3.0, 1.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
    }
}

fn inner(packet: &[u8]) -> &[u8] {
    assert_eq!(&packet[..3], &[5, 0, 1]);
    let length = u32::from_le_bytes(packet[3..7].try_into().unwrap()) as usize;
    assert_eq!(packet.len(), 7 + length);
    &packet[7..]
}

#[test]
fn enter_and_leave_public_zone_builds_content_spawn_set() {
    let mut world = World::new(content(), 1_000);
    let entered = world.enter_public(10, 7, pose(1.0), 5).unwrap();

    assert_eq!(entered.zone_id, 7);
    assert_eq!(entered.instance_id, 7);
    assert_eq!(
        u16::from_le_bytes(inner(&entered.packets[0])[..2].try_into().unwrap()),
        31
    );
    assert_eq!(
        u16::from_le_bytes(inner(&entered.packets[1])[..2].try_into().unwrap()),
        43
    );
    assert_eq!(
        u16::from_le_bytes(inner(&entered.packets[2])[..2].try_into().unwrap()),
        35
    );
    assert_eq!(
        u16::from_le_bytes(inner(&entered.packets[2])[2..4].try_into().unwrap()),
        2
    );
    let npc = startup_world::parse_add_npc(inner(&entered.packets[2])).unwrap();
    assert_eq!(npc.actor_id, 2732686529148769137);
    assert_eq!(npc.name_string_id, 432135);
    assert_eq!(npc.model_id, 3311);
    assert!(entered.awaits_client_ready);
    assert_eq!(world.instance(7).unwrap().npc_spawns.len(), 1);
    assert!(world.mark_zone_ready(10, 7).unwrap());

    assert!(world.leave(10, 20));
    assert!(world.player_location(10).is_none());
    assert!(world.instance(7).unwrap().members.is_empty());
    assert_eq!(world.public_instance(7), Some(7));
}

#[test]
fn transfer_orders_begin_zoning_before_zone_details_and_npcs() {
    let mut world = World::new(content(), 1_000);
    world.enter_public(10, 7, pose(1.0), 0).unwrap();
    let target = world
        .create_child_instance(7, 7, InstanceKind::House { owner_id: 10 }, 1)
        .unwrap();

    let moved = world.enter_instance(10, target, pose(8.0), 2).unwrap();

    assert_eq!(moved.packets.len(), 3);
    assert_eq!(
        u16::from_le_bytes(inner(&moved.packets[0])[..2].try_into().unwrap()),
        31
    );
    assert_eq!(zone_transition::decode(&moved.packets[0]).unwrap().id, 7);
    assert_ne!(moved.instance_id, moved.zone_id as i32);
    assert_eq!(
        u16::from_le_bytes(inner(&moved.packets[1])[..2].try_into().unwrap()),
        43
    );
    assert_eq!(
        u16::from_le_bytes(inner(&moved.packets[2])[..2].try_into().unwrap()),
        35
    );
    assert_eq!(world.player_location(10).unwrap().instance_id, target);
}

#[test]
fn two_players_share_one_public_instance_and_ready_barrier() {
    let mut world = World::new(content(), 1_000);
    let first = world.enter_public(10, 7, pose(1.0), 5).unwrap();
    let second = world.enter_public(11, 7, pose(2.0), 6).unwrap();

    assert_eq!(first.instance_id, second.instance_id);
    assert_eq!(world.instance(7).unwrap().members.len(), 2);
    assert!(!world.mark_zone_ready(10, 7).unwrap());
    assert!(world.mark_zone_ready(11, 7).unwrap());
}

#[test]
fn encounter_child_waits_for_zone_ready_callback() {
    let mut world = World::new(content(), 1_000);
    world.enter_public(10, 7, pose(1.0), 0).unwrap();
    let child = world
        .create_child_instance(7, 7, InstanceKind::Encounter { encounter_id: 501 }, 1)
        .unwrap();
    let entered = world.enter_instance(10, child, pose(2.0), 2).unwrap();

    assert_eq!(entered.instance_id, child);
    assert_eq!(entered.zone_id, 7);
    assert_eq!(
        world.instance(child).unwrap().kind,
        InstanceKind::Encounter { encounter_id: 501 }
    );
    assert!(world.mark_zone_ready(10, child).unwrap());
}

#[test]
fn dungeon_child_is_private_and_expires_after_empty_grace() {
    let mut world = World::new(content(), 100);
    world.enter_public(10, 7, pose(1.0), 0).unwrap();
    let entered = world
        .enter_dungeon(10, 7, "fixture-dungeon", pose(4.0), 10)
        .unwrap();
    let child = world.instance(entered.instance_id).unwrap();

    assert_ne!(child.id, 7);
    assert_eq!(child.parent, Some(7));
    assert_eq!(
        child.kind,
        InstanceKind::Dungeon {
            dungeon_id: "fixture-dungeon".into()
        }
    );
    assert_eq!(child.members.len(), 1);
    assert_eq!(child.npc_spawns.len(), 1);

    assert!(world.leave(10, 20));
    assert!(world.collect_empty_instances(119).is_empty());
    assert_eq!(
        world.collect_empty_instances(120),
        vec![entered.instance_id]
    );
    assert!(world.instance(entered.instance_id).is_none());
    assert!(world.instance(7).is_some());
}

#[test]
fn failed_transfer_keeps_previous_instance_pose_and_membership() {
    let mut world = World::new(content(), 100);
    world.enter_public(10, 7, pose(12.5), 1).unwrap();
    assert!(world.mark_zone_ready(10, 7).unwrap());
    let target = world
        .create_child_instance(7, 0, InstanceKind::House { owner_id: 10 }, 2)
        .unwrap();
    let invalid = WorldPose {
        position: [f32::NAN, 0.0, 0.0, 1.0],
        ..WorldPose::ORIGIN
    };

    assert_eq!(
        world.enter_instance(10, target, invalid, 3),
        Err(WorldError::InvalidPose)
    );
    let location = world.player_location(10).unwrap();
    assert_eq!(location.instance_id, 7);
    assert_eq!(location.pose, pose(12.5));
    assert!(location.ready);
    assert!(world.instance(7).unwrap().members.contains(&10));
    assert!(world.instance(target).unwrap().members.is_empty());
}

fn current_gateway_zone_details(name: &str, sky: &str, zone_id: u32, geometry_id: u32) -> Vec<u8> {
    // Exact current gateway::initial_zone field order, wrapped as an application frame.
    let mut inner = 43u16.to_le_bytes().to_vec();
    inner.extend((name.len() as u32).to_le_bytes());
    inner.extend(name.as_bytes());
    inner.extend(2_i32.to_le_bytes());
    inner.extend([0, 0]);
    inner.extend((sky.len() as u32).to_le_bytes());
    inner.extend(sky.as_bytes());
    inner.push(0);
    inner.extend(zone_id.to_le_bytes());
    inner.extend(geometry_id.to_le_bytes());
    inner.extend([0, 0]);
    let mut packet = vec![5, 0, 1];
    packet.extend((inner.len() as u32).to_le_bytes());
    packet.extend(inner);
    packet
}

#[test]
fn opening_zone_zero_entry_matches_current_zone_details_frame() {
    let mut world = World::new(content(), 1_000);
    let entered = world.enter_public(10, 0, WorldPose::ORIGIN, 0).unwrap();

    assert_eq!(entered.instance_id, 0);
    assert_eq!(
        entered.packets,
        vec![current_gateway_zone_details(
            "Synthetic Opening Zone",
            "",
            0,
            0,
        )]
    );
}

#[test]
fn npc_visibility_loads_content_caps_the_view_and_respects_startup_exclusions() {
    use std::{collections::BTreeMap, fs, path::PathBuf};

    let content_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let zones: serde_json::Value =
        serde_json::from_slice(&fs::read(content_dir.join("zones.json")).unwrap()).unwrap();
    let zone_ids: BTreeMap<_, _> = zones["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|zone| {
            Some((
                zone["name"].as_str()?.to_ascii_lowercase(),
                zone["id"].as_u64()? as u32,
            ))
        })
        .collect();
    let mut files: Vec<_> = fs::read_dir(content_dir.join("npcs"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    files.sort();
    let regional_rows: Vec<_> = files
        .iter()
        .flat_map(|path| {
            let document: serde_json::Value =
                serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
            document["records"].as_array().unwrap().clone()
        })
        .collect();
    let zone_and_position = |npc: &serde_json::Value| {
        let position: [f32; 3] = npc["position"]
            .as_array()?
            .iter()
            .map(|value| value.as_f64().map(|number| number as f32))
            .collect::<Option<Vec<_>>>()?
            .try_into()
            .ok()?;
        let zone_id = npc["zone_id"].as_u64().map(|id| id as u32).or_else(|| {
            npc["zone"]
                .as_str()
                .and_then(|name| zone_ids.get(&name.to_ascii_lowercase()).copied())
        })?;
        Some((zone_id, position))
    };
    let regional_anchors: Vec<_> = regional_rows
        .iter()
        .filter_map(|npc| zone_and_position(npc))
        .collect();
    // All regional and extra spawns share the open-world zone 0 (R); only
    // data-backed (`legacy_data`) regional placements spawn.
    let zone_id = 0;
    let (runtime_guid, position) = regional_rows
        .iter()
        .filter(|npc| npc["placement"] == "legacy_data")
        .find_map(|npc| {
            let (_, position) = zone_and_position(npc)?;
            Some((npc["npc_id"].as_u64()?, position))
        })
        .unwrap();
    let reconstructed_guid = regional_rows
        .iter()
        .find(|npc| npc["placement"] != "legacy_data" && zone_and_position(npc).is_some())
        .and_then(|npc| npc["npc_id"].as_u64())
        .unwrap();
    let extras: serde_json::Value =
        serde_json::from_slice(&fs::read(content_dir.join("npc_spawns_extra.json")).unwrap())
            .unwrap();
    let (extra_guid, extra_position) = extras["records"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|npc| {
            if !npc["capture_match"].is_null() {
                return None;
            }
            let position: [f32; 3] = npc["position"]
                .as_array()?
                .iter()
                .map(|value| value.as_f64().map(|number| number as f32))
                .collect::<Option<Vec<_>>>()?
                .try_into()
                .ok()?;
            Some((npc["spawn_guid"].as_u64()?, position))
        })
        .unwrap();
    let extra_zone = zone_id;
    let _ = &regional_anchors;

    let mut visibility = world::NpcVisibility::load(&content_dir).unwrap();
    assert_eq!(visibility.content_key(runtime_guid), Some(runtime_guid));
    assert_eq!(visibility.content_key(reconstructed_guid), None);
    let teleport_targets = visibility.teleport_targets(zone_id);
    assert!(
        teleport_targets
            .iter()
            .any(|target| { target.guid == 9000000189 && target.name == "Samantha" })
    );
    assert!(
        teleport_targets
            .iter()
            .any(|target| { target.guid == 100000001308 && target.name == "Gus Galewind" })
    );
    assert!(
        !teleport_targets
            .iter()
            .any(|target| target.guid == reconstructed_guid)
    );
    let packets = visibility.update(zone_id, position).unwrap();
    let additions: Vec<_> = packets
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 2, 0]))
        .map(|packet| startup_world::parse_add_npc(packet).unwrap())
        .collect();
    assert!(!additions.is_empty());
    assert!(additions.len() <= 64);
    assert!(additions.iter().any(|npc| npc.actor_id == runtime_guid));
    assert_eq!(
        additions
            .iter()
            .map(|npc| npc.actor_id)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        additions.len()
    );

    let extra_packets = visibility.update(extra_zone, extra_position).unwrap();
    assert!(extra_packets.iter().any(|packet| {
        packet.starts_with(&[35, 0, 2, 0])
            && startup_world::parse_add_npc(packet).is_ok_and(|npc| npc.actor_id == extra_guid)
    }));
    let unrenderable: Vec<_> = extras["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|npc| npc["model_file_name"] == "knocker_m_boss.adr")
        .map(|npc| {
            let position: [f32; 3] = npc["position"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_f64().unwrap() as f32)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            (npc["spawn_guid"].as_u64().unwrap(), position)
        })
        .collect();
    assert_eq!(unrenderable.len(), 1); // T468 retired 10170 and 17596.
    for (guid, position) in unrenderable {
        let nearby = visibility.update(zone_id, position).unwrap();
        assert!(!nearby.iter().any(|packet| {
            packet.starts_with(&[35, 0, 2, 0])
                && startup_world::parse_add_npc(packet).is_ok_and(|npc| npc.actor_id == guid)
        }));
    }

    visibility.exclude_actor_guids([runtime_guid]);
    assert_eq!(visibility.content_key(runtime_guid), None);
    assert!(
        !visibility
            .teleport_targets(zone_id)
            .iter()
            .any(|target| target.guid == runtime_guid)
    );
    let after_exclusion = visibility.update(zone_id, position).unwrap();
    assert!(!after_exclusion.iter().any(|packet| {
        packet.starts_with(&[35, 0, 2, 0])
            && startup_world::parse_add_npc(packet).is_ok_and(|npc| npc.actor_id == runtime_guid)
    }));
}

#[test]
fn zone_less_legacy_regional_npcs_spawn_in_the_open_world_and_teleport_index() {
    const ROOSEY_ID: u64 = 9_500_000_255;
    let content_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let npc_dir = content_dir.join("npcs");
    let mut zone_less_ids = Vec::new();
    let mut roosey_position = None;
    for entry in std::fs::read_dir(&npc_dir).unwrap() {
        let file: serde_json::Value =
            serde_json::from_slice(&std::fs::read(entry.unwrap().path()).unwrap()).unwrap();
        for npc in file["records"].as_array().unwrap() {
            if npc["placement"] == "legacy_data"
                && npc["zone"].is_null()
                && npc["zone_id"].is_null()
            {
                zone_less_ids.push(npc["npc_id"].as_u64().unwrap());
            }
            if npc["npc_id"].as_u64() == Some(ROOSEY_ID) {
                roosey_position =
                    Some(serde_json::from_value::<[f32; 3]>(npc["position"].clone()).unwrap());
            }
        }
    }
    assert_eq!(zone_less_ids.len(), 50);

    let mut visibility = world::NpcVisibility::load(&content_dir).unwrap();
    let targets = visibility.teleport_targets(0);
    assert_eq!(
        zone_less_ids
            .iter()
            .filter(|id| targets.iter().any(|target| target.guid == **id))
            .count(),
        50
    );
    let roosey = targets
        .iter()
        .find(|target| target.guid == ROOSEY_ID)
        .unwrap();
    assert_eq!(Some(roosey.position), roosey_position);

    let packets = visibility.update(0, roosey.position).unwrap();
    assert!(packets.iter().any(|packet| {
        packet.starts_with(&[35, 0, 2, 0])
            && startup_world::parse_add_npc(packet).is_ok_and(|spawn| spawn.actor_id == ROOSEY_ID)
    }));
}

/// T508: a minimal content tree whose only extras rows are `rows` (one regional anchor).
fn extras_content(tag: &str, rows: serde_json::Value) -> std::path::PathBuf {
    use serde_json::json;
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("t508-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("npcs")).unwrap();
    let write = |name: &str, value: serde_json::Value| {
        std::fs::write(dir.join(name), serde_json::to_vec(&value).unwrap()).unwrap()
    };
    write("zones.json", json!({"records": []}));
    write(
        "npcs/anchor.json",
        json!({"records": [{"npc_id": 1, "position": [0.0, 0.0, 0.0]}]}),
    );
    write("npc_spawns_extra.json", json!({"records": rows}));
    dir
}

fn extra_row(guid: u64, fields: serde_json::Value) -> serde_json::Value {
    let mut row = serde_json::json!({
        "spawn_guid": guid, "model_id": 32, "position": [1.0, 0.0, 1.0], "heading": 0.5,
    });
    row.as_object_mut()
        .unwrap()
        .extend(fields.as_object().unwrap().clone());
    row
}

/// The AddNpc a loaded content dir sends for `guid` when a player stands at `position`.
fn add_npc_at(dir: &Path, guid: u64, position: [f32; 3]) -> startup_world::NpcSpawn {
    let mut visibility = world::NpcVisibility::load(dir).unwrap();
    visibility
        .update(0, position)
        .unwrap()
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 2, 0]))
        .filter_map(|packet| startup_world::parse_add_npc(packet).ok())
        .find(|npc| npc.actor_id == guid)
        .unwrap_or_else(|| panic!("{guid} is not drawn"))
}

fn add_npc_of(dir: &Path, guid: u64) -> startup_world::NpcSpawn {
    add_npc_at(dir, guid, [1.0, 0.0, 1.0])
}

#[test]
fn extra_spawn_display_fields_reach_add_npc_and_plain_rows_keep_the_template() {
    use serde_json::json;
    let dir = extras_content(
        "fields",
        json!([
            extra_row(
                11,
                json!({"texture_alias": "skullz_mohawkthug", "tint_alias": "mahogany",
                       "sub_text_name_string_id": 38093, "scale": 1.2})
            ),
            extra_row(12, json!({})),
        ]),
    );
    let shown = add_npc_of(&dir, 11);
    assert_eq!(shown.texture_alias, "skullz_mohawkthug");
    assert_eq!(shown.tint_alias, "mahogany");
    assert_eq!(shown.sub_text_name_string_id, 38093);
    assert_eq!(shown.scale, 1.2);

    // Regression canary: a row without the fields is the template, and the display row
    // differs from it in those four fields only.
    let mut plain = add_npc_of(&dir, 12);
    assert_eq!(plain.texture_alias, "");
    assert_eq!(plain.tint_alias, "");
    assert_eq!(plain.sub_text_name_string_id, -1);
    assert_eq!(plain.scale, 1.0);
    plain.actor_id = 11;
    plain.texture_alias = shown.texture_alias.clone();
    plain.tint_alias = shown.tint_alias.clone();
    plain.sub_text_name_string_id = shown.sub_text_name_string_id;
    plain.scale = shown.scale;
    assert_eq!(plain, shown);
}

#[test]
fn extra_spawn_display_fields_out_of_bounds_refuse_the_load() {
    use serde_json::json;
    let long = "a".repeat(121);
    let cases = [
        json!({"texture_alias": long}),
        json!({"tint_alias": "bad\u{7}tint"}),
        json!({"scale": 0.05}),
        json!({"scale": 10.5}),
        json!({"sub_text_name_string_id": 0}),
        json!({"sub_text_name_string_id": -3}),
    ];
    for (index, fields) in cases.into_iter().enumerate() {
        let dir = extras_content(
            &format!("bad{index}"),
            json!([extra_row(21, json!({})), extra_row(22, fields)]),
        );
        let error = world::NpcVisibility::load(&dir).err().unwrap();
        assert_eq!(
            error.kind(),
            std::io::ErrorKind::InvalidData,
            "case {index}"
        );
        assert!(error.to_string().contains("22"), "case {index}: {error}");
    }
    // The longest alias the rule allows still loads.
    let dir = extras_content(
        "edge",
        json!([extra_row(23, json!({"texture_alias": "a".repeat(120)}))]),
    );
    assert_eq!(add_npc_of(&dir, 23).texture_alias.len(), 120);
}

#[test]
fn shipped_extra_spawns_carry_legacy_skins_and_sanctuary_subtitles() {
    let content_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let extras = json::read(&content_dir.join("npc_spawns_extra.json"));
    let rows = extras["records"].as_array().unwrap();
    // (spawn guid, texture alias, sub text id): a Legacy skullz thug (Npcs.json Id 3738),
    // Bezzle the banker and Felipe the postman (Sanctuary-main Ids 33349, 33390).
    for (guid, alias, sub_text) in [
        (100_000_003_965_u64, "skullz_mohawkthug", -1),
        (9_500_000_450, "Banker", 441_416),
        (9_000_000_086, "", 24_621),
    ] {
        let position: [f32; 3] = serde_json::from_value(
            rows.iter().find(|row| row["spawn_guid"] == guid).unwrap()["position"].clone(),
        )
        .unwrap();
        let npc = add_npc_at(&content_dir, guid, position);
        assert_eq!(npc.texture_alias, alias, "{guid}");
        assert_eq!(npc.sub_text_name_string_id, sub_text, "{guid}");
    }
    assert!(
        rows.iter()
            .filter(|row| row["texture_alias"] == "skullz_mohawkthug")
            .count()
            >= 58
    );
}
