//! T485: the pet care panel through the gateway route (`PetService::handle_at_ms`, which
//! `Gameplay::handle_pets` calls) and the world tick (`care_tick`, which `Gameplay::pet_tick`
//! calls): fit kit items (53/16), feed (53/19) with the 53/20 lifecycle, water (53/28) and live
//! 53/3 needs.
#[path = "support/pet_care.rs"]
mod care;
#[path = "support/json.rs"]
mod json;
#[path = "support/pet_inventory.rs"]
mod pet_inventory;
#[path = "../src/bin/osfr_gateway/pet_wire.rs"]
mod pet_wire;

use care::*;
use osfr_tools::pets::{PetContent, PetRequest, PetService};
use pet_inventory::InventoryFixture;
use pet_wire::{feed, summon, water};

/// Crab, a vanity Pet Pal (T454).
const CRAB_ITEM: i32 = 7885;

/// A summoned dog with the bowl and food fitted.
fn ready_to_feed() -> (PetService, InventoryFixture, u32) {
    let (mut service, pet) = dog();
    let mut fixture = kit();
    send(&mut service, &mut fixture, &fit(pet, 5, BOWL_ITEM), T0);
    send(&mut service, &mut fixture, &fit(pet, 6, FOOD_ITEM), T0);
    send(&mut service, &mut fixture, &summon(pet), T0);
    (service, fixture, pet)
}

#[test]
fn utility_feed_and_water_requests_decode_with_exact_lengths() {
    assert_eq!(
        PetRequest::decode(&pet_wire::utility(true, 7, 6, 22)).unwrap(),
        Some(PetRequest::Utility {
            remove: true,
            pet_id: 7,
            slot: 6,
            item: 22,
        })
    );
    assert_eq!(
        PetRequest::decode(&feed(7)).unwrap(),
        Some(PetRequest::Feed { pet_id: 7 })
    );
    assert_eq!(
        PetRequest::decode(&water(true)).unwrap(),
        Some(PetRequest::Water(true))
    );
    for bad in [
        words(&[53, 0, 16, 0], &[7, 6]),
        words(&[53, 0, 16, 2], &[7, 6, 22]),
        words(&[53, 0, 19], &[7]),
        words(&[53, 0, 19], &[7, 0, 0]),
        vec![53, 0, 28],
        vec![53, 0, 28, 2],
        vec![53, 0, 28, 1, 0],
    ] {
        assert!(PetRequest::decode(&bad).is_err(), "{bad:?}");
    }
}

#[test]
fn fitting_echoes_53_16_and_a_wrong_slot_type_or_vanity_pet_changes_nothing() {
    let (mut service, pet) = dog();
    let mut fixture = kit();
    let wire = send(&mut service, &mut fixture, &fit(pet, 5, BOWL_ITEM), T0);
    assert_eq!(
        wire.private,
        vec![words(&[53, 0, 16, 0], &[pet, 5, BOWL_ITEM as u32])]
    );
    assert!(wire.changed);
    let wire = send(&mut service, &mut fixture, &fit(pet, 6, FOOD_ITEM), T0);
    assert_eq!(
        wire.private,
        vec![words(&[53, 0, 16, 0], &[pet, 6, FOOD_ITEM as u32])]
    );

    let roster = service.document_value();
    let inventory = fixture.inventory.state().clone();
    // Food in the bowl's slot: re-echo the bowl that stays there.
    let wire = send(&mut service, &mut fixture, &fit(pet, 5, FOOD_ITEM), T0);
    assert_eq!(
        wire.private,
        vec![words(&[53, 0, 16, 0], &[pet, 5, BOWL_ITEM as u32])]
    );
    assert!(!wire.changed);
    // A plain item, an unknown slot and an item not owned: an empty slot echoes as removed.
    for (slot, item) in [(1, PLAIN_ITEM), (8, SHAMPOO_ITEM), (1, 99)] {
        let wire = send(&mut service, &mut fixture, &fit(pet, slot, item), T0);
        assert_eq!(
            wire.private,
            vec![words(&[53, 0, 16, 1], &[pet, slot, 0])],
            "{slot} {item}"
        );
        assert!(!wire.changed);
    }
    assert_eq!(service.document_value(), roster);
    assert_eq!(fixture.inventory.state(), &inventory);

    // The shampoo fits slot 1; removing it clears the slot.
    send(&mut service, &mut fixture, &fit(pet, 1, SHAMPOO_ITEM), T0);
    assert_eq!(
        service.pet(pet).unwrap().equipment.utilities[1],
        Some(SHAMPOO_ITEM)
    );
    let wire = send(
        &mut service,
        &mut fixture,
        &pet_wire::utility(true, pet, 1, SHAMPOO_ITEM as u32),
        T0,
    );
    assert_eq!(wire.private, vec![words(&[53, 0, 16, 1], &[pet, 1, 0])]);
    assert_eq!(service.pet(pet).unwrap().equipment.utilities[1], None);

    // (R) A vanity Pet Pal takes no kit and is never fed.
    let content = PetContent::bundled().unwrap();
    let crab = content.definition_for_item(CRAB_ITEM).unwrap();
    let crab = service.adopt_vendor_grant(2, &crab, 0).unwrap();
    let wire = send(&mut service, &mut fixture, &fit(crab, 5, BOWL_ITEM), T0);
    assert!(!wire.changed);
    assert_eq!(service.pet(crab).unwrap().equipment.utilities[5], None);
    send(&mut service, &mut fixture, &summon(crab), T0);
    let wire = send(&mut service, &mut fixture, &feed(crab), T0);
    assert_eq!(notify_kinds(&wire.private), vec![6]);
}

