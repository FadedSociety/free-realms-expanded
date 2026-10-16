//! W147 (W134 brief #5): native free revive over 41/122.
//!
//! Recovered facts under test (client `FreeRealms.exe`): 41/122 is the 12-byte
//! encounter header plus one bool (`0093bd80` free, `0093be30` paid); 41/125 is
//! the respawn window (`0090d830` -> `00904300`); 41/126 hides it. Everything
//! else here (hold the knockout, checkpoint destination, refusing paid revive,
//! grace) is reconstructed policy (R).

use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    encounter::{EncounterEngine, EncounterOutcome, EncounterPhase},
    gameplay::GameplayState,
    gateway::Gateway,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static SERIAL: AtomicU64 = AtomicU64::new(0);
const WOLF: u64 = 100;
const CHECKPOINT: [f32; 3] = [30.0, 0.0, 40.0];
const FACING: [f32; 3] = [0.0, 0.0, 1.0];

struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
impl Fixture {
    /// A wolf at the origin that kills a level-1 player in one blow, and a
    /// one-enemy cave for the instance knockout.
    fn new() -> Self {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "w147-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        let mut template = vec![0; 334];
        template[..4].copy_from_slice(&[35, 0, 2, 0]);
        template[133..137].copy_from_slice(&100_i32.to_le_bytes());
        template[224] = 1;
        fs::write(dir.join("npc.bin"), &template).unwrap();
        let zone = json!({"zone_id":0,"name":"SyntheticWorld","sky":"","geometry_id":214,"position":CHECKPOINT,"rotation":FACING,"tutorial":false});
        let mut cave = zone.clone();
        cave["zone_id"] = json!(411);
        cave["position"] = json!([0, 0, 0]);
        let wolf = json!({"guid":WOLF,"definition_id":400,"model_id":3,"name_id":100,"name":"Synthetic wolf","texture_alias":"","position":[0,0,0],"max_health":100,"damage":1000,"boss":false,"level":1,"level_band":[1,20],"zone_id":0,"region":"Synthetic region","spawn_area":{"id":"test","centre":[0,0,0],"radius":0,"count":1},"leash_radius":12,"respawn_seconds":1,"aggro_range":8});
        let mut cave_enemy = wolf.clone();
        cave_enemy["guid"] = json!(200);
        fs::write(dir.join("encounters.json"), json!({"schema_version":1,"template_file":"npc.bin","template_sha256":format!("{:x}",Sha256::digest(&template)),"world":zone,"world_enemies":[],"dungeons":[{"id":411,"title_id":12,"description_id":13,"objective_id":14,"zone":cave,"guide":null,"enemies":[cave_enemy],"reward_coins":0,"reward_items":[]}]}).to_string()).unwrap();
        fs::write(
            dir.join("field_enemies.json"),
            json!({"schema_version":1,"enemies":[wolf]}).to_string(),
        )
        .unwrap();
        Self(dir)
    }
    fn load(&self) -> EncounterEngine {
        let mut e = EncounterEngine::load(&self.0, None).unwrap();
        e.load_field_enemies(&self.0.join("field_enemies.json"))
            .unwrap();
        e
    }
}

/// 41/122 as the client builds it: header words (u16 41, u16 122, i32, i32)
/// and the trailing bool at packet offset 0x14.
fn resume(paid: u8, encounter: i32, instance: i32) -> Vec<u8> {
    [
        vec![41, 0, 122, 0],
        encounter.to_le_bytes().to_vec(),
        instance.to_le_bytes().to_vec(),
        vec![paid],
    ]
    .concat()
}
fn free() -> Vec<u8> {
    resume(0, 0, 0)
}
fn window() -> Vec<u8> {
    [
        vec![41, 0, 125, 0],
        vec![0; 8],
        10_000_i32.to_le_bytes().to_vec(),
        i32::MAX.to_le_bytes().to_vec(),
    ]
    .concat()
}
fn hide() -> Vec<u8> {
    [vec![41, 0, 126, 0], vec![0; 8]].concat()
}
fn hp(health: i32, max: i32) -> Vec<u8> {
    [
        vec![38, 0, 1, 0],
        health.to_le_bytes().to_vec(),
        max.to_le_bytes().to_vec(),
    ]
    .concat()
}
fn count(out: &EncounterOutcome, packet: &[u8]) -> usize {
    out.replies
        .iter()
        .filter(|p| p.as_slice() == packet)
        .count()
}
fn untouched(out: &EncounterOutcome) -> bool {
    !out.changed
        && out.replies.is_empty()
        && out.zoning.is_none()
        && out.feedback.is_none()
        && out.rewards.is_empty()
        && out.completed.is_none()
}
fn send(e: &mut EncounterEngine, packet: &[u8], now: u64) -> EncounterOutcome {
    e.handle(packet, 1, [0.0; 3], 43, now).unwrap()
}
/// The wolf hits for far more than the player's health.
fn knocked_out(f: &Fixture) -> (EncounterEngine, EncounterOutcome) {
    let mut e = f.load();
    e.on_zone_ready(11, 0).unwrap();
    let out = e.tick([0.0; 3], 11, 100).unwrap();
    assert_eq!(e.state().player_health, 0);
    (e, out)
}

