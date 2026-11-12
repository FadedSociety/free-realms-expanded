//! KNOWN-BUGS C4 + C9 for the crafting movies, on the cooking engine with the movie's own
//! messages: a won craft previews and lands its recipe's prize wheel (39/45 between 39/18 and
//! 39/19) and carries the recipe's coins, the wheel's slot and its `experience_points` (the wiki's
//! stars) to the gameplay payout. The gameplay applier (`apply_craft`) is the one every station
//! and Games window craft goes through.

use osfr_tools::{
    cooking::{Cooking, payload_packet},
    minigame::{MinigameRewards, Prize},
};
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
const STAGES: [i32; 6] = [1, 2, 3, 6, 17, 7];
const RECIPE: &str = "chef.strong_arm_stew.sample";
const REWARDS: &str = r#"{
  "schema_version": 1,
  "recipes": [{
    "recipe": "chef.strong_arm_stew.sample",
    "coins": 4,
    "wheel": {"coins": 3, "slices": [
      {"item_id": 31224, "name_id": 0, "icon_id": 0, "quantity": 3, "gender": 0}
    ]}
  }]
}"#;

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-minigame-rewards-crafting-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        let data = [
            [10, 8, 15, 2, 50, 1, 0, 0, 0, 0, 0],
            [24, 15, 1, 1, 100, 0, 0, 0, 0, 0, 0],
            [40, 2, 2, 0, 20, 20, 2, 3, 100, 200, 400],
            [49, 8, 3, 100, 500, 0, 0, 0, 0, 0, 0],
            [46, 0, 500, 0, 0, 0, 0, 0, 0, 0, 0],
            [46, 10, 0, 100, 0, 0, 0, 0, 0, 0, 0],
        ];
        let phases: Vec<_> = (data.iter().enumerate())
            .map(|(i, d)| {
                json!({"id": i + 1, "sequence": i + 1, "duration": 90, "objective_id": 0,
                    "start_id": 0, "end_id": 0, "stage_type": STAGES[i], "data": d})
            })
            .collect();
        let cooking = json!({"schema_version": 1, "game_id": 46, "profile_id": 45,
            "name_id": 501, "description_id": 502, "icon_id": 503,
            "reconstructed_phase_values": true,
            "phase_notice": "Synthetic explicitly reconstructed configuration",
            "phases": phases, "recipe": {"key": RECIPE}});
        fs::write(path.join("cooking.json"), serde_json::to_vec(&cooking).unwrap()).unwrap();
        let recipes = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/recipes.json");
        fs::copy(recipes, path.join("recipes.json")).unwrap();
        fs::write(path.join("minigame_rewards.json"), REWARDS).unwrap();
        Self(path)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn id(game: &Cooking) -> i32 {
    game.state().active.as_ref().unwrap().instance_id
}

/// The movie's 39/14 text as the client sends it after launch (GameId -1).
fn text(game: &mut Cooking, text: &str) -> osfr_tools::cooking::CookingOutcome {
    let mut packet = payload_packet(id(game), text).unwrap();
    packet[11..15].copy_from_slice(&(-1i32).to_le_bytes());
    game.handle(&packet).unwrap()
}

fn i32_at(bytes: &[u8], at: usize) -> i32 {
    i32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

#[test]
fn a_won_craft_lands_its_wheel_and_carries_the_recipes_payout() {
    let fixture = Fixture::new();
    let mut game = Cooking::load(&fixture.0, None).unwrap();
    let offer = game.begin(45).unwrap();
    // The 39/16 preview bundle (third) lists the wheel: Coins slot 3, one Item entry.
    let info = &offer.replies[0];
    let preview = 40 + 2 * 69;
    assert_eq!(i32_at(info, preview + 1), 3);
    assert_eq!(i32_at(info, preview + 61), 1);
    assert_eq!(i32_at(info, preview + 86), 31224);

    let mut go = vec![39, 0, 5];
    for n in [id(&game), -1, -1] {
        go.extend(n.to_le_bytes());
    }
    game.handle(&go).unwrap();
    text(&mut game, "FRServer_ReadyForGameData");
    for stage in STAGES {
        for kind in 0..4 {
            text(&mut game, &format!("FRServer_StatUpdate\t{stage}\t{kind}\t1)"));
        }
    }
    for kind in 4..8 {
        text(&mut game, &format!("FRServer_StatUpdate\t-1\t{kind}\t6)"));
    }
    let instance = id(&game);
    let won = text(&mut game, "FRServer_GameWon\t900\t0");
    let subs: Vec<u8> = won.replies.iter().map(|reply| reply[2]).collect();
    assert_eq!(subs, [14, 18, 47, 45, 19], "the wheel lands ahead of 39/19");

    let wheel = MinigameRewards::parse(REWARDS.as_bytes()).unwrap().recipe(RECIPE).unwrap().wheel.clone();
    let prize = wheel.prize(instance).unwrap();
    let payout = won.payout.expect("a listed recipe pays");
    match prize {
        Prize::Item(slice) => {
            assert_eq!((payout.coins, payout.items.clone()), (4, vec![(slice.item_id, 3)]));
            assert_eq!(i32_at(&won.replies[3], 15 + 86), 31224, "the landing entry");
        }
        Prize::Coins(coins) => {
            assert_eq!((payout.coins, payout.items.len()), (4 + coins, 0));
            assert_eq!(i32_at(&won.replies[3], 15 + 1), coins as i32, "the Coins slot");
        }
    }
    // C9: the recipe's stars ride the transaction (recipes.json experience_points, wiki 25).
    let recipes: serde_json::Value = serde_json::from_slice(
        &fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/recipes.json")).unwrap(),
    )
    .unwrap();
    let stars = (recipes["records"].as_array().unwrap().iter())
        .find(|row| row["key"] == RECIPE)
        .and_then(|row| row["experience_points"].as_u64())
        .unwrap();
    assert!(stars > 0, "the recipe pays stars");
    assert_eq!(u64::from(won.transaction.unwrap().experience_points), stars);
}
