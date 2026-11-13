//! Fights as data: waves, objectives, the fight script, the exit door and the knockout limit
//! (combat design sections 10 and 11). Piece P5 (`combat-director`).
//!
//! - `director/fight.rs`: one `fights/<id>.json` file, its checks and its roster;
//! - `director/run.rs`: a fight in progress as a pure state machine;
//! - `director/bind.rs`: the engine side (loading, spawning, goal rows, score, knockouts).
//!
//! Frozen by the combat seam: `begin`, `on_fact`, `tick`, the types below and the hook names
//! `encounter.rs` and `arena.rs` call. A row of `field_battles.json` or the cave that has no
//! fight file runs as before; its run is a `legacy` one that only reports.
use super::brain::Act;
use super::*;

mod bind;
pub mod entrances;
pub mod fight;
pub mod run;

pub use self::fight::{Catalogue, Fight};
pub use self::run::{Life, Run, ScoreCard};

#[derive(Clone, Debug, PartialEq)]
pub struct Member {
    pub guid: u64,
    pub profile: u32,
    pub level: u8,
}

/// Who starts the fight; the group factor and the shared knockout allowance follow it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Party {
    pub members: Vec<Member>,
}

/// What the engine and the brain tell a run.
#[derive(Clone, Debug, PartialEq)]
pub enum Signal {
    /// A signal raised by a behaviour rule (`signal: {name}`). `arrived:<actor>` is the
    /// engine's word that an escorted actor reached the end of its route.
    Named(String),
    Died {
        guid: u64,
        kind: u32,
        tags: Vec<String>,
    },
    Talked {
        guid: u64,
    },
    PlayerDown {
        guid: u64,
    },
    /// A player crossed into a named point or room; `left:<place>` when the last one left,
    /// `exit` at the open exit door.
    Reached {
        place: String,
    },
    Collected {
        pickup: String,
    },
}

/// What a run asks the engine to do on a tick.
#[derive(Clone, Debug, PartialEq)]
pub enum DirectorAct {
    /// Spawn the named wave.
    Wave(String),
    /// One act for one actor of the fight; guid 0 is the fight itself (a line, a drop, a
    /// wake by tag).
    Actor {
        guid: u64,
        act: Act,
    },
    /// A signal for every actor's rules.
    Signal(String),
    OpenExit,
    Fact(FightFact),
    Win,
    Lose,
}

/// Start a run of `fight` for `party`.
pub fn begin(fight: &Fight, party: &Party) -> Run {
    Run::new(fight, party)
}

/// Something happened; push the acts the fight's script answers with at once and that need
/// no actor (a line, a signal, a wake). Everything else comes out of the next `tick`.
pub fn on_fact(run: &mut Run, fact: &Signal, out: &mut Vec<Act>) {
    run.fact_in(fact, out);
}

/// Advance waves, objectives and timers to `now_ms`.
pub fn tick(run: &mut Run, now_ms: u64, out: &mut Vec<DirectorAct>) {
    run.tick_to(now_ms, out);
}

/// Every fight file of a content directory (`<directory>/fights/*.json`), checked. A
/// directory without the folder has none. Studio's save check and the tests call this; the
/// engine's own load goes through it too.
pub fn load_dir(directory: &Path) -> io::Result<Vec<Fight>> {
    let folder = directory.join("fights");
    if !folder.is_dir() {
        return Ok(Vec::new());
    }
    let catalogue = Catalogue::load(directory)?;
    let pacing = super::pacing::PacingTable::load(directory)?;
    let mut paths: Vec<_> = fs::read_dir(&folder)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().is_some_and(|e| e == "json"))
        .collect();
    paths.sort();
    if paths.len() > fight::MAX_FIGHTS {
        return Err(invalid("too many fight files"));
    }
    let mut fights: Vec<Fight> = Vec::new();
    for path in paths {
        let name = format!(
            "fights/{}",
            path.file_name().unwrap_or_default().to_string_lossy()
        );
        if fs::metadata(&path)?.len() > fight::MAX_FILE_BYTES {
            return Err(invalid(&format!("{name}: file too large")));
        }
        let fight = Fight::parse_paced(&fs::read(&path)?, &catalogue, &name, &pacing)?;
        if path.file_stem().and_then(|s| s.to_str()) != Some(fight.id.to_string().as_str()) {
            return Err(invalid(&format!("{name}: the file is named after its id")));
        }
        if fights.iter().any(|other| other.id == fight.id) {
            return Err(invalid(&format!("{name}: duplicate fight id")));
        }
        fights.push(fight);
    }
    Ok(fights)
}

/// What this player's client has been told about the run, so each member of a shared run
/// gets their own goal rows, facts and results.
#[derive(Clone, Debug, Default)]
pub(super) struct Seen {
    goals: Vec<(fight::GoalState, u32, u32)>,
    knockouts: u32,
    settings: bool,
}

/// The engine's own slot for the run in progress (session only, like the arena clocks). A
/// shared arena carries it: `enter_arena` copies the arena's slot in, `leave_arena` copies it
/// back, so every member's action runs against the one run.
#[derive(Clone, Debug, Default)]
pub struct RunSlot {
    pub run: Option<Run>,
    /// Signals the fight raised for the actors' rules since the brain last took them.
    pub signals: Vec<String>,
}

/// What of a run is saved with the character (`ActiveEncounter::run_save`), so a relog resumes
/// the fight: `run::Saved` as JSON. A save that does not fit the fight is dropped.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub struct RunSave(pub serde_json::Value);

/// The engine field `fight_state`: the loaded fights and what this player alone holds of a
/// run (it outlives `reset_session`, and a shared arena never carries it).
#[derive(Clone, Debug, Default)]
pub struct FightState {
    pub(super) fights: Arc<BTreeMap<u32, Arc<Fight>>>,
    /// Chance drops of the fights that are not `field_battles.json` rows.
    pub(super) drops: Arc<BTreeMap<u32, Vec<battle_drops::BattleDrop>>>,
    /// The placed entrances of `entrances.json` (`director/entrances.rs`).
    pub(super) doors: Arc<entrances::Doors>,
    pub(super) seen: Seen,
    /// When this player went down inside the fight.
    pub(super) down_since: Option<u64>,
    /// Enemies cannot hurt this player before then: they stood up from a knockout (the fight
    /// grace of `combat_rules.json` `revive.fight_grace_ms`).
    pub(super) guarded_until: u64,
    /// When the tick first saw this player's run lost with its results card up (the fallback
    /// return of `revive.lost_exit_ms` counts from it).
    pub(super) lost_since: Option<u64>,
    /// The run as last written to the save.
    pub(super) saved: Option<run::Saved>,
    /// The overworld instance fights' pacing (`instance_pacing.json`, `pacing.rs`).
    pub(super) pacing: Arc<super::pacing::PacingTable>,
}
