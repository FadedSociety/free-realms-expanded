//! Optional private gateway acceptance. No game bytes are embedded here.
use osfr_tools::{game_store, gameplay::Gameplay, gateway::Gateway, zone_transition};
use std::{fs, path::PathBuf};
fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut b = vec![5, 0, 1];
    b.extend((inner.len() as u32).to_le_bytes());
    b.extend(inner);
    b
}

#[test]
#[ignore = "requires OSFR_KART_PICKUP_PROBE and OSFR_KART_PICKUP_PROBE_APPLICATIONS private fixtures"]
fn private_early_grid_claim_is_refused_and_native_refill_ctrl_still_work() {
    use osfr_tools::kart_service::*;
    let root = PathBuf::from(std::env::var_os("OSFR_KART_PICKUP_PROBE").unwrap());
    let doc: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("initial-character.json")).unwrap()).unwrap();
    let mut gateway = Gateway::load_gameplay(
        &root.join("../../references/rust-gateway-reference"),
        serde_json::from_value(doc["character"].clone()).unwrap(),
        &root.join("content"),
        Some(serde_json::from_value(doc["gameplay"].clone()).unwrap()),
    )
    .unwrap();
    initialize(&mut gateway);
    gateway.set_dev_tools(true);
    gateway.set_role(osfr_tools::account_store::AccountRole::Admin);
    let before = gateway.snapshot().gameplay.unwrap();
    gateway.handle_at(&wrap(&chat("realm race")), 1100).unwrap();
    gateway.handle_at(&wrap(&[13, 0]), 1200).unwrap();
    gateway.handle_at(&wrap(&[10, 0]), 1300).unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_slice(
        &fs::read(std::env::var_os("OSFR_KART_PICKUP_PROBE_APPLICATIONS").unwrap()).unwrap(),
    )
    .unwrap();
    let mut counts = std::collections::BTreeMap::<u8, usize>::new();
    let mut registered = false;
    for row in rows {
        let Some(hex) = row["hex"].as_str() else {
            continue;
        };
        if row["direction"] != "client-to-server" {
            continue;
        }
        let raw: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        if raw.len() < 11 || !matches!(raw[0], 5 | 6) || raw[1] != 0 {
            continue;
        }
        let inner = &raw[7..];
        let now = (row["time"].as_f64().unwrap() * 1000.).round() as u64;
        let out = if inner.starts_with(&[33, 0, 9, 0]) && !registered {
            registered = true;
            gateway.handle_at(&wrap(inner), now).unwrap()
        } else if inner.starts_with(&[41, 0, 108, 0]) {
            gateway
                .handle_at(&wrap(&race_start(48_000_001, 1)), now)
                .unwrap()
        } else if registered && inner.starts_with(&[33, 0]) && matches!(inner[2], 7 | 17 | 19 | 22)
        {
            gateway.tick_at(now).unwrap();
            gateway.handle_at(&wrap(inner), now).unwrap()
        } else {
            continue;
        };
        for reply in out.replies {
            if reply.len() >= 11 && reply[7..9] == [33, 0] {
                *counts.entry(reply[9]).or_default() += 1;
            }
        }
    }
    assert_eq!(
        counts.get(&23).copied().unwrap_or(0),
        1,
        "native refill at10.603s must receive its acknowledgement"
    );
    // This old diagnostic placed a box on the grid: its only claim precedes
    // state9. Replay must now refuse it, rather than grant into a toolbar that
    // is about to be replaced. The refill/use later in the capture still work.
    assert_eq!(counts.get(&18).copied().unwrap_or(0), 0);
    assert_eq!(counts.get(&16).copied().unwrap_or(0), 0);
    assert_eq!(counts.get(&20).copied().unwrap_or(0), 1);
    let after = gateway.snapshot().gameplay.unwrap();
    assert_eq!(after.inventory, before.inventory);
    assert_eq!(
        after.profiles.as_ref().unwrap().experience,
        before.profiles.as_ref().unwrap().experience
    );
    let race = after.kart_entry.unwrap().race.unwrap();
    assert_eq!(race.phase, RacePhase::Racing);
    assert!(race.pickups_consumed.is_empty());
    assert!(race.queued_items.is_empty());
    assert_eq!(race.session.next_index, 1);
    assert_eq!(race.session.laps_completed, 0);
    assert!(!race.reward_taken);
}

#[test]
#[ignore = "requires OSFR_KART_PLAYABLE and OSFR_KART_NATIVE_APPLICATIONS private fixtures"]
fn private_actual_native_checkpoint_capture_completes_three_laps() {
    use osfr_tools::{kart_entry::EntryGate, kart_service::*};
    let root = PathBuf::from(std::env::var_os("OSFR_KART_PLAYABLE").unwrap());
    let cfg: osfr_tools::kart_gateway::KartGatewayContent =
        serde_json::from_slice(&fs::read(root.join("content/kart_entry.json")).unwrap()).unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_slice(
        &fs::read(std::env::var_os("OSFR_KART_NATIVE_APPLICATIONS").unwrap()).unwrap(),
    )
    .unwrap();
    let mut service = KartService::new(cfg.race.unwrap()).unwrap();
    service.set_gate(EntryGate::Enabled);
    let mut prepared = false;
    let mut accepted = 0;
    let mut refused = Vec::new();
    for row in rows {
        let now = (row["time"].as_f64().unwrap() * 1000.).round() as u64;
        let Some(hex) = row["hex"].as_str() else {
            continue;
        };
        let raw: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        if raw.len() < 11 || !matches!(raw[0], 5 | 6) || raw[1] != 0 {
            continue;
        }
        let inner = &raw[7..];
        let client = row["direction"] == "client-to-server";
        if client && inner.starts_with(&[33, 0, 9, 0]) && !prepared {
            service
                .prepare(
                    RaceContext {
                        session: 1,
                        player_guid: 1,
                        instance_id: 48_000_003,
                        encounter_id: 917566,
                    },
                    now,
                )
                .unwrap();
            prepared = true;
        } else if client && inner.starts_with(&[41, 0, 108, 0]) {
            service.start_countdown(now).unwrap();
        } else if prepared {
            service.tick(now).unwrap();
            if client && inner.starts_with(&[33, 0, 7, 0]) {
                match service.handle_inner(inner, now).unwrap() {
                    ServiceOutcome::Handled { replies, .. } => {
                        assert_eq!(replies.len(), 1);
                        accepted += 1;
                    }
                    ServiceOutcome::Refused(why) => refused.push((now, why)),
                }
            }
        }
    }
    assert_eq!(
        service.phase(),
        RacePhase::Finished,
        "accepted={accepted}; refusals={refused:?}"
    );
    assert!(service.finish_flag());
    assert_eq!(service.snapshot().unwrap().session.laps_completed, 3);
    assert_eq!(accepted, 93);
}

