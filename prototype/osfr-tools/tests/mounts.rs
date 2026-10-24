use osfr_tools::{inventory, mounts};

use inventory::{Inventory, InventoryItem, InventoryState};
use mounts::{
    ActiveRide, MountCatalog, MountDefinition, MountEffectKind, MountError, MountService,
    MountSource, MountState, MountType, RideContext, RidePose, RideRestrictions,
};
use std::path::PathBuf;

const OWNER: u64 = 0x1112_1314_1516_1718;
const OBSERVER: u64 = 0x2122_2324_2526_2728;
const MOUNT_ACTOR: u64 = 0x0102_0304_0506_0708;
const ITEM_DEFINITION: i32 = 70;
const MOUNT_ID: i32 = 7;

fn source() -> Vec<MountSource> {
    vec![MountSource::Reconstructed(
        "synthetic test data (R)".to_owned(),
    )]
}

fn catalog() -> MountCatalog {
    MountCatalog::new(
        vec![MountType {
            id: 1,
            anim_slot_prefix: "saddleground".to_owned(),
            description: "ground mount".to_owned(),
            source: vec![MountSource::Client("synthetic fixture".to_owned())],
        }],
        vec![MountDefinition {
            id: MOUNT_ID,
            item_definition_id_candidates: vec![ITEM_DEFINITION],
            name: "Synthetic Ride".to_owned(),
            name_string_id: 10,
            image_set_id: 11,
            model_id: Some(33),
            mount_type_id: Some(1),
            max_movement_speed: 12.5,
            members_only: false,
            is_upgradable: true,
            is_upgraded: false,
            tint_id: 0,
            tint_alias: "dyetint".to_owned(),
            seat_count: 1,
            stats: Default::default(),
            source: source(),
            field_sources: Default::default(),
        }],
        8.0,
        Default::default(),
    )
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

fn restrictions() -> RideRestrictions {
    RideRestrictions {
        zone_allows_riding: true,
        in_minigame: false,
        ability_active: false,
        knocked_out: false,
    }
}

fn packet_effects(outcome: &mounts::MountOutcome) -> Vec<(Vec<u64>, Vec<u8>)> {
    outcome
        .effects
        .iter()
        .filter_map(|effect| match &effect.kind {
            MountEffectKind::Packet(bytes) => Some((effect.recipients.clone(), bytes.clone())),
            _ => None,
        })
        .collect()
}

fn expected_mount_list(actor_guid: u64) -> Vec<u8> {
    let mut bytes = vec![0xa8, 0x00, 0x05, 1, 0, 0, 0];
    // T482: the row id is the inventory instance id (the fixture's first item), not the ride id.
    bytes.extend_from_slice(&1_i32.to_le_bytes());
    bytes.extend_from_slice(&10_i32.to_le_bytes());
    bytes.extend_from_slice(&11_i32.to_le_bytes());
    bytes.extend_from_slice(&actor_guid.to_le_bytes());
    bytes.push(0); // MembersOnly
    bytes.extend_from_slice(&0_i32.to_le_bytes()); // Tint
    bytes.extend_from_slice(&7_i32.to_le_bytes());
    bytes.extend_from_slice(b"dyetint");
    bytes.extend_from_slice(&[1, 0]); // IsUpgradable, IsUpgraded
    bytes
}

fn expected_mount_response() -> Vec<u8> {
    let mut bytes = vec![0xa8, 0x00, 0x02];
    bytes.extend_from_slice(&OWNER.to_le_bytes());
    bytes.extend_from_slice(&MOUNT_ACTOR.to_le_bytes());
    bytes.extend_from_slice(&0_i32.to_le_bytes()); // seat
    bytes.extend_from_slice(&1_i32.to_le_bytes()); // queue position: the client mounts on 1
    bytes.extend_from_slice(&1_i32.to_le_bytes()); // (R) Mount's fourth argument, as in the AddPc tail
    bytes.extend_from_slice(&0_i32.to_le_bytes()); // (R) stored on the rider (+0x17bc)
    bytes.extend_from_slice(&0.0_f32.to_le_bytes()); // (R) Mount's last argument
    bytes
}

fn expected_dismount_response() -> Vec<u8> {
    let mut bytes = vec![0xa8, 0x00, 0x04];
    bytes.extend_from_slice(&OWNER.to_le_bytes());
    bytes.extend_from_slice(&0_i32.to_le_bytes()); // (R) unresolved result
    bytes
}

#[test]
fn client_table_and_mount_data_load_with_provenance() {
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/mounts.json");
    let catalog = MountCatalog::load(data).unwrap();
    assert_eq!(catalog.mount_types().count(), 30); // 29 types plus empty id 0.
    assert_eq!(catalog.mount_types().filter(|row| row.id > 0).count(), 29);
    assert_eq!(catalog.mounts().count(), 89); // T452: 54 seeded plus 35 OSFR-parity rides.
    assert!(catalog.mounts().all(|mount| !mount.source.is_empty()));
    assert!(
        catalog
            .mounts()
            .all(|mount| !mount.item_definition_id_candidates.is_empty())
    );
}

#[test]
fn list_is_derived_from_owned_inventory_items() {
    let mut service = MountService::new(catalog());
    let owned = inventory(&[ITEM_DEFINITION]);
    assert_eq!(service.catalog().owned_mounts(&owned).len(), 1);
    assert_eq!(service.list_packet(&owned), expected_mount_list(0));
    assert_eq!(
        service.list_packet(&inventory(&[])),
        [0xa8, 0, 5, 0, 0, 0, 0]
    );
}

#[test]
fn spawn_mount_dismount_then_despawn_have_expected_bytes_and_fanout() {
    let owned = inventory(&[ITEM_DEFINITION]);
    let mut service = MountService::new(catalog());
    let spawned = service
        .spawn(
            OWNER,
            &owned,
            MOUNT_ID,
            MOUNT_ACTOR,
            restrictions(),
            0,
            &[OBSERVER],
        )
        .unwrap();
    assert!(matches!(
        &spawned.effects[0].kind,
        MountEffectKind::Packet(bytes) if bytes == &expected_mount_list(MOUNT_ACTOR)
    ));
    assert!(matches!(
        &spawned.effects[1].kind,
        MountEffectKind::SpawnRide {
            owner_guid: OWNER,
            mount_id: MOUNT_ID,
            actor_guid: MOUNT_ACTOR,
            model_id: Some(33),
            max_movement_speed: 12.5,
        }
    ));
    assert_eq!(
        packet_effects(&spawned),
        vec![(vec![OWNER], expected_mount_list(MOUNT_ACTOR))]
    );

    let mounted = service
        .mount(
            OWNER,
            OWNER,
            &owned,
            MOUNT_ACTOR,
            0,
            restrictions(),
            &[OBSERVER],
        )
        .unwrap();
    assert_eq!(
        packet_effects(&mounted),
        vec![(vec![OWNER, OBSERVER], expected_mount_response())]
    );

    // One 168/3 from a rider dismounts and removes the ride actor (R).
    let dismounted = service.dismount_request(OWNER, &owned, &[OBSERVER]);
    assert_eq!(
        packet_effects(&dismounted),
        vec![
            (vec![OWNER, OBSERVER], expected_dismount_response()),
            (vec![OWNER], expected_mount_list(0)),
        ]
    );
    assert!(matches!(
        &dismounted.effects[1].kind,
        MountEffectKind::DespawnRide {
            actor_guid: MOUNT_ACTOR
        }
    ));
    assert_eq!(dismounted.effects[1].recipients, vec![OWNER, OBSERVER]);
    assert_eq!(service.state(), &MountState::Empty);

    // A repeated request is an idempotent no-op.
    let again = service.dismount_request(OWNER, &owned, &[OBSERVER]);
    assert!(again.effects.is_empty() && !again.changed);
}

#[test]
fn repeated_spawn_is_debounced_for_one_second() {
    let owned = inventory(&[ITEM_DEFINITION]);
    let mut service = MountService::new(catalog());
    service
        .spawn(
            OWNER,
            &owned,
            MOUNT_ID,
            MOUNT_ACTOR,
            restrictions(),
            5_000,
            &[],
        )
        .unwrap();
    let blocked = service
        .spawn(
            OWNER,
            &owned,
            MOUNT_ID,
            MOUNT_ACTOR + 1,
            restrictions(),
            5_999,
            &[],
        )
        .unwrap();
    assert!(blocked.debounced);
    assert!(blocked.effects.is_empty());
    assert!(matches!(
        service.state(),
        MountState::Spawned(ActiveRide {
            actor_guid: MOUNT_ACTOR,
            ..
        })
    ));

    let allowed = service
        .spawn(
            OWNER,
            &owned,
            MOUNT_ID,
            MOUNT_ACTOR + 1,
            restrictions(),
            6_000,
            &[],
        )
        .unwrap();
    assert!(!allowed.debounced);
    assert!(matches!(
        service.state(),
        MountState::Spawned(ActiveRide { actor_guid, .. }) if actor_guid == &(MOUNT_ACTOR + 1)
    ));
}

#[test]
fn unowned_mount_is_rejected_without_mutating_state() {
    let mut service = MountService::new(catalog());
    assert_eq!(
        service.spawn(
            OWNER,
            &inventory(&[ITEM_DEFINITION + 1]),
            MOUNT_ID,
            MOUNT_ACTOR,
            restrictions(),
            0,
            &[]
        ),
        Err(MountError::NotOwned)
    );
    assert_eq!(service.state(), &MountState::Empty);
}

#[test]
fn mount_request_rechecks_inventory_ownership() {
    let owned = inventory(&[ITEM_DEFINITION]);
    let mut service = MountService::new(catalog());
    service
        .spawn(OWNER, &owned, MOUNT_ID, MOUNT_ACTOR, restrictions(), 0, &[])
        .unwrap();
    assert_eq!(
        service.mount(
            OWNER,
            OWNER,
            &inventory(&[]),
            MOUNT_ACTOR,
            0,
            restrictions(),
            &[]
        ),
        Err(MountError::NotOwned)
    );
    assert!(matches!(service.state(), MountState::Spawned(_)));
}

#[test]
fn item_definition_spawn_uses_the_catalog_crosswalk() {
    let owned = inventory(&[ITEM_DEFINITION]);
    let mut service = MountService::new(catalog());
    let result = service
        .spawn_by_item_definition(
            OWNER,
            &owned,
            ITEM_DEFINITION,
            MOUNT_ACTOR,
            restrictions(),
            0,
            &[],
        )
        .unwrap();
    assert!(result.changed);
    assert!(matches!(
        service.state(),
        MountState::Spawned(ActiveRide {
            mount_id: MOUNT_ID,
            actor_guid: MOUNT_ACTOR,
            ..
        })
    ));
    assert_eq!(
        service.spawn_by_item_definition(
            OWNER,
            &owned,
            ITEM_DEFINITION + 1,
            MOUNT_ACTOR + 1,
            restrictions(),
            2_000,
            &[],
        ),
        Err(MountError::UnknownMountItem)
    );
}

#[test]
fn minigame_and_ability_start_remove_the_ride() {
    let owned = inventory(&[ITEM_DEFINITION]);
    let mut service = MountService::new(catalog());
    for (minigame, now_ms) in [(true, 0), (false, 5_000)] {
        service
            .spawn(
                OWNER,
                &owned,
                MOUNT_ID,
                MOUNT_ACTOR,
                restrictions(),
                now_ms,
                &[],
            )
            .unwrap();
        service
            .mount(OWNER, OWNER, &owned, MOUNT_ACTOR, 0, restrictions(), &[])
            .unwrap();
        assert!(service.is_mounted());
        let outcome = if minigame {
            service.on_minigame_start(OWNER, &owned, &[])
        } else {
            service.on_ability_use(OWNER, &owned, &[])
        };
        assert_eq!(
            packet_effects(&outcome)[0],
            (vec![OWNER], expected_dismount_response())
        );
        assert!(matches!(
            outcome.effects[1].kind,
            MountEffectKind::DespawnRide {
                actor_guid: MOUNT_ACTOR
            }
        ));
        assert_eq!(service.state(), &MountState::Empty);
        assert!(!service.is_mounted());
    }
}

#[test]
fn restrictions_reject_spawn_in_minigame_ability_or_blocked_zone() {
    let mut service = MountService::new(catalog());
    let owned = inventory(&[ITEM_DEFINITION]);
    for context in [
        RideRestrictions {
            in_minigame: true,
            ..restrictions()
        },
        RideRestrictions {
            ability_active: true,
            ..restrictions()
        },
        RideRestrictions {
            zone_allows_riding: false,
            ..restrictions()
        },
        RideRestrictions {
            knocked_out: true,
            ..restrictions()
        },
    ] {
        assert_eq!(
            service.spawn(OWNER, &owned, MOUNT_ID, MOUNT_ACTOR, context, 0, &[]),
            Err(MountError::Restricted)
        );
    }
}

// ---------------------------------------------------------------- T482: one row per owned ride

/// Lava Tiger (ride 12688) has two items in the captured reference catalogue with different looks:
/// 17240 (alias `blizzard`) and 76906 (alias `dyetint`).
const LAVA_TIGER: i32 = 12688;
const BLIZZARD_ITEM: i32 = 17240;
const DYETINT_ITEM: i32 = 76906;

/// The real ride catalogue with the item looks of the reference item catalogue.
fn lava_service() -> MountService {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let reference = manifest.join("../../../lab/references/rust-opening-cooking-original-assets");
    let items = Inventory::load(&reference, None).unwrap();
    MountService::load(&manifest.join("data/content/mounts.json"), &items).unwrap()
}

/// One 168/5 record as the client reads it: (row id, guid, tint, alias).
fn rows(list: &[u8]) -> Vec<(i32, u64, i32, String)> {
    let word = |at: usize| i32::from_le_bytes(list[at..at + 4].try_into().unwrap());
    let mut at = 7;
    let mut out = Vec::new();
    for _ in 0..word(3) {
        let guid = u64::from_le_bytes(list[at + 12..at + 20].try_into().unwrap());
        let alias_len = word(at + 25) as usize;
        let alias = String::from_utf8(list[at + 29..at + 29 + alias_len].to_vec()).unwrap();
        out.push((word(at), guid, word(at + 21), alias));
        at += 29 + alias_len + 2;
    }
    assert_eq!(at, list.len(), "no trailing bytes");
    out
}

fn request(row: i32) -> Vec<u8> {
    [vec![168, 0, 6], row.to_le_bytes().to_vec()].concat()
}

fn context(inventory: &InventoryState, now_ms: u64) -> RideContext<'_> {
    RideContext {
        guid: OWNER,
        inventory,
        restrictions: restrictions(),
        pose: RidePose {
            position: [0.0; 3],
            heading: 0.0,
        },
        now_ms,
    }
}

