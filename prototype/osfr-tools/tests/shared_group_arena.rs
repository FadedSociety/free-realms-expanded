//! T441 (Shared World slice 6): grouped players in one battle run share one arena, over real
//! sockets. The party is a real family-40 party; A starts battle 96 and B accepts the original
//! 41/102 prompt (T440). From then on the run has one summoner: one health, one minion set, one
//! driver tick for its enemies, one win; each member keeps their own health, knockout, results
//! and payout. Two parties at one marker, an ungrouped player, a disconnecting member and a
//! finished or abandoned run stay apart.
//!
//! Combat: every player is a Brawler (3 m reach: the rig walks up to the enemy, then hits) at
//! level 4 (one hit is 444, so the 75% crossing is two hits) or level 16 (2,382, so a win takes
//! a few). Client layouts: 36/10 writer `00a342e0`, 35/5 enemy health, 35/2 AddNpc `00921a10`,
//! 32/7 attack processed, 39/18 game over, 39/45 prize wheel (see `encounter.rs`).
#[path = "../src/bin/osfr-bot.rs"]
#[allow(dead_code)]
mod bot;
#[path = "common/field_battle_fixture.rs"]
mod field_battle_fixture;
#[path = "../src/bin/osfr-gateway.rs"]
#[allow(dead_code)]
mod gateway_runner;
#[path = "common/group_battle_rig.rs"]
mod group_battle_rig;
#[path = "common/group_invite_wire.rs"]
mod group_invite_wire;
use field_battle_fixture::{Economy, assert_payout, economy, prize_ids};
use group_battle_rig::{A, B, BATTLE, BRAWLER, C, D, Rig, is_sub};
use osfr_tools::{
    encounter::{EncounterPhase, SharedArena},
    encounter_invite::Reply,
};
use std::{collections::BTreeSet, time::Duration};

const SUMMONER: u64 = 100_000_096_001;
/// The summoner's minions: two per 75/50/25% tier.
const MINIONS: [u64; 6] = [
    100_000_096_002,
    100_000_096_003,
    100_000_096_004,
    100_000_096_005,
    100_000_096_006,
    100_000_096_007,
];
/// The job XP one battle win pays (R, `BATTLE_JOB_XP`).
const WIN_XP: u32 = 50;

fn accept(who: u64) -> Reply {
    Reply {
        encounter: 0,
        run: 0,
        character: who,
        accept: true,
    }
}

fn arena_of(rig: &Rig, guid: u64) -> Option<(i32, SharedArena)> {
    rig.host(move |s| s.test_arena(guid))
}

fn health(arena: &SharedArena, guid: u64) -> u32 {
    arena
        .enemies()
        .iter()
        .find(|e| e.guid == guid)
        .unwrap()
        .health
}

/// Minions that are up and standing in the shared run.
fn awake(arena: &SharedArena) -> Vec<u64> {
    arena
        .enemies()
        .iter()
        .filter(|e| e.guid != SUMMONER && !e.dormant && e.health > 0)
        .map(|e| e.guid)
        .collect()
}

fn guid_at(message: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(message[at..at + 8].try_into().unwrap())
}

/// 35/2 AddNpc packets for `guid` in `messages`.
fn spawns(messages: &[Vec<u8>], guid: u64) -> usize {
    messages
        .iter()
        .filter(|m| is_sub(m, 35, 2) && guid_at(m, 4) == guid)
        .count()
}

/// The 35/5 health values sent for `guid`, in order.
fn hp_updates(messages: &[Vec<u8>], guid: u64) -> Vec<u32> {
    messages
        .iter()
        .filter(|m| is_sub(m, 35, 5) && guid_at(m, 4) == guid)
        .map(|m| u32::from_le_bytes(m[12..16].try_into().unwrap()))
        .collect()
}

/// 32/7 attack-processed packets whose attacker is `guid`.
fn swings_by(messages: &[Vec<u8>], guid: u64) -> usize {
    messages
        .iter()
        .filter(|m| is_sub(m, 32, 7) && guid_at(m, 4) == guid)
        .count()
}

