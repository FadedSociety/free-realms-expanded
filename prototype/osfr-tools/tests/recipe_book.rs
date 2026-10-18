//! 26/24 recipe-book start: decode, station proximity and the call into the
//! real `begin_recipe` of the Cooking, Smelting and Forging services.
//! Synthetic content only; no player names or chat.
use osfr_tools::{
    cooking::Cooking,
    forging::Forging,
    opening::{OpeningActor, OpeningTrigger},
    recipe_book::{self, Player, Refusal, Services, Started, Station},
    recipes::{CraftingJob, RecipeCatalog},
    smelting::Smelting,
};
use serde_json::json;
use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
const STAGES: [i32; 6] = [1, 2, 3, 6, 17, 7];
const ZONE: u32 = 7;
const HERE: [f32; 3] = [100.0, 0.0, 100.0];

// Catalog facts used below (data/content/recipes.json).
const GUMBO: i32 = 35; // chef, rank 1
const BRIARMELON: i32 = 57; // chef, rank 20
const HAMMER: i32 = 11; // blacksmith, rank 1
const GUMBO_INPUTS: [(i32, u32); 4] = [(2875, 3), (2881, 3), (2890, 3), (2899, 3)];
const HAMMER_INPUTS: [(i32, u32); 4] = [(21785, 10), (2752, 5), (55, 5), (56, 5)];