#[test]
fn each_owned_ride_item_is_its_own_row() {
    let mut service = lava_service();
    let mut owned = inventory(&[BLIZZARD_ITEM, DYETINT_ITEM, 1, BLIZZARD_ITEM]);
    owned.items[3].count = 0; // an emptied instance is not a row; the non-ride item 1 is not either
    assert_eq!(
        rows(&service.list_packet(&owned)),
        [
            (1, 0, 0, "blizzard".to_owned()),
            (2, 0, 0, "dyetint".to_owned())
        ]
    );
    // A dyed instance lists its own tint; the other row keeps 0.
    owned.items[1].tint = 249;
    assert_eq!(
        rows(&service.list_packet(&owned))[1],
        (2, 0, 249, "dyetint".to_owned())
    );
    // Duplicates of one item are separate rows too, ordered by instance id.
    let twins = inventory(&[BLIZZARD_ITEM, BLIZZARD_ITEM]);
    let list = service.list_packet(&twins);
    let ids: Vec<i32> = rows(&list).iter().map(|row| row.0).collect();
    assert_eq!(ids, [1, 2]);
    // The ride's name and image set still come from the ride definition.
    let ride = service.catalog().mount(LAVA_TIGER).unwrap();
    assert_eq!(&list[11..15], &ride.name_string_id.to_le_bytes());
    assert_eq!(&list[15..19], &ride.image_set_id.to_le_bytes());
}

