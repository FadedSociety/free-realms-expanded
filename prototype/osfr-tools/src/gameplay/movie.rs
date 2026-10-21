//! The five crafting and gathering movies: gate order, busy row and fixed
//! refusals. One type serves the gate and the tick so their order and strings
//! cannot diverge.
use super::occupation::Occupation;

/// (R) The server step for the five movie clocks; reconstructed server policy,
/// not a recovered client constant.
const MOVIE_TICK_MS: u64 = 250;

/// The five crafting and gathering movies. One gate and one tick path serve
/// all of them, so a change to the leave, refusal or clock order cannot
/// diverge between engines.
#[derive(Clone, Copy)]
pub(super) enum Movie {
    Forging,
    Cooking,
    Smelting,
    Mining,
    Harvesting,
}

impl Movie {
    /// The order the five per-engine blocks were checked in. Only one movie
    /// runs at a time; a hand-made state with two keeps the earlier answer.
    pub(super) const ALL: [Movie; 5] = [
        Movie::Forging,
        Movie::Cooking,
        Movie::Smelting,
        Movie::Mining,
        Movie::Harvesting,
    ];

    /// The malformed 26/42 refusal, byte-identical to the five old blocks. Static
    /// so a refused garbage request allocates no intermediate `String`.
    pub(super) fn leave_error(self) -> &'static str {
        match self {
            Movie::Forging => "invalid forging leave request",
            Movie::Cooking => "invalid cooking leave request",
            Movie::Smelting => "invalid smelting leave request",
            Movie::Mining => "invalid mining leave request",
            Movie::Harvesting => "invalid harvesting leave request",
        }
    }

    /// The blocked-job/equipment/interaction reply, byte-identical to the five
    /// old blocks.
    pub(super) fn blocked_message(self) -> &'static str {
        match self {
            Movie::Forging => {
                "Close the forging game before changing jobs, equipment or interacting."
            }
            Movie::Cooking => {
                "Close the cooking game before changing jobs, equipment or interacting."
            }
            Movie::Smelting => {
                "Close the smelting game before changing jobs, equipment or interacting."
            }
            Movie::Mining => {
                "Close the mining game before changing jobs, equipment or interacting."
            }
            Movie::Harvesting => {
                "Close the harvesting game before changing jobs, equipment or interacting."
            }
        }
    }

    /// The other-`realm` reply, byte-identical to the five old blocks.
    pub(super) fn realm_message(self) -> &'static str {
        match self {
            Movie::Forging => "Close the forging game before other opening actions.",
            Movie::Cooking => "Close the cooking game before other opening actions.",
            Movie::Smelting => "Close the smelting game before other opening actions.",
            Movie::Mining => "Close the mining game before other opening actions.",
            Movie::Harvesting => "Close the harvesting game before other opening actions.",
        }
    }

    /// The busy-table row that detects this movie (see `Gameplay::is_occupied`).
    pub(super) fn occupation(self) -> Occupation {
        match self {
            Movie::Forging => Occupation::Forging,
            Movie::Cooking => Occupation::Cooking,
            Movie::Smelting => Occupation::Smelting,
            Movie::Mining => Occupation::Mining,
            Movie::Harvesting => Occupation::Harvesting,
        }
    }
}

/// The elapsed milliseconds a movie clock is due for, in `MOVIE_TICK_MS`
/// steps. A missing clock means the movie does not tick.
pub(super) fn elapsed_if_due(clock: Option<u64>, now_ms: u64) -> Option<u32> {
    let elapsed = now_ms.saturating_sub(clock?);
    (elapsed >= MOVIE_TICK_MS).then(|| elapsed.min(u32::MAX as u64) as u32)
}

/// Test-only start of the Forging movie as the production start path leaves it:
/// the engine run is live and the movie's own tick clock is stamped. Shared by
/// the default tick test and the private movie matrix so the two cannot drift.
#[cfg(test)]
pub(super) fn start_forging(game: &mut super::Gameplay, now_ms: u64) {
    let forging = game.forging.as_mut().unwrap();
    let key = forging.recipe_key().to_owned();
    let requirements = forging.requirements().to_vec();
    forging
        .begin_recipe(
            crate::forging::profile_for_test(),
            &key,
            u8::MAX,
            true,
            &requirements,
        )
        .unwrap();
    game.forging_tick_ms = Some(now_ms);
}

#[cfg(test)]
mod tests {
    use super::super::t013_objective_events::game;
    use super::super::*;
    use super::start_forging;

    /// The consolidated Forging tick through the production `Gameplay::tick`
    /// path. The private W104 matrix is ignored; this fixture is the shared
    /// opening reference the other default tests already use, so the Forging
    /// arm stays covered by the default suite.
    #[test]
    fn forging_tick_advances_once_per_250ms_step() {
        let mut game = game();
        start_forging(&mut game, 1_000);
        let character = CharacterState {
            guid: 1,
            zone_id: 0,
            position: [0.0; 3],
            rotation: [0.0, 0.0, 1.0],
        };
        let wall = |game: &Gameplay| {
            game.forging
                .as_ref()
                .unwrap()
                .state()
                .active
                .as_ref()
                .unwrap()
                .wall_ms
        };
        let before = wall(&game);
        game.tick(&character, 1_100).unwrap();
        assert_eq!(game.forging_tick_ms, Some(1_000), "an early tick restamped");
        assert_eq!(wall(&game), before, "an early tick advanced the run");

        game.tick(&character, 1_250).unwrap();
        assert_eq!(
            game.forging_tick_ms,
            Some(1_250),
            "the due tick did not stamp"
        );
        assert_eq!(wall(&game), before + 250, "the due tick did not advance");
    }

    /// Pins all five movies' three fixed refusals. The default gateway tests
    /// start Harvesting only, so without this a typo in another movie's text
    /// would leave the suite green. The exhaustive match also forces a new
    /// movie variant to add its strings here.
    #[test]
    fn every_movie_owns_its_exact_static_refusals() {
        for movie in Movie::ALL {
            let (blocked, realm, leave) = match movie {
                Movie::Forging => (
                    "Close the forging game before changing jobs, equipment or interacting.",
                    "Close the forging game before other opening actions.",
                    "invalid forging leave request",
                ),
                Movie::Cooking => (
                    "Close the cooking game before changing jobs, equipment or interacting.",
                    "Close the cooking game before other opening actions.",
                    "invalid cooking leave request",
                ),
                Movie::Smelting => (
                    "Close the smelting game before changing jobs, equipment or interacting.",
                    "Close the smelting game before other opening actions.",
                    "invalid smelting leave request",
                ),
                Movie::Mining => (
                    "Close the mining game before changing jobs, equipment or interacting.",
                    "Close the mining game before other opening actions.",
                    "invalid mining leave request",
                ),
                Movie::Harvesting => (
                    "Close the harvesting game before changing jobs, equipment or interacting.",
                    "Close the harvesting game before other opening actions.",
                    "invalid harvesting leave request",
                ),
            };
            assert_eq!(movie.blocked_message(), blocked);
            assert_eq!(movie.realm_message(), realm);
            assert_eq!(movie.leave_error(), leave);
        }
    }
}
