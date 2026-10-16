//! Piece 057 - loot and enemy rewards: the recovered loot evidence, the pinned community
//! weighted-drop schema and a reconstructed once-only kill/loot grant ledger.
//!
//! Provenance, hashes and the independent re-derivation live in
//! `notes/Research/Pieces/057 Loot and enemy rewards.md` and `scripts/pieces/piece_057.py`.
//!
//! ORIGINAL facts carried here:
//! * The player-update loot sub-opcode 29. The original client binary's
//!   `PlayerUpdatePacketLootEvent` constructor table (`FUN_0089fbb0(0x1d)` in the preserved
//!   Ghidra pseudocode) matches the community `BasePlayerUpdatePackets` enum independently.
//! * The four loot prop model ids from the client-derived model table: 808 `loot_bag_small_01`,
//!   807 `loot_bag_big_01`, 841 `loot_coins_01`, 720 `loot_chest_01`. The asset names are present
//!   in the preserved original client manifest.
//! * The loot toast locale rows 2 ("You receive #count([*item*])") and 3 ("You receive
//!   #count([*experience*]) and #count([*coins*])") and the `Coins` string id 4923.
//! * Five wiki pages that document an enemy drop list. They are recorded verbatim, including the
//!   page id, the revision id and every name that does not resolve to a locale or item row.
//!
//! COMMUNITY, pinned, not historical: the `WeightedDropTable`/`RewardTableDefinition` schema and
//! validation rules (nonempty table, positive weight, positive item quantity, defined currency,
//! lowercase unique key) come from the pinned Sanctuary C# server. Only that schema is mirrored;
//! its `Rewards.json` is a hand-authored dev fixture, not original drop data.
//!
//! RECONSTRUCTED, labelled: the ledger policy - only the credited killer may loot, only a
//! registered defeat may be looted, and a kill is granted at most once - plus the bag/coins/chest
//! prop plan below. The original server-side enemy tables, weights and coin amounts were not
//! recovered, so no enemy list here is ever turned into a weighted table.
//!
//! NOT CLAIMED: any original drop probability, coin/item amount, XP-per-kill value, loot pickup
//! packet layout beyond sub-opcode 29, or native behaviour. This module grants nothing by itself;
//! it returns decisions for the caller that owns inventory, currency and saving.

use std::collections::{BTreeMap, BTreeSet};

/// Original client player-update sub-opcode of `PlayerUpdatePacketLootEvent`.
pub const PLAYER_UPDATE_LOOT_EVENT: u16 = 29;

/// Original model id of `loot_bag_small_01.adr`.
pub const LOOT_BAG_SMALL_MODEL_ID: i32 = 808;
/// Original model id of `loot_bag_big_01.adr`.
pub const LOOT_BAG_BIG_MODEL_ID: i32 = 807;
/// Original model id of `loot_coins_01.adr`.
pub const LOOT_COINS_MODEL_ID: i32 = 841;
/// Original model id of `loot_chest_01.adr`.
pub const LOOT_CHEST_MODEL_ID: i32 = 720;

/// Original locale row "You receive #count([*item*])".
pub const LOOT_TOAST_ITEM_LOCALE_ID: u32 = 2;
/// Original locale row "You receive #count([*experience*]) and #count([*coins*])".
pub const LOOT_TOAST_EXPERIENCE_COINS_LOCALE_ID: u32 = 3;
/// Original locale row "Coins", the currency label the documented lists use.
pub const COINS_NAME_ID: u32 = 4923;

/// A loot prop the reconstructed encounter may spawn at a defeat position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LootProp {
    SmallBag,
    BigBag,
    Coins,
    Chest,
}

impl LootProp {
    /// Original model id recovered from the client-derived model table.
    pub const fn model_id(self) -> i32 {
        match self {
            LootProp::SmallBag => LOOT_BAG_SMALL_MODEL_ID,
            LootProp::BigBag => LOOT_BAG_BIG_MODEL_ID,
            LootProp::Coins => LOOT_COINS_MODEL_ID,
            LootProp::Chest => LOOT_CHEST_MODEL_ID,
        }
    }
}

