//! T560: the daily wheel, part 1. World entry offers today's spin (143/1, reader `00adafa0`),
//! 26/11 (`009bdbd0`) or the Games window's Play opens a type-22 run, and the client's own
//! start (39/5) is answered with the `game_wheel.gfx` Flash launch (26/12). Driven through the
//! real `Gameplay` dispatch with raw inner packets, like `chess_route.rs`; the content is the
//! shipped `data/content` (a scratch directory without an NPC package binds it).
use osfr_tools::{
    character_store::CharacterState,
    daily_wheel::{Content, Ledger, NO_SPIN, START_WHEEL},
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
#[path = "common/content_copy.rs"]
mod content_copy;
use field_battle_fixture::portal_play as play;
use fishing_packets::lifecycle;
use scratch::Scratch;

const LAKESHORE: [f32; 3] = [-733.952, 9.09, 945.973];
const DAY: u64 = 86_400;
/// A pinned server clock: UTC day 20,000, one hour in.
const TODAY: u64 = 20_000;
const CLOCK: u64 = TODAY * DAY + 3_600;
const WHEEL_GAME: i32 = 8;
/// Single-player chess row (`chess_route.rs`): a real occupation with no extra content. L095: its
/// live Games window id (MiniGameData 20), the id Play names.
const CHESS_ROW: u32 = 515;
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
    session_in(Scratch::new("daily-wheel"), last_spin_day)
}

/// A fresh player whose content has the shipped item tables (`data/release`), so item prizes
/// pay exactly when a released server would pay them.
fn paying_session() -> (Gameplay, Outcome, Scratch) {
    let scratch = Scratch::new("daily-wheel-items");
    let release = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/release");
    for file in ["inventory.json", "item-table.json", "item-table-supplement.json"] {
        std::fs::copy(release.join(file), scratch.path().join(file)).expect(file);
    }
    session_in(scratch, None)
}

