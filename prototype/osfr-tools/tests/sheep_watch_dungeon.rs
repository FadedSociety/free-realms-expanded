//! T319: Sheep Watch as a playable level-1 field battle at Farnum Farmstead.
//!
//! Content-only slice: one new `field_battles.json` row and the existing route
//! (167/1/2 Play or marker 26/8 -> 41/114 start screen -> 41/108 Go -> arena
//! zone 9600 -> 36/10 attacks -> results -> 26/42 Close). No new wire field,
//! handler, opcode, selector or engine code is added.
//!
//! Client shapes checked in the supplied 2014 client:
//! * 41/108 entrance constructor `0x008b6e70` (Go).
//! * 26/42 results Close constructor `0x008a2eb0`, sender `0x0092f300`.
//! * 36/10 basic attack writer `0x00a342e0` (and its reader `0x00a35150`):
//!   bar 1, slot 0, target mode 0, target GUID.
//! * 41/114 start-screen reader chain `0x00aa3430` -> `0x00aa32d0` ->
//!   `0x00a29120` -> `0x009bdd70`; this test checks the title/icon/
//!   description/difficulty fields. The win results are `present_results`
//!   (39/18 game over, 39/47 scores, 39/19 UI), already pinned by
//!   `tests/encounter.rs`; 41/114 is the start/running-encounter screen, not
//!   the results screen (the brief's "41/114 results" wording is corrected
//!   here).
//! * 35/2 AddNpc client reader `0x00921a10` reads the u64 actor id, name id,
//!   model id and transform this test asserts.
//! * 167/1/1 list reader `0x00be79d0` -> `0x00be75c0` -> `0x00be6fa0` ->
//!   `0x00be63e0`; the row layout is the one pinned by `activity_portal.rs`.
//!
//! Every arena enemy carries the recovered `enemies.json` kind: two Alpha
//! Wolves `930050` and three Wolves `930071`. The production engine reports
//! the actor's definition id in `EncounterOutcome::defeated`, so a quest goal
//! authored on either kind is credited (tests 8 and 9). The marker keeps the
//! local actor definition `1235000` because it is a non-combat gate adaptation,
//! not an enemy kind.
//!
//! Placement evidence: the guide's Farnum Farmstead entrance and the video
//! marker 19 m from Pappy. The marker uses the visible Alpha Wolf model (R),
//! like T282's Thugamug marker, at a terrain-sampled point 19.00 m east of
//! Pappy, inside the brief's "within 25 m of Pappy" bound. Its 105-sample 5 m
//! wander disc is terrain-covered and at most 19.96 degrees; the nearest other
//! spawned NPC is 17.38 m away (the 5 m disc stays outside the 12 m NPC
//! interaction radius), and the nearest field enemy and other battle marker are
//! over 200 m away. Water, object collision and a native walkable route remain
//! unverified.

use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    encounter::{EncounterEngine, EncounterPhase},
    gateway::Gateway,
    inventory::{InventoryState, MAX_ITEMS},
    npc_interaction::{QUEST_CHOICE_RESPONSE, decode_dialogue_26_3, decode_quest_offer_49_1},
    quest_engine::{CharacterEligibility, QuestEngine, QuestProgress, QuestState, QuestStatus},
    startup_world::{NpcSpawn, parse_add_npc},
};
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};

#[path = "common/field_battle_fixture.rs"]
mod field_battle_fixture;
#[path = "support/terrain.rs"]
mod terrain;

use field_battle_fixture::*;

/// The new row. 12350 is free on this base and sits after Stone Heart (12303).
const SHEEP_WATCH: u32 = 12350;
const TITLE: i32 = 30404;
const DESCRIPTION: i32 = 92659;
const OBJECTIVE: i32 = 388319;
const DIFFICULTY: u32 = 1;
const REWARD_COINS: u32 = 25;
const MARKER_GUID: u64 = 100001235000;
const ENEMY_GUIDS: [u64; 5] = [
    100001235001,
    100001235002,
    100001235003,
    100001235004,
    100001235005,
];
/// `-32.96` is the bilinear zone-0 height `-32.9577` rounded.
const MARKER: [f32; 3] = [-2165.513, -32.96, 328.1112];
const PAPPY: [f32; 3] = [-2184.513, -31.53728, 328.1112];
const ALPHA_NAME: i32 = 102128;
const WOLF_NAME: i32 = 374;
/// The recovered `enemies.json` kinds: Alpha Wolf and Wolf.
const ALPHA_KIND: i32 = 930050;
const WOLF_KIND: i32 = 930071;
/// Marker adaptation: a non-combat gate has no recovered enemy kind.
const MARKER_DEFINITION: i32 = 1235000;
/// The shipped `Brawler: Delinquent Dilemma` quest (`data/content/quests/brawler.json:405`):
/// defeat four actors whose definition is the recovered Alpha Wolf kind `930050`.
const DELINQUENT_QUEST: u32 = 3030007;
const DELINQUENT_GOAL: u32 = 1;
/// `Brawler: Building your Rep`, the completed prerequisite Delinquent Dilemma
/// needs; its own earlier chain is the seeded reconnect seam, not this test.
const DELINQUENT_PREREQUISITE: u32 = 3030006;
/// Jones, the shipped Delinquent Dilemma giver in `npcs/cobblestone_village.json`.
const JONES: u64 = 9_000_000_121;
/// Generic rebuilt Talk option label (client 26/9 menu).
const TALK_LABEL: u32 = 3_300;
const MODEL: i32 = 3284;
const NAMEPLATE: i32 = 11970;
const BRAWLER: i32 = 43;
const BATTLE_JOB_XP: u32 = 50;
const ARENA_ZONE: u32 = 9600;
const ARENA_ARRIVAL: [f32; 3] = [136.0, -0.5, 155.0];
/// The designated five-battle roster from `field_battle_roster`; the new row
/// is deliberately not part of it.
const TASK_ROSTER: [u32; 5] = [96, 12301, 12302, 12310, 12311];
const REFUSAL_PRIZE: &str = "one of its rewards cannot be granted";

// ---------------------------------------------------------------------------
// Local fixture seams: the full bag and the shipped defeat goal.
// ---------------------------------------------------------------------------

impl Fixture {
    /// Restore a save built by the production `game_store` path with a bag of
    /// exactly `capacity` stacks. The seed has entered the world and is on a
    /// level-1 combat job.
    fn gateway_with_bag(
        &self,
        position: [f32; 3],
        guid: u64,
        capacity: usize,
    ) -> (Gateway, InventoryState) {
        self.gateway_with_bag_seeded(position, guid, capacity, SHEEP_WATCH, |seed| {
            assert!(send(seed, &switch_job(BRAWLER), 300).state_changed);
            assert_eq!(level(seed, BRAWLER), 1);
        })
    }

