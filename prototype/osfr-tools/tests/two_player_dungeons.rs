//! T373/F396: two players run the same field-battle marker independently.
//!
//! Existing suites already prove most of the per-session model with two
//! in-process gateways: `sheep_watch_dungeon.rs` (the win/disconnect block at
//! lines 858-928) checks that one player's win and reconnect never pay or
//! change the other, `bixie_hive_dungeon.rs`
//! (`a_second_player_runs_its_own_instance_and_is_paid_once`) checks two runs
//! and payouts, and `bixie_hive_dungeon.rs`
//! (`a_second_session_sees_the_marker_not_the_arena`) checks that a second and
//! a late session receive only the static marker. This suite keeps only what
//! those do not assert:
//!
//! * two seeded saves (41 and 108) each name their own instance on the start
//!   screen and in the active run, and each start screen matches the row
//!   byte-for-byte through the client reader (`assert_start_screen`);
//! * each arena's five actors reach the client through the production 35/2
//!   AddNpc batch (`enter_battle`);
//! * two different pre-entry poses each restore their own player on Close;
//! * forged, out-of-range, replayed and cross-session Go/Close packets change
//!   no byte (snapshot + save payload equality);
//! * a malformed packet is refused for one session alone;
//! * one player's win or knockout does not change the other's live run, save,
//!   quest counters or wallet;
//! * a third late joiner sees the static marker only and starts its own run.
//!
//! Scope: two or three in-process `Gateway` values on the production
//! `Gateway::handle_at` dispatch (`handle_at` is also the server's packet
//! dispatch). The shared-world fanout after that (`Server::step`, `world` /
//! `players` interest, `contain`) is owned by the `osfr_gateway` inline suites
//! (`isolation.rs`, `t020_group_routing.rs`); this file does not claim it.
//!
//! Client shapes (IDA, 2014 client; unchanged by this suite):
//! * 26/8 interaction constructor `0x008b6bc0` = target u64 + bool, the
//!   marker click.
//! * 41/114 start/running screen reader chain
//!   `0x00aa3430 -> 0x00aa32d0 -> 0x00a29120 -> 0x009bdd70`: encounter id at
//!   bytes 4..8, instance at 8..12, title/icon/description/difficulty at
//!   47..71 (`assert_start_screen`).
//! * 41/108 Go constructor `0x008b6e70`, header only; the server starts only
//!   its own pending screen.
//! * 36/10 attack writer `0x00a342e0` / reader `0x00a35150`: bar 1, slot 0,
//!   target mode 0, target GUID (the shared `melee` helper).
//! * 35/2 AddNpc reader `0x00921a10` reads the u64 actor id first; the shared
//!   `enter_battle` checks every row actor there.
//! * 26/42 results Close constructor `0x008a2eb0`, sender `0x0092f300`.
//! * 26/10 (`0x00a36f80`) is the NPC dialogue/choice request, **not** a
//!   field-battle step, so the marker route stays 26/8 -> 41/114 -> 41/108.

use osfr_tools::{
    account_store::AccountRole,
    chat,
    encounter::{ActiveEncounter, EncounterPhase},
    gateway::Gateway,
    quest_engine::{CharacterEligibility, QuestEngine, QuestProgress, QuestState, QuestStatus},
    startup_world::parse_add_npc,
};
use serde_json::Value;
use std::collections::BTreeMap;

#[path = "common/field_battle_fixture.rs"]
mod field_battle_fixture;

use field_battle_fixture::*;

/// T319's authored Sheep Watch row.
const SHEEP_WATCH: u32 = 12350;
/// The Brawler job profile every session switches to so it can start a battle.
const BRAWLER: i32 = 43;
/// The shipped `Brawler: Delinquent Dilemma` defeat goal (kind `930050`).
const DELINQUENT_QUEST: u32 = 3030007;
/// (R) `encounter.rs::BATTLE_JOB_XP` (half a rank on the local curve): the
/// field-battle engine policy, not a content row field. The same value the
/// other field-battle suites assert.
const BATTLE_JOB_XP: u32 = 50;

// ---------------------------------------------------------------------------
// Session seeding: a production save loaded through the reconnect path.
// ---------------------------------------------------------------------------

