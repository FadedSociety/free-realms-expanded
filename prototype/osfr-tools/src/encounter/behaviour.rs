//! Enemy behaviour rows: the `behaviours.json` presets, a row's own overrides and the engine's
//! per-enemy lookup. Moved out of `encounter.rs` by the combat seam so the brain (P3) can
//! turn these rows into rules without touching the hub file.
//!
//! Version 2 of `behaviours.json` adds the behaviour language (combat design section 8): a
//! preset or a row may carry a `stance` and `rules`, and may build on another preset with
//! `use`. The older keys still load: `mode` becomes the stance, and a battle's
//! `summon_below_percent` rows become `health_below` rules on its summoner, so old content
//! plays through the same runtime.
use super::*;
use serde_json::{Value, json};

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
    /// The language's stance (`{"move": "keep_away", "min": 8, "max": 14, "attack": "bolt"}`);
    /// absent, `mode` decides it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stance: Option<Value>,
    /// The language's when / if / do rules. A row or preset that builds on another adds to
    /// that one's rules.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<Value>,
    /// Tags other rules count and wake this enemy by (its name as a slug and `boss` are
    /// always among them).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// It takes no damage while an actor with this tag stands.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shielded_by: Option<String>,
    /// What the loader built from the fields above.
    #[serde(skip)]
    pub(super) brain: Compiled,
}

/// The loaded form of a behaviour. It never takes part in comparing two behaviours: it is
/// derived from the compared fields (plus the enemy's own name and its battle's roster).
#[derive(Clone, Debug, Default)]
pub(super) struct Compiled {
    pub(super) program: Arc<brain::Program>,
    pub(super) tags: Vec<String>,
    pub(super) shield: Option<String>,
    pub(super) data: Arc<BrainData>,
}

impl PartialEq for Compiled {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// What `behaviours.json` and `attacks.json` hold besides the presets. Every loaded
/// behaviour carries the same one.
#[derive(Clone, Debug, Default)]
pub(super) struct BrainData {
    pub(super) attacks: BTreeMap<String, brain::Attack>,
    pub(super) visuals: Visuals,
    pub(super) field: Option<FieldRules>,
    /// Enemy GUID to the preset it uses whatever its row says.
    pub(super) attach: BTreeMap<u64, String>,
}

/// (R) How a summon and a relocation look. The defaults are this build's choices for the
/// troll models (troll_shaman.adr has com_cast_01/02 and com_spawn but no com_cast_special_*;
/// the puff is PFX_bones_yellow_troll-spawn, ActorCompositeEffectDefinitions.xml 16337); the
/// live game's choices are not recovered.
#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(super) struct Visuals {
    /// The caller's cast (com_cast_02).
    pub(super) summon_animation: i32,
    /// What a helper plays as it appears (com_spawn).
    pub(super) spawn_animation: i32,
    /// The puff a helper rises from.
    pub(super) mound_effect: i32,
    /// Named effects for `relocate`'s `effect`.
    pub(super) effects: BTreeMap<String, i32>,
    /// A fallen helper may be raised again this long after it fell.
    pub(super) reuse_after_ms: u64,
    note: Option<String>,
}

impl Default for Visuals {
    fn default() -> Self {
        Self {
            summon_animation: 1112,
            spawn_animation: 1404,
            mound_effect: 16337,
            effects: BTreeMap::new(),
            reuse_after_ms: 6000,
            note: None,
        }
    }
}

