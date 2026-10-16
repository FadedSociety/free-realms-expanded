use osfr_tools::{
    inventory::Inventory,
    profiles::{ProfileState, Profiles},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn signed(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

struct Fixture {
    path: PathBuf,
    profiles: Value,
    template: Vec<u8>,
    offsets: [usize; 2],
    active: usize,
}
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-profiles-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        let mut template = vec![5, 0, 1, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0];
        template.extend(0i32.to_le_bytes());
        let mut offsets = [0; 2];
        let mut proofs = Vec::new();
        let mut spans = vec![
            json!({"kind":"coins","offset":13,"length":4,"profile_id":0,"sha256":digest(&0i32.to_le_bytes())}),
        ];
        for (index, id) in [1, 43].into_iter().enumerate() {
            offsets[index] = template.len();
            let raw: Vec<_> = [1i32, 0, 0, 1, 3, 0, 3, 0]
                .into_iter()
                .flat_map(i32::to_le_bytes)
                .collect();
            proofs.push(json!({"id":id,"offset":template.len(),"length":raw.len(),"sha256":digest(&raw),"serialized_hex":hex(&raw),"rank_offset":0,"equipment_offset":12,"equipment_length":16}));
            spans.push(json!({"kind":"equipment","offset":template.len()+12,"length":16,"profile_id":id,"sha256":digest(&raw[12..28])}));
            template.extend(raw);
        }
        let active = template.len();
        template.extend(1i32.to_le_bytes());
        spans.push(json!({"kind":"items","offset":template.len(),"length":4,"profile_id":0,"sha256":digest(&0i32.to_le_bytes())}));
        template.extend(0i32.to_le_bytes());
        template.extend(1i32.to_le_bytes()); // GenderFilter after the original items.
        let len = template.len();
        template[3..7].copy_from_slice(&((len - 7) as u32).to_le_bytes());
        template[9..13].copy_from_slice(&((len - 13) as u32).to_le_bytes());
        let definition = [100i32.to_le_bytes(), 0i32.to_le_bytes()].concat();
        let inventory = json!({"schema_version":1,"player_guid":1,
            "default_state":{"coins":0,"items":[],"equipment":[{"profile_id":1,"slot":3,"item_id":0,"category":3},{"profile_id":43,"slot":3,"item_id":0,"category":3}]},
            "profiles":[{"id":1,"allowed_classes":[1],"slots":[3]},{"id":43,"allowed_classes":[1],"slots":[3]}],"spans":spans,
            "definitions":[{"id":100,"slot":3,"class_id":1,"default_tint":5,"max_stack":1,"model":"shirt.adr","texture":"cloth","tint_alias":"dye","effect":0,"wield_type":0,"serialized_hex":hex(&definition),"sha256":digest(&definition)}]});
        fs::write(
            path.join("inventory.json"),
            serde_json::to_vec(&inventory).unwrap(),
        )
        .unwrap();
        let profiles = json!({"schema_version":1,"active_offset":active,"active_sha256":digest(&1i32.to_le_bytes()),"profiles":proofs,
            "default_state":{"version":1,"active":1,"unlocked":[1],"experience":{}},"starter_items":{"43":[100]}});
        let f = Self {
            path,
            profiles,
            template,
            offsets,
            active,
        };
        f.write();
        f
    }
    fn write(&self) {
        fs::write(
            self.path.join("profiles.json"),
            serde_json::to_vec(&self.profiles).unwrap(),
        )
        .unwrap();
    }
    fn load(&self) -> Profiles {
        Profiles::load(&self.path, None).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        for name in ["profiles.json", "inventory.json"] {
            let _ = fs::remove_file(self.path.join(name));
        }
        let _ = fs::remove_dir(&self.path);
    }
}

#[test]
fn class_unlock_selection_activation_and_restart_retain_equipment() {
    let f = Fixture::new();
    let mut profiles = f.load();
    let mut inventory = Inventory::load(&f.path, None).unwrap();
    let before = profiles.state().clone();
    assert!(profiles.select(43, &inventory).is_err());
    assert_eq!(profiles.state(), &before);
    assert!(profiles.unlock(43).unwrap());
    assert!(!profiles.unlock(43).unwrap());
    assert_eq!(profiles.starter_items(43), &[100]);
    inventory.grant(0, 100, 1).unwrap();
    inventory.equip_owned_definition(100, 43).unwrap();
    let packet = profiles.select(43, &inventory).unwrap();
    assert_eq!(&packet[..4], &[38, 0, 21, 0]);
    assert_eq!(signed(&packet, 4), 32);
    assert_eq!(
        (
            signed(&packet, 20),
            signed(&packet, 24),
            signed(&packet, 28)
        ),
        (1, 3, 1)
    );
    assert_eq!(
        &packet[packet.len() - 12..],
        &[
            3001i32.to_le_bytes(),
            4005i32.to_le_bytes(),
            0i32.to_le_bytes()
        ]
        .concat()
    );
    let saved: ProfileState =
        serde_json::from_slice(&serde_json::to_vec(profiles.state()).unwrap()).unwrap();
    let restored = Profiles::load(&f.path, Some(saved)).unwrap();
    let restored_inventory = Inventory::load(&f.path, Some(inventory.state().clone())).unwrap();
    assert_eq!(
        restored.activation(43, &restored_inventory).unwrap(),
        packet
    );
}