    /// Restore a save whose NPC quest state carries the given entries, through
    /// the same production bind a reconnect uses. The seed is on the Brawler
    /// job at level 1. Test 9 seeds only Delinquent Dilemma's completed
    /// prerequisite; the target quest is then offered and accepted live.
    fn gateway_with_quest(&self, position: [f32; 3], guid: u64, quests: QuestState) -> Gateway {
        let mut seed = self.gateway(position, guid);
        assert!(send(&mut seed, &switch_job(BRAWLER), 300).state_changed);
        assert_eq!(level(&seed, BRAWLER), 1);

        let mut snapshot = seed.snapshot();
        let seeded = quests.clone();
        snapshot.gameplay.as_mut().unwrap().npc_quests = Some(quests);
        let gameplay = snapshot.gameplay.unwrap();
        let mut gateway = Gateway::load_gameplay(
            &self.reference,
            seed.state().clone(),
            &self.content,
            Some(gameplay),
        )
        .unwrap();
        gateway.set_dev_tools(true);
        gateway.set_role(AccountRole::Admin);
        self.login(&mut gateway);
        send(&mut gateway, &13_u16.to_le_bytes(), 400);
        let loaded = send(&mut gateway, &10_u16.to_le_bytes(), 500);
        assert!(loaded.entered_world);
        let restored = gateway
            .snapshot()
            .gameplay
            .unwrap()
            .npc_quests
            .expect("the catalogue bind keeps the saved quest state");
        for (quest, progress) in &seeded.quests {
            assert_eq!(
                restored.quests.get(quest),
                Some(progress),
                "world entry restores the seeded quest {quest} unchanged"
            );
        }
        gateway
    }
}

// ---------------------------------------------------------------------------
// Directed visibility: the 35/2 AddNpc the client reader 0x00921a10 parses.
// ---------------------------------------------------------------------------

fn marker_packet<'a>(replies: &'a [Vec<u8>], guid: u64) -> &'a [u8] {
    replies
        .iter()
        .map(|packet| unframe(packet))
        .find(|packet| {
            packet.starts_with(&[35, 0, 2, 0])
                && packet.len() >= 12
                && u64::from_le_bytes(packet[4..12].try_into().unwrap()) == guid
        })
        .unwrap_or_else(|| panic!("actor {guid} was not sent as 35/2 AddNpc"))
}

fn marker_spawn(packet: &[u8]) -> NpcSpawn {
    parse_add_npc(packet).expect("the 35/2 AddNpc body parses through the production reader")
}

// ---------------------------------------------------------------------------
// Published 167/1/1 activity list (shared row reader in the fixture module).
// ---------------------------------------------------------------------------

fn published_list(replies: &[Vec<u8>]) -> Vec<PortalRow> {
    let packet = replies
        .iter()
        .map(|packet| unframe(packet))
        .find(|packet| packet.starts_with(&[167, 0, 1, 1]))
        .expect("a 167/1/1 activity list was sent");
    decode_list(packet)
}

// ---------------------------------------------------------------------------
// NPC quest state: the shipped defeat goal keyed to the Alpha Wolf kind.
// ---------------------------------------------------------------------------

/// The saved state a reconnect restores after `Brawler: Building your Rep`
/// completes. Only that prerequisite is seeded; test 9 offers and accepts
/// Delinquent Dilemma through the real NPC route afterwards.
fn delinquent_prerequisite(guid: u64) -> QuestState {
    let engine = QuestEngine::from_split_content(
        &content_dir(),
        guid,
        CharacterEligibility {
            player_level: 20,
            job_levels: BTreeMap::new(),
            member: true,
            nonmember_quest_limit: 30,
            member_quest_limit: 30,
        },
        None,
    )
    .expect("the split quest catalogue loads");
    let mut state = engine.state().clone();
    state.quests.insert(
        DELINQUENT_PREREQUISITE,
        QuestProgress {
            status: QuestStatus::Completed,
            counts: vec![1],
        },
    );
    state
}

