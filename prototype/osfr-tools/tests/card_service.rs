//! T763/T764: the TCG service wire against byte fixtures built in the DLL writers' field order
//! (lab/content/tcg/analysis/lead-login-decode-2026-10-26.md), not from this crate's encoder,
//! and the one-time launch tickets the session login is checked against.
use osfr_tools::card_service::{
    self, ConnectionServer, LaunchTickets, MAX_FRAME, MAX_TICKETS, Reader, SessionLogin,
    client_frame, reply,
};
#[path = "support/card_packets.rs"]
mod card_packets;
use card_packets::{frame, gateway_request, headers, int, read_results, session_login, text};

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

const CHARACTER: u64 = 123_456_789;
const TTL_MS: u64 = 120_000;

/// The login, its ticket checked against the store, answered the way a listener would.
fn login_with(tickets: &mut LaunchTickets, character_id: &str, ticket: &str, now: u64) -> Vec<u8> {
    let body = session_login(character_id, ticket);
    let check = |login: &SessionLogin| {
        tickets
            .redeem(&login.ticket, &login.character_id, now)
            .and_then(Result::ok)
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
    assert_eq!(tickets.redeem(&ticket, "", 1), Some(Ok(CHARACTER)));
    assert_eq!(tickets.redeem(&ticket, "", 2), None, "single use");
    // A non-empty id must be the owner's decimal, and a refusal leaves the ticket usable.
    let ticket = tickets.mint(CHARACTER, 0).unwrap().to_string();
    for bad in [" ", "Tester", "12x", "+123456789"] {
        assert_eq!(
            tickets.redeem(&ticket, bad, 1),
            Some(Err("character_mismatch")),
            "{bad:?}"
        );
        assert_eq!(tickets.outstanding(), 1, "a refusal changes nothing");
    }
    assert_eq!(tickets.redeem(&ticket, "", 2), Some(Ok(CHARACTER)));
    let ticket = tickets.mint(CHARACTER, 0).unwrap().to_string();
    assert_eq!(tickets.redeem(&ticket, "", TTL_MS), Some(Err("expired")));
    assert_eq!(tickets.redeem("", "", 1), Some(Err("no_ticket_field")));
}

#[test]
fn a_wrong_character_expired_replaced_or_malformed_ticket_is_refused() {
    let mut tickets = LaunchTickets::new(TTL_MS);
    let character = CHARACTER.to_string();

    // A refused login changes nothing: after the wrong character, the right one still redeems.
    let ticket = tickets.mint(CHARACTER, 0).unwrap().to_string();
    assert_eq!(
        tickets.redeem(&ticket, "5", 1),
        Some(Err("character_mismatch"))
    );
    assert_eq!(tickets.outstanding(), 1, "the ticket was not spent");
    assert_eq!(tickets.redeem(&ticket, &character, 2), Some(Ok(CHARACTER)));

    // Expired: refused without spending; the next mint sweeps the dead ticket.
    let ticket = tickets.mint(CHARACTER, 0).unwrap().to_string();
    assert_eq!(
        tickets.redeem(&ticket, &character, TTL_MS),
        Some(Err("expired"))
    );
    assert_eq!(tickets.outstanding(), 1);

    // A second launch replaces the first ticket.
    let first = tickets.mint(CHARACTER, 0).unwrap().to_string();
    let second = tickets.mint(CHARACTER, 1).unwrap().to_string();
    assert_eq!(tickets.outstanding(), 1);
    assert_eq!(tickets.redeem(&first, &character, 2), None);
    assert_eq!(tickets.redeem(&second, &character, 3), Some(Ok(CHARACTER)));

    // Malformed decimals never match, and do not spend a real ticket.
    let ticket = tickets.mint(CHARACTER, 0).unwrap();
    assert_eq!(
        tickets.redeem("", &character, 1),
        Some(Err("no_ticket_field"))
    );
    for bad in [
        format!("+{ticket}"),
        format!(" {ticket}"),
        format!("{ticket}x"),
        "18446744073709551616".into(),
    ] {
        assert_eq!(
            tickets.redeem(&bad, &character, 1),
            Some(Err("ticket_not_decimal")),
            "{bad:?}"
        );
    }
    assert_eq!(tickets.outstanding(), 1);
    assert_eq!(
        tickets.redeem(&ticket.to_string(), &character, 2),
        Some(Ok(CHARACTER))
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
        collection: Vec::new(),
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

/// T825: the collection sync bodies, spelled out in the client readers' order (account 7; the
/// starter: archetype 900001 twice, archetype 5 once). Compact ids: 83 = `23 05`, 84 = `24 05`,
/// 85 = `25 05`, 86 = `26 05`, `InstanceID` 350 = `2E 15`; 900001 = `41 BA DB`. Each instance
/// reads like the first one of the client's own `tcg/data/collections/default.cln`
/// (`2E 15 01 41 BA DB 00 00`: class, version, archetype, series 0, instance number 0).
#[test]
fn the_collection_sync_bodies_match_the_client_readers() {
    use osfr_tools::card_game::StarterCard;
    let cards = [
        StarterCard {
            archetype_id: 900001,
            copies: 2,
        },
        StarterCard {
            archetype_id: 5,
            copies: 1,
        },
    ];
    let modid = int(card_service::collection_mod_id(&cards));
    // 83 Start v3 (`101da040`): headers, instance count 3, ModID, account, 0.
    assert_eq!(
        card_service::sync_start(7, &cards),
        [
            &headers(&[0x23, 0x05], 3)[..],
            &[0x03],
            &modid,
            &[0x07, 0x00]
        ]
        .concat()
    );
    // 84 StartInstances v2 (`101dadf0`; the writer `101dae80` labels the third field
    // "Archetype Count"): headers, account, type 1 (online), 2 archetypes, ModID. The count is
    // the archetype count, not the 3 instances.
    assert_eq!(
        card_service::sync_start_instances(7, &cards),
        [&headers(&[0x24, 0x05], 2)[..], &[0x07, 0x01, 0x02], &modid].concat()
    );
    // 85 SendInstances v2 (`101d94c0`; the writer `101d8d70` labels the field after the base
    // archetype "Archetype Count"): headers, base archetype 0, 2 archetypes, the instance count
    // 3, three instances, no non-tradable ones, delivery 1.
    let instance = |archetype: &[u8], number: u8| {
        [&[0x2E, 0x15, 0x01][..], archetype, &[0x00, number]].concat()
    };
    assert_eq!(
        card_service::sync_send_instances(&cards),
        [
            &headers(&[0x25, 0x05], 2)[..],
            &[0x00, 0x02, 0x03],
            &instance(&[0x41, 0xBA, 0xDB], 0),
            &instance(&[0x41, 0xBA, 0xDB], 1),
            &instance(&[0x05], 0),
            &[0x00, 0x01],
        ]
        .concat()
    );
    // 86 Complete v2 (`101d77c0`): headers, delivery 1.
    assert_eq!(
        card_service::sync_complete(),
        [&headers(&[0x26, 0x05], 2)[..], &[0x01]].concat()
    );
}

/// T825: the ModID is stable, positive and never 0, and moves when the starter changes.
#[test]
fn the_collection_mod_id_follows_the_starter() {
    use osfr_tools::card_game::StarterCard;
    let one = [StarterCard {
        archetype_id: 100009946,
        copies: 2,
    }];
    let other = [StarterCard {
        archetype_id: 100009946,
        copies: 3,
    }];
    let id = card_service::collection_mod_id(&one);
    assert!(id > 0);
    assert_eq!(id, card_service::collection_mod_id(&one));
    assert_ne!(id, card_service::collection_mod_id(&other));
    assert!(card_service::collection_mod_id(&[]) > 0);
}

/// T825: the client's status (137 v1: LobbyCommand level, status) comes back for the account the
/// server names; the client's Join request yields its group and role.
#[test]
fn the_client_status_and_join_request_decode() {
    let status = [&headers(&[0x29, 0x08], 1)[..], &[0x0C, 0x07]].concat();
    assert_eq!(
        card_service::change_status(5, &status).unwrap(),
        [&headers(&[0x29, 0x08], 1)[..], &[0x05, 0x07]].concat()
    );
    assert!(card_service::change_status(5, &[status.clone(), vec![0]].concat()).is_err());
    // A request as the client's Join writer (`101638d0`) builds it: account, group 1, role 3,
    // empty string, empty list, empty name, no property set (1), 0.
    let join = [
        &headers(&[0x2C, 0x05], 4)[..],
        &[0x07, 0x01, 0x03],
        &text(""),
        &[0x00],
        &text(""),
        &[0x01, 0x00],
    ]
    .concat();
    assert_eq!(card_service::join_request(&join).unwrap(), (1, 3));
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
