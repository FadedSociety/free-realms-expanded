//! W96/W106 opt-in measurements; no timing thresholds on shared hardware.
//! Run with --release --test load_players -- --ignored --nocapture --test-threads=1.
//!
//! Receipt 2026-10-16, Windows GNU release, shared host with concurrent builds.
//! Synthetic one-zone layouts, radius 100 m, cap 64; one full moving-player
//! sweep at each population, setup excluded. Microseconds per registry update:
//! N     spread old/grid       crowded old/grid
//! 16       20.006 / 18.113        43.656 / 85.831
//! 64       91.398 / 32.062       707.448 / 1445.120
//! 256     886.157 / 63.108      6205.189 / 19191.554
//! 1000  13515.359 / 174.502    71643.858 / 196971.673
//! Every ordered output matched the old algorithm, including the cap-64 load.
//! At N=1000 spread, cloning the registry alone averaged 1102.621 us.
//! Actual 16-player SharedWorld clone+drop: 12.215 us/message (1000 samples).
//! UDP gateway step: median 1172.000 us, max 1945.700 us (10 samples), exact
//! peer relay asserted; excludes drain/read timeouts, content/history and save I/O.
//!
//! W96 recommendation (historical, R): keep 16 / 4 per-IP and 100 m / 64 interest
//! defaults. The cap bounds fanout, not dense-cell CPU: local work remains O(k^2)
//! and this crowded run regressed. A grid is not 1000-player gateway acceptance.
//! At 16, preserve transactional staging; clone cost was about 1% of the step.
//! Before raising caps, skip SharedWorld clones for outcomes without world
//! changes and stage affected records for movement, preserving save rollback
//! and shared security budgets. Bound dense-cell work and run a content/saves
//! load test before raising admission. No new native facts were recovered;
//! players_reference.rs freezes this project's pre-W96 Rust algorithm.
//!
//! W106 threat model: an authenticated hostile mover can supply malformed poses,
//! churn radius/cutoff boundaries, or disconnect/rejoin. Pose, identity and
//! presentation validation still precede all index/cache changes. Cached views
//! hold only registry-owned ids and at most the existing interest cap per viewer;
//! leave removes the viewer's cache and repairs affected directed views.
//! Security-reviewer pass: scoped added-line scan plus manual validation/cache
//! lifetime review found no new unsafe/FFI, panic/unwrap, credentials or external
//! APIs. Semgrep/Gitleaks were unavailable. Residual CPU risk: coincident crowds
//! or many cutoff crossings can still force local rescans; admission is unchanged.
//! The differential canary fails when selected-mover cutoff re-ranking is disabled
//! (id 79, x=0.5); the source was restored byte-for-byte before the clean gates.
//!
//! W106 receipt 2026-10-16, same release layout/sweep and shared-host caveat:
//! N       spread W96/W106 us       crowded W96/W106 us
//! 16         18.113 / 9.194           85.831 / 7.231
//! 64         32.062 / 9.833         1445.120 / 46.386
//! 256        63.108 / 22.634       19191.554 / 244.430
//! 1000      174.502 / 20.910      196971.673 / 898.865
//! Historical W96 values are the receipt above, not simultaneous measurements.
//! Rerun pre-grid reference: crowded N=1000 61082.144 us/update. Every ordered
//! output matched. Crowded N=1000 registry clone+drop: 619.456 us (100 samples).
//! Logs: lab/build/w106/{registry-scaling,focused-tests,cutoff-canary-caught}.log
//! Clean release gates: players 11/0, multiplayer 11/0 (4 existing ignores),
//! differential 5/0 (including 96-player dense/coincident crowds), load 1/0.
//!
//! W106 recommendation (R): retain 16 total / 4 per-IP and 100 m / 64 interest.
//! Cached views and partial selection remove the measured dense-cell regression,
//! but cutoff churn retains quadratic worst cases. Full gateway queueing, saves,
//! content, transactional SharedWorld clones and long-running bot load were not
//! measured here. These registry-only results do not justify raising admission.
#![allow(unexpected_cfgs)] // w96_gateway_probe is used only by the isolated source probe below.

