//! T256/T302: a shared-field kill credits every credited player's own quests.
//!
//! T256 gave the engine that lands the lethal hit its own `defeated` event.
//! T302 carries the same server-authored definition on T240's kill credit, so
//! every other credited participant (`who`: hitters inside the participation
//! window, the engaged target and the killer) advances their own quests and
//! opening defeat goals on their next shared tick, exactly once. A participant
//! who leaves the field before the claim keeps T240's reward pair (F276's
//! reconnect) but forfeits the quest event.
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
    Fixture, character_at_for, coins, content_dir, content_position, counts, finish_quest,
    has_family, npc_interact, npc_select, objective_activated_packet, objective_complete_packet,
    objective_update_packet, profile_xp, quest_end_reply, quest_status, release_end_screen, saved,
    send, talk,
};
use osfr_tools::{
    character_store::CharacterState,
    encounter::{SharedFieldAction, SharedFieldState, field_rewards::CREDIT_TTL_MS},
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
const C: u64 = 303;
const D: u64 = 404;
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

/// The same fixture without `field_kill_rewards.json`: the production loader
/// reads a missing table as the empty table, so the T240 lookup returns no
/// reward and the actor has no `power_up_table` either. That leaves both the
/// drop table and the reward pair `None` while the definition stays positive,
/// which is the zero-reward kill T302 must still credit.
fn rewardless_fixture(enemies: &[Value]) -> Fixture {
    let fixture = wolf_fixture(enemies);
    fs::remove_file(fixture.path().join("field_kill_rewards.json"))
        .expect("the synthetic reward table was copied");
    fixture
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
    // 49/10 body reader `0x00c7b070` (common header `0x00c7acd0`): quest id,
    // objective id, progress float, one dword and a bool.
    assert!(
        fifth
            .replies
            .contains(&objective_complete_packet(QUEST, DEFEAT_OBJECTIVE, 0.5)),
        "the fifth kill produces the 49/10 Goal Complete packet"
    );
    // 49/8 body reader `0x00c7aeb0` through dispatcher `0x00c7b570` (apply
    // `0x00c7a910`): quest id, objective id, required count and a bool.
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
    // A player who is physically in the field but never hits or becomes the
    // enemy's target is not in T240's `who` set, so A's kill leaves no credit
    // for it. The contributor half of the same kill is covered by
    // `a_contributors_own_quest_advances_once_from_a_shared_field_kill`.
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

    // A kills the 400 HP wolf alone; B is present but contributes nothing.
    let b_before = b.state();
    for now in [0, 700, 1_400, 2_100] {
        attack(&mut a, &mut field, &a_character, &known, WOLF_GUID, now).unwrap();
    }
    assert_eq!(field.health(WOLF_GUID), Some(0));
    assert_eq!(counts(&a, QUEST), [1, 0], "the killer is credited");
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
    assert_eq!(field.pending_kill_credits(B), 0, "B earned no credit");
    assert_eq!(
        b.state(),
        b_before,
        "A's kill changed nothing in B's serialized gameplay state"
    );

    // The shared tick that pays B's T240 credit must not deliver A's defeat
    // event to B either, because B has no credit at all.
    let quiet = b
        .tick_shared_effects(&mut field, &b_character, 2_850)
        .unwrap();
    assert!(!quiet.changed);
    assert!(
        !quiet
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 3, None)
                || has_family(packet, 49, 9, None)
                || has_family(packet, 49, 10, None)),
        "A's defeat event is not fanned out to B"
    );
    assert_eq!(counts(&b, QUEST), [0, 0]);
    assert_eq!(opening_counts(&b), [1, 1, 0]);

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

/// A session with no bound NPC quest catalogue: the reachable form of a
/// claimant whose own kill event cannot be applied. The opening save stays at
/// its first stage, which has no defeat goal, so the claim changes nothing.
fn unbound_player(fixture: &Fixture, guid: u64, position: [f32; 3]) -> Gameplay {
    let mut state = Gameplay::load(fixture.path(), None).unwrap().state();
    state.inventory.coins = 0;
    {
        let profiles = state.profiles.as_mut().unwrap();
        profiles.active = BRAWLER;
        profiles.unlocked.insert(BRAWLER);
    }
    let mut game = Gameplay::load(fixture.path(), Some(state)).unwrap();
    game.initialize_world(&character_at_for(guid, 0, position), 0)
        .unwrap();
    game
}

