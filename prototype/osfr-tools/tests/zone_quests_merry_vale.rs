//! T501: The Party Pooper (`3000370`), Suspicious Serenity (`3000371`), Picking
//! Petals (`3000375`), Birthday Surprises (`3000376`) and The Perfect Present
//! (`3000377`), community zone quests imported as data only into
//! `data/content/quests/zone_merry_vale.json`, plus Shannon and Sugaboo, the two
//! talk targets that were drawn only as `npc_spawns_extra.json` rows without a
//! `content_key` and could not be talked to.
//!
//! Every test drives the production paths in `tests/support/npc_quest_chain.rs`:
//! 26/8 -> 26/10 -> 26/6 offers (49/1, decoded with the production reader), 49/2
//! accepts, ordered Talk steps, the 26/6 that releases the held 49/13, and the
//! 49/14 turn-in.
//!
//! Client readers (IDA, 2014 client; no new wire): 26/3 payload reader `0x00a9ef10`
//! stores the first dword after the 4-byte header as the dialogue body, which a
//! completed talk step fills with its `dialogue_string_id`; the speaker sits at
//! bytes `12..20` (`patch_npc_actor`); 49/13 `QuestEndPacket::ReadFields`
//! `0x00c7b990` reads the actor at `6..14`, quest `14..18`, end text `18..22`, one
//! dword, then the reward bundle `0x008e7930`.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, ZoneQuest, ZoneQuests, active_job, assert_no_offer,
    assert_packet_changes_nothing, coins, content_dir, counts, dialogue_body, profile_xp,
    quest_end_reply, quest_reply, quest_status, saved, talk, zone_player,
};
use osfr_tools::{quest_engine::QuestStatus, world::NpcVisibility};

const MAYOR: u64 = 9_500_000_405;
const WUGUNBUMP: u64 = 9_500_000_432;
const RIMPADUN: u64 = 9_500_000_419;
const RATSPLAT: u64 = 9_500_000_418;
/// The new Shannon row this slice adds (the fork's definition 2187).
const SHANNON: u64 = 9_500_001_009;
/// Shannon's runtime actor: the Legacy extra spawn whose `content_key` is `SHANNON`.
const SHANNON_SPAWN: u64 = 100_000_002_187;
const ROSALIE: u64 = 9_500_000_421;
const VERNA: u64 = 9_500_000_431;
const DUBAWUG: u64 = 9_500_000_381;
/// The new Sugaboo row this slice adds (the fork's definition 2279).
const SUGABOO: u64 = 9_500_001_010;
/// Sugaboo's runtime actor: the Legacy extra spawn whose `content_key` is `SUGABOO`.
const SUGABOO_SPAWN: u64 = 100_000_002_279;
const TUMBOLINA: u64 = 9_000_000_222;
const PLEXI: u64 = 9_500_000_417;
const CHEF_JARVIE: u64 = 9_000_000_044;
const OTHER: u64 = CHARACTER + 1;
const MINER: i32 = 14;

/// Every row is in `merry_vale.json`; the two new rows are drawn by their extra spawns.
const MAP: ZoneQuests = ZoneQuests {
    default_file: "merry_vale.json",
    files: &[],
    runtime: &[(SHANNON, SHANNON_SPAWN), (SUGABOO, SUGABOO_SPAWN)],
};

const PARTY_POOPER: ZoneQuest = ZoneQuest {
    id: 3_000_370,
    giver: MAYOR,
    card: [385_597, 385_596, 385_598],
    steps: &[],
    turnin: WUGUNBUMP,
    end_text: 385_599,
    coins: 6,
    xp: 15,
};
const SERENITY: ZoneQuest = ZoneQuest {
    id: 3_000_371,
    giver: WUGUNBUMP,
    card: [74_199, 74_198, 74_200],
    steps: &[
        (RIMPADUN, 74_217),
        (RATSPLAT, 74_222),
        (SHANNON, 74_214),
        (ROSALIE, 74_226),
    ],
    turnin: WUGUNBUMP,
    end_text: 74_201,
    coins: 30,
    xp: 30,
};
const PICKING_PETALS: ZoneQuest = ZoneQuest {
    id: 3_000_375,
    giver: VERNA,
    card: [387_034, 387_033, 387_035],
    steps: &[],
    turnin: DUBAWUG,
    end_text: 387_036,
    coins: 10,
    xp: 15,
};
const BIRTHDAY_SURPRISES: ZoneQuest = ZoneQuest {
    id: 3_000_376,
    giver: DUBAWUG,
    card: [387_044, 387_043, 387_045],
    steps: &[(SUGABOO, 387_057), (TUMBOLINA, 387_059), (PLEXI, 387_061)],
    turnin: DUBAWUG,
    end_text: 387_046,
    coins: 24,
    xp: 30,
};
const PERFECT_PRESENT: ZoneQuest = ZoneQuest {
    id: 3_000_377,
    giver: DUBAWUG,
    card: [387_067, 387_066, 387_068],
    steps: &[(CHEF_JARVIE, 387_080), (PLEXI, 387_081)],
    turnin: VERNA,
    end_text: 387_069,
    coins: 18,
    xp: 30,
};

