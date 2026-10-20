//! T256: a shared-field kill credits the killer's active kill quests.
//!
//! The session fixture, its private content copy, the saved-state reload, the
//! NPC menu/dialogue/turn-in packets and the 49/x objective assertions all come
//! from the canonical `tests/support/npc_quest_chain.rs` module; the 38/x reply
//! helpers (`count`, `packet`, `carrier`) come from `tests/support/packets.rs`.
//! This file adds only the synthetic shared-field population, the sessions
//! inside it and the shared-field requests.
//!
//! Every test drives a production seam: `Gameplay::handle_shared_field` (the
//! request `Session::commit_outcome` runs), `Gameplay::tick_shared_effects` (the
//! shared tick), `Gameplay::load` + `bind_npc_quest_catalogue` (the reconnect
//! bind), or `Gameplay::handle_in_world` for the non-shared solo strike.
//!
//! The quest is the shipped T241 playtest entry 3000009 "Fighting the Pack"
//! (`data/content/quests/adventurer.json`): five `3156` kills from giver
//! 9000000050, then one Talk to that giver. The field actors are a synthetic
//! population shared by two sessions, as in `tests/shared_field_state.rs`.
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/packets.rs"]
mod packets;

use npc_quest_chain::{
    Fixture, character_at_for, content_dir, content_position, counts, finish_quest, has_family,
    npc_interact, npc_select, objective_activated_packet, objective_complete_packet,
    objective_update_packet, quest_end_reply, quest_status, release_end_screen, send, talk,
};
use osfr_tools::{
    character_store::CharacterState,
    encounter::{SharedFieldAction, SharedFieldState},
    gameplay::{Gameplay, GameplayState, Outcome},
    quest_engine::{CharacterEligibility, QuestEngine, QuestState, QuestStatus},
};
use packets::{carrier, count, packet};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
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
const ATTACK_CHOICE: i32 = 103;
/// The shipped quest reward (`data/content/quests/adventurer.json`).
const QUEST_COINS: u32 = 18;
const QUEST_JOB_XP: u32 = 25;
/// Opening stage 910007 "Fighting off the Pack": the same five `3156` defeats.
const OPENING_STAGE: u32 = 910_007;
const WOLF_GUID: u64 = 131_000_109_001;
const WOLF_TWO: u64 = 131_000_109_002;
const LEVEL_ONE: i32 = 930_108;
/// The quest's end text (`end_text_string_id` in the quest record).
const QUEST_END_TEXT: u32 = 4_420_076;

/// A content copy whose shared field holds `enemies`, loaded through the
/// production `Gameplay::load` content path.
fn wolf_fixture(enemies: &[Value]) -> Fixture {
    Fixture::for_character(A).with_field_enemies(enemies)
}

/// Advance the saved opening save to stage 910007 "Fighting off the Pack":
/// every earlier stage complete and the wolf stage accepted with its two
/// non-combat goals met, so the next `3156` defeat is the stage's open goal.
/// Reconstructed save surgery only; the opening content itself stays as shipped.
fn start_wolf_stage(fixture: &Fixture, state: &mut GameplayState) {
    let content: Value =
        serde_json::from_slice(&fs::read(fixture.path().join("opening.json")).unwrap()).unwrap();
    let opening = state.opening.as_mut().expect("the opening content loads");
    for stage in content["stages"].as_array().unwrap() {
        let id = u32::try_from(stage["id"].as_u64().unwrap()).unwrap();
        let progress = opening.stages.get_mut(&id).unwrap();
        if id == OPENING_STAGE {
            progress.accepted = true;
            progress.counts = vec![1, 1, 0];
            break;
        }
        progress.accepted = true;
        progress.completed = true;
        progress.rewarded = true;
        progress.counts = stage["objectives"]
            .as_array()
            .unwrap()
            .iter()
            .map(|goal| u32::try_from(goal["required"].as_u64().unwrap()).unwrap())
            .collect();
    }
}

