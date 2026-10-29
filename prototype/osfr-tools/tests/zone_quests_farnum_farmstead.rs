//! T730: One More Gloam (`3000010`) and Where's the Help? (`3000011`), the two Farnum
//! Farmstead opening-arc quests the round-3 objective specs wire end to end.
//!
//! Pappy Farnum (9000000165) offers One More Gloam to a player who finished Fighting the
//! Pack; the quest spawns one Gloam Creature (definition 920001, a quest-gated row in
//! `field_enemies.json`) that dies to the ordinary field attack, and Pappy pays the coins
//! on the turn-in. Pappy then offers Where's the Help?, which completes on a Talk to Bartle
//! (9000000021, Cobblestone Village), whose own greeting is the end text.
//!
//! Every test drives the shared `ZoneQuests` walkers in `tests/support/npc_quest_chain.rs`:
//! 26/8 -> 26/10 -> 26/6 offers (49/1 card ids pinned), 49/2 accepts, the Talk that holds the
//! 49/13 and the 49/14 turn-in; only the field Attack menu loop is local. The payouts come
//! from the shipped records (`tests/support/quest_records.rs`), never pinned here. Client readers (IDA, 2014 client; no new wire):
//! 49/1 `0x00c7b7a0`, 49/13 `QuestEndPacket::ReadFields` `0x00c7b990`, 49/14 `0x00a864c0`,
//! 26/8 `0x008b6bc0`, 26/10 `0x00a36f80`, 35/3 removal `0x008d69f0`.
//!
//! The three other quests in the round-3 list stay disabled; see `notes/Autopilot/T730.md`.

#[path = "common/field_enemy_fixture.rs"]
mod field_enemy_fixture;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_records.rs"]
mod quest_records;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, ZoneQuest, ZoneQuests, assert_no_offer,
    assert_packet_changes_nothing, character_at_for, coins, counts, has_family, npc_interact,
    npc_select, objective_complete, pending_turnin, quest_end_reply, quest_reply, quest_status,
    saved, send, zone_player_with_quests,
};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, Outcome},
    quest_engine::QuestStatus,
};
use quest_records::shipped_quest;
use serde_json::Value;

const PAPPY: u64 = 9_000_000_165;
const BARTLE: u64 = 9_000_000_021;
/// The combat job Fighting the Pack asks for; the field Attack menu refuses a non-combat job.
const BRAWLER: i32 = 43;
const FIGHTING_THE_PACK: u32 = 3_000_009;
const ONE_MORE_GLOAM: u32 = 3_000_010;
const WHERES_THE_HELP: u32 = 3_000_011;
/// The Gloam Creature's quest-gated row in `field_enemies.json` (definition 920001).
const GLOAM_ROW: u64 = 0x4649_454c_0000_0012;
const ATTACK_CHOICE: i32 = 103;
/// Title, intro and description ids the 49/1 card carries (client locale).
const GLOAM_CARD: [u32; 3] = [4_420_082, 4_420_081, 4_420_083];
const HELP_CARD: [u32; 3] = [4_420_090, 4_420_089, 4_420_091];
const GLOAM_END_TEXT: u32 = 4_420_084;
const HELP_END_TEXT: u32 = 4_420_092;

/// Pappy's row is in `farnum_farmstead.json`, Bartle's in `cobblestone_village.json`.
const MAP: ZoneQuests = ZoneQuests {
    default_file: "farnum_farmstead.json",
    files: &[(BARTLE, "cobblestone_village.json")],
    runtime: &[],
};

/// The shipped record's payout and gates, with the card and end text pinned here.
fn quest(id: u32, card: [u32; 3], turnin: u64, end_text: u32) -> ZoneQuest {
    let record = shipped_quest(id);
    assert!(record.enabled, "quest {id} ships enabled");
    ZoneQuest {
        card,
        end_text,
        ..ZoneQuest::from_record(record, PAPPY, turnin, BRAWLER)
    }
}

fn one_more_gloam() -> ZoneQuest {
    quest(ONE_MORE_GLOAM, GLOAM_CARD, PAPPY, GLOAM_END_TEXT)
}

fn wheres_the_help() -> ZoneQuest {
    quest(WHERES_THE_HELP, HELP_CARD, BARTLE, HELP_END_TEXT)
}

/// The shipped Gloam Creature field row.
fn creature_row() -> Value {
    field_enemy_fixture::shipped_row(GLOAM_ROW)
        .expect("the shipped field has a Gloam Creature row")
        .clone()
}

