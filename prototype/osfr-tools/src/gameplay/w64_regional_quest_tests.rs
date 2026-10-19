use super::*;

const SAMANTHA_ID: u64 = 9_000_000_189;
const INTRODUCE_YOURSELF_ID: u32 = 3_000_021;

/// Per-run content copy; removed on drop, including while a test unwinds.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl std::ops::Deref for Scratch {
    type Target = std::path::Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The guard also runs while a test unwinds, so a failing run leaves no content
/// behind for the next one.
#[test]
fn scratch_content_is_removed_on_drop() {
    let path = {
        let scratch = Scratch::new("scratch-drop");
        std::fs::write(scratch.join("leftover.json"), b"{}").unwrap();
        scratch.to_path_buf()
    };
    assert!(!path.exists(), "scratch content was not removed on drop");
}

#[test]
fn live_quest_offers_require_real_level_and_the_active_job() {
    let source = bundled_content();
    let scratch = Scratch::new("eligibility");
    std::fs::copy(source.join("jobs.json"), scratch.join("jobs.json")).unwrap();
    for folder in ["quests", "npcs"] {
        std::fs::create_dir_all(scratch.join(folder)).unwrap();
        for entry in std::fs::read_dir(source.join(folder)).unwrap() {
            let path = entry.unwrap().path();
            std::fs::copy(&path, scratch.join(folder).join(path.file_name().unwrap())).unwrap();
        }
    }
    let path = scratch.join("quests/adventurer.json");
    let mut file: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let set_requirement = |file: &mut serde_json::Value, job: Option<i32>| {
        let quest = file["records"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|q| q["id"] == INTRODUCE_YOURSELF_ID)
            .unwrap();
        quest["min_level"] = serde_json::json!(3);
        quest["job"] = serde_json::json!(job);
        std::fs::write(&path, serde_json::to_vec(file).unwrap()).unwrap();
    };
    set_requirement(&mut file, None);
    let mut game = Gameplay::load(&reference(), None).unwrap();
    game.profiles = Some(Profiles::load(&reference(), Some(native_test_profiles())).unwrap());
    game.bind_npc_quest_catalogue(&scratch, 1).unwrap();
    assert_eq!(
        game.npc_quests.as_ref().unwrap().eligibility().player_level,
        1,
        "the active job starts at rank 1"
    );
    assert!(
        !game
            .npc_quests
            .as_ref()
            .unwrap()
            .can_offer(INTRODUCE_YOURSELF_ID, SAMANTHA_ID)
    );
    game.profiles.as_mut().unwrap().set_experience(200);
    game.refresh_npc_quest_eligibility(0);
    assert_eq!(
        game.npc_quests.as_ref().unwrap().eligibility().player_level,
        3,
        "eligibility follows the active job's XP"
    );
    assert!(
        game.npc_quests
            .as_mut()
            .unwrap()
            .offer(INTRODUCE_YOURSELF_ID, SAMANTHA_ID)
            .changed
    );

    set_requirement(&mut file, Some(43));
    let mut profile = game.profiles.as_ref().unwrap().state().clone();
    assert!(profile.unlocked.contains(&43));
    assert_ne!(profile.active, 43);
    profile.experience.insert(43, 200);
    game.profiles = Some(Profiles::load(&reference(), Some(profile)).unwrap());
    game.bind_npc_quest_catalogue(&scratch, 1).unwrap();
    assert!(
        !game
            .npc_quests
            .as_ref()
            .unwrap()
            .can_offer(INTRODUCE_YOURSELF_ID, SAMANTHA_ID)
    );
    game.profiles
        .as_mut()
        .unwrap()
        .select(43, &game.inventory)
        .unwrap();
    game.refresh_npc_quest_eligibility(0);
    assert_eq!(
        game.npc_quests.as_ref().unwrap().eligibility().player_level,
        3,
        "eligibility follows the newly active job's XP"
    );
    assert!(
        game.npc_quests
            .as_mut()
            .unwrap()
            .offer(INTRODUCE_YOURSELF_ID, SAMANTHA_ID)
            .changed
    );
}

fn level_three_world_game(label: &str) -> (Gameplay, CharacterState, Scratch) {
    let source = bundled_content();
    let scratch = Scratch::new(label);
    std::fs::copy(source.join("jobs.json"), scratch.join("jobs.json")).unwrap();
    for folder in ["quests", "npcs"] {
        std::fs::create_dir_all(scratch.join(folder)).unwrap();
        for entry in std::fs::read_dir(source.join(folder)).unwrap() {
            let path = entry.unwrap().path();
            std::fs::copy(&path, scratch.join(folder).join(path.file_name().unwrap())).unwrap();
        }
    }
    let path = scratch.join("quests/adventurer.json");
    let mut file: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let quest = file["records"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|quest| quest["id"] == INTRODUCE_YOURSELF_ID)
        .unwrap();
    quest["min_level"] = serde_json::json!(3);
    quest["job"] = serde_json::Value::Null;
    std::fs::write(&path, serde_json::to_vec(&file).unwrap()).unwrap();

    let mut game = Gameplay::load(&reference(), None).unwrap();
    game.profiles = Some(Profiles::load(&reference(), Some(native_test_profiles())).unwrap());
    // Exercise the non-opening request path, which does not refresh eligibility
    // while switching jobs. Only the NPC request path can make the offer legal.
    game.opening = None;
    game.bind_npc_quest_catalogue(&scratch, 1).unwrap();

    let npc_file: serde_json::Value = serde_json::from_slice(
        &std::fs::read(source.join("npcs/cobblestone_village.json")).unwrap(),
    )
    .unwrap();
    let giver = npc_file["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["npc_id"].as_u64() == Some(SAMANTHA_ID))
        .unwrap();
    let position: [f32; 3] = serde_json::from_value(giver["position"].clone()).unwrap();
    let character = CharacterState {
        guid: 1,
        zone_id: 0,
        position,
        rotation: [1.0, 0.0, 0.0],
    };
    game.update_npc_visibility(&character).unwrap();
    (game, character, scratch)
}

#[test]
fn npc_request_path_refreshes_eligibility_after_a_request_handler_job_switch() {
    let (mut game, character, _scratch) = level_three_world_game("npc-refresh");
    let mut switch = vec![26, 0, 13, 0];
    switch.extend(43i32.to_le_bytes());
    assert!(
        game.handle_in_world(&switch, &character, 1_000)
            .unwrap()
            .changed
    );
    assert_eq!(game.profiles.as_ref().unwrap().active(), 43);

    game.handle_in_world(&packet_for_npc(SAMANTHA_ID), &character, 1_001)
        .unwrap();
    game.handle_in_world(&select_npc_option(SAMANTHA_ID, 1), &character, 1_002)
        .unwrap();
    let chosen = game
        .handle_in_world(&choose_quest_row(), &character, 1_003)
        .unwrap();
    assert!(
        chosen
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 1, 0])),
        "the NPC request path must refresh eligibility after the job switch"
    );
    assert_eq!(
        game.npc_quests.as_ref().unwrap().state().pending_offer,
        Some(INTRODUCE_YOURSELF_ID)
    );
}

#[test]
fn denied_accept_after_a_request_handler_job_switch_changes_nothing_and_requests_no_save() {
    let (mut game, character, _scratch) = level_three_world_game("npc-denied");
    let mut switch = vec![26, 0, 13, 0];
    switch.extend(43i32.to_le_bytes());
    assert!(
        game.handle_in_world(&switch, &character, 1_000)
            .unwrap()
            .changed
    );
    assert_eq!(game.profiles.as_ref().unwrap().active(), 43);

    game.handle_in_world(&packet_for_npc(SAMANTHA_ID), &character, 1_001)
        .unwrap();
    game.handle_in_world(&select_npc_option(SAMANTHA_ID, 1), &character, 1_002)
        .unwrap();
    let offered = game
        .handle_in_world(&choose_quest_row(), &character, 1_003)
        .unwrap();
    assert!(
        offered
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 1, 0]))
    );

    let mut back = vec![26, 0, 13, 0];
    // Chef has a complete starter outfit in this package; Kart Driver does
    // not, and the client rejects a profile swap without Chest and Legs.
    back.extend(45i32.to_le_bytes());
    assert!(
        game.handle_in_world(&back, &character, 2_000)
            .unwrap()
            .changed
    );
    let before = game.npc_quests.as_ref().unwrap().state().clone();
    assert_eq!(before.pending_offer, Some(INTRODUCE_YOURSELF_ID));
    assert_eq!(
        before.quests[&INTRODUCE_YOURSELF_ID].status,
        crate::quest_engine::QuestStatus::Offered
    );

    let mut accept = vec![49, 0, 2, 0, 0, 0];
    accept.extend(INTRODUCE_YOURSELF_ID.to_le_bytes());
    accept.push(1);
    let refused = game.handle_in_world(&accept, &character, 3_000).unwrap();
    // Gateway (src/gateway.rs:1758-1759) requests persistence only from `changed`.
    assert!(!refused.changed, "a denied accept must not request a save");
    assert!(refused.relocation.is_none());
    assert!(refused.zoning.is_none());
    assert!(refused.scene_zoning.is_none());
    assert!(!refused.pets_changed);
    assert_eq!(game.npc_quests.as_ref().unwrap().state(), &before);
    assert!(
        !refused
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 3, 0])),
        "a denied accept must not activate the quest"
    );
}

fn reference() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-opening-cooking-original-assets")
}

fn bundled_content() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn native_test_profiles() -> ProfileState {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/reports/round5b-native-20261014T053834Z/character.json");
    let saved: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).expect("native test save exists"))
            .expect("native test save parses");
    serde_json::from_value(saved["gameplay"]["profiles"].clone())
        .expect("native gameplay snapshot has the profile state")
}

fn packet_for_npc(npc_id: u64) -> Vec<u8> {
    npc_interaction::NpcInteractRequest {
        target: npc_id,
        trailing_flag: false,
    }
    .encode()
}

fn select_npc_option(npc_id: u64, option_id: i32) -> Vec<u8> {
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(npc_id.to_le_bytes());
    packet.extend(option_id.to_le_bytes());
    packet
}

fn offer_chat(text: &str) -> Vec<u8> {
    let mut packet = vec![15, 0, 1, 0, 0, 0];
    packet.extend(1_u64.to_le_bytes());
    packet.extend(0_u64.to_le_bytes());
    for _ in 0..2 {
        packet.extend([0; 12]);
        packet.extend(0_u32.to_le_bytes());
        packet.extend(0_u32.to_le_bytes());
    }
    packet.extend((text.len() as u32).to_le_bytes());
    packet.extend(text.as_bytes());
    packet.extend([0; 28]);
    packet
}

fn expected_samantha_dialogue() -> Vec<u8> {
    // Quest choice row ("Introduce Yourself") first, then Goodbye. Dwords 2-3
    // carry Samantha's GUID as the dialog camera target.
    let mut packet = vec![26, 0, 3, 0];
    packet.extend(24_940_u32.to_le_bytes()); // Samantha's greeting
    packet.extend([0; 4]);
    packet.extend(SAMANTHA_ID.to_le_bytes());
    packet.extend([0; 5]);
    packet.extend(2_u32.to_le_bytes());
    for value in [301_u32, 17, 94_247, 0, 18, 300, 17, 8075, 0, 18] {
        packet.extend(value.to_le_bytes());
    }
    packet.extend([0; 64]);
    assert_eq!(packet.len(), 133);
    packet
}

fn expected_blank_dialogue() -> Vec<u8> {
    // Diagnostic variant v4: Goodbye only, no quest ids.
    let mut packet = vec![26, 0, 3, 0];
    packet.extend([0; 21]);
    packet.extend(1_u32.to_le_bytes());
    for value in [300_u32, 17, 8075, 0, 18] {
        packet.extend(value.to_le_bytes());
    }
    packet.extend([0; 64]);
    packet
}

fn choose_quest_row() -> Vec<u8> {
    let mut packet = vec![26, 0, 6, 0];
    packet.extend(npc_interaction::QUEST_CHOICE_RESPONSE.to_le_bytes());
    packet
}

fn expected_quest_offer(_prefix: u16, intro_id: u32) -> Vec<u8> {
    // Native layout (run 142216): no u16 prefix; quest id + five dwords; 114 bytes.
    let mut packet = vec![49, 0, 1, 0, 0, 0];
    packet.extend(INTRODUCE_YOURSELF_ID.to_le_bytes());
    for value in [intro_id, 94_247, 94_247, 94_248, 94_247] {
        packet.extend(value.to_le_bytes());
    }
    packet.push(0);
    packet.extend([0; 69]);
    packet.extend([0; 14]);
    assert_eq!(packet.len(), 114);
    packet
}

