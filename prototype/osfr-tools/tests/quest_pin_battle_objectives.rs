//! T427: the "Take Me There" pin (47/14) for dungeon and battle-only objectives, and the
//! pin re-sync after movement, shared-field kill credit and ticks.
//!
//! A `complete_dungeon` step pins its field battle's marker (the spawner actor); a kill
//! step whose kind never roams the field pins the lowest-id battle that fields it (R).
//! Movement (`Gameplay::on_location`), the killer's shared-field request
//! (`handle_shared_field`), the shared tick (`tick_shared_effects`) and the world tick
//! now re-sync the pin, so it never waits on the next non-movement packet.
//!
//! Every test drives a production seam with shipped content: the NPC menu (26/8 ->
//! 26/10 -> 26/6 -> 49/1 -> 49/2), the client's 47/12 select, the battle marker 26/8 ->
//! 41/114 -> 41/108 Go -> arena zone ready -> win -> 26/42 Close -> world zone ready,
//! `on_location`, `handle_shared_field`, `tick_shared_effects` and `tick`. Shared
//! fixtures come from `tests/support/npc_quest_chain.rs`.
//!
//! Client evidence (IDA, 2014 client):
//! * 47/14 ObjectiveTargetUpdate constructor `0x00a88080`, handler `0x00a8fab0`: active
//!   byte, three task dwords, guid, name id, x/y/z/w, zone. The handler hands the
//!   packet's position to `0x009cd6c0`/`0x009ce260` (the waypoint the tracker distance,
//!   route and Take Me There use) and the guid only to `0x00a8e660`, which highlights a
//!   minimap indicator of that guid when one exists (`0x00a88f90`) and otherwise clears
//!   the highlight (`0x00a88280`). An inactive 47/14 clears the target.
//! * 47/12 SelectQuest writer `0x00c7c250`, constructor `0x00c79e40`: one i32 quest id.
//! * 41/108 Go constructor `0x008b6e70` (sender `0x00938f00`): the 41/114 encounter and
//!   instance pair, then eight zero bytes. 26/42 results Close `0x008a2eb0`.

#[path = "support/npc_quest_chain.rs"]
mod chain;
#[path = "common/combat_scale.rs"]
mod combat_scale;

use chain::{
    CHARACTER, Fixture, accept, character_at_for, content_dir, content_position, counts, interact,
    objective_complete, offer, read_json, send,
};
use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    chat,
    encounter::{EncounterEngine, SharedFieldAction},
    gameplay::{Gameplay, Outcome},
    quest_engine::{CharacterEligibility, QuestEngine, QuestProgress, QuestState, QuestStatus},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};

const ADVENTURER: i32 = 1;
const BRAWLER: i32 = 43;
/// "Investigate the Wolves": Anne's one `complete_dungeon 12350` step.
const INVESTIGATE: u32 = 3_000_061;
const ANNE: u64 = 9_500_000_266;
/// Sheep Watch (battle 12350): the spawner marker in `field_battles.json`.
const SHEEP_WATCH_GUID: u64 = 100_001_235_000;
const SHEEP_WATCH: [f32; 3] = [-2165.513, -32.96, 328.1112];
/// Carlos's C2: `complete_dungeon 12400`, then a Talk back to Carlos.
const RING_RETURN: u32 = 3_030_005;
const RING_FIRST: u32 = 3_030_004;
const CARLOS: u64 = 9_000_000_042;
const GATE: u64 = 100_000_124_000;
const GATE_POSITION: [f32; 3] = [-1908.92576, -39.4, 410.69792];
const ARENA_ZONE: u32 = 9_600;
const ARENA_ARRIVAL: [f32; 3] = [136.0, -0.5, 155.0];
/// "Missing Sisters": visit the wolf field, three 3156 kills, Talk to Geoffrey.
const MISSING_SISTERS: u32 = 3_000_029;
const GEOFFREY: u64 = 9_000_000_330;
const WOLF_CENTRE: [f32; 3] = [-1899.204, -50.59, 307.902];
/// "Fighting the Pack": five 3156 kills, then a Talk to its giver.
const PACK: u32 = 3_000_009;
const PACK_GIVER: u64 = 9_000_000_050;
const WOLF_KIND: i32 = 3_156;
const WOLF_GUID: u64 = 131_000_109_001;
const WOLF_HOME: [f32; 3] = [0.0, 0.0, 0.0];

