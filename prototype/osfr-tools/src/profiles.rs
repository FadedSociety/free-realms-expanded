//! Native class selection backed by verified private profile payloads.
//! Unlock timing and the small opening XP curve are reconstructed local rules.
use crate::inventory::Inventory;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::Path,
    sync::Arc,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileState {
    pub version: u32,
    pub active: i32,
    pub unlocked: BTreeSet<i32>,
    pub experience: BTreeMap<i32, u32>,
}

#[derive(Deserialize)]
struct Profile {
    id: i32,
    offset: usize,
    length: usize,
    sha256: String,
    serialized_hex: String,
    rank_offset: usize,
    equipment_offset: usize,
    equipment_length: usize,
}

#[derive(Deserialize)]
struct Content {
    schema_version: u32,
    active_offset: usize,
    active_sha256: String,
    profiles: Vec<Profile>,
    default_state: ProfileState,
    starter_items: BTreeMap<i32, Vec<i32>>,
}

#[derive(Clone)]
pub struct Profiles {
    content: Arc<Content>,
    state: ProfileState,
}

fn invalid(message: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn decode(text: &str) -> io::Result<Vec<u8>> {
    if text.len() % 2 != 0 || text.len() > 131072 {
        return Err(invalid("invalid profile bytes"));
    }
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let text = std::str::from_utf8(pair).map_err(invalid)?;
            u8::from_str_radix(text, 16).map_err(invalid)
        })
        .collect()
}

impl Profiles {
    pub fn load(directory: &Path, state: Option<ProfileState>) -> io::Result<Self> {
        let bytes = fs::read(directory.join("profiles.json"))?;
        if bytes.len() > 1_048_576 {
            return Err(invalid("profile content exceeds bounds"));
        }
        let content: Content = serde_json::from_slice(&bytes).map_err(invalid)?;
        if content.schema_version != 1 || content.profiles.len() > 32 {
            return Err(invalid("unsupported profile content"));
        }
        let mut ids = BTreeSet::new();
        for profile in &content.profiles {
            let bytes = decode(&profile.serialized_hex)?;
            if profile.length != bytes.len()
                || digest(&bytes) != profile.sha256
                || profile
                    .rank_offset
                    .checked_add(12)
                    .is_none_or(|end| end > bytes.len())
                || profile
                    .equipment_offset
                    .checked_add(profile.equipment_length)
                    .is_none_or(|end| end > bytes.len())
                || !ids.insert(profile.id)
            {
                return Err(invalid("profile proof or bounds differ"));
            }
        }
        let state = state.unwrap_or_else(|| content.default_state.clone());
        if state.version != 1
            || !state.unlocked.contains(&state.active)
            || !state.unlocked.is_subset(&ids)
            || state
                .experience
                .keys()
                .any(|id| !state.unlocked.contains(id))
        {
            return Err(invalid("invalid class progression snapshot"));
        }
        Ok(Self {
            content: Arc::new(content),
            state,
        })
    }

    pub fn state(&self) -> &ProfileState {
        &self.state
    }
    pub fn active(&self) -> i32 {
        self.state.active
    }
    pub fn starter_items(&self, profile: i32) -> &[i32] {
        self.content
            .starter_items
            .get(&profile)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }
    pub fn unlock(&mut self, profile: i32) -> io::Result<bool> {
        if !self.content.profiles.iter().any(|p| p.id == profile) {
            return Err(invalid("unknown class unlock"));
        }
        Ok(self.state.unlocked.insert(profile))
    }
    /// Developer: set the active job's experience outright.
    pub fn set_experience(&mut self, xp: u32) -> bool {
        let previous = self.state.experience.insert(self.state.active, xp);
        previous != Some(xp)
    }

    pub fn add_experience(&mut self, amount: u32) -> io::Result<bool> {
        if amount == 0 {
            return Ok(false);
        }
        let xp = self.state.experience.entry(self.state.active).or_default();
        *xp = xp
            .checked_add(amount)
            .ok_or_else(|| invalid("class experience overflow"))?;
        Ok(true)
    }

