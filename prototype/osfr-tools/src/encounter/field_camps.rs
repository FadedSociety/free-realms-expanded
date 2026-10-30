//! T892: camp enemies. The Legacy overworld spawns that `npc_spawns_extra.json` draws as friendly
//! NPC actors although they are enemies (Sandscale, Skullz, cultists, crays, widows, ...) fight
//! through the field enemy system instead: hostile name, fightable, respawning.
//!
//! `field_camps.json` lists camps. A camp names an `enemies.json` definition (stats and the
//! kill-credit definition id), a level, the field policy (respawn, aggro, leash, behaviour,
//! drop table) and the `spawn_guid` of every body it takes over. Each body keeps its position,
//! model, texture and name from `npc_spawns_extra.json`, so there is one placement store; the
//! field GUID is the body's GUID, so the passive copy is hidden by the same exclusion that hides
//! `field_enemies.json` rows (`Gameplay` excludes `field_guids`). Rows are expanded into the
//! rows `field_enemies.json` produces and then validated by the same code
//! (`build_field_enemy`), so level scaling, the combat marker and the GUID checks are shared.
use super::field_content::check_sources;
use super::*;

const FILE: &str = "field_camps.json";
const MAX_CAMPS: usize = 256;
const MAX_MEMBERS: usize = 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CampFile {
    schema_version: u32,
    camps: Vec<Camp>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Camp {
    id: String,
    /// `enemies.json` definition `id`.
    definition: String,
    level: u32,
    respawn_seconds: u32,
    aggro_range: f32,
    leash_radius: f32,
    /// A `behaviours.json` preset name.
    behaviour: String,
    /// A `power_ups.json` drop table id; absent drops nothing.
    #[serde(default)]
    power_up_table: Option<String>,
    members: Vec<u64>,
    source: Vec<serde_json::Value>,
}

/// The slice of `enemies.json` a camp needs (the catalogue loader in `enemies.rs` is stricter and
/// private to that module).
#[derive(Deserialize)]
struct DefinitionFile {
    definitions: Vec<Definition>,
}

#[derive(Deserialize)]
struct Definition {
    id: String,
    kind: String,
    definition_id: i32,
    name_string_id: i32,
    max_health: u32,
    damage: u32,
    boss: bool,
}

/// The slice of `npc_spawns_extra.json` a camp needs: the body that becomes the enemy.
#[derive(Deserialize)]
struct BodyFile {
    records: Vec<Body>,
}

#[derive(Deserialize)]
struct Body {
    spawn_guid: u64,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    model_id: i32,
    #[serde(default)]
    name_string_id: Option<i32>,
    #[serde(default)]
    texture_alias: String,
    position: [f32; 3],
    #[serde(default)]
    capture_match: Option<serde_json::Value>,
}

/// The shared combat marker row of `field_enemies.json`.
#[derive(Deserialize)]
struct SharedMarker {
    #[serde(default)]
    combat_marker: Option<Notification>,
}

fn read<T: for<'a> Deserialize<'a>>(path: &Path, max_bytes: u64) -> io::Result<T> {
    let refuse = |why: &dyn std::fmt::Display| invalid(&format!("{}: {why}", path.display()));
    if fs::metadata(path).map_err(|e| refuse(&e))?.len() > max_bytes {
        return Err(refuse(&"file too large"));
    }
    serde_json::from_slice(&fs::read(path).map_err(|e| refuse(&e))?).map_err(|e| refuse(&e))
}

impl EncounterEngine {
    /// Add the camp enemies of `directory/field_camps.json` to the loaded field enemies. A content
    /// directory without the file has no camps. Load after [`Self::load_field_enemies`] (which
    /// replaces the field rows) and [`Self::load_behaviours`]; every refusal leaves the engine as
    /// it was, and loading the same camps twice is refused by the duplicate-GUID check.
    pub fn load_field_camps(&mut self, directory: &Path) -> io::Result<()> {
        let path = directory.join(FILE);
        if !path.is_file() {
            return Ok(());
        }
        let file: CampFile = read(&path, 256 * 1024)?;
        if file.schema_version != 1
            || file.camps.len() > MAX_CAMPS
            || file.camps.iter().map(|c| c.members.len()).sum::<usize>() > MAX_MEMBERS
        {
            return Err(invalid("unsupported field camp content"));
        }
        let definitions: DefinitionFile = read(&directory.join("enemies.json"), 1 << 20)?;
        let bodies: BodyFile = read(&directory.join("npc_spawns_extra.json"), 16 << 20)?;
        let marker: SharedMarker = read(&directory.join("field_enemies.json"), 1 << 20)?;
        let bodies: BTreeMap<u64, Body> = bodies
            .records
            .into_iter()
            .map(|body| (body.spawn_guid, body))
            .collect();
        let mut guids = self.claimed_guids();
        guids.extend(self.field_sim.enemies.iter().map(|f| f.actor.guid));
        let mut areas = BTreeSet::new();
        let mut ids = BTreeSet::new();
        let mut field = Vec::new();
        for camp in file.camps {
            if camp.id.is_empty()
                || camp.id.len() > 120
                || camp.id.chars().any(char::is_control)
                || !ids.insert(camp.id.clone())
                || camp.members.is_empty()
            {
                return Err(invalid("invalid field camp"));
            }
            check_sources("field camp", &camp.source)?;
            let definition = definitions
                .definitions
                .iter()
                .find(|d| d.id == camp.definition)
                .filter(|d| !d.boss)
                .ok_or_else(|| invalid("field camp needs a non-boss enemy definition"))?;
            for guid in camp.members {
                let body = bodies
                    .get(&guid)
                    .filter(|body| body.capture_match.is_none())
                    .ok_or_else(|| invalid("field camp member is not a drawn extras body"))?;
                let actor = FieldActor {
                    actor: Actor {
                        guid,
                        definition_id: definition.definition_id,
                        model_id: body.model_id,
                        name_id: body.name_string_id.unwrap_or(definition.name_string_id),
                        name: body.name.clone().unwrap_or_else(|| definition.kind.clone()),
                        texture_alias: body.texture_alias.clone(),
                        position: body.position,
                        max_health: definition.max_health,
                        damage: definition.damage,
                        level: camp.level,
                        boss: false,
                        stages: Vec::new(),
                        summon_below_percent: None,
                        skittish: false,
                        scale: None,
                        behaviour: Some(camp.behaviour.clone().into()),
                        power_up_table: camp.power_up_table.clone(),
                    },
                    respawn_seconds: Some(camp.respawn_seconds),
                    aggro_range: Some(camp.aggro_range),
                    level_band: None,
                    spawn_area: None,
                    leash_radius: Some(camp.leash_radius),
                    region: None,
                    zone_id: self.content.world.zone_id,
                    quest_ids: Vec::new(),
                    combat_marker: MarkerSetting::Shared,
                    source: serde_json::Value::Null,
                };
                field.push(self.build_field_enemy(
                    actor,
                    &mut guids,
                    &mut areas,
                    marker.combat_marker,
                )?);
            }
        }
        Arc::make_mut(&mut self.content).field_sites.extend(
            field
                .iter()
                .map(|f| (f.actor.definition_id, f.actor.guid, f.actor.position)),
        );
        self.field_sim.enemies.extend(field);
        self.rebuild_behaviours()
    }
}
