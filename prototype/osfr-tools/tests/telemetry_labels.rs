//! W174 (brief 3 of "Next 10"): telemetry and benign requests are a named quiet list, and
//! requests that gameplay.rs handles carry their own handler names in the gateway trace.
//!
//! Original facts: 105 and 109 are Legacy `PacketClientMetrics` / `PacketClientLog` (no reply),
//! 194 is `Ui.SetWallOfData`, 193/1 is the client's `AnnouncementDataRequestPacket`
//! (`00a3e050`, handler `00a40aa0`). Family 16 (two bytes) is named from the native traces
//! only: it is the last application message before every client disconnect (R). Everything
//! else here (accept, discard, never store or relay) is reconstructed policy (R).
//!
//! The replay test reads the private native traces under `lab/runs` when present; the shape
//! table in `quiet_shapes_seen_in_native_traces` stands alone without them.

use osfr_tools::{
    account_store::AccountRole,
    gameplay::{Gameplay, GameplayState},
    gateway::{Gateway, GatewayOutcome},
};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

const NOW: u64 = 10_000;
static SERIAL: AtomicU64 = AtomicU64::new(0);

fn lab() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab")
}

/// A copy of the private content package, removed on drop.
struct Content(PathBuf);

impl Content {
    fn new() -> Self {
        let source = lab().join("reports/round59-native-20261015T124443Z/content");
        assert!(
            source.join("harvesting.json").is_file(),
            "private content fixture: {}",
            source.display()
        );
        let dir = lab().join("build/w174-content").join(format!(
            "{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        for entry in fs::read_dir(&source).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file()
                && matches!(
                    path.extension().and_then(|s| s.to_str()),
                    Some("json" | "bin")
                )
            {
                fs::copy(&path, dir.join(path.file_name().unwrap())).unwrap();
            }
        }
        Self(dir)
    }
}

impl Drop for Content {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn wrap(outer: u8, inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![outer, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

fn login() -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    for (i, text) in ["w174-synthetic-ticket", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
        bytes.extend((text.len() as u32).to_le_bytes());
        bytes.extend(text.as_bytes());
        if i == 0 {
            bytes.extend(1u64.to_le_bytes());
        }
    }
    bytes
}

fn loaded_gateway(content: &Content) -> Gateway {
    let reference = lab().join("references/rust-gateway-reference");
    let mut character = Gateway::reference_state(&reference).unwrap();
    character.position = [0.; 3];
    let state: GameplayState = Gameplay::load(&content.0, None).unwrap().state();
    let mut gateway =
        Gateway::load_gameplay(&reference, character, &content.0, Some(state)).unwrap();
    gateway.set_dev_tools(false);
    gateway.set_role(AccountRole::Player);
    gateway.set_trace_enabled(true);
    gateway.handle_at(&login(), NOW).unwrap();
    gateway.handle_at(&wrap(5, &[13, 0]), NOW).unwrap();
    gateway.handle_at(&wrap(5, &[10, 0]), NOW).unwrap();
    gateway
}

/// Send one message; the gateway state must not change.
fn send(gateway: &mut Gateway, outer: u8, inner: &[u8]) -> GatewayOutcome {
    let before = gateway.snapshot();
    let out = gateway.handle_at(&wrap(outer, inner), NOW).unwrap();
    assert!(gateway.snapshot() == before, "state changed: {inner:02x?}");
    out
}

/// The trace says this request was understood, produced no reply and reached the budgeted router.
fn assert_quiet(out: &GatewayOutcome, label: &str, inner: &[u8]) {
    assert_eq!(out.trace.len(), 1, "{inner:02x?}");
    let event = &out.trace[0];
    assert_eq!(event.handler, label, "{inner:02x?}");
    assert_eq!(event.outcome, "handled", "{inner:02x?}");
    assert_eq!(event.reason, None, "{inner:02x?}");
    assert!(out.replies.is_empty(), "{inner:02x?}");
    // Forwarded to the binary's `route_application`, where the per-account budget counts it
    // and no parser claims it, so nothing is relayed.
    assert_eq!(out.applications, [inner.to_vec()], "{inner:02x?}");
}

fn assert_unknown(out: &GatewayOutcome, inner: &[u8]) {
    assert_eq!(out.trace.len(), 1, "{inner:02x?}");
    let event = &out.trace[0];
    assert_eq!(event.outcome, "rejected", "{inner:02x?}");
    assert_eq!(
        event.reason.as_deref(),
        Some("unknown application message"),
        "{inner:02x?}"
    );
    assert!(out.applications.is_empty(), "{inner:02x?}");
}

fn body(family: u16, sub_bytes: &[u8], len: usize) -> Vec<u8> {
    let mut bytes = family.to_le_bytes().to_vec();
    bytes.extend(sub_bytes);
    bytes.resize(len, 0);
    bytes
}

/// (family, bytes after the family, total length, label), copied from the 62 native traces.
const SHAPES: &[(u16, &[u8], usize, &str)] = &[
    (105, &[1, 0], 150, "client_metrics"),
    (109, &[17, 0], 138, "client_log"),
    (109, &[18, 0], 35, "client_log"),
    (109, &[19, 0], 37, "client_log"),
    (109, &[20, 0], 500, "client_log"),
    (109, &[25, 0], 188, "client_log"),
    (194, &[2, 1], 15, "ui_telemetry"),
    (194, &[2, 2], 23, "ui_telemetry"),
    (194, &[2, 3], 31, "ui_telemetry"),
    (194, &[3, 3], 19, "ui_telemetry"),
    (194, &[4, 7], 37, "ui_telemetry"),
    (194, &[4, 13], 32, "ui_telemetry"),
    (194, &[4, 15], 36, "ui_telemetry"),
    (193, &[1], 3, "announcements_request"),
    (16, &[], 2, "client_exit_notice"),
    (26, &[20, 0], 4, "free_interaction"),
    (26, &[26, 0], 12, "profile_notice"),
    (26, &[37, 0], 16, "profile_notice"),
];

#[test]
fn quiet_shapes_seen_in_native_traces() {
    let content = Content::new();
    let mut gateway = loaded_gateway(&content);
    // Both tunnel flavours occur in the traces: outer 5 (reliable) and outer 6 (direct).
    for outer in [5u8, 6] {
        for &(family, sub, len, label) in SHAPES {
            let inner = body(family, sub, len);
            assert_quiet(&send(&mut gateway, outer, &inner), label, &inner);
        }
    }
}

#[test]
fn real_unknowns_still_log_unknown() {
    let content = Content::new();
    let mut gateway = loaded_gateway(&content);
    for inner in [
        // Families nothing routes, including the neighbours of the quiet list.
        body(0xffff, &[0xff, 0xff], 12),
        body(777, &[1, 0], 12),
        body(104, &[1, 0], 12),
        body(106, &[1, 0], 12),
        body(192, &[1, 0], 12),
        body(195, &[1, 0], 12),
        // Same families, other shapes: server-side 193 subs and a longer 16.
        body(193, &[2], 3),
        body(193, &[4], 3),
        body(16, &[], 3),
        body(16, &[0, 0], 4),
    ] {
        assert_unknown(&send(&mut gateway, 5, &inner), &inner);
    }
}

#[test]
fn log_text_never_reaches_the_trace() {
    let content = Content::new();
    let mut gateway = loaded_gateway(&content);
    // No run of three letters, so the generic text heuristic would keep the head bytes.
    let inner = body(109, &[17, 0, 0, 0, 0x31, 0x32, 0x5c, 0x3a, 0x2e], 40);
    let out = send(&mut gateway, 5, &inner);
    assert_quiet(&out, "client_log", &inner);
    assert_eq!(out.trace[0].body_head_hex, None);
}

/// Replay every distinct inbound "unknown application message" shape from the native traces
/// (family, bytes 2..4, length): the quiet families must now be understood. Skipped when the
/// private traces are not on this machine.
#[test]
fn native_trace_replay_leaves_no_unknown_quiet_family() {
    let runs = lab().join("runs");
    let mut shapes: BTreeMap<(u16, u16, usize), Vec<u8>> = BTreeMap::new();
    let mut files = 0;
    for dir in fs::read_dir(&runs).into_iter().flatten().flatten() {
        let Ok(text) = fs::read_to_string(dir.path().join("server-trace.jsonl")) else {
            continue;
        };
        files += 1;
        for line in text.lines().filter(|l| l.contains("unknown application")) {
            let row: Value = serde_json::from_str(line).unwrap();
            let (Some(family), Some(len)) = (
                row["family"].as_u64().map(|f| f as u16),
                row["body_len"].as_u64().map(|l| l as usize),
            ) else {
                continue;
            };
            let sub = row["subtype"].as_u64().map_or(0, |s| s as u16);
            let mut inner = family.to_le_bytes().to_vec();
            if let Some(hex) = row["body_head_hex"].as_str() {
                inner = (0..hex.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                    .collect();
            } else if len >= 4 {
                inner.extend(sub.to_le_bytes());
            }
            inner.resize(len, 0);
            shapes.entry((family, sub, len)).or_insert(inner);
        }
    }
    if files == 0 {
        println!(
            "W174 replay skipped: no server-trace.jsonl under {}",
            runs.display()
        );
        return;
    }
    let content = Content::new();
    let mut gateway = loaded_gateway(&content);
    let (mut quiet, mut other) = (0, 0);
    for inner in shapes.values() {
        let family = u16::from_le_bytes([inner[0], inner[1]]);
        let sub = inner.get(2..4).map(|b| u16::from_le_bytes([b[0], b[1]]));
        let wanted = matches!(family, 105 | 109 | 194 | 193 | 16)
            || (family == 26 && matches!(sub, Some(20 | 26 | 37)));
        if wanted {
            let out = send(&mut gateway, 5, inner);
            assert_eq!(out.trace.len(), 1);
            assert_eq!(out.trace[0].outcome, "handled", "{inner:02x?}");
            assert_eq!(out.applications.len(), 1, "{inner:02x?}");
            quiet += 1;
        } else {
            other += 1;
        }
    }
    println!(
        "W174 replay: {files} traces, {quiet} quiet shapes understood, {other} other shapes left to later briefs"
    );
    assert!(quiet > 0);
}
