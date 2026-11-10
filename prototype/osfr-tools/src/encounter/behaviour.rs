//! Enemy behaviour rows: the `behaviours.json` presets, a row's own overrides and the engine's
//! per-enemy lookup. Moved out of `encounter.rs` by the combat seam so the brain (P3) can
//! turn these rows into rules without touching the hub file.
use super::*;

/// How an enemy moves and fights. Named presets live in `behaviours.json`;
/// an enemy record picks one with `"behaviour": "skittish"` or adjusts one
/// with `{"preset": "melee", "move_speed": 4.0}`. Without either it is
/// `melee` (or `skittish` for the older `skittish: true` flag).
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// Chase and swing.
    Melee,
    /// Fights beside its summoned minions; alone it runs, rests, runs again
    /// (the Troll Summoner, the owner's design).
    Skittish,
    /// Never moves; swings at players within reach.
    Stationary,
}

/// (R) Every default is this build's original constant, so content without
/// presets behaves as before.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Behaviour {
    pub mode: Mode,
    /// Chase speed in m/s.
    pub move_speed: f32,
    /// Melee reach in metres.
    pub reach: f32,
    /// Time between attacks in ms.
    pub attack_ms: u64,
    /// In arenas, stops chasing a player farther than this (m).
    pub chase_limit: f32,
    /// Skittish cycle when alone: run for `run_ms` at `run_speed` (slower
    /// than a running player), then rest `rest_ms` playing `rest_animation`
    /// (com_stunned, AnimationTypes.xml 1401) without attacking - the window
    /// to hit it. Stays within `leash` m of its spawn, sliding along that edge
    /// when cornered, and only runs while the player is within `run_from` m
    /// (farther away it stands and watches, and that time does not tire it).
    /// It follows the terrain grid (`--terrain`); without one it takes the
    /// player's ground height (a fixed height sank it under the slopes at the
    /// clearing's edge in run 053415 - "phasing out of reality").
    pub run_ms: u64,
    pub rest_ms: u64,
    pub run_speed: f32,
    pub leash: f32,
    pub run_from: f32,
    pub rest_animation: i32,
    /// Free text for content authors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl Default for Behaviour {
    fn default() -> Self {
        Self {
            mode: Mode::Melee,
            move_speed: 3.0,
            reach: 2.5,
            attack_ms: 1000,
            chase_limit: 40.0,
            run_ms: 4000,
            rest_ms: 3000,
            run_speed: 3.5,
            leash: 12.0,
            run_from: 10.0,
            rest_animation: 1401,
            note: None,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BehaviourFile {
    #[allow(dead_code)]
    schema_version: u32,
    pub(super) presets: BTreeMap<String, Behaviour>,
    /// Evidence entries for the content checker.
    #[serde(default)]
    #[allow(dead_code)]
    source: Vec<serde_json::Value>,
}

/// An enemy's behaviour: its record's `behaviour` (a preset name, or an
/// object with an optional `preset` plus overrides) over the default preset.
pub(super) fn resolve_behaviour(
    actor: &Actor,
    presets: &BTreeMap<String, Behaviour>,
) -> Result<Behaviour, String> {
    let named = |name: &str| {
        presets.get(name).cloned().ok_or(format!(
            "{} ({}): unknown behaviour preset '{name}'",
            actor.name, actor.guid
        ))
    };
    let base = if actor.skittish {
        presets.get("skittish").cloned().unwrap_or(Behaviour {
            mode: Mode::Skittish,
            ..Behaviour::default()
        })
    } else {
        presets.get("melee").cloned().unwrap_or_default()
    };
    match &actor.behaviour {
        None => Ok(base),
        Some(serde_json::Value::String(name)) => named(name),
        Some(serde_json::Value::Object(fields)) => {
            let start = match fields.get("preset") {
                Some(serde_json::Value::String(name)) => named(name)?,
                Some(_) => return Err(format!("{}: behaviour preset must be a name", actor.name)),
                None => base,
            };
            let mut merged = serde_json::to_value(start).map_err(|e| e.to_string())?;
            for (key, value) in fields.iter().filter(|(key, _)| *key != "preset") {
                merged[key] = value.clone();
            }
            serde_json::from_value(merged)
                .map_err(|e| format!("{} ({}): behaviour {e}", actor.name, actor.guid))
        }
        Some(_) => Err(format!(
            "{}: behaviour must be a preset name or an object",
            actor.name
        )),
    }
    .and_then(|b| {
        b.check()
            .map(|()| b)
            .map_err(|e| format!("{} ({}): {e}", actor.name, actor.guid))
    })
}

impl Behaviour {
    /// Any value is tunable, but not nonsense (negative, infinite, or a
    /// reach past the arena's chase limit).
    fn check(&self) -> Result<(), String> {
        let distances = [
            ("move_speed", self.move_speed),
            ("reach", self.reach),
            ("chase_limit", self.chase_limit),
            ("run_speed", self.run_speed),
            ("leash", self.leash),
            ("run_from", self.run_from),
        ];
        for (name, value) in distances {
            if !value.is_finite() || value < 0.0 || value > 1000.0 {
                return Err(format!("behaviour {name} {value} must be 0..1000"));
            }
        }
        if self.reach > self.chase_limit {
            return Err(format!(
                "behaviour reach {} is beyond its chase_limit {}",
                self.reach, self.chase_limit
            ));
        }
        Ok(())
    }
}

impl EncounterEngine {
    /// Load `behaviours.json` presets (also a hot reload).
    pub fn load_behaviours(&mut self, path: &Path) -> io::Result<()> {
        let bytes = fs::read(path)?;
        if bytes.len() > 256 * 1024 {
            return Err(invalid("behaviour presets too large"));
        }
        let file: BehaviourFile = serde_json::from_slice(&bytes)
            .map_err(|e| invalid(&format!("behaviours.json: {e}")))?;
        let previous = std::mem::replace(&mut self.behaviour_presets, file.presets);
        if let Err(error) = self.rebuild_behaviours() {
            self.behaviour_presets = previous;
            self.rebuild_behaviours()?;
            return Err(error);
        }
        Ok(())
    }

    pub(super) fn rebuild_behaviours(&mut self) -> io::Result<()> {
        let actors = self
            .content
            .dungeons
            .iter()
            .flat_map(|d| d.enemies.iter())
            .chain(self.content.world_enemies.iter())
            .chain(self.field_sim.enemies.iter().map(|f| &f.actor));
        let mut resolved = BTreeMap::new();
        for actor in actors {
            let behaviour =
                resolve_behaviour(actor, &self.behaviour_presets).map_err(|e| invalid(&e))?;
            resolved.insert(actor.guid, behaviour);
        }
        self.behaviour_of = resolved;
        Ok(())
    }

    /// An enemy's resolved behaviour (for Studio and tests).
    pub fn behaviour(&self, guid: u64) -> Option<&Behaviour> {
        self.behaviour_of.get(&guid)
    }
}
