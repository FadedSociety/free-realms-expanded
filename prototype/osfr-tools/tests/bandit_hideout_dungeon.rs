//! T353: the Bandit Hideout as a difficulty-4, level-9 field battle.
//!
//! Content-only slice: one new `field_battles.json` row and the existing route
//! (26/8 marker -> 41/114 start screen -> 41/108 Go -> arena zone 9600 ->
//! 36/10 attacks -> results -> 26/42 Close, or 167/1/2 portal Play). No new
//! wire field, handler, opcode, selector or engine code is added. The session,
//! packet, route, bag and placement helpers are the shared field-battle fixture.
//!
//! Client shapes checked in the supplied 2014 client:
//! * 41/108 Go constructor `0x008b6e70` (stores the encounter and instance at
//!   +6/+7 and calls `sub_8A2FD0(108)`).
//! * 41/114 start-screen reader chain `0x00aa3430` -> `0x00aa32d0` ->
//!   `0x00a29120` -> `0x009bdd70`; the trailing reader reads the title, icon,
//!   description, difficulty, combat profile type and type consecutively.
//! * 35/2 AddNpc body reader `0x00921a10` reads the u64 actor id, name id and
//!   model id. The 35-family dispatcher at `0x0092f460` is not inside a carved
//!   IDA function (`func` reports "not inside a function"); the case 2 = 35/2
//!   and case 3 = 35/3 (removal constructor `0x008b3840`) mapping is the
//!   lead-reviewed `notes/Design/Shared World.md` N1-N3, not re-derived here.
//! * 32/7 AttackProcessed is built by `0x00a2a470` (`sub_A297E0(7)`); hit
//!   feedback does not change the authoritative HP this test asserts.
//! * The start-panel picture is this battle's own Images.txt row 660,
//!   `icon_ui_combatstart_panelgraphic_bandithideout.dds`, from the 660-683
//!   panel block that also gives Bixie Hive its 662.
//!
//! Placement evidence: the guide gives only "click the barrier near the
//! Greenwood Camp"; no barrier prop or entrance transform is recovered (R).
//! The marker is the visible Muggenstomp model 200 at
//! `[-1903.0, -34.75, -660.0]`, south-east of Lugabow on the camp approach.
//! Using `lab/terrain/zone_0.hgt`, y is the bilinear height `-34.7500`
//! (delta 0.0000 m); all 105 samples of the 5 m wander disc (81 grid + 24 ring)
//! are terrain-covered with a finite y; the full disc reaches at most 14.04
//! degrees of slope. The nearest visible `npc_spawns_extra.json` actor is the
//! Bandit Patrolman `100000004341` at 25.76 m and the nearest regional
//! `legacy_data` NPC is Lugabow at 26.78 m; both clear the 15 m gate. The
//! nearest other battle marker (Thugamug, 12340) is 339.18 m, and the nearest
//! field-enemy home is 946.62 m. Water, object collision and a native walkable
//! route remain unverified, as in the T282/T319 rows.

use osfr_tools::{
    account_store::AccountRole,
    chat,
    encounter::EncounterEngine,
    gameplay::Gameplay,
    gateway::{Gateway, GatewayOutcome},
    inventory::{InventoryState, MAX_ITEMS},
    mining::{MINER_PROFILE_ID, MINING_GAME_ID},
    startup_world::{NpcSpawn, parse_add_npc},
};
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};

#[path = "common/combat_scale.rs"]
mod combat_scale;
#[path = "common/content_copy.rs"]
mod content_copy;
#[path = "common/field_battle_fixture.rs"]
mod field_battle_fixture;
#[path = "support/terrain.rs"]
mod terrain;

use field_battle_fixture::*;
use terrain::{disc_points, slope_degrees};

/// The new row, appended after battle 12400.
const BANDIT_HIDEOUT: u32 = 12360;
const TITLE: i32 = 30422;
const DESCRIPTION: i32 = 6995;
const OBJECTIVE: i32 = 6997;
const DIFFICULTY: u32 = 4;
const REQUIRED_LEVEL: u32 = 9;
const REWARD_COINS: u32 = 60;
const MARKER_GUID: u64 = 100001236000;
const ENEMY_GUIDS: [u64; 5] = [
    100001236001,
    100001236002,
    100001236003,
    100001236004,
    100001236005,
];
/// `-34.75` is the bilinear zone-0 height, already on a decimetre node.
const MARKER: [f32; 3] = [-1903.0, -34.75, -660.0];
const LUGABOW: [f32; 3] = [-1915.146, -36.9307, -636.1354];
const GLUGMORT: [f32; 3] = [-1943.932, -39.94606, -600.8123];
const MUGGENSTOMP_NAME: i32 = 7058;
const BIG_BANDIT_NAME: i32 = 139345;
const BOSS_MODEL: i32 = 200;
const BANDIT_MODEL: i32 = 199;
const BOSS_BASE_HEALTH: u32 = 320;
const BOSS_BASE_DAMAGE: u32 = 22;
const BANDIT_BASE_HEALTH: u32 = 160;
const BANDIT_BASE_DAMAGE: u32 = 12;
/// The loader's difficulty-4 factor is the level-9 health growth; the live
/// arena health is the authored baseline times that factor and the combat
/// scale's enemy health scale.
fn boss_live_health() -> u32 {
    combat_scale::enemy_health_at(BOSS_BASE_HEALTH, 9)
}
fn bandit_live_health() -> u32 {
    combat_scale::enemy_health_at(BANDIT_BASE_HEALTH, 9)
}
const NAMEPLATE: i32 = 11970;
/// Images.txt 660: icon_ui_combatstart_panelgraphic_bandithideout.dds.
const BATTLE_ICON: i32 = 660;
const BRAWLER: i32 = 43;
const BATTLE_JOB_XP: u32 = 50;
const ARENA_ZONE: u32 = 9600;
const ARENA_ARRIVAL: [f32; 3] = [136.0, -0.5, 155.0];
const PRIZE_IDS: [i32; 4] = [10482, 3023, 75354, 11002];
/// The designated five-battle roster from `field_battle_roster`; the new row
/// is deliberately not part of it.
const TASK_ROSTER: [u32; 5] = [96, 12301, 12302, 12310, 12311];
const MINING_CONTENT: &str = "../../../lab/references/opening-journal-text-v2/content";
const REFUSAL_PRIZE: &str = "one of its rewards cannot be granted";
/// The exact refusal sentence is a local (R) copy in
/// `gameplay/encounter_rewards.rs` and has changed wording twice on the
/// integration tip; the stable contract is that the reward was not granted,
/// which the byte-equal economy assertion also proves.
const REWARD_REFUSED: &str = "not granted";

