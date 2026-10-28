//! T765: a Trading Card Game row with a launch block is admitted and runs the client's type-16
//! handshake. T795 (client order: `ClientTradingCardGameManager` update `00ad4f50`, dispatcher
//! `009bf890`): the 39/16 offer carries the card asset list, the client's 39/37 ticket request is
//! answered while the run is offered with 39/0x28 (a one-time launch ticket the card login
//! redeems once), the Start click 39/5 is answered with 39/17 (no Flash launch), 39/6 leaves.
//! Packets are shaped like the client's (`009bdc60`: sub, instance id, -1, -1).
//! T782: the client's close of the start screen (26/42, then 39/6 and 39/7) through the Gateway.
use osfr_tools::{
    account_store::AccountRole,
    activity_portal::{Catalogue, Refusal, Viewer},
    card_game::{self, Content, FIRST_INSTANCE_ID, Launches},
    gateway::{Gateway, GatewayOutcome},
};
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

#[path = "support/card_packets.rs"]
mod card_packets;
#[path = "common/content_copy.rs"]
#[allow(dead_code)] // this suite only overlays and links; the helper also has `copy_tree`
mod content_copy;
#[path = "common/field_battle_fixture.rs"]
mod field_battle_fixture;
#[path = "support/json.rs"]
mod json;
use card_packets::ticket_of;
use field_battle_fixture::{Fixture, bundle, content_dir, portal_play, send, unframe, wrap};

static FIXTURE_SERIAL: AtomicU64 = AtomicU64::new(0);

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
    json::read(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/card_game.json"))
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
    load(&launch_json(ttl_ms), &ttl_ms.to_string())
}

