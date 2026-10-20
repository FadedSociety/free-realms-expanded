//! T021 acceptance: soccer from the activity portal's Play request through the
//! shared minigame result and one payout, plus refusals that change nothing.
//!
//! Uses the same private opening content as `activity_portal_route.rs` and
//! writes a test-owned `soccer.json` with a short match so the gate is bounded.
//! No player names or chat text are embedded.

use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    game_store,
    gameplay::Gameplay,
    gateway::{Gateway, GatewayOutcome},
    progression_extras,
    rewards::{ItemGrant, RewardDisplay, RewardGrant},
    soccer::{self, SoccerPacket},
    soccer_service::FIRST_INSTANCE_ID,
    world::{World, WorldPose},
};
use serde_json::json;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const GUID: u64 = 0x0000_0010_0080_06b2;
const SOCCER_ACTIVITY: u32 = 43;
static NEXT: AtomicU64 = AtomicU64::new(0);
/// Trusted server claim issued to this session while a match runs.
const CLAIM: u64 = 0x0f26_0000_0000_0001;
const CLAIM_TRACKER: u32 = 0xBEEF;
const CLAIM_COINS: u32 = 25;
const CLAIM_ITEMS: u32 = 3;

fn source_content() -> PathBuf {
    std::env::var_os("OSFR_SOCCER_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/rust-opening-cooking-original-assets")
        })
}

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let source = source_content();
        assert!(
            source.join("opening.json").is_file(),
            "T021 needs the private opening content at {}",
            source.display()
        );
        let directory = std::env::temp_dir().join(format!(
            "osfr-soccer-entry-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        copy_files(&source, &directory);
        fs::write(directory.join("soccer.json"), soccer_content()).unwrap();
        Self(directory)
    }

    fn game(&self) -> Gameplay {
        Gameplay::load(&self.0, None).unwrap()
    }

    fn character(&self) -> CharacterState {
        Gameplay::initial_character(
            &self.0,
            CharacterState {
                guid: GUID,
                zone_id: 0,
                position: [0.0; 3],
                rotation: [0.0, 0.0, 1.0],
            },
        )
        .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn copy_files(source: &Path, target: &Path) {
    for entry in fs::read_dir(source).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            let name = path.file_name().unwrap();
            fs::copy(&path, target.join(name)).unwrap();
        }
    }
}

fn soccer_content() -> Vec<u8> {
    soccer_content_with_reward(0)
}

/// The same short match with an optional one-win job XP reward. The reward
/// reaches the bank from the tick payout (`soccer_entry::Session::tick`), not
/// from a request handler, which is the path the achievement repair covers.
fn soccer_content_with_reward(job_xp: u32) -> Vec<u8> {
    let job_xp = if job_xp == 0 {
        json!({})
    } else {
        json!({ "1": job_xp })
    };
    serde_json::to_vec(&json!({
        "schema_version": 1,
        "source": "T021 synthetic short-match fixture; recovered ids and category only",
        "games": [{
            "id": SOCCER_ACTIVITY,
            "name_id": 37945,
            "description_id": 37948,
            "icon_id": 20992,
            "difficulty": 1,
            "required_level": 0,
            "encounter_id": 2
        }],
        "match_config": {
            "bounds": {
                "half_x": 50.0,
                "half_z": 10.0,
                "goal_half_width": 8.0,
                "goal_height": 5.0
            },
            "physics": {
                "gravity": 0.0,
                "max_position_step": 10.0,
                "max_speed": 40.0,
                "control_radius": 2.0,
                "max_kick_power": 4.0,
                "pass_radius": 40.0,
                "max_tick": 0.25
            },
            "half_seconds": 6.0,
            "kickoff_spawn": [0.0, 0.0, 0.0],
            "half_policy": "keep_sides",
            "adapter": {
                "max_power": 4.0,
                "charge_floor": 0.01,
                "min_direction_norm": 0.001
            },
            "entry_position": [0.0, 0.0, 0.0, 1.0],
            "entry_rotation": [0.0, 0.0, 1.0, 0.0]
        },
        "reward": {"coins": 10, "job_xp": job_xp}
    }))
    .unwrap()
}

fn play(activity: u32) -> Vec<u8> {
    let mut inner = vec![167, 0, 1, 2];
    inner.extend(activity.to_le_bytes());
    inner.extend(0u32.to_le_bytes());
    inner
}