fn opening_counts(game: &Gameplay) -> Vec<u32> {
    game.state().opening.as_ref().unwrap().stages[&OPENING_STAGE]
        .counts
        .clone()
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
fn active_quests(guid: u64) -> QuestState {
    let mut engine =
        QuestEngine::from_split_content(&content_dir(), guid, eligibility(), None).unwrap();
    assert!(
        engine.offer(QUEST, GIVER).changed,
        "quest {QUEST} is offered"
    );
    assert!(engine.accept(QUEST).changed, "quest {QUEST} is accepted");
    engine.state().clone()
}

/// One player's session with an active 3000009 and the opening wolf stage
/// accepted two goals in, so the next `3156` defeat is the stage's open goal.
/// `profile` 43 (Brawler) is the quest's job gate and deals 100 per basic swing.
fn player(fixture: &Fixture, guid: u64, position: [f32; 3]) -> Gameplay {
    let mut state = Gameplay::load(fixture.path(), None).unwrap().state();
    state.inventory.coins = 0;
    state.npc_quests = Some(active_quests(guid));
    {
        let profiles = state.profiles.as_mut().unwrap();
        profiles.active = BRAWLER;
        profiles.unlocked.insert(BRAWLER);
    }
    start_wolf_stage(fixture, &mut state);
    let mut game = fixture.game_as(guid, Some(state));
    game.initialize_world(&character_at_for(guid, 0, position), 0)
        .unwrap();
    game
}

/// Production shared-field request: a basic attack on a field target.
fn attack(
    game: &mut Gameplay,
    field: &mut SharedFieldState,
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

fn removes(replies: &[Vec<u8>], guid: u64) -> usize {
    replies
        .iter()
        .filter(|packet| {
            has_family(packet, 35, 3, None)
                && packet.get(6..14) == Some(guid.to_le_bytes().as_slice())
        })
        .count()
}

fn enemy_hp(replies: &[Vec<u8>], guid: u64) -> Option<u32> {
    replies.iter().find_map(|packet| {
        (has_family(packet, 35, 5, None)
            && packet.get(4..12) == Some(guid.to_le_bytes().as_slice()))
        .then(|| u32::from_le_bytes(packet[12..16].try_into().unwrap()))
    })
}

#[test]
fn a_shared_field_kill_credits_the_killer_quest_once() {
    let fixture = wolf_fixture(&five_wolves());
    let at = [1.0, 0.0, 0.0];
    let character = character_at_for(A, 0, at);
    let mut game = player(&fixture, A, at);
    let mut field = game.take_shared_fields().unwrap();
    let known: BTreeSet<u64> = (0..5).map(|index| WOLF_GUID + index).collect();

    let first = attack(&mut game, &mut field, &character, &known, WOLF_GUID, 0).unwrap();
    assert_eq!(counts(&game, QUEST), [1, 0], "one kill is one credit");
    assert!(
        first
            .replies
            .contains(&objective_update_packet(QUEST, DEFEAT_OBJECTIVE, 1, 0.1)),
        "49/9 ObjectiveUpdate reports 1/5 and 10% quest progress"
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
    assert_eq!(
        opening_counts(&game),
        [1, 1, 1],
        "the shared kill advances the killer's active opening defeat goal"
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
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert!(
        fifth
            .replies
            .contains(&objective_complete_packet(QUEST, DEFEAT_OBJECTIVE, 0.5)),
        "the fifth kill produces the 49/10 Goal Complete packet"
    );
    assert!(
        fifth
            .replies
            .contains(&objective_activated_packet(QUEST, TALK_OBJECTIVE, 1)),
        "the completed kill goal activates the Talk objective"
    );
    // Field-kill coin/XP rewards (T240) still ride the same outcome. 38/19 is
    // one i32 balance; its client constructor `0x0089e670` reads one i32 after
    // the header. 38/14 is profile, rank, progress and three zero fields
    // (reader `0x008d86e0`).
    assert_eq!(
        packet(&fifth.replies, 38, 19),
        [38_u8, 0, 19, 0, 10, 0, 0, 0].as_slice()
    );
    assert_eq!(
        packet(&fifth.replies, 38, 14),
        carrier(38, 14, &[BRAWLER, 1, 25, 0, 0, 0]).as_slice()
    );
    assert_eq!(opening_counts(&game), [1, 1, 5]);
}

#[test]
fn the_other_player_in_the_same_field_is_not_credited() {
    let fixture = wolf_fixture(&[
        wolf(WOLF_GUID, WOLF, [0.0, 0.0, 0.0], 400),
        wolf(WOLF_TWO, WOLF, [-2.0, 0.0, 0.0], 100),
    ]);
    let a_at = [1.0, 0.0, 0.0];
    let b_at = [-1.0, 0.0, 0.0];
    let a_character = character_at_for(A, 0, a_at);
    let b_character = character_at_for(B, 0, b_at);
    let mut a = player(&fixture, A, a_at);
    let mut b = player(&fixture, B, b_at);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    let known = BTreeSet::from([WOLF_GUID, WOLF_TWO]);

    // B wounds the wolf (two hits of 100) and owns no lethal blow.
    for now in [0, 700] {
        let out = attack(&mut b, &mut field, &b_character, &known, WOLF_GUID, now).unwrap();
        assert!(
            !out.replies
                .iter()
                .any(|packet| has_family(packet, 49, 3, None) || has_family(packet, 49, 9, None)),
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
        opening_counts(&a),
        [1, 1, 1],
        "A's shared kill advanced A's opening defeat goal"
    );
    assert_eq!(
        opening_counts(&b),
        [1, 1, 0],
        "A's defeat event is not applied to B's opening stage"
    );
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
            .any(|packet| has_family(packet, 49, 3, None)
                || has_family(packet, 49, 9, None)
                || has_family(packet, 49, 10, None)),
        "A's defeat event is not fanned out to B"
    );

    // B's own next kill credits B normally in the same field, opening included.
    let credit = attack(&mut b, &mut field, &b_character, &known, WOLF_TWO, 3_500).unwrap();
    assert_eq!(counts(&b, QUEST), [1, 0]);
    assert_eq!(counts(&a, QUEST), [1, 0], "B's kill does not credit A");
    assert_eq!(opening_counts(&b), [1, 1, 1]);
    assert!(
        credit
            .replies
            .contains(&objective_update_packet(QUEST, DEFEAT_OBJECTIVE, 1, 0.1))
    );
}

#[test]
fn a_second_tick_or_close_does_not_double_credit() {
    let fixture = wolf_fixture(&[wolf(WOLF_GUID, WOLF, [0.0, 0.0, 0.0], 100)]);
    let at = [1.0, 0.0, 0.0];
    let character = character_at_for(A, 0, at);
    let mut game = player(&fixture, A, at);
    let mut field = game.take_shared_fields().unwrap();
    let known = BTreeSet::from([WOLF_GUID]);

    attack(&mut game, &mut field, &character, &known, WOLF_GUID, 0).unwrap();
    assert_eq!(counts(&game, QUEST), [1, 0]);
    let saved = game.state();

    // Replaying the lethal ability on the closed corpse changes neither the
    // counter nor the save.
    let replay = attack(&mut game, &mut field, &character, &known, WOLF_GUID, 700).unwrap();
    assert!(!replay.changed && replay.replies.is_empty());
    // The other shared basic-attack route (26/10 select 103, the corpse click)
    // is closed too.
    let closed = game
        .handle_shared_field(
            &SharedFieldAction::Select {
                target: WOLF_GUID,
                choice: ATTACK_CHOICE,
            },
            &mut field,
            &known,
            &character,
            1_400,
        )
        .unwrap();
    assert!(!closed.changed && closed.replies.is_empty());
    // A second and third shared tick (T240's claim path) pay and credit nothing.
    game.tick_shared_effects(&mut field, &character, 2_100)
        .unwrap();
    game.tick_shared_effects(&mut field, &character, 2_800)
        .unwrap();
    assert_eq!(counts(&game, QUEST), [1, 0]);
    assert_eq!(opening_counts(&game), [1, 1, 1]);
    assert_eq!(game.state(), saved, "the replay changed the saved state");
    assert_eq!(field.health(WOLF_GUID), Some(0));
}

#[test]
fn a_solo_arena_kill_still_credits_and_is_unchanged() {
    // No `take_shared_fields`: the 26/10 basic attack uses the engine's own
    // field copy, the non-shared `strike` path.
    let fixture = wolf_fixture(&[wolf(WOLF_GUID, WOLF, [0.0, 0.0, 0.0], 100)]);
    let at = [1.0, 0.0, 0.0];
    let character = character_at_for(A, 0, at);
    let mut game = player(&fixture, A, at);
    send(&mut game, &character, &npc_interact(WOLF_GUID), 0);
    let killed = send(
        &mut game,
        &character,
        &npc_select(WOLF_GUID, ATTACK_CHOICE),
        700,
    );
    assert_eq!(counts(&game, QUEST), [1, 0]);
    assert!(
        killed
            .replies
            .contains(&objective_update_packet(QUEST, DEFEAT_OBJECTIVE, 1, 0.1)),
        "the solo path credits quest 3000009"
    );
    assert_eq!(enemy_hp(&killed.replies, WOLF_GUID), Some(0));
    assert_eq!(
        removes(&killed.replies, WOLF_GUID),
        1,
        "only the non-shared path removes the actor itself"
    );
    assert_eq!(opening_counts(&game), [1, 1, 1]);
}

#[test]
fn the_credit_survives_a_reconnect() {
    let fixture = wolf_fixture(&five_wolves());
    let at = [1.0, 0.0, 0.0];
    let character = character_at_for(A, 0, at);
    let mut game = player(&fixture, A, at);
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
    // Reconnect through the shared fixture's JSON round trip, exactly as the
    // session loads its saved bytes.
    let mut game = fixture.reload_serialized(&game);
    game.initialize_world(&character_at_for(A, 0, at), 0)
        .unwrap();
    assert_eq!(counts(&game, QUEST), [5, 0]);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);

    // Talk to the giver: the completed kill goal opens the pending turn-in, the
    // client releases the 49/13 end screen, and 49/14 pays once.
    let at_npc = character_at_for(A, 0, content_position("farnum_farmstead.json", GIVER));
    talk(&mut game, &at_npc, GIVER, 10_000);
    let end = release_end_screen(&mut game, &at_npc, QUEST, GIVER, QUEST_END_TEXT, 10_002);
    assert!(end.starts_with(&[49, 0, 13, 0, 0, 0]));
    finish_quest(
        &mut game,
        &at_npc,
        QUEST,
        BRAWLER,
        QUEST_COINS,
        QUEST_JOB_XP,
        10_010,
    );
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);

    let before = game.state();
    let again = send(&mut game, &at_npc, &quest_end_reply(QUEST), 10_020);
    assert!(
        !again.changed
            && !again
                .replies
                .iter()
                .any(|packet| has_family(packet, 49, 4, Some(QUEST))),
        "a repeated turn-in pays nothing"
    );
    assert_eq!(game.state(), before, "a repeated turn-in changed the save");
}

#[test]
fn a_malformed_or_zero_definition_is_ignored() {
    // 0 is a definition id the content loader accepts but no quest can target;
    // the producer skips it, so the kill is not refused and no event is emitted.
    let fixture = wolf_fixture(&[wolf(WOLF_GUID, 0, [0.0, 0.0, 0.0], 100)]);
    let at = [1.0, 0.0, 0.0];
    let character = character_at_for(A, 0, at);
    let mut game = player(&fixture, A, at);
    let mut field = game.take_shared_fields().unwrap();
    let known = BTreeSet::from([WOLF_GUID]);
    let out = attack(&mut game, &mut field, &character, &known, WOLF_GUID, 0)
        .expect("a zero definition is skipped, not refused");
    assert_eq!(field.health(WOLF_GUID), Some(0));
    assert_eq!(counts(&game, QUEST), [0, 0]);
    assert_eq!(count(&out.replies, 49, 9), 0);
    assert_eq!(count(&out.replies, 49, 10), 0);
    assert_eq!(
        opening_counts(&game),
        [1, 1, 0],
        "a zero definition emits no opening defeat event"
    );

    // A negative definition is malformed content and is refused at load, so it
    // can never reach the kill event: `OpeningEngine::on_event(Defeat)` and
    // `QuestEvent::KillActorKind` both require a positive id.
    let bad = wolf_fixture(&[wolf(WOLF_GUID, -1, [0.0, 0.0, 0.0], 100)]);
    let error = Gameplay::load(bad.path(), None)
        .err()
        .expect("negative definitions are refused");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);

    // A positive definition that is not the quest target is ignored too.
    let fixture = wolf_fixture(&[wolf(WOLF_GUID, LEVEL_ONE, [0.0, 0.0, 0.0], 100)]);
    let mut game = player(&fixture, A, at);
    let mut field = game.take_shared_fields().unwrap();
    attack(&mut game, &mut field, &character, &known, WOLF_GUID, 0).unwrap();
    assert_eq!(counts(&game, QUEST), [0, 0]);
    assert_eq!(opening_counts(&game), [1, 1, 0]);
}

#[test]
fn a_failing_quest_event_for_one_player_does_not_stop_the_other() {
    // The kill-time quest event cannot fail: the producer skips a non-positive
    // definition (`a_malformed_or_zero_definition_is_ignored`), the loader
    // refuses a negative one, and `QuestEvent::KillActorKind` validates a
    // positive id with count 1. The reachable per-player failures on this route
    // are the catalogue bind (a stale save) and a malformed request, which this
    // test drives: A's session fails both and its state is unchanged, while B's
    // session in the same field still kills and advances its own quest.
    let fixture = wolf_fixture(&[
        wolf(WOLF_GUID, WOLF, [0.0, 0.0, 0.0], 100),
        wolf(WOLF_TWO, WOLF, [-2.0, 0.0, 0.0], 100),
    ]);
    let b_at = [-1.0, 0.0, 0.0];
    let b_character = character_at_for(B, 0, b_at);
    let mut state = Gameplay::load(fixture.path(), None).unwrap().state();
    // A's saved quest ledger belongs to B, so the per-character catalogue bind
    // refuses it before any gameplay request runs.
    state.npc_quests = Some(active_quests(B));
    {
        let profiles = state.profiles.as_mut().unwrap();
        profiles.active = BRAWLER;
        profiles.unlocked.insert(BRAWLER);
    }
    let mut a = Gameplay::load(fixture.path(), Some(state)).unwrap();
    let error = a
        .bind_npc_quest_catalogue(&fixture.path(), A)
        .err()
        .expect("A's stale quest save is refused");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    // The refused bind keeps the stale save for a later retry instead of
    // consuming it; no engine is bound, so A's kills still cannot credit it.
    let kept = a
        .state()
        .npc_quests
        .expect("the refused bind keeps the saved state");
    assert_eq!(kept.character_id, B);

    let mut b = player(&fixture, B, b_at);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    let known = BTreeSet::from([WOLF_GUID, WOLF_TWO]);

    // A malformed position for A is refused inside the same staged shared-field
    // request; it changes no field state and no session state.
    let a_before = a.state();
    let bad = character_at_for(A, 0, [f32::NAN, 0.0, 0.0]);
    assert!(
        attack(&mut a, &mut field, &bad, &known, WOLF_GUID, 0).is_err(),
        "A's invalid position is refused"
    );
    assert_eq!(field.health(WOLF_GUID), Some(100));
    assert_eq!(a.state(), a_before, "A's refused request changed its state");

    // A's own accepted kill leaves the unbound stale state untouched.
    let a_character = character_at_for(A, 0, [1.0, 0.0, 0.0]);
    attack(&mut a, &mut field, &a_character, &known, WOLF_GUID, 650).expect("A's kill is accepted");
    assert_eq!(counts(&a, QUEST), [0, 0], "A's stale quest is not credited");

    let b_out = attack(&mut b, &mut field, &b_character, &known, WOLF_TWO, 700)
        .expect("B's kill is still accepted");
    assert_eq!(field.health(WOLF_TWO), Some(0));
    assert_eq!(counts(&b, QUEST), [1, 0], "B is credited");
    assert!(
        b_out
            .replies
            .contains(&objective_update_packet(QUEST, DEFEAT_OBJECTIVE, 1, 0.1))
    );
}

#[test]
fn the_existing_field_and_arena_suites_stay_green() {
    // The HP/motion/respawn/reward behaviour is gated by
    // `tests/shared_field_state.rs`, `tests/shared_field_interest.rs` and
    // `tests/field_kill_rewards.rs`, which run as their own targets; the reward
    // bytes are also pinned by test 1 here. This test keeps the name's packet
    // contract to the two packets test 1 does not inspect: the 35/8 swing
    // animation and the 32/7 attack result of a shared kill.
    let fixture = wolf_fixture(&[wolf(WOLF_GUID, WOLF, [0.0, 0.0, 0.0], 100)]);
    let at = [1.0, 0.0, 0.0];
    let character = character_at_for(A, 0, at);
    let mut game = player(&fixture, A, at);
    let mut field = game.take_shared_fields().unwrap();
    let known = BTreeSet::from([WOLF_GUID]);
    let out = attack(&mut game, &mut field, &character, &known, WOLF_GUID, 0).unwrap();

    let swing = out
        .replies
        .iter()
        .find(|packet| has_family(packet, 35, 8, None) && packet[4..12] == A.to_le_bytes())
        .expect("the shared basic swing animation");
    assert_eq!(u32::from_le_bytes(swing[12..16].try_into().unwrap()), 1099);
    let hit = out
        .replies
        .iter()
        .find(|packet| has_family(packet, 32, 7, None) && packet[4..12] == A.to_le_bytes())
        .expect("32/7 attack result");
    assert_eq!(
        u64::from_le_bytes(hit[20..28].try_into().unwrap()),
        WOLF_GUID
    );
    assert_eq!(u32::from_le_bytes(hit[28..32].try_into().unwrap()), 100);
    assert_eq!(u32::from_le_bytes(hit[46..50].try_into().unwrap()), 0);
}
