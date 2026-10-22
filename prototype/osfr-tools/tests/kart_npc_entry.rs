//! W149 (brief #3): ordinary kart entry from an NPC menu choice, at the `KartGateway` layer.
//!
//! ORIGINAL facts relied on (nothing here is a new recovered fact):
//! * the player picks an option from the 26/9 menu and the client answers 26/10
//!   (`notes/Research/NPC Interaction.md`; senders `00a36f80`/`00a39090`); the client sends nothing
//!   else to "start a race" - 102/1 (`00a27790`) is only the lobby catalogue request.
//! * the race itself (33/x register, 41/108 Go, 39/7 close, once-only payout) is the already
//!   reviewed flow in `kart_gateway`/`kart_service`/`kart_rewards`.
//!
//! RECONSTRUCTED (R): the gate NPC list, the menu label id, and every refusal in `admit`.
//! No game bytes, player names or chat appear here; all content is synthetic.
use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState},
    inventory::Inventory,
    kart_entry::{ProxiedVehicle, RaceConfig, RaceEntryContent, RaceEntryContext},
    kart_gateway::{
        DEFAULT_MENU_STRING_ID, EntryDenied, KartGateway, KartGatewayContent, MAX_GATES,
    },
    kart_progress_native::ProgressPayload,
    kart_racing::RewardSchedule,
    kart_service::{
        GateWaypoint, KartServiceContent, RaceBounds, RacePhase, RaceStep, ScheduledFrame,
    },
    profiles::Profiles,
    zone_transition::ZoneTransition,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

#[path = "common/content_copy.rs"]
mod content_copy;

use content_copy::copy_tree;

static NEXT: AtomicU64 = AtomicU64::new(0);
const GATE_NPC: u64 = 9_000_000_273;
const OTHER_NPC: u64 = 9_000_000_274;
const PLAYER: u64 = 1;
const KART_JOB: i32 = 48;
const PRIOR_JOB: i32 = 43;
const T0: u64 = 100_000;

