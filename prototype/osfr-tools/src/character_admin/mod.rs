//! G3 (gm-panel): staff character and inventory editing, the plain types and the parts that need
//! no session. One core for every front end (the web panel's routes, the `realm` chat words):
//!
//! - the gameplay side, `Gameplay::admin_edit` and the read views, lives in `live.rs`, compiled
//!   as a child of `gameplay` (`gameplay/admin_move.rs` includes it) so an edit goes through the
//!   same inventory, quest and job paths a quest reward, a vendor or `realm give` uses;
//! - the quest engine's one admin setter lives in `quest_set.rs`, a child of `quest_engine`;
//! - the gateway host (`bin/osfr_gateway/character_admin.rs`) finds the character: an online one
//!   is edited on a clone of its own session and committed through that session (client packets,
//!   save), an offline one is loaded through the login path, edited the same way and saved after
//!   a timestamped backup.
//!
//! Every write returns an [`EditRecord`]: the touched coins, stacks, equipment rows, quests and
//! job XP before and after. [`Edit::Apply`] with [`Change::inverse`] reverses it, refused when
//! the character changed since (no lost update).
use crate::{
    account_store::AccountRole,
    gameplay::GameplayState,
    inventory::{EquippedItem, Inventory, InventoryItem},
    quest_engine::QuestProgress,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fmt, io, path::Path};

/// (R) Coins one give or take may move, the bound `realm coins` and the GM wheel already use.
pub const MAX_COIN_STEP: u32 = 1_000_000;
/// (R) The XP a job may be set to: the shaped curves top out at 10,000,000 (`profiles.rs`).
pub const MAX_JOB_XP: u32 = 10_000_000;
/// (R) Removal rows one call may carry.
pub const MAX_REMOVALS: usize = 64;
/// (R) Item search results per page.
pub const SEARCH_PAGE: usize = 50;
const MAX_QUERY: usize = 64;

/// Who is asking: the verified staff account and its stored role.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Actor {
    pub account_id: u64,
    pub role: AccountRole,
}

impl Actor {
    /// Moderators see a character (PLAN 4.8: read-only views); admins see it too.
    pub fn may_read(&self) -> bool {
        matches!(self.role, AccountRole::Admin | AccountRole::Moderator)
    }

    /// Writes are admin only. `can_run_realm_command` makes every unknown word admin-only, but a
    /// Tester holds every key for its own session, so the role is named here, as L090 does for
    /// the panel's moves.
    pub fn may_write(&self) -> bool {
        self.role == AccountRole::Admin
            && crate::account_store::can_run_realm_command(self.role, "take")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdminError {
    /// The actor's role may not do this.
    Forbidden,
    /// No such character (or it never entered the world).
    NotFound(String),
    /// The game's rules refuse it (unknown item, bad count, full bag, ...). Nothing changed.
    Refused(String),
    /// The character is busy or changed under the edit (loading, a stale undo). Nothing changed.
    Conflict(String),
    /// A save or session failure.
    Failed(String),
}

impl fmt::Display for AdminError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Forbidden => f.write_str("admins only"),
            Self::NotFound(text)
            | Self::Refused(text)
            | Self::Conflict(text)
            | Self::Failed(text) => f.write_str(text),
        }
    }
}

impl std::error::Error for AdminError {}

/// The game paths refuse with `io::Error` sentences ("inventory is full", "not enough coins").
impl From<io::Error> for AdminError {
    fn from(error: io::Error) -> Self {
        Self::Refused(error.to_string())
    }
}

fn refused(text: impl Into<String>) -> AdminError {
    AdminError::Refused(text.into())
}

/// One bag stack: `slot` is the instance id the client and removals name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ItemRow {
    pub slot: i32,
    pub item_id: i32,
    pub name: String,
    pub count: u32,
    pub icon: i32,
}

