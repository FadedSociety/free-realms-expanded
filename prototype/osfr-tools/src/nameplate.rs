//! Whose name the client draws over an actor: AddNpc (35/2) `hide_nameplate`, set the way the
//! 2014 servers set it (nameplates piece, lead spec 2026-11-11).
//!
//! Client: `PlayerUpdatePacketAddNpc_ReadBody` (`00921a10`) reads the switch into the packet
//! record at +0xbc. `sub_963D10` (`00963d10`) keeps it as bit 0x10 of the actor's +0x141 byte and
//! shows or hides the overhead name text with its inverse; `ProxiedCharacter__Process`
//! (`00973200`, through `maybe_ReadBit4At141` `0095f640`) re-applies the bit whenever it rebuilds
//! that text. The copy from +0xbc to +0x141 is not traced. The bit only drives the name text:
//! the hover cursor is the 35/12 entry the client keeps at +0x708 (`00962f30`,
//! `npc_presentation.rs`) and a click needs AddNpc `is_interactable`, so a hidden name still
//! lets the player hover and use the actor (native check owed).
//!
//! The rules are `data/content/nameplate_rules.json`, compiled in and written by
//! `scripts/build_nameplate_rules.py`. Adapted from Sanctuary (AGPL-3.0-or-later),
//! `src/Resources/Npcs.json` at 10f129e: which names and models its `HideNamePlate` always or
//! never hides (names and ids only). Critter and prop models come from the client's own
//! `Models.txt` RACE_ID, the races client-rs `fr-client` `names.rs` treats as things (R).

use crate::startup_world::NpcSpawn;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fs, io,
    path::Path,
    sync::LazyLock,
};

/// The shipped rules file.
pub const SHIPPED: &str = include_str!("../data/content/nameplate_rules.json");

/// The parsed shipped rules. Its own test proves the file parses; a broken copy hides only
/// pick-ups rather than failing every spawn.
static RULES: LazyLock<Rules> = LazyLock::new(|| Rules::parse(SHIPPED).unwrap_or_default());

/// What a spawn is, as far as its name is concerned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A content NPC row or placed actor: its 2014 name, its model, then its model's race decide.
    Npc,
    /// A player, a quest giver, a live quest's talk target or a vendor: always named.
    Character,
    /// A gathering, harvest, quest pick-up or collection node, or a collection coin.
    Pickup,
}

/// Whether AddNpc should hide this actor's name. In order: a [`Kind::Character`] never is; a
/// row's own `hide_nameplate` (`explicit`) wins next; a [`Kind::Pickup`] always is; otherwise
/// the first rules block that names the row decides (2014 hidden names, 2014 shown names, 2014
/// hidden models, 2014 shown models, then critter, prop and marker models); a row none of them
/// names keeps its name.
pub fn hidden(model_id: i32, name: &str, kind: Kind, explicit: Option<bool>) -> bool {
    match (kind, explicit) {
        (Kind::Character, _) => false,
        (_, Some(hide)) => hide,
        (Kind::Pickup, None) => true,
        (Kind::Npc, None) => RULES.npc(model_id, name),
    }
}

/// The rules blocks, as sets. Names compare trimmed and lower-cased.
#[derive(Clone, Debug, Default)]
pub struct Rules {
    hidden_names: HashSet<String>,
    shown_names: HashSet<String>,
    hidden_models: HashSet<i32>,
    shown_models: HashSet<i32>,
    /// Critter, prop and marker models.
    thing_models: HashSet<i32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    schema_version: u32,
    #[serde(rename = "provenance")]
    _provenance: BTreeMap<String, String>,
    hidden_names: Block,
    shown_names: Block,
    hidden_models: Block,
    shown_models: Block,
    critter_races: Block,
    prop_races: Block,
    marker_models: Block,
}

/// One rules block: its source and the names or `Models.txt` ids it lists (a race block also
/// names its race ids, for readers).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Block {
    source: String,
    #[serde(default)]
    names: Vec<String>,
    #[serde(default)]
    model_ids: Vec<i32>,
    #[serde(default, rename = "race_ids")]
    _race_ids: Vec<u32>,
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn key(name: &str) -> String {
    name.trim().to_lowercase()
}

