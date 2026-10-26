//! T763: the TCG service wire against byte fixtures built in the DLL writers' field order
//! (lab/content/tcg/analysis/lead-login-decode-2026-10-26.md), not from this crate's encoder.
use osfr_tools::card_service::{
    self, ConnectionServer, MAX_FRAME, Reader, SessionLogin, client_frame, put_int, reply,
};

fn int(value: i32) -> Vec<u8> {
    let mut out = Vec::new();
    put_int(&mut out, value);
    out
}

/// A client string as `10097b40` writes it: length, text, NUL (the length byte is literal).
fn text(value: &str) -> Vec<u8> {
    assert!(value.len() < 16, "fixture strings use a one-byte length");
    [vec![value.len() as u8], value.as_bytes().to_vec(), vec![0]].concat()
}

/// Three class levels of `[id, version]` as the client writes them (`1019de90`).
fn headers(id: &[u8], version: u8) -> Vec<u8> {
    [id, &[version], id, &[version], id, &[version]].concat()
}

/// The client's 414 (`101935d0` -> `10121d20`): session id, user name, empty host, 0, false, 87.
fn gateway_request() -> Vec<u8> {
    [
        headers(&[0x2E, 0x19], 1),
        text("w-ticket"),
        text("tester"),
        text(""),
        vec![0x00, 0x00, 0x27, 0x05],
    ]
    .concat()
}

/// The client's 411 at version 8 (`103c9140` -> `1017b700`).
fn session_login(session: &str) -> Vec<u8> {
    [
        headers(&[0x2B, 0x19], 8),
        text(session),
        // {1: 1314, 2: 7, 4: 9}: count 3, then key/value pairs.
        vec![0x03, 0x01, 0x22, 0x52, 0x02, 0x07, 0x04, 0x09],
        text("x"),
        text(""),
        vec![0x00],
        text("winXP"),
        text("en_US"),
        text("FR"),
        text("Standalone"),
        text(""),
        text("STATION"),
    ]
    .concat()
}

fn frame(routing: Option<[u32; 3]>, body: &[u8]) -> Vec<u8> {
    let words: Vec<u8> = routing
        .into_iter()
        .flatten()
        .flat_map(u32::to_be_bytes)
        .collect();
    let mut out = ((words.len() + body.len()) as u32).to_be_bytes().to_vec();
    out.extend(words);
    out.extend(body);
    out
}

#[test]
fn compact_integers_match_the_dll_writer() {
    for (value, bytes) in [
        (0, vec![0x00]),
        (1, vec![0x01]),
        (15, vec![0x0F]),
        (16, vec![0x20, 0x01]),
        (81, vec![0x21, 0x05]),
        (87, vec![0x27, 0x05]),
        (411, vec![0x2B, 0x19]),
        (414, vec![0x2E, 0x19]),
        (1314, vec![0x22, 0x52]),
        (-1, vec![0x10]),
        (-2, vec![0x11]),
        (i32::MAX, vec![0x8F, 0xFF, 0xFF, 0xFF, 0x07]),
        (i32::MIN, vec![0x9F, 0xFF, 0xFF, 0xFF, 0x07]),
    ] {
        assert_eq!(int(value), bytes, "{value}");
        let mut reader = Reader::new(&bytes);
        assert_eq!(reader.int().unwrap(), value);
        assert!(reader.finished());
    }
    for value in (-70_000..70_000).step_by(7) {
        assert_eq!(Reader::new(&int(value)).int().unwrap(), value);
    }
    // Five following bytes never come from the writer; the reader refuses them.
    assert!(Reader::new(&[0xA0, 1, 2, 3, 4, 5]).int().is_err());
    assert!(Reader::new(&[0x21]).int().is_err(), "truncated");
}

#[test]
fn strings_carry_their_nul_and_a_short_one_is_refused() {
    let mut out = Vec::new();
    card_service::put_string(&mut out, "ab");
    assert_eq!(out, vec![0x02, b'a', b'b', 0x00]);
    assert_eq!(Reader::new(&out).string().unwrap(), "ab");
    assert!(Reader::new(&[0x02, b'a', b'b']).string().is_err());
    assert!(Reader::new(&[0x10]).string().is_err(), "negative length");
}

