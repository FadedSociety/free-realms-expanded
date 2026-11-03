//! L932: the server's soccer players (`soccer_bots`) on the live pitch (`data/content/soccer.json`
//! geometry: centre (226.7, 0, 212.9), half_x 27, half_z 17.5, goal mouth 8).
use std::collections::{BTreeMap, BTreeSet};

use osfr_tools::soccer_bots::{self, BotEvent, KEEPER_SLOT};
use osfr_tools::soccer_match::{Authority, FieldBounds, Intent, KickKind, MatchConfig, MatchPhase};

const HUMAN: u64 = 0x10;
const CENTRE: [f32; 3] = [226.7, 0.0, 212.9];

fn config() -> MatchConfig {
    let mut config = MatchConfig::new(FieldBounds {
        centre: CENTRE,
        half_x: 27.0,
        half_z: 17.5,
        goal_half_width: 8.0,
        goal_height: 5.0,
    });
    config.bot_ai = true;
    config
}

/// The human on team 0 slot 0, bots in every other slot (the solo 4v4 fill).
fn match_with_bots() -> (Authority, BTreeSet<u64>) {
    let mut game = Authority::new(config()).unwrap();
    game.register(HUMAN, 0, 0).unwrap();
    let mut bots = BTreeSet::new();
    for team in 0..2u32 {
        for slot in 0..4u32 {
            if (team, slot) == (0, 0) {
                continue;
            }
            let bot = 0x100 + u64::from(team * 4 + slot);
            game.register(bot, team, slot).unwrap();
            bots.insert(bot);
        }
    }
    game.kickoff().unwrap();
    (game, bots)
}

fn place_human(game: &mut Authority, at: [f32; 3], sequence: u64) {
    let now = game.server_time();
    game.apply(
        HUMAN,
        now,
        Intent::Position {
            player: HUMAN,
            sequence,
            position: at,
        },
    )
    .unwrap();
}

#[test]
fn bots_line_up_on_the_live_kickoff_spots() {
    let (mut game, bots) = match_with_bots();
    let events = soccer_bots::line_up(&mut game, &bots).unwrap();
    assert_eq!(events.len(), 7, "every bot gets a pose");
    // Live kickoff frame 6451: the other side's keeper stood at (200.9, 212.9) and its field players
    // at (216.2, 204.0) and (216.2, 221.9); the human's side's keeper at (252.5, 212.9).
    let at = |id| game.player(id).unwrap().position.unwrap();
    let keeper_1 = at(0x100 + 4 + u64::from(KEEPER_SLOT));
    let keeper_0 = at(0x100 + u64::from(KEEPER_SLOT));
    assert!(
        (keeper_1[0] - 200.9).abs() < 0.01 && (keeper_1[2] - 212.9).abs() < 0.01,
        "{keeper_1:?}"
    );
    assert!((keeper_0[0] - 252.5).abs() < 0.01, "{keeper_0:?}");
    assert!((at(0x105)[0] - 216.2).abs() < 0.01 && (at(0x105)[2] - 204.0).abs() < 0.01);
    assert!(
        events
            .iter()
            .all(|e| matches!(e, BotEvent::Moved { speed, .. } if *speed == 0.0))
    );
}

/// The bots play a real match around an idle human: they run, tackle (the tackled carrier goes
/// down), pass forward under pressure and score within two minutes.
#[test]
fn bots_play_a_match_around_a_player_who_stands_still() {
    let (mut game, bots) = match_with_bots();
    soccer_bots::line_up(&mut game, &bots).unwrap();
    // The human stands off the ball (a first position is bounded from the kickoff spot) and never
    // touches it again.
    place_human(&mut game, [CENTRE[0] + 6.0, 0.0, CENTRE[2] + 6.0], 1);
    let mut memory = BTreeMap::new();
    let mut moved = BTreeSet::new();
    let (mut tackles, mut passes) = (0, 0);
    for _ in 0..(120 * 4) {
        if game.phase() == MatchPhase::Restart {
            game.kickoff().unwrap();
            soccer_bots::line_up(&mut game, &bots).unwrap();
        }
        if game.phase() != MatchPhase::Running {
            break;
        }
        for event in soccer_bots::step(&mut game, &bots, &mut memory, 0.25).unwrap() {
            match event {
                BotEvent::Moved { player, speed, .. } if speed > 0.0 => {
                    moved.insert(player);
                }
                BotEvent::Tackled { .. } => tackles += 1,
                BotEvent::Kicked {
                    kind: KickKind::Pass,
                    ..
                } => passes += 1,
                _ => {}
            }
        }
        game.advance(0.25).unwrap();
    }
    assert!(moved.len() >= 4, "the bots run: {moved:?}");
    assert!(
        tackles > 0 && passes > 0,
        "tackles {tackles}, passes {passes}"
    );
    let score = game.score();
    assert!(score[0] + score[1] >= 1, "the bots score: {score:?}");
}

