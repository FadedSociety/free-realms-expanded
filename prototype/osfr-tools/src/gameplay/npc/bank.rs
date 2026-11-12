//! P-banking's gameplay side: a banker or Storage Vault click, the storage numbers, a move and
//! the Coin Shop's bank upgrades. The rules and packet shapes are `crate::bank`'s; this file
//! routes, checks reach and the busy rule, and commits the bag. Every reply is in the client
//! shapes `crate::bank` cites.
//!
//! ponytail: `bank.json` is read per bank packet (a few KiB, human-paced clicks and moves), so an
//! edit is live at once and no `Gameplay` field grows. A broken file turns banking off and says
//! why in the NPC log; it never ends a session.
use super::*;
use crate::bank::{self, Bank};
use crate::coin_shop::{
    RESULT_INSUFFICIENT_COINS, RESULT_INVALID_ITEM, RESULT_INVALID_QUANTITY,
    RESULT_SERVICE_UNAVAILABLE, RESULT_SUCCESS, transaction_complete,
};

/// (R) The line a first bank visit hands the Storage Vault over with.
const VAULT_GIVEN: &str =
    "The banker hands you a Storage Vault. Place it in your house to reach your bank from home.";

impl Gameplay {
    /// One bank packet: 26/8 on a banker or a placed vault, 26/44, 42/17, a 165/4 for a bank
    /// upgrade and 165/8. `None` for everything else and when the content has no `bank.json`.
    pub(super) fn bank_packet(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
    ) -> io::Result<Option<Outcome>> {
        let Some(&[family, 0, opcode, 0]) = inner.get(..4) else {
            return Ok(None);
        };
        if !matches!((family, opcode), (26, 8 | 44) | (42, 17) | (0xa5, 4 | 8)) {
            return Ok(None);
        }
        let Some(bank) = self.bank_rules() else {
            return Ok(None);
        };
        let reply = |packet: Vec<u8>| Outcome {
            replies: vec![packet],
            ..Outcome::default()
        };
        Ok(match (family, opcode) {
            (26, 8) => self.bank_click(&bank, inner, character)?,
            (26, 44) => Some(reply(bank.usage_packet(self.inventory.state()))),
            (42, 17) => Some(self.bank_move(&bank, inner)),
            (0xa5, 4) => self.bank_upgrade_buy(&bank, inner),
            _ => Some(reply(self.coin_store_list(&bank))),
        })
    }

    /// rc-0137: the one 165/9 the client keeps (it replaces its whole dynamic list): the bank's
    /// upgrades merged into the Coin Shop's login rows and the open vendor's stock. The bank's rows
    /// alone (run 20261112T020333Z: 11 rows) left 271 of Andrea Motorspark's 280 rows unbuyable.
    fn coin_store_list(&self, bank: &Bank) -> Vec<u8> {
        let state = self.inventory.state();
        match self.npc_coin_shop.as_ref() {
            Some(shop) => shop.dynamic_list_update(&bank.coin_store_records(state)),
            None => bank.coin_store_rows(state),
        }
    }

    fn bank_rules(&self) -> Option<Bank> {
        Bank::load(&self.content_dir).unwrap_or_else(|error| {
            npc_event(
                serde_json::json!({"event": "bank_content_refused", "error": error.to_string()}),
            );
            None
        })
    }

    /// A banker in reach, or a Storage Vault standing in the lot the player is in, opens storage.
    fn bank_click(
        &mut self,
        bank: &Bank,
        inner: &[u8],
        character: &CharacterState,
    ) -> io::Result<Option<Outcome>> {
        let Ok(request) = npc_interaction::decode_npc_interact_26_8(inner) else {
            return Ok(None);
        };
        let target = request.target;
        let banker = self
            .content_npc_key(target)
            .filter(|key| bank.is_banker(*key));
        if banker.is_none() && self.lot_fixture_item(target) != Some(bank.vault()) {
            return Ok(None);
        }
        let event = |outcome: &str| {
            npc_event(serde_json::json!({
                "event": "bank_open", "guid": target, "content_key": banker, "outcome": outcome
            }));
        };
        if let Some(refusal) = self.busy_interaction_refusal() {
            event("busy");
            return Ok(Some(refusal));
        }
        // (R) A vault opens from anywhere in the lot; a banker needs the ordinary NPC reach.
        let reach = |key: u64| {
            (self.npc_interaction.as_ref())
                .is_some_and(|npcs| npcs.in_reach(key, character.zone_id, character.position))
        };
        if banker.is_some_and(|key| !reach(key)) {
            event("out_of_range");
            return Ok(Some(Outcome::default()));
        }
        // T543: another actor's window ends the open conversation.
        self.quest_conversation = None;
        let mut out = Outcome::default();
        self.give_vault_once(bank, &mut out);
        out.replies.push(bank.usage_packet(self.inventory.state()));
        out.replies.push(bank::open_packet());
        event("open");
        Ok(Some(out))
    }

