//! T105: authored interaction and terminal dungeon objectives through the
//! production content loader, gameplay hooks and quest engine.
//!
//! The split quests below are synthetic and bounded to this test. The gameplay
//! package is the preserved local opening fixture used by adjacent integration
//! tests; it is copied before the encounter is reduced to one terminal target.

use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    chat,
    content::ContentDb,
    encounter::EncounterPhase,
    gameplay::{Gameplay, GameplayState, Outcome},
    harvest_nodes::GatheringNodes,
    quest_engine::{CharacterEligibility, QuestEngine, QuestEvent},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const CHARACTER: u64 = 1;
const INTERACT_ACTOR: u64 = 9_000_000_189;
const OTHER_ACTOR: u64 = 9_000_000_121;
const INTERACT_QUEST: u32 = 8_105_001;
const DUNGEON_QUEST: u32 = 8_105_002;
const INTERACT_OBJECTIVE: u32 = 1;
const DUNGEON_OBJECTIVE: u32 = 1;
const ENCOUNTER: u32 = 411;
const GUIDE_START_CHOICE: i32 = 104;
/// Synthetic opening actor: the shipped package has no `interact` actor with a job gate.
const OPENING_ACTOR: u64 = 9_000_000_901;
const OPENING_ACTOR_POSITION: [f32; 3] = [-1897.0, -39.7, 430.0];
const MINER: i32 = 14;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
    gameplay: PathBuf,
    quests: PathBuf,
    interact_position: [f32; 3],
    other_position: [f32; 3],
    guide: u64,
    enemy_position: [f32; 3],
}

