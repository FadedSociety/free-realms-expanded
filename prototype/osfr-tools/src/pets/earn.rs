//! Earned pets: a pet is a reward for playing, never a coin purchase (owner decision 2026-11-09).
//!
//! `pet_rewards.json` in the content directory (shipped: `data/content/pet_rewards.json`) lists
//! one row per earnable pet adoption item: a counter, the count it needs and the item whose pet
//! the owner then gets. Every shipped row is reconstructed (R) by theme and tunable: the live
//! game sold almost every pet, and the few it gave away (wiki "Special Pets (FR)": Daily Wheel,
//! TCG reward cards, boss chests, events) came from sources this server does not run yet.
//!
//! Counters are facts the server already settled: a quest turned in, a dungeon cleared, a
//! minigame with a result, an accepted pet trick, a feed applied, a grooming session that ran to
//! its end (a full bar or its time limit), a treasure chest opened, a job rank reached, a day on
//! which a paid completion landed. They are bumped where those facts are raised, and only the
//! counters some row names are stored, so the saved state stays bounded by the table. The table
//! may cap how much a counter grows per day (`daily_caps`): pet care has no cooldown of its own.
//!
//! A due row is claimed on the pet tick, which owns the pet clock: the pet is inserted straight
//! into the roster (the client reads only the 53/5 roster, `00b596f0`, and its list reader
//! `00b5aea0` has no count limit), and the row key is recorded so it never pays twice. A row's
//! key is its pet item, not its goal, so retuning a ladder never pays an owned pet again. A full
//! roster leaves the row due; it is claimed when there is room. A content directory without the
//! file earns nothing (dormant by default).

use super::*;

/// The earn table's file in the content directory.
pub const EARN_FILE: &str = "pet_rewards.json";
/// A save mark for counters alone is spread out, so a string of kills is one save.
const COUNTER_SAVE_MS: u64 = 30_000;
/// (R) Sanity bound on one row's goal; a larger value is a typo, not a design.
const MAX_GOAL: u32 = 1_000_000;

/// One earnable pet.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EarnRow {
    /// Stable id of the row and the saved "already earned" record: `pet#<item id>` in the shipped
    /// table, so a retuned counter or goal keeps the record.
    pub key: String,
    /// The counter this row watches (`quest`, `dungeon`, `minigame:12`, `pet_trick`, ...).
    pub counter: String,
    /// The count that earns it.
    pub at: u32,
    /// The pet adoption item (`pets.json` `items`) whose pet is created.
    pub item_id: i32,
    /// The pet's name for the chat line.
    pub name: String,
    /// The goal in the player's words ("Clear 3 dungeons").
    pub goal: String,
    /// `reconstructed` (R) for every shipped row; `original` is for a row that uses the live
    /// game's own source, once one is run.
    pub kind: String,
    /// Why this pet sits on this goal.
    pub basis: String,
}

/// The earn table.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EarnRules {
    schema_version: u32,
    enabled: bool,
    #[serde(default)]
    provenance: serde_json::Value,
    /// (R) The most a counter may grow in one activity day; a counter not listed is unlimited.
    #[serde(default)]
    daily_caps: BTreeMap<String, u32>,
    rows: Vec<EarnRow>,
    /// Pets with no row, each with its reason; documentation for the next builder.
    #[serde(default)]
    unwired: Vec<serde_json::Value>,
    #[serde(skip)]
    counters: BTreeSet<String>,
}

impl EarnRules {
    /// Parse and check a table against the pet content: unique keys, sane goals, and every item a
    /// pet adoption item the content can turn into a pet.
    pub fn from_json(bytes: &[u8], content: &PetContent) -> io::Result<Self> {
        let mut rules: Self = serde_json::from_slice(bytes).map_err(parse_error)?;
        if rules.schema_version != 1 {
            return Err(invalid("unsupported pet rewards schema"));
        }
        if rules.daily_caps.values().any(|&cap| cap == 0) {
            return Err(invalid("a pet reward daily cap must be at least 1"));
        }
        let mut keys = BTreeSet::new();
        for row in &rules.rows {
            let sound = !row.key.is_empty()
                && !row.counter.is_empty()
                && (1..=MAX_GOAL).contains(&row.at)
                && matches!(row.kind.as_str(), "original" | "reconstructed")
                && !row.name.is_empty()
                && !row.goal.is_empty()
                && keys.insert(row.key.as_str())
                && content.definition_for_item(row.item_id).is_some();
            if !sound {
                return Err(invalid("invalid pet reward row"));
            }
        }
        rules
            .rows
            .sort_by(|a, b| (&a.counter, a.at, &a.key).cmp(&(&b.counter, b.at, &b.key)));
        rules.counters = rules.rows.iter().map(|row| row.counter.clone()).collect();
        Ok(rules)
    }

