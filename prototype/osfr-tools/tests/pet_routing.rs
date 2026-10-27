use osfr_tools::{
    character_store::CharacterState,
    game_store,
    inventory::Inventory,
    pets::{PetBusy, PetContent, PetDefinition, PetRequest, PetService},
    startup_world,
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

#[path = "../src/bin/osfr_gateway/pet_wire.rs"]
mod pet_wire;
use pet_wire::{summon, summon_by_item};

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

fn owner() -> CharacterState {
    CharacterState {
        guid: OWNER,
        zone_id: 0,
        position: [0.0; 3],
        rotation: [0.0, 0.0, 1.0],
    }
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

/// Client `Pet.Dismiss` shape: writer 00b5a650 always emits a u32 id followed by
/// the action byte, and 00c07420 sets only the action (the id word is whatever
/// the stack held), so the id must never select a pet.
fn dismiss(stale_id: u32) -> Vec<u8> {
    [vec![53, 0, 4], stale_id.to_le_bytes().to_vec(), vec![0]].concat()
}

fn name_request(name: &str) -> Vec<u8> {
    [
        vec![53, 0, 6],
        (name.len() as u32).to_le_bytes().to_vec(),
        name.as_bytes().to_vec(),
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
fn an_owned_pet_summons_one_server_actor_and_the_serializer_dismiss_removes_it() {
    let mut service = service();
    let pet_id = owned_pet(&mut service);
    let actor = PetService::pet_guid(OWNER);
    assert_ne!(actor, OWNER);
    assert_eq!(
        PetRequest::decode(&summon(pet_id)).unwrap(),
        Some(PetRequest::Summon { pet_id })
    );
    // The native dismiss sender leaves the id word uninitialized, so any four
    // bytes followed by action 0 mean "dismiss the active pet".
    assert_eq!(
        PetRequest::decode(&dismiss(0)).unwrap(),
        Some(PetRequest::Dismiss)
    );
    assert_eq!(
        PetRequest::decode(&dismiss(0xdead_beef)).unwrap(),
        Some(PetRequest::Dismiss)
    );
    assert!(
        PetRequest::decode(&[53, 0, 4, 0]).is_err(),
        "a one-byte body is not the client's serializer shape"
    );

    let summon_wire = service
        .handle(
            &summon(pet_id),
            OWNER,
            [1.0, 2.0, 3.0],
            PetBusy::default(),
            NOW,
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        summon_wire
            .shared
            .iter()
            .filter(|packet| packet.starts_with(&[35, 0, 2, 0]))
            .count(),
        1
    );
    assert!(actor_packet(&summon_wire.shared, actor));
    // Actor creation precedes activation: the client resolves the actor as soon
    // as PetActive arrives.
    // T456: the owner's replies end with 53/9 then the roster (53/5).
    assert_eq!(
        summon_wire.private[..summon_wire.private.len() - 2],
        summon_wire.shared
    );
    let active = &summon_wire.private[summon_wire.private.len() - 2]; // then the roster (T456)
    assert_eq!(&summon_wire.private.last().unwrap()[..3], [53, 0, 5]);
    assert_eq!(&active[..3], [53, 0, 9]);
    assert_eq!(&active[3..7], &pet_id.to_le_bytes());
    assert_eq!(&active[7..15], &actor.to_le_bytes());
    assert_eq!(service.active_pet_id(), Some(pet_id));
    assert_eq!(service.view(OWNER).unwrap().actor_guid, actor);

    let dismiss_wire = service
        .handle(
            &dismiss(0),
            OWNER,
            [1.0, 2.0, 3.0],
            PetBusy::default(),
            NOW + 1,
        )
        .unwrap()
        .unwrap();
    assert!(has(&dismiss_wire.private, 53, 9));
    assert!(removal_packet(&dismiss_wire.shared, actor));
    assert_eq!(
        dismiss_wire.private[..dismiss_wire.private.len() - 1],
        dismiss_wire.shared
    );
    assert_eq!(service.active_pet_id(), None);
    assert!(service.view(OWNER).is_none());

    // A stale non-zero id from the same sender still dismisses.
    service
        .handle(
            &summon(pet_id),
            OWNER,
            [1.0, 2.0, 3.0],
            PetBusy::default(),
            NOW + 2,
        )
        .unwrap()
        .unwrap();
    let stale = service
        .handle(
            &dismiss(0x7fff_ffff),
            OWNER,
            [1.0, 2.0, 3.0],
            PetBusy::default(),
            NOW + 3,
        )
        .unwrap()
        .unwrap();
    assert!(removal_packet(&stale.shared, actor));
    assert_eq!(service.active_pet_id(), None);
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
            &summon_by_item(9001, 0),
            OWNER,
            [1.0, 2.0, 3.0],
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
                PetBusy::default(),
                NOW + 2,
            )
            .is_err()
    );
    assert_eq!(service.roster_snapshot(), before_roster);
    assert_eq!(service.active_pet_id(), None);
}

/// Lead (2026-10-18): each adoption item is redeemed once into its own pet, even when the
/// species is already owned; a definition match must not re-summon the old pet and strand
/// the certificate. With none left, the request is refused and changes nothing.
#[test]
fn each_adoption_item_adopts_a_new_pet_even_when_the_species_is_owned() {
    let mut service = service();
    let mut inventory = inventory_handle(70352);
    inventory.grant(0, 70352, 1).unwrap();
    let certificates = |inventory: &Inventory| -> u32 {
        inventory
            .state()
            .items
            .iter()
            .filter(|item| item.definition_id == 70352)
            .map(|item| item.count)
            .sum()
    };
    assert_eq!(certificates(&inventory), 2);
    for pet in [1u32, 2] {
        let wire = service
            .handle_at_seconds(
                &summon_by_item(70352, 0),
                OWNER,
                [0.0, 1.0, 2.0],
                &mut inventory,
                PetBusy::default(),
                NOW + u64::from(pet),
            )
            .unwrap()
            .unwrap();
        assert!(wire.changed);
        assert_eq!(service.active_pet_id(), Some(pet));
        assert_eq!(certificates(&inventory), 2 - pet);
    }
    assert!(service.pet(1).is_some() && service.pet(2).is_some());

    let roster = service.roster_snapshot();
    let refused = service
        .handle_at_seconds(
            &summon_by_item(70352, 0),
            OWNER,
            [0.0, 1.0, 2.0],
            &mut inventory,
            PetBusy::default(),
            NOW + 3,
        )
        .unwrap()
        .unwrap();
    assert!(refused.notice.is_some());
    assert!(!refused.changed);
    assert_eq!(service.roster_snapshot(), roster);
    assert_eq!(service.active_pet_id(), Some(2));
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
    let request = summon_by_item(70352, 0);
    let wire = service
        .handle_at_seconds(
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
    // Actor creation precedes the roster (whose trailing active pair activates
    // the pet) and the explicit activation.
    let actor_at = wire
        .private
        .iter()
        .position(|packet| packet.starts_with(&[35, 0, 2, 0]))
        .expect("new pet actor");
    let list_at = wire
        .private
        .iter()
        .position(|packet| packet.starts_with(&[53, 0, 5]))
        .expect("roster after creation");
    let active_at = wire
        .private
        .iter()
        .position(|packet| packet.starts_with(&[53, 0, 9]))
        .expect("activation last");
    assert!(
        actor_at < list_at && list_at < active_at,
        "order was actor={actor_at} list={list_at} active={active_at}"
    );
    let (keys, pets, active, actor) = decode_pet_list(&wire.private[list_at]);
    assert_eq!(keys, vec![1]);
    assert_eq!(pets.len(), 1);
    assert_eq!(pets[0].id, 1);
    assert_eq!(active, 1);
    assert_eq!(actor, PetService::pet_guid(OWNER));
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
            PetBusy::default(),
            NOW + 1,
        )
        .unwrap();
    let accepted = service
        .handle(
            &name_request("Admiral Ace"),
            OWNER,
            [0.0; 3],
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
    let mut service = service();
    let pet_id = owned_pet(&mut service);
    game_store::save(
        &file.path,
        &game_store::Snapshot {
            character: owner(),
            gameplay: None,
            pets: service.document_value(),
        },
    )
    .unwrap();
    service
        .handle(
            &summon(pet_id),
            OWNER,
            [7.0, 8.0, 9.0],
            PetBusy::default(),
            NOW,
        )
        .unwrap();

    let saved = game_store::load_or_create(&file.path, owner()).unwrap();
    let reloaded =
        PetService::from_document(PetService::bundled_content().unwrap(), saved.pets).unwrap();
    assert_eq!(reloaded.pet(pet_id).unwrap().id, pet_id);
    assert_eq!(reloaded.active_pet_id(), None);
    let list = reloaded.list_packet();
    // Client `00b5aea0` reads one key per pet and `00912cf0` then reads one
    // record that starts with its own id. Decode the whole packet independently
    // and require exact consumption: a duplicate id shifts every later field.
    let (keys, pets, active, actor) = decode_pet_list(&list);
    assert_eq!(keys, vec![pet_id], "one list key per pet");
    assert_eq!(pets.len(), 1, "one owned pet");
    assert_eq!(pets[0].id, pet_id, "record id");
    assert_ne!(
        pets[0].icon, 652,
        "+0x08 is the model icon, not the species id (T456)"
    );
    assert_eq!(pets[0].name, "");
    assert!(
        pets[0].tricks.contains(&1),
        "a learned trick stays listed (T456)"
    );
    assert_eq!(active, 0);
    assert_eq!(actor, 0);
}

/// Independent client-shaped decoder for subtype 5. Mirrors the reader layout
/// (`00912cf0` fixed fields, counted trick/item rows, three strings, four
/// favourite dwords, eight utility dwords) and asserts the trailing active pair.
fn decode_pet_list(bytes: &[u8]) -> (Vec<u32>, Vec<DecodedPet>, u32, u64) {
    let mut at = ListCursor::new(bytes);
    assert_eq!(at.take(2), [53, 0], "pet family");
    assert_eq!(at.u8(), 5, "PetList subtype");
    let count = at.u32();
    let mut keys = Vec::new();
    let mut pets = Vec::new();
    for _ in 0..count {
        keys.push(at.u32());
        let mut pet = DecodedPet {
            id: at.u32(),
            icon: 0,
            name: String::new(),
            tricks: Vec::new(),
            items: Vec::new(),
            favorites: Vec::new(),
            utilities: Vec::new(),
        };
        at.u8(); // T456: +0x04 sex (the model's Models.txt GENDER (R))
        pet.icon = at.u32(); // +0x08 icon id
        for _ in 0..4 {
            assert!(at.f32().is_finite(), "need value");
        }
        at.u8(); // +0x30 members-only
        for _ in 0..at.u32() {
            let key = at.u32();
            for _ in 0..8 {
                at.u32();
            }
            if at.u8() == 1 {
                pet.tricks.push(key); // known tricks only
            }
        }
        for _ in 0..at.u32() {
            pet.items.push(at.u32());
        }
        for _ in 0..at.u32() {
            at.take(12); // T456: item-class rows (key, id, unknown)
        }
        pet.name = at.string();
        at.u32();
        at.u32();
        at.string(); // +0x108 tint alias
        at.u32();
        assert_eq!(at.u8(), 0);
        at.u32();
        assert_eq!(at.u8(), 0);
        for _ in 0..4 {
            pet.favorites.push(at.u32());
        }
        for _ in 0..8 {
            pet.utilities.push(at.u32());
        }
        pets.push(pet);
    }
    let active = at.u32();
    let actor = at.u64();
    assert_eq!(at.at, bytes.len(), "PetList must be fully consumed");
    (keys, pets, active, actor)
}

struct DecodedPet {
    id: u32,
    icon: u32,
    name: String,
    tricks: Vec<u32>,
    items: Vec<u32>,
    favorites: Vec<u32>,
    utilities: Vec<u32>,
}

struct ListCursor<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> ListCursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    fn take(&mut self, count: usize) -> &'a [u8] {
        let end = self.at.checked_add(count).expect("bounded list");
        let slice = &self.bytes[self.at..end];
        self.at = end;
        slice
    }

    fn u8(&mut self) -> u8 {
        self.take(1)[0]
    }

    fn u32(&mut self) -> u32 {
        u32::from_le_bytes(self.take(4).try_into().unwrap())
    }

    fn u64(&mut self) -> u64 {
        u64::from_le_bytes(self.take(8).try_into().unwrap())
    }

    fn f32(&mut self) -> f32 {
        f32::from_bits(self.u32())
    }

    fn string(&mut self) -> String {
        let length = self.u32() as usize;
        String::from_utf8(self.take(length).to_vec()).unwrap()
    }
}

#[test]
fn switching_pets_removes_the_old_actor_then_creates_and_activates_the_new_one() {
    let mut service = service();
    let first = service.adopt_vendor_grant(1, &definition(), 0).unwrap();
    let second = service.adopt_vendor_grant(2, &definition(), 0).unwrap();
    let actor = PetService::pet_guid(OWNER);
    service
        .handle(&summon(first), OWNER, [0.0; 3], PetBusy::default(), NOW)
        .unwrap()
        .unwrap();

    let switched = service
        .handle(
            &summon(second),
            OWNER,
            [0.0; 3],
            PetBusy::default(),
            NOW + 1,
        )
        .unwrap()
        .unwrap();
    assert_eq!(switched.shared.len(), 2);
    assert_eq!(switched.private[..2], switched.shared[..]);
    let removal = &switched.private[0];
    assert!(removal.starts_with(&[35, 0, 3, 0, 1, 0]));
    assert_eq!(&removal[6..14], &actor.to_le_bytes());
    let created = &switched.private[1];
    assert!(created.starts_with(&[35, 0, 2, 0]));
    assert_eq!(&created[4..12], &actor.to_le_bytes());
    let active = &switched.private[2];
    assert_eq!(&active[..3], [53, 0, 9]);
    assert_eq!(&active[3..7], &second.to_le_bytes());
    assert_eq!(&active[7..15], &actor.to_le_bytes());
    assert_eq!(service.active_pet_id(), Some(second));
}

#[test]
fn an_item_record_with_a_wrong_tint_refuses_without_consuming_or_saving() {
    let file = SaveFile::new();
    let mut service = service();
    let save = |service: &PetService| {
        game_store::save(
            &file.path,
            &game_store::Snapshot {
                character: owner(),
                gameplay: None,
                pets: service.document_value(),
            },
        )
        .unwrap()
    };
    save(&service);
    let saved_before = fs::read(&file.path).unwrap();
    let mut inventory = inventory_handle(70352);
    let inventory_before = inventory.state().clone();
    let roster_before = service.roster_snapshot();

    // The old cross-field fallback read the first word as an instance or
    // definition id when the second found nothing, so this pair consumed the
    // owned 70352 item and created a pet.
    let wire = service
        .handle_at_seconds(
            &summon_by_item(70352, 999_999),
            OWNER,
            [0.0; 3],
            &mut inventory,
            PetBusy::default(),
            NOW,
        )
        .unwrap()
        .expect("refusal is a private roster refresh");
    assert!(wire.notice.is_some());
    assert!(!wire.changed);
    assert!(wire.shared.is_empty());
    assert_eq!(inventory.state(), &inventory_before);
    assert_eq!(service.roster_snapshot(), roster_before);
    assert_eq!(service.active_pet_id(), None);
    assert!(service.pet(1).is_none());
    assert_eq!(fs::read(&file.path).unwrap(), saved_before);
    save(&service);
    assert_eq!(fs::read(&file.path).unwrap(), saved_before);
}

#[test]
fn a_package_configured_pet_speed_reaches_the_actor_packet() {
    let mut document: serde_json::Value =
        serde_json::from_str(include_str!("../data/content/pets.json")).unwrap();
    document["movement_speed"]["speed"] = serde_json::json!(7.5);
    let content = PetContent::from_json(&serde_json::to_vec(&document).unwrap()).unwrap();
    let mut service = PetService::new(content).unwrap();
    let pet_id = service.adopt_vendor_grant(1, &definition(), 0).unwrap();
    service
        .handle(
            &summon(pet_id),
            OWNER,
            [1.0, 2.0, 3.0],
            PetBusy::default(),
            NOW,
        )
        .unwrap();
    let presence = service.view(OWNER).unwrap();
    assert_eq!(presence.speed, 7.5);
    let npc = startup_world::parse_add_npc(&presence.spawn_packet().unwrap()).unwrap();
    assert_eq!(npc.speed, 7.5);

    document["movement_speed"]["speed"] = serde_json::json!(0.0);
    assert!(PetContent::from_json(&serde_json::to_vec(&document).unwrap()).is_err());
    document["movement_speed"]["speed"] = serde_json::json!(1000.0);
    assert!(PetContent::from_json(&serde_json::to_vec(&document).unwrap()).is_err());
}

/// T772: a granted adoption item (`realm give`, a buy) becomes a pet at once through
/// `adopt_granted`: each granted unit is consumed into its own pet, units past the roster's room
/// stay owned, and the 53/5 roster closes the replies once.
#[test]
fn gained_adoption_items_become_pets_up_to_the_roster_room() {
    let mut service = service();
    // 49 pets already: one slot left.
    for grant in 1..=49 {
        service.adopt_vendor_grant(grant, &definition(), 0).unwrap();
    }
    assert_eq!(service.room(), 1);
    let mut inventory = inventory_handle(70352);
    let mut granted = Vec::new();
    for _ in 0..2 {
        granted.extend(inventory.grant(0, 70352, 1).unwrap().granted);
    }
    let replies = service.adopt_granted(&granted, &mut inventory, 10).unwrap();
    assert_eq!(service.room(), 0, "the roster filled to its 50 pets");
    let left: u32 = inventory
        .state()
        .items
        .iter()
        .filter(|item| item.definition_id == 70352)
        .map(|item| item.count)
        .sum();
    assert_eq!(left, 2, "units past the roster's room stay owned");
    assert!(replies.last().unwrap().starts_with(&[53, 0, 5]));
    assert_eq!(
        replies
            .iter()
            .filter(|p| p.starts_with(&[53, 0, 5]))
            .count(),
        1
    );
    // Nothing granted: nothing adopted, no replies.
    assert!(
        service
            .adopt_granted(&[], &mut inventory, 11)
            .unwrap()
            .is_empty()
    );
}