#[test]
fn a_contributors_own_quest_advances_once_from_a_shared_field_kill() {
    let fixture = wolf_fixture(&[wolf(WOLF_GUID, WOLF, [0.0, 0.0, 0.0], 200)]);
    let a_at = [1.0, 0.0, 0.0];
    let b_at = [-1.0, 0.0, 0.0];
    let a_character = character_at_for(A, 0, a_at);
    let b_character = character_at_for(B, 0, b_at);
    let mut a = player(&fixture, A, a_at);
    let mut b = player(&fixture, B, b_at);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    let known = BTreeSet::from([WOLF_GUID]);

    // B contributes one 100-damage hit; A lands the lethal 100-damage swing.
    attack(&mut b, &mut field, &b_character, &known, WOLF_GUID, 0).unwrap();
    attack(&mut a, &mut field, &a_character, &known, WOLF_GUID, 100).unwrap();
    assert_eq!(field.health(WOLF_GUID), Some(0));
    assert_eq!(counts(&a, QUEST), [1, 0], "the killer is credited at once");
    assert_eq!(
        counts(&b, QUEST),
        [0, 0],
        "the contributor waits for its tick"
    );
    assert_eq!(field.pending_kill_credits(B), 1);
    assert_eq!(
        field.pending_kill_credits(A),
        0,
        "the killer is never in its own credit list"
    );
    // The real prune keeps a present contributor's credit, event included.
    field.retain_casters(&BTreeSet::from([A, B]), 120);
    assert_eq!(
        field.pending_kill_credits(B),
        1,
        "a present contributor keeps the credit"
    );

    let claimed = b
        .tick_shared_effects(&mut field, &b_character, 150)
        .unwrap();
    assert!(claimed.changed);
    assert_eq!(
        counts(&b, QUEST),
        [1, 0],
        "the credit advances B's own quest"
    );
    assert!(
        claimed
            .replies
            .contains(&objective_update_packet(QUEST, DEFEAT_OBJECTIVE, 1, 0.1)),
        "the contributor's own 49/9 reports 1/5 (client reader 0x00c7af60)"
    );
    let again = b
        .tick_shared_effects(&mut field, &b_character, 200)
        .unwrap();
    assert!(!again.changed && again.replies.is_empty());
    assert_eq!(
        counts(&b, QUEST),
        [1, 0],
        "the credit is claimed exactly once"
    );
    assert_eq!(field.pending_kill_credits(B), 0);

    // The claimed progress is part of the saved character: a serialized reload
    // through the production bind restores it, exactly as a reconnect does.
    let reconnected = fixture.reload_serialized_as(B, &b);
    assert_eq!(
        counts(&reconnected, QUEST),
        [1, 0],
        "the claimed counter survives the save round trip"
    );
    assert_eq!(quest_status(&reconnected, QUEST), QuestStatus::Active);
}

#[test]
fn the_credit_pays_the_contributor_reward_and_quest_event_once() {
    let fixture = wolf_fixture(&[wolf(WOLF_GUID, WOLF, [0.0, 0.0, 0.0], 200)]);
    let a_at = [1.0, 0.0, 0.0];
    let b_at = [-1.0, 0.0, 0.0];
    let a_character = character_at_for(A, 0, a_at);
    let b_character = character_at_for(B, 0, b_at);
    let mut a = player(&fixture, A, a_at);
    let mut b = player(&fixture, B, b_at);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    let known = BTreeSet::from([WOLF_GUID]);

    attack(&mut b, &mut field, &b_character, &known, WOLF_GUID, 0).unwrap();
    attack(&mut a, &mut field, &a_character, &known, WOLF_GUID, 100).unwrap();
    assert_eq!((coins(&b), profile_xp(&b, BRAWLER)), (0, 0));

    let claimed = b
        .tick_shared_effects(&mut field, &b_character, 150)
        .unwrap();
    assert_eq!(count(&claimed.replies, 38, 19), 1, "one coin balance reply");
    assert_eq!(
        packet(&claimed.replies, 38, 19),
        [38u8, 0, 19, 0, 2, 0, 0, 0].as_slice(),
        "the contributor is paid T240's 2 coins once (client ctor 0x0089e670)"
    );
    assert_eq!(
        packet(&claimed.replies, 38, 14),
        carrier(38, 14, &[BRAWLER, 1, 5, 0, 0, 0]).as_slice(),
        "the contributor is paid 5 job XP once (client reader 0x008d86e0)"
    );
    assert_eq!(coins(&b), 2);
    assert_eq!(profile_xp(&b, BRAWLER), 5);
    assert_eq!(
        counts(&b, QUEST),
        [1, 0],
        "the same credit advanced the quest once"
    );

    let again = b
        .tick_shared_effects(&mut field, &b_character, 200)
        .unwrap();
    assert!(
        !again.changed && again.replies.is_empty(),
        "no second EncounterReward and no second kill event"
    );
    assert_eq!((coins(&b), profile_xp(&b, BRAWLER)), (2, 5));
    assert_eq!(counts(&b, QUEST), [1, 0]);
}