struct Fixture(PathBuf);
impl Fixture {
    fn new(race: bool) -> Self {
        let path = std::env::temp_dir().join(format!(
            "kart-npc-entry-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        let fixture = Self(path);
        fixture.write_content(race);
        fixture
    }
    fn write_content(&self, race: bool) {
        let mut content = content();
        if race {
            content.race = Some(race_content());
        }
        self.write(&content);
    }
    fn write(&self, content: &KartGatewayContent) {
        fs::write(
            self.0.join("kart_entry.json"),
            serde_json::to_vec(content).unwrap(),
        )
        .unwrap();
    }
    fn write_gates(&self, npcs: &[u64]) {
        self.write_gates_raw(
            &serde_json::to_vec(&json!({"schema_version": 1, "npcs": npcs})).unwrap(),
        );
    }
    fn write_gates_raw(&self, bytes: &[u8]) {
        let path = self.0.join("kart_gates.json");
        let _ = fs::remove_dir(&path);
        fs::write(path, bytes).unwrap();
    }
    fn load(&self) -> KartGateway {
        KartGateway::load(&self.0, None).unwrap().unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn character() -> CharacterState {
    CharacterState {
        guid: PLAYER,
        zone_id: 0,
        position: [10., 20., 30.],
        rotation: [0., 0., 1.],
    }
}

/// Same synthetic entry package as `tests/kart_gateway.rs` (one owned vehicle, no AI racers).
fn content() -> KartGatewayContent {
    let context = RaceEntryContext {
        session: 123,
        instance_id: 1008,
        encounter_id: 917566,
        scene: ZoneTransition {
            envelope_flag: 1,
            scene: "sg_kt_track".into(),
            type_id: 2,
            position: [719., 72., 421., 1.],
            rotation: [0., 0., 1., 0.],
            sky: "sky.xml".into(),
            geometry_id: 101,
            ..Default::default()
        },
    };
    let details = |state: u32| {
        let mut b = vec![0u8; 391];
        b[..4].copy_from_slice(&[41, 0, 114, 0]);
        for (offset, word) in [
            (4, context.encounter_id),
            (8, context.instance_id),
            (12, context.encounter_id),
            (16, state),
        ] {
            b[offset..offset + 4].copy_from_slice(&word.to_le_bytes());
        }
        b[32..40].copy_from_slice(&context.session.to_le_bytes());
        for i in 0..3 {
            b[51 + i * 4..55 + i * 4].copy_from_slice(&context.scene.position[i].to_le_bytes());
        }
        b
    };
    KartGatewayContent {
        schema_version: 1,
        details_loading: details(2),
        details_ready: details(4),
        vehicle_guid_offset: 0,
        race: None,
        entry: RaceEntryContent {
            race_config: RaceConfig { words: vec![0; 21] },
            entry_vehicles: vec![ProxiedVehicle {
                body_asset: "test.cdt".into(),
                first_attribute: 400.,
                attribute_block: 123u64.to_le_bytes().to_vec(),
            }],
            client_data: vec![0; 85],
            player_count: 1,
            npc_count: 0,
            progress: vec![ProgressPayload {
                racer_guid: 123,
                slot: 0,
                unknown_0c_bits: 0,
                unknown_10_bits: 0,
                laps_completed: 0,
                local_racer: 0,
                lap_limit: 2,
                unknown_1d_bits: 0,
                best_lap_bits: 0,
                total_time_bits: 0,
                tail_0x29: 123,
                tail_0x31: 0,
                tail_0x39: 0,
                tail_0x41: 0,
            }],
        },
        context,
    }
}

/// Synthetic two-lap race: four gates 10 m apart, the recovered countdown cadence, 40 coins / 60 xp.
fn race_content() -> KartServiceContent {
    let step = |at_ms, frame| RaceStep { at_ms, frame };
    KartServiceContent {
        speedway: 31,
        laps: 2,
        reward: Some(RewardSchedule { coins: 40, xp: 60 }),
        gates: (1..=4)
            .map(|i| GateWaypoint {
                position: [i as f32 * 10.0, 0.0, 0.0],
                radius: 3.0,
            })
            .collect(),
        schedule: vec![
            step(3_026, ScheduledFrame::GameState(4)),
            step(3_026, ScheduledFrame::NpcCount(0)),
            step(3_353, ScheduledFrame::GameState(5)),
            step(8_246, ScheduledFrame::GameState(6)),
            step(8_694, ScheduledFrame::GameState(7)),
            step(9_290, ScheduledFrame::GameState(8)),
            step(9_290, ScheduledFrame::Countdown(4)),
            step(10_074, ScheduledFrame::Countdown(3)),
            step(10_529, ScheduledFrame::Countdown(2)),
            step(11_954, ScheduledFrame::Countdown(1)),
            step(13_241, ScheduledFrame::Countdown(0)),
            step(14_505, ScheduledFrame::GameState(9)),
        ],
        progress_template: ProgressPayload {
            racer_guid: 0,
            slot: 0,
            unknown_0c_bits: 0,
            unknown_10_bits: 0x7f7f_ffff,
            laps_completed: 0,
            local_racer: 1,
            lap_limit: 2,
            unknown_1d_bits: 0,
            best_lap_bits: 0,
            total_time_bits: 0,
            tail_0x29: 0,
            tail_0x31: 0,
            tail_0x39: 0,
            tail_0x41: 0,
        },
        start_position: [0.0, 0.0, 0.0],
        bounds: RaceBounds {
            max_speed_mps: 60.0,
            max_step_ms: 60_000,
            max_race_ms: 600_000,
            max_claims: 64,
            max_observations: 4_096,
        },
        gate_band: None,
        gate_curtains: Vec::new(),
        pickups: Vec::new(),
        pickup_respawn: None,
        weapon_effects: false,
    }
}

fn register(guid: u64) -> Vec<u8> {
    let mut b = vec![33, 0, 9, 0];
    b.extend(guid.to_le_bytes());
    b.extend([0; 8]);
    b
}

/// Native Go: 41/108 + encounter id + instance id + owned guid (20 bytes).
fn go(encounter: u32, instance: u32, guid: u64) -> Vec<u8> {
    let mut b = vec![41, 0, 108, 0];
    b.extend(encounter.to_le_bytes());
    b.extend(instance.to_le_bytes());
    b.extend(guid.to_le_bytes());
    b
}

/// 33/7 gate hit: `u64 guid | 3 x f32 position | f32 1.0`.
fn gate_hit(guid: u64, gate: f32) -> Vec<u8> {
    let mut b = vec![33, 0, 7, 0];
    b.extend(guid.to_le_bytes());
    for v in [gate * 10.0, 0.0, 0.0, 1.0] {
        b.extend(f32::to_le_bytes(v));
    }
    b
}

/// 39/7 close: encounter id then two -1 words.
fn close(encounter: u32) -> Vec<u8> {
    let mut b = vec![39, 0, 7];
    b.extend(encounter.to_le_bytes());
    b.extend([255; 8]);
    b
}

/// Starts as the gameplay route will: admit, then begin + remember the prior job.
fn enter(g: &mut KartGateway) {
    g.admit(GATE_NPC, false).unwrap();
    g.begin(&character()).unwrap();
    g.set_previous_job(PRIOR_JOB);
}

// ---- availability --------------------------------------------------------------------------

#[test]
fn no_gate_file_means_no_npc_offers_the_race() {
    let f = Fixture::new(true);
    let g = f.load();
    assert!(g.gate_npcs().is_empty());
    assert_eq!(g.menu_string_id(), DEFAULT_MENU_STRING_ID);
    for npc in [0, 1, GATE_NPC, u64::MAX] {
        assert_eq!(g.admit(npc, false), Err(EntryDenied::NotAGate), "npc {npc}");
    }
}

#[test]
fn gates_without_race_content_are_never_offered() {
    // A package with an entry but no completable race must not reach ordinary players.
    let f = Fixture::new(false);
    f.write_gates(&[GATE_NPC]);
    let g = f.load();
    assert!(g.gate_npcs().is_empty());
    assert_eq!(g.admit(GATE_NPC, false), Err(EntryDenied::NoRace));
    // An unlisted NPC is still just "not a gate", not a hint that content is missing.
    assert_eq!(g.admit(OTHER_NPC, false), Err(EntryDenied::NotAGate));
}

#[test]
fn listed_gate_is_admitted_and_everything_else_is_refused_without_moving_state() {
    let f = Fixture::new(true);
    f.write_gates(&[GATE_NPC]);
    let g = f.load();
    let before = g.state().clone();
    assert_eq!(g.gate_npcs(), &[GATE_NPC]);
    assert_eq!(g.admit(GATE_NPC, false), Ok(()));
    // Canaries: neighbour id, zero, max, and a forged id that merely contains the gate id.
    for npc in [OTHER_NPC, GATE_NPC - 1, 0, u64::MAX, GATE_NPC << 1] {
        assert_eq!(g.admit(npc, false), Err(EntryDenied::NotAGate), "npc {npc}");
    }
    assert_eq!(g.state(), &before, "admission must not mutate anything");
    assert!(!g.active());
}

#[test]
fn busy_and_already_racing_are_refused_and_leaving_reopens_the_gate() {
    let f = Fixture::new(true);
    f.write_gates(&[GATE_NPC]);
    let mut g = f.load();
    assert_eq!(g.admit(GATE_NPC, true), Err(EntryDenied::Busy));
    assert!(!g.active(), "a busy refusal must not start a race");
    enter(&mut g);
    // Re-choosing the option while racing (replay) is refused and precedes the busy check.
    assert_eq!(g.admit(GATE_NPC, false), Err(EntryDenied::AlreadyRacing));
    assert_eq!(g.admit(GATE_NPC, true), Err(EntryDenied::AlreadyRacing));
    let mid_race = g.state().clone();
    let _ = g.admit(GATE_NPC, false);
    assert_eq!(g.state(), &mid_race);
    assert_eq!(g.exit(), Some(character()));
    assert_eq!(g.admit(GATE_NPC, false), Ok(()));
}

#[test]
fn refusals_carry_distinct_player_readable_messages() {
    let all = [
        EntryDenied::NotAGate,
        EntryDenied::NoRace,
        EntryDenied::AlreadyRacing,
        EntryDenied::Busy,
        EntryDenied::Cooldown,
    ];
    let texts: BTreeSet<_> = all.iter().map(|d| d.message()).collect();
    assert_eq!(texts.len(), all.len());
    assert!(texts.iter().all(|t| !t.is_empty() && t.len() < 80));
}

// ---- gate file -----------------------------------------------------------------------------

#[test]
fn menu_label_defaults_and_can_be_overridden() {
    let f = Fixture::new(true);
    f.write_gates_raw(br#"{"schema_version":1,"menu_string_id":3040,"npcs":[5]}"#);
    assert_eq!(f.load().menu_string_id(), 3040);
    f.write_gates_raw(br#"{"schema_version":1,"npcs":[5]}"#);
    assert_eq!(f.load().menu_string_id(), DEFAULT_MENU_STRING_ID);
}

#[test]
fn malformed_gate_files_are_refused_at_load() {
    let f = Fixture::new(true);
    let too_many: Vec<u64> = (1..=MAX_GATES as u64 + 1).collect();
    let at_limit: Vec<u64> = (1..=MAX_GATES as u64).collect();
    f.write_gates(&at_limit);
    assert_eq!(f.load().gate_npcs().len(), MAX_GATES);
    let oversize = format!(
        r#"{{"schema_version":1,"npcs":[1]{}}}"#,
        " ".repeat(17 * 1024)
    );
    let bad: Vec<(&str, Vec<u8>)> = vec![
        ("zero id", br#"{"schema_version":1,"npcs":[0]}"#.to_vec()),
        (
            "duplicate",
            br#"{"schema_version":1,"npcs":[7,8,7]}"#.to_vec(),
        ),
        ("negative", br#"{"schema_version":1,"npcs":[-1]}"#.to_vec()),
        (
            "too many",
            serde_json::to_vec(&json!({"schema_version": 1, "npcs": too_many})).unwrap(),
        ),
        ("schema", br#"{"schema_version":2,"npcs":[1]}"#.to_vec()),
        (
            "label zero",
            br#"{"schema_version":1,"menu_string_id":0,"npcs":[1]}"#.to_vec(),
        ),
        (
            "unknown field",
            br#"{"schema_version":1,"npcs":[1],"free_play":true}"#.to_vec(),
        ),
        ("not json", b"{".to_vec()),
        ("oversize", oversize.into_bytes()),
    ];
    for (name, bytes) in bad {
        f.write_gates_raw(&bytes);
        assert!(
            KartGateway::load(&f.0, None).is_err(),
            "{name} must be refused"
        );
    }
    // A directory (or anything that is not a regular file) in the file's place is refused too.
    let path = f.0.join("kart_gates.json");
    fs::remove_file(&path).unwrap();
    fs::create_dir(&path).unwrap();
    assert!(KartGateway::load(&f.0, None).is_err());
    fs::remove_dir(&path).unwrap();
    // Canary: removing the bad file restores a loadable, closed gate.
    assert!(f.load().gate_npcs().is_empty());
}

#[test]
fn race_content_that_never_reaches_countdown_zero_is_refused() {
    let f = Fixture::new(true);
    let mut content = content();
    let mut race = race_content();
    race.schedule
        .retain(|step| step.frame != ScheduledFrame::Countdown(0));
    content.race = Some(race);
    f.write(&content);
    assert!(KartGateway::load(&f.0, None).is_err());

    // Canary for the fixture itself: the valid generated content loads.
    f.write_content(true);
    assert!(KartGateway::load(&f.0, None).unwrap().is_some());
}

#[test]
fn editing_the_gate_list_does_not_invalidate_a_saved_race() {
    let f = Fixture::new(true);
    f.write_gates(&[GATE_NPC]);
    let mut g = f.load();
    enter(&mut g);
    let saved = g.state().clone();
    f.write_gates(&[OTHER_NPC]);
    let restored = KartGateway::load(&f.0, Some(saved.clone()))
        .unwrap()
        .unwrap();
    assert_eq!(restored.state(), &saved);
    assert_eq!(restored.previous_job(), Some(PRIOR_JOB));
    assert_eq!(
        restored.admit(OTHER_NPC, false),
        Err(EntryDenied::AlreadyRacing)
    );
    assert_eq!(restored.admit(GATE_NPC, false), Err(EntryDenied::NotAGate));
}

// ---- hostile frames ------------------------------------------------------------------------

#[test]
fn client_race_frames_without_an_entry_move_nothing() {
    let f = Fixture::new(true);
    f.write_gates(&[GATE_NPC]);
    let mut g = f.load();
    let before = g.state().clone();
    assert!(g.handle(&register(PLAYER), T0).unwrap().is_empty());
    assert!(g.handle(&gate_hit(PLAYER, 1.0), T0).unwrap().is_empty());
    assert!(
        g.handle(&go(917566, 48_000_001, PLAYER), T0)
            .unwrap()
            .is_empty()
    );
    assert!(!g.accepts_close(&close(917566)).unwrap());
    assert!(g.tick(T0).unwrap().is_empty());
    assert_eq!(g.state(), &before);
}

// ---- the full ordinary flow ----------------------------------------------------------------

/// Synthetic profiles + inventory for the payout leg (same shape the reviewed reward tests use).
struct Economy(PathBuf);
impl Economy {
    fn new(active: i32) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "kart-npc-economy-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).unwrap();
        let digest = |b: &[u8]| format!("{:x}", Sha256::digest(b));
        let hex = |b: &[u8]| b.iter().map(|x| format!("{x:02x}")).collect::<String>();
        let mut template = vec![5, 0, 1, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0];
        template.extend(0i32.to_le_bytes());
        let mut proofs = Vec::new();
        for id in [KART_JOB, PRIOR_JOB] {
            let raw: Vec<u8> = [1i32, 0, 0, 1, 3, 0, 3, 0]
                .into_iter()
                .flat_map(i32::to_le_bytes)
                .collect();
            proofs.push(json!({
                "id": id, "offset": template.len(), "length": raw.len(),
                "sha256": digest(&raw), "serialized_hex": hex(&raw),
                "rank_offset": 0, "equipment_offset": 12, "equipment_length": 16,
            }));
            template.extend(raw);
        }
        let active_offset = template.len();
        template.extend(active.to_le_bytes());
        fs::write(
            dir.join("profiles.json"),
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "active_offset": active_offset,
                "active_sha256": digest(&active.to_le_bytes()),
                "profiles": proofs,
                "default_state": {
                    "version": 1, "active": active,
                    "unlocked": [PRIOR_JOB, KART_JOB], "experience": {},
                },
                "starter_items": {},
            }))
            .unwrap(),
        )
        .unwrap();
        let mut packet = vec![5, 0, 1, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0];
        let coins_offset = packet.len();
        packet.extend(0i32.to_le_bytes());
        let mut equipment = Vec::new();
        for id in [KART_JOB, PRIOR_JOB] {
            equipment.push((id, packet.len()));
            packet.extend(0i32.to_le_bytes());
        }
        let items_offset = packet.len();
        packet.extend(0i32.to_le_bytes());
        let length = packet.len();
        packet[3..7].copy_from_slice(&((length - 7) as u32).to_le_bytes());
        packet[9..13].copy_from_slice(&((length - 13) as u32).to_le_bytes());
        let span = |kind: &str, offset: usize, len: usize, id: i32| {
            json!({"kind": kind, "offset": offset, "length": len, "profile_id": id,
                   "sha256": digest(&packet[offset..offset + len])})
        };
        let mut spans = vec![span("coins", coins_offset, 4, 0)];
        for (id, offset) in &equipment {
            spans.push(span("equipment", *offset, 4, *id));
        }
        spans.push(span("items", items_offset, 4, 0));
        let allowed: Vec<_> = [KART_JOB, PRIOR_JOB]
            .iter()
            .map(|id| json!({"id": id, "allowed_classes": [1], "slots": []}))
            .collect();
        fs::write(
            dir.join("inventory.json"),
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "player_guid": PLAYER,
                "default_state": {"coins": 0, "items": [], "equipment": [], "next_instance_id": null},
                "profiles": allowed,
                "spans": spans,
                "definitions": [],
            }))
            .unwrap(),
        )
        .unwrap();
        Self(dir)
    }
    fn banks(&self) -> (Inventory, Profiles) {
        (
            Inventory::load(&self.0, None).unwrap(),
            Profiles::load(&self.0, None).unwrap(),
        )
    }
}
impl Drop for Economy {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn coins(inventory: &Inventory) -> u32 {
    inventory.state().coins
}
fn xp(profiles: &Profiles) -> u32 {
    profiles
        .state()
        .experience
        .get(&KART_JOB)
        .copied()
        .unwrap_or(0)
}

/// Registers, presses Go and ticks to the green light. Returns (instance, clock at green).
fn start_to_green(g: &mut KartGateway) -> (u32, u64) {
    let active = g.state().active.clone().unwrap();
    let instance = active.context.instance_id;
    let encounter = active.context.encounter_id;
    assert!(!g.handle(&register(PLAYER), T0).unwrap().is_empty());
    assert_eq!(
        g.handle(&go(encounter, instance, PLAYER), T0 + 500)
            .unwrap()
            .len(),
        1
    );
    let green = T0 + 500 + 6_000;
    g.tick(green).unwrap();
    (instance, green)
}

/// Two laps, gates 1..=4, one second apart, starting after `from`.
fn drive(g: &mut KartGateway, from: u64) -> u64 {
    let mut now = from;
    for _lap in 0..2 {
        for gate in 1..=4 {
            now += 1_000;
            g.handle(&gate_hit(PLAYER, gate as f32), now).unwrap();
        }
    }
    now
}

/// Lead (R): a solo race pays on every finish, so a gate NPC waits
/// `reward_cooldown_s` after a paid race before starting another. The time is
/// saved with the character; 0 turns it off; above one day the file is refused.
#[test]
fn gate_cooldown_after_a_paid_race_blocks_entry_until_it_passes() {
    let f = Fixture::new(true);
    f.write_gates_raw(
        &serde_json::to_vec(
            &json!({"schema_version": 1, "npcs": [GATE_NPC], "reward_cooldown_s": 600}),
        )
        .unwrap(),
    );
    let eco = Economy::new(KART_JOB);
    let (mut inventory, mut profiles) = eco.banks();
    let mut g = f.load();
    enter(&mut g);
    let (_, green) = start_to_green(&mut g);
    drive(&mut g, green);
    assert!(
        !g.claim_reward(&mut inventory, &mut profiles)
            .unwrap()
            .is_empty()
    );
    let paid = g.state().last_paid_unix.expect("a paid race is recorded");
    let active = g.state().active.clone().unwrap();
    assert!(
        g.accepts_close(&close(active.context.encounter_id))
            .unwrap()
    );
    g.exit();
    let before_refusal = g.state().clone();
    assert_eq!(
        g.admit_at(GATE_NPC, false, paid + 10),
        Err(EntryDenied::Cooldown)
    );
    assert_eq!(
        g.state(),
        &before_refusal,
        "a cooldown refusal must not consume the saved allowance"
    );
    assert_eq!(g.admit_at(GATE_NPC, false, paid + 600), Ok(()));
    // The cooldown survives a reload from the saved state.
    let reloaded = KartGateway::load(&f.0, Some(g.state().clone()))
        .unwrap()
        .unwrap();
    assert_eq!(
        reloaded.admit_at(GATE_NPC, false, paid + 10),
        Err(EntryDenied::Cooldown)
    );
    // Sanity bound: more than a day is refused at load.
    f.write_gates_raw(
        &serde_json::to_vec(
            &json!({"schema_version": 1, "npcs": [GATE_NPC], "reward_cooldown_s": 86_401}),
        )
        .unwrap(),
    );
    assert!(KartGateway::load(&f.0, None).is_err());
}

#[test]
fn ordinary_entry_finishes_pays_exactly_once_and_returns_in_the_prior_job() {
    let f = Fixture::new(true);
    f.write_gates(&[GATE_NPC]);
    let eco = Economy::new(KART_JOB);
    let (mut inventory, mut profiles) = eco.banks();
    let mut g = f.load();
    enter(&mut g);
    assert_eq!(g.previous_job(), Some(PRIOR_JOB));
    let (_, green) = start_to_green(&mut g);
    // Nothing is owed before the finish.
    assert!(
        g.claim_reward(&mut inventory, &mut profiles)
            .unwrap()
            .is_empty()
    );
    assert_eq!((coins(&inventory), xp(&profiles)), (0, 0));

    let end = drive(&mut g, green);
    let paid = g
        .claim_reward_with_event(&mut inventory, &mut profiles)
        .unwrap();
    assert!(
        !paid.replies.is_empty(),
        "a finished race must send its reward frames"
    );
    assert_eq!(
        paid.minigame_id,
        Some(31),
        "the stable speedway id is the quest completion identity"
    );
    assert_eq!((coins(&inventory), xp(&profiles)), (40, 60));

    // Replays and later claims pay nothing.
    for _ in 0..3 {
        assert!(
            g.claim_reward(&mut inventory, &mut profiles)
                .unwrap()
                .is_empty()
        );
    }
    assert_eq!(
        g.claim_reward_with_event(&mut inventory, &mut profiles)
            .unwrap()
            .minigame_id,
        None
    );
    g.handle(&gate_hit(PLAYER, 1.0), end + 1_000).unwrap();
    assert!(
        g.claim_reward(&mut inventory, &mut profiles)
            .unwrap()
            .is_empty()
    );
    assert_eq!((coins(&inventory), xp(&profiles)), (40, 60));

    // The native Close (39/7) is bound to the live encounter, returns the prior pose and job.
    let active = g.state().active.clone().unwrap();
    assert!(
        g.accepts_close(&close(active.context.encounter_id))
            .unwrap()
    );
    assert!(!g.accepts_close(&close(1)).unwrap());
    let job = g.previous_job();
    assert_eq!(g.exit(), Some(character()));
    assert_eq!(job, Some(PRIOR_JOB));
    assert!(!g.active() && g.previous_job().is_none());
    // Exiting again (a replayed close) changes nothing and pays nothing.
    assert_eq!(g.exit(), None);
    assert!(
        g.claim_reward(&mut inventory, &mut profiles)
            .unwrap()
            .is_empty()
    );
    assert_eq!((coins(&inventory), xp(&profiles)), (40, 60));
}

#[test]
fn cancelling_before_the_finish_pays_nothing_and_a_new_race_is_independent() {
    let f = Fixture::new(true);
    f.write_gates(&[GATE_NPC]);
    let eco = Economy::new(KART_JOB);
    let (mut inventory, mut profiles) = eco.banks();
    let mut g = f.load();
    enter(&mut g);
    let first_instance = g.state().active.as_ref().unwrap().context.instance_id;
    let (_, green) = start_to_green(&mut g);
    // One lap only, then quit.
    for gate in 1..=4 {
        g.handle(&gate_hit(PLAYER, gate as f32), green + 1_000 * gate as u64)
            .unwrap();
    }
    assert!(
        g.claim_reward(&mut inventory, &mut profiles)
            .unwrap()
            .is_empty()
    );
    assert_eq!(g.exit(), Some(character()));
    assert!(
        g.claim_reward(&mut inventory, &mut profiles)
            .unwrap()
            .is_empty()
    );
    assert_eq!((coins(&inventory), xp(&profiles)), (0, 0));

    // A fresh entry gets a fresh instance and starts unpaid and unfinished.
    enter(&mut g);
    let second_instance = g.state().active.as_ref().unwrap().context.instance_id;
    assert_eq!(second_instance, first_instance + 1);
    assert!(
        g.claim_reward(&mut inventory, &mut profiles)
            .unwrap()
            .is_empty()
    );
    assert_eq!((coins(&inventory), xp(&profiles)), (0, 0));
}

#[test]
fn a_disconnect_mid_race_survives_a_snapshot_with_the_prior_job() {
    let f = Fixture::new(true);
    f.write_gates(&[GATE_NPC]);
    let mut g = f.load();
    enter(&mut g);
    start_to_green(&mut g);
    let saved: osfr_tools::kart_gateway::KartGatewayState =
        serde_json::from_slice(&serde_json::to_vec(g.state()).unwrap()).unwrap();
    let mut restored = KartGateway::load(&f.0, Some(saved)).unwrap().unwrap();
    assert_eq!(restored.previous_job(), Some(PRIOR_JOB));
    assert_eq!(restored.exit(), Some(character()));
}

// ---- gameplay tick: a clock-only racing tick must not request a save -------------------------

/// The recovered original-journal record the opening loader parses: one quest with no objectives.
/// All flags, rewards and counters are zero. The quest engine only hashes this file.
fn synthetic_journal() -> Vec<u8> {
    let mut journal = vec![0u8; 134];
    journal[..4].copy_from_slice(&1i32.to_le_bytes());
    journal
}

/// A self-contained gameplay package: synthetic class/inventory content (the payout tests' shape),
/// a minimal category catalogue, encounter template, quest journal and opening/world content, plus
/// this file's synthetic kart package with weapon effects enabled. No private lab content is
/// required.
struct GameplayFixture(PathBuf);

impl GameplayFixture {
    fn new() -> Self {
        Self::with_weapon_effects(true)
    }

    /// `weapon_effects` stamps `WeaponState::last_tick_ms` on each racing tick; tests that probe
    /// non-weapon saved state turn the simulation off.
    fn with_weapon_effects(weapon_effects: bool) -> Self {
        let path = Self::scratch();
        let fixture = Self(path);
        fixture.write_content(weapon_effects);
        fixture
    }

    fn scratch() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "kart-gameplay-tick-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        path
    }

    fn write_content(&self, weapon_effects: bool) {
        let dir = &self.0;
        // A fresh save starts in the prior job with Kart Driver already unlocked.
        let economy = Economy::new(PRIOR_JOB);
        for file in ["profiles.json", "inventory.json"] {
            fs::copy(economy.0.join(file), dir.join(file)).unwrap();
        }
        // The loader re-derives this one-row 44/2 packet and checks the prepared proof.
        let mut packet = vec![44, 0, 2, 0];
        for value in [1i32, 10, 10, 962, 1007, 1200] {
            packet.extend(value.to_le_bytes());
        }
        packet.push(1);
        packet.extend(1i32.to_le_bytes());
        packet.extend(0i32.to_le_bytes());
        fs::write(
            dir.join("categories.json"),
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "categories": [{"id": 10, "name_id": 962, "icon_id": 1007, "unknown": 1200,
                                "unknown2": true, "display": 1}],
                "groups": [],
                "packet_sha256": format!("{:x}", Sha256::digest(&packet)),
            }))
            .unwrap(),
        )
        .unwrap();
        // The synthetic combat template and rules from `tests/combat.rs`.
        let mut template = vec![0; 334];
        template[..4].copy_from_slice(&[35, 0, 2, 0]);
        template[33..37].copy_from_slice(&1_f32.to_le_bytes());
        template[49..53].copy_from_slice(&1_f32.to_le_bytes());
        template[61..65].copy_from_slice(&1_f32.to_le_bytes());
        template[133..137].copy_from_slice(&100_i32.to_le_bytes());
        template[224] = 1;
        template[250] = 0x5a;
        fs::write(dir.join("npc.bin"), &template).unwrap();
        fs::write(
            dir.join("combat.json"),
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "template_file": "npc.bin",
                "template_sha256": format!("{:x}", Sha256::digest(&template)),
                "npc": {"definition_id": 0, "model_id": 3311, "name": "Synthetic guide",
                        "texture_alias": "", "position": [0.0, 0.0, 0.0]},
                "enemy": {"definition_id": 1294, "model_id": 4, "name": "Synthetic enemy",
                          "texture_alias": "", "position": [4.0, 0.0, 0.0]},
                "max_health": 100, "hit_damage": 25, "range": 8.0, "cooldown_ms": 600,
            }))
            .unwrap(),
        )
        .unwrap();
        // The gameplay loader requires one quest definition and its journal even though a kart
        // tick never touches them. The opening journal also parses the recovered quest-record
        // shape, so this fixture's journal is a valid one-quest record (see `synthetic_journal`).
        let journal = synthetic_journal();
        fs::write(dir.join("journal.bin"), &journal).unwrap();
        fs::write(
            dir.join("quests.json"),
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "definitions": [{
                    "id": 1617, "title": "Synthetic quest", "description": "Synthetic quest",
                    "original": true, "initially_accepted": false, "giver_guid": 0,
                    "goals": [{"id": 14096, "text": "matching enemies", "required": 20,
                               "enemy_definition": 777}],
                    "coins": 0, "items": [],
                }],
                "journal": {
                    "file": "journal.bin",
                    "sha256": format!("{:x}", Sha256::digest(&journal)),
                    "length": journal.len(), "offset": 29, "bindings": [],
                },
                "trial": {
                    "start": [0.0, 0.0, 0.0],
                    "checkpoints": [[24.0, 0.0, 0.0], [24.0, 0.0, 24.0], [0.0, 0.0, 0.0]],
                    "radius": 3.0, "time_limit_ms": 90_000, "cooldown_ms": 30_000,
                    "reward_coins": 10,
                },
            }))
            .unwrap(),
        )
        .unwrap();
        // The timeout exit needs a world to return to: the opening content supplies the encounter
        // engine, and the encounter content supplies the world the kart return frame names.
        let zone = json!({
            "zone_id": 0, "name": "SyntheticWorld", "sky": "synthetic.xml",
            "geometry_id": 101, "position": [10.0, 20.0, 30.0],
            "rotation": [0.0, 0.0, 1.0], "tutorial": false,
        });
        fs::write(
            dir.join("opening.json"),
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "scope": "synthetic kart timeout",
                "reconstruction_notice": "synthetic fixture",
                "initial_profiles": [PRIOR_JOB],
                "actors": [{
                    "guid": 2, "definition_id": 20, "name_id": 0,
                    "name": "Synthetic guide", "model_id": 1,
                    "model_file_name": "synthetic.adr", "zone_id": 0,
                    "position": [0.0, 0.0, 0.0], "heading": 0.0,
                    "interaction_radius": 12.0, "kind": "guide",
                    "action": null, "required_profile": null, "provenance": "synthetic",
                }],
                "discoveries": [],
                "stages": [{
                    "id": 1617, "original_quest_id": null, "title_id": 0, "description_id": 0,
                    "title": "Synthetic stage", "description": "Synthetic stage",
                    "giver_guid": 2, "turnin_guid": 2, "ordered": true,
                    "objectives": [{
                        "id": 14096, "name_id": 0, "description_id": 0,
                        "description": "Synthetic objective", "required": 1,
                        "trigger": {"kind": "talk", "actor_guid": 2},
                        "recovery": "synthetic",
                    }],
                    "reward": {"stage_id": 1617, "coins": 0, "items": []},
                    "unlock_profiles": [], "provenance": "synthetic",
                }],
                "provenance": {},
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            dir.join("opening-abilities.json"),
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "basics": [{
                    "profile_id": PRIOR_JOB, "definition_id": 4895, "name_id": 420172,
                    "description_id": 0, "icon_id": 14290, "weapon_name_id": 131914,
                }],
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            dir.join("encounters.json"),
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "template_file": "npc.bin",
                "template_sha256": format!("{:x}", Sha256::digest(&template)),
                "world": zone.clone(),
                "world_enemies": [],
                "dungeons": [{
                    "id": 411, "title_id": 12, "description_id": 13, "objective_id": 14,
                    "zone": zone,
                    "guide": null,
                    "enemies": [{
                        "guid": 201, "definition_id": 500, "model_id": 3, "name_id": 100,
                        "name": "Synthetic enemy", "texture_alias": "",
                        "position": [2.0, 0.0, 0.0], "max_health": 100, "damage": 1,
                        "boss": false,
                    }],
                    "reward_coins": 0, "reward_items": [],
                }],
            }))
            .unwrap(),
        )
        .unwrap();
        self.write_kart(weapon_effects);
    }

    fn write_kart(&self, weapon_effects: bool) {
        // Only a live weapon lifecycle stamps `WeaponState::last_tick_ms` on each racing tick.
        let mut kart = content();
        let mut race = race_content();
        race.weapon_effects = weapon_effects;
        kart.race = Some(race);
        fs::write(
            self.0.join("kart_entry.json"),
            serde_json::to_vec(&kart).unwrap(),
        )
        .unwrap();
    }

    fn load(&self, saved: Option<GameplayState>) -> Gameplay {
        Gameplay::load(&self.0, saved).unwrap()
    }
}

