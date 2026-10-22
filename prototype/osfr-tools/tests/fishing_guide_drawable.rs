//! T385: every shipped fishing spot has a drawable guide whose ordinary menu starts its trip.
//!
//! `fishing_spots.json` lists six trips. Reed Stillwater and Lila Darklure are
//! `legacy_data` NPC rows; T385 adds `npc_spawns_extra.json` actors for the other four
//! guides (Selvi Slowreel, Gunther, Big J, Chip Numbwing), keyed by their NPC id. The tests
//! load the production `NpcVisibility` and `FishingSpots` from shipped content and drive the
//! real `Gameplay::handle_in_world` 26/8 -> 26/9 -> 26/10 route into the fishing service.
//!
//! Client evidence: 35/2 AddNpc reader `0x00921a10` (u64 actor, i32 name id, i32 model id;
//! the name is a `Global.Text.%d` id, format string at `0x01804c14`); 26/8 constructor
//! `0x008b6bc0` (u64 target + bool); 26/9 chain `0x00aa2360 -> 0x00aa2220 -> 0x00aa1600 ->
//! 0x00aa04d0` (option reader `0x00a9dca0`, seven dwords); 26/10 constructor `0x00a36f80`
//! (u64 target + i32 choice); the fishing lifecycle and opcode 138 shapes are in
//! `support/fishing_packets.rs`. No new wire shape.

#[path = "support/fishing_packets.rs"]
mod fishing_packets;
#[path = "common/gateway_packets.rs"]
mod gateway_packets;
#[path = "support/models.rs"]
mod models;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use fishing_packets::{add_fish_definitions, cast_at, fish_item_ids, lifecycle, reel, register};
use gateway_packets::{chat, interact, select};
use npc_quest_chain::{
    CHARACTER, Fixture, character_at_for, content_dir, menu_option, profile_xp, read_json, saved,
    send,
};
use osfr_tools::{
    character_store::CharacterState,
    fishing::Handle,
    fishing_service::{FISHERMAN_PROFILE, FishingSpots},
    gameplay::{Gameplay, Outcome},
    startup_world::parse_add_npc,
    world::NpcVisibility,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashMap},
    fs, io,
    path::Path,
    sync::OnceLock,
};

const REED: u64 = 9_000_000_181;
const LILA: u64 = 9_000_000_377;
const SELVI: u64 = 9_000_000_194;
const GUNTHER: u64 = 9_000_000_339;
const BIG_J: u64 = 9_000_000_024;
const CHIP: u64 = 9_500_000_736;
/// The guides T385 makes drawable; Reed and Lila were already `legacy_data` actors.
const NEW_GUIDES: [u64; 4] = [SELVI, GUNTHER, BIG_J, CHIP];
const ACTOR: Handle = Handle {
    id: 0x1122_3344,
    seq: 0x5566_7788,
};
/// The fishing service's timer base; bite 1,500 ms, nibble 900 ms.
const FISH_T0: u64 = 1_000_000;

// ---- shipped content ------------------------------------------------------------------------

struct Spot {
    npc: u64,
    area: String,
    game_id: i32,
    name_id: u64,
}

fn spots() -> Vec<Spot> {
    read_json(&content_dir().join("fishing_spots.json"))["spots"]
        .as_array()
        .unwrap()
        .iter()
        .map(|spot| Spot {
            npc: spot["npc"].as_u64().unwrap(),
            area: spot["area"].as_str().unwrap().to_string(),
            game_id: spot["game_id"].as_i64().unwrap() as i32,
            name_id: spot["name_id"].as_u64().unwrap(),
        })
        .collect()
}

fn menu_label() -> u32 {
    FishingSpots::load(&content_dir())
        .unwrap()
        .expect("fishing_spots.json ships")
        .menu_string_id()
}

/// Every regional NPC row, keyed by NPC id.
fn npc_rows() -> &'static HashMap<u64, Value> {
    static ROWS: OnceLock<HashMap<u64, Value>> = OnceLock::new();
    ROWS.get_or_init(|| {
        let mut rows = HashMap::new();
        for entry in fs::read_dir(content_dir().join("npcs")).unwrap() {
            for row in read_json(&entry.unwrap().path())["records"]
                .as_array()
                .unwrap()
            {
                rows.insert(row["npc_id"].as_u64().unwrap(), row.clone());
            }
        }
        rows
    })
}