#[test]
fn a_bot_slide_tackle_steals_the_ball_and_knocks_the_carrier_down() {
    let (mut game, bots) = match_with_bots();
    soccer_bots::line_up(&mut game, &bots).unwrap();
    // The human takes the loose ball at the kickoff spot, next to the other side's centre forward.
    place_human(&mut game, CENTRE, 1);
    game.advance(0.02).unwrap();
    assert_eq!(game.possession(), Some(HUMAN));
    let mut memory = BTreeMap::new();
    let mut tackled = None;
    for _ in 0..40 {
        for event in soccer_bots::step(&mut game, &bots, &mut memory, 0.25).unwrap() {
            if let BotEvent::Tackled { victim, tackler } = event {
                assert_eq!(victim, HUMAN);
                assert_eq!(game.player(tackler).unwrap().team, 1, "an opponent tackles");
                tackled = Some(tackler);
            }
        }
        if tackled.is_some() {
            break;
        }
        game.advance(0.25).unwrap();
    }
    let tackler = tackled.expect("the nearest opponent closes in and tackles");
    assert_eq!(
        game.possession(),
        Some(tackler),
        "the tackle steals the ball"
    );
}

/// Review fix: a tackled bot waits out the cooldown too, so the ball cannot ping-pong between
/// two bots on consecutive steps.
#[test]
fn a_tackled_bot_cannot_tackle_straight_back() {
    let (mut game, bots) = match_with_bots();
    soccer_bots::line_up(&mut game, &bots).unwrap();
    let (carrier, chaser) = (0x105, 0x101);
    game.place_server_player(carrier, CENTRE).unwrap();
    game.advance(0.02).unwrap();
    assert_eq!(game.possession(), Some(carrier));
    game.place_server_player(chaser, [CENTRE[0] + 1.0, 0.0, CENTRE[2]])
        .unwrap();
    let mut memory = BTreeMap::new();
    let mut tackles = Vec::new();
    // Five 0.25 s steps stay inside the 1.5 s cooldown.
    for _ in 0..5 {
        for event in soccer_bots::step(&mut game, &bots, &mut memory, 0.25).unwrap() {
            if let BotEvent::Tackled { tackler, victim } = event {
                tackles.push((tackler, victim));
            }
        }
        if game.phase() != MatchPhase::Running {
            break;
        }
        game.advance(0.25).unwrap();
    }
    assert_eq!(
        tackles.first(),
        Some(&(chaser, carrier)),
        "the chaser takes the ball"
    );
    assert!(
        !tackles.iter().any(|&(tackler, _)| tackler == carrier),
        "the victim does not tackle back inside the cooldown: {tackles:?}"
    );
}

#[test]
fn a_keeper_stays_on_its_line_and_tracks_the_ball_within_reach() {
    let (mut game, bots) = match_with_bots();
    soccer_bots::line_up(&mut game, &bots).unwrap();
    let keeper = 0x100 + 4 + u64::from(KEEPER_SLOT);
    let line = game.player(keeper).unwrap().position.unwrap()[0];
    // Live play moves the ball about: the keeper slides with it but never leaves its line or reach.
    place_human(&mut game, [CENTRE[0] + 6.0, 0.0, CENTRE[2] + 6.0], 1);
    let mut memory = BTreeMap::new();
    let mut moved = false;
    for _ in 0..120 {
        soccer_bots::step(&mut game, &bots, &mut memory, 0.25).unwrap();
        let at = game.player(keeper).unwrap().position.unwrap();
        assert!((at[0] - line).abs() < 0.01, "the keeper holds its line");
        // L949: a shot on goal widens the keeper's reach to the goal mouth.
        assert!((at[2] - CENTRE[2]).abs() <= 8.0 + 0.01, "{at:?}");
        moved |= (at[2] - CENTRE[2]).abs() > 0.01;
        if game.phase() == MatchPhase::Restart {
            game.kickoff().unwrap();
            soccer_bots::line_up(&mut game, &bots).unwrap();
        }
        if game.phase() != MatchPhase::Running {
            break;
        }
        game.advance(0.25).unwrap();
    }
    assert!(moved, "the keeper slides with the ball");
}