/// The 41/114 prize preview and roster fields this test decodes: the prize
/// bundle at 210 (count at +61, then 38-byte entries); the roster count sits
/// at 525 after the two i32 bundles and 69 zero bytes.
const DETAILS_PRIZE_BUNDLE: usize = 210;
const DETAILS_PRIZE_COUNT: usize = DETAILS_PRIZE_BUNDLE + 61;
const DETAILS_PRIZE_ENTRY: usize = 38;
const DETAILS_ROSTER: usize = 525;

// ---------------------------------------------------------------------------
// Local fixture seams: the combat-job bag, the capped wallet and the mining
// package. Everything else is `tests/common/field_battle_fixture.rs`.
// ---------------------------------------------------------------------------

impl Fixture {
    /// Restore a save built by the production `game_store` path with a bag of
    /// exactly `capacity` stacks. The seed has entered the world and is on a
    /// level-20 combat job.
    fn gateway_with_bag(
        &self,
        position: [f32; 3],
        guid: u64,
        capacity: usize,
    ) -> (Gateway, InventoryState) {
        self.gateway_with_bag_seeded(position, guid, capacity, BANDIT_HIDEOUT, |seed| {
            assert!(send(seed, &switch_job(BRAWLER), 300).state_changed);
            send(seed, &chat::local_request("realm level 20"), 350);
            assert_eq!(level(seed, BRAWLER), 20);
        })
    }

    /// A level-20 combat job reloaded through the same save path with a coin
    /// balance a save can hold but a 60-coin payout cannot add to.
    fn gateway_with_coins(&self, position: [f32; 3], guid: u64, balance: u32) -> Gateway {
        let mut seed = self.gateway(position, guid);
        assert!(send(&mut seed, &switch_job(BRAWLER), 300).state_changed);
        send(&mut seed, &chat::local_request("realm level 20"), 350);
        let gateway = reopen_with(self, &seed, "capped-wallet.json", 400, |state| {
            state.inventory.coins = balance;
        });
        assert_eq!(coins(&gateway), balance);
        gateway
    }

    /// A session on a private opening package with a live Miner and a bound
    /// mining game, for the busy refusal. It copies the package to the temp
    /// root first; no lab content is written.
    fn mining_gateway(&self, guid: u64) -> Gateway {
        let directory = self.root.join("mining-content");
        content_copy::copy_tree(&manifest_dir().join(MINING_CONTENT), &directory);
        osfr_tools::mining::write_content(&directory, MINING_GAME_ID).unwrap();
        let mut state = Gameplay::load(&directory, None).unwrap().state();
        let profiles = state.profiles.as_mut().unwrap();
        profiles.active = MINER_PROFILE_ID;
        profiles.unlocked.insert(MINER_PROFILE_ID);
        let mut character = Gateway::reference_state(&self.reference).unwrap();
        character.guid = guid;
        character.zone_id = 0;
        character.position = MARKER;
        character.rotation = [0.0, 0.0, 1.0];
        let mut gateway =
            Gateway::load_gameplay(&self.reference, character, &directory, Some(state)).unwrap();
        gateway.set_dev_tools(true);
        gateway.set_role(AccountRole::Admin);
        self.login(&mut gateway);
        send(&mut gateway, &13_u16.to_le_bytes(), 100);
        send(&mut gateway, &10_u16.to_le_bytes(), 200);
        gateway
    }
}

// ---------------------------------------------------------------------------
// Directed visibility: the 35/2 AddNpc the client reader 0x00921a10 parses.
// ---------------------------------------------------------------------------

fn has_add_npc(outcome: &GatewayOutcome, guid: u64) -> bool {
    outcome
        .replies
        .iter()
        .map(|packet| unframe(packet))
        .any(|packet| packet.starts_with(&[35, 0, 2, 0]) && packet[4..12] == guid.to_le_bytes())
}

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

/// The 167/1/1 list, decoded by the shared row reader.
fn published_list(replies: &[Vec<u8>]) -> Vec<PortalRow> {
    let packet = replies
        .iter()
        .map(|packet| unframe(packet))
        .find(|packet| packet.starts_with(&[167, 0, 1, 1]))
        .expect("a 167/1/1 activity list was sent");
    decode_list(packet)
}

fn next_instance(gateway: &Gateway) -> i32 {
    gateway
        .snapshot()
        .gameplay
        .unwrap()
        .encounter
        .unwrap()
        .next_instance_id
}

fn count_definition(gateway: &Gateway, definition_id: i32) -> u32 {
    gateway
        .snapshot()
        .gameplay
        .unwrap()
        .inventory
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id)
        .map(|item| item.count)
        .sum()
}

/// Decode the start screen's prize preview through the 41/114 layout:
/// (icon id, name id, quantity, item id) per bundle entry.
fn prize_preview(details: &[u8]) -> Vec<(i32, i32, i32, i32)> {
    let count = i32_at(details, DETAILS_PRIZE_COUNT);
    assert!(count >= 0 && count <= 4);
    (0..count as usize)
        .map(|index| {
            let entry = DETAILS_PRIZE_COUNT + 4 + index * DETAILS_PRIZE_ENTRY;
            (
                i32_at(details, entry + 5),
                i32_at(details, entry + 13),
                i32_at(details, entry + 17),
                i32_at(details, entry + 21),
            )
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

/// A level-8 combat job cannot start the battle through Go: 41/108 answers
/// with the shared level refusal, changes no saved state and leaves the same
/// pending screen usable after the job reaches level 9.
#[test]
fn level_eight_is_refused_and_keeps_the_pending_screen() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    let mut gateway = fixture.gateway(MARKER, 1001);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &chat::local_request("realm level 8"), 350);
    assert_eq!(level(&gateway, BRAWLER), REQUIRED_LEVEL - 1);

    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    let before = gateway.snapshot();
    let refused = send(&mut gateway, &go(&screen), 600);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(
        chat_text(&refused).contains("This activity needs level 9."),
        "{}",
        chat_text(&refused)
    );
    assert_eq!(
        gateway.snapshot(),
        before,
        "a level refusal consumes no open screen, coin, item or cooldown"
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
    // The 41/108 level branch answers in chat and leaves the pending screen in
    // place; it does not re-send 41/114. The identical Go starts once the job
    // reaches the floor, which proves the screen was not consumed.
    assert!(
        details(&refused).is_none(),
        "the level branch answers in chat without a second 41/114"
    );
    send(&mut gateway, &chat::local_request("realm level 9"), 700);
    assert_eq!(level(&gateway, BRAWLER), REQUIRED_LEVEL);
    let started = send(&mut gateway, &go(&screen), 800);
    assert!(started.began_zoning, "the pending screen still starts");
    assert_eq!(gateway.state().zone_id, ARENA_ZONE);
}

/// A level-9 combat job enters the arena: five actors spawn in zone 9600 with
/// the loader's single difficulty-4 scaling, and an explicit 41/109 exit
/// returns the character to the marker.
#[test]
fn level_nine_starts_the_battle() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    let mut gateway = fixture.gateway(MARKER, 1002);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &chat::local_request("realm level 9"), 350);
    assert_eq!(level(&gateway, BRAWLER), REQUIRED_LEVEL);

    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    let instance = i32_at(&screen, 8);
    let enemy_guids = enter_battle(&mut gateway, &screen, row, 1_000);
    assert_eq!(enemy_guids, ENEMY_GUIDS);
    let active = gateway
        .snapshot()
        .gameplay
        .unwrap()
        .encounter
        .unwrap()
        .active
        .unwrap();
    assert_eq!(active.encounter_id, BANDIT_HIDEOUT);
    assert_eq!(active.instance_id, instance);
    // The shared `enter_battle` only checks at-or-above the authored health
    // above difficulty 1, so pin the exact difficulty-4 scaling here.
    let health: Vec<u32> = active.enemies.iter().map(|enemy| enemy.health).collect();
    assert_eq!(
        health,
        [
            boss_live_health(),
            bandit_live_health(),
            bandit_live_health(),
            bandit_live_health(),
            bandit_live_health()
        ],
        "the loader scales the authored baseline once"
    );

    // 41/109 LeaveMiniGameOrEncounter: encounter 0 and the launch instance.
    let cancel = [
        vec![41, 0, 109, 0],
        0_i32.to_le_bytes().to_vec(),
        instance.to_le_bytes().to_vec(),
    ]
    .concat();
    let left = send(&mut gateway, &cancel, 2_000);
    assert!(left.began_zoning);
    assert_eq!(gateway.state().zone_id, 0);
    assert_eq!(gateway.state().position, MARKER);
    assert!(
        gateway
            .snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active
            .is_none(),
        "the exit clears the encounter, leaving no stuck busy state"
    );
}

