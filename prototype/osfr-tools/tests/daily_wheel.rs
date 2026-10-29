//! T560: the daily wheel, part 1. World entry offers today's spin (143/1, reader `00adafa0`),
//! 26/11 (`009bdbd0`) or the Games window's Play opens a type-22 run, and the client's own
//! start (39/5) is answered with the `game_wheel.gfx` Flash launch (26/12). Driven through the
//! real `Gameplay` dispatch with raw inner packets, like `chess_route.rs`; the content is the
//! shipped `data/content` (a scratch directory without an NPC package binds it).
use osfr_tools::{
    character_store::CharacterState,
    daily_wheel::{Content, START_WHEEL},
    gameplay::{Gameplay, GameplayState, Outcome},
};
use serde_json::json;

#[path = "common/field_battle_fixture.rs"]
mod field_battle_fixture;
#[path = "support/fishing_packets.rs"]
mod fishing_packets;
#[path = "support/json.rs"]
mod json;
#[path = "support/scratch.rs"]
mod scratch;
use field_battle_fixture::portal_play as play;
use fishing_packets::lifecycle;
use scratch::Scratch;

const LAKESHORE: [f32; 3] = [-733.952, 9.09, 945.973];
const DAY: u64 = 86_400;
/// A pinned server clock: UTC day 20,000, one hour in.
const TODAY: u64 = 20_000;
const CLOCK: u64 = TODAY * DAY + 3_600;
const WHEEL_GAME: i32 = 8;
/// Single-player chess row (`chess_route.rs`): a real occupation with no extra content.
const CHESS_ROW: u32 = 20;
const FIRST_INSTANCE: i32 = 62_000_001;

fn character() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: LAKESHORE,
        rotation: [0.0, 0.0, 1.0],
    }
}

/// A player who has just entered the world on the pinned clock.
fn session(last_spin_day: Option<u64>) -> (Gameplay, Outcome, Scratch) {
    let scratch = Scratch::new("daily-wheel");
    let mut state = Gameplay::load(scratch.path(), None).unwrap().state();
    state.last_spin_day = last_spin_day;
    let mut game = Gameplay::load(scratch.path(), Some(state)).unwrap();
    game.seed_wheel_clock(CLOCK);
    let entry = game.initialize_world(&character(), 500).unwrap();
    (game, entry, scratch)
}

fn send(game: &mut Gameplay, inner: &[u8]) -> Outcome {
    game.handle_in_world(inner, &character(), 1_000).unwrap()
}

fn int(value: i32) -> [u8; 4] {
    value.to_le_bytes()
}

/// The 143/1 the client reads (`00adafa0`): u16 opcode, u8 sub, five i32, two strings, one i32.
fn spins(count: i32, seconds: i32) -> Vec<u8> {
    let mut bytes = vec![143, 0, 1];
    for value in [8, count, 0, seconds, seconds] {
        bytes.extend(int(value));
    }
    for name in ["wheel", "wheel"] {
        bytes.extend(int(name.len() as i32));
        bytes.extend(name.as_bytes());
    }
    bytes.extend(int(0));
    bytes
}

/// The 26/12 Flash launch (`00aa0a10`): two strings and a flag byte.
fn flash_launch() -> Vec<u8> {
    let mut bytes = vec![26, 0, 12, 0];
    for name in ["MiniGameFlash", "game_wheel.gfx"] {
        bytes.extend(int(name.len() as i32));
        bytes.extend(name.as_bytes());
    }
    bytes.push(0);
    bytes
}

fn published<'a>(out: &'a Outcome, prefix: &[u8]) -> Vec<&'a Vec<u8>> {
    out.replies
        .iter()
        .filter(|reply| reply.starts_with(prefix))
        .collect()
}

/// The 39/16 offers in `out`.
fn offers(out: &Outcome) -> Vec<&Vec<u8>> {
    published(out, &[39, 0, 16])
}

fn chat_lines(out: &Outcome) -> usize {
    published(out, &[15, 0, 3, 0]).len()
}

/// Open the wheel with 26/11 and return its instance id.
fn open(game: &mut Gameplay) -> i32 {
    let out = send(game, &START_WHEEL);
    let offer = offers(&out);
    assert_eq!(offer.len(), 1, "26/11 offers one run");
    i32::from_le_bytes(offer[0][3..7].try_into().unwrap())
}

#[test]
fn world_entry_offers_one_spin_with_the_seconds_to_the_next_utc_midnight() {
    let (_game, entry, _scratch) = session(None);
    let counts = published(&entry, &[143, 0, 1]);
    assert_eq!(counts.len(), 1, "one 143/1 at world entry");
    assert_eq!(*counts[0], spins(1, 82_800));
}

