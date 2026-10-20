//! Bodies of the W127 directed-field and shared combat-budget pins. The crate root keeps the
//! `#[test]` names because the protected test list matches them by name (F179).

use super::*;

pub(super) fn w127_other_instance_recipient_canary() {
    assert!(field_recipient_matches(true, Some((0, true)), 0));
    assert!(
        !field_recipient_matches(true, Some((7, true)), 0),
        "recipient assertion: another instance received field deltas"
    );
    // The field view path uses this same predicate: either signal alone hides it.
    assert!(
        !field_recipient_matches(true, Some((0, false)), 0),
        "readiness loss alone"
    );
    assert!(
        !field_recipient_matches(false, Some((0, true)), 0),
        "world-load loss alone"
    );
    assert!(
        field_recipient_matches(true, Some((0, true)), 0),
        "readiness restored"
    );
    assert!(!field_recipient_matches(true, None, 0));
}

pub(super) fn shared_combat_budget_cannot_be_refunded_by_staging() {
    let world = SharedWorld::new();
    let clone = world.clone();
    let now = Instant::now();
    for _ in 0..20 {
        assert!(clone.security.lock().unwrap().allow_combat(101, now));
    }
    assert!(!world.security.lock().unwrap().allow_combat(101, now));
    drop(clone);
    assert!(!world.security.lock().unwrap().allow_combat(101, now));
    assert!(world.security.lock().unwrap().allow_combat(202, now));
}

/// Pins the shared combat budget's rate (10/s), burst (20) and key cap
/// (`MAX_SESSIONS`): sixteen busy accounts refuse a seventeenth, 100 ms of idle
/// time returns exactly one token, and two seconds of idle time lets the
/// seventeenth in by evicting the refilled buckets.
pub(super) fn shared_combat_budget_pins_rate_burst_and_session_cap() {
    let world = SharedWorld::new();
    let allow =
        |account: u64, at: Instant| world.security.lock().unwrap().allow_combat(account, at);
    let now = Instant::now();
    for account in 1..=MAX_SESSIONS as u64 {
        assert!(allow(account, now), "account {account}");
    }
    let seventeenth = MAX_SESSIONS as u64 + 1;
    assert!(!allow(seventeenth, now));
    assert_eq!(world.security.lock().unwrap().combat.len(), MAX_SESSIONS);

    // Drain account 1, then 10/s returns exactly one token after 100 ms.
    for _ in 1..20 {
        assert!(allow(1, now));
    }
    let refilled = now + Duration::from_millis(100);
    assert!(allow(1, refilled));
    assert!(!allow(1, refilled));

    // Every bucket that can pay a whole burst is idle and evicted.
    let idle = now + Duration::from_secs(2);
    assert!(allow(seventeenth, idle));
    assert!(world.security.lock().unwrap().combat.len() <= MAX_SESSIONS);
}
