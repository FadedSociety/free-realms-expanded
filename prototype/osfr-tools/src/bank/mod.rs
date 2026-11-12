//! P-banking: personal storage on the client's own packets and window (wiki "Banking", added
//! 2012-12-04). A banked unit stays in its item record, whose stored count says how many of its
//! units are in the bank. A deposit or a withdrawal therefore never changes how many units the
//! player owns: a failed save, a disconnect mid-move or a second move can neither lose nor copy
//! an item, and the bank persists with the character save that already holds the bag.
//!
//! Client facts (2014 PC client, `lab/tools/ida-export/ida.py`):
//! * 42/18 opens the window: `sub_991730` reads the two-u16 header and calls `StorageHandler:Show`.
//! * `Show` asks for the numbers through `Ui.RequestStorageUsage` (`RequestStorageCheck`
//!   `00c24370`): 26/44, the bare header (ctor `00c14a80`, writer `00c23150`).
//! * 38/31 (`ClientUpdatePacketStorage`, ctor `0089d5e0`, reader `008d8cb0`) answers: i32 slots
//!   used, then i32 slots owned. Case 0x1f of `009392c0` stores them at +0x88 and +0x84, read by
//!   `Ui.GetStorageUsage` (`00c1a1c0`) and `Ui.GetStorageCap` (`00c1a1f0`), and calls
//!   `StorageHandler:PopulateStorageInfo`.
//! * A move is 42/17 (`Ui.SendItemToStorage` is `MoveItemToStorage` `00c24270`; ctor `00c1a8c0`,
//!   writer `00c20b30`): the player guid (u64), the item instance id (i32, the window's text
//!   through `atoi` `00738e70`) and a signed count. The window's info panel sends a positive
//!   count from the bag and a negative one from storage (`storage.gfx` `InfoPanel.as`).
//! * An item record holds count +0x14, consumed +0x18 and stored +0x1c (reader `008fa4b0`; the
//!   38/3 setters `009904c0`, `00990500`, `00990540`). Storage lists a record with a stored count
//!   (`StorageDataSource__RowPassesFilter` `00a6db90`); the bag one with units left after the
//!   stored and consumed ones, and while the window is open an equipped record keeps one
//!   (`00a6d9b0`).
//! * The Coin Shop's bank upgrades are dynamic-list rows (165/9 handler `00c76100`, row reader
//!   `00c71510`) of item category 62. `CanBuyItem` (`00c70fe0`) wants the row's first flag clear
//!   and fewer copies owned than the definition's stack (`InventoryNodeRemainingCount` `00bdac10`).
//!
//! Reconstructed (R), every value in `data/content/bank.json`: the slots and their order (the
//! wiki's table), which NPCs bank, the Storage Vault item, and the refusal lines below.

use crate::inventory::{BAG_STACKS, InventoryState};
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    fs::File,
    io::{self, Read},
    path::Path,
};

const FILE: &str = "bank.json";
const MAX_FILE_BYTES: u64 = 64 * 1024;
/// The bank upgrades' item category: their definitions' `CategoryId` and the shipped 165/1 rows.
const UPGRADE_CATEGORY: u32 = 62;

/// Why a move or a purchase changed nothing: the chat line the player is shown (R).
pub type Refusal = &'static str;

pub const NOT_OWNED: Refusal = "That item is not in your bag.";
pub const NOTHING: Refusal = "Choose how many to move.";
pub const RENTAL: Refusal = "Rented items cannot go in the bank.";
pub const TOO_MANY: Refusal = "You do not have that many to put in the bank.";
pub const FULL: Refusal = "Your bank is full. Buy a bank upgrade in the Coin Shop for more room.";
pub const TOO_FEW: Refusal = "Your bank holds fewer than that.";
pub const BAG_FULL: Refusal = "Your bag is full. Make room before taking items out of the bank.";
pub const OUT_OF_ORDER: Refusal = "Bank upgrades are bought in order: buy the one before it first.";
pub const OWNED: Refusal = "You already own this bank upgrade.";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rows {
    schema_version: u32,
    #[serde(rename = "source", default)]
    _source: serde::de::IgnoredAny,
    base_slots: u32,
    bankers: Vec<u64>,
    upgrades: Vec<Upgrade>,
    vault_item_id: i32,
}

