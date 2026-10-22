use super::*;
use std::path::PathBuf;

fn content_directory() -> PathBuf {
    std::env::var_os("OSFR_OPENING_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/rust-opening-cooking-original-assets")
        })
}

fn character() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: [-1063.60595703125, 4.418806552886963, 305.6822509765625],
        rotation: [1.0, 0.0, 0.0],
    }
}

fn chat(text: &str) -> Vec<u8> {
    let mut p = vec![15, 0, 1, 0, 0, 0];
    p.extend(1u64.to_le_bytes());
    p.extend(0u64.to_le_bytes());
    for _ in 0..2 {
        p.extend([0; 12]);
        p.extend(0u32.to_le_bytes());
        p.extend(0u32.to_le_bytes());
    }
    p.extend((text.len() as u32).to_le_bytes());
    p.extend(text.as_bytes());
    p.extend([0; 28]);
    p
}

#[test]
fn crafting_movie_refuses_relocation_and_keeps_state() {
    let content = content_directory();
    assert!(content.join("cooking-campaign.json").is_file());
    let mut game = Gameplay::load(&content, None).unwrap();
    game.set_dev_tools(true);
    game.set_role(crate::account_store::AccountRole::Admin);
    game.cooking.as_mut().unwrap().begin(45).unwrap();
    let before = game.state();
    let out = game
        .handle_in_world(&chat("realm goto mae farnum"), &character(), 1_000)
        .unwrap();
    assert!(out.relocation.is_none());
    assert!(
        out.replies
            .iter()
            .any(|reply| String::from_utf8_lossy(reply).contains("Close the cooking game"))
    );
    assert_eq!(game.state(), before);
}