#[test]
fn world_knockout_is_held_and_offers_the_window_once() {
    let f = Fixture::new();
    let (mut e, out) = knocked_out(&f);
    assert!(out.changed && out.zoning.is_none());
    assert_eq!(count(&out, &window()), 1);
    assert_eq!(count(&out, &hp(0, 1000)), 1);
    assert_eq!(window().len(), 20); // 12-byte family header + timer + cost (`008d6730`)
    // Dead players are not hit again and the window is not re-sent.
    let before = e.state().clone();
    for step in 2..60 {
        let later = e.tick([0.0; 3], 11, step * 100).unwrap();
        assert!(untouched(&later), "step {step}");
    }
    assert_eq!(e.state(), &before);
    // Nothing the dead player sends acts: attacks and a new battle are refused.
    assert!(untouched(&e.attack(WOLF, [0.0; 3], 11, 0, 9_000).unwrap()));
    assert!(e.begin(411, location(), 11).is_err());
    assert_eq!(e.state(), &before);
}

/// Lead (R): if the client never sends 41/122 (for example the window does
/// not show), the server revives the player itself after 15 s.
#[test]
fn unanswered_knockout_is_revived_by_the_server() {
    let f = Fixture::new();
    let (mut e, _) = knocked_out(&f);
    // The first tick that sees the knockout starts the 15 s fallback clock.
    assert!(untouched(&e.tick([0.0; 3], 11, 200).unwrap()));
    // Canary: just before the deadline nothing happens.
    assert!(untouched(&e.tick([0.0; 3], 11, 15_100).unwrap()));
    assert_eq!(e.state().player_health, 0);
    let out = e.tick([0.0; 3], 11, 15_200).unwrap();
    assert!(out.changed);
    assert_eq!(e.state().player_health, 1000);
    let zone = out.zoning.as_ref().expect("safe location");
    assert_eq!((zone.zone_id, zone.position), (0, CHECKPOINT));
    assert_eq!(count(&out, &hide()), 1);
}

#[test]
fn free_revive_returns_to_the_checkpoint_at_full_health_with_grace() {
    let f = Fixture::new();
    let (mut e, _) = knocked_out(&f);
    let out = send(&mut e, &free(), 5_000);
    assert!(out.changed);
    let zone = out.zoning.as_ref().expect("safe location");
    assert_eq!(
        (zone.zone_id, zone.position, zone.rotation),
        (0, CHECKPOINT, FACING)
    );
    assert_eq!(count(&out, &hp(1000, 1000)), 1);
    assert_eq!(count(&out, &hide()), 1);
    assert_eq!(e.state().player_health, 1000);
    assert!(e.state().active.is_none());
    // Eight seconds of grace: the wolf standing on the spot cannot kill again.
    let _ = e.on_zone_ready(11, 5_000).unwrap();
    for step in 0..70 {
        e.tick([0.0; 3], 11, 5_100 + step * 100).unwrap();
        assert_eq!(e.state().player_health, 1000, "inside grace, step {step}");
    }
    e.tick([0.0; 3], 11, 13_300).unwrap();
    assert_eq!(e.state().player_health, 0, "grace is finite");
}

#[test]
fn replay_and_alive_requests_change_nothing() {
    let f = Fixture::new();
    // Alive: the button cannot be pressed, a forged packet does nothing.
    let mut e = f.load();
    e.on_zone_ready(11, 0).unwrap();
    let before = e.state().clone();
    for paid in [0, 1] {
        assert!(
            untouched(&send(&mut e, &resume(paid, 0, 0), 50)),
            "alive paid={paid}"
        );
    }
    assert_eq!(e.state(), &before);
    // Knocked out once, revived once; replaying the same packet is inert.
    let (mut e, _) = knocked_out(&f);
    assert!(send(&mut e, &free(), 5_000).changed);
    let after = e.state().clone();
    assert!(untouched(&send(&mut e, &free(), 5_001)));
    assert!(untouched(&send(&mut e, &resume(1, 0, 0), 5_002)));
    assert_eq!(e.state(), &after);
}

