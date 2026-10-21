//! T282: Thugamug's bridge as a level-5 field battle (Thugamug and his two Hands).
//! T330: "Chapter 1: Thugamug's Thievery" (quest 3000032) is played against that
//! battle end to end. The T330 session harness and client-reader notes are in the
//! section at the end of this file.
//!
//! Content-only slice. The battle is one new `field_battles.json` row and the
//! existing route: activity-portal Play (167/1/2) or marker 26/8 -> 41/114
//! start screen -> 41/108 Go (client constructor 0x008b6e70, sender
//! 0x00938f00) -> arena zone 9600 -> 26/42 Close. The 41/114 reader chain is
//! 0x00aa3430 -> 0x00aa32d0 -> 0x00a29120 -> 0x009bdd70; the marker actor is
//! 35/2 AddNpc (constructor 0x0091cac0, reader 0x00921a10); the coin balance
//! is 38/19 (0x0089e670). No new wire field, handler, opcode or engine code is
//! added, so the byte layouts stay the ones `field_battle_roster` pins.
//!
//! Placement evidence: the `9500000322` Thugamug row is the Greenwood bridge
//! poi_anchor at [-1580.39915,-45.37643,-555.223024]. Its full 5 m wander disc
//! reaches 40.06 degrees against this project's 25 degree authoring rule, so
//! the marker uses the nearest 1 m-grid safe point 3.16 m away,
//! [-1581.39915,-45.29,-552.22302], whose 105-sample disc is terrain-covered
//! and at most 24.81 degrees (see the battle's source rows).
//!
//! Thresholds (R, the same conservative authoring rules as T103): centre
//! height within 0.05 m of the bilinear terrain height, 24 ring samples all
//! finite, the 105-sample full disc all finite and at most 25 degrees, and at
//! least 25 m to the nearest spawnable NPC, field enemy and other battle
//! marker. Only `legacy_data` NPC rows spawn (`world.rs` skips POI/zone-centre
//! scaffolding), so the nearest *spawned* NPC is measured, not the Thugamug
//! poi_anchor itself.

use osfr_tools::{
    account_store::AccountRole,
    activity_portal::{Catalogue, Kind, Refusal, Viewer},
    chat,
    encounter::{EncounterEngine, SharedFieldState},
    gameplay::GameplayState,
    gateway::{Gateway, GatewayOutcome},
    inventory::{InventoryState, MAX_ITEMS},
    quest_engine::{CharacterEligibility, QuestEngine, QuestStatus},
    startup_world, world,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};

#[path = "common/field_battle_fixture.rs"]
mod field_battle_fixture;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/terrain.rs"]
mod terrain;

use field_battle_fixture::*;
use npc_quest_chain::{gateway_active_job as active_job, gateway_profile_xp as profile_xp};

/// The new battle. 12340 was free on this base; the row is found by this id
/// and test 1 also pins its published values.
const THUGAMUG: u32 = 12340;
const TITLE: u32 = 90703;
const DESCRIPTION: u32 = 90704;
const OBJECTIVE: u32 = 90715;
const DIFFICULTY: u32 = 3;
const REQUIRED_LEVEL: u32 = 5;
const REWARD_COINS: u32 = 25;
const MARKER_GUID: u64 = 100001234000;
const ENEMY_GUIDS: [u64; 3] = [100001234001, 100001234002, 100001234003];
const MARKER: [f32; 3] = [-1581.39915, -45.29, -552.22302];
const ORIGINAL_ANCHOR: [f32; 3] = [-1580.39915, -45.37643, -555.223024];
const NAME_THUGAMUG: i32 = 103031;
const NAME_HENCHMAN: i32 = 103726;
const MODEL: i32 = 199;
const BRAWLER: i32 = 43;
const REFUSAL_PRIZE: &str = "one of its rewards cannot be granted";
const BATTLE_JOB_XP: u32 = 50;

// ---------------------------------------------------------------------------
// Fixture seam: the T282 start screen needs a level-5 combat job in the bag
// seed, and the full bag is built around this battle's prize pool.
// ---------------------------------------------------------------------------

impl Fixture {
    fn gateway_with_bag(
        &self,
        position: [f32; 3],
        guid: u64,
        capacity: usize,
    ) -> (Gateway, InventoryState) {
        self.gateway_with_bag_seeded(position, guid, capacity, THUGAMUG, |seed| {
            assert!(send(seed, &switch_job(BRAWLER), 300).state_changed);
            send(seed, &chat::local_request("realm level 5"), 350);
            assert_eq!(level(seed, BRAWLER), REQUIRED_LEVEL);
        })
    }
}

// ---------------------------------------------------------------------------
// Economy and fight helpers.
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
struct Economy {
    coins: u32,
    prizes: BTreeMap<i32, u32>,
    experience: u32,
}

fn economy(gateway: &Gateway, prize_ids: &BTreeSet<i32>) -> Economy {
    let state = gateway.snapshot().gameplay.unwrap();
    Economy {
        coins: state.inventory.coins,
        prizes: prize_ids
            .iter()
            .map(|id| {
                let count = state
                    .inventory
                    .items
                    .iter()
                    .filter(|item| item.definition_id == *id)
                    .map(|item| item.count)
                    .sum();
                (*id, count)
            })
            .collect(),
        experience: state
            .profiles
            .unwrap()
            .experience
            .get(&BRAWLER)
            .copied()
            .unwrap_or(0),
    }
}

fn active_enemy(gateway: &Gateway) -> Option<(u64, [f32; 3], u32)> {
    let state = gateway.snapshot().gameplay?.encounter?;
    let active = state.active?;
    let enemy = active.enemies.iter().find(|enemy| enemy.health > 0)?;
    Some((enemy.guid, enemy.position, enemy.health))
}

fn open_battle(gateway: &mut Gateway, spawner_guid: u64, row: &Value, now_ms: u64) -> Vec<u8> {
    let screen = send(gateway, &interact(spawner_guid), now_ms);
    let details = details(&screen).expect("clicking the marker opens 41/114");
    assert_start_screen(details, row);
    details.to_vec()
}

/// The 41/114 start screen fields, pinned byte-for-byte against the client
/// reader chain 0x00aa3430 -> 0x00aa32d0 -> 0x00a29120 -> 0x009bdd70.
fn assert_start_screen(details: &[u8], row: &Value) {
    assert_eq!(i32_at(details, 4), THUGAMUG as i32);
    assert!(i32_at(details, 8) > 0, "the start screen has an instance");

    // 41/114 reader chain 0x00aa3430 -> 0x00aa32d0 -> 0x00a29120 ->
    // 0x009bdd70: title, icon, description, difficulty, profile type,
    // MiniGameTypes.
    let expected: Vec<u8> = [
        i32_field(row, "title_id"),
        i32_field(row, "icon_id"),
        i32_field(row, "description_id"),
        i32_field(row, "difficulty"),
        2, // MiniGameInfo.ProfileType: combat
        4, // MiniGameTypes.COMBAT_TYPE
    ]
    .into_iter()
    .flat_map(i32::to_le_bytes)
    .collect();
    assert_eq!(
        &details[47..71],
        expected.as_slice(),
        "41/114 fields must agree byte-for-byte with the client reader"
    );
}

