//! Adventurer exploration/collection progression (work item 038).
//!
//! ORIGINAL facts implemented here, with the recovered source of each:
//! * the four client update packet opcodes, from the original client binary. The
//!   `ClientUpdatePacketCollection*` constructors pass 8, 9, 10 and 11 to `FUN_0089d180`
//!   (`lab/decompiled/client/FreeRealms/pseudocode.c`); 12 is `ClientUpdatePacketUpdateLocation`, so
//!   the family is exactly four wide.
//! * the two reward-bundle entry types that carry collection progress: `RewardBundleEntryCollectionAdd`
//!   returns 10 and formats one `CollectionID`, `RewardBundleEntryCollectionEntryAdd` returns 11 and
//!   formats `CollectionEntryType` plus three parameters.
//! * the UI action and event strings `StartCollection`, `UpdateCollection`, `CompleteCollection`,
//!   `CollectionStarted` and `CollectionCompleted`, the `CollectionCategories.txt` data file with its
//!   `Collection Category ID/NAME/ICON_ID` columns, the `BaseClient.CollectionData` table and its
//!   `Reward` column, and the two counters `CollectionsCompleted` (about-me) and `CollectionCount`
//!   (profile).
//! * the accepted initialization payload
//!   (`lab/working/OSFR.Server/OSFR Server/Customize/PacketSendSelfToClient.json`) delivers 253
//!   collection rows to one profile, every entry carrying a collected flag. Collection progress is
//!   therefore per profile and is delivered inside that profile's own packet.
//!
//! COMMUNITY facts used as the unlock model (wiki pages `Collections`, `Exploration (Collections)`,
//! `List of Collections`, `Adventurer`):
//! * exploration collection entries unlock by approaching the listed location;
//! * rare exploration entries are sparkling coins that must be clicked, and item collection entries
//!   come from sparkling spawns, so proximity alone never unlocks either;
//! * explorer collections take their entries from the listed child collections;
//! * the Adventurer is the default job and is unlocked by finishing the first tutorial quest or
//!   automatically when the tutorial is skipped.
//!
//! RECONSTRUCTED here: the reconciliation rules only - once-only entry, once-only completion,
//! profile isolation and a caller-supplied proximity range. No distance, reward value, experience
//! amount, currency or item is invented, and this module grants nothing: it reports outcomes for the
//! lead-owned economy and wire layers to act on.
//!
//! Explicit gaps, do not imply these:
//! * the proximity radius that activates an exploration entry is not recovered. `approach` takes the
//!   range from the caller and validates it;
//! * captured collection reward rows live in `data/content/collection_catalogue.json` (T447); this
//!   module still grants none of them;
//! * item definition ids for item collection entries live in that catalogue, not here;
//! * the captured payload's unnamed fields keep their positions only; their meaning is unverified.

use std::collections::BTreeMap;

/// ORIGINAL: `ClientUpdatePacketCollectionStart`, constructor argument 8.
pub const CLIENT_UPDATE_COLLECTION_START: u16 = 8;
/// ORIGINAL: `ClientUpdatePacketCollectionRemove`, constructor argument 9.
pub const CLIENT_UPDATE_COLLECTION_REMOVE: u16 = 9;
/// ORIGINAL: `ClientUpdatePacketCollectionAddEntry`, constructor argument 10.
pub const CLIENT_UPDATE_COLLECTION_ADD_ENTRY: u16 = 10;
/// ORIGINAL: `ClientUpdatePacketCollectionRemoveEntry`, constructor argument 11.
pub const CLIENT_UPDATE_COLLECTION_REMOVE_ENTRY: u16 = 11;

/// ORIGINAL: `RewardBundleEntryCollectionAdd` type id.
pub const REWARD_BUNDLE_COLLECTION_ADD: i32 = 10;
/// ORIGINAL: `RewardBundleEntryCollectionEntryAdd` type id.
pub const REWARD_BUNDLE_COLLECTION_ENTRY_ADD: i32 = 11;

/// ORIGINAL: fired when a collection gains its first entry.
pub const ACTION_START_COLLECTION: &str = "StartCollection";
/// ORIGINAL: fired when an entry is added to an already started collection.
pub const ACTION_UPDATE_COLLECTION: &str = "UpdateCollection";
/// ORIGINAL: fired when the last entry of a collection arrives.
pub const ACTION_COMPLETE_COLLECTION: &str = "CompleteCollection";
/// ORIGINAL: the paired display events.
pub const EVENT_COLLECTION_STARTED: &str = "CollectionStarted";
pub const EVENT_COLLECTION_COMPLETED: &str = "CollectionCompleted";