/// RECONSTRUCTED policy: a normal enemy leaves a small bag plus coins; a boss leaves the chest
/// model. The wiki documents boss drops "from the treasure chest"; the per-enemy mapping to a
/// prop was not captured and stays reconstructed.
pub const fn loot_props(boss: bool) -> &'static [LootProp] {
    if boss {
        &[LootProp::Chest]
    } else {
        &[LootProp::SmallBag, LootProp::Coins]
    }
}

/// What one weighted table row grants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RewardTarget {
    Item {
        item_definition_id: i32,
        quantity: i32,
    },
    Coins {
        amount: i32,
    },
}

/// One row of a weighted drop table. Mirrors the pinned community `RewardDropDefinition`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WeightedDrop {
    pub target: RewardTarget,
    pub weight: i32,
}

impl WeightedDrop {
    pub const fn item(item_definition_id: i32, quantity: i32, weight: i32) -> Self {
        Self {
            target: RewardTarget::Item {
                item_definition_id,
                quantity,
            },
            weight,
        }
    }

    pub const fn coins(amount: i32, weight: i32) -> Self {
        Self {
            target: RewardTarget::Coins { amount },
            weight,
        }
    }
}

/// Why a weighted table or a roll over it is refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableError {
    Empty,
    ZeroWeight { index: usize },
    NonPositiveQuantity { index: usize },
    NonPositiveAmount { index: usize },
    TotalWeightOverflow,
    RollOutOfRange { roll: u64, total: u64 },
}

/// A weighted drop table under the pinned community schema and validation rules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DropTable {
    /// Trimmed and lowercased, exactly like the pinned community loader.
    pub key: String,
    pub drops: Vec<WeightedDrop>,
}

impl DropTable {
    pub fn new(key: &str, drops: Vec<WeightedDrop>) -> Self {
        Self {
            key: key.trim().to_lowercase(),
            drops,
        }
    }

    pub fn validate(&self) -> Result<(), TableError> {
        if self.drops.is_empty() {
            return Err(TableError::Empty);
        }
        for (index, drop) in self.drops.iter().enumerate() {
            if drop.weight <= 0 {
                return Err(TableError::ZeroWeight { index });
            }
            match drop.target {
                RewardTarget::Item { quantity, .. } if quantity <= 0 => {
                    return Err(TableError::NonPositiveQuantity { index });
                }
                RewardTarget::Coins { amount } if amount <= 0 => {
                    return Err(TableError::NonPositiveAmount { index });
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Sum of the positive weights, capped like the pinned community table (`int` max).
    pub fn total_weight(&self) -> Result<u64, TableError> {
        self.validate()?;
        let total: u64 = self.drops.iter().map(|drop| drop.weight as u64).sum();
        if total > i32::MAX as u64 {
            return Err(TableError::TotalWeightOverflow);
        }
        Ok(total)
    }

    /// Walks the cumulative weights for `roll` in `0..total`, exactly like the pinned community
    /// `WeightedDropTable::Select`. The caller supplies the roll so nothing here invents a
    /// distribution.
    pub fn select(&self, roll: u64) -> Result<&WeightedDrop, TableError> {
        let mut remaining = roll;
        let total = self.total_weight()?;
        if roll >= total {
            return Err(TableError::RollOutOfRange { roll, total });
        }
        for drop in &self.drops {
            if remaining < drop.weight as u64 {
                return Ok(drop);
            }
            remaining -= drop.weight as u64;
        }
        Err(TableError::Empty)
    }
}

/// Which documented list entry a name is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentedKind {
    Coins,
    Item,
}

/// One name a wiki page documents as an enemy drop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DocumentedDrop {
    pub label: &'static str,
    pub kind: DocumentedKind,
    /// Locale row of the exact documented text, `None` when the locale export has no such row.
    pub name_id: Option<u32>,
    /// Captured client item definition joined on that name id, `None` when no row joined.
    pub item_definition_id: Option<i32>,
}

/// A wiki-documented enemy drop list: observed names only, with no weight, quantity or amount.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DocumentedDropList {
    pub npc: &'static str,
    pub source_page_id: u32,
    pub source_revision_id: u32,
    pub drops: &'static [DocumentedDrop],
}

/// The exact evidence a documented list is missing before it could become a drop table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LootGap {
    NoProbabilityData,
    NoQuantityData,
    NoAmountData,
}

