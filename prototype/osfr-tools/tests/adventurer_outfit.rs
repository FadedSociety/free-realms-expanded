//! The Adventurer has no starter kit in `profiles.json`: it wears the creation clothes
//! `apply_identity` gives it once. A save whose Adventurer lost them could never switch back,
//! since the client's `ProcessProfileSwap` (`00bfb620`) needs Chest and Legs on every job (native
//! run 2026-11-07: the HUD job arrow stuck on "missing a required Chest or Legs item").
use osfr_tools::{
    gameplay::Gameplay,
    identity::{Attachment, CharacterIdentity, NativeCreation},
};
use serde_json::Value;
use std::{fs, path::PathBuf};

fn release() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/release")
}

fn identity() -> CharacterIdentity {
    let raw: Value =
        serde_json::from_slice(&fs::read(release().join("inventory.json")).unwrap()).unwrap();
    let piece = |id: i64, tint: i32| {
        let d = raw["definitions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["id"] == id)
            .expect("creation clothing definition");
        Attachment {
            definition: id as i32,
            model_name: d["model"].as_str().unwrap().into(),
            texture_alias: d["texture"].as_str().unwrap().into(),
            tint_alias: d["tint_alias"].as_str().unwrap().into(),
            tint_id: tint,
            composite_effect_id: 0,
            slot: d["slot"].as_i64().unwrap() as i32,
        }
    };
    CharacterIdentity {
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
        gender: 2,
        creation: NativeCreation {
            first_name: "Ari".into(),
            last_name: "Vale".into(),
            temporary_first_name: String::new(),
            temporary_last_name: String::new(),
            choices: [0; 21],
        },
        // Creation clothes: chest, legs, feet.
        attachments: vec![piece(199, 241), piece(76504, 253), piece(76505, 241)],
    }
}

fn select(game: &mut Gameplay, job: i32) -> Vec<Vec<u8>> {
    let mut request = vec![26, 0, 13, 0];
    request.extend(job.to_le_bytes());
    game.handle(&request, 1, [0.0; 3], 0).unwrap().replies
}

fn says(replies: &[Vec<u8>], text: &str) -> bool {
    replies.iter().any(|reply| String::from_utf8_lossy(reply).contains(text))
}

#[test]
fn a_bare_adventurer_is_dressed_in_its_creation_clothes_on_the_way_back() {
    let identity = identity();
    let mut game = Gameplay::load(&release(), None).unwrap();
    assert!(game.apply_identity(&identity).unwrap());
    assert_eq!(game.active_profile(), Some(1));
    select(&mut game, 43);
    assert_eq!(game.active_profile(), Some(43), "the first switch away");

    // An older save: the Adventurer's outfit is empty.
    let mut state = game.state();
    for row in state.inventory.equipment.iter_mut().filter(|row| row.profile_id == 1) {
        row.item_id = 0;
    }
    let mut game = Gameplay::load(&release(), Some(state)).unwrap();
    assert!(!game.apply_identity(&identity).unwrap(), "the identity is already applied");

    let replies = select(&mut game, 1);
    assert!(
        !says(&replies, "missing a required Chest or Legs"),
        "the switch back was refused"
    );
    assert_eq!(game.active_profile(), Some(1), "back in the Adventurer");
    let state = game.state();
    for slot in [3, 4] {
        let row = state
            .inventory
            .equipment
            .iter()
            .find(|row| row.profile_id == 1 && row.slot == slot && row.item_id != 0)
            .unwrap_or_else(|| panic!("the Adventurer wears nothing in slot {slot}"));
        let item = state.inventory.items.iter().find(|item| item.id == row.item_id).unwrap();
        assert_eq!(item.definition_id, [199, 76504][slot as usize - 3]);
    }
    // The owned creation clothes are worn again, not granted twice.
    assert_eq!(
        state.inventory.items.iter().filter(|item| item.definition_id == 199).count(),
        1
    );
}
