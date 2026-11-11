//! P2 combat-kits: the 2014 weapon-driven action bar, the 150 abilities and the 24 traits
//! (`data/content/weapon_kits.json`). Damage numbers are the wiki tooltips' ladder by weapon
//! Power tier; the bar shape and the six captured ability-id pairs are the March 2014 capture's.
use osfr_tools::encounter::{
    FightFact,
    kits::{Bar, Body, Caster, Kind, Kit, KitTable, Shape, StatusTable, Targets, WeaponFacts, bar_for, resolve},
};
use serde_json::Value;
use std::sync::Arc;

#[path = "common/kit_fixture.rs"]
mod kit_fixture;
use kit_fixture::{Fixture, dealt, energy, refusal, shipped};

/// Profile ids of the six combat jobs.
const JOBS: [u32; 6] = [2, 11, 12, 32, 35, 43];
/// The wiki's damage by weapon Power tier (evidence-abilities 4.1).
const LADDERS: [(&str, [u32; 5]); 5] = [
    ("basic_rider", [254, 444, 776, 1357, 2372]),
    ("basic_plain", [279, 488, 853, 1492, 2609]),
    ("super_utility", [640, 1118, 1955, 3420, 5977]),
    ("super_standard", [889, 1554, 2716, 4750, 8302]),
    ("super_heavy", [1143, 1998, 3492, 6107, 10674]),
];

fn table() -> &'static Arc<KitTable> {
    KitTable::builtin().expect("the shipped weapon_kits.json parses")
}

fn ladder(class: &str) -> [u32; 5] {
    LADDERS.iter().find(|(name, _)| *name == class).expect("a known class").1
}

fn caster(profile: u32, level: u8, tier: u8, now_ms: u64) -> Caster {
    Caster {
        guid: 1,
        position: [0.0; 3],
        heading: [0.0; 3],
        profile,
        level,
        tier,
        energy: 100,
        now_ms,
    }
}

fn body(guid: u64, x: f32, z: f32) -> Body {
    Body {
        guid,
        kind: if guid < 50 { 0 } else { 400 },
        position: [x, 0.0, z],
        health: 1000,
        max_health: 1000,
    }
}

fn kit(id: &str) -> &'static Kit {
    table().kit(id).unwrap_or_else(|| panic!("kit {id} exists"))
}

fn default_weapon() -> WeaponFacts {
    WeaponFacts::default()
}

/// An item the table lists for `kit_id`, with the Power it lists it at.
fn item_of(kit_id: &str) -> (i32, u8) {
    let content: Value = serde_json::from_slice(&std::fs::read(shipped("weapon_kits.json")).unwrap()).unwrap();
    let items = content["kits"][kit_id]["items"].as_object().expect("the kit has items");
    let (power, ids) = items.iter().next().expect("at least one item");
    (ids[0].as_i64().unwrap() as i32, power.parse().unwrap())
}

#[test]
fn the_roster_is_150_abilities_and_24_traits() {
    let t = table();
    assert_eq!(t.abilities().count(), 150);
    for job in JOBS {
        let of = |kind: Kind| t.abilities().filter(|a| a.job == job && a.kind == kind).count();
        assert_eq!((of(Kind::Basic), of(Kind::Super), of(Kind::Third)), (12, 12, 1), "job {job}");
        assert_eq!(t.traits(job, 20).count(), 4, "job {job} has four traits");
        for (level, count) in [(4, 0), (5, 1), (10, 2), (15, 3), (20, 4)] {
            assert_eq!(t.traits(job, level).count(), count, "job {job} traits at level {level}");
        }
        assert!(t.kits().filter(|k| k.job == job).count() >= 12, "job {job} has its twelve weapon families");
    }
    for (class, steps) in LADDERS {
        assert_eq!(t.class(class), Some(steps), "class {class} is the wiki ladder");
    }
}