    /// The free economy's vault (R, `bank.json`): the first storage visit of a character that owns
    /// none, in the bag or standing in a lot, is given one. A full bag waits for the next visit.
    fn give_vault_once(&mut self, bank: &Bank, out: &mut Outcome) {
        let vault = bank.vault();
        let state = self.inventory.state();
        if !self.inventory.has_definition(vault)
            || state.items.iter().any(|item| item.definition_id == vault)
            || self.has_placed(vault)
            || state.bag_stacks() >= crate::inventory::BAG_STACKS
        {
            return;
        }
        // `grant` is staged inside: a refusal leaves the bag exactly as it was.
        if let Ok(granted) = self.inventory.grant(0, vault, 1) {
            out.replies.extend(granted.replies);
            out.replies.push(chat::system(VAULT_GIVEN));
            out.granted_items.extend(granted.granted);
            out.changed = true;
        }
    }

    /// 42/17: the units move between the bag and the bank inside their record, or nothing moves
    /// and the player is told why. A malformed body is ignored.
    fn bank_move(&mut self, bank: &Bank, inner: &[u8]) -> Outcome {
        let mut out = Outcome::default();
        let Some((item, quantity)) = bank::decode_move(inner) else {
            return out;
        };
        let moved = (bank.moved(self.inventory.state(), item, quantity)).and_then(|next| {
            self.inventory
                .adopt_state(next)
                .map_err(|_| bank::NOT_OWNED)
        });
        let outcome = match moved {
            Ok(replies) => {
                out.replies = replies;
                out.replies.push(bank.usage_packet(self.inventory.state()));
                out.changed = true;
                "moved"
            }
            Err(reason) => {
                out.replies.push(chat::system(reason));
                "refused"
            }
        };
        npc_event(serde_json::json!({
            "event": "bank_move", "item": item, "quantity": quantity, "outcome": outcome
        }));
        out
    }

    /// 165/4 for a bank upgrade: the next one in order, at its definition's coin price, applied at
    /// once. The record stays (the capacity reads it and the client's `CanBuyItem` counts it) but
    /// leaves the bag as consumed. Any other item is the Coin Shop's.
    fn bank_upgrade_buy(&mut self, bank: &Bank, inner: &[u8]) -> Option<Outcome> {
        let item = crate::le::i32(inner, 16).filter(|_| inner.len() == 28)?;
        if !bank.is_upgrade(item) {
            return None;
        }
        let read = |at| crate::le::u32(inner, at).unwrap_or(0);
        let (transaction, quantity) = (read(12), read(24));
        let merchant = crate::le::u64(inner, 4).unwrap_or(0);
        let complete = |result, price| {
            transaction_complete(1, result, transaction, item, 0, quantity, price, merchant)
        };
        let refuse = |result, price, line: &str| Outcome {
            replies: vec![complete(result, price), chat::system(line)],
            ..Outcome::default()
        };
        if let Some(reason) = bank.purchase_refusal(self.inventory.state(), item) {
            return Some(refuse(RESULT_INVALID_ITEM, 0, reason));
        }
        if quantity != 1 {
            return Some(refuse(
                RESULT_INVALID_QUANTITY,
                0,
                "Bank upgrades are bought one at a time.",
            ));
        }
        let Some(price) = self.inventory.definition_coin_price(item) else {
            return Some(refuse(
                RESULT_SERVICE_UNAVAILABLE,
                0,
                "This upgrade is not for sale.",
            ));
        };
        let mut staged = self.inventory.clone();
        let Ok(spent) = staged.spend_coins(price) else {
            return Some(refuse(
                RESULT_INSUFFICIENT_COINS,
                price,
                "You need more coins for this upgrade.",
            ));
        };
        let mut replies = spent.replies;
        let granted = match staged.grant(0, item, 1) {
            Ok(granted) => granted,
            Err(_) => {
                return Some(refuse(
                    RESULT_SERVICE_UNAVAILABLE,
                    price,
                    "Your bag is full.",
                ));
            }
        };
        replies.extend(granted.replies);
        let mut applied = staged.state().clone();
        (applied.items.iter_mut())
            .filter(|record| record.definition_id == item)
            .for_each(|record| record.consumed_count = record.count);
        match staged.adopt_state(applied) {
            Ok(update) => replies.extend(update),
            Err(_) => {
                return Some(refuse(
                    RESULT_SERVICE_UNAVAILABLE,
                    price,
                    "This upgrade is not for sale.",
                ));
            }
        }
        self.inventory = staged;
        let state = self.inventory.state();
        replies.push(complete(RESULT_SUCCESS, price));
        replies.push(bank.usage_packet(state));
        replies.push(self.coin_store_list(bank));
        let line = format!("Your bank now holds {} slots.", bank.capacity(state));
        replies.push(chat::system(&line));
        npc_event(serde_json::json!({"event": "bank_upgrade", "item": item, "price": price}));
        Some(Outcome {
            replies,
            changed: true,
            granted_items: granted.granted,
            ..Outcome::default()
        })
    }
}
