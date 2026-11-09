//! L937: in-world timed challenges, the client's minigame TYPE 1 ("SAMPLE").
//!
//! An NPC offers the challenge from its menu; the client shows the start screen, the HUD (the
//! countdown, the goal tracker, the star counter, the target arrow) and the end screen as it is
//! told, so what is out there, what the player touched and whether time ran out is decided here.
//! This module is the pure part: the content (`data/content/world_challenges.json`), the layout
//! of the things around the giver and one run's rules (`Session`). The server glue that owns the
//! 39/x lifecycle, the rewards and the quest credit is `gameplay/world_challenge.rs`.
//!
//! Adapted from the Sanctuary community server fork (BubbaJoeX): `Sanctuary.Game/Challenges/
//! ChallengeManager.cs`, `ChallengeLayout.cs` and `Resources/Challenges.json` (AGPL-3.0, credited
//! in NOTICE). Kept: the six kinds, the touch and click rules, the packet order. Changed: the
//! things are placed on the height grid instead of a navmesh, a run is ticked by the host's
//! clock, a race is won inside its one time limit (no "slow" half pay), and a loss pays nothing.
//! Every count, timer and reward is a reconstruction (R): only the names, descriptions, icons and
//! difficulties are the client's own (`MiniGameData.txt`).
//!
//! What a TYPE 2 mail-delivery run needs on top of this: the same lifecycle with the client's
//! "no results page" exit, the Postman ability bar (Mail Toss, Bone Toss), the route order and
//! mailbox targets; `Kind::Deliver` already visits fixed points in any order.

mod layout;
pub(crate) mod wire;

use crate::quest_journal::Pin;
use layout::{Field, Rng, flat_distance};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::Path,
    sync::Arc,
};

pub const CONTENT_FILE: &str = "world_challenges.json";
/// Instance ids start here, clear of every other activity's range.
pub const FIRST_INSTANCE_ID: i32 = 937_000_001;
/// Sent as the client's SCAVENGER_HUNT type for every row, SAMPLE rows included: type 2 ends with
/// no score screen (`SetMiniGameEndData`). Owner 2026-11-09: the results board is for dungeons and
/// proper minigames, not quests in the world, although a 2009 video shows it on Guest of Honor.
pub const TYPE_ID: i32 = 2;
/// The goal every game has first: "Win the Game" on the results board. It is never in the panel.
const WIN_GOAL_ID: u32 = 1;
const MAX_STAGES: usize = 8;
/// How long a used target that has an effect stays up (the fireworks of frame 128 last seconds).
const EFFECT_LINGER_MS: i32 = 8_000;
const MAX_BONUSES: usize = 4;
/// Briarwood green and red flags (client `Models.txt` 371, 374) for a race.
const GREEN_FLAG_MODEL: i32 = 371;
const RED_FLAG_MODEL: i32 = 374;
/// "WC" in the top bytes: the guids of a run's things (tag, instance, index).
const THING_GUID_TAG: u64 = 0x5743 << 48;
const MAX_FILE_BYTES: u64 = 1024 * 1024;
const MAX_ROWS: usize = 512;
const MAX_COUNT: u32 = 64;
const MAX_SECONDS: u32 = 3_600;
const MAX_PAYOUT: u32 = 100_000;
const MAX_DISTANCE: f32 = 1_000.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// Pick everything scattered about up, by clicking it or walking over it.
    Collect,
    /// Pick things up one at a time and bring each back to the giver.
    Carry,
    /// Go back to the giver.
    Return,
    /// Run through the flags in order to the last, beside the giver.
    Race,
    /// Catch the animals (or whoever) standing about, by clicking or running into them.
    Catch,
    /// Click each of the things put out; they stay where they are.
    Use,
    /// Go to each of the named people, in any order.
    Deliver,
}

/// Distances and layout bounds. (R) The fork's values, except `max_rise` (new: it stands in for
/// the navmesh's idea of reachable ground).
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tuning {
    /// The player must be this near the giver when pressing Go.
    pub start_range: f32,
    /// A run is lost this far beyond its reach.
    pub abandon_margin: f32,
    /// Walking over a thing takes it.
    pub touch_range: f32,
    pub flag_range: f32,
    pub return_range: f32,
    pub deliver_range: f32,
    /// A click counts from this far, and it is the client's X-prompt radius.
    pub click_range: f32,
    pub min_center_distance: f32,
    pub min_spacing: f32,
    /// A spot whose ground is higher or lower than the giver's by more is skipped.
    pub max_rise: f32,
    pub attempts_per_point: u32,
    /// Walking out from the giver is tried in steps of this many metres.
    pub walk_cell: f32,
    /// One step may climb or drop this much: more is a wall or a cliff the player cannot walk.
    pub walk_step: f32,
    /// A prop lower than this above the feet is stepped over.
    pub prop_clearance: f32,
    /// A prop inside this body height blocks the way, and a thing is not put inside one.
    pub body_height: f32,
}

impl Tuning {
    fn check(&self) -> io::Result<()> {
        let distances = [
            self.start_range,
            self.abandon_margin,
            self.touch_range,
            self.flag_range,
            self.return_range,
            self.deliver_range,
            self.click_range,
            self.min_center_distance,
            self.min_spacing,
            self.max_rise,
            self.walk_cell,
            self.walk_step,
            self.prop_clearance,
            self.body_height,
        ];
        if distances
            .iter()
            .any(|d| !d.is_finite() || *d <= 0.0 || *d > MAX_DISTANCE)
            || !(1..=100).contains(&self.attempts_per_point)
            || self.walk_cell < 0.5
        {
            return Err(invalid("world challenge tuning is out of bounds"));
        }
        Ok(())
    }
}

/// One goal of a run, in the order the player meets them (the panel grows as they open).
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stage {
    pub kind: Kind,
    /// Things to take, flags to run, or people to visit; 1 for `Return`.
    pub count: u32,
    /// How far from the giver the things go. Unused by `Deliver` and `Return`.
    #[serde(default)]
    pub radius: f32,
    /// The client model of what is taken, caught or used (a race uses the flags).
    #[serde(default)]
    pub model_id: i32,
    /// `Deliver`: the NPC content keys to visit.
    #[serde(default)]
    pub recipients: Vec<u64>,
    /// The goal's `Global.Text` id.
    pub name_id: u32,
    /// The text over each thing (`Global.Text`), like the live game's "Fireworks"; 0 shows none.
    #[serde(default)]
    pub label_id: i32,
    /// The composite effect when a `Use` target is used (the fireworks going off); 0 for none.
    #[serde(default)]
    pub effect_id: i32,
}