fn enter_battle(gateway: &mut Gateway, details: &[u8], row: &Value, now_ms: u64) -> Vec<u64> {
    let entered = send(gateway, &go(details), now_ms);
    assert!(entered.began_zoning, "Go enters the arena");
    assert_eq!(gateway.state().zone_id, 9600);

    let enemy_guids: Vec<u64> = row["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .map(|enemy| u64_field(enemy, "guid"))
        .collect();
    let ready = arena_spawned(gateway, now_ms + 100);
    for guid in &enemy_guids {
        assert!(
            ready
                .replies
                .iter()
                .map(|packet| unframe(packet))
                .any(|packet| packet.starts_with(&[35, 0, 2, 0])
                    && packet[4..12] == guid.to_le_bytes()),
            "arena enemy {guid} spawned"
        );
    }
    assert!(
        ready
            .replies
            .iter()
            .map(|packet| unframe(packet))
            .any(|packet| packet
                .windows(4)
                .any(|field| field == i32_field(row, "objective_id").to_le_bytes())),
        "the recovered goal reaches the client"
    );
    assert!(
        active_enemy(gateway).is_some(),
        "the arena starts with a fightable enemy"
    );
    enemy_guids
}

fn fight_to_win(gateway: &mut Gateway, now_ms: &mut u64) {
    let player_guid = gateway.state().guid;
    for _ in 0..200 {
        let Some((guid, position, _)) = active_enemy(gateway) else {
            break;
        };
        let phase = gateway
            .snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active
            .unwrap()
            .phase;
        if phase != osfr_tools::encounter::EncounterPhase::Fighting {
            break;
        }
        walk(
            gateway,
            player_guid,
            [position[0] + 1.5, position[1], position[2]],
            now_ms,
        );
        *now_ms += 700;
        // 36/10 writer 0x00a342e0: bar 1, slot 0, target mode 0, target GUID.
        send(gateway, &melee(guid), *now_ms);
        walk(gateway, player_guid, [136.0, -0.5, 155.0], now_ms);
        *now_ms += 100;
        gateway.tick_at(*now_ms).unwrap();
    }
    let phase = gateway
        .snapshot()
        .gameplay
        .unwrap()
        .encounter
        .unwrap()
        .active
        .map(|active| active.phase);
    assert_eq!(
        phase,
        Some(osfr_tools::encounter::EncounterPhase::Won),
        "the three actors die through the real attack path"
    );
}

fn assert_payout(before: &Economy, after: &Economy, row: &Value) {
    let prizes = prize_ids(row);
    assert_eq!(after.coins, before.coins + u32_field(row, "reward_coins"));
    assert_eq!(after.experience, before.experience + BATTLE_JOB_XP);
    let gains: Vec<(i32, u32)> = after
        .prizes
        .iter()
        .filter_map(|(id, count)| {
            let prior = before.prizes.get(id).copied().unwrap_or(0);
            (count > &prior).then_some((*id, count - prior))
        })
        .collect();
    assert_eq!(gains.len(), 1, "exactly one prize was granted");
    assert!(prizes.contains(&gains[0].0));
    assert_eq!(gains[0].1, 1);
}

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

/// 1. The real loader accepts the file, the portal publishes the row with the
/// verified ids, and every actor id/name/model is well formed.
#[test]
fn the_battle_loads_and_is_published_with_the_expected_values() {
    let content = manifest_dir().join(RUNTIME_CONTENT);
    let path = battle_path();
    let mut engine = EncounterEngine::load(&content, None).unwrap();
    engine
        .load_field_battles(&path)
        .expect("the real field-battle loader accepts the file");
    let spawners = engine.battle_spawners();
    let marker = spawners
        .iter()
        .find(|entry| entry.0 == MARKER_GUID)
        .expect("the Thugamug marker is published to the world");
    assert_eq!(marker.1, "Thugamug");
    assert_eq!(marker.2, NAME_THUGAMUG);
    assert_eq!(marker.3, MODEL);
    assert_eq!(marker.4, MARKER);
    assert_eq!(marker.5, 11970, "the crossed-swords nameplate is visible");

    let file = battles();
    let row = battle(&file, THUGAMUG);
    assert_eq!(u32_field(row, "title_id"), TITLE);
    assert_eq!(u32_field(row, "description_id"), DESCRIPTION);
    assert_eq!(u32_field(row, "objective_id"), OBJECTIVE);
    assert_eq!(u32_field(row, "difficulty"), DIFFICULTY);
    assert_eq!(u32_field(row, "required_combat_level"), REQUIRED_LEVEL);
    assert_eq!(u32_field(row, "reward_coins"), REWARD_COINS);
    assert_eq!(i32_field(row, "icon_id"), -1);
    assert_eq!(row["spawner"]["boss"], false);
    assert_eq!(u32_field(&row["spawner"], "max_health"), 1);

    // The roster: one boss and two Hands, all model 199, with distinct GUIDs
    // and battle-local definition ids. The guide gives no gender evidence for
    // the pair, so the single recovered male bandit body is used for both (R).
    let enemies = row["enemies"].as_array().unwrap();
    assert_eq!(enemies.len(), 3);
    assert_eq!(u64_field(&enemies[0], "guid"), ENEMY_GUIDS[0]);
    assert_eq!(enemies[0]["boss"], true);
    assert_eq!(u32_field(&enemies[0], "max_health"), 220);
    assert_eq!(u32_field(&enemies[0], "damage"), 12);
    for hand in &enemies[1..] {
        assert_eq!(hand["boss"], false);
        assert_eq!(
            u32_field(hand, "name_id"),
            u32::try_from(NAME_HENCHMAN).unwrap()
        );
        assert_eq!(u32_field(hand, "model_id"), u32::try_from(MODEL).unwrap());
        assert_eq!(u32_field(hand, "max_health"), 140);
        assert_eq!(u32_field(hand, "damage"), 9);
    }
    let guids: BTreeSet<u64> = std::iter::once(&row["spawner"])
        .chain(enemies.iter())
        .map(|actor| u64_field(actor, "guid"))
        .collect();
    let definitions: BTreeSet<i32> = std::iter::once(&row["spawner"])
        .chain(enemies.iter())
        .map(|actor| i32_field(actor, "definition_id"))
        .collect();
    assert_eq!(guids.len(), 4, "marker and three enemies have unique GUIDs");
    assert_eq!(
        definitions.len(),
        4,
        "each actor has its own battle-local definition id"
    );
    assert!(
        guids.contains(&MARKER_GUID),
        "the marker GUID is one of the new ids"
    );

    // Names resolve in the client locale report; models are drawable.
    let locale: Value = read_json(&lab("reports/locale-id-to-text.json"));
    let names: BTreeMap<i32, String> = locale
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
        names.get(&NAME_THUGAMUG).map(String::as_str),
        Some("Thugamug")
    );
    assert_eq!(
        names.get(&NAME_HENCHMAN).map(String::as_str),
        Some("Thugamug Henchman")
    );
    let models =
        fs::read_to_string(lab("working/OSFR.Client/OSFR Client/Resources/Models.txt")).unwrap();
    assert!(
        models
            .lines()
            .any(|line| line.starts_with("199^chugawug_m_bandit_01.adr^")),
        "model 199 is in Models.txt"
    );
    let catalog = fs::read_to_string(lab("extracted/client-assets/catalog.tsv")).unwrap();
    assert!(
        catalog.contains("chugawug_m_bandit_01.adr"),
        "model 199 is present in catalog.tsv"
    );

    // The activity portal publishes the row with the same ids and difficulty.
    let catalogue = Catalogue::load(&path).unwrap();
    let activity = catalogue
        .activities()
        .iter()
        .find(|activity| activity.id == THUGAMUG)
        .expect("the portal publishes the new battle");
    assert_eq!(activity.name_id, TITLE);
    assert_eq!(activity.description_id, DESCRIPTION);
    assert_eq!(activity.difficulty, DIFFICULTY);
    assert_eq!(activity.required_level, REQUIRED_LEVEL);
    assert_eq!(activity.kind, Kind::Battle);
    assert_eq!(
        catalogue
            .admit(
                THUGAMUG,
                Viewer {
                    combat_job: true,
                    level: REQUIRED_LEVEL
                },
                false
            )
            .unwrap()
            .id,
        THUGAMUG
    );
    assert_eq!(
        catalogue.admit(
            THUGAMUG,
            Viewer {
                combat_job: true,
                level: REQUIRED_LEVEL - 1
            },
            false
        ),
        Err(Refusal::LevelTooLow(REQUIRED_LEVEL))
    );
    assert_eq!(
        catalogue.admit(
            THUGAMUG,
            Viewer {
                combat_job: false,
                level: 20
            },
            false
        ),
        Err(Refusal::NotCombatJob)
    );

    // The 167/1/1 row the client reads carries the same values.
    let rows = decode_list(&catalogue.list_packet(Viewer {
        combat_job: true,
        level: REQUIRED_LEVEL,
    }));
    let published = rows.iter().find(|row| row.id == THUGAMUG).unwrap();
    assert_eq!(published.title, TITLE);
    assert_eq!(published.description, DESCRIPTION);
    assert_eq!(published.difficulty, DIFFICULTY);
    assert!(published.can_join);
    let refused = decode_list(&catalogue.list_packet(Viewer {
        combat_job: true,
        level: REQUIRED_LEVEL - 1,
    }));
    assert!(
        !refused
            .iter()
            .find(|row| row.id == THUGAMUG)
            .unwrap()
            .can_join
    );
}

/// 2. The chosen marker is on the terrain, its full 5 m wander disc is
/// standable under the project's authoring rule, and it is clear of the
/// nearest spawned NPC, field enemy and other battle marker. The arena
/// arrival is inside zone 9600.
#[test]
fn the_marker_is_on_the_ground_and_clear() {
    let file = battles();
    let row = battle(&file, THUGAMUG);
    let marker = position(&row["spawner"]["position"]);
    assert!(
        (marker[0] - ORIGINAL_ANCHOR[0]).hypot(marker[2] - ORIGINAL_ANCHOR[2]) <= 5.0,
        "the marker stays on the bridge landmark"
    );

    let zone = &row["zone"];
    assert_eq!(u32_field(zone, "zone_id"), 9600);
    assert_eq!(zone["name"], "sg_random_encounter_clearing");
    assert_eq!(u32_field(zone, "geometry_id"), 214);
    assert_eq!(zone["sky"], "sky_wilds24.xml");
    let arrival = position(&zone["position"]);
    assert_eq!(arrival, [136.0, -0.5, 155.0]);

    let terrain = terrain::load();
    let grid = terrain.grid(0).expect("zone 0 is loaded");
    let height = grid
        .height_at(marker[0], marker[2])
        .expect("the marker is on covered ground");
    assert!(
        (marker[1] - height).abs() <= 0.05,
        "marker y {} is {:.4} m from the sampled ground {height}",
        marker[1],
        (marker[1] - height).abs()
    );

    // The 24-point 5 m ring, then all 105 disc samples.
    for angle in (0..360).step_by(15) {
        let radians = (angle as f32).to_radians();
        let (x, z) = (
            marker[0] + 5.0 * radians.cos(),
            marker[2] + 5.0 * radians.sin(),
        );
        let sample = grid
            .height_at(x, z)
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

    // The bridge poi_anchor itself is why the marker moved: its disc is not
    // standable under the same rule.
    let bridge_slope = terrain::disc_points(ORIGINAL_ANCHOR[0], ORIGINAL_ANCHOR[2])
        .iter()
        .map(|(x, z)| terrain::slope_degrees(grid, *x, *z))
        .fold(0.0_f32, f32::max);
    assert!(
        bridge_slope > 25.0,
        "the poi_anchor disc should need the recorded 3.16 m adaptation"
    );

    // Thresholds: 25 m to a spawnable NPC, field enemy and other battle
    // marker (T103's conservative authoring gate).
    let (npc, npc_id) = nearest_spawned_npc(marker);
    assert!(
        npc >= 25.0,
        "nearest spawned NPC {npc_id} is {npc:.1} m away"
    );
    let enemy = nearest_field_enemy(marker);
    assert!(enemy >= 25.0, "nearest field enemy is {enemy:.1} m away");
    let other = nearest_other_battle_marker(marker, THUGAMUG);
    assert!(other >= 25.0, "nearest battle marker is {other:.1} m away");

    let arena = terrain.grid(9600).expect("zone 9600 is loaded");
    assert!(
        arena.height_at(arrival[0], arrival[2]).is_some(),
        "the arena arrival is inside the zone-9600 grid"
    );
}

/// 3. A level-5 combat job opens the 41/114 screen, enters through Go, kills
/// the three actors through the real attack path, and is paid 25 coins, one
/// prize and the active job's 50 XP exactly once.
#[test]
fn a_level_five_character_can_start_and_win_it_once() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, THUGAMUG);
    let marker = position(&row["spawner"]["position"]);
    let prizes = prize_ids(row);
    let mut gateway = fixture.gateway(marker, 1001);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &chat::local_request("realm level 5"), 350);
    assert_eq!(level(&gateway, BRAWLER), REQUIRED_LEVEL);

    let before = economy(&gateway, &prizes);
    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    let enemy_guids = enter_battle(&mut gateway, &screen, row, 1_000);
    let mut now = 2_000;
    fight_to_win(&mut gateway, &mut now);

    let paid = economy(&gateway, &prizes);
    assert_payout(&before, &paid, row);
    let won = gateway.snapshot();
    let replay = send(&mut gateway, &melee(enemy_guids[0]), now + 700);
    assert!(!replay.state_changed, "a won fight cannot pay twice");
    assert_eq!(gateway.snapshot(), won);

    let closed = send(&mut gateway, &[26, 0, 42, 0], now + 1_400);
    assert!(closed.began_zoning, "Close returns to the open world");
    assert_eq!(gateway.state().zone_id, 0);
    assert_eq!(gateway.state().position, marker);
    assert_eq!(
        economy(&gateway, &prizes),
        paid,
        "leaving pays nothing more"
    );
}

