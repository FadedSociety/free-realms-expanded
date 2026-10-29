//! T556: the placed-cake table and the pure live-cake registry. No gateway, no items.
//! Packets are read back with the production AddNpc reader (`0x00921a10`), the 35/16 field offsets
//! of the client reader (`0x008e5f50`) and the shared 35/3 reader (`pickup_packets::removed`);
//! the content is the shipped `placed_cakes.json`.

#[path = "support/node_content.rs"]
mod node_content;
#[allow(dead_code)] // the shared helpers; this suite counts packets only
#[path = "support/packets.rs"]
mod packets;
#[allow(dead_code)] // only the 35/3 reader is used
#[path = "support/pickup_packets.rs"]
mod pickup_packets;
#[path = "support/scratch.rs"]
mod scratch;

use osfr_tools::{
    consumable_effects::{Kind, Spec},
    encounter::animation,
    mounts::MountService,
    pets::PetService,
    placed_cakes::{self, Catalogue, GUID_TAG, PlacedCakes, Refusal, Row},
    players::ZoneInstance,
    startup_world::parse_add_npc,
};
use std::{collections::BTreeSet, sync::Arc};

const BOSS: i32 = 7974;
const REGIONAL: [i32; 8] = [7966, 7967, 7968, 7969, 7970, 7971, 7972, 7973];
const HERE: ZoneInstance = ZoneInstance {
    zone_id: 1,
    instance_id: 1,
};
const ELSEWHERE: ZoneInstance = ZoneInstance {
    zone_id: 1,
    instance_id: 2,
};
const PLACER: u64 = 7;
const CLICKER: u64 = 8;
const FIRST: u64 = 1_000;

/// A 35/16 as (actor guid, composite effect id, position): reader `0x008e5f50`.
fn effect_fields(packet: &[u8]) -> (u64, i32, [f32; 3]) {
    assert_eq!(packet[..4], [35, 0, 16, 0]);
    let f = |at: usize| f32::from_le_bytes(packet[at..at + 4].try_into().unwrap());
    (
        u64::from_le_bytes(packet[4..12].try_into().unwrap()),
        i32::from_le_bytes(packet[20..24].try_into().unwrap()),
        [f(32), f(36), f(40)],
    )
}

fn catalogue() -> Catalogue {
    Catalogue::load(&node_content::content_dir()).expect("the shipped cakes load")
}

fn row(item: i32) -> Arc<Row> {
    Arc::clone(catalogue().get(item).expect("cake row"))
}

fn registry() -> PlacedCakes {
    PlacedCakes::new(catalogue().max_live())
}

/// A boss cake placed at the origin facing +z, so it stands 1.5 m out at z = 1.5.
fn boss(cakes: &mut PlacedCakes) -> u64 {
    cakes
        .place(&row(BOSS), PLACER, HERE, [0.0; 3], 0.0, FIRST)
        .expect("place")
        .guid
}

fn near(cakes: &mut PlacedCakes, guid: u64, who: u64, now: u64) -> Result<Spec, Refusal> {
    cakes.click(guid, who, HERE, [0.0, 0.0, 1.0], now)
}

#[test]
fn shipped_rows_are_the_nine_cakes_with_sources_and_bounds() {
    let catalogue = catalogue();
    assert_eq!(catalogue.max_live(), 64);
    let boss = catalogue.get(BOSS).expect("boss row");
    assert_eq!((boss.model_id, boss.name_id), (2090, 30109));
    assert_eq!(boss.spawn_effect_ids, [16372, 16441]);
    assert_eq!(boss.one_shot.expect("boss one-shot").animation_id, 2001);
    assert_eq!(
        (
            boss.lifetime_ms,
            boss.item_cooldown_ms,
            boss.click_window_ms
        ),
        (60_000, 60_000, 5_000)
    );
    let models = |row: &Row| -> Vec<i32> {
        row.outcomes
            .iter()
            .filter_map(|o| match o.effect {
                placed_cakes::Effect::Transform { model_id, .. } => Some(model_id),
                placed_cakes::Effect::Aura { .. } => None,
            })
            .collect()
    };
    assert_eq!(models(boss), [2055, 1944, 1702]);
    let regional = [
        (7966, 2082, 9392),
        (7967, 2083, 9393),
        (7968, 2084, 9394),
        (7969, 2085, 9395),
        (7970, 2086, 9396),
        (7971, 2087, 9397),
        (7972, 2088, 9398),
        (7973, 2089, 9399),
    ];
    for (item, model, name) in regional {
        let row = catalogue.get(item).expect("regional row");
        assert_eq!((row.model_id, row.name_id), (model, name));
        assert_eq!(row.spawn_effect_ids, [21]);
        assert!(row.one_shot.is_none());
    }
    assert_eq!(models(catalogue.get(7967).unwrap()), [750, 749]);
    for item in REGIONAL.into_iter().chain([BOSS]) {
        let row = catalogue.get(item).expect("row");
        assert!(!row.source.is_empty(), "{item} names its source");
        assert!((1..=600_000).contains(&row.lifetime_ms));
        assert!(row.click_window_ms <= 60_000);
        assert!((1..=8).contains(&row.outcomes.len()));
        assert!(row.outcomes.iter().all(|o| o.duration_ms == 1_800_000));
    }
}

