#[path = "common/crafting_results.rs"]
mod crafting_results;
use osfr_tools::smelting::{self, payload_packet, Smelting, SmeltingPhase, SmeltingState};
use serde_json::{json, Value};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    path: PathBuf,
    content: Value,
}
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-smelting-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        let data = [
            [1, 5, 1, 1, 0, 20, 2, 3, 50, 100, 200],
            [65, 8, 3, 100, 100, 0, 0, 0, 0, 0, 0],
            [100, 1, 500, 0, 0, 0, 0, 0, 0, 0, 0],
            [2, 500, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ];
        let content = json!({"schema_version":1,"game_id":77,"profile_id":14,
            "name_id":501,"description_id":502,"icon_id":503,
            "reconstructed_phase_values":true,"phase_notice":"synthetic explicitly reconstructed test configuration",
            "phases":data.iter().enumerate().map(|(i,d)| json!({"id":i+1,"sequence":i+1,"duration":90,
                "objective_id":0,"start_id":0,"end_id":0,"stage_type":101+i,"data":d})).collect::<Vec<_>>(),
            "recipe":{"key":"miner.copper_bar.sample"}});
        let f = Self { path, content };
        f.write();
        f
    }
    fn write(&self) {
        fs::write(
            self.path.join("smelting.json"),
            serde_json::to_vec(&self.content).unwrap(),
        )
        .unwrap();
        fs::write(
            self.path.join("recipes.json"),
            include_str!("../data/content/recipes.json"),
        )
        .unwrap();
    }
    fn load(&self, state: Option<SmeltingState>) -> Smelting {
        Smelting::load(&self.path, state).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.path.join("smelting.json"));
        let _ = fs::remove_file(self.path.join("recipes.json"));
        let _ = fs::remove_dir(&self.path);
    }
}
fn id(g: &Smelting) -> i32 {
    g.state().active.as_ref().unwrap().instance_id
}
fn send(g: &mut Smelting, text: &str) -> smelting::SmeltingOutcome {
    let mut packet = payload_packet(id(g), text).unwrap();
    // Existing synthetic paths send the initial data request before launch;
    // native payloads use GameId=-1 once the movie is playing.
    if g.state().active.as_ref().unwrap().phase != SmeltingPhase::Offered {
        packet[11..15].copy_from_slice(&(-1_i32).to_le_bytes());
    }
    g.handle(&packet).unwrap()
}

