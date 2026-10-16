//! Work item 050 acceptance: character naming/select lifecycle.
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `lab/decompiled/client/FreeRealms/pseudocode.c`, `ClientAccountManager::OnRequestCheckName`
//!   (`FUN_00710e20`) switches the check-name result code onto the literal message keys
//!   `Char_Name_Taken_PS3` (2), `Char_Name_Profane_PS3` (3), `Char_Name_Incorrect_Length_PS3` (5),
//!   `Char_Server_Failure_PS3` (6), `First_Name_Short_PS3` (7), `Last_Name_Short_PS3` (8),
//!   `First_Name_Long_PS3` (9), `Last_Name_Long_PS3` (10), `-1 | 4 -> Char_Name_Invalid_PS3`,
//!   `1 -> available`, and every other value -> `Char_Unable_Log_Server_PS3`. The same function
//!   walks a suggestion list whose rows carry Name/LastPrefix/LastSuffix.
//! * The same file's `FUN_00953950`/`FUN_00953a10` constructors pin the request/reply opcodes
//!   0xd2 (210) and 0xd3 (211).
//! * `lab/working/OSFR.Client/OSFR Client/Resources/CharacterCreate/CharacterCreateNames.txt`
//!   holds 840 `character.first.name` rows (PART_ID 1) and 476 last-name rows (PART_ID 2:
//!   234 `last.name.prefix` + 242 `last.name.suffix`), plus the two format rows
//!   `character.name.format` and `character.lastname.format`.
//! * `Resources/CodeStringMappings.txt` maps the `Char_Name_*_PS3` keys onto locale ids, and
//!   `ErrorCodesCheckName.txt` is a header-only table in the supplied client and in the pinned
//!   public manifest (36 bytes, XXHash64 9897321053530986055).
//! * The saved native-created character `lab/state/opening-supported-identity.json` is
//!   `Capo` + `Indigocauldron` = `male.first.name.56` + `last.name.prefix.102` +
//!   `last.name.suffix.32`.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers `pub mod piece_050;` under `src/restoration/` and `src/lib.rs`.

#[path = "../src/restoration/piece_050.rs"]
#[allow(dead_code)]
mod piece_050;

use osfr_tools::{
    identity::{self, Resources},
    login::{self, LoginConfig, LoginService},
};
use piece_050::{
    CHARACTER_DELETE_REPLY_OPCODE, CHARACTER_DELETE_REQUEST_OPCODE, CHECK_NAME_REPLY_OPCODE,
    CHECK_NAME_REQUEST_OPCODE, CUSTOM_NAME_COMBINED_MIN_CHARS, CUSTOM_NAME_PART_MAX_CHARS,
    CUSTOM_NAME_PART_MIN_CHARS, DeleteDecision, FIRST_NAME_LONG_MESSAGE, FIRST_NAME_SHORT_MESSAGE,
    INCORRECT_LENGTH_MESSAGE, LAST_NAME_LONG_MESSAGE, LAST_NAME_SHORT_MESSAGE,
    NAME_FIELD_MAX_BYTES, NAME_PART_MAX_CHARS, NameCatalogue, NameCheckReply, NameCheckRequest,
    NameResultCode, SHORTEST_PRESET_NAME, SHORTEST_PRESET_NAME_LENGTH, classify_custom_name,
    classify_name, delete_decision,
};
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

#[test]
fn check_name_wire_round_trips_the_recovered_fields() {
    let request = NameCheckRequest {
        first_name: "Capo".into(),
        last_name: "Indigocauldron".into(),
        name_definition: "male.first.name.56".into(),
        last_prefix_definition: "last.name.prefix.102".into(),
        last_suffix_definition: "last.name.suffix.32".into(),
    };
    let bytes = request.encode();
    assert_eq!(&bytes[..2], &CHECK_NAME_REQUEST_OPCODE.to_le_bytes());
    assert_eq!(NameCheckRequest::decode(&bytes).unwrap(), request);
    assert!(NameCheckRequest::decode(&bytes[..bytes.len() - 1]).is_err());

    let mut extra = bytes.clone();
    extra.push(0);
    assert!(NameCheckRequest::decode(&extra).is_err());
}

