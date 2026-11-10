//! P-games-window: the shared minigame window's extras on `Gameplay`: the offer's Marketplace
//! panel (`crate::offer_boosts`) and the loading screens (`crate::splash_screens`).
use super::occupation::Occupation;
use super::*;
use crate::{
    consumable_effects::{Boosted, Kind},
    offer_boosts::Watch,
    splash_screens,
};

#[derive(Clone)]
pub(super) struct Extras {
    pub(super) boosts: Watch,
    splash: Option<Arc<splash_screens::Table>>,
    /// The occupation that closed the start gate when the open offer went out: the game that
    /// made it. The offer sells only while that game, and no other, still holds the gate.
    offered_by: Option<Occupation>,
}

impl Extras {
    pub(super) fn load(content: &Path) -> io::Result<Self> {
        Ok(Self {
            boosts: Watch::load(content)?,
            splash: splash_screens::Table::load(content)?.map(Arc::new),
            offered_by: None,
        })
    }

    /// Ship combat's tutorial movie per dock; none without the table or while its switch is off.
    pub(super) fn dock_movies(&self, scene: &str, colours: &[String]) -> Vec<String> {
        (self.splash.as_ref()).map_or_else(Vec::new, |table| table.dock_movies(scene, colours, 0))
    }
}

impl Gameplay {
    /// The events framework's hook: the keys of the events running now. An event's ads
    /// (`offer_boosts.json` `event_ads`) join every later offer, and its screens
    /// (`splash_screens.json` `event:<key>`) join [`Self::loading_screen_xml`]. Nothing calls
    /// this yet: the events piece will.
    pub fn set_live_events(&mut self, keys: Vec<String>) {
        self.offer_extras.boosts.set_events(keys);
    }

    /// `LoadingScreen.xml` for this content and the live events; `None` without the table. For
    /// the events piece and a launcher hand-off; the server itself has no use for the file.
    pub fn loading_screen_xml(&self) -> Option<String> {
        let table = self.offer_extras.splash.as_ref()?;
        Some(table.loading_screen_xml(self.offer_extras.boosts.events()))
    }

    /// One commit's outgoing packets, each an inner packet after `head` wrapper bytes: every
    /// minigame offer gains its Marketplace panel (see [`Watch::dress`]).
    pub fn dress_offers(
        &mut self,
        replies: &mut Vec<Vec<u8>>,
        head: usize,
        wrap: fn(&[u8]) -> Vec<u8>,
        entered_world: bool,
    ) {
        let before = self.offer_extras.boosts.pending_instance();
        let shop = self.npc_coin_shop.as_ref();
        self.offer_extras.boosts.dress(
            replies,
            head,
            wrap,
            entered_world,
            |bundle| shop.is_some_and(|shop| shop.lists_bundle(bundle)),
            || shop.map(|shop| shop.marketplace_packets()).unwrap_or_default(),
        );
        let after = self.offer_extras.boosts.pending_instance();
        if after != before {
            // A new offer belongs to the game that holds the start gate as it goes out.
            self.offer_extras.offered_by = after.and(self.blocker(|gates| gates.start));
        }
    }

    /// Whether `inner` is a Station Cash preview or order (66/1, 66/3) for bundles the open offer
    /// advertises. Such an order is not refused as busy: the offer is where it is sold. The offer
    /// counts as open only until the client's Go or leave (`Watch::inbound`) or the game's own
    /// closing packet, and only while the game that made it is the one occupation holding the
    /// start gate: a house, a fight, the wheel or any other activity keeps the T230 busy refusal.
    pub(super) fn buys_at_offer(&self, inner: &[u8]) -> bool {
        let preview = match inner.get(..4) {
            Some([66, 0, 1, 0]) => true,
            Some([66, 0, 3, 0]) => false,
            _ => return false,
        };
        let Some(game) = self.offer_extras.offered_by else {
            return false;
        };
        let held_by_the_game_alone = self.is_occupied(game)
            && (Occupation::ALL.iter())
                .all(|other| *other == game || !other.gates().start || !self.is_occupied(*other));
        held_by_the_game_alone
            && crate::station_cash::decode_order(inner, preview).is_ok_and(|order| {
                (self.offer_extras.boosts).advertises(order.details.iter().map(|d| d.bundle_id))
            })
    }

    /// (R) A booster just bought at the offer is used at once, unless a boost of its kind at
    /// least as strong is running: the start screen is modal, so the bag cannot be opened before
    /// Go. The scroll leaves the bag, its recast starts and its effect begins in this one call,
    /// and a stack it emptied is no longer reported as granted.
    pub(super) fn use_bought_boosts(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
        out: &mut Outcome,
    ) -> io::Result<()> {
        if !inner.starts_with(&[66, 0, 3, 0]) || !self.offer_extras.boosts.use_when_bought() {
            return Ok(());
        }
        let Ok(order) = crate::station_cash::decode_order(inner, false) else {
            return Ok(());
        };
        for detail in order.details {
            let Ok(item) = i32::try_from(detail.bundle_id) else {
                continue;
            };
            let Some(spec) = self.item_bar.effect(item).and_then(|effect| effect.spec()) else {
                continue;
            };
            let Kind::Boost { raises, percent } = spec.kind else {
                continue;
            };
            let live = self.consumable_effects.boost();
            let running = match raises {
                Boosted::Stars => live.stars,
                Boosted::Coins => live.coins,
            };
            if running >= percent || self.consumable_effects.admits(&spec).is_err() {
                continue;
            }
            let used = self.inventory.consume_definition(item, 1)?;
            out.replies.extend(used.replies);
            self.item_bar.stamp_use(item, now_ms);
            // `admits` passed just above, so this cannot refuse.
            if let Ok(applied) =
                self.apply_consumable_effect(character.guid, character.position, spec, now_ms)
            {
                out.absorb(applied);
            }
            out.changed = true;
        }
        let held = &self.inventory.state().items;
        out.granted_items.retain(|(id, _)| held.iter().any(|item| item.id == *id));
        Ok(())
    }
}
