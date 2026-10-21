//! T373: two players run the same dungeon marker with independent encounters.
//!
//! The five playable field battles are validated one player at a time
//! (`tests/sheep_watch_dungeon.rs`, `tests/thugamug_battle.rs`). This suite
//! proves the production model is a **session-local** encounter: two in-process
//! `Gateway`s (two characters) click the same static Sheep Watch marker (battle
//! `12350`, T319), each owns its start screen, instance, arena actors, HP,
//! results, payout and return pose, and a third late joiner is not pulled into
//! either fight.
//!
//! Nothing in `src/` changes. The route is the shipped marker route
//! (26/8 click -> 41/114 start screen -> 41/108 Go -> zone 9600 -> 36/10
//! attacks -> 39/18/47/19 results -> 26/42 Close) through the real `Gateway`
//! dispatch, so every request is the production packet.
//!
//! Client shapes checked in the supplied 2014 client (IDA):
//! * 26/8 interaction constructor `0x008b6bc0` = target u64 + bool, the
//!   marker click.
//! * 26/10 constructor `0x00a36f80` = target u64 + i32 choice: it is the NPC
//!   dialogue/choice request, **not** a field-battle step. The marker path is
//!   the existing 26/8 -> 41/114 -> 41/108 route pinned by T319; the brief's
//!   "26/8 -> 26/10 choose the battle" is corrected here.
//! * 41/108 Go constructor `0x008b6e70` (header only; the start screen owns
//!   the encounter/instance, so the server starts only its own pending
//!   screen).
//! * 41/114 start/running screen reader chain
//!   `0x00aa3430 -> 0x00aa32d0 -> 0x00a29120 -> 0x009bdd70`: encounter id at
//!   bytes 4..8 and instance id at bytes 8..12, the two fields this suite
//!   compares per session.
//! * 36/10 attack writer `0x00a342e0` / reader `0x00a35150`: bar 1, slot 0,
//!   target mode 0, target GUID.
//! * 35/2 AddNpc reader `0x00921a10` reads the u64 actor id first, so the
//!   marker/enemy GUID membership below is a real visibility check.
//! * 26/42 results Close constructor `0x008a2eb0`, sender `0x0092f300`.
//!
//! The arena is not a shared instance: `encounter.battle_spawners()` publishes
//! only the static marker to every session's `NpcVisibility`, while the arena
//! actors are spawned into the acting session's replies on zone-ready. Test 4
//! pins that through the production 35/2 batch.

