#[path = "common/crafting_results.rs"]
mod crafting_results;
use osfr_tools::cooking::{self, Cooking, CookingPhase, CookingState, payload_packet};
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
const STAGES: [i32; 6] = [1, 2, 3, 6, 17, 7];

struct Fixture {
    path: PathBuf,
    content: Value,
}
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-cooking-{}-{}",
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
        let content = json!({"schema_version":1,"game_id":46,"profile_id":45,
            "name_id":501,"description_id":502,"icon_id":503,
            "reconstructed_phase_values":true,"phase_notice":"Synthetic explicitly reconstructed configuration",
            "phases":data.iter().enumerate().map(|(i,d)| json!({"id":i+1,"sequence":i+1,"duration":90,
                "objective_id":0,"start_id":0,"end_id":0,"stage_type":STAGES[i],"data":d})).collect::<Vec<_>>(),
            "recipe":{"key":"chef.strong_arm_stew.sample"}});
        let f = Self { path, content };
        f.write();
        f
    }
    fn write(&self) {
        fs::write(
            self.path.join("cooking.json"),
            serde_json::to_vec(&self.content).unwrap(),
        )
        .unwrap();
        fs::write(
            self.path.join("recipes.json"),
            include_str!("../data/content/recipes.json"),
        )
        .unwrap();
    }
    fn load(&self, state: Option<CookingState>) -> Cooking {
        Cooking::load(&self.path, state).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.path.join("cooking.json"));
        let _ = fs::remove_file(self.path.join("recipes.json"));
        let _ = fs::remove_dir(&self.path);
    }
}
fn id(g: &Cooking) -> i32 {
    g.state().active.as_ref().unwrap().instance_id
}
fn send(g: &mut Cooking, text: &str) -> cooking::CookingOutcome {
    g.handle(&payload_packet(id(g), text).unwrap()).unwrap()
}
fn start(g: &mut Cooking) -> cooking::CookingOutcome {
    let mut b = vec![39, 0, 5];
    for n in [id(g), -1, 46] {
        b.extend(n.to_le_bytes());
    }
    g.handle(&b).unwrap()
}