#[test]
fn experience_updates_only_selected_class_and_overflow_does_not_change_state() {
    let f = Fixture::new();
    let mut profiles = f.load();
    profiles.add_experience(125).unwrap();
    let patched = profiles.patch_self(&f.template).unwrap();
    assert_eq!(
        (
            signed(&patched, f.offsets[0]),
            signed(&patched, f.offsets[0] + 4),
            signed(&patched, f.offsets[0] + 8)
        ),
        (2, 25, 0) // The third field is upgrade stars, never raw local XP.
    );
    assert_eq!(signed(&patched, f.offsets[1]), 1);
    let mut state = profiles.state().clone();
    state.experience.insert(1, u32::MAX);
    let mut profiles = Profiles::load(&f.path, Some(state.clone())).unwrap();
    let capped = profiles.patch_self(&f.template).unwrap();
    assert_eq!(
        (
            signed(&capped, f.offsets[0]),
            signed(&capped, f.offsets[0] + 4),
            signed(&capped, f.offsets[0] + 8)
        ),
        (20, 0, 0)
    );
    assert!(!profiles.add_experience(0).unwrap());
    assert!(profiles.add_experience(1).is_err());
    assert_eq!(profiles.state(), &state);
}

#[test]
fn snapshot_rejects_unknown_locked_or_unsupported_classes() {
    let f = Fixture::new();
    let mut state = f.load().state().clone();
    state.active = 43;
    assert!(Profiles::load(&f.path, Some(state)).is_err());
    let mut state = f.load().state().clone();
    state.unlocked.insert(999);
    assert!(Profiles::load(&f.path, Some(state)).is_err());
    let mut state = f.load().state().clone();
    state.experience.insert(43, 1);
    assert!(Profiles::load(&f.path, Some(state)).is_err());
    let mut state = f.load().state().clone();
    state.version = 2;
    assert!(Profiles::load(&f.path, Some(state)).is_err());
}

#[test]
fn native_experience_updates_have_exact_fields_and_rank_notification_precedes_update() {
    let f = Fixture::new();
    let mut profiles = f.load();
    profiles.add_experience(75).unwrap();
    let update = profiles.experience_packets(0);
    assert_eq!(update.len(), 1);
    assert_eq!(update[0].len(), 28);
    assert_eq!(&update[0][..4], &[38, 0, 14, 0]);
    assert_eq!(
        &update[0][4..],
        &[1i32, 1, 75, 0, 0, 0]
            .into_iter()
            .flat_map(i32::to_le_bytes)
            .collect::<Vec<_>>()
    );
    profiles.add_experience(50).unwrap();
    let update = profiles.experience_packets(75);
    assert_eq!(update.len(), 2);
    assert_eq!(
        update[0],
        [
            vec![38, 0, 18, 0],
            1i32.to_le_bytes().to_vec(),
            2i32.to_le_bytes().to_vec()
        ]
        .concat()
    );
    assert_eq!(&update[1][..4], &[38, 0, 14, 0]);
    assert_eq!(
        &update[1][4..],
        &[1i32, 2, 25, 0, 0, 0]
            .into_iter()
            .flat_map(i32::to_le_bytes)
            .collect::<Vec<_>>()
    );
    profiles.add_experience(2000).unwrap();
    let update = profiles.experience_packets(125);
    assert_eq!(signed(&update[0], 8), 20);
    assert_eq!(
        &update[1][4..],
        &[1i32, 20, 0, 0, 0, 0]
            .into_iter()
            .flat_map(i32::to_le_bytes)
            .collect::<Vec<_>>()
    );
}

#[test]
fn profile_proofs_and_field_bounds_reject_corruption() {
    let mut f = Fixture::new();
    let original = f.profiles.clone();
    for field in [
        "rank_offset",
        "equipment_offset",
        "equipment_length",
        "length",
    ] {
        f.profiles = original.clone();
        f.profiles["profiles"][0][field] = json!(usize::MAX);
        f.write();
        assert!(Profiles::load(&f.path, None).is_err(), "{field}");
    }
    f.profiles = original;
    f.profiles["profiles"][0]["sha256"] = json!("00");
    f.write();
    assert!(Profiles::load(&f.path, None).is_err());
}