use osfr_tools::{
    account_store::AccountRole,
    chat,
    encounter::{ActiveEncounter, EncounterPhase},
    gateway::{Gateway, GatewayOutcome},
    quest_engine::{CharacterEligibility, QuestEngine, QuestProgress, QuestState, QuestStatus},
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[path = "common/field_battle_fixture.rs"]
mod field_battle_fixture;

use field_battle_fixture::*;

/// T319's authored Sheep Watch row and marker.
const SHEEP_WATCH: u32 = 12350;
const MARKER_GUID: u64 = 100001235000;
const MARKER: [f32; 3] = [-2165.513, -32.96, 328.1112];
/// B stands 1.41 m from the marker: inside the 8 m screen radius
/// (`BATTLE_SCREEN_RADIUS`), with its own return pose.
const B_START: [f32; 3] = [MARKER[0] + 1.0, MARKER[1], MARKER[2] + 1.0];
const ARENA_ZONE: u32 = 9600;
const ARENA_ARRIVAL: [f32; 3] = [136.0, -0.5, 155.0];
const BRAWLER: i32 = 43;
/// The shipped `Brawler: Delinquent Dilemma` defeat goal (kind `930050`).
const DELINQUENT_QUEST: u32 = 3030007;
/// (R) The engine's fixed field-battle job XP (half a rank on the local curve).
const FIELD_BATTLE_JOB_XP: u32 = 50;

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
// Encounter state and the production marker route.
// ---------------------------------------------------------------------------

fn encounter_active(gateway: &Gateway) -> Option<ActiveEncounter> {
    gateway.snapshot().gameplay?.encounter?.active
}

fn active(gateway: &Gateway) -> ActiveEncounter {
    encounter_active(gateway).expect("the session has an active encounter")
}

fn enemy_guids(row: &Value) -> Vec<u64> {
    row["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .map(|enemy| u64_field(enemy, "guid"))
        .collect()
}

/// Click the marker (26/8, constructor 0x008b6bc0) and return the 41/114 start
/// screen. The screen must name this session's own battle and instance.
fn click_marker(gateway: &mut Gateway, expected_instance: i32, now_ms: u64) -> Vec<u8> {
    let outcome = send(gateway, &interact(MARKER_GUID), now_ms);
    let screen = details(&outcome)
        .expect("the marker opens the 41/114 start screen")
        .to_vec();
    assert_eq!(i32_at(&screen, 4), SHEEP_WATCH as i32);
    assert_eq!(
        i32_at(&screen, 8),
        expected_instance,
        "the start screen names this session's own instance"
    );
    screen
}

/// Go (41/108) and load the arena scene, then return the running 41/114
/// refresh. `arena_spawned` is the client's 13/10 scene-ready pair after the
/// zoning reply.
fn go_and_enter(
    gateway: &mut Gateway,
    screen: &[u8],
    instance: i32,
    now_ms: u64,
) -> GatewayOutcome {
    let entered = send(gateway, &go(screen), now_ms);
    assert!(entered.began_zoning, "Go starts the battle");
    assert_eq!(gateway.state().zone_id, ARENA_ZONE);
    assert_eq!(gateway.state().position, ARENA_ARRIVAL);
    assert_eq!(active(gateway).instance_id, instance);
    let ready = arena_spawned(gateway, now_ms + 100);
    let running = active(gateway);
    assert_eq!(running.instance_id, instance);
    assert_eq!(running.phase, EncounterPhase::Fighting);
    assert_eq!(
        running_instance(&ready),
        instance,
        "the running 41/114 refresh names this session's instance"
    );
    ready
}

/// The instance id in the running 41/114 refresh (reader chain 0x00aa3430).
fn running_instance(outcome: &GatewayOutcome) -> i32 {
    let packet = outcome
        .replies
        .iter()
        .map(|packet| unframe(packet))
        .find(|packet| {
            packet.starts_with(&[41, 0, 114, 0])
                && packet.len() >= 12
                && i32_at(packet, 4) == SHEEP_WATCH as i32
        })
        .expect("the running encounter refreshes through 41/114");
    i32_at(packet, 8)
}

fn active_enemy(gateway: &Gateway) -> Option<(u64, [f32; 3], EncounterPhase)> {
    let active = encounter_active(gateway)?;
    let enemy = active.enemies.iter().find(|enemy| enemy.health > 0)?;
    Some((enemy.guid, enemy.position, active.phase))
}

/// One ordinary 36/10 hit on the nearest standing arena actor: walk into the
/// profile's 3 m reach, swing, then walk back to the arena arrival.
fn strike_once(gateway: &mut Gateway, now_ms: &mut u64) -> Option<(u64, Vec<Vec<u8>>)> {
    let (guid, position, phase) = active_enemy(gateway)?;
    if phase != EncounterPhase::Fighting {
        return None;
    }
    let player = gateway.state().guid;
    walk(
        gateway,
        player,
        [position[0] + 1.5, position[1], position[2]],
        now_ms,
    );
    *now_ms += 700;
    // 36/10 writer 0x00a342e0: bar 1, slot 0, target mode 0, target GUID.
    let hit = send(gateway, &melee(guid), *now_ms);
    assert!(hit.state_changed, "the real 36/10 hit lands on {guid}");
    walk(gateway, player, ARENA_ARRIVAL, now_ms);
    *now_ms += 100;
    Some((guid, hit.replies))
}

/// Kill every arena actor through the ordinary 36/10 path and return every
/// reply the fight produced.
fn fight_to_win(gateway: &mut Gateway, now_ms: &mut u64) -> Vec<Vec<u8>> {
    let mut replies = Vec::new();
    for _ in 0..200 {
        let Some((_guid, hit_replies)) = strike_once(gateway, now_ms) else {
            break;
        };
        replies.extend(hit_replies);
        let tick = gateway.tick_at(*now_ms).expect("the fight tick succeeds");
        replies.extend(tick.replies);
        *now_ms += 100;
    }
    assert_eq!(
        encounter_active(gateway).map(|active| active.phase),
        Some(EncounterPhase::Won),
        "the five actors die through the real attack path"
    );
    replies
}

/// 39/18 (game over), 39/47 (scores) and 39/19 (results UI) from
/// `present_results`.
fn results_lifecycle(replies: &[Vec<u8>]) -> Vec<u8> {
    replies
        .iter()
        .map(|packet| unframe(packet))
        .filter(|packet| packet.starts_with(&[39, 0]) && packet.len() >= 3)
        .map(|packet| packet[2])
        .filter(|sub| [18, 47, 19].contains(sub))
        .collect()
}

fn won_flag(replies: &[Vec<u8>]) -> Option<u8> {
    replies
        .iter()
        .map(|packet| unframe(packet))
        .find(|packet| packet.len() >= 16 && packet.starts_with(&[39, 0, 18]))
        .map(|packet| packet[15])
}

// ---------------------------------------------------------------------------
// Economy.
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
struct Economy {
    coins: u32,
    prizes: BTreeMap<i32, u32>,
    experience: BTreeMap<i32, u32>,
}

fn economy(gateway: &Gateway, prize_ids: &BTreeSet<i32>) -> Economy {
    let state = gateway.snapshot().gameplay.expect("gameplay loads");
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
        experience: state.profiles.expect("profiles load").experience,
    }
}

fn assert_payout(before: &Economy, after: &Economy, row: &Value, prize_ids: &BTreeSet<i32>) {
    assert_eq!(after.coins, before.coins + u32_field(row, "reward_coins"));
    for (profile, experience) in &after.experience {
        let prior = before.experience.get(profile).copied().unwrap_or(0);
        let delta = experience - prior;
        assert_eq!(
            delta,
            if *profile == BRAWLER {
                FIELD_BATTLE_JOB_XP
            } else {
                0
            },
            "profile {profile} gained {delta} XP"
        );
    }
    let gains: Vec<(i32, u32)> = after
        .prizes
        .iter()
        .filter_map(|(id, count)| {
            let prior = before.prizes.get(id).copied().unwrap_or(0);
            (count > &prior).then_some((*id, count - prior))
        })
        .collect();
    assert_eq!(gains.len(), 1, "exactly one prize is granted");
    assert!(prize_ids.contains(&gains[0].0));
    assert_eq!(gains[0].1, 1);
}

// ---------------------------------------------------------------------------
// Directed visibility: the 35/2 AddNpc the client reader 0x00921a10 parses.
// ---------------------------------------------------------------------------

fn add_npc_packets(outcome: &GatewayOutcome, actor: u64) -> Vec<Vec<u8>> {
    outcome
        .replies
        .iter()
        .map(|packet| unframe(packet))
        .filter(|packet| {
            packet.starts_with(&[35, 0, 2, 0])
                && packet.len() >= 12
                && u64::from_le_bytes(packet[4..12].try_into().unwrap()) == actor
        })
        .map(<[u8]>::to_vec)
        .collect()
}

fn reply_bytes(outcome: &GatewayOutcome) -> Vec<Vec<u8>> {
    outcome
        .replies
        .iter()
        .map(|packet| unframe(packet).to_vec())
        .collect()
}

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

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

/// 1. Both players click the same marker and each reaches its own pending then
/// active encounter. A's click/Go changes none of B's state, pending screen,
/// position or save, a forged and an out-of-range Go change nothing, and each
/// start packet names its own encounter and instance.
#[test]
fn two_players_start_the_same_marker_with_independent_encounters() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, SHEEP_WATCH);
    let mut a = fixture.seated_session(MARKER, 6001, 41, None);
    let mut b = fixture.seated_session(B_START, 6002, 108, None);

    // A forged Go with no open start screen is inert and changes no byte.
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
    let a_screen = click_marker(&mut a, 41, 500);
    assert_eq!(i32_at(&a_screen, 4), SHEEP_WATCH as i32);
    assert_eq!(
        (b.snapshot(), saved_bytes(&b)),
        (b_before.0.clone(), b_before.1.clone()),
        "A's start screen changes no B state"
    );
    assert_eq!(b.state().position, b_before.2);
    assert_eq!(b.state().zone_id, b_before.3);
    assert!(encounter_active(&b).is_none());

    // B opens its own screen with its own instance; A's pending screen is not
    // consumed by B's click (A's Go below still starts A's instance 41).
    let b_screen = click_marker(&mut b, 108, 600);
    assert_eq!(i32_at(&b_screen, 4), SHEEP_WATCH as i32);
    assert_ne!(
        i32_at(&a_screen, 8),
        i32_at(&b_screen, 8),
        "the two start screens name different sessions' instances"
    );
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
        [MARKER[0] + 50.0, MARKER[1], MARKER[2]],
        &mut a_now,
    );
    let a_out_of_range = (a.snapshot(), saved_bytes(&a));
    let refused_go = send(&mut a, &go(&a_screen), a_now + 100);
    assert!(
        !refused_go.began_zoning && !refused_go.state_changed && refused_go.replies.is_empty(),
        "an out-of-range Go changes nothing"
    );
    assert_eq!((a.snapshot(), saved_bytes(&a)), a_out_of_range);
    assert_eq!(
        (b.snapshot(), saved_bytes(&b)),
        (b_before.0.clone(), b_before.1.clone()),
        "A's out-of-range Go changes no B state"
    );
    a_now += 200;
    walk(&mut a, a_guid, MARKER, &mut a_now);
    let a_screen = click_marker(&mut a, 41, a_now);
    a_now += 100;

    // A's Go starts only A. B keeps its own pending screen and save.
    let a_go = send(&mut a, &go(&a_screen), a_now);
    a_now += 100;
    assert!(a_go.began_zoning);
    assert_eq!(a.state().zone_id, ARENA_ZONE);
    assert_eq!(active(&a).encounter_id, SHEEP_WATCH);
    assert_eq!(active(&a).instance_id, 41);
    assert_eq!(
        (b.snapshot(), saved_bytes(&b)),
        (b_before.0.clone(), b_before.1.clone()),
        "A's Go changes no B state"
    );
    assert_eq!(b.state().zone_id, 0);
    assert_eq!(b.state().position, B_START);

    // B's own Go still works after A started, and it does not touch A.
    let a_started = (a.snapshot(), a.state().position, a.state().zone_id);
    let b_go = send(&mut b, &go(&b_screen), 800);
    assert!(
        b_go.began_zoning,
        "B's own pending screen survives A's start"
    );
    assert_eq!(b.state().zone_id, ARENA_ZONE);
    assert_eq!(active(&b).instance_id, 108);
    assert_eq!(
        (a.snapshot(), a.state().position, a.state().zone_id),
        a_started,
        "B's Go changes no A state"
    );

    // Both arenas load their own copies of the five actors; the running 41/114
    // screens keep naming each session's own instance.
    let a_ready = arena_spawned(&mut a, a_now);
    let b_ready = arena_spawned(&mut b, 900);
    assert_eq!(running_instance(&a_ready), 41);
    assert_eq!(running_instance(&b_ready), 108);
    let expected_enemies = enemy_guids(row);
    let a_enemies = active(&a).enemies;
    let b_enemies = active(&b).enemies;
    assert_eq!(a_enemies.len(), expected_enemies.len());
    assert_eq!(b_enemies.len(), expected_enemies.len());
    for (left, right) in a_enemies.iter().zip(&b_enemies) {
        assert_eq!(left.guid, right.guid, "both arenas carry the row's actors");
        assert_eq!(left.health, right.health);
    }
}

