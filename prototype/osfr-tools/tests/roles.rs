use osfr_tools::{
    account::AccountFlags,
    account_store::{AccountRole, AccountStore, can_run_realm_command},
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-roles-test-{}-{}",
            std::process::id(),
            NEXT_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn legacy_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/account/accounts-v1.json")
}

#[test]
fn roles_allow_only_the_documented_commands() {
    let builder_commands = [
        "place",
        "unplace",
        "mark",
        "unmark",
        "marks",
        "placements",
        "text",
        "texts",
        "goto",
        "here",
        "reload all",
        "reload battles",
        "reload enemies",
        "reload abilities",
        "reload behaviours",
        "reload placements",
    ];
    let admin_commands = [
        "god",
        "coins",
        "level",
        "give",
        "battle",
        "win",
        "sim",
        "role",
        "broadcast",
        "heal",
        "fightlog",
        "harvest",
        "mine",
        "quest",
        "future_command",
        "reload accounts",
    ];

    for command in builder_commands {
        assert!(
            can_run_realm_command(AccountRole::Builder, command),
            "{command}"
        );
        assert!(
            can_run_realm_command(AccountRole::Admin, command),
            "{command}"
        );
        assert!(
            !can_run_realm_command(AccountRole::Player, command),
            "{command}"
        );
    }
    for command in admin_commands {
        assert!(
            can_run_realm_command(AccountRole::Admin, command),
            "{command}"
        );
        assert!(
            !can_run_realm_command(AccountRole::Builder, command),
            "{command}"
        );
        assert!(
            !can_run_realm_command(AccountRole::Player, command),
            "{command}"
        );
    }
}

#[test]
fn schema_one_accounts_load_as_players_and_save_as_schema_two() {
    let store = AccountStore::load(&legacy_fixture()).unwrap();
    assert_eq!(store.account_role(1), Some(AccountRole::Player));

    let temp = TempDir::new();
    let path = temp.0.join("accounts.json");
    store.save(&path).unwrap();
    let document: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(document["schema_version"], 2);
    assert_eq!(document["store"]["roles"]["1"], "player");
    assert_eq!(
        AccountStore::load(&path).unwrap().account_role(1),
        Some(AccountRole::Player)
    );
}

#[test]
fn first_new_account_starts_as_player() {
    let mut store = AccountStore::new();
    let account = store
        .create_account(
            [1; 32],
            AccountFlags::default(),
            osfr_tools::account::SlotPolicy { max_characters: 1 },
        )
        .unwrap();
    assert_eq!(account, 1);
    assert_eq!(store.account_role(account), Some(AccountRole::Player));
}

#[test]
fn schema_two_rejects_missing_role_records() {
    let mut document: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/account/accounts-v1.json")).unwrap();
    document["schema_version"] = serde_json::json!(2);

    let temp = TempDir::new();
    let path = temp.0.join("accounts.json");
    fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
    assert_eq!(
        AccountStore::load(&path).unwrap_err().kind(),
        std::io::ErrorKind::InvalidData
    );
}

#[test]
fn only_admins_can_change_roles_and_changes_persist() {
    let mut document: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/account/accounts-v1.json")).unwrap();
    document["schema_version"] = serde_json::json!(2);
    document["store"]["roles"] = serde_json::json!({ "1": "admin" });

    let temp = TempDir::new();
    let path = temp.0.join("accounts.json");
    fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
    let mut store = AccountStore::load(&path).unwrap();
    let target = store
        .create_account(
            [1; 32],
            AccountFlags::default(),
            osfr_tools::account::SlotPolicy { max_characters: 1 },
        )
        .unwrap();
    assert_eq!(store.account_role(target), Some(AccountRole::Player));

    let denied = store.set_account_role(target, target, AccountRole::Admin);
    assert_eq!(
        denied.unwrap_err().kind(),
        std::io::ErrorKind::PermissionDenied
    );
    assert_eq!(store.account_role(target), Some(AccountRole::Player));

    store
        .set_account_role(1, target, AccountRole::Builder)
        .unwrap();
    store.save(&path).unwrap();
    let mut reloaded = AccountStore::load(&path).unwrap();
    assert_eq!(reloaded.account_role(1), Some(AccountRole::Admin));
    assert_eq!(reloaded.account_role(target), Some(AccountRole::Builder));
    assert_eq!(
        reloaded
            .set_account_role(1, u64::MAX, AccountRole::Admin)
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::NotFound
    );
}

#[test]
fn role_names_are_exact_and_closed() {
    assert_eq!(AccountRole::parse("player"), Some(AccountRole::Player));
    assert_eq!(AccountRole::parse("builder"), Some(AccountRole::Builder));
    assert_eq!(AccountRole::parse("admin"), Some(AccountRole::Admin));
    assert_eq!(AccountRole::parse("Admin"), None);
    assert_eq!(AccountRole::parse("owner"), None);
}