#[test]
fn gender_and_class_splices_preserve_unowned_bytes_and_reject_unproved_input() {
    let f = Fixture::new();
    let inventory = Inventory::load(&f.path, None).unwrap();
    let profiles = f.load();
    let mut template = inventory.patch_gender(&f.template, 2).unwrap();
    assert_eq!(signed(&template, template.len() - 4), 2);
    let patched = inventory
        .initialization_patch(&profiles.patch_self(&template).unwrap())
        .unwrap();
    assert_eq!(patched, template);
    assert_eq!(signed(&patched, f.active), 1);
    assert!(inventory.patch_gender(&f.template, 0).is_err());
    assert!(
        inventory
            .patch_gender(&f.template[..f.template.len() - 1], 1)
            .is_err()
    );
    template[f.offsets[0] + 1] ^= 1;
    assert!(profiles.patch_self(&template).is_err());
}

#[test]
#[ignore = "requires OSFR_OPENING_CONTENT and OSFR_GATEWAY_REFERENCE private files"]
fn private_all_opening_starters_and_fresh_identity_survive_activation_and_restore() {
    use osfr_tools::{
        gameplay::Gameplay,
        identity::{Attachment, CharacterIdentity, NativeCreation},
    };
    let content = PathBuf::from(std::env::var("OSFR_OPENING_CONTENT").expect("opening content"));
    let reference =
        PathBuf::from(std::env::var("OSFR_GATEWAY_REFERENCE").expect("gateway reference"));
    let manifest: Value =
        serde_json::from_slice(&fs::read(reference.join("manifest.json")).unwrap()).unwrap();
    let template =
        fs::read(reference.join(manifest["templates"][7]["file"].as_str().unwrap())).unwrap();
    let mut profiles = Profiles::load(&content, None).unwrap();
    let mut inventory = Inventory::load(&content, None).unwrap();
    assert_eq!(inventory.state().items.len(), 7);
    assert_eq!(inventory.state().coins, 0);
    for id in [14, 16, 43, 45, 11, 35] {
        profiles.unlock(id).unwrap();
        for definition in profiles.starter_items(id).to_vec() {
            inventory.grant(0, definition, 1).unwrap();
            inventory.equip_owned_definition(definition, id).unwrap();
        }
        let packet = profiles.select(id, &inventory).unwrap();
        assert_eq!(&packet[..4], &[38, 0, 21, 0]);
        let restored = Inventory::load(&content, Some(inventory.state().clone())).unwrap();
        assert_eq!(profiles.activation(id, &restored).unwrap(), packet);
    }
    let raw: Value =
        serde_json::from_slice(&fs::read(content.join("inventory.json")).unwrap()).unwrap();
    let definition = raw["definitions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["id"] == 199)
        .expect("native creator shirt");
    for gender in [1, 2] {
        let identity = CharacterIdentity {
            guid: 1,
            first_name: "Ari".into(),
            last_name: "Vale".into(),
            model: 1,
            head: "head.adr".into(),
            hair: "hair.adr".into(),
            model_customization: String::new(),
            face_paint: "none".into(),
            skin_tone: "skin".into(),
            eye_color: 262,
            hair_color: 116,
            gender,
            creation: NativeCreation {
                first_name: "Ari".into(),
                last_name: "Vale".into(),
                temporary_first_name: String::new(),
                temporary_last_name: String::new(),
                choices: [0; 21],
            },
            attachments: vec![Attachment {
                definition: 199,
                model_name: definition["model"].as_str().unwrap().into(),
                texture_alias: definition["texture"].as_str().unwrap().into(),
                tint_alias: definition["tint_alias"].as_str().unwrap().into(),
                tint_id: 241,
                composite_effect_id: 0,
                slot: 3,
            }],
        };
        let mut game = Gameplay::load(&content, None).unwrap();
        assert!(game.apply_identity(&identity).unwrap());
        assert!(!game.apply_identity(&identity).unwrap());
        let state = game.state();
        let equipped = state
            .inventory
            .equipment
            .iter()
            .find(|e| e.profile_id == 1 && e.slot == 3)
            .unwrap();
        let shirt = state
            .inventory
            .items
            .iter()
            .find(|i| i.id == equipped.item_id)
            .unwrap();
        assert_eq!((shirt.definition_id, shirt.tint), (199, 241));
        let bytes = game.patch_gender(&template, gender).unwrap();
        let patched = game.patch_self(&bytes).unwrap();
        assert!(!patched.is_empty());
        let mut restored = Gameplay::load(&content, Some(state.clone())).unwrap();
        assert!(!restored.apply_identity(&identity).unwrap());
        assert_eq!(restored.state(), state);
        let mut changed = identity.clone();
        changed.gender = 3 - gender;
        assert!(restored.apply_identity(&changed).is_err());
        assert_eq!(restored.state(), state);
    }
}
