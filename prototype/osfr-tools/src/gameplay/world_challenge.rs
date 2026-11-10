//! L937: in-world timed challenges (client minigame TYPE 1) in `Gameplay`.
//!
//! The giver's menu offers the challenge (39/16); the client's Go (39/5) lays the run out around
//! the giver; the host tick and the player's clicks drive it (`crate::world_challenge::Session`);
//! the end settles once through `MinigameRuns::finish`: a win pays the row's coins and the job's
//! XP through the one reward applier and credits `FinishMinigame` and `MinigameResult` to the
//! quests; a loss pays nothing. The run is session-only, like the other in-world games: a
//! reconnect drops it unpaid, and nothing is paid before the end.
//!
//! Adapted from the Sanctuary community server fork (BubbaJoeX), `Sanctuary.Game/Challenges/
//! ChallengeManager.cs` (AGPL-3.0, credited in NOTICE): its offer, start, abandon and end flow,
//! rebuilt on this crate's run lifecycle, reward applier and quest credit.
use super::*;
use crate::{
    minigame::{self, Activity, ActivityFinish, MinigameRuns, RunState},
    rewards::{RewardDisplay, RewardGrant},
    world_challenge::{self, Challenge, Challenges, End, Session, Setup, Step},
};

const UNAVAILABLE: &str = "That challenge is not available right now.";
const BUSY: &str = "Finish what you are doing before you start a challenge.";
const JOB: &str = "You can't play this minigame because you don't have the required job.";
const FAR: &str = "Go back to whoever offered you the challenge to start it.";
const NO_ROOM: &str = "There's no room for that challenge here right now.";
const UNPAID: &str = "Your reward could not be paid.";

/// The offer on the start screen, or the run after it.
#[derive(Clone, Copy)]
struct Offered {
    id: u32,
    instance: i32,
    /// Where the start screen went up: zoning away from it drops the offer.
    zone: u32,
}

#[derive(Clone)]
pub(super) struct State {
    content: Option<Arc<Challenges>>,
    runs: MinigameRuns,
    offered: Option<Offered>,
    session: Option<Session>,
}

impl State {
    /// Load `world_challenges.json` if the content set has one and list each challenge in its
    /// giver's menu. A giver that is not NPC content is logged and left out.
    pub(super) fn load(
        directory: &Path,
        interaction: Option<&mut npc_interaction::NpcInteraction>,
    ) -> io::Result<Box<Self>> {
        let content = Challenges::load(directory)?.map(Arc::new);
        if let (Some(content), Some(interaction)) = (&content, interaction) {
            let offers: Vec<(u64, u32, u32)> = content
                .rows()
                .map(|row| (row.giver, row.id, row.name_id))
                .collect();
            for npc in interaction.set_challenge_gates(&offers) {
                npc_event(serde_json::json!({
                    "event": "challenge_giver_unknown_npc", "content_key": npc
                }));
            }
        }
        // Boxed: `Gameplay` is moved by value, and the debug gateway's test threads are near
        // their stack limit already.
        Ok(Box::new(Self {
            content,
            runs: MinigameRuns::new(world_challenge::FIRST_INSTANCE_ID)?,
            offered: None,
            session: None,
        }))
    }

    /// The start screen or the HUD is up: the player is busy.
    pub(super) fn is_active(&self) -> bool {
        self.runs.current_instance_id().is_some_and(|id| {
            matches!(
                self.runs.state(id),
                Some(RunState::Offered | RunState::Starting | RunState::Playing)
            )
        })
    }

    /// A reconnect drops whatever was up, unpaid; the new client holds no state of it.
    pub(super) fn reset(&mut self) {
        self.session = None;
        self.offered = None;
        let _ = self.runs.disconnect();
    }

    /// The end screen of a finished run is closed for good, so the next offer is not blocked.
    fn close_finished(&mut self) {
        let finished = self
            .runs
            .current_instance_id()
            .is_some_and(|id| matches!(self.runs.state(id), Some(RunState::Won | RunState::Lost)));
        if finished {
            let _ = self.runs.handle_client(&[26, 0, 42, 0]);
        }
    }

    /// Drop an unstarted or just-refused run, telling the client to close its start screen.
    fn cancel(&mut self, out: &mut Outcome, text: &str) {
        if let Some(offered) = self.offered.take() {
            let _ = self.runs.disconnect();
            out.replies.push(chat::system(text));
            out.replies.push(minigame::state_remove_packet(
                offered.instance,
                offered.id as i32,
            ));
        }
    }
}

