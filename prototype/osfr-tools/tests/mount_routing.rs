//! W162: routing of the mount service (family 168) without the gateway: request decoding,
//! summon and dismount from inventory ownership, the server-side ride endings, what a second
//! player in interest range sees, and the `mounted` signal of the emote rules.
//!
//! RECOVERED (client): the request shapes (168/1, 3, 6, 8), the roster list and 168/2 / 168/4
//! reply layouts, and the emote button's mount gate. RECONSTRUCTED (R): the ride GUID, the
//! one-request dismount that also removes the ride, refusal notices and the AddNpc field values.
//! The gateway wiring is in `lab/runs/claude/W162/shared-patch` (lead-owned files).
use osfr_tools::{
    inventory::{InventoryItem, InventoryState},
    mounts::{
        MountCatalog, MountEffectKind, MountRequest, MountService, RideContext, RidePose,
        RideRestrictions, RideWire, decode_request,
    },
    players::{PlayerRegistry, SessionId, ZoneInstance},
    social_chat::{
        QuickChatCatalog, emote_context,
        piece_093::{EMOTE_ACTIONS, EmoteRejection},
    },
};
use std::path::PathBuf;

const OWNER: u64 = 7;
const OTHER: u64 = 8;
/// Skeletal Dragon (mount 715) and the one item definition that grants it in `mounts.json`.
const DRAGON: i32 = 715;
const DRAGON_ITEM: i32 = 10530;

fn catalog() -> MountCatalog {
    MountCatalog::load(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/mounts.json"))
        .unwrap()
}

fn inventory(definitions: &[i32]) -> InventoryState {
    InventoryState {
        coins: 0,
        items: definitions
            .iter()
            .enumerate()
            .map(|(index, definition_id)| InventoryItem {
                definition_id: *definition_id,
                tint: 0,
                id: index as i32 + 1,
                count: 1,
                consumed_count: 0,
                stored_count: 0,
                last_cast_time: i64::from(i32::MAX),
                rental: None,
                ability_count: 0,
                activate_enabled: false,
            })
            .collect(),
        equipment: Vec::new(),
        next_instance_id: None,
    }
}

fn open() -> RideRestrictions {
    RideRestrictions {
        zone_allows_riding: true,
        in_minigame: false,
        ability_active: false,
        knocked_out: false,
    }
}

fn pose() -> RidePose {
    RidePose {
        position: [10.0, 2.0, 30.0],
        heading: 0.5,
    }
}

fn ctx<'a>(
    inventory: &'a InventoryState,
    restrictions: RideRestrictions,
    now_ms: u64,
) -> RideContext<'a> {
    RideContext {
        guid: OWNER,
        inventory,
        restrictions,
        pose: pose(),
        now_ms,
    }
}

fn spawn_request(mount_id: i32) -> Vec<u8> {
    let mut bytes = vec![168, 0, 6];
    bytes.extend(mount_id.to_le_bytes());
    bytes
}

fn item_request(item: i32) -> Vec<u8> {
    let mut bytes = vec![168, 0, 8];
    bytes.extend(item.to_le_bytes());
    bytes.push(1);
    bytes
}

fn mount_request(target: u64) -> Vec<u8> {
    // Target GUID, then the third constructor dword and the two flag bytes (008b3730).
    let mut bytes = vec![168, 0, 1];
    bytes.extend(target.to_le_bytes());
    bytes.extend([1, 0, 0, 0, 0, 0]);
    bytes
}

const DISMOUNT_REQUEST: [u8; 3] = [168, 0, 3];

fn handle(service: &mut MountService, inner: &[u8], inv: &InventoryState, now_ms: u64) -> RideWire {
    service
        .handle(inner, &ctx(inv, open(), now_ms))
        .unwrap()
        .expect("a mount request")
}

fn sub_opcodes(packets: &[Vec<u8>]) -> Vec<(u16, u8)> {
    packets
        .iter()
        .map(|p| (u16::from_le_bytes([p[0], p[1]]), p[2]))
        .collect()
}