impl Stage {
    fn places(&self) -> bool {
        !matches!(self.kind, Kind::Deliver | Kind::Return)
    }

    fn is_bad(&self) -> bool {
        self.name_id == 0
            || !(1..=MAX_COUNT).contains(&self.count)
            || (self.places()
                && (!self.radius.is_finite() || self.radius <= 0.0 || self.radius > MAX_DISTANCE))
            || (matches!(
                self.kind,
                Kind::Collect | Kind::Carry | Kind::Catch | Kind::Use
            ) && self.model_id <= 0)
            || (self.kind == Kind::Race && self.count < 2)
            || (self.kind == Kind::Return && self.count != 1)
            || (self.kind == Kind::Deliver && self.recipients.len() != self.count as usize)
    }
}

/// A bonus goal: winning inside this many seconds.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bonus {
    pub name_id: u32,
    pub within_seconds: u32,
}

/// One challenge row.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Challenge {
    /// The client's `MiniGameData.txt` id.
    pub id: u32,
    pub title: String,
    /// The NPC content key whose menu offers it.
    pub giver: u64,
    pub name_id: u32,
    pub description_id: u32,
    pub icon_id: u32,
    pub difficulty: u32,
    /// The job it belongs to and pays; 0 pays whichever job is active.
    #[serde(default)]
    pub profile: i32,
    /// The deadline, in seconds. A race is won inside it.
    pub time_limit_seconds: u32,
    /// The text of the "Win the Game" line on the results board.
    pub win_goal_id: u32,
    pub stages: Vec<Stage>,
    #[serde(default)]
    pub bonuses: Vec<Bonus>,
    pub coins: u32,
    /// The XP when the job has no `activity_xp` rate.
    pub xp: u32,
    pub source: Vec<String>,
}

impl Challenge {
    fn check(&self) -> io::Result<()> {
        let bad = self.id == 0
            || self.title.trim().is_empty()
            || self.giver == 0
            || self.name_id == 0
            || self.description_id == 0
            || self.win_goal_id == 0
            || !(1..=5).contains(&self.difficulty)
            || self.profile < 0
            || !(1..=MAX_SECONDS).contains(&self.time_limit_seconds)
            || self.coins > MAX_PAYOUT
            || self.xp > MAX_PAYOUT
            || self.source.is_empty()
            || self.stages.is_empty()
            || self.stages.len() > MAX_STAGES
            || self.bonuses.len() > MAX_BONUSES
            || self.stages.iter().any(Stage::is_bad)
            || self
                .bonuses
                .iter()
                .any(|b| b.name_id == 0 || !(1..=MAX_SECONDS).contains(&b.within_seconds));
        if bad {
            return Err(invalid(format!("world challenge {} is malformed", self.id)));
        }
        Ok(())
    }

    fn stage_goal_id(stage: usize) -> u32 {
        2 + stage as u32
    }

    fn bonus_goal_id(&self, bonus: usize) -> u32 {
        2 + self.stages.len() as u32 + bonus as u32
    }

    /// The goal record of stage `index`, as the offer lists it and the panel shows it.
    fn stage_goal(&self, index: usize) -> crate::minigame::Goal {
        let stage = &self.stages[index];
        crate::minigame::Goal {
            id: Self::stage_goal_id(index),
            name_id: stage.name_id,
            description_id: stage.name_id,
            category: 2,
            bonus: false,
            stars: 0,
            coins: 0,
            total: stage.count,
            failure_condition: false,
            members_only: false,
        }
    }

    /// The 39/16 offer. The prize preview shows the coins and the stars; the goals are what the
    /// results board lists: the win, each stage, and the bonuses.
    pub fn run_spec(&self, stars: u32) -> crate::minigame::RunSpec {
        use crate::minigame::{Goal, RunMode, RunSpec};
        let mut spec = RunSpec::empty(self.id as i32, TYPE_ID, RunMode::InWorld);
        spec.name_id = self.name_id as i32;
        spec.icon_id = self.icon_id as i32;
        spec.description_id = self.description_id as i32;
        spec.difficulty = self.difficulty as i32;
        spec.profile_type = self.profile;
        spec.reward_bundles = self.bundles(None, stars);
        let plain = |id, name_id, category, bonus| Goal {
            id,
            name_id,
            description_id: name_id,
            category,
            bonus,
            stars: 0,
            coins: 0,
            total: 1,
            failure_condition: false,
            members_only: false,
        };
        let mut goals = vec![Goal {
            stars: stars as i32,
            coins: self.coins as i32,
            ..plain(WIN_GOAL_ID, self.win_goal_id, 1, false)
        }];
        goals.extend((0..self.stages.len()).map(|i| self.stage_goal(i)));
        goals.extend(
            (self.bonuses.iter().enumerate())
                .map(|(i, b)| plain(self.bonus_goal_id(i), b.name_id, 4, true)),
        );
        spec.objectives = goals;
        spec
    }

    /// Reward, member reward and preview bundles. `won` fills the first with what was paid.
    pub fn bundles(&self, paid: Option<(u32, u32)>, stars: u32) -> Vec<u8> {
        let (coins, paid_stars) = paid.unwrap_or_default();
        let mut bytes = wire::bundle(paid.is_some(), coins, paid_stars);
        bytes.extend(wire::bundle(false, 0, 0));
        bytes.extend(wire::bundle(true, self.coins, stars));
        bytes
    }
}

/// The loaded content.
#[derive(Clone, Debug)]
pub struct Challenges {
    tuning: Tuning,
    rows: BTreeMap<u32, Arc<Challenge>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    schema_version: u32,
    #[allow(dead_code)]
    #[serde(default)]
    source: serde_json::Value,
    tuning: Tuning,
    challenges: Vec<Challenge>,
}

