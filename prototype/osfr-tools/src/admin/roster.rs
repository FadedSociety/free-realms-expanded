//! Who is on the server: the online list, the all-players search and a player's profile.
//!
//! The account store is the list of every account and character; a character's save
//! (`characters/<guid>.json`, `game_store`) holds its world state. Profile reads parse the save
//! as plain JSON and pick fields, so they never create, migrate or lock a save. The store keeps
//! only a hash of each login handle, so an account is shown and searched by its id.
use super::{audit, roles};
use crate::account_store::{AccountRole, AccountStore};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::Path;
use std::time::UNIX_EPOCH;

/// Search results per page (R).
pub const PAGE: usize = 50;
/// Moderation history and notes shown in a profile, newest first (R).
const HISTORY: usize = 50;
const MODERATION: [&str; 8] = [
    "mute", "unmute", "kick", "ban", "unban", "warn", "jail", "release",
];

/// One live session (`GET /admin/api/players/online`).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OnlineRow {
    pub character_id: u64,
    pub name: String,
    pub account: u64,
    pub role: Option<&'static str>,
    /// Absent while the session has no place in the world yet (loading).
    pub zone: Option<u32>,
    /// The zone's scene name as the session knows it (`Gateway::runtime_zone`), e.g. the open
    /// world's; null while loading.
    pub zone_name: Option<String>,
    pub position: Option<[f32; 3]>,
    pub job: Option<i32>,
    pub level: Option<i32>,
    pub session_s: u64,
    /// The original client and the Rust client send the same version string, so this stays
    /// null until a client marks itself.
    pub client: Option<&'static str>,
}

