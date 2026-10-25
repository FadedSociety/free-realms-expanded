//! T525: live coin authoring (`realm coin place|move|unbind|list`), Admin-only by the
//! existing realm role gate (`account_store::can_run_realm_command`, unknown commands are
//! Admin).
//!
//! A command writes one overlay row (`collection_coin_edits.json`, beside
//! `npc_placements.json`) through `character_store::replace_bytes`. The row holds the actor
//! and its collection binding together, so a coin and the entry it ticks can never disagree;
//! the shipped `collection_coins.json` is never rewritten in game.
//!
//! The client sees overlay actors through the existing AddNpc 35/2 reader (`00921a10`: u64
//! actor, name string id, model id, ...) and removals through the existing 35/3 removal
//! (`008d69f0`). No new opcode, field or enum value is introduced.
//!
//! Live exclusion is add-only (`NpcVisibility::exclude_actor_guids`), so `unbind` of a row
//! that replaced a content coin removes the overlay actor at once but the content coin
//! returns only in a new session (R).
//!
//! (R) A placed or moved coin stands at the caller's server-known position and heading; the
//! live placement workflow is our reconstruction, not recovered 2014 server behaviour.
use super::*;
use crate::collection_coins::{
    CoinEdit, EDIT_GUID_BASE, EDIT_GUID_END, EditSource, LIVE_SOURCE, Pairing, Tier,
};

const USAGE: &str = "Usage: realm coin place <entry id> | move <coin guid> | unbind <coin guid> | list [radius 1-200]";
const LIST_RADIUS: f32 = 200.0;

impl Gameplay {
    /// The live-world overlay path, beside `npc_placements.json`.
    pub(super) fn coin_edits_path(&self) -> PathBuf {
        self.live_dir()
            .join(crate::collection_coins::EDITS_FILE_NAME)
    }

    /// Re-read the live overlay and adopt it for this session. A missing file is the
    /// content-only map. Called after `set_world_dir`, so a `--world` session never uses
    /// the content folder's overlay.
    pub(super) fn reload_coin_edits(&mut self) -> io::Result<()> {
        if self.collections.is_none() {
            return Ok(());
        }
        let edits = crate::collection_coins::read_edits(&self.coin_edits_path())?;
        self.commit_coin_edits(edits).map(|_| ())
    }

    /// Apply the overlay when its file stamp changed. One stat per session per second; no
    /// path allocation while the window is closed. World visibility only: never saved.
    pub(super) fn tick_live_coin_edits(&mut self, now_ms: u64, out: &mut Outcome) {
        if self.collections.is_none() || !self.coin_edits_watch.due(now_ms) {
            return;
        }
        let path = self.coin_edits_path();
        let edits = match self.coin_edits_watch.poll_with(
            &path,
            now_ms,
            crate::collection_coins::read_edits,
        ) {
            Ok(Some(edits)) => edits,
            Ok(None) => return,
            Err(error) => return coin_edits_error(error),
        };
        match self.commit_coin_edits(edits) {
            Ok(packets) => out.replies.extend(packets),
            Err(error) => coin_edits_error(error), // keep the old view; one report per stamp
        }
    }

    fn commit_coin_edits(&mut self, edits: Vec<CoinEdit>) -> io::Result<Vec<Vec<u8>>> {
        let Some(world) = self.collections.clone() else {
            return Err(invalid("collection content is not loaded"));
        };
        let next = world.with_edits(edits)?;
        self.install_coin_world(&world, next)
    }

    /// Swap in `next`, add/remove overlay actors and return the 35/2/35/3 packets. Nothing
    /// changes when the view fails: the caller keeps the old world and reports the failure
    /// once for this file stamp.
    fn install_coin_world(
        &mut self,
        old: &collection_pickup::CollectionWorld,
        next: collection_pickup::CollectionWorld,
    ) -> io::Result<Vec<Vec<u8>>> {
        let old_guids: BTreeSet<u64> = old.coins().overlay_guids().collect();
        let new_guids: BTreeSet<u64> = next.coins().overlay_guids().collect();
        let mut visibility = self.npc_visibility.clone();
        let packets = match visibility.as_mut() {
            Some(visibility) => collection_pickup::apply_view(
                visibility,
                &next,
                self.progression_extras.collections(),
                old_guids.difference(&new_guids).copied(),
            )?,
            None => Vec::new(),
        };
        self.collections = Some(Arc::new(next));
        self.npc_visibility = visibility;
        self.coin_edits_watch.observe(&self.coin_edits_path());
        Ok(packets)
    }

