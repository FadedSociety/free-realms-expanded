//! T533: pet treasure. A summoned, settled, non-vanity pet in a good mood sometimes sniffs, digs and
//! unearths a chest that only its owner sees and can open, once, for a reward from the `treasure`
//! block of `pets.json`. The service tests drive the service calls `Gameplay::pet_tick` makes
//! (`follow_owner`, `care_tick`, `treasure_tick`) for the rules the service owns; the world tick's
//! order and every path through the session go through `pet_tick`, `handle_in_world` (26/8 on the
//! chest), `settle_pets` and the housing request the busy gate reads.
#[path = "support/pet_care.rs"]
mod care;
#[path = "common/gateway_packets.rs"]
mod gateway_packets;
#[path = "common/item_catalogue.rs"]
mod item_catalogue;
#[path = "support/json.rs"]
mod json;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/pet_inventory.rs"]
mod pet_inventory;
#[path = "support/pet_wire.rs"]
mod pet_wire;
// Only `source()` is used; the rest of the shared round70 fixture is other suites' base.
#[path = "common/round70_content.rs"]
#[allow(dead_code)]
mod round70_content;

use care::*;
use gateway_packets::{body, login, wrap};
use npc_quest_chain::{CHARACTER, Fixture};
use osfr_tools::{
    character_store::CharacterState,
    chat,
    gameplay::{Gameplay, GameplayState, Outcome},
    gateway::Gateway,
    inventory::{Inventory, InventoryItem},
    npc_interaction::NpcInteractRequest,
    pets::{PetContent, PetService, TreasureRefusal, pet_removal_packet},
    startup_world::{NpcSpawn, parse_add_npc},
};
use pet_inventory::InventoryFixture;
use pet_wire::{feed, groom, perform_trick, summon, water};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

/// One second after the dog's adoption: every need is still at its starting level (60, above the 25 mood threshold).
const START: u64 = 1_000;
/// 8,000 s after adoption: the idle decay (0.01 per second) has taken every need down to 20.
const SAD_START: u64 = 8_000_000;
/// The world tick, in ms.
const TICK: u64 = 500;
/// The wall clock the cooldown is kept in (any Unix second; the tests move it by hand).
const WALL: u64 = 1_800_000_000;
const CHEST_MODEL: i32 = 151;
const SNIFF: i32 = 5520;
const DIG: i32 = 5525;
/// A cat (model 50): its ADR declares neither hunt slot.
const CAT: i32 = 50;
/// Crab, a vanity Pet Pal (T454).
const CRAB_ITEM: i32 = 7885;
/// A trick the dog's model (364) knows and has an animation row for (T489).
const TRICK: u32 = 11;
const DISMISS: [u8; 8] = [53, 0, 4, 0, 0, 0, 0, 0];
const SEEDS: std::ops::Range<u64> = 0..8;

fn treasure() -> Value {
    pets_json()["treasure"].clone()
}

fn rule(key: &str) -> u64 {
    treasure()[key].as_u64().unwrap()
}

/// A tuned copy of the shipped content.
fn tuned(edit: impl FnOnce(&mut Value)) -> PetContent {
    content_with(|document| edit(&mut document["treasure"]))
}

/// The packet's chest spawns (35/2, model 151); the pet's own AddNpc is another model.
fn chests(packets: &[Vec<u8>]) -> Vec<NpcSpawn> {
    packets
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 2, 0]))
        .filter_map(|packet| parse_add_npc(packet).ok())
        .filter(|npc| npc.model_id == CHEST_MODEL)
        .collect()
}

/// The hunt's animation slots (sniff, dig) among the 35/8 packets (base words, u64 guid, i32
/// slot, i32 0, u8 flag); the pet's idle animations are other slots.
fn slots(packets: &[Vec<u8>]) -> Vec<i32> {
    packets
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 8, 0]))
        .map(|packet| u32_at(packet, 12) as i32)
        .filter(|slot| [SNIFF, DIG].contains(slot))
        .collect()
}

fn removals(packets: &[Vec<u8>], guid: u64) -> usize {
    packets
        .iter()
        .filter(|packet| **packet == pet_removal_packet(guid))
        .count()
}

/// What one world tick sent: to the owner, to the viewers, and whether it asked for a save.
struct Seen {
    at: u64,
    private: Vec<Vec<u8>>,
    shared: Vec<Vec<u8>>,
    changed: bool,
}

impl Seen {
    fn chest(&self) -> Option<NpcSpawn> {
        chests(&self.private).into_iter().next()
    }

    fn eventful(&self) -> bool {
        !chests(&self.private).is_empty() || !slots(&self.private).is_empty() || self.changed
    }
}

/// The owner's pet summoned at `start`, with the three service calls a world tick makes. This is
/// the service-level harness: the content rules, the cooldown in the roster, the find schedule and
/// the open all live in `PetService`. The world tick's own order and every session path are pinned
/// through `Gameplay::pet_tick` below, never by this harness.
struct Dig {
    service: PetService,
    fixture: InventoryFixture,
    pet: u32,
    actor: u64,
    owner: [f32; 3],
    wall: u64,
    /// The owner's start-activity busy gate, as `Gameplay::pet_tick` passes `activity_busy`.
    busy: bool,
    at: u64,
}

impl Dig {
    fn new(service: PetService, pet: u32, seed: u64, start: u64) -> Self {
        Self::with_bag(service, pet, seed, start, kit())
    }

    /// `new` with a bag the caller built (a loop of sessions can reuse one).
    fn with_bag(
        mut service: PetService,
        pet: u32,
        seed: u64,
        start: u64,
        mut fixture: InventoryFixture,
    ) -> Self {
        service.set_treasure_seed(seed);
        send(&mut service, &mut fixture, &summon(pet), start);
        Self {
            service,
            fixture,
            pet,
            actor: PetService::pet_guid(OWNER),
            owner: [0.0; 3],
            wall: WALL,
            busy: false,
            at: start,
        }
    }

    /// The shipped dog, mood high.
    fn dog(seed: u64) -> Self {
        let (service, pet) = dog();
        Self::new(service, pet, seed, START)
    }

    fn on(content: PetContent, seed: u64) -> Self {
        let (service, pet) = dog_with(content);
        Self::new(service, pet, seed, START)
    }

    /// One tick `TICK` ms after the last.
    fn step(&mut self) -> Seen {
        self.at += TICK;
        self.tick(self.at)
    }

    fn tick(&mut self, at: u64) -> Seen {
        self.at = at;
        let mut wire = self
            .service
            .follow_owner(OWNER, self.owner, |_, _| None, false, at);
        let care = self.service.care_tick(&mut self.fixture.inventory, at);
        wire.private.extend(care.private);
        wire.shared.extend(care.shared);
        let treasure =
            self.service
                .treasure_tick(self.busy, self.owner, |_, _| None, at, self.wall);
        wire.private.extend(treasure.private);
        wire.shared.extend(treasure.shared);
        Seen {
            at,
            private: wire.private,
            shared: wire.shared,
            changed: care.changed || treasure.changed,
        }
    }

    /// Ticks until `ms` after the last one; only the ticks that showed a chest, a hunt animation
    /// or a save.
    fn run(&mut self, ms: u64) -> Vec<Seen> {
        let until = self.at + ms;
        let mut seen = Vec::new();
        while self.at < until {
            let tick = self.step();
            if tick.eventful() || removals_of_any(&tick.private) {
                seen.push(tick);
            }
        }
        seen
    }

    /// Ticks until the chest appears, for at most `ms`.
    fn until_chest(&mut self, ms: u64) -> Option<Seen> {
        let until = self.at + ms;
        while self.at < until {
            let tick = self.step();
            if tick.chest().is_some() {
                return Some(tick);
            }
        }
        None
    }

    fn send(&mut self, packet: &[u8]) -> osfr_tools::pets::PetWire {
        send(&mut self.service, &mut self.fixture, packet, self.at)
    }

    /// The saved roster's `treasure_found_at` of this pet.
    fn found_at(&self) -> Option<u64> {
        found_at(&self.service.document_value().unwrap(), self.pet)
    }
}

/// Any 35/3 in the packets (an expired chest's removal).
fn removals_of_any(packets: &[Vec<u8>]) -> bool {
    packets
        .iter()
        .any(|packet| packet.starts_with(&[35, 0, 3, 0]))
}

