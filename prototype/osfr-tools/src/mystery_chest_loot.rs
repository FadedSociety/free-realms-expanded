//! Local loot bindings; the original drop distribution was not supplied.
//! Presentation and inventory commits use the recovered chest protocol.
use crate::{
    chat,
    inventory::{Inventory, InventoryOutcome},
    mystery_chest::{self, Request},
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, Read},
    path::Path,
};

const KEY: i32 = 78137;
const MAX_CONTENT: usize = 256 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    definition_id: i32,
    rewards: Vec<i32>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    schema_version: u32,
    chests: Vec<Binding>,
}

pub struct MysteryChests {
    rewards: BTreeMap<i32, Vec<i32>>,
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

impl MysteryChests {
    pub fn load(directory: &Path, inventory: &Inventory) -> io::Result<Option<Self>> {
        let path = directory.join("mystery_chests.json");
        let metadata = match std::fs::symlink_metadata(&path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
            Ok(m) => m,
        };
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(invalid("chest content must be a regular file"));
        }
        let mut bytes = Vec::new();
        File::open(path)?
            .take((MAX_CONTENT + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_CONTENT {
            return Err(invalid("chest content exceeds 256 KiB"));
        }
        let document: Document = serde_json::from_slice(&bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if document.schema_version != 1 || document.chests.is_empty() || document.chests.len() > 64
        {
            return Err(invalid("invalid chest content version or count"));
        }
        inventory.presentation(KEY)?;
        let mut rewards = BTreeMap::new();
        for binding in document.chests {
            if inventory.presentation(binding.definition_id)?.item_type != 28
                || binding.definition_id == KEY
                || binding.rewards.is_empty()
                || binding.rewards.len() > 64
            {
                return Err(invalid("invalid chest or reward binding"));
            }
            let mut unique = std::collections::BTreeSet::new();
            for &reward in &binding.rewards {
                let metadata = inventory.presentation(reward)?;
                if reward == KEY
                    || metadata.item_type == 28
                    || metadata.icon_id <= 0
                    || metadata.name_id <= 0
                    || !unique.insert(reward)
                {
                    return Err(invalid("invalid chest reward definition"));
                }
            }
            if rewards
                .insert(binding.definition_id, binding.rewards)
                .is_some()
            {
                return Err(invalid("duplicate chest binding"));
            }
        }
        Ok(Some(Self { rewards }))
    }

    pub fn handle(
        &self,
        inventory: &mut Inventory,
        pending: &mut Option<i32>,
        request: Request,
        player_guid: u64,
    ) -> io::Result<InventoryOutcome> {
        let id = match request {
            Request::Check(id) | Request::Use(id) => id,
        };
        let chest = inventory
            .state()
            .items
            .iter()
            .find(|i| {
                i.id == id
                    && i.rental.is_none()
                    && i.consumed_count == 0
                    && self.rewards.contains_key(&i.definition_id)
            })
            .cloned();
        let key = inventory
            .state()
            .items
            .iter()
            .find(|i| i.definition_id == KEY && i.rental.is_none() && i.consumed_count == 0)
            .cloned();
        if matches!(request, Request::Check(_)) {
            let success = chest.is_some() && key.is_some();
            *pending = if success { Some(id) } else { None };
            return Ok(InventoryOutcome {
                replies: vec![mystery_chest::check_reply(id, success)],
                changed: false,
                ..InventoryOutcome::default()
            });
        }
        // A Use for another instance must not cancel this chest's confirmation (W90).
        let confirmed = *pending == Some(id);
        if confirmed {
            *pending = None;
        }
        let (Some(chest), Some(key)) = (chest, key) else {
            return Ok(InventoryOutcome {
                replies: vec![chat::system(
                    "Opening requires an owned Mystery Chest and Mystery Key.",
                )],
                changed: false,
                ..InventoryOutcome::default()
            });
        };
        if !confirmed {
            return Ok(InventoryOutcome {
                replies: vec![chat::system("Confirm the Mystery Chest before opening it.")],
                changed: false,
                ..InventoryOutcome::default()
            });
        }
        let choices = &self.rewards[&chest.definition_id];
        // ponytail: reproducible local selection; replace with recovered roll
        // rules when historical distribution and entropy behavior are known.
        let mut hash = Sha256::new();
        hash.update(player_guid.to_le_bytes());
        hash.update(chest.id.to_le_bytes());
        hash.update(chest.count.to_le_bytes());
        hash.update(key.id.to_le_bytes());
        hash.update(key.count.to_le_bytes());
        let digest = hash.finalize();
        let draw = u64::from_le_bytes(digest[..8].try_into().unwrap());
        let reward = choices[(draw % choices.len() as u64) as usize];
        let metadata = inventory.presentation(reward)?;
        let mut staged = inventory.clone();
        let mut outcome = staged.consume(chest.id, 1)?;
        outcome.replies.extend(staged.consume(key.id, 1)?.replies);
        outcome.replies.extend(staged.grant(0, reward, 1)?.replies);
        let item = staged
            .state()
            .items
            .iter()
            .find(|i| {
                i.definition_id == reward
                    && i.tint == 0
                    && i.rental.is_none()
                    && inventory
                        .state()
                        .items
                        .iter()
                        .find(|old| old.id == i.id)
                        .is_none_or(|old| old.count != i.count)
            })
            .ok_or_else(|| invalid("chest grant missing from inventory"))?;
        outcome.replies.push(mystery_chest::item_reward(
            chest.id as u64,
            player_guid,
            item.id,
            reward,
            item.tint,
            metadata.icon_tint,
            metadata.icon_id,
            metadata.name_id,
        )?);
        *inventory = staged;
        Ok(outcome)
    }
}