impl Drop for GameplayFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The chat layout `tests/gameplay.rs` uses. The only text here is a developer command.
fn chat_request(text: &str) -> Vec<u8> {
    let mut packet = vec![15, 0, 1, 0, 0, 0];
    packet.extend(PLAYER.to_le_bytes());
    packet.extend(0u64.to_le_bytes());
    for _ in 0..2 {
        packet.extend([0; 12]);
        packet.extend(0u32.to_le_bytes());
        packet.extend(0u32.to_le_bytes());
    }
    packet.extend((text.len() as u32).to_le_bytes());
    packet.extend(text.as_bytes());
    packet.extend([0; 28]);
    packet
}

fn race_clock(game: &Gameplay) -> u64 {
    game.state()
        .kart_entry
        .unwrap()
        .race
        .unwrap()
        .weapons
        .last_tick_ms
}

fn race_phase(game: &Gameplay) -> RacePhase {
    game.state().kart_entry.unwrap().race.unwrap().phase
}

/// Enters through the shipped `realm race` command and drives the real handler to green.
fn enter_to_green(game: &mut Gameplay, character: &CharacterState) -> u64 {
    let entered = game
        .handle_in_world(&chat_request("realm race"), character, T0)
        .unwrap();
    assert!(entered.changed, "entering the race must be saved");
    let active = game.state().kart_entry.unwrap().active.unwrap();
    let (instance, encounter) = (active.context.instance_id, active.context.encounter_id);
    assert!(
        !game
            .handle_in_world(&register(PLAYER), character, T0 + 100)
            .unwrap()
            .replies
            .is_empty(),
        "the grid register must be answered"
    );
    let go_at = T0 + 200;
    assert!(
        !game
            .handle_in_world(&go(encounter, instance, PLAYER), character, go_at)
            .unwrap()
            .replies
            .is_empty(),
        "Go must start the countdown"
    );
    let green = go_at + 6_000;
    let started = game.tick(character, green).unwrap();
    assert!(
        started.changed,
        "the countdown-to-racing phase change must be saved"
    );
    green
}