#[test]
fn party_pooper_then_suspicious_serenity() {
    let visibility = NpcVisibility::load(&content_dir()).expect("NPC visibility loads");
    assert_eq!(
        visibility.content_key(SHANNON_SPAWN),
        Some(SHANNON),
        "the extra spawn keys to the new Shannon row"
    );
    // The regional row is scaffolding: only the extra spawn draws Shannon.
    assert_eq!(visibility.content_key(SHANNON), None);

    let fixture = Fixture::minimal(CHARACTER);
    let mut game = zone_player(&fixture, CHARACTER, ADVENTURER);
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, ADVENTURER));
    let mut now = 1_000;
    MAP.run(&mut game, CHARACTER, &PARTY_POOPER, ADVENTURER, &mut now);
    assert_eq!(quest_status(&game, PARTY_POOPER.id), QuestStatus::Completed);

    // Wugunbump now offers Suspicious Serenity; its goals are ordered.
    MAP.accept(&mut game, CHARACTER, &SERENITY, &mut now);
    let rosalie = MAP.at(CHARACTER, ROSALIE);
    assert_no_offer(&mut game, &rosalie, ROSALIE, &mut now, "Rosalie first");
    assert_eq!(counts(&game, SERENITY.id), [0, 0, 0, 0, 0]);
    MAP.step(&mut game, CHARACTER, &SERENITY, 0, &mut now);
    MAP.step(&mut game, CHARACTER, &SERENITY, 1, &mut now);

    // The Talk is sent to the runtime guid; the 26/3 names that actor as speaker.
    let talked = talk(&mut game, &MAP.at(CHARACTER, SHANNON), SHANNON_SPAWN, now);
    assert_eq!(dialogue_body(&talked), 74_214);
    assert!(talked.changed, "Shannon's Talk did not credit the step");
    let dialogue = talked
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 3, 0]))
        .expect("Shannon answers 26/3");
    assert_eq!(dialogue[12..20], SHANNON_SPAWN.to_le_bytes());
    assert_eq!(counts(&game, SERENITY.id), [1, 1, 1, 0, 0]);
    now += 10;

    MAP.step(&mut game, CHARACTER, &SERENITY, 3, &mut now);
    MAP.turn_in(&mut game, CHARACTER, &SERENITY, ADVENTURER, &mut now);
    assert_eq!(coins(&game), coins_before + 36);
    assert_eq!(profile_xp(&game, ADVENTURER), xp_before + 45);
    assert_eq!(quest_status(&game, SERENITY.id), QuestStatus::Completed);
    let wugunbump = MAP.at(CHARACTER, WUGUNBUMP);
    assert_packet_changes_nothing(
        &mut game,
        &wugunbump,
        &quest_end_reply(SERENITY.id),
        now,
        "a duplicate 49/14",
    );
    assert_no_offer(
        &mut game,
        &wugunbump,
        WUGUNBUMP,
        &mut now,
        "Wugunbump after",
    );
}

