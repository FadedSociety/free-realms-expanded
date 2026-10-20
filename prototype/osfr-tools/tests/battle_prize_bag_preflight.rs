//! T243: a battle start must fit the whole previewed prize bundle in the bag.
//!
//! The frames here are the production routes: click a battle marker (26/8 ->
//! 41/114 start screen), Go (41/108, client constructor `0x008b6e70` and sender
//! `0x00938f00`), activity-portal Play (167/1/2), and results Close (26/42,
//! constructor `0x008a2eb0`). The refusal under test is the existing
//! unavailable-prize chat line and the re-opened start screen; no wire byte
//! changed, so the packet shapes stay the ones `field_battle_roster` pins.
//!
//! Battle 12310 keeps its shipped spawner, arena and enemy rows; only the
//! test copy's prize rows are replaced, because shipped prizes are uncapped.
//! `data/content/field_battles.json` itself is never edited.

use osfr_tools::{
    account_store::AccountRole,
    chat,
    encounter::{EncounterEngine, MAX_PRIZE_QUANTITY},
    gameplay::GameplayState,
    gateway::{Gateway, GatewayOutcome},
    inventory::{Inventory, InventoryItem, InventoryState, MAX_ITEMS},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};

const RUNTIME_CONTENT: &str = "../../../lab/reports/round70-native-20261016T061000Z/content";
/// The shipped battle whose prize rows the synthetic bundle replaces.
const BATTLE: u32 = 12310;
/// Brawler, the combat job the opening gives every player.
const BRAWLER: i32 = 43;
/// The existing server refusal line; the test asserts this wording, not a new
/// refusal.
const REFUSAL: &str = "one of its rewards cannot be granted";

/// Which synthetic prize bundle the shared content copy carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Bundle {
    /// The same bounded item twice, each row a full stack: two new stacks.
    TwoFullStacks,
    /// The same bounded item twice at one each: together they overflow the
    /// existing stack and need a second one.
    TwoSingles,
}

// ---------------------------------------------------------------------------
// Fixture: the runtime content package plus one synthetic battle prize bundle.
// ---------------------------------------------------------------------------

struct Fixture {
    content: PathBuf,
    reference: PathBuf,
    version: String,
}

impl Fixture {
    fn new(bundle: Bundle) -> Self {
        let reference = manifest_dir().join("../../../lab/references/rust-gateway-reference");
        let version = read_json(&reference.join("manifest.json"))["client_version"]
            .as_str()
            .unwrap()
            .to_owned();
        Self {
            content: shared_content(bundle),
            reference,
            version,
        }
    }

    fn login(&self, gateway: &mut Gateway) {
        let mut login = 1_u16.to_le_bytes().to_vec();
        for (index, value) in ["local-private-test", self.version.as_str(), ""]
            .into_iter()
            .enumerate()
        {
            login.extend((value.len() as u32).to_le_bytes());
            login.extend(value.as_bytes());
            if index == 0 {
                login.extend(gateway.state().guid.to_le_bytes());
            }
        }
        gateway.handle_at(&login, 0).unwrap();
    }

    fn reference_guid(&self) -> u64 {
        Gateway::reference_state(&self.reference).unwrap().guid
    }

    /// A logged-in admin session at `position` with the synthetic battle loaded
    /// and the combat job active.
    fn seed(&self, position: [f32; 3], guid: u64) -> Gateway {
        let mut character = Gateway::reference_state(&self.reference).unwrap();
        character.guid = guid;
        character.zone_id = 0;
        character.position = position;
        character.rotation = [0.0, 0.0, 1.0];
        let mut gateway =
            Gateway::load_gameplay(&self.reference, character, &self.content, None).unwrap();
        gateway.set_dev_tools(true);
        gateway.set_role(AccountRole::Admin);
        self.login(&mut gateway);
        send(&mut gateway, &13_u16.to_le_bytes(), 100);
        send(&mut gateway, &10_u16.to_le_bytes(), 200);
        gateway
    }

