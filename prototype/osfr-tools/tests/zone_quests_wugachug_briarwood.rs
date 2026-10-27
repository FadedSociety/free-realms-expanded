//! T502: Aiding our Allies (`3000380`), Misplaced Mugwopp (`3000390`), How Bad is it?
//! (`3000391`), Briarwood's Not So Bad (`3000392`) and Join the Party (`3000400`),
//! community zone quests imported as data only into
//! `data/content/quests/zone_wugachug_briarwood.json`, plus the `legacy_data`
//! conversion that draws Headmaster Merk (his row was a POI anchor and was not
//! drawn; no second Merk row is added).
//!
//! Every test drives the production paths in `tests/support/npc_quest_chain.rs`:
//! 26/8 -> 26/10 -> 26/6 offers (49/1, decoded with the production reader), 49/2
//! accepts, the 26/6 that releases the held 49/13, and the 49/14 turn-in.
//!
//! Client readers (IDA, 2014 client; no new wire): 35/2 AddNpc `sub_921A10`
//! reads the actor as two dwords into `this+16/+20`, then a dword name string
//! id (`this+24`) and a dword model id (`this+28`), which is the order
//! `startup_world::parse_add_npc` decodes and the field order the Merk assertion
//! pins. 49/13 `QuestEndPacket::ReadFields` `0x00c7b990` reads the actor u64, the
//! quest dword, the end-text dword and the animation dword after the common
//! header (`sub_C7ACD0`), the layout `release_end_screen` pins. 26/3 payload
//! reader `0x00a9ef10` stores the first dword after the header as the dialogue
//! body. The Merk row needs no new writer.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, ZoneQuest, ZoneQuests, active_job, assert_no_offer,
    assert_packet_changes_nothing, coins, content_dir, counts, profile_xp, quest_end_reply,
    quest_reply, quest_status, read_json, saved, zone_player,
};
use osfr_tools::{quest_engine::QuestStatus, startup_world::parse_add_npc, world::NpcVisibility};

/// Arch Druid Camellia, the giver of Aiding our Allies.
const CAMELLIA: u64 = 9_500_000_650;
/// Headmaster Merk: a `poi_anchor` row before this slice, now a `legacy_data` spawn.
const MERK: u64 = 9_500_000_690;
const MERK_MODEL: i32 = 3_952;
const MERK_NAME: i32 = 45_386;
const BLORPLE: u64 = 9_500_000_288;
const MUGWOPP: u64 = 9_500_000_410;
const SMOGGO: u64 = 9_500_000_213;
const CARGUG: u64 = 9_500_000_209;
const BORANUG: u64 = 9_500_000_967;
const BREWMASTER: u64 = 9_000_000_191;
const OTHER: u64 = CHARACTER + 1;
const MINER: i32 = 14;

/// The rows live in five region files; the giver and the turn-in of Aiding our
/// Allies are both in `shrouded_glade.json`, so it is the default file.
const MAP: ZoneQuests = ZoneQuests {
    default_file: "shrouded_glade.json",
    files: &[
        (BLORPLE, "greenwood_forest.json"),
        (MUGWOPP, "merry_vale.json"),
        (SMOGGO, "briarwood.json"),
        (CARGUG, "briarwood.json"),
        (BORANUG, "wugachug.json"),
        (BREWMASTER, "wugachug.json"),
    ],
    runtime: &[],
};

const AIDING: ZoneQuest = ZoneQuest {
    id: 3_000_380,
    giver: CAMELLIA,
    card: [440_466, 440_465, 440_467],
    steps: &[],
    turnin: MERK,
    end_text: 440_468,
    coins: 50,
    xp: 15,
    profile_xp: 15,
};
const MISPLACED: ZoneQuest = ZoneQuest {
    id: 3_000_390,
    giver: BLORPLE,
    card: [46_211, 46_210, 46_212],
    steps: &[],
    turnin: MUGWOPP,
    end_text: 46_252,
    coins: 15,
    xp: 15,
    profile_xp: 15,
};
const HOW_BAD: ZoneQuest = ZoneQuest {
    id: 3_000_391,
    giver: MUGWOPP,
    card: [390_259, 390_258, 390_260],
    steps: &[],
    turnin: SMOGGO,
    end_text: 390_261,
    coins: 2,
    xp: 15,
    profile_xp: 15,
};
const NOT_SO_BAD: ZoneQuest = ZoneQuest {
    id: 3_000_392,
    giver: SMOGGO,
    card: [390_589, 390_588, 390_590],
    steps: &[],
    turnin: CARGUG,
    end_text: 390_591,
    coins: 2,
    xp: 30,
    profile_xp: 30,
};
const JOIN_PARTY: ZoneQuest = ZoneQuest {
    id: 3_000_400,
    giver: BORANUG,
    card: [385_979, 385_978, 385_980],
    steps: &[],
    turnin: BREWMASTER,
    end_text: 385_981,
    coins: 4,
    xp: 15,
    profile_xp: 15,
};