#[test]
fn request_bodies_are_decoded_strictly() {
    assert_eq!(
        decode_request(&spawn_request(DRAGON)).unwrap(),
        Some(MountRequest::Spawn { mount_id: DRAGON })
    );
    assert_eq!(
        decode_request(&item_request(DRAGON_ITEM)).unwrap(),
        Some(MountRequest::SpawnByItem {
            item_definition_id: DRAGON_ITEM
        })
    );
    assert_eq!(
        decode_request(&mount_request(0x1122_3344_5566_7788)).unwrap(),
        Some(MountRequest::Mount {
            target: 0x1122_3344_5566_7788
        })
    );
    assert_eq!(
        decode_request(&DISMOUNT_REQUEST).unwrap(),
        Some(MountRequest::Dismount)
    );
    // Other families and the client-bound sub-opcodes are not requests.
    assert_eq!(decode_request(&[42, 0, 6, 0]).unwrap(), None);
    assert_eq!(decode_request(&[168, 0, 5, 0, 0, 0, 0]).unwrap(), None);
    assert_eq!(decode_request(&[168]).unwrap(), None);
    // Truncated, padded or oversized bodies of a handled request are errors, never panics.
    for bad in [
        vec![168, 0, 6],
        vec![168, 0, 6, 1, 0, 0, 0, 0],
        vec![168, 0, 8, 1, 0, 0, 0],
        vec![168, 0, 1, 1, 2, 3],
        [vec![168, 0, 1], vec![0; 200]].concat(),
        [vec![168, 0, 3], vec![0; 200]].concat(),
    ] {
        assert!(decode_request(&bad).is_err(), "{bad:?}");
    }
}

#[test]
fn an_owned_ride_is_summoned_boarded_and_put_away() {
    let owned = inventory(&[DRAGON_ITEM]);
    let mut service = MountService::new(catalog());
    let actor = MountService::ride_guid(OWNER);
    assert_ne!(actor, OWNER);

    let spawned = handle(&mut service, &spawn_request(DRAGON), &owned, 0);
    // The owner's roster entry carries the ride actor's GUID before the actor appears.
    assert_eq!(sub_opcodes(&spawned.private), vec![(168, 5)]);
    assert!(
        spawned.private[0]
            .windows(8)
            .any(|w| w == actor.to_le_bytes())
    );
    // The ride is an AddNpc (35/2) whose first field is its GUID, at the owner's pose.
    assert_eq!(spawned.shared.len(), 1);
    assert_eq!(&spawned.shared[0][..4], &[35, 0, 2, 0]);
    assert_eq!(&spawned.shared[0][4..12], &actor.to_le_bytes());
    assert!(!service.is_mounted());
    assert_eq!(spawned.notice, None);

    // The client automounts: 168/1 names the actor it saw. The reply is shared.
    let boarded = handle(&mut service, &mount_request(actor), &owned, 10);
    assert_eq!(sub_opcodes(&boarded.shared), vec![(168, 2)]);
    assert!(boarded.private.is_empty());
    assert_eq!(&boarded.shared[0][3..11], &OWNER.to_le_bytes());
    assert_eq!(&boarded.shared[0][11..19], &actor.to_le_bytes());
    assert!(service.is_mounted());

    // One 168/3 dismounts and removes the ride; a repeat is a no-op.
    let away = handle(&mut service, &DISMOUNT_REQUEST, &owned, 20);
    assert_eq!(sub_opcodes(&away.shared[..1]), vec![(168, 4)]);
    assert_eq!(&away.shared[1][..6], &[35, 0, 3, 0, 1, 0]);
    assert_eq!(&away.shared[1][6..14], &actor.to_le_bytes());
    assert_eq!(sub_opcodes(&away.private), vec![(168, 5)]);
    assert!(!service.is_mounted());
    assert_eq!(service.view(OWNER), None);
    let again = handle(&mut service, &DISMOUNT_REQUEST, &owned, 30);
    assert_eq!(again, RideWire::default());

    // The item path (168/8) summons the same ride.
    let by_item = handle(&mut service, &item_request(DRAGON_ITEM), &owned, 5_000);
    assert_eq!(by_item.shared.len(), 1);
    assert!(service.view(OWNER).is_some());
}