#[test]
fn a_bad_row_is_refused_at_load() {
    let bad = scratch::Scratch::new("placed-cakes");
    let mut file =
        std::fs::read_to_string(node_content::content_dir().join("placed_cakes.json")).unwrap();
    file = file.replacen("\"lifetime_ms\": 60000", "\"lifetime_ms\": 600001", 1);
    std::fs::write(bad.path().join("placed_cakes.json"), file).unwrap();
    assert!(Catalogue::load(bad.path()).is_err());
    // A missing file is an empty catalogue, not an error.
    let empty = scratch::Scratch::new("placed-cakes");
    assert!(Catalogue::load(empty.path()).unwrap().get(BOSS).is_none());
}

#[test]
fn placing_sends_add_npc_cursor_and_spawn_effects_in_order() {
    let mut cakes = registry();
    let placed = cakes
        .place(&row(BOSS), PLACER, HERE, [10.0, 2.0, 20.0], 0.0, FIRST)
        .unwrap();
    let npc = parse_add_npc(&placed.spawn[0]).expect("AddNpc decodes");
    assert_eq!(npc.actor_id, placed.guid);
    assert_eq!((npc.model_id, npc.name_string_id), (2090, 30109));
    assert_eq!((npc.is_interactable, npc.animation_id), (1, 1));
    // Heading 0 faces +z: 1.5 m out from the placer.
    assert_eq!(npc.position[..3], [10.0, 2.0, 21.5]);
    // 35/12, one entry: guid, has-cursor 1, cursor 5 (`cursor_talk`), flag 0.
    let mut relevance = vec![35, 0, 12, 0, 1, 0, 0, 0];
    relevance.extend(placed.guid.to_le_bytes());
    relevance.extend([1, 5, 0]);
    assert_eq!(placed.spawn[1], relevance);
    let at = [10.0, 2.0, 21.5];
    let effects: Vec<_> = placed.spawn[2..].iter().map(|p| effect_fields(p)).collect();
    assert_eq!(
        effects,
        [(placed.guid, 16372, at), (placed.guid, 16441, at)]
    );
    assert_eq!(packets::count(&placed.spawn, 35, 16), 2);
    // A regional cake is one spawn effect.
    let regional = cakes
        .place(&row(7966), PLACER, HERE, [0.0; 3], 0.0, FIRST)
        .unwrap();
    assert_eq!(packets::count(&regional.spawn, 35, 16), 1);
    assert_eq!(cakes.len(), 2);
}

#[test]
fn a_second_click_inside_the_window_is_refused_and_changes_nothing() {
    let mut cakes = registry();
    let guid = boss(&mut cakes);
    let first = near(&mut cakes, guid, CLICKER, FIRST + 1_000).expect("first click wins");
    assert!(matches!(first.kind, Kind::Transform { .. }));
    let after_first = cakes.clone();
    // Another player in the same window: refused, registry equal.
    assert_eq!(
        near(&mut cakes, guid, 99, FIRST + 2_000),
        Err(Refusal::Window)
    );
    assert_eq!(
        near(&mut cakes, guid, CLICKER, FIRST + 5_999),
        Err(Refusal::Window)
    );
    assert_eq!(cakes, after_first);
    // The window opens again 5 s after the accepted click.
    assert!(near(&mut cakes, guid, 99, FIRST + 6_000).is_ok());
}