fn offer_test_game(dev_tools: bool) -> (Gameplay, CharacterState) {
    let content = bundled_content();
    let reference = reference();
    let mut game = Gameplay::load(&reference, None).unwrap();
    game.profiles = Some(Profiles::load(&reference, Some(native_test_profiles())).unwrap());
    game.bind_npc_quest_catalogue(&content, 1).unwrap();
    game.set_dev_tools(dev_tools);
    game.set_role(crate::account_store::AccountRole::Admin);

    let npc_file: serde_json::Value = serde_json::from_slice(
        &std::fs::read(content.join("npcs/cobblestone_village.json")).unwrap(),
    )
    .unwrap();
    let samantha = npc_file["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["npc_id"].as_u64() == Some(SAMANTHA_ID))
        .unwrap();
    let position: [f32; 3] = serde_json::from_value(samantha["position"].clone()).unwrap();
    let character = CharacterState {
        guid: 1,
        zone_id: 0,
        position,
        rotation: [1.0, 0.0, 0.0],
    };
    game.update_npc_visibility(&character).unwrap();
    game.handle_in_world(&packet_for_npc(SAMANTHA_ID), &character, 1_000)
        .unwrap();
    let talk = game
        .handle_in_world(&select_npc_option(SAMANTHA_ID, 1), &character, 1_001)
        .unwrap();
    // Talk opens the dialog with the quest as a choice; choosing it sends 49/1.
    assert!(!talk.replies.iter().any(|p| p.starts_with(&[49, 0, 1, 0])));
    assert!(
        talk.replies
            .iter()
            .any(|p| p == &expected_samantha_dialogue())
    );
    let chosen = game
        .handle_in_world(&choose_quest_row(), &character, 1_002)
        .unwrap();
    assert!(chosen.replies.iter().any(|p| p.starts_with(&[49, 0, 1, 0])));
    assert_eq!(
        game.last_npc_dialogue
            .as_ref()
            .map(|(_, _, _, packet)| packet),
        Some(&expected_samantha_dialogue())
    );
    (game, character)
}

#[test]
fn journal_requests_route_through_the_npc_quest_engine() {
    let (mut game, character) = offer_test_game(false);
    let mut accept = vec![49, 0, 2, 0, 0, 0];
    accept.extend((INTRODUCE_YOURSELF_ID as i32).to_le_bytes());
    accept.push(1);
    game.handle_in_world(&accept, &character, 1_005).unwrap();
    let state = |game: &Gameplay| game.npc_quests.as_ref().unwrap().state().clone();

    // 47/12 selects the owned active quest and the state is marked for saving.
    let mut select = vec![47, 0, 12];
    select.extend((INTRODUCE_YOURSELF_ID as i32).to_le_bytes());
    let out = game.handle_in_world(&select, &character, 1_006).unwrap();
    assert!(out.changed);
    assert_eq!(state(&game).selected_quest_id, Some(INTRODUCE_YOURSELF_ID));

    // A forged id changes nothing and answers nothing; a malformed one is an error.
    let before = state(&game);
    let mut forged = vec![47, 0, 12];
    forged.extend(999_999i32.to_le_bytes());
    let out = game.handle_in_world(&forged, &character, 1_007).unwrap();
    assert!(!out.changed && out.replies.is_empty());
    assert_eq!(state(&game), before);
    assert!(
        game.handle_in_world(&[47, 0, 12, 1, 0, 0], &character, 1_008)
            .is_err()
    );

    // 47/13: wire 0 = unlocked = client auto-switch set; wire 1 = locked.
    assert!(
        game.handle_in_world(&[47, 0, 13, 0], &character, 1_009)
            .unwrap()
            .changed
    );
    assert!(state(&game).auto_switch_selected);
    assert!(
        game.handle_in_world(&[47, 0, 13, 1], &character, 1_010)
            .unwrap()
            .changed
    );
    assert!(!state(&game).auto_switch_selected);

    // 49/15 for an id the server never registered offers nothing.
    let before = state(&game);
    let mut crumb = vec![49, 0, 15, 0, 0, 0];
    crumb.extend(999_999i32.to_le_bytes());
    let out = game.handle_in_world(&crumb, &character, 1_011).unwrap();
    assert!(!out.changed);
    // W175: an unlisted original id is refused with one chat line.
    assert_eq!(
        out.replies,
        vec![chat::system(crate::quest_journal::BREADCRUMB_REFUSED)]
    );
    assert_eq!(state(&game), before);
}

// ---- W175: the 49/15 original-id crosswalk and 45/7 client-reported goals ----

/// "Introduce Yourself" at Highroad Junction: enabled, no prerequisite, no level gate.
const CROSSWALK_QUEST: u32 = 3_000_022;
/// "Farming 101": disabled in the catalogue today, so the engine refuses to offer it.
const DISABLED_QUEST: u32 = 3_000_006;

fn quest_game() -> (Gameplay, CharacterState) {
    let reference = reference();
    let mut game = Gameplay::load(&reference, None).unwrap();
    game.profiles = Some(Profiles::load(&reference, Some(native_test_profiles())).unwrap());
    game.bind_npc_quest_catalogue(&bundled_content(), 1)
        .unwrap();
    let character = CharacterState {
        guid: 1,
        zone_id: 0,
        position: [0.0; 3],
        rotation: [1.0, 0.0, 0.0],
    };
    (game, character)
}

fn breadcrumb_packet(quest_id: u32) -> Vec<u8> {
    let mut packet = vec![49, 0, 15, 0, 0, 0];
    packet.extend((quest_id as i32).to_le_bytes());
    packet
}

fn claim_packet(client_id: i32) -> Vec<u8> {
    let mut packet = vec![45, 0, 7];
    packet.extend(client_id.to_le_bytes());
    packet
}

fn quest_state(game: &Gameplay) -> crate::quest_engine::QuestState {
    game.npc_quests.as_ref().unwrap().state().clone()
}

#[test]
fn binding_the_catalogue_loads_the_shipped_crosswalk() {
    let (game, _) = quest_game();
    assert!(!game.breadcrumb_crosswalk.is_empty());
    // Every listed target has a definition, so every one is registered with its giver.
    let targets: BTreeSet<u32> = game.breadcrumb_crosswalk.values().copied().collect();
    let registered: BTreeSet<u32> = game.breadcrumb_quests.keys().copied().collect();
    assert_eq!(targets, registered);
}

#[test]
fn breadcrumb_start_goes_through_the_original_id_crosswalk() {
    let (mut game, character) = quest_game();
    game.register_breadcrumb_crosswalk(std::collections::BTreeMap::from([
        (1807, CROSSWALK_QUEST),
        (1806, DISABLED_QUEST),
    ]));
    let refusal = chat::system(crate::quest_journal::BREADCRUMB_REFUSED);
    let before = quest_state(&game);

    // An unlisted original id, a regional id used as an original one and a forged id are
    // each answered with one chat line and change nothing. So does a listed id whose
    // quest the engine will not offer (disabled content).
    for (step, id) in [1808, CROSSWALK_QUEST, 999_999, 1806]
        .into_iter()
        .enumerate()
    {
        let out = game
            .handle_in_world(&breadcrumb_packet(id), &character, 2_000 + step as u64)
            .unwrap();
        assert!(!out.changed, "id {id}");
        assert_eq!(out.replies, vec![refusal.clone()], "id {id}");
        assert_eq!(quest_state(&game), before, "id {id}");
    }

    // A listed original id offers its REGIONAL quest (49/1), once.
    let out = game
        .handle_in_world(&breadcrumb_packet(1807), &character, 2_010)
        .unwrap();
    assert!(out.changed);
    assert!(out.replies.iter().any(|p| p.starts_with(&[49, 0, 1, 0])));
    assert!(!out.replies.contains(&refusal));
    let offered = quest_state(&game);
    assert_eq!(offered.pending_offer, Some(CROSSWALK_QUEST));
    assert_eq!(offered.quests.len(), 1);
    // Pressed again with the offer still open: shown again, nothing new made.
    let again = game
        .handle_in_world(&breadcrumb_packet(1807), &character, 2_011)
        .unwrap();
    assert!(!again.changed);
    assert!(again.replies.iter().any(|p| p.starts_with(&[49, 0, 1, 0])));
    assert_eq!(quest_state(&game), offered);
}

#[test]
fn an_unlisted_breadcrumb_is_refused_without_a_quest_engine() {
    let mut game = Gameplay::load(&reference(), None).unwrap();
    assert!(game.npc_quests.is_none());
    let character = CharacterState {
        guid: 1,
        zone_id: 0,
        position: [0.0; 3],
        rotation: [1.0, 0.0, 0.0],
    };
    let out = game
        .handle_in_world(&breadcrumb_packet(1807), &character, 6_000)
        .unwrap();
    assert!(!out.changed);
    assert_eq!(
        out.replies,
        vec![chat::system(crate::quest_journal::BREADCRUMB_REFUSED)]
    );
}

/// Two synthetic quests that each ask the client to judge one goal.
fn client_goal_engine() -> crate::quest_engine::QuestEngine {
    use crate::quest_engine::{
        CharacterEligibility, ClientTrigger, ObjectiveKind, QuestDefinition, QuestEngine,
        QuestObjective, QuestRequirements,
    };
    let quest = |id: u32, client_id: u32, trigger: ClientTrigger| QuestDefinition {
        id,
        title: format!("Client goal {id} (R)"),
        description: String::new(),
        giver_npc_id: 1,
        turnin_npc_id: 1,
        prerequisite_quest_ids: vec![],
        requirements: QuestRequirements::default(),
        ordered: true,
        objectives: vec![QuestObjective {
            id: 1,
            required: 1,
            description: "Judged by the client (R)".into(),
            kind: ObjectiveKind::ClientReported { client_id, trigger },
        }],
        reward: crate::rewards::RewardGrant {
            coins: 5,
            items: vec![],
            job_xp: std::collections::BTreeMap::new(),
            display: crate::rewards::RewardDisplay {
                source_guid: 1,
                player_guid: 1,
                reward_kind: 0,
                icon_id: 0,
                name_id: 0,
                rows: vec![],
            },
        },
    };
    QuestEngine::new(
        1,
        vec![
            quest(
                7001,
                55,
                ClientTrigger::UiEvent {
                    event: "onTestEvent".into(),
                },
            ),
            quest(7002, 77, ClientTrigger::FirstMovement { mouse: true }),
            quest(
                7003,
                73,
                ClientTrigger::LookAt {
                    guid: 0x1122_3344_5566_7788,
                    offset_tenths: 15,
                    range: 12,
                    angle_degrees: 45,
                },
            ),
            {
                // Ordered: reach the table, then close the journal. The second goal only
                // opens after the first, so the movement event must register it.
                let mut ordered = quest(
                    7004,
                    88,
                    ClientTrigger::UiEvent {
                        event: "onCloseJournal".into(),
                    },
                );
                ordered.objectives.insert(
                    0,
                    QuestObjective {
                        id: 1,
                        required: 1,
                        description: "Visit the table (R)".into(),
                        kind: ObjectiveKind::VisitPosition {
                            zone_id: 0,
                            position: [5.0, 0.0, 5.0],
                            radius: 2.0,
                        },
                    },
                );
                ordered.objectives[1].id = 2;
                ordered
            },
        ],
        CharacterEligibility {
            member: true,
            member_quest_limit: 10,
            nonmember_quest_limit: 10,
            ..Default::default()
        },
        None,
    )
    .unwrap()
}

fn engine_mut(game: &mut Gameplay) -> &mut crate::quest_engine::QuestEngine {
    game.npc_quests.as_mut().unwrap()
}

#[test]
fn client_goal_claims_move_only_an_open_goal_of_an_active_quest() {
    let (mut game, character) = quest_game();
    game.npc_quests = Some(client_goal_engine());

    // Nothing is active yet: a claim for a real id changes nothing and registers nothing.
    let out = game
        .handle_in_world(&claim_packet(55), &character, 3_000)
        .unwrap();
    assert!(!out.changed && out.replies.is_empty());

    // 7001 becomes active; 7002 stays offered.
    assert!(engine_mut(&mut game).offer(7001, 1).changed);
    assert!(engine_mut(&mut game).accept(7001).changed);
    assert!(engine_mut(&mut game).offer(7002, 1).changed);
    let before = quest_state(&game);

    // A forged id moves nothing; the client is told to watch goal 55 (45/10, exact bytes)
    // and not goal 77, whose quest is only offered.
    let mut watch = vec![45, 0, 10];
    watch.extend(55u32.to_le_bytes());
    watch.extend(11u32.to_le_bytes());
    watch.extend(b"onTestEvent");
    let out = game
        .handle_in_world(&claim_packet(99), &character, 3_001)
        .unwrap();
    assert!(!out.changed);
    assert_eq!(out.replies, vec![watch]);
    assert_eq!(quest_state(&game), before);

    // A claim for the goal of the merely offered quest changes nothing.
    let out = game
        .handle_in_world(&claim_packet(77), &character, 3_002)
        .unwrap();
    assert!(!out.changed && out.replies.is_empty());
    assert_eq!(quest_state(&game), before);

    // Malformed claims are errors and change nothing.
    for bad in [
        claim_packet(0),
        claim_packet(-1),
        claim_packet(i32::MIN),
        vec![45, 0, 7, 55, 0, 0],
        vec![45, 0, 7, 55, 0, 0, 0, 0],
        vec![45, 0, 7],
    ] {
        assert!(game.handle_in_world(&bad, &character, 3_003).is_err());
        assert_eq!(quest_state(&game), before);
    }

    // The real claim advances the goal once: update and completion are sent, the quest
    // waits for its turn-in and nothing is paid.
    let out = game
        .handle_in_world(&claim_packet(55), &character, 3_004)
        .unwrap();
    assert!(out.changed);
    assert!(
        out.replies
            .iter()
            .any(|p| p.starts_with(&[49, 0, 9, 0, 0, 0]))
    );
    assert!(
        out.replies
            .iter()
            .any(|p| p.starts_with(&[49, 0, 10, 0, 0, 0]))
    );
    assert!(!out.replies.iter().any(|p| p.starts_with(&[45, 0, 10])));
    let state = quest_state(&game);
    assert_eq!(state.quests[&7001].counts, vec![1]);
    assert_eq!(
        state.quests[&7001].status,
        crate::quest_engine::QuestStatus::Active
    );
    assert_eq!(state.quests[&7002].counts, vec![0]);

    // A repeat, and a claim for the other quest's goal, change nothing.
    for id in [55, 77] {
        let out = game
            .handle_in_world(&claim_packet(id), &character, 3_005)
            .unwrap();
        assert!(!out.changed && out.replies.is_empty());
        assert_eq!(quest_state(&game), state);
    }
}
#[test]
fn a_look_goal_claim_advances_only_an_open_goal_of_an_active_quest() {
    let (mut game, character) = quest_game();
    game.npc_quests = Some(client_goal_engine());
    assert!(engine_mut(&mut game).offer(7003, 1).changed);
    assert!(engine_mut(&mut game).accept(7003).changed);
    let before = quest_state(&game);

    // The client receives the recovered 45/6 actor form: id, GUID, vertical
    // offset in tenths, range, and angle in degrees.
    let out = game
        .handle_in_world(&claim_packet(99), &character, 5_000)
        .unwrap();
    let mut watch = vec![45, 0, 6];
    watch.extend(73u32.to_le_bytes());
    watch.extend(0x1122_3344_5566_7788u64.to_le_bytes());
    watch.extend(15i32.to_le_bytes()); // 009be130 scales the i32 offset by 0.1
    watch.extend(12u32.to_le_bytes());
    watch.extend(45u32.to_le_bytes());
    assert_eq!(watch.len(), 27);
    assert_eq!(out.replies, vec![watch]);
    assert_eq!(quest_state(&game), before);

    let out = game
        .handle_in_world(&claim_packet(73), &character, 5_001)
        .unwrap();
    assert!(out.changed);
    assert!(
        out.replies
            .iter()
            .any(|p| p.starts_with(&[49, 0, 10, 0, 0, 0]))
    );
    assert!(!out.replies.iter().any(|p| p.starts_with(&[50, 0, 1])));
    let state = quest_state(&game);
    assert_eq!(state.quests[&7003].counts, vec![1]);
    assert_eq!(
        state.quests[&7003].status,
        crate::quest_engine::QuestStatus::Active
    );

    // Claim replay changes nothing.
    let replay = game
        .handle_in_world(&claim_packet(73), &character, 5_002)
        .unwrap();
    assert!(!replay.changed && replay.replies.is_empty());
    assert_eq!(quest_state(&game), state);
}

#[test]
fn movement_registers_the_ordered_goal_it_opens() {
    let (mut game, mut character) = quest_game();
    game.npc_quests = Some(client_goal_engine());
    assert!(engine_mut(&mut game).offer(7004, 1).changed);
    assert!(engine_mut(&mut game).accept(7004).changed);
    character.position = [5.0, 0.0, 5.0];

    // Walking onto the first goal opens the journal-close goal in the same response:
    // 45/10 UIEvent, client id 88, length 14 and the exact event bytes.
    let out = game.on_location(&character, 7_000).unwrap();
    assert!(out.changed);
    assert_eq!(quest_state(&game).quests[&7004].counts, vec![1, 0]);
    let mut watch = vec![45, 0, 10];
    watch.extend(88u32.to_le_bytes());
    watch.extend(14u32.to_le_bytes());
    watch.extend(b"onCloseJournal");
    assert_eq!(watch.len(), 25);
    assert!(
        out.replies.iter().any(|reply| reply == &watch),
        "movement did not register the opened 45/10 goal"
    );

    // Standing here again must not register it twice.
    let replay = game.on_location(&character, 7_001).unwrap();
    assert!(
        !replay
            .replies
            .iter()
            .any(|reply| reply.starts_with(&[45, 0, 10]))
    );
}

#[test]
fn a_goal_that_opens_again_is_rearmed_not_registered_twice() {
    let (mut game, character) = quest_game();
    game.npc_quests = Some(client_goal_engine());
    let watching = |out: &Outcome, sub: u8| {
        out.replies
            .iter()
            .filter(|p| p.starts_with(&[45, 0, sub]))
            .count()
    };
    assert!(engine_mut(&mut game).offer(7002, 1).changed);
    assert!(engine_mut(&mut game).accept(7002).changed);

    // 45/11 FirstMovement with the mouse flag, once.
    let out = game
        .handle_in_world(&claim_packet(99), &character, 4_000)
        .unwrap();
    let mut first = vec![45, 0, 11];
    first.extend(77u32.to_le_bytes());
    first.push(1);
    assert_eq!(out.replies, vec![first]);
    let out = game
        .handle_in_world(&claim_packet(99), &character, 4_001)
        .unwrap();
    assert!(out.replies.is_empty());

    // Abandon and take it again: the client still holds the registration, so it gets a
    // 45/9 re-arm and no second 45/11.
    assert!(engine_mut(&mut game).abandon(7002).changed);
    let out = game
        .handle_in_world(&claim_packet(99), &character, 4_002)
        .unwrap();
    assert!(out.replies.is_empty());
    assert!(engine_mut(&mut game).offer(7002, 1).changed);
    assert!(engine_mut(&mut game).accept(7002).changed);
    let out = game
        .handle_in_world(&claim_packet(99), &character, 4_003)
        .unwrap();
    let mut rearm = vec![45, 0, 9];
    rearm.extend(77u32.to_le_bytes());
    assert_eq!(out.replies, vec![rearm]);
    assert_eq!(watching(&out, 11), 0);
}

fn tracked_pins(out: &Outcome) -> Vec<Vec<u8>> {
    out.replies
        .iter()
        .filter(|packet| packet.starts_with(&[47, 0, 14]))
        .cloned()
        .collect()
}

fn select_packet(quest_id: u32) -> Vec<u8> {
    let mut select = vec![47, 0, 12];
    select.extend((quest_id as i32).to_le_bytes());
    select
}

#[test]
fn take_me_there_pin_follows_the_selected_quest() {
    let (mut game, character) = offer_test_game(false);
    let mut accept = vec![49, 0, 2, 0, 0, 0];
    accept.extend((INTRODUCE_YOURSELF_ID as i32).to_le_bytes());
    accept.push(1);
    let out = game.handle_in_world(&accept, &character, 1_005).unwrap();
    // Accepting does not select the quest, and nothing is pinned for an unselected one.
    assert_eq!(
        game.npc_quests.as_ref().unwrap().state().selected_quest_id,
        None
    );
    assert!(tracked_pins(&out).is_empty());

    // Selecting the active quest pins its talk objective once: Ricky Danger's spot.
    let select = select_packet(INTRODUCE_YOURSELF_ID);
    let first = game.handle_in_world(&select, &character, 1_006).unwrap();
    let mut expected = vec![47, 0, 14, 1];
    expected.extend(INTRODUCE_YOURSELF_ID.to_le_bytes());
    expected.extend(0u32.to_le_bytes());
    expected.extend(1u32.to_le_bytes());
    expected.extend(9_500_000_254u64.to_le_bytes());
    expected.extend(0u32.to_le_bytes());
    for float in [-1909.511f32, -44.3937, 462.83, 1.0] {
        expected.extend(float.to_le_bytes());
    }
    expected.extend(0u32.to_le_bytes());
    assert_eq!(tracked_pins(&first), vec![expected]);

    // Repeating the selection, or any unrelated packet, does not resend it.
    let again = game.handle_in_world(&select, &character, 1_007).unwrap();
    assert!(tracked_pins(&again).is_empty());

    // A forged id is refused by the engine, so the pin stays where it was.
    let forged = select_packet(999_999);
    let out = game.handle_in_world(&forged, &character, 1_008).unwrap();
    assert!(tracked_pins(&out).is_empty());

    // Leaving the pin's zone clears it; coming back sends it again.
    let away = CharacterState {
        zone_id: 9_600,
        ..character.clone()
    };
    let out = game.handle_in_world(&select, &away, 1_009).unwrap();
    assert_eq!(tracked_pins(&out), vec![vec![47, 0, 14, 0]]);
    let out = game.handle_in_world(&select, &character, 1_010).unwrap();
    assert_eq!(tracked_pins(&out).len(), 1);
    assert_eq!(tracked_pins(&out)[0].len(), 48);
}

#[test]
fn kill_objective_pins_its_field_enemy() {
    let (mut game, character) = offer_test_game(false);
    let engine = game.npc_quests.as_mut().unwrap();
    // "Defeat the Hooligan Delinquents" (kind 930050), a real enabled quest.
    assert!(engine.dev_mark_offered(3_030_007));
    assert!(engine.accept(3_030_007).changed);
    let content = bundled_content();
    let file: serde_json::Value =
        serde_json::from_slice(&std::fs::read(content.join("field_enemies.json")).unwrap())
            .unwrap();
    let home = file["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|enemy| enemy["definition_id"] == 930_050)
        .min_by_key(|enemy| enemy["guid"].as_u64().unwrap())
        .unwrap();
    let out = game
        .handle_in_world(&select_packet(3_030_007), &character, 1_005)
        .unwrap();
    let pins = tracked_pins(&out);
    assert_eq!(pins.len(), 1);
    assert_eq!(
        pins[0][16..24],
        home["guid"].as_u64().unwrap().to_le_bytes()
    );
    let position: [f32; 3] = serde_json::from_value(home["position"].clone()).unwrap();
    for (axis, value) in position.iter().enumerate() {
        assert_eq!(pins[0][28 + 4 * axis..32 + 4 * axis], value.to_le_bytes());
    }
}

#[test]
fn an_objective_without_a_known_place_sends_no_pin() {
    let (mut game, character) = offer_test_game(false);
    let engine = game.npc_quests.as_mut().unwrap();
    // Kind 930096 has no field enemy, so the kill step has no known place.
    assert!(engine.dev_mark_offered(3_110_002));
    assert!(engine.accept(3_110_002).changed);
    let out = game
        .handle_in_world(&select_packet(3_110_002), &character, 1_005)
        .unwrap();
    assert!(out.changed);
    assert!(tracked_pins(&out).is_empty());
    let again = game
        .handle_in_world(&select_packet(3_110_002), &character, 1_006)
        .unwrap();
    assert!(tracked_pins(&again).is_empty());
}

#[test]
fn native_save_samantha_uses_localized_quest_offer_and_reply_state() {
    let content = bundled_content();
    let reference = reference();
    let mut game = Gameplay::load(&reference, None).unwrap();
    game.profiles = Some(Profiles::load(&reference, Some(native_test_profiles())).unwrap());
    game.bind_npc_quest_catalogue(&content, 1).unwrap();
    let eligibility = game.npc_quests.as_ref().unwrap().eligibility();
    assert!(eligibility.member);
    assert_eq!(eligibility.member_quest_limit, 100);
    let active = game.profiles.as_ref().unwrap().active();
    let active_job = game
        .npc_quests
        .as_ref()
        .unwrap()
        .job_profile_ids()
        .find(|(_, profile)| *profile == active)
        .unwrap()
        .0;
    assert_eq!(eligibility.job_levels.len(), 1);
    assert_eq!(
        eligibility.job_levels.get(active_job),
        Some(&eligibility.player_level)
    );

    game.set_game_rules(crate::game_rules::GameRuleSet::new(
        crate::game_rules::MembershipState::new(false, None),
        crate::game_rules::RulePolicy::default(),
    ));
    assert!(!game.npc_quests.as_ref().unwrap().eligibility().member);
    game.set_game_rules(GameRuleSet::default());
    assert!(game.npc_quests.as_ref().unwrap().eligibility().member);

    let npc_file: serde_json::Value = serde_json::from_slice(
        &std::fs::read(content.join("npcs/cobblestone_village.json")).unwrap(),
    )
    .unwrap();
    let samantha = npc_file["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["npc_id"].as_u64() == Some(SAMANTHA_ID))
        .unwrap();
    let position: [f32; 3] = serde_json::from_value(samantha["position"].clone()).unwrap();
    let mut character = CharacterState {
        guid: 1,
        zone_id: 0,
        position,
        rotation: [1.0, 0.0, 0.0],
    };

    let menu = game
        .handle_in_world(&packet_for_npc(SAMANTHA_ID), &character, 1_000)
        .unwrap();
    let menu_packet = menu
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 9, 0]))
        .expect("Samantha returns a radial menu");
    let option_count = i32::from_le_bytes(menu_packet[13..17].try_into().unwrap());
    assert_eq!(
        option_count, 1,
        "new quests are offered after selecting Talk"
    );

    let talk = game
        .handle_in_world(&select_npc_option(SAMANTHA_ID, 1), &character, 1_001)
        .unwrap();
    assert!(talk.replies.iter().any(|p| p.starts_with(&[26, 0, 3, 0])));
    let first_offer = game
        .handle_in_world(&choose_quest_row(), &character, 1_001)
        .unwrap();
    let first_offer_packet = first_offer
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[49, 0, 1, 0]))
        .expect("selecting Talk sends QuestInfo 49/1 for the eligible quest");
    let offer_fields = npc_interaction::decode_quest_offer_49_1(first_offer_packet).unwrap();
    assert_eq!(offer_fields.quest_id, INTRODUCE_YOURSELF_ID);
    assert_eq!(offer_fields.intro_text_string_id, 94246);
    assert_eq!(offer_fields.name_string_id, 94247);
    assert_eq!(offer_fields.description_string_id, 94248);
    assert_eq!(offer_fields.field_5, 94247);
    assert!(!offer_fields.members_only);

    let mut decline = vec![49, 0, 2, 0, 0, 0];
    decline.extend(INTRODUCE_YOURSELF_ID.to_le_bytes());
    decline.push(0);
    let declined = game.handle_in_world(&decline, &character, 1_002).unwrap();
    assert!(
        !declined
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 3, 0]))
    );
    assert_eq!(
        game.npc_quests.as_ref().unwrap().state().pending_offer,
        None
    );
    assert_eq!(
        game.npc_quests
            .as_ref()
            .unwrap()
            .progress(INTRODUCE_YOURSELF_ID),
        None
    );

    game.handle_in_world(&packet_for_npc(SAMANTHA_ID), &character, 1_003)
        .unwrap();
    game.handle_in_world(&select_npc_option(SAMANTHA_ID, 1), &character, 1_004)
        .unwrap();
    let offer = game
        .handle_in_world(&choose_quest_row(), &character, 1_004)
        .unwrap();
    assert!(
        offer
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 1, 0]))
    );

    let mut accept = vec![49, 0, 2, 0, 0, 0];
    accept.extend((INTRODUCE_YOURSELF_ID as i32).to_le_bytes());
    accept.push(1);
    let accepted = game.handle_in_world(&accept, &character, 1_005).unwrap();
    assert!(
        accepted
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 3, 0]))
    );
    assert_eq!(
        game.npc_quests.as_ref().unwrap().state().pending_offer,
        None
    );
    assert_eq!(
        game.npc_quests
            .as_ref()
            .unwrap()
            .progress(INTRODUCE_YOURSELF_ID)
            .unwrap()
            .status,
        crate::quest_engine::QuestStatus::Active
    );
    let direct_talk = game
        .npc_quests
        .as_ref()
        .unwrap()
        .clone()
        .on_event(crate::quest_engine::QuestEvent::TalkToNpc {
            npc_id: 9_500_000_254,
        })
        .unwrap();
    assert!(direct_talk.outbound.iter().any(|message| matches!(
        message,
        crate::quest_engine::OutboundMessage::Quest(
            crate::quest_engine::QuestMessage::QuestEnd { quest_id }
        ) if *quest_id == INTRODUCE_YOURSELF_ID
    )));

    // W92 now requires the explicit referral target, rather than a hollow
    // quest's accepting conversation. Exercise the real target route.
    let target_id = 9_500_000_254;
    let target = npc_file["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["npc_id"].as_u64() == Some(target_id))
        .unwrap();
    character.position = serde_json::from_value(target["position"].clone()).unwrap();
    game.update_npc_visibility(&character).unwrap();
    game.handle_in_world(&packet_for_npc(target_id), &character, 1_006)
        .unwrap();
    let talk = game
        .handle_in_world(&select_npc_option(target_id, 1), &character, 1_007)
        .unwrap();
    // The end screen waits behind a dialog choice, like the offer.
    assert!(
        !talk
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 13, 0]))
    );
    let chosen = game
        .handle_in_world(&choose_quest_row(), &character, 1_007)
        .unwrap();
    assert!(
        chosen
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 13, 0, 0, 0])
                && packet.len() == 99
                && u64::from_le_bytes(packet[6..14].try_into().unwrap()) == target_id
                && u32::from_le_bytes(packet[14..18].try_into().unwrap()) == INTRODUCE_YOURSELF_ID)
    );
    let dialogue = talk
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 3, 0]))
        .expect("referral target returns a ShowDialog packet");
    assert!(npc_interaction::decode_dialogue_26_3(dialogue).is_ok());

    let mut end_reply = vec![49, 0, 14, 0, 0, 0];
    end_reply.extend(INTRODUCE_YOURSELF_ID.to_le_bytes());
    let completed = game.handle_in_world(&end_reply, &character, 1_008).unwrap();
    assert!(
        completed
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 4, 0]))
    );
    assert_eq!(completed.replies.last(), Some(&vec![26, 0, 4, 0]));
    assert_eq!(
        game.state().npc_quests.unwrap().quests[&INTRODUCE_YOURSELF_ID].status,
        crate::quest_engine::QuestStatus::Completed
    );

    let inventory = game.inventory.clone();
    game.profiles
        .as_mut()
        .unwrap()
        .select(1, &inventory)
        .unwrap();
    game.profiles.as_mut().unwrap().add_experience(75).unwrap();
    game.refresh_npc_quest_eligibility(current_unix_seconds());
    assert_eq!(
        game.npc_quests
            .as_ref()
            .unwrap()
            .eligibility()
            .job_levels
            .get("adventurer"),
        Some(&2)
    );
}