#[test]
fn an_expired_or_left_field_credit_advances_nothing() {
    // Expiry: the contributor waits past CREDIT_TTL_MS.
    let fixture = wolf_fixture(&[wolf(WOLF_GUID, WOLF, [0.0, 0.0, 0.0], 200)]);
    let a_at = [1.0, 0.0, 0.0];
    let b_at = [-1.0, 0.0, 0.0];
    let a_character = character_at_for(A, 0, a_at);
    let b_character = character_at_for(B, 0, b_at);
    let mut a = player(&fixture, A, a_at);
    let mut b = player(&fixture, B, b_at);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    let known = BTreeSet::from([WOLF_GUID]);
    attack(&mut b, &mut field, &b_character, &known, WOLF_GUID, 0).unwrap();
    attack(&mut a, &mut field, &a_character, &known, WOLF_GUID, 100).unwrap();
    let before = saved(&b);
    let expired = b
        .tick_shared_effects(&mut field, &b_character, 100 + CREDIT_TTL_MS + 1)
        .unwrap();
    assert!(!expired.changed && expired.replies.is_empty());
    assert_eq!(
        saved(&b),
        before,
        "an expired credit changes neither the state nor its serialization"
    );
    assert_eq!(counts(&b, QUEST), [0, 0]);
    assert_eq!(opening_counts(&b), [1, 1, 0]);
    assert_eq!(
        field.pending_kill_credits(B),
        0,
        "the expired credit was consumed once"
    );

    // Leaving the field: a quest-only credit is dropped by the real prune.
    let fixture = rewardless_fixture(&[wolf(WOLF_GUID, WOLF, [0.0, 0.0, 0.0], 200)]);
    let mut a = player(&fixture, A, a_at);
    let mut b = player(&fixture, B, b_at);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    attack(&mut b, &mut field, &b_character, &known, WOLF_GUID, 0).unwrap();
    attack(&mut a, &mut field, &a_character, &known, WOLF_GUID, 100).unwrap();
    assert_eq!(field.pending_kill_credits(B), 1);
    field.retain_casters(&BTreeSet::from([A]), 150);
    assert_eq!(
        field.pending_kill_credits(B),
        0,
        "a quest-only credit does not survive leaving the field"
    );
    let before = saved(&b);
    let left = b
        .tick_shared_effects(&mut field, &b_character, 200)
        .unwrap();
    assert!(!left.changed && left.replies.is_empty());
    assert_eq!(saved(&b), before);
    assert_eq!(counts(&b, QUEST), [0, 0]);
    assert_eq!(opening_counts(&b), [1, 1, 0]);
    assert_eq!((coins(&b), profile_xp(&b, BRAWLER)), (0, 0));
}