/// 2. A wins through the real 36/10 attack path while B is still fighting. A's
/// payout, quest credit and results state never appear in B; B keeps its own
/// HP/target/cooldown and can still attack and win.
///
/// The T319 `realm win` shortcut is **not** used (test 6 proves it is not
/// player-reachable). The knockout/revive sub-case is not exercised because the
/// arena knockout is the tick's Lost path, not a per-session request; A's
/// knockout is therefore not "used" here.
#[test]
fn a_win_or_loss_in_one_encounter_does_not_affect_the_other() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, SHEEP_WATCH);
    let prizes = prize_ids(row);
    let mut a = fixture.seated_session(MARKER, 6101, 41, Some(delinquent_quests(6101, 0)));
    let mut b = fixture.seated_session(B_START, 6102, 108, Some(delinquent_quests(6102, 0)));
    assert_eq!(quest_count(&a, DELINQUENT_QUEST), 0);
    assert_eq!(quest_count(&b, DELINQUENT_QUEST), 0);

    let a_screen = click_marker(&mut a, 41, 400);
    go_and_enter(&mut a, &a_screen, 41, 500);
    let b_screen = click_marker(&mut b, 108, 400);
    go_and_enter(&mut b, &b_screen, 108, 500);

    // B takes its own hit and its own live state is captured.
    let mut b_now = 1_000;
    let (b_target, _) = strike_once(&mut b, &mut b_now).expect("B has a standing enemy");
    let b_after_hit = active(&b);
    let b_target_health = b_after_hit
        .enemies
        .iter()
        .find(|enemy| enemy.guid == b_target)
        .expect("B's target is one of its own arena actors")
        .health;
    assert_eq!(b_after_hit.phase, EncounterPhase::Fighting);
    let b_before_a_win = (
        b.snapshot(),
        saved_bytes(&b),
        economy(&b, &prizes),
        quest_count(&b, DELINQUENT_QUEST),
    );
    let a_before_win = economy(&a, &prizes);

    // A wins through the ordinary attacks. B is not ticked or sent anything.
    let mut a_now = 1_000;
    let a_replies = fight_to_win(&mut a, &mut a_now);
    assert_eq!(results_lifecycle(&a_replies), vec![18, 47, 19]);
    assert_eq!(won_flag(&a_replies), Some(1));
    let a_paid = economy(&a, &prizes);
    assert_payout(&a_before_win, &a_paid, row, &prizes);
    assert_eq!(quest_count(&a, DELINQUENT_QUEST), 2);

    // Nothing of A's win reached B: state, save, wallet, quest and fight.
    assert_eq!(
        (b.snapshot(), saved_bytes(&b)),
        (b_before_a_win.0.clone(), b_before_a_win.1.clone()),
        "A's win changes no B state or save"
    );
    assert_eq!(
        economy(&b, &prizes),
        b_before_a_win.2,
        "A's reward never pays B"
    );
    assert_eq!(
        quest_count(&b, DELINQUENT_QUEST),
        b_before_a_win.3,
        "A's quest credit never advances B"
    );
    assert_eq!(
        active(&b),
        b_after_hit,
        "B keeps its own enemy HP, target and phase"
    );
    assert_eq!(
        active(&b)
            .enemies
            .iter()
            .find(|enemy| enemy.guid == b_target)
            .unwrap()
            .health,
        b_target_health
    );
    assert!(won_flag(&a_replies).is_some());
    assert_eq!(
        encounter_active(&b).map(|active| active.phase),
        Some(EncounterPhase::Fighting),
        "A's results screen does not open on B"
    );

    // B's own cooldown/target still work, then B wins its own fight.
    strike_once(&mut b, &mut b_now).expect("B can still attack after A's win");
    let b_replies = fight_to_win(&mut b, &mut b_now);
    assert_eq!(results_lifecycle(&b_replies), vec![18, 47, 19]);
    assert_eq!(won_flag(&b_replies), Some(1));
    let b_paid = economy(&b, &prizes);
    assert_payout(&b_before_a_win.2, &b_paid, row, &prizes);
    assert_eq!(quest_count(&b, DELINQUENT_QUEST), 2);
    // B's win does not touch A's finished state or wallet.
    assert_eq!(economy(&a, &prizes), a_paid);
    assert_eq!(quest_count(&a, DELINQUENT_QUEST), 2);
}