    /// `realm coin <place|move|unbind|list>`: the live authoring surface. Zone 0 only, like
    /// `realm place`. Every refusal returns text and changes neither the file nor the view.
    pub(super) fn realm_coin(
        &mut self,
        args: &[&str],
        character: &CharacterState,
        now_ms: u64,
        out: &mut Outcome,
    ) -> io::Result<String> {
        if character.zone_id != crate::world::OPEN_WORLD_ZONE {
            return Ok(
                "Coin authoring works in the main world (zone 0) for now. Nothing changed.".into(),
            );
        }
        match args {
            ["place", entry] => self.coin_place(entry, character, now_ms, out),
            ["move", guid] => self.coin_move(guid, character, now_ms, out),
            ["unbind", guid] => self.coin_unbind(guid, out),
            ["list"] => Ok(self.coin_list(LIST_RADIUS, character)),
            ["list", radius] => match radius.parse::<f32>() {
                Ok(radius) if (1.0..=200.0).contains(&radius) => {
                    Ok(self.coin_list(radius, character))
                }
                _ => Ok(format!("Radius must be 1-200. {USAGE}")),
            },
            _ => Ok(USAGE.into()),
        }
    }

    fn coin_place(
        &mut self,
        entry: &str,
        character: &CharacterState,
        now_ms: u64,
        out: &mut Outcome,
    ) -> io::Result<String> {
        let Ok(entry_id) = entry.parse::<i32>() else {
            return Ok(USAGE.into());
        };
        let Some(world) = self.collections.clone() else {
            return Ok("No collection content is loaded. Nothing changed.".into());
        };
        let Some((_, collection)) = world.catalogue().entry(entry_id) else {
            return Ok(format!("No collection entry {entry_id}. Nothing changed."));
        };
        let Some(tier) = Tier::of_collection(collection.id) else {
            return Ok(format!(
                "Entry {entry_id} is not an exploration entry (collection {}). Nothing changed.",
                collection.id
            ));
        };
        if let Some((bound, _)) = world.coins().coin_for_entry(entry_id) {
            return Ok(format!(
                "Entry {entry_id} already has coin {bound}; realm coin move {bound} moves it. Nothing changed."
            ));
        }
        let Some(model_id) = self.coin_model(&world, collection.id) else {
            return Ok(format!(
                "Collection {} has no placed coin to copy a model from. Nothing changed.",
                collection.id
            ));
        };
        let mut edits = world.coins().edits().to_vec();
        let Some(guid) = next_live_guid(&edits) else {
            return Ok("No live coin GUID left below 132000000000. Nothing changed.".into());
        };
        edits.push(CoinEdit {
            spawn_guid: guid,
            model_id,
            name_string_id: tier.name_string_id(),
            position: character.position,
            heading: character.rotation[0].atan2(character.rotation[2]),
            collection_id: collection.id,
            entry_id,
            tier,
            pairing: Pairing::Live,
            source: EditSource {
                kind: LIVE_SOURCE.into(),
                recorded_ms: now_ms,
                character_guid: character.guid,
            },
            replaces: None,
        });
        if let Err(refusal) = self.save_and_apply(&world, edits, out) {
            return Ok(refusal);
        }
        Ok(format!(
            "Placed coin {guid}: entry {entry_id}, collection {}, {}, model {model_id}, at x {:.1}, y {:.2}, z {:.1}. realm coin unbind {guid} removes it.",
            collection.id,
            tier.name(),
            character.position[0],
            character.position[1],
            character.position[2]
        ))
    }

    fn coin_move(
        &mut self,
        guid: &str,
        character: &CharacterState,
        now_ms: u64,
        out: &mut Outcome,
    ) -> io::Result<String> {
        let Ok(old) = guid.parse::<u64>() else {
            return Ok(USAGE.into());
        };
        let Some(world) = self.collections.clone() else {
            return Ok("No collection content is loaded. Nothing changed.".into());
        };
        let Some(coin) = world.coins().by_guid(old) else {
            return Ok(format!(
                "No coin {old}; realm coin list shows the coins near you. Nothing changed."
            ));
        };
        let (collection_id, entry_id, tier) = (coin.collection_id, coin.entry_id, coin.tier);
        let Some(model_id) = self.coin_model(&world, collection_id) else {
            return Ok(format!(
                "Collection {collection_id} has no placed coin to copy a model from. Nothing changed."
            ));
        };
        let mut edits = world.coins().edits().to_vec();
        // Allocate from the rows as they stand, so a moved coin always gets a new GUID.
        let Some(new) = next_live_guid(&edits) else {
            return Ok("No live coin GUID left below 132000000000. Nothing changed.".into());
        };
        let replaces = match world.coins().edit(old) {
            Some(row) => {
                // Moving a live row keeps hiding the content coin it replaced.
                edits.retain(|edit| edit.spawn_guid != old);
                row.replaces
            }
            None => Some(old),
        };
        edits.push(CoinEdit {
            spawn_guid: new,
            model_id,
            name_string_id: tier.name_string_id(),
            position: character.position,
            heading: character.rotation[0].atan2(character.rotation[2]),
            collection_id,
            entry_id,
            tier,
            pairing: Pairing::Live,
            source: EditSource {
                kind: LIVE_SOURCE.into(),
                recorded_ms: now_ms,
                character_guid: character.guid,
            },
            replaces,
        });
        if let Err(refusal) = self.save_and_apply(&world, edits, out) {
            return Ok(refusal);
        }
        let hidden = if replaces.is_some() {
            " The replaced content coin stays hidden until a new session."
        } else {
            ""
        };
        Ok(format!(
            "Moved coin {old} to new coin {new}: entry {entry_id}, collection {collection_id}, {}, model {model_id}, at x {:.1}, y {:.2}, z {:.1}.{hidden}",
            tier.name(),
            character.position[0],
            character.position[1],
            character.position[2]
        ))
    }

