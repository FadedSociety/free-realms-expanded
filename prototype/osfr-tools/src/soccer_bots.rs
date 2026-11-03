//! L932: the server's soccer players (owner 2026-11-02: "Soccer needs enemy ai").
//!
//! Built to the live server's recorded match (packets_soccer_minigame.pcapng, 1,191 76/1 updates):
//! three field players and a keeper a side; field players roam the whole pitch at about 6.5 units/s,
//! keepers slide along their goal line (about 3.3 units either side of the goal centre) at 3-5.
//! The rules a bot follows are the wiki's game: chase a loose ball, dribble at the other goal, shoot
//! when close, pass when an opponent blocks the way, and slide-tackle the ball carrier. Every number
//! below is (R) unless it says live.
use std::collections::{BTreeMap, BTreeSet};
use std::io;

use crate::soccer_eval;
use crate::soccer_match::{Authority, Intent, KickKind, MatchPhase, TEAM_SLOT_COUNT};

/// Field player run speed (live: 6.5-6.65).
pub const RUN_SPEED: f32 = 6.5;
/// A defender closing on the ball carrier sprints (live: 8.0 bursts), so a carrier at
/// `RUN_SPEED` can be caught.
pub const SPRINT_SPEED: f32 = 8.0;
/// Off-the-ball support speed (live: 6.0).
pub const SUPPORT_SPEED: f32 = 6.0;
/// Keeper speed along the goal line (live: 3-5).
pub const KEEPER_SPEED: f32 = 4.0;
/// How far a keeper strays from the goal centre along the line (live: about 3.3).
pub const KEEPER_REACH: f32 = 3.3;
/// L955: how far off its spot on the line a keeper drifts before it moves (R).
pub const KEEPER_DEADBAND: f32 = 0.6;
/// A ball carrier shoots inside this distance of the goal (R).
pub const SHOOT_RANGE: f32 = 14.0;
/// Shot and pass power (R; the match caps kicks at `max_kick_power`).
pub const SHOT_POWER: f32 = 22.0;
pub const PASS_POWER: f32 = 14.0;
/// L956: a carrier shoots in range only through a line this open (`soccer_eval::passing_odds`,
/// field players only), or from half the range regardless (R).
const SHOT_ODDS: f32 = 0.5;
/// L956: a pass goes when its rating beats the carrier's own situation by this much, weighing the
/// goal gain this much against safety (R).
const PASS_MARGIN: f32 = 0.3;
const OPPORTUNISM: f32 = 0.5;
/// L956: a dribble looks this far ahead for space, and leans goalward by this much (R).
const DRIBBLE_LOOK: f32 = 3.0;
const DRIBBLE_GOALWARD: f32 = 0.3;
/// L956: a marker stands this far goal-side of its man (R).
const MARK_GAP: f32 = 2.0;
/// L956: a support or marking run ends this close to its spot (R): a marker never quite reaches a
/// spot that moves with its man, and kept re-aiming.
const SUPPORT_ARRIVE: f32 = 1.0;
/// L956: how often a supporting or marking bot re-picks its spot (R).
const SUPPORT_REPLAN: f32 = 0.5;
/// L956: a supporting bot goes this share of the way to the ball along the pitch when its side has
/// it, this share otherwise, and this share across (R).
const ATTACK_PUSH: f32 = 0.7;
const SHAPE_PUSH: f32 = 0.5;
const SIDE_PULL: f32 = 0.3;
/// Seconds between one bot's tackles (R), so a tackle can be escaped. L955: 1.5 let the bots steal
/// back and forth every two seconds (owner playtest: 169 steals a match).
pub const TACKLE_COOLDOWN: f32 = 3.0;
/// L955: seconds a player who just won the ball cannot be tackled (R), so possession settles.
pub const PROTECT: f32 = 1.5;
/// L955: seconds a juking player cannot be tackled (R): the client's Ctrl juke (states 4-6).
pub const DODGE_TIME: f32 = 0.8;
/// L955: seconds between jukes that count (R), so spamming the dodge cannot keep a player
/// untackleable (Opus review).
pub const JUKE_COOLDOWN: f32 = 2.0;
/// L955: another bot takes over the chase only when it is this much nearer the ball (R).
pub const CHASE_SWITCH: f32 = 2.5;
/// L955: a supporting bot re-forms only when its spot moved this far (R): no stop-start churn.
pub const SUPPORT_DEADBAND: f32 = 2.0;
/// L955: a carrier does not pass back to who passed to it within this many seconds (R).
pub const PASS_BACK: f32 = 3.0;
/// L955: a teammate that has not moved for this long gets no pass (R): no feeding a still player.
pub const IDLE_MATE: f32 = 2.0;
/// L955: how close a chaser stays to a carrier it cannot tackle yet (R).
const SHADOW: f32 = 2.5;
/// L955: a shadowing chaser closes in again only this far past `SHADOW` (R): stopping and starting
/// at the one distance flapped its stick every tick.
const SHADOW_SLACK: f32 = 1.0;
/// L955: share of in-reach bot tackles that land (R); a miss still spends the cooldown. The live
/// match had about one slide tackle per 11 s; with every bot landing every try ours had one per 2 s.
pub const TACKLE_LANDS: f32 = 0.4;
/// L955: a tackle reaches this far: the client's own tackle range (00cd8330: distance^2 <= 3.0).
/// The 2 m control radius let bots steal from behind the player (owner, native 2026-11-03).
pub const TACKLE_REACH: f32 = 1.732;
/// Seconds a slide-tackled bot stays down (R; the Soccer Star Toughness stat shortens it).
pub const KNOCKDOWN: f32 = 1.5;
/// The keeper slot (live: the fourth server player of each side stood on the goal line).
pub const KEEPER_SLOT: u32 = TEAM_SLOT_COUNT - 1;
/// L949: a keeper dives at a shot this close to its line (seconds) and this far across it (R).
pub const DIVE_TIME: f32 = 0.3;
pub const DIVE_REACH: f32 = 3.0;
/// L955: how long a dive lasts: its time to the line and a little after (R). The keeper is down and
/// its reach wide for all of it, so a shot that arrives late is still its save.
const DIVE_WINDOW: f32 = DIVE_TIME + 0.2;
/// L949: a shot this hard (of `max_kick_power`) is blocked and cleared, not caught (R: live, a
/// KickRight was caught; full-charge SuperKicks were blocked or went in).
pub const BLOCK_SHOT: f32 = 0.95;
/// L949: a bot's shot aims at least this share of the way from the goal centre to the far post (R).
const AIM_MIN: f32 = 0.3;
/// L949: how hard a keeper clears a blocked shot (R).
const CLEAR_POWER: f32 = 12.0;
/// L949: a caught ball stays in the keeper's hands this long, then the ball-in-hand state, then the
/// throw (live: 0.99 s, then 0.08 s).
pub const KEEPER_HOLD: f32 = 0.9;
const THROW_WINDUP: f32 = 0.1;
/// L949: goalie states (client state table 0x1BA2338): BlockHighLeft 51 (+1 right, +2 low),
/// BlockCenter 57, CatchHigh 58 / Mid 59 / Low 60, IdleWithBallInHand 61, Throw 62.
pub const BLOCK_HIGH_LEFT: u8 = 51;
pub const BLOCK_CENTER: u8 = 57;
pub const CATCH_HIGH: u8 = 58;
pub const BALL_IN_HAND: u8 = 61;
pub const THROW: u8 = 62;

