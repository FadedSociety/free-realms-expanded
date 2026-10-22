//! T450: exploration coins through the real 26/8 route (`Gameplay::handle_in_world`).
//!
//! The gameplay content is the private opening package the gathering suite uses, copied with the
//! shipped `mining.json` so a player can be busy; NPCs, coins and the collection catalogue are the
//! repository's `data/content`. Coin 1946 ticks Seaside Rare Exploration (267) entry 1937.
#[path = "common/content_copy.rs"]
mod content_copy;

use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    chat,
    gameplay::{Gameplay, GameplayState, Outcome},
    harvest_nodes::GatheringNodes,
    mining::MINER_PROFILE_ID,
    npc_interaction::NpcInteractRequest,
};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const COIN: u64 = 100_000_001_946;
const COIN_AT: [f32; 3] = [-507.1435, -27.20111, -1201.367];
const ENTRY: i32 = 1937;
const SEASIDE_RARE: i32 = 267;
/// Boddington, a placed Blackspore vendor (`npcs/`, `vendors.json`).
const VENDOR: u64 = 9_500_000_007;
const VENDOR_AT: [f32; 3] = [-2496.12, -23.96224, 1084.769];
static NEXT: AtomicU64 = AtomicU64::new(0);

fn bundled() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

struct Fixture {
    dir: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "osfr-coin-pickup-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let opening = std::env::var_os("OSFR_TYPE8_CONTENT")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../../lab/references/opening-journal-text-v2/content")
            });
        content_copy::copy_tree(&opening, &dir);
        fs::copy(bundled().join("mining.json"), dir.join("mining.json")).unwrap();
        Self { dir }
    }

    fn load(&self, state: Option<GameplayState>) -> Gameplay {
        Gameplay::load(&self.dir, state).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

/// Every placed coin of a collection, as (guid, position) from the shipped coin map.
fn coins_of(collection: i32) -> Vec<(u64, [f32; 3])> {
    let spawns = json(&bundled().join("npc_spawns_extra.json"));
    let position = |guid: u64| {
        let row = spawns["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["spawn_guid"] == guid)
            .expect("coin spawn row");
        let p = &row["position"];
        [0, 1, 2].map(|i| p[i].as_f64().unwrap() as f32)
    };
    json(&bundled().join("collection_coins.json"))["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|coin| coin["collection_id"] == collection)
        .map(|coin| coin["spawn_guid"].as_u64().unwrap())
        .map(|guid| (guid, position(guid)))
        .collect()
}

fn at(position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position,
        rotation: [1.0, 0.0, 0.0],
    }
}

fn click(game: &mut Gameplay, target: u64, who: &CharacterState) -> Outcome {
    let request = NpcInteractRequest {
        target,
        trailing_flag: false,
    };
    game.handle_in_world(&request.encode(), who, 1_000).unwrap()
}

/// 35/2 AddNpc: the actor guid follows the header (reader `0x00921a10`).
fn added(packets: &[Vec<u8>], guid: u64) -> bool {
    packets
        .iter()
        .any(|p| p.starts_with(&[35, 0, 2, 0]) && p.get(4..12) == Some(&guid.to_le_bytes()[..]))
}

/// 35/3: a u16 count, then the guid (reader `0x008d69f0`).
fn removed(packets: &[Vec<u8>], guid: u64) -> bool {
    packets
        .iter()
        .any(|p| p.starts_with(&[35, 0, 3, 0]) && p.get(6..14) == Some(&guid.to_le_bytes()[..]))
}

fn ledger_holds(game: &Gameplay, entry: i32) -> bool {
    game.state()
        .progression_extras
        .is_some_and(|extras| extras.collections.is_collected(entry))
}

/// What a refusal must leave alone: the save and the visibility revision.
fn snapshot(game: &Gameplay) -> (GameplayState, u64) {
    (game.state(), game.npc_visibility_revision())
}

/// Standing on coin 1946, which the client has been sent.
fn at_coin(fixture: &Fixture) -> (Gameplay, CharacterState) {
    let mut game = fixture.load(None);
    let who = at(COIN_AT);
    assert!(added(&game.update_npc_visibility(&who).unwrap(), COIN));
    (game, who)
}

#[test]
fn click_in_range_collects_and_hides() {
    let fixture = Fixture::new();
    let (mut game, who) = at_coin(&fixture);
    let out = click(&mut game, COIN, &who);
    // 38/10 AddEntry in reader `008d8570` order: entry, icon, tint, name, collection, index,
    // unknown, collected (catalogue row 1937 of collection 267).
    let mut add_entry = vec![38, 0, 10, 0];
    for value in [1937u32, 2237, 0, 386_338, 267, 1, 0] {
        add_entry.extend(value.to_le_bytes());
    }
    add_entry.push(1);
    assert_eq!(out.replies, vec![add_entry]);
    assert!(out.changed);
    assert!(ledger_holds(&game, ENTRY));
    let next = game.update_npc_visibility(&who).unwrap();
    assert!(
        removed(&next, COIN),
        "the next visibility update drops the coin"
    );
}

#[test]
fn second_click_does_nothing() {
    let fixture = Fixture::new();
    let (mut game, who) = at_coin(&fixture);
    assert!(click(&mut game, COIN, &who).changed);
    let before = snapshot(&game);
    let replay = click(&mut game, COIN, &who);
    assert!(replay.replies.is_empty() && !replay.changed);
    assert_eq!(snapshot(&game), before);
}

#[test]
fn out_of_range_and_busy_refuse_without_change() {
    let fixture = Fixture::new();
    let mut state = fixture.load(None).state();
    let profiles = state.profiles.as_mut().unwrap();
    profiles.unlocked.insert(MINER_PROFILE_ID);
    profiles.active = MINER_PROFILE_ID;
    let mut game = fixture.load(Some(state));
    let far = at([COIN_AT[0] + 12.5, COIN_AT[1], COIN_AT[2]]);
    let elsewhere = CharacterState {
        zone_id: 5,
        ..at(COIN_AT)
    };
    for who in [far, elsewhere] {
        let before = snapshot(&game);
        let out = click(&mut game, COIN, &who);
        assert_eq!(out.replies, vec![chat::system("Move closer to use that.")]);
        assert!(!out.changed);
        assert_eq!(snapshot(&game), before);
    }
    // Busy: a mining run is under way.
    let nodes = GatheringNodes::load(&bundled()).unwrap().unwrap();
    let vein = nodes
        .actors()
        .find(|node| node.name == "Copper Vein")
        .unwrap()
        .clone();
    assert!(click(&mut game, vein.guid, &at(vein.position)).changed);
    let before = snapshot(&game);
    let out = click(&mut game, COIN, &at(COIN_AT));
    assert_eq!(
        out.replies,
        vec![chat::system("Finish what you are doing first.")]
    );
    assert!(!out.changed);
    assert_eq!(snapshot(&game), before);
    assert!(!ledger_holds(&game, ENTRY));
}

#[test]
fn relog_keeps_coin_hidden() {
    let fixture = Fixture::new();
    let (mut game, who) = at_coin(&fixture);
    let fresh = game.shared_npc_view(&who, &[], &BTreeSet::new()).unwrap();
    assert!(
        fresh.0.contains(&COIN),
        "the coin is in the shared view before"
    );
    assert!(click(&mut game, COIN, &who).changed);

    let mut relog = fixture.load(Some(game.state()));
    assert!(ledger_holds(&relog, ENTRY));
    let (visible, packets) = relog.shared_npc_view(&who, &[], &BTreeSet::new()).unwrap();
    assert!(!visible.contains(&COIN) && !added(&packets, COIN));
    assert!(!added(&relog.update_npc_visibility(&who).unwrap(), COIN));
    let again = click(&mut relog, COIN, &who);
    assert!(again.replies.is_empty() && !again.changed);
}

#[test]
fn content_reload_keeps_coin_hidden() {
    let fixture = Fixture::new();
    let (mut game, who) = at_coin(&fixture);
    assert!(click(&mut game, COIN, &who).changed);
    assert!(removed(&game.update_npc_visibility(&who).unwrap(), COIN));
    let world = fixture.dir.join("live");
    fs::create_dir_all(&world).unwrap();
    game.set_dev_tools(true);
    game.set_role(AccountRole::Admin);
    game.set_world_dir(Some(world)).unwrap();
    let reload = game
        .handle_in_world(&chat::local_request("realm reload all"), &who, 2_000)
        .unwrap();
    let text: String = reload
        .replies
        .iter()
        .map(|p| String::from_utf8_lossy(p))
        .collect();
    assert!(text.contains("Reloaded:"), "{text}");
    assert!(!added(&reload.replies, COIN));
    assert!(!added(&game.update_npc_visibility(&who).unwrap(), COIN));
    let (visible, _) = game.shared_npc_view(&who, &[], &BTreeSet::new()).unwrap();
    assert!(!visible.contains(&COIN));
}

#[test]
fn other_player_still_sees_it() {
    let fixture = Fixture::new();
    let (mut first, who) = at_coin(&fixture);
    assert!(click(&mut first, COIN, &who).changed);
    let (mut second, who) = at_coin(&fixture);
    assert!(!ledger_holds(&second, ENTRY));
    let out = click(&mut second, COIN, &who);
    assert!(out.changed && out.replies.len() == 1);
    assert!(ledger_holds(&second, ENTRY));
}

#[test]
fn non_coin_click_falls_through() {
    let fixture = Fixture::new();
    let mut game = fixture.load(None);
    let who = at(VENDOR_AT);
    assert!(added(&game.update_npc_visibility(&who).unwrap(), VENDOR));
    let out = click(&mut game, VENDOR, &who);
    // The NPC router's 26/9 interaction list (reader `00aa2360`), and no collection entry.
    assert!(out.replies.iter().any(|p| p.starts_with(&[26, 0, 9, 0])));
    assert!(out.replies.iter().all(|p| !p.starts_with(&[38, 0, 10, 0])));
    assert!(game.state().progression_extras.is_none());
}

#[test]
fn completing_a_set_records_completion_once() {
    let fixture = Fixture::new();
    let mut game = fixture.load(None);
    let coins = coins_of(SEASIDE_RARE);
    assert_eq!(coins.len(), 5, "every Seaside Rare entry has a coin");
    for (guid, position) in &coins {
        let who = at(*position);
        game.update_npc_visibility(&who).unwrap();
        let out = click(&mut game, *guid, &who);
        assert_eq!(out.replies.len(), 1, "only the 38/10, no reward packet");
        assert!(out.replies[0].starts_with(&[38, 0, 10, 0]));
    }
    let ledger = game.state().progression_extras.unwrap().collections;
    assert!(ledger.is_completed(SEASIDE_RARE));
    assert_eq!(ledger.completed_count(), 1);
    let (guid, position) = coins[4];
    let replay = click(&mut game, guid, &at(position));
    assert!(replay.replies.is_empty() && !replay.changed);
}