#[test]
fn out_of_range_other_instance_and_unknown_clicks_are_refused_and_change_nothing() {
    let mut cakes = registry();
    let guid = boss(&mut cakes);
    let before = cakes.clone();
    let at = FIRST + 1_000;
    // The cake stands at z = 1.5; range is 5 m.
    assert_eq!(
        cakes.click(guid, CLICKER, HERE, [0.0, 0.0, 6.6], at),
        Err(Refusal::Range)
    );
    assert_eq!(
        cakes.click(guid, CLICKER, HERE, [f32::NAN, 0.0, 0.0], at),
        Err(Refusal::Range)
    );
    assert_eq!(
        cakes.click(guid, CLICKER, ELSEWHERE, [0.0, 0.0, 1.0], at),
        Err(Refusal::Instance)
    );
    assert_eq!(
        cakes.click(guid + 1, CLICKER, HERE, [0.0, 0.0, 1.0], at),
        Err(Refusal::Unknown)
    );
    assert_eq!(cakes, before);
    assert!(
        cakes
            .click(guid, CLICKER, HERE, [0.0, 0.0, 6.4], at)
            .is_ok()
    );
}

#[test]
fn a_refused_place_changes_nothing_and_the_cap_holds() {
    let mut cakes = PlacedCakes::new(2);
    let row = row(BOSS);
    for _ in 0..2 {
        cakes
            .place(&row, PLACER, HERE, [0.0; 3], 0.0, FIRST)
            .unwrap();
    }
    let before = cakes.clone();
    assert_eq!(
        cakes.place(&row, PLACER, HERE, [0.0; 3], 0.0, FIRST).err(),
        Some(Refusal::Full)
    );
    assert_eq!(cakes, before);
    let mut open = PlacedCakes::new(2);
    let empty = open.clone();
    assert_eq!(
        open.place(&row, PLACER, HERE, [f32::NAN, 0.0, 0.0], 0.0, FIRST)
            .err(),
        Some(Refusal::Position)
    );
    assert_eq!(
        open.place(&row, PLACER, HERE, [0.0; 3], f32::INFINITY, FIRST)
            .err(),
        Some(Refusal::Position)
    );
    assert_eq!(open, empty);
}

#[test]
fn a_thousand_clicks_never_repeat_an_outcome_and_reach_every_one() {
    let mut cakes = registry();
    // The shipped 5 s window and 60 s life would stop the run: a long-lived, windowless copy.
    let mut long = (*row(BOSS)).clone();
    long.lifetime_ms = u64::MAX / 2;
    long.click_window_ms = 0;
    let guid = cakes
        .place(&Arc::new(long), PLACER, HERE, [0.0; 3], 0.0, FIRST)
        .unwrap()
        .guid;
    let mut seen = BTreeSet::new();
    let mut last = None;
    for n in 0..1_000_u64 {
        let spec = near(&mut cakes, guid, CLICKER, FIRST + n * 37).unwrap();
        let Kind::Transform { model_id, .. } = spec.kind else {
            panic!("the boss cake transforms")
        };
        assert_ne!(Some(model_id), last, "click {n} repeated the last outcome");
        last = Some(model_id);
        seen.insert(model_id);
    }
    assert_eq!(seen, BTreeSet::from([2055, 1944, 1702]));
}

#[test]
fn a_single_outcome_cake_repeats_and_roll_excluding_stays_uniform() {
    let mut cakes = registry();
    let mut long = (*row(7968)).clone();
    long.lifetime_ms = u64::MAX / 2;
    long.click_window_ms = 0;
    let guid = cakes
        .place(&Arc::new(long), PLACER, HERE, [0.0; 3], 0.0, FIRST)
        .unwrap()
        .guid;
    for n in 0..3 {
        assert!(near(&mut cakes, guid, CLICKER, FIRST + n).is_ok());
    }
    let mut hits = [0_u32; 3];
    for seed in 0..3_000 {
        hits[placed_cakes::roll_excluding(3, Some(1), seed)] += 1;
    }
    assert_eq!(hits[1], 0);
    assert!(hits[0] > 1_300 && hits[2] > 1_300, "{hits:?}");
}

