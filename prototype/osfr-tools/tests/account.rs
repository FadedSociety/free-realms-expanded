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
        check_name(&store, 1, "SampleA", "VeryLongerLastName").status,
        NameCheckStatus::LastTooLong
    );
    assert_eq!(
        check_name(&store, 1, "Bad!", "Record").status,
        NameCheckStatus::Invalid
    );
    // Latin letters with accents are allowed (OSFR parity); other scripts, look-alikes, combining
    // marks, Latin-range symbols and C1 controls are refused for new names.
    for (first, last) in [("José", "Núñez"), ("Zoë", "Ångström"), ("Łukasz", "Dvořák")] {
        assert_eq!(
            check_name(&store, 1, first, last).status,
            NameCheckStatus::Available,
            "{first} {last}"
        );
    }
    for (first, last) in [
        ("Jоse", "Record"),         // Cyrillic о
        ("名前", "Record"),         // CJK renders invisibly in the client
        ("Jose\u{0301}", "Record"), // combining acute instead of é
        ("Jo×e", "Record"),         // U+00D7 sits inside the Latin range
        ("Jo\u{0085}e", "Record"),  // C1 control
        ("Jo\u{00A0}e", "Record"),  // no-break space
    ] {
        assert_eq!(
            check_name(&store, 1, first, last).status,
            NameCheckStatus::Invalid,
            "{first:?}"
        );
    }
    assert_eq!(
        create_character(
            &mut store,
            account_id,
            creation("Jоse", "Record"),
            &resources
        )
        .status,
        CharacterCreateStatus::InvalidName
    );
    assert_eq!(NameCheckStatus::Profane.client_code(), 3);
    assert_eq!(NameCheckStatus::TotalLengthInvalid.client_code(), 5);
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

#[test]
fn every_new_name_part_is_letters_joined_by_single_separators() {
    let temp = TempDir::new();
    let resources = resources(&temp.0.join("creator"));
    let mut store = AccountStore::new();
    let account_id = account(&mut store, 7, 4);
    for (first, last) in [
        ("Éva", "O'Vale"),
        ("Zoë", "Núñez"),
        ("Łukasz", "Dvořák"),
        ("Ana-Lu", "De Vale"),
    ] {
        assert_eq!(
            check_name(&store, 1, first, last).status,
            NameCheckStatus::Available,
            "{first} {last}"
        );
    }
    for part in ["--", "'-", "-ab", "ab-", "a--b", "a  b", "a -b", "a'-b"] {
        for (first, last) in [(part, "Record"), ("Eva", part)] {
            assert_eq!(
                check_name(&store, 1, first, last).status,
                NameCheckStatus::Invalid,
                "{first:?} {last:?}"
            );
        }
    }
    // Refusals change nothing, for the displayed and the temporary pair alike.
    let before = store.clone();
    for refused in [creation("-ab", "TempA"), creation("SampleA", "a--b")] {
        assert_eq!(
            create_character(&mut store, account_id, refused, &resources).status,
            CharacterCreateStatus::InvalidName
        );
    }
    assert_eq!(store, before);
}

// L058: the creator's picker joins a first name and a last-name prefix and suffix from the client's
// CharacterCreateNames.txt (longest rows 11, 8 and 8) and sends the pick as the temporary pair
// (client 0x0070cfa0). A native run was refused "Jessica Grittycandle" by a 15-letter total cap.
#[test]
fn the_longest_names_the_creator_picker_offers_pass_check_name_and_create() {
    let temp = TempDir::new();
    let resources = resources(&temp.0.join("creator"));
    let mut store = AccountStore::new();
    let account_id = account(&mut store, 9, 8);
    let picks = [
        ("Christopher", "Imperialslippers"), // longest first + longest last: 27
        ("Alexzandria", "Majesticcauldron"),
        ("Jessica", "Grittycandle"),
        ("Ari", "Oxarm"), // shortest: 9
    ];
    for (first, last) in picks {
        assert_eq!(
            check_name(&store, 1, first, last).status,
            NameCheckStatus::Available,
            "{first} {last}"
        );
        let picked = NativeCreation {
            first_name: String::new(),
            last_name: String::new(),
            temporary_first_name: first.into(),
            temporary_last_name: last.into(),
            ..creation("", "")
        };
        let created = create_character(&mut store, account_id, picked, &resources);
        assert_eq!(created.status, CharacterCreateStatus::Created, "{first}");
        assert_eq!(
            check_name(&store, 1, first, last).status,
            NameCheckStatus::Taken
        );
    }
    // A typed (displayed) pair of the same lengths is created too.
    let typed = NativeCreation {
        last_name: "Sapphirefeathers".into(),
        ..creation("Christophera", "TempTyped")
    };
    let created = create_character(&mut store, account_id, typed, &resources);
    assert_eq!(created.status, CharacterCreateStatus::Created);
}