/// F207: an accepted observation commits its spent flood-budget slot but requests no character
/// save, and a `Gameplay::load` round trip resumes it. A production login does not resume a race:
/// it abandons the saved one (see the reconnect assertions below), so the persisted count bounds
/// only the race it belongs to.
#[test]
fn an_observation_commits_the_persisted_budget_without_a_save() {
    let fixture = GameplayFixture::with_weapon_effects(false);
    let character = character();
    let mut game = fixture.load(None);
    game.set_dev_tools(true);
    game.set_role(osfr_tools::account_store::AccountRole::Admin);
    let green = enter_to_green(&mut game, &character);
    let before = game.state();

    let mut bulk = vec![33, 0, 6, 0];
    bulk.extend([0u8; 85]);
    let out = game.handle_in_world(&bulk, &character, green + 1).unwrap();
    assert!(out.replies.is_empty(), "33/6 is send-only");
    assert!(
        !out.changed,
        "spending the observation budget must not request a character save"
    );
    let saved = game.state();
    let before_kart = before.kart_entry.unwrap();
    let saved_kart = saved.kart_entry.clone().unwrap();
    assert!(
        !before_kart.clone().changed_for_save(&saved_kart),
        "a frame that only spends the budget and moves no other saved field must not save"
    );
    let mut real = saved_kart;
    real.next_instance += 1;
    assert!(
        before_kart.changed_for_save(&real),
        "a real saved change must still request a save"
    );
    assert_eq!(
        saved
            .kart_entry
            .as_ref()
            .unwrap()
            .race
            .as_ref()
            .unwrap()
            .observations,
        1,
        "the committed frame carries the spent slot"
    );
    // The same state is what a disconnect writes; a save/load round trip resumes the spent slot.
    // This is the save contract, not the production login: `Gateway::load_internal` calls
    // `Gameplay::abandon_kart_for_reconnect`, so a real reconnect aborts the saved race.
    let mut restored = fixture.load(Some(saved));
    assert_eq!(
        restored
            .state()
            .kart_entry
            .as_ref()
            .unwrap()
            .race
            .as_ref()
            .unwrap()
            .observations,
        1,
        "a save/load round trip resumes the spent slot"
    );
    assert_eq!(
        restored.abandon_kart_for_reconnect(),
        Some(character.clone()),
        "a reconnect returns the player to the pre-race location"
    );
    let abandoned = restored.state();
    assert_eq!(
        abandoned.profiles.as_ref().map(|profiles| profiles.active),
        Some(PRIOR_JOB),
        "a reconnect restores the prior job"
    );
    assert!(
        abandoned
            .kart_entry
            .as_ref()
            .is_none_or(|kart| kart.active.is_none()),
        "a reconnect abandons the saved race, so its observation budget bounds nothing"
    );
    // With the race gone the next observation is inert: no slot, no reply, no save.
    let mut bulk = vec![33, 0, 6, 0];
    bulk.extend([0u8; 85]);
    let out = restored
        .handle_in_world(&bulk, &character, green + 2)
        .unwrap();
    assert!(out.replies.is_empty(), "an abandoned race answers nothing");
    assert!(
        !out.changed,
        "an observation after a reconnect must not request a save"
    );
    assert_eq!(
        restored.state(),
        abandoned,
        "an observation after a reconnect moves nothing"
    );
}