/// The settled facts the shared lifecycle reports on the 39/18.
struct Verdict {
    won: bool,
    stars: u32,
    elapsed: u32,
    score: u32,
}

impl Activity for Verdict {
    fn score(&self) -> u32 {
        self.score
    }

    fn finish(&self) -> Option<ActivityFinish> {
        Some(ActivityFinish {
            won: self.won,
            stars: self.stars,
            elapsed_seconds: self.elapsed,
        })
    }
}

impl Gameplay {
    /// The job a challenge belongs to; 0 pays whichever is active.
    fn challenge_profile(&self, row: &Challenge) -> i32 {
        if row.profile != 0 {
            return row.profile;
        }
        self.profiles.as_ref().map_or(0, Profiles::active)
    }

    /// L927: a challenge pays the job's `activity_xp` where one challenge is one unit of its
    /// activity (a mail round, a race). A combat job's rate is per kill, so its skill tests keep
    /// the row's own constant (R).
    fn challenge_xp(&self, row: &Challenge) -> u32 {
        let profile = self.challenge_profile(row);
        match &self.profiles {
            Some(profiles) if profiles.kill_xp(profile).is_none() => {
                self.activity_xp(profile, row.xp)
            }
            _ => row.xp,
        }
    }

    /// The giver's menu choice: put the start screen (39/16) up. A refusal is one chat line.
    pub(super) fn begin_world_challenge(
        &mut self,
        id: u32,
        giver: u64,
        zone: u32,
    ) -> io::Result<Outcome> {
        let refuse = |text: &str| Outcome {
            replies: vec![chat::system(text)],
            ..Outcome::default()
        };
        let row = self
            .world_challenge
            .content
            .as_ref()
            .and_then(|content| content.get(id))
            .filter(|row| row.giver == giver)
            .cloned();
        let Some(row) = row else {
            return Ok(refuse(UNAVAILABLE));
        };
        if self.activity_busy() {
            return Ok(refuse(BUSY));
        }
        let locked = row.profile != 0
            && self
                .profiles
                .as_ref()
                .is_some_and(|profiles| !profiles.state().unlocked.contains(&row.profile));
        if locked {
            return Ok(refuse(JOB));
        }
        self.world_challenge.close_finished();
        let offer = self
            .world_challenge
            .runs
            .offer(row.run_spec(self.challenge_xp(&row)))?;
        let Some(instance) = self.world_challenge.runs.current_instance_id() else {
            return Ok(refuse(BUSY));
        };
        if !offer.accepted {
            return Ok(refuse(BUSY));
        }
        self.world_challenge.offered = Some(Offered { id, instance, zone });
        Ok(Outcome {
            replies: offer.replies,
            changed: true,
            ..Outcome::default()
        })
    }

    /// The daily wheel's and the in-world challenge's client packets.
    pub(super) fn handle_open_world_games(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        if let Some(out) = self.handle_daily_wheel(inner, character.guid)? {
            return Ok(Some(out));
        }
        self.handle_world_challenge(inner, character, now_ms)
    }

    /// The run's client packets: a click on one of its things, and the shared 39/x lifecycle
    /// (Go, leave, the automatic acknowledgement) and 26/42 close while it is on the screen.
    /// `None` leaves every other packet to later owners.
    pub(super) fn handle_world_challenge(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        let Some(offered) = self.world_challenge.offered else {
            return Ok(None);
        };
        if inner.starts_with(&[26, 0, 8, 0]) {
            let step = npc_interaction::decode_npc_interact_26_8(inner)
                .ok()
                .zip(self.world_challenge.session.as_mut())
                .and_then(|(request, session)| {
                    session.click(request.target, character.position, now_ms)
                });
            return match step {
                Some(step) => Ok(Some(self.world_challenge_step(step, character, now_ms))),
                None => Ok(None),
            };
        }
        let runs = &self.world_challenge.runs;
        // Closing the start screen sends 26/42 and then 39/6: only the 39/6 answers it.
        let close = inner == [26, 0, 42, 0]
            && !matches!(runs.state(offered.instance), None | Some(RunState::Offered));
        if !(runs.names_run(inner) || close) {
            return Ok(None);
        }
        // Quitting mid-game still needs the 39/18, or the client's HUD never comes back
        // (`009bf190`; the same trap as `minigame.rs`'s closing note).
        if inner.starts_with(&[39, 0, 6]) && self.world_challenge.session.is_some() {
            let mut out = Outcome::default();
            self.finish_world_challenge(End::Left, now_ms, character.guid, &mut out);
            return Ok(Some(out));
        }
        let run = self.world_challenge.runs.handle_client(inner)?;
        if !run.accepted {
            return Ok(None);
        }
        let mut out = Outcome {
            changed: run.changed,
            ..Outcome::default()
        };
        match run.state {
            Some(RunState::Starting) => {
                self.start_world_challenge(run.replies, character, now_ms, &mut out)?;
            }
            Some(RunState::Cancelled) => {
                // The player left the game: its things and HUD go, and no verdict is shown.
                if let Some(session) = self.world_challenge.session.take() {
                    out.replies.extend(session.cleanup_packets());
                }
                self.world_challenge.offered = None;
                out.replies.extend(run.replies);
            }
            Some(RunState::Closed) => self.world_challenge.offered = None,
            _ => out.replies.extend(run.replies),
        }
        Ok(Some(out))
    }