fn lifecycle(sub: u8, instance: i32) -> Vec<u8> {
    let mut packet = vec![39, 0, sub];
    for value in [instance, -1, -1] {
        packet.extend(value.to_le_bytes());
    }
    packet
}

fn offer_instance(replies: &[Vec<u8>]) -> i32 {
    let offer = replies
        .iter()
        .find(|reply| reply.starts_with(&[39, 0, 16]))
        .expect("39/16 soccer offer");
    i32::from_le_bytes(offer[3..7].try_into().unwrap())
}

fn portal_rows(list: &[u8]) -> Vec<(u32, u32, String, u8)> {
    assert_eq!(&list[..4], &[167, 0, 1, 1]);
    let count = u32::from_le_bytes(list[8..12].try_into().unwrap()) as usize;
    let mut offset = 12;
    let mut rows = Vec::new();
    for _ in 0..count {
        let id = u32::from_le_bytes(list[offset..offset + 4].try_into().unwrap());
        let category = u32::from_le_bytes(list[offset + 8..offset + 12].try_into().unwrap());
        offset += 40;
        let can_join = list[offset];
        offset += 1 + 4 + 4 + 4 + 1;
        let mut art = String::new();
        for field in 0..2 {
            let length = u32::from_le_bytes(list[offset..offset + 4].try_into().unwrap()) as usize;
            offset += 4;
            if field == 0 {
                art = String::from_utf8(list[offset..offset + length].to_vec()).unwrap();
            }
            offset += length;
        }
        offset += 16;
        rows.push((id, category, art, can_join));
    }
    assert_eq!(offset, list.len(), "portal list has no trailing bytes");
    rows
}

fn position(inner_player: u64, position: [f32; 3]) -> Vec<u8> {
    soccer::encode_inner(&SoccerPacket::UpdatePlayerPos {
        player: inner_player,
        position,
        tail: [0; 45],
    })
}

fn shot(inner_player: u64) -> Vec<u8> {
    soccer::encode_inner(&SoccerPacket::UpdatePlayerState {
        player: inner_player,
        state: 17,
        aux: 255,
        state_counter: 1,
        vectors: [[0.0, 0.0, 1.0, 1.0], [0.0; 4], [0.0; 4]],
    })
}

fn client_goal() -> Vec<u8> {
    soccer::encode_inner(&SoccerPacket::GoalMade { fields: [0, 1, 1] })
}

#[test]
fn portal_play_reaches_a_server_result_and_pays_once() {
    let fixture = Fixture::new();
    let mut game = fixture.game();
    let character = fixture.character();
    game.initialize_world(&character, 500).unwrap();
    let coins_before = game.inventory_state().coins;
    let listed = game
        .handle_in_world(&[167, 0, 1, 4], &character, 600)
        .unwrap();
    let rows = portal_rows(&listed.replies[0]);
    let soccer = rows
        .iter()
        .find(|(id, _, _, _)| *id == SOCCER_ACTIVITY)
        .expect("soccer row is published");
    assert_eq!(soccer.1, 22, "client soccer category");
    assert_eq!(soccer.2, "Soccer_Briarwood.dds");
    assert_eq!(soccer.3, 1, "soccer does not need a combat job");

    let played = game
        .handle_in_world(&play(SOCCER_ACTIVITY), &character, 1_000)
        .unwrap();
    let scene = played.scene_zoning.expect("Play zones into sg_soccer");
    assert_eq!(scene.scene, "sg_soccer");
    assert!(played.changed);

    // The client's Loaded after the scene change receives the roster and offer.
    let loaded = game.initialize_world(&character, 1_100).unwrap();
    assert!(
        loaded
            .replies
            .iter()
            .any(|reply| reply.starts_with(&[76, 0, 3, 0])),
        "server registration"
    );
    assert!(
        loaded
            .replies
            .iter()
            .any(|reply| reply.starts_with(&[76, 0, 6, 0])),
        "team assignment"
    );
    let instance = offer_instance(&loaded.replies);

    let started = game
        .handle_in_world(&lifecycle(5, instance), &character, 1_200)
        .unwrap();
    assert!(
        started
            .replies
            .iter()
            .any(|reply| reply.starts_with(&[41, 0, 2, 0])),
        "shared encounter start"
    );
    assert!(
        started
            .replies
            .iter()
            .any(|reply| reply.starts_with(&[76, 0, 23, 0])),
        "kickoff clock"
    );

    game.handle_in_world(&position(GUID, [0.0, 0.0, 0.0]), &character, 1_200)
        .unwrap();
    game.tick(&character, 1_450).unwrap();
    game.handle_in_world(&shot(GUID), &character, 1_450)
        .unwrap();

    let mut now = 1_450;
    let mut won = false;
    let mut lost = false;
    for _ in 0..80 {
        now += 250;
        let out = game.tick(&character, now).unwrap();
        for reply in &out.replies {
            if reply.starts_with(&[39, 0, 18]) && reply.get(15) == Some(&1) {
                won = true;
            }
            if reply.starts_with(&[39, 0, 18]) && reply.get(15) == Some(&0) {
                lost = true;
            }
        }
        if won {
            break;
        }
    }
    assert!(won, "the server-scored goal settles as a win");
    assert!(!lost);
    assert_eq!(
        game.inventory_state().coins,
        coins_before + 10,
        "one payout"
    );

    // A second tick cannot pay again.
    let after = game.tick(&character, now + 250).unwrap();
    assert!(after.replies.is_empty(), "{:?}", after.replies);
    assert_eq!(game.inventory_state().coins, coins_before + 10);

    let closed = game
        .handle_in_world(&[26, 0, 42, 0], &character, now + 500)
        .unwrap();
    assert!(!game.is_soccer_active(), "results Close releases the match");
    assert!(closed.scene_zoning.is_some(), "Close returns to the world");
}

