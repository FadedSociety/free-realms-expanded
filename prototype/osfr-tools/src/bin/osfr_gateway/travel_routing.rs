//! Friend teleport (op 58): the host's side.
//!
//! The request names one character guid (`Ui.TeleportToFriend` `00c23b90`; the Atlas sends it for
//! a friend dot and a guild dot, the Friends window for a row). Only the host knows who that is
//! and where it stands, so before a session's message is handled each op 58 in it is resolved
//! here and the answer lent to that session's gameplay (`Gameplay::lend_friend`), which applies
//! its own rules (zone, cooldown, what the requester is doing) and moves the character.
//!
//! (R) Who may be joined: a mutual friend, a member of the same group or of the same guild, when
//! neither ignores the other. Where: in the open world and free by `Gameplay::travel_presence`.
//! Anything else is refused with the client's own wording, 74/8 code 4 (in combat), 11 (playing
//! a minigame) or 10 (cannot teleport to that player: offline, unknown, in a house, not a
//! friend), so a probing client learns no more than the friends list already shows.
use super::*;
use osfr_tools::gameplay::{FriendLend, Presence};
use osfr_tools::social_lists::FriendResultCode;

const OPEN_WORLD_ZONE: u32 = 0;

/// Resolve the op 58 requests in `message` for the staged gateway's character.
pub(super) fn lend(
    staged: &mut Gateway,
    sessions: &SessionMap,
    world: &SharedWorld,
    message: &[u8],
) {
    // A lend never outlives the message it was made for.
    if let Some(gameplay) = staged.gameplay_mut() {
        gameplay.lend_friend(None);
    }
    // Every message passes here, most of them movement: decode only one that may hold an op 58.
    if !message.windows(2).any(|w| w == [58, 0]) {
        return;
    }
    let requester = staged.state().guid;
    let Ok(requests) = staged.application_requests(message) else {
        return;
    };
    for inner in requests {
        let Some(([58, 0], guid)) = inner.split_first_chunk::<2>() else {
            continue;
        };
        let Ok(guid) = <[u8; 8]>::try_from(guid).map(u64::from_le_bytes) else {
            continue;
        };
        // L1177: prefer the session that selected `guid`; an unbound handshake carries guid 1.
        let target = (sessions.values())
            .filter(|session| session.character_guid == guid)
            .max_by_key(|session| session.selected_character)
            .map(|session| {
                let presence = (session.gateway.gameplay()).map(|g| g.travel_presence());
                (session.gateway.state().clone(), presence)
            });
        let lend = resolve(requester, guid, related(world, requester, guid), target);
        let lend = match lend {
            Ok(position) => FriendLend::Go { guid, position },
            Err(code) => FriendLend::Refuse {
                guid,
                packet: world.social.teleport_refusal(code, guid),
            },
        };
        if let Some(gameplay) = staged.gameplay_mut() {
            gameplay.lend_friend(Some(lend));
        }
    }
}

fn related(world: &SharedWorld, a: u64, b: u64) -> bool {
    let same = |x: Option<u64>, y: Option<u64>| x.is_some() && x == y;
    let group = |guid| world.social.group_of(guid).map(u64::from);
    let guild = |guid| world.guilds.store().guild_guid_for(guid);
    a != b
        && !world.social.ignores_either(a, b)
        && (world.social.house_friends(a, b)
            || same(group(a), group(b))
            || same(guild(a), guild(b)))
}

/// Where the requester arrives, or the refusal code.
fn resolve(
    requester: u64,
    guid: u64,
    related: bool,
    target: Option<(CharacterState, Option<Presence>)>,
) -> Result<[f32; 3], FriendResultCode> {
    let refused = FriendResultCode::CannotTeleportPlayer;
    let (state, presence) = target
        .filter(|_| related && requester != guid)
        .ok_or(refused)?;
    match presence {
        Some(Presence::Combat) => Err(FriendResultCode::CannotTeleportCombat),
        Some(Presence::Minigame) => Err(FriendResultCode::CannotTeleportMiniGame),
        Some(Presence::Free) if state.zone_id == OPEN_WORLD_ZONE => Ok(state.position),
        _ => Err(refused),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(zone_id: u32) -> CharacterState {
        CharacterState {
            guid: 2,
            zone_id,
            position: [10.0, 5.0, -3.0],
            rotation: [0.0, 0.0, 1.0],
        }
    }

    #[test]
    fn only_a_related_free_player_in_the_open_world_can_be_joined() {
        let free = Some(Presence::Free);
        assert_eq!(
            resolve(1, 2, true, Some((at(0), free))),
            Ok([10.0, 5.0, -3.0])
        );
        let player = Err(FriendResultCode::CannotTeleportPlayer);
        // A stranger, yourself, someone offline, in another scene or in a house.
        assert_eq!(resolve(1, 2, false, Some((at(0), free))), player);
        assert_eq!(resolve(2, 2, true, Some((at(0), free))), player);
        assert_eq!(resolve(1, 2, true, None), player);
        assert_eq!(resolve(1, 2, true, Some((at(7), free))), player);
        assert_eq!(
            resolve(1, 2, true, Some((at(0), Some(Presence::Away)))),
            player
        );
        assert_eq!(resolve(1, 2, true, Some((at(0), None))), player);
        // The client words these two itself.
        assert_eq!(
            resolve(1, 2, true, Some((at(0), Some(Presence::Minigame)))),
            Err(FriendResultCode::CannotTeleportMiniGame)
        );
        assert_eq!(
            resolve(1, 2, true, Some((at(9), Some(Presence::Combat)))),
            Err(FriendResultCode::CannotTeleportCombat)
        );
        // A stranger in a minigame still reads as "cannot teleport to that player".
        assert_eq!(
            resolve(1, 2, false, Some((at(0), Some(Presence::Minigame)))),
            player
        );
    }
}
