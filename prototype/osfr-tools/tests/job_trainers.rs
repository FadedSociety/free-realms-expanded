//! W176: learn a job from its trainer NPC. Service-level tests cover every row of
//! `data/content/job_trainers.json`; the gateway tests run the ordinary frame path (dev tools off)
//! for all listed trainers. They need the gameplay.rs apply script and fail to compile without it.

use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState},
    gateway::{Gateway, GatewayOutcome},
    inventory::{InventoryItem, REQUIRED_PROFILE_SLOTS},
    mining,
    npc_interaction::{NpcAction, NpcInteraction, NpcSelection},
    profiles::{LearnDenied, ProfileState, Profiles, Trainer},
    quest_engine::{CharacterEligibility, QuestEngine, QuestState},
};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const CRATE: &str = env!("CARGO_MANIFEST_DIR");
/// The four jobs today's features need: Miner, Chef, Fisherman, Kart Driver.
const PRIORITY: [i32; 4] = [14, 45, 137, 48];
/// Synthetic collect objective used to prove one starter grant raises one count.
const COLLECT_QUEST: u32 = 9_900_001;
static NEXT: AtomicU64 = AtomicU64::new(0);

fn data() -> PathBuf {
    Path::new(CRATE).join("data/content")
}

fn reference_content() -> PathBuf {
    std::env::var_os("OSFR_TYPE8_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(CRATE).join("../../../lab/references/opening-journal-text-v2/content")
        })
}

fn copy_dir(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
}

/// Reference content plus the repository's trainer and Mining content, like a bundle.
struct Content(PathBuf);