    /// The seed session with an exactly `capacity`-stacks bag, saved through the
    /// production `Gateway::load_gameplay` restore path (no test seam).
    ///
    /// Switching the combat job happens before the bag is built so no outfit
    /// packet can move the stack count the test just set.
    fn full_bag_gateway(
        &self,
        position: [f32; 3],
        guid: u64,
        capacity: usize,
        held: Option<(i32, u32)>,
    ) -> (Gateway, InventoryState) {
        let mut seed = self.seed(position, guid);
        let switched = send(&mut seed, &switch_job(BRAWLER), 300);
        assert!(switched.state_changed, "the combat job is unlocked");

        let mut snapshot = seed.snapshot();
        let bag = full_bag(
            &snapshot.gameplay.as_ref().unwrap().inventory,
            capacity,
            held,
        );
        snapshot.gameplay.as_mut().unwrap().inventory = bag.clone();
        let gameplay: GameplayState = snapshot.gameplay.unwrap();

        let mut gateway = Gateway::load_gameplay(
            &self.reference,
            seed.state().clone(),
            &self.content,
            Some(gameplay),
        )
        .unwrap();
        gateway.set_dev_tools(true);
        gateway.set_role(AccountRole::Admin);
        self.login(&mut gateway);
        send(&mut gateway, &13_u16.to_le_bytes(), 400);
        let loaded = send(&mut gateway, &10_u16.to_le_bytes(), 500);
        assert!(loaded.entered_world);
        assert_eq!(
            gateway.snapshot().gameplay.unwrap().inventory,
            bag,
            "world entry restores the test bag unchanged"
        );
        (gateway, bag)
    }
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn runtime_content() -> PathBuf {
    manifest_dir().join(RUNTIME_CONTENT)
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn hard_link_or_copy(source: &Path, destination: &Path) {
    if fs::hard_link(source, destination).is_err() {
        fs::copy(source, destination).unwrap();
    }
}

/// `field_battles.json` with battle 12310's prize rows replaced. The rest of
/// the shipped row (spawner, arena, enemies, coins) is untouched.
fn synthetic_battles(bundle: Bundle) -> Value {
    let (item, max_stack) = bounded_item();
    let quantities: &[u32] = match bundle {
        Bundle::TwoFullStacks => &[max_stack, max_stack],
        Bundle::TwoSingles => &[1, 1],
    };
    let mut file = read_json(&manifest_dir().join("data/content/field_battles.json"));
    let row = file["battles"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["id"] == BATTLE)
        .expect("battle 12310 is shipped");
    let name_id = row["prizes"][0]["name_id"].clone();
    let icon_id = row["prizes"][0]["icon_id"].clone();
    row["prizes"] = Value::Array(
        quantities
            .iter()
            .map(|quantity| {
                json!({
                    "item_id": item,
                    "name_id": name_id.clone(),
                    "icon_id": icon_id.clone(),
                    "quantity": quantity,
                })
            })
            .collect(),
    );
    file
}

/// One content directory per bundle, shared by every test using it. The
/// runtime package is hard-linked read-only; only our own copy is written.
fn shared_content(bundle: Bundle) -> PathBuf {
    static CONTENT: OnceLock<Mutex<BTreeMap<Bundle, PathBuf>>> = OnceLock::new();
    let mut cache = CONTENT
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
        .unwrap();
    if let Some(path) = cache.get(&bundle) {
        return path.clone();
    }
    // A concurrent test binary shares this build dir: the process id and a
    // per-process serial keep every process's copy private, so one process can
    // never rebuild the content another process is loading.
    static SERIAL: AtomicU64 = AtomicU64::new(0);
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "t243-prize-bag-{}-{}-{bundle:?}",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    // Rebuild only this build-dir scratch directory; the runtime content is
    // never written or removed.
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("npcs")).unwrap();
    for entry in fs::read_dir(runtime_content()).unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_file() || entry.file_name() == "zones.json" {
            continue;
        }
        hard_link_or_copy(&entry.path(), &root.join(entry.file_name()));
    }
    fs::write(
        root.join("zones.json"),
        json!({
            "schema_version": 1,
            "records": [{"id": 0, "name": "SyntheticWorld"}]
        })
        .to_string(),
    )
    .unwrap();
    fs::write(
        root.join("npcs/synthetic.json"),
        json!({
            "schema_version": 1,
            "records": [{
                "npc_id": 1,
                "name": "Synthetic NPC",
                "zone_id": 0,
                "position": [0.0, 0.0, 0.0],
                "placement": "legacy_data",
                "model_id": 1
            }]
        })
        .to_string(),
    )
    .unwrap();
    fs::write(
        root.join("npc_spawns_extra.json"),
        json!({"schema_version": 1, "records": []}).to_string(),
    )
    .unwrap();
    let battle_file = root.join("field_battles.json");
    fs::write(
        &battle_file,
        serde_json::to_vec(&synthetic_battles(bundle)).unwrap(),
    )
    .unwrap();
    // Fail at the fixture, not at a later refusal, if the synthetic prize row
    // is not content the battle loader would accept.
    let mut engine = EncounterEngine::load(&root, None).unwrap();
    engine
        .load_field_battles(&battle_file)
        .expect("the synthetic prize rows pass content validation");
    cache.insert(bundle, root.clone());
    root
}