/// The 41/114 start screen carries this row's instance, roster count, title,
/// icon, description and difficulty, and the four previewed prizes.
#[test]
fn the_start_screen_carries_the_instance_and_roster_count() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    let mut gateway = fixture.gateway(MARKER, 1003);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &chat::local_request("realm level 9"), 350);

    let before = next_instance(&gateway);
    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    assert_eq!(
        i32_at(&screen, 8),
        before,
        "the screen carries the instance"
    );
    assert_eq!(
        i32_at(&screen, DETAILS_ROSTER),
        row["enemies"].as_array().unwrap().len() as i32,
        "the screen carries the roster count"
    );
    let preview = prize_preview(&screen);
    assert_eq!(preview.len(), 4);
    for (entry, prize) in preview.iter().zip(row["prizes"].as_array().unwrap()) {
        assert_eq!(
            *entry,
            (
                i32_field(prize, "icon_id"),
                i32_field(prize, "name_id"),
                1,
                i32_field(prize, "item_id"),
            ),
            "the prize preview matches the row"
        );
    }
    // The client start screen has no level field: the level floor is enforced
    // server-side at 41/108 and reported in chat.
    assert!(screen.len() > DETAILS_ROSTER);

    let started = send(&mut gateway, &go(&screen), 600);
    assert!(started.began_zoning);
    assert_eq!(next_instance(&gateway), before + 1);
    let active = gateway
        .snapshot()
        .gameplay
        .unwrap()
        .encounter
        .unwrap()
        .active
        .unwrap();
    assert_eq!(active.instance_id, before);
    assert_eq!(active.enemies.len(), 5);
}

/// The full client route: marker 26/8, Go, five kills through 36/10, results
/// 39/18/47/19, then Close. It pays 60 coins, one prize and 50 job XP once;
/// replayed Go, wheel acknowledgement and Close pay nothing and the Close
/// restores the pre-entry pose.
#[test]
fn the_full_route_pays_sixty_coins_and_one_prize() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    let prizes = prize_ids(row);
    let mut gateway = fixture.gateway(MARKER, 1004);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &chat::local_request("realm level 20"), 350);
    let pose = (gateway.state().position, gateway.state().rotation);
    let before = economy(&gateway, &prizes);

    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    let enemy_guids = enter_battle(&mut gateway, &screen, row, 1_000);
    assert_eq!(enemy_guids, ENEMY_GUIDS);
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

    // A wheel acknowledgement (39/46), a repeated hit and the old Go are inert.
    let won = gateway.snapshot();
    let wheel = send(&mut gateway, &[39, 0, 46, 0], now + 700);
    assert!(!wheel.state_changed && wheel.replies.is_empty());
    let repeated = send(&mut gateway, &melee(enemy_guids[0]), now + 800);
    assert!(!repeated.state_changed, "a won fight cannot pay twice");
    let replay_go = send(&mut gateway, &go(&screen), now + 900);
    assert!(!replay_go.began_zoning && !replay_go.state_changed);
    assert_eq!(gateway.snapshot(), won);

    let closed = send(&mut gateway, &[26, 0, 42, 0], now + 1_400);
    assert!(closed.began_zoning, "Close returns to the open world");
    assert_eq!(gateway.state().zone_id, 0);
    assert_eq!(
        (gateway.state().position, gateway.state().rotation),
        pose,
        "the results Close restores the pre-entry pose"
    );
    assert_eq!(
        economy(&gateway, &prizes),
        paid,
        "leaving pays nothing more"
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
    let after_close = gateway.snapshot();
    let duplicate = send(&mut gateway, &[26, 0, 42, 0], now + 2_100);
    assert!(!duplicate.began_zoning && !duplicate.state_changed);
    assert_eq!(
        gateway.snapshot(),
        after_close,
        "a duplicate results Close changes nothing"
    );
}

