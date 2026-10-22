//! T450: exploration coins. A 26/8 click (client constructor `008b6bc0`, sent by `0092efe0`) on a
//! placed coin records its collection entry on the saved ledger, sends the 38/10 AddEntry (reader
//! `008d8570`) and hides the coin for this player for good.
//!
//! No 38/8 Start is sent. AddEntry case 10 finds the login-sent collection (`0x00939e2f` ->
//! `00992bf0`), inserts the entry (`009939a0`) and raises the banners itself: started
//! (`00a93c80`) when `00992ad0` flips, update (`00a93f40`) always, complete (`00a94380`) when
//! `00992b10` sees every entry collected (`0x00939e41`-`0x00939ee9`).
//!
//! (R) Coins are personal: collecting one hides it only for the collector (behaviour of
//! Sanctuary@10f129e `HandleCollectionCoin`, no code). Completion pays nothing yet (empty grant).
use super::*;
use crate::{
    collection_catalogue::CollectionCatalogueContent,
    collection_coins::CollectionCoins,
    harvest_nodes::Denied,
    progression_extras::{
        CollectionEntryUpdate, CollectionEvent, CollectionEventOrigin, ProgressionExtras,
        empty_grant,
    },
    world::{NpcVisibility, OPEN_WORLD_ZONE},
};

/// The collection catalogue and the coins bound to it: read-only content shared by staged clones.
pub(super) struct CollectionWorld {
    catalogue: CollectionCatalogueContent,
    coins: CollectionCoins,
}

/// Reads both files from `dir` (dormant unless both exist) and hides every coin `extras` has
/// already collected, so a relog never sends it.
pub(super) fn load(
    dir: &Path,
    extras: &ProgressionExtras,
    visibility: &mut Option<NpcVisibility>,
) -> io::Result<Option<Arc<CollectionWorld>>> {
    let Some(catalogue) = CollectionCatalogueContent::load(dir)? else {
        return Ok(None);
    };
    let Some(coins) = CollectionCoins::load(dir, &catalogue)? else {
        return Ok(None);
    };
    if let Some(visibility) = visibility {
        let ledger = extras.collections();
        visibility.exclude_actor_guids(
            coins
                .iter()
                .filter(|(_, coin)| ledger.is_collected(coin.entry_id))
                .map(|(guid, _)| guid),
        );
    }
    Ok(Some(Arc::new(CollectionWorld { catalogue, coins })))
}

impl Gameplay {
    /// A 26/8 click on an exploration coin. `None` for any other target, which keeps its NPC
    /// route. Refusals change nothing; only a pickup marks the save changed.
    pub(super) fn handle_collection_coin(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
    ) -> io::Result<Option<Outcome>> {
        if !inner.starts_with(&[26, 0, 8, 0]) || inner.len() < 12 {
            return Ok(None);
        }
        let guid = u64::from_le_bytes(inner[4..12].try_into().unwrap());
        let Some(world) = self.collections.clone() else {
            return Ok(None);
        };
        let Some(coin) = world.coins.by_guid(guid) else {
            return Ok(None);
        };
        npc_interaction::decode_npc_interact_26_8(inner)
            .map_err(|_| invalid("invalid coin interaction request"))?;
        let mut out = Outcome::default();
        let shown = self
            .npc_visibility
            .as_ref()
            .and_then(|v| v.actor_position(guid));
        let collected = self
            .progression_extras
            .collections()
            .is_collected(coin.entry_id);
        let Some(position) = shown.filter(|_| !collected) else {
            return Ok(Some(out)); // collected already, or never shown to this player: silent
        };
        let reach: f32 = position
            .iter()
            .zip(character.position)
            .map(|(a, b)| (a - b).powi(2))
            .sum();
        let denied = if self.activity_busy() {
            Some(Denied::Busy)
        } else if character.zone_id != OPEN_WORLD_ZONE
            || !(reach <= world.coins.pickup_range().powi(2))
        {
            Some(Denied::TooFar)
        } else {
            None
        };
        if let Some(denied) = denied {
            out.replies.push(chat::system(&denied.message()));
            return Ok(Some(out));
        }
        out.replies = self.record_collection_entry(&world, coin.entry_id)?;
        if let Some(visibility) = self.npc_visibility.as_mut() {
            visibility.remove(guid); // the next visibility update sends its 35/3
        }
        out.changed = true;
        Ok(Some(out))
    }

    /// Record `entry_id` on the saved ledger and return its 38/10, built from the catalogue row.
    /// An `Err` leaves the ledger as it was. The grant is empty, so the transition's reward state
    /// is the snapshot unchanged and is not adopted; a paying grant must adopt it.
    fn record_collection_entry(
        &mut self,
        world: &CollectionWorld,
        entry_id: i32,
    ) -> io::Result<Vec<Vec<u8>>> {
        let (entry, collection) = world
            .catalogue
            .entry(entry_id)
            .ok_or_else(|| invalid("collection entry is not in the catalogue"))?;
        // Wire fields are the client's raw dwords; `collect` checks the two ids against the
        // ledger's own join, so a negative id is refused rather than sent.
        let update = CollectionEntryUpdate {
            definition_id: entry.id as u32,
            icon_id: entry.icon_id as u32,
            icon_tint_id: entry.icon_tint_id as u32,
            name_id: entry.name_id as u32,
            collection_id: collection.id as u32,
            index: entry.index,
            unknown: 0,
            collected: true,
        };
        let event = CollectionEvent {
            entry_id,
            origin: CollectionEventOrigin::ExplorationCoin,
            wire_update: Some(update),
        };
        let rewards = self.reward_snapshot();
        let transition = self.progression_extras.collect(
            world.catalogue.catalogue(),
            event,
            &empty_grant(),
            &rewards,
        )?;
        Ok(transition.collection_packets)
    }
}