#[test]
fn a_ride_the_player_does_not_own_is_refused() {
    let mut service = MountService::new(catalog());
    // Another item and no item: neither grants the Skeletal Dragon.
    for inv in [inventory(&[]), inventory(&[1, 2, 3])] {
        let refused = handle(&mut service, &spawn_request(DRAGON), &inv, 0);
        assert!(refused.shared.is_empty());
        assert_eq!(refused.notice, Some("You do not own that ride."));
        // The owner still gets a roster refresh so the client UI resets.
        assert_eq!(sub_opcodes(&refused.private), vec![(168, 5)]);
        let by_item = handle(&mut service, &item_request(DRAGON_ITEM), &inv, 0);
        assert_eq!(by_item.notice, Some("You do not own that ride."));
    }
    // Unknown ride ids and item ids, and a mount request with nothing summoned.
    let owned = inventory(&[DRAGON_ITEM]);
    assert!(
        handle(&mut service, &spawn_request(-1), &owned, 0)
            .notice
            .is_some()
    );
    assert!(
        handle(&mut service, &item_request(1), &owned, 0)
            .notice
            .is_some()
    );
    assert!(
        handle(
            &mut service,
            &mount_request(MountService::ride_guid(OWNER)),
            &owned,
            0
        )
        .shared
        .is_empty()
    );
    // A hostile 168/1 naming some other actor boards nothing.
    handle(&mut service, &spawn_request(DRAGON), &owned, 0);
    let wrong = handle(&mut service, &mount_request(OTHER), &owned, 10);
    assert!(wrong.shared.is_empty() && wrong.notice.is_some());
    assert!(!service.is_mounted());
}

#[test]
fn restrictions_block_summoning_and_boarding() {
    let owned = inventory(&[DRAGON_ITEM]);
    for blocked in [
        RideRestrictions {
            in_minigame: true,
            ..open()
        },
        RideRestrictions {
            knocked_out: true,
            ..open()
        },
    ] {
        let mut service = MountService::new(catalog());
        let wire = service
            .handle(&spawn_request(DRAGON), &ctx(&owned, blocked, 0))
            .unwrap()
            .unwrap();
        assert!(wire.shared.is_empty());
        assert_eq!(wire.notice, Some("You cannot use a ride here right now."));
        assert!(service.view(OWNER).is_none());
    }
}

/// Zoning, combat start, knockout and entering a minigame all end the ride through the same
/// `dismount_request`: a rider is dismounted, the actor removed, the roster refreshed.
#[test]
fn every_server_trigger_dismounts_and_removes_the_ride() {
    let owned = inventory(&[DRAGON_ITEM]);
    let actor = MountService::ride_guid(OWNER);
    for trigger in ["zoning", "combat", "knockout", "minigame"] {
        for mounted in [true, false] {
            let mut service = MountService::new(catalog());
            handle(&mut service, &spawn_request(DRAGON), &owned, 0);
            if mounted {
                handle(&mut service, &mount_request(actor), &owned, 10);
            }
            assert_eq!(service.is_mounted(), mounted);
            let outcome = if trigger == "minigame" {
                service.on_minigame_start(OWNER, &owned, &[])
            } else {
                service.dismount_request(OWNER, &owned, &[])
            };
            let wire = service.wire(&outcome, pose());
            let removed = wire
                .shared
                .iter()
                .any(|p| p[..6] == [35, 0, 3, 0, 1, 0] && p[6..14] == actor.to_le_bytes());
            assert!(removed, "{trigger}: the ride actor is removed");
            assert_eq!(
                wire.shared.iter().any(|p| p[..3] == [168, 0, 4]),
                mounted,
                "{trigger}: a rider gets 168/4"
            );
            assert!(!service.is_mounted(), "{trigger}");
            assert!(service.view(OWNER).is_none(), "{trigger}");
        }
    }
    // After a knockout the same player cannot board again until revived.
    let mut service = MountService::new(catalog());
    let down = RideRestrictions {
        knocked_out: true,
        ..open()
    };
    let wire = service
        .handle(&spawn_request(DRAGON), &ctx(&owned, down, 5_000))
        .unwrap()
        .unwrap();
    assert!(wire.shared.is_empty());
}

