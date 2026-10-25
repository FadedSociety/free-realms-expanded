//! T525: live coin authoring through the real chat path (`Gameplay::handle_in_world`).
//!
//! The gameplay content is the private opening package the collection pickup suite uses,
//! copied with the shipped `mining.json`; NPCs, coins and the collection catalogue are the
//! repository's `data/content`. The live-world folder is the overlay's home, exactly as
//! `--world` uses it.
#[path = "common/content_copy.rs"]
mod content_copy;

use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    chat,
    gameplay::{Gameplay, GameplayState, Outcome},
    npc_interaction::NpcInteractRequest,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const LIVE_GUID_BASE: u64 = 131_000_000_000;
const LIVE_GUID_END: u64 = 132_000_000_000;
/// A content coin bound to Blackspore Elite (collection 288) entry 2039; it cannot be
/// reached on foot (Lxzxrus 65ec688), which is what `move` is for.
const COIN_2115: u64 = 100_000_002_115;
const COIN_2115_AT: [f32; 3] = [-2519.431, -1.902856, 1201.663];
/// An empty patch of the open world for new coins.
const HERE: [f32; 3] = [321.0, 5.0, -654.0];
static NEXT: AtomicU64 = AtomicU64::new(0);

fn bundled() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

struct Fixture {
    dir: PathBuf,
    live: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "osfr-coin-authoring-{}-{}",
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
        let live = dir.join("live");
        fs::create_dir_all(&live).unwrap();
        Self { dir, live }
    }

    /// A session with the live-world layer already applied, like the gateway's login.
    fn load(&self, state: Option<GameplayState>) -> Gameplay {
        let mut game = Gameplay::load(&self.dir, state).unwrap();
        game.set_world_dir(Some(self.live.clone())).unwrap();
        game
    }

    fn admin(&self) -> Gameplay {
        let mut game = self.load(None);
        game.set_role_commands(true);
        game.set_role(AccountRole::Admin);
        game
    }

    fn with_role(&self, role: AccountRole) -> Gameplay {
        let mut game = self.load(None);
        game.set_role_commands(true);
        game.set_role(role);
        game
    }

    fn edits_bytes(&self) -> Option<Vec<u8>> {
        fs::read(self.live.join("collection_coin_edits.json")).ok()
    }

    fn rows(&self) -> Vec<Value> {
        self.edits_bytes()
            .map(|bytes| serde_json::from_slice::<Value>(&bytes).unwrap()["records"].clone())
            .and_then(|records| records.as_array().cloned())
            .unwrap_or_default()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let overlay = self.live.join("collection_coin_edits.json");
        if overlay.is_file() {
            let mut permissions = fs::metadata(&overlay).unwrap().permissions();
            permissions.set_readonly(false);
            let _ = fs::set_permissions(&overlay, permissions);
        }
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn at(position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position,
        rotation: [1.0, 0.0, 0.0],
    }
}

fn realm(game: &mut Gameplay, line: &str, who: &CharacterState) -> Outcome {
    game.handle_in_world(&chat::local_request(line), who, 1_000)
        .unwrap()
}