#[test]
fn start_wheel_offers_the_type_22_run_and_the_clients_start_launches_the_movie() {
    let (mut game, _entry, _scratch) = session(None);
    let out = send(&mut game, &START_WHEEL);
    let offer = offers(&out);
    assert_eq!(offer.len(), 1);
    let offer = offer[0];
    // 39/16 (`009bfc50`): header (instance, -1, game), name, icon, description, difficulty,
    // profile, type, members byte, three empty reward bundles, goal count, five flags.
    assert_eq!(offer[3..7], int(FIRST_INSTANCE));
    assert_eq!(offer[11..15], int(WHEEL_GAME));
    let field = |index: usize| offer[15 + 4 * index..19 + 4 * index].to_vec();
    assert_eq!(field(0), int(409_962).to_vec(), "name id");
    assert_eq!(field(1), int(20_985).to_vec(), "icon id");
    assert_eq!(field(2), int(409_969).to_vec(), "description id");
    assert_eq!(field(5), int(22).to_vec(), "TYPE 22");
    let flags = 15 + 24 + 1 + 3 * 69 + 4;
    assert_eq!(
        offer[flags..flags + 5],
        [1, 1, 1, 0, 1],
        "start-screen byte +0x267 off, the rest the client default"
    );
    assert!(!out.changed, "a session-only run marks no save");

    // The client asks to start by itself (39/5, `009be7d0`).
    let started = send(&mut game, &lifecycle(5, FIRST_INSTANCE));
    assert_eq!(
        started.replies,
        [[&[39u8, 0, 17][..], &offer[3..15]].concat(), flash_launch()]
    );
}

#[test]
fn the_games_window_play_request_for_the_wheel_row_opens_it_too() {
    let (mut game, _entry, _scratch) = session(None);
    assert_eq!(offers(&send(&mut game, &play(8))).len(), 1);
}

#[test]
fn a_second_start_while_the_wheel_is_open_is_refused_and_changes_nothing() {
    let (mut game, _entry, _scratch) = session(None);
    open(&mut game);
    for stage in 0..2 {
        if stage == 1 {
            send(&mut game, &lifecycle(5, FIRST_INSTANCE));
        }
        let before = game.state();
        let (dock, window) = (send(&mut game, &START_WHEEL), send(&mut game, &play(8)));
        for refused in [dock, window] {
            assert!(offers(&refused).is_empty(), "stage {stage}: no second run");
            assert_eq!(chat_lines(&refused), 1, "stage {stage}: one chat line");
            assert!(!refused.changed);
        }
        assert_eq!(game.state(), before, "stage {stage}: state unchanged");
    }
    // The first run is still the live one: its start works once.
    let started = send(&mut game, &lifecycle(5, FIRST_INSTANCE));
    assert!(started.replies.is_empty(), "the run was already started");
}

#[test]
fn leaving_the_wheel_frees_the_player_and_costs_no_spin() {
    let (mut game, _entry, _scratch) = session(None);
    open(&mut game);
    send(&mut game, &lifecycle(5, FIRST_INSTANCE));
    // The wheel's close button (39/6) is answered with the instance removal 39/19.
    let left = send(&mut game, &lifecycle(6, FIRST_INSTANCE));
    assert_eq!(left.replies.len(), 1);
    assert!(left.replies[0].starts_with(&[39, 0, 19]));
    assert_eq!(game.state().last_spin_day, None, "no spin used");
    // The player is free again: the wheel opens anew, on the next instance id.
    assert_eq!(open(&mut game), FIRST_INSTANCE + 1);
}

#[test]
fn a_forged_start_with_no_spin_left_is_refused_and_changes_nothing() {
    for last in [Some(TODAY), Some(TODAY + 5)] {
        // Spun today; and a clock that went backwards (the stamp is in the future).
        let (mut game, entry, _scratch) = session(last);
        assert_eq!(*published(&entry, &[143, 0, 1])[0], spins(0, 82_800));
        let before = game.state();
        let dock = send(&mut game, &START_WHEEL);
        let window = send(&mut game, &play(8));
        for refused in [dock, window] {
            assert!(offers(&refused).is_empty());
            assert_eq!(chat_lines(&refused), 1);
        }
        assert_eq!(game.state(), before);
        assert_eq!(before.last_spin_day, last, "the stamp is never rewritten");
        // Nothing was occupied by the refusals: a chess table opens.
        assert_eq!(offers(&send(&mut game, &play(CHESS_ROW))).len(), 1);
    }
}

