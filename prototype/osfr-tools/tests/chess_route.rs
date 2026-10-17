//! T022: chess from the activity-portal entry request to the lifecycle payout.
//!
//! Drives the real `Gameplay` with raw inner packets, like `activity_portal_route.rs`.
//! Needs the private opening content (`OSFR_ACTIVITY_CONTENT` overrides the default).
//! All content, player ids and chat lines here are synthetic; no names are recorded.
use osfr_tools::{
    character_store::CharacterState,
    chess,
    gameplay::{Gameplay, Outcome},
};
use std::path::PathBuf;

const LAKESHORE: [f32; 3] = [-733.952, 9.09, 945.973];
/// Easy single-player row; the portal id is the client minigame row id.
const CHESS_ROW: u32 = 20;
/// `data/content/chess.json` (R): coins for a win on row 20.
const SUCCESS_COINS: u32 = 10;

fn content() -> PathBuf {
    std::env::var_os("OSFR_ACTIVITY_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/rust-opening-cooking-original-assets")
        })
}

fn character() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: LAKESHORE,
        rotation: [0.0, 0.0, 1.0],
    }
}

fn game() -> Gameplay {
    Gameplay::load(&content(), None).unwrap()
}

fn play(activity: u32) -> Vec<u8> {
    let mut inner = vec![167, 0, 1, 2];
    inner.extend(activity.to_le_bytes());
    inner.extend(0u32.to_le_bytes());
    inner
}

fn lifecycle(sub: u8, instance: i32) -> Vec<u8> {
    let mut inner = vec![39, 0, sub];
    for id in [instance, -1, -1] {
        inner.extend(id.to_le_bytes());
    }
    inner
}

fn movie(instance: i32, text: &str) -> Vec<u8> {
    chess::payload_packet(instance, text).unwrap()
}

fn packets<'a>(out: &'a Outcome, prefix: &[u8]) -> Vec<&'a Vec<u8>> {
    out.replies
        .iter()
        .filter(|reply| reply.starts_with(prefix))
        .collect()
}

fn texts(out: &Outcome) -> Vec<String> {
    packets(out, &[39, 0, 14])
        .into_iter()
        .map(|packet| {
            let length = i32::from_le_bytes(packet[15..19].try_into().unwrap()) as usize;
            String::from_utf8(packet[19..19 + length - 1].to_vec()).unwrap()
        })
        .collect()
}