#[test]
fn check_name_reply_matches_the_community_vector() {
    let reply = NameCheckReply {
        first_name: "Ari".into(),
        last_name: "Vale".into(),
        result: NameResultCode::Available,
    }
    .encode();
    assert_eq!(&reply[..2], &CHECK_NAME_REPLY_OPCODE.to_le_bytes());
    assert_eq!(
        NameCheckReply::decode(&reply).unwrap().result,
        NameResultCode::Available
    );

    // The pinned Sanctuary vector wraps opcode 211 in the login tunnel envelope
    // (`17`, server id, payload length); rebuild that envelope around this reply.
    let mut wrapped = vec![17];
    wrapped.extend(1i32.to_le_bytes());
    payload(&mut wrapped, &reply);
    assert_eq!(wrapped, vector("name_ok"));
}

#[test]
fn every_client_handled_result_code_keeps_its_recovered_message_key() {
    let expected = [
        (-1, NameResultCode::Invalid),
        (1, NameResultCode::Available),
        (2, NameResultCode::Taken),
        (3, NameResultCode::Profane),
        (4, NameResultCode::Invalid),
        (5, NameResultCode::IncorrectLength),
        (6, NameResultCode::ServiceFailure),
        (7, NameResultCode::FirstNameTooShort),
        (8, NameResultCode::LastNameTooShort),
        (9, NameResultCode::FirstNameTooLong),
        (10, NameResultCode::LastNameTooLong),
    ];
    for (code, variant) in expected {
        assert_eq!(
            NameResultCode::from_code(code),
            Some(variant),
            "code {code} must map to its recovered case"
        );
        assert!(NameResultCode::is_recovered_code(code));
    }
    assert_eq!(NameResultCode::Available.message_key(), "");
    assert_eq!(
        NameResultCode::from_code(-1).unwrap().code(),
        NameResultCode::Invalid.code(),
        "-1 is an alias of the invalid case, not a distinct wire value"
    );
    assert_eq!(NameResultCode::Taken.message_key(), "Char_Name_Taken_PS3");
    // The client switch has no case for 11; the community enum's `IllegalCharacters = 11` is
    // therefore only reachable through the default branch (server failure), not a real rule.
    assert_eq!(NameResultCode::from_code(11), None);
    assert_eq!(NameResultCode::from_code(0xff), None);
    assert_eq!(
        piece_050::UNKNOWN_RESULT_MESSAGE_KEY,
        "Char_Unable_Log_Server_PS3"
    );
}

#[test]
fn classify_matches_the_create_side_name_rule_and_reports_duplicates() {
    let cases = [
        ("Ari", "Vale", true),
        ("Éva", "O'Vale", true),
        ("Mary-Jane", "d'Arc", true),
        ("", "Vale", false),
        (" Ari", "Vale", false),
        ("Ari", " Vale", false),
        ("Ari", "", true),
        ("Ari9", "Vale", false),
        ("<script>", "", false),
        ("Ari\0", "Vale", false),
    ];
    for (first, last, valid) in cases {
        assert_eq!(
            identity::valid_name(first, last),
            valid,
            "test table disagrees with identity::valid_name for {first:?} {last:?}"
        );
        let expected = if valid {
            NameResultCode::Available
        } else {
            NameResultCode::Invalid
        };
        assert_eq!(
            classify_name(first, last, &[]),
            expected,
            "classify_name disagrees for {first:?} {last:?}"
        );
    }

    let over_long = "a".repeat(NAME_PART_MAX_CHARS + 1);
    assert_eq!(
        classify_name(&over_long, "Vale", &[]),
        NameResultCode::Invalid
    );
    let at_bound = "a".repeat(NAME_PART_MAX_CHARS);
    assert_eq!(
        classify_name(&at_bound, "Vale", &[]),
        NameResultCode::Available
    );
    // The 128-byte field bound is the wire cap: a maximum-length name part round trips and a
    // 129-byte field is rejected by the decoder.
    let request = NameCheckRequest {
        first_name: "a".repeat(NAME_FIELD_MAX_BYTES),
        last_name: String::new(),
        name_definition: String::new(),
        last_prefix_definition: String::new(),
        last_suffix_definition: String::new(),
    };
    assert!(NameCheckRequest::decode(&request.encode()).is_ok());
    let mut too_long = request.encode();
    too_long[1..5].copy_from_slice(&((NAME_FIELD_MAX_BYTES + 1) as u32).to_le_bytes());
    too_long.insert(5, b'a');
    assert!(NameCheckRequest::decode(&too_long).is_err());

    assert_eq!(
        classify_name("ari", "vale", &["Ari Vale"]),
        NameResultCode::Taken
    );
    assert_eq!(
        classify_name("Ari", "Vale", &["Capo Indigocauldron"]),
        NameResultCode::Available
    );
    assert_eq!(
        classify_name("", "Vale", &["Ari Vale"]),
        NameResultCode::Invalid,
        "an invalid name is never reported as taken"
    );
}