/// 4. A level-4 character and a non-combat job are refused by the portal with
/// the existing message, and the refusal changes no saved state.
#[test]
fn a_level_four_or_non_combat_job_is_refused_and_changes_nothing() {
    let fixture = Fixture::new();
    let mut gateway = fixture.gateway(MARKER, 1002);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &chat::local_request("realm level 4"), 350);
    assert_eq!(level(&gateway, BRAWLER), REQUIRED_LEVEL - 1);

    let before = saved_bytes(&gateway);
    let refused = send(&mut gateway, &portal_play(THUGAMUG), 600);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(
        chat_text(&refused).contains("This activity needs level 5."),
        "{}",
        chat_text(&refused)
    );
    assert_eq!(
        saved_bytes(&gateway),
        before,
        "a level refusal changes no encounter, inventory, coins or save"
    );
    assert!(
        gateway
            .snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active
            .is_none()
    );

    // A fresh session stays on the starting Adventurer profile, which is not
    // a combat job. The earlier session cannot switch back to it because the
    // Adventurer outfit lacks the required Chest/Legs slots, so this gate is
    // driven from the real default job.
    let mut civilian = fixture.gateway(MARKER, 1003);
    let before = saved_bytes(&civilian);
    let refused = send(&mut civilian, &portal_play(THUGAMUG), 700);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(
        chat_text(&refused).contains("Switch to a combat job before starting a battle."),
        "{}",
        chat_text(&refused)
    );
    assert_eq!(
        saved_bytes(&civilian),
        before,
        "a job refusal changes no state"
    );
}

/// 5. T243's whole-bundle preflight: a full bag with none of the prize
/// definitions refuses without changing anything; a bag with four free stacks
/// starts the same battle.
#[test]
fn the_battle_prize_preflight_and_full_bag_refuse() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, THUGAMUG);
    let marker = position(&row["spawner"]["position"]);
    let prizes = prize_ids(row);

    // Full bag: the four uncapped prize rows each need a new stack.
    let (mut full, bag) = fixture.gateway_with_bag(marker, 1003, MAX_ITEMS);
    assert_eq!(bag.items.len(), MAX_ITEMS);
    assert!(
        !bag.items
            .iter()
            .any(|item| prizes.contains(&item.definition_id)),
        "no prize definition is owned, so each needs a new stack"
    );
    let before = saved_bytes(&full);
    let refused = send(&mut full, &portal_play(THUGAMUG), 600);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(
        chat_text(&refused).contains(REFUSAL_PRIZE),
        "{}",
        chat_text(&refused)
    );
    assert_eq!(
        saved_bytes(&full),
        before,
        "a refused prize bundle changes nothing"
    );
    assert!(
        full.snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active
            .is_none()
    );

    // Four free stacks: the whole previewed bundle fits and the battle starts.
    let (mut fitting, bag) = fixture.gateway_with_bag(marker, 1004, MAX_ITEMS - 4);
    assert_eq!(bag.items.len(), MAX_ITEMS - 4);
    assert!(
        !bag.items
            .iter()
            .any(|item| prizes.contains(&item.definition_id)),
        "the fitting bag also starts without the prizes"
    );
    let started = send(&mut fitting, &portal_play(THUGAMUG), 900);
    assert!(started.began_zoning, "the fitting bundle starts the battle");
    assert_eq!(fitting.state().zone_id, 9600);
}

/// 6. Finish, close, save/reload and replay the result/win: the single payout
/// survives the reconnect and is never paid again by the old win.
#[test]
fn reconnect_and_duplicate_win_do_not_pay_twice() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, THUGAMUG);
    let marker = position(&row["spawner"]["position"]);
    let prizes = prize_ids(row);
    let mut gateway = fixture.gateway(marker, 1005);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &chat::local_request("realm level 5"), 350);

    let before = economy(&gateway, &prizes);
    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    let enemy_guids = enter_battle(&mut gateway, &screen, row, 1_000);
    arena_spawned(&mut gateway, 1_500);
    let won = send(&mut gateway, &chat::local_request("realm win"), 2_000);
    assert!(won.state_changed);
    let paid = economy(&gateway, &prizes);
    assert_payout(&before, &paid, row);

    // Replaying the win on the won run pays nothing before the close.
    let won_state = gateway.snapshot();
    let duplicate_win = send(&mut gateway, &chat::local_request("realm win"), 2_100);
    assert!(!duplicate_win.state_changed);
    assert_eq!(gateway.snapshot(), won_state);

    let closed = send(&mut gateway, &[26, 0, 42, 0], 2_200);
    assert!(closed.began_zoning);
    assert_eq!(gateway.state().zone_id, 0);
    assert_eq!(gateway.state().position, marker);
    assert_eq!(economy(&gateway, &prizes), paid);
    let after_close = gateway.snapshot();
    let duplicate_close = send(&mut gateway, &[26, 0, 42, 0], 2_300);
    assert!(!duplicate_close.began_zoning && !duplicate_close.state_changed);
    assert_eq!(
        gateway.snapshot(),
        after_close,
        "a duplicate results Close changes nothing"
    );

    // Reconnect: the paid economy and the marker return survive; the old
    // won run is gone and replaying its result pays nothing.
    let mut connected = reopen(&fixture, &gateway, "won.json", 3_000);
    assert_eq!(economy(&connected, &prizes), paid);
    assert_eq!(connected.state().zone_id, 0);
    assert_eq!(connected.state().position, marker);
    assert!(
        connected
            .snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active
            .is_none()
    );
    let reconnect_before = saved_bytes(&connected);
    let old_hit = send(&mut connected, &melee(enemy_guids[0]), 3_100);
    assert!(!old_hit.state_changed, "the old enemy GUID pays nothing");
    assert_eq!(saved_bytes(&connected), reconnect_before);
    let old_close = send(&mut connected, &[26, 0, 42, 0], 3_100);
    assert!(!old_close.state_changed, "the replayed Close pays nothing");
    assert_eq!(saved_bytes(&connected), reconnect_before);
    let duplicate = send(&mut connected, &chat::local_request("realm win"), 3_200);
    assert!(!duplicate.state_changed, "an orphaned win pays nothing");
    assert_eq!(economy(&connected, &prizes), paid);
}