/// The activity portal publishes the row automatically and applies the same
/// level floor and whole-bundle prize preflight as Go.
#[test]
fn the_portal_row_applies_the_same_level_and_prize_gates() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    assert_eq!(i32_field(row, "title_id"), TITLE);
    assert_eq!(u32_field(row, "difficulty"), DIFFICULTY);
    let mut gateway = fixture.gateway(MARKER, 1005);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &chat::local_request("realm level 8"), 350);
    let below = published_list(&send(&mut gateway, &[167, 0, 1, 4], 400).replies);
    let below = below
        .iter()
        .find(|row| row.id == BANDIT_HIDEOUT)
        .expect("the portal publishes Bandit Hideout");
    assert!(!below.can_join, "a level-8 combat job cannot join");
    assert_eq!(below.title, TITLE as u32);
    assert_eq!(below.description, DESCRIPTION as u32);
    assert_eq!(below.difficulty, DIFFICULTY);

    let before = (gateway.snapshot(), saved_bytes(&gateway));
    let refused = send(&mut gateway, &portal_play(BANDIT_HIDEOUT), 500);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(
        chat_text(&refused).contains("This activity needs level 9."),
        "{}",
        chat_text(&refused)
    );
    assert_eq!((gateway.snapshot(), saved_bytes(&gateway)), before);

    send(&mut gateway, &chat::local_request("realm level 9"), 600);
    let allowed = published_list(&send(&mut gateway, &[167, 0, 1, 4], 700).replies);
    assert!(
        allowed
            .iter()
            .find(|row| row.id == BANDIT_HIDEOUT)
            .is_some_and(|row| row.can_join),
        "a level-9 combat job may join"
    );
    let started = send(&mut gateway, &portal_play(BANDIT_HIDEOUT), 800);
    assert!(started.began_zoning);
    assert_eq!(gateway.state().zone_id, ARENA_ZONE);
    arena_spawned(&mut gateway, 850);
    let active = gateway
        .snapshot()
        .gameplay
        .unwrap()
        .encounter
        .unwrap()
        .active
        .unwrap();
    assert_eq!(active.encounter_id, BANDIT_HIDEOUT);
    assert_eq!(active.enemies.len(), 5);

    // The same Play on a full bag refuses at the whole-bundle preflight.
    let (mut full, bag) = fixture.gateway_with_bag(MARKER, 1006, MAX_ITEMS);
    assert_eq!(bag.items.len(), MAX_ITEMS);
    let before = (full.snapshot(), saved_bytes(&full));
    let refused = send(&mut full, &portal_play(BANDIT_HIDEOUT), 600);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(
        chat_text(&refused).contains(REFUSAL_PRIZE),
        "{}",
        chat_text(&refused)
    );
    assert_eq!((full.snapshot(), saved_bytes(&full)), before);
    assert!(
        full.snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active
            .is_none()
    );

    let (mut fitting, bag) = fixture.gateway_with_bag(MARKER, 1007, MAX_ITEMS - 4);
    assert_eq!(bag.items.len(), MAX_ITEMS - 4);
    let started = send(&mut fitting, &portal_play(BANDIT_HIDEOUT), 900);
    assert!(started.began_zoning, "the fitting bundle starts the battle");
    assert_eq!(fitting.state().zone_id, ARENA_ZONE);
}

/// Forged, out-of-range, knocked-out, full-bag and already-consumed requests
/// change no saved state; the controls prove the same requests start when the
/// qualifier holds.
#[test]
fn refusals_leave_state_and_pending_battle_unchanged() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    let prizes = prize_ids(row);

    // Forged Go with no open screen.
    let mut gateway = fixture.gateway(MARKER, 2001);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &chat::local_request("realm level 20"), 350);
    let before = (gateway.snapshot(), saved_bytes(&gateway));
    let mut forged = vec![41, 0, 108, 0];
    forged.extend(BANDIT_HIDEOUT.to_le_bytes());
    forged.extend(0x7fff_i32.to_le_bytes());
    forged.extend(0_u64.to_le_bytes());
    let refused = send(&mut gateway, &forged, 400);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(refused.replies.is_empty(), "a forged Go has no reply");
    assert_eq!((gateway.snapshot(), saved_bytes(&gateway)), before);

    // Wrong marker: a forged actor GUID opens no start screen.
    let wrong = send(&mut gateway, &interact(0xdead_beef), 500);
    assert!(details(&wrong).is_none());
    assert!(!wrong.state_changed && !wrong.began_zoning);
    assert_eq!((gateway.snapshot(), saved_bytes(&gateway)), before);

    // Out-of-range Go: the screen opens at the marker, the player walks 50 m
    // away and Go is inert. The out-of-reach screen is dropped, so the same Go
    // stays inert after walking back; reopening at the marker works.
    let mut gateway = fixture.gateway(MARKER, 2002);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &chat::local_request("realm level 20"), 350);
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
    assert_eq!((gateway.snapshot(), saved_bytes(&gateway)), before);
    walk(&mut gateway, player, MARKER, &mut now);
    let replay = send(&mut gateway, &go(&screen), now + 100);
    assert!(
        !replay.began_zoning && !replay.state_changed,
        "an out-of-reach screen was dropped, not kept for a later Go"
    );
    let screen = open_battle(&mut gateway, MARKER_GUID, row, now + 200);
    assert!(send(&mut gateway, &go(&screen), now + 300).began_zoning);

    // Knocked out: Go is a chat refusal that reopens the same screen.
    let mut knocked = fixture.gateway(MARKER, 2003);
    assert!(send(&mut knocked, &switch_job(BRAWLER), 300).state_changed);
    send(&mut knocked, &chat::local_request("realm level 20"), 350);
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
    assert_eq!((knocked.snapshot(), saved_bytes(&knocked)), before);

    // Full bag: the four-prize bundle has no room, so Go is refused and the
    // same pending screen is re-sent; a replay refuses identically.
    let (mut full, bag) = fixture.gateway_with_bag(MARKER, 2004, MAX_ITEMS);
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
    let shown = details(&refused)
        .expect("the refused start screen remains open")
        .to_vec();
    assert_eq!((full.snapshot(), saved_bytes(&full)), before);
    let repeated = send(&mut full, &go(&screen), 800);
    assert!(!repeated.began_zoning && !repeated.state_changed);
    assert!(
        chat_text(&repeated).contains(REFUSAL_PRIZE),
        "{}",
        chat_text(&repeated)
    );
    assert_eq!(
        details(&repeated).expect("a refused Go keeps the start screen pending"),
        shown,
        "the same start screen is shown again"
    );

    // Already consumed: once the arena is live, the old Go is inert.
    let (mut fitting, _) = fixture.gateway_with_bag(MARKER, 2005, MAX_ITEMS - 4);
    let screen = open_battle(&mut fitting, MARKER_GUID, row, 800);
    assert!(send(&mut fitting, &go(&screen), 900).began_zoning);
    arena_spawned(&mut fitting, 950);
    let before = (fitting.snapshot(), saved_bytes(&fitting));
    let replay = send(&mut fitting, &go(&screen), 1_000);
    assert!(!replay.began_zoning && !replay.state_changed);
    assert!(
        details(&replay).is_none(),
        "the consumed screen stays closed"
    );
    assert_eq!((fitting.snapshot(), saved_bytes(&fitting)), before);
}

