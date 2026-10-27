//! T765: a Trading Card Game row with a launch block is admitted and runs the client's type-16
//! handshake: 39/16 offer, 39/5 -> 39/17 (no Flash launch), 39/37 -> 39/0x28 with a one-time
//! launch ticket the card login redeems once, 39/6 leave. Packets are shaped like the client's
//! (`009bdc60`: sub, instance id, -1, -1).
use osfr_tools::{
    activity_portal::{Catalogue, Refusal, Viewer},
    card_game::{self, Content, FIRST_INSTANCE_ID, Launches},
};
use serde_json::{Value, json};
use std::{fs, path::PathBuf, sync::Arc};

const TCG_ROW: u32 = 490;
const TREASURE_WAR_ROW: u32 = 1078;
const CHARACTER: u64 = 7_000_123;
const VIEWER: Viewer = Viewer {
    level: 1,
    combat_job: true,
};

fn shipped() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/card_game.json");
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

/// The shipped card rows plus a launch block, loaded through the real loader.
fn with_launch(ttl_ms: u64) -> std::io::Result<Content> {
    let mut content = shipped();
    content["launch"] = json!({
        "ticket_ttl_ms": ttl_ms,
        "source": [{"kind": "reconstructed", "evidence": "T765 test"}]
    });
    let path = std::env::temp_dir().join(format!(
        "osfr-card-launch-{}-{ttl_ms}.json",
        std::process::id()
    ));
    fs::write(&path, serde_json::to_vec(&content).unwrap()).unwrap();
    let loaded = Content::load(&path);
    let _ = fs::remove_file(&path);
    loaded
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