impl Fixture {
    /// A logged-in session whose save was written by `game_store` and reloaded
    /// exactly like a reconnect: the seed switches to the Brawler job, then the
    /// restored character gets its own `next_instance_id` and optional saved
    /// NPC quest state. Instance counters are per character save, so distinct
    /// seeds make each session's start screen observably its own.
    fn seated_session(
        &self,
        position: [f32; 3],
        guid: u64,
        next_instance_id: i32,
        quests: Option<QuestState>,
    ) -> Gateway {
        let mut seed = self.gateway(position, guid);
        assert!(send(&mut seed, &switch_job(BRAWLER), 300).state_changed);
        let gateway = reopen_with(
            self,
            &seed,
            &format!("seated-{guid}.json"),
            400,
            move |gameplay| {
                gameplay
                    .encounter
                    .as_mut()
                    .expect("the seed session has an encounter state")
                    .next_instance_id = next_instance_id;
                if let Some(quests) = quests {
                    gameplay.npc_quests = Some(quests);
                }
            },
        );
        let state = gateway.snapshot().gameplay.expect("gameplay loads");
        assert_eq!(state.profiles.as_ref().unwrap().active, BRAWLER);
        assert_eq!(
            state
                .encounter
                .expect("encounter state loads")
                .next_instance_id,
            next_instance_id
        );
        assert_eq!(gateway.state().zone_id, 0);
        assert_eq!(gateway.state().position, position);
        gateway
    }
}

/// The shipped active quest state, restored by the production catalogue bind
/// (the same state a reconnect loads). `count` is the saved goal progress.
fn delinquent_quests(guid: u64, count: u32) -> QuestState {
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
        DELINQUENT_QUEST,
        QuestProgress {
            status: QuestStatus::Active,
            counts: vec![count],
        },
    );
    state
}

fn quest_count(gateway: &Gateway, quest: u32) -> u32 {
    gateway
        .snapshot()
        .gameplay
        .expect("gameplay loads")
        .npc_quests
        .expect("the catalogue bind keeps the saved quest state")
        .quests
        .get(&quest)
        .map_or(0, |progress| progress.counts[0])
}

// ---------------------------------------------------------------------------
// The row's marker, zone and the session's own active run.
// ---------------------------------------------------------------------------

/// The row's marker GUID, authored marker pose and arena zone.
fn marker_of(row: &Value) -> ([f32; 3], u64, u32) {
    (
        position(&row["spawner"]["position"]),
        u64_field(&row["spawner"], "guid"),
        u32_field(&row["zone"], "zone_id"),
    )
}

/// B stands 1.41 m from the marker: inside the 8 m screen radius
/// (`BATTLE_SCREEN_RADIUS`) with its own return pose.
fn b_start(marker: [f32; 3]) -> [f32; 3] {
    [marker[0] + 1.0, marker[1], marker[2] + 1.0]
}

fn encounter_active(gateway: &Gateway) -> Option<ActiveEncounter> {
    gateway.snapshot().gameplay?.encounter?.active
}

fn active(gateway: &Gateway) -> ActiveEncounter {
    encounter_active(gateway).expect("the session has an active encounter")
}

fn phase(gateway: &Gateway) -> Option<EncounterPhase> {
    encounter_active(gateway).map(|run| run.phase)
}

// ---------------------------------------------------------------------------
// Refusal payloads and the directed 35/2 AddNpc reader.
// ---------------------------------------------------------------------------

/// A forged 41/108 Go whose payload names an encounter/instance this session
/// does not own. The server ignores client ids and starts only its own pending
/// screen, so with no screen open the request is inert.
fn forged_go() -> Vec<u8> {
    let mut packet = vec![41, 0, 108, 0];
    packet.extend(SHEEP_WATCH.to_le_bytes());
    packet.extend(0x7fff_i32.to_le_bytes());
    packet.extend(0_u64.to_le_bytes());
    packet
}

