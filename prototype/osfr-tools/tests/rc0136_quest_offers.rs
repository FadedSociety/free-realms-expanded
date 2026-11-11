//! rc-0136 (fix group F1): the quest givers that said nothing in the owner's release-candidate
//! check, walked the way the owner did, through real `Gateway` sessions on the shipped quest
//! catalogue: a trainer's menu for the job, the GM panel's edit for the prerequisite (the
//! `Gameplay::admin_edit` the panel's route stages on an online session, `character_admin.rs`
//! `commit_online`), then a click on the giver.
//!
//! - Caitlyn Gravefog (9000000038), Brawler: Graveyard Grumble (3030015): Brawler learned in the
//!   opening but not active, and Brawler: Grave Danger (3951547) not done.
//! - Petal Oceanmist (9500000613), Kart Driver: The Price of Fame (3080023): Kart Driver learned
//!   from a trainer, not active; Dominating at Frost Ridge (3080022) done through the panel.
//! - Smoggo (9500000213), Fighting is for the Birds! (3957750): Slash and Smash (3955914)
//!   completed through the panel, while its own prerequisite Smoggo's Sword (3955917) stayed
//!   open, so Smoggo kept offering that first quest.
//! - Rizzle (9500000848), A Beautiful Friendship (3953012): the offer said the client's default
//!   "Can you help me?" (8125) for want of an intro id.
//! - Going Batty (3950200): its bat stood at the Seaside mount pen, not in Memorial Caverns; the
//!   wiki's tip puts it just ahead of the Forgotten Caves teleport's arrival.
//!
//! Client layouts: 26/8 interact `008b6bc0`, 26/10 menu select `00a36f80`, 49/2 reply
//! `00a86400`; the 49/1 offer reader `00c7bb60` -> `00c7b7a0` (intro at 10..14), whose intro id
//! `ShowQuestStartScreen` (`0x00a94be0`) formats as QuestNpcIntroTextMessage.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::*;
use osfr_tools::{
    character_admin::{Edit, JobSet, QuestOp},
    gameplay::GameplayState,
    gateway::Gateway,
    npc_interaction::decode_quest_offer_49_1,
    quest_engine::QuestStatus,
};
use serde_json::{Value, json};
use std::path::Path;

const GUID: u64 = CHARACTER;
const BRAWLER: i32 = 43;
const KART_DRIVER: i32 = 48;

const CAITLYN: u64 = 9_000_000_038;
const GRAVEYARD_GRUMBLE: u32 = 3_030_015;
const GRAVE_DANGER: u32 = 3_951_547;

const KART_TRAINER: u64 = 9_500_000_911;
const BECOME_A_KART_DRIVER: u32 = 407_752;
const PETAL: u64 = 9_500_000_613;
const PRICE_OF_FAME: u32 = 3_080_023;
const FROST_RIDGE: u32 = 3_080_022;

const SMOGGO: u64 = 9_500_000_213;
const SMOGGOS_SWORD: u32 = 3_955_917;
const SLASH_AND_SMASH: u32 = 3_955_914;
const FOR_THE_BIRDS: u32 = 3_957_750;

const RIZZLE: u64 = 9_500_000_848;
const BEAUTIFUL_FRIENDSHIP: u32 = 3_953_012;
/// Global.Text 29631: "You're new around here, right? ..." (the string block after the title).
const RIZZLE_OFFER_LINE: u32 = 29_631;
/// The client's default NPC line, what a 49/1 intro of 0 shows (`Ui.GetString(8125)`).
const DEFAULT_LINE: u32 = 8_125;

/// One visit: a session logged in beside `npc` with the character's state so far, the way the
/// owner walked (and relogged) between givers.
fn beside(fixture: &Fixture, npc: u64, state: Option<GameplayState>) -> Gateway {
    let (_, position) = npc_location(npc);
    fixture.session(GUID, position, state)
}

