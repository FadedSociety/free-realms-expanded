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

/// W176: one job a trainer NPC teaches (`job_trainers.json`). The job, trainer pointer and
/// "Become a <job>" locale label are documented per row. That a menu choice immediately grants
/// the job, without the original quest, is reconstructed policy (R).
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trainer {
    pub npc: u64,
    pub profile: i32,
    /// Human name, used in chat lines only.
    pub job: String,
    /// Client locale id of the menu label ("Become a Miner").
    pub label_string_id: u32,
    /// A recovered prerequisite: another job at this rank first (client text, e.g. the Ninja
    /// trainer wants a level 20 Archer). The rank curve itself is the local (R) one.
    #[serde(default)]
    pub requires: Option<Requirement>,
    /// Evidence for the row; not read by the server.
    pub source: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Requirement {
    pub profile: i32,
    pub job: String,
    pub rank: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TrainerFile {
    schema_version: u32,
    trainers: Vec<Trainer>,
}

/// Why a trainer would not teach a job. Nothing changes on any of these.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LearnDenied {
    Busy,
    Unavailable,
    AlreadyKnown,
    NeedsRank { job: String, rank: u32 },
}

impl LearnDenied {
    pub fn message(&self) -> String {
        match self {
            Self::Busy => "Finish what you are doing first.".into(),
            Self::Unavailable => "Job training is not available here.".into(),
            Self::AlreadyKnown => "You already know that job.".into(),
            Self::NeedsRank { job, rank } => {
                format!("Come back when you are a level {rank} {job}.")
            }
        }
    }
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
        Self::load_with_retired_job(directory, state, None)
    }

    /// Load a saved snapshot that may still carry a predecessor derby marker naming
    /// `retired_job`. T023 unlocked the pre-derby job, so content that no longer ships it would
    /// otherwise fail the subset check below before `Gameplay::load` can skip the marker (F206
    /// review). Only that one unknown id and its experience are dropped, and only when it was
    /// actually unlocked; every other unknown id still fails closed.
    pub(crate) fn load_with_retired_job(
        directory: &Path,
        state: Option<ProfileState>,
        retired_job: Option<i32>,
    ) -> io::Result<Self> {
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
        let mut state = state.unwrap_or_else(|| content.default_state.clone());
        if let Some(job) = retired_job.filter(|job| *job > 0 && !ids.contains(job))
            && state.unlocked.remove(&job)
        {
            state.experience.remove(&job);
            eprintln!("profiles: dropped retired job {job} from the saved progression");
        }
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
    /// F228: every starter item id across every job profile. A starter outfit piece is granted
    /// free when its job is learned or reselected, so the store resells it for 0 wherever the
    /// player got that copy.
    pub fn all_starter_items(&self) -> BTreeSet<i32> {
        self.content
            .starter_items
            .values()
            .flatten()
            .copied()
            .collect()
    }
    pub fn unlock(&mut self, profile: i32) -> io::Result<bool> {
        if !self.has_profile(profile) {
            return Err(invalid("unknown class unlock"));
        }
        Ok(self.state.unlocked.insert(profile))
    }
    fn has_profile(&self, profile: i32) -> bool {
        self.content.profiles.iter().any(|p| p.id == profile)
    }

    /// The job's level on the local (R) 100-XP curve, 1 to 20.
    pub fn rank(&self, profile: i32) -> u32 {
        (1 + self.state.experience.get(&profile).copied().unwrap_or(0) / 100).min(20)
    }

    /// (R) Any unlocked job at `rank` or higher; the client locale's guild-creation rule.
    pub fn has_job_rank(&self, rank: u32) -> bool {
        self.state
            .unlocked
            .iter()
            .any(|&profile| self.rank(profile) >= rank)
    }

    /// W176: read the optional `job_trainers.json`. Absent means no trainers. A row naming a job
    /// the profile content lacks is a content error, so a typo cannot silently offer nothing.
    pub fn load_trainers(&self, directory: &Path) -> io::Result<Vec<Trainer>> {
        let path = directory.join("job_trainers.json");
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error),
        };
        if bytes.len() > 65_536 {
            return Err(invalid("job trainer content exceeds bounds"));
        }
        let file: TrainerFile = serde_json::from_slice(&bytes).map_err(invalid)?;
        if file.schema_version != 1 || file.trainers.len() > 128 {
            return Err(invalid("unsupported job trainer content"));
        }
        let mut seen = BTreeSet::new();
        for trainer in &file.trainers {
            let required = trainer.requires.as_ref();
            if trainer.npc == 0
                || trainer.label_string_id == 0
                || trainer.job.trim().is_empty()
                || trainer.job.len() > 40
                || !self.has_profile(trainer.profile)
                || trainer.source.trim().is_empty()
                || trainer.source.len() > 1024
                || required.is_some_and(|r| {
                    r.job.trim().is_empty()
                        || r.job.len() > 40
                        || !(1..=20).contains(&r.rank)
                        || r.profile == trainer.profile
                        || !self.has_profile(r.profile)
                })
                || !seen.insert((trainer.npc, trainer.profile))
            {
                return Err(invalid("invalid job trainer row"));
            }
        }
        Ok(file.trainers)
    }

    /// W176 (R): a trainer teaches a job the player lacks, once. The caller has already proved
    /// the menu choice was offered, the range, and that the player is not busy.
    pub fn learn(&mut self, trainer: &Trainer) -> Result<(), LearnDenied> {
        if !self.has_profile(trainer.profile) {
            return Err(LearnDenied::Unavailable);
        }
        if self.state.unlocked.contains(&trainer.profile) {
            return Err(LearnDenied::AlreadyKnown);
        }
        if let Some(need) = &trainer.requires
            && (!self.state.unlocked.contains(&need.profile) || self.rank(need.profile) < need.rank)
        {
            return Err(LearnDenied::NeedsRank {
                job: need.job.clone(),
                rank: need.rank,
            });
        }
        self.state.unlocked.insert(trainer.profile);
        Ok(())
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

    /// Change the active job after validating it is unlocked. The 38/21 activation is a
    /// separate [`Self::activation`] call so a caller can repair the outfit first.
    pub fn activate(&mut self, id: i32) -> io::Result<()> {
        if !self.state.unlocked.contains(&id) {
            return Err(invalid("that class has not been unlocked"));
        }
        self.state.active = id;
        Ok(())
    }

    /// Activate `id` and return the 38/21 activation for the current inventory.
    /// Callers that repair the outfit first call [`Self::activate`], repair, then
    /// [`Self::activation`], because the attachment list must match what is worn.
    pub fn select(&mut self, id: i32, inventory: &Inventory) -> io::Result<Vec<u8>> {
        self.activate(id)?;
        self.activation(id, inventory)
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
