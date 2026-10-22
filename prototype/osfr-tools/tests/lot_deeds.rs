//! L046: buying a lot deed in the Station Cash Marketplace grants the lot itself. The purchase
//! is driven through the real `Gameplay` route (66/3 order -> Marketplace -> housing session),
//! the content gate reads the shipped `houses.json`/`stores.json` against the item catalogue, and
//! every refusal is checked to change nothing. Raw inner packets as the client sends them; no
//! player data.
use osfr_tools::{
    character_store::CharacterState,
    coin_shop::{CoinShop, StoreCatalogue},
    game_store::{self, Snapshot},
    gameplay::{Gameplay, GameplayState},
    housing::HousingError,
    housing_native::{InstanceList, NotifyHouseAdded},
    housing_service::{FixtureConfig, HousingContent, HousingReply, HousingService, ServiceError},
    inventory::Inventory,
    station_cash::{self, DEED_ITEM_TYPE, Wallet},
};
use std::{
    fs,
    path::PathBuf,
    sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};

#[path = "common/item_catalogue.rs"]
#[allow(dead_code)]
mod item_catalogue;

const OWNER: u64 = 1;
const STORE: u32 = 1;
/// Bog Shore Lot: houses.json property 3, deed 17192 (bundle 4701), wiki 400 SC -> 100 SC.
const BOG_SHORE_LOT: i32 = 3;
const BOG_SHORE_DEED: i32 = 17_192;
const BOG_SHORE_NAME: i32 = 69_974;
const BOG_SHORE_PRICE: u32 = 100;
/// Large Wilds House: property 26 is disabled (no scene), deed 2214.
const DISABLED_DEED: i32 = 2_214;
/// Wilds Condo: the starter (property 25), deed 17184. Free through 127/23 with its kit (L040).
const CONDO: i32 = 25;
const CONDO_DEED: i32 = 17_184;
/// The L040 starter kit's bag unit (Chicken) and its standing fixtures (houses.json `starter_kit`).
const CHICKEN: i32 = 70_429;
const KIT_FIXTURES: usize = 4;
static NEXT: AtomicU64 = AtomicU64::new(0);

fn round70() -> PathBuf {
    std::env::var_os("OSFR_STATION_CASH_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/reports/round70-native-20261016T061000Z/content")
        })
}

/// The 69 MB catalogue is parsed once per test process.
fn round70_inventory() -> Inventory {
    static ONCE: OnceLock<Inventory> = OnceLock::new();
    ONCE.get_or_init(|| Inventory::load(&round70(), None).expect("item content loads"))
        .clone()
}

fn stores() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/stores.json")
}

fn put_string(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend((value.len() as i32).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

/// The client order serializer `00b3fc60`/`00b3af80` (66/3), with one detail.
fn order(tracking: u32, bundle: i32) -> Vec<u8> {
    request(3, tracking, bundle)
}

/// The 66/1 preview carries the same body (its trailing string is optional).
fn preview(tracking: u32, bundle: i32) -> Vec<u8> {
    request(1, tracking, bundle)
}

fn request(opcode: u8, tracking: u32, bundle: i32) -> Vec<u8> {
    let mut bytes = vec![66, 0, opcode, 0];
    put_string(&mut bytes, "");
    put_string(&mut bytes, "");
    bytes.extend(tracking.to_le_bytes());
    put_string(&mut bytes, "en_US");
    put_string(&mut bytes, "");
    put_string(&mut bytes, "");
    put_string(&mut bytes, "");
    bytes.extend(1u32.to_le_bytes());
    bytes.extend((bundle as u32).to_le_bytes());
    bytes.extend(STORE.to_le_bytes());
    bytes.extend(1u32.to_le_bytes());
    put_string(&mut bytes, "0");
    put_string(&mut bytes, "");
    bytes
}

fn order_result(replies: &[Vec<u8>]) -> (u32, u32) {
    let reply = replies
        .iter()
        .find(|reply| reply.starts_with(&[66, 0, 4, 0]))
        .expect("a 66/4 order reply");
    let word = |at: usize| u32::from_le_bytes(reply[at..at + 4].try_into().unwrap());
    let id_length = word(12) as usize;
    (word(4), word(16 + id_length + 4))
}

/// 66/2 (`preview_packet`): result, tracking id, a zero word, total.
fn preview_result(replies: &[Vec<u8>]) -> (u32, u32) {
    let reply = replies
        .iter()
        .find(|reply| reply.starts_with(&[66, 0, 2, 0]))
        .expect("a 66/2 preview reply");
    let word = |at: usize| u32::from_le_bytes(reply[at..at + 4].try_into().unwrap());
    (word(4), word(16))
}

fn subtypes(replies: &[Vec<u8>]) -> Vec<(u8, u8)> {
    replies.iter().map(|reply| (reply[0], reply[2])).collect()
}

/// The opening reference content every gameplay route test loads, plus the repository's
/// `houses.json` and the real catalogue row of the deed under test (from the round70 catalogue).
struct Content(PathBuf);

impl Content {
    fn new() -> Self {
        let reference = std::env::var_os("OSFR_ACTIVITY_CONTENT")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../../lab/references/rust-opening-cooking-original-assets")
            });
        let dir = std::env::temp_dir().join(format!(
            "osfr-lot-deeds-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        for entry in fs::read_dir(&reference).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_file() {
                fs::copy(entry.path(), dir.join(entry.file_name())).unwrap();
            }
        }
        fs::copy(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/houses.json"),
            dir.join("houses.json"),
        )
        .unwrap();
        let record = round70_inventory()
            .definition_record(BOG_SHORE_DEED)
            .expect("the deed has a catalogue row")
            .to_vec();
        item_catalogue::write(&dir, &[item_catalogue::row(BOG_SHORE_DEED, record)]);
        Self(dir)
    }

    fn game(&self, state: Option<GameplayState>) -> Gameplay {
        Gameplay::load(&self.0, state).unwrap()
    }

    /// A fresh character holding `balance` Station Cash.
    fn game_with_balance(&self, balance: u32) -> Gameplay {
        let mut state = self.game(None).state();
        state.sc_wallet = Some(Wallet {
            balance,
            orders: Vec::new(),
        });
        self.game(Some(state))
    }
}

