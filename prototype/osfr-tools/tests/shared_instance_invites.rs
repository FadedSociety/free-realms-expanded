//! T440 (Shared World slice 5): the ordinary group encounter invitation over two real sockets.
//!
//! The party is formed with the family-40 codec. A starts battle 96 (marker click, then Go);
//! B decodes exactly one 41/102 with the recorded fields. Decline keeps both grouped and B's
//! location, membership and save bytes. Accept puts B into A's World instance through one
//! transfer, also when the reply carries a stale non-zero word B (a real client's `client+0xc10`
//! is its last 41/107 instance). A reply with no invite, an expired deadline (injected clock), a
//! replayed accept and an accept after the starter left the run change nothing and send no
//! message beyond ambient actor traffic. Only a Decline or a committed join consumes the invite
//! (the table is read through `test_has_invite`); a refused accept leaves it answerable. Accept
//! begins the same battle for B in A's run. Ungrouped and ineligible players get no invite.
//!
//! Client layouts (IDA): 41/102 reader `00a9f920`, 41/103 writer `00a97720` (see
//! `src/encounter_invite.rs`).
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
use group_battle_rig::{A, B, BATTLE, BRAWLER, Rig, is_sub};
use osfr_tools::encounter_invite::{self as invite, Reply};
use osfr_tools::world::{InstanceKind, World, WorldPose};
use std::{collections::BTreeSet, time::Duration};

/// A non-zero header word B: the last battle instance the invitee's client saw (`client+0xc10`).
const STALE_RUN: i32 = 7_777;

fn invites_in(messages: &[Vec<u8>]) -> Vec<invite::InvitePacket> {
    messages
        .iter()
        .filter(|m| is_sub(m, 41, 102))
        .map(|m| invite::decode_invite(m).expect("a strict 41/102"))
        .collect()
}

fn reply(encounter: i32, run: i32, character: u64, accept: bool) -> Reply {
    Reply {
        encounter,
        run,
        character,
        accept,
    }
}

#[test]
fn a_grouped_start_invites_the_member_and_only_accept_enters_the_run() {
    let mut rig = Rig::new(true, Some(BRAWLER));
    let battles = field_battle_fixture::battles();
    let title =
        field_battle_fixture::i32_field(field_battle_fixture::battle(&battles, BATTLE), "title_id");

    // No invite outstanding: an accept changes nothing.
    let before = rig.place(B);
    rig.answers(B, reply(0, 0, B, true));
    rig.assert_inert("a reply with no invite", &before);

    // Round 1: B gets exactly one prompt with the recorded fields.
    let run = rig.starts_battle(A);
    rig.pump(Duration::from_millis(600));
    let invites = invites_in(&rig.take(1));
    assert_eq!(
        invites,
        vec![invite::InvitePacket {
            encounter: BATTLE,
            run,
            inviter: A,
            title_id: title,
            countdown: invite::COUNTDOWN,
        }]
    );
    assert!(
        invites_in(&rig.take(0)).is_empty(),
        "the starter gets no prompt"
    );

    // Malformed shapes: the pre-codec forged layout (13 bytes), trailing bytes, accept flag 2.
    // They leave the invite answerable.
    let before = rig.place(B);
    let good = invite::encode_reply(&reply(0, 0, B, true));
    let mut flag = good.clone();
    flag[20] = 2;
    for (what, bytes) in [
        ("short", good[..13].to_vec()),
        ("trailing", [good.clone(), vec![0]].concat()),
        ("accept flag 2", flag),
    ] {
        rig.sends(B, &bytes);
        rig.assert_inert(what, &before);
        assert!(rig.has_invite(B), "{what}: the invite is still answerable");
    }

    // Decline with the stale word B a client that already saw a battle sends: the invite is
    // cleared and nothing else moves; both stay grouped.
    rig.answers(B, reply(0, STALE_RUN, B, false));
    rig.assert_inert("decline", &before);
    assert!(!rig.has_invite(B), "a decline consumes the invite");
    assert!(rig.group(A).is_some() && rig.group(A) == rig.group(B));
    // A late accept finds no invite.
    rig.answers(B, reply(0, 0, B, true));
    rig.assert_inert("accept after decline", &before);

    // Round 2: A leaves the run after inviting; B's accept is then inert (the run is gone).
    rig.leaves_battle(A, run);
    let run = rig.starts_battle(A);
    rig.pump(Duration::from_millis(600));
    assert_eq!(
        invites_in(&rig.take(1)).len(),
        1,
        "an invite for the new run"
    );
    rig.leaves_battle(A, run);
    let before = rig.place(B);
    rig.answers(B, reply(0, STALE_RUN, B, true));
    rig.assert_inert("accept after the starter left", &before);

    // Round 3: an expired reply (clock moved past the 30 s deadline) is inert.
    let run = rig.starts_battle(A);
    rig.pump(Duration::from_millis(600));
    assert_eq!(
        invites_in(&rig.take(1)).len(),
        1,
        "a fresh invite for the new run"
    );
    rig.host(|s| s.test_advance_clock(invite::TTL_MS + 1_000));
    let before = rig.place(B);
    rig.answers(B, reply(0, 0, B, true));
    rig.assert_inert("expired", &before);

    // Round 4: Accept enters A's instance through exactly one transfer. The reply carries the
    // stale word B, a wrong word A and another character's GUID: none of them is a key.
    rig.leaves_battle(A, run);
    let run = rig.starts_battle(A);
    rig.pump(Duration::from_millis(600));
    assert_eq!(invites_in(&rig.take(1)).len(), 1);
    let outside = rig.place(B);
    rig.answers(B, reply(95, STALE_RUN, A, true));
    rig.until("B in A's instance", |rig| {
        rig.host(move |s| s.test_location(B).is_some_and(|row| row.1 == run))
    });
    rig.ready(B);
    rig.pump(Duration::from_millis(600));
    let received = rig.take(1);
    assert_eq!(
        received.iter().filter(|m| m.starts_with(&[31, 0])).count(),
        1,
        "exactly one transfer"
    );
    let at_a = rig.host(|s| s.test_location(A).unwrap());
    let at_b = rig.host(|s| s.test_location(B).unwrap());
    assert_eq!((at_b.0.zone_id, at_b.1), (9600, run));
    assert_eq!(at_a.1, at_b.1, "B shares A's instance id");
    assert_eq!(at_b.3, BTreeSet::from([A, B]));
    // The battle itself began for B: the same battle, in A's run, live (not just a transfer).
    let title =
        field_battle_fixture::i32_field(field_battle_fixture::battle(&battles, BATTLE), "title_id");
    assert_eq!(
        rig.host(|s| s.test_joinable_run(A)),
        Some((BATTLE, run, title))
    );
    assert_eq!(
        rig.host(|s| s.test_joinable_run(B)),
        Some((BATTLE, run, title))
    );
    assert_ne!(outside, rig.place(B));
    assert!(!rig.has_invite(B), "the committed join consumed the invite");
    assert!(rig.group(A).is_some() && rig.group(A) == rig.group(B));

    // A replayed accept sends nothing and moves nobody.
    let before = rig.place(B);
    rig.answers(B, reply(0, 0, B, true));
    rig.assert_inert("replayed accept", &before);
}

