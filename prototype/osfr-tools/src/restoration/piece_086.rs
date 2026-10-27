//! Piece 086: currency and economy audit.
//!
//! This module is the typed, validated view of the two recovered currencies and
//! the invariant checker the lead can run over reward producers. It grants
//! nothing: no coins, no Station Cash, no items and no native acceptance.
//!
//! ORIGINAL facts (supplied client and supplied server builds; every path and
//! hash is pinned in `notes/Research/Pieces/086 Currency and economy audit.md`
//! and re-verified by the private recovery checker `piece_086.py`):
//!
//! * Coins are the character currency. `ScriptsBase.lua` stores the count in
//!   `PlayerInfo_DS.NumCoins = 2` (`_getCoinCount()` reads row 2),
//!   `PlayerInfo_DS.CoinCount = 16` and `PlayerInspectDS.Coins = 9`. The wire
//!   update is `ClientUpdatePacketCoinCount = 19` inside
//!   `BaseClientUpdatePacket = 38`, carrying one 32-bit value - exactly the
//!   header the local inventory already writes (`header(38, 19)` + `i32`), and
//!   the login profile writes `ClientPcData.Coins` as an `int` after
//!   `LastName`.
//! * Station Cash is the account wallet. `PlayerInfo_DS.Wallet = 3`
//!   (`_getStationCashCount()` reads row 3), `PlayerInfo_DS.SCCount = 17` and
//!   `PlayerInspectDS.StationCash = 17`. The wallet travels in the
//!   `PacketBaseInGamePurchase = 66` family
//!   (`PacketInGamePurchaseWalletInfoRequest/Response`, Station Cash product
//!   lists), funded by the account credit-card flow - not by any recovered
//!   gameplay reward.
//! * Coin flows in the recovered protocol: `BaseCoinStorePacket = 165`
//!   (Coin Shop buy/sell/buyback/transaction history), `BaseTradePacket = 59`
//!   with `TradeCoinCountPacket = 10` / `TradeUpdateCoinCountPacket = 11`,
//!   `RewardBasePacket = 50` with the deferred/granted non-member rewards, and
//!   `PacketClientNotifyCoinSpinAvailable = 171` for the coin wheel.
//! * Price rows: `ItemDefinitionSelect_DS.CoinPurchasePrice = 25`,
//!   `CoinSellPrice = 26`, `MerchantSell_DS.Price = 27`,
//!   `MemberPurchasePrice = 30`. `PlayerInspectDS.CoinBoost = 16` is a reward
//!   stat, not a currency.
//! * The minigame reward preview (`BaseClient.MiniGame.RewardPreview` row 0)
//!   carries coins in column 3 and XP in column 2; a positive coin value adds
//!   the `CoinsScoreScreen` loot-wheel entry.
//!
//! COMMUNITY facts (pinned wiki, secondary): Coins come from quests and
//! dungeons; the Coin Shop spends coins; Station Cash funds microtransactions
//! and can be spent on coin packs.
//!
//! RECONSTRUCTED policy (labelled, not historical): the invariant set below.
//! The local quest, checkpoint, encounter, opening and kart amounts are
//! reconstructed content, so their flows are graded `Reconstructed`. No
//! original payout number is invented here.

/// `BaseClientUpdatePacket` - the client-update family.
pub const COIN_UPDATE_SECTION: u16 = 38;
/// `ClientUpdatePacketCoinCount` inside [`COIN_UPDATE_SECTION`].
pub const COIN_UPDATE_OPCODE: u16 = 19;
/// `BaseCoinStorePacket` - the Coin Shop family.
pub const COIN_STORE_SECTION: u16 = 165;
/// `BaseTradePacket` - player trading.
pub const TRADE_SECTION: u16 = 59;
/// `RewardBasePacket` - reward bundles and deferred non-member rewards.
pub const REWARD_SECTION: u16 = 50;
/// `PacketBaseInGamePurchase` - Station Cash wallet and store orders.
pub const IN_GAME_PURCHASE_SECTION: u16 = 66;
/// `PacketClientNotifyCoinSpinAvailable` - the coin wheel notify opcode.
pub const COIN_SPIN_NOTIFY_OPCODE: u16 = 171;

/// Highest value the recovered 32-bit signed wire fields can carry. Both the
/// coin-count update and the login profile store a signed 32-bit value.
pub const COIN_MAX: i64 = i32::MAX as i64;

