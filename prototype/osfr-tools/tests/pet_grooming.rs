//! T488: pet grooming through the service's two entry points (`PetService::handle_at_ms`, which
//! `Gameplay::handle_pets` calls, and `care_tick`, which `Gameplay::pet_tick` calls): 53/18 starts
//! a timed session (53/20 kind 3, shampoo spent), 53/22 strokes raise the groom bar (or, with no
//! session, play), and a session ends with 53/20 kind 4. The `Gameplay`/gateway wrappers are
//! covered by `tests/pet_wired.rs`.
#[path = "support/pet_care.rs"]
mod care;
#[path = "support/json.rs"]
mod json;
#[path = "support/pet_inventory.rs"]
mod pet_inventory;
#[path = "../src/bin/osfr_gateway/pet_wire.rs"]
mod pet_wire;

use care::*;
use osfr_tools::{
    pets::{PetBusy, PetContent, PetEquipmentSlot, PetRequest, PetService, PetWire},
    world_events::piece_052::animation_packet,
};
use pet_inventory::InventoryFixture;
use pet_wire::{feed, groom, summon, summon_by_item, tactile, water};
use serde_json::Value;
use std::path::Path;

/// 1,000 s after adoption: idle decay has taken the needs from 60 to 50, so a brushing has room.
const START: u64 = 1_000_000;
/// The CRAB item (a vanity Pet Pal, T454).
const CRAB_ITEM: i32 = 7885;
/// Brisk strokes: 300 ms and a movement sum of 15,000 is a speed of 50, the most one message may claim.
const BRISK: (u32, u32) = (300, 15_000);

/// The shipped `care.groom` block.
fn rules() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content/pets.json");
    json::read(&path)["care"]["groom"].clone()
}

fn number(key: &str) -> f64 {
    rules()[key].as_f64().unwrap()
}

/// A summoned dog with the given `(utility slot, item)` fitted, all at `START`.
fn fitted(slots: &[(u32, i32)]) -> (PetService, InventoryFixture, u32) {
    let (mut service, pet) = dog();
    let mut fixture = kit();
    for &(slot, item) in slots {
        send(&mut service, &mut fixture, &fit(pet, slot, item), START);
    }
    send(&mut service, &mut fixture, &summon(pet), START);
    (service, fixture, pet)
}

/// The usual kit: shampoo, conditioner and brush.
fn ready() -> (PetService, InventoryFixture, u32) {
    fitted(&[(1, SHAMPOO_ITEM), (2, CONDITIONER_ITEM), (4, BRUSH_ITEM)])
}

fn stroke(
    service: &mut PetService,
    fixture: &mut InventoryFixture,
    at: u64,
    (duration, movement): (u32, u32),
) -> PetWire {
    send(service, fixture, &tactile(2, duration, movement), at)
}

fn groom_need(service: &PetService, pet: u32) -> f32 {
    service.pet(pet).unwrap().needs.groom
}

/// `wire` is the lone kind-6 unlock and the roster and bag are exactly as before.
fn assert_refused(
    service: &mut PetService,
    fixture: &mut InventoryFixture,
    pet: u32,
    wire: PetWire,
    why: &str,
) {
    assert_eq!(wire.private, vec![notify(pet, 6, 0)], "{why}");
    assert!(!wire.changed, "{why}");
    // No session is running either: the tick never ends one.
    let later = service.care_tick(&mut fixture.inventory, START + 200_000);
    assert!(notify_kinds(&later.private).is_empty(), "{why}");
}

/// `wire` is the lone kind-6 unlock of `pet` and nothing else changed.
fn assert_unlocked(wire: PetWire, pet: u32, why: &str) {
    assert_eq!(wire.private, vec![notify(pet, 6, 0)], "{why}");
    assert!(wire.shared.is_empty() && !wire.changed, "{why}");
}

