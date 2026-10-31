use osfr_tools::{
    account::{self, AccountFlags, SlotPolicy},
    account_store::{AccountStore, AccountStoreLock},
    character_store::CharacterState,
    identity::{self, Attachment, CharacterIdentity, NativeCreation, Resources},
    login::{self, LoginConfig, LoginService, Rc4},
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

fn hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|x| u8::from_str_radix(std::str::from_utf8(x).unwrap(), 16).unwrap())
        .collect()
}
fn vectors() -> serde_json::Value {
    serde_json::from_str(include_str!("fixtures/login-community-vectors.json")).unwrap()
}
fn vector(name: &str) -> Vec<u8> {
    hex(vectors()["vectors"][name].as_str().unwrap())
}
fn string(out: &mut Vec<u8>, s: &str) {
    out.extend((s.len() as u32).to_le_bytes());
    out.extend(s.as_bytes());
}
fn payload(out: &mut Vec<u8>, b: &[u8]) {
    out.extend((b.len() as u32).to_le_bytes());
    out.extend(b);
}
fn expected_character_list(identity: &CharacterIdentity) -> Vec<u8> {
    // A one-character list carries the same appearance payload that the
    // native character select accepted (run 023623); see `selection_reply`.
    login::selection_reply(Some(identity))
}
fn login_reply_with_max_characters(member: bool, max_characters: i32) -> Vec<u8> {
    let mut out = vec![2, 1];
    out.extend(1i32.to_le_bytes());
    out.push(u8::from(member));
    let mut account = vec![u8::from(member)];
    account.extend(max_characters.to_le_bytes());
    account.push(0);
    payload(&mut out, &account);
    out
}
fn sample() -> CharacterIdentity {
    CharacterIdentity {
        guid: 1,
        first_name: "Ari".into(),
        last_name: "Vale".into(),
        model: 1,
        head: "head.adr".into(),
        hair: "hair.adr".into(),
        model_customization: "".into(),
        face_paint: "none".into(),
        skin_tone: "skintone1".into(),
        eye_color: 262,
        hair_color: 116,
        gender: 1,
        creation: NativeCreation::decode(&vector("creation_payload")).unwrap(),
        attachments: vec![Attachment {
            model_name: "shirt.adr".into(),
            texture_alias: "cloth".into(),
            tint_alias: "dyetint".into(),
            tint_id: 241,
            composite_effect_id: 0,
            slot: 3,
            definition: 199,
        }],
    }
}

#[test]
fn packets_equal_independent_community_serializers() {
    assert_eq!(login::login_reply(true), vector("login_ok"));
    assert_eq!(login::create_reply(1, 1), vector("created"));
    assert_eq!(login::selection_reply(None), vector("selection_empty"));
    assert_eq!(
        login::selection_reply(Some(&sample())),
        vector("selection_one")
    );
    assert_eq!(
        login::character_login_reply(1, 1, 1, "127.0.0.1:20260", "synthetic-ticket"),
        vector("character_login")
    );
    assert_eq!(login::server_list_reply(), vector("servers"));
    let c = NativeCreation::decode(&vector("creation_payload")).unwrap();
    assert_eq!(c.encode(), vector("creation_payload"));
    assert_eq!(&c.choices[7..10], &[201, 189, 199]);
}

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "osfr-login-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn resources(t: &Temp) -> Resources {
    fs::create_dir_all(t.0.join("CharacterCreate")).unwrap();
    let mut model = vec!["0"; 17];
    model[0] = "1";
    model[5] = "1";
    model[13] = "1";
    fs::write(t.0.join("Models.txt"), model.join("^")).unwrap();
    for (name, data) in [
        ("HeadMappings.txt", "1^head.adr^"),
        ("HairMappings.txt", "2^hair.adr^"),
        ("ModelCustomizationMappings.txt", "1^beard.adr^"),
        ("FacePaintMappings.txt", "0^none^"),
        ("SkinToneMappings.txt", "1^skintone1^"),
    ] {
        fs::write(t.0.join("CharacterCreate").join(name), data).unwrap();
    }
    let mut lines = String::new();
    for (id, slot) in [(201, 5), (189, 4), (199, 3)] {
        let mut row = vec!["0".to_owned(); 51];
        row[0] = id.to_string();
        row[12] = slot.to_string();
        row[16] = "shirt.adr".into();
        row[17] = "1".into();
        row[18] = "cloth".into();
        row[19] = "241".into();
        row[41] = "dyetint".into();
        lines.push_str(&row.join("^"));
        lines.push('\n');
    }
    fs::write(t.0.join("CharacterCreate/ItemDefinitions.txt"), lines).unwrap();
    Resources::load(&t.0).unwrap()
}
fn service(t: &Temp) -> LoginService {
    service_as(t, "synthetic-session")
}
fn service_as(t: &Temp, local_session: &str) -> LoginService {
    service_with_paths(
        t,
        t.0.join("identity.json"),
        t.0.join("state.json"),
        t.0.join("accounts.json"),
        local_session,
    )
}
fn service_with_paths(
    t: &Temp,
    identity_file: PathBuf,
    legacy_save_file: PathBuf,
    account_store_file: PathBuf,
    local_session: &str,
) -> LoginService {
    LoginService::new(
        resources(t),
        LoginConfig {
            identity_file,
            legacy_save_file,
            account_store_file,
            gateway_address: "127.0.0.1:20260".into(),
            local_session: local_session.into(),
            gateway_ticket: "synthetic-ticket".into(),
            handoff_key: b"synthetic-handoff-key".to_vec(),
            max_characters: 6,
            underage: false,
            open_chat: true,
        },
    )
    .unwrap()
}
fn login_request() -> Vec<u8> {
    login_request_as("synthetic-session")
}
fn login_request_as(local_session: &str) -> Vec<u8> {
    let mut out = vec![1];
    string(&mut out, local_session);
    string(&mut out, "synthetic-machine");
    out
}

