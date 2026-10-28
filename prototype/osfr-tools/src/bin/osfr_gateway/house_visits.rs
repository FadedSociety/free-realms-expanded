//! T872 house visits: another player's lot, served to a visitor and kept live.
//!
//! Before a session's message is handled, each 127/19 (enter) or 127/14 (list houses) in it that
//! names a lot the visitor may enter gets a read-only copy of that lot from the owner's online
//! session, so the ordinary housing service answers it (`HousingService::admit_foreign_lot`).
//! After the message, `settle` drops every copy except the lot the visitor is inside, so a listing
//! or a refused enter keeps nothing. After an owner's edit commits, each visitor inside that lot
//! gets its copy replaced and the edit's packets.
//!
//! Who may enter (F874, the T872 review): `may_visit`. A lot the visitor may not enter is never
//! copied, so the visitor's 127/19 is refused with 127/46 code 1, exactly as for an owner who is
//! not online: the code does not reveal whether the owner is online (offline owners are a
//! follow-up).
use super::*;
use osfr_tools::housing::{LotSnapshot, PERMISSION_CAN_VISIT};
use osfr_tools::housing_service::{HouseBroadcast, Request, decode_request};

/// Whether `visitor` may enter `lot`. Never when either ignores the other. A house friend (mutual,
/// `SocialService::house_friends`) always may, lock included (reference HouseAccess). Anyone else
/// only with a visit permission on an unlocked lot.
fn may_visit(lot: &LotSnapshot, visitor: u64, house_friends: bool, ignored: bool) -> bool {
    if ignored || lot.owner_guid == visitor {
        return false;
    }
    let permitted = (lot.permissions.iter())
        .any(|permission| permission.guid == visitor && permission.level >= PERMISSION_CAN_VISIT);
    house_friends || (permitted && !lot.locked)
}

/// Copy the lots the housing requests in `message` name, and the visitor may enter, into its
/// staged gateway.
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
    let others = || {
        (sessions.values())
            .filter(|session| session.character_guid != visitor)
            .filter_map(|session| Some((session.character_guid, session.gateway.gameplay()?)))
    };
    for inner in requests {
        // One lot cloned for an enter; one owner's lots for a listing.
        let lots = match decode_request(&inner) {
            Ok(Request::Enter { house_guid }) => others()
                .find_map(|(_, gameplay)| gameplay.own_house_lot(house_guid))
                .into_iter()
                .collect(),
            Ok(Request::RequestHouses { owner_guid }) => others()
                .find(|(guid, _)| *guid == owner_guid)
                .map(|(_, gameplay)| gameplay.own_house_lots())
                .unwrap_or_default(),
            _ => continue,
        };
        for lot in lots {
            let owner = lot.owner_guid;
            let friends = world.social.house_friends(owner, visitor);
            let ignored = world.social.ignores_either(owner, visitor);
            if !may_visit(&lot, visitor, friends, ignored) {
                continue;
            }
            if let Some(gameplay) = staged.gameplay_mut() {
                gameplay.admit_foreign_lot(lot, visitor);
            }
        }
    }
}

/// After the message: keep only the copy of the lot the visitor is inside.
pub(super) fn settle(staged: &mut Gateway) {
    if let Some(gameplay) = staged.gameplay_mut() {
        gameplay.drop_unvisited_foreign_lots();
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

#[cfg(test)]
mod tests {
    use super::*;
    use osfr_tools::housing::LotPermission;

    const OWNER: u64 = 1;
    const GUEST: u64 = 2;

    fn lot(locked: bool, permission: Option<i32>) -> LotSnapshot {
        LotSnapshot {
            owner_guid: OWNER,
            property_id: 1,
            lot_guid: 0xA,
            zone_id: 1,
            name_id: 1,
            max_fixtures: 1,
            max_landmarks: 0,
            count: 0,
            build_areas: Vec::new(),
            fixtures: Vec::new(),
            permissions: (permission.into_iter())
                .map(|level| LotPermission { guid: GUEST, level })
                .collect(),
            locked,
        }
    }

    /// F874: a stranger is never let in; a house friend is, lock included; a visit permission
    /// only on an unlocked lot; an ignore refuses everyone; the owner is never a visitor.
    #[test]
    fn only_house_friends_and_permitted_visitors_may_enter() {
        assert!(
            !may_visit(&lot(false, None), GUEST, false, false),
            "a stranger"
        );
        assert!(
            may_visit(&lot(true, None), GUEST, true, false),
            "a friend, lock included"
        );
        assert!(
            may_visit(&lot(false, Some(1)), GUEST, false, false),
            "a visit permission"
        );
        assert!(
            !may_visit(&lot(true, Some(3)), GUEST, false, false),
            "a locked lot"
        );
        assert!(
            !may_visit(&lot(false, Some(0)), GUEST, false, false),
            "level 0"
        );
        assert!(
            !may_visit(&lot(false, Some(1)), GUEST, true, true),
            "an ignore refuses all"
        );
        assert!(
            !may_visit(&lot(false, None), OWNER, true, false),
            "the owner"
        );
    }
}