/// 7. Every battle that existed before this content slice keeps its title,
/// difficulty, required level and coin payout; only the new row is added.
#[test]
fn the_pre_existing_battles_stay_unchanged() {
    let file = battles();
    // The pre-change roster on this base: T252's 12400 is already the seventh
    // row, so seven battles predate T282. Values are the loader-consumed ones
    // the existing tests pin; a balance edit to any row fails here.
    let pre_existing: [(u32, u32, u32, u32, u32); 7] = [
        (96, 42056, 1, 0, 25),
        (12301, 75452, 1, 0, 25),
        (12302, 42054, 2, 0, 25),
        (12303, 426971, 5, 13, 0),
        (12310, 93274, 2, 0, 25),
        (12311, 141039, 2, 0, 25),
        (12400, 24276, 1, 0, 19),
    ];
    for (id, title, difficulty, required, coins) in pre_existing {
        let row = battle(&file, id);
        assert_eq!(u32_field(row, "title_id"), title, "battle {id} title");
        assert_eq!(
            u32_field(row, "difficulty"),
            difficulty,
            "battle {id} difficulty"
        );
        assert_eq!(
            u32_field(row, "required_combat_level"),
            required,
            "battle {id} required level"
        );
        assert_eq!(
            u32_field(row, "reward_coins"),
            coins,
            "battle {id} coin payout"
        );
    }
    let ids: Vec<u32> = file["battles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].as_u64().unwrap() as u32)
        .collect();
    // The insertion anchor is at the head of the file, after battle 96, not at
    // the end. T354's Bixie Hive (12370) now occupies index 1, so assert the
    // T282 row still precedes every other pre-existing battle rather than
    // pinning its exact index.
    assert_eq!(ids[0], 96);
    let thugamug_at = ids
        .iter()
        .position(|id| *id == THUGAMUG)
        .expect("the Thugamug row survived");
    let first_other = ids
        .iter()
        .position(|id| *id == 12301)
        .expect("battle 12301 survived");
    assert!(
        thugamug_at > 0 && thugamug_at < first_other,
        "Thugamug left its head anchor (index {thugamug_at}, 12301 at {first_other})"
    );
    // Later content slices add their own rows (T319 adds 12350, T354 adds
    // 12370), so this test requires every pre-existing row to survive in its
    // exact relative order rather than re-pinning the whole file. The known
    // order is pinned by `tests/field_battle_roster.rs::
    // five_distinct_field_battles_load_with_the_existing_schema` and by
    // `sheep_watch_dungeon` test 7; a silent deletion or reorder fails here too.
    let existing: BTreeSet<u32> = pre_existing.iter().map(|row| row.0).collect();
    let actual: BTreeSet<u32> = ids.iter().copied().collect();
    assert!(
        existing.is_subset(&actual),
        "a pre-existing battle was removed"
    );
    let positions: Vec<usize> = pre_existing
        .iter()
        .map(|(id, ..)| {
            ids.iter()
                .position(|row| row == id)
                .unwrap_or_else(|| panic!("battle {id} was removed"))
        })
        .collect();
    assert!(
        positions.windows(2).all(|pair| pair[0] < pair[1]),
        "the T282-era battles were reordered: {positions:?}"
    );
    assert_eq!(
        actual.iter().filter(|id| **id == THUGAMUG).count(),
        1,
        "the Thugamug row is present exactly once"
    );
    assert_eq!(
        file["battles"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["title_id"] == json!(TITLE))
            .count(),
        1,
        "the new title is unique"
    );
}

/// 8. Two sessions run their own arena instance: one player's kill and payout
/// never credit or pay the other, and a malformed request from one leaves the
/// other's battle running.
#[test]
fn a_second_player_can_run_its_own_instance() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, THUGAMUG);
    let marker = position(&row["spawner"]["position"]);
    let prizes = prize_ids(row);

    let mut first = fixture.gateway(marker, 2001);
    let mut second = fixture.gateway(marker, 2002);
    for gateway in [&mut first, &mut second] {
        assert!(send(gateway, &switch_job(BRAWLER), 300).state_changed);
        send(gateway, &chat::local_request("realm level 5"), 350);
    }
    let first_screen = open_battle(&mut first, MARKER_GUID, row, 500);
    let second_screen = open_battle(&mut second, MARKER_GUID, row, 500);
    enter_battle(&mut first, &first_screen, row, 1_000);
    enter_battle(&mut second, &second_screen, row, 1_000);

    let first_before = economy(&first, &prizes);
    let second_before = economy(&second, &prizes);
    let (target, target_position, target_health) = active_enemy(&first).unwrap();
    assert_eq!(target, ENEMY_GUIDS[0]);
    let second_target = active_enemy(&second).unwrap();
    assert_eq!(second_target.2, target_health, "both instances start equal");

    // One real hit on the first player's boss. The second player's copy of the
    // same GUID stays at full health and is still fightable.
    let player_guid = first.state().guid;
    let mut now = 2_000;
    walk(
        &mut first,
        player_guid,
        [
            target_position[0] + 1.5,
            target_position[1],
            target_position[2],
        ],
        &mut now,
    );
    now += 700;
    send(&mut first, &melee(target), now);
    first.tick_at(now + 100).unwrap();
    assert!(
        active_enemy(&first)
            .is_none_or(|(guid, _, health)| { guid != target || health < target_health }),
        "the first player's hit lands on its own boss"
    );
    assert_eq!(
        active_enemy(&second).unwrap().2,
        target_health,
        "the first player's hit never damages the second player's boss"
    );
    assert_eq!(economy(&second, &prizes), second_before);

    // The first player's payout is its own.
    let won = send(&mut first, &chat::local_request("realm win"), now + 800);
    assert!(won.state_changed);
    let first_paid = economy(&first, &prizes);
    assert_payout(&first_before, &first_paid, row);
    assert_eq!(
        economy(&second, &prizes),
        second_before,
        "the first player's win never pays the second"
    );

    // A malformed Play on the won first session is its own no-op, and the
    // second player's battle still runs and pays.
    let malformed = send(&mut first, &[167, 0, 1, 2], now + 900);
    assert!(!malformed.began_zoning && !malformed.state_changed);
    assert!(
        malformed.replies.is_empty(),
        "a short Play is silently dropped"
    );
    let still_running = second
        .snapshot()
        .gameplay
        .unwrap()
        .encounter
        .unwrap()
        .active;
    assert!(still_running.is_some(), "the second battle still runs");
    let won = send(&mut second, &chat::local_request("realm win"), now + 1_000);
    assert!(won.state_changed);
    let second_paid = economy(&second, &prizes);
    assert_payout(&second_before, &second_paid, row);
    assert_eq!(
        economy(&first, &prizes),
        first_paid,
        "the second player's win never pays the first"
    );
}

// ---------------------------------------------------------------------------
// T330: "Chapter 1: Thugamug's Thievery" (quest 3000032) over battle 12340.
//
// The chapter record lives in `data/content/quests/adventurer.json`; the battle
// is T282's `field_battles.json` row. These tests drive the production sessions:
// the gateway's 26/8 -> 26/10 -> 26/6 -> 49/1 -> 49/2 offer/accept path, the
// shared-field attack path for T279's wolves, the 41/108 Go and 36/10 arena
// path, and the 49/13 -> 49/14 turn-in. A `QuestSession` also mirrors the one
// server-owned side table the bare gateway does not carry: `commit_outcome`
// (bin/osfr-gateway.rs) dispatches an accepted `shared_field` action through
// `handle_shared_field` for actors in the session's known-NPC set.
//
// Client layouts used by these tests (IDA):
// - 26/8 interact `0x008b6bc0`; 26/10 select `0x00a36f80`; 26/6 dialogue
//   `0x009f5890`.
// - 49/1 offer reader `0x00c7bb60` -> `0x00c7b7a0` (quest 6..10, intro 10..14,
//   title 14..18 and 18..22, description 22..26).
// - 49/2 reply `0x00a86400`; 49/3 quest add `0x00c7bd10`; 49/9 update
//   `0x00c7af60` (quest 6..10, objective 10..14, current 14..18); 49/10
//   complete `0x00c7b070`; 49/13 end reader `0x00c7b990` / writer `0x00c7bbd0`
//   (actor 6..14, quest 14..18, end text 18..22, reward bundle at 26: coins
//   27..31, displayed XP 31..35); 49/14 reply `0x00a864c0`.
// - 41/108 Go `0x008b6e70`; 41/114 start screen `0x009bdd70`; 36/10 attacker
//   `0x00a35150`; 35/2 AddNpc `0x00921a10`; 26/42 results Close `0x008a2eb0`.

/// T279's playable Chapter 1 entry, the chapter's only prerequisite.
const MISSING_SISTERS: u32 = 3_000_029;
/// The chapter this task enables.
const THIEVERY: u32 = 3_000_032;
const INTRO_TEXT: u32 = 90_702;
const END_TEXT: u32 = 90_705;
const QUEST_COINS: u32 = 13;
const QUEST_XP: u32 = 25;
const MISSING_SISTERS_END_TEXT: u32 = 90_624;
const MISSING_SISTERS_COINS: u32 = 5;
const MISSING_SISTERS_XP: u32 = 25;
const GEOFFREY: u64 = 9_000_000_330;
/// A drawable `legacy_data` Sanctuary quest giver who offers no enabled quest.
const HARRY: u64 = 9_500_000_493;
const GEOFFREY_MODEL: i32 = 415;
const GEOFFREY_NAME_ID: u32 = 70_318;
const ADVENTURER: i32 = 1;
const MINER: i32 = 14;
/// The generic Talk option label and the quest row of the 26/6 dialogue.
const TALK_LABEL: u32 = 3_300;
const QUEST_CHOICE: i32 = 301;
const WOLF_DEFINITION: i32 = 3_156;
const WOLF_GROUP: &str = "starting_farnum_wolves";
const WOLF_CENTRE: [f32; 3] = [-1899.204, -50.59, 307.902];

