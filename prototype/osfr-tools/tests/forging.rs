// Independent inclusion proves the isolated module before shared integration.
#[path = "common/crafting_results.rs"]
mod crafting_results;
use osfr_tools::forging::{payload_packet, Forging, ForgingOutcome, ForgingPhase, ForgingState};
use serde_json::{json, Value};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    directory: PathBuf,
    content: Value,
}
impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-forging-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        let content = json!({"schema_version":1,"game_id":139,"profile_id":16,"name_id":501,"description_id":502,"icon_id":503,
            "reconstructed_rules":true,"rule_notice":"Explicit synthetic reconstructed test board, consumption and tint.",
            "board":{"left":3,"top":3,"width":8,"height":8,"duration_seconds":180,"quota":1,"selectable_types":[1,2,3,4,5],"collectable_percent":10,"seed":139,"mold_id":6},
            "recipe":{"key":"blacksmith.spunky_scrapper_hammer.sample","reconstructed_consumption":true,"reconstructed_tint_selection":true}});
        let f = Self { directory, content };
        f.write();
        f
    }
    fn write(&self) {
        fs::write(
            self.directory.join("forging.json"),
            serde_json::to_vec(&self.content).unwrap(),
        )
        .unwrap();
        fs::write(
            self.directory.join("recipes.json"),
            include_str!("../data/content/recipes.json"),
        )
        .unwrap();
    }
    fn load(&self, state: Option<ForgingState>) -> Forging {
        Forging::load(&self.directory, state).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.directory.join("forging.json"));
        let _ = fs::remove_file(self.directory.join("recipes.json"));
        let _ = fs::remove_dir(&self.directory);
    }
}
fn id(g: &Forging) -> i32 {
    g.state().active.as_ref().unwrap().instance_id
}
fn send(g: &mut Forging, s: &str) -> ForgingOutcome {
    g.handle(&payload_packet(id(g), s).unwrap()).unwrap()
}
fn request_start(g: &mut Forging) -> ForgingOutcome {
    let mut p = payload_packet(id(g), "OnConnectMsg").unwrap();
    p.truncate(15);
    p[2] = 5;
    g.handle(&p).unwrap()
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
fn shared_native_go_sentinel_is_bound_to_offered_forging_instance() {
    let fixture = Fixture::new();
    let mut game = fixture.load(None);
    game.begin(16).unwrap();
    let mut native = vec![39, 0, 5];
    for value in [id(&game), -1, -1] {
        native.extend(value.to_le_bytes());
    }
    assert_eq!(native, native_vector("forging_start"));
    let offered = game.state().clone();
    for (offset, value) in [(3, id(&game) + 1), (7, 0), (11, 0), (11, 46), (11, -2)] {
        let mut wrong = native.clone();
        wrong[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        assert!(game.handle(&wrong).unwrap().replies.is_empty());
        assert_eq!(game.state(), &offered);
    }
    let mut payload = payload_packet(id(&game), "OnConnectMsg").unwrap();
    payload[11..15].copy_from_slice(&(-1i32).to_le_bytes());
    assert_eq!(payload, native_vector("forging_connect"));
    assert!(game.handle(&payload).unwrap().replies.is_empty());
    assert_eq!(game.state(), &offered);
    let launched = game.handle(&native).unwrap();
    assert_eq!(launched.replies.len(), 2);
    assert_eq!(launched.replies[0], vector("start"));
    assert_eq!(launched.replies[1], vector("launch"));
    assert_eq!(
        game.state().active.as_ref().unwrap().phase,
        ForgingPhase::Launched
    );
    let state = game.state().clone();
    assert!(game.handle(&native).unwrap().replies.is_empty());
    for (offset, value) in [(3, id(&game) + 1), (7, 0), (11, 0), (11, 46), (11, -2)] {
        let mut wrong = payload.clone();
        wrong[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        assert!(game.handle(&wrong).unwrap().replies.is_empty());
        assert_eq!(game.state(), &state);
    }
    let mut malformed = payload.clone();
    malformed.pop();
    assert!(game.handle(&malformed).is_err());
    assert_eq!(game.state(), &state);
    assert_eq!(game.handle(&payload).unwrap().replies.len(), 1);
    let connected = game.state().clone();
    assert!(game.handle(&payload).unwrap().replies.is_empty());
    assert_eq!(game.state(), &connected);
    game.cancel();
    game.begin(16).unwrap();
    let next = game.state().clone();
    assert!(game.handle(&native).unwrap().replies.is_empty());
    assert_eq!(game.state(), &next);
}
fn playing(f: &Fixture) -> Forging {
    let mut g = f.load(None);
    g.begin(16).unwrap();
    request_start(&mut g);
    send(&mut g, "OnConnectMsg");
    send(&mut g, "OnStartGameMsg\t0");
    send(&mut g, "OnStartLevelMsg\t0");
    g
}
fn texts(o: &ForgingOutcome) -> Vec<String> {
    o.replies
        .iter()
        .filter(|p| p.len() >= 20 && p[0..3] == [39, 0, 14])
        .map(|p| String::from_utf8(p[19..p.len() - 1].to_vec()).unwrap())
        .collect()
}
fn vector(key: &str) -> Vec<u8> {
    let v: Value =
        serde_json::from_str(include_str!("fixtures/forging-community-vectors.json")).unwrap();
    v["vectors"][key]
        .as_str()
        .unwrap()
        .as_bytes()
        .chunks_exact(2)
        .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
        .collect()
}
fn known_board(f: &Fixture) -> Forging {
    let mut s = playing(f).state().clone();
    let a = s.active.as_mut().unwrap();
    let left = f.content["board"]["left"].as_u64().unwrap() as usize;
    let top = f.content["board"]["top"].as_u64().unwrap() as usize;
    let width = f.content["board"]["width"].as_u64().unwrap() as usize;
    let height = f.content["board"]["height"].as_u64().unwrap() as usize;
    for y in top..top + height {
        for x in left..left + width {
            a.board[y * 14 + x] = 401;
            a.idle[y * 14 + x] = true;
        }
    }
    a.tile_queue = vec![402; 70];
    f.load(Some(s))
}

#[test]
fn unused_generic_stat_and_stop_reports_cannot_change_authoritative_forging() {
    let f = Fixture::new();
    let mut g = playing(&f);
    let before = g.state().clone();
    // Copied SoeInterface definitions are not calls from the original board.
    for message in [
        "FRServer_StatUpdate\t1\t0\t33)",
        "FRServer_StatUpdate\t1\t0\t33",
        "FRServer_StopLevel\t1\t1)",
        "FRServer_StopLevel\t1\t1",
        "FRServer_ScoreSet\t800)",
    ] {
        assert!(
            g.handle(&payload_packet(id(&g), message).unwrap()).is_err(),
            "{message}"
        );
        assert_eq!(g.state(), &before, "{message}");
    }
    assert!(!send(&mut g, "FRServer_ScoreSet\t800").changed);
    assert_eq!(g.state(), &before);
}
fn play_board(g: &mut Forging, quota: u32) -> ForgingOutcome {
    // Actual native message semantics, with no edited played run/counters.
    for _ in 0..2000 {
        let a = g.state().active.as_ref().unwrap();
        if a.phase == ForgingPhase::LevelComplete {
            assert_eq!(a.collected, quota);
            return send(g, "OnNextLevelMsg");
        }
        assert_eq!(a.phase, ForgingPhase::Playing);
        let mut idle = String::from("OnIdleMsg\t");
        for y in 0..14 {
            for x in 0..14 {
                if (401..=405).contains(&a.board[y * 14 + x]) || a.board[y * 14 + x] == 606 {
                    idle += &format!("{x},{y},");
                }
            }
        }
        send(g, &idle);
        let a = g.state().active.as_ref().unwrap();
        if let Some((x, y)) = (0..13)
            .flat_map(|y| (0..14).map(move |x| (x, y)))
            .find(|&(x, y)| a.board[y * 14 + x] == 606 && a.board[(y + 1) * 14 + x] == 100)
        {
            send(g, &format!("OnLevelEventMsg\t50\t{x}\t{y}"));
            continue;
        }
        let mut found = None;
        'outer: for y in (1..13).rev() {
            for x in 1..13 {
                let v = a.board[y * 14 + x];
                if !(401..=405).contains(&v) {
                    continue;
                }
                for (x2, y2) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                    if a.board[y2 * 14 + x2] != v {
                        continue;
                    }
                    for (x3, y3) in [
                        (x2.saturating_sub(1), y2),
                        (x2 + 1, y2),
                        (x2, y2.saturating_sub(1)),
                        (x2, y2 + 1),
                    ] {
                        if x3 < 14 && y3 < 14 && (x3, y3) != (x, y) && a.board[y3 * 14 + x3] == v {
                            found = Some(format!("OnChainMsg\t{x},{y},{x2},{y2},{x3},{y3},"));
                            break 'outer;
                        }
                    }
                }
            }
        }
        send(
            g,
            &found.expect("server should reshuffle a board without a legal move"),
        );
    }
    panic!("bounded original board transcript did not finish");
}

fn before_drop(f: &Fixture) -> Forging {
    let mut s = known_board(f).state().clone();
    s.active.as_mut().unwrap().board[7 * 14 + 3] = 606;
    f.load(Some(s))
}

#[test]
fn native_header_info_payload_launch_and_finish_match_compiled_community() {
    let f = Fixture::new();
    let mut g = f.load(None);
    assert!(g.begin(43).is_err());
    let offer = g.begin(16).unwrap();
    assert_eq!(offer.replies, vec![vector("info")]);
    assert_eq!(
        payload_packet(id(&g), "OnConnectMsg").unwrap(),
        vector("payload")
    );
    let start = request_start(&mut g);
    assert_eq!(start.replies, vec![vector("start"), vector("launch")]);
    let state = g.state().clone();
    let duplicate = request_start(&mut g);
    assert_eq!(duplicate.replies, vec![vector("start")]);
    assert_eq!(g.state(), &state);
    assert_eq!(g.cancel().replies, vec![vector("remove")]);
    assert!(!g.active());
    let mut g = before_drop(&f);
    send(&mut g, "OnChainMsg\t3,8,3,9,3,10,");
    send(&mut g, "OnLevelEventMsg\t50\t3\t10");
    send(&mut g, "OnNextLevelMsg");
    let mut fixture_state = g.state().clone();
    let a = fixture_state.active.as_mut().unwrap();
    a.score = 900;
    a.remaining_ms = 174_000;
    a.wall_ms = 6000;
    let mut g = f.load(Some(fixture_state));
    assert_eq!(
        send(&mut g, "FRServer_GameOver").replies,
        vec![
            crafting_results::vector("forging_won"),
            crafting_results::vector("forging_scores"),
            crafting_results::vector("forging_remove")
        ]
    );
    assert!(send(&mut g, "FRServer_GameOver").replies.is_empty());
}

#[test]
fn native_results_ack_waits_for_actual_close_and_never_repeats_board_reward() {
    let f = Fixture::new();
    let mut g = before_drop(&f);
    let active = id(&g);
    let ack = crafting_results::ack(active);
    let playing = g.state().clone();
    assert!(!g.handle(&ack).unwrap().changed);
    assert_eq!(g.state(), &playing);
    send(&mut g, "OnChainMsg\t3,8,3,9,3,10,");
    send(&mut g, "OnLevelEventMsg\t50\t3\t10");
    assert!(send(&mut g, "OnNextLevelMsg").completed);
    let won = g.state().clone();
    assert!(!g.handle(&ack).unwrap().changed);
    assert_eq!(g.state(), &won);
    let presented = send(&mut g, "FRServer_GameOver");
    assert_eq!(
        presented.replies.iter().map(|p| p[2]).collect::<Vec<_>>(),
        [18, 47, 19]
    );
    let terminal = g.state().clone();
    for (offset, value) in [(3, active), (3, active - 1), (7, 0), (11, 139), (11, 46)] {
        let mut altered = ack.clone();
        altered[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        let out = g.handle(&altered).unwrap();
        assert!(!out.changed && out.replies.is_empty() && out.transaction.is_none());
        assert_eq!(g.state(), &terminal);
    }
    for bad in [&ack[..14], &[ack.as_slice(), &[0]].concat()] {
        assert!(g.handle(bad).is_err());
    }
    for text in ["FRServer_GameOver", "OnNextLevelMsg"] {
        let out = send(&mut g, text);
        assert!(out.replies.is_empty() && out.transaction.is_none());
        assert_eq!(g.state(), &terminal);
    }
    let mut restored = f.load(Some(terminal));
    assert!(!restored.handle(&ack).unwrap().changed);
    assert!(send(&mut restored, "OnNextLevelMsg").transaction.is_none());
    let close = send(&mut g, "FRServer_GameClose");
    assert!(close.changed && close.replies.is_empty() && close.transaction.is_none());
    assert!(!g.active());
    assert_eq!(g.state().completed, 1);
}

#[test]
fn original_handshake_initializes_grid_mappings_queues_and_mold_once() {
    let f = Fixture::new();
    let mut g = f.load(None);
    g.begin(16).unwrap();
    assert!(send(&mut g, "OnConnectMsg").replies.is_empty());
    request_start(&mut g);
    assert_eq!(
        texts(&send(&mut g, "OnConnectMsg")),
        vec!["OnServerReadyMsg"]
    );
    assert!(send(&mut g, "FRServer_Connect").replies.is_empty());
    assert_eq!(
        texts(&send(&mut g, "OnStartGameMsg\t0")),
        vec!["OnNextLevelMsg\t1\t0\t0"]
    );
    let o = send(&mut g, "OnStartLevelMsg\t0");
    let t = texts(&o);
    assert!(t.contains(&"OnTileInfoMsg\t606\t6".into()));
    assert!(t.contains(&"OnLevelEventMsg\t32\t6\t1".into()));
    let queues: Vec<_> = t
        .iter()
        .filter(|s| s.starts_with("OnTileQueueMsg"))
        .collect();
    assert_eq!(queues.len(), 2);
    assert_eq!(
        queues[0].split('\t').nth(2).unwrap().split(',').count(),
        196
    );
    assert_eq!(queues[1].split('\t').nth(2).unwrap().split(',').count(), 70);
    let map = t.iter().find(|s| s.starts_with("OnBuildMapMsg")).unwrap();
    assert!(map.starts_with("OnBuildMapMsg\t14\t14\t"));
    assert_eq!(g.state().active.as_ref().unwrap().board.len(), 196);
    assert_eq!(g.requirements(), [(21785, 10), (2752, 5), (55, 5), (56, 5)]);
    let prior = g.state().clone();
    assert!(send(&mut g, "OnStartLevelMsg\t0").replies.is_empty());
    assert_eq!(g.state(), &prior);
}

#[test]
fn chain_gravity_matches_right_to_left_bottom_to_top_queue_order() {
    let f = Fixture::new();
    let mut s = before_drop(&f).state().clone();
    let a = s.active.as_mut().unwrap();
    a.tile_queue[..3].copy_from_slice(&[402, 403, 404]);
    let mut g = f.load(Some(s));
    let o = send(&mut g, "OnChainMsg\t3,8,3,9,3,10,");
    assert!(o.transaction.is_none());
    let a = g.state().active.as_ref().unwrap();
    assert_eq!(
        [
            a.board[3 * 14 + 3],
            a.board[4 * 14 + 3],
            a.board[5 * 14 + 3]
        ],
        [404, 403, 402]
    );
    assert_eq!(a.board[10 * 14 + 3], 606);
    assert_eq!(a.tile_queue.len(), 67);
    assert_eq!(a.chains, 1);
    assert_eq!(a.score, 400);
    assert_eq!(a.remaining_ms, 186_000);
    assert!(texts(&o).contains(&"OnLevelEventMsg\t25\t6\t1".into()));
    let before = g.state().clone();
    assert!(g
        .handle(&payload_packet(id(&g), "OnChainMsg\t3,6,3,7,3,8,").unwrap())
        .is_err());
    assert_eq!(g.state(), &before);
    send(&mut g, "OnIdleMsg\t3,3,3,4,3,5,3,6,3,7,3,8,3,9,3,10,");
    send(&mut g, "OnChainMsg\t3,6,3,7,3,8,");
    // Repeated chain colors do not change the fixed recipe or trust any score.
    assert_eq!(g.state().active.as_ref().unwrap().chains, 2);
}

#[test]
fn quota_then_native_next_level_awards_fixed_recipe_once_even_after_reload() {
    let f = Fixture::new();
    let mut g = before_drop(&f);
    let before = g.state().clone();
    let spoof = send(&mut g, "FRServer_GameWon\t999999\t999");
    assert!(spoof.transaction.is_none());
    assert_eq!(g.state(), &before);
    assert!(g
        .handle(&payload_packet(id(&g), "OnLevelEventMsg\t50\t3\t7").unwrap())
        .is_err());
    assert_eq!(g.state(), &before);
    send(&mut g, "OnChainMsg\t3,8,3,9,3,10,");
    let drop = send(&mut g, "OnLevelEventMsg\t50\t3\t10");
    assert_eq!(
        g.state().active.as_ref().unwrap().phase,
        ForgingPhase::LevelComplete
    );
    assert!(drop.transaction.is_none());
    assert!(!drop.completed);
    assert!(texts(&drop).contains(&"OnStopLevel\t1\ttrue".into()));
    // GameClient.GameApplyTick returns while globally paused. Grid.Success is
    // empty, so the original next-level and GameOver callbacks require ticks.
    assert!(!texts(&drop).contains(&"OnActiveMsg\tfalse".into()));
    let saved: ForgingState =
        serde_json::from_slice(&serde_json::to_vec(g.state()).unwrap()).unwrap();
    let mut g = f.load(Some(saved));
    let won = send(&mut g, "OnNextLevelMsg");
    assert!(won.completed);
    assert_eq!(g.state().completed, 1);
    let tx = won.transaction.unwrap();
    assert_eq!(tx.consumes, [(21785, 10), (2752, 5), (55, 5), (56, 5)]);
    assert_eq!(tx.grants, [(30280, 1)]);
    let catalog: Value =
        serde_json::from_str(include_str!("../data/content/recipes.json")).unwrap();
    let table_row = catalog["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["key"] == "blacksmith.spunky_scrapper_hammer.sample")
        .unwrap();
    assert_eq!(
        tx.consumes,
        table_row["ingredients"]
            .as_array()
            .unwrap()
            .iter()
            .map(|ingredient| (
                ingredient["definition_id"].as_i64().unwrap() as i32,
                ingredient["count"].as_u64().unwrap() as u32,
            ))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        tx.grants[0].0,
        table_row["output"]["definition_id"].as_i64().unwrap() as i32
    );
    assert_eq!(
        tx.grants[0].1,
        table_row["output"]["count"].as_u64().unwrap() as u32
    );
    assert_eq!(
        tx.experience_points,
        table_row["experience_points"].as_u64().unwrap() as u32
    );
    assert_eq!(
        texts(&ForgingOutcome {
            replies: won.replies,
            ..Default::default()
        }),
        vec!["OnGameWonMsg"]
    );
    let saved = g.state().clone();
    let mut restored = f.load(Some(saved));
    assert!(send(&mut restored, "OnNextLevelMsg").transaction.is_none());
    assert!(send(&mut restored, "FRServer_GameWon\t1000000\t1000000")
        .transaction
        .is_none());
    assert_eq!(restored.state().completed, 1);
    let old_id = id(&restored);
    restored.cancel();
    restored.begin(16).unwrap();
    assert_ne!(id(&restored), old_id);
    let prior = restored.state().clone();
    assert!(
        !restored
            .handle(&payload_packet(old_id, "OnNextLevelMsg").unwrap())
            .unwrap()
            .changed
    );
    assert_eq!(restored.state(), &prior);
}

#[test]
fn forging_recipe_rejections_leave_state_unchanged() {
    let fixture = Fixture::new();
    let mut game = fixture.load(None);
    let state = game.state().clone();
    let key = "blacksmith.spunky_scrapper_hammer.sample";
    let available = game.requirements().to_vec();
    let mut missing = available.clone();
    missing[0].1 -= 1;
    assert!(game.begin_recipe(16, key, 1, true, &missing).is_err());
    assert_eq!(game.state(), &state);
    assert!(game.begin_recipe(16, key, 0, true, &available).is_err());
    assert_eq!(game.state(), &state);
    assert!(game.begin_recipe(16, key, 1, false, &available).is_err());
    assert_eq!(game.state(), &state);
    assert!(game
        .begin_recipe(16, "blacksmith.unknown", 20, true, &available)
        .is_err());
    assert_eq!(game.state(), &state);
}

#[test]
fn playable_generated_board_reaches_five_hammer_pieces_through_real_messages() {
    let mut f = Fixture::new();
    f.content["board"]["quota"] = json!(5);
    f.write();
    let mut g = playing(&f);
    let out = play_board(&mut g, 5);
    assert!(out.completed);
    assert_eq!(out.transaction.unwrap().grants, [(30280, 1)]);
    assert_eq!(g.state().completed, 1);
    assert!(g.state().active.as_ref().unwrap().chains > 0);
}

#[test]
fn malformed_chains_packets_and_claimed_grids_roll_back_all_state() {
    let f = Fixture::new();
    let mut g = known_board(&f);
    for s in [
        "OnChainMsg\t3,3,4,3,",
        "OnChainMsg\t3,3,4,4,5,5,",
        "OnChainMsg\t3,3,4,3,3,3,",
        "OnChainMsg\t13,3,12,3,11,3,",
        "OnChainMsg\t-1,3,4,3,5,3,",
        "OnChainMsg\t3,3,4,3,5,",
        "OnIdleMsg\t99,99,",
        "OnVerifyMsg\t3,3,402,",
        "OnLevelEventMsg\t51\t3\t3",
        "FRServer_Pause\tNaN",
    ] {
        let prior = g.state().clone();
        assert!(
            g.handle(&payload_packet(id(&g), s).unwrap()).is_err(),
            "{s}"
        );
        assert_eq!(g.state(), &prior);
    }
    let packet = payload_packet(id(&g), "OnChainMsg\t3,3,4,3,5,3,").unwrap();
    for n in 2..packet.len() {
        let prior = g.state().clone();
        assert!(g.handle(&packet[..n]).is_err(), "prefix{n}");
        assert_eq!(g.state(), &prior);
    }
    let mut extra = packet.clone();
    extra.push(0);
    assert!(g.handle(&extra).is_err());
    let mut no_nul = packet.clone();
    *no_nul.last_mut().unwrap() = b'x';
    assert!(g.handle(&no_nul).is_err());
    let mut enormous = packet;
    enormous[15..19].copy_from_slice(&i32::MAX.to_le_bytes());
    assert!(g.handle(&enormous).is_err());
    assert!(payload_packet(id(&g), "OnChainMsg\n3,3,").is_err());
    assert!(payload_packet(id(&g), &"a".repeat(4096)).is_err());
    let prior = g.state().clone();
    send(&mut g, "OnVerifyMsg\t3,3,401,4,3,401,");
    assert_eq!(g.state(), &prior);
}

#[test]
fn no_moves_reshuffles_through_original_clear_build_queue_protocol() {
    let mut f = Fixture::new();
    f.content["board"]["left"] = json!(1);
    f.content["board"]["top"] = json!(1);
    f.content["board"]["width"] = json!(3);
    f.content["board"]["height"] = json!(3);
    f.write();
    let mut s = known_board(&f).state().clone();
    let a = s.active.as_mut().unwrap();
    for (y, row) in [
        (1, [402, 403, 402]),
        (2, [403, 402, 403]),
        (3, [401, 401, 401]),
    ] {
        for (dx, v) in row.into_iter().enumerate() {
            a.board[y * 14 + dx + 1] = v;
        }
    }
    a.tile_queue[..3].copy_from_slice(&[403, 402, 403]);
    let mut g = f.load(Some(s));
    let out = send(&mut g, "OnChainMsg\t1,3,2,3,3,3,");
    let t = texts(&out);
    assert!(t.contains(&"OnLevelEventMsg\t11\t1".into()));
    assert!(t.iter().any(|s| s.starts_with("OnBuildMapMsg\t14\t14\t")));
    assert_eq!(g.state().active.as_ref().unwrap().reshuffles, 1);
    assert_eq!(g.state().active.as_ref().unwrap().tile_queue.len(), 70);
    assert!(out.transaction.is_none());
}

#[test]
fn pause_server_deadline_cancel_and_pending_queue_reconnect_never_award() {
    let f = Fixture::new();
    let mut g = known_board(&f);
    send(&mut g, "FRServer_Pause\ttrue");
    let remaining = g.state().active.as_ref().unwrap().remaining_ms;
    g.tick(5000).unwrap();
    assert_eq!(g.state().active.as_ref().unwrap().remaining_ms, remaining);
    send(&mut g, "FRServer_Pause\tfalse");
    g.tick(1000).unwrap();
    assert_eq!(
        g.state().active.as_ref().unwrap().remaining_ms,
        remaining - 1000
    );
    let loss = g.tick(u32::MAX).unwrap();
    assert!(loss.transaction.is_none());
    assert_eq!(g.state().active.as_ref().unwrap().phase, ForgingPhase::Lost);
    assert!(texts(&loss).contains(&"OnStopLevel\t1\tfalse".into()));
    assert!(!texts(&loss).contains(&"OnActiveMsg\tfalse".into()));
    assert!(send(&mut g, "OnNextLevelMsg").transaction.is_none());
    let saved = g.state().clone();
    let mut restored = f.load(Some(saved));
    let next = restored.state().next_instance_id;
    assert!(restored.cancel().transaction.is_none());
    assert_eq!(restored.state().completed, 0);
    restored.begin(16).unwrap();
    assert_eq!(id(&restored), next);
    assert!(restored.state().active.as_ref().unwrap().board.is_empty());
    assert!(send(&mut restored, "FRServer_GameClose")
        .transaction
        .is_none());
    assert!(!restored.active());
}

#[test]
fn content_and_persisted_bounds_fail_closed_with_explicit_reconstruction_labels() {
    let mut f = Fixture::new();
    let original = f.content.clone();
    for (path, value) in [
        (vec!["reconstructed_rules"], json!(false)),
        (vec!["board", "quota"], json!(0)),
        (vec!["board", "quota"], json!(101)),
        (vec!["board", "quota"], json!(3)),
        (vec!["board", "width"], json!(u64::MAX)),
        (vec!["board", "mold_id"], json!(999)),
        (vec!["board", "selectable_types"], json!([1, 1])),
        (vec!["recipe", "reconstructed_consumption"], json!(false)),
        (vec!["recipe", "key"], json!("blacksmith.unknown")),
    ] {
        f.content = original.clone();
        let mut target = &mut f.content;
        for k in &path[..path.len() - 1] {
            target = &mut target[*k];
        }
        target[path[path.len() - 1]] = value;
        f.write();
        assert!(Forging::load(&f.directory, None).is_err());
    }
    f.content = original;
    f.write();
    let saved = known_board(&f).state().clone();
    for bad in 0..7 {
        let mut s = saved.clone();
        let a = s.active.as_mut().unwrap();
        match bad {
            0 => a.board = vec![401; 197],
            1 => a.tile_queue = vec![401; 71],
            2 => a.board[0] = 606,
            3 => a.collected = 2,
            4 => a.rng = 0,
            5 => a.phase = ForgingPhase::Won,
            _ => a.board[3 * 14 + 3] = u16::MAX,
        };
        assert!(Forging::load(&f.directory, Some(s)).is_err());
    }
    fs::write(f.directory.join("forging.json"), vec![b' '; 32769]).unwrap();
    assert!(Forging::load(&f.directory, None).is_err());
}

#[test]
#[ignore = "requires private original Forging139 inputs via OSFR_OPENING_REFERENCE"]
fn private_original_content_movie_metadata_and_real_board_roundtrip() {
    let p =
        PathBuf::from(std::env::var_os("OSFR_OPENING_REFERENCE").expect("OSFR_OPENING_REFERENCE"));
    let mut g = Forging::load(&p, None).unwrap();
    let offer = g.begin(16).unwrap();
    assert_eq!(
        i32::from_le_bytes(offer.replies[0][15..19].try_into().unwrap()),
        386081
    );
    request_start(&mut g);
    send(&mut g, "OnConnectMsg");
    send(&mut g, "OnStartGameMsg\t0");
    let out = send(&mut g, "OnStartLevelMsg\t0");
    assert!(texts(&out).contains(&"OnLevelEventMsg\t32\t6\t5".into()));
    let s: ForgingState = serde_json::from_slice(&serde_json::to_vec(g.state()).unwrap()).unwrap();
    let restored = Forging::load(&p, Some(s)).unwrap();
    assert_eq!(g.state(), restored.state());
    assert!(send(&mut g, "FRServer_GameWon\t999\t999")
        .transaction
        .is_none());
    let result = play_board(&mut g, 5);
    assert!(result.completed);
    assert_eq!(g.state().completed, 1);
    assert!(g.state().active.as_ref().unwrap().chains > 0);
    let catalog: Value =
        serde_json::from_str(include_str!("../data/content/recipes.json")).unwrap();
    let recipe = catalog["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["key"] == "blacksmith.spunky_scrapper_hammer.sample")
        .unwrap();
    assert_eq!(
        result.transaction.unwrap().grants,
        [(
            recipe["output"]["definition_id"].as_i64().unwrap() as i32,
            recipe["output"]["count"].as_u64().unwrap() as u32
        )]
    );
    eprintln!(
        "forging139 private board: chains={} pieces={} reshuffles={} completed={}",
        g.state().active.as_ref().unwrap().chains,
        g.state().active.as_ref().unwrap().collected,
        g.state().active.as_ref().unwrap().reshuffles,
        g.state().completed
    );
    assert!(send(&mut g, "OnNextLevelMsg").transaction.is_none());
    let out = send(&mut g, "FRServer_GameOver");
    assert!(out.replies.iter().any(|p| p[0..3] == [39, 0, 18]));
    assert!(g.cancel().transaction.is_none());
}