#[test]
fn clock_only_racing_ticks_do_not_request_a_character_save() {
    let fixture = GameplayFixture::new();
    let character = character();
    let mut game = fixture.load(None);
    game.set_dev_tools(true);
    game.set_role(osfr_tools::account_store::AccountRole::Admin);
    let prior_job = game.state().profiles.unwrap().active;
    let green = enter_to_green(&mut game, &character);
    assert_eq!(game.state().profiles.unwrap().active, KART_JOB);
    // The green tick requested a save, so this is the last persisted snapshot. The next two
    // ticks are clock-only and would never be written; a server save/restore checkpoint (not the
    // login path, which abandons the race) starts from this older state, not `game.state()`.
    let saved = game.state();
    assert_eq!(race_clock(&game), green);

    // The green tick already settled the one-off node visibility; drain one more tick so the
    // assertion below isolates the weapon lifecycle clock.
    game.tick(&character, green + 50).unwrap();
    let idle = game.tick(&character, green + 100).unwrap();
    assert!(
        !idle.changed,
        "a clock-only racing tick must not request a character save"
    );
    assert!(idle.replies.is_empty(), "a clock-only tick sends nothing");
    // The clock still advanced: the race was really ticking, not stalled.
    assert_eq!(race_clock(&game), green + 100);
    assert_ne!(
        saved
            .kart_entry
            .as_ref()
            .unwrap()
            .race
            .as_ref()
            .unwrap()
            .weapons
            .last_tick_ms,
        race_clock(&game),
        "the saved snapshot must be older than the live clock"
    );

    // A server save/restore checkpoint between two clock-only ticks restores the last persisted
    // snapshot: the older clock and the prior job survive, and the next racing ticks still request
    // no save. A login is different: it abandons the race (F231).
    let mut restored = fixture.load(Some(saved));
    assert_eq!(race_phase(&restored), RacePhase::Racing);
    assert_eq!(
        race_clock(&restored),
        green,
        "restore must start from the saved clock"
    );
    assert_eq!(
        restored
            .state()
            .kart_entry
            .unwrap()
            .active
            .unwrap()
            .previous_job,
        Some(prior_job)
    );
    restored.tick(&character, green + 120).unwrap();
    let resumed = restored.tick(&character, green + 150).unwrap();
    assert!(
        !resumed.changed,
        "the restored racing tick is still clock-only"
    );
    assert_eq!(race_clock(&restored), green + 150);

    // A finish still persists: the last gate pays the registered reward and reports a change.
    let coins = game.state().inventory.coins;
    let mut now = green + 1_000;
    let mut last = None;
    for _ in 0..2 {
        for gate in 1..=4 {
            last = Some(
                game.handle_in_world(&gate_hit(PLAYER, gate as f32), &character, now)
                    .unwrap(),
            );
            now += 1_000;
        }
    }
    let finish = last.unwrap();
    assert!(
        finish.changed,
        "a finish and its reward claim must request a save"
    );
    assert_eq!(game.state().inventory.coins, coins + 40);
    assert_eq!(race_phase(&game), RacePhase::Finished);
    let paid = game.tick(&character, now).unwrap();
    assert!(!paid.changed, "a finished race pays and saves once");
}

