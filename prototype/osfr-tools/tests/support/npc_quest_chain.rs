//! Shared fixture and packet helpers for the NPC quest-chain tests.
//!
//! Test files include this module with
//! `#[path = "support/npc_quest_chain.rs"] mod npc_quest_chain;` (grep `tests`
//! for the current consumers: the entry chains, `quest_job_xp.rs`, the
//! story-quest suites including `missing_sisters_quest.rs` and
//! `local_hero_quest.rs`, and `shared_field_kill_quest_credit.rs`). Test-only;
//! production never copies content. The fixture supplies profile/inventory
//! content; the repository's `job_trainers.json` and split quest/NPC content are
//! bound from `data/content` exactly as the bundled play package does.
#![allow(dead_code)]

#[path = "../common/content_copy.rs"]
mod content_copy;

#[path = "../common/gateway_packets.rs"]
mod gateway_packets;

#[path = "npc_visibility.rs"]
mod npc_visibility;

#[path = "pickup_packets.rs"]
mod pickup_packets;

pub use content_copy::{copy_missing, copy_tree};
#[allow(unused_imports)]
pub use npc_visibility::node_content::{pickups, shipped_nodes};
#[allow(unused_imports)]
pub use npc_visibility::{content_dir, seen_from};

use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    chat,
    encounter::{FieldParticipant, SharedFieldAction, SharedFieldState},
    gameplay::{Gameplay, GameplayState, Outcome},
    gateway::Gateway,
    harvest_nodes::NodeActor,
    inventory::InventoryItem,
    npc_interaction::{decode_dialogue_26_3, decode_quest_offer_49_1},
    quest_engine::{QuestProgress, QuestStatus},
};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};

pub const CHARACTER: u64 = 194_017;
/// The Adventurer profile every imported zone quest pays its XP to.
pub const ADVENTURER: i32 = 1;
/// Generic Talk option label in the reconstructed 26/9 menu.
pub const TALK_LABEL: u32 = 3_300;
/// First response row of the quest dialogue (client 26/6).
pub const QUEST_CHOICE: i32 = 301;
/// (R) The production turn-in floor in `gameplay/npc.rs`: the reward
/// `bag_capacity` is `items.len().max(100)`.
pub const BAG_SLOTS: usize = 100;
/// A real inventory definition used to fill the bag; only the stack count matters.
pub const FILLER_ITEM: i32 = 75_060;

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

pub fn reference_content() -> PathBuf {
    std::env::var_os("OSFR_NPC_QUEST_CHAIN_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/opening-anvil-placement-v1/content")
        })
}

