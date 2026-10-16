use osfr_tools::account::{
    AccountFlags, AuthenticatedSession, CharacterCreateStatus, CharacterDeleteStatus,
    CharacterLoginStatus, NameCheckStatus, SlotPolicy, character_list, check_name,
    create_character, delete_character, select_and_bind_session,
};
use osfr_tools::account_store::AccountStore;
use osfr_tools::identity::{NativeCreation, Resources};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn read_u32(bytes: &[u8], offset: &mut usize) -> u32 {
    let value = u32::from_le_bytes(bytes[*offset..*offset + 4].try_into().unwrap());
    *offset += 4;
    value
}

fn read_string(bytes: &[u8], offset: &mut usize) -> String {
    let length = read_u32(bytes, offset) as usize;
    let value = String::from_utf8(bytes[*offset..*offset + length].to_vec()).unwrap();
    *offset += length;
    value
}

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-account-test-{}-{}",
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

fn resources(directory: &Path) -> Resources {
    let create = directory.join("CharacterCreate");
    fs::create_dir_all(&create).unwrap();
    let mut model = vec!["0"; 17];
    model[0] = "1";
    model[5] = "1";
    model[13] = "1";
    fs::write(directory.join("Models.txt"), model.join("^")).unwrap();
    for (name, row) in [
        ("HeadMappings.txt", "1^head.adr^"),
        ("HairMappings.txt", "2^hair.adr^"),
        ("ModelCustomizationMappings.txt", "1^beard.adr^"),
        ("FacePaintMappings.txt", "0^none^"),
        ("SkinToneMappings.txt", "1^skin.adr^"),
    ] {
        fs::write(create.join(name), row).unwrap();
    }
    let mut definitions = String::new();
    for (id, slot) in [(201, 5), (189, 4), (199, 3)] {
        let mut row = vec!["0".to_owned(); 42];
        row[0] = id.to_string();
        row[12] = slot.to_string();
        row[16] = "item.adr".into();
        row[17] = "1".into();
        row[18] = "cloth".into();
        row[19] = "0".into();
        row[41] = "tint".into();
        definitions.push_str(&row.join("^"));
        definitions.push('\n');
    }
    fs::write(create.join("ItemDefinitions.txt"), definitions).unwrap();
    Resources::load(directory).unwrap()
}

fn creation(first: &str, temporary: &str) -> NativeCreation {
    let mut choices = [0; 21];
    choices[0] = 1;
    choices[1] = 1;
    choices[2] = 2;
    choices[5] = 1;
    choices[7] = 201;
    choices[8] = 189;
    choices[9] = 199;
    choices[20] = 1;
    NativeCreation {
        first_name: first.into(),
        last_name: "Record".into(),
        temporary_first_name: temporary.into(),
        temporary_last_name: "TempRecord".into(),
        choices,
    }
}

fn account(store: &mut AccountStore, hash: u8, slots: u32) -> u64 {
    store
        .create_account(
            [hash; 32],
            AccountFlags::default(),
            SlotPolicy {
                max_characters: slots,
            },
        )
        .unwrap()
}

#[test]
fn one_account_owns_two_characters_and_reload_keeps_both() {
    let temp = TempDir::new();
    let resources = resources(&temp.0.join("creator"));
    let mut store = AccountStore::new();
    let account_id = store
        .create_account(
            [1; 32],
            AccountFlags {
                membership: true,
                underage: true,
                open_chat: false,
            },
            SlotPolicy { max_characters: 2 },
        )
        .unwrap();
    assert_eq!(store.account_id_by_login_hash(&[1; 32]), Some(account_id));

    let first = create_character(
        &mut store,
        account_id,
        creation("SampleA", "TempA"),
        &resources,
    );
    let second = create_character(
        &mut store,
        account_id,
        creation("SampleB", "TempB"),
        &resources,
    );
    assert_eq!(first.status, CharacterCreateStatus::Created);
    assert_eq!(second.status, CharacterCreateStatus::Created);
    assert_ne!(first.guid, second.guid);
    assert_eq!(first.message_bytes[0], 6);
    assert_eq!(
        i32::from_le_bytes(first.message_bytes[1..5].try_into().unwrap()),
        1
    );
    assert_eq!(
        u64::from_le_bytes(first.message_bytes[5..13].try_into().unwrap()),
        first.guid
    );
    assert_eq!(
        create_character(
            &mut store,
            account_id,
            creation("SampleC", "TempC"),
            &resources
        )
        .status,
        CharacterCreateStatus::TooManyCharacters
    );

    let path = temp.0.join("accounts.json");
    store.save(&path).unwrap();
    let mut reloaded = AccountStore::load(&path).unwrap();
    let flags = &reloaded.account(account_id).unwrap().flags;
    assert!(flags.membership);
    assert!(flags.underage);
    assert!(!flags.open_chat);
    let reply = character_list(&reloaded, account_id, 1);
    assert_eq!(
        reply
            .characters
            .iter()
            .map(|row| row.guid)
            .collect::<Vec<_>>(),
        [first.guid, second.guid]
    );
    assert_eq!(
        i32::from_le_bytes(reply.message_bytes[6..10].try_into().unwrap()),
        2
    );
    assert_eq!(reply.message_bytes[0], 12);
    assert!(reply.characters.iter().all(|row| row.status == 1));
    assert!(
        reply
            .characters
            .iter()
            .all(|row| !row.payload_base64.is_empty())
    );
    let mut offset = 10;
    for row in &reply.characters {
        assert_eq!(
            u64::from_le_bytes(reply.message_bytes[offset..offset + 8].try_into().unwrap()),
            row.guid
        );
        offset += 8;
        assert_eq!(
            i32::from_le_bytes(reply.message_bytes[offset..offset + 4].try_into().unwrap()),
            row.server_id
        );
        offset += 4;
        assert_eq!(
            i32::from_le_bytes(reply.message_bytes[offset..offset + 4].try_into().unwrap()),
            row.status
        );
        offset += 4;
        assert_eq!(
            read_string(&reply.message_bytes, &mut offset),
            row.payload_base64
        );
        assert_eq!(row.payload_base64.len() % 4, 0);
        assert!(
            row.payload_base64
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"+/=".contains(&byte))
        );
    }
    assert_eq!(offset, reply.message_bytes.len());
    assert_eq!(
        reloaded
            .create_account(
                [9; 32],
                AccountFlags::default(),
                SlotPolicy { max_characters: 1 }
            )
            .unwrap(),
        2
    );
}