/// Both kart save decisions use the type's save-decision equality: a re-stamped session weapon
/// clock alone is not a persisted change, and every other field still requests a character save.
#[test]
fn a_re_stamped_session_weapon_clock_is_not_a_saved_change() {
    let fixture = GameplayFixture::new();
    let character = character();
    let mut game = fixture.load(None);
    game.set_dev_tools(true);
    game.set_role(osfr_tools::account_store::AccountRole::Admin);
    enter_to_green(&mut game, &character);
    let before = game.state().kart_entry.unwrap();

    let mut clock_only = before.clone();
    clock_only.race.as_mut().unwrap().weapons.last_tick_ms += 50;
    let clock = KartGateway::load(&fixture.0, Some(clock_only))
        .unwrap()
        .unwrap();
    assert!(
        !before.clone().changed_for_save(clock.state()),
        "a re-stamped session clock alone is not a persisted change"
    );

    let mut race_progress = before.clone();
    race_progress.race.as_mut().unwrap().attempts += 1;
    let progressed = KartGateway::load(&fixture.0, Some(race_progress))
        .unwrap()
        .unwrap();
    assert!(
        before.clone().changed_for_save(progressed.state()),
        "a race field must still request a save"
    );

    let mut entry = before.clone();
    entry.active.as_mut().unwrap().previous_job = Some(PRIOR_JOB + 1);
    let moved = KartGateway::load(&fixture.0, Some(entry)).unwrap().unwrap();
    assert!(
        before.changed_for_save(moved.state()),
        "an entry field must still request a save"
    );
}