impl Challenges {
    /// `None` when the content set has no `world_challenges.json`.
    pub fn load(directory: &Path) -> io::Result<Option<Self>> {
        let path = directory.join(CONTENT_FILE);
        if !path.is_file() {
            return Ok(None);
        }
        if fs::metadata(&path)?.len() > MAX_FILE_BYTES {
            return Err(invalid("world challenge file exceeds bounds"));
        }
        Self::parse(&fs::read(path)?).map(Some)
    }

    pub fn parse(bytes: &[u8]) -> io::Result<Self> {
        let file: File = serde_json::from_slice(bytes).map_err(invalid)?;
        if file.schema_version != 2 || file.challenges.len() > MAX_ROWS {
            return Err(invalid("unsupported world challenge file"));
        }
        file.tuning.check()?;
        let mut rows = BTreeMap::new();
        for row in file.challenges {
            row.check()?;
            let id = row.id;
            if rows.insert(id, Arc::new(row)).is_some() {
                return Err(invalid(format!("world challenge {id} is listed twice")));
            }
        }
        Ok(Self {
            tuning: file.tuning,
            rows,
        })
    }

    pub fn tuning(&self) -> &Tuning {
        &self.tuning
    }

    pub fn get(&self, id: u32) -> Option<&Arc<Challenge>> {
        self.rows.get(&id)
    }

    pub fn rows(&self) -> impl Iterator<Item = &Arc<Challenge>> {
        self.rows.values()
    }
}

/// What a run needs from its surroundings.
pub struct Setup<'a> {
    pub instance: i32,
    pub zone_id: u32,
    pub giver: [f32; 3],
    /// `Deliver`: where each recipient stands, with the key the pin names.
    pub recipients: Vec<(u64, [f32; 3])>,
    /// What a win pays in XP (resolved once, so the star counter matches the payout).
    pub xp: u32,
    pub now_ms: u64,
    pub seed: u64,
    pub ground: &'a dyn Fn(f32, f32) -> Option<f32>,
    /// Whether a placed prop (a tree, a wall) stands between two points on the ground.
    pub blocked: &'a dyn Fn([f32; 3], [f32; 3]) -> bool,
    /// A placed prop's floor (a deck, a cave) at (x, z), within a step of the height given.
    pub prop_floor: &'a dyn Fn(f32, f32, f32) -> Option<f32>,
    /// The zone has no height grid.
    pub flat: bool,
}

#[derive(Clone, Debug)]
struct Thing {
    /// The actor guid; for a `Deliver` stop, the recipient's NPC key.
    guid: u64,
    position: [f32; 3],
}

/// How a run ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    Won,
    TimedOut,
    /// The player left the zone or the challenge's area.
    Left,
}

/// Packets to send now, and the end if this step was the last.
#[derive(Debug, Default)]
pub struct Step {
    pub packets: Vec<Vec<u8>>,
    pub end: Option<End>,
}

/// One live run.
#[derive(Clone, Debug)]
pub struct Session {
    challenge: Arc<Challenge>,
    tuning: Tuning,
    instance: i32,
    zone_id: u32,
    giver: [f32; 3],
    reach: f32,
    xp: u32,
    started_ms: u64,
    ends_ms: u64,
    /// Which stage is open, and its things.
    stage: usize,
    things: Vec<Thing>,
    /// Every stage's things, laid out at the start so a stage cannot fail to fit half-way.
    laid: Vec<Vec<Thing>>,
    taken: BTreeSet<usize>,
    done: u32,
    total: u32,
    /// Everything done across the stages: the 39/18 score.
    score: u32,
    carrying: bool,
}

impl Session {
    /// Lay a run out, or `None` when the ground around the giver has no room for it.
    pub fn plan(challenge: &Arc<Challenge>, tuning: &Tuning, setup: Setup) -> Option<Self> {
        let mut rng = Rng::new(setup.seed);
        let widest = (challenge.stages.iter())
            .filter(|stage| stage.places())
            .fold(0.0_f32, |w, stage| w.max(stage.radius));
        let field = Field::new(
            setup.giver,
            setup.ground,
            setup.blocked,
            setup.prop_floor,
            setup.flat,
            tuning,
            widest,
        );
        let mut reach = 0.0_f32;
        let mut laid = Vec::with_capacity(challenge.stages.len());
        for (s, stage) in challenge.stages.iter().enumerate() {
            let tagged = |points: Vec<[f32; 3]>| -> Vec<(u64, [f32; 3])> {
                (points.into_iter().enumerate())
                    .map(|(i, p)| (guid(setup.instance, s, i), p))
                    .collect()
            };
            let positions = match stage.kind {
                Kind::Return => Vec::new(),
                Kind::Deliver => (setup.recipients.iter())
                    .filter(|(key, _)| stage.recipients.contains(key))
                    .copied()
                    .collect(),
                Kind::Race => tagged(layout::race_loop(&field, stage.radius, stage.count, &mut rng)),
                _ => tagged(layout::scatter(&field, stage.radius, stage.count, &mut rng)),
            };
            if stage.kind != Kind::Return && positions.len() < (stage.count.min(2) as usize).max(1)
            {
                return None;
            }
            // Someone at the far end of town is still part of the delivery.
            reach = (positions.iter())
                .map(|(_, p)| flat_distance(setup.giver, *p))
                .fold(reach.max(stage.radius), f32::max);
            laid.push(
                positions
                    .into_iter()
                    .map(|(guid, position)| Thing { guid, position })
                    .collect::<Vec<_>>(),
            );
        }
        let things = laid.first().cloned()?;
        let total = goal_total(&challenge.stages[0], &things);
        Some(Self {
            challenge: Arc::clone(challenge),
            tuning: *tuning,
            instance: setup.instance,
            zone_id: setup.zone_id,
            giver: setup.giver,
            reach,
            xp: setup.xp,
            started_ms: setup.now_ms,
            ends_ms: setup
                .now_ms
                .saturating_add(u64::from(challenge.time_limit_seconds) * 1_000),
            stage: 0,
            total,
            things,
            laid,
            taken: BTreeSet::new(),
            done: 0,
            score: 0,
            carrying: false,
        })
    }

    pub fn challenge(&self) -> &Arc<Challenge> {
        &self.challenge
    }