#[test]
fn a_changed_tint_resends_the_roster() {
    let mut service = lava_service();
    let mut owned = inventory(&[BLIZZARD_ITEM, DYETINT_ITEM]);
    assert_eq!(service.roster_sync(&owned), None, "before world entry");
    service.list_packet(&owned);
    assert_eq!(service.roster_sync(&owned), None, "nothing changed");
    owned.items[0].tint = 249; // the ride set is the same; only a tint moved
    let resent = service
        .roster_sync(&owned)
        .expect("a dye resends the roster");
    assert_eq!(rows(&resent)[0].2, 249);
    assert_eq!(service.roster_sync(&owned), None, "sent once");
}

#[test]
fn spawning_a_row_summons_that_instance() {
    let mut service = lava_service();
    let owned = inventory(&[BLIZZARD_ITEM, DYETINT_ITEM]);
    let wire = service
        .handle(&request(2), &context(&owned, 0))
        .unwrap()
        .unwrap();
    assert_eq!(wire.notice, None);
    assert!(matches!(
        service.state(),
        MountState::Spawned(ActiveRide {
            mount_id: LAVA_TIGER,
            instance_id: 2,
            ..
        })
    ));
    let actor = MountService::ride_guid(OWNER);
    let sent = rows(&wire.private[0]);
    assert_eq!(
        sent.iter().map(|row| (row.0, row.1)).collect::<Vec<_>>(),
        [(1, 0), (2, actor)],
        "only the summoned row carries the guid"
    );
    // Boarding rechecks that instance: its sibling does not stand in for it.
    let mut only_sibling = inventory(&[]);
    only_sibling.items.push(owned.items[0].clone());
    assert_eq!(
        service.mount(OWNER, OWNER, &only_sibling, actor, 0, restrictions(), &[]),
        Err(MountError::NotOwned)
    );
    assert!(
        service
            .mount(OWNER, OWNER, &owned, actor, 0, restrictions(), &[])
            .is_ok()
    );
}

