//! The actors a player clicks to open a dungeon or a battle (`entrances.json`): a dungeon's door
//! at its point of interest, a wandering battle's starter creature, and the scene each one runs
//! in. Piece `combat-jobs` (entrances).
//!
//! Every entrance becomes one more starter of the battle list `field_battles.json` fills, so the
//! click (26/8), the start card (41/114, then 41/107), Go (41/108), the level and prize gates,
//! the shared view and the group invite are the ones the 11 battle markers already use.
//!
//! - `fight` names a `field_battles.json` row: the entrance runs that battle under its own
//!   encounter id, and counts as it for quests. With `enter_scene` the battle is set down in the
//!   entrance's scene; without it the battle keeps its own arena.
//! - `fight` names a `fights/<id>.json` file: the entrance is that fight's starter, and the
//!   fight's own `place` decides the scene.
//! - No such fight is loaded: the actor stands closed. A click answers with the client's own
//!   refusal (41/4) and nothing can begin.
//!
//! As captured (p12.pcap, 2014-03-31) a click went 26/8, 26/9 with one "Fight" entry, 26/10,
//! then 41/120 "Your job has been switched..." and straight into the zone load. `launch` picks
//! that form (`direct`) or the waiting card with Go (`offer`), the form with native proof.
use super::*;
use serde_json::{Value, json};

pub const FILE: &str = "entrances.json";
const MAX_BYTES: u64 = 512 * 1024;
const MAX_ROWS: usize = 256;
/// Guids of the enemies of a battle run through an entrance: one block per encounter id, clear
/// of every authored guid (the loader checks).
const ENEMY_GUIDS: u64 = 0x454E_0000_0000_0000;

/// What a click does once the gates pass.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Launch {
    /// The waiting start card; Go begins.
    #[default]
    Offer,
    /// The one-entry "Fight" menu the client picks by itself; its 26/10 begins.
    Direct,
}

/// The single menu entry of the captured 26/9: event, icon, name, type, then the encounter.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FightMenu {
    pub event_id: i32,
    pub icon_id: i32,
    pub name_id: i32,
    pub entry_type: i32,
}

/// One scene a fight runs in: the zone row the client is sent.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    pub scene: String,
    pub zone_id: u32,
    pub sky: String,
    pub geometry_id: i32,
    pub arrive: [f32; 3],
    pub exit_door: [f32; 3],
    /// (R) Where a battle written for another arena is set down: the middle of its enemies.
    pub stage: [f32; 3],
    #[serde(default)]
    pub source: Vec<Value>,
}