#[test]
fn developer_quest_offer_variants_match_exact_diagnostic_bytes() {
    let (mut game, character) = offer_test_game(true);
    let dialogue = expected_samantha_dialogue();
    let offer = |prefix, intro| expected_quest_offer(prefix, intro);
    let cases: [(&str, Vec<Vec<u8>>); 7] = [
        ("v0", vec![dialogue.clone(), offer(0, 94_246)]),
        ("v1", vec![offer(0, 94_246)]),
        ("v2", vec![offer(0, 0)]),
        ("v3", vec![offer(0, 94_247)]),
        ("v4", vec![expected_blank_dialogue(), offer(0, 94_246)]),
        ("v5", vec![offer(1, 94_246)]),
        ("v6", vec![offer(0, 0), offer(0, 94_246)]),
    ];

    for (index, (variant, expected)) in cases.into_iter().enumerate() {
        let result = game
            .handle_in_world(
                &offer_chat(&format!("realm offer {INTRODUCE_YOURSELF_ID} {variant}")),
                &character,
                1_010 + index as u64,
            )
            .unwrap();
        let actual = result
            .replies
            .iter()
            .filter(|packet| {
                packet.starts_with(&[26, 0, 3, 0]) || packet.starts_with(&[49, 0, 1, 0])
            })
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "variant {variant}");
    }
}

