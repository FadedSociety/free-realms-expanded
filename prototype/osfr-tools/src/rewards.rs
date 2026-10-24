//! Pure reward transaction staging over the existing inventory and profile snapshots.
//!
//! The client readers establish the grant packet families and reward-bundle layout. The server-side
//! grant-key bounds/deduplication semantics, profile-unlock gate, bundle success/default fields,
//! XP-to-stars mapping, packet ordering, default item tint, and bag-slot limit are reconstructed
//! policies (R). The caller persists the returned inventory, profile state, and keys together with
//! its save snapshot.

use crate::{
    inventory::{Inventory, MAX_ITEMS},
    profiles::ProfileState,
    station_cash::{self, Wallet},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fmt, io};

const MAX_GRANT_KEY_BYTES: usize = 256;
const REWARD_FAMILY: u16 = 50;
const REWARD_BUNDLE_SUBTYPE: u8 = 1;
const BASE_ENTRY_BODY_MIN_BYTES: usize = 34;

/// One inventory award. The simple reward API uses tint zero (R).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemGrant {
    pub definition_id: i32,
    pub count: u32,
}

/// A reward-bundle entry's exact serialized body, excluding its leading type id.
///
/// Bodies must follow the client entry reader. For Item and Chest entries, a successful bundle
/// also requires the entry-specific trailing u32 described by reader `008e7930` and its callees.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RewardBundleRow {
    pub type_id: u32,
    pub body: Vec<u8>,
}

/// Display-only fields for family 50/subtype 1. Rows never mutate inventory or progression.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RewardDisplay {
    pub source_guid: u64,
    pub player_guid: u64,
    pub reward_kind: i32,
    pub icon_id: i32,
    pub name_id: i32,
    pub rows: Vec<RewardBundleRow>,
}

/// One atomic server grant. `job_xp` maps profile/job ids to XP amounts. `sc` is the Station Cash
/// kind added by slice 2; it is paid only when the snapshot carries the character's wallet and is
/// capped by the wallet ceiling. Zero SC is omitted from the save form so existing saves and
/// serialized grants stay byte-identical.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RewardGrant {
    pub coins: u32,
    pub items: Vec<ItemGrant>,
    pub job_xp: std::collections::BTreeMap<i32, u32>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub sc: u32,
    pub display: RewardDisplay,
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}

/// Runtime state returned by [`apply_grant`]. Persist `applied_grant_keys` with the inventory and
/// profile snapshot; this module intentionally does not perform persistence. `wallet` is the
/// staged Station Cash wallet and `wallet_ceiling` its data cap; both stay `None`/zero for reward
/// paths that cannot pay Station Cash.
#[derive(Clone)]
pub struct RewardState {
    pub inventory: Inventory,
    pub profiles: ProfileState,
    pub applied_grant_keys: BTreeSet<String>,
    /// Maximum occupied inventory stacks. The original client's bag limit is not recovered (R).
    pub bag_capacity: usize,
    pub wallet: Option<Wallet>,
    pub wallet_ceiling: u32,
}

impl RewardState {
    pub fn new(inventory: Inventory, profiles: ProfileState, bag_capacity: usize) -> Self {
        Self {
            inventory,
            profiles,
            applied_grant_keys: BTreeSet::new(),
            bag_capacity,
            wallet: None,
            wallet_ceiling: 0,
        }
    }

    /// Stage the character's Station Cash wallet and its data ceiling with this transaction.
    pub fn with_wallet(mut self, wallet: Wallet, ceiling: u32) -> Self {
        self.wallet = Some(wallet);
        self.wallet_ceiling = ceiling;
        self
    }
}

/// Why [`apply_grant`] refused a grant. The variant is the classification: a caller matches it
/// (a quest turn-in turns it into a per-player denial), never the message text (T382).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrantFailure {
    /// The bag has no room for the granted stacks.
    BagFull,
    /// The grant pays XP to a profile the character has not unlocked.
    LockedProfile,
    /// A reward item definition is not in the loaded catalogue.
    UnknownItem,
    /// The inventory cannot grant a known reward row: an invalid id or count, more than one stack
    /// holds, or (a server fault) an exhausted item-id counter or an unencodable item.
    BadItem,
    /// A coin or XP amount, or the resulting balance, does not fit its wire field.
    WireRange,
    /// The idempotency key is empty, oversized or padded.
    InvalidKey,
    /// The display bundle has an unsupported, truncated or oversized row.
    Shape,
    /// A Station Cash grant on a snapshot that staged no wallet (a server fault, T229).
    NoWallet,
}