#[test]
fn birthday_line_runs_in_order() {
    let visibility = NpcVisibility::load(&content_dir()).expect("NPC visibility loads");
    assert_eq!(
        visibility.content_key(SUGABOO_SPAWN),
        Some(SUGABOO),
        "the extra spawn keys to the new Sugaboo row"
    );
    assert_eq!(visibility.content_key(SUGABOO), None);

    let fixture = Fixture::minimal(CHARACTER);
    let mut game = zone_player(&fixture, CHARACTER, ADVENTURER);
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, ADVENTURER));
    let mut now = 1_000;
    MAP.run(&mut game, CHARACTER, &PICKING_PETALS, ADVENTURER, &mut now);
    assert_eq!(coins(&game), coins_before + 10);
    assert_eq!(profile_xp(&game, ADVENTURER), xp_before + 15);

    MAP.accept(&mut game, CHARACTER, &BIRTHDAY_SURPRISES, &mut now);
    // Sugaboo's Talk is sent to the extra spawn's runtime guid.
    let talked = talk(&mut game, &MAP.at(CHARACTER, SUGABOO), SUGABOO_SPAWN, now);
    assert_eq!(dialogue_body(&talked), 387_057);
    assert!(talked.changed, "Sugaboo's Talk did not credit the step");
    let dialogue = talked
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 3, 0]))
        .expect("Sugaboo answers 26/3");
    assert_eq!(dialogue[12..20], SUGABOO_SPAWN.to_le_bytes());
    assert_eq!(counts(&game, BIRTHDAY_SURPRISES.id), [1, 0, 0, 0]);
    now += 10;
    MAP.step(&mut game, CHARACTER, &BIRTHDAY_SURPRISES, 1, &mut now);
    // Plexi's Talk here credits Birthday Surprises' open step only.
    MAP.step(&mut game, CHARACTER, &BIRTHDAY_SURPRISES, 2, &mut now);
    MAP.turn_in(
        &mut game,
        CHARACTER,
        &BIRTHDAY_SURPRISES,
        ADVENTURER,
        &mut now,
    );
    assert_eq!(coins(&game), coins_before + 34);
    assert_eq!(profile_xp(&game, ADVENTURER), xp_before + 45);

    // The next quest is offered only now, and Plexi's earlier Talk did not
    // credit its Plexi step, which was not active.
    MAP.accept(&mut game, CHARACTER, &PERFECT_PRESENT, &mut now);
    assert_eq!(counts(&game, PERFECT_PRESENT.id), [0, 0, 0]);
    MAP.step(&mut game, CHARACTER, &PERFECT_PRESENT, 0, &mut now);
    assert_eq!(counts(&game, PERFECT_PRESENT.id), [1, 0, 0]);
    MAP.step(&mut game, CHARACTER, &PERFECT_PRESENT, 1, &mut now);
    assert_eq!(counts(&game, PERFECT_PRESENT.id), [1, 1, 0]);
    // The Perfect Present is handed in at Verna, not at its giver Dubawug.
    assert_no_offer(
        &mut game,
        &MAP.at(CHARACTER, DUBAWUG),
        DUBAWUG,
        &mut now,
        "Dubawug with the present open",
    );
    MAP.turn_in(&mut game, CHARACTER, &PERFECT_PRESENT, ADVENTURER, &mut now);
    assert_eq!(coins(&game), coins_before + 52);
    assert_eq!(profile_xp(&game, ADVENTURER), xp_before + 75);
    for quest in [&PICKING_PETALS, &BIRTHDAY_SURPRISES, &PERFECT_PRESENT] {
        assert_eq!(quest_status(&game, quest.id), QuestStatus::Completed);
    }
}

#[test]
fn later_quests_wait() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = zone_player(&fixture, CHARACTER, ADVENTURER);
    let mut now = 1_000;
    let wugunbump = MAP.at(CHARACTER, WUGUNBUMP);
    let dubawug = MAP.at(CHARACTER, DUBAWUG);

    // Suspicious Serenity waits for The Party Pooper.
    assert_no_offer(
        &mut game,
        &wugunbump,
        WUGUNBUMP,
        &mut now,
        "Wugunbump before Party Pooper",
    );
    assert_packet_changes_nothing(
        &mut game,
        &wugunbump,
        &quest_reply(SERENITY.id, true),
        now,
        "a forged 49/2 for Suspicious Serenity",
    );

    // The birthday line waits for Picking Petals.
    assert_no_offer(
        &mut game,
        &dubawug,
        DUBAWUG,
        &mut now,
        "Dubawug before Picking Petals",
    );
    for quest in [BIRTHDAY_SURPRISES.id, PERFECT_PRESENT.id] {
        assert_packet_changes_nothing(
            &mut game,
            &dubawug,
            &quest_reply(quest, true),
            now,
            "a forged 49/2 for a birthday quest",
        );
    }

    MAP.run(&mut game, CHARACTER, &PARTY_POOPER, ADVENTURER, &mut now);
    // The follow-up is offered, but its steps are open: a Talk at the turn-in
    // neither credits nor opens the end screen, and an early 49/14 is refused.
    MAP.accept(&mut game, CHARACTER, &SERENITY, &mut now);
    assert_no_offer(
        &mut game,
        &wugunbump,
        WUGUNBUMP,
        &mut now,
        "Wugunbump with the steps open",
    );
    assert_eq!(counts(&game, SERENITY.id), [0, 0, 0, 0, 0]);
    assert_packet_changes_nothing(
        &mut game,
        &wugunbump,
        &quest_end_reply(SERENITY.id),
        now,
        "an early 49/14",
    );
    // Dubawug still waits for Picking Petals, not for Serenity.
    assert_no_offer(
        &mut game,
        &dubawug,
        DUBAWUG,
        &mut now,
        "Dubawug with Serenity active",
    );
    assert_packet_changes_nothing(
        &mut game,
        &dubawug,
        &quest_reply(BIRTHDAY_SURPRISES.id, true),
        now,
        "a 49/2 for Birthday Surprises mid-Serenity",
    );
}

