//! T201: Drill Sergeant Dewey is drawable in Snowhill and is the Warrior trainer.
//! The service rows live in `data/content/npc_spawns_extra.json` and `job_trainers.json`;
//! these tests drive the production loader and the ordinary 26/8 -> 26/10 frame path.
//! The buildable player package may lack `starter_items["32"]`; the unlock path must still
//! work and must say the outfit is unavailable instead of failing.

use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    game_store,
    gameplay::{Gameplay, GameplayState},
    gateway::{Gateway, GatewayOutcome},
    mining,
    profiles::{LearnDenied, Profiles, Trainer},
    world,
};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const CRATE: &str = env!("CARGO_MANIFEST_DIR");
const DEWEY: u64 = 9_000_000_076;
/// Blacksmith trainer used as the "another NPC" refusal target.
const SMITH: u64 = 9_000_000_206;
const WARRIOR: i32 = 32;
const BRAWLER: i32 = 43;
/// Client locale id of `Become a Warrior`.
const BECOME_A_WARRIOR: u32 = 412_471;
/// Client locale id of `Drill Sergeant Dewey` (Global.Text).
const DEWEY_NAME: u64 = 22_435;
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

/// Reference content plus the repository's trainer content and a Mining run, like a bundle.
struct Content(PathBuf);

