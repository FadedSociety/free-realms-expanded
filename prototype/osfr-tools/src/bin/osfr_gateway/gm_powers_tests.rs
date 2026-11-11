//! G5: the staff powers through the production path, in account-store mode, on the L043 rig: a
//! real `Server`, sessions bound by login tickets, `realm` chat through
//! `Session::dispatch_message`, opcode-125 reports for movement and `Session::tick` for the
//! server tick. Synthetic characters on the bundled opening reference.
use super::*;
use osfr_tools::gm_powers::{SPECTATE_BEHIND_M, behind};

/// A client position report (opcode 125) from `first`'s own character.
fn walk(rig: &mut Rig, first: &str, position: [f32; 3], now: u64) -> Vec<Vec<u8>> {
    let guid = rig.guid(first);
    let rotation = rig.session(guid).gateway.state().rotation;
    let mut pose = vec![125, 0];
    pose.extend(guid.to_le_bytes());
    for value in position.into_iter().chain(rotation) {
        pose.extend(value.to_le_bytes());
    }
    pose.extend([0, 0]);
    rig.dispatch(guid, &wrap_application(&pose), now).unwrap();
    rig.drain(guid)
}

/// One server tick for `first`'s session, as `Server::step` runs it.
fn tick(rig: &mut Rig, first: &str, now: u64) {
    let key = rig.keys[&rig.guid(first)];
    let server = &mut rig.server;
    let mut session = server.sessions.remove(&key).unwrap();
    let ticked = session.tick(
        &server.options,
        now,
        &mut server.world,
        &mut server.membership,
        &mut server.sessions,
    );
    server.sessions.insert(key, session);
    ticked.unwrap();
}

fn pose(rig: &Rig, first: &str) -> CharacterState {
    rig.session(rig.guid(first)).gateway.state().clone()
}

/// Whether `viewer` has `subject` in its world (its directed AddPc).
fn sees(rig: &Rig, viewer: &str, subject: &str) -> bool {
    let id = rig.session(rig.guid(viewer)).id;
    rig.server
        .world
        .players
        .knows_character(id, rig.guid(subject))
}

fn system_lines(replies: &[Vec<u8>]) -> Vec<String> {
    replies
        .iter()
        .filter(|message| message.get(7..11) == Some(&[15, 0, 3, 0]))
        .map(|message| {
            let length = u32::from_le_bytes(message[11..15].try_into().unwrap()) as usize;
            String::from_utf8(message[15..15 + length].to_vec()).unwrap()
        })
        .collect()
}

fn location(replies: &[Vec<u8>]) -> Vec<[f32; 3]> {
    replies
        .iter()
        .filter(|message| message.get(7..11) == Some(&[38, 0, 12, 0]))
        .map(|message| std::array::from_fn(|i| osfr_tools::le::f32(message, 11 + 4 * i).unwrap()))
        .collect()
}

fn removes(replies: &[Vec<u8>], guid: u64) -> bool {
    replies.iter().any(|message| {
        message.get(7..11) == Some(&[35, 0, 3, 0])
            && message.get(13..21) == Some(&guid.to_le_bytes()[..])
    })
}

fn gap(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f32>()
        .sqrt()
}

fn three(tag: &str, roles: [AccountRole; 3]) -> Rig {
    let mut rig = Rig::new(
        tag,
        &[
            ("Alpha", roles[0]),
            ("Bravo", roles[1]),
            ("Charlie", roles[2]),
        ],
        true,
    );
    for first in ["Alpha", "Bravo", "Charlie"] {
        rig.login(first, true);
    }
    for first in ["Alpha", "Bravo", "Charlie"] {
        rig.drain(rig.guid(first));
    }
    rig
}