#[test]
fn shared_native_go_sentinel_is_bound_to_offered_smelting_instance() {
    let fixture = Fixture::new();
    let mut game = fixture.load(None);
    game.begin(14).unwrap();
    let mut native = vec![39, 0, 5];
    for value in [id(&game), -1, -1] {
        native.extend(value.to_le_bytes());
    }
    assert_eq!(native, native_vector("smelting_start"));
    let offered = game.state().clone();
    for (offset, value) in [(3, id(&game) + 1), (7, 0), (11, 0), (11, 46), (11, -2)] {
        let mut wrong = native.clone();
        wrong[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        assert!(game.handle(&wrong).unwrap().replies.is_empty());
        assert_eq!(game.state(), &offered);
    }
    let mut payload = payload_packet(id(&game), "FRServer_ReadyForGameData").unwrap();
    payload[11..15].copy_from_slice(&(-1i32).to_le_bytes());
    assert_eq!(payload, native_vector("smelting_ready"));
    assert!(game.handle(&payload).unwrap().replies.is_empty());
    assert_eq!(game.state(), &offered);
    let launched = game.handle(&native).unwrap();
    assert_eq!(launched.replies, vec![vector("start"), vector("launch")]);
    assert_eq!(
        game.state().active.as_ref().unwrap().phase,
        SmeltingPhase::Playing
    );
    let playing = game.state().clone();
    assert!(game.handle(&native).unwrap().replies.is_empty());
    assert_eq!(game.state(), &playing);
    for (offset, value) in [(3, id(&game) + 1), (7, 0), (11, 0), (11, 46), (11, -2)] {
        let mut wrong = payload.clone();
        wrong[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        assert!(game.handle(&wrong).unwrap().replies.is_empty());
        assert_eq!(game.state(), &playing);
    }
    let mut malformed = payload.clone();
    malformed.pop();
    assert!(game.handle(&malformed).is_err());
    assert_eq!(game.state(), &playing);
    assert_eq!(game.handle(&payload).unwrap().replies.len(), 6);
    let supplied = game.state().clone();
    assert!(game.handle(&payload).unwrap().replies.is_empty());
    assert_eq!(game.state(), &supplied);
    game.cancel();
    game.begin(14).unwrap();
    let next = game.state().clone();
    assert!(game.handle(&native).unwrap().replies.is_empty());
    assert_eq!(game.state(), &next);
}
fn rounds(g: &mut Smelting) {
    for stage in 101..=104 {
        for stat in 0..=3 {
            send(g, &format!("FRServer_StatUpdate\t{stage}\t{stat}\t1"));
        }
    }
}

#[test]
fn native_results_ack_never_closes_or_repeats_recipe() {
    let f = Fixture::new();
    let mut g = f.load(None);
    g.begin(14).unwrap();
    g.handle(&native_vector("smelting_start")).unwrap();
    send(&mut g, "FRServer_ReadyForGameData");
    let active = id(&g);
    let ack = crafting_results::ack(active);
    let playing = g.state().clone();
    assert!(!g.handle(&ack).unwrap().changed);
    assert_eq!(g.state(), &playing);
    rounds(&mut g);
    let won = send(&mut g, "FRServer_GameWon\t900\t1");
    assert_eq!(
        won.replies.iter().map(|p| p[2]).collect::<Vec<_>>(),
        [14, 18, 47, 19]
    );
    let terminal = g.state().clone();
    for (offset, value) in [(3, active), (3, active - 1), (7, 0), (11, 77), (11, 46)] {
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
        "FRServer_GameWon\t900\t10",
        "FRServer_GameLost\tundefined",
        "FRServer_ScoreSet\t901",
    ] {
        let out = send(&mut g, text);
        assert!(out.replies.is_empty() && out.transaction.is_none());
        assert_eq!(g.state(), &terminal);
    }
    let mut restored = f.load(Some(terminal));
    assert!(!restored.handle(&ack).unwrap().changed);
    assert!(send(&mut restored, "FRServer_GameWon\t900\t10")
        .transaction
        .is_none());
    let close = send(&mut g, "FRServer_GameClose");
    assert!(close.changed && close.replies.is_empty() && close.transaction.is_none());
    assert!(!g.active());
    assert_eq!(g.state().completed, 1);
}

#[test]
fn shared_original_stat_suffix_is_narrow_and_never_completes_a_recipe() {
    let f = Fixture::new();
    let mut g = f.load(None);
    g.begin(14).unwrap();
    g.handle(&native_vector("smelting_start")).unwrap();
    send(&mut g, "FRServer_ReadyForGameData");
    let out = send(&mut g, "FRServer_StatUpdate\t101\t0\t33)");
    assert!(out.changed && out.transaction.is_none() && !out.completed);
    assert_eq!(
        g.state().active.as_ref().unwrap().round_stats[0][0],
        Some(33.0)
    );
    let before = g.state().clone();
    for message in [
        "FRServer_StatUpdate\t101\t0\t33))",
        "FRServer_StatUpdate\t101\t0\t)",
        "FRServer_StatUpdate\t101\t0\t(33)",
        "FRServer_StatUpdate\t101\t0\t33)junk",
        "FRServer_StatUpdate\t101)\t0\t33)",
        "FRServer_StatUpdate\t101\t0)\t33)",
        "FRServer_StatUpdate\t101\t0\t0.5)",
        "FRServer_StatUpdate\t101\t3\t3601)",
        "FRServer_StatUpdate\t101\t0\tNaN)",
        "FRServer_StatUpdate\t101\t0\tinf)",
        "FRServer_StatUpdate\t102\t0\t1)",
        "FRServer_StatUpdate\t-1\t4\t99)",
        "FRServer_StopLevel\t4\t1))",
        "FRServer_StopLevel\t4)\t1)",
        "FRServer_StopLevel\t3\t1)",
        "FRServer_StopLevel\t4\t2)",
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
    for message in ["FRServer_StopLevel\t4\t1)", "FRServer_StopLevel\t4\t0"] {
        let out = send(&mut g, message);
        assert!(!out.changed && out.transaction.is_none() && !out.completed);
    }
    assert!(send(&mut g, "FRServer_ScoreSet\t800").transaction.is_none());
    assert_eq!(g.state().completed, 0);
}
fn vector(name: &str) -> Vec<u8> {
    let v: Value =
        serde_json::from_str(include_str!("fixtures/smelting-community-vectors.json")).unwrap();
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
fn native_payload_info_and_game_over_equal_compiled_community_vectors() {
    let f = Fixture::new();
    let mut g = f.load(None);
    assert_eq!(g.begin(14).unwrap().replies[0], vector("info"));
    let mut start = vec![39, 0, 5];
    for n in [id(&g), -1, 77] {
        start.extend(n.to_le_bytes());
    }
    let started = g.handle(&start).unwrap();
    assert_eq!(started.replies[0], vector("start"));
    assert_eq!(started.replies[1], vector("launch"));
    assert_eq!(g.handle(&start).unwrap().replies.len(), 1);
    assert_eq!(
        payload_packet(id(&g), "FRServer_ReadyForGameData").unwrap(),
        vector("payload")
    );
    send(&mut g, "FRServer_ReadyForGameData");
    rounds(&mut g);
    let won = send(&mut g, "FRServer_GameWon\t900\t10");
    assert_eq!(
        &won.replies[1..],
        &[
            crafting_results::vector("smelting_won"),
            crafting_results::vector("smelting_scores"),
            crafting_results::vector("smelting_remove")
        ]
    );
    assert!(g.cancel().replies.is_empty());
}

#[test]
fn four_round_win_is_single_server_recipe_intent_and_survives_restart() {
    let f = Fixture::new();
    let mut g = f.load(None);
    assert!(g.begin(43).is_err());
    assert!(g.state().active.is_none());
    g.begin(14).unwrap();
    assert!(g.begin(14).is_err());
    assert_eq!(g.requirements(), &[(2739, 5), (879, 3)]);
    let data = send(&mut g, "FRServer_ReadyForGameData");
    assert_eq!(data.replies.len(), 6);
    for (i, b) in data.replies[1..5].iter().enumerate() {
        let s = std::str::from_utf8(&b[19..b.len() - 1]).unwrap();
        assert!(s.starts_with("OnPhaseDataMsg\t"));
        let cells: Vec<_> = s
            .split('\t')
            .nth(1)
            .unwrap()
            .trim_end_matches('^')
            .split('^')
            .collect();
        assert_eq!(cells.len(), 19);
        assert_eq!(cells[2], (i + 1).to_string());
    }
    rounds(&mut g);
    // Client quantity is deliberately NOT the granted quantity.
    let won = send(&mut g, "FRServer_GameWon\t900\t1");
    let transaction = won.transaction.unwrap();
    // Client tutorial locale 1772250646 says five Copper Bars, superseding the old local ten-bar value.
    assert_eq!(transaction.grants, vec![(21785, 5)]);
    assert_eq!(transaction.consumes, vec![(2739, 5), (879, 3)]);
    assert!(won.completed);
    let saved: SmeltingState =
        serde_json::from_slice(&serde_json::to_vec(g.state()).unwrap()).unwrap();
    let mut restored = f.load(Some(saved));
    assert_eq!(restored.state().completed, 1);
    assert!(send(&mut restored, "FRServer_GameWon\t900\t10")
        .transaction
        .is_none());
    assert_eq!(restored.state().completed, 1);
    let previous = id(&restored);
    restored.cancel();
    restored.begin(14).unwrap();
    assert_ne!(previous, id(&restored));
}

#[test]
fn malformed_stale_and_early_victory_do_not_commit() {
    let f = Fixture::new();
    let mut g = f.load(None);
    g.begin(14).unwrap();
    send(&mut g, "FRServer_ReadyForGameData");
    let old = g.state().clone();
    for text in [
        "FRServer_GameWon\t1\t10",
        "FRServer_StatUpdate\t104\t0\t1",
        "FRServer_StatUpdate\t101\t0\tNaN",
        "FRServer_StatUpdate\t101\t0\t1.5",
        "FRServer_StatUpdate\t101\t3\t3601",
        "FRServer_Pause\tmaybe",
        "FRServer_GameWon\t1\t11",
    ] {
        assert!(
            g.handle(&payload_packet(id(&g), text).unwrap()).is_err(),
            "{text}"
        );
        assert_eq!(g.state(), &old);
    }
    let valid = payload_packet(id(&g), "FRServer_ReadyForGameData").unwrap();
    for end in 2..valid.len() {
        assert!(g.handle(&valid[..end]).is_err());
        assert_eq!(g.state(), &old);
    }
    let mut bad = valid.clone();
    bad.push(0);
    assert!(g.handle(&bad).is_err());
    let mut bad = valid.clone();
    bad[19] = 0;
    assert!(g.handle(&bad).is_err());
    let stale = payload_packet(id(&g) - 1, "FRServer_GameWon\t1\t10").unwrap();
    assert!(!g.handle(&stale).unwrap().changed);
    assert_eq!(g.state(), &old);
    assert!(payload_packet(id(&g), "x".repeat(4096).as_str()).is_err());
}

#[test]
fn cancelling_or_losing_never_grants_and_does_not_reuse_instance() {
    let f = Fixture::new();
    let mut g = f.load(None);
    g.begin(14).unwrap();
    let previous = id(&g);
    send(&mut g, "FRServer_ReadyForGameData");
    assert!(send(&mut g, "FRServer_GameLost\tundefined")
        .transaction
        .is_none());
    assert_eq!(
        g.state().active.as_ref().unwrap().phase,
        SmeltingPhase::Lost
    );
    send(&mut g, "FRServer_GameClose");
    assert!(!g.active());
    assert_eq!(g.state().completed, 0);
    g.begin(14).unwrap();
    assert_ne!(previous, id(&g));
    assert!(g.cancel().changed);
    assert!(!g.cancel().changed);
}

#[test]
fn staged_completion_can_roll_back_and_duplicate_data_does_not_reset_stopwatch() {
    let f = Fixture::new();
    let mut committed = f.load(None);
    committed.begin(14).unwrap();
    send(&mut committed, "FRServer_ReadyForGameData");
    // The original timer counts up; the row's nominal90 is not a deadline.
    send(&mut committed, "FRServer_StatUpdate\t101\t3\t120");
    let before = committed.state().clone();
    let duplicate = send(&mut committed, "FRServer_ReadyForGameData");
    assert!(duplicate.replies.is_empty());
    assert!(!duplicate.changed);
    assert_eq!(committed.state(), &before);

    // Dropping a staged service models failed inventory/persistence commit.
    let mut staged = committed.clone();
    rounds(&mut staged);
    assert!(send(&mut staged, "FRServer_GameWon\t100\t10")
        .transaction
        .is_some());
    assert_eq!(committed.state(), &before);
    assert_eq!(committed.state().completed, 0);
}

#[test]
fn unsafe_content_and_corrupt_state_are_rejected_without_reset() {
    let mut f = Fixture::new();
    let original = f.content.clone();
    for (key, value) in [
        ("schema_version", json!(2)),
        ("reconstructed_phase_values", json!(false)),
        ("phase_notice", json!("")),
    ] {
        f.content = original.clone();
        f.content[key] = value;
        f.write();
        assert!(Smelting::load(&f.path, None).is_err());
    }
    for (stage, field, value) in [
        (0, "duration", json!(0)),
        (0, "stage_type", json!(104)),
        (2, "data", json!([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])),
    ] {
        f.content = original.clone();
        f.content["phases"][stage][field] = value;
        f.write();
        assert!(Smelting::load(&f.path, None).is_err());
    }
    f.content = original;
    f.write();
    let mut g = f.load(None);
    g.begin(14).unwrap();
    let mut s = g.state().clone();
    s.active.as_mut().unwrap().phase = SmeltingPhase::Won;
    assert!(Smelting::load(&f.path, Some(s)).is_err());
    let mut s = g.state().clone();
    s.active.as_mut().unwrap().round_stats[0][0] = Some(f64::INFINITY);
    assert!(Smelting::validate_state(&s).is_err());
    send(&mut g, "FRServer_ReadyForGameData");
    for (kind, value) in [(0, 1.5), (3, 3601.0)] {
        let mut s = g.state().clone();
        s.active.as_mut().unwrap().round_stats[0][kind] = Some(value);
        assert!(Smelting::validate_state(&s).is_err());
    }
    let mut s = g.state().clone();
    s.active.as_mut().unwrap().game_stats[4] = Some(1.0);
    assert!(Smelting::validate_state(&s).is_err());
    let mut s = g.state().clone();
    s.next_instance_id = i32::MAX;
    let mut exhausted = f.load(Some(s));
    exhausted.cancel();
    assert!(exhausted.begin(14).is_err());
    fs::write(f.path.join("smelting.json"), vec![b' '; 32769]).unwrap();
    assert!(Smelting::load(&f.path, None).is_err());
}

#[test]
#[ignore = "requires opt-in private prepared Smelting77 config, no client launch"]
fn private_original_movie_metadata_and_recipe_load() {
    let path = std::env::var_os("OSFR_SMELTING_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab/references/rust-opening")
        });
    let mut g = Smelting::load(&path, None).unwrap();
    let init = g.begin(14).unwrap();
    assert_eq!(init.replies[0].len(), 299);
    assert_eq!(
        i32::from_le_bytes(init.replies[0][15..19].try_into().unwrap()),
        386068
    );
    send(&mut g, "FRServer_ReadyForGameData");
    rounds(&mut g);
    assert!(send(&mut g, "FRServer_GameWon\t1000\t10").completed);
}

#[test]
fn every_miner_catalog_recipe_uses_exact_transaction_once() {
    let catalog: Value =
        serde_json::from_str(include_str!("../data/content/recipes.json")).unwrap();
    let recipes = catalog["records"].as_array().unwrap();
    let mut tested = 0;
    for row in recipes.iter().filter(|row| row["job"] == "miner") {
        let key = row["key"].as_str().unwrap();
        let game_id = row["minigame_id"].as_i64().unwrap() as i32;
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
        let offered = game.begin_recipe(14, key, 20, true, &available).unwrap();
        let instance = id(&game);
        assert_eq!(
            i32::from_le_bytes(offered.replies[0][11..15].try_into().unwrap()),
            game_id
        );
        let mut start_request = vec![39, 0, 5];
        for value in [instance, -1, -1] {
            start_request.extend(value.to_le_bytes());
        }
        game.handle(&start_request).unwrap();
        let data = send(&mut game, "FRServer_ReadyForGameData");
        assert_eq!(
            i32::from_le_bytes(data.replies[0][11..15].try_into().unwrap()),
            game_id
        );
        rounds(&mut game);
        let won = send(&mut game, "FRServer_GameWon\t900\t1");
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
            send(&mut game, "FRServer_GameWon\t900\t1")
                .transaction
                .is_none(),
            "{key}"
        );
        assert_eq!(game.state().completed, 1, "{key}");
        tested += 1;
    }
    assert_eq!(tested, 5);
}

#[test]
fn smelting_recipe_rejections_leave_state_unchanged() {
    let fixture = Fixture::new();
    let mut game = fixture.load(None);
    let state = game.state().clone();
    let key = "miner.copper_bar.sample";
    let available = game.requirements().to_vec();
    let mut missing = available.clone();
    missing[0].1 -= 1;
    assert!(game.begin_recipe(14, key, 1, true, &missing).is_err());
    assert_eq!(game.state(), &state);
    assert!(game.begin_recipe(14, key, 1, false, &available).is_err());
    assert_eq!(game.state(), &state);
    assert!(game
        .begin_recipe(14, "miner.unknown", 20, true, &available)
        .is_err());
    assert_eq!(game.state(), &state);
}
