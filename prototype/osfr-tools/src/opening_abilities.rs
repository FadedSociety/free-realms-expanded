//! Basic native hotbars from private, pinned community metadata.
//! This presents one supported basic attack per combat job. Encounter damage,
//! cooldowns and targeting remain reconstructed; original skills are not implied.
use crate::power_ups::Slot as PowerUpSlot;
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
    /// (R) Canonical starter weapon granted when a clothed job has no weapon.
    /// Older campaigns predate this field; the loader fills it from the
    /// built-in roster below, just as it fills a missing weapon class.
    #[serde(default)]
    weapon_definition_id: i32,
    /// (R) Required item class in Weapon slot 7, not an instance/definition id.
    #[serde(default)]
    weapon_class: Option<i32>,
}

#[derive(Deserialize)]
struct Content {
    schema_version: u32,
    basics: Vec<Basic>,
}

/// The client draws four ability buttons: the basic attack and three job rows.
/// `GraphicActionBar.setButtonData` ignores `id >= 4` (ScriptsBase.lua:5215) and
/// `resetButtons` clears 0..3; native populate `0x008ee3d0` hands it every row.
const JOB_ROWS: i32 = 3;

/// What the live slot-7 weapon means for one combat job's toolbar (T521).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeaponLabel {
    /// No inventory was read: label the basic slot with the content's
    /// `weapon_name_id` (unit tests, content reload).
    Starter,
    /// A slot-7 weapon whose class matches the job: label the basic slot with
    /// this `ClientItemDefinition` name id.
    Equipped(i32),
    /// No matching weapon: the combat bar is eight empty slots, like a
    /// noncombat job (R, OSFR #124). A held power-up keeps its key slot.
    Unarmed,
}