fn creation_named(first: &str, last: &str) -> NativeCreation {
    let mut creation = NativeCreation::decode(&vector("creation_payload")).unwrap();
    creation.first_name = first.into();
    creation.last_name = last.into();
    creation.temporary_first_name = format!("{first}T");
    creation.temporary_last_name = "Temp".into();
    creation
}

fn create_request(first: &str, last: &str) -> Vec<u8> {
    let mut out = vec![5];
    out.extend(1i32.to_le_bytes());
    payload(&mut out, &creation_named(first, last).encode());
    out
}

fn select_request(guid: u64) -> Vec<u8> {
    let mut client = Vec::new();
    string(&mut client, "en_US");
    client.extend(8i32.to_le_bytes());
    client.extend([0, 0]);
    client.extend(0i64.to_le_bytes());
    let mut out = vec![7];
    out.extend(guid.to_le_bytes());
    out.extend(1i32.to_le_bytes());
    payload(&mut out, &client);
    out
}

fn name_check_request(first: &str, last: &str) -> Vec<u8> {
    let mut check = 210i16.to_le_bytes().to_vec();
    for value in [first, last, "definition", "prefix", "suffix"] {
        string(&mut check, value);
    }
    let mut out = vec![16];
    out.extend(1i32.to_le_bytes());
    payload(&mut out, &check);
    out
}

fn name_check_status(reply: &[u8]) -> i32 {
    assert_eq!(reply[0], 17);
    let mut at = 1 + 4 + 4 + 2;
    for _ in 0..2 {
        let size = u32::from_le_bytes(reply[at..at + 4].try_into().unwrap()) as usize;
        at += 4 + size;
    }
    i32::from_le_bytes(reply[at..at + 4].try_into().unwrap())
}

#[test]
fn native_creation_selection_reconnect_and_name_check_preserve_identity() {
    let t = Temp::new();
    let mut service = service(&t);
    assert!(service.handle(&[11]).is_err());
    // L057: the account login creates here (no store yet) is a member.
    assert_eq!(
        service.handle(&login_request()).unwrap().replies[0],
        login_reply_with_max_characters(true, 6)
    );
    assert_eq!(
        service.handle(&[11]).unwrap().replies[0],
        vector("selection_empty")
    );
    let mut check = 210i16.to_le_bytes().to_vec();
    for s in ["Ari", "Vale", "definition", "prefix", "suffix"] {
        string(&mut check, s);
    }
    let mut tunneled = vec![16];
    tunneled.extend(1i32.to_le_bytes());
    payload(&mut tunneled, &check);
    assert_eq!(
        service.handle(&tunneled).unwrap().replies[0],
        vector("name_ok")
    );
    let mut create = vec![5];
    create.extend(1i32.to_le_bytes());
    payload(&mut create, &vector("creation_payload"));
    let created = service.handle(&create).unwrap();
    assert!(created.created);
    assert_eq!(created.replies[0], vector("created"));
    let saved = identity::load(&t.0.join("identity.json")).unwrap().unwrap();
    assert_eq!(saved.first_name, "Ari");
    assert_eq!(saved.attachments.len(), 3);
    assert_eq!(
        service.handle(&create).unwrap().replies[0],
        login::create_reply(4, 0)
    );
    service.reset_connection();
    assert!(service.handle(&[11]).is_err());
    service.handle(&login_request()).unwrap();
    assert_eq!(
        service.handle(&[11]).unwrap().replies[0],
        expected_character_list(&saved)
    );
    let mut client = Vec::new();
    string(&mut client, "en_US");
    client.extend(8i32.to_le_bytes());
    client.extend([0, 0]);
    client.extend(0i64.to_le_bytes());
    let mut select = vec![7];
    select.extend(1u64.to_le_bytes());
    select.extend(1i32.to_le_bytes());
    payload(&mut select, &client);
    let selected = service.handle(&select).unwrap();
    assert!(selected.selected);
    // L023: 1/1 gets its HMAC binding ticket, never the public fixed ticket.
    let store = AccountStore::load(&t.0.join("accounts.json")).unwrap();
    let ticket =
        login::gateway_binding_ticket(b"synthetic-handoff-key", store.account(1).unwrap(), 1);
    assert_eq!(
        selected.replies[0],
        login::character_login_reply(1, 1, 1, "127.0.0.1:20260", &ticket)
    );
    select[1..9].copy_from_slice(&2u64.to_le_bytes());
    assert!(!service.handle(&select).unwrap().selected);
    assert_eq!(
        identity::load(&t.0.join("identity.json")).unwrap(),
        Some(saved)
    );
}

#[test]
fn account_login_creates_lists_selects_and_deletes_two_characters() {
    let t = Temp::new();
    let mut service = service(&t);
    service.handle(&login_request()).unwrap();

    for (first, last) in [("Testa", "One"), ("Testb", "Two")] {
        let created = service.handle(&create_request(first, last)).unwrap();
        assert_eq!(created.replies[0][0], 6);
        assert_eq!(
            i32::from_le_bytes(created.replies[0][1..5].try_into().unwrap()),
            1
        );
    }

    let listed = service.handle(&[11]).unwrap().replies.remove(0);
    assert_eq!(listed[0], 12);
    assert_eq!(u32::from_le_bytes(listed[6..10].try_into().unwrap()), 2);
    let store = AccountStore::load(&t.0.join("accounts.json")).unwrap();
    assert_eq!(store.account(1).unwrap().character_guids, [1, 2]);

    for guid in [1, 2] {
        let selected = service.handle(&select_request(guid)).unwrap();
        assert!(selected.selected);
        assert_eq!(
            selected.selected_binding,
            Some(login::CharacterBinding {
                account_id: 1,
                character_guid: guid,
            })
        );
        let account = store.account(1).unwrap();
        let ticket = login::gateway_binding_ticket(b"synthetic-handoff-key", account, guid);
        assert_eq!(
            selected.replies[0],
            login::character_login_reply(guid, 1, 1, "127.0.0.1:20260", &ticket,)
        );
    }

    let mut delete = vec![9];
    delete.extend(2u64.to_le_bytes());
    let deleted = service.handle(&delete).unwrap();
    assert_eq!(deleted.replies[0][0], 10);
    assert!(!deleted.selected);
    let missing = service.handle(&select_request(2)).unwrap();
    assert!(!missing.selected);
    assert!(missing.selected_binding.is_none());
    assert_eq!(
        i32::from_le_bytes(missing.replies[0][13..17].try_into().unwrap()),
        6
    );
    let persisted = AccountStore::load(&t.0.join("accounts.json")).unwrap();
    assert_eq!(persisted.account(1).unwrap().character_guids, [1]);
}

