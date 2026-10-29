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
#[path = "support/pet_wire.rs"]
mod pet_wire;

use care::*;
use osfr_tools::pets::{PetContent, PetRequest, PetService};
use pet_inventory::InventoryFixture;
use pet_wire::{feed, summon, water};

/// Crab, a vanity Pet Pal (T454).
const CRAB_ITEM: i32 = 7885;
/// T828: AnimationTypes.xml line 1590, `pet_feed` (the pet one-shot block, type 4); the shipped
/// `care.feed_animation`.
const EAT: i32 = 5507;
/// T828 (R): the one line a feed that ate sends its owner.
const FEED_DONE: &str = "Your pet has finished eating.";

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
    // A plain item or an item not owned: the unchanged (empty) slot echoes as removed.
    for (slot, item) in [(1, PLAIN_ITEM), (1, 99)] {
        let wire = send(&mut service, &mut fixture, &fit(pet, slot, item), T0);
        assert_eq!(
            wire.private,
            vec![words(&[53, 0, 16, 1], &[pet, slot, 0])],
            "{slot} {item}"
        );
        assert!(!wire.changed);
    }
    // F778: a slot outside the client's eight utility entries echoes nothing. The client's
    // 53/16 handler stores the echoed slot into its pet record with no bounds check
    // (00b5ce60 case 16, write at 00b5ea69), so a forged slot would corrupt or crash it.
    for slot in [0u32, 8, 0x3fff_ffff] {
        let wire = send(
            &mut service,
            &mut fixture,
            &fit(pet, slot, SHAMPOO_ITEM),
            T0,
        );
        assert_eq!(wire.private, Vec::<Vec<u8>>::new(), "slot {slot}");
        assert!(!wire.changed, "slot {slot}");
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
    assert_eq!(wire.private.len(), 1, "the 53/20 begin only");
    assert_eq!(notify_kinds(&wire.private), vec![1]);
    assert!(!wire.changed, "starting a feed spends nothing");
    assert_eq!(wire.notice, None, "starting is not completing");

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
    let inventory = fixture.inventory.state().clone();
    let wire = send(&mut service, &mut fixture, &feed(pet), T0 + 1_000);
    assert_eq!(
        wire.private,
        vec![words(&[53, 0, 20], &[pet, 6, 0, 0, 0, 0])]
    );
    assert!(!wire.changed);
    assert_eq!(wire.notice, None, "a refused feed never announces a meal");
    assert!(
        wire.shared.is_empty(),
        "a refused feed shows the viewers nothing"
    );
    assert!(
        animations(&wire.private, PetService::pet_guid(OWNER, pet)).is_empty(),
        "and plays no eat animation"
    );
    for at in [T0 + 2_000, T0 + 8_000] {
        assert!(!service.care_tick(&mut fixture.inventory, at).changed);
    }
    assert_eq!(count(&fixture, FOOD_ITEM), 2);
    assert_eq!(service.document_value(), roster);
    assert_eq!(fixture.inventory.state(), &inventory, "nothing was fitted");
}

#[test]
fn a_feed_plays_the_eat_animation_for_the_owner_and_the_viewers() {
    assert_eq!(pets_json()["care"]["feed_animation"], EAT);
    let (mut service, mut fixture, pet) = ready_to_feed();
    let actor = PetService::pet_guid(OWNER, pet);
    let eat = osfr_tools::world_events::piece_052::animation_packet(actor, EAT, false);
    let started = send(&mut service, &mut fixture, &feed(pet), T0 + 1_000);
    assert_eq!(started.private, vec![notify(pet, 1, 0)], "the begin only");
    assert!(started.shared.is_empty(), "nothing plays yet");
    // T797/T828: the apply plays the one eat animation for the owner and the viewers, and says
    // the feed's one completion line to the owner (the client's 53/20 kind 2 only unlocks).
    let applied = service.care_tick(&mut fixture.inventory, T0 + 2_000);
    assert_eq!(animations(&applied.private, actor), vec![EAT]);
    assert_eq!(applied.shared, vec![eat], "the viewers see the pet eat");
    assert_eq!(
        applied
            .private
            .iter()
            .filter(|packet| packet.starts_with(&[35, 0, 8]))
            .count(),
        1,
        "one eat animation"
    );
    assert_eq!(applied.notice, Some(FEED_DONE), "the owner is told it ate");
    // The end only unlocks the panel: no second animation and no second line.
    let ended = service.care_tick(&mut fixture.inventory, T0 + 7_000);
    assert_eq!(notify_kinds(&ended.private), vec![2]);
    assert!(
        animations(&ended.private, actor).is_empty(),
        "the eat plays once"
    );
    assert!(ended.shared.is_empty());
    assert_eq!(ended.notice, None, "the completion is said once");
}