/// Table sweep: every ability at every tier resolves without a refusal and each hit is the
/// ladder value of its class (or the tooltip's own Power 5 number scaled along that ladder).
#[test]
fn every_ability_resolves_at_every_tier_on_the_ladder() {
    let enemies: Vec<Body> = (0..6).map(|i| body(100 + i, (i as f32 - 2.5) * 0.6, 3.0)).collect();
    let allies = [body(1, 0.0, 0.0), body(2, 1.0, 0.0)];
    let mut swept = 0;
    for k in table().kits() {
        for slot in 0..8u8 {
            let Some(ability) = k.slot(slot) else { continue };
            let steps = ladder(&ability.class);
            for tier in 1..=5u8 {
                let targets = Targets { selected: None, enemies: &enemies, allies: &allies };
                let strike = resolve(k, slot, &caster(k.job, 20, tier, 1000 + u64::from(tier)), &targets)
                    .unwrap_or_else(|id| panic!("{} tier {tier} refused with {id}", ability.key));
                let i = usize::from(tier) - 1;
                let per_hit = ability.damage[i];
                let expected = if ability.damage[4] == steps[4] {
                    steps[i]
                } else {
                    (f64::from(ability.damage[4]) * f64::from(steps[i]) / f64::from(steps[4])).round() as u32
                };
                assert_eq!(per_hit, expected, "{} tier {tier}", ability.key);
                if per_hit > 0 {
                    let first = strike.hits[0].0;
                    let plain = strike.hits.iter().filter(|h| h.0 == first && h.1 == per_hit).count();
                    assert!(plain >= usize::from(ability.hits), "{} lands its {} hits", ability.key, ability.hits);
                } else {
                    assert!(
                        !strike.heals.is_empty() || !strike.statuses.is_empty() || strike.summon.is_some() || !strike.dots.is_empty(),
                        "{} does something without damage",
                        ability.key
                    );
                }
                assert_eq!(strike.energy_cost, ability.cost);
                swept += 1;
            }
        }
    }
    assert!(swept >= 150 * 5, "swept {swept} casts");
}

#[test]
fn tooltip_numbers_are_the_damage() {
    let at = |key: &str| {
        let a = table().ability(key).unwrap_or_else(|| panic!("{key}"));
        (a.damage, a.hits)
    };
    assert_eq!(at("flame_flash"), ([279, 488, 853, 1492, 2609], 1));
    assert_eq!(at("glancing_blow"), ([254, 444, 776, 1357, 2372], 1));
    assert_eq!(at("dragonstrike"), ([1143, 1998, 3492, 6107, 10674], 1));
    assert_eq!(at("leg_sweep").0[1], 1554, "the Power 2 super read off a video frame");
    assert_eq!((at("volley").0[4], at("volley").1), (4732, 3));
    assert_eq!((at("pummel").0[4], at("pummel").1), (3522, 4));
    assert_eq!((at("frenzy").0[4], at("frenzy").1), (2348, 5));
    assert_eq!((at("cauterize").0[4], at("cauterize").1), (1186, 2));
    assert_eq!(at("twisted_edge").0[4], 2870);
    assert_eq!(at("sniper_shot").0[4], 11741);
    assert_eq!(at("leg_sweep_awakened").0[4], 9132);
    let triage = table().ability("triage").unwrap();
    assert_eq!(triage.damage[0], 889, "Triage at Power 1");
    assert_eq!(triage.heal, Some((true, [254, 444, 776, 1357, 2372])), "and its group heal");
    let mend = table().ability("mend").unwrap();
    assert_eq!((mend.cost, mend.heal.map(|h| h.1[4])), (50, Some(2609)), "Mend costs 50 energy");
}

fn check_bar(bar: &Bar, basic_range: f32) {
    let basic = bar.slots[0].as_ref().expect("slot 0 is the basic");
    let sup = bar.slots[1].as_ref().expect("slot 1 is the super");
    assert_eq!((basic.energy_cost, sup.energy_cost), (0, 100));
    assert_eq!((basic.range, sup.range), (basic_range, 15.0));
    assert!(bar.slots[2..].iter().all(Option::is_none), "two slots filled, as captured");
}

#[test]
fn each_job_shows_the_captured_bar() {
    for (job, range) in [(2, 5.0), (11, 5.0), (12, 10.0), (32, 5.0), (35, 15.0), (43, 5.0)] {
        for level in [1, 4, 8, 12, 16, 19] {
            let bar = table().bar(job, level, &default_weapon());
            check_bar(&bar, range);
            assert_eq!(bar, bar_for(job, level, &default_weapon()), "the free function is the shipped table");
        }
    }
    let ids = |bar: Bar| (bar.slots[0].clone().unwrap(), bar.slots[1].clone().unwrap());
    // p12 36/5 t=546232.0 (Ninja) and t=546359.0 (Archer), a level-20 character.
    let (basic, sup) = ids(table().bar(2, 20, &default_weapon()));
    assert_eq!((basic.ability_id, basic.name_id), (4895, 419883));
    assert_eq!((sup.ability_id, sup.name_id), (4899, 419930));
    let (basic, sup) = ids(table().bar(35, 20, &default_weapon()));
    assert_eq!((basic.ability_id, basic.name_id), (5060, 420256));
    assert_eq!((sup.ability_id, sup.name_id, sup.icon_id), (5061, 420258, 22802));
    // The tier-4 definitions in the same capture carry 4894 and 4898.
    let (basic, sup) = ids(table().bar(2, 12, &default_weapon()));
    assert_eq!((basic.ability_id, sup.ability_id), (4894, 4898));
    // Medic and Warrior were captured with Power 5 weapons of other families (no item of
    // those two families is in the item table, so read the kits themselves).
    let medic = kit("medic/cauterize+laser_surgery");
    let (basic, sup) = (medic.slot(0).unwrap(), medic.slot(1).unwrap());
    assert_eq!((basic.wire_id(5), sup.wire_id(5), sup.icon_id), (4646, 4645, 22951));
    let warrior = kit("warrior/hack_n_slash+berserk");
    let (basic, sup) = (warrior.slot(0).unwrap(), warrior.slot(1).unwrap());
    assert_eq!((basic.wire_id(5), basic.name_id, sup.wire_id(5), sup.name_id), (4974, 35612, 4975, 7933));
}