/// ORIGINAL: `CollectionCategories.txt` columns.
pub const CATEGORY_COLUMN_ID: &str = "Collection Category ID";
pub const CATEGORY_COLUMN_NAME: &str = "Collection Category NAME";
pub const CATEGORY_COLUMN_ICON_ID: &str = "Collection Category ICON_ID";

/// ORIGINAL: the about-me field that shows how many collections a profile completed.
pub const ABOUT_ME_COLLECTIONS_COMPLETED: &str = "CollectionsCompleted";
/// ORIGINAL: the profile column carrying the same count.
pub const PROFILE_COLLECTION_COUNT: &str = "CollectionCount";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollectionUpdateOp {
    Start,
    Remove,
    AddEntry,
    RemoveEntry,
}

impl CollectionUpdateOp {
    pub const ALL: [Self; 4] = [Self::Start, Self::Remove, Self::AddEntry, Self::RemoveEntry];

    /// ORIGINAL: the recovered constructor argument, in wire order.
    pub fn opcode(self) -> u16 {
        match self {
            Self::Start => CLIENT_UPDATE_COLLECTION_START,
            Self::Remove => CLIENT_UPDATE_COLLECTION_REMOVE,
            Self::AddEntry => CLIENT_UPDATE_COLLECTION_ADD_ENTRY,
            Self::RemoveEntry => CLIENT_UPDATE_COLLECTION_REMOVE_ENTRY,
        }
    }

    pub fn from_opcode(opcode: u16) -> Option<Self> {
        Self::ALL.into_iter().find(|op| op.opcode() == opcode)
    }
}

/// ORIGINAL: the two reward-bundle entry kinds that describe collection progress.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RewardBundleCollectionEntry {
    /// `CollectionAdd` - the collection itself, one `CollectionID` parameter.
    Completed,
    /// `CollectionEntryAdd` - one entry, `CollectionEntryType` plus three parameters.
    EntryAdded,
}

impl RewardBundleCollectionEntry {
    pub fn type_id(self) -> i32 {
        match self {
            Self::Completed => REWARD_BUNDLE_COLLECTION_ADD,
            Self::EntryAdded => REWARD_BUNDLE_COLLECTION_ENTRY_ADD,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Completed => "CollectionAdd",
            Self::EntryAdded => "CollectionEntryAdd",
        }
    }

    /// Number of `%d` fields in the recovered describe format, so a writer cannot silently drop one.
    pub fn parameter_count(self) -> usize {
        match self {
            Self::Completed => 1,
            Self::EntryAdded => 4,
        }
    }
}

/// COMMUNITY: how an entry is meant to unlock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollectionKind {
    /// Item collection: entries come from sparkling spawns (item definitions are work item 074).
    Item,
    /// Exploration collection: approaching the listed location activates the entry.
    Exploration,
    /// Rare exploration: sparkling coins that must be clicked.
    RareExploration,
    /// Explorer collection: entries are the listed child collections, never collected directly.
    Explorer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpdateOutcome {
    Started,
    EntryAdded,
    EntryRemoved,
    Completed,
    /// Replayed or already-applied update. The state is unchanged.
    DuplicateIgnored,
    /// The collection left this profile's client list; the definition is kept so it can be re-earned.
    CollectionRemoved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnlockOutcome {
    Unlocked,
    OutOfRange,
    /// Proximity never unlocks this entry; a click/interaction is required.
    RequiresInteraction,
    /// Explorer entries follow their child collections.
    CompletedByChildren,
    AlreadyCollected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgressError {
    /// The update or snapshot belongs to a different profile.
    ForeignProfile,
    UnknownCollection,
    UnknownEntry,
    InvalidRange,
}

impl std::fmt::Display for ProgressError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::ForeignProfile => "update belongs to a different profile",
            Self::UnknownCollection => "collection is not defined for this profile",
            Self::UnknownEntry => "entry is not defined for this collection",
            Self::InvalidRange => "distance and range must be finite and not negative",
        };
        formatter.write_str(text)
    }
}

impl std::error::Error for ProgressError {}

#[derive(Clone, Debug)]
struct TrackedCollection {
    kind: CollectionKind,
    /// Entry definition ids in index order. The captured rows carry the same ids with an explicit
    /// index; 19 of those rows arrive out of ascending order and are normalised here.
    entries: Vec<i32>,
    /// Parallel to `entries`, in definition order.
    collected: Vec<bool>,
    started: bool,
}

impl TrackedCollection {
    fn index_of(&self, entry: i32) -> Option<usize> {
        self.entries
            .iter()
            .position(|candidate| *candidate == entry)
    }

    fn collected_count(&self) -> usize {
        self.collected.iter().filter(|flag| **flag).count()
    }

