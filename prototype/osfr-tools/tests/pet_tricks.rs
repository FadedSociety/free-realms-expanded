//! T489: pet tricks through the gateway route (`PetService::handle_at_ms`, which
//! `Gameplay::handle_pets` calls), the world tick (`care_tick`, which `Gameplay::pet_tick` calls)
//! and `Gameplay` itself: 53/7 performs a trick (53/15 kind 1, 53/20 kind 5, the owner and pet
//! animations, then 53/15 kinds 2 and 3 with 53/20 kind 6), 53/8 is the client's gesture report,
//! 53/14 sets a favourite. A refusal sends only the two unlocks (53/15 kind 3, 53/20 kind 6).
#[path = "support/pet_care.rs"]
mod care;
#[path = "support/json.rs"]
mod json;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/pet_inventory.rs"]
mod pet_inventory;
#[path = "../src/bin/osfr_gateway/pet_wire.rs"]
mod pet_wire;

use care::*;
use npc_quest_chain::{CHARACTER, Fixture};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    pets::{PetBusy, PetContent, PetRequest, PetService, PetWire},
    world_events::piece_052::animation_packet,
};
use pet_inventory::InventoryFixture;
use pet_wire::{favorite, feed, groom, perform_trick, summon, trick_result, water};
use serde_json::{Value, json};

/// 1,000 s after adoption: the same base the grooming suite uses.
const START: u64 = 1_000_000;
/// Model 364 (the dog) lists this trick and has an animation row for it.
const TRICK: u32 = 11;
/// A vanity Pet Pal (T454).
const CRAB_ITEM: i32 = 7885;
const DISMISS: [u8; 8] = [53, 0, 4, 0, 0, 0, 0, 0];

/// The shipped `care.trick` number `key`.
fn rule(key: &str) -> u64 {
    pets_json()["care"]["trick"][key].as_u64().unwrap()
}