#[test]
fn name_check_and_create_return_client_status_codes() {
    let temp = TempDir::new();
    let resources = resources(&temp.0.join("creator"));
    let mut store = AccountStore::new();
    let account_id = account(&mut store, 2, 4);
    let created = create_character(
        &mut store,
        account_id,
        creation("SampleA", "TempA"),
        &resources,
    );
    assert_eq!(created.status, CharacterCreateStatus::Created);

    assert_eq!(
        check_name(&store, 1, "SampleA", "Record").status,
        NameCheckStatus::Taken
    );
    assert_eq!(
        check_name(&store, 1, "A", "Record").status,
        NameCheckStatus::FirstTooShort
    );
    assert_eq!(
        check_name(&store, 1, "SampleA", "B").status,
        NameCheckStatus::LastTooShort
    );
    assert_eq!(
        check_name(&store, 1, "LongFirstNameABCDE", "Record").status,
        NameCheckStatus::FirstTooLong
    );
    assert_eq!(
        check_name(&store, 1, "SampleA", "VeryLongLastName").status,
        NameCheckStatus::LastTooLong
    );
    assert_eq!(
        check_name(&store, 1, "LongNamePart", "LastPart").status,
        NameCheckStatus::TotalLengthInvalid
    );
    assert_eq!(
        check_name(&store, 1, "Bad!", "Record").status,
        NameCheckStatus::Invalid
    );
    assert_eq!(NameCheckStatus::Profane.client_code(), 3);
    assert_eq!(NameCheckStatus::ServerFailure.client_code(), 6);
    assert_eq!(
        create_character(
            &mut store,
            u64::MAX,
            creation("SampleZ", "TempZ"),
            &resources
        )
        .status,
        CharacterCreateStatus::ServerFailure
    );
    assert_eq!(
        create_character(
            &mut store,
            account_id,
            creation("SampleA", "OtherTemp"),
            &resources
        )
        .status,
        CharacterCreateStatus::NameTaken
    );
    assert_eq!(
        create_character(
            &mut store,
            account_id,
            NativeCreation {
                temporary_first_name: "TempA".into(),
                temporary_last_name: "TempRecord".into(),
                ..creation("Other", "TempA")
            },
            &resources
        )
        .status,
        CharacterCreateStatus::TemporaryNameTaken
    );
    assert_eq!(
        create_character(
            &mut store,
            account_id,
            creation("Bad!", "TempBad"),
            &resources
        )
        .status,
        CharacterCreateStatus::InvalidName
    );

    let response = check_name(&store, 1, "FreeName", "Record");
    assert_eq!(response.status, NameCheckStatus::Available);
    assert_eq!(response.message_bytes[0], 17);
    let mut packet_offset = 5;
    let length = read_u32(&response.message_bytes, &mut packet_offset) as usize;
    let packet = &response.message_bytes[9..9 + length];
    assert_eq!(&packet[..2], &211i16.to_le_bytes());
    let mut offset = 2;
    assert_eq!(read_string(packet, &mut offset), "FreeName");
    assert_eq!(read_string(packet, &mut offset), "Record");
    assert_eq!(
        i32::from_le_bytes(packet[offset..offset + 4].try_into().unwrap()),
        1
    );
    offset += 4;
    assert_eq!(offset, packet.len());
}

