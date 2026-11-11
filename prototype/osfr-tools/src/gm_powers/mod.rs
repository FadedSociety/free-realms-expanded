//! G5: staff powers for the original 2014 client: spectate, invisible, freeze, follow, bring,
//! goto, send, waypoints and the fly exemption. This module holds the rules, the chat words and
//! the staff state in plain types; the gateway host applies them to live sessions
//! (`bin/osfr_gateway/gm_powers_routing.rs`, `run`), which is also what an admin HTTP route calls
//! after one of the facade functions below has checked the caller's role.
//!
//! Spectate. The client has no packet that puts its camera on another actor:
//! `UpdateCameraFromConfigOrPlayer` (`0091dc20`) places the camera from the local player entity
//! (+0x3e0) or from the local `Solo` `Camera.*` config; opcode 126 (`PlayerUpdatePacketCameraUpdate`,
//! a member built at `009417c0`) is the client's own camera report; the camera configs it reads
//! from the server are minigame ones (ship combat `00a62140`, Battle Mages `00b70c20`); and none
//! of the 35 family's cases (dispatcher `0092f460`) is a camera or attach packet (its rail and
//! seek controllers move proxied actors, `009668a0`, `00966970`). So the server hides the GM
//! from every other viewer (`PlayerRegistry::set_hidden`) and keeps the GM's character locked a
//! little behind the target with the 38/12 relocation, every tick; the client's camera follows
//! its own character. The GM's reports are ignored meanwhile (`movement::StaffMotion::Driven`).
//!
//! Freeze holds the pose on the server (`StaffMotion::Held`) and also sends the client's own
//! 35/32 `PlayerUpdatePacketFreeze`: case 0x20 of `0092f460` reads a bool (`008f8110`) and sets
//! bit 0x10 at actor +0x146 (`008dcaa0`), which `UpdateMotionAndActorState` (`0096e490`) checks
//! before it runs the actor's motion.
//!
//! The original client cannot fly (no fly, noclip or free-camera handler). `fly` only sets the
//! movement exemption (`StaffMotion::Flight`), for the Rust client's phase 2.

pub mod waypoints;

use crate::account_store::{AccountRole, can_run_realm_command};
use crate::encounter::ZoneTransition;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// (R) How far behind its target a spectating GM stands: the GM's camera looks over the
/// target's shoulder, and the GM stays within a metre of it.
pub const SPECTATE_BEHIND_M: f32 = 0.5;
/// (R) Follow distance when none is typed, and the largest one allowed.
pub const FOLLOW_DEFAULT_M: f32 = 3.0;
pub const FOLLOW_MAX_M: f32 = 30.0;
/// (R) Audit rows kept in memory for the admin panel; the security log keeps every one.
pub const AUDIT_ROWS: usize = 512;
/// The refusal for a role that may not use a power.
pub const NOT_ALLOWED: &str = "your account role does not allow that power";

/// Whom a power acts on, as typed: a character GUID or a displayed name (ignoring case).
#[derive(Clone, Debug, PartialEq)]
pub enum Who {
    Guid(u64),
    Name(String),
}

/// One staff power, already parsed. `run` in the gateway host carries it out.
#[derive(Clone, Debug, PartialEq)]
pub enum Power {
    Spectate(Who),
    Unspectate,
    /// `true` hides the caller, `false` shows it again.
    Invisible(bool),
    /// `true` freezes the target, `false` releases it.
    Freeze(Who, bool),
    /// Stay this many metres behind the target.
    Follow(Who, f32),
    Unfollow,
    /// Move the target to the caller (open world, as the GM wheel's Summon).
    Bring(Who),
    /// Move the caller to the target (open world, as the GM wheel's Go to).
    Goto(Who),
    /// Move the target to an open-world position. In chat, `realm send` is `realm move`.
    Send(Who, [f32; 3]),
    Fly(bool),
    Waypoint(WaypointOp),
}