fn said(out: &Outcome) -> String {
    let text: String = out
        .replies
        .iter()
        .map(|packet| String::from_utf8_lossy(packet))
        .collect();
    text
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

/// What a refusal must leave alone: the save, the visibility revision and both files.
#[derive(Debug, PartialEq)]
struct Snapshot {
    state: GameplayState,
    revision: u64,
    coins: Vec<u8>,
    edits: Option<Vec<u8>>,
    placements: Option<Vec<u8>>,
}

fn snapshot(game: &Gameplay, fixture: &Fixture) -> Snapshot {
    Snapshot {
        state: game.state(),
        revision: game.npc_visibility_revision(),
        coins: fs::read(bundled().join("collection_coins.json")).unwrap(),
        edits: fixture.edits_bytes(),
        placements: fs::read(fixture.live.join("npc_placements.json")).ok(),
    }
}

fn single_row(fixture: &Fixture) -> Value {
    let rows = fixture.rows();
    assert_eq!(rows.len(), 1, "{rows:?}");
    rows.into_iter().next().unwrap()
}

#[test]
fn place_binds_1950_and_a_reload_can_collect_it() {
    let fixture = Fixture::new();
    let mut game = fixture.admin();
    let who = at(HERE);
    game.update_npc_visibility(&who).unwrap();
    let out = realm(&mut game, "realm coin place 1950", &who);
    assert!(said(&out).contains("Placed coin"), "{}", said(&out));

    let row = single_row(&fixture);
    let guid = row["spawn_guid"].as_u64().unwrap();
    assert!(
        (LIVE_GUID_BASE..LIVE_GUID_END).contains(&guid),
        "{guid} is outside the live band"
    );
    assert_eq!(row["entry_id"], json!(1950));
    assert_eq!(row["collection_id"], json!(265));
    assert_eq!(row["tier"], "rare");
    assert_eq!(
        row["model_id"],
        json!(818),
        "the model collection 265 coins use"
    );
    assert_eq!(row["name_string_id"], json!(386_170));
    assert_eq!(row["pairing"], "live");
    assert_eq!(row["source"]["kind"], "reconstructed");
    assert_eq!(row["source"]["recorded_ms"], json!(1_000));
    assert_eq!(row["source"]["character_guid"], json!(1));
    assert!(row["replaces"].is_null(), "a fresh entry replaces nothing");
    assert_eq!(row["position"][0].as_f64().unwrap(), HERE[0] as f64);
    assert_eq!(row["position"][1].as_f64().unwrap(), HERE[1] as f64);
    assert_eq!(row["position"][2].as_f64().unwrap(), HERE[2] as f64);
    assert!(added(&out.replies, guid), "the acting session sees it");

    // A new session merges the overlay from the live folder and can collect it.
    let mut reloaded = fixture.load(Some(game.state()));
    let who = at(HERE);
    assert!(added(&reloaded.update_npc_visibility(&who).unwrap(), guid));
    let out = click(&mut reloaded, guid, &who);
    assert!(out.changed);
    assert!(
        out.replies[0].starts_with(&[38, 0, 10, 0]),
        "38/10 AddEntry"
    );
    assert!(ledger_holds(&reloaded, 1950));
}

#[test]
fn move_2115_repoints_2039_and_hides_the_old_guid() {
    let fixture = Fixture::new();
    let mut game = fixture.admin();
    let who = at(HERE);
    game.update_npc_visibility(&who).unwrap();
    let out = realm(&mut game, "realm coin move 100000002115", &who);
    assert!(
        said(&out).contains("Moved coin 100000002115"),
        "{}",
        said(&out)
    );

    let row = single_row(&fixture);
    let new = row["spawn_guid"].as_u64().unwrap();
    assert_eq!(row["entry_id"], json!(2039));
    assert_eq!(row["collection_id"], json!(288));
    assert_eq!(row["tier"], "elite");
    assert_eq!(row["model_id"], json!(947));
    assert_eq!(row["name_string_id"], json!(4_412));
    assert_eq!(row["replaces"], json!(COIN_2115));
    assert!(added(&out.replies, new));

    // The old content coin is hidden for everyone; only the new coin ticks entry 2039.
    let old_spot = at(COIN_2115_AT);
    assert!(!added(
        &game.update_npc_visibility(&old_spot).unwrap(),
        COIN_2115
    ));
    let old_click = click(&mut game, COIN_2115, &old_spot);
    assert!(!old_click.changed);
    assert!(
        old_click
            .replies
            .iter()
            .all(|p| !p.starts_with(&[38, 0, 10, 0])),
        "a hidden content coin records nothing"
    );

    let mut reloaded = fixture.load(Some(game.state()));
    let old_spot = at(COIN_2115_AT);
    assert!(!added(
        &reloaded.update_npc_visibility(&old_spot).unwrap(),
        COIN_2115
    ));
    let who = at(HERE);
    assert!(added(&reloaded.update_npc_visibility(&who).unwrap(), new));
    let out = click(&mut reloaded, new, &who);
    assert!(out.changed);
    assert!(ledger_holds(&reloaded, 2039));
}

#[test]
fn list_names_the_nearest_live_coin() {
    let fixture = Fixture::new();
    let mut game = fixture.admin();
    let who = at(HERE);
    game.update_npc_visibility(&who).unwrap();
    let placed = realm(&mut game, "realm coin place 1950", &who);
    assert!(said(&placed).contains("Placed coin"));
    let guid = single_row(&fixture)["spawn_guid"].as_u64().unwrap();

    let out = realm(&mut game, "realm coin list 5", &who);
    let text = said(&out);
    assert!(text.contains(&guid.to_string()), "{text}");
    assert!(text.contains("entry 1950 rare live"), "{text}");

    let out = realm(&mut game, "realm coin list 0", &who);
    assert!(
        said(&out).contains("Radius must be 1-200"),
        "{}",
        said(&out)
    );
}

#[test]
fn unbind_removes_the_overlay_row_and_its_actor() {
    let fixture = Fixture::new();
    let mut game = fixture.admin();
    let who = at(HERE);
    game.update_npc_visibility(&who).unwrap();
    let placed = realm(&mut game, "realm coin place 1950", &who);
    let guid = single_row(&fixture)["spawn_guid"].as_u64().unwrap();
    assert!(added(&placed.replies, guid));

    let out = realm(&mut game, &format!("realm coin unbind {guid}"), &who);
    assert!(said(&out).contains("Removed placed coin"), "{}", said(&out));
    assert!(removed(&out.replies, guid), "the view drops the actor");
    assert!(fixture.rows().is_empty());
    assert!(
        fixture.edits_bytes().is_some(),
        "the empty overlay is a file"
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&fixture.edits_bytes().unwrap()).unwrap()["schema_version"],
        json!(1)
    );

    let mut reloaded = fixture.load(Some(game.state()));
    assert!(!added(&reloaded.update_npc_visibility(&who).unwrap(), guid));
    let replay = click(&mut reloaded, guid, &who);
    assert!(!replay.changed && replay.replies.is_empty());
}