/// Two independent sessions run their own instance, kills and payout; one
/// player's win neither removes nor damages the other's actors.
#[test]
fn a_second_player_runs_its_own_instance_and_is_paid_once() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    let prizes = prize_ids(row);
    let mut first = fixture.gateway(MARKER, 3001);
    let mut second = fixture.gateway(MARKER, 3002);
    for session in [&mut first, &mut second] {
        assert!(send(session, &switch_job(BRAWLER), 300).state_changed);
        send(session, &chat::local_request("realm level 20"), 350);
    }
    let first_before = economy(&first, &prizes);
    let second_before = economy(&second, &prizes);
    let first_next = next_instance(&first);
    let second_next = next_instance(&second);

    let first_screen = open_battle(&mut first, MARKER_GUID, row, 500);
    let second_screen = open_battle(&mut second, MARKER_GUID, row, 500);
    assert_eq!(i32_at(&first_screen, 8), first_next);
    assert_eq!(i32_at(&second_screen, 8), second_next);
    enter_battle(&mut first, &first_screen, row, 600);
    enter_battle(&mut second, &second_screen, row, 600);
    assert_eq!(next_instance(&first), first_next + 1);
    assert_eq!(next_instance(&second), second_next + 1);

    let second_active = second
        .snapshot()
        .gameplay
        .unwrap()
        .encounter
        .unwrap()
        .active
        .expect("the second run is live");
    assert_eq!(second_active.instance_id, second_next);
    assert_eq!(second_active.enemies.len(), 5);

    assert!(send(&mut first, &chat::local_request("realm win"), 800).state_changed);
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
        Some(second_active),
        "the first player's win never removes or damages the second's actors"
    );

    assert!(send(&mut second, &chat::local_request("realm win"), 900).state_changed);
    let second_paid = economy(&second, &prizes);
    assert_payout(&second_before, &second_paid, row, &prizes, BATTLE_JOB_XP);
    assert_eq!(
        economy(&first, &prizes),
        first_paid,
        "the second player's win never pays the first"
    );
}

/// A mid-fight save/reload drops the run and returns to the marker without a
/// payout; a won save/reload keeps the coins and the prize and cannot pay the
/// old win again.
#[test]
fn the_win_and_the_mid_fight_return_survive_a_reload() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    let prizes = prize_ids(row);

    let mut gateway = fixture.gateway(MARKER, 4001);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &chat::local_request("realm level 20"), 350);
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

    let mut gateway = fixture.gateway(MARKER, 4002);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &chat::local_request("realm level 20"), 350);
    let before = economy(&gateway, &prizes);
    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    enter_battle(&mut gateway, &screen, row, 600);
    let won = send(&mut gateway, &chat::local_request("realm win"), 800);
    assert!(won.state_changed);
    let paid = economy(&gateway, &prizes);
    assert_payout(&before, &paid, row, &prizes, BATTLE_JOB_XP);
    let mut dropped = reopen(&fixture, &gateway, "won-unclosed.json", 2_000);
    assert_eq!(economy(&dropped, &prizes), paid);
    assert_eq!(dropped.state().zone_id, 0);
    assert_eq!(dropped.state().position, MARKER);
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
    let before_replay = saved_bytes(&dropped);
    let replay = send(&mut dropped, &chat::local_request("realm win"), 2_100);
    assert!(!replay.state_changed, "a reloaded win pays nothing");
    assert_eq!(saved_bytes(&dropped), before_replay);
}

/// Every previewed prize resolves against the shipped catalogue, fits one
/// stack, and the real loader accepts the row's bounds.
#[test]
fn every_previewed_prize_resolves_for_the_shipped_catalogue() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    let inventory = runtime_inventory();
    let prizes = row["prizes"].as_array().unwrap();
    assert_eq!(prizes.len(), 4);
    assert_eq!(
        prize_ids(row),
        PRIZE_IDS.into_iter().collect::<BTreeSet<_>>()
    );
    for prize in prizes {
        let item = i32_field(prize, "item_id");
        let quantity = prize["quantity"].as_u64().unwrap_or(1) as u32;
        assert!(i32_field(prize, "name_id") > 0);
        assert!(i32_field(prize, "icon_id") > 0);
        assert!(
            inventory.accepts_reward(item, quantity),
            "prize {item} cannot be granted"
        );
    }
    let mut engine = EncounterEngine::load(&fixture.content, None).unwrap();
    engine
        .load_field_battles(&battle_path())
        .expect("the real loader accepts the new row");
    assert!(
        engine.battle_prizes_available(BANDIT_HIDEOUT, |item, quantity| {
            inventory.accepts_reward(item, quantity)
        }),
        "T243's whole-bundle preflight accepts the four prizes"
    );
    assert_eq!(
        engine.battle_spawners().len(),
        file["battles"].as_array().unwrap().len(),
        "every published battle row supplies its marker spawner"
    );
}

/// The new GUIDs collide with no battle, field actor or encounter actor, and
/// the loader refuses a duplicate GUID or a duplicate id.
#[test]
fn guids_are_unique_across_the_shipped_content() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    let mut own = BTreeSet::new();
    for actor in std::iter::once(&row["spawner"]).chain(row["enemies"].as_array().unwrap().iter()) {
        let guid = u64_field(actor, "guid");
        assert!(guid > 1);
        assert!(own.insert(guid), "the new row repeats GUID {guid}");
    }
    let mut all = own.clone();
    for other in file["battles"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|other| other["id"] != json!(BANDIT_HIDEOUT))
    {
        for actor in
            std::iter::once(&other["spawner"]).chain(other["enemies"].as_array().unwrap().iter())
        {
            assert!(
                all.insert(u64_field(actor, "guid")),
                "a shipped battle repeats GUID {}",
                u64_field(actor, "guid")
            );
        }
    }
    let field = read_json(&content_dir().join("field_enemies.json"));
    for enemy in field["enemies"].as_array().unwrap() {
        let guid = u64_field(enemy, "guid");
        assert!(
            !own.contains(&guid),
            "field enemy {guid} reuses a Bandit Hideout GUID"
        );
    }
    let encounters = read_json(&content_dir().join("encounters.json"));
    for dungeon in encounters["dungeons"].as_array().unwrap() {
        assert_ne!(dungeon["id"].as_u64().unwrap() as u32, BANDIT_HIDEOUT);
        for enemy in dungeon["enemies"].as_array().unwrap() {
            let guid = u64_field(enemy, "guid");
            assert!(
                !own.contains(&guid),
                "encounter actor {guid} reuses a Bandit Hideout GUID"
            );
        }
        if let Some(guide) = dungeon["guide"].as_object() {
            let guid = guide["guid"].as_u64().unwrap();
            assert!(!own.contains(&guid), "encounter guide {guid} collides");
        }
    }
    let mut engine = EncounterEngine::load(&fixture.content, None).unwrap();
    engine
        .load_field_battles(&battle_path())
        .expect("the real loader accepts the file");

    // A duplicate GUID and a duplicate row are refused at load.
    let mut collision = file.clone();
    collision["battles"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["id"] == json!(BANDIT_HIDEOUT))
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
    let copied = battle(&duplicate, BANDIT_HIDEOUT).clone();
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