/// Jones's shipped content position (`npcs/cobblestone_village.json`).
fn jones_position() -> [f32; 3] {
    let npcs = read_json(&content_dir().join("npcs/cobblestone_village.json"));
    let row = npcs["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["npc_id"].as_u64() == Some(JONES))
        .expect("Jones is defined in cobblestone_village.json");
    position(&row["position"])
}

fn saved_quest_count(gateway: &Gateway, quest: u32) -> u32 {
    quest_counts(gateway, quest).first().copied().unwrap_or(0)
}

/// 49/9 ObjectiveUpdate readers: header, quest id, objective id, current count,
/// progress float, notification bool and one unresolved dword. Body reader
/// `0x00c7af60` (pinned by `tests/shared_field_kill_quest_credit.rs`).
fn objective_updates(replies: &[Vec<u8>], quest: u32) -> Vec<u32> {
    replies
        .iter()
        .map(|packet| unframe(packet))
        .filter(|packet| packet.starts_with(&[49, 0, 9, 0]) && packet.len() >= 22)
        .filter(|packet| u32::from_le_bytes(packet[6..10].try_into().unwrap()) == quest)
        .map(|packet| {
            assert_eq!(
                u32::from_le_bytes(packet[10..14].try_into().unwrap()),
                DELINQUENT_GOAL
            );
            u32::from_le_bytes(packet[14..18].try_into().unwrap())
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

/// 1. The row publishes a drawable 35/2 Alpha Wolf marker at the sampled
/// Farnum point, a second independent session receives the identical actor,
/// and the portal lists Sheep Watch once with `can_join` set for a level-1
/// combat job.
#[test]
fn the_marker_is_drawable_and_the_portal_publishes_sheep_watch() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, SHEEP_WATCH);
    assert_eq!(u32_field(row, "title_id"), TITLE as u32);
    assert_eq!(u32_field(row, "description_id"), DESCRIPTION as u32);
    assert_eq!(u32_field(row, "objective_id"), OBJECTIVE as u32);
    assert_eq!(u32_field(row, "difficulty"), DIFFICULTY);
    assert_eq!(u32_field(row, "required_combat_level"), 0);
    assert_eq!(u32_field(row, "reward_coins"), REWARD_COINS);
    assert_eq!(i32_field(row, "icon_id"), -1);
    assert_eq!(i32_field(row, "nameplate_image_id"), NAMEPLATE);
    assert_eq!(row["spawner"]["boss"], false);
    assert_eq!(u32_field(&row["spawner"], "max_health"), 1);
    assert_eq!(u32_field(&row["spawner"], "damage"), 0);

    let marker = position(&row["spawner"]["position"]);
    assert_eq!(marker, MARKER);
    let pappy_distance = (marker[0] - PAPPY[0]).hypot(marker[2] - PAPPY[2]);
    assert!(
        (8.0..=25.0).contains(&pappy_distance),
        "the marker is {pappy_distance:.2} m from Pappy Farnum"
    );
    assert_eq!(u32_field(&row["zone"], "zone_id"), ARENA_ZONE);
    assert_eq!(row["zone"]["name"], "sg_random_encounter_clearing");
    assert_eq!(row["zone"]["sky"], "sky_wilds24.xml");
    assert_eq!(u32_field(&row["zone"], "geometry_id"), 214);
    assert_eq!(position(&row["zone"]["position"]), ARENA_ARRIVAL);

    // The full 5 m wander disc is standable under the T103 authoring rule.
    let terrain = terrain::load();
    let grid = terrain.grid(0).expect("zone 0 is loaded");
    let ground = grid
        .height_at(marker[0], marker[2])
        .expect("the marker is on covered ground");
    assert!(
        (marker[1] - ground).abs() <= 0.05,
        "marker y {} is {:.4} m from the sampled ground {ground}",
        marker[1],
        (marker[1] - ground).abs()
    );
    for angle in (0..360).step_by(15) {
        let radians = (angle as f32).to_radians();
        let sample = grid
            .height_at(
                marker[0] + 5.0 * radians.cos(),
                marker[2] + 5.0 * radians.sin(),
            )
            .unwrap_or_else(|| panic!("ring sample {angle} degrees has no ground"));
        assert!(sample.is_finite());
    }
    let disc = terrain::disc_points(marker[0], marker[2]);
    assert_eq!(disc.len(), 105);
    for (x, z) in &disc {
        assert!(
            grid.height_at(*x, *z).is_some(),
            "disc sample ({x},{z}) has no ground"
        );
    }
    let max_slope = disc
        .iter()
        .map(|(x, z)| terrain::slope_degrees(grid, *x, *z))
        .fold(0.0_f32, f32::max);
    assert!(
        max_slope <= 25.0,
        "the full 5 m wander disc reaches {max_slope:.2} degrees"
    );
    // Clear of other click targets: the 5 m disc stays outside the 12 m NPC
    // interaction radius, and the field-enemy/marker gap follows T103.
    let (npc, npc_id) = nearest_spawned_npc(marker);
    assert!(
        npc - 5.0 >= 12.0,
        "nearest spawned NPC {npc_id} is {npc:.2} m away"
    );
    let enemy = nearest_field_enemy(marker);
    assert!(enemy >= 25.0, "nearest field enemy is {enemy:.1} m away");
    let other = nearest_other_battle_marker(marker, SHEEP_WATCH);
    assert!(other >= 25.0, "nearest battle marker is {other:.1} m away");

    // Two independent sessions each receive the same static marker.
    let (mut first, first_entry) = fixture.session(MARKER, 1001);
    let (_second, second_entry) = fixture.session(MARKER, 1002);
    let first_packet = marker_packet(&first_entry.replies, MARKER_GUID);
    let second_packet = marker_packet(&second_entry.replies, MARKER_GUID);
    assert_eq!(
        first_packet, second_packet,
        "the marker is identical for both sessions (no per-player actor)"
    );
    let spawn = marker_spawn(first_packet);
    assert_eq!(spawn.actor_id, MARKER_GUID);
    assert_eq!(spawn.name_string_id, ALPHA_NAME);
    assert_eq!(spawn.model_id, MODEL);
    assert_eq!(spawn.position[0], MARKER[0]);
    assert_eq!(spawn.position[1], MARKER[1]);
    assert_eq!(spawn.position[2], MARKER[2]);
    assert_eq!(spawn.nameplate_image_id, NAMEPLATE);
    let catalogued = read_json(&lab("reports/locale-id-to-text.json"));
    let names: BTreeMap<i32, String> = catalogued
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            (
                entry["id"].as_i64().unwrap() as i32,
                entry["text"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(
        names.get(&ALPHA_NAME).map(String::as_str),
        Some("Alpha Wolf")
    );
    assert_eq!(names.get(&WOLF_NAME).map(String::as_str), Some("Wolf"));
    assert_eq!(names.get(&TITLE).map(String::as_str), Some("Sheep Watch"));
    assert_eq!(
        names.get(&DESCRIPTION).map(String::as_str),
        Some(
            "A pack of wolves is attacking the sheep at Farnum Farmstead. Save the sheep by defeating the wolves!"
        )
    );
    assert_eq!(
        names.get(&OBJECTIVE).map(String::as_str),
        Some("Defeat the Ravenous Wolves at Sheep Watch")
    );
    let models =
        fs::read_to_string(lab("working/OSFR.Client/OSFR Client/Resources/Models.txt")).unwrap();
    assert!(
        models
            .lines()
            .any(|line| line.starts_with("3284^wolf_generic.adr^")),
        "model 3284 is in Models.txt"
    );
    let catalog = fs::read_to_string(lab("extracted/client-assets/catalog.tsv")).unwrap();
    assert!(
        catalog.contains("\twolf_generic.adr\t"),
        "model 3284 is present in catalog.tsv"
    );

    // At world entry the default Adventurer job is not a combat job; the
    // production job switch plus 167/1/4 refresh flips CanPlayerJoin.
    let entry_rows = published_list(&first_entry.replies);
    let entry_row = entry_rows
        .iter()
        .find(|row| row.id == SHEEP_WATCH)
        .expect("Sheep Watch is in the 167/1/1 list");
    assert_eq!(
        entry_rows
            .iter()
            .filter(|row| row.id == SHEEP_WATCH)
            .count(),
        1
    );
    assert_eq!(entry_row.title, TITLE as u32);
    assert_eq!(entry_row.description, DESCRIPTION as u32);
    assert_eq!(entry_row.difficulty, DIFFICULTY);
    assert!(!entry_row.can_join, "a non-combat job cannot join");

    assert!(send(&mut first, &switch_job(BRAWLER), 300).state_changed);
    assert_eq!(level(&first, BRAWLER), 1);
    let refreshed = send(&mut first, &[167, 0, 1, 4], 400);
    let rows = published_list(&refreshed.replies);
    let published = rows
        .iter()
        .find(|row| row.id == SHEEP_WATCH)
        .expect("the refreshed list still contains Sheep Watch");
    assert!(published.can_join, "a level-1 combat job may join");
    assert_eq!(published.title, TITLE as u32);
    assert_eq!(published.description, DESCRIPTION as u32);
    assert_eq!(published.difficulty, DIFFICULTY);
}

/// 2. A level-1 combat job starts from the marker, five enemies spawn in zone
/// 9600, the live run refuses another portal Play through the shared busy gate,
/// ordinary 36/10 attacks win, the results packets open, Close returns to the
/// marker, and 25 coins, one prize and 50 job XP (active Brawler only) are paid
/// exactly once.
#[test]
fn a_level_one_character_starts_wins_and_returns() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, SHEEP_WATCH);
    let prizes = prize_ids(row);
    let mut gateway = fixture.gateway(MARKER, 2001);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    assert_eq!(active_profile(&gateway), BRAWLER);
    assert_eq!(level(&gateway, BRAWLER), 1);

    let before = economy(&gateway, &prizes);
    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    let enemy_guids = enter_battle(&mut gateway, &screen, row, 1_000);
    assert_eq!(enemy_guids, ENEMY_GUIDS);
    // The same activity is refused while its own run is live (the shared busy
    // gate), without touching the arena state.
    let busy = send(&mut gateway, &portal_play(SHEEP_WATCH), 1_900);
    assert!(!busy.began_zoning && !busy.state_changed);
    assert!(
        chat_text(&busy).contains("Finish what you are doing"),
        "{}",
        chat_text(&busy)
    );
    assert_eq!(
        gateway
            .snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active
            .map(|active| active.phase),
        Some(EncounterPhase::Fighting)
    );
    let mut now = 2_000;
    let replies = fight_to_win(&mut gateway, &mut now, row);
    assert_eq!(
        results_lifecycle(&replies),
        vec![18, 47, 19],
        "the results open through 39/18, 39/47 and 39/19"
    );
    assert_eq!(won_flag(&replies), Some(1), "the results report a win");

    let paid = economy(&gateway, &prizes);
    assert_payout(&before, &paid, row, &prizes, BATTLE_JOB_XP);
    assert_eq!(active_profile(&gateway), BRAWLER);
    assert_eq!(
        paid.experience.get(&BRAWLER).copied().unwrap_or(0),
        before.experience.get(&BRAWLER).copied().unwrap_or(0) + BATTLE_JOB_XP
    );

    let won = gateway.snapshot();
    let repeated = send(&mut gateway, &melee(enemy_guids[0]), now + 700);
    assert!(!repeated.state_changed, "a won fight cannot pay twice");
    assert_eq!(gateway.snapshot(), won);

    let closed = send(&mut gateway, &[26, 0, 42, 0], now + 1_400);
    assert!(closed.began_zoning, "Close returns to the open world");
    assert_eq!(gateway.state().zone_id, 0);
    assert_eq!(gateway.state().position, MARKER);
    assert_eq!(
        economy(&gateway, &prizes),
        paid,
        "leaving pays nothing more"
    );
    let after_close = gateway.snapshot();
    let duplicate = send(&mut gateway, &[26, 0, 42, 0], now + 2_100);
    assert!(!duplicate.began_zoning && !duplicate.state_changed);
    assert_eq!(
        gateway.snapshot(),
        after_close,
        "a duplicate results Close changes nothing"
    );
}

/// 3. Field battles are repeatable (`guide.is_none()`): a second completed
/// run pays 25 coins, one prize and 50 job XP again, and the first run's Go,
/// enemy hit and Close replay without paying.
#[test]
fn a_second_run_pays_again_once_and_only_once() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, SHEEP_WATCH);
    let prizes = prize_ids(row);
    let mut gateway = fixture.gateway(MARKER, 3001);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    let before = economy(&gateway, &prizes);

    let mut now = 500;
    let first_screen = open_battle(&mut gateway, MARKER_GUID, row, now);
    now += 100;
    let first_enemies = enter_battle(&mut gateway, &first_screen, row, now);
    let first_replies = fight_to_win(&mut gateway, &mut now, row);
    assert_eq!(results_lifecycle(&first_replies), vec![18, 47, 19]);
    let paid_once = economy(&gateway, &prizes);
    assert_payout(&before, &paid_once, row, &prizes, BATTLE_JOB_XP);
    let closed = send(&mut gateway, &[26, 0, 42, 0], now + 200);
    assert!(closed.began_zoning);
    assert_eq!(gateway.state().zone_id, 0);
    assert_eq!(gateway.state().position, MARKER);
    // The client sends the scene-ready pair after every zoning reply; load the
    // world again before the second start.
    arena_spawned(&mut gateway, now + 300);

    now += 500;
    let second_screen = open_battle(&mut gateway, MARKER_GUID, row, now);
    now += 100;
    enter_battle(&mut gateway, &second_screen, row, now);
    let second_replies = fight_to_win(&mut gateway, &mut now, row);
    assert_eq!(results_lifecycle(&second_replies), vec![18, 47, 19]);
    let paid_twice = economy(&gateway, &prizes);
    assert_payout(&paid_once, &paid_twice, row, &prizes, BATTLE_JOB_XP);
    let closed = send(&mut gateway, &[26, 0, 42, 0], now + 200);
    assert!(closed.began_zoning);
    assert_eq!(gateway.state().zone_id, 0);
    arena_spawned(&mut gateway, now + 300);

    // The first run's screen/Go, enemy GUID and results Close are inert now.
    // 41/114 is a server-to-client screen; its client-side replay is the Go
    // built from those bytes, which is what this asserts.
    let after_runs = gateway.snapshot();
    let replay_go = send(&mut gateway, &go(&first_screen), now + 500);
    assert!(!replay_go.began_zoning && !replay_go.state_changed);
    let replay_hit = send(&mut gateway, &melee(first_enemies[0]), now + 600);
    assert!(!replay_hit.state_changed);
    let replay_close = send(&mut gateway, &[26, 0, 42, 0], now + 700);
    assert!(!replay_close.began_zoning && !replay_close.state_changed);
    assert_eq!(
        gateway.snapshot(),
        after_runs,
        "first-run packets pay nothing after the second run"
    );
    assert_eq!(economy(&gateway, &prizes), paid_twice);
}