/// `PlayerInfo_DS.NumCoins` and `_getCoinCount()` row.
pub const PLAYER_INFO_NUM_COINS_ROW: i32 = 2;
/// `PlayerInfo_DS.Wallet` and `_getStationCashCount()` row.
pub const PLAYER_INFO_WALLET_ROW: i32 = 3;
/// `PlayerInfo_DS.CoinCount` row (About-screen copy).
pub const PLAYER_INFO_COIN_COUNT_ROW: i32 = 16;
/// `PlayerInfo_DS.SCCount` row (About-screen copy).
pub const PLAYER_INFO_SC_COUNT_ROW: i32 = 17;
/// `PlayerInspectDS.Coins` row.
pub const INSPECT_COINS_ROW: i32 = 9;
/// `PlayerInspectDS.CoinBoost` row (reward stat, not a balance).
pub const INSPECT_COIN_BOOST_ROW: i32 = 16;
/// `PlayerInspectDS.StationCash` row.
pub const INSPECT_STATION_CASH_ROW: i32 = 17;
/// `ItemDefinitionSelect_DS.CoinPurchasePrice` row.
pub const ITEM_COIN_PURCHASE_PRICE_ROW: i32 = 25;
/// `ItemDefinitionSelect_DS.CoinSellPrice` row.
pub const ITEM_COIN_SELL_PRICE_ROW: i32 = 26;
/// `MerchantSell_DS.Price` row.
pub const MERCHANT_SELL_PRICE_ROW: i32 = 27;
/// `MerchantSell_DS.MemberPurchasePrice` row.
pub const MERCHANT_MEMBER_PRICE_ROW: i32 = 30;

/// Where a currency balance lives, recovered from the client rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurrencyScope {
    /// Character-bound (coins ride the login profile and the inventory state).
    Character,
    /// Account-bound (Station Cash is funded and spent by the account).
    AccountWallet,
}

/// The two recovered currencies. Tokens and entitlements (for example the
/// paid name-change token) are deliberately not modelled as currencies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurrencyKind {
    Coins,
    StationCash,
}

impl CurrencyKind {
    pub fn scope(self) -> CurrencyScope {
        match self {
            Self::Coins => CurrencyScope::Character,
            Self::StationCash => CurrencyScope::AccountWallet,
        }
    }

    /// Whether a gameplay reward path may create this currency. Every recovered
    /// reward, quest, inventory and trade path moves coins; no recovered
    /// gameplay path creates Station Cash.
    pub fn gameplay_grantable(self) -> bool {
        match self {
            Self::Coins => true,
            Self::StationCash => false,
        }
    }

    pub fn wire_max(self) -> i64 {
        COIN_MAX
    }
}

/// Which way value moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlowDirection {
    Source,
    Sink,
}

/// How well the flow itself is recovered. Amounts are never graded here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvidenceGrade {
    /// Named by the supplied client or supplied server build.
    Original,
    /// Named by the pinned community wiki only.
    Community,
    /// Local reconstruction with no recovered payout number.
    Reconstructed,
}

/// A recovered currency producer or sink, plus the local reward producers the
/// crate implements today.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Producer {
    /// Local quest completion payout (`quests.rs`).
    QuestCompletion,
    /// Local checkpoint-trial payout (`quests.rs`).
    CheckpointTrial,
    /// Local encounter clear payout (`encounter.rs`).
    EncounterClear,
    /// Local opening stage payout (`opening.rs`).
    OpeningStage,
    /// Local opening discovery payout (`opening.rs`).
    OpeningDiscovery,
    /// Local kart-racing payout (`kart_racing.rs`), priced only by the lead.
    KartRace,
    /// Recovered coin wheel (`PacketClientNotifyCoinSpinAvailable = 171`).
    CoinSpin,
    /// Recovered deferred/granted non-member reward (`RewardBasePacket = 50`).
    DeferredNonMemberReward,
    /// Recovered Coin Shop purchase (`BaseCoinStorePacket = 165`).
    CoinShopPurchase,
    /// Recovered Coin Shop buyback (`CoinStore.TransactionHistory`).
    CoinShopBuyback,
    /// Recovered paid revive (`RespawnWindow:DisplayRespawn` cost argument).
    PaidRevive,
    /// Recovered trade give (`TradeCoinCountPacket = 10`).
    TradeGive,
    /// Recovered trade receive (`TradeUpdateCoinCountPacket = 11`).
    TradeReceive,
    /// Recovered account wallet funding (credit-card top-off).
    StationCashWalletFunding,
    /// Recovered Station Cash store order (`PacketInGamePurchasePlaceOrderPacket`).
    StationCashStorePurchase,
}

impl Producer {
    /// Every producer, for exhaustive invariant checks.
    pub const ALL: [Producer; 15] = [
        Producer::QuestCompletion,
        Producer::CheckpointTrial,
        Producer::EncounterClear,
        Producer::OpeningStage,
        Producer::OpeningDiscovery,
        Producer::KartRace,
        Producer::CoinSpin,
        Producer::DeferredNonMemberReward,
        Producer::CoinShopPurchase,
        Producer::CoinShopBuyback,
        Producer::PaidRevive,
        Producer::TradeGive,
        Producer::TradeReceive,
        Producer::StationCashWalletFunding,
        Producer::StationCashStorePurchase,
    ];

    pub fn currency(self) -> CurrencyKind {
        match self {
            Self::StationCashWalletFunding | Self::StationCashStorePurchase => {
                CurrencyKind::StationCash
            }
            _ => CurrencyKind::Coins,
        }
    }