pub fn read_json(path: &Path) -> Value {
    serde_json::from_slice(
        &fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

pub struct Fixture {
    path: PathBuf,
    character_id: u64,
}

impl Fixture {
    pub fn new() -> Self {
        Self::for_character(CHARACTER)
    }

    /// A private content copy whose `game` binds this character's quest and reward
    /// ledger; two sessions in the same fixture need separate character ids.
    pub fn for_character(character_id: u64) -> Self {
        let source = reference_content();
        assert!(
            source.join("profiles.json").is_file() && source.join("inventory.json").is_file(),
            "the quest chapter needs the opening content fixture at {}",
            source.display()
        );
        let path = std::env::temp_dir().join(format!(
            "osfr-npc-quest-chain-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        copy_tree(&source, &path);
        fs::copy(
            content_dir().join("job_trainers.json"),
            path.join("job_trainers.json"),
        )
        .unwrap();
        Self { path, character_id }
    }

    /// Copy every repository content file this fixture does not already supply, so
    /// `Gameplay::load` reads NPCs, field actors and `jobs.json` from the fixture's own
    /// tree. Opening/profile/inventory files keep winning (`copy_missing`), and the
    /// caller may then patch the scratch copy.
    pub fn overlay_content(&self, source: &Path) {
        copy_missing(source, &self.path);
    }

    /// Copy one repository content file into this fixture's own tree. The opening
    /// fixture does not ship every family (for example the housing family's
    /// `houses.json`), and a session reads only the fixture's `game` directory, so
    /// a test that needs one copies it here rather than into `data/content`.
    pub fn copy_content(&self, name: &str) {
        fs::copy(content_dir().join(name), self.path.join(name)).unwrap();
    }

    /// A copy without the opening scene and the campaign files that require it,
    /// so `Gameplay::on_location` cannot advance the opening story. Event-quest
    /// tests use it when their objective and wallet deltas must be exact.
    pub fn minimal(character_id: u64) -> Self {
        let fixture = Self::for_character(character_id);
        for name in [
            "opening.json",
            "opening-abilities.json",
            "smelting.json",
            "forging.json",
            "forging-campaign.json",
            "cooking.json",
            "cooking-campaign.json",
            "mining.json",
            "harvesting.json",
        ] {
            let path = fixture.path.join(name);
            if path.is_file() {
                fs::remove_file(&path).unwrap();
            }
        }
        fixture
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// T418: this fixture's private live folder (the `--world` layer) with the
    /// live events `ids` open, as an admin's `realm event open` leaves
    /// `events.json`. A session reads it only after `set_world_dir`; without one
    /// its live folder is its NPC content directory, usually the bundled
    /// `data/content` every test shares.
    pub fn open_live_events(&self, ids: impl IntoIterator<Item = u32>) -> PathBuf {
        let live = self.path.join("live");
        fs::create_dir_all(&live).unwrap();
        let open: Vec<u32> = ids.into_iter().collect();
        let text = serde_json::json!({"schema_version": 1, "open": open}).to_string();
        fs::write(live.join("events.json"), text).unwrap();
        live
    }

    /// Replace the shared field population with `enemies`. The reference opening
    /// content has no `field_enemies.json`, so this also makes the private copy
    /// the session's NPC content root (the spawn catalogue, store and vendor
    /// files are copied from `data/content`) and copies T240's
    /// `field_kill_rewards.json`; `Gameplay::load` then reads the synthetic field
    /// through its production path instead of the bundled one. The split quest
    /// catalogue is still bound from `data/content` by [`Self::game_as`].
    pub fn with_field_enemies(self, enemies: &[Value]) -> Self {
        for name in [
            "zones.json",
            "npc_spawns_extra.json",
            "client_areas.json",
            "stores.json",
            "vendors.json",
            "field_kill_rewards.json",
        ] {
            fs::copy(content_dir().join(name), self.path.join(name)).unwrap();
        }
        copy_tree(&content_dir().join("npcs"), &self.path.join("npcs"));
        fs::write(
            self.path.join("field_enemies.json"),
            serde_json::to_vec(&serde_json::json!({
                "schema_version": 1,
                "enemies": enemies,
            }))
            .unwrap(),
        )
        .unwrap();
        self
    }

    pub fn game(&self, state: Option<GameplayState>) -> Gameplay {
        self.game_as(self.character_id, state)
    }

    /// Bind the split catalogue for a specific character, so a second session in
    /// the same fixture owns its own quest and reward ledger. The bind reads
    /// [`Self::quest_catalogue`], which keeps every quest and giver binding but drops the NPC
    /// rows the loader never reads.
    pub fn game_as(&self, character_id: u64, state: Option<GameplayState>) -> Gameplay {
        let mut game = Gameplay::load(&self.path, state).expect("opening fixture loads");
        game.bind_npc_quest_catalogue(&self.quest_catalogue(), character_id)
            .expect("split quest content loads");
        game
    }

    /// The shipped split quest catalogue, narrowed to what the quest loader reads, inside this
    /// fixture (built once, reused by every later bind). `QuestEngine::from_split_content` reads
    /// all 25 MB of `data/content/npcs` for each NPC's quest bindings alone, and a suite that
    /// binds one engine per save layer pays that once per layer. The copy keeps every quest file,
    /// the job/crosswalk/job-browser/marker side files and exactly the NPC rows that carry a
    /// binding (the loader keeps nothing else from a row), so every giver and turn-in mapping and
    /// every cross-check against `data/content` is unchanged. No test writes to `data/content`,
    /// so the binding rows are read once per test binary.
    pub fn quest_catalogue(&self) -> PathBuf {
        let catalogue = self.path.join("quest-catalogue");
        if catalogue.join("quests").is_dir() {
            return catalogue;
        }
        let source = content_dir();
        fs::create_dir_all(catalogue.join("quests")).unwrap();
        fs::create_dir_all(catalogue.join("npcs")).unwrap();
        for entry in fs::read_dir(source.join("quests")).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|ext| ext == "json") {
                fs::copy(
                    &path,
                    catalogue.join("quests").join(path.file_name().unwrap()),
                )
                .unwrap();
            }
        }
        for name in [
            "jobs.json",
            "job_browser.json",
            "quest_markers.json",
            "breadcrumb_quests.json",
        ] {
            let path = source.join(name);
            if path.is_file() {
                fs::copy(&path, catalogue.join(name)).unwrap();
            }
        }
        fs::write(
            catalogue.join("npcs/bindings.json"),
            serde_json::to_vec(&serde_json::json!({
                "schema_version": 1,
                "records": quest_binding_rows(),
            }))
            .unwrap(),
        )
        .unwrap();
        catalogue
    }

    /// Rewrite one quest record in this fixture's narrowed catalogue (the shipped tree is never
    /// touched) and return the file's original bytes, so a test can put them back.
    pub fn patch_quest(&self, file: &str, quest_id: u32, edit: impl FnOnce(&mut Value)) -> Vec<u8> {
        let path = self.quest_catalogue().join("quests").join(file);
        let original = fs::read(&path).unwrap();
        let mut document: Value = serde_json::from_slice(&original).unwrap();
        let record = document["records"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|record| record["id"] == quest_id)
            .unwrap_or_else(|| panic!("quest {quest_id} is in {file}"));
        edit(record);
        fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();
        original
    }

    /// A logged-in, world-loaded `Gateway` session standing at `position`, running this
    /// fixture's gameplay with the split quest catalogue bound for `guid`. Requests go through
    /// [`gateway_send`], the real `Gateway::handle_at` framing and commit rules.
    pub fn session(&self, guid: u64, position: [f32; 3], state: Option<GameplayState>) -> Gateway {
        let reference = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../lab/references/rust-gateway-reference");
        let version = read_json(&reference.join("manifest.json"))["client_version"]
            .as_str()
            .unwrap()
            .to_owned();
        let mut character = Gateway::reference_state(&reference).unwrap();
        character.guid = guid;
        character.position = position;
        let mut gateway = Gateway::load(&reference, character).unwrap();
        gateway.enable_gameplay(&self.path, state).unwrap();
        gateway
            .gameplay_mut()
            .unwrap()
            .bind_npc_quest_catalogue(&self.quest_catalogue(), guid)
            .unwrap();
        let login = gateway_packets::login_with_version(guid, &version);
        gateway.handle_at(&login, 0).unwrap();
        // The client's Ready (13) then Loaded (10) events.
        for (event, now) in [(13_u16, 100), (10_u16, 200)] {
            gateway_send(&mut gateway, &event.to_le_bytes(), now);
        }
        gateway
    }

    /// Reload the saved character through the same production bind the session uses.
    pub fn reload(&self, game: &Gameplay) -> Gameplay {
        self.game(Some(game.state()))
    }

    /// Reload through a JSON round trip, so decoding the saved state is exercised too.
    pub fn reload_serialized(&self, game: &Gameplay) -> Gameplay {
        self.reload_serialized_as(self.character_id, game)
    }

    /// As [`Self::reload_serialized`], but bind the split quest catalogue for
    /// `character_id`: a second session in the same fixture saves under its own id.
    pub fn reload_serialized_as(&self, character_id: u64, game: &Gameplay) -> Gameplay {
        let bytes = serde_json::to_vec(&game.state()).unwrap();
        let state: GameplayState = serde_json::from_slice(&bytes).unwrap();
        self.game_as(character_id, Some(state))
    }

    /// The production save envelope, not a `GameplayState` proxy: `game_store::save`
    /// -> `load_or_create` -> the same split-catalogue bind a login performs. The
    /// reload must carry the serialized state, not just the character it was given.
    pub fn save_and_reload(&self, game: &Gameplay, character: &CharacterState) -> Gameplay {
        let path = self.path.join("character-save.json");
        let snapshot = osfr_tools::game_store::Snapshot {
            character: character.clone(),
            gameplay: Some(game.state()),
            pets: None,
        };
        osfr_tools::game_store::save(&path, &snapshot).expect("the character save writes");
        let loaded = osfr_tools::game_store::load_or_create(&path, character.clone())
            .expect("the character save loads");
        let reloaded = self.game_as(character.guid, loaded.gameplay);
        assert_eq!(
            saved(&reloaded),
            saved(game),
            "the save round trip preserved the serialized state"
        );
        reloaded
    }
}

/// The shipped NPC rows that name a quest, in directory order. `from_split_content` keeps an NPC
/// row's id and its three quest-binding lists and nothing else, so this is the whole of the
/// 25 MB `data/content/npcs` tree the loader reads. Read once per test binary: every fixture's
/// narrowed catalogue is written from it.
fn quest_binding_rows() -> &'static [Value] {
    static ROWS: OnceLock<Vec<Value>> = OnceLock::new();
    ROWS.get_or_init(|| {
        let mut rows = Vec::new();
        for entry in fs::read_dir(content_dir().join("npcs")).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|ext| ext != "json") {
                continue;
            }
            for record in read_json(&path)["records"].as_array().unwrap() {
                let bound = ["quest_ids_given", "quests_given", "quests_accepted"]
                    .iter()
                    .any(|key| {
                        record[*key]
                            .as_array()
                            .is_some_and(|bindings| !bindings.is_empty())
                    });
                if bound {
                    rows.push(serde_json::json!({
                        "npc_id": record["npc_id"],
                        "quest_ids_given": record["quest_ids_given"],
                        "quests_given": record["quests_given"],
                        "quests_accepted": record["quests_accepted"],
                    }));
                }
            }
        }
        rows
    })
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// T229: the wallet one accepted quest turn-in writes (see [`wallet_after`]).
pub fn wallet_after_quest(
    wallet: Option<osfr_tools::station_cash::Wallet>,
    quest_id: u32,
) -> Option<osfr_tools::station_cash::Wallet> {
    wallet_after(
        wallet,
        osfr_tools::station_cash::EarnSource::Quest { id: quest_id },
    )
}

/// T229: the wallet one server-verified completion writes, computed from the shipped earn table
/// through production's own `EarnTable::earned` (a content change updates the expectation; the
/// daily bonus and its UTC day are never hard-coded). `None` means the completion left no wallet.
pub fn wallet_after(
    wallet: Option<osfr_tools::station_cash::Wallet>,
    source: osfr_tools::station_cash::EarnSource,
) -> Option<osfr_tools::station_cash::Wallet> {
    let table = osfr_tools::station_cash::EarnTable::load(&content_dir().join("sc_rewards.json"))
        .expect("the earn table reads")
        .expect("the bundled earn table exists");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_secs();
    let mut wallet = wallet.unwrap_or_default();
    let earned = table.earned(
        &mut wallet,
        source,
        osfr_tools::station_cash::activity_day(now),
        None,
    );
    assert!(
        earned.sc > 0 && earned.ride.is_none(),
        "the shipped table pays this completion in Station Cash only"
    );
    // The one reward applier adds the decided amount to the staged wallet (below the bundled
    // ceiling, so the cap and its notice do not apply here).
    wallet.balance = wallet
        .balance
        .checked_add(earned.sc)
        .expect("the expected wallet balance fits");
    (!wallet.is_empty()).then_some(wallet)
}

pub fn character_at_for(guid: u64, zone_id: u32, position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid,
        zone_id,
        position,
        rotation: [1.0, 0.0, 0.0],
    }
}