/// 4. Forged Go, a forged portal Play id, a wrong marker, out-of-range Go, an
/// arbitrary 36/10 target, a non-combat job, a knocked-out character and the
/// full-bag preflight each change no typed state or save bytes and leave the
/// session alive. Controls prove the same requests start under the qualifying
/// condition.
#[test]
fn refusals_change_nothing() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, SHEEP_WATCH);
    let prizes = prize_ids(row);

    // Forged Go with no open start screen.
    let mut gateway = fixture.gateway(MARKER, 4001);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    let before = (gateway.snapshot(), saved_bytes(&gateway));
    let mut forged = vec![41, 0, 108, 0];
    forged.extend(SHEEP_WATCH.to_le_bytes());
    forged.extend(0x7fff_i32.to_le_bytes());
    forged.extend(0_u64.to_le_bytes());
    let refused = send(&mut gateway, &forged, 400);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(refused.replies.is_empty(), "a forged Go has no reply");
    assert_eq!(
        (gateway.snapshot(), saved_bytes(&gateway)),
        before,
        "a forged Go changes no state"
    );
    // A forged 167/1/2 Play id: the catalogue does not publish it, so the
    // request is one chat refusal and changes nothing.
    let before = (gateway.snapshot(), saved_bytes(&gateway));
    let unknown = send(&mut gateway, &portal_play(0x00ff_ffff), 430);
    assert!(!unknown.began_zoning && !unknown.state_changed);
    assert!(
        chat_text(&unknown).contains("That activity is not available."),
        "{}",
        chat_text(&unknown)
    );
    assert_eq!(
        (gateway.snapshot(), saved_bytes(&gateway)),
        before,
        "a forged portal Play id changes no state"
    );
    // The session still answers an ordinary refresh.
    assert!(
        published_list(&send(&mut gateway, &[167, 0, 1, 4], 450).replies)
            .iter()
            .any(|row| row.id == SHEEP_WATCH)
    );

    // Wrong marker: a forged actor GUID opens no start screen.
    let before = (gateway.snapshot(), saved_bytes(&gateway));
    let wrong = send(&mut gateway, &interact(0xdead_beef), 500);
    assert!(details(&wrong).is_none());
    assert!(!wrong.state_changed && !wrong.began_zoning);
    assert_eq!(
        (gateway.snapshot(), saved_bytes(&gateway)),
        before,
        "a wrong marker changes no state"
    );

    // Out-of-range Go: the screen opens at the marker, the player walks 50 m
    // away and Go is inert. Reopening the screen at the marker then works.
    let mut gateway = fixture.gateway(MARKER, 4002);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    let player = gateway.state().guid;
    let mut now = 600;
    walk(
        &mut gateway,
        player,
        [MARKER[0] + 50.0, MARKER[1], MARKER[2]],
        &mut now,
    );
    let before = (gateway.snapshot(), saved_bytes(&gateway));
    let refused = send(&mut gateway, &go(&screen), now + 100);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert_eq!(
        (gateway.snapshot(), saved_bytes(&gateway)),
        before,
        "an out-of-range Go changes no state"
    );
    walk(&mut gateway, player, MARKER, &mut now);
    let screen = open_battle(&mut gateway, MARKER_GUID, row, now + 100);
    assert!(send(&mut gateway, &go(&screen), now + 200).began_zoning);
    assert_eq!(gateway.state().zone_id, ARENA_ZONE);
    arena_spawned(&mut gateway, now + 250);
    // An arbitrary 36/10 target GUID is not an actor in this arena, so the
    // attack is inert: no defeat, no reward and no arena mutation.
    let before = (gateway.snapshot(), saved_bytes(&gateway));
    let arbitrary = send(&mut gateway, &melee(0xdead_beef), now + 450);
    assert!(!arbitrary.began_zoning && !arbitrary.state_changed);
    assert_eq!(
        (gateway.snapshot(), saved_bytes(&gateway)),
        before,
        "an arbitrary 36/10 target changes no state"
    );
    let instance = i32_at(&screen, 8);
    let cancel = [
        vec![41, 0, 109, 0],
        0_i32.to_le_bytes().to_vec(),
        instance.to_le_bytes().to_vec(),
    ]
    .concat();
    assert!(send(&mut gateway, &cancel, now + 500).began_zoning);
    assert_eq!(gateway.state().position, MARKER);

    // Non-combat job: the same Go is a chat refusal that reopens the screen.
    let mut civilian = fixture.gateway(MARKER, 4003);
    let screen = open_battle(&mut civilian, MARKER_GUID, row, 400);
    let before = (civilian.snapshot(), saved_bytes(&civilian));
    let refused = send(&mut civilian, &go(&screen), 500);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(
        chat_text(&refused).contains("Switch to a combat job before starting a battle."),
        "{}",
        chat_text(&refused)
    );
    assert_eq!(
        details(&refused).expect("the start screen reopens"),
        screen.as_slice()
    );
    assert_eq!(
        (civilian.snapshot(), saved_bytes(&civilian)),
        before,
        "a job refusal changes no state"
    );
    assert!(send(&mut civilian, &switch_job(BRAWLER), 600).state_changed);
    let started = send(&mut civilian, &go(&screen), 700);
    assert!(started.began_zoning, "the control job starts the battle");

    // Knocked out: Go is a refusal, not a session-ending error.
    let mut knocked = fixture.gateway(MARKER, 4004);
    assert!(send(&mut knocked, &switch_job(BRAWLER), 300).state_changed);
    let screen = open_battle(&mut knocked, MARKER_GUID, row, 400);
    knocked.receive_field_damage(u32::MAX, 500, true).unwrap();
    assert_eq!(
        knocked
            .snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .player_health,
        0
    );
    let before = (knocked.snapshot(), saved_bytes(&knocked));
    let refused = knocked
        .handle_at(&wrap(&go(&screen)), 600)
        .expect("a knocked-out Go is a chat refusal, not a session error");
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(
        chat_text(&refused).contains("knocked out"),
        "{}",
        chat_text(&refused)
    );
    assert_eq!(
        details(&refused).expect("the start screen reopens"),
        screen.as_slice()
    );
    assert_eq!(
        (knocked.snapshot(), saved_bytes(&knocked)),
        before,
        "a knocked-out Go changes no state"
    );

    // Full bag: the four-prize bundle has no room, so Go is refused.
    let (mut full, bag) = fixture.gateway_with_bag(MARKER, 4005, MAX_ITEMS);
    assert_eq!(bag.items.len(), MAX_ITEMS);
    assert!(
        !bag.items
            .iter()
            .any(|item| prizes.contains(&item.definition_id))
    );
    let screen = open_battle(&mut full, MARKER_GUID, row, 600);
    let before = (full.snapshot(), saved_bytes(&full));
    let refused = send(&mut full, &go(&screen), 700);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(
        chat_text(&refused).contains(REFUSAL_PRIZE),
        "{}",
        chat_text(&refused)
    );
    assert_eq!(
        details(&refused).expect("the start screen reopens"),
        screen.as_slice()
    );
    assert_eq!(
        (full.snapshot(), saved_bytes(&full)),
        before,
        "a full bag changes no state"
    );

    // Control: four free stacks fit the bundle and the same battle starts.
    let (mut fitting, bag) = fixture.gateway_with_bag(MARKER, 4006, MAX_ITEMS - 4);
    assert_eq!(bag.items.len(), MAX_ITEMS - 4);
    let screen = open_battle(&mut fitting, MARKER_GUID, row, 800);
    let started = send(&mut fitting, &go(&screen), 900);
    assert!(started.began_zoning, "the fitting bundle starts the battle");
    assert_eq!(fitting.state().zone_id, ARENA_ZONE);
}