fn npc_position(file: &str, npc_id: u64) -> [f32; 3] {
    let document = read_json(&content_dir().join("npcs").join(file));
    let row = document["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["npc_id"].as_u64() == Some(npc_id))
        .unwrap_or_else(|| panic!("NPC {npc_id} is defined in {file}"));
    position(&row["position"])
}

fn geoffrey_position() -> [f32; 3] {
    npc_position("sanctuary.json", GEOFFREY)
}

/// The three live `starting_farnum_wolves` actors (definition 3156), in GUID order.
fn starting_wolves() -> Vec<(u64, [f32; 3])> {
    let document = read_json(&content_dir().join("field_enemies.json"));
    let group = document["spawn_groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|group| group["id"] == WOLF_GROUP)
        .expect("the starting_farnum_wolves group");
    let members: BTreeSet<u64> = group["members"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| id.as_u64().unwrap())
        .collect();
    let mut wolves: Vec<(u64, [f32; 3])> = document["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|enemy| members.contains(&enemy["guid"].as_u64().unwrap()))
        .map(|enemy| {
            assert_eq!(
                enemy["definition_id"].as_u64(),
                Some(WOLF_DEFINITION as u64)
            );
            (u64_field(enemy, "guid"), position(&enemy["position"]))
        })
        .collect();
    wolves.sort_by_key(|(guid, _)| *guid);
    assert_eq!(
        wolves.len(),
        3,
        "the group has exactly three starter wolves"
    );
    wolves
}

/// Every actor the server delivers through the shared field: field enemies and
/// battle spawners. The server seeds a session's known-NPC set from this
/// population, so a test that does not run the field delivery tick seeds it here.
fn shared_guids() -> BTreeSet<u64> {
    let field = read_json(&content_dir().join("field_enemies.json"));
    field["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .map(|enemy| u64_field(enemy, "guid"))
        .chain(
            battles()["battles"]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| u64_field(&row["spawner"], "guid")),
        )
        .collect()
}

fn packet_family(packet: &[u8], family: u16, subtype: u16) -> bool {
    packet.len() >= 4
        && packet[..2] == family.to_le_bytes()
        && packet[2..4] == subtype.to_le_bytes()
}

fn u32_at(packet: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(packet[offset..offset + 4].try_into().unwrap())
}

fn u64_at(packet: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(packet[offset..offset + 8].try_into().unwrap())
}

fn replies(outcome: &GatewayOutcome) -> impl Iterator<Item = &[u8]> {
    outcome.replies.iter().map(|packet| unframe(packet))
}

fn offered_quests(outcome: &GatewayOutcome) -> Vec<u32> {
    replies(outcome)
        .filter(|packet| packet_family(packet, 49, 1) && packet.len() >= 10)
        .map(|packet| u32_at(packet, 6))
        .collect()
}

fn menu_option(outcome: &GatewayOutcome, label: u32) -> Option<i32> {
    let menu = replies(outcome).find(|packet| packet_family(packet, 26, 9))?;
    let count = i32::from_le_bytes(menu[13..17].try_into().unwrap());
    (0..count).find_map(|index| {
        let offset = 17 + usize::try_from(index).unwrap() * 28;
        let id = i32::from_le_bytes(menu[offset..offset + 4].try_into().unwrap());
        (u32_at(menu, offset + 8) == label).then_some(id)
    })
}

fn npc_select(target: u64, choice: i32) -> Vec<u8> {
    [
        vec![26, 0, 10, 0],
        target.to_le_bytes().to_vec(),
        choice.to_le_bytes().to_vec(),
    ]
    .concat()
}

fn dialogue_choice(row: i32) -> Vec<u8> {
    [vec![26, 0, 6, 0], row.to_le_bytes().to_vec()].concat()
}

fn quest_reply(quest: u32, accept: bool) -> Vec<u8> {
    [
        vec![49, 0, 2, 0, 0, 0],
        quest.to_le_bytes().to_vec(),
        vec![u8::from(accept)],
    ]
    .concat()
}

fn quest_end_reply(quest: u32) -> Vec<u8> {
    [vec![49, 0, 14, 0, 0, 0], quest.to_le_bytes().to_vec()].concat()
}

fn quest_counts(gateway: &Gateway, quest: u32) -> Vec<u32> {
    gateway
        .snapshot()
        .gameplay
        .unwrap()
        .npc_quests
        .unwrap()
        .quests[&quest]
        .counts
        .clone()
}

fn quest_status(gateway: &Gateway, quest: u32) -> QuestStatus {
    gateway
        .snapshot()
        .gameplay
        .unwrap()
        .npc_quests
        .unwrap()
        .quests[&quest]
        .status
        .clone()
}

fn pending_turnin(gateway: &Gateway) -> Option<u32> {
    gateway
        .snapshot()
        .gameplay
        .unwrap()
        .npc_quests
        .unwrap()
        .pending_turnin
}

fn wallet(gateway: &Gateway) -> u32 {
    gateway.snapshot().gameplay.unwrap().inventory.coins
}

fn typed_state(gateway: &Gateway) -> GameplayState {
    gateway.snapshot().gameplay.unwrap()
}

/// Rebuild the production engine over the session's saved quest state, so an
/// offer gate can be asserted without starting a second session.
fn engine_for(session: &QuestSession) -> QuestEngine {
    let eligibility = CharacterEligibility {
        player_level: 20,
        job_levels: BTreeMap::from([("adventurer".into(), 20)]),
        member: true,
        nonmember_quest_limit: 30,
        member_quest_limit: 30,
    };
    QuestEngine::from_split_content(
        &content_dir(),
        session.gateway.state().guid,
        eligibility,
        Some(
            session
                .gateway
                .snapshot()
                .gameplay
                .unwrap()
                .npc_quests
                .unwrap(),
        ),
    )
    .expect("the saved quest state rebuilds")
}

/// Track the 35/2 AddNpc / 35/3 removal stream the server turns into `known_npcs`.
fn observe(known: &mut BTreeSet<u64>, outcome: &GatewayOutcome) {
    for packet in replies(outcome) {
        if packet_family(packet, 35, 2) {
            let npc =
                startup_world::parse_add_npc(packet).expect("the gateway emits a readable AddNpc");
            known.insert(npc.actor_id);
        } else if packet_family(packet, 35, 3) && packet.len() >= 14 {
            known.remove(&u64_at(packet, 6));
        }
    }
}

/// One gateway session plus the server-side shared-field dispatch and known-NPC
/// set `bin/osfr-gateway.rs` owns outside the gateway.
struct QuestSession {
    gateway: Gateway,
    field: SharedFieldState,
    known: BTreeSet<u64>,
    now: u64,
}

impl Fixture {
    /// A session whose shared field is taken before world entry, in the
    /// server's `Session::new` order. `extra_jobs` seeds an otherwise-locked
    /// profile set so a test can switch to a second job.
    fn quest_session(&self, position: [f32; 3], guid: u64, extra_jobs: &[i32]) -> QuestSession {
        let mut character = Gateway::reference_state(&self.reference).unwrap();
        character.guid = guid;
        character.zone_id = 0;
        character.position = position;
        character.rotation = [0.0, 0.0, 1.0];
        let saved = (!extra_jobs.is_empty()).then(|| {
            let seed =
                Gateway::load_gameplay(&self.reference, character.clone(), &self.content, None)
                    .unwrap();
            let mut gameplay = seed.snapshot().gameplay.unwrap();
            gameplay
                .profiles
                .as_mut()
                .unwrap()
                .unlocked
                .extend(extra_jobs.iter().copied());
            gameplay
        });
        let mut gateway =
            Gateway::load_gameplay(&self.reference, character, &self.content, saved).unwrap();
        gateway.set_dev_tools(true);
        gateway.set_role(AccountRole::Admin);
        let field = gateway
            .take_shared_fields()
            .expect("the bundled shared field content loads");
        self.login(&mut gateway);
        let mut known = shared_guids();
        for packet in [&13_u16.to_le_bytes(), &10_u16.to_le_bytes()] {
            let entry = send(&mut gateway, packet, 100);
            observe(&mut known, &entry);
        }
        QuestSession {
            gateway,
            field,
            known,
            now: 300,
        }
    }
}

impl QuestSession {
    fn send(&mut self, packet: &[u8]) -> GatewayOutcome {
        self.now += 10;
        self.send_at(packet, self.now)
    }

    /// The gateway request plus the host's post-commit shared-field dispatch
    /// (`commit_outcome` -> `handle_shared_field`). A shared action for an
    /// unknown actor is dropped, exactly as the host drops it.
    fn send_at(&mut self, packet: &[u8], now: u64) -> GatewayOutcome {
        let mut outcome = send(&mut self.gateway, packet, now);
        observe(&mut self.known, &outcome);
        if let Some(action) = outcome.shared_field.take() {
            let target = action.target();
            if target == self.gateway.state().guid || target == 0 || self.known.contains(&target) {
                let field = &mut self.field;
                let result = self
                    .gateway
                    .handle_shared_field(&action, field, &self.known, now)
                    .expect("the shared field action succeeds");
                outcome.state_changed |= result.changed;
                outcome.gameplay_changed |= result.changed;
                for reply in result.replies {
                    outcome.replies.push(wrap(&reply));
                }
                observe(&mut self.known, &outcome);
            }
        }
        outcome
    }

    /// The real 26/13 switch to the only combat job the fixture unlocks, then
    /// the local `realm level 5` tuning command.
    fn arm(&mut self) {
        assert!(
            self.send(&switch_job(BRAWLER)).state_changed,
            "the Brawler outfit is selectable"
        );
        let levelled = self.send(&chat::local_request("realm level 5"));
        assert!(levelled.state_changed, "the level command commits");
        assert_eq!(level(&self.gateway, BRAWLER), REQUIRED_LEVEL);
    }

    fn talk(&mut self, target: u64) -> GatewayOutcome {
        let menu = self.send(&interact(target));
        let choice = menu_option(&menu, TALK_LABEL)
            .unwrap_or_else(|| panic!("NPC {target} exposes a Talk option"));
        self.send(&npc_select(target, choice))
    }

    fn offer(&mut self, target: u64, quest: u32) -> GatewayOutcome {
        let dialogue = self.talk(target);
        assert!(
            replies(&dialogue).any(|packet| packet_family(packet, 26, 3)),
            "Talk opens the 26/3 dialogue"
        );
        let offered = self.send(&dialogue_choice(QUEST_CHOICE));
        assert_eq!(
            offered_quests(&offered),
            [quest],
            "one Talk offers exactly quest {quest}"
        );
        offered
    }

    fn accept(&mut self, quest: u32) -> GatewayOutcome {
        let accepted = self.send(&quest_reply(quest, true));
        assert!(accepted.state_changed, "accepting quest {quest} commits");
        assert!(
            replies(&accepted)
                .any(|packet| packet_family(packet, 49, 3) && u32_at(packet, 6) == quest),
            "accepting quest {quest} sends QuestAdd"
        );
        assert!(
            replies(&accepted)
                .any(|packet| packet_family(packet, 49, 8) && u32_at(packet, 6) == quest),
            "accepting quest {quest} activates its first objective"
        );
        assert_eq!(self.status(quest), QuestStatus::Active);
        accepted
    }

    fn walk(&mut self, to: [f32; 3]) {
        let guid = self.gateway.state().guid;
        let from = self.gateway.state().position;
        let distance = ((to[0] - from[0]).powi(2) + (to[2] - from[2]).powi(2)).sqrt();
        let steps = (distance / 8.0).ceil().max(1.0) as usize;
        for step in 1..=steps {
            let amount = step as f32 / steps as f32;
            let position = [0, 1, 2].map(|axis| from[axis] + (to[axis] - from[axis]) * amount);
            self.now += 400;
            let moved = self.send_at(&move_to(guid, position), self.now);
            assert!(
                !replies(&moved).any(|packet| packet_family(packet, 38, 12)),
                "a walk step is never snapped back"
            );
            assert_eq!(self.gateway.state().position, position);
        }
    }

    /// One basic attack on a shared-field actor, walking into reach first.
    fn attack(&mut self, target: u64) {
        for _ in 0..16 {
            let position = self
                .field
                .position(target)
                .unwrap_or_else(|| panic!("shared target {target} exists"));
            self.walk([position[0] + 1.0, position[1], position[2]]);
            self.now += 700;
            let hit = self.send_at(&melee(target), self.now);
            assert!(hit.state_changed, "the shared-field attack commits");
            if self.field.health(target) == Some(0) {
                return;
            }
        }
        panic!("shared target {target} did not die");
    }

    /// T279's chapter through the real path: accept at Geoffrey, visit the
    /// search-party spot, kill the three starter wolves, return and turn in.
    fn accept_missing_sisters(&mut self) {
        self.offer(GEOFFREY, MISSING_SISTERS);
        self.accept(MISSING_SISTERS);
        assert_eq!(self.counts(MISSING_SISTERS), [0, 0, 0]);
    }

    fn finish_missing_sisters(&mut self) {
        assert_eq!(self.status(MISSING_SISTERS), QuestStatus::Active);
        self.walk(WOLF_CENTRE);
        assert_eq!(
            self.counts(MISSING_SISTERS),
            [1, 0, 0],
            "the movement visit advances the search-party step"
        );
        for (guid, _) in starting_wolves() {
            self.attack(guid);
        }
        assert_eq!(self.counts(MISSING_SISTERS), [1, 3, 0]);
        self.walk(geoffrey_position());
        let end = self.open_end_screen(MISSING_SISTERS, MISSING_SISTERS_END_TEXT);
        assert_eq!(u32_at(&end, 27), MISSING_SISTERS_COINS);
        assert_eq!(u32_at(&end, 31), MISSING_SISTERS_XP);
        self.finish_quest(
            MISSING_SISTERS,
            ADVENTURER,
            MISSING_SISTERS_COINS,
            MISSING_SISTERS_XP,
        );
    }

    fn complete_missing_sisters(&mut self) {
        self.accept_missing_sisters();
        self.finish_missing_sisters();
    }

    /// Talk to Geoffrey and release the held 49/13 with the 26/6 quest choice.
    fn open_end_screen(&mut self, quest: u32, end_text: u32) -> Vec<u8> {
        let dialogue = self.talk(GEOFFREY);
        assert!(
            replies(&dialogue).any(|packet| packet_family(packet, 26, 3)),
            "Geoffrey opens the 26/3 dialogue"
        );
        let released = self.send(&dialogue_choice(QUEST_CHOICE));
        let end = replies(&released)
            .find(|packet| packet_family(packet, 49, 13) && u32_at(packet, 14) == quest)
            .unwrap_or_else(|| panic!("quest {quest} releases its 49/13 end screen"))
            .to_vec();
        assert_eq!(end.len(), 99, "the empty-display reward bundle is 99 bytes");
        assert_eq!(u64_at(&end, 6), GEOFFREY, "the end screen names the actor");
        assert_eq!(u32_at(&end, 18), end_text, "49/13 carries the end text");
        end
    }

    fn finish_quest(&mut self, quest: u32, profile: i32, coins: u32, xp: u32) {
        let coins_before = self.coins();
        let xp_before = self.xp(profile);
        let completed = self.send(&quest_end_reply(quest));
        assert!(completed.state_changed, "the turn-in settles");
        assert!(
            replies(&completed)
                .any(|packet| packet_family(packet, 49, 4) && u32_at(packet, 6) == quest),
            "quest {quest} completes"
        );
        assert_eq!(self.coins(), coins_before + coins);
        assert_eq!(self.xp(profile), xp_before + xp);
        assert_eq!(self.status(quest), QuestStatus::Completed);
        assert_eq!(self.pending(), None);
    }

    fn open_battle(&mut self, row: &Value, marker: [f32; 3]) -> Vec<u8> {
        self.walk(marker);
        let screen = self.send(&interact(MARKER_GUID));
        let details = replies(&screen)
            .find(|packet| packet_family(packet, 41, 114))
            .expect("clicking the bridge marker opens 41/114")
            .to_vec();
        assert_start_screen(&details, row);
        details
    }

    fn enter_arena(&mut self, details: &[u8], row: &Value) -> Vec<u64> {
        self.now += 100;
        let guids = enter_battle(&mut self.gateway, details, row, self.now);
        self.now += 200;
        guids
    }

    fn fight(&mut self) {
        fight_to_win(&mut self.gateway, &mut self.now);
    }

    /// 26/42 Close, then the ready/loaded handshake the client repeats for the
    /// scene it returns to.
    fn close_results(&mut self) -> GatewayOutcome {
        self.now += 100;
        let closed = send(&mut self.gateway, &[26, 0, 42, 0], self.now);
        assert!(closed.began_zoning, "Close returns to the open world");
        assert_eq!(self.gateway.state().zone_id, 0);
        self.now += 100;
        self.send(&13_u16.to_le_bytes());
        self.now += 100;
        let loaded = self.send(&10_u16.to_le_bytes());
        assert!(loaded.entered_world, "the open world re-loads");
        closed
    }

    /// Win the bridge battle through Go and the real attack path.
    fn win_bridge_battle(&mut self, row: &Value) {
        let marker = position(&row["spawner"]["position"]);
        let details = self.open_battle(row, marker);
        self.enter_arena(&details, row);
        self.fight();
        self.close_results();
    }

    fn counts(&self, quest: u32) -> Vec<u32> {
        quest_counts(&self.gateway, quest)
    }

    fn status(&self, quest: u32) -> QuestStatus {
        quest_status(&self.gateway, quest)
    }

    fn pending(&self) -> Option<u32> {
        pending_turnin(&self.gateway)
    }

    fn coins(&self) -> u32 {
        wallet(&self.gateway)
    }

    fn xp(&self, profile: i32) -> u32 {
        profile_xp(&self.gateway, profile)
    }

    fn active_job(&self) -> i32 {
        active_job(&self.gateway)
    }

    fn capture(&self) -> (GameplayState, Vec<u8>) {
        (typed_state(&self.gateway), saved_bytes(&self.gateway))
    }

    fn assert_unchanged(&mut self, packet: &[u8], label: &str) {
        let before = self.capture();
        let outcome = self.send(packet);
        assert!(!outcome.state_changed, "{label} changed the character");
        assert!(
            replies(&outcome).all(|reply| {
                !(packet_family(reply, 49, 1)
                    || packet_family(reply, 49, 3)
                    || packet_family(reply, 49, 4)
                    || packet_family(reply, 49, 13))
            }),
            "{label} offered, added, completed or ended a quest"
        );
        assert_eq!(
            self.capture(),
            before,
            "{label} changed the typed state or the save bytes"
        );
    }
}

/// The chapter is offered at the drawable Geoffrey only after T279's `3000029`
/// is Completed, and its 49/1 carries the reviewed text ids.
#[test]
fn geoffrey_offers_thugamugs_thievery_after_the_missing_sisters() {
    let fixture = Fixture::new();
    let mut session = fixture.quest_session(geoffrey_position(), 3_001, &[]);
    session.arm();

    // Before the prerequisite: the only 49/1 is T279's chapter. The disabled
    // middle records 3000030/3000031 are never offered either.
    let dialogue = session.talk(GEOFFREY);
    assert!(replies(&dialogue).any(|packet| packet_family(packet, 26, 3)));
    let first = session.send(&dialogue_choice(QUEST_CHOICE));
    assert_eq!(offered_quests(&first), [MISSING_SISTERS]);

    // Finish the prerequisite through the real wolf path, then the chapter appears.
    session.accept(MISSING_SISTERS);
    assert!(
        !engine_for(&session).can_offer(THIEVERY, GEOFFREY),
        "the unfinished prerequisite denies the chapter"
    );
    session.finish_missing_sisters();
    assert_eq!(session.status(MISSING_SISTERS), QuestStatus::Completed);
    assert!(
        engine_for(&session).can_offer(THIEVERY, GEOFFREY),
        "the completed prerequisite admits the chapter"
    );

    let offered = session.offer(GEOFFREY, THIEVERY);
    let packet = replies(&offered)
        .find(|packet| packet_family(packet, 49, 1) && u32_at(packet, 6) == THIEVERY)
        .expect("the 49/1 offer");
    assert_eq!(u32_at(packet, 10), INTRO_TEXT, "49/1 intro id");
    assert_eq!(u32_at(packet, 14), TITLE, "49/1 title id");
    assert_eq!(u32_at(packet, 18), TITLE, "49/1 title id (card)");
    assert_eq!(u32_at(packet, 22), DESCRIPTION, "49/1 description id");

    // A second adventurer without 3000029 gets T279's offer, never this chapter.
    let mut fresh = fixture.quest_session(geoffrey_position(), 3_002, &[]);
    let dialogue = fresh.talk(GEOFFREY);
    assert!(replies(&dialogue).any(|packet| packet_family(packet, 26, 3)));
    let offered = fresh.send(&dialogue_choice(QUEST_CHOICE));
    assert_eq!(offered_quests(&offered), [MISSING_SISTERS]);
    assert_eq!(fresh.status(MISSING_SISTERS), QuestStatus::Offered);
}

/// The chapter's dungeon step is the T282 bridge battle; the results Close leaves
/// 1/1 on it and the Talk step open, and 49/14 pays 13 coins + 25 Adventurer XP once.
#[test]
fn winning_the_bridge_battle_completes_the_chapter_and_pays_once() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, THUGAMUG);
    let mut session = fixture.quest_session(geoffrey_position(), 3_003, &[]);
    session.arm();
    session.complete_missing_sisters();
    session.offer(GEOFFREY, THIEVERY);
    session.accept(THIEVERY);
    assert_eq!(session.counts(THIEVERY), [0, 0]);

    session.win_bridge_battle(row);

    // T312 credits the dungeon step at the win; asserting after the results
    // Close holds on both that code and the older close-time credit.
    assert_eq!(
        session.counts(THIEVERY),
        [1, 0],
        "the dungeon step is complete and the Talk step is open"
    );
    assert_eq!(session.status(THIEVERY), QuestStatus::Active);
    assert_eq!(session.pending(), None);

    session.walk(geoffrey_position());
    let dialogue = session.talk(GEOFFREY);
    assert!(
        replies(&dialogue).any(|packet| packet_family(packet, 49, 9)
            && u32_at(packet, 6) == THIEVERY
            && u32_at(packet, 10) == 2
            && u32_at(packet, 14) == 1),
        "the Talk objective advances to 1/1"
    );
    let released = session.send(&dialogue_choice(QUEST_CHOICE));
    let end = replies(&released)
        .find(|packet| packet_family(packet, 49, 13) && u32_at(packet, 14) == THIEVERY)
        .expect("the 49/13 end screen")
        .to_vec();
    assert_eq!(end.len(), 99);
    assert_eq!(u64_at(&end, 6), GEOFFREY);
    assert_eq!(u32_at(&end, 18), END_TEXT);
    assert_eq!(u32_at(&end, 27), QUEST_COINS, "49/13 coin bundle");
    assert_eq!(u32_at(&end, 31), QUEST_XP, "49/13 displayed XP bundle");
    assert_eq!(session.counts(THIEVERY), [1, 1]);

    // The end screen alone pays nothing.
    let coins_before = session.coins();
    let adventurer_before = session.xp(ADVENTURER);
    let brawler_before = session.xp(BRAWLER);
    session.finish_quest(THIEVERY, ADVENTURER, QUEST_COINS, QUEST_XP);
    assert_eq!(session.coins(), coins_before + QUEST_COINS);
    assert_eq!(session.xp(ADVENTURER), adventurer_before + QUEST_XP);
    assert_eq!(session.xp(BRAWLER), brawler_before);
    assert_eq!(session.status(THIEVERY), QuestStatus::Completed);
    assert_eq!(session.counts(THIEVERY), [1, 1]);

    // A replayed turn-in or accept pays nothing and changes no byte.
    let finished = session.capture();
    session.assert_unchanged(&quest_end_reply(THIEVERY), "a replayed 49/14");
    session.assert_unchanged(&quest_reply(THIEVERY, true), "a replayed 49/2");
    assert_eq!(session.capture(), finished);
}

/// The reward map is explicit: switching jobs after accepting still pays profile 1.
#[test]
fn the_reward_goes_to_profile_1_after_a_job_switch() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, THUGAMUG);
    let mut session = fixture.quest_session(geoffrey_position(), 3_004, &[MINER]);
    session.arm();
    session.complete_missing_sisters();
    session.offer(GEOFFREY, THIEVERY);
    session.accept(THIEVERY);
    session.win_bridge_battle(row);

    // The real 26/13 path switches to the other unlocked job after accepting.
    session.walk(geoffrey_position());
    let switched = session.send(&switch_job(MINER));
    assert!(switched.state_changed, "the Miner outfit is selectable");
    assert_eq!(session.active_job(), MINER);
    let adventurer_before = session.xp(ADVENTURER);
    let miner_before = session.xp(MINER);
    let brawler_before = session.xp(BRAWLER);

    let end = session.open_end_screen(THIEVERY, END_TEXT);
    assert_eq!(u32_at(&end, 27), QUEST_COINS);
    assert_eq!(u32_at(&end, 31), QUEST_XP);
    session.finish_quest(THIEVERY, ADVENTURER, QUEST_COINS, QUEST_XP);

    assert_eq!(
        session.active_job(),
        MINER,
        "the turn-in must not switch jobs"
    );
    assert_eq!(session.xp(ADVENTURER), adventurer_before + QUEST_XP);
    assert_eq!(
        session.xp(MINER),
        miner_before,
        "the active job is not paid"
    );
    assert_eq!(session.xp(BRAWLER), brawler_before);
}