impl Fixture {
    fn new() -> Self {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let source = manifest.join("../../../lab/references/opening-anvil-placement-v1/content");
        assert!(
            source.is_dir(),
            "T105 requires the preserved opening fixture at {}",
            source.display()
        );
        let root = std::env::temp_dir().join(format!(
            "osfr-quest-content-goals-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let gameplay = root.join("gameplay");
        let quests = root.join("quests-content");
        fs::create_dir_all(&gameplay).unwrap();
        for entry in fs::read_dir(&source).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file() {
                fs::copy(&path, gameplay.join(path.file_name().unwrap())).unwrap();
            }
        }
        let (guide, enemy_position) = patch_encounter(&gameplay);
        write_quest_content(&quests, INTERACT_ACTOR, ENCOUNTER, true);
        let (interact_position, other_position) =
            bundled_npc_positions(&manifest, [INTERACT_ACTOR, OTHER_ACTOR]);
        Self {
            root,
            gameplay,
            quests,
            interact_position,
            other_position,
            guide,
            enemy_position,
        }
    }

    fn engine(&self) -> QuestEngine {
        QuestEngine::from_split_content(
            &self.quests,
            CHARACTER,
            CharacterEligibility {
                member: true,
                nonmember_quest_limit: 10,
                member_quest_limit: 10,
                ..Default::default()
            },
            None,
        )
        .unwrap()
    }

    fn active_state(&self, quest_id: u32) -> GameplayState {
        let mut engine = self.engine();
        assert!(engine.offer(quest_id, INTERACT_ACTOR).changed);
        assert!(engine.accept(quest_id).changed);
        let base = Gameplay::load(&self.gameplay, None).unwrap();
        let mut state = base.state();
        state.npc_quests = Some(engine.state().clone());
        state
    }

    fn game(&self, state: Option<GameplayState>) -> Gameplay {
        let mut game = Gameplay::load(&self.gameplay, state).unwrap();
        game.bind_npc_quest_catalogue(&self.quests, CHARACTER)
            .unwrap();
        game.set_dev_tools(true);
        game.set_role(AccountRole::Admin);
        game
    }

    fn character(&self, position: [f32; 3]) -> CharacterState {
        CharacterState {
            guid: CHARACTER,
            zone_id: 0,
            position,
            rotation: [1.0, 0.0, 0.0],
        }
    }

    /// Add the shipped Mining18 content so a real Type8 run can start in place.
    fn add_mining(&self) {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        fs::copy(
            manifest.join("data/content/mining.json"),
            self.gameplay.join("mining.json"),
        )
        .unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn patch_encounter(gameplay: &Path) -> (u64, [f32; 3]) {
    let path = gameplay.join("encounters.json");
    let mut content: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let dungeon = content["dungeons"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|dungeon| dungeon["id"].as_u64() == Some(u64::from(ENCOUNTER)))
        .expect("opening fixture has encounter 411");
    let guide = dungeon["guide"]["guid"].as_u64().unwrap();
    let mut enemy = dungeon["enemies"][0].clone();
    enemy["max_health"] = json!(1);
    enemy["damage"] = json!(1_000);
    dungeon["enemies"] = json!([enemy]);
    dungeon["reward_coins"] = json!(0);
    dungeon["reward_items"] = json!([]);
    let enemy_position: [f32; 3] =
        serde_json::from_value(dungeon["enemies"][0]["position"].clone()).unwrap();
    fs::write(path, serde_json::to_vec(&content).unwrap()).unwrap();
    (guide, enemy_position)
}

/// Add one synthetic opening actor whose interact trigger requires the Brawler job. The shipped
/// package has no `interact` actor with a job requirement, but the content model allows it and
/// this is the refusal the T105 hook must not turn into quest progress.
fn patch_opening_interact_actor(gameplay: &Path) {
    let path = gameplay.join("opening.json");
    let mut content: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let mut actor = content["actors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|actor| actor["guid"].as_u64() == Some(5_715_144_128_442_925_058))
        .cloned()
        .expect("opening fixture has the Hero's Journal interact actor");
    actor["guid"] = json!(OPENING_ACTOR);
    actor["name"] = json!("Wrong-Job Test Guide");
    actor["position"] = json!(OPENING_ACTOR_POSITION);
    actor["action"] = json!({"kind": "interact", "actor_guid": OPENING_ACTOR});
    actor["required_profile"] = json!(43);
    actor["provenance"] = json!("(R) F227 wrong-job refusal regression actor; synthetic.");
    content["actors"].as_array_mut().unwrap().push(actor);
    let bytes = serde_json::to_vec(&content).unwrap();
    fs::write(&path, &bytes).unwrap();
    // The presentation and campaign markers pin the opening content digest.
    let digest = format!("{:x}", Sha256::digest(&bytes));
    for name in [
        "journal-text.json",
        "cooking-campaign.json",
        "forging-campaign.json",
    ] {
        let marker = gameplay.join(name);
        if !marker.is_file() {
            continue;
        }
        let mut document: Value = serde_json::from_slice(&fs::read(&marker).unwrap()).unwrap();
        if document.get("opening_sha256").is_some() {
            document["opening_sha256"] = json!(digest);
            fs::write(&marker, serde_json::to_vec(&document).unwrap()).unwrap();
        }
    }
}

fn write_quest_content(path: &Path, actor_id: u64, encounter_id: u32, with_source: bool) {
    fs::create_dir_all(path.join("quests")).unwrap();
    fs::create_dir_all(path.join("npcs")).unwrap();
    let source = if with_source {
        json!([{
            "kind": "reconstructed",
            "evidence": "(R) T105 synthetic quest-goal fixture"
        }])
    } else {
        json!([])
    };
    let quests = json!({
        "schema_version": 1,
        "records": [
            {
                "id": INTERACT_QUEST,
                "title_string_id": 1,
                "giver_npc_id": INTERACT_ACTOR,
                "prerequisite_quest_ids": [],
                "objectives": [{
                    "kind": "interact_actor",
                    "id": INTERACT_OBJECTIVE,
                    "actor_guid": actor_id,
                    "source": source.clone()
                }],
                "rewards": [{
                    "coins": 7,
                    "items": [],
                    "job_xp": {},
                    "source": source.clone()
                }],
                "source": source.clone()
            },
            {
                "id": DUNGEON_QUEST,
                "title_string_id": 2,
                "giver_npc_id": INTERACT_ACTOR,
                "prerequisite_quest_ids": [],
                "objectives": [{
                    "kind": "complete_dungeon",
                    "id": DUNGEON_OBJECTIVE,
                    "encounter_id": encounter_id,
                    "source": source.clone()
                }],
                "rewards": [{
                    "coins": 9,
                    "items": [],
                    "job_xp": {},
                    "source": source.clone()
                }],
                "source": source.clone()
            }
        ]
    });
    fs::write(
        path.join("quests/synthetic.json"),
        serde_json::to_vec(&quests).unwrap(),
    )
    .unwrap();
    let jobs = json!({
        "schema_version": 1,
        "records": [{
            "id": "quest-goal-fixture",
            "name": "Quest goal fixture",
            "profile_id": 1,
            "category": "freestyle",
            "max_level": 20,
            "members_only": false,
            "trainer_npc_ids": [],
            "quest_ids": [INTERACT_QUEST, DUNGEON_QUEST],
            "abilities": [],
            "source": source
        }]
    });
    fs::write(path.join("jobs.json"), serde_json::to_vec(&jobs).unwrap()).unwrap();
    let npcs = json!({
        "schema_version": 1,
        "records": [{
            "npc_id": INTERACT_ACTOR,
            "quest_ids_given": [INTERACT_QUEST, DUNGEON_QUEST],
            "quests_given": [],
            "quests_accepted": []
        }]
    });
    fs::write(
        path.join("npcs/synthetic.json"),
        serde_json::to_vec(&npcs).unwrap(),
    )
    .unwrap();
}

fn write_flat_validation_fixture(path: &Path, goal: Value) {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content");
    fs::create_dir_all(path).unwrap();
    for name in [
        "dungeons.json",
        "items.json",
        "jobs.json",
        "minigames.json",
        "npcs.json",
        "pois.json",
        "zones.json",
    ] {
        fs::copy(fixture.join(name), path.join(name)).unwrap();
    }
    let source = json!([{
        "kind": "reconstructed",
        "evidence": "(R) T105 flat validation fixture"
    }]);
    let quests = json!({
        "schema_version": 1,
        "records": [{
            "id": INTERACT_QUEST,
            "title_string_id": null,
            "giver_npc_id": 2_732_686_529_148_769_137_u64,
            "prerequisite_quest_ids": [],
            "objectives": [goal],
            "rewards": [],
            "source": source
        }]
    });
    fs::write(
        path.join("quests.json"),
        serde_json::to_vec(&quests).unwrap(),
    )
    .unwrap();
}

fn bundled_npc_positions(manifest: &Path, ids: [u64; 2]) -> ([f32; 3], [f32; 3]) {
    let records: Value = serde_json::from_slice(
        &fs::read(manifest.join("data/content/npcs/cobblestone_village.json")).unwrap(),
    )
    .unwrap();
    let position = |id: u64| {
        let record = records["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|record| record["npc_id"].as_u64() == Some(id))
            .unwrap_or_else(|| panic!("bundled NPC {id} is present"));
        serde_json::from_value(record["position"].clone()).unwrap()
    };
    (position(ids[0]), position(ids[1]))
}

fn count(state: &GameplayState, quest_id: u32) -> u32 {
    state.npc_quests.as_ref().unwrap().quests[&quest_id].counts[0]
}

fn copper_vein() -> osfr_tools::harvest_nodes::NodeActor {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let nodes = GatheringNodes::load(&manifest.join("data/content"))
        .unwrap()
        .expect("bundled gathering nodes");
    nodes
        .actors()
        .find(|actor| actor.name == "Copper Vein")
        .expect("bundled copper vein")
        .clone()
}

fn saved(game: &Gameplay) -> Vec<u8> {
    serde_json::to_vec(&game.state()).unwrap()
}

fn said(out: &Outcome, needle: &str) -> bool {
    out.replies.iter().any(|packet| {
        packet.starts_with(&[15, 0, 3, 0])
            && packet.len() >= 9
            && String::from_utf8_lossy(&packet[8..packet.len() - 1]).contains(needle)
    })
}

fn interact_packet(actor: u64) -> Vec<u8> {
    // Client `CommandPacketInteractRequest` ctor 0x008b6bc0: family 26 / subtype 8, the
    // caller's two GUID dwords at +0x10/+0x14 and the flag byte at +0x18. The captured native
    // inner frame is 1a000800050000004e45504f00 (13 bytes).
    let mut packet = vec![26, 0, 8, 0];
    packet.extend(actor.to_le_bytes());
    packet.push(0);
    packet
}

fn select_packet(actor: u64, choice: i32) -> Vec<u8> {
    // Client `CommandPacketInteractionSelect` ctor 0x00a36f80: family 26 / subtype 10, GUID
    // dwords at +0x10/+0x14 and the i32 choice at +0x18 (16 bytes).
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(actor.to_le_bytes());
    packet.extend(choice.to_le_bytes());
    packet
}

fn client_claim(objective: u32) -> Vec<u8> {
    let mut packet = vec![45, 0, 7];
    packet.extend((objective as i32).to_le_bytes());
    packet
}

fn select_job(job: i32) -> Vec<u8> {
    let mut packet = vec![26, 0, 13, 0];
    packet.extend(job.to_le_bytes());
    packet
}

fn basic_attack() -> Vec<u8> {
    [
        vec![36, 0, 10, 0],
        1_i32.to_le_bytes().to_vec(),
        0_i32.to_le_bytes().to_vec(),
        0_i32.to_le_bytes().to_vec(),
        CHARACTER.to_le_bytes().to_vec(),
    ]
    .concat()
}

fn close_encounter(instance: i32) -> Vec<u8> {
    [
        vec![41, 0, 109, 0],
        0_i32.to_le_bytes().to_vec(),
        instance.to_le_bytes().to_vec(),
    ]
    .concat()
}

fn begin_fight(
    fixture: &Fixture,
    game: &mut Gameplay,
    now: &mut u64,
) -> (CharacterState, u64, i32) {
    let world = fixture.character([0.0; 3]);
    game.handle_in_world(&select_job(43), &world, *now).unwrap();
    *now += 100;
    game.handle_in_world(&chat::local_request("realm battle 411"), &world, *now)
        .unwrap();
    let active = game
        .state()
        .encounter
        .and_then(|encounter| encounter.active)
        .expect("fixture encounter starts");
    let instance = active.instance_id;
    let mut arena = fixture.character(fixture.enemy_position);
    arena.zone_id = ENCOUNTER;
    *now += 100;
    game.initialize_world(&arena, *now).unwrap();
    *now += 100;
    game.handle_in_world(
        &select_packet(fixture.guide, GUIDE_START_CHOICE),
        &arena,
        *now,
    )
    .unwrap();
    (arena, active.enemies[0].guid, instance)
}

#[test]
fn authored_interact_actor_advances_only_the_matching_active_quest() {
    let fixture = Fixture::new();

    let mut inactive = fixture.game(None);
    let at_actor = fixture.character(fixture.interact_position);
    inactive.update_npc_visibility(&at_actor).unwrap();
    let before = saved(&inactive);
    let out = inactive
        .handle_in_world(&interact_packet(INTERACT_ACTOR), &at_actor, 1_000)
        .unwrap();
    assert!(!out.changed);
    assert_eq!(
        saved(&inactive),
        before,
        "inactive interaction changed the save"
    );

    let mut game = fixture.game(Some(fixture.active_state(INTERACT_QUEST)));
    game.update_npc_visibility(&at_actor).unwrap();
    let mut now = 2_000;

    let mut wrong_zone = at_actor.clone();
    wrong_zone.zone_id = ENCOUNTER;
    assert!(
        game.handle_in_world(&interact_packet(INTERACT_ACTOR), &wrong_zone, now)
            .is_ok_and(|out| out.replies.is_empty() && !out.changed)
    );
    assert_eq!(count(&game.state(), INTERACT_QUEST), 0);

    let mut away = at_actor.clone();
    away.position = [
        at_actor.position[0] + 1_000.0,
        at_actor.position[1],
        at_actor.position[2],
    ];
    let _ = game.handle_in_world(&interact_packet(INTERACT_ACTOR), &away, now);
    assert_eq!(count(&game.state(), INTERACT_QUEST), 0);

    let at_other = fixture.character(fixture.other_position);
    game.update_npc_visibility(&at_other).unwrap();
    now += 100;
    game.handle_in_world(&interact_packet(OTHER_ACTOR), &at_other, now)
        .unwrap();
    now += 100;
    game.handle_in_world(&select_packet(OTHER_ACTOR, 1), &at_other, now)
        .unwrap();
    assert_eq!(
        count(&game.state(), INTERACT_QUEST),
        0,
        "another actor's menu or Talk advanced the target"
    );

    game.update_npc_visibility(&at_actor).unwrap();
    now += 100;
    let advanced = game
        .handle_in_world(&interact_packet(INTERACT_ACTOR), &at_actor, now)
        .unwrap();
    assert!(advanced.changed);
    assert_eq!(count(&game.state(), INTERACT_QUEST), 1);
    let once = saved(&game);
    now += 100;
    assert!(
        !game
            .handle_in_world(&interact_packet(INTERACT_ACTOR), &at_actor, now)
            .unwrap()
            .changed
    );
    assert_eq!(count(&game.state(), INTERACT_QUEST), 1);
    assert_eq!(
        saved(&game),
        once,
        "a duplicate interaction changed the save"
    );
}

#[test]
fn authored_complete_dungeon_advances_only_on_a_win() {
    let fixture = Fixture::new();

    let mut lost = fixture.game(Some(fixture.active_state(DUNGEON_QUEST)));
    let mut now = 1_000;
    let (arena, _, _) = begin_fight(&fixture, &mut lost, &mut now);
    for _ in 0..8 {
        now += 1_000;
        lost.tick(&arena, now).unwrap();
        if lost
            .state()
            .encounter
            .as_ref()
            .and_then(|state| state.active.as_ref())
            .is_some_and(|active| active.phase == EncounterPhase::Lost)
        {
            break;
        }
    }
    assert_eq!(
        lost.state()
            .encounter
            .as_ref()
            .and_then(|state| state.active.as_ref())
            .map(|active| &active.phase),
        Some(&EncounterPhase::Lost)
    );
    assert_eq!(count(&lost.state(), DUNGEON_QUEST), 0);

    let mut won = fixture.game(Some(fixture.active_state(DUNGEON_QUEST)));
    let mut now = 1_000;
    let (arena, _, instance) = begin_fight(&fixture, &mut won, &mut now);
    now += 100;
    won.handle_in_world(&basic_attack(), &arena, now).unwrap();
    assert_eq!(
        won.state()
            .encounter
            .as_ref()
            .and_then(|state| state.active.as_ref())
            .map(|active| &active.phase),
        Some(&EncounterPhase::Won)
    );
    assert_eq!(
        count(&won.state(), DUNGEON_QUEST),
        1,
        "the win transition credits the terminal dungeon objective"
    );

    now += 100;
    let closed = won
        .handle_in_world(&close_encounter(instance), &arena, now)
        .unwrap();
    assert!(closed.changed);
    assert_eq!(
        count(&won.state(), DUNGEON_QUEST),
        1,
        "the results close does not credit again"
    );
    let after_win = saved(&won);
    now += 100;
    assert!(
        !won.handle_in_world(&close_encounter(instance), &arena, now)
            .unwrap()
            .changed
    );
    assert_eq!(count(&won.state(), DUNGEON_QUEST), 1);
    assert_eq!(saved(&won), after_win, "a replayed result changed the save");
}

#[test]
fn a_refused_opening_interaction_cannot_advance_a_regional_quest() {
    let fixture = Fixture::new();
    patch_opening_interact_actor(&fixture.gameplay);
    write_quest_content(&fixture.quests, OPENING_ACTOR, ENCOUNTER, true);
    let mut game = fixture.game(Some(fixture.active_state(INTERACT_QUEST)));
    let at_actor = fixture.character(OPENING_ACTOR_POSITION);
    let mut now = 1_000;
    game.initialize_world(&at_actor, now).unwrap();
    now += 100;
    let menu = game
        .handle_in_world(&interact_packet(OPENING_ACTOR), &at_actor, now)
        .unwrap();
    assert!(
        menu.replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 9, 0])),
        "the opening actor must offer its menu"
    );

    let before = saved(&game);
    let inventory = game.state().inventory.clone();
    let quests = game.state().npc_quests.clone();
    let coins = game.state().inventory.coins;
    now += 100;
    let refused = game
        .handle_in_world(&select_packet(OPENING_ACTOR, 1), &at_actor, now)
        .unwrap();
    assert!(!refused.changed);
    assert!(said(&refused, "Switch to Brawler"));
    assert_eq!(
        saved(&game),
        before,
        "the wrong-job refusal changed the save"
    );
    assert_eq!(game.state().inventory, inventory);
    assert_eq!(game.state().inventory.coins, coins);
    assert_eq!(game.state().npc_quests, quests);
    assert_eq!(count(&game.state(), INTERACT_QUEST), 0);

    // Positive control: the same actor and request advance once the job matches.
    now += 100;
    assert!(
        game.handle_in_world(&select_job(43), &at_actor, now)
            .unwrap()
            .changed
    );
    now += 100;
    game.handle_in_world(&interact_packet(OPENING_ACTOR), &at_actor, now)
        .unwrap();
    now += 100;
    let accepted = game
        .handle_in_world(&select_packet(OPENING_ACTOR, 1), &at_actor, now)
        .unwrap();
    assert!(accepted.changed);
    assert_eq!(count(&game.state(), INTERACT_QUEST), 1);
    // 49/9 `QuestObjectiveUpdatePacket` reader 0x00c7af60 with common prefix 0x00c7acd0:
    // quest, objective, current, progress, bool, trailing dword = 6 + 21 = 27 bytes.
    let update = accepted
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[49, 0, 9, 0, 0, 0]))
        .expect("the advance emits 49/9");
    let mut expected = vec![49, 0, 9, 0, 0, 0];
    expected.extend(INTERACT_QUEST.to_le_bytes());
    expected.extend(INTERACT_OBJECTIVE.to_le_bytes());
    expected.extend(1_i32.to_le_bytes());
    expected.extend(1.0_f32.to_le_bytes());
    expected.push(0);
    expected.extend(0_i32.to_le_bytes());
    assert_eq!(update, &expected, "49/9 must match reader 0x00c7af60");
    // 49/10 `QuestObjectiveCompletePacket` reader 0x00c7b070: quest, objective, progress,
    // trailing dword, bool = 6 + 17 = 23 bytes.
    let complete = accepted
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[49, 0, 10, 0, 0, 0]))
        .expect("the advance emits 49/10");
    let mut expected = vec![49, 0, 10, 0, 0, 0];
    expected.extend(INTERACT_QUEST.to_le_bytes());
    expected.extend(INTERACT_OBJECTIVE.to_le_bytes());
    expected.extend(1.0_f32.to_le_bytes());
    expected.extend(0_i32.to_le_bytes());
    expected.push(0);
    assert_eq!(complete, &expected, "49/10 must match reader 0x00c7b070");

    now += 100;
    assert!(
        !game
            .handle_in_world(&select_packet(OPENING_ACTOR, 1), &at_actor, now)
            .unwrap()
            .changed,
        "a repeated accepted talk must not advance the objective again"
    );
    assert_eq!(count(&game.state(), INTERACT_QUEST), 1);
}

