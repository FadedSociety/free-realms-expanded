//! Treasure War through the production dispatch: Play on the Games window row 1078 (the join the
//! client's own card doors send too), the offer, the movie's text both ways, a paid win, an
//! unpaid loss, and two sessions sharing one lobby game. The rules are tested in
//! `src/treasure_war.rs`, the tables in `src/treasure_war_table.rs`.
//!
//! The shipped `treasure_war.json` is closed (the movie is our own and has to reach players
//! first), so these fixtures open a private copy.
use osfr_tools::{
    account_store::AccountRole,
    gateway::{Gateway, GatewayOutcome},
    minigame::movie_text,
    power_ups::Rng,
    treasure_war_table::{GAME_TYPE, HostedMatch},
};
use serde_json::{Value, json};
use std::fs;

#[path = "common/content_copy.rs"]
#[allow(dead_code)]
mod content_copy;
#[path = "common/field_battle_fixture.rs"]
#[allow(dead_code)]
mod field_battle_fixture;
use field_battle_fixture::{Fixture, chat_text, coins, content_dir, portal_play, send, unframe};

const ROW: u32 = 1078;
const GAME: i32 = 727;
const LAKESHORE: [f32; 3] = [-733.952, 9.09, 945.973];

/// A production-shaped session root whose Treasure War file has `open` set.
fn fixture(open: bool) -> Option<Fixture> {
    if !field_battle_fixture::available() {
        eprintln!("skipped: private round70 content and gateway reference are absent");
        return None;
    }
    let mut fixture = Fixture::new();
    let root = fixture.root.join("content");
    fs::create_dir_all(&root).unwrap();
    for entry in fs::read_dir(&fixture.content).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            content_copy::hard_link_or_copy(&entry.path(), &root.join(entry.file_name()));
        }
    }
    let file = root.join("treasure_war.json");
    let mut content: Value =
        serde_json::from_slice(&fs::read(content_dir().join("treasure_war.json")).unwrap())
            .unwrap();
    assert_eq!(content["open"], json!(false), "the shipped row is closed");
    content["open"] = json!(open);
    // Never write through a hard link into the shared content.
    let _ = fs::remove_file(&file);
    fs::write(&file, serde_json::to_vec(&content).unwrap()).unwrap();
    content_copy::copy_missing(&content_dir(), &root);
    fixture.content = root;
    Some(fixture)
}

fn player(fixture: &Fixture, guid: u64) -> Gateway {
    let mut gateway = fixture.gateway(LAKESHORE, guid);
    gateway.set_dev_tools(false);
    gateway.set_role(AccountRole::Player);
    gateway
}

fn bodies(out: &GatewayOutcome) -> Vec<Vec<u8>> {
    out.replies
        .iter()
        .map(|reply| unframe(reply).to_vec())
        .collect()
}

fn i32_at(packet: &[u8], at: usize) -> i32 {
    i32::from_le_bytes(packet[at..at + 4].try_into().unwrap())
}

/// A client lifecycle packet for `instance`.
fn client(sub: u8, instance: i32) -> Vec<u8> {
    let mut packet = vec![39, 0, sub];
    for value in [instance, -1, -1] {
        packet.extend(value.to_le_bytes());
    }
    packet
}

/// The movie's 39/14 as the client sends it (`SendMiniGameFlashPayload`: game -1).
fn movie(instance: i32, text: &str) -> Vec<u8> {
    let mut packet = client(14, instance);
    packet.extend((text.len() as i32 + 1).to_le_bytes());
    packet.extend(text.as_bytes());
    packet.push(0);
    packet
}

fn texts(packets: &[Vec<u8>]) -> Vec<String> {
    let movie = packets
        .iter()
        .filter(|packet| packet.starts_with(&[39, 0, 14]));
    movie
        .map(|packet| movie_text(packet).unwrap().to_owned())
        .collect()
}

