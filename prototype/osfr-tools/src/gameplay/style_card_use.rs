//! Style cards, part 2: 42/10 UseStyleCard and 42/12 UseStyleCardByItemRecord change the
//! character's look, consume one card, and tell the player and everyone nearby (35/39).
//!
//! The request only names a card the player owns. The slot, mapping and gender come from the
//! card's catalogue row and the strings from the client tables (`style_cards::StyleCard`), so a
//! hostile client can pick which owned card to spend and nothing else. The look lives in the
//! character save (`GameplayState::appearance`); `CharacterIdentity` is never rewritten.
//! 42/11 PreviewStyleCard stays unrouted.
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

    /// 42/10 and 42/12. `None` for every other request. Every refusal is one chat line and
    /// changes nothing: the card, the look and the save stay as they were.
    pub(super) fn handle_style_card(
        &mut self,
        inner: &[u8],
        guid: u64,
    ) -> io::Result<Option<Outcome>> {
        if !matches!(inner.get(..4), Some([42, 0, 10 | 12, 0])) {
            return Ok(None);
        }
        // Like every malformed inventory request, a wrong length ends the session.
        let request = style_cards::decode_request(inner)?;
        let refuse = |text: &str| {
            Ok(Some(Outcome {
                replies: vec![chat::system(text)],
                ..Outcome::default()
            }))
        };
        let (Some(tables), Some(look)) = (self.style_cards.tables.clone(), self.look()) else {
            return refuse("Style cards are not available on this server.");
        };
        let items = &self.inventory.state().items;
        let owned = match request {
            Request::Use(id) => items.iter().find(|item| item.id == id),
            // (R) The lowest-id stack, as Legacy picks the first owned one.
            Request::UseByDefinition(definition) => items
                .iter()
                .filter(|item| item.definition_id == definition)
                .min_by_key(|item| item.id),
            Request::Preview(_) => return Ok(None),
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
        // (R) The same gate as starting an activity.
        if self.blocks(|gates| gates.start) {
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
