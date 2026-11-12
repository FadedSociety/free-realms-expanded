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
    rewards: Vec<Prize>,
    /// L093: content provenance; the server never reads it.
    #[serde(default)]
    #[allow(dead_code)]
    source: String,
}

/// L086: one prize: a bare item id, or an item for members only and/or one gender (1 boy, 2 girl). Wiki
/// (Mystery Chest page): the Sci-Fi chest's Flying Saucer was a members' prize, and the Chest of Wonders held a
/// boys' Desert Traveler set and a girls' Elegant Desert set.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
enum Prize {
    Item(i32),
    Ruled {
        item: i32,
        #[serde(default)]
        members: bool,
        #[serde(default)]
        gender: Option<i32>,
        /// QF: how many of the item one draw pays.
        #[serde(default = "one")]
        count: u32,
        /// QF: the prize's odds against the others (all 1: equal odds).
        #[serde(default = "one")]
        weight: u32,
    },
}

fn one() -> u32 {
    1
}

impl Prize {
    fn item(self) -> i32 {
        match self {
            Self::Item(item) | Self::Ruled { item, .. } => item,
        }
    }

    fn count(self) -> u32 {
        match self {
            Self::Item(_) => 1,
            Self::Ruled { count, .. } => count,
        }
    }

    fn weight(self) -> u32 {
        match self {
            Self::Item(_) => 1,
            Self::Ruled { weight, .. } => weight,
        }
    }

    /// Whether this player may draw it. An unknown gender (0) draws no gendered prize.
    fn fits(self, member: bool, gender: i32) -> bool {
        match self {
            Self::Item(_) => true,
            Self::Ruled {
                members,
                gender: only,
                ..
            } => (member || !members) && only.is_none_or(|only| only == gender),
        }
    }
}

/// The prizes of a chest this player may draw, in content order.
fn eligible(prizes: &[Prize], member: bool, gender: i32) -> Vec<i32> {
    prizes
        .iter()
        .filter(|prize| prize.fits(member, gender))
        .map(|prize| prize.item())
        .collect()
}

/// One prize drawn by weight among those this player may draw; `draw` picks.
fn draw_prize(prizes: &[Prize], member: bool, gender: i32, draw: u64) -> Option<Prize> {
    let fits: Vec<Prize> = (prizes.iter().copied()).filter(|p| p.fits(member, gender)).collect();
    let total: u64 = fits.iter().map(|prize| u64::from(prize.weight())).sum();
    let mut pick = draw.checked_rem(total)?;
    for prize in fits {
        let weight = u64::from(prize.weight());
        if pick < weight {
            return Some(prize);
        }
        pick -= weight;
    }
    None
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    schema_version: u32,
    chests: Vec<Binding>,
    /// QF: gifts, opened the moment they reach the bag (no key, no Use).
    #[serde(default)]
    gifts: Vec<Gift>,
}

/// QF (owner 2026-11-11): a gift item that opens into one weighted draw of its prizes the moment
/// it reaches the bag. The stock client offers no Use for it: the inventory's Use button is drawn
/// only for item type 28 (`inventory` movie `DescriptionPane.equipItem` sends `useItem` for 28
/// only), and the Mystery Gifts are type 6, the type of the client's grab bags (R).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Gift {
    definition_id: i32,
    /// Shown in the chat line that names the opening.
    name: String,
    rewards: Vec<Prize>,
    #[allow(dead_code)]
    source: String,
}