#[test]
fn a_live_activity_refuses_regional_interaction_and_keeps_the_menu() {
    let fixture = Fixture::new();
    fixture.add_mining();
    let node = copper_vein();
    let mut state = fixture.active_state(INTERACT_QUEST);
    {
        let profiles = state.profiles.as_mut().unwrap();
        profiles.unlocked.insert(MINER);
        profiles.active = MINER;
    }
    let mut game = fixture.game(Some(state));
    let at_other = fixture.character(fixture.other_position);
    let mut now = 1_000;
    game.initialize_world(&at_other, now).unwrap();
    now += 100;
    // An idle regional menu that does not target the active quest, left open across the start.
    game.handle_in_world(&interact_packet(OTHER_ACTOR), &at_other, now)
        .unwrap();
    now += 100;
    let at_node = fixture.character([node.position[0] + 2.0, node.position[1], node.position[2]]);
    let started = game
        .handle_in_world(&interact_packet(node.guid), &at_node, now)
        .unwrap();
    assert!(started.changed);
    assert!(
        game.state().mining.as_ref().unwrap().active.is_some(),
        "the node starts a real Type8 run"
    );

    let before = saved(&game);
    let quests = game.state().npc_quests.clone();
    now += 100;
    let refused = game
        .handle_in_world(&interact_packet(INTERACT_ACTOR), &at_node, now)
        .unwrap();
    assert!(!refused.changed);
    assert!(said(
        &refused,
        "Finish the active activity before interacting."
    ));
    assert!(
        refused
            .replies
            .iter()
            .all(|packet| !packet.starts_with(&[26, 0, 9, 0])),
        "a busy interaction must not open a menu"
    );
    assert_eq!(saved(&game), before, "a busy 26/8 changed the save");
    assert_eq!(game.state().npc_quests, quests);
    assert_eq!(count(&game.state(), INTERACT_QUEST), 0);

    now += 100;
    let refused_select = game
        .handle_in_world(&select_packet(OTHER_ACTOR, 1), &at_other, now)
        .unwrap();
    assert!(!refused_select.changed);
    assert!(said(
        &refused_select,
        "Finish the active activity before interacting."
    ));
    assert_eq!(saved(&game), before, "a busy 26/10 changed the save");

    now += 100;
    game.handle_in_world(&[26, 0, 42, 0], &at_other, now)
        .unwrap();
    assert!(game.state().mining.as_ref().unwrap().active.is_none());

    // The refusal did not consume the menu: the same selection now reaches Talk.
    now += 100;
    let dialogue = game
        .handle_in_world(&select_packet(OTHER_ACTOR, 1), &at_other, now)
        .unwrap();
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "the busy refusal must not consume the open menu"
    );

    // The earlier interaction was only busy-blocked: the same request now advances.
    let at_actor = fixture.character(fixture.interact_position);
    now += 100;
    let advanced = game
        .handle_in_world(&interact_packet(INTERACT_ACTOR), &at_actor, now)
        .unwrap();
    assert!(advanced.changed);
    assert_eq!(count(&game.state(), INTERACT_QUEST), 1);
}