#[test]
fn groom_and_tactile_requests_decode_with_exact_lengths() {
    assert_eq!(
        PetRequest::decode(&groom(7)).unwrap(),
        Some(PetRequest::Groom { pet_id: 7 })
    );
    assert_eq!(groom(7), words(&[53, 0, 18], &[7, 0]));
    // Reader-derived layout (`00ccdb70`/`00ccea80`): the state word, a constant 3, time, movement.
    assert_eq!(
        tactile(2, 300, 15_000),
        words(&[53, 0, 22], &[2, 3, 300, 15_000])
    );
    assert_eq!(
        PetRequest::decode(&tactile(2, 300, 15_000)).unwrap(),
        Some(PetRequest::Tactile {
            mode: 2,
            duration_ms: 300,
            movement: 15_000,
        })
    );
    for bad in [
        words(&[53, 0, 18], &[7]),
        words(&[53, 0, 18], &[7, 0, 0]),
        vec![53, 0, 18],
        words(&[53, 0, 22], &[2, 3, 300]),
        words(&[53, 0, 22], &[2, 3, 300, 1, 1]),
        vec![53, 0, 22],
    ] {
        assert!(PetRequest::decode(&bad).is_err(), "{bad:?}");
    }
}

#[test]
fn a_groom_sends_kind_3_with_the_configured_time_and_spends_one_shampoo_and_the_conditioner() {
    let (mut service, mut fixture, pet) = ready();
    let seconds = number("seconds") as u32;
    let wire = send(&mut service, &mut fixture, &groom(pet), START + 1_000);
    assert_eq!(notify_kinds(&wire.private), vec![3]);
    assert_eq!(wire.private.last().unwrap(), &notify(pet, 3, seconds));
    assert!(wire.private.len() > 1, "the bag's replies come first");
    assert!(wire.changed, "the spent kit is saved");
    assert_eq!(count(&fixture, SHAMPOO_ITEM), 1, "exactly one shampoo");
    assert_eq!(count(&fixture, CONDITIONER_ITEM), 0, "the one conditioner");
    assert_eq!(count(&fixture, BRUSH_ITEM), 1, "the brush is kept");
    let utilities = &service.pet(pet).unwrap().equipment.utilities;
    assert_eq!(utilities[1], Some(SHAMPOO_ITEM));
    assert_eq!(utilities[2], None, "an emptied stack leaves its slot");
    assert_eq!(utilities[4], Some(BRUSH_ITEM));

    // The conditioner is optional: with none fitted only the shampoo goes.
    let (mut service, mut fixture, pet) = fitted(&[(1, SHAMPOO_ITEM), (4, BRUSH_ITEM)]);
    let wire = send(&mut service, &mut fixture, &groom(pet), START + 1_000);
    assert_eq!(wire.private.last().unwrap(), &notify(pet, 3, seconds));
    assert_eq!(count(&fixture, SHAMPOO_ITEM), 1);
    assert_eq!(
        count(&fixture, CONDITIONER_ITEM),
        1,
        "an unfitted conditioner stays"
    );
}

#[test]
fn strokes_raise_groom_and_a_full_bar_ends_the_session_with_status_dry_off_and_kind_4() {
    let (mut service, mut fixture, pet) = ready();
    let start = START + 1_000;
    send(&mut service, &mut fixture, &groom(pet), start);
    let before = groom_need(&service, pet);
    let mut at = start;
    let mut ended = None;
    for step in 0..200 {
        at += 300;
        let wire = stroke(&mut service, &mut fixture, at, BRISK);
        if step == 2 {
            assert!(
                groom_need(&service, pet) > before + 2.0,
                "strokes raise groom"
            );
            let needs = service.pet(pet).unwrap().needs;
            assert_eq!(needs.mood, (needs.food + needs.groom + needs.play) / 3.0);
        }
        if !wire.private.is_empty() {
            ended = Some(wire);
            break;
        }
        assert!(!wire.changed, "a stroke sends and saves nothing by itself");
    }
    let wire = ended.expect("the bar fills before the time runs out");
    assert!(at - start < number("seconds") as u64 * 1_000);
    // T812: a new pet is not already near full, so a groom is real scrubbing, not a few strokes.
    assert!(at - start >= 10_000, "{} ms", at - start);
    let dry_off = animation_packet(
        PetService::pet_guid(OWNER),
        number("dry_off_animation") as i32,
        false,
    );
    assert_eq!(wire.private.len(), 3);
    assert!(
        wire.private[0].starts_with(&[53, 0, 3]),
        "the final needs first"
    );
    assert_eq!(f32::from_bits(u32_at(&wire.private[0], 11)), 100.0);
    assert_eq!(wire.private[1], dry_off);
    assert_eq!(wire.private[2], notify(pet, 4, 0));
    assert_eq!(wire.shared, vec![dry_off], "viewers see the dry-off too");
    assert!(wire.changed);
    // T812: the session did not just stop; the owner is told it finished.
    assert!(wire.notice.is_some_and(|text| !text.is_empty()));
    assert_eq!(groom_need(&service, pet), 100.0);

    // The session is over: later strokes only unlock a stale client mode and add nothing.
    let roster = service.document_value();
    for extra in 1..=3 {
        let late = stroke(&mut service, &mut fixture, at + extra * 300, BRISK);
        assert_unlocked(late, pet, "no session after the full bar");
    }
    assert_eq!(service.document_value(), roster);
    let tick = service.care_tick(&mut fixture.inventory, at + 120_000);
    assert!(notify_kinds(&tick.private).is_empty() && !tick.changed);
}