/// The owner's "see another player's view": the spectating GM leaves every other player's
/// world, is kept within a metre of a moving target every tick (its own reports ignored, never
/// rejected), and `unspectate` puts it back where it was and shows it again. Audited.
#[test]
fn g5_spectate_hides_the_gm_tracks_the_target_and_returns() {
    if !test_support::bin_test_target("g5") {
        return;
    }
    use AccountRole::{Admin, Player};
    let mut rig = three("g5-spectate", [Admin, Player, Player]);
    assert!(
        sees(&rig, "Charlie", "Alpha") && sees(&rig, "Bravo", "Alpha"),
        "fixture"
    );
    let start = pose(&rig, "Alpha");
    let alpha = rig.guid("Alpha");

    let replies = rig.realm("Alpha", "spectate Bravo Test");
    assert!(
        replies
            .iter()
            .any(|l| l.starts_with("Spectating Bravo Test")),
        "{replies:?}"
    );
    assert!(!sees(&rig, "Charlie", "Alpha") && !sees(&rig, "Bravo", "Alpha"));
    assert!(sees(&rig, "Alpha", "Bravo"), "the GM still sees its target");
    let charlie = rig.guid("Charlie");
    assert!(
        removes(&rig.drain(charlie), alpha),
        "Charlie got no 35/3 for the GM"
    );
    let bravo = pose(&rig, "Bravo");
    let lock = behind(bravo.position, bravo.rotation, SPECTATE_BEHIND_M);
    assert_eq!(
        pose(&rig, "Alpha").position,
        lock,
        "the first lock is at once"
    );

    let mut now = 2_000;
    let mut at = bravo.position;
    for _ in 0..6 {
        at[0] += 3.0;
        at[2] -= 1.0;
        now += 100;
        walk(&mut rig, "Bravo", at, now);
        tick(&mut rig, "Alpha", now);
        let gm = pose(&rig, "Alpha");
        let target = pose(&rig, "Bravo").position;
        assert_eq!(target, at, "Bravo's walk was refused");
        assert!(
            gap(gm.position, target) <= 1.0,
            "{:?} vs {target:?}",
            gm.position
        );
        let sent = location(&rig.drain(alpha));
        assert_eq!(sent.last(), Some(&gm.position), "no 38/12 to the lock pose");
        assert!(!sees(&rig, "Charlie", "Alpha") && !sees(&rig, "Bravo", "Alpha"));
    }
    // The GM's own client wanders off: ignored, no correction, never rejected.
    let held = pose(&rig, "Alpha").position;
    let wander = [held[0] + 250.0, held[1], held[2]];
    now += 100;
    let answer = walk(&mut rig, "Alpha", wander, now);
    assert!(
        location(&answer).is_empty(),
        "the GM was snapped back: {answer:?}"
    );
    assert_eq!(pose(&rig, "Alpha").position, held);

    let replies = rig.realm("Alpha", "unspectate");
    assert!(
        replies.iter().any(|l| l.contains("Back where you started")),
        "{replies:?}"
    );
    let back = pose(&rig, "Alpha");
    assert_eq!(
        (back.zone_id, back.position),
        (start.zone_id, start.position)
    );
    assert!(sees(&rig, "Charlie", "Alpha") && sees(&rig, "Bravo", "Alpha"));
    // Normal movement checks resume from the way back: arrive, then a 250 m jump in 100 ms is
    // snapped back as for anyone.
    now += 100;
    let arrived = [
        start.position[0] + 0.5,
        start.position[1],
        start.position[2],
    ];
    walk(&mut rig, "Alpha", arrived, now);
    assert_eq!(pose(&rig, "Alpha").position, arrived);
    now += 100;
    let answer = walk(&mut rig, "Alpha", wander, now);
    assert_eq!(
        location(&answer),
        [arrived],
        "a 250 m jump is snapped back again"
    );

    let rows = rig.audit_rows("gm_power");
    let actions: Vec<_> = rows.iter().map(|row| row["action"].clone()).collect();
    assert_eq!(actions, ["spectate", "unspectate"], "{rows:?}");
    assert_eq!(rows[0]["target_guid"], rig.guid("Bravo"));
    assert_eq!(rows[0]["outcome"], "applied");
    assert_eq!(rig.server.world.gm.audit().count(), 2);
}