    fn coin_unbind(&mut self, guid: &str, out: &mut Outcome) -> io::Result<String> {
        let Ok(guid) = guid.parse::<u64>() else {
            return Ok(USAGE.into());
        };
        let Some(world) = self.collections.clone() else {
            return Ok("No collection content is loaded. Nothing changed.".into());
        };
        let Some(row) = world.coins().edit(guid) else {
            return Ok(format!(
                "No placed coin {guid}; only live overlay rows can be unbound. Nothing changed."
            ));
        };
        let replaced = row.replaces.is_some();
        let mut edits = world.coins().edits().to_vec();
        edits.retain(|edit| edit.spawn_guid != guid);
        if let Err(refusal) = self.save_and_apply(&world, edits, out) {
            return Ok(refusal);
        }
        let note = if replaced {
            " Its content coin returns in a new session."
        } else {
            ""
        };
        Ok(format!("Removed placed coin {guid}.{note}"))
    }

    fn coin_list(&self, radius: f32, character: &CharacterState) -> String {
        let Some(world) = self.collections.clone() else {
            return "No collection content is loaded.".into();
        };
        let here = character.position;
        let mut rows: Vec<(f32, u64, &crate::collection_coins::Coin)> = world
            .coins()
            .iter()
            .filter_map(|(guid, coin)| {
                let position =
                    world
                        .coins()
                        .edit(guid)
                        .map(|edit| edit.position)
                        .or_else(|| {
                            self.npc_visibility
                                .as_ref()?
                                .authored_spawn(guid)
                                .map(|(_, position)| position)
                        })?;
                let distance = flat_distance(position, here);
                (distance <= radius).then_some((distance, guid, coin))
            })
            .collect();
        rows.sort_by(|left, right| {
            left.0
                .total_cmp(&right.0)
                .then_with(|| left.1.cmp(&right.1))
        });
        let total = rows.len();
        rows.truncate(10);
        if rows.is_empty() {
            return format!("No exploration coins within {radius:.0} m.");
        }
        let list: Vec<String> = rows
            .iter()
            .map(|(distance, guid, coin)| {
                format!(
                    "{guid} entry {} {} {} ({distance:.0} m)",
                    coin.entry_id,
                    coin.tier.name(),
                    coin.pairing.name()
                )
            })
            .collect();
        format!(
            "{total} coins within {radius:.0} m, nearest: {}.",
            list.join("; ")
        )
    }

    /// The model a collection's placed coins use. Live rows carry it; content rows are read
    /// from the loaded actors (their own row, ignoring this viewer's exclusions).
    fn coin_model(
        &self,
        world: &collection_pickup::CollectionWorld,
        collection_id: i32,
    ) -> Option<u32> {
        world.coins().iter().find_map(|(guid, coin)| {
            if coin.collection_id != collection_id {
                return None;
            }
            if let Some(edit) = world.coins().edit(guid) {
                return Some(edit.model_id);
            }
            self.npc_visibility
                .as_ref()?
                .authored_spawn(guid)
                .map(|(model, _)| model)
        })
    }

    /// Validate the whole overlay, write it atomically, then adopt it. A failed write leaves
    /// both the old file and the old view.
    fn save_and_apply(
        &mut self,
        world: &collection_pickup::CollectionWorld,
        edits: Vec<CoinEdit>,
        out: &mut Outcome,
    ) -> Result<(), String> {
        let next = world
            .with_edits(edits.clone())
            .map_err(|error| format!("Could not save the coin: {error}. Nothing changed."))?;
        let bytes = crate::collection_coins::edits_bytes(&edits)
            .map_err(|error| format!("Could not save the coin: {error}. Nothing changed."))?;
        crate::character_store::replace_bytes(&self.coin_edits_path(), &bytes)
            .map_err(|error| format!("Could not save the coin: {error}. Nothing changed."))?;
        match self.install_coin_world(world, next) {
            Ok(packets) => out.replies.extend(packets),
            Err(error) => coin_edits_error(error), // saved; the watch retries the view
        }
        Ok(())
    }
}

/// The next live GUID above every row currently in the file.
fn next_live_guid(edits: &[CoinEdit]) -> Option<u64> {
    let next = edits
        .iter()
        .map(|edit| edit.spawn_guid)
        .max()
        .map_or(EDIT_GUID_BASE, |guid| guid.saturating_add(1));
    (next < EDIT_GUID_END).then_some(next)
}

fn coin_edits_error(error: io::Error) {
    npc_event(serde_json::json!({
        "event": "coin_edits_watch_error",
        "error": error.to_string(),
    }));
}