/// The creature's actor id and a character standing on its spawn point.
fn creature() -> (u64, CharacterState) {
    let row = creature_row();
    let position = serde_json::from_value(row["position"].clone()).unwrap();
    (GLOAM_ROW, character_at_for(CHARACTER, 0, position))
}

/// The field holds only the shipped Gloam Creature row.
fn fixture() -> Fixture {
    Fixture::new().with_field_enemies(&[creature_row()])
}

/// A Brawler who already finished `done` (the authored prerequisites, each with its
/// objectives' full counts); Fighting the Pack has its own suite (`farnum_starting_quests`).
fn player(fixture: &Fixture, done: &[(u32, &[u32])]) -> Gameplay {
    zone_player_with_quests(fixture, CHARACTER, BRAWLER, &[ADVENTURER, BRAWLER], done)
}

const PACK_DONE: (u32, &[u32]) = (FIGHTING_THE_PACK, &[5, 1]);
const GLOAM_DONE: (u32, &[u32]) = (ONE_MORE_GLOAM, &[1]);

fn has_menu(outcome: &Outcome) -> bool {
    outcome
        .replies
        .iter()
        .any(|packet| has_family(packet, 26, 9, None))
}

/// One world tick at the creature's spawn point: the quest-gated field follows the active
/// quests there (`Gameplay::sync_field_quests`).
fn tick_at_creature(game: &mut Gameplay, now: u64) {
    game.tick(&creature().1, now).expect("the world ticks");
}

/// One field attack loop through the real 26/8 -> Attack menu -> 26/10 path. Returns the
/// outcome that carries the creature's removal (35/3, actor at `6..14`).
fn kill_creature(game: &mut Gameplay, now: &mut u64) -> Outcome {
    let (guid, at) = creature();
    *now += 100;
    tick_at_creature(game, *now);
    assert!(
        has_menu(&send(game, &at, &npc_interact(guid), *now)),
        "the Gloam Creature exposes the field interaction menu"
    );
    for _ in 0..16 {
        *now += 700;
        let attack = send(game, &at, &npc_select(guid, ATTACK_CHOICE), *now);
        if attack.replies.iter().any(|packet| {
            has_family(packet, 35, 3, None)
                && packet.get(6..14) == Some(guid.to_le_bytes().as_slice())
        }) {
            return attack;
        }
    }
    panic!("the Gloam Creature did not die");
}

#[test]
fn pappy_offers_one_more_gloam_and_the_kill_is_paid_once() {
    let fixture = fixture();
    let mut game = player(&fixture, &[PACK_DONE]);
    let (gloam, help) = (one_more_gloam(), wheres_the_help());
    let (guid, at) = creature();
    let pappy = MAP.at(CHARACTER, PAPPY);
    let mut now = 1_000;

    // The Gloam Creature is a quest actor: absent until the quest is active.
    tick_at_creature(&mut game, now);
    assert!(
        !has_menu(&send(&mut game, &at, &npc_interact(guid), now)),
        "no Gloam Creature is in the field before the quest"
    );

    MAP.accept(&mut game, CHARACTER, &gloam, &mut now);
    assert_eq!(counts(&game, ONE_MORE_GLOAM), [0]);

    let killed = kill_creature(&mut game, &mut now);
    assert!(
        objective_complete(&killed, ONE_MORE_GLOAM, 1),
        "the defeat sends the existing 49/10"
    );
    assert_eq!(counts(&game, ONE_MORE_GLOAM), [1]);
    assert_eq!(coins(&game), 0, "the kill itself pays nothing");

    // The Talk at Pappy holds the 49/13; the 49/14 pays the record's coins once.
    now += 100;
    MAP.turn_in(&mut game, CHARACTER, &gloam, BRAWLER, &mut now);
    assert_eq!(coins(&game), gloam.coins);

    // Done: a replayed 49/14 pays nothing, Pappy now offers the next quest, and the
    // creature leaves the field with its quest.
    now += 100;
    assert_packet_changes_nothing(
        &mut game,
        &pappy,
        &quest_end_reply(ONE_MORE_GLOAM),
        now,
        "a duplicate 49/14",
    );
    MAP.accept(&mut game, CHARACTER, &help, &mut now);
    tick_at_creature(&mut game, now + 60_000);
    assert!(
        !has_menu(&send(&mut game, &at, &npc_interact(guid), now + 60_000)),
        "the creature is gone after the quest"
    );
}