#[test]
fn refused_or_forged_goal_inputs_change_nothing() {
    let fixture = Fixture::new();
    let flat_actor = fixture.root.join("flat-malformed-actor");
    write_flat_validation_fixture(
        &flat_actor,
        json!({
            "kind": "interact_actor",
            "id": 1,
            "actor_guid": 0,
            "source": [{
                "kind": "reconstructed",
                "evidence": "(R) T105 malformed flat actor"
            }]
        }),
    );
    assert!(ContentDb::load(&flat_actor).is_err());
    let flat_encounter = fixture.root.join("flat-malformed-encounter");
    write_flat_validation_fixture(
        &flat_encounter,
        json!({
            "kind": "complete_dungeon",
            "id": 1,
            "encounter_id": 0,
            "source": [{
                "kind": "reconstructed",
                "evidence": "(R) T105 malformed flat encounter"
            }]
        }),
    );
    assert!(ContentDb::load(&flat_encounter).is_err());

    let malformed_actor = fixture.root.join("malformed-actor");
    write_quest_content(&malformed_actor, 0, ENCOUNTER, true);
    assert!(
        QuestEngine::from_split_content(
            &malformed_actor,
            CHARACTER,
            CharacterEligibility::default(),
            None
        )
        .is_err()
    );
    let malformed_encounter = fixture.root.join("malformed-encounter");
    write_quest_content(&malformed_encounter, INTERACT_ACTOR, 0, true);
    assert!(
        QuestEngine::from_split_content(
            &malformed_encounter,
            CHARACTER,
            CharacterEligibility::default(),
            None
        )
        .is_err()
    );
    let missing_source = fixture.root.join("missing-source");
    write_quest_content(&missing_source, INTERACT_ACTOR, ENCOUNTER, false);
    assert!(
        QuestEngine::from_split_content(
            &missing_source,
            CHARACTER,
            CharacterEligibility::default(),
            None
        )
        .is_err()
    );

    let mut engine = fixture.engine();
    assert!(engine.offer(INTERACT_QUEST, INTERACT_ACTOR).changed);
    assert!(engine.accept(INTERACT_QUEST).changed);
    let engine_before = engine.state().clone();
    assert!(
        !engine
            .on_event(QuestEvent::InteractNpc {
                npc_id: OTHER_ACTOR
            })
            .unwrap()
            .changed
    );
    assert!(
        !engine
            .on_event(QuestEvent::CompleteDungeon {
                encounter_id: ENCOUNTER + 1
            })
            .unwrap()
            .changed
    );
    assert_eq!(engine.state(), &engine_before);
    assert_eq!(engine.progress(INTERACT_QUEST).unwrap().counts, [0]);

    let mut game = fixture.game(Some(fixture.active_state(INTERACT_QUEST)));
    let at_actor = fixture.character(fixture.interact_position);
    game.update_npc_visibility(&at_actor).unwrap();
    let before = saved(&game);
    let wallet = game.state().inventory.coins;
    let quest_state = game.state().npc_quests.clone();

    let claimed = game
        .handle_in_world(&client_claim(INTERACT_OBJECTIVE), &at_actor, 1_000)
        .unwrap();
    assert!(!claimed.changed);
    assert_eq!(
        saved(&game),
        before,
        "a forged client claim changed the save"
    );
    assert_eq!(game.state().inventory.coins, wallet);
    assert_eq!(game.state().npc_quests, quest_state);
    assert_eq!(count(&game.state(), INTERACT_QUEST), 0);
}