#[test]
fn recovered_length_message_is_not_a_pc_rule() {
    // The recovered length texts state the custom-name rules, and the shipped preset tables
    // compose names as short as "Ari Oxarm" (9 characters). The production-matching shape rule
    // therefore stays permissive while the custom-name policy reports the recovered code.
    assert!(INCORRECT_LENGTH_MESSAGE.contains("at least 15 characters"));
    assert!(FIRST_NAME_SHORT_MESSAGE.contains("at least two characters"));
    assert!(LAST_NAME_SHORT_MESSAGE.contains("at least two characters"));
    assert!(FIRST_NAME_LONG_MESSAGE.contains("less than 15 characters"));
    assert!(LAST_NAME_LONG_MESSAGE.contains("less than 15 characters"));
    assert_eq!(SHORTEST_PRESET_NAME, "Ari Oxarm");
    assert_eq!(SHORTEST_PRESET_NAME.len(), SHORTEST_PRESET_NAME_LENGTH);
    assert_eq!(
        classify_name("Ari", "Oxarm", &[]),
        NameResultCode::Available
    );
    assert_eq!(
        classify_custom_name("Ari", "Oxarm", &[]),
        NameResultCode::IncorrectLength
    );
}

#[test]
fn custom_name_length_codes_follow_the_recovered_locale_text() {
    assert_eq!(CUSTOM_NAME_PART_MIN_CHARS, 2);
    assert_eq!(CUSTOM_NAME_PART_MAX_CHARS, 14);
    assert_eq!(CUSTOM_NAME_COMBINED_MIN_CHARS, 15);

    assert_eq!(
        classify_custom_name("A", "Merryweather", &[]),
        NameResultCode::FirstNameTooShort
    );
    assert_eq!(
        classify_custom_name("Alexzandria", "V", &[]),
        NameResultCode::LastNameTooShort
    );
    let long_first = "A".repeat(CUSTOM_NAME_PART_MAX_CHARS + 1);
    assert_eq!(
        classify_custom_name(&long_first, "Merryweather", &[]),
        NameResultCode::FirstNameTooLong
    );
    let long_last = "V".repeat(CUSTOM_NAME_PART_MAX_CHARS + 1);
    assert_eq!(
        classify_custom_name("Alexzandria", &long_last, &[]),
        NameResultCode::LastNameTooLong
    );
    // 10 + 1 + 10 = 21 characters: inside every recovered bound.
    assert_eq!(
        classify_custom_name("Alexzandria", "Merryweathe", &[]),
        NameResultCode::Available
    );
    assert_eq!(
        classify_custom_name("Alexzandria", "merryweathe", &["alexzandria merryweathe"]),
        NameResultCode::Taken
    );
    assert_eq!(
        classify_custom_name("Alexzandria9", "Merryweathe", &[]),
        NameResultCode::Invalid,
        "illegal characters stay invalid rather than length-coded"
    );
    // The production shape rule keeps accepting the shorter name the local service already
    // creates; only the recovered custom policy reports the combined-length code.
    assert_eq!(classify_name("Ari", "Vale", &[]), NameResultCode::Available);
    assert_eq!(
        classify_custom_name("Ari", "Vale", &[]),
        NameResultCode::IncorrectLength
    );
}

