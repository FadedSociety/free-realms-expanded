//! L956: how a bot reads the pitch around the ball: the odds a pass gets through, how pressed a
//! player is, and how good a pass to each team-mate is. Before, a carrier ran straight at goal,
//! shot inside 14 m and passed only when an opponent was within 5 m (owner: a goal every few
//! seconds, no football).
//!
//! Adapted from League-Soccer (github.com/awest813/League-Soccer, src/onthepitch/AIsupport/
//! AIfunctions.cpp `AI_CalculatePassingOdds`, `AI_GetSituationRating`, `AI_GetPassRatings`, and
//! gamedefines.hpp `PassRating::CalculateRating`), Apache-2.0, of the GameplayFootball / Google
//! Research Football lineage. Positions are flat (x, z). Left out: their 0.3-0.5 s position
//! predictions, the passer's body-direction penalty and the off-pitch penalty (the dribble applies
//! its own).

/// Opponents within this of a player count against its situation (theirs, kept: pressure is about
/// running speed, not pitch size; halving it left a free carrier never passing, Opus check).
const SAFE_DISTANCE: f32 = 8.0;
/// A pass line is threatened by an opponent within `LANE_NEAR + LANE_GROWTH * d` of the point `d`
/// metres along it (theirs, kept: speed-bound, like `SAFE_DISTANCE`; R for our players).
const LANE_NEAR: f32 = 0.45;
const LANE_GROWTH: f32 = 0.22;
/// A pass line is checked to this length at most (no caller passes further; bounds the loop).
const LANE_MAX: f32 = 100.0;
/// Goal distances are scored inside these bounds (theirs: 8-100 m; halved for our 54 x 35 pitch
/// against their 105 x 68, R).
const GOAL_NEAR: f32 = 4.0;
const GOAL_FAR: f32 = 50.0;

fn gap(a: [f32; 2], b: [f32; 2]) -> f32 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// The odds (0..1) a pass from `from` to `to` gets past `opponents`: sampled every metre along the
/// line (to 0.96 of it), each opponent cuts it to its distance over the lane's width there.
pub fn passing_odds(from: [f32; 2], to: [f32; 2], opponents: &[[f32; 2]]) -> f32 {
    let length = gap(from, to);
    if !length.is_finite() {
        return 0.0;
    }
    let length = length.min(LANE_MAX);
    // The cast is exact: `length` is finite and at most LANE_MAX.
    let checks = 1 + length.ceil() as usize;
    let mut odds: f32 = 1.0;
    for i in 1..=checks {
        let share = i as f32 / checks as f32;
        let along = share * 0.96;
        let point = [
            from[0] + (to[0] - from[0]) * along,
            from[1] + (to[1] - from[1]) * along,
        ];
        let lane = LANE_NEAR + LANE_GROWTH * length * share;
        for &opponent in opponents {
            odds = odds.min(gap(opponent, point).min(lane) / lane);
        }
    }
    odds
}

/// How free a player at `at` is (0 pressed .. 1 free): its nearest opponent's distance against
/// `SAFE_DISTANCE`, square-rooted, as theirs.
pub fn situation(at: [f32; 2], opponents: &[[f32; 2]]) -> f32 {
    opponents
        .iter()
        .map(|&opponent| (gap(opponent, at).min(SAFE_DISTANCE) / SAFE_DISTANCE).sqrt())
        .fold(1.0, f32::min)
}

/// A pass's rating (0..1): the odds it gets through and the receiver's situation, against how much
/// nearer the goal it takes the ball. `opportunism` (0..1) weighs the gain over the safety.
pub fn pass_rating(
    from: [f32; 2],
    to: [f32; 2],
    goal: [f32; 2],
    opponents: &[[f32; 2]],
    opportunism: f32,
) -> f32 {
    let odds = passing_odds(from, to, opponents);
    let scaled = |at: [f32; 2]| gap(goal, at).clamp(GOAL_NEAR, GOAL_FAR) / GOAL_FAR;
    let position = (0.5 + (scaled(from) - scaled(to)) * 0.5).clamp(0.3, 1.0);
    let receiver = situation(to, opponents);
    (receiver.powf(0.45) + odds.powf(0.8)) * 0.5 * (1.0 - opportunism)
        + position.powf(0.7) * opportunism
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_open_lane_passes_and_a_blocker_on_it_does_not() {
        assert_eq!(passing_odds([0.0, 0.0], [10.0, 0.0], &[[5.0, 8.0]]), 1.0);
        assert!(passing_odds([0.0, 0.0], [10.0, 0.0], &[[5.0, 0.1]]) < 0.2);
        // Beside the far end of the lane threatens more than beside its near end: the defender
        // there has longer to reach the ball (the lane widens with the distance).
        let near = passing_odds([0.0, 0.0], [10.0, 0.0], &[[1.0, 0.6]]);
        let far = passing_odds([0.0, 0.0], [10.0, 0.0], &[[9.0, 0.6]]);
        assert!(far < near, "{far} {near}");
    }

    #[test]
    fn a_player_is_as_pressed_as_its_nearest_opponent_makes_it() {
        assert_eq!(situation([0.0, 0.0], &[[10.0, 0.0]]), 1.0);
        assert_eq!(situation([0.0, 0.0], &[]), 1.0);
        // sqrt(2 / 8): the nearest of several counts.
        let nearest = situation([0.0, 0.0], &[[6.0, 0.0], [0.0, 2.0], [-7.0, 0.0]]);
        assert!((nearest - 0.5).abs() < 1e-6, "{nearest}");
    }

    #[test]
    fn of_two_equally_free_mates_the_one_nearer_the_goal_rates_higher() {
        let goal = [30.0, 0.0];
        let opponents = [[0.0, 20.0]];
        let ahead = pass_rating([0.0, 0.0], [10.0, 0.0], goal, &opponents, 0.5);
        let behind = pass_rating([0.0, 0.0], [-10.0, 0.0], goal, &opponents, 0.5);
        assert!(ahead > behind, "{ahead} {behind}");
        // Safety counts too: the same pass with a defender on the receiver rates lower.
        let marked = pass_rating([0.0, 0.0], [10.0, 0.0], goal, &[[10.5, 0.0]], 0.5);
        assert!(marked < ahead, "{marked} {ahead}");
    }
}