pub struct MysteryChests {
    rewards: BTreeMap<i32, Vec<Prize>>,
    gifts: BTreeMap<i32, (String, Vec<Prize>)>,
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

/// A chest's or gift's prizes: 1 to 64 distinct real items (no chest, no key), gender 1 or 2,
/// count 1 to 1000, weight at least 1.
fn check_prizes(inventory: &Inventory, prizes: &[Prize]) -> io::Result<()> {
    if prizes.is_empty() || prizes.len() > 64 {
        return Err(invalid("invalid chest or reward binding"));
    }
    let mut unique = std::collections::BTreeSet::new();
    for prize in prizes {
        if let Prize::Ruled { gender, count, weight, .. } = prize {
            if gender.is_some_and(|gender| !matches!(gender, 1 | 2)) {
                return Err(invalid("a chest prize's gender is 1 (boy) or 2 (girl)"));
            }
            if !(1..=1000).contains(count) || *weight == 0 {
                return Err(invalid("a prize's count is 1 to 1000 and its weight at least 1"));
            }
        }
        let reward = prize.item();
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
    Ok(())
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
        if document.schema_version != 1
            || document.chests.len() + document.gifts.len() == 0
            || document.chests.len() > 64
            || document.gifts.len() > 64
        {
            return Err(invalid("invalid chest content version or count"));
        }
        if !document.chests.is_empty() {
            inventory.presentation(KEY)?;
        }
        let mut rewards = BTreeMap::new();
        for binding in document.chests {
            if inventory.presentation(binding.definition_id)?.item_type != 28
                || binding.definition_id == KEY
            {
                return Err(invalid("invalid chest or reward binding"));
            }
            check_prizes(inventory, &binding.rewards)?;
            if rewards
                .insert(binding.definition_id, binding.rewards)
                .is_some()
            {
                return Err(invalid("duplicate chest binding"));
            }
        }
        let mut gifts = BTreeMap::new();
        for gift in document.gifts {
            // A type-28 item would be a chest, with the key and the Use button.
            if inventory.presentation(gift.definition_id)?.item_type == 28
                || gift.definition_id == KEY
                || gift.name.trim().is_empty()
                || gift.source.trim().is_empty()
                || rewards.contains_key(&gift.definition_id)
            {
                return Err(invalid("a gift is a named, sourced item that is not a chest"));
            }
            check_prizes(inventory, &gift.rewards)?;
            if gifts.insert(gift.definition_id, (gift.name, gift.rewards)).is_some() {
                return Err(invalid("duplicate gift binding"));
            }
        }
        Ok(Some(Self { rewards, gifts }))
    }

    /// QF: open every gift in `inventory`: each unit is consumed and replaced by one weighted
    /// draw of its prizes, with the chest's 50/1 reward window. `draw` gives the roll for an item
    /// instance and the opening's place in this call. Returns the client replies and one chat
    /// line per opening, per gift a full bag kept, or per opening that failed (a failure changes
    /// nothing). At most 100 openings a call (R).
    pub fn open_gifts(
        &self,
        inventory: &mut Inventory,
        player_guid: u64,
        member: bool,
        gender: i32,
        draw: &dyn Fn(i32, usize) -> u64,
    ) -> (Vec<Vec<u8>>, Vec<String>) {
        let (mut replies, mut lines) = (Vec::new(), Vec::new());
        let held: Vec<(i32, i32)> = (inventory.state().items.iter())
            .filter(|item| item.rental.is_none() && self.gifts.contains_key(&item.definition_id))
            .flat_map(|item| {
                let units = (item.count).saturating_sub(item.consumed_count.saturating_add(item.stored_count));
                std::iter::repeat_n((item.id, item.definition_id), units as usize)
            })
            .take(100)
            .collect();
        for (index, (instance, definition)) in held.into_iter().enumerate() {
            let Some((name, prizes)) = self.gifts.get(&definition) else {
                continue;
            };
            let Some(prize) = draw_prize(prizes, member, gender, draw(instance, index)) else {
                continue;
            };
            match open_gift(inventory, instance, prize, player_guid) {
                Ok(Some(staged)) => {
                    lines.push(format!("You open the {name}."));
                    replies.extend(staged.1);
                    *inventory = staged.0;
                }
                Ok(None) => {
                    lines.push(format!("Make room in your bag to open your {name}."));
                    break;
                }
                Err(error) => lines.push(format!("Your {name} could not be opened: {error}.")),
            }
        }
        (replies, lines)
    }
}

/// One gift instance opened on a staged copy; `None` when the prize would overfill the bag
/// (`BAG_STACKS` shown stacks, or as many as the bag already shows).
fn open_gift(
    inventory: &Inventory,
    instance: i32,
    prize: Prize,
    player_guid: u64,
) -> io::Result<Option<(Inventory, Vec<Vec<u8>>)>> {
    let room = inventory.state().bag_stacks().max(crate::inventory::BAG_STACKS);
    let reward = prize.item();
    let metadata = inventory.presentation(reward)?;
    let mut staged = inventory.clone();
    let mut outcome = staged.consume(instance, 1)?;
    let grant = staged.grant(0, reward, prize.count())?;
    outcome.replies.extend(grant.replies);
    if staged.state().bag_stacks() > room {
        return Ok(None);
    }
    for (granted, _) in grant.granted {
        let tint = (staged.state().items.iter()).find(|row| row.id == granted).map_or(0, |row| row.tint);
        outcome.replies.push(mystery_chest::item_reward(
            u64::try_from(instance).unwrap_or(0),
            player_guid,
            granted,
            reward,
            tint,
            metadata.icon_tint,
            metadata.icon_id,
            metadata.name_id,
        )?);
    }
    Ok(Some((staged, outcome.replies)))
}

/// A gift's roll: unpredictable to the player, from the instance, its place in the call and the
/// clock.
pub fn gift_draw(player_guid: u64, instance: i32, index: usize, unix_nanos: u128) -> u64 {
    let digest = Sha256::new()
        .chain_update(player_guid.to_le_bytes())
        .chain_update(instance.to_le_bytes())
        .chain_update((index as u64).to_le_bytes())
        .chain_update(unix_nanos.to_le_bytes())
        .finalize();
    crate::le::u64(&digest, 0).unwrap_or(0)
}

impl MysteryChests {