impl Rules {
    /// Parses a rules file. Every block must cite its source and list something.
    pub fn parse(text: &str) -> io::Result<Self> {
        let file: File = serde_json::from_str(text)
            .map_err(|error| invalid(format!("nameplate rules: {error}")))?;
        if file.schema_version != 1 {
            return Err(invalid("nameplate rules: schema_version is not 1"));
        }
        for (name, block) in [
            ("hidden_names", &file.hidden_names),
            ("shown_names", &file.shown_names),
            ("hidden_models", &file.hidden_models),
            ("shown_models", &file.shown_models),
            ("critter_races", &file.critter_races),
            ("prop_races", &file.prop_races),
            ("marker_models", &file.marker_models),
        ] {
            if block.source.trim().is_empty()
                || block.names.is_empty() && block.model_ids.is_empty()
            {
                return Err(invalid(format!(
                    "nameplate rules: {name} has no source or no entries"
                )));
            }
        }
        let names = |block: Block| block.names.iter().map(|name| key(name)).collect();
        let models = |block: Block| block.model_ids.into_iter().collect();
        Ok(Self {
            hidden_names: names(file.hidden_names),
            shown_names: names(file.shown_names),
            hidden_models: models(file.hidden_models),
            shown_models: models(file.shown_models),
            thing_models: [file.critter_races, file.prop_races, file.marker_models]
                .into_iter()
                .flat_map(|block| block.model_ids)
                .collect(),
        })
    }

    fn npc(&self, model_id: i32, name: &str) -> bool {
        let name = key(name);
        if self.hidden_names.contains(&name) {
            true
        } else if self.shown_names.contains(&name) {
            false
        } else if self.hidden_models.contains(&model_id) {
            true
        } else if self.shown_models.contains(&model_id) {
            false
        } else {
            self.thing_models.contains(&model_id)
        }
    }
}

/// One `quests/*.json` record, as far as names go.
#[derive(Deserialize)]
struct Quest {
    #[serde(default)]
    giver_npc_id: u64,
    #[serde(default = "enabled_default")]
    enabled: bool,
    #[serde(default)]
    objectives: Vec<Goal>,
}

fn enabled_default() -> bool {
    true
}

#[derive(Deserialize)]
struct Goal {
    #[serde(default)]
    kind: String,
    #[serde(default)]
    npc_id: Option<u64>,
    #[serde(default)]
    alternatives: Vec<Goal>,
}

impl Goal {
    /// Its talk or delivery NPC, or those of an `any_of`'s leaves (`ObjectiveKind::talk_targets`).
    fn talk_targets(&self) -> impl Iterator<Item = u64> + '_ {
        let leaves = if self.alternatives.is_empty() {
            std::slice::from_ref(self)
        } else {
            &self.alternatives
        };
        leaves
            .iter()
            .filter(|goal| matches!(goal.kind.as_str(), "talk_to_npc" | "deliver_item"))
            .filter_map(|goal| goal.npc_id)
    }
}

/// The content NPC ids that keep their name whatever their model: every quest's giver, every
/// talk or delivery target of a live quest (enabled and with steps, as
/// `QuestEngine::is_enabled`), and the vendors the caller names.
#[derive(Clone, Debug, Default)]
pub struct Named(BTreeSet<u64>);

impl Named {
    /// Reads `content_dir/quests/*.json`; a content set without `quests/` names only `vendors`.
    pub fn load(content_dir: &Path, vendors: impl IntoIterator<Item = u64>) -> io::Result<Self> {
        let mut ids: BTreeSet<u64> = vendors.into_iter().collect();
        let dir = content_dir.join("quests");
        if dir.is_dir() {
            for entry in fs::read_dir(&dir)? {
                let path = entry?.path();
                if !path
                    .extension()
                    .is_some_and(|extension| extension == "json")
                {
                    continue;
                }
                for quest in crate::world::read_rows::<Quest>(&path)?.records {
                    ids.insert(quest.giver_npc_id);
                    if quest.enabled {
                        ids.extend(quest.objectives.iter().flat_map(Goal::talk_targets));
                    }
                }
            }
        }
        Ok(Self(ids))
    }

    pub fn kind(&self, npc_id: u64) -> Kind {
        if self.0.contains(&npc_id) {
            Kind::Character
        } else {
            Kind::Npc
        }
    }

    /// Sets `spawn.hide_nameplate` for the content NPC `npc_id` (see [`hidden`]).
    pub(crate) fn mark(
        &self,
        spawn: &mut NpcSpawn,
        npc_id: u64,
        name: Option<&str>,
        explicit: Option<bool>,
    ) {
        let hide = hidden(
            spawn.model_id,
            name.unwrap_or_default(),
            self.kind(npc_id),
            explicit,
        );
        spawn.hide_nameplate = u8::from(hide);
    }
}