#[test]
fn ban_blocks_login_until_expiry() {
    use osfr_tools::{
        account_store::AccountRole,
        moderation::{self, Command},
    };
    let t = Temp::new();
    let path = t.0.join("accounts.json");
    let mut first = service(&t);
    first.handle(&login_request()).unwrap();
    first.handle(&create_request("Testa", "One")).unwrap();
    // L042: a moderator account of its own; the login account stays a player.
    let mut store = AccountStore::load(&path).unwrap();
    let moderator = store
        .create_account(
            [7; 32],
            AccountFlags::default(),
            SlotPolicy { max_characters: 1 },
        )
        .unwrap();
    store
        .set_account_role_offline(moderator, AccountRole::Moderator)
        .unwrap();
    store.save(&path).unwrap();
    let run = |text: &str, now: u64| {
        moderation::run(&path, moderator, &Command::parse(text).unwrap(), now).unwrap()
    };
    run("ban testa one 30", moderation::unix_now());
    // The banned account gets the ordinary failed login and no character requests.
    let mut banned = service(&t);
    assert_eq!(
        banned.handle(&login_request()).unwrap().replies,
        [login::login_reply(false)]
    );
    assert!(banned.handle(&[11]).is_err());
    assert!(banned.handle(&select_request(1)).is_err());
    // Thirty minutes from a clock two hours behind: already over, so login works again.
    run("unban testa one", moderation::unix_now());
    run("ban testa one 30", moderation::unix_now() - 7_200);
    assert!(
        AccountStore::load(&path)
            .unwrap()
            .sanctions(1)
            .banned_until
            .is_some()
    );
    let mut expired = service(&t);
    assert_eq!(
        expired.handle(&login_request()).unwrap().replies,
        [login_reply_with_max_characters(true, 6)]
    );
    assert!(expired.handle(&select_request(1)).unwrap().selected);
}

#[test]
fn account_login_rejects_a_character_owned_by_another_account() {
    let t = Temp::new();
    let mut store = AccountStore::new();
    let first_account = store
        .create_account(
            Sha256::digest(b"synthetic-session").into(),
            AccountFlags::default(),
            SlotPolicy { max_characters: 6 },
        )
        .unwrap();
    let other_account = store
        .create_account(
            [0xa5; 32],
            AccountFlags::default(),
            SlotPolicy { max_characters: 6 },
        )
        .unwrap();
    let resources = resources(&t);
    assert_eq!(
        account::create_character(
            &mut store,
            first_account,
            creation_named("Testa", "One"),
            &resources,
        )
        .guid,
        1
    );
    assert_eq!(
        account::create_character(
            &mut store,
            other_account,
            creation_named("Testb", "Two"),
            &resources,
        )
        .guid,
        2
    );
    store.save(&t.0.join("accounts.json")).unwrap();

    let mut service = service(&t);
    service.handle(&login_request()).unwrap();
    let selected = service.handle(&select_request(2)).unwrap();
    assert!(!selected.selected);
    assert!(selected.selected_binding.is_none());
    assert_eq!(
        i32::from_le_bytes(selected.replies[0][13..17].try_into().unwrap()),
        6
    );
}

/// The (IsMember, MaxCharacters, IsAdmin) the client stores from a login reply:
/// reader 0x00fa40e0 takes IsMember from byte 6 and the payload (0x00953790) from
/// byte 7; OnLoginAuthReply 0x009542d0 then replaces the member flag with byte 6
/// and MaxCharacters with 1 + 2 * IsMember (0x00954375-0x00954398) before
/// ClientAccountManager::OnAccountInfo 0x00719330 copies them. Admin is payload byte 16.
fn client_account_info(reply: &[u8]) -> (bool, i32, bool) {
    let member = reply[6] != 0;
    (member, 1 + 2 * i32::from(member), reply[16] != 0)
}

#[test]
fn login_reply_carries_account_membership_in_the_byte_the_client_reads() {
    let t = Temp::new();
    let mut store = AccountStore::new();
    for (name, membership) in [("synthetic-session", true), ("synthetic-other", false)] {
        let flags = AccountFlags {
            membership,
            ..AccountFlags::default()
        };
        let slots = SlotPolicy { max_characters: 6 };
        store
            .create_account(Sha256::digest(name).into(), flags, slots)
            .unwrap();
    }
    store.save(&t.0.join("accounts.json")).unwrap();
    let saved = fs::read(t.0.join("accounts.json")).unwrap();

    let member = &service(&t).handle(&login_request()).unwrap().replies[0];
    assert_eq!(member, &login_reply_with_max_characters(true, 6));
    assert_eq!(client_account_info(member), (true, 3, false));

    let other = service_as(&t, "synthetic-other")
        .handle(&login_request_as("synthetic-other"))
        .unwrap();
    assert_eq!(other.replies[0], login_reply_with_max_characters(false, 6));
    assert_eq!(client_account_info(&other.replies[0]), (false, 1, false));

    // A refused login carries no account fields (no membership oracle).
    let refused = service(&t)
        .handle(&login_request_as("synthetic-other"))
        .unwrap();
    assert_eq!(refused.replies[0][1], 0);
    assert_eq!(client_account_info(&refused.replies[0]), (false, 1, false));
    assert_eq!(fs::read(t.0.join("accounts.json")).unwrap(), saved);
}

fn osfr_account(args: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_osfr-account"))
        .args(args)
        .output()
        .unwrap()
}

