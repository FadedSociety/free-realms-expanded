//! Piece 085 (player trading) tests: the recovered offer/accept/cancel wire and
//! the bounded two-sided trade session.
//!
//! Evidence boundary, read before the assertions:
//!
//! ORIGINAL (first-party client inputs; hashes pinned in the lab bundle):
//! * `decompiled/client/FreeRealms/pseudocode.c`, `FUN_00c6a7b0`: the base trade
//!   packet constructor writes `param_1[1] = 0x3b` (SOE opcode 59) into every
//!   trade packet and takes the trade sub-opcode as its argument.
//! * The same constructor family carries the sub-opcode for all 18 recovered
//!   trade classes (`TradeInvitePacket` = 2 .. `TradeItemRemovedPacket` = 19).
//! * Native field readers pin the payload shape of three inbound packets:
//!   `FUN_00c6c110` (u32 + bool) used for `TradeUpdateLockPacket` (17) with the
//!   first u32 compared against the active session id, `FUN_00c6c180` (three
//!   u32) used for `TradeItemRemovedPacket` (19) with the first u32 compared
//!   against the active session id, and `FUN_00c6ce30` (u32 + u32 + bool) used
//!   for `TradeUpdateAddItemPacket` (12).
//! * `TradeHandler` events in the same binary: `TradeCompleted`,
//!   `TradeCanceledPartnerDisconnected`, `TradeCanceledPartnerOfferInvalid`,
//!   `TradeCanceledYourOfferInvalid`,
//!   `TradeCanceledYourOfferExceedsPartnersMaxStack`,
//!   `TradeCanceledPartnersOfferExceedsYourMaxStack` and
//!   `TradeHandler:OtherAccepted` (shown when the partner's lock frame flips on).
//! * Community cross-check: `OpenSourceFreeRealms` `PacketCodes.cs` at pinned
//!   revision lists the identical `BaseTradePacket = 59` and the identical 18
//!   `BaseTradePackets` rows.
//!
//! RECONSTRUCTED, labelled at each call site: the two-sided session state
//! machine, the offer-revision stale-lock rule, the atomic transfer plan and the
//! disconnect policy. The recovered binary only supplies the wire, the session
//! id staleness check and the event vocabulary.
//!
//! NOT CLAIMED: the full field order of the packets whose native reader was not
//! recovered (invite/start session carry a partner name region at offsets
//! 0x26/0x27/0x28 in the native object), the exact second field of
//! `TradeAcceptPacket`/`TradeLockPacket`, or any native trade run. Those stay
//! gaps.
// The module is not registered in `src/lib.rs` (lead-owned). Including the exact
// source file keeps this target compiling against the real artifact. Integration
// step for the lead: register the module and drop this `#[path]` include.
#[path = "../src/restoration/piece_085.rs"]
#[allow(dead_code)]
mod piece_085;

use piece_085::{
    Inventory, SOE_BASE_TRADE_OPCODE, TradeError, TradeOffer, TradeOutcome, TradePacket,
    TradeSession, TradeSide, TradeUpdateLock, WireFrame,
};