fn load(content: &Value, tag: &str) -> std::io::Result<Content> {
    // The serial keeps parallel tests of this binary (and their remove_file) apart.
    let path = std::env::temp_dir().join(format!(
        "osfr-card-launch-{}-{}-{tag}.json",
        std::process::id(),
        FIXTURE_SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, serde_json::to_vec(content).unwrap()).unwrap();
    let loaded = Content::load(&path);
    let _ = fs::remove_file(&path);
    loaded
}

/// A production-shaped session root whose card file is `card`: the runtime content, the bundled
/// content over it, and `card`.
fn card_fixture(card: &Value) -> Fixture {
    let mut fixture = Fixture::new();
    let root = fixture.root.join("content");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("card_game.json"),
        serde_json::to_vec(card).unwrap(),
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

/// The full fixture loads the private round70 content base and gateway reference; a public
/// checkout has neither and skips with this one message. `OSFR_ROUND70_CONTENT` and
/// `OSFR_GATEWAY_REFERENCE` point a checkout at its own package.
fn private_fixture_available() -> bool {
    if field_battle_fixture::available() {
        return true;
    }
    eprintln!("skipped: private round70 content and gateway reference are absent");
    false
}

/// `card_fixture` with a launch block; `None` when the private inputs are absent (the wire and
/// run rules are still covered by the `with_launch`/`Launches` tests).
fn launch_fixture() -> Option<Fixture> {
    private_fixture_available().then(|| card_fixture(&launch_json(120_000)))
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

/// The card asset list a 39/16 offer names. Layout (reader `009bdd70`): the 15-byte header, six
/// i32 (name, icon, description, difficulty, profile, type), the members byte, three 69-byte
/// reward bundles, the goal count (0 goals here), five flag bytes, then the preload string
/// (i32 length, bytes).
fn offer_preload(offer: &[u8]) -> String {
    let at = 15 + 6 * 4 + 1 + 3 * 69 + 4 + 5;
    assert_eq!(i32_at(offer, 15 + 5 * 4), 16, "the card game type");
    let len = i32_at(offer, at) as usize;
    String::from_utf8(offer[at + 4..at + 4 + len].to_vec()).unwrap()
}

/// The five flag bytes of a 39/16 offer (`+612..+616` of the client's info struct).
fn offer_flags(offer: &[u8]) -> [u8; 5] {
    let at = 15 + 6 * 4 + 1 + 3 * 69 + 4;
    offer[at..at + 5].try_into().unwrap()
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

/// T813: the lobby a card login enters lives in the launch block (R values), with sanity bounds.
#[test]
fn a_launch_block_names_the_lobby_within_bounds() {
    let lobby = |group: i32, lobby_type: i32, role: i32| {
        let mut content = launch_json(120_000);
        content["launch"]["lobby"] =
            json!({"group_id": group, "lobby_type": lobby_type, "role": role});
        load(&content, &format!("lobby-{group}-{lobby_type}-{role}"))
    };
    let loaded = lobby(1, 1, 1).unwrap();
    assert_eq!(
        loaded.launch().unwrap().lobby,
        Some(card_game::Lobby {
            group_id: 1,
            lobby_type: 1,
            role: 1,
            collection: Vec::new()
        })
    );
    assert_eq!(with_launch(120_000).unwrap().launch().unwrap().lobby, None);
    for (group, lobby_type, role) in [(0, 1, 1), (1, 1, 0), (1, 1, 6), (1, -1, 1), (1, 256, 1)] {
        assert!(
            lobby(group, lobby_type, role).is_err(),
            "{group} {lobby_type} {role}"
        );
    }
}

/// T825: the starter collection lives in the lobby block (R ids), with sanity bounds.
#[test]
fn a_lobby_names_its_starter_collection_within_bounds() {
    let starter = |cards: Value, tag: &str| {
        let mut content = launch_json(120_000);
        content["launch"]["lobby"] =
            json!({"group_id": 1, "lobby_type": 1, "role": 1, "collection": cards});
        load(&content, &format!("starter-{tag}"))
    };
    let loaded = starter(json!([{"archetype_id": 100009946, "copies": 2}]), "good").unwrap();
    assert_eq!(
        loaded.launch().unwrap().lobby.as_ref().unwrap().collection,
        [card_game::StarterCard {
            archetype_id: 100009946,
            copies: 2
        }]
    );
    let bad = [
        json!([{"archetype_id": 0, "copies": 1}]),
        json!([{"archetype_id": 100009946, "copies": 0}]),
        json!([{"archetype_id": 100009946, "copies": 17}]),
        json!([{"archetype_id": 100009946, "copies": 1}, {"archetype_id": 100009946, "copies": 1}]),
    ];
    for (index, cards) in bad.into_iter().enumerate() {
        assert!(starter(cards, &format!("bad-{index}")).is_err(), "{index}");
    }
    // The 100-archetype cap is the server's: 100 loads, one more is refused.
    let archetypes = |count: i32| {
        json!(
            (1..=count)
                .map(|n| json!({"archetype_id": 100_000_000 + n, "copies": 1}))
                .collect::<Vec<_>>()
        )
    };
    assert!(starter(archetypes(100), "archetypes-100").is_ok());
    assert!(starter(archetypes(101), "archetypes-101").is_err());
}

#[test]
fn the_type_16_handshake_sends_a_one_time_ticket_in_39_28() {
    let cards = with_launch(120_000).unwrap();
    let row = cards.row(TCG_ROW).unwrap();
    let mut launches = Launches::new(cards.launch().unwrap()).unwrap();
    assert!(!launches.is_active());

    // 39/16 names the card art list: the client starts its loader only for a non-empty one
    // (`009beb70`), and its card manager itself loads `TradingCardAssets.txt` (`00ad4f50` state
    // 1, string `00ad4f9e`). The narration-audio list (`TradingCardGameAssets.txt`) carries no art.
    let offer = launches.begin(row).unwrap();
    assert_eq!(subs(&offer.replies), vec![(39, 16)]);
    assert_eq!(offer_preload(&offer.replies[0]), "TradingCardAssets.txt");
    assert_eq!(
        offer.replies[0].len(),
        281 + "TradingCardAssets.txt".len(),
        "an empty list is 281 bytes"
    );
    let instance = i32_at(&offer.replies[0], 3);
    assert_eq!(instance, FIRST_INSTANCE_ID);
    assert!(launches.is_active());

    // The Start click cannot beat the ticket: 39/17 starts the card DLL with the ticket 39/0x28
    // stored (`00ad47f0`), so 39/5 is ignored until one was sent.
    let early = launches
        .route(&client(5, instance), CHARACTER, 1_000)
        .unwrap();
    assert!(!early.accepted && !early.changed && early.replies.is_empty());

    // Another instance's request is ignored; the run's own, while it is still offered, gets
    // 39/0x28.
    let stray = launches
        .route(&client(37, instance + 1), CHARACTER, 1_100)
        .unwrap();
    assert!(!stray.accepted && stray.replies.is_empty());
    let asked = launches
        .route(&client(37, instance), CHARACTER, 1_200)
        .unwrap();
    assert!(asked.accepted && !asked.changed, "a ticket marks no save");
    assert_eq!(subs(&asked.replies), vec![(39, 0x28)]);
    let packet = &asked.replies[0];
    assert_eq!(
        (i32_at(packet, 3), i32_at(packet, 7), i32_at(packet, 11)),
        (instance, -1, row.minigame_id as i32)
    );
    let first = ticket_of(packet);
    assert_ne!(first, 0);

    // A repeated request (a second launch in one client session asks again) gets a fresh ticket
    // that replaces the character's earlier one.
    let again = launches
        .route(&client(37, instance), CHARACTER, 2_000)
        .unwrap();
    let second = ticket_of(&again.replies[0]);
    assert_ne!(second, first);
    let character = CHARACTER.to_string();
    assert_eq!(
        launches.redeem(&first.to_string(), &character, 2_050),
        None,
        "replaced by the second ticket"
    );

    // Start, now that a ticket was sent: 39/17 alone, the client hosts the card DLL itself.
    let start = launches
        .route(&client(5, instance), CHARACTER, 2_100)
        .unwrap();
    assert_eq!(subs(&start.replies), vec![(39, 17)]);

    // The card login redeems a ticket once, for its own character; a wrong id is refused and
    // (T764 fix) does not spend the ticket. The retail 411's id is empty (T804).
    assert_eq!(
        launches.redeem(&second.to_string(), "1", 2_200),
        Some(Err("character_mismatch"))
    );
    assert_eq!(
        launches.redeem(&second.to_string(), &character, 2_250),
        Some(Ok(CHARACTER))
    );
    assert_eq!(
        launches.redeem(&second.to_string(), &character, 2_275),
        None,
        "single use"
    );
    let spent = launches
        .route(&client(37, instance), CHARACTER, 2_300)
        .unwrap();
    assert_eq!(
        subs(&spent.replies),
        vec![(39, 0x28)],
        "still answered once started"
    );
    let last = ticket_of(&spent.replies[0]);
    assert_eq!(
        launches.redeem(&last.to_string(), &character, 2_500),
        Some(Ok(CHARACTER))
    );
    assert_eq!(launches.redeem(&last.to_string(), &character, 2_600), None);

    // A ticket expires after the launch block's lifetime.
    let late = launches
        .route(&client(37, instance), CHARACTER, 3_000)
        .unwrap();
    let late = ticket_of(&late.replies[0]);
    assert_eq!(
        launches.redeem(&late.to_string(), &character, 3_000 + 120_000),
        Some(Err("expired"))
    );

    // Leaving the card window ends the run (39/19); later requests are ignored.
    let left = launches
        .route(&client(6, instance), CHARACTER, 4_000)
        .unwrap();
    assert_eq!(subs(&left.replies), vec![(39, 19)]);
    assert!(!launches.is_active());
    let after = launches
        .route(&client(37, instance), CHARACTER, 4_100)
        .unwrap();
    assert!(after.replies.is_empty());
}

/// The whole order through a production-shaped session, as the client sends it: Play, the
/// ticket request while the run is offered, the Start click, the window close. The refused early
/// Start leaves the run where it was.
#[test]
fn the_card_start_order_runs_end_to_end_through_the_gateway() {
    let Some(fixture) = launch_fixture() else {
        return;
    };
    let mut gateway = player(&fixture, 5_064);
    gateway.set_trace_enabled(true);

    let offered = bodies(&send(&mut gateway, &portal_play(TCG_ROW), 1_000));
    let offer = offered
        .iter()
        .find(|p| p.starts_with(&[39, 0, 16]))
        .expect("39/16 offer");
    let instance = i32_at(offer, 3);
    assert_eq!(offer_preload(offer), "TradingCardAssets.txt");
    // The flags stay the client default: `+616` = 1 (`009bf190` reads it as game byte `+2312`)
    // makes the 39/19 on leaving take the standard end-of-game path instead of the Lua
    // `:QuitGameWithNoEndGameScreen` re-entry that overflows the client (T856).
    assert_eq!(offer_flags(offer), [1, 1, 1, 1, 1]);

    // Start before the ticket: nothing is sent and nothing is saved.
    let before = gateway.snapshot();
    let early = send(&mut gateway, &client(5, instance), 1_500);
    assert!(early.replies.is_empty() && !early.state_changed && !early.gameplay_changed);
    assert_eq!(gateway.snapshot(), before);

    // The card manager's 39/37, while the run is offered, gets 39/0x28.
    let asked = send(&mut gateway, &client(37, instance), 2_000);
    assert_eq!(asked.trace[0].outcome, "handled", "{:?}", asked.trace);
    let ticket = match bodies(&asked).as_slice() {
        [packet] => ticket_of(packet),
        other => panic!("one 39/0x28 expected, got {:?}", subs(other)),
    };
    assert_eq!(gateway.snapshot(), before, "a ticket marks no save");

    // 39/5 is answered with 39/17 alone; the run is still the player's (busy).
    let started = bodies(&send(&mut gateway, &client(5, instance), 2_500));
    assert_eq!(subs(&started), vec![(39, 17)]);
    let busy = send(&mut gateway, &portal_play(TCG_ROW), 2_600);
    assert!(field_battle_fixture::chat_text(&busy).contains("Finish what you are doing"));

    // The card login redeems the ticket in this session, once.
    let character = 5_064.to_string();
    let service = gateway.gameplay_mut().unwrap();
    assert_eq!(
        service.redeem_card_ticket(&ticket.to_string(), &character, 3_000),
        Some(Ok(5_064))
    );
    assert_eq!(
        service.redeem_card_ticket(&ticket.to_string(), &character, 3_100),
        None
    );

    // Closing the card window: 39/6 -> 39/19, then the client's 39/7 ack.
    let left = bodies(&send(&mut gateway, &client(6, instance), 4_000));
    assert_eq!(subs(&left), vec![(39, 19)]);
    let ack = send(&mut gateway, &client(7, instance), 4_010);
    assert!(ack.replies.is_empty());
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
/// Close, then 39/6 (CancelMiniGame) and 39/7, the order three native plays showed. The native
/// cancel can put 39/6 and 39/7 in one combined datagram; this test sends them one at a time and
/// `a_combined_datagram_...` sends the pair in one bundle.
#[test]
fn the_clients_close_of_the_start_screen_retires_the_offered_run_once() {
    // The bytes the native client sent for the first run (instance 61,000,001).
    let captured = |sub: u8| [vec![0x27, 0, sub, 0x41, 0xc9, 0xa2, 3], vec![0xff; 8]].concat();
    assert_eq!(client(6, FIRST_INSTANCE_ID), captured(6));
    assert_eq!(client(7, FIRST_INSTANCE_ID), captured(7));

    let Some(fixture) = launch_fixture() else {
        return;
    };
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

/// T782 review: the native cancel can put 39/6 and 39/7 in one combined datagram. The Gateway
/// decodes every child before committing, so the pair retires the run once; a malformed child
/// (here a 16-byte 39/6) rejects the whole bundle before the staged cancel lands, leaving the
/// run offered so the player can cancel for real.
#[test]
fn a_combined_datagram_retires_the_run_and_a_malformed_child_changes_nothing() {
    let Some(fixture) = launch_fixture() else {
        return;
    };
    let offer_of = |out: &GatewayOutcome| {
        bodies(out)
            .into_iter()
            .find(|p| p.starts_with(&[39, 0, 16]))
    };
    let mut gateway = player(&fixture, 5_065);
    gateway.set_trace_enabled(true);
    let offer = offer_of(&send(&mut gateway, &portal_play(TCG_ROW), 1_000)).expect("39/16 offer");
    let instance = i32_at(&offer, 3);
    let before = gateway.snapshot();

    let pair = bundle(&[wrap(&client(6, instance)), wrap(&client(7, instance))]);
    let out = gateway.handle_at(&pair, 2_000).unwrap();
    assert_eq!(subs(&bodies(&out)), vec![(39, 19)]);
    assert_eq!(
        out.trace.iter().filter(|event| event.combined).count(),
        2,
        "both children are traced as part of one bundle: {:?}",
        out.trace
    );
    assert_eq!(gateway.snapshot(), before, "the pair marks no save");
    let again = offer_of(&send(&mut gateway, &portal_play(TCG_ROW), 2_500)).expect("next offer");
    assert_eq!(i32_at(&again, 3), FIRST_INSTANCE_ID + 1, "the run retired");

    // A malformed child refuses the whole bundle; the run is still offered and cancellable.
    let mut broken = player(&fixture, 5_066);
    let offer = offer_of(&send(&mut broken, &portal_play(TCG_ROW), 3_000)).expect("39/16 offer");
    let instance = i32_at(&offer, 3);
    let before = broken.snapshot();
    let malformed = [client(6, instance), vec![0]].concat();
    let pair = bundle(&[wrap(&client(6, instance)), wrap(&malformed)]);
    assert!(broken.handle_at(&pair, 3_100).is_err());
    assert_eq!(broken.snapshot(), before, "the staged cancel is discarded");
    let busy = send(&mut broken, &portal_play(TCG_ROW), 3_200);
    assert!(
        field_battle_fixture::chat_text(&busy).contains("Finish what you are doing"),
        "the run is still offered"
    );
    let good = bundle(&[wrap(&client(6, instance)), wrap(&client(7, instance))]);
    let out = broken.handle_at(&good, 3_300).unwrap();
    assert_eq!(subs(&bodies(&out)), vec![(39, 19)]);
}

/// The cancel pair on its own (no 26/42 before it): 39/6 retires the offered run with the
/// state-remove reply and 39/7 is understood, not traced as an unknown message.
#[test]
fn the_cancel_pair_alone_retires_the_offered_run_and_is_understood() {
    let Some(fixture) = launch_fixture() else {
        return;
    };
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
    // The run's own subs for another game, a sub the run does not take, and wrong lengths:
    // ignored or refused while offered, and again once started.
    let other_game = |sub| {
        let mut packet = client(sub, instance);
        packet[11..15].copy_from_slice(&500i32.to_le_bytes());
        packet
    };
    let ignored = [
        other_game(5),
        other_game(6),
        other_game(37),
        client(46, instance),
        client(14, instance),
    ];
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
            // The ignored requests above asked for no ticket, so the Start click still waits.
            let early = launches
                .route(&client(5, instance), CHARACTER, 1_050)
                .unwrap();
            assert!(!early.accepted && early.replies.is_empty());
            launches
                .route(&client(37, instance), CHARACTER, 1_075)
                .unwrap();
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

/// The card asset list is data with a default, and an empty one is refused: the client starts
/// its loader only for a non-empty list, so an empty one leaves the start screen on Loading.
#[test]
fn the_card_asset_list_is_data_and_never_empty() {
    // T849: the default is the client's own card art list, not the narration-audio list
    // (`Assets_009.pack` entry 3,843) that the old name happened to be. The client's card
    // manager loads `TradingCardAssets.txt` (`00ad4f50` state 1, string `00ad4f9e`).
    assert_eq!(card_game::PRELOAD_FILE, "TradingCardAssets.txt");

    let named = |name: &str| {
        let mut content = launch_json(60_000);
        content["launch"]["preload_file"] = json!(name);
        load(&content, &format!("preload-{}", name.len()))
    };
    let custom = named("OtherCardAssets.txt").unwrap();
    let mut launches = Launches::new(custom.launch().unwrap()).unwrap();
    let offer = launches.begin(custom.row(TCG_ROW).unwrap()).unwrap();
    assert_eq!(offer_preload(&offer.replies[0]), "OtherCardAssets.txt");
    for bad in ["", &"x".repeat(65), "Caf\u{e9}.txt"] {
        let error = named(bad).unwrap_err().to_string();
        assert!(error.contains("asset list"), "{bad:?}: {error}");
    }
}

/// T798: the 82 (`PacketUpdateClientSessionData`) a login sends. Its ticket, after checking the
/// layout around it (reader `008e4e40`: the opcode, the ticket as the first of four counted
/// strings, then the station name and two realms, all empty here); `None` when the login sent
/// none. A login never sends two.
fn session_ticket(login: &GatewayOutcome) -> Option<String> {
    let sent: Vec<_> = bodies(login)
        .into_iter()
        .filter(|body| body.starts_with(&[82, 0]))
        .collect();
    assert!(sent.len() <= 1, "one 82 per login, got {}", sent.len());
    sent.first().map(|body| {
        assert_eq!((body.len(), i32_at(body, 2)), (2 + 4 + 32 + 3 * 4, 32));
        assert_eq!(body[38..], [0; 12], "station and realms stay empty");
        String::from_utf8(body[6..38].to_vec()).unwrap()
    })
}

/// T798: the card window logs itself in only when the client holds a session ticket, and the
/// client holds one only from the 82 (`009485f9`). A login with a card launch block sends it
/// last, once, as 32 fresh random hex characters; a second login gets a different one.
#[test]
fn a_card_login_sends_one_fresh_session_ticket_last() {
    let Some(fixture) = launch_fixture() else {
        return;
    };
    let (mut first, login) = fixture.login_session(LAKESHORE, 5_098);
    let ticket = session_ticket(&login).expect("a card login sends the 82");
    assert!(
        ticket
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    );
    assert!(bodies(&login).last().unwrap().starts_with(&[82, 0]));
    // Once per login: a repeated login message on the same session sends nothing more.
    assert!(fixture.login(&mut first).replies.is_empty());

    let (_, other) = fixture.login_session(LAKESHORE, 5_099);
    assert_ne!(session_ticket(&other).unwrap(), ticket);
}

/// T798: without a launch block the card windows stay closed, so a login is what it was: no 82.
#[test]
fn a_login_without_a_card_launch_block_sends_no_session_ticket() {
    if !private_fixture_available() {
        return;
    }
    let no_card_file = Fixture::new();
    let shipped_card_file = card_fixture(&shipped());
    for (fixture, name) in [
        (&no_card_file, "no card file"),
        (&shipped_card_file, "no launch"),
    ] {
        let (_, login) = fixture.login_session(LAKESHORE, 5_097);
        assert_eq!(session_ticket(&login), None, "{name}");
    }
    // The launch block is the only difference: the same login gains the 82 at its end.
    let (_, plain) = shipped_card_file.login_session(LAKESHORE, 5_097);
    let (_, card) = card_fixture(&launch_json(120_000)).login_session(LAKESHORE, 5_097);
    assert_eq!(card.replies.len(), plain.replies.len() + 1);
    assert_eq!(card.replies[..plain.replies.len()], plain.replies[..]);
}

/// T798: the session ticket is not a credential. The card login is authorised by the one-time
/// launch ticket only (a non-empty character id must name the owner, and the retail 411 carries
/// none, T804): the 82's ticket redeems nothing, with the right character id and while a launch
/// ticket is outstanding, and the launch ticket still works afterwards. A trace never shows the
/// 82's body as hex.
#[test]
fn the_session_ticket_authorises_nothing_and_is_never_traced() {
    let Some(fixture) = launch_fixture() else {
        return;
    };
    let (mut gateway, login) = fixture.login_session(LAKESHORE, 5_096);
    let ticket = session_ticket(&login).unwrap();
    let packet = login
        .replies
        .iter()
        .find(|reply| unframe(reply).starts_with(&[82, 0]))
        .unwrap();
    let summary = osfr_tools::trace::protocol_summary(packet);
    assert!(summary.sensitive);
    assert_eq!(
        osfr_tools::trace::hex_prefix(summary.body, summary.sensitive),
        None
    );

    send(&mut gateway, &13_u16.to_le_bytes(), 100);
    send(&mut gateway, &10_u16.to_le_bytes(), 200);
    let offered = bodies(&send(&mut gateway, &portal_play(TCG_ROW), 1_000));
    let instance = i32_at(
        offered
            .iter()
            .find(|p| p.starts_with(&[39, 0, 16]))
            .unwrap(),
        3,
    );
    let launch = ticket_of(&bodies(&send(&mut gateway, &client(37, instance), 2_000))[0]);

    let character = 5_096.to_string();
    let service = gateway.gameplay_mut().unwrap();
    assert_eq!(
        service.redeem_card_ticket(&ticket, &character, 3_000),
        Some(Err("ticket_not_decimal"))
    );
    assert_eq!(
        service.redeem_card_ticket(&launch.to_string(), &character, 3_100),
        Some(Ok(5_096))
    );
}