fn state(gateway: &Gateway) -> GameplayState {
    gateway.gameplay().unwrap().state()
}

fn menu(replies: &[Vec<u8>]) -> &[u8] {
    replies
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 9, 0]))
        .expect("26/8 opens the 26/9 menu")
}

/// Talk to `npc`; the 49/1 offer it sends, if any.
fn talk_offer(gateway: &mut Gateway, npc: u64, now: u64) -> Option<u32> {
    let opened = gateway_send(gateway, &npc_interact(npc), now);
    let talk = menu_option_bytes(menu(&opened), TALK_LABEL);
    let mut replies = gateway_send(gateway, &npc_select(npc, talk), now + 1);
    if !replies.iter().any(|p| p.starts_with(&[49, 0, 1, 0])) {
        replies.extend(gateway_send(
            gateway,
            &dialogue_choice(QUEST_CHOICE),
            now + 2,
        ));
    }
    let offer = replies.iter().find(|p| p.starts_with(&[49, 0, 1, 0]))?;
    Some(
        decode_quest_offer_49_1(offer)
            .expect("49/1 decodes")
            .quest_id,
    )
}

/// The panel's Complete, staged on the online session as `commit_online` stages it.
fn panel(gateway: &mut Gateway, edit: Edit) {
    gateway
        .stage_admin_edit(|game| game.admin_edit(&edit, GUID, 9_000))
        .unwrap_or_else(|error| panic!("{edit:?}: {error}"));
}

/// The `no_eligible_quest` log row's reason for `quest` at `npc`.
fn block(gateway: &Gateway, npc: u64, quest: u32) -> Value {
    let rows = gateway.gameplay().unwrap().npc_offer_blocks(npc);
    rows.as_array()
        .into_iter()
        .flatten()
        .find(|row| row["quest_id"] == quest)
        .cloned()
        .unwrap_or_else(|| panic!("{npc} lists quest {quest}: {rows}"))
}

fn status(gateway: &Gateway, quest: u32) -> Option<QuestStatus> {
    let state = state(gateway);
    let quests = state.npc_quests?;
    quests
        .quests
        .get(&quest)
        .map(|progress| progress.status.clone())
}

fn active_job(gateway: &Gateway) -> i32 {
    state(gateway).profiles.unwrap().active
}

#[test]
fn caitlyn_offers_graveyard_grumble_to_a_brawler_in_another_job() {
    let fixture = Fixture::for_character(GUID).with_bulk_quests();
    let mut gateway = beside(&fixture, CAITLYN, None);
    // The opening's save knows Brawler and wears the Adventurer, as the owner's did.
    let profiles = state(&gateway).profiles.unwrap();
    assert!(profiles.unlocked.contains(&BRAWLER) && profiles.active != BRAWLER);
    // Why she said nothing: the quest before hers. The job no longer blocks (learned counts).
    assert_eq!(
        block(&gateway, CAITLYN, GRAVEYARD_GRUMBLE),
        json!({"quest_id": GRAVEYARD_GRUMBLE, "reason": "prerequisite", "missing": [GRAVE_DANGER]})
    );
    assert_eq!(talk_offer(&mut gateway, CAITLYN, 1_000), None);
    panel(&mut gateway, Edit::Quest(QuestOp::Complete, GRAVE_DANGER));
    assert_eq!(status(&gateway, GRAVE_DANGER), Some(QuestStatus::Completed));
    assert_eq!(
        talk_offer(&mut gateway, CAITLYN, 2_000),
        Some(GRAVEYARD_GRUMBLE)
    );
    // Accepting a job quest switches into its job (wiki "Brawler: Restless Rumors").
    let accepted = gateway_send(&mut gateway, &quest_reply(GRAVEYARD_GRUMBLE, true), 3_000);
    assert_eq!(
        status(&gateway, GRAVEYARD_GRUMBLE),
        Some(QuestStatus::Active)
    );
    assert_eq!(active_job(&gateway), BRAWLER);
    assert!(
        accepted.iter().any(|p| p.starts_with(&[38, 0, 21, 0])),
        "the switch sends the job list's 38/21 activation"
    );
}

