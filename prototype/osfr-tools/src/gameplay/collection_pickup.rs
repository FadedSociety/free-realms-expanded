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
//! Sanctuary@10f129e `HandleCollectionCoin`, no code).
//!
//! T480: the click that completes a collection pays its captured reward once, built by
//! `CollectionCatalogueContent::completion_grant` and applied through `collect`'s completion
//! path, so the 38/10 comes first and the reward packets (50/1 bundle, XP) after it. The ledger,
//! bag, coins and XP commit together on a staged copy; a full bag or an XP/coin overflow refuses
//! the whole pickup and the coin stays clickable. The once-only guard is the saved `completed`
//! set (the `collection:<id>` key lives only on the per-click snapshot).
//!
//! T522: a completion also records its meta collection entry (the Elite sets 284-291 tick `Elite
//! Explorer` 309; 299-304 tick 306), mapped at load by `collection_catalogue::meta_map` from the
//! captured `collection_add` targets. Both states commit in the same staged transaction.
use super::*;
use crate::{
    collection_catalogue::CollectionCatalogueContent,
    collection_coins::CollectionCoins,
    harvest_nodes::Denied,
    progression_extras::{
        CollectionEntryUpdate, CollectionEvent, CollectionEventOrigin, ProgressionExtras,
    },
    rewards::GrantFailure,
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
    // A reward item without a definition is never granted; say which, once per load.
    let skipped: BTreeSet<i32> = catalogue
        .collections()
        .iter()
        .flat_map(|collection| &collection.reward.items)
        .filter_map(|item| item.captured_item_definition_id)
        .collect();
    if !skipped.is_empty() {
        npc_event(serde_json::json!({"event": "collection_reward_items_skipped", "ids": skipped}));
    }
    // T522: the accepted meta collections, once per load.
    if !catalogue.meta_pairs().is_empty() {
        let targets: BTreeSet<i32> = catalogue
            .meta_pairs()
            .values()
            .filter_map(|entry_id| {
                catalogue
                    .entry(*entry_id)
                    .map(|(_, collection)| collection.id)
            })
            .collect();
        npc_event(serde_json::json!({
            "event": "collection_meta_targets",
            "targets": targets,
        }));
    }
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

/// Log a refused collection reward once, naming the entry that failed: the clicked source entry,
/// or (T522) the meta entry refused after the source's reward already succeeded.
fn log_refused_entry(entry: i32, error: &io::Error) {
    if let Some(failure) = error
        .get_ref()
        .and_then(|error| error.downcast_ref::<GrantFailure>())
    {
        npc_event(serde_json::json!({
            "event": "collection_reward_refused",
            "entry": entry,
            "reason": failure.to_string(),
        }));
    }
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
        // Stage everything: a refusal or an error leaves `self` exactly as it was.
        let mut staged = self.clone();
        if let Err(error) =
            staged.record_collection_entry(&world, coin.entry_id, character.guid, &mut out)
        {
            let Some(failure) = error
                .get_ref()
                .and_then(|e| e.downcast_ref::<GrantFailure>())
            else {
                return Err(error);
            };
            // A source reward can set `changed` before a later meta entry is refused; the clone
            // was discarded, so the refusal carries none of the staged outcome.
            return Ok(Some(Outcome {
                replies: vec![chat::system(match failure {
                    GrantFailure::BagFull => "Your bag is full. Make room before collecting that.",
                    _ => "You cannot collect that right now.",
                })],
                ..Outcome::default()
            }));
        }
        if let Some(visibility) = staged.npc_visibility.as_mut() {
            visibility.remove(guid); // the next visibility update sends its 35/3
        }
        *self = staged;
        out.changed = true;
        Ok(Some(out))
    }

    /// Record `entry_id` on the saved ledger: its 38/10 first, then, when it completes the
    /// collection, the reward packets, with the paid bag, coins and XP adopted. A completed
    /// collection that maps to a meta collection entry (T522) records that entry too through the
    /// same staged path, and that collection's completion then pays its own reward. An `Err` may
    /// leave `self` half changed; the caller discards the staged copy.
    fn record_collection_entry(
        &mut self,
        world: &CollectionWorld,
        entry_id: i32,
        player_guid: u64,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let collection_id = match self.collect_entry(
            world,
            entry_id,
            CollectionEventOrigin::ExplorationCoin,
            player_guid,
            out,
        ) {
            Ok(collection_id) => collection_id,
            Err(error) => {
                log_refused_entry(entry_id, &error);
                return Err(error);
            }
        };
        let Some(collection_id) = collection_id else {
            return Ok(());
        };
        // One level only: the load drops a pair whose source is another pair's target, and a
        // meta completion is never cascaded again.
        let Some(meta_entry) = world.catalogue.meta_entry(collection_id) else {
            return Ok(());
        };
        if let Err(error) = self.collect_entry(
            world,
            meta_entry,
            CollectionEventOrigin::CollectionBundleEntry,
            player_guid,
            out,
        ) {
            log_refused_entry(meta_entry, &error);
            return Err(error);
        }
        Ok(())
    }

    /// Record one catalogue entry, send its 38/10 and, when it completes its collection, build
    /// and pay the captured reward. The returned id is set only when this event completes that
    /// collection. An `Err` may leave `self` half changed; the staged caller discards it.
    fn collect_entry(
        &mut self,
        world: &CollectionWorld,
        entry_id: i32,
        origin: CollectionEventOrigin,
        player_guid: u64,
        out: &mut Outcome,
    ) -> io::Result<Option<i32>> {
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
            origin,
            wire_update: Some(update),
        };
        let rewards = self.reward_snapshot();
        let grant = world.catalogue.completion_grant(
            collection.id,
            player_guid,
            self.gender,
            &rewards.profiles.unlocked,
            |id| rewards.inventory.has_definition(id),
        );
        let transition = self.progression_extras.collect(
            world.catalogue.catalogue(),
            event,
            &grant,
            &rewards,
        )?;
        out.replies.extend(transition.collection_packets);
        if !transition.reward_packets.is_empty() {
            out.replies.extend(transition.reward_packets);
            self.adopt_reward_state(transition.reward_state, out)?;
        }
        Ok(transition
            .outcome
            .completion
            .map(|completion| completion.collection_id))
    }
}
