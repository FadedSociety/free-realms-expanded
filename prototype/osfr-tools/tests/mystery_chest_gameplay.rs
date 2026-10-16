//! Private content integration; no game assets or donor code embedded here.
use osfr_tools::{character_store::CharacterState, game_store, gameplay::Gameplay};
use std::{fs, path::PathBuf};

fn content() -> PathBuf {
    PathBuf::from(std::env::var_os("OSFR_CHEST_CONTENT").expect("private chest test package"))
}
fn character() -> CharacterState {
    serde_json::from_value(
        serde_json::json!({"guid":1,"zone_id":1,"position":[0.0,0.0,0.0],"rotation":[0.0,0.0,1.0]}),
    )
    .unwrap()
}
fn request(sub: u16, id: i32) -> Vec<u8> {
    [
        42u16.to_le_bytes().as_slice(),
        sub.to_le_bytes().as_slice(),
        id.to_le_bytes().as_slice(),
    ]
    .concat()
}
fn ids(game: &Gameplay) -> (i32, i32) {
    let s = game.state();
    (
        s.inventory
            .items
            .iter()
            .find(|i| i.definition_id == 910100)
            .unwrap()
            .id,
        s.inventory
            .items
            .iter()
            .find(|i| i.definition_id == 78137)
            .unwrap()
            .id,
    )
}
fn count(game: &Gameplay, definition: i32) -> u32 {
    game.state()
        .inventory
        .items
        .iter()
        .filter(|i| i.definition_id == definition)
        .map(|i| i.count)
        .sum()
}

#[test]
#[ignore = "requires private reconstructed Mystery Chest test package"]
fn private_confirmation_atomic_grant_replay_and_disk_restore() {
    let dir = content();
    let mut game = Gameplay::load(&dir, None).unwrap();
    let ch = character();
    let (chest, key) = ids(&game);
    let before = game.state();
    let check = game.handle_in_world(&request(16, chest), &ch, 0).unwrap();
    assert_eq!(
        check.replies.len(),
        1,
        "normal check must produce confirmation packet"
    );
    assert_eq!(&check.replies[0][..4], &[38, 0, 30, 0]);
    assert!(!check.changed);
    assert_eq!(game.state(), before);
    let won = game.handle_in_world(&request(15, chest), &ch, 1).unwrap();
    if let Some(dir) = std::env::var_os("OSFR_CHEST_EVIDENCE") {
        let dir = PathBuf::from(dir);
        fs::write(dir.join("check-reply.bin"), &check.replies[0]).unwrap();
        for (index, packet) in won.replies.iter().enumerate() {
            fs::write(dir.join(format!("grant-reply-{index}.bin")), packet).unwrap();
        }
    }
    assert!(won.changed);
    assert_eq!(count(&game, 910100), 1);
    assert_eq!(count(&game, 78137), 1);
    assert_eq!(count(&game, 11584), 1);
    assert!(won.replies.iter().any(|p| p.starts_with(&[50, 0, 1])));
    let after = game.state();
    let repeat = game.handle_in_world(&request(15, chest), &ch, 2).unwrap();
    assert!(!repeat.changed);
    assert_eq!(
        game.state(),
        after,
        "replayed confirm must not open remaining stack"
    );
    let save = dir
        .parent()
        .unwrap()
        .join(format!("chest-save-{}.json", std::process::id()));
    let snap = game_store::Snapshot {
        character: ch.clone(),
        gameplay: Some(after.clone()),
    };
    game_store::save(&save, &snap).unwrap();
    let restored = game_store::load_or_create(&save, ch.clone()).unwrap();
    let mut fresh = Gameplay::load(&dir, restored.gameplay).unwrap();
    assert_eq!(fresh.state(), after);
    assert!(
        !fresh
            .handle_in_world(&request(15, chest), &ch, 3)
            .unwrap()
            .changed
    );
    assert_eq!(fresh.state(), after);
    fresh.handle_in_world(&request(16, chest), &ch, 4).unwrap();
    assert!(
        fresh
            .handle_in_world(&request(15, chest), &ch, 5)
            .unwrap()
            .changed
    );
    assert_eq!(count(&fresh, 910100), 0);
    assert_eq!(count(&fresh, 78137), 0);
    assert_eq!(count(&fresh, 11584), 2);
    let final_state = fresh.state();
    assert!(
        !fresh
            .handle_in_world(&request(15, chest), &ch, 6)
            .unwrap()
            .changed
    );
    assert_eq!(fresh.state(), final_state);
    assert!(!final_state.inventory.items.iter().any(|i| i.id == key));
    fs::remove_file(save).unwrap();
}

#[test]
#[ignore = "requires private reconstructed Mystery Chest test package"]
fn private_missing_key_forged_item_and_malformed_requests_conserve_everything() {
    let dir = content();
    let base = Gameplay::load(&dir, None).unwrap();
    let ch = character();
    let (chest, _) = ids(&base);
    let mut state = base.state();
    state.inventory.items.retain(|i| i.definition_id != 78137);
    let mut game = Gameplay::load(&dir, Some(state)).unwrap();
    let before = game.state();
    let check = game.handle_in_world(&request(16, chest), &ch, 0).unwrap();
    assert_eq!(check.replies[0].last(), Some(&0));
    assert!(
        !game
            .handle_in_world(&request(15, chest), &ch, 1)
            .unwrap()
            .changed
    );
    assert_eq!(game.state(), before);
    let normal_item = before
        .inventory
        .items
        .iter()
        .find(|i| i.definition_id != 910100)
        .unwrap()
        .id;
    for item in [normal_item, i32::MAX] {
        assert_eq!(
            game.handle_in_world(&request(16, item), &ch, 2)
                .unwrap()
                .replies[0]
                .last(),
            Some(&0)
        );
        assert!(
            !game
                .handle_in_world(&request(15, item), &ch, 3)
                .unwrap()
                .changed
        );
        assert_eq!(game.state(), before);
    }
    for sub in [16, 15] {
        let mut malformed = request(sub, chest);
        malformed.push(0);
        assert!(game.handle_in_world(&malformed, &ch, 4).is_err());
        assert_eq!(game.state(), before);
    }
}

#[test]
#[ignore = "requires private reconstructed Mystery Chest test package"]
fn private_failed_reward_allocation_rolls_back_both_consumptions() {
    let dir = content();
    let base = Gameplay::load(&dir, None).unwrap();
    let ch = character();
    let (chest, _) = ids(&base);
    let mut state = base.state();
    state.inventory.next_instance_id = Some(i64::from(i32::MAX) + 1);
    let mut game = Gameplay::load(&dir, Some(state)).unwrap();
    let before = game.state();
    game.handle_in_world(&request(16, chest), &ch, 0).unwrap();
    assert!(game.handle_in_world(&request(15, chest), &ch, 1).is_err());
    assert_eq!(
        game.state(),
        before,
        "chest and key must survive failed grant"
    );
}

#[test]
#[ignore = "requires private reconstructed Mystery Chest test package"]
fn private_confirmation_is_cleared_on_session_reset() {
    let mut game = Gameplay::load(&content(), None).unwrap();
    let ch = character();
    let (chest, _) = ids(&game);
    game.handle_in_world(&request(16, chest), &ch, 0).unwrap();
    let before = game.state();
    game.reset_session();
    assert!(
        !game
            .handle_in_world(&request(15, chest), &ch, 1)
            .unwrap()
            .changed
    );
    assert_eq!(game.state(), before);
}
