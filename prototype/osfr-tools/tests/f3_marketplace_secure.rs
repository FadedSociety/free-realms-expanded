//! rc-0137 recheck (run 20261112T020333Z), Station Cash: "asks for a zip code and purchases don't
//! go through", and the session then ended on a 112-byte top-level message ("family 55094").
//!
//! That message is the client's secure channel (`secure_channel.rs`): its Marketplace preview
//! (66/1) for bundle 81, sealed with AES-CCM under the login ticket and marked `01 C9A5D4F3`.
//! The gateway now drops any top-level message it cannot read, and the session opens the sealed
//! ones before parsing; the wallet record says state and zip are on file, so the client asks for
//! no billing address.
//!
//! The sealed vectors here were made with OpenSSL's AES-CCM (Python `cryptography`) from the
//! plaintext the native message opens to, under a test key, with the native nonce layout
//! (counter 1, then the run's five random bytes); the 66/3 is the same request as an order
//! (tracking 2, no trailing preview string, counter 2).

#[path = "common/gateway_vendor.rs"]
#[allow(dead_code)] // the shared rig also ships helpers these tests do not call
mod gateway_vendor;
#[path = "common/vendor_fixture.rs"]
#[allow(dead_code)]
mod vendor_fixture;

use gateway_vendor::{connect_with, fixture_root, send};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    secure_channel::{Opened, SecureChannel},
    station_cash::Wallet,
};
use std::path::PathBuf;
use vendor_fixture::VendorFixture;

/// The native message, byte for byte (frame 036700 payload): sealed under that session's ticket.
const NATIVE: &str = "36d79df25f0a3497b7fea21127a6c83f6e63a01c9aa01057491154c07e17bdff13fc3df62e0d2ff1c94a178334be3d2d073ae7beccbd559eee63320f6d353429c1ad064fb4d476662217ec7e2f909fbc79b6d19a34e19e37c764a61e53510100000000000000611aac4c9001c9a5d4f3";
/// What it opens to: tunnel 6, then the 66/1 preview of bundle 81 in store 1 (tracking 1).
const PREVIEW: &str = "0600014700000042000100010000003101000000310100000005000000656e5f55530000000003000000534f45000000000100000051000000010000000100000001000000300000000000000000";
const TEST_KEY: &str = "rc0137-secure-channel-test-key!!";
const SEALED_PREVIEW: &str = "502e50ef1e7ec9cd5ca8d8bdd5172dcbc61ffcd699b1b0c193584c63bcf54f86d63e62a6224570f78996b92c2df29655f7f34d59761e86361f248da6dacb0d98e15b01d29641de97afeeaa65efdedbb5af65d1fed9fd5dabcf250616dda60100000000000000611aac4c9001c9a5d4f3";
const SEALED_ORDER: &str = "c05490fae2d647e83c8711ecde156f07ce2fa40936f4e538d1bbd2a6d5b7cf236b70e89038c71a3ea1fee6b991dd77d65c718a71917cda3aaf1d7def6f02964967b867a1a3c9cb4a95427f21051a11b7954f2767ba462754f83a0200000000000000611aac4c9001c9a5d4f3";
/// The Super Coin booster: 200 SC at the shipped scale of 0.25.
const BUNDLE: i32 = 81;
const PRICE: u32 = 50;
const NOW: u64 = 1_000;

fn hex(text: &str) -> Vec<u8> {
    osfr_tools::trace::parse_hex(text).expect("test hex")
}

/// The client's gateway login (u16 1, the ticket, the guid, the version, one more string).
fn login(ticket: &str, guid: u64) -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    bytes.extend((ticket.len() as u32).to_le_bytes());
    bytes.extend(ticket.as_bytes());
    bytes.extend(guid.to_le_bytes());
    bytes
}

fn opened(channel: &mut SecureChannel, sealed: &str) -> Vec<u8> {
    match channel.open(&hex(sealed)) {
        Opened::Unwrapped(plain) => plain,
        other => panic!("the sealed message opens: {other:?}"),
    }
}

/// The inner request of a tunnel (u16 5/6, a flag byte, a u32 length).
fn inner(tunnel: &[u8]) -> &[u8] {
    let length = u32::from_le_bytes(tunnel[3..7].try_into().unwrap()) as usize;
    assert_eq!(tunnel.len(), 7 + length);
    &tunnel[7..]
}

