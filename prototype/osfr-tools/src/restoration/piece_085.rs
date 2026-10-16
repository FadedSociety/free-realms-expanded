//! Piece 085 - player trading: recovered offer/accept/cancel wire and the
//! bounded two-sided trade session.
//!
//! ORIGINAL facts implemented here (provenance in
//! `notes/Research/Pieces/085 Player trading.md`):
//!
//! * `decompiled/client/FreeRealms/pseudocode.c`, `FUN_00c6a7b0`: the base trade
//!   packet constructor writes `param_1[1] = 0x3b` (SOE opcode 59) and stores
//!   the trade sub-opcode in `param_1[2]`. The same constructor family covers
//!   all 18 recovered classes, `TradeInvitePacket` = 2 through
//!   `TradeItemRemovedPacket` = 19.
//! * Native field readers pin three inbound payloads: `FUN_00c6c110`
//!   (session u32 + lock bool, used for `TradeUpdateLockPacket` 17, with the
//!   session compared against the active session id), `FUN_00c6c180` (three
//!   u32, used for `TradeItemRemovedPacket` 19, session first) and
//!   `FUN_00c6ce30` (u32 + u32 + bool, used for `TradeUpdateAddItemPacket` 12).
//! * `TradeHandler` event vocabulary: `TradeCompleted`,
//!   `TradeCanceledPartnerDisconnected`, `TradeCanceledPartnerOfferInvalid`,
//!   `TradeCanceledYourOfferInvalid`, the two max-stack cancellations and
//!   `TradeHandler:OtherAccepted` for a partner lock flip.
//! * Community cross-check at pinned revision: `OpenSourceFreeRealms`
//!   `PacketCodes.cs` lists `BaseTradePacket = 59` and the same 18 rows.
//!
//! RECONSTRUCTED, labelled at each call site: the two-sided session machine,
//! the offer-revision stale-lock rule, the atomic transfer plan and the
//! disconnect policy. Order of the fields whose native reader was not recovered
//! is marked on each codec below.

use std::collections::BTreeMap;

/// SOE opcode every trade packet is written under (`FUN_00c6a7b0`: `param_1[1] = 0x3b`).
pub const SOE_BASE_TRADE_OPCODE: u8 = 0x3b;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TradePacket {
    Invite,
    InviteReply,
    StartSession,
    EndSession,
    Accept,
    Cancel,
    Lock,
    OfferItem,
    CoinCount,
    UpdateCoinCount,
    UpdateAddItem,
    UpdateItemCount,
    UpdateAccept,
    Message,
    Rejected,
    UpdateLock,
    Confirm,
    ItemRemoved,
}

impl TradePacket {
    pub const ALL: [TradePacket; 18] = [
        TradePacket::Invite,
        TradePacket::InviteReply,
        TradePacket::StartSession,
        TradePacket::EndSession,
        TradePacket::Accept,
        TradePacket::Cancel,
        TradePacket::Lock,
        TradePacket::OfferItem,
        TradePacket::CoinCount,
        TradePacket::UpdateCoinCount,
        TradePacket::UpdateAddItem,
        TradePacket::UpdateItemCount,
        TradePacket::UpdateAccept,
        TradePacket::Message,
        TradePacket::Rejected,
        TradePacket::UpdateLock,
        TradePacket::Confirm,
        TradePacket::ItemRemoved,
    ];

    pub fn code(self) -> u8 {
        match self {
            TradePacket::Invite => 2,
            TradePacket::InviteReply => 3,
            TradePacket::StartSession => 4,
            TradePacket::EndSession => 5,
            TradePacket::Accept => 6,
            TradePacket::Cancel => 7,
            TradePacket::Lock => 8,
            TradePacket::OfferItem => 9,
            TradePacket::CoinCount => 10,
            TradePacket::UpdateCoinCount => 11,
            TradePacket::UpdateAddItem => 12,
            TradePacket::UpdateItemCount => 13,
            TradePacket::UpdateAccept => 14,
            TradePacket::Message => 15,
            TradePacket::Rejected => 16,
            TradePacket::UpdateLock => 17,
            TradePacket::Confirm => 18,
            TradePacket::ItemRemoved => 19,
        }
    }

