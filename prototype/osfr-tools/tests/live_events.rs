//! T418: the scavenger hunt (3000200) is a live event. It stays closed until an
//! admin runs `realm event open 3000200`; `realm event close` stops new offers,
//! while a hunt already accepted still advances, turns in and pays once.
//!
//! Every step drives the production router: `Gameplay::handle_in_world` for the
//! realm chat line and the NPC/quest packets, `Gameplay::tick` for the
//! once-a-second watch. Sessions run with `set_role_commands(true)` and no
//! `--dev-tools`, as an account-store gateway runs them, and share one live
//! folder (the `--world` layer) where the command writes `events.json`. The admin
//! session carries the opening content because realm builder commands route
//! through the opening handler; players use the minimal hunt fixture so
//! `on_location` cannot advance the opening story. Hunt steps come from
//! `tests/support/scavenger_hunt.rs`.
//!
//! No packet is added: the offer is the existing 49/1 (client reader `0x00c7bb60`
//! -> `0x00c7b7a0`), and each command answers with the existing `chat::system`
//! line. "Logged once" for a refused file is a unit test beside the reader
//! (`src/gameplay/live_events.rs`), because the log is the server's stdout.
//!
//! Canary (run by hand, then reverted): without `|| self.event_closed(quest_id)` in
//! `QuestEngine::offer_denial`, `a_closed_hunt_is_not_offered` fails.

#[path = "support/scavenger_hunt.rs"]
mod scavenger_hunt;

use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    chat,
    gameplay::{Gameplay, Outcome},
    quest_engine::QuestStatus,
};
use scavenger_hunt::{
    ADVENTURER, Fixture, HUNT_OPEN, JACQUE, QUEST, accept_hunt, assert_no_offer, character_at_for,
    coins, counts, finish_quest, fresh_in, has_family, has_objective, npc_position, offer_hunt,
    pending_offer, profile_xp, quest_end_reply, quest_reply, quest_status, refused,
    release_end_screen, saved, talk, visit_stop,
};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

const ADMIN: u64 = 418_001;
const PLAYER: u64 = 418_002;
const OTHER: u64 = 418_003;
const LATE: u64 = 418_004;
const ROLE_REFUSAL: &str = "Cannot do that: your account role does not allow this command.";

/// A private live folder shared by every session of one test.
struct Live(PathBuf);

impl Live {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("osfr-live-events-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn events(&self) -> PathBuf {
        self.0.join("events.json")
    }

    /// Windows file times advance in clock ticks (~15.6 ms), so two writes can
    /// share an mtime; wait one tick so the watch sees a distinct stamp.
    fn write(&self, text: &str) {
        std::thread::sleep(Duration::from_millis(100));
        fs::write(self.events(), text).unwrap();
    }
}