#[test]
fn another_weapon_changes_both_abilities() {
    let content: Value = serde_json::from_slice(&std::fs::read(shipped("weapon_kits.json")).unwrap()).unwrap();
    for job in JOBS {
        // The first other weapon family of the job that the item table knows.
        let default = table().kit_for(job, &default_weapon()).unwrap().id.clone();
        let other = content["kits"]
            .as_object()
            .unwrap()
            .iter()
            .find(|(id, row)| row["job"] == job && **id != default && !row["items"].as_object().unwrap().is_empty())
            .map(|(id, _)| id.as_str())
            .unwrap_or_else(|| panic!("job {job} has a second weapon family with items"));
        let starter = table().bar(job, 1, &default_weapon());
        let (item, _) = item_of(other);
        let weapon = WeaponFacts { item_id: item, ..Default::default() };
        let swapped = table().bar(job, 1, &weapon);
        check_bar(&swapped, starter.slots[0].as_ref().unwrap().range);
        for slot in 0..2 {
            let (a, b) = (starter.slots[slot].as_ref().unwrap(), swapped.slots[slot].as_ref().unwrap());
            assert_ne!(a.ability_id, b.ability_id, "job {job} slot {slot}");
            assert_ne!(a.name_id, b.name_id, "job {job} slot {slot}");
        }
        assert_eq!(table().kit_for(job, &weapon).unwrap().id, other);
        // A weapon of another job never gives its kit.
        let wrong = JOBS.iter().copied().find(|j| *j != job).unwrap();
        assert_ne!(table().kit_for(wrong, &weapon).unwrap().id, other);
    }
}

#[test]
fn the_suffix_rule_and_the_default_follow_item_ids() {
    let mut content: Value = serde_json::from_slice(&std::fs::read(shipped("weapon_kits.json")).unwrap()).unwrap();
    content["weapons"]["by_suffix"] = serde_json::json!({"2": {"of Storms": "ninja/lightning_strike+1000_storms"}});
    let t = KitTable::parse(&serde_json::to_vec(&content).unwrap(), StatusTable::builtin()).unwrap();
    let named = WeaponFacts { item_id: 999_999, name: "Katana of Storms".into(), power: 3 };
    assert_eq!(t.kit_for(2, &named).unwrap().id, "ninja/lightning_strike+1000_storms");
    assert_eq!(t.tier(2, 20, &named), 3, "its own Power");
    let unknown = WeaponFacts { item_id: 999_999, name: "Katana".into(), power: 0 };
    assert_eq!(t.kit_for(2, &unknown).unwrap().id, "ninja/flame_flash+dragonstrike");
    assert!(t.kit_for(1, &unknown).is_none(), "a job without a kit has none");
}

#[test]
fn the_tier_is_the_weapons_power_capped_by_the_level() {
    let t = table();
    let (strong, power) = item_of("ninja/hidden_strike+fan_of_blades");
    assert_eq!(power, 5);
    let strong = WeaponFacts { item_id: strong, ..Default::default() };
    for (level, tier) in [(1, 1), (3, 1), (4, 2), (8, 3), (12, 4), (16, 5), (20, 5)] {
        assert_eq!(t.tier(2, level, &strong), tier, "a Power 5 weapon at level {level}");
        assert_eq!(t.tier(2, level, &default_weapon()), tier, "the starter weapon follows the level");
    }
    let weak = WeaponFacts { item_id: 1, name: String::new(), power: 1 };
    assert_eq!(t.tier(2, 20, &weak), 1, "a Power 1 weapon stays tier 1 at level 20");
}