#[path = "players_reference.rs"]
mod old;

use osfr_tools::players::{InterestPolicy, PlayerRegistry, SessionId, ZoneInstance};
use std::{hint::black_box, time::Instant};

const ZONE: ZoneInstance = ZoneInstance {
    zone_id: 0,
    instance_id: 0,
};

fn position(id: usize, crowded: bool) -> [f32; 3] {
    let spacing = if crowded { 0.5 } else { 75.0 };
    [(id % 32) as f32 * spacing, 0.0, (id / 32) as f32 * spacing]
}

#[test]
#[ignore = "bounded scaling measurement; run in release mode on an idle machine"]
fn registry_scaling() {
    println!(
        "layout,N,updates,old_us_per_update,w96_grid_us_per_update,w106_us_per_update,w96_speedup,registry_clone_us"
    );
    for crowded in [false, true] {
        for (index, count) in [16, 64, 256, 1000].into_iter().enumerate() {
            // Historical W96 receipt above, not a second live timing on this host.
            let w96_us = if crowded {
                [85.831, 1445.120, 19191.554, 196971.673][index]
            } else {
                [18.113, 32.062, 63.108, 174.502][index]
            };
            let mut grid = PlayerRegistry::new(InterestPolicy::new(100.0, 64).unwrap());
            for id in 0..count {
                grid.join(
                    SessionId(id as u64),
                    10_000 + id as u64,
                    20_000 + id as u64,
                    ZONE,
                    position(id, crowded),
                    0.0,
                )
                .unwrap();
            }
            let mut old = old::PlayerRegistry::seeded(
                old::InterestPolicy::new(100.0, 64).unwrap(),
                (0..count).map(|id| grid.get(SessionId(id as u64)).unwrap().clone()),
            );
            // One complete sweep moves EVERY simulated player. Setup excluded;
            // alternate small moves preserve comparable local density and fanout.
            let moves: Vec<_> = (0..count)
                .map(|id| {
                    let mut p = position(id, crowded);
                    p[0] += 0.25;
                    (SessionId(id as u64), p)
                })
                .collect();
            let start = Instant::now();
            let expected: Vec<_> = moves
                .iter()
                .map(|&(id, p)| old.update_pose(id, ZONE, p, 0.5).unwrap())
                .collect();
            let old_us = start.elapsed().as_secs_f64() * 1e6 / count as f64;
            let start = Instant::now();
            let actual: Vec<_> = moves
                .iter()
                .map(|&(id, p)| grid.update_pose(id, ZONE, p, 0.5).unwrap())
                .collect();
            let grid_us = start.elapsed().as_secs_f64() * 1e6 / count as f64;
            assert_eq!(
                actual, expected,
                "load outputs differ, N={count} crowded={crowded}"
            );
            black_box((&actual, &expected));
            let start = Instant::now();
            for _ in 0..100 {
                black_box(grid.clone());
            }
            let clone_us = start.elapsed().as_secs_f64() * 1e6 / 100.0;
            println!(
                "{},{count},{count},{old_us:.3},{w96_us:.3},{grid_us:.3},{:.2},{clone_us:.3}",
                if crowded { "crowded" } else { "spread" },
                w96_us / grid_us
            );
        }
    }
}

// SharedWorld is private to the gateway binary. To time the REAL live world
// without modifying it or duplicating its fields/helpers, the opt-in parent
// builds a test-only snapshot inside W96's build directory. It embeds the
// CURRENT gateway and multiplayer.rs, adds one read-only timing helper, and
// compiles through scripts/cargo.ps1. No network listener outside loopback,
// native client, existing save, or production source is touched.
#[cfg(w96_gateway_probe)]
mod gateway_probe {
    include!(env!("OSFR_W96_GATEWAY_PROBE"));
}