/// A live mining movie refuses the marker click and the portal Play through the
/// existing busy table, and changes nothing.
#[test]
fn a_live_mining_movie_refuses_the_battle() {
    let fixture = Fixture::new();
    let mut gateway = fixture.mining_gateway(5001);
    let started = send(&mut gateway, &chat::local_request("realm mine"), 1_000);
    assert!(
        chat_text(&started).contains("Mining test started"),
        "{}",
        chat_text(&started)
    );
    assert!(
        gateway
            .snapshot()
            .gameplay
            .unwrap()
            .mining
            .is_some_and(|mining| mining.active.is_some()),
        "the mining movie is live"
    );

    let before = (gateway.snapshot(), saved_bytes(&gateway));
    let click = send(&mut gateway, &interact(MARKER_GUID), 1_100);
    assert!(!click.began_zoning && !click.state_changed);
    assert!(
        chat_text(&click).contains("Close the mining game"),
        "{}",
        chat_text(&click)
    );
    assert_eq!((gateway.snapshot(), saved_bytes(&gateway)), before);

    let play = send(&mut gateway, &portal_play(BANDIT_HIDEOUT), 1_200);
    assert!(!play.began_zoning && !play.state_changed);
    assert!(
        chat_text(&play).contains("Finish what you are doing before starting an activity."),
        "{}",
        chat_text(&play)
    );
    assert_eq!((gateway.snapshot(), saved_bytes(&gateway)), before);
    assert!(
        gateway
            .snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active
            .is_none(),
        "the refused battle never starts"
    );
}

/// A second independent session sees the Bandit Hideout marker in zone 0 and
/// never the arena actors; a late joiner sees the same marker.
#[test]
fn a_second_session_sees_the_marker_not_the_arena() {
    let fixture = Fixture::new();
    let (mut first, first_entry) = fixture.session(MARKER, 6001);
    let (mut second, second_entry) = fixture.session(MARKER, 6002);
    let first_packet = marker_packet(&first_entry.replies, MARKER_GUID);
    let second_packet = marker_packet(&second_entry.replies, MARKER_GUID);
    assert_eq!(
        first_packet, second_packet,
        "the marker is identical for both sessions (no per-player actor)"
    );
    let spawn = marker_spawn(first_packet);
    assert_eq!(spawn.actor_id, MARKER_GUID);
    assert_eq!(spawn.name_string_id, MUGGENSTOMP_NAME);
    assert_eq!(spawn.model_id, BOSS_MODEL);
    assert_eq!(spawn.position[0], MARKER[0]);
    assert_eq!(spawn.position[1], MARKER[1]);
    assert_eq!(spawn.position[2], MARKER[2]);
    assert_eq!(spawn.nameplate_image_id, NAMEPLATE);
    for guid in ENEMY_GUIDS {
        assert!(
            !has_add_npc(&first_entry, guid) && !has_add_npc(&second_entry, guid),
            "arena enemy {guid} is not sent at world entry"
        );
    }

    assert!(send(&mut first, &switch_job(BRAWLER), 300).state_changed);
    send(&mut first, &chat::local_request("realm level 20"), 350);
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    let screen = open_battle(&mut first, MARKER_GUID, row, 500);
    let entered = send(&mut first, &go(&screen), 600);
    assert!(entered.began_zoning);
    let arena = arena_spawned(&mut first, 700);
    for guid in ENEMY_GUIDS {
        assert!(
            has_add_npc(&arena, guid),
            "the owner sees arena enemy {guid}"
        );
    }

    // The second session's world is unchanged by the first player's arena:
    // its refresh carries only deltas, so it has no arena AddNpc.
    let again = arena_spawned(&mut second, 700);
    for guid in ENEMY_GUIDS {
        assert!(
            !has_add_npc(&again, guid),
            "the first player's arena enemy {guid} is not aliased into the second stream"
        );
    }
    assert_eq!(
        marker_packet(&second_entry.replies, MARKER_GUID),
        first_packet
    );

    // A late joiner gets the same static marker and no arena actors.
    let (_late, late_entry) = fixture.session(MARKER, 6003);
    assert_eq!(
        marker_packet(&late_entry.replies, MARKER_GUID),
        first_packet
    );
    for guid in ENEMY_GUIDS {
        assert!(!has_add_npc(&late_entry, guid));
    }
}

/// The win pays from the live bag, coins and profiles: a dev grant between Go
/// and the killing blow survives the payout, which adds its own 60 coins and
/// one prize.
#[test]
fn a_live_change_before_the_close_survives_the_payout() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    let prizes = prize_ids(row);
    let filler = filler_definition(BANDIT_HIDEOUT);
    assert!(!prizes.contains(&filler));
    let mut gateway = fixture.gateway(MARKER, 7001);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &chat::local_request("realm level 20"), 350);
    let before = economy(&gateway, &prizes);
    let filler_before = count_definition(&gateway, filler);

    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    enter_battle(&mut gateway, &screen, row, 600);
    let coins = send(&mut gateway, &chat::local_request("realm coins 7"), 700);
    assert!(
        chat_text(&coins).contains("Gave 7 coins"),
        "{}",
        chat_text(&coins)
    );
    let given = send(
        &mut gateway,
        &chat::local_request(&format!("realm give {filler} 2")),
        800,
    );
    assert!(
        chat_text(&given).contains("Gave 2 x item"),
        "{}",
        chat_text(&given)
    );
    assert_eq!(count_definition(&gateway, filler), filler_before + 2);

    let mut now = 1_000;
    let replies = fight_to_win(&mut gateway, &mut now, row);
    assert_eq!(results_lifecycle(&replies), vec![18, 47, 19]);
    let paid = economy(&gateway, &prizes);
    assert_eq!(
        paid.coins,
        before.coins + 7 + u32_field(row, "reward_coins"),
        "the payout starts from the live wallet"
    );
    assert_eq!(
        count_definition(&gateway, filler),
        filler_before + 2,
        "the live item grant is not restored over"
    );
    assert_eq!(
        paid.experience.get(&BRAWLER).copied().unwrap_or(0),
        before.experience.get(&BRAWLER).copied().unwrap_or(0) + BATTLE_JOB_XP,
        "the profile is paid live"
    );
    let gains: Vec<(i32, u32)> = paid
        .prizes
        .iter()
        .filter_map(|(id, count)| {
            let prior = before.prizes.get(id).copied().unwrap_or(0);
            (count > &prior).then_some((*id, count - prior))
        })
        .collect();
    assert_eq!(gains.len(), 1, "exactly one prize is added to the live bag");

    let closed = send(&mut gateway, &[26, 0, 42, 0], now + 200);
    assert!(closed.began_zoning);
    assert_eq!(economy(&gateway, &prizes), paid);
    assert_eq!(count_definition(&gateway, filler), filler_before + 2);
}