fn phase(texts: &[String]) -> Option<&str> {
    let phases = texts
        .iter()
        .filter_map(|text| text.strip_prefix("TW_Phase\t"));
    phases.last().and_then(|rest| rest.split('\t').next())
}

/// The move a seat that has heard `heard` makes next, taking the first card or treasure.
fn next_move(heard: &[String]) -> Option<String> {
    match phase(heard)? {
        "choose" => Some("TW_Play\t0".to_owned()),
        "attack" => {
            let board = heard
                .iter()
                .rev()
                .find(|text| text.starts_with("TW_Board"))?;
            let slot = board.split('\t').nth(2)?.find('d')?;
            Some(format!("TW_Take\t{slot}"))
        }
        _ => None,
    }
}

/// Plays one game from Play to the closed run; whether it was won.
fn play_one(gateway: &mut Gateway, now: &mut u64) -> bool {
    let mut tick = || {
        *now += 1_000;
        *now
    };
    let offered = bodies(&send(gateway, &portal_play(ROW), tick()));
    let offer = offered
        .iter()
        .find(|p| p.starts_with(&[39, 0, 16]))
        .expect("39/16 offer");
    let instance = i32_at(offer, 3);
    // The offer's type is the one the server declares, never the table's 23 (the exe's own card
    // client), and it names the card game's asset list.
    assert_eq!(i32_at(offer, 15 + 5 * 4), GAME_TYPE);
    assert!(
        offer.windows(20).any(|w| w == b"SimpleCardAssets.txt"),
        "the preload list"
    );
    let busy = send(gateway, &portal_play(ROW), tick());
    assert!(chat_text(&busy).contains("Finish"), "{}", chat_text(&busy));

    let started = bodies(&send(gateway, &client(5, instance), tick()));
    assert!(
        started.iter().any(|p| p[0] == 39),
        "the start was not answered: {started:?}"
    );
    let mut heard = texts(&bodies(&send(
        gateway,
        &movie(instance, "TW_Ready"),
        tick(),
    )));
    assert!(heard[0].starts_with("TW_Names\t"), "{heard:?}");
    for _ in 0..2_000 {
        let Some(next) = next_move(&heard) else { break };
        heard.extend(texts(&bodies(&send(
            gateway,
            &movie(instance, &next),
            tick(),
        ))));
    }
    let won = match phase(&heard) {
        Some("won") => true,
        Some("lost") => false,
        other => panic!("the game stopped in {other:?}"),
    };
    let before = coins(gateway);
    let settled = bodies(&send(
        gateway,
        &movie(instance, "FRServer_GameOver"),
        tick(),
    ));
    assert!(
        settled.iter().any(|p| p.starts_with(&[39, 0, 18])),
        "no result: {settled:?}"
    );
    assert_eq!(
        coins(gateway) - before,
        if won { 5 } else { 0 },
        "a win pays 5, a loss nothing"
    );
    let again = bodies(&send(
        gateway,
        &movie(instance, "FRServer_GameOver"),
        tick(),
    ));
    assert!(
        !again.iter().any(|p| p.starts_with(&[39, 0, 18])),
        "settled twice"
    );
    assert_eq!(coins(gateway) - before, if won { 5 } else { 0 });
    // The client closes the finished game's result (26/42).
    send(gateway, &[26, 0, 42, 0], tick());
    won
}