#[test]
fn recovered_trade_opcode_table_matches_the_native_constructors() {
    assert_eq!(SOE_BASE_TRADE_OPCODE, 0x3b);
    assert_eq!(SOE_BASE_TRADE_OPCODE, 59);
    assert_eq!(TradePacket::ALL.len(), 18);

    let rows: Vec<(TradePacket, u8, &str)> = TradePacket::ALL
        .iter()
        .map(|packet| (*packet, packet.code(), packet.name()))
        .collect();
    assert_eq!(
        rows,
        vec![
            (TradePacket::Invite, 2, "TradeInvitePacket"),
            (TradePacket::InviteReply, 3, "TradeInviteReplyPacket"),
            (TradePacket::StartSession, 4, "TradeStartSessionPacket"),
            (TradePacket::EndSession, 5, "TradeEndSessionPacket"),
            (TradePacket::Accept, 6, "TradeAcceptPacket"),
            (TradePacket::Cancel, 7, "TradeCancelPacket"),
            (TradePacket::Lock, 8, "TradeLockPacket"),
            (TradePacket::OfferItem, 9, "TradeOfferItemPacket"),
            (TradePacket::CoinCount, 10, "TradeCoinCountPacket"),
            (
                TradePacket::UpdateCoinCount,
                11,
                "TradeUpdateCoinCountPacket"
            ),
            (TradePacket::UpdateAddItem, 12, "TradeUpdateAddItemPacket"),
            (
                TradePacket::UpdateItemCount,
                13,
                "TradeUpdateItemCountPacket"
            ),
            (TradePacket::UpdateAccept, 14, "TradeUpdateAcceptPacket"),
            (TradePacket::Message, 15, "TradeMessagePacket"),
            (TradePacket::Rejected, 16, "TradeRejectedPacket"),
            (TradePacket::UpdateLock, 17, "TradeUpdateLockPacket"),
            (TradePacket::Confirm, 18, "TradeConfirmPacket"),
            (TradePacket::ItemRemoved, 19, "TradeItemRemovedPacket"),
        ]
    );

    for packet in TradePacket::ALL {
        assert_eq!(TradePacket::from_code(packet.code()), Some(packet));
    }
    assert_eq!(TradePacket::from_code(0), None);
    assert_eq!(TradePacket::from_code(20), None);
}

#[test]
fn offer_wire_round_trips_through_the_recovered_frame() {
    let offer = TradeOffer {
        session: 0x2000_0001,
        item_guid: 1001,
        count: 2,
    };
    let frame = offer.to_frame();
    assert_eq!(frame.packet, TradePacket::OfferItem);
    assert_eq!(frame.body, vec![1, 0, 0, 0x20, 0xe9, 3, 0, 0, 2, 0, 0, 0]);

    let decoded = TradeOffer::from_frame(&frame.encode()).unwrap();
    assert_eq!(decoded, offer);
}

#[test]
fn cancel_and_end_session_carry_the_session_id() {
    let cancel = WireFrame::session(TradePacket::Cancel, 0x2000_0001);
    assert_eq!(cancel.encode(), vec![0x3b, 7, 1, 0, 0, 0x20]);
    assert_eq!(WireFrame::decode(&cancel.encode()).unwrap(), cancel);

    let end = WireFrame::session(TradePacket::EndSession, 0x2000_0001);
    assert_eq!(end.encode(), vec![0x3b, 5, 1, 0, 0, 0x20]);
    assert_eq!(WireFrame::decode(&end.encode()).unwrap(), end);
}

#[test]
fn update_lock_wire_round_trips_and_rejects_a_truncated_frame() {
    let lock = TradeUpdateLock {
        session: 0x2000_0001,
        locked: true,
    };
    assert_eq!(lock.to_frame().encode(), vec![0x3b, 17, 1, 0, 0, 0x20, 1]);
    assert_eq!(
        TradeUpdateLock::from_frame(&lock.to_frame().encode()).unwrap(),
        lock
    );

    assert_eq!(
        TradeUpdateLock::from_frame(&[0x3b, 17, 1, 0, 0, 0x20]).unwrap_err(),
        TradeError::TruncatedField
    );
}

#[test]
fn item_removed_wire_round_trips_three_recovered_fields() {
    let frame = WireFrame {
        packet: TradePacket::ItemRemoved,
        body: vec![1, 0, 0, 0x20, 0xe9, 3, 0, 0, 2, 0, 0, 0],
    };
    let encoded = frame.encode();
    assert_eq!(encoded.len(), 14);
    assert_eq!(WireFrame::decode(&encoded).unwrap(), frame);
}

#[test]
fn frame_decode_rejects_a_foreign_base_opcode_and_an_unknown_sub_opcode() {
    assert_eq!(
        WireFrame::decode(&[0x3c, 9, 0, 0, 0, 0]).unwrap_err(),
        TradeError::WrongBaseOpcode(0x3c)
    );
    assert_eq!(
        WireFrame::decode(&[0x3b, 40, 0, 0, 0, 0]).unwrap_err(),
        TradeError::UnknownPacket(40)
    );
    assert_eq!(
        WireFrame::decode(&[]).unwrap_err(),
        TradeError::TruncatedField
    );
}

