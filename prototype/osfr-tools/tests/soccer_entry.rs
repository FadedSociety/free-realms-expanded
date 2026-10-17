//! T021 acceptance: soccer from the activity portal's Play request through the
//! shared minigame result and one payout, plus refusals that change nothing.
//!
//! Uses the same private opening content as `activity_portal_route.rs` and
//! writes a test-owned `soccer.json` with a short match so the gate is bounded.
//! No player names or chat text are embedded.

use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    soccer::{self, SoccerPacket},
};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const GUID: u64 = 0x0000_0010_0080_06b2;
const SOCCER_ACTIVITY: u32 = 43;
static NEXT: AtomicU64 = AtomicU64::new(0);

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
        "reward": {"coins": 10, "job_xp": {}}
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