pub fn character_at(zone_id: u32, position: [f32; 3]) -> CharacterState {
    character_at_for(CHARACTER, zone_id, position)
}

/// Fill the bag to the production capacity floor (exactly [`BAG_SLOTS`] stacks), so
/// the next new stack is refused. Existing opening items count toward the total; the
/// filler rows are real inventory instances.
pub fn fill_bag(state: &mut GameplayState) {
    let base = 100_000;
    for offset in state.inventory.items.len()..BAG_SLOTS {
        state.inventory.items.push(InventoryItem {
            definition_id: FILLER_ITEM,
            tint: 0,
            id: base + offset as i32,
            count: 1,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: 0,
            rental: None,
            ability_count: 0,
            activate_enabled: false,
        });
    }
    state.inventory.next_instance_id = Some(i64::from(base) + BAG_SLOTS as i64);
}

/// `npc_id`'s row position in the shipped `npcs/<file>` record. The zone files are up to 3 MB and
/// the suites ask for many rows, so each file is parsed once per test binary; no test writes to
/// `data/content`.
pub fn content_position(file: &str, npc_id: u64) -> [f32; 3] {
    static DOCUMENTS: OnceLock<std::sync::Mutex<std::collections::BTreeMap<String, Value>>> =
        OnceLock::new();
    let mut documents = DOCUMENTS
        .get_or_init(|| std::sync::Mutex::new(std::collections::BTreeMap::new()))
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let document = documents
        .entry(file.to_owned())
        .or_insert_with(|| read_json(&content_dir().join("npcs").join(file)));
    let row = document["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["npc_id"].as_u64() == Some(npc_id))
        .unwrap_or_else(|| panic!("NPC {npc_id} is defined in {file}"));
    serde_json::from_value(row["position"].clone()).expect("NPC has a position")
}

pub fn npc_interact(target: u64) -> Vec<u8> {
    let mut packet = vec![26, 0, 8, 0];
    packet.extend(target.to_le_bytes());
    packet.push(0);
    packet
}

pub fn npc_select(target: u64, choice: i32) -> Vec<u8> {
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(target.to_le_bytes());
    packet.extend(choice.to_le_bytes());
    packet
}

pub fn dialogue_choice(row: i32) -> Vec<u8> {
    let mut packet = vec![26, 0, 6, 0];
    packet.extend(row.to_le_bytes());
    packet
}

pub fn quest_reply(quest_id: u32, accept: bool) -> Vec<u8> {
    let mut packet = vec![49, 0, 2, 0, 0, 0];
    packet.extend(quest_id.to_le_bytes());
    packet.push(u8::from(accept));
    packet
}

pub fn quest_end_reply(quest_id: u32) -> Vec<u8> {
    let mut packet = vec![49, 0, 14, 0, 0, 0];
    packet.extend(quest_id.to_le_bytes());
    packet
}

/// 49/15 `QuestStartBreadcrumb`: the client's 10-byte request, `[49, 0, 15, 0,
/// 0, 0]` then the ORIGINAL quest id as `i32`. Header writer `0x00a8f860` writes
/// `u16 49` then `u32 15`; payload writer `0x00a91730` writes the id from the
/// ctor's `+0xc`. Sender `0x00a97f90` (`Ui.StartBreadcrumbQuest`, Lua `0x00c18aa0`)
/// -> ctor `0x00a86580`: family 49, subtype 15, the id at `+0xc`. The server
/// decoder is `quest_journal::decode`. The client sends the original id; the
/// server translates it through `breadcrumb_quests.json` or the Jobs-window
/// crosswalk.
pub fn breadcrumb_start(original_quest_id: i32) -> Vec<u8> {
    [&[49, 0, 15, 0, 0, 0][..], &original_quest_id.to_le_bytes()].concat()
}

/// 26/23 abandon, the client's 8-byte body: writer `0x00c0a350` (preamble `0x0090ac70`
/// writes u16 26 + u16 23, then the u32 quest id).
pub fn quest_abandon(quest_id: u32) -> Vec<u8> {
    let mut packet = vec![26, 0, 23, 0];
    packet.extend(quest_id.to_le_bytes());
    packet
}

pub fn select_job(profile: i32) -> Vec<u8> {
    let mut packet = vec![26, 0, 13, 0];
    packet.extend(profile.to_le_bytes());
    packet
}

pub fn send(game: &mut Gameplay, character: &CharacterState, packet: &[u8], now: u64) -> Outcome {
    game.handle_in_world(packet, character, now)
        .unwrap_or_else(|error| {
            panic!(
                "packet {:?} was refused: {error}",
                &packet[..packet.len().min(4)]
            )
        })
}

/// One request through the real `Gateway::handle_at` framing: the application bodies it replies
/// with, `005` tunnel stripped.
pub fn gateway_send(gateway: &mut Gateway, inner: &[u8], now: u64) -> Vec<Vec<u8>> {
    gateway
        .handle_at(&gateway_packets::wrap(inner), now)
        .expect("gateway frame")
        .replies
        .iter()
        .map(|reply| gateway_packets::body(reply).to_vec())
        .collect()
}

pub fn has_family(packet: &[u8], family: u16, subtype: u16, id: Option<u32>) -> bool {
    if packet.len() < 6
        || packet[..2] != family.to_le_bytes()
        || packet[2..4] != subtype.to_le_bytes()
    {
        return false;
    }
    let Some(id) = id else { return true };
    // 49/13 carries the turn-in speaker at 6..14 and the reply quest id at 14..18;
    // the other family/subtype pairs used here carry their id at 6..10.
    let offset = if (family, subtype) == (49, 13) { 14 } else { 6 };
    packet
        .get(offset..offset + 4)
        .is_some_and(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()) == id)
}

