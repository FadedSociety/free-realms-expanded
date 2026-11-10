//! The enemy catalogue: the kinds of `enemies.json` as the live engine reads them (combat
//! design section 7). A definition names a kind once; camps, field rows, battle rows and
//! fights point at it by `id` or by its kill-credit `definition_id`.
//!
//! The keys this file adds to a definition are all optional. `rank` is the switch: a kind
//! with a rank takes its health and hit from the rank tables of `combat_scale.json` at the
//! tier of whatever places it, and `health` or `hit` state a number outright. A kind without
//! a rank keeps the older rule, where `max_health` and `damage` are level-one baselines the
//! loaders scale. A content directory without the file has an empty catalogue.
use super::*;

const FILE: &str = "enemies.json";
const MAX_BYTES: u64 = 1 << 20;
const MAX_KINDS: usize = 4096;

#[derive(Deserialize)]
struct File {
    definitions: Vec<Definition>,
}

/// Walk, run and return-home speed in metres per second.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Speeds {
    pub walk: f32,
    pub run: f32,
    pub home: f32,
}

/// One enemy kind. Keys of the file this struct does not name are left to their own readers.
#[derive(Clone, Debug, Deserialize)]
pub struct Definition {
    pub id: String,
    #[serde(default)]
    pub kind: String,
    pub definition_id: i32,
    #[serde(default)]
    pub model_id: i32,
    #[serde(default)]
    pub name_string_id: i32,
    #[serde(default)]
    pub boss: bool,
    /// How strong the kind is; absent keeps the baseline rule.
    #[serde(default)]
    pub rank: Option<Rank>,
    /// The weapon tier the kind was measured at; whoever places it may give another.
    #[serde(default)]
    pub tier: Option<u8>,
    /// Health stated outright instead of the rank's.
    #[serde(default)]
    pub health: Option<u32>,
    /// Raw hit stated outright instead of the rank's.
    #[serde(default)]
    pub hit: Option<u32>,
    /// A `behaviours.json` preset name, or a preset with added rules.
    #[serde(default)]
    pub behaviour: Option<serde_json::Value>,
    /// `attacks.json` ids.
    #[serde(default)]
    pub attacks: Vec<String>,
    #[serde(default)]
    pub speeds: Option<Speeds>,
    /// AddNpc scale.
    #[serde(default)]
    pub scale: Option<f32>,
    #[serde(default)]
    pub status_immune: Vec<String>,
    /// A drop table id, or `none`.
    #[serde(default)]
    pub loot: Option<String>,
    /// What quest steps and fight rules match the kind by.
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Default)]
pub struct Catalogue {
    kinds: Vec<Definition>,
    by_id: BTreeMap<String, usize>,
    by_kind: BTreeMap<i32, usize>,
}

fn word(text: &str) -> bool {
    !text.is_empty() && text.len() <= 64 && !text.chars().any(char::is_control)
}

fn words(list: &[String]) -> bool {
    list.len() <= 16 && list.iter().all(|text| word(text))
}

impl Definition {
    fn valid(&self) -> bool {
        let speed = |v: f32| v.is_finite() && v > 0.0 && v <= 30.0;
        word(&self.id)
            && self.definition_id >= 0
            && self.tier.is_none_or(|tier| (1..=5).contains(&tier))
            && self.health.is_none_or(|h| (1..=10_000_000).contains(&h))
            && self.hit.is_none_or(|h| h <= 100_000)
            // A number stated outright, or a tier, means nothing without a rank.
            && (self.rank.is_some()
                || (self.tier.is_none() && self.health.is_none() && self.hit.is_none()))
            && (self.speeds).is_none_or(|s| speed(s.walk) && speed(s.run) && speed(s.home))
            && self.scale.is_none_or(|s| s.is_finite() && (0.1..=10.0).contains(&s))
            && self.loot.as_deref().is_none_or(word)
            && words(&self.attacks)
            && words(&self.status_immune)
            && words(&self.tags)
    }
}

impl Catalogue {
    /// The catalogue of `directory`; empty when it has no `enemies.json`. A file that is
    /// there but bad refuses the load and names itself.
    pub(in crate::encounter) fn load(directory: &Path) -> io::Result<Self> {
        let path = directory.join(FILE);
        if !path.is_file() {
            return Ok(Self::default());
        }
        let refuse = |why: &dyn std::fmt::Display| invalid(&format!("{}: {why}", path.display()));
        if fs::metadata(&path).map_err(|e| refuse(&e))?.len() > MAX_BYTES {
            return Err(refuse(&"file too large"));
        }
        let file: File = serde_json::from_slice(&fs::read(&path).map_err(|e| refuse(&e))?)
            .map_err(|e| refuse(&e))?;
        if file.definitions.len() > MAX_KINDS {
            return Err(refuse(&"too many definitions"));
        }
        let mut catalogue = Self::default();
        for (index, definition) in file.definitions.into_iter().enumerate() {
            if !definition.valid()
                || (catalogue.by_id)
                    .insert(definition.id.clone(), index)
                    .is_some()
            {
                return Err(refuse(&format!("invalid definition {}", definition.id)));
            }
            // Kinds may share a kill-credit id; the first one answers for it.
            (catalogue.by_kind)
                .entry(definition.definition_id)
                .or_insert(index);
            catalogue.kinds.push(definition);
        }
        Ok(catalogue)
    }

    pub fn by_id(&self, id: &str) -> Option<&Definition> {
        self.by_id.get(id).map(|&index| &self.kinds[index])
    }

    /// The kind a kill of `definition_id` is credited to.
    pub fn by_kind(&self, definition_id: i32) -> Option<&Definition> {
        (self.by_kind.get(&definition_id)).map(|&index| &self.kinds[index])
    }

    pub fn len(&self) -> usize {
        self.kinds.len()
    }

    pub fn is_empty(&self) -> bool {
        self.kinds.is_empty()
    }

    pub fn kinds(&self) -> &[Definition] {
        &self.kinds
    }
}
