use super::waypoints::{Waypoint, Waypoints};
use super::*;
use crate::movement::{
    Decision, ExpectedDiscontinuity, MovementState, MovementUpdate, Reason, SpeedModifiers,
    StaffMotion, validate,
};
use crate::players::{InterestPolicy, MessageKind, PlayerRegistry, SessionId, ZoneInstance};
use std::time::Duration;

#[test]
fn g5_chat_words_parse_with_their_role_keys() {
    let name = |text: &str| Who::Name(text.into());
    for (text, key, power) in [
        (
            "realm spectate Alpha Test",
            "spectate",
            Power::Spectate(name("Alpha Test")),
        ),
        ("REALM Unspectate", "unspectate", Power::Unspectate),
        ("realm invisible", "invisible", Power::Invisible(true)),
        ("realm visible", "visible", Power::Invisible(false)),
        (
            "realm freeze 42",
            "freeze",
            Power::Freeze(Who::Guid(42), true),
        ),
        (
            "realm unfreeze bravo test",
            "unfreeze",
            Power::Freeze(name("bravo test"), false),
        ),
        (
            "realm follow Bravo Test",
            "follow",
            Power::Follow(name("Bravo Test"), FOLLOW_DEFAULT_M),
        ),
        (
            "realm follow Bravo Test 7",
            "follow",
            Power::Follow(name("Bravo Test"), 7.0),
        ),
        ("realm unfollow", "unfollow", Power::Unfollow),
        (
            "realm bring Bravo Test",
            "bring",
            Power::Bring(name("Bravo Test")),
        ),
        (
            "realm goto Bravo Test",
            "goto",
            Power::Goto(name("Bravo Test")),
        ),
        ("realm fly", "fly", Power::Fly(true)),
        ("realm fly off", "fly", Power::Fly(false)),
        (
            "realm waypoint save Snow Peak",
            "waypoint",
            Power::Waypoint(WaypointOp::Save("Snow Peak".into())),
        ),
        (
            "realm wp go snow peak",
            "waypoint",
            Power::Waypoint(WaypointOp::Go("snow peak".into())),
        ),
        (
            "realm waypoint list",
            "waypoint",
            Power::Waypoint(WaypointOp::List),
        ),
        (
            "realm waypoint del Snow Peak",
            "waypoint",
            Power::Waypoint(WaypointOp::Delete("Snow Peak".into())),
        ),
    ] {
        let (parsed_key, parsed) = parse(text).unwrap_or_else(|| panic!("{text}: not a G5 word"));
        assert_eq!(parsed_key, key, "{text}");
        assert_eq!(parsed, Ok(power.clone()), "{text}");
        assert_eq!(power.key(), key, "{text}");
    }
}

#[test]
fn g5_other_words_and_teleport_gotos_stay_with_gameplay() {
    for text in [
        "realm move Bravo Test 1 2 3",
        "realm send Bravo Test snowhill",
        "realm goto",
        "realm goto 10 20",
        "realm goto 10, 2, 20",
        "realm goto 9000000016",
        "realm goto mark home",
        "realm goto list farnum",
        "realm god",
        "hello realm spectate",
        "realm",
    ] {
        assert_eq!(parse(text), None, "{text}");
    }
    for text in [
        "realm spectate",
        "realm freeze",
        "realm follow Bravo Test 99",
        "realm follow",
        "realm fly sideways",
        "realm waypoint",
        "realm waypoint save",
        "realm waypoint list extra",
        "realm waypoint save this label is far longer than thirty two characters",
    ] {
        let (_, parsed) = parse(text).unwrap_or_else(|| panic!("{text}: not a G5 word"));
        assert!(parsed.is_err(), "{text}: {parsed:?}");
    }
}

/// The role table: moderators get spectate, freeze and goto; admins everything; testers keep
/// `goto` only; a player nothing.
#[test]
fn g5_facade_gates_each_power_by_role() {
    use AccountRole::{Admin, Builder, Moderator, Player, Tester};
    let who = || Who::Guid(7);
    let all = || {
        vec![
            spectate(Moderator, who()),
            unspectate(Moderator),
            freeze(Moderator, who(), true),
            freeze(Moderator, who(), false),
            goto(Moderator, who()),
        ]
    };
    assert!(all().iter().all(Result::is_ok));
    for role in [Moderator, Tester, Builder, Player] {
        for refused in [
            invisible(role, true),
            invisible(role, false),
            follow(role, who(), 3.0),
            bring(role, who()),
            send(role, who(), [1.0, 2.0, 3.0]),
            fly(role, true),
            waypoint(role, WaypointOp::List),
        ] {
            assert_eq!(refused, Err(NOT_ALLOWED), "{role:?}");
        }
    }
    for role in [Tester, Builder, Player] {
        assert_eq!(spectate(role, who()), Err(NOT_ALLOWED), "{role:?}");
        assert_eq!(freeze(role, who(), true), Err(NOT_ALLOWED), "{role:?}");
    }
    assert!(goto(Tester, who()).is_ok() && goto(Builder, who()).is_ok());
    assert_eq!(goto(Player, who()), Err(NOT_ALLOWED));
    for power in [
        spectate(Admin, who()),
        invisible(Admin, true),
        follow(Admin, who(), 3.0),
        bring(Admin, who()),
        send(Admin, who(), [1.0, 2.0, 3.0]),
        fly(Admin, false),
        waypoint(Admin, WaypointOp::Save("home".into())),
    ] {
        assert!(power.is_ok(), "{power:?}");
    }
    assert!(follow(Admin, who(), 31.0).is_err());
    assert!(send(Admin, who(), [f32::NAN, 0.0, 0.0]).is_err());
    assert!(waypoint(Admin, WaypointOp::Go(String::new())).is_err());
}