fn game_overs(messages: &[Vec<u8>]) -> usize {
    messages
        .iter()
        .filter(|m| m.starts_with(&[39, 0, 18]))
        .count()
}

fn phase(rig: &Rig, guid: u64) -> Option<EncounterPhase> {
    rig.host(move |s| {
        s.test_gateway(guid)?
            .snapshot()
            .gameplay?
            .encounter?
            .active
            .map(|run| run.phase)
    })
}

fn econ(rig: &Rig, guid: u64, prizes: &BTreeSet<i32>) -> Economy {
    let prizes = prizes.clone();
    rig.host(move |s| economy(s.test_gateway(guid).unwrap(), &prizes))
}

/// `starter` starts the battle, `joiner` accepts the prompt and loads in: both are members of
/// one arena in the starter's run.
fn join(rig: &mut Rig, starter: u64, joiner: u64) -> i32 {
    let run = rig.starts_battle(starter);
    rig.pump(Duration::from_millis(600));
    rig.answers(joiner, accept(joiner));
    rig.until("the joiner in the run", |rig| {
        rig.host(move |s| s.test_location(joiner).is_some_and(|row| row.1 == run))
    });
    rig.ready(joiner);
    rig.pump(Duration::from_millis(600));
    let (instance, arena) = arena_of(rig, starter).expect("the accept opened a shared arena");
    assert_eq!(instance, run);
    assert_eq!(arena.members, BTreeSet::from([starter, joiner]));
    run
}

fn clear_inboxes(rig: &mut Rig) {
    for seat in 0..rig.bots.len() {
        rig.take(seat);
    }
}

#[test]
fn two_members_share_one_summoner_one_health_and_one_minion_set() {
    let mut rig = Rig::fighters(2, BRAWLER, 4);
    let run = rig.starts_battle(A);
    rig.pump(Duration::from_millis(600));

    // A hits once before B has joined: the run is still A's alone.
    rig.land(A, SUMMONER);
    let after_a = hp_updates(&rig.inbox[0], SUMMONER).last().copied().unwrap();
    assert!(
        arena_of(&rig, A).is_none(),
        "a lone run has no shared arena"
    );

    // B accepts. Gate 1: B gets the arena's current actors and health, and both are members.
    rig.take(1);
    rig.answers(B, accept(B));
    rig.until("B in the run", |rig| {
        rig.host(move |s| s.test_location(B).is_some_and(|row| row.1 == run))
    });
    rig.ready(B);
    // The late-join burst: the arena's actors at their current health.
    rig.until("B's late-join burst", |rig| {
        spawns(&rig.inbox[1], SUMMONER) > 0
            && hp_updates(&rig.inbox[1], SUMMONER).last() == Some(&after_a)
    });
    rig.pump(Duration::from_millis(300));
    let (instance_a, arena) = arena_of(&rig, A).expect("the accept opened a shared arena");
    let (instance_b, _) = arena_of(&rig, B).expect("B is a member");
    assert_eq!((instance_a, instance_b), (run, run));
    assert_eq!(arena.members, BTreeSet::from([A, B]));
    assert_eq!(
        health(&arena, SUMMONER),
        after_a,
        "the arena starts from A's run"
    );
    let burst = rig.take(1);
    assert_eq!(spawns(&burst, SUMMONER), 1, "B's burst spawns the summoner");
    assert_eq!(
        hp_updates(&burst, SUMMONER).last(),
        Some(&after_a),
        "at the health A left it"
    );

    // Gate 2: B and A hit alternately. One canonical health falls by every hit, and each hit
    // shows in the other member's view, until the 75% threshold calls the first minions in.
    clear_inboxes(&mut rig);
    let mut last = after_a;
    for round in 0..8 {
        if !awake(&arena_of(&rig, A).unwrap().1).is_empty() {
            break;
        }
        let (who, other) = if round % 2 == 0 { (B, 0) } else { (A, 1) };
        rig.land(who, SUMMONER);
        let now = health(&arena_of(&rig, A).unwrap().1, SUMMONER);
        assert!(
            now < last,
            "round {round}: {who}'s hit lowers the one health"
        );
        last = now;
        assert_eq!(
            hp_updates(&rig.inbox[other], SUMMONER).last(),
            Some(&now),
            "the other member sees the one health"
        );
    }
    let (_, arena) = arena_of(&rig, A).unwrap();
    let minions = awake(&arena);
    assert!(!minions.is_empty(), "the 75% threshold was crossed");
    assert!(minions.iter().all(|guid| MINIONS.contains(guid)));
    for guid in &minions {
        for seat in 0..2 {
            assert_eq!(
                spawns(&rig.inbox[seat], *guid),
                1,
                "minion {guid} spawned once for member {seat}"
            );
        }
    }
}

