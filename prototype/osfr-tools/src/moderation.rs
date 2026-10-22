//! L042 staff moderation: mute, kick and ban, stored per account in the account store as unix
//! seconds (ids only, never names). The gateway stages a role-gated command; the host runs it
//! after the moderator's transaction commits, audits it and ends a kicked or banned session.
//! The login server and the gateway's character binding refuse a banned account.
use crate::account_store::{
    AccountStore, AccountStoreLock, STAFF_LOCK_WAIT, can_run_realm_command,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// The longest timed sanction, in minutes (R): `max_minutes` in `data/content/moderation.json`
/// (one year shipped). A file that does not parse leaves 0, which refuses every timed sanction
/// and never panics; `bundled_bounds_are_sane` keeps the shipped file usable.
pub static MAX_MINUTES: LazyLock<u32> = LazyLock::new(|| {
    serde_json::from_str::<serde_json::Value>(include_str!("../data/content/moderation.json"))
        .ok()
        .and_then(|bounds| bounds["max_minutes"].as_u64())
        .and_then(|max| u32::try_from(max).ok())
        .unwrap_or(0)
});
/// Stored expiry of a sanction given without minutes: it lasts until lifted (R).
pub const UNTIL_LIFTED: u64 = u64::MAX;

const USAGE: &str =
    "Usage: realm <mute|ban> <First Last> [minutes], or realm <unmute|unban|kick> <First Last>";
const NOT_ALLOWED: &str = "Cannot do that: your account role does not allow this command.";
const BUSY: &str = "Cannot do that: account store is busy or unavailable.";

/// Moderation state of one account. Absent fields mean no sanction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sanctions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub muted_until: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banned_until: Option<u64>,
}

impl Sanctions {
    /// What a session assumes when it cannot read the account store: chat refused (R).
    pub const FAIL_CLOSED: Self = Self {
        muted_until: Some(UNTIL_LIFTED),
        banned_until: None,
    };

    pub fn muted(&self, now: u64) -> bool {
        self.muted_until.is_some_and(|until| now < until)
    }

    pub fn banned(&self, now: u64) -> bool {
        self.banned_until.is_some_and(|until| now < until)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Mute,
    Unmute,
    Kick,
    Ban,
    Unban,
}

impl Action {
    /// The `realm` command word, which is also the role-table key.
    pub fn parse(word: &str) -> Option<Self> {
        Some(match word {
            "mute" => Self::Mute,
            "unmute" => Self::Unmute,
            "kick" => Self::Kick,
            "ban" => Self::Ban,
            "unban" => Self::Unban,
            _ => return None,
        })
    }

    fn word(self) -> &'static str {
        match self {
            Self::Mute => "mute",
            Self::Unmute => "unmute",
            Self::Kick => "kick",
            Self::Ban => "ban",
            Self::Unban => "unban",
        }
    }

    /// The moderator's confirmation line.
    pub fn done(self) -> &'static str {
        match self {
            Self::Mute => "Player muted.",
            Self::Unmute => "Player unmuted.",
            Self::Kick => "Player kicked.",
            Self::Ban => "Player banned.",
            Self::Unban => "Player unbanned.",
        }
    }
}

/// `realm <action> <display name> [minutes]`, already stripped of `realm `.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    pub action: Action,
    /// Words of the target's display name as typed; the store folds them with `name_key`, as it
    /// folds names for uniqueness (lowercasing first would turn 'İ' into 'i' and a combining dot).
    name: Vec<String>,
    pub minutes: Option<u32>,
}