#[test]
fn moved_coin_is_hidden_from_a_player_who_collected_the_entry() {
    let fixture = Fixture::new();
    let mut game = fixture.admin();
    let old_spot = at(COIN_2115_AT);
    assert!(added(
        &game.update_npc_visibility(&old_spot).unwrap(),
        COIN_2115
    ));
    assert!(click(&mut game, COIN_2115, &old_spot).changed);
    assert!(ledger_holds(&game, 2039));

    let who = at(HERE);
    let out = realm(&mut game, "realm coin move 100000002115", &who);
    assert!(said(&out).contains("Moved coin"), "{}", said(&out));
    let new = single_row(&fixture)["spawn_guid"].as_u64().unwrap();

    let mut reloaded = fixture.load(Some(game.state()));
    let who = at(HERE);
    let packets = reloaded.update_npc_visibility(&who).unwrap();
    assert!(
        !added(&packets, new),
        "the entry is collected, so its new actor is excluded"
    );
    let out = click(&mut reloaded, new, &who);
    assert!(!out.changed && out.replies.is_empty());
}

#[test]
fn refusals_change_nothing() {
    let fixture = Fixture::new();
    let mut game = fixture.admin();
    let who = at(HERE);
    game.update_npc_visibility(&who).unwrap();
    let before = snapshot(&game, &fixture);

    for (line, expected) in [
        ("realm coin place 999999", "No collection entry"),
        ("realm coin place 41", "not an exploration entry"),
        ("realm coin place 1935", "already has coin"),
    ] {
        let out = realm(&mut game, line, &who);
        assert!(said(&out).contains(expected), "{line}: {}", said(&out));
        assert!(!out.changed, "{line}");
        assert_eq!(snapshot(&game, &fixture), before, "{line}");
    }

    let elsewhere = CharacterState {
        zone_id: 5,
        ..at(HERE)
    };
    let out = realm(&mut game, "realm coin place 1950", &elsewhere);
    assert!(said(&out).contains("zone 0"), "{}", said(&out));
    assert_eq!(snapshot(&game, &fixture), before, "outside zone 0");

    for role in [AccountRole::Builder, AccountRole::Player] {
        let mut game = fixture.with_role(role);
        let who = at(HERE);
        game.update_npc_visibility(&who).unwrap();
        let before = snapshot(&game, &fixture);
        let out = realm(&mut game, "realm coin place 1950", &who);
        assert!(out.dev_command_denied, "{role:?}");
        assert!(
            said(&out).contains("role does not allow"),
            "{role:?}: {}",
            said(&out)
        );
        assert_eq!(snapshot(&game, &fixture), before, "{role:?}");
    }
    assert!(fixture.edits_bytes().is_none(), "no refusal wrote a file");
}

#[test]
fn a_second_session_sees_the_coin_after_its_poll() {
    let fixture = Fixture::new();
    let mut admin = fixture.admin();
    let who = at(HERE);
    admin.update_npc_visibility(&who).unwrap();
    let mut friend = fixture.load(None);
    let friend_who = at(HERE);
    friend.update_npc_visibility(&friend_who).unwrap();

    let out = realm(&mut admin, "realm coin place 1950", &who);
    assert!(said(&out).contains("Placed coin"), "{}", said(&out));
    let guid = single_row(&fixture)["spawn_guid"].as_u64().unwrap();

    let ticked = friend.tick(&friend_who, 1_000).unwrap();
    assert!(
        added(&ticked.replies, guid),
        "the one-second poll applies the overlay and sends 35/2"
    );
}