/// A flat terrain grid at `height` around the test character, in a temp dir.
fn flat_terrain(name: &str, height: f32) -> Arc<crate::terrain::Terrain> {
    let p = character().position;
    let mut grid = crate::terrain::HeightGrid::new(
        [p[0] - 100.0, p[2] - 100.0],
        [p[0] + 100.0, p[2] + 100.0],
        2.0,
    )
    .unwrap();
    let corner = |dx: f32, dz: f32| [p[0] + dx, height, p[2] + dz];
    grid.add_triangle([
        corner(-100.0, -100.0),
        corner(100.0, -100.0),
        corner(100.0, 100.0),
    ]);
    grid.add_triangle([
        corner(-100.0, -100.0),
        corner(100.0, 100.0),
        corner(-100.0, 100.0),
    ]);
    let dir = std::env::temp_dir().join(format!("osfr-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("zone_0.hgt"), grid.to_bytes()).unwrap();
    Arc::new(crate::terrain::Terrain::load(&dir).unwrap())
}

fn reply_text(out: &Outcome) -> String {
    out.replies
        .iter()
        .map(|reply| String::from_utf8_lossy(reply).into_owned())
        .collect()
}

#[test]
fn realm_goto_coordinates_land_on_the_terrain() {
    let content = content_directory();
    let mut game = Gameplay::load(&content, None).unwrap();
    game.set_dev_tools(true);
    game.set_role(crate::account_store::AccountRole::Admin);
    let start = character();
    let (x, z) = (start.position[0] + 20.0, start.position[2] - 15.0);
    // Without terrain data a ground teleport needs an explicit height.
    let out = game
        .handle_in_world(&chat(&format!("realm goto {x} {z}")), &start, 1_000)
        .unwrap();
    assert!(out.relocation.is_none());
    assert!(
        reply_text(&out).contains("No terrain heights"),
        "{}",
        reply_text(&out)
    );
    let out = game
        .handle_in_world(&chat(&format!("realm goto {x}, 3.5, {z}")), &start, 1_001)
        .unwrap();
    assert_eq!(out.relocation.unwrap().position, [x, 3.5, z]);

    game.set_terrain(Some(flat_terrain("goto", 42.0)));
    let out = game
        .handle_in_world(&chat(&format!("realm goto {x} {z}")), &start, 1_002)
        .unwrap();
    let relocation = out.relocation.expect("lands on the ground");
    assert_eq!(relocation.position, [x, 42.5, z]);
    assert_eq!(relocation.rotation, start.rotation);
    let here = game
        .handle_in_world(&chat("realm here"), &start, 1_003)
        .unwrap();
    assert!(
        reply_text(&here).contains("Ground 42.00"),
        "{}",
        reply_text(&here)
    );
    // Actor names still teleport as before.
    let out = game
        .handle_in_world(&chat("realm goto Roosey"), &start, 1_004)
        .unwrap();
    assert!(out.relocation.is_some());
}

#[test]
fn realm_place_adds_saves_and_removes_npcs_live() {
    let content = content_directory();
    let mut game = Gameplay::load(&content, None).unwrap();
    game.set_dev_tools(true);
    game.set_role(crate::account_store::AccountRole::Admin);
    let dir = std::env::temp_dir().join(format!("osfr-placements-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let _ = std::fs::remove_file(dir.join("npc_placements.json"));
    game.content_dir = dir.clone();
    let start = character();
    game.update_npc_visibility(&start).unwrap();
    let has = |out: &Outcome, sub: u8, guid: u64| {
        out.replies.iter().any(|p| {
            p.starts_with(&[35, 0, sub, 0])
                && (p.get(4..12) == Some(&guid.to_le_bytes()[..])
                    || p.get(6..14) == Some(&guid.to_le_bytes()[..]))
        })
    };
    let first = crate::world::PLACEMENT_GUID_BASE + 1;
    let out = game
        .handle_in_world(&chat("realm place 168 15623"), &start, 1_000)
        .unwrap();
    assert!(
        reply_text(&out).contains(&format!("Placed NPC {first}")),
        "{}",
        reply_text(&out)
    );
    assert!(has(&out, 2, first), "AddNpc sent at once");
    let saved = game.load_placements().unwrap();
    assert_eq!(saved.len(), 1);
    assert_eq!((saved[0].model_id, saved[0].name_string_id), (168, 15623));
    assert_eq!(saved[0].position, start.position);
    // Hand-added fields survive the next rewrite.
    let mut edited: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dir.join("npc_placements.json")).unwrap()).unwrap();
    assert_eq!(edited["records"][0]["source"][0]["kind"], "reconstructed");
    edited["records"][0]["note"] = "guards the bridge".into();
    std::fs::write(dir.join("npc_placements.json"), edited.to_string()).unwrap();

    // On the ground at x, z (needs terrain heights).
    let (x, z) = (start.position[0] + 10.0, start.position[2]);
    let refused = game
        .handle_in_world(&chat(&format!("realm place 182 at {x} {z}")), &start, 1_001)
        .unwrap();
    assert!(reply_text(&refused).contains("No terrain heights"));
    game.set_terrain(Some(flat_terrain("place", 12.0)));
    let second = first + 1;
    let out = game
        .handle_in_world(&chat(&format!("realm place 182 at {x} {z}")), &start, 1_002)
        .unwrap();
    assert!(has(&out, 2, second), "{}", reply_text(&out));
    let placed = game.load_placements().unwrap();
    assert_eq!(placed[1].position, [x, 12.0, z]);
    // Where you stand, with terrain: on the ground, not at your height.
    let raised = CharacterState {
        position: [start.position[0] - 5.0, 12.5, start.position[2]],
        ..start.clone()
    };
    game.handle_in_world(&chat("realm place 166"), &raised, 1_002)
        .unwrap();
    assert_eq!(game.load_placements().unwrap()[2].position[1], 12.0);
    game.handle_in_world(
        &chat(&format!("realm unplace {}", second + 1)),
        &start,
        1_002,
    )
    .unwrap();
    assert_eq!(placed[0].extra["note"], "guards the bridge");
    let list = game
        .handle_in_world(&chat("realm placements"), &start, 1_003)
        .unwrap();
    assert!(
        reply_text(&list).contains("2 placed NPCs"),
        "{}",
        reply_text(&list)
    );

    // The nearest is the one where the character stands.
    let out = game
        .handle_in_world(&chat("realm unplace nearest"), &start, 1_004)
        .unwrap();
    assert!(has(&out, 3, first), "RemoveNpc sent: {}", reply_text(&out));
    assert_eq!(game.load_placements().unwrap().len(), 1);

    // Editing the file then reloading swaps the placed NPCs live.
    std::fs::write(
        dir.join("npc_placements.json"),
        r#"{"schema_version":1,"records":[]}"#,
    )
    .unwrap();
    let out = game
        .handle_in_world(&chat("realm reload placements"), &start, 1_005)
        .unwrap();
    assert!(has(&out, 3, second), "{}", reply_text(&out));
    let bad = game
        .handle_in_world(&chat("realm place troll"), &start, 1_006)
        .unwrap();
    assert!(reply_text(&bad).contains("Usage: realm place"));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn live_world_keeps_builds_out_of_content_and_across_restarts() {
    let content = content_directory();
    let world = std::env::temp_dir().join(format!("osfr-world-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&world);
    let start = character();
    let guid;
    {
        let mut game = Gameplay::load(&content, None).unwrap();
        game.set_dev_tools(true);
        game.set_role(crate::account_store::AccountRole::Admin);
        game.set_world_dir(Some(world.clone())).unwrap();
        game.update_npc_visibility(&start).unwrap();
        let out = game
            .handle_in_world(&chat("realm place 168 named Bob the Lost"), &start, 1_000)
            .unwrap();
        let placed = game.load_placements().unwrap();
        assert_eq!(placed.len(), 1, "{}", reply_text(&out));
        assert_eq!(placed[0].name.as_deref(), Some("Bob the Lost"), "case kept");
        guid = placed[0].guid;
        let add = out
            .replies
            .iter()
            .find(|p| p.starts_with(&[35, 0, 2, 0]))
            .expect("AddNpc sent");
        assert!(
            add.windows(12).any(|w| w == b"Bob the Lost"),
            "the literal name rides in AddNpc"
        );
        game.handle_in_world(&chat("realm mark hunt start"), &start, 1_001)
            .unwrap();
        assert!(world.join("npc_placements.json").is_file());
        assert!(world.join("marks.json").is_file());
        assert!(
            !game.content_dir.join("npc_placements.json").exists(),
            "content untouched"
        );
    }
    // A restart loads the live world again and shows the placed NPC.
    let mut game = Gameplay::load(&content, None).unwrap();
    game.set_dev_tools(true);
    game.set_role(crate::account_store::AccountRole::Admin);
    game.set_world_dir(Some(world.clone())).unwrap();
    let spawned = game.update_npc_visibility(&start).unwrap();
    assert!(
            spawned
                .iter()
                .any(|p| p.starts_with(&[35, 0, 2, 0])
                    && p.windows(8).any(|w| w == guid.to_le_bytes())),
            "placed NPC persists past a restart"
        );
    let marks = game
        .handle_in_world(&chat("realm marks"), &start, 1_002)
        .unwrap();
    assert!(reply_text(&marks).contains("hunt start"));
    std::fs::remove_dir_all(&world).unwrap();
}

#[test]
fn realm_text_assigns_stable_ids_in_the_live_world() {
    let content = content_directory();
    let world = std::env::temp_dir().join(format!("osfr-world-text-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&world);
    let mut game = Gameplay::load(&content, None).unwrap();
    game.set_dev_tools(true);
    game.set_role(crate::account_store::AccountRole::Admin);
    game.set_world_dir(Some(world.clone())).unwrap();
    let first = crate::world_text::FIRST_WORLD_TEXT_ID;
    game.set_stock_text(Some(Arc::new(HashSet::from([
        crate::world_text::locale_hash(first),
    ]))));
    let start = character();
    let out = game
        .handle_in_world(&chat("realm text Find Bob the Lost!"), &start, 1_000)
        .unwrap();
    let id = first + 1;
    assert!(
        reply_text(&out).contains(&format!("Text {id}")),
        "{}",
        reply_text(&out)
    );
    let again = game
        .handle_in_world(&chat("realm text Find Bob the Lost!"), &start, 1_001)
        .unwrap();
    assert!(
        reply_text(&again).contains(&format!("Text {id}")),
        "same text keeps its id"
    );
    let list = game
        .handle_in_world(&chat("realm texts"), &start, 1_002)
        .unwrap();
    assert!(
        reply_text(&list).contains("Find Bob the Lost!"),
        "case kept: {}",
        reply_text(&list)
    );
    assert!(world.join("text.json").is_file());
    std::fs::remove_dir_all(&world).unwrap();
}

#[test]
fn developer_commands_follow_the_account_role() {
    use crate::account_store::AccountRole;
    let content = content_directory();
    let world = std::env::temp_dir().join(format!("osfr-roles-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&world);
    let mut game = Gameplay::load(&content, None).unwrap();
    game.set_dev_tools(true);
    game.set_role(crate::account_store::AccountRole::Admin);
    game.set_world_dir(Some(world.clone())).unwrap();
    let start = character();
    let try_cmd = |game: &mut Gameplay, text: &str| {
        let out = game.handle_in_world(&chat(text), &start, 1_000).unwrap();
        reply_text(&out)
    };
    // Player: no developer command runs.
    game.set_role(AccountRole::Player);
    assert!(!try_cmd(&mut game, "realm mark spot").contains("Marked"));
    assert!(!try_cmd(&mut game, "realm here").contains("Zone 0 at"));
    assert!(!world.join("marks.json").exists());
    // Builder: building commands yes, cheats no.
    game.set_role(AccountRole::Builder);
    assert!(try_cmd(&mut game, "realm mark spot").contains("Marked spot"));
    assert!(try_cmd(&mut game, "realm here").contains("Zone 0 at"));
    assert!(!try_cmd(&mut game, "realm god").contains("God mode"));
    // Admin: everything.
    game.set_role(AccountRole::Admin);
    let god = try_cmd(&mut game, "realm god");
    assert!(god.to_lowercase().contains("god"), "{god}");
    // No --dev-tools: nothing, whatever the role.
    game.set_dev_tools(false);
    assert!(!try_cmd(&mut game, "realm mark other").contains("Marked"));
    std::fs::remove_dir_all(&world).unwrap();
}

#[test]
fn account_roles_run_realm_commands_without_dev_tools() {
    use crate::account_store::AccountRole;
    let content = content_directory();
    let world = std::env::temp_dir().join(format!("osfr-role-cmds-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&world);
    let mut game = Gameplay::load(&content, None).unwrap();
    game.set_world_dir(Some(world.clone())).unwrap();
    let start = character();
    let try_cmd = |game: &mut Gameplay, text: &str| {
        let out = game.handle_in_world(&chat(text), &start, 1_000).unwrap();
        (reply_text(&out), out.dev_command_denied)
    };
    // Canary: neither flag, even an admin runs nothing.
    game.set_role(AccountRole::Admin);
    assert!(!try_cmd(&mut game, "realm mark spot").0.contains("Marked"));
    game.set_role_commands(true);
    game.set_role(AccountRole::Player);
    let (text, denied) = try_cmd(&mut game, "realm mark spot");
    assert!(denied && text.contains("account role"), "{text}");
    assert!(!world.join("marks.json").exists());
    game.set_role(AccountRole::Builder);
    assert!(
        try_cmd(&mut game, "realm mark spot")
            .0
            .contains("Marked spot")
    );
    assert!(try_cmd(&mut game, "realm god").1, "builder gets no cheats");
    game.set_role(AccountRole::Admin);
    assert!(
        try_cmd(&mut game, "realm god")
            .0
            .to_lowercase()
            .contains("god")
    );
    // Wire experiments stay behind --dev-tools for every role.
    assert!(try_cmd(&mut game, "realm mine").0.contains("--dev-tools"));
    std::fs::remove_dir_all(&world).unwrap();
}

#[test]
fn realm_marks_save_list_return_and_remove() {
    let content = content_directory();
    let mut game = Gameplay::load(&content, None).unwrap();
    game.set_dev_tools(true);
    game.set_role(crate::account_store::AccountRole::Admin);
    let dir = std::env::temp_dir().join(format!("osfr-marks-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let _ = std::fs::remove_file(dir.join("marks.json"));
    game.content_dir = dir.clone();
    let start = character();
    let out = game
        .handle_in_world(&chat("realm mark Troll Bridge"), &start, 1_000)
        .unwrap();
    assert!(
        reply_text(&out).contains("Marked troll bridge"),
        "{}",
        reply_text(&out)
    );
    let list = game
        .handle_in_world(&chat("realm marks"), &start, 1_001)
        .unwrap();
    assert!(
        reply_text(&list).contains("troll bridge (zone 0"),
        "{}",
        reply_text(&list)
    );
    let elsewhere = CharacterState {
        position: [
            start.position[0] + 50.0,
            start.position[1],
            start.position[2],
        ],
        ..start.clone()
    };
    let back = game
        .handle_in_world(&chat("realm goto mark troll bridge"), &elsewhere, 1_002)
        .unwrap();
    assert_eq!(back.relocation.unwrap().position, start.position);
    let gone = game
        .handle_in_world(&chat("realm unmark troll bridge"), &start, 1_003)
        .unwrap();
    assert!(
        reply_text(&gone).contains("Removed mark"),
        "{}",
        reply_text(&gone)
    );
    let missing = game
        .handle_in_world(&chat("realm goto mark troll bridge"), &start, 1_004)
        .unwrap();
    assert!(missing.relocation.is_none());
    assert!(reply_text(&missing).contains("No mark named"));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn realm_goto_reaches_and_interacts_with_zone_less_roosey() {
    const ROOSEY_ID: u64 = 9_500_000_255;
    let content = content_directory();
    let mut game = Gameplay::load(&content, None).unwrap();
    game.set_dev_tools(true);
    game.set_role(crate::account_store::AccountRole::Admin);
    let start = character();
    let moved = game
        .handle_in_world(&chat("realm goto Roosey"), &start, 1_000)
        .unwrap();
    let relocation = moved.relocation.expect("Roosey is in realm goto targets");
    let near_roosey = CharacterState {
        guid: start.guid,
        zone_id: start.zone_id,
        position: relocation.position,
        rotation: relocation.rotation,
    };
    let interact = npc_interaction::NpcInteractRequest {
        target: ROOSEY_ID,
        trailing_flag: false,
    }
    .encode();
    let menu = game
        .handle_in_world(&interact, &near_roosey, 1_001)
        .unwrap();
    assert!(
        menu.replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 9, 0]))
    );
}

#[test]
fn realm_npx_off_emits_player_enter_clear_only_with_dev_tools() {
    let content = content_directory();
    let mut game = Gameplay::load(&content, None).unwrap();
    game.set_dev_tools(true);
    game.set_role(crate::account_store::AccountRole::Admin);
    let out = game
        .handle_in_world(&chat("realm npx off"), &character(), 1_000)
        .unwrap();

    let mut expected = vec![41, 0, 2, 0];
    expected.extend([0; 17]);
    assert!(out.replies.iter().any(|packet| packet == &expected));
}

#[test]
fn realm_npx_off_does_not_emit_player_enter_without_dev_tools() {
    let content = content_directory();
    let mut game = Gameplay::load(&content, None).unwrap();
    let out = game
        .handle_in_world(&chat("realm npx off"), &character(), 1_000)
        .unwrap();

    assert!(
        !out.replies
            .iter()
            .any(|packet| packet.starts_with(&[41, 0, 2, 0]))
    );
}

/// A 98/1 breadcrumb request from `from` to `to` (layout: `pathing.rs`).
fn path_request(from: [f32; 3], to: [f32; 3]) -> Vec<u8> {
    let mut p = vec![98, 0, 1];
    p.extend(1u64.to_le_bytes());
    p.extend(1u32.to_le_bytes());
    p.extend(0u32.to_le_bytes());
    for v in [from, to] {
        p.extend(v.iter().chain(&[1.0]).flat_map(|f| f.to_le_bytes()));
    }
    p.extend(0u32.to_le_bytes());
    p
}

fn path_replies(out: &Outcome) -> Vec<&Vec<u8>> {
    out.replies
        .iter()
        .filter(|r| r.starts_with(&[98, 0, 2]))
        .collect()
}

#[test]
fn path_request_is_answered_from_the_terrain() {
    let content = content_directory();
    let mut game = Gameplay::load(&content, None).unwrap();
    game.set_terrain(Some(flat_terrain("path", 42.0)));
    let start = character();
    let goal = [start.position[0] + 90.0, 42.0, start.position[2]];
    let out = game
        .handle_in_world(&path_request(start.position, goal), &start, 1_000)
        .unwrap();
    let replies = path_replies(&out);
    assert_eq!(replies.len(), 1, "one 98/2 reply");
    let reply = replies[0];
    // kind 1, aux 0, then 19 points (90 m every 5 m); interior points sit on the ground.
    assert_eq!(&reply[3..11], &[1, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&reply[11..15], &19i32.to_le_bytes());
    let y = |i: usize| f32::from_le_bytes(reply[19 + i * 16..23 + i * 16].try_into().unwrap());
    assert!((y(5) - 42.0).abs() < 0.1, "{}", y(5));
}

#[test]
fn path_request_without_terrain_gets_the_failure_reply() {
    let content = content_directory();
    let mut game = Gameplay::load(&content, None).unwrap();
    let start = character();
    let goal = [start.position[0] + 50.0, 4.0, start.position[2]];
    let out = game
        .handle_in_world(&path_request(start.position, goal), &start, 1_000)
        .unwrap();
    let expected = [98, 0, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    assert_eq!(out.replies, vec![expected.to_vec()]);
}

#[test]
fn path_requests_are_capped_per_second_and_refusals_are_silent() {
    let content = content_directory();
    let mut game = Gameplay::load(&content, None).unwrap();
    game.set_terrain(Some(flat_terrain("path-rate", 42.0)));
    let start = character();
    let goal = [start.position[0] + 50.0, 42.0, start.position[2]];
    let ask = |game: &mut Gameplay, packet: &[u8], now| {
        game.handle_in_world(packet, &start, now).unwrap()
    };
    let valid = path_request(start.position, goal);
    let answered = (0..8)
        .filter(|i| !path_replies(&ask(&mut game, &valid, 5_000 + i)).is_empty())
        .count();
    assert_eq!(answered, crate::pathing::MAX_PER_SECOND as usize);
    assert!(
        !path_replies(&ask(&mut game, &valid, 6_000)).is_empty(),
        "next second"
    );
    // Too far, and a truncated packet: no reply and no error.
    let far = path_request(start.position, [start.position[0] + 400.0, 0.0, 0.0]);
    assert!(ask(&mut game, &far, 9_000).replies.is_empty());
    assert!(ask(&mut game, &valid[..20], 9_000).replies.is_empty());
}

/// T404: the quest panel's Take Me There (kind 2, `009ce560`) gets the breadcrumb's path,
/// under its own kind so the client hands it to `009cead0`.
#[test]
fn take_me_there_is_answered_like_the_breadcrumb() {
    let content = content_directory();
    let mut game = Gameplay::load(&content, None).unwrap();
    game.set_terrain(Some(flat_terrain("take-me-there", 42.0)));
    let start = character();
    let goal = [start.position[0] + 90.0, 42.0, start.position[2]];
    let mut packet = path_request(start.position, goal);
    let breadcrumb = game.handle_in_world(&packet, &start, 1_000).unwrap();
    packet[11] = 2;
    let take_me_there = game.handle_in_world(&packet, &start, 1_000).unwrap();
    let (one, two) = (path_replies(&breadcrumb), path_replies(&take_me_there));
    assert_eq!((one.len(), two.len()), (1, 1));
    assert_eq!(&two[0][3..7], &2u32.to_le_bytes());
    assert_eq!(two[0][7..], one[0][7..]);
}
