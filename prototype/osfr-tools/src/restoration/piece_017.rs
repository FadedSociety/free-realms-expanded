//! Work item 017: minigame disconnect recovery conservation audit.
//!
//! Recovered original facts: the shipped mini-game movies expose
//! `SoeGameFramework.OnClientDisconnect() -> GameClientDisconnect()`, and that
//! hook is an empty default in all five decompiled movies, so the movie has no
//! client-side resume path. The supplied community server (`Gateway`,
//! `LibSOE`) has no mini-game session state at all - only the packet class
//! enums - so a dropped connection cannot resume a Cooking46, Smelting77 or
//! Forging139 run.
//!
//! The local reconstruction therefore cancels any persisted run on the next
//! world load (`Gameplay::initialize_world`), without consuming ingredients,
//! awarding products or reusing attempt ids. This module audits two persisted
//! gameplay snapshots across that boundary. It is a checker, not a second copy
//! of the crafting services: production recovery and its packet regressions
//! stay in `cooking.rs`, `smelting.rs`, `forging.rs` and `gameplay.rs`.

use serde_json::Value;
use std::collections::BTreeMap;

/// Crafting movies that persist attempt state across a disconnect.
pub const MOVIES: [&str; 3] = ["cooking", "smelting", "forging"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClearedRun {
    pub service: &'static str,
    pub instance_id: i64,
    pub phase: String,
}

/// Runs that are no longer active in `after` (cancelled or replaced).
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Conservation {
    pub cleared: Vec<ClearedRun>,
}

/// Accepts either a full `Snapshot` (`{"character":..,"gameplay":..}`) or the
/// bare `GameplayState` object.
fn gameplay(value: &Value) -> &Value {
    value
        .get("gameplay")
        .filter(|game| game.is_object())
        .unwrap_or(value)
}

fn inventory_totals(gameplay: &Value) -> Result<BTreeMap<(i64, i64), u64>, String> {
    let items = gameplay
        .pointer("/inventory/items")
        .and_then(Value::as_array)
        .ok_or("snapshot has no inventory items")?;
    let mut totals = BTreeMap::new();
    for item in items {
        let definition = item
            .get("definition_id")
            .and_then(Value::as_i64)
            .ok_or("inventory item has no definition_id")?;
        let tint = item.get("tint").and_then(Value::as_i64).unwrap_or_default();
        let count = item
            .get("count")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("inventory item {definition} has no count"))?;
        *totals.entry((definition, tint)).or_default() += count;
    }
    Ok(totals)
}

fn active(movie: &Value) -> Option<&Value> {
    movie.get("active").filter(|run| run.is_object())
}

fn instance_id(run: &Value) -> Result<i64, String> {
    run.get("instance_id")
        .and_then(Value::as_i64)
        .ok_or_else(|| "active run has no instance_id".to_string())
}

fn phase(run: &Value) -> String {
    run.get("phase")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string()
}

fn counter(movie: &Value, service: &str) -> Result<i64, String> {
    movie
        .get("next_instance_id")
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("{service} state has no next_instance_id"))
}

fn reject_active_runs(after: &Value) -> Result<(), String> {
    let after = gameplay(after);
    for service in MOVIES {
        let Some(movie) = after.get(service).filter(|movie| movie.is_object()) else {
            continue;
        };
        if let Some(run) = active(movie) {
            return Err(format!(
                "{service} run {} ({}) survived the recovery boundary",
                instance_id(run)?,
                phase(run)
            ));
        }
    }
    Ok(())
}

/// Materials, wallet, equipment, quests and attempt bookkeeping must be
/// identical across the boundary; a cleared run may only be replaced by a
/// freshly allocated attempt.
pub fn audit_conservation(before: &Value, after: &Value) -> Result<Conservation, String> {
    let (before_game, after_game) = (gameplay(before), gameplay(after));
    let (before_items, after_items) = (
        inventory_totals(before_game)?,
        inventory_totals(after_game)?,
    );
    if before_items != after_items {
        let mut keys: Vec<_> = before_items.keys().chain(after_items.keys()).collect();
        keys.sort_unstable();
        keys.dedup();
        for key in keys {
            let before_count = before_items.get(key).copied().unwrap_or_default();
            let after_count = after_items.get(key).copied().unwrap_or_default();
            if before_count != after_count {
                return Err(format!(
                    "definition {} tint {} count {before_count} -> {after_count}",
                    key.0, key.1
                ));
            }
        }
    }
    let coins = |game: &Value| game.pointer("/inventory/coins").cloned();
    if coins(before_game) != coins(after_game) {
        return Err(format!(
            "inventory coins changed {:?} -> {:?}",
            coins(before_game),
            coins(after_game)
        ));
    }
    if before_game.pointer("/inventory/equipment") != after_game.pointer("/inventory/equipment") {
        return Err("inventory equipment changed across the boundary".into());
    }
    if before_game.get("quests") != after_game.get("quests") {
        return Err("quest state changed across the boundary".into());
    }

    let mut conservation = Conservation::default();
    for service in MOVIES {
        let Some(before_movie) = before_game.get(service).filter(|movie| movie.is_object()) else {
            continue;
        };
        let Some(after_movie) = after_game.get(service).filter(|movie| movie.is_object()) else {
            return Err(format!("{service} state is missing after the boundary"));
        };
        if counter(before_movie, service)? > counter(after_movie, service)? {
            return Err(format!(
                "{service} next_instance_id regressed {} -> {}",
                counter(before_movie, service)?,
                counter(after_movie, service)?
            ));
        }
        let completed = |movie: &Value| movie.get("completed").and_then(Value::as_u64);
        if completed(before_movie) != completed(after_movie) {
            return Err(format!(
                "{service} completed changed {:?} -> {:?}",
                completed(before_movie),
                completed(after_movie)
            ));
        }
        let Some(before_run) = active(before_movie) else {
            continue;
        };
        let id = instance_id(before_run)?;
        if matches!(phase(before_run).as_str(), "Won" | "Lost") {
            return Err(format!(
                "{service} run {id} already reached {}; that boundary belongs to the \
                 crafting transaction audit, not to disconnect recovery",
                phase(before_run)
            ));
        }
        if let Some(after_run) = active(after_movie) {
            let replacement = instance_id(after_run)?;
            if replacement == id {
                return Err(format!("{service} run {id} reused its instance id"));
            }
            if replacement < counter(before_movie, service)? {
                return Err(format!(
                    "{service} run {replacement} was not allocated from the attempt counter {}",
                    counter(before_movie, service)?
                ));
            }
        }
        conservation.cleared.push(ClearedRun {
            service,
            instance_id: id,
            phase: phase(before_run),
        });
    }
    Ok(conservation)
}

/// The reload boundary is stricter than a same-session comparison: the
/// persisted run must be gone, not resumed.
pub fn audit_recovery(before: &Value, after: &Value) -> Result<Conservation, String> {
    reject_active_runs(after)?;
    audit_conservation(before, after)
}