fn select_packet(quest_id: u32) -> Vec<u8> {
    let mut packet = vec![47, 0, 12];
    packet.extend((quest_id as i32).to_le_bytes());
    packet
}

/// The active 47/14 written field by field from the client body (`0x00a88080`).
fn active_pin(quest: u32, objective: u32, guid: u64, position: [f32; 3]) -> Vec<u8> {
    let mut packet = vec![47, 0, 14, 1];
    for dword in [quest, 0, objective] {
        packet.extend(dword.to_le_bytes());
    }
    packet.extend(guid.to_le_bytes());
    packet.extend(0u32.to_le_bytes());
    for float in [position[0], position[1], position[2], 1.0] {
        packet.extend(float.to_le_bytes());
    }
    packet.extend(0u32.to_le_bytes()); // zone 0, the open world
    packet
}

const CLEARED: [u8; 4] = [47, 0, 14, 0];

fn pins(outcome: &Outcome) -> Vec<Vec<u8>> {
    outcome
        .replies
        .iter()
        .filter(|packet| packet.starts_with(&[47, 0, 14]))
        .cloned()
        .collect()
}

/// A session over the opening fixture with the shipped split catalogue, optionally
/// restored from `quests`, after the production world-entry pass at `at`.
fn player(
    fixture: &Fixture,
    guid: u64,
    active: i32,
    quests: Option<QuestState>,
    at: &CharacterState,
) -> Gameplay {
    let mut state = Gameplay::load(fixture.path(), None)
        .expect("the opening fixture loads")
        .state();
    let profiles = state
        .profiles
        .as_mut()
        .expect("the fixture has job profiles");
    profiles.active = active;
    profiles.unlocked.insert(active);
    state.npc_quests = quests;
    let mut game = fixture.game_as(guid, Some(state));
    game.initialize_world(at, 0).expect("the world initializes");
    game
}

/// A saved quest ledger in which `completed` are done and `quest` was offered and
/// accepted by the real engine, then progressed to `counts`.
fn ledger(
    guid: u64,
    completed: &[(u32, Vec<u32>)],
    quest: u32,
    giver: u64,
    counts: Vec<u32>,
) -> QuestState {
    let eligibility = CharacterEligibility {
        player_level: 20,
        job_levels: BTreeMap::from([("brawler".into(), 20), ("adventurer".into(), 20)]),
        member: true,
        nonmember_quest_limit: 30,
        member_quest_limit: 30,
    };
    let mut state =
        QuestEngine::from_split_content(&content_dir(), guid, eligibility.clone(), None)
            .unwrap()
            .state()
            .clone();
    for (id, done) in completed {
        state.quests.insert(
            *id,
            QuestProgress {
                status: QuestStatus::Completed,
                counts: done.clone(),
            },
        );
    }
    let mut engine =
        QuestEngine::from_split_content(&content_dir(), guid, eligibility, Some(state)).unwrap();
    assert!(
        engine.offer(quest, giver).changed,
        "quest {quest} is offered"
    );
    assert!(engine.accept(quest).changed, "quest {quest} is accepted");
    let mut state = engine.state().clone();
    state.quests.get_mut(&quest).unwrap().counts = counts;
    state
}