impl Content {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "osfr-warrior-trainer-{}-{}",
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

fn warrior_trainer(content: &Content) -> Trainer {
    trainers(content)
        .into_iter()
        .find(|trainer| trainer.npc == DEWEY)
        .expect("Warrior trainer row for Dewey")
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn dewey_spawn() -> Value {
    read_json(&data().join("npc_spawns_extra.json"))["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["spawn_guid"].as_u64() == Some(DEWEY))
        .cloned()
        .expect("Dewey spawn row in npc_spawns_extra.json")
}

/// The regional NPC content record (the source of the interaction position and role).
fn regional_npc(id: u64) -> Value {
    for entry in fs::read_dir(data().join("npcs")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let document: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        if let Some(row) = document["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["npc_id"].as_u64() == Some(id))
        {
            return row.clone();
        }
    }
    panic!("NPC {id} content row");
}

fn npc_position(id: u64) -> [f32; 3] {
    serde_json::from_value(regional_npc(id)["position"].clone()).unwrap()
}

fn near(id: u64) -> [f32; 3] {
    let position = npc_position(id);
    [position[0] + 2.0, position[1], position[2]]
}

/// Rank on the local 100-XP curve: rank N needs 100 * (N - 1) experience.
fn profiles_with_brawler(content: &Content, rank: u32) -> Profiles {
    let profiles = Profiles::load(&content.0, None).unwrap();
    let mut state = profiles.state().clone();
    state.unlocked.insert(BRAWLER);
    state.experience.insert(BRAWLER, 100 * (rank - 1));
    Profiles::load(&content.0, Some(state)).unwrap()
}

fn gameplay_state(content: &Content, brawler_rank: u32) -> GameplayState {
    let mut state = Gameplay::load(&content.0, None).unwrap().state();
    state.profiles = Some(profiles_with_brawler(content, brawler_rank).state().clone());
    state
}

fn starter_items(content: &Content, profile: i32) -> Vec<i64> {
    read_json(&content.0.join("profiles.json"))["starter_items"][profile.to_string()]
        .as_array()
        .map(|items| items.iter().filter_map(Value::as_i64).collect())
        .unwrap_or_default()
}

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
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(guid.to_le_bytes());
    packet.extend(choice.to_le_bytes());
    send(gateway, &packet, now)
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
    let packet = menu
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 9, 0]))?;
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

/// The save document the next login would load, byte for byte.
fn save_bytes(gateway: &Gateway) -> Vec<u8> {
    serde_json::to_vec(&gateway.snapshot()).unwrap()
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

fn reference() -> PathBuf {
    std::env::var_os("OSFR_GATEWAY_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(CRATE).join("../../../lab/references/rust-gateway-reference"))
}

fn gateway_at(content: &Content, state: GameplayState, near: [f32; 3]) -> Gateway {
    let mut character: CharacterState = Gateway::reference_state(&reference()).unwrap();
    character.zone_id = 0;
    character.position = near;
    let mut gateway =
        Gateway::load_gameplay(&reference(), character, &content.0, Some(state)).unwrap();
    login(&mut gateway);
    gateway
}

// ---- content ------------------------------------------------------------------------------

#[test]
fn dewey_is_drawable_and_has_a_warrior_trainer_row() {
    // The Snowhill record names and places Dewey (Sanctuary-main's spawn, T506) but has no model.
    let regional = regional_npc(DEWEY);
    assert_eq!(regional["name"], "Drill Sergeant Dewey");
    assert_eq!(regional["placement"], "reference_data");
    assert!(
        regional["roles"]
            .as_array()
            .unwrap()
            .iter()
            .any(|role| role == "trainer"),
        "the Snowhill record still carries the trainer role"
    );

    // The extra spawn is what makes him a drawn, clickable actor.
    let spawn = dewey_spawn();
    assert_eq!(spawn["content_key"].as_u64(), Some(DEWEY));
    assert!(spawn["legacy_npc_id"].is_null());
    assert!(
        spawn["capture_match"].is_null(),
        "capture-matched rows are filtered out of the spawn list"
    );
    assert_eq!(spawn["name"], "Drill Sergeant Dewey");
    let model_id = spawn["model_id"].as_u64().expect("positive model id");
    assert!(model_id > 0);
    let model_file = spawn["model_file_name"].as_str().unwrap();
    assert!(
        model_file.ends_with(".agr") || model_file.ends_with(".adr"),
        "a real client model file, not a placeholder: {model_file}"
    );
    let name_id = spawn["name_string_id"].as_u64().expect("name string id");
    assert_eq!(
        name_id, DEWEY_NAME,
        "the AddNpc name field resolves through Global.Text"
    );
    let position: [f32; 3] = serde_json::from_value(spawn["position"].clone()).unwrap();
    assert!(position.iter().all(|value| value.is_finite()));
    assert!(spawn["heading"].as_f64().is_some_and(f64::is_finite));
    let regional_position: [f32; 3] = serde_json::from_value(regional["position"].clone()).unwrap();
    for (axis, (spawned, shared)) in position.iter().zip(regional_position).enumerate() {
        assert!(
            (spawned - shared).abs() < 1e-3,
            "axis {axis}: the spawn must stand where the interaction row does"
        );
    }

    // The production loader draws him for a nearby player: one 35/2 AddNpc with his model.
    let mut visibility = world::NpcVisibility::load(&data()).unwrap();
    assert_eq!(visibility.content_key(DEWEY), Some(DEWEY));
    let packets = visibility.update(0, position).unwrap();
    let added = packets
        .iter()
        .filter_map(|packet| osfr_tools::startup_world::parse_add_npc(packet).ok())
        .find(|npc| npc.actor_id == DEWEY)
        .expect("Dewey AddNpc for a nearby player");
    assert_eq!(added.model_id, model_id as i32);
    assert_eq!(added.name_string_id, name_id as i32);
    let added_position = [added.position[0], added.position[1], added.position[2]];
    for (axis, (drawn, shared)) in added_position.iter().zip(position).enumerate() {
        assert!((drawn - shared).abs() < 1e-3, "axis {axis} differs");
    }

    // The trainer row the production loader accepts names Warrior profile 32.
    let content = Content::new();
    let trainer = warrior_trainer(&content);
    assert_eq!(trainer.profile, WARRIOR);
    assert_eq!(trainer.job, "Warrior");
    assert_eq!(trainer.label_string_id, BECOME_A_WARRIOR);
    let need = trainer.requires.as_ref().expect("Brawler prerequisite");
    assert_eq!(need.profile, BRAWLER);
    assert_eq!(need.job, "Brawler");
    assert_eq!(need.rank, 5);
    // The row documents the guide, the client gate text, the label and the (R) rank reduction.
    for needle in ["Warrior.md", "412471", "5100388", "Level 20 Brawler", "(R)"] {
        assert!(
            trainer.source.contains(needle),
            "source must document {needle:?}: {}",
            trainer.source
        );
    }
}

// ---- service level ------------------------------------------------------------------------

#[test]
fn warrior_requires_brawler_rank_5() {
    let content = Content::new();
    let trainer = warrior_trainer(&content);

    // Rank 4: refused, nothing changes.
    let mut profiles = profiles_with_brawler(&content, 4);
    assert_eq!(profiles.rank(BRAWLER), 4);
    assert!(profiles.state().unlocked.contains(&BRAWLER));
    let before = profiles.state().clone();
    assert_eq!(
        profiles.learn(&trainer),
        Err(LearnDenied::NeedsRank {
            job: "Brawler".into(),
            rank: 5,
        })
    );
    assert_eq!(profiles.state(), &before);
    assert!(!profiles.state().unlocked.contains(&WARRIOR));

    // Rank 5: exactly one unlock, and it does not switch the active job.
    let mut profiles = profiles_with_brawler(&content, 5);
    assert_eq!(profiles.rank(BRAWLER), 5);
    let before = profiles.state().clone();
    assert_eq!(profiles.learn(&trainer), Ok(()));
    let after = profiles.state().clone();
    let added: Vec<i32> = after
        .unlocked
        .difference(&before.unlocked)
        .copied()
        .collect();
    assert_eq!(added, vec![WARRIOR], "only Warrior is unlocked");
    assert_eq!(after.active, before.active, "learning must not switch jobs");
    assert_ne!(after.active, WARRIOR);
    assert_eq!(
        profiles.learn(&trainer),
        Err(LearnDenied::AlreadyKnown),
        "the unlock is granted once"
    );
    assert_eq!(profiles.state(), &after);
}

// ---- gateway level: the ordinary frame path -----------------------------------------------

#[test]
fn dewey_learns_warrior_through_the_npc_menu() {
    let content = Content::new();
    let near = near(DEWEY);
    let mut gateway = gateway_at(&content, gameplay_state(&content, 5), near);
    let before = unlocked(&gateway);
    assert!(!before.contains(&i64::from(WARRIOR)));
    let active_before = gameplay_json(&gateway)["profiles"]["active"].clone();

    let menu = click(&mut gateway, DEWEY, 2_000);
    let choice = option_with_label(&menu, BECOME_A_WARRIOR)
        .expect("Become a Warrior option in Dewey's menu");
    assert!(
        said(&menu, "Drill Sergeant Dewey"),
        "the 26/9 body names the clicked trainer"
    );
    let learned = choose(&mut gateway, DEWEY, choice, 2_100).unwrap();
    assert!(unlocked(&gateway).contains(&i64::from(WARRIOR)));
    assert!(said(&bodies(&learned), "You learned the Warrior"));
    let added: Vec<i64> = unlocked(&gateway)
        .into_iter()
        .filter(|profile| !before.contains(profile))
        .collect();
    assert_eq!(added, vec![i64::from(WARRIOR)], "only Warrior is unlocked");
    assert_eq!(
        gameplay_json(&gateway)["profiles"]["active"],
        active_before,
        "learning must not switch jobs"
    );
    assert_ne!(
        gameplay_json(&gateway)["profiles"]["active"],
        Value::from(WARRIOR)
    );

    // The same choice again is refused with a line and changes nothing.
    let learned_state = save_bytes(&gateway);
    click(&mut gateway, DEWEY, 3_000);
    let again = choose(&mut gateway, DEWEY, choice, 3_100).unwrap();
    assert!(said(&bodies(&again), "already know"));
    assert_eq!(save_bytes(&gateway), learned_state);

    // A second player under rank is refused and does not disturb the first player.
    let mut peer = gateway_at(&content, gameplay_state(&content, 4), near);
    let peer_state = save_bytes(&peer);
    let peer_menu = click(&mut peer, DEWEY, 2_000);
    let peer_choice =
        option_with_label(&peer_menu, BECOME_A_WARRIOR).expect("the trainer is still reachable");
    let refused = choose(&mut peer, DEWEY, peer_choice, 2_100).unwrap();
    assert!(said(&bodies(&refused), "level 5 Brawler"));
    assert_eq!(save_bytes(&peer), peer_state);
    assert!(!unlocked(&peer).contains(&i64::from(WARRIOR)));
    assert_eq!(save_bytes(&gateway), learned_state);
}

#[test]
fn refused_learn_changes_nothing() {
    let content = Content::new();
    let near = near(DEWEY);

    // Under rank: a chat refusal, the menu stays open, and the same choice stays understood.
    // The same gateway then proves two more refusals, because building one costs a full
    // content load.
    let mut gateway = gateway_at(&content, gameplay_state(&content, 4), near);
    let menu = click(&mut gateway, DEWEY, 2_000);
    let choice = option_with_label(&menu, BECOME_A_WARRIOR).unwrap();
    let before = save_bytes(&gateway);
    let refused = choose(&mut gateway, DEWEY, choice, 2_100).unwrap();
    assert!(said(&bodies(&refused), "level 5 Brawler"));
    assert!(
        !refused.state_changed && !refused.gameplay_changed,
        "a prerequisite refusal must not commit"
    );
    assert_eq!(save_bytes(&gateway), before);
    assert!(!unlocked(&gateway).contains(&i64::from(WARRIOR)));
    let again = choose(&mut gateway, DEWEY, choice, 2_200).unwrap();
    assert!(
        said(&bodies(&again), "level 5 Brawler"),
        "the refused choice must not consume the open menu"
    );
    assert_eq!(save_bytes(&gateway), before);

    // A forged selection (never offered) changes nothing.
    let _ = choose(&mut gateway, DEWEY, 9_999, 2_300);
    assert_eq!(save_bytes(&gateway), before);
    assert!(!unlocked(&gateway).contains(&i64::from(WARRIOR)));

    // A selection addressed to another NPC never teaches Warrior.
    let _ = choose(&mut gateway, SMITH, choice, 2_400);
    assert_eq!(save_bytes(&gateway), before);
    assert!(!unlocked(&gateway).contains(&i64::from(WARRIOR)));

    // A busy player can neither open the trainer menu nor replay an option from a menu
    // opened before the activity started. The shared busy gate refuses the interaction
    // (26/8) and the selection (26/10) with no state change.
    let mut state = gameplay_state(&content, 5);
    let profiles = state.profiles.as_mut().unwrap();
    profiles.active = 14;
    profiles.unlocked.insert(14);
    let mut gateway = gateway_at(&content, state, near);
    gateway.set_dev_tools(true);
    gateway.set_role(AccountRole::Admin);
    // Open the menu while free, so the replay below names a genuinely offered option.
    let menu = click(&mut gateway, DEWEY, 2_450);
    let choice = option_with_label(&menu, BECOME_A_WARRIOR).unwrap();
    let started = send(&mut gateway, &chat_inner("realm mine"), 2_500).unwrap();
    assert!(
        said(&bodies(&started), "Mining test started"),
        "{}",
        String::from_utf8_lossy(&started.replies.concat())
    );
    let before = save_bytes(&gateway);
    let busy_click = click_result(&mut gateway, DEWEY, 2_600).unwrap();
    assert!(
        bodies(&busy_click)
            .iter()
            .all(|reply| !reply.starts_with(&[26, 0, 9, 0])),
        "a busy player must not open a trainer menu"
    );
    assert!(said(
        &bodies(&busy_click),
        "Finish the active activity before interacting"
    ));
    assert_eq!(save_bytes(&gateway), before);
    let refused = choose(&mut gateway, DEWEY, choice, 2_700).unwrap();
    assert!(said(
        &bodies(&refused),
        "Finish the active activity before interacting"
    ));
    assert_eq!(save_bytes(&gateway), before);
    assert!(!unlocked(&gateway).contains(&i64::from(WARRIOR)));

    // Out of range: no menu and no unlock, even with an option id that was offered nearby.
    let far_position = [near[0] + 400.0, near[1], near[2]];
    let mut far = gateway_at(&content, gameplay_state(&content, 5), far_position);
    let before = save_bytes(&far);
    let far_menu = click_result(&mut far, DEWEY, 2_000).unwrap();
    assert!(
        bodies(&far_menu)
            .iter()
            .all(|reply| !reply.starts_with(&[26, 0, 9, 0])),
        "a trainer outside interaction range must not open a menu"
    );
    let _ = choose(&mut far, DEWEY, choice, 2_100);
    assert_eq!(save_bytes(&far), before);
    assert!(!unlocked(&far).contains(&i64::from(WARRIOR)));
}

#[test]
fn warrior_unlock_survives_reload() {
    let content = Content::new();
    let near = near(DEWEY);
    let mut gateway = gateway_at(&content, gameplay_state(&content, 5), near);
    let active_before = gameplay_json(&gateway)["profiles"]["active"].clone();

    let menu = click(&mut gateway, DEWEY, 2_000);
    let choice = option_with_label(&menu, BECOME_A_WARRIOR).unwrap();
    let learned = choose(&mut gateway, DEWEY, choice, 2_100).unwrap();
    let learned_bodies = bodies(&learned);
    assert!(said(&learned_bodies, "You learned the Warrior"));
    assert!(unlocked(&gateway).contains(&i64::from(WARRIOR)));

    // The production player package ships starter items for profile 32; a test package
    // may not. Learning must still work and say the outfit is unavailable.
    let starter = starter_items(&content, WARRIOR);
    if starter.is_empty() {
        assert!(said(&learned_bodies, "starter outfit is unavailable"));
        assert_eq!(equipped(&gateway, WARRIOR), 0);
    } else {
        assert!(
            !said(&learned_bodies, "starter outfit is unavailable"),
            "a package with starter items must equip the outfit"
        );
        assert!(equipped(&gateway, WARRIOR) > 0);
    }

    // Save through the real store and log in again with that document.
    let directory = std::env::temp_dir().join(format!(
        "osfr-warrior-save-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&directory).unwrap();
    let path = directory.join("snapshot.json");
    let saved = gateway.snapshot();
    game_store::save(&path, &saved).unwrap();
    let loaded = game_store::load_or_create(&path, saved.character.clone()).unwrap();
    fs::remove_file(&path).unwrap();
    fs::remove_dir(&directory).unwrap();
    let mut reloaded =
        Gateway::load_gameplay(&reference(), loaded.character, &content.0, loaded.gameplay)
            .unwrap();
    login(&mut reloaded);

    assert!(unlocked(&reloaded).contains(&i64::from(WARRIOR)));
    assert_eq!(
        gameplay_json(&reloaded)["profiles"]["active"],
        active_before,
        "reload must not switch the active job"
    );
    assert_eq!(
        gameplay_json(&reloaded),
        gameplay_json(&gateway),
        "the unlock and starter grant live in the save"
    );

    // Selecting Learn again after reload is AlreadyKnown and grants nothing new.
    let before = save_bytes(&reloaded);
    let menu = click(&mut reloaded, DEWEY, 3_000);
    let choice = option_with_label(&menu, BECOME_A_WARRIOR).expect("the trainer still offers it");
    let again = choose(&mut reloaded, DEWEY, choice, 3_100).unwrap();
    assert!(said(&bodies(&again), "already know"));
    assert_eq!(save_bytes(&reloaded), before);
    if starter.is_empty() {
        assert_eq!(equipped(&reloaded, WARRIOR), 0);
    } else {
        assert!(equipped(&reloaded, WARRIOR) > 0, "no second outfit");
    }
}

#[test]
fn non_warrior_trainers_do_not_offer_warrior() {
    let content = Content::new();
    let smith = trainers(&content)
        .into_iter()
        .find(|trainer| trainer.npc == SMITH)
        .expect("Blacksmith trainer row");
    let mut gateway = gateway_at(&content, gameplay_state(&content, 5), near(SMITH));
    let menu = click(&mut gateway, SMITH, 2_000);
    assert!(
        option_with_label(&menu, smith.label_string_id).is_some(),
        "the Blacksmith trainer still offers his own job"
    );
    assert!(
        option_with_label(&menu, BECOME_A_WARRIOR).is_none(),
        "only Dewey offers Warrior"
    );
    assert!(!unlocked(&gateway).contains(&i64::from(WARRIOR)));
}
