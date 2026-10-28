//! Player housing in the world session (lead, 2026-10-20, slice 1): the client asks for its
//! houses (127/23 grant, 127/14 list), enters one (127/19) and leaves (127/10). The lot logic is
//! `crate::housing_service`; this file only binds it to the session: the saved lots, the world
//! pose to come back to, and the house packets that must wait until the lot scene has loaded.
//!
//! Dormant by default: without `houses.json` in the content directory there is no session and
//! family 127 falls through exactly as before.
use super::*;
use crate::housing_service::{
    FixtureConfig, HousingContent, HousingReply, HousingService, HousingServiceState,
    HousingZoneChange, Request, ServiceOutcome, decode_request,
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
    /// T872: the lot entered, own or visited; it keys the shared house instance.
    house: Option<u64>,
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
        // The generated table (T518) when the content directory ships it; the historical captured
        // rows stay the fallback for content packages without one.
        let fixtures = directory.join("housing_fixtures.json");
        let config = if fixtures.is_file() {
            FixtureConfig::load(&fixtures).map_err(|e| invalid(format!("{e:?}")))?
        } else {
            FixtureConfig::capture_defaults()
        };
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
            house: None,
        }))
    }

    pub(super) fn state(&self) -> Option<HousingServiceState> {
        Some(self.service.state()).filter(|state| !state.lots.is_empty())
    }

    pub(super) fn in_house(&self) -> bool {
        self.return_to.is_some()
    }

    /// L046: the lot for a paid Marketplace deed. Staged on a copy and committed only once its
    /// replies encode and `commit` is set (a preview only asks), so `None` (no enabled lot for
    /// the deed, already owned) changes nothing.
    pub(super) fn grant_deed(
        &mut self,
        owner: u64,
        deed: i32,
        commit: bool,
    ) -> Option<Vec<Vec<u8>>> {
        let mut service = self.service.clone();
        let replies = encode_replies(&service.grant_deed(owner, deed).ok()?).ok()?;
        if commit {
            self.service = service;
        }
        Some(replies)
    }
}

/// A scene the player was zoned into (a house lot, a match, a race, a fishing hole) as
/// `current_zone` reports it to the startup zone record and the membership registry. `None` for
/// a negative id.
pub(super) fn scene_zone(
    scene: &crate::zone_transition::ZoneTransition,
) -> Option<crate::encounter::ZoneTransition> {
    let [x, y, z, _] = scene.position;
    let [rx, ry, rz, _] = scene.rotation;
    Some(crate::encounter::ZoneTransition {
        zone_id: u32::try_from(scene.id).ok()?,
        name: scene.scene.clone(),
        sky: scene.sky.clone(),
        geometry_id: scene.geometry_id,
        position: [x, y, z],
        rotation: [rx, ry, rz],
        tutorial: scene.tutorial,
    })
}

/// The housing records, then the fixtures' actors: the client draws a fixture only from its NPC
/// (35/2), which a retail server sends after the 127/40 and 127/44, and removes with a 35/3.
fn encode_replies(outcome: &ServiceOutcome) -> io::Result<Vec<Vec<u8>>> {
    let records = outcome.replies.iter().map(|reply| reply.encode_inner());
    let actors = outcome.actors.iter().map(|actor| actor.encode_inner());
    records.chain(actors).collect()
}

/// T872: what an owner's lot edit shows the lot's visitors: the fixture records (127/40, 127/41,
/// 127/51) and the fixtures' actors, with the new snapshot their copies are replaced with. The
/// owner-only replies (item list, house info, place answers, denials) stay with the owner.
fn house_broadcast(
    session: &HousingSession,
    outcome: &ServiceOutcome,
) -> io::Result<Option<crate::housing_service::HouseBroadcast>> {
    let Some(house_guid) = session.house.filter(|_| outcome.changed) else {
        return Ok(None);
    };
    let Ok(lot) = session.service.lot(house_guid) else {
        return Ok(None);
    };
    let records = (outcome.replies.iter())
        .filter(|reply| {
            matches!(
                reply,
                HousingReply::FixtureUpdate(_)
                    | HousingReply::FixtureRemove(_)
                    | HousingReply::PositionUpdate(_)
            )
        })
        .map(HousingReply::encode_inner);
    let actors = outcome.actors.iter().map(|actor| actor.encode_inner());
    let packets = records.chain(actors).collect::<io::Result<Vec<_>>>()?;
    Ok(
        (!packets.is_empty()).then_some(crate::housing_service::HouseBroadcast {
            house_guid,
            lot,
            packets,
        }),
    )
}