#[test]
fn a_feed_begins_applies_one_food_once_and_ends() {
    let (mut service, mut fixture, pet) = ready_to_feed();
    let before = service.pet(pet).unwrap().needs.food;
    let wire = send(&mut service, &mut fixture, &feed(pet), T0 + 1_000);
    assert_eq!(wire.private.len(), 1);
    assert_eq!(notify_kinds(&wire.private), vec![1]);
    assert!(!wire.changed, "starting a feed spends nothing");

    let early = service.care_tick(&mut fixture.inventory, T0 + 1_500);
    assert!(!early.changed);
    assert_eq!(
        count(&fixture, FOOD_ITEM),
        2,
        "nothing is eaten before the apply step"
    );
    assert!(notify_kinds(&early.private).is_empty());

    let applied = service.care_tick(&mut fixture.inventory, T0 + 1_900);
    assert!(applied.changed, "the apply step marks a save");
    assert_eq!(count(&fixture, FOOD_ITEM), 1, "exactly one food");
    assert_eq!(count(&fixture, BOWL_ITEM), 1, "the bowl is kept");
    let status = statuses(&applied.private);
    assert_eq!(status.len(), 1);
    let food = f32::from_le_bytes(status[0][7..11].try_into().unwrap());
    assert!(food > before && food <= 100.0, "{before} -> {food}");

    for at in [T0 + 2_000, T0 + 4_000] {
        let wire = service.care_tick(&mut fixture.inventory, at);
        assert!(!wire.changed && notify_kinds(&wire.private).is_empty());
    }
    let ended = service.care_tick(&mut fixture.inventory, T0 + 7_000);
    assert_eq!(notify_kinds(&ended.private), vec![2]);
    assert_eq!(count(&fixture, FOOD_ITEM), 1);
    // The panel may start the next feed.
    let again = send(&mut service, &mut fixture, &feed(pet), T0 + 8_000);
    assert_eq!(notify_kinds(&again.private), vec![1]);
}

#[test]
fn feeding_without_a_bowl_only_unlocks_and_spends_nothing() {
    let (mut service, pet) = dog();
    let mut fixture = kit();
    send(&mut service, &mut fixture, &fit(pet, 6, FOOD_ITEM), T0);
    send(&mut service, &mut fixture, &summon(pet), T0);
    let roster = service.document_value();
    let wire = send(&mut service, &mut fixture, &feed(pet), T0 + 1_000);
    assert_eq!(
        wire.private,
        vec![words(&[53, 0, 20], &[pet, 6, 0, 0, 0, 0])]
    );
    assert!(!wire.changed);
    for at in [T0 + 2_000, T0 + 8_000] {
        assert!(!service.care_tick(&mut fixture.inventory, at).changed);
    }
    assert_eq!(count(&fixture, FOOD_ITEM), 2);
    assert_eq!(service.document_value(), roster);
}

