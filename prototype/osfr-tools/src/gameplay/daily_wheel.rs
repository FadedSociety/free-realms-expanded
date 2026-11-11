//! Daily wheel glue for `Gameplay`; the rules, the 143 spin count, the movie's messages and the
//! run live in `crate::daily_wheel`. 26/11 (the dock and calendar button) or the Games window's
//! Play for the wheel row opens the run; the client then starts it (39/5, answered with the Flash
//! launch), the movie connects and spins, and its close (39/6 or `FRServer_GameClose`) ends it.
//! A spin is drawn, paid and saved in one outcome here; nothing the client sends picks a prize.
use super::*;
use crate::{
    daily_wheel::Say,
    rewards::{GrantFailure, apply_grant},
};

/// (R) Shown when the bag lacks room for the wheel's largest prize; the spin is kept.
const BAG_FULL: &str = "Your bag is too full for the wheel's prizes. Make room and spin again.";
const CANNOT: &str = "The wheel could not pay its prize. Your spin has been kept.";

impl Gameplay {
    /// World entry, the opening campaign's included: the Games window lists (167/1) and the
    /// wheel's state. F3 (rc-0136 m-wheel): the opening's world entry sent the lists, whose wheel
    /// row is live, but never the 143/1, so the client read no spin and said "out of spins".
    pub(super) fn append_games_lists(&mut self, out: &mut Outcome) {
        self.append_activity_list(out);
        self.append_daily_wheel(out);
    }

    /// World entry: end a stale run and publish today's spin count (143/1).
    pub(super) fn append_daily_wheel(&mut self, out: &mut Outcome) {
        out.replies.extend(self.daily_wheel.world_entry());
    }

    /// The wheel's start requests and, while its run is live or just ended, its movie messages
    /// and the run's lifecycle packets. `None` leaves the packet to later owners (also whenever
    /// the wheel is off).
    pub(super) fn handle_daily_wheel(
        &mut self,
        inner: &[u8],
        player: u64,
    ) -> io::Result<Option<Outcome>> {
        if self.daily_wheel.is_start_request(inner) {
            return self.start_daily_wheel().map(Some);
        }
        if let Some(say) = self.daily_wheel.say(inner) {
            let mut out = Outcome::default();
            match say {
                Say::Connect => out.replies = self.daily_wheel.connect()?,
                Say::Change(wheel) => out.replies.extend(self.daily_wheel.change(wheel)?),
                Say::Stop(wheel) => out.replies = self.daily_wheel.stop(wheel)?,
                Say::Close => out.replies = self.daily_wheel.close()?,
                Say::Spin(wheel) => self.spin_daily_wheel(wheel, player, &mut out)?,
            }
            return Ok(Some(out));
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
        // F3: the opening campaign offers the wheel too (`append_games_lists`).
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

    /// The spin button. A request that is not allowed (no spin, unknown wheel, a turn in
    /// flight) is ignored before anything is copied. Then, before the draw, every prize the
    /// wheel could pay must fit the bag: if one does not, one chat line, the wheel closes and the
    /// spin is kept, so a full bag can never be used to refuse only the prizes it dislikes.
    /// Only then: draw the wedge and prize, pay through the one reward path, use up the spin
    /// and credit the minigame, on a staged copy so a failure changes nothing.
    fn spin_daily_wheel(&mut self, wheel: i32, player: u64, out: &mut Outcome) -> io::Result<()> {
        let Some(possible) = self.daily_wheel.possible_grants(wheel, player) else {
            return Ok(());
        };
        // ponytail: one staged apply per prize (at most a few dozen, once a spin); count free
        // bag rows instead if this ever shows in a profile.
        let snapshot = self.reward_snapshot();
        let unpayable = (possible.iter())
            .find_map(|grant| apply_grant(&snapshot, grant, "daily-wheel").err());
        if let Some(failure) = unpayable {
            let text = if failure == GrantFailure::BagFull {
                BAG_FULL
            } else {
                eprintln!("daily wheel payout refused: {failure}");
                CANNOT
            };
            out.replies.push(chat::system(text));
            out.replies.extend(self.daily_wheel.close()?);
            return Ok(());
        }
        let mut staged = self.clone();
        let Some(draw) = staged.daily_wheel.draw(wheel) else {
            return Ok(());
        };
        let mut packets = Vec::new();
        let mut coins = 0;
        if let Some(grant) = staged.daily_wheel.grant(&draw, player) {
            // The spin count is the once-only guard; the key only satisfies the applier (the
            // grant is staged fresh here, as `pay_earned_station_cash` does).
            // Every possible prize was just applied to this same snapshot, so this cannot fail.
            let (next, paid) = apply_grant(&snapshot, &grant, "daily-wheel")?;
            // What the coins really paid (a coin booster adds to it).
            coins = (next.inventory.state().coins).saturating_sub(self.inventory.state().coins);
            packets = paid;
            staged.adopt_reward_state(next, out)?;
        }
        let item = (draw.prize.items.first())
            .and_then(|item| self.inventory.presentation(item.definition_id).ok())
            .map(|look| (look.icon_id, look.name_id));
        // A paid spin is a finished game: the server measured no score, time or placement.
        if let Some(minigame_id) = staged.daily_wheel.minigame_id() {
            let result = crate::quest_engine::MinigameResult {
                won: true,
                elapsed_seconds: None,
                score: None,
                placement: None,
                knocked_out: false,
            };
            staged.credit_minigame(minigame_id, result, out)?;
        }
        out.replies
            .push(staged.daily_wheel.commit(&draw, coins, item, packets)?);
        out.changed = true;
        *self = staged;
        Ok(())
    }
}