#[test]
fn forged_entries_and_client_goals_change_nothing() {
    let fixture = Fixture::new();
    let mut game = fixture.game();
    let character = fixture.character();
    game.initialize_world(&character, 500).unwrap();
    let before = game.state();

    let unknown = game.handle_in_world(&play(999), &character, 1_000).unwrap();
    assert!(unknown.scene_zoning.is_none());
    assert!(!unknown.changed);
    assert!(text(&unknown).contains("not available"));
    assert_eq!(game.state(), before);

    let stray = game
        .handle_in_world(&position(GUID, [0.0, 0.0, 0.0]), &character, 1_000)
        .unwrap();
    assert!(stray.replies.is_empty());
    assert!(!stray.changed);
    assert_eq!(game.state(), before);

    assert!(
        game.handle_in_world(&[167, 0, 1, 2, 1, 0, 0, 0], &character, 1_000)
            .is_err(),
        "malformed Play is refused"
    );
    assert_eq!(game.state(), before);

    // Enter a real match, then prove a client goal and a second Play change no state.
    game.handle_in_world(&play(SOCCER_ACTIVITY), &character, 2_000)
        .unwrap();
    game.initialize_world(&character, 2_100).unwrap();
    let before_match = game.state();
    let forged_goal = match game.handle_in_world(&client_goal(), &character, 2_200) {
        Err(error) => error,
        Ok(_) => panic!("client GoalMade is refused"),
    };
    assert!(forged_goal.to_string().contains("goal"), "{forged_goal}");
    assert_eq!(game.soccer_score(), Some([0, 0]));
    assert_eq!(game.state(), before_match);

    let second = game
        .handle_in_world(&play(SOCCER_ACTIVITY), &character, 2_300)
        .unwrap();
    assert!(second.scene_zoning.is_none());
    assert!(text(&second).contains("Finish what you are doing"));
    assert_eq!(game.state(), before_match);
}

#[test]
fn quitting_before_a_result_cancels_without_a_payout() {
    let fixture = Fixture::new();
    let mut game = fixture.game();
    let character = fixture.character();
    game.initialize_world(&character, 500).unwrap();
    let coins_before = game.inventory_state().coins;

    game.handle_in_world(&play(SOCCER_ACTIVITY), &character, 1_000)
        .unwrap();
    let loaded = game.initialize_world(&character, 1_100).unwrap();
    let instance = offer_instance(&loaded.replies);
    game.handle_in_world(&lifecycle(5, instance), &character, 1_200)
        .unwrap();
    game.tick(&character, 1_450).unwrap();

    let quit = game
        .handle_in_world(&lifecycle(6, instance), &character, 1_500)
        .unwrap();
    assert!(
        quit.replies
            .iter()
            .any(|reply| reply.starts_with(&[39, 0, 19])),
        "ordinary quit removes the run"
    );
    assert!(quit.scene_zoning.is_some(), "quit returns to the world");
    assert!(!game.is_soccer_active());
    assert_eq!(game.inventory_state().coins, coins_before);
    game.tick(&character, 2_000).unwrap();
    assert_eq!(game.inventory_state().coins, coins_before);
}