/// One Coin Shop upgrade: its item and the slots it adds. A `repeatable` one (the 50-slot
/// upgrade, no purchase limit) is outside the order and adds its slots for every copy owned.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Upgrade {
    pub item_id: i32,
    pub slots: u32,
    #[serde(default)]
    pub repeatable: bool,
}

/// The validated `bank.json`.
#[derive(Clone, Debug)]
pub struct Bank {
    base_slots: u32,
    bankers: BTreeSet<u64>,
    /// The ordered ones in the order they must be bought.
    upgrades: Vec<Upgrade>,
    vault: i32,
}

fn invalid(message: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn header(family: u16, opcode: u16) -> Vec<u8> {
    [family.to_le_bytes(), opcode.to_le_bytes()].concat()
}

/// Copies of `item` the player owns, in the bag, the bank or applied.
fn owned(state: &InventoryState, item: i32) -> u32 {
    (state.items.iter())
        .filter(|record| record.definition_id == item)
        .fold(0, |count, record| count.saturating_add(record.count))
}

fn owns(state: &InventoryState, item: i32) -> bool {
    owned(state, item) > 0
}

/// 42/18: open the storage window.
pub fn open_packet() -> Vec<u8> {
    header(42, 18)
}

/// A 42/17 move as `(item instance id, signed count)`. The guid it leads with is not read: the
/// session's own bag is the only one a move can touch.
pub fn decode_move(inner: &[u8]) -> Option<(i32, i32)> {
    if inner.len() != 20 || !inner.starts_with(&[42, 0, 17, 0]) {
        return None;
    }
    Some((crate::le::i32(inner, 12)?, crate::le::i32(inner, 16)?))
}

impl Bank {
    /// `<content>/bank.json`, or `None` when the content set has none (banking stays off).
    pub fn load(content: &Path) -> io::Result<Option<Self>> {
        let file = match File::open(content.join(FILE)) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        let mut bytes = Vec::new();
        file.take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_FILE_BYTES {
            return Err(invalid("bank.json exceeds 64 KiB"));
        }
        let rows: Rows = serde_json::from_slice(&bytes).map_err(invalid)?;
        Self::validate(rows).map(Some)
    }

    fn validate(rows: Rows) -> io::Result<Self> {
        let mut items = BTreeSet::new();
        let total = (rows.upgrades.iter()).try_fold(rows.base_slots, |slots, upgrade| {
            slots.checked_add(upgrade.slots)
        });
        let bad = rows.schema_version != 1
            || rows.base_slots == 0
            || rows.bankers.is_empty()
            || rows.bankers.contains(&0)
            || rows.upgrades.is_empty()
            || (rows.upgrades.iter())
                .any(|u| u.item_id <= 0 || u.slots == 0 || !items.insert(u.item_id))
            || rows.vault_item_id <= 0
            || items.contains(&rows.vault_item_id)
            // The 38/31 capacity is an i32 on the wire.
            || total.is_none_or(|slots| i32::try_from(slots).is_err());
        if bad {
            return Err(invalid("invalid bank.json"));
        }
        Ok(Self {
            base_slots: rows.base_slots,
            bankers: rows.bankers.into_iter().collect(),
            upgrades: rows.upgrades,
            vault: rows.vault_item_id,
        })
    }

    pub fn is_banker(&self, npc: u64) -> bool {
        self.bankers.contains(&npc)
    }

    pub fn is_upgrade(&self, item: i32) -> bool {
        self.upgrades.iter().any(|upgrade| upgrade.item_id == item)
    }

    /// The Storage Vault's item definition.
    pub fn vault(&self) -> i32 {
        self.vault
    }

    /// Slots the player owns: the base plus every upgrade they own (a repeatable one per copy).
    pub fn capacity(&self, state: &InventoryState) -> u32 {
        (self.upgrades.iter()).fold(self.base_slots, |slots, upgrade| {
            let copies = match upgrade.repeatable {
                true => owned(state, upgrade.item_id),
                false => u32::from(owns(state, upgrade.item_id)),
            };
            slots.saturating_add(upgrade.slots.saturating_mul(copies))
        })
    }

    /// Slots in use: one per item and tint with units in storage (the wiki: identical items with
    /// the same tint stack in one slot).
    pub fn slots_used(state: &InventoryState) -> u32 {
        let keys: BTreeSet<_> = (state.items.iter())
            .filter(|record| record.stored_count > 0)
            .map(|record| (record.definition_id, record.tint))
            .collect();
        // A validated bag holds at most `MAX_ITEMS` (4,096) records.
        u32::try_from(keys.len()).unwrap_or(u32::MAX)
    }

    /// 38/31: slots used, then slots owned.
    pub fn usage_packet(&self, state: &InventoryState) -> Vec<u8> {
        let mut packet = header(38, 31);
        for value in [Self::slots_used(state), self.capacity(state)] {
            let wire = i32::try_from(value).unwrap_or(i32::MAX);
            packet.extend(wire.to_le_bytes());
        }
        packet
    }

    /// The next ordered upgrade the player may buy: the first one, in order, they do not own.
    pub fn next_upgrade(&self, state: &InventoryState) -> Option<Upgrade> {
        (self.upgrades.iter())
            .find(|upgrade| !upgrade.repeatable && !owns(state, upgrade.item_id))
            .copied()
    }

    /// Why upgrade `item` may not be bought now, or `None` when it is the next one (a repeatable
    /// upgrade always may be).
    pub fn purchase_refusal(&self, state: &InventoryState, item: i32) -> Option<Refusal> {
        if (self.upgrades.iter()).any(|u| u.item_id == item && u.repeatable) {
            return None;
        }
        match self.next_upgrade(state) {
            Some(next) if next.item_id == item => None,
            _ if owns(state, item) => Some(OWNED),
            _ => Some(OUT_OF_ORDER),
        }
    }

    /// 165/9, the Coin Shop's dynamic list: every upgrade in the shipped 165/1 row shape (the item,
    /// its category, 0), locked by the first flag unless it may be bought now.
    pub fn coin_store_rows(&self, state: &InventoryState) -> Vec<u8> {
        let mut packet = header(0xa5, 9);
        let records = self.coin_store_records(state);
        let count = i32::try_from(records.len()).unwrap_or(i32::MAX);
        packet.extend(count.to_le_bytes());
        for (id, words, flags) in records {
            packet.extend(id.to_le_bytes());
            words.iter().for_each(|word| packet.extend(word.to_le_bytes()));
            packet.extend(flags);
        }
        packet
    }

    /// The upgrades as 165/9 records (id, three dwords, three flags). rc-0137: the client replaces
    /// its whole dynamic list with each 165/9, so a Coin Shop session merges these into its one
    /// list (`CoinShop::dynamic_list_update`); sent alone they wiped the login rows and the open
    /// vendor's stock, and no vendor row could be bought.
    pub fn coin_store_records(&self, state: &InventoryState) -> Vec<(i32, [u32; 3], [u8; 3])> {
        (self.upgrades.iter())
            .map(|upgrade| {
                let locked = self.purchase_refusal(state, upgrade.item_id).is_some();
                let id = upgrade.item_id;
                (id, [id.cast_unsigned(), UPGRADE_CATEGORY, 0], [u8::from(locked), 0, 0])
            })
            .collect()
    }

    /// The bag after moving `quantity` units of record `id`: a positive count into the bank, a
    /// negative one back to the bag. Only the stored count changes.
    pub fn moved(
        &self,
        state: &InventoryState,
        id: i32,
        quantity: i32,
    ) -> Result<InventoryState, Refusal> {
        let index = (state.items.iter())
            .position(|record| record.id == id)
            .ok_or(NOT_OWNED)?;
        let record = &state.items[index];
        if quantity == 0 {
            return Err(NOTHING);
        }
        if record.rental.is_some() {
            return Err(RENTAL);
        }
        let in_bag = (record.count)
            .saturating_sub(record.consumed_count)
            .saturating_sub(record.stored_count);
        let units = quantity.unsigned_abs();
        let mut next = state.clone();
        let stored = &mut next.items[index].stored_count;
        if quantity > 0 {
            // The window lists an equipped record only while it keeps one in the bag (`00a6d9b0`).
            let equipped = state.equipment.iter().any(|entry| entry.item_id == id);
            if units > in_bag.saturating_sub(u32::from(equipped)) {
                return Err(TOO_MANY);
            }
            let key = (record.definition_id, record.tint);
            let has_slot = (state.items.iter())
                .any(|other| other.stored_count > 0 && (other.definition_id, other.tint) == key);
            if !has_slot && Self::slots_used(state) >= self.capacity(state) {
                return Err(FULL);
            }
            *stored += units;
        } else {
            if units > record.stored_count {
                return Err(TOO_FEW);
            }
            if in_bag == 0 && state.bag_stacks() >= BAG_STACKS {
                return Err(BAG_FULL);
            }
            *stored -= units;
        }
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{EquippedItem, InventoryItem};

    fn upgrade(item_id: i32, slots: u32) -> Upgrade {
        Upgrade {
            item_id,
            slots,
            repeatable: false,
        }
    }

    fn bank_rows() -> Rows {
        Rows {
            schema_version: 1,
            _source: serde::de::IgnoredAny,
            base_slots: 2,
            bankers: vec![7],
            upgrades: vec![
                upgrade(50, 1),
                upgrade(51, 3),
            ],
            vault_item_id: 60,
        }
    }

    fn bank() -> Bank {
        Bank::validate(bank_rows()).unwrap()
    }

    fn record(id: i32, definition_id: i32, tint: i32, count: u32) -> InventoryItem {
        InventoryItem {
            definition_id,
            tint,
            id,
            count,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: 0,
            rental: None,
            ability_count: 0,
            activate_enabled: false,
        }
    }

    fn state(items: Vec<InventoryItem>) -> InventoryState {
        InventoryState {
            coins: 0,
            items,
            equipment: Vec::new(),
            next_instance_id: None,
        }
    }

    #[test]
    fn a_move_changes_only_the_stored_count_and_never_the_owned_units() {
        let bank = bank();
        let before = state(vec![record(1, 10, 0, 5)]);
        let after = bank.moved(&before, 1, 3).unwrap();
        assert_eq!((after.items[0].count, after.items[0].stored_count), (5, 3));
        let back = bank.moved(&after, 1, -3).unwrap();
        assert_eq!(back, before);
        assert_eq!(bank.moved(&before, 1, 6), Err(TOO_MANY));
        assert_eq!(bank.moved(&after, 1, -4), Err(TOO_FEW));
        assert_eq!(bank.moved(&after, 1, i32::MIN), Err(TOO_FEW));
        assert_eq!(bank.moved(&before, 1, 0), Err(NOTHING));
        assert_eq!(bank.moved(&before, 2, 1), Err(NOT_OWNED));
    }

    #[test]
    fn one_slot_per_item_and_tint_and_a_full_bank_refuses_a_new_one() {
        let bank = bank();
        let mut bag = state(vec![
            record(1, 10, 0, 1),
            record(2, 10, 0, 1),
            record(3, 10, 4, 1),
            record(4, 11, 0, 1),
        ]);
        for id in [1, 2] {
            bag = bank.moved(&bag, id, 1).unwrap();
        }
        assert_eq!(Bank::slots_used(&bag), 1, "same item, same tint: one slot");
        bag = bank.moved(&bag, 3, 1).unwrap();
        assert_eq!(Bank::slots_used(&bag), 2, "another tint takes its own slot");
        assert_eq!(bank.moved(&bag, 4, 1), Err(FULL));
        bag.items.push(record(5, 50, 0, 1));
        assert_eq!(bank.capacity(&bag), 3, "an owned upgrade adds its slots");
        assert!(bank.moved(&bag, 4, 1).is_ok());
    }

    #[test]
    fn an_equipped_record_keeps_one_and_a_rental_stays_out() {
        let bank = bank();
        let mut bag = state(vec![record(1, 10, 0, 2), record(2, 11, 0, 1)]);
        bag.equipment.push(EquippedItem {
            profile_id: 1,
            slot: 1,
            item_id: 1,
            category: 1,
        });
        assert_eq!(bank.moved(&bag, 1, 2), Err(TOO_MANY));
        assert!(bank.moved(&bag, 1, 1).is_ok());
        bag.items[1].rental = Some(crate::inventory::RentalState {
            expiration: 1,
            upsells: 0,
            bundle_id: 0,
        });
        assert_eq!(bank.moved(&bag, 2, 1), Err(RENTAL));
    }

    #[test]
    fn upgrades_unlock_one_at_a_time() {
        let bank = bank();
        let mut bag = state(Vec::new());
        assert_eq!(bank.purchase_refusal(&bag, 51), Some(OUT_OF_ORDER));
        assert_eq!(bank.purchase_refusal(&bag, 50), None);
        let rows = bank.coin_store_rows(&bag);
        // header, count, then (id, id, 62, 0, flags) per row: the first unlocked, the second locked.
        assert_eq!(&rows[..8], &[0xa5, 0, 9, 0, 2, 0, 0, 0]);
        assert_eq!(rows[8 + 16], 0);
        assert_eq!(rows[8 + 19 + 16], 1);
        bag.items.push(record(1, 50, 0, 1));
        assert_eq!(bank.purchase_refusal(&bag, 50), Some(OWNED));
        assert_eq!(bank.purchase_refusal(&bag, 51), None);
        assert_eq!(
            bank.usage_packet(&bag),
            [38, 0, 31, 0, 0, 0, 0, 0, 3, 0, 0, 0]
        );
    }

    #[test]
    fn a_withdrawal_needs_bag_room_only_when_the_stack_comes_back() {
        let bank = bank();
        let mut bag = state(Vec::new());
        for id in 1..=i32::try_from(BAG_STACKS).unwrap() {
            bag.items.push(record(id, 10_000 + id, 0, 1));
        }
        let mut banked = record(0, 10, 0, 2);
        banked.stored_count = 2;
        bag.items.push(banked);
        assert_eq!(bank.moved(&bag, 0, -1), Err(BAG_FULL));
        bag.items.last_mut().unwrap().stored_count = 1;
        assert!(
            bank.moved(&bag, 0, -1).is_ok(),
            "the stack is already in the bag"
        );
    }

    #[test]
    fn a_repeatable_upgrade_is_outside_the_order_and_counts_every_copy() {
        let mut rows = bank_rows();
        rows.upgrades.push(Upgrade {
            item_id: 52,
            slots: 50,
            repeatable: true,
        });
        let bank = Bank::validate(rows).unwrap();
        let mut bag = state(Vec::new());
        assert_eq!(bank.purchase_refusal(&bag, 52), None, "no order, no limit");
        assert_eq!(bank.purchase_refusal(&bag, 51), Some(OUT_OF_ORDER));
        let rows = bank.coin_store_rows(&bag);
        assert_eq!(rows[8 + 2 * 19 + 16], 0, "the repeatable row stays open");
        bag.items.push(record(1, 52, 0, 3));
        assert_eq!(bank.capacity(&bag), 2 + 150);
        assert_eq!(bank.purchase_refusal(&bag, 52), None);
    }

    #[test]
    fn a_broken_table_is_refused() {
        let good = || Rows {
            schema_version: 1,
            _source: serde::de::IgnoredAny,
            base_slots: 25,
            bankers: vec![1],
            upgrades: vec![upgrade(5, 2)],
            vault_item_id: 6,
        };
        assert!(Bank::validate(good()).is_ok());
        let cases: [fn(&mut Rows); 6] = [
            |rows| rows.schema_version = 2,
            |rows| rows.bankers.clear(),
            |rows| {
                rows.upgrades.push(upgrade(5, 1))
            },
            |rows| rows.vault_item_id = 5,
            |rows| rows.upgrades[0].slots = 0,
            |rows| rows.upgrades[0].slots = u32::MAX,
        ];
        for case in cases {
            let mut rows = good();
            case(&mut rows);
            assert!(Bank::validate(rows).is_err());
        }
    }
}