/// `realm here` (dev tools) reports the zone, position and nearest actors;
/// Studio's /api/command runs the same chat path.
#[test]
fn developer_here_reports_position_and_neighbours() {
    let (mut game, character) = offer_test_game(true);
    let result = game
        .handle_in_world(&offer_chat("realm here"), &character, 1_000)
        .unwrap();
    let text = String::from_utf8_lossy(&result.replies.concat()).to_string();
    assert!(text.contains("Zone 0 at x"), "{text}");
    assert!(text.contains(" m, "), "lists nearby actors: {text}");
    assert_eq!(
        crate::chat::local_request("realm here"),
        offer_chat("realm here")
    );
}

#[test]
fn developer_quest_offer_command_requires_dev_tools() {
    let (mut game, character) = offer_test_game(false);
    let result = game
        .handle_in_world(
            &offer_chat(&format!("realm offer {INTRODUCE_YOURSELF_ID} v1")),
            &character,
            1_010,
        )
        .unwrap();
    assert!(
        !result
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 1, 0]))
    );
}

fn spawned_actors(packets: &[Vec<u8>]) -> Vec<(u64, [f32; 3])> {
    packets
        .iter()
        .filter(|p| p.starts_with(&[35, 0, 2, 0]))
        .map(|p| {
            let f = |o: usize| f32::from_le_bytes(p[o..o + 4].try_into().unwrap());
            (
                u64::from_le_bytes(p[4..12].try_into().unwrap()),
                [f(37), f(41), f(45)],
            )
        })
        .collect()
}

/// Overworld battle: clicking the walking Troll Summoner shows the start
/// screen (41/114 not launched + 41/107 ready). Go (41/108) needs a combat
/// job, then zones into the instanced arena instead of fighting in place.
#[test]
fn battle_go_requires_a_combat_job_and_zones_into_the_arena() {
    const SPAWNER: u64 = 100_000_096_000;
    let reference = reference();
    let mut game = Gameplay::load(&reference, None).unwrap();
    let character = CharacterState {
        guid: 1,
        zone_id: 0,
        position: [-440.0, 0.8, 1292.0],
        rotation: [1.0, 0.0, 0.0],
    };
    let mut click = vec![26, 0, 8, 0];
    click.extend(SPAWNER.to_le_bytes());
    click.push(0);
    let mut go = vec![41, 0, 108, 0];
    go.extend([0; 8]);

    let screen = game.handle_in_world(&click, &character, 1_000).unwrap();
    let details = screen
        .replies
        .iter()
        .find(|p| p.starts_with(&[41, 0, 114, 0]))
        .expect("start screen");
    // The preview bundle lists the four prizes (Item entries).
    for item in [10_482_i32, 3_023, 75_354, 11_002] {
        assert!(
            details.windows(4).any(|w| w == item.to_le_bytes()),
            "prize {item} on the start screen"
        );
    }
    assert!(
        screen
            .replies
            .iter()
            .any(|p| p.starts_with(&[41, 0, 107, 0]))
    );
    let refused = game.handle_in_world(&go, &character, 2_000).unwrap();
    assert!(refused.zoning.is_none(), "no combat job: no battle");
    assert!(
        refused
            .replies
            .iter()
            .any(|p| p.starts_with(&[41, 0, 114, 0])),
        "the start screen stays up so Switch Jobs is reachable"
    );

    let inventory = game.inventory.clone();
    game.profiles
        .as_mut()
        .unwrap()
        .select(43, &inventory)
        .unwrap();
    game.handle_in_world(&click, &character, 3_000).unwrap();
    let entered = game.handle_in_world(&go, &character, 4_000).unwrap();
    let zone = entered.zoning.expect("Go zones into the battle arena");
    assert_eq!(zone.name, "sg_random_encounter_clearing");
    // Go without a fresh start screen does nothing.
    assert!(
        game.handle_in_world(&go, &character, 5_000)
            .unwrap()
            .zoning
            .is_none()
    );
}

/// Inside the arena only the summoner stands at first. Hitting it below
/// 75/50/25% calls in two small trolls each with the cast animation and
/// the troll spawn puff (35/16); it takes no damage until they are down;
/// defeating it wins the battle and reports the quest kill (930096).
#[test]
fn troll_summoner_battle_summons_waves_and_is_won_on_the_summoner() {
    const SUMMONER: u64 = 100_000_096_001;
    let reference = reference();
    let mut game = Gameplay::load(&reference, None).unwrap();
    let encounter = game.encounter.as_mut().unwrap();
    let world = CharacterState {
        guid: 1,
        zone_id: 0,
        position: [-440.0, 0.8, 1292.0],
        rotation: [1.0, 0.0, 0.0],
    };
    let begun = encounter.begin(96, world, 43).unwrap();
    assert_eq!(begun.zoning.unwrap().name, "sg_random_encounter_clearing");
    let ready = encounter.on_zone_ready(43, 1_000).unwrap();
    let first = spawned_actors(&ready.replies);
    assert_eq!(first.len(), 1, "only the summoner at first");
    assert_eq!(first[0].0, SUMMONER);

    let at = first[0].1;
    let mut now = 2_000;
    let mut waves = Vec::new();
    let mut defeated = Vec::new();
    let mut won = false;
    for _ in 0..40 {
        now += 700;
        let out = encounter.attack(SUMMONER, at, 43, 0, now).unwrap();
        defeated.extend(out.defeated.iter().copied());
        won |= out.replies.iter().any(|p| p.starts_with(&[39, 0, 18]));
        if !out.defeated.is_empty() {
            let reward = out.rewards.first().expect("battle reward");
            assert_eq!(reward.coins, 25);
            assert_eq!(reward.job_xp, 50, "battles level the combat job");
            assert!(
                reward.items.len() == 1
                    && [10_482, 3_023, 75_354, 11_002].contains(&reward.items[0].0),
                "one prize from the wheel: {:?}",
                reward.items
            );
            // The loot wheel lands on the same prize (39/45, by name id).
            let name = [
                (10_482, 6_666),
                (3_023, 422_907),
                (75_354, 131_098),
                (11_002, 422_909),
            ]
            .iter()
            .find(|(item, _)| *item == reward.items[0].0)
            .unwrap()
            .1;
            let wheel = out
                .replies
                .iter()
                .find(|p| p.starts_with(&[39, 0, 45]))
                .expect("loot wheel landing");
            assert!(wheel.windows(4).any(|w| w == (name as i32).to_le_bytes()));
            break;
        }
        let called = spawned_actors(&out.replies);
        if called.is_empty() {
            continue;
        }
        assert!(
            out.replies.iter().any(|p| p.starts_with(&[35, 0, 16, 0])),
            "spawn puff"
        );
        assert!(
            out.replies
                .iter()
                .any(|p| p.starts_with(&[35, 0, 8, 0]) && p[4..12] == SUMMONER.to_le_bytes()),
            "summon cast"
        );
        waves.push(called.len());
        // Shielded while the wave stands: a hit does no damage.
        now += 700;
        let shielded = encounter.attack(SUMMONER, at, 43, 0, now).unwrap();
        let hit = shielded
            .replies
            .iter()
            .find(|p| p.starts_with(&[32, 0, 7, 0]))
            .expect("the hit is still shown");
        assert_eq!(
            &hit[28..32],
            &0_u32.to_le_bytes(),
            "no damage while shielded"
        );
        for (guid, position) in called {
            for _ in 0..4 {
                now += 700;
                let out = encounter.attack(guid, position, 43, 0, now).unwrap();
                defeated.extend(out.defeated.iter().copied());
            }
        }
    }
    assert_eq!(waves, vec![2, 2, 2], "summon waves at 75/50/25%");
    assert_eq!(
        defeated.iter().filter(|&&d| d == 930_097).count(),
        6,
        "all minions"
    );
    assert_eq!(
        defeated.last(),
        Some(&930_096),
        "summoner kill id for the quest"
    );
    assert!(won, "game over (won) on the summoner");

    // The save right after the win loads again (run 150132 refused it:
    // the battle id was recorded in `rewarded`), back in the world.
    let saved = game.state();
    assert!(
        !saved.encounter.as_ref().unwrap().rewarded.contains(&96),
        "battles are repeatable"
    );
    let mut reloaded = Gameplay::load(&reference, Some(saved)).expect("save after a win loads");
    assert!(
        !reloaded.encounter.as_ref().unwrap().active(),
        "returned to the world"
    );
    // The character, still saved in the arena, goes back where it started.
    let back = reloaded
        .abandon_battle_for_reconnect()
        .expect("battle start location");
    assert_eq!(back.zone_id, 0);
    assert!(reloaded.supports_location(&back));
    assert!(
        reloaded.abandon_battle_for_reconnect().is_none(),
        "only once"
    );
}