    /// The table of a content directory; none without the file. A broken file disables earning
    /// and says so instead of stopping every login; `tests/pet_earn.rs` checks the shipped file.
    fn load(directory: &std::path::Path, content: &PetContent) -> Self {
        let path = directory.join(EARN_FILE);
        if !path.is_file() {
            return Self::default();
        }
        std::fs::read(&path)
            .and_then(|bytes| Self::from_json(&bytes, content))
            .unwrap_or_else(|error| {
                eprintln!("pet rewards disabled ({}): {error}", path.display());
                Self::default()
            })
    }

    pub fn rows(&self) -> &[EarnRow] {
        &self.rows
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// The daily cap of a counter, if the table sets one.
    pub fn daily_cap(&self, counter: &str) -> Option<u32> {
        self.daily_caps.get(counter).copied()
    }

    /// Where the table came from (generator, sources).
    pub fn provenance(&self) -> &serde_json::Value {
        &self.provenance
    }

    /// The pets with no row, each with its reason.
    pub fn unwired(&self) -> &[serde_json::Value] {
        &self.unwired
    }
}

/// The saved part: counters and the rows already paid. Empty until the first tracked fact, so an
/// older roster round-trips byte for byte.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EarnState {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    counts: BTreeMap<String, u32>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    granted: BTreeSet<String>,
    /// The last activity day counted for `active_day`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_day: Option<u64>,
    /// The activity day `capped` belongs to, and what each capped counter gained on it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    capped_day: Option<u64>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    capped: BTreeMap<String, u32>,
}

impl EarnState {
    pub(super) fn is_empty(&self) -> bool {
        self.counts.is_empty()
            && self.granted.is_empty()
            && self.last_day.is_none()
            && self.capped.is_empty()
    }
}

/// Session-only bookkeeping beside the saved state.
#[derive(Clone, Debug)]
pub(super) struct EarnSession {
    rules: Arc<EarnRules>,
    /// A counter moved (or the session just began), so due rows must be looked for.
    check: bool,
    /// A counter moved since the last save mark, and the first tick that saw it (session ms).
    unsaved: bool,
    unsaved_since: Option<u64>,
    /// The full-roster notice was already given this session.
    full_told: bool,
    /// The last second the bag was searched for unadopted pet items.
    swept_at: Option<u64>,
    /// Today's activity day, set by the pet tick; the daily caps count against it.
    day: u64,
    /// Every item definition that becomes a pet, for the bag search of `adopt_bag_items`.
    adoption_items: Arc<BTreeSet<i32>>,
}

impl EarnSession {
    pub(super) fn new(adoption_items: Arc<BTreeSet<i32>>) -> Self {
        Self {
            rules: Arc::new(EarnRules::default()),
            check: true,
            unsaved: false,
            unsaved_since: None,
            full_told: false,
            swept_at: None,
            day: 0,
            adoption_items,
        }
    }

    pub(super) fn reset(&mut self) {
        self.check = true;
        self.full_told = false;
        self.swept_at = None;
    }
}

/// One pet the owner just earned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EarnedPet {
    pub pet_id: u32,
    pub name: String,
    pub goal: String,
    pub counter: String,
    /// The next unearned pet on the same counter: name, goal, count so far, count needed.
    pub next: Option<(String, String, u32, u32)>,
}

/// What one claim step did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EarnClaim {
    pub earned: Vec<EarnedPet>,
    /// A pet is due but the roster is full; set once per session.
    pub roster_full: bool,
    /// The 53/5 roster, when a pet was added.
    pub replies: Vec<Vec<u8>>,
}