    fn all_entries_collected(&self) -> bool {
        !self.entries.is_empty() && self.collected.iter().all(|flag| *flag)
    }

    fn clear_progress(&mut self) {
        self.collected.iter_mut().for_each(|flag| *flag = false);
        self.started = false;
    }
}

/// One profile's exploration/collection progress.
///
/// Progress is bound to the profile it was created for: an update carrying another player's GUID is
/// refused instead of being merged, which is what keeps per-profile collection state from leaking
/// between characters.
#[derive(Clone, Debug)]
pub struct ProfileCollections {
    player: u64,
    collections: BTreeMap<i32, TrackedCollection>,
    /// Explorer collections whose completion has already been reported, so a completion is emitted
    /// once per completion transition.
    announced: Vec<i32>,
}

impl ProfileCollections {
    pub fn new(player: u64) -> Self {
        Self {
            player,
            collections: BTreeMap::new(),
            announced: Vec::new(),
        }
    }

    pub fn player(&self) -> u64 {
        self.player
    }

    /// Register a collection and its entries in index order. Re-defining an existing collection keeps
    /// the collected flag of every entry id that survives and the started flag. Entry ids are assumed
    /// unique within a collection, which every captured row satisfies.
    pub fn define(&mut self, id: i32, kind: CollectionKind, entry_ids: &[i32]) {
        let previous = self.collections.remove(&id);
        let started = previous
            .as_ref()
            .map(|state| state.started)
            .unwrap_or(false);
        let collected = entry_ids
            .iter()
            .map(|entry| {
                previous
                    .as_ref()
                    .and_then(|state| state.index_of(*entry).map(|index| state.collected[index]))
                    .unwrap_or(false)
            })
            .collect();
        self.collections.insert(
            id,
            TrackedCollection {
                kind,
                entries: entry_ids.to_vec(),
                collected,
                started,
            },
        );
    }

    /// Apply one recovered collection update packet. The profile GUID comes from the packet owner, so
    /// a replayed or misdelivered update cannot move another character's progress.
    pub fn apply(
        &mut self,
        player: u64,
        op: CollectionUpdateOp,
        collection: i32,
        entry: i32,
    ) -> Result<UpdateOutcome, ProgressError> {
        if player != self.player {
            return Err(ProgressError::ForeignProfile);
        }
        let tracked = self
            .collections
            .get_mut(&collection)
            .ok_or(ProgressError::UnknownCollection)?;
        match op {
            CollectionUpdateOp::Start => {
                if tracked.started {
                    Ok(UpdateOutcome::DuplicateIgnored)
                } else {
                    tracked.started = true;
                    Ok(UpdateOutcome::Started)
                }
            }
            CollectionUpdateOp::Remove => {
                tracked.clear_progress();
                self.announced.retain(|id| *id != collection);
                Ok(UpdateOutcome::CollectionRemoved)
            }
            CollectionUpdateOp::AddEntry => {
                let index = tracked.index_of(entry).ok_or(ProgressError::UnknownEntry)?;
                if tracked.collected[index] {
                    return Ok(UpdateOutcome::DuplicateIgnored);
                }
                tracked.collected[index] = true;
                if tracked.all_entries_collected() {
                    Ok(UpdateOutcome::Completed)
                } else {
                    Ok(UpdateOutcome::EntryAdded)
                }
            }
            CollectionUpdateOp::RemoveEntry => {
                let index = tracked.index_of(entry).ok_or(ProgressError::UnknownEntry)?;
                if !tracked.collected[index] {
                    return Ok(UpdateOutcome::DuplicateIgnored);
                }
                tracked.collected[index] = false;
                Ok(UpdateOutcome::EntryRemoved)
            }
        }
    }

    /// COMMUNITY: an exploration entry activates when the player comes within `range` of the
    /// location. The range is the caller's, because no distance is recovered from the client data.
    pub fn approach(
        &mut self,
        collection: i32,
        entry: i32,
        distance: f32,
        range: f32,
    ) -> Result<UnlockOutcome, ProgressError> {
        validate_range(distance, range)?;
        let tracked = self
            .collections
            .get_mut(&collection)
            .ok_or(ProgressError::UnknownCollection)?;
        let index = tracked.index_of(entry).ok_or(ProgressError::UnknownEntry)?;
        match tracked.kind {
            CollectionKind::Explorer => Ok(UnlockOutcome::CompletedByChildren),
            CollectionKind::RareExploration | CollectionKind::Item => {
                Ok(UnlockOutcome::RequiresInteraction)
            }
            CollectionKind::Exploration => {
                if tracked.collected[index] {
                    return Ok(UnlockOutcome::AlreadyCollected);
                }
                if distance <= range {
                    tracked.collected[index] = true;
                    Ok(UnlockOutcome::Unlocked)
                } else {
                    Ok(UnlockOutcome::OutOfRange)
                }
            }
        }
    }