/// 3. Both players win, close their own results and return to their own
/// pre-entry pose. A live coin delta inserted mid-fight survives A's Close,
/// each save carries only its own wallet/XP/items/quest counters, and replaying
/// A's Close/Go pays nothing and does not start B.
#[test]
fn both_get_their_own_payout_and_return_to_their_own_pose() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, SHEEP_WATCH);
    let prizes = prize_ids(row);
    let mut a = fixture.seated_session(MARKER, 6201, 41, Some(delinquent_quests(6201, 1)));
    let mut b = fixture.seated_session(B_START, 6202, 108, Some(delinquent_quests(6202, 0)));
    assert_eq!(quest_count(&a, DELINQUENT_QUEST), 1);
    assert_eq!(quest_count(&b, DELINQUENT_QUEST), 0);
    let a_before = economy(&a, &prizes);
    let b_before = economy(&b, &prizes);

    let a_screen = click_marker(&mut a, 41, 400);
    go_and_enter(&mut a, &a_screen, 41, 500);
    let b_screen = click_marker(&mut b, 108, 400);
    go_and_enter(&mut b, &b_screen, 108, 500);

    let mut a_now = 1_000;
    let a_replies = fight_to_win(&mut a, &mut a_now);
    assert_eq!(results_lifecycle(&a_replies), vec![18, 47, 19]);
    let a_paid = economy(&a, &prizes);
    assert_payout(&a_before, &a_paid, row, &prizes);
    assert_eq!(quest_count(&a, DELINQUENT_QUEST), 3);

    let mut b_now = 1_000;
    let b_replies = fight_to_win(&mut b, &mut b_now);
    assert_eq!(results_lifecycle(&b_replies), vec![18, 47, 19]);
    let b_paid = economy(&b, &prizes);
    assert_payout(&b_before, &b_paid, row, &prizes);
    assert_eq!(quest_count(&b, DELINQUENT_QUEST), 2);

    // A live delta during the results screen: no old snapshot is restored over
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
    assert_eq!(a.state().zone_id, 0);
    assert_eq!(a.state().position, MARKER);
    assert!(encounter_active(&a).is_none());
    assert_eq!(economy(&a, &prizes), a_with_delta, "Close pays nothing");
    assert_eq!(quest_count(&a, DELINQUENT_QUEST), 3);

    // B is still in its own won arena, untouched by A's Close.
    assert_eq!(b.state().zone_id, ARENA_ZONE);
    assert_eq!(b.state().position, ARENA_ARRIVAL);
    assert_eq!(
        encounter_active(&b).map(|active| active.phase),
        Some(EncounterPhase::Won)
    );
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
    assert_eq!(b.state().zone_id, 0);
    assert_eq!(b.state().position, B_START);
    assert!(encounter_active(&b).is_none());
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
    assert_eq!(reopened_a.state().zone_id, 0);
    assert_eq!(reopened_a.state().position, MARKER);
    assert!(encounter_active(&reopened_a).is_none());
    assert_eq!(economy(&reopened_b, &prizes), b_paid);
    assert_eq!(quest_count(&reopened_b, DELINQUENT_QUEST), 2);
    assert_eq!(reopened_b.state().zone_id, 0);
    assert_eq!(reopened_b.state().position, B_START);
    assert!(encounter_active(&reopened_b).is_none());
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
    let enemy_guids = enemy_guids(row);
    let mut a = fixture.seated_session(MARKER, 6301, 41, None);
    let mut b = fixture.seated_session(B_START, 6302, 108, None);
    let a_screen = click_marker(&mut a, 41, 400);
    go_and_enter(&mut a, &a_screen, 41, 500);
    let b_screen = click_marker(&mut b, 108, 400);
    go_and_enter(&mut b, &b_screen, 108, 500);
    assert_eq!(
        encounter_active(&a).map(|active| active.phase),
        Some(EncounterPhase::Fighting)
    );
    assert_eq!(
        encounter_active(&b).map(|active| active.phase),
        Some(EncounterPhase::Fighting)
    );

    // The late joiner enters the same open-world field at the marker.
    let (mut c, c_entry) = fixture.session(MARKER, 6303);
    assert!(send(&mut c, &switch_job(BRAWLER), 300).state_changed);
    let replies = reply_bytes(&c_entry);
    assert_eq!(
        add_npc_packets(&c_entry, MARKER_GUID).len(),
        1,
        "the static marker is published exactly once to the late joiner"
    );
    for enemy in &enemy_guids {
        assert!(
            add_npc_packets(&c_entry, *enemy).is_empty(),
            "arena actor {enemy} is not in a third session's 35/2 batch"
        );
        assert!(
            !replies.iter().any(|packet| packet
                .windows(8)
                .any(|word| word == enemy.to_le_bytes().as_slice())),
            "arena actor {enemy} leaks into a third session's world entry"
        );
    }
    assert!(
        !replies
            .iter()
            .any(|packet| packet.starts_with(&[41, 0, 114, 0])),
        "another player's running encounter is not replayed to a late joiner"
    );
    assert!(encounter_active(&c).is_none());
    assert_eq!(c.state().zone_id, 0);

    // The marker click opens only C's own start screen; A and B are untouched.
    let a_before = (a.snapshot(), saved_bytes(&a));
    let b_before = (b.snapshot(), saved_bytes(&b));
    let c_screen = click_marker(&mut c, 1, 600);
    assert_eq!(i32_at(&c_screen, 4), SHEEP_WATCH as i32);
    assert!(encounter_active(&c).is_none());
    assert_eq!(c.state().zone_id, 0);
    assert_eq!((a.snapshot(), saved_bytes(&a)), a_before);
    assert_eq!((b.snapshot(), saved_bytes(&b)), b_before);

    // C's own Go starts C's own fight, not A's or B's.
    let c_go = send(&mut c, &go(&c_screen), 700);
    assert!(c_go.began_zoning);
    assert_eq!(active(&c).instance_id, 1);
    let ready = arena_spawned(&mut c, 800);
    assert_eq!(running_instance(&ready), 1);
    assert_eq!(
        encounter_active(&a).map(|active| active.phase),
        Some(EncounterPhase::Fighting)
    );
    assert_eq!(
        encounter_active(&b).map(|active| active.phase),
        Some(EncounterPhase::Fighting)
    );
    assert_eq!((a.snapshot(), saved_bytes(&a)), a_before);
    assert_eq!((b.snapshot(), saved_bytes(&b)), b_before);
    assert_eq!(active(&c).instance_id, 1);
}