#[test]
fn g5_watchers_stand_behind_the_facing() {
    assert_eq!(
        behind([10.0, 5.0, 0.0], [1.0, 0.0, 0.0], 2.0),
        [8.0, 5.0, 0.0]
    );
    assert_eq!(
        behind([0.0, 0.0, 0.0], [0.0, 0.0, -2.0], 3.0),
        [0.0, 0.0, 3.0]
    );
    // No facing: on the spot, never a NaN.
    assert_eq!(
        behind([1.0, 2.0, 3.0], [0.0, 0.0, 0.0], 3.0),
        [1.0, 2.0, 3.0]
    );
    let lock = behind([1.0, 2.0, 3.0], [0.6, 0.0, 0.8], SPECTATE_BEHIND_M);
    let gap = ((lock[0] - 1.0).powi(2) + (lock[2] - 3.0).powi(2)).sqrt();
    assert!((gap - SPECTATE_BEHIND_M).abs() < 1e-5 && gap < 1.0);
}

fn report(position: [f32; 3]) -> MovementUpdate {
    MovementUpdate {
        position,
        direction: [1.0, 0.0, 0.0],
        state: 0,
        unknown: 0,
    }
}

/// The one staff check in `movement::validate`: a driven or flying character is never snapped
/// back or rejected, a held one is clamped to its hold, and the token stays.
#[test]
fn g5_validation_exempts_staff_motion_and_keeps_the_token() {
    let start = MovementState::new(0, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
    let jump = report([500.0, 0.0, 0.0]);
    let step = Duration::from_millis(100);
    let policy = SpeedModifiers::riding(None);
    let plain = validate(&start, jump, 0, step, policy, None);
    assert_eq!(plain.decision, Decision::RejectAndSnapBack, "the control");

    let token = |motion| Some(ExpectedDiscontinuity::staff(motion, 0, [0.0, 0.0, 0.0]));
    let driven = validate(&start, jump, 0, step, policy, token(StaffMotion::Driven));
    assert_eq!(
        (driven.decision, driven.reason, driven.state),
        (Decision::Ignore, Reason::StaffDriven, start)
    );
    let kept = |result: &crate::movement::ValidationResult| {
        let token = result.expected_discontinuity.as_ref();
        token.and_then(|t| t.staff_motion())
    };
    assert_eq!(kept(&driven), Some(StaffMotion::Driven));

    let flying = validate(&start, jump, 0, step, policy, token(StaffMotion::Flight));
    assert_eq!(
        (flying.decision, flying.reason),
        (Decision::Accept, Reason::StaffFlight)
    );
    assert_eq!(flying.state.position, [500.0, 0.0, 0.0]);
    assert_eq!(kept(&flying), Some(StaffMotion::Flight));

    let held = validate(&start, jump, 0, step, policy, token(StaffMotion::Held));
    assert_eq!(
        (held.decision, held.reason),
        (Decision::Clamp, Reason::StaffHeld)
    );
    assert_eq!(
        held.state.position,
        [0.0, 0.0, 0.0],
        "clamped back to the hold"
    );
    assert_eq!(kept(&held), Some(StaffMotion::Held));
    let still = validate(
        &start,
        report([0.2, 0.0, 0.1]),
        0,
        step,
        policy,
        token(StaffMotion::Held),
    );
    assert_eq!(
        (still.decision, still.state.position),
        (Decision::Ignore, [0.0, 0.0, 0.0])
    );

    // Garbage is still refused, and a token for another zone is no exemption.
    let bad = validate(
        &start,
        report([f32::NAN, 0.0, 0.0]),
        0,
        step,
        policy,
        token(StaffMotion::Flight),
    );
    assert_eq!(bad.decision, Decision::RejectAndSnapBack);
    let elsewhere = Some(ExpectedDiscontinuity::staff(
        StaffMotion::Flight,
        9,
        [0.0; 3],
    ));
    let other_zone = validate(&start, jump, 0, step, policy, elsewhere);
    assert_eq!(other_zone.decision, Decision::RejectAndSnapBack);
}

/// The registry side of invisible: a hidden character leaves every other view, moves unseen,
/// still sees others, stays hidden across a leave and join, and comes back when shown.
#[test]
fn g5_a_hidden_character_is_in_no_other_view() {
    let zone = ZoneInstance {
        zone_id: 0,
        instance_id: 1,
    };
    let mut players = PlayerRegistry::new(InterestPolicy::new(100.0, 64).unwrap());
    let (gm, viewer) = (SessionId(1), SessionId(2));
    players.join(gm, 11, 101, zone, [0.0; 3], 0.0).unwrap();
    players
        .join(viewer, 12, 102, zone, [5.0, 0.0, 0.0], 0.0)
        .unwrap();
    assert!(players.knows_character(viewer, 101));

    let gone = players.set_hidden(101, true);
    assert!(
        gone.iter()
            .any(|m| m.recipient == viewer && m.kind == MessageKind::Despawn),
        "{gone:?}"
    );
    assert!(gone.iter().all(|m| m.recipient == viewer));
    assert!(!players.knows_character(viewer, 101) && players.is_hidden(101));
    let moved = players.update_pose(gm, zone, [6.0, 0.0, 1.0], 0.0).unwrap();
    assert!(moved.iter().all(|m| m.recipient == gm), "{moved:?}");
    assert!(
        players.knows_character(gm, 102),
        "the GM still sees the viewer"
    );
    let near = players.nearest_session(zone, [6.0, 0.0, 1.0], 2.0, |_| true);
    assert_eq!(
        near.map(|p| p.character_guid),
        Some(102),
        "the GM is never picked"
    );

    players.leave(gm);
    let rejoin = players
        .join(gm, 11, 101, zone, [5.0, 0.0, 1.0], 0.0)
        .unwrap();
    assert!(rejoin.iter().all(|m| m.recipient == gm), "{rejoin:?}");
    assert!(!players.knows_character(viewer, 101));

    let back = players.set_hidden(101, false);
    assert!(
        back.iter()
            .any(|m| m.recipient == viewer && m.kind == MessageKind::Spawn),
        "{back:?}"
    );
    assert!(players.knows_character(viewer, 101));
    players.set_hidden(101, true);
    players.forget_hidden(101);
    assert!(!players.is_hidden(101));
}

#[test]
fn g5_waypoints_persist_across_a_reload() {
    let dir = std::env::temp_dir().join(format!("osfr-g5-waypoints-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("gm-waypoints.json");
    let _ = std::fs::remove_file(&path);
    assert_eq!(Waypoints::load(&path).unwrap(), Waypoints::default());

    let spot = |label: &str, x: f32| Waypoint {
        label: label.into(),
        zone_id: 0,
        position: [x, 1.5, -2.0],
    };
    let mut book = Waypoints::default();
    book.put(5, spot("Snow Peak", 10.0)).unwrap();
    book.put(5, spot("dock", 20.0)).unwrap();
    book.put(6, spot("home", 30.0)).unwrap();
    book.put(5, spot("snow peak", 11.0)).unwrap();
    book.save(&path).unwrap();

    let reloaded = Waypoints::load(&path).unwrap();
    assert_eq!(reloaded, book);
    assert_eq!(reloaded.list(5).len(), 2, "a same label replaces");
    assert_eq!(reloaded.find(5, "SNOW PEAK").unwrap().position[0], 11.0);
    assert!(
        reloaded.find(6, "dock").is_none(),
        "each staff member has their own"
    );
    let mut edited = reloaded;
    assert!(edited.remove(5, "Dock") && !edited.remove(5, "dock"));
    assert!(edited.put(5, spot("", 1.0)).is_err());

    std::fs::write(
        &path,
        br#"{"staff":{"5":[{"label":"x","zone_id":0,"position":[1e30,0,0]}]}}"#,
    )
    .unwrap();
    assert!(
        Waypoints::load(&path).is_err(),
        "an out-of-bounds spot is refused"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn g5_audit_keeps_the_newest_rows() {
    let mut powers = GmPowers::default();
    for n in 0..(AUDIT_ROWS as u64 + 3) {
        powers.record(AuditRow {
            timestamp_ms: n,
            account_id: 1,
            character_guid: 2,
            action: "freeze",
            target_guid: Some(n),
            applied: true,
        });
    }
    let rows: Vec<_> = powers.audit().collect();
    assert_eq!(rows.len(), AUDIT_ROWS);
    assert_eq!(rows[0].timestamp_ms, 3);
}