/// Premature, wrong-quest, forged and wrong-NPC requests change nothing; a coin
/// balance that cannot hold the grant refuses the turn-in without paying XP or an
/// item and without a server error.
#[test]
fn a_premature_wrong_or_forged_turn_in_changes_nothing() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, THUGAMUG);
    let mut session = fixture.quest_session(geoffrey_position(), 3_005, &[]);
    session.arm();

    // Before the chain: no pending screen exists for either quest.
    session.assert_unchanged(&quest_end_reply(THIEVERY), "a 49/14 before the battle");
    session.assert_unchanged(
        &quest_end_reply(MISSING_SISTERS),
        "a 49/14 for another quest",
    );
    session.assert_unchanged(&quest_reply(THIEVERY, true), "a 49/2 with no open offer");

    // A wrong NPC: a drawable quest giver who offers no enabled quest. Its own
    // menu is allowed; the chapter is not offered and the save stays equal.
    session.walk(npc_position("sanctuary.json", HARRY));
    let before = session.capture();
    let menu = session.send(&interact(HARRY));
    assert!(
        replies(&menu).any(|packet| packet_family(packet, 26, 9)),
        "the wrong NPC opens its own menu"
    );
    if let Some(choice) = menu_option(&menu, TALK_LABEL) {
        let dialogue = session.send(&npc_select(HARRY, choice));
        assert!(
            replies(&dialogue).all(|packet| !packet_family(packet, 49, 1)),
            "the wrong NPC offers no quest"
        );
    }
    assert_eq!(
        session.capture(),
        before,
        "a wrong NPC Talk changed the save"
    );

    // An out-of-range click on Geoffrey is refused by the NPC router.
    session.assert_unchanged(&interact(GEOFFREY), "an out-of-range Geoffrey click");

    // Run the chain once and open the end screen.
    session.walk(geoffrey_position());
    session.complete_missing_sisters();
    session.offer(GEOFFREY, THIEVERY);
    session.accept(THIEVERY);
    session.win_bridge_battle(row);
    session.walk(geoffrey_position());
    session.open_end_screen(THIEVERY, END_TEXT);
    assert_eq!(session.pending(), Some(THIEVERY));

    // The coin balance cannot hold the 13-coin grant: the grant is refused for
    // this player, nothing is paid, and the pending screen stays retryable.
    let mut capped = reopen_with(
        &fixture,
        &session.gateway,
        "near-cap.json",
        session.now + 100,
        |gameplay| gameplay.inventory.coins = i32::MAX as u32 - 5,
    );
    let before = (typed_state(&capped), saved_bytes(&capped));
    let refused = send(&mut capped, &quest_end_reply(THIEVERY), session.now + 200);
    assert!(!refused.state_changed, "the over-cap turn-in is a refusal");
    assert!(
        !replies(&refused).any(|packet| {
            packet_family(packet, 49, 4)
                || packet_family(packet, 49, 13)
                || packet_family(packet, 38, 19)
        }),
        "a refused turn-in sends no completion, bundle or coin update"
    );
    assert_eq!(
        (typed_state(&capped), saved_bytes(&capped)),
        before,
        "the refusal changed the typed state or the save bytes"
    );
    assert_eq!(wallet(&capped), i32::MAX as u32 - 5);
    assert_eq!(pending_turnin(&capped), Some(THIEVERY));
    assert_eq!(quest_status(&capped, THIEVERY), QuestStatus::Active);

    // The same pending save with a normal wallet pays exactly once.
    let mut paying = reopen(
        &fixture,
        &session.gateway,
        "pending.json",
        session.now + 300,
    );
    let coins_before = wallet(&paying);
    let xp_before = profile_xp(&paying, ADVENTURER);
    let paid = send(&mut paying, &quest_end_reply(THIEVERY), session.now + 400);
    assert!(paid.state_changed);
    assert_eq!(wallet(&paying), coins_before + QUEST_COINS);
    assert_eq!(profile_xp(&paying, ADVENTURER), xp_before + QUEST_XP);
    assert_eq!(quest_status(&paying, THIEVERY), QuestStatus::Completed);

    // Replaying the old accept or turn-in after payment changes nothing.
    let finished = (typed_state(&paying), saved_bytes(&paying));
    for (packet, label) in [
        (quest_end_reply(THIEVERY), "a replayed 49/14"),
        (quest_reply(THIEVERY, true), "a replayed 49/2"),
    ] {
        let out = send(&mut paying, &packet, session.now + 500);
        assert!(!out.state_changed, "{label} changed the character");
        assert_eq!(
            (typed_state(&paying), saved_bytes(&paying)),
            finished,
            "{label} changed the typed state or the save bytes"
        );
    }
}

