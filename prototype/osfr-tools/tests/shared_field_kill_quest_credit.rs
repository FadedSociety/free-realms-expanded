//! T256: a shared-field kill credits the killer's active kill quests.
//!
//! Every test drives a production seam: `Gameplay::handle_shared_field` (the
//! request `Session::commit_outcome` runs), `Gameplay::tick_shared_effects` (the
//! shared tick), `Gameplay::load` + `bind_npc_quest_catalogue` (the reconnect
//! bind), or `Gameplay::handle_in_world` for the non-shared solo strike.
//!
//! The quest is the shipped T241 playtest entry 3000009 "Fighting the Pack"
//! (`data/content/quests/adventurer.json`): five `3156` kills from giver
//! 9000000050, then one Talk to that giver. The field actors are a synthetic
//! two-engine population, as in `tests/shared_field_state.rs`.
use osfr_tools::{
    character_store::CharacterState,
    encounter::SharedFieldAction,
    gameplay::{Gameplay, Outcome},
    quest_engine::{CharacterEligibility, QuestEngine, QuestState, QuestStatus},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const A: u64 = 101;
const B: u64 = 202;
/// T241: "Fighting the Pack" is five wolf defeats, then a Talk to the giver.
const QUEST: u32 = 3_000_009;
const GIVER: u64 = 9_000_000_050;
/// The quest objective ids in `adventurer.json`.
const DEFEAT_OBJECTIVE: u32 = 1;
const TALK_OBJECTIVE: u32 = 2;
const WOLF: i32 = 3156;
const BRAWLER: i32 = 43;
const TALK_CHOICE: i32 = 1;
const ATTACK_CHOICE: i32 = 103;
/// The shipped quest reward (`data/content/quests/adventurer.json`).
const QUEST_COINS: u32 = 18;
const QUEST_JOB_XP: u32 = 25;
const WOLF_GUID: u64 = 131_000_109_001;
const WOLF_TWO: u64 = 131_000_109_002;
const LEVEL_ONE: i32 = 930_108;
const CLIENT_QUEST_CHOICE: i32 = 301;
/// The quest's 49/13 end text (`end_text_string_id` in the quest record).
const QUEST_END_TEXT: u32 = 4_420_076;
static SERIAL: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    dir: PathBuf,
}

impl Fixture {
    fn new(enemies: &[Value]) -> Self {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "t256-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        copy_missing(&opening_content(), &dir);
        copy_missing(&content_dir(), &dir);
        // The opening inventory does not carry every optional farming reward;
        // this suite exercises field combat only (same rule as T240).
        let _ = fs::remove_file(dir.join("harvesting.json"));
        fs::write(
            dir.join("field_enemies.json"),
            serde_json::to_vec(&json!({"schema_version": 1, "enemies": enemies})).unwrap(),
        )
        .unwrap();
        Self { dir }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn opening_content() -> PathBuf {
    std::env::var_os("OSFR_OPENING_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/opening-anvil-placement-v1/content")
        })
}

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn copy_missing(from: &Path, to: &Path) {
    for row in fs::read_dir(from).unwrap() {
        let path = row.unwrap().path();
        let target = to.join(path.file_name().unwrap());
        if path.is_dir() {
            fs::create_dir_all(&target).unwrap();
            copy_missing(&path, &target);
        } else if !target.exists() {
            fs::copy(path, target).unwrap();
        }
    }
}

