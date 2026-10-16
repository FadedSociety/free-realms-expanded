//! (R) Host-only account provisioning. Stop login before use until the lead
//! integrates AccountStoreLock into its single authoritative store owner.
//! Public keys are 64 hex characters; private keys are never accepted here.

use osfr_tools::{
    account::{AccountFlags, SlotPolicy},
    account_store::{AccountRole, AccountStore, AccountStoreLock},
    login_auth::{login_handle_hash, validate_public_key},
};
use std::{collections::BTreeMap, error::Error, io, path::PathBuf};

const USAGE: &str = "osfr-account <command> --store <accounts.json>
  create     --handle <3-32 ASCII bytes> --public-key <64 hex characters>
  add-key    --account-id <id> --public-key <64 hex characters>
  remove-key --account-id <id> --public-key <64 hex characters>
  set-role   --account-id <id> --role <player|builder|admin>
  list
Host-only; stop login first. Never supply a private key.";

fn main() {
    if let Err(error) = run(std::env::args().skip(1)) {
        eprintln!("osfr-account: {error}");
        std::process::exit(1);
    }
}

fn public_key(hex: &str) -> Result<[u8; 32], Box<dyn Error>> {
    if hex.len() != 64 {
        return Err("public key must contain 64 hex characters".into());
    }
    let nibble = |b| match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    };
    let mut key = [0; 32];
    // Decode bytes, never slice a potentially non-ASCII string at byte offsets.
    for (pair, output) in hex.as_bytes().chunks_exact(2).zip(&mut key) {
        *output = (nibble(pair[0]).ok_or("invalid public key hex")? << 4)
            | nibble(pair[1]).ok_or("invalid public key hex")?;
    }
    Ok(validate_public_key(&key)?)
}