#[test]
fn a_refused_hit_changes_nothing_in_the_shared_run() {
    let mut rig = Rig::fighters(2, BRAWLER, 16);
    join(&mut rig, A, B);
    rig.pump(Duration::from_millis(500));
    clear_inboxes(&mut rig);
    let (_, before) = arena_of(&rig, A).unwrap();
    // A still stands on the arena spawn: the summoner is a thousand metres off, out of reach,
    // so the hit is refused. The enemies may walk (a lone summoner kites); their health may not.
    rig.hit(A, SUMMONER);
    rig.pump(Duration::from_millis(600));
    let (_, after) = arena_of(&rig, A).unwrap();
    let healths = |arena: &SharedArena| {
        arena
            .enemies()
            .iter()
            .map(|e| (e.guid, e.health))
            .collect::<Vec<_>>()
    };
    assert_eq!(healths(&after), healths(&before), "the refusal changed HP");
    assert_eq!(after.attack_clock(SUMMONER), before.attack_clock(SUMMONER));
    assert_eq!(after.members, before.members);
    assert_eq!(phase(&rig, A), Some(EncounterPhase::Fighting));
    for seat in 0..2 {
        assert!(
            hp_updates(&rig.inbox[seat], SUMMONER).is_empty(),
            "member {seat} was sent the refused hit"
        );
    }
}

#[test]
fn a_win_pays_every_member_once_and_a_close_returns_only_that_member() {
    let mut rig = Rig::fighters(2, BRAWLER, 16);
    let battles = field_battle_fixture::battles();
    let row = field_battle_fixture::battle(&battles, BATTLE).clone();
    let prizes = prize_ids(&row);
    let run = join(&mut rig, A, B);
    let (before_a, before_b) = (econ(&rig, A, &prizes), econ(&rig, B, &prizes));

    // A's first hit calls every minion in; B clears them; A lands the last hit. B never acts
    // after the minions fall.
    rig.land(A, SUMMONER);
    for _ in 0..MINIONS.len() {
        let (_, arena) = arena_of(&rig, B).unwrap();
        let Some(target) = awake(&arena).first().copied() else {
            break;
        };
        rig.land(B, target);
    }
    let (_, arena) = arena_of(&rig, A).unwrap();
    assert!(awake(&arena).is_empty() && health(&arena, SUMMONER) > 0);
    clear_inboxes(&mut rig);
    rig.land(A, SUMMONER);
    rig.until("the win", |rig| {
        rig.host(move |s| s.test_arena(A).is_some_and(|(_, arena)| arena.won()))
    });
    rig.until("B's results", |rig| game_overs(&rig.inbox[1]) > 0);
    rig.pump(Duration::from_millis(500));
    let (a_inbox, b_inbox) = (rig.take(0), rig.take(1));
    assert_eq!(game_overs(&a_inbox), 1, "A's results once");
    assert_eq!(
        game_overs(&b_inbox),
        1,
        "B's results once, though B did not act"
    );
    assert!(
        a_inbox.iter().any(|m| m.starts_with(&[39, 0, 45])),
        "A's prize wheel"
    );
    assert!(
        b_inbox.iter().any(|m| m.starts_with(&[39, 0, 45])),
        "B's prize wheel"
    );
    assert_eq!(phase(&rig, A), Some(EncounterPhase::Won));
    assert_eq!(phase(&rig, B), Some(EncounterPhase::Won));

    // Each member is paid exactly once: coins, one prize, the active job's XP.
    let (after_a, after_b) = (econ(&rig, A, &prizes), econ(&rig, B, &prizes));
    assert_payout(&before_a, &after_a, &row, &prizes, WIN_XP);
    assert_payout(&before_b, &after_b, &row, &prizes, WIN_XP);

    // A closes its results: only A returns. B is still in the run, still Won, unpaid again.
    rig.leaves_battle(A, run);
    let at_b = rig.host(|s| s.test_location(B).unwrap());
    assert_eq!((at_b.0.zone_id, at_b.1), (9600, run), "B stays in the run");
    assert_eq!(phase(&rig, B), Some(EncounterPhase::Won));
    // A replayed Close pays nothing and moves nobody.
    let settled = econ(&rig, A, &prizes);
    rig.leaves_battle(A, run);
    assert_eq!(econ(&rig, A, &prizes), settled);
    assert_eq!(econ(&rig, B, &prizes), after_b);
    rig.leaves_battle(B, run);
    assert_eq!(econ(&rig, B, &prizes), after_b, "closing pays nothing more");
    assert_eq!(econ(&rig, A, &prizes), settled);
}