/// 5. A's malformed packets are refused for A alone with byte-identical state,
/// and A's mid-fight disconnect drops only A's run. B keeps moving, attacking,
/// works through its own fight and returns; the test process never panics.
#[test]
fn one_players_disconnect_or_bad_packet_does_not_stop_the_other() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, SHEEP_WATCH);
    let prizes = prize_ids(row);
    let mut a = fixture.seated_session(MARKER, 6401, 41, None);
    let mut b = fixture.seated_session(B_START, 6402, 108, None);
    let a_screen = click_marker(&mut a, 41, 400);
    go_and_enter(&mut a, &a_screen, 41, 500);
    let b_screen = click_marker(&mut b, 108, 400);
    go_and_enter(&mut b, &b_screen, 108, 500);
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
    let a_paid = economy(&a, &prizes);
    assert_eq!(
        (b.snapshot(), saved_bytes(&b), economy(&b, &prizes)),
        b_before
    );

    // A disconnects mid-fight: the production save/reload path drops A's run
    // without paying, and leaves B's live run untouched.
    let mut dropped = reopen(&fixture, &a, "a-dropped.json", 2_000);
    assert_eq!(dropped.state().zone_id, 0);
    assert_eq!(dropped.state().position, MARKER);
    assert_eq!(economy(&dropped, &prizes), a_paid);
    assert!(encounter_active(&dropped).is_none());
    assert_eq!(
        (b.snapshot(), saved_bytes(&b), economy(&b, &prizes)),
        b_before
    );

    // B keeps moving, attacks, wins and returns.
    let b_guid = b.state().guid;
    let mut b_now = 2_100;
    walk(
        &mut b,
        b_guid,
        [ARENA_ARRIVAL[0] + 1.0, ARENA_ARRIVAL[1], ARENA_ARRIVAL[2]],
        &mut b_now,
    );
    assert_eq!(
        b.state().position,
        [ARENA_ARRIVAL[0] + 1.0, ARENA_ARRIVAL[1], ARENA_ARRIVAL[2]]
    );
    let b_replies = fight_to_win(&mut b, &mut b_now);
    assert_eq!(results_lifecycle(&b_replies), vec![18, 47, 19]);
    let b_paid = economy(&b, &prizes);
    assert_payout(&b_before.2, &b_paid, row, &prizes);
    let b_closed = send(&mut b, &[26, 0, 42, 0], b_now + 200);
    assert!(b_closed.began_zoning);
    assert_eq!(b.state().zone_id, 0);
    assert_eq!(b.state().position, B_START);
    assert!(encounter_active(&b).is_none());
    assert_eq!(economy(&b, &prizes), b_paid);

    // The dropped session has no stuck pending state: it can start a fresh
    // battle, and that start does not disturb B's returned state.
    let next = dropped
        .snapshot()
        .gameplay
        .expect("gameplay loads")
        .encounter
        .expect("encounter state loads")
        .next_instance_id;
    let screen = click_marker(&mut dropped, next, 3_000);
    assert!(send(&mut dropped, &go(&screen), 3_100).began_zoning);
    assert_eq!(active(&dropped).instance_id, next);
    assert_eq!(b.state().zone_id, 0);
    assert!(encounter_active(&b).is_none());
}

