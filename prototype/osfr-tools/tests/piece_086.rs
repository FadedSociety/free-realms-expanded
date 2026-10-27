//! Piece 086 (currency and economy audit) tests: recovered currency rows,
//! packet families and the reward-producer invariant audit.
//!
//! Evidence boundary, read before the assertions:
//!
//! ORIGINAL (supplied client / supplied server builds, hashes pinned by
//! the private recovery checker `piece_086.py`):
//! * `decompiled/client-ui/ScriptsBase.lua` data-source rows: PlayerInfo
//!   `NumCoins = 2` / `Wallet = 3` / `CoinCount = 16` / `SCCount = 17`,
//!   `_getCoinCount() = GetData(0, 2)`, `_getStationCashCount() =
//!   GetData(0, 3)`, PlayerInspect `Coins = 9` / `CoinBoost = 16` /
//!   `StationCash = 17`, `ItemDefinitionSelect_DS.CoinPurchasePrice = 25` /
//!   `CoinSellPrice = 26`, `MerchantSell_DS.Price = 27` /
//!   `MemberPurchasePrice = 30`.
//! * `decompiled/server/LibSOE/SOE/BaseClientUpdatePackets.cs`:
//!   `ClientUpdatePacketCoinCount = 19` (family `BaseClientUpdatePacket = 38`).
//! * `BaseCoinStorePackets.cs` family `BaseCoinStorePacket = 165`,
//!   `BaseTradePackets.cs` `TradeCoinCountPacket = 10` /
//!   `TradeUpdateCoinCountPacket = 11` in family `BaseTradePacket = 59`,
//!   `RewardBasePackets.cs` deferred/granted non-member rewards,
//!   `BasePackets.cs` `PacketClientNotifyCoinSpinAvailable = 171` and the
//!   in-game purchase wallet family `PacketBaseInGamePurchase = 66`.
//! * The recorded `PacketSendSelfToClient.json` sample stores `Coins` as an
//!   integer in the character profile.
//!
//! RECONSTRUCTED (labelled, not historical): the invariant set below and the
//! local producer amounts. Nothing here grants money, and no native economy
//! acceptance is claimed.
// The module is not registered in `src/lib.rs` (lead-owned). Including the exact
// source file keeps this target compiling against the real artifact. Integration
// step for the lead: register the module and drop this `#[path]` include.
#[path = "../src/restoration/piece_086.rs"]
#[allow(dead_code)]
mod piece_086;

use piece_086::{
    COIN_MAX, COIN_SPIN_NOTIFY_OPCODE, COIN_STORE_SECTION, COIN_UPDATE_OPCODE, COIN_UPDATE_SECTION,
    CoinLedger, CurrencyKind, CurrencyScope, EvidenceGrade, Flow, FlowDirection,
    IN_GAME_PURCHASE_SECTION, INSPECT_COIN_BOOST_ROW, INSPECT_COINS_ROW, INSPECT_STATION_CASH_ROW,
    ITEM_COIN_PURCHASE_PRICE_ROW, ITEM_COIN_SELL_PRICE_ROW, LedgerError, MERCHANT_MEMBER_PRICE_ROW,
    MERCHANT_SELL_PRICE_ROW, PLAYER_INFO_COIN_COUNT_ROW, PLAYER_INFO_NUM_COINS_ROW,
    PLAYER_INFO_SC_COUNT_ROW, PLAYER_INFO_WALLET_ROW, Producer, REWARD_SECTION, TRADE_SECTION,
    Violation, audit,
};

fn source(producer: Producer, instance: u64, amount: i64) -> Flow {
    Flow {
        producer,
        instance,
        amount,
    }
}

#[test]
fn recovered_coin_wire_constants_match_the_supplied_packet_enums() {
    // BaseClientUpdatePacket = 38, ClientUpdatePacketCoinCount = 19. The Rust
    // inventory update writes header(38, 19) followed by the i32 balance.
    assert_eq!(COIN_UPDATE_SECTION, 38);
    assert_eq!(COIN_UPDATE_OPCODE, 19);
    assert_eq!(COIN_STORE_SECTION, 165);
    assert_eq!(TRADE_SECTION, 59);
    assert_eq!(REWARD_SECTION, 50);
    assert_eq!(IN_GAME_PURCHASE_SECTION, 66);
    assert_eq!(COIN_SPIN_NOTIFY_OPCODE, 171);
    assert_eq!(COIN_MAX, i32::MAX as i64);
}