impl Drop for Live {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A session on an account-store gateway: role commands on, no --dev-tools.
fn role_session(fixture: &Fixture, role: AccountRole, live: &Path) -> Gameplay {
    let mut game = fixture.game(None);
    game.set_role_commands(true);
    game.set_role(role);
    game.set_world_dir(Some(live.to_path_buf())).unwrap();
    game
}

fn player(fixture: &Fixture, id: u64, live: &Path) -> Gameplay {
    let mut game = fresh_in(fixture, id, live);
    game.set_role_commands(true);
    game
}

/// Log back in from the save: a new session bound the production way.
fn relog(fixture: &Fixture, game: &Gameplay, id: u64, live: &Path) -> Gameplay {
    let mut game = fixture.game_as(id, Some(game.state()));
    game.set_role_commands(true);
    game.set_world_dir(Some(live.to_path_buf())).unwrap();
    game
}

fn at_jacque(id: u64) -> CharacterState {
    character_at_for(id, 0, npc_position(JACQUE))
}

/// One realm chat line through the production router, and its one reply line
/// as the admin reads it (`chat::system` escapes `<`, `>` and `&` on the wire).
fn realm(game: &mut Gameplay, id: u64, text: &str, now: u64) -> (Outcome, String) {
    let out = game
        .handle_in_world(&chat::local_request(text), &at_jacque(id), now)
        .unwrap();
    let lines: Vec<String> = out
        .replies
        .iter()
        .filter(|packet| packet.starts_with(&[15, 0, 3, 0]))
        .map(|packet| {
            let length = u32::from_le_bytes(packet[4..8].try_into().unwrap()) as usize;
            String::from_utf8(packet[8..8 + length].to_vec())
                .unwrap()
                .replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&amp;", "&")
        })
        .collect();
    assert_eq!(
        lines.len(),
        1,
        "{text} answers with one chat line: {lines:?}"
    );
    (out, lines[0].clone())
}

/// The hunt is open for `game`: Jacque offers it; the decline puts it back.
fn assert_offered(game: &mut Gameplay, at: &CharacterState, now: &mut u64) {
    offer_hunt(game, at, now);
    let declined = game
        .handle_in_world(&quest_reply(QUEST, false), at, *now)
        .unwrap();
    *now += 2;
    assert!(declined.changed && pending_offer(game).is_none());
}

fn has_hunt(game: &Gameplay) -> bool {
    game.state().npc_quests.unwrap().quests.contains_key(&QUEST)
}

#[test]
fn a_closed_hunt_is_not_offered() {
    let live = Live::new("closed");
    let fixture = Fixture::minimal(PLAYER);
    let mut game = player(&fixture, PLAYER, &live.0);
    let at = at_jacque(PLAYER);
    let mut now = 10_000;
    assert!(!live.events().exists());
    assert_no_offer(&mut game, &at, JACQUE, &mut now, "the closed hunt");
    let ticked = game.tick(&at, now).unwrap();
    assert!(!ticked.changed, "the watch is world state, never a save");
    assert_no_offer(
        &mut game,
        &at,
        JACQUE,
        &mut now,
        "the closed hunt after a tick",
    );
    assert!(!has_hunt(&game));
}

#[test]
fn an_admin_open_reaches_another_session_within_one_poll() {
    let live = Live::new("open");
    let admin_fixture = Fixture::for_character(ADMIN);
    let mut admin = role_session(&admin_fixture, AccountRole::Admin, &live.0);
    let fixture = Fixture::minimal(PLAYER);
    let mut game = player(&fixture, PLAYER, &live.0);
    let at = at_jacque(PLAYER);
    let mut now = 10_000;
    game.tick(&at, now).unwrap(); // the next poll is due at 11_000
    assert_no_offer(&mut game, &at, JACQUE, &mut now, "before the open");

    let (out, text) = realm(&mut admin, ADMIN, "realm event open 3000200", now);
    assert_eq!(text, "Event 3000200 is open.");
    assert!(
        !out.changed,
        "an event is world state, not the admin's save"
    );
    assert_eq!(
        fs::read_to_string(live.events())
            .unwrap()
            .split_whitespace()
            .collect::<String>(),
        HUNT_OPEN
    );
    let (_, text) = realm(&mut admin, ADMIN, "realm event list", now);
    assert_eq!(
        text, "Open events: 3000200. Closed events: none.",
        "the acting session applies its own write at once"
    );
    let (_, text) = realm(&mut admin, ADMIN, "realm event open 3000200", now);
    assert_eq!(text, "Event 3000200 is already open.");

    let early = game.tick(&at, now).unwrap();
    assert!(!early.changed);
    assert_no_offer(&mut game, &at, JACQUE, &mut now, "inside the poll window");
    now = 11_000;
    let polled = game.tick(&at, now).unwrap();
    assert!(!polled.changed, "the open is not a character change");
    offer_hunt(&mut game, &at, &mut now);
    accept_hunt(&mut game, &at, &mut now);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
}

#[test]
fn close_refuses_a_shown_offer_while_an_accepted_hunt_pays_once() {
    let live = Live::new("close");
    let admin_fixture = Fixture::for_character(ADMIN);
    let mut admin = role_session(&admin_fixture, AccountRole::Admin, &live.0);
    let mut now = 20_000;
    realm(&mut admin, ADMIN, "realm event open 3000200", now);

    let fixture = Fixture::minimal(PLAYER);
    let (shown_at, hunter_at) = (at_jacque(PLAYER), at_jacque(OTHER));
    let mut shown = player(&fixture, PLAYER, &live.0);
    let mut hunter = player(&fixture, OTHER, &live.0);
    offer_hunt(&mut hunter, &hunter_at, &mut now);
    accept_hunt(&mut hunter, &hunter_at, &mut now);
    visit_stop(&mut hunter, OTHER, 0, &mut now);
    offer_hunt(&mut shown, &shown_at, &mut now);

    std::thread::sleep(Duration::from_millis(100)); // a distinct file time
    let (_, text) = realm(&mut admin, ADMIN, "realm event close 3000200", now);
    assert_eq!(text, "Event 3000200 is closed.");
    now = 30_000;
    shown.tick(&shown_at, now).unwrap();
    hunter.tick(&hunter_at, now).unwrap();

    // The offer shown before the close is not accepted after it. The client
    // forgot its offer when it sent the accept (0x00a97d70 sets the offer id to
    // -1), so the refusal withdraws the offer, as a decline does, and saves
    // that: a kept pending offer would block every later offer.
    let before = saved(&shown);
    let withdrawn = shown
        .handle_in_world(&quest_reply(QUEST, true), &shown_at, now)
        .unwrap();
    now += 2;
    assert!(withdrawn.changed, "the withdrawn offer is saved");
    assert_ne!(saved(&shown), before);
    assert!(
        !withdrawn
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 3, None)),
        "an accept after the close added the quest"
    );
    assert_eq!(pending_offer(&shown), None);
    assert!(!has_hunt(&shown), "the withdrawn hunt left the quest list");

    // A character who arrives after the close is not offered the hunt.
    let mut late = player(&fixture, LATE, &live.0);
    assert_no_offer(
        &mut late,
        &at_jacque(LATE),
        JACQUE,
        &mut now,
        "after the close",
    );

    // The accepted hunt still advances, turns in and pays exactly once.
    visit_stop(&mut hunter, OTHER, 1, &mut now);
    visit_stop(&mut hunter, OTHER, 2, &mut now);
    let final_talk = talk(&mut hunter, &hunter_at, JACQUE, now);
    now += 2;
    assert!(has_objective(&final_talk.replies, 10, 4));
    release_end_screen(&mut hunter, &hunter_at, QUEST, JACQUE, 0, now);
    now += 2;
    finish_quest(&mut hunter, &hunter_at, QUEST, ADVENTURER, 50, 25, now);
    refused(
        &mut hunter,
        &hunter_at,
        &quest_end_reply(QUEST),
        now + 2,
        "a replayed turn-in",
    );
    assert_eq!(coins(&hunter), 50);
    assert_eq!(profile_xp(&hunter, ADVENTURER), 25);

    // The withdrawn offer blocks nothing: after a relog and a reopen, the same
    // character is offered the hunt again.
    shown = relog(&fixture, &shown, PLAYER, &live.0);
    assert_eq!(pending_offer(&shown), None);
    std::thread::sleep(Duration::from_millis(100)); // a distinct file time
    realm(&mut admin, ADMIN, "realm event open 3000200", now);
    now += 1_000;
    shown.tick(&shown_at, now).unwrap();
    offer_hunt(&mut shown, &shown_at, &mut now);
}