// ---------------------------------------------------------------------------
// Catalogue-owned reward facts: the bounded item, its stack, a filler, a bag.
// ---------------------------------------------------------------------------

fn runtime_inventory() -> &'static Inventory {
    static RUNTIME: OnceLock<Inventory> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        Inventory::load(&runtime_content(), None).expect("the round70 runtime item catalogue loads")
    })
}

/// A real catalogue item with a bounded stack of 2..=MAX_PRIZE_QUANTITY that
/// the initial bag does not own. Shipped battle prizes are uncapped, so the
/// combined-capacity regression has to introduce a bounded row; the item, its
/// stack and the prize ceiling all come from their owners, never a literal id.
fn bounded_item() -> (i32, u32) {
    static BOUNDED: OnceLock<(i32, u32)> = OnceLock::new();
    *BOUNDED.get_or_init(|| {
        let inventory = runtime_inventory();
        let owned: BTreeSet<i32> = inventory
            .state()
            .items
            .iter()
            .map(|item| item.definition_id)
            .collect();
        inventory
            .definition_ids()
            .find_map(|id| {
                let (_, max_stack) = inventory.trade_flags(id)?;
                let max_stack = max_stack?;
                (!owned.contains(&id) && (2..=MAX_PRIZE_QUANTITY).contains(&max_stack))
                    .then_some((id, max_stack))
            })
            .expect("the runtime catalogue has a bounded, unowned item")
    })
}

/// A real uncapped catalogue row that is not the prize item: each filler stack
/// stays a separate stack and never merges with a granted prize.
fn filler_definition() -> i32 {
    static FILLER: OnceLock<i32> = OnceLock::new();
    *FILLER.get_or_init(|| {
        let (prize, _) = bounded_item();
        let inventory = runtime_inventory();
        inventory
            .definition_ids()
            .find(|id| {
                *id != prize
                    && inventory
                        .trade_flags(*id)
                        .is_some_and(|(_, max_stack)| max_stack.is_none())
            })
            .expect("the runtime catalogue has an uncapped item")
    })
}

fn stack(id: i32, definition_id: i32, count: u32) -> InventoryItem {
    InventoryItem {
        definition_id,
        tint: 0,
        id,
        count,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: i64::from(i32::MAX),
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    }
}

/// A bag with exactly `capacity` occupied stacks: an optional partial bounded
/// stack, then unique filler stacks. The instance counter stays valid.
fn full_bag(base: &InventoryState, capacity: usize, held: Option<(i32, u32)>) -> InventoryState {
    assert!(
        base.items.len() <= capacity,
        "the initial bag ({} stacks) is smaller than the target capacity {capacity}",
        base.items.len()
    );
    let mut state = base.clone();
    let mut id = state.items.iter().map(|item| item.id).max().unwrap_or(0) + 1;
    if let Some((definition, count)) = held {
        state.items.push(stack(id, definition, count));
        id += 1;
    }
    let filler = filler_definition();
    while state.items.len() < capacity {
        state.items.push(stack(id, filler, 1));
        id += 1;
    }
    state.next_instance_id = Some(i64::from(id));
    state
}

// ---------------------------------------------------------------------------
// Packet helpers (shapes pinned by `field_battle_roster`).
// ---------------------------------------------------------------------------

