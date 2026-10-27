//! T480: the click that completes a collection pays its captured reward once, through the real
//! 26/8 route (`Gameplay::handle_in_world`) that `collection_coin_pickup.rs` drives.
//!
//! T522: the same completing click also records the set's meta collection entry (the Elite sets
//! 284-291 tick `Elite Explorer` 309; 299-304 tick 306), and finishing all eight Elite sets pays
//! the 309 backpack 70349 once.
//!
//! Plain fixtures use the repository's `data/content` catalogue, so Seaside Rare (267: item 17300)
//! and the girls' boots set (271: 17210, girls' 15561) are the shipped rows. No coin-backed set
//! pays coins or XP, so those tests copy the bundled NPC content (spawns, NPCs, coins, catalogue)
//! into the opening package and tune 267's reward in the copied catalogue, which is also what a
//! data edit would do. The opening package defines neither girls' boots, so a synthetic definition
//! for 15561 goes in its `item-catalogue.json`; the same file defines the Elite reward items.
#[path = "common/content_copy.rs"]
mod content_copy;
#[path = "common/item_catalogue.rs"]
mod item_catalogue;

use osfr_tools::{
    character_store::CharacterState,
    chat,
    collection_catalogue::CollectionCatalogueContent,
    gameplay::{Gameplay, GameplayState, Outcome},
    identity::{Attachment, CharacterIdentity, NativeCreation},
    npc_interaction::NpcInteractRequest,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const SEASIDE_RARE: i32 = 267;
const BOOTS: i32 = 17300;
/// 271's boys' and girls' boots (`collection_catalogue.json`).
const SET_271: i32 = 271;
const BOYS: i32 = 17210;
const GIRLS: i32 = 15561;
const MEDIC: i32 = 11;
/// T522: the Elite region sets, `Elite Explorer` (309) and the backpack it pays.
const ELITE_SETS: [i32; 8] = [284, 285, 286, 287, 288, 289, 290, 291];
const ELITE_EXPLORER: i32 = 309;
const ELITE_BACKPACK: i32 = 70349;
const FULL_BAG: &str = "Your bag is full. Make room before collecting that.";
const CANNOT: &str = "You cannot collect that right now.";
static NEXT: AtomicU64 = AtomicU64::new(0);

fn bundled() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

struct Fixture {
    dir: PathBuf,
}

impl Fixture {
    /// The opening package, plus (when `tune` is given) the bundled NPC content with the catalogue
    /// rewritten by `tune`.
    fn new(tune: Option<&dyn Fn(&mut Value)>) -> Self {
        Self::build(tune, tune.is_some())
    }

    /// The opening package plus the bundled NPC content and catalogue, untouched.
    fn bundled() -> Self {
        Self::build(None, true)
    }

    fn build(tune: Option<&dyn Fn(&mut Value)>, bundled_content: bool) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "osfr-collection-rewards-{}-{}",
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
        // T522: the Elite sets' rewards (70341-70348) and the 309 backpack need definitions so
        // their completion grants are paid; 70342 also exists in the opening inventory, which wins.
        let mut items = vec![item_catalogue::sized_row(GIRLS as u32, 64)];
        items
            .extend((70341u32..=ELITE_BACKPACK as u32).map(|id| item_catalogue::sized_row(id, 64)));
        item_catalogue::write(&dir, &items);
        if bundled_content {
            content_copy::copy_tree(&bundled().join("npcs"), &dir.join("npcs"));
            for file in [
                "npc_spawns_extra.json",
                "client_areas.json",
                "collection_coins.json",
                "zones.json",
                "boomboxes.json",
            ] {
                fs::copy(bundled().join(file), dir.join(file)).unwrap();
            }
            fs::copy(
                bundled().join("collection_catalogue.json"),
                dir.join("collection_catalogue.json"),
            )
            .unwrap();
            if let Some(tune) = tune {
                let path = dir.join("collection_catalogue.json");
                let mut catalogue = json(&path);
                tune(&mut catalogue);
                fs::write(&path, serde_json::to_vec(&catalogue).unwrap()).unwrap();
            }
        }
        Self { dir }
    }

    fn load(&self, state: Option<GameplayState>) -> Gameplay {
        Gameplay::load(&self.dir, state).unwrap()
    }

    /// A girl or boy: the identity the client sends sets the gender the grant is fitted to.
    fn load_as(&self, gender: i32) -> Gameplay {
        let raw = json(&self.dir.join("inventory.json"));
        let shirt = raw["definitions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["id"] == 199)
            .expect("native creator shirt");
        let identity = CharacterIdentity {
            guid: 1,
            first_name: "Ari".into(),
            last_name: "Vale".into(),
            model: 1,
            head: "head.adr".into(),
            hair: "hair.adr".into(),
            model_customization: String::new(),
            face_paint: "none".into(),
            skin_tone: "skin".into(),
            eye_color: 262,
            hair_color: 116,
            gender,
            creation: NativeCreation {
                first_name: "Ari".into(),
                last_name: "Vale".into(),
                temporary_first_name: String::new(),
                temporary_last_name: String::new(),
                choices: [0; 21],
            },
            attachments: vec![Attachment {
                definition: 199,
                model_name: shirt["model"].as_str().unwrap().into(),
                texture_alias: shirt["texture"].as_str().unwrap().into(),
                tint_alias: shirt["tint_alias"].as_str().unwrap().into(),
                tint_id: 241,
                composite_effect_id: 0,
                slot: 3,
            }],
        };
        let mut game = self.load(None);
        assert!(game.apply_identity(&identity).unwrap());
        game
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

/// Every placed coin of a collection as (guid, position, entry id), from the shipped coin map.
fn coins_of(collection: i32) -> Vec<(u64, [f32; 3], i64)> {
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
        .map(|coin| {
            let guid = coin["spawn_guid"].as_u64().unwrap();
            (guid, position(guid), coin["entry_id"].as_i64().unwrap())
        })
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

fn click(game: &mut Gameplay, target: u64, position: [f32; 3]) -> Outcome {
    let who = at(position);
    game.update_npc_visibility(&who).unwrap();
    let request = NpcInteractRequest {
        target,
        trailing_flag: false,
    };
    game.handle_in_world(&request.encode(), &who, 1_000)
        .unwrap()
}

/// Click every coin of `collection` but the last; returns that last coin.
fn all_but_last(game: &mut Gameplay, collection: i32) -> (u64, [f32; 3]) {
    let coins = coins_of(collection);
    let (last, rest) = coins.split_last().unwrap();
    for &(guid, position, _) in rest {
        let out = click(game, guid, position);
        assert_eq!(out.replies.len(), 1, "a non-final click sends only 38/10");
    }
    (last.0, last.1)
}

fn held(state: &GameplayState, definition_id: i32) -> u32 {
    state
        .inventory
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id)
        .map(|item| item.count)
        .sum()
}

fn completed(game: &Gameplay, collection: i32) -> bool {
    game.state()
        .progression_extras
        .is_some_and(|extras| extras.collections.is_completed(collection))
}

/// What a refusal must leave alone: the whole save and the visibility revision.
fn snapshot(game: &Gameplay) -> (GameplayState, u64) {
    (game.state(), game.npc_visibility_revision())
}

/// The 50/1 reward bundle's coins and stars (`encode_display_bundle`: dwords 0 and 1 after the
/// success byte, client reader `008e7930`).
fn bundle(out: &Outcome) -> (i32, i32) {
    let packet = out
        .replies
        .iter()
        .find(|p| p.starts_with(&[50, 0, 1]))
        .expect("a reward bundle");
    let dword = |at: usize| i32::from_le_bytes(packet[at..at + 4].try_into().unwrap());
    (dword(4), dword(8))
}

/// 35/3: a u16 count, then the guid (reader `0x008d69f0`).
fn removed(packets: &[Vec<u8>], guid: u64) -> bool {
    packets
        .iter()
        .any(|p| p.starts_with(&[35, 0, 3, 0]) && p.get(6..14) == Some(&guid.to_le_bytes()[..]))
}

/// 267's reward replaced by 9 coins and 50 XP to Medic.
fn pays_coins_and_medic_xp(catalogue: &mut Value) {
    for collection in catalogue["collections"].as_array_mut().unwrap() {
        if collection["id"] == SEASIDE_RARE {
            collection["reward"] = json!({
                "coins": 9, "items": [], "xp": [{"profile_id": MEDIC, "amount": 50}],
                "tokens": [], "collection_add": [], "quest_add": []
            });
        }
    }
}

fn medic_state(fixture: &Fixture, unlocked: bool, xp: Option<u32>) -> GameplayState {
    let mut state = fixture.load(None).state();
    let profiles = state.profiles.as_mut().unwrap();
    if unlocked {
        profiles.unlocked.insert(MEDIC);
    }
    if let Some(xp) = xp {
        profiles.experience.insert(MEDIC, xp);
    }
    state
}

#[test]
fn seaside_rare_pays_its_boots_once_after_the_38_10() {
    let fixture = Fixture::new(None);
    let mut game = fixture.load(None);
    let (guid, position) = all_but_last(&mut game, SEASIDE_RARE);
    assert_eq!(held(&game.state(), BOOTS), 0);
    let out = click(&mut game, guid, position);
    assert!(out.changed && completed(&game, SEASIDE_RARE));
    assert!(out.replies[0].starts_with(&[38, 0, 10, 0]));
    let bundle_at = out.replies.iter().position(|p| p.starts_with(&[50, 0, 1]));
    assert!(bundle_at.is_some_and(|at| at > 0), "reward after the 38/10");
    assert_eq!(held(&game.state(), BOOTS), 1);
    // The save holds it.
    let relog = fixture.load(Some(game.state()));
    assert_eq!(held(&relog.state(), BOOTS), 1);
    // Replaying the last click pays nothing.
    let before = snapshot(&game);
    let replay = click(&mut game, guid, position);
    assert!(replay.replies.is_empty() && !replay.changed);
    assert_eq!(snapshot(&game), before);
}

#[test]
fn a_saved_completion_pays_nothing_again_even_when_an_entry_comes_back() {
    let fixture = Fixture::new(None);
    let (guid, position, last_entry) = *coins_of(SEASIDE_RARE).last().unwrap();
    let collected: Vec<i64> = coins_of(SEASIDE_RARE)
        .iter()
        .map(|coin| coin.2)
        .filter(|&entry| entry != last_entry)
        .collect();
    let mut state = fixture.load(None).state();
    let mut extras = state.progression_extras.take().unwrap_or_default();
    extras.collections = serde_json::from_value(json!({
        "collected": collected, "completed": [SEASIDE_RARE]
    }))
    .unwrap();
    state.progression_extras = Some(extras);
    let mut game = fixture.load(Some(state));
    let out = click(&mut game, guid, position);
    assert_eq!(
        out.replies.len(),
        1,
        "the entry records, the reward does not"
    );
    assert_eq!(held(&game.state(), BOOTS), 0);
}

#[test]
fn relog_after_completion_never_pays_again() {
    let fixture = Fixture::new(None);
    let mut game = fixture.load(None);
    let (guid, position) = all_but_last(&mut game, SEASIDE_RARE);
    assert!(click(&mut game, guid, position).changed);
    let mut relog = fixture.load(Some(game.state()));
    for (guid, position, _) in coins_of(SEASIDE_RARE) {
        let out = click(&mut relog, guid, position);
        assert!(out.replies.is_empty() && !out.changed);
    }
    assert_eq!(held(&relog.state(), BOOTS), 1);
}

#[test]
fn coins_and_xp_go_to_the_named_profile_when_unlocked() {
    let fixture = Fixture::new(Some(&pays_coins_and_medic_xp));
    let state = medic_state(&fixture, true, None);
    let coins = state.inventory.coins;
    let mut game = fixture.load(Some(state));
    let active = game.state().profiles.unwrap().active;
    assert_ne!(active, MEDIC);
    let (guid, position) = all_but_last(&mut game, SEASIDE_RARE);
    let out = click(&mut game, guid, position);
    assert_eq!(bundle(&out), (9, 50));
    let paid = game.state();
    assert_eq!(paid.inventory.coins, coins + 9);
    let profiles = paid.profiles.unwrap();
    assert_eq!(profiles.experience.get(&MEDIC), Some(&50));
    assert_eq!(profiles.active, active, "the active job is not the target");
}

#[test]
fn a_locked_profile_skips_the_xp_and_still_pays_the_coins() {
    let fixture = Fixture::new(Some(&pays_coins_and_medic_xp));
    let state = medic_state(&fixture, false, None);
    let coins = state.inventory.coins;
    let mut game = fixture.load(Some(state));
    let (guid, position) = all_but_last(&mut game, SEASIDE_RARE);
    let out = click(&mut game, guid, position);
    assert_eq!(bundle(&out), (9, 0));
    let paid = game.state();
    assert_eq!(paid.inventory.coins, coins + 9);
    assert!(paid.profiles.unwrap().experience.get(&MEDIC).is_none());
}

#[test]
fn a_girl_gets_the_girls_item_and_a_boy_the_captured_one() {
    let fixture = Fixture::new(None);
    for (gender, item) in [(2, GIRLS), (1, BOYS)] {
        let mut game = fixture.load_as(gender);
        let (guid, position) = all_but_last(&mut game, SET_271);
        assert!(click(&mut game, guid, position).changed);
        let state = game.state();
        assert_eq!(held(&state, item), 1, "gender {gender}");
        assert_eq!(held(&state, BOYS) + held(&state, GIRLS), 1, "one boot only");
    }
    // No identity yet (gender 0) gets the captured item.
    let mut game = fixture.load(None);
    let (guid, position) = all_but_last(&mut game, SET_271);
    assert!(click(&mut game, guid, position).changed);
    assert_eq!(held(&game.state(), BOYS), 1);
}

/// 100 filler stacks: the bag capacity is 100 occupied stacks.
fn full_bag(fixture: &Fixture) -> GameplayState {
    let mut state = fixture.load(None).state();
    let filler = state.inventory.items[0].clone();
    for id in 0..100 - state.inventory.items.len() as i32 {
        let mut item = filler.clone();
        item.id = 50_000 + id;
        state.inventory.items.push(item);
    }
    state
}

#[test]
fn a_full_bag_refuses_the_completing_click_and_changes_nothing() {
    let fixture = Fixture::new(None);
    let mut game = fixture.load(Some(full_bag(&fixture)));
    let (guid, position) = all_but_last(&mut game, SEASIDE_RARE);
    let before = snapshot(&game);
    for _ in 0..2 {
        let out = click(&mut game, guid, position);
        assert_eq!(out.replies, vec![chat::system(FULL_BAG)]);
        assert!(!out.changed);
        assert_eq!(snapshot(&game), before);
    }
    assert!(!completed(&game, SEASIDE_RARE));
    assert_eq!(held(&game.state(), BOOTS), 0);
    // The coin was not hidden: the next visibility update does not drop it.
    let next = game.update_npc_visibility(&at(position)).unwrap();
    assert!(!removed(&next, guid));
    // Making room (a relog with a free slot) lets the same coin pay.
    let mut state = game.state();
    state.inventory.items.pop();
    let mut relog = fixture.load(Some(state));
    let out = click(&mut relog, guid, position);
    assert!(out.changed && completed(&relog, SEASIDE_RARE));
    assert_eq!(held(&relog.state(), BOOTS), 1);
}

#[test]
fn an_xp_overflow_refuses_the_completing_click_and_changes_nothing() {
    let fixture = Fixture::new(Some(&pays_coins_and_medic_xp));
    let state = medic_state(&fixture, true, Some(u32::MAX - 10));
    let mut game = fixture.load(Some(state));
    let (guid, position) = all_but_last(&mut game, SEASIDE_RARE);
    let before = snapshot(&game);
    let out = click(&mut game, guid, position);
    assert_eq!(out.replies, vec![chat::system(CANNOT)]);
    assert!(!out.changed);
    assert_eq!(snapshot(&game), before);
    assert!(!completed(&game, SEASIDE_RARE));
    let next = game.update_npc_visibility(&at(position)).unwrap();
    assert!(!removed(&next, guid));
}

/// T522: the load-time map from a completing set to its meta collection entry, from the captured
/// `collection_add` rows.
#[test]
fn map_has_exactly_14_pairs() {
    let catalogue = CollectionCatalogueContent::load(&bundled())
        .unwrap()
        .expect("bundled collection catalogue");
    assert_eq!(catalogue.meta_pairs().len(), 14);
    for (offset, set) in ELITE_SETS.iter().enumerate() {
        assert_eq!(
            catalogue.meta_entry(*set),
            Some(2170 + offset as i32),
            "set {set}"
        );
    }
    for (offset, set) in (299..=304).enumerate() {
        assert_eq!(
            catalogue.meta_entry(set),
            Some(2150 + offset as i32),
            "set {set}"
        );
    }
    // A chain target (33 adds 34) has one set for eight entries; 295-297 all name 298, whose
    // three entries share one icon, so no target entry is accepted either.
    for source in [33, 295, 296, 297] {
        assert_eq!(catalogue.meta_entry(source), None, "source {source}");
    }
    assert!(
        catalogue
            .meta_pairs()
            .values()
            .all(|entry| ![2104, 2105, 2106].contains(entry)),
        "298's ambiguous entries are not mapped"
    );
}

/// T522: finishing 284 records 309's entry 2170 after 284's own entry, and both are saved.
#[test]
fn completing_284_records_309s_entry_2170_after_its_own() {
    let fixture = Fixture::bundled();
    let mut game = fixture.load(None);
    let coins = coins_of(284);
    let (last, rest) = coins.split_last().unwrap();
    for &(guid, position, _) in rest {
        let out = click(&mut game, guid, position);
        assert_eq!(out.replies.len(), 1, "a non-final click sends only 38/10");
    }
    let out = click(&mut game, last.0, last.1);
    let entries: Vec<u32> = out
        .replies
        .iter()
        .filter(|packet| packet.starts_with(&[38, 0, 10, 0]))
        .map(|packet| u32::from_le_bytes(packet[4..8].try_into().unwrap()))
        .collect();
    assert_eq!(entries, [last.2 as u32, 2170]);
    let meta = out
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[38, 0, 10, 0]) && packet[4..8] == 2170u32.to_le_bytes())
        .unwrap();
    // 38/10 AddEntry in reader `008d8570` order for 2170: entry, icon, tint, name, collection
    // 309, index, unknown, collected.
    let mut expected = vec![38, 0, 10, 0];
    for value in [2170u32, 2239, 0, 419_916, ELITE_EXPLORER as u32, 1, 0] {
        expected.extend(value.to_le_bytes());
    }
    expected.push(1);
    assert_eq!(meta, &expected);
    assert!(completed(&game, 284));
    assert!(!completed(&game, ELITE_EXPLORER));
    let extras = game.state().progression_extras.unwrap();
    assert!(extras.collections.is_collected(2170));
}

/// T522: finishing all eight Elite sets pays the 309 backpack once, and a relog replay pays nothing.
#[test]
fn completing_all_elite_sets_pays_the_backpack_once() {
    let fixture = Fixture::bundled();
    let mut game = fixture.load(None);
    for set in ELITE_SETS {
        for (guid, position, _) in coins_of(set) {
            assert!(click(&mut game, guid, position).changed, "set {set}");
        }
        assert!(completed(&game, set), "set {set} completes");
    }
    assert!(completed(&game, ELITE_EXPLORER));
    for item in 70341..=70348 {
        assert_eq!(held(&game.state(), item), 1, "item {item}");
    }
    assert_eq!(held(&game.state(), ELITE_BACKPACK), 1);
    let mut relog = fixture.load(Some(game.state()));
    assert_eq!(held(&relog.state(), ELITE_BACKPACK), 1);
    for set in ELITE_SETS {
        for (guid, position, _) in coins_of(set) {
            let out = click(&mut relog, guid, position);
            assert!(
                out.replies.is_empty() && !out.changed,
                "set {set} replay is silent"
            );
        }
    }
    assert_eq!(held(&relog.state(), ELITE_BACKPACK), 1);
}

/// T522/F694: at 99 stacks the eighth set's own 70348 fits, so its reward sets `changed` before
/// 309's 70349 finds no room. The refusal must discard that staged change: the click records
/// nothing, hides nothing and saves nothing; with one free slot the same click pays both once.
#[test]
fn a_meta_reward_refusal_changes_nothing_and_the_retry_pays_once() {
    let fixture = Fixture::bundled();
    let mut game = fixture.load(None);
    for set in &ELITE_SETS[..7] {
        for (guid, position, _) in coins_of(*set) {
            assert!(click(&mut game, guid, position).changed, "set {set}");
        }
        assert!(completed(&game, *set), "set {set} completes");
    }
    // 99 stacks: the eighth set's 70348 is the hundredth, and 309's 70349 is the 101st.
    let mut state = game.state();
    let filler = state.inventory.items[0].clone();
    for id in 0..99 - state.inventory.items.len() as i32 {
        let mut item = filler.clone();
        item.id = 60_000 + id;
        state.inventory.items.push(item);
    }
    state.inventory.next_instance_id = Some(70_000); // above the filler ids
    assert_eq!(state.inventory.items.len(), 99);
    let mut game = fixture.load(Some(state));
    let coins = coins_of(ELITE_SETS[7]);
    let (last, rest) = coins.split_last().unwrap();
    for &(guid, position, _) in rest {
        assert!(click(&mut game, guid, position).changed);
    }
    let before = snapshot(&game);
    let out = click(&mut game, last.0, last.1);
    assert_eq!(out.replies, vec![chat::system(FULL_BAG)]);
    assert!(
        !out.changed,
        "a refused meta reward must not mark the save changed"
    );
    assert_eq!(snapshot(&game), before);
    assert!(!completed(&game, ELITE_SETS[7]));
    assert!(!completed(&game, ELITE_EXPLORER));
    let extras = game.state().progression_extras.unwrap();
    assert!(
        !extras.collections.is_collected(last.2 as i32),
        "the completing source entry is not recorded"
    );
    assert!(
        !extras.collections.is_collected(2177),
        "309's entry is not recorded"
    );
    assert_eq!(held(&game.state(), 70348), 0);
    assert_eq!(held(&game.state(), ELITE_BACKPACK), 0);
    // The coin was not hidden: the next visibility update does not drop it.
    let next = game.update_npc_visibility(&at(last.1)).unwrap();
    assert!(!removed(&next, last.0));
    // One free slot: the same click records both entries and pays both rewards once.
    let mut state = game.state();
    state.inventory.items.pop();
    let mut relog = fixture.load(Some(state));
    let out = click(&mut relog, last.0, last.1);
    assert!(out.changed && completed(&relog, ELITE_SETS[7]) && completed(&relog, ELITE_EXPLORER));
    assert_eq!(held(&relog.state(), 70348), 1);
    assert_eq!(held(&relog.state(), ELITE_BACKPACK), 1);
    let replay = click(&mut relog, last.0, last.1);
    assert!(replay.replies.is_empty() && !replay.changed);
    assert_eq!(held(&relog.state(), ELITE_BACKPACK), 1);
}

/// T522: a full bag refuses 284's completing click before either ledger entry is recorded.
#[test]
fn a_full_bag_completing_elite_click_leaves_309_untouched() {
    let fixture = Fixture::bundled();
    let mut game = fixture.load(Some(full_bag(&fixture)));
    let coins = coins_of(284);
    let (last, rest) = coins.split_last().unwrap();
    for &(guid, position, _) in rest {
        assert!(click(&mut game, guid, position).changed);
    }
    let before = snapshot(&game);
    for _ in 0..2 {
        let out = click(&mut game, last.0, last.1);
        assert_eq!(out.replies, vec![chat::system(FULL_BAG)]);
        assert!(!out.changed);
        assert_eq!(snapshot(&game), before);
    }
    assert!(!completed(&game, 284));
    let extras = game.state().progression_extras.unwrap();
    assert!(!extras.collections.is_collected(2170));
    assert!(!extras.collections.is_completed(ELITE_EXPLORER));
}