    /// Go was pressed: lay the run out around the giver, or refuse it where it cannot be.
    fn start_world_challenge(
        &mut self,
        ack: Vec<Vec<u8>>,
        character: &CharacterState,
        now_ms: u64,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let (Some(offered), Some(content)) = (
            self.world_challenge.offered,
            self.world_challenge.content.clone(),
        ) else {
            return Ok(());
        };
        let Some(row) = content.get(offered.id).cloned() else {
            self.world_challenge.cancel(out, UNAVAILABLE);
            return Ok(());
        };
        let tuning = *content.tuning();
        let interaction = self.npc_interaction.as_ref();
        let giver = interaction.and_then(|npcs| npcs.location(row.giver));
        let Some((_, giver)) = giver else {
            self.world_challenge.cancel(out, UNAVAILABLE);
            return Ok(());
        };
        if !Session::within_start_range(&tuning, giver, character.position) {
            self.world_challenge.cancel(out, FAR);
            return Ok(());
        }
        let recipients = (row.stages.iter())
            .flat_map(|stage| &stage.recipients)
            .filter_map(|key| Some((*key, interaction?.location(*key)?.1)))
            .collect();
        let zone = character.zone_id;
        let terrain = self.terrain.clone();
        let flat = !terrain
            .as_ref()
            .is_some_and(|terrain| terrain.grid(zone).is_some());
        let ground = |x: f32, z: f32| terrain.as_ref()?.height_at(zone, x, z);
        let (clearance, body, step) = (tuning.prop_clearance, tuning.body_height, tuning.walk_step);
        let prop_floor = |x: f32, z: f32, from: f32| {
            (terrain.as_ref())?.obstacles().surface_below(zone, x, z, from + step, 2.0 * step)
        };
        let blocked = |a: [f32; 3], b: [f32; 3]| {
            (terrain.as_ref()).is_some_and(|t| t.obstacles().segment_blocked(zone, a, b, clearance, body))
        };
        let setup = Setup {
            instance: offered.instance,
            zone_id: zone,
            giver,
            recipients,
            xp: self.challenge_xp(&row),
            now_ms,
            seed: now_ms
                ^ character.guid.rotate_left(17)
                ^ (u64::from(offered.instance as u32) << 32),
            ground: &ground,
            blocked: &blocked,
            prop_floor: &prop_floor,
            flat,
        };
        let Some(session) = Session::plan(&row, &tuning, setup) else {
            self.world_challenge.cancel(out, NO_ROOM);
            return Ok(());
        };
        npc_event(serde_json::json!({
            "event": "world_challenge_laid", "id": row.id, "stages": session.laid_positions()
        }));
        let (before, after) = session.start_packets();
        out.replies.extend(before);
        out.replies.extend(ack);
        out.replies.extend(after);
        self.world_challenge.runs.started(offered.instance);
        self.world_challenge.session = Some(session);
        Ok(())
    }