impl Gameplay {
    /// Family 127, plus the 174/16 rating directory probe. Requests are staged on a copy of the
    /// session and the inventory, so a refusal (one chat line, plus the client's 127/53 for a
    /// decorate request) changes nothing. What the request did to the backpack reaches the client
    /// as ordinary inventory updates.
    pub(super) fn handle_housing(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
    ) -> io::Result<Option<Outcome>> {
        // T517: on every house entry the client asks the rating directory for one candidate (127/39
        // list handler `0x00ac2a10`, 127/45 zone-data handler `0x00ac36c0`). Decode first so a
        // dormant server refuses malformed bodies on the same bounded path; while housing content
        // is loaded answer with the still-empty directory (reply reader `0x00c85530` needs only the
        // echoed correlation and a zero count). Read-only: nothing to stage or roll back.
        if let Some(request) = crate::housing_rating::CandidateInfoRequest::decode(inner)
            && self.housing.is_some()
            && request.system == crate::housing_rating::HOUSING_SYSTEM
        {
            let reply = crate::housing_rating::CandidateInfoReply {
                correlation: request.correlation,
                candidates: Vec::new(),
            };
            return Ok(Some(Outcome {
                replies: vec![reply.encode_inner()?],
                ..Outcome::default()
            }));
        }
        if !inner.starts_with(&FAMILY) {
            return Ok(None);
        }
        let Some(session) = self.housing.as_ref() else {
            return Ok(None);
        };
        // The occupation table: another activity (a fishing hole, a match, a race) owns the
        // player's scene, and a lot entered from it would keep that scene as its way back. Going
        // from one lot to another stays open.
        if matches!(decode_request(inner), Ok(Request::Enter { .. }))
            && self.activity_busy()
            && !self.is_occupied(occupation::Occupation::Housing)
        {
            return Ok(Some(Outcome {
                replies: vec![chat::system(
                    "Finish the active activity before entering a house.",
                )],
                ..Outcome::default()
            }));
        }
        let mut session = session.clone();
        let mut inventory = self.inventory.clone();
        let result = session
            .service
            .handle(character.guid, inner, &mut inventory);
        let outcome = match result {
            Ok(outcome) => outcome,
            Err(error) => {
                let mut replies = vec![chat::system(&error.denial_message())];
                let request = decode_request(inner);
                // Client reader 00ab3030: one length-prefixed reason string.
                if request.as_ref().is_ok_and(Request::answers_with_denial) {
                    replies.push(error.denial_reply().encode_inner()?);
                }
                // T872: a refused enter (a visit above all) clears the client's enter busy flag.
                if matches!(request, Ok(Request::Enter { .. })) {
                    replies.push(crate::housing_service::enter_failed(error.enter_failure()));
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
                house_guid,
                scene,
                spawn_position,
                spawn_rotation,
                ..
            }) => {
                // Entering from inside a house keeps the original world pose.
                session.return_to.get_or_insert_with(|| character.clone());
                session.house = Some(*house_guid);
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
                session.house = None;
                // The fixtures' 35/3s go out before the zone change, so no actor outlives the visit.
                out.replies = encode_replies(&outcome)?;
                out.scene_zoning = Some(self.world_zoning(&back)?);
            }
            None => {
                out.replies = encode_replies(&outcome)?;
                out.house_broadcast = house_broadcast(&session, &outcome)?;
            }
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
        session.house = None;
        let back = session.return_to.take()?;
        session.service.end_visit(back.guid);
        Some(back)
    }

    /// T872: the lot this character is inside, own or visited; it keys the shared house instance.
    pub fn visiting_house(&self) -> Option<u64> {
        self.housing.as_ref().filter(|s| s.in_house())?.house
    }

    /// T872: this character's own lots (never a visit's copy), for another player's 127/14 or
    /// 127/19.
    pub fn own_house_lots(&self) -> Vec<crate::housing::LotSnapshot> {
        (self.housing.as_ref())
            .and_then(HousingSession::state)
            .map(|state| state.lots)
            .unwrap_or_default()
    }

    /// T872: serve another character's lot to this visitor (`HousingService::admit_foreign_lot`).
    /// False when this session has no housing or the copy is refused.
    pub fn admit_foreign_lot(
        &mut self,
        lot: crate::housing::LotSnapshot,
        visitor: u64,
        visit_pass: bool,
    ) -> bool {
        (self.housing.as_mut()).is_some_and(|s| {
            s.service
                .admit_foreign_lot(lot, visitor, visit_pass)
                .is_ok()
        })
    }

    /// T872: an owner's edit reaching a visitor inside that lot: its copy is replaced, and the
    /// packets come back to send now. While the lot scene is still loading they join the packets
    /// sent once it has (the client needs 127/28 before any 127/4x). Nothing for anyone else.
    pub fn visit_update(
        &mut self,
        broadcast: &crate::housing_service::HouseBroadcast,
        visitor: u64,
    ) -> Vec<Vec<u8>> {
        let house = broadcast.house_guid;
        let Some(session) = (self.housing.as_mut())
            .filter(|s| s.in_house() && s.house == Some(house) && s.service.has_foreign_lot(house))
        else {
            return Vec::new();
        };
        if (session.service)
            .admit_foreign_lot(broadcast.lot.clone(), visitor, true)
            .is_err()
        {
            return Vec::new();
        }
        if session.pending.is_empty() {
            return broadcast.packets.clone();
        }
        session.pending.extend(broadcast.packets.iter().cloned());
        Vec::new()
    }

    /// While inside, the lot is the player's scene (like a soccer arena), so the server's
    /// membership registry registers it; a lot's visitors share its instance (T872).
    pub(super) fn house_zone(&self) -> Option<crate::encounter::ZoneTransition> {
        let session = self.housing.as_ref().filter(|s| s.in_house())?;
        scene_zone(session.scene.as_ref()?)
    }

    /// The op31 frame back to the saved world pose (house, kart race and fishing-hole returns).
    pub(super) fn world_zoning(
        &self,
        back: &CharacterState,
    ) -> io::Result<crate::zone_transition::ZoneTransition> {
        let world = self
            .encounter
            .as_ref()
            .ok_or_else(|| invalid("world return requires world content"))?
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