/// The opening fixture's profiles have no Kart Driver kit; the release's (what players get)
/// does. Give the fixture that kit and its four item rows, so the switch on accept can dress.
fn with_release_kart_kit(fixture: &Fixture) {
    let release = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/release");
    let kit = read_json(&release.join("profiles.json"))["starter_items"]["48"].clone();
    let path = fixture.path().join("profiles.json");
    let mut profiles = read_json(&path);
    profiles["starter_items"]["48"] = kit.clone();
    std::fs::write(&path, serde_json::to_vec(&profiles).unwrap()).unwrap();
    let wanted: Vec<i64> = kit
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_i64)
        .collect();
    let table = read_json(&release.join("item-table.json"));
    let path = fixture.path().join("item-table.json");
    let mut mine = read_json(&path);
    let rows = table["rows"].as_array().unwrap().iter();
    let kit_rows = rows.filter(|row| row[0].as_i64().is_some_and(|id| wanted.contains(&id)));
    mine["rows"]
        .as_array_mut()
        .unwrap()
        .extend(kit_rows.cloned());
    std::fs::write(&path, serde_json::to_vec(&mine).unwrap()).unwrap();
}

#[test]
fn petal_offers_the_price_of_fame_after_a_trainer_and_the_panel() {
    let fixture = Fixture::for_character(GUID).with_bulk_quests();
    with_release_kart_kit(&fixture);
    let mut gateway = beside(&fixture, PETAL, None);
    let missing = block(&gateway, PETAL, PRICE_OF_FAME);
    assert_eq!(missing["reason"], "prerequisite", "{missing}");
    // The trainer the owner used (job_trainers.json: Kart Driver).
    let mut gateway = beside(&fixture, KART_TRAINER, Some(state(&gateway)));
    let opened = gateway_send(&mut gateway, &npc_interact(KART_TRAINER), 1_000);
    let learn = menu_option_bytes(menu(&opened), BECOME_A_KART_DRIVER);
    gateway_send(&mut gateway, &npc_select(KART_TRAINER, learn), 1_001);
    let profiles = state(&gateway).profiles.unwrap();
    assert!(profiles.unlocked.contains(&KART_DRIVER) && profiles.active != KART_DRIVER);
    // The panel completes the quest before hers.
    panel(&mut gateway, Edit::Quest(QuestOp::Complete, FROST_RIDGE));
    let mut gateway = beside(&fixture, PETAL, Some(state(&gateway)));
    assert_eq!(talk_offer(&mut gateway, PETAL, 2_000), Some(PRICE_OF_FAME));
    gateway_send(&mut gateway, &quest_reply(PRICE_OF_FAME, true), 3_000);
    assert_eq!(
        active_job(&gateway),
        KART_DRIVER,
        "accepting switches into Kart Driver"
    );
}

#[test]
fn a_job_never_learned_still_blocks_its_quests_and_the_log_says_so() {
    let fixture = Fixture::for_character(GUID).with_bulk_quests();
    let mut gateway = beside(&fixture, PETAL, None);
    panel(&mut gateway, Edit::Quest(QuestOp::Complete, FROST_RIDGE));
    assert_eq!(
        block(&gateway, PETAL, PRICE_OF_FAME),
        json!({"quest_id": PRICE_OF_FAME, "reason": "job", "job": "kart_driver",
            "learned": false, "need": 1, "have": 0})
    );
    assert_eq!(talk_offer(&mut gateway, PETAL, 1_000), None);
    // The panel gives the job as the trainer does; then she offers.
    panel(
        &mut gateway,
        Edit::Job(KART_DRIVER, JobSet::Learn { active: false }),
    );
    assert_eq!(talk_offer(&mut gateway, PETAL, 2_000), Some(PRICE_OF_FAME));
}