/// F839: the win fan-out commits each member alone. B's save fails: B keeps nothing, sees no
/// results and the run is not frozen; once B's save works again, a later tick pays B once.
#[test]
fn a_member_whose_win_save_fails_is_paid_once_on_a_later_tick() {
    let mut rig = Rig::fighters(2, BRAWLER, 16);
    let battles = field_battle_fixture::battles();
    let row = field_battle_fixture::battle(&battles, BATTLE).clone();
    let prizes = prize_ids(&row);
    join(&mut rig, A, B);
    rig.land(A, SUMMONER);
    for _ in 0..MINIONS.len() {
        let (_, arena) = arena_of(&rig, B).unwrap();
        let Some(target) = awake(&arena).first().copied() else {
            break;
        };
        rig.land(B, target);
    }
    let before_b = econ(&rig, B, &prizes);
    let real = rig.host(|s| {
        let path = s.test_state_path(B);
        let real = path.clone();
        *path = real.join("blocked.json"); // under B's save file: every save fails
        real
    });
    clear_inboxes(&mut rig);
    rig.land(A, SUMMONER);
    rig.until("A's results", |rig| game_overs(&rig.inbox[0]) > 0);
    rig.pump(Duration::from_millis(500)); // several ticks; B's fan-out fails on each
    assert_eq!(game_overs(&rig.take(1)), 0, "B was shown a win it could not keep");
    assert_eq!(phase(&rig, B), Some(EncounterPhase::Fighting));
    assert_eq!(econ(&rig, B, &prizes), before_b, "B's failed save changed B");
    let (_, arena) = arena_of(&rig, B).expect("B is still a member");
    assert!(arena.won() && !arena.faulted, "a save failure froze the run");

    rig.host(move |s| *s.test_state_path(B) = real);
    rig.until("B's results", |rig| game_overs(&rig.inbox[1]) > 0);
    rig.pump(Duration::from_millis(500));
    assert_eq!(game_overs(&rig.take(1)), 1, "B's results once");
    assert_eq!(phase(&rig, B), Some(EncounterPhase::Won));
    assert_payout(&before_b, &econ(&rig, B, &prizes), &row, &prizes, WIN_XP);
}