fn malformed_packets() -> [Vec<u8>; 5] {
    [
        // 26/8 must be exactly 13 bytes (constructor 0x008b6bc0).
        vec![26, 0, 8, 0, 0, 0, 0, 0],
        // 36/10 targeted path must be 24 bytes (writer 0x00a342e0).
        vec![36, 0, 10, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        // 26/42 Close is exactly four bytes (constructor 0x008a2eb0).
        vec![26, 0, 42, 0, 9],
        // 41/109 exit is exactly 12 bytes.
        vec![41, 0, 109, 0, 0],
        // 36/10 header only: too short for the ability body.
        vec![36, 0, 10, 0],
    ]
}

/// Actor ids in a batch of unframed replies, decoded by the production 35/2
/// reader (`parse_add_npc` rejects every other family/subtype).
fn add_npc_ids(packets: &[&[u8]]) -> Vec<u64> {
    packets
        .iter()
        .filter_map(|packet| parse_add_npc(packet).ok())
        .map(|npc| npc.actor_id)
        .collect()
}

/// Knock `gateway` out through the arena's own tick: stand in the pack and let
/// the row's enemies attack until the server sets `Lost`. Returns the knockout
/// tick's replies so the caller can assert the results packets.
fn knock_out(gateway: &mut Gateway, now: &mut u64) -> Vec<Vec<u8>> {
    let player = gateway.state().guid;
    let (_, at, _) = active_enemy(gateway).expect("a standing arena actor");
    walk(gateway, player, [at[0] + 1.0, at[1], at[2] + 1.0], now);
    for _ in 0..400 {
        *now += 1_000;
        let tick = gateway.tick_at(*now).expect("the arena tick runs");
        if phase(gateway) == Some(EncounterPhase::Lost) {
            return tick.replies;
        }
    }
    panic!("the arena actors must knock the idle player out");
}

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

/// 1. Both players click the same marker and each reaches its own pending then
/// active encounter. A's click/Go changes none of B's state, pending screen,
/// position or save; a forged and an out-of-range Go change nothing; each start
/// screen matches the row and names its own seeded instance.
#[test]
fn two_players_start_the_same_marker_with_independent_encounters() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, SHEEP_WATCH);
    let (marker, marker_guid, arena_zone) = marker_of(row);
    let b_start = b_start(marker);
    let mut a = fixture.seated_session(marker, 6001, 41, None);
    let mut b = fixture.seated_session(b_start, 6002, 108, None);

    // A forged Go with no open screen is inert and changes no byte.
    let a_before = (a.snapshot(), saved_bytes(&a));
    let refused = send(&mut a, &forged_go(), 400);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(refused.replies.is_empty(), "a forged Go has no reply");
    assert_eq!((a.snapshot(), saved_bytes(&a)), a_before);

    // A opens its start screen: B is untouched in every observable field.
    let b_before = (
        b.snapshot(),
        saved_bytes(&b),
        b.state().position,
        b.state().zone_id,
    );
    let a_screen = open_battle(&mut a, marker_guid, row, 500);
    assert_eq!(i32_at(&a_screen, 8), 41, "A's own seeded instance");
    assert_eq!(
        (b.snapshot(), saved_bytes(&b)),
        (b_before.0.clone(), b_before.1.clone()),
        "A's start screen changes no B state"
    );
    assert_eq!(b.state().position, b_before.2);
    assert_eq!(b.state().zone_id, b_before.3);
    assert!(encounter_active(&b).is_none());

    // B opens its own screen with its own instance; A's pending screen is not
    // consumed by B's click (A's Go below still starts A's instance).
    let b_screen = open_battle(&mut b, marker_guid, row, 600);
    assert_eq!(i32_at(&b_screen, 8), 108, "B's own seeded instance");
    assert_ne!(i32_at(&a_screen, 8), i32_at(&b_screen, 8));
    assert!(encounter_active(&a).is_none());
    assert!(encounter_active(&b).is_none());

    // An out-of-range Go is inert: A walks 50 m away, the request changes no
    // byte, and the server drops A's stale screen (it is never saved). A walks
    // back and reopens its own marker screen before starting.
    let mut a_now = 700;
    let a_guid = a.state().guid;
    walk(
        &mut a,
        a_guid,
        [marker[0] + 50.0, marker[1], marker[2]],
        &mut a_now,
    );
    let a_away = (a.snapshot(), saved_bytes(&a));
    let refused_go = send(&mut a, &go(&a_screen), a_now + 100);
    assert!(
        !refused_go.began_zoning && !refused_go.state_changed && refused_go.replies.is_empty(),
        "an out-of-range Go changes nothing"
    );
    assert_eq!((a.snapshot(), saved_bytes(&a)), a_away);
    a_now += 200;
    walk(&mut a, a_guid, marker, &mut a_now);
    let a_screen = open_battle(&mut a, marker_guid, row, a_now);
    assert_eq!(i32_at(&a_screen, 8), 41);
    a_now += 100;

    // A's Go starts only A. B keeps its own pending screen and save.
    enter_battle(&mut a, &a_screen, row, a_now);
    assert_eq!(a.state().zone_id, arena_zone);
    assert_eq!(active(&a).instance_id, 41);
    assert_eq!(
        (b.snapshot(), saved_bytes(&b)),
        (b_before.0.clone(), b_before.1.clone()),
        "A's Go changes no B state"
    );
    assert_eq!(b.state().zone_id, 0);
    assert_eq!(b.state().position, b_start);

    // B's own pending screen still works after A started; A is unchanged.
    let a_started = (a.snapshot(), a.state().position, a.state().zone_id);
    enter_battle(&mut b, &b_screen, row, 800);
    assert_eq!(b.state().zone_id, arena_zone);
    assert_eq!(active(&b).instance_id, 108);
    assert_eq!(
        (a.snapshot(), a.state().position, a.state().zone_id),
        a_started,
        "B's Go changes no A state"
    );

    // Both arenas load their own copies of the row's actors; the running
    // 41/114 refresh is asserted by `enter_battle` for each.
    let a_enemies = active(&a).enemies;
    let b_enemies = active(&b).enemies;
    assert_eq!(a_enemies.len(), row["enemies"].as_array().unwrap().len());
    assert_eq!(b_enemies.len(), a_enemies.len());
    for (left, right) in a_enemies.iter().zip(&b_enemies) {
        assert_eq!(left.guid, right.guid, "both arenas carry the row's actors");
    }
    assert_ne!(active(&a).instance_id, active(&b).instance_id);
}