fn text(out: &osfr_tools::gameplay::Outcome) -> String {
    out.replies
        .iter()
        .filter(|reply| reply.starts_with(&[15, 0, 3, 0]))
        .map(|reply| {
            let length = u32::from_le_bytes(reply[4..8].try_into().unwrap()) as usize;
            String::from_utf8_lossy(&reply[8..8 + length]).into_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

fn gateway_send(gateway: &mut Gateway, inner: &[u8], now_ms: u64) -> GatewayOutcome {
    gateway
        .handle_at(&wrap(inner), now_ms)
        .expect("gateway frame")
}

fn gateway_inner(out: &GatewayOutcome) -> Vec<Vec<u8>> {
    out.replies
        .iter()
        .map(|packet| {
            if packet.starts_with(&[5, 0, 1]) && packet.len() >= 7 {
                packet[7..].to_vec()
            } else {
                packet.clone()
            }
        })
        .collect()
}

fn reference_dir() -> PathBuf {
    std::env::var_os("OSFR_GATEWAY_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/rust-gateway-reference")
        })
}

fn client_version(reference: &Path) -> String {
    serde_json::from_slice::<serde_json::Value>(&fs::read(reference.join("manifest.json")).unwrap())
        .unwrap()["client_version"]
        .as_str()
        .unwrap()
        .to_owned()
}

/// Login, world entry and world load. The next portal Play opens soccer.
fn enter_world(fixture: &Fixture, xp: u32, ticket: &str) -> Gateway {
    let mut state = Gameplay::load(&fixture.0, None).unwrap().state();
    let profiles = state.profiles.as_mut().unwrap();
    profiles.active = 1;
    profiles.unlocked.insert(1);
    profiles.experience.insert(1, xp);

    let reference = reference_dir();
    let version = client_version(&reference);
    let character = fixture.character();
    let mut gateway =
        Gateway::load_gameplay(&reference, character.clone(), &fixture.0, Some(state)).unwrap();
    gateway.set_dev_tools(false);
    gateway.set_role(AccountRole::Player);
    let mut login = 1u16.to_le_bytes().to_vec();
    for (index, text) in [ticket, version.as_str(), ""].into_iter().enumerate() {
        login.extend((text.len() as u32).to_le_bytes());
        login.extend(text.as_bytes());
        if index == 0 {
            login.extend(character.guid.to_le_bytes());
        }
    }
    gateway.handle_at(&login, 0).unwrap();
    gateway_send(&mut gateway, &13u16.to_le_bytes(), 100);
    gateway_send(&mut gateway, &10u16.to_le_bytes(), 200);
    gateway
}

/// Portal Play through Loaded: returns the offered lifecycle instance id and
/// the Loaded time.
fn enter_match(gateway: &mut Gateway, at: u64) -> (i32, u64) {
    let played = gateway_send(gateway, &play(SOCCER_ACTIVITY), at);
    assert!(played.began_zoning, "Play zones into the soccer scene");
    gateway_send(gateway, &13u16.to_le_bytes(), at + 100);
    let loaded = gateway_send(gateway, &10u16.to_le_bytes(), at + 200);
    (offer_instance(&gateway_inner(&loaded)), at + 200)
}

/// Shoot until the server settles the win. Returns the inner replies and the
/// time of the settling tick.
fn settle_win(gateway: &mut Gateway, at: u64) -> (Vec<Vec<u8>>, u64) {
    let guid = gateway.state().guid;
    gateway_send(gateway, &position(guid, [0.0, 0.0, 0.0]), at);
    gateway.tick_at(at + 150).unwrap();
    gateway_send(gateway, &shot(guid), at + 150);

    let mut now = at + 150;
    let mut packets = Vec::new();
    let mut won = false;
    for _ in 0..80 {
        now += 250;
        let out = gateway.tick_at(now).unwrap();
        let batch = gateway_inner(&out);
        if batch
            .iter()
            .any(|reply| reply.starts_with(&[39, 0, 18]) && reply.get(15) == Some(&1))
        {
            won = true;
        }
        packets.extend(batch);
        if won {
            break;
        }
    }
    assert!(won, "the server-scored goal settles as a win");
    (packets, now)
}

fn coins(gateway: &Gateway) -> u32 {
    gateway
        .snapshot()
        .gameplay
        .expect("gameplay")
        .inventory
        .coins
}

fn item_count(gateway: &Gateway, definition_id: i32) -> u32 {
    gateway
        .snapshot()
        .gameplay
        .expect("gameplay")
        .inventory
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id)
        .map(|item| item.count)
        .sum()
}