/// L057: both account-creating paths (`osfr-account create`, and login for a name the store
/// lacks) make members, and `osfr-account member` changes exactly one account; the next login
/// reply carries the new IsMember byte. Refusals leave the store bytes unchanged.
#[test]
fn new_accounts_are_members_and_the_operator_sets_one_accounts_membership() {
    let t = Temp::new();
    let path = t.0.join("accounts.json");
    let store = path.to_str().unwrap();
    let key: String = ed25519_dalek::SigningKey::from_bytes(&[7; 32])
        .verifying_key()
        .to_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let created = osfr_account(&[
        "create",
        "--store",
        store,
        "--handle",
        "synthetic-session",
        "--public-key",
        &key,
    ]);
    assert!(created.status.success());
    let login = |name: &str| {
        service_as(&t, name)
            .handle(&login_request_as(name))
            .unwrap()
    };
    let first = login("synthetic-session");
    assert_eq!(first.replies[0], login_reply_with_max_characters(true, 6));
    assert_eq!(client_account_info(&first.replies[0]), (true, 3, false));
    let second = login("synthetic-other");
    assert_eq!(client_account_info(&second.replies[0]), (true, 3, false));

    let member = |rest: &[&str]| osfr_account(&[&["member", "--store", store][..], rest].concat());
    let off = member(&["--handle", "synthetic-session", "--state", "off"]);
    assert!(off.status.success());
    assert_eq!(off.stdout, b"account 1 membership off\n");
    let first = login("synthetic-session");
    assert_eq!(first.replies[0], login_reply_with_max_characters(false, 6));
    assert_eq!(client_account_info(&first.replies[0]), (false, 1, false));
    let other = AccountStore::load(&path).unwrap();
    assert!(other.account(2).unwrap().flags.membership);

    let saved = fs::read(&path).unwrap();
    for refused in [
        &["--account-id", "3", "--state", "on"][..],
        &["--handle", "synthetic-unknown", "--state", "on"],
        &["--account-id", "1", "--state", "yes"],
        &["--account-id", "1", "--handle", "synthetic-session"],
        &[
            "--account-id",
            "1",
            "--handle",
            "synthetic-other",
            "--state",
            "on",
        ],
    ] {
        assert!(!member(refused).status.success(), "{refused:?}");
        assert_eq!(fs::read(&path).unwrap(), saved, "{refused:?}");
    }

    assert!(
        member(&["--account-id", "1", "--state", "on"])
            .status
            .success()
    );
    let first = login("synthetic-session");
    assert_eq!(client_account_info(&first.replies[0]), (true, 3, false));
}

#[test]
fn current_character_save_and_identity_migrate_without_rewriting_sources() {
    let t = Temp::new();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab/state");
    let save_path = root.join("opening-mmo-continuation-20261009T023433Z/character.json");
    let identity_path = root.join("opening-mmo-continuation-20261009T023433Z/identity.json");
    assert!(save_path.is_file() && identity_path.is_file());
    let save_before = fs::read(&save_path).unwrap();
    let identity_before = fs::read(&identity_path).unwrap();
    let original_identity = identity::load(&identity_path).unwrap().unwrap();
    let original_state: CharacterState = serde_json::from_value(
        serde_json::from_slice::<serde_json::Value>(&save_before).unwrap()["character"].clone(),
    )
    .unwrap();

    let mut service = service_with_paths(
        &t,
        identity_path.clone(),
        save_path.clone(),
        t.0.join("accounts.json"),
        "synthetic-session",
    );
    service.handle(&login_request()).unwrap();
    assert_eq!(
        service.handle(&[11]).unwrap().replies[0],
        expected_character_list(&original_identity)
    );
    let store = AccountStore::load(&t.0.join("accounts.json")).unwrap();
    assert_eq!(
        store.account(1).unwrap().character_guids,
        [original_state.guid]
    );
    assert!(store.account(1).unwrap().flags.membership, "L057");
    assert_eq!(
        store.character(original_state.guid).unwrap().world_state,
        Some(original_state)
    );
    assert_eq!(fs::read(&save_path).unwrap(), save_before);
    assert_eq!(fs::read(&identity_path).unwrap(), identity_before);
}

#[test]
fn account_name_check_and_create_share_global_uniqueness() {
    let t = Temp::new();
    let mut service = service(&t);
    service.handle(&login_request()).unwrap();
    let created = service.handle(&create_request("Testa", "One")).unwrap();
    assert_eq!(
        i32::from_le_bytes(created.replies[0][1..5].try_into().unwrap()),
        1
    );

    let checked = service.handle(&name_check_request("testa", "one")).unwrap();
    assert_eq!(name_check_status(&checked.replies[0]), 2);
    let duplicate = service.handle(&create_request("Testa", "One")).unwrap();
    assert_eq!(
        i32::from_le_bytes(duplicate.replies[0][1..5].try_into().unwrap()),
        4
    );
}

// L058: the creator's picker sends its pick (up to 11 + 16 letters) as the temporary pair
// (client 0x0070cfa0); a native run was refused "Jessica Grittycandle" with check-name code 5.
#[test]
fn picker_names_pass_check_name_and_create_through_the_login_service() {
    let t = Temp::new();
    let mut service = service(&t);
    service.handle(&login_request()).unwrap();
    for (first, last) in [
        ("Jessica", "Grittycandle"),
        ("Christopher", "Imperialslippers"),
    ] {
        let checked = service.handle(&name_check_request(first, last)).unwrap();
        assert_eq!(name_check_status(&checked.replies[0]), 1, "{first} {last}");
        let mut picked = creation_named("", "");
        picked.temporary_first_name = first.into();
        picked.temporary_last_name = last.into();
        let mut request = vec![5];
        request.extend(1i32.to_le_bytes());
        payload(&mut request, &picked.encode());
        assert!(service.handle(&request).unwrap().created, "{first} {last}");
    }
    let too_long = service
        .handle(&name_check_request("Jessica", "Imperialslipperss"))
        .unwrap();
    assert_eq!(name_check_status(&too_long.replies[0]), 10); // Last_Name_Long_PS3
}