fn sample_catalogue() -> NameCatalogue {
    NameCatalogue::parse(
        "#GLOSSARY_ID^NAME^PART_ID^\n\
         character.lastname.format^[*char_lastname_prefix*][*char_lastname_suffix*]^0^\n\
         character.name.format^[*char_firstname*] [*char_lastname*]^0^\n\
         male.first.name.56^Capo^1^\n\
         female.first.name.1^Abby^1^\n\
         last.name.prefix.102^Indigo^2^\n\
         last.name.suffix.32^cauldron^2^\n\
         last.name.suffix.1^antlers^2^\n",
    )
    .unwrap()
}

#[test]
fn catalogue_parses_the_supplied_table_shape() {
    let catalogue = sample_catalogue();
    assert_eq!(catalogue.len(), 5);
    assert_eq!(
        catalogue.first_name_format(),
        "[*char_firstname*] [*char_lastname*]"
    );
    assert_eq!(
        catalogue.last_name_format(),
        "[*char_lastname_prefix*][*char_lastname_suffix*]"
    );
    assert_eq!(catalogue.lookup("male.first.name.56"), Some("Capo"));
    assert_eq!(catalogue.lookup("last.name.suffix.32"), Some("cauldron"));
    assert_eq!(catalogue.lookup("last.name.suffix.999"), None);

    assert!(NameCatalogue::parse("#GLOSSARY_ID^NAME^PART_ID^\n").is_err());
    assert!(NameCatalogue::parse("character.name.format^[*x*]^0^\n").is_err());
    assert!(NameCatalogue::parse("a^1^1^\na^2^1^\n").is_err());
}

#[test]
fn catalogue_composes_the_saved_native_character_name() {
    let catalogue = sample_catalogue();
    let request = NameCheckRequest {
        first_name: "Capo".into(),
        last_name: "Indigocauldron".into(),
        name_definition: "male.first.name.56".into(),
        last_prefix_definition: "last.name.prefix.102".into(),
        last_suffix_definition: "last.name.suffix.32".into(),
    };
    let resolved = catalogue.resolve(&request).unwrap();
    assert_eq!(resolved.first_name, "Capo");
    assert_eq!(resolved.last_name, "Indigocauldron");
    assert_eq!(resolved.display_name(), "Capo Indigocauldron");

    let custom = NameCheckRequest {
        first_name: "Capo".into(),
        last_name: "Indigocauldron".into(),
        name_definition: String::new(),
        last_prefix_definition: String::new(),
        last_suffix_definition: String::new(),
    };
    assert_eq!(catalogue.resolve(&custom).unwrap(), resolved);

    let mut wrong = request.clone();
    wrong.first_name = "NotCapo".into();
    assert!(catalogue.resolve(&wrong).is_err());
    let mut unknown = request.clone();
    unknown.last_prefix_definition = "last.name.prefix.999".into();
    assert!(catalogue.resolve(&unknown).is_err());
    let mut half = request.clone();
    half.last_suffix_definition = String::new();
    assert!(catalogue.resolve(&half).is_err());
}

