//! W156 (Unrouted Client Requests brief #14): while player trade is not offered, every trade
//! request a client can still send is answered with the native close lifecycle and changes
//! nothing. Expected bytes are the sub 5 / sub 15 layouts of `notes/Build Guides/Gameplay/Trade.md`.

use osfr_tools::trade::unavailable_replies;

const SESSION: u32 = 0x0102_0304;
// Values that must never be echoed back: a counter, an item id and a coin amount.
const CANARY: u32 = 0x5EC2_E7ED;

fn request(sub: u8, session: u32, body: &[u8]) -> Vec<u8> {
    let mut out = vec![0x3b, 0, sub];
    out.extend_from_slice(&session.to_le_bytes());
    out.extend_from_slice(body);
    out
}

fn end(session: u32) -> Vec<u8> {
    request(5, session, &[])
}

const CANCELLED: [u8; 4] = [0x3b, 0, 15, 1];

/// One well-formed request per sub, every body field carrying the canary.
fn canary_requests() -> Vec<(u8, Vec<u8>)> {
    let c = CANARY.to_le_bytes();
    vec![
        (3, request(3, SESSION, &[1])),
        (6, request(6, SESSION, &c)),
        (7, request(7, SESSION, &[])),
        (8, request(8, SESSION, &[c.as_slice(), &[1]].concat())),
        (9, request(9, SESSION, &[c, c].concat())),
        (10, request(10, SESSION, &c)),
    ]
}

#[test]
fn accepted_invite_reply_is_closed_with_sub_5_only() {
    // The client keeps the session id after accepting; only sub 5 clears it.
    assert_eq!(
        unavailable_replies(&request(3, SESSION, &[1])),
        vec![end(SESSION)]
    );
    // Any nonzero accepted byte is an acceptance, like the friend response (R).
    assert_eq!(
        unavailable_replies(&request(3, SESSION, &[0xff])),
        vec![end(SESSION)]
    );
}

#[test]
fn declined_invite_reply_needs_no_answer() {
    assert!(unavailable_replies(&request(3, SESSION, &[0])).is_empty());
}

#[test]
fn cancel_closes_the_window_with_sub_5() {
    assert_eq!(
        unavailable_replies(&request(7, SESSION, &[])),
        vec![end(SESSION)]
    );
}

#[test]
fn accept_lock_unlock_offer_item_and_offer_coins_get_status_then_close() {
    let c = CANARY.to_le_bytes();
    let cases = [
        request(6, SESSION, &c),
        request(8, SESSION, &[c.as_slice(), &[1]].concat()),
        request(8, SESSION, &[c.as_slice(), &[0]].concat()),
        request(9, SESSION, &[c, c].concat()),
        request(10, SESSION, &c),
    ];
    for input in cases {
        assert_eq!(
            unavailable_replies(&input),
            vec![CANCELLED.to_vec(), end(SESSION)],
            "{input:02x?}"
        );
    }
}

#[test]
fn replies_carry_only_the_session_id_never_request_fields() {
    for (sub, input) in canary_requests() {
        let canary = CANARY.to_le_bytes();
        let replies = unavailable_replies(&input);
        assert!(!replies.is_empty(), "sub {sub} is a valid request");
        for reply in replies {
            assert!(reply.len() <= 7, "sub {sub}");
            assert!(
                !reply.windows(4).any(|w| w == canary),
                "sub {sub} echoed a request field"
            );
            assert_eq!(reply[..2], [0x3b, 0], "sub {sub}");
        }
    }
}

#[test]
fn replays_are_identical_and_stateless() {
    for (sub, input) in canary_requests() {
        let first = unavailable_replies(&input);
        for _ in 0..50 {
            assert_eq!(unavailable_replies(&input), first, "sub {sub}");
        }
    }
}

#[test]
fn the_idle_session_sentinel_gets_no_reply() {
    // 0xFFFFFFFF is the client's "no session" value: closing it would only flicker the UI.
    for (_, input) in canary_requests() {
        let mut idle = input.clone();
        idle[3..7].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(unavailable_replies(&idle).is_empty());
    }
}

#[test]
fn wrong_body_length_gets_no_reply() {
    for (sub, input) in canary_requests() {
        let mut long = input.clone();
        long.push(0);
        assert!(unavailable_replies(&long).is_empty(), "sub {sub} trailing");
        // Every strict prefix, including a missing session id, is refused.
        for len in 0..input.len() {
            assert!(
                unavailable_replies(&input[..len]).is_empty(),
                "sub {sub} prefix {len}"
            );
        }
    }
}

#[test]
fn server_to_client_subs_and_unknown_subs_get_no_reply() {
    // Subs 2, 4, 5, 11-19 are server writers; a client sending them is hostile.
    for sub in (0..=u8::MAX).filter(|s| !matches!(s, 3 | 6..=10)) {
        for body in [&[][..], &[1], &[0; 4], &[0; 5], &[0; 8], &[0xff; 16]] {
            assert!(
                unavailable_replies(&request(sub, SESSION, body)).is_empty(),
                "sub {sub} body {}",
                body.len()
            );
        }
    }
    // Sub 5 is the server's close message, not a request.
    assert!(unavailable_replies(&end(SESSION)).is_empty());
}

#[test]
fn other_families_get_no_reply() {
    for (_, input) in canary_requests() {
        for header in [[0x3a, 0], [0x3c, 0], [0x3b, 1], [26, 0], [0xff, 0xff]] {
            let mut other = input.clone();
            other[..2].copy_from_slice(&header);
            assert!(unavailable_replies(&other).is_empty(), "{header:02x?}");
        }
    }
    assert!(unavailable_replies(&[]).is_empty());
}

#[test]
fn hostile_bytes_never_panic_or_amplify() {
    let mut seed = 0x1234_5678_u32;
    for _ in 0..20_000 {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let len = (seed >> 24) as usize % 24;
        let mut input = vec![0x3b, 0];
        for _ in 0..len {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            input.push((seed >> 16) as u8);
        }
        let replies = unavailable_replies(&input);
        assert!(replies.len() <= 2);
        assert!(replies.iter().all(|r| r.len() <= 7));
    }
}