/// 2. A wins through the real 36/10 path while B is still fighting; later B is
/// knocked out through its own arena tick. Neither result reaches the other
/// session, and each Close returns its own player to its own pre-entry pose.
#[test]
fn a_win_or_loss_in_one_encounter_does_not_affect_the_other() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, SHEEP_WATCH);
    let (marker, marker_guid, _) = marker_of(row);
    let b_start = b_start(marker);
    let prizes = prize_ids(row);
    let mut a = fixture.seated_session(marker, 6101, 41, Some(delinquent_quests(6101, 0)));
    let mut b = fixture.seated_session(b_start, 6102, 108, Some(delinquent_quests(6102, 0)));
    assert_eq!(quest_count(&a, DELINQUENT_QUEST), 0);
    assert_eq!(quest_count(&b, DELINQUENT_QUEST), 0);

    let a_screen = open_battle(&mut a, marker_guid, row, 400);
    enter_battle(&mut a, &a_screen, row, 500);
    let b_screen = open_battle(&mut b, marker_guid, row, 400);
    enter_battle(&mut b, &b_screen, row, 500);

    // B takes its own hit: its own actor's HP drops and it keeps fighting.
    let mut b_now = 1_000;
    let (b_target, b_at, b_health) = active_enemy(&b).expect("B has a standing actor");
    let b_guid = b.state().guid;
    walk(
        &mut b,
        b_guid,
        [b_at[0] + 1.5, b_at[1], b_at[2]],
        &mut b_now,
    );
    b_now += 700;
    assert!(send(&mut b, &melee(b_target), b_now).state_changed);
    let b_hit = active(&b)
        .enemies
        .into_iter()
        .find(|enemy| enemy.guid == b_target)
        .expect("B's target is one of its own arena actors");
    assert!(b_hit.health < b_health, "B's own attack landed");
    let b_after_hit = active(&b);
    assert_eq!(b_after_hit.phase, EncounterPhase::Fighting);
    let b_before_a_win = (
        b.snapshot(),
        saved_bytes(&b),
        economy(&b, &prizes),
        quest_count(&b, DELINQUENT_QUEST),
    );

    // A wins through ordinary attacks; B is not ticked or sent anything.
    let mut a_now = 1_000;
    let a_before_win = economy(&a, &prizes);
    let a_replies = fight_to_win(&mut a, &mut a_now, row);
    assert_eq!(results_lifecycle(&a_replies), vec![18, 47, 19]);
    assert_eq!(won_flag(&a_replies), Some(1));
    let a_paid = economy(&a, &prizes);
    assert_payout(&a_before_win, &a_paid, row, &prizes, BATTLE_JOB_XP);
    assert_eq!(quest_count(&a, DELINQUENT_QUEST), 2);

    // Nothing of A's win reached B: state, save, wallet, quest and fight.
    assert_eq!(
        (b.snapshot(), saved_bytes(&b)),
        (b_before_a_win.0.clone(), b_before_a_win.1.clone()),
        "A's win changes no B state or save"
    );
    assert_eq!(economy(&b, &prizes), b_before_a_win.2);
    assert_eq!(quest_count(&b, DELINQUENT_QUEST), b_before_a_win.3);
    assert_eq!(active(&b), b_after_hit);
    assert_eq!(phase(&b), Some(EncounterPhase::Fighting));

    // B is knocked out by its own arena tick while A sits on its results. The
    // loss pays and credits neither player and does not close A's run.
    let a_finished = (a.snapshot(), saved_bytes(&a));
    let b_lost = knock_out(&mut b, &mut b_now);
    assert_eq!(results_lifecycle(&b_lost), vec![18, 47, 19]);
    assert_eq!(won_flag(&b_lost), Some(0), "B's result is a loss");
    assert_eq!(phase(&b), Some(EncounterPhase::Lost));
    assert_eq!(
        economy(&b, &prizes),
        b_before_a_win.2,
        "a loss pays nothing"
    );
    assert_eq!(
        quest_count(&b, DELINQUENT_QUEST),
        0,
        "a loss credits nothing"
    );
    assert_eq!(
        (a.snapshot(), saved_bytes(&a)),
        a_finished,
        "B's loss changes no A state"
    );

    // B closes its lost run and returns to its own pose; A keeps its results.
    let b_closed = send(&mut b, &[26, 0, 42, 0], b_now + 200);
    assert!(b_closed.began_zoning);
    assert_returned(&b, b_start);
    assert_eq!(economy(&b, &prizes), b_before_a_win.2);
    assert_eq!(economy(&a, &prizes), a_paid, "B's Close never pays A");
    assert_eq!(phase(&a), Some(EncounterPhase::Won));

    // A closes its own win and returns to the marker; B's return stays.
    let b_returned = (b.snapshot(), saved_bytes(&b));
    let a_closed = send(&mut a, &[26, 0, 42, 0], a_now + 200);
    assert!(a_closed.began_zoning);
    assert_returned(&a, marker);
    assert_eq!(economy(&a, &prizes), a_paid, "Close pays nothing");
    assert_eq!((b.snapshot(), saved_bytes(&b)), b_returned);
}