#[test]
fn recovered_client_rows_match_scriptsbase() {
    assert_eq!(PLAYER_INFO_NUM_COINS_ROW, 2);
    assert_eq!(PLAYER_INFO_WALLET_ROW, 3);
    assert_eq!(PLAYER_INFO_COIN_COUNT_ROW, 16);
    assert_eq!(PLAYER_INFO_SC_COUNT_ROW, 17);
    assert_eq!(INSPECT_COINS_ROW, 9);
    assert_eq!(INSPECT_COIN_BOOST_ROW, 16);
    assert_eq!(INSPECT_STATION_CASH_ROW, 17);
    assert_eq!(ITEM_COIN_PURCHASE_PRICE_ROW, 25);
    assert_eq!(ITEM_COIN_SELL_PRICE_ROW, 26);
    assert_eq!(MERCHANT_SELL_PRICE_ROW, 27);
    assert_eq!(MERCHANT_MEMBER_PRICE_ROW, 30);
}

#[test]
fn the_two_recovered_currencies_have_distinct_scopes_and_grants() {
    assert_eq!(CurrencyKind::Coins.scope(), CurrencyScope::Character);
    assert_eq!(
        CurrencyKind::StationCash.scope(),
        CurrencyScope::AccountWallet
    );
    // Gameplay sources grant coins. The recovered reward/quest/inventory paths
    // carry no Station Cash; the wallet is funded by the account purchase flow.
    assert!(CurrencyKind::Coins.gameplay_grantable());
    assert!(!CurrencyKind::StationCash.gameplay_grantable());
    assert_eq!(CurrencyKind::Coins.wire_max(), COIN_MAX);
}

#[test]
fn producers_classify_currency_direction_and_evidence() {
    use FlowDirection::{Sink, Source};
    let rows = [
        (Producer::QuestCompletion, CurrencyKind::Coins, Source),
        (Producer::CheckpointTrial, CurrencyKind::Coins, Source),
        (Producer::EncounterClear, CurrencyKind::Coins, Source),
        (Producer::OpeningStage, CurrencyKind::Coins, Source),
        (Producer::OpeningDiscovery, CurrencyKind::Coins, Source),
        (Producer::KartRace, CurrencyKind::Coins, Source),
        (Producer::CoinSpin, CurrencyKind::Coins, Source),
        (
            Producer::DeferredNonMemberReward,
            CurrencyKind::Coins,
            Source,
        ),
        (Producer::CoinShopPurchase, CurrencyKind::Coins, Sink),
        (Producer::CoinShopBuyback, CurrencyKind::Coins, Sink),
        (Producer::PaidRevive, CurrencyKind::Coins, Sink),
        (Producer::TradeGive, CurrencyKind::Coins, Sink),
        (Producer::TradeReceive, CurrencyKind::Coins, Source),
        (
            Producer::StationCashWalletFunding,
            CurrencyKind::StationCash,
            Source,
        ),
        (
            Producer::StationCashStorePurchase,
            CurrencyKind::StationCash,
            Sink,
        ),
    ];
    for (producer, currency, direction) in rows {
        assert_eq!(producer.currency(), currency, "{producer:?}");
        assert_eq!(producer.direction(), direction, "{producer:?}");
    }
    // The coin wheel and deferred non-member rewards are recovered protocol
    // facts; the local quest/opening payouts are reconstructed.
    assert_eq!(Producer::CoinSpin.evidence(), EvidenceGrade::Original);
    assert_eq!(
        Producer::DeferredNonMemberReward.evidence(),
        EvidenceGrade::Original
    );
    assert_eq!(
        Producer::StationCashWalletFunding.evidence(),
        EvidenceGrade::Original
    );
    assert_eq!(
        Producer::QuestCompletion.evidence(),
        EvidenceGrade::Reconstructed
    );
}