/// L949: a team 0 bot shoots from 10 units out at the goal team 1's keeper guards, `offset` across
/// from the keeper, with `power`; only the keeper plays.
fn shot_at_keeper(offset: f32, power: f32) -> (Authority, u64, Vec<BotEvent>) {
    let (mut game, bots) = match_with_bots();
    soccer_bots::line_up(&mut game, &bots).unwrap();
    let (shooter, keeper) = (0x101, 0x100 + 4 + u64::from(KEEPER_SLOT));
    game.place_server_player(shooter, CENTRE).unwrap();
    game.advance(0.02).unwrap();
    assert_eq!(game.possession(), Some(shooter));
    let goal = game.attack_goal(0);
    let guard = game.player(keeper).unwrap().position.unwrap();
    let spot = [goal[0] - game.attack_sign(0) * 10.0, 0.0, guard[2]];
    game.place_server_player(shooter, spot).unwrap();
    game.advance(0.02).unwrap();
    let from = game.ball_position();
    let now = game.server_time();
    let kick = Intent::Kick {
        player: shooter,
        sequence: 1,
        kind: KickKind::Shot,
        direction: [guard[0] - from[0], 0.5, guard[2] + offset - from[2]],
        power,
        target: None,
    };
    game.apply(shooter, now, kick).unwrap();
    let keepers = BTreeSet::from([keeper]);
    let mut memory = BTreeMap::new();
    let mut events = Vec::new();
    for _ in 0..80 {
        if game.phase() != MatchPhase::Running {
            break;
        }
        events.extend(soccer_bots::step(&mut game, &keepers, &mut memory, 0.05).unwrap());
        game.advance(0.05).unwrap();
    }
    (game, keeper, events)
}

fn goalie_states(events: &[BotEvent]) -> Vec<u8> {
    events
        .iter()
        .filter_map(|event| match event {
            BotEvent::Goalie { state, .. } => Some(*state),
            _ => None,
        })
        .collect()
}

fn kicked_by(events: &[BotEvent], keeper: u64) -> bool {
    events
        .iter()
        .any(|event| matches!(event, BotEvent::Kicked { player, .. } if *player == keeper))
}

/// L949: as live (a KickRight caught: CatchLow 60, the ball in hand 61, Throw 62), a keeper dives at
/// a shot it can reach, catches it, holds it and throws it to a teammate.
#[test]
fn a_keeper_catches_a_shot_it_can_reach_and_throws_it_out() {
    let (game, keeper, events) = shot_at_keeper(1.5, 20.0);
    assert_eq!(game.score(), [0, 0], "saved");
    let states = goalie_states(&events);
    assert!((58..=60).contains(&states[0]), "a catch: {states:?}");
    assert_eq!(states[1..], [61, 62], "the ball in hand, then the throw");
    assert!(kicked_by(&events, keeper), "thrown to a teammate");
    assert_ne!(game.possession(), Some(keeper));
}

/// L949: a full-power shot is blocked (live SuperKicks were blocked or went in) and cleared at once.
#[test]
fn a_full_power_shot_is_blocked_and_cleared() {
    let (game, keeper, events) = shot_at_keeper(1.5, 30.0);
    assert_eq!(game.score(), [0, 0], "saved");
    let states = goalie_states(&events);
    assert_eq!(states.len(), 1, "{states:?}");
    assert!(
        (51..=54).contains(&states[0]) || states[0] == 57,
        "a block: {states:?}"
    );
    assert!(kicked_by(&events, keeper), "cleared");
}

/// L955: a keeper saves where it stands, as live: its dive widens its reach instead of moving it
/// (it used to jump, then slide, 3 m across to the shot's line: the owner saw it teleport).
#[test]
fn a_keeper_saves_without_leaping_across() {
    let (game, keeper, events) = shot_at_keeper(2.5, 20.0);
    assert_eq!(game.score(), [0, 0], "saved");
    let steps: Vec<[f32; 3]> = events
        .iter()
        .filter_map(|event| match event {
            BotEvent::Moved {
                player, position, ..
            } if *player == keeper => Some(*position),
            _ => None,
        })
        .collect();
    let longest = steps
        .windows(2)
        .map(|w| (w[1][0] - w[0][0]).hypot(w[1][2] - w[0][2]))
        .fold(0.0f32, f32::max);
    assert!(
        longest <= soccer_bots::KEEPER_SPEED * 0.05 + 1e-3,
        "keeper stepped {longest} m in one 0.05 s step"
    );
}

