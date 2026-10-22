//! T465: the three Introduce Yourself versions are mutually exclusive, so a
//! character can finish only one. Content: `excludes_quest_ids` on 3000020
//! (Highroad, Merigold), 3000021 (Cobblestone, Samantha) and 3000022
//! (Stillwater, Valerie) in `data/content/quests/adventurer.json`; the rule is
//! `QuestEngine::requirement_denial`, shared by offer and accept.
//!
//! Server policy only: the client's 49/3 quest record (`0x00c7cc80` -> body
//! `0x0093c840`: dwords, bools, the reward bundle `0x008e7930`, the goal list
//! `0x0092b050`, a dword and two bools) carries no exclusion field.
//!
//! Every step is a client request through `Gameplay::handle_in_world`: 26/8
//! interact (`0x008b6bc0`), 26/10 menu select (`0x00a36f80`), 26/6 dialogue row
//! (`0x009f5890`), 49/2 accept (`0x00a86400`), 49/14 end reply (`0x00a864c0`) and
//! 26/23 abandon (writer `0x00c0a350`). No new wire.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    CHARACTER, Fixture, accept, assert_no_offer, assert_packet_changes_nothing, character_at_for,
    content_dir, copy_tree, finish_quest, offer, pending_offer, quest_abandon, quest_reply,
    quest_status, release_end_screen, send, talk_step,
};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    npc_interaction::NpcInteraction,
    quest_engine::{CharacterEligibility, QuestEngine, QuestProgress, QuestStatus},
};
use serde_json::Value;
use std::{fs, io::ErrorKind, path::Path, sync::OnceLock};

const OTHER: u64 = CHARACTER + 1;
const HIGHROAD: u32 = 3_000_020;
const COBBLESTONE: u32 = 3_000_021;
const STILLWATER: u32 = 3_000_022;
const MERIGOLD: u64 = 9_000_000_402;
const SAMANTHA: u64 = 9_000_000_189;
const VALERIE: u64 = 9_000_000_227;
const RICKY_DANGER: u64 = 9_500_000_254;
const ADVENTURER: i32 = 1;
/// 3000021's authored reward row: 5 coins and 20 Adventurer XP; no end text.
const COBBLESTONE_COINS: u32 = 5;
const COBBLESTONE_XP: u32 = 20;

/// Where `guid` stands to click `npc` (the content row the 26/8 range check reads).
fn at(guid: u64, npc: u64) -> CharacterState {
    static NPCS: OnceLock<NpcInteraction> = OnceLock::new();
    let npcs = NPCS.get_or_init(|| {
        let root = content_dir();
        NpcInteraction::load(&root.join("npcs"), &root.join("stores.json")).unwrap()
    });
    let (zone, position) = npcs.location(npc).expect("a placed NPC");
    character_at_for(guid, zone, position)
}

/// Samantha offers 3000021, the player accepts, talks to Ricky Danger and turns in.
fn finish_cobblestone(game: &mut Gameplay, guid: u64, now: &mut u64) {
    let samantha = at(guid, SAMANTHA);
    let ricky = at(guid, RICKY_DANGER);
    *now += 10;
    offer(game, &samantha, SAMANTHA, COBBLESTONE, *now);
    *now += 10;
    accept(game, &samantha, COBBLESTONE, *now);
    *now += 10;
    talk_step(game, &ricky, RICKY_DANGER, *now, "Ricky Danger");
    *now += 10;
    release_end_screen(game, &ricky, COBBLESTONE, RICKY_DANGER, 0, *now);
    *now += 10;
    finish_quest(
        game,
        &ricky,
        COBBLESTONE,
        ADVENTURER,
        COBBLESTONE_COINS,
        COBBLESTONE_XP,
        *now,
    );
}