#[test]
fn a_timeout_ends_the_session_with_kind_4() {
    let (mut service, mut fixture, pet) = ready();
    let start = START + 1_000;
    send(&mut service, &mut fixture, &groom(pet), start);
    stroke(&mut service, &mut fixture, start + 300, BRISK);
    let time = number("seconds") as u64 * 1_000;
    let early = service.care_tick(&mut fixture.inventory, start + time - 100);
    assert!(notify_kinds(&early.private).is_empty());
    assert!(!early.changed);

    let ended = service.care_tick(&mut fixture.inventory, start + time);
    assert_eq!(ended.private.len(), 3, "status, dry-off, kind 4");
    assert!(ended.private[0].starts_with(&[53, 0, 3]));
    assert_eq!(notify_kinds(&ended.private), vec![4]);
    assert_eq!(ended.private[2], notify(pet, 4, 0));
    assert_eq!(ended.shared.len(), 1, "the dry-off reaches the viewers");
    assert!(ended.changed, "the gains so far are saved");
    assert!(ended.notice.is_none(), "only a full bar claims a clean pet");

    // No further gain, and the end is sent once.
    let roster = service.document_value();
    let late = stroke(&mut service, &mut fixture, start + time + 300, BRISK);
    assert_unlocked(late, pet, "no session after the timeout");
    assert_eq!(service.document_value(), roster);
    let tick = service.care_tick(&mut fixture.inventory, start + time + 5_000);
    assert!(notify_kinds(&tick.private).is_empty() && !tick.changed);
}

