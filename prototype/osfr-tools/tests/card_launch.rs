//! T765: a Trading Card Game row with a launch block is admitted and runs the client's type-16
//! handshake: 39/16 offer, 39/5 -> 39/17 (no Flash launch), 39/37 -> 39/0x28 with a one-time
//! launch ticket the card login redeems once, 39/6 leave. Packets are shaped like the client's
//! (`009bdc60`: sub, instance id, -1, -1).
//! T782: the client's close of the start screen (26/42, then 39/6 and 39/7) through the Gateway.
use osfr_tools::{
    account_store::AccountRole,
    activity_portal::{Catalogue, Refusal, Viewer},
    card_game::{self, Content, FIRST_INSTANCE_ID, Launches},
    gateway::{Gateway, GatewayOutcome},
};
use serde_json::{Value, json};
use std::{fs, path::PathBuf, sync::Arc};

#[path = "common/content_copy.rs"]
#[allow(dead_code)] // this suite only overlays and links; the helper also has `copy_tree`
mod content_copy;
#[path = "common/field_battle_fixture.rs"]
mod field_battle_fixture;
use field_battle_fixture::{Fixture, content_dir, portal_play, send, unframe};

const TCG_ROW: u32 = 490;
const TUTORIAL_ROW: u32 = 1049;
const TREASURE_WAR_ROW: u32 = 1078;
const LAKESHORE: [f32; 3] = [-733.952, 9.09, 945.973];
const CHARACTER: u64 = 7_000_123;
const VIEWER: Viewer = Viewer {
    level: 1,
    combat_job: true,
};

fn shipped() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/card_game.json");
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

/// The shipped card rows plus a launch block (the shipped file has none).
fn launch_json(ttl_ms: u64) -> Value {
    let mut content = shipped();
    content["launch"] = json!({
        "ticket_ttl_ms": ttl_ms,
        "source": [{"kind": "reconstructed", "evidence": "T765 test"}]
    });
    content
}

/// `launch_json` loaded through the real loader.
fn with_launch(ttl_ms: u64) -> std::io::Result<Content> {
    let path = std::env::temp_dir().join(format!(
        "osfr-card-launch-{}-{ttl_ms}.json",
        std::process::id()
    ));
    fs::write(&path, serde_json::to_vec(&launch_json(ttl_ms)).unwrap()).unwrap();
    let loaded = Content::load(&path);
    let _ = fs::remove_file(&path);
    loaded
}

/// A production-shaped session root whose card file has a launch block: the runtime content,
/// the bundled content over it, and `launch_json`.
fn launch_fixture() -> Fixture {
    let mut fixture = Fixture::new();
    let root = fixture.root.join("content");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("card_game.json"),
        serde_json::to_vec(&launch_json(120_000)).unwrap(),
    )
    .unwrap();
    for entry in fs::read_dir(&fixture.content).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            content_copy::hard_link_or_copy(&entry.path(), &root.join(entry.file_name()));
        }
    }
    content_copy::copy_missing(&content_dir(), &root);
    fixture.content = root;
    fixture
}

/// An ordinary player in a production-shaped gateway session.
fn player(fixture: &Fixture, guid: u64) -> Gateway {
    let mut gateway = fixture.gateway(LAKESHORE, guid);
    gateway.set_dev_tools(false);
    gateway.set_role(AccountRole::Player);
    gateway
}

/// The application bodies a gateway answer carries, tunnel stripped.
fn bodies(out: &GatewayOutcome) -> Vec<Vec<u8>> {
    out.replies.iter().map(|r| unframe(r).to_vec()).collect()
}

/// A client lifecycle packet for `instance`.
fn client(sub: u8, instance: i32) -> Vec<u8> {
    let mut packet = vec![39, 0, sub];
    for value in [instance, -1, -1] {
        packet.extend(value.to_le_bytes());
    }
    packet
}

fn i32_at(packet: &[u8], at: usize) -> i32 {
    i32::from_le_bytes(packet[at..at + 4].try_into().unwrap())
}