fn claimed(gateway: &Gateway) -> Vec<u64> {
    gateway
        .snapshot()
        .gameplay
        .expect("gameplay")
        .progression_extras
        .map(|state| state.claimed_ids.into_iter().collect())
        .unwrap_or_default()
}

/// First definition of the private opening inventory fixture; the claim grants
/// that real catalogue item so the bag diff is observable.
fn fixture_definition(fixture: &Fixture) -> i32 {
    let bytes = fs::read(fixture.0.join("inventory.json")).unwrap();
    let content: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    i32::try_from(content["definitions"][0]["id"].as_i64().unwrap()).unwrap()
}

fn claim_grant(definition_id: i32) -> RewardGrant {
    RewardGrant {
        coins: CLAIM_COINS,
        items: vec![ItemGrant {
            definition_id,
            count: CLAIM_ITEMS,
        }],
        job_xp: BTreeMap::new(),
        display: RewardDisplay {
            source_guid: 0,
            player_guid: GUID,
            reward_kind: 0,
            icon_id: 0,
            name_id: 0,
            rows: Vec::new(),
        },
    }
}

/// The 108/4 result row: tracker u32 then the success byte.
fn claim_reply(replies: &[Vec<u8>]) -> &[u8] {
    replies
        .iter()
        .find(|reply| reply.starts_with(&[108, 0, 4, 0]))
        .map(Vec::as_slice)
        .expect("claim result packet")
}

/// Login, play the short paid match through the activity portal and shoot until
/// the server settles a win. The win pays 50 job XP from `Gameplay::tick`, the
/// tick-only path the achievement repair covers.
fn play_a_paid_win(fixture: &Fixture, xp: u32, ticket: &str) -> (Gateway, Vec<Vec<u8>>, u64) {
    fs::write(
        fixture.0.join("soccer.json"),
        soccer_content_with_reward(50),
    )
    .unwrap();
    let mut gateway = enter_world(fixture, xp, ticket);
    let (instance, loaded_at) = enter_match(&mut gateway, 1_000);
    gateway_send(&mut gateway, &lifecycle(5, instance), loaded_at + 100);
    let (packets, now) = settle_win(&mut gateway, loaded_at + 100);
    (gateway, packets, now)
}

fn adventurer_title_id() -> u32 {
    progression_extras::parse_title_content(
        &fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/titles.json"))
            .unwrap(),
    )
    .unwrap()
    .definitions
    .iter()
    .find(|title| title.name == "The Adventurer Enthusiast")
    .and_then(|title| title.client_id)
    .expect("the shipped level-20 Adventurer title")
}

/// A paid match is a tick-only reward: the finish settlement reaches the bank
/// from `Gameplay::tick`, outside every application request. Its rank crossing
/// must award the bound title in that same tick, so the player can select it
/// without reconnecting.
#[test]
fn tick_reward_crossing_a_rank_awards_its_title_without_reconnect() {
    let fixture = Fixture::new();
    let title_id = adventurer_title_id();
    // Rank 19: the 50 XP tick payout crosses the level-20 threshold.
    let (mut gateway, packets, now) = play_a_paid_win(&fixture, 1_850, "soccer-achievement");
    assert!(
        packets.iter().any(|packet| {
            packet.starts_with(&[152, 0, 1, 0])
                && packet.get(6..10) == Some(&title_id.to_le_bytes())
        }),
        "the tick payout did not award the level-20 title: {packets:?}"
    );

    let selected = gateway_send(
        &mut gateway,
        &progression_extras::title_select_request(title_id),
        now + 1_000,
    );
    let replies = gateway_inner(&selected);
    assert!(
        replies.iter().any(|reply| {
            reply.starts_with(&[152, 0, 3, 0]) && reply.get(30..34) == Some(&title_id.to_le_bytes())
        }),
        "the tick-awarded title could not be selected without reconnecting: {replies:?}"
    );
    let snapshot = gateway.snapshot();
    let titles = snapshot
        .gameplay
        .as_ref()
        .unwrap()
        .progression_extras
        .as_ref()
        .unwrap()
        .titles
        .clone();
    assert_eq!(titles.active(), Some(title_id));

    // The tick marks the character changed, so the gateway saves the staged
    // state; prove the title and the active selection survive the same
    // `save_with_pet_roster` path the session uses and a real reload.
    let path = fixture.0.join("tick-title-save.json");
    gateway.save_with_pet_roster(&path).unwrap();
    let reloaded = game_store::load_or_create(&path, snapshot.character.clone()).unwrap();
    let saved = &reloaded
        .gameplay
        .as_ref()
        .and_then(|state| state.progression_extras.as_ref())
        .expect("the tick-awarded title was saved")
        .titles;
    assert!(saved.contains(title_id), "the title was lost on reload");
    assert_eq!(
        saved.active(),
        Some(title_id),
        "the active title selection was lost on reload"
    );
}

