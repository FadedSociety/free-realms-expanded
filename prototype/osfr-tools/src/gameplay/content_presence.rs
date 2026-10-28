//! T852: a gameplay load starts with the content its directory actually carries.
//!
//! `data/content` on its own is a valid server. A module whose own file (or a prerequisite
//! module's file) is absent is skipped: one `gameplay_module_skipped` line names the module
//! and the absent file, the module stays inactive, and `Dormant` keeps the save's state for
//! it so a smaller content set never wipes a character. Everything that IS present stays
//! fail-closed: malformed content, a campaign marker whose hash no longer matches, or a saved
//! snapshot that disagrees with content that IS present still refuses the load, because a
//! server must never run half-matching content.

use super::{Gameplay, GameplayState};
use crate::{
    combat::{Combat, CombatState},
    cooking::CookingState,
    encounter::EncounterState,
    forging::ForgingState,
    housing_service::HousingServiceState,
    inventory::{Inventory, InventoryState},
    item_categories::ItemCategories,
    mining::Type8State,
    opening::OpeningState,
    profiles::ProfileState,
    quests::{QuestEngine, QuestState},
    smelting::SmeltingState,
};
use serde_json::{Value, json};
use std::io;
use std::path::Path;
use std::sync::Arc;

/// What each gated module reads, its own file first. `Presence::has` names the first absent
/// one in the skip event. A prerequisite is listed because the module's loader reads it: the
/// opening campaign builds its actor template from `combat.json` and its journal from
/// `quests.json`.
pub(super) const PROFILES: &[&str] = &["profiles.json"];
pub(super) const OPENING: &[&str] = &[
    "opening.json",
    "profiles.json",
    "quests.json",
    "combat.json",
    "opening-abilities.json",
    "encounters.json",
];
pub(super) const INVENTORY: &[&str] = &["inventory.json"];
pub(super) const QUESTS: &[&str] = &["quests.json"];
pub(super) const COMBAT: &[&str] = &["combat.json"];
pub(super) const CATEGORIES: &[&str] = &["categories.json"];

/// The modules one load skipped, in load order. Each row is the record `npc_event` wrote, so
/// a test can assert what the server logged without reading the process's stdout. Clone is
/// cheap while unchanged; `record_skip` copy-on-writes the `Arc` only once a staged clone
/// diverges from its session.
#[derive(Clone, Debug, Default)]
pub(super) struct Presence {
    skipped: Vec<Value>,
}

impl Presence {
    /// `true` when every file in `files` is under `directory`; otherwise the module is
    /// skipped and its event is logged here (once per process, see `record`).
    pub(super) fn has(&mut self, directory: &Path, module: &'static str, files: &[&str]) -> bool {
        let Some(missing) = files.iter().find(|file| !directory.join(file).is_file()) else {
            return true;
        };
        self.record(module, missing);
        false
    }

    /// A crafting or gathering movie: it needs its own file and the opening campaign that
    /// hosts it. The file is named first, then the campaign, so the log says what to add.
    pub(super) fn minigame(
        &mut self,
        directory: &Path,
        module: &'static str,
        file: &str,
        opening: bool,
    ) -> bool {
        if !self.has(directory, module, &[file]) {
            return false;
        }
        if opening {
            return true;
        }
        self.record(module, "the opening campaign");
        false
    }

    /// Record and log one skipped module. Public to the `gameplay` children that own a load
    /// step of their own (`bind_npc_quest_catalogue`).
    pub(super) fn record(&mut self, module: &'static str, missing: &str) {
        let record = json!({
            "event": "gameplay_module_skipped",
            "module": module,
            "missing": missing,
        });
        // The server builds its template and its first session in the same start, and every
        // login after that re-reads the content: one line per module per process is what a
        // start log should show, not one per load.
        if logged()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(record.to_string())
        {
            super::npc_event(record.clone());
        }
        self.skipped.push(record);
    }
}

/// The skip records this process has already written.
fn logged() -> &'static std::sync::Mutex<std::collections::BTreeSet<String>> {
    static LOGGED: std::sync::OnceLock<std::sync::Mutex<std::collections::BTreeSet<String>>> =
        std::sync::OnceLock::new();
    LOGGED.get_or_init(Default::default)
}

