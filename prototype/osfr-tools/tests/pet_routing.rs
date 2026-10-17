use osfr_tools::{
    character_store::CharacterState,
    game_store,
    inventory::{Inventory, InventoryItem, InventoryState},
    pets::{PetBusy, PetDefinition, PetRequest, PetService},
    startup_world,
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

const OWNER: u64 = 77;
const NOW: u64 = 10_000;

fn definition() -> PetDefinition {
    PetDefinition {
        species_id: 652,
        model: "wolf_generic.adr".into(),
        adoption_item_definition_id: Some(9001),
        initial_tricks: vec![1],
    }
}

fn service() -> PetService {
    PetService::new(PetService::bundled_content().unwrap()).unwrap()
}

fn reference() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-opening-cooking-original-assets")
}

fn inventory_handle(item_definition: i32) -> Inventory {
    let mut inventory = Inventory::load(&reference(), None).unwrap();
    inventory.grant(0, item_definition, 1).unwrap();
    inventory
}

fn owned_pet(service: &mut PetService) -> u32 {
    service.adopt_vendor_grant(1, &definition(), 0).unwrap()
}

fn inventory(item_definition: i32) -> InventoryState {
    InventoryState {
        coins: 0,
        items: vec![InventoryItem {
            definition_id: item_definition,
            tint: 0,
            id: 10,
            count: 1,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: i64::from(i32::MAX),
            rental: None,
            ability_count: 0,
            activate_enabled: false,
        }],
        equipment: Vec::new(),
        next_instance_id: None,
    }
}

fn summon(pet_id: u32) -> Vec<u8> {
    [vec![53, 0, 4], pet_id.to_le_bytes().to_vec(), vec![1]].concat()
}

fn recall(pet_id: u32) -> Vec<u8> {
    [vec![53, 0, 4], pet_id.to_le_bytes().to_vec(), vec![0]].concat()
}

fn name_request(name: &str) -> Vec<u8> {
    [
        vec![53, 0, 6],
        (name.len() as u32).to_le_bytes().to_vec(),
        name.as_bytes().to_vec(),
    ]
    .concat()
}

fn summon_by_item(pet_id: u32, item_instance_id: u32) -> Vec<u8> {
    [
        vec![53, 0, 32],
        pet_id.to_le_bytes().to_vec(),
        item_instance_id.to_le_bytes().to_vec(),
    ]
    .concat()
}

fn has(packets: &[Vec<u8>], family: u16, subtype: u8) -> bool {
    packets.iter().any(|packet| {
        packet.get(..2) == Some(&family.to_le_bytes()) && packet.get(2) == Some(&subtype)
    })
}

fn actor_packet(packets: &[Vec<u8>], actor: u64) -> bool {
    packets
        .iter()
        .any(|packet| packet.starts_with(&[35, 0, 2, 0]) && packet[4..12] == actor.to_le_bytes())
}

fn removal_packet(packets: &[Vec<u8>], actor: u64) -> bool {
    packets.iter().any(|packet| {
        packet.starts_with(&[35, 0, 3, 0, 1, 0]) && packet[6..14] == actor.to_le_bytes()
    })
}

fn name_response(packets: &[Vec<u8>], actor: u64, accepted: bool) -> bool {
    packets.iter().any(|packet| {
        packet.starts_with(&[53, 0, 10])
            && packet.get(3..11) == Some(&actor.to_le_bytes()[..])
            && packet.last() == Some(&u8::from(accepted))
    })
}