/// A job ability beyond the basic attack, on toolbar row 1..=JOB_ROWS (keys 2-4).
/// Names, icons and heal amounts are client/wiki facts; damage, energy and
/// cooldowns are reconstructed (R). See data/content/job_abilities.json.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobAbility {
    /// Toolbar row. Several abilities may share a row with different
    /// `min_level`s: the row shows the highest unlocked one (`job_bar`).
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
    /// A documented skill whose mechanic is not implemented: never placed on
    /// the bar, and casting fails closed.
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
    /// Job level that unlocks it (The owner: every job skill is level-locked;
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
        (1..=JOB_ROWS).contains(&self.slot)
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

    /// Canonical starter weapon for a supported combat job. The definition,
    /// rather than a display name, seeds the item grant and equip path.
    pub fn starter_weapon(&self, profile_id: i32) -> Option<i32> {
        self.basics
            .get(&profile_id)
            .map(|basic| basic.weapon_definition_id)
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
        // (R) Old frozen campaigns predate weapon metadata. Fill its missing
        // class and starter definition from the authored roster; never let
        // omission mean unarmed.
        let defaults: Content =
            serde_json::from_str(include_str!("../data/content/opening-abilities.json"))
                .map_err(|_| invalid("invalid built-in weapon requirements"))?;
        for mut basic in content.basics {
            let default = defaults
                .basics
                .iter()
                .find(|row| row.profile_id == basic.profile_id);
            basic.weapon_class = basic
                .weapon_class
                .or_else(|| default.and_then(|row| row.weapon_class));
            if basic.weapon_definition_id == 0 {
                basic.weapon_definition_id = default
                    .map(|row| row.weapon_definition_id)
                    .unwrap_or_default();
            }
            if basic.profile_id <= 0
                || basic.definition_id <= 0
                || basic.name_id < 0
                || basic.description_id < 0
                || basic.icon_id <= 0
                || basic.weapon_name_id <= 0
                || basic.weapon_definition_id <= 0
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
    /// Rows are 1..=JOB_ROWS, one ability per (row, min_level); a job needs a
    /// loaded basic before activation/casting.
    pub fn load_job_abilities(&mut self, path: &Path) -> io::Result<()> {
        // (R) Operator-authored file with per-field provenance (24 skills was ~66 KB
        // after W125); 1 MiB stays bounded with room for every job's roster.
        const MAX_JOB_CONTENT: u64 = 1024 * 1024;
        let file = File::open(path)?;
        let mut bytes = Vec::new();
        file.take(MAX_JOB_CONTENT + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_JOB_CONTENT {
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
            // Unique (row, min_level) also bounds a job to 3 x 20 abilities.
            let mut placed = std::collections::BTreeSet::new();
            let valid = row.profile_id > 0
                && !row.slots.is_empty()
                && row.slots.iter().all(|a| {
                    a.valid()
                        && placed.insert((a.slot, a.min_level))
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

    /// One entry per toolbar row 1..=7 for the job at `level`; only rows
    /// 1..=JOB_ROWS are ever filled, so rows 4-7 go out empty. Each row shows
    /// its highest unlocked, available ability, so a later skill upgrades the
    /// row in place and a locked row stays empty instead of shifting others.
    pub fn job_bar(&self, profile_id: i32, level: u32) -> [Option<&JobAbility>; 7] {
        let mut bar: [Option<&JobAbility>; 7] = [None; 7];
        for ability in self.jobs.get(&profile_id).into_iter().flatten() {
            let row = &mut bar[(ability.slot - 1) as usize];
            if ability.min_level <= level
                && !ability.unavailable
                && row.is_none_or(|shown| shown.min_level < ability.min_level)
            {
                *row = Some(ability);
            }
        }
        bar
    }

    /// The job ability on toolbar `slot` (1..) for this profile, if the job
    /// level has unlocked it.
    pub fn job_slot(&self, profile_id: i32, slot: i32, level: u32) -> Option<&JobAbility> {
        self.basics.get(&profile_id)?;
        let index = usize::try_from(slot).ok()?.checked_sub(1)?;
        self.job_bar(profile_id, level)
            .get(index)
            .copied()
            .flatten()
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

    /// What the live slot-7 weapon means for this job's toolbar (T521). A
    /// matching weapon names the basic slot with its definition's `name_id`;
    /// an empty slot or a wrong-class weapon leaves the job unarmed (R, OSFR
    /// #124). Display only: `weapon_matches` remains the cast-time authority.
    pub fn weapon_label(
        &self,
        profile: i32,
        inventory: &crate::inventory::Inventory,
    ) -> WeaponLabel {
        let Some(required) = self
            .basics
            .get(&profile)
            .and_then(|basic| basic.weapon_class)
        else {
            return WeaponLabel::Starter;
        };
        match inventory.equipped_weapon_for_class(profile, required) {
            Some(definition) => inventory
                .presentation(definition)
                .map_or(WeaponLabel::Starter, |presentation| {
                    WeaponLabel::Equipped(presentation.name_id)
                }),
            None => WeaponLabel::Unarmed,
        }
    }

    /// Whether `level` unlocks a job ability `previous` did not.
    pub fn unlocks_between(&self, profile_id: i32, previous: u32, level: u32) -> bool {
        self.jobs.get(&profile_id).is_some_and(|slots| {
            slots
                .iter()
                .any(|a| !a.unavailable && a.min_level > previous && a.min_level <= level)
        })
    }

    /// Definitions precede their toolbar. Noncombat jobs receive eight empty
    /// slots so switching jobs clears the prior combat toolbar.
    pub fn activation(&self, profile_id: i32, level: u32) -> io::Result<Vec<Vec<u8>>> {
        self.activation_with_power_up(profile_id, level, None, WeaponLabel::Starter)
    }

    /// `activation` with a held power-up on its own key slot
    /// (`power_ups::SLOT`). The authored job row below it is not moved: the
    /// client's bar shows the kind instead for as long as it is held, which is
    /// the locale's "add super shield to bar" behavior, and the same row comes
    /// back on the next activation once the kind is used. `weapon` is the live
    /// slot-7 verdict: `Unarmed` sends the eight empty slots a noncombat job
    /// gets (R, OSFR #124) and still places a held power-up on its key.
    pub fn activation_with_power_up(
        &self,
        profile_id: i32,
        level: u32,
        power_up: Option<&PowerUpSlot>,
        weapon: WeaponLabel,
    ) -> io::Result<Vec<Vec<u8>>> {
        if profile_id <= 0 {
            return Err(invalid("invalid hotbar profile"));
        }
        let mut toolbar = vec![36, 0, 5, 0];
        put(&mut toolbar, profile_id);
        put(&mut toolbar, 8);
        let mut replies = Vec::new();
        if let Some(basic) = self.basics.get(&profile_id) {
            if weapon == WeaponLabel::Unarmed {
                put(&mut toolbar, 0);
            } else {
                // The evidenced 36/13 definition is 160 bytes. All optional
                // fields are zero for the supported basic attack: no mana/aura/AoE.
                replies.push(definition(
                    basic.definition_id,
                    basic.name_id,
                    basic.description_id,
                    basic.icon_id,
                    0,
                ));
                let name_id = match weapon {
                    WeaponLabel::Equipped(name_id) => name_id,
                    WeaponLabel::Starter | WeaponLabel::Unarmed => basic.weapon_name_id,
                };
                // The client resolves this name id and passes it to
                // HandlerActionBar:setButtonData (native populate 0x008ee3d0).
                ability_entry(&mut toolbar, basic.definition_id, 0, basic.icon_id, name_id);
            }
            for (index, ability) in self.job_bar(profile_id, level).into_iter().enumerate() {
                if let Some(power_up) =
                    power_up.filter(|_| index as i32 + 1 == crate::power_ups::SLOT)
                {
                    replies.push(definition(
                        power_up.definition_id,
                        power_up.name_id,
                        0,
                        power_up.icon_id,
                        0,
                    ));
                    ability_entry(
                        &mut toolbar,
                        power_up.definition_id,
                        0,
                        power_up.icon_id,
                        power_up.name_id,
                    );
                    continue;
                }
                // An unarmed job shows no skill rows, but a held power-up kept
                // its key above (R, OSFR #124).
                let ability = ability.filter(|_| weapon != WeaponLabel::Unarmed);
                let Some(ability) = ability else {
                    // Not learned yet: the slot stays empty (keys keep their order).
                    put(&mut toolbar, 0);
                    continue;
                };
                replies.push(definition(
                    ability.definition_id,
                    ability.name_id,
                    ability.description_id,
                    ability.icon_id,
                    ability.energy_cost,
                ));
                ability_entry(
                    &mut toolbar,
                    ability.definition_id,
                    ability.energy_cost,
                    ability.icon_id,
                    ability.name_id,
                );
            }
        } else {
            toolbar.extend([0; 32]);
        }
        replies.push(toolbar);
        Ok(replies)
    }
}

/// The evidenced 160-byte 36/13 definition: id at 4, name at 10, description
/// at 14, icon at 18 and energy cost at 50; optional fields stay zero.
fn definition(
    id: i32,
    name_id: i32,
    description_id: i32,
    icon_id: i32,
    energy_cost: i32,
) -> Vec<u8> {
    let mut definition = vec![0; 160];
    definition[..4].copy_from_slice(&[36, 0, 13, 0]);
    for (offset, value) in [
        (4, id),
        (10, name_id),
        (14, description_id),
        (18, icon_id),
        (50, energy_cost),
    ] {
        definition[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    definition
}

/// One type-3 toolbar entry, byte-for-byte the row the pinned
/// `CreateToolbarSlot`/36/5 reader already accepts: type, definition id,
/// energy cost, icon, name, then the self-cast selector pair (4, 0, 1) and the
/// trailing ForceDismount byte.
fn ability_entry(
    toolbar: &mut Vec<u8>,
    definition_id: i32,
    energy_cost: i32,
    icon_id: i32,
    name_id: i32,
) {
    for value in [
        3,
        definition_id,
        energy_cost,
        icon_id,
        name_id,
        4,
        0,
        1,
        definition_id,
        0,
    ] {
        put(toolbar, value);
    }
    toolbar.push(1);
}
