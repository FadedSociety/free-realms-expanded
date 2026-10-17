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
    inventory::Inventory,
    kart_entry::{ProxiedVehicle, RaceConfig, RaceEntryContent, RaceEntryContext},
    kart_gateway::{
        DEFAULT_MENU_STRING_ID, EntryDenied, KartGateway, KartGatewayContent, MAX_GATES,
    },
    kart_progress_native::ProgressPayload,
    kart_racing::RewardSchedule,
    kart_service::{GateWaypoint, KartServiceContent, RaceBounds, RaceStep, ScheduledFrame},
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
        fs::write(
            self.0.join("kart_entry.json"),
            serde_json::to_vec(&content).unwrap(),
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
    assert_eq!(
        g.admit_at(GATE_NPC, false, paid + 10),
        Err(EntryDenied::Cooldown)
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
    fn chat(text: &str) -> Vec<u8> {
        let mut b = vec![15, 0, 1, 0, 0, 0];
        b.extend(1u64.to_le_bytes());
        b.extend(0u64.to_le_bytes());
        b.extend([0; 40]);
        b.extend((text.len() as u32).to_le_bytes());
        b.extend(text.bytes());
        b.extend([0; 28]);
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
    fn copy_tree(from: &std::path::Path, to: &std::path::Path) {
        fs::create_dir_all(to).unwrap();
        for entry in fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_tree(&entry.path(), &target);
            } else {
                fs::copy(entry.path(), target).unwrap();
            }
        }
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
        send(&mut g, &chat("realm race"));
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