/// A field wolf. `definition_id` is the quest's `defeat`/`model_id` target; 0
/// is accepted by the content loader (`validate_actor` refuses negatives only).
fn wolf(guid: u64, definition_id: i32, position: [f32; 3], max_health: u32) -> Value {
    json!({
        "guid": guid,
        "definition_id": definition_id,
        "model_id": 3,
        "name_id": 100,
        "name": "Synthetic wolf",
        "texture_alias": "",
        "position": position,
        "max_health": max_health,
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

/// Five reachable wolves for the five-kill objective.
fn five_wolves() -> Vec<Value> {
    [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 1.0],
        [2.0, 0.0, 0.0],
        [0.0, 0.0, 2.0],
        [2.0, 0.0, 2.0],
    ]
    .into_iter()
    .enumerate()
    .map(|(index, position)| wolf(WOLF_GUID + index as u64, WOLF, position, 100))
    .collect()
}

fn eligibility() -> CharacterEligibility {
    CharacterEligibility {
        player_level: 20,
        job_levels: BTreeMap::from([("brawler".into(), 20)]),
        member: false,
        nonmember_quest_limit: 30,
        member_quest_limit: 30,
    }
}

/// Build the real quest engine and start 3000009. The saved `QuestState` is
/// what the production reconnect restores through `bind_npc_quest_catalogue`.
fn active_quests(content: &Path, guid: u64) -> QuestState {
    let mut engine = QuestEngine::from_split_content(content, guid, eligibility(), None).unwrap();
    assert!(
        engine.offer(QUEST, GIVER).changed,
        "quest {QUEST} is offered"
    );
    assert!(engine.accept(QUEST).changed, "quest {QUEST} is accepted");
    engine.state().clone()
}

fn character_at(guid: u64, position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid,
        zone_id: 0,
        position,
        rotation: [0.0, 0.0, 1.0],
    }
}

/// One player's session with the opening engine the kill path needs and an
/// active 3000009. `profile` 43 (Brawler) is the quest's job gate and deals 100
/// per basic swing.
fn player(fixture: &Fixture, guid: u64, position: [f32; 3], coins: u32) -> Gameplay {
    let base = Gameplay::load(&fixture.dir, None).unwrap();
    let mut state = base.state();
    state.inventory.coins = coins;
    state.npc_quests = Some(active_quests(&fixture.dir, guid));
    {
        let profiles = state.profiles.as_mut().unwrap();
        profiles.active = BRAWLER;
        profiles.unlocked.insert(BRAWLER);
    }
    let mut game = Gameplay::load(&fixture.dir, Some(state)).unwrap();
    game.bind_npc_quest_catalogue(&fixture.dir, guid).unwrap();
    game.initialize_world(&character_at(guid, position), 0)
        .unwrap();
    game
}

fn reconnect(fixture: &Fixture, saved: &osfr_tools::gameplay::GameplayState) -> Gameplay {
    let mut game = Gameplay::load(&fixture.dir, Some(saved.clone())).unwrap();
    game.bind_npc_quest_catalogue(&fixture.dir, A).unwrap();
    game.initialize_world(&character_at(A, [1.0, 0.0, 0.0]), 0)
        .unwrap();
    game
}

/// Production shared-field request: a basic attack on a field target.
fn attack(
    game: &mut Gameplay,
    field: &mut osfr_tools::encounter::SharedFieldState,
    character: &CharacterState,
    known: &BTreeSet<u64>,
    target: u64,
    now_ms: u64,
) -> std::io::Result<Outcome> {
    game.handle_shared_field(
        &SharedFieldAction::Ability { slot: 0, target },
        field,
        known,
        character,
        now_ms,
    )
}

fn counts(game: &Gameplay, quest: u32) -> Vec<u32> {
    game.state().npc_quests.as_ref().unwrap().quests[&quest]
        .counts
        .clone()
}

fn status(game: &Gameplay, quest: u32) -> QuestStatus {
    game.state().npc_quests.as_ref().unwrap().quests[&quest]
        .status
        .clone()
}

fn count(replies: &[Vec<u8>], family: u16, subtype: u16) -> usize {
    replies
        .iter()
        .filter(|packet| {
            packet.starts_with(&family.to_le_bytes()) && packet[2..4] == subtype.to_le_bytes()
        })
        .count()
}

fn family(packet: &[u8], family: u16, subtype: u16) -> bool {
    packet.starts_with(&family.to_le_bytes()) && packet[2..4] == subtype.to_le_bytes()
}

fn has_quest_packet(replies: &[Vec<u8>], subtype: u16, quest: u32) -> bool {
    replies.iter().any(|packet| {
        family(packet, 49, subtype)
            && packet
                .get(6..10)
                .is_some_and(|id| u32::from_le_bytes(id.try_into().unwrap()) == quest)
    })
}

/// 49/9 ObjectiveUpdate: 6-byte family/subtype header (u16 + i32), then quest,
/// objective key and current counter. Client readers 0x00c7acd0/0x00c7af60.
fn objective_update(replies: &[Vec<u8>], quest: u32, objective: u32, current: u32) -> bool {
    replies.iter().any(|packet| {
        family(packet, 49, 9)
            && packet.len() >= 18
            && u32::from_le_bytes(packet[6..10].try_into().unwrap()) == quest
            && u32::from_le_bytes(packet[10..14].try_into().unwrap()) == objective
            && u32::from_le_bytes(packet[14..18].try_into().unwrap()) == current
    })
}

fn objective_complete(replies: &[Vec<u8>], quest: u32, objective: u32) -> bool {
    replies.iter().any(|packet| {
        family(packet, 49, 10)
            && packet.len() >= 14
            && u32::from_le_bytes(packet[6..10].try_into().unwrap()) == quest
            && u32::from_le_bytes(packet[10..14].try_into().unwrap()) == objective
    })
}

fn removes(replies: &[Vec<u8>], guid: u64) -> usize {
    replies
        .iter()
        .filter(|packet| {
            family(packet, 35, 3) && packet.get(6..14) == Some(guid.to_le_bytes().as_slice())
        })
        .count()
}

fn enemy_hp(replies: &[Vec<u8>], guid: u64) -> Option<u32> {
    replies.iter().find_map(|packet| {
        (family(packet, 35, 5) && packet.get(4..12) == Some(guid.to_le_bytes().as_slice()))
            .then(|| u32::from_le_bytes(packet[12..16].try_into().unwrap()))
    })
}

fn npc_interact(npc: u64) -> Vec<u8> {
    let mut packet = vec![26, 0, 8, 0];
    packet.extend(npc.to_le_bytes());
    packet.push(0);
    packet
}

fn npc_select(npc: u64, choice: i32) -> Vec<u8> {
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(npc.to_le_bytes());
    packet.extend(choice.to_le_bytes());
    packet
}

fn dialogue_choice() -> Vec<u8> {
    let mut packet = vec![26, 0, 6, 0];
    packet.extend(CLIENT_QUEST_CHOICE.to_le_bytes());
    packet
}

fn quest_end_reply(quest: u32) -> Vec<u8> {
    // Client QuestEndReplyPacket constructor 0x00a864c0; sender 0x00a97ef0.
    let mut packet = vec![49, 0, 14, 0, 0, 0];
    packet.extend(quest.to_le_bytes());
    packet
}

fn giver_position() -> [f32; 3] {
    let document: Value = serde_json::from_slice(
        &fs::read(content_dir().join("npcs/farnum_farmstead.json")).unwrap(),
    )
    .unwrap();
    let npc = document["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["npc_id"].as_u64() == Some(GIVER))
        .expect("the quest giver is authored");
    serde_json::from_value(npc["position"].clone()).unwrap()
}

/// Talk to the giver and choose its quest dialogue row; the 49/13 end screen is
/// released by the client's 26/6 response.
fn open_end_screen(game: &mut Gameplay, at: &CharacterState, now: u64) -> Vec<u8> {
    game.handle_in_world(&npc_interact(GIVER), at, now).unwrap();
    let dialogue = game
        .handle_in_world(&npc_select(GIVER, TALK_CHOICE), at, now + 1)
        .unwrap();
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "the giver must open its dialogue"
    );
    let released = game
        .handle_in_world(&dialogue_choice(), at, now + 2)
        .unwrap();
    let end = released
        .replies
        .iter()
        .find(|packet| family(packet, 49, 13))
        .cloned()
        .unwrap_or_else(|| panic!("quest {QUEST} did not release its 49/13 end screen"));
    assert_eq!(
        u32::from_le_bytes(end[14..18].try_into().unwrap()),
        QUEST,
        "49/13 carries the quest id"
    );
    assert_eq!(
        u32::from_le_bytes(end[18..22].try_into().unwrap()),
        QUEST_END_TEXT,
        "49/13 carries the quest's end text (client reader 0x00c7b990)"
    );
    end
}

fn step(game: &mut Gameplay, at: &CharacterState, packet: &[u8], now: u64) -> Outcome {
    game.handle_in_world(packet, at, now)
        .unwrap_or_else(|error| panic!("packet {:?} was refused: {error}", &packet[..4]))
}

#[test]
fn a_shared_field_kill_credits_the_killer_quest_once() {
    let fixture = Fixture::new(&five_wolves());
    let at = [1.0, 0.0, 0.0];
    let character = character_at(A, at);
    let mut game = player(&fixture, A, at, 0);
    let mut field = game.take_shared_fields().unwrap();
    let known: BTreeSet<u64> = (0..5).map(|index| WOLF_GUID + index).collect();

    let first = attack(&mut game, &mut field, &character, &known, WOLF_GUID, 0).unwrap();
    assert_eq!(counts(&game, QUEST), [1, 0], "one kill is one credit");
    assert!(
        objective_update(&first.replies, QUEST, DEFEAT_OBJECTIVE, 1),
        "49/9 ObjectiveUpdate reports 1/5 to the client"
    );
    assert_eq!(field.health(WOLF_GUID), Some(0), "the shared wolf is dead");
    assert_eq!(
        enemy_hp(&first.replies, WOLF_GUID),
        Some(0),
        "the killer sees the 35/5 zero-health update"
    );
    assert_eq!(
        removes(&first.replies, WOLF_GUID),
        0,
        "the shared field owns the removal broadcast; the killer's reply has no 35/3"
    );

    // The remaining four kills finish the objective and open the Talk step.
    let mut fifth = attack(
        &mut game,
        &mut field,
        &character,
        &known,
        WOLF_GUID + 1,
        700,
    )
    .unwrap();
    for index in 2..5_u64 {
        fifth = attack(
            &mut game,
            &mut field,
            &character,
            &known,
            WOLF_GUID + index,
            700 * index,
        )
        .unwrap();
    }
    assert_eq!(counts(&game, QUEST), [5, 0]);
    assert_eq!(status(&game, QUEST), QuestStatus::Active);
    assert!(
        objective_complete(&fifth.replies, QUEST, DEFEAT_OBJECTIVE),
        "the fifth kill produces the 49/10 Goal Complete packet"
    );
    assert!(
        fifth.replies.iter().any(|packet| family(packet, 49, 8)
            && u32::from_le_bytes(packet[6..10].try_into().unwrap()) == QUEST
            && u32::from_le_bytes(packet[10..14].try_into().unwrap()) == TALK_OBJECTIVE),
        "the completed kill goal activates the Talk objective"
    );
    // Field-kill coin/XP rewards (T240) still ride the same outcome.
    assert_eq!(count(&fifth.replies, 38, 19), 1);
    assert_eq!(count(&fifth.replies, 38, 14), 1);
}

#[test]
fn the_other_player_in_the_same_field_is_not_credited() {
    let fixture = Fixture::new(&[
        wolf(WOLF_GUID, WOLF, [0.0, 0.0, 0.0], 400),
        wolf(WOLF_TWO, WOLF, [-2.0, 0.0, 0.0], 100),
    ]);
    let a_at = [1.0, 0.0, 0.0];
    let b_at = [-1.0, 0.0, 0.0];
    let a_character = character_at(A, a_at);
    let b_character = character_at(B, b_at);
    let mut a = player(&fixture, A, a_at, 0);
    let mut b = player(&fixture, B, b_at, 0);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    let known = BTreeSet::from([WOLF_GUID, WOLF_TWO]);

    // B wounds the wolf (two hits of 100) and owns no lethal blow.
    for now in [0, 700] {
        let out = attack(&mut b, &mut field, &b_character, &known, WOLF_GUID, now).unwrap();
        assert!(
            !out.replies
                .iter()
                .any(|packet| family(packet, 49, 3) || family(packet, 49, 9)),
            "B's non-lethal hit must not add or advance the quest"
        );
    }
    let b_before = b.state();
    for now in [1_400, 2_100, 2_800] {
        attack(&mut a, &mut field, &a_character, &known, WOLF_GUID, now).unwrap();
    }
    assert_eq!(field.health(WOLF_GUID), Some(0));
    assert_eq!(counts(&a, QUEST), [1, 0], "only the killer is credited");
    assert_eq!(counts(&b, QUEST), [0, 0], "the bystander's counter is 0");
    assert_eq!(
        b.state(),
        b_before,
        "A's kill changed nothing in B's saved gameplay state"
    );

    // The shared tick that claims B's T240 reward must not deliver A's defeat
    // event to B either.
    let claimed = b
        .tick_shared_effects(&mut field, &b_character, 2_850)
        .unwrap();
    assert_eq!(
        counts(&b, QUEST),
        [0, 0],
        "the shared tick credits no bystander"
    );
    assert!(
        !claimed
            .replies
            .iter()
            .any(|packet| family(packet, 49, 3) || family(packet, 49, 9) || family(packet, 49, 10)),
        "A's defeat event is not fanned out to B"
    );

    // B's own next kill credits B normally in the same field.
    let credit = attack(&mut b, &mut field, &b_character, &known, WOLF_TWO, 3_500).unwrap();
    assert_eq!(counts(&b, QUEST), [1, 0]);
    assert_eq!(counts(&a, QUEST), [1, 0], "B's kill does not credit A");
    assert!(objective_update(
        &credit.replies,
        QUEST,
        DEFEAT_OBJECTIVE,
        1
    ));
}

#[test]
fn a_second_tick_or_close_does_not_double_credit() {
    let fixture = Fixture::new(&[wolf(WOLF_GUID, WOLF, [0.0, 0.0, 0.0], 100)]);
    let at = [1.0, 0.0, 0.0];
    let character = character_at(A, at);
    let mut game = player(&fixture, A, at, 0);
    let mut field = game.take_shared_fields().unwrap();
    let known = BTreeSet::from([WOLF_GUID]);

    attack(&mut game, &mut field, &character, &known, WOLF_GUID, 0).unwrap();
    assert_eq!(counts(&game, QUEST), [1, 0]);
    let saved = game.state();

    // Replaying the lethal request on the corpse and ticking the shared field
    // (T240's claim path) changes neither the counter nor the save.
    let replay = attack(&mut game, &mut field, &character, &known, WOLF_GUID, 700).unwrap();
    assert!(!replay.changed && replay.replies.is_empty());
    game.tick_shared_effects(&mut field, &character, 1_400)
        .unwrap();
    game.tick_shared_effects(&mut field, &character, 2_100)
        .unwrap();
    assert_eq!(counts(&game, QUEST), [1, 0]);
    assert_eq!(game.state(), saved, "the replay changed the saved state");
    assert_eq!(field.health(WOLF_GUID), Some(0));
}

#[test]
fn a_solo_arena_kill_still_credits_and_is_unchanged() {
    // No `take_shared_fields`: the 26/10 basic attack uses the engine's own
    // field copy, the non-shared `strike` path.
    let fixture = Fixture::new(&[wolf(WOLF_GUID, WOLF, [0.0, 0.0, 0.0], 100)]);
    let at = [1.0, 0.0, 0.0];
    let character = character_at(A, at);
    let mut game = player(&fixture, A, at, 0);
    game.handle_in_world(&npc_interact(WOLF_GUID), &character, 0)
        .unwrap();
    let killed = step(
        &mut game,
        &character,
        &npc_select(WOLF_GUID, ATTACK_CHOICE),
        700,
    );
    assert_eq!(counts(&game, QUEST), [1, 0]);
    assert!(
        objective_update(&killed.replies, QUEST, DEFEAT_OBJECTIVE, 1),
        "the solo path credits quest 3000009"
    );
    assert_eq!(enemy_hp(&killed.replies, WOLF_GUID), Some(0));
    assert_eq!(
        removes(&killed.replies, WOLF_GUID),
        1,
        "only the non-shared path removes the actor itself"
    );
}

#[test]
fn the_credit_survives_a_reconnect() {
    let fixture = Fixture::new(&five_wolves());
    let at = [1.0, 0.0, 0.0];
    let character = character_at(A, at);
    let mut game = player(&fixture, A, at, 0);
    let mut field = game.take_shared_fields().unwrap();
    let known: BTreeSet<u64> = (0..5).map(|index| WOLF_GUID + index).collect();
    for (index, guid) in (0..5).map(|index| WOLF_GUID + index).enumerate() {
        attack(
            &mut game,
            &mut field,
            &character,
            &known,
            guid,
            700 * index as u64,
        )
        .unwrap();
    }
    assert_eq!(counts(&game, QUEST), [5, 0]);
    let saved = game.state();
    let bytes = serde_json::to_vec(&saved).unwrap();
    let stored: osfr_tools::gameplay::GameplayState = serde_json::from_slice(&bytes).unwrap();
    let mut game = reconnect(&fixture, &stored);
    assert_eq!(counts(&game, QUEST), [5, 0]);
    assert_eq!(status(&game, QUEST), QuestStatus::Active);

    // Talk to the giver: the completed kill goal opens the pending turn-in, the
    // client releases the 49/13 end screen, and 49/14 pays once.
    let at_npc = character_at(A, giver_position());
    open_end_screen(&mut game, &at_npc, 10_000);
    let coins = game.state().inventory.coins;
    let xp = game.state().profiles.as_ref().unwrap().experience[&BRAWLER];
    let paid = step(&mut game, &at_npc, &quest_end_reply(QUEST), 10_010);
    assert!(paid.changed);
    assert!(has_quest_packet(&paid.replies, 4, QUEST));
    assert_eq!(game.state().inventory.coins, coins + QUEST_COINS);
    assert_eq!(
        game.state().profiles.as_ref().unwrap().experience[&BRAWLER],
        xp + QUEST_JOB_XP
    );
    assert_eq!(status(&game, QUEST), QuestStatus::Completed);

    let before = game.state();
    let again = step(&mut game, &at_npc, &quest_end_reply(QUEST), 10_020);
    assert!(
        !again.changed && !has_quest_packet(&again.replies, 4, QUEST),
        "a repeated turn-in pays nothing"
    );
    assert_eq!(game.state(), before, "a repeated turn-in changed the save");
}

#[test]
fn a_malformed_or_zero_definition_is_ignored() {
    // 0 is a definition id the content loader accepts but no quest can target.
    let fixture = Fixture::new(&[wolf(WOLF_GUID, 0, [0.0, 0.0, 0.0], 100)]);
    let at = [1.0, 0.0, 0.0];
    let character = character_at(A, at);
    let mut game = player(&fixture, A, at, 0);
    let mut field = game.take_shared_fields().unwrap();
    let known = BTreeSet::from([WOLF_GUID]);
    let out = attack(&mut game, &mut field, &character, &known, WOLF_GUID, 0)
        .expect("a zero definition is skipped, not refused");
    assert_eq!(field.health(WOLF_GUID), Some(0));
    assert_eq!(counts(&game, QUEST), [0, 0]);
    assert_eq!(count(&out.replies, 49, 9), 0);
    assert_eq!(count(&out.replies, 49, 10), 0);

    // A positive definition that is not the quest target is ignored too.
    let fixture = Fixture::new(&[wolf(WOLF_GUID, LEVEL_ONE, [0.0, 0.0, 0.0], 100)]);
    let mut game = player(&fixture, A, at, 0);
    let mut field = game.take_shared_fields().unwrap();
    attack(&mut game, &mut field, &character, &known, WOLF_GUID, 0).unwrap();
    assert_eq!(counts(&game, QUEST), [0, 0]);
}

#[test]
fn a_failing_quest_event_for_one_player_does_not_stop_the_other() {
    // A kill's quest event is server-generated and validated, so the refusal a
    // stale quest save actually gets is the catalogue bind every session runs.
    // A's save belongs to another character: A's bind returns Err, B's session
    // in the same field still kills and advances its own quest.
    let fixture = Fixture::new(&[
        wolf(WOLF_GUID, WOLF, [0.0, 0.0, 0.0], 100),
        wolf(WOLF_TWO, WOLF, [-2.0, 0.0, 0.0], 100),
    ]);
    let b_at = [-1.0, 0.0, 0.0];
    let b_character = character_at(B, b_at);
    let base = Gameplay::load(&fixture.dir, None).unwrap();
    let mut state = base.state();
    let mut stale = active_quests(&fixture.dir, A);
    stale.character_id = B;
    state.npc_quests = Some(stale);
    {
        let profiles = state.profiles.as_mut().unwrap();
        profiles.active = BRAWLER;
        profiles.unlocked.insert(BRAWLER);
    }
    let mut a = Gameplay::load(&fixture.dir, Some(state)).unwrap();
    let error = a
        .bind_npc_quest_catalogue(&fixture.dir, A)
        .err()
        .expect("A's stale quest save is refused");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    // The refused bind keeps the stale save for a later retry instead of consuming
    // it; no engine is bound, so A's kills still cannot credit it.
    let kept = a
        .state()
        .npc_quests
        .expect("the refused bind keeps the saved state");
    assert_eq!(kept.character_id, B);

    let mut b = player(&fixture, B, b_at, 0);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    let known = BTreeSet::from([WOLF_GUID, WOLF_TWO]);

    // A malformed position for A is refused inside the same staged shared-field
    // request; it changes no field state.
    let bad = character_at(A, [f32::NAN, 0.0, 0.0]);
    assert!(
        attack(&mut a, &mut field, &bad, &known, WOLF_GUID, 0).is_err(),
        "A's invalid position is refused"
    );
    assert_eq!(field.health(WOLF_GUID), Some(100));

    // A's own accepted kill leaves the unbound stale state untouched.
    let a_character = character_at(A, [1.0, 0.0, 0.0]);
    attack(&mut a, &mut field, &a_character, &known, WOLF_GUID, 650).expect("A's kill is accepted");
    assert_eq!(counts(&a, QUEST), [0, 0], "A's stale quest is not credited");

    let b_out = attack(&mut b, &mut field, &b_character, &known, WOLF_TWO, 700)
        .expect("B's kill is still accepted");
    assert_eq!(field.health(WOLF_TWO), Some(0));
    assert_eq!(counts(&b, QUEST), [1, 0], "B is credited");
    assert!(objective_update(&b_out.replies, QUEST, DEFEAT_OBJECTIVE, 1));
}

#[test]
fn the_existing_field_and_arena_suites_stay_green() {
    // Pin the replies the slice must not change: the shared kill keeps the
    // existing swing animation, 32/7 result, 35/5 zero-health update, no 35/3,
    // and T240's 38/19/38/14 field reward. HP, motion, respawn and power-up
    // suites are run separately and assert their own packets.
    let fixture = Fixture::new(&[wolf(WOLF_GUID, WOLF, [0.0, 0.0, 0.0], 100)]);
    let at = [1.0, 0.0, 0.0];
    let character = character_at(A, at);
    let mut game = player(&fixture, A, at, 0);
    let mut field = game.take_shared_fields().unwrap();
    let known = BTreeSet::from([WOLF_GUID]);
    let out = attack(&mut game, &mut field, &character, &known, WOLF_GUID, 0).unwrap();

    let swing = out
        .replies
        .iter()
        .find(|packet| family(packet, 35, 8) && packet[4..12] == A.to_le_bytes())
        .expect("the shared basic swing animation");
    assert_eq!(u32::from_le_bytes(swing[12..16].try_into().unwrap()), 1099);
    let hit = out
        .replies
        .iter()
        .find(|packet| family(packet, 32, 7) && packet[4..12] == A.to_le_bytes())
        .expect("32/7 attack result");
    assert_eq!(
        u64::from_le_bytes(hit[20..28].try_into().unwrap()),
        WOLF_GUID
    );
    assert_eq!(u32::from_le_bytes(hit[28..32].try_into().unwrap()), 100);
    assert_eq!(u32::from_le_bytes(hit[46..50].try_into().unwrap()), 0);
    assert_eq!(enemy_hp(&out.replies, WOLF_GUID), Some(0));
    assert_eq!(removes(&out.replies, WOLF_GUID), 0);
    assert_eq!(count(&out.replies, 38, 19), 1);
    assert_eq!(
        count(&out.replies, 38, 14),
        1,
        "the active job still receives the kill XP"
    );
}