#[test]
fn a_name_part_over_the_limit_is_refused_and_changes_nothing() {
    let temp = TempDir::new();
    let resources = resources(&temp.0.join("creator"));
    let mut store = AccountStore::new();
    let account_id = account(&mut store, 10, 8);
    let (first_14, first_15) = ("Christopherann", "Christopheranne");
    let (last_16, last_17) = ("Imperialslippers", "Imperialslipperss");
    assert_eq!(
        check_name(&store, 1, first_14, last_16).status,
        NameCheckStatus::Available
    );
    assert_eq!(
        check_name(&store, 1, first_15, "Record").status,
        NameCheckStatus::FirstTooLong
    );
    assert_eq!(
        check_name(&store, 1, "Jessica", last_17).status,
        NameCheckStatus::LastTooLong
    );
    // Characters, not bytes: 16 two-byte letters are a valid last name, 17 are not.
    let (accented_16, accented_17) = ("é".repeat(16), "é".repeat(17));
    assert_eq!(
        check_name(&store, 1, "Jessica", &accented_16).status,
        NameCheckStatus::Available
    );
    assert_eq!(
        check_name(&store, 1, "Jessica", &accented_17).status,
        NameCheckStatus::LastTooLong
    );

    let before = store.clone();
    let refused = [
        creation(first_15, "TempA"),
        NativeCreation {
            last_name: last_17.into(),
            ..creation("Jessica", "TempB")
        },
        creation("SampleC", first_15),
        NativeCreation {
            temporary_last_name: last_17.into(),
            ..creation("SampleD", "TempD")
        },
    ];
    for over in refused {
        assert_eq!(
            create_character(&mut store, account_id, over, &resources).status,
            CharacterCreateStatus::InvalidName
        );
    }
    assert_eq!(store, before);
}

#[test]
fn accented_and_cased_look_alikes_of_a_saved_name_are_taken() {
    let temp = TempDir::new();
    let resources = resources(&temp.0.join("creator"));
    let mut store = AccountStore::new();
    let account_id = account(&mut store, 8, 12);
    let saved = [
        "Eva", "Kyle", "Lukasz", "Dorde", "Soren", "Weiss", "Aesa", "Oella", "Thora", "Ilka",
    ];
    for first in saved {
        let created = create_character(&mut store, account_id, creation(first, ""), &resources);
        assert_eq!(created.status, CharacterCreateStatus::Created, "{first}");
    }
    let accented = NativeCreation {
        last_name: "Núñez".into(),
        ..creation("Zoë", "")
    };
    let created = create_character(&mut store, account_id, accented, &resources);
    assert_eq!(created.status, CharacterCreateStatus::Created);

    for (first, last) in [
        ("Éva", "Record"),
        ("EVA", "RECORD"),
        ("Kýle", "Record"),
        ("Łukasz", "Record"),
        ("Đorđe", "Record"),
        ("Søren", "Record"),
        ("Weiß", "Record"),
        ("Æsa", "Record"),
        ("Œlla", "Record"),
        ("Þora", "Record"),
        ("İlka", "Record"),
        ("ılka", "Record"),
        ("Zoe", "Nunez"),
        ("ZOE", "NÚÑEZ"),
    ] {
        assert_eq!(
            check_name(&store, 1, first, last).status,
            NameCheckStatus::Taken,
            "{first} {last}"
        );
    }
    assert_eq!(
        check_name(&store, 1, "Evan", "Record").status,
        NameCheckStatus::Available
    );

    let before = store.clone();
    assert_eq!(
        create_character(&mut store, account_id, creation("Éva", ""), &resources).status,
        CharacterCreateStatus::NameTaken
    );
    let temporary_look_alike = NativeCreation {
        temporary_last_name: "Record".into(),
        ..creation("Fresh", "Évà")
    };
    assert_eq!(
        create_character(&mut store, account_id, temporary_look_alike, &resources).status,
        CharacterCreateStatus::TemporaryNameTaken
    );
    assert_eq!(store, before);
}

#[test]
fn saves_with_now_refused_or_colliding_names_still_load_and_play() {
    let temp = TempDir::new();
    let resources = resources(&temp.0.join("creator"));
    let mut store = AccountStore::new();
    let account_id = account(&mut store, 9, 4);
    let guids = ["Eva", "Evb", "Evc"].map(|first| {
        create_character(&mut store, account_id, creation(first, ""), &resources).guid
    });
    let path = temp.0.join("accounts.json");
    store.save(&path).unwrap();
    // Names an older server accepted: one folds onto 'Eva', one has no letters.
    let older = fs::read_to_string(&path)
        .unwrap()
        .replace("\"Evb\"", "\"Éva\"")
        .replace("\"Evc\"", "\"--\"");
    fs::write(&path, older).unwrap();

    let legacy = AccountStore::load(&path).unwrap();
    assert_eq!(
        legacy.character(guids[1]).unwrap().identity.first_name,
        "Éva"
    );
    assert_eq!(
        legacy.character(guids[2]).unwrap().identity.first_name,
        "--"
    );
    assert_eq!(character_list(&legacy, account_id, 1).characters.len(), 3);
    for guid in guids {
        let mut session = AuthenticatedSession::new(account_id);
        let selected = select_and_bind_session(
            &legacy,
            &mut session,
            guid,
            1,
            "127.0.0.1:20260",
            "synthetic-ticket",
        );
        assert_eq!(selected.status, CharacterLoginStatus::Success);
    }
    assert_eq!(
        check_name(&legacy, 1, "EVA", "Record").status,
        NameCheckStatus::Taken
    );
    legacy.save(&path).unwrap();
    assert_eq!(AccountStore::load(&path).unwrap(), legacy);
}