fn found_at(document: &Value, pet: u32) -> Option<u64> {
    document["pets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == pet)
        .unwrap()["treasure_found_at"]
        .as_u64()
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn spot(npc: &NpcSpawn) -> [f32; 3] {
    [npc.position[0], npc.position[1], npc.position[2]]
}

/// The tier (its table row) a chest spawn belongs to, by the name id the client shows.
fn tier_of(npc: &NpcSpawn) -> Value {
    treasure()["tiers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|tier| tier["name_id"] == npc.name_string_id)
        .unwrap_or_else(|| panic!("no tier names {}", npc.name_string_id))
        .clone()
}

#[test]
fn a_happy_settled_dog_sniffs_digs_and_unearths_a_chest_for_its_owner_alone() {
    let sniff_ms = treasure()["sniff"]["duration_ms"].as_u64().unwrap();
    let dig_ms = treasure()["dig"]["duration_ms"].as_u64().unwrap();
    let (find_min, find_max) = (rule("find_min_ms"), rule("find_max_ms"));
    // T828: a digging pet keeps its own sniff and dig, never the generic find animation.
    let dog_find = PetContent::bundled()
        .unwrap()
        .model(364)
        .unwrap()
        .find_animation_id()
        .unwrap();
    for seed in SEEDS {
        let mut dog = Dig::dog(seed);
        let seen = dog.run(find_max + sniff_ms + dig_ms + 15_000);
        let chest = seen.iter().position(|tick| tick.chest().is_some()).unwrap();
        let spawn = seen[chest].chest().unwrap();
        // Sniff then dig, in that order, for the owner and the viewers alike.
        let hunt: Vec<(u64, i32)> = seen
            .iter()
            .flat_map(|tick| slots(&tick.private).into_iter().map(|slot| (tick.at, slot)))
            .collect();
        let slot_ids: Vec<i32> = hunt.iter().map(|(_, slot)| *slot).collect();
        assert_eq!(slot_ids, [SNIFF, DIG], "seed {seed}");
        let shared: Vec<i32> = seen.iter().flat_map(|tick| slots(&tick.shared)).collect();
        assert_eq!(shared, [SNIFF, DIG], "the viewers see the dig, seed {seed}");
        let (sniff_at, dig_at) = (hunt[0].0, hunt[1].0);
        assert!(
            (sniff_ms..sniff_ms + TICK).contains(&(dig_at - sniff_at)),
            "seed {seed}: sniff at {sniff_at}, dig at {dig_at}"
        );
        assert!(
            (dig_ms..dig_ms + TICK).contains(&(seen[chest].at - dig_at)),
            "seed {seed}: dig at {dig_at}, chest at {}",
            seen[chest].at
        );
        // The wait is the data's window, counted from the moment the pet stood settled.
        assert!(
            sniff_at >= START + find_min,
            "seed {seed}: sniffed at {sniff_at}"
        );
        assert!(
            sniff_at <= START + 10_000 + find_max + TICK,
            "seed {seed}: sniffed at {sniff_at}"
        );
        // One chest only (the cooldown and the live chest stop a second), and a save for it.
        assert_eq!(seen.iter().filter(|tick| tick.chest().is_some()).count(), 1);
        assert!(seen[chest].changed, "the cooldown is saved with the find");
        assert!(
            seen.iter().filter(|tick| tick.changed).count() == 1,
            "only the find marks a save"
        );
        assert!(
            !animations(&seen[chest].private, dog.actor).contains(&dog_find),
            "a digging pet plays no generic find animation"
        );
        // The owner alone gets the chest: never a viewer, never a packet for any model-151 actor.
        for tick in &seen {
            assert!(
                chests(&tick.shared).is_empty(),
                "seed {seed}: a viewer got the chest"
            );
            assert!(
                !tick
                    .shared
                    .iter()
                    .any(|packet| packet.starts_with(&[35, 0, 2, 0])),
                "seed {seed}: no AddNpc reaches a viewer"
            );
        }
        // What the client is told: the tier's name and texture, clickable, a unique guid.
        let tier = tier_of(&spawn);
        assert_eq!(spawn.texture_alias, tier["texture_alias"].as_str().unwrap());
        assert_eq!(
            spawn.composite_effect_id,
            tier["effect_id"].as_i64().unwrap() as i32,
            "T828: the chest carries the tier's reward sparkle"
        );
        assert_eq!(spawn.is_interactable, 1);
        assert_eq!(spawn.model_id, CHEST_MODEL);
        assert_eq!(
            spawn.unknown_float_51, 5.0,
            "the X-prompt radius is the open range"
        );
        assert_ne!(spawn.actor_id, dog.actor);
        let pet = dog.service.view(OWNER).unwrap();
        assert!(
            distance(spot(&spawn), pet.position) <= 2.01,
            "beside the pet"
        );
        assert!(
            distance(spot(&spawn), dog.owner) <= distance(pet.position, dog.owner),
            "toward the owner"
        );
        assert_eq!(
            dog.found_at(),
            Some(WALL),
            "the cooldown starts when the chest appears"
        );
    }
}

#[test]
fn a_pet_whose_model_declares_no_dig_hunts_without_the_animations() {
    // The importer marks the models whose ADR declares both slots; a cat's does not, so it never
    // plays sniff/dig. T828: it still shows the find, playing its first declared trick animation
    // as the chest appears.
    let content = PetContent::bundled().unwrap();
    let cat_model = content.model(CAT).unwrap();
    assert!(!cat_model.digs);
    let find = cat_model.find_animation_id().unwrap();
    assert_eq!(find, 5004, "the cat's first PetTricks animation row");
    let (service, pet) = model_pet(content, CAT);
    let mut cat = Dig::new(service, pet, 3, START);
    let seen = cat.run(90_000);
    let found = seen.iter().position(|tick| tick.chest().is_some()).unwrap();
    assert_eq!(seen.iter().filter(|tick| tick.chest().is_some()).count(), 1);
    assert!(
        seen.iter()
            .all(|tick| slots(&tick.private).is_empty() && slots(&tick.shared).is_empty()),
        "no hunt animation without the slots"
    );
    assert!(
        animations(&seen[found].private, cat.actor).contains(&find),
        "the owner sees the find"
    );
    assert!(
        animations(&seen[found].shared, cat.actor).contains(&find),
        "the viewers see the find"
    );
    let spawn = seen[found].chest().unwrap();
    let tier = tier_of(&spawn);
    assert_eq!(
        spawn.composite_effect_id,
        tier["effect_id"].as_i64().unwrap() as i32
    );
}

#[test]
fn a_pet_with_no_declared_animation_finds_by_the_chests_effect_alone() {
    // The sparkle is the one sign every model can show; a model with no animation row at all
    // still gets it.
    let content = content_with(|document| {
        let model = model_row(document, i64::from(CAT));
        let model = model.as_object_mut().unwrap();
        model.remove("idle");
        model.remove("trick_animations");
    });
    assert!(content.model(CAT).unwrap().find_animation_id().is_none());
    let (service, pet) = model_pet(content, CAT);
    let mut cat = Dig::new(service, pet, 3, START);
    let seen = cat.run(90_000);
    let found = seen.iter().position(|tick| tick.chest().is_some()).unwrap();
    assert!(animations(&seen[found].private, cat.actor).is_empty());
    assert!(animations(&seen[found].shared, cat.actor).is_empty());
    let spawn = seen[found].chest().unwrap();
    assert!(
        spawn.composite_effect_id > 0,
        "the chest itself marks the find"
    );
}

/// Whether the pet unearths a chest within `ms`.
fn digs(dig: &mut Dig, ms: u64) -> bool {
    dig.until_chest(ms).is_some()
}

#[test]
fn a_pet_with_a_low_mood_a_vanity_pet_and_a_busy_one_never_dig() {
    let long = 5 * 60_000;
    for seed in SEEDS {
        // Control: the happy dog digs well within the window.
        assert!(digs(&mut Dig::dog(seed), long), "control, seed {seed}");
        // Mood under the 25 threshold: every need at 20.
        let (service, pet) = dog();
        let mut sad = Dig::new(service, pet, seed, SAD_START);
        assert!(!digs(&mut sad, long), "a sad pet, seed {seed}");
        assert_eq!(sad.found_at(), None);
        // A vanity pet with full needs.
        let content = PetContent::bundled().unwrap();
        let definition = content.definition_for_item(CRAB_ITEM).unwrap();
        let (mut service, _) = dog();
        let crab = service.adopt_vendor_grant(2, &definition, 0).unwrap();
        assert!(
            !digs(&mut Dig::new(service, crab, seed, START), long),
            "a vanity pet, seed {seed}"
        );
        // The owner is busy with another activity (the start gate closed, as the open reads it).
        let mut busy = Dig::dog(seed);
        busy.busy = true;
        assert!(!digs(&mut busy, long), "a busy owner, seed {seed}");
        // The owner keeps walking, so the pet never settles.
        let mut walking = Dig::dog(seed);
        for _ in 0..long / TICK {
            walking.owner[2] += 1.0;
            walking.step();
        }
        assert_eq!(walking.found_at(), None, "a walking owner, seed {seed}");
        assert!(!walking.service.has_treasure());
        // The owner is in water.
        let mut wet = Dig::dog(seed);
        wet.send(&water(true));
        assert!(!digs(&mut wet, long), "water, seed {seed}");
        // The pet was sent away.
        let mut away = Dig::dog(seed);
        away.send(&DISMISS);
        assert!(!digs(&mut away, long), "a dismissed pet, seed {seed}");
        // Lying down: a pet that rests (the default is after 90 s) hunts only when it stands again.
        let slow = tuned(|treasure| {
            treasure["find_min_ms"] = json!(120_000);
            treasure["find_max_ms"] = json!(120_000);
        });
        assert!(
            !digs(&mut Dig::on(slow, seed), long),
            "a pet that lay down, seed {seed}"
        );
        let no_rest = content_with(|document| {
            document["treasure"]["find_min_ms"] = json!(120_000);
            document["treasure"]["find_max_ms"] = json!(120_000);
            document["care"]["idle"]["rest_after_ms"] = json!(3_600_000);
        });
        assert!(
            digs(&mut Dig::on(no_rest, seed), long),
            "the same pet that never rests digs"
        );
    }
}

#[test]
fn the_mood_threshold_and_the_find_chance_are_live_data() {
    for seed in SEEDS {
        // Mood starts at the shipped starting level and falls while summoned: a threshold a few
        // points under it holds for the first minute, one at the starting level never.
        let start = pets_json()["need_policy"]["initial"].as_f64().unwrap();
        let digs_at = |min_mood: f64| {
            Dig::on(
                tuned(|treasure| treasure["min_mood"] = json!(min_mood)),
                seed,
            )
            .until_chest(5 * 60_000)
            .is_some()
        };
        assert!(digs_at(start - 5.0), "seed {seed}");
        assert!(!digs_at(start), "seed {seed}");
    }
    // A find chance under 1 makes a roll fail now and then and wait again; the pet still digs (this
    // pet never lies down, which would end a hunt).
    let window = rule("find_max_ms") + 7_000 + 10_000 + TICK;
    let (mut late, mut all) = (0, 0);
    for seed in 0..16 {
        let unlucky = content_with(|document| {
            document["treasure"]["find_chance"] = json!(0.1);
            document["care"]["idle"]["rest_after_ms"] = json!(3_600_000);
        });
        let mut dog = Dig::on(unlucky, seed);
        all += 1;
        if dog.until_chest(window).is_none() {
            late += 1;
            assert!(
                dog.until_chest(40 * 60_000).is_some(),
                "a failed roll rolls again, seed {seed}"
            );
        }
        assert!(
            Dig::dog(seed).until_chest(window).is_some(),
            "a find chance of 1 always digs in time, seed {seed}"
        );
    }
    assert!(
        (1..all).contains(&late),
        "{late} of {all} found late at a chance of 0.1"
    );
}

/// Canary: stop `may_hunt` keeping the cooldown, and a second chest comes inside it.
#[test]
fn the_cooldown_blocks_a_second_find_and_survives_a_save_and_a_reload() {
    let cooldown = rule("cooldown_ms") / 1_000;
    let lifetime = rule("lifetime_ms");
    let mut dog = Dig::dog(5);
    let first = dog.until_chest(120_000).unwrap();
    let guid = first.chest().unwrap().actor_id;
    assert_eq!(dog.found_at(), Some(WALL));
    // The chest expires; the pet is happy and settled, yet digs nothing for the whole cooldown.
    dog.wall = WALL + cooldown - 1;
    let later = dog.run(lifetime + 10 * 60_000);
    assert!(
        later.iter().any(|tick| removals(&tick.private, guid) == 1),
        "the chest left"
    );
    assert!(
        later.iter().all(|tick| tick.chest().is_none()),
        "no second find inside the cooldown"
    );
    assert_eq!(dog.found_at(), Some(WALL), "nothing touched the saved time");
    // The cooldown over (and the owner moves, so a pet that lay down stands again): it finds again.
    dog.wall = WALL + cooldown;
    dog.owner = [0.0, 0.0, 6.0];
    let second = dog
        .until_chest(120_000)
        .expect("a pet off its cooldown digs again");
    assert_ne!(
        second.chest().unwrap().actor_id,
        guid,
        "a new chest, a new guid"
    );
    assert_eq!(dog.found_at(), Some(WALL + cooldown));

    // A save and a reload (a reconnect): the saved time keeps the pet from digging again.
    let mut once = Dig::dog(6);
    once.until_chest(120_000).unwrap();
    let document = once.service.document_value().unwrap();
    assert_eq!(found_at(&document, once.pet), Some(WALL));
    let reload = |document: Value, wall: u64| {
        let service =
            PetService::from_document(PetContent::bundled().unwrap(), Some(document)).unwrap();
        let mut dog = Dig::new(service, once.pet, 7, START);
        dog.wall = wall;
        dog
    };
    for wall in [WALL + 3_600, WALL + cooldown - 1] {
        assert!(
            !digs(&mut reload(document.clone(), wall), 5 * 60_000),
            "wall {wall}"
        );
    }
    assert!(digs(
        &mut reload(document.clone(), WALL + cooldown),
        5 * 60_000
    ));
    // A pet that never dug saves no field, and an older save without it loads and digs.
    let fresh = Dig::dog(8);
    let before = fresh.service.document_value().unwrap();
    assert!(before["pets"][0].get("treasure_found_at").is_none());
    assert!(digs(&mut reload(before.clone(), WALL), 5 * 60_000));
    // A saved time far after the wall clock (the clock was set back by more than a cooldown) means
    // nothing, so it never freezes a pet; one inside the cooldown still counts.
    let mut set_back = document.clone();
    set_back["pets"][0]["treasure_found_at"] = json!(WALL + 30 * cooldown);
    assert!(digs(&mut reload(set_back, WALL), 5 * 60_000));
    let mut near = document.clone();
    near["pets"][0]["treasure_found_at"] = json!(WALL + 60);
    assert!(!digs(&mut reload(near, WALL), 5 * 60_000));
}

#[test]
fn a_pet_that_stops_being_settled_cancels_the_dig_and_no_cooldown_starts() {
    // Whether the pet is still settled is the world tick's call (`follow_owner` runs first), so this
    // goes through `Gameplay::pet_tick`.
    let fixture = fixture();
    let (mut game, pet) = session(&fixture, None);
    let mut owner = who();
    let (mut at, mut sniff) = (START, None);
    while sniff.is_none() && at < START + 120_000 {
        at += TICK;
        let out = game.pet_tick(&owner, at).expect("a pet is out");
        sniff = (slots(&out.replies) == [SNIFF]).then_some(at);
    }
    let sniff = sniff.expect("the pet never sniffed");
    // The owner walks off in the middle of the sniff: no dig, no chest, no cooldown.
    for _ in 0..20 {
        owner.position[2] += 1.0;
        at += TICK;
        let out = game.pet_tick(&owner, at).expect("a pet is out");
        assert!(
            chests(&out.replies).is_empty() && slots(&out.replies) != [DIG],
            "after {sniff}"
        );
    }
    assert_eq!(
        found_at(&game.pets_document().unwrap(), pet),
        None,
        "an interrupted dig costs nothing"
    );
    // Standing still again, the hunt starts over and finds.
    dig_up(&mut game, &owner, at);
    assert!(
        found_at(&game.pets_document().unwrap(), pet).is_some(),
        "the cooldown starts with the find"
    );
}

#[test]
fn the_dig_and_the_care_actions_refuse_each_other() {
    // A dog with the bowl, the food, the shampoo and the brush fitted, and a 20 s wait, so a feed
    // at 18 s lands inside it.
    let ready = |content: PetContent| {
        let (mut service, pet) = dog_with(content);
        let mut fixture = kit();
        send(&mut service, &mut fixture, &fit(pet, 5, BOWL_ITEM), START);
        send(&mut service, &mut fixture, &fit(pet, 6, FOOD_ITEM), START);
        send(
            &mut service,
            &mut fixture,
            &fit(pet, 1, SHAMPOO_ITEM),
            START,
        );
        send(&mut service, &mut fixture, &fit(pet, 4, BRUSH_ITEM), START);
        let mut dog = Dig::new(service, pet, 4, START);
        dog.fixture = fixture;
        dog
    };
    let steady = || {
        content_with(|document| {
            document["treasure"]["find_min_ms"] = json!(20_000);
            document["treasure"]["find_max_ms"] = json!(20_000);
        })
    };
    // Control: undisturbed, the pet sniffs 20 s after it settled.
    let mut dog = ready(steady());
    let mut first = None;
    while first.is_none() && dog.at < START + 60_000 {
        let tick = dog.step();
        first = (slots(&tick.private) == [SNIFF]).then_some(tick.at);
    }
    let undisturbed = first.expect("the control sniffs");
    assert!(undisturbed < START + 30_000, "sniffed at {undisturbed}");
    // A feed that starts while it waits cancels the hunt: no sniff during the feed, and the wait
    // starts over once the pet has eaten.
    let mut dog = ready(steady());
    let (mut fed_at, mut first) = (None, None);
    while first.is_none() && dog.at < START + 120_000 {
        let tick = dog.step();
        if fed_at.is_none() && dog.at >= START + 18_000 {
            let wire = dog.send(&feed(dog.pet));
            assert_eq!(notify_kinds(&wire.private), [1], "the feed starts");
            fed_at = Some(dog.at);
        }
        first = (slots(&tick.private) == [SNIFF]).then_some(tick.at);
    }
    let fed_at = fed_at.expect("fed");
    let sniffed = first.expect("it hunts again after the feed");
    let feed_ms = pets_json()["care"]["feed_end_ms"].as_u64().unwrap();
    assert!(
        sniffed >= fed_at + feed_ms + 20_000 - TICK,
        "fed at {fed_at}, sniffed at {sniffed}"
    );
    // While the pet sniffs or digs, every care action is refused and starts nothing.
    let mut dog = ready(steady());
    while slots(&dog.step().private) != [SNIFF] {
        assert!(dog.at < START + 120_000, "the pet never sniffed");
    }
    let before = dog.service.document_value();
    // A refused feed answers 53/20 kind 6 (unlock) so the care panel never sticks; kind 1 would be a start.
    let refused = dog.send(&feed(dog.pet));
    assert_eq!(
        notify_kinds(&refused.private),
        [6],
        "no feed starts mid-dig"
    );
    assert_eq!(
        dog.service.document_value(),
        before,
        "a refused feed changes nothing"
    );
    let refused = dog.send(&groom(dog.pet));
    assert_eq!(
        notify_kinds(&refused.private),
        [6],
        "no groom starts mid-dig"
    );
    assert_eq!(
        dog.service.document_value(),
        before,
        "a refused groom changes nothing"
    );
    let refused = dog.send(&perform_trick(TRICK));
    assert_eq!(
        notify_kinds(&refused.private),
        [6],
        "no trick starts mid-dig"
    );
    assert_eq!(
        dog.service.document_value(),
        before,
        "a refused trick changes nothing"
    );
    assert!(dog.until_chest(30_000).is_some(), "the dig goes on");
}

#[test]
fn the_tiers_follow_their_weights_and_pay_from_their_tables() {
    let table = treasure();
    let tiers = table["tiers"].as_array().unwrap();
    let weight: u64 = tiers
        .iter()
        .map(|tier| tier["weight"].as_u64().unwrap())
        .sum();
    let bonus: BTreeSet<i64> = table["bonus"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| id.as_i64().unwrap())
        .collect();
    let (service, pet) = dog();
    let (mut counts, mut bonuses, mut finds) = (BTreeMap::new(), 0u32, 0u32);
    let mut bag = Some(kit());
    for seed in 0..800 {
        let mut dog = Dig::with_bag(service.clone(), pet, seed, START, bag.take().unwrap());
        let seen = dog.until_chest(120_000).unwrap();
        let spawn = seen.chest().unwrap();
        let tier = tier_of(&spawn);
        *counts.entry(spawn.name_string_id).or_insert(0u32) += 1;
        finds += 1;
        // The chest holds exactly what the table says: its coins and one item of the tier (or a bonus one).
        let grant = dog
            .service
            .open_treasure(spawn.actor_id, OWNER, dog.owner, dog.at)
            .unwrap();
        assert_eq!(u64::from(grant.coins), tier["coins"].as_u64().unwrap());
        assert_eq!(grant.items.len(), 1);
        let item = &grant.items[0];
        if bonus.contains(&i64::from(item.definition_id)) {
            bonuses += 1;
            assert_eq!(item.count, 1);
        } else {
            let row = tier["items"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["definition_id"] == item.definition_id)
                .unwrap_or_else(|| panic!("{} is in no table", item.definition_id));
            assert_eq!(u64::from(item.count), row["count"].as_u64().unwrap());
        }
        assert_eq!(grant.display.player_guid, OWNER);
        assert_eq!(grant.display.source_guid, spawn.actor_id);
        assert_eq!(grant.display.name_id, spawn.name_string_id);
        bag = Some(dog.fixture);
    }
    for tier in tiers {
        let found = counts.get(&(tier["name_id"].as_i64().unwrap() as i32));
        let share = f64::from(found.copied().unwrap_or(0)) / f64::from(finds);
        let expected = tier["weight"].as_f64().unwrap() / weight as f64;
        assert!(found.is_some(), "{} never came up", tier["key"]);
        assert!(
            (share - expected).abs() < 0.05,
            "{}: {share} vs {expected}",
            tier["key"]
        );
    }
    let share = f64::from(bonuses) / f64::from(finds);
    assert!((share - 0.1).abs() < 0.05, "bonus share {share}");
}

#[test]
fn opening_pays_the_grant_once_and_refuses_everything_else_without_a_change() {
    let mut dog = Dig::dog(9);
    let chest = dog.until_chest(120_000).unwrap().chest().unwrap();
    let guid = chest.actor_id;
    let near = dog.owner;
    let now = dog.at;
    let before = dog.service.document_value();
    // Refusals: another owner, too far, a guid that is not the chest, a chest past its lifetime.
    assert_eq!(
        dog.service
            .open_treasure(guid, OWNER + 1, near, now)
            .unwrap_err(),
        TreasureRefusal::NotYours
    );
    assert_eq!(
        dog.service
            .open_treasure(guid, OWNER, [200.0, 0.0, 0.0], now)
            .unwrap_err(),
        TreasureRefusal::TooFar
    );
    assert_eq!(
        dog.service
            .open_treasure(guid, OWNER, [f32::NAN, 0.0, 0.0], now)
            .unwrap_err(),
        TreasureRefusal::TooFar
    );
    assert_eq!(
        dog.service
            .open_treasure(guid ^ 1, OWNER, near, now)
            .unwrap_err(),
        TreasureRefusal::Gone
    );
    assert_eq!(
        dog.service
            .open_treasure(guid, OWNER, near, now + rule("lifetime_ms"))
            .unwrap_err(),
        TreasureRefusal::Gone
    );
    assert!(
        dog.service.is_treasure_chest(guid),
        "no refusal touched the chest"
    );
    assert_eq!(dog.service.document_value(), before);
    // Reading the grant twice gives the same grant: opening rolls nothing.
    let grant = dog.service.open_treasure(guid, OWNER, near, now).unwrap();
    assert_eq!(
        dog.service.open_treasure(guid, OWNER, near, now).unwrap(),
        grant
    );
    // Clearing removes the chest for the owner alone; a second open and a second clear find nothing.
    assert!(dog.service.is_treasure_chest(guid));
    assert_eq!(dog.service.clear_treasure(), vec![pet_removal_packet(guid)]);
    assert!(!dog.service.has_treasure());
    assert_eq!(
        dog.service
            .open_treasure(guid, OWNER, near, now)
            .unwrap_err(),
        TreasureRefusal::Gone,
        "a chest opens once"
    );
    assert!(dog.service.clear_treasure().is_empty());
}

#[test]
fn an_unopened_chest_expires_for_the_owner_alone_and_its_cooldown_stays() {
    let lifetime = rule("lifetime_ms");
    let mut dog = Dig::dog(11);
    let found = dog.until_chest(120_000).unwrap();
    let guid = found.chest().unwrap().actor_id;
    let seen = dog.run(lifetime + 2_000);
    let gone: Vec<&Seen> = seen
        .iter()
        .filter(|tick| removals(&tick.private, guid) > 0)
        .collect();
    assert_eq!(gone.len(), 1, "one removal");
    assert!(
        (found.at + lifetime..found.at + lifetime + TICK).contains(&gone[0].at),
        "removed at {} after the find at {}",
        gone[0].at,
        found.at
    );
    assert!(
        gone[0]
            .shared
            .iter()
            .all(|packet| !packet.starts_with(&[35, 0, 3, 0])),
        "owner only"
    );
    assert!(!dog.service.has_treasure());
    assert_eq!(
        dog.service
            .open_treasure(guid, OWNER, dog.owner, dog.at)
            .unwrap_err(),
        TreasureRefusal::Gone
    );
    assert_eq!(
        dog.found_at(),
        Some(WALL),
        "the cooldown outlives the chest"
    );
    // The tick keeps expiring a chest after its pet was dismissed.
    let mut away = Dig::dog(12);
    away.until_chest(120_000).unwrap();
    away.send(&DISMISS);
    assert!(away.service.has_treasure());
    let seen = away.run(lifetime + 2_000);
    assert!(seen.iter().any(|tick| removals_of_any(&tick.private)));
    assert!(!away.service.has_treasure());
}

#[test]
fn a_clock_that_ran_backwards_or_content_without_the_block_hunts_nothing_and_breaks_nothing() {
    // The pet's needs clock is at 20 s after its summon. A 2 s find wait armed at 1 s is due at 3 s,
    // so every tick up to 20 s runs with the clock still behind the pet's: the needs cannot be
    // decayed there (the mood unknown), and the hunt must not start.
    let quick = tuned(|treasure| {
        treasure["find_min_ms"] = json!(2_000);
        treasure["find_max_ms"] = json!(2_000);
    });
    let (service, pet) = dog_with(quick);
    let mut dog = Dig::new(service, pet, 1, 20_000);
    for at in (1_000..20_000).step_by(TICK as usize) {
        let seen = dog.tick(at);
        assert!(!seen.eventful(), "no hunt, no chest, no save at {at}");
    }
    assert!(!dog.service.has_treasure());
    assert_eq!(
        dog.found_at(),
        None,
        "nothing was saved before the clock recovered"
    );
    // The same session recovers once the clock is sane.
    assert!(dog.until_chest(5 * 60_000).is_some());
    assert_eq!(dog.found_at(), Some(WALL), "and then the find is saved");
    // Content without a `treasure` block: the tick is a no-op.
    let none = content_with(|document| {
        document.as_object_mut().unwrap().remove("treasure");
    });
    assert!(!digs(&mut Dig::on(none, 1), 5 * 60_000));
}

#[test]
fn bad_treasure_content_is_refused_at_load() {
    let edits: Vec<(&str, Box<dyn Fn(&mut Value)>)> = vec![
        ("an unknown key", Box::new(|t| t["extra"] = json!(1))),
        (
            "a mood over 100",
            Box::new(|t| t["min_mood"] = json!(101.0)),
        ),
        ("a negative mood", Box::new(|t| t["min_mood"] = json!(-1.0))),
        (
            "a find wait under a second",
            Box::new(|t| t["find_min_ms"] = json!(999)),
        ),
        (
            "a find window upside down",
            Box::new(|t| {
                t["find_min_ms"] = json!(40_000);
                t["find_max_ms"] = json!(30_000);
            }),
        ),
        (
            "a find wait over an hour",
            Box::new(|t| t["find_max_ms"] = json!(3_600_001)),
        ),
        (
            "a zero find chance",
            Box::new(|t| t["find_chance"] = json!(0.0)),
        ),
        (
            "a find chance over 1",
            Box::new(|t| t["find_chance"] = json!(1.5)),
        ),
        (
            "a cooldown over 30 days",
            Box::new(|t| t["cooldown_ms"] = json!(30 * 24 * 3_600_000u64 + 1)),
        ),
        (
            "a lifetime under a second",
            Box::new(|t| t["lifetime_ms"] = json!(999)),
        ),
        (
            "a chest model of 0",
            Box::new(|t| t["chest_model_id"] = json!(0)),
        ),
        (
            "a spawn distance past 20",
            Box::new(|t| t["spawn_distance"] = json!(21.0)),
        ),
        (
            "a negative spawn distance",
            Box::new(|t| t["spawn_distance"] = json!(-1.0)),
        ),
        (
            "an open range under 1",
            Box::new(|t| t["open_range"] = json!(0.5)),
        ),
        (
            "an open range over 50",
            Box::new(|t| t["open_range"] = json!(51.0)),
        ),
        (
            "a sniff animation of 0",
            Box::new(|t| t["sniff"]["animation_id"] = json!(0)),
        ),
        (
            "a dig of no time",
            Box::new(|t| t["dig"]["duration_ms"] = json!(0)),
        ),
        (
            "a dig of over a minute",
            Box::new(|t| t["dig"]["duration_ms"] = json!(60_001)),
        ),
        ("no tiers", Box::new(|t| t["tiers"] = json!([]))),
        (
            "nine tiers",
            Box::new(|t| {
                let tier = t["tiers"][0].clone();
                let tiers: Vec<Value> = (0..9)
                    .map(|i| {
                        let mut tier = tier.clone();
                        tier["key"] = json!(format!("t{i}"));
                        tier
                    })
                    .collect();
                t["tiers"] = json!(tiers);
            }),
        ),
        (
            "a repeated tier key",
            Box::new(|t| t["tiers"][1]["key"] = json!("common")),
        ),
        (
            "an empty tier key",
            Box::new(|t| t["tiers"][0]["key"] = json!("")),
        ),
        (
            "a tier of weight 0",
            Box::new(|t| t["tiers"][0]["weight"] = json!(0)),
        ),
        (
            "a tier with no name",
            Box::new(|t| t["tiers"][0]["name_id"] = json!(0)),
        ),
        (
            "a tier with no texture",
            Box::new(|t| t["tiers"][0]["texture_alias"] = json!("")),
        ),
        (
            "a chest effect of 0",
            Box::new(|t| t["tiers"][0]["effect_id"] = json!(0)),
        ),
        (
            "a chest effect over the sanity bound",
            Box::new(|t| t["tiers"][0]["effect_id"] = json!(100_001)),
        ),
        (
            "a tier paying nothing",
            Box::new(|t| {
                t["tiers"][0]["coins"] = json!(0);
                t["tiers"][0]["items"] = json!([]);
            }),
        ),
        (
            "coins past the sanity bound",
            Box::new(|t| t["tiers"][0]["coins"] = json!(1_000_001)),
        ),
        (
            "a reward row of weight 0",
            Box::new(|t| t["tiers"][0]["items"][0]["weight"] = json!(0)),
        ),
        (
            "a reward row of count 0",
            Box::new(|t| t["tiers"][0]["items"][0]["count"] = json!(0)),
        ),
        (
            "a reward row of count 1000",
            Box::new(|t| t["tiers"][0]["items"][0]["count"] = json!(1_000)),
        ),
        (
            "a reward row with a negative item",
            Box::new(|t| t["tiers"][0]["items"][0]["definition_id"] = json!(-5)),
        ),
        (
            "a bonus chance over 1",
            Box::new(|t| t["bonus"]["chance"] = json!(1.1)),
        ),
        (
            "a bonus with a chance and no items",
            Box::new(|t| t["bonus"]["items"] = json!([])),
        ),
        (
            "a repeated bonus item",
            Box::new(|t| t["bonus"]["items"][1] = t["bonus"]["items"][0].clone()),
        ),
        (
            "a bonus item of 0",
            Box::new(|t| t["bonus"]["items"][0] = json!(0)),
        ),
        (
            "no source",
            Box::new(|t| {
                t.as_object_mut().unwrap().remove("source");
            }),
        ),
    ];
    for (why, edit) in edits {
        assert!(
            loads(|document| edit(&mut document["treasure"])).is_err(),
            "{why}"
        );
    }
    // The knobs a tuner reaches for stay legal: no cooldown, a certain bonus, coins only, a quick pet.
    for (why, edit) in [
        (
            "no cooldown",
            Box::new(|t: &mut Value| t["cooldown_ms"] = json!(0)) as Box<dyn Fn(&mut Value)>,
        ),
        (
            "a certain bonus",
            Box::new(|t| t["bonus"]["chance"] = json!(1.0)),
        ),
        (
            "no bonus block",
            Box::new(|t| {
                t.as_object_mut().unwrap().remove("bonus");
            }),
        ),
        (
            "a coins-only tier",
            Box::new(|t| t["tiers"][0]["items"] = json!([])),
        ),
        (
            "an items-only tier",
            Box::new(|t| t["tiers"][0]["coins"] = json!(0)),
        ),
        (
            "a one-second wait",
            Box::new(|t| {
                t["find_min_ms"] = json!(1_000);
                t["find_max_ms"] = json!(1_000);
            }),
        ),
        (
            "another chest sparkle",
            Box::new(|t| t["tiers"][0]["effect_id"] = json!(5596)),
        ),
    ] {
        assert!(
            loads(|document| edit(&mut document["treasure"])).is_ok(),
            "{why}"
        );
    }
}

/// The shipped table pays only items the shipped client item catalogue defines, and the models
/// that play the hunt animations declare both slots.
#[test]
fn every_reward_item_is_in_the_client_catalogue() {
    let table = treasure();
    let mut ids: BTreeSet<i32> = table["bonus"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| id.as_i64().unwrap() as i32)
        .collect();
    for tier in table["tiers"].as_array().unwrap() {
        ids.extend(
            tier["items"]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| row["definition_id"].as_i64().unwrap() as i32),
        );
    }
    let inventory =
        Inventory::load(&round70_content::source(), None).expect("the item catalogue loads");
    let undefined: Vec<i32> = ids
        .iter()
        .copied()
        .filter(|id| !inventory.has_definition(*id))
        .collect();
    assert!(undefined.is_empty(), "paid but undefined: {undefined:?}");
    assert_eq!(table["tiers"].as_array().unwrap().len(), 3);
}

/// T828: the shipped tier effects are the client's chest-reward sparkles
/// (`ActorCompositeEffectDefinitions.xml`: 5595 small, 5596 medium, 5597 large), so a find is
/// visible with no pet animation.
#[test]
fn the_tier_effects_are_the_clients_chest_reward_sparkles() {
    let ids: Vec<i64> = treasure()["tiers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tier| tier["effect_id"].as_i64().unwrap())
        .collect();
    assert_eq!(ids, [5595, 5596, 5597]);
}

// ---- Gameplay ----

fn who() -> CharacterState {
    CharacterState {
        guid: CHARACTER,
        zone_id: 0,
        position: [-440.0, 0.8, 1292.0],
        rotation: [1.0, 0.0, 0.0],
    }
}

/// Every item the shipped table can pay.
fn reward_items() -> BTreeSet<i64> {
    let table = treasure();
    let mut ids: BTreeSet<i64> = table["bonus"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| id.as_i64().unwrap())
        .collect();
    for tier in table["tiers"].as_array().unwrap() {
        ids.extend(
            tier["items"]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| row["definition_id"].as_i64().unwrap()),
        );
    }
    ids
}

/// The minimal fixture with a bag definition for every reward item (the fixture's own catalogue
/// is the small opening one; the real catalogue is checked above).
fn fixture() -> Fixture {
    let fixture = Fixture::minimal(CHARACTER);
    let rows: Vec<Value> = reward_items()
        .iter()
        .map(|id| item_catalogue::sized_row(*id as u32, 64))
        .collect();
    item_catalogue::write(fixture.path(), &rows);
    fixture
}

/// A gameplay session with the dog adopted and summoned at `START`.
fn session(fixture: &Fixture, state: Option<GameplayState>) -> (Gameplay, u32) {
    let (service, pet) = dog();
    let mut game = Gameplay::load(fixture.path(), state).unwrap();
    game.adopt_pets(service.document_value()).unwrap();
    game.initialize_world(&who(), START).unwrap();
    game.handle_in_world(&summon(pet), &who(), START).unwrap();
    (game, pet)
}

/// 127/23 grants the starter lot and 127/19 enters it: the player is inside a house, where the
/// start gate is closed (so the open refuses) and the pets gate is open. The content must carry
/// `houses.json` (`Fixture::copy_content`); the enter body is the shared
/// `gateway_packets::enter_house`.
fn grant_and_enter_house(game: &mut Gameplay, who: &CharacterState, now: u64) {
    let granted = game.handle_in_world(&[127, 0, 23, 0], who, now).unwrap();
    assert!(granted.changed, "the starter lot is granted");
    let lot = game.state().housing.expect("the grant saves the lot").lots[0].lot_guid;
    let inside = game
        .handle_in_world(&gateway_packets::enter_house(lot), who, now + 1)
        .unwrap();
    assert!(inside.scene_zoning.is_some(), "the lot scene loads");
}

/// World ticks until the chest appears (at most two minutes): the tick, its outcome and the chest.
fn dig_up(game: &mut Gameplay, who: &CharacterState, from: u64) -> (u64, Outcome, NpcSpawn) {
    let mut at = from;
    while at < from + 120_000 {
        at += TICK;
        let out = game.pet_tick(who, at).expect("a pet is out");
        assert!(
            chests(&out.nearby).is_empty()
                && !out
                    .nearby
                    .iter()
                    .any(|packet| packet.starts_with(&[35, 0, 2, 0])),
            "a viewer must never get the chest: {:?}",
            out.nearby
        );
        if let Some(spawn) = chests(&out.replies).into_iter().next() {
            return (at, out, spawn);
        }
    }
    panic!("no chest within two minutes");
}

fn click(guid: u64) -> Vec<u8> {
    NpcInteractRequest {
        target: guid,
        trailing_flag: false,
    }
    .encode()
}

fn held(state: &GameplayState) -> BTreeMap<i32, u32> {
    let mut held = BTreeMap::new();
    for item in &state.inventory.items {
        *held.entry(item.definition_id).or_insert(0) += item.count;
    }
    held
}

fn said(out: &Outcome, text: &str) -> bool {
    out.replies.contains(&chat::system(text))
}

#[test]
fn the_owner_digs_up_a_chest_and_opens_it_once_through_gameplay() {
    let fixture = fixture();
    let (mut game, pet) = session(&fixture, None);
    let owner = who();
    let (at, found, spawn) = dig_up(&mut game, &owner, START);
    assert!(found.changed, "the find saves the cooldown");
    let guid = spawn.actor_id;
    let tier = tier_of(&spawn);
    let before = game.state();
    let saved = game.pets_document().unwrap();
    assert!(
        found_at(&saved, pet).is_some_and(|secs| secs > 1_700_000_000),
        "a wall-clock stamp"
    );

    // The owner clicks the chest (26/8): the table's coins and one item, the chest removed.
    let opened = game
        .handle_in_world(&click(guid), &owner, at + 1_000)
        .unwrap();
    assert!(opened.changed);
    assert_eq!(removals(&opened.replies, guid), 1, "the chest is removed");
    assert_eq!(removals(&opened.nearby, guid), 0, "and only for the owner");
    assert!(
        opened
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[50, 0, 1])),
        "the reward toast"
    );
    let after = game.state();
    assert_eq!(
        i64::from(after.inventory.coins) - i64::from(before.inventory.coins),
        tier["coins"].as_i64().unwrap()
    );
    let (was, now) = (held(&before), held(&after));
    let gained: Vec<(i32, u32)> = now
        .iter()
        .filter_map(|(id, count)| {
            let gain = count - was.get(id).copied().unwrap_or(0);
            (gain > 0).then_some((*id, gain))
        })
        .collect();
    assert_eq!(gained.len(), 1, "one item: {gained:?}");
    let (item, count) = gained[0];
    let in_tier = tier["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["definition_id"] == item && row["count"] == count);
    let in_bonus = treasure()["bonus"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|id| *id == item)
        && count == 1;
    assert!(in_tier || in_bonus, "{item} x{count} is not in the table");

    // Canary: keep the chest on open and this second click pays again.
    let again = game
        .handle_in_world(&click(guid), &owner, at + 2_000)
        .unwrap();
    assert!(!again.changed, "a second open pays nothing");
    assert_eq!(game.state(), after, "and changes nothing");
    assert_eq!(removals(&again.replies, guid), 0);
    // The chest does not come back: the pet rests, and the saved cooldown is the one from the find.
    for step in 1..=120 {
        let out = game.pet_tick(&owner, at + 2_000 + step * TICK).unwrap();
        assert!(chests(&out.replies).is_empty() && !out.changed);
    }
    assert_eq!(
        game.pets_document().unwrap(),
        saved,
        "opening saves no second stamp"
    );
}

/// F819 (T533 review finding 8): the world tick's order, through `Gameplay::pet_tick`: the pet
/// settles (`follow_pet` runs first), the sniff plays, then the dig, and only then the chest, for
/// the owner and the viewers. The `Dig` harness above composes the three service calls for the
/// rules the service owns; a reorder in `pet_tick` fails here.
#[test]
fn the_world_tick_plays_the_hunt_in_order_for_the_owner_and_the_viewers() {
    let fixture = fixture();
    let sniff_ms = treasure()["sniff"]["duration_ms"].as_u64().unwrap();
    let dig_ms = treasure()["dig"]["duration_ms"].as_u64().unwrap();
    let (find_min, find_max) = (rule("find_min_ms"), rule("find_max_ms"));
    for _ in 0..3 {
        let (mut game, _) = session(&fixture, None);
        let owner = who();
        let (mut at, mut events, mut chest) = (START, Vec::new(), None);
        while chest.is_none() && at < START + 120_000 {
            at += TICK;
            let out = game.pet_tick(&owner, at).expect("a pet is out");
            events.extend(slots(&out.replies).into_iter().map(|slot| (at, slot)));
            assert_eq!(
                slots(&out.replies),
                slots(&out.nearby),
                "the owner and the viewers see the same hunt"
            );
            chest = chests(&out.replies)
                .into_iter()
                .next()
                .map(|spawn| (at, out, spawn));
        }
        let (chest_at, found, spawn) = chest.expect("the world tick dug up a chest");
        let played: Vec<i32> = events.iter().map(|(_, slot)| *slot).collect();
        assert_eq!(played, [SNIFF, DIG], "the world tick's order");
        let (sniff_at, dig_at) = (events[0].0, events[1].0);
        assert!(
            (sniff_ms..sniff_ms + TICK).contains(&(dig_at - sniff_at)),
            "sniff at {sniff_at}, dig at {dig_at}"
        );
        assert!(
            (dig_ms..dig_ms + TICK).contains(&(chest_at - dig_at)),
            "dig at {dig_at}, chest at {chest_at}"
        );
        // The wait is the data's window, counted from the moment the pet stood settled.
        assert!(
            sniff_at >= START + find_min && sniff_at <= START + 10_000 + find_max + TICK,
            "sniffed at {sniff_at}"
        );
        assert!(found.changed, "the find saves the cooldown");
        assert_eq!(spawn.model_id, CHEST_MODEL);
    }
}

/// F819 (T533 review finding 1): a pet in a house does not hunt. Housing closes the start gate (the
/// open's gate) but keeps the pets gate open, so before the fix a happy dog dug up a chest there,
/// every click was refused, the zone change cleared the chest, and the find and its 8 h cooldown
/// were lost. The control is the same content and the same dog, outside any lot.
#[test]
fn a_pet_in_a_house_does_not_hunt_and_a_pet_outside_it_does() {
    let fixture = fixture();
    fixture.copy_content("houses.json");
    let (mut game, pet) = session(&fixture, None);
    let owner = who();
    grant_and_enter_house(&mut game, &owner, START);
    let mut at = START;
    while at < START + 80_000 {
        at += TICK;
        let out = game.pet_tick(&owner, at).expect("the pet is out");
        assert!(
            chests(&out.replies).is_empty(),
            "no chest inside a house, at {at}"
        );
        assert!(!out.changed, "nothing is saved inside a house, at {at}");
    }
    assert_eq!(
        found_at(&game.pets_document().unwrap(), pet),
        None,
        "no cooldown was spent inside the house"
    );
    // The same dog, outside any lot, hunts and saves its find.
    let (mut world, _) = session(&fixture, None);
    let (_, out, spawn) = dig_up(&mut world, &owner, START);
    assert!(out.changed, "the outdoor find saves the cooldown");
    assert_eq!(spawn.model_id, CHEST_MODEL);
}

/// F819: the open's busy gate (T533 checklist 4). A live chest cannot be opened while another
/// activity owns the player; here a house, the cheapest start-gate occupation (it keeps the pets
/// gate open, so the pet still exists and follows). The refusal changes nothing and keeps the
/// chest, so it can be clicked again later; the free open is covered by the tests above.
#[test]
fn the_chest_cannot_be_opened_while_another_activity_owns_the_player() {
    let fixture = fixture();
    fixture.copy_content("houses.json");
    let (mut game, _) = session(&fixture, None);
    let owner = who();
    let (at, _, spawn) = dig_up(&mut game, &owner, START);
    let guid = spawn.actor_id;
    grant_and_enter_house(&mut game, &owner, at + 1);
    let before = game.state();
    let saved = game.pets_document();
    for step in 1..=3 {
        let refused = game
            .handle_in_world(&click(guid), &owner, at + step * 1_000)
            .unwrap();
        assert!(!refused.changed, "a refused open changes nothing");
        assert!(said(
            &refused,
            "Finish the active activity before opening the treasure."
        ));
        assert_eq!(removals(&refused.replies, guid), 0, "the chest stays");
        assert_eq!(game.state(), before);
        assert_eq!(game.pets_document(), saved);
    }
    // The chest still opens: the same session may click it once the lot is left (the reconnect
    // path drops the visit here, because the fixture carries no opening world content).
    assert!(game.abandon_house_for_reconnect().is_some());
    let opened = game.handle_in_world(&click(guid), &owner, at + 4).unwrap();
    assert!(opened.changed, "the free owner opens the chest");
    assert_eq!(removals(&opened.replies, guid), 1);
}

#[test]
fn a_reload_keeps_the_reward_and_the_cooldown_and_loses_the_chest() {
    let fixture = fixture();
    let (mut game, pet) = session(&fixture, None);
    let owner = who();
    let (at, _, spawn) = dig_up(&mut game, &owner, START);
    game.handle_in_world(&click(spawn.actor_id), &owner, at + 1_000)
        .unwrap();
    let state = game.state();
    let roster = game.pets_document();
    assert!(found_at(roster.as_ref().unwrap(), pet).is_some());
    // The reconnect: a fresh session from the saved character and roster.
    let mut reloaded = Gameplay::load(fixture.path(), Some(state.clone())).unwrap();
    reloaded.adopt_pets(roster).unwrap();
    reloaded.initialize_world(&owner, START).unwrap();
    assert_eq!(reloaded.state(), state, "the reward is in the save");
    reloaded
        .handle_in_world(&summon(pet), &owner, START)
        .unwrap();
    let mut at = START;
    while at < START + 10 * 60_000 {
        at += TICK;
        let out = reloaded.pet_tick(&owner, at).unwrap();
        assert!(
            chests(&out.replies).is_empty(),
            "the cooldown survived the reload"
        );
    }
    assert_eq!(held(&reloaded.state()), held(&state), "no second grant");
}

#[test]
fn a_stranger_neither_gets_nor_opens_the_chest() {
    let fixture = fixture();
    let (mut game, _) = session(&fixture, None);
    let owner = who();
    let (at, _, spawn) = dig_up(&mut game, &owner, START);
    let guid = spawn.actor_id;
    // Another player's own session: its ticks and clicks know nothing of this guid.
    let other = Fixture::minimal(CHARACTER + 1);
    let stranger = CharacterState {
        guid: CHARACTER + 1,
        ..who()
    };
    let mut theirs = Gameplay::load(other.path(), None).unwrap();
    theirs.initialize_world(&stranger, START).unwrap();
    let before = theirs.state();
    let out = theirs.handle_in_world(&click(guid), &stranger, at).unwrap();
    assert!(!out.changed && removals(&out.replies, guid) == 0);
    assert_eq!(theirs.state(), before);
    // The same click sent as another character to the owner's session: refused, chest intact.
    let before = game.state();
    let saved = game.pets_document();
    let refused = game.handle_in_world(&click(guid), &stranger, at).unwrap();
    assert!(!refused.changed, "a stranger's click changes nothing");
    assert!(said(&refused, TreasureRefusal::NotYours.message()));
    assert_eq!(game.state(), before);
    assert_eq!(game.pets_document(), saved);
    // ...so the owner still opens it.
    let opened = game.handle_in_world(&click(guid), &owner, at).unwrap();
    assert!(opened.changed && removals(&opened.replies, guid) == 1);
}

#[test]
fn a_far_garbled_or_late_click_refuses_or_is_ignored_and_changes_nothing() {
    let fixture = fixture();
    let (mut game, _) = session(&fixture, None);
    let owner = who();
    let (at, _, spawn) = dig_up(&mut game, &owner, START);
    let guid = spawn.actor_id;
    let before = game.state();
    let saved = game.pets_document();
    let far = CharacterState {
        position: [
            owner.position[0] + 100.0,
            owner.position[1],
            owner.position[2],
        ],
        ..who()
    };
    let refused = game.handle_in_world(&click(guid), &far, at).unwrap();
    assert!(!refused.changed && said(&refused, TreasureRefusal::TooFar.message()));
    // A click on a guid that is not a chest is no business of this handler; garbage is no panic.
    for packet in [
        click(guid ^ 0xff),
        vec![26, 0, 8, 0],
        vec![26, 0, 8, 0, 1, 2],
        click(0),
    ] {
        let _ = game.handle_in_world(&packet, &owner, at);
    }
    assert_eq!(game.state(), before);
    assert_eq!(game.pets_document(), saved);
    // Past its lifetime the chest is gone even before the tick removes it.
    let late = game
        .handle_in_world(&click(guid), &owner, at + rule("lifetime_ms") + 1)
        .unwrap();
    assert!(!late.changed && said(&late, TreasureRefusal::Gone.message()));
}

/// 100 filler stacks: the bag capacity is 100 occupied stacks.
fn full_bag(fixture: &Fixture) -> GameplayState {
    let mut state = Gameplay::load(fixture.path(), None).unwrap().state();
    let filler = state.inventory.items[0].clone();
    let mut id = 50_000;
    while state.inventory.items.len() < 100 {
        state.inventory.items.push(InventoryItem {
            id,
            ..filler.clone()
        });
        id += 1;
    }
    state
}

#[test]
fn a_full_bag_refuses_the_chest_and_keeps_it() {
    let fixture = fixture();
    // A bag of 100 stacks that holds none of the reward items: the item needs a new stack.
    let mut state = full_bag(&fixture);
    state
        .inventory
        .items
        .retain(|item| !reward_items().contains(&i64::from(item.definition_id)));
    let filler = state.inventory.items[0].clone();
    let mut id = 60_000;
    while state.inventory.items.len() < 100 {
        state.inventory.items.push(InventoryItem {
            id,
            ..filler.clone()
        });
        id += 1;
    }
    let (mut game, pet) = session(&fixture, Some(state));
    let owner = who();
    let (at, _, spawn) = dig_up(&mut game, &owner, START);
    let guid = spawn.actor_id;
    let before = game.state();
    let saved = game.pets_document();
    assert_eq!(before.inventory.items.len(), 100);
    for step in 1..=3 {
        let refused = game
            .handle_in_world(&click(guid), &owner, at + step * 1_000)
            .unwrap();
        assert!(!refused.changed, "a refused open changes nothing");
        assert_eq!(
            refused.replies,
            vec![chat::system(
                "Your bag is full. Make room before opening the treasure."
            )]
        );
        assert_eq!(game.state(), before);
        assert_eq!(game.pets_document(), saved);
    }
    // The chest is still there, so the tick does not remove it before its lifetime...
    let out = game.pet_tick(&owner, at + 5_000).unwrap();
    assert_eq!(removals(&out.replies, guid), 0);
    // ...and then it expires like any chest; the pet keeps its cooldown.
    let out = game
        .pet_tick(&owner, at + rule("lifetime_ms") + TICK)
        .unwrap();
    assert_eq!(removals(&out.replies, guid), 1);
    assert!(found_at(&game.pets_document().unwrap(), pet).is_some());
}

#[test]
fn a_full_bag_still_takes_a_reward_that_stacks_onto_what_it_holds() {
    let fixture = fixture();
    // 100 stacks, among them one of every reward item: no reward needs a new slot.
    let mut state = full_bag(&fixture);
    let template = state.inventory.items[0].clone();
    state.inventory.items.truncate(100 - reward_items().len());
    for (offset, definition) in reward_items().iter().enumerate() {
        state.inventory.items.push(InventoryItem {
            id: 70_000 + offset as i32,
            definition_id: *definition as i32,
            count: 1,
            tint: 0,
            rental: None,
            ..template.clone()
        });
    }
    assert_eq!(state.inventory.items.len(), 100);
    let (mut game, _) = session(&fixture, Some(state));
    let owner = who();
    let (at, _, spawn) = dig_up(&mut game, &owner, START);
    let before = game.state();
    let opened = game
        .handle_in_world(&click(spawn.actor_id), &owner, at + 1_000)
        .unwrap();
    assert!(opened.changed, "{:?}", opened.replies);
    assert_eq!(game.state().inventory.items.len(), 100);
    assert_ne!(held(&game.state()), held(&before));
}

#[test]
fn a_zone_change_takes_the_chest_and_keeps_the_cooldown() {
    let fixture = fixture();
    let (mut game, pet) = session(&fixture, None);
    let owner = who();
    let (at, _, spawn) = dig_up(&mut game, &owner, START);
    let guid = spawn.actor_id;
    let stamp = found_at(&game.pets_document().unwrap(), pet);
    assert!(stamp.is_some());
    // Zoning with the pet out: its actor and the chest go, and only to the owner (the chest).
    let ended = game.settle_pets(&owner, true, at + 1_000).unwrap();
    assert_eq!(removals(&ended.replies, guid), 1);
    assert_eq!(removals(&ended.nearby, guid), 0);
    assert_eq!(
        found_at(&game.pets_document().unwrap(), pet),
        stamp,
        "the cooldown is untouched"
    );
    let gone = game
        .handle_in_world(&click(guid), &owner, at + 1_500)
        .unwrap();
    assert!(
        !gone.changed
            && gone
                .replies
                .iter()
                .all(|packet| removals(std::slice::from_ref(packet), guid) == 0)
    );
    // A chest that outlives its pet is removed by the zone change too (pet dismissed first).
    let (mut second, _) = session(&fixture, None);
    let (at, _, spawn) = dig_up(&mut second, &owner, START);
    second.handle_in_world(&DISMISS, &owner, at + 100).unwrap();
    let ended = second.settle_pets(&owner, true, at + 200).unwrap();
    assert_eq!(removals(&ended.replies, spawn.actor_id), 1);
    assert!(!ended.changed);
    // Not zoning leaves everything as it is.
    let (mut third, _) = session(&fixture, None);
    let (at, _, _) = dig_up(&mut third, &owner, START);
    assert!(
        third
            .settle_pets(&owner, false, at + 100)
            .unwrap()
            .replies
            .is_empty()
    );
}

#[test]
fn a_reconnect_drops_the_chest_and_keeps_the_cooldown() {
    let fixture = fixture();
    let (mut game, pet) = session(&fixture, None);
    let owner = who();
    let (at, _, spawn) = dig_up(&mut game, &owner, START);
    let stamp = found_at(&game.pets_document().unwrap(), pet);
    // The connection ends: the session state goes, the roster (with its cooldown) stays.
    game.reset_session();
    assert_eq!(found_at(&game.pets_document().unwrap(), pet), stamp);
    assert!(
        game.pet_tick(&owner, at + TICK).is_none(),
        "no pet and no chest left to tick"
    );
    let before = game.state();
    let gone = game
        .handle_in_world(&click(spawn.actor_id), &owner, at + 1_000)
        .unwrap();
    assert!(!gone.changed);
    assert_eq!(removals(&gone.replies, spawn.actor_id), 0);
    assert_eq!(
        game.state(),
        before,
        "a click on the old chest pays nothing"
    );
}

/// The production route: `Gateway::tick_at` runs the hunt, the owner's reply carries the chest and
/// a save request, the viewers' `nearby` never does, and a 26/8 through `handle_at` pays once.
#[test]
fn the_gateway_ticks_the_hunt_and_pays_the_click_once() {
    let fixture = fixture();
    let (service, pet) = dog();
    let reference = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-gateway-reference");
    let state = Gameplay::load(fixture.path(), None).unwrap().state();
    let mut gateway =
        Gateway::load_gameplay(&reference, who(), fixture.path(), Some(state)).unwrap();
    gateway.adopt_pets(service.document_value()).unwrap();
    gateway.handle_at(&login(CHARACTER), 0).unwrap();
    for packet in [wrap(&[13, 0]), wrap(&[10, 0])] {
        gateway.handle_at(&packet, START).unwrap();
    }
    gateway.handle_at(&wrap(&summon(pet)), START).unwrap();
    let (mut at, mut found) = (START, None);
    while found.is_none() && at < START + 120_000 {
        at += TICK;
        let out = gateway.tick_at(at).unwrap();
        let inner: Vec<Vec<u8>> = out.replies.iter().map(|r| body(r).to_vec()).collect();
        assert!(
            !out.nearby
                .iter()
                .any(|packet| packet.starts_with(&[35, 0, 2, 0])),
            "a viewer must never get the chest"
        );
        if let Some(spawn) = chests(&inner).into_iter().next() {
            assert!(
                out.state_changed && out.gameplay_changed,
                "the find is saved"
            );
            found = Some(spawn);
        } else {
            assert!(!out.state_changed, "a quiet tick saves nothing");
        }
    }
    let spawn = found.expect("the gateway tick dug up a chest");
    let guid = spawn.actor_id;
    let tier = tier_of(&spawn);
    assert!(found_at(&gateway.snapshot().pets.unwrap(), pet).is_some());
    let coins = gateway.snapshot().gameplay.unwrap().inventory.coins;
    let opened = gateway.handle_at(&wrap(&click(guid)), at + 1_000).unwrap();
    assert!(opened.state_changed && opened.gameplay_changed);
    let inner: Vec<Vec<u8>> = opened.replies.iter().map(|r| body(r).to_vec()).collect();
    assert_eq!(removals(&inner, guid), 1, "the chest is removed");
    assert_eq!(removals(&opened.nearby, guid), 0, "for the owner alone");
    let paid = gateway.snapshot().gameplay.unwrap().inventory.coins - coins;
    assert_eq!(i64::from(paid), tier["coins"].as_i64().unwrap());
    let again = gateway.handle_at(&wrap(&click(guid)), at + 2_000).unwrap();
    assert!(!again.state_changed, "a second click pays nothing");
    assert_eq!(
        gateway.snapshot().gameplay.unwrap().inventory.coins - coins,
        paid
    );
}

#[test]
fn chest_guids_are_unique_across_finds_owners_and_sessions() {
    let mut seen = BTreeSet::new();
    for seed in 0..40 {
        let mut dog = Dig::dog(seed);
        let guid = dog.until_chest(120_000).unwrap().chest().unwrap().actor_id;
        assert!(seen.insert(guid), "guid {guid:#x} was minted twice");
        assert_ne!(guid, PetService::pet_guid(OWNER));
    }
    // Two finds of one session (the cooldown moved on between them) never share a guid either.
    let mut dog = Dig::dog(77);
    let first = dog.until_chest(120_000).unwrap().chest().unwrap().actor_id;
    dog.run(rule("lifetime_ms") + 2_000);
    dog.wall = WALL + rule("cooldown_ms") / 1_000;
    dog.owner = [0.0, 0.0, 6.0]; // a pet that lay down stands again when its owner moves
    let second = dog.until_chest(120_000).unwrap().chest().unwrap().actor_id;
    assert!(seen.insert(first) && seen.insert(second));
}