/// 5. A run dropped by save/reload returns to the world without a second
/// payout or stuck busy state; two independent sessions keep their own runs.
#[test]
fn a_disconnect_mid_run_drops_the_run_without_paying() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, SHEEP_WATCH);
    let prizes = prize_ids(row);

    // No-win variant: a fighting run is saved and reloaded. No payout, no
    // active run, and the marker can start a fresh battle.
    let mut gateway = fixture.gateway(MARKER, 5001);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    let before = economy(&gateway, &prizes);
    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    enter_battle(&mut gateway, &screen, row, 600);
    let mut dropped = reopen(&fixture, &gateway, "mid-run.json", 2_000);
    assert_eq!(dropped.state().zone_id, 0);
    assert_eq!(dropped.state().position, MARKER);
    assert_eq!(economy(&dropped, &prizes), before);
    assert!(
        dropped
            .snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active
            .is_none(),
        "a reload drops the abandoned run"
    );
    let screen = open_battle(&mut dropped, MARKER_GUID, row, 2_500);
    let restarted = send(&mut dropped, &go(&screen), 2_600);
    assert!(
        restarted.began_zoning,
        "the dropped run leaves no stuck busy state"
    );

    // Won, unclosed variant: a real 36/10 win applies the payout; reloading
    // must not pay again and a replayed attack must be inert.
    let mut gateway = fixture.gateway(MARKER, 5002);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    let before = economy(&gateway, &prizes);
    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    enter_battle(&mut gateway, &screen, row, 600);
    let mut now = 700;
    let won = fight_to_win(&mut gateway, &mut now, row);
    assert_eq!(results_lifecycle(&won), vec![18, 47, 19]);
    let paid = economy(&gateway, &prizes);
    assert_payout(&before, &paid, row, &prizes, BATTLE_JOB_XP);
    let mut dropped = reopen(&fixture, &gateway, "won-unclosed.json", now + 1_000);
    assert_eq!(economy(&dropped, &prizes), paid);
    assert_eq!(dropped.state().zone_id, 0);
    assert_eq!(dropped.state().position, MARKER);
    let before_replay = saved_bytes(&dropped);
    let replay = send(&mut dropped, &melee(ENEMY_GUIDS[0]), now + 1_100);
    assert!(!replay.state_changed, "a reloaded win pays nothing");
    assert_eq!(saved_bytes(&dropped), before_replay);
    assert!(
        dropped
            .snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active
            .is_none()
    );

    // Two independent sessions: the first player's win and disconnect do not
    // change the second player's live run or save.
    let mut first = fixture.gateway(MARKER, 5101);
    let mut second = fixture.gateway(MARKER, 5102);
    for session in [&mut first, &mut second] {
        assert!(send(session, &switch_job(BRAWLER), 300).state_changed);
    }
    let first_before = economy(&first, &prizes);
    let second_before = economy(&second, &prizes);
    let first_screen = open_battle(&mut first, MARKER_GUID, row, 500);
    let second_screen = open_battle(&mut second, MARKER_GUID, row, 500);
    enter_battle(&mut first, &first_screen, row, 600);
    enter_battle(&mut second, &second_screen, row, 600);
    let second_save = saved_bytes(&second);
    let second_active = second
        .snapshot()
        .gameplay
        .unwrap()
        .encounter
        .unwrap()
        .active;
    assert!(second_active.is_some());

    let mut first_now = 700;
    fight_to_win(&mut first, &mut first_now, row);
    let first_paid = economy(&first, &prizes);
    assert_payout(&first_before, &first_paid, row, &prizes, BATTLE_JOB_XP);
    assert_eq!(
        economy(&second, &prizes),
        second_before,
        "the first player's win never pays the second"
    );
    assert_eq!(
        second
            .snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active,
        second_active,
        "the second player's run is unchanged"
    );
    let dropped = reopen(&fixture, &first, "first-dropped.json", first_now + 1_000);
    assert_eq!(economy(&dropped, &prizes), first_paid);
    assert_eq!(
        saved_bytes(&second),
        second_save,
        "the first player's disconnect never changes the second player's save"
    );
    assert_eq!(
        second
            .snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active,
        second_active,
        "the second player's run survives the first player's disconnect"
    );
    let mut second_now = 700;
    fight_to_win(&mut second, &mut second_now, row);
    let second_paid = economy(&second, &prizes);
    assert_payout(&second_before, &second_paid, row, &prizes, BATTLE_JOB_XP);
    assert_eq!(
        economy(&dropped, &prizes),
        first_paid,
        "the second player's win never pays the first"
    );
}