impl DocumentedDropList {
    /// Every documented list carries the same three unrecovered facts.
    pub const fn blocking_gaps(&self) -> &'static [LootGap] {
        &[
            LootGap::NoProbabilityData,
            LootGap::NoQuantityData,
            LootGap::NoAmountData,
        ]
    }
}

/// Refuses to build a weighted table from a documented list. Kept as a real function so a caller
/// cannot silently invent weights for these enemies.
pub fn weighted_from_documented(_list: &DocumentedDropList) -> Result<DropTable, LootGap> {
    Err(LootGap::NoProbabilityData)
}

const fn coins_entry(name_id: Option<u32>) -> DocumentedDrop {
    DocumentedDrop {
        label: "Coins",
        kind: DocumentedKind::Coins,
        name_id,
        item_definition_id: None,
    }
}

const fn item_entry(
    label: &'static str,
    name_id: Option<u32>,
    item_definition_id: Option<i32>,
) -> DocumentedDrop {
    DocumentedDrop {
        label,
        kind: DocumentedKind::Item,
        name_id,
        item_definition_id,
    }
}

/// COMMUNITY OBSERVATION: the five preserved wiki pages that document enemy drops ("Once
/// defeated, there is a chance to receive the following items"). Names are recorded in page
/// order; `source_revision_id` pins the revision the checker re-derived them from.
pub const COMMUNITY_DOCUMENTED_DROPS: [DocumentedDropList; 5] = [
    DocumentedDropList {
        npc: "Floren Defender",
        source_page_id: 1035,
        source_revision_id: 48775,
        drops: &[
            coins_entry(Some(COINS_NAME_ID)),
            item_entry("Archer's Flair Shard of Forestry", Some(442929), None),
            item_entry("Archer's Power Shard of Durability II", Some(442947), None),
            item_entry("Archer's Power Shard of Recklessness II", None, None),
            item_entry("Archer's Power Shard of Recovery II", None, None),
            item_entry("Brawler's Flair Shard of Forestry", Some(442930), None),
            item_entry("Brawler's Power Shard of Durability II", Some(442948), None),
            item_entry(
                "Brawler's Power Shard of Recklessness II",
                Some(442954),
                None,
            ),
            item_entry("Brawler's Power Shard of Recovery II", Some(442942), None),
            item_entry("Medic's Flair Shard of Forestry", Some(442931), None),
            item_entry("Ninja's Flair Shard of Forestry", Some(442932), None),
            item_entry("Ninja's Power Shard of Recklessness II", Some(442956), None),
            item_entry("Warrior's Flair Shard of Forestry", Some(442933), None),
            item_entry("Wizard's Flair Shard of Forestry", Some(442934), None),
            item_entry("Wilds Small Pillar", Some(442067), None),
            item_entry("Wilds Wall", Some(442064), None),
            item_entry("Wilds Floor", Some(442062), None),
            item_entry("Wilds Ramp", Some(442065), None),
        ],
    },
    DocumentedDropList {
        npc: "Sandscale Magus",
        source_page_id: 2377,
        source_revision_id: 45373,
        drops: &[
            coins_entry(Some(COINS_NAME_ID)),
            item_entry("Manguana Tribal Necklace", Some(8459), Some(18172)),
        ],
    },
    DocumentedDropList {
        npc: "Sandscale Scavenger",
        source_page_id: 2378,
        source_revision_id: 45371,
        drops: &[
            coins_entry(Some(COINS_NAME_ID)),
            item_entry("Blast Spheres", None, None),
            item_entry("Flabbergast Spheres", None, None),
            item_entry("Manguana Tribal Necklace", Some(8459), Some(18172)),
            item_entry("Cloth Wraps", Some(34006), None),
        ],
    },
    DocumentedDropList {
        npc: "Sandspear",
        source_page_id: 2383,
        source_revision_id: 50196,
        drops: &[
            coins_entry(Some(COINS_NAME_ID)),
            item_entry("Manguana Tribal Necklace", Some(8459), Some(18172)),
            item_entry("Spiked Gloves", Some(34012), None),
            item_entry("Dusty Bone Potion", None, None),
            item_entry("Medium Energy Potions", None, None),
        ],
    },
    DocumentedDropList {
        npc: "Skullz Roughrider",
        source_page_id: 2520,
        source_revision_id: 45434,
        drops: &[
            coins_entry(Some(COINS_NAME_ID)),
            item_entry("Skullz Biker Badge", Some(8462), None),
            item_entry("Buckle-strapped Shirt", Some(33988), None),
            item_entry("Coyote Juice Bottle", Some(33952), Some(38822)),
            item_entry("Dusty Bones Potion", Some(33962), Some(38827)),
        ],
    },
];