    /// The host tick (the common world path runs it from `pet_tick`): the deadline, the area, and
    /// walking over things.
    pub(super) fn tick_world_challenge(
        &mut self,
        character: &CharacterState,
        now_ms: u64,
        out: &mut Outcome,
    ) {
        out.replies.extend(self.daily_wheel.day_rollover()); // the wheel's count at UTC midnight
        let Some(session) = self.world_challenge.session.as_mut() else {
            let state = &mut self.world_challenge;
            if state.offered.is_some_and(|o| {
                o.zone != character.zone_id && state.runs.state(o.instance) == Some(RunState::Offered)
            }) {
                state.cancel(out, UNAVAILABLE);
            }
            return;
        };
        let step = session.tick(character.zone_id, character.position, now_ms);
        out.replies.extend(step.packets);
        if let Some(end) = step.end {
            self.finish_world_challenge(end, now_ms, character.guid, out);
        }
    }

    fn world_challenge_step(
        &mut self,
        step: Step,
        character: &CharacterState,
        now_ms: u64,
    ) -> Outcome {
        let mut out = Outcome {
            replies: step.packets,
            ..Outcome::default()
        };
        if let Some(end) = step.end {
            self.finish_world_challenge(end, now_ms, character.guid, &mut out);
        }
        out
    }

    /// Settle on a staged copy, so a refusal part-way changes nothing. A run that cannot settle
    /// is dropped with its course taken away, not retried every tick.
    fn finish_world_challenge(&mut self, end: End, now_ms: u64, player: u64, out: &mut Outcome) {
        let mut staged = self.clone();
        let mut settled = Outcome::default();
        match staged.settle_world_challenge(end, now_ms, player, &mut settled) {
            Ok(()) => {
                *self = staged;
                out.absorb(settled);
            }
            Err(error) => {
                npc_event(serde_json::json!({
                    "event": "world_challenge_unsettled", "error": error.to_string()
                }));
                if let Some(session) = self.world_challenge.session.take() {
                    out.replies.extend(session.cleanup_packets());
                }
                self.world_challenge.cancel(out, UNPAID);
            }
        }
    }

    fn settle_world_challenge(
        &mut self,
        end: End,
        now_ms: u64,
        player: u64,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let (Some(session), Some(offered)) = (
            self.world_challenge.session.take(),
            self.world_challenge.offered,
        ) else {
            return Ok(());
        };
        let row = Arc::clone(session.challenge());
        let won = end == End::Won;
        let verdict = Verdict {
            won,
            stars: if won { session.xp() } else { 0 },
            elapsed: session.elapsed_seconds(now_ms),
            score: session.done(),
        };
        let grant = self.challenge_grant(&row, &session, player);
        let snapshot = self.reward_snapshot();
        let runs = &mut self.world_challenge.runs;
        let paid = if won {
            runs.finish(offered.instance, &verdict, Some((&snapshot, &grant)))
                .ok()
        } else {
            None
        };
        let run = match paid {
            Some(run) => run,
            None if won => {
                out.replies.push(chat::system(UNPAID));
                runs.finish_win_without_reward(offered.instance, &verdict)?
            }
            None => runs.finish(offered.instance, &verdict, None)?,
        };
        out.replies.extend(session.ending_packets(won, now_ms));
        if run.reward_state.is_some() {
            // The score screen's rewards tab reads the bundle this replaces.
            let mut spec = row.run_spec(session.xp());
            spec.reward_bundles = row.bundles(Some((row.coins, session.xp())), session.xp());
            out.replies
                .push(minigame::reward_update_packet(offered.instance, &spec)?);
        }
        out.replies.extend(run.replies.iter().cloned());
        out.replies.extend(run.reward_packets);
        if let Some(next) = run.reward_state {
            self.adopt_reward_state(next, out)?;
        }
        if won {
            self.npc_minigame_win(&run.replies, out)?;
        }
        Ok(())
    }

    /// A win's pay: the row's coins and the job's XP (nothing for a job the content lacks).
    fn challenge_grant(&self, row: &Challenge, session: &Session, player: u64) -> RewardGrant {
        let profile = self.challenge_profile(row);
        let job_xp = if self.profiles.is_some() && profile != 0 && session.xp() > 0 {
            BTreeMap::from([(profile, session.xp())])
        } else {
            BTreeMap::new()
        };
        RewardGrant {
            coins: row.coins,
            items: Vec::new(),
            job_xp,
            sc: 0,
            display: RewardDisplay {
                source_guid: player,
                player_guid: player,
                reward_kind: 0,
                icon_id: row.icon_id as i32,
                name_id: row.name_id as i32,
                rows: Vec::new(),
            },
        }
    }
}

#[cfg(test)]
mod tests;