#[test]
fn an_accept_rechecks_the_party_and_the_go_gates_after_the_invite() {
    let mut rig = Rig::new(true, Some(BRAWLER));
    // Each round: A starts the battle and B holds one live invite; then B's standing changes
    // and B accepts. The invite is still outstanding, so only Accept's own rechecks refuse.
    let round = |rig: &mut Rig, what: &str, change: fn(&mut Rig)| {
        let run = rig.starts_battle(A);
        rig.pump(Duration::from_millis(600));
        assert_eq!(invites_in(&rig.take(1)).len(), 1, "{what}: one invite");
        change(rig);
        rig.pump(Duration::from_millis(600));
        let before = rig.place(B);
        rig.answers(B, reply(0, STALE_RUN, B, true));
        rig.assert_inert(what, &before);
        let at = rig.host(|s| s.test_location(B).unwrap());
        assert_ne!(at.1, run, "{what}: B is not in A's run");
        rig.leaves_battle(A, run);
    };
    // B leaves the party after the invite: no longer grouped with A, so Accept enters nothing.
    round(&mut rig, "B left the party", |rig| rig.leave_party(B));
    assert!(rig.group(A).is_none() && rig.group(B).is_none());
    rig.form_party();
    // A leaves the party after the invite (A stays in its run).
    round(&mut rig, "A left the party", |rig| rig.leave_party(A));
    assert!(rig.group(A).is_none() && rig.group(B).is_none());
    rig.form_party();
    // B drops the combat job after the invite: Accept's Go gates refuse it.
    round(&mut rig, "B lost the combat job", Rig::b_goes_idle);
    assert!(rig.group(A).is_some() && rig.group(A) == rig.group(B));
}

