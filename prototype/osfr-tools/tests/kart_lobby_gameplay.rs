//! Private integration gate: synthetic catalogue, copied private campaign.
//! No captured tables, assets or player saves are embedded in this test.
use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    gateway::Gateway,
    kart_lobby_native::{self, LobbyNativeBody, TrailingBytes},
};
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let source = PathBuf::from(
            std::env::var_os("OSFR_KART_LOBBY_CONTENT").expect("private campaign content"),
        );
        let path = std::env::temp_dir().join(format!(
            "osfr-kart-lobby-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        for entry in fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            assert!(
                entry.file_type().unwrap().is_file(),
                "gate requires flat content directory"
            );
            fs::copy(entry.path(), path.join(entry.file_name())).unwrap();
        }
        Self(path)
    }
    fn catalogue(&self) -> LobbyNativeBody {
        // Deliberately synthetic values, exercising all five tables.
        let body = json!({
            "games": [{"key": 7, "id": 8, "game_type": 1, "unknown": -9, "name_id": 10}],
            "options": [{"key": 11, "unknown": [12, -13, 14]}],
            "values": [{"key": 15, "unknown": [-16, 17, 18]}],
            "display": [{"key": 19, "unknown": 20, "locale_id": 21, "text": [65, 0, 255]}],
            "settings": [{"key": 22, "fields": [23, 24, -25, 26, 27]}]
        });
        fs::write(
            self.0.join("kart_lobby.json"),
            serde_json::to_vec(&body).unwrap(),
        )
        .unwrap();
        serde_json::from_value(body).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn character() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: [0.0; 3],
        rotation: [0.0, 0.0, 1.0],
    }
}

#[test]
#[ignore = "requires OSFR_KART_LOBBY_CONTENT private campaign"]
fn private_lobby_opt_in_exact_request_full_reply_and_no_save_changes() {
    let fixture = Fixture::new();
    let mut game = Gameplay::load(&fixture.0, None).unwrap();
    let before = game.state();
    // No catalogue remains inert, including with developer tools enabled.
    game.set_dev_tools(true);
    game.set_role(osfr_tools::account_store::AccountRole::Admin);
    assert!(
        game.handle_in_world(&[102, 0, 1, 0], &character(), 0)
            .unwrap()
            .replies
            .is_empty()
    );
    assert_eq!(game.state(), before);

    let body = fixture.catalogue();
    let bytes_before = fs::read(fixture.0.join("kart_lobby.json")).unwrap();
    let mut game = Gameplay::load(&fixture.0, None).unwrap();
    let before = game.state();
    assert!(
        game.handle_in_world(&[102, 0, 1, 0], &character(), 0)
            .unwrap()
            .replies
            .is_empty()
    );
    game.set_dev_tools(true);
    game.set_role(osfr_tools::account_store::AccountRole::Admin);
    for time in [1, 2] {
        let reply = game
            .handle_in_world(&[102, 0, 1, 0], &character(), time)
            .unwrap();
        assert!(!reply.changed);
        assert!(reply.zoning.is_none() && reply.relocation.is_none());
        assert_eq!(reply.replies.len(), 1);
        assert_eq!(
            kart_lobby_native::decode_inner(&reply.replies[0], TrailingBytes::Strict)
                .unwrap()
                .body,
            body
        );
        assert_eq!(game.state(), before);
    }
    assert!(
        game.handle_in_world(&[102, 0, 2, 0], &character(), 3)
            .unwrap()
            .replies
            .is_empty()
    );
    for malformed in [vec![102, 0, 1], vec![102, 0, 1, 0, 0]] {
        assert!(game.handle_in_world(&malformed, &character(), 4).is_err());
        assert_eq!(game.state(), before);
    }
    assert_eq!(
        fs::read(fixture.0.join("kart_lobby.json")).unwrap(),
        bytes_before
    );
    game.set_dev_tools(false);
    assert!(
        game.handle_in_world(&[102, 0, 1, 0], &character(), 5)
            .unwrap()
            .replies
            .is_empty()
    );
}

#[test]
#[ignore = "requires OSFR_KART_LOBBY_CONTENT private campaign"]
fn private_lobby_load_refuses_malformed_oversized_and_unknown_fields() {
    let fixture = Fixture::new();
    for bytes in [
        b"{".to_vec(),
        vec![b' '; 256 * 1024 + 1],
        br#"{"games":[],"options":[],"values":[],"display":[],"settings":[],"guess":1}"#.to_vec(),
    ] {
        fs::write(fixture.0.join("kart_lobby.json"), &bytes).unwrap();
        assert!(Gameplay::load(&fixture.0, None).is_err());
        assert_eq!(fs::read(fixture.0.join("kart_lobby.json")).unwrap(), bytes);
    }
    fs::remove_file(fixture.0.join("kart_lobby.json")).unwrap();
    fs::create_dir(fixture.0.join("kart_lobby.json")).unwrap();
    assert!(Gameplay::load(&fixture.0, None).is_err());
}

#[test]
#[ignore = "requires OSFR_KART_LOBBY_CONTENT and OSFR_GATEWAY_REFERENCE private inputs"]
fn private_gateway_wraps_catalogue_once_and_preserves_whole_snapshot() {
    let fixture = Fixture::new();
    let body = fixture.catalogue();
    let reference = PathBuf::from(
        std::env::var_os("OSFR_GATEWAY_REFERENCE").expect("private gateway reference"),
    );
    let character = Gateway::reference_state(&reference).unwrap();
    let mut gateway = Gateway::load_gameplay(&reference, character, &fixture.0, None).unwrap();
    gateway.set_dev_tools(true);
    gateway.set_role(osfr_tools::account_store::AccountRole::Admin);
    let mut login = 1u16.to_le_bytes().to_vec();
    for (index, text) in ["local-private-test", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
        login.extend((text.len() as u32).to_le_bytes());
        login.extend(text.as_bytes());
        if index == 0 {
            login.extend(1u64.to_le_bytes());
        }
    }
    fn wrap(inner: &[u8]) -> Vec<u8> {
        let mut bytes = vec![5, 0, 1];
        bytes.extend((inner.len() as u32).to_le_bytes());
        bytes.extend(inner);
        bytes
    }
    gateway.handle_at(&login, 0).unwrap();
    gateway
        .handle_at(&wrap(&13u16.to_le_bytes()), 1_000)
        .unwrap();
    gateway
        .handle_at(&wrap(&10u16.to_le_bytes()), 1_000)
        .unwrap();
    let before = gateway.snapshot();
    for time in [2_000, 3_000] {
        let reply = gateway.handle_at(&wrap(&[102, 0, 1, 0]), time).unwrap();
        assert!(!reply.state_changed && !reply.gameplay_changed && !reply.entered_world);
        assert_eq!(reply.replies.len(), 1);
        assert_eq!(
            kart_lobby_native::decode_tunnel(&reply.replies[0], TrailingBytes::Strict)
                .unwrap()
                .body,
            body
        );
        assert_eq!(gateway.snapshot(), before);
    }
    assert!(gateway.handle_at(&wrap(&[102, 0, 1, 0, 0]), 4_000).is_err());
    assert_eq!(gateway.snapshot(), before);
    gateway.set_dev_tools(false);
    assert!(
        gateway
            .handle_at(&wrap(&[102, 0, 1, 0]), 5_000)
            .unwrap()
            .replies
            .is_empty()
    );
    assert_eq!(gateway.snapshot(), before);
}