#[test]
fn the_shipped_row_stays_closed() {
    let Some(fixture) = fixture(false) else {
        return;
    };
    let mut gateway = player(&fixture, 5_301);
    let refused = send(&mut gateway, &portal_play(ROW), 1_000);
    assert_eq!(chat_text(&refused), "Treasure War is not open yet.");
    // No lobby seat either, and nothing to deal a lobby game from.
    let service = gateway.gameplay_mut().unwrap();
    assert!(
        service
            .begin_hosted_treasure_war(77, 1_500)
            .unwrap()
            .1
            .is_none()
    );
    assert!(service.treasure_war_content().is_none() && service.hosted_treasure_war().is_none());
    // The arena keeper offers no game menu while it is closed.
    let mut keeper = fixture.gateway([-1480.0, -39.5, -530.0], 5_307);
    keeper.set_dev_tools(false);
    keeper.set_role(AccountRole::Player);
    let mut click = vec![26, 0, 8, 0];
    click.extend(100_000_002_504_u64.to_le_bytes());
    click.push(0);
    let clicked = bodies(&send(&mut keeper, &click, 2_000));
    assert!(
        !clicked.iter().any(|p| p.starts_with(&[26, 0, 9, 0])),
        "a menu: {clicked:?}"
    );
    let mut choose = vec![26, 0, 10, 0];
    choose.extend(100_000_002_504_u64.to_le_bytes());
    choose.extend(1_i32.to_le_bytes());
    let chosen = bodies(&send(&mut keeper, &choose, 2_100));
    assert!(
        !chosen.iter().any(|p| p.starts_with(&[39, 0, 16])),
        "an offer: {chosen:?}"
    );
    assert!(!bodies(&refused).iter().any(|p| p.starts_with(&[39, 0, 16])));
}

#[test]
fn an_open_row_plays_to_a_paid_win_and_an_unpaid_loss() {
    let Some(fixture) = fixture(true) else { return };
    let mut gateway = player(&fixture, 5_302);
    let (mut wins, mut losses, mut now) = (0, 0, 1_000);
    for _ in 0..40 {
        if play_one(&mut gateway, &mut now) {
            wins += 1;
        } else {
            losses += 1;
        }
        if wins > 0 && losses > 0 {
            return;
        }
    }
    panic!("{wins} wins and {losses} losses in 40 games");
}

#[test]
fn a_quit_mid_game_leaves_the_player_free_to_play_again() {
    let Some(fixture) = fixture(true) else { return };
    let mut gateway = player(&fixture, 5_303);
    let offered = bodies(&send(&mut gateway, &portal_play(ROW), 1_000));
    let instance = i32_at(
        offered
            .iter()
            .find(|p| p.starts_with(&[39, 0, 16]))
            .unwrap(),
        3,
    );
    send(&mut gateway, &client(5, instance), 2_000);
    send(&mut gateway, &movie(instance, "TW_Ready"), 3_000);
    let before = coins(&gateway);
    let left = bodies(&send(&mut gateway, &client(6, instance), 4_000));
    assert!(left.iter().any(|p| p.starts_with(&[39, 0, 19])), "{left:?}");
    send(&mut gateway, &client(7, instance), 4_100);
    assert_eq!(coins(&gateway), before, "a quit pays nothing");
    let again = bodies(&send(&mut gateway, &portal_play(ROW), 5_000));
    assert!(
        again.iter().any(|p| p.starts_with(&[39, 0, 16])),
        "{}",
        again.len()
    );
}

/// One seat's movie text, with the lobby's game lent around the message as the lobby host does
/// for checkers; what the other seat is owed comes back.
fn seat_says(
    gateway: &mut Gateway,
    game: &mut HostedMatch,
    instance: i32,
    text: &str,
    now: u64,
) -> (Vec<Vec<u8>>, Vec<String>) {
    gateway
        .gameplay_mut()
        .unwrap()
        .lend_treasure_war(game.clone());
    let out = bodies(&send(gateway, &movie(instance, text), now));
    let (back, relay) = gateway
        .gameplay_mut()
        .unwrap()
        .reclaim_treasure_war()
        .unwrap();
    *game = back;
    (out, relay.into_iter().map(|message| message.text).collect())
}