#[test]
fn every_groom_refusal_sends_kind_6_and_changes_nothing() {
    let at = START + 1_000;
    type Setup = fn() -> (PetService, InventoryFixture, u32);
    let cases: [(&str, Setup); 3] = [
        ("no brush", || {
            fitted(&[(1, SHAMPOO_ITEM), (2, CONDITIONER_ITEM)])
        }),
        ("no shampoo", || {
            fitted(&[(2, CONDITIONER_ITEM), (4, BRUSH_ITEM)])
        }),
        ("not summoned", || {
            let (mut service, pet) = dog();
            let mut fixture = kit();
            for (slot, item) in [(1, SHAMPOO_ITEM), (4, BRUSH_ITEM)] {
                send(&mut service, &mut fixture, &fit(pet, slot, item), START);
            }
            (service, fixture, pet)
        }),
    ];
    for (why, setup) in cases {
        let (mut service, mut fixture, pet) = setup();
        let (roster, bag) = (service.document_value(), fixture.inventory.state().clone());
        let wire = send(&mut service, &mut fixture, &groom(pet), at);
        assert_refused(&mut service, &mut fixture, pet, wire, why);
        assert_eq!(service.document_value(), roster, "{why}");
        assert_eq!(fixture.inventory.state(), &bag, "{why}");
    }

    // The same kit, refused for what the owner or the pet is doing.
    let (mut service, mut fixture, pet) = ready();
    let (roster, bag) = (service.document_value(), fixture.inventory.state().clone());
    let other = send(&mut service, &mut fixture, &groom(pet + 50), at);
    assert_refused(
        &mut service,
        &mut fixture,
        pet + 50,
        other,
        "another pet's id",
    );
    let busy = service
        .handle_at_ms(
            &groom(pet),
            OWNER,
            [0.0; 3],
            &mut fixture.inventory,
            PetBusy {
                minigame_active: true,
            },
            at,
        )
        .unwrap()
        .unwrap();
    assert_refused(&mut service, &mut fixture, pet, busy, "busy");
    send(&mut service, &mut fixture, &water(true), at);
    let wet = send(&mut service, &mut fixture, &groom(pet), at);
    assert_refused(&mut service, &mut fixture, pet, wet, "in water");
    assert_eq!(service.document_value(), roster);
    assert_eq!(fixture.inventory.state(), &bag);

    // (R) A pet already at the maximum is not shampooed: nothing is wasted. A new pet starts
    // below it (T812), so its saved groom bar is filled and the roster reloaded; no time passes.
    let (mut service, pet) = dog();
    let mut fixture = kit();
    for (slot, item) in [(1, SHAMPOO_ITEM), (4, BRUSH_ITEM)] {
        send(&mut service, &mut fixture, &fit(pet, slot, item), 0);
    }
    let mut document = service.document_value().unwrap();
    document["pets"][0]["needs"]["groom"] = 100.0.into();
    service = PetService::from_document(PetContent::bundled().unwrap(), Some(document)).unwrap();
    send(&mut service, &mut fixture, &summon(pet), 0);
    let (roster, bag) = (service.document_value(), fixture.inventory.state().clone());
    let clean = send(&mut service, &mut fixture, &groom(pet), 0);
    assert_refused(&mut service, &mut fixture, pet, clean, "already clean");
    assert_eq!(service.document_value(), roster);
    assert_eq!(fixture.inventory.state(), &bag);

    // (R) A vanity Pet Pal takes no grooming even with a kit forced into its slots.
    let content = PetContent::bundled().unwrap();
    let (mut service, mut fixture, _) = ready();
    let crab = content.definition_for_item(CRAB_ITEM).unwrap();
    let crab = service.adopt_vendor_grant(2, &crab, 0).unwrap();
    for (slot, item) in [(1, SHAMPOO_ITEM), (4, BRUSH_ITEM)] {
        service
            .equip_item(
                crab,
                item,
                PetEquipmentSlot::Utility(slot),
                &fixture.inventory,
            )
            .unwrap();
    }
    send(&mut service, &mut fixture, &summon(crab), START);
    let (roster, bag) = (service.document_value(), fixture.inventory.state().clone());
    let vain = send(&mut service, &mut fixture, &groom(crab), at);
    assert_refused(&mut service, &mut fixture, crab, vain, "vanity");
    assert_eq!(service.document_value(), roster);
    assert_eq!(fixture.inventory.state(), &bag);
}

#[test]
fn one_session_per_pet_and_a_feed_and_a_groom_exclude_each_other() {
    let at = START + 1_000;
    let all = [
        (1, SHAMPOO_ITEM),
        (4, BRUSH_ITEM),
        (5, BOWL_ITEM),
        (6, FOOD_ITEM),
    ];
    // A second 53/18 during a session, and a feed during it.
    let (mut service, mut fixture, pet) = fitted(&all);
    let first = send(&mut service, &mut fixture, &groom(pet), at);
    assert_eq!(notify_kinds(&first.private), vec![3]);
    assert_eq!(count(&fixture, SHAMPOO_ITEM), 1);
    let (roster, bag) = (service.document_value(), fixture.inventory.state().clone());
    let again = send(&mut service, &mut fixture, &groom(pet), at + 100);
    assert_eq!(again.private, vec![notify(pet, 6, 0)]);
    assert!(!again.changed);
    let feeding = send(&mut service, &mut fixture, &feed(pet), at + 200);
    assert_eq!(feeding.private, vec![notify(pet, 6, 0)]);
    assert_eq!(service.document_value(), roster);
    assert_eq!(fixture.inventory.state(), &bag);
    assert_eq!(count(&fixture, SHAMPOO_ITEM), 1, "one session, one shampoo");

    // A groom during a feed.
    let (mut service, mut fixture, pet) = fitted(&all);
    send(&mut service, &mut fixture, &feed(pet), at);
    let (roster, bag) = (service.document_value(), fixture.inventory.state().clone());
    let wire = send(&mut service, &mut fixture, &groom(pet), at + 100);
    assert_eq!(wire.private, vec![notify(pet, 6, 0)]);
    assert_eq!(service.document_value(), roster);
    assert_eq!(fixture.inventory.state(), &bag);
}