/// Alone, the summoner runs from a chasing player for about 4 s of
/// running, then rests (com_stunned, no attacks) for about 3 s; it does not
/// run from a player who stays away (R).
#[test]
fn lone_troll_summoner_runs_then_rests_without_attacking() {
    const SUMMONER: u64 = 100_000_096_001;
    let reference = reference();
    let mut game = Gameplay::load(&reference, None).unwrap();
    let encounter = game.encounter.as_mut().unwrap();
    let world = CharacterState {
        guid: 1,
        zone_id: 0,
        position: [-440.0, 0.8, 1292.0],
        rotation: [1.0, 0.0, 0.0],
    };
    encounter.begin(96, world, 43).unwrap();
    let ready = encounter.on_zone_ready(43, 1_000).unwrap();
    let mut troll = spawned_actors(&ready.replies)[0].1;
    let summoner_moves = |replies: &[Vec<u8>]| {
        replies
            .iter()
            .filter(|p| p.starts_with(&[125, 0]) && p[2..10] == SUMMONER.to_le_bytes())
            .map(|p| {
                let f = |o: usize| f32::from_le_bytes(p[o..o + 4].try_into().unwrap());
                [f(10), f(14), f(18)]
            })
            .collect::<Vec<_>>()
    };

    // A player 30 m away is watched, not run from.
    let far = [troll[0], troll[1], troll[2] - 30.0];
    for step in 1..=20_u64 {
        let out = encounter.tick(far, 43, 1_000 + step * 100).unwrap();
        assert!(
            summoner_moves(&out.replies).is_empty(),
            "no running from afar"
        );
    }

    // A player chasing 3 m behind it: it runs, then rests without attacking.
    let mut moves = 0;
    let mut rested_at = None;
    let mut hits_while_resting = 0;
    for step in 1..=80_u64 {
        let now = 3_000 + step * 100;
        let player = [troll[0], troll[1], troll[2] - 3.0];
        let out = encounter.tick(player, 43, now).unwrap();
        for at in summoner_moves(&out.replies) {
            moves += 1;
            troll = at;
        }
        for p in &out.replies {
            if p.starts_with(&[35, 0, 8, 0])
                && p[4..12] == SUMMONER.to_le_bytes()
                && p[12..16] == 1401_i32.to_le_bytes()
            {
                rested_at.get_or_insert(now);
            }
            if rested_at.is_some_and(|at| now < at + 3_000) && p.starts_with(&[32, 0, 7, 0]) {
                hits_while_resting += 1;
            }
        }
    }
    let rested_at = rested_at.expect("the summoner rests");
    assert!(
        (6_500..=7_600).contains(&rested_at),
        "rest after ~4 s of running ({rested_at})"
    );
    assert!(moves >= 20, "it ran ({moves} moves)");
    assert_eq!(hits_while_resting, 0, "no attacks while resting");
}

/// The Medic (profile 11) bar carries Incision, Triage and First Aid
/// (data/content/job_abilities.json). In a battle Triage strikes for 3.2x
/// the basic attack and heals 254, spending all 100 energy; First Aid
/// heals 250 on a 15 s cooldown; refusals use the client's messages.
#[test]
fn medic_job_abilities_strike_heal_and_refuse() {
    const SUMMONER: u64 = 100_000_096_001;
    let reference = reference();
    let mut game = Gameplay::load(&reference, None).unwrap();
    let bar = game.abilities.as_ref().unwrap().activation(11, 1).unwrap();
    let definitions: Vec<i32> = bar
        .iter()
        .filter(|p| p.starts_with(&[36, 0, 13, 0]))
        .map(|p| i32::from_le_bytes(p[4..8].try_into().unwrap()))
        .collect();
    assert_eq!(definitions.len(), 3, "basic, Triage, First Aid");
    assert_eq!(&definitions[1..], &[4899, 4901]);
    let toolbar = bar.last().unwrap();
    assert!(toolbar.starts_with(&[36, 0, 5, 0]));
    assert_eq!(
        toolbar.len(),
        4 + 8 + 3 * 41 + 5 * 4,
        "three slots, five empty"
    );

    let triage = game
        .abilities
        .as_ref()
        .unwrap()
        .job_slot(11, 1, 1)
        .unwrap()
        .clone();
    let first_aid = game
        .abilities
        .as_ref()
        .unwrap()
        .job_slot(11, 2, 1)
        .unwrap()
        .clone();
    let encounter = game.encounter.as_mut().unwrap();
    let world = CharacterState {
        guid: 1,
        zone_id: 0,
        position: [-440.0, 0.8, 1292.0],
        rotation: [1.0, 0.0, 0.0],
    };
    encounter.begin(96, world, 11).unwrap();
    let ready = encounter.on_zone_ready(11, 1_000).unwrap();
    let at = spawned_actors(&ready.replies)[0].1;
    let damage = |out: &EncounterOutcome| {
        out.replies
            .iter()
            .find(|p| p.starts_with(&[32, 0, 7, 0]))
            .map(|p| u32::from_le_bytes(p[28..32].try_into().unwrap()))
    };
    let refusal = |out: &EncounterOutcome| {
        out.replies
            .iter()
            .find(|p| p.starts_with(&[36, 0, 1, 0]))
            .map(|p| i32::from_le_bytes(p[4..8].try_into().unwrap()))
    };

    // Self-centred request (guid 1) picks the summoner in reach.
    let hit = encounter.job_ability(&triage, 1, at, 11, 2_000).unwrap();
    assert_eq!(damage(&hit), Some(240), "75 x 3.2");
    assert!(
        hit.replies.iter().any(|p| p.starts_with(&[35, 0, 35, 0])),
        "heal number"
    );
    assert!(
        hit.replies.iter().any(|p| p.starts_with(&[38, 0, 13, 0])),
        "energy spent"
    );
    let again = encounter.job_ability(&triage, 1, at, 11, 3_500).unwrap();
    assert_eq!(refusal(&again), Some(285), "not enough energy");

    let healed = encounter.job_ability(&first_aid, 1, at, 11, 3_600).unwrap();
    assert!(healed.replies.iter().any(|p| p.starts_with(&[38, 0, 1, 0])));
    let soon = encounter.job_ability(&first_aid, 1, at, 11, 5_000).unwrap();
    assert_eq!(refusal(&soon), Some(287), "cooldown");

    // Out of reach: no target, nothing spent.
    let far = [at[0] + 20.0, at[1], at[2]];
    let missed = encounter
        .job_ability(&triage, SUMMONER, far, 11, 60_000)
        .unwrap();
    assert!(matches!(refusal(&missed), Some(285) | Some(288)));
}

/// The overworld Troll Summoner walks a small loop around its home while
/// a player is near enough to see it, and stays put otherwise.
#[test]
fn battle_marker_wanders_only_when_seen() {
    const SPAWNER: u64 = 100_000_096_000;
    let home = [-440.0_f32, 0.8, 1290.0];
    let reference = reference();
    let mut game = Gameplay::load(&reference, None).unwrap();
    let encounter = game.encounter.as_mut().unwrap();
    let moves = |out: &EncounterOutcome| {
        out.replies
            .iter()
            .filter(|p| p.starts_with(&[125, 0]) && p[2..10] == SPAWNER.to_le_bytes())
            .map(|p| {
                let f = |o: usize| f32::from_le_bytes(p[o..o + 4].try_into().unwrap());
                [f(10), f(14), f(18)]
            })
            .collect::<Vec<_>>()
    };
    let far = [home[0] + 500.0, home[1], home[2]];
    for step in 1..=30_u64 {
        assert!(moves(&encounter.tick(far, 11, step * 100).unwrap()).is_empty());
    }
    let near = [home[0] + 20.0, home[1], home[2]];
    let mut seen = Vec::new();
    for step in 31..=200_u64 {
        seen.extend(moves(&encounter.tick(near, 11, step * 100).unwrap()));
    }
    assert!(seen.len() > 20, "it walks ({} moves)", seen.len());
    // Close by, it stops and faces the player: no further walking.
    let last = *seen.last().unwrap();
    let close = [last[0] + 3.0, last[1], last[2]];
    encounter.tick(close, 11, 20_100).unwrap();
    for step in 202..=260_u64 {
        assert!(
            moves(&encounter.tick(close, 11, step * 100).unwrap()).is_empty(),
            "stands still near the player"
        );
    }
    for p in &seen {
        let d = ((p[0] - home[0]).powi(2) + (p[2] - home[2]).powi(2)).sqrt();
        assert!(d <= 5.01, "stays near home ({d})");
        assert_eq!(p[1], home[1], "keeps its home height");
    }
}

/// With terrain heights the marker walks on the slope and still reaches
/// every waypoint (its goals are snapped to the ground too).
#[test]
fn battle_marker_follows_the_terrain() {
    const SPAWNER: u64 = 100_000_096_000;
    let home = [-440.0_f32, 0.8, 1290.0];
    let slope = |x: f32| 0.8 + (x - home[0]) * 0.5;
    let mut grid = crate::terrain::HeightGrid::new(
        [home[0] - 50.0, home[2] - 50.0],
        [home[0] + 50.0, home[2] + 50.0],
        2.0,
    )
    .unwrap();
    let v = |dx: f32, dz: f32| [home[0] + dx, slope(home[0] + dx), home[2] + dz];
    grid.add_triangle([v(-50.0, -50.0), v(50.0, -50.0), v(50.0, 50.0)]);
    grid.add_triangle([v(-50.0, -50.0), v(50.0, 50.0), v(-50.0, 50.0)]);
    let dir = std::env::temp_dir().join(format!("osfr-wander-hgt-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("zone_0.hgt"), grid.to_bytes()).unwrap();
    let terrain = Arc::new(crate::terrain::Terrain::load(&dir).unwrap());

    let reference = reference();
    let mut game = Gameplay::load(&reference, None).unwrap();
    game.set_terrain(Some(terrain));
    let encounter = game.encounter.as_mut().unwrap();
    let near = [home[0] + 20.0, home[1], home[2]];
    let mut seen = Vec::new();
    for step in 1..=400_u64 {
        let out = encounter.tick(near, 11, step * 100).unwrap();
        seen.extend(
            out.replies
                .iter()
                .filter(|p| p.starts_with(&[125, 0]) && p[2..10] == SPAWNER.to_le_bytes())
                .map(|p| {
                    let f = |o: usize| f32::from_le_bytes(p[o..o + 4].try_into().unwrap());
                    [f(10), f(14), f(18)]
                }),
        );
    }
    assert!(seen.len() > 20, "it walks ({} moves)", seen.len());
    for p in &seen {
        assert!((p[1] - slope(p[0])).abs() < 0.06, "on the ground: {p:?}");
    }
    // All four waypoints reached: it visits every quadrant around home.
    let quadrants: std::collections::BTreeSet<(bool, bool)> = seen
        .iter()
        .map(|p| (p[0] > home[0], p[2] > home[2]))
        .collect();
    assert_eq!(quadrants.len(), 4, "loops around its home");
    std::fs::remove_dir_all(&dir).unwrap();
}

/// Job skills are level-locked: a level-5 skill is an empty bar slot and
/// is refused at level 1, and appears once the job reaches level 5.
#[test]
fn job_skills_unlock_at_their_level() {
    let reference = reference();
    let mut game = Gameplay::load(&reference, None).unwrap();
    let dir = std::env::temp_dir().join(format!("osfr-job-levels-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("job_abilities.json");
    std::fs::write(
            &path,
            r#"{"schema_version":1,"jobs":[{"profile_id":11,"slots":[
                {"slot":1,"definition_id":4901,"name_id":3784,"icon_id":260,"heal":250,"cooldown_ms":15000,"min_level":1},
                {"slot":2,"definition_id":4902,"name_id":548869300,"icon_id":11639,"damage_multiplier":1.5,"cooldown_ms":5000,"min_level":5}]}]}"#,
        )
        .unwrap();
    let abilities = game.abilities.as_mut().unwrap();
    abilities.load_job_abilities(&path).unwrap();
    let definitions = |bar: &[Vec<u8>]| {
        bar.iter()
            .filter(|p| p.starts_with(&[36, 0, 13, 0]))
            .map(|p| i32::from_le_bytes(p[4..8].try_into().unwrap()))
            .collect::<Vec<_>>()
    };
    let level_one = abilities.activation(11, 1).unwrap();
    assert!(
        !definitions(&level_one).contains(&4902),
        "locked at level 1"
    );
    assert!(abilities.job_slot(11, 2, 1).is_none());
    let level_five = abilities.activation(11, 5).unwrap();
    assert!(
        definitions(&level_five).contains(&4902),
        "unlocked at level 5"
    );
    assert!(abilities.job_slot(11, 2, 5).is_some());
    assert!(abilities.unlocks_between(11, 4, 5));
    assert!(!abilities.unlocks_between(11, 1, 4));
    // Same toolbar length either way: the locked slot is an empty entry.
    assert_eq!(
        level_one.last().unwrap().len() + 37,
        level_five.last().unwrap().len()
    );
    std::fs::remove_dir_all(dir).ok();
}