/// 3. Both players win, close their own results and return to their own
/// pre-entry pose. A live coin delta inserted during the results survives A's
/// Close, each save carries only its own wallet/XP/items/quest counters, and
/// replaying A's Close/Go pays nothing and does not start B.
#[test]
fn both_get_their_own_payout_and_return_to_their_own_pose() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, SHEEP_WATCH);
    let (marker, marker_guid, arena_zone) = marker_of(row);
    let b_start = b_start(marker);
    let arena_arrival = position(&row["zone"]["position"]);
    let prizes = prize_ids(row);
    let mut a = fixture.seated_session(marker, 6201, 41, Some(delinquent_quests(6201, 1)));
    let mut b = fixture.seated_session(b_start, 6202, 108, Some(delinquent_quests(6202, 0)));
    assert_eq!(quest_count(&a, DELINQUENT_QUEST), 1);
    assert_eq!(quest_count(&b, DELINQUENT_QUEST), 0);
    let a_before = economy(&a, &prizes);
    let b_before = economy(&b, &prizes);

    let a_screen = open_battle(&mut a, marker_guid, row, 400);
    enter_battle(&mut a, &a_screen, row, 500);
    let b_screen = open_battle(&mut b, marker_guid, row, 400);
    enter_battle(&mut b, &b_screen, row, 500);

    let mut a_now = 1_000;
    let a_replies = fight_to_win(&mut a, &mut a_now, row);
    assert_eq!(results_lifecycle(&a_replies), vec![18, 47, 19]);
    let a_paid = economy(&a, &prizes);
    assert_payout(&a_before, &a_paid, row, &prizes, BATTLE_JOB_XP);
    assert_eq!(quest_count(&a, DELINQUENT_QUEST), 3);

    let mut b_now = 1_000;
    let b_replies = fight_to_win(&mut b, &mut b_now, row);
    assert_eq!(results_lifecycle(&b_replies), vec![18, 47, 19]);
    let b_paid = economy(&b, &prizes);
    assert_payout(&b_before, &b_paid, row, &prizes, BATTLE_JOB_XP);
    assert_eq!(quest_count(&b, DELINQUENT_QUEST), 2);

    // A live delta during A's results screen: no old snapshot is restored over
    // it on Close. B's wallet does not move.
    let b_before_delta = (b.snapshot(), saved_bytes(&b));
    let delta = send(&mut a, &chat::local_request("realm coins 7"), a_now + 100);
    assert!(delta.state_changed, "the coin command grants");
    let a_with_delta = economy(&a, &prizes);
    assert_eq!(a_with_delta.coins, a_paid.coins + 7);
    assert_eq!(
        (b.snapshot(), saved_bytes(&b)),
        (b_before_delta.0.clone(), b_before_delta.1.clone())
    );

    // A closes its own results and returns to A's pose only.
    let a_closed = send(&mut a, &[26, 0, 42, 0], a_now + 200);
    assert!(a_closed.began_zoning, "Close returns to the open world");
    assert_returned(&a, marker);
    assert_eq!(economy(&a, &prizes), a_with_delta, "Close pays nothing");
    assert_eq!(quest_count(&a, DELINQUENT_QUEST), 3);

    // B is still in its own won arena, untouched by A's Close.
    assert_eq!(b.state().zone_id, arena_zone);
    assert_eq!(b.state().position, arena_arrival);
    assert_eq!(phase(&b), Some(EncounterPhase::Won));
    assert_eq!(economy(&b, &prizes), b_paid);
    assert_eq!(quest_count(&b, DELINQUENT_QUEST), 2);

    // Replaying A's Close and Go pays nothing and does not start B.
    let a_after_close = (a.snapshot(), saved_bytes(&a));
    let replay_close = send(&mut a, &[26, 0, 42, 0], a_now + 300);
    assert!(!replay_close.began_zoning && !replay_close.state_changed);
    assert_eq!((a.snapshot(), saved_bytes(&a)), a_after_close);
    let replay_go = send(&mut a, &go(&a_screen), a_now + 400);
    assert!(!replay_go.began_zoning && !replay_go.state_changed);
    assert_eq!((a.snapshot(), saved_bytes(&a)), a_after_close);
    let b_before_replay = (b.snapshot(), saved_bytes(&b));
    let cross_go = send(&mut b, &go(&a_screen), b_now + 200);
    assert!(
        !cross_go.began_zoning && !cross_go.state_changed,
        "A's old screen cannot start B"
    );
    assert_eq!((b.snapshot(), saved_bytes(&b)), b_before_replay);

    // B closes its own results and returns to B's own pose.
    let b_closed = send(&mut b, &[26, 0, 42, 0], b_now + 300);
    assert!(b_closed.began_zoning);
    assert_returned(&b, b_start);
    assert_eq!(economy(&b, &prizes), b_paid);
    assert_ne!(a_with_delta.coins, b_paid.coins, "each wallet is its own");
    assert_eq!(economy(&a, &prizes), a_with_delta, "B's Close never pays A");

    // Each save holds exactly its own coins/XP/items/quest counters and no
    // live encounter: reload both through the production reconnect path.
    let a_state = (a.snapshot(), saved_bytes(&a));
    let b_state = (b.snapshot(), saved_bytes(&b));
    let reopened_a = reopen(&fixture, &a, "a-after-close.json", a_now + 500);
    let reopened_b = reopen(&fixture, &b, "b-after-close.json", b_now + 500);
    assert_eq!(economy(&reopened_a, &prizes), a_with_delta);
    assert_eq!(quest_count(&reopened_a, DELINQUENT_QUEST), 3);
    assert_returned(&reopened_a, marker);
    assert_eq!(economy(&reopened_b, &prizes), b_paid);
    assert_eq!(quest_count(&reopened_b, DELINQUENT_QUEST), 2);
    assert_returned(&reopened_b, b_start);
    assert_eq!((a.snapshot(), saved_bytes(&a)), a_state);
    assert_eq!((b.snapshot(), saved_bytes(&b)), b_state);
}

