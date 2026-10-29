//! T560: daily wheel glue for `Gameplay`; the rules, the 143/1 spin count and the run live in
//! `crate::daily_wheel`. 26/11 (the dock and calendar button) or the Games window's Play for the
//! wheel row opens the run; the client then starts it (39/5, answered with the Flash launch) and
//! its close (39/6) ends it. The run is session-only: nothing here marks a save, and no spin is
//! used up (part 2 pays the prize and stamps the day).
use super::*;

impl Gameplay {
    /// World entry: end a stale run and publish today's spin count (143/1). The opening campaign
    /// has no activity list either, so it never offers the wheel.
    pub(super) fn append_daily_wheel(&mut self, out: &mut Outcome) {
        out.replies.extend(self.daily_wheel.world_entry());
    }

    /// The wheel's start requests and, while its run is live or just ended, the run's lifecycle
    /// packets. `None` leaves the packet to later owners (also whenever the wheel is off).
    pub(super) fn handle_daily_wheel(&mut self, inner: &[u8]) -> io::Result<Option<Outcome>> {
        if self.daily_wheel.is_start_request(inner) {
            return self.start_daily_wheel().map(Some);
        }
        Ok(self.daily_wheel.route(inner)?.and_then(|run| {
            run.accepted.then(|| Outcome {
                replies: run.replies,
                ..Outcome::default()
            })
        }))
    }

    /// Every start path ends here: the wheel's own refusals, then the shared busy rules. A
    /// refusal answers with one chat line and changes nothing.
    fn start_daily_wheel(&mut self) -> io::Result<Outcome> {
        let refuse = |text: &str| Outcome {
            replies: vec![chat::system(text)],
            ..Outcome::default()
        };
        // A forged request can name the wheel in the opening campaign, which never offered it.
        if self.opening.is_some() {
            return Ok(refuse("The wheel is not available right now."));
        }
        if !self.daily_wheel.is_active() && (self.activity_busy() || self.warp_blocker().is_some())
        {
            return Ok(refuse(
                "Finish what you are doing before you spin the wheel.",
            ));
        }
        Ok(match self.daily_wheel.begin()? {
            Ok(replies) => Outcome {
                replies,
                ..Outcome::default()
            },
            Err(refusal) => refuse(refusal.message()),
        })
    }
}