fn xyz(row: &Value) -> [f32; 3] {
    serde_json::from_value(row["position"].clone()).unwrap()
}

fn guide_position(npc: u64) -> [f32; 3] {
    xyz(&npc_rows()[&npc])
}

fn same_place(left: [f32; 3], right: [f32; 3]) -> bool {
    (0..3).all(|axis| (left[axis] - right[axis]).abs() < 1e-3)
}

fn locale() -> &'static HashMap<u64, String> {
    static LOCALE: OnceLock<HashMap<u64, String>> = OnceLock::new();
    LOCALE.get_or_init(|| {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../lab/reports/locale-id-to-text.json");
        read_json(&path)
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|row| Some((row["id"].as_u64()?, row["text"].as_str()?.to_string())))
            .collect()
    })
}

fn sources(row: &Value, kind: &str) -> String {
    row["source"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|source| source["kind"] == kind)
        .map(|source| source["evidence"].as_str().unwrap())
        .collect::<Vec<_>>()
        .join(" ")
}

// ---- one player on the real gameplay route ----------------------------------------------

/// The reviewed opening fixture plus the shipped spot file and fish definitions; NPC
/// visibility and interaction come from the bundled `data/content`, as in production.
struct Player {
    fixture: Fixture,
    game: Gameplay,
    character: CharacterState,
    now: u64,
}

impl Player {
    fn new(guid: u64) -> Self {
        let fixture = Fixture::for_character(guid);
        fs::copy(
            content_dir().join("fishing_spots.json"),
            fixture.path().join("fishing_spots.json"),
        )
        .unwrap();
        add_fish_definitions(fixture.path());
        let game = fixture.game(None);
        Self {
            fixture,
            game,
            character: character_at_for(guid, 0, guide_position(REED)),
            now: 10_000,
        }
    }

    fn at(&mut self, npc: u64) {
        self.character.position = guide_position(npc);
    }

    fn try_send(&mut self, packet: &[u8]) -> io::Result<Outcome> {
        self.now += 1_000;
        self.game.handle_in_world(packet, &self.character, self.now)
    }

    fn send(&mut self, packet: &[u8]) -> Outcome {
        self.now += 1_000;
        send(&mut self.game, &self.character, packet, self.now)
    }

    /// The real menu: 26/8 lists the trip, 26/10 takes it. Returns the fish option, the
    /// 39/16 instance and the offered MiniGameData id.
    fn take_trip(&mut self, npc: u64) -> (i32, i32, i32) {
        let menu = self.send(&interact(npc));
        assert!(
            find(&menu.replies, &[26, 0, 9, 0]).is_some(),
            "guide {npc} has no menu"
        );
        let option = menu_option(&menu, menu_label());
        let replies = self.send(&select(npc, option as u32)).replies;
        let offer = find(&replies, &[39, 0, 16]).expect("the fish option offers the round");
        let instance = i32::from_le_bytes(offer[3..7].try_into().unwrap());
        let game_id = i32::from_le_bytes(offer[11..15].try_into().unwrap());
        (option, instance, game_id)
    }

    /// Take the trip and press Go (39/5): the encounter starts and the fishing flag is on.
    fn start(&mut self, npc: u64) -> i32 {
        let (_, instance, _) = self.take_trip(npc);
        let go = self.send(&lifecycle(5, instance));
        assert!(
            find(&go.replies, &[41, 0, 2]).is_some(),
            "39/5 starts the run"
        );
        assert_eq!(
            status_flags(&go.replies),
            vec![0x40_0001],
            "fishing flag on"
        );
        instance
    }

    /// The client's ordinary quit (39/6): the round ends and the flag goes off.
    fn quit(&mut self, instance: i32) {
        let left = self.send(&lifecycle(6, instance));
        assert_eq!(status_flags(&left.replies), vec![1], "fishing flag off");
    }