/// A progress-only counter move stays session-local (F121): the tick settle
/// observes the moved counter, but no `150/7` packet is sent and a later tick
/// must not add one.
#[test]
fn progress_only_tick_reward_does_not_emit_achievement_progress() {
    let fixture = Fixture::new();
    // Rank 18 before the win: the payout moves the achievement counter from 17
    // to 18, still below the level-20 threshold of 19.
    let (mut gateway, packets, now) = play_a_paid_win(&fixture, 1_750, "soccer-progress");
    assert!(
        packets
            .iter()
            .all(|packet| !packet.starts_with(&[150, 0, 7, 0])),
        "a progress-only gain emitted an achievement progress packet: {packets:?}"
    );
    let next = gateway_inner(&gateway.tick_at(now + 250).unwrap());
    assert!(
        next.iter()
            .all(|packet| !packet.starts_with(&[150, 0, 7, 0])),
        "a later tick re-emitted the achievement progress packet: {next:?}"
    );
    let xp = gateway
        .snapshot()
        .gameplay
        .unwrap()
        .profiles
        .unwrap()
        .experience
        .get(&1)
        .copied()
        .unwrap();
    assert!(
        (1_800..1_900).contains(&xp),
        "the win must move the rank counter without crossing level 20: {xp}"
    );
}

/// T021 review (F226): the win payout used to replace the bag with the entry
/// snapshot, so a trusted claim redeemed mid-match was wiped. The settlement
/// now applies its grant on the live wallet and bag.
#[test]
fn a_claim_made_during_a_match_survives_the_payout() {
    let fixture = Fixture::new();
    let definition = fixture_definition(&fixture);
    let mut gateway = enter_world(&fixture, 0, "soccer-claim-during-match");
    let (instance, loaded_at) = enter_match(&mut gateway, 1_000);
    gateway_send(&mut gateway, &lifecycle(5, instance), loaded_at + 100);

    let coins_before = coins(&gateway);
    let items_before = item_count(&gateway, definition);
    gateway.issue_claim(CLAIM, claim_grant(definition)).unwrap();
    let claimed_out = gateway_send(
        &mut gateway,
        &progression_extras::claim_request(CLAIM_TRACKER, &[CLAIM]).unwrap(),
        loaded_at + 200,
    );
    let claimed_replies = gateway_inner(&claimed_out);
    let reply = claim_reply(&claimed_replies);
    assert_eq!(&reply[4..8], &CLAIM_TRACKER.to_le_bytes());
    assert_eq!(reply[8], 1, "the trusted claim is paid: {reply:?}");
    assert_eq!(coins(&gateway), coins_before + CLAIM_COINS);
    assert_eq!(item_count(&gateway, definition), items_before + CLAIM_ITEMS);
    assert_eq!(claimed(&gateway), vec![CLAIM]);

    let (_, now) = settle_win(&mut gateway, loaded_at + 300);
    assert_eq!(
        coins(&gateway),
        coins_before + CLAIM_COINS + 10,
        "the win grant lands on the live wallet, not the entry snapshot"
    );
    assert_eq!(
        item_count(&gateway, definition),
        items_before + CLAIM_ITEMS,
        "the claim's item survives the payout"
    );
    assert_eq!(
        claimed(&gateway),
        vec![CLAIM],
        "the claim ledger survives the payout"
    );

    let after = gateway_inner(&gateway.tick_at(now + 250).unwrap());
    assert!(
        after.is_empty(),
        "a settled match does not pay twice: {after:?}"
    );
    assert_eq!(coins(&gateway), coins_before + CLAIM_COINS + 10);
}