/// W158 patch B: `realm reload power_ups` loads the drop tables (it used to
/// be an unknown target), and a broken file is refused with the old tables kept.
#[test]
fn power_up_tables_reload_by_name() {
    let reference = reference();
    let mut game = Gameplay::load(&reference, None).unwrap();
    let dir = Scratch::new("osfr-power-reload");
    std::fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/power_ups.json"),
        dir.join("power_ups.json"),
    )
    .unwrap();
    game.content_dir = dir.to_path_buf();
    let mut out = Outcome::default();
    let reply = game.reload_content("power_ups", &mut out).unwrap();
    assert!(reply.contains("power-ups"), "{reply}");
    std::fs::write(dir.join("power_ups.json"), b"{ not json").unwrap();
    assert!(game.reload_content("power_ups", &mut out).is_err());
}

/// A later reload failure must not publish an earlier ability or behaviour
/// load, and the activation packets it staged must not escape the refusal.
#[test]
fn refused_reload_all_keeps_behaviour_and_shared_field_state() {
    const GROUPED_WOLF: u64 = 131_000_102_001;

    let reference = reference();
    let mut game = Gameplay::load(&reference, None).unwrap();
    game.set_dev_tools(true);
    game.set_role(crate::account_store::AccountRole::Admin);
    let profile = game.profiles.as_ref().unwrap().active();
    let level = game.job_level(profile);
    let before_abilities = game
        .abilities
        .as_ref()
        .unwrap()
        .job_bar(profile, level)
        .map(|slot| slot.map(|ability| ability.definition_id));
    let before_activation = game
        .abilities
        .as_ref()
        .unwrap()
        .activation(profile, level)
        .unwrap();
    let shared = game.take_shared_fields().expect("shipped field enemies");
    let actors = |shared: &crate::encounter::SharedFieldState| {
        shared
            .interest_candidates(false)
            .into_iter()
            .map(|npc| (npc.guid, npc.position, npc.engaged))
            .collect::<Vec<_>>()
    };
    let before_actors = actors(&shared);
    let before_guid_order = game
        .encounter
        .as_ref()
        .unwrap()
        .shared_field_guids()
        .collect::<Vec<_>>();
    let before_behaviour = game
        .encounter
        .as_ref()
        .unwrap()
        .behaviour(GROUPED_WOLF)
        .cloned()
        .expect("grouped wolf behaviour");

    let dir = Scratch::new("osfr-reload-all");
    std::fs::copy(
        bundled_content().join("field_enemies.json"),
        dir.join("field_enemies.json"),
    )
    .unwrap();
    let mut behaviours: serde_json::Value =
        serde_json::from_slice(&std::fs::read(bundled_content().join("behaviours.json")).unwrap())
            .unwrap();
    behaviours["presets"]["field_roamer"]["move_speed"] = serde_json::json!(9.0);
    std::fs::write(
        dir.join("behaviours.json"),
        serde_json::to_vec(&behaviours).unwrap(),
    )
    .unwrap();
    std::fs::write(
        dir.join("job_abilities.json"),
        serde_json::to_vec(&serde_json::json!({
            "schema_version": 1,
            "jobs": [{
                "profile_id": profile,
                "slots": [{
                    "slot": 1,
                    "definition_id": 4901,
                    "name_id": 3784,
                    "icon_id": 260,
                    "heal": 250,
                    "cooldown_ms": 15000,
                    "min_level": 1,
                }]
            }]
        }))
        .unwrap(),
    )
    .unwrap();
    game.content_dir = dir.to_path_buf();

    let mut queued = Outcome::default();
    queued.replies.push(vec![0xde, 0xad]);
    let queued_before = queued.replies.clone();
    let error = game.reload_content("all", &mut queued).unwrap_err();
    assert!(error.to_string().contains("shared field reload"), "{error}");
    assert_eq!(queued.replies, queued_before);
    assert!(!queued.changed);

    let refused = game
        .handle_in_world(
            &offer_chat("realm reload all"),
            &CharacterState {
                guid: 1,
                zone_id: 0,
                position: [-440.0, 0.8, 1292.0],
                rotation: [1.0, 0.0, 0.0],
            },
            1_000,
        )
        .unwrap();
    assert_eq!(
        refused.replies,
        vec![chat::system(
            "Reload failed: shared field reload requires an idle world reload. Nothing changed."
        )]
    );
    assert!(!refused.changed);
    assert_eq!(actors(&shared), before_actors);
    assert_eq!(
        game.encounter
            .as_ref()
            .unwrap()
            .shared_field_guids()
            .collect::<Vec<_>>(),
        before_guid_order
    );
    assert_eq!(
        game.encounter.as_ref().unwrap().behaviour(GROUPED_WOLF),
        Some(&before_behaviour)
    );
    assert_eq!(
        game.abilities
            .as_ref()
            .unwrap()
            .job_bar(profile, level)
            .map(|slot| slot.map(|ability| ability.definition_id)),
        before_abilities,
        "a refused reload must not publish the staged job abilities"
    );
    assert_eq!(
        game.abilities
            .as_ref()
            .unwrap()
            .activation(profile, level)
            .unwrap(),
        before_activation,
        "the live hotbar packets are unchanged"
    );
}

/// W158 slice 2b: a big heart's group heal leaves the engine, reaches the
/// gameplay outcome through `apply_encounter`, and heals a hurt member.
#[test]
fn group_heal_reaches_the_outcome_and_heals_a_hurt_member() {
    let reference = reference();
    let mut game = Gameplay::load(&reference, None).unwrap();
    let heal = crate::encounter::GroupHeal {
        percent: 50,
        kill: (7, 9),
    };
    let mut out = Outcome::default();
    game.apply_encounter(
        EncounterOutcome {
            group_heal: Some(heal),
            ..Default::default()
        },
        &mut out,
    )
    .unwrap();
    assert_eq!(out.group_heal, Some(heal));
    game.receive_field_damage(400, 1_000, false).unwrap();
    let before = game.encounter.as_ref().unwrap().player_health().0;
    let healed = game.receive_group_heal(50).unwrap();
    let (now, max) = game.encounter.as_ref().unwrap().player_health();
    assert!(now > before && now <= max, "{before} -> {now} of {max}");
    assert!(healed.changed && !healed.replies.is_empty());
    // At full health a group heal says nothing and changes nothing.
    game.encounter.as_mut().unwrap().heal_full();
    let quiet = game.receive_group_heal(50).unwrap();
    assert!(!quiet.changed && quiet.replies.is_empty());
}

/// Lead: a 4-byte 167/1/4 refresh returns the whole activity list, so it is
/// answered at most every 250 ms (amplification bound, R). The client's own
/// re-asks (job change, membership) are further apart than that.
#[test]
fn activity_portal_refresh_is_throttled() {
    let reference = reference();
    let mut game = Gameplay::load(&reference, None).unwrap();
    let who = CharacterState {
        guid: 1,
        zone_id: 0,
        position: [-440.0, 0.8, 1292.0],
        rotation: [1.0, 0.0, 0.0],
    };
    let refresh = [167, 0, 1, 4];
    let first = game.handle_in_world(&refresh, &who, 10_000).unwrap();
    assert!(!first.replies.is_empty(), "the first refresh is answered");
    let burst = game.handle_in_world(&refresh, &who, 10_100).unwrap();
    assert!(burst.replies.is_empty(), "a burst refresh is dropped");
    let later = game.handle_in_world(&refresh, &who, 10_250).unwrap();
    assert_eq!(
        later.replies, first.replies,
        "after 250 ms it is answered again"
    );
}

/// W143: Inspect shows only unlocked jobs with ranks and the active job's
/// worn items, each with its definition row; nothing for an unworn slot.
#[test]
fn public_look_holds_only_jobs_and_active_worn_items() {
    let reference = reference();
    let mut game = Gameplay::load(&reference, None).unwrap();
    let look = game.public_look().expect("profiles are loaded");
    let profiles = game.profiles.as_ref().unwrap().state().clone();
    assert_eq!(look.active_profile, profiles.active);
    assert_eq!(look.profiles.len(), profiles.unlocked.len());
    let state = game.inventory.state().clone();
    let worn = state
        .equipment
        .iter()
        .filter(|e| e.profile_id == profiles.active && e.item_id != 0)
        .count();
    assert_eq!(look.items.len(), worn);
    assert!(look.items.iter().all(|i| !i.definition_record.is_empty()));
    // Another profile's outfit never leaks into the active look.
    game.profiles.as_mut().unwrap().unlock(11).unwrap();
    let after = game.public_look().unwrap();
    assert_eq!(after.items, look.items);
    assert!(after.profiles.iter().any(|&(id, _)| id == 11));
}

/// T027: an active-job equip or unequip, and a job switch, produce the
/// neighbour packets the gateway fans out; alt-job changes and no-op
/// replays stay local.
#[test]
fn live_equipment_and_job_changes_are_ready_for_neighbours() {
    fn i32_at(bytes: &[u8], at: usize) -> i32 {
        i32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
    }
    fn text_at(bytes: &[u8], at: &mut usize) -> String {
        let length = i32_at(bytes, *at) as usize;
        let text = String::from_utf8(bytes[*at + 4..*at + 4 + length].to_vec()).unwrap();
        *at += 4 + length;
        text
    }
    let reference = reference();
    let mut game = Gameplay::load(&reference, None).unwrap();
    let equipped_profile = game
        .inventory
        .state()
        .equipment
        .iter()
        .find(|entry| {
            entry.item_id != 0
                && game
                    .inventory
                    .state()
                    .items
                    .iter()
                    .any(|item| item.id == entry.item_id)
        })
        .expect("the fixture has an equipped job")
        .profile_id;
    game.profiles
        .as_mut()
        .unwrap()
        .select(equipped_profile, &game.inventory)
        .unwrap();
    let alt_profile = 35;
    game.profiles.as_mut().unwrap().unlock(alt_profile).unwrap();
    game.ensure_outfit(alt_profile, &mut Outcome::default())
        .unwrap();
    let who = CharacterState {
        guid: 42,
        zone_id: 0,
        position: [-440.0, 0.8, 1292.0],
        rotation: [1.0, 0.0, 0.0],
    };
    let state = game.inventory.state().clone();
    let active = game.profiles.as_ref().unwrap().active();
    let worn = state
        .equipment
        .iter()
        .find(|entry| {
            entry.profile_id == active
                && entry.item_id != 0
                && state.items.iter().any(|item| item.id == entry.item_id)
        })
        .unwrap()
        .clone();
    let item = state
        .items
        .iter()
        .find(|item| item.id == worn.item_id)
        .unwrap()
        .clone();

    // Unequip: an empty model is the client's removal path.
    let mut unequip = vec![42, 0, 2, 0];
    unequip.extend(worn.slot.to_le_bytes());
    unequip.extend(active.to_le_bytes());
    let removed = game.handle_in_world(&unequip, &who, 1_000).unwrap();
    assert!(removed.changed);
    let change = removed
        .nearby
        .iter()
        .find(|packet| packet.starts_with(&[35, 0, 6, 0]))
        .expect("an unequip is broadcast");
    assert_eq!(u64::from_le_bytes(change[4..12].try_into().unwrap()), 42);
    assert_eq!(i32_at(change, 12), 0);
    let mut at = 16;
    assert_eq!(text_at(change, &mut at), "");
    assert_eq!(text_at(change, &mut at), "");
    assert_eq!(text_at(change, &mut at), "");
    assert_eq!(i32_at(change, at), 0); // tint
    assert_eq!(i32_at(change, at + 4), 0); // effect
    assert_eq!(i32_at(change, at + 8), worn.slot);
    assert_eq!(i32_at(change, at + 12), active);
    assert_eq!(change.len(), at + 24);

    // Equip it again: one slot with the real attachment row.
    let mut equip = vec![42, 0, 3, 0];
    equip.extend(item.id.to_le_bytes());
    equip.extend(active.to_le_bytes());
    equip.extend(worn.slot.to_le_bytes());
    let worn_again = game.handle_in_world(&equip, &who, 2_000).unwrap();
    assert!(worn_again.changed);
    let change = worn_again
        .nearby
        .iter()
        .find(|packet| packet.starts_with(&[35, 0, 6, 0]))
        .expect("an equip is broadcast");
    assert_eq!(u64::from_le_bytes(change[4..12].try_into().unwrap()), 42);
    assert_eq!(i32_at(change, 12), item.id);
    let mut at = 16;
    assert!(!text_at(change, &mut at).is_empty());
    text_at(change, &mut at);
    text_at(change, &mut at);
    assert_eq!(i32_at(change, at + 8), worn.slot);

    // A replay is not a change, so nobody is told.
    let replay = game.handle_in_world(&equip, &who, 3_000).unwrap();
    assert!(!replay.changed);
    assert!(replay.nearby.is_empty());

    // An alt job's outfit is not visible to a stranger.
    let alt = game
        .inventory
        .state()
        .equipment
        .iter()
        .find(|entry| entry.profile_id == alt_profile && entry.item_id != 0)
        .unwrap()
        .clone();
    let mut alt_unequip = vec![42, 0, 2, 0];
    alt_unequip.extend(alt.slot.to_le_bytes());
    alt_unequip.extend(alt_profile.to_le_bytes());
    let local = game.handle_in_world(&alt_unequip, &who, 4_000).unwrap();
    assert!(local.changed);
    assert!(local.nearby.is_empty(), "an alt job stays local");
    let mut alt_equip = vec![42, 0, 3, 0];
    alt_equip.extend(alt.item_id.to_le_bytes());
    alt_equip.extend(alt_profile.to_le_bytes());
    alt_equip.extend(alt.slot.to_le_bytes());
    assert!(
        game.handle_in_world(&alt_equip, &who, 5_000)
            .unwrap()
            .nearby
            .is_empty()
    );

    // A job switch sends the whole new outfit (35/7).
    let mut switch = vec![26, 0, 13, 0];
    switch.extend(alt_profile.to_le_bytes());
    let switched = game.handle_in_world(&switch, &who, 6_000).unwrap();
    assert!(switched.changed);
    let packet = switched
        .nearby
        .iter()
        .find(|packet| packet.starts_with(&[35, 0, 7, 0]))
        .expect("a job switch is broadcast");
    assert_eq!(u64::from_le_bytes(packet[4..12].try_into().unwrap()), 42);
    let expected = game
        .inventory
        .state()
        .equipment
        .iter()
        .filter(|entry| entry.profile_id == alt_profile && entry.item_id != 0)
        .count() as i32;
    assert_eq!(i32_at(packet, 12), expected);
    assert_eq!(game.public_look().unwrap().active_profile, alt_profile);

    // Switching back reports the previous outfit.
    let mut back = vec![26, 0, 13, 0];
    back.extend(active.to_le_bytes());
    let switched = game.handle_in_world(&back, &who, 7_000).unwrap();
    let packet = switched
        .nearby
        .iter()
        .find(|packet| packet.starts_with(&[35, 0, 7, 0]))
        .expect("the next job switch is broadcast");
    let expected = game
        .inventory
        .state()
        .equipment
        .iter()
        .filter(|entry| entry.profile_id == active && entry.item_id != 0)
        .count() as i32;
    assert_eq!(i32_at(packet, 12), expected);
}