#[test]
#[ignore = "requires OSFR_KART_PLAYABLE and OSFR_KART_NATIVE_APPLICATIONS private fixtures"]
fn private_actual_native_route_pays_registered_reward_once() {
    let root = PathBuf::from(std::env::var_os("OSFR_KART_PLAYABLE").unwrap());
    let doc: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("initial-character.json")).unwrap()).unwrap();
    let mut g = Gateway::load_gameplay(
        &root.join("../../references/rust-gateway-reference"),
        serde_json::from_value(doc["character"].clone()).unwrap(),
        &root.join("content"),
        Some(serde_json::from_value(doc["gameplay"].clone()).unwrap()),
    )
    .unwrap();
    initialize(&mut g);
    g.set_dev_tools(true);
    g.set_role(osfr_tools::account_store::AccountRole::Admin);
    let before = g.snapshot().gameplay.unwrap();
    g.handle_at(&wrap(&chat("realm race")), 1100).unwrap();
    g.handle_at(&wrap(&[13, 0]), 1200).unwrap();
    g.handle_at(&wrap(&[10, 0]), 1300).unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_slice(
        &fs::read(std::env::var_os("OSFR_KART_NATIVE_APPLICATIONS").unwrap()).unwrap(),
    )
    .unwrap();
    let mut registered = false;
    let mut progress = 0;
    let mut toasts = 0;
    for row in rows {
        let now = (row["time"].as_f64().unwrap() * 1000.).round() as u64;
        let Some(hex) = row["hex"].as_str() else {
            continue;
        };
        let raw: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        if raw.len() < 11 || !matches!(raw[0], 5 | 6) || raw[1] != 0 {
            continue;
        }
        let inner = &raw[7..];
        if row["direction"] != "client-to-server" {
            continue;
        }
        let out = if inner.starts_with(&[33, 0, 9, 0]) && !registered {
            registered = true;
            g.handle_at(&wrap(inner), now).unwrap()
        } else if inner.starts_with(&[41, 0, 108, 0]) {
            // The captured instance belongs to the previous run; this fresh gateway owns instance1.
            g.handle_at(&wrap(&race_start(48_000_001, 1)), now).unwrap()
        } else if registered && inner.starts_with(&[33, 0, 7, 0]) {
            g.tick_at(now).unwrap();
            g.handle_at(&wrap(inner), now).unwrap()
        } else {
            continue;
        };
        progress += out
            .replies
            .iter()
            .filter(|p| p[7..].starts_with(&[33, 0, 14, 0]))
            .count();
        toasts += out
            .replies
            .iter()
            .filter(|p| p[7..].starts_with(&[50, 0, 1]))
            .count();
    }
    let after = g.snapshot().gameplay.unwrap();
    let race = after.kart_entry.as_ref().unwrap().race.as_ref().unwrap();
    assert_eq!(race.session.laps_completed, 3);
    assert_eq!(race.phase, osfr_tools::kart_service::RacePhase::Finished);
    assert!(race.reward_taken);
    assert_eq!(progress, 94, "initial progress plus93 accepted checkpoints");
    assert_eq!(toasts, 1);
    assert_eq!(after.inventory.coins, before.inventory.coins + 50);
    let xp = |game: &osfr_tools::gameplay::GameplayState| {
        game.profiles
            .as_ref()
            .unwrap()
            .experience
            .get(&48)
            .copied()
            .unwrap_or(0)
    };
    assert_eq!(xp(&after), xp(&before) + 25);
}
#[test]
#[ignore = "requires OSFR_KART_PICKUP_V15 disposable private package"]
fn private_v15_shield_respawn_preserves_paid_economy() {
    let root = PathBuf::from(std::env::var_os("OSFR_KART_PICKUP_V15").unwrap());
    let doc: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("initial-character.json")).unwrap()).unwrap();
    let cfg: osfr_tools::kart_gateway::KartGatewayContent =
        serde_json::from_slice(&fs::read(root.join("content/kart_entry.json")).unwrap()).unwrap();
    let race = cfg.race.as_ref().unwrap();
    assert_eq!(race.pickups.len(), 15);
    assert_eq!(race.pickups.iter().filter(|p| p.item_type == 4).count(), 5);
    let instance = doc["gameplay"]["kart_entry"]["next_instance"]
        .as_u64()
        .unwrap() as u32;
    let mut g = Gateway::load_gameplay(
        &root.join("../../references/rust-gateway-reference"),
        serde_json::from_value(doc["character"].clone()).unwrap(),
        &root.join("content"),
        Some(serde_json::from_value(doc["gameplay"].clone()).unwrap()),
    )
    .unwrap();
    initialize(&mut g);
    g.set_dev_tools(true);
    g.set_role(osfr_tools::account_store::AccountRole::Admin);
    let before = g.snapshot().gameplay.unwrap();
    g.handle_at(&wrap(&chat("realm race")), 1100).unwrap();
    g.handle_at(&wrap(&[13, 0]), 1200).unwrap();
    let mut register = vec![33, 0, 9, 0];
    register.extend(1u64.to_le_bytes());
    register.extend([0; 8]);
    g.handle_at(&wrap(&register), 1500).unwrap();
    g.handle_at(&wrap(&[10, 0]), 1600).unwrap();
    g.handle_at(&wrap(&race_start(instance, 1)), 2000).unwrap();
    let spawn = g.tick_at(20_000).unwrap();
    assert_eq!(
        spawn
            .replies
            .iter()
            .filter(|r| r[7..].starts_with(&[33, 0, 15, 0]))
            .count(),
        15
    );
    let mut claim = vec![33, 0, 17, 0];
    claim.extend(1u64.to_le_bytes());
    for value in race.pickups[0].position {
        claim.extend(value.to_le_bytes());
    }
    claim.extend(1f32.to_le_bytes());
    let grant = g.handle_at(&wrap(&claim), 20_100).unwrap();
    assert!(
        grant
            .replies
            .iter()
            .any(|r| r[7..] == [33, 0, 18, 0, 4, 0, 0, 0])
    );
    let mut use_item = vec![33, 0, 19, 0];
    use_item.extend(1u64.to_le_bytes());
    let used = g.handle_at(&wrap(&use_item), 20_200).unwrap();
    assert!(
        used.replies
            .iter()
            .any(|r| r[7..] == [33, 0, 20, 0, 4, 0, 0, 0])
    );
    assert!(
        !g.tick_at(25_199)
            .unwrap()
            .replies
            .iter()
            .any(|r| r[7..].starts_with(&[33, 0, 21, 0]))
    );
    assert!(
        g.tick_at(25_200)
            .unwrap()
            .replies
            .iter()
            .any(|r| r[7..] == [33, 0, 21, 0, 4, 0, 0, 0])
    );
    assert!(
        !g.tick_at(28_099)
            .unwrap()
            .replies
            .iter()
            .any(|r| r[7..].starts_with(&[33, 0, 15, 0]))
    );
    let respawn = g.tick_at(28_100).unwrap();
    assert_eq!(
        respawn
            .replies
            .iter()
            .filter(|r| r[7..].starts_with(&[33, 0, 15, 0]))
            .count(),
        1
    );
    assert!(
        g.handle_at(&wrap(&claim), 28_200)
            .unwrap()
            .replies
            .iter()
            .any(|r| r[7..] == [33, 0, 18, 0, 4, 0, 0, 0])
    );
    g.handle_at(&wrap(&[26, 0, 42, 0]), 28_300).unwrap();
    let after = g.snapshot();
    let game = after.gameplay.as_ref().unwrap();
    assert_eq!(game.inventory, before.inventory);
    assert_eq!(
        game.profiles.as_ref().unwrap().experience,
        before.profiles.as_ref().unwrap().experience
    );
    assert!(game.kart_entry.as_ref().unwrap().active.is_none());
    let path = root.join(format!("pickup-v15-save-{}.json", std::process::id()));
    game_store::save(&path, &after).unwrap();
    assert_eq!(
        game_store::load_or_create(&path, after.character.clone()).unwrap(),
        after
    );
    fs::remove_file(path).unwrap();
}