/// 6. The normal start/fight/return path uses no developer command. The
/// `realm win` shortcut is refused for a normal player role and cannot end a
/// live fight; the win here comes from the real 36/10 attacks.
#[test]
fn the_route_uses_no_dev_commands() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, SHEEP_WATCH);
    let prizes = prize_ids(row);
    let mut gateway = fixture.seated_session(MARKER, 6501, 41, None);
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
    let screen = click_marker(&mut gateway, 41, 500);
    let entered = send(&mut gateway, &go(&screen), 600);
    assert!(entered.began_zoning);
    assert_eq!(entered.dev_command_denials, 0);
    let ready = arena_spawned(&mut gateway, 700);
    assert_eq!(ready.dev_command_denials, 0);
    let mut now = 800;
    strike_once(&mut gateway, &mut now).expect("a real attack lands");
    // The same shortcut is still refused mid-fight and cannot end it.
    let mid_denied = send(&mut gateway, &chat::local_request("realm win"), now + 100);
    assert_eq!(mid_denied.dev_command_denials, 1);
    assert!(!mid_denied.state_changed && !mid_denied.began_zoning);
    assert_eq!(
        encounter_active(&gateway).map(|active| active.phase),
        Some(EncounterPhase::Fighting),
        "the refused developer win cannot win the fight"
    );
    let before_win = economy(&gateway, &prizes);
    let replies = fight_to_win(&mut gateway, &mut now);
    assert_eq!(results_lifecycle(&replies), vec![18, 47, 19]);
    assert_eq!(won_flag(&replies), Some(1), "the real attacks win");
    assert_payout(&before_win, &economy(&gateway, &prizes), row, &prizes);
    let closed = send(&mut gateway, &[26, 0, 42, 0], now + 200);
    assert!(closed.began_zoning);
    assert_eq!(closed.dev_command_denials, 0);
    assert_eq!(gateway.state().zone_id, 0);
    assert_eq!(gateway.state().position, MARKER);
    assert!(encounter_active(&gateway).is_none());
}