#[test]
fn the_third_ability_needs_the_weapon_and_level_twenty() {
    // No Awakened weapon's item id is in the item table yet: give the kit one here.
    let mut content: Value = serde_json::from_slice(&std::fs::read(shipped("weapon_kits.json")).unwrap()).unwrap();
    content["kits"]["brawler/brawler_s_smash+leg_sweep_awakened"]["items"] = serde_json::json!({"5": [999_001]});
    let t = KitTable::parse(&serde_json::to_vec(&content).unwrap(), StatusTable::builtin()).unwrap();
    let weapon = WeaponFacts { item_id: 999_001, ..Default::default() };
    let third = usize::from(t.slots.third);
    assert_eq!(third, 3);
    assert!(t.bar(43, 19, &weapon).slots[third].is_none());
    let bar = t.bar(43, 20, &weapon);
    assert_eq!(bar.slots[third].as_ref().map(|s| s.name_id), Some(5101085), "Reckless");
    assert_eq!(bar.slots[1].as_ref().map(|s| s.name_id), Some(5101083), "the Awakened Leg Sweep");
    assert!(t.bar(43, 20, &default_weapon()).slots[third].is_none());
}

#[test]
fn a_cone_hits_what_is_in_front_up_to_its_cap() {
    let k = kit("ninja/flame_flash+dragonstrike");
    let Shape::Front { cone_deg, targets: cap } = k.slot(0).unwrap().shape else {
        panic!("Flame Flash is frontal");
    };
    assert_eq!((cone_deg, cap), (120.0, 5));
    let enemies = [body(100, 0.0, 3.0), body(101, 2.0, 3.0), body(102, 0.0, -3.0), body(103, 4.9, 0.5), body(104, 0.0, 6.0)];
    let targets = Targets { selected: Some(100), enemies: &enemies, allies: &[] };
    let strike = resolve(k, 0, &caster(2, 1, 1, 5), &targets).unwrap();
    let hit: Vec<u64> = strike.hits.iter().map(|h| h.0).collect();
    assert_eq!(hit, [100, 101], "behind, beside and out of reach are missed");
    assert_eq!(strike.primary, Some(100));
    let crowd: Vec<Body> = (0..9).map(|i| body(100 + i, (i as f32 - 4.0) * 0.3, 3.0)).collect();
    let targets = Targets { selected: Some(104), enemies: &crowd, allies: &[] };
    let strike = resolve(k, 0, &caster(2, 1, 1, 5), &targets).unwrap();
    assert_eq!(strike.hits.len(), 5, "target cap");
    assert_eq!(strike.hits[0].0, 104, "the selected enemy is hit first");
    // A heading given by the caller is used instead of the line to the target.
    let mut turned = caster(2, 1, 1, 5);
    turned.heading = [0.0, 0.0, -1.0];
    let targets = Targets { selected: None, enemies: &enemies, allies: &[] };
    let hit: Vec<u64> = resolve(k, 0, &turned, &targets).unwrap().hits.iter().map(|h| h.0).collect();
    assert_eq!(hit, [102]);
}

#[test]
fn a_radius_hits_around_the_caster() {
    let k = kit("warrior/sweeping_slash+spinning_attack");
    assert_eq!(k.slot(0).unwrap().shape, Shape::Circle { radius: 6.0, targets: 5 });
    let enemies = [body(100, 0.0, 5.9), body(101, -3.0, -3.0), body(102, 0.0, 6.2)];
    let targets = Targets { selected: None, enemies: &enemies, allies: &[] };
    let hit: Vec<u64> = resolve(k, 0, &caster(32, 1, 1, 5), &targets).unwrap().hits.iter().map(|h| h.0).collect();
    assert_eq!(hit, [101, 100], "nearest first, 6.2 m is outside");
    // Spinning Attack: three hits on each.
    let strike = resolve(k, 1, &caster(32, 1, 1, 5), &targets).unwrap();
    assert_eq!(strike.hits.len(), 6);
    let around = kit("brawler/glancing_blow+leg_sweep").slot(1).unwrap();
    assert_eq!(around.shape, Shape::Circle { radius: 8.0, targets: 8 });
}