#[test]
fn an_owned_pet_summons_one_server_actor_and_the_short_dismiss_removes_it() {
    let mut service = service();
    let pet_id = owned_pet(&mut service);
    let actor = PetService::pet_guid(OWNER);
    assert_ne!(actor, OWNER);
    assert_eq!(
        PetRequest::decode(&summon(pet_id)).unwrap(),
        Some(PetRequest::Summon { pet_id })
    );
    assert_eq!(
        PetRequest::decode(&recall(pet_id)).unwrap(),
        Some(PetRequest::Recall { pet_id })
    );

    let summon_wire = service
        .handle(
            &summon(pet_id),
            OWNER,
            [1.0, 2.0, 3.0],
            &inventory(9001),
            PetBusy::default(),
            NOW,
        )
        .unwrap()
        .unwrap();
    assert!(has(&summon_wire.private, 53, 9));
    assert_eq!(
        summon_wire
            .shared
            .iter()
            .filter(|packet| packet.starts_with(&[35, 0, 2, 0]))
            .count(),
        1
    );
    assert!(actor_packet(&summon_wire.shared, actor));
    assert_eq!(summon_wire.private[1..], summon_wire.shared);
    assert_eq!(service.active_pet_id(), Some(pet_id));
    assert_eq!(service.view(OWNER).unwrap().actor_guid, actor);

    let dismiss_wire = service
        .handle(
            &[53, 0, 4, 0],
            OWNER,
            [1.0, 2.0, 3.0],
            &inventory(9001),
            PetBusy::default(),
            NOW + 1,
        )
        .unwrap()
        .unwrap();
    assert!(has(&dismiss_wire.private, 53, 9));
    assert!(removal_packet(&dismiss_wire.shared, actor));
    assert_eq!(service.active_pet_id(), None);
    assert!(service.view(OWNER).is_none());
}

#[test]
fn a_bundled_pet_actor_carries_its_models_txt_id_and_texture_alias() {
    let mut service = service();
    let definition = PetService::bundled_content()
        .unwrap()
        .definition(1_000_015)
        .unwrap();
    let pet_id = service.adopt_vendor_grant(1, &definition, 0).unwrap();
    service
        .handle(
            &summon(pet_id),
            OWNER,
            [1.0, 2.0, 3.0],
            &inventory(70352),
            PetBusy::default(),
            NOW,
        )
        .unwrap();

    let packet = service.view(OWNER).unwrap().spawn_packet().unwrap();
    let npc = startup_world::parse_add_npc(&packet).unwrap();
    assert_eq!(npc.actor_id, PetService::pet_guid(OWNER));
    assert_eq!(npc.model_id, 1756);
    assert_eq!(npc.texture_alias, "goldtips");
    assert_eq!(npc.npc_name, "Char");
    assert_eq!(npc.rotation, [0.0, 0.0, 1.0, 0.0]);
    assert_eq!(npc.speed, 5.0);
}

#[test]
fn unowned_pet_item_and_malformed_refusals_each_change_nothing() {
    let mut service = service();
    let pet_id = owned_pet(&mut service);
    let before_roster = service.roster_snapshot();
    assert!(before_roster.is_some());

    let unowned = service
        .handle(
            &summon(pet_id + 100),
            OWNER,
            [1.0, 2.0, 3.0],
            &inventory(9001),
            PetBusy::default(),
            NOW,
        )
        .unwrap()
        .unwrap();
    assert!(unowned.notice.is_some());
    assert!(has(&unowned.private, 53, 5));
    assert!(unowned.shared.is_empty());
    assert_eq!(service.roster_snapshot(), before_roster);
    assert_eq!(service.active_pet_id(), None);

    let wrong_item = service
        .handle(
            &summon_by_item(pet_id, 10),
            OWNER,
            [1.0, 2.0, 3.0],
            &inventory(9002),
            PetBusy::default(),
            NOW + 1,
        )
        .unwrap()
        .unwrap();
    assert!(wrong_item.notice.is_some());
    assert!(wrong_item.shared.is_empty());
    assert_eq!(service.roster_snapshot(), before_roster);
    assert_eq!(service.active_pet_id(), None);

    let malformed = [vec![53, 0, 4], pet_id.to_le_bytes().to_vec()].concat();
    assert!(
        service
            .handle(
                &malformed,
                OWNER,
                [1.0, 2.0, 3.0],
                &inventory(9001),
                PetBusy::default(),
                NOW + 2,
            )
            .is_err()
    );
    assert_eq!(service.roster_snapshot(), before_roster);
    assert_eq!(service.active_pet_id(), None);
}