#[test]
fn a_timed_out_race_still_reports_the_abort_for_saving() {
    let fixture = GameplayFixture::new();
    let character = character();
    let mut game = fixture.load(None);
    game.set_dev_tools(true);
    game.set_role(osfr_tools::account_store::AccountRole::Admin);
    let prior_job = game.state().profiles.unwrap().active;
    enter_to_green(&mut game, &character);
    let snapshot = game.state().kart_entry.unwrap().race.unwrap();
    let timeout = snapshot.race_started_ms + snapshot.bounds.max_race_ms + 1;
    let out = game.tick(&character, timeout).unwrap();
    assert!(out.changed, "an aborted race must request a save");
    assert!(
        out.scene_zoning.is_some(),
        "the aborted racer returns to the world"
    );
    let entry = game.state().kart_entry.unwrap();
    assert!(entry.active.is_none());
    assert_eq!(entry.race.unwrap().phase, RacePhase::Aborted);
    assert_eq!(game.state().profiles.unwrap().active, prior_job);
}

// ---- T392: the client's mid-race quit ----------------------------------------------------------

/// `Ui.QuitMiniGame` (`00939100`) mid-race: 41/109 with a zero word, then the instance the
/// client stored from 41/107. Retail capture frame 42898: `29006d00 00000000 f0030000`.
fn quit(instance: u32) -> Vec<u8> {
    let mut b = vec![41, 0, 109, 0, 0, 0, 0, 0];
    b.extend(instance.to_le_bytes());
    b
}

/// The race start sends what the client's Exit needs (41/107 names the instance the quit
/// carries; GameState 8 runs `Countdown:Show`, which shows `MinigameTitleBar` and its Exit), and
/// the quit it sends ends the race unpaid, answered with 41/121 first, in the prior job, back at
/// the world pose. Foreign, malformed and replayed quits change nothing.
#[test]
fn the_client_quit_mid_race_ends_it_unpaid_in_the_prior_job_and_a_replay_changes_nothing() {
    let fixture = GameplayFixture::new();
    let character = character();
    let mut game = fixture.load(None);
    game.set_dev_tools(true);
    game.set_role(osfr_tools::account_store::AccountRole::Admin);
    let before_race = game.state();
    let entered = game
        .handle_in_world(&chat_request("realm race"), &character, T0)
        .unwrap();
    assert!(entered.changed);
    let active = game.state().kart_entry.unwrap().active.unwrap();
    let (instance, encounter) = (active.context.instance_id, active.context.encounter_id);
    game.handle_in_world(&register(PLAYER), &character, T0 + 100)
        .unwrap();
    let mut zone_ready = vec![41, 0, 107, 0];
    zone_ready.extend(encounter.to_le_bytes());
    zone_ready.extend(instance.to_le_bytes());
    let loaded = game.initialize_world(&character, T0 + 150).unwrap();
    assert!(loaded.replies.contains(&zone_ready));
    game.handle_in_world(&go(encounter, instance, PLAYER), &character, T0 + 200)
        .unwrap();
    let green = T0 + 6_200;
    let started = game.tick(&character, green).unwrap();
    assert!(started.replies.contains(&vec![33, 0, 1, 0, 8, 0, 0, 0]));
    assert_eq!(race_phase(&game), RacePhase::Racing);
    // One lap of two, like the capture's racer, then quit.
    for gate in 1..=4u64 {
        game.handle_in_world(
            &gate_hit(PLAYER, gate as f32),
            &character,
            green + gate * 1_000,
        )
        .unwrap();
    }
    let now = green + 5_000;

    let racing = game.state();
    let mut not_zero = quit(instance);
    not_zero[4] = 1;
    for foreign in [quit(instance + 1), not_zero] {
        let out = game.handle_in_world(&foreign, &character, now).unwrap();
        assert!(out.replies.is_empty() && !out.changed && out.scene_zoning.is_none());
        assert_eq!(game.state(), racing);
    }
    assert!(
        game.handle_in_world(&quit(instance)[..11], &character, now)
            .is_err()
    );
    assert_eq!(game.state(), racing);

    let out = game
        .handle_in_world(&quit(instance), &character, now)
        .unwrap();
    // Capture frame 42925: `29007900 3e000e00 f0030000 00` (encounter, instance, flag 0).
    let mut terminate = vec![41, 0, 121, 0];
    terminate.extend(encounter.to_le_bytes());
    terminate.extend(instance.to_le_bytes());
    terminate.push(0);
    assert_eq!(out.replies.first(), Some(&terminate));
    assert!(out.changed, "the quit must be saved");
    let scene = out.scene_zoning.expect("the racer returns to the world");
    assert_eq!(
        (scene.scene.as_str(), scene.id, &scene.position[..3]),
        ("SyntheticWorld", 0, &character.position[..])
    );
    let after = game.state();
    let entry = after.kart_entry.clone().unwrap();
    assert!(entry.active.is_none());
    assert_eq!(entry.race.unwrap().phase, RacePhase::Aborted);
    assert_eq!(
        after.inventory, before_race.inventory,
        "a quit pays nothing"
    );
    assert_eq!(
        after.profiles, before_race.profiles,
        "the prior job is back"
    );

    // The capture's client follows 41/121 with 39/7, a second 41/109 and 26/42: all inert now.
    for replay in [quit(instance), close(encounter), vec![26, 0, 42, 0]] {
        if let Ok(out) = game.handle_in_world(&replay, &character, now + 100) {
            assert!(!out.changed && out.scene_zoning.is_none());
        }
        assert_eq!(game.state(), after);
    }
    // A reload keeps the finished quit: no race, the prior job.
    let reloaded = fixture.load(Some(after.clone()));
    assert!(reloaded.state().kart_entry.unwrap().active.is_none());
    assert_eq!(reloaded.state().profiles, after.profiles);
}