#[test]
fn delete_wire_and_local_gate_stay_explicit() {
    let request = piece_050::CharacterDeleteRequest { entity_key: 7 };
    let bytes = request.encode();
    assert_eq!(bytes[0], CHARACTER_DELETE_REQUEST_OPCODE);
    assert_eq!(bytes.len(), 9);
    assert_eq!(
        piece_050::CharacterDeleteRequest::decode(&bytes).unwrap(),
        request
    );
    assert!(piece_050::CharacterDeleteRequest::decode(&bytes[..8]).is_err());

    let reply = piece_050::CharacterDeleteReply {
        entity_key: 7,
        status: 2,
    }
    .encode();
    assert_eq!(reply[0], CHARACTER_DELETE_REPLY_OPCODE);
    assert_eq!(
        piece_050::CharacterDeleteReply::decode(&reply)
            .unwrap()
            .status,
        2
    );

    assert_eq!(delete_decision(7, None, false), DeleteDecision::UnknownKey);
    assert_eq!(
        delete_decision(7, Some(8), false),
        DeleteDecision::UnknownKey
    );
    assert_eq!(
        delete_decision(7, Some(7), true),
        DeleteDecision::GameplaySavePresent
    );
    assert_eq!(delete_decision(7, Some(7), false), DeleteDecision::Allowed);
}

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Temp(PathBuf);

impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "osfr-piece-050-{}-{}",
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
    fs::create_dir(t.0.join("CharacterCreate")).unwrap();
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
    LoginService::new(
        resources(t),
        LoginConfig {
            identity_file: t.0.join("identity.json"),
            legacy_save_file: t.0.join("state.json"),
            account_store_file: t.0.join("accounts.json"),
            gateway_address: "127.0.0.1:20260".into(),
            local_session: "synthetic-session".into(),
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
    let mut out = vec![1];
    string(&mut out, "synthetic-session");
    string(&mut out, "synthetic-machine");
    out
}

fn check_name(first: &str, last: &str) -> Vec<u8> {
    let mut check = 210i16.to_le_bytes().to_vec();
    for s in [first, last, "definition", "prefix", "suffix"] {
        string(&mut check, s);
    }
    let mut tunneled = vec![16];
    tunneled.extend(1i32.to_le_bytes());
    payload(&mut tunneled, &check);
    tunneled
}

/// The login tunnel envelope around opcode 211, anchored on the pinned `name_ok` vector.
fn expected_name_reply(first: &str, last: &str, result: i32) -> Vec<u8> {
    let mut inner = 211i16.to_le_bytes().to_vec();
    string(&mut inner, first);
    string(&mut inner, last);
    inner.extend(result.to_le_bytes());
    let mut out = vec![17];
    out.extend(1i32.to_le_bytes());
    payload(&mut out, &inner);
    out
}

/// The production login service already answers opcode 210. This drives that real path to prove
/// the duplicate/invalid name answers and that a rejected name leaves the saved appearance
/// untouched.
#[test]
fn production_check_name_preserves_the_saved_native_appearance() {
    let t = Temp::new();
    let mut service = service(&t);
    service.handle(&login_request()).unwrap();

    let available = expected_name_reply("Ari", "Vale", 1);
    assert_eq!(available, vector("name_ok"));
    assert_eq!(
        service.handle(&check_name("Ari", "Vale")).unwrap().replies[0],
        available
    );
    assert_eq!(
        service.handle(&check_name("Ari9", "Vale")).unwrap().replies[0],
        expected_name_reply("Ari9", "Vale", 4)
    );

    let creation = vector("creation_payload");
    let mut create = vec![5];
    create.extend(1i32.to_le_bytes());
    payload(&mut create, &creation);
    assert!(service.handle(&create).unwrap().created);
    let saved_bytes = fs::read(t.0.join("identity.json")).unwrap();
    let saved = identity::load(&t.0.join("identity.json")).unwrap().unwrap();

    let duplicate = service.handle(&check_name("ari", "vale")).unwrap();
    assert_eq!(duplicate.replies[0], expected_name_reply("ari", "vale", 2));
    assert_eq!(
        fs::read(t.0.join("identity.json")).unwrap(),
        saved_bytes,
        "a rejected duplicate must not touch the saved character"
    );

    service.reset_connection();
    service.handle(&login_request()).unwrap();
    assert_eq!(
        service.handle(&[11]).unwrap().replies[0],
        login::selection_reply(Some(&saved)),
        "selection must still carry the saved appearance"
    );
}