/// The open-world rules of design 5.4 that are not a row's own: what the client is told when
/// an enemy starts to fight or gives up, how fast it runs home and how far a neighbour joins.
/// Absent from `behaviours.json`, none of it is sent and the enemy returns at its `run_speed`.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FieldRules {
    pub(super) home_speed: f32,
    pub(super) idle_state: u32,
    pub(super) fight_state: u32,
    pub(super) home_state: u32,
    pub(super) home_tag: i32,
    pub(super) home_effect: i32,
    pub(super) join_radius: f32,
    #[serde(default)]
    #[allow(dead_code)]
    note: Option<String>,
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
            stance: None,
            rules: Vec::new(),
            tags: Vec::new(),
            shielded_by: None,
            brain: Compiled::default(),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BehaviourFile {
    #[allow(dead_code)]
    schema_version: u32,
    /// Each preset as written; `use` is resolved after the whole file is read.
    pub(super) presets: BTreeMap<String, Value>,
    /// Evidence entries for the content checker.
    #[serde(default)]
    #[allow(dead_code)]
    source: Vec<serde_json::Value>,
    #[serde(default)]
    visuals: Visuals,
    #[serde(default)]
    field: Option<FieldRules>,
    #[serde(default)]
    attach: BTreeMap<u64, String>,
}

/// `data/content/attacks.json` as built, for a `behaviours.json` that has none beside it.
const SHIPPED_ATTACKS: &str = include_str!("../../data/content/attacks.json");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AttackFile {
    #[allow(dead_code)]
    schema_version: u32,
    #[serde(default)]
    #[allow(dead_code)]
    source: Vec<Value>,
    attacks: Value,
}

/// `fields` laid over `start`: a key replaces the one below it, while `rules` and `tags`
/// add to it. `preset` and `use` name `start` and are not fields.
fn merged(start: &Behaviour, fields: &serde_json::Map<String, Value>) -> Result<Behaviour, String> {
    let mut merged = serde_json::to_value(start).map_err(|e| e.to_string())?;
    for (key, value) in fields {
        match key.as_str() {
            "preset" | "use" => {}
            "rules" | "tags" => {
                let mut all = merged[key].as_array().cloned().unwrap_or_default();
                all.extend(
                    value
                        .as_array()
                        .cloned()
                        .ok_or(format!("{key} must be a list"))?,
                );
                merged[key] = Value::Array(all);
            }
            _ => merged[key] = value.clone(),
        }
    }
    let mut behaviour: Behaviour = serde_json::from_value(merged).map_err(|e| e.to_string())?;
    behaviour.brain.data = start.brain.data.clone();
    Ok(behaviour)
}

/// One preset with everything it builds on (`use`), checked and compiled.
fn resolve_preset(
    name: &str,
    raw: &BTreeMap<String, Value>,
    data: &Arc<BrainData>,
    done: &mut BTreeMap<String, Behaviour>,
    depth: u8,
) -> Result<Behaviour, String> {
    if let Some(found) = done.get(name) {
        return Ok(found.clone());
    }
    let fields = raw
        .get(name)
        .ok_or(format!("unknown behaviour preset '{name}'"))?
        .as_object()
        .ok_or(format!("preset {name}: must be an object"))?;
    let mut start = match fields.get("use") {
        None => Behaviour::default(),
        Some(_) if depth >= 8 => {
            return Err(format!("preset {name}: 'use' goes round in a circle"));
        }
        Some(Value::String(base)) => resolve_preset(base, raw, data, done, depth + 1)
            .map_err(|e| format!("preset {name}: {e}"))?,
        Some(_) => return Err(format!("preset {name}: 'use' must be a preset name")),
    };
    start.brain.data = data.clone();
    let mut behaviour = merged(&start, fields).map_err(|e| format!("preset {name}: {e}"))?;
    behaviour
        .compile(&[])
        .map_err(|e| format!("preset {name}: {e}"))?;
    done.insert(name.to_string(), behaviour.clone());
    Ok(behaviour)
}

