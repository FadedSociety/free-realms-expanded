//! Trade bridges on a session gateway (W163/W172).
//!
//! The hub keeps only dispatch; the W163 trade commit reads and adopts the session inventory
//! through this module.

use super::{Gateway, invalid};
use crate::gameplay::Gameplay;
use std::io;

impl Gateway {
    /// See `Gameplay::in_encounter` (W163 player trade).
    pub fn in_encounter(&self) -> bool {
        self.gameplay.as_ref().is_some_and(Gameplay::in_encounter)
    }

    /// See `Gameplay::trade_inventory` (W163 player trade).
    /// See `Gameplay::inventory_state` (trade commit).
    pub fn inventory_state(&self) -> Option<&crate::inventory::InventoryState> {
        self.gameplay.as_ref().map(Gameplay::inventory_state)
    }

    /// See `Gameplay::adopt_trade_inventory` (trade commit).
    pub fn adopt_trade_inventory(
        &mut self,
        after: crate::inventory::InventoryState,
    ) -> io::Result<Vec<Vec<u8>>> {
        self.gameplay
            .as_mut()
            .ok_or_else(|| invalid("no gameplay to trade with"))?
            .adopt_trade_inventory(after)
    }

    pub fn trade_inventory(&self) -> Option<crate::trade::TradeInventory> {
        self.gameplay.as_ref()?.trade_inventory()
    }
}
