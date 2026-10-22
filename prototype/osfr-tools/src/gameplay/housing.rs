//! Player housing in the world session (lead, 2026-10-20, slice 1): the client asks for its
//! houses (127/23 grant, 127/14 list), enters one (127/19) and leaves (127/10). The lot logic is
//! `crate::housing_service`; this file only binds it to the session: the saved lots, the world
//! pose to come back to, and the house packets that must wait until the lot scene has loaded.
//!
//! Dormant by default: without `houses.json` in the content directory there is no session and
//! family 127 falls through exactly as before.
use super::*;
use crate::housing_service::{
    FixtureConfig, HousingContent, HousingService, HousingServiceState, HousingZoneChange,
    ServiceOutcome, decode_request,
};

/// Base opcode of the housing family, as the first two bytes of every inner packet.
const FAMILY: [u8; 2] = [127, 0];

#[derive(Clone, Debug)]
pub(super) struct HousingSession {
    service: HousingService,
    /// The world pose captured on entry; `Some` exactly while the player is inside a house.
    return_to: Option<CharacterState>,
    /// House packets for the lot being entered, sent once the client reports the scene loaded.
    pending: Vec<Vec<u8>>,
    /// The lot scene sent on entry: the server's membership registry places the player in it.
    scene: Option<crate::zone_transition::ZoneTransition>,
}

impl HousingSession {
    /// Restore the saved lots against this character's inventory (the furniture conservation
    /// check lives in the service). `Ok(None)` when the content has no housing table.
    pub(super) fn load(
        directory: &Path,
        saved: Option<&HousingServiceState>,
        inventory: &crate::inventory::InventoryState,
    ) -> io::Result<Option<Self>> {
        let path = directory.join("houses.json");
        if !path.is_file() {
            return Ok(None);
        }
        let content = HousingContent::load(&path).map_err(|e| invalid(format!("{e:?}")))?;
        // ponytail: placement rows are the reconstructed defaults until decorating (slice 2)
        // gets a real fixture table.
        let config = FixtureConfig::capture_defaults();
        let service = match saved {
            Some(state) => {
                HousingService::from_state_with_content(config, content, state, inventory)
                    .map_err(|e| invalid(format!("saved housing: {e:?}")))?
            }
            None => HousingService::with_content(config, content)
                .map_err(|e| invalid(format!("{e:?}")))?,
        };
        Ok(Some(Self {
            service,
            return_to: None,
            pending: Vec::new(),
            scene: None,
        }))
    }

    pub(super) fn state(&self) -> Option<HousingServiceState> {
        Some(self.service.state()).filter(|state| !state.lots.is_empty())
    }

    pub(super) fn in_house(&self) -> bool {
        self.return_to.is_some()
    }
}

fn encode_replies(outcome: &ServiceOutcome) -> io::Result<Vec<Vec<u8>>> {
    outcome
        .replies
        .iter()
        .map(|reply| reply.encode_inner())
        .collect()
}

