//! The behaviour language's runtime: one stance plus when / if / do rules per enemy (combat
//! design section 8). Pure: no packets and no engine types, so it is tested without a client.
//! Piece P3 (`combat-brain`) owns the bodies, the loader and everything inside `Program` and
//! `Memory`; further files go under `brain/`.
//!
//! Frozen by the combat seam: `step`'s signature, `View`'s fields and the `Act` variants with
//! their fields (the director emits and applies the same acts). P3 may add to `Program` and
//! `Memory`, which no other piece looks inside. Until P3 lands, `step` decides nothing and the
//! engine's own enemy code runs as before.
use super::wire::Line;
use std::collections::BTreeMap;

/// A loaded behaviour: its stance and rules. Built by P3's loader.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Program {}

/// What one enemy (or one fight script) remembers between steps: phase, rule fire counts and
/// next-allowed times, the hit counter, crossed thresholds, the stance and pending timed acts.
/// Events (spawned, engaged, hit, died, talked) are written here by the owner before a step.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Memory {}

/// What a step may read about its enemy and the fight around it.
#[derive(Clone, Copy, Debug)]
pub struct View<'a> {
    pub now_ms: u64,
    /// Own health, 0 to 100.
    pub health_pct: u32,
    pub position: [f32; 3],
    pub home: [f32; 3],
    pub target: Option<u64>,
    /// Metres to the target, when there is one.
    pub target_distance: Option<f32>,
    /// Metres to the nearest player who can be fought.
    pub nearest_player: Option<f32>,
    /// Living actors per tag.
    pub tags_alive: &'a BTreeMap<String, u32>,
    /// Signals raised since the last step, by rules or by the fight.
    pub signals: &'a [String],
    pub phase: u8,
    pub night: bool,
}

/// How an enemy moves while nothing else is going on.
#[derive(Clone, Debug, PartialEq)]
pub enum Move {
    Hold,
    Chase,
    KeepAway { min: f32, max: f32 },
    Flee { ms: u32, speed: f32 },
    Behind { tag: String, m: f32 },
    Route(String),
    Perch(String),
    Roam(f32),
    Home,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Stance {
    pub movement: Move,
    /// An attack id in `attacks.json`; `None` never attacks.
    pub attack: Option<String>,
    pub leash: bool,
}

/// Where a summon appears or a relocation ends.
#[derive(Clone, Debug, PartialEq)]
pub enum Place {
    Own,
    Target,
    /// Away from the target.
    Away,
    /// The fight's named point list for this actor.
    Points,
    Ring,
    Point(String),
    BehindAllies,
}

/// Who a heal or a status lands on.
#[derive(Clone, Debug, PartialEq)]
pub enum Who {
    Own,
    Target,
    AllPlayers,
    Boss,
    Tag(String),
}

/// The `set` action: only the named fields change.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Flags {
    pub invulnerable: Option<bool>,
    pub untargetable: Option<bool>,
    pub speed: Option<f32>,
    pub scale: Option<f32>,
    pub name_id: Option<i32>,
    pub model_id: Option<i32>,
    pub disposition: Option<i32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetRule {
    Attacker,
    Nearest,
    LowestHealth,
    Taunter,
}

/// One resolved action (design 8.2), applied by `acts`.
#[derive(Clone, Debug, PartialEq)]
pub enum Act {
    Stance(Stance),
    /// Use one attack from `attacks.json` now.
    Attack(String),
    Say(Line),
    Telegraph {
        say: Line,
        ms: u32,
        attack: String,
    },
    Summon {
        kind: String,
        count: u8,
        at: Place,
        radius: f32,
        tag: String,
        /// The helper rises from a mound with its bar showing.
        mound: bool,
    },
    /// Make dormant actors with the tag active.
    Wake(String),
    Relocate {
        to: Place,
        min: f32,
        max: f32,
        effect: Option<String>,
    },
    Heal {
        who: Who,
        pct: u32,
    },
    Status {
        id: String,
        to: Who,
        ms: u32,
    },
    Set(Flags),
    Phase(u8),
    Become {
        definition: String,
        health_pct: u32,
    },
    Signal(String),
    /// A loot table or pickup id.
    Drop(String),
    /// Stop fighting, count as defeated, stay as an NPC.
    Yield,
    /// Despawn without credit after this many ms.
    Leave(u32),
    Revive {
        pct: u32,
        after_ms: u32,
    },
    Target(TargetRule),
}

/// One decision: read the view, update the memory, push what the enemy does now.
pub fn step(_p: &Program, _m: &mut Memory, _v: &View, _out: &mut Vec<Act>) {}
