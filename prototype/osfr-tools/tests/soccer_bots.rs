//! L932: the server's soccer players (`soccer_bots`) on the live pitch (`data/content/soccer.json`
//! geometry: centre (226.7, 0, 212.9), half_x 27, half_z 17.5, goal mouth 8).
use std::collections::{BTreeMap, BTreeSet};

use osfr_tools::soccer_bots::{self, BotEvent, KEEPER_REACH, KEEPER_SLOT};
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
        assert!((at[2] - CENTRE[2]).abs() <= KEEPER_REACH + 0.01, "{at:?}");
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