    /// Native readers FUN_008d86e0 (38/14) and FUN_008d8910 (38/18).
    /// The local 100-XP rank curve is reconstructed; native star balances stay0.
    pub fn experience_packets(&self, previous_xp: u32) -> Vec<Vec<u8>> {
        let xp = self
            .state
            .experience
            .get(&self.state.active)
            .copied()
            .unwrap_or(0);
        let rank = (1 + xp / 100).min(20) as i32;
        let old_rank = (1 + previous_xp / 100).min(20) as i32;
        let mut packets = Vec::new();
        // Rank notification must precede the full update, which installs rank.
        if rank > old_rank {
            let mut packet = vec![38, 0, 18, 0];
            packet.extend(self.state.active.to_le_bytes());
            packet.extend(rank.to_le_bytes());
            packets.push(packet);
        }
        let mut packet = vec![38, 0, 14, 0];
        for value in [
            self.state.active,
            rank,
            if rank == 20 { 0 } else { (xp % 100) as i32 },
            0,
            0,
            0,
        ] {
            packet.extend(value.to_le_bytes());
        }
        packets.push(packet);
        packets
    }

    fn progression_bytes(&self, id: i32) -> [u8; 12] {
        let xp = self.state.experience.get(&id).copied().unwrap_or(0);
        let rank = (1 + xp / 100).min(20) as i32;
        let mut bytes = [0; 12];
        bytes[..4].copy_from_slice(&rank.to_le_bytes());
        bytes[4..8]
            .copy_from_slice(&(if rank == 20 { 0 } else { (xp % 100) as i32 }).to_le_bytes());
        // The third native field is StarsAvailable, not total experience.
        // Ability-star spending is not implemented in this opening slice.
        bytes[8..].copy_from_slice(&0i32.to_le_bytes());
        bytes
    }

    /// Apply before inventory splices, while original profile offsets are valid.
    pub fn patch_self(&self, template: &[u8]) -> io::Result<Vec<u8>> {
        let mut bytes = template.to_vec();
        for profile in &self.content.profiles {
            let end = profile
                .offset
                .checked_add(profile.length)
                .ok_or_else(|| invalid("profile offset overflow"))?;
            let original = bytes
                .get(profile.offset..end)
                .ok_or_else(|| invalid("truncated self profile"))?;
            if digest(original) != profile.sha256 {
                return Err(invalid("self profile differs from proof"));
            }
            let offset = profile.offset + profile.rank_offset;
            bytes[offset..offset + 12].copy_from_slice(&self.progression_bytes(profile.id));
        }
        let offset = self.content.active_offset;
        let end = offset
            .checked_add(4)
            .ok_or_else(|| invalid("active class offset overflow"))?;
        let original = bytes
            .get(offset..end)
            .ok_or_else(|| invalid("missing active class"))?;
        if digest(original) != self.content.active_sha256 {
            return Err(invalid("self active class differs from proof"));
        }
        bytes[offset..end].copy_from_slice(&self.state.active.to_le_bytes());
        Ok(bytes)
    }

    /// Selection mutates only after the payload can be built. Gateway persists
    /// the complete staged gameplay snapshot before releasing these replies.
    pub fn select(&mut self, id: i32, inventory: &Inventory) -> io::Result<Vec<u8>> {
        if !self.state.unlocked.contains(&id) {
            return Err(invalid("that class has not been unlocked"));
        }
        let packet = self.activation(id, inventory)?;
        self.state.active = id;
        Ok(packet)
    }

    pub fn activation(&self, id: i32, inventory: &Inventory) -> io::Result<Vec<u8>> {
        let profile = self
            .content
            .profiles
            .iter()
            .find(|p| p.id == id)
            .ok_or_else(|| invalid("unknown active class"))?;
        let mut payload = decode(&profile.serialized_hex)?;
        payload[profile.rank_offset..profile.rank_offset + 12]
            .copy_from_slice(&self.progression_bytes(id));
        payload.splice(
            profile.equipment_offset..profile.equipment_offset + profile.equipment_length,
            inventory.profile_equipment(id),
        );
        let mut packet = vec![38, 0, 21, 0];
        packet.extend((payload.len() as u32).to_le_bytes());
        packet.extend(payload);
        packet.extend(inventory.profile_attachments(id)?);
        packet.extend(3001i32.to_le_bytes());
        packet.extend(4005i32.to_le_bytes());
        packet.extend(0i32.to_le_bytes());
        Ok(packet)
    }
}