#[test]
fn session_commits_both_offers_atomically_on_confirm() {
    let mut host = Inventory::new();
    host.insert(1001, 5);
    host.insert(1002, 1);
    host.set_coins(100);
    let mut guest = Inventory::new();
    guest.insert(2002, 3);
    guest.set_coins(40);

    let mut session = TradeSession::open(0x2000_0001, 11, 22);
    session.offer_item(TradeSide::Host, 1001, 2).unwrap();
    session.offer_coins(TradeSide::Host, 30).unwrap();
    session.offer_item(TradeSide::Guest, 2002, 3).unwrap();
    session.offer_coins(TradeSide::Guest, 10).unwrap();

    // Nothing moves while the offer is still open.
    assert_eq!(host.count_of(1001), 5);
    assert_eq!(guest.count_of(2002), 3);

    session.lock(TradeSide::Host).unwrap();
    session.lock(TradeSide::Guest).unwrap();
    session.confirm(TradeSide::Host).unwrap();
    session.confirm(TradeSide::Guest).unwrap();

    let outcome: TradeOutcome = session.complete(&mut host, &mut guest).unwrap();
    assert_eq!(
        outcome.received_by_host,
        vec![piece_085::ItemStack {
            guid: 2002,
            count: 3
        }]
    );
    assert_eq!(
        outcome.received_by_guest,
        vec![piece_085::ItemStack {
            guid: 1001,
            count: 2
        }]
    );
    assert_eq!(host.count_of(1001), 3);
    assert_eq!(host.count_of(2002), 3);
    assert_eq!(guest.count_of(1001), 2);
    assert_eq!(guest.count_of(2002), 0);
    assert_eq!(host.coins(), 80);
    assert_eq!(guest.coins(), 60);
    assert!(!session.is_open());
}

#[test]
fn confirm_before_both_sides_lock_is_refused() {
    let mut host = Inventory::new();
    host.insert(1001, 1);
    let mut guest = Inventory::new();

    let mut session = TradeSession::open(0x2000_0001, 11, 22);
    session.offer_item(TradeSide::Host, 1001, 1).unwrap();
    session.lock(TradeSide::Host).unwrap();
    // Neither side can confirm while the partner has not locked, regardless of
    // which side asks: the whole offer has to be on the table first.
    assert_eq!(
        session.confirm(TradeSide::Host).unwrap_err(),
        TradeError::NotBothLocked
    );
    assert_eq!(
        session.confirm(TradeSide::Guest).unwrap_err(),
        TradeError::NotBothLocked
    );
    assert_eq!(
        session.complete(&mut host, &mut guest).unwrap_err(),
        TradeError::NotBothConfirmed
    );
    assert_eq!(host.count_of(1001), 1);
}

#[test]
fn changing_an_offer_after_lock_invalidates_the_lock() {
    let mut host = Inventory::new();
    host.insert(1001, 5);
    let mut guest = Inventory::new();
    guest.insert(2002, 1);

    let mut session = TradeSession::open(0x2000_0001, 11, 22);
    session.offer_item(TradeSide::Host, 1001, 1).unwrap();
    session.lock(TradeSide::Host).unwrap();
    session.lock(TradeSide::Guest).unwrap();
    // The host changes the offer after locking; the recovered client hides the
    // partner's confirmation when the lock state changes.
    session.offer_item(TradeSide::Host, 1001, 2).unwrap();
    assert_eq!(
        session.confirm(TradeSide::Guest).unwrap_err(),
        TradeError::OfferChanged
    );
    assert_eq!(
        session.complete(&mut host, &mut guest).unwrap_err(),
        TradeError::OfferChanged
    );
    assert_eq!(host.count_of(1001), 5);
    assert_eq!(guest.count_of(2002), 1);
}