/// L955: a pass's target runs to meet the ball and takes it. Before, only the team's nearest bot
/// moved and the target stood still (owner: passes never reached a team-mate).
#[test]
fn a_pass_target_runs_to_meet_the_ball() {
    let (mut game, bots) = match_with_bots();
    soccer_bots::line_up(&mut game, &bots).unwrap();
    place_human(&mut game, CENTRE, 1);
    game.advance(0.02).unwrap();
    assert_eq!(game.possession(), Some(HUMAN));
    let target = 0x102;
    let start = game.player(target).unwrap().position.unwrap();
    let now = game.server_time();
    let pass = Intent::Kick {
        player: HUMAN,
        sequence: 2,
        kind: KickKind::Pass,
        direction: [start[0] - CENTRE[0], 0.0, start[2] - CENTRE[2]],
        power: 6.0,
        target: Some(target),
    };
    game.apply(HUMAN, now, pass).unwrap();
    let mut memory = BTreeMap::new();
    for _ in 0..80 {
        if game.possession().is_some() || game.phase() != MatchPhase::Running {
            break;
        }
        soccer_bots::step(&mut game, &bots, &mut memory, 0.05).unwrap();
        game.advance(0.05).unwrap();
    }
    assert_eq!(game.possession(), Some(target), "the pass reached its target");
    let at = game.player(target).unwrap().position.unwrap();
    assert!(
        (at[0] - start[0]).hypot(at[2] - start[2]) > 0.5,
        "the target moved to meet the ball"
    );
}

/// L949: a shot past the keeper's dive goes in.
#[test]
fn a_shot_beyond_the_keepers_dive_goes_in() {
    let (game, _, events) = shot_at_keeper(6.5, 20.0);
    assert_eq!(game.score(), [1, 0]);
    assert!(goalie_states(&events).is_empty(), "out of reach: no dive");
}

/// L949: a keeper's ball is in its hands: a tackle cannot take it.
#[test]
fn a_keepers_ball_cannot_be_tackled() {
    let (mut game, bots) = match_with_bots();
    soccer_bots::line_up(&mut game, &bots).unwrap();
    let (keeper, opponent) = (0x100 + 4 + u64::from(KEEPER_SLOT), 0x101);
    game.place_server_player(keeper, CENTRE).unwrap();
    game.advance(0.02).unwrap();
    assert_eq!(game.possession(), Some(keeper));
    game.place_server_player(opponent, [CENTRE[0] + 1.0, 0.0, CENTRE[2]])
        .unwrap();
    assert_eq!(
        game.tackle(opponent, soccer_bots::TACKLE_REACH).unwrap(),
        None
    );
    assert_eq!(game.possession(), Some(keeper));
}

/// L955: a bot tackles only inside the client's own tackle range (1.73 m), not the 2 m control
/// radius: bots stole from behind the player (owner, native 2026-11-03).
#[test]
fn a_tackle_reaches_only_the_clients_tackle_range() {
    let (mut game, bots) = match_with_bots();
    soccer_bots::line_up(&mut game, &bots).unwrap();
    let (carrier, tackler) = (0x105, 0x101);
    game.place_server_player(carrier, CENTRE).unwrap();
    game.advance(0.02).unwrap();
    assert_eq!(game.possession(), Some(carrier));
    game.place_server_player(tackler, [CENTRE[0] + 1.9, 0.0, CENTRE[2]])
        .unwrap();
    assert_eq!(
        game.tackle(tackler, soccer_bots::TACKLE_REACH).unwrap(),
        None
    );
    game.place_server_player(tackler, [CENTRE[0] + 1.5, 0.0, CENTRE[2]])
        .unwrap();
    assert_eq!(
        game.tackle(tackler, soccer_bots::TACKLE_REACH).unwrap(),
        Some(carrier)
    );
}

#[test]
fn passive_bots_without_bot_ai_never_move() {
    let mut config = config();
    config.bot_ai = false;
    let mut game = Authority::new(config).unwrap();
    game.register(HUMAN, 0, 0).unwrap();
    game.register(0x101, 0, 1).unwrap();
    game.kickoff().unwrap();
    let bots = BTreeSet::from([0x101]);
    assert!(soccer_bots::line_up(&mut game, &bots).unwrap().is_empty());
    assert!(
        soccer_bots::step(&mut game, &bots, &mut BTreeMap::new(), 0.25)
            .unwrap()
            .is_empty()
    );
    assert_eq!(game.player(0x101).unwrap().position, None);
}