#[test]
fn the_cake_despawns_exactly_once_at_sixty_seconds() {
    let mut cakes = registry();
    let guid = boss(&mut cakes);
    // The one-shot animation may fire on the way; the removal is the only 35/3.
    let mut removals = Vec::new();
    for now in (FIRST..FIRST + 59_000).step_by(500) {
        for (_, _, packets) in cakes.tick(now) {
            assert_eq!(packets::count(&packets, 35, 3), 0, "removed early at {now}");
        }
    }
    assert_eq!(cakes.len(), 1);
    // Due time, but not yet ticked: a click is refused as expired and changes nothing.
    let before = cakes.clone();
    assert_eq!(
        near(&mut cakes, guid, CLICKER, FIRST + 60_000),
        Err(Refusal::Expired)
    );
    assert_eq!(cakes, before);
    for now in (FIRST + 59_000..FIRST + 70_000).step_by(500) {
        removals.extend(
            cakes
                .tick(now)
                .into_iter()
                .flat_map(|(instance, _, packets)| packets.into_iter().map(move |p| (instance, p))),
        );
    }
    let removed: Vec<_> = removals
        .iter()
        .filter(|(_, p)| packets::count(std::slice::from_ref(p), 35, 3) == 1)
        .collect();
    assert_eq!(removed.len(), 1, "one removal in total");
    assert_eq!(removed[0].0, HERE);
    assert!(pickup_packets::removed(
        std::slice::from_ref(&removed[0].1),
        guid
    ));
    assert!(cakes.is_empty());
    assert_eq!(
        near(&mut cakes, guid, CLICKER, FIRST + 70_000),
        Err(Refusal::Unknown)
    );
    assert!(cakes.tick(FIRST + 80_000).is_empty());
}

#[test]
fn the_boss_cake_plays_its_one_shot_every_ten_to_twenty_seconds() {
    let mut cakes = registry();
    let guid = boss(&mut cakes);
    let mut regional = registry();
    regional
        .place(&row(7966), PLACER, HERE, [0.0; 3], 0.0, FIRST)
        .unwrap();
    let (mut starts, mut returns) = (Vec::new(), Vec::new());
    for now in (FIRST..FIRST + 59_000).step_by(100) {
        assert!(
            regional.tick(now).is_empty(),
            "a regional cake has no one-shot"
        );
        for (instance, _, packets) in cakes.tick(now) {
            assert_eq!(instance, HERE);
            for packet in packets {
                if packet == animation(guid, 2001, true) {
                    starts.push(now);
                } else if packet == animation(guid, 1, true) {
                    returns.push(now);
                } else {
                    panic!("unexpected tick packet at {now}");
                }
            }
        }
    }
    assert!(starts.len() >= 2, "{starts:?}");
    assert_eq!(starts.len(), returns.len());
    // First shot 10-20 s after placing, each play lasts 3 s, the next one 10-20 s after that.
    assert!(
        (10_000..=20_000).contains(&(starts[0] - FIRST)),
        "{starts:?}"
    );
    for (start, back) in starts.iter().zip(&returns) {
        assert!(
            (3_000..=3_100).contains(&(back - start)),
            "{starts:?} {returns:?}"
        );
    }
    for (back, next) in returns.iter().zip(&starts[1..]) {
        assert!(
            (10_000..=20_100).contains(&(next - back)),
            "{starts:?} {returns:?}"
        );
    }
}

#[test]
fn an_empty_registry_ticks_without_allocating() {
    let mut cakes = registry();
    assert_eq!(cakes.tick(5).capacity(), 0);
}

#[test]
fn cake_guids_are_disjoint_from_players_content_pets_rides_and_world_actors() {
    let mut cakes = registry();
    let guids: Vec<u64> = (0..64).map(|_| boss(&mut cakes)).collect();
    assert_eq!(guids.iter().collect::<BTreeSet<_>>().len(), 64, "unique");
    assert!(guids.iter().all(|g| g >> 32 == GUID_TAG >> 32));
    // Characters count up from 1; content NPCs sit in 9e9..1e10 and at 1e11.
    let others = [
        1_u64,
        100_000,
        9_000_000_000,
        9_999_999_999,
        100_000_000_000,
        0x1000_0000_0000_0001,
        0x0f00_0000_0000_0001,
        0x4f50_454e_0000_f001,
        0x5255_5354_0000_0001,
        0x5057_5550_0000_0000,
        0x4e4f_4400_0000_0000,
        0x424f_4f4d_0000_0001,
    ];
    assert!(others.iter().all(|g| !guids.contains(g)));
    for owner in [1_u64, 2, 1_000_000, 0xffff_ffff] {
        for pet_id in [0_u32, 1, 2, 1_000, 65_535] {
            let pet = PetService::pet_guid(owner, pet_id);
            assert!(!guids.contains(&pet), "pet {pet:x}");
            assert_ne!(pet >> 32, GUID_TAG >> 32);
        }
        assert_ne!(MountService::ride_guid(owner) >> 32, GUID_TAG >> 32);
    }
}