    pub fn from_code(code: u8) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|packet| packet.code() == code)
    }

    pub fn name(self) -> &'static str {
        match self {
            TradePacket::Invite => "TradeInvitePacket",
            TradePacket::InviteReply => "TradeInviteReplyPacket",
            TradePacket::StartSession => "TradeStartSessionPacket",
            TradePacket::EndSession => "TradeEndSessionPacket",
            TradePacket::Accept => "TradeAcceptPacket",
            TradePacket::Cancel => "TradeCancelPacket",
            TradePacket::Lock => "TradeLockPacket",
            TradePacket::OfferItem => "TradeOfferItemPacket",
            TradePacket::CoinCount => "TradeCoinCountPacket",
            TradePacket::UpdateCoinCount => "TradeUpdateCoinCountPacket",
            TradePacket::UpdateAddItem => "TradeUpdateAddItemPacket",
            TradePacket::UpdateItemCount => "TradeUpdateItemCountPacket",
            TradePacket::UpdateAccept => "TradeUpdateAcceptPacket",
            TradePacket::Message => "TradeMessagePacket",
            TradePacket::Rejected => "TradeRejectedPacket",
            TradePacket::UpdateLock => "TradeUpdateLockPacket",
            TradePacket::Confirm => "TradeConfirmPacket",
            TradePacket::ItemRemoved => "TradeItemRemovedPacket",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TradeError {
    TruncatedField,
    WrongBaseOpcode(u8),
    UnknownPacket(u8),
    NotBothLocked,
    NotBothConfirmed,
    OfferChanged,
    OfferInvalid,
    SessionClosed,
    StaleSession { expected: u32, got: u32 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WireFrame {
    pub packet: TradePacket,
    pub body: Vec<u8>,
}

impl WireFrame {
    pub fn session(packet: TradePacket, session: u32) -> Self {
        Self {
            packet,
            body: session.to_le_bytes().to_vec(),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(2 + self.body.len());
        bytes.push(SOE_BASE_TRADE_OPCODE);
        bytes.push(self.packet.code());
        bytes.extend_from_slice(&self.body);
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, TradeError> {
        if bytes.is_empty() {
            return Err(TradeError::TruncatedField);
        }
        if bytes[0] != SOE_BASE_TRADE_OPCODE {
            return Err(TradeError::WrongBaseOpcode(bytes[0]));
        }
        let code = *bytes.get(1).ok_or(TradeError::TruncatedField)?;
        let packet = TradePacket::from_code(code).ok_or(TradeError::UnknownPacket(code))?;
        Ok(Self {
            packet,
            body: bytes[2..].to_vec(),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemStack {
    pub guid: u32,
    pub count: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Inventory {
    items: BTreeMap<u32, u32>,
    coins: u32,
}

impl Inventory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, guid: u32, count: u32) {
        self.items.insert(guid, count);
    }

    pub fn count_of(&self, guid: u32) -> u32 {
        self.items.get(&guid).copied().unwrap_or(0)
    }

    pub fn set_coins(&mut self, coins: u32) {
        self.coins = coins;
    }

    pub fn coins(&self) -> u32 {
        self.coins
    }

    fn add_item(&mut self, guid: u32, count: u32) {
        let entry = self.items.entry(guid).or_insert(0);
        *entry = entry.saturating_add(count);
    }

    fn remove_item(&mut self, guid: u32, count: u32) -> bool {
        match self.items.get_mut(&guid) {
            Some(entry) if *entry >= count => {
                *entry -= count;
                if *entry == 0 {
                    self.items.remove(&guid);
                }
                true
            }
            _ => false,
        }
    }

    fn add_coins(&mut self, coins: u32) {
        self.coins = self.coins.saturating_add(coins);
    }

    fn remove_coins(&mut self, coins: u32) -> bool {
        if self.coins < coins {
            return false;
        }
        self.coins -= coins;
        true
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TradeSide {
    Host,
    Guest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TradeOffer {
    pub session: u32,
    pub item_guid: u32,
    pub count: u32,
}

impl TradeOffer {
    pub fn to_frame(&self) -> WireFrame {
        let mut body = self.session.to_le_bytes().to_vec();
        body.extend_from_slice(&self.item_guid.to_le_bytes());
        body.extend_from_slice(&self.count.to_le_bytes());
        WireFrame {
            packet: TradePacket::OfferItem,
            body,
        }
    }

    pub fn from_frame(bytes: &[u8]) -> Result<Self, TradeError> {
        let frame = WireFrame::decode(bytes)?;
        if frame.packet != TradePacket::OfferItem || frame.body.len() < 12 {
            return Err(TradeError::TruncatedField);
        }
        Ok(Self {
            session: u32::from_le_bytes(frame.body[0..4].try_into().unwrap()),
            item_guid: u32::from_le_bytes(frame.body[4..8].try_into().unwrap()),
            count: u32::from_le_bytes(frame.body[8..12].try_into().unwrap()),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TradeUpdateLock {
    pub session: u32,
    pub locked: bool,
}

impl TradeUpdateLock {
    pub fn to_frame(&self) -> WireFrame {
        let mut body = self.session.to_le_bytes().to_vec();
        body.push(u8::from(self.locked));
        WireFrame {
            packet: TradePacket::UpdateLock,
            body,
        }
    }

    pub fn from_frame(bytes: &[u8]) -> Result<Self, TradeError> {
        let frame = WireFrame::decode(bytes)?;
        if frame.packet != TradePacket::UpdateLock || frame.body.len() < 5 {
            return Err(TradeError::TruncatedField);
        }
        Ok(Self {
            session: u32::from_le_bytes(frame.body[0..4].try_into().unwrap()),
            locked: frame.body[4] != 0,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TradeClose {
    Completed,
    Cancelled,
    PartnerDisconnected,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TradeOutcome {
    pub received_by_host: Vec<ItemStack>,
    pub received_by_guest: Vec<ItemStack>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct SideOffer {
    items: BTreeMap<u32, u32>,
    coins: u32,
}

fn side_index(side: TradeSide) -> usize {
    match side {
        TradeSide::Host => 0,
        TradeSide::Guest => 1,
    }
}

/// Bounded two-sided trade session (RECONSTRUCTED policy over the recovered
/// wire).
///
/// Rules, each traceable to a recovered client event:
/// * offers may change until both sides lock; changing an offer bumps the
///   revision, so a lock recorded at an earlier revision is stale
///   (`TradeCanceledPartnerOfferInvalid` / `TradeCanceledYourOfferInvalid`).
/// * `complete` refuses to move anything unless both sides locked and confirmed
///   at the current revision (`TradeCompleted` only fires for a whole trade).
/// * cancel or either side disconnecting closes the session with no transfer
///   (`TradeCanceledPartnerDisconnected`).
#[derive(Clone, Debug)]
pub struct TradeSession {
    id: u32,
    host: u64,
    guest: u64,
    offers: [SideOffer; 2],
    revision: u64,
    /// Revision each side's lock was recorded at; `None` until that side locks.
    locks: [Option<u64>; 2],
    /// Revision each side's confirm was recorded at.
    confirmations: [Option<u64>; 2],
    /// The partner's own lock bit as last reported by a `TradeUpdateLockPacket`.
    remote_locked: [bool; 2],
    closed: Option<TradeClose>,
}

impl TradeSession {
    pub fn open(id: u32, host: u64, guest: u64) -> Self {
        Self {
            id,
            host,
            guest,
            offers: [SideOffer::default(), SideOffer::default()],
            revision: 1,
            locks: [None, None],
            confirmations: [None, None],
            remote_locked: [false, false],
            closed: None,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    /// Add or replace one offered stack. Any change bumps the revision, which
    /// invalidates locks recorded against the previous offer.
    pub fn offer_item(&mut self, side: TradeSide, guid: u32, count: u32) -> Result<(), TradeError> {
        if self.closed.is_some() {
            return Err(TradeError::SessionClosed);
        }
        if count == 0 {
            return Err(TradeError::OfferInvalid);
        }
        self.offers[side_index(side)].items.insert(guid, count);
        self.revision += 1;
        Ok(())
    }

    pub fn offer_coins(&mut self, side: TradeSide, coins: u32) -> Result<(), TradeError> {
        if self.closed.is_some() {
            return Err(TradeError::SessionClosed);
        }
        if coins == 0 {
            return Err(TradeError::OfferInvalid);
        }
        self.offers[side_index(side)].coins = coins;
        self.revision += 1;
        Ok(())
    }

    /// The client's accept/lock of one's own offer (`TradeLockPacket` 8); the
    /// partner observes it as `TradeUpdateLockPacket` 17 / `OtherAccepted`.
    pub fn lock(&mut self, side: TradeSide) -> Result<(), TradeError> {
        if self.closed.is_some() {
            return Err(TradeError::SessionClosed);
        }
        self.locks[side_index(side)] = Some(self.revision);
        Ok(())
    }

    /// Final confirmation (`TradeConfirmPacket` 18). Refused until both sides
    /// have locked, and refused if any offer changed after the locks.
    pub fn confirm(&mut self, side: TradeSide) -> Result<(), TradeError> {
        if self.closed.is_some() {
            return Err(TradeError::SessionClosed);
        }
        if self.locks[0].is_none() || self.locks[1].is_none() {
            return Err(TradeError::NotBothLocked);
        }
        if self.locks[0] != Some(self.revision) || self.locks[1] != Some(self.revision) {
            return Err(TradeError::OfferChanged);
        }
        self.confirmations[side_index(side)] = Some(self.revision);
        Ok(())
    }

    /// Whole-or-nothing transfer of both offers.
    ///
    /// The plan is built on copies of both inventories; the real inventories are
    /// only replaced after every offer has been validated, so a failure on
    /// either side leaves both untouched.
    pub fn complete(
        &mut self,
        host: &mut Inventory,
        guest: &mut Inventory,
    ) -> Result<TradeOutcome, TradeError> {
        if self.closed.is_some() {
            return Err(TradeError::SessionClosed);
        }
        let both_locked = self.locks[0].is_some() && self.locks[1].is_some();
        let stale = self.locks[0] != Some(self.revision) || self.locks[1] != Some(self.revision);
        if both_locked && stale {
            return Err(TradeError::OfferChanged);
        }
        if self.confirmations[0] != Some(self.revision)
            || self.confirmations[1] != Some(self.revision)
        {
            return Err(TradeError::NotBothConfirmed);
        }

        let host_offer = self.offers[0].clone();
        let guest_offer = self.offers[1].clone();
        let mut host_plan = host.clone();
        let mut guest_plan = guest.clone();

        for (guid, count) in &host_offer.items {
            if !host_plan.remove_item(*guid, *count) {
                return Err(TradeError::OfferInvalid);
            }
        }
        if !host_plan.remove_coins(host_offer.coins) {
            return Err(TradeError::OfferInvalid);
        }
        for (guid, count) in &guest_offer.items {
            if !guest_plan.remove_item(*guid, *count) {
                return Err(TradeError::OfferInvalid);
            }
        }
        if !guest_plan.remove_coins(guest_offer.coins) {
            return Err(TradeError::OfferInvalid);
        }

        let received_by_host: Vec<ItemStack> = guest_offer
            .items
            .iter()
            .map(|(guid, count)| ItemStack {
                guid: *guid,
                count: *count,
            })
            .collect();
        let received_by_guest: Vec<ItemStack> = host_offer
            .items
            .iter()
            .map(|(guid, count)| ItemStack {
                guid: *guid,
                count: *count,
            })
            .collect();
        for stack in &received_by_host {
            host_plan.add_item(stack.guid, stack.count);
        }
        host_plan.add_coins(guest_offer.coins);
        for stack in &received_by_guest {
            guest_plan.add_item(stack.guid, stack.count);
        }
        guest_plan.add_coins(host_offer.coins);

        *host = host_plan;
        *guest = guest_plan;
        self.closed = Some(TradeClose::Completed);
        Ok(TradeOutcome {
            received_by_host,
            received_by_guest,
        })
    }

    pub fn cancel(&mut self, _side: TradeSide) -> Result<(), TradeError> {
        if self.closed.is_some() {
            return Err(TradeError::SessionClosed);
        }
        self.closed = Some(TradeClose::Cancelled);
        Ok(())
    }

    /// Either side dropping is a cancel with no transfer
    /// (`TradeCanceledPartnerDisconnected`).
    pub fn partner_disconnected(&mut self, _side: TradeSide) -> Result<(), TradeError> {
        if self.closed.is_some() {
            return Err(TradeError::SessionClosed);
        }
        self.closed = Some(TradeClose::PartnerDisconnected);
        Ok(())
    }

    pub fn closed_reason(&self) -> Option<TradeClose> {
        self.closed
    }

    pub fn is_open(&self) -> bool {
        self.closed.is_none()
    }

    /// Handle an inbound `TradeUpdateLockPacket` (17). The native client
    /// compares the frame's session field against its active session and ignores
    /// frames that do not match; mismatches are surfaced here as a typed error.
    pub fn apply_lock_frame(
        &mut self,
        side: TradeSide,
        frame: &TradeUpdateLock,
    ) -> Result<(), TradeError> {
        if self.closed.is_some() {
            return Err(TradeError::SessionClosed);
        }
        if frame.session != self.id {
            return Err(TradeError::StaleSession {
                expected: self.id,
                got: frame.session,
            });
        }
        self.remote_locked[side_index(side)] = frame.locked;
        Ok(())
    }

    pub fn remote_lock(&self, side: TradeSide) -> bool {
        self.remote_locked[side_index(side)]
    }

    pub fn players(&self) -> (u64, u64) {
        (self.host, self.guest)
    }
}