    fn cast_and_hook(&mut self) {
        let position = self.character.position;
        send(
            &mut self.game,
            &self.character,
            &cast_at(ACTOR, position, 10.0),
            FISH_T0 + 10,
        );
        for at in [FISH_T0 + 1_510, FISH_T0 + 2_410] {
            self.game.tick(&self.character, at).expect("world tick");
        }
    }

    /// Free in the world: the guide's menu opens instead of the busy refusal.
    fn is_free(&mut self, npc: u64) -> bool {
        find(&self.send(&interact(npc)).replies, &[26, 0, 9, 0]).is_some()
    }

    fn fish_held(&self) -> u32 {
        let ids = fish_item_ids();
        self.game
            .state()
            .inventory
            .items
            .iter()
            .filter(|item| ids.contains(&item.definition_id))
            .map(|item| item.count)
            .sum()
    }

    /// A refused request: an error or no menu/offer, and the serialized save is unchanged.
    fn refused(&mut self, packet: &[u8], label: &str) {
        let before = saved(&self.game);
        if let Ok(outcome) = self.try_send(packet) {
            assert!(!outcome.changed, "{label} changed gameplay");
            for prefix in [&[26, 0, 9, 0][..], &[39, 0, 16]] {
                assert!(find(&outcome.replies, prefix).is_none(), "{label}");
            }
        }
        assert_eq!(saved(&self.game), before, "{label} changed the save");
    }
}

fn find<'a>(packets: &'a [Vec<u8>], prefix: &[u8]) -> Option<&'a Vec<u8>> {
    packets.iter().find(|packet| packet.starts_with(prefix))
}

fn status_flags(replies: &[Vec<u8>]) -> Vec<u32> {
    replies
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 20, 0]))
        .map(|packet| u32::from_le_bytes(packet[12..16].try_into().unwrap()))
        .collect()
}

// ---- gates --------------------------------------------------------------------------------

#[test]
fn all_six_fishing_spots_have_a_drawable_guide() {
    let content = content_dir();
    let gates = FishingSpots::load(&content).unwrap().unwrap().gate_npcs();
    let visibility = NpcVisibility::load(&content).expect("production NPC visibility loads");
    let mut actors = BTreeMap::new();
    for target in visibility.teleport_targets(0) {
        assert!(
            actors.insert(target.guid, target.position).is_none(),
            "duplicate actor {}",
            target.guid
        );
    }
    let models = models::read_models();
    let spots = spots();
    assert_eq!(spots.len(), 6);
    for spot in &spots {
        let npc = spot.npc;
        assert!(gates.contains(&npc));
        let name = npc_rows()[&npc]["name"].as_str().unwrap();
        let position = guide_position(npc);
        assert_eq!(
            visibility.content_key(npc),
            Some(npc),
            "{} guide {name} has no runtime actor",
            spot.area
        );
        assert!(
            same_place(actors[&npc], position),
            "{name} is not at its row"
        );
        // The 35/2 the production view sends a player standing at the guide.
        let mut view = visibility.clone();
        let added: Vec<_> = view
            .update(0, position)
            .unwrap()
            .iter()
            .filter(|packet| packet.starts_with(&[35, 0, 2, 0]))
            .map(|packet| parse_add_npc(packet).unwrap())
            .filter(|actor| actor.actor_id == npc)
            .collect();
        assert_eq!(added.len(), 1, "one 35/2 actor for {name}");
        let actor = &added[0];
        assert!(same_place(
            [actor.position[0], actor.position[1], actor.position[2]],
            position
        ));
        assert!(
            actor.model_id > 0 && models.contains_key(&i64::from(actor.model_id)),
            "{name} has no client model"
        );
        assert_eq!(
            locale()
                .get(&(actor.name_string_id as u64))
                .map(String::as_str),
            Some(name),
            "{name}'s nameplate id"
        );
        assert!(
            locale()
                .get(&spot.name_id)
                .is_some_and(|text| !text.is_empty()),
            "{} has no trip label",
            spot.area
        );
    }
}