#[test]
fn the_gateway_request_is_unrouted_and_answered_with_this_service() {
    let body = gateway_request();
    let bytes = frame(None, &body);
    let (used, parsed) = client_frame(&bytes).unwrap().unwrap();
    assert_eq!(used, bytes.len());
    assert_eq!(parsed.routing, None);
    let request = ConnectionServer::decode(parsed.body).unwrap();
    assert_eq!(
        (
            request.session_id.as_str(),
            request.username.as_str(),
            request.error
        ),
        ("w-ticket", "tester", 87)
    );

    let answer = reply(parsed.body, "127.0.0.1", 17782, |_| None)
        .unwrap()
        .unwrap();
    let length = u32::from_be_bytes(answer[..4].try_into().unwrap()) as usize;
    assert_eq!(
        length,
        answer.len() - 4,
        "server frames carry no routing words"
    );
    let decoded = ConnectionServer::decode(&answer[4..]).unwrap();
    assert_eq!(
        decoded,
        ConnectionServer {
            session_id: "w-ticket".into(),
            username: "tester".into(),
            host: "127.0.0.1".into(),
            port: 17782,
            ok: true,
            error: 0,
        }
    );
}

/// The 81 reply read the way `10179c70` reads it.
fn read_results(body: &[u8]) -> (bool, i32, i32, String) {
    let mut reader = Reader::new(body);
    let version = reader.headers(card_service::LOGIN_RESULTS).unwrap();
    assert_eq!(version, 5);
    let success = reader.int().unwrap() != 0;
    let code = reader.int().unwrap(); // version >= 3
    let account = reader.int().unwrap();
    reader.int().unwrap(); // version > 1
    reader.int().unwrap(); // version > 3
    let session = reader.string().unwrap(); // version > 4
    assert!(reader.finished(), "no zlib blob below version 6");
    (success, code, account, session)
}

#[test]
fn a_routed_session_login_is_accepted_only_for_a_known_session() {
    let body = session_login("w-ticket");
    let bytes = frame(Some([1_000_000_000, 0, 0]), &body);
    let (used, parsed) = client_frame(&bytes).unwrap().unwrap();
    assert_eq!(used, bytes.len());
    assert_eq!(parsed.routing, Some([1_000_000_000, 0, 0]));
    let login = SessionLogin::decode(parsed.body).unwrap();
    assert_eq!(login.session_id, "w-ticket");
    assert_eq!(login.data_versions, vec![(1, 1314), (2, 7), (4, 9)]);
    assert_eq!(login.strings.last().map(String::as_str), Some("STATION"));
    assert_eq!(login.strings.len(), 8);

    let accept = |session: &str| (session == "w-ticket").then_some(42);
    let answer = reply(parsed.body, "127.0.0.1", 17782, accept)
        .unwrap()
        .unwrap();
    assert_eq!(read_results(&answer[4..]), (true, 0, 42, "w-ticket".into()));

    let refused = reply(&session_login("someone-else"), "127.0.0.1", 17782, accept)
        .unwrap()
        .unwrap();
    assert_eq!(read_results(&refused[4..]), (false, 1, 0, String::new()));
}

#[test]
fn partial_oversized_unknown_and_malformed_frames() {
    let bytes = frame(None, &gateway_request());
    assert_eq!(client_frame(&bytes[..3]).unwrap(), None);
    assert_eq!(client_frame(&bytes[..bytes.len() - 1]).unwrap(), None);
    let huge = ((MAX_FRAME + 1) as u32).to_be_bytes();
    assert!(client_frame(&huge).is_err());
    assert!(
        client_frame(&frame(None, &[0x01])).is_err(),
        "no routing words"
    );

    // A command this stub does not answer yet (92 LobbyCommand_Join) is left to the caller.
    let lobby = headers(&[0x2C, 0x05], 1);
    assert_eq!(reply(&lobby, "h", 1, |_| None).unwrap(), None);
    // A 411 with the wrong class id at a later level, or trailing bytes, is refused.
    let mut mixed = session_login("w-ticket");
    mixed[3] = 0x2E;
    assert!(SessionLogin::decode(&mixed).is_err());
    let mut trailing = session_login("w-ticket");
    trailing.push(0);
    assert!(SessionLogin::decode(&trailing).is_err());
}