#[test]
fn paid_revive_is_refused_and_the_window_comes_back() {
    let f = Fixture::new();
    let (mut e, _) = knocked_out(&f);
    let before = e.state().clone();
    let out = send(&mut e, &resume(1, 0, 0), 2_000);
    assert!(!out.changed && out.zoning.is_none() && out.rewards.is_empty());
    assert_eq!(e.state(), &before); // no health, no relocation, no currency
    assert_eq!(out.replies, vec![window()]); // the window hid itself on the click
    assert!(out.feedback.as_deref().unwrap().contains("not available"));
    // Free still works afterwards.
    assert!(send(&mut e, &free(), 2_100).zoning.is_some());
}

#[test]
fn client_supplied_ids_carry_no_authority_in_the_world() {
    // Nothing the client sends picks the place: the packet has no coordinates,
    // and header ids (the native free button may echo a stale instance) are not
    // read outside an instance.
    let f = Fixture::new();
    let (mut e, _) = knocked_out(&f);
    let out = send(&mut e, &resume(0, 411, 77), 2_000);
    assert_eq!(out.zoning.unwrap().position, CHECKPOINT);
}

#[test]
fn malformed_resume_is_rejected_without_mutation() {
    let f = Fixture::new();
    for down in [false, true] {
        let (mut e, _) = if down {
            knocked_out(&f)
        } else {
            let mut e = f.load();
            e.on_zone_ready(11, 0).unwrap();
            (e, EncounterOutcome::default())
        };
        let before = e.state().clone();
        let mut long = free();
        long.push(0);
        for bad in [
            free()[..12].to_vec(),
            free()[..4].to_vec(),
            long,
            resume(2, 0, 0),
            resume(255, 0, 0),
        ] {
            assert!(
                e.handle(&bad, 1, [0.0; 3], 43, 1_000).is_err(),
                "down={down} len={} flag={}",
                bad.len(),
                bad.last().unwrap()
            );
        }
        assert_eq!(e.state(), &before, "down={down}");
    }
}

#[test]
fn a_knockout_saved_to_disk_wakes_up_on_reload() {
    let f = Fixture::new();
    let (e, _) = knocked_out(&f);
    // The gateway saves after the knockout; the window is not persisted.
    let reloaded = EncounterEngine::load(&f.0, Some(e.state().clone())).unwrap();
    assert_eq!(reloaded.state().player_health, 1000);
    assert!(reloaded.state().active.is_none());
    // The relaxed check is only for the world: a fighting instance at zero
    // health is still an inconsistent save.
    let mut fighting = f.load();
    fighting.begin(411, location(), 43).unwrap();
    let mut state = fighting.state().clone();
    state.player_health = 0;
    assert!(EncounterEngine::load(&f.0, Some(state)).is_err());
}

#[test]
fn lost_instance_revives_through_the_existing_exit() {
    let f = Fixture::new();
    let lose = || {
        let mut e = f.load();
        e.begin(411, location(), 43).unwrap();
        e.on_zone_ready(43, 0).unwrap();
        // The battle arena armed itself; the cave enemy kills in one hit.
        e.tick([0.0; 3], 43, 100).unwrap();
        e.tick([0.0; 3], 43, 300).unwrap();
        assert_eq!(e.state().player_health, 0);
        assert_eq!(
            e.state().active.as_ref().unwrap().phase,
            EncounterPhase::Lost
        );
        e
    };
    let mut e = lose();
    let instance = e.state().active.as_ref().unwrap().instance_id;
    let before = e.state().clone();
    // Paid: refused, nothing moves, no world window over the results screen.
    let paid = send(&mut e, &resume(1, 0, 0), 1_000);
    assert!(!paid.changed && paid.zoning.is_none() && paid.replies.is_empty());
    assert!(paid.feedback.is_some());
    // A header naming another instance is a replay from an earlier one.
    for (encounter, other) in [(0, instance + 5), (999, instance), (0, -1)] {
        assert!(untouched(&send(
            &mut e,
            &resume(0, encounter, other),
            1_100
        )));
    }
    assert_eq!(e.state(), &before);
    // The current instance (zero header, as the native free packet sends, or
    // the exact ids) uses the same exit as 41/109: return place, full health.
    for header in [(0, 0), (411, instance)] {
        let mut e = lose();
        let out = send(&mut e, &resume(0, header.0, header.1), 1_200);
        let zone = out.zoning.as_ref().unwrap();
        assert_eq!((zone.zone_id, zone.position), (0, location().position));
        assert!(out.completed.is_none() && out.rewards.is_empty());
        assert!(e.state().active.is_none());
        assert_eq!(e.state().player_health, 1000);
        assert!(untouched(&send(
            &mut e,
            &resume(0, header.0, header.1),
            1_300
        )));
    }
    // Alive in a fighting instance: the request is inert.
    let mut alive = f.load();
    alive.begin(411, location(), 43).unwrap();
    alive.on_zone_ready(43, 0).unwrap();
    let before = alive.state().clone();
    assert!(untouched(&send(&mut alive, &free(), 10)));
    assert_eq!(alive.state(), &before);
}