#[derive(Clone, Debug, PartialEq)]
pub enum WaypointOp {
    Save(String),
    Go(String),
    List,
    Delete(String),
}

impl Power {
    /// The `can_run_realm_command` key, which is also the audit row's action.
    pub fn key(&self) -> &'static str {
        match self {
            Self::Spectate(_) => "spectate",
            Self::Unspectate => "unspectate",
            Self::Invisible(true) => "invisible",
            Self::Invisible(false) => "visible",
            Self::Freeze(_, true) => "freeze",
            Self::Freeze(_, false) => "unfreeze",
            Self::Follow(..) => "follow",
            Self::Unfollow => "unfollow",
            Self::Bring(_) => "bring",
            Self::Goto(_) => "goto",
            Self::Send(..) => "send",
            Self::Fly(_) => "fly",
            Self::Waypoint(_) => "waypoint",
        }
    }
}

/// The facade's one check: the role table (`can_run_realm_command`) for the power's key.
pub fn authorize(role: AccountRole, power: Power) -> Result<Power, &'static str> {
    if can_run_realm_command(role, power.key()) {
        Ok(power)
    } else {
        Err(NOT_ALLOWED)
    }
}

pub fn spectate(role: AccountRole, target: Who) -> Result<Power, &'static str> {
    authorize(role, Power::Spectate(target))
}

pub fn unspectate(role: AccountRole) -> Result<Power, &'static str> {
    authorize(role, Power::Unspectate)
}

pub fn invisible(role: AccountRole, on: bool) -> Result<Power, &'static str> {
    authorize(role, Power::Invisible(on))
}

pub fn freeze(role: AccountRole, target: Who, on: bool) -> Result<Power, &'static str> {
    authorize(role, Power::Freeze(target, on))
}

pub fn follow(role: AccountRole, target: Who, metres: f32) -> Result<Power, &'static str> {
    let metres = follow_metres(metres).ok_or(FOLLOW_USAGE)?;
    authorize(role, Power::Follow(target, metres))
}

pub fn bring(role: AccountRole, target: Who) -> Result<Power, &'static str> {
    authorize(role, Power::Bring(target))
}

pub fn goto(role: AccountRole, target: Who) -> Result<Power, &'static str> {
    authorize(role, Power::Goto(target))
}

pub fn send(role: AccountRole, target: Who, position: [f32; 3]) -> Result<Power, &'static str> {
    let bound = crate::character_store::MAX_COMPONENT_MAGNITUDE;
    if !position.iter().all(|v| v.is_finite() && v.abs() <= bound) {
        return Err("that position is out of bounds");
    }
    authorize(role, Power::Send(target, position))
}

pub fn fly(role: AccountRole, on: bool) -> Result<Power, &'static str> {
    authorize(role, Power::Fly(on))
}

pub fn waypoint(role: AccountRole, op: WaypointOp) -> Result<Power, &'static str> {
    if let WaypointOp::Save(label) | WaypointOp::Go(label) | WaypointOp::Delete(label) = &op {
        waypoints::check_label(label)?;
    }
    authorize(role, Power::Waypoint(op))
}

const FOLLOW_USAGE: &str = "Usage: realm follow <name> [metres, 1 to 30].";
const WAYPOINT_USAGE: &str = "Usage: realm waypoint save|go|del <label>, or realm waypoint list.";