/// The pending end screen, counters and completion live in the save; the reward
/// is applied once by the `quest:<character>:<quest>` key at 49/14.
#[test]
fn progress_survives_a_reload_and_a_replay_pays_nothing() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, THUGAMUG);
    let mut session = fixture.quest_session(geoffrey_position(), 3_006, &[]);
    session.arm();
    session.complete_missing_sisters();
    session.offer(GEOFFREY, THIEVERY);
    session.accept(THIEVERY);
    session.win_bridge_battle(row);
    session.walk(geoffrey_position());
    let end = session.open_end_screen(THIEVERY, END_TEXT);
    assert_eq!(u32_at(&end, 27), QUEST_COINS);
    assert_eq!(u32_at(&end, 31), QUEST_XP);
    let coins_before = session.coins();
    let xp_before = session.xp(ADVENTURER);
    let counts_before = session.counts(THIEVERY);

    // Reconnect with the end screen still pending: the saved quest state,
    // counters and wallet survive unchanged.
    let mut restored = reopen(
        &fixture,
        &session.gateway,
        "pending.json",
        session.now + 100,
    );
    assert_eq!(quest_status(&restored, THIEVERY), QuestStatus::Active);
    assert_eq!(quest_counts(&restored, THIEVERY), counts_before);
    assert_eq!(pending_turnin(&restored), Some(THIEVERY));
    assert_eq!(wallet(&restored), coins_before);
    assert_eq!(profile_xp(&restored, ADVENTURER), xp_before);

    // The 49/14 settles the pending screen after the reconnect and pays once.
    let paid = send(&mut restored, &quest_end_reply(THIEVERY), session.now + 200);
    assert!(paid.state_changed);
    assert_eq!(wallet(&restored), coins_before + QUEST_COINS);
    assert_eq!(profile_xp(&restored, ADVENTURER), xp_before + QUEST_XP);
    assert_eq!(quest_status(&restored, THIEVERY), QuestStatus::Completed);
    assert_eq!(quest_counts(&restored, THIEVERY), counts_before);
    assert_eq!(pending_turnin(&restored), None);
    let finished = (typed_state(&restored), saved_bytes(&restored));

    // Reload the completed save and replay both old replies: nothing changes.
    let mut replayed = reopen(&fixture, &restored, "completed.json", session.now + 300);
    assert_eq!(quest_status(&replayed, THIEVERY), QuestStatus::Completed);
    assert_eq!(wallet(&replayed), wallet(&restored));
    assert_eq!(
        profile_xp(&replayed, ADVENTURER),
        profile_xp(&restored, ADVENTURER)
    );
    let reloaded = (typed_state(&replayed), saved_bytes(&replayed));
    for (packet, label) in [
        (quest_end_reply(THIEVERY), "a post-relog 49/14"),
        (quest_reply(THIEVERY, true), "a post-relog 49/2"),
    ] {
        let out = send(&mut replayed, &packet, session.now + 400);
        assert!(!out.state_changed, "{label} changed the character");
        assert_eq!(
            (typed_state(&replayed), saved_bytes(&replayed)),
            reloaded,
            "{label} changed the typed state or the save bytes"
        );
    }
    assert_eq!(
        (typed_state(&restored), saved_bytes(&restored)),
        finished,
        "the second session's replay changed the paid save"
    );
}