#[test]
fn captured_native_go_sentinel_starts_only_the_matching_offered_instance() {
    let fixture = Fixture::new();
    let mut game = fixture.load(None);
    game.begin(45).unwrap();
    // Exact reassembled native run154503 frame4726 / app2558,39/5.
    let captured = [
        0x27, 0, 5, 0x81, 0xe7, 0xbd, 2, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    ];
    assert_eq!(captured.as_slice(), native_vector("cooking_start"));
    assert_eq!(id(&game), 46_000_001);
    let offered = game.state().clone();
    for (offset, value) in [
        (3, id(&game) + 1),
        (7, 0),
        (11, 0),
        (11, 77),
        (11, 560),
        (11, -2),
    ] {
        let mut wrong = captured;
        wrong[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        assert!(game.handle(&wrong).unwrap().replies.is_empty());
        assert_eq!(game.state(), &offered);
    }
    let mut sentinel_payload = payload_packet(id(&game), "FRServer_ReadyForGameData").unwrap();
    sentinel_payload[11..15].copy_from_slice(&(-1i32).to_le_bytes());
    assert_eq!(sentinel_payload, native_vector("cooking_ready"));
    assert!(game.handle(&sentinel_payload).unwrap().replies.is_empty());
    assert_eq!(game.state(), &offered);
    let launched = game.handle(&captured).unwrap();
    assert_eq!(launched.replies, vec![vector("start"), vector("launch")]);
    assert_eq!(
        game.state().active.as_ref().unwrap().phase,
        CookingPhase::Playing
    );
    let playing = game.state().clone();
    assert!(game.handle(&captured).unwrap().replies.is_empty());
    assert_eq!(game.state(), &playing);
    for (offset, value) in [(3, id(&game) + 1), (7, 0), (11, 0), (11, 77), (11, -2)] {
        let mut wrong = sentinel_payload.clone();
        wrong[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        assert!(game.handle(&wrong).unwrap().replies.is_empty());
        assert_eq!(game.state(), &playing);
    }
    let mut malformed = sentinel_payload.clone();
    malformed.pop();
    assert!(game.handle(&malformed).is_err());
    assert_eq!(game.state(), &playing);
    assert_eq!(game.handle(&sentinel_payload).unwrap().replies.len(), 8);
    let supplied = game.state().clone();
    assert!(game.handle(&sentinel_payload).unwrap().replies.is_empty());
    assert_eq!(game.state(), &supplied);
    game.cancel();
    assert!(game.handle(&captured).unwrap().replies.is_empty());
    game.begin(45).unwrap();
    let next_offer = game.state().clone();
    assert!(game.handle(&captured).unwrap().replies.is_empty());
    assert_eq!(game.state(), &next_offer);
}
fn ready(g: &mut Cooking) {
    g.begin(45).unwrap();
    start(g);
    send(g, "FRServer_ReadyForGameData");
}

#[test]
fn actual_native_stat_suffix_is_numeric_only_and_preserves_order_and_economy() {
    let f = Fixture::new();
    let mut g = f.load(None);
    // Native162238 restored/canceled instance46000001 before this offer.
    g.begin(45).unwrap();
    g.cancel();
    ready(&mut g);
    assert_eq!(id(&g), 46_000_002);
    let captured = [
        "27000e82e7bd02ffffffffffffffff1c00000046525365727665725f53746174557064617465093109300933332900",
        "27000e82e7bd02ffffffffffffffff1b00000046525365727665725f537461745570646174650931093109382900",
        "27000e82e7bd02ffffffffffffffff1b00000046525365727665725f537461745570646174650931093209302900",
        "27000e82e7bd02ffffffffffffffff1c00000046525365727665725f53746174557064617465093109330939392900",
    ];
    for hex in captured {
        let packet: Vec<u8> = hex
            .as_bytes()
            .chunks_exact(2)
            .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
            .collect();
        let out = g.handle(&packet).unwrap();
        assert!(out.transaction.is_none());
        assert!(!out.completed);
    }
    assert_eq!(
        g.state().active.as_ref().unwrap().round_stats[0],
        [Some(33.0), Some(8.0), Some(0.0), Some(99.0)]
    );
    let before = g.state().clone();
    for message in [
        "FRServer_StatUpdate\t1\t0\t33))",
        "FRServer_StatUpdate\t1\t0\t)",
        "FRServer_StatUpdate\t1\t0\t(33)",
        "FRServer_StatUpdate\t1\t0\t33)junk",
        "FRServer_StatUpdate\t1)\t0\t33)",
        "FRServer_StatUpdate\t1\t0)\t33)",
        "FRServer_StatUpdate\t1\t0\t34)",
        "FRServer_StatUpdate\t2\t0\t0.5)",
        "FRServer_StatUpdate\t2\t3\t3601)",
        "FRServer_StatUpdate\t2\t0\tNaN)",
        "FRServer_StatUpdate\t2\t0\tinf)",
        "FRServer_StatUpdate\t3\t0\t1)",
        "FRServer_StatUpdate\t-1\t4\t99)",
        "FRServer_StopLevel\t6\t1))",
        "FRServer_StopLevel\t6)\t1)",
        "FRServer_StopLevel\t7\t1)",
        "FRServer_StopLevel\t6\t2)",
        "FRServer_ScoreSet\t800)",
        "FRServer_ScoreSet\t1000000001",
        "FRServer_GameWon\t800\t0)",
    ] {
        assert!(
            g.handle(&payload_packet(id(&g), message).unwrap()).is_err(),
            "{message}"
        );
        assert_eq!(g.state(), &before, "{message}");
    }
    assert!(!send(&mut g, "FRServer_StatUpdate\t1\t0\t33)").changed);
    for message in ["FRServer_StopLevel\t6\t1)", "FRServer_StopLevel\t6\t0"] {
        let out = send(&mut g, message);
        assert!(!out.changed && out.transaction.is_none() && !out.completed);
    }
    let score = send(&mut g, "FRServer_ScoreSet\t800");
    assert!(score.changed && score.transaction.is_none() && !score.completed);
    assert_eq!(g.state().active.as_ref().unwrap().score, 800);
    assert_eq!(g.state().completed, 0);
    let before = g.state().clone();
    assert!(
        g.handle(&payload_packet(id(&g), "FRServer_GameWon\t800\t0").unwrap())
            .is_err()
    );
    assert_eq!(g.state(), &before);
}
fn rounds(g: &mut Cooking) {
    for stage in STAGES {
        for kind in 0..4 {
            send(g, &format!("FRServer_StatUpdate\t{stage}\t{kind}\t1"));
        }
    }
}
fn totals(g: &mut Cooking) {
    for kind in 4..8 {
        send(g, &format!("FRServer_StatUpdate\t-1\t{kind}\t6"));
    }
}
fn vector(name: &str) -> Vec<u8> {
    let v: Value =
        serde_json::from_str(include_str!("fixtures/cooking-community-vectors.json")).unwrap();
    v["vectors"][name]
        .as_str()
        .unwrap()
        .as_bytes()
        .chunks_exact(2)
        .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
        .collect()
}

fn native_vector(name: &str) -> Vec<u8> {
    let value: Value =
        serde_json::from_str(include_str!("fixtures/native-crafting-client-vectors.json")).unwrap();
    value["vectors"][name]
        .as_str()
        .unwrap()
        .as_bytes()
        .chunks_exact(2)
        .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn native_protocol_matches_compiled_community_vectors() {
    let f = Fixture::new();
    let mut g = f.load(None);
    assert_eq!(g.begin(45).unwrap().replies[0], vector("info"));
    let launched = start(&mut g);
    assert_eq!(launched.replies[0], vector("start"));
    assert_eq!(launched.replies[1], vector("launch"));
    assert_eq!(start(&mut g).replies.len(), 1);
    assert_eq!(
        payload_packet(id(&g), "FRServer_ReadyForGameData").unwrap(),
        vector("payload")
    );
    let data = send(&mut g, "FRServer_ReadyForGameData");
    assert_eq!(data.replies.len(), 8);
    for (i, b) in data.replies[1..7].iter().enumerate() {
        let s = std::str::from_utf8(&b[19..b.len() - 1]).unwrap();
        let row: Vec<_> = s
            .split('\t')
            .nth(1)
            .unwrap()
            .trim_end_matches('^')
            .split('^')
            .collect();
        assert_eq!(row.len(), 19);
        assert_eq!(row[2], (i + 1).to_string());
        assert_eq!(row[7], STAGES[i].to_string());
    }
    assert!(send(&mut g, "FRServer_ReadyForGameData").replies.is_empty());
    rounds(&mut g);
    totals(&mut g);
    let won = send(&mut g, "FRServer_GameWon\t900\t0");
    assert_eq!(
        &won.replies[1..],
        &[
            crafting_results::vector("cooking_won"),
            crafting_results::vector("cooking_scores"),
            crafting_results::vector("cooking_remove")
        ]
    );
    assert!(g.cancel().replies.is_empty());
}

#[test]
fn six_ordered_rounds_create_fixed_recipe_intent_once_and_restore() {
    let f = Fixture::new();
    let mut g = f.load(None);
    assert!(g.begin(43).is_err());
    ready(&mut g);
    assert!(g.begin(45).is_err());
    assert_eq!(
        g.requirements(),
        &[(2862, 4), (2875, 4), (2884, 4), (2900, 4)]
    );
    rounds(&mut g);
    totals(&mut g);
    let won = send(&mut g, "FRServer_GameWon\t900\t0");
    assert!(won.completed);
    let t = won.transaction.unwrap();
    assert_eq!(t.consumes, g.requirements());
    assert_eq!(t.grants, vec![(35, 1)]);
    let saved: CookingState =
        serde_json::from_slice(&serde_json::to_vec(g.state()).unwrap()).unwrap();
    let mut restored = f.load(Some(saved));
    assert_eq!(restored.state().completed, 1);
    assert!(
        send(&mut restored, "FRServer_GameWon\t900\t0")
            .transaction
            .is_none()
    );
    let old = id(&restored);
    restored.cancel();
    restored.begin(45).unwrap();
    assert_ne!(id(&restored), old);
}

#[test]
fn malformed_unordered_early_or_forged_reports_do_not_commit() {
    let f = Fixture::new();
    let mut g = f.load(None);
    ready(&mut g);
    for text in [
        "FRServer_StatUpdate\t2\t0\t1",
        "FRServer_StatUpdate\t1\t0\tNaN",
        "FRServer_StatUpdate\t1\t1\t1.2",
        "FRServer_StatUpdate\t1\t3\t3601",
        "FRServer_StatUpdate\t-1\t4\t1",
        "FRServer_GameWon\t900\t0",
        "FRServer_GameWon\t900\t999",
    ] {
        let before = g.state().clone();
        assert!(
            g.handle(&payload_packet(id(&g), text).unwrap()).is_err(),
            "{text}"
        );
        assert_eq!(&before, g.state());
    }
    let mut stale = payload_packet(id(&g) - 1, "FRServer_ReadyForGameData").unwrap();
    assert!(!g.handle(&stale).unwrap().changed);
    stale.pop();
    assert!(g.handle(&stale).is_err());
    rounds(&mut g);
    let before = g.state().clone();
    assert!(
        g.handle(&payload_packet(id(&g), "FRServer_StatUpdate\t1\t0\t2").unwrap())
            .is_err()
    );
    assert_eq!(&before, g.state());
    assert!(!send(&mut g, "FRServer_StatUpdate\t1\t0\t1").changed);
    totals(&mut g); // Mutating even a completed statistic cannot alter the totals.
    assert!(
        g.handle(&payload_packet(id(&g), "FRServer_StatUpdate\t-1\t5\t999").unwrap())
            .is_err()
    );
}

#[test]
fn final_totals_and_all_six_rounds_are_required_for_reward() {
    let f = Fixture::new();
    let mut g = f.load(None);
    ready(&mut g);
    rounds(&mut g);
    assert!(
        g.handle(&payload_packet(id(&g), "FRServer_GameWon\t900\t0").unwrap())
            .is_err()
    );
    for kind in 4..8 {
        send(&mut g, &format!("FRServer_StatUpdate\t-1\t{kind}\t7"));
    }
    let before = g.state().clone();
    assert!(
        g.handle(&payload_packet(id(&g), "FRServer_GameWon\t900\t0").unwrap())
            .is_err()
    );
    assert_eq!(&before, g.state());
    assert_eq!(g.state().completed, 0);
}

#[test]
fn loss_close_and_stale_attempt_never_grant() {
    let f = Fixture::new();
    let mut g = f.load(None);
    ready(&mut g);
    let old = id(&g);
    let lost = send(&mut g, "FRServer_GameLost\tundefined");
    assert!(lost.transaction.is_none());
    assert_eq!(g.state().active.as_ref().unwrap().phase, CookingPhase::Lost);
    assert!(
        g.handle(&payload_packet(old, "FRServer_GameWon\t900\t0").unwrap())
            .is_err()
    );
    assert!(send(&mut g, "FRServer_GameClose").changed);
    assert!(!g.active());
    assert_eq!(g.state().completed, 0);
    ready(&mut g);
    assert!(
        !g.handle(&payload_packet(old, "FRServer_GameWon\t900\t0").unwrap())
            .unwrap()
            .changed
    );
}

#[test]
fn native_results_wait_for_actual_close_ack_and_replays_preserve_recipe() {
    let f = Fixture::new();
    let mut g = f.load(None);
    ready(&mut g);
    let active = id(&g);
    let ack = crafting_results::ack(active);
    let before = g.state().clone();
    assert!(!g.handle(&ack).unwrap().changed);
    assert_eq!(g.state(), &before);
    rounds(&mut g);
    totals(&mut g);
    let won = send(&mut g, "FRServer_GameWon\t5199\t0");
    assert_eq!(
        won.replies.iter().map(|p| p[2]).collect::<Vec<_>>(),
        [14, 18, 47, 19]
    );
    let terminal = g.state().clone();
    for (offset, value) in [(3, active), (3, active - 1), (7, 0), (11, 46), (11, 77)] {
        let mut altered = ack.clone();
        altered[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        let out = g.handle(&altered).unwrap();
        assert!(!out.changed && out.replies.is_empty() && out.transaction.is_none());
        assert_eq!(g.state(), &terminal);
    }
    for bad in [&ack[..14], &[ack.as_slice(), &[0]].concat()] {
        assert!(g.handle(bad).is_err());
    }
    for text in [
        "FRServer_GameOver",
        "FRServer_GameWon\t5199\t0",
        "FRServer_GameLost\tundefined",
    ] {
        let out = send(&mut g, text);
        assert!(out.replies.is_empty() && out.transaction.is_none());
        assert_eq!(g.state(), &terminal);
    }
    let mut restored = f.load(Some(terminal));
    assert!(!restored.handle(&ack).unwrap().changed);
    assert!(
        send(&mut restored, "FRServer_GameWon\t5199\t0")
            .transaction
            .is_none()
    );
    let close = send(&mut g, "FRServer_GameClose");
    assert!(close.changed && close.replies.is_empty() && close.transaction.is_none());
    assert!(!g.active());
    assert_eq!(g.state().completed, 1);
    assert!(!g.handle(&ack).unwrap().changed);
}

#[test]
fn content_and_saved_state_reject_unlabelled_or_inconsistent_recipe() {
    let mut f = Fixture::new();
    f.content["reconstructed_phase_values"] = json!(false);
    f.write();
    assert!(Cooking::load(&f.path, None).is_err());
    f.content["reconstructed_phase_values"] = json!(true);
    f.content["recipe"]["key"] = json!("chef.unknown");
    f.write();
    assert!(Cooking::load(&f.path, None).is_err());
    f.content["recipe"]["key"] = json!("chef.strong_arm_stew.sample");
    f.write();
    let mut g = f.load(None);
    ready(&mut g);
    rounds(&mut g);
    totals(&mut g);
    send(&mut g, "FRServer_GameWon\t900\t0");
    let mut state = g.state().clone();
    state.active.as_mut().unwrap().game_stats[1] = Some(900.0);
    assert!(Cooking::load(&f.path, Some(state)).is_err());
    let mut state = g.state().clone();
    state.active.as_mut().unwrap().round_stats[3][3] = None;
    assert!(Cooking::load(&f.path, Some(state)).is_err());
}

#[test]
#[ignore = "requires private original movie and prepared cooking content"]
fn private_original_movie_phase_configuration_loads() {
    let path = std::env::var_os("OSFR_COOKING_REFERENCE").expect("set OSFR_COOKING_REFERENCE");
    let mut g = Cooking::load(std::path::Path::new(&path), None).unwrap();
    ready(&mut g);
    rounds(&mut g);
    totals(&mut g);
    assert_eq!(
        send(&mut g, "FRServer_GameWon\t900\t0")
            .transaction
            .unwrap()
            .grants,
        vec![(35, 1)]
    );
}

#[test]
fn every_cooking_catalog_recipe_uses_exact_transaction_once() {
    let catalog: Value =
        serde_json::from_str(include_str!("../data/content/recipes.json")).unwrap();
    let recipes = catalog["records"].as_array().unwrap();
    let mut tested = 0;
    for row in recipes.iter().filter(|row| row["job"] == "chef") {
        let key = row["key"].as_str().unwrap();
        let available: Vec<_> = row["ingredients"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| {
                (
                    item["definition_id"].as_i64().unwrap() as i32,
                    item["count"].as_u64().unwrap() as u32,
                )
            })
            .collect();
        let mut fixture = Fixture::new();
        fixture.content["recipe"] = json!({"key": key});
        fixture.write();
        let mut game = fixture.load(None);
        let offered = game.begin_recipe(45, key, 20, true, &available).unwrap();
        let instance = id(&game);
        let expected_game = row["minigame_id"].as_i64().unwrap_or(46) as i32;
        assert_eq!(
            i32::from_le_bytes(offered.replies[0][11..15].try_into().unwrap()),
            expected_game,
            "{key}"
        );
        let mut start_request = vec![39, 0, 5];
        for value in [instance, -1, expected_game] {
            start_request.extend(value.to_le_bytes());
        }
        game.handle(&start_request).unwrap();
        send(&mut game, "FRServer_ReadyForGameData");
        rounds(&mut game);
        totals(&mut game);
        let won = send(&mut game, "FRServer_GameWon\t900\t0");
        let transaction = won.transaction.unwrap();
        assert_eq!(transaction.consumes, available, "{key}");
        assert_eq!(
            transaction.grants,
            vec![(
                row["output"]["definition_id"].as_i64().unwrap() as i32,
                row["output"]["count"].as_u64().unwrap() as u32,
            )],
            "{key}"
        );
        assert_eq!(
            transaction.experience_points,
            row["experience_points"].as_u64().unwrap() as u32,
            "{key}"
        );
        assert!(
            send(&mut game, "FRServer_GameWon\t900\t0")
                .transaction
                .is_none(),
            "{key}"
        );
        assert_eq!(game.state().completed, 1, "{key}");
        tested += 1;
    }
    assert_eq!(tested, 26);
}

#[test]
fn cooking_recipe_rejections_leave_state_unchanged() {
    let fixture = Fixture::new();
    let mut game = fixture.load(None);
    let state = game.state().clone();
    let key = "chef.strong_arm_stew.sample";
    let available = game.requirements().to_vec();
    let mut missing = available.clone();
    missing[0].1 -= 1;
    assert!(game.begin_recipe(45, key, 1, true, &missing).is_err());
    assert_eq!(game.state(), &state);
    assert!(game.begin_recipe(45, key, 0, true, &available).is_err());
    assert_eq!(game.state(), &state);
    assert!(game.begin_recipe(45, key, 1, false, &available).is_err());
    assert_eq!(game.state(), &state);
    assert!(
        game.begin_recipe(45, "chef.unknown", 20, true, &available)
            .is_err()
    );
    assert_eq!(game.state(), &state);
}