// L055: osfr-login gives every peer its own LoginService on one store file (on the unauthenticated
// loopback path all of them log in as the one local session), and each caches the store at login.
fn delete_request(guid: u64) -> Vec<u8> {
    let mut out = vec![9];
    out.extend(guid.to_le_bytes());
    out
}

fn delete_status(reply: &[u8]) -> i32 {
    assert_eq!(reply[0], 10);
    i32::from_le_bytes(reply[9..13].try_into().unwrap())
}

#[test]
fn two_sessions_on_one_store_both_keep_the_characters_they_create() {
    let t = Temp::new();
    let (mut a, mut b) = (service(&t), service(&t));
    a.handle(&login_request()).unwrap();
    b.handle(&login_request()).unwrap();
    let created = a.handle(&create_request("Testa", "One")).unwrap();
    assert_eq!(created.replies[0], login::create_reply(1, 1));
    // B logged in before A saved: its create still sees guid 1 taken.
    let created = b.handle(&create_request("Testb", "Two")).unwrap();
    assert_eq!(created.replies[0], login::create_reply(1, 2));
    let store = AccountStore::load(&t.0.join("accounts.json")).unwrap();
    assert_eq!(store.account(1).unwrap().character_guids, [1, 2]);
    assert_eq!(store.character(1).unwrap().identity.first_name, "Testa");
    assert_eq!(store.character(2).unwrap().identity.first_name, "Testb");
    // Account 1's guid 1 is mirrored to identity.json; it still holds A's character.
    let mirrored = identity::load(&t.0.join("identity.json")).unwrap().unwrap();
    assert_eq!(mirrored.first_name, "Testa");
}

#[test]
fn a_name_another_session_created_is_refused_and_nothing_is_written() {
    let t = Temp::new();
    let (mut a, mut b) = (service(&t), service(&t));
    a.handle(&login_request()).unwrap();
    b.handle(&login_request()).unwrap();
    assert!(a.handle(&create_request("Testn", "Same")).unwrap().created);
    let store_before = fs::read(t.0.join("accounts.json")).unwrap();
    let identity_before = fs::read(t.0.join("identity.json")).unwrap();
    let refused = b.handle(&create_request("Testn", "Same")).unwrap();
    assert!(!refused.created);
    assert_eq!(refused.replies[0], login::create_reply(4, 0)); // NameTaken
    assert_eq!(fs::read(t.0.join("accounts.json")).unwrap(), store_before);
    assert_eq!(
        fs::read(t.0.join("identity.json")).unwrap(),
        identity_before
    );
}

#[test]
fn delete_checks_ownership_on_the_file_and_keeps_characters_saved_since() {
    let t = Temp::new();
    let mut a = service(&t);
    let mut b = service_as(&t, "synthetic-other");
    a.handle(&login_request()).unwrap(); // account 1
    b.handle(&login_request_as("synthetic-other")).unwrap(); // account 2
    let created = b.handle(&create_request("Testb", "Two")).unwrap();
    assert_eq!(created.replies[0], login::create_reply(1, 1));
    let created = a.handle(&create_request("Testa", "One")).unwrap();
    assert_eq!(created.replies[0], login::create_reply(1, 2));
    let before = fs::read(t.0.join("accounts.json")).unwrap();
    // B does not own guid 2: refused, nothing written.
    let refused = b.handle(&delete_request(2)).unwrap();
    assert_eq!(delete_status(&refused.replies[0]), 2);
    assert_eq!(fs::read(t.0.join("accounts.json")).unwrap(), before);
    // B deletes its own character without dropping the one A saved after B's last write.
    let deleted = b.handle(&delete_request(1)).unwrap();
    assert_eq!(delete_status(&deleted.replies[0]), 1);
    let store = AccountStore::load(&t.0.join("accounts.json")).unwrap();
    assert!(store.character(1).is_none());
    assert_eq!(store.account(1).unwrap().character_guids, [2]);
    assert!(store.account(2).unwrap().character_guids.is_empty());
}

#[test]
fn create_and_delete_refuse_when_the_file_maps_the_account_to_another_login() {
    let t = Temp::new();
    let mut a = service(&t);
    a.handle(&login_request()).unwrap(); // account 1 for this login
    // The store is replaced (an operator restore, say): account 1 now belongs to another login.
    fs::remove_file(t.0.join("accounts.json")).unwrap();
    let mut other = service_as(&t, "synthetic-other");
    other.handle(&login_request_as("synthetic-other")).unwrap();
    let created = other.handle(&create_request("Testb", "Two")).unwrap();
    assert!(created.created);
    let store_before = fs::read(t.0.join("accounts.json")).unwrap();
    let identity_before = fs::read(t.0.join("identity.json")).unwrap();
    assert!(a.handle(&create_request("Testa", "One")).is_err());
    assert!(a.handle(&delete_request(1)).is_err());
    assert_eq!(fs::read(t.0.join("accounts.json")).unwrap(), store_before);
    assert_eq!(
        fs::read(t.0.join("identity.json")).unwrap(),
        identity_before
    );
}

