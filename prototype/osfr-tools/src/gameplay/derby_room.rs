//! A lobby room's derby, as one driver's `Gameplay` sees it: the room's limits, the seat, and the
//! first tick after the countdown, when this driver switches job and zones into the arena. The
//! match is `crate::derby_gateway::HostedDerby`; `bin/osfr_gateway/room_routing.rs` owns it.
use super::*;
use crate::derby_gateway::{DerbySeat, HostedDerby};
use crate::game_room::Limits;

/// (R) The line a driver reads when their seat went on without them.
pub const DERBY_SEAT_LOST: &str = "The derby started without you.";

impl Gameplay {
    /// A new room's limits from its host's options. `Ok(None)`: a two-seat board game. `Err`: the
    /// lobby is the derby's and the options do not make a match.
    pub(crate) fn lobby_limits(
        &self,
        lobby_type: u32,
        options: &[(u32, u32)],
    ) -> Result<Option<Limits>, ()> {
        match &self.derby {
            Some(derby) => derby.lobby_limits(lobby_type, options),
            None => Ok(None),
        }
    }

    /// Whether `lobby_type` is the derby's lobby here.
    pub(crate) fn is_derby_lobby(&self, lobby_type: u32) -> bool {
        let lobby = self.derby.as_ref().and_then(|derby| derby.lobby());
        lobby.is_some_and(|lobby| lobby.lobby_type == lobby_type)
    }

    /// This player as a driver of a room's match: their garage car and their ledger's next id.
    pub(crate) fn derby_seat(&self, guid: u64, name: String) -> Option<DerbySeat> {
        let derby = self.derby.as_ref()?;
        Some(DerbySeat {
            guid,
            name,
            look: self.kart_entry.as_ref().and_then(|kart| kart.chosen_look(2)),
            next_instance: derby.next_instance(),
        })
    }

    pub(crate) fn host_derby(
        &self,
        room: u32,
        options: &[(u32, u32)],
        seats: &[DerbySeat],
    ) -> io::Result<HostedDerby> {
        let derby = self.derby.as_ref().ok_or_else(|| invalid("the derby is not loaded"))?;
        derby.host(room, options, seats)
    }

    pub(crate) fn sit_derby(&mut self, table: &HostedDerby, guid: u64) {
        if let Some(derby) = self.derby.as_mut() {
            derby.sit(table, guid);
        }
    }

    /// One tick of a room's driver, before the match's own: the start records on the first tick
    /// after the countdown, or the way out when the seat is gone. `None`: nothing of the room's.
    pub(super) fn hosted_derby_step(
        &mut self,
        character: &CharacterState,
    ) -> io::Result<Option<Outcome>> {
        let Some(derby) = self.derby.as_mut() else {
            return Ok(None);
        };
        if derby.hosted_lost() {
            return self.leave_lost_derby(character, DERBY_SEAT_LOST).map(Some);
        }
        let Some(start) = derby.take_hosted_begin() else {
            return Ok(None);
        };
        let close = derby.lobby_close();
        // The job switch is all or nothing: a driver it refuses gives the seat up.
        let mut tried = self.clone();
        let mut out = match tried.derby_job(character) {
            Ok(out) => out,
            Err(error) => {
                let line = error.to_string();
                return self.leave_lost_derby(character, &line).map(Some);
            }
        };
        *self = tried;
        out.replies.extend(close);
        let Some(derby) = self.derby.as_mut() else {
            return Ok(Some(out));
        };
        derby.mark_offered();
        // As a solo offer: an arena row zones in, and the records wait for its Loaded.
        match derby.scene().cloned() {
            Some(scene) => {
                derby.enter_arena(character.clone(), start);
                out.scene_zoning = Some(scene);
            }
            None => out.replies.extend(start),
        }
        out.changed = true;
        Ok(Some(out))
    }

    /// Out of a room's match with nothing to show for it: the client's game goes, the job bar and
    /// status come back, and an arena visit ends.
    fn leave_lost_derby(&mut self, character: &CharacterState, line: &str) -> io::Result<Outcome> {
        let mut out = Outcome { changed: true, ..Outcome::default() };
        let Some(derby) = self.derby.as_mut() else {
            return Ok(out);
        };
        // None for a driver who was never offered the match: nothing on their client to take away.
        out.replies.extend(derby.lost_packet());
        out.replies.push(chat::system(line));
        let previous_job = derby.exit();
        self.leave_activity_job(previous_job, character.guid, &mut out)?;
        out.replies.push(crate::kart_gateway::player_status(character.guid, 1));
        self.leave_derby_arena(&mut out)?;
        self.refresh_npc_quest_eligibility(current_unix_seconds());
        Ok(out)
    }
}