impl PetService {
    /// Use the earn table of this content directory (`pet_rewards.json`), if it has one.
    pub fn with_earn_rules_in(mut self, directory: &std::path::Path) -> Self {
        self.set_earn_rules(EarnRules::load(directory, &self.content));
        self
    }

    /// Replace the earn table.
    pub fn set_earn_rules(&mut self, rules: EarnRules) {
        self.earn.rules = Arc::new(rules);
        self.earn.check = true;
    }

    /// Keep `other`'s earn table: a roster reloaded from its document stays on the session's table.
    pub fn keep_earn_rules_of(&mut self, other: &Self) {
        self.earn.rules = Arc::clone(&other.earn.rules);
    }

    pub fn earn_rules(&self) -> &EarnRules {
        &self.earn.rules
    }

    /// The saved count of one counter.
    pub fn earn_count(&self, counter: &str) -> u32 {
        self.roster.earn.counts.get(counter).copied().unwrap_or(0)
    }

    /// Whether the row with this key was already paid.
    pub fn has_earned(&self, key: &str) -> bool {
        self.roster.earn.granted.contains(key)
    }

    /// Add `amount` to a counter. Counters no row names are not stored.
    pub fn earn_bump(&mut self, counter: &str, amount: u32) {
        if !self.tracks(counter) {
            return;
        }
        let amount = self.under_daily_cap(counter, amount);
        if amount == 0 {
            return;
        }
        let count = self
            .roster
            .earn
            .counts
            .entry(counter.to_owned())
            .or_insert(0);
        *count = count.saturating_add(amount);
        self.earn.check = true;
        self.earn.unsaved = true;
    }

    /// Today's activity day for the daily caps; the pet tick sets it.
    pub fn set_earn_day(&mut self, day: u64) {
        self.earn.day = day;
    }

    /// How much of `amount` today's cap on `counter` still allows, recorded as used.
    fn under_daily_cap(&mut self, counter: &str, amount: u32) -> u32 {
        let Some(cap) = self.earn.rules.daily_cap(counter) else {
            return amount;
        };
        let state = &mut self.roster.earn;
        if state.capped_day != Some(self.earn.day) {
            state.capped_day = Some(self.earn.day);
            state.capped.clear();
        }
        let used = state.capped.get(counter).copied().unwrap_or(0);
        let allowed = amount.min(cap.saturating_sub(used));
        if allowed > 0 {
            state.capped.insert(counter.to_owned(), used + allowed);
        }
        allowed
    }

    /// Raise a counter to `value` when it is below it (a job level, never a running total).
    pub fn earn_reach(&mut self, counter: &str, value: u32) {
        if value > self.earn_count(counter) && self.tracks(counter) {
            self.roster.earn.counts.insert(counter.to_owned(), value);
            self.earn.check = true;
            self.earn.unsaved = true;
        }
    }

    /// Count one activity day (`active_day`) the first time a completion lands on `day`.
    pub fn earn_day(&mut self, day: u64) {
        if self.roster.earn.last_day == Some(day) || !self.tracks("active_day") {
            return;
        }
        self.roster.earn.last_day = Some(day);
        self.earn_bump("active_day", 1);
    }

    fn tracks(&self, counter: &str) -> bool {
        self.earn.rules.enabled && self.earn.rules.counters.contains(counter)
    }

    /// True once counters have waited `COUNTER_SAVE_MS` for a save, so the caller marks one
    /// without saving on every tick. Any other save in between carries the counters too.
    pub fn take_earn_unsaved(&mut self, now_ms: u64) -> bool {
        if !self.earn.unsaved {
            return false;
        }
        let since = *self.earn.unsaved_since.get_or_insert(now_ms);
        if now_ms.saturating_sub(since) < COUNTER_SAVE_MS {
            return false;
        }
        self.earn.unsaved = false;
        self.earn.unsaved_since = None;
        true
    }