fn chat(text: &str) -> Vec<u8> {
    let mut b = vec![15, 0, 1, 0, 0, 0];
    b.extend(1u64.to_le_bytes());
    b.extend(0u64.to_le_bytes());
    for _ in 0..2 {
        b.extend([0; 20]);
    }
    b.extend((text.len() as u32).to_le_bytes());
    b.extend(text.bytes());
    b.extend([0; 28]);
    b
}
fn race_start(instance: u32, guid: u64) -> Vec<u8> {
    let mut b = vec![41, 0, 108, 0];
    b.extend(917566u32.to_le_bytes());
    b.extend(instance.to_le_bytes());
    b.extend(guid.to_le_bytes());
    b
}
fn initialize(g: &mut Gateway) {
    let mut b = 1u16.to_le_bytes().to_vec();
    for (i, s) in ["local-private-test", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
        b.extend((s.len() as u32).to_le_bytes());
        b.extend(s.bytes());
        if i == 0 {
            b.extend(1u64.to_le_bytes());
        }
    }
    g.handle_at(&b, 0).unwrap();
    g.handle_at(&wrap(&[13, 0]), 100).unwrap();
    g.handle_at(&wrap(&[10, 0]), 200).unwrap();
}

#[test]
#[ignore = "requires OSFR_KART_PLAYABLE with recovered private pickup definitions"]
fn private_pickup_gateway_grant_use_rollback_and_restore() {
    let root = PathBuf::from(std::env::var_os("OSFR_KART_PLAYABLE").unwrap());
    let content = root.join("content");
    let reference = root.join("../../references/rust-gateway-reference");
    let doc: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("initial-character.json")).unwrap()).unwrap();
    let character = serde_json::from_value(doc["character"].clone()).unwrap();
    let saved = serde_json::from_value(doc["gameplay"].clone()).unwrap();
    let cfg: serde_json::Value =
        serde_json::from_slice(&fs::read(content.join("kart_entry.json")).unwrap()).unwrap();
    let specs: Vec<osfr_tools::kart_pickups::PickupSpec> =
        serde_json::from_value(cfg["race"]["pickups"].clone()).unwrap();
    assert_eq!(specs.len(), 37);
    let mut g = Gateway::load_gameplay(&reference, character, &content, Some(saved)).unwrap();
    initialize(&mut g);
    g.set_dev_tools(true);
    g.set_role(osfr_tools::account_store::AccountRole::Admin);
    g.handle_at(&wrap(&chat("realm race")), 1100).unwrap();
    g.handle_at(&wrap(&[13, 0]), 1200).unwrap();
    let mut register = vec![33, 0, 9, 0];
    register.extend(1u64.to_le_bytes());
    register.extend([0; 8]);
    g.handle_at(&wrap(&register), 1500).unwrap();
    g.handle_at(&wrap(&[10, 0]), 1600).unwrap();
    g.handle_at(&wrap(&race_start(48_000_001, 1)), 2000)
        .unwrap();
    let started = g.tick_at(20_000).unwrap();
    let creates: Vec<_> = started
        .replies
        .iter()
        .filter(|p| p[7..].starts_with(&[33, 0, 15, 0]))
        .collect();
    assert_eq!(
        creates.len(),
        specs.len(),
        "every authored actor must spawn exactly once"
    );
    for (packet, spec) in creates.iter().zip(&specs) {
        assert_eq!(&packet[7..], osfr_tools::kart_pickups::encode_create(spec));
    }
    assert!(
        !g.tick_at(20_001)
            .unwrap()
            .replies
            .iter()
            .any(|p| p[7..].starts_with(&[33, 0, 15, 0]))
    );
    // Reach the first authored checkpoint before its nearby pickup. The box is228m from
    // the grid, beyond a pristine180m movement window even after idle time.
    let first_position: [f32; 3] =
        serde_json::from_value(cfg["race"]["gates"][0]["position"].clone()).unwrap();
    let mut first_gate = vec![33, 0, 7, 0];
    first_gate.extend(1u64.to_le_bytes());
    for v in first_position {
        first_gate.extend(v.to_le_bytes());
    }
    first_gate.extend(1f32.to_le_bytes());
    assert!(
        g.handle_at(&wrap(&first_gate), 22_000)
            .unwrap()
            .replies
            .iter()
            .any(|p| p[7..].starts_with(&[33, 0, 14, 0]))
    );
    let before = g.snapshot();
    let mut claim = vec![33, 0, 17, 0];
    claim.extend(1u64.to_le_bytes());
    for v in specs[0].position {
        claim.extend(v.to_le_bytes());
    }
    claim.extend(1f32.to_le_bytes());
    let mut foreign = claim.clone();
    foreign[4..12].copy_from_slice(&999u64.to_le_bytes());
    assert!(
        g.handle_at(&wrap(&foreign), 25_000)
            .unwrap()
            .replies
            .is_empty()
    );
    assert_eq!(g.snapshot(), before);
    let valid = wrap(&claim);
    let malformed = wrap(&[33, 0, 9, 0]);
    let mut bundle = vec![0, 25, valid.len() as u8];
    bundle.extend(valid);
    bundle.push(malformed.len() as u8);
    bundle.extend(malformed);
    assert!(g.handle_at(&bundle, 25_000).is_err());
    assert_eq!(
        g.snapshot(),
        before,
        "failed bundle must not consume actor or queue slot"
    );
    let picked = g.handle_at(&wrap(&claim), 25_000).unwrap();
    assert_eq!(
        picked
            .replies
            .iter()
            .map(|p| p[7..].to_vec())
            .collect::<Vec<_>>(),
        vec![
            osfr_tools::kart_pickups::encode_grant(5),
            osfr_tools::kart_pickups::encode_destroy(specs[0].handle)
        ]
    );
    let after = g.snapshot();
    let game = after.gameplay.as_ref().unwrap();
    let banks = before.gameplay.as_ref().unwrap();
    assert_eq!(game.inventory, banks.inventory);
    assert_eq!(game.profiles, banks.profiles);
    assert_eq!(
        game.kart_entry
            .as_ref()
            .unwrap()
            .race
            .as_ref()
            .unwrap()
            .session,
        banks
            .kart_entry
            .as_ref()
            .unwrap()
            .race
            .as_ref()
            .unwrap()
            .session
    );
    assert_eq!(
        game.kart_entry
            .as_ref()
            .unwrap()
            .race
            .as_ref()
            .unwrap()
            .queued_items,
        [5]
    );
    assert!(
        g.handle_at(&wrap(&claim), 25_100)
            .unwrap()
            .replies
            .is_empty()
    );
    let replayed = g.snapshot();
    assert_eq!(
        replayed
            .gameplay
            .as_ref()
            .unwrap()
            .kart_entry
            .as_ref()
            .unwrap()
            .race
            .as_ref()
            .unwrap()
            .queued_items,
        [5]
    );
    // Service restore is a server checkpoint, unlike reconnect which deliberately aborts a race.
    let race_cfg: osfr_tools::kart_gateway::KartGatewayContent =
        serde_json::from_value(cfg).unwrap();
    let mut restored = osfr_tools::kart_service::KartService::restore(
        race_cfg.race.unwrap(),
        game.kart_entry.as_ref().unwrap().race.clone().unwrap(),
    )
    .unwrap();
    restored.set_gate(osfr_tools::kart_entry::EntryGate::Enabled);
    assert!(
        restored
            .handle_inner(&claim, 25_100)
            .unwrap()
            .replies()
            .is_empty()
    );
    let mut use_item = vec![33, 0, 19, 0];
    use_item.extend(1u64.to_le_bytes());
    assert_eq!(
        g.handle_at(&wrap(&use_item), 25_200)
            .unwrap()
            .replies
            .iter()
            .map(|p| p[7..].to_vec())
            .collect::<Vec<_>>(),
        vec![vec![33, 0, 20, 0, 5, 0, 0, 0]]
    );
    assert!(
        g.handle_at(&wrap(&use_item), 25_201)
            .unwrap()
            .replies
            .is_empty()
    );
    assert_eq!(
        g.snapshot().gameplay.as_ref().unwrap().inventory,
        banks.inventory
    );
}
#[test]
#[ignore = "requires OSFR_KART_PLAYABLE pointing at the private disposable package"]
fn private_leaving_a_race_restores_the_previous_job() {
    let root = PathBuf::from(std::env::var_os("OSFR_KART_PLAYABLE").unwrap());
    let reference = root.join("../../references/rust-gateway-reference");
    let doc: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("initial-character.json")).unwrap()).unwrap();
    let character = serde_json::from_value(doc["character"].clone()).unwrap();
    let saved = serde_json::from_value(doc["gameplay"].clone()).unwrap();
    let mut g =
        Gateway::load_gameplay(&reference, character, &root.join("content"), Some(saved)).unwrap();
    initialize(&mut g);
    g.set_dev_tools(true);
    g.set_role(osfr_tools::account_store::AccountRole::Admin);
    let job = |g: &Gateway| g.snapshot().gameplay.unwrap().profiles.unwrap().active;
    let before = job(&g);
    assert_ne!(before, 48);
    g.handle_at(&wrap(&chat("realm race")), 1000).unwrap();
    assert_eq!(job(&g), 48); // a race needs Kart Driver
    g.handle_at(&wrap(&[13, 0]), 1200).unwrap(); // the race scene loads
    g.handle_at(&wrap(&[10, 0]), 1300).unwrap();
    g.handle_at(&wrap(&chat("realm exit")), 2000).unwrap();
    assert_eq!(job(&g), before); // (R) and leaving puts the old job back
}