#[test]
fn live_guids_are_unique_against_content_and_placements() {
    let fixture = Fixture::new();
    let mut game = fixture.admin();
    let who = at(HERE);
    game.update_npc_visibility(&who).unwrap();
    let placed_npc = realm(&mut game, "realm place 168", &who);
    assert!(
        said(&placed_npc).contains("Placed NPC"),
        "{}",
        said(&placed_npc)
    );
    let placement: Value =
        serde_json::from_slice(&fs::read(fixture.live.join("npc_placements.json")).unwrap())
            .unwrap();
    let npc_guid = placement["records"][0]["guid"].as_u64().unwrap();

    let placed = realm(&mut game, "realm coin place 1950", &who);
    assert!(said(&placed).contains("Placed coin"), "{}", said(&placed));
    let first = single_row(&fixture)["spawn_guid"].as_u64().unwrap();
    let moved = realm(&mut game, "realm coin move 100000002115", &who);
    assert!(said(&moved).contains("Moved coin"), "{}", said(&moved));
    let rows = fixture.rows();
    assert_eq!(rows.len(), 2);
    let second = rows[1]["spawn_guid"].as_u64().unwrap();
    assert_ne!(first, second, "a move allocates a new GUID");
    assert!(
        (LIVE_GUID_BASE..LIVE_GUID_END).contains(&second),
        "{second}"
    );

    let content = content_guids();
    for guid in [first, second] {
        assert_ne!(guid, npc_guid, "a coin must not reuse a placement guid");
        assert!(!content.contains(&guid), "{guid} is a content guid");
    }
}

/// Every `guid`/`spawn_guid` in the actor content files, recursively.
fn content_guids() -> std::collections::BTreeSet<u64> {
    fn walk(value: &Value, out: &mut std::collections::BTreeSet<u64>) {
        match value {
            Value::Object(map) => {
                for (key, child) in map {
                    if matches!(key.as_str(), "guid" | "spawn_guid") {
                        if let Some(guid) = child.as_u64() {
                            out.insert(guid);
                        }
                    }
                    walk(child, out);
                }
            }
            Value::Array(items) => {
                for item in items {
                    walk(item, out);
                }
            }
            _ => {}
        }
    }
    let mut guids = std::collections::BTreeSet::new();
    for name in [
        "npc_spawns_extra.json",
        "field_enemies.json",
        "field_battles.json",
    ] {
        let document: Value =
            serde_json::from_slice(&fs::read(bundled().join(name)).unwrap()).unwrap();
        walk(&document, &mut guids);
    }
    guids
}

/// A hostile or careless overlay cannot grow past the loader cap: that would
/// make every later login fail to read the world.
#[test]
fn an_oversized_overlay_is_refused_at_load() {
    let fixture = Fixture::new();
    let mut game = fixture.load(None);
    let row = json!({
        "spawn_guid": LIVE_GUID_BASE,
        "model_id": 818,
        "name_string_id": 386_170,
        "position": [0.0, 0.0, 0.0],
        "heading": 0.0,
        "collection_id": 265,
        "entry_id": 1950,
        "tier": "rare",
        "pairing": "live",
        "source": {"kind": "reconstructed", "recorded_ms": 0, "character_guid": 1}
    });
    let records: Vec<Value> = std::iter::repeat_n(row, 513).collect();
    let bytes = serde_json::to_vec(&json!({"schema_version": 1, "records": records})).unwrap();
    fs::write(fixture.live.join("collection_coin_edits.json"), bytes).unwrap();
    let error = game.set_world_dir(Some(fixture.live.clone())).unwrap_err();
    assert!(error.to_string().contains("at most 512"), "{error}");
}

/// A hand-edited overlay row is validated before it reaches the client; a model
/// id that does not fit the AddNpc i32 field is refused.
#[test]
fn a_hostile_overlay_row_is_refused() {
    let fixture = Fixture::new();
    let mut game = fixture.load(None);
    let row = json!({
        "spawn_guid": LIVE_GUID_BASE,
        "model_id": u32::MAX,
        "name_string_id": 386_170,
        "position": [0.0, 0.0, 0.0],
        "heading": 0.0,
        "collection_id": 265,
        "entry_id": 1950,
        "tier": "rare",
        "pairing": "live",
        "source": {"kind": "reconstructed", "recorded_ms": 0, "character_guid": 1}
    });
    let bytes = serde_json::to_vec(&json!({"schema_version": 1, "records": [row]})).unwrap();
    fs::write(fixture.live.join("collection_coin_edits.json"), bytes).unwrap();
    let error = game.set_world_dir(Some(fixture.live.clone())).unwrap_err();
    assert!(error.to_string().contains("model_id"), "{error}");
}

#[cfg(windows)]
#[test]
fn a_failed_write_leaves_the_file_and_the_view() {
    let fixture = Fixture::new();
    let mut game = fixture.admin();
    let who = at(HERE);
    game.update_npc_visibility(&who).unwrap();
    let first = realm(&mut game, "realm coin place 1950", &who);
    assert!(said(&first).contains("Placed coin"));
    let before = snapshot(&game, &fixture);
    let path = fixture.live.join("collection_coin_edits.json");
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&path, permissions).unwrap();

    let out = realm(&mut game, "realm coin place 1954", &who);
    assert!(
        said(&out).contains("Could not save the coin"),
        "{}",
        said(&out)
    );
    assert!(!out.changed);
    assert_eq!(snapshot(&game, &fixture), before, "old file, old view");
    let rows = fixture.rows();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["entry_id"], json!(1950));
}
