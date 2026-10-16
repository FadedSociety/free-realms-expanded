//! Player-to-player trade session service.
//!
//! Client wire facts come from `notes/Build Guides/Gameplay/Trade.md` and the native readers
//! cited beside each packet builder. Two-party admission, offer ownership, capacity, lock reset,
//! accept timing, timeout, and settlement are reconstructed server policy (R). The caller owns
//! persistence and copies committed `InventoryState` snapshots back into the player save layer.

use crate::{
    inventory::{Inventory, InventoryItem, InventoryState, RentalState},
    players::{PlayerRegistry, PlayerSession, SessionId},
    social_lists::SocialService,
};
use std::collections::{BTreeMap, BTreeSet};

const FAMILY: u16 = 0x3b;
const ACCEPT_DELAY_MS: u64 = 10_000;
// (R) Reuse the signed 32-bit inventory/wire ceiling; native trade has no recovered coin cap.
const MAX_COINS: u32 = i32::MAX as u32;

/// Definition metadata required by the trade gate. `no_trade` and stack size come from the
/// client definition; bag capacity and pet-equipment tracking are supplied by the host (R).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TradeItemDefinition {
    pub no_trade: bool,
    /// `None` means uncapped (R); the native resource's `-1` meaning is unresolved.
    pub max_stack: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct TradeInventory {
    pub state: InventoryState,
    pub definitions: BTreeMap<i32, TradeItemDefinition>,
    /// (R) Pet equipment is separate from `InventoryState::equipment` in this prototype.
    pub equipped_on_pet: BTreeSet<i32>,
    /// Maximum occupied item rows. The original bag limit is not recovered (R).
    pub bag_capacity: usize,
}

