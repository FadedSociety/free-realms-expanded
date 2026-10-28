//! T507/T552: the per-tick ambient animation pass for passive content NPCs.
//!
//! The table and its stateless schedules live in `osfr_tools::npc_ambient`, so
//! every viewer that knows an actor plays the same 35/8, 35/26 and 35/17 in the
//! same window. A look-at target is resolved for each viewer's pass from the
//! shared player grid (`PlayerRegistry::nearest_session`: the same answer for
//! every viewer, one grid query per viewer per look-at firing, at most the
//! per-viewer cap); it is sent only to viewers that already know that player.
//! One viewer's full reliable queue is refused for that viewer only; the shared
//! tick and every other session keep running.
use super::*;
use osfr_tools::npc_ambient::AmbientUpdate;

/// Queue this viewer's due ambient packets for `(from_ms, to_ms]`.
pub(super) fn queue_due(
    session: &mut Session,
    world: (&PlayerRegistry, &World, ZoneInstance, i32),
    from_ms: u64,
    to_ms: u64,
) {
    let (players, membership, zone, instance) = world;
    let Some(game) = session.gateway.gameplay() else {
        return;
    };
    let Some(ambient) = game.npc_ambient() else {
        return;
    };
    let due = ambient.due_for_known(
        &session.known_npcs,
        |guid| game.npc_model(guid),
        from_ms,
        to_ms,
    );
    let viewer = session.id;
    for update in due {
        let target = match &update {
            AmbientUpdate::LookAt {
                position, radius, ..
            } => {
                let nearest =
                    nearest_target(players, membership, zone, instance, *position, *radius);
                visible_target(nearest, viewer, |character| {
                    players.knows_character(viewer, character)
                })
            }
            _ => Some(0),
        };
        let Some(target) = target else {
            continue;
        };
        for packet in update.packets(target) {
            if let Err(error) = session.queue_shared_message(&wrap_application(&packet)) {
                Server::refuse_field_delivery(session, error);
                return;
            }
        }
    }
}

#[derive(Clone, Copy)]
struct LookAtTarget {
    session: SessionId,
    character: u64,
}

/// What the viewer may be told: `0` (clear) when nobody is in range, the target's
/// character GUID when it is the viewer itself or the viewer already received its
/// AddPc, and `None` (send nothing) otherwise, so a look-at never reveals a player
/// the viewer cannot see.
fn visible_target(
    nearest: Option<LookAtTarget>,
    viewer: SessionId,
    knows: impl FnOnce(u64) -> bool,
) -> Option<u64> {
    match nearest {
        None => Some(0),
        Some(target) if target.session == viewer || knows(target.character) => {
            Some(target.character)
        }
        Some(_) => None,
    }
}

/// Nearest ready player in the same instance and inside `radius`.
fn nearest_target(
    players: &PlayerRegistry,
    membership: &World,
    zone: ZoneInstance,
    instance: i32,
    position: [f32; 3],
    radius: f32,
) -> Option<LookAtTarget> {
    players
        .nearest_session(zone, position, radius, |player| {
            membership
                .player_location(player.character_guid)
                .is_some_and(|location| location.instance_id == instance && location.ready)
        })
        .map(|player| LookAtTarget {
            session: player.session_id,
            character: player.character_guid,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const ZONE: ZoneInstance = ZoneInstance {
        zone_id: 0,
        instance_id: 0,
    };

    /// Players 1..=3 stand 2, 4 and 40 m from the origin; `ready` is each one's
    /// membership flag. Everyone is in the open-world instance 0.
    fn rig(ready: [bool; 3]) -> (PlayerRegistry, World) {
        let mut players = PlayerRegistry::new(InterestPolicy::new(100.0, 16).unwrap());
        let mut membership = World::runtime();
        for (index, (x, ready)) in [2.0, 4.0, 40.0].into_iter().zip(ready).enumerate() {
            let id = index as u64 + 1;
            players
                .join(SessionId(id), id + 500, id + 600, ZONE, [x, 0.0, 0.0], 0.0)
                .unwrap();
            let pose = WorldPose {
                position: [x, 0.0, 0.0, 1.0],
                rotation: [0.0, 0.0, 1.0, 0.0],
            };
            let change = membership
                .prepare_membership(id + 600, 0, Some(0), pose, ready, 1_000)
                .unwrap();
            membership.commit_membership(change);
        }
        (players, membership)
    }

    fn nearest(players: &PlayerRegistry, world: &World, instance: i32, radius: f32) -> Option<u64> {
        nearest_target(players, world, ZONE, instance, [0.0; 3], radius).map(|t| t.character)
    }

    /// The target is the nearest ready player of the same instance inside the
    /// radius: a nearer player who is not ready, or in another instance, never wins.
    #[test]
    fn the_look_at_target_is_the_nearest_ready_player_of_the_same_instance() {
        if !super::super::tests::test_support::bin_test_target("t552") {
            return;
        }
        let (players, world) = rig([true, true, true]);
        assert_eq!(nearest(&players, &world, 0, 10.0), Some(601));
        assert_eq!(
            nearest(&players, &world, 0, 1.0),
            None,
            "outside the radius"
        );
        assert_eq!(nearest(&players, &world, 0, 30.0), Some(601));

        let (players, world) = rig([false, true, true]);
        assert_eq!(
            nearest(&players, &world, 0, 10.0),
            Some(602),
            "a player still loading is skipped"
        );
        let (players, world) = rig([false, false, true]);
        assert_eq!(nearest(&players, &world, 0, 10.0), None);
        assert_eq!(
            nearest(&players, &world, 7, 10.0),
            None,
            "a player of another instance is never a target"
        );
    }

    /// A viewer is told only what it can already see: nothing in range clears
    /// (0), itself or a known player is named, an unknown player sends nothing.
    #[test]
    fn a_look_at_never_names_a_player_the_viewer_cannot_see() {
        if !super::super::tests::test_support::bin_test_target("t552") {
            return;
        }
        let target = LookAtTarget {
            session: SessionId(2),
            character: 602,
        };
        let viewer = SessionId(1);
        assert_eq!(visible_target(None, viewer, |_| false), Some(0));
        assert_eq!(visible_target(Some(target), viewer, |_| true), Some(602));
        assert_eq!(
            visible_target(Some(target), SessionId(2), |_| false),
            Some(602),
            "a viewer may be told it is the target itself"
        );
        assert_eq!(
            visible_target(Some(target), viewer, |_| false),
            None,
            "an unknown player is not revealed"
        );
    }
}