/// `realm <words>` typed in chat: `Some((key, power))` for a G5 word, with its role-table key
/// and the power (or a usage line when its arguments are wrong); `None` for every other
/// command. `goto` is a G5 word only with a name: coordinates, `mark` and `list` stay with
/// gameplay's developer teleport, and the host hands back a name that is no online player, so
/// `realm goto <NPC>` still works. `realm send` is gameplay's `realm move`.
pub fn parse(text: &str) -> Option<(&'static str, Result<Power, &'static str>)> {
    let text = text.trim();
    let lower = text.to_ascii_lowercase();
    // ASCII lowercasing keeps byte offsets, so `raw` keeps the typed case of a name.
    lower.strip_prefix("realm ")?;
    let raw = text.get("realm ".len()..)?.trim_start();
    let (word, args) = raw.split_once(char::is_whitespace).unwrap_or((raw, ""));
    let args = args.trim();
    let named = |power: fn(Who) -> Power, usage| who(args).map(power).ok_or(usage);
    // The role-table key is the word itself (`wp` is short for `waypoint`).
    let key = match word.to_ascii_lowercase().as_str() {
        "wp" | "waypoint" => "waypoint",
        word => *KEYS.iter().find(|key| **key == word)?,
    };
    let parsed = match key {
        "spectate" => named(Power::Spectate, "Usage: realm spectate <name>."),
        "unspectate" => Ok(Power::Unspectate),
        "invisible" => Ok(Power::Invisible(true)),
        "visible" => Ok(Power::Invisible(false)),
        "freeze" => named(|w| Power::Freeze(w, true), "Usage: realm freeze <name>."),
        "unfreeze" => named(|w| Power::Freeze(w, false), "Usage: realm unfreeze <name>."),
        "follow" => parse_follow(args),
        "unfollow" => Ok(Power::Unfollow),
        "bring" => named(Power::Bring, "Usage: realm bring <name>."),
        "goto" => {
            let first = args.split_whitespace().next().unwrap_or_default();
            let coordinates = args
                .split(|c: char| c.is_whitespace() || c == ',')
                .filter(|w| !w.is_empty())
                .all(|w| w.parse::<f32>().is_ok());
            if coordinates || ["mark", "list"].contains(&first.to_ascii_lowercase().as_str()) {
                return None;
            }
            named(Power::Goto, "Usage: realm goto <name>.")
        }
        "fly" => match args.to_ascii_lowercase().as_str() {
            "" | "on" => Ok(Power::Fly(true)),
            "off" => Ok(Power::Fly(false)),
            _ => Err("Usage: realm fly [on|off]."),
        },
        _ => parse_waypoint(args),
    };
    Some((key, parsed))
}

/// Every chat word `parse` answers, each its own role-table key.
const KEYS: [&str; 12] = [
    "spectate",
    "unspectate",
    "invisible",
    "visible",
    "freeze",
    "unfreeze",
    "follow",
    "unfollow",
    "bring",
    "goto",
    "fly",
    "waypoint",
];

fn who(args: &str) -> Option<Who> {
    match args.parse::<u64>() {
        _ if args.is_empty() => None,
        Ok(guid) => Some(Who::Guid(guid)),
        Err(_) => Some(Who::Name(args.to_owned())),
    }
}

/// `<name> [metres]`: names hold no digits, so a trailing number is the distance.
fn parse_follow(args: &str) -> Result<Power, &'static str> {
    let (name, metres) = match args.rsplit_once(char::is_whitespace) {
        Some((name, last)) if last.parse::<f32>().is_ok() => (name.trim(), last.parse().ok()),
        _ => (args, None),
    };
    let metres = follow_metres(metres.unwrap_or(FOLLOW_DEFAULT_M)).ok_or(FOLLOW_USAGE)?;
    Ok(Power::Follow(who(name).ok_or(FOLLOW_USAGE)?, metres))
}

fn follow_metres(metres: f32) -> Option<f32> {
    (metres.is_finite() && (1.0..=FOLLOW_MAX_M).contains(&metres)).then_some(metres)
}