    /// COMMUNITY: the click that collects a sparkling coin or spawn. Exploration entries accept a
    /// direct interaction too; explorer entries never do.
    pub fn collect(&mut self, collection: i32, entry: i32) -> Result<UnlockOutcome, ProgressError> {
        let tracked = self
            .collections
            .get_mut(&collection)
            .ok_or(ProgressError::UnknownCollection)?;
        let index = tracked.index_of(entry).ok_or(ProgressError::UnknownEntry)?;
        if tracked.kind == CollectionKind::Explorer {
            return Ok(UnlockOutcome::CompletedByChildren);
        }
        if tracked.collected[index] {
            return Ok(UnlockOutcome::AlreadyCollected);
        }
        tracked.collected[index] = true;
        Ok(UnlockOutcome::Unlocked)
    }

    /// Explorer collections complete by themselves once every listed child collection is complete.
    /// Returns the explorer collections that just completed, once per transition.
    pub fn resolve_explorer_unlocks(&mut self) -> Vec<i32> {
        let explorers: Vec<i32> = self
            .collections
            .iter()
            .filter(|(_, tracked)| tracked.kind == CollectionKind::Explorer)
            .map(|(id, _)| *id)
            .collect();
        // A child that lost an entry after it was announced can complete again later.
        let stale: Vec<i32> = self
            .announced
            .iter()
            .copied()
            .filter(|id| !self.is_complete(*id))
            .collect();
        self.announced.retain(|id| !stale.contains(id));
        let mut newly_completed = Vec::new();
        for id in explorers {
            if self.announced.contains(&id) || !self.is_complete(id) {
                continue;
            }
            self.announced.push(id);
            newly_completed.push(id);
        }
        newly_completed
    }

    pub fn is_complete(&self, collection: i32) -> bool {
        self.collection_complete(collection, &mut Vec::new())
    }

    /// Stored entry flags, in collection id and definition order. Explorer entries have no stored
    /// flag: their state is derived from the child collections, so they always report `false` here.
    pub fn snapshot(&self) -> Vec<(i32, i32, bool)> {
        let mut rows = Vec::new();
        for (id, tracked) in &self.collections {
            for (entry, collected) in tracked.entries.iter().zip(tracked.collected.iter()) {
                rows.push((*id, *entry, *collected));
            }
        }
        rows
    }

    pub fn collected_entry_count(&self, collection: i32) -> usize {
        self.collections
            .get(&collection)
            .map(TrackedCollection::collected_count)
            .unwrap_or(0)
    }

    /// ORIGINAL counters: this is the value the client reports as `CollectionsCompleted` /
    /// `CollectionCount`.
    pub fn completed_count(&self) -> usize {
        self.collections
            .keys()
            .filter(|id| self.is_complete(**id))
            .count()
    }

    fn collection_complete(&self, collection: i32, visiting: &mut Vec<i32>) -> bool {
        if visiting.contains(&collection) {
            // A definition loop can never be satisfied; refusing it keeps the traversal finite.
            return false;
        }
        let Some(tracked) = self.collections.get(&collection) else {
            return false;
        };
        if tracked.kind != CollectionKind::Explorer {
            return tracked.all_entries_collected();
        }
        if tracked.entries.is_empty() {
            return false;
        }
        visiting.push(collection);
        let complete = tracked
            .entries
            .iter()
            .all(|child| self.collection_complete(*child, visiting));
        visiting.pop();
        complete
    }
}

fn validate_range(distance: f32, range: f32) -> Result<(), ProgressError> {
    let ok = distance.is_finite() && range.is_finite() && distance >= 0.0 && range >= 0.0;
    if ok {
        Ok(())
    } else {
        Err(ProgressError::InvalidRange)
    }
}

/// COMMUNITY: the Adventurer is the default job. It unlocks on the first tutorial quest or when the
/// tutorial is skipped, and never a second time, so no duplicate unlock reward can be granted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TutorialExit {
    FirstQuestCompleted,
    Skipped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdventurerJob {
    unlocked: bool,
}

impl AdventurerJob {
    pub fn locked() -> Self {
        Self { unlocked: false }
    }

    pub fn is_unlocked(&self) -> bool {
        self.unlocked
    }

    /// Returns true only for the call that unlocked the job.
    pub fn apply(&mut self, exit: TutorialExit) -> bool {
        let _ = exit;
        if self.unlocked {
            return false;
        }
        self.unlocked = true;
        true
    }
}
