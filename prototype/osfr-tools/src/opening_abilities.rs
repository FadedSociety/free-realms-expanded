//! Basic native hotbars from private, pinned community metadata.
//! This presents one supported basic attack per combat job. Encounter damage,
//! cooldowns and targeting remain reconstructed; original skills are not implied.
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, Read},
    path::Path,
    sync::Arc,
};

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Basic {
    profile_id: i32,
    definition_id: i32,
    name_id: i32,
    description_id: i32,
    icon_id: i32,
    weapon_name_id: i32,
    /// (R) Required item class in Weapon slot 7, not an instance/definition id.
    #[serde(default)]
    weapon_class: Option<i32>,
}

#[derive(Deserialize)]
struct Content {
    schema_version: u32,
    basics: Vec<Basic>,
}

/// A job ability beyond the basic attack, on toolbar slot 1.. (keys 2..).
/// Names, icons and heal amounts are client/wiki facts; damage, energy and
/// cooldowns are reconstructed (R). See data/content/job_abilities.json.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobAbility {
    pub slot: i32,
    pub definition_id: i32,
    pub name_id: i32,
    #[serde(default)]
    pub description_id: i32,
    pub icon_id: i32,
    /// Client animation group; zero preserves the legacy content's swing.
    #[serde(default)]
    pub animation_id: i32,
    #[serde(default)]
    pub effect_id: i32,
    /// (R) Explicit range, or zero to use the job's existing basic reach.
    #[serde(default)]
    pub range: f32,
    #[serde(default)]
    pub effects: Vec<crate::abilities::JobEffect>,
    /// A documented skill whose mechanic is not implemented; casting fails closed.
    #[serde(default)]
    pub unavailable: bool,
    /// Strike multiplier of the job's basic attack damage; 0 = no strike.
    #[serde(default)]
    pub damage_multiplier: f32,
    /// Health restored to the user.
    #[serde(default)]
    pub heal: u32,
    #[serde(default)]
    pub energy_cost: i32,
    pub cooldown_ms: u64,
    /// Job level that unlocks it (the owner: every job skill is level-locked;
    /// Medic First Aid 1, Target Vitals 5, Vitamins 10, Shock Paddles 15).
    /// Required in authored content: omission must not unlock a later skill.
    /// Weapon abilities are level 1.
    pub min_level: u32,
    /// (R) Job weapon requirement. Legacy rows inherit their basic's class.
    #[serde(default)]
    pub weapon_class: Option<i32>,
    #[serde(default)]
    #[allow(dead_code)]
    source: serde_json::Value,
}