#[test]
fn each_guide_opens_its_spot_and_starts_the_matching_trip() {
    let mut player = Player::new(CHARACTER);
    for spot in spots() {
        let npc = spot.npc;
        player.at(npc);
        let (option, instance, game_id) = player.take_trip(npc);
        assert_eq!(game_id, spot.game_id, "{} offers its own game", spot.area);

        // An open trip is the busy activity: no other menu or second trip starts.
        player.refused(&interact(npc), "a menu while busy");
        player.refused(&select(npc, option as u32), "a second trip while busy");

        let go = player.send(&lifecycle(5, instance));
        assert!(find(&go.replies, &[41, 0, 2]).is_some());
        player.quit(instance);
        assert!(player.is_free(npc), "{}: quit frees the player", spot.area);

        // Out of range: the click is refused and nothing starts.
        player.character.position[0] += 100.0;
        player.refused(&interact(npc), "a far click");
        player.refused(&select(npc, option as u32), "a far choice");
        player.at(npc);
    }
}

#[test]
fn the_four_new_spawns_use_recovered_or_documented_models() {
    let extras: Vec<Value> = read_json(&content_dir().join("npc_spawns_extra.json"))["records"]
        .as_array()
        .unwrap()
        .clone();
    let models = models::read_models();
    for npc in [REED, LILA] {
        assert_eq!(npc_rows()[&npc]["placement"], "legacy_data");
        assert!(
            !extras
                .iter()
                .any(|row| row["spawn_guid"] == npc || row["content_key"] == npc),
            "{npc} is already drawn from its legacy_data row"
        );
    }
    for npc in NEW_GUIDES {
        let matches: Vec<_> = extras
            .iter()
            .filter(|row| row["content_key"] == npc)
            .collect();
        assert_eq!(matches.len(), 1, "one spawn for guide {npc}");
        let row = matches[0];
        let name = npc_rows()[&npc]["name"].as_str().unwrap();
        assert_eq!(row["spawn_guid"], npc);
        assert_eq!(row["name"], name);
        assert_eq!(xyz(row), guide_position(npc), "{name} position is verbatim");
        assert_eq!(
            locale()
                .get(&row["name_string_id"].as_u64().unwrap())
                .map(String::as_str),
            Some(name)
        );
        for field in ["legacy_npc_id", "capture_match", "position_w"] {
            assert!(row[field].is_null(), "{name} {field}");
        }
        let model_id = row["model_id"].as_i64().unwrap();
        let file = row["model_file_name"].as_str().unwrap();
        assert!(
            models
                .get(&model_id)
                .is_some_and(|files| files.contains(file)),
            "Models.txt does not map {model_id} to {file}"
        );
        assert!(!sources(row, "legacy-data").is_empty() && !sources(row, "client").is_empty());
        let reconstructed = sources(row, "reconstructed");
        assert!(
            reconstructed.starts_with("(R)") && reconstructed.contains("not a captured transform"),
            "{name} must record its reconstructed placement"
        );
        match npc {
            SELVI => assert_eq!((model_id, file), (1593, "fairy_m_selvi_slowreel.agr")),
            CHIP => assert_eq!((model_id, file), (1585, "fairy_m_chip_numbwing.agr")),
            // No client asset names these two: the body is a documented (R) choice.
            _ => assert!(
                reconstructed.contains(&format!("(R) Model {model_id} is a generic")),
                "{name}'s substitute body must be marked (R)"
            ),
        }
    }
}

#[test]
fn a_far_or_wrong_guide_changes_nothing() {
    let mut player = Player::new(CHARACTER);
    player.at(CHIP);
    player.character.position[2] += 100.0;
    player.refused(&interact(CHIP), "a far click");
    player.at(CHIP);
    let menu = player.send(&interact(CHIP));
    let option = menu_option(&menu, menu_label());
    player.refused(
        &select(SELVI, option as u32),
        "a choice naming another guide",
    );
    player.refused(&interact(u64::MAX), "an unknown actor");
    player.refused(&[26, 0, 8, 0, 1, 2, 3], "a malformed 26/8");
    player.refused(&select(CHIP, (option + 99) as u32), "an unoffered choice");
    // Nothing above consumed the menu or started a trip: the real choice still offers it.
    let offer = player.send(&select(CHIP, option as u32));
    assert!(find(&offer.replies, &[39, 0, 16]).is_some());
}