/// Which defeat a loot grant belongs to. A respawn is a new `spawn_generation`, so the next kill
/// of the same actor is a distinct grant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct KillId {
    pub npc_definition_id: i32,
    pub actor_guid: u64,
    pub spawn_generation: u32,
}

impl KillId {
    pub const fn new(npc_definition_id: i32, actor_guid: u64, spawn_generation: u32) -> Self {
        Self {
            npc_definition_id,
            actor_guid,
            spawn_generation,
        }
    }
}

/// One loot attempt against a registered defeat.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LootRequest {
    pub kill: KillId,
    /// The player the encounter credited with the defeat.
    pub killer: u64,
    /// The player asking for the drop. Group sharing is work item 063, so only the killer is
    /// eligible here.
    pub claimant: u64,
    /// Caller-supplied roll in `0..total_weight`; this module never invents a distribution.
    pub roll: u64,
}

impl LootRequest {
    pub const fn new(kill: KillId, killer: u64, claimant: u64, roll: u64) -> Self {
        Self {
            kill,
            killer,
            claimant,
            roll,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LootOutcome {
    Granted {
        drop: WeightedDrop,
    },
    /// This kill was already looted; repeats never grant again.
    Duplicate,
    NotEligible {
        killer: u64,
        claimant: u64,
    },
    /// No defeat was registered for this kill, so the request cannot be a real kill.
    UnregisteredKill,
    InvalidTable(TableError),
}

/// RECONSTRUCTED policy ledger: defeats are registered once, each kill is looted at most once, and
/// only the credited killer may take the drop.
#[derive(Clone, Debug, Default)]
pub struct LootLedger {
    defeats: BTreeSet<KillId>,
    granted: BTreeMap<KillId, WeightedDrop>,
}

impl LootLedger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a defeat. Returns `false` for a repeat of an already-known kill, which the caller
    /// treats as a duplicate terminal event.
    pub fn register_defeat(&mut self, kill: KillId) -> bool {
        self.defeats.insert(kill)
    }

    pub fn grant(&mut self, request: LootRequest, table: &DropTable) -> LootOutcome {
        if request.claimant != request.killer {
            return LootOutcome::NotEligible {
                killer: request.killer,
                claimant: request.claimant,
            };
        }
        if !self.defeats.contains(&request.kill) {
            return LootOutcome::UnregisteredKill;
        }
        if self.granted.contains_key(&request.kill) {
            return LootOutcome::Duplicate;
        }
        let drop = match table.select(request.roll) {
            Ok(drop) => *drop,
            Err(error) => return LootOutcome::InvalidTable(error),
        };
        self.granted.insert(request.kill, drop);
        LootOutcome::Granted { drop }
    }

    pub fn granted(&self, kill: &KillId) -> Option<WeightedDrop> {
        self.granted.get(kill).copied()
    }

    pub fn granted_kills(&self) -> usize {
        self.granted.len()
    }
}