impl fmt::Display for GrantFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::BagFull => "reward exceeds configured bag capacity",
            Self::LockedProfile => "reward XP references a locked profile",
            Self::UnknownItem => "unknown reward item definition",
            Self::BadItem => "invalid reward item",
            Self::WireRange => "reward coins or XP exceed the wire field",
            Self::InvalidKey => "invalid reward grant key",
            Self::Shape => "unsupported or truncated reward bundle entry",
            Self::NoWallet => "Station Cash reward has no wallet",
        })
    }
}

impl std::error::Error for GrantFailure {}

/// `?` callers keep the `InvalidData` error and wording they had before the type existed.
impl From<GrantFailure> for io::Error {
    fn from(failure: GrantFailure) -> Self {
        io::Error::new(io::ErrorKind::InvalidData, failure)
    }
}

/// The first non-zero `job_xp` entry the profile snapshot has not unlocked. `apply_grant`
/// refuses such a grant as [`GrantFailure::LockedProfile`], which `QuestEngine::finish_turnin`
/// turns into a clean refusal for a save written before a quest gained its job XP (R).
fn locked_job_profile(grant: &RewardGrant, profiles: &ProfileState) -> Option<i32> {
    grant.job_xp.iter().find_map(|(&profile_id, &amount)| {
        (amount > 0 && (profile_id <= 0 || !profiles.unlocked.contains(&profile_id)))
            .then_some(profile_id)
    })
}

/// Apply one grant to a clone and return its ordered inner packets. The input is never mutated;
/// persistence and send-after-save ordering remain the caller's responsibility. A refusal is a
/// typed [`GrantFailure`].
pub fn apply_grant(
    state: &RewardState,
    grant: &RewardGrant,
    grant_key: &str,
) -> Result<(RewardState, Vec<Vec<u8>>), GrantFailure> {
    if grant_key.is_empty()
        || grant_key.len() > MAX_GRANT_KEY_BYTES
        || grant_key.trim() != grant_key
    {
        return Err(GrantFailure::InvalidKey);
    }
    if state.applied_grant_keys.contains(grant_key) {
        return Ok((state.clone(), Vec::new()));
    }
    if state.inventory.state().items.len() > state.bag_capacity {
        return Err(GrantFailure::BagFull);
    }
    // Granting only to unlocked jobs is server authorization policy (R).
    if locked_job_profile(grant, &state.profiles).is_some() {
        return Err(GrantFailure::LockedProfile);
    }

    let total_xp = grant
        .job_xp
        .values()
        .try_fold(0u32, |total, amount| total.checked_add(*amount))
        .ok_or(GrantFailure::WireRange)?;
    let coins = i32::try_from(grant.coins).map_err(|_| GrantFailure::WireRange)?;
    let stars = i32::try_from(total_xp).map_err(|_| GrantFailure::WireRange)?;
    let row_count = i32::try_from(grant.display.rows.len()).map_err(|_| GrantFailure::Shape)?;

    let mut next = state.clone();
    let mut outgoing = Vec::new();

    // The guide requires grant messages before the presentation bundle. Their relative order is
    // reconstructed policy (R); rank notification precedes 38/14 per the existing profile writer.
    // `grant_coins` refuses only a balance past the i32 wire cap.
    outgoing.extend(
        next.inventory
            .grant_coins(grant.coins)
            .map_err(|_| GrantFailure::WireRange)?
            .replies,
    );
    for (&profile_id, &amount) in &grant.job_xp {
        if amount == 0 {
            continue;
        }
        let previous = next
            .profiles
            .experience
            .get(&profile_id)
            .copied()
            .unwrap_or(0);
        let updated = previous
            .checked_add(amount)
            .ok_or(GrantFailure::WireRange)?;
        next.profiles.experience.insert(profile_id, updated);
        outgoing.extend(experience_packets(profile_id, previous, updated));
    }
    for item in &grant.items {
        if item.definition_id <= 0 || item.count == 0 {
            return Err(GrantFailure::BadItem);
        }
        if !next.inventory.has_definition(item.definition_id) {
            return Err(GrantFailure::UnknownItem);
        }
        // Bag room is the `bag_capacity` check below. Past the row checks above, `grant` refuses
        // an over-stack row, its item-id counter or encoding (`BadItem`), or a new stack at its
        // `MAX_ITEMS` ceiling, which making room fixes (`BagFull`).
        outgoing.extend(
            next.inventory
                .grant(0, item.definition_id, item.count)
                .map_err(|_| {
                    if next.inventory.state().items.len() >= MAX_ITEMS
                        && next
                            .inventory
                            .accepts_reward(item.definition_id, item.count)
                    {
                        GrantFailure::BagFull
                    } else {
                        GrantFailure::BadItem
                    }
                })?
                .replies,
        );
    }
    if next.inventory.state().items.len() > next.bag_capacity {
        return Err(GrantFailure::BagFull);
    }

    // The Station Cash kind (slice 2): pay into the staged wallet, capped by the data ceiling.
    // Over the ceiling pays up to it and tells the player instead of refusing the whole reward.
    let mut sc_replies = Vec::new();
    if grant.sc > 0 {
        let wallet = next.wallet.as_mut().ok_or(GrantFailure::NoWallet)?;
        let paid = grant
            .sc
            .min(next.wallet_ceiling.saturating_sub(wallet.balance));
        wallet.balance = wallet
            .balance
            .checked_add(paid)
            .ok_or(GrantFailure::WireRange)?;
        // 66/11: client reader 00b3a6e0 -> 00b2b450; the record's first u32 is the balance.
        sc_replies.push(station_cash::wallet_packet(wallet.balance));
        if paid < grant.sc {
            sc_replies.push(crate::chat::system(
                "Station Cash is at its maximum, so the rest of this reward was not paid.",
            ));
        }
    }

    // Reward bundle fields and entry ids follow the body consumed by client reader 008e7930. A
    // Station Cash payout (its transaction carries the wallet) with nothing for that bundle sends
    // none; its wallet packet is the display, and a ride-only payout shows through its item event.
    let station_cash = grant.sc > 0 || next.wallet.is_some();
    if !(station_cash && coins == 0 && stars == 0 && grant.display.rows.is_empty()) {
        outgoing.push(encode_display_bundle(
            coins,
            stars,
            row_count,
            &grant.display,
        )?);
    }
    outgoing.extend(sc_replies);
    next.applied_grant_keys.insert(grant_key.to_owned());
    Ok((next, outgoing))
}