#[test]
fn the_open_state_survives_a_restart_and_a_closed_relog_keeps_the_hunt() {
    let live = Live::new("restart");
    let mut now = 40_000;
    {
        let admin_fixture = Fixture::for_character(ADMIN);
        let mut admin = role_session(&admin_fixture, AccountRole::Admin, &live.0);
        realm(&mut admin, ADMIN, "realm event open 3000200", now);
    } // every session ends: a server restart reads only the live folder

    // A session started after the restart sees the open hunt before any tick.
    let fixture = Fixture::minimal(PLAYER);
    let at = at_jacque(PLAYER);
    let mut game = player(&fixture, PLAYER, &live.0);
    offer_hunt(&mut game, &at, &mut now);
    accept_hunt(&mut game, &at, &mut now);
    visit_stop(&mut game, PLAYER, 0, &mut now);

    std::thread::sleep(Duration::from_millis(100));
    let admin_fixture = Fixture::for_character(ADMIN);
    let mut admin = role_session(&admin_fixture, AccountRole::Admin, &live.0);
    realm(&mut admin, ADMIN, "realm event close 3000200", now);

    // A relog while the event is closed keeps the active hunt, which advances.
    let before = saved(&game);
    game = relog(&fixture, &game, PLAYER, &live.0);
    assert_eq!(saved(&game), before, "the relog changed the hunt");
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(counts(&game, QUEST), [1, 0, 0, 0]);
    visit_stop(&mut game, PLAYER, 1, &mut now);
}