#[test]
fn two_players_see_the_same_guides() {
    let mut first = Player::new(CHARACTER);
    let mut second = Player::new(CHARACTER + 1);
    let mut seen = [BTreeMap::new(), BTreeMap::new()];
    for npc in NEW_GUIDES {
        for (player, seen) in [&mut first, &mut second].into_iter().zip(&mut seen) {
            player.at(npc);
            for packet in player
                .game
                .update_npc_visibility(&player.character)
                .unwrap()
            {
                if packet.starts_with(&[35, 0, 2, 0]) {
                    seen.insert(parse_add_npc(&packet).unwrap().actor_id, packet);
                }
            }
        }
        assert!(seen[0].contains_key(&npc), "guide {npc} was never sent");
        assert_eq!(
            seen[0].get(&npc),
            seen[1].get(&npc),
            "different 35/2 for {npc}"
        );
    }

    // Independent trips at the same guide: one player's quit leaves the other fishing.
    first.at(SELVI);
    second.at(SELVI);
    let first_round = first.start(SELVI);
    assert!(
        second.is_free(SELVI),
        "one player's trip does not busy the other"
    );
    let second_round = second.start(SELVI);
    first.quit(first_round);
    assert!(first.is_free(SELVI));
    second.refused(&interact(SELVI), "the second player is still fishing");
    second.quit(second_round);
    assert!(second.is_free(SELVI));
}

#[test]
fn a_trip_can_be_quit_and_a_reconnect_releases_the_round() {
    let mut player = Player::new(CHARACTER);
    player.at(GUNTHER);
    let round = player.start(GUNTHER);
    let registered = player.send(&register(ACTOR));
    assert!(find(&registered.replies, &[138, 0, 3, 0]).is_some());
    player.cast_and_hook();
    let caught = send(
        &mut player.game,
        &player.character,
        &reel(ACTOR),
        FISH_T0 + 2_420,
    );
    assert!(find(&caught.replies, &[138, 0, 14, 0]).is_some());
    assert_eq!(player.fish_held(), 1, "one catch, paid by the server");
    let xp = profile_xp(&player.game, FISHERMAN_PROFILE);
    player.quit(round);
    assert!(
        player.is_free(GUNTHER),
        "the ordinary quit frees the player"
    );

    // Start again and drop the connection mid-attempt; reconnect through the saved state.
    player.start(GUNTHER);
    player.send(&register(ACTOR));
    player.cast_and_hook();
    player.game.reset_session();
    player.game = player.fixture.reload_serialized(&player.game);
    let late = player.try_send(&reel(ACTOR));
    assert!(!late.is_ok_and(|out| find(&out.replies, &[138, 0, 14, 0]).is_some()));
    assert_eq!(player.fish_held(), 1, "the dropped cast pays nothing");
    assert_eq!(profile_xp(&player.game, FISHERMAN_PROFILE), xp);
    assert!(player.is_free(GUNTHER), "no stuck trip after the reconnect");
    let (_, _, game_id) = player.take_trip(GUNTHER);
    assert_eq!(game_id, 597, "a fresh Brambleback's Bayou trip");
}

#[test]
fn fishing_entry_uses_no_dev_commands() {
    let mut player = Player::new(CHARACTER);
    let denied = player.send(&chat("realm goto Selvi Slowreel"));
    assert!(denied.dev_command_denied && denied.relocation.is_none());
    assert!(same_place(player.character.position, guide_position(REED)));
    for npc in NEW_GUIDES {
        player.at(npc);
        let menu = player.send(&interact(npc));
        let option = menu_option(&menu, menu_label());
        let choice = player.send(&select(npc, option as u32));
        for out in [&menu, &choice] {
            assert!(!out.dev_command_denied && out.relocation.is_none() && out.zoning.is_none());
        }
        let offer = find(&choice.replies, &[39, 0, 16]).expect("the trip is offered");
        let instance = i32::from_le_bytes(offer[3..7].try_into().unwrap());
        player.send(&lifecycle(5, instance));
        player.quit(instance);
    }
}