fn run(mut args: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let command = args.next().ok_or(USAGE)?;
    if command == "--help" && args.next().is_none() {
        println!("{USAGE}");
        return Ok(());
    }
    let expected: &[&str] = match command.as_str() {
        "create" => &["--store", "--handle", "--public-key"],
        "add-key" | "remove-key" => &["--store", "--account-id", "--public-key"],
        "set-role" => &["--store", "--account-id", "--role"],
        "list" => &["--store"],
        _ => return Err(USAGE.into()),
    };
    let mut options = BTreeMap::new();
    while let Some(flag) = args.next() {
        if !expected.contains(&flag.as_str()) || options.contains_key(&flag) {
            return Err("unknown or duplicate option".into());
        }
        options.insert(flag, args.next().ok_or("missing option value")?);
    }
    if options.len() != expected.len() {
        return Err("missing required option".into());
    }
    let get = |name: &str| {
        options
            .get(name)
            .map(String::as_str)
            .ok_or("missing required option")
    };
    let path = PathBuf::from(get("--store")?);
    if path.file_name().is_none() {
        return Err("store path must name a file".into());
    }
    let key = options
        .get("--public-key")
        .map(|hex| public_key(hex))
        .transpose()?;
    let id = options
        .get("--account-id")
        .map(|id| id.parse::<u64>())
        .transpose()?;
    let hash = options
        .get("--handle")
        .map(|handle| login_handle_hash(handle))
        .transpose()?;
    let role = options
        .get("--role")
        .map(|role| AccountRole::parse(role).ok_or("invalid role"))
        .transpose()?;
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    // This is cooperative serialization, not a filesystem ACL or remote role.
    let _lock = AccountStoreLock::acquire(&path)?;
    let mut store = match AccountStore::load(&path) {
        Ok(store) => store,
        Err(error) if command == "create" && error.kind() == io::ErrorKind::NotFound => {
            AccountStore::new()
        }
        Err(error) => return Err(error.into()),
    };
    match command.as_str() {
        "create" => {
            // (R) Operator-assisted ten-friend playtest; 16 accounts, 6 slots.
            if store.account_ids().count() >= 16 {
                return Err("account limit reached".into());
            }
            let id = store.create_account(
                hash.ok_or("handle required")?,
                AccountFlags::default(),
                SlotPolicy { max_characters: 6 },
            )?;
            store.add_public_key(id, &key.ok_or("public key required")?)?;
            store.save(&path)?;
            println!("created account {id}");
        }
        "add-key" | "remove-key" => {
            let id = id.ok_or("account id required")?;
            let key = key.ok_or("public key required")?;
            if command == "add-key" {
                store.add_public_key(id, &key)?;
            } else {
                store.remove_public_key(id, &key)?;
            }
            store.save(&path)?;
            println!("updated account {id}");
        }
        "set-role" => {
            let id = id.ok_or("account id required")?;
            store.set_account_role_offline(id, role.ok_or("role required")?)?;
            store.save(&path)?;
            println!("updated account {id}");
        }
        "list" => {
            for id in store.account_ids() {
                let role = store.account_role(id).ok_or("missing account role")?;
                let keys = store.public_keys(id).ok_or("missing public keys")?;
                println!("{id}\t{role:?}\t{} keys", keys.len());
            }
        }
        _ => return Err(USAGE.into()),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;

    fn hex_key(seed: u8) -> String {
        SigningKey::from_bytes(&[seed; 32])
            .verifying_key()
            .to_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    #[test]
    fn offline_create_rotate_set_role_list_and_disable() {
        let dir = std::env::temp_dir().join(format!("osfr-w83-cli-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("accounts.json");
        let path_text = path.to_str().unwrap();
        let invoke = |args: &[&str]| run(args.iter().map(|s| (*s).to_owned()));
        let first = hex_key(17);
        let second = hex_key(18);
        invoke(&[
            "create",
            "--store",
            path_text,
            "--handle",
            "acct_003",
            "--public-key",
            &first,
        ])
        .unwrap();
        invoke(&[
            "add-key",
            "--store",
            path_text,
            "--account-id",
            "1",
            "--public-key",
            &second,
        ])
        .unwrap();
        invoke(&[
            "set-role",
            "--store",
            path_text,
            "--account-id",
            "1",
            "--role",
            "admin",
        ])
        .unwrap();
        invoke(&["list", "--store", path_text]).unwrap();
        let store = AccountStore::load(&path).unwrap();
        assert_eq!(store.account_role(1), Some(AccountRole::Admin));
        assert_eq!(store.public_keys(1).unwrap().len(), 2);
        for key in [&first, &second] {
            invoke(&[
                "remove-key",
                "--store",
                path_text,
                "--account-id",
                "1",
                "--public-key",
                key,
            ])
            .unwrap();
        }
        assert!(
            AccountStore::load(&path)
                .unwrap()
                .public_keys(1)
                .unwrap()
                .is_empty()
        );
        let before = std::fs::read(&path).unwrap();
        let lock = AccountStoreLock::acquire(&path).unwrap();
        assert!(
            invoke(&[
                "set-role",
                "--store",
                path_text,
                "--account-id",
                "1",
                "--role",
                "player"
            ])
            .is_err()
        );
        assert_eq!(std::fs::read(&path).unwrap(), before);
        drop(lock);
        assert!(
            invoke(&[
                "set-role",
                "--store",
                path_text,
                "--account-id",
                "2",
                "--role",
                "admin"
            ])
            .is_err()
        );
        assert_eq!(std::fs::read(&path).unwrap(), before);
        // Malformed stores never become fresh accounts, even on create.
        std::fs::write(&path, b"{").unwrap();
        assert!(
            invoke(&[
                "create",
                "--store",
                path_text,
                "--handle",
                "acct_004",
                "--public-key",
                &first
            ])
            .is_err()
        );
        assert_eq!(std::fs::read(&path).unwrap(), b"{");
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }

    #[test]
    fn malformed_public_keys_are_rejected_without_unicode_slicing() {
        for value in [
            "".to_owned(),
            "0".repeat(64),
            "é".repeat(32),
            "z".repeat(64),
            "1".repeat(63),
        ] {
            assert!(public_key(&value).is_err());
        }
    }
}