/// An enemy's name as a tag: lower case, anything else an underscore ("Troll Scoutling" is
/// `troll_scoutling`). `summon` and `become` find their helpers by it.
pub(super) fn slug(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

/// An enemy's behaviour: its record's `behaviour` (a preset name, or an
/// object with an optional `preset` plus overrides) over the default preset.
pub(super) fn resolve_behaviour(
    actor: &Actor,
    presets: &BTreeMap<String, Behaviour>,
) -> Result<Behaviour, String> {
    let data = presets.values().next().map(|p| p.brain.data.clone());
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
    // `attach` in `behaviours.json` wins over the row: content owned elsewhere can be given
    // a behaviour without editing its file.
    let attached = data.as_ref().and_then(|d| d.attach.get(&actor.guid));
    match (attached, &actor.behaviour) {
        (Some(name), _) => named(name),
        (None, None) => Ok(base),
        (None, Some(serde_json::Value::String(name))) => named(name),
        (None, Some(serde_json::Value::Object(fields))) => {
            let start = match fields.get("preset").or(fields.get("use")) {
                Some(serde_json::Value::String(name)) => named(name)?,
                Some(_) => return Err(format!("{}: behaviour preset must be a name", actor.name)),
                None => base,
            };
            merged(&start, fields)
                .map_err(|e| format!("{} ({}): behaviour {e}", actor.name, actor.guid))
        }
        (None, Some(_)) => Err(format!(
            "{}: behaviour must be a preset name or an object",
            actor.name
        )),
    }
    .and_then(|mut b| {
        if let Some(data) = &data {
            b.brain.data = data.clone();
        }
        b.brain.tags = b.tags.clone();
        b.brain.tags.extend(actor.tags.iter().cloned());
        b.brain.tags.push(slug(&actor.name));
        if actor.boss {
            b.brain.tags.push("boss".into());
        }
        b.compile(&[])
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

    /// Check the numbers and build the program: the stance (`mode` when none is written),
    /// the rules, then `more` (rules the loader adds for old content), with the row's own
    /// swing as the `melee` attack.
    fn compile(&mut self, more: &[Value]) -> Result<(), String> {
        self.check()?;
        let (movement, attack) = match self.mode {
            Mode::Melee => (brain::Move::Chase, Some(brain::Limits::MELEE.to_string())),
            Mode::Stationary => (brain::Move::Hold, Some(brain::Limits::MELEE.to_string())),
            Mode::Skittish => (
                brain::Move::Flee {
                    ms: self.run_ms.min(u64::from(u32::MAX)) as u32,
                    speed: self.run_speed,
                },
                None,
            ),
        };
        let fallback = brain::Stance {
            movement,
            attack,
            leash: true,
        };
        let rules: Vec<Value> = self.rules.iter().chain(more).cloned().collect();
        let program = brain::Program::build(
            self.stance.as_ref(),
            &rules,
            fallback,
            brain::Attack::melee(self.reach, self.attack_ms),
            &self.brain.data.attacks,
        )?;
        self.brain.program = Arc::new(program);
        self.brain.shield = self.shielded_by.clone();
        Ok(())
    }

    /// The compiled program (for Studio and tests).
    pub fn program(&self) -> &brain::Program {
        &self.brain.program
    }
}

/// Read `behaviours.json` at `path`, with `attacks.json` beside it when there is one, into
/// checked and compiled presets and the data they share. Any refusal names the preset and
/// the rule.
pub(super) fn read_presets(
    path: &Path,
) -> io::Result<(BTreeMap<String, Behaviour>, Arc<BrainData>)> {
        let bytes = fs::read(path)?;
        if bytes.len() > 256 * 1024 {
            return Err(invalid("behaviour presets too large"));
        }
        let file: BehaviourFile = serde_json::from_slice(&bytes)
            .map_err(|e| invalid(&format!("behaviours.json: {e}")))?;
        if !(1..=2).contains(&file.schema_version) {
            return Err(invalid("behaviours.json: schema_version must be 1 or 2"));
        }
        // Enemy attacks sit beside the presets.
        let attacks = match path.parent().map(|dir| dir.join("attacks.json")) {
            Some(attacks) if attacks.is_file() => {
                let bytes = fs::read(&attacks)?;
                if bytes.len() > 256 * 1024 {
                    return Err(invalid("attacks.json too large"));
                }
                let file: AttackFile = serde_json::from_slice(&bytes)
                    .map_err(|e| invalid(&format!("attacks.json: {e}")))?;
                brain::parse_attacks(&file.attacks)
                    .map_err(|e| invalid(&format!("attacks.json: {e}")))?
            }
            // No file beside the presets (an older package, a test's private copy): the
            // attacks this build shipped with, so the shipped presets always load.
            _ => {
                let file: AttackFile = serde_json::from_str(SHIPPED_ATTACKS)
                    .map_err(|e| invalid(&format!("shipped attacks.json: {e}")))?;
                brain::parse_attacks(&file.attacks)
                    .map_err(|e| invalid(&format!("shipped attacks.json: {e}")))?
            }
        };
        let data = Arc::new(BrainData {
            attacks,
            visuals: file.visuals,
            field: file.field,
            attach: file.attach,
        });
        let mut presets = BTreeMap::new();
        for name in file.presets.keys() {
            resolve_preset(name, &file.presets, &data, &mut presets, 0)
                .map_err(|e| invalid(&format!("behaviours.json: {e}")))?;
        }
        if let Some((guid, name)) = data.attach.iter().find(|(_, n)| !presets.contains_key(*n)) {
            return Err(invalid(&format!(
                "behaviours.json: attach {guid} names unknown preset '{name}'"
            )));
        }
        Ok((presets, data))
}

impl EncounterEngine {
    /// Load `behaviours.json` presets (also a hot reload).
    pub fn load_behaviours(&mut self, path: &Path) -> io::Result<()> {
        let (presets, data) = read_presets(path)?;
        let previous = std::mem::replace(&mut self.behaviour_presets, presets);
        let previous_data = std::mem::replace(&mut self.brain_state.data, data);
        if let Err(error) = self.rebuild_behaviours() {
            self.behaviour_presets = previous;
            self.brain_state.data = previous_data;
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
        // Old summoner battles (`summon_below_percent` on the helpers): each helper gets its
        // tags, and every other enemy of the battle that is still written the old way gets
        // the ladder as rules. A skittish one fights beside its helpers, flees when alone and
        // is shielded while they stand, as before.
        for d in &self.content.dungeons {
            let ladder: BTreeSet<u32> = d
                .enemies
                .iter()
                .filter_map(|a| a.summon_below_percent)
                .filter(|below| *below > 0)
                .collect();
            if ladder.is_empty() {
                continue;
            }
            for a in &d.enemies {
                let Some(b) = resolved.get_mut(&a.guid) else {
                    continue;
                };
                if let Some(below) = a.summon_below_percent {
                    b.brain.tags.push("summoned".into());
                    b.brain.tags.push(format!("summon_below_{below}"));
                    continue;
                }
                // A row with rules of its own writes its own ladder.
                if !b.rules.is_empty() {
                    continue;
                }
                let skittish = b.mode == Mode::Skittish;
                let mut rules = Vec::new();
                for below in &ladder {
                    let mut acts = vec![json!({"wake": {"tag": format!("summon_below_{below}")}})];
                    if skittish {
                        acts.push(json!({"stance": {"move": "chase", "attack": "melee"}}));
                    }
                    rules.push(json!({"id": format!("summon_below_{below}"), "once": true,
                        "when": {"health_below": {"pct": (*below).min(100)}}, "do": acts}));
                }
                if skittish {
                    rules.push(json!({"id": "alone",
                        "when": {"tag_dead": {"tag": "summoned", "left": 0}},
                        "do": [{"stance": {"move": "flee", "ms": b.run_ms.max(250),
                            "speed": b.run_speed}}]}));
                }
                b.compile(&rules)
                    .map_err(|e| invalid(&format!("{} ({}): {e}", a.name, a.guid)))?;
                if skittish && b.brain.shield.is_none() {
                    b.brain.shield = Some("summoned".into());
                }
            }
        }
        self.behaviour_of = Arc::new(resolved);
        Ok(())
    }

    /// The brain data every behaviour shares (empty until `behaviours.json` loads).
    pub(super) fn brain_data(&self) -> Arc<BrainData> {
        self.brain_state.data.clone()
    }

    /// An enemy's resolved behaviour (for Studio and tests).
    pub fn behaviour(&self, guid: u64) -> Option<&Behaviour> {
        self.behaviour_of.get(&guid)
    }
}