/// L944: Toughness (76/19 word 3) gets a player back up quicker: twice the base, half the knockdown.
#[test]
fn toughness_shortens_the_knockdown_a_tackle_gives() {
    let (mut game, bots) = match_with_bots();
    soccer_bots::line_up(&mut game, &bots).unwrap();
    place_human(&mut game, CENTRE, 1);
    game.advance(0.02).unwrap();
    let mut memory = BTreeMap::new();
    let tough = soccer_bots::Stats {
        toughness: 2 * soccer_bots::BASE_STATS.toughness,
        ..soccer_bots::BASE_STATS
    };
    memory.insert(
        HUMAN,
        soccer_bots::BotMemory {
            stats: Some(tough),
            ..Default::default()
        },
    );
    for _ in 0..40 {
        let now = game.server_time();
        let events = soccer_bots::step(&mut game, &bots, &mut memory, 0.25).unwrap();
        if events.iter().any(|event| matches!(event, BotEvent::Tackled { .. })) {
            let down = memory[&HUMAN].down_until - now;
            assert!((down - soccer_bots::KNOCKDOWN / 2.0).abs() < 1e-4, "{down}");
            return;
        }
        game.advance(0.25).unwrap();
    }
    panic!("the nearest opponent closes in and tackles");
}

/// L955: the player's Ctrl juke (client states 4-6) dodges tackles while it lasts; once it stops,
/// the nearest opponent's tackle lands again (owner playtest 2026-11-03: dodge did nothing).
#[test]
fn a_juke_dodges_for_a_moment_and_spamming_it_does_not_last() {
    let (mut game, bots) = match_with_bots();
    soccer_bots::line_up(&mut game, &bots).unwrap();
    place_human(&mut game, CENTRE, 1);
    game.advance(0.02).unwrap();
    assert_eq!(game.possession(), Some(HUMAN));
    let mut memory = BTreeMap::new();
    let mut tackled_at = None;
    for step in 0..60 {
        // L955: jukes sent every step count once per JUKE_COOLDOWN.
        soccer_bots::juke(&mut memory, HUMAN, game.server_time());
        for event in soccer_bots::step(&mut game, &bots, &mut memory, 0.25).unwrap() {
            if let BotEvent::Tackled { victim: HUMAN, .. } = event {
                tackled_at.get_or_insert(step);
            }
        }
        if game.phase() != MatchPhase::Running {
            break;
        }
        game.advance(0.25).unwrap();
    }
    let step = tackled_at.expect("spammed jukes do not keep the carrier untackleable");
    assert!(
        step as f32 * 0.25 >= soccer_bots::DODGE_TIME,
        "no tackle while the first juke lasts (tackled at step {step})"
    );
}

/// L955: a player who just won the ball keeps it for `PROTECT`: the bots stopped stealing back
/// and forth every two seconds (owner playtest: 169 steals in a match).
#[test]
fn a_fresh_carrier_keeps_the_ball_through_the_protect_window() {
    let (mut game, bots) = match_with_bots();
    soccer_bots::line_up(&mut game, &bots).unwrap();
    let (carrier, chaser) = (0x105, 0x101);
    game.place_server_player(carrier, CENTRE).unwrap();
    game.advance(0.02).unwrap();
    game.place_server_player(chaser, [CENTRE[0] + 1.0, 0.0, CENTRE[2]])
        .unwrap();
    let mut memory = BTreeMap::new();
    let mut tackles = Vec::new();
    for _ in 0..12 {
        let now = game.server_time();
        for event in soccer_bots::step(&mut game, &bots, &mut memory, 0.25).unwrap() {
            if let BotEvent::Tackled { tackler, victim } = event {
                tackles.push((now, tackler, victim));
            }
        }
        if game.phase() != MatchPhase::Running {
            break;
        }
        game.advance(0.25).unwrap();
    }
    let &(won_at, winner, _) = tackles.first().expect("the chaser takes the ball");
    assert_eq!(winner, chaser);
    assert!(
        !tackles
            .iter()
            .any(|&(at, _, victim)| victim == chaser && at < won_at + soccer_bots::PROTECT),
        "nobody steals from the new carrier inside the protect window: {tackles:?}"
    );
}