#[test]
fn investigate_the_wolves_pins_the_sheep_watch_marker_and_idle_ticks_send_nothing() {
    let fixture = Fixture::new();
    let at = character_at_for(
        CHARACTER,
        0,
        content_position("farnum_farmstead.json", ANNE),
    );
    let mut game = player(&fixture, CHARACTER, ADVENTURER, None, &at);

    // Accepted at Anne through the real NPC menu. Accepting does not select a quest, so
    // nothing is pinned yet (the W161 trigger rules).
    offer(&mut game, &at, ANNE, INVESTIGATE, 1_000);
    let accepted = accept(&mut game, &at, INVESTIGATE, 1_010);
    assert!(pins(&accepted).is_empty());

    // The client's 47/12 selects it: the complete_dungeon step pins the battle marker.
    let selected = send(&mut game, &at, &select_packet(INVESTIGATE), 1_020);
    assert_eq!(
        pins(&selected),
        vec![active_pin(INVESTIGATE, 1, SHEEP_WATCH_GUID, SHEEP_WATCH)]
    );

    // An idle tick, a still stand and an unrelated packet leave the shown pin alone.
    for now in [1_200, 1_400] {
        let ticked = game.tick(&at, now).expect("the world ticks");
        assert!(pins(&ticked).is_empty(), "an idle tick sends no 47/14");
    }
    let stood = game.on_location(&at, 1_500).unwrap();
    assert!(pins(&stood).is_empty());
    let again = send(&mut game, &at, &select_packet(INVESTIGATE), 1_600);
    assert!(pins(&again).is_empty());
}

fn go(details: &[u8]) -> Vec<u8> {
    let mut packet = vec![41, 0, 108, 0];
    packet.extend(&details[4..12]);
    packet.extend(0_u64.to_le_bytes());
    packet
}

#[test]
fn the_pin_clears_in_the_arena_and_moves_to_the_talk_back_after_the_return() {
    let fixture = Fixture::new();
    let carlos = content_position("cobblestone_village.json", CARLOS);
    let mut at = character_at_for(CHARACTER, 0, carlos);
    let quests = ledger(
        CHARACTER,
        &[(RING_FIRST, vec![1, 1])],
        RING_RETURN,
        CARLOS,
        vec![0, 0],
    );
    let mut game = player(&fixture, CHARACTER, BRAWLER, Some(quests), &at);
    game.set_dev_tools(true);
    game.set_role(AccountRole::Admin);

    let selected = send(&mut game, &at, &select_packet(RING_RETURN), 1_000);
    assert_eq!(
        pins(&selected),
        vec![active_pin(RING_RETURN, 1, GATE, GATE_POSITION)]
    );

    // Marker -> start screen -> Go -> the arena's zone ready clears the pin once.
    at.position = GATE_POSITION;
    let screen = interact(&mut game, &at, GATE, 1_100);
    let details = screen
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[41, 0, 114, 0]))
        .cloned()
        .expect("the gate opens the 41/114 start screen");
    let zoned = send(&mut game, &at, &go(&details), 1_200);
    assert_eq!(zoned.zoning.expect("Go zones").zone_id, ARENA_ZONE);
    at.zone_id = ARENA_ZONE;
    at.position = ARENA_ARRIVAL;
    let loaded = game.initialize_world(&at, 1_300).unwrap();
    assert_eq!(pins(&loaded), vec![CLEARED.to_vec()]);

    // The win completes the dungeon step; the Talk back is in another zone, so the pin
    // stays cleared inside the arena.
    let won = send(&mut game, &at, &chat::local_request("realm win"), 1_400);
    assert!(won.changed);
    assert_eq!(counts(&game, RING_RETURN), [1, 0]);
    assert!(pins(&won).is_empty());

    // 26/42 Close and the open world's zone ready: the pin moves to Carlos.
    let closed = send(&mut game, &at, &[26, 0, 42, 0], 1_500);
    let zone = closed.zoning.expect("Close returns to the open world");
    at.zone_id = zone.zone_id;
    at.position = zone.position;
    let returned = game.initialize_world(&at, 1_600).unwrap();
    assert_eq!(
        pins(&returned),
        vec![active_pin(RING_RETURN, 2, CARLOS, carlos)]
    );
}

