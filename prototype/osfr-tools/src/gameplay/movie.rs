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