fn session_in(scratch: Scratch, last_spin_day: Option<u64>) -> (Gameplay, Outcome, Scratch) {
    let mut state = Gameplay::load(scratch.path(), None).unwrap().state();
    // `Some`: both wheels' free spins were used on that day.
    state.wheel_spins =
        last_spin_day.map_or_else(Ledger::default, |day| Ledger::spun_on(day, &["daily", "coin"]));
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
    assert_eq!(*counts[0], spins(2, 82_800));
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
    assert_eq!(*published(&entry, &[143, 0, 1])[0], spins(2, 82_800));
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
    assert_eq!(*published(&entry, &[143, 0, 1])[0], spins(2, 82_800));
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
    assert_eq!(*published(&entry, &[143, 0, 1])[0], spins(2, 82_800));
    open(&mut relog);

    // A stamped day survives the save round trip and the relog.
    let mut stamped = game.state();
    stamped.wheel_spins = Ledger::spun_on(TODAY, &["daily"]);
    let saved: GameplayState =
        serde_json::from_slice(&serde_json::to_vec(&stamped).unwrap()).unwrap();
    assert_eq!(saved.wheel_spins, Ledger::spun_on(TODAY, &["daily"]));
    let mut relog = Gameplay::load(scratch.path(), Some(saved)).unwrap();
    relog.seed_wheel_clock(CLOCK);
    let entry = relog.initialize_world(&character(), 500).unwrap();
    assert_eq!(*published(&entry, &[143, 0, 1])[0], spins(1, 82_800));
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
        ("schema_version", json!(3)),
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

/// Mounts piece: the wheel closes the ride gate (the client greys the ride button for the wheel's
/// minigame type and dismounts when a Flash minigame starts), so opening it ends a boarded ride
/// and no ride is summoned under it.
#[test]
fn opening_the_wheel_ends_a_boarded_ride_and_refuses_a_summon() {
    const WOLF: i32 = 993;
    const WOLF_ITEM: i32 = 14_289;
    let spawn = [vec![168, 0, 6], WOLF.to_le_bytes().to_vec()].concat();
    let ride = osfr_tools::mounts::MountService::ride_guid(character().guid);
    let board = [
        vec![168, 0, 1],
        ride.to_le_bytes().to_vec(),
        vec![1, 0, 0, 0, 0, 0],
    ]
    .concat();

    let scratch = Scratch::new("daily-wheel-ride");
    let mut state = Gameplay::load(scratch.path(), None).unwrap().state();
    state
        .inventory
        .items
        .push(osfr_tools::inventory::InventoryItem {
            definition_id: WOLF_ITEM,
            tint: 0,
            id: WOLF,
            count: 1,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: i64::from(i32::MAX),
            rental: None,
            ability_count: 0,
            activate_enabled: false,
        });
    let mut game = Gameplay::load(scratch.path(), Some(state)).unwrap();
    game.seed_wheel_clock(CLOCK);
    game.initialize_world(&character(), 500).unwrap();

    game.handle_in_world(&spawn, &character(), 10_000).unwrap();
    game.handle_in_world(&board, &character(), 10_010).unwrap();
    assert!(game.is_mounted(), "boarded before the wheel");

    open(&mut game);
    // The gateway settles rides after every handled message.
    game.settle_mounts(&character(), false);
    assert!(!game.is_mounted(), "the wheel ends the ride");
    assert!(game.ride_view(character().guid).is_none());

    game.handle_in_world(&spawn, &character(), 14_000).unwrap();
    assert!(
        game.ride_view(character().guid).is_none(),
        "no summon under the wheel"
    );
}

// ---- The movie's messages, the spin and its prize (piece daily-wheel) ----

/// A 39/14 movie message from the client for `instance`.
fn movie(instance: i32, text: &str) -> Vec<u8> {
    let mut packet = lifecycle(14, instance);
    packet.extend(int(text.len() as i32 + 1));
    packet.extend(text.as_bytes());
    packet.push(0);
    packet
}

/// The texts of the 39/14 messages in `out`, in order.
fn said(out: &Outcome) -> Vec<String> {
    published(out, &[39, 0, 14])
        .into_iter()
        .map(|packet| String::from_utf8(packet[19..packet.len() - 1].to_vec()).unwrap())
        .collect()
}

/// Open the wheel through the real entry path (26/11, the client's 39/5, the movie's connect)
/// and return the instance id with what the connect answered.
fn connected(game: &mut Gameplay) -> (i32, Vec<String>) {
    let instance = open(game);
    send(game, &lifecycle(5, instance));
    let hello = send(game, &movie(instance, "OnConnectMsg"));
    (instance, said(&hello))
}

/// One whole turn of wheel `id` landing on (`slot`, `prize`): the request, then the stop.
fn turn(game: &mut Gameplay, instance: i32, id: i32, slot: usize, prize: usize) -> (Outcome, Outcome) {
    game.seed_wheel_draw(slot, prize);
    let spin = send(game, &movie(instance, &format!("OnWheelSpinRequestMsg\t{id}")));
    let stop = send(game, &movie(instance, &format!("OnWheelStopMsg\t{id}")));
    (spin, stop)
}

/// 143/1 with a streak in the consecutive field.
fn count(spins_left: i32, streak: i32, seconds: i32) -> Vec<u8> {
    let mut bytes = spins(spins_left, seconds);
    bytes[11..15].copy_from_slice(&int(streak));
    bytes
}

#[test]
fn the_connect_sends_both_wheels_with_their_spins_then_ready() {
    let (mut game, _entry, _scratch) = session(None);
    let (instance, hello) = connected(&mut game);
    assert_eq!(
        hello,
        [
            "OnWheelDataMsg\t1\t1\t1 2 3 4 5 6 7 8 13 10\t408046\t409970",
            "OnWheelUpdateMsg\t1\t1\t0\t0",
            "OnWheelDataMsg\t2\t5\t15 16 15 18 22 17 16 15 21 22\t427102\t427101",
            "OnWheelUpdateMsg\t2\t1\t0\t0",
            "OnServerReadyMsg",
        ]
    );
    // A second connect adds no second set of wheels; the page buttons are confirmed.
    assert!(send(&mut game, &movie(instance, "OnConnectMsg")).replies.is_empty());
    for id in [1, 2] {
        let paged = send(&mut game, &movie(instance, &format!("OnChangeWheelRequestMsg\t{id}")));
        assert_eq!(said(&paged), [format!("OnWheelChangedMsg\t{id}")]);
    }
    assert!(send(&mut game, &movie(instance, "OnChangeWheelRequestMsg\t9")).replies.is_empty());
    assert_eq!(game.state().wheel_spins, Ledger::default(), "nothing is used by looking");
}

#[test]
fn a_spin_pays_the_wedge_the_needle_stops_on_once_and_only_shows_it_at_the_stop() {
    let (mut game, _entry, _scratch) = paying_session();
    let (instance, _) = connected(&mut game);
    let before = game.state().inventory.coins;
    // The Coin Wheel's wedge 8 is the coin marked 50.
    game.seed_wheel_draw(8, 0);
    let spin = send(&mut game, &movie(instance, "OnWheelSpinRequestMsg\t2"));
    assert_eq!(said(&spin), ["OnSpinInfoMsg\t8"], "the needle is told where to stop");
    assert_eq!(spin.replies.len(), 1, "nothing of the prize shows before the stop");
    assert!(spin.changed, "the paid spin is saved with the prize");
    assert_eq!(game.state().inventory.coins, before + 50);
    let ledger = serde_json::to_value(game.state().wheel_spins).unwrap();
    assert_eq!(ledger, json!({"coin": {"day": TODAY, "streak": 1}}));

    // A second request while the wheel turns, and one after: no spin left on this wheel.
    let paid = game.state();
    assert!(send(&mut game, &movie(instance, "OnWheelSpinRequestMsg\t2")).replies.is_empty());
    let stop = send(&mut game, &movie(instance, "OnWheelStopMsg\t2"));
    assert_eq!(
        said(&stop),
        [
            "OnRewardInfoMsg\t22663\t0\t50\t6513\t427100\t0",
            "OnWheelUpdateMsg\t2\t0\t1\t82800",
        ]
    );
    assert!(stop.replies.len() > 3, "the reward packets arrive with the window");
    assert_eq!(**published(&stop, &[143, 0, 1]).last().unwrap(), spins(1, 82_800));
    assert!(send(&mut game, &movie(instance, "OnWheelStopMsg\t2")).replies.is_empty());
    assert!(send(&mut game, &movie(instance, "OnWheelSpinRequestMsg\t2")).replies.is_empty());
    assert_eq!(game.state(), paid, "one spin, one prize");
}

#[test]
fn the_client_cannot_name_a_wedge_spin_early_or_spin_an_unknown_wheel() {
    let (mut game, _entry, _scratch) = paying_session();
    let instance = open(&mut game);
    send(&mut game, &lifecycle(5, instance));
    let fresh = game.state();
    // Before the movie connected.
    assert!(send(&mut game, &movie(instance, "OnWheelSpinRequestMsg\t1")).replies.is_empty());
    send(&mut game, &movie(instance, "OnConnectMsg"));
    for forged in [
        "OnWheelSpinRequestMsg\t9",
        "OnWheelSpinRequestMsg\tfirst",
        "OnWheelSpinRequestMsg",
        "OnSpinInfoMsg\t8",
        "OnRewardInfoMsg\t22663\t0\t5000\t6513\t0\t0",
        "OnWheelStopMsg\t1",
    ] {
        assert!(send(&mut game, &movie(instance, forged)).replies.is_empty(), "{forged}");
    }
    // Another run's instance id.
    assert!(send(&mut game, &movie(instance + 7, "OnWheelSpinRequestMsg\t1")).replies.is_empty());
    assert_eq!(game.state(), fresh);
    // Extra fields after the wheel id are not read: the server still draws the wedge itself.
    game.seed_wheel_draw(0, 0);
    let spin = send(&mut game, &movie(instance, "OnWheelSpinRequestMsg\t2\t8\t5000"));
    assert_eq!(said(&spin), ["OnSpinInfoMsg\t0"]);
    assert_eq!(game.state().inventory.coins, fresh.inventory.coins + 2);
}

#[test]
fn used_spins_survive_a_relog_and_a_start_without_one_gets_the_clients_text() {
    let (mut game, _entry, scratch) = paying_session();
    let (instance, _) = connected(&mut game);
    turn(&mut game, instance, 2, 0, 0);
    turn(&mut game, instance, 1, 0, 0);
    // The movie closes itself after the last wheel's reward window.
    let closed = send(&mut game, &movie(instance, "FRServer_GameClose"));
    assert_eq!(published(&closed, &[39, 0, 19]).len(), 1, "the run ends");
    let saved: GameplayState =
        serde_json::from_slice(&serde_json::to_vec(&game.state()).unwrap()).unwrap();
    assert_eq!(saved.last_spin_day, Some(TODAY));

    // The same day after a restart: no spin, and the refusal is the client's own sentence.
    let mut relog = Gameplay::load(scratch.path(), Some(saved.clone())).unwrap();
    relog.seed_wheel_clock(CLOCK + 3_600);
    let entry = relog.initialize_world(&character(), 500).unwrap();
    assert_eq!(*published(&entry, &[143, 0, 1])[0], count(0, 1, 79_200));
    let refused = send(&mut relog, &START_WHEEL);
    assert!(offers(&refused).is_empty());
    let line = published(&refused, &[15, 0, 3, 0]);
    assert_eq!(line.len(), 1);
    assert!(
        line[0].windows(NO_SPIN.len()).any(|part| part == NO_SPIN.as_bytes()),
        "\"{NO_SPIN}\""
    );
    assert_eq!(relog.state(), saved);

    // The next UTC day both wheels are back.
    let mut tomorrow = Gameplay::load(scratch.path(), Some(saved)).unwrap();
    tomorrow.seed_wheel_clock(CLOCK + DAY);
    let entry = tomorrow.initialize_world(&character(), 500).unwrap();
    assert_eq!(*published(&entry, &[143, 0, 1])[0], count(2, 1, 82_800));
}

#[test]
fn leaving_mid_turn_still_delivers_the_paid_prize_and_frees_no_spin() {
    let (mut game, _entry, _scratch) = paying_session();
    let (instance, _) = connected(&mut game);
    game.seed_wheel_draw(8, 0);
    send(&mut game, &movie(instance, "OnWheelSpinRequestMsg\t2"));
    let left = send(&mut game, &lifecycle(6, instance));
    assert!(left.replies.len() > 2, "the held reward packets go out with the leave");
    assert_eq!(**published(&left, &[143, 0, 1]).last().unwrap(), spins(1, 82_800));
    // The Daily Wheel is still there. The used Coin Wheel is not sent at all: the movie keeps
    // its current-wheel index at 0, so a spent wheel in front would swallow the spin button.
    let (instance, hello) = connected(&mut game);
    assert_eq!(
        hello,
        [
            "OnWheelDataMsg\t1\t1\t1 2 3 4 5 6 7 8 13 10\t408046\t409970",
            "OnWheelUpdateMsg\t1\t1\t0\t0",
            "OnServerReadyMsg",
        ]
    );
    assert!(send(&mut game, &movie(instance, "OnWheelSpinRequestMsg\t2")).replies.is_empty());
}

#[test]
fn the_extra_spin_wedge_gives_the_wheel_another_turn() {
    let (mut game, _entry, _scratch) = paying_session();
    let (instance, _) = connected(&mut game);
    // The Daily Wheel's wedge 3 is the wheel picture; its first prize is one extra spin.
    let (spin, stop) = turn(&mut game, instance, 1, 3, 0);
    assert_eq!(said(&spin), ["OnSpinInfoMsg\t3"]);
    assert_eq!(
        said(&stop),
        [
            "OnRewardInfoMsg\t20985\t0\t1\t409187\t412559\t0",
            "OnWheelUpdateMsg\t1\t1\t1\t0",
        ]
    );
    assert_eq!(**published(&stop, &[143, 0, 1]).last().unwrap(), count(2, 1, 82_800));
    // The extra spin is used next, and then the wheel is empty until tomorrow.
    let (again, stop) = turn(&mut game, instance, 1, 0, 0);
    assert_eq!(said(&again), ["OnSpinInfoMsg\t0"]);
    assert_eq!(said(&stop)[1], "OnWheelUpdateMsg\t1\t0\t1\t82800");
    assert!(send(&mut game, &movie(instance, "OnWheelSpinRequestMsg\t1")).replies.is_empty());
}

#[test]
fn three_days_in_a_row_bank_a_bonus_spin_with_the_clients_notice_and_a_gap_starts_over() {
    let (mut game, _entry, _scratch) = paying_session();
    let mut notices = Vec::new();
    // Days 0, 1, 2 in a row, then a missed day, then one more.
    for (day, streak, left_after) in [(0, 1, 0), (1, 2, 0), (2, 3, 1), (4, 1, 1)] {
        game.seed_wheel_clock(CLOCK + day * DAY);
        game.initialize_world(&character(), 500).unwrap();
        let (instance, _) = connected(&mut game);
        let (_, stop) = turn(&mut game, instance, 1, 0, 0);
        let next = if left_after == 0 { 82_800 } else { 0 };
        assert_eq!(
            said(&stop)[1],
            format!("OnWheelUpdateMsg\t1\t{left_after}\t{streak}\t{next}"),
            "day {day}"
        );
        notices.extend(published(&stop, &[143, 0, 3]).into_iter().cloned());
        send(&mut game, &movie(instance, "FRServer_GameClose"));
    }
    // One notice, on the third day: 143/3 id, day, spin, string, next day, next spin.
    let mut notice = vec![143, 0, 3];
    for value in [8, 3, 1, 415_176, 7, 2] {
        notice.extend(int(value));
    }
    assert_eq!(notices, [notice]);
}

#[test]
fn every_shipped_prize_is_paid_through_the_reward_path() {
    let (mut base, _entry, _scratch) = paying_session();
    let (instance, _) = connected(&mut base);
    let content = Content::load_optional(&field_battle_fixture::content_dir())
        .unwrap()
        .unwrap();
    let mut seen = 0;
    for (wheel, slot, at, prize) in content.prizes() {
        let mut game = base.clone();
        let coins = game.state().inventory.coins;
        let (spin, stop) = turn(&mut game, instance, wheel, slot, at);
        assert_eq!(
            said(&spin),
            [format!("OnSpinInfoMsg\t{slot}")],
            "wheel {wheel} wedge {slot} prize {at} is refused: {prize:?}"
        );
        let window: Vec<i32> = (said(&stop)[0].strip_prefix("OnRewardInfoMsg\t").unwrap())
            .split('\t')
            .map(|field| field.parse().unwrap())
            .collect();
        assert!(window[0] > 0 && window[2] > 0 && window[3] > 0, "{window:?} for {prize:?}");
        assert!(game.state().inventory.coins >= coins + prize.coins);
        seen += 1;
    }
    assert!(seen >= 60, "{seen} prizes");
}

#[test]
fn unweighted_spins_reach_every_wedge_of_both_wheels() {
    let (mut base, _entry, _scratch) = paying_session();
    let (instance, _) = connected(&mut base);
    for wheel in [1, 2] {
        let mut wedges = std::collections::BTreeSet::new();
        for seed in 0..200 {
            let mut game = base.clone();
            game.seed_wheel_roll(seed);
            let spin = send(&mut game, &movie(instance, &format!("OnWheelSpinRequestMsg\t{wheel}")));
            wedges.insert(said(&spin)[0].clone());
        }
        assert_eq!(wedges.len(), 10, "wheel {wheel}: {wedges:?}");
    }
}

#[test]
fn malformed_wheels_are_refused_at_load() {
    let document = json::read(&field_battle_fixture::content_dir().join("daily_wheel.json"));
    let mut cases = Vec::new();
    let mut short = document.clone();
    short["wheels"][0]["slots"].as_array_mut().unwrap().pop();
    cases.push(("nine wedges on a ten-wedge wheel", short));
    for (what, pointer, bad) in [
        ("a wedge weight of zero", "/wheels/0/slots/0/weight", json!(0)),
        ("a picture frame the movie lacks", "/wheels/0/slots/0/category", json!(26)),
        ("a prize that pays nothing", "/wheels/1/slots/0/prizes/0/coins", json!(0)),
        ("a wheel type without art", "/wheels/1/wheel_type", json!(3)),
        ("two wheels with one save key", "/wheels/1/key", json!("daily")),
        ("two wheels with one id", "/wheels/1/wheel_id", json!(1)),
        ("a missing coin picture", "/coin_icon_id", json!(0)),
        ("schema 1", "/schema_version", json!(1)),
    ] {
        let mut broken = document.clone();
        *broken.pointer_mut(pointer).unwrap() = bad;
        cases.push((what, broken));
    }
    for (what, broken) in cases {
        let dir = Scratch::new("daily-wheel-bad-wheel");
        json::write_pretty(&dir.path().join("daily_wheel.json"), &broken);
        assert!(Content::load_optional(dir.path()).is_err(), "{what}");
    }
}

#[test]
fn a_full_bag_refuses_the_spin_before_the_draw_whatever_it_would_have_paid() {
    let (mut game, _entry, scratch) = paying_session();
    // Win one item, then fill the bag to its 100 rows with copies of it. The next day both
    // wheels have their spin again.
    let (instance, _) = connected(&mut game);
    turn(&mut game, instance, 1, 0, 0);
    let mut state = game.state();
    let filler = (state.inventory.items.last().cloned()).expect("the shard just won");
    let mut id = state.inventory.items.iter().map(|item| item.id).max().unwrap() + 1;
    while state.inventory.items.len() < 100 {
        let mut item = filler.clone();
        item.id = id;
        id += 1;
        state.inventory.items.push(item);
    }
    state.inventory.next_instance_id = Some(i64::from(id));
    let mut game = Gameplay::load(scratch.path(), Some(state)).unwrap();
    game.seed_wheel_clock(CLOCK + DAY);
    game.initialize_world(&character(), 500).unwrap();
    let (instance, _) = connected(&mut game);
    let before = game.state();
    // An Extra Spin, a coin wedge and an item wedge are refused alike: the refusal comes before
    // the draw, so a full bag cannot keep the prizes that need no room and reroll the rest.
    for (wheel, slot) in [(1, 3), (2, 8), (1, 0)] {
        let mut probe = game.clone();
        probe.seed_wheel_draw(slot, 0);
        let refused = send(&mut probe, &movie(instance, &format!("OnWheelSpinRequestMsg\t{wheel}")));
        assert!(said(&refused).is_empty(), "wheel {wheel} wedge {slot}: no needle");
        assert_eq!(chat_lines(&refused), 1, "wheel {wheel} wedge {slot}");
        assert_eq!(published(&refused, &[39, 0, 19]).len(), 1, "the wheel closes");
        assert!(!refused.changed);
        assert_eq!(probe.state(), before, "wheel {wheel} wedge {slot}: nothing paid, the spin kept");
    }
}

#[test]
fn a_player_online_across_utc_midnight_is_sent_the_new_days_count_once() {
    let (mut game, _entry, _scratch) = session(Some(TODAY));
    let counts = |tick: Option<Outcome>| {
        tick.map_or(0, |out| published(&out, &[143, 0, 1]).len())
    };
    assert_eq!(counts(game.pet_tick(&character(), 2_000)), 0, "the same day");
    assert!(offers(&send(&mut game, &START_WHEEL)).is_empty(), "no spin today");
    game.seed_wheel_clock(CLOCK + DAY);
    let midnight = game.pet_tick(&character(), 3_000).expect("the new day's count");
    assert_eq!(*published(&midnight, &[143, 0, 1])[0], spins(2, 82_800));
    assert_eq!(counts(game.pet_tick(&character(), 4_000)), 0, "once");
    assert_eq!(offers(&send(&mut game, &START_WHEEL)).len(), 1, "and the wheel opens");
}

/// F3 (rc-0136 m-wheel, owner: "Says I'm out of spins for the day even though I never got one"):
/// a fresh character is in the opening campaign, whose world entry sent the Games window lists
/// (the wheel row live) but never the 143/1 count, so the client read no spin and refused it
/// itself; a start that did reach the server was refused for the opening too. The released
/// content (data/content + data/release, the opening included), a fresh character at its opening
/// checkpoint: world entry publishes both spins, the first spin of each wheel pays, and the next
/// start that day is the out-of-spins refusal.
#[test]
fn a_fresh_character_in_the_opening_gets_its_first_spins_then_the_refusal() {
    let scratch = Scratch::new("daily-wheel-opening");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    content_copy::copy_tree(&root.join("data/content"), scratch.path());
    content_copy::copy_tree(&root.join("data/release"), scratch.path());
    let fresh = Gameplay::initial_character(scratch.path(), character()).unwrap();
    let mut game = Gameplay::load(scratch.path(), None).unwrap();
    assert!(game.state().opening.is_some(), "a fresh character starts in the opening campaign");
    game.seed_wheel_clock(CLOCK);
    let entry = game.initialize_world(&fresh, 500).unwrap();
    assert_eq!(published(&entry, &[143, 0, 1]), [&spins(2, 82_800)], "the count is published");

    let (instance, _) = connected(&mut game);
    let (_, coin_stop) = turn(&mut game, instance, 2, 0, 0);
    assert!(!said(&coin_stop).is_empty(), "the coin wheel's first spin stops and pays");
    let (_, daily_stop) = turn(&mut game, instance, 1, 0, 0);
    assert!(!said(&daily_stop).is_empty(), "the daily wheel's first spin stops and pays");
    send(&mut game, &movie(instance, "FRServer_GameClose"));
    assert_eq!(game.state().last_spin_day, Some(TODAY));

    let refused = send(&mut game, &START_WHEEL);
    assert!(offers(&refused).is_empty(), "no second run today");
    let line = published(&refused, &[15, 0, 3, 0]);
    assert!(
        line.len() == 1 && line[0].windows(NO_SPIN.len()).any(|part| part == NO_SPIN.as_bytes()),
        "\"{NO_SPIN}\""
    );
}