#[test]
fn a_chain_jumps_enemy_to_enemy_without_repeats() {
    let k = kit("archer/cover_fire+ricochet");
    assert_eq!(k.slot(1).unwrap().shape, Shape::Chain { jumps: 3, jump_range: 8.0 });
    let line: Vec<Body> = (0..6).map(|i| body(100 + i, 0.0, 10.0 + 7.0 * i as f32)).collect();
    let targets = Targets { selected: Some(100), enemies: &line, allies: &[] };
    let hit: Vec<u64> = resolve(k, 1, &caster(35, 20, 5, 5), &targets).unwrap().hits.iter().map(|h| h.0).collect();
    assert_eq!(hit, [100, 101, 102, 103], "the target and three jumps");
    let gap = [body(100, 0.0, 10.0), body(101, 0.0, 19.0)];
    let targets = Targets { selected: Some(100), enemies: &gap, allies: &[] };
    assert_eq!(resolve(k, 1, &caster(35, 20, 5, 5), &targets).unwrap().hits.len(), 1, "9 m is too far to jump");
}

#[test]
fn an_area_lands_around_the_target() {
    let k = kit("archer/charged_shot+explosive_shot");
    assert_eq!(k.slot(1).unwrap().shape, Shape::AtTarget { radius: 5.0, targets: 5 });
    let enemies = [body(100, 0.0, 12.0), body(101, 3.0, 12.0), body(102, 0.0, 2.0), body(103, 0.0, 17.5)];
    let targets = Targets { selected: Some(100), enemies: &enemies, allies: &[] };
    let hit: Vec<u64> = resolve(k, 1, &caster(35, 4, 2, 5), &targets).unwrap().hits.iter().map(|h| h.0).collect();
    assert_eq!(hit, [100, 101], "the one beside the caster and the one 5.5 m behind the target are missed");
    // The single-target basic takes the nearest enemy when the client names none.
    let targets = Targets { selected: None, enemies: &enemies, allies: &[] };
    assert_eq!(resolve(k, 0, &caster(35, 4, 2, 5), &targets).unwrap().hits, [(102, 444)]);
}

#[test]
fn refusals_use_the_clients_strings() {
    let k = kit("ninja/flame_flash+dragonstrike");
    let enemies = [body(100, 0.0, 3.0)];
    let near = Targets { selected: None, enemies: &enemies, allies: &[] };
    let mut tired = caster(2, 1, 1, 5);
    tired.energy = 99;
    assert_eq!(resolve(k, 1, &tired, &near).unwrap_err(), 285, "not enough energy");
    assert!(resolve(k, 0, &tired, &near).is_ok(), "the basic is free");
    assert_eq!(resolve(k, 3, &caster(2, 20, 5, 5), &near).unwrap_err(), 286, "an empty slot");
    let nobody = Targets { selected: None, enemies: &[], allies: &[] };
    assert_eq!(resolve(k, 0, &caster(2, 1, 1, 5), &nobody).unwrap_err(), 288, "no target");
    let far = [body(100, 0.0, 3.0), body(101, 0.0, 40.0)];
    let chosen = Targets { selected: Some(101), enemies: &far, allies: &[] };
    assert_eq!(resolve(k, 0, &caster(2, 1, 1, 5), &chosen).unwrap_err(), 288, "a chosen enemy out of reach");
    let dead = [Body { health: 0, ..body(100, 0.0, 3.0) }];
    let dead = Targets { selected: Some(100), enemies: &dead, allies: &[] };
    assert_eq!(resolve(k, 0, &caster(2, 1, 1, 5), &dead).unwrap_err(), 288);
}