#[test]
fn a_flood_of_1000_petting_messages_in_one_second_raises_play_by_no_more_than_the_rate_limit_allows()
 {
    let (mut service, mut fixture, pet) = fitted(&[]);
    let before = service.pet(pet).unwrap().needs.play;
    let at = START + 1_000;
    for step in 0..1_000 {
        let wire = send(
            &mut service,
            &mut fixture,
            &tactile(0, 300, 5_000),
            at + step,
        );
        assert!(
            wire.private.is_empty() && !wire.changed,
            "petting sends nothing itself"
        );
    }
    // One message per interval counts, the first at once: 1,000 ms at the shipped interval.
    let interval = number("petting_min_interval_ms") as u64;
    let allowed = 1_000_u64.div_ceil(interval) as f64 * number("petting_gain");
    let gained = f64::from(service.pet(pet).unwrap().needs.play - before);
    assert!(
        gained >= number("petting_gain") - 0.1,
        "petting raises play: {gained}"
    );
    assert!(gained <= allowed, "{gained} gained, {allowed} allowed");
    // The live 53/3 carries the new play value to the owner.
    let tick = service.care_tick(&mut fixture.inventory, at + 3_000);
    let status = statuses(&tick.private);
    assert_eq!(status.len(), 1);
    assert!(f64::from(f32::from_bits(u32_at(status[0], 15))) > f64::from(before));
}

#[test]
fn petting_needs_a_free_summoned_pet_and_no_session_and_strokes_need_their_mode() {
    let at = START + 1_000;
    let ignored = |service: &mut PetService, fixture: &mut InventoryFixture, packet: &[u8], at| {
        let roster = service.document_value();
        let wire = send(service, fixture, packet, at);
        assert!(wire.private.is_empty() && wire.shared.is_empty() && !wire.changed);
        assert_eq!(service.document_value(), roster, "{packet:?}");
    };
    // No pet out: petting counts for nothing; a grooming-mode report is a stale client lock and
    // is unlocked with id 0 (there is no active pet id to name).
    let (mut service, pet) = dog();
    let mut fixture = kit();
    ignored(&mut service, &mut fixture, &tactile(0, 300, 5_000), at);
    let stale = send(&mut service, &mut fixture, &tactile(2, 300, 5_000), at);
    assert_unlocked(stale, 0, "no pet out");
    // Not in water, and not while the owner is busy.
    send(&mut service, &mut fixture, &summon(pet), START);
    send(&mut service, &mut fixture, &water(true), at);
    ignored(&mut service, &mut fixture, &tactile(0, 300, 5_000), at);
    // In water there is no session either, so the stale mode is still unlocked.
    let wet = send(&mut service, &mut fixture, &tactile(2, 300, 5_000), at);
    assert_unlocked(wet, pet, "in water with no session");
    send(&mut service, &mut fixture, &water(false), at);
    let busy = service
        .handle_at_ms(
            &tactile(0, 300, 5_000),
            OWNER,
            [0.0; 3],
            &mut fixture.inventory,
            PetBusy {
                minigame_active: true,
            },
            at,
        )
        .unwrap()
        .unwrap();
    assert!(busy.private.is_empty() && !busy.changed);
    // A groom mode with no session is unlocked; an unknown mode counts for nothing.
    let stale = send(&mut service, &mut fixture, &tactile(2, 300, 15_000), at);
    assert_unlocked(stale, pet, "no session");
    ignored(&mut service, &mut fixture, &tactile(1, 300, 15_000), at);

    // A session takes only its own mode: petting messages do not count during grooming.
    let (mut service, mut fixture, pet) = ready();
    send(&mut service, &mut fixture, &groom(pet), at);
    ignored(
        &mut service,
        &mut fixture,
        &tactile(0, 300, 5_000),
        at + 300,
    );
    ignored(
        &mut service,
        &mut fixture,
        &tactile(7, 300, 15_000),
        at + 300,
    );

    // (R) A vanity Pet Pal is not petted for play.
    let content = PetContent::bundled().unwrap();
    let (mut service, mut fixture, _) = ready();
    let crab = content.definition_for_item(CRAB_ITEM).unwrap();
    let crab = service.adopt_vendor_grant(2, &crab, 0).unwrap();
    send(&mut service, &mut fixture, &summon(crab), START);
    ignored(&mut service, &mut fixture, &tactile(0, 300, 5_000), at);
}

