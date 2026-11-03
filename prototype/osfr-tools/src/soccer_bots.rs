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
/// A ball carrier shoots inside this distance of the goal (R).
pub const SHOOT_RANGE: f32 = 14.0;
/// Shot and pass power (R; the match caps kicks at `max_kick_power`).
pub const SHOT_POWER: f32 = 22.0;
pub const PASS_POWER: f32 = 14.0;
/// An opponent this close puts the carrier under pressure, so it passes forward (R).
pub const PRESSURE_RANGE: f32 = 5.0;
/// Seconds between one bot's tackles (R), so a tackle can be escaped.
pub const TACKLE_COOLDOWN: f32 = 1.5;
/// Seconds a slide-tackled bot stays down (R; the Soccer Star Toughness stat shortens it).
pub const KNOCKDOWN: f32 = 1.5;
/// The keeper slot (live: the fourth server player of each side stood on the goal line).
pub const KEEPER_SLOT: u32 = TEAM_SLOT_COUNT - 1;

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
    // The field player of each side nearest the ball goes for it; the rest support.
    let mut chaser = [None::<(u64, f32)>; 2];
    for (id, state) in authority.players() {
        let (Some(position), true) = (state.position, state.slot != KEEPER_SLOT) else {
            continue;
        };
        // A knocked-down bot cannot chase; the next nearest teammate goes instead.
        if !bots.contains(&id) || memory.get(&id).is_some_and(|m| now < m.down_until) {
            continue;
        }
        let distance = flat_distance(position, ball);
        let side = &mut chaser[(state.team & 1) as usize];
        if side.is_none_or(|(_, best)| distance < best) {
            *side = Some((id, distance));
        }
    }
    let roster: Vec<(u64, u32, u32, [f32; 3])> = authority
        .players()
        .filter_map(|(id, s)| s.position.map(|p| (id, s.team, s.slot, p)))
        .collect();
    let mut events = Vec::new();
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
        let c = authority.config().bounds.centre;
        // Read possession per bot: an earlier bot this step may have tackled or kicked.
        let holder = authority.possession();
        let holder_team = holder.and_then(|id| authority.player(id)).map(|s| s.team);
        let in_reach = |mate_at: [f32; 3]| flat_distance(mate_at, at) <= pass_radius;

        // Keeper: slide along the own goal line with the ball.
        if state.slot == KEEPER_SLOT {
            let line = home(authority, state.team, state.slot);
            let target = [
                line[0],
                at[1],
                (ball[2]).clamp(c[2] - KEEPER_REACH, c[2] + KEEPER_REACH),
            ];
            let (next, facing) = toward(at, target, KEEPER_SPEED * pace * dt);
            if next != at {
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
            // A keeper with the ball distributes it to the nearest teammate (R).
            if holder == Some(bot) {
                let mate = roster
                    .iter()
                    .filter(|&&(id, team, _, p)| team == state.team && id != bot && in_reach(p))
                    .min_by(|a, b| flat_distance(a.3, at).total_cmp(&flat_distance(b.3, at)));
                if let Some(&(mate, _, _, mate_at)) = mate {
                    events.extend(pass(authority, bot, mate, mate_at, strength));
                }
            }
            continue;
        }

        // Carrier: shoot, pass round a blocker, or dribble at goal.
        if holder == Some(bot) {
            if flat_distance(at, goal) <= SHOOT_RANGE {
                events.extend(kick(
                    authority,
                    bot,
                    far_post(authority, &roster, state.team, goal),
                    strength,
                ));
                continue;
            }
            // Under pressure (an opponent close by), pass forward to the teammate nearest the goal.
            let pressed = roster.iter().any(|&(_, team, _, p)| {
                team != state.team && flat_distance(p, at) <= PRESSURE_RANGE
            });
            if pressed {
                let ahead = flat_distance(at, goal);
                let open = roster
                    .iter()
                    .filter(|&&(id, team, _, p)| {
                        team == state.team
                            && id != bot
                            && in_reach(p)
                            && flat_distance(p, goal) < ahead
                    })
                    .min_by(|a, b| flat_distance(a.3, goal).total_cmp(&flat_distance(b.3, goal)));
                if let Some(&(mate, _, _, mate_at)) = open {
                    events.extend(pass(authority, bot, mate, mate_at, strength));
                    continue;
                }
            }
            let (next, facing) = toward(at, goal, RUN_SPEED * pace * dt);
            authority.place_server_player(bot, next)?;
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
                let (next, facing) = toward(at, carrier_at, SPRINT_SPEED * pace * dt);
                authority.place_server_player(bot, next)?;
                events.push(BotEvent::Moved {
                    player: bot,
                    position: next,
                    facing,
                    speed: SPRINT_SPEED * pace,
                });
                let ready = memory.get(&bot).map_or(0.0, |m| m.tackle_ready_at);
                if now >= ready {
                    if let Some(victim) = authority.tackle(bot)? {
                        memory.entry(bot).or_default().tackle_ready_at = now + TACKLE_COOLDOWN;
                        // The carrier goes down, so it cannot chase and tackle straight back.
                        knock_down(memory, bot, victim, now);
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
        // Support: the kickoff spot, shifted halfway toward the ball along the pitch.
        let spot = home(authority, state.team, state.slot);
        let target = [spot[0] + (ball[0] - spot[0]) * 0.5, at[1], spot[2]];
        if flat_distance(at, target) > 0.5 {
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

/// Aim inside the post away from the defending keeper (R: a shot at the goal centre is the keeper's).
fn far_post(
    authority: &Authority,
    roster: &[(u64, u32, u32, [f32; 3])],
    team: u32,
    goal: [f32; 3],
) -> [f32; 3] {
    let inset = (authority.config().bounds.goal_half_width - 1.5).max(0.0);
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