    pub fn direction(self) -> FlowDirection {
        match self {
            Self::CoinShopPurchase
            | Self::CoinShopBuyback
            | Self::PaidRevive
            | Self::TradeGive
            | Self::StationCashStorePurchase => FlowDirection::Sink,
            _ => FlowDirection::Source,
        }
    }

    pub fn evidence(self) -> EvidenceGrade {
        match self {
            Self::QuestCompletion
            | Self::CheckpointTrial
            | Self::EncounterClear
            | Self::OpeningStage
            | Self::OpeningDiscovery
            | Self::KartRace => EvidenceGrade::Reconstructed,
            Self::CoinSpin
            | Self::DeferredNonMemberReward
            | Self::CoinShopPurchase
            | Self::CoinShopBuyback
            | Self::PaidRevive
            | Self::TradeGive
            | Self::TradeReceive
            | Self::StationCashWalletFunding
            | Self::StationCashStorePurchase => EvidenceGrade::Original,
        }
    }
}

/// One currency movement recorded for audit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Flow {
    pub producer: Producer,
    /// The producing outcome (quest id, race session, encounter id, ...). Zero
    /// is rejected: an unkeyed grant cannot be made once-only.
    pub instance: u64,
    /// Signed so a negative amount is representable and gets rejected instead
    /// of wrapping.
    pub amount: i64,
}

/// A rejected flow and why.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Violation {
    ZeroInstance,
    NegativeAmount,
    /// The amount cannot be carried by the recovered 32-bit signed field.
    AmountExceedsWireRange(i64),
    /// A source instance was granted twice; every recovered reward path is
    /// once-only (quest `rewarded`, encounter `rewarded`, crate `claimed`).
    DuplicateSourceInstance,
}

/// Result of auditing a flow list. Totals are raw accounting sums over every
/// listed flow; `violations` is the invariant verdict and stands alone.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AuditReport {
    pub violations: Vec<(usize, Violation)>,
    pub coins_sourced: i64,
    pub coins_sunk: i64,
    pub station_cash_sourced: i64,
    pub station_cash_sunk: i64,
}

impl AuditReport {
    pub fn ok(&self) -> bool {
        self.violations.is_empty()
    }
}

/// Audit a flow list against the recovered invariants: every amount fits the
/// 32-bit signed field, no negative amount, every source instance is keyed and
/// granted at most once, and no gameplay producer creates Station Cash.
pub fn audit(flows: &[Flow]) -> AuditReport {
    let mut report = AuditReport::default();
    let mut seen: Vec<(Producer, u64)> = Vec::new();
    for (index, flow) in flows.iter().enumerate() {
        let currency = flow.producer.currency();
        match flow.producer.direction() {
            FlowDirection::Source => match currency {
                CurrencyKind::Coins => report.coins_sourced += flow.amount,
                CurrencyKind::StationCash => report.station_cash_sourced += flow.amount,
            },
            FlowDirection::Sink => match currency {
                CurrencyKind::Coins => report.coins_sunk += flow.amount,
                CurrencyKind::StationCash => report.station_cash_sunk += flow.amount,
            },
        }
        if flow.instance == 0 {
            report.violations.push((index, Violation::ZeroInstance));
            continue;
        }
        if flow.amount < 0 {
            report.violations.push((index, Violation::NegativeAmount));
            continue;
        }
        if flow.amount > currency.wire_max() {
            report.violations.push((
                index,
                Violation::AmountExceedsWireRange(currency.wire_max()),
            ));
            continue;
        }
        if flow.producer.direction() == FlowDirection::Source {
            let key = (flow.producer, flow.instance);
            if seen.contains(&key) {
                report
                    .violations
                    .push((index, Violation::DuplicateSourceInstance));
                continue;
            }
            seen.push(key);
        }
    }
    report
}

/// Why a ledger movement was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LedgerError {
    NonPositiveAmount,
    WouldOverflow,
    InsufficientFunds,
}

/// Coin balance checker for the sink side: the balance never leaves
/// `0..=COIN_MAX`, so a paid revive, shop purchase or trade can never overdraw
/// or wrap the recovered 32-bit field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoinLedger {
    balance: i64,
}

impl CoinLedger {
    pub fn new(balance: i64) -> Self {
        Self { balance }
    }

    pub fn balance(&self) -> i64 {
        self.balance
    }

    pub fn credit(&mut self, amount: i64) -> Result<i64, LedgerError> {
        if amount <= 0 {
            return Err(LedgerError::NonPositiveAmount);
        }
        let next = self
            .balance
            .checked_add(amount)
            .filter(|value| *value <= COIN_MAX)
            .ok_or(LedgerError::WouldOverflow)?;
        self.balance = next;
        Ok(next)
    }

    pub fn debit(&mut self, amount: i64) -> Result<i64, LedgerError> {
        if amount <= 0 {
            return Err(LedgerError::NonPositiveAmount);
        }
        let next = self
            .balance
            .checked_sub(amount)
            .filter(|value| *value >= 0)
            .ok_or(LedgerError::InsufficientFunds)?;
        self.balance = next;
        Ok(next)
    }
}