/// A claim request that names no queued claim is refused without touching the
/// wallet, the bag, the ledger or the running match.
#[test]
fn a_refused_claim_during_a_match_changes_nothing() {
    let fixture = Fixture::new();
    let mut gateway = enter_world(&fixture, 0, "soccer-refused-claim");
    let (instance, loaded_at) = enter_match(&mut gateway, 1_000);
    gateway_send(&mut gateway, &lifecycle(5, instance), loaded_at + 100);
    let coins_before = coins(&gateway);

    let before = gateway.snapshot();
    let refused = gateway_send(
        &mut gateway,
        &progression_extras::claim_request(0x1111, &[CLAIM]).unwrap(),
        loaded_at + 200,
    );
    let refused_replies = gateway_inner(&refused);
    let reply = claim_reply(&refused_replies);
    assert_eq!(&reply[4..8], &0x1111u32.to_le_bytes());
    assert_eq!(reply[8], 0, "an unqueued claim id is refused: {reply:?}");
    assert_eq!(
        gateway.snapshot(),
        before,
        "a refused claim changes nothing"
    );
    assert_eq!(coins(&gateway), coins_before);
    assert!(claimed(&gateway).is_empty(), "no claim was recorded");

    // The refusal consumed neither the match nor its single payout.
    settle_win(&mut gateway, loaded_at + 300);
    assert_eq!(coins(&gateway), coins_before + 10);
}

/// T021 review (F226): each match used a fresh service whose lifecycle counter
/// started at 1, so a replayed quit from match A cancelled match B.
#[test]
fn consecutive_matches_get_fresh_instance_ids_and_ignore_stale_lifecycle_packets() {
    let fixture = Fixture::new();
    let mut gateway = enter_world(&fixture, 0, "soccer-two-matches");
    let coins_before = coins(&gateway);

    let (first, first_loaded) = enter_match(&mut gateway, 1_000);
    gateway_send(&mut gateway, &lifecycle(5, first), first_loaded + 100);
    let quit = gateway_send(&mut gateway, &lifecycle(6, first), first_loaded + 200);
    let quit_replies = gateway_inner(&quit);
    assert!(
        quit_replies
            .iter()
            .any(|reply| reply.starts_with(&[39, 0, 19])),
        "the first quit removes its run: {quit_replies:?}"
    );
    assert!(quit.began_zoning, "quitting returns to the world");

    // The client loads the return scene before it can use the portal again.
    gateway_send(&mut gateway, &13u16.to_le_bytes(), first_loaded + 300);
    gateway_send(&mut gateway, &10u16.to_le_bytes(), first_loaded + 400);
    let (second, second_loaded) = enter_match(&mut gateway, first_loaded + 500);
    assert_ne!(
        second, first,
        "a later match must not reuse the earlier lifecycle id"
    );
    assert!(
        first >= FIRST_INSTANCE_ID && second >= FIRST_INSTANCE_ID,
        "lifecycle ids come from the shared counter: {first}, {second}"
    );

    gateway_send(&mut gateway, &lifecycle(5, second), second_loaded + 100);
    let stale_quit = gateway_send(&mut gateway, &lifecycle(6, first), second_loaded + 200);
    assert!(
        !stale_quit.began_zoning,
        "a stale quit must not zone the player out"
    );
    let stale_quit_replies = gateway_inner(&stale_quit);
    assert!(
        stale_quit_replies
            .iter()
            .all(|reply| !reply.starts_with(&[39, 0, 19])),
        "a stale quit cancelled the new run: {stale_quit_replies:?}"
    );
    let stale_start = gateway_send(&mut gateway, &lifecycle(5, first), second_loaded + 300);
    assert!(!stale_start.began_zoning);
    let stale_start_replies = gateway_inner(&stale_start);
    assert!(
        stale_start_replies
            .iter()
            .all(|reply| !reply.starts_with(&[41, 0, 2, 0])),
        "a stale start restarted the new run: {stale_start_replies:?}"
    );

    settle_win(&mut gateway, second_loaded + 400);
    assert_eq!(
        coins(&gateway),
        coins_before + 10,
        "the untouched second match settles once"
    );
}