// ---- private gateway proof (needs a playtest bundle; fails until the shared patch lands) ----

mod gateway_route {
    use super::*;
    use osfr_tools::{account_store::AccountRole, gateway::Gateway};

    /// Cale Yardbird: Kart Driver trainer 13 m from the Tangletrack speedway POI (content only).
    const GATE: u64 = 9_000_000_039;
    const GATE_POSITION: [f32; 3] = [-276.7101, -59.53247, 1964.762];
    /// Clack: a different NPC that is not a gate.
    const NOT_A_GATE: u64 = 9_000_000_048;
    const NOW: u64 = 10_000;

    fn wrap(inner: &[u8]) -> Vec<u8> {
        let mut b = vec![5, 0, 1];
        b.extend((inner.len() as u32).to_le_bytes());
        b.extend(inner);
        b
    }
    fn login() -> Vec<u8> {
        let mut b = 1u16.to_le_bytes().to_vec();
        for (i, s) in ["w149-private-test", "1.910.1.530630", ""]
            .iter()
            .enumerate()
        {
            b.extend((s.len() as u32).to_le_bytes());
            b.extend(s.bytes());
            if i == 0 {
                b.extend(1u64.to_le_bytes());
            }
        }
        b
    }
    fn npc(sub: u16, target: u64, choice: Option<i32>) -> Vec<u8> {
        let mut b = [26u16.to_le_bytes(), sub.to_le_bytes()].concat();
        b.extend(target.to_le_bytes());
        match choice {
            Some(c) => b.extend(c.to_le_bytes()),
            None => b.push(0),
        }
        b
    }
    fn send(g: &mut Gateway, packet: &[u8]) -> Vec<Vec<u8>> {
        let out = g.handle_at(&wrap(packet), NOW).unwrap();
        out.replies.iter().map(|p| p[7..].to_vec()).collect()
    }
    fn active(g: &Gateway) -> bool {
        g.snapshot()
            .gameplay
            .unwrap()
            .kart_entry
            .unwrap()
            .active
            .is_some()
    }
    fn job(g: &Gateway) -> i32 {
        g.snapshot().gameplay.unwrap().profiles.unwrap().active
    }

    #[test]
    #[ignore = "requires OSFR_KART_NPC_BUNDLE (a playtest bundle), OSFR_GATEWAY_STARTUP_SNAPSHOT=<bundle>/startup/startup.json and the gameplay/npc_interaction patch"]
    fn private_ordinary_player_starts_the_race_from_the_npc_menu_without_realm_race() {
        let bundle = PathBuf::from(std::env::var_os("OSFR_KART_NPC_BUNDLE").unwrap());
        let content = std::env::temp_dir().join(format!("kart-npc-bundle-{}", std::process::id()));
        let _ = fs::remove_dir_all(&content);
        copy_tree(&bundle.join("content"), &content);
        struct Cleanup(PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(content.clone());
        let reference = bundle.join("reference");
        let connect = |gates: Option<&[u64]>| {
            let path = content.join("kart_gates.json");
            let _ = fs::remove_file(&path);
            if let Some(npcs) = gates {
                let body = serde_json::to_vec(&json!({"schema_version": 1, "npcs": npcs}));
                fs::write(&path, body.unwrap()).unwrap();
            }
            let mut character = Gateway::reference_state(&reference).unwrap();
            character.position = GATE_POSITION;
            let mut g = Gateway::load_gameplay(&reference, character, &content, None).unwrap();
            g.set_dev_tools(false);
            g.set_role(AccountRole::Player);
            for packet in [login(), wrap(&[13, 0]), wrap(&[10, 0])] {
                g.handle_at(&packet, NOW).unwrap();
            }
            g
        };
        let menu = |g: &mut Gateway, target: u64| -> Vec<Vec<u8>> {
            send(g, &npc(8, target, None))
                .into_iter()
                .filter(|p| p.starts_with(&[26, 0, 9, 0]))
                .collect()
        };
        let offers_race = |packets: &[Vec<u8>]| {
            let label = DEFAULT_MENU_STRING_ID.to_le_bytes();
            packets.iter().any(|p| p.windows(4).any(|w| w == label))
        };

        // 1. No gate file: the menu has no race option and a forged choice starts nothing.
        let mut g = connect(None);
        assert!(!offers_race(&menu(&mut g, GATE)));
        let _ = g.handle_at(&wrap(&npc(10, GATE, Some(2))), NOW);
        assert!(!active(&g));

        // 2. Gate file present: only the listed NPC offers the race.
        let mut g = connect(Some(&[GATE]));
        assert!(
            offers_race(&menu(&mut g, GATE)),
            "gate NPC must list the race"
        );
        assert!(
            !offers_race(&menu(&mut g, NOT_A_GATE)),
            "other NPCs must not"
        );

        // 3. Canaries: `realm race` stays developer-only; forged option ids and a choice that
        //    names an NPC other than the open menu start nothing.
        send(&mut g, &chat_request("realm race"));
        assert!(!active(&g), "an ordinary player must not reach realm race");
        menu(&mut g, GATE);
        for forged in [0, 9, 99, -1, i32::MAX] {
            let _ = g.handle_at(&wrap(&npc(10, GATE, Some(forged))), NOW);
            assert!(!active(&g), "forged choice {forged} must not start a race");
        }
        let _ = g.handle_at(&wrap(&npc(10, NOT_A_GATE, Some(2))), NOW);
        assert!(!active(&g));

        // 4. The real choice (Talk = 1, race = 2) enters the track in Kart Driver.
        let before_job = job(&g);
        assert_ne!(before_job, 48, "fixture must start in another job");
        menu(&mut g, GATE);
        let replies = send(&mut g, &npc(10, GATE, Some(2)));
        assert!(active(&g), "the menu choice must start the race");
        assert_eq!(job(&g), 48);
        assert!(
            replies.iter().any(|p| p.starts_with(&[41, 0, 114, 0])),
            "encounter details"
        );
        let entry = g.snapshot().gameplay.unwrap().kart_entry.unwrap();
        let racing = entry.active.clone().unwrap();
        assert_eq!(racing.previous_job, Some(before_job));
        let next = entry.next_instance;

        // 5. A replayed choice while racing changes nothing (no second instance).
        let _ = g.handle_at(&wrap(&npc(10, GATE, Some(2))), NOW);
        let again = g.snapshot().gameplay.unwrap().kart_entry.unwrap();
        assert_eq!(again.next_instance, next);

        // 6. Finish loading the track, then cancel with the native close: back in the world,
        //    prior job restored, nothing paid.
        for loading in [wrap(&[13, 0]), wrap(&[10, 0])] {
            g.handle_at(&loading, NOW).unwrap();
        }
        assert!(active(&g));
        let coins = g.snapshot().gameplay.unwrap().inventory.coins;
        let mut close = vec![39, 0, 7];
        close.extend(racing.context.encounter_id.to_le_bytes());
        close.extend([255; 8]);
        send(&mut g, &close);
        assert!(!active(&g));
        assert_eq!(job(&g), before_job);
        assert_eq!(g.snapshot().gameplay.unwrap().inventory.coins, coins);
    }
}