#[test]
#[ignore = "requires OSFR_KART_PLAYABLE pointing at the private disposable package"]
fn private_gateway_entry_save_restore_spoof_job_guard_and_abort() {
    let root = PathBuf::from(std::env::var_os("OSFR_KART_PLAYABLE").unwrap());
    let content = root.join("content");
    let reference = root.join("../../references/rust-gateway-reference");
    let doc: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("initial-character.json")).unwrap()).unwrap();
    let character = serde_json::from_value(doc["character"].clone()).unwrap();
    let saved = serde_json::from_value(doc["gameplay"].clone()).unwrap();
    let mut g = Gateway::load_gameplay(&reference, character, &content, Some(saved)).unwrap();
    initialize(&mut g);
    let before = g.snapshot();
    assert!(
        !g.handle_at(&wrap(&chat("realm race")), 1000)
            .unwrap()
            .gameplay_changed
    );
    assert_eq!(g.snapshot(), before);
    g.set_dev_tools(true);
    g.set_role(osfr_tools::account_store::AccountRole::Admin);
    let start = g.handle_at(&wrap(&chat("realm race")), 1100).unwrap();
    assert!(start.gameplay_changed && start.state_changed);
    let scene = start
        .replies
        .iter()
        .find_map(|r| zone_transition::decode(r).ok())
        .unwrap();
    assert_eq!(scene.scene, "sg_kt_track");
    assert_eq!(scene.rotation[3], 0.);
    // The package save may already have raced; take its next instance.
    let instance = doc["gameplay"]["kart_entry"]["next_instance"].as_u64().unwrap_or(48_000_001) as u32;
    assert_eq!(g.state().zone_id, instance);
    let gameplay = g.snapshot().gameplay.unwrap();
    assert_eq!(gameplay.profiles.as_ref().unwrap().active, 48);
    assert_eq!(
        gameplay.inventory,
        before.gameplay.as_ref().unwrap().inventory
    );
    let restored = Gameplay::load(&content, Some(gameplay.clone())).unwrap();
    assert!(restored.supports_location(g.state()));
    let reconnect =
        Gateway::load_gameplay(&reference, g.state().clone(), &content, Some(gameplay)).unwrap();
    assert_eq!(reconnect.state(), &before.character);
    assert!(
        reconnect
            .snapshot()
            .gameplay
            .unwrap()
            .kart_entry
            .unwrap()
            .active
            .is_none()
    );
    let ready = g.handle_at(&wrap(&[13, 0]), 1200).unwrap();
    assert!(!ready.gameplay_changed);
    assert!(
        ready
            .replies
            .iter()
            .any(|r| r[7..].starts_with(&[41, 0, 114, 0]))
    );
    assert_eq!(&ready.replies.last().unwrap()[7..], &[14, 0]);
    let details: Vec<_> = ready
        .replies
        .iter()
        .filter(|r| r[7..].starts_with(&[41, 0, 114, 0]))
        .collect();
    assert_eq!(details.len(), 2);
    let mut cleared = details[0].clone();
    let activity = cleared.len() - 15;
    cleared[activity..activity + 4].fill(0);
    assert_eq!(&cleared, details[1]);
    assert!(ready.replies.iter().any(|r| r[7..] == [38, 0, 26, 0, 0]));
    let status = ready
        .replies
        .iter()
        .find(|r| r[7..].starts_with(&[35, 0, 20, 0]))
        .unwrap();
    assert_eq!(&status[11..19], &1u64.to_le_bytes());
    assert_eq!(&status[19..23], &0x401u32.to_le_bytes());
    let mut register = vec![33, 0, 9, 0];
    register.extend(999u64.to_le_bytes());
    register.extend([0; 8]);
    assert!(
        g.handle_at(&wrap(&register), 1400)
            .unwrap()
            .replies
            .is_empty()
    );
    register[4..12].copy_from_slice(&1u64.to_le_bytes());
    let entry = g.handle_at(&wrap(&register), 1500).unwrap();
    assert!(
        entry
            .replies
            .iter()
            .any(|r| r[7..].starts_with(&[33, 0, 11, 0]))
    );
    let loaded = g.handle_at(&wrap(&[10, 0]), 1510).unwrap();
    assert!(!loaded.replies.iter().any(|r| r[7..].starts_with(&[35, 0])));
    assert_eq!(&loaded.replies[0][7..11], &[41, 0, 107, 0]);
    assert_eq!(loaded.replies[0].len(), 19);
    let enter = loaded
        .replies
        .iter()
        .find(|r| r[7..].starts_with(&[41, 0, 2, 0]))
        .unwrap();
    assert_eq!(enter.len(), 28);
    assert_eq!(&enter[19..27], &1u64.to_le_bytes());
    assert_eq!(enter[27], 0);
    // Assets/menu time is not racing time. Foreign, premature and malformed
    // start messages cannot arm the race or alter its clock.
    let setup = g.tick_at(90_000).unwrap();
    assert!(
        setup
            .replies
            .iter()
            .any(|r| r[7..] == [33, 0, 1, 0, 6, 0, 0, 0])
    );
    assert!(
        !setup
            .replies
            .iter()
            .any(|r| r[7..].starts_with(&[33, 0, 10, 0]))
    );
    let waiting = g.snapshot();
    assert!(g.tick_at(90_001).unwrap().replies.is_empty());
    assert_eq!(g.snapshot(), waiting);
    for request in [race_start(instance + 1, 1), race_start(instance, 999)] {
        assert!(
            g.handle_at(&wrap(&request), 90_000)
                .unwrap()
                .replies
                .is_empty()
        );
        assert_eq!(g.snapshot(), waiting);
    }
    assert!(g.handle_at(&wrap(&[41, 0, 108, 0]), 90_000).is_err());
    assert_eq!(g.snapshot(), waiting);
    g.handle_at(&wrap(&race_start(instance, 1)), 90_000)
        .unwrap();
    let started = g.snapshot();
    assert_eq!(
        started
            .gameplay
            .as_ref()
            .unwrap()
            .kart_entry
            .as_ref()
            .unwrap()
            .race
            .as_ref()
            .unwrap()
            .phase,
        osfr_tools::kart_service::RacePhase::Countdown
    );
    g.handle_at(&wrap(&race_start(instance, 1)), 90_100)
        .unwrap();
    assert_eq!(g.snapshot(), started);
    let snapshot = g.snapshot();
    assert!(g.handle_at(&wrap(&[33, 0, 9, 0]), 1600).is_err());
    assert_eq!(g.snapshot(), snapshot);
    g.handle_at(&wrap(&[26, 0, 13, 0, 43, 0, 0, 0]), 1700)
        .unwrap();
    assert_eq!(
        g.snapshot()
            .gameplay
            .as_ref()
            .unwrap()
            .profiles
            .as_ref()
            .unwrap()
            .active,
        48
    );
    let stop = g.handle_at(&wrap(&chat("realm exit")), 1800).unwrap();
    assert!(stop.gameplay_changed);
    assert_eq!(g.state(), &before.character);
    let after = g.snapshot();
    // Leaving the race restores the job the player had before it.
    assert_eq!(
        after.gameplay.as_ref().unwrap().profiles.as_ref().unwrap().active,
        before.gameplay.as_ref().unwrap().profiles.as_ref().unwrap().active
    );
    assert_eq!(
        after.gameplay.as_ref().unwrap().inventory,
        before.gameplay.as_ref().unwrap().inventory
    );
    assert!(
        after
            .gameplay
            .as_ref()
            .unwrap()
            .kart_entry
            .as_ref()
            .unwrap()
            .active
            .is_none()
    );
    let path = root.join(format!("integration-test-save-{}.json", std::process::id()));
    game_store::save(&path, &after).unwrap();
    let reloaded = game_store::load_or_create(&path, after.character.clone()).unwrap();
    assert_eq!(reloaded, after);
    fs::remove_file(path).unwrap();
}