fn battle_row() -> Value {
    let file = read_json(&manifest_dir().join("data/content/field_battles.json"));
    file["battles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == BATTLE)
        .unwrap()
        .clone()
}

fn spawner_guid() -> u64 {
    battle_row()["spawner"]["guid"].as_u64().unwrap()
}

fn marker_position() -> [f32; 3] {
    let points = battle_row()["spawner"]["position"]
        .as_array()
        .unwrap()
        .clone();
    [
        points[0].as_f64().unwrap() as f32,
        points[1].as_f64().unwrap() as f32,
        points[2].as_f64().unwrap() as f32,
    ]
}

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

fn unframe(reply: &[u8]) -> &[u8] {
    if reply.starts_with(&[5, 0, 1]) && reply.len() >= 7 {
        let length = u32::from_le_bytes(reply[3..7].try_into().unwrap()) as usize;
        if length == reply.len() - 7 {
            return &reply[7..];
        }
    }
    reply
}

fn send(gateway: &mut Gateway, inner: &[u8], now_ms: u64) -> GatewayOutcome {
    gateway
        .handle_at(&wrap(inner), now_ms)
        .expect("gateway frame")
}

fn interact(guid: u64) -> Vec<u8> {
    [vec![26, 0, 8, 0], guid.to_le_bytes().to_vec(), vec![0]].concat()
}

fn switch_job(profile: i32) -> Vec<u8> {
    [vec![26, 0, 13, 0], profile.to_le_bytes().to_vec()].concat()
}

/// 41/108 Go: the start screen's encounter and instance, then the session
/// field the native sender writes as zero (`0x008b6e70`, `0x00938f00`).
fn go(details: &[u8]) -> Vec<u8> {
    let mut packet = vec![41, 0, 108, 0];
    packet.extend(&details[4..12]);
    packet.extend(0_u64.to_le_bytes());
    packet
}

/// A Go naming the battle but not taken from an open start screen.
fn go_from_id(battle: u32) -> Vec<u8> {
    let mut details = vec![0_u8; 12];
    details[4..8].copy_from_slice(&(battle as i32).to_le_bytes());
    go(&details)
}

fn portal_play(activity: u32) -> Vec<u8> {
    let mut inner = vec![167, 0, 1, 2];
    inner.extend(activity.to_le_bytes());
    inner.extend(0_u32.to_le_bytes());
    inner
}

fn chat_text(outcome: &GatewayOutcome) -> String {
    outcome
        .replies
        .iter()
        .map(|packet| unframe(packet))
        .filter(|packet| packet.starts_with(&[15, 0, 3, 0]) && packet.len() >= 8)
        .map(|packet| {
            let length = u32::from_le_bytes(packet[4..8].try_into().unwrap()) as usize;
            String::from_utf8_lossy(&packet[8..8 + length]).into_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn details(outcome: &GatewayOutcome) -> Option<&[u8]> {
    outcome
        .replies
        .iter()
        .map(|packet| unframe(packet))
        .find(|packet| packet.starts_with(&[41, 0, 114, 0]))
}

fn arena_spawned(gateway: &mut Gateway, now_ms: u64) -> GatewayOutcome {
    let mut outcome = send(gateway, &13_u16.to_le_bytes(), now_ms);
    outcome.replies.extend(
        send(gateway, &10_u16.to_le_bytes(), now_ms + 100)
            .replies
            .into_iter(),
    );
    outcome
}

fn open_start_screen(gateway: &mut Gateway, now_ms: u64) -> Vec<u8> {
    send(gateway, &interact(spawner_guid()), now_ms)
        .replies
        .iter()
        .map(|packet| unframe(packet))
        .find(|packet| packet.starts_with(&[41, 0, 114, 0]))
        .expect("clicking the marker opens 41/114")
        .to_vec()
}

fn economy(gateway: &Gateway, definition: i32) -> (u32, u32, usize) {
    let state = gateway.snapshot().gameplay.unwrap();
    let mut count = 0;
    let mut stacks = 0;
    for item in state
        .inventory
        .items
        .iter()
        .filter(|item| item.definition_id == definition)
    {
        count += item.count;
        stacks += 1;
    }
    (state.inventory.coins, count, stacks)
}

fn encounter_active(gateway: &Gateway) -> bool {
    gateway
        .snapshot()
        .gameplay
        .unwrap()
        .encounter
        .unwrap()
        .active
        .is_some()
}

/// Every field this session contributes to a save, byte for byte. The save
/// envelope only adds formatting and separate housing/pet rows; equal bytes
/// prove equal saved state.
fn saved_bytes(gateway: &Gateway) -> Vec<u8> {
    serde_json::to_vec(&gateway.snapshot()).unwrap()
}

// ---------------------------------------------------------------------------
// Gates.
// ---------------------------------------------------------------------------

/// Test 1: two full stacks of one bounded item need two new stacks; the bag
/// has room for one. Every shipped row passes the single-row check, so the
/// refusal proves the combined bundle is what is refused.
#[test]
fn the_whole_previewed_bundle_must_fit_before_the_battle_starts() {
    let (item, max_stack) = bounded_item();
    let fixture = Fixture::new(Bundle::TwoFullStacks);
    let (mut gateway, bag) = fixture.full_bag_gateway(
        marker_position(),
        fixture.reference_guid(),
        MAX_ITEMS - 1,
        None,
    );

    assert_eq!(bag.items.len(), MAX_ITEMS - 1);
    assert!(
        !bag.items.iter().any(|row| row.definition_id == item),
        "the prize item starts unowned so the bundle needs two new stacks"
    );
    // The single-row catalogue/stack check every shipped row already passes.
    assert!(runtime_inventory().accepts_reward(item, max_stack));
    // One row fits the free stack; the bundle needs a second one.
    let live = Inventory::load(&fixture.content, Some(bag)).unwrap();
    assert!(live.accepts_reward_bundle(&[(item, max_stack)]));
    assert!(!live.accepts_reward_bundle(&[(item, max_stack), (item, max_stack)]));

    let before = saved_bytes(&gateway);
    let refused = send(&mut gateway, &portal_play(BATTLE), 600);
    assert!(!refused.began_zoning, "a refused start zones nowhere");
    assert!(!refused.state_changed);
    assert_eq!(
        refused.replies.len(),
        1,
        "the unavailable-prize line is the only output"
    );
    assert!(
        chat_text(&refused).contains(REFUSAL),
        "{}",
        chat_text(&refused)
    );
    assert!(!encounter_active(&gateway), "no encounter began");
    assert_eq!(saved_bytes(&gateway), before, "a refusal saves nothing");

    // The portal refusal leaves no start screen pending: a Go naming the battle
    // falls through to the encounter handler and answers nothing.
    let inert = send(&mut gateway, &go_from_id(BATTLE), 700);
    assert!(!inert.began_zoning && !inert.state_changed && inert.replies.is_empty());
    assert!(!encounter_active(&gateway));
    assert_eq!(saved_bytes(&gateway), before);
}

/// Test 2: the same bounded item twice. Each row alone would merge into the one
/// partial stack; together they need a second stack, and the bag is full.
#[test]
fn duplicate_prizes_are_counted_together() {
    let (item, max_stack) = bounded_item();
    let fixture = Fixture::new(Bundle::TwoSingles);
    let (mut gateway, bag) = fixture.full_bag_gateway(
        marker_position(),
        fixture.reference_guid(),
        MAX_ITEMS,
        Some((item, max_stack - 1)),
    );

    assert_eq!(bag.items.len(), MAX_ITEMS);
    assert_eq!(
        bag.items
            .iter()
            .filter(|row| row.definition_id == item)
            .count(),
        1,
        "the held partial stack is the only stack of the prize item"
    );
    let live = Inventory::load(&fixture.content, Some(bag)).unwrap();
    assert!(live.accepts_reward(item, 1), "each row passes alone");
    assert!(
        live.accepts_reward_bundle(&[(item, 1)]),
        "one row merges into the partial stack"
    );
    assert!(
        !live.accepts_reward_bundle(&[(item, 1), (item, 1)]),
        "both rows together overflow the stack and the bag"
    );

    let before = saved_bytes(&gateway);
    let refused = send(&mut gateway, &portal_play(BATTLE), 800);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(
        chat_text(&refused).contains(REFUSAL),
        "{}",
        chat_text(&refused)
    );
    assert!(!encounter_active(&gateway));
    assert_eq!(saved_bytes(&gateway), before);
}

/// Test 3: the same bundle on a bag with two free stacks. The start succeeds,
/// the arena spawns, the win pays exactly one previewed prize, and nothing pays
/// twice.
#[test]
fn a_bundle_that_fits_still_starts_and_pays() {
    let (item, max_stack) = bounded_item();
    let fixture = Fixture::new(Bundle::TwoFullStacks);
    let (mut gateway, bag) = fixture.full_bag_gateway(
        marker_position(),
        fixture.reference_guid(),
        MAX_ITEMS - 2,
        None,
    );
    assert_eq!(bag.items.len(), MAX_ITEMS - 2);
    assert!(!bag.items.iter().any(|row| row.definition_id == item));

    let screen = open_start_screen(&mut gateway, 900);
    let started = send(&mut gateway, &go(&screen), 1_000);
    assert!(started.began_zoning, "the fitting bundle starts the battle");
    assert_eq!(gateway.state().zone_id, 9600);
    arena_spawned(&mut gateway, 1_100);

    let reward_coins = u64::from(battle_row()["reward_coins"].as_u64().unwrap()) as u32;
    let (coins_before, count_before, stacks_before) = economy(&gateway, item);
    assert_eq!((count_before, stacks_before), (0, 0));

    send(&mut gateway, &chat::local_request("realm win"), 1_200);
    let (coins_paid, count_paid, stacks_paid) = economy(&gateway, item);
    assert_eq!(
        count_paid, max_stack,
        "exactly one previewed prize was paid"
    );
    assert_eq!(stacks_paid, 1, "one new stack pays the prize");
    assert_eq!(coins_paid, coins_before + reward_coins);

    let won = saved_bytes(&gateway);
    let repeated = send(&mut gateway, &chat::local_request("realm win"), 1_300);
    assert!(!repeated.state_changed);
    assert_eq!(saved_bytes(&gateway), won, "a won battle cannot pay twice");

    let closed = send(&mut gateway, &[26, 0, 42, 0], 1_400);
    assert!(closed.began_zoning, "Close returns to the open world");
    assert_eq!(gateway.state().zone_id, 0);
    assert_eq!(gateway.state().position, marker_position());
    assert_eq!(
        economy(&gateway, item),
        (coins_paid, count_paid, stacks_paid),
        "leaving the results pays nothing more"
    );
}

/// Test 4: invalid bundles are refused without a panic and without touching the
/// live bag. The positive control shows the same method accepts a fitting row.
#[test]
fn an_invalid_bundle_is_refused_without_a_panic() {
    let (item, max_stack) = bounded_item();
    let inventory = runtime_inventory();
    let before = serde_json::to_vec(inventory.state()).unwrap();
    assert!(inventory.accepts_reward_bundle(&[]), "an empty bundle fits");
    assert!(inventory.accepts_reward_bundle(&[(item, 1)]));

    let too_long: Vec<(i32, u32)> = (0..17).map(|_| (item, 1)).collect();
    for (case, bundle) in [
        ("zero definition", vec![(0, 1)]),
        ("negative definition", vec![(-1, 1)]),
        ("malformed row", vec![(i32::MIN, u32::MAX)]),
        ("zero quantity", vec![(item, 0)]),
        ("malformed quantity", vec![(item, u32::MAX)]),
        ("unknown definition", vec![(i32::MAX, 1)]),
        ("more than 16 rows", too_long),
        (
            "valid row after two invalid ones",
            vec![(0, 1), (item, 0), (item, max_stack)],
        ),
    ] {
        assert!(
            !inventory.accepts_reward_bundle(&bundle),
            "{case} must be refused"
        );
        assert_eq!(
            serde_json::to_vec(inventory.state()).unwrap(),
            before,
            "{case} must not touch the live bag"
        );
    }
}

/// Test 5: a refused start screen changes no inventory, coin, profile or saved
/// state, and a second player's refusal leaves the first player's state alone.
#[test]
fn a_refused_preflight_changes_nothing() {
    let fixture = Fixture::new(Bundle::TwoFullStacks);
    let (item, _) = bounded_item();
    let marker = marker_position();
    let first_guid = fixture.reference_guid();
    let (mut first, _) = fixture.full_bag_gateway(marker, first_guid, MAX_ITEMS - 1, None);

    let first_before = saved_bytes(&first);
    let first_economy = economy(&first, item);
    let screen = open_start_screen(&mut first, 900);
    let refused = send(&mut first, &go(&screen), 1_000);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(
        chat_text(&refused).contains(REFUSAL),
        "{}",
        chat_text(&refused)
    );
    assert_eq!(
        details(&refused).expect("the refused start screen reopens"),
        screen.as_slice(),
        "the same screen stays open"
    );
    assert!(!encounter_active(&first));
    assert_eq!(
        saved_bytes(&first),
        first_before,
        "inventory, coins, profiles and save are byte-identical after the refusal"
    );

    // The screen is still pending, so a replay refuses identically: the attempt
    // consumed neither the screen nor any claimed state.
    let repeated = send(&mut first, &go(&screen), 1_100);
    assert!(!repeated.began_zoning && !repeated.state_changed);
    assert!(
        chat_text(&repeated).contains(REFUSAL),
        "{}",
        chat_text(&repeated)
    );
    assert_eq!(
        details(&repeated).expect("the screen stays open"),
        screen.as_slice()
    );
    assert_eq!(saved_bytes(&first), first_before);

    // A different character with the same full bag is refused independently.
    let (mut second, _) = fixture.full_bag_gateway(marker, first_guid + 1, MAX_ITEMS - 1, None);
    let second_before = saved_bytes(&second);
    let second_screen = open_start_screen(&mut second, 1_200);
    let second_refused = send(&mut second, &go(&second_screen), 1_300);
    assert!(!second_refused.began_zoning && !second_refused.state_changed);
    assert!(
        chat_text(&second_refused).contains(REFUSAL),
        "{}",
        chat_text(&second_refused)
    );
    assert!(!encounter_active(&second));
    assert_eq!(saved_bytes(&second), second_before);
    assert_eq!(
        saved_bytes(&first),
        first_before,
        "another player's refusal leaves the first player untouched"
    );
    assert_eq!(
        economy(&first, item),
        first_economy,
        "the first player's coins, prize count and stacks are unchanged"
    );
}

/// Test 6: the bundle preflight must answer with `grant_bundle`'s own outcome,
/// not a second copy of its loop. Every bundle the payout path would commit the
/// preflight accepts, and every bundle it would commit nothing for the
/// preflight refuses, so a rule added to `grant_bundle` reaches the start gate
/// and the payout together. Bundles above the loader's 16-row bound are the one
/// deliberate difference (`grant_bundle` has no row cap) and are pinned above.
#[test]
fn the_preflight_answers_like_the_payout_path() {
    let (item, max_stack) = bounded_item();
    let filler = filler_definition();
    let inventory = runtime_inventory();
    let before = serde_json::to_vec(inventory.state()).unwrap();
    for bundle in [
        vec![],
        vec![(item, 1)],
        vec![(item, max_stack)],
        vec![(item, max_stack), (item, 1)],
        vec![(filler, 1), (item, 1)],
        vec![(item, max_stack + 1)],
        vec![(item, 0)],
        vec![(0, 1)],
        vec![(i32::MAX, 1)],
    ] {
        let accepted = inventory.accepts_reward_bundle(&bundle);
        let paid = inventory.clone().grant_bundle(0, &bundle).is_ok();
        assert_eq!(
            accepted, paid,
            "the start preflight and the payout disagree for {bundle:?}"
        );
    }
    // The one deliberate difference is the loader row bound: `grant_bundle`
    // would pay seventeen valid rows, but no encounter can preview that bundle
    // (test 4 pins the 16-row refusal).
    let long: Vec<(i32, u32)> = (0..17).map(|_| (filler, 1)).collect();
    assert!(!inventory.accepts_reward_bundle(&long));
    assert!(inventory.clone().grant_bundle(0, &long).is_ok());
    assert_eq!(
        serde_json::to_vec(inventory.state()).unwrap(),
        before,
        "neither check changes the live bag"
    );
}