#[test]
fn a_second_player_in_range_sees_the_ride_and_its_removal() {
    let owned = inventory(&[DRAGON_ITEM]);
    let zone = ZoneInstance {
        zone_id: 1,
        instance_id: 1,
    };
    let (a, b, far, late) = (SessionId(1), SessionId(2), SessionId(3), SessionId(4));
    let mut players =
        PlayerRegistry::new(osfr_tools::players::InterestPolicy::new(100.0, 64).unwrap());
    for (session, guid, x) in [(a, OWNER, 10.0), (b, OTHER, 40.0), (far, 9, 5_000.0)] {
        players
            .join(session, guid, guid, zone, [x, 2.0, 30.0], 0.5)
            .unwrap();
    }
    let watchers = |players: &PlayerRegistry| {
        players
            .nearby_sessions(zone, pose().position)
            .into_iter()
            .filter(|session| *session != a)
            .collect::<Vec<_>>()
    };

    // The gateway delivers `shared` to the owner's neighbours; B sees it, the distant player not.
    let mut service = MountService::new(catalog());
    let spawned = handle(&mut service, &spawn_request(DRAGON), &owned, 0);
    let boarded = handle(
        &mut service,
        &mount_request(MountService::ride_guid(OWNER)),
        &owned,
        10,
    );
    assert_eq!(watchers(&players), vec![b]);
    assert_eq!(&spawned.shared[0][..4], &[35, 0, 2, 0]);
    assert_eq!(sub_opcodes(&boarded.shared), vec![(168, 2)]);

    // A player who comes into range afterwards gets the same scene from the view: the ride
    // actor, then the rider's attachment (168/2) once both exist on that client.
    players
        .join(late, 10, 10, zone, [20.0, 2.0, 30.0], 0.0)
        .unwrap();
    assert_eq!(watchers(&players), vec![b, late]);
    let view = service.view(OWNER).expect("a mounted rider has a view");
    assert_eq!(view.seat, Some(0));
    let scene = view.packets(pose());
    assert_eq!(&scene[0][..4], &[35, 0, 2, 0]);
    assert_eq!(scene[1], boarded.shared[0]);
    // A ride that is summoned but not boarded has no attachment to replay.
    let mut idle = MountService::new(catalog());
    handle(&mut idle, &spawn_request(DRAGON), &owned, 0);
    assert_eq!(idle.view(OWNER).unwrap().packets(pose()).len(), 1);

    // Putting the ride away removes it for everyone in range.
    let away = handle(&mut service, &DISMOUNT_REQUEST, &owned, 20);
    assert_eq!(away.shared[1][6..14], view.removal_packet()[6..14]);
}

#[test]
fn a_mounted_rider_cannot_emote() {
    let (phrase, animation) = {
        let row = EMOTE_ACTIONS.first().expect("a recovered emote");
        (row.quick_chat_id as i32, row.animation_id as i32)
    };
    let catalog_rows = QuickChatCatalog::from_rows([(phrase, animation)]);
    let owned = inventory(&[DRAGON_ITEM]);
    let mut service = MountService::new(catalog());
    let can_emote = |service: &MountService| {
        catalog_rows.authorize_emote(phrase, emote_context(service.is_mounted(), None, 1_000))
    };
    assert!(can_emote(&service).is_ok());
    handle(&mut service, &spawn_request(DRAGON), &owned, 0);
    assert!(
        can_emote(&service).is_ok(),
        "summoned but not yet on the ride"
    );
    handle(
        &mut service,
        &mount_request(MountService::ride_guid(OWNER)),
        &owned,
        10,
    );
    assert!(service.is_mounted());
    let refused = can_emote(&service);
    assert!(
        matches!(
            refused,
            Err(osfr_tools::social_chat::ChatError::EmoteRefused(
                EmoteRejection::Mounted
            ))
        ),
        "{refused:?}"
    );
    handle(&mut service, &DISMOUNT_REQUEST, &owned, 20);
    assert!(can_emote(&service).is_ok());
}

/// Every ride in the shipped content can be summoned: it has a model and builds a valid AddNpc
/// whose tunnel frame is stripped back to the inner application packet.
#[test]
fn every_catalogued_ride_builds_a_valid_actor() {
    let catalog = catalog();
    let mut seen = 0;
    for mount in catalog.mounts() {
        let item = mount.item_definition_id_candidates[0];
        let owned = inventory(&[item]);
        let mut service = MountService::new(catalog.clone());
        let wire = handle(&mut service, &spawn_request(mount.id), &owned, 0);
        assert!(wire.notice.is_none(), "{}", mount.name);
        assert_eq!(wire.shared.len(), 1, "{}", mount.name);
        assert_eq!(&wire.shared[0][..4], &[35, 0, 2, 0], "{}", mount.name);
        seen += 1;
    }
    assert_eq!(seen, catalog.mounts().count());
    assert!(seen >= 50);
}

#[test]
fn effects_keep_the_owner_roster_private() {
    let owned = inventory(&[DRAGON_ITEM]);
    let mut service = MountService::new(catalog());
    let outcome = service
        .spawn(
            OWNER,
            &owned,
            DRAGON,
            MountService::ride_guid(OWNER),
            open(),
            0,
            &[],
        )
        .unwrap();
    let flags: Vec<(bool, bool)> = outcome
        .effects
        .iter()
        .map(|e| {
            (
                e.nearby(),
                matches!(e.kind, MountEffectKind::SpawnRide { .. }),
            )
        })
        .collect();
    // [roster list (private), ride actor (public)]
    assert_eq!(flags, vec![(false, false), (true, true)]);
}
