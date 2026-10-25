//! The login/ready startup stream: the typed snapshot's 46 messages with their live substitutions.

use super::{Gateway, Kind, invalid, wrapped};
use crate::character_store::CharacterState;
use crate::gameplay::Gameplay;
use crate::profiles::ProfileState;
use crate::startup_selfdata::{self, SelfData};
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

    pub(super) fn build_self_message(
        &self,
        state: &CharacterState,
        gameplay: Option<&Gameplay>,
    ) -> io::Result<Vec<u8>> {
        let mut data = self.self_data_template.clone();
        data.identity.guid = state.guid;
        for index in 0..3 {
            data.transform.position[index] = state.position[index].to_bits();
            data.transform.rotation[index] = state.rotation[index].to_bits();
        }
        if let Some(look) = self.current_look(gameplay) {
            use crate::style_cards::{FACE_PAINT, HAIR, HEAD, MODEL_CUSTOMIZATION, SKIN_TONE};
            data.identity.model = look.model as u32;
            data.appearance.hair_color = look.hair_color as u32;
            data.appearance.eye_color = look.eye_color as u32;
            // The client's customization order: head, hair, skin tone, face paint, model customization.
            data.appearance.customization_ids =
                [HEAD, HAIR, SKIN_TONE, FACE_PAINT, MODEL_CUSTOMIZATION]
                    .map(|table| look.ids[table] as u32);
            (data.appearance.head, data.appearance.hair) = (look.head, look.hair);
            (data.appearance.skin_tone, data.appearance.face_paint) =
                (look.skin_tone, look.face_paint);
            data.appearance.model_customization = look.model_customization;
        }
        if let Some(identity) = &self.identity {
            data.name.first.clone_from(&identity.first_name);
            data.name.last.clone_from(&identity.last_name);
            data.gender = identity.gender as u32;
        }
        let mut collections = crate::progression_extras::CollectionLedger::default();
        if let Some(game) = gameplay {
            let current = game.state();
            data.inventory = current.inventory;
            collections = current.progression_extras.unwrap_or_default().collections;
            if let Some(profiles) = current.profiles {
                self.apply_profile_state(&mut data, &profiles)?;
            }
            data.jobs.profiles.extend(game.staff_job_rows());
        }
        crate::progression_extras::apply_collected_flags(&mut data.collections, &collections);
        Gameplay::write_self_quests(gameplay, &mut data)?;
        // (R) Action-bar assignments have no field in the current saved
        // GameplayState, so preserve the captured typed rows until that state exists.
        // The module-owned opaque post-effect tail remains untouched.
        Ok(startup_selfdata::build_self(&data))
    }

    fn apply_profile_state(&self, data: &mut SelfData, profiles: &ProfileState) -> io::Result<()> {
        data.jobs.active_profile_id = profiles.active as u32;
        for (id, rank_offset) in &self.profile_rank_offsets {
            let row = data
                .jobs
                .profiles
                .iter_mut()
                .find(|row| row.map_key == *id as u32)
                .ok_or_else(|| invalid("self-data is missing a configured job profile"))?;
            // The content offset includes the four-byte profile-map key; the
            // typed row stores its opaque prefix after that key.
            let offset = rank_offset
                .checked_sub(4)
                .ok_or_else(|| invalid("invalid job progression offset"))?;
            let end = offset
                .checked_add(12)
                .ok_or_else(|| invalid("job progression offset overflow"))?;
            let progression = row
                .opaque_before_equipment_without_key
                .get_mut(offset..end)
                .ok_or_else(|| invalid("self-data job progression field is truncated"))?;
            let xp = profiles.experience.get(id).copied().unwrap_or(0);
            let rank = (1 + xp / 100).min(20) as i32;
            let progress = if rank == 20 { 0 } else { (xp % 100) as i32 };
            // (R) The local rank curve and zero-star policy match Profiles.
            progression[..4].copy_from_slice(&rank.to_le_bytes());
            progression[4..8].copy_from_slice(&progress.to_le_bytes());
            progression[8..12].copy_from_slice(&0i32.to_le_bytes());
        }
        Ok(())
    }
}