/// Every NPC row in `data/content/npcs`, as the production content loader reads
/// them (T502 only changes one: Merk's `name_string_id` becomes 45386). The JSON
/// read is the shared `npc_quest_chain::read_json`, so a read-rule change lands
/// here too.
fn npc_rows() -> Vec<(String, serde_json::Value)> {
    let dir = content_dir().join("npcs");
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
        .expect("NPC content directory")
        .map(|entry| entry.expect("NPC content entry").path())
        .collect();
    files.sort();
    files
        .into_iter()
        .flat_map(|path| {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .expect("NPC file name")
                .to_owned();
            let document = read_json(&path);
            document["records"]
                .as_array()
                .expect("NPC records")
                .iter()
                .map(|row| (name.clone(), row.clone()))
                .collect::<Vec<_>>()
        })
        .collect()
}

#[test]
fn aiding_our_allies_ends_at_merk() {
    // Exactly one Merk row, and it is the row this slice converted.
    let rows = npc_rows();
    let merks: Vec<(String, u64)> = rows
        .iter()
        .filter(|(_, row)| row["name_string_id"].as_u64() == Some(MERK_NAME as u64))
        .map(|(file, row)| (file.clone(), row["npc_id"].as_u64().expect("npc id")))
        .collect();
    assert_eq!(
        merks,
        [("shrouded_glade.json".to_owned(), MERK)],
        "one drawn Headmaster Merk row with name_string_id 45386"
    );
    assert_eq!(
        rows.iter()
            .filter(|(_, row)| row["name"].as_str() == Some("Headmaster Merk"))
            .count(),
        1,
        "the slice converts the existing Merk row, it adds no second one"
    );

    // The production visibility loader draws him as his own content key, and the
    // 35/2 AddNpc body (reader 0x00921a10) carries the model and name ids.
    let mut visibility = NpcVisibility::load(&content_dir()).expect("NPC visibility loads");
    assert_eq!(
        visibility.content_key(MERK),
        Some(MERK),
        "the converted legacy_data row spawns as its own content key"
    );
    let merkov = MAP.at(CHARACTER, MERK);
    let packets = visibility
        .update(0, merkov.position)
        .expect("NPC visibility updates");
    let add_npc = packets
        .iter()
        .find(|packet| {
            packet.starts_with(&[35, 0, 2, 0])
                && packet.get(4..12) == Some(MERK.to_le_bytes().as_slice())
        })
        .expect("Merk is sent as a 35/2 AddNpc at his own position");
    let spawn = parse_add_npc(add_npc).expect("the 35/2 AddNpc body parses");
    assert_eq!(spawn.actor_id, MERK);
    assert_eq!(spawn.model_id, MERK_MODEL, "AddNpc model id 3952");
    assert_eq!(spawn.name_string_id, MERK_NAME, "AddNpc name id 45386");

    let fixture = Fixture::minimal(CHARACTER);
    let mut game = zone_player(&fixture, CHARACTER, ADVENTURER);
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, ADVENTURER));
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &AIDING, &mut now);
    // The one goal is the turn-in Talk, so a 49/14 before it is refused and
    // changes nothing.
    assert_packet_changes_nothing(
        &mut game,
        &merkov,
        &quest_end_reply(AIDING.id),
        now,
        "an early 49/14 while Aiding our Allies' step is open",
    );
    MAP.turn_in(&mut game, CHARACTER, &AIDING, ADVENTURER, &mut now);
    assert_eq!(quest_status(&game, AIDING.id), QuestStatus::Completed);
    assert_eq!(counts(&game, AIDING.id), [1]);
    assert_eq!(coins(&game), coins_before + AIDING.coins);
    assert_eq!(
        profile_xp(&game, ADVENTURER),
        xp_before + AIDING.profile_xp,
        "the quest pays 50 coins and 15 Adventurer XP"
    );
    assert_packet_changes_nothing(
        &mut game,
        &merkov,
        &quest_end_reply(AIDING.id),
        now,
        "a duplicate 49/14",
    );
}

