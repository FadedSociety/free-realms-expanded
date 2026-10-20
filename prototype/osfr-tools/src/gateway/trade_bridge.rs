//! Trade bridges on a session gateway (T230).
//!
//! The hub keeps only dispatch: the trade service's per-request busy answer is mirrored into the
//! shared occupation table here, and the trade commit reads and adopts the session inventory
//! through the same module.

use super::{Gateway, invalid};
use crate::gameplay::Gameplay;
use std::io;

impl Gateway {
    /// T230: clone this session gateway with the player's live trade state mirrored into the
    /// shared occupation table. Derived per message by the server; never saved.
    pub fn staged(&self, trades: &crate::trade::TradeService, guid: u64) -> Self {
        let mut staged = self.clone();
        staged.set_trading(trades.session_id_for(guid).is_some());
        staged
    }

    /// Mirror the trade busy flag into gameplay (session-only; derived per message).
    pub fn set_trading(&mut self, trading: bool) {
        if let Some(gameplay) = self.gameplay.as_mut() {
            gameplay.set_trading(trading);
        }
    }

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