#[test]
fn in_water_a_feed_is_refused_and_after_leaving_it_works() {
    let (mut service, mut fixture, pet) = ready_to_feed();
    assert_eq!(
        send(&mut service, &mut fixture, &water(true), T0).private,
        Vec::<Vec<u8>>::new()
    );
    let wire = send(&mut service, &mut fixture, &feed(pet), T0 + 1_000);
    assert_eq!(notify_kinds(&wire.private), vec![6]);
    send(&mut service, &mut fixture, &water(false), T0 + 1_500);
    let wire = send(&mut service, &mut fixture, &feed(pet), T0 + 2_000);
    assert_eq!(notify_kinds(&wire.private), vec![1]);
}

#[test]
fn a_second_feed_during_one_and_a_dismiss_spend_nothing_more() {
    let (mut service, mut fixture, pet) = ready_to_feed();
    send(&mut service, &mut fixture, &feed(pet), T0 + 1_000);
    let duplicate = send(&mut service, &mut fixture, &feed(pet), T0 + 1_200);
    assert_eq!(notify_kinds(&duplicate.private), vec![6]);
    service.care_tick(&mut fixture.inventory, T0 + 2_000);
    assert_eq!(count(&fixture, FOOD_ITEM), 1, "one feed, one food");

    // A dismiss mid-feed ends it (kind 2); later ticks consume nothing.
    let (mut service, mut fixture, pet) = ready_to_feed();
    send(&mut service, &mut fixture, &feed(pet), T0 + 1_000);
    let dismissed = send(
        &mut service,
        &mut fixture,
        &[53, 0, 4, 0, 0, 0, 0, 0],
        T0 + 1_100,
    );
    assert_eq!(notify_kinds(&dismissed.private), vec![2]);
    for at in [T0 + 2_000, T0 + 8_000] {
        assert!(!service.care_tick(&mut fixture.inventory, at).changed);
    }
    assert_eq!(count(&fixture, FOOD_ITEM), 2);
}

#[test]
fn a_reconnect_before_the_apply_step_eats_nothing_and_after_it_keeps_one_eaten() {
    let content = PetContent::bundled().unwrap();
    // Before: the session ends after 53/20 begin; the reloaded roster has no feed running.
    let (mut service, mut fixture, pet) = ready_to_feed();
    send(&mut service, &mut fixture, &feed(pet), T0 + 1_000);
    let mut reloaded =
        PetService::from_document(content.clone(), service.document_value()).unwrap();
    for at in [T0 + 2_000, T0 + 8_000] {
        assert!(!reloaded.care_tick(&mut fixture.inventory, at).changed);
    }
    assert_eq!(count(&fixture, FOOD_ITEM), 2);
    assert_eq!(
        reloaded.pet(pet).unwrap().equipment.utilities[6],
        Some(FOOD_ITEM),
        "the fitted food is saved"
    );

    // After: the food was eaten at the apply step; a reload never eats it again.
    let (mut service, mut fixture, pet) = ready_to_feed();
    send(&mut service, &mut fixture, &feed(pet), T0 + 1_000);
    assert!(
        service
            .care_tick(&mut fixture.inventory, T0 + 2_000)
            .changed
    );
    let mut reloaded = PetService::from_document(content, service.document_value()).unwrap();
    send(&mut reloaded, &mut fixture, &summon(pet), T0 + 3_000);
    for at in [T0 + 4_000, T0 + 10_000] {
        assert!(!reloaded.care_tick(&mut fixture.inventory, at).changed);
    }
    assert_eq!(count(&fixture, FOOD_ITEM), 1);
}

#[test]
fn live_needs_reach_the_owner_at_most_every_1_5_s_without_a_save() {
    let (mut service, pet) = dog();
    let mut fixture = kit();
    send(&mut service, &mut fixture, &summon(pet), T0);
    let roster = service.document_value();
    let mut sent = Vec::new();
    for step in 1..=100u64 {
        let at = T0 + step * 100;
        let wire = service.care_tick(&mut fixture.inventory, at);
        assert!(!wire.changed, "live needs never mark a save");
        for status in statuses(&wire.private) {
            assert_eq!(u32_at(status, 3), pet);
            sent.push(at);
        }
    }
    assert!(!sent.is_empty(), "a summoned pet's needs are shown");
    assert!(
        sent.windows(2).all(|pair| pair[1] - pair[0] >= 1_500),
        "{sent:?}"
    );
    assert_eq!(
        service.document_value(),
        roster,
        "the live view writes no roster"
    );
}