fn experience_packets(profile_id: i32, previous: u32, updated: u32) -> Vec<Vec<u8>> {
    // This 100-XP rank curve and rank-20 cap are reconstructed in profiles.rs (R).
    let rank = (1 + updated / 100).min(20) as i32;
    let old_rank = (1 + previous / 100).min(20) as i32;
    let mut packets = Vec::with_capacity(2);
    if rank > old_rank {
        let mut packet = vec![38, 0, 18, 0];
        packet.extend(profile_id.to_le_bytes());
        packet.extend(rank.to_le_bytes());
        packets.push(packet);
    }
    let progress = if rank == 20 {
        0
    } else {
        (updated % 100) as i32
    };
    let mut packet = vec![38, 0, 14, 0];
    for value in [profile_id, rank, progress, 0, 0, 0] {
        packet.extend(value.to_le_bytes());
    }
    packets.push(packet);
    packets
}

fn encode_display_bundle(
    coins: i32,
    stars: i32,
    row_count: i32,
    display: &RewardDisplay,
) -> Result<Vec<u8>, GrantFailure> {
    for row in &display.rows {
        if !supported_entry(row.type_id) || row.body.len() < BASE_ENTRY_BODY_MIN_BYTES {
            return Err(GrantFailure::Shape);
        }
    }

    // Success=1 and zero unresolved fields/multiplier are server defaults (R).
    let capacity = display
        .rows
        .iter()
        .try_fold(72usize, |total, row| {
            total.checked_add(4)?.checked_add(row.body.len())
        })
        .ok_or(GrantFailure::Shape)?;
    let mut bytes = Vec::with_capacity(capacity);
    bytes.extend(REWARD_FAMILY.to_le_bytes());
    bytes.push(REWARD_BUNDLE_SUBTYPE);
    bytes.push(1);
    for value in [coins, stars, 0, display.reward_kind, 0, 0] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(1.0f32.to_le_bytes());
    bytes.extend(0i32.to_le_bytes());
    bytes.extend(0i32.to_le_bytes());
    bytes.extend(display.source_guid.to_le_bytes());
    bytes.extend(display.player_guid.to_le_bytes());
    bytes.extend(display.icon_id.to_le_bytes());
    bytes.extend(display.name_id.to_le_bytes());
    bytes.extend(row_count.to_le_bytes());
    for row in &display.rows {
        bytes.extend(row.type_id.to_le_bytes());
        bytes.extend(&row.body);
    }
    bytes.extend(0i32.to_le_bytes());
    Ok(bytes)
}

fn supported_entry(type_id: u32) -> bool {
    matches!(
        type_id,
        1 | 3 | 6 | 7 | 8 | 10 | 11 | 12 | 13 | 14 | 15 | 17 | 18 | 19 | 20 | 22 | 23
    )
}