/// 35/5 `PlayerUpdatePacketUpdateHitpoints` reader `0x008d6f10` (base header reader
/// `0x008d6830`): after the four-byte header it reads the actor guid as two dwords
/// (`4..12`), then three i32s; the first is the current hitpoints (`12..16`).
pub fn enemy_hp(replies: &[Vec<u8>], guid: u64) -> Option<u32> {
    replies.iter().find_map(|packet| {
        (has_family(packet, 35, 5, None)
            && packet.get(4..12) == Some(guid.to_le_bytes().as_slice())
            && packet.len() >= 16)
            .then(|| u32::from_le_bytes(packet[12..16].try_into().unwrap()))
    })
}

/// Full 49/8 `ObjectiveActivated` packet: the 6-byte family/subtype header
/// (`0x00c7acd0`), quest id, objective id, required count, and the reconstructed
/// (R) visible-notification bool. Body reader `0x00c7aeb0` (through `0x00c7b570`);
/// apply `0x00c7a910`.
pub fn objective_activated_packet(quest_id: u32, objective_id: u32, required: u32) -> Vec<u8> {
    let mut packet = vec![49, 0, 8, 0, 0, 0];
    for value in [quest_id, objective_id, required] {
        packet.extend(value.to_le_bytes());
    }
    packet.push(0);
    packet
}

/// Full 49/9 `QuestObjectiveUpdatePacket`: header, quest id, objective id,
/// current counter, progress float, (R) notification bool and one (R)
/// unresolved trailing dword. Reader `0x00c7af60` reads exactly that order.
pub fn objective_update_packet(
    quest_id: u32,
    objective_id: u32,
    current: u32,
    progress: f32,
) -> Vec<u8> {
    let mut packet = vec![49, 0, 9, 0, 0, 0];
    for value in [quest_id, objective_id, current] {
        packet.extend(value.to_le_bytes());
    }
    packet.extend(progress.to_le_bytes());
    packet.push(0);
    packet.extend(0_i32.to_le_bytes());
    packet
}

/// Full 49/10 `QuestObjectiveCompletePacket`: header, quest id, objective id,
/// progress float, one (R) unresolved dword and the (R) completion bool. Reader
/// `0x00c7b070` reads exactly that order.
pub fn objective_complete_packet(quest_id: u32, objective_id: u32, progress: f32) -> Vec<u8> {
    let mut packet = vec![49, 0, 10, 0, 0, 0];
    for value in [quest_id, objective_id] {
        packet.extend(value.to_le_bytes());
    }
    packet.extend(progress.to_le_bytes());
    packet.extend(0_i32.to_le_bytes());
    packet.push(0);
    packet
}

/// The selected option id for `label` in one 26/9 interaction menu body.
/// Dispatcher `0x00aa2360` -> `0x00aa2220` -> `0x00aa1600` (guid at 4, flag
/// at 12) -> list reader `0x00aa04d0` (count at 13) -> row reader `0x00a9dca0`
/// (seven dwords, 28-byte rows from 17; label at +8). `multiplayer.rs` shares it.
pub fn menu_option_bytes(packet: &[u8], label: u32) -> i32 {
    let count = i32::from_le_bytes(packet[13..17].try_into().unwrap());
    assert!(count > 0, "the interaction menu has no options");
    (0..count)
        .find_map(|index| {
            let offset = 17 + usize::try_from(index).unwrap() * 28;
            let id = i32::from_le_bytes(packet[offset..offset + 4].try_into().unwrap());
            let string_id = u32::from_le_bytes(packet[offset + 8..offset + 12].try_into().unwrap());
            (string_id == label).then_some(id)
        })
        .unwrap_or_else(|| panic!("menu label {label} is absent"))
}

pub fn menu_option(outcome: &Outcome, label: u32) -> i32 {
    let menu = outcome
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 9, 0]))
        .expect("26/8 returns the 26/9 interaction menu");
    menu_option_bytes(menu, label)
}

pub fn interact(game: &mut Gameplay, character: &CharacterState, target: u64, now: u64) -> Outcome {
    send(game, character, &npc_interact(target), now)
}

pub fn talk(game: &mut Gameplay, character: &CharacterState, target: u64, now: u64) -> Outcome {
    let menu = interact(game, character, target, now);
    let choice = menu_option(&menu, TALK_LABEL);
    send(game, character, &npc_select(target, choice), now + 1)
}

/// Complete one ordered Talk objective. A final target opens the pending end screen,
/// which the client releases with its 26/6 dialogue choice.
pub fn talk_step(
    game: &mut Gameplay,
    character: &CharacterState,
    target: u64,
    now: u64,
    label: &str,
) {
    let talked = talk(game, character, target, now);
    assert!(talked.changed, "{label} did not advance its objective");
}

pub fn offer(
    game: &mut Gameplay,
    character: &CharacterState,
    target: u64,
    quest_id: u32,
    now: u64,
) -> Outcome {
    let dialogue = talk(game, character, target, now);
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "Talk must open the 26/3 dialogue"
    );
    let offered = send(game, character, &dialogue_choice(QUEST_CHOICE), now + 2);
    let offers: Vec<&Vec<u8>> = offered
        .replies
        .iter()
        .filter(|packet| packet.starts_with(&[49, 0, 1, 0]))
        .collect();
    assert_eq!(offers.len(), 1, "one Talk offers exactly one quest");
    // Decode with the production reader so a wrong field order or a corrupt
    // packet fails here in every chapter, not only in the Wizard card test.
    let decoded = decode_quest_offer_49_1(offers[0].as_slice())
        .expect("the offered 49/1 decodes with the production reader");
    assert_eq!(
        decoded.quest_id, quest_id,
        "quest {quest_id} was not offered"
    );
    assert_eq!(quest_status(game, quest_id), QuestStatus::Offered);
    offered
}