/// T021 review (F226): scene id 0 named the public overworld instance. The
/// arena now carries its own server-owned scene id and the membership path
/// gives each entry its own instance.
#[test]
fn soccer_scene_is_not_a_public_zone_and_gets_its_own_instance() {
    let fixture = Fixture::new();
    let mut gateway = enter_world(&fixture, 0, "soccer-scene-instance");
    let (_, _) = enter_match(&mut gateway, 1_000);
    let zone = gateway.state().zone_id;
    assert_ne!(
        zone, 0,
        "soccer must not alias the public overworld instance"
    );

    let zones: serde_json::Value = serde_json::from_slice(
        &fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/zones.json"))
            .unwrap(),
    )
    .unwrap();
    for record in zones["records"].as_array().unwrap() {
        assert_ne!(
            u64::from(zone),
            record["id"].as_u64().unwrap(),
            "the soccer scene id is a public zone id"
        );
    }

    let runtime = gateway.runtime_zone().unwrap();
    assert_eq!(
        runtime.id, zone,
        "runtime-zone metadata reports the match scene"
    );
    assert_eq!(runtime.name, "sg_soccer");

    let mut world = World::runtime();
    world.register_runtime_zone(runtime).unwrap();
    assert_eq!(
        world.public_instance(zone),
        None,
        "the arena has no public instance"
    );
    let first = world
        .prepare_membership(1, zone, None, WorldPose::ORIGIN, true, 10)
        .unwrap();
    let first_instance = first.instance_id();
    assert_ne!(
        first_instance, 0,
        "the public instance must not host the match"
    );
    world.commit_membership(first);
    let second = world
        .prepare_membership(2, zone, None, WorldPose::ORIGIN, true, 20)
        .unwrap();
    assert_ne!(
        second.instance_id(),
        first_instance,
        "each entry gets its own arena instance"
    );
}

/// L012 (lead, from the T021 review): a save taken mid-match must reload and
/// put the player back in the pre-match world. Before L012 the arena scene id
/// stopped the load at `saved zone differs from gameplay encounter`.
#[test]
fn a_mid_match_save_reloads_back_into_the_world() {
    let fixture = Fixture::new();
    let mut gateway = enter_world(&fixture, 0, "soccer-reconnect");
    let start = gateway.state().clone();
    let (instance, loaded_at) = enter_match(&mut gateway, 1_000);
    gateway_send(&mut gateway, &lifecycle(5, instance), loaded_at + 100);

    let path = fixture.0.join("mid-match-save.json");
    gateway.save_with_pet_roster(&path).unwrap();
    let saved = game_store::load_or_create(&path, fixture.character()).unwrap();
    let reloaded = Gateway::load_gameplay(
        &reference_dir(),
        saved.character,
        &fixture.0,
        saved.gameplay,
    )
    .expect("L012: a save taken mid-match must load");
    assert_eq!(
        reloaded.state().zone_id,
        start.zone_id,
        "L012: the pre-match zone is recovered before location validation"
    );
    assert_eq!(
        reloaded.state().position,
        start.position,
        "L012: the pre-match pose is recovered"
    );
}

/// L012: a reconnect inside the same process drops the match (no result, no
/// reward) and returns the player to the pre-match pose, as kart does.
#[test]
fn a_reconnect_mid_match_drops_the_match_and_restores_the_pre_match_pose() {
    let fixture = Fixture::new();
    let mut gateway = enter_world(&fixture, 0, "soccer-reset");
    let start = gateway.state().clone();
    let (instance, loaded_at) = enter_match(&mut gateway, 1_000);
    gateway_send(&mut gateway, &lifecycle(5, instance), loaded_at + 100);
    assert!(gateway.gameplay().unwrap().is_soccer_active());
    let coins = gateway.snapshot().gameplay.unwrap().inventory.coins;

    gateway.reset_session();

    assert!(
        !gateway.gameplay().unwrap().is_soccer_active(),
        "the match ends with the connection"
    );
    assert_eq!(gateway.state().zone_id, start.zone_id);
    assert_eq!(gateway.state().position, start.position);
    assert_eq!(gateway.state().rotation, start.rotation);
    assert_eq!(
        gateway.snapshot().gameplay.unwrap().inventory.coins,
        coins,
        "no reward"
    );
}