/// Kickoff spots in "defending" coordinates: x is distance toward the side's own goal from the
/// pitch centre, z is across. Live kickoff frame 6451: (10.5, -8.9), (10.5, 9.0), keeper (25.8, 0);
/// slot 0 is the other side's centre forward (3.9, 2.7) (the joining player kicked off at (1.6, 4.5)).
const HOMES: [[f32; 2]; TEAM_SLOT_COUNT as usize] =
    [[3.9, 2.7], [10.5, -8.9], [10.5, 9.0], [25.8, 0.0]];

/// What one bot did this step, for the service to send.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BotEvent {
    Moved {
        player: u64,
        position: [f32; 3],
        facing: [f32; 2],
        speed: f32,
    },
    Kicked {
        player: u64,
        kind: KickKind,
        power: f32,
    },
    Tackled {
        tackler: u64,
        victim: u64,
    },
    /// L949: a keeper's save, ball in hand or throw: its goalie state, shown for `hold` seconds.
    Goalie {
        player: u64,
        state: u8,
        hold: f32,
    },
}

/// One bot's memory between steps.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BotMemory {
    pub tackle_ready_at: f32,
    /// A knocked-down bot neither moves nor tackles until this server time.
    pub down_until: f32,
    /// L942: a grabbed power-up (until, factor): Speed scales the run speeds, Toughness the knockdown time.
    pub speed_boost: Option<(f32, f32)>,
    pub tough_boost: Option<(f32, f32)>,
    /// L944: the player's Soccer Star stats, when the match sent them (76/19).
    pub stats: Option<Stats>,
    /// L949: a keeper's caught ball: when the next hand step is due, whether the ball-in-hand state
    /// went out, and whether its dive was a block (the ball goes straight back out).
    pub hold_until: Option<f32>,
    pub in_hand: bool,
    pub blocking: bool,
    /// L955: this player won the ball and cannot be tackled until then.
    pub protected_until: f32,
    /// L955: this player is juking (client states 4-6) and cannot be tackled until then.
    pub dodge_until: f32,
    /// L955: this bot is its side's chaser (kept until another is clearly nearer).
    pub chasing: bool,
    /// L955: who last passed to this player, and when.
    pub passed_by: Option<(u64, f32)>,
    /// L955: where this player was when it last moved, and when (for idle teammates).
    pub last_at: Option<([f32; 3], f32)>,
    /// L955: a support run under way (until it arrives) and a chaser holding off a carrier.
    pub support_run: bool,
    pub shadowing: bool,
    /// L955: when this player's next juke counts (`JUKE_COOLDOWN`).
    pub juke_ready_at: f32,
    /// L956: the opponent this bot marks while its side defends.
    pub mark: Option<u64>,
    /// L956: a supporting bot's spot and when it was picked.
    pub support_spot: Option<([f32; 3], f32)>,
    /// L956: a carrier's dribble direction and when it was picked.
    pub dribble: Option<([f32; 2], f32)>,
}