#[test]
fn cancel_and_disconnect_close_the_session_without_moving_items() {
    for cancel in [false, true] {
        let mut host = Inventory::new();
        host.insert(1001, 5);
        let mut guest = Inventory::new();
        guest.insert(2002, 1);

        let mut session = TradeSession::open(0x2000_0001, 11, 22);
        session.offer_item(TradeSide::Host, 1001, 2).unwrap();
        session.offer_item(TradeSide::Guest, 2002, 1).unwrap();
        session.lock(TradeSide::Host).unwrap();
        session.lock(TradeSide::Guest).unwrap();
        session.confirm(TradeSide::Host).unwrap();
        session.confirm(TradeSide::Guest).unwrap();

        if cancel {
            session.cancel(TradeSide::Guest).unwrap();
            assert_eq!(
                session.closed_reason(),
                Some(piece_085::TradeClose::Cancelled)
            );
        } else {
            session.partner_disconnected(TradeSide::Guest).unwrap();
            assert_eq!(
                session.closed_reason(),
                Some(piece_085::TradeClose::PartnerDisconnected)
            );
        }

        assert_eq!(
            session.complete(&mut host, &mut guest).unwrap_err(),
            TradeError::SessionClosed
        );
        assert!(!session.is_open());
        assert_eq!(host.count_of(1001), 5);
        assert_eq!(guest.count_of(2002), 1);
    }
}

#[test]
fn an_offer_the_owner_does_not_have_fails_atomically() {
    let mut host = Inventory::new();
    host.insert(1001, 1);
    let mut guest = Inventory::new();
    guest.insert(2002, 4);

    let mut session = TradeSession::open(0x2000_0001, 11, 22);
    session.offer_item(TradeSide::Host, 1001, 3).unwrap();
    session.offer_item(TradeSide::Guest, 2002, 4).unwrap();
    session.lock(TradeSide::Host).unwrap();
    session.lock(TradeSide::Guest).unwrap();
    session.confirm(TradeSide::Host).unwrap();
    session.confirm(TradeSide::Guest).unwrap();

    assert_eq!(
        session.complete(&mut host, &mut guest).unwrap_err(),
        TradeError::OfferInvalid
    );
    // Neither side moved: the transfer is a plan that only commits whole.
    assert_eq!(host.count_of(1001), 1);
    assert_eq!(host.count_of(2002), 0);
    assert_eq!(guest.count_of(2002), 4);
    assert_eq!(guest.count_of(1001), 0);
}

#[test]
fn a_replayed_complete_does_not_transfer_twice() {
    let mut host = Inventory::new();
    host.insert(1001, 2);
    let mut guest = Inventory::new();
    guest.insert(2002, 2);

    let mut session = TradeSession::open(0x2000_0001, 11, 22);
    session.offer_item(TradeSide::Host, 1001, 2).unwrap();
    session.offer_item(TradeSide::Guest, 2002, 2).unwrap();
    session.lock(TradeSide::Host).unwrap();
    session.lock(TradeSide::Guest).unwrap();
    session.confirm(TradeSide::Host).unwrap();
    session.confirm(TradeSide::Guest).unwrap();
    session.complete(&mut host, &mut guest).unwrap();
    assert_eq!(host.count_of(1001), 0);

    assert_eq!(
        session.complete(&mut host, &mut guest).unwrap_err(),
        TradeError::SessionClosed
    );
    assert_eq!(host.count_of(1001), 0);
    assert_eq!(host.count_of(2002), 2);
    assert_eq!(
        session.closed_reason(),
        Some(piece_085::TradeClose::Completed)
    );
}

#[test]
fn a_lock_frame_for_another_session_is_stale() {
    let mut session = TradeSession::open(0x2000_0001, 11, 22);
    let stale = TradeUpdateLock {
        session: 0x2000_0009,
        locked: true,
    };
    assert_eq!(
        session
            .apply_lock_frame(TradeSide::Guest, &stale)
            .unwrap_err(),
        TradeError::StaleSession {
            expected: 0x2000_0001,
            got: 0x2000_0009
        }
    );
    assert_eq!(session.remote_lock(TradeSide::Guest), false);

    let fresh = TradeUpdateLock {
        session: 0x2000_0001,
        locked: true,
    };
    session.apply_lock_frame(TradeSide::Guest, &fresh).unwrap();
    assert!(session.remote_lock(TradeSide::Guest));
}