#[test]
fn an_owned_adoption_item_can_summon_through_the_item_form() {
    let mut service = service();
    let pet_id = owned_pet(&mut service);
    let wire = service
        .handle(
            &summon_by_item(pet_id, 10),
            OWNER,
            [0.0, 1.0, 2.0],
            &inventory(9001),
            PetBusy::default(),
            NOW,
        )
        .unwrap()
        .unwrap();
    assert!(has(&wire.private, 53, 9));
    assert_eq!(service.active_pet_id(), Some(pet_id));
}

#[test]
fn an_owned_adoption_item_can_create_and_summon_a_roster_pet() {
    let mut service = service();
    let mut inventory = inventory_handle(70352);
    let before = inventory
        .state()
        .items
        .iter()
        .find(|item| item.definition_id == 70352)
        .map_or(0, |item| item.count);
    let request = summon_by_item(1, 70352);
    let wire = service
        .handle_with_inventory(
            &request,
            OWNER,
            [0.0, 1.0, 2.0],
            &mut inventory,
            PetBusy::default(),
            NOW,
        )
        .unwrap()
        .unwrap();
    assert!(wire.changed);
    assert!(wire.pets_changed);
    assert!(has(&wire.private, 53, 5));
    assert!(has(&wire.private, 53, 9));
    assert_eq!(service.active_pet_id(), Some(1));
    assert!(service.pet(1).is_some());
    let after = inventory
        .state()
        .items
        .iter()
        .find(|item| item.definition_id == 70352)
        .map_or(0, |item| item.count);
    assert!(after < before);
}

#[test]
fn ui_mode_is_bounded_and_a_reconnect_keeps_the_roster_but_not_the_actor() {
    let mut service = service();
    let pet_id = owned_pet(&mut service);
    let before = service.roster_snapshot().unwrap();
    assert_eq!(
        PetRequest::decode(&[53, 0, 24, 2, 0, 0, 0]).unwrap(),
        Some(PetRequest::UiMode(2))
    );
    service
        .handle(
            &[53, 0, 24, 2, 0, 0, 0],
            OWNER,
            [0.0; 3],
            &inventory(9001),
            PetBusy::default(),
            NOW,
        )
        .unwrap();
    assert_eq!(service.ui_mode(), 2);
    assert!(
        service
            .handle(
                &[53, 0, 24, 3, 0, 0, 0],
                OWNER,
                [0.0; 3],
                &inventory(9001),
                PetBusy::default(),
                NOW,
            )
            .is_err()
    );
    assert_eq!(service.ui_mode(), 2);
    assert_eq!(service.roster_snapshot(), Some(before.clone()));

    service
        .handle(
            &summon(pet_id),
            OWNER,
            [4.0, 5.0, 6.0],
            &inventory(9001),
            PetBusy::default(),
            NOW,
        )
        .unwrap();
    let reloaded =
        PetService::from_saved(PetService::bundled_content().unwrap(), Some(before)).unwrap();
    assert_eq!(reloaded.pet(pet_id).unwrap().id, pet_id);
    assert_eq!(reloaded.active_pet_id(), None);
}

