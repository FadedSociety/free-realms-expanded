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
            data.identity.model = look.model as u32;
            data.appearance.hair_color = look.hair_color as u32;
            data.appearance.eye_color = look.eye_color as u32;
            // Self-data carries all five ids; the order lives on `Look`.
            data.appearance.customization_ids = look.customization_ids().map(|id| id as u32);
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
            self.apply_profile_state(&mut data, current.profiles.as_ref(), game)?;
            data.jobs.profiles.extend(game.staff_job_rows());
        }
        crate::progression_extras::apply_collected_flags(&mut data.collections, &collections);
        Gameplay::write_self_quests(gameplay, &mut data)?;
        // (R) Action-bar assignments have no field in the current saved
        // GameplayState, so preserve the captured typed rows until that state exists.
        // The module-owned opaque post-effect tail remains untouched.
        // L124: a save the self data cannot carry is refused here instead of panicking at login.
        startup_selfdata::try_build_self(&data)
    }

    /// L107: every captured job record carries the captured player's rank, XP percent and stars;
    /// each is rewritten from this character's profiles, or as rank 1 with nothing earned when no
    /// profile state exists (the shipped content has no `profiles.json`, so its fresh characters
    /// used to show the capture's half-full Adventurer bar).
    fn apply_profile_state(
        &self,
        data: &mut SelfData,
        profiles: Option<&ProfileState>,
        game: &Gameplay,
    ) -> io::Result<()> {
        if let Some(profiles) = profiles {
            data.jobs.active_profile_id = profiles.active as u32;
        }
        for row in &mut data.jobs.profiles {
            let id =
                i32::try_from(row.map_key).map_err(|_| invalid("job profile id out of range"))?;
            let opaque = &mut row.opaque_before_equipment_without_key;
            let offset = progression_offset(opaque)
                .ok_or_else(|| invalid("self-data job record is truncated"))?;
            if let Some(configured) = self.profile_rank_offsets.get(&id)
                && configured.checked_sub(4) != Some(offset)
            {
                return Err(invalid(
                    "profiles.json rank offset disagrees with the job record",
                ));
            }
            let xp = profiles
                .and_then(|profiles| profiles.experience.get(&id).copied())
                .unwrap_or(0);
            opaque[offset..offset + 12]
                .copy_from_slice(&crate::profiles::progression_bytes(xp, &game.job_curve(id)));
        }
        Ok(())
    }
}

/// L107: where a job record's rank, XP percent and stars sit in its bytes after the map key: seven
/// header dwords, the members-only byte, an index dword, the item-class count and its 12-byte rows,
/// then five unnamed fields (14 bytes); layout as `profiles.rs` `StaffJob::prefix` writes it.
/// `None` when the record is too short to hold them.
fn progression_offset(opaque: &[u8]) -> Option<usize> {
    let classes = u32::from_le_bytes(opaque.get(33..37)?.try_into().ok()?);
    let offset = usize::try_from(classes)
        .ok()?
        .checked_mul(12)?
        .checked_add(51)?;
    (opaque.len() >= offset.checked_add(12)?).then_some(offset)
}