/// 6. Loader and reward bounds: unique ids/GUIDs, 1..=16 enemies (the loader's
/// real cap, not the brief's 32), positive text ids, a bounded coin payout,
/// one-prize-per-win rows and the zone-9600 arena; synthetic duplicate
/// rows/GUIDs are refused by the real loader.
#[test]
fn the_row_is_inside_the_loader_and_reward_bounds() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, SHEEP_WATCH);
    let rows = file["battles"].as_array().unwrap();
    assert_eq!(
        rows.iter()
            .filter(|row| row["id"].as_u64() == Some(u64::from(SHEEP_WATCH)))
            .count(),
        1
    );
    let mut guids = BTreeSet::new();
    for actor in std::iter::once(&row["spawner"]).chain(row["enemies"].as_array().unwrap().iter()) {
        let guid = u64_field(actor, "guid");
        assert!(guid > 1);
        assert!(guids.insert(guid), "the new row repeats GUID {guid}");
    }
    for actor in rows.iter().filter(|row| row["id"] != json!(SHEEP_WATCH)) {
        for actor in
            std::iter::once(&actor["spawner"]).chain(actor["enemies"].as_array().unwrap().iter())
        {
            assert!(
                guids.insert(u64_field(actor, "guid")),
                "a shipped battle repeats GUID {}",
                u64_field(actor, "guid")
            );
        }
    }
    let enemies = row["enemies"].as_array().unwrap();
    // The loader refuses more than 16 enemies (`src/encounter/field.rs`), not
    // the 32 the brief assumed; the real load below also proves the shipped five.
    assert!((1..=16).contains(&enemies.len()));
    assert!(i32_field(row, "title_id") > 0);
    assert!(i32_field(row, "description_id") > 0);
    assert!(i32_field(row, "objective_id") > 0);
    assert!(u32_field(row, "reward_coins") <= 1_000_000);
    let prizes = row["prizes"].as_array().unwrap();
    assert!(
        (1..=4).contains(&prizes.len()),
        "one prize is drawn per win"
    );
    assert_eq!(u32_field(&row["zone"], "zone_id"), ARENA_ZONE);

    let inventory = runtime_inventory();
    for prize in prizes {
        let item = i32_field(prize, "item_id");
        assert!(
            inventory.accepts_reward(item, 1),
            "prize {item} cannot be granted"
        );
    }
    let mut engine = EncounterEngine::load(&fixture.content, None).unwrap();
    engine
        .load_field_battles(&battle_path())
        .expect("the real loader accepts the new row");
    assert!(
        engine.battle_prizes_available(SHEEP_WATCH, |item, quantity| {
            inventory.accepts_reward(item, quantity)
        }),
        "T243's whole-bundle preflight accepts the four prizes"
    );

    // A duplicate GUID and a duplicate row are refused at load.
    let mut collision = file.clone();
    collision["battles"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["id"] == json!(SHEEP_WATCH))
        .unwrap()["enemies"][0]["guid"] = json!(MARKER_GUID);
    let collision_path = fixture.root.join("duplicate-guid.json");
    fs::write(&collision_path, serde_json::to_vec(&collision).unwrap()).unwrap();
    let mut engine = EncounterEngine::load(&fixture.content, None).unwrap();
    let error = engine
        .load_field_battles(&collision_path)
        .unwrap_err()
        .to_string();
    assert!(error.contains("duplicate field battle GUID"), "{error}");

    let mut duplicate = file.clone();
    let copied = battle(&duplicate, SHEEP_WATCH).clone();
    duplicate["battles"].as_array_mut().unwrap().push(copied);
    let duplicate_path = fixture.root.join("duplicate-row.json");
    fs::write(&duplicate_path, serde_json::to_vec(&duplicate).unwrap()).unwrap();
    let mut engine = EncounterEngine::load(&fixture.content, None).unwrap();
    let error = engine
        .load_field_battles(&duplicate_path)
        .unwrap_err()
        .to_string();
    assert!(error.contains("invalid field battle"), "{error}");
}