    pub fn handle(
        &self,
        inventory: &mut Inventory,
        pending: &mut Option<i32>,
        request: Request,
        player_guid: u64,
        member: bool,
        gender: i32,
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
        // The chest lookup above admits only bound definitions, so the binding is present.
        let choices = self
            .rewards
            .get(&chest.definition_id)
            .map(|prizes| eligible(prizes, member, gender))
            .unwrap_or_default();
        if choices.is_empty() {
            return Ok(InventoryOutcome {
                replies: vec![chat::system("Nothing in this Mystery Chest can be yours.")],
                changed: false,
                ..InventoryOutcome::default()
            });
        }
        // ponytail: reproducible local selection; replace with recovered roll
        // rules when historical distribution and entropy behavior are known.
        let mut hash = Sha256::new();
        hash.update(player_guid.to_le_bytes());
        hash.update(chest.id.to_le_bytes());
        hash.update(chest.count.to_le_bytes());
        hash.update(key.id.to_le_bytes());
        hash.update(key.count.to_le_bytes());
        let digest = hash.finalize();
        let draw =
            crate::le::u64(&digest, 0).ok_or_else(|| invalid("chest draw digest is short"))?;
        // QF: by weight (equal odds when no prize names one, as before).
        let prize = (self.rewards.get(&chest.definition_id))
            .and_then(|prizes| draw_prize(prizes, member, gender, draw))
            .ok_or_else(|| invalid("chest draw found no prize"))?;
        let reward = prize.item();
        let metadata = inventory.presentation(reward)?;
        let mut staged = inventory.clone();
        let mut outcome = staged.consume(chest.id, 1)?;
        outcome.replies.extend(staged.consume(key.id, 1)?.replies);
        outcome.replies.extend(staged.grant(0, reward, prize.count())?.replies);
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

#[cfg(test)]
mod tests {
    use super::*;

    /// L086: members-only and gendered prizes are drawn only by the players they fit, in content order.
    #[test]
    fn prizes_fit_by_membership_and_gender() {
        let prizes: Vec<Prize> = serde_json::from_str(
            r#"[11, {"item": 12, "members": true}, {"item": 13, "gender": 1}, {"item": 14, "gender": 2, "members": true}]"#,
        )
        .unwrap();
        assert_eq!(eligible(&prizes, false, 1), vec![11, 13]);
        assert_eq!(eligible(&prizes, true, 1), vec![11, 12, 13]);
        assert_eq!(eligible(&prizes, true, 2), vec![11, 12, 14]);
        assert_eq!(
            eligible(&prizes, false, 0),
            vec![11],
            "an unknown gender draws no gendered prize"
        );
    }
}