/// The panel's job edits go through the trainer's and the job list's own paths (the client gets
/// the job row and the 38/21 activation), and their undo puts the jobs back.
#[test]
fn the_panel_gives_and_switches_a_job_and_undoes_it() {
    let fixture = Fixture::for_character(GUID).with_bulk_quests();
    with_release_kart_kit(&fixture);
    let mut gateway = beside(&fixture, PETAL, None);
    // The fixture's Adventurer wears no creation clothes, so it could not be switched back to;
    // start from the opening's Brawler, switched to through the panel (the job list's path).
    panel(&mut gateway, Edit::Job(BRAWLER, JobSet::Activate));
    assert_eq!(active_job(&gateway), BRAWLER);
    let before = state(&gateway);
    let edit = Edit::Job(KART_DRIVER, JobSet::Learn { active: true });
    let out = gateway
        .stage_admin_edit(|game| game.admin_edit(&edit, GUID, 9_000))
        .unwrap();
    let sent = |out: &osfr_tools::gateway::GatewayOutcome, head: [u8; 4]| {
        out.replies.iter().any(|m| m.windows(4).any(|w| w == head))
    };
    assert!(sent(&out, [38, 0, 21, 0]), "the switch's 38/21 activation");
    let after = state(&gateway);
    assert_eq!(after.profiles.as_ref().unwrap().active, KART_DRIVER);
    let change = osfr_tools::character_admin::Change::between(&before, &after);
    assert_eq!(
        change.active_job,
        Some([before.profiles.as_ref().unwrap().active, KART_DRIVER])
    );
    let refused = gateway.stage_admin_edit(|game| game.admin_edit(&edit, GUID, 9_100));
    assert!(refused.is_err(), "a job known already is refused");
    panel(&mut gateway, Edit::Apply(change.inverse()));
    let undone = state(&gateway).profiles.unwrap();
    let was = before.profiles.unwrap();
    assert_eq!((undone.active, undone.unlocked), (was.active, was.unlocked));
}

#[test]
fn smoggo_offers_fighting_is_for_the_birds_after_the_panel_completes_slash_and_smash() {
    let fixture = Fixture::for_character(GUID).with_bulk_quests();
    // A grant (the owner's first try) leaves the chain open: Smoggo starts it from the top.
    let mut granted = beside(&fixture, SMOGGO, None);
    panel(&mut granted, Edit::Quest(QuestOp::Grant, SLASH_AND_SMASH));
    assert_eq!(status(&granted, SMOGGOS_SWORD), None);
    assert_eq!(talk_offer(&mut granted, SMOGGO, 1_000), Some(SMOGGOS_SWORD));
    // Complete finishes the quest before it too, so the next one is his offer.
    let mut gateway = beside(&fixture, SMOGGO, None);
    panel(
        &mut gateway,
        Edit::Quest(QuestOp::Complete, SLASH_AND_SMASH),
    );
    assert_eq!(
        status(&gateway, SMOGGOS_SWORD),
        Some(QuestStatus::Completed)
    );
    assert_eq!(
        status(&gateway, SLASH_AND_SMASH),
        Some(QuestStatus::Completed)
    );
    assert_eq!(talk_offer(&mut gateway, SMOGGO, 2_000), Some(FOR_THE_BIRDS));
}