pub fn accept(game: &mut Gameplay, character: &CharacterState, quest_id: u32, now: u64) -> Outcome {
    let accepted = send(game, character, &quest_reply(quest_id, true), now);
    assert!(
        accepted
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 3, Some(quest_id))),
        "quest {quest_id} did not send QuestAdd"
    );
    assert!(
        accepted
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 8, Some(quest_id))),
        "quest {quest_id} did not activate its first objective"
    );
    assert_eq!(quest_status(game, quest_id), QuestStatus::Active);
    accepted
}

/// 26/6 releases the 49/13 that the final Talk held. Its speaker must be the
/// runtime actor the client clicked, not the content key, and its end text must
/// be the quest's debrief. `QuestEndPacket::ReadFields` (`00c7b990`) reads the
/// common header, actor `6..14`, quest id `14..18`, end text `18..22`, one more
/// dword at `22..26`, the reward bundle (`008e7930`), then a trailing float;
/// `ShowQuestEndScreen` (`00a95420`) passes the end text dword as the message id.
pub fn release_end_screen(
    game: &mut Gameplay,
    character: &CharacterState,
    quest_id: u32,
    expected_actor: u64,
    expected_end_text: u32,
    now: u64,
) -> Vec<u8> {
    let released = send(game, character, &dialogue_choice(QUEST_CHOICE), now);
    let end = released
        .replies
        .iter()
        .find(|packet| has_family(packet, 49, 13, Some(quest_id)))
        .cloned()
        .unwrap_or_else(|| panic!("quest {quest_id} did not release its 49/13 end screen"));
    assert_eq!(
        u64::from_le_bytes(end[6..14].try_into().unwrap()),
        expected_actor,
        "the end screen must name the clicked runtime actor"
    );
    assert_eq!(
        u32::from_le_bytes(end[18..22].try_into().unwrap()),
        expected_end_text,
        "the end screen must carry the quest's debrief text"
    );
    end
}

/// The 49/13 reward bundle's leading end-screen words. `QuestEndPacket::ReadFields`
/// (`00c7b990`) reads the actor at `6..14`, quest id at `14..18`, end text at `18..22`, the
/// animation dword at `22..26`, then calls the reward bundle reader (`008e7930`): a bool at
/// `26`, coins at `27..31`, and the stars total at `31..35` (the sum of the quest's job XP,
/// not one profile's).
pub fn end_screen_reward(end: &[u8]) -> (u32, u32) {
    assert!(
        end.len() >= 35,
        "a 49/13 packet carries the reward bundle's coin and star words"
    );
    (
        u32::from_le_bytes(end[27..31].try_into().unwrap()),
        u32::from_le_bytes(end[31..35].try_into().unwrap()),
    )
}

/// The ending profile is explicit: a quest's reward map names its own job, which
/// need not be the profile the character has active at turn-in. `expected_profile_xp`
/// is that profile's share, not the 49/13 star word (every profile summed).
pub fn finish_quest(
    game: &mut Gameplay,
    character: &CharacterState,
    quest_id: u32,
    profile: i32,
    expected_coins: u32,
    expected_profile_xp: u32,
    now: u64,
) -> Outcome {
    let coins_before = coins(game);
    let xp_before = profile_xp(game, profile);
    let completed = send(game, character, &quest_end_reply(quest_id), now);
    assert!(
        completed.changed,
        "quest {quest_id} turn-in did not change state"
    );
    assert!(
        completed
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 4, Some(quest_id))),
        "quest {quest_id} did not complete"
    );
    assert_eq!(coins(game), coins_before + expected_coins);
    assert_eq!(profile_xp(game, profile), xp_before + expected_profile_xp);
    assert_eq!(quest_status(game, quest_id), QuestStatus::Completed);
    assert_eq!(pending_offer(game), None);
    assert_eq!(pending_turnin(game), None);
    completed
}

/// One imported community zone quest: giver, the ordered Talk steps before the
/// turn-in (each with the 26/3 body its NPC speaks when the step completes), and
/// the final Talk at `turnin`.
pub struct ZoneQuest {
    pub id: u32,
    pub giver: u64,
    /// Title, intro and description ids the 49/1 card carries.
    pub card: [u32; 3],
    pub steps: &'static [(u64, u32)],
    pub turnin: u64,
    pub end_text: u32,
    /// The 49/13 reward bundle's coin word.
    pub coins: u32,
    /// The 49/13 reward bundle's star word: every `job_xp` profile's XP summed
    /// (production's `encode_quest_reward_bundle`).
    pub xp: u32,
    /// What `finish_quest` expects the turn-in's named `profile` to gain. A record
    /// that pays one profile sets it equal to `xp`; a multi-profile record does not.
    pub profile_xp: u32,
}

/// A zone's quest runner: where its NPC rows live, the runtime actor the client
/// clicks for each row, and the one 26/8 -> 26/10 -> 26/6 offer, 49/2 accept,
/// ordered Talk step and 26/6 -> 49/13 -> 49/14 turn-in walk every zone suite uses.
pub struct ZoneQuests {
    /// The zone file under `data/content/npcs` holding every NPC not in `files`.
    pub default_file: &'static str,
    /// NPCs whose row lives in another zone file (a giver in a neighbouring zone).
    pub files: &'static [(u64, &'static str)],
    /// Extra spawns: the runtime actor guid -> the content NPC its `content_key` names.
    pub runtime: &'static [(u64, u64)],
}