impl BotMemory {
    /// L956: drops the shape plans (man, spot, dribble). The match clock stands still between a
    /// goal or the whistle and the kickoff, so a plan from before a line-up still looked fresh.
    pub fn forget_plans(&mut self) {
        self.mark = None;
        self.support_spot = None;
        self.dribble = None;
    }
}

/// L955: whether `player` may be tackled now (not just won the ball, not juking).
pub fn tackleable(memory: &BTreeMap<u64, BotMemory>, player: u64, now: f32) -> bool {
    memory
        .get(&player)
        .is_none_or(|m| now >= m.protected_until && now >= m.dodge_until)
}

/// L955: a tackle took the ball: the tackler keeps it for PROTECT, the carrier goes down.
pub fn won_by_tackle(memory: &mut BTreeMap<u64, BotMemory>, tackler: u64, victim: u64, now: f32) {
    memory.entry(tackler).or_default().protected_until = now + PROTECT;
    knock_down(memory, tackler, victim, now);
}

/// L955: the client's juke (states 4 Juke, 5 JukeWithBall, 6 Juke360WithBall).
pub fn juke(memory: &mut BTreeMap<u64, BotMemory>, player: u64, now: f32) {
    let mind = memory.entry(player).or_default();
    if now < mind.juke_ready_at {
        return;
    }
    mind.juke_ready_at = now + JUKE_COOLDOWN;
    mind.dodge_until = now + DODGE_TIME;
}

/// L944: a player's Soccer Star stats as sent in 76/19 words 0..5 (the client's stat-name table order,
/// stat ids 39-43, R). The wiki: Kick Power harder kicks, Tackle Power stronger tackles, Toughness
/// back up quicker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stats {
    pub kick: u32,
    pub footwork: u32,
    pub speed: u32,
    pub toughness: u32,
    pub tackle: u32,
}

impl Stats {
    pub fn from_words(words: &[u32; 9]) -> Self {
        Self {
            kick: words[0],
            footwork: words[1],
            speed: words[2],
            toughness: words[3],
            tackle: words[4],
        }
    }
}

/// The live server's field player (8, 8, 8, 8, 2): a stat at its base leaves the rule as it is.
pub const BASE_STATS: Stats = Stats {
    kick: 8,
    footwork: 8,
    speed: 8,
    toughness: 8,
    tackle: 2,
};

/// L944/L945: a tackled player goes down for KNOCKDOWN, longer by the tackler's Tackle Power,
/// shorter by its own Toughness and the Toughness power-up. Bots and the human tackle alike.
pub fn knock_down(memory: &mut BTreeMap<u64, BotMemory>, tackler: u64, victim: u64, now: f32) {
    let hit = ratio(memory.get(&tackler).and_then(|m| m.stats), |s| s.tackle);
    let victim = memory.entry(victim).or_default();
    victim.down_until = now
        + KNOCKDOWN * hit * boost(victim.tough_boost, now) / ratio(victim.stats, |s| s.toughness);
}

/// A stat against its base. Zero means "not this player's stat" (the keepers' Speed, Kick and
/// Tackle are 0 and they still move), so it leaves the rule as it is.
fn ratio(stats: Option<Stats>, pick: fn(&Stats) -> u32) -> f32 {
    match stats.map(|stats| pick(&stats)) {
        Some(value) if value > 0 => value as f32 / pick(&BASE_STATS) as f32,
        _ => 1.0,
    }
}

/// L942: a live boost's factor, or 1.
fn boost(boost: Option<(f32, f32)>, now: f32) -> f32 {
    boost.filter(|(until, _)| now < *until).map_or(1.0, |(_, factor)| factor)
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn flat_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = sub(a, b);
    (d[0] * d[0] + d[2] * d[2]).sqrt()
}

/// Move `from` toward `to` by at most `step` on the pitch plane.
fn toward(from: [f32; 3], to: [f32; 3], step: f32) -> ([f32; 3], [f32; 2]) {
    let d = sub(to, from);
    let length = (d[0] * d[0] + d[2] * d[2]).sqrt();
    if length <= f32::EPSILON {
        return (from, [0.0, 0.0]);
    }
    let facing = [d[0] / length, d[2] / length];
    let moved = step.min(length);
    (
        [
            from[0] + facing[0] * moved,
            from[1],
            from[2] + facing[1] * moved,
        ],
        facing,
    )
}

/// The kickoff spot of `slot` on `team`.
pub fn home(authority: &Authority, team: u32, slot: u32) -> [f32; 3] {
    let c = authority.config().bounds.centre;
    let defend = -authority.attack_sign(team);
    let [x, z] = HOMES[(slot % TEAM_SLOT_COUNT) as usize];
    [c[0] + defend * x, c[1], c[2] + z]
}