#[test]
#[ignore = "requires OSFR_KART_PLAYABLE pointing at the priced private race package"]
fn private_complete_race_pays_once_and_preserves_both_banks_on_restart() {
    complete_private_race(false);
}

#[test]
#[ignore = "requires OSFR_KART_PLAYABLE pointing at the private race package"]
fn private_timed_out_race_returns_to_world_without_rewards() {
    let root = PathBuf::from(std::env::var_os("OSFR_KART_PLAYABLE").unwrap());
    let doc: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("initial-character.json")).unwrap()).unwrap();
    let mut gateway = Gateway::load_gameplay(
        &root.join("../../references/rust-gateway-reference"),
        serde_json::from_value(doc["character"].clone()).unwrap(),
        &root.join("content"),
        Some(serde_json::from_value(doc["gameplay"].clone()).unwrap()),
    )
    .unwrap();
    initialize(&mut gateway);
    gateway.set_dev_tools(true);
    gateway.set_role(osfr_tools::account_store::AccountRole::Admin);
    let before = gateway.snapshot();
    gateway.handle_at(&wrap(&chat("realm race")), 1100).unwrap();
    gateway.handle_at(&wrap(&[13, 0]), 1200).unwrap();
    let mut register = vec![33, 0, 9, 0];
    register.extend(1u64.to_le_bytes());
    register.extend([0; 8]);
    gateway.handle_at(&wrap(&register), 1500).unwrap();
    gateway.handle_at(&wrap(&[10, 0]), 1600).unwrap();
    gateway
        .handle_at(&wrap(&race_start(48_000_001, 1)), 2000)
        .unwrap();
    gateway.tick_at(20_000).unwrap();
    let returned = gateway.tick_at(700_000).unwrap();
    assert!(returned.gameplay_changed);
    assert!(
        returned
            .replies
            .iter()
            .any(|packet| zone_transition::decode(packet).is_ok())
    );
    assert!(
        !returned
            .replies
            .iter()
            .any(|packet| packet[7..].starts_with(&[50, 0, 1]))
    );
    let after = gateway.snapshot();
    assert_eq!(after.character, before.character);
    let game = after.gameplay.as_ref().unwrap();
    assert_eq!(game.inventory, before.gameplay.as_ref().unwrap().inventory);
    assert_eq!(
        game.profiles.as_ref().unwrap().experience,
        before
            .gameplay
            .as_ref()
            .unwrap()
            .profiles
            .as_ref()
            .unwrap()
            .experience
    );
    assert!(game.kart_entry.as_ref().unwrap().active.is_none());
    assert_eq!(
        game.kart_entry
            .as_ref()
            .unwrap()
            .race
            .as_ref()
            .unwrap()
            .phase,
        osfr_tools::kart_service::RacePhase::Aborted
    );
    assert!(gateway.tick_at(700_100).unwrap().replies.is_empty());
}