/// A frozen player's moves are refused (held where it stood, with the correction), it is told
/// and its client gets 35/32; a moderator may freeze a player but not an admin; release lets it
/// walk again.
#[test]
fn g5_freeze_holds_the_player_and_tells_it() {
    if !test_support::bin_test_target("g5") {
        return;
    }
    use AccountRole::{Admin, Moderator, Player};
    let mut rig = three("g5-freeze", [Admin, Player, Moderator]);
    let bravo = rig.guid("Bravo");
    let held = pose(&rig, "Bravo").position;

    let refused = rig.realm("Charlie", "freeze Alpha Test");
    assert!(
        refused.iter().any(|l| l.contains("lower role")),
        "{refused:?}"
    );
    let replies = rig.realm("Charlie", "freeze Bravo Test");
    assert!(
        replies.iter().any(|l| l.starts_with("Froze Bravo Test")),
        "{replies:?}"
    );
    let told = rig.drain(bravo);
    assert!(
        system_lines(&told).iter().any(|l| l.contains("frozen you")),
        "{:?}",
        system_lines(&told)
    );
    assert!(
        told.contains(&wrap_application(&[35, 0, 32, 0, 1])),
        "no 35/32 freeze"
    );

    let step = [held[0] + 4.0, held[1], held[2]];
    let answer = walk(&mut rig, "Bravo", step, 2_100);
    assert_eq!(
        pose(&rig, "Bravo").position,
        held,
        "the frozen move was applied"
    );
    assert_eq!(location(&answer), [held], "no correction back to the hold");
    tick(&mut rig, "Bravo", 2_150);
    let answer = walk(&mut rig, "Bravo", step, 2_200);
    assert_eq!(pose(&rig, "Bravo").position, held, "the hold lasts");
    assert_eq!(location(&answer), [held]);

    let replies = rig.realm("Charlie", "unfreeze Bravo Test");
    assert!(
        replies.iter().any(|l| l.starts_with("Released")),
        "{replies:?}"
    );
    let told = rig.drain(bravo);
    assert!(
        told.contains(&wrap_application(&[35, 0, 32, 0, 0])),
        "no 35/32 release"
    );
    assert!(system_lines(&told).iter().any(|l| l.contains("move again")));
    walk(&mut rig, "Bravo", step, 2_300);
    assert_eq!(
        pose(&rig, "Bravo").position,
        step,
        "the release did not let it walk"
    );
    let actions: Vec<_> = rig
        .audit_rows("gm_power")
        .iter()
        .map(|row| (row["action"].clone(), row["outcome"].clone()))
        .collect();
    assert_eq!(
        actions,
        [
            ("freeze".into(), "refused".into()),
            ("freeze".into(), "applied".into()),
            ("unfreeze".into(), "applied".into()),
        ]
    );
}

/// `realm goto <name>` lands the GM on the player, `realm bring <name>` lands the player on
/// the GM, and `realm goto <NPC>` is still the developer teleport.
#[test]
fn g5_bring_and_goto_by_name_land_on_the_spot() {
    if !test_support::bin_test_target("g5") {
        return;
    }
    use AccountRole::{Admin, Player};
    let mut rig = three("g5-move", [Admin, Player, Player]);
    let start = pose(&rig, "Bravo").position;
    let away = [start[0] + 6.0, start[1], start[2] + 3.0];
    walk(&mut rig, "Bravo", away, 2_000);
    assert_eq!(pose(&rig, "Bravo").position, away, "fixture walk");

    let replies = rig.realm("Alpha", "goto bravo test");
    assert!(
        replies.iter().any(|l| l.starts_with("Moved you to")),
        "{replies:?}"
    );
    assert_eq!(pose(&rig, "Alpha").position, away);

    let elsewhere = [away[0] - 5.0, away[1], away[2] - 4.0];
    walk(&mut rig, "Alpha", elsewhere, 2_200);
    assert_eq!(pose(&rig, "Alpha").position, elsewhere, "fixture walk");
    let replies = rig.realm("Alpha", "bring Bravo Test");
    assert!(
        replies.iter().any(|l| l.starts_with("Moved character")),
        "{replies:?}"
    );
    assert_eq!(pose(&rig, "Bravo").position, elsewhere);
    let sent = location(&rig.drain(rig.guid("Bravo")));
    assert_eq!(
        sent.last(),
        Some(&elsewhere),
        "Bravo's client was not moved"
    );

    // A name that is no online player stays with gameplay (an NPC or "no target" reply).
    let replies = rig.realm("Alpha", "goto Nobody Here");
    assert!(
        !replies
            .iter()
            .any(|l| l.contains("no single online player")),
        "{replies:?}"
    );
    let actions: Vec<_> = rig
        .audit_rows("gm_power")
        .iter()
        .map(|row| row["action"].clone())
        .collect();
    assert_eq!(actions, ["goto", "bring"]);
    assert_eq!(
        rig.audit_rows("admin_move").len(),
        1,
        "bring is also an admin move"
    );
}

