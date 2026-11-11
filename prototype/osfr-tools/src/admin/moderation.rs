//! The panel's moderation ops (`POST /admin/api/players/{id}/moderate`). Mute, unmute, kick, ban
//! and unban are the chat commands' own `crate::moderation::Command`, run by the same
//! `crate::moderation::run_with_change` and hierarchy (`check_target`); the gateway ends a kicked
//! or banned session the way `realm kick` does. Warn is new: a line to the player and a record.
//! Jail sends the player to [`jail_spot`] and holds it there with the staff freeze (G5's
//! `gm_powers_routing::jail`) until release; both use the same role table and hierarchy.
use super::{Staff, audit, roles};
use crate::account_store::AccountStore;
use crate::moderation::{Action, Applied, Command, Sanctions, check_target};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Sanction(Action),
    Warn,
    Jail,
    Release,
}

impl Op {
    pub fn parse(word: &str) -> Option<Self> {
        Some(match word {
            "warn" => Self::Warn,
            "jail" => Self::Jail,
            "release" => Self::Release,
            other => Self::Sanction(Action::parse(other)?),
        })
    }

    /// The op's name, which is also its permission key (`roles::ACTIONS`).
    pub fn word(self) -> &'static str {
        match self {
            Self::Sanction(action) => action.word(),
            Self::Warn => "warn",
            Self::Jail => "jail",
            Self::Release => "release",
        }
    }
}

/// The audit row for an applied sanction, from the panel or from chat.
pub fn sanction_row(
    staff: &Staff,
    command: &Command,
    change: (Applied, Sanctions, Sanctions),
    store: Option<&AccountStore>,
    reason: Option<String>,
    via: &str,
    now: u64,
) -> audit::Row {
    let (applied, before, after) = change;
    let (character_id, typed) = command.target_label();
    let name = character_id
        .and_then(|guid| store?.character(guid))
        .map(|record| record.identity.display_name())
        .or(typed);
    audit::Row {
        time: now,
        staff: staff.clone(),
        action: applied.action.word().to_owned(),
        target: audit::Target {
            character_id,
            account_id: Some(applied.target_account_id),
            name,
        },
        before: serde_json::to_value(before).unwrap_or_default(),
        after: serde_json::json!({"sanctions": after, "minutes": applied.minutes}),
        reason,
        via: via.to_owned(),
    }
}

/// Warn: the same role table and hierarchy as a sanction. Returns the target's account and the
/// line the player sees; the caller delivers it and records the row.
pub fn warn(
    store: &AccountStore,
    actor: u64,
    character: u64,
    reason: &str,
) -> Result<(u64, String), &'static str> {
    let target = may_act(store, actor, character, "warn")?;
    Ok((target, format!("Warning from the staff: {reason}")))
}

/// Whether `actor` may run `action` (a [`roles::ACTIONS`] name) on `character`: the role table,
/// then the sanction hierarchy (a strictly lower role, never one's own account). The target's
/// account on success.
pub fn may_act(
    store: &AccountStore,
    actor: u64,
    character: u64,
    action: &str,
) -> Result<u64, &'static str> {
    let actor_role = store
        .account_role(actor)
        .filter(|role| roles::allowed(*role, action))
        .ok_or("Cannot do that: your account role does not allow this command.")?;
    let target = store
        .character(character)
        .map(|record| record.owner_account_id)
        .ok_or("Cannot do that: no single character has that name.")?;
    check_target(store, actor_role, actor, target)?;
    Ok(target)
}

/// Where `jail` sends a player, in the open world (R): `jail.position` in
/// `data/content/moderation.json`; `None` when the file has no valid spot.
pub fn jail_spot() -> Option<[f32; 3]> {
    let document: serde_json::Value =
        serde_json::from_str(include_str!("../../data/content/moderation.json")).ok()?;
    let at = document["jail"]["position"].as_array()?;
    let bound = crate::character_store::MAX_COMPONENT_MAGNITUDE;
    let mut spot = [0.0f32; 3];
    for (slot, value) in spot.iter_mut().zip(at) {
        *slot = value.as_f64()? as f32;
    }
    (at.len() == 3 && spot.iter().all(|v| v.is_finite() && v.abs() <= bound)).then_some(spot)
}