#[test]
fn chugawug_friends_chain_across_three_regions() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = zone_player(&fixture, CHARACTER, ADVENTURER);
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, ADVENTURER));
    let mut now = 1_000;

    // Blorple (Greenwood Forest) -> Mugwopp (Merry Vale).
    MAP.run(&mut game, CHARACTER, &MISPLACED, ADVENTURER, &mut now);
    assert_eq!(quest_status(&game, MISPLACED.id), QuestStatus::Completed);
    assert_eq!(coins(&game), coins_before + 15);
    assert_eq!(profile_xp(&game, ADVENTURER), xp_before + 15);

    // Mugwopp -> Smoggo (Briarwood).
    MAP.run(&mut game, CHARACTER, &HOW_BAD, ADVENTURER, &mut now);
    assert_eq!(quest_status(&game, HOW_BAD.id), QuestStatus::Completed);
    assert_eq!(coins(&game), coins_before + 17);
    assert_eq!(profile_xp(&game, ADVENTURER), xp_before + 30);

    // Smoggo -> Sergeant Cargug (Briarwood); the last quest of the chain pays 30.
    MAP.run(&mut game, CHARACTER, &NOT_SO_BAD, ADVENTURER, &mut now);
    assert_eq!(quest_status(&game, NOT_SO_BAD.id), QuestStatus::Completed);
    assert_eq!(
        coins(&game),
        coins_before + 19,
        "the chain pays 15 + 2 + 2 coins"
    );
    assert_eq!(
        profile_xp(&game, ADVENTURER),
        xp_before + 60,
        "the chain pays 15 + 15 + 30 Adventurer XP"
    );
    for quest in [&MISPLACED, &HOW_BAD, &NOT_SO_BAD] {
        assert_eq!(quest_status(&game, quest.id), QuestStatus::Completed);
    }

    // Cargug is a turn-in, not a giver: after the chain his menu offers nothing.
    assert_no_offer(
        &mut game,
        &MAP.at(CHARACTER, CARGUG),
        CARGUG,
        &mut now,
        "Cargug after the chain",
    );
    assert_packet_changes_nothing(
        &mut game,
        &MAP.at(CHARACTER, CARGUG),
        &quest_end_reply(NOT_SO_BAD.id),
        now,
        "a duplicate 49/14 for Briarwood's Not So Bad",
    );
}

#[test]
fn join_the_party() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = zone_player(&fixture, CHARACTER, ADVENTURER);
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, ADVENTURER));
    let mut now = 1_000;
    MAP.run(&mut game, CHARACTER, &JOIN_PARTY, ADVENTURER, &mut now);
    assert_eq!(quest_status(&game, JOIN_PARTY.id), QuestStatus::Completed);
    assert_eq!(coins(&game), coins_before + 4);
    assert_eq!(profile_xp(&game, ADVENTURER), xp_before + 15);
    assert_no_offer(
        &mut game,
        &MAP.at(CHARACTER, BREWMASTER),
        BREWMASTER,
        &mut now,
        "the Brewmaster after Join the Party",
    );
}

#[test]
fn chain_order_is_enforced() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = zone_player(&fixture, CHARACTER, ADVENTURER);
    let mut now = 1_000;
    let mugwopp = MAP.at(CHARACTER, MUGWOPP);
    let smoggo = MAP.at(CHARACTER, SMOGGO);
    let cargug = MAP.at(CHARACTER, CARGUG);

    // Misplaced Mugwopp is not done, so Mugwopp offers nothing and a forged 49/2
    // changes nothing.
    assert_no_offer(
        &mut game,
        &mugwopp,
        MUGWOPP,
        &mut now,
        "Mugwopp before Misplaced Mugwopp",
    );
    assert_packet_changes_nothing(
        &mut game,
        &mugwopp,
        &quest_reply(HOW_BAD.id, true),
        now,
        "a forged 49/2 for How Bad is it?",
    );

    // Smoggo and Cargug are two steps further down the chain.
    assert_no_offer(
        &mut game,
        &smoggo,
        SMOGGO,
        &mut now,
        "Smoggo before How Bad is it?",
    );
    assert_packet_changes_nothing(
        &mut game,
        &smoggo,
        &quest_reply(NOT_SO_BAD.id, true),
        now,
        "a forged 49/2 for Briarwood's Not So Bad",
    );
    assert_no_offer(
        &mut game,
        &cargug,
        CARGUG,
        &mut now,
        "Cargug, the wrong NPC before the chain",
    );
    assert_packet_changes_nothing(
        &mut game,
        &cargug,
        &quest_reply(HOW_BAD.id, true),
        now,
        "a forged 49/2 for How Bad is it? at its turn-in",
    );

    // Completing the first quest opens the second; the third still waits.
    MAP.run(&mut game, CHARACTER, &MISPLACED, ADVENTURER, &mut now);
    MAP.accept(&mut game, CHARACTER, &HOW_BAD, &mut now);
    assert_packet_changes_nothing(
        &mut game,
        &smoggo,
        &quest_end_reply(HOW_BAD.id),
        now,
        "an early 49/14 for How Bad is it?",
    );
    assert_packet_changes_nothing(
        &mut game,
        &smoggo,
        &quest_reply(NOT_SO_BAD.id, true),
        now,
        "a forged 49/2 for Briarwood's Not So Bad with its prerequisite active",
    );
}