struct Fixture {
    dir: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "osfr-recipe-book-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).unwrap();
        let phases = |rows: Vec<(Vec<i32>, i32)>| {
            rows.into_iter()
                .enumerate()
                .map(|(i, (data, stage))| {
                    json!({"id":i+1,"sequence":i+1,"duration":90,"objective_id":0,
                    "start_id":0,"end_id":0,"stage_type":stage,"data":data})
                })
                .collect::<Vec<_>>()
        };
        let cooking = [
            [10, 8, 15, 2, 50, 1, 0, 0, 0, 0, 0],
            [24, 15, 1, 1, 100, 0, 0, 0, 0, 0, 0],
            [40, 2, 2, 0, 20, 20, 2, 3, 100, 200, 400],
            [49, 8, 3, 100, 500, 0, 0, 0, 0, 0, 0],
            [46, 0, 500, 0, 0, 0, 0, 0, 0, 0, 0],
            [46, 10, 0, 100, 0, 0, 0, 0, 0, 0, 0],
        ];
        let smelting = [
            [1, 5, 1, 1, 0, 20, 2, 3, 50, 100, 200],
            [65, 8, 3, 100, 100, 0, 0, 0, 0, 0, 0],
            [100, 1, 500, 0, 0, 0, 0, 0, 0, 0, 0],
            [2, 500, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ];
        let write = |name: &str, value: serde_json::Value| {
            fs::write(dir.join(name), serde_json::to_vec(&value).unwrap()).unwrap()
        };
        write(
            "cooking.json",
            json!({"schema_version":1,"game_id":46,"profile_id":45,"name_id":501,
            "description_id":502,"icon_id":503,"reconstructed_phase_values":true,
            "phase_notice":"Synthetic reconstructed test configuration",
            "phases":phases(cooking.iter().zip(STAGES).map(|(d, s)| (d.to_vec(), s)).collect()),
            "recipe":{"key":"chef.strong_arm_stew.sample"}}),
        );
        write(
            "smelting.json",
            json!({"schema_version":1,"game_id":77,"profile_id":14,"name_id":501,
            "description_id":502,"icon_id":503,"reconstructed_phase_values":true,
            "phase_notice":"synthetic explicitly reconstructed test configuration",
            "phases":phases(smelting.iter().enumerate().map(|(i, d)| (d.to_vec(), 101 + i as i32)).collect()),
            "recipe":{"key":"miner.copper_bar.sample"}}),
        );
        write(
            "forging.json",
            json!({"schema_version":1,"game_id":139,"profile_id":16,"name_id":501,
            "description_id":502,"icon_id":503,"reconstructed_rules":true,
            "rule_notice":"Explicit synthetic reconstructed test board, consumption and tint.",
            "board":{"left":3,"top":3,"width":8,"height":8,"duration_seconds":180,"quota":1,
            "selectable_types":[1,2,3,4,5],"collectable_percent":10,"seed":139,"mold_id":6},
            "recipe":{"key":"blacksmith.spunky_scrapper_hammer.sample",
            "reconstructed_consumption":true,"reconstructed_tint_selection":true}}),
        );
        fs::write(
            dir.join("recipes.json"),
            include_str!("../data/content/recipes.json"),
        )
        .unwrap();
        Self { dir }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

struct World {
    _fixture: Fixture,
    catalog: RecipeCatalog,
    cooking: Cooking,
    smelting: Smelting,
    forging: Forging,
    stations: Vec<Station>,
    learned: BTreeSet<u32>,
    available: Vec<(i32, u32)>,
    profile: i32,
    rank: u8,
    position: [f32; 3],
    zone: u32,
    in_activity: bool,
}
impl World {
    fn new() -> Self {
        let fixture = Fixture::new();
        let station = |job| Station {
            job,
            zone_id: ZONE,
            position: HERE,
            radius: 12.0,
        };
        Self {
            catalog: RecipeCatalog::load_runtime(&fixture.dir).unwrap(),
            cooking: Cooking::load(&fixture.dir, None).unwrap(),
            smelting: Smelting::load(&fixture.dir, None).unwrap(),
            forging: Forging::load(&fixture.dir, None).unwrap(),
            _fixture: fixture,
            stations: vec![
                station(CraftingJob::Chef),
                station(CraftingJob::Miner),
                station(CraftingJob::Blacksmith),
            ],
            learned: BTreeSet::from([GUMBO as u32, BRIARMELON as u32, HAMMER as u32]),
            available: GUMBO_INPUTS.iter().chain(&HAMMER_INPUTS).copied().collect(),
            profile: 45,
            rank: 1,
            position: HERE,
            zone: ZONE,
            in_activity: false,
        }
    }
    fn start(&mut self, recipe: i32) -> Result<Started, Refusal> {
        let player = Player {
            zone_id: self.zone,
            position: self.position,
            active_profile: self.profile,
            job_rank: self.rank,
            learned: &self.learned,
            available: &self.available,
            in_activity: self.in_activity,
        };
        recipe_book::start(
            recipe,
            &self.catalog,
            &player,
            &self.stations,
            Services {
                cooking: Some(&mut self.cooking),
                smelting: Some(&mut self.smelting),
                forging: Some(&mut self.forging),
            },
        )
    }
    /// The refusal changed nothing in any of the three services.
    fn refuse(&mut self, recipe: i32, expected: Refusal) {
        let before = (
            self.cooking.state().clone(),
            self.smelting.state().clone(),
            self.forging.state().clone(),
        );
        match self.start(recipe) {
            Err(refusal) => assert_eq!(refusal, expected, "recipe {recipe}"),
            Ok(_) => panic!("recipe {recipe} should have been refused with {expected:?}"),
        }
        assert_eq!(self.cooking.state(), &before.0);
        assert_eq!(self.smelting.state(), &before.1);
        assert_eq!(self.forging.state(), &before.2);
    }
}

fn request(recipe: i32) -> Vec<u8> {
    let mut packet = vec![26, 0, 24, 0];
    packet.extend(recipe.to_le_bytes());
    packet
}

// ---- decode ----

#[test]
fn decode_reads_the_recovered_eight_byte_body() {
    assert_eq!(recipe_book::decode(&request(35)).unwrap(), Some(35));
    assert_eq!(recipe_book::decode(&request(-5)).unwrap(), Some(-5));
}

#[test]
fn decode_ignores_other_messages_and_rejects_bad_lengths() {
    for other in [
        &[26u8, 0, 10, 0, 1, 0, 0, 0][..],
        &[27, 0, 24, 0, 0, 0, 0, 0],
        &[],
        &[26, 0],
    ] {
        assert_eq!(recipe_book::decode(other).unwrap(), None);
    }
    let good = request(35);
    for len in [4, 5, 7] {
        assert!(recipe_book::decode(&good[..len]).is_err(), "len {len}");
    }
    let mut long = good;
    long.push(0);
    assert!(recipe_book::decode(&long).is_err());
}

// ---- accepted ----

#[test]
fn chef_recipe_starts_at_a_station_with_exact_inputs() {
    let mut w = World::new();
    let Ok(Started::Cooking(outcome)) = w.start(GUMBO) else {
        panic!("chef start refused")
    };
    assert!(outcome.changed && !outcome.replies.is_empty());
    assert!(w.cooking.active());
    assert_eq!(w.cooking.recipe_key(), "chef.gummyfruit_gumbo");
    // The completion transaction will consume exactly the catalog inputs.
    assert_eq!(w.cooking.requirements(), GUMBO_INPUTS);
    assert!(!w.forging.active() && !w.smelting.active());
}

#[test]
fn blacksmith_recipe_starts_at_the_anvil() {
    let mut w = World::new();
    w.profile = 16;
    assert!(matches!(w.start(HAMMER), Ok(Started::Forging(_))));
    assert!(w.forging.active() && !w.cooking.active());
    assert_eq!(w.forging.requirements(), HAMMER_INPUTS);
}

#[test]
fn miner_recipe_with_an_id_starts_through_smelting() {
    // The shipped miner rows have no recipe id yet (client cannot name them),
    // so give the copper-bar sample one to prove the smelting arm.
    let mut w = World::new();
    let sample = w
        .catalog
        .records
        .iter_mut()
        .find(|r| r.key == "miner.copper_bar.sample")
        .unwrap();
    sample.recipe_id = Some(900);
    let inputs: Vec<(i32, u32)> = sample
        .ingredients
        .iter()
        .map(|i| (i.definition_id, i.count))
        .collect();
    w.learned.insert(900);
    w.available = inputs;
    w.profile = 14;
    assert!(matches!(w.start(900), Ok(Started::Smelting(_))));
    assert!(w.smelting.active());
}

#[test]
fn rank_and_inputs_at_the_limit_are_accepted() {
    let mut w = World::new();
    w.rank = 20;
    w.available = vec![(2883, 3), (2875, 3), (2884, 3), (2906, 8)]; // exact briarmelon set
    assert!(matches!(w.start(BRIARMELON), Ok(Started::Cooking(_))));
}

#[test]
fn split_stacks_of_one_ingredient_add_up() {
    let mut w = World::new();
    w.available = vec![
        (2875, 1),
        (2875, 2),
        (2881, 3),
        (2890, 3),
        (2899, 1),
        (2899, 2),
    ];
    assert!(w.start(GUMBO).is_ok());
}

#[test]
fn station_edge_is_inclusive() {
    let mut w = World::new();
    w.position = [HERE[0] + 12.0, HERE[1], HERE[2]];
    assert!(w.start(GUMBO).is_ok());
}

// ---- refused, each with a no-mutation canary ----

#[test]
fn unknown_recipe_ids_are_refused_without_panic() {
    let mut w = World::new();
    for id in [0, -1, 1, 12345, i32::MAX, i32::MIN, 2001, 2002, 2003] {
        w.refuse(id, Refusal::UnknownRecipe);
    }
}

#[test]
fn unlearned_recipe_is_refused() {
    let mut w = World::new();
    w.learned.remove(&(GUMBO as u32));
    w.refuse(GUMBO, Refusal::NotLearned);
}

#[test]
fn wrong_active_job_is_refused() {
    let mut w = World::new();
    w.profile = 16; // Blacksmith asking for a Chef recipe
    w.refuse(GUMBO, Refusal::WrongJob(CraftingJob::Chef));
    w.profile = 45;
    w.refuse(HAMMER, Refusal::WrongJob(CraftingJob::Blacksmith));
}

#[test]
fn distant_recipe_is_refused() {
    let mut w = World::new();
    w.position = [HERE[0] + 12.5, HERE[1], HERE[2]];
    w.refuse(GUMBO, Refusal::NoStation(CraftingJob::Chef));
    w.position = [0.0, 0.0, 0.0];
    w.refuse(GUMBO, Refusal::NoStation(CraftingJob::Chef));
}

#[test]
fn other_zone_station_does_not_count() {
    let mut w = World::new();
    w.zone = ZONE + 1;
    w.refuse(GUMBO, Refusal::NoStation(CraftingJob::Chef));
}

#[test]
fn only_a_station_of_the_recipes_own_job_counts() {
    let mut w = World::new();
    w.stations.retain(|s| s.job == CraftingJob::Blacksmith);
    w.refuse(GUMBO, Refusal::NoStation(CraftingJob::Chef));
    w.stations.clear();
    w.refuse(GUMBO, Refusal::NoStation(CraftingJob::Chef));
}

#[test]
fn non_finite_positions_never_reach_a_station() {
    let mut w = World::new();
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        w.position = [bad, 0.0, 100.0];
        w.refuse(GUMBO, Refusal::NoStation(CraftingJob::Chef));
    }
}

#[test]
fn rank_too_low_is_refused() {
    let mut w = World::new();
    w.rank = 19;
    w.refuse(BRIARMELON, Refusal::RankTooLow);
}

#[test]
fn one_missing_ingredient_is_refused() {
    let mut w = World::new();
    w.available = vec![(2875, 3), (2881, 3), (2890, 3), (2899, 2)]; // one short
    w.refuse(GUMBO, Refusal::MissingIngredients);
    w.available.clear();
    w.refuse(GUMBO, Refusal::MissingIngredients);
}

#[test]
fn any_live_activity_blocks_the_start() {
    let mut w = World::new();
    w.in_activity = true;
    w.refuse(GUMBO, Refusal::Busy);
}

#[test]
fn an_active_run_cannot_be_replaced_or_doubled() {
    let mut w = World::new();
    assert!(w.start(GUMBO).is_ok());
    let after_first = w.cooking.state().clone();
    w.refuse(GUMBO, Refusal::Busy); // replay of the same request
    assert_eq!(w.cooking.state(), &after_first);
    w.profile = 16;
    w.refuse(HAMMER, Refusal::Busy); // another job's movie while cooking runs
}

#[test]
fn missing_service_is_unavailable_not_a_panic() {
    let w = World::new();
    let player = Player {
        zone_id: ZONE,
        position: HERE,
        active_profile: 45,
        job_rank: 1,
        learned: &w.learned,
        available: &w.available,
        in_activity: false,
    };
    let result = recipe_book::start(
        GUMBO,
        &w.catalog,
        &player,
        &w.stations,
        Services {
            cooking: None,
            smelting: None,
            forging: None,
        },
    );
    assert!(matches!(result, Err(Refusal::Unavailable)));
}

// ---- stations and feedback ----

fn actor(action: Option<OpeningTrigger>) -> OpeningActor {
    OpeningActor {
        guid: 1,
        definition_id: 0,
        name_id: 0,
        name: String::new(),
        model_id: 0,
        model_file_name: String::new(),
        zone_id: ZONE,
        position: HERE,
        heading: 0.0,
        interaction_radius: 12.0,
        kind: "object".into(),
        action,
        required_profile: None,
        provenance: String::new(),
    }
}

#[test]
fn stations_come_only_from_craft_actors() {
    let actors = [
        actor(Some(OpeningTrigger::Craft { recipe_id: 2001 })),
        actor(Some(OpeningTrigger::Craft { recipe_id: 2002 })),
        actor(Some(OpeningTrigger::Craft { recipe_id: 2003 })),
        actor(Some(OpeningTrigger::Craft { recipe_id: 70 })),
        actor(Some(OpeningTrigger::Gather { resource_id: 1001 })),
        actor(None),
    ];
    let jobs: Vec<_> = recipe_book::stations(&actors)
        .iter()
        .map(|s| s.job)
        .collect();
    assert_eq!(
        jobs,
        [
            CraftingJob::Chef,
            CraftingJob::Miner,
            CraftingJob::Blacksmith
        ]
    );
}

#[test]
fn every_refusal_has_a_system_chat_line() {
    for refusal in [
        Refusal::Busy,
        Refusal::UnknownRecipe,
        Refusal::NotLearned,
        Refusal::WrongJob(CraftingJob::Chef),
        Refusal::NoStation(CraftingJob::Miner),
        Refusal::RankTooLow,
        Refusal::MissingIngredients,
        Refusal::Unavailable,
    ] {
        let packet = refusal.feedback();
        assert_eq!(&packet[..4], &[15, 0, 3, 0]);
        assert!(!refusal.message().is_empty());
    }
}