impl JobAbility {
    pub fn valid(&self) -> bool {
        (1..=7).contains(&self.slot)
            && self.definition_id > 0
            && self.name_id > 0
            && self.description_id >= 0
            && self.icon_id > 0
            && self.animation_id >= 0
            && self.effect_id >= 0
            && (0.0..=30.0).contains(&self.range)
            && (0.0..=10.0).contains(&self.damage_multiplier)
            && self.heal <= 10_000
            && (0..=100).contains(&self.energy_cost)
            && self.cooldown_ms <= 600_000
            && (1..=20).contains(&self.min_level)
            && self
                .weapon_class
                .is_none_or(|class| (1..=999).contains(&class))
            && self.effects.len() <= 4
            && self.effects.iter().all(crate::abilities::JobEffect::valid)
            && self
                .effects
                .iter()
                .filter(|e| matches!(e, crate::abilities::JobEffect::AreaDamage { .. }))
                .count()
                <= 1
            && (self.unavailable
                || self.damage_multiplier > 0.0
                || self.heal > 0
                || self
                    .effects
                    .iter()
                    .any(|e| !matches!(e, crate::abilities::JobEffect::AreaDamage { .. })))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct JobRow {
    profile_id: i32,
    slots: Vec<JobAbility>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct JobContent {
    schema_version: u32,
    jobs: Vec<JobRow>,
}

#[derive(Clone)]
pub struct OpeningAbilities {
    basics: Arc<BTreeMap<i32, Basic>>,
    jobs: Arc<BTreeMap<i32, Vec<JobAbility>>>,
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn put(bytes: &mut Vec<u8>, value: i32) {
    bytes.extend(value.to_le_bytes());
}

impl OpeningAbilities {
    pub fn supports_basic(&self, profile_id: i32) -> bool {
        self.basics.contains_key(&profile_id)
    }

    pub fn load(directory: &Path) -> io::Result<Self> {
        let file = File::open(directory.join("opening-abilities.json"))?;
        let mut bytes = Vec::new();
        file.take(65537).read_to_end(&mut bytes)?;
        if bytes.len() > 65536 {
            return Err(invalid("opening ability content exceeds bounds"));
        }
        let content: Content = serde_json::from_slice(&bytes)
            .map_err(|_| invalid("invalid opening ability content"))?;
        if content.schema_version != 1 || content.basics.is_empty() || content.basics.len() > 16 {
            return Err(invalid("unsupported opening ability content"));
        }
        let mut basics = BTreeMap::new();
        // (R) Old frozen campaigns predate weapon requirements. Fill only the
        // missing class from the authored roster; never let omission mean unarmed.
        let defaults: Content =
            serde_json::from_str(include_str!("../data/content/opening-abilities.json"))
                .map_err(|_| invalid("invalid built-in weapon requirements"))?;
        for mut basic in content.basics {
            basic.weapon_class = basic.weapon_class.or_else(|| {
                defaults
                    .basics
                    .iter()
                    .find(|row| row.profile_id == basic.profile_id)
                    .and_then(|row| row.weapon_class)
            });
            if basic.profile_id <= 0
                || basic.definition_id <= 0
                || basic.name_id < 0
                || basic.description_id < 0
                || basic.icon_id <= 0
                || basic.weapon_name_id <= 0
                || !basic
                    .weapon_class
                    .is_some_and(|class| (1..=999).contains(&class))
                || basics.insert(basic.profile_id, basic).is_some()
            {
                return Err(invalid("invalid or duplicate opening basic attack"));
            }
        }
        Ok(Self {
            basics: Arc::new(basics),
            jobs: Arc::new(BTreeMap::new()),
        })
    }

    /// Load the full roster independently of a campaign's basic-attack subset.
    /// Slots run 1, 2, ...; a job needs a loaded basic before activation/casting.
    pub fn load_job_abilities(&mut self, path: &Path) -> io::Result<()> {
        let file = File::open(path)?;
        let mut bytes = Vec::new();
        file.take(65537).read_to_end(&mut bytes)?;
        if bytes.len() > 65536 {
            return Err(invalid("job ability content exceeds bounds"));
        }
        let content: JobContent =
            serde_json::from_slice(&bytes).map_err(|e| invalid(&e.to_string()))?;
        if content.schema_version != 1 || content.jobs.len() > 32 {
            return Err(invalid("unsupported job ability content"));
        }
        let mut jobs = BTreeMap::new();
        let mut ids = std::collections::BTreeSet::new();
        for mut row in content.jobs {
            for ability in &mut row.slots {
                ability.weapon_class = ability.weapon_class.or_else(|| {
                    self.basics
                        .get(&row.profile_id)
                        .and_then(|b| b.weapon_class)
                });
            }
            let valid = row.profile_id > 0
                && !row.slots.is_empty()
                && row.slots.len() <= 7
                && row.slots.iter().enumerate().all(|(i, a)| {
                    a.slot == i as i32 + 1
                        && a.valid()
                        && ids.insert(a.definition_id)
                        && !self
                            .basics
                            .values()
                            .any(|b| b.definition_id == a.definition_id)
                });
            if !valid || jobs.insert(row.profile_id, row.slots).is_some() {
                return Err(invalid("invalid or duplicate job abilities"));
            }
        }
        self.jobs = Arc::new(jobs);
        Ok(())
    }

    /// The job ability on toolbar `slot` (1..) for this profile, if the job
    /// level has unlocked it.
    pub fn job_slot(&self, profile_id: i32, slot: i32, level: u32) -> Option<&JobAbility> {
        self.basics.get(&profile_id)?;
        self.jobs.get(&profile_id).and_then(|slots| {
            slots
                .iter()
                .find(|a| a.slot == slot && a.min_level <= level)
        })
    }

    /// Resolve server-owned requirements against CURRENT inventory at cast
    /// time. Equip/unequip needs no cached combat snapshot or resync hook.
    pub fn weapon_matches(
        &self,
        inventory: &crate::inventory::Inventory,
        profile: i32,
        slot: i32,
        level: u32,
    ) -> bool {
        let required = if slot == 0 {
            self.basics.get(&profile).and_then(|b| b.weapon_class)
        } else {
            self.job_slot(profile, slot, level)
                .and_then(|a| a.weapon_class)
        };
        crate::abilities::equipped_weapon_matches(inventory, profile, required)
    }

    /// Whether `level` unlocks a job ability `previous` did not.
    pub fn unlocks_between(&self, profile_id: i32, previous: u32, level: u32) -> bool {
        self.jobs.get(&profile_id).is_some_and(|slots| {
            slots
                .iter()
                .any(|a| a.min_level > previous && a.min_level <= level)
        })
    }

    /// Definitions precede their toolbar. Noncombat jobs receive eight empty
    /// slots so switching jobs clears the prior combat toolbar.
    pub fn activation(&self, profile_id: i32, level: u32) -> io::Result<Vec<Vec<u8>>> {
        if profile_id <= 0 {
            return Err(invalid("invalid hotbar profile"));
        }
        let mut toolbar = vec![36, 0, 5, 0];
        put(&mut toolbar, profile_id);
        put(&mut toolbar, 8);
        let mut replies = Vec::new();
        if let Some(basic) = self.basics.get(&profile_id) {
            // The evidenced 36/13 definition is 160 bytes. All optional fields
            // are zero for the supported basic attack: no mana/aura/AoE effects.
            let mut definition = vec![0; 160];
            definition[..4].copy_from_slice(&[36, 0, 13, 0]);
            for (offset, value) in [
                (4, basic.definition_id),
                (10, basic.name_id),
                (14, basic.description_id),
                (18, basic.icon_id),
            ] {
                definition[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            }
            replies.push(definition);
            for value in [
                3,
                basic.definition_id,
                0,
                basic.icon_id,
                basic.weapon_name_id,
                4,
                0,
                1,
                basic.definition_id,
                0,
            ] {
                put(&mut toolbar, value);
            }
            toolbar.push(1); // ForceDismount, as in the pinned CreateToolbarSlot.
            let extra = self
                .jobs
                .get(&profile_id)
                .map_or(&[][..], |slots| slots.as_slice());
            for ability in extra {
                if ability.min_level > level {
                    // Not learned yet: the slot stays empty (keys keep their order).
                    put(&mut toolbar, 0);
                    continue;
                }
                // 36/13 layout: id 4, name 10, description 14, icon 18, energy cost 50.
                let mut definition = vec![0; 160];
                definition[..4].copy_from_slice(&[36, 0, 13, 0]);
                for (offset, value) in [
                    (4, ability.definition_id),
                    (10, ability.name_id),
                    (14, ability.description_id),
                    (18, ability.icon_id),
                    (50, ability.energy_cost),
                ] {
                    definition[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
                }
                replies.push(definition);
                for value in [
                    3,
                    ability.definition_id,
                    ability.energy_cost,
                    ability.icon_id,
                    ability.name_id,
                    4,
                    0,
                    1,
                    ability.definition_id,
                    0,
                ] {
                    put(&mut toolbar, value);
                }
                toolbar.push(1);
            }
            // The remaining slots have Type=0.
            toolbar.extend(vec![0; 4 * (7 - extra.len())]);
        } else {
            toolbar.extend([0; 32]);
        }
        replies.push(toolbar);
        Ok(replies)
    }
}
