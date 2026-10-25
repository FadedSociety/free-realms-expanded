//! Style cards: 42/10 UseStyleCard and 42/12 UseStyleCardByItemRecord change the character's
//! look, consume one card, and tell the player and everyone nearby (35/39); 42/11
//! PreviewStyleCard is the store try-on, answered to its sender only with preview 1 and saved
//! nowhere. World entry sends 35/65 so the client has the committed set a closed preview
//! reverts to (`00bd8a00`; see `notes/Autopilot/T520.md`).
//!
//! A use request only names a card the player owns. The slot, mapping and gender come from the
//! card's catalogue row and the strings from the client tables (`style_cards::StyleCard`), so a
//! hostile client can pick which owned card to spend and nothing else. The look lives in the
//! character save (`GameplayState::appearance`); `CharacterIdentity` is never rewritten. A
//! preview needs no ownership (it is a store try-on) but changes no state at all.
use super::*;
use crate::style_cards::{self, ClientTables, Look, Request, StyleCard};

/// The look state a Gameplay carries: the saved look, the identity look it overrides, and the
/// tables a card resolves through (`None`: style cards are off on this server).
#[derive(Clone, Default)]
pub(super) struct StyleCards {
    appearance: Option<Look>,
    /// Session-only, set by `apply_identity`; the saved look wins over it.
    base: Option<Look>,
    tables: Option<Arc<ClientTables>>,
}

impl StyleCards {
    pub(super) fn saved(appearance: Option<Look>) -> Self {
        Self {
            appearance,
            ..Self::default()
        }
    }

    pub(super) fn saved_look(&self) -> Option<Look> {
        self.appearance.clone()
    }
}

impl Gameplay {
    pub fn set_style_tables(&mut self, tables: Option<Arc<ClientTables>>) {
        self.style_cards.tables = tables;
    }

    /// The identity's look, remembered for `look`. Called on every login with the identity.
    pub(super) fn remember_identity_look(&mut self, identity: &CharacterIdentity) {
        self.style_cards.base = Some(Look::from_identity(identity));
    }

    /// The character's current look: the saved style-card look, else the identity's. The one
    /// source for self-data, AddPc and the next card. `None` until an identity is applied or a card was used.
    pub fn look(&self) -> Option<Look> {
        let cards = &self.style_cards;
        cards.appearance.clone().or_else(|| cards.base.clone())
    }

    /// The 35/65 that installs the client's committed customization set: the current look as
    /// the eight records the client's replace-all (`00bd9730`) accepts. World entry only; a
    /// closed 42/11 preview reverts to this set (`00bd8a00`). `None` when style cards are off
    /// or the look is not ready, so a session without them receives nothing.
    pub(super) fn customization_data(&self) -> Option<Vec<u8>> {
        let tables = self.style_cards.tables.as_ref()?;
        Some(style_cards::customization_data(&self.look()?, &**tables))
    }

    /// 42/10, 42/11 and 42/12. `None` for every other request. A use refusal is one chat line;
    /// a preview refusal sends nothing. Every refusal changes nothing: the card, the look and
    /// the save stay as they were.
    pub(super) fn handle_style_card(
        &mut self,
        inner: &[u8],
        guid: u64,
    ) -> io::Result<Option<Outcome>> {
        if !matches!(inner.get(..4), Some([42, 0, 10..=12, 0])) {
            return Ok(None);
        }
        // Like every malformed inventory request, a wrong length ends the session.
        let request = style_cards::decode_request(inner)?;
        // 42/11 PreviewStyleCard is the store try-on: any type 12 definition that fits the current
        // look, owned or not, answered to the sender only. Every refusal sends nothing and
        // changes nothing (no card is spent, the saved look is untouched).
        if let Request::Preview(definition) = request {
            let Some(tables) = self.style_cards.tables.as_ref() else {
                return Ok(Some(Outcome::default()));
            };
            let Some(look) = self.look() else {
                return Ok(Some(Outcome::default()));
            };
            let record = self
                .inventory
                .definition_record(definition)
                .ok_or(style_cards::Refusal::NotAStyleCard)
                .and_then(StyleCard::from_row)
                .and_then(|card| card.record(&look, &**tables, 0));
            let replies = match record {
                Ok(record) => vec![style_cards::customization_change(guid, true, &[record])],
                Err(_) => Vec::new(),
            };
            return Ok(Some(Outcome {
                replies,
                ..Outcome::default()
            }));
        }
        let refuse = |text: &str| {
            Ok(Some(Outcome {
                replies: vec![chat::system(text)],
                ..Outcome::default()
            }))
        };
        let Some(tables) = self.style_cards.tables.clone() else {
            return refuse("Style cards are not available on this server.");
        };
        // No identity applied and no saved look: nothing to restyle yet.
        let Some(look) = self.look() else {
            return refuse("Your character's look is not ready yet.");
        };
        let items = &self.inventory.state().items;
        let owned = match request {
            Request::Use(id) => items.iter().find(|item| item.id == id),
            // (R) The lowest-id stack, as Legacy picks the first owned one.
            Request::UseByDefinition(definition) => items
                .iter()
                .filter(|item| item.definition_id == definition)
                .min_by_key(|item| item.id),
            // Every preview returned above.
            Request::Preview(_) => None,
        };
        let Some(&crate::inventory::InventoryItem {
            id: item_id,
            definition_id,
            ..
        }) = owned
        else {
            return refuse("You do not have that style card.");
        };
        let card = match self
            .inventory
            .definition_record(definition_id)
            .ok_or(style_cards::Refusal::NotAStyleCard)
            .and_then(StyleCard::from_row)
        {
            Ok(card) => card,
            Err(refusal) => return refuse(refusal.message()),
        };
        // (R) The same gate as starting an activity, except inside the player's own lot.
        if self.blocks(|gates| gates.style_card) {
            return refuse("Finish what you are doing before using a style card.");
        }
        let record = match card.record(&look, &*tables, item_id) {
            Ok(record) => record,
            Err(refusal) => return refuse(refusal.message()),
        };
        let new_look = look.apply(&record);
        // (R) OSFR spends the card anyway; we keep it.
        if new_look == look {
            return refuse("You already look like that.");
        }
        // Everything that can refuse is behind us; `consume` stages and commits atomically.
        let before = self.inventory.state().clone();
        let consumed = self.inventory.consume(item_id, 1)?;
        self.style_cards.appearance = Some(new_look);
        let packet = style_cards::customization_change(guid, false, &[record]);
        let mut out = Outcome {
            replies: consumed.replies,
            nearby: vec![packet.clone()],
            changed: true,
            ..Outcome::default()
        };
        out.replies.push(packet);
        self.npc_inventory_events(&before, &mut out)?;
        Ok(Some(out))
    }
}