impl ZoneQuests {
    /// The NPC content file a row lives in.
    fn file(&self, npc: u64) -> &'static str {
        self.files
            .iter()
            .find_map(|(id, file)| (*id == npc).then_some(*file))
            .unwrap_or(self.default_file)
    }

    /// The actor the client clicks: an extra spawn is its runtime guid, every other
    /// row spawns as its own content key.
    pub fn actor(&self, npc: u64) -> u64 {
        self.runtime
            .iter()
            .find_map(|(id, actor)| (*id == npc).then_some(*actor))
            .unwrap_or(npc)
    }

    /// The character at `npc`'s content row; every imported zone quest is in zone 0.
    pub fn at(&self, guid: u64, npc: u64) -> CharacterState {
        character_at_for(guid, 0, content_position(self.file(npc), npc))
    }

    /// Offer the quest at its giver and accept it, pinning the 49/1 card ids.
    pub fn accept(&self, game: &mut Gameplay, guid: u64, quest: &ZoneQuest, now: &mut u64) {
        let giver = self.at(guid, quest.giver);
        let offered = offer(game, &giver, self.actor(quest.giver), quest.id, *now);
        let card = offered
            .replies
            .iter()
            .find(|packet| has_family(packet, 49, 1, Some(quest.id)))
            .and_then(|packet| decode_quest_offer_49_1(packet).ok())
            .expect("the 49/1 card decodes");
        assert_eq!(
            [
                card.name_string_id,
                card.intro_text_string_id,
                card.description_string_id
            ],
            quest.card,
            "quest {}: 49/1 title, intro and description",
            quest.id
        );
        accept(game, &giver, quest.id, *now + 2);
        *now += 10;
    }

    /// Talk at `npc`, clicking its runtime actor.
    pub fn talk(&self, game: &mut Gameplay, guid: u64, npc: u64, now: u64) -> Outcome {
        talk(game, &self.at(guid, npc), self.actor(npc), now)
    }

    /// Click response `row` of the open dialogue while standing at `npc`.
    pub fn answer(&self, game: &mut Gameplay, guid: u64, npc: u64, row: i32, now: u64) -> Outcome {
        send(game, &self.at(guid, npc), &dialogue_choice(row), now)
    }

    /// Complete one ordered Talk objective, clicking the runtime actor of an extra spawn.
    pub fn step(
        &self,
        game: &mut Gameplay,
        guid: u64,
        quest: &ZoneQuest,
        index: usize,
        now: &mut u64,
    ) {
        let (npc, body) = quest.steps[index];
        let talked = talk(game, &self.at(guid, npc), self.actor(npc), *now);
        assert_eq!(
            dialogue_body(&talked),
            body,
            "quest {} step {}: the NPC's 26/3 line",
            quest.id,
            index + 1
        );
        assert!(
            talked.changed,
            "quest {} step {} did not advance",
            quest.id,
            index + 1
        );
        assert_eq!(counts(game, quest.id)[index], 1);
        *now += 10;
    }

    /// The final Talk, the held 49/13 with its reward bundle, and the 49/14 that pays.
    pub fn turn_in(
        &self,
        game: &mut Gameplay,
        guid: u64,
        quest: &ZoneQuest,
        profile: i32,
        now: &mut u64,
    ) {
        let turnin = self.at(guid, quest.turnin);
        let talked = talk(game, &turnin, self.actor(quest.turnin), *now);
        assert!(talked.changed, "quest {}: the final Talk", quest.id);
        self.pay_held_turnin(game, guid, quest, profile, now);
    }

    /// With the hand-in already held (the final Talk done, possibly before a reload), the
    /// 26/6 that releases the 49/13 and the 49/14 that pays. The end screen carries the
    /// record's coin word and star total; the grant pays `profile` only its share.
    pub fn pay_held_turnin(
        &self,
        game: &mut Gameplay,
        guid: u64,
        quest: &ZoneQuest,
        profile: i32,
        now: &mut u64,
    ) {
        let turnin = self.at(guid, quest.turnin);
        assert_eq!(pending_turnin(game), Some(quest.id));
        let end = release_end_screen(
            game,
            &turnin,
            quest.id,
            self.actor(quest.turnin),
            quest.end_text,
            *now + 2,
        );
        assert_eq!(end_screen_reward(&end), (quest.coins, quest.xp));
        finish_quest(
            game,
            &turnin,
            quest.id,
            profile,
            quest.coins,
            quest.profile_xp,
            *now + 4,
        );
        *now += 10;
    }

    /// Offer, accept, walk every ordered step and turn the quest in.
    pub fn run(
        &self,
        game: &mut Gameplay,
        guid: u64,
        quest: &ZoneQuest,
        profile: i32,
        now: &mut u64,
    ) {
        self.accept(game, guid, quest, now);
        for index in 0..quest.steps.len() {
            self.step(game, guid, quest, index, now);
        }
        self.turn_in(game, guid, quest, profile, now);
    }
}

/// The 26/3 dialogue body (its first dword) a completed Talk step speaks. Client
/// reader `0x00a9ef10`; the layout is the one `decode_dialogue_26_3` parses.
pub fn dialogue_body(outcome: &Outcome) -> u32 {
    dialogue_line(&outcome.replies).0
}

/// The 26/3 body and its five-dword response rows (the first dword is the id a 26/6 echoes),
/// decoded with the production reader. A quest conversation answers every step with one.
pub fn dialogue_line(replies: &[Vec<u8>]) -> (u32, Vec<[u32; 5]>) {
    let packet = replies
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 3, 0]))
        .expect("the reply carries a 26/3");
    let decoded = decode_dialogue_26_3(packet).expect("the 26/3 decodes");
    (decoded.initial_dwords[0], decoded.choices)
}

/// 26/8 then the Talk row of the 26/10 menu through [`gateway_send`], like [`talk`] does for a
/// bare `Gameplay`. Returns the 26/10 replies (the dialogue or the conversation's first line).
pub fn gateway_talk(gateway: &mut Gateway, target: u64, now: u64) -> Vec<Vec<u8>> {
    let menu = gateway_send(gateway, &npc_interact(target), now);
    let menu = menu
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 9, 0]))
        .expect("26/8 answers the 26/9 menu");
    let choice = menu_option_bytes(menu, TALK_LABEL);
    gateway_send(gateway, &npc_select(target, choice), now + 1)
}

/// Whether the replies carry the server's 26/4 EndDialog.
pub fn dialogue_closed(replies: &[Vec<u8>]) -> bool {
    replies.iter().any(|packet| packet == &[26, 0, 4, 0])
}

/// Put `quest_id` in the saved quest state with `status` and `counts`, when a test needs a
/// quest mid-way without replaying its chain (the loader still validates the counters). The
/// full chains have their own suites.
pub fn seed_quest(state: &mut GameplayState, quest_id: u32, status: QuestStatus, counts: &[u32]) {
    state.npc_quests.as_mut().unwrap().quests.insert(
        quest_id,
        QuestProgress {
            status,
            counts: counts.to_vec(),
        },
    );
}

/// A player on the Adventurer ladder with `active` as the saved job, so the XP
/// destination is observable. The fixture owns the quest and reward ledger.
pub fn zone_player(fixture: &Fixture, guid: u64, active: i32) -> Gameplay {
    zone_player_unlocked(fixture, guid, active, &[ADVENTURER])
}