#[test]
fn session_cannot_bind_another_accounts_character_and_delete_checks_owner() {
    let temp = TempDir::new();
    let resources = resources(&temp.0.join("creator"));
    let mut store = AccountStore::new();
    let owner_a = account(&mut store, 3, 2);
    let owner_b = account(&mut store, 4, 2);
    let owned = create_character(
        &mut store,
        owner_a,
        creation("SampleA", "TempA"),
        &resources,
    );
    let foreign = create_character(
        &mut store,
        owner_b,
        creation("SampleB", "TempB"),
        &resources,
    );
    assert_eq!(owned.status, CharacterCreateStatus::Created);
    assert_eq!(foreign.status, CharacterCreateStatus::Created);

    let mut session = AuthenticatedSession::new(owner_a);
    let rejected = select_and_bind_session(
        &store,
        &mut session,
        foreign.guid,
        1,
        "127.0.0.1:20260",
        "synthetic-ticket",
    );
    assert_eq!(rejected.status, CharacterLoginStatus::UnableToLoadCharacter);
    assert_eq!(session.selected_character(), None);
    assert_eq!(session.account_id(), owner_a);
    assert_eq!(CharacterLoginStatus::AccountInUse.client_code(), 2);
    assert_eq!(CharacterLoginStatus::UnableToAssignGateway.client_code(), 7);

    let selected = select_and_bind_session(
        &store,
        &mut session,
        owned.guid,
        1,
        "127.0.0.1:20260",
        "synthetic-ticket",
    );
    assert_eq!(selected.status, CharacterLoginStatus::Success);
    assert_eq!(session.selected_character(), Some(owned.guid));
    assert_eq!(
        i32::from_le_bytes(selected.message_bytes[13..17].try_into().unwrap()),
        1
    );
    let rejected_delete = delete_character(&mut store, owner_a, foreign.guid);
    assert_eq!(
        rejected_delete.status,
        CharacterDeleteStatus::NotOwnedOrMissing
    );
    assert_eq!(rejected_delete.message_bytes[0], 10);
    let deleted = delete_character(&mut store, owner_a, owned.guid);
    assert_eq!(deleted.status, CharacterDeleteStatus::Deleted);
    assert_eq!(
        u64::from_le_bytes(deleted.message_bytes[1..9].try_into().unwrap()),
        owned.guid
    );
    assert_eq!(
        i32::from_le_bytes(deleted.message_bytes[9..13].try_into().unwrap()),
        1
    );
    assert!(store.character(owned.guid).is_none());
    let next = create_character(
        &mut store,
        owner_a,
        creation("SampleC", "TempC"),
        &resources,
    );
    assert_eq!(next.status, CharacterCreateStatus::Created);
    assert!(next.guid > foreign.guid);
}

#[test]
fn migration_adopts_a_legacy_character_save_without_rewriting_it() {
    let temp = TempDir::new();
    let legacy_save = temp.0.join("legacy-character.json");
    let legacy_identity = temp.0.join("legacy-identity.json");
    fs::write(
        &legacy_save,
        include_str!("fixtures/account/legacy-character-save-v2.json"),
    )
    .unwrap();
    fs::write(
        &legacy_identity,
        include_str!("fixtures/account/legacy-identity-v1.json"),
    )
    .unwrap();
    let before = fs::read(&legacy_save).unwrap();
    let path = temp.0.join("accounts.json");
    let mut migrated = AccountStore::load_or_migrate(
        &path,
        &legacy_save,
        &legacy_identity,
        [5; 32],
        AccountFlags::default(),
        SlotPolicy { max_characters: 2 },
    )
    .unwrap();
    assert_eq!(fs::read(&legacy_save).unwrap(), before);
    let account = migrated.account(1).unwrap();
    assert_eq!(account.character_guids, [900_000_001]);
    assert_eq!(
        migrated
            .character(900_000_001)
            .unwrap()
            .world_state
            .as_ref()
            .unwrap()
            .guid,
        900_000_001
    );
    let resources = resources(&temp.0.join("creator"));
    let next_character =
        create_character(&mut migrated, 1, creation("SampleA", "TempA"), &resources);
    assert_eq!(next_character.status, CharacterCreateStatus::Created);
    assert_eq!(next_character.guid, 900_000_002);
    let next = migrated
        .create_account(
            [6; 32],
            AccountFlags::default(),
            SlotPolicy { max_characters: 1 },
        )
        .unwrap();
    assert_eq!(next, 2);
    migrated.save(&path).unwrap();
    assert_eq!(
        AccountStore::load(&path)
            .unwrap()
            .account(1)
            .unwrap()
            .character_guids,
        [900_000_001, 900_000_002]
    );
}