#[test]
fn riders_heals_and_buffs_land_where_the_tooltip_says() {
    let enemies = [body(100, 0.0, 3.0), body(101, 1.0, 3.0)];
    let allies = [body(1, 0.0, 0.0), body(2, 5.0, 0.0), body(3, 40.0, 0.0)];
    let targets = Targets { selected: None, enemies: &enemies, allies: &allies };
    // "Sometimes stuns": the reconstructed 15 %, rolled per target from the cast itself.
    let ninja = kit("ninja/flame_flash+dragonstrike");
    let stuns: usize = (0..2000)
        .map(|t| resolve(ninja, 0, &caster(2, 1, 1, t * 650), &targets).unwrap().statuses.len())
        .sum();
    assert!((440..=760).contains(&stuns), "about 15 % of 4000 rolls, got {stuns}");
    let again = |t| resolve(ninja, 0, &caster(2, 1, 1, t), &targets).unwrap();
    assert_eq!(again(777), again(777), "the same cast resolves the same way");
    // Leg Sweep always knocks down everyone it hits.
    let sweep = resolve(kit("brawler/glancing_blow+leg_sweep"), 1, &caster(43, 4, 2, 5), &targets).unwrap();
    assert_eq!(sweep.hits, [(100, 1554), (101, 1554)]);
    assert_eq!(sweep.statuses, [(100, "knock_down".into(), 2000), (101, "knock_down".into(), 2000)]);
    // Triage heals the caster and the group member in range, not the one 40 m away.
    let triage = resolve(kit("medic/incision+triage"), 1, &caster(11, 1, 1, 5), &targets).unwrap();
    assert_eq!(triage.heals, [(1, 254), (2, 254)]);
    assert_eq!(triage.group_heal, 254);
    assert_eq!(triage.hits.len(), 2, "and still strikes");
    // Ready to Rumble raises the caster's damage; Warcry the whole group's.
    let rumble = resolve(kit("brawler/wallop+ready_to_rumble"), 1, &caster(43, 1, 1, 5), &targets).unwrap();
    assert!(rumble.statuses.contains(&(1, "damage_up".into(), 10_000)));
    let warcry = resolve(kit("warrior/dual_strike+warcry"), 1, &caster(32, 4, 2, 5), &targets).unwrap();
    assert!(warcry.statuses.contains(&(1, "damage_up".into(), 10_000)) && warcry.statuses.contains(&(2, "damage_up".into(), 10_000)));
    assert!(!warcry.statuses.iter().any(|s| s.0 == 3));
    // A buff needs no enemy; a summon neither.
    let nobody = Targets { selected: None, enemies: &[], allies: &allies };
    let nurse = resolve(kit("medic/assist+nurse"), 1, &caster(11, 4, 2, 5), &nobody).unwrap();
    assert_eq!(nurse.summon, Some((2, 15_000, 444, 2000)));
    // Burning Slash burns for 6 s; Ninja's Purge hits burning targets again.
    let forged = kit("ninja/burning_slash+ninja_s_purge");
    let slash = resolve(forged, 0, &caster(2, 16, 5, 5), &targets).unwrap();
    assert_eq!((slash.hits[0].1, slash.dots[0].damage, slash.dots[0].every_ms, slash.dots[0].ms), (1304, 711, 3000, 6000));
    let purge = resolve(forged, 1, &caster(2, 16, 5, 5), &targets).unwrap();
    assert_eq!(purge.conditional, [(100, "burn".into(), 7471)]);
    // Perfect Shot adds 1186 against a lone target only.
    let perfect = kit("archer/perfect_shot+archer_s_advantage");
    assert_eq!(resolve(perfect, 0, &caster(35, 16, 5, 5), &targets).unwrap().hits, [(100, 2372)]);
    let lone = Targets { selected: None, enemies: &enemies[..1], allies: &allies };
    assert_eq!(resolve(perfect, 0, &caster(35, 16, 5, 5), &lone).unwrap().hits, [(100, 2372), (100, 1186)]);
}

#[test]
fn the_loader_refuses_broken_tables() {
    let good: Value = serde_json::from_slice(&std::fs::read(shipped("weapon_kits.json")).unwrap()).unwrap();
    let parse = |change: &dyn Fn(&mut Value)| {
        let mut content = good.clone();
        change(&mut content);
        KitTable::parse(&serde_json::to_vec(&content).unwrap(), StatusTable::builtin())
    };
    assert!(parse(&|_| {}).is_ok());
    assert!(parse(&|c| c["abilities"]["flame_flash"]["riders"][0]["status"] = "dizzy".into()).is_err(), "unknown status");
    assert!(parse(&|c| c["abilities"]["zap"]["ability_id"] = c["abilities"]["chill"]["ability_id"].clone()).is_err(), "duplicate id");
    assert!(parse(&|c| c["abilities"]["zap"]["class"] = "basic_huge".into()).is_err(), "unknown class");
    assert!(parse(&|c| c["abilities"]["zap"]["shape"] = "spiral".into()).is_err(), "unknown shape");
    assert!(parse(&|c| c["weapons"]["default"]["2"] = "medic/incision+triage".into()).is_err(), "another job's kit");
    assert!(parse(&|c| c["kits"]["ninja/flame_flash+dragonstrike"]["super"] = "zap".into()).is_err(), "a basic as a super");
    assert!(parse(&|c| c["kits"]["medic/incision+triage"]["items"]["1"] = c["kits"]["wizard/zap+lightning_blast"]["items"]["1"].clone()).is_err(), "an item in two kits");
    assert!(parse(&|c| c["classes"]["basic_plain"]["tier_damage"][0] = 0.into()).is_err());
    assert!(parse(&|c| c["schema_version"] = 2.into()).is_err());
    assert!(parse(&|c| c["traits"]["2"][0]["on"] = "sneeze".into()).is_err(), "unknown trait trigger");
    // Without the kit file the engine keeps its older job skills.
    let f = Fixture::new(10_000, 0, false);
    std::fs::remove_file(f.0.join("weapon_kits.json")).unwrap();
    let mut e = f.load(2, 1);
    assert!(!e.kits_live());
    assert_eq!(refusal(&e.cast_slot(1, 100, [0.0; 3], 2, 0).unwrap()), Some(286));
}