/// A refused start and an unrepresentable payout in one session leave the
/// other session running; neither error escapes the gateway.
#[test]
fn a_failing_start_or_payout_does_not_stop_the_other_session() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    let prizes = prize_ids(row);

    // Session A: the full-bag preflight refuses the start.
    let (mut failing, _) = fixture.gateway_with_bag(MARKER, 8001, MAX_ITEMS);
    let before = (failing.snapshot(), saved_bytes(&failing));
    let refused = send(&mut failing, &portal_play(BANDIT_HIDEOUT), 600);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(chat_text(&refused).contains(REFUSAL_PRIZE));
    assert_eq!((failing.snapshot(), saved_bytes(&failing)), before);

    // Session B runs and is paid normally.
    let mut healthy = fixture.gateway(MARKER, 8002);
    assert!(send(&mut healthy, &switch_job(BRAWLER), 300).state_changed);
    send(&mut healthy, &chat::local_request("realm level 20"), 350);
    let healthy_before = economy(&healthy, &prizes);
    let started = send(&mut healthy, &portal_play(BANDIT_HIDEOUT), 600);
    assert!(started.began_zoning);
    arena_spawned(&mut healthy, 650);
    assert!(send(&mut healthy, &chat::local_request("realm win"), 700).state_changed);
    let healthy_paid = economy(&healthy, &prizes);
    assert_payout(&healthy_before, &healthy_paid, row, &prizes, BATTLE_JOB_XP);

    // Session C cannot represent the payout: the win refuses the reward alone.
    let mut capped = fixture.gateway_with_coins(MARKER, 8003, i32::MAX as u32);
    let capped_before = economy(&capped, &prizes);
    let started = send(&mut capped, &portal_play(BANDIT_HIDEOUT), 600);
    assert!(started.began_zoning);
    arena_spawned(&mut capped, 650);
    let won = send(&mut capped, &chat::local_request("realm win"), 700);
    assert!(won.state_changed);
    assert!(
        chat_text(&won).contains(REWARD_REFUSED),
        "{}",
        chat_text(&won)
    );
    assert_eq!(
        economy(&capped, &prizes),
        capped_before,
        "an unrepresentable payout leaves the capped session's economy byte-equal"
    );
    let closed = send(&mut capped, &[26, 0, 42, 0], 800);
    assert!(closed.began_zoning);
    assert_eq!(capped.state().zone_id, 0);
    // The client sends the scene-ready pair after every zoning reply.
    arena_spawned(&mut capped, 900);

    // Neither failing session disturbed the healthy one, and all still answer.
    assert_eq!(economy(&healthy, &prizes), healthy_paid);
    for (name, session) in [
        ("failing", &mut failing),
        ("healthy", &mut healthy),
        ("capped", &mut capped),
    ] {
        let refreshed = send(session, &[167, 0, 1, 4], 1_500);
        let list = refreshed
            .replies
            .iter()
            .map(|packet| unframe(packet))
            .find(|packet| packet.starts_with(&[167, 0, 1, 1]));
        let rows = list.map(decode_list).unwrap_or_default();
        assert!(
            rows.iter().any(|row| row.id == BANDIT_HIDEOUT),
            "{name} did not answer the portal refresh: {:?}",
            refreshed
                .replies
                .iter()
                .map(|packet| {
                    let packet = unframe(packet);
                    packet.get(..4).unwrap_or_default().to_vec()
                })
                .collect::<Vec<_>>()
        );
    }
}

