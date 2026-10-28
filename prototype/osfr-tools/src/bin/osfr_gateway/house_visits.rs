//! T872 house visits: another player's lot, served to a visitor and kept live.
//!
//! Before a session's message is handled, each 127/19 (enter) or 127/14 (list houses) in it that
//! names a lot the session does not own gets a read-only copy of that lot from the owner's online
//! session, so the ordinary housing service answers it (`HousingService::admit_foreign_lot`).
//! The service's `can_visit` decides who gets in. A friend of the owner (the owner is on the
//! visitor's friends list; reference HouseAccess) gets a pass on their copy, lock included.
//! After an owner's edit commits, each visitor inside that lot gets its copy replaced and the
//! edit's packets. An owner who is not online has no copy to give: the visitor's enter is refused
//! with 127/46 code 1 (offline owners are a follow-up).
use super::*;
use osfr_tools::housing_service::{HouseBroadcast, Request, decode_request};

/// Copy the lots that the housing requests in `message` name into the visitor's staged gateway.
pub(super) fn admit(
    staged: &mut Gateway,
    sessions: &SessionMap,
    world: &SharedWorld,
    message: &[u8],
) {
    // Every message passes here, most of them movement: decode only one that carries a 127/19 or
    // 127/14 (application payloads are verbatim slices of the message).
    let named = |w: &[u8]| w == [127, 0, 19, 0] || w == [127, 0, 14, 0];
    if !message.windows(4).any(named) {
        return;
    }
    let visitor = staged.state().guid;
    let Ok(requests) = staged.application_requests(message) else {
        return;
    };
    for inner in requests {
        let (house, owner) = match decode_request(&inner) {
            Ok(Request::Enter { house_guid }) => (Some(house_guid), None),
            Ok(Request::RequestHouses { owner_guid }) => (None, Some(owner_guid)),
            _ => continue,
        };
        let lots = (sessions.values())
            .filter(|session| session.character_guid != visitor)
            .filter_map(|session| session.gateway.gameplay())
            .flat_map(Gameplay::own_house_lots)
            .filter(|lot| house == Some(lot.lot_guid) || owner == Some(lot.owner_guid));
        for mut lot in lots.collect::<Vec<_>>() {
            let friend = (world.social.profile(visitor))
                .is_some_and(|profile| profile.friends.contains_key(&lot.owner_guid));
            if friend {
                lot.locked = false;
            }
            if let Some(gameplay) = staged.gameplay_mut() {
                gameplay.admit_foreign_lot(lot, visitor, friend);
            }
        }
    }
}

/// After an owner's edit commits: replace each visitor's copy and send them the edit.
pub(super) fn fan_out(sessions: &mut SessionMap, owner: u64, broadcast: Option<HouseBroadcast>) {
    let Some(broadcast) = broadcast else {
        return;
    };
    for session in sessions.values_mut() {
        let visitor = session.character_guid;
        if visitor == owner {
            continue;
        }
        let Some(gameplay) = session.gateway.gameplay_mut() else {
            continue;
        };
        for packet in gameplay.visit_update(&broadcast, visitor) {
            if session
                .queue_shared_message(&wrap_application(&packet))
                .is_err()
            {
                event(json!({"event": "house_visit_send_refused", "session": session.id.0}));
                break;
            }
        }
    }
}