#[test]
fn a_refused_accept_leaves_the_invite_answerable() {
    let mut rig = Rig::new(true, Some(BRAWLER));
    let run = rig.starts_battle(A);
    rig.pump(Duration::from_millis(600));
    assert_eq!(invites_in(&rig.take(1)).len(), 1);
    // B drops the combat job, so Accept's Go gates refuse it: nothing moves and no message goes.
    rig.b_goes_idle();
    rig.pump(Duration::from_millis(600));
    let before = rig.place(B);
    rig.answers(B, reply(0, STALE_RUN, B, true));
    rig.assert_inert("accept without a combat job", &before);
    assert_eq!(rig.host(|s| s.test_joinable_run(B)), None);
    assert!(rig.has_invite(B), "a refused accept keeps the invite");
    // B picks the combat job back up and answers the same invite: it was not burned.
    rig.take(1);
    let job = field_battle_fixture::switch_job(BRAWLER);
    rig.bot(B).send_message(&bot::wrap(&job), "26/13").unwrap();
    rig.until("B's job change back", |rig| {
        rig.inbox[1].iter().any(|m| is_sub(m, 38, 21))
    });
    rig.answers(B, reply(0, 0, B, true));
    rig.until("B in A's instance", |rig| {
        rig.host(move |s| s.test_location(B).is_some_and(|row| row.1 == run))
    });
    assert_eq!(rig.host(|s| s.test_joinable_run(B)).map(|r| r.1), Some(run));
}

/// A runtime World with one encounter run (zone 9600, child of the public instance) and its id.
fn encounter_world() -> (World, i32) {
    let mut world = World::runtime();
    world
        .register_runtime_zone(osfr_tools::content::Zone {
            id: 9600,
            name: "9600".into(),
            geometry_id: Some(9600),
            zone_type: Some(2),
            tutorial: Some(false),
            sky: Some(String::new()),
            source: Vec::new(),
        })
        .unwrap();
    let run = world
        .create_child_instance(0, 9600, InstanceKind::Encounter { encounter_id: 0 }, 0)
        .unwrap();
    (world, run)
}

/// One membership install, as the gateway's commit step does it.
fn place(world: &mut World, guid: u64, zone: u32, target: Option<i32>) {
    let change = world
        .prepare_membership(guid, zone, target, WorldPose::ORIGIN, true, 0)
        .unwrap();
    world.commit_membership(change);
}

/// A's invite for B into `run`, expiring at 30 s.
fn row(run: i32) -> invite::Pending {
    invite::Pending {
        inviter: A,
        run,
        encounter: BATTLE,
        deadline_ms: 30_000,
    }
}

#[test]
fn the_world_sweep_voids_an_invite_when_a_player_or_the_run_goes_away() {
    let (mut world, run) = encounter_world();
    let offered = |world: &mut World| {
        place(world, A, 9600, Some(run));
        place(world, B, 0, None);
        assert!(world.invites_mut().offer(B, row(run), 0, 16));
        world.collect_empty_instances(1_000);
        assert_eq!(
            world.invites_mut().len(),
            1,
            "both placed, inviter in its run"
        );
    };
    offered(&mut world);
    // The inviter leaves the run.
    place(&mut world, A, 0, None);
    world.collect_empty_instances(1_000);
    assert!(world.invites_mut().is_empty(), "inviter left the run");
    // The invitee disconnects (the server's `end_session` calls `leave`).
    offered(&mut world);
    world.leave(B, 2_000);
    world.collect_empty_instances(2_000);
    assert!(world.invites_mut().is_empty(), "invitee gone");
    // The inviter disconnects.
    offered(&mut world);
    world.leave(A, 3_000);
    world.collect_empty_instances(3_000);
    assert!(world.invites_mut().is_empty(), "inviter gone");
    // The deadline passes.
    offered(&mut world);
    world.collect_empty_instances(30_000);
    assert!(world.invites_mut().is_empty(), "expired");
}

#[test]
fn the_membership_install_that_places_the_invitee_in_the_run_removes_the_invite() {
    let (mut world, run) = encounter_world();
    place(&mut world, A, 9600, Some(run));
    place(&mut world, B, 0, None);
    assert!(world.invites_mut().offer(B, row(run), 0, 16));
    // Moving elsewhere, and another player entering the run, answer nothing.
    place(&mut world, B, 0, None);
    place(&mut world, 303, 9600, Some(run));
    assert_eq!(world.invites_mut().len(), 1);
    // B landing in the invited run is the answer: gone in the same install.
    place(&mut world, B, 9600, Some(run));
    assert!(world.invites_mut().is_empty());
}

#[test]
fn an_ungrouped_or_ineligible_player_gets_no_invite() {
    // Ungrouped: the solo Go sends B nothing.
    let mut rig = Rig::new(false, Some(BRAWLER));
    rig.starts_battle(A);
    rig.pump(Duration::from_millis(800));
    assert!(invites_in(&rig.take(1)).is_empty());
    assert!(invites_in(&rig.take(0)).is_empty());
    drop(rig);
    // Grouped but without a combat job: B could not press Go, so it gets no prompt (R).
    let mut rig = Rig::new(true, None);
    rig.starts_battle(A);
    rig.pump(Duration::from_millis(800));
    assert!(invites_in(&rig.take(1)).is_empty());
}