#[test]
fn progress_survives_reconnect() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = zone_player(&fixture, CHARACTER, ADVENTURER);
    let mut now = 1_000;

    // Mid-activity: accepted, not yet turned in.
    MAP.accept(&mut game, CHARACTER, &MISPLACED, &mut now);
    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, MISPLACED.id), QuestStatus::Active);
    assert_eq!(counts(&game, MISPLACED.id), [0]);
    MAP.turn_in(&mut game, CHARACTER, &MISPLACED, ADVENTURER, &mut now);

    // A finished quest stays finished and still satisfies the next prerequisite.
    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, MISPLACED.id), QuestStatus::Completed);
    MAP.accept(&mut game, CHARACTER, &HOW_BAD, &mut now);
    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, HOW_BAD.id), QuestStatus::Active);
    MAP.turn_in(&mut game, CHARACTER, &HOW_BAD, ADVENTURER, &mut now);
    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, HOW_BAD.id), QuestStatus::Completed);
    MAP.run(&mut game, CHARACTER, &NOT_SO_BAD, ADVENTURER, &mut now);
    assert_eq!(quest_status(&game, NOT_SO_BAD.id), QuestStatus::Completed);
}

#[test]
fn two_players_are_independent() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut a = zone_player(&fixture, CHARACTER, ADVENTURER);
    let mut b = zone_player(&fixture, OTHER, ADVENTURER);
    let mut now = 1_000;
    MAP.accept(&mut a, CHARACTER, &MISPLACED, &mut now);
    MAP.accept(&mut b, OTHER, &MISPLACED, &mut now);
    let b_before = saved(&b);

    // A's Talk at Mugwopp must credit A only.
    MAP.turn_in(&mut a, CHARACTER, &MISPLACED, ADVENTURER, &mut now);
    assert_eq!(saved(&b), b_before, "A's run changed B");
    assert_eq!(counts(&b, MISPLACED.id), [0]);
    assert_eq!(
        quest_status(&b, MISPLACED.id),
        QuestStatus::Active,
        "A's Talk credited B's goal"
    );

    // B's own run still pays B, once.
    let b_coins = coins(&b);
    MAP.turn_in(&mut b, OTHER, &MISPLACED, ADVENTURER, &mut now);
    assert_eq!(coins(&b), b_coins + MISPLACED.coins);
    assert_eq!(quest_status(&a, MISPLACED.id), QuestStatus::Completed);
}

#[test]
fn xp_goes_to_adventurer_with_another_job_active() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = zone_player(&fixture, CHARACTER, MINER);
    let miner_before = profile_xp(&game, MINER);
    let adventurer_before = profile_xp(&game, ADVENTURER);
    let mut now = 1_000;
    MAP.run(&mut game, CHARACTER, &AIDING, ADVENTURER, &mut now);
    MAP.run(&mut game, CHARACTER, &MISPLACED, ADVENTURER, &mut now);
    assert_eq!(
        active_job(&game),
        MINER,
        "the turn-ins must not switch jobs"
    );
    assert_eq!(profile_xp(&game, MINER), miner_before);
    assert_eq!(
        profile_xp(&game, ADVENTURER),
        adventurer_before + 30,
        "the quests' XP goes to the Adventurer profile"
    );
}