/// Name order, ignoring case: the panel's default.
pub fn sort_online(rows: &mut [OnlineRow]) {
    rows.sort_by_cached_key(|row| (row.name.to_lowercase(), row.character_id));
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Sort {
    #[default]
    Name,
    LastSeen,
    Created,
}

impl Sort {
    pub fn parse(text: &str) -> Option<Self> {
        Some(match text {
            "" | "name" => Self::Name,
            "last_seen" => Self::LastSeen,
            "created" => Self::Created,
            _ => return None,
        })
    }
}

/// `GET /admin/api/players` parameters.
#[derive(Clone, Debug, Default)]
pub struct Query {
    pub q: String,
    pub online: Option<bool>,
    pub role: Option<AccountRole>,
    pub banned: Option<bool>,
    pub muted: Option<bool>,
    pub sort: Sort,
    /// 1-based.
    pub page: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlayerRow {
    pub character_id: u64,
    pub name: String,
    pub account: u64,
    pub role: &'static str,
    pub online: bool,
    /// Unix seconds: now when online, else when the save last changed.
    pub last_seen: Option<u64>,
    /// Unix seconds when the save was first written (R: the file's creation time).
    pub created: Option<u64>,
    pub muted: bool,
    pub banned: bool,
}

/// How well `name` matches the search `q`, best first: 0 a prefix of the name or of one of its
/// words, 1 a substring, 2 the letters in order (fuzzy). Case is ignored; empty matches all.
pub fn rank(name: &str, q: &str) -> Option<u8> {
    let (name, q) = (name.to_lowercase(), q.trim().to_lowercase());
    if q.is_empty() || name.starts_with(&q) || name.split(' ').any(|word| word.starts_with(&q)) {
        return Some(0);
    }
    if name.contains(&q) {
        return Some(1);
    }
    let mut letters = name.chars();
    q.chars()
        .filter(|c| !c.is_whitespace())
        .all(|c| letters.any(|n| n == c))
        .then_some(2)
}

fn unix(time: std::io::Result<std::time::SystemTime>) -> Option<u64> {
    time.ok()?.duration_since(UNIX_EPOCH).ok().map(|d| d.as_secs())
}

/// When the save was first and last written.
fn save_times(saves: &Path, guid: u64) -> (Option<u64>, Option<u64>) {
    match std::fs::metadata(saves.join(format!("{guid}.json"))) {
        Ok(meta) => (unix(meta.created()), unix(meta.modified())),
        Err(_) => (None, None),
    }
}

/// Every character in the store matching `query`, one page of them, and the total.
pub fn search(
    store: &AccountStore,
    saves: &Path,
    online: &BTreeSet<u64>,
    query: &Query,
    now: u64,
) -> (usize, Vec<PlayerRow>) {
    let number = query.q.trim().parse::<u64>().ok();
    let mut found: Vec<(u8, PlayerRow)> = store
        .characters()
        .filter_map(|record| {
            let name = record.identity.display_name();
            let account = record.owner_account_id;
            let rank = if number.is_some_and(|n| n == record.guid || n == account) {
                Some(0)
            } else {
                rank(&name, &query.q)
            }?;
            let role = store.account_role(account).unwrap_or(AccountRole::Player);
            let sanctions = store.sanctions(account);
            let is_online = online.contains(&record.guid);
            let (created, saved) = save_times(saves, record.guid);
            let row = PlayerRow {
                character_id: record.guid,
                name,
                account,
                role: roles::name(role),
                online: is_online,
                last_seen: if is_online { Some(now) } else { saved },
                created,
                muted: sanctions.muted(now),
                banned: sanctions.banned(now),
            };
            let keep = query.online.is_none_or(|wanted| wanted == row.online)
                && query.role.is_none_or(|wanted| wanted == role)
                && query.banned.is_none_or(|wanted| wanted == row.banned)
                && query.muted.is_none_or(|wanted| wanted == row.muted);
            keep.then_some((rank, row))
        })
        .collect();
    let newest = |time: Option<u64>| std::cmp::Reverse(time.unwrap_or(0));
    match query.sort {
        Sort::Name => found.sort_by_cached_key(|(rank, row)| {
            (*rank, row.name.to_lowercase(), row.character_id)
        }),
        Sort::LastSeen => found.sort_by_key(|(_, row)| (newest(row.last_seen), row.character_id)),
        Sort::Created => found.sort_by_key(|(_, row)| (newest(row.created), row.character_id)),
    }
    let total = found.len();
    let skip = query.page.max(1).saturating_sub(1).saturating_mul(PAGE);
    let rows = found.into_iter().skip(skip).take(PAGE).map(|(_, row)| row).collect();
    (total, rows)
}

/// A character save as plain JSON, bounded like `game_store`; `None` when absent or unreadable.
fn read_save(saves: &Path, guid: u64) -> Option<Value> {
    let file = std::fs::File::open(saves.join(format!("{guid}.json"))).ok()?;
    let mut bytes = Vec::new();
    file.take(crate::game_store::MAX_SNAPSHOT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() > crate::game_store::MAX_SNAPSHOT_BYTES {
        return None;
    }
    serde_json::from_slice(&bytes).ok()
}

/// The jobs a save holds: id, XP, level on the job's curve (`job_xp_curve.json` in `content`,
/// else the local 100-XP curve) and whether it is the active one.
fn jobs(gameplay: &Value, content: Option<&Path>) -> Vec<Value> {
    let profiles = &gameplay["profiles"];
    let curves = content
        .and_then(|dir| crate::profiles::load_curves(dir).ok())
        .unwrap_or_default();
    let active = profiles["active"].as_i64();
    let experience = profiles["experience"].as_object();
    let mut ids: BTreeSet<i64> = profiles["unlocked"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_i64)
        .collect();
    ids.extend(experience.into_iter().flatten().filter_map(|(id, _)| id.parse::<i64>().ok()));
    ids.into_iter()
        .map(|id| {
            let xp = experience
                .and_then(|map| map.get(&id.to_string()))
                .and_then(Value::as_u64)
                .and_then(|xp| u32::try_from(xp).ok())
                .unwrap_or(0);
            let level = i32::try_from(id)
                .ok()
                .and_then(|id| curves.get(&id).cloned())
                .unwrap_or_else(crate::profiles::RankCurve::flat)
                .level(xp);
            json!({"job": id, "xp": xp, "level": level, "active": active == Some(id)})
        })
        .collect()
}

/// Quest ids by state, from the opening quests and the regional quest engine.
fn quests(gameplay: &Value) -> Value {
    let (mut active, mut done) = (BTreeSet::new(), BTreeSet::new());
    if let Some(quests) = gameplay["quests"]["quests"].as_object() {
        for (id, progress) in quests {
            let Ok(id) = id.parse::<u64>() else { continue };
            if progress["completed"].as_bool() == Some(true) {
                done.insert(id);
            } else if progress["accepted"].as_bool() == Some(true) {
                active.insert(id);
            }
        }
    }
    if let Some(quests) = gameplay["npc_quests"]["quests"].as_object() {
        for (id, progress) in quests {
            let Ok(id) = id.parse::<u64>() else { continue };
            match progress["status"].as_str() {
                Some("completed") => done.insert(id),
                Some("active") => active.insert(id),
                _ => false,
            };
        }
    }
    json!({"active": active, "done": done})
}

/// Rides the inventory holds, by `mounts.json` (`item_definition_id_candidates`).
fn mounts(gameplay: &Value, content: Option<&Path>) -> Vec<Value> {
    let owned: BTreeSet<i64> = gameplay["inventory"]["items"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| item["definition_id"].as_i64())
        .collect();
    let Some(table) = content
        .and_then(|dir| std::fs::read(dir.join("mounts.json")).ok())
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
    else {
        return Vec::new();
    };
    table["mounts"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|mount| {
            mount["item_definition_id_candidates"]
                .as_array()
                .is_some_and(|ids| ids.iter().filter_map(Value::as_i64).any(|id| owned.contains(&id)))
        })
        .map(|mount| json!({"id": mount["id"], "name": mount["name"]}))
        .collect()
}

/// Audit rows about one player, newest first: moderation by account, notes by character.
pub fn history(audit_dir: &Path, account: u64, guid: u64) -> (Vec<audit::Row>, Vec<audit::Row>) {
    let (mut moderation, mut notes) = (Vec::new(), Vec::new());
    // A log that cannot be read shows no history; the profile itself still answers.
    let _ = audit::scan(audit_dir, |row| {
        if row.action == "note" && row.target.character_id == Some(guid) {
            notes.push(row);
        } else if MODERATION.contains(&row.action.as_str()) && row.target.account_id == Some(account)
        {
            moderation.push(row);
        }
        moderation.len() < HISTORY || notes.len() < HISTORY
    });
    moderation.truncate(HISTORY);
    notes.truncate(HISTORY);
    (moderation, notes)
}

/// Where a profile reads from.
pub struct Sources<'a> {
    pub store: &'a AccountStore,
    /// The character saves folder (`characters/`).
    pub saves: &'a Path,
    pub content: Option<&'a Path>,
    pub audit: &'a Path,
}

/// `GET /admin/api/players/{id}`: everything the panel shows about one character, or `None` when
/// the store has no such character. `online` is the live row when the character is playing; its
/// zone and position then replace the saved ones.
pub fn profile(sources: &Sources, guid: u64, online: Option<&OnlineRow>, now: u64) -> Option<Value> {
    let store = sources.store;
    let record = store.character(guid)?;
    let account = record.owner_account_id;
    let role = store.account_role(account).unwrap_or(AccountRole::Player);
    let sanctions = store.sanctions(account);
    let save = read_save(sources.saves, guid).unwrap_or(Value::Null);
    let gameplay = &save["gameplay"];
    let (created, saved) = save_times(sources.saves, guid);
    let (zone, position) = match online {
        Some(row) => (json!(row.zone), json!(row.position)),
        None => (
            save["character"]["zone_id"].clone(),
            save["character"]["position"].clone(),
        ),
    };
    let collections = &gameplay["progression_extras"]["collections"];
    let count = |value: &Value| value.as_array().map_or(0, Vec::len);
    let (moderation, notes) = history(sources.audit, account, guid);
    let characters: BTreeMap<u64, String> = store
        .account(account)
        .into_iter()
        .flat_map(|a| &a.character_guids)
        .filter_map(|id| Some((*id, store.character(*id)?.identity.display_name())))
        .collect();
    Some(json!({
        "identity": {
            "character_id": guid,
            "name": record.identity.display_name(),
            "first_name": record.identity.first_name,
            "last_name": record.identity.last_name,
            "account": account,
            "account_characters": characters,
            "role": roles::name(role),
            "online": online.is_some(),
            "created": created,
            "last_seen": if online.is_some() { Some(now) } else { saved },
            "play_time_s": null,
        },
        "location": {
            "online": online.is_some(), "zone": zone,
            "zone_name": online.and_then(|row| row.zone_name.as_deref()), "position": position,
        },
        "coins": gameplay["inventory"]["coins"],
        "jobs": jobs(gameplay, sources.content),
        "quests": quests(gameplay),
        "mounts": mounts(gameplay, sources.content),
        "pets": save["pets"]["pets"].as_array().cloned().unwrap_or_default(),
        "collections_n": count(&collections["collected"]),
        "collections_done": count(&collections["completed"]),
        "sanctions": {
            "muted": sanctions.muted(now), "muted_until": sanctions.muted_until,
            "banned": sanctions.banned(now), "banned_until": sanctions.banned_until,
        },
        "moderation": moderation,
        "notes": notes,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_ranks_prefix_then_substring_then_fuzzy() {
        assert_eq!(rank("Pip Player", ""), Some(0));
        assert_eq!(rank("Pip Player", "pla"), Some(0), "a later word's prefix");
        assert_eq!(rank("Pip Player", "ip p"), Some(1));
        assert_eq!(rank("Pip Player", "ppr"), Some(2));
        assert_eq!(rank("Pip Player", "zz"), None);
    }

    #[test]
    fn quests_split_by_state_from_both_engines() {
        let gameplay = json!({
            "quests": {"quests": {"1": {"accepted": true, "completed": false}, "2": {"accepted": true, "completed": true}}},
            "npc_quests": {"quests": {"30": {"status": "active"}, "31": {"status": "completed"}, "32": {"status": "offered"}}},
        });
        assert_eq!(quests(&gameplay), json!({"active": [1, 30], "done": [2, 31]}));
        let jobs = jobs(&json!({"profiles": {"active": 3, "unlocked": [3, 5], "experience": {"3": 250}}}), None);
        assert_eq!(jobs[0], json!({"job": 3, "xp": 250, "level": 3, "active": true}));
        assert_eq!(jobs[1]["level"], 1);
    }
}