#[test]
fn audit_accepts_the_local_producers_once_each() {
    // Every flow reuses instance 1: the once-only key is namespaced by
    // producer, so the same captured instance id in one command is fine.
    let flows = [
        source(Producer::QuestCompletion, 1, 25),
        source(Producer::CheckpointTrial, 1, 10),
        source(Producer::EncounterClear, 1, 40),
        source(Producer::OpeningStage, 1, 15),
        source(Producer::OpeningDiscovery, 1, 5),
        source(Producer::KartRace, 1, 30),
        source(Producer::CoinSpin, 1, 100),
        source(Producer::DeferredNonMemberReward, 1, 12),
    ];
    let report = audit(&flows);
    assert!(report.ok(), "{:?}", report.violations);
    assert_eq!(report.coins_sourced, 237);
    assert_eq!(report.coins_sunk, 0);
    assert_eq!(report.station_cash_sourced, 0);
    assert_eq!(report.station_cash_sunk, 0);
}

#[test]
fn audit_rejects_a_replayed_source_instance() {
    let flows = [
        source(Producer::QuestCompletion, 9, 25),
        source(Producer::QuestCompletion, 9, 25),
    ];
    let report = audit(&flows);
    assert_eq!(
        report.violations,
        vec![(1, Violation::DuplicateSourceInstance)]
    );
}

#[test]
fn audit_rejects_negative_and_above_wire_range_amounts() {
    let negative = audit(&[source(Producer::QuestCompletion, 1, -5)]);
    assert_eq!(negative.violations, vec![(0, Violation::NegativeAmount)]);
    let oversized = audit(&[source(Producer::EncounterClear, 1, COIN_MAX + 1)]);
    assert_eq!(
        oversized.violations,
        vec![(0, Violation::AmountExceedsWireRange(COIN_MAX))]
    );
}

#[test]
fn audit_rejects_an_unkeyed_source() {
    let report = audit(&[source(Producer::OpeningStage, 0, 15)]);
    assert_eq!(report.violations, vec![(0, Violation::ZeroInstance)]);
}

#[test]
fn account_wallet_funding_is_the_only_station_cash_source() {
    // The type system carries the recovered split: no gameplay producer can
    // create or spend Station Cash, and no recovered wallet flow moves coins.
    let sources: Vec<_> = Producer::ALL
        .into_iter()
        .filter(|p| p.currency() == CurrencyKind::StationCash)
        .filter(|p| p.direction() == FlowDirection::Source)
        .collect();
    assert_eq!(sources, vec![Producer::StationCashWalletFunding]);
    for producer in Producer::ALL {
        if producer.currency() == CurrencyKind::StationCash {
            assert_ne!(producer.evidence(), EvidenceGrade::Reconstructed);
        }
    }

    // Accounting keeps the two currencies apart.
    let mixed = audit(&[
        source(Producer::StationCashWalletFunding, 1, 499),
        source(Producer::CoinShopPurchase, 2, 100),
    ]);
    assert!(mixed.ok(), "{:?}", mixed.violations);
    assert_eq!(mixed.station_cash_sourced, 499);
    assert_eq!(mixed.coins_sunk, 100);
    assert_eq!(mixed.coins_sourced, 0);
}

#[test]
fn coin_ledger_refuses_overflow_and_overdraw() {
    let mut ledger = CoinLedger::new(COIN_MAX - 1);
    assert_eq!(ledger.credit(1).unwrap(), COIN_MAX);
    assert_eq!(ledger.credit(1).unwrap_err(), LedgerError::WouldOverflow);
    assert_eq!(ledger.balance(), COIN_MAX);

    assert_eq!(ledger.debit(COIN_MAX).unwrap(), 0);
    assert_eq!(ledger.debit(1).unwrap_err(), LedgerError::InsufficientFunds);
    assert_eq!(ledger.balance(), 0);

    assert_eq!(
        ledger.credit(0).unwrap_err(),
        LedgerError::NonPositiveAmount
    );
    assert_eq!(
        ledger.debit(-1).unwrap_err(),
        LedgerError::NonPositiveAmount
    );
}