/// The three core modules whose content-absent form is an empty engine rather than a saved
/// state: no item-category list, no legacy checkpoint quests, no training sandbox. Each keeps
/// its own `without_content` constructor and the same fail-closed checks when its file exists.
pub(super) fn core(
    directory: &Path,
    presence: &mut Presence,
    quests: Option<QuestState>,
    combat: Option<CombatState>,
) -> io::Result<(Option<ItemCategories>, QuestEngine, Combat)> {
    let categories = if presence.has(directory, "categories", CATEGORIES) {
        Some(ItemCategories::load(directory)?)
    } else {
        None
    };
    let quests = if presence.has(directory, "quests", QUESTS) {
        QuestEngine::load(directory, quests)?
    } else {
        QuestEngine::without_content(quests)
    };
    let combat = if presence.has(directory, "combat", COMBAT) {
        Combat::load(directory, combat).map_err(super::invalid)?
    } else {
        Combat::without_content(combat)
    };
    Ok((categories, quests, combat))
}

/// The item catalogue, or the no-content inventory that keeps the save's coins and instances
/// but can define, grant or equip nothing.
pub(super) fn inventory(
    directory: &Path,
    presence: &mut Presence,
    state: Option<InventoryState>,
) -> io::Result<Inventory> {
    if presence.has(directory, "inventory", INVENTORY) {
        Inventory::load(directory, state)
    } else {
        Ok(Inventory::without_content(state))
    }
}

/// The activity portal publishes battle rows only when the encounter engine that runs them
/// loaded: a row without its engine offers a Play the server must refuse (the rule chess,
/// soccer and cards already follow). A content set without `opening.json` builds no engine.
pub(super) fn battle_rows<'a>(file: &'a Path, engine_loaded: bool) -> Option<&'a Path> {
    (file.is_file() && engine_loaded).then_some(file)
}

/// Record a skip found outside `Gameplay::load`: the regional quest catalogue binds after
/// the load. Copy-on-write, so a staged session clone never mutates another session's records.
pub(super) fn record_skip(presence: &mut Arc<Presence>, module: &'static str, missing: &str) {
    Arc::make_mut(presence).record(module, missing);
}

impl Gameplay {
    /// T852: the `gameplay_module_skipped` records this load wrote, `(module, absent file)`.
    pub fn skipped_modules(&self) -> &[Value] {
        &self.presence.skipped
    }
}

/// The saved state of the modules this content set skips. `Gameplay::state` writes each field
/// back while its module is inactive, so a server started with less content leaves a
/// character's progress unchanged instead of wiping it.
///
/// Only a module that did NOT load may hold a value here: an active module's live state is
/// authoritative, so a module added to `Gameplay` must decide at its own load site whether to
/// record the saved value here. A field that is empty simply has nothing to preserve.
#[derive(Debug, Default)]
pub(super) struct Dormant {
    pub profiles: Option<ProfileState>,
    pub opening: Option<OpeningState>,
    pub encounter: Option<EncounterState>,
    pub smelting: Option<SmeltingState>,
    pub forging: Option<ForgingState>,
    pub cooking: Option<CookingState>,
    pub mining: Option<Type8State>,
    pub harvesting: Option<Type8State>,
    pub housing: Option<HousingServiceState>,
}

/// Write every skipped module's saved state back into `state`. A module that loaded already
/// filled its field, so the live value wins; `Dormant` only ever holds a skipped module.
pub(super) fn restore_skipped(state: &mut GameplayState, dormant: &Dormant) {
    fn keep<T: Clone>(live: &mut Option<T>, saved: &Option<T>) {
        if live.is_none() {
            *live = saved.clone();
        }
    }
    keep(&mut state.profiles, &dormant.profiles);
    keep(&mut state.opening, &dormant.opening);
    keep(&mut state.encounter, &dormant.encounter);
    keep(&mut state.smelting, &dormant.smelting);
    keep(&mut state.forging, &dormant.forging);
    keep(&mut state.cooking, &dormant.cooking);
    keep(&mut state.mining, &dormant.mining);
    keep(&mut state.harvesting, &dormant.harvesting);
    keep(&mut state.housing, &dormant.housing);
}