/// 4. A third session joining the field at the marker after A and B are inside
/// sees no arena actor and exactly one static marker, and can only open its own
/// start screen. It is not pulled into A's or B's fight.
#[test]
fn a_third_late_joiner_sees_no_arena_actors() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, SHEEP_WATCH);
    let (marker, marker_guid, _) = marker_of(row);
    let b_start = b_start(marker);
    let mut a = fixture.seated_session(marker, 6301, 41, None);
    let mut b = fixture.seated_session(b_start, 6302, 108, None);
    let a_screen = open_battle(&mut a, marker_guid, row, 400);
    enter_battle(&mut a, &a_screen, row, 500);
    let b_screen = open_battle(&mut b, marker_guid, row, 400);
    enter_battle(&mut b, &b_screen, row, 500);
    assert_eq!(phase(&a), Some(EncounterPhase::Fighting));
    assert_eq!(phase(&b), Some(EncounterPhase::Fighting));

    // The late joiner enters the same open-world field at the marker.
    let (mut c, c_entry) = fixture.session(marker, 6303);
    assert!(send(&mut c, &switch_job(BRAWLER), 300).state_changed);
    let entry: Vec<&[u8]> = c_entry
        .replies
        .iter()
        .map(|packet| unframe(packet))
        .collect();
    let spawned = add_npc_ids(&entry);
    assert_eq!(
        spawned.iter().filter(|id| **id == marker_guid).count(),
        1,
        "the static marker is published exactly once to the late joiner"
    );
    for enemy in row["enemies"].as_array().unwrap() {
        let guid = u64_field(enemy, "guid");
        assert!(
            !spawned.contains(&guid),
            "arena actor {guid} is not in a third session's 35/2 batch"
        );
        assert!(
            !entry.iter().any(|packet| packet
                .windows(8)
                .any(|word| word == guid.to_le_bytes().as_slice())),
            "arena actor {guid} leaks into a third session's world entry"
        );
    }
    assert!(
        !entry
            .iter()
            .any(|packet| packet.starts_with(&[41, 0, 114, 0])),
        "another player's running encounter is not replayed to a late joiner"
    );
    assert!(encounter_active(&c).is_none());
    assert_eq!(c.state().zone_id, 0);

    // The marker click opens only C's own start screen; A and B are untouched.
    let a_before = (a.snapshot(), saved_bytes(&a));
    let b_before = (b.snapshot(), saved_bytes(&b));
    let c_instance = c
        .snapshot()
        .gameplay
        .expect("gameplay loads")
        .encounter
        .expect("encounter state loads")
        .next_instance_id;
    let c_screen = open_battle(&mut c, marker_guid, row, 600);
    assert_eq!(i32_at(&c_screen, 8), c_instance);
    assert!(encounter_active(&c).is_none());
    assert_eq!(c.state().zone_id, 0);
    assert_eq!((a.snapshot(), saved_bytes(&a)), a_before);
    assert_eq!((b.snapshot(), saved_bytes(&b)), b_before);

    // C's own Go starts C's own fight, not A's or B's.
    enter_battle(&mut c, &c_screen, row, 700);
    assert_eq!(active(&c).instance_id, c_instance);
    assert_eq!(phase(&a), Some(EncounterPhase::Fighting));
    assert_eq!(phase(&b), Some(EncounterPhase::Fighting));
    assert_eq!((a.snapshot(), saved_bytes(&a)), a_before);
    assert_eq!((b.snapshot(), saved_bytes(&b)), b_before);
}

