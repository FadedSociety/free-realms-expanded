use osfr_tools::{inventory, mounts};

use inventory::{InventoryItem, InventoryState};
use mounts::{
    ActiveRide, MountCatalog, MountDefinition, MountEffectKind, MountError, MountService,
    MountSource, MountState, MountType, RideRestrictions,
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
            source: source(),
            field_sources: Default::default(),
        }],
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
    bytes.extend_from_slice(&MOUNT_ID.to_le_bytes());
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
    bytes.extend_from_slice(&1_i32.to_le_bytes()); // (R) accept
    bytes.extend_from_slice(&0_i32.to_le_bytes()); // (R) queue position
    bytes.extend_from_slice(&0_i32.to_le_bytes()); // (R) unknown field
    bytes.extend_from_slice(&0.0_f32.to_le_bytes()); // (R) unknown field
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
    assert_eq!(catalog.mounts().count(), 54);
    assert!(catalog.mounts().all(|mount| !mount.source.is_empty()));
    assert!(
        catalog
            .mounts()
            .all(|mount| !mount.item_definition_id_candidates.is_empty())
    );
}

#[test]
fn list_is_derived_from_owned_inventory_items() {
    let service = MountService::new(catalog());
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
            actor_guid: MOUNT_ACTOR
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