/// F796/Finding 7: a client that missed the kind-4 end keeps its grooming state word at 2. A
/// mode-2 report with no server session must unlock it (kind 6) without changing anything; while
/// a session is live the same answer would clear the client under it, so the server stays silent.
#[test]
fn a_grooming_mode_report_without_a_session_is_unlocked_with_kind_6() {
    let start = START + 1_000;
    // A session that ran out its time: the kind 4 went out, but the client may have missed it.
    let (mut service, mut fixture, pet) = ready();
    send(&mut service, &mut fixture, &groom(pet), start);
    let seconds = number("seconds") as u64 * 1_000;
    let ended = service.care_tick(&mut fixture.inventory, start + seconds);
    assert_eq!(notify_kinds(&ended.private), vec![4]);
    let roster = service.document_value();
    let stale = stroke(&mut service, &mut fixture, start + seconds + 300, BRISK);
    assert_unlocked(stale, pet, "stale after the timeout");
    assert_eq!(service.document_value(), roster);

    // A live session is not cleared by a kind 6: a busy mode-2 report stays silent.
    let (mut service, mut fixture, pet) = ready();
    send(&mut service, &mut fixture, &groom(pet), start);
    let busy = service
        .handle_at_ms(
            &tactile(2, 300, 15_000),
            OWNER,
            [0.0; 3],
            &mut fixture.inventory,
            PetBusy {
                minigame_active: true,
            },
            start + 300,
        )
        .unwrap()
        .unwrap();
    assert!(
        busy.private.is_empty() && !busy.changed,
        "the live session is left alone"
    );
    assert!(service.is_grooming());
}

#[test]
fn hostile_stroke_values_are_clamped_per_message_and_per_second() {
    let (mut service, mut fixture, pet) = ready();
    let start = START + 1_000;
    send(&mut service, &mut fixture, &groom(pet), start);
    let base = groom_need(&service, pet);

    // No time, no movement and words that are negative as the client's signed ints count as 0.
    let roster = service.document_value();
    for claim in [
        (0, 5_000),
        (300, 0),
        (u32::MAX, 5_000),
        (300, u32::MAX),
        (0x8000_0000, 15_000),
    ] {
        let wire = stroke(&mut service, &mut fixture, start, claim);
        assert!(wire.private.is_empty(), "{claim:?}");
    }
    assert_eq!(service.document_value(), roster, "nothing was gained");

    // One enormous message adds at most the per-message bound.
    let huge = (1, i32::MAX as u32);
    stroke(&mut service, &mut fixture, start, huge);
    let one = f64::from(groom_need(&service, pet) - base);
    assert!(one > 0.0 && one <= number("max_gain_per_message"), "{one}");

    // A flood of 1,000 of them inside one second stays under that second's cap.
    for step in 1..1_000 {
        stroke(&mut service, &mut fixture, start + step, huge);
    }
    let gained = f64::from(groom_need(&service, pet) - base);
    assert!(
        gained <= number("max_gain_per_second"),
        "{gained} in one second"
    );
    assert!(
        gained > number("max_gain_per_second") - 0.5,
        "the cap is reached: {gained}"
    );
}