/// 5. A's malformed packets are refused for A alone with byte-identical state,
/// and A's mid-fight reconnect drops only A's run. B keeps moving, lands its
/// own 36/10 hit and is still fighting; A can start a fresh battle. The
/// disconnect continuation itself (B wins and returns) is owned by
/// `sheep_watch_dungeon.rs`'s two-session block and is not repeated here.
#[test]
fn one_players_disconnect_or_bad_packet_does_not_stop_the_other() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, SHEEP_WATCH);
    let (marker, marker_guid, _) = marker_of(row);
    let b_start = b_start(marker);
    let prizes = prize_ids(row);
    let mut a = fixture.seated_session(marker, 6401, 41, None);
    let mut b = fixture.seated_session(b_start, 6402, 108, None);
    let a_before_battle = economy(&a, &prizes);
    let a_screen = open_battle(&mut a, marker_guid, row, 400);
    enter_battle(&mut a, &a_screen, row, 500);
    let b_screen = open_battle(&mut b, marker_guid, row, 400);
    enter_battle(&mut b, &b_screen, row, 500);
    let b_before = (b.snapshot(), saved_bytes(&b), economy(&b, &prizes));

    let a_before_bad = (a.snapshot(), saved_bytes(&a));
    for (index, malformed) in malformed_packets().iter().enumerate() {
        let refused = send(&mut a, malformed, 1_000 + index as u64);
        assert!(
            !refused.state_changed && !refused.began_zoning && refused.replies.is_empty(),
            "malformed packet {index} is refused for A alone"
        );
        assert_eq!(
            (a.snapshot(), saved_bytes(&a)),
            a_before_bad,
            "malformed packet {index} changes no A state"
        );
    }
    assert_eq!(
        (b.snapshot(), saved_bytes(&b), economy(&b, &prizes)),
        b_before
    );

    // A disconnects mid-fight: the production save/reload path drops A's run
    // without paying, and leaves B's live run untouched.
    let mut dropped = reopen(&fixture, &a, "a-dropped.json", 2_000);
    assert_returned(&dropped, marker);
    assert_eq!(economy(&dropped, &prizes), a_before_battle);
    assert_eq!(
        (b.snapshot(), saved_bytes(&b), economy(&b, &prizes)),
        b_before
    );

    // B keeps moving and lands its own real hit.
    let mut b_now = 2_100;
    let (b_target, b_at, b_health) = active_enemy(&b).expect("B has a standing actor");
    let b_guid = b.state().guid;
    walk(
        &mut b,
        b_guid,
        [b_at[0] + 1.5, b_at[1], b_at[2]],
        &mut b_now,
    );
    b_now += 700;
    assert!(
        send(&mut b, &melee(b_target), b_now).state_changed,
        "B still fights after A's reconnect"
    );
    let b_hit = active(&b)
        .enemies
        .into_iter()
        .find(|enemy| enemy.guid == b_target)
        .expect("B's own actor");
    assert!(b_hit.health < b_health);
    assert_eq!(phase(&b), Some(EncounterPhase::Fighting));

    // The dropped session has no stuck pending state: it can start a fresh
    // battle, and that start does not disturb B's live run.
    let next = dropped
        .snapshot()
        .gameplay
        .expect("gameplay loads")
        .encounter
        .expect("encounter state loads")
        .next_instance_id;
    let screen = open_battle(&mut dropped, marker_guid, row, 3_000);
    assert_eq!(i32_at(&screen, 8), next);
    let b_live = (b.snapshot(), saved_bytes(&b));
    enter_battle(&mut dropped, &screen, row, 3_100);
    assert_eq!(active(&dropped).instance_id, next);
    assert_eq!((b.snapshot(), saved_bytes(&b)), b_live);
}