#[test]
fn finishing_one_version_hides_the_other_two() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 1_000;
    finish_cobblestone(&mut game, CHARACTER, &mut now);
    // `assert_no_offer` also proves each refused Talk leaves the save bytes equal.
    for (giver, label) in [(MERIGOLD, "Merigold"), (VALERIE, "Valerie")] {
        assert_no_offer(&mut game, &at(CHARACTER, giver), giver, &mut now, label);
    }
}

#[test]
fn an_active_version_blocks_the_others() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let merigold = at(CHARACTER, MERIGOLD);
    let samantha = at(CHARACTER, SAMANTHA);
    let mut now = 1_000;
    offer(&mut game, &merigold, MERIGOLD, HIGHROAD, now);
    accept(&mut game, &merigold, HIGHROAD, now + 10);
    now += 20;
    assert_no_offer(&mut game, &samantha, SAMANTHA, &mut now, "Samantha");
    let valerie = at(CHARACTER, VALERIE);
    assert_no_offer(&mut game, &valerie, VALERIE, &mut now, "Valerie");
    assert_packet_changes_nothing(
        &mut game,
        &samantha,
        &quest_reply(COBBLESTONE, true),
        now + 10,
        "a forged 3000021 accept",
    );

    // An older save offered 3000021 while 3000020 was active: its accept is refused too.
    let mut state = game.state();
    let quests = state.npc_quests.as_mut().unwrap();
    quests.quests.insert(
        COBBLESTONE,
        QuestProgress {
            status: QuestStatus::Offered,
            counts: vec![0],
        },
    );
    quests.pending_offer = Some(COBBLESTONE);
    let mut old = fixture.game(Some(state));
    assert_packet_changes_nothing(
        &mut old,
        &samantha,
        &quest_reply(COBBLESTONE, true),
        now + 20,
        "accepting 3000021 offered before 3000020 started",
    );
    assert_eq!(quest_status(&old, COBBLESTONE), QuestStatus::Offered);
    assert_eq!(pending_offer(&old), Some(COBBLESTONE));
}

#[test]
fn abandoning_a_version_reopens_the_others() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let merigold = at(CHARACTER, MERIGOLD);
    let samantha = at(CHARACTER, SAMANTHA);
    let mut now = 1_000;
    offer(&mut game, &merigold, MERIGOLD, HIGHROAD, now);
    accept(&mut game, &merigold, HIGHROAD, now + 10);
    now += 20;
    assert_no_offer(&mut game, &samantha, SAMANTHA, &mut now, "Samantha");
    assert!(send(&mut game, &merigold, &quest_abandon(HIGHROAD), now + 10).changed);
    assert_eq!(quest_status(&game, HIGHROAD), QuestStatus::Abandoned);
    // (R) The forks lift the exclusion when the quest leaves the journal.
    offer(&mut game, &samantha, SAMANTHA, COBBLESTONE, now + 20);
}

#[test]
fn an_old_save_with_two_versions_keeps_both() {
    let fixture = Fixture::new();
    let mut state = fixture.game(None).state();
    let quests = &mut state.npc_quests.as_mut().unwrap().quests;
    quests.insert(
        HIGHROAD,
        QuestProgress {
            status: QuestStatus::Completed,
            counts: vec![1],
        },
    );
    quests.insert(
        COBBLESTONE,
        QuestProgress {
            status: QuestStatus::Active,
            counts: vec![0],
        },
    );
    let mut game = fixture.reload_serialized(&fixture.game(Some(state)));
    assert_eq!(quest_status(&game, HIGHROAD), QuestStatus::Completed);
    assert_eq!(quest_status(&game, COBBLESTONE), QuestStatus::Active);

    let ricky = at(CHARACTER, RICKY_DANGER);
    let mut now = 1_000;
    talk_step(&mut game, &ricky, RICKY_DANGER, now, "Ricky Danger");
    release_end_screen(&mut game, &ricky, COBBLESTONE, RICKY_DANGER, 0, now + 10);
    finish_quest(
        &mut game,
        &ricky,
        COBBLESTONE,
        ADVENTURER,
        COBBLESTONE_COINS,
        COBBLESTONE_XP,
        now + 20,
    );
    now += 30;
    let valerie = at(CHARACTER, VALERIE);
    assert_no_offer(&mut game, &valerie, VALERIE, &mut now, "Valerie");
}