#[test]
fn two_parties_at_one_marker_run_apart() {
    let mut rig = Rig::fighters(4, BRAWLER, 16);
    let first = join(&mut rig, A, B);
    let second = join(&mut rig, C, D);
    assert_ne!(first, second, "each party has its own run");
    assert_eq!(rig.host(|s| s.test_arena_count()), 2);
    let (_, one) = arena_of(&rig, A).unwrap();
    let (_, two) = arena_of(&rig, C).unwrap();
    assert!(one.members.is_disjoint(&two.members));
    assert_eq!(arena_of(&rig, B).unwrap().0, first);
    assert_eq!(arena_of(&rig, D).unwrap().0, second);
    let full = health(&two, SUMMONER);

    clear_inboxes(&mut rig);
    rig.land(A, SUMMONER);
    rig.pump(Duration::from_millis(800));
    let (_, one) = arena_of(&rig, B).unwrap();
    let (_, two) = arena_of(&rig, D).unwrap();
    assert!(
        health(&one, SUMMONER) < full,
        "the first run's summoner took the hit"
    );
    assert_eq!(health(&two, SUMMONER), full, "the second run's did not");
    // A late retransmit of its own scene load may still arrive: what must not is the new health.
    let hit = health(&one, SUMMONER);
    for seat in [2, 3] {
        assert!(
            !hp_updates(&rig.inbox[seat], SUMMONER).contains(&hit),
            "the other party saw nothing of it"
        );
    }
    assert_eq!(
        hp_updates(&rig.inbox[1], SUMMONER).last(),
        Some(&hit),
        "A's partner saw it"
    );
}

#[test]
fn a_disconnecting_member_leaves_the_fight_and_its_enemies_turn_on_the_other() {
    let mut rig = Rig::fighters(2, BRAWLER, 4);
    let run = join(&mut rig, A, B);
    // A calls the 75% tier in; B then hits one minion, which survives it and so is B's.
    for _ in 0..8 {
        if !awake(&arena_of(&rig, A).unwrap().1).is_empty() {
            break;
        }
        rig.land(A, SUMMONER);
    }
    let minion = *awake(&arena_of(&rig, A).unwrap().1)
        .first()
        .expect("a minion is up");
    rig.land(B, minion);
    let (_, arena) = arena_of(&rig, B).unwrap();
    assert!(
        health(&arena, minion) > 0,
        "B's hit left the minion standing"
    );
    // It walks over and swings at B alone: A is never its target while B is.
    clear_inboxes(&mut rig);
    rig.until("the minion's swing at B", |rig| {
        swings_by(&rig.inbox[1], minion) > 0
    });
    assert_eq!(
        swings_by(&rig.inbox[0], minion),
        0,
        "A is not the minion's target"
    );

    // B drops its connection mid-fight: it leaves the arena, and the minion turns on A.
    rig.disconnect(B);
    rig.until("B out of the arena", |rig| {
        rig.host(|s| {
            s.test_arena(A)
                .is_some_and(|(_, arena)| arena.members.len() == 1)
        })
    });
    rig.take(0);
    rig.until("the minion turning on A", |rig| {
        swings_by(&rig.inbox[0], minion) > 0
    });
    // A is still in the fight: its next hit lands on the one summoner.
    let (_, arena) = arena_of(&rig, A).unwrap();
    let before = health(&arena, SUMMONER);
    rig.land(A, minion);
    assert!(health(&arena_of(&rig, A).unwrap().1, minion) < health(&arena, minion));
    assert_eq!(health(&arena_of(&rig, A).unwrap().1, SUMMONER), before);

    // B reconnects outside the run, at the safe pose the battle started from.
    rig.reconnect(B);
    let (state, location) = rig.host(|s| {
        (
            s.test_gateway(B).unwrap().state().clone(),
            s.test_location(B),
        )
    });
    assert_eq!(state.zone_id, 0, "B is back in the open world");
    assert_ne!(location.unwrap().1, run, "B is not in the run");
    assert!(arena_of(&rig, B).is_none(), "B is in no arena");
    let battles = field_battle_fixture::battles();
    let marker = field_battle_fixture::position(
        &field_battle_fixture::battle(&battles, BATTLE)["spawner"]["position"],
    );
    assert_eq!(state.position, marker, "B stands where the battle began");
    assert_eq!(rig.host(|s| s.test_arena(A).unwrap().1.members.len()), 1);
}