/// 7. Every battle that existed before this slice keeps its title, difficulty,
/// required level, payout, actor GUIDs and positions; the new row is the only
/// addition, after 12303. The canonical dungeon-loader gate
/// (`tests/content_dungeons.rs`) owns the 75-record `ContentDb` proof.
#[test]
fn the_existing_battles_and_content_are_unchanged() {
    let file = battles();
    let expected: [(u32, u32, u32, u32, u32); 8] = [
        (96, 42056, 1, 0, 25),
        (12340, 90703, 3, 5, 25),
        (12301, 75452, 1, 0, 25),
        (12302, 42054, 2, 0, 25),
        (12303, 426971, 5, 13, 0),
        (12310, 93274, 2, 0, 25),
        (12311, 141039, 2, 0, 25),
        (12400, 24276, 1, 0, 19),
    ];
    for (id, title, difficulty, required, coins) in expected {
        let row = battle(&file, id);
        assert_eq!(u32_field(row, "title_id"), title, "battle {id} title");
        assert_eq!(u32_field(row, "difficulty"), difficulty, "battle {id}");
        assert_eq!(
            u32_field(row, "required_combat_level"),
            required,
            "battle {id}"
        );
        assert_eq!(u32_field(row, "reward_coins"), coins, "battle {id}");
    }

    // Actor GUIDs and positions are pinned per shipped row.
    let actors: [(u32, &[(u64, [f32; 3])]); 8] = [
        (
            96,
            &[
                (100000096000, [-440.0, 0.8, 1290.0]),
                (100000096001, [136.0, -0.6, 175.0]),
                (100000096002, [133.0, -0.6, 173.0]),
                (100000096003, [139.0, -0.6, 173.0]),
                (100000096004, [133.0, -0.6, 173.0]),
                (100000096005, [139.0, -0.6, 173.0]),
                (100000096006, [133.0, -0.6, 173.0]),
                (100000096007, [139.0, -0.6, 173.0]),
            ],
        ),
        (
            12340,
            &[
                (100001234000, [-1581.39915, -45.29, -552.22302]),
                (100001234001, [136.0, -0.6, 175.0]),
                (100001234002, [133.0, -0.6, 173.0]),
                (100001234003, [139.0, -0.6, 173.0]),
            ],
        ),
        (
            12301,
            &[
                (100001230100, [-726.0, -7.1, 152.0]),
                (100001230101, [133.0, -0.6, 173.0]),
                (100001230102, [136.0, -0.6, 175.0]),
                (100001230103, [139.0, -0.6, 173.0]),
            ],
        ),
        (
            12302,
            &[
                (100001230200, [-540.0, 0.5, 1010.0]),
                (100001230201, [133.0, -0.6, 173.0]),
                (100001230202, [136.0, -0.6, 175.0]),
                (100001230203, [139.0, -0.6, 173.0]),
            ],
        ),
        (
            12303,
            &[
                (100000004750, [-950.582, -34.42554, -651.9259]),
                (100001230301, [136.0, -0.6, 175.0]),
                (100001230302, [133.0, -0.6, 173.0]),
                (100001230303, [136.0, -0.6, 173.0]),
                (100001230304, [139.0, -0.6, 173.0]),
            ],
        ),
        (
            12310,
            &[
                (100001231000, [364.0, 53.3, 572.0]),
                (100001231001, [133.0, -0.6, 173.0]),
                (100001231002, [136.0, -0.6, 175.0]),
                (100001231003, [139.0, -0.6, 173.0]),
            ],
        ),
        (
            12311,
            &[
                (100001231100, [-888.0, -33.5, -376.0]),
                (100001231101, [133.0, -0.6, 173.0]),
                (100001231102, [136.0, -0.6, 175.0]),
                (100001231103, [139.0, -0.6, 173.0]),
            ],
        ),
        (
            12400,
            &[
                (100000124000, [-1908.92576, -39.4, 410.69792]),
                (100000124001, [136.0, -0.6, 175.0]),
            ],
        ),
    ];
    for (id, actors) in actors {
        let row = battle(&file, id);
        let actual: Vec<(u64, [f32; 3])> = std::iter::once(&row["spawner"])
            .chain(row["enemies"].as_array().unwrap().iter())
            .map(|actor| (u64_field(actor, "guid"), position(&actor["position"])))
            .collect();
        assert_eq!(actual, actors, "battle {id} actors changed");
    }

    let ids: Vec<u32> = file["battles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].as_u64().unwrap() as u32)
        .collect();
    // Later content slices add their own rows (T354 inserts Bixie Hive 12370
    // directly after battle 96), so every pre-existing row must survive
    // exactly once rather than pinning the whole file.
    for (id, _, _, _, _) in expected {
        assert_eq!(
            ids.iter().filter(|candidate| **candidate == id).count(),
            1,
            "battle {id} disappeared or was duplicated"
        );
    }
    let stone_heart = ids
        .iter()
        .position(|id| *id == 12303)
        .expect("Stone Heart is still in the file");
    assert_eq!(
        ids[stone_heart + 1],
        SHEEP_WATCH,
        "Sheep Watch sits after battle 12303"
    );
    assert!(!TASK_ROSTER.contains(&SHEEP_WATCH));

    // The canonical dungeon loader gate is
    // `tests/content_dungeons.rs::every_dungeon_file_loads_and_validates`:
    // it parses all 75 records through `ContentDb::load` and asserts the count.
    // This content-only suite does not duplicate that loader proof.
    let sheep_watch = read_json(&content_dir().join("dungeons/sheep_watch.json"));
    assert_eq!(
        sheep_watch["records"][0]["record"]["name_string_id"],
        1279835357
    );
    assert_eq!(
        sheep_watch["records"][0]["dungeon_data"]["difficulty"]["value"],
        1
    );
}

/// 8. Each arena enemy carries the recovered `enemies.json` kind, and the
/// production engine (the real `load_field_battles` plus the same `strike` the
/// 36/10 handler calls) reports that kind in `EncounterOutcome::defeated`: the
/// input a quest bind matches a goal against. Test 9 proves the credit.
#[test]
fn the_arena_reports_the_recovered_enemy_kinds_on_every_defeat() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, SHEEP_WATCH);
    let expected = [ALPHA_KIND, ALPHA_KIND, WOLF_KIND, WOLF_KIND, WOLF_KIND];
    let enemy_rows = row["enemies"].as_array().unwrap();

    let definitions = read_json(&content_dir().join("enemies.json"));
    let definitions = definitions["definitions"].as_array().unwrap();
    let actual: Vec<i32> = enemy_rows
        .iter()
        .map(|enemy| i32_field(enemy, "definition_id"))
        .collect();
    assert_eq!(
        actual, expected,
        "the five arena actors must carry the recovered enemy kinds, not local ids"
    );
    for (enemy, kind) in enemy_rows.iter().zip(expected) {
        let definition = definitions
            .iter()
            .find(|definition| i32_field(definition, "definition_id") == kind)
            .unwrap_or_else(|| panic!("enemies.json has no definition {kind}"));
        assert_eq!(i32_field(definition, "actor_kind_id"), kind);
        assert_eq!(
            i32_field(definition, "model_id"),
            i32_field(enemy, "model_id"),
            "kind {kind} model"
        );
        assert_eq!(
            i32_field(definition, "name_string_id"),
            i32_field(enemy, "name_id"),
            "kind {kind} display name"
        );
    }
    // The marker is the only actor without a recovered enemy kind.
    assert_eq!(
        i32_field(&row["spawner"], "definition_id"),
        MARKER_DEFINITION
    );

    let mut engine = EncounterEngine::load(&fixture.content, None).unwrap();
    engine
        .load_field_battles(&battle_path())
        .expect("the production loader accepts the row");
    let location = CharacterState {
        guid: 8001,
        zone_id: 0,
        position: MARKER,
        rotation: [0.0, 0.0, 1.0],
    };
    engine.begin(SHEEP_WATCH, location, BRAWLER).unwrap();
    engine.on_zone_ready(BRAWLER, 0).unwrap();
    let mut defeated = Vec::new();
    let mut now = 0;
    for enemy in enemy_rows {
        let guid = u64_field(enemy, "guid");
        let at = position(&enemy["position"]);
        let kills = defeated.len();
        for _ in 0..8 {
            now += 700;
            let outcome = engine.attack(guid, at, BRAWLER, 0, now).unwrap();
            defeated.extend(outcome.defeated);
            if defeated.len() > kills {
                break;
            }
        }
    }
    assert_eq!(
        defeated, expected,
        "every defeat event names the recovered enemy kind"
    );
}