/// 6. The normal start/fight/return path uses no developer command. The
/// `realm win` shortcut is refused for a normal player role and cannot end a
/// live fight; the win here comes from the real 36/10 attacks.
#[test]
fn the_route_uses_no_dev_commands() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, SHEEP_WATCH);
    let (marker, marker_guid, _) = marker_of(row);
    let prizes = prize_ids(row);
    let mut gateway = fixture.seated_session(marker, 6501, 41, None);
    gateway.set_dev_tools(false);
    gateway.set_role(AccountRole::Player);
    let before = (
        gateway.snapshot(),
        saved_bytes(&gateway),
        economy(&gateway, &prizes),
    );

    // The developer shortcut is not a player-reachable path.
    let denied = send(&mut gateway, &chat::local_request("realm win"), 400);
    assert_eq!(
        denied.dev_command_denials, 1,
        "a player role may not run the developer win"
    );
    assert!(!denied.state_changed && !denied.began_zoning);
    assert_eq!(
        (gateway.snapshot(), saved_bytes(&gateway)),
        (before.0.clone(), before.1.clone()),
        "the developer win changes nothing"
    );
    let denied_battle = send(
        &mut gateway,
        &chat::local_request("realm battle 12350"),
        450,
    );
    assert_eq!(denied_battle.dev_command_denials, 1);
    assert!(!denied_battle.state_changed && !denied_battle.began_zoning);
    assert_eq!(
        (gateway.snapshot(), saved_bytes(&gateway)),
        (before.0.clone(), before.1.clone())
    );

    // Ordinary client requests only: 26/8 -> 41/114 -> 41/108 -> 36/10 ->
    // 26/42. No route outcome is a developer-command refusal.
    let screen = open_battle(&mut gateway, marker_guid, row, 500);
    let entered = send(&mut gateway, &go(&screen), 600);
    assert!(entered.began_zoning);
    assert_eq!(entered.dev_command_denials, 0);
    let ready = arena_spawned(&mut gateway, 700);
    assert_eq!(ready.dev_command_denials, 0);
    let mut now = 800;
    let (target, at, _) = active_enemy(&gateway).expect("a standing arena actor");
    let player = gateway.state().guid;
    walk(&mut gateway, player, [at[0] + 1.5, at[1], at[2]], &mut now);
    now += 700;
    assert!(send(&mut gateway, &melee(target), now).state_changed);
    // The same shortcut is still refused mid-fight and cannot end it.
    let mid_denied = send(&mut gateway, &chat::local_request("realm win"), now + 100);
    assert_eq!(mid_denied.dev_command_denials, 1);
    assert!(!mid_denied.state_changed && !mid_denied.began_zoning);
    assert_eq!(
        phase(&gateway),
        Some(EncounterPhase::Fighting),
        "the refused developer win cannot win the fight"
    );
    let before_win = economy(&gateway, &prizes);
    let replies = fight_to_win(&mut gateway, &mut now, row);
    assert_eq!(results_lifecycle(&replies), vec![18, 47, 19]);
    assert_eq!(won_flag(&replies), Some(1), "the real attacks win");
    assert_payout(
        &before_win,
        &economy(&gateway, &prizes),
        row,
        &prizes,
        BATTLE_JOB_XP,
    );
    let closed = send(&mut gateway, &[26, 0, 42, 0], now + 200);
    assert!(closed.began_zoning);
    assert_eq!(closed.dev_command_denials, 0);
    assert_returned(&gateway, marker);
}
