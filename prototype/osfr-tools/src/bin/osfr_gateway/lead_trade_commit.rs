use super::*;
use osfr_tools::inventory::InventoryState;
use osfr_tools::trade::{SettlementSide, TradeSettlement};

fn scratch(name: &str) -> PathBuf {
    let path =
        std::env::temp_dir().join(format!("osfr-trade-commit-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

fn party(scratch: &Path, id: u64, guid: u64) -> Session {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
    let options = Options {
        bind: "127.0.0.1:0".parse().unwrap(),
        reference: root.join("references/rust-gateway-reference"),
        state: scratch.join(format!("{guid}.json")),
        content: Some(root.join("references/rust-opening-cooking-original-assets")),
        identity: None,
        account_store: None,
        gateway_ticket: login::DEFAULT_GATEWAY_TICKET.into(),
        handoff_key: Vec::new(),
        seconds: 1,
        dev_tools: false,
        studio_port: 0,
        client_logs: None,
        terrain: None,
        world: None,
        stock_text: None,
        allow_lan: false,
        security_log: None,
        group_invite_ttl_ms: osfr_tools::social_lists::DEFAULT_GROUP_INVITE_TTL_MS,
    };
    let mut session = Session::new(
        SessionId(id),
        format!("127.0.0.1:{}", 20_000 + id).parse().unwrap(),
        SessionRequest {
            id: id as u32,
            crc_length: 3,
            buffer_size: 512,
        },
        &options,
        CRC_SEED,
        None,
        None,
    )
    .unwrap();
    session.character_guid = guid;
    session.selected_character = true;
    session.state_path = options.state;
    session
}

fn with_coins(session: &mut Session, coins: u32) -> InventoryState {
    let mut state = session.gateway.inventory_state().unwrap().clone();
    state.coins = coins;
    session
        .gateway
        .adopt_trade_inventory(state.clone())
        .unwrap();
    state
}

/// A gives B 10 coins.
fn settlement(a: &InventoryState, b: &InventoryState) -> TradeSettlement {
    let side = |guid, before: &InventoryState, delta: i64| {
        let mut after = before.clone();
        after.coins = (i64::from(before.coins) + delta) as u32;
        SettlementSide {
            guid,
            before: before.clone(),
            after,
            gave: Vec::new(),
            received: Vec::new(),
        }
    };
    TradeSettlement {
        session_id: 7,
        sides: [side(101, a, -10), side(202, b, 10)],
    }
}

fn saved_coins(path: &Path) -> u64 {
    let document: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    document["gameplay"]["inventory"]["coins"].as_u64().unwrap()
}

fn leftovers(directory: &Path) -> Vec<String> {
    fs::read_dir(directory)
        .unwrap()
        .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
        .filter(|name| name.starts_with("trade-"))
        .collect()
}

#[test]
fn a_settled_trade_moves_coins_in_both_saves_and_both_live_sessions() {
    let dir = scratch("ok");
    let mut a = party(&dir, 1, 101);
    let mut b = party(&dir, 2, 202);
    let before_a = with_coins(&mut a, 25);
    let before_b = with_coins(&mut b, 5);
    let key = SessionKey {
        peer: b.peer,
        connection_id: b.request.id,
    };
    let mut taken = vec![(key, b)];
    let deliveries = commit_trade_parties(&settlement(&before_a, &before_b), &mut a, &mut taken)
        .unwrap()
        .expect("a matching trade commits");
    assert_eq!(a.gateway.inventory_state().unwrap().coins, 15);
    assert_eq!(taken[0].1.gateway.inventory_state().unwrap().coins, 15);
    assert_eq!(saved_coins(&a.state_path), 15);
    assert_eq!(saved_coins(&taken[0].1.state_path), 15);
    // Each player is told their new balance (38/19); no journal is left behind.
    for id in [a.id, taken[0].1.id] {
        assert!(deliveries.iter().any(|delivery| delivery.recipient == id
            && delivery.bytes.windows(4).any(|w| w == [38, 0, 19, 0])));
    }
    assert!(leftovers(&dir).is_empty(), "{:?}", leftovers(&dir));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn a_stale_trade_moves_nothing_and_writes_nothing() {
    let dir = scratch("stale");
    let mut a = party(&dir, 3, 101);
    let mut b = party(&dir, 4, 202);
    let before_a = with_coins(&mut a, 25);
    let before_b = with_coins(&mut b, 5);
    // A spends coins after the offers were validated: the settlement is stale.
    with_coins(&mut a, 24);
    let saves =
        |a: &Session, b: &Session| [&a.state_path, &b.state_path].map(|path| fs::read(path).ok());
    let before_saves = saves(&a, &b);
    let key = SessionKey {
        peer: b.peer,
        connection_id: b.request.id,
    };
    let mut taken = vec![(key, b)];
    let result =
        commit_trade_parties(&settlement(&before_a, &before_b), &mut a, &mut taken).unwrap();
    assert!(result.is_none(), "a stale trade must be refused");
    assert_eq!(a.gateway.inventory_state().unwrap().coins, 24);
    assert_eq!(taken[0].1.gateway.inventory_state().unwrap().coins, 5);
    assert_eq!(saves(&a, &taken[0].1), before_saves);
    assert!(leftovers(&dir).is_empty());
    // Canary: the same settlement against the state it was made for does commit.
    with_coins(&mut a, 25);
    assert!(
        commit_trade_parties(&settlement(&before_a, &before_b), &mut a, &mut taken)
            .unwrap()
            .is_some()
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn an_interrupted_trade_is_finished_from_its_journal_at_start() {
    let dir = scratch("recover");
    let mut a = party(&dir, 5, 101);
    let mut b = party(&dir, 6, 202);
    with_coins(&mut a, 15);
    with_coins(&mut b, 15);
    // The crash happened after the journal and before either save.
    let journal: Vec<TradeJournalEntry> = [&a, &b]
        .iter()
        .map(|party| TradeJournalEntry {
            path: party.state_path.clone(),
            snapshot: party.gateway.snapshot(),
        })
        .collect();
    fs::write(
        dir.join(format!("trade-7-1.{TRADE_JOURNAL_EXTENSION}")),
        serde_json::to_vec(&journal).unwrap(),
    )
    .unwrap();
    // A half-written journal means the trade never started writing saves.
    fs::write(dir.join("trade-8-2.tmp"), b"{ half").unwrap();
    assert_eq!(recover_trade_journals(&dir).unwrap(), 1);
    assert_eq!(saved_coins(&a.state_path), 15);
    assert_eq!(saved_coins(&b.state_path), 15);
    assert!(leftovers(&dir).is_empty(), "{:?}", leftovers(&dir));
    // Running recovery again is harmless.
    assert_eq!(recover_trade_journals(&dir).unwrap(), 0);
    fs::remove_dir_all(dir).unwrap();
}