/// F796/Finding 4: the per-second cap counts fixed one-second windows, so a span that straddles a
/// boundary can see two caps over about one second. The 2x worst case is accepted (a 60 s session,
/// need points only) and pinned here; the old threat note claimed the cap held over any span.
#[test]
fn a_stroke_span_across_a_window_boundary_sees_at_most_two_caps() {
    let (mut service, mut fixture, pet) = ready();
    let start = START + 1_000;
    send(&mut service, &mut fixture, &groom(pet), start);
    let base = groom_need(&service, pet);
    let huge = (1, i32::MAX as u32);
    let cap = number("max_gain_per_second");
    // Three messages fill the first window just before its boundary...
    for at in [start, start + 1, start + 2] {
        stroke(&mut service, &mut fixture, at, huge);
    }
    let first = f64::from(groom_need(&service, pet) - base);
    assert!(
        first <= cap && first > cap - 0.5,
        "{first} in the first window"
    );
    // ...and the next message after it opens a fresh cap inside the same second.
    for at in [start + 1_001, start + 1_002, start + 1_003] {
        stroke(&mut service, &mut fixture, at, huge);
    }
    let both = f64::from(groom_need(&service, pet) - base);
    assert!(both > cap + 1.0, "{both} across the boundary");
    assert!(both <= 2.0 * cap, "{both} over 1.002 s");
}

#[test]
fn a_dismiss_water_a_new_summon_and_zoning_end_the_session_with_kind_4_and_no_more_gain() {
    let start = START + 1_000;
    let session = || {
        let (mut service, mut fixture, pet) = ready();
        send(&mut service, &mut fixture, &groom(pet), start);
        stroke(&mut service, &mut fixture, start + 300, BRISK);
        (service, fixture, pet)
    };

    // Zoning: the gateway sends this packet before it dismisses the pet.
    let (service, _, pet) = session();
    assert_eq!(service.care_end_packets(), vec![notify(pet, 4, 0)]);

    // A dismiss and a replacing summon.
    let (mut service, mut fixture, pet) = session();
    let dismissed = send(
        &mut service,
        &mut fixture,
        &[53, 0, 4, 0, 0, 0, 0, 0],
        start + 400,
    );
    assert_eq!(notify_kinds(&dismissed.private), vec![4]);
    send(&mut service, &mut fixture, &summon(pet), start + 500);
    let roster = service.document_value();
    let wire = stroke(&mut service, &mut fixture, start + 800, BRISK);
    assert_unlocked(wire, pet, "no gain after the dismiss");
    assert_eq!(
        service.document_value(),
        roster,
        "no gain after the dismiss"
    );
    let (mut service, mut fixture, pet) = session();
    let replaced = send(&mut service, &mut fixture, &summon(pet), start + 400);
    assert_eq!(notify_kinds(&replaced.private), vec![4]);
    assert!(
        notify_kinds(
            &service
                .care_tick(&mut fixture.inventory, start + 90_000)
                .private
        )
        .is_empty()
    );

    // Entering water, then leaving it: the session is over, and a new one can start.
    let (mut service, mut fixture, pet) = session();
    let wet = send(&mut service, &mut fixture, &water(true), start + 400);
    assert_eq!(wet.private, vec![notify(pet, 4, 0)]);
    assert!(wet.changed, "the gains so far are saved");
    send(&mut service, &mut fixture, &water(false), start + 450);
    let roster = service.document_value();
    let late = stroke(&mut service, &mut fixture, start + 500, BRISK);
    assert_unlocked(late, pet, "no gain after the water");
    assert_eq!(service.document_value(), roster, "no gain after the water");
    let restart = send(&mut service, &mut fixture, &groom(pet), start + 600);
    assert_eq!(
        notify_kinds(&restart.private),
        vec![3],
        "a new session starts clean"
    );
}