#[test]
fn a_zero_reward_event_credits_a_contributor_quest() {
    // The reward table is missing (the loader's empty table) and the actor has
    // no `power_up_table`, so `credit_field_kill` sees no drop table and no
    // reward while the definition stays positive. The old early return would
    // leave B without a credit; T302 must still deliver the quest event.
    let fixture = rewardless_fixture(&[wolf(WOLF_GUID, WOLF, [0.0, 0.0, 0.0], 200)]);
    let a_at = [1.0, 0.0, 0.0];
    let b_at = [-1.0, 0.0, 0.0];
    let a_character = character_at_for(A, 0, a_at);
    let b_character = character_at_for(B, 0, b_at);
    let mut a = player(&fixture, A, a_at);
    let mut b = player(&fixture, B, b_at);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    let known = BTreeSet::from([WOLF_GUID]);

    attack(&mut b, &mut field, &b_character, &known, WOLF_GUID, 0).unwrap();
    let lethal = attack(&mut a, &mut field, &a_character, &known, WOLF_GUID, 100).unwrap();
    assert_eq!(field.health(WOLF_GUID), Some(0));
    assert_eq!(
        count(&lethal.replies, 38, 19),
        0,
        "a missing reward table pays no coins"
    );
    assert_eq!(
        field.pending_kill_credits(B),
        1,
        "the quest-bearing kill still credits B"
    );

    let claimed = b
        .tick_shared_effects(&mut field, &b_character, 150)
        .unwrap();
    assert_eq!(
        counts(&b, QUEST),
        [1, 0],
        "the zero-reward kill advances B's quest"
    );
    assert_eq!(opening_counts(&b), [1, 1, 1]);
    assert_eq!(count(&claimed.replies, 38, 19), 0);
    assert_eq!(count(&claimed.replies, 38, 14), 0);
    assert_eq!((coins(&b), profile_xp(&b, BRAWLER)), (0, 0));
}

#[test]
fn a_zero_or_invalid_definition_is_ignored_without_an_error() {
    // 0 is accepted by the content loader but names no quest target. With no
    // reward table either, the producer's early return still applies and no
    // credit is queued; the kill itself stays server-authoritative.
    let fixture = rewardless_fixture(&[wolf(WOLF_GUID, 0, [0.0, 0.0, 0.0], 200)]);
    let a_at = [1.0, 0.0, 0.0];
    let b_at = [-1.0, 0.0, 0.0];
    let a_character = character_at_for(A, 0, a_at);
    let b_character = character_at_for(B, 0, b_at);
    let mut a = player(&fixture, A, a_at);
    let mut b = player(&fixture, B, b_at);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    let known = BTreeSet::from([WOLF_GUID]);

    attack(&mut b, &mut field, &b_character, &known, WOLF_GUID, 0).unwrap();
    let killed = attack(&mut a, &mut field, &a_character, &known, WOLF_GUID, 100)
        .expect("a zero definition is skipped, not refused");
    assert_eq!(field.health(WOLF_GUID), Some(0));
    assert_eq!(field.pending_kill_credits(B), 0);
    assert_eq!(counts(&a, QUEST), [0, 0], "zero is no quest target");
    assert_eq!(count(&killed.replies, 49, 9), 0);
    let before = saved(&b);
    let quiet = b
        .tick_shared_effects(&mut field, &b_character, 150)
        .unwrap();
    assert!(!quiet.changed && quiet.replies.is_empty());
    assert_eq!(saved(&b), before);

    // A negative definition cannot become a u32 and is refused at content
    // load, before any kill can reach the credit path.
    let bad = rewardless_fixture(&[wolf(WOLF_GUID, -1, [0.0, 0.0, 0.0], 200)]);
    let error = Gameplay::load(bad.path(), None)
        .err()
        .expect("negative definitions are refused");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
}