/// 9. The shipped `Brawler: Delinquent Dilemma` goal (`data/content/quests/brawler.json:405`,
/// quest 3030007) counts four defeats of the recovered Alpha Wolf kind
/// `930050`. Delinquent Dilemma is offered and accepted through the real
/// NPC route (26/8 -> 26/10 Talk -> 26/6 row -> 49/1 -> 49/2); only its
/// completed prerequisite is a reconnect-style save seam. After a
/// `game_store` reconnect at the marker, the real marker -> Go -> 36/10 route
/// advances its two Sheep Watch Alphas 1/4 then 2/4, and the three Wolves
/// (`930071`) do not touch that goal. With the local `1235001`-`1235005` ids
/// the counter would stay at 0.
#[test]
fn an_arena_defeat_credits_a_goal_keyed_to_the_enemy_kind() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, SHEEP_WATCH);

    // Seed only the previous chapter step, then acquire the target quest live.
    let mut gateway =
        fixture.gateway_with_quest(jones_position(), 8101, delinquent_prerequisite(8101));
    assert_eq!(active_profile(&gateway), BRAWLER);
    assert_eq!(
        quest_status(&gateway, DELINQUENT_PREREQUISITE),
        QuestStatus::Completed
    );
    assert!(
        !gateway
            .snapshot()
            .gameplay
            .unwrap()
            .npc_quests
            .unwrap()
            .quests
            .contains_key(&DELINQUENT_QUEST)
    );

    let menu = send(&mut gateway, &interact(JONES), 700);
    let choice = menu_option(&menu, TALK_LABEL).expect("Jones offers a Talk option");
    let dialogue = send(&mut gateway, &npc_select(JONES, choice), 800);
    let dialogue_packet = dialogue
        .replies
        .iter()
        .map(|packet| unframe(packet))
        .find(|packet| packet.starts_with(&[26, 0, 3, 0]))
        .expect("Talk opens the 26/3 dialogue");
    let decoded =
        decode_dialogue_26_3(dialogue_packet).expect("the production 26/3 reader parses it");
    assert!(
        decoded
            .choices
            .iter()
            .any(|row| row[0] == QUEST_CHOICE_RESPONSE as u32),
        "the dialogue carries the Delinquent Dilemma quest row"
    );
    let offered = send(&mut gateway, &dialogue_choice(QUEST_CHOICE_RESPONSE), 900);
    let offer_packet = offered
        .replies
        .iter()
        .map(|packet| unframe(packet))
        .find(|packet| packet.starts_with(&[49, 0, 1, 0]))
        .expect("the quest row opens one 49/1 offer");
    let offer =
        decode_quest_offer_49_1(offer_packet).expect("the production 49/1 reader parses it");
    assert_eq!(offer.quest_id, DELINQUENT_QUEST);
    assert_eq!(
        quest_status(&gateway, DELINQUENT_QUEST),
        QuestStatus::Offered
    );
    send(&mut gateway, &quest_reply(DELINQUENT_QUEST, true), 1_000);
    assert_eq!(
        quest_status(&gateway, DELINQUENT_QUEST),
        QuestStatus::Active
    );
    assert_eq!(saved_quest_count(&gateway, DELINQUENT_QUEST), 0);

    // Reconnect through game_store at the marker; the accepted goal survives
    // the catalogue bind.
    let mut gateway = reopen_at(&fixture, &gateway, "accepted-quest.json", MARKER, 1_100);
    assert_eq!(
        quest_status(&gateway, DELINQUENT_QUEST),
        QuestStatus::Active
    );
    assert_eq!(saved_quest_count(&gateway, DELINQUENT_QUEST), 0);

    let screen = open_battle(&mut gateway, MARKER_GUID, row, 1_200);
    enter_battle(&mut gateway, &screen, row, 1_300);
    let mut now = 1_400;
    let replies = fight_to_win(&mut gateway, &mut now, row);
    assert_eq!(results_lifecycle(&replies), vec![18, 47, 19]);

    // Two Alpha Wolf defeats match the goal; the three Wolves are another
    // kind, so the goal stops at 2/4.
    assert_eq!(saved_quest_count(&gateway, DELINQUENT_QUEST), 2);
    assert_eq!(
        objective_updates(&replies, DELINQUENT_QUEST),
        vec![1, 2],
        "49/9 ObjectiveUpdate reports 1/4 then 2/4 (reader 0x00c7af60)"
    );
}
