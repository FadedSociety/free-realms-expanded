//! Native soccer entry through the activity portal and the shared minigame lifecycle.
//!
//! Recovered client facts reused here:
//!
//! * The activity portal row is played with `167/1/2` and the minigame records
//!   `42`/`43`/`44` are the three Soccer activities (type 12) from the shipped
//!   `MiniGameData` rows; category 22 and `Soccer_Briarwood.dds` come from the
//!   client `Resources/ActivityCategories.txt`.
//! * Family 76 input layouts and the shared `39/*` lifecycle are owned by the
//!   already-reviewed `soccer`, `soccer_match` and `soccer_service` modules.
//! * The scene name `sg_soccer` and the opcode-31 entry frame are the captured
//!   candidate in `soccer_service`; its sky, geometry and match values stay
//!   reconstructed (R). The content's pitch centre, bounds and player spawn
//!   follow the live capture cited in `soccer.json`.
//!
//! Everything in this module is the gateway adapter around those modules:
//! content admission, roster setup, packet wrapping, auto-resume/second-half
//! transitions, and settling the shared result's grant on the caller's live
//! reward state. The original soccer launch/result exchange is not recovered,
//! so the shared lifecycle remains a reconstructed compatibility adapter (R).

use crate::activity_portal::Activity;
use crate::character_store::CharacterState;
use crate::minigame::RunState;
use crate::rewards::{RewardDisplay, RewardGrant, RewardState};
use crate::soccer::SoccerPacket;
use crate::soccer_match::{
    Authority, FieldBounds, HalfPolicy, Intent, MatchConfig, MatchPhase, PhysicsConfig,
};
use crate::soccer_service::{
    AdapterPolicy, ServiceEvent, SoccerService, entry_scene, wire_from_inner,
};
use crate::zone_transition::ZoneTransition;
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    fs,
    io::{self, Read},
    path::Path,
};

/// Bundled/content filename; loaded from the gameplay directory when present,
/// otherwise from the server's bundled `data/content`.
pub const CONTENT_FILE: &str = "soccer.json";
pub const SCHEMA_VERSION: u32 = 1;
/// The Soccer Star job (`jobs.json` profile 52): the match's job XP goes here.
pub const SOCCER_STAR_PROFILE: i32 = 52;
const MAX_FILE: u64 = 1 << 20;
const MAX_GAMES: usize = 32;

/// One soccer MiniGameData row (type 12): a Games window row (42/43/44) or a quest-only row.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Game {
    /// MiniGameData row; the id a settled win credits to quests.
    pub id: u32,
    pub name_id: u32,
    pub description_id: u32,
    pub icon_id: u32,
    pub difficulty: u32,
    /// R level floor. The original Soccer Star gate is not recovered.
    #[serde(default)]
    pub required_level: u32,
    /// R encounter identity used by the shared `RunMode::Encounter` start.
    pub encounter_id: i32,
    /// L932: no Games window tile. A match credits this row only when a quest's open
    /// `finish_minigame` objective names it (see [`Content::credited_row`]).
    #[serde(default)]
    pub quest_only: bool,
    /// L932 (R): a win counts for the quest only when the other team scored at most this many
    /// goals. Tackles, steals, dodges and knockdowns are not measured, so rows whose bonus goals
    /// ask for them credit on the win alone.
    #[serde(default)]
    pub max_conceded: Option<u32>,
    /// Which of `arenas` the row plays on; with none, the captured pitch.
    #[serde(default)]
    pub arena: Option<String>,
    /// Which of `teams` the row plays against; with none, the captured Snowspurs.
    #[serde(default)]
    pub opponent: Option<String>,
    /// L952: a client Lua `Table.Method` the match runs at its first kickoff (BaseUi 47/7), such
    /// as the Soccer Tutorial row's `SoccerTutorial.StartSoccerTut`, which opens the client's own
    /// tutorial pages over the match.
    #[serde(default)]
    pub client_script: Option<String>,
    /// Evidence for the row's win condition; documentation only.
    #[serde(default, rename = "source")]
    _source: String,
}

impl Game {
    /// The offer card's name, icon, description and difficulty.
    fn offer_card(&self) -> io::Result<(i32, i32, i32, i32)> {
        let number = |value: u32| i32::try_from(value).map_err(|_| invalid("soccer row number"));
        Ok((
            number(self.name_id)?,
            number(self.icon_id)?,
            number(self.description_id)?,
            number(self.difficulty)?,
        ))
    }

    /// L932: whether a won match counts for this row when the other team scored `conceded`.
    pub fn counts(&self, conceded: u32) -> bool {
        self.max_conceded.is_none_or(|max| conceded <= max)
    }
}

/// One pitch. All three share a mesh and a rotation, so each is the captured pitch moved on the
/// map: `delta` is x and z from `sg_soccer`, measured between the goal props.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct Arena {
    zone: String,
    sky: String,
    delta: [f32; 2],
}

