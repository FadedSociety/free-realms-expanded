//! T022: chess from the activity-portal entry request to the lifecycle payout.
//!
//! Drives the real `Gameplay` with raw inner packets, like `activity_portal_route.rs`.
//! Needs the private opening content (`OSFR_ACTIVITY_CONTENT` overrides the default).
//! All content, player ids and chat lines here are synthetic; no names are recorded.
use osfr_tools::{
    character_store::CharacterState,
    chess,
    chess_service::ChessRewardLedger,
    gameplay::{Gameplay, Outcome},
    inventory::InventoryItem,
    opening::{OpeningEngine, OpeningTrigger},
};
use std::path::PathBuf;

#[path = "support/chess_bot.rs"]
mod chess_bot;
use chess_bot::{EndOfGame, Human};

const LAKESHORE: [f32; 3] = [-733.952, 9.09, 945.973];
/// Easy single-player row; the portal id is the client minigame row id.
const CHESS_ROW: u32 = 20;
const GUMBO: i32 = 70;
const COOKING_INPUTS: [(i32, u32); 4] = [(2862, 4), (2875, 4), (2884, 4), (2900, 4)];

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

/// A Chef at the stove with the starter stew learned and supplied. Also
/// accepts the tutorial's cooking objective so the 26/10 station interaction
/// can start the cooking movie, and returns the station's runtime GUID.
fn cooking_game() -> (Gameplay, CharacterState, u64) {
    let directory = content();
    let base = Gameplay::load(&directory, None).unwrap();
    let mut state = base.state();
    state.learned_recipes.insert(GUMBO as u32);
    let profiles = state.profiles.as_mut().unwrap();
    profiles.active = 45;
    profiles.unlocked.insert(45);
    for (index, (definition_id, count)) in COOKING_INPUTS.into_iter().enumerate() {
        state.inventory.items.push(InventoryItem {
            definition_id,
            tint: 0,
            id: 7_000_001 + index as i32,
            count,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: 0,
            rental: None,
            ability_count: 0,
            activate_enabled: true,
        });
    }
    let opening = OpeningEngine::load(&directory, state.opening.clone()).unwrap();
    let (stage_id, objective_index) = opening
        .stages()
        .iter()
        .find_map(|stage| {
            stage
                .objectives
                .iter()
                .position(|goal| matches!(goal.trigger, OpeningTrigger::Craft { recipe_id: 2001 }))
                .map(|index| (stage.id, index))
        })
        .expect("tutorial cooking objective");
    for stage in opening.stages() {
        let progress = state
            .opening
            .as_mut()
            .unwrap()
            .stages
            .get_mut(&stage.id)
            .unwrap();
        if stage.id < stage_id {
            progress.accepted = true;
            progress.completed = true;
            progress.rewarded = true;
            progress.counts = stage.objectives.iter().map(|goal| goal.required).collect();
        } else if stage.id == stage_id {
            progress.accepted = true;
            progress.completed = false;
            progress.rewarded = false;
            progress.counts = stage
                .objectives
                .iter()
                .enumerate()
                .map(|(index, goal)| {
                    if index < objective_index {
                        goal.required
                    } else {
                        0
                    }
                })
                .collect();
        }
    }
    let station = opening
        .actors()
        .iter()
        .find(|actor| {
            matches!(
                actor.action,
                Some(OpeningTrigger::Craft { recipe_id: 2001 })
            )
        })
        .unwrap()
        .clone();
    let mut who = character();
    who.zone_id = station.zone_id;
    who.position = station.position;
    let game = Gameplay::load(&directory, Some(state)).unwrap();
    (game, who, station.guid)
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

#[test]
fn the_daily_chess_ledger_round_trips_through_the_save_and_gameplay_load() {
    let directory = content();
    let mut state = Gameplay::load(&directory, None).unwrap().state();
    let ledger = ChessRewardLedger {
        day: 3,
        paid_coins: 40,
    };
    state.chess = Some(ledger);
    let bytes = serde_json::to_vec(&state).unwrap();
    let restored: osfr_tools::gameplay::GameplayState = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(restored.chess, Some(ledger));
    let game = Gameplay::load(&directory, Some(restored)).unwrap();
    assert_eq!(game.state().chess, Some(ledger));
}

fn paid_won(out: &Outcome) -> bool {
    packets(out, &[39, 0, 18])
        .into_iter()
        .any(|packet| packet.get(15) == Some(&1))
}

fn coins(game: &Gameplay) -> u32 {
    game.state().inventory.coins
}

fn chess_row_coins() -> u32 {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/chess.json");
    osfr_tools::chess_service::ChessContent::load(&path)
        .unwrap()
        .rows
        .into_iter()
        .find(|row| row.row_id == CHESS_ROW)
        .expect("shipped chess row")
        .success_coins
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
    running_game_at(game, &character())
}

fn running_game_at(game: &mut Gameplay, who: &CharacterState) -> i32 {
    game.initialize_world(who, 500).unwrap();
    let offered = game.handle_in_world(&play(CHESS_ROW), who, 1_000).unwrap();
    let instance = offer_instance(&offered);
    assert!(offered.changed);

    let started = game
        .handle_in_world(&lifecycle(5, instance), who, 1_100)
        .unwrap();
    assert_eq!(packets(&started, &[39, 0, 17]).len(), 1, "39/17 ack");
    let launch = packets(&started, &[26, 0, 12]);
    assert_eq!(launch.len(), 1, "26/12 Flash launch");
    assert!(launch[0].windows(15).any(|w| w == b"game_chess.gfx\0"));

    let connected = game
        .handle_in_world(&movie(instance, "OnConnectMsg"), who, 1_200)
        .unwrap();
    assert_eq!(texts(&connected), ["OnServerReadyMsg"]);

    let began = game
        .handle_in_world(&movie(instance, "OnStartGameMsg\t0"), who, 1_300)
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

fn recipe_start(recipe: i32) -> Vec<u8> {
    let mut inner = vec![26, 0, 24, 0];
    inner.extend(recipe.to_le_bytes());
    inner
}

fn cooking_active(game: &Gameplay) -> bool {
    game.state()
        .cooking
        .as_ref()
        .is_some_and(|cooking| cooking.active.is_some())
}

#[test]
fn entry_request_uses_a_server_reply_and_ignores_client_opponent_moves() {
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

    // White e4. The server, not the client's local engine, answers for Black.
    let before = coins(&game);
    let e4 = human_move(&mut game, instance, 52, 36, 2_000);
    let events = texts(&e4);
    assert!(
        !events
            .iter()
            .any(|event| event.starts_with("OnChessEventMsg\t30")),
        "the client is not asked to choose the computer's move"
    );
    assert!(events.iter().any(|event| {
        [
            "OnChessEventMsg\t10",
            "OnChessEventMsg\t11",
            "OnChessEventMsg\t12",
            "OnChessEventMsg\t13",
        ]
        .iter()
        .any(|prefix| event.starts_with(prefix))
    }));
    assert!(events.contains(&"OnChessEventMsg\t1".to_owned()));

    // A client-selected losing reply is inert. It cannot alter the server's
    // position or turn the board into a payout. Repeating the old Scholar's
    // mate black replies still leaves the game to the server.
    for (now, bytes) in [
        (2_100, [12, 28, 0, 0x80]),
        (2_300, [1, 18, 0, 0]),
        (2_500, [6, 21, 0, 0]),
    ] {
        let server_state = game.state();
        let forged = computer_move(&mut game, instance, bytes, now);
        assert!(!forged.changed && forged.replies.is_empty());
        assert_eq!(game.state(), server_state);
        assert_eq!(coins(&game), before);
    }

    // The original table still accepts the human's next moves.
    let live = human_move(&mut game, instance, 61, 34, 2_200);
    assert!(has_event(&live, "OnChessEventMsg\t31"), "MOVE_DATA");
    let live = human_move(&mut game, instance, 59, 31, 2_400);
    assert!(has_event(&live, "OnChessEventMsg\t31"), "MOVE_DATA");

    // An unsolicited GameOver cannot cash a non-terminal run.
    let early = game
        .handle_in_world(&movie(instance, "FRServer_GameOver"), &who, 2_600)
        .unwrap();
    assert!(!early.changed && early.replies.is_empty());
    assert_eq!(coins(&game), before);

    // Quit retires the run; a new Play publishes a fresh instance id.
    let close = game
        .handle_in_world(&lifecycle(6, instance), &who, 2_700)
        .unwrap();
    assert!(close.changed, "quit retires the run");
    let again = game.handle_in_world(&play(CHESS_ROW), &who, 2_800).unwrap();
    assert_ne!(
        offer_instance(&again),
        instance,
        "fresh instance after Quit"
    );
}

#[test]
fn a_single_player_win_pays_once_after_acknowledgement_and_closes_cleanly() {
    let mut game = game();
    // (R) Deterministic table stream so the greedy line is a known win.
    game.seed_chess(22);
    let who = character();
    game.initialize_world(&who, 500).unwrap();
    let instance = running_game(&mut game);
    let payout = chess_row_coins();
    let before = coins(&game);
    let mut human = Human::new();
    let mut end = None;
    let mut last = None;
    for ply in 0..400u64 {
        let mv = human.choose();
        let text = human.move_text(&mv);
        human.position.apply(&mv).unwrap();
        human.remember();
        let out = game
            .handle_in_world(&movie(instance, &text), &who, 2_000 + ply * 10)
            .unwrap();
        if let Some(game_end) = human.apply_events(&texts(&out)) {
            end = Some(game_end);
            last = Some(out);
            break;
        }
    }
    assert_eq!(end, Some(EndOfGame::Won), "seed 22 must win");
    let won = last.unwrap();
    assert!(
        texts(&won).contains(&"OnGameWonMsg".to_owned()),
        "the winning result message"
    );
    assert!(
        texts(&won).contains(&"OnChessEventMsg\t15".to_owned()),
        "the checkmate event"
    );
    // The result screen and the payout wait for the client acknowledgement.
    assert!(packets(&won, &[39, 0, 18]).is_empty());
    assert_eq!(coins(&game), before);

    let ack = game
        .handle_in_world(&movie(instance, "FRServer_GameOver"), &who, 9_000)
        .unwrap();
    assert!(paid_won(&ack), "the acknowledged win pays");
    assert_eq!(packets(&ack, &[39, 0, 47]).len(), 1, "the score row");
    assert_eq!(packets(&ack, &[39, 0, 19]).len(), 1, "state removal");
    let results: Vec<u8> = ack
        .replies
        .iter()
        .filter_map(|packet| {
            (packet.starts_with(&[39, 0]) && matches!(packet.get(2), Some(18 | 47 | 19)))
                .then(|| packet[2])
        })
        .collect();
    assert_eq!(results, [18, 47, 19], "ordered result packets");
    let state = game.state();
    assert_eq!(coins(&game), before + payout, "exactly one content award");
    assert_eq!(state.chess.map(|ledger| ledger.paid_coins), Some(payout));

    // A duplicate acknowledgement and late movie traffic cannot pay again.
    let replay = game
        .handle_in_world(&movie(instance, "FRServer_GameOver"), &who, 9_100)
        .unwrap();
    assert!(!replay.changed && replay.replies.is_empty());
    assert_eq!(game.state(), state);
    let late = game
        .handle_in_world(&movie(instance, "OnMoveMsg\t52\t36"), &who, 9_200)
        .unwrap();
    assert!(!late.changed && late.replies.is_empty());
    assert_eq!(game.state(), state);

    // Results Close retires the finished run; the next Play is a fresh table.
    let close = game.handle_in_world(&[26, 0, 42, 0], &who, 9_300).unwrap();
    assert!(close.changed, "Close retires the finished run");
    assert_eq!(game.state(), state);
    let again = game.handle_in_world(&play(CHESS_ROW), &who, 9_400).unwrap();
    assert_eq!(packets(&again, &[39, 0, 16]).len(), 1);
    assert_ne!(offer_instance(&again), instance);
}

#[test]
fn active_chess_blocks_recipe_start_and_preserves_both_activities() {
    // Prove the fixture itself can start the learned recipe when idle.
    let (mut idle, idle_who, _) = cooking_game();
    idle.initialize_world(&idle_who, 500).unwrap();
    let started = idle
        .handle_in_world(&recipe_start(GUMBO), &idle_who, 1_000)
        .unwrap();
    assert!(started.changed, "the idle fixture starts cooking");
    assert!(cooking_active(&idle));

    // With chess open, the same request is admitted nowhere and changes
    // neither the activity set nor the saved gameplay snapshot.
    let (mut game, who, _) = cooking_game();
    let instance = running_game_at(&mut game, &who);
    let before = game.state();
    let refused = game
        .handle_in_world(&recipe_start(GUMBO), &who, 2_000)
        .unwrap();
    assert!(!refused.changed);
    assert!(!cooking_active(&game));
    assert_eq!(game.state(), before, "the refusal changed the saved state");
    assert!(chat_text(&refused).contains("Finish what you are doing"));
    assert!(packets(&refused, &[39, 0, 16]).is_empty());
    assert!(packets(&refused, &[26, 0, 12]).is_empty());

    // The refusal did not retire or freeze the original chess table.
    let live = game
        .handle_in_world(&movie(instance, "OnMoveMsg\t52\t36"), &who, 2_100)
        .unwrap();
    assert!(has_event(&live, "OnChessEventMsg\t31"), "MOVE_DATA");
}

fn interact(guid: u64) -> Vec<u8> {
    let mut inner = vec![26, 0, 10, 0];
    inner.extend(guid.to_le_bytes());
    inner.extend(1i32.to_le_bytes());
    inner
}

#[test]
fn active_chess_refuses_a_real_crafting_interaction_and_remains_playable() {
    // The idle request must start the cooking movie, or the refusal below
    // would not prove that chess prevented a real conflicting activity.
    let (mut idle, idle_who, station) = cooking_game();
    idle.initialize_world(&idle_who, 500).unwrap();
    let started = idle
        .handle_in_world(&interact(station), &idle_who, 1_000)
        .unwrap();
    assert!(
        started.changed,
        "the idle station interaction starts cooking"
    );
    assert!(cooking_active(&idle));

    let (mut game, who, station) = cooking_game();
    let instance = running_game_at(&mut game, &who);
    let before = game.state();
    let refused = game
        .handle_in_world(&interact(station), &who, 2_000)
        .unwrap();
    assert!(!refused.changed);
    assert!(!cooking_active(&game));
    assert_eq!(game.state(), before);
    assert!(chat_text(&refused).contains("Finish the chess game before interacting"));
    assert!(packets(&refused, &[39, 0, 16]).is_empty());
    let live = human_move(&mut game, instance, 52, 36, 2_100);
    assert!(has_event(&live, "OnChessEventMsg\t31"), "MOVE_DATA");
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