#[test]
fn a_full_credit_list_drops_the_oldest_and_other_players_continue() {
    // 17 kills inside the 10 s TTL. B contributed to all of them, so its
    // 16-entry list drops the oldest; the first kill is the only one that
    // targets quest 3000009. C contributed only to that first kill and keeps
    // its one credit, so the same kill still advances C.
    let wolves: Vec<Value> = (0..17_u64)
        .map(|index| {
            let definition = if index == 0 { WOLF } else { LEVEL_ONE };
            // The first wolf takes B's and C's hits before A's lethal swing.
            let health = if index == 0 { 300 } else { 200 };
            wolf(
                WOLF_GUID + index,
                definition,
                [0.0, 0.0, index as f32 * 0.01],
                health,
            )
        })
        .collect();
    let fixture = wolf_fixture(&wolves);
    let a_at = [1.0, 0.0, 0.0];
    let b_at = [-1.0, 0.0, 0.0];
    let c_at = [0.0, 0.0, 1.0];
    let a_character = character_at_for(A, 0, a_at);
    let b_character = character_at_for(B, 0, b_at);
    let c_character = character_at_for(C, 0, c_at);
    let mut a = player(&fixture, A, a_at);
    let mut b = player(&fixture, B, b_at);
    let mut c = player(&fixture, C, c_at);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    c.take_shared_fields();
    let known: BTreeSet<u64> = (0..17_u64).map(|index| WOLF_GUID + index).collect();

    // The 600 ms basic-swing clock bounds the kill rate; 17 kills fit inside
    // the 10 s credit TTL.
    for index in 0..17_u64 {
        let hit_at = index * 600;
        let kill_at = hit_at + 100;
        attack(
            &mut b,
            &mut field,
            &b_character,
            &known,
            WOLF_GUID + index,
            hit_at,
        )
        .unwrap();
        if index == 0 {
            attack(&mut c, &mut field, &c_character, &known, WOLF_GUID, hit_at).unwrap();
        }
        let killed = attack(
            &mut a,
            &mut field,
            &a_character,
            &known,
            WOLF_GUID + index,
            kill_at,
        )
        .unwrap();
        assert!(killed.changed, "kill {index} is server-authoritative");
        assert_eq!(field.health(WOLF_GUID + index), Some(0), "kill {index}");
    }
    assert_eq!(
        field.pending_kill_credits(B),
        16,
        "the cap keeps 16 and drops the oldest"
    );
    assert_eq!(field.pending_kill_credits(C), 1);
    assert_eq!(counts(&b, QUEST), [0, 0]);
    assert_eq!(counts(&c, QUEST), [0, 0]);

    let claimed = b
        .tick_shared_effects(&mut field, &b_character, 9_750)
        .unwrap();
    assert_eq!(
        counts(&b, QUEST),
        [0, 0],
        "the dropped oldest credit was the only quest target"
    );
    assert_eq!(
        opening_counts(&b),
        [1, 1, 0],
        "no surviving credit matches the opening goal"
    );
    assert_eq!(count(&claimed.replies, 49, 9), 0);
    assert_eq!(field.pending_kill_credits(B), 0);

    // A different participant's identical oldest credit was not dropped.
    let c_credit = c
        .tick_shared_effects(&mut field, &c_character, 9_800)
        .unwrap();
    assert_eq!(counts(&c, QUEST), [1, 0], "C still gets the kill's event");
    assert_eq!(opening_counts(&c), [1, 1, 1]);
    assert!(
        c_credit
            .replies
            .contains(&objective_update_packet(QUEST, DEFEAT_OBJECTIVE, 1, 0.1))
    );
    assert_eq!(coins(&c), 2);
    assert_eq!(profile_xp(&c, BRAWLER), 5);
}

#[test]
fn a_group_member_who_never_hit_or_engaged_gets_no_credit() {
    // D is a field member (same interest set, same shared tick) but never hit
    // the wolf or became its target. T240's `who` is hitters inside the
    // participation window, the engaged target and the killer; mere
    // co-presence (or group-like membership) is not a contribution.
    let fixture = wolf_fixture(&[wolf(WOLF_GUID, WOLF, [0.0, 0.0, 0.0], 200)]);
    let a_at = [1.0, 0.0, 0.0];
    let b_at = [-1.0, 0.0, 0.0];
    let d_at = [0.0, 0.0, 1.0];
    let a_character = character_at_for(A, 0, a_at);
    let b_character = character_at_for(B, 0, b_at);
    let d_character = character_at_for(D, 0, d_at);
    let mut a = player(&fixture, A, a_at);
    let mut b = player(&fixture, B, b_at);
    let mut d = player(&fixture, D, d_at);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    d.take_shared_fields();
    let known = BTreeSet::from([WOLF_GUID]);

    attack(&mut b, &mut field, &b_character, &known, WOLF_GUID, 0).unwrap();
    attack(&mut a, &mut field, &a_character, &known, WOLF_GUID, 100).unwrap();
    assert_eq!(field.pending_kill_credits(B), 1);
    assert_eq!(field.pending_kill_credits(D), 0);

    let quiet = d
        .tick_shared_effects(&mut field, &d_character, 150)
        .unwrap();
    assert!(!quiet.changed && quiet.replies.is_empty());
    assert_eq!(counts(&d, QUEST), [0, 0]);
    assert_eq!(opening_counts(&d), [1, 1, 0]);
    assert_eq!((coins(&d), profile_xp(&d, BRAWLER)), (0, 0));
}