/// L042: moderation finds a player by the words of their name, so a new name whose words match
/// a taken one, however the spaces fall between its parts, is taken too; else a player could
/// hide behind a look-alike on a spare slot.
#[test]
fn split_variants_of_a_taken_name_are_taken() {
    let t = Temp::new();
    let mut service = service(&t);
    service.handle(&login_request()).unwrap();
    let status = |service: &mut LoginService, packet: Vec<u8>| {
        let reply = &service.handle(&packet).unwrap().replies[0];
        i32::from_le_bytes(reply[1..5].try_into().unwrap())
    };
    assert_eq!(status(&mut service, create_request("Al Bo", "Cy")), 1);
    for (first, last) in [("Al", "Bo Cy"), ("al", "BO cy")] {
        let checked = service.handle(&name_check_request(first, last)).unwrap();
        assert_eq!(name_check_status(&checked.replies[0]), 2, "{first} {last}");
        let created = status(&mut service, create_request(first, last));
        assert_eq!(created, 4, "{first} {last}");
    }
    // L049 refuses a doubled space as a separator run: invalid (7), before the name check.
    assert_eq!(status(&mut service, create_request("Al  Bo", "Cy")), 7);
    // The pending (temporary) pair follows the same rule.
    let mut creation = creation_named("Dee", "Eff");
    creation.temporary_first_name = "Al".into();
    creation.temporary_last_name = "Bo Cy".into();
    let mut packet = vec![5];
    packet.extend(1i32.to_le_bytes());
    payload(&mut packet, &creation.encode());
    assert_eq!(status(&mut service, packet), 5);
    let store = AccountStore::load(&t.0.join("accounts.json")).unwrap();
    let characters: usize = store
        .account_ids()
        .map(|id| store.account(id).unwrap().character_guids.len())
        .sum();
    assert_eq!(characters, 1);
}

/// L042: only a save holds the store writer lock. While another writer holds it (a staff command,
/// osfr-account or a stale lock), login, the character list and the name check still answer, so a
/// client spamming them cannot keep a staff write waiting. Create, delete and a first login that
/// adds an account wait the lock out, are refused and write nothing, and work once it is free.
#[test]
fn only_saves_wait_for_the_store_lock() {
    let t = Temp::new();
    let path = t.0.join("accounts.json");
    let held = AccountStoreLock::acquire(&path).unwrap();
    assert!(service(&t).handle(&login_request()).is_err());
    assert!(!path.exists());
    drop(held);
    let mut a = service(&t);
    a.handle(&login_request()).unwrap();
    assert!(a.handle(&create_request("Testa", "One")).unwrap().created);

    let held = AccountStoreLock::acquire(&path).unwrap();
    let (store_before, identity_before) = (
        fs::read(&path).unwrap(),
        fs::read(t.0.join("identity.json")).unwrap(),
    );
    let mut b = service(&t);
    assert_eq!(
        b.handle(&login_request()).unwrap().replies,
        [login_reply_with_max_characters(true, 6)]
    );
    assert_eq!(b.handle(&[11]).unwrap().replies[0][0], 12);
    let checked = b.handle(&name_check_request("testa", "one")).unwrap();
    assert_eq!(name_check_status(&checked.replies[0]), 2);
    let mut other = service_as(&t, "synthetic-other");
    assert!(other.handle(&login_request_as("synthetic-other")).is_err());
    assert!(b.handle(&create_request("Testb", "Two")).is_err());
    assert!(b.handle(&delete_request(1)).is_err());
    assert_eq!(fs::read(&path).unwrap(), store_before);
    assert_eq!(
        fs::read(t.0.join("identity.json")).unwrap(),
        identity_before
    );
    drop(held);

    assert!(b.handle(&create_request("Testb", "Two")).unwrap().created);
    let deleted = b.handle(&delete_request(1)).unwrap();
    assert_eq!(delete_status(&deleted.replies[0]), 1);
    assert!(other.handle(&login_request_as("synthetic-other")).is_ok());
    let store = AccountStore::load(&path).unwrap();
    assert_eq!(store.account(1).unwrap().character_guids, [2]);
    assert!(store.account(2).is_some());
    assert!(!t.0.join("accounts.json.lock").exists());
}

#[test]
fn malformed_creation_and_invalid_appearance_do_not_save() {
    let t = Temp::new();
    let mut service = service(&t);
    service.handle(&login_request()).unwrap();
    let bytes = vector("creation_payload");
    for n in 0..bytes.len() {
        assert!(NativeCreation::decode(&bytes[..n]).is_err());
    }
    let mut bad = bytes.clone();
    bad.push(0);
    assert!(NativeCreation::decode(&bad).is_err());
    let mut c = NativeCreation::decode(&bytes).unwrap();
    c.choices[0] = 4;
    let mut packet = vec![5];
    packet.extend(1i32.to_le_bytes());
    payload(&mut packet, &c.encode());
    assert_eq!(
        service.handle(&packet).unwrap().replies[0],
        login::create_reply(2, 0)
    );
    assert!(
        identity::load(&t.0.join("identity.json"))
            .unwrap()
            .is_none()
    );
    assert!(!identity::valid_name("<script>", ""));
    assert!(!identity::valid_name("Ari\0", ""));
    assert!(identity::valid_name("Éva", "O'Vale"));
}

#[test]
fn native_default_server_zero_uses_only_local_realm_and_preserves_identity() {
    let t = Temp::new();
    let mut service = service(&t);
    service.handle(&login_request()).unwrap();
    let mut create = vec![5];
    create.extend(2i32.to_le_bytes());
    payload(&mut create, &vector("creation_payload"));
    let rejected = service.handle(&create).unwrap();
    assert_eq!(rejected.replies[0], login::create_reply(2, 0));
    assert_eq!(
        rejected.creation_rejection.as_deref(),
        Some("unsupported local server")
    );
    assert!(!t.0.join("identity.json").exists());
    create[1..5].copy_from_slice(&0i32.to_le_bytes());
    let created = service.handle(&create).unwrap();
    assert!(created.created);
    assert!(created.creation_rejection.is_none());
    let saved = fs::read(t.0.join("identity.json")).unwrap();
    let duplicate = service.handle(&create).unwrap();
    assert_eq!(duplicate.replies[0], login::create_reply(4, 0));
    assert_eq!(fs::read(t.0.join("identity.json")).unwrap(), saved);
    let mut client = Vec::new();
    string(&mut client, "en_US");
    client.extend(8i32.to_le_bytes());
    client.extend([0, 0]);
    client.extend(0i64.to_le_bytes());
    let mut select = vec![7];
    select.extend(1u64.to_le_bytes());
    select.extend(0i32.to_le_bytes());
    payload(&mut select, &client);
    assert!(service.handle(&select).unwrap().selected);
    select[9..13].copy_from_slice(&2i32.to_le_bytes());
    assert!(!service.handle(&select).unwrap().selected);
    assert_eq!(fs::read(t.0.join("identity.json")).unwrap(), saved);
}

