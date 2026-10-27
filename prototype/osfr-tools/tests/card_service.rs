//! T763/T764: the TCG service wire against byte fixtures built in the DLL writers' field order
//! (lab/content/tcg/analysis/lead-login-decode-2026-10-26.md), not from this crate's encoder,
//! and the one-time launch tickets the session login is checked against.
use osfr_tools::card_service::{
    self, ConnectionServer, LaunchTickets, MAX_FRAME, MAX_TICKETS, Reader, SessionLogin,
    UNKNOWN_TICKET, client_frame, put_int, reply,
};

fn int(value: i32) -> Vec<u8> {
    let mut out = Vec::new();
    put_int(&mut out, value);
    out
}

/// A client string as `10097b40` writes it: length, text, NUL. The compact length is spelled
/// out by hand: one byte below 16, else `0x20 | low nibble` and the rest.
fn text(value: &str) -> Vec<u8> {
    let length = value.len();
    assert!(length < 256, "fixture strings stay short");
    let prefix = if length < 16 {
        vec![length as u8]
    } else {
        vec![0x20 | (length & 0xF) as u8, (length >> 4) as u8]
    };
    [prefix, value.as_bytes().to_vec(), vec![0]].concat()
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

/// The client's 411 at version 8 (`103c9140` -> `1017b700`), fields in the writer's order with
/// the names from `1017b4b0`.
fn session_login(character_id: &str, ticket: &str) -> Vec<u8> {
    [
        headers(&[0x2B, 0x19], 8),
        text("w-session"), // SessionID (the game's --sessionID)
        // {1: 1314, 2: 7, 4: 9}: count 3, then key/value pairs.
        vec![0x03, 0x01, 0x22, 0x52, 0x02, 0x07, 0x04, 0x09],
        text("winXP"),  // Platform
        text(""),       // Match Challenge Key
        vec![0x01],     // Is Parent Session ID (the exe passes --host=fr)
        text("FR"),     // Login Client
        text("en_US"),  // Locale
        text("Tester"), // CharacterName
        text(character_id),
        text(ticket),
        text("STATION"), // Login Namespace
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

    let answer = reply(parsed.body, "127.0.0.1", 17782, false, |_| None)
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

const CHARACTER: u64 = 123_456_789;
const TTL_MS: u64 = 120_000;

/// The login, its ticket checked against the store, answered the way a listener would.
fn login_with(tickets: &mut LaunchTickets, character_id: &str, ticket: &str, now: u64) -> Vec<u8> {
    let body = session_login(character_id, ticket);
    let check = |login: &SessionLogin| {
        tickets
            .redeem(&login.ticket, &login.character_id, now)
            .ok()
            .map(|_| 42)
    };
    reply(&body, "127.0.0.1", 17782, false, check)
        .unwrap()
        .unwrap()
}

#[test]
fn a_routed_session_login_names_every_field() {
    let body = session_login("123456789", "987654321");
    let bytes = frame(Some([1_000_000_000, 0, 0]), &body);
    let (used, parsed) = client_frame(&bytes).unwrap().unwrap();
    assert_eq!(used, bytes.len());
    assert_eq!(parsed.routing, Some([1_000_000_000, 0, 0]));
    assert_eq!(
        SessionLogin::decode(parsed.body).unwrap(),
        SessionLogin {
            version: 8,
            session_id: "w-session".into(),
            data_versions: vec![(1, 1314), (2, 7), (4, 9)],
            platform: "winXP".into(),
            match_challenge_key: String::new(),
            is_parent_session: true,
            login_client: "FR".into(),
            locale: "en_US".into(),
            character_name: "Tester".into(),
            character_id: "123456789".into(),
            ticket: "987654321".into(),
            namespace: "STATION".into(),
        }
    );
}

#[test]
fn a_launch_ticket_logs_its_own_character_in_once() {
    let mut tickets = LaunchTickets::new(TTL_MS);
    let ticket = tickets.mint(CHARACTER, 1_000).unwrap();
    assert_ne!(ticket, 0);
    let character = CHARACTER.to_string();
    let answer = login_with(&mut tickets, &character, &ticket.to_string(), 2_000);
    assert_eq!(
        read_results(&answer[4..]),
        (true, 0, 42, "w-session".into())
    );
    // Single use: the same ticket again is refused, with message 63 "Your session is not valid."
    let again = login_with(&mut tickets, &character, &ticket.to_string(), 3_000);
    assert_eq!(read_results(&again[4..]), (false, 63, 0, String::new()));
    assert_eq!(tickets.outstanding(), 0);
}

#[test]
fn a_refused_card_login_changes_nothing_and_the_next_one_still_logs_in() {
    // The real 411 path: a name-valued CharacterID (the client's `--character` value, never its
    // guid) is refused without spending the launch ticket, so the correct login still works.
    let mut tickets = LaunchTickets::new(TTL_MS);
    let ticket = tickets.mint(CHARACTER, 1_000).unwrap().to_string();
    let refused = login_with(&mut tickets, "Tester", &ticket, 2_000);
    assert_eq!(read_results(&refused[4..]), (false, 63, 0, String::new()));
    assert_eq!(tickets.outstanding(), 1, "the refusal spent nothing");
    let accepted = login_with(&mut tickets, "", &ticket, 2_500);
    assert_eq!(
        read_results(&accepted[4..]),
        (true, 0, 42, "w-session".into())
    );
    let again = login_with(&mut tickets, "", &ticket, 3_000);
    assert_eq!(
        read_results(&again[4..]),
        (false, 63, 0, String::new()),
        "single use"
    );
}

#[test]
fn the_retail_login_without_a_character_id_redeems_by_ticket_alone() {
    // Native 2026-10-27: the exe passes `--character=<name> --ticket=<n>` and no `--characterID`
    // (the exe's only argument builders are 0x10D2240, 0x10D2160, 0x10D21D0, 0x10D22B0 and
    // 0x10D2450; its strings have no `--characterID`), so the 411's CharacterID is empty.
    let mut tickets = LaunchTickets::new(TTL_MS);
    let ticket = tickets.mint(CHARACTER, 0).unwrap().to_string();
    assert_eq!(tickets.redeem(&ticket, "", 1), Ok(CHARACTER));
    assert_eq!(
        tickets.redeem(&ticket, "", 2),
        Err(UNKNOWN_TICKET),
        "single use"
    );
    // A non-empty id must be the owner's decimal, and a refusal leaves the ticket usable.
    let ticket = tickets.mint(CHARACTER, 0).unwrap().to_string();
    for bad in [" ", "Tester", "12x", "+123456789"] {
        assert_eq!(
            tickets.redeem(&ticket, bad, 1),
            Err("character_mismatch"),
            "{bad:?}"
        );
        assert_eq!(tickets.outstanding(), 1, "a refusal changes nothing");
    }
    assert_eq!(tickets.redeem(&ticket, "", 2), Ok(CHARACTER));
    let ticket = tickets.mint(CHARACTER, 0).unwrap().to_string();
    assert_eq!(tickets.redeem(&ticket, "", TTL_MS), Err("expired"));
    assert_eq!(tickets.redeem("", "", 1), Err("no_ticket_field"));
}

#[test]
fn a_wrong_character_expired_replaced_or_malformed_ticket_is_refused() {
    let mut tickets = LaunchTickets::new(TTL_MS);
    let character = CHARACTER.to_string();

    // A refused login changes nothing: after the wrong character, the right one still redeems.
    let ticket = tickets.mint(CHARACTER, 0).unwrap().to_string();
    assert_eq!(tickets.redeem(&ticket, "5", 1), Err("character_mismatch"));
    assert_eq!(tickets.outstanding(), 1, "the ticket was not spent");
    assert_eq!(tickets.redeem(&ticket, &character, 2), Ok(CHARACTER));

    // Expired: refused without spending; the next mint sweeps the dead ticket.
    let ticket = tickets.mint(CHARACTER, 0).unwrap().to_string();
    assert_eq!(tickets.redeem(&ticket, &character, TTL_MS), Err("expired"));
    assert_eq!(tickets.outstanding(), 1);

    // A second launch replaces the first ticket.
    let first = tickets.mint(CHARACTER, 0).unwrap().to_string();
    let second = tickets.mint(CHARACTER, 1).unwrap().to_string();
    assert_eq!(tickets.outstanding(), 1);
    assert_eq!(tickets.redeem(&first, &character, 2), Err(UNKNOWN_TICKET));
    assert_eq!(tickets.redeem(&second, &character, 3), Ok(CHARACTER));

    // Malformed decimals never match, and do not spend a real ticket.
    let ticket = tickets.mint(CHARACTER, 0).unwrap();
    assert_eq!(tickets.redeem("", &character, 1), Err("no_ticket_field"));
    for bad in [
        format!("+{ticket}"),
        format!(" {ticket}"),
        format!("{ticket}x"),
        "18446744073709551616".into(),
    ] {
        assert_eq!(
            tickets.redeem(&bad, &character, 1),
            Err("ticket_not_decimal"),
            "{bad:?}"
        );
    }
    assert_eq!(tickets.outstanding(), 1);
    assert_eq!(
        tickets.redeem(&ticket.to_string(), &character, 2),
        Ok(CHARACTER)
    );

    // Expired tickets are dropped when a new one is minted.
    tickets.mint(1, 0).unwrap();
    tickets.mint(2, 0).unwrap();
    tickets.mint(3, TTL_MS + 1).unwrap();
    assert_eq!(tickets.outstanding(), 1);
}

/// T808: the account id a success carries is never 0 (the client's handler writes through a null
/// player for 0), positive, and the identity for the store's sequential guids.
#[test]
fn every_character_gets_a_non_zero_account_id() {
    for (guid, account) in [(1, 1), (5041, 5041), (0x7fff_fffe, 0x7fff_fffe)] {
        assert_eq!(card_service::account_id(guid), account);
    }
    for guid in [0, 0x7fff_ffff, 1 << 31, 1 << 32, (1 << 40) | 5041, u64::MAX] {
        let account = card_service::account_id(guid);
        assert!(account > 0, "{guid:#x} -> {account}");
    }
}

/// T808: the client's ping (writer `10190020`: one header, then the spoof value) is echoed once
/// logged in, byte for byte as the client's reader `1018ffe0` reads it; before login it is left
/// to the caller.
/// T813: the lobby bodies, spelled out field by field in the client readers' order (account 7,
/// group 12, type 5, role 2, name "Tester"). Compact ids: 94 = `2E 05`, 92 = `2C 05`, 291 =
/// `23 12`, Lobby 103 = `27 06`, PropertySet 23 = `27 01`.
#[test]
fn the_lobby_bodies_match_the_client_readers() {
    let lobby = osfr_tools::card_game::Lobby {
        group_id: 12,
        lobby_type: 5,
        role: 2,
    };
    let [groups, join, display] = card_service::lobby_entry(7, &lobby, "Tester");
    // 94 v1 (`101583f0`): headers, account, one group (`10158350`), then the `Lobby` (`10155710`):
    // its header (103 v3), five absent property sets, five empty role lists, containing group 0,
    // group, type, no deck, tournament round 0, sub-state 0.
    let lobby_object = [
        &[0x01][..],
        &[0x27, 0x06, 0x03],
        &[0x01; 5],
        &[0x00; 5],
        &[0x00, 0x0C, 0x05, 0x01, 0x00, 0x00],
    ];
    assert_eq!(
        groups,
        [
            &headers(&[0x2E, 0x05], 1)[..],
            &[0x07],
            &lobby_object.concat()
        ]
        .concat()
    );
    // 92 v4 (`10163b20`): headers, account, group, role, empty string, empty int list, the name,
    // a present (0) empty `PropertySet` (23 v1, no entries), the trailing int.
    assert_eq!(
        join,
        [
            &headers(&[0x2C, 0x05], 4)[..],
            &[0x07, 0x0C, 0x02],
            &text(""),
            &[0x00],
            &text("Tester"),
            &[0x00, 0x27, 0x01, 0x01, 0x00],
            &[0x00],
        ]
        .concat()
    );
    // 291 v1 (`1015e5f0`): headers, account, group.
    assert_eq!(
        display,
        [&headers(&[0x23, 0x12], 1)[..], &[0x07, 0x0C]].concat()
    );
}

#[test]
fn a_ping_is_echoed_only_after_login() {
    // Compact 112 = `20 07`, version `01`, spoof 1234 = `22 4D`.
    let ping = [0x20, 0x07, 0x01, 0x22, 0x4D];
    let echo = reply(&ping, "h", 1, true, |_| None).unwrap().unwrap();
    assert_eq!(echo, [&[0, 0, 0, 5][..], &ping].concat());
    assert_eq!(reply(&ping, "h", 1, false, |_| None).unwrap(), None);
    // A ping the client's reader would not read the same way is an error, never echoed.
    assert!(reply(&[0x20, 0x07, 0x01, 0x22, 0x4D, 0], "h", 1, true, |_| None).is_err());
    assert!(reply(&[0x20, 0x07, 0x01], "h", 1, true, |_| None).is_err());
}

#[test]
fn a_full_ticket_table_refuses_and_the_next_mint_sweeps_expired_ones() {
    let mut tickets = LaunchTickets::new(TTL_MS);
    for character in 1..=MAX_TICKETS as u64 {
        tickets.mint(character, 0).unwrap();
    }
    assert_eq!(tickets.outstanding(), MAX_TICKETS);
    let error = tickets.mint(MAX_TICKETS as u64 + 1, 1).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::Other);
    assert_eq!(
        tickets.outstanding(),
        MAX_TICKETS,
        "a refusal changes nothing"
    );
    // At the lifetime boundary the sweep makes room again.
    assert_ne!(tickets.mint(MAX_TICKETS as u64 + 1, TTL_MS).unwrap(), 0);
    assert_eq!(tickets.outstanding(), 1);
}

#[test]
fn the_ticket_store_debug_never_names_a_ticket() {
    let mut tickets = LaunchTickets::new(TTL_MS);
    let ticket = tickets.mint(CHARACTER, 0).unwrap();
    let text = format!("{tickets:?}");
    assert!(text.contains("outstanding: 1"), "{text}");
    assert!(!text.contains(&ticket.to_string()), "{text}");
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
    assert_eq!(reply(&lobby, "h", 1, true, |_| None).unwrap(), None);
    // A 411 with the wrong class id at a later level, or trailing bytes, is refused.
    let mut mixed = session_login("1", "2");
    mixed[3] = 0x2E;
    assert!(SessionLogin::decode(&mixed).is_err());
    let mut trailing = session_login("1", "2");
    trailing.push(0);
    assert!(SessionLogin::decode(&trailing).is_err());
}