#[test]
fn a_player_outside_the_field_instance_gets_no_credit() {
    // The server computes each session's interest set (`known`); an outside
    // player's set never contains this instance's actor, so the request finds
    // no present enemy and records no contribution.
    let fixture = wolf_fixture(&[wolf(WOLF_GUID, WOLF, [0.0, 0.0, 0.0], 200)]);
    let a_at = [1.0, 0.0, 0.0];
    let b_at = [-1.0, 0.0, 0.0];
    let d_at = [0.0, 0.0, 1.0];
    let a_character = character_at_for(A, 0, a_at);
    let b_character = character_at_for(B, 0, b_at);
    let d_character = character_at_for(D, 0, d_at);
    let mut a = player(&fixture, A, a_at);
    let mut b = player(&fixture, B, b_at);
    let mut d = player(&fixture, D, d_at);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    d.take_shared_fields();
    let known = BTreeSet::from([WOLF_GUID]);
    let d_before = d.state();
    let d_serialized = saved(&d);

    let outside = attack(
        &mut d,
        &mut field,
        &d_character,
        &BTreeSet::new(),
        WOLF_GUID,
        0,
    )
    .unwrap();
    assert!(!outside.changed && outside.replies.is_empty());
    assert_eq!(field.health(WOLF_GUID), Some(200));
    assert_eq!(
        d.state(),
        d_before,
        "an outside-field request changed the player's full state"
    );
    assert_eq!(
        saved(&d),
        d_serialized,
        "an outside-field request changed the serialized state"
    );

    attack(&mut b, &mut field, &b_character, &known, WOLF_GUID, 0).unwrap();
    attack(&mut a, &mut field, &a_character, &known, WOLF_GUID, 100).unwrap();
    assert_eq!(field.pending_kill_credits(B), 1);
    assert_eq!(field.pending_kill_credits(D), 0);
    let quiet = d
        .tick_shared_effects(&mut field, &d_character, 150)
        .unwrap();
    assert!(!quiet.changed && quiet.replies.is_empty());
    assert_eq!(
        d.state(),
        d_before,
        "the shared tick changed the outside player's full state"
    );
    assert_eq!(saved(&d), d_serialized);
    assert_eq!(counts(&d, QUEST), [0, 0]);
    assert_eq!(opening_counts(&d), [1, 1, 0]);
}