#[test]
fn the_next_utc_day_offers_the_spin_again() {
    let (mut game, entry, _scratch) = session(Some(TODAY - 1));
    assert_eq!(*published(&entry, &[143, 0, 1])[0], spins(1, 82_800));
    open(&mut game);
}

#[test]
fn the_wheel_and_other_activities_refuse_each_other() {
    // Chess running: the wheel refuses.
    let (mut game, _entry, _scratch) = session(None);
    assert_eq!(offers(&send(&mut game, &play(CHESS_ROW))).len(), 1);
    let before = game.state();
    let refused = send(&mut game, &START_WHEEL);
    assert!(offers(&refused).is_empty());
    assert_eq!(chat_lines(&refused), 1);
    assert_eq!(game.state(), before);

    // Wheel open: chess refuses.
    let (mut game, _entry, _scratch) = session(None);
    open(&mut game);
    let refused = send(&mut game, &play(CHESS_ROW));
    assert!(offers(&refused).is_empty(), "no chess run under the wheel");
    assert_eq!(chat_lines(&refused), 1);
}

#[test]
fn entering_a_zone_ends_a_stale_wheel_without_using_the_spin() {
    let (mut game, _entry, _scratch) = session(None);
    open(&mut game);
    let entry = game.initialize_world(&character(), 2_000).unwrap();
    assert_eq!(*published(&entry, &[143, 0, 1])[0], spins(1, 82_800));
    // The player is free: chess starts, and so does a new wheel once chess is left.
    assert_eq!(offers(&send(&mut game, &play(CHESS_ROW))).len(), 1);
}

#[test]
fn a_relog_keeps_the_day_and_a_disconnect_mid_wheel_keeps_the_spin() {
    // Mid-wheel save: the run is session-only, so the stamp is still empty after a relog.
    let (mut game, _entry, scratch) = session(None);
    open(&mut game);
    send(&mut game, &lifecycle(5, FIRST_INSTANCE));
    let saved: GameplayState =
        serde_json::from_slice(&serde_json::to_vec(&game.state()).unwrap()).unwrap();
    assert_eq!(saved.last_spin_day, None);
    let mut relog = Gameplay::load(scratch.path(), Some(saved)).unwrap();
    relog.seed_wheel_clock(CLOCK);
    let entry = relog.initialize_world(&character(), 500).unwrap();
    assert_eq!(*published(&entry, &[143, 0, 1])[0], spins(1, 82_800));
    open(&mut relog);

    // A stamped day survives the save round trip and the relog.
    let mut stamped = game.state();
    stamped.last_spin_day = Some(TODAY);
    let saved: GameplayState =
        serde_json::from_slice(&serde_json::to_vec(&stamped).unwrap()).unwrap();
    assert_eq!(saved.last_spin_day, Some(TODAY));
    let mut relog = Gameplay::load(scratch.path(), Some(saved)).unwrap();
    relog.seed_wheel_clock(CLOCK);
    let entry = relog.initialize_world(&character(), 500).unwrap();
    assert_eq!(*published(&entry, &[143, 0, 1])[0], spins(0, 82_800));
    assert_eq!(relog.state().last_spin_day, Some(TODAY));
}

#[test]
fn the_shipped_content_loads_and_a_malformed_file_is_refused() {
    let shipped = Content::load_optional(&field_battle_fixture::content_dir())
        .unwrap()
        .unwrap();
    assert_eq!(
        (shipped.activity_id, shipped.minigame_id, shipped.type_id),
        (8, 8, 22)
    );
    assert_eq!(shipped.movie, "game_wheel.gfx");
    let empty = Scratch::new("daily-wheel-empty");
    assert!(Content::load_optional(empty.path()).unwrap().is_none());

    let document = json::read(&field_battle_fixture::content_dir().join("daily_wheel.json"));
    for (field, bad) in [
        ("movie", json!("../game_wheel.gfx")),
        ("minigame_id", json!(0)),
        ("activity_id", json!(0)),
        ("activity_id", json!(u32::MAX)),
        ("schema_version", json!(2)),
    ] {
        let mut broken = document.clone();
        broken[field] = bad;
        let dir = Scratch::new("daily-wheel-bad");
        json::write_pretty(&dir.path().join("daily_wheel.json"), &broken);
        assert!(
            Content::load_optional(dir.path()).is_err(),
            "{field} is validated at load"
        );
    }
}