#[test]
fn wheres_the_help_completes_on_the_talk_to_bartle_and_pays_once() {
    let fixture = fixture();
    let mut game = player(&fixture, &[PACK_DONE, GLOAM_DONE]);
    let help = wheres_the_help();
    let (pappy, bartle) = (MAP.at(CHARACTER, PAPPY), MAP.at(CHARACTER, BARTLE));
    let mut now = 1_000;

    MAP.accept(&mut game, CHARACTER, &help, &mut now);
    assert_eq!(counts(&game, WHERES_THE_HELP), [0]);
    now += 100;

    // Pappy is not the target: a Talk back at him advances nothing and pays nothing.
    let before = saved(&game);
    let back = MAP.talk(&mut game, CHARACTER, PAPPY, now);
    assert!(!back.changed, "Pappy is not the Talk target");
    assert_eq!(saved(&game), before);
    now += 100;

    let talked = MAP.talk(&mut game, CHARACTER, BARTLE, now);
    assert!(talked.changed);
    assert!(objective_complete(&talked, WHERES_THE_HELP, 1));
    MAP.pay_held_turnin(&mut game, CHARACTER, &help, BRAWLER, &mut now);
    assert_eq!(coins(&game), help.coins);

    now += 100;
    assert_packet_changes_nothing(
        &mut game,
        &bartle,
        &quest_end_reply(WHERES_THE_HELP),
        now,
        "a duplicate 49/14",
    );
    assert_no_offer(&mut game, &pappy, PAPPY, &mut now, "Pappy after the chain");
    assert_eq!(quest_status(&game, WHERES_THE_HELP), QuestStatus::Completed);
}

#[test]
fn refusals_change_nothing() {
    let fixture = fixture();
    let pappy = MAP.at(CHARACTER, PAPPY);
    let bartle = MAP.at(CHARACTER, BARTLE);
    let mut now = 1_000;

    // Without Fighting the Pack there is no offer and a forged accept does nothing.
    let mut fresh = player(&fixture, &[]);
    assert_no_offer(&mut fresh, &pappy, PAPPY, &mut now, "Pappy before the Pack");
    for quest in [ONE_MORE_GLOAM, WHERES_THE_HELP] {
        now += 1;
        assert_packet_changes_nothing(
            &mut fresh,
            &pappy,
            &quest_reply(quest, true),
            now,
            "a forged accept before the Pack",
        );
    }

    // Past the Pack but not the Gloam, the second quest is still closed.
    let mut pack = player(&fixture, &[PACK_DONE]);
    assert_packet_changes_nothing(
        &mut pack,
        &pappy,
        &quest_reply(WHERES_THE_HELP, true),
        now + 2,
        "a forged accept of Where's the Help? before the Gloam",
    );

    // Mid-quest: early and unaccepted 49/14s and a Talk back at Pappy leave the save as it was.
    now += 10;
    MAP.accept(&mut pack, CHARACTER, &one_more_gloam(), &mut now);
    assert_packet_changes_nothing(
        &mut pack,
        &pappy,
        &quest_end_reply(ONE_MORE_GLOAM),
        now + 30,
        "an early 49/14",
    );
    assert_packet_changes_nothing(
        &mut pack,
        &bartle,
        &quest_end_reply(WHERES_THE_HELP),
        now + 31,
        "a 49/14 for a quest that was never accepted",
    );
    let mid = saved(&pack);
    assert!(!MAP.talk(&mut pack, CHARACTER, PAPPY, now + 40).changed);
    assert_eq!(saved(&pack), mid, "Pappy holds nothing before the kill");
    assert_eq!(counts(&pack, ONE_MORE_GLOAM), [0]);
}

#[test]
fn a_mid_quest_reconnect_keeps_the_kill() {
    let fixture = fixture();
    let mut game = player(&fixture, &[PACK_DONE]);
    let gloam = one_more_gloam();
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &gloam, &mut now);
    now += 100;
    kill_creature(&mut game, &mut now);

    let mut again = fixture.reload_serialized(&game);
    assert_eq!(counts(&again, ONE_MORE_GLOAM), [1]);
    now += 100;
    MAP.talk(&mut again, CHARACTER, PAPPY, now);
    assert_eq!(pending_turnin(&again), Some(ONE_MORE_GLOAM));
    MAP.pay_held_turnin(&mut again, CHARACTER, &gloam, BRAWLER, &mut now);
    assert_eq!(
        coins(&again),
        gloam.coins,
        "one payout across the reconnect"
    );
}