    /// Where every stage's things went, for the server log.
    pub fn laid_positions(&self) -> Vec<Vec<[f32; 3]>> {
        (self.laid.iter())
            .map(|things| things.iter().map(|t| t.position).collect())
            .collect()
    }

    /// Everything done so far, over all the stages.
    pub fn done(&self) -> u32 {
        self.score
    }

    pub fn total(&self) -> u32 {
        self.total
    }

    pub fn xp(&self) -> u32 {
        self.xp
    }

    fn current(&self) -> &Stage {
        &self.challenge.stages[self.stage]
    }

    fn goal_id(&self) -> i32 {
        Challenge::stage_goal_id(self.stage) as i32
    }

    fn spawns(&self) -> bool {
        self.current().places()
    }

    fn model_for(&self, index: usize) -> i32 {
        match self.current().kind {
            Kind::Race if index + 1 == self.things.len() => RED_FLAG_MODEL,
            Kind::Race => GREEN_FLAG_MODEL,
            _ => self.current().model_id,
        }
    }

    /// The seconds elapsed at `now_ms`, never past the deadline.
    pub fn elapsed_seconds(&self, now_ms: u64) -> u32 {
        (now_ms.min(self.ends_ms).saturating_sub(self.started_ms) / 1_000) as u32
    }

    /// The current stage's things going up, and its panel row.
    fn stage_packets(&self) -> Vec<Vec<u8>> {
        let mut packets = Vec::new();
        if self.spawns() {
            for (index, thing) in self.things.iter().enumerate() {
                let yaw = ((thing.guid & 0xff) as f32 * 0.618).fract() * std::f32::consts::TAU;
                packets.extend(wire::spawn(
                    thing.guid,
                    self.model_for(index),
                    self.current().label_id,
                    thing.position,
                    yaw,
                    self.tuning.click_range,
                ));
            }
        }
        // A rough spot can lay out fewer than the row asks for: the panel counts what is there.
        let mut goal = self.challenge.stage_goal(self.stage);
        goal.total = self.total;
        packets.push(crate::minigame::goal_row_packet(&goal));
        packets.push(crate::minigame::goal_activate_packet(&goal));
        packets
    }

    /// What goes out before the client's 39/17 (the HUD reads the tracker row when it opens) and
    /// what goes out after it.
    pub fn start_packets(&self) -> (Vec<Vec<u8>>, Vec<Vec<u8>>) {
        let before = self.stage_packets();
        let mut after = vec![wire::start_timer(
            self.challenge.time_limit_seconds as i32,
        )];
        after.extend(self.pin_next([self.giver[0], self.giver[1], self.giver[2]]));
        (before, after)
    }

    /// The target arrow to the next thing from `from`, or to the giver while carrying.
    fn pin_next(&self, from: [f32; 3]) -> Option<Vec<u8>> {
        let open = |i: usize| !self.taken.contains(&i);
        let target = if self.carrying || self.current().kind == Kind::Return {
            (0, self.giver)
        } else {
            let index = match self.current().kind {
                Kind::Race => (self.done as usize).min(self.things.len().checked_sub(1)?),
                _ => layout::nearest(
                    from,
                    &self.things.iter().map(|t| t.position).collect::<Vec<_>>(),
                    open,
                )?,
            };
            let thing = self.things.get(index)?;
            let guid = if self.spawns() { 0 } else { thing.guid };
            (guid, thing.position)
        };
        Some(Pin::encode(Some(&Pin {
            quest_id: self.challenge.id,
            objective_id: self.goal_id() as u32,
            guid: target.0,
            zone_id: self.zone_id,
            position: target.1,
        })))
    }

    /// Time, distance and walk-over checks for the player at `position`.
    pub fn tick(&mut self, zone_id: u32, position: [f32; 3], now_ms: u64) -> Step {
        if zone_id != self.zone_id
            || flat_distance(position, self.giver) > self.reach + self.tuning.abandon_margin
        {
            return Step::ended(End::Left);
        }
        if now_ms >= self.ends_ms {
            return Step::ended(End::TimedOut);
        }
        let t = self.tuning;
        let near = |thing: &Thing, range: f32| flat_distance(position, thing.position) <= range;
        match self.current().kind {
            Kind::Race => {
                let next = self.done as usize;
                if self.things.get(next).is_some_and(|f| near(f, t.flag_range)) {
                    return self.touch(next, position);
                }
            }
            Kind::Deliver => {
                let reached = (0..self.things.len())
                    .find(|i| !self.taken.contains(i) && near(&self.things[*i], t.deliver_range));
                if let Some(index) = reached {
                    self.taken.insert(index);
                    return self.progress(position);
                }
            }
            Kind::Return => {
                if flat_distance(position, self.giver) <= t.return_range {
                    return self.progress(position);
                }
            }
            Kind::Carry if self.carrying => {
                if flat_distance(position, self.giver) <= t.return_range {
                    self.carrying = false;
                    return self.progress(position);
                }
            }
            Kind::Use => {}
            _ => {
                let reached = (0..self.things.len())
                    .find(|i| !self.taken.contains(i) && near(&self.things[*i], t.touch_range));
                if let Some(index) = reached {
                    return self.touch(index, position);
                }
            }
        }
        Step::default()
    }

    /// A click on actor `guid`: `None` when it is not one of this run's things.
    pub fn click(&mut self, guid: u64, position: [f32; 3], now_ms: u64) -> Option<Step> {
        if !self.spawns() {
            return None;
        }
        let index = self.things.iter().position(|t| t.guid == guid)?;
        if now_ms >= self.ends_ms {
            return Some(Step::ended(End::TimedOut));
        }
        if self.taken.contains(&index)
            || flat_distance(position, self.things[index].position) > self.tuning.click_range
        {
            return Some(Step::default());
        }
        let kind = self.current().kind;
        if kind == Kind::Race && index != self.done as usize {
            return Some(Step::default());
        }
        if kind == Kind::Carry && self.carrying {
            return Some(Step::say("Bring back the one you have first."));
        }
        Some(self.touch(index, position))
    }