    /// Create the pet of every due row, oldest goal first. `now` is the pet clock in seconds.
    /// A full roster stops the step and leaves the rows due.
    pub fn claim_earned(&mut self, now: u64) -> EarnClaim {
        let mut claim = EarnClaim::default();
        if !std::mem::take(&mut self.earn.check) || !self.earn.rules.enabled {
            return claim;
        }
        let rules = Arc::clone(&self.earn.rules);
        for row in rules.rows() {
            if self.has_earned(&row.key) || self.earn_count(&row.counter) < row.at {
                continue;
            }
            if self.room() == 0 {
                claim.roster_full = !std::mem::replace(&mut self.earn.full_told, true);
                break;
            }
            let Some(definition) = self.content.definition_for_item(row.item_id) else {
                continue;
            };
            let mut next = self.roster.clone();
            let Ok(pet_id) = insert_pet(&self.content, &mut next, &definition, now) else {
                continue;
            };
            // (R) The item row's own tint is the coat: a bought unit carries it on the item
            // instance (`adopt_with_replies`), an earned pet has no instance.
            let coat = self
                .content
                .item(row.item_id)
                .and_then(|item| item.icon_tint_id);
            if let Some(pet) = next.pets.last_mut() {
                pet.tint = coat.filter(|&tint| tint != 0);
            }
            next.earn.granted.insert(row.key.clone());
            if validate_roster(&self.content, &next).is_err() {
                continue;
            }
            self.roster = next;
            claim.earned.push(EarnedPet {
                pet_id,
                name: row.name.clone(),
                goal: row.goal.clone(),
                counter: row.counter.clone(),
                next: None,
            });
        }
        for earned in &mut claim.earned {
            earned.next = self.next_on_counter(&rules, &earned.counter);
        }
        if !claim.earned.is_empty() {
            claim.replies.push(self.list_packet());
        }
        claim
    }

    /// The nearest unearned row on `counter`.
    fn next_on_counter(
        &self,
        rules: &EarnRules,
        counter: &str,
    ) -> Option<(String, String, u32, u32)> {
        let have = self.earn_count(counter);
        rules
            .rows()
            .iter()
            .find(|row| row.counter == counter && !self.has_earned(&row.key) && row.at > have)
            .map(|row| (row.name.clone(), row.goal.clone(), have, row.at))
    }

    /// Every row with the owner's progress: `(row, count so far, already earned)`.
    pub fn earn_progress(&self) -> impl Iterator<Item = (&EarnRow, u32, bool)> + '_ {
        self.earn.rules.rows().iter().map(|row| {
            (
                row,
                self.earn_count(&row.counter),
                self.has_earned(&row.key),
            )
        })
    }

    /// Pet adoption items a reward left in the bag (a quest or collection reward, a treasure
    /// chest, a full roster earlier) become pets, at most once a second. The buy and `realm give`
    /// paths adopt at once (`adopt_granted`); the reward applier replaces the inventory without a
    /// grant list, so its pet items are found here.
    pub fn adopt_bag_items(&mut self, inventory: &mut Inventory, now: u64) -> Vec<Vec<u8>> {
        if self.earn.swept_at == Some(now) || self.room() == 0 {
            return Vec::new();
        }
        self.earn.swept_at = Some(now);
        let waiting: Vec<(i32, u32)> = inventory
            .state()
            .items
            .iter()
            .filter(|item| self.earn.adoption_items.contains(&item.definition_id))
            .map(|item| (item.id, item.count))
            .collect();
        // One item at a time: an item that cannot be adopted is skipped, and the pets already
        // adopted keep their inventory replies and the roster packet.
        let mut replies = Vec::new();
        for (instance_id, units) in waiting {
            let definition = inventory
                .state()
                .items
                .iter()
                .find(|item| item.id == instance_id)
                .and_then(|item| self.content.definition_for_item(item.definition_id));
            let Some(definition) = definition else {
                continue;
            };
            for _ in 0..units.min(self.room() as u32) {
                match self.adopt_with_replies(inventory, instance_id, &definition, now) {
                    Ok((_, adopted)) => replies.extend(adopted),
                    Err(_) => break,
                }
            }
        }
        if !replies.is_empty() {
            replies.push(self.list_packet());
        }
        replies
    }
}
