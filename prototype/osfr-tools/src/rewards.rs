//! Pure reward transaction staging over the existing inventory and profile snapshots.
//!
//! The client readers establish the grant packet families and reward-bundle layout. The server-side
//! grant-key bounds/deduplication semantics, profile-unlock gate, bundle success/default fields,
//! XP-to-stars mapping, packet ordering, default item tint, and bag-slot limit are reconstructed
//! policies (R). The caller persists the returned inventory, profile state, and keys together with
//! its save snapshot.

use crate::{inventory::Inventory, profiles::ProfileState};
use std::{collections::BTreeSet, io};

const MAX_GRANT_KEY_BYTES: usize = 256;
const REWARD_FAMILY: u16 = 50;
const REWARD_BUNDLE_SUBTYPE: u8 = 1;
const BASE_ENTRY_BODY_MIN_BYTES: usize = 34;

/// One inventory award. The simple reward API uses tint zero (R).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemGrant {
    pub definition_id: i32,
    pub count: u32,
}

/// A reward-bundle entry's exact serialized body, excluding its leading type id.
///
/// Bodies must follow the client entry reader. For Item and Chest entries, a successful bundle
/// also requires the entry-specific trailing u32 described by reader `008e7930` and its callees.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardBundleRow {
    pub type_id: u32,
    pub body: Vec<u8>,
}

/// Display-only fields for family 50/subtype 1. Rows never mutate inventory or progression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardDisplay {
    pub source_guid: u64,
    pub player_guid: u64,
    pub reward_kind: i32,
    pub icon_id: i32,
    pub name_id: i32,
    pub rows: Vec<RewardBundleRow>,
}

/// One atomic server grant. `job_xp` maps profile/job ids to XP amounts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardGrant {
    pub coins: u32,
    pub items: Vec<ItemGrant>,
    pub job_xp: std::collections::BTreeMap<i32, u32>,
    pub display: RewardDisplay,
}

/// Runtime state returned by [`apply_grant`]. Persist `applied_grant_keys` with the inventory and
/// profile snapshot; this module intentionally does not perform persistence.
#[derive(Clone)]
pub struct RewardState {
    pub inventory: Inventory,
    pub profiles: ProfileState,
    pub applied_grant_keys: BTreeSet<String>,
    /// Maximum occupied inventory stacks. The original client's bag limit is not recovered (R).
    pub bag_capacity: usize,
}

impl RewardState {
    pub fn new(inventory: Inventory, profiles: ProfileState, bag_capacity: usize) -> Self {
        Self {
            inventory,
            profiles,
            applied_grant_keys: BTreeSet::new(),
            bag_capacity,
        }
    }
}

/// Apply one grant to a clone and return its ordered inner packets. The input is never mutated;
/// persistence and send-after-save ordering remain the caller's responsibility.
pub fn apply_grant(
    state: &RewardState,
    grant: &RewardGrant,
    grant_key: &str,
) -> io::Result<(RewardState, Vec<Vec<u8>>)> {
    if grant_key.is_empty()
        || grant_key.len() > MAX_GRANT_KEY_BYTES
        || grant_key.trim() != grant_key
    {
        return Err(invalid("invalid reward grant key"));
    }
    if state.applied_grant_keys.contains(grant_key) {
        return Ok((state.clone(), Vec::new()));
    }
    if state.inventory.state().items.len() > state.bag_capacity {
        return Err(invalid("inventory exceeds configured bag capacity"));
    }

    let total_xp = grant.job_xp.values().try_fold(0u32, |total, amount| {
        total
            .checked_add(*amount)
            .ok_or_else(|| invalid("reward XP display total overflow"))
    })?;
    let coins = i32::try_from(grant.coins).map_err(|_| invalid("coin award exceeds wire field"))?;
    let stars = i32::try_from(total_xp).map_err(|_| invalid("XP display exceeds wire field"))?;
    let row_count = i32::try_from(grant.display.rows.len())
        .map_err(|_| invalid("reward bundle has too many rows"))?;

    let mut next = state.clone();
    let mut outgoing = Vec::new();

    // The guide requires grant messages before the presentation bundle. Their relative order is
    // reconstructed policy (R); rank notification precedes 38/14 per the existing profile writer.
    outgoing.extend(next.inventory.grant_coins(grant.coins)?.replies);
    for (&profile_id, &amount) in &grant.job_xp {
        if amount == 0 {
            continue;
        }
        // Granting only to unlocked jobs is server authorization policy (R).
        if profile_id <= 0 || !next.profiles.unlocked.contains(&profile_id) {
            return Err(invalid("reward XP references a locked profile"));
        }
        let previous = next
            .profiles
            .experience
            .get(&profile_id)
            .copied()
            .unwrap_or(0);
        let updated = previous
            .checked_add(amount)
            .ok_or_else(|| invalid("profile experience overflow"))?;
        next.profiles.experience.insert(profile_id, updated);
        outgoing.extend(experience_packets(profile_id, previous, updated));
    }
    for item in &grant.items {
        if item.definition_id <= 0 || item.count == 0 {
            return Err(invalid("invalid reward item"));
        }
        if !next.inventory.has_definition(item.definition_id) {
            return Err(invalid("unknown reward item definition"));
        }
        outgoing.extend(
            next.inventory
                .grant(0, item.definition_id, item.count)?
                .replies,
        );
    }
    if next.inventory.state().items.len() > next.bag_capacity {
        return Err(invalid("reward exceeds configured bag capacity"));
    }

    // Reward bundle fields and entry ids follow the body consumed by client reader 008e7930.
    outgoing.push(encode_display_bundle(
        coins,
        stars,
        row_count,
        &grant.display,
    )?);
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
) -> io::Result<Vec<u8>> {
    for row in &display.rows {
        if !supported_entry(row.type_id) || row.body.len() < BASE_ENTRY_BODY_MIN_BYTES {
            return Err(invalid("unsupported or truncated reward bundle entry"));
        }
    }

    // Success=1 and zero unresolved fields/multiplier are server defaults (R).
    let capacity = display.rows.iter().try_fold(72usize, |total, row| {
        total
            .checked_add(4)
            .and_then(|size| size.checked_add(row.body.len()))
            .ok_or_else(|| invalid("reward bundle size overflow"))
    })?;
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

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