    fn touch(&mut self, index: usize, position: [f32; 3]) -> Step {
        self.taken.insert(index);
        let mut step = Step::default();
        let kind = self.current().kind;
        // A target stays where it is and cannot be used twice; everything else is taken.
        let thing = &self.things[index];
        if kind == Kind::Use {
            if self.current().effect_id != 0 {
                step.packets.push(wire::play_effect(
                    thing.guid,
                    self.current().effect_id,
                    thing.position,
                ));
            }
        } else {
            step.packets.push(wire::remove_with_sparkle(thing.guid));
        }
        if kind == Kind::Carry {
            self.carrying = true;
            step.packets.extend(self.pin_next(position));
            return step;
        }
        let next = self.progress(position);
        step.packets.extend(next.packets);
        step.end = next.end;
        step
    }

    /// One more done: the tracker, the goal, the star counter, the arrow to the next; the stage's
    /// end opens the next stage, and the last stage's end is the win.
    fn progress(&mut self, from: [f32; 3]) -> Step {
        self.done += 1;
        self.score += 1;
        let goal = self.goal_id();
        let mut step = Step::default();
        step.packets.push(wire::task_update(goal, self.done as i32));
        step.packets
            .push(wire::objective_update(goal, self.done as i32));
        step.packets.push(wire::stars_earned(
            self.instance,
            self.challenge.id as i32,
            self.stars_so_far() as i32,
        ));
        if self.done < self.total {
            step.packets.extend(self.pin_next(from));
            return step;
        }
        step.packets.push(wire::task_complete(goal));
        step.packets.push(wire::objective_complete(goal, 0));
        if self.stage + 1 == self.challenge.stages.len() {
            step.end = Some(End::Won);
            return step;
        }
        step.packets.extend(self.clear_things());
        self.stage += 1;
        self.things = self.laid[self.stage].clone();
        self.total = goal_total(self.current(), &self.things);
        self.done = 0;
        self.taken.clear();
        self.carrying = false;
        step.packets.extend(self.stage_packets());
        step.packets.extend(self.pin_next(from));
        step
    }

    /// The stars shown so far: the XP a win pays, in proportion over the stages.
    fn stars_so_far(&self) -> u32 {
        let stages = self.challenge.stages.len() as u64;
        let total = u64::from(self.total.max(1));
        let part = self.stage as u64 * total + u64::from(self.done);
        (u64::from(self.xp) * part / (stages * total)) as u32
    }

    /// Whatever of this stage's things is still standing.
    fn clear_things(&self) -> Vec<Vec<u8>> {
        if !self.spawns() {
            return Vec::new();
        }
        let used = self.current().kind == Kind::Use;
        // A target going off stays up while its effect plays.
        let linger = if self.current().effect_id != 0 { EFFECT_LINGER_MS } else { 0 };
        (self.things.iter().enumerate())
            // A target that was used is still standing.
            .filter(|(index, _)| used || !self.taken.contains(index))
            .map(|(_, thing)| wire::remove_after(thing.guid, linger))
            .collect()
    }

    /// The HUD teardown that closes a run, without a verdict: the timer, the arrow and whatever
    /// is still standing about.
    pub fn cleanup_packets(&self) -> Vec<Vec<u8>> {
        let mut packets = vec![wire::stop_timer(), Pin::encode(None)];
        packets.extend(self.clear_things());
        packets
    }

    /// [`Self::cleanup_packets`] and the verdict: the win goal, the bonuses made, or the goal that
    /// was open when it was lost.
    pub fn ending_packets(&self, won: bool, now_ms: u64) -> Vec<Vec<u8>> {
        let mut packets = self.cleanup_packets();
        if won {
            packets.push(wire::objective_complete(WIN_GOAL_ID as i32, self.xp as i32));
            let elapsed = self.elapsed_seconds(now_ms);
            for (i, bonus) in self.challenge.bonuses.iter().enumerate() {
                let id = self.challenge.bonus_goal_id(i) as i32;
                packets.push(if elapsed <= bonus.within_seconds {
                    wire::objective_complete(id, 0)
                } else {
                    wire::objective_fail(id)
                });
            }
        } else {
            let goal = self.goal_id();
            packets.push(wire::task_fail(goal));
            packets.push(wire::objective_fail(goal));
            packets.push(wire::objective_fail(WIN_GOAL_ID as i32));
        }
        packets
    }

    /// Whether the player is close enough to the giver to press Go.
    pub fn within_start_range(tuning: &Tuning, giver: [f32; 3], position: [f32; 3]) -> bool {
        flat_distance(giver, position) <= tuning.start_range
    }
}

fn goal_total(stage: &Stage, things: &[Thing]) -> u32 {
    if stage.kind == Kind::Return {
        1
    } else {
        things.len() as u32
    }
}

impl Step {
    fn ended(end: End) -> Self {
        Self {
            end: Some(end),
            ..Self::default()
        }
    }

    fn say(text: &str) -> Self {
        Self {
            packets: vec![crate::chat::system(text)],
            end: None,
        }
    }
}

fn guid(instance: i32, stage: usize, index: usize) -> u64 {
    THING_GUID_TAG
        | (u64::from(instance as u32) << 12)
        | ((stage as u64 & 0xf) << 8)
        | (index as u64 & 0xff)
}