impl TradeInventory {
    pub fn new(
        state: InventoryState,
        definitions: BTreeMap<i32, TradeItemDefinition>,
        equipped_on_pet: BTreeSet<i32>,
        bag_capacity: usize,
    ) -> Result<Self, TradeError> {
        let result = Self {
            state,
            definitions,
            equipped_on_pet,
            bag_capacity,
        };
        validate_trade_inventory(&result)?;
        Ok(result)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeMessage {
    pub recipient: SessionId,
    /// Inner application payload. Gateway wrapping belongs to the caller.
    pub bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum TradeStatus {
    Cancelled = 1,
    PartnerDisconnected = 2,
    PartnerOfferInvalid = 3,
    YourOfferInvalid = 4,
    Completed = 6,
    EnteredInstancedZone = 7,
    PartnerEnteredInstancedZone = 8,
    YourOfferExceedsPartnerMaxStack = 9,
    PartnerOfferExceedsYourMaxStack = 10,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TradeError {
    UnknownPlayer,
    PlayerOffline,
    PlayerBusy,
    Ignored,
    UnknownSession,
    NotParticipant,
    InvalidInviteReply,
    UnknownItem,
    UntradableItem,
    InvalidQuantity,
    InsufficientCoins,
    StaleCounter,
    NotBothLocked,
    AcceptTooEarly,
    SessionIdExhausted,
    RevisionExhausted,
    InvalidInventory,
    WireRange,
}

impl std::fmt::Display for TradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for TradeError {}

#[derive(Clone, Debug)]
struct TradePlayer {
    session_id: SessionId,
    inventory: TradeInventory,
}

#[derive(Clone, Copy, Debug)]
struct PendingInvite {
    inviter: u64,
    invitee: u64,
}

#[derive(Clone, Debug)]
struct Session {
    players: [u64; 2],
    offers: [BTreeMap<i32, u32>; 2],
    coins: [u32; 2],
    counter: u32,
    locked: [bool; 2],
    accepted: [bool; 2],
    confirm_started_at: Option<u64>,
}

impl Session {
    fn index(&self, guid: u64) -> Result<usize, TradeError> {
        self.players
            .iter()
            .position(|player| *player == guid)
            .ok_or(TradeError::NotParticipant)
    }

    fn other(&self, index: usize) -> usize {
        1 - index
    }
}

#[derive(Clone, Debug, Default)]
pub struct TradeService {
    players: BTreeMap<u64, TradePlayer>,
    busy: BTreeMap<u64, u32>,
    pending: BTreeMap<u32, PendingInvite>,
    sessions: BTreeMap<u32, Session>,
    next_session_id: u32,
}

impl TradeService {
    pub fn new() -> Self {
        Self {
            next_session_id: 1,
            ..Self::default()
        }
    }

    /// Register the inventory snapshot bound to one live player session.
    pub fn register_player(
        &mut self,
        player: &PlayerSession,
        inventory: TradeInventory,
    ) -> Result<(), TradeError> {
        validate_trade_inventory(&inventory)?;
        if player.character_guid == 0 {
            return Err(TradeError::UnknownPlayer);
        }
        if self.busy.contains_key(&player.character_guid) {
            return Err(TradeError::PlayerBusy);
        }
        self.players.insert(
            player.character_guid,
            TradePlayer {
                session_id: player.session_id,
                inventory,
            },
        );
        Ok(())
    }

    /// Replace a snapshot after another authoritative inventory operation. Offers are deliberately
    /// retained; both sides are revalidated at settlement (R).
    pub fn sync_inventory(
        &mut self,
        guid: u64,
        inventory: TradeInventory,
    ) -> Result<(), TradeError> {
        validate_trade_inventory(&inventory)?;
        let player = self
            .players
            .get_mut(&guid)
            .ok_or(TradeError::UnknownPlayer)?;
        player.inventory = inventory;
        Ok(())
    }

    pub fn inventory(&self, guid: u64) -> Option<&InventoryState> {
        self.players
            .get(&guid)
            .map(|player| &player.inventory.state)
    }

    pub fn session_counter(&self, session_id: u32) -> Option<u32> {
        self.sessions
            .get(&session_id)
            .map(|session| session.counter)
    }

    pub fn session_id_for(&self, guid: u64) -> Option<u32> {
        self.busy.get(&guid).copied()
    }

    pub fn is_locked(&self, session_id: u32, guid: u64) -> bool {
        self.sessions
            .get(&session_id)
            .and_then(|session| session.index(guid).ok().map(|i| session.locked[i]))
            .unwrap_or(false)
    }

    /// Handle the inferred command 26/10 selection for the server-supplied Trade entry.
    pub fn invite(
        &mut self,
        players: &PlayerRegistry,
        social: &SocialService,
        inviter: u64,
        invitee: u64,
    ) -> Result<Vec<TradeMessage>, TradeError> {
        self.online_player(players, social, inviter)?;
        self.online_player(players, social, invitee)?;
        if inviter == invitee {
            return Err(TradeError::NotParticipant);
        }
        if self.busy.contains_key(&inviter) || self.busy.contains_key(&invitee) {
            return Err(TradeError::PlayerBusy);
        }
        let target = social.profile(invitee).ok_or(TradeError::UnknownPlayer)?;
        if target.ignored.contains_key(&inviter) {
            // (R) Match the client's ignore check and prevent silent invite loss.
            return Err(TradeError::Ignored);
        }
        let session_id = self.allocate_session_id()?;
        let name = &social
            .profile(inviter)
            .ok_or(TradeError::UnknownPlayer)?
            .name;
        let bytes = invite_packet(name, inviter, session_id)?;
        self.pending
            .insert(session_id, PendingInvite { inviter, invitee });
        self.busy.insert(inviter, session_id);
        self.busy.insert(invitee, session_id);
        Ok(vec![self.message(invitee, bytes)?])
    }

    /// Accept or decline the pending invite. The client sends sub 3; only acceptance opens a
    /// session and causes the server to send sub 4 to both participants.
    pub fn reply_invite(
        &mut self,
        players: &PlayerRegistry,
        social: &SocialService,
        invitee: u64,
        session_id: u32,
        accept: bool,
    ) -> Result<Vec<TradeMessage>, TradeError> {
        let invite = *self
            .pending
            .get(&session_id)
            .ok_or(TradeError::UnknownSession)?;
        if invite.invitee != invitee {
            return Err(TradeError::InvalidInviteReply);
        }
        self.online_player(players, social, invitee)?;
        self.online_player(players, social, invite.inviter)?;
        let responder_name = &social
            .profile(invitee)
            .ok_or(TradeError::UnknownPlayer)?
            .name;
        if !accept {
            self.pending.remove(&session_id);
            self.clear_busy(session_id, [invite.inviter, invite.invitee]);
            return Ok(vec![
                self.message(invite.inviter, rejected_packet(responder_name)?)?,
            ]);
        }

        let participants = [invite.inviter, invite.invitee];
        let mut out = Vec::with_capacity(2);
        for (index, guid) in participants.iter().enumerate() {
            let other = participants[1 - index];
            let profile = social.profile(other).ok_or(TradeError::UnknownPlayer)?;
            out.push(self.message(*guid, start_packet(other, &profile.name, session_id)?)?);
        }
        self.pending.remove(&session_id);
        self.sessions.insert(
            session_id,
            Session {
                players: participants,
                offers: [BTreeMap::new(), BTreeMap::new()],
                coins: [0, 0],
                counter: 1,
                locked: [false, false],
                accepted: [false, false],
                confirm_started_at: None,
            },
        );
        Ok(out)
    }

    /// Set an offered item total. Zero removes a row. The client sends sub 9; server echoes use
    /// sub 12 for a new row and sub 13 for later totals/removal.
    pub fn offer_item(
        &mut self,
        players: &PlayerRegistry,
        social: &SocialService,
        guid: u64,
        session_id: u32,
        item_id: i32,
        quantity: u32,
    ) -> Result<Vec<TradeMessage>, TradeError> {
        self.online_player(players, social, guid)?;
        let session = self
            .sessions
            .get(&session_id)
            .ok_or(TradeError::UnknownSession)?;
        let index = session.index(guid)?;
        let previous = session.offers[index].get(&item_id).copied().unwrap_or(0);
        if previous == quantity {
            return Ok(Vec::new());
        }
        if quantity > 0 {
            let player = self.players.get(&guid).ok_or(TradeError::UnknownPlayer)?;
            validate_offer_item(&player.inventory, item_id, quantity)?;
        } else if previous == 0 {
            return Err(TradeError::UnknownItem);
        }
        let counter = session
            .counter
            .checked_add(1)
            .ok_or(TradeError::RevisionExhausted)?;
        let session = self.sessions.get_mut(&session_id).unwrap();
        let unlocked = clear_acceptance(session);
        session.counter = counter;
        if quantity == 0 {
            session.offers[index].remove(&item_id);
        } else {
            session.offers[index].insert(item_id, quantity);
        }
        let item = if quantity > 0 {
            let mut item = self.players[&guid]
                .inventory
                .state
                .items
                .iter()
                .find(|item| item.id == item_id)
                .cloned()
                .ok_or(TradeError::UnknownItem)?;
            item.count = quantity;
            // (R) The row represents only the offered available quantity, not stored/consumed units.
            item.consumed_count = 0;
            item.stored_count = 0;
            Some(item)
        } else {
            None
        };
        let session = &self.sessions[&session_id];
        let mut out = self.lock_reset_messages(session, session_id, unlocked)?;
        for recipient_index in 0..2 {
            let side = u8::from(recipient_index == index);
            let bytes = match &item {
                Some(item) if previous == 0 => {
                    update_add_item_packet(session_id, counter, side, item)?
                }
                Some(_) => update_item_count_packet(session_id, counter, side, item_id, quantity)?,
                None => update_item_count_packet(session_id, counter, side, item_id, 0)?,
            };
            out.push(self.message(session.players[recipient_index], bytes)?);
        }
        Ok(out)
    }

    /// Set the coin offer total. The server checks the current wallet before echoing sub 11.
    pub fn offer_coins(
        &mut self,
        players: &PlayerRegistry,
        social: &SocialService,
        guid: u64,
        session_id: u32,
        amount: u32,
    ) -> Result<Vec<TradeMessage>, TradeError> {
        self.online_player(players, social, guid)?;
        let session = self
            .sessions
            .get(&session_id)
            .ok_or(TradeError::UnknownSession)?;
        let index = session.index(guid)?;
        if session.coins[index] == amount {
            return Ok(Vec::new());
        }
        let player = self.players.get(&guid).ok_or(TradeError::UnknownPlayer)?;
        if amount > player.inventory.state.coins || amount > MAX_COINS {
            return Err(TradeError::InsufficientCoins);
        }
        let counter = session
            .counter
            .checked_add(1)
            .ok_or(TradeError::RevisionExhausted)?;
        let session = self.sessions.get_mut(&session_id).unwrap();
        let unlocked = clear_acceptance(session);
        session.counter = counter;
        session.coins[index] = amount;
        let session = &self.sessions[&session_id];
        let mut out = self.lock_reset_messages(session, session_id, unlocked)?;
        for recipient_index in 0..2 {
            let side = u8::from(recipient_index == index);
            out.push(self.message(
                session.players[recipient_index],
                update_coin_packet(session_id, counter, side, amount),
            )?);
        }
        Ok(out)
    }

    /// Record a lock using the client's current counter. Both locks start the client-side 10 s timer.
    pub fn set_lock(
        &mut self,
        players: &PlayerRegistry,
        social: &SocialService,
        guid: u64,
        session_id: u32,
        counter: u32,
        locked: bool,
        now_ms: u64,
    ) -> Result<Vec<TradeMessage>, TradeError> {
        self.online_player(players, social, guid)?;
        let session = self
            .sessions
            .get(&session_id)
            .ok_or(TradeError::UnknownSession)?;
        let index = session.index(guid)?;
        if counter != session.counter {
            return Err(TradeError::StaleCounter);
        }
        if session.locked[index] == locked {
            return Ok(Vec::new());
        }
        let session = self.sessions.get_mut(&session_id).unwrap();
        if !locked {
            let had_lock = session.locked.iter().any(|value| *value);
            session.locked = [false, false];
            session.accepted = [false, false];
            session.confirm_started_at = None;
            if !had_lock {
                return Ok(Vec::new());
            }
            let participants = session.players;
            let mut out = Vec::with_capacity(2);
            for participant in participants {
                out.push(self.message(participant, update_lock_packet(session_id, false))?);
            }
            return Ok(out);
        }
        session.locked[index] = true;
        let both_locked = session.locked.iter().all(|value| *value);
        if both_locked {
            session.confirm_started_at = Some(now_ms);
        }
        let other = session.players[session.other(index)];
        let participants = session.players;
        let mut out = vec![self.message(other, update_lock_packet(session_id, true))?];
        if both_locked {
            for participant in participants {
                out.push(self.message(participant, confirm_packet(session_id))?);
            }
        }
        Ok(out)
    }

    /// Accept only after both locks and the native ten-second timer. Both snapshots are staged
    /// before either is replaced, following the existing atomic reward transaction pattern (R).
    pub fn accept(
        &mut self,
        players: &PlayerRegistry,
        social: &SocialService,
        guid: u64,
        session_id: u32,
        counter: u32,
        now_ms: u64,
    ) -> Result<Vec<TradeMessage>, TradeError> {
        self.online_player(players, social, guid)?;
        let session = self
            .sessions
            .get(&session_id)
            .ok_or(TradeError::UnknownSession)?;
        let index = session.index(guid)?;
        if counter != session.counter {
            return Err(TradeError::StaleCounter);
        }
        if !session.locked.iter().all(|value| *value) {
            return Err(TradeError::NotBothLocked);
        }
        let start = session
            .confirm_started_at
            .ok_or(TradeError::NotBothLocked)?;
        if now_ms.saturating_sub(start) < ACCEPT_DELAY_MS {
            return Err(TradeError::AcceptTooEarly);
        }
        if session.accepted[index] {
            return Ok(Vec::new());
        }
        let session = self.sessions.get_mut(&session_id).unwrap();
        session.accepted[index] = true;
        if !session.accepted.iter().all(|value| *value) {
            return Ok(Vec::new());
        }

        let session = self.sessions[&session_id].clone();
        let first = self.players[&session.players[0]].inventory.clone();
        let second = self.players[&session.players[1]].inventory.clone();
        match stage_swap(&session, first, second) {
            Ok([first, second]) => {
                self.players.get_mut(&session.players[0]).unwrap().inventory = first;
                self.players.get_mut(&session.players[1]).unwrap().inventory = second;
                Ok(self.close_active(session_id, [TradeStatus::Completed; 2]))
            }
            Err(failure) => Ok(self.close_active(session_id, failure.statuses())),
        }
    }

    /// Client cancel (sub 7) or host timeout. Active sessions always receive status then end 5.
    pub fn cancel(&mut self, guid: u64, session_id: u32) -> Result<Vec<TradeMessage>, TradeError> {
        if let Some(session) = self.sessions.get(&session_id) {
            session.index(guid)?;
            return Ok(self.close_active(session_id, [TradeStatus::Cancelled; 2]));
        }
        if let Some(invite) = self.pending.get(&session_id).copied() {
            if guid != invite.inviter && guid != invite.invitee {
                return Err(TradeError::NotParticipant);
            }
            return Ok(self.close_pending(session_id, invite));
        }
        Err(TradeError::UnknownSession)
    }

    /// Disconnect cleanup. No inventory changes occur before both accepts commit successfully.
    pub fn disconnect(&mut self, guid: u64) -> Vec<TradeMessage> {
        let Some(session_id) = self.busy.get(&guid).copied() else {
            return Vec::new();
        };
        if let Some(invite) = self.pending.get(&session_id).copied() {
            self.close_pending(session_id, invite)
        } else {
            self.close_active(session_id, [TradeStatus::PartnerDisconnected; 2])
        }
    }

    /// Cancel when either participant enters an instanced zone (statuses 7/8).
    pub fn enter_instanced_zone(&mut self, guid: u64) -> Vec<TradeMessage> {
        let Some(session_id) = self.busy.get(&guid).copied() else {
            return Vec::new();
        };
        if let Some(session) = self.sessions.get(&session_id) {
            let Ok(index) = session.index(guid) else {
                return Vec::new();
            };
            let statuses = if index == 0 {
                [
                    TradeStatus::EnteredInstancedZone,
                    TradeStatus::PartnerEnteredInstancedZone,
                ]
            } else {
                [
                    TradeStatus::PartnerEnteredInstancedZone,
                    TradeStatus::EnteredInstancedZone,
                ]
            };
            return self.close_active(session_id, statuses);
        }
        if let Some(invite) = self.pending.get(&session_id).copied() {
            let statuses = if invite.inviter == guid {
                [
                    TradeStatus::EnteredInstancedZone,
                    TradeStatus::PartnerEnteredInstancedZone,
                ]
            } else if invite.invitee == guid {
                [
                    TradeStatus::PartnerEnteredInstancedZone,
                    TradeStatus::EnteredInstancedZone,
                ]
            } else {
                return Vec::new();
            };
            return self.close_pending_with_status(session_id, invite, statuses);
        }
        Vec::new()
    }

    /// Called by the host's chosen expiry policy. No invite/session lifetime is recovered from the
    /// client, so this service does not invent one. Active timeout closes with status 1 and end 5.
    pub fn timeout(&mut self, session_id: u32) -> Result<Vec<TradeMessage>, TradeError> {
        if let Some(invite) = self.pending.get(&session_id).copied() {
            return Ok(self.close_pending(session_id, invite));
        }
        if self.sessions.contains_key(&session_id) {
            return Ok(self.close_active(session_id, [TradeStatus::Cancelled; 2]));
        }
        Err(TradeError::UnknownSession)
    }

    fn online_player<'a>(
        &'a self,
        players: &PlayerRegistry,
        social: &SocialService,
        guid: u64,
    ) -> Result<&'a TradePlayer, TradeError> {
        let player = self.players.get(&guid).ok_or(TradeError::UnknownPlayer)?;
        if !players
            .get(player.session_id)
            .is_some_and(|session| session.character_guid == guid)
            || !social.is_online(guid)
            || social.profile(guid).is_none()
        {
            return Err(TradeError::PlayerOffline);
        }
        Ok(player)
    }

    fn allocate_session_id(&mut self) -> Result<u32, TradeError> {
        // (R) Allocate fresh ids and wrap while skipping zero and the client's no-session sentinel.
        let attempts = self.pending.len() + self.sessions.len() + 1;
        for _ in 0..=attempts {
            let id = self.next_session_id.max(1);
            self.next_session_id = if id >= u32::MAX - 1 { 1 } else { id + 1 };
            if id != u32::MAX && !self.pending.contains_key(&id) && !self.sessions.contains_key(&id)
            {
                return Ok(id);
            }
        }
        Err(TradeError::SessionIdExhausted)
    }

    fn message(&self, guid: u64, bytes: Vec<u8>) -> Result<TradeMessage, TradeError> {
        let player = self.players.get(&guid).ok_or(TradeError::UnknownPlayer)?;
        Ok(TradeMessage {
            recipient: player.session_id,
            bytes,
        })
    }

    fn lock_reset_messages(
        &self,
        session: &Session,
        session_id: u32,
        was_locked: bool,
    ) -> Result<Vec<TradeMessage>, TradeError> {
        if !was_locked {
            return Ok(Vec::new());
        }
        session
            .players
            .iter()
            .map(|guid| self.message(*guid, update_lock_packet(session_id, false)))
            .collect()
    }

    fn close_active(&mut self, session_id: u32, statuses: [TradeStatus; 2]) -> Vec<TradeMessage> {
        let Some(session) = self.sessions.remove(&session_id) else {
            return Vec::new();
        };
        self.clear_busy(session_id, session.players);
        let mut out = Vec::with_capacity(4);
        for index in 0..2 {
            if let Ok(status) = self.message(session.players[index], status_packet(statuses[index]))
            {
                out.push(status);
            }
            if let Ok(end) = self.message(session.players[index], end_packet(session_id)) {
                out.push(end);
            }
        }
        out
    }

    fn close_pending(&mut self, session_id: u32, invite: PendingInvite) -> Vec<TradeMessage> {
        self.close_pending_with_status(session_id, invite, [TradeStatus::PartnerDisconnected; 2])
    }

    fn close_pending_with_status(
        &mut self,
        session_id: u32,
        invite: PendingInvite,
        statuses: [TradeStatus; 2],
    ) -> Vec<TradeMessage> {
        self.pending.remove(&session_id);
        self.clear_busy(session_id, [invite.inviter, invite.invitee]);
        let mut out = Vec::with_capacity(4);
        // Sub 2/7/8 clears pending toasts; end 5 follows each terminal status (R).
        for (index, guid) in [invite.inviter, invite.invitee].into_iter().enumerate() {
            if let Ok(status) = self.message(guid, status_packet(statuses[index])) {
                out.push(status);
            }
            if let Ok(end) = self.message(guid, end_packet(session_id)) {
                out.push(end);
            }
        }
        out
    }

    fn clear_busy(&mut self, session_id: u32, participants: [u64; 2]) {
        for guid in participants {
            if self.busy.get(&guid) == Some(&session_id) {
                self.busy.remove(&guid);
            }
        }
    }
}

/// Answer a client trade request while player trade is not offered (R).
///
/// No client path creates a trade: Lua `Trade` has no invite entry, the chat commands are
/// say/tell/groupsay/yell/lfg/area/tradechat/guildsay/mo (`00b15450`, `tradechat` is a chat
/// channel) and the console ones attack/hit/report (`00aa06a0`), and the player radial offers
/// Inspect only. The six requests a client can still send (Trade.md "Client to server": sub 3
/// reply `00c6cee0`, 6 accept `00c6d060`, 7 cancel `00c6d100`, 8 lock `00c6cf80`, 9 item
/// `00c6d160`, 10 coins `00c6d240`) therefore only come from a stale or forged client. Nothing
/// is created, read or changed here, so replays cannot mutate anything.
///
/// The only window-close message is sub 5 (`00c6c530`), and a status alone does not close it, so
/// every request that could mean an open window or an accepted toast is answered to the sender
/// with sub 5 for the same session id; actions that imply an open window get status 1 first.
/// A decline (sub 3, accepted 0) already cleared the client, and a malformed packet, the
/// client's idle id `0xFFFFFFFF` or another family gets no reply (R).
pub fn unavailable_replies(inner: &[u8]) -> Vec<Vec<u8>> {
    // Layout: u16 family 59, u8 sub, u32 session, then the body.
    let [0x3b, 0, sub, session @ ..] = inner else {
        return Vec::new();
    };
    let Some((session, body)) = session.split_first_chunk::<4>() else {
        return Vec::new();
    };
    let session = u32::from_le_bytes(*session);
    let body_len = match sub {
        3 => 1,
        6 | 10 => 4,
        7 => 0,
        8 => 5,
        9 => 8,
        _ => return Vec::new(),
    };
    if session == u32::MAX || body.len() != body_len {
        return Vec::new();
    }
    match (*sub, body.first()) {
        (3, Some(0)) => Vec::new(),
        (3 | 7, _) => vec![end_packet(session)],
        _ => vec![status_packet(TradeStatus::Cancelled), end_packet(session)],
    }
}

fn clear_acceptance(session: &mut Session) -> bool {
    let had_lock = session.locked.iter().any(|value| *value);
    session.locked = [false, false];
    session.accepted = [false, false];
    session.confirm_started_at = None;
    had_lock
}

fn validate_trade_inventory(inventory: &TradeInventory) -> Result<(), TradeError> {
    Inventory::validate_state(&inventory.state).map_err(|_| TradeError::InvalidInventory)?;
    if inventory.state.items.len() > inventory.bag_capacity
        || inventory.definitions.iter().any(|(id, def)| {
            *id <= 0 || def.max_stack.is_some_and(|max| max == 0 || max > MAX_COINS)
        })
        || inventory
            .equipped_on_pet
            .iter()
            .any(|id| !inventory.state.items.iter().any(|item| item.id == *id))
        || inventory.state.items.iter().any(|item| {
            i32::try_from(item.last_cast_time).is_err()
                || i32::try_from(item.count).is_err()
                || i32::try_from(item.consumed_count).is_err()
                || i32::try_from(item.stored_count).is_err()
        })
    {
        return Err(TradeError::InvalidInventory);
    }
    Ok(())
}

fn validate_offer_item(
    inventory: &TradeInventory,
    item_id: i32,
    quantity: u32,
) -> Result<(), TradeError> {
    let item = inventory
        .state
        .items
        .iter()
        .find(|item| item.id == item_id)
        .ok_or(TradeError::UnknownItem)?;
    let definition = inventory
        .definitions
        .get(&item.definition_id)
        .ok_or(TradeError::UnknownItem)?;
    let used = item
        .stored_count
        .checked_add(item.consumed_count)
        .ok_or(TradeError::InvalidQuantity)?;
    let available = item
        .count
        .checked_sub(used)
        .ok_or(TradeError::InvalidQuantity)?;
    if definition.no_trade
        || inventory
            .state
            .equipment
            .iter()
            .any(|equipment| equipment.item_id == item_id)
        || inventory.equipped_on_pet.contains(&item_id)
    {
        // (R) NO_TRADE, equipped state, and pet equipment block an offer.
        return Err(TradeError::UntradableItem);
    }
    if quantity == 0
        || quantity > available
        || definition.max_stack.is_some_and(|max| quantity > max)
    {
        return Err(TradeError::InvalidQuantity);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug)]
enum TransferFailureKind {
    Invalid,
    Stack,
}

#[derive(Clone, Copy, Debug)]
struct TransferFailure {
    source: usize,
    kind: TransferFailureKind,
}

impl TransferFailure {
    fn statuses(self) -> [TradeStatus; 2] {
        match self.kind {
            TransferFailureKind::Invalid => {
                let mut statuses = [TradeStatus::PartnerOfferInvalid; 2];
                statuses[self.source] = TradeStatus::YourOfferInvalid;
                statuses[1 - self.source] = TradeStatus::PartnerOfferInvalid;
                statuses
            }
            TransferFailureKind::Stack => {
                let mut statuses = [TradeStatus::PartnerOfferExceedsYourMaxStack; 2];
                statuses[self.source] = TradeStatus::YourOfferExceedsPartnerMaxStack;
                statuses[1 - self.source] = TradeStatus::PartnerOfferExceedsYourMaxStack;
                statuses
            }
        }
    }
}

fn stage_swap(
    session: &Session,
    first: TradeInventory,
    second: TradeInventory,
) -> Result<[TradeInventory; 2], TransferFailure> {
    // (R) Stage both owners before mutating either authoritative snapshot.
    let mut staged = [first, second];
    for source in 0..2 {
        if session.coins[source] > staged[source].state.coins {
            return Err(TransferFailure {
                source,
                kind: TransferFailureKind::Invalid,
            });
        }
        for (&item_id, &quantity) in &session.offers[source] {
            if validate_offer_item(&staged[source], item_id, quantity).is_err() {
                return Err(TransferFailure {
                    source,
                    kind: TransferFailureKind::Invalid,
                });
            }
        }
    }

    let mut offered = [Vec::new(), Vec::new()];
    for source in 0..2 {
        staged[source].state.coins -= session.coins[source];
        for (&item_id, &quantity) in &session.offers[source] {
            let Some(index) = staged[source]
                .state
                .items
                .iter()
                .position(|item| item.id == item_id)
            else {
                return Err(TransferFailure {
                    source,
                    kind: TransferFailureKind::Invalid,
                });
            };
            let mut item = staged[source].state.items[index].clone();
            if quantity == item.count {
                staged[source].state.items.remove(index);
            } else {
                let owned = &mut staged[source].state.items[index];
                owned.count -= quantity;
                owned.consumed_count = owned.consumed_count.min(owned.count);
                owned.stored_count = owned.stored_count.min(owned.count - owned.consumed_count);
            }
            item.count = quantity;
            item.consumed_count = 0;
            item.stored_count = 0;
            offered[source].push(item);
        }
    }

    for source in 0..2 {
        let target = 1 - source;
        let total = staged[target]
            .state
            .coins
            .checked_add(session.coins[source])
            .filter(|coins| *coins <= MAX_COINS)
            .ok_or(TransferFailure {
                source,
                kind: TransferFailureKind::Invalid,
            })?;
        staged[target].state.coins = total;
        for item in &offered[source] {
            receive_item(&mut staged[target], item.clone())
                .map_err(|kind| TransferFailure { source, kind })?;
        }
    }

    for source in 0..2 {
        if Inventory::validate_state(&staged[source].state).is_err()
            || staged[source].state.items.len() > staged[source].bag_capacity
        {
            // (R) No capacity-specific status exists; report the affected offer as invalid.
            return Err(TransferFailure {
                source: 1 - source,
                kind: TransferFailureKind::Invalid,
            });
        }
    }
    Ok(staged)
}

fn receive_item(
    target: &mut TradeInventory,
    mut incoming: InventoryItem,
) -> Result<(), TransferFailureKind> {
    let definition = *target
        .definitions
        .get(&incoming.definition_id)
        .ok_or(TransferFailureKind::Invalid)?;
    if definition.no_trade {
        return Err(TransferFailureKind::Invalid);
    }
    // (R) Follow Inventory::grant's same-definition/tint, non-rental whole-stack merge rule.
    let fits =
        |count: u32| count <= MAX_COINS && definition.max_stack.is_none_or(|max| count <= max);
    if let Some(existing) = target.state.items.iter_mut().find(|item| {
        item.definition_id == incoming.definition_id
            && item.tint == incoming.tint
            && item.rental.is_none()
            && incoming.rental.is_none()
            && item.count.checked_add(incoming.count).is_some_and(fits)
    }) {
        existing.count += incoming.count;
        return Ok(());
    }
    if !fits(incoming.count) {
        return Err(TransferFailureKind::Stack);
    }
    if target.state.items.len() >= target.bag_capacity {
        return Err(TransferFailureKind::Invalid);
    }
    // (R) Item instance ids are local to this inventory; transfer allocates a new destination id.
    incoming.id = allocate_item_id(&mut target.state).ok_or(TransferFailureKind::Invalid)?;
    target.state.items.push(incoming);
    Ok(())
}

fn allocate_item_id(state: &mut InventoryState) -> Option<i32> {
    let max_live = state.items.iter().map(|item| item.id).max().unwrap_or(0);
    let next = state.next_instance_id.unwrap_or(i64::from(max_live) + 1);
    let mut candidate = if next > i64::from(max_live) && next <= i64::from(i32::MAX) {
        next as i32
    } else {
        1
    };
    for _ in 0..=state.items.len() {
        if !state.items.iter().any(|item| item.id == candidate) {
            state.next_instance_id = Some(i64::from(max_live.max(candidate)) + 1);
            return Some(candidate);
        }
        candidate = if candidate == i32::MAX {
            1
        } else {
            candidate + 1
        };
    }
    None
}

fn invite_packet(
    name: &crate::social_lists::SocialName,
    guid: u64,
    session: u32,
) -> Result<Vec<u8>, TradeError> {
    // Sub 2 reader `00c6cab0`: NameData, sender guid pair, session id.
    let mut out = packet(2);
    write_name(&mut out, name)?;
    out.extend_from_slice(&guid.to_le_bytes());
    out.extend_from_slice(&session.to_le_bytes());
    Ok(out)
}

fn start_packet(
    guid: u64,
    name: &crate::social_lists::SocialName,
    session: u32,
) -> Result<Vec<u8>, TradeError> {
    // Sub 4 reader `00c6cb40`: partner guid pair, NameData, session id.
    let mut out = packet(4);
    out.extend_from_slice(&guid.to_le_bytes());
    write_name(&mut out, name)?;
    out.extend_from_slice(&session.to_le_bytes());
    Ok(out)
}

fn rejected_packet(name: &crate::social_lists::SocialName) -> Result<Vec<u8>, TradeError> {
    // Sub 16 reader `00c6cdb0`: one NameData record.
    let mut out = packet(16);
    write_name(&mut out, name)?;
    Ok(out)
}

fn end_packet(session: u32) -> Vec<u8> {
    // Sub 5 reader `00c6c530`: session id; this is the only window-close message.
    let mut out = packet(5);
    out.extend_from_slice(&session.to_le_bytes());
    out
}

fn status_packet(status: TradeStatus) -> Vec<u8> {
    // Sub 15 reader `00c6c630`: one status byte.
    let mut out = packet(15);
    out.push(status as u8);
    out
}

fn update_coin_packet(session: u32, counter: u32, side: u8, coins: u32) -> Vec<u8> {
    // Sub 11 reader `00c6bf50`: session, counter, side byte, coins.
    let mut out = packet(11);
    out.extend_from_slice(&session.to_le_bytes());
    out.extend_from_slice(&counter.to_le_bytes());
    out.push(side);
    out.extend_from_slice(&coins.to_le_bytes());
    out
}

fn update_add_item_packet(
    session: u32,
    counter: u32,
    side: u8,
    item: &InventoryItem,
) -> Result<Vec<u8>, TradeError> {
    // Sub 12 reader `00c6ce30`: session, counter, side byte, ItemInstance.
    let mut out = packet(12);
    out.extend_from_slice(&session.to_le_bytes());
    out.extend_from_slice(&counter.to_le_bytes());
    out.push(side);
    write_item_record(&mut out, item)?;
    Ok(out)
}

fn update_item_count_packet(
    session: u32,
    counter: u32,
    side: u8,
    item_id: i32,
    quantity: u32,
) -> Result<Vec<u8>, TradeError> {
    // Sub 13 reader `00c6c000`: session, counter, side, item id, resulting quantity.
    let mut out = packet(13);
    out.extend_from_slice(&session.to_le_bytes());
    out.extend_from_slice(&counter.to_le_bytes());
    out.push(side);
    out.extend_from_slice(
        &u32::try_from(item_id)
            .map_err(|_| TradeError::WireRange)?
            .to_le_bytes(),
    );
    out.extend_from_slice(&quantity.to_le_bytes());
    Ok(out)
}

fn update_lock_packet(session: u32, locked: bool) -> Vec<u8> {
    // Sub 17 reader `00c6c110`: session id and lock byte.
    let mut out = packet(17);
    out.extend_from_slice(&session.to_le_bytes());
    out.push(u8::from(locked));
    out
}

fn confirm_packet(session: u32) -> Vec<u8> {
    // Sub 18 reader `00c6c5f0`: session id.
    let mut out = packet(18);
    out.extend_from_slice(&session.to_le_bytes());
    out
}

fn write_item_record(out: &mut Vec<u8>, item: &InventoryItem) -> Result<(), TradeError> {
    // ItemInstance reader `008fa4b0`: definition, tint, guid, count, consumed, cast age,
    // rental flag/block, then stored count. The client guide records the field order.
    let count = i32::try_from(item.count).map_err(|_| TradeError::WireRange)?;
    let consumed = i32::try_from(item.consumed_count).map_err(|_| TradeError::WireRange)?;
    let stored = i32::try_from(item.stored_count).map_err(|_| TradeError::WireRange)?;
    let last_cast = i32::try_from(item.last_cast_time).map_err(|_| TradeError::WireRange)?;
    for value in [
        item.definition_id,
        item.tint,
        item.id,
        count,
        consumed,
        last_cast,
    ] {
        out.extend_from_slice(&value.to_le_bytes());
    }
    match &item.rental {
        Some(RentalState {
            expiration,
            upsells,
            bundle_id,
        }) => {
            out.push(1);
            out.extend_from_slice(&expiration.to_le_bytes());
            out.extend_from_slice(&upsells.to_le_bytes());
            out.extend_from_slice(&bundle_id.to_le_bytes());
        }
        None => out.push(0),
    }
    out.extend_from_slice(&stored.to_le_bytes());
    Ok(())
}

fn write_name(out: &mut Vec<u8>, name: &crate::social_lists::SocialName) -> Result<(), TradeError> {
    // Shared NameData reader `008e7290`: three dwords and two length-prefixed strings.
    // (R) The client's three NameData metadata words have unknown meaning; use neutral zeroes.
    out.extend_from_slice(
        &[0u32; 3]
            .into_iter()
            .flat_map(u32::to_le_bytes)
            .collect::<Vec<_>>(),
    );
    write_string(out, &name.first)?;
    write_string(out, &name.last)
}

fn write_string(out: &mut Vec<u8>, value: &str) -> Result<(), TradeError> {
    let length = i32::try_from(value.len()).map_err(|_| TradeError::WireRange)?;
    out.extend_from_slice(&length.to_le_bytes());
    out.extend_from_slice(value.as_bytes());
    Ok(())
}

fn packet(sub: u8) -> Vec<u8> {
    let mut out = Vec::with_capacity(16);
    out.extend_from_slice(&FAMILY.to_le_bytes());
    out.push(sub);
    out
}