#[test]
fn progress_survives_reconnect() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = zone_player(&fixture, CHARACTER, ADVENTURER);
    let mut now = 1_000;
    MAP.run(&mut game, CHARACTER, &PARTY_POOPER, ADVENTURER, &mut now);
    MAP.accept(&mut game, CHARACTER, &SERENITY, &mut now);
    MAP.step(&mut game, CHARACTER, &SERENITY, 0, &mut now);
    MAP.step(&mut game, CHARACTER, &SERENITY, 1, &mut now);

    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, SERENITY.id), QuestStatus::Active);
    assert_eq!(counts(&game, SERENITY.id), [1, 1, 0, 0, 0]);
    MAP.step(&mut game, CHARACTER, &SERENITY, 2, &mut now);
    MAP.step(&mut game, CHARACTER, &SERENITY, 3, &mut now);
    MAP.turn_in(&mut game, CHARACTER, &SERENITY, ADVENTURER, &mut now);

    // A finished quest stays finished across a reconnect, and the birthday line
    // resumes mid-quest with its own counts.
    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, SERENITY.id), QuestStatus::Completed);
    MAP.run(&mut game, CHARACTER, &PICKING_PETALS, ADVENTURER, &mut now);
    MAP.accept(&mut game, CHARACTER, &BIRTHDAY_SURPRISES, &mut now);
    MAP.step(&mut game, CHARACTER, &BIRTHDAY_SURPRISES, 0, &mut now);
    let mut game = fixture.reload_serialized(&game);
    assert_eq!(
        quest_status(&game, BIRTHDAY_SURPRISES.id),
        QuestStatus::Active
    );
    assert_eq!(counts(&game, BIRTHDAY_SURPRISES.id), [1, 0, 0, 0]);
    MAP.step(&mut game, CHARACTER, &BIRTHDAY_SURPRISES, 1, &mut now);
    MAP.step(&mut game, CHARACTER, &BIRTHDAY_SURPRISES, 2, &mut now);
    MAP.turn_in(
        &mut game,
        CHARACTER,
        &BIRTHDAY_SURPRISES,
        ADVENTURER,
        &mut now,
    );
}

#[test]
fn two_players_are_independent() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut a = zone_player(&fixture, CHARACTER, ADVENTURER);
    let mut b = zone_player(&fixture, OTHER, ADVENTURER);
    let mut now = 1_000;
    MAP.accept(&mut a, CHARACTER, &PARTY_POOPER, &mut now);
    MAP.accept(&mut b, OTHER, &PARTY_POOPER, &mut now);
    let b_before = saved(&b);
    MAP.turn_in(&mut a, CHARACTER, &PARTY_POOPER, ADVENTURER, &mut now);
    assert_eq!(saved(&b), b_before, "A's run changed B");
    assert_eq!(counts(&b, PARTY_POOPER.id), [0]);
    assert_eq!(
        quest_status(&b, PARTY_POOPER.id),
        QuestStatus::Active,
        "A's Talk credited B's goal"
    );

    // B's own run still pays B, once.
    let b_coins = coins(&b);
    MAP.turn_in(&mut b, OTHER, &PARTY_POOPER, ADVENTURER, &mut now);
    assert_eq!(coins(&b), b_coins + PARTY_POOPER.coins);
    assert_eq!(quest_status(&a, PARTY_POOPER.id), QuestStatus::Completed);
}

#[test]
fn xp_goes_to_adventurer_with_another_job_active() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = zone_player(&fixture, CHARACTER, MINER);
    let miner_before = profile_xp(&game, MINER);
    let adventurer_before = profile_xp(&game, ADVENTURER);
    let mut now = 1_000;
    MAP.run(&mut game, CHARACTER, &PARTY_POOPER, ADVENTURER, &mut now);
    MAP.run(&mut game, CHARACTER, &SERENITY, ADVENTURER, &mut now);
    assert_eq!(active_job(&game), MINER);
    assert_eq!(profile_xp(&game, MINER), miner_before);
    assert_eq!(
        profile_xp(&game, ADVENTURER),
        adventurer_before + 45,
        "the quest's XP must go to the Adventurer profile"
    );
}