fn invalid(error: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::minigame::MinigameRuns;

    const GIVER: [f32; 3] = [0.0, 10.0, 0.0];

    fn tuning() -> Tuning {
        Tuning {
            start_range: 30.0,
            abandon_margin: 60.0,
            touch_range: 2.2,
            flag_range: 4.0,
            return_range: 4.5,
            deliver_range: 3.5,
            click_range: 6.0,
            min_center_distance: 6.0,
            min_spacing: 4.0,
            max_rise: 8.0,
            attempts_per_point: 12,
            walk_cell: 2.0,
            walk_step: 1.4,
            prop_clearance: 0.5,
            body_height: 1.8,
        }
    }

    fn stage(kind: Kind, count: u32) -> Stage {
        Stage {
            kind,
            count,
            radius: 30.0,
            model_id: 5,
            recipients: Vec::new(),
            name_id: 1,
            label_id: 0,
            effect_id: 0,
        }
    }

    fn row(kind: Kind, count: u32) -> Arc<Challenge> {
        Arc::new(Challenge {
            id: 900,
            title: "Test".into(),
            giver: 1,
            name_id: 1,
            description_id: 1,
            icon_id: 1,
            difficulty: 1,
            profile: 0,
            time_limit_seconds: 60,
            win_goal_id: 1,
            stages: vec![stage(kind, count)],
            bonuses: Vec::new(),
            coins: 10,
            xp: 20,
            source: vec!["test".into()],
        })
    }

    fn plan_with(
        row: &Arc<Challenge>,
        recipients: Vec<(u64, [f32; 3])>,
        ground: &dyn Fn(f32, f32) -> Option<f32>,
        flat: bool,
    ) -> Option<Session> {
        Session::plan(
            row,
            &tuning(),
            Setup {
                instance: 7,
                zone_id: 0,
                giver: GIVER,
                recipients,
                xp: 20,
                now_ms: 1_000,
                seed: 42,
                ground,
                blocked: &|_, _| false,
                prop_floor: &|_, _, _| None,
                flat,
            },
        )
    }

    fn plan(row: &Arc<Challenge>) -> Session {
        plan_with(row, Vec::new(), &|_, _| None, true).expect("room for the run")
    }

    fn has(packets: &[Vec<u8>], prefix: &[u8]) -> bool {
        packets.iter().any(|p| p.starts_with(prefix))
    }

    #[test]
    fn walking_over_every_collect_thing_wins() {
        let mut run = plan(&row(Kind::Collect, 4));
        assert_eq!((run.done(), run.total()), (0, 4));
        let spots: Vec<_> = run.things.iter().map(|t| t.position).collect();
        for (n, spot) in spots.iter().enumerate() {
            let step = run.tick(0, *spot, 2_000);
            assert!(has(&step.packets, &[47, 0, 2]), "tracker count");
            assert!(has(&step.packets, &[45, 0, 2]), "goal count");
            assert!(has(&step.packets, &[39, 0, 21]), "star counter");
            assert!(has(&step.packets, &[35, 0, 3]), "the thing goes");
            assert_eq!(step.end.is_some(), n == 3);
        }
        assert_eq!(run.done(), 4);
    }

    #[test]
    fn a_click_needs_range_and_a_thing_of_the_run() {
        let mut run = plan(&row(Kind::Collect, 3));
        let first = run.things[0].clone();
        assert!(run.click(0xdead, GIVER, 2_000).is_none(), "a stranger's actor");
        let far = run.click(first.guid, [500.0, 10.0, 500.0], 2_000).unwrap();
        assert!(
            far.packets.is_empty() && run.done() == 0,
            "too far to click"
        );
        let near = run.click(first.guid, first.position, 2_000).unwrap();
        assert!(!near.packets.is_empty() && run.done() == 1);
        let again = run.click(first.guid, first.position, 2_000).unwrap();
        assert!(again.packets.is_empty() && run.done() == 1, "taken once");
    }

    #[test]
    fn a_return_run_wants_each_pickup_brought_to_the_giver() {
        let mut run = plan(&row(Kind::Carry, 2));
        let (a, b) = (run.things[0].clone(), run.things[1].clone());
        run.tick(0, a.position, 2_000);
        assert_eq!(run.done(), 0, "carried, not yet handed in");
        let step = run.tick(0, b.position, 2_100);
        assert!(step.packets.is_empty(), "no second pickup while carrying");
        let said = run.click(b.guid, b.position, 2_000).unwrap();
        assert!(has(&said.packets, &[15, 0, 3]), "the player is told why");
        run.tick(0, GIVER, 2_200);
        assert_eq!(run.done(), 1);
        run.tick(0, b.position, 2_300);
        let last = run.tick(0, GIVER, 2_400);
        assert_eq!(last.end, Some(End::Won));
    }

    #[test]
    fn a_chain_opens_each_goal_in_turn_and_wins_at_the_end() {
        let mut challenge = (*row(Kind::Collect, 2)).clone();
        challenge.stages.push(Stage {
            count: 1,
            radius: 0.0,
            model_id: 0,
            ..stage(Kind::Return, 1)
        });
        challenge.stages.push(stage(Kind::Use, 1));
        let mut run = plan(&Arc::new(challenge));
        let spots: Vec<_> = run.things.iter().map(|t| t.position).collect();
        run.tick(0, spots[0], 2_000);
        let filled = run.tick(0, spots[1], 2_100);
        assert!(has(&filled.packets, &[45, 0, 3]), "the first goal is complete");
        assert!(has(&filled.packets, &[47, 0, 1]), "the next goal's panel row");
        assert_eq!(run.total(), 1);
        assert!(run.tick(0, spots[1], 2_200).packets.is_empty(), "not back yet");
        let back = run.tick(0, GIVER, 2_300);
        assert!(has(&back.packets, &[35, 0, 2]), "the target goes up");
        let target = run.things[0].clone();
        let won = run.click(target.guid, target.position, 2_000).unwrap();
        assert_eq!(won.end, Some(End::Won));
        assert_eq!(run.done(), 4, "the score counts every stage");
    }

    #[test]
    fn a_bonus_goal_needs_the_win_inside_its_time() {
        let mut challenge = (*row(Kind::Collect, 1)).clone();
        challenge.bonuses.push(Bonus {
            name_id: 7,
            within_seconds: 5,
        });
        let run = plan(&Arc::new(challenge));
        let id = 3_i32.to_le_bytes();
        let done = |packets: &[Vec<u8>], sub: u8| {
            packets.iter().any(|p| p.starts_with(&[45, 0, sub]) && p[3..7] == id)
        };
        let quick = run.ending_packets(true, 2_000);
        assert!(done(&quick, 3) && !done(&quick, 4));
        let slow = run.ending_packets(true, 30_000);
        assert!(done(&slow, 4) && !done(&slow, 3));
    }

    #[test]
    fn a_race_takes_its_flags_in_order() {
        let mut run = plan(&row(Kind::Race, 4));
        let flags: Vec<_> = run.things.clone();
        assert!(flags.len() >= 2);
        let wrong = run.click(flags[1].guid, flags[1].position, 2_000).unwrap();
        assert!(
            wrong.packets.is_empty() && run.done() == 0,
            "second flag first"
        );
        let mut last = Step::default();
        for flag in &flags {
            last = run.tick(0, flag.position, 3_000);
        }
        assert_eq!(last.end, Some(End::Won));
        assert_eq!(run.model_for(0), GREEN_FLAG_MODEL);
        assert_eq!(run.model_for(flags.len() - 1), RED_FLAG_MODEL);
    }

    #[test]
    fn a_use_target_stays_standing_and_counts_once() {
        let mut run = plan(&row(Kind::Use, 3));
        let spots: Vec<_> = run.things.clone();
        let walked = run.tick(0, spots[0].position, 2_000);
        assert!(
            walked.packets.is_empty(),
            "targets are clicked, not walked over"
        );
        let step = run.click(spots[0].guid, spots[0].position, 2_000).unwrap();
        assert_eq!(run.done(), 1);
        assert!(
            !has(&step.packets, &[35, 0, 3]),
            "a used target is not removed"
        );
        let again = run.click(spots[0].guid, spots[0].position, 2_000).unwrap();
        assert!(again.packets.is_empty() && run.done() == 1);
        let removed = run
            .cleanup_packets()
            .iter()
            .filter(|p| p.starts_with(&[35, 0, 3]))
            .count();
        assert_eq!(removed, 3, "the end clears every target, used or not");
    }

    #[test]
    fn a_delivery_visits_each_recipient_once_in_any_order() {
        let mut challenge = (*row(Kind::Deliver, 2)).clone();
        challenge.stages[0].recipients = vec![11, 12];
        challenge.stages[0].radius = 0.0;
        let challenge = Arc::new(challenge);
        let stops = vec![(11, [20.0, 10.0, 0.0]), (12, [0.0, 10.0, 30.0])];
        let mut run = plan_with(&challenge, stops, &|_, _| None, true).unwrap();
        assert!(run.reach >= 30.0, "the far stop is inside the area");
        let first = run.tick(0, [0.0, 10.0, 30.0], 2_000);
        assert_eq!(run.done(), 1);
        assert!(has(&first.packets, &[47, 0, 14]), "the arrow moves on");
        assert!(run.tick(0, [0.0, 10.0, 30.0], 2_100).packets.is_empty());
        let last = run.tick(0, [20.0, 10.0, 0.0], 2_200);
        assert_eq!(last.end, Some(End::Won));
        assert!(
            !has(&run.cleanup_packets(), &[35, 0, 3]),
            "nothing was spawned"
        );
    }

    #[test]
    fn the_deadline_the_zone_and_the_distance_end_a_run() {
        let mut run = plan(&row(Kind::Collect, 3));
        assert_eq!(run.tick(0, GIVER, 61_000).end, Some(End::TimedOut));
        assert_eq!(run.tick(5, GIVER, 2_000).end, Some(End::Left));
        let far = [GIVER[0] + 500.0, 10.0, GIVER[2]];
        assert_eq!(run.tick(0, far, 2_000).end, Some(End::Left));
        assert_eq!(
            run.elapsed_seconds(400_000),
            60,
            "elapsed stops at the deadline"
        );
    }

    #[test]
    fn the_layout_keeps_its_distances() {
        let tune = tuning();
        let mut rng = Rng::new(3);
        let ground = |_: f32, _: f32| None;
        let field = Field::new(GIVER, &ground, &|_, _| false, &|_, _, _| None, true, &tune, 30.0);
        let points = layout::scatter(&field, 30.0, 10, &mut rng);
        assert_eq!(points.len(), 10);
        for (i, p) in points.iter().enumerate() {
            assert!(flat_distance(*p, GIVER) >= tune.min_center_distance);
            assert!(flat_distance(*p, GIVER) <= 30.0 + 0.01);
            for q in &points[i + 1..] {
                assert!(flat_distance(*p, *q) >= tune.min_spacing);
            }
        }
    }

    #[test]
    fn a_cliff_or_a_hole_is_never_planted() {
        let tune = tuning();
        let mut rng = Rng::new(9);
        // East of the giver the ground is a plateau 100 m up; north-west is a hole.
        let ground = |x: f32, z: f32| {
            if x > 0.0 {
                Some(110.0)
            } else if z > 0.0 {
                None
            } else {
                Some(10.0)
            }
        };
        let field = Field::new(GIVER, &ground, &|_, _| false, &|_, _, _| None, false, &tune, 30.0);
        let points = layout::scatter(&field, 30.0, 6, &mut rng);
        assert!(!points.is_empty());
        assert!(
            points
                .iter()
                .all(|p| p[0] <= 0.0 && p[2] <= 0.0 && p[1] == 10.0)
        );
    }

    #[test]
    fn a_rock_top_behind_a_steep_side_is_never_planted() {
        let tune = tuning();
        let mut rng = Rng::new(11);
        // A flat top 5 m up, inside the area and within max_rise, with a sheer side.
        let ground = |x: f32, _: f32| Some(if x > 8.0 { 15.0 } else { 10.0 });
        let field = Field::new(GIVER, &ground, &|_, _| false, &|_, _, _| None, false, &tune, 30.0);
        let points = layout::scatter(&field, 30.0, 8, &mut rng);
        assert!(!points.is_empty());
        assert!(points.iter().all(|p| p[0] <= 9.0), "{points:?}");
    }

    #[test]
    fn nothing_is_planted_inside_or_behind_a_prop() {
        let tune = tuning();
        let mut rng = Rng::new(13);
        let ground = |_: f32, _: f32| Some(10.0);
        // A tree on the giver's east side, and a long wall north of it.
        let hit = |p: [f32; 3]| p[0].hypot(p[2] - 12.0) < 2.0 || (p[2] > 18.0 && p[2] < 20.0);
        let blocked = |a: [f32; 3], b: [f32; 3]| hit(a) || hit(b);
        let field = Field::new(GIVER, &ground, &blocked, &|_, _, _| None, false, &tune, 30.0);
        let points = layout::scatter(&field, 30.0, 12, &mut rng);
        assert!(!points.is_empty());
        assert!(points.iter().all(|p| !hit(*p)), "inside a prop: {points:?}");
        assert!(points.iter().all(|p| p[2] < 20.0), "behind the wall: {points:?}");
    }

    #[test]
    fn a_boxed_in_giver_or_a_cave_still_gets_a_game() {
        let tune = tuning();
        let mut rng = Rng::new(14);
        // A wall all round, three metres off.
        let ground = |_: f32, _: f32| Some(10.0);
        let walled = |a: [f32; 3], b: [f32; 3]| a[0].hypot(a[2]) > 3.0 || b[0].hypot(b[2]) > 3.0;
        let field = Field::new(GIVER, &ground, &walled, &|_, _, _| None, false, &tune, 30.0);
        assert!(!field.walkable_is_filtering());
        // The grid is a hillside 50 m over the giver's cave.
        let hill = |_: f32, _: f32| Some(60.0);
        let cave = Field::new(GIVER, &hill, &|_, _| false, &|_, _, _| None, false, &tune, 30.0);
        let points = layout::scatter(&cave, 30.0, 6, &mut rng);
        assert_eq!(points.len(), 6);
        assert!(points.iter().all(|p| p[1] == GIVER[1]), "at the giver's height");
    }

    #[test]
    fn a_floor_the_grid_does_not_know_is_walked_on() {
        let tune = tuning();
        let mut rng = Rng::new(15);
        // The grid is a hillside 50 m up; a mesh floor runs out to x = 15 at the giver's height.
        let hill = |_: f32, _: f32| Some(60.0);
        let deck = |x: f32, _: f32, from: f32| ((from - 10.0).abs() < 2.0 && x < 15.0).then_some(10.0);
        let field = Field::new(GIVER, &hill, &|_, _| false, &deck, false, &tune, 30.0);
        assert!(field.walkable_is_filtering(), "walked, not planned flat");
        let points = layout::scatter(&field, 30.0, 8, &mut rng);
        assert!(!points.is_empty());
        assert!(points.iter().all(|p| p[1] == 10.0 && p[0] < 16.0), "{points:?}");
    }

    #[test]
    fn a_gentle_slope_is_walked_up() {
        let tune = tuning();
        let mut rng = Rng::new(12);
        let ground = |x: f32, _: f32| Some(10.0 + x.max(0.0) * 0.3);
        let field = Field::new(GIVER, &ground, &|_, _| false, &|_, _, _| None, false, &tune, 20.0);
        let points = layout::scatter(&field, 20.0, 12, &mut rng);
        assert!(points.iter().any(|p| p[0] > 8.0), "{points:?}");
    }

    #[test]
    fn the_star_counter_follows_progress_toward_the_payout() {
        let mut run = plan(&row(Kind::Collect, 4));
        let spots: Vec<_> = run.things.iter().map(|t| t.position).collect();
        run.tick(0, spots[0], 2_000);
        let step = run.tick(0, spots[1], 2_100);
        let stars = step
            .packets
            .iter()
            .find(|p| p.starts_with(&[39, 0, 21]))
            .unwrap();
        assert_eq!(&stars[3..7], &7_i32.to_le_bytes(), "instance");
        assert_eq!(&stars[7..11], &10_i32.to_le_bytes(), "2 of 4 of 20 stars");
        assert_eq!(&stars[11..15], &900_i32.to_le_bytes(), "game id");
    }

    #[test]
    fn the_offer_is_a_valid_info_record_with_no_score_screen_type() {
        let challenge = row(Kind::Collect, 4);
        let spec = challenge.run_spec(20);
        assert_eq!(spec.objectives.len(), 2, "the win goal and one stage");
        assert_eq!(spec.objectives[0].category, 1);
        assert_eq!(spec.objectives[1].category, 2);
        assert_eq!(spec.reward_bundles.len(), 3 * 69);
        let mut runs = MinigameRuns::new(FIRST_INSTANCE_ID).unwrap();
        let offered = runs.offer(spec).unwrap();
        assert!(offered.accepted);
        assert!(offered.replies[0].starts_with(&[39, 0, 16]));
    }

    #[test]
    fn malformed_content_is_refused() {
        let shipped = fs::read(crate::content::bundled_content().join(CONTENT_FILE)).unwrap();
        let good: serde_json::Value = serde_json::from_slice(&shipped).unwrap();
        assert!(Challenges::parse(&shipped).is_ok());
        let refuse = |edit: &dyn Fn(&mut serde_json::Value)| {
            let mut value = good.clone();
            edit(&mut value);
            Challenges::parse(&serde_json::to_vec(&value).unwrap()).is_err()
        };
        assert!(refuse(
            &|v| v["challenges"][1]["id"] = v["challenges"][0]["id"].clone()
        ));
        assert!(refuse(&|v| v["challenges"][0]["stages"][0]["count"] = 0.into()));
        assert!(refuse(
            &|v| v["challenges"][0]["time_limit_seconds"] = 99_999.into()
        ));
        assert!(refuse(&|v| v["challenges"][0]["stages"][0]["kind"] = "swim".into()));
        assert!(refuse(&|v| v["challenges"][0]["stages"] = serde_json::json!([])));
        assert!(refuse(&|v| v["tuning"]["touch_range"] = (-1.0).into()));
        assert!(refuse(&|v| v["schema_version"] = 9.into()));
        // A delivery names exactly as many people as it visits.
        let deliver = good["challenges"]
            .as_array()
            .unwrap()
            .iter()
            .position(|r| r["stages"][0]["kind"] == "deliver")
            .unwrap();
        assert!(refuse(
            &|v| v["challenges"][deliver]["stages"][0]["count"] = 9.into()
        ));
    }

    #[test]
    fn the_shipped_challenges_resolve_against_the_npc_content() {
        let dir = crate::content::bundled_content();
        let content = Challenges::load(&dir).unwrap().expect("the shipped file");
        let ids: Vec<u32> = content.rows().map(|r| r.id).collect();
        for want in [126, 224, 241, 587, 588] {
            assert!(ids.contains(&want), "challenge {want} is missing");
        }
        let npcs = crate::npc_interaction::NpcInteraction::load(
            &dir.join("npcs"),
            &dir.join("stores.json"),
        )
        .unwrap();
        for row in content.rows() {
            assert!(npcs.location(row.giver).is_some(), "{}: giver", row.id);
            for stop in row.stages.iter().flat_map(|s| &s.recipients) {
                assert!(npcs.location(*stop).is_some(), "{}: recipient", row.id);
            }
        }
    }
}
