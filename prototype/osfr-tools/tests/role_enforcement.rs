use osfr_tools::{account_store::AccountRole, chat, gameplay::Gameplay};
use std::path::PathBuf;

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

#[test]
fn verified_owner_roles_persist_and_online_demotion_is_immediate() {
    use osfr_tools::{
        account::{AccountFlags, SlotPolicy},
        account_store::{AccountStore, AccountStoreLock},
        gateway::Gateway,
    };
    let scratch = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("w120-role-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    let path = scratch.join("accounts.json");
    let mut store = AccountStore::new();
    for hash in [1, 2] {
        store
            .create_account(
                [hash; 32],
                AccountFlags::default(),
                SlotPolicy { max_characters: 1 },
            )
            .unwrap();
    }
    store
        .set_account_role_offline(2, AccountRole::Admin)
        .unwrap();
    store.save(&path).unwrap();
    let reference = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-gateway-reference");
    let content = reference.with_file_name("rust-opening-cooking-original-assets");
    let initial = Gateway::reference_state(&reference).unwrap();
    let mut gateway = Gateway::load_gameplay(&reference, initial, &content, None).unwrap();
    gateway.set_dev_tools(true);
    let mut login = 1u16.to_le_bytes().to_vec();
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(reference.join("manifest.json")).unwrap()).unwrap();
    for (i, value) in ["w120", manifest["client_version"].as_str().unwrap(), ""]
        .into_iter()
        .enumerate()
    {
        login.extend((value.len() as u32).to_le_bytes());
        login.extend(value.as_bytes());
        if i == 0 {
            login.extend(1u64.to_le_bytes());
        }
    }
    gateway.handle(&login).unwrap();
    gateway.handle(&wrap(&[13, 0])).unwrap();
    gateway.handle(&wrap(&[10, 0])).unwrap();
    // Character GUID 1 is deliberately different from verified owner account 2.
    gateway.bind_account_role(&path, 1);
    let denied = gateway
        .handle(&wrap(&chat::local_request("realm god")))
        .unwrap();
    assert_eq!(denied.dev_command_denials, 1);
    gateway.bind_account_role(&path, 2);
    let send = |gateway: &mut Gateway, text: &str| {
        gateway.handle(&wrap(&chat::local_request(text))).unwrap()
    };
    assert!(
        send(&mut gateway, "realm role 0 admin")
            .replies
            .iter()
            .any(|p| String::from_utf8_lossy(p).contains("Usage:"))
    );
    let lock = AccountStoreLock::acquire(&path).unwrap();
    assert!(
        send(&mut gateway, "realm role 1 builder")
            .replies
            .iter()
            .any(|p| String::from_utf8_lossy(p).contains("busy"))
    );
    assert_eq!(
        AccountStore::load(&path).unwrap().account_role(1),
        Some(AccountRole::Player)
    );
    drop(lock);
    assert!(
        send(&mut gateway, "realm role 1 builder")
            .replies
            .iter()
            .any(|p| String::from_utf8_lossy(p).contains("Account role saved"))
    );
    assert_eq!(
        AccountStore::load(&path).unwrap().account_role(1),
        Some(AccountRole::Builder)
    );
    assert!(
        send(&mut gateway, "realm role 2 player")
            .replies
            .iter()
            .any(|p| String::from_utf8_lossy(p).contains("Account role saved"))
    );
    assert_eq!(gateway.role(), AccountRole::Player);
    assert_eq!(send(&mut gateway, "realm god").dev_command_denials, 1);
    assert_eq!(
        send(&mut gateway, "realm role 2 admin").dev_command_denials,
        1
    );
    // Refresh fails closed on loss of the trusted store.
    std::fs::remove_file(&path).unwrap();
    assert_eq!(send(&mut gateway, "realm here").dev_command_denials, 1);
    std::fs::remove_dir_all(&scratch).unwrap();
}

fn game() -> (Gameplay, osfr_tools::character_store::CharacterState) {
    let content = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-opening-cooking-original-assets");
    let character = Gameplay::initial_character(
        &content,
        osfr_tools::character_store::CharacterState {
            guid: 1,
            zone_id: 0,
            position: [0.; 3],
            rotation: [0., 0., 1.],
        },
    )
    .unwrap();
    let mut game = Gameplay::load(&content, None).unwrap();
    game.set_dev_tools(true);
    (game, character)
}

#[test]
fn player_and_builder_refusals_cover_every_tool_route() {
    let (mut game, character) = game();
    for role in [AccountRole::Player, AccountRole::Builder] {
        game.set_role(role);
        for command in [
            "god",
            "heal",
            "coins 1",
            "give 1195",
            "level 5",
            "battle 96",
            "win",
            "sim",
            "fightlog",
            "npx off",
            "offer 3000021 v1",
            "mine",
            "harvest",
            "role 1 admin",
            "reload accounts",
            "future_command",
        ] {
            let before = game.state();
            let out = game
                .handle_in_world(
                    &chat::local_request(&format!("realm {command}")),
                    &character,
                    1000,
                )
                .unwrap();
            let text = out
                .replies
                .iter()
                .flat_map(|p| p.iter().copied())
                .collect::<Vec<_>>();
            assert!(
                String::from_utf8_lossy(&text).contains("Cannot do that"),
                "{role:?}: {command}"
            );
            assert_eq!(game.state(), before, "{role:?}: {command}");
        }
    }
}

#[test]
fn builder_tools_admin_cheats_and_feature_switch_are_independent() {
    let (mut game, character) = game();
    for (role, enabled, allowed) in [
        (AccountRole::Player, true, false),
        (AccountRole::Builder, true, true),
        (AccountRole::Admin, true, true),
        (AccountRole::Admin, false, false),
    ] {
        game.set_role(role);
        game.set_dev_tools(enabled);
        for command in [
            "here",
            "marks",
            "placements",
            "reload battles",
            "reload enemies",
            "reload abilities",
            "reload behaviours",
            "reload placements",
            "reload all",
        ] {
            let out = game
                .handle_in_world(
                    &chat::local_request(&format!("realm {command}")),
                    &character,
                    1000,
                )
                .unwrap();
            let text = out
                .replies
                .iter()
                .flat_map(|p| p.iter().copied())
                .collect::<Vec<_>>();
            assert_eq!(
                !String::from_utf8_lossy(&text).contains("Cannot do that"),
                allowed,
                "{role:?}: {command}"
            );
        }
    }
    game.set_dev_tools(true);
    let out = game
        .handle_in_world(&chat::local_request("realm god"), &character, 1000)
        .unwrap();
    assert!(
        out.replies
            .iter()
            .any(|p| String::from_utf8_lossy(p).contains("God mode on"))
    );
}