#[test]
fn a_stale_row_is_refused_without_change() {
    let mut owned = inventory(&[BLIZZARD_ITEM, DYETINT_ITEM, 1]);
    owned.items[1].count = 0;
    let mut service = lava_service();
    service
        .handle(&request(1), &context(&owned, 0))
        .unwrap()
        .unwrap();
    let before = service.state().clone();
    // A zero-count instance, an unknown id, zero, a negative id, a non-ride item and a ride id.
    for row in [2, 99, 0, -1, 3, LAVA_TIGER] {
        let wire = service
            .handle(&request(row), &context(&owned, 5_000))
            .unwrap()
            .unwrap();
        assert!(wire.notice.is_some(), "row {row}");
        assert!(wire.shared.is_empty(), "row {row}");
        // The refresh is the roster as it stands: the one owned row, still the active ride.
        let roster = rows(&wire.private[0]);
        assert_eq!(roster.len(), 1, "row {row}");
        assert_eq!(roster[0].1, MountService::ride_guid(OWNER), "row {row}");
        assert_eq!(service.state(), &before, "row {row}");
    }
}

#[test]
fn spawn_by_item_picks_the_lowest_owned_instance() {
    let mut service = lava_service();
    let mut owned = inventory(&[BLIZZARD_ITEM, DYETINT_ITEM, BLIZZARD_ITEM]);
    owned.items.reverse(); // listed as instance 3, 2, 1: the lowest id is not the first listed
    let item = [
        vec![168, 0, 8],
        BLIZZARD_ITEM.to_le_bytes().to_vec(),
        vec![1],
    ]
    .concat();
    service.handle(&item, &context(&owned, 0)).unwrap().unwrap();
    assert!(matches!(
        service.state(),
        MountState::Spawned(ActiveRide { instance_id: 1, .. })
    ));
    // An item that is not owned is refused and changes nothing.
    let mut service = lava_service();
    let other = inventory(&[DYETINT_ITEM]);
    let wire = service.handle(&item, &context(&other, 0)).unwrap().unwrap();
    assert!(wire.notice.is_some());
    assert_eq!(service.state(), &MountState::Empty);
}