impl Content {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "osfr-job-trainers-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        copy_dir(&reference_content(), &dir);
        fs::copy(
            data().join("job_trainers.json"),
            dir.join("job_trainers.json"),
        )
        .unwrap();
        mining::write_content(&dir, mining::MINING_GAME_ID).unwrap();
        Self(dir)
    }

    /// Add one active collect goal. Its item is a trainer starter definition, so
    /// one grant must raise the count once.
    fn install_collect_quest(&self, npc: u64, item_id: i32, quantity: u32) -> QuestState {
        let source = serde_json::json!([{
            "kind": "reconstructed",
            "evidence": "F110 synthetic trainer collect objective"
        }]);
        fs::create_dir_all(self.0.join("quests")).unwrap();
        fs::write(
            self.0.join("quests/f110_collect.json"),
            serde_json::to_vec(&serde_json::json!({
                "schema_version": 1,
                "records": [{
                    "id": COLLECT_QUEST,
                    "title_string_id": 1,
                    "giver_npc_id": npc,
                    "prerequisite_quest_ids": [],
                    "objectives": [{
                        "kind": "collect_item",
                        "id": 1,
                        "item_id": item_id,
                        "quantity": quantity,
                        "source": source.clone()
                    }],
                    "rewards": [],
                    "source": source.clone()
                }]
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            self.0.join("jobs.json"),
            serde_json::to_vec(&serde_json::json!({
                "schema_version": 1,
                "records": [{
                    "id": "f110_synthetic",
                    "name": "Synthetic",
                    "profile_id": null,
                    "category": "freestyle",
                    "max_level": 20,
                    "members_only": false,
                    "trainer_npc_ids": [],
                    "quest_ids": [COLLECT_QUEST],
                    "abilities": [],
                    "source": source
                }]
            }))
            .unwrap(),
        )
        .unwrap();
        fs::create_dir_all(self.0.join("npcs")).unwrap();
        fs::write(
            self.0.join("npcs/f110_collect.json"),
            serde_json::to_vec(&serde_json::json!({
                "schema_version": 1,
                "records": [{
                    "npc_id": npc,
                    "quest_ids_given": [COLLECT_QUEST]
                }]
            }))
            .unwrap(),
        )
        .unwrap();
        let mut engine = QuestEngine::from_split_content(
            &self.0,
            1,
            CharacterEligibility {
                player_level: 1,
                member: true,
                nonmember_quest_limit: 10,
                member_quest_limit: 10,
                ..CharacterEligibility::default()
            },
            None,
        )
        .unwrap();
        assert!(engine.offer(COLLECT_QUEST, npc).changed);
        assert!(engine.accept(COLLECT_QUEST).changed);
        engine.state().clone()
    }

    /// Make the trainer's starter grant fail after the unlock commits in the
    /// staged clone: the definition exists and is owned, but this profile does
    /// not allow its class.
    fn install_unusable_starter(&self, profile: i32, definition: i32) {
        let path = self.0.join("profiles.json");
        let mut profiles: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        profiles["starter_items"][profile.to_string()] = serde_json::json!([definition]);
        fs::write(path, serde_json::to_vec(&profiles).unwrap()).unwrap();
    }
}

impl Drop for Content {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn trainers(content: &Content) -> Vec<Trainer> {
    Profiles::load(&content.0, None)
        .unwrap()
        .load_trainers(&content.0)
        .unwrap()
}

fn npc_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for entry in fs::read_dir(data().join("npcs")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }
        let json: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        rows.extend(json["records"].as_array().unwrap().iter().cloned());
    }
    rows
}

fn npc(rows: &[Value], id: u64) -> &Value {
    rows.iter()
        .find(|r| r["npc_id"].as_u64() == Some(id))
        .expect("trainer NPC in content")
}

fn extra_rows() -> Vec<Value> {
    let json: Value =
        serde_json::from_slice(&fs::read(data().join("npc_spawns_extra.json")).unwrap()).unwrap();
    json["records"].as_array().unwrap().clone()
}

/// The existing "must be drawn in game" rule: the regional row is data-backed, or an
/// `npc_spawns_extra` spawn with a real model binds the same content key (T201 Dewey).
fn is_drawn(extras: &[Value], npc: &Value, trainer: &Trainer) -> bool {
    npc["placement"] == "legacy_data"
        || extras.iter().any(|row| {
            row["content_key"].as_u64() == Some(trainer.npc)
                && row["model_id"].as_u64().is_some_and(|model| model > 0)
        })
}

/// Start state with the trainer's prerequisite met when requested.
fn profile_state(profiles: &Profiles, trainer: &Trainer, prerequisite: bool) -> ProfileState {
    let mut state = profiles.state().clone();
    if let (true, Some(need)) = (prerequisite, &trainer.requires) {
        state.unlocked.insert(need.profile);
        state.experience.insert(need.profile, 100 * (need.rank - 1));
    }
    state
}

// ---- content ------------------------------------------------------------------------------

#[test]
fn trainer_rows_match_the_job_and_spawned_npc_content() {
    let content = Content::new();
    let rows = trainers(&content);
    let jobs: Value = serde_json::from_slice(&fs::read(data().join("jobs.json")).unwrap()).unwrap();
    let jobs = jobs["records"].as_array().unwrap();
    let npcs = npc_rows();
    let extras = extra_rows();
    for trainer in &rows {
        let job = jobs
            .iter()
            .find(|j| j["profile_id"].as_i64() == Some(i64::from(trainer.profile)))
            .unwrap_or_else(|| panic!("job for profile {}", trainer.profile));
        assert_eq!(job["name"], trainer.job.as_str());
        let row = npc(&npcs, trainer.npc);
        assert!(
            row["roles"]
                .as_array()
                .unwrap()
                .iter()
                .any(|role| role == "trainer"),
            "{} is an NPC trainer",
            trainer.job
        );
        assert!(
            is_drawn(&extras, row, trainer),
            "{}: the trainer must be drawn in game",
            trainer.job
        );
        assert!(!trainer.source.trim().is_empty());
        if let Some(need) = &trainer.requires {
            let required = jobs
                .iter()
                .find(|j| j["profile_id"].as_i64() == Some(i64::from(need.profile)))
                .unwrap();
            assert_eq!(required["name"], need.job.as_str());
        }
    }
    for profile in PRIORITY {
        assert!(
            rows.iter().any(|t| t.profile == profile),
            "a trainer for profile {profile}"
        );
    }
}

#[test]
fn malformed_trainer_files_are_refused_at_load() {
    let content = Content::new();
    let profiles = Profiles::load(&content.0, None).unwrap();
    let path = content.0.join("job_trainers.json");
    let good: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let cases: Vec<(&str, Box<dyn Fn(&mut Value)>)> = vec![
        (
            "unknown job",
            Box::new(|v| v["trainers"][0]["profile"] = 9999.into()),
        ),
        ("zero npc", Box::new(|v| v["trainers"][0]["npc"] = 0.into())),
        (
            "zero label",
            Box::new(|v| v["trainers"][0]["label_string_id"] = 0.into()),
        ),
        ("schema", Box::new(|v| v["schema_version"] = 2.into())),
        (
            "duplicate",
            Box::new(|v| {
                let row = v["trainers"][0].clone();
                v["trainers"].as_array_mut().unwrap().push(row);
            }),
        ),
        (
            "extra field",
            Box::new(|v| v["trainers"][0]["bonus"] = 1.into()),
        ),
        (
            "empty job",
            Box::new(|v| v["trainers"][0]["job"] = "".into()),
        ),
        (
            "empty source",
            Box::new(|v| v["trainers"][0]["source"] = "".into()),
        ),
        (
            "self prerequisite",
            Box::new(|v| {
                let profile = v["trainers"][0]["profile"].clone();
                let job = v["trainers"][0]["job"].clone();
                v["trainers"][0]["requires"] =
                    serde_json::json!({"profile": profile, "job": job, "rank": 20});
            }),
        ),
        (
            "rank zero",
            Box::new(|v| {
                v["trainers"][0]["requires"] =
                    serde_json::json!({"profile": 43, "job": "Brawler", "rank": 0});
            }),
        ),
        (
            "unknown prerequisite",
            Box::new(|v| {
                v["trainers"][0]["requires"] =
                    serde_json::json!({"profile": 9999, "job": "Missing", "rank": 20});
            }),
        ),
    ];
    for (name, edit) in cases {
        let mut bad = good.clone();
        edit(&mut bad);
        fs::write(&path, serde_json::to_vec(&bad).unwrap()).unwrap();
        assert!(profiles.load_trainers(&content.0).is_err(), "{name}");
    }

    let mut too_many = good.clone();
    let rows = too_many["trainers"].as_array_mut().unwrap();
    let row = rows[0].clone();
    rows.clear();
    for index in 0..129u64 {
        let mut row = row.clone();
        row["npc"] = (9_010_000_000u64 + index).into();
        rows.push(row);
    }
    fs::write(&path, serde_json::to_vec(&too_many).unwrap()).unwrap();
    assert!(profiles.load_trainers(&content.0).is_err(), "too many rows");

    fs::write(&path, vec![b' '; 65_537]).unwrap();
    assert!(profiles.load_trainers(&content.0).is_err(), "oversize");

    fs::remove_file(&path).unwrap();
    fs::create_dir(&path).unwrap();
    assert!(profiles.load_trainers(&content.0).is_err(), "directory");
    fs::remove_dir(&path).unwrap();
    assert!(
        profiles.load_trainers(&content.0).unwrap().is_empty(),
        "absent file means no trainers"
    );
}

// ---- service level: every listed trainer ----------------------------------------------------

fn menu_choice(service: &mut NpcInteraction, trainer: &Trainer, position: [f32; 3]) -> Option<i32> {
    let menu = service.interact(trainer.npc, 0, position, None).ok()?;
    menu.options
        .iter()
        .find(|option| {
            option.action
                == NpcAction::Learn {
                    profile: trainer.profile,
                }
        })
        .map(|option| {
            assert_eq!(option.string_id, trainer.label_string_id);
            option.id
        })
}

fn select(
    service: &mut NpcInteraction,
    npc: u64,
    choice: i32,
    position: [f32; 3],
) -> Result<NpcSelection, ()> {
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(npc.to_le_bytes());
    packet.extend(choice.to_le_bytes());
    service.select(&packet, 0, position, None).map_err(|_| ())
}

#[test]
fn each_listed_trainer_offers_the_job_and_unlocks_it_once() {
    let content = Content::new();
    let npcs = npc_rows();
    let mut service =
        NpcInteraction::load(&data().join("npcs"), &data().join("stores.json")).unwrap();
    let rows = trainers(&content);
    assert!(service.set_job_trainers(rows.clone()).is_empty());
    for trainer in &rows {
        let position: [f32; 3] =
            serde_json::from_value(npc(&npcs, trainer.npc)["position"].clone()).unwrap();
        let mut profiles = Profiles::load(&content.0, None).unwrap();
        profiles = {
            let state = profile_state(&profiles, trainer, true);
            Profiles::load(&content.0, Some(state)).unwrap()
        };
        assert!(!profiles.state().unlocked.contains(&trainer.profile));
        let choice = menu_choice(&mut service, trainer, position).expect("Learn option offered");
        let NpcSelection::Learn(picked) =
            select(&mut service, trainer.npc, choice, position).unwrap()
        else {
            panic!("Learn selection");
        };
        assert_eq!(&picked, trainer);
        assert_eq!(profiles.learn(&picked), Ok(()));
        assert!(profiles.state().unlocked.contains(&trainer.profile));
        let after = profiles.state().clone();
        assert_eq!(profiles.learn(&picked), Err(LearnDenied::AlreadyKnown));
        assert_eq!(
            profiles.state(),
            &after,
            "{}: second learn changes nothing",
            trainer.job
        );
    }
}

#[test]
fn service_refusals_change_nothing() {
    let content = Content::new();
    let npcs = npc_rows();
    let mut service =
        NpcInteraction::load(&data().join("npcs"), &data().join("stores.json")).unwrap();
    let rows = trainers(&content);
    service.set_job_trainers(rows.clone());
    let wizard = rows.iter().find(|t| t.profile == 12).unwrap();
    let ninja = rows.iter().find(|t| t.profile == 2).unwrap();
    let miner = rows.iter().find(|t| t.profile == 14).unwrap();

    // A prerequisite that is absent or below rank 20 grants nothing.
    let mut profiles = Profiles::load(&content.0, None).unwrap();
    let before = profiles.state().clone();
    for (trainer, job) in [(wizard, "Medic"), (ninja, "Archer")] {
        let Err(LearnDenied::NeedsRank {
            job: named,
            rank: 20,
        }) = profiles.learn(trainer)
        else {
            panic!("{} needs rank", trainer.job);
        };
        assert_eq!(named, job);
    }
    assert_eq!(profiles.state(), &before);
    let mut state = before.clone();
    state.unlocked.insert(35);
    state.experience.insert(35, 1800);
    let mut profiles = Profiles::load(&content.0, Some(state.clone())).unwrap();
    assert!(profiles.learn(ninja).is_err());
    assert_eq!(profiles.state(), &state);

    // A row that lies about its job is refused without a panic or a change.
    let mut forged = miner.clone();
    forged.profile = 9999;
    let mut profiles = Profiles::load(&content.0, None).unwrap();
    assert_eq!(profiles.learn(&forged), Err(LearnDenied::Unavailable));

    // Out of range: neither the menu nor a formerly offered choice is accepted.
    let position: [f32; 3] =
        serde_json::from_value(npc(&npcs, miner.npc)["position"].clone()).unwrap();
    let far = [position[0] + 500.0, position[1], position[2]];
    assert!(service.interact(miner.npc, 0, far, None).is_err());
    let choice = menu_choice(&mut service, miner, position).unwrap();
    assert!(select(&mut service, miner.npc, choice, far).is_err());
    assert!(select(&mut service, miner.npc, choice + 50, position).is_err());

    // An NPC outside the file offers no Learn option.
    let nobody = npcs
        .iter()
        .find(|n| {
            rows.iter().all(|t| t.npc != n["npc_id"].as_u64().unwrap()) && n["position"].is_array()
        })
        .unwrap();
    let nobody_pos: [f32; 3] = serde_json::from_value(nobody["position"].clone()).unwrap();
    let menu = service
        .interact(nobody["npc_id"].as_u64().unwrap(), 0, nobody_pos, None)
        .unwrap();
    assert!(
        menu.options
            .iter()
            .all(|option| !matches!(option.action, NpcAction::Learn { .. }))
    );

    // Unknown NPC ids in the list are reported, not fatal.
    let mut ghost = miner.clone();
    ghost.npc = 1;
    assert_eq!(service.set_job_trainers(vec![ghost]), vec![1]);
}

// ---- gateway level: the ordinary frame path -----------------------------------------------

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut packet = vec![5, 0, 1];
    packet.extend((inner.len() as u32).to_le_bytes());
    packet.extend(inner);
    packet
}