/// Put every bot on its kickoff spot (kickoff and each restart).
pub fn line_up(authority: &mut Authority, bots: &BTreeSet<u64>) -> io::Result<Vec<BotEvent>> {
    let mut events = Vec::new();
    if !authority.config().bot_ai {
        return Ok(events);
    }
    for &bot in bots {
        let Some(state) = authority.player(bot).copied() else {
            continue;
        };
        let mut position = home(authority, state.team, state.slot);
        // The kicking-off side's forward stands at the ball (a step back toward its own goal).
        if state.team == authority.kickoff_team() && state.slot == 0 {
            let spot = authority.config().kickoff_spawn;
            position = [
                spot[0] - authority.attack_sign(state.team),
                spot[1],
                spot[2],
            ];
        }
        authority.place_server_player(bot, position)?;
        let facing = [authority.attack_sign(state.team), 0.0];
        events.push(BotEvent::Moved {
            player: bot,
            position,
            facing,
            speed: 0.0,
        });
    }
    Ok(events)
}

/// One AI step of `dt` seconds for every bot, before the match advances.
pub fn step(
    authority: &mut Authority,
    bots: &BTreeSet<u64>,
    memory: &mut BTreeMap<u64, BotMemory>,
    dt: f32,
) -> io::Result<Vec<BotEvent>> {
    if !authority.config().bot_ai || authority.phase() != MatchPhase::Running || !(dt > 0.0) {
        return Ok(Vec::new());
    }
    let now = authority.server_time();
    let ball = authority.ball_position();
    let pass_radius = authority.config().physics.pass_radius;
    // L955: every player's last move (a still teammate gets no pass).
    for (id, state) in authority.players() {
        if let Some(position) = state.position {
            let m = memory.entry(id).or_default();
            let moved = m
                .last_at
                .is_none_or(|(at, _)| flat_distance(at, position) > 0.3);
            if moved {
                m.last_at = Some((position, now));
            }
        }
    }
    // The field player of each side nearest the ball goes for it; the rest support. L955: the
    // current chaser keeps the chase unless another is clearly nearer (no target flip-flop).
    let mut chaser = [None::<(u64, f32)>; 2];
    let mut current = [None::<(u64, f32)>; 2];
    for (id, state) in authority.players() {
        let (Some(position), true) = (state.position, state.slot != KEEPER_SLOT) else {
            continue;
        };
        // A knocked-down bot cannot chase; the next nearest teammate goes instead.
        if !bots.contains(&id) || memory.get(&id).is_some_and(|m| now < m.down_until) {
            continue;
        }
        let distance = flat_distance(position, ball);
        let team = (state.team & 1) as usize;
        if memory.get(&id).is_some_and(|m| m.chasing) {
            current[team] = Some((id, distance));
        }
        let side = &mut chaser[team];
        if side.is_none_or(|(_, best)| distance < best) {
            *side = Some((id, distance));
        }
    }
    for team in 0..2 {
        if let (Some(kept), Some((_, best))) = (current[team], chaser[team])
            && kept.1 <= best + CHASE_SWITCH
        {
            chaser[team] = Some(kept);
        }
    }
    // L955: a pass's target goes to meet the ball (live: it was already running to it when the
    // ball arrived); before, only the team's nearest bot moved and the target stood still.
    if let Some(target) = authority.live_pass()
        && bots.contains(&target)
        && let Some(state) = authority.player(target)
    {
        chaser[(state.team & 1) as usize] = Some((target, 0.0));
    }
    for &bot in bots {
        let chasing = chaser.iter().flatten().any(|(id, _)| *id == bot);
        let mind = memory.entry(bot).or_default();
        mind.chasing = chasing;
        // L956: a chaser or the carrier marks nobody (a stale man would be taken back later).
        if chasing || authority.possession() == Some(bot) {
            mind.mark = None;
        }
    }
    let roster: Vec<(u64, u32, u32, [f32; 3])> = authority
        .players()
        .filter_map(|(id, s)| s.position.map(|p| (id, s.team, s.slot, p)))
        .collect();
    let mut events = Vec::new();
    // L956: opponents already marked (each once). Kept men are claimed up front, so a bot earlier
    // in id order does not take a later bot's man.
    let mut marked: BTreeSet<u64> = bots
        .iter()
        .filter_map(|bot| memory.get(bot).and_then(|m| m.mark))
        .collect();
    for &bot in bots {
        let Some(state) = authority.player(bot).copied() else {
            continue;
        };
        let Some(at) = state.position else { continue };
        if memory.get(&bot).is_some_and(|m| now < m.down_until) {
            events.push(stopped(bot, at, ball));
            continue;
        }
        let stats = memory.get(&bot).and_then(|m| m.stats);
        let pace = boost(memory.get(&bot).and_then(|m| m.speed_boost), now)
            * ratio(stats, |s| s.speed);
        let strength = ratio(stats, |s| s.kick);
        let goal = authority.attack_goal(state.team);
        let bounds = authority.config().bounds;
        let c = bounds.centre;
        // Read possession per bot: an earlier bot this step may have tackled or kicked.
        let holder = authority.possession();
        let holder_team = holder.and_then(|id| authority.player(id)).map(|s| s.team);
        let in_reach = |mate_at: [f32; 3]| flat_distance(mate_at, at) <= pass_radius;

        // Keeper: slide along the own goal line with the ball.
        if state.slot == KEEPER_SLOT {
            let line = home(authority, state.team, state.slot);
            let outward = [authority.attack_sign(state.team), 0.0];
            // L949: a shot on goal: the keeper moves to where it crosses, then dives at it.
            let shot = incoming(authority, line[0]);
            if let Some((time, z, height)) = shot {
                if time <= DIVE_TIME && (z - at[2]).abs() <= DIVE_REACH {
                    let max = authority.config().physics.max_kick_power;
                    let v = authority.ball_velocity();
                    let blocking =
                        (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt() >= BLOCK_SHOT * max;
                    // Positive is the keeper's right (it faces the field).
                    let side = -(z - at[2]) * outward[0];
                    let dive = if blocking {
                        block_state(side, height)
                    } else {
                        catch_state(height)
                    };
                    // L955: the keeper saves where it stands, as live (blocks 51-57 and catches
                    // 58-60 are in-place animations): for the dive its reach covers the shot's
                    // line. It used to jump, then slide, across to it (owner: it teleports).
                    // R: half a metre of arm past the dive's reach.
                    authority.extend_reach(bot, DIVE_REACH + 0.5, now + DIVE_WINDOW);
                    let mind = memory.entry(bot).or_default();
                    mind.blocking = blocking;
                    mind.down_until = now + DIVE_WINDOW;
                    events.push(BotEvent::Moved {
                        player: bot,
                        position: at,
                        facing: outward,
                        speed: 0.0,
                    });
                    events.push(BotEvent::Goalie {
                        player: bot,
                        state: dive,
                        hold: KEEPER_HOLD,
                    });
                    continue;
                }
            }
            let reach = if shot.is_some() {
                authority.config().bounds.goal_half_width
            } else {
                KEEPER_REACH
            };
            let target = [
                line[0],
                at[1],
                shot.map_or(ball[2], |shot| shot.1)
                    .clamp(c[2] - reach, c[2] + reach),
            ];
            // L955: a keeper re-places itself only past KEEPER_DEADBAND (at once for a shot) and
            // finishes the move: inching after the ball flapped its stick.
            let away = flat_distance(at, target);
            let mind = memory.entry(bot).or_default();
            // R: 5 cm counts as on its spot.
            if away > KEEPER_DEADBAND || (shot.is_some() && away > 0.05) {
                mind.support_run = true;
            } else if away <= 0.05 {
                mind.support_run = false;
            }
            let (next, facing) = toward(at, target, KEEPER_SPEED * pace * dt);
            if mind.support_run && next != at {
                authority.place_server_player(bot, next)?;
                events.push(BotEvent::Moved {
                    player: bot,
                    position: next,
                    facing,
                    speed: KEEPER_SPEED * pace,
                });
            } else {
                events.push(stopped(bot, at, ball));
            }
            let mind = memory.entry(bot).or_default();
            if holder != Some(bot) {
                *mind = BotMemory {
                    hold_until: None,
                    in_hand: false,
                    blocking: false,
                    ..*mind
                };
                continue;
            }
            // L949: a blocked shot goes straight back out, toward the side it came from.
            if mind.blocking {
                mind.blocking = false;
                let across = if at[2] >= c[2] { 0.5 } else { -0.5 };
                let away = [outward[0], 0.3, across];
                events.extend(bot_kick(
                    authority,
                    bot,
                    KickKind::Shot,
                    away,
                    CLEAR_POWER,
                    None,
                ));
                continue;
            }
            // L949: a caught ball is held, shown in hand, then thrown to the nearest teammate (R).
            match mind.hold_until {
                None => mind.hold_until = Some(now + KEEPER_HOLD),
                Some(due) if now < due => {}
                Some(_) if !mind.in_hand => {
                    mind.in_hand = true;
                    mind.hold_until = Some(now + THROW_WINDUP);
                    events.push(BotEvent::Goalie {
                        player: bot,
                        state: BALL_IN_HAND,
                        hold: THROW_WINDUP,
                    });
                }
                Some(_) => {
                    let mate = roster
                        .iter()
                        .filter(|&&(id, team, _, p)| team == state.team && id != bot && in_reach(p))
                        .min_by(|a, b| flat_distance(a.3, at).total_cmp(&flat_distance(b.3, at)));
                    if let Some(&(mate, _, _, mate_at)) = mate {
                        let thrown = pass(authority, bot, mate, mate_at, strength);
                        if thrown.is_some() {
                            mind.hold_until = None;
                            mind.in_hand = false;
                            events.push(BotEvent::Goalie {
                                player: bot,
                                state: THROW,
                                hold: KEEPER_HOLD,
                            });
                        }
                        events.extend(thrown);
                    }
                }
            }
            continue;
        }

        // Carrier: shoot through an open line, pass when a team-mate is better placed, or dribble
        // goalward into space. L956: read with `soccer_eval` (League-Soccer's ratings).
        if holder == Some(bot) {
            let flat = |p: [f32; 3]| [p[0], p[2]];
            let opponents: Vec<[f32; 2]> = roster
                .iter()
                .filter(|r| r.1 != state.team)
                .map(|r| flat(r.3))
                .collect();
            let to_goal = flat_distance(at, goal);
            if to_goal <= SHOOT_RANGE {
                let field: Vec<[f32; 2]> = roster
                    .iter()
                    .filter(|&&(_, team, slot, _)| team != state.team && slot != KEEPER_SLOT)
                    .map(|r| flat(r.3))
                    .collect();
                let aim = far_post(authority, &roster, state.team, goal, aim_roll(bot, now));
                // The keeper is the dive's business; field players in the line block it.
                let open = soccer_eval::passing_odds(flat(at), flat(aim), &field) >= SHOT_ODDS;
                if open || to_goal <= SHOOT_RANGE * 0.5 {
                    events.extend(kick(authority, bot, aim, strength));
                    continue;
                }
            }
            let mates = roster.iter().filter(|&&(id, team, slot, p)| {
                // L955: not back to who just passed here, not to a team-mate standing still.
                let passed_back = memory
                    .get(&bot)
                    .and_then(|m| m.passed_by)
                    .is_some_and(|(by, at)| by == id && now - at < PASS_BACK);
                let idle = memory
                    .get(&id)
                    .and_then(|m| m.last_at)
                    .is_some_and(|(_, at)| now - at > IDLE_MATE);
                team == state.team
                    && id != bot
                    && slot != KEEPER_SLOT
                    && !passed_back
                    && !idle
                    && in_reach(p)
            });
            let best = mates
                .map(|&(id, _, _, p)| {
                    let rating = soccer_eval::pass_rating(
                        flat(at),
                        flat(p),
                        flat(goal),
                        &opponents,
                        OPPORTUNISM,
                    );
                    (id, p, rating)
                })
                .max_by(|a, b| a.2.total_cmp(&b.2));
            let free = soccer_eval::situation(flat(at), &opponents);
            if let Some((mate, mate_at, rating)) = best
                && rating > free + PASS_MARGIN
            {
                let thrown = pass(authority, bot, mate, mate_at, strength);
                if thrown.is_some() {
                    memory.entry(mate).or_default().passed_by = Some((bot, now));
                }
                events.extend(thrown);
                continue;
            }
            // Dribble: straight at goal or 45 degrees off it, whichever leaves it freest a few
            // metres on (their AI_GetBestDribbleMovement, cut down; R).
            let heading = toward(at, goal, 1.0).1;
            let (sin, cos) = std::f32::consts::FRAC_PI_4.sin_cos();
            let turns = [
                heading,
                [
                    heading[0] * cos - heading[1] * sin,
                    heading[0] * sin + heading[1] * cos,
                ],
                [
                    heading[0] * cos + heading[1] * sin,
                    -heading[0] * sin + heading[1] * cos,
                ],
            ];
            let score = |d: [f32; 2]| {
                let ahead = [at[0] + d[0] * DRIBBLE_LOOK, at[2] + d[1] * DRIBBLE_LOOK];
                let goalward = d[0] * heading[0] + d[1] * heading[1];
                // As theirs: space past the touchline is no space.
                let over = ((ahead[0] - c[0]).abs() - bounds.half_x).max(0.0)
                    + ((ahead[1] - c[2]).abs() - bounds.half_z).max(0.0);
                soccer_eval::situation(ahead, &opponents) + DRIBBLE_GOALWARD * goalward - over
            };
            // Re-picked every SUPPORT_REPLAN, like a support spot: near-equal choices flipped
            // the stick every tick.
            let mind = memory.entry(bot).or_default();
            let way = match mind.dribble {
                Some((way, picked)) if now - picked < SUPPORT_REPLAN => way,
                _ => {
                    let way = turns
                        .into_iter()
                        .max_by(|a, b| score(*a).total_cmp(&score(*b)))
                        .unwrap_or(heading);
                    mind.dribble = Some((way, now));
                    way
                }
            };
            let target = [
                at[0] + way[0] * DRIBBLE_LOOK,
                at[1],
                at[2] + way[1] * DRIBBLE_LOOK,
            ];
            let (next, facing) = toward(at, target, RUN_SPEED * pace * dt);
            let (next, facing) = if authority.place_server_player(bot, next).is_ok() {
                (next, facing)
            } else {
                let inside = toward(at, goal, RUN_SPEED * pace * dt);
                authority.place_server_player(bot, inside.0)?;
                inside
            };
            events.push(BotEvent::Moved {
                player: bot,
                position: next,
                facing,
                speed: RUN_SPEED * pace,
            });
            continue;
        }

        let chasing = chaser[(state.team & 1) as usize].is_some_and(|(id, _)| id == bot);
        // An opponent has the ball: the chaser closes in and slide-tackles.
        if let (Some(carrier), Some(team)) = (holder, holder_team) {
            if team != state.team && chasing {
                let carrier_at = roster
                    .iter()
                    .find(|r| r.0 == carrier)
                    .map(|r| r.3)
                    .unwrap_or(ball);
                // L955: a carrier that cannot be tackled yet is shadowed, not run into.
                let open = tackleable(memory, carrier, now);
                let gap = flat_distance(at, carrier_at);
                let mind = memory.entry(bot).or_default();
                mind.shadowing =
                    !open && (gap <= SHADOW || (mind.shadowing && gap <= SHADOW + SHADOW_SLACK));
                if mind.shadowing {
                    events.push(stopped(bot, at, carrier_at));
                    continue;
                }
                let speed = if open { SPRINT_SPEED } else { RUN_SPEED } * pace;
                let (next, facing) = toward(at, carrier_at, speed * dt);
                authority.place_server_player(bot, next)?;
                events.push(BotEvent::Moved {
                    player: bot,
                    position: next,
                    facing,
                    speed,
                });
                let ready = memory.get(&bot).map_or(0.0, |m| m.tackle_ready_at);
                let in_reach = flat_distance(next, carrier_at) <= TACKLE_REACH;
                if open && now >= ready && in_reach {
                    // L955: an attempt; it lands TACKLE_LANDS of the time (repeatable per bot and clock).
                    memory.entry(bot).or_default().tackle_ready_at = now + TACKLE_COOLDOWN;
                    let landed = if aim_roll(bot, now) < TACKLE_LANDS {
                        authority.tackle(bot, TACKLE_REACH)?
                    } else {
                        None
                    };
                    if let Some(victim) = landed {
                        // The carrier goes down, so it cannot chase and tackle straight back.
                        won_by_tackle(memory, bot, victim, now);
                        events.push(BotEvent::Tackled {
                            tackler: bot,
                            victim,
                        });
                    }
                }
                continue;
            }
        }
        // A loose ball: the chaser runs to it (the match gives possession on contact).
        if holder.is_none() && chasing {
            let (next, facing) = toward(at, ball, RUN_SPEED * pace * dt);
            authority.place_server_player(bot, next)?;
            events.push(BotEvent::Moved {
                player: bot,
                position: next,
                facing,
                speed: RUN_SPEED * pace,
            });
            continue;
        }
        // L956: team shape. Defending, a bot marks goal-side the nearest opponent nobody marks yet
        // (the carrier is the chaser's); otherwise it holds its kickoff spot moved with the ball,
        // along the pitch (further when its side has the ball) and a little toward the ball's
        // side (League-Soccer's marking and adapted formation, cut down). Before, the bots stood
        // off the play and nobody tracked back.
        // A bot keeps its man while its side defends (re-picking the nearest each tick swapped
        // men and flapped the stick).
        let defending = holder_team.is_some_and(|team| team != state.team);
        let kept = memory.get(&bot).and_then(|m| m.mark);
        let markable = |&&(id, team, slot, _): &&(u64, u32, u32, [f32; 3])| {
            defending
                && team != state.team
                && slot != KEEPER_SLOT
                && Some(id) != holder
                && (Some(id) == kept || !marked.contains(&id))
        };
        let marking = roster
            .iter()
            .filter(markable)
            .find(|r| Some(r.0) == kept)
            .or_else(|| {
                roster
                    .iter()
                    .filter(markable)
                    .min_by(|a, b| flat_distance(a.3, at).total_cmp(&flat_distance(b.3, at)))
            })
            .map(|&(id, _, _, p)| (id, p));
        let mind = memory.entry(bot).or_default();
        mind.mark = marking.map(|(id, _)| id);
        let target = if let Some((opponent, there)) = marking {
            marked.insert(opponent);
            let own_goal = authority.attack_goal(state.team ^ 1);
            let goal_side = toward(there, own_goal, 1.0).1;
            [
                there[0] + goal_side[0] * MARK_GAP,
                at[1],
                there[2] + goal_side[1] * MARK_GAP,
            ]
        } else {
            let spot = home(authority, state.team, state.slot);
            let push = if holder_team == Some(state.team) {
                ATTACK_PUSH
            } else {
                SHAPE_PUSH
            };
            [
                spot[0] + (ball[0] - spot[0]) * push,
                at[1],
                spot[2] + (ball[2] - spot[2]) * SIDE_PULL,
            ]
        };
        // Half a metre inside the touchlines (a pitch under a metre wide has no inside).
        let inner = [(bounds.half_x - 0.5).max(0.0), (bounds.half_z - 0.5).max(0.0)];
        let target = [
            target[0].clamp(c[0] - inner[0], c[0] + inner[0]),
            target[1],
            target[2].clamp(c[2] - inner[1], c[2] + inner[1]),
        ];
        // Re-planned every SUPPORT_REPLAN: a spot that moved with the ball or a man each tick
        // swung the bot's heading (and its stick) back and forth.
        let target = match mind.support_spot {
            Some((spot, at_time)) if now - at_time < SUPPORT_REPLAN => spot,
            _ => {
                mind.support_spot = Some((target, now));
                target
            }
        };
        // L955: re-form only past the deadband, and finish the run once started (no churn).
        let away = flat_distance(at, target);
        if away > SUPPORT_DEADBAND {
            mind.support_run = true;
        } else if away <= SUPPORT_ARRIVE {
            mind.support_run = false;
        }
        if mind.support_run {
            let (next, facing) = toward(at, target, SUPPORT_SPEED * pace * dt);
            authority.place_server_player(bot, next)?;
            events.push(BotEvent::Moved {
                player: bot,
                position: next,
                facing,
                speed: SUPPORT_SPEED * pace,
            });
        } else {
            events.push(stopped(bot, at, ball));
        }
    }
    Ok(events)
}

/// L949: a repeatable 0..1 per shot (the bot and the match clock), for its aim.
fn aim_roll(bot: u64, now: f32) -> f32 {
    (now * 7.77 + (bot % 13) as f32 * 0.618).fract()
}

/// L949: a free ball on its way into the goal whose keeper stands on `line_x`: the time until it
/// reaches the line, where it crosses and how high. None when it is held, going away or wide.
fn incoming(authority: &Authority, line_x: f32) -> Option<(f32, f32, f32)> {
    if authority.possession().is_some() {
        return None;
    }
    let ball = authority.ball_position();
    let velocity = authority.ball_velocity();
    let time = (line_x - ball[0]) / velocity[0];
    if !(time.is_finite() && time > 0.0) {
        return None;
    }
    let bounds = authority.config().bounds;
    let z = ball[2] + velocity[2] * time;
    let height = (ball[1] + velocity[1] * time - bounds.centre[1]).max(0.0);
    ((z - bounds.centre[2]).abs() <= bounds.goal_half_width).then_some((time, z, height))
}

/// L949: a block's state by the keeper's side (positive is its right) and the ball's height (R).
fn block_state(side: f32, height: f32) -> u8 {
    if side.abs() < 0.5 {
        return BLOCK_CENTER;
    }
    BLOCK_HIGH_LEFT + u8::from(side > 0.0) + 2 * u8::from(height < 1.2)
}

/// L949: a catch's state by the ball's height: high, mid or low (live: a ball at 0.25 was CatchLow).
fn catch_state(height: f32) -> u8 {
    CATCH_HIGH + u8::from(height <= 1.6) + u8::from(height <= 0.6)
}

/// Aim inside the post away from the defending keeper (R: a shot at the goal centre is the keeper's).
/// L949: how far inside varies shot to shot by `roll` (0..1), from `AIM_MIN` of the way to the post
/// out to it, so a keeper reaches some (live shots crossed near the keeper; R).
fn far_post(
    authority: &Authority,
    roster: &[(u64, u32, u32, [f32; 3])],
    team: u32,
    goal: [f32; 3],
    roll: f32,
) -> [f32; 3] {
    let inset = (authority.config().bounds.goal_half_width - 1.5).max(0.0)
        * (AIM_MIN + (1.0 - AIM_MIN) * roll);
    let keeper_z = roster
        .iter()
        .find(|&&(_, t, slot, _)| t != team && slot == KEEPER_SLOT)
        .map_or(goal[2], |r| r.3[2]);
    let side = if keeper_z >= goal[2] { -1.0 } else { 1.0 };
    [goal[0], goal[1], goal[2] + side * inset]
}

/// A bot that holds still: one speed-0 pose so the client stops extrapolating its last run.
fn stopped(player: u64, at: [f32; 3], ball: [f32; 3]) -> BotEvent {
    BotEvent::Moved {
        player,
        position: at,
        facing: toward(at, ball, 0.0).1,
        speed: 0.0,
    }
}

/// A shot at `at`. The match's own rules decide; a refused kick (the ball moved on) is no kick.
/// `strength` is the bot's Kick Power against its base (L944).
fn kick(authority: &mut Authority, bot: u64, at: [f32; 3], strength: f32) -> Option<BotEvent> {
    let from = authority.ball_position();
    let direction = sub([at[0], from[1] + 0.5, at[2]], from);
    bot_kick(authority, bot, KickKind::Shot, direction, SHOT_POWER * strength, None)
}

fn pass(
    authority: &mut Authority,
    bot: u64,
    mate: u64,
    mate_at: [f32; 3],
    strength: f32,
) -> Option<BotEvent> {
    let direction = sub(mate_at, authority.ball_position());
    bot_kick(
        authority,
        bot,
        KickKind::Pass,
        direction,
        PASS_POWER * strength,
        Some(mate),
    )
}

fn bot_kick(
    authority: &mut Authority,
    bot: u64,
    kind: KickKind,
    direction: [f32; 3],
    power: f32,
    target: Option<u64>,
) -> Option<BotEvent> {
    let power = power.min(authority.config().physics.max_kick_power);
    let sequence = authority.player(bot).map_or(0, |state| state.sequence) + 1;
    let now = authority.server_time();
    let kick = Intent::Kick {
        player: bot,
        sequence,
        kind,
        direction,
        power,
        target,
    };
    authority.apply(bot, now, kick).ok()?;
    Some(BotEvent::Kicked {
        player: bot,
        kind,
        power,
    })
}