fn chat_text(out: &Outcome) -> String {
    out.replies
        .iter()
        .filter(|reply| reply.starts_with(&[15, 0, 3, 0]))
        .map(|reply| {
            let length = u32::from_le_bytes(reply[4..8].try_into().unwrap()) as usize;
            String::from_utf8_lossy(&reply[8..8 + length]).into_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn has_event(out: &Outcome, prefix: &str) -> bool {
    texts(out).iter().any(|text| text.starts_with(prefix))
}

fn coins(game: &Gameplay) -> u32 {
    game.state().inventory.coins
}

/// The instance id of the single 39/16 offer.
fn offer_instance(out: &Outcome) -> i32 {
    let offers = packets(out, &[39, 0, 16]);
    assert_eq!(offers.len(), 1, "exactly one 39/16 offer");
    assert!(
        offers[0].len() > 15 + 3 * 69,
        "offer carries the three bundles"
    );
    let header = &offers[0][3..15];
    assert_eq!(
        i32::from_le_bytes(header[4..8].try_into().unwrap()),
        -1,
        "group is -1"
    );
    assert_eq!(
        i32::from_le_bytes(header[8..12].try_into().unwrap()),
        CHESS_ROW as i32,
        "chess row id in the info header"
    );
    i32::from_le_bytes(header[0..4].try_into().unwrap())
}

fn paid_won(out: &Outcome) -> bool {
    packets(out, &[39, 0, 18])
        .into_iter()
        .any(|packet| packet.get(15) == Some(&1))
}

/// `enter` publishes the portal list (167/1/1); return the row ids and join flags.
fn published_list(entry: &Outcome) -> (Vec<u32>, Vec<u8>) {
    let list = packets(entry, &[167, 0, 1, 1]);
    assert_eq!(list.len(), 1, "one list at world entry");
    let list = list[0];
    let count = i32::from_le_bytes(list[8..12].try_into().unwrap()) as usize;
    let mut at = 12;
    let mut ids = Vec::new();
    let mut joins = Vec::new();
    for _ in 0..count {
        ids.push(u32::from_le_bytes(list[at..at + 4].try_into().unwrap()));
        at += 10 * 4;
        joins.push(list[at]);
        at += 1 + 4 + 4 + 4 + 1;
        for _ in 0..2 {
            let length = i32::from_le_bytes(list[at..at + 4].try_into().unwrap()) as usize;
            at += 4 + length;
        }
        at += 4 * 4;
    }
    assert_eq!(at, list.len(), "the client reader consumes the whole list");
    (ids, joins)
}

/// Enter, start, connect and begin the game. Returns the instance id.
fn running_game(game: &mut Gameplay) -> i32 {
    let who = character();
    game.initialize_world(&who, 500).unwrap();
    let offered = game.handle_in_world(&play(CHESS_ROW), &who, 1_000).unwrap();
    let instance = offer_instance(&offered);
    assert!(offered.changed);

    let started = game
        .handle_in_world(&lifecycle(5, instance), &who, 1_100)
        .unwrap();
    assert_eq!(packets(&started, &[39, 0, 17]).len(), 1, "39/17 ack");
    let launch = packets(&started, &[26, 0, 12]);
    assert_eq!(launch.len(), 1, "26/12 Flash launch");
    assert!(launch[0].windows(15).any(|w| w == b"game_chess.gfx\0"));

    let connected = game
        .handle_in_world(&movie(instance, "OnConnectMsg"), &who, 1_200)
        .unwrap();
    assert_eq!(texts(&connected), ["OnServerReadyMsg"]);

    let began = game
        .handle_in_world(&movie(instance, "OnStartGameMsg\t0"), &who, 1_300)
        .unwrap();
    let began = texts(&began);
    assert!(
        began.contains(&"OnChessEventMsg\t0\t1\t1".to_owned()),
        "INIT white"
    );
    assert!(
        began.contains(&"OnChessEventMsg\t42\t1".to_owned()),
        "LOCAL_AI black"
    );
    assert!(
        began.contains(&"OnChessEventMsg\t1".to_owned()),
        "TURN_START"
    );
    instance
}

fn human_move(game: &mut Gameplay, instance: i32, from: u8, to: u8, now: u64) -> Outcome {
    game.handle_in_world(
        &movie(instance, &format!("OnMoveMsg\t{from}\t{to}")),
        &character(),
        now,
    )
    .unwrap()
}

fn computer_move(game: &mut Gameplay, instance: i32, bytes: [u8; 4], now: u64) -> Outcome {
    game.handle_in_world(
        &movie(
            instance,
            &format!(
                "OnMoveComputerMsg\t{}\t{}\t{}\t{}",
                bytes[0], bytes[1], bytes[2], bytes[3]
            ),
        ),
        &character(),
        now,
    )
    .unwrap()
}

#[test]
fn entry_request_to_payout_runs_once_through_the_minigame_lifecycle() {
    let mut game = game();
    let who = character();
    let entry = game.initialize_world(&who, 500).unwrap();
    let (ids, joins) = published_list(&entry);
    let chess_at = ids
        .iter()
        .position(|id| *id == CHESS_ROW)
        .expect("chess row");
    assert_eq!(joins[chess_at], 1, "chess needs no combat job");

    let instance = running_game(&mut game);

    // Scholar's mate: white e4, Bc4, Qh5, Qxf7; black answers with e5, Nc6, Nf6.
    let e4 = human_move(&mut game, instance, 52, 36, 2_000);
    assert!(
        texts(&e4).contains(&"OnChessEventMsg\t30\t1\t20".to_owned()),
        "MOVE_REQUEST"
    );
    let e5 = computer_move(&mut game, instance, [12, 28, 0, 0x80], 2_100);
    assert!(
        texts(&e5).contains(&"OnChessEventMsg\t10\t12\t28".to_owned()),
        "black move"
    );

    human_move(&mut game, instance, 61, 34, 2_200);
    computer_move(&mut game, instance, [1, 18, 0, 0], 2_300);
    human_move(&mut game, instance, 59, 31, 2_400);
    computer_move(&mut game, instance, [6, 21, 0, 0], 2_500);

    let before = coins(&game);
    let mate = human_move(&mut game, instance, 31, 13, 2_600);
    let mate_text = texts(&mate);
    assert!(
        mate_text.contains(&"OnChessEventMsg\t15".to_owned()),
        "CHECKMATE"
    );
    assert!(mate_text.contains(&"OnGameWonMsg".to_owned()));
    assert!(
        !paid_won(&mate),
        "the results sequence waits for the client ack"
    );
    assert_eq!(coins(&game), before, "no payout before the ack");

    // The movie answers a result with FRServer_GameOver; that is the only payout trigger.
    let ack = game
        .handle_in_world(&movie(instance, "FRServer_GameOver"), &who, 2_700)
        .unwrap();
    assert!(paid_won(&ack), "39/18 won");
    assert_eq!(packets(&ack, &[39, 0, 47]).len(), 1, "39/47 score rows");
    assert_eq!(packets(&ack, &[39, 0, 19]).len(), 1, "39/19 state removal");
    assert_eq!(coins(&game), before + SUCCESS_COINS, "one coin payout");

    // Replayed terminal packets cannot pay twice.
    let replay = game
        .handle_in_world(&movie(instance, "FRServer_GameOver"), &who, 2_800)
        .unwrap();
    assert!(!replay.changed && replay.replies.is_empty());
    assert_eq!(coins(&game), before + SUCCESS_COINS);

    // The results Close retires the run; a new Play publishes a new instance id.
    let close = game.handle_in_world(&[26, 0, 42, 0], &who, 2_900).unwrap();
    assert!(
        close.changed && close.replies.is_empty(),
        "Close retires silently"
    );
    let again = game.handle_in_world(&play(CHESS_ROW), &who, 3_000).unwrap();
    assert_ne!(
        offer_instance(&again),
        instance,
        "fresh instance after Close"
    );
}

#[test]
fn refusals_and_illegal_traffic_change_nothing() {
    let mut game = game();
    let who = character();
    game.initialize_world(&who, 500).unwrap();
    let before = coins(&game);

    // Unpublished ids are unknown and start nothing.
    for id in [0, 1, 21, CHESS_ROW + 1, u32::MAX] {
        let out = game.handle_in_world(&play(id), &who, 1_000).unwrap();
        assert!(packets(&out, &[39, 0, 16]).is_empty(), "id {id}");
        assert!(!out.changed && out.replies.len() == 1, "id {id}");
        assert!(chat_text(&out).contains("not available"), "id {id}");
    }
    assert_eq!(coins(&game), before);

    let instance = running_game(&mut game);
    // A second Play while the table is open is refused and sends no offer.
    let busy = game.handle_in_world(&play(CHESS_ROW), &who, 2_000).unwrap();
    assert!(packets(&busy, &[39, 0, 16]).is_empty());
    assert!(!busy.changed);
    assert!(chat_text(&busy).contains("Finish what you are doing"));
    // The original table still works after the refusal: its movie already sent
    // OnConnectMsg/OnStartGameMsg inside `running_game`, so a legal move is the
    // proof that the refusal did not disturb it.
    let live = human_move(&mut game, instance, 52, 36, 2_100);
    assert!(has_event(&live, "OnChessEventMsg\t31"), "MOVE_DATA");
    // A forged "engine has no move" reply cannot mint a win or a payout.
    let forged = computer_move(&mut game, instance, [0, 0, 0, 0], 2_120);
    assert!(!forged.changed && forged.replies.is_empty());
    assert!(!paid_won(&forged));
    assert_eq!(coins(&game), before);
    computer_move(&mut game, instance, [12, 28, 0, 0x80], 2_150);

    // An illegal move is refused with MOVE_INVALID and leaves the board playable.
    let illegal = human_move(&mut game, instance, 0, 16, 2_300);
    assert!(texts(&illegal).contains(&"OnChessEventMsg\t20".to_owned()));
    assert!(!illegal.changed);
    assert_eq!(coins(&game), before);
    let legal = human_move(&mut game, instance, 61, 34, 2_400);
    assert!(has_event(&legal, "OnChessEventMsg\t31"), "MOVE_DATA");

    // A payload for another instance is rejected outright and changes nothing.
    let wrong = game.handle_in_world(&movie(instance + 1, "OnMoveMsg\t36\t44"), &who, 2_500);
    assert!(wrong.is_err());
    assert_eq!(coins(&game), before);

    // GameOver without a finished game is inert.
    let early = game
        .handle_in_world(&movie(instance, "FRServer_GameOver"), &who, 2_600)
        .unwrap();
    assert!(!early.changed && early.replies.is_empty());
    assert_eq!(coins(&game), before);

    // Quit cancels the run without a payout; a new Play then works.
    let quit = game
        .handle_in_world(&lifecycle(6, instance), &who, 2_700)
        .unwrap();
    assert_eq!(
        packets(&quit, &[39, 0, 19]).len(),
        1,
        "quit removes the state"
    );
    assert!(!paid_won(&quit));
    assert_eq!(coins(&game), before);
    let again = game.handle_in_world(&play(CHESS_ROW), &who, 2_800).unwrap();
    assert_eq!(packets(&again, &[39, 0, 16]).len(), 1);
    assert_ne!(offer_instance(&again), instance);
}

#[test]
fn results_close_without_chess_is_inert_and_start_is_malformed_safe() {
    let mut game = game();
    let who = character();
    game.initialize_world(&who, 500).unwrap();
    let idle = game.handle_in_world(&[26, 0, 42, 0], &who, 900).unwrap();
    assert!(!idle.changed && idle.replies.is_empty());

    // Canary for the lifecycle parser: a short start body is an error, not a start.
    let offered = game.handle_in_world(&play(CHESS_ROW), &who, 1_000).unwrap();
    let instance = offer_instance(&offered);
    let mut short = lifecycle(5, instance);
    short.pop();
    assert!(game.handle_in_world(&short, &who, 1_100).is_err());
    // The untouched run still starts with the exact body.
    let started = game
        .handle_in_world(&lifecycle(5, instance), &who, 1_200)
        .unwrap();
    assert_eq!(packets(&started, &[39, 0, 17]).len(), 1);
}