/// F796/Finding 1: a pet summoned by an owned adoption item (53/32) replaces the active pet, so a
/// running session must end with kind 4 like a 53/4 summon does; otherwise the client stays
/// locked in grooming. The feed half is `tests/pet_care.rs`'s adoption test (kind 2).
#[test]
fn summon_by_item_during_a_groom_ends_the_session_with_kind_4_and_summons_the_new_pet() {
    let start = START + 1_000;
    let (mut service, pet) = dog();
    let mut fixture = InventoryFixture::with_kit_items(
        &[
            (SHAMPOO_ITEM, SHAMPOO, 2),
            (CONDITIONER_ITEM, CONDITIONER, 1),
            (BRUSH_ITEM, BRUSH, 1),
            (ADOPT_ITEM, ADOPT_DEF, 1),
        ],
        &KIT,
    );
    for (slot, item) in [(1, SHAMPOO_ITEM), (2, CONDITIONER_ITEM), (4, BRUSH_ITEM)] {
        send(&mut service, &mut fixture, &fit(pet, slot, item), START);
    }
    send(&mut service, &mut fixture, &summon(pet), START);
    send(&mut service, &mut fixture, &groom(pet), start);
    stroke(&mut service, &mut fixture, start + 300, BRISK);
    let raised = groom_need(&service, pet);

    let adopted = send(
        &mut service,
        &mut fixture,
        &summon_by_item(ADOPT_DEF as u32, 0),
        start + 500,
    );
    assert_eq!(
        notify_kinds(&adopted.private),
        vec![4],
        "the replaced pet's session ends"
    );
    assert_eq!(adopted.private.last(), Some(&notify(pet, 4, 0)));
    assert!(adopted.changed);
    let new_pet = service.active_pet_id().expect("the adopted pet is out");
    assert_ne!(new_pet, pet);
    assert!(!service.is_grooming(), "the new pet has no session");
    assert_eq!(
        count(&fixture, SHAMPOO_ITEM),
        1,
        "the old session's shampoo"
    );
    assert_eq!(count(&fixture, ADOPT_ITEM), 0, "the certificate was spent");

    // The old session is gone: no gain, no second end, and only the new pet's stale-mode unlock.
    let roster = service.document_value();
    let late = stroke(&mut service, &mut fixture, start + 800, BRISK);
    assert_unlocked(late, new_pet, "no session after the adoption");
    assert!((groom_need(&service, pet) - raised).abs() < 1e-4);
    assert_eq!(service.document_value(), roster);
    let tick = service.care_tick(&mut fixture.inventory, start + 120_000);
    assert!(notify_kinds(&tick.private).is_empty() && !tick.changed);
}

#[test]
fn a_reconnect_mid_session_keeps_the_spent_shampoo_and_the_gains_but_not_the_session() {
    let start = START + 1_000;
    let (mut service, mut fixture, pet) = ready();
    let seconds = number("seconds") as u64 * 1_000;
    let before = groom_need(&service, pet);
    send(&mut service, &mut fixture, &groom(pet), start);
    for step in 1..=4 {
        stroke(&mut service, &mut fixture, start + step * 300, BRISK);
    }
    let kept = groom_need(&service, pet);
    assert!(kept > before + 2.0);

    // The reload has the raised bar and the spent kit, and no session to end or to feed.
    let content = PetContent::bundled().unwrap();
    let mut reloaded = PetService::from_document(content, service.document_value()).unwrap();
    assert!((groom_need(&reloaded, pet) - kept).abs() < 1e-4);
    assert_eq!(count(&fixture, SHAMPOO_ITEM), 1);
    send(&mut reloaded, &mut fixture, &summon(pet), start + 2_000);
    let roster = reloaded.document_value();
    let wire = stroke(&mut reloaded, &mut fixture, start + 2_300, BRISK);
    assert_unlocked(wire, pet, "the session did not survive the reconnect");
    assert_eq!(reloaded.document_value(), roster);
    let tick = reloaded.care_tick(&mut fixture.inventory, start + seconds + 5_000);
    assert!(notify_kinds(&tick.private).is_empty() && !tick.changed);
    assert_eq!(
        count(&fixture, SHAMPOO_ITEM),
        1,
        "the spent shampoo is not given back"
    );
}