/// [`zone_player`] with the jobs `unlocked` learned while `active` stays selected, so
/// a test can prove a quest gate reads the *active* job and not merely a learned one.
pub fn zone_player_unlocked(
    fixture: &Fixture,
    guid: u64,
    active: i32,
    unlocked: &[i32],
) -> Gameplay {
    zone_player_with_quests(fixture, guid, active, unlocked, &[])
}

/// [`zone_player_unlocked`] with `completed` quests seeded as completed with their counters,
/// so a follow-up's offer gate runs against the loader's own state.
pub fn zone_player_with_quests(
    fixture: &Fixture,
    guid: u64,
    active: i32,
    unlocked: &[i32],
    completed: &[(u32, &[u32])],
) -> Gameplay {
    let mut state = fixture.game_as(guid, None).state();
    seed_active_profile(&mut state, active, unlocked);
    for (quest, counters) in completed {
        seed_quest(&mut state, *quest, QuestStatus::Completed, counters);
    }
    fixture.game_as(guid, Some(state))
}

/// The developer `realm level <level>` command, run as an admin. This is the
/// production rank path a `job_level` goal reads; a quest never trusts a client
/// level claim.
pub fn realm_level(
    game: &mut Gameplay,
    character: &CharacterState,
    level: u32,
    now: u64,
) -> Outcome {
    game.set_dev_tools(true);
    game.set_role(AccountRole::Admin);
    send(
        game,
        character,
        &chat::local_request(&format!("realm level {level}")),
        now,
    )
}

pub fn counts(game: &Gameplay, quest_id: u32) -> Vec<u32> {
    game.state().npc_quests.as_ref().unwrap().quests[&quest_id]
        .counts
        .clone()
}

pub fn quest_status(game: &Gameplay, quest_id: u32) -> QuestStatus {
    game.state().npc_quests.as_ref().unwrap().quests[&quest_id]
        .status
        .clone()
}

pub fn pending_offer(game: &Gameplay) -> Option<u32> {
    game.state().npc_quests.as_ref().unwrap().pending_offer
}

pub fn pending_turnin(game: &Gameplay) -> Option<u32> {
    game.state().npc_quests.as_ref().unwrap().pending_turnin
}

pub fn active_job(game: &Gameplay) -> i32 {
    state_active_job(&game.state())
}

pub fn coins(game: &Gameplay) -> u32 {
    game.state().inventory.coins
}

pub fn profile_xp(game: &Gameplay, profile: i32) -> u32 {
    state_profile_xp(&game.state(), profile)
}

fn state_active_job(state: &GameplayState) -> i32 {
    state.profiles.as_ref().unwrap().active
}

fn state_profile_xp(state: &GameplayState, profile: i32) -> u32 {
    state
        .profiles
        .as_ref()
        .unwrap()
        .experience
        .get(&profile)
        .copied()
        .unwrap_or(0)
}

/// The same two reads through the gateway session snapshot battle suites assert.
pub fn gateway_active_job(gateway: &Gateway) -> i32 {
    state_active_job(
        gateway
            .snapshot()
            .gameplay
            .as_ref()
            .expect("the session has gameplay"),
    )
}

pub fn gateway_profile_xp(gateway: &Gateway, profile: i32) -> u32 {
    state_profile_xp(
        gateway
            .snapshot()
            .gameplay
            .as_ref()
            .expect("the session has gameplay"),
        profile,
    )
}

/// Seed the saved active profile when direct play cannot reach a job switch (the captured
/// opening inventory may lack that job's starter outfit). The quest gate reads this
/// production save, not the caller's expectation. Jobs the caller lists stay unlocked; the
/// caller's own fixture decides what was unlocked before.
pub fn seed_active_profile(state: &mut GameplayState, active: i32, unlocked: &[i32]) {
    let profiles = state
        .profiles
        .as_mut()
        .expect("the fixture has profile content");
    profiles.active = active;
    profiles.unlocked.insert(active);
    profiles.unlocked.extend(unlocked.iter().copied());
}

/// The serialized `GameplayState`, not the character-store save envelope. Use it to compare
/// two in-memory sessions; a save-file claim needs the real store round trip.
pub fn saved(game: &Gameplay) -> Vec<u8> {
    serde_json::to_vec(&game.state()).unwrap()
}

/// A Talk with nothing eligible must not offer, complete or mutate anything: the
/// open 26/9 menu stays usable, the 26/6 choice releases no offer or end screen,
/// and the serialized state stays equal.
pub fn assert_no_offer(
    game: &mut Gameplay,
    character: &CharacterState,
    target: u64,
    now: &mut u64,
    label: &str,
) {
    let before = saved(game);
    *now += 10;
    let menu = interact(game, character, target, *now);
    let choice = menu_option(&menu, TALK_LABEL);
    *now += 1;
    let dialogue = send(game, character, &npc_select(target, choice), *now);
    assert!(
        !dialogue.changed,
        "{label}: a Talk with nothing eligible changed gameplay"
    );
    assert!(
        !dialogue
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, None)),
        "{label}: a Talk with nothing eligible offered a quest"
    );
    // The refusal must also leave the open NPC menu intact: the same 26/9 option
    // still accepts a second Talk selection.
    *now += 1;
    let again = send(game, character, &npc_select(target, choice), *now);
    assert!(
        again
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "{label}: a refused Talk consumed the open NPC menu"
    );
    *now += 8;
    let closed = send(game, character, &dialogue_choice(QUEST_CHOICE), *now);
    assert!(
        !closed
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, None) || has_family(packet, 49, 13, None)),
        "{label}: no quest may be offered or turned in"
    );
    assert_eq!(
        saved(game),
        before,
        "{label}: the refusal changed the serialized state"
    );
}

pub fn assert_packet_changes_nothing(
    game: &mut Gameplay,
    character: &CharacterState,
    packet: &[u8],
    now: u64,
    label: &str,
) {
    let before = saved(game);
    let outcome = send(game, character, packet, now);
    assert!(!outcome.changed, "{label} changed gameplay");
    assert!(
        !outcome
            .replies
            .iter()
            .any(|reply| has_family(reply, 49, 3, None) || has_family(reply, 49, 4, None)),
        "{label} completed or added a quest"
    );
    assert_eq!(saved(game), before, "{label} changed the serialized state");
}

/// One shared-field basic attack (`SharedFieldAction::Ability` slot 0): the
/// production request `Session::commit_outcome` runs. Panics on a refused
/// request; a test that proves a refusal calls `Gameplay::handle_shared_field`
/// directly so it can inspect the `io::Result`.
pub fn shared_attack(
    game: &mut Gameplay,
    field: &mut SharedFieldState,
    character: &CharacterState,
    known: &BTreeSet<u64>,
    target: u64,
    now: u64,
) -> Outcome {
    game.handle_shared_field(
        &SharedFieldAction::Ability { slot: 0, target },
        field,
        known,
        character,
        now,
    )
    .unwrap_or_else(|error| panic!("shared attack on {target} was refused: {error}"))
}