impl Gameplay {
    /// Family 127. Requests are staged on a copy of the session and the inventory, so a refusal
    /// (one chat line, plus the client's 127/53 for a decorate request) changes nothing. What the
    /// request did to the backpack reaches the client as ordinary inventory updates.
    pub(super) fn handle_housing(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
    ) -> io::Result<Option<Outcome>> {
        if !inner.starts_with(&FAMILY) {
            return Ok(None);
        }
        let Some(session) = self.housing.as_ref() else {
            return Ok(None);
        };
        let mut session = session.clone();
        let mut inventory = self.inventory.clone();
        let result = session
            .service
            .handle(character.guid, inner, &mut inventory);
        let outcome = match result {
            Ok(outcome) => outcome,
            Err(error) => {
                let mut replies = vec![chat::system(&error.denial_message())];
                // Client reader 00ab3030: one length-prefixed reason string.
                if decode_request(inner).is_ok_and(|request| request.answers_with_denial()) {
                    replies.push(error.denial_reply().encode_inner()?);
                }
                return Ok(Some(Outcome {
                    replies,
                    ..Outcome::default()
                }));
            }
        };
        let mut out = Outcome {
            changed: outcome.changed,
            ..Outcome::default()
        };
        match &outcome.zone_change {
            Some(HousingZoneChange::EnterHouse {
                scene,
                spawn_position,
                spawn_rotation,
                ..
            }) => {
                // Entering from inside a house keeps the original world pose.
                session.return_to.get_or_insert_with(|| character.clone());
                session.pending = encode_replies(&outcome)?;
                let lot_scene = crate::zone_transition::ZoneTransition {
                    envelope_flag: 1,
                    scene: scene.name.clone(),
                    type_id: i32::from(scene.zone_type),
                    position: *spawn_position,
                    rotation: *spawn_rotation,
                    sky: scene.sky.clone(),
                    tutorial: scene.tutorial,
                    unknown: u8::try_from(scene.unknown).unwrap_or(1),
                    id: i32::try_from(scene.zone_id).map_err(|_| invalid("house zone id"))?,
                    update_radius: scene.update_radius,
                    geometry_id: i32::try_from(scene.geometry_id)
                        .map_err(|_| invalid("house geometry id"))?,
                    override_update_radius: scene.override_update_radius,
                    wait_for_zone_ready: scene.wait_for_zone_ready,
                };
                session.scene = Some(lot_scene.clone());
                out.scene_zoning = Some(lot_scene);
            }
            Some(HousingZoneChange::ReturnToPreviousZone { .. }) => {
                let back = session
                    .return_to
                    .take()
                    .ok_or_else(|| invalid("left a house with no saved world pose"))?;
                session.pending.clear();
                session.scene = None;
                out.scene_zoning = Some(self.world_zoning(&back)?);
            }
            None => out.replies = encode_replies(&outcome)?,
        }
        // 38/4 for a placed item, 38/2 for a returned or granted one. `adopt_state` commits the
        // inventory only if it validates, before the session, so a failure here changes nothing.
        if inventory.state() != self.inventory.state() {
            out.replies
                .extend(self.inventory.adopt_state(inventory.state().clone())?);
        }
        self.housing = Some(session);
        Ok(Some(out))
    }

    /// After the lot scene loads: zone data, the house record, house info, the item list and the
    /// saved fixtures, in the order the client requires (127/28 before any 127/4x).
    pub(super) fn housing_loaded(&mut self) -> Option<Vec<Vec<u8>>> {
        let session = self.housing.as_mut().filter(|s| s.in_house())?;
        Some(std::mem::take(&mut session.pending))
    }

    /// A save taken inside a house stores the world pose, never house coordinates.
    pub fn house_return(&self) -> Option<CharacterState> {
        self.housing.as_ref()?.return_to.clone()
    }

    /// A reconnect leaves the house and resumes at the world pose; the service forgets the visit,
    /// so no edit mode or selection outlives it.
    pub fn abandon_house_for_reconnect(&mut self) -> Option<CharacterState> {
        let session = self.housing.as_mut()?;
        session.pending.clear();
        session.scene = None;
        let back = session.return_to.take()?;
        session.service.end_visit(back.guid);
        Some(back)
    }

    /// While inside, the lot is the player's scene (like a soccer arena), so the server's
    /// membership registry registers it and gives the visit its own instance.
    pub(super) fn house_zone(&self) -> Option<crate::encounter::ZoneTransition> {
        let scene = self
            .housing
            .as_ref()
            .filter(|s| s.in_house())?
            .scene
            .as_ref()?;
        Some(crate::encounter::ZoneTransition {
            zone_id: u32::try_from(scene.id).ok()?,
            name: scene.scene.clone(),
            sky: scene.sky.clone(),
            geometry_id: scene.geometry_id,
            position: scene.position[..3].try_into().ok()?,
            rotation: scene.rotation[..3].try_into().ok()?,
            tutorial: scene.tutorial,
        })
    }

    fn world_zoning(
        &self,
        back: &CharacterState,
    ) -> io::Result<crate::zone_transition::ZoneTransition> {
        let world = self
            .encounter
            .as_ref()
            .ok_or_else(|| invalid("house return requires world content"))?
            .world();
        Ok(crate::zone_transition::ZoneTransition {
            envelope_flag: 1,
            scene: world.name.clone(),
            type_id: 2,
            position: [back.position[0], back.position[1], back.position[2], 1.0],
            rotation: [back.rotation[0], back.rotation[1], back.rotation[2], 1.0],
            sky: world.sky.clone(),
            id: i32::try_from(back.zone_id).map_err(|_| invalid("world zone id"))?,
            geometry_id: world.geometry_id,
            override_update_radius: true,
            unknown: 1,
            ..Default::default()
        })
    }
}