/// W141: 36/12 resolves only this job's own toolbar definitions; a miss is
/// silent; unsupported 36/10 modes, other bars and empty or level-locked
/// slots get the client failure string. None of it touches game state.
#[test]
fn ability_definition_lookup_and_unsupported_casts_are_explicit() {
    let reference = reference();
    let mut game = Gameplay::load(&reference, None).unwrap();
    let profiles = game.profiles.as_mut().unwrap();
    profiles.unlock(11).unwrap();
    profiles.select(11, &game.inventory).unwrap();
    let level = game.job_level(11);
    let loaded: Vec<Vec<u8>> = game
        .abilities
        .as_ref()
        .unwrap()
        .activation(11, level)
        .unwrap()
        .into_iter()
        .filter(|p| p.starts_with(&[36, 0, 13, 0]))
        .collect();
    assert!(loaded.len() >= 3, "basic, Triage, First Aid at level 1");
    let who = CharacterState {
        guid: 1,
        zone_id: 0,
        position: [-440.0, 0.8, 1292.0],
        rotation: [1.0, 0.0, 0.0],
    };
    let failure = [vec![36, 0, 1, 0], 286_i32.to_le_bytes().to_vec()].concat();
    let state = |game: &Gameplay| {
        (
            serde_json::to_string(&game.summary()).unwrap(),
            serde_json::to_string(&game.encounter.as_ref().unwrap().state()).unwrap(),
        )
    };
    let before = state(&game);
    for definition in &loaded {
        let mut ask = vec![36, 0, 12, 0];
        ask.extend(&definition[4..8]);
        let out = game.handle_in_world(&ask, &who, 5_000).unwrap();
        assert_eq!(out.replies, vec![definition.clone()]);
        assert!(!out.changed);
    }
    for id in [0, -1, 62_002, 62_004, 999_999, i32::MIN, i32::MAX] {
        let mut ask = vec![36, 0, 12, 0];
        ask.extend(id.to_le_bytes());
        let out = game.handle_in_world(&ask, &who, 5_000).unwrap();
        assert!(out.replies.is_empty(), "locked, other-job or unknown {id}");
    }
    assert!(
        game.handle_in_world(&[36, 0, 12, 0, 1, 2], &who, 5_000)
            .is_err()
    );
    let cast = |bar: i32, slot: i32, tail: &[u8], mode: i32| {
        let mut p = vec![36, 0, 10, 0];
        for v in [bar, slot, mode] {
            p.extend(v.to_le_bytes());
        }
        p.extend(tail);
        p
    };
    let target = 1_u64.to_le_bytes();
    for refused in [
        cast(1, 1, &[0; 16], 1), // ground mode on a real slot
        cast(1, 0, &[0; 16], 1), // ground mode on the basic slot
        cast(2, 0, &target, 0),  // another bar
        cast(1, 9, &target, 0),  // past the toolbar
        cast(1, 3, &target, 0),  // Medic slot 3 unlocks at level 5
        cast(1, 7, &target, 0),  // empty slot
    ] {
        let out = game.handle_in_world(&refused, &who, 5_000).unwrap();
        assert_eq!(out.replies, vec![failure.clone()], "{refused:?}");
    }
    assert_eq!(state(&game), before, "refusals change nothing");
}

/// Final-era scaling on the level-1 baseline: max health x1.75 every four
/// levels, weapon damage x1.75 per power tier (levels 4, 5, 8, 12).
#[test]
fn player_health_and_damage_scale_with_job_level() {
    const SUMMONER: u64 = 100_000_096_001;
    let reference = reference();
    let mut game = Gameplay::load(&reference, None).unwrap();
    let encounter = game.encounter.as_mut().unwrap();
    let health = |packets: &[Vec<u8>]| {
        let p = packets
            .iter()
            .find(|p| p.starts_with(&[38, 0, 1, 0]))
            .unwrap();
        (
            u32::from_le_bytes(p[4..8].try_into().unwrap()),
            u32::from_le_bytes(p[8..12].try_into().unwrap()),
        )
    };
    assert!(
        encounter.set_player_job(32, 1).is_empty(),
        "level 1 is the baseline"
    );
    assert_eq!(
        health(&encounter.set_player_job(32, 10)),
        (3522, 3522),
        "full stays full"
    );
    assert_eq!(health(&encounter.set_player_job(32, 20)), (14_270, 14_270));
    assert_eq!(
        health(&encounter.set_player_job(32, 10)),
        (3522, 3522),
        "capped down"
    );

    let world = CharacterState {
        guid: 1,
        zone_id: 0,
        position: [-440.0, 0.8, 1292.0],
        rotation: [1.0, 0.0, 0.0],
    };
    encounter.begin(96, world, 11).unwrap();
    let ready = encounter.on_zone_ready(11, 1_000).unwrap();
    let at = spawned_actors(&ready.replies)[0].1;
    let hit = encounter.attack(SUMMONER, at, 11, 0, 2_000).unwrap();
    let dealt = hit
        .replies
        .iter()
        .find(|p| p.starts_with(&[32, 0, 7, 0]))
        .map(|p| u32::from_le_bytes(p[28..32].try_into().unwrap()));
    // Medic 75 at weapon tier 4 (x1.75^3), or a x1.5 crit.
    assert!(matches!(dealt, Some(402) | Some(603)), "{dealt:?}");
}

/// Job identity: Brawler is the tank, Wizard the most fragile; Ninja crits
/// far more often than Medic (x1.5 hits flagged critical).
#[test]
fn combat_jobs_differ_in_health_and_crits() {
    let reference = reference();
    let mut game = Gameplay::load(&reference, None).unwrap();
    let encounter = game.encounter.as_mut().unwrap();
    let max = |e: &mut EncounterEngine, profile: i32| {
        e.set_player_job(profile, 1);
        let p = e.set_player_job(profile, 20);
        u32::from_le_bytes(p[0][8..12].try_into().unwrap())
    };
    let brawler = max(encounter, 43);
    let ninja = max(encounter, 2);
    let medic = max(encounter, 11);
    let wizard = max(encounter, 12);
    assert!(
        brawler > ninja && ninja >= medic && medic > wizard,
        "{brawler} {ninja} {medic} {wizard}"
    );

    let crits = |e: &mut EncounterEngine, profile: i32| {
        e.set_player_job(profile, 1);
        (0..1000).filter(|_| e.roll_crit()).count()
    };
    let ninja_crits = crits(encounter, 2);
    let medic_crits = crits(encounter, 11);
    assert!(
        (150..=250).contains(&ninja_crits),
        "ninja ~20%: {ninja_crits}"
    );
    assert!((20..=80).contains(&medic_crits), "medic ~5%: {medic_crits}");
}

/// Studio cheat console through the normal chat path: sim, battle, win,
/// reload (refused mid-fight), level.
#[test]
fn developer_cheat_console_drives_a_battle() {
    let reference = reference();
    let mut game = Gameplay::load(&reference, None).unwrap();
    game.set_dev_tools(true);
    game.set_role(crate::account_store::AccountRole::Admin);
    let inventory = game.inventory.clone();
    game.profiles
        .as_mut()
        .unwrap()
        .select(43, &inventory)
        .unwrap();
    let character = CharacterState {
        guid: 1,
        zone_id: 0,
        position: [-440.0, 0.8, 1292.0],
        rotation: [1.0, 0.0, 0.0],
    };
    let text = |out: &Outcome| String::from_utf8_lossy(&out.replies.concat()).to_string();
    let mut now = 1_000;
    let mut run = |game: &mut Gameplay, command: &str| {
        now += 1_000;
        game.handle_in_world(&offer_chat(&format!("realm {command}")), &character, now)
            .unwrap()
    };

    let sim = run(&mut game, "sim 96");
    assert!(text(&sim).contains("level  1:"), "{}", text(&sim));
    assert!(text(&sim).contains("level 20:"), "{}", text(&sim));

    let started = run(&mut game, "battle 96");
    assert!(started.zoning.is_some(), "battle zones into the arena");
    let refused = run(&mut game, "reload battles");
    assert!(
        text(&refused).contains("Reload failed"),
        "{}",
        text(&refused)
    );
    game.encounter
        .as_mut()
        .unwrap()
        .on_zone_ready(43, 500_000)
        .unwrap();
    let won = run(&mut game, "win");
    assert!(
        won.replies.iter().any(|p| p.starts_with(&[39, 0, 18])),
        "game over"
    );
    assert!(game.fight_log().iter().any(|e| e.kind == "defeat"));
    let log = run(&mut game, "fightlog");
    assert!(text(&log).contains("Fight log:"));
    run(&mut game, "exit");

    let reload = run(&mut game, "reload all");
    assert!(text(&reload).contains("Reloaded:"), "{}", text(&reload));
    assert!(
        reload.replies.iter().any(|p| p.starts_with(&[36, 0, 5, 0])),
        "bar re-sent"
    );

    let level = run(&mut game, "level 10");
    assert!(text(&level).contains("level 10"), "{}", text(&level));
    assert!(
        game.encounter.as_ref().unwrap().player_health().1 > 1_000,
        "health scaled"
    );
}

/// Battle spawners (data/content/field_battles.json) are world-style AddNpc
/// actors sent by proximity, so the client keeps and renders them.
#[test]
fn battle_spawner_spawns_at_zone_ready() {
    const SPAWNER: u64 = 100_000_096_000;
    let reference = reference();
    let mut game = Gameplay::load(&reference, None).unwrap();
    let encounter = game.encounter.as_mut().unwrap();
    assert!(encounter.is_battle_spawner(SPAWNER));
    let far = CharacterState {
        guid: 1,
        zone_id: 0,
        position: [-1900.0, -40.0, 450.0],
        rotation: [1.0, 0.0, 0.0],
    };
    let spawned = |packets: &[Vec<u8>]| {
        packets
            .iter()
            .any(|p| p.starts_with(&[35, 0, 2, 0]) && p[4..12] == SPAWNER.to_le_bytes())
    };
    assert!(
        !spawned(&game.update_npc_visibility(&far).unwrap()),
        "not sent from afar"
    );
    let near = CharacterState {
        position: [-435.0, 0.7, 1289.6],
        ..far
    };
    let near_packets = game.update_npc_visibility(&near).unwrap();
    assert!(spawned(&near_packets), "sent when near");
}