#[test]
#[ignore = "requires OSFR_KART_PLAYABLE pointing at the lane-band private package"]
fn private_different_driving_lane_finishes_and_registers_rewards_once() {
    complete_private_race(true);
}

fn complete_private_race(different_lane: bool) {
    let root = PathBuf::from(std::env::var_os("OSFR_KART_PLAYABLE").unwrap());
    let content = root.join("content");
    let reference = root.join("../../references/rust-gateway-reference");
    let mut doc: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("initial-character.json")).unwrap()).unwrap();
    // Exercise the rank-up path as well as the ordinary XP delta.
    doc["gameplay"]["profiles"]["unlocked"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!(48));
    doc["gameplay"]["profiles"]["experience"]["48"] = serde_json::json!(75);
    let character = serde_json::from_value(doc["character"].clone()).unwrap();
    let saved = serde_json::from_value(doc["gameplay"].clone()).unwrap();
    let cfg: osfr_tools::kart_gateway::KartGatewayContent =
        serde_json::from_slice(&fs::read(content.join("kart_entry.json")).unwrap()).unwrap();
    let mut race = cfg.race.unwrap();
    if different_lane {
        if !race.gate_curtains.is_empty() {
            // Use the authored curtain's horizontal edge as the lane direction.
            // The old waypoint secant is not the physical gate normal.
            for (gate, leaves) in race.gates.iter_mut().zip(&race.gate_curtains) {
                let vertices = leaves[0].vertices;
                // Ring order may start on a vertical edge. Choose a horizontal edge,
                // then move toward its centre so the alternate lane stays on the surface.
                let (dx, dz) = (0..4)
                    .map(|i| {
                        let a = vertices[i];
                        let b = vertices[(i + 1) % 4];
                        (b[0] - a[0], b[2] - a[2])
                    })
                    .max_by(|a, b| a.0.hypot(a.1).total_cmp(&b.0.hypot(b.1)))
                    .unwrap();
                let length = dx.hypot(dz);
                assert!(length > 20.0);
                let center_x = vertices.iter().map(|v| v[0]).sum::<f32>() / 4.0;
                let center_z = vertices.iter().map(|v| v[2]).sum::<f32>() / 4.0;
                let toward =
                    (center_x - gate.position[0]) * dx + (center_z - gate.position[2]) * dz;
                let shift = if toward >= 0.0 { 10.0 } else { -10.0 };
                gate.position[0] += shift * dx / length;
                gate.position[2] += shift * dz / length;
            }
        } else {
            let band = race.gate_band.expect("lane test requires opt-in band");
            assert!(band.half_width >= 10.0);
            let centers: Vec<_> = race.gates.iter().map(|gate| gate.position).collect();
            for (i, gate) in race.gates.iter_mut().enumerate() {
                let (a, b) = if i == 0 {
                    (centers[0], centers[1])
                } else if i + 1 == centers.len() {
                    (centers[i - 1], centers[i])
                } else {
                    (centers[i - 1], centers[i + 1])
                };
                let dx = b[0] - a[0];
                let dz = b[2] - a[2];
                let length = dx.hypot(dz);
                // Real-sized alternate driving line, well outside the old three-metre sphere.
                gate.position[0] -= 10.0 * dz / length;
                gate.position[2] += 10.0 * dx / length;
            }
        }
    }
    assert_eq!(race.speedway, 31);
    let reward = race.reward.unwrap();
    let mut g = Gateway::load_gameplay(&reference, character, &content, Some(saved)).unwrap();
    initialize(&mut g);
    g.set_dev_tools(true);
    g.set_role(osfr_tools::account_store::AccountRole::Admin);
    let before = g.snapshot();
    let before_game = before.gameplay.as_ref().unwrap();
    let coins = before_game.inventory.coins;
    let xp = before_game
        .profiles
        .as_ref()
        .unwrap()
        .experience
        .get(&48)
        .copied()
        .unwrap_or(0);
    g.handle_at(&wrap(&chat("realm race")), 1100).unwrap();
    g.handle_at(&wrap(&[13, 0]), 1200).unwrap();
    let mut register = vec![33, 0, 9, 0];
    register.extend(1u64.to_le_bytes());
    register.extend([0; 8]);
    g.handle_at(&wrap(&register), 1500).unwrap();
    g.handle_at(&wrap(&[10, 0]), 1600).unwrap();
    g.handle_at(&wrap(&race_start(48_000_001, 1)), 2000)
        .unwrap();
    let zero = race
        .schedule
        .iter()
        .find(|s| s.frame == osfr_tools::kart_service::ScheduledFrame::Countdown(0))
        .unwrap()
        .at_ms;
    let mut now = 2000 + race.schedule.last().unwrap().at_ms.max(zero);
    g.tick_at(now).unwrap();
    // Automatic boost acknowledgement must be registered through the gateway,
    // preserve both economy banks, and stay distinct from the finish reward.
    now += 20_000;
    let boost_before = g.snapshot();
    let mut boost = vec![33, 0, 22, 0];
    boost.extend(1u64.to_le_bytes());
    let ack = g.handle_at(&wrap(&boost), now).unwrap();
    assert_eq!(
        ack.replies.iter().map(|p| &p[7..]).collect::<Vec<_>>(),
        vec![&[33, 0, 23, 0][..]]
    );
    // Native refill is config-driven, so separate reliable requests fill available
    // slots; the fourth entry must be refused without changing the snapshot.
    for offset in [1, 2] {
        let ack = g.handle_at(&wrap(&boost), now + offset).unwrap();
        assert_eq!(
            ack.replies.iter().map(|p| &p[7..]).collect::<Vec<_>>(),
            vec![&[33, 0, 23, 0][..]]
        );
    }
    let boosted = g.snapshot();
    assert_eq!(
        boosted.gameplay.as_ref().unwrap().inventory,
        boost_before.gameplay.as_ref().unwrap().inventory
    );
    assert_eq!(
        boosted.gameplay.as_ref().unwrap().profiles,
        boost_before.gameplay.as_ref().unwrap().profiles
    );
    assert!(
        g.handle_at(&wrap(&boost), now + 3)
            .unwrap()
            .replies
            .is_empty()
    );
    assert_eq!(g.snapshot(), boosted);
    // Native Ctrl/Alt sends UseItemRequest, not another refill. A malformed
    // bundle must not consume the queued boost before its later child fails.
    let mut use_item = vec![33, 0, 19, 0];
    use_item.extend(1u64.to_le_bytes());
    let valid = wrap(&use_item);
    let malformed = wrap(&[33, 0, 9, 0]);
    let mut bad_bundle = vec![0, 25, valid.len() as u8];
    bad_bundle.extend(valid);
    bad_bundle.push(malformed.len() as u8);
    bad_bundle.extend(malformed);
    assert!(g.handle_at(&bad_bundle, now + 4).is_err());
    assert_eq!(g.snapshot(), boosted);
    for offset in [5, 6, 7] {
        let activated = g.handle_at(&wrap(&use_item), now + offset).unwrap();
        assert_eq!(
            activated
                .replies
                .iter()
                .map(|p| &p[7..])
                .collect::<Vec<_>>(),
            vec![&[33, 0, 20, 0, 5, 0, 0, 0][..]]
        );
    }
    let consumed = g.snapshot();
    assert_eq!(
        consumed.gameplay.as_ref().unwrap().inventory,
        boost_before.gameplay.as_ref().unwrap().inventory
    );
    assert_eq!(
        consumed.gameplay.as_ref().unwrap().profiles,
        boost_before.gameplay.as_ref().unwrap().profiles
    );
    assert!(
        g.handle_at(&wrap(&use_item), now + 8)
            .unwrap()
            .replies
            .is_empty()
    );
    assert_eq!(g.snapshot(), consumed);
    let mut last = race.start_position;
    let mut last_packet = Vec::new();
    let mut final_replies = Vec::new();
    for lap in 0..race.laps {
        for (i, gate) in race.gates.iter().enumerate() {
            let distance = last
                .iter()
                .zip(gate.position)
                .map(|(a, b)| (*a - b).powi(2))
                .sum::<f32>()
                .sqrt();
            now += (distance as f64 / 30.0 * 1000.0).ceil() as u64 + 100;
            let mut packet = vec![33, 0, 7, 0];
            packet.extend(1u64.to_le_bytes());
            for x in gate.position {
                packet.extend(x.to_le_bytes());
            }
            packet.extend(1.0f32.to_le_bytes());
            let final_gate = lap + 1 == race.laps && i + 1 == race.gates.len();
            if final_gate {
                let staged_before = g.snapshot();
                // An otherwise valid finish with a full wallet keeps the
                // offer unpaid and retryable; it must not consume the offer.
                let game = staged_before.gameplay.as_ref().unwrap();
                let mut adapter =
                    osfr_tools::kart_gateway::KartGateway::load(&content, game.kart_entry.clone())
                        .unwrap()
                        .unwrap();
                let mut inventory_state = game.inventory.clone();
                inventory_state.coins = i32::MAX as u32;
                let mut inventory =
                    osfr_tools::inventory::Inventory::load(&content, Some(inventory_state))
                        .unwrap();
                let mut profiles =
                    osfr_tools::profiles::Profiles::load(&content, game.profiles.clone()).unwrap();
                adapter.handle(&packet, now).unwrap();
                let pending = adapter.state().clone();
                let bank = inventory.state().clone();
                let jobs = profiles.state().clone();
                assert!(
                    adapter
                        .claim_reward(&mut inventory, &mut profiles)
                        .unwrap()
                        .is_empty()
                );
                assert_eq!(adapter.state(), &pending);
                assert_eq!(inventory.state(), &bank);
                assert_eq!(profiles.state(), &jobs);
                let first = wrap(&packet);
                let bad = wrap(&[33, 0, 9, 0]);
                let mut bundle = vec![0, 25, first.len() as u8];
                bundle.extend(first);
                bundle.push(bad.len() as u8);
                bundle.extend(bad);
                assert!(g.handle_at(&bundle, now).is_err());
                assert_eq!(g.snapshot(), staged_before);
            }
            let out = g.handle_at(&wrap(&packet), now).unwrap();
            assert!(out.gameplay_changed, "lap {lap} gate {i}");
            let session = g
                .snapshot()
                .gameplay
                .unwrap()
                .kart_entry
                .unwrap()
                .race
                .unwrap();
            assert_eq!(
                session.session.laps_completed,
                if i + 1 == race.gates.len() {
                    lap + 1
                } else {
                    lap
                }
            );
            if final_gate {
                final_replies = out.replies.clone();
                last_packet = packet;
            }
            last = gate.position;
        }
    }
    let after = g.snapshot();
    let after_game = after.gameplay.as_ref().unwrap();
    assert_eq!(after_game.inventory.coins, coins + reward.coins);
    assert_eq!(
        after_game.profiles.as_ref().unwrap().experience[&48],
        xp + reward.xp
    );
    assert!(
        final_replies
            .iter()
            .any(|r| r[7..].starts_with(&[50, 0, 1]))
    );
    assert!(
        final_replies
            .iter()
            .any(|r| r[7..].starts_with(&[38, 0, 19, 0]))
    );
    assert!(
        final_replies
            .iter()
            .any(|r| r[7..].starts_with(&[38, 0, 14, 0]))
    );
    let inners: Vec<_> = final_replies.iter().map(|r| &r[7..]).collect();
    let toast = inners.iter().find(|r| r.starts_with(&[50, 0, 1])).unwrap();
    assert_eq!(
        u32::from_le_bytes(toast[4..8].try_into().unwrap()),
        reward.coins
    );
    assert_eq!(
        u32::from_le_bytes(toast[8..12].try_into().unwrap()),
        reward.xp
    );
    let coins_packet = inners
        .iter()
        .find(|r| r.starts_with(&[38, 0, 19, 0]))
        .unwrap();
    assert_eq!(
        u32::from_le_bytes(coins_packet[4..8].try_into().unwrap()),
        coins + reward.coins
    );
    let rank = inners
        .iter()
        .find(|r| r.starts_with(&[38, 0, 18, 0]))
        .unwrap();
    assert_eq!(&rank[4..12], &[48, 0, 0, 0, 2, 0, 0, 0]);
    let xp_packet = inners
        .iter()
        .find(|r| r.starts_with(&[38, 0, 14, 0]))
        .unwrap();
    assert_eq!(&xp_packet[4..16], &[48, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0]);
    let mut corrupt = after_game.kart_entry.clone().unwrap();
    corrupt.race.as_mut().unwrap().reward_taken = false;
    assert!(osfr_tools::kart_gateway::KartGateway::load(&content, Some(corrupt)).is_err());
    assert!(
        g.handle_at(&wrap(&last_packet), now + 100)
            .unwrap()
            .replies
            .is_empty()
    );
    assert_eq!(g.snapshot(), after);
    // Native Close returns to the world and never claims a second award.
    let mut close = vec![39, 0, 7];
    close.extend(cfg.context.encounter_id.to_le_bytes());
    close.extend([255; 8]);
    let mut closing = g.clone();
    let returned = closing.handle_at(&wrap(&close), now + 200).unwrap();
    assert!(
        returned
            .replies
            .iter()
            .any(|r| zone_transition::decode(r).is_ok())
    );
    assert_eq!(closing.state(), &before.character);
    assert_eq!(
        closing.snapshot().gameplay.as_ref().unwrap().inventory,
        after_game.inventory
    );
    let path = root.join(format!(
        "finish-save-{}-{}.json",
        std::process::id(),
        different_lane
    ));
    game_store::save(&path, &after).unwrap();
    let saved = game_store::load_or_create(&path, after.character.clone()).unwrap();
    fs::remove_file(path).unwrap();
    let mut reconnect =
        Gateway::load_gameplay(&reference, saved.character, &content, saved.gameplay).unwrap();
    assert_eq!(reconnect.state(), &before.character);
    initialize(&mut reconnect);
    reconnect.tick_at(1000).unwrap();
    let restored = reconnect.snapshot().gameplay.unwrap();
    assert_eq!(restored.inventory, after_game.inventory);
    assert_eq!(restored.profiles, after_game.profiles);
    assert!(restored.kart_entry.unwrap().race.unwrap().reward_taken);
}