/// The basic-attack interval the field weapon table uses.
pub const SHARED_ATTACK_INTERVAL_MS: u64 = 700;

/// Attack `target` from its current field position until its health reaches
/// zero; returns the lethal outcome and the next free timestamp. Reading the
/// position before each swing covers wandering actors and stationary ones alike;
/// `character` supplies the attacker's guid and zone.
pub fn kill_shared_target(
    game: &mut Gameplay,
    field: &mut SharedFieldState,
    known: &BTreeSet<u64>,
    character: &CharacterState,
    target: u64,
    now: u64,
) -> (Outcome, u64) {
    let mut now = now;
    for _ in 0..16 {
        let position = field
            .position(target)
            .unwrap_or_else(|| panic!("field target {target} exists"));
        let attacker = character_at_for(character.guid, character.zone_id, position);
        let outcome = shared_attack(game, field, &attacker, known, target, now);
        now += SHARED_ATTACK_INTERVAL_MS;
        if field.health(target) == Some(0) {
            return (outcome, now);
        }
    }
    panic!("field target {target} did not die");
}

/// One shared-motion tick for a participant far from the field, so the tick only
/// settles and respawns the actors. `elapsed` is the elapsed time the session
/// reports for the same tick.
pub fn tick_shared_field(
    game: &mut Gameplay,
    field: &mut SharedFieldState,
    known: &BTreeSet<u64>,
    player: u64,
    now: u64,
    elapsed: u64,
) {
    let participant = FieldParticipant {
        guid: player,
        position: [0.0, 0.0, 0.0],
        health: 1_000,
        max_health: 1_000,
        mitigation: 1.0,
        grace_until: 0,
        combat: true,
        known_npcs: Some(known.clone()),
    };
    game.tick_shared_motion(field, &[participant], now, elapsed)
        .expect("the shared field ticks");
}

/// 49/9 `ObjectiveUpdate`: quest `6..10`, objective `10..14`, current `14..18`
/// (client reader `0x00c7af60`).
pub fn objective_update(outcome: &Outcome, quest: u32, objective: u32, current: u32) -> bool {
    outcome.replies.iter().any(|packet| {
        has_family(packet, 49, 9, Some(quest))
            && packet.len() >= 18
            && u32::from_le_bytes(packet[10..14].try_into().unwrap()) == objective
            && u32::from_le_bytes(packet[14..18].try_into().unwrap()) == current
    })
}

/// 49/10 `GoalComplete`: quest `6..10`, objective `10..14` (client reader
/// `0x00c7b070`).
pub fn objective_complete(outcome: &Outcome, quest: u32, objective: u32) -> bool {
    outcome.replies.iter().any(|packet| {
        has_family(packet, 49, 10, Some(quest))
            && packet.len() >= 14
            && u32::from_le_bytes(packet[10..14].try_into().unwrap()) == objective
    })
}

/// 49/8 `ObjectiveActivated`: quest `6..10`, objective `10..14` (client reader
/// `0x00c7aeb0`).
pub fn objective_activated(outcome: &Outcome, quest: u32, objective: u32) -> bool {
    outcome.replies.iter().any(|packet| {
        has_family(packet, 49, 8, Some(quest))
            && packet.len() >= 14
            && u32::from_le_bytes(packet[10..14].try_into().unwrap()) == objective
    })
}

/// The client's click (26/8) on `guid` from `position`, after the location report and the
/// pose-driven NPC view update that puts the actor in this character's visible set.
pub fn click_from(
    game: &mut Gameplay,
    player: u64,
    guid: u64,
    zone_id: u32,
    position: [f32; 3],
    now: u64,
) -> Outcome {
    let character = character_at_for(player, zone_id, position);
    game.on_location(&character, now)
        .expect("the location report is accepted");
    game.update_npc_visibility(&character)
        .expect("the NPC view update is served");
    send(game, &character, &npc_interact(guid), now + 1)
}

/// One pickup click at `node`'s placed position in the open world; advances `now`.
pub fn pick(game: &mut Gameplay, player: u64, node: &NodeActor, now: &mut u64) -> Outcome {
    let clicked = click_from(game, player, node.guid, 0, node.position, *now);
    *now += 10;
    clicked
}

/// Whether the outcome carries a 15/3 system chat line containing `needle` (reader
/// `0x00b0f5f0`: a `u32` byte length at `4..8`, then the text).
pub fn said(outcome: &Outcome, needle: &str) -> bool {
    pickup_packets::said(&outcome.replies, needle)
}

/// Whether the outcome carries the 35/2 `AddNpc` of `guid` (reader `0x00921a10`: the guid
/// follows the four-byte header).
pub fn added(outcome: &Outcome, guid: u64) -> bool {
    pickup_packets::added(&outcome.replies, guid)
}

/// Whether the outcome carries the 35/3 removal of `guid`: the four-byte header, a `u16` at
/// `4..6`, then the guid at `6..14` (reader `0x008d69f0`, base `0x008d6830`).
pub fn removed(outcome: &Outcome, guid: u64) -> bool {
    pickup_packets::removed(&outcome.replies, guid)
}

/// Pick every spawn of one node type, asserting the counted 49/9 update and, on the last
/// pickup, the 49/10 complete and the next objective's 49/8. Only the target objective's
/// counter may move; a pickup that credits any other step fails here.
pub fn collect(
    game: &mut Gameplay,
    player: u64,
    quest: u32,
    objective: u32,
    node_type: &str,
    quantity: u32,
    next: u32,
    now: &mut u64,
) {
    let nodes = pickups(node_type);
    assert_eq!(
        nodes.len(),
        quantity as usize,
        "{node_type} must ship one spawn per required pickup"
    );
    let mut expected = counts(game, quest);
    for (index, node) in nodes.iter().enumerate() {
        let current = index as u32 + 1;
        let clicked = pick(game, player, node, now);
        assert!(
            !said(&clicked, "You don't need that right now."),
            "{node_type} pickup {current} was refused"
        );
        assert!(
            removed(&clicked, node.guid),
            "{node_type} pickup {current} must hide the node"
        );
        assert!(
            objective_update(&clicked, quest, objective, current),
            "{node_type} pickup {current} must send the counted 49/9 update"
        );
        expected[objective as usize - 1] = current;
        assert_eq!(
            counts(game, quest),
            expected,
            "{node_type} pickup {current} changed an objective counter other than {objective}"
        );
        if current == quantity {
            assert!(
                objective_complete(&clicked, quest, objective),
                "{node_type} pickup {current} must complete the step"
            );
            assert!(
                objective_activated(&clicked, quest, next),
                "objective {next} must open after {objective}"
            );
        }
    }
}