#[test]
fn rizzle_offers_with_his_own_line() {
    let fixture = Fixture::for_character(GUID).with_bulk_quests();
    let mut gateway = beside(&fixture, RIZZLE, None);
    let opened = gateway_send(&mut gateway, &npc_interact(RIZZLE), 1_000);
    let talk = menu_option_bytes(menu(&opened), TALK_LABEL);
    let mut replies = gateway_send(&mut gateway, &npc_select(RIZZLE, talk), 1_001);
    if !replies.iter().any(|p| p.starts_with(&[49, 0, 1, 0])) {
        replies.extend(gateway_send(
            &mut gateway,
            &dialogue_choice(QUEST_CHOICE),
            1_002,
        ));
    }
    let offer = replies
        .iter()
        .find(|p| p.starts_with(&[49, 0, 1, 0]))
        .expect("Rizzle offers");
    let offer = decode_quest_offer_49_1(offer).unwrap();
    assert_eq!(offer.quest_id, BEAUTIFUL_FRIENDSHIP);
    assert_eq!(offer.intro_text_string_id, RIZZLE_OFFER_LINE);
    assert_ne!(offer.intro_text_string_id, DEFAULT_LINE);
}

/// Every enabled quest's offer carries the quest's own text: an intro id, else its description
/// (`quest_offer_packet_with`, R). These enabled quests have neither, so their offer still shows
/// the client's default line; a new quest must not join them.
#[test]
fn every_enabled_quest_offer_has_its_own_text() {
    let mut silent = Vec::new();
    let mut intro = 0;
    for entry in std::fs::read_dir(content_dir().join("quests")).unwrap() {
        let doc = read_json(&entry.unwrap().path());
        for quest in doc["records"].as_array().into_iter().flatten() {
            if quest["enabled"] != true {
                continue;
            }
            if quest["intro_text_string_id"].is_u64() {
                intro += 1;
            } else if !quest["description_string_id"].is_u64() {
                silent.push(quest["id"].as_u64().unwrap());
            }
        }
    }
    silent.sort_unstable();
    assert_eq!(
        silent, SILENT,
        "enabled quests whose offer has no text of their own"
    );
    assert!(
        intro >= 733,
        "the intro ids filled for rc-0136 stay ({intro})"
    );
}

/// Enabled quests with no offer line in the client strings, none on the wiki and no description.
const SILENT: &[u64] = &[3_000_200, 3_040_032, 3_070_002, 3_080_037, 3_140_002];

#[test]
fn going_batty_finds_its_bat_in_the_memorial_caverns() {
    const BAT: u64 = 9_500_001_077;
    let spawns = read_json(&content_dir().join("npc_spawns_extra.json"));
    let rows = spawns["records"].as_array().unwrap();
    let drawn: Vec<&Value> = rows.iter().filter(|r| r["content_key"] == BAT).collect();
    assert_eq!(drawn.len(), 1, "one spawn draws the quest's bat");
    let at: [f32; 3] = serde_json::from_value(drawn[0]["position"].clone()).unwrap();
    // The wiki's tip: teleport to Forgotten Caves (travel.json row 63) and the bat is just ahead,
    // on the arrival's floor, under Sanctuary and not at the Seaside mount pen.
    let travel = read_json(&content_dir().join("travel.json"));
    let caves = (travel["points"].as_array().unwrap().iter())
        .find(|row| row["id"] == 63)
        .expect("the Forgotten Caves teleport");
    assert_eq!(caves["name"], "Forgotten Caves");
    let spawn: [f32; 3] = serde_json::from_value(caves["spawn"].clone()).unwrap();
    let heading = caves["heading"].as_f64().unwrap() as f32;
    let (dx, dz) = (at[0] - spawn[0], at[2] - spawn[2]);
    let ahead = dx * heading.sin() + dz * heading.cos();
    assert!(
        (3.0..8.0).contains(&ahead) && (dx * dx + dz * dz).sqrt() < 8.0,
        "{at:?}"
    );
    assert!(
        (at[1] - spawn[1]).abs() < 0.01,
        "on the arrival's floor: {at:?}"
    );
    let seaside = rows
        .iter()
        .find(|r| r["spawn_guid"] == 100_000_001_736_u64)
        .unwrap();
    assert!(
        seaside.get("content_key").is_none(),
        "the Seaside bat mount is dressing again"
    );
    assert_eq!(
        npc_location(BAT).1,
        at,
        "the NPC row stands where its spawn is drawn"
    );
}