fn result_of(replies: &[Vec<u8>], subtype: u8) -> u32 {
    let reply = (replies.iter())
        .find(|reply| reply.starts_with(&[66, 0, subtype, 0]))
        .unwrap_or_else(|| panic!("a 66/{subtype} answer"));
    u32::from_le_bytes(reply[8..12].try_into().unwrap())
}

#[test]
fn the_sealed_marketplace_requests_open_under_the_login_ticket_and_the_purchase_goes_through() {
    let mut channel = SecureChannel::default();
    channel.learn(&login(TEST_KEY, 1));
    let preview = opened(&mut channel, SEALED_PREVIEW);
    assert_eq!(preview, hex(PREVIEW), "the native preview, exactly");
    let order = opened(&mut channel, SEALED_ORDER);
    assert_eq!(
        channel.open(&hex(SEALED_ORDER)),
        Opened::Refused("replayed secure counter"),
        "a repeated counter is refused"
    );
    let mut forged = hex(SEALED_ORDER);
    forged[3] ^= 1;
    let mut fresh = SecureChannel::default();
    fresh.learn(&login(TEST_KEY, 1));
    assert_eq!(fresh.open(&forged), Opened::Refused("secure tag mismatch"));

    // The opened requests on the shipped release content: a player with the welcome 50 SC.
    let content = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/release");
    let who = CharacterState {
        guid: 1,
        zone_id: 0,
        position: [-733.952, 9.09, 945.973],
        rotation: [0.0, 0.0, 1.0],
    };
    let mut state = Gameplay::load(&content, None).unwrap().state();
    state.sc_wallet = Some(Wallet { balance: PRICE, welcomed: true, ..Wallet::default() });
    let mut game = Gameplay::load(&content, Some(state)).unwrap();
    game.initialize_world(&who, 500).unwrap();
    let wallet = game.handle_in_world(&[66, 0, 10, 0], &who, 800).unwrap();
    let record = (wallet.replies.iter())
        .find(|reply| reply.starts_with(&[66, 0, 11, 0]))
        .unwrap();
    assert_eq!(record.last(), Some(&1), "state and zip on file: no billing prompt (`00b3e120`)");
    let previewed = game.handle_in_world(inner(&preview), &who, 900).unwrap();
    assert_eq!(result_of(&previewed.replies, 2), 1, "the preview succeeds");
    let boosters = |game: &Gameplay| {
        (game.inventory_state().items.iter())
            .filter(|item| item.definition_id == BUNDLE)
            .map(|item| item.count)
            .sum::<u32>()
    };
    let before = boosters(&game);
    let paid = game.handle_in_world(inner(&order), &who, 1_000).unwrap();
    assert_eq!(result_of(&paid.replies, 4), 1, "the order succeeds");
    assert!(paid.changed, "and is saved");
    assert_eq!(boosters(&game), before + 1, "the booster is in the bag");
}

#[test]
fn the_native_sealed_message_no_longer_ends_the_session() {
    let fixture = VendorFixture::new(&fixture_root(), "f3-secure");
    let mut gateway = connect_with(
        fixture.reference(),
        fixture.character(),
        fixture.content(),
        fixture.state(100, false),
    );
    gateway.set_trace_enabled(true);
    // The session's channel has the run's key only in the run, so here (as for any message the
    // session cannot open) the gateway sees the sealed bytes themselves.
    let outcome = gateway.handle_at(&hex(NATIVE), NOW).expect("not fatal");
    assert!(outcome.replies.is_empty());
    let record = &outcome.trace[0];
    assert_eq!(record.family, Some(55_094), "the run's 'family 55094'");
    assert_eq!(record.outcome, "ignored");
    assert_eq!(record.reason.as_deref(), Some("unknown top-level message"));
    // Any other unknown top-level word, alone or bundled, is dropped the same way.
    for message in [vec![2, 0], vec![0xff, 0xff, 1, 2, 3]] {
        assert!(gateway.handle_at(&message, NOW).unwrap().replies.is_empty());
    }
    // The session goes on: the wallet still answers, with state and zip on file.
    let wallet = send(&mut gateway, &[66, 0, 10, 0]);
    let record = (wallet.iter()).find(|reply| reply.starts_with(&[66, 0, 11, 0])).unwrap();
    assert_eq!(record.last(), Some(&1));
}