#[test]
fn gateway_crypto_handoff_rejects_invalid_native_key_length() {
    assert_eq!(login::DEFAULT_GATEWAY_TICKET.len(), 32);
    let t = Temp::new();
    assert!(
        LoginService::new(
            resources(&t),
            LoginConfig {
                identity_file: t.0.join("identity.json"),
                legacy_save_file: t.0.join("state.json"),
                account_store_file: t.0.join("accounts.json"),
                gateway_address: "127.0.0.1:20260".into(),
                local_session: "synthetic-session".into(),
                gateway_ticket: "local-rust-character".into(),
                handoff_key: b"synthetic-handoff-key".to_vec(),
                max_characters: 6,
                underage: false,
                open_chat: true,
            }
        )
        .is_err()
    );
}

#[test]
fn rc4_stream_zero_escape_and_group_framing() {
    let mut cipher = Rc4::new(b"Key").unwrap();
    let mut encrypted = b"Plaintext".to_vec();
    cipher.apply(&mut encrypted);
    assert_eq!(encrypted, hex("bbf316e8d940af0ad3"));
    let mut parts = Rc4::new(b"Key").unwrap();
    let mut a = b"Plain".to_vec();
    let mut b = b"text".to_vec();
    parts.apply(&mut a);
    parts.apply(&mut b);
    a.extend(b);
    assert_eq!(a, encrypted);
    assert_eq!(
        login::frame_ciphertext(&[0, 25, 9]).unwrap(),
        vec![0, 0, 25, 9]
    );
    assert_eq!(
        login::ciphertext_messages(&[0, 0, 25, 9]).unwrap(),
        vec![&[0, 25, 9][..]]
    );
    let grouped = [0, 25, 3, 1, 2, 3, 3, 0, 0, 7];
    assert_eq!(
        login::ciphertext_messages(&grouped).unwrap(),
        vec![&[1, 2, 3][..], &[0, 7][..]]
    );
    assert!(login::ciphertext_messages(&[0, 25, 9, 1]).is_err());
    assert!(login::ciphertext_messages(&[0, 25, 2, 0, 25]).is_err());
}

#[test]
fn group_child_lengths_follow_the_client_variable_length() {
    // Client reader GetVariableValue 0x010daea0: `FF hi lo`, and only `FF FF FF` selects a u32.
    let short = [2u8; 6];
    for (prefix, size) in [
        (vec![255, 0, 255], 255),
        (vec![255, 0, 254], 254),
        (vec![255, 1, 44], 300),
        (vec![255, 255, 255, 0, 0, 1, 44], 300),
    ] {
        let child = vec![1u8; size];
        let group = [vec![0, 25], prefix, child.clone(), vec![6], short.to_vec()].concat();
        let children = login::ciphertext_messages(&group).unwrap();
        assert_eq!(children, vec![&child[..], &short[..]], "{size}-byte child");
    }
    for bad in [
        vec![0, 25, 255],
        vec![0, 25, 255, 0],
        vec![0, 25, 255, 255, 255],
        vec![0, 25, 255, 255, 255, 0, 1, 17],
        vec![0, 25, 255, 255, 255, 0, 1, 17, 112, 1, 1],
        [vec![0, 25, 255, 0, 255], vec![1; 254]].concat(),
    ] {
        assert!(login::ciphertext_messages(&bad).is_err(), "{bad:?}");
    }
}

fn self_packet() -> Vec<u8> {
    let mut out = vec![5, 0, 1, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0];
    out.extend(99i64.to_le_bytes());
    out.extend(1u64.to_le_bytes());
    out.extend(1i32.to_le_bytes());
    string(&mut out, "old-head-long.adr");
    string(&mut out, "old-hair.adr");
    out.extend(11i32.to_le_bytes());
    out.extend(22i32.to_le_bytes());
    for s in ["old-skin", "none", ""] {
        string(&mut out, s);
    }
    out.extend([0xab; 64]);
    string(&mut out, "Before");
    string(&mut out, "Example");
    out.extend(35i32.to_le_bytes());
    out.extend([0x5a; 100]);
    let n = out.len();
    out[3..7].copy_from_slice(&((n - 7) as u32).to_le_bytes());
    out[9..13].copy_from_slice(&((n - 13) as u32).to_le_bytes());
    out
}

#[test]
fn self_identity_patch_preserves_unowned_tail_and_updates_nested_lengths() {
    let old = self_packet();
    let changed = identity::patch_self(&old, &sample()).unwrap();
    assert_eq!(&changed[changed.len() - 104..], &old[old.len() - 104..]);
    assert_eq!(
        u32::from_le_bytes(changed[3..7].try_into().unwrap()) as usize,
        changed.len() - 7
    );
    assert_eq!(
        u32::from_le_bytes(changed[9..13].try_into().unwrap()) as usize,
        changed.len() - 13
    );
    assert_eq!(identity::patch_self(&changed, &sample()).unwrap(), changed);
    assert!(changed.windows(8).any(|x| x == b"head.adr"));
    assert!(!changed.windows(16).any(|x| x == b"old-head-long.adr"));
    let mut bad = old;
    bad[9] = 0;
    assert!(identity::patch_self(&bad, &sample()).is_err());
}

#[test]
#[ignore = "requires explicit private supplied-assembly oracle directory"]
fn private_self_patch_equals_supplied_compiled_serializer() {
    let directory = PathBuf::from(
        std::env::var_os("OSFR_IDENTITY_ORACLE")
            .expect("set OSFR_IDENTITY_ORACLE to private rust-login content"),
    );
    let original = fs::read(directory.join("self-original-body.bin")).unwrap();
    let expected = fs::read(directory.join("self-identity-expected-body.bin")).unwrap();
    fn wrap(body: &[u8]) -> Vec<u8> {
        let mut inner = 12u16.to_le_bytes().to_vec();
        payload(&mut inner, body);
        let mut out = vec![5, 0, 1];
        payload(&mut out, &inner);
        out
    }
    let mut identity = sample();
    identity.model = 2;
    identity.model_customization = "fairy_wings.adr".into();
    identity.creation.choices[0] = 2;
    identity.creation.choices[3] = 1;
    assert_eq!(
        identity::patch_self(&wrap(&original), &identity).unwrap(),
        wrap(&expected)
    );
}