#[test]
#[ignore = "builds and runs an isolated 16-client gateway/SharedWorld clone probe"]
#[cfg(not(w96_gateway_probe))]
fn gateway_sixteen_players() {
    use std::{fs, path::PathBuf, process::Command};
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project = crate_dir.parent().unwrap().parent().unwrap();
    let build = project.parent().unwrap().join("lab/build/w96");
    let probe_dir = build.join("gateway-probe");
    fs::create_dir_all(&probe_dir).unwrap();
    let trace = crate_dir
        .join("src/trace.rs")
        .display()
        .to_string()
        .replace('\\', "/");
    let bin_dir = crate_dir
        .join("src/bin")
        .display()
        .to_string()
        .replace('\\', "/");
    let studio = crate_dir
        .join("studio/index.html")
        .display()
        .to_string()
        .replace('\\', "/");
    let gateway = fs::read_to_string(crate_dir.join("src/bin/osfr-gateway.rs"))
        .unwrap()
        .replace("//!", "//")
        .replace(
            "#[path = \"../trace.rs\"]",
            &format!("#[path = \"{trace}\"]"),
        )
        .replace(
            "include_str!(\"../../studio/index.html\")",
            &format!("include_str!(\"{studio}\")"),
        )
        // The bin's test modules live in src/bin/osfr_gateway/ (#[path]); the splice moves the
        // source, so make those paths absolute.
        .replace(
            "#[path = \"osfr_gateway/",
            &format!("#[path = \"{bin_dir}/osfr_gateway/"),
        );
    let fixture = crate_dir
        .join("tests/common/startup_selfdata_fixture.rs")
        .display()
        .to_string()
        .replace('\\', "/");
    let multiplayer = fs::read_to_string(crate_dir.join("tests/multiplayer.rs"))
        .unwrap()
        .replace("\r\n", "\n")
        .replace(
            "#[path = \"../src/bin/osfr-gateway.rs\"]\n#[allow(dead_code)]\nmod gateway_runner;",
            &format!("#[allow(dead_code)] mod gateway_runner {{\n{gateway}\n{CLONE_PROBE}\n}}"),
        )
        .replace(
            "#[path = \"common/startup_selfdata_fixture.rs\"]",
            &format!("#[path = \"{fixture}\"]"),
        );
    assert!(
        multiplayer.contains("fn w96_clone_us"),
        "gateway module header drifted"
    );
    let source = probe_dir.join("gateway_probe.rs");
    fs::write(&source, format!("{multiplayer}\n{GATEWAY_PROBE}")).unwrap();
    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile", "-Command",
            "& $env:OSFR_W96_CARGO -CargoArgs @('rustc', '--manifest-path', $env:OSFR_W96_MANIFEST, '--release', '--test', 'load_players', '--message-format=json', '--', '--cfg', 'w96_gateway_probe'); exit $LASTEXITCODE",
        ])
        .env("OSFR_W96_CARGO", project.join("scripts/cargo.ps1"))
        .env("OSFR_W96_MANIFEST", crate_dir.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", build.join("gateway-probe-build"))
        .env("OSFR_W96_GATEWAY_PROBE", &source)
        .output()
        .unwrap();
    let diagnostics = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let v: serde_json::Value = serde_json::from_str(line).ok()?;
            v["message"]["rendered"].as_str().map(str::to_owned)
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        output.status.success(),
        "probe compile failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stderr),
        diagnostics
    );
    let executable = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let v: serde_json::Value = serde_json::from_str(line).ok()?;
            if v["reason"] == "compiler-artifact" && v["target"]["name"] == "load_players" {
                v["executable"].as_str().map(PathBuf::from)
            } else {
                None
            }
        })
        .last()
        .expect("cargo did not report probe executable");
    let output = Command::new(executable)
        .args([
            "gateway_probe::w96_step_and_clone",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .output()
        .unwrap();
    println!("{}", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.status.success(),
        "probe failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(not(w96_gateway_probe))]
const CLONE_PROBE: &str = r#"
pub(super) fn w96_clone_us(server: &Server) -> f64 {
    assert_eq!(server.world.chat_sessions.len(), 16);
    let start = std::time::Instant::now();
    for _ in 0..1000 { std::hint::black_box(server.world.clone()); }
    start.elapsed().as_secs_f64() * 1e6 / 1000.0
}
"#;

#[cfg(not(w96_gateway_probe))]
const GATEWAY_PROBE: &str = r#"
#[test]
#[ignore]
fn w96_step_and_clone() {
    let temp = TempDir::new();
    let (mut server, path) = start_server(&temp);
    let resources = Resources::load(&temp.0.join("creator")).unwrap();
    let mut store = AccountStore::load(&path).unwrap();
    for n in 3u8..16 {
        let account = store.create_account([n + 10; 32], AccountFlags::default(), SlotPolicy { max_characters: 1 }).unwrap();
        let mut choices = [0; 21];
        choices[0] = 1; choices[1] = 1; choices[2] = 2; choices[5] = 1;
        choices[7] = 201; choices[8] = 189; choices[9] = 199; choices[20] = 1;
        let label = format!("Synthetic{}", char::from(b'A' + n));
        let reply = create_character(&mut store, account, NativeCreation {
            first_name: label.clone(), last_name: "QA".into(),
            temporary_first_name: label, temporary_last_name: "QA".into(), choices,
        }, &resources);
        assert_eq!(reply.status, osfr_tools::account::CharacterCreateStatus::Created);
    }
    store.save(&path).unwrap();
    let mut clients = Vec::new();
    for guid in 1..=16 {
        let (client, messages) = log_in(&mut server, 500 + guid as u32, guid, &path);
        assert!(messages.iter().any(|m| is_opcode(m, 12)), "world entry missing for synthetic id {guid}");
        clients.push(client);
        // Ack the spawn fanout for every live client; otherwise the earlier
        // sessions hit the reliable retry limit while the later ones log in.
        for client in &mut clients { client.drain(&mut server); }
    }
    assert_eq!(server.active_session_count(), 16);
    let clone_us = gateway_runner::w96_clone_us(&server);
    // Flags-only movement avoids disk saves. 125 still traverses the real
    // decoder, gateway, commit_outcome world clone, registry and UDP fanout.
    let mut costs = Vec::new();
    for state in 1u8..=10 {
        let mut fields = 1u64.to_le_bytes().to_vec();
        for v in [1.0f32, 2.0, 3.0, 0.0, 0.0, 1.0] { fields.extend_from_slice(&v.to_le_bytes()); }
        fields.extend_from_slice(&[state, 0]);
        let message = packet(125, &fields);
        let first = &mut clients[0];
        first.sender.queue_message(&message).unwrap();
        first.clock_ms += 1;
        let wires = first.sender.poll(first.clock_ms).unwrap();
        assert_eq!(wires.len(), 1, "one application per measured step");
        let start = std::time::Instant::now();
        first.socket.send_to(&wires[0], first.server).unwrap();
        server.step().unwrap();
        costs.push(start.elapsed().as_secs_f64() * 1e6);
        let received = clients[1].drain(&mut server);
        assert_eq!(received.iter().filter(|m| is_opcode(m, 125) && m.ends_with(&[state, 0])).count(), 1,
            "measured step did not deliver the exact movement relay");
        for client in &mut clients { client.drain(&mut server); }
    }
    costs.sort_by(f64::total_cmp);
    println!("gateway,N=16,samples=10,step_median_us={:.3},step_max_us={:.3},SharedWorld_clone_drop_us_per_message={clone_us:.3}",
        costs[5], costs[9]);
    println!("clone uses actual live 16-player SharedWorld; step excludes drain/read-timeout and setup; synthetic empty content/history; flags-only movement, no save I/O");
}
"#;