#[test]
fn a_contributors_opening_defeat_goal_advances_once() {
    let fixture = wolf_fixture(&[
        wolf(WOLF_GUID, WOLF, [0.0, 0.0, 0.0], 200),
        wolf(WOLF_TWO, LEVEL_ONE, [0.0, 0.0, 1.0], 200),
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

    // The matching definition 3156 advances B's opening stage goal once.
    attack(&mut b, &mut field, &b_character, &known, WOLF_GUID, 0).unwrap();
    attack(&mut a, &mut field, &a_character, &known, WOLF_GUID, 100).unwrap();
    assert_eq!(opening_counts(&b), [1, 1, 0], "the claim is still pending");
    b.tick_shared_effects(&mut field, &b_character, 150)
        .unwrap();
    assert_eq!(opening_counts(&b), [1, 1, 1]);
    b.tick_shared_effects(&mut field, &b_character, 200)
        .unwrap();
    assert_eq!(opening_counts(&b), [1, 1, 1], "one event, not one per tick");

    // A different definition is carried (it is positive) but matches neither
    // the quest target nor the opening goal.
    attack(&mut b, &mut field, &b_character, &known, WOLF_TWO, 700).unwrap();
    attack(&mut a, &mut field, &a_character, &known, WOLF_TWO, 800).unwrap();
    assert_eq!(field.pending_kill_credits(B), 1);
    let other = b
        .tick_shared_effects(&mut field, &b_character, 850)
        .unwrap();
    assert_eq!(counts(&b, QUEST), [1, 0]);
    assert_eq!(opening_counts(&b), [1, 1, 1]);
    assert_eq!(count(&other.replies, 49, 9), 0);
    assert_eq!(field.pending_kill_credits(B), 0);
}

#[test]
fn the_killer_and_two_contributors_each_advance_once() {
    let fixture = wolf_fixture(&[wolf(WOLF_GUID, WOLF, [0.0, 0.0, 0.0], 300)]);
    let a_at = [1.0, 0.0, 0.0];
    let b_at = [-1.0, 0.0, 0.0];
    let c_at = [0.0, 0.0, 1.0];
    let a_character = character_at_for(A, 0, a_at);
    let b_character = character_at_for(B, 0, b_at);
    let c_character = character_at_for(C, 0, c_at);
    let mut a = player(&fixture, A, a_at);
    let mut b = player(&fixture, B, b_at);
    let mut c = player(&fixture, C, c_at);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    c.take_shared_fields();
    let known = BTreeSet::from([WOLF_GUID]);

    // B and C each wound the wolf; A lands the lethal hit.
    attack(&mut b, &mut field, &b_character, &known, WOLF_GUID, 0).unwrap();
    attack(&mut c, &mut field, &c_character, &known, WOLF_GUID, 0).unwrap();
    attack(&mut a, &mut field, &a_character, &known, WOLF_GUID, 0).unwrap();
    assert_eq!(field.health(WOLF_GUID), Some(0));
    assert_eq!(counts(&a, QUEST), [1, 0], "the killer advances at once");
    assert_eq!(counts(&b, QUEST), [0, 0]);
    assert_eq!(counts(&c, QUEST), [0, 0]);
    assert_eq!(field.pending_kill_credits(A), 0);
    assert_eq!(field.pending_kill_credits(B), 1);
    assert_eq!(field.pending_kill_credits(C), 1);

    b.tick_shared_effects(&mut field, &b_character, 50).unwrap();
    c.tick_shared_effects(&mut field, &c_character, 60).unwrap();
    assert_eq!(counts(&a, QUEST), [1, 0]);
    assert_eq!(counts(&b, QUEST), [1, 0]);
    assert_eq!(counts(&c, QUEST), [1, 0]);

    // A second shared tick, and the killer's own tick, advance nothing.
    for (game, character, now) in [
        (&mut a, &a_character, 70),
        (&mut b, &b_character, 80),
        (&mut c, &c_character, 90),
    ] {
        let quiet = game
            .tick_shared_effects(&mut field, character, now)
            .unwrap();
        assert!(!quiet.changed && quiet.replies.is_empty());
    }
    assert_eq!(counts(&a, QUEST), [1, 0]);
    assert_eq!(counts(&b, QUEST), [1, 0]);
    assert_eq!(counts(&c, QUEST), [1, 0]);
}

#[test]
fn a_failing_quest_event_for_one_contributor_does_not_change_or_stop_the_other() {
    // The event itself cannot fail (`KillActorKind` needs a positive id and
    // count 1), so this is the reachable no-op, not a failing-event test (the
    // protected-pass name is historical). B has no bound NPC quest catalogue and
    // an opening stage with no defeat goal: its claim consumes the credit and
    // changes nothing, while C's identical credit still pays.
    let fixture = rewardless_fixture(&[wolf(WOLF_GUID, WOLF, [0.0, 0.0, 0.0], 300)]);
    let a_at = [1.0, 0.0, 0.0];
    let b_at = [-1.0, 0.0, 0.0];
    let c_at = [0.0, 0.0, 1.0];
    let a_character = character_at_for(A, 0, a_at);
    let b_character = character_at_for(B, 0, b_at);
    let c_character = character_at_for(C, 0, c_at);
    let mut a = player(&fixture, A, a_at);
    let mut b = unbound_player(&fixture, B, b_at);
    let mut c = player(&fixture, C, c_at);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    c.take_shared_fields();
    let known = BTreeSet::from([WOLF_GUID]);

    attack(&mut b, &mut field, &b_character, &known, WOLF_GUID, 0).unwrap();
    attack(&mut c, &mut field, &c_character, &known, WOLF_GUID, 0).unwrap();
    attack(&mut a, &mut field, &a_character, &known, WOLF_GUID, 100).unwrap();
    assert_eq!(field.pending_kill_credits(B), 1);
    assert_eq!(field.pending_kill_credits(C), 1);

    let before_state = b.state();
    let before = saved(&b);
    let noop = b
        .tick_shared_effects(&mut field, &b_character, 150)
        .expect("an absent quest engine is not an error");
    assert!(!noop.changed && noop.replies.is_empty());
    assert_eq!(b.state(), before_state, "B's full state is unchanged");
    assert_eq!(saved(&b), before, "B's serialized state is unchanged");
    assert_eq!(
        field.pending_kill_credits(B),
        0,
        "the credit was consumed, not retried"
    );

    let paid = c
        .tick_shared_effects(&mut field, &c_character, 160)
        .unwrap();
    assert_eq!(counts(&c, QUEST), [1, 0], "C's credit still pays");
    assert!(
        paid.replies
            .contains(&objective_update_packet(QUEST, DEFEAT_OBJECTIVE, 1, 0.1))
    );
    assert_eq!(field.pending_kill_credits(C), 0);
}

#[test]
fn a_reward_bearing_credit_advances_a_reconnected_contributor_once() {
    // F276 keeps an unexpired reward-bearing credit in the world-owned ledger
    // across a disconnect, so its owner still collects the earned pair. T302's
    // defeat event is not a saved entitlement: the real `retain_casters` prune
    // forfeits it for a departed player (the brief: leaving the field drops it)
    // while the reward pair survives. The protected-pass name predates that
    // split and is kept; this test now pins the reward-only reconnect.
    let fixture = wolf_fixture(&[wolf(WOLF_GUID, WOLF, [0.0, 0.0, 0.0], 200)]);
    let a_at = [1.0, 0.0, 0.0];
    let b_at = [-1.0, 0.0, 0.0];
    let a_character = character_at_for(A, 0, a_at);
    let b_character = character_at_for(B, 0, b_at);
    let mut a = player(&fixture, A, a_at);
    let mut b = player(&fixture, B, b_at);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    let known = BTreeSet::from([WOLF_GUID]);

    attack(&mut b, &mut field, &b_character, &known, WOLF_GUID, 0).unwrap();
    attack(&mut a, &mut field, &a_character, &known, WOLF_GUID, 100).unwrap();
    // Reconnect through the serialized save (the production load + per-character
    // bind), then drop the old session; the world ledger keeps the credit.
    let mut reconnected = fixture.reload_serialized_as(B, &b);
    drop(b);
    field.retain_casters(&BTreeSet::from([A]), 150);
    assert_eq!(
        field.pending_kill_credits(B),
        1,
        "a reward-bearing credit survives the disconnect (F276)"
    );
    assert_eq!(
        counts(&reconnected, QUEST),
        [0, 0],
        "the serialized save carries no credit and no quest progress"
    );
    assert_eq!(opening_counts(&reconnected), [1, 1, 0]);

    reconnected
        .initialize_world(&character_at_for(B, 0, b_at), 0)
        .unwrap();
    reconnected.take_shared_fields();
    let claimed = reconnected
        .tick_shared_effects(&mut field, &b_character, 200)
        .unwrap();
    assert_eq!(
        packet(&claimed.replies, 38, 19),
        [38u8, 0, 19, 0, 2, 0, 0, 0].as_slice(),
        "the carried reward is paid once (client ctor 0x0089e670)"
    );
    assert_eq!(
        packet(&claimed.replies, 38, 14),
        carrier(38, 14, &[BRAWLER, 1, 5, 0, 0, 0]).as_slice(),
        "the carried job XP is paid once (client reader 0x008d86e0)"
    );
    assert_eq!(
        counts(&reconnected, QUEST),
        [0, 0],
        "leaving the field forfeits the quest event"
    );
    assert_eq!(
        opening_counts(&reconnected),
        [1, 1, 0],
        "leaving the field forfeits the opening defeat goal"
    );
    assert_eq!(
        count(&claimed.replies, 49, 9),
        0,
        "no objective update for a departed player"
    );
    assert_eq!(field.pending_kill_credits(B), 0);
}