impl Scene {
    /// The arrival faces the way the clearing's row does (no rotation), the only form loaded
    /// natively so far.
    pub fn zone(&self) -> ZoneTransition {
        ZoneTransition {
            zone_id: self.zone_id,
            name: self.scene.clone(),
            sky: self.sky.clone(),
            geometry_id: self.geometry_id,
            position: self.arrive,
            rotation: [0.0; 3],
            tutorial: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntranceKind {
    Door,
    Starter,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Entrance {
    pub id: String,
    pub kind: EntranceKind,
    pub title: String,
    pub guid: u64,
    /// The id this entrance's own run goes by when `fight` is a `field_battles.json` row, and
    /// while it stands closed.
    pub encounter_id: u32,
    pub name: String,
    pub name_id: i32,
    pub model_id: i32,
    pub scale: f32,
    pub at: [f32; 3],
    pub heading: f32,
    /// 0 stands still (a door).
    pub roam: f32,
    pub fight: Option<u32>,
    /// The battle a fight file's run still counts as for quests, kills and results (a door
    /// whose `field_battles.json` row was replaced by the dungeon's own fight).
    #[serde(default)]
    pub counts_as: Option<u32>,
    pub scene: Option<String>,
    #[serde(default)]
    pub enter_scene: bool,
    #[serde(default)]
    pub source: Vec<Value>,
}

/// An entrance the file lists but does not place, with its reason.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Held {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Table {
    schema_version: u32,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub launch: Launch,
    pub fight_menu: FightMenu,
    /// Text of the 41/120 sent when the entry switched the player's job.
    pub job_switch_text_id: i32,
    /// Text of the 41/4 a closed entrance answers with.
    pub closed_text_id: i32,
    /// The marker an open entrance shows; a row whose `name_id` is 0 takes the actor's own.
    #[serde(default)]
    combat_marker: Option<Notification>,
    #[serde(default)]
    pub source: Vec<Value>,
    pub scenes: Vec<Scene>,
    pub entrances: Vec<Entrance>,
    #[serde(default)]
    pub held: Vec<Held>,
}

impl Table {
    /// `<directory>/entrances.json`, checked; `None` without the file (no entrances).
    ///
    /// # Errors
    /// A file that is too large, does not parse or breaks a rule below, as
    /// `entrances.json: <row>: <reason>`.
    pub fn load(directory: &Path) -> io::Result<Option<Self>> {
        let path = directory.join(FILE);
        if !path.is_file() {
            return Ok(None);
        }
        let refuse = |why: &str| invalid(&format!("{FILE}: {why}"));
        if fs::metadata(&path)?.len() > MAX_BYTES {
            return Err(refuse("file too large"));
        }
        let table: Self =
            serde_json::from_slice(&fs::read(&path)?).map_err(|e| refuse(&e.to_string()))?;
        table.check().map_err(|why| refuse(&why))?;
        Ok(Some(table))
    }

    pub fn scene(&self, name: &str) -> Option<&Scene> {
        self.scenes.iter().find(|scene| scene.scene == name)
    }

    fn check(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("schema_version must be 1".into());
        }
        if self.scenes.len() > MAX_ROWS || self.entrances.len() > MAX_ROWS {
            return Err(format!("at most {MAX_ROWS} scenes and entrances"));
        }
        let menu = self.fight_menu;
        if self.job_switch_text_id <= 0
            || self.closed_text_id <= 0
            || menu.event_id <= 0
            || menu.name_id <= 0
        {
            return Err("the text and menu ids must be positive".into());
        }
        if let Some(why) = self.combat_marker.and_then(|row| row.validate()) {
            return Err(format!("combat_marker: {why}"));
        }
        let mut names = BTreeSet::new();
        for scene in &self.scenes {
            if validate_zone(&scene.zone()).is_err()
                || scene.zone_id == 0
                || !valid_position(scene.exit_door)
                || !valid_position(scene.stage)
                || !names.insert(scene.scene.as_str())
            {
                return Err(format!(
                    "scene {}: not a valid, unique zone row",
                    scene.scene
                ));
            }
        }
        let (mut ids, mut guids, mut encounters) =
            (BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
        for row in &self.entrances {
            let fresh = ids.insert(row.id.as_str())
                && guids.insert(row.guid)
                && encounters.insert(row.encounter_id);
            if !fresh
                || row.id.is_empty()
                || row.guid <= 1
                || row.encounter_id == 0
                || row.encounter_id >= 1 << 24
                || row.model_id <= 0
                || row.name_id <= 0
                || row.name.is_empty()
                || row.name.len() > 120
                || row.name.chars().any(char::is_control)
                || !valid_position(row.at)
                || !(row.scale > 0.0 && row.scale <= 20.0)
                || !row.heading.is_finite()
                || !(0.0..=50.0).contains(&row.roam)
            {
                return Err(format!("{}: not a valid, unique actor", row.id));
            }
            if row.scene.as_deref().is_some_and(|s| !names.contains(s)) {
                return Err(format!("{}: its scene has no row", row.id));
            }
            if row.enter_scene && row.scene.is_none() {
                return Err(format!("{}: enter_scene needs a scene", row.id));
            }
        }
        Ok(())
    }
}

/// What the engine keeps of one placed entrance, by its actor's guid.
#[derive(Clone, Copy, Debug)]
struct Door {
    encounter: u32,
    open: bool,
    heading: f32,
    scale: f32,
    fixed: bool,
}

/// The placed entrances (`FightState::doors`). Empty without the file.
#[derive(Clone, Debug, Default)]
pub struct Doors {
    table: Option<Table>,
    rows: BTreeMap<u64, Door>,
    /// An entrance's own encounter id to the `field_battles.json` battle it runs.
    counts_as: BTreeMap<u32, u32>,
}

impl Doors {
    /// A fight file that names one of the table's scenes and carries no sky of its own takes
    /// the scene row's sky and geometry id.
    pub(in crate::encounter) fn complete(&self, zone: ZoneTransition) -> ZoneTransition {
        match &self.table {
            Some(table) => table.complete(zone),
            None => zone,
        }
    }

    /// The fight an encounter id is for quests, kills and results.
    pub(in crate::encounter) fn fight(&self, encounter: u32) -> u32 {
        self.counts_as.get(&encounter).copied().unwrap_or(encounter)
    }

    /// An actor that never walks (a door).
    pub(in crate::encounter) fn fixed(&self, guid: u64) -> bool {
        self.rows.get(&guid).is_some_and(|door| door.fixed)
    }

    /// Heading and scale of an entrance's actor.
    pub(in crate::encounter) fn pose(&self, guid: u64) -> Option<(f32, f32)> {
        self.rows.get(&guid).map(|door| (door.heading, door.scale))
    }
}

impl Table {
    fn complete(&self, mut zone: ZoneTransition) -> ZoneTransition {
        if let Some(scene) = self.scene(&zone.name).filter(|_| zone.sky.is_empty()) {
            zone.sky.clone_from(&scene.sky);
            if zone.geometry_id == 0 {
                zone.geometry_id = scene.geometry_id;
            }
        }
        zone
    }
}

/// 41/4 EncounterLaunchFailed: the 12-byte encounter header and one text id, shown as a system
/// message (client reader `00a9ec30`, dispatcher `00aa36c0` case 4, which also drops a pending
/// offer).
fn launch_failed(id: u32, instance: i32, text_id: i32) -> Vec<u8> {
    let mut b = encounter_header(4, id, instance);
    b.extend(text_id.to_le_bytes());
    b
}

/// 41/120 EncounterParticipantMessage: header, text id, the player's guid and a name the text
/// may quote as `invited_player` (client reader `00a9cf90`, dispatcher `00aa36c0` case 0x78).
fn participant_message(id: u32, instance: i32, text_id: i32, player: u64) -> Vec<u8> {
    let mut b = encounter_header(120, id, instance);
    b.extend(text_id.to_le_bytes());
    b.extend(player.to_le_bytes());
    b.extend(0_u32.to_le_bytes());
    b
}

/// 26/9 with the captured single entry (event, icon, name, type, the encounter); the client
/// picks a lone entry by itself and answers 26/10.
fn fight_menu(guid: u64, name: &str, menu: FightMenu, encounter: u32) -> Vec<u8> {
    let mut b = header(26, 9);
    b.extend(guid.to_le_bytes());
    b.push(1);
    b.extend(1_i32.to_le_bytes());
    let entry = [
        menu.event_id,
        menu.icon_id,
        menu.name_id,
        menu.entry_type,
        encounter as i32,
        0,
        0,
    ];
    for v in entry {
        b.extend(v.to_le_bytes());
    }
    b.extend((name.len() as u32).to_le_bytes());
    b.extend(name.bytes());
    b.extend([1, 0]);
    b
}

/// (R) Set a roster written for another arena down around `stage`, keeping its shape.
fn set_down(enemies: &mut [Actor], stage: [f32; 3]) {
    let n = enemies.len().max(1) as f32;
    let mut middle = [0.0_f32; 3];
    for actor in enemies.iter() {
        for (sum, v) in middle.iter_mut().zip(actor.position) {
            *sum += v / n;
        }
    }
    for actor in enemies {
        for axis in 0..3 {
            actor.position[axis] += stage[axis] - middle[axis];
        }
    }
}

impl EncounterEngine {
    /// Place every entrance of `<directory>/entrances.json` as a starter. The battle file's
    /// load ends with this (a reload too), because an entrance may run one of its rows.
    ///
    /// # Errors
    /// A bad file, or a guid or encounter id another actor or encounter already holds.
    pub(in crate::encounter) fn place_entrances(&mut self, directory: &Path) -> io::Result<()> {
        let Some(mut table) = Table::load(directory)? else {
            self.fight_state.doors = Arc::default();
            return Ok(());
        };
        // The provenance and the held list are for tools: a session does not carry them.
        table.note.clear();
        table.source.clear();
        table.held.clear();
        table
            .scenes
            .iter_mut()
            .for_each(|scene| scene.source.clear());
        table
            .entrances
            .iter_mut()
            .for_each(|row| row.source.clear());
        let refuse = |id: &str, why: &str| invalid(&format!("{FILE}: {id}: {why}"));
        let mut guids = self.claimed_guids();
        guids.extend(self.field_sim.enemies.iter().map(|f| f.actor.guid));
        let mut doors = Doors::default();
        let fights = self.fight_state.fights.clone();
        // Fight files that name a scene take its sky.
        for dungeon in &mut Arc::make_mut(&mut self.content).dungeons {
            if fights.get(&dungeon.id).is_some_and(|f| !f.entry.battle_row) {
                dungeon.zone = table.complete(dungeon.zone.clone());
            }
        }
        let file_battles = self.battles.len();
        for row in &table.entrances {
            if !guids.insert(row.guid) {
                return Err(refuse(&row.id, "its guid is taken"));
            }
            let spawner: Actor = serde_json::from_value(json!({
                "guid": row.guid,
                "definition_id": 0,
                "model_id": row.model_id,
                "name_id": row.name_id,
                "name": row.name,
                "texture_alias": "",
                "position": row.at,
                "max_health": 1,
                "damage": 0,
                "boss": false,
                "scale": row.scale,
            }))
            .map_err(|e| refuse(&row.id, &e.to_string()))?;
            let marker = MarkerSetting::default().resolve(table.combat_marker, row.name_id)?;
            let scene = row.scene.as_deref().and_then(|name| table.scene(name));
            let base = self.battles[..file_battles]
                .iter()
                .find(|b| Some(b.id) == row.fight)
                .cloned();
            let fight = row.fight.and_then(|id| fights.get(&id));
            let dungeon = fight.filter(|f| !f.entry.battle_row).and_then(|f| {
                (self.content.dungeons.iter()).find(|d| d.id == f.id && d.guide.is_none())
            });
            // The id it goes by: a fight file's own, else this entrance's.
            let id = match (&base, dungeon) {
                (None, Some(dungeon)) => dungeon.id,
                _ => row.encounter_id,
            };
            let own_id = base.is_some() || dungeon.is_none();
            if self.battles.iter().any(|b| b.id == id)
                || (own_id && self.content.dungeons.iter().any(|d| d.id == id))
            {
                return Err(refuse(&row.id, "its encounter id is taken"));
            }
            let battle = if let Some(base) = base {
                // A `field_battles.json` battle under this entrance's own id.
                let mut enemies = base.enemies.clone();
                let moved = scene.filter(|_| row.enter_scene);
                if let Some(scene) = moved {
                    set_down(&mut enemies, scene.stage);
                }
                for (n, actor) in enemies.iter_mut().enumerate() {
                    actor.guid = ENEMY_GUIDS | (u64::from(row.encounter_id) << 8) | (n as u64 + 1);
                    if !guids.insert(actor.guid) {
                        return Err(refuse(&row.id, "an enemy guid is taken"));
                    }
                }
                let zone = moved.map_or_else(|| base.zone.clone(), Scene::zone);
                let Some(arena) = (self.content.dungeons.iter()).find(|d| d.id == base.id) else {
                    return Err(refuse(&row.id, "its battle has no arena row"));
                };
                let arena = Dungeon {
                    id,
                    zone: zone.clone(),
                    enemies: enemies.clone(),
                    ..arena.clone()
                };
                Arc::make_mut(&mut self.content).dungeons.push(arena);
                doors.counts_as.insert(id, base.id);
                FieldBattle {
                    id,
                    spawner,
                    marker,
                    zone,
                    enemies,
                    ..base
                }
            } else {
                // A fight file's starter, or a closed entrance (no arena row: nothing can
                // begin). The card is the fight's.
                if let Some(old) = row.counts_as.filter(|_| dungeon.is_some()) {
                    doors.counts_as.insert(id, old);
                }
                FieldBattle {
                    id,
                    title_id: dungeon.map_or(0, |d| d.title_id),
                    description_id: dungeon.map_or(0, |d| d.description_id),
                    objective_id: dungeon.map_or(0, |d| d.objective_id),
                    difficulty: fight.map_or(1, |f| f.difficulty),
                    required_combat_level: fight.map_or(0, |f| f.entry.min_combat_level),
                    spawner,
                    combat_marker: MarkerSetting::default(),
                    marker: marker.filter(|_| dungeon.is_some()),
                    zone: dungeon.map_or_else(|| self.content.world.clone(), |d| d.zone.clone()),
                    icon_id: dungeon.map_or(-1, |d| d.icon_id),
                    prizes: dungeon.map_or_else(Vec::new, |d| d.prizes.clone()),
                    drops: (self.fight_state.drops.get(&id).cloned()).unwrap_or_default(),
                    nameplate_image_id: default_battle_icon(),
                    enemies: dungeon.map_or_else(Vec::new, |d| d.enemies.clone()),
                    reward_coins: dungeon.map_or(0, |d| d.reward_coins),
                    source: Value::Null,
                }
            };
            doors.rows.insert(
                row.guid,
                Door {
                    encounter: battle.id,
                    open: !battle.enemies.is_empty(),
                    heading: row.heading,
                    scale: row.scale,
                    fixed: row.roam <= 0.0,
                },
            );
            self.battles.push(battle);
        }
        doors.table = Some(table);
        self.fight_state.doors = Arc::new(doors);
        Ok(())
    }

    /// The placed entrance table (without its provenance and held rows; `Table::load` has
    /// those).
    pub fn entrances(&self) -> Option<&Table> {
        self.fight_state.doors.table.as_ref()
    }

    /// The encounter an entrance's actor starts and whether it is open; `None` for any other
    /// guid.
    pub fn entrance(&self, guid: u64) -> Option<(u32, bool)> {
        let door = self.fight_state.doors.rows.get(&guid)?;
        Some((door.encounter, door.open))
    }

    /// Heading and scale of an entrance's actor, for the world's own view of it.
    pub fn entrance_pose(&self, guid: u64) -> Option<(f32, f32)> {
        self.fight_state.doors.pose(guid)
    }

    /// A click on an entrance that does not open the waiting card: the closed answer, or the
    /// "Fight" menu of the straight launch. `None` shows the card.
    pub(in crate::encounter) fn entrance_click(&mut self, guid: u64) -> Option<EncounterOutcome> {
        let doors = self.fight_state.doors.clone();
        let (door, table) = (doors.rows.get(&guid)?, doors.table.as_ref()?);
        let instance = self.state.next_instance_id;
        let reply = if !door.open {
            launch_failed(door.encounter, instance, table.closed_text_id)
        } else if table.launch == Launch::Direct {
            let name = (self.battles.iter().find(|b| b.spawner.guid == guid))
                .map_or("", |b| b.spawner.name.as_str());
            self.pending_battle = Some((door.encounter, instance));
            fight_menu(guid, name, table.fight_menu, door.encounter)
        } else {
            return None;
        };
        Some(EncounterOutcome {
            replies: vec![reply],
            ..Default::default()
        })
    }

    /// Whether `inner` is the client's own pick of the "Fight" entry (26/10) on the entrance
    /// whose menu is open: the straight launch's Go.
    /// PB2: `battle` is an entrance's (its Go switches a non-combat job), not a battle marker's.
    pub fn is_entrance_battle(&self, battle: u32) -> bool {
        (self.fight_state.doors.rows.values()).any(|door| door.encounter == battle)
    }

    pub fn entrance_picked(&self, inner: &[u8]) -> bool {
        let doors = &self.fight_state.doors;
        let Some(table) = doors.table.as_ref().filter(|t| t.launch == Launch::Direct) else {
            return false;
        };
        inner.len() == 16
            && inner.starts_with(&[26, 0, 10, 0])
            && crate::le::i32(inner, 12) == Some(table.fight_menu.event_id)
            && crate::le::u64(inner, 4)
                .and_then(|guid| doors.rows.get(&guid))
                .is_some_and(|door| self.pending_battle.map(|(id, _)| id) == Some(door.encounter))
    }

    /// The 41/120 for a job the entry switched, when the start card open now is an
    /// entrance's; `None` for the battle markers of `field_battles.json`, which keep their
    /// refusal.
    pub fn entrance_job_notice(&self, player: u64) -> Option<Vec<u8>> {
        let doors = &self.fight_state.doors;
        let (id, instance) = self.pending_battle?;
        let ours = doors.rows.values().any(|door| door.encounter == id);
        let table = doors.table.as_ref().filter(|_| ours)?;
        Some(participant_message(
            id,
            instance,
            table.job_switch_text_id,
            player,
        ))
    }
}