/// Waypoints are a file beside the saves: saved, listed and gone to across a logout and login,
/// and deleted.
#[test]
fn g5_waypoints_persist_across_a_relog() {
    if !test_support::bin_test_target("g5") {
        return;
    }
    use AccountRole::{Admin, Player};
    let mut rig = three("g5-waypoints", [Admin, Player, Player]);
    let camp = pose(&rig, "Alpha").position;
    let replies = rig.realm("Alpha", "waypoint save Base Camp");
    assert!(
        replies
            .iter()
            .any(|l| l.starts_with("Waypoint Base Camp saved")),
        "{replies:?}"
    );
    let file = rig
        .server
        .options
        .state
        .parent()
        .unwrap()
        .join("gm-waypoints.json");
    assert!(file.is_file(), "no waypoint file");

    rig.logout("Alpha");
    rig.login("Alpha", true);
    let listed = rig.realm("Alpha", "waypoint list");
    assert!(
        listed.iter().any(|l| l.contains("Base Camp (zone 0")),
        "{listed:?}"
    );
    let away = [camp[0] + 7.0, camp[1], camp[2] - 2.0];
    walk(&mut rig, "Alpha", away, 2_000);
    assert_eq!(pose(&rig, "Alpha").position, away, "fixture walk");
    let replies = rig.realm("Alpha", "wp go base camp");
    assert!(
        replies.iter().any(|l| l.contains("waypoint base camp")),
        "{replies:?}"
    );
    assert_eq!(pose(&rig, "Alpha").position, camp);

    let reloaded = osfr_tools::gm_powers::waypoints::Waypoints::load(&file).unwrap();
    let account = rig.session(rig.guid("Alpha")).account_guid;
    assert_eq!(reloaded.find(account, "base camp").unwrap().position, camp);
    let replies = rig.realm("Alpha", "waypoint del Base Camp");
    assert!(replies.iter().any(|l| l.contains("deleted")), "{replies:?}");
    let listed = rig.realm("Alpha", "waypoint list");
    assert!(
        listed.iter().any(|l| l.starts_with("No waypoints yet")),
        "{listed:?}"
    );
}

/// A player (and a moderator, for the admin powers) gets the ordinary role refusal for every
/// power and nothing changes: nobody hidden, frozen, watched or moved, no power audit row.
#[test]
fn g5_non_staff_cannot_run_any_power() {
    if !test_support::bin_test_target("g5") {
        return;
    }
    use AccountRole::{Admin, Moderator, Player};
    let people = ["Alpha", "Bravo", "Charlie", "Delta"];
    let mut rig = Rig::new(
        "g5-roles",
        &[
            ("Alpha", Admin),
            ("Bravo", Player),
            ("Charlie", Moderator),
            ("Delta", Player),
        ],
        true,
    );
    for first in people {
        rig.login(first, true);
    }
    let before: Vec<_> = people.map(|first| pose(&rig, first)).to_vec();
    // Two players share the words: the account action budget allows a burst of 10 chats.
    let words = [
        ("Bravo", "spectate Alpha Test"),
        ("Bravo", "unspectate"),
        ("Bravo", "invisible"),
        ("Bravo", "visible"),
        ("Bravo", "freeze Alpha Test"),
        ("Bravo", "unfreeze Alpha Test"),
        ("Bravo", "goto Alpha Test"),
        ("Delta", "follow Alpha Test"),
        ("Delta", "unfollow"),
        ("Delta", "bring Alpha Test"),
        ("Delta", "send Alpha Test 1 2 3"),
        ("Delta", "fly"),
        ("Delta", "waypoint save here"),
        ("Delta", "waypoint list"),
    ];
    for (first, text) in words {
        let replies = rig.realm(first, text);
        assert!(
            replies.iter().any(|l| l.contains("does not allow")),
            "{first} {text}: {replies:?}"
        );
    }
    for text in [
        "invisible",
        "follow Bravo Test",
        "bring Bravo Test",
        "waypoint list",
        "fly",
    ] {
        let replies = rig.realm("Charlie", text);
        assert!(
            replies.iter().any(|l| l.contains("does not allow")),
            "Charlie {text}: {replies:?}"
        );
    }
    let after: Vec<_> = people.map(|first| pose(&rig, first)).to_vec();
    assert_eq!(after, before, "a refused power moved someone");
    let world = &rig.server.world;
    assert!(world.gm.watching.is_empty() && world.gm.frozen.is_empty());
    assert!(world.gm.flying.is_empty() && world.gm.audit().count() == 0);
    for first in people {
        assert!(!world.players.is_hidden(rig.guid(first)));
    }
    assert!(sees(&rig, "Bravo", "Alpha") && sees(&rig, "Charlie", "Alpha"));
    assert!(rig.audit_rows("gm_power").is_empty());
}