fn send(gateway: &mut Gateway, packet: &[u8], now_ms: u64) -> io_result::R {
    gateway.handle_at(&wrap(packet), now_ms)
}

mod io_result {
    pub type R = std::io::Result<super::GatewayOutcome>;
}

fn login(gateway: &mut Gateway) {
    let mut packet = 1u16.to_le_bytes().to_vec();
    for (index, value) in ["local-private-test", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
        packet.extend((value.len() as u32).to_le_bytes());
        packet.extend(value.as_bytes());
        if index == 0 {
            packet.extend(1u64.to_le_bytes());
        }
    }
    gateway.handle_at(&packet, 0).unwrap();
    send(gateway, &13u16.to_le_bytes(), 1_000).unwrap();
    send(gateway, &10u16.to_le_bytes(), 1_000).unwrap();
}

fn chat_inner(text: &str) -> Vec<u8> {
    let mut packet = vec![15, 0, 1, 0, 0, 0];
    packet.extend(1u64.to_le_bytes());
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

fn click(gateway: &mut Gateway, guid: u64, now: u64) -> Vec<Vec<u8>> {
    bodies(&click_result(gateway, guid, now).unwrap())
}

fn click_result(gateway: &mut Gateway, guid: u64, now: u64) -> io_result::R {
    let mut packet = vec![26, 0, 8, 0];
    packet.extend(guid.to_le_bytes());
    packet.push(0);
    send(gateway, &packet, now)
}

fn choose(gateway: &mut Gateway, guid: u64, choice: i32, now: u64) -> io_result::R {
    send(gateway, &choose_inner(guid, choice), now)
}

fn choose_inner(guid: u64, choice: i32) -> Vec<u8> {
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(guid.to_le_bytes());
    packet.extend(choice.to_le_bytes());
    packet
}

/// A client message bundle: a child is one wrapped application message.
fn bundle(messages: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = vec![0, 25];
    for message in messages {
        bytes.push(message.len() as u8);
        bytes.extend(message);
    }
    bytes
}

fn bodies(outcome: &GatewayOutcome) -> Vec<Vec<u8>> {
    outcome
        .replies
        .iter()
        .map(|reply| reply[7..].to_vec())
        .collect()
}

/// The option id of the 26/9 menu entry whose label is `string_id`.
fn option_with_label(menu: &[Vec<u8>], string_id: u32) -> Option<i32> {
    let packet = menu.iter().find(|p| p.starts_with(&[26, 0, 9, 0]))?;
    let count = i32::from_le_bytes(packet[13..17].try_into().ok()?) as usize;
    (0..count).find_map(|index| {
        let at = 17 + index * 28;
        let id = i32::from_le_bytes(packet[at..at + 4].try_into().ok()?);
        let label = u32::from_le_bytes(packet[at + 8..at + 12].try_into().ok()?);
        (label == string_id).then_some(id)
    })
}

fn said(replies: &[Vec<u8>], needle: &str) -> bool {
    replies
        .iter()
        .any(|reply| String::from_utf8_lossy(reply).contains(needle))
}

fn gameplay_json(gateway: &Gateway) -> Value {
    serde_json::to_value(gateway.snapshot().gameplay.unwrap()).unwrap()
}

fn unlocked(gateway: &Gateway) -> Vec<i64> {
    gameplay_json(gateway)["profiles"]["unlocked"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_i64().unwrap())
        .collect()
}

fn equipped(gateway: &Gateway, job: i32) -> usize {
    gameplay_json(gateway)["inventory"]["equipment"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| {
            entry["profile_id"].as_i64() == Some(i64::from(job))
                && entry["item_id"].as_i64().is_some_and(|id| id != 0)
        })
        .count()
}

fn switch(gateway: &mut Gateway, job: i32, now: u64) -> io_result::R {
    let mut packet = vec![26, 0, 13, 0];
    packet.extend(job.to_le_bytes());
    send(gateway, &packet, now)
}

/// Attachment slot ids from a 38/21 body, read in the order the client does.
fn activation_attachment_slots(body: &[u8]) -> Vec<i32> {
    assert_eq!(&body[..4], &[38, 0, 21, 0]);
    let payload = u32::from_le_bytes(body[4..8].try_into().unwrap()) as usize;
    let mut at = 8 + payload;
    let count = i32::from_le_bytes(body[at..at + 4].try_into().unwrap());
    at += 4;
    let mut slots = Vec::new();
    for _ in 0..count {
        for _ in 0..3 {
            let length = u32::from_le_bytes(body[at..at + 4].try_into().unwrap()) as usize;
            at += 4 + length;
        }
        slots.push(i32::from_le_bytes(
            body[at + 8..at + 12].try_into().unwrap(),
        ));
        at += 12;
    }
    slots
}

fn collect_count(gateway: &Gateway) -> u64 {
    gameplay_json(gateway)["npc_quests"]["quests"][COLLECT_QUEST.to_string()]["counts"][0]
        .as_u64()
        .unwrap()
}

fn gateway_at(content: &Content, state: GameplayState, near: [f32; 3]) -> Gateway {
    let reference = std::env::var_os("OSFR_GATEWAY_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(CRATE).join("../../../lab/references/rust-gateway-reference"));
    let mut character: CharacterState = Gateway::reference_state(&reference).unwrap();
    character.zone_id = 0;
    character.position = near;
    let mut gateway =
        Gateway::load_gameplay(&reference, character, &content.0, Some(state)).unwrap();
    login(&mut gateway);
    gateway
}

fn state_for(content: &Content, trainer: &Trainer, prerequisite: bool) -> GameplayState {
    let profiles = Profiles::load(&content.0, None).unwrap();
    let mut state = Gameplay::load(&content.0, None).unwrap().state();
    state.profiles = Some(profile_state(&profiles, trainer, prerequisite));
    state
}

fn trainer_position(npcs: &[Value], trainer: &Trainer) -> [f32; 3] {
    serde_json::from_value(npc(npcs, trainer.npc)["position"].clone()).unwrap()
}

#[test]
fn an_ordinary_player_learns_each_trainers_job_once_and_only_switches_to_a_complete_outfit() {
    let content = Content::new();
    let npcs = npc_rows();
    let mut checked = Vec::new();
    let mut selectable = 0;
    for trainer in trainers(&content) {
        let position = trainer_position(&npcs, &trainer);
        let near = [position[0] + 2.0, position[1], position[2]];
        let mut gateway = gateway_at(&content, state_for(&content, &trainer, true), near);
        assert!(
            !unlocked(&gateway).contains(&i64::from(trainer.profile)),
            "{}",
            trainer.job
        );
        let active_before = gameplay_json(&gateway)["profiles"]["active"].clone();
        let outfit_before = equipped(&gateway, trainer.profile);

        let menu = click(&mut gateway, trainer.npc, 2_000);
        let choice = option_with_label(&menu, trainer.label_string_id)
            .unwrap_or_else(|| panic!("{}: Learn option in the menu", trainer.job));
        let learned = choose(&mut gateway, trainer.npc, choice, 2_100).unwrap();
        assert!(
            unlocked(&gateway).contains(&i64::from(trainer.profile)),
            "{}",
            trainer.job
        );
        assert!(
            said(&bodies(&learned), "You learned the"),
            "{}",
            trainer.job
        );
        assert_eq!(
            gameplay_json(&gateway)["profiles"]["active"],
            active_before,
            "{}: learning must not switch jobs",
            trainer.job
        );
        let learned_state = gameplay_json(&gateway);

        // The client cannot apply a job outfit without Chest and Legs. A job
        // whose content lacks them stays unlocked but is refused at selection.
        let selected = switch(&mut gateway, trainer.profile, 2_900).unwrap();
        let selected_bodies = bodies(&selected);
        if let Some(activation) = selected_bodies
            .iter()
            .find(|body| body.starts_with(&[38, 0, 21, 0]))
        {
            selectable += 1;
            let slots = activation_attachment_slots(activation);
            for required in REQUIRED_PROFILE_SLOTS {
                assert!(
                    slots.contains(&required),
                    "{}: activation is missing required slot {required}: {slots:?}",
                    trainer.job
                );
            }
            assert!(
                equipped(&gateway, trainer.profile) >= outfit_before,
                "{} lost starter outfit pieces",
                trainer.job
            );
            assert_eq!(
                gameplay_json(&gateway)["profiles"]["active"],
                Value::from(trainer.profile),
                "{} selected",
                trainer.job
            );
        } else {
            assert!(
                said(&selected_bodies, "outfit is missing"),
                "{}: selection refusal: {}",
                trainer.job,
                String::from_utf8_lossy(&selected_bodies.concat())
            );
            assert_eq!(
                gameplay_json(&gateway),
                learned_state,
                "{}: refused selection changed state",
                trainer.job
            );
            assert_eq!(
                gameplay_json(&gateway)["profiles"]["active"],
                active_before,
                "{}: refused selection switched jobs",
                trainer.job
            );
        }

        // The same choice again is refused with a line, changing nothing.
        let before_again = gameplay_json(&gateway);
        click(&mut gateway, trainer.npc, 3_000);
        let again = choose(&mut gateway, trainer.npc, choice, 3_100).unwrap();
        assert!(said(&bodies(&again), "already know"), "{}", trainer.job);
        assert_eq!(
            gameplay_json(&gateway),
            before_again,
            "{}: refusal changed state",
            trainer.job
        );
        checked.push(trainer.profile);
    }
    for profile in PRIORITY {
        assert!(
            checked.contains(&profile),
            "priority job {profile} has a spawned trainer"
        );
    }
    assert!(
        selectable >= 5,
        "the five trainer jobs with complete starter kits must remain selectable"
    );
}

#[test]
fn learning_a_starter_grant_counts_each_item_once() {
    let content = Content::new();
    let npcs = npc_rows();
    let miner = trainers(&content)
        .into_iter()
        .find(|trainer| trainer.profile == 14)
        .unwrap();
    // 21665 is the Miner starter hat in profiles.json.
    let quest_state = content.install_collect_quest(miner.npc, 21665, 2);
    let position = trainer_position(&npcs, &miner);
    let near = [position[0] + 2.0, position[1], position[2]];
    let mut state = state_for(&content, &miner, false);
    state.npc_quests = Some(quest_state);
    let mut gateway = gateway_at(&content, state, near);
    assert_eq!(collect_count(&gateway), 0);

    let menu = click(&mut gateway, miner.npc, 2_000);
    let choice = option_with_label(&menu, miner.label_string_id).unwrap();
    let learned = choose(&mut gateway, miner.npc, choice, 2_100).unwrap();
    assert!(said(&bodies(&learned), "You learned the"));
    assert_eq!(
        collect_count(&gateway),
        1,
        "one starter grant must raise a collect objective once"
    );
}

#[test]
fn an_outfit_grant_error_leaves_the_trainer_menu_usable() {
    let content = Content::new();
    // The Ninja weapon is owned, but its class is not allowed on the Miner
    // profile, so the starter grant errors after the staged unlock.
    content.install_unusable_starter(14, 75091);
    let npcs = npc_rows();
    let miner = trainers(&content)
        .into_iter()
        .find(|trainer| trainer.profile == 14)
        .unwrap();
    let position = trainer_position(&npcs, &miner);
    let near = [position[0] + 2.0, position[1], position[2]];
    let mut gateway = gateway_at(&content, state_for(&content, &miner, false), near);
    let menu = click(&mut gateway, miner.npc, 2_000);
    let choice = option_with_label(&menu, miner.label_string_id).unwrap();
    let before = gameplay_json(&gateway);

    // The gateway drops an application error without committing the staged
    // gameplay clone; the save must remain exactly as it was.
    let failed = choose(&mut gateway, miner.npc, choice, 2_100).unwrap();
    assert!(
        !failed.state_changed && !failed.gameplay_changed,
        "the bad starter must fail the transaction"
    );
    assert!(failed.replies.is_empty());
    assert_eq!(gameplay_json(&gateway), before);
    assert!(!unlocked(&gateway).contains(&14));

    // A different offered option still works: the failed grant must not have
    // consumed the shared menu before the transaction rolled back.
    let talked = choose(&mut gateway, miner.npc, 1, 2_200).unwrap();
    assert!(!talked.replies.is_empty(), "the dialog option still opens");
}

#[test]
fn selecting_an_unlocked_job_without_an_outfit_is_refused_atomically() {
    let content = Content::new();
    let npcs = npc_rows();
    let fisherman = trainers(&content)
        .into_iter()
        .find(|trainer| trainer.profile == 137)
        .unwrap();
    let position = trainer_position(&npcs, &fisherman);
    let near = [position[0] + 2.0, position[1], position[2]];
    let mut state = state_for(&content, &fisherman, false);
    state
        .profiles
        .as_mut()
        .unwrap()
        .unlocked
        .insert(fisherman.profile);
    let mut gateway = gateway_at(&content, state, near);
    let before = gameplay_json(&gateway);

    let refused = switch(&mut gateway, fisherman.profile, 2_000).unwrap();
    assert!(
        !refused.state_changed && !refused.gameplay_changed,
        "a refused switch must not mark the save changed"
    );
    let bodies = bodies(&refused);
    assert!(
        said(&bodies, "outfit is missing"),
        "{}",
        String::from_utf8_lossy(&bodies.concat())
    );
    assert!(
        bodies.iter().all(|body| !body.starts_with(&[38, 0, 21, 0])),
        "a refused switch must not send an activation"
    );
    assert_eq!(gameplay_json(&gateway), before);
}

#[test]
fn gateway_refusals_leave_the_save_untouched() {
    let content = Content::new();
    let npcs = npc_rows();
    let rows = trainers(&content);
    let smith = rows.iter().find(|t| t.profile == 16).unwrap();
    let ninja = rows.iter().find(|t| t.profile == 2).unwrap();
    let position = trainer_position(&npcs, smith);
    let near = [position[0] + 2.0, position[1], position[2]];

    // Out of range: the menu and the formerly offered choice both fail, with no change.
    let mut near_gateway = gateway_at(&content, state_for(&content, smith, false), near);
    let menu = click(&mut near_gateway, smith.npc, 2_000);
    let choice = option_with_label(&menu, smith.label_string_id).unwrap();
    let far = [near[0] + 400.0, near[1], near[2]];
    let mut far_gateway = gateway_at(&content, state_for(&content, smith, false), far);
    let before = gameplay_json(&far_gateway);
    let far_menu = click_result(&mut far_gateway, smith.npc, 2_000).unwrap();
    assert!(
        bodies(&far_menu)
            .iter()
            .all(|reply| !reply.starts_with(&[26, 0, 9, 0])),
        "a trainer outside visibility must not open a menu"
    );
    let _ = choose(&mut far_gateway, smith.npc, choice, 2_100);
    assert_eq!(gameplay_json(&far_gateway), before);
    assert!(!unlocked(&far_gateway).contains(&16));

    // A choice that was never offered changes nothing.
    let before = gameplay_json(&near_gateway);
    let _ = choose(&mut near_gateway, smith.npc, 9_999, 2_200);
    assert_eq!(gameplay_json(&near_gateway), before);
    assert!(!unlocked(&near_gateway).contains(&16));

    // An unmet recovered prerequisite is a chat refusal, not an unlock.
    let position = trainer_position(&npcs, ninja);
    let near = [position[0] + 2.0, position[1], position[2]];
    let mut gateway = gateway_at(&content, state_for(&content, ninja, false), near);
    let menu = click(&mut gateway, ninja.npc, 2_000);
    let choice = option_with_label(&menu, ninja.label_string_id).unwrap();
    let before = gameplay_json(&gateway);
    let refused = choose(&mut gateway, ninja.npc, choice, 2_100).unwrap();
    assert!(said(&bodies(&refused), "level 20 Archer"));
    assert!(
        !refused.state_changed && !refused.gameplay_changed,
        "a prerequisite refusal must not mark the save changed"
    );
    // The refused choice leaves the menu active, so the same option is still
    // understood instead of failing with NoInteraction.
    let again = choose(&mut gateway, ninja.npc, choice, 2_200).unwrap();
    assert!(said(&bodies(&again), "level 20 Archer"));
    assert!(!again.state_changed && !again.gameplay_changed);
    assert_eq!(gameplay_json(&gateway), before);
    assert!(!unlocked(&gateway).contains(&2));

    // A live Mining run refuses the interaction at the shared busy gate before the menu
    // opens, so neither the activity nor the NPC interaction state changes.
    let position = trainer_position(&npcs, smith);
    let near = [position[0] + 2.0, position[1], position[2]];
    let mut state = state_for(&content, smith, false);
    state.profiles.as_mut().unwrap().active = 14;
    state.profiles.as_mut().unwrap().unlocked.insert(14);
    let mut gateway = gateway_at(&content, state, near);
    gateway.set_dev_tools(true);
    gateway.set_role(AccountRole::Admin);
    let started = send(&mut gateway, &chat_inner("realm mine"), 2_500).unwrap();
    assert!(
        said(&bodies(&started), "Mining test started"),
        "{}",
        String::from_utf8_lossy(&started.replies.concat())
    );
    let before = gameplay_json(&gateway);
    let refused = click_result(&mut gateway, smith.npc, 2_600).unwrap();
    assert!(
        said(
            &bodies(&refused),
            "Finish the active activity before interacting."
        ),
        "{}",
        String::from_utf8_lossy(&refused.replies.concat())
    );
    assert!(
        bodies(&refused)
            .iter()
            .all(|reply| !reply.starts_with(&[26, 0, 9, 0])),
        "a busy trainer must not open a menu"
    );
    assert!(!refused.state_changed && !refused.gameplay_changed);
    assert_eq!(gameplay_json(&gateway), before);
    assert!(!unlocked(&gateway).contains(&16));

    // A choice sent without an open menu is refused by the same gate, still changing nothing.
    let refused = choose(&mut gateway, smith.npc, 2, 2_700).unwrap();
    assert!(said(
        &bodies(&refused),
        "Finish the active activity before interacting."
    ));
    assert!(!refused.state_changed && !refused.gameplay_changed);
    assert_eq!(gameplay_json(&gateway), before);
    assert!(!unlocked(&gateway).contains(&16));
}

#[test]
fn without_the_content_file_nobody_offers_a_job() {
    let content = Content::new();
    fs::remove_file(content.0.join("job_trainers.json")).unwrap();
    let npcs = npc_rows();
    let with = Content::new();
    let smith = trainers(&with)
        .into_iter()
        .find(|trainer| trainer.profile == 16)
        .unwrap();
    let position = trainer_position(&npcs, &smith);
    let near = [position[0] + 2.0, position[1], position[2]];
    let mut gateway = gateway_at(&with, state_for(&with, &smith, false), near);
    fs::remove_file(content.0.join("job_trainers.json")).ok();
    let menu = click(&mut gateway, smith.npc, 2_000);
    // This gateway was built with the file; prove the real absent-file path separately.
    assert!(option_with_label(&menu, smith.label_string_id).is_some());

    let state = state_for(&with, &smith, false);
    let mut absent = gateway_at(&content, state, near);
    let menu = click(&mut absent, smith.npc, 2_000);
    assert!(option_with_label(&menu, smith.label_string_id).is_none());
}

fn owned_count(gateway: &Gateway, definition: i32) -> u64 {
    gameplay_json(gateway)["inventory"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| item["definition_id"].as_i64() == Some(i64::from(definition)))
        .map(|item| item["count"].as_u64().unwrap())
        .sum()
}

/// F155: the 38/21 activation must be serialized from the outfit after the starter
/// repair. Selecting a bare but unlocked job used to emit the pre-repair list (zero
/// attachments), which the client's ProcessProfileSwap rejects.
#[test]
fn selecting_a_bare_unlocked_job_emits_an_activation_from_the_repaired_outfit() {
    let content = Content::new();
    let npcs = npc_rows();
    let miner = trainers(&content)
        .into_iter()
        .find(|trainer| trainer.profile == 14)
        .unwrap();
    let position = trainer_position(&npcs, &miner);
    let near = [position[0] + 2.0, position[1], position[2]];
    let mut state = state_for(&content, &miner, false);
    let profiles = state.profiles.as_mut().unwrap();
    assert_ne!(
        profiles.active, miner.profile,
        "the test starts on another job"
    );
    profiles.unlocked.insert(miner.profile);
    let mut gateway = gateway_at(&content, state, near);
    assert_eq!(
        equipped(&gateway, miner.profile),
        0,
        "the unlocked job starts bare"
    );

    let switched = switch(&mut gateway, miner.profile, 2_000).unwrap();
    let bodies = bodies(&switched);
    let activation = bodies
        .iter()
        .find(|body| body.starts_with(&[38, 0, 21, 0]))
        .unwrap_or_else(|| {
            panic!(
                "{}: a repairable job switch emits its activation: {}",
                miner.job,
                String::from_utf8_lossy(&bodies.concat())
            )
        });
    let slots = activation_attachment_slots(activation);
    for required in REQUIRED_PROFILE_SLOTS {
        assert!(
            slots.contains(&required),
            "{}: activation slot {required} missing after the starter repair: {slots:?}",
            miner.job
        );
    }
    assert_eq!(
        gameplay_json(&gateway)["profiles"]["active"],
        Value::from(miner.profile),
        "{} selected",
        miner.job
    );
}

/// F155: learning a job must equip starter pieces the player already owns rather
/// than granting another copy (no free re-grant).
#[test]
fn learning_does_not_increase_owned_starter_quantities() {
    let content = Content::new();
    let npcs = npc_rows();
    let miner = trainers(&content)
        .into_iter()
        .find(|trainer| trainer.profile == 14)
        .unwrap();
    // 21665 is the Miner starter hat; the player owns one already.
    let starter = 21665;
    let position = trainer_position(&npcs, &miner);
    let near = [position[0] + 2.0, position[1], position[2]];
    let mut state = state_for(&content, &miner, false);
    let id = state
        .inventory
        .items
        .iter()
        .map(|item| item.id)
        .max()
        .unwrap_or(0)
        + 1;
    state.inventory.items.push(InventoryItem {
        definition_id: starter,
        tint: 0,
        id,
        count: 1,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: 0,
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    });
    state.inventory.next_instance_id = Some(i64::from(id) + 1);
    let mut gateway = gateway_at(&content, state, near);
    assert_eq!(owned_count(&gateway, starter), 1);

    let menu = click(&mut gateway, miner.npc, 2_000);
    let choice = option_with_label(&menu, miner.label_string_id).unwrap();
    let learned = choose(&mut gateway, miner.npc, choice, 2_100).unwrap();
    assert!(said(&bodies(&learned), "You learned the"), "{}", miner.job);
    assert!(unlocked(&gateway).contains(&i64::from(miner.profile)));
    assert_eq!(
        owned_count(&gateway, starter),
        1,
        "{}: training granted a second copy of an owned starter piece",
        miner.job
    );
}

/// F155: the menu a bundle consumes must roll back with the rest of the staged
/// state when a later child fails.
#[test]
fn a_failed_bundle_keeps_the_trainer_menu_usable() {
    let content = Content::new();
    let npcs = npc_rows();
    let miner = trainers(&content)
        .into_iter()
        .find(|trainer| trainer.profile == 14)
        .unwrap();
    let position = trainer_position(&npcs, &miner);
    let near = [position[0] + 2.0, position[1], position[2]];
    let mut gateway = gateway_at(&content, state_for(&content, &miner, false), near);
    let menu = click(&mut gateway, miner.npc, 2_000);
    let choice = option_with_label(&menu, miner.label_string_id).unwrap();
    let before = gameplay_json(&gateway);

    // A successful Learn choice followed by a truncated 26/13 fails the whole
    // bundle: the unlock, the starter items and the menu consumption must all
    // roll back before the gateway commits anything.
    let failed = gateway.handle_at(
        &bundle(&[
            wrap(&choose_inner(miner.npc, choice)),
            wrap(&[26, 0, 13, 0]),
        ]),
        2_100,
    );
    assert!(failed.is_err(), "the truncated child rejects the bundle");
    assert_eq!(gameplay_json(&gateway), before);
    assert!(!unlocked(&gateway).contains(&i64::from(miner.profile)));

    // The original menu is still open: its Talk choice (1) opens the dialog.
    let talked = choose(&mut gateway, miner.npc, 1, 2_200).unwrap();
    assert!(
        !talked.replies.is_empty(),
        "the failed bundle consumed the menu"
    );
}

/// S15: the choice path stages one selection at a time, so two Talk choices in
/// one client bundle both read the same open menu and neither waits on the other.
#[test]
fn two_talk_choices_in_one_bundle_both_reach_the_open_menu() {
    let content = Content::new();
    let npcs = npc_rows();
    let smith = trainers(&content)
        .into_iter()
        .find(|trainer| trainer.profile == 16)
        .unwrap();
    let position = trainer_position(&npcs, &smith);
    let near = [position[0] + 2.0, position[1], position[2]];
    let mut gateway = gateway_at(&content, state_for(&content, &smith, false), near);
    click(&mut gateway, smith.npc, 2_000);
    let before = gameplay_json(&gateway);
    let both = gateway
        .handle_at(
            &bundle(&[
                wrap(&choose_inner(smith.npc, 1)),
                wrap(&choose_inner(smith.npc, 1)),
            ]),
            2_100,
        )
        .unwrap();
    let bodies = bodies(&both);
    assert_eq!(
        bodies
            .iter()
            .filter(|body| body.starts_with(&[26, 0, 3, 0]))
            .count(),
        2,
        "both Talk choices must open the dialogue: {:?}",
        bodies
    );
    assert_eq!(
        gameplay_json(&gateway),
        before,
        "Talk without a quest engine must not change the save"
    );
}