#[test]
fn a_selected_name_updates_only_the_active_pet_and_refusals_change_nothing() {
    let mut service = service();
    let pet_id = owned_pet(&mut service);
    let before_name = service.pet(pet_id).unwrap().name.clone();

    let inactive = service
        .handle(
            &name_request("Admiral Ace"),
            OWNER,
            [0.0; 3],
            &inventory(9001),
            PetBusy::default(),
            NOW,
        )
        .unwrap()
        .unwrap();
    assert!(!inactive.changed);
    assert!(name_response(&inactive.private, 0, false));
    assert_eq!(service.pet(pet_id).unwrap().name, before_name);

    service
        .handle(
            &summon(pet_id),
            OWNER,
            [0.0; 3],
            &inventory(9001),
            PetBusy::default(),
            NOW + 1,
        )
        .unwrap();
    let accepted = service
        .handle(
            &name_request("Admiral Ace"),
            OWNER,
            [0.0; 3],
            &inventory(9001),
            PetBusy::default(),
            NOW + 2,
        )
        .unwrap()
        .unwrap();
    assert!(accepted.changed);
    assert!(name_response(
        &accepted.private,
        PetService::pet_guid(OWNER),
        true
    ));
    assert!(has(&accepted.private, 53, 5));
    assert_eq!(service.pet(pet_id).unwrap().name, "Admiral Ace");

    let before = service.roster_snapshot();
    let rejected = service
        .handle(
            &name_request("not a selected name"),
            OWNER,
            [0.0; 3],
            &inventory(9001),
            PetBusy::default(),
            NOW + 3,
        )
        .unwrap()
        .unwrap();
    assert!(!rejected.changed);
    assert!(name_response(
        &rejected.private,
        PetService::pet_guid(OWNER),
        false
    ));
    assert_eq!(service.roster_snapshot(), before);
    assert_eq!(service.pet(pet_id).unwrap().name, "Admiral Ace");
}

#[test]
fn the_roster_refuses_pet_fifty_one_without_mutation() {
    let mut service = service();
    for grant in 1..=50 {
        service.adopt_vendor_grant(grant, &definition(), 0).unwrap();
    }
    let before = service.roster_snapshot();
    assert!(service.adopt_vendor_grant(51, &definition(), 0).is_err());
    assert_eq!(service.roster_snapshot(), before);
}

struct SaveFile {
    directory: PathBuf,
    path: PathBuf,
}

impl SaveFile {
    fn new() -> Self {
        let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "osfr-pet-routing-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&directory).unwrap();
        Self {
            path: directory.join("save.json"),
            directory,
        }
    }
}

impl Drop for SaveFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
        let _ = fs::remove_dir(&self.directory);
    }
}

#[test]
fn roster_reload_preserves_ownership_and_the_client_list_reports_no_active_actor() {
    let file = SaveFile::new();
    game_store::save(
        &file.path,
        &game_store::Snapshot {
            character: CharacterState {
                guid: OWNER,
                zone_id: 0,
                position: [0.0; 3],
                rotation: [0.0, 0.0, 1.0],
            },
            gameplay: None,
        },
    )
    .unwrap();
    let mut service = service();
    let pet_id = owned_pet(&mut service);
    service.save_to(&file.path).unwrap();
    service
        .handle(
            &summon(pet_id),
            OWNER,
            [7.0, 8.0, 9.0],
            &inventory(9001),
            PetBusy::default(),
            NOW,
        )
        .unwrap();

    let reloaded =
        PetService::load_from(PetService::bundled_content().unwrap(), &file.path).unwrap();
    assert_eq!(reloaded.pet(pet_id).unwrap().id, pet_id);
    assert_eq!(reloaded.active_pet_id(), None);
    let list = reloaded.list_packet();
    assert_eq!(list[..3], [53, 0, 5]);
    assert_eq!(
        u32::from_le_bytes(list[3..7].try_into().unwrap()),
        1,
        "one owned pet"
    );
    assert_eq!(
        u32::from_le_bytes(list[7..11].try_into().unwrap()),
        pet_id,
        "list key"
    );
    assert_eq!(
        u32::from_le_bytes(list[11..15].try_into().unwrap()),
        pet_id,
        "record id"
    );
    assert_eq!(&list[list.len() - 12..list.len() - 8], &[0; 4]);
    assert_eq!(&list[list.len() - 8..], &[0; 8]);
}