#[test]
fn a_feed_announces_the_meal_once_and_a_food_that_left_the_bag_says_nothing() {
    let (mut service, mut fixture, pet) = ready_to_feed();
    let started = send(&mut service, &mut fixture, &feed(pet), T0 + 1_000);
    assert_eq!(started.notice, None, "starting is not completing");
    let applied = service.care_tick(&mut fixture.inventory, T0 + 2_000);
    assert!(applied.changed);
    assert_eq!(applied.notice, Some(FEED_DONE));
    assert_eq!(count(&fixture, FOOD_ITEM), 1, "exactly one food is eaten");
    let after = service.care_tick(&mut fixture.inventory, T0 + 3_000);
    assert_eq!(after.notice, None, "the completion is said once");

    // A food that left the bag before the apply step consumes nothing, so it is not a meal and
    // the feed says nothing when it ends.
    let (mut service, mut fixture, pet) = ready_to_feed();
    let before = service.pet(pet).unwrap().needs.food;
    send(&mut service, &mut fixture, &feed(pet), T0 + 1_000);
    fixture.inventory.consume(FOOD_ITEM, 2).unwrap();
    let applied = service.care_tick(&mut fixture.inventory, T0 + 2_000);
    assert!(!applied.changed, "nothing was eaten");
    assert_eq!(applied.notice, None, "nothing was eaten, nothing is said");
    assert!(
        animations(&applied.private, PetService::pet_guid(OWNER, pet)).is_empty(),
        "no eat animation either"
    );
    let ended = service.care_tick(&mut fixture.inventory, T0 + 7_000);
    assert_eq!(notify_kinds(&ended.private), vec![2]);
    assert_eq!(ended.notice, None, "the feed still ends silently");
    assert_eq!(service.pet(pet).unwrap().needs.food, before);
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
    assert_eq!(wire.notice, None, "a refused feed says nothing");
    assert!(wire.shared.is_empty());
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
    assert_eq!(duplicate.notice, None, "a refused feed says nothing");
    assert!(duplicate.shared.is_empty());
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
fn adopting_a_pet_during_a_feed_ends_the_feed_without_spending_the_food() {
    let (mut service, pet) = dog();
    let mut fixture = InventoryFixture::with_kit_items(
        &[
            (BOWL_ITEM, BOWL, 1),
            (FOOD_ITEM, FOOD, 2),
            (ADOPT_ITEM, ADOPT_DEF, 1),
        ],
        &KIT,
    );
    send(&mut service, &mut fixture, &fit(pet, 5, BOWL_ITEM), T0);
    send(&mut service, &mut fixture, &fit(pet, 6, FOOD_ITEM), T0);
    send(&mut service, &mut fixture, &summon(pet), T0);
    let started = send(&mut service, &mut fixture, &feed(pet), T0 + 1_000);
    assert_eq!(notify_kinds(&started.private), vec![1]);

    let adopted = send(
        &mut service,
        &mut fixture,
        &pet_wire::summon_by_item(ADOPT_DEF as u32, 0),
        T0 + 1_500,
    );
    // F778: the adopted pet replaced the one being fed, so the feed ends (kind 2) as it does
    // on a re-summon; the client's care panel only unlocks on 53/20 kinds 2, 4 and 6.
    assert_eq!(notify_kinds(&adopted.private), vec![2]);
    assert!(adopted.changed);
    assert_eq!(adopted.notice, None, "a feed that never ate says nothing");
    assert_eq!(service.active_pet_id(), Some(2), "the adopted pet is out");
    assert!(
        fixture
            .inventory
            .state()
            .items
            .iter()
            .all(|item| item.definition_id != ADOPT_DEF || item.count == 0),
        "the certificate is spent"
    );
    for at in [T0 + 2_000, T0 + 8_000] {
        assert!(!service.care_tick(&mut fixture.inventory, at).changed);
    }
    assert_eq!(count(&fixture, FOOD_ITEM), 2, "the fed food was not eaten");
}

#[test]
fn feeding_the_last_food_clears_the_fitted_slot_for_the_client() {
    let (mut service, pet) = dog();
    let mut fixture =
        InventoryFixture::with_kit_items(&[(BOWL_ITEM, BOWL, 1), (FOOD_ITEM, FOOD, 1)], &KIT);
    send(&mut service, &mut fixture, &fit(pet, 5, BOWL_ITEM), T0);
    send(&mut service, &mut fixture, &fit(pet, 6, FOOD_ITEM), T0);
    send(&mut service, &mut fixture, &summon(pet), T0);
    send(&mut service, &mut fixture, &feed(pet), T0 + 1_000);
    let applied = service.care_tick(&mut fixture.inventory, T0 + 2_000);
    assert!(applied.changed);
    assert_eq!(count(&fixture, FOOD_ITEM), 0, "the last food is eaten");
    assert_eq!(
        service.pet(pet).unwrap().equipment.utilities[6],
        None,
        "the empty stack leaves the slot"
    );
    // The care panel shows the slot from the 53/16 echo it already received (00b5ce60 case
    // 16), so the empty slot is echoed as removed for the client too.
    assert_eq!(
        applied
            .private
            .iter()
            .filter(|packet| packet.starts_with(&[53, 0, 16]))
            .cloned()
            .collect::<Vec<_>>(),
        vec![words(&[53, 0, 16, 1], &[pet, 6, 0])]
    );
    assert_eq!(count(&fixture, BOWL_ITEM), 1, "the bowl stays");
}

/// F820: `spend_one` clears the emptied instance from every pet that fitted it, so the apply's
/// 53/16 clears must cover those slots too, not only the summoned pet's: the other pet's care
/// panel kept a ghost food.
#[test]
fn feeding_the_last_food_clears_every_pet_that_fitted_it() {
    let (mut service, pet) = dog();
    let content = PetContent::bundled().unwrap();
    let mut definition = content.definition(1_000_015).unwrap();
    definition.model = content.model(364).unwrap().adr.clone();
    let other = service.adopt_vendor_grant(2, &definition, 0).unwrap();
    let mut fixture =
        InventoryFixture::with_kit_items(&[(BOWL_ITEM, BOWL, 1), (FOOD_ITEM, FOOD, 1)], &KIT);
    send(&mut service, &mut fixture, &fit(pet, 5, BOWL_ITEM), T0);
    send(&mut service, &mut fixture, &fit(pet, 6, FOOD_ITEM), T0);
    // The same owned instance fits a second pet's slot: no rule refuses a duplicate fit.
    assert!(
        send(&mut service, &mut fixture, &fit(other, 6, FOOD_ITEM), T0).changed,
        "the second pet takes the shared food"
    );
    send(&mut service, &mut fixture, &summon(pet), T0);
    send(&mut service, &mut fixture, &feed(pet), T0 + 1_000);
    let applied = service.care_tick(&mut fixture.inventory, T0 + 2_000);
    assert!(applied.changed);
    assert_eq!(count(&fixture, FOOD_ITEM), 0, "the last food is eaten");
    for who in [pet, other] {
        assert_eq!(
            service.pet(who).unwrap().equipment.utilities[6],
            None,
            "pet {who} loses the emptied stack"
        );
    }
    assert_eq!(
        applied
            .private
            .iter()
            .filter(|packet| packet.starts_with(&[53, 0, 16]))
            .cloned()
            .collect::<Vec<_>>(),
        vec![
            words(&[53, 0, 16, 1], &[pet, 6, 0]),
            words(&[53, 0, 16, 1], &[other, 6, 0]),
        ],
        "one clear for every slot that lost the food"
    );
    assert_eq!(count(&fixture, BOWL_ITEM), 1, "the bowl stays");
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
    // 60 s at the shipped 0.04/s summoned decay moves the bars by 2.4; the 0.5 epsilon is
    // crossed four times, so the gap and the value change are both exercised (F778: 10 s was below
    // the epsilon and never tested either).
    let mut sent: Vec<(u64, f32)> = Vec::new();
    for step in 1..=600u64 {
        let at = T0 + step * 100;
        let wire = service.care_tick(&mut fixture.inventory, at);
        assert!(!wire.changed, "live needs never mark a save");
        for status in statuses(&wire.private) {
            assert_eq!(u32_at(status, 3), pet);
            sent.push((at, f32::from_le_bytes(status[7..11].try_into().unwrap())));
        }
    }
    assert!(
        sent.len() >= 2,
        "the summoned pet's needs are shown more than once: {sent:?}"
    );
    assert!(
        sent.windows(2).all(|pair| pair[1].0 - pair[0].0 >= 1_500),
        "{sent:?}"
    );
    assert!(
        sent.windows(2).all(|pair| pair[1].1 < pair[0].1),
        "each 53/3 carries the decayed food value: {sent:?}"
    );
    assert_eq!(
        service.document_value(),
        roster,
        "the live view writes no roster"
    );
}

/// The four floats of a 53/3 (food, groom, play, mood).
fn bars(status: &[u8]) -> [f32; 4] {
    std::array::from_fn(|i| f32::from_le_bytes(status[7 + 4 * i..11 + 4 * i].try_into().unwrap()))
}

#[test]
fn a_new_pet_starts_below_full_and_the_start_level_is_bounded() {
    let (service, pet) = dog();
    let needs = service.pet(pet).unwrap().needs;
    let start = pets_json()["need_policy"]["initial"].as_f64().unwrap() as f32;
    assert!(start > 0.0 && start < 100.0, "care has work to do: {start}");
    assert_eq!(
        [needs.food, needs.groom, needs.play, needs.mood],
        [start; 4]
    );
    // Sanity bounds, not pinned values: the level and the summoned rate stay inside the range
    // the bars can show, and a bad file fails at load.
    for (key, bad) in [("initial", 0.0), ("initial", 101.0)] {
        let refused = loads(|doc| doc["need_policy"][key] = bad.into());
        assert!(refused.is_err(), "{key} {bad}");
    }
    assert!(loads(|doc| doc["need_policy"]["summoned_decay_per_second"] = 5.0.into()).is_err());
    assert!(loads(|doc| doc["need_policy"]["initial"] = 100.0.into()).is_ok());
}

/// The owner, native 2026-10-27: nobody could tell the bars fall. Over three minutes of play the
/// summoned pet's live 53/3 must show every bar a few points lower, and the bars never leave
/// 0..=100 however long the pet stays out.
#[test]
fn a_summoned_pets_bars_fall_visibly_within_three_minutes_and_stay_in_range() {
    let (mut service, pet) = dog();
    let mut fixture = kit();
    let start = service.pet(pet).unwrap().needs.food;
    send(&mut service, &mut fixture, &summon(pet), 1_000);
    let (mut last, mut sent) = ([start; 4], 0);
    for step in 1..=180u64 {
        let wire = service.care_tick(&mut fixture.inventory, 1_000 + step * 1_000);
        for status in statuses(&wire.private) {
            last = bars(status);
            sent += 1;
        }
    }
    assert!(sent >= 5, "the bars are refreshed as they fall: {sent}");
    for (name, now) in ["food", "groom", "play", "mood"].into_iter().zip(last) {
        assert!(
            start - now >= 5.0,
            "{name} fell {start} -> {now} in 3 minutes"
        );
    }
    // Two hours later every bar is empty, not negative.
    let wire = service.care_tick(&mut fixture.inventory, 1_000 + 7_200_000);
    assert_eq!(bars(statuses(&wire.private)[0]), [0.0; 4]);
}