#[test]
// OSFR_CREATOR_RESOURCES is a client Resources/ with the creator overlay installed
// (scripts/bundle/client_overlay.py): a stock tree fails the pinned-table check.
#[ignore = "requires private native creator capture, resource tables and login key"]
fn private_actual_native_creation_resolves_against_supplied_resources() {
    use osfr_tools::{
        session::decode_controls,
        soe::{Codec, RELIABLE_DATA, RELIABLE_FRAGMENT, ReliableReceiver},
    };
    use sha2::{Digest, Sha256};
    let run =
        PathBuf::from(std::env::var_os("OSFR_CREATOR_RUN").expect("OSFR_CREATOR_RUN is required"));
    let resources = PathBuf::from(
        std::env::var_os("OSFR_CREATOR_RESOURCES").expect("OSFR_CREATOR_RESOURCES is required"),
    );
    let key =
        PathBuf::from(std::env::var_os("OSFR_CREATOR_KEY").expect("OSFR_CREATOR_KEY is required"));
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(run.join("manifest.json")).unwrap()).unwrap();
    let frames = manifest["frames"].as_array().unwrap();
    let response = frames
        .iter()
        .find(|f| {
            f["service"] == "login" && f["direction"] == "server-to-client" && f["opcode"] == 2
        })
        .unwrap();
    let response = fs::read(run.join(response["path"].as_str().unwrap())).unwrap();
    assert_eq!(response.len(), 21);
    let seed = u32::from_be_bytes(response[6..10].try_into().unwrap());
    let codec = Codec::new(seed, response[10], false, login::MAX_LOGIN_MESSAGE + 1).unwrap();
    let mut receiver =
        ReliableReceiver::new(0, 128, login::MAX_LOGIN_MESSAGE + 1, 128 * 1024).unwrap();
    let mut cipher = Rc4::new(&fs::read(&key).unwrap()).unwrap();
    let t = Temp::new();
    let mut service = LoginService::new(
        Resources::load(&resources).unwrap(),
        LoginConfig {
            identity_file: t.0.join("identity.json"),
            legacy_save_file: t.0.join("state.json"),
            account_store_file: t.0.join("accounts.json"),
            gateway_address: "127.0.0.1:20260".into(),
            local_session: "synthetic-session".into(),
            gateway_ticket: "synthetic-ticket".into(),
            handoff_key: fs::read(&key).unwrap(),
            max_characters: 6,
            underage: false,
            open_chat: true,
        },
    )
    .unwrap();
    service.handle(&login_request()).unwrap();
    let resources = Resources::load(&resources).unwrap();
    let mut creations = 0;
    for frame in frames.iter().filter(|f| {
        f["service"] == "login" && f["direction"] == "client-to-server" && f["opcode"] != 1
    }) {
        let wire = fs::read(run.join(frame["path"].as_str().unwrap())).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(&wire)),
            frame["sha256"].as_str().unwrap()
        );
        for packet in decode_controls(&codec, &wire).unwrap() {
            if !matches!(packet.opcode, RELIABLE_DATA | RELIABLE_FRAGMENT) {
                continue;
            }
            for message in receiver.receive(&packet).unwrap().messages {
                for ciphertext in login::ciphertext_messages(&message).unwrap() {
                    let mut application = ciphertext.to_vec();
                    cipher.apply(&mut application);
                    if application[0] != 5 {
                        continue;
                    }
                    assert_eq!(i32::from_le_bytes(application[1..5].try_into().unwrap()), 0);
                    let length = u32::from_le_bytes(application[5..9].try_into().unwrap()) as usize;
                    assert_eq!(application.len(), length + 9);
                    let creation = NativeCreation::decode(&application[9..]).unwrap();
                    let choices = creation.choices;
                    let identity = resources.resolve(creation).unwrap_or_else(|error| {
                        panic!(
                            "native choice validation failed: {error}; numeric choices {choices:?}"
                        )
                    });
                    identity.validate().unwrap();
                    let created = service.handle(&application).unwrap();
                    assert!(created.created, "{:?}", created.creation_rejection);
                    assert_eq!(created.replies[0], login::create_reply(1, 1));
                    assert_eq!(
                        identity::load(&t.0.join("identity.json")).unwrap(),
                        Some(identity)
                    );
                    assert_eq!(
                        service.handle(&application).unwrap().replies[0],
                        login::create_reply(6, 0)
                    );
                    creations += 1;
                }
            }
        }
    }
    assert_eq!(creations, 1);
}

#[test]
fn a_port_forwarded_realm_hands_outside_players_its_public_gateway() {
    // L076: v0.1.4 refused the --advertise address here, so every remote player's session failed.
    let t = Temp::new();
    let config = |gateway: &str| LoginConfig {
        identity_file: t.0.join("identity.json"),
        legacy_save_file: t.0.join("state.json"),
        account_store_file: t.0.join("accounts.json"),
        gateway_address: gateway.into(),
        local_session: "synthetic-session".into(),
        gateway_ticket: "synthetic-ticket".into(),
        handoff_key: b"synthetic-handoff-key".to_vec(),
        max_characters: 6,
        underage: false,
        open_chat: true,
    };
    for gateway in ["203.0.113.7:20041", "192.168.1.20:20041", "127.0.0.1:20041"] {
        assert!(
            LoginService::new(resources(&t), config(gateway)).is_ok(),
            "{gateway}"
        );
    }
    for gateway in ["0.0.0.0:20041", "224.0.0.1:20041", "255.255.255.255:20041"] {
        assert!(
            LoginService::new(resources(&t), config(gateway)).is_err(),
            "{gateway}"
        );
    }
}