#[test]
fn a_knockout_is_personal_and_an_ex_member_gets_a_solo_run() {
    let mut rig = Rig::fighters(2, BRAWLER, 16);
    let run = join(&mut rig, A, B);
    // A is knocked out: A's results show, B fights on, the run is not lost.
    clear_inboxes(&mut rig);
    rig.host(|s| s.test_knock_out(A, 1_000));
    rig.pump(Duration::from_millis(500));
    assert_eq!(phase(&rig, A), Some(EncounterPhase::Lost));
    assert_eq!(phase(&rig, B), Some(EncounterPhase::Fighting));
    assert_eq!(game_overs(&rig.take(0)), 1, "A's own results");
    assert_eq!(game_overs(&rig.take(1)), 0, "B's fight goes on");
    assert!(!arena_of(&rig, A).unwrap().1.lost());
    // Every member knocked out: the run is lost and takes nobody new.
    rig.host(|s| s.test_knock_out(B, 1_100));
    rig.until("the run lost", |rig| {
        rig.host(|s| s.test_arena(B).is_some_and(|(_, arena)| arena.lost()))
    });
    assert!(!arena_of(&rig, B).unwrap().1.admits());

    // Both close; the arena goes with its members.
    rig.leaves_battle(A, run);
    rig.leaves_battle(B, run);
    rig.until("the arena gone", |rig| {
        rig.host(|s| s.test_arena_count()) == 0
    });
    // B, an ex-member, starts again: its own run, in an instance of its own, no arena.
    let again = rig.starts_battle(B);
    assert_ne!(again, run, "a new run, a new instance");
    assert!(arena_of(&rig, B).is_none(), "an ex-member's run is solo");
    clear_inboxes(&mut rig);
    rig.land(B, SUMMONER);
    assert!(
        rig.inbox[0].iter().all(|m| !is_sub(m, 35, 5)),
        "A sees none of B's run"
    );
    assert_eq!(rig.host(|s| s.test_arena_count()), 0);
}

/// What a hit itself answers with, minus what depends on where the kiting summoner stood: the
/// enemies' own motion and swings are not part of it, and positions are left out.
fn hit_answer(messages: &[Vec<u8>]) -> Vec<Vec<u8>> {
    messages
        .iter()
        .filter_map(|m| {
            let swing = |id: i32| [1112, 1404].contains(&id);
            let id = || i32::from_le_bytes(m[12..16].try_into().unwrap());
            if is_sub(m, 35, 2) || is_sub(m, 35, 16) {
                Some(m[..12].to_vec())
            } else if is_sub(m, 35, 5) || is_sub(m, 35, 12) {
                Some(m.clone())
            } else if is_sub(m, 32, 7) && guid_at(m, 4) == 1 {
                Some(m.clone())
            } else if is_sub(m, 35, 8) && (guid_at(m, 4) == 1 || swing(id())) {
                Some(m.clone())
            } else {
                None
            }
        })
        .collect()
}

/// A solo player's one hit, alone or with a grouped fight going on in another run.
fn solo_hit(grouped_fight: bool) -> Vec<Vec<u8>> {
    let mut rig = Rig::fighters(4, BRAWLER, 16);
    // C and D fight alone: the party is dissolved so that neither is invited or joined.
    rig.leave_party(C);
    if grouped_fight {
        join(&mut rig, A, B);
        rig.land(A, SUMMONER);
        rig.land(B, MINIONS[0]);
    }
    rig.starts_battle(C);
    rig.pump(Duration::from_millis(600));
    clear_inboxes(&mut rig);
    rig.land(C, SUMMONER);
    assert!(arena_of(&rig, C).is_none(), "C's run is solo");
    hit_answer(&rig.take(2))
}

#[test]
fn a_solo_run_answers_the_same_with_or_without_a_group_fight_elsewhere() {
    let alone = solo_hit(false);
    assert!(
        alone.len() >= 8,
        "the hit summoned the minions: {}",
        alone.len()
    );
    assert_eq!(solo_hit(true), alone);
}