/// The engine's hit path: slot 0 lands the kit's basic on the tier ladder, once per 650 ms.
#[test]
fn the_basic_lands_the_ladder_once_per_650_ms() {
    let f = Fixture::new(100_000, 0, false);
    for (job, level, hit, count) in [(2, 1, 279, 3), (2, 14, 1492, 3), (43, 4, 444, 3), (35, 1, 279, 3), (32, 8, 853, 3), (12, 12, 1492, 3), (11, 1, 279, 3)] {
        let mut e = f.load(job, level);
        assert!(e.kits_live());
        let out = e.attack(101, [0.0; 3], job, 0, 1000).unwrap();
        let landed = dealt(&out);
        assert_eq!(landed.len(), count, "job {job}: the three enemies in front");
        for damage in landed {
            assert!(damage == hit || damage == hit * 3 / 2, "job {job} level {level}: {damage} is {hit} or its critical");
        }
        assert!(dealt(&e.attack(101, [0.0; 3], job, 0, 1649).unwrap()).is_empty(), "the gate is silent");
        assert!(refusal(&e.attack(101, [0.0; 3], job, 0, 1649).unwrap()).is_none());
        assert!(!dealt(&e.attack(101, [0.0; 3], job, 0, 1650).unwrap()).is_empty(), "650 ms later it lands");
    }
}

#[test]
fn a_super_costs_100_and_energy_returns_at_4_a_second() {
    let f = Fixture::new(100_000, 0, false);
    let mut e = f.load(2, 1);
    let out = e.cast_slot(1, 101, [0.0; 3], 2, 1000).unwrap();
    assert_eq!(energy(&out), Some(0), "Dragonstrike spends the whole bar");
    assert!(dealt(&out).iter().all(|d| *d == 1143 || *d == 1143 * 3 / 2) && dealt(&out).len() == 3);
    assert_eq!(refusal(&e.cast_slot(1, 101, [0.0; 3], 2, 1500).unwrap()), Some(287), "not ready inside its recast");
    assert_eq!(refusal(&e.cast_slot(1, 101, [0.0; 3], 2, 2100).unwrap()), Some(285), "then not enough energy");
    let mut seen = Vec::new();
    for t in (1000..=27_000).step_by(250) {
        seen.extend(energy(&e.tick([0.0; 3], 2, t).unwrap()));
    }
    assert_eq!(seen.len(), 25, "one step a second");
    assert!(seen.windows(2).all(|w| w[1] - w[0] == 4), "{seen:?}");
    assert_eq!(seen.last(), Some(&100));
    assert!(refusal(&e.cast_slot(1, 101, [0.0; 3], 2, 27_100).unwrap()).is_none());
    // An empty slot and a locked third ability are refused, not ignored.
    assert_eq!(refusal(&e.cast_slot(3, 101, [0.0; 3], 2, 27_200).unwrap()), Some(286));
    let mut e = f.load(2, 1);
    assert_eq!(refusal(&e.cast_slot(1, 101, [500.0, 0.0, 500.0], 2, 1000).unwrap()), Some(288), "nothing in reach");
    assert_eq!(energy(&e.cast_slot(1, 101, [0.0; 3], 2, 1100).unwrap()), Some(0), "and a refusal spent nothing");
}

#[test]
fn casts_and_kills_name_the_ability() {
    let f = Fixture::new(1, 0, false);
    let mut e = f.load(35, 1);
    let volley = table().ability("volley").unwrap().ability_id;
    let barrage = table().ability("barrage").unwrap().ability_id;
    let out = e.cast_slot(1, 100, [0.0; 3], 35, 1000).unwrap();
    // Our id, and the 2009 skill id quest 3010011 still names.
    assert_eq!(out.abilities_used, [(volley, Some(400)), (62019, Some(400))]);
    assert!(out.facts.contains(&FightFact::Used { ability: volley, on_kind: Some(400) }));
    assert!(out.facts.contains(&FightFact::Used { ability: 62019, on_kind: Some(400) }));
    assert_eq!(out.kills.len(), 3, "Volley fells the three weak enemies");
    assert!(out.kills.iter().all(|k| k.ability == Some(volley) && k.kind == 400 && k.job == 35));
    let mut e = f.load(35, 1);
    let out = e.attack(100, [0.0; 3], 35, 0, 1000).unwrap();
    assert!(!out.kills.is_empty() && out.kills.iter().all(|k| k.ability == Some(barrage)));
    assert_eq!(out.abilities_used, [(barrage, Some(400))]);
}