impl Drop for Content {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn character() -> CharacterState {
    CharacterState {
        guid: OWNER,
        zone_id: 0,
        position: [-1904.0, -42.0, 420.0],
        rotation: [0.0, 0.0, 1.0],
    }
}

fn owned_properties(state: &GameplayState) -> Vec<i32> {
    state.housing.as_ref().map_or_else(Vec::new, |housing| {
        housing
            .lots
            .iter()
            .filter(|lot| lot.owner_guid == OWNER)
            .map(|lot| lot.property_id)
            .collect()
    })
}

fn deed_items(state: &GameplayState) -> u32 {
    state
        .inventory
        .items
        .iter()
        .filter(|item| item.definition_id == BOG_SHORE_DEED)
        .map(|item| item.count)
        .sum()
}

fn balance(state: &GameplayState) -> u32 {
    state.sc_wallet.as_ref().map_or(0, |wallet| wallet.balance)
}

#[test]
fn buying_a_deed_grants_its_lot_once_and_charges_once() {
    let content = Content::new();
    let mut game = content.game_with_balance(1_000);
    let out = game
        .handle_in_world(&order(1, BOG_SHORE_DEED), &character(), 1_000)
        .unwrap();
    assert!(out.changed, "the paid lot is saved");
    assert_eq!(
        subtypes(&out.replies),
        vec![(66, 11), (127, 49), (127, 39), (66, 4)],
        "wallet, new-house notice, My Houses list, order result"
    );
    assert_eq!(
        order_result(&out.replies),
        (station_cash::ORDER_SUCCESS, BOG_SHORE_PRICE)
    );
    let mut notice = vec![127, 0, 49, 0];
    notice.extend(BOG_SHORE_NAME.to_le_bytes());
    notice.extend([0; 16]);
    assert_eq!(out.replies[1], notice, "127/49 names the lot");
    let list = InstanceList::decode(&out.replies[2][2..]).expect("a well-formed 127/39");
    assert_eq!(list.owner_guid, OWNER);
    assert_eq!(list.records.len(), 1);
    assert_eq!(list.records[0].name_id, BOG_SHORE_NAME);

    let paid = game.state();
    assert_eq!(owned_properties(&paid), vec![BOG_SHORE_LOT]);
    assert_eq!(balance(&paid), 1_000 - BOG_SHORE_PRICE, "charged once");
    assert_eq!(deed_items(&paid), 0, "the deed never enters the inventory");

    // A replay of the paid order answers success and changes nothing.
    let replay = game
        .handle_in_world(&order(1, BOG_SHORE_DEED), &character(), 2_000)
        .unwrap();
    assert!(!replay.changed);
    assert_eq!(order_result(&replay.replies).0, station_cash::ORDER_SUCCESS);
    assert_eq!(game.state(), paid);

    // A second purchase of a lot the player owns is refused with no charge.
    let before = serde_json::to_vec(&game.state()).unwrap();
    let again = game
        .handle_in_world(&order(2, BOG_SHORE_DEED), &character(), 3_000)
        .unwrap();
    assert!(!again.changed);
    assert_eq!(order_result(&again.replies).0, station_cash::ORDER_FAILURE);
    assert_eq!(
        serde_json::to_vec(&game.state()).unwrap(),
        before,
        "a refused purchase leaves SC, inventory and lots byte-equal"
    );

    // A restart (the one atomic save envelope) keeps the lot and the debit.
    let save = content.0.join("character-save.json");
    game_store::save(
        &save,
        &Snapshot {
            character: character(),
            gameplay: Some(game.state()),
            pets: None,
        },
    )
    .unwrap();
    let restored = game_store::load_or_create(&save, character()).unwrap();
    let mut game = content.game(restored.gameplay);
    assert_eq!(owned_properties(&game.state()), vec![BOG_SHORE_LOT]);
    assert_eq!(balance(&game.state()), 1_000 - BOG_SHORE_PRICE);
    let mut list_request = vec![127, 0, 14, 0];
    list_request.extend(OWNER.to_le_bytes());
    list_request.extend(0u32.to_le_bytes());
    let listed = game
        .handle_in_world(&list_request, &character(), 4_000)
        .unwrap();
    let list = InstanceList::decode(&listed.replies[0][2..]).unwrap();
    assert_eq!(list.records.len(), 1, "My Houses lists the bought lot");
    let before = game.state();
    let after_restart = game
        .handle_in_world(&order(3, BOG_SHORE_DEED), &character(), 5_000)
        .unwrap();
    assert_eq!(
        order_result(&after_restart.replies).0,
        station_cash::ORDER_FAILURE
    );
    assert_eq!(game.state(), before);
}

#[test]
fn a_short_wallet_buys_no_lot() {
    let content = Content::new();
    let mut game = content.game_with_balance(BOG_SHORE_PRICE - 1);
    let before = serde_json::to_vec(&game.state()).unwrap();
    let out = game
        .handle_in_world(&order(1, BOG_SHORE_DEED), &character(), 1_000)
        .unwrap();
    assert!(!out.changed);
    assert_eq!(
        order_result(&out.replies).0,
        station_cash::ORDER_INSUFFICIENT_SC
    );
    assert_eq!(serde_json::to_vec(&game.state()).unwrap(), before);
}

/// A deed listed with no housing session to grant its lot (a direct shop call) is refused and
/// never lands in the bag as an item.
#[test]
fn a_deed_without_a_lot_grant_is_refused_and_never_granted_as_an_item() {
    let inventory = round70_inventory();
    let catalogue = StoreCatalogue::load(&stores()).expect("stores load");
    let mut shop = CoinShop::new(
        &round70(),
        inventory,
        catalogue,
        usize::MAX,
        None,
        Wallet {
            balance: 1_000,
            orders: Vec::new(),
        },
    )
    .expect("shop loads");
    assert!(
        shop.sc_catalogue()
            .listing(BOG_SHORE_DEED as u32)
            .is_some_and(|listing| listing.deed)
    );
    let before = (shop.inventory().state().clone(), shop.sc_wallet().clone());
    let outcome = shop
        .handle(&order(1, BOG_SHORE_DEED))
        .expect("order parses");
    assert_eq!(outcome.result_code, station_cash::ORDER_FAILURE);
    assert!(!outcome.changed);
    assert_eq!(
        (shop.inventory().state().clone(), shop.sc_wallet().clone()),
        before
    );
}

#[test]
fn only_an_enabled_unowned_lot_answers_its_deed() {
    let mut service = HousingService::with_content(
        FixtureConfig::capture_defaults(),
        HousingContent::bundled().unwrap(),
    )
    .unwrap();
    let empty = service.state();
    for deed in [DISABLED_DEED, CONDO_DEED, 999_999] {
        assert_eq!(
            service.grant_deed(OWNER, deed).unwrap_err(),
            ServiceError::Housing(HousingError::InvalidProperty),
            "deed {deed} is a lookup refusal, not a content error"
        );
        assert_eq!(service.state(), empty, "deed {deed}");
    }
    let granted = service.grant_deed(OWNER, BOG_SHORE_DEED).unwrap();
    assert!(granted.changed);
    assert_eq!(
        granted.replies[0],
        HousingReply::NotifyHouseAdded(NotifyHouseAdded {
            name_id: BOG_SHORE_NAME
        })
    );
    let owned = service.state();
    assert_eq!(owned.lots.len(), 1);
    assert!(service.grant_deed(OWNER, BOG_SHORE_DEED).is_err());
    assert_eq!(service.state(), owned, "a second grant changes nothing");
}

/// L040 x L046: the starter kit rides only the free 127/23 grant. A bought lot stands empty and
/// adds nothing to the bag, and a 127/23 after the purchase still grants the Condo with its kit.
#[test]
fn a_deed_lot_comes_without_the_starter_kit_and_the_condo_keeps_it() {
    let content = Content::new();
    let mut game = content.game_with_balance(1_000);
    let bag = game.state().inventory;
    let out = game
        .handle_in_world(&order(1, BOG_SHORE_DEED), &character(), 1_000)
        .unwrap();
    assert_eq!(order_result(&out.replies).0, station_cash::ORDER_SUCCESS);
    let paid = game.state();
    assert_eq!(paid.inventory, bag, "a deed lot puts nothing in the bag");
    let lots = paid.housing.unwrap().lots;
    assert!(
        matches!(&lots[..], [lot] if lot.property_id == BOG_SHORE_LOT && lot.fixtures.is_empty())
    );

    let out = game
        .handle_in_world(&[127, 0, 23, 0], &character(), 2_000)
        .unwrap();
    assert!(out.changed, "the Condo is saved");
    let state = game.state();
    let chickens: Vec<u32> = state
        .inventory
        .items
        .iter()
        .filter(|item| item.definition_id == CHICKEN)
        .map(|item| item.count)
        .collect();
    assert_eq!(chickens, vec![1], "the Condo brings one Chicken");
    let fixtures = |property: i32| {
        let lots = &state.housing.as_ref().unwrap().lots;
        lots.iter()
            .find(|lot| lot.property_id == property)
            .map(|lot| lot.fixtures.len())
    };
    assert_eq!(
        fixtures(CONDO),
        Some(KIT_FIXTURES),
        "the kit stands in the Condo"
    );
    assert_eq!(
        fixtures(BOG_SHORE_LOT),
        Some(0),
        "the bought lot stays empty"
    );
}

/// 127/49 against its client reader `00ab2f50`: subtype, `u32` name id, two `u64`s, no more.
#[test]
fn new_house_notice_matches_the_client_reader() {
    let body = NotifyHouseAdded { name_id: 69_974 }.encode().unwrap();
    let mut expected = vec![49, 0];
    expected.extend(69_974i32.to_le_bytes());
    expected.extend([0; 16]);
    assert_eq!(body, expected);
    assert_eq!(
        HousingReply::NotifyHouseAdded(NotifyHouseAdded { name_id: 69_974 })
            .encode_inner()
            .unwrap()[..4],
        [127, 0, 49, 0]
    );
}

#[test]
fn a_deed_that_names_two_lots_is_refused() {
    let mut content = HousingContent::bundled().unwrap();
    let deed = content.lots[2].deed.clone().expect("lot row 3 has a deed");
    content.lots[3].deed = Some(deed);
    assert!(content.validate().is_err());
}

/// The data gate: every recorded deed is a Type 16 catalogue row whose NameId is the lot's
/// name_id (no mismatch is recorded), and every deed the Marketplace lists names exactly one
/// enabled, non-starter lot that an original store bundle sold.
#[test]
fn shipped_deeds_name_their_lot_and_only_sold_lots_are_listed() {
    let houses = HousingContent::bundled().unwrap();
    let inventory = round70_inventory();
    for lot in &houses.lots {
        let Some(deed) = &lot.deed else { continue };
        let presentation = inventory
            .presentation(deed.item_id)
            .unwrap_or_else(|_| panic!("deed {} has no catalogue row", deed.item_id));
        assert_eq!(
            presentation.item_type, DEED_ITEM_TYPE,
            "deed {}",
            deed.item_id
        );
        assert_eq!(
            presentation.name_id, lot.name_id,
            "deed {} names property {}",
            deed.item_id, lot.property_id
        );
    }
    let shop = CoinShop::new(
        &round70(),
        inventory,
        StoreCatalogue::load(&stores()).unwrap(),
        usize::MAX,
        None,
        Wallet::default(),
    )
    .unwrap();
    let mut listed = Vec::new();
    for listing in shop.sc_catalogue().listings().iter().filter(|l| l.deed) {
        let lots: Vec<_> = houses
            .lots
            .iter()
            .filter(|lot| {
                lot.deed
                    .as_ref()
                    .is_some_and(|deed| deed.item_id == listing.item_id)
            })
            .collect();
        assert_eq!(lots.len(), 1, "deed {} names one lot", listing.item_id);
        let lot = lots[0];
        assert!(lot.enabled, "property {} is disabled", lot.property_id);
        assert_ne!(lot.property_id, houses.starter_property_id);
        let deed = lot.deed.as_ref().unwrap();
        assert!(
            !deed.store_bundle_ids.is_empty(),
            "property {} was never sold",
            lot.property_id
        );
        // A buyer must get the lot's own scene: the one its deed's source zone file names. This
        // keeps lots 9 and 14 (and 22) off sale until their scenes are corrected.
        let scene = &lot.scene.as_ref().expect("an enabled lot has a scene").name;
        assert!(
            deed.source
                .contains(&format!("src/Resources/Zones/{scene}.json ")),
            "property {} is sold with scene {scene}, which its deed source does not name",
            lot.property_id
        );
        listed.push(lot.property_id);
    }
    listed.sort_unstable();
    assert_eq!(listed, vec![3, 4, 5, 6, 10, 16, 19, 20, 21]);
}

/// 66/1 asks the housing session without committing: an unowned lot previews at its price and
/// nothing changes; a lot the player owns previews as the refusal the order would give.
#[test]
fn a_preview_asks_for_the_lot_and_grants_nothing() {
    let content = Content::new();
    let mut game = content.game_with_balance(1_000);
    let before = serde_json::to_vec(&game.state()).unwrap();
    let out = game
        .handle_in_world(&preview(1, BOG_SHORE_DEED), &character(), 1_000)
        .unwrap();
    assert!(!out.changed);
    assert_eq!(
        preview_result(&out.replies),
        (station_cash::ORDER_SUCCESS, BOG_SHORE_PRICE)
    );
    assert_eq!(serde_json::to_vec(&game.state()).unwrap(), before);

    let paid = game
        .handle_in_world(&order(2, BOG_SHORE_DEED), &character(), 2_000)
        .unwrap();
    assert_eq!(order_result(&paid.replies).0, station_cash::ORDER_SUCCESS);
    let before = serde_json::to_vec(&game.state()).unwrap();
    let owned = game
        .handle_in_world(&preview(3, BOG_SHORE_DEED), &character(), 3_000)
        .unwrap();
    assert!(!owned.changed);
    assert_eq!(
        preview_result(&owned.replies),
        (station_cash::ORDER_FAILURE, 0)
    );
    assert_eq!(serde_json::to_vec(&game.state()).unwrap(), before);
}

/// With no `houses.json` at login housing is dormant: a deed previews and orders as a refusal
/// with no charge, and a character's saved (paid) lots pass through the save unchanged.
#[test]
fn dormant_housing_refuses_deeds_and_keeps_saved_lots() {
    let content = Content::new();
    let mut game = content.game_with_balance(1_000);
    let out = game
        .handle_in_world(&order(1, BOG_SHORE_DEED), &character(), 1_000)
        .unwrap();
    assert_eq!(order_result(&out.replies).0, station_cash::ORDER_SUCCESS);
    let paid = game.state();
    assert_eq!(owned_properties(&paid), vec![BOG_SHORE_LOT]);

    fs::remove_file(content.0.join("houses.json")).unwrap();
    let dormant = content.game(Some(paid.clone()));
    assert_eq!(
        dormant.state().housing,
        paid.housing,
        "a dormant session keeps the saved lots"
    );

    let mut fresh = content.game_with_balance(1_000);
    let before = serde_json::to_vec(&fresh.state()).unwrap();
    let previewed = fresh
        .handle_in_world(&preview(2, BOG_SHORE_DEED), &character(), 2_000)
        .unwrap();
    assert_eq!(
        preview_result(&previewed.replies),
        (station_cash::ORDER_FAILURE, 0)
    );
    let ordered = fresh
        .handle_in_world(&order(3, BOG_SHORE_DEED), &character(), 3_000)
        .unwrap();
    assert!(!ordered.changed);
    assert_eq!(
        order_result(&ordered.replies).0,
        station_cash::ORDER_FAILURE
    );
    assert_eq!(serde_json::to_vec(&fresh.state()).unwrap(), before);
}