impl Command {
    /// Names hold no digits (`identity::valid_name`), so a trailing number is the minutes.
    pub fn parse(command: &str) -> Result<Self, &'static str> {
        let mut words: Vec<String> = command.split_whitespace().map(str::to_owned).collect();
        let action = words
            .first()
            .and_then(|word| Action::parse(&word.to_lowercase()))
            .ok_or(USAGE)?;
        words.remove(0);
        let minutes = match words.last() {
            Some(last) if last.bytes().all(|byte| byte.is_ascii_digit()) => {
                let minutes = last
                    .parse()
                    .ok()
                    .filter(|minutes| (1..=*MAX_MINUTES).contains(minutes))
                    .ok_or(
                        "Cannot do that: minutes must be at least 1 and within the server limit.",
                    )?;
                words.pop();
                Some(minutes)
            }
            _ => None,
        };
        if words.is_empty() || (minutes.is_some() && !matches!(action, Action::Mute | Action::Ban))
        {
            return Err(USAGE);
        }
        Ok(Self {
            action,
            name: words,
            minutes,
        })
    }
}

/// What a successful command did, for the audit row and the host's kick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Applied {
    pub action: Action,
    pub target_account_id: u64,
    pub minutes: Option<u32>,
}

/// Authorize `command` for the server-bound `actor` account and stage its change on `store`;
/// the caller saves. Every refusal returns before `store` changes. Hierarchy (R): the actor
/// must strictly outrank the target's account role, and never targets its own account.
fn apply(
    store: &mut AccountStore,
    actor: u64,
    command: &Command,
    now: u64,
) -> Result<Applied, &'static str> {
    let actor_role = store
        .account_role(actor)
        .filter(|role| can_run_realm_command(*role, command.action.word()))
        .ok_or(NOT_ALLOWED)?;
    let target = store
        .account_by_display_name(&command.name)
        .ok_or("Cannot do that: no single character has that name.")?;
    if target == actor {
        return Err("Cannot do that: you cannot moderate your own account.");
    }
    if !store
        .account_role(target)
        .is_some_and(|role| actor_role.outranks(role))
    {
        return Err("Cannot do that: that player's role is equal to or above yours.");
    }
    let mut sanctions = store.sanctions(target);
    // Saturating: a u32 of minutes in seconds fits a u64, and the sum clamps instead of panicking.
    let until = command.minutes.map_or(UNTIL_LIFTED, |minutes| {
        now.saturating_add(u64::from(minutes).saturating_mul(60))
    });
    // (R) A second sanction keeps the later expiry.
    let later = |old: Option<u64>| Some(old.map_or(until, |old| old.max(until)));
    match command.action {
        Action::Mute => sanctions.muted_until = later(sanctions.muted_until),
        Action::Ban => sanctions.banned_until = later(sanctions.banned_until),
        Action::Unmute if sanctions.muted(now) => sanctions.muted_until = None,
        Action::Unban if sanctions.banned(now) => sanctions.banned_until = None,
        Action::Unmute => return Err("Cannot do that: that player is not muted."),
        Action::Unban => return Err("Cannot do that: that player is not banned."),
        Action::Kick => {}
    }
    store.set_sanctions(target, sanctions);
    Ok(Applied {
        action: command.action,
        target_account_id: target,
        minutes: command.minutes,
    })
}

/// Run `command` against the store file. A kick only reads it; every other action holds the
/// writer lock across load, check and atomic save. A refusal leaves the file untouched.
pub fn run(path: &Path, actor: u64, command: &Command, now: u64) -> Result<Applied, &'static str> {
    let writes = command.action != Action::Kick;
    let _lock = writes
        .then(|| AccountStoreLock::acquire_within(path, STAFF_LOCK_WAIT))
        .transpose()
        .map_err(|_| BUSY)?;
    let mut store = AccountStore::load(path).map_err(|_| BUSY)?;
    let applied = apply(&mut store, actor, command, now)?;
    if writes {
        store
            .save(path)
            .map_err(|_| "Cannot do that: the moderation action was not saved.")?;
    }
    Ok(applied)
}

/// Wall-clock unix seconds: sanctions outlive restarts and expire by this clock.
pub fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

#[cfg(test)]
mod tests {
    /// Sanity bound, not a pinned value: the shipped file parses and allows a timed sanction.
    #[test]
    fn bundled_bounds_are_sane() {
        assert!(*super::MAX_MINUTES >= 1);
    }
}