/// A following GM stays visible, a set distance behind its target, and its own wandering
/// reports are never snapped back or rejected; `fly` lets a GM's long jump stand.
#[test]
fn g5_following_or_flying_gm_is_never_flagged() {
    if !test_support::bin_test_target("g5") {
        return;
    }
    use AccountRole::{Admin, Player};
    let mut rig = three("g5-follow", [Admin, Player, Player]);
    let alpha = rig.guid("Alpha");
    let replies = rig.realm("Alpha", "follow Bravo Test 4");
    assert!(
        replies
            .iter()
            .any(|l| l.starts_with("Following Bravo Test")),
        "{replies:?}"
    );
    assert!(sees(&rig, "Charlie", "Alpha"), "a follower stays visible");

    let mut at = pose(&rig, "Bravo").position;
    let mut now = 2_000;
    for _ in 0..4 {
        at[2] += 3.0;
        now += 100;
        walk(&mut rig, "Bravo", at, now);
        tick(&mut rig, "Alpha", now);
        let bravo = pose(&rig, "Bravo");
        let expected = behind(bravo.position, bravo.rotation, 4.0);
        assert_eq!(pose(&rig, "Alpha").position, expected);
        assert_eq!(location(&rig.drain(alpha)), [expected], "the lock's 38/12");
        now += 10;
        let wander = [expected[0] + 300.0, expected[1], expected[2]];
        let answer = walk(&mut rig, "Alpha", wander, now);
        assert!(location(&answer).is_empty(), "a follower was snapped back");
        assert_eq!(
            pose(&rig, "Alpha").position,
            expected,
            "a wander was applied"
        );
        assert!(sees(&rig, "Charlie", "Alpha"));
    }
    rig.drain(alpha);
    let replies = rig.realm("Alpha", "unfollow");
    assert!(
        replies.iter().any(|l| l.contains("stopped following")),
        "{replies:?}"
    );
    assert!(rig.server.world.gm.watching.is_empty());

    let replies = rig.realm("Alpha", "fly");
    assert!(
        replies.iter().any(|l| l.starts_with("Flight on")),
        "{replies:?}"
    );
    let here = pose(&rig, "Alpha").position;
    let far = [here[0] + 300.0, here[1] + 40.0, here[2]];
    now += 100;
    let answer = walk(&mut rig, "Alpha", far, now);
    assert!(location(&answer).is_empty(), "a flying GM was snapped back");
    assert_eq!(pose(&rig, "Alpha").position, far);
    let replies = rig.realm("Alpha", "fly off");
    assert!(
        replies.iter().any(|l| l.starts_with("Flight off")),
        "{replies:?}"
    );
    now += 100;
    let answer = walk(&mut rig, "Alpha", here, now);
    assert!(
        !location(&answer).is_empty(),
        "the checks are back after fly off"
    );
}