#[test]
fn two_sessions_share_a_lobby_game_and_each_settles_once() {
    let Some(fixture) = fixture(true) else { return };
    let guids = [5_304_u64, 5_305];
    let mut seats = [player(&fixture, guids[0]), player(&fixture, guids[1])];
    let mut instances = [0; 2];
    for (seat, gateway) in seats.iter_mut().enumerate() {
        let service = gateway.gameplay_mut().unwrap();
        let (out, instance) = service.begin_hosted_treasure_war(77, 1_000).unwrap();
        instances[seat] = instance.expect("a free player takes the lobby game");
        assert!(
            out.replies.iter().any(|p| p.starts_with(&[39, 0, 16])),
            "the hidden offer"
        );
        assert_eq!(service.hosted_treasure_war(), Some(77));
        let twice = service.begin_hosted_treasure_war(77, 1_100).unwrap();
        assert!(twice.1.is_none(), "one run at a time");
    }
    let content = seats[0]
        .gameplay_mut()
        .unwrap()
        .treasure_war_content()
        .unwrap();
    let mut game = HostedMatch::new(
        77,
        (guids[0], instances[0]),
        (guids[1], instances[1]),
        content,
        Rng::seeded(21),
    );
    let mut heard = [Vec::new(), Vec::new()];
    let mut now = 2_000;
    let mut say = |heard: &mut [Vec<String>; 2],
                   game: &mut HostedMatch,
                   seats: &mut [Gateway; 2],
                   seat: usize,
                   text: &str| {
        now += 500;
        let (mine, theirs) = seat_says(&mut seats[seat], game, instances[seat], text, now);
        heard[seat].extend(texts(&mine));
        heard[1 - seat].extend(theirs);
        mine
    };
    for seat in [0, 1] {
        say(&mut heard, &mut game, &mut seats, seat, "TW_Ready");
    }
    for _ in 0..4_000 {
        let Some(waiting) = game.waiting_on() else {
            break;
        };
        let seat = usize::from(waiting == guids[1]);
        let next = next_move(&heard[seat]).expect("the seat the game waits on has a move");
        say(&mut heard, &mut game, &mut seats, seat, &next);
    }
    let before = [coins(&seats[0]), coins(&seats[1])];
    for seat in [0, 1] {
        let won = game.won(guids[seat]).expect("the game has a result");
        assert_eq!(phase(&heard[seat]), Some(if won { "won" } else { "lost" }));
        let settled = say(&mut heard, &mut game, &mut seats, seat, "FRServer_GameOver");
        assert!(
            settled.iter().any(|p| p.starts_with(&[39, 0, 18])),
            "seat {seat}: {settled:?}"
        );
        assert_eq!(
            coins(&seats[seat]) - before[seat],
            if won { 5 } else { 0 },
            "seat {seat}"
        );
        let again = say(&mut heard, &mut game, &mut seats, seat, "FRServer_GameOver");
        assert!(
            !again.iter().any(|p| p.starts_with(&[39, 0, 18])),
            "seat {seat} settled twice"
        );
    }
    assert!(game.all_settled());
    let both = game.won(guids[0]) == Some(true) && game.won(guids[1]) == Some(true);
    assert!(!both, "one winner at most");
}

/// The arena keeper offers the game while it is open: his menu, then the same offer.
#[test]
fn the_arena_keeper_offers_the_game() {
    const KEEPER: u64 = 100_000_002_504;
    let Some(fixture) = fixture(true) else { return };
    let mut gateway = fixture.gateway([-1480.0, -39.5, -530.0], 5_306);
    gateway.set_dev_tools(false);
    gateway.set_role(AccountRole::Player);
    let mut click = vec![26, 0, 8, 0];
    click.extend(KEEPER.to_le_bytes());
    click.push(0);
    let menu = bodies(&send(&mut gateway, &click, 2_000));
    assert!(
        menu.iter().any(|p| p.starts_with(&[26, 0, 9, 0])),
        "the menu: {menu:?}"
    );
    let mut choose = vec![26, 0, 10, 0];
    choose.extend(KEEPER.to_le_bytes());
    choose.extend(1_i32.to_le_bytes());
    let offered = bodies(&send(&mut gateway, &choose, 2_100));
    let offers: Vec<_> = (offered.iter())
        .filter(|p| p.starts_with(&[39, 0, 16]))
        .map(|p| i32_at(p, 11))
        .collect();
    assert_eq!(offers, [GAME]);
}