/// An opposing team: its name on the board and its four players, by slot.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Team {
    name_id: u32,
    bots: Vec<BotBody>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct BoundsSettings {
    /// World pitch centre; older files without it keep the origin.
    #[serde(default)]
    centre: [f32; 3],
    half_x: f32,
    half_z: f32,
    goal_half_width: f32,
    goal_height: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct PhysicsSettings {
    gravity: f32,
    max_position_step: f32,
    max_speed: f32,
    control_radius: f32,
    max_kick_power: f32,
    pass_radius: f32,
    max_tick: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum HalfPolicySetting {
    SwitchSides,
    KeepSides,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct AdapterSettings {
    max_power: f32,
    charge_floor: f32,
    min_direction_norm: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct MatchSettings {
    bounds: BoundsSettings,
    physics: PhysicsSettings,
    half_seconds: f32,
    kickoff_spawn: [f32; 3],
    half_policy: HalfPolicySetting,
    adapter: AdapterSettings,
    entry_position: [f32; 4],
    entry_rotation: [f32; 4],
    /// L932: the server bots play (chase, dribble, pass, shoot, tackle); absent = passive bots.
    #[serde(default)]
    bot_ai: bool,
    /// L955: the ball's tuning; absent = the defaults fitted to the live ball.
    #[serde(default)]
    ball: crate::soccer_ball::BallTuning,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct RewardSettings {
    coins: u32,
    /// R job XP; empty unless an authored reward references an unlocked profile.
    #[serde(default)]
    job_xp: BTreeMap<i32, u32>,
    /// (R) The share of this a finished, lost match pays. A forfeit pays nothing.
    #[serde(default)]
    loss_percent: u32,
}

/// L932: the world gates that start a match: the client's "Left/Right Side Soccer Gate" NPCs
/// (Global.Text 416257, 416258; the quests say "Click the soccer gate").
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct GateSettings {
    /// NPC content keys (`npcs/*.json` rows, drawn by `npc_spawns_extra.json`).
    npcs: Vec<u64>,
    /// The Games window rows the gate's menu lists; an open quest objective's row still wins.
    games: Vec<u32>,
}

/// L940: the roster with each bot's body: its AddNpc right before its 76/3, then its 35/9 and
/// 35/20 after the roster, the order the live server sent (`packets_soccer_minigame.pcapng`).
/// Without the actor the client builds the soccer player but draws no body (native 2026-11-03).
fn with_bot_bodies(
    service: &SoccerService,
    bodies: &BTreeMap<(u32, u32), (i32, i32)>,
    human: u64,
    roster: &[ServiceEvent],
) -> io::Result<Vec<Vec<u8>>> {
    let mut out = Vec::with_capacity(roster.len() + bodies.len() * 3);
    let mut after = Vec::with_capacity(bodies.len() * 2);
    for event in roster {
        // L943: the live server sent the human only its 76/6; the client registers its own player,
        // and a second registration bound another soccer player to the human's body.
        if matches!(event, ServiceEvent::Registered { player, .. } if *player == human) {
            continue;
        }
        if let ServiceEvent::Registered { player, .. } = event
            && *player != human
            && let Some(state) = service.player(*player)
            && let Some(&(name, model)) = bodies.get(&(state.team, state.slot))
        {
            // Passive bots (no `bot_ai`) are never lined up; they stand at the kickoff spot.
            let position = state
                .position
                .unwrap_or(service.authority().config().kickoff_spawn);
            out.push(bot_actor(*player, name, model, position)?);
            let mut health = [35u16.to_le_bytes(), 9u16.to_le_bytes()].concat();
            health.extend(player.to_le_bytes());
            // (R) read as health, max and a third word; the live server sent 100 for all three.
            for value in [100u32; 3] {
                health.extend(value.to_le_bytes());
            }
            after.push(health);
            after.push(crate::kart_gateway::player_status(*player, 1));
        }
        out.push(event.inner());
    }
    out.append(&mut after);
    Ok(out)
}

/// L940: a bot's 35/2 with the live server's values; the fields not set here were zero.
fn bot_actor(guid: u64, name: i32, model: i32, position: [f32; 3]) -> io::Result<Vec<u8>> {
    let actor = crate::startup_world::NpcSpawn {
        actor_id: guid,
        name_string_id: name,
        model_id: model,
        chat_bubble_foreground_color: 408_679,
        chat_bubble_background_color: 13_951_728,
        chat_bubble_size: 1,
        scale: 1.0,
        position: [position[0], position[1], position[2], 1.0],
        rotation: [0.0, 0.0, 1.0, 0.0],
        animation_id: 1,
        interact_range: 100,
        walk_animation_id: -1,
        run_animation_id: -1,
        stand_animation_id: -1,
        sub_text_name_string_id: -1,
        area_definition_id: -1,
        is_interactable: 1,
        movement_type: 2,
        active_profile: 8,
        // (R) unnamed; the live value.
        unknown_field_68: 3442,
        ..Default::default()
    };
    crate::world::unframe_tunnel(&crate::startup_world::build_add_npc(&actor))
}

/// 76/7's world floats: the box the client normalises against (`00b7b4c0`, `00b75660`) and the
/// ball's spot.
const CONFIG_X: [usize; 3] = [0, 8, 157];
const CONFIG_Z: [usize; 3] = [4, 12, 165];

fn move_config(config: &mut [u8], [dx, dz]: [f32; 2]) -> io::Result<()> {
    for (offsets, delta) in [(CONFIG_X, dx), (CONFIG_Z, dz)] {
        for at in offsets {
            let bad = || invalid("soccer setup match config");
            let word = config.get_mut(at..at + 4).ok_or_else(bad)?;
            let value = <[u8; 4]>::try_from(&*word).map_err(|_| bad())?;
            word.copy_from_slice(&(f32::from_le_bytes(value) + delta).to_le_bytes());
        }
    }
    Ok(())
}

/// L946: how long a goal's celebration holds the next kickoff (live: step 17 to 18, 3.1 s) and
/// the half-time board holds the second half (R).
const GOAL_CELEBRATION_MS: u64 = 3_100;
const HALF_TIME_MS: u64 = 10_000;

/// L940: the player status word while a soccer match runs: bit 12 (soccer) plus the normal bit 0.
const SOCCER_STATUS: u32 = 0x1001;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Content {
    schema_version: u32,
    #[serde(default, rename = "source")]
    _source: String,
    games: Vec<Game>,
    #[serde(default)]
    gates: Option<GateSettings>,
    match_config: MatchSettings,
    /// The pitches by name (see [`Game::arena`]).
    #[serde(default)]
    arenas: BTreeMap<String, Arena>,
    /// The pitch [`Self::for_game`] moved this copy to.
    #[serde(skip)]
    moved_to: Option<Arena>,
    /// The opposing teams by name (see [`Game::opponent`]).
    #[serde(default)]
    teams: BTreeMap<String, Team>,
    /// The name [`Self::for_game`] gave this copy's other team.
    #[serde(skip)]
    opponent_name: Option<u32>,
    reward: RewardSettings,
    /// L940: the live server's match setup records (see `soccer_service::Setup`).
    #[serde(default)]
    setup: Option<SetupSettings>,
    /// L942: the live server's power-ups (see `soccer_pickups`).
    #[serde(default)]
    pickups: Option<crate::soccer_pickups::Settings>,
}

/// L940: `soccer.json` `setup`, as captured: 76/7 and 76/8 as hex, 76/21 floats and 76/19 words per role.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SetupSettings {
    #[serde(default, rename = "source")]
    _source: String,
    match_config_hex: String,
    teams_hex: String,
    roles: std::collections::BTreeMap<String, RoleSetup>,
    /// Each bot's name and model by team and slot, as the live server spawned them.
    #[serde(default)]
    bots: Vec<BotBody>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BotBody {
    #[serde(default, rename = "name")]
    _name: String,
    team: u32,
    slot: u32,
    name_string_id: i32,
    model_id: i32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RoleSetup {
    one_time: Vec<f32>,
    variable: Vec<u32>,
}

const SETUP_ROLES: [&str; 5] = ["human", "team_0", "team_1", "keeper_0", "keeper_1"];

impl Content {
    /// L940: the setup records, checked: every role present with 42 finite floats and 9 words.
    pub fn setup(&self) -> io::Result<Option<crate::soccer_service::Setup>> {
        let Some(setup) = &self.setup else { return Ok(None) };
        let decode = crate::startup_snapshot::decode_hex;
        let mut roles = std::collections::BTreeMap::new();
        for role in SETUP_ROLES {
            let entry = setup.roles.get(role).ok_or_else(|| invalid("soccer setup role missing"))?;
            let stats: [f32; 42] = entry.one_time.as_slice().try_into().map_err(|_| invalid("soccer setup floats"))?;
            let words: [u32; 9] = entry.variable.as_slice().try_into().map_err(|_| invalid("soccer setup words"))?;
            if !stats.iter().all(|value| value.is_finite()) {
                return Err(invalid("soccer setup floats"));
            }
            roles.insert(role.to_owned(), (stats, words));
        }
        let mut seats = std::collections::BTreeSet::new();
        for bot in &setup.bots {
            if bot.team > 1
                || bot.slot >= crate::soccer_match::TEAM_SLOT_COUNT
                || !seats.insert((bot.team, bot.slot))
            {
                return Err(invalid("soccer setup bot seat"));
            }
        }
        let mut match_config = decode(&setup.match_config_hex)?;
        if let Some(arena) = &self.moved_to {
            move_config(&mut match_config, arena.delta)?;
        }
        let mut teams = decode(&setup.teams_hex)?;
        if let Some(name) = self.opponent_name {
            // 76/8 ends with the two teams' name ids and one zero byte.
            let at = teams.len().checked_sub(5).ok_or_else(|| invalid("soccer setup teams"))?;
            teams[at..at + 4].copy_from_slice(&name.to_le_bytes());
        }
        Ok(Some(crate::soccer_service::Setup {
            match_config,
            teams,
            roles,
        }))
    }

    /// L940: each bot's (name string, model) by (team, slot).
    fn bot_bodies(&self) -> BTreeMap<(u32, u32), (i32, i32)> {
        self.setup
            .iter()
            .flat_map(|setup| &setup.bots)
            .map(|bot| ((bot.team, bot.slot), (bot.name_string_id, bot.model_id)))
            .collect()
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        let mut bytes = Vec::new();
        fs::File::open(path)?
            .take(MAX_FILE + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_FILE {
            return Err(invalid("soccer content too large"));
        }
        let content: Self = serde_json::from_slice(&bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
        content.validate()?;
        Ok(content)
    }

    pub fn games(&self) -> &[Game] {
        &self.games
    }

    pub fn game(&self, id: u32) -> Option<&Game> {
        self.games.iter().find(|game| game.id == id)
    }

    /// L932: the NPCs whose menu offers a match (none without a `gates` block).
    pub fn gate_npcs(&self) -> &[u64] {
        self.gates.as_ref().map_or(&[], |gates| &gates.npcs)
    }

    /// The rows a gate's menu lists: game id, name id.
    pub fn gate_rows(&self) -> Vec<(u32, u32)> {
        (self.gates.iter().flat_map(|gates| &gates.games))
            .filter_map(|id| self.game(*id))
            .map(|game| (game.id, game.name_id))
            .collect()
    }

    /// Portal rows for this file; admission keeps its own per-player checks. A quest-only row has
    /// no Games window tile.
    pub fn portal_rows(&self) -> Vec<Activity> {
        self.games
            .iter()
            .filter(|game| !game.quest_only)
            .map(|game| Activity {
                id: game.id,
                name_id: game.name_id,
                description_id: game.description_id,
                icon_id: game.icon_id,
                difficulty: game.difficulty,
                required_level: game.required_level,
                kind: crate::activity_portal::Kind::Soccer,
            })
            .collect()
    }

    /// L932: the row a match entered as `entered` credits. A quest objective that is open and
    /// names the entered row wins, then the first open objective naming a quest-only row (the
    /// derby does the same per gate, `gameplay/derby_rows.rs`); with none, the entered row.
    pub fn credited_row(&self, entered: u32, open: &[u32]) -> u32 {
        if open.contains(&entered) {
            return entered;
        }
        let quest_only = |id: &&u32| self.game(**id).is_some_and(|game| game.quest_only);
        open.iter().find(quest_only).copied().unwrap_or(entered)
    }

    pub fn entry_scene(&self) -> io::Result<ZoneTransition> {
        let mut scene = entry_scene(
            self.match_config.entry_position,
            self.match_config.entry_rotation,
        )?;
        if let Some(arena) = &self.moved_to {
            scene.scene.clone_from(&arena.zone);
            scene.sky.clone_from(&arena.sky);
        }
        Ok(scene)
    }

    /// This content as `game` plays: on its pitch, against its team.
    fn for_game(&self, game: &Game) -> io::Result<Self> {
        let mut content = self.in_arena(game)?;
        let Some(name) = &game.opponent else {
            return Ok(content);
        };
        let team = (self.teams.get(name)).ok_or_else(|| invalid("soccer row's opponent is unknown"))?;
        if let Some(setup) = &mut content.setup {
            setup.bots.retain(|bot| bot.team != 1);
            setup.bots.extend(team.bots.iter().cloned());
        }
        content.opponent_name = Some(team.name_id);
        Ok(content)
    }

    /// Every world position moved to `game`'s pitch. Bots stand relative to the centre, so they
    /// follow.
    fn in_arena(&self, game: &Game) -> io::Result<Self> {
        let mut content = self.clone();
        let Some(name) = &game.arena else {
            return Ok(content);
        };
        let arena = (self.arenas.get(name)).ok_or_else(|| invalid("soccer row's arena is unknown"))?;
        let [dx, dz] = arena.delta;
        let settings = &mut content.match_config;
        for spot in [&mut settings.bounds.centre, &mut settings.kickoff_spawn] {
            spot[0] += dx;
            spot[2] += dz;
        }
        settings.entry_position[0] += dx;
        settings.entry_position[2] += dz;
        for spot in (content.pickups.iter_mut()).flat_map(|pickups| &mut pickups.spots) {
            spot[0] += dx;
            spot[1] += dz;
        }
        content.moved_to = Some(arena.clone());
        Ok(content)
    }

    /// The service validates the match configuration and adapter policy, and the entry must be a
    /// legal spot on this pitch.
    fn validate_pitch(&self) -> io::Result<()> {
        SoccerService::new(self.config(), self.policy())?;
        self.entry_scene()?;
        // The player spawns at the entry position, so the authority must take it
        // as a first position: on the pitch and one step from the kickoff.
        let [x, y, z, _] = self.match_config.entry_position;
        let mut probe = Authority::new(self.config())?;
        probe.register(1, 0, 0)?;
        probe.kickoff()?;
        let first = Intent::Position {
            player: 1,
            sequence: 1,
            position: [x, y, z],
        };
        probe
            .apply(1, 0.0, first)
            .map_err(|_| invalid("soccer entry position is not a first position on the pitch"))?;
        Ok(())
    }

    pub fn config(&self) -> MatchConfig {
        let settings = self.match_config;
        MatchConfig {
            bounds: FieldBounds {
                centre: settings.bounds.centre,
                half_x: settings.bounds.half_x,
                half_z: settings.bounds.half_z,
                goal_half_width: settings.bounds.goal_half_width,
                goal_height: settings.bounds.goal_height,
            },
            physics: PhysicsConfig {
                gravity: settings.physics.gravity,
                max_position_step: settings.physics.max_position_step,
                max_speed: settings.physics.max_speed,
                control_radius: settings.physics.control_radius,
                max_kick_power: settings.physics.max_kick_power,
                pass_radius: settings.physics.pass_radius,
                max_tick: settings.physics.max_tick,
            },
            half_seconds: settings.half_seconds,
            kickoff_spawn: settings.kickoff_spawn,
            half_policy: match settings.half_policy {
                HalfPolicySetting::SwitchSides => HalfPolicy::SwitchSides,
                HalfPolicySetting::KeepSides => HalfPolicy::KeepSides,
            },
            bot_ai: settings.bot_ai,
            ball: settings.ball,
        }
    }

    pub fn policy(&self) -> AdapterPolicy {
        AdapterPolicy {
            max_power: self.match_config.adapter.max_power,
            charge_floor: self.match_config.adapter.charge_floor,
            min_direction_norm: self.match_config.adapter.min_direction_norm,
        }
    }

    /// L927: `xp` is the Soccer Star's `activity_xp` per match; without one the file's `job_xp` pays.
    fn grant(&self, game: &Game, player_guid: u64, xp: Option<u32>) -> RewardGrant {
        RewardGrant {
            coins: self.reward.coins,
            items: Vec::new(),
            job_xp: xp.map_or_else(
                || self.reward.job_xp.clone(),
                |xp| BTreeMap::from([(SOCCER_STAR_PROFILE, xp)]),
            ),
            sc: 0,
            display: RewardDisplay {
                source_guid: 0,
                player_guid,
                reward_kind: 0,
                icon_id: i32::try_from(game.icon_id).unwrap_or(0),
                name_id: i32::try_from(game.name_id).unwrap_or(0),
                rows: Vec::new(),
            },
        }
    }

    fn validate(&self) -> io::Result<()> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(invalid("soccer content schema version is not supported"));
        }
        self.setup()?;
        if let Some(pickups) = &self.pickups {
            pickups.validate()?;
        }
        if self.games.is_empty() || self.games.len() > MAX_GAMES {
            return Err(invalid("soccer content needs 1..=32 games"));
        }
        for (index, game) in self.games.iter().enumerate() {
            if game.id == 0
                || game.id > i32::MAX as u32
                || game.encounter_id <= 0
                || self.games[..index].iter().any(|other| other.id == game.id)
                || (game.client_script.as_deref()).is_some_and(|s| !crate::base_ui::valid_script(s))
            {
                return Err(invalid("soccer game ids/encounters are invalid"));
            }
        }
        if let Some(gates) = &self.gates {
            let listed = |id: &u32| self.game(*id).is_some_and(|game| !game.quest_only);
            if !gates.games.iter().all(listed)
                || gates.games.is_empty()
                || gates.npcs.is_empty()
                || gates.npcs.contains(&0)
                || gates
                    .npcs
                    .iter()
                    .enumerate()
                    .any(|(i, npc)| gates.npcs[..i].contains(npc))
            {
                return Err(invalid("soccer gates need NPCs and a Games window row"));
            }
        }
        self.validate_pitch()?;
        for game in &self.games {
            self.for_game(game)?.validate_pitch()?;
        }
        if self.reward.coins > i32::MAX as u32
            || self.reward.loss_percent > 100
            || self
                .reward
                .job_xp
                .iter()
                .any(|(profile, xp)| *profile <= 0 || *xp == 0)
        {
            return Err(invalid("soccer reward is invalid"));
        }
        Ok(())
    }
}

/// One adapted match. It is session-only: `GameplayState` intentionally does
/// not persist it, and a reconnect releases the run without a grant.
#[derive(Clone)]
pub struct Session {
    service: SoccerService,
    game: Game,
    pub(crate) human: u64,
    pending: Vec<Vec<u8>>,
    /// L940: with the live setup the roster waits for the start (see `handle_lifecycle`).
    hold_roster: bool,
    offer: Vec<Vec<u8>>,
    /// The server-owned arena scene this match entered; the gateway stages it
    /// and reports it as the runtime zone while the session lives.
    scene: ZoneTransition,
    sequence: u64,
    last_tick_ms: Option<u64>,
    /// L946: when a held restart or second half may start (a goal's celebration, the half-time board).
    resume_at_ms: Option<u64>,
    /// True once the terminal score was settled; a settled match never pays
    /// again, and the flag is session-only like the match itself.
    settled: bool,
    /// L952: the row's client script, sent once with the first kickoff.
    script: Option<Vec<u8>>,
    return_zone: u32,
    return_position: [f32; 3],
    return_rotation: [f32; 3],
}

#[derive(Default)]
pub struct LifecycleResult {
    pub replies: Vec<Vec<u8>>,
    pub state: Option<RunState>,
}

/// One tick's staged output. `settlement_due` means full time was reached and
/// the caller must call [`Session::settle`] with its live reward state.
#[derive(Default)]
pub struct TickResult {
    pub replies: Vec<Vec<u8>>,
    pub reward_state: Option<RewardState>,
    pub changed: bool,
    pub finished: bool,
    pub settlement_due: bool,
    /// L932: the settled row's win condition held, so the caller may credit the quest event.
    pub credit: bool,
}

impl Session {
    /// Join the authenticated player, fill the solo roster with passive bots,
    /// and create the shared type-12 offer. The caller stages the scene change
    /// and sends [`Self::loaded_replies`] after the client's Loaded packet.
    /// No reward snapshot is taken here: the payout reads the caller's live
    /// inventory when the match settles.
    pub fn begin(
        content: &Content,
        activity_id: u32,
        character: &CharacterState,
        xp: Option<u32>,
    ) -> io::Result<Self> {
        let game = content
            .game(activity_id)
            .cloned()
            .ok_or_else(|| invalid("soccer activity is not configured"))?;
        let content = &content.for_game(&game)?;
        let scene = content.entry_scene()?;
        let mut service = SoccerService::new(content.config(), content.policy())?;
        service.set_loss_percent(content.reward.loss_percent);
        let hold_roster = match content.setup()? {
            Some(setup) => {
                service.set_setup(setup);
                true
            }
            None => false,
        };
        if let Some(pickups) = &content.pickups {
            service.set_pickups(pickups.clone())?;
        }
        let mut roster = service.join(character.guid, 0, 0)?;
        roster.extend(service.fill_bots_for_solo(character.guid)?);
        // L940: the live server posed the human at its spawn with the bots' line-up; without it the
        // client picks its own spot 8 m away and every pose it sends is refused (native 2026-11-03).
        let [x, y, z, _] = scene.position;
        roster.push(ServiceEvent::PlayerPos {
            player: character.guid,
            position: [x, y, z],
            facing: [0.0, 0.0],
            time: service.authority().server_time(),
            speed: 0.0,
            snap: true,
        });
        let pending = with_bot_bodies(&service, &content.bot_bodies(), character.guid, &roster)?;
        service.set_offer_card(game.offer_card()?);
        let offered = service.offer_minigame(
            character.guid,
            game.id as i32,
            game.encounter_id,
            content.grant(&game, character.guid, xp),
        )?;
        if !offered.accepted {
            return Err(invalid("soccer offer was refused"));
        }
        let script = (game.client_script.as_deref())
            .map(|script| crate::base_ui::execute_script(script, &[]))
            .transpose()?;
        Ok(Self {
            service,
            game,
            human: character.guid,
            pending,
            hold_roster,
            offer: offered.replies,
            scene,
            sequence: 0,
            last_tick_ms: None,
            resume_at_ms: None,
            settled: false,
            script,
            return_zone: character.zone_id,
            return_position: character.position,
            return_rotation: character.rotation,
        })
    }

    /// The 39/16 offer staged for the client after Loaded, with the roster unless it is held.
    pub fn loaded_replies(&mut self) -> Vec<Vec<u8>> {
        let mut replies = if self.hold_roster {
            Vec::new()
        } else {
            std::mem::take(&mut self.pending)
        };
        replies.append(&mut self.offer);
        replies
    }

    pub fn game(&self) -> &Game {
        &self.game
    }

    pub fn phase(&self) -> MatchPhase {
        self.service.phase()
    }

    pub fn score(&self) -> [u32; 2] {
        self.service.score()
    }

    pub fn run_state(&self) -> Option<RunState> {
        self.service.run_state(self.human)
    }

    /// The server-owned arena scene the gateway stages on entry.
    pub fn scene(&self) -> &ZoneTransition {
        &self.scene
    }

    /// L012: the character back at the pose it left for the match. A match belongs to one
    /// connection, so saves and reconnects use this pose, never the arena scene.
    pub fn return_character(&self, character: &CharacterState) -> CharacterState {
        CharacterState {
            zone_id: self.return_zone,
            position: self.return_position,
            rotation: self.return_rotation,
            ..character.clone()
        }
    }

    /// The shared `39/*`/`41/109`/`26/42` controls. A pre-win quit cancels the
    /// run; a terminal result close retires it without another grant.
    pub fn handle_lifecycle(
        &mut self,
        guid: u64,
        inner: &[u8],
        now_ms: u64,
    ) -> io::Result<LifecycleResult> {
        let mut service = self.service.clone();
        let change = service.handle_minigame_lifecycle(guid, inner)?;
        let mut result = LifecycleResult {
            replies: change.run.replies,
            state: change.run.state,
        };
        result
            .replies
            .extend(change.soccer_events.iter().map(ServiceEvent::inner));
        if result.state == Some(RunState::Starting) {
            let started = service.mark_started(guid)?;
            // L940: player status bit 12 (0x1000) puts the client in soccer: its status handler
            // (0x932d68) then runs the soccer start (sub_B7E2D0), which builds the local
            // "SoccerPlayerLocal". Without it the pitch stays empty and the HUD crashes on the setup.
            // L942: the live server first stowed whatever the player held (35/61 active wield type 0),
            // then sent the status.
            let mut unarmed = [35u16.to_le_bytes(), 61u16.to_le_bytes()].concat();
            unarmed.extend(guid.to_le_bytes());
            unarmed.extend(0u32.to_le_bytes());
            result.replies.push(unarmed);
            result
                .replies
                .push(crate::kart_gateway::player_status(guid, SOCCER_STATUS));
            // That soccer start also deletes every soccer player the client already has, so a held
            // roster goes after the status, where the live server sent it: after state step 4.
            let split = started
                .iter()
                .position(|event| {
                    matches!(event.packet(), SoccerPacket::UpdateGameState { words } if words[12] == 5)
                })
                .unwrap_or(0);
            result
                .replies
                .extend(started[..split].iter().map(ServiceEvent::inner));
            result.replies.append(&mut self.pending);
            result
                .replies
                .extend(started[split..].iter().map(ServiceEvent::inner));
            // L952: the row's client script goes with the first kickoff.
            if !started.is_empty() {
                result.replies.extend(self.script.take());
            }
            result.state = service.run_state(guid);
            if result.state == Some(RunState::Playing) {
                self.last_tick_ms = Some(now_ms);
            }
        }
        // L940: leaving the match clears the soccer status bit again.
        if matches!(result.state, Some(RunState::Cancelled | RunState::Closed)) {
            result.replies.push(crate::kart_gateway::player_status(guid, 1));
        }
        self.service = service;
        Ok(result)
    }

    /// One family-76 application body. `pass_target` is deliberately absent:
    /// the client payload does not establish one, so pass states stay refused
    /// until a server-verified target source exists.
    pub fn handle_client(
        &mut self,
        guid: u64,
        inner: &[u8],
        now_ms: u64,
    ) -> io::Result<Vec<Vec<u8>>> {
        let wire = wire_from_inner(inner)?;
        // L942: the client reports its own registration and non kick/pass states as it plays (the
        // live client did too). They are advisory: nothing changes and nothing is sent. Refusing
        // them ended the session, because the client bundles them with its poses and a refused
        // bundle child fails the whole message (native 2026-11-03).
        use crate::soccer_match::{ClientPacket, SLIDE_TACKLE_START, STEAL_TACKLE, client_packet};
        match client_packet(&wire, 0) {
            // L945: the player's own tackle is one of those states; the server runs it.
            Ok(ClientPacket::ReadOnly(SoccerPacket::UpdatePlayerState { player, state, .. }))
                if player == guid && matches!(state, STEAL_TACKLE | SLIDE_TACKLE_START) =>
            {
                let mut service = self.service.clone();
                let events = service.player_tackle(guid)?;
                self.service = service;
                return Ok(events.iter().map(ServiceEvent::inner).collect());
            }
            // L955: the Ctrl juke (4 Juke, 5 JukeWithBall, 6 Juke360WithBall) dodges tackles.
            Ok(ClientPacket::ReadOnly(SoccerPacket::UpdatePlayerState {
                player, state, ..
            })) if player == guid && (4..=6).contains(&state) => {
                let events = self.service.player_juke(guid);
                return Ok(events.iter().map(ServiceEvent::inner).collect());
            }
            Ok(ClientPacket::ReadOnly(_)) => return Ok(Vec::new()),
            _ => {}
        }
        let sequence = self
            .sequence
            .checked_add(1)
            .ok_or_else(|| invalid("soccer input sequence exhausted"))?;
        let mut service = self.service.clone();
        // L952: a refused input (a pose past the bounds, a stray packet around the whistle) leaves
        // the match as it was and is dropped. Refusing it ended the player's session (native
        // 2026-11-03), as the advisory states did before L942.
        let Ok(events) = service.handle_client(guid, now_ms as f32 / 1000.0, &wire, sequence, None)
        else {
            return Ok(Vec::new());
        };
        self.service = service;
        self.sequence = sequence;
        Ok(events.iter().map(ServiceEvent::inner).collect())
    }

    /// Advance the authoritative clock. Restart and interval transitions are
    /// automatic (R): the client has no recovered resume packet, and the shared
    /// service exposes those transitions only as server calls. Full time only
    /// reports `settlement_due`; the caller settles on its live reward state via
    /// [`Self::settle`] so no entry snapshot can replace later changes.
    pub fn tick(&mut self, now_ms: u64) -> io::Result<TickResult> {
        let mut result = TickResult::default();
        let mut service = self.service.clone();
        if service.cancelled() {
            return Ok(result);
        }
        let mut last_tick_ms = self.last_tick_ms;
        if service.phase() == MatchPhase::Running {
            let previous = self.last_tick_ms.unwrap_or(now_ms);
            let elapsed = now_ms.saturating_sub(previous).min(250);
            last_tick_ms = Some(now_ms);
            if elapsed > 0 {
                let events = service.tick(elapsed as f32 / 1000.0)?;
                result
                    .replies
                    .extend(events.iter().map(ServiceEvent::inner));
                result.changed |= !events.is_empty();
            }
        }
        // L946: a goal's celebration (3.1 s live, 17 to 18) and the half-time board (R) hold play.
        let phase = service.phase();
        let pause = match phase {
            MatchPhase::Restart if service.celebrating() => GOAL_CELEBRATION_MS,
            MatchPhase::HalfTime if service.has_setup() => HALF_TIME_MS,
            _ => 0,
        };
        let due = matches!(phase, MatchPhase::Restart | MatchPhase::HalfTime)
            && now_ms >= *self.resume_at_ms.get_or_insert(now_ms + pause);
        if due {
            self.resume_at_ms = None;
        }
        match phase {
            MatchPhase::Restart if due => {
                let events = service.start()?;
                result
                    .replies
                    .extend(events.iter().map(ServiceEvent::inner));
                result.replies.extend(self.script.take());
                result.changed |= !events.is_empty();
            }
            MatchPhase::HalfTime if due => {
                let events = service.second_half()?;
                result
                    .replies
                    .extend(events.iter().map(ServiceEvent::inner));
                result.changed |= !events.is_empty();
            }
            MatchPhase::Finished if !self.settled => {
                result.settlement_due = true;
            }
            _ => {}
        }
        self.service = service;
        self.last_tick_ms = last_tick_ms;
        Ok(result)
    }

    /// Settle the terminal score on the caller's live inventory and profiles.
    ///
    /// The win grant is applied on top of those live values, never to a
    /// snapshot captured at entry: a claim or trade made during the match stays
    /// in place. A loss pays its share, a tie is a loss, and a duplicate call is inert.
    pub fn settle(&mut self, guid: u64, live: &RewardState) -> io::Result<TickResult> {
        let mut result = TickResult::default();
        let mut service = self.service.clone();
        let outcome = service.finish_minigame(guid, live)?;
        if outcome.accepted {
            let [home, away] = service.score();
            result.credit = match service.session_team(guid) {
                Some((0, _)) => self.game.counts(away),
                Some(_) => self.game.counts(home),
                None => false,
            };
            self.settled = true;
            result.replies.extend(outcome.replies);
            result.replies.extend(outcome.reward_packets);
            result.reward_state = outcome.reward_state;
            result.changed = true;
            result.finished = true;
        }
        self.service = service;
        Ok(result)
    }
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// L927: a match pays the Soccer Star's `activity_xp` when there is one, else the file's
    /// `job_xp` (empty in the shipped file: no XP).
    /// A row's scene, entry, kickoff, ball and power-up spots all stand inside its 76/7 box.
    #[test]
    fn every_row_stands_on_its_own_pitch() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content/soccer.json");
        let content = Content::load(&path).unwrap();
        let mut zones = std::collections::BTreeSet::new();
        for game in &content.games {
            let moved = content.for_game(game).unwrap();
            let config = moved.setup().unwrap().unwrap().match_config;
            let float = |at: usize| f32::from_le_bytes(config[at..at + 4].try_into().unwrap());
            let inside = |x: f32, z: f32| {
                (float(0)..=float(8)).contains(&x) && (float(4)..=float(12)).contains(&z)
            };
            let scene = moved.entry_scene().unwrap();
            let kickoff = moved.match_config.kickoff_spawn;
            assert!(inside(scene.position[0], scene.position[2]), "{} entry", game.id);
            assert!(inside(kickoff[0], kickoff[2]), "{} kickoff", game.id);
            assert!(inside(float(157), float(165)), "{} ball", game.id);
            for spot in &moved.pickups.as_ref().unwrap().spots {
                assert!(inside(spot[0], spot[1]), "{} power-up", game.id);
            }
            zones.insert(scene.scene);
        }
        assert_eq!(zones.len(), 3, "the three pitches");
    }

    /// A row plays its named team: the board's name word and all four bodies change.
    #[test]
    fn a_row_plays_against_its_own_team() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content/soccer.json");
        let content = Content::load(&path).unwrap();
        let goal_stopper = content.game(323).unwrap();
        let moved = content.for_game(goal_stopper).unwrap();
        let teams = moved.setup().unwrap().unwrap().teams;
        let name = u32::from_le_bytes(teams[teams.len() - 5..teams.len() - 1].try_into().unwrap());
        assert_eq!(name, 411_067, "Goalbusters");
        let bodies = moved.bot_bodies();
        assert_eq!(bodies[&(1, 0)], (401_626, 1432), "Dimitri Fastcannon");
        assert_eq!(bodies[&(1, 3)], (401_627, 1433), "their goalie");
        assert_eq!(bodies[&(0, 1)], (401_726, 1415), "our side stays");
        for team in content.teams.values() {
            let slots: Vec<_> = team.bots.iter().map(|bot| (bot.team, bot.slot)).collect();
            assert_eq!(slots, [(1, 0), (1, 1), (1, 2), (1, 3)]);
        }
    }

    #[test]
    fn a_match_pays_the_activity_xp_or_the_files_job_xp() {
        let shipped = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content/soccer.json");
        let content = Content::load(&shipped).unwrap();
        let game = content.games.first().unwrap();
        assert!(content.grant(game, 1, None).job_xp.is_empty());
        assert_eq!(
            content.grant(game, 1, Some(72)).job_xp,
            BTreeMap::from([(SOCCER_STAR_PROFILE, 72)])
        );
        assert_eq!(content.grant(game, 1, Some(72)).coins, content.reward.coins);
    }

    fn shipped() -> Content {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content/soccer.json");
        Content::load(&path).unwrap()
    }

    /// L932: the Soccer Star quest rows are loaded, sit outside the Games window and each ask for
    /// the win alone, except Goal Stopper, which also asks the other team to stay under 5 goals.
    #[test]
    fn quest_rows_are_unlisted_and_goal_stopper_limits_the_other_team() {
        let content = shipped();
        let ids: Vec<u32> = content.games().iter().map(|game| game.id).collect();
        assert_eq!(
            ids,
            [
                42, 43, 44, 1030, 322, 323, 324, 325, 326, 327, 328, 329, 330, 331, 332, 333
            ]
        );
        let listed: Vec<u32> = content.portal_rows().iter().map(|row| row.id).collect();
        // L952: the Soccer Tutorial (1030) is a Games window row too.
        assert_eq!(
            listed,
            [42, 43, 44, 1030],
            "quest rows have no Games window tile"
        );
        for game in content.games() {
            assert!(game.counts(4), "row {} counts 4 conceded", game.id);
            assert_eq!(game.counts(5), game.id != 323, "row {} at 5", game.id);
        }
    }

    /// L932: an open objective on the entered row wins, then the first open quest-only row;
    /// a closed or non-soccer objective leaves the entered row.
    #[test]
    fn the_credited_row_follows_the_open_objective() {
        let content = shipped();
        assert_eq!(content.credited_row(43, &[]), 43);
        assert_eq!(content.credited_row(43, &[326]), 326, "a quest-only row");
        assert_eq!(content.credited_row(43, &[333, 326]), 333, "first open row");
        assert_eq!(content.credited_row(43, &[326, 43]), 43, "the entered row");
        assert_eq!(content.credited_row(43, &[44]), 43, "another tile's row");
        assert_eq!(content.credited_row(43, &[39, 604]), 43, "not soccer rows");
    }
}