fn parse_waypoint(args: &str) -> Result<Power, &'static str> {
    let (op, label) = args.split_once(char::is_whitespace).unwrap_or((args, ""));
    let label = label.trim().to_owned();
    let op = match op.to_ascii_lowercase().as_str() {
        "list" if label.is_empty() => WaypointOp::List,
        "save" | "set" => WaypointOp::Save(label),
        "go" => WaypointOp::Go(label),
        "del" | "delete" => WaypointOp::Delete(label),
        _ => return Err(WAYPOINT_USAGE),
    };
    if let WaypointOp::Save(label) | WaypointOp::Go(label) | WaypointOp::Delete(label) = &op {
        waypoints::check_label(label)?;
    }
    Ok(Power::Waypoint(op))
}

/// Where a watcher stands: `metres` behind `position` along the horizontal facing `rotation`
/// (the direction vector a pose carries), at the target's height. A facing with no length
/// leaves the watcher on the spot.
pub fn behind(position: [f32; 3], rotation: [f32; 3], metres: f32) -> [f32; 3] {
    let (x, z) = (rotation[0], rotation[2]);
    let length = x.hypot(z);
    if !length.is_finite() || length <= f32::EPSILON {
        return position;
    }
    [
        position[0] - x / length * metres,
        position[1],
        position[2] - z / length * metres,
    ]
}

/// Where a spectating GM goes back to: its pose, World instance and scene when it began, and
/// whether it was already invisible (then it stays so).
#[derive(Clone, Debug, PartialEq)]
pub struct Return {
    pub instance_id: i32,
    pub scene: ZoneTransition,
    pub was_hidden: bool,
}

/// What drives a staff character's pose, keyed by that character in [`GmPowers::watching`].
#[derive(Clone, Debug, PartialEq)]
pub enum Watch {
    Spectate { target: u64, back: Return },
    Follow { target: u64, metres: f32 },
}

impl Watch {
    pub fn target(&self) -> u64 {
        match self {
            Self::Spectate { target, .. } | Self::Follow { target, .. } => *target,
        }
    }

    pub fn metres(&self) -> f32 {
        match self {
            Self::Spectate { .. } => SPECTATE_BEHIND_M,
            Self::Follow { metres, .. } => *metres,
        }
    }
}

/// Unix milliseconds, for an audit row.
pub fn unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| {
            u64::try_from(since.as_millis()).unwrap_or(u64::MAX)
        })
}

/// One power use. Numbers and the closed action name only: no names or typed text.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AuditRow {
    pub timestamp_ms: u64,
    pub account_id: u64,
    pub character_guid: u64,
    pub action: &'static str,
    pub target_guid: Option<u64>,
    pub applied: bool,
}

/// The live staff state. Session-only: a restart ends every spectate, follow, freeze and
/// flight, and invisibility ends with its session (the registry flag is cleared at logout).
#[derive(Clone, Debug, Default)]
pub struct GmPowers {
    /// By the watching staff character.
    pub watching: BTreeMap<u64, Watch>,
    /// Frozen characters; their hold lives in their own movement token.
    pub frozen: BTreeSet<u64>,
    /// Jailed characters (the panel's moderation `jail`): held like `frozen`, but kept across
    /// their logouts until a release or a restart, so a relog does not walk out.
    pub jailed: BTreeSet<u64>,
    /// Staff characters with the flight exemption.
    pub flying: BTreeSet<u64>,
    audit: VecDeque<AuditRow>,
}

impl GmPowers {
    /// Keep `row`, dropping the oldest past [`AUDIT_ROWS`].
    pub fn record(&mut self, row: AuditRow) {
        if self.audit.len() == AUDIT_ROWS {
            self.audit.pop_front();
        }
        self.audit.push_back(row);
    }

    /// Newest last.
    pub fn audit(&self) -> impl Iterator<Item = &AuditRow> {
        self.audit.iter()
    }

    /// Whether `guid` is watched by anyone (spectated or followed).
    pub fn watchers_of(&self, guid: u64) -> impl Iterator<Item = u64> + '_ {
        self.watching
            .iter()
            .filter(move |(_, watch)| watch.target() == guid)
            .map(|(staff, _)| *staff)
    }
}

#[cfg(test)]
mod tests;