#[test]
fn players_builders_and_bad_ids_cannot_change_the_event() {
    let live = Live::new("refused");
    let mut now = 50_000;
    for (id, role) in [(PLAYER, AccountRole::Player), (OTHER, AccountRole::Builder)] {
        let fixture = Fixture::for_character(id);
        let mut game = role_session(&fixture, role, &live.0);
        for command in [
            "realm event open 3000200",
            "realm event close 3000200",
            "realm event list",
        ] {
            let before = saved(&game);
            let (out, text) = realm(&mut game, id, command, now);
            now += 1;
            assert_eq!(text, ROLE_REFUSAL, "{role:?}: {command}");
            assert!(out.dev_command_denied && !out.changed);
            assert_eq!(saved(&game), before, "{role:?}: {command} changed the save");
            assert!(
                !live.events().exists(),
                "{role:?}: {command} wrote the file"
            );
        }
    }

    // An admin's refusals answer with one line and write nothing either.
    let fixture = Fixture::for_character(ADMIN);
    let mut admin = role_session(&fixture, AccountRole::Admin, &live.0);
    for (command, reply) in [
        (
            "realm event open 3000059",
            "3000059 is not a live event quest; realm event list names them.",
        ),
        (
            "realm event open hunt",
            "hunt is not a live event quest; realm event list names them.",
        ),
        (
            "realm event close 3000200",
            "Event 3000200 is already closed.",
        ),
        (
            "realm event start",
            "Usage: realm event open|close <quest id>, realm event list or realm event start|stop <world event key>.",
        ),
    ] {
        let before = saved(&admin);
        let (out, text) = realm(&mut admin, ADMIN, command, now);
        now += 1;
        assert_eq!(text, reply);
        assert!(!out.changed);
        assert_eq!(saved(&admin), before);
        assert!(!live.events().exists(), "{command} wrote the file");
    }

    // The hunt is still closed for a player.
    let fixture = Fixture::minimal(PLAYER);
    let mut game = player(&fixture, PLAYER, &live.0);
    assert_no_offer(
        &mut game,
        &at_jacque(PLAYER),
        JACQUE,
        &mut now,
        "after refusals",
    );
}

#[test]
fn a_malformed_oversized_or_missing_file_means_closed() {
    let live = Live::new("bad-file");
    live.write(HUNT_OPEN);
    let fixture = Fixture::minimal(PLAYER);
    let at = at_jacque(PLAYER);
    let mut game = player(&fixture, PLAYER, &live.0);
    let admin_fixture = Fixture::for_character(ADMIN);
    let mut admin = role_session(&admin_fixture, AccountRole::Admin, &live.0);
    let mut now = 60_000;
    assert_offered(&mut game, &at, &mut now);

    let oversized = format!("{HUNT_OPEN}{}", " ".repeat(4096));
    for (label, text) in [
        ("a malformed file", r#"{"schema_version":1,"open":["#),
        ("an oversized file", oversized.as_str()),
        ("a wrong schema", r#"{"schema_version":2,"open":[3000200]}"#),
    ] {
        live.write(text);
        now += 1_000;
        assert!(!game.tick(&at, now).unwrap().changed);
        assert_no_offer(&mut game, &at, JACQUE, &mut now, label);
        // The admin cannot write over a file it could not read either.
        let (_, reply) = realm(&mut admin, ADMIN, "realm event close 3000200", now);
        assert!(reply.starts_with("Could not read events.json: "), "{reply}");
        assert!(reply.ends_with(". Nothing changed."), "{reply}");
        assert_eq!(fs::read_to_string(live.events()).unwrap(), text, "{label}");
    }

    live.write(HUNT_OPEN);
    now += 1_000;
    game.tick(&at, now).unwrap();
    assert_offered(&mut game, &at, &mut now);

    fs::remove_file(live.events()).unwrap();
    now += 1_000;
    game.tick(&at, now).unwrap();
    assert_no_offer(&mut game, &at, JACQUE, &mut now, "a missing file");
    assert!(!has_hunt(&game));
}