/// The row's identity, roster and zone match the recovered client text and
/// models, and the marker is on the sampled ground, clear of every spawn and
/// other marker.
#[test]
fn the_marker_is_on_the_ground_and_clear() {
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    assert_eq!(u32_field(row, "title_id"), TITLE as u32);
    assert_eq!(u32_field(row, "description_id"), DESCRIPTION as u32);
    assert_eq!(u32_field(row, "objective_id"), OBJECTIVE as u32);
    assert_eq!(u32_field(row, "difficulty"), DIFFICULTY);
    assert_eq!(u32_field(row, "required_combat_level"), REQUIRED_LEVEL);
    assert_eq!(u32_field(row, "reward_coins"), REWARD_COINS);
    assert_eq!(i32_field(row, "icon_id"), BATTLE_ICON);
    assert_eq!(i32_field(row, "nameplate_image_id"), NAMEPLATE);
    let marker = position(&row["spawner"]["position"]);
    assert_eq!(marker, MARKER);
    assert_eq!(i32_field(&row["spawner"], "name_id"), MUGGENSTOMP_NAME);
    assert_eq!(i32_field(&row["spawner"], "model_id"), BOSS_MODEL);
    assert_eq!(row["spawner"]["boss"], false);
    assert_eq!(u32_field(&row["spawner"], "max_health"), 1);
    assert_eq!(u32_field(&row["spawner"], "damage"), 0);
    let lua_dist = (marker[0] - LUGABOW[0]).hypot(marker[2] - LUGABOW[2]);
    assert!(
        (15.0..=40.0).contains(&lua_dist),
        "the marker is {lua_dist:.2} m from Lugabow, on the camp approach"
    );
    assert!(lua_dist < (marker[0] - GLUGMORT[0]).hypot(marker[2] - GLUGMORT[2]));

    let enemies = row["enemies"].as_array().unwrap();
    assert_eq!(enemies.len(), 5);
    let boss = &enemies[0];
    assert_eq!(u32_field(boss, "max_health"), BOSS_BASE_HEALTH);
    assert_eq!(u32_field(boss, "damage"), BOSS_BASE_DAMAGE);
    assert_eq!(boss["boss"], true);
    assert_eq!(i32_field(boss, "model_id"), BOSS_MODEL);
    assert_eq!(i32_field(boss, "name_id"), MUGGENSTOMP_NAME);
    assert_eq!(position(&boss["position"]), [136.0, -0.6, 175.0]);
    for (index, enemy) in enemies[1..].iter().enumerate() {
        assert_eq!(u32_field(enemy, "max_health"), BANDIT_BASE_HEALTH);
        assert_eq!(u32_field(enemy, "damage"), BANDIT_BASE_DAMAGE);
        assert_eq!(enemy["boss"], false);
        assert_eq!(i32_field(enemy, "model_id"), BANDIT_MODEL);
        assert_eq!(i32_field(enemy, "name_id"), BIG_BANDIT_NAME);
        assert_eq!(u64_field(enemy, "guid"), 100001236002 + index as u64);
    }
    assert_eq!(
        enemies
            .iter()
            .map(|enemy| position(&enemy["position"]))
            .collect::<Vec<_>>(),
        [
            [136.0, -0.6, 175.0],
            [133.0, -0.6, 173.0],
            [139.0, -0.6, 173.0],
            [134.0, -0.6, 176.0],
            [138.0, -0.6, 176.0],
        ]
    );

    let zone = &row["zone"];
    assert_eq!(u32_field(zone, "zone_id"), ARENA_ZONE);
    assert_eq!(zone["name"], "sg_random_encounter_clearing");
    assert_eq!(zone["sky"], "sky_wilds24.xml");
    assert_eq!(u32_field(zone, "geometry_id"), 214);
    assert_eq!(position(&zone["position"]), ARENA_ARRIVAL);
    assert_eq!(position(&zone["rotation"]), [0.0, 0.0, 0.0]);
    assert_eq!(zone["tutorial"], false);

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
    let disc = disc_points(marker[0], marker[2]);
    assert_eq!(disc.len(), 105);
    for (x, z) in &disc {
        assert!(
            grid.height_at(*x, *z).is_some(),
            "disc sample ({x},{z}) has no ground"
        );
    }
    let max_slope = disc
        .iter()
        .map(|(x, z)| slope_degrees(grid, *x, *z))
        .fold(0.0_f32, f32::max);
    assert!(
        max_slope <= 25.0,
        "the full 5 m wander disc reaches {max_slope:.2} degrees"
    );
    let (npc, npc_id) = nearest_spawned_npc(marker);
    assert!(
        npc >= 15.0,
        "nearest spawned NPC {npc_id} is {npc:.2} m away"
    );
    let (extra, extra_id) = nearest_extra_spawn(marker);
    assert!(
        extra >= 15.0,
        "nearest npc_spawns_extra actor {extra_id} is {extra:.2} m away"
    );
    let enemy = nearest_field_enemy(marker);
    assert!(enemy >= 20.0, "nearest field enemy is {enemy:.1} m away");
    let other = nearest_other_battle_marker(marker, BANDIT_HIDEOUT);
    assert!(
        other >= 100.0,
        "the nearest battle marker is {other:.1} m away, so the marker stays well clear of Thugamug"
    );

    let arena = terrain.grid(ARENA_ZONE).expect("zone 9600 is loaded");
    assert!(
        arena
            .height_at(ARENA_ARRIVAL[0], ARENA_ARRIVAL[2])
            .is_some(),
        "the arena arrival is inside the zone-9600 grid"
    );

    // Client evidence: locale rows, both bandit models and the two images.
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
        names.get(&TITLE).map(String::as_str),
        Some("Bandit Hideout!")
    );
    assert_eq!(
        names.get(&DESCRIPTION).map(String::as_str),
        Some(
            "An unruly band of thugawugs are inside the Bandit Hideout! Defeat their leader, Muggenstomp, to end their greedy ways!"
        )
    );
    assert_eq!(
        names.get(&OBJECTIVE).map(String::as_str),
        Some("Defeat Muggenstomp!")
    );
    assert_eq!(
        names.get(&MUGGENSTOMP_NAME).map(String::as_str),
        Some("Muggenstomp")
    );
    assert_eq!(
        names.get(&BIG_BANDIT_NAME).map(String::as_str),
        Some("Big Bandit")
    );
    let models =
        fs::read_to_string(lab("working/OSFR.Client/OSFR Client/Resources/Models.txt")).unwrap();
    assert!(
        models
            .lines()
            .any(|line| line.starts_with("200^chugawug_f_bandit_01.adr^")),
        "model 200 is the female bandit body"
    );
    assert!(
        models
            .lines()
            .any(|line| line.starts_with("199^chugawug_m_bandit_01.adr^")),
        "model 199 is the male bandit body"
    );
    let images = fs::read_to_string(lab(
        "working/OSFR.Client/OSFR Client/Resources/Images/Images.txt",
    ))
    .unwrap();
    assert!(
        images
            .lines()
            .any(|line| line.starts_with("660^icon_ui_combatstart_panelgraphic_bandithideout.dds^")),
        "the start-panel graphic is this battle's own client image"
    );
    assert!(
        images
            .lines()
            .any(|line| line
                .starts_with("11970^icon_minigame_combat_encounter_start_hard_free_64.dds^")),
        "the shared battle marker icon is a client image"
    );
    let catalog = fs::read_to_string(lab("extracted/client-assets/catalog.tsv")).unwrap();
    assert!(catalog.contains("\tchugawug_f_bandit_01.adr\t"));
    assert!(catalog.contains("\tchugawug_m_bandit_01.adr\t"));
}

/// Every earlier row keeps its title, difficulty, required level and payout;
/// the new row is the only addition and sits after battle 12400.
#[test]
fn the_shipped_rows_are_unchanged_and_the_new_row_is_appended() {
    let file = battles();
    let expected: [(u32, u32, u32, u32, u32); 9] = [
        (96, 42056, 1, 0, 25),
        (12340, 90703, 3, 5, 25),
        (12301, 75452, 1, 0, 25),
        (12302, 42054, 2, 0, 25),
        (12303, 426971, 5, 13, 0),
        (12350, 30404, 1, 0, 25),
        (12310, 93274, 2, 0, 25),
        (12311, 141039, 2, 0, 25),
        (12400, 24276, 1, 0, 19),
    ];
    let ids: Vec<u32> = file["battles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].as_u64().unwrap() as u32)
        .collect();
    // Concurrent content slices insert their own rows (the Bixie Hive slice
    // after battle 96), so pin every shipped row's values and relative order
    // plus this row's place after 12400, not the whole file.
    let mut previous = None;
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
        let at = ids
            .iter()
            .position(|candidate| *candidate == id)
            .unwrap_or_else(|| panic!("battle {id} was removed"));
        if let Some(previous) = previous {
            assert!(previous < at, "battle {id} was re-ordered");
        }
        previous = Some(at);
    }
    let existing_last = previous.expect("the shipped rows are non-empty");
    let new_at = ids
        .iter()
        .position(|id| *id == BANDIT_HIDEOUT)
        .expect("the new row is published");
    assert_eq!(
        ids.iter().filter(|id| **id == BANDIT_HIDEOUT).count(),
        1,
        "the new row is published exactly once"
    );
    assert!(
        new_at > existing_last,
        "the new row is appended after battle 12400"
    );
    assert!(!TASK_ROSTER.contains(&BANDIT_HIDEOUT));
    assert_eq!(ids.iter().filter(|id| TASK_ROSTER.contains(id)).count(), 5);

    // No dungeon data file is added: the runtime row is the playable backing.
    // Later content slices may add their own dungeon files, so only pin that
    // this slice removed none; the new row itself is not an encounters.json id.
    let dungeon_files = fs::read_dir(content_dir().join("dungeons"))
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "json")
        })
        .count();
    assert!(
        dungeon_files >= 75,
        "a pre-existing dungeon data file was removed"
    );
    let encounters = read_json(&content_dir().join("encounters.json"));
    assert!(
        !encounters["dungeons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|dungeon| dungeon["id"].as_u64() == Some(u64::from(BANDIT_HIDEOUT)))
    );
}