/// The text id the client resolves for the trick's name.
fn name_id(trick: u32) -> u32 {
    let document = pets_json();
    let found = document["tricks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == trick)
        .unwrap();
    found["name_id"].as_u64().unwrap() as u32
}

/// An expected 53/15 PetTrickNotify: pet, kind, trick, text id, value.
fn trick_notify(pet: u32, kind: u32, trick: u32, text: u32, value: u32) -> Vec<u8> {
    words(&[53, 0, 15], &[pet, kind, trick, text, value])
}

/// The 53/15 kind 2 result the shipped rules send.
fn result(pet: u32, trick: u32) -> Vec<u8> {
    let (text, value) = (rule("result_text_id"), rule("result_value"));
    trick_notify(pet, 2, trick, text as u32, value as u32)
}

/// What a trick's end sends, in order: the result, 53/15 kind 3, then the 53/20 unlock.
fn ending(pet: u32, trick: u32) -> Vec<Vec<u8>> {
    let mut packets = vec![result(pet, trick)];
    packets.extend(unlocks(pet, trick));
    packets
}

/// The two packets of a refusal: 53/15 kind 3 and 53/20 kind 6.
fn unlocks(pet: u32, trick: u32) -> Vec<Vec<u8>> {
    vec![trick_notify(pet, 3, trick, 0, 0), notify(pet, 6, 0)]
}

fn owner_animation(trick: u32) -> Vec<u8> {
    animation_packet(OWNER, trick_row(trick).owner_animation_id, false)
}

fn pet_animation(trick: u32) -> Vec<u8> {
    animation_packet(
        PetService::pet_guid(OWNER),
        trick_row(trick).animation_id,
        false,
    )
}

/// Every trick packet in `packets`, in order: 53/15, 53/20 and 35/8.
fn traffic(packets: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let animation = animation_packet(0, 0, false);
    packets
        .iter()
        .filter(|packet| {
            packet.starts_with(&[53, 0, 15])
                || packet.starts_with(&[53, 0, 20])
                || packet.starts_with(&animation[..4])
        })
        .cloned()
        .collect()
}

/// A summoned dog at `START`.
fn ready() -> (PetService, InventoryFixture, u32) {
    let (mut service, pet) = dog();
    let mut fixture = kit();
    send(&mut service, &mut fixture, &summon(pet), START);
    (service, fixture, pet)
}

/// The tick at `at` sends no trick traffic.
fn quiet(service: &mut PetService, fixture: &mut InventoryFixture, at: u64, why: &str) {
    let wire = service.care_tick(&mut fixture.inventory, at);
    assert!(traffic(&wire.private).is_empty(), "{why}");
    assert!(wire.shared.is_empty(), "{why}");
    assert!(!wire.changed, "{why}");
}

/// A refusal: exactly the two unlocks, nothing shared, nothing accepted or saved.
fn assert_refused(wire: &PetWire, pet: u32, trick: u32, why: &str) {
    assert_eq!(wire.private, unlocks(pet, trick), "{why}");
    assert!(wire.shared.is_empty(), "{why}");
    assert_eq!(wire.accepted_trick, None, "{why}");
    assert!(!wire.changed, "{why}");
    assert!(wire.notice.is_none(), "{why}");
}

#[test]
fn trick_requests_decode_with_exact_lengths() {
    assert_eq!(
        PetRequest::decode(&perform_trick(11)).unwrap(),
        Some(PetRequest::PerformTrick { trick_id: 11 })
    );
    assert_eq!(perform_trick(11), words(&[53, 0, 7], &[11]));
    assert_eq!(
        PetRequest::decode(&trick_result(11)).unwrap(),
        Some(PetRequest::TrickResult { trick_id: 11 })
    );
    assert_eq!(trick_result(11), words(&[53, 0, 8], &[11]));
    assert_eq!(
        PetRequest::decode(&favorite(7, 3, 12)).unwrap(),
        Some(PetRequest::Favorite {
            pet_id: 7,
            slot: 3,
            trick_id: 12,
        })
    );
    // Reader-derived layout (`00b5f670`, ctor `00b51290`): the pet id, slot - 1, the trick.
    assert_eq!(favorite(7, 3, 12), words(&[53, 0, 14], &[7, 3, 12]));
    for bad in [
        vec![53, 0, 7],
        words(&[53, 0, 7], &[1, 2]),
        vec![53, 0, 8],
        words(&[53, 0, 8], &[1, 2]),
        words(&[53, 0, 14], &[7, 3]),
        words(&[53, 0, 14], &[7, 3, 12, 1]),
        vec![53, 0, 14],
    ] {
        assert!(PetRequest::decode(&bad).is_err(), "{bad:?}");
    }
}

#[test]
fn a_known_trick_runs_kind_1_lock_owner_animation_pet_animation_then_the_result_and_the_unlocks() {
    let (mut service, mut fixture, pet) = ready();
    let (delay, run) = (
        trick_row(TRICK).start_delay_ms as u64,
        trick_row(TRICK).duration_ms as u64,
    );
    let start = START + 1_000;

    let accepted = send(&mut service, &mut fixture, &perform_trick(TRICK), start);
    let owner = owner_animation(TRICK);
    assert_eq!(
        accepted.private,
        vec![
            trick_notify(pet, 1, TRICK, name_id(TRICK), 0),
            notify(pet, 5, 0),
            owner.clone(),
        ],
        "the name line, the lock, then the owner's animation"
    );
    assert_eq!(accepted.shared, vec![owner], "viewers see the owner too");
    assert_eq!(accepted.accepted_trick, Some(TRICK), "the fact T788 reads");
    assert!(!accepted.changed, "a trick saves nothing");

    // Nothing until the delay passes; then the pet's animation, for the owner and the viewers.
    quiet(
        &mut service,
        &mut fixture,
        start + delay - 1,
        "before the delay",
    );
    let animated = service.care_tick(&mut fixture.inventory, start + delay);
    assert_eq!(traffic(&animated.private), vec![pet_animation(TRICK)]);
    assert_eq!(animated.shared, vec![pet_animation(TRICK)]);
    assert_eq!(animated.accepted_trick, None);
    // It plays once, and the trick keeps running until its time is over.
    quiet(
        &mut service,
        &mut fixture,
        start + delay + run - 1,
        "mid-run",
    );

    // The end: the result, kind 3, then the 53/20 unlock (the lock's state word is cleared).
    let ended = service.care_tick(&mut fixture.inventory, start + delay + run);
    assert_eq!(traffic(&ended.private), ending(pet, TRICK));
    assert!(ended.shared.is_empty() && !ended.changed);
    quiet(
        &mut service,
        &mut fixture,
        start + delay + run + 60_000,
        "after the end",
    );

    // The pet is free again: the next trick starts like the first.
    let again = send(
        &mut service,
        &mut fixture,
        &perform_trick(TRICK),
        start + delay + run + 100,
    );
    assert_eq!(again.accepted_trick, Some(TRICK));
}

#[test]
fn a_late_tick_still_plays_the_animation_before_the_result() {
    let (mut service, mut fixture, pet) = ready();
    send(&mut service, &mut fixture, &perform_trick(TRICK), START);
    let wire = service.care_tick(&mut fixture.inventory, START + 10 * 60_000);
    let mut expected = vec![pet_animation(TRICK)];
    expected.extend(ending(pet, TRICK));
    assert_eq!(traffic(&wire.private), expected);
}

#[test]
fn every_refusal_sends_only_the_unlocks_and_changes_nothing() {
    let at = START + 1_000;
    // The dog's own trick list lacks one of the cat's.
    let content = PetContent::bundled().unwrap();
    let dog_tricks: Vec<u32> = content
        .model(364)
        .unwrap()
        .tricks
        .iter()
        .map(|trick| trick.id)
        .collect();
    let foreign = content
        .model(50)
        .unwrap()
        .tricks
        .iter()
        .map(|trick| trick.id)
        .find(|id| !dog_tricks.contains(id))
        .expect("the cat has a trick the dog lacks");

    type Setup = fn() -> (PetService, InventoryFixture, u32);
    let cases: [(&str, u32, Setup); 3] = [
        ("a trick the pet's model lacks", foreign, ready),
        ("trick 0", 0, ready),
        ("no pet summoned", TRICK, || {
            let (service, pet) = dog();
            (service, kit(), pet)
        }),
    ];
    for (why, trick, setup) in cases {
        let (mut service, mut fixture, pet) = setup();
        let (roster, bag) = (service.document_value(), fixture.inventory.state().clone());
        let wire = send(&mut service, &mut fixture, &perform_trick(trick), at);
        // With no pet out the unlock names pet 0; the client ignores a pet it does not know.
        let named = if service.active_pet_id().is_some() {
            pet
        } else {
            0
        };
        assert_refused(&wire, named, trick, why);
        assert_eq!(service.document_value(), roster, "{why}");
        assert_eq!(fixture.inventory.state(), &bag, "{why}");
        quiet(&mut service, &mut fixture, at + 200_000, why);
    }

    // The same dog, refused for what its owner is doing.
    let (mut service, mut fixture, pet) = ready();
    let (roster, bag) = (service.document_value(), fixture.inventory.state().clone());
    let busy = service
        .handle_at_ms(
            &perform_trick(TRICK),
            OWNER,
            [0.0; 3],
            &mut fixture.inventory,
            PetBusy {
                minigame_active: true,
            },
            |_, _| None,
            at,
        )
        .unwrap()
        .unwrap();
    assert_refused(&busy, pet, TRICK, "busy");
    send(&mut service, &mut fixture, &water(true), at);
    let wet = send(&mut service, &mut fixture, &perform_trick(TRICK), at);
    assert_refused(&wet, pet, TRICK, "in water");
    assert_eq!(service.document_value(), roster);
    assert_eq!(fixture.inventory.state(), &bag);
    quiet(&mut service, &mut fixture, at + 200_000, "busy and water");
    // Out of the water the same request is accepted: the refusals left no residue.
    send(&mut service, &mut fixture, &water(false), at);
    let dry = send(&mut service, &mut fixture, &perform_trick(TRICK), at);
    assert_eq!(dry.accepted_trick, Some(TRICK));
}

#[test]
fn a_vanity_pet_is_refused_even_with_a_trick_row_forced_onto_its_model() {
    // The shipped vanity models list no tricks, so a refusal there proves little: give the crab's
    // model a trick and an animation row, then ask. Only the vanity rule can refuse it.
    let probe = PetContent::bundled().unwrap();
    let crab_model = probe.model_id(&probe.definition_for_item(CRAB_ITEM).unwrap().model);
    let template = trick_row(TRICK);
    let content = content_with(|document| {
        let model = model_row(document, i64::from(crab_model));
        model["tricks"] = json!([{"id": TRICK, "known": true}]);
        model["trick_animations"] = json!([{
            "trick_id": TRICK,
            "animation_id": template.animation_id,
            "owner_animation_id": template.owner_animation_id,
            "duration_ms": template.duration_ms,
            "start_delay_ms": template.start_delay_ms,
            "src": "test",
        }]);
    });
    assert!(content.is_vanity(crab_model));
    let (mut service, pet) = dog_with(content.clone());
    let mut fixture = kit();
    let crab = content.definition_for_item(CRAB_ITEM).unwrap();
    let crab = service.adopt_vendor_grant(2, &crab, 0).unwrap();
    send(&mut service, &mut fixture, &summon(crab), START);
    let roster = service.document_value();
    let vain = send(
        &mut service,
        &mut fixture,
        &perform_trick(TRICK),
        START + 1_000,
    );
    assert_refused(&vain, crab, TRICK, "vanity");
    assert_eq!(service.document_value(), roster);
    // The same request on the dog is accepted: it is the vanity flag that refused.
    send(&mut service, &mut fixture, &summon(pet), START + 2_000);
    let dog = send(
        &mut service,
        &mut fixture,
        &perform_trick(TRICK),
        START + 3_000,
    );
    assert_eq!(dog.accepted_trick, Some(TRICK));
}

#[test]
fn a_trick_the_pet_has_not_learned_is_refused_until_it_learns_it() {
    // The model row lists the trick as not yet known, and the pet's save lacks it.
    let content = content_with(|document| {
        let model = model_row(document, 364);
        for trick in model["tricks"].as_array_mut().unwrap() {
            if trick["id"] == TRICK {
                trick["known"] = json!(false);
            }
        }
    });
    let (service, pet) = dog_with(content.clone());
    let mut document = service.document_value().unwrap();
    document["pets"][0]["learned_tricks"] = json!([]);
    let mut service = PetService::from_document(content, Some(document)).unwrap();
    let mut fixture = kit();
    send(&mut service, &mut fixture, &summon(pet), START);
    let (roster, bag) = (service.document_value(), fixture.inventory.state().clone());
    let refused = send(
        &mut service,
        &mut fixture,
        &perform_trick(TRICK),
        START + 1_000,
    );
    assert_refused(&refused, pet, TRICK, "not learned");
    assert_eq!(service.document_value(), roster);
    assert_eq!(fixture.inventory.state(), &bag);
    // A favourite needs a known trick too.
    let favourite = send(
        &mut service,
        &mut fixture,
        &favorite(pet, 0, TRICK),
        START + 1_000,
    );
    assert!(favourite.private.is_empty() && !favourite.changed);

    service.learn_trick(pet, TRICK).unwrap();
    let learned = send(
        &mut service,
        &mut fixture,
        &perform_trick(TRICK),
        START + 2_000,
    );
    assert_eq!(learned.accepted_trick, Some(TRICK));
}

#[test]
fn without_the_trick_rules_every_perform_is_refused() {
    let content = content_with(|document| {
        document["care"].as_object_mut().unwrap().remove("trick");
    });
    let (mut service, pet) = dog_with(content);
    let mut fixture = kit();
    send(&mut service, &mut fixture, &summon(pet), START);
    let wire = send(
        &mut service,
        &mut fixture,
        &perform_trick(TRICK),
        START + 1_000,
    );
    assert_refused(&wire, pet, TRICK, "dormant");
}

#[test]
fn a_perform_during_another_trick_is_refused_and_the_first_trick_runs_on() {
    let (mut service, mut fixture, pet) = ready();
    let (delay, run) = (
        trick_row(TRICK).start_delay_ms as u64,
        trick_row(TRICK).duration_ms as u64,
    );
    let start = START + 1_000;
    send(&mut service, &mut fixture, &perform_trick(TRICK), start);
    let other = 12;
    for (trick, at) in [(TRICK, start + 10), (other, start + 20)] {
        let wire = send(&mut service, &mut fixture, &perform_trick(trick), at);
        assert_refused(&wire, pet, trick, "a trick is running");
    }
    // The first trick is untouched: its animation, then one result at its own time.
    let animated = service.care_tick(&mut fixture.inventory, start + delay);
    assert_eq!(traffic(&animated.private), vec![pet_animation(TRICK)]);
    let ended = service.care_tick(&mut fixture.inventory, start + delay + run);
    assert_eq!(traffic(&ended.private), ending(pet, TRICK));
    quiet(
        &mut service,
        &mut fixture,
        start + delay + run + 1_000,
        "once",
    );

    // A feed or a groom refuses while a trick runs, and a trick refuses while they run.
    let (mut service, mut fixture, pet) = ready();
    send(&mut service, &mut fixture, &fit(pet, 5, BOWL_ITEM), START);
    send(&mut service, &mut fixture, &fit(pet, 6, FOOD_ITEM), START);
    send(
        &mut service,
        &mut fixture,
        &fit(pet, 1, SHAMPOO_ITEM),
        START,
    );
    send(&mut service, &mut fixture, &fit(pet, 4, BRUSH_ITEM), START);
    send(&mut service, &mut fixture, &perform_trick(TRICK), start);
    let (roster, bag) = (service.document_value(), fixture.inventory.state().clone());
    for request in [feed(pet), groom(pet)] {
        let wire = send(&mut service, &mut fixture, &request, start + 5);
        assert_eq!(
            wire.private,
            vec![notify(pet, 6, 0)],
            "refused for the trick"
        );
    }
    assert_eq!(service.document_value(), roster);
    assert_eq!(fixture.inventory.state(), &bag);
    service.care_tick(&mut fixture.inventory, start + delay + run);
    let feeding = send(
        &mut service,
        &mut fixture,
        &feed(pet),
        start + delay + run + 1,
    );
    assert_eq!(
        notify_kinds(&feeding.private),
        vec![1],
        "the feed starts after the trick"
    );
    let wire = send(
        &mut service,
        &mut fixture,
        &perform_trick(TRICK),
        start + delay + run + 2,
    );
    assert_refused(&wire, pet, TRICK, "a feed is running");
}

#[test]
fn a_gesture_report_for_the_running_trick_ends_it_only_once_the_pet_animation_began() {
    let (mut service, mut fixture, pet) = ready();
    let (delay, run) = (
        trick_row(TRICK).start_delay_ms as u64,
        trick_row(TRICK).duration_ms as u64,
    );
    let start = START + 1_000;
    send(&mut service, &mut fixture, &perform_trick(TRICK), start);

    // `00cc7420` sends 53/8 right behind 53/7: it must not cut the show short.
    let early = send(&mut service, &mut fixture, &trick_result(TRICK), start + 1);
    assert_eq!(early, PetWire::default());
    quiet(
        &mut service,
        &mut fixture,
        start + delay - 1,
        "the early report did nothing",
    );

    // After the pet animation began (a drawing round that ended), the same report ends the trick.
    service.care_tick(&mut fixture.inventory, start + delay);
    let report = send(
        &mut service,
        &mut fixture,
        &trick_result(TRICK),
        start + delay + 10,
    );
    assert_eq!(traffic(&report.private), ending(pet, TRICK));
    assert!(report.shared.is_empty() && !report.changed && report.accepted_trick.is_none());
    // It is over once: the natural end sends nothing more, and no reward or save came of it.
    quiet(
        &mut service,
        &mut fixture,
        start + delay + run + 1_000,
        "already ended",
    );

    // A report that arrives before the tick delivered the animation brings it first.
    let (mut service, mut fixture, pet) = ready();
    send(&mut service, &mut fixture, &perform_trick(TRICK), start);
    let sudden = send(
        &mut service,
        &mut fixture,
        &trick_result(TRICK),
        start + delay + 1,
    );
    let mut expected = vec![pet_animation(TRICK)];
    expected.extend(ending(pet, TRICK));
    assert_eq!(traffic(&sudden.private), expected);
    assert_eq!(sudden.shared, vec![pet_animation(TRICK)]);
}

#[test]
fn a_gesture_report_for_a_different_trick_is_ignored() {
    let (mut service, mut fixture, pet) = ready();
    let (delay, run) = (
        trick_row(TRICK).start_delay_ms as u64,
        trick_row(TRICK).duration_ms as u64,
    );
    let start = START + 1_000;
    send(&mut service, &mut fixture, &perform_trick(TRICK), start);
    service.care_tick(&mut fixture.inventory, start + delay);
    let roster = service.document_value();
    for other in [12, 13, 999_999, 0] {
        let wire = send(
            &mut service,
            &mut fixture,
            &trick_result(other),
            start + delay + 10,
        );
        assert_eq!(wire, PetWire::default(), "trick {other}");
    }
    assert_eq!(service.document_value(), roster);
    // The running trick did not end: it still ends at its own time, once.
    quiet(
        &mut service,
        &mut fixture,
        start + delay + run - 1,
        "still running",
    );
    let ended = service.care_tick(&mut fixture.inventory, start + delay + run);
    assert_eq!(traffic(&ended.private), ending(pet, TRICK));
}

#[test]
fn a_late_gesture_report_is_answered_with_the_unlocks_only_within_the_grace_window() {
    let (mut service, mut fixture, pet) = ready();
    let (delay, run) = (
        trick_row(TRICK).start_delay_ms as u64,
        trick_row(TRICK).duration_ms as u64,
    );
    let grace = rule("gesture_grace_ms");
    let start = START + 1_000;
    let end = start + delay + run;

    // No trick yet: a stray report is dropped.
    let stray = send(&mut service, &mut fixture, &trick_result(TRICK), start - 1);
    assert_eq!(stray, PetWire::default());

    send(&mut service, &mut fixture, &perform_trick(TRICK), start);
    service.care_tick(&mut fixture.inventory, end);
    // The state word 53/8 sets is cleared again by the repeated unlocks.
    let late = send(
        &mut service,
        &mut fixture,
        &trick_result(TRICK),
        end + grace,
    );
    assert_eq!(late.private, unlocks(pet, TRICK));
    assert!(late.shared.is_empty() && !late.changed && late.accepted_trick.is_none());
    // A different trick's report, and a report past the window, are dropped.
    let other = send(&mut service, &mut fixture, &trick_result(12), end + 1);
    assert_eq!(other, PetWire::default());
    let past = send(
        &mut service,
        &mut fixture,
        &trick_result(TRICK),
        end + grace + 1,
    );
    assert_eq!(past, PetWire::default());
}

#[test]
fn a_dismiss_a_new_summon_water_and_zoning_end_the_trick_with_the_unlocks_and_no_stale_timer() {
    let (delay, run) = (
        trick_row(TRICK).start_delay_ms as u64,
        trick_row(TRICK).duration_ms as u64,
    );
    let start = START + 1_000;
    let end = start + delay + run;
    let running = || {
        let (mut service, mut fixture, pet) = ready();
        send(&mut service, &mut fixture, &perform_trick(TRICK), start);
        (service, fixture, pet)
    };

    // Zoning: the gateway sends these packets before it dismisses the pet.
    let (service, _, pet) = running();
    assert_eq!(service.care_end_packets(), unlocks(pet, TRICK));

    // A dismiss, then the pet summoned again: the old timer must not fire on the new session.
    let (mut service, mut fixture, pet) = running();
    let dismissed = send(&mut service, &mut fixture, &DISMISS, start + 100);
    assert_eq!(traffic(&dismissed.private), unlocks(pet, TRICK));
    send(&mut service, &mut fixture, &summon(pet), start + 200);
    quiet(
        &mut service,
        &mut fixture,
        start + delay,
        "the old animation",
    );
    quiet(&mut service, &mut fixture, end + 1_000, "the old end");
    assert!(service.care_end_packets().is_empty());

    // A replacing summon.
    let (mut service, mut fixture, pet) = running();
    let replaced = send(&mut service, &mut fixture, &summon(pet), start + 100);
    assert_eq!(traffic(&replaced.private), unlocks(pet, TRICK));
    quiet(
        &mut service,
        &mut fixture,
        end + 1_000,
        "after the replacing summon",
    );

    // A failed summon (a pet the owner lacks) leaves the trick running.
    let (mut service, mut fixture, pet) = running();
    let failed = send(&mut service, &mut fixture, &summon(pet + 77), start + 100);
    assert!(traffic(&failed.private).is_empty());
    let ended = service.care_tick(&mut fixture.inventory, end);
    let mut expected = vec![pet_animation(TRICK)];
    expected.extend(ending(pet, TRICK));
    assert_eq!(traffic(&ended.private), expected);

    // Entering water ends it: the unlocks, no result, no save.
    let (mut service, mut fixture, pet) = running();
    let wet = send(&mut service, &mut fixture, &water(true), start + 100);
    assert_eq!(wet.private, unlocks(pet, TRICK));
    assert!(!wet.changed && wet.accepted_trick.is_none());
    quiet(&mut service, &mut fixture, end + 1_000, "after the water");
    // No grace answer for a trick the water ended, and a new trick can start after it.
    let report = send(&mut service, &mut fixture, &trick_result(TRICK), end);
    assert_eq!(report, PetWire::default());
    send(&mut service, &mut fixture, &water(false), end);
    let next = send(&mut service, &mut fixture, &perform_trick(TRICK), end + 1);
    assert_eq!(next.accepted_trick, Some(TRICK));
}

#[test]
fn a_reconnect_mid_trick_keeps_nothing_of_the_session() {
    let (mut service, mut fixture, pet) = ready();
    let start = START + 1_000;
    send(&mut service, &mut fixture, &perform_trick(TRICK), start);
    let content = PetContent::bundled().unwrap();
    let mut reloaded = PetService::from_document(content, service.document_value()).unwrap();
    // Not even summoned yet: nothing to end, nothing to answer.
    quiet(
        &mut reloaded,
        &mut fixture,
        start + 60_000,
        "before the summon",
    );
    assert_eq!(
        send(
            &mut reloaded,
            &mut fixture,
            &trick_result(TRICK),
            start + 60_000
        ),
        PetWire::default()
    );
    send(&mut reloaded, &mut fixture, &summon(pet), start + 61_000);
    assert!(
        reloaded.care_end_packets().is_empty(),
        "no session to unlock"
    );
    quiet(
        &mut reloaded,
        &mut fixture,
        start + 120_000,
        "after the summon",
    );
    let fresh = send(
        &mut reloaded,
        &mut fixture,
        &perform_trick(TRICK),
        start + 121_000,
    );
    assert_eq!(fresh.accepted_trick, Some(TRICK));
}

#[test]
fn a_favourite_is_echoed_saved_listed_and_survives_a_reload() {
    let (mut service, mut fixture, pet) = ready();
    let wire = send(&mut service, &mut fixture, &favorite(pet, 2, 12), START);
    assert_eq!(wire.private, vec![words(&[53, 0, 14], &[pet, 2, 12])]);
    assert!(wire.changed, "a favourite is saved");
    assert!(wire.shared.is_empty());
    assert_eq!(service.pet(pet).unwrap().favorite_tricks[2], Some(12));

    // The 53/5 record ends with the four favourites then the eight utilities, then the active pair.
    let list = service.list_packet();
    let tail = list.len() - 12 - 48;
    assert_eq!(u32_at(&list, tail + 8), 12);
    assert_eq!(u32_at(&list, tail), 0);

    // A reload (a reconnect) keeps it, and every slot takes its own trick.
    let content = PetContent::bundled().unwrap();
    let mut reloaded = PetService::from_document(content, service.document_value()).unwrap();
    assert_eq!(reloaded.pet(pet).unwrap().favorite_tricks[2], Some(12));
    for (slot, trick) in [(0, 17), (1, 11), (3, 13)] {
        let echo = send(
            &mut reloaded,
            &mut fixture,
            &favorite(pet, slot, trick),
            START,
        );
        assert_eq!(echo.private, vec![words(&[53, 0, 14], &[pet, slot, trick])]);
    }
    let kept = reloaded.pet(pet).unwrap().favorite_tricks;
    assert_eq!(kept, [Some(17), Some(11), Some(12), Some(13)]);
    // The favourite of a pet that is not summoned is allowed: the id names the pet.
    let (mut service, pet) = dog();
    let echo = send(&mut service, &mut fixture, &favorite(pet, 0, 15), START);
    assert_eq!(echo.private, vec![words(&[53, 0, 14], &[pet, 0, 15])]);
}

#[test]
fn a_bad_favourite_sends_nothing_and_changes_nothing() {
    let (mut service, mut fixture, pet) = ready();
    // The inbound handler (`00b5ce60` case 14) also stores slot 4, which addresses the first
    // utility slot, so the server refuses 4 and everything above, signed or not.
    let cases = [
        ("slot 4", favorite(pet, 4, 12)),
        ("slot 5", favorite(pet, 5, 12)),
        ("slot 0x80000000", favorite(pet, 0x8000_0000, 12)),
        ("slot -1", favorite(pet, u32::MAX, 12)),
        ("an unknown trick id", favorite(pet, 0, 999_999)),
        ("trick 0", favorite(pet, 0, 0)),
        ("another pet's id", favorite(pet + 40, 0, 12)),
        ("pet 0", favorite(0, 0, 12)),
    ];
    let (roster, bag) = (service.document_value(), fixture.inventory.state().clone());
    for (why, packet) in cases {
        let wire = send(&mut service, &mut fixture, &packet, START);
        assert_eq!(wire, PetWire::default(), "{why}");
        assert_eq!(service.document_value(), roster, "{why}");
    }
    // A trick that exists but belongs to another species is not known by this dog.
    let content = PetContent::bundled().unwrap();
    let foreign = content
        .model(50)
        .unwrap()
        .tricks
        .iter()
        .map(|trick| trick.id)
        .find(|id| {
            !content
                .model(364)
                .unwrap()
                .tricks
                .iter()
                .any(|own| own.id == *id)
        })
        .unwrap();
    let wire = send(
        &mut service,
        &mut fixture,
        &favorite(pet, 0, foreign),
        START,
    );
    assert_eq!(wire, PetWire::default());
    assert_eq!(service.document_value(), roster);
    assert_eq!(fixture.inventory.state(), &bag);
}

#[test]
fn hostile_clocks_and_floods_stay_bounded() {
    // A clock at the top of the range: the end saturates instead of overflowing, and the trick ends.
    let (mut service, mut fixture, _) = ready();
    let accepted = send(
        &mut service,
        &mut fixture,
        &perform_trick(TRICK),
        u64::MAX - 1,
    );
    assert_eq!(accepted.accepted_trick, Some(TRICK));
    let ended = service.care_tick(&mut fixture.inventory, u64::MAX);
    assert_eq!(notify_kinds(&ended.private), vec![6]);

    // A clock that runs backwards never ends a trick early, and a flood of 53/7 and 53/8 changes
    // nothing: one session, one result.
    let (mut service, mut fixture, pet) = ready();
    let start = START + 10_000;
    send(&mut service, &mut fixture, &perform_trick(TRICK), start);
    let roster = service.document_value();
    let mut results = 0;
    for step in 0..500u64 {
        let at = start.saturating_sub(step * 7);
        let wire = send(&mut service, &mut fixture, &perform_trick(TRICK), at);
        assert_eq!(wire.private, unlocks(pet, TRICK));
        let report = send(&mut service, &mut fixture, &trick_result(TRICK), at);
        assert_eq!(report, PetWire::default(), "no end before the animation");
        results += service
            .care_tick(&mut fixture.inventory, at)
            .private
            .iter()
            .filter(|packet| packet.starts_with(&[53, 0, 15]))
            .count();
    }
    assert_eq!(results, 0);
    assert_eq!(service.document_value(), roster);
}

#[test]
fn bad_trick_content_is_rejected_at_load() {
    let tamper = |edit: &dyn Fn(&mut Value)| {
        let mut document = pets_json();
        edit(&mut document);
        PetContent::from_json(&serde_json::to_vec(&document).unwrap()).is_err()
    };
    assert!(!tamper(&|_| ()), "the shipped content loads");
    assert!(
        tamper(&|document| {
            let row = model_row(document, 364)["trick_animations"][0].clone();
            model_row(document, 364)["trick_animations"]
                .as_array_mut()
                .unwrap()
                .push(row);
        }),
        "a second row for one trick"
    );
    assert!(
        tamper(&|document| {
            model_row(document, 364)["trick_animations"][0]["trick_id"] = json!(999_999);
        }),
        "a trick the model does not list"
    );
    for (key, bad) in [
        ("duration_ms", 0),
        ("duration_ms", 30_001),
        ("start_delay_ms", 30_000),
        ("animation_id", 0),
        ("owner_animation_id", -1),
    ] {
        assert!(
            tamper(&|document| {
                model_row(document, 364)["trick_animations"][0][key] = json!(bad);
            }),
            "{key} = {bad}"
        );
    }
    for (key, bad) in [
        ("gesture_grace_ms", 0u64),
        ("gesture_grace_ms", 60_001),
        ("result_text_id", 0),
        ("result_value", 1_001),
    ] {
        assert!(
            tamper(&|document| {
                document["care"]["trick"][key] = json!(bad);
            }),
            "{key} = {bad}"
        );
    }
}

/// The real route: `Gameplay::handle_in_world` reaches the pet handler, `Gameplay::pet_tick` runs the
/// trick on the world tick, and the owner's and the viewers' packets leave through `Outcome`.
#[test]
fn a_trick_runs_through_gameplay_for_the_owner_and_the_viewers() {
    let fixture = Fixture::minimal(CHARACTER);
    let (service, pet) = dog();
    let who = CharacterState {
        guid: CHARACTER,
        zone_id: 0,
        position: [-440.0, 0.8, 1292.0],
        rotation: [1.0, 0.0, 0.0],
    };
    let mut game = Gameplay::load(fixture.path(), None).unwrap();
    game.adopt_pets(service.document_value()).unwrap();
    game.initialize_world(&who, START).unwrap();
    let (delay, run) = (
        trick_row(TRICK).start_delay_ms as u64,
        trick_row(TRICK).duration_ms as u64,
    );
    let start = START + 1_000;
    let actor = PetService::pet_guid(who.guid);
    game.handle_in_world(&summon(pet), &who, start).unwrap();

    let refused = game
        .handle_in_world(&perform_trick(999_999), &who, start + 10)
        .unwrap();
    assert_eq!(traffic(&refused.replies), unlocks(pet, 999_999));
    assert!(refused.nearby.is_empty() && !refused.changed);

    let accepted = game
        .handle_in_world(&perform_trick(TRICK), &who, start + 20)
        .unwrap();
    let owner = animation_packet(who.guid, trick_row(TRICK).owner_animation_id, false);
    assert_eq!(
        traffic(&accepted.replies),
        vec![
            trick_notify(pet, 1, TRICK, name_id(TRICK), 0),
            notify(pet, 5, 0),
            owner.clone(),
        ]
    );
    assert_eq!(traffic(&accepted.nearby), vec![owner]);
    assert!(!accepted.changed);

    let pet_plays = animation_packet(actor, trick_row(TRICK).animation_id, false);
    let at_delay = game
        .pet_tick(&who, start + 20 + delay)
        .expect("a pet is out");
    assert_eq!(traffic(&at_delay.replies), vec![pet_plays.clone()]);
    assert_eq!(traffic(&at_delay.nearby), vec![pet_plays]);
    let ended = game
        .pet_tick(&who, start + 20 + delay + run)
        .expect("a pet is out");
    assert_eq!(traffic(&ended.replies), ending(pet, TRICK));
    assert!(!ended.changed, "a trick saves nothing");

    // Zoning ends a running trick with the unlocks before the pet is dismissed.
    game.handle_in_world(&perform_trick(TRICK), &who, start + 1_000_000)
        .unwrap();
    let zoned = game.settle_pets(&who, true, start + 1_000_100).unwrap();
    assert_eq!(traffic(&zoned.replies), unlocks(pet, TRICK));
}
