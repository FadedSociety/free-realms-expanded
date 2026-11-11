//! F3 (rc-0136, owner: "sellable sc mounts and pets aren't clickable to open the shop menu in the
//! overworld"): the Station Cash ride and pet displays that stand in the world. In the original a
//! click on one opened its shop; here every click answered `unknown_runtime_guid`, because the
//! displays are Legacy extra spawns with no NPC content record.
//!
//! A display is a drawn actor with no content record whose nameplate text id is the name of an
//! item the Marketplace lists (the Gold-Tipped Dragon at the Seaside stables shows ride 76855,
//! named 4893). Its click (26/8) opens the client's Station Cash merchant window: 66/42
//! (`00b48a10` case 42 -> `00b40a60` -> `00b3d5f0`) raises `NpcMerchantV2` in its StationCash mode
//! on that bundle. The window's Buy sends 66/43 (`MerchantSell.BuyFromMerchant` `00c0aab0` ->
//! `00b43310` -> `00b3ab20`), paid by `CoinShop::buy_open_bundle` as a Marketplace order. The open
//! window is the coin shop's, the same open-vendor state a coin merchant's 165/8 reads.
use super::*;

/// (R) How near the display a click must come from, as an NPC click's reach.
const REACH_M: f32 = 12.0;

impl Gameplay {
    /// The 26/8 click on a Station Cash display; `None` for every other actor.
    pub(super) fn sc_display_26_8(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        if !inner.starts_with(&[26, 0, 8, 0]) {
            return Ok(None);
        }
        let Ok(request) = npc_interaction::decode_npc_interact_26_8(inner) else {
            return Ok(None);
        };
        if self.content_npc_key(request.target).is_some() {
            return Ok(None);
        }
        let Some((name, at)) = (self.npc_visibility.as_ref())
            .and_then(|visibility| visibility.actor_name_at(request.target))
        else {
            return Ok(None);
        };
        let Some(shop) = self.npc_coin_shop.as_ref() else {
            return Ok(None);
        };
        let listings: Vec<crate::station_cash::Listing> = (shop.sc_catalogue().listings().iter())
            .filter(|listing| name > 0 && i64::from(listing.name_id) == i64::from(name))
            .copied()
            .collect();
        if listings.is_empty() {
            return Ok(None);
        }
        let reach = at.iter().zip(character.position).map(|(a, b)| (a - b).powi(2)).sum::<f32>();
        if self.busy_interaction_refusal().is_some() || reach > REACH_M * REACH_M {
            npc_event(serde_json::json!({
                "event": "sc_display", "guid": request.target, "outcome": "refused"
            }));
            return Ok(Some(Outcome::default()));
        }
        let entries = (listings.iter())
            .map(|listing| npc_interaction::MerchantEntryV2 {
                item_id: listing.bundle_id,
                icon_id: listing.icon_id,
                tint_value: listing.icon_tint,
                name_id: listing.name_id,
                description_id: listing.description_id,
                quantity_available: 0,
                members_only: false,
                field_7: 0,
                // (R) The SC row's FirstItemId (`MerchantSellSC_DS.FirstItemId`): its one item.
                first_item_id: listing.item_id.cast_unsigned(),
                available_tint_group_id: 0,
                can_buy: true,
            })
            .collect();
        let target = request.target;
        let window = npc_interaction::encode_merchant_list_66_42(&npc_interaction::MerchantListV2 {
            // -1 hides the window's sale timer (`NpcMerchantV2:Show` timeLeft).
            initial_dword: u32::MAX,
            group: npc_interaction::MerchantGroupV2 {
                fields: [now_ms as u32, 0, 0, target as u32, (target >> 32) as u32, 0, 0],
                entries,
            },
        })
        .map_err(|_| invalid("Station Cash display window could not be encoded"))?;
        let bundles: Vec<u32> = listings.iter().map(|listing| listing.bundle_id).collect();
        npc_event(serde_json::json!({
            "event": "sc_display", "guid": target, "outcome": "shop", "bundles": bundles
        }));
        // The window reads its bundles from the Marketplace rows (66/5) every world entry's wallet
        // request (66/10) already brought.
        if let Some(shop) = self.npc_coin_shop.as_mut() {
            shop.open_sc_merchant(bundles);
        }
        Ok(Some(Outcome {
            replies: vec![window],
            ..Outcome::default()
        }))
    }
}