#[test]
fn walking_onto_a_visit_objective_moves_the_pin_in_the_same_location_outcome() {
    let fixture = Fixture::new();
    let at = character_at_for(CHARACTER, 0, content_position("sanctuary.json", GEOFFREY));
    let mut game = player(&fixture, CHARACTER, ADVENTURER, None, &at);
    offer(&mut game, &at, GEOFFREY, MISSING_SISTERS, 1_000);
    accept(&mut game, &at, MISSING_SISTERS, 1_010);
    let selected = send(&mut game, &at, &select_packet(MISSING_SISTERS), 1_020);
    assert_eq!(
        pins(&selected),
        vec![active_pin(MISSING_SISTERS, 1, 0, WOLF_CENTRE)]
    );

    // The next step's place: the lowest-guid shipped field wolf (`field_enemy_site`).
    let field = read_json(&content_dir().join("field_enemies.json"));
    let wolf = field["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|enemy| enemy["definition_id"] == WOLF_KIND)
        .min_by_key(|enemy| enemy["guid"].as_u64().unwrap())
        .unwrap();
    let home: [f32; 3] = serde_json::from_value(wolf["position"].clone()).unwrap();

    let walked = game
        .on_location(&character_at_for(CHARACTER, 0, WOLF_CENTRE), 1_100)
        .expect("movement is accepted");
    assert!(objective_complete(&walked, MISSING_SISTERS, 1));
    assert_eq!(
        pins(&walked),
        vec![active_pin(
            MISSING_SISTERS,
            2,
            wolf["guid"].as_u64().unwrap(),
            home
        )]
    );
    let still = game
        .on_location(&character_at_for(CHARACTER, 0, WOLF_CENTRE), 1_200)
        .unwrap();
    assert!(pins(&still).is_empty());
}

fn synthetic_wolf() -> Value {
    json!({
        "guid": WOLF_GUID,
        "definition_id": WOLF_KIND,
        "model_id": 3,
        "name_id": 100,
        "name": "Synthetic wolf",
        "texture_alias": "",
        "position": WOLF_HOME,
        "max_health": 200,
        "damage": 1,
        "boss": false,
        "level": 1,
        "respawn_seconds": 60,
        "aggro_range": 8.0,
        "leash_radius": 12.0,
        "behaviour": {"mode":"stationary","reach":2.5,"attack_ms":1000},
        "quest_ids": []
    })
}

fn attack(
    game: &mut Gameplay,
    field: &mut osfr_tools::encounter::SharedFieldState,
    character: &CharacterState,
    known: &BTreeSet<u64>,
    now: u64,
) -> Outcome {
    game.handle_shared_field(
        &SharedFieldAction::Ability {
            slot: 0,
            target: WOLF_GUID,
        },
        field,
        known,
        character,
        now,
    )
    .expect("the shared attack is accepted")
}

#[test]
fn a_shared_field_kill_moves_the_killers_and_the_contributors_pin() {
    // Two Brawlers four wolves into "Fighting the Pack" share one wolf: B hits, A lands the
    // lethal swing (credited in A's request), B is credited on its tick.
    let (a_guid, b_guid) = (101, 202);
    let fixture = Fixture::for_character(a_guid).with_field_enemies(&[synthetic_wolf()]);
    let a_at = character_at_for(a_guid, 0, [1.0, 0.0, 0.0]);
    let b_at = character_at_for(b_guid, 0, [-1.0, 0.0, 0.0]);
    let quests = |guid| ledger(guid, &[], PACK, PACK_GIVER, vec![4, 0]);
    let mut a = player(&fixture, a_guid, BRAWLER, Some(quests(a_guid)), &a_at);
    let mut b = player(&fixture, b_guid, BRAWLER, Some(quests(b_guid)), &b_at);
    let mut field = a.take_shared_fields().expect("the shared field exists");
    b.take_shared_fields();
    let known = BTreeSet::from([WOLF_GUID]);

    // The kill step pins the wolf's home although the live rows now sit in the shared
    // field, as every production session's do (`osfr-gateway` takes them at login).
    let at_wolf = active_pin(PACK, 1, WOLF_GUID, WOLF_HOME);
    assert_eq!(
        pins(&send(&mut a, &a_at, &select_packet(PACK), 10)),
        vec![at_wolf.clone()]
    );
    assert_eq!(
        pins(&send(&mut b, &b_at, &select_packet(PACK), 10)),
        vec![at_wolf]
    );

    let giver = active_pin(
        PACK,
        2,
        PACK_GIVER,
        content_position("farnum_farmstead.json", PACK_GIVER),
    );
    let hit = attack(&mut b, &mut field, &b_at, &known, 100);
    assert!(
        pins(&hit).is_empty(),
        "a hit that credits nothing keeps the pin"
    );
    // A swings until the wolf falls; a swing that does not kill credits nothing. The
    // bound is the hits its live-scale health needs, so tuning the scale keeps the story.
    let hits = combat_scale::enemy_health(200).div_ceil(combat_scale::strike(BRAWLER, 1)) + 1;
    let mut now = 200;
    let mut killed = attack(&mut a, &mut field, &a_at, &known, now);
    for _ in 1..hits {
        if field.health(WOLF_GUID) == Some(0) {
            break;
        }
        assert!(
            pins(&killed).is_empty(),
            "a swing that credits nothing keeps the pin"
        );
        now += 700;
        killed = attack(&mut a, &mut field, &a_at, &known, now);
    }
    assert_eq!(
        field.health(WOLF_GUID),
        Some(0),
        "A kills the wolf within the bound"
    );
    assert_eq!(counts(&a, PACK), [5, 0]);
    assert_eq!(
        pins(&killed),
        vec![giver.clone()],
        "the killer's own request"
    );

    let claimed = b.tick_shared_effects(&mut field, &b_at, now + 100).unwrap();
    assert_eq!(counts(&b, PACK), [5, 0]);
    assert_eq!(pins(&claimed), vec![giver], "the contributor's shared tick");
    let idle = b.tick_shared_effects(&mut field, &b_at, now + 200).unwrap();
    assert!(pins(&idle).is_empty());
}

/// The battle-kind fallback's (R) tie-break: the lowest battle id wins, not file order.
#[test]
fn the_battle_kind_fallback_takes_the_lowest_battle_id() {
    let shipped = read_json(&content_dir().join("field_battles.json"));
    let ninety_six = shipped["battles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|battle| battle["id"] == 96)
        .cloned()
        .unwrap();
    // A second battle fielding the same kind, listed after 96 but with a lower id.
    let mut lower = ninety_six.clone();
    lower["id"] = json!(90);
    lower["spawner"]["guid"] = json!(100_000_090_000_u64);
    lower["spawner"]["position"] = json!([1.0, 2.0, 3.0]);
    for (index, enemy) in lower["enemies"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .enumerate()
    {
        enemy["guid"] = json!(100_000_090_001_u64 + index as u64);
    }
    let dir = std::env::temp_dir().join(format!("osfr-t427-battles-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("field_battles.json");
    let mut file = shipped.clone();
    file["battles"] = json!([ninety_six, lower]);
    fs::write(&path, serde_json::to_vec(&file).unwrap()).unwrap();

    let mut engine = EncounterEngine::load(chain::reference_content(), None).unwrap();
    engine.load_field_battles(&path).expect("the battles load");
    fs::remove_dir_all(&dir).unwrap();

    assert_eq!(
        engine.battle_marker_for_enemy(930_096),
        Some((100_000_090_000, [1.0, 2.0, 3.0]))
    );
    assert_eq!(
        engine.battle_marker(96),
        Some((100_000_096_000, [-440.0, 0.8, 1290.0]))
    );
    // A battle that is not loaded, and a kind no battle fields: nothing, not a guess.
    assert_eq!(engine.battle_marker(12_350), None);
    assert_eq!(engine.battle_marker_for_enemy(930_050), None);
    assert_eq!(engine.battle_marker_for_enemy(u32::MAX), None);
}