/// The shipped three-way group with `edit` applied, loaded through the split loader.
fn load_with(scratch: &Path, edit: impl FnOnce(&mut Vec<Value>)) -> std::io::Result<QuestEngine> {
    let path = scratch.join("quests/adventurer.json");
    let mut document: Value =
        serde_json::from_slice(&fs::read(content_dir().join("quests/adventurer.json")).unwrap())
            .unwrap();
    edit(document["records"].as_array_mut().unwrap());
    fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
    QuestEngine::from_split_content(scratch, 1, CharacterEligibility::default(), None)
}

fn set(records: &mut [Value], quest: u32, ids: &[u32]) {
    let record = records
        .iter_mut()
        .find(|record| record["id"].as_u64() == Some(u64::from(quest)))
        .unwrap();
    record["excludes_quest_ids"] = ids.into();
}

/// Replace the group with a star: `HIGHROAD` excludes `leaves` other quests,
/// each of which lists it back, so only the count can be wrong.
fn star(records: &mut [Value], leaves: usize) {
    let ids: Vec<u32> = records
        .iter()
        .map(|record| record["id"].as_u64().unwrap() as u32)
        .filter(|id| *id != HIGHROAD)
        .take(leaves)
        .collect();
    assert_eq!(ids.len(), leaves);
    for record in records.iter_mut() {
        record["excludes_quest_ids"] = Value::Array(Vec::new());
    }
    set(records, HIGHROAD, &ids);
    for id in ids {
        set(records, id, &[HIGHROAD]);
    }
}

#[test]
fn bad_exclusion_data_is_refused_at_load() {
    let fixture = Fixture::new();
    let scratch = fixture.path().join("split");
    copy_tree(&content_dir().join("quests"), &scratch.join("quests"));
    copy_tree(&content_dir().join("npcs"), &scratch.join("npcs"));
    fs::copy(content_dir().join("jobs.json"), scratch.join("jobs.json")).unwrap();

    // Controls: the shipped group and a 16-quest star both load.
    assert!(load_with(&scratch, |_| {}).is_ok());
    assert!(load_with(&scratch, |records| star(records, 16)).is_ok());

    let cases: [(&str, fn(&mut Vec<Value>)); 5] = [
        ("a missing id", |records| {
            set(records, HIGHROAD, &[COBBLESTONE, STILLWATER, 3_999_999])
        }),
        ("the quest itself", |records| {
            set(records, HIGHROAD, &[HIGHROAD, COBBLESTONE, STILLWATER])
        }),
        ("a repeated id", |records| {
            set(records, HIGHROAD, &[COBBLESTONE, COBBLESTONE, STILLWATER])
        }),
        ("a one-sided pair", |records| {
            set(records, STILLWATER, &[COBBLESTONE])
        }),
        ("17 ids", |records| star(records, 17)),
    ];
    for (label, edit) in cases {
        match load_with(&scratch, edit) {
            Ok(_) => panic!("{label} must fail content load"),
            Err(error) => assert_eq!(error.kind(), ErrorKind::InvalidData, "{label}"),
        }
    }
}

#[test]
fn players_are_independent() {
    let fixture = Fixture::new();
    let mut a = fixture.game(None);
    let mut now = 1_000;
    finish_cobblestone(&mut a, CHARACTER, &mut now);
    let mut b = fixture.game_as(OTHER, None);
    offer(&mut b, &at(OTHER, MERIGOLD), MERIGOLD, HIGHROAD, now + 10);
    now += 20;
    assert_no_offer(&mut a, &at(CHARACTER, MERIGOLD), MERIGOLD, &mut now, "A");
}