#[test]
fn the_engine_casts_the_equipped_weapons_kit() {
    let f = Fixture::new(100_000, 0, false);
    let mut e = f.load(2, 20);
    assert_eq!(e.kit_of(2).unwrap().0.id, "ninja/flame_flash+dragonstrike");
    let (item, power) = item_of("ninja/lightning_strike+1000_storms");
    assert_eq!(power, 2);
    e.equip_weapon(2, WeaponFacts { item_id: item, ..Default::default() });
    let (k, tier) = e.kit_of(2).unwrap();
    assert_eq!((k.id.as_str(), tier), ("ninja/lightning_strike+1000_storms", 2), "a Power 2 weapon at level 20");
    let out = e.attack(100, [0.0; 3], 2, 0, 1000).unwrap();
    assert_eq!(out.abilities_used[0].0, table().ability("lightning_strike").unwrap().ability_id);
    assert_eq!(dealt(&out).len(), 1, "Lightning Strike is single target");
    // 444 at Power 2, and the level-15 trait Ninja's Grace adds 5 %.
    assert!(matches!(dealt(&out)[0], 466 | 699), "{:?}", dealt(&out));
    // Another job's weapon record does not leak across a job switch.
    e.set_player_job(43, 20);
    assert_eq!(e.kit_of(43).unwrap().0.id, "brawler/glancing_blow+leg_sweep");
}

#[test]
fn traits_come_with_the_job_level() {
    // Warrior 15, High Morale: one more energy per step.
    let f = Fixture::new(100_000, 0, false);
    let mut e = f.load(32, 15);
    assert_eq!(energy(&e.cast_slot(1, 101, [0.0; 3], 32, 1000).unwrap()), Some(0));
    let mut first = None;
    for t in (1000..=2500).step_by(250) {
        first = first.or(energy(&e.tick([0.0; 3], 32, t).unwrap()));
    }
    assert_eq!(first, Some(5));
    let mut e = f.load(32, 14);
    e.cast_slot(1, 101, [0.0; 3], 32, 1000).unwrap();
    let mut first = None;
    for t in (1000..=2500).step_by(250) {
        first = first.or(energy(&e.tick([0.0; 3], 32, t).unwrap()));
    }
    assert_eq!(first, Some(4), "not before level 15");
    // Medic 20, Recovery: energy for a knockout.
    let weak = Fixture::new(1, 0, false);
    let mut e = weak.load(11, 20);
    let out = e.cast_slot(1, 101, [0.0; 3], 11, 1000).unwrap();
    assert!(!out.kills.is_empty());
    assert_eq!(energy(&out), Some(10), "100 spent, 10 back for the knockout");
    let mut e = weak.load(11, 19);
    assert_eq!(energy(&e.cast_slot(1, 101, [0.0; 3], 11, 1000).unwrap()), Some(0));
    // Archer 5, Precision: 5 % more damage.
    let mut e = f.load(35, 5);
    assert!(dealt(&e.attack(101, [0.0; 3], 35, 0, 1000).unwrap()).iter().all(|d| matches!(*d, 512 | 768)), "488 x 1.05");
}

#[test]
fn pb2_a_medic_group_heal_aimed_at_a_member_goes_out_and_a_hit_aimed_at_one_does_not() {
    // The client sends the selected member's guid with the cast. A group heal treats it as the
    // ally the Medic aims at: the cast lands and the group heal goes out for the host to route.
    let f = Fixture::new(100_000, 0, false);
    let member = 0x0000_0000_0000_0303;
    let mut medic = f.load(11, 1);
    let out = medic.cast_slot(1, member, [0.0; 3], 11, 1000).unwrap();
    assert!(refusal(&out).is_none(), "the cast is not refused");
    assert!(out.group_heal.is_some(), "Triage's group heal goes out");
    // A damage ability aimed at a member is still the forged target it was: nothing happens.
    let mut ninja = f.load(2, 1);
    let out = ninja.cast_slot(1, member, [0.0; 3], 2, 1000).unwrap();
    assert!(out.replies.is_empty() && out.group_heal.is_none() && dealt(&out).is_empty());
}