/// Behaviour presets (data/content/behaviours.json): the summoner's
/// `skittish: true` maps to the skittish preset, field enemies default to
/// melee, and a record can pick `stationary` or override a preset's speed.
#[test]
fn behaviour_presets_drive_enemy_movement() {
    const HOOLIGAN: u64 = 100_000_004_154;
    let reference = reference();
    let mut game = Gameplay::load(&reference, None).unwrap();
    let encounter = game.encounter.as_mut().unwrap();
    let melee = encounter.behaviour(HOOLIGAN).unwrap().clone();
    assert_eq!(melee.mode, crate::encounter::Mode::Melee);
    assert_eq!(
        (melee.move_speed, melee.reach, melee.attack_ms),
        (3.0, 2.5, 1000)
    );
    let summoner = encounter
        .behaviour(100_000_096_001)
        .expect("battle 96 summoner");
    assert_eq!(summoner.mode, crate::encounter::Mode::Skittish);
    assert_eq!((summoner.run_ms, summoner.leash), (4000, 12.0));

    // A copy of the field enemies with the Hooligans given a behaviour.
    let dir = std::env::temp_dir().join(format!("osfr-behaviours-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/field_enemies.json");
    let mut enemies: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&source).unwrap()).unwrap();
    let with = |enemies: &mut serde_json::Value, behaviour: serde_json::Value| {
        for e in enemies["enemies"].as_array_mut().unwrap() {
            if e["name"] == "Hooligan" {
                e["behaviour"] = behaviour.clone();
            }
        }
        let path = dir.join("field_enemies.json");
        std::fs::write(&path, enemies.to_string()).unwrap();
        path
    };
    let moves = |out: &EncounterOutcome| -> Vec<[f32; 3]> {
        out.replies
            .iter()
            .filter(|p| p.starts_with(&[125, 0]) && p[2..10] == HOOLIGAN.to_le_bytes())
            .map(|p| {
                let f = |o: usize| f32::from_le_bytes(p[o..o + 4].try_into().unwrap());
                [f(10), f(14), f(18)]
            })
            .collect()
    };
    let home = [-1971.408_f32, -34.51335, 539.8358];
    let near = [home[0] + 6.0, home[1], home[2]];

    let path = with(&mut enemies, "stationary".into());
    encounter.reload_field_enemies(&path).unwrap();
    encounter.set_active_quests(&BTreeSet::from([3_030_007]));
    assert_eq!(
        encounter.behaviour(HOOLIGAN).unwrap().mode,
        crate::encounter::Mode::Stationary
    );
    for step in 1..=20_u64 {
        let out = encounter.tick(near, 2, 50_000 + step * 250).unwrap();
        assert!(moves(&out).is_empty(), "a stationary enemy never moves");
    }
    let beside = [home[0] + 1.5, home[1], home[2]];
    let swing = encounter.tick(beside, 2, 60_000).unwrap();
    assert!(
        swing.replies.iter().any(|p| p.starts_with(&[32, 0, 7, 0])),
        "but it still swings within reach"
    );

    // Twice the default speed: two 250 ms ticks cover 3 m, not 1.5 m.
    let path = with(
        &mut enemies,
        serde_json::json!({"preset": "melee", "move_speed": 6.0}),
    );
    encounter.reload_field_enemies(&path).unwrap();
    encounter.set_active_quests(&BTreeSet::from([3_030_007]));
    encounter.tick(near, 2, 70_000).unwrap();
    let first = moves(&encounter.tick(near, 2, 70_250).unwrap());
    let travelled = ((first[0][0] - home[0]).powi(2) + (first[0][2] - home[2]).powi(2)).sqrt();
    assert!((travelled - 3.0).abs() < 0.05, "moved {travelled} m");

    // Reach is tunable on the fly; nonsense is refused.
    let path = with(
        &mut enemies,
        serde_json::json!({"preset": "melee", "reach": 4.0}),
    );
    encounter.reload_field_enemies(&path).unwrap();
    assert_eq!(encounter.behaviour(HOOLIGAN).unwrap().reach, 4.0);
    let path = with(
        &mut enemies,
        serde_json::json!({"preset": "melee", "reach": -1.0}),
    );
    assert!(encounter.reload_field_enemies(&path).is_err());
    let path = with(
        &mut enemies,
        serde_json::json!({"preset": "melee", "move_speed": 6.0}),
    );
    encounter.reload_field_enemies(&path).unwrap();

    // Mistakes are refused and change nothing.
    let path = with(&mut enemies, "sneaky".into());
    let error = encounter
        .reload_field_enemies(&path)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("unknown behaviour preset 'sneaky'"),
        "{error}"
    );
    assert_eq!(encounter.behaviour(HOOLIGAN).unwrap().move_speed, 6.0);
    let typo = dir.join("behaviours.json");
    std::fs::write(
        &typo,
        r#"{"schema_version":1,"presets":{"melee":{"move_sped":9}}}"#,
    )
    .unwrap();
    assert!(encounter.load_behaviours(&typo).is_err());
    assert_eq!(encounter.behaviour(100_000_096_001).unwrap().run_ms, 4000);
    std::fs::remove_dir_all(&dir).unwrap();
}

/// Quest-gated overworld enemies (data/content/field_enemies.json): hidden
/// until Delinquent Dilemma is active, then spawned, fought with the opening's
/// proven attack packets, removed on defeat, respawned, and hidden again when
/// the quest ends. Played natively in run 002731 (Medic, X to attack).
#[test]
fn quest_gated_field_enemies_spawn_fight_and_respawn() {
    const HOOLIGAN: u64 = 100_000_004_154;
    let reference = reference();
    let mut game = Gameplay::load(&reference, None).unwrap();
    let encounter = game
        .encounter
        .as_mut()
        .expect("opening content loads the encounter engine");
    assert!(encounter.field_guids().contains(&HOOLIGAN));
    assert!(
        !encounter.is_field_enemy(HOOLIGAN),
        "hidden until the quest is active"
    );

    let shown = encounter.set_active_quests(&BTreeSet::from([3_030_007]));
    let spawns = shown
        .replies
        .iter()
        .filter(|p| p.starts_with(&[35, 0, 2, 0]))
        .count();
    assert_eq!(spawns, 4, "four Hooligans appear with the quest");
    assert!(encounter.is_field_enemy(HOOLIGAN));

    // Profile 2 deals 100 per swing; 220 HP falls on the third swing.
    let near = [-1971.408, -34.51335, 541.0];
    let mut defeated = Vec::new();
    for swing in 0..3_u64 {
        let out = encounter
            .attack(HOOLIGAN, near, 2, 0, 10_000 + swing * 700)
            .unwrap();
        defeated.extend(out.defeated.iter().copied());
    }
    assert_eq!(
        defeated,
        vec![930_050],
        "a defeat reports the quest kill id"
    );
    assert!(
        encounter
            .attack(HOOLIGAN, near, 2, 0, 13_000)
            .unwrap()
            .replies
            .is_empty(),
        "a defeated enemy cannot be hit"
    );

    // Respawn after 30 s; the player stands far away so nothing aggroes.
    let far = [-1700.0, -30.0, 400.0];
    let respawned = encounter.tick(far, 2, 10_000 + 30_000 + 1_500).unwrap();
    assert!(
        respawned
            .replies
            .iter()
            .any(|p| p.starts_with(&[35, 0, 2, 0]) && p[4..12] == HOOLIGAN.to_le_bytes()),
        "the Hooligan respawns"
    );

    let hidden = encounter.set_active_quests(&BTreeSet::new());
    assert_eq!(
        hidden
            .replies
            .iter()
            .filter(|p| p.starts_with(&[35, 0, 3, 0]))
            .count(),
        4,
        "they leave when the quest is no longer active"
    );
    assert!(!encounter.is_field_enemy(HOOLIGAN));
}

/// Cobblestone starter chain, as played natively in run 215840: Introduce Yourself
/// (Samantha -> Ricky), Call the Crew! (Ricky -> Shakey -> Ricky), Back for More
/// (Ricky -> Samantha). Steps are ordered: Ricky does not count before Shakey.
#[test]
fn cobblestone_starter_chain_runs_in_order() {
    const RICKY: u64 = 9_500_000_254;
    const SHAKEY: u64 = 9_500_000_941;
    const CALL_THE_CREW: u32 = 3_000_059;
    const BACK_FOR_MORE: u32 = 3_000_060;
    let content = bundled_content();
    let reference = reference();
    let mut game = Gameplay::load(&reference, None).unwrap();
    game.profiles = Some(Profiles::load(&reference, Some(native_test_profiles())).unwrap());
    game.bind_npc_quest_catalogue(&content, 1).unwrap();

    let mut positions = std::collections::BTreeMap::new();
    for entry in std::fs::read_dir(content.join("npcs")).unwrap() {
        let file: serde_json::Value =
            serde_json::from_slice(&std::fs::read(entry.unwrap().path()).unwrap()).unwrap();
        for record in file["records"].as_array().unwrap() {
            if let Ok(position) = serde_json::from_value::<[f32; 3]>(record["position"].clone()) {
                positions.insert(record["npc_id"].as_u64().unwrap(), position);
            }
        }
    }
    let clock = std::cell::Cell::new(2_000_u64);
    let talk = |game: &mut Gameplay, npc: u64| -> (CharacterState, Vec<Vec<u8>>) {
        let character = CharacterState {
            guid: 1,
            zone_id: 0,
            position: positions[&npc],
            rotation: [1.0, 0.0, 0.0],
        };
        game.update_npc_visibility(&character).unwrap();
        clock.set(clock.get() + 10);
        game.handle_in_world(&packet_for_npc(npc), &character, clock.get())
            .unwrap();
        let out = game
            .handle_in_world(&select_npc_option(npc, 1), &character, clock.get() + 1)
            .unwrap();
        (character, out.replies)
    };
    let has = |replies: &[Vec<u8>], prefix: &[u8], quest: u32| {
        replies
            .iter()
            .any(|p| p.starts_with(prefix) && p.get(6..10) == Some(&quest.to_le_bytes()[..]))
    };
    let quest_reply = |subtype: u8, quest: u32, accept: bool| {
        let mut packet = vec![49, 0, subtype, 0, 0, 0];
        packet.extend(quest.to_le_bytes());
        if subtype == 2 {
            packet.push(accept as u8);
        }
        packet
    };
    let offer_and_accept = |game: &mut Gameplay, npc: u64, quest: u32| {
        let (character, _) = talk(game, npc);
        let offer = game
            .handle_in_world(&choose_quest_row(), &character, 3_000)
            .unwrap();
        assert!(has(&offer.replies, &[49, 0, 1, 0], quest), "offer {quest}");
        game.handle_in_world(&quest_reply(2, quest, true), &character, 3_001)
            .unwrap();
        character
    };
    let turn_in = |game: &mut Gameplay, character: &CharacterState, quest: u32, end_text: u32| {
        let end = game
            .handle_in_world(&choose_quest_row(), character, 3_002)
            .unwrap();
        let screen = end
            .replies
            .iter()
            .find(|p| {
                p.starts_with(&[49, 0, 13, 0]) && p.get(14..18) == Some(&quest.to_le_bytes()[..])
            })
            .unwrap_or_else(|| panic!("end screen {quest}"));
        // Offset 6 carries the turn-in NPC's GUID for the client's actor lookup.
        assert!(
            [SAMANTHA_ID, RICKY].contains(&u64::from_le_bytes(screen[6..14].try_into().unwrap())),
            "end screen {quest} names its NPC"
        );
        assert_eq!(screen[18..22], end_text.to_le_bytes(), "end text {quest}");
        let done = game
            .handle_in_world(&quest_reply(14, quest, true), character, 3_003)
            .unwrap();
        assert!(
            done.replies.iter().any(|p| p.starts_with(&[49, 0, 4, 0])),
            "complete {quest}"
        );
    };
    let status = |game: &Gameplay, quest: u32| {
        game.npc_quests
            .as_ref()
            .unwrap()
            .progress(quest)
            .map(|p| p.status.clone())
    };

    offer_and_accept(&mut game, SAMANTHA_ID, INTRODUCE_YOURSELF_ID);
    let (at_ricky, _) = talk(&mut game, RICKY);
    turn_in(&mut game, &at_ricky, INTRODUCE_YOURSELF_ID, 0);

    offer_and_accept(&mut game, RICKY, CALL_THE_CREW);
    let (_, early) = talk(&mut game, RICKY);
    assert!(
        !has(&early, &[49, 0, 13, 0], CALL_THE_CREW),
        "Ricky before Shakey must not complete"
    );
    let (_, at_shakey) = talk(&mut game, SHAKEY);
    let shakey_dialogue = at_shakey
        .iter()
        .find(|p| p.starts_with(&[26, 0, 3, 0]))
        .expect("Shakey answers with a dialog");
    assert_eq!(
        npc_interaction::decode_dialogue_26_3(shakey_dialogue)
            .unwrap()
            .initial_dwords[0],
        94_258,
        "Shakey says his step line"
    );
    assert!(
        has(&at_shakey, &[49, 0, 9, 0], CALL_THE_CREW),
        "Shakey completes step 1"
    );
    let (at_ricky, _) = talk(&mut game, RICKY);
    turn_in(&mut game, &at_ricky, CALL_THE_CREW, 94_257);

    offer_and_accept(&mut game, RICKY, BACK_FOR_MORE);
    let (at_samantha, _) = talk(&mut game, SAMANTHA_ID);
    turn_in(&mut game, &at_samantha, BACK_FOR_MORE, 94_265);
    for quest in [INTRODUCE_YOURSELF_ID, CALL_THE_CREW, BACK_FOR_MORE] {
        assert_eq!(
            status(&game, quest),
            Some(crate::quest_engine::QuestStatus::Completed)
        );
    }
}