/// Two adventurers own their chapter state, battle instance and reward. One
/// player's win or refusal never credits, pays or saves the other.
#[test]
fn two_adventurers_do_not_share_the_chapter() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, THUGAMUG);
    let at = geoffrey_position();
    let mut a = fixture.quest_session(at, 3_007, &[]);
    let mut b = fixture.quest_session(at, 3_008, &[]);
    for session in [&mut a, &mut b] {
        session.arm();
        session.complete_missing_sisters();
        session.offer(GEOFFREY, THIEVERY);
        session.accept(THIEVERY);
    }

    // A runs its own arena. B's chapter, wallet and save never move.
    let b_before_fight = b.capture();
    a.win_bridge_battle(row);
    assert_eq!(a.counts(THIEVERY), [1, 0]);
    assert_eq!(b.capture(), b_before_fight, "A's battle changed B");
    assert_eq!(b.counts(THIEVERY), [0, 0]);
    assert_eq!(b.status(THIEVERY), QuestStatus::Active);

    // A turns in; only A is paid.
    let a_coins = a.coins();
    let b_coins = b.coins();
    a.walk(at);
    a.open_end_screen(THIEVERY, END_TEXT);
    a.finish_quest(THIEVERY, ADVENTURER, QUEST_COINS, QUEST_XP);
    assert_eq!(a.coins(), a_coins + QUEST_COINS);
    assert_eq!(a.status(THIEVERY), QuestStatus::Completed);
    assert_eq!(b.coins(), b_coins, "A's turn-in paid B");
    assert_eq!(b.status(THIEVERY), QuestStatus::Active);
    assert_eq!(b.capture(), b_before_fight, "A's turn-in changed B's save");
    let a_paid = a.capture();

    // B's own battle and turn-in pay B exactly once; A stays untouched.
    b.win_bridge_battle(row);
    assert_eq!(b.counts(THIEVERY), [1, 0]);
    assert_eq!(a.capture(), a_paid, "B's battle changed A's paid save");
    let b_coins = b.coins();
    b.walk(at);
    b.open_end_screen(THIEVERY, END_TEXT);
    b.finish_quest(THIEVERY, ADVENTURER, QUEST_COINS, QUEST_XP);
    assert_eq!(b.coins(), b_coins + QUEST_COINS);
    assert_eq!(b.status(THIEVERY), QuestStatus::Completed);
    assert_eq!(a.capture(), a_paid, "B's turn-in changed A's paid save");

    // A refused request for A (a replayed turn-in and a forged accept) is its own
    // no-op; B's completed save stays byte-identical.
    let b_paid = b.capture();
    a.assert_unchanged(&quest_end_reply(THIEVERY), "A's replayed 49/14");
    a.assert_unchanged(&quest_reply(THIEVERY, true), "A's forged 49/2");
    assert_eq!(a.capture(), a_paid);
    assert_eq!(b.capture(), b_paid, "A's refusals changed B's save");
}

/// The record, its text ids, Geoffrey's drawable spawn, the battle row and the
/// reward profile all resolve to the ids the runtime offer and end screen used.
#[test]
fn the_chapter_text_giver_battle_and_reward_resolve() {
    // The record: one prerequisite, two ordered steps and one reward row.
    let adventurer = read_json(&content_dir().join("quests/adventurer.json"));
    let record = adventurer["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(THIEVERY)))
        .expect("the 3000032 record");
    assert_eq!(record["enabled"], true);
    assert_eq!(record["title_string_id"], json!(TITLE));
    assert_eq!(record["description_string_id"], json!(DESCRIPTION));
    assert_eq!(record["intro_text_string_id"], json!(INTRO_TEXT));
    assert_eq!(record["end_text_string_id"], json!(END_TEXT));
    assert_eq!(record["giver_npc_id"], json!(GEOFFREY));
    assert_eq!(record["prerequisite_quest_ids"], json!([MISSING_SISTERS]));
    assert!(record.get("job").is_none(), "the chapter has no job gate");
    let objectives = record["objectives"].as_array().unwrap();
    assert_eq!(objectives.len(), 2);
    assert_eq!(objectives[0]["kind"], json!("complete_dungeon"));
    assert_eq!(objectives[0]["id"], json!(1));
    assert_eq!(objectives[0]["encounter_id"], json!(THUGAMUG));
    assert_eq!(objectives[1]["kind"], json!("talk_to_npc"));
    assert_eq!(objectives[1]["id"], json!(2));
    assert_eq!(objectives[1]["npc_id"], json!(GEOFFREY));
    for objective in objectives {
        assert!(
            objective["source"]
                .as_array()
                .is_some_and(|source| !source.is_empty()),
            "every objective carries provenance"
        );
    }
    let rewards = record["rewards"].as_array().unwrap();
    assert_eq!(rewards.len(), 1);
    assert_eq!(rewards[0]["coins"], json!(QUEST_COINS));
    assert_eq!(rewards[0]["items"], json!([]));
    assert_eq!(rewards[0]["job_xp"], json!({"1": QUEST_XP}));
    assert!(
        record["source"].as_array().unwrap().iter().any(|tag| {
            tag["kind"] == "reconstructed"
                && tag["evidence"]
                    .as_str()
                    .is_some_and(|text| text.starts_with("(R) T330:"))
        }),
        "the record carries its (R) decision"
    );

    // The skipped middle records stay disabled and hollow.
    for disabled in [3_000_030u32, 3_000_031] {
        let row = adventurer["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|record| record["id"].as_u64() == Some(u64::from(disabled)))
            .unwrap_or_else(|| panic!("the {disabled} record"));
        assert_eq!(row["enabled"], false, "{disabled} must stay disabled");
        assert_eq!(row["objectives"], json!([]));
        assert_eq!(row["rewards"], json!([]));
    }

    // The engine resolves the same ids, and the 49/13 packet carries the bundle.
    let engine =
        QuestEngine::from_split_content(&content_dir(), 1, CharacterEligibility::default(), None)
            .expect("the split catalogue loads");
    assert!(engine.is_enabled(THIEVERY));
    assert_eq!(engine.giver_npc_id(THIEVERY), Some(GEOFFREY));
    assert_eq!(engine.title_string_id(THIEVERY), Some(TITLE));
    assert_eq!(engine.description_string_id(THIEVERY), Some(DESCRIPTION));
    assert_eq!(engine.intro_text_string_id(THIEVERY), Some(INTRO_TEXT));
    assert_eq!(engine.end_text_string_id(THIEVERY), Some(END_TEXT));
    let end = engine.quest_end_packet(THIEVERY, GEOFFREY).unwrap();
    assert_eq!(end.len(), 99);
    assert_eq!(u32_at(&end, 14), THIEVERY);
    assert_eq!(u32_at(&end, 18), END_TEXT);
    assert_eq!(u32_at(&end, 27), QUEST_COINS);
    assert_eq!(u32_at(&end, 31), QUEST_XP);

    // The reward profile is the Adventurer job that lists the chapter.
    let jobs = read_json(&content_dir().join("jobs.json"));
    let adventurer_job = jobs["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|job| job["profile_id"].as_i64() == Some(i64::from(ADVENTURER)))
        .expect("the Adventurer job profile");
    assert_eq!(adventurer_job["id"], json!("adventurer"));
    assert!(
        adventurer_job["quest_ids"]
            .as_array()
            .unwrap()
            .contains(&json!(THIEVERY)),
        "the Adventurer job lists the chapter"
    );

    // Geoffrey's drawable actor is the extra spawn with the same content key the
    // quest names; the Sanctuary row alone is not a drawable placement.
    let npc = read_json(&content_dir().join("npcs/sanctuary.json"));
    let npc_row = npc["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["npc_id"].as_u64() == Some(GEOFFREY))
        .expect("the Geoffrey NPC row");
    assert_eq!(npc_row["placement"], json!("unanchored"));
    let spawn = read_json(&content_dir().join("npc_spawns_extra.json"));
    let spawn_row = spawn["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["spawn_guid"].as_u64() == Some(GEOFFREY))
        .expect("Geoffrey's drawable extra spawn");
    assert_eq!(spawn_row["content_key"], json!(GEOFFREY));
    assert_eq!(spawn_row["model_id"], json!(GEOFFREY_MODEL));
    assert_eq!(spawn_row["name_string_id"], json!(GEOFFREY_NAME_ID));
    assert_eq!(
        position(&spawn_row["position"]),
        position(&npc_row["position"])
    );
    let mut visibility = world::NpcVisibility::load(&content_dir()).unwrap();
    let drawn = visibility
        .update(0, geoffrey_position())
        .unwrap()
        .iter()
        .filter_map(|packet| startup_world::parse_add_npc(packet).ok())
        .find(|npc| npc.actor_id == GEOFFREY)
        .expect("Geoffrey's AddNpc for a nearby viewer");
    assert_eq!(drawn.model_id, GEOFFREY_MODEL);
    assert_eq!(drawn.name_string_id, GEOFFREY_NAME_ID as i32);

    // The battle row the chapter credits: same title, description and goal,
    // level 5, three enemies and the arena the runtime Go enters.
    let file = battles();
    let row = battle(&file, THUGAMUG);
    assert_eq!(u32_field(row, "title_id"), TITLE);
    assert_eq!(u32_field(row, "description_id"), DESCRIPTION);
    assert_eq!(u32_field(row, "objective_id"), OBJECTIVE);
    assert_eq!(u32_field(row, "required_combat_level"), REQUIRED_LEVEL);
    assert_eq!(u32_field(&row["zone"], "zone_id"), 9600);
    assert_eq!(row["enemies"].as_array().unwrap().len(), 3);
}