#[test]
fn developer_heal_hides_a_stale_window() {
    let f = Fixture::new();
    let (mut e, _) = knocked_out(&f);
    assert!(e.heal_full().contains(&hide()));
    assert!(!e.heal_full().contains(&hide()));
}

fn location() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: [9.0, 2.0, 3.0],
        rotation: [0.1, 0.2, 0.3],
    }
}

// ---- Gateway route: the same request through login, tunnel and gameplay. ----
// Uses the private five-movie content and gateway reference like app_fuzz.

const NOW: u64 = 10_000;
fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}
fn login() -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    for (i, text) in ["w147-synthetic-ticket", "1.910.1.530630", ""]
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
fn inner_replies(replies: &[Vec<u8>]) -> Vec<&[u8]> {
    replies.iter().filter_map(|p| p.get(7..)).collect()
}

#[test]
fn gateway_routes_resume_for_a_held_world_knockout() {
    let lab = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
    let source = lab.join("reports/round59-native-20261015T124443Z/content");
    let reference = lab.join("references/rust-gateway-reference");
    assert!(
        source.join("harvesting.json").is_file(),
        "private content fixture"
    );
    let content = lab.join("build/w147-content").join(format!(
        "{}-{}",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&content).unwrap();
    for entry in fs::read_dir(&source).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file()
            && matches!(
                path.extension().and_then(|s| s.to_str()),
                Some("json" | "bin")
            )
        {
            fs::copy(&path, content.join(path.file_name().unwrap())).unwrap();
        }
    }
    let result = std::panic::catch_unwind(|| route(&content, &reference));
    let _ = fs::remove_dir_all(&content);
    result.unwrap();
}

fn route(content: &std::path::Path, reference: &std::path::Path) {
    let mut character = Gateway::reference_state(reference).unwrap();
    character.position = [0.; 3];
    let state: GameplayState = osfr_tools::gameplay::Gameplay::load(content, None)
        .unwrap()
        .state();
    let mut gateway = Gateway::load_gameplay(reference, character, content, Some(state)).unwrap();
    gateway.set_dev_tools(false);
    gateway.set_role(AccountRole::Player);
    gateway.set_trace_enabled(true);
    gateway.handle_at(&login(), NOW).unwrap();
    gateway.handle_at(&wrap(&[13, 0]), NOW).unwrap();
    gateway.handle_at(&wrap(&[10, 0]), NOW).unwrap();
    let health = |g: &Gateway| {
        g.snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .player_health
    };
    let full = health(&gateway);
    assert!(full > 0);

    // Alive: the request is routed and ignored.
    let alive = gateway.handle_at(&wrap(&free()), NOW + 1).unwrap();
    assert!(!alive.began_zoning && !alive.gameplay_changed);
    assert_eq!(health(&gateway), full);

    // The shared-field damage path (the live server's) knocks the player out
    // and offers the window; a second lethal tick is silent.
    let down = gateway
        .receive_field_damage(u32::MAX, NOW + 2, true)
        .unwrap();
    assert!(
        down.replies.iter().any(|r| r == &window()),
        "41/125 offered"
    );
    assert_eq!(health(&gateway), 0);
    let again = gateway.receive_field_damage(500, NOW + 3, true).unwrap();
    assert!(again.replies.is_empty() && !again.changed);

    // Paid: refused over the real route, state untouched, window re-shown.
    let paid = gateway.handle_at(&wrap(&resume(1, 0, 0)), NOW + 4).unwrap();
    assert!(!paid.began_zoning);
    assert!(inner_replies(&paid.replies).contains(&window().as_slice()));
    assert!(
        !paid.trace.iter().any(|t| t.outcome == "rejected"),
        "{:?}",
        paid.trace
    );
    assert_eq!(health(&gateway), 0);

    // Free: full health, zoning to the world checkpoint, window hidden.
    let revived = gateway.handle_at(&wrap(&free()), NOW + 5).unwrap();
    assert!(revived.began_zoning && revived.gameplay_changed);
    let inner = inner_replies(&revived.replies);
    assert!(inner.contains(&hide().as_slice()));
    assert!(inner.contains(&hp(full as i32, full as i32).as_slice()));
    assert!(
        !revived.trace.iter().any(|t| t.outcome == "rejected"),
        "{:?}",
        revived.trace
    );
    assert_eq!(health(&gateway), full);

    // Replay after the revive does nothing.
    let replay = gateway.handle_at(&wrap(&free()), NOW + 6).unwrap();
    assert!(!replay.began_zoning && !replay.gameplay_changed);
}