fn subs(packets: &[Vec<u8>]) -> Vec<(u8, u8)> {
    packets.iter().map(|p| (p[0], p[2])).collect()
}

#[test]
fn a_launch_block_admits_trading_card_rows_but_not_treasure_war() {
    let cards = Arc::new(with_launch(120_000).unwrap());
    let catalogue = Catalogue::assemble(None, None, None, Some(&cards)).unwrap();
    assert!(catalogue.admit(TCG_ROW, VIEWER, false).is_ok());
    assert_eq!(catalogue.admit(TCG_ROW, VIEWER, true), Err(Refusal::Busy));
    assert_eq!(
        catalogue.admit(TREASURE_WAR_ROW, VIEWER, false),
        Err(Refusal::TreasureWarClosed)
    );
    // The shipped file has no launch block: every card row stays closed.
    let closed = Arc::new(
        Content::load_optional(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content"))
            .unwrap()
            .unwrap(),
    );
    assert!(closed.launch().is_none());
    let catalogue = Catalogue::assemble(None, None, None, Some(&closed)).unwrap();
    assert_eq!(
        catalogue.admit(TCG_ROW, VIEWER, false),
        Err(Refusal::CardGameClosed)
    );
    // The ticket lifetime is bounded.
    assert!(with_launch(999).is_err());
    assert!(with_launch(600_001).is_err());
}

#[test]
fn the_type_16_handshake_sends_a_one_time_ticket_in_39_28() {
    let cards = with_launch(120_000).unwrap();
    let row = cards.row(TCG_ROW).unwrap();
    let mut launches = Launches::new(cards.launch().unwrap()).unwrap();
    assert!(!launches.is_active());

    let offer = launches.begin(row).unwrap();
    assert_eq!(subs(&offer.replies), vec![(39, 16)]);
    let instance = i32_at(&offer.replies[0], 3);
    assert_eq!(instance, FIRST_INSTANCE_ID);
    assert!(launches.is_active());

    // A ticket request before the run starts is ignored.
    let early = launches
        .route(&client(37, instance), CHARACTER, 1_000)
        .unwrap();
    assert!(!early.accepted && early.replies.is_empty());

    // 39/5 is answered with 39/17 alone: the client hosts the card DLL itself.
    let start = launches
        .route(&client(5, instance), CHARACTER, 1_100)
        .unwrap();
    assert_eq!(subs(&start.replies), vec![(39, 17)]);

    // Another instance's request is ignored; the run's own gets 39/0x28.
    let stray = launches
        .route(&client(37, instance + 1), CHARACTER, 1_200)
        .unwrap();
    assert!(stray.replies.is_empty());
    let asked = launches
        .route(&client(37, instance), CHARACTER, 1_300)
        .unwrap();
    assert!(asked.accepted && !asked.changed, "a ticket marks no save");
    assert_eq!(asked.replies.len(), 1);
    let packet = &asked.replies[0];
    assert_eq!(packet.len(), 15 + 8, "the run header, then two i32 halves");
    assert_eq!(
        (packet[0], packet[2]),
        (39, 0x28),
        "TradingCardStartGamePacket"
    );
    assert_eq!(
        (i32_at(packet, 3), i32_at(packet, 7), i32_at(packet, 11)),
        (instance, -1, row.minigame_id as i32)
    );
    let low = u32::from_le_bytes(packet[15..19].try_into().unwrap());
    let high = u32::from_le_bytes(packet[19..23].try_into().unwrap());
    let ticket = (u64::from(high) << 32) | u64::from(low);
    assert_ne!(ticket, 0);

    // The card login redeems it once, for this character only.
    let character = CHARACTER.to_string();
    assert_eq!(launches.redeem(&ticket.to_string(), "1", 2_000), None);
    let again = launches
        .route(&client(37, instance), CHARACTER, 2_100)
        .unwrap();
    let ticket = {
        let p = &again.replies[0];
        (u64::from(u32::from_le_bytes(p[19..23].try_into().unwrap())) << 32)
            | u64::from(u32::from_le_bytes(p[15..19].try_into().unwrap()))
    };
    assert_eq!(
        launches.redeem(&ticket.to_string(), &character, 2_200),
        Some(CHARACTER)
    );
    assert_eq!(
        launches.redeem(&ticket.to_string(), &character, 2_300),
        None
    );

    // Leaving the card window ends the run (39/19); later requests are ignored.
    let left = launches
        .route(&client(6, instance), CHARACTER, 3_000)
        .unwrap();
    assert_eq!(subs(&left.replies), vec![(39, 19)]);
    assert!(!launches.is_active());
    let after = launches
        .route(&client(37, instance), CHARACTER, 3_100)
        .unwrap();
    assert!(after.replies.is_empty());
}

#[test]
fn card_runs_use_the_client_game_type_and_their_own_instance_range() {
    assert_eq!(card_game::CARD_GAME_TYPE, 16);
    let cards = with_launch(60_000).unwrap();
    let mut launches = Launches::new(cards.launch().unwrap()).unwrap();
    let offer = launches.begin(cards.row(TCG_ROW).unwrap()).unwrap();
    // 39/16 info: the run's header carries the row's MiniGameData id.
    assert_eq!(i32_at(&offer.replies[0], 11), 500);
}

/// What the native client sends when its card start screen is closed (X): the generic 26/42
/// Close, then 39/6 (CancelMiniGame) and 39/7, the order three native plays showed. The binary
/// hands each message of a combined datagram to the Gateway one at a time, as here.
#[test]
fn the_clients_close_of_the_start_screen_retires_the_offered_run_once() {
    // The bytes the native client sent for the first run (instance 61,000,001).
    let captured = |sub: u8| [vec![0x27, 0, sub, 0x41, 0xc9, 0xa2, 3], vec![0xff; 8]].concat();
    assert_eq!(client(6, FIRST_INSTANCE_ID), captured(6));
    assert_eq!(client(7, FIRST_INSTANCE_ID), captured(7));

    let fixture = launch_fixture();
    for (n, row) in [TCG_ROW, TUTORIAL_ROW].into_iter().enumerate() {
        let mut gateway = player(&fixture, 5_061 + n as u64);
        gateway.set_trace_enabled(true);
        let offer_of = |out: &GatewayOutcome| {
            bodies(out)
                .into_iter()
                .find(|p| p.starts_with(&[39, 0, 16]))
        };
        let offer = offer_of(&send(&mut gateway, &portal_play(row), 1_000)).expect("39/16 offer");
        let instance = i32_at(&offer, 3);
        assert_eq!(instance, FIRST_INSTANCE_ID, "row {row}");
        let before = gateway.snapshot();

        // The generic Close is not the card run's cancel: the run stays offered, so the player
        // is still busy.
        let close = send(&mut gateway, &[26, 0, 42, 0], 2_000);
        assert!(close.replies.is_empty(), "row {row}");
        let busy = send(&mut gateway, &portal_play(row), 2_500);
        assert!(
            field_battle_fixture::chat_text(&busy).contains("Finish what you are doing"),
            "row {row}: the run is still offered"
        );
        assert_eq!(offer_of(&busy), None, "row {row}");

        // 39/6 retires it with the state-remove reply; 39/7 is accepted without a reply.
        let cancel = bodies(&send(&mut gateway, &client(6, instance), 3_000));
        assert_eq!(subs(&cancel), vec![(39, 19)], "row {row}");
        assert_eq!(i32_at(&cancel[0], 3), instance, "row {row}");
        let ack = send(&mut gateway, &client(7, instance), 3_010);
        assert!(ack.replies.is_empty(), "row {row}");
        assert_eq!(
            ack.trace[0].outcome, "handled",
            "row {row}: {:?}",
            ack.trace
        );

        // A repeated cancel retires nothing; no save changed through the whole close.
        let repeat = send(&mut gateway, &client(6, instance), 3_020);
        assert!(repeat.replies.is_empty(), "row {row}");
        assert!(
            !repeat.state_changed && !repeat.gameplay_changed,
            "row {row}"
        );
        assert_eq!(gateway.snapshot(), before, "row {row}");

        // The player is free again: the next Play offers the next instance.
        let again = offer_of(&send(&mut gateway, &portal_play(row), 4_000)).expect("second offer");
        assert_eq!(i32_at(&again, 3), FIRST_INSTANCE_ID + 1, "row {row}");
    }
}

/// The cancel pair on its own (no 26/42 before it): 39/6 retires the offered run with the
/// state-remove reply and 39/7 is understood, not traced as an unknown message.
#[test]
fn the_cancel_pair_alone_retires_the_offered_run_and_is_understood() {
    let fixture = launch_fixture();
    let mut gateway = player(&fixture, 5_063);
    gateway.set_trace_enabled(true);
    let offered = bodies(&send(&mut gateway, &portal_play(TCG_ROW), 1_000));
    let instance = offered
        .iter()
        .find(|p| p.starts_with(&[39, 0, 16]))
        .map(|p| i32_at(p, 3))
        .expect("39/16 offer");
    let cancel = send(&mut gateway, &client(6, instance), 2_000);
    assert_eq!(subs(&bodies(&cancel)), vec![(39, 19)]);
    assert_eq!(cancel.trace[0].outcome, "handled", "{:?}", cancel.trace);
    let ack = send(&mut gateway, &client(7, instance), 2_010);
    assert!(ack.replies.is_empty());
    assert_eq!(ack.trace[0].outcome, "handled", "{:?}", ack.trace);
}

/// Only a card run's own 39/x packets reach it. The generic 26/42 Close that the client sends
/// before its cancel ends neither an offered nor a started run; 39/6 does, once, for its own
/// instance, with 39/7 as the ack. Refused and malformed packets change nothing.
#[test]
fn only_the_cancel_ends_a_card_run_and_only_by_its_own_instance() {
    let cards = with_launch(120_000).unwrap();
    let mut launches = Launches::new(cards.launch().unwrap()).unwrap();
    let offer = launches.begin(cards.row(TCG_ROW).unwrap()).unwrap();
    let instance = i32_at(&offer.replies[0], 3);

    for stranger in [
        vec![26, 0, 42, 0],
        client(6, instance + 1),
        client(7, instance - 1),
        vec![39, 0, 6],
    ] {
        assert!(!launches.owns(&stranger), "{stranger:02x?}");
    }
    // A cancel for another game, a sub the run does not take, and wrong lengths: ignored or
    // refused while offered, and again once started.
    let mut other_game = client(6, instance);
    other_game[11..15].copy_from_slice(&500i32.to_le_bytes());
    let ignored = [other_game, client(46, instance), client(14, instance)];
    let malformed = [
        client(6, instance)[..14].to_vec(),
        [client(6, instance), vec![0]].concat(),
    ];
    for started in [false, true] {
        for packet in &ignored {
            assert!(launches.owns(packet));
            let out = launches.route(packet, CHARACTER, 1_000).unwrap();
            assert!(!out.accepted && out.replies.is_empty(), "{packet:02x?}");
        }
        for packet in &malformed {
            assert!(launches.owns(packet));
            assert!(launches.route(packet, CHARACTER, 1_000).is_err());
        }
        assert!(launches.is_active());
        if !started {
            let start = launches
                .route(&client(5, instance), CHARACTER, 1_100)
                .unwrap();
            assert_eq!(subs(&start.replies), vec![(39, 17)]);
        }
    }

    let cancelled = launches
        .route(&client(6, instance), CHARACTER, 2_000)
        .unwrap();
    assert_eq!(subs(&cancelled.replies), vec![(39, 19)]);
    assert!(!launches.is_active());
    // The ack of the ended run is still the run's; a second cancel retires nothing.
    assert!(launches.owns(&client(7, instance)));
    let ack = launches
        .route(&client(7, instance), CHARACTER, 2_010)
        .unwrap();
    assert!(ack.accepted && ack.replies.is_empty() && !ack.changed);
    let repeat = launches
        .route(&client(6, instance), CHARACTER, 2_020)
        .unwrap();
    assert!(!repeat.accepted && repeat.replies.is_empty());
}
