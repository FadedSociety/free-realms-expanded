//! The login/ready startup stream: the typed snapshot's 46 messages with their live substitutions.

use super::{Gateway, Kind, wrapped};
use crate::character_store::CharacterState;
use crate::gameplay::Gameplay;
use std::io;

impl Gateway {
    /// One phase of the startup stream. Index 41 is the welcome splash (opcode 93): the client
    /// copies its show flag (`00ad0790`) and calls OnShowWelcomeScreen on every one (`00ad0000`),
    /// so it goes out once per login and a reload (`welcomed`) skips it (T437).
    pub(super) fn startup_for(
        &self,
        kind: Kind,
        state: &CharacterState,
        gameplay: Option<&Gameplay>,
        welcomed: bool,
    ) -> io::Result<Vec<Vec<u8>>> {
        let snapshot = &self.startup_snapshot;
        let range = match kind {
            Kind::Login => 0..8,
            Kind::Ready => 8..46,
        };
        range
            .filter(|index| {
                !(gameplay.is_some_and(Gameplay::is_opening) && (10..=32).contains(index))
                    && !(welcomed && *index == 41)
            })
            .map(|index| {
                let message = match index {
                    3 => {
                        let zone = gameplay.and_then(Gameplay::current_zone);
                        snapshot.build_zone(state.zone_id, zone.as_ref())
                    }
                    5 => match gameplay.and_then(Gameplay::definitions_packet) {
                        Some(inner) => wrapped(&inner),
                        None => snapshot.build_packet(index, state.guid)?,
                    },
                    7 => self.build_self_message(state, gameplay)?,
                    // T437: 38/1 carries the live health once known, not the captured pair.
                    33 => match gameplay.and_then(Gameplay::startup_health) {
                        Some((current, max)) => {
                            crate::startup_messages::build_hit_points(current as i32, max as i32)
                        }
                        None => snapshot.build_packet(index, state.guid)?,
                    },
                    _ => snapshot.build_packet(index, state.guid)?,
                };
                Ok(message)
            })
            .collect()
    }
}