/// One worn item: the job (profile) and equipment slot, and the bag stack it wears.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EquippedRow {
    pub profile_id: i32,
    pub slot: i32,
    pub instance: i32,
    pub item_id: i32,
    pub name: String,
    pub icon: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InventoryView {
    pub coins: u32,
    pub bags: Vec<ItemRow>,
    pub equipped: Vec<EquippedRow>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct JobRow {
    pub job_id: i32,
    pub level: u32,
    pub xp: u32,
    pub active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct QuestRow {
    pub quest_id: u32,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MountRow {
    pub mount_id: i32,
    pub name: String,
}

/// The profile facts G1's player page shows beside identity and moderation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CharacterView {
    pub coins: u32,
    pub jobs: Vec<JobRow>,
    pub quests_active: Vec<QuestRow>,
    pub quests_done: Vec<QuestRow>,
    pub mounts: Vec<MountRow>,
    pub pets: Vec<String>,
    /// Completed collections, and collected entries across all of them.
    pub collections_n: usize,
    pub collection_entries: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ItemHit {
    pub item_id: i32,
    pub name: String,
    pub icon: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ItemPage {
    pub total: usize,
    /// Zero-based.
    pub page: usize,
    pub rows: Vec<ItemHit>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pick {
    /// A bag stack by instance id ([`ItemRow::slot`]).
    Slot(i32),
    /// Every stack of an item definition.
    Item(i32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Amount {
    Count(u32),
    All,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Removal {
    pub pick: Pick,
    pub amount: Amount,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoinOp {
    Give,
    Take,
    Set,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuestOp {
    /// Active from the start, as if accepted (prerequisites are not asked).
    Grant,
    /// Done without its turn-in reward.
    Complete,
    /// Forgotten, so it can be offered again.
    Reset,
    /// An active quest dropped, as the player's own abandon does.
    Abandon,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobSet {
    /// The start of a level (1-20) on the job's own curve.
    Level(u32),
    Xp(u32),
}

/// One staff write. Every one runs on the character's gameplay (`Gameplay::admin_edit`).
#[derive(Clone, Debug, PartialEq)]
pub enum Edit {
    AddItems {
        item_id: i32,
        count: u32,
        /// Wear it on the active job (the bag otherwise).
        equip: bool,
    },
    RemoveItems(Vec<Removal>),
    Coins(CoinOp, u32),
    Quest(QuestOp, u32),
    Job(i32, JobSet),
    /// Move every touched part from `change`'s before to its after, refused unless the character
    /// is still at the before. An undo is `Apply(record.change.inverse())`.
    Apply(Change),
}

impl Edit {
    /// Bounds checked before anything loads: a bad count or amount is refused for every target.
    pub fn check(&self) -> Result<(), AdminError> {
        match self {
            Self::AddItems { item_id, count, .. } => {
                if *item_id <= 0 {
                    return Err(refused(format!("{item_id} is not an item id")));
                }
                if *count == 0 {
                    return Err(refused("the count must be at least 1"));
                }
            }
            Self::RemoveItems(rows) => {
                if rows.is_empty() || rows.len() > MAX_REMOVALS {
                    return Err(refused(format!(
                        "remove 1 to {MAX_REMOVALS} rows at a time"
                    )));
                }
                if rows.iter().any(|row| row.amount == Amount::Count(0)) {
                    return Err(refused("the count must be at least 1"));
                }
            }
            Self::Coins(op, amount) => {
                if *op != CoinOp::Set && (*amount == 0 || *amount > MAX_COIN_STEP) {
                    return Err(refused(format!(
                        "give or take 1 to {MAX_COIN_STEP} coins at a time"
                    )));
                }
                if i32::try_from(*amount).is_err() {
                    return Err(refused("that balance does not fit the coin field"));
                }
            }
            Self::Job(_, JobSet::Level(level)) if !(1..=20).contains(level) => {
                return Err(refused("the level must be 1 to 20"));
            }
            Self::Job(_, JobSet::Xp(xp)) if *xp > MAX_JOB_XP => {
                return Err(refused(format!("the XP must be at most {MAX_JOB_XP}")));
            }
            Self::Quest(_, 0) => return Err(refused("0 is not a quest id")),
            Self::Apply(change) if change.is_empty() => {
                return Err(refused("the record changes nothing"));
            }
            _ => {}
        }
        Ok(())
    }

    /// A short line for the audit row and the chat reply (ids and numbers only).
    pub fn summary(&self) -> String {
        match self {
            Self::AddItems {
                item_id,
                count,
                equip,
            } => format!(
                "add {count} x item {item_id}{}",
                if *equip { ", worn" } else { "" }
            ),
            Self::RemoveItems(rows) => format!("remove {} row(s)", rows.len()),
            Self::Coins(op, amount) => format!("coins {op:?} {amount}").to_lowercase(),
            Self::Quest(op, id) => format!("quest {op:?} {id}").to_lowercase(),
            Self::Job(job, JobSet::Level(level)) => format!("job {job} level {level}"),
            Self::Job(job, JobSet::Xp(xp)) => format!("job {job} xp {xp}"),
            Self::Apply(_) => "undo".to_owned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StackChange {
    pub id: i32,
    pub before: Option<InventoryItem>,
    pub after: Option<InventoryItem>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QuestChange {
    pub quest_id: u32,
    pub before: Option<QuestProgress>,
    pub after: Option<QuestProgress>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobChange {
    pub job_id: i32,
    pub before: u32,
    pub after: u32,
}

/// What one edit touched, before and after. Session-only and untracked state (an adopted pet,
/// the opening chain, the item-id allocator) is not in it, so an undo leaves those alone.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Change {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coins: Option<[u32; 2]>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stacks: Vec<StackChange>,
    /// Rows keyed by (profile, slot); a row the save lacked counts as empty (item 0).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub equipment: Vec<[EquippedItem; 2]>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub quests: Vec<QuestChange>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub jobs: Vec<JobChange>,
}

impl Change {
    pub fn between(before: &GameplayState, after: &GameplayState) -> Self {
        let (old, new) = (&before.inventory, &after.inventory);
        let ids: BTreeSet<i32> = old.items.iter().chain(&new.items).map(|i| i.id).collect();
        let stack = |items: &[InventoryItem], id| items.iter().find(|i| i.id == id).cloned();
        let stacks = ids
            .into_iter()
            .map(|id| StackChange {
                id,
                before: stack(&old.items, id),
                after: stack(&new.items, id),
            })
            .filter(|s| s.before != s.after)
            .collect();
        let equipment = new
            .equipment
            .iter()
            .filter_map(|row| {
                let was = old
                    .equipment
                    .iter()
                    .find(|o| o.profile_id == row.profile_id && o.slot == row.slot)
                    .cloned()
                    .unwrap_or(EquippedItem {
                        item_id: 0,
                        ..row.clone()
                    });
                (was.item_id != row.item_id).then(|| [was, row.clone()])
            })
            .collect();
        let (qb, qa) = (quest_map(before), quest_map(after));
        let quests = qb
            .keys()
            .chain(qa.keys())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|id| QuestChange {
                quest_id: *id,
                before: qb.get(id).cloned(),
                after: qa.get(id).cloned(),
            })
            .filter(|q| q.before != q.after)
            .collect();
        let (jb, ja) = (job_map(before), job_map(after));
        let jobs = jb
            .keys()
            .chain(ja.keys())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|id| JobChange {
                job_id: *id,
                before: jb.get(id).copied().unwrap_or(0),
                after: ja.get(id).copied().unwrap_or(0),
            })
            .filter(|j| j.before != j.after)
            .collect();
        Self {
            coins: (old.coins != new.coins).then_some([old.coins, new.coins]),
            stacks,
            equipment,
            quests,
            jobs,
        }
    }

    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    pub fn touches_inventory(&self) -> bool {
        self.coins.is_some() || !self.stacks.is_empty() || !self.equipment.is_empty()
    }

    /// The same change backwards: applying it is the undo.
    pub fn inverse(&self) -> Self {
        Self {
            coins: self.coins.map(|[b, a]| [a, b]),
            stacks: self
                .stacks
                .iter()
                .map(|s| StackChange {
                    id: s.id,
                    before: s.after.clone(),
                    after: s.before.clone(),
                })
                .collect(),
            equipment: self
                .equipment
                .iter()
                .map(|[b, a]| [a.clone(), b.clone()])
                .collect(),
            quests: self
                .quests
                .iter()
                .map(|q| QuestChange {
                    quest_id: q.quest_id,
                    before: q.after.clone(),
                    after: q.before.clone(),
                })
                .collect(),
            jobs: self
                .jobs
                .iter()
                .map(|j| JobChange {
                    job_id: j.job_id,
                    before: j.after,
                    after: j.before,
                })
                .collect(),
        }
    }
}

fn quest_map(state: &GameplayState) -> std::collections::BTreeMap<u32, QuestProgress> {
    state
        .npc_quests
        .as_ref()
        .map(|quests| quests.quests.clone())
        .unwrap_or_default()
}

fn job_map(state: &GameplayState) -> std::collections::BTreeMap<i32, u32> {
    state
        .profiles
        .as_ref()
        .map(|profiles| profiles.experience.clone())
        .unwrap_or_default()
}

/// One applied write, for the audit log and a later undo.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditRecord {
    pub target: u64,
    pub actor_account: u64,
    pub at_unix: u64,
    pub summary: String,
    pub change: Change,
}

/// What a write reports back. `backup` is the offline save's copy taken before the write.
#[derive(Clone, Debug, PartialEq)]
pub struct Receipt {
    pub record: EditRecord,
    pub online: bool,
    pub backup: Option<std::path::PathBuf>,
}

/// An item's name (the client locale's text, `#id` without one) and icon id.
pub fn item_label(inventory: &Inventory, item_id: i32) -> (String, i32) {
    match inventory.presentation(item_id) {
        Ok(look) => (
            crate::world_text::locale_text(look.name_id).unwrap_or_else(|| format!("#{item_id}")),
            look.icon_id,
        ),
        Err(_) => (format!("#{item_id}"), 0),
    }
}

/// Every item definition the inventory accepts (`inventory.json`, `item-catalogue.json`,
/// `item-table.json`, `item-table-supplement.json` in `content`), by name substring (any case)
/// or by id, a page of [`SEARCH_PAGE`].
// ponytail: one pass over the table per search (~58k rows); a cached index if the panel's
// typing ever makes it slow.
pub fn search_items(content: &Path, query: &str, page: usize) -> io::Result<ItemPage> {
    search_in(&Inventory::load(content, None)?, query, page)
}

/// [`search_items`] over item tables already loaded (the panel keeps one copy).
pub fn search_in(inventory: &Inventory, query: &str, page: usize) -> io::Result<ItemPage> {
    let query = query.trim();
    if query.len() > MAX_QUERY {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "query is too long",
        ));
    }
    let needle = query.to_lowercase();
    let id = query.parse::<i32>().ok();
    let hits: Vec<ItemHit> = inventory
        .definition_ids()
        .filter_map(|item_id| {
            let (name, icon) = item_label(inventory, item_id);
            let hit = needle.is_empty()
                || id == Some(item_id)
                || (id.is_none() && name.to_lowercase().contains(&needle));
            hit.then_some(ItemHit {
                item_id,
                name,
                icon,
            })
        })
        .collect();
    Ok(ItemPage {
        total: hits.len(),
        page,
        rows: hits
            .into_iter()
            .skip(page.saturating_mul(SEARCH_PAGE))
            .take(SEARCH_PAGE)
            .collect(),
    })
}

/// A staff chat word for another character (`realm give|take|coins <First Last> ...`).
#[derive(Clone, Debug, PartialEq)]
pub struct ChatCommand {
    /// The typed display name; the host resolves it in the account store.
    pub name: String,
    pub edit: Edit,
}

pub const GIVE_USAGE: &str = "Usage: realm give <First Last> <item id> [count]. Nothing changed.";
pub const TAKE_USAGE: &str =
    "Usage: realm take <First Last> <item id> <count|all>. Nothing changed.";
pub const COINS_USAGE: &str =
    "Usage: realm coins <First Last> give|take|set <amount>. Nothing changed.";

/// `command` is the lowercased text after `realm `. `None` leaves it to the existing words:
/// `give <item> [count]` and `coins <n>` for the sender's own character. Names hold no digits
/// (`identity::valid_name`), so the trailing words are the item, count, op and amount; a
/// negative or out-of-range number is a usage error.
pub fn parse_chat(command: &str) -> Option<Result<ChatCommand, &'static str>> {
    let words: Vec<&str> = command.split_whitespace().collect();
    let named = |rest: &[&str]| rest.first().is_some_and(|w| w.parse::<f64>().is_err());
    let (usage, parsed) = match words.as_slice() {
        ["give", rest @ ..] if named(rest) => (GIVE_USAGE, parse_give(rest)),
        ["take", rest @ ..] => (TAKE_USAGE, parse_take(rest)),
        ["coins", rest @ ..] if named(rest) => (COINS_USAGE, parse_coins(rest)),
        _ => return None,
    };
    Some(parsed.ok_or(usage))
}

/// The display name in front of `tail` trailing words: at least one word, none a number.
fn name_before(rest: &[&str], tail: usize) -> Option<String> {
    let name = rest.get(..rest.len().checked_sub(tail)?)?;
    (!name.is_empty() && name.iter().all(|w| w.parse::<f64>().is_err())).then(|| name.join(" "))
}

fn parse_give(rest: &[&str]) -> Option<ChatCommand> {
    let (tail, item, count) = match rest {
        [.., item, count] if count.parse::<u32>().is_ok() && item.parse::<i32>().is_ok() => {
            (2, item, count.parse().ok()?)
        }
        [.., item] => (1, item, 1),
        [] => return None,
    };
    Some(ChatCommand {
        name: name_before(rest, tail)?,
        edit: Edit::AddItems {
            item_id: item.parse().ok()?,
            count,
            equip: false,
        },
    })
}

fn parse_take(rest: &[&str]) -> Option<ChatCommand> {
    let [.., item, amount] = rest else {
        return None;
    };
    let amount = match *amount {
        "all" => Amount::All,
        count => Amount::Count(count.parse().ok()?),
    };
    Some(ChatCommand {
        name: name_before(rest, 2)?,
        edit: Edit::RemoveItems(vec![Removal {
            pick: Pick::Item(item.parse().ok()?),
            amount,
        }]),
    })
}

fn parse_coins(rest: &[&str]) -> Option<ChatCommand> {
    let [.., op, amount] = rest else {
        return None;
    };
    let op = match *op {
        "give" => CoinOp::Give,
        "take" => CoinOp::Take,
        "set" => CoinOp::Set,
        _ => return None,
    };
    Some(ChatCommand {
        name: name_before(rest, 2)?,
        edit: Edit::Coins(op, amount.parse().ok()?),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_words_parse_from_the_end_and_leave_the_self_forms_alone() {
        let parse = |text: &str| parse_chat(text).map(|r| r.map(|c| (c.name, c.edit)));
        assert_eq!(
            parse("give 1234 5"),
            None,
            "realm give <item> stays the self give"
        );
        assert_eq!(
            parse("coins 500"),
            None,
            "realm coins <n> stays the self grant"
        );
        assert_eq!(parse("help"), None);
        let add = |count| Edit::AddItems {
            item_id: 1234,
            count,
            equip: false,
        };
        assert_eq!(
            parse("give pip test 1234 5"),
            Some(Ok(("pip test".into(), add(5))))
        );
        assert_eq!(
            parse("give pip test 1234"),
            Some(Ok(("pip test".into(), add(1))))
        );
        assert_eq!(
            parse("take pip test 77 all"),
            Some(Ok((
                "pip test".into(),
                Edit::RemoveItems(vec![Removal {
                    pick: Pick::Item(77),
                    amount: Amount::All
                }])
            )))
        );
        assert_eq!(
            parse("coins pip test set 40"),
            Some(Ok(("pip test".into(), Edit::Coins(CoinOp::Set, 40))))
        );
        for (text, usage) in [
            ("give pip test 1234 -5", GIVE_USAGE),
            ("give pip", GIVE_USAGE),
            ("take pip test 77", TAKE_USAGE),
            ("take 77 3", TAKE_USAGE),
            ("take pip test 77 -1", TAKE_USAGE),
            ("take pip test 77 99999999999", TAKE_USAGE),
            ("coins pip test steal 5", COINS_USAGE),
            ("coins pip test give -5", COINS_USAGE),
            ("coins pip give", COINS_USAGE),
        ] {
            assert_eq!(parse(text), Some(Err(usage)), "{text}");
        }
    }

    #[test]
    fn bounds_refuse_zero_negative_and_huge_amounts_before_any_load() {
        let add = |item_id, count| Edit::AddItems {
            item_id,
            count,
            equip: false,
        };
        assert!(add(5, 0).check().is_err());
        assert!(add(5, 1).check().is_ok());
        assert!(add(-3, 1).check().is_err());
        assert!(Edit::Coins(CoinOp::Give, 0).check().is_err());
        assert!(
            Edit::Coins(CoinOp::Give, MAX_COIN_STEP + 1)
                .check()
                .is_err()
        );
        assert!(Edit::Coins(CoinOp::Set, 0).check().is_ok());
        assert!(Edit::Coins(CoinOp::Set, u32::MAX).check().is_err());
        assert!(Edit::Job(1, JobSet::Level(21)).check().is_err());
        assert!(Edit::Job(1, JobSet::Xp(MAX_JOB_XP + 1)).check().is_err());
        assert!(Edit::RemoveItems(Vec::new()).check().is_err());
        assert!(Edit::Apply(Change::default()).check().is_err());
    }

    #[test]
    fn a_change_inverts_and_names_only_what_moved() {
        let item = |id, count| {
            serde_json::json!({"definition_id": 9, "tint": 0, "id": id, "count": count,
                "consumed_count": 0, "last_cast_time": 0, "rental": null, "ability_count": 0,
                "activate_enabled": false})
        };
        let state = |coins, items: Vec<serde_json::Value>| -> GameplayState {
            serde_json::from_value(serde_json::json!({
                "inventory": {"coins": coins, "items": items, "equipment": []},
                "quests": {"version": 1, "quests": {}, "trial_best_ms": null,
                    "trial_completions": 0},
                "combat": {"enemy_health": 0, "defeats": 0},
            }))
            .unwrap()
        };
        let before = state(10, vec![item(1, 2), item(2, 1)]);
        let after = state(10, vec![item(1, 5), item(3, 1)]);
        let change = Change::between(&before, &after);
        assert_eq!(change.coins, None);
        let ids: Vec<i32> = change.stacks.iter().map(|s| s.id).collect();
        assert_eq!(ids, [1, 2, 3]);
        assert_eq!(change.inverse().inverse(), change);
        assert_eq!(Change::between(&after, &before), change.inverse());
        assert!(Change::between(&before, &before).is_empty());
        assert_eq!(
            Change::between(&before, &state(4, Vec::new())).coins,
            Some([10, 4])
        );
    }
}
