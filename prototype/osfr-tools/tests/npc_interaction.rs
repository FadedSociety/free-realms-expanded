#[path = "common/resale.rs"]
mod resale;

use osfr_tools::npc_interaction::{
    DialoguePacket26_3, MerchantBuyRequestV2, MerchantEntryV2, MerchantGroupV2, MerchantListV2,
    NpcAction, NpcInteraction, NpcInteractionError, NpcSelection, QuestOfferPacket49_1,
    decode_coin_merchant_list_165_10, decode_dialogue_26_3, decode_dialogue_response_26_6,
    decode_merchant_list_66_42, decode_npc_interact_26_8, decode_quest_end_reply_49_14,
    decode_quest_offer_49_1, decode_quest_reply_49_2, encode_coin_merchant_list_165_10,
    encode_dialogue_26_3, encode_dialogue_close_26_4, encode_merchant_buy_66_43,
    encode_merchant_list_66_42, encode_quest_offer_49_1, goodbye_dialogue_choice,
};
use osfr_tools::{
    coin_shop::{self, CoinShop, StoreCatalogue},
    inventory::{Inventory, InventoryState},
    quest_engine::{
        CharacterEligibility, ObjectiveKind, QuestDefinition, QuestEngine, QuestObjective,
        QuestRequirements,
    },
    rewards::{RewardDisplay, RewardGrant},
};
use resale::resale;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

const VENDOR_ID: u64 = 9_500_000_339;
const QUEST_GIVER_ID: u64 = 9_000_000_047;
const QUEST_ID: u32 = 3_110_005;

fn content_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn inventory_content() -> PathBuf {
    std::env::var_os("OSFR_COIN_SHOP_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab/references/rust-gameplay")
        })
}

fn coin_shop(directory: &Path) -> CoinShop {
    let loaded = Inventory::load(directory, None).expect("inventory definitions load");
    let mut state = loaded.state().clone();
    state.coins = 100;
    let inventory = Inventory::load(directory, Some(state)).expect("test balance is valid");
    let catalogue = StoreCatalogue::load(&content_dir().join("stores.json"))
        .expect("store price catalogue loads");
    CoinShop::new(
        directory,
        inventory,
        catalogue,
        usize::MAX,
        None,
        osfr_tools::station_cash::Wallet::default(),
    )
    .expect("coin shop loads")
}

fn count_definition(state: &InventoryState, definition_id: i32) -> u32 {
    state
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id)
        .map(|item| item.count)
        .sum()
}

/// 165/4 client request: client writer `0x00c75ed0` (BuyItem `0x00c77230`) writes
/// merchant u64, record id, definition, tint, then quantity as the last dword.
fn request_buy(
    merchant_id: u64,
    transaction_id: u32,
    definition_id: i32,
    tint: i32,
    quantity: u32,
) -> Vec<u8> {
    let mut packet = vec![165, 0, 4, 0];
    packet.extend(merchant_id.to_le_bytes());
    packet.extend(transaction_id.to_le_bytes());
    packet.extend(definition_id.to_le_bytes());
    packet.extend(tint.to_le_bytes());
    packet.extend(quantity.to_le_bytes());
    packet
}

fn request_selection(npc_id: u64, option_id: i32) -> Vec<u8> {
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(npc_id.to_le_bytes());
    packet.extend(option_id.to_le_bytes());
    packet
}

#[test]
fn vendor_menu_shop_buy_sell_and_close_use_coin_shop_validation() {
    let data = content_dir();
    let mut service = NpcInteraction::load(&data.join("npcs"), &data.join("stores.json"))
        .expect("NPC and store content load");
    // (R) The source stock is prose; item 8 is an explicit fixture mapping for this path test.
    service
        .set_vendor_stock_items(VENDOR_ID, &[8])
        .expect("test vendor stock mapping is accepted");
    let (zone, position) = service.location(VENDOR_ID).expect("content vendor exists");
    let menu = service.interact(VENDOR_ID, zone, position, None).unwrap();
    let mut expected_menu = vec![26, 0, 9, 0];
    expected_menu.extend(VENDOR_ID.to_le_bytes());
    expected_menu.push(0);
    expected_menu.extend(2i32.to_le_bytes());
    for value in [1i32, 0, 3300, 0, 0, 0, 0, 2, 0, 432156, 0, 0, 0, 0] {
        expected_menu.extend(value.to_le_bytes());
    }
    expected_menu.extend(9u32.to_le_bytes());
    expected_menu.extend(b"Alexander");
    expected_menu.extend([1, 0]);
    assert_eq!(menu.packet, expected_menu);
    let shop_option = menu
        .options
        .iter()
        .find(|option| option.action == NpcAction::Shop)
        .expect("vendor exposes shop option");
    let selected = service
        .select(
            &request_selection(VENDOR_ID, shop_option.id),
            zone,
            position,
            None,
        )
        .unwrap();
    let NpcSelection::Shop(window) = selected else {
        panic!("shop option opens merchant window")
    };
    assert!(
        window
            .stock_description
            .as_deref()
            .is_some_and(|text| !text.is_empty())
    );
    assert_eq!(window.items.len(), 1);
    assert!(
        window
            .items
            .iter()
            .any(|item| item.item_id == 8 && item.purchase_price_coins == Some(15))
    );

    let directory = inventory_content();
    let mut shop = coin_shop(&directory);
    let original_count = count_definition(shop.inventory().state(), 8);
    let bought = service
        .handle_coin_store(
            &mut shop,
            &request_buy(VENDOR_ID, 77001, 8, 0, 1),
            zone,
            position,
        )
        .unwrap();
    assert_eq!(bought.result_code, coin_shop::RESULT_SUCCESS);
    assert_eq!(shop.inventory().state().coins, 85);
    assert_eq!(
        count_definition(shop.inventory().state(), 8),
        original_count + 1
    );
    // The same 165/4 again is a second purchase: the native writer (`00c75ed0`) has no
    // per-request id, and its context dword repeats for every buy from one open shop.
    let again = service
        .handle_coin_store(
            &mut shop,
            &request_buy(VENDOR_ID, 77001, 8, 0, 1),
            zone,
            position,
        )
        .unwrap();
    assert!(again.changed);
    assert_eq!(shop.inventory().state().coins, 70);
    assert_eq!(
        count_definition(shop.inventory().state(), 8),
        original_count + 2
    );

    let instance_id = shop
        .inventory()
        .state()
        .items
        .iter()
        .find(|item| item.definition_id == 8)
        .expect("purchased item exists")
        .id;
    let mut sell = vec![165, 0, 5, 0];
    sell.extend(VENDOR_ID.to_le_bytes());
    sell.extend(instance_id.to_le_bytes());
    sell.extend(1u32.to_le_bytes());
    let sold = service
        .handle_coin_store(&mut shop, &sell, zone, position)
        .unwrap();
    assert_eq!(sold.result_code, coin_shop::RESULT_SUCCESS);
    assert_eq!(shop.inventory().state().coins, 70 + resale(8));
    assert_eq!(
        count_definition(shop.inventory().state(), 8),
        original_count + 1
    );

    assert_eq!(service.close(VENDOR_ID).unwrap(), [66, 0, 46, 0]);
    assert!(matches!(
        service.handle_coin_store(
            &mut shop,
            &request_buy(VENDOR_ID, 77001, 8, 0, 1),
            zone,
            position
        ),
        Err(NpcInteractionError::NoOpenShop)
    ));
}

#[test]
fn quest_giver_talk_candidate_uses_quest_engine_eligibility() {
    let data = content_dir();
    let mut service = NpcInteraction::load(&data.join("npcs"), &data.join("stores.json"))
        .expect("NPC and store content load");
    let (zone, position) = service
        .location(QUEST_GIVER_ID)
        .expect("content quest giver exists");
    let definition = QuestDefinition {
        id: QUEST_ID,
        title: "Synthetic quest".into(),
        description: "Synthetic offer body".into(),
        giver_npc_id: QUEST_GIVER_ID,
        turnin_npc_id: QUEST_GIVER_ID,
        prerequisite_quest_ids: Vec::new(),
        requirements: QuestRequirements::default(),
        ordered: false,
        objectives: vec![QuestObjective {
            id: 1,
            required: 1,
            description: "Talk to the guide".into(),
            kind: ObjectiveKind::TalkToNpc {
                npc_id: QUEST_GIVER_ID,
            },
        }],
        reward: RewardGrant {
            coins: 0,
            items: Vec::new(),
            job_xp: BTreeMap::new(),
            sc: 0,
            display: RewardDisplay {
                source_guid: QUEST_GIVER_ID,
                player_guid: 7,
                reward_kind: 0,
                icon_id: 0,
                name_id: 0,
                rows: Vec::new(),
            },
        },
    };
    let eligibility = CharacterEligibility {
        nonmember_quest_limit: 20,
        member_quest_limit: 30,
        ..Default::default()
    };
    let mut quests = QuestEngine::new(7, vec![definition], eligibility, None).unwrap();
    let menu = service
        .interact(QUEST_GIVER_ID, zone, position, Some(&quests))
        .unwrap();
    let talk = menu
        .options
        .iter()
        .find(|option| option.action == NpcAction::Talk)
        .expect("NPC exposes talk option");
    let NpcSelection::Talk(dialogue) = service
        .select(
            &request_selection(QUEST_GIVER_ID, talk.id),
            zone,
            position,
            Some(&mut quests),
        )
        .unwrap()
    else {
        panic!("talk option returns dialogue context")
    };
    // T378: Chloe now carries the recovered Ninja offer greeting; the synthetic
    // quest still adds no menu option and keeps its title id.
    assert_eq!(dialogue.body_string_id, Some(78_852));
    // Global.Text id of "Ninja: Strike from the Shadows" (was the raw locale
    // hash 2580872460, which the client would show blank).
    assert!(dialogue.quest_title_string_ids.contains(&78_853));
    assert_eq!(
        menu.options.len(),
        1,
        "new quests do not add a radial option"
    );
    assert_eq!(
        service.offerable_quest_for_talk(QUEST_GIVER_ID, &quests),
        Some(QUEST_ID),
        "the gameplay Talk path chooses the eligible offer"
    );
    assert_eq!(quests.state().pending_offer, None);
    assert_eq!(
        service.quest_title_string_id(QUEST_GIVER_ID, QUEST_ID),
        Some(78_853)
    );
}

#[test]
fn interaction_rejects_out_of_range_npc() {
    let data = content_dir();
    let mut service = NpcInteraction::load(&data.join("npcs"), &data.join("stores.json")).unwrap();
    let (zone, position) = service.location(VENDOR_ID).unwrap();
    assert_eq!(
        service.interact(
            VENDOR_ID,
            zone,
            [position[0] + 12.01, position[1], position[2]],
            None
        ),
        Err(NpcInteractionError::OutOfRange)
    );
}

#[test]
fn interaction_rejects_unknown_npc() {
    let data = content_dir();
    let mut service = NpcInteraction::load(&data.join("npcs"), &data.join("stores.json")).unwrap();
    assert_eq!(
        service.interact(1, 0, [0.0; 3], None),
        Err(NpcInteractionError::UnknownNpc)
    );
}

#[test]
fn typed_quest_giver_id_fills_an_unlisted_regional_npc_offer() {
    let data = content_dir();
    let mut service = NpcInteraction::load(&data.join("npcs"), &data.join("stores.json"))
        .expect("NPC and store content load");
    // Quest 3000020 is keyed to giver 9000000402 in quests/adventurer.json; that
    // relationship is absent from the NPC record's quest_ids_given array. (3000008,
    // used before, is now disabled until its craft step is routable: W110.)
    let npc_id = 9_000_000_402;
    let quest_id = 3_000_020;
    let eligibility = CharacterEligibility {
        player_level: 1,
        nonmember_quest_limit: 100,
        member_quest_limit: 100,
        ..Default::default()
    };
    let quests = QuestEngine::from_split_content(&data, 7, eligibility, None).unwrap();
    let (zone, position) = service
        .location(npc_id)
        .expect("regional giver has a location");
    let menu = service
        .interact(npc_id, zone, position, Some(&quests))
        .unwrap();
    assert!(
        !menu
            .options
            .iter()
            .any(|option| { option.string_id == quests.title_string_id(quest_id).unwrap() })
    );
    assert_eq!(
        service.offerable_quest_for_talk(npc_id, &quests),
        Some(quest_id)
    );
}

#[test]
fn npc_interact_26_8_decodes_and_reencodes_captured_request() {
    // Real inner frame from lab/runs/client-baseline-20261009T103605Z/analysis/application-0066.bin;
    // the seven-byte capture envelope is stripped. It contains only the request fields.
    let frame = [26, 0, 8, 0, 5, 0, 0, 0, 0x4e, 0x45, 0x50, 0x4f, 0];
    let decoded = decode_npc_interact_26_8(&frame).unwrap();
    assert_eq!(decoded.trailing_flag, false);
    assert_eq!(decoded.encode(), frame);
    assert!(decode_npc_interact_26_8(&frame[..12]).is_err());
    let mut invalid_bool = frame;
    invalid_bool[12] = 2;
    assert!(decode_npc_interact_26_8(&invalid_bool).is_err());
}

#[test]
fn zone_less_legacy_vendor_uses_open_world_range_and_shop_options() {
    const ROOSEY_ID: u64 = 9_500_000_255;
    let data = content_dir();
    let mut service = NpcInteraction::load(&data.join("npcs"), &data.join("stores.json"))
        .expect("regional NPCs load");
    let (zone_id, position) = service
        .location(ROOSEY_ID)
        .expect("zone-less vendor has a world location");
    assert_eq!(zone_id, 0);

    let menu = service
        .interact(ROOSEY_ID, 0, position, None)
        .expect("player can interact in the open world");
    let shop = menu
        .options
        .iter()
        .find(|option| option.action == NpcAction::Shop)
        .expect("vendor menu exposes Shop");
    assert!(matches!(
        service.select(&request_selection(ROOSEY_ID, shop.id), 0, position, None,),
        Ok(NpcSelection::Shop(_))
    ));
}

#[test]
fn dialogue_26_3_synthetic_fixture_round_trips_exact_native_field_order() {
    let message = DialoguePacket26_3 {
        initial_dwords: [1, 2, 3, 4],
        initial_flag: true,
        initial_float_bits: 0x3f80_0000,
        choices: vec![[10, 11, 12, 13, 14], [20, 21, 22, 23, 24]],
        choice_vectors: [[30, 31, 32, 33], [40, 41, 42, 43]],
        vector_flag: false,
        final_vector: [50, 51, 52, 53],
        float_after_vector_bits: 0x4000_0000,
        final_flags: [true, false, true],
        final_float_bits: 0x4040_0000,
        final_dword: 60,
    };
    let encoded = encode_dialogue_26_3(&message).unwrap();
    assert_eq!(encoded.len(), 93 + 2 * 20);
    assert_eq!(decode_dialogue_26_3(&encoded).unwrap(), message);
}

#[test]
fn quest_offer_49_1_matches_captured_empty_reward_packet() {
    // Native reader layout (Frida, run 142216): u16 family + u32 subtype, quest
    // id + five dwords, bool, 69-byte empty bundle, 8 bytes, float, two bools.
    let message = QuestOfferPacket49_1 {
        quest_id: 3_000_021,
        intro_text_string_id: 94_246,
        field_2: 94_247,
        name_string_id: 94_247,
        description_string_id: 94_248,
        field_5: 94_247,
        members_only: false,
        reward_bundle_body: vec![0; 69],
        trailing_dwords: [0; 2],
        trailing_float_bits: 0,
        trailing_flags: [false; 2],
    };
    let encoded = encode_quest_offer_49_1(&message).unwrap();
    assert_eq!(&encoded[..6], &[49, 0, 1, 0, 0, 0]);
    assert_eq!(&encoded[6..10], &[0xd5, 0xc6, 0x2d, 0x00]);
    assert_eq!(
        &encoded[10..30],
        &[
            0x26, 0x70, 0x01, 0x00, 0x27, 0x70, 0x01, 0x00, 0x27, 0x70, 0x01, 0x00, 0x28, 0x70,
            0x01, 0x00, 0x27, 0x70, 0x01, 0x00,
        ]
    );
    assert_eq!(encoded[30], 0);
    assert!(encoded[31..100].iter().all(|byte| *byte == 0));
    assert!(encoded[100..].iter().all(|byte| *byte == 0));
    assert_eq!(encoded.len(), 114);
    assert_eq!(decode_quest_offer_49_1(&encoded).unwrap(), message);
}

#[test]
fn dialogue_uses_a_greeting_locale_id_instead_of_the_npc_name() {
    let data = content_dir();
    let mut service = NpcInteraction::load(&data.join("npcs"), &data.join("stores.json"))
        .expect("NPC and store content load");
    let merigold_id = 9_000_000_402;
    let (zone, position) = service
        .location(merigold_id)
        .expect("Merigold is in content");
    let menu = service.interact(merigold_id, zone, position, None).unwrap();
    let talk = menu
        .options
        .iter()
        .find(|option| option.action == NpcAction::Talk)
        .unwrap();
    let NpcSelection::Talk(context) = service
        .select(
            &request_selection(merigold_id, talk.id),
            zone,
            position,
            None,
        )
        .unwrap()
    else {
        panic!("Talk selection returns dialogue context")
    };
    assert_eq!(context.body_string_id, Some(80));
}

fn sample_merchant_group() -> MerchantGroupV2 {
    MerchantGroupV2 {
        fields: [1, 2, 3, 4, 5, 6, 0],
        entries: vec![MerchantEntryV2 {
            item_id: 8,
            icon_id: 9,
            tint_value: 10,
            name_id: 11,
            description_id: 12,
            quantity_available: 13,
            members_only: true,
            field_7: 14,
            first_item_id: 15,
            available_tint_group_id: 16,
            can_buy: false,
        }],
    }
}

#[test]
fn merchant_list_66_42_matches_the_client_reader_layout() {
    // Client reader (`00b3d5f0`): the header, the time-left dword, a six-dword record
    // (`00b33020`), the entry count (`00b2e620`), then 38-byte rows. Synthetic values, no
    // captured frame exists.
    let message = MerchantListV2 {
        initial_dword: 99,
        group: sample_merchant_group(),
    };
    let encoded = encode_merchant_list_66_42(&message).unwrap();
    assert_eq!(&encoded[..8], &[66, 0, 42, 0, 99, 0, 0, 0]);
    assert_eq!(encoded.len(), 4 + 4 + 24 + 4 + 38);
    assert_eq!(&encoded[36..40], &8u32.to_le_bytes());
    assert_eq!(decode_merchant_list_66_42(&encoded).unwrap(), message);
    // The off-wire seventh record dword is refused; the old 8-byte-longer shape is refused.
    let mut legacy = message.clone();
    legacy.group.fields[6] = 1;
    assert!(encode_merchant_list_66_42(&legacy).is_err());
    let mut extended = encoded.clone();
    extended.splice(4..4, [0u8; 4]);
    assert!(decode_merchant_list_66_42(&extended).is_err());
    assert!(decode_merchant_list_66_42(&encoded[..encoded.len() - 1]).is_err());
}

#[test]
fn coin_merchant_list_165_10_matches_the_same_row_reader_layout() {
    // `00c78fb0` reads the 4-byte header, then the same six-dword record and 38-byte rows
    // (`00b33020`/`00b2e620`). It has no time-left dword and no Station Cash field.
    let group = sample_merchant_group();
    let encoded = encode_coin_merchant_list_165_10(&group).unwrap();
    assert_eq!(&encoded[..4], &[165, 0, 10, 0]);
    assert_eq!(encoded.len(), 4 + 24 + 4 + 38);
    assert_eq!(&encoded[4..8], &1u32.to_le_bytes()); // record scalar
    assert_eq!(&encoded[16..20], &4u32.to_le_bytes()); // merchant GUID low
    assert_eq!(&encoded[20..24], &5u32.to_le_bytes()); // merchant GUID high
    assert_eq!(&encoded[28..32], &1u32.to_le_bytes()); // entry count
    assert_eq!(&encoded[32..36], &8u32.to_le_bytes()); // item id
    assert_eq!(&encoded[36..40], &9u32.to_le_bytes()); // icon
    assert_eq!(&encoded[40..44], &10u32.to_le_bytes()); // tint
    assert_eq!(&encoded[44..48], &11u32.to_le_bytes()); // name
    assert_eq!(&encoded[48..52], &12u32.to_le_bytes()); // description
    assert_eq!(&encoded[52..56], &13u32.to_le_bytes()); // quantity
    assert_eq!(encoded[56], 1); // members only
    assert_eq!(&encoded[57..61], &14u32.to_le_bytes()); // field 7
    assert_eq!(&encoded[61..65], &15u32.to_le_bytes()); // first item id
    assert_eq!(&encoded[65..69], &16u32.to_le_bytes()); // available tint group
    assert_eq!(encoded[69], 0); // can buy
    assert_eq!(decode_coin_merchant_list_165_10(&encoded).unwrap(), group);
    assert!(decode_coin_merchant_list_165_10(&encoded[..encoded.len() - 1]).is_err());
    // The two families are not interchangeable.
    assert!(decode_merchant_list_66_42(&encoded).is_err());
    let other = encode_merchant_list_66_42(&MerchantListV2 {
        initial_dword: 0,
        group: group.clone(),
    })
    .unwrap();
    assert!(decode_coin_merchant_list_165_10(&other).is_err());
}

#[test]
fn merchant_buy_66_43_is_parsed_only_for_an_open_vendor_session() {
    let data = content_dir();
    let mut service = NpcInteraction::load(&data.join("npcs"), &data.join("stores.json")).unwrap();
    let request = MerchantBuyRequestV2 {
        initial_dwords: [1, 2],
        context_dwords: [3, 4, 5],
        context_bytes: b"synthetic-context".to_vec(),
        trailing_dwords: [6, 7, 8],
    };
    let packet = encode_merchant_buy_66_43(&request).unwrap();
    assert!(matches!(
        service.handle_merchant_request_66_43(&packet),
        Err(NpcInteractionError::NoOpenShop)
    ));
    let (zone, position) = service.location(VENDOR_ID).unwrap();
    let menu = service.interact(VENDOR_ID, zone, position, None).unwrap();
    let shop = menu
        .options
        .iter()
        .find(|item| item.action == NpcAction::Shop)
        .unwrap();
    let opened = service
        .select(&request_selection(VENDOR_ID, shop.id), zone, position, None)
        .unwrap();
    // Stock contents are covered by `npc_interaction_vendors.rs` (W47c).
    assert!(matches!(opened, NpcSelection::Shop(_)));
    assert_eq!(
        service.handle_merchant_request_66_43(&packet).unwrap(),
        request
    );
    assert!(matches!(
        service.handle_merchant_request_66_43(&packet[..packet.len() - 1]),
        Err(NpcInteractionError::InvalidPacket)
    ));
}

/// (R) The NPC `vendor` role and the explicit `vendors.json` mapping together authorize
/// stock. A mapping for a role-less NPC must fail the load loudly instead of authorizing a
/// sale; the F307 review found the mapping-only rule was an unsigned roles change.
#[test]
fn a_vendor_mapping_without_the_content_role_is_refused() {
    let root = std::env::temp_dir().join(format!(
        "osfr-vendor-role-{}-{VENDOR_ID}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("npcs")).expect("fixture NPC directory");
    let role_less = br#"{"schema_version":1,"records":[{"npc_id":9500000339,"name":"Fixture","zone_id":0,"position":[0.0,0.0,0.0],"roles":["ambient"],"vendor_stock":"synthetic"}]}"#;
    let role_bearing = br#"{"schema_version":1,"records":[{"npc_id":9500000339,"name":"Fixture","zone_id":0,"position":[0.0,0.0,0.0],"roles":["vendor"],"vendor_stock":"synthetic"}]}"#;
    std::fs::write(root.join("npcs/fixture.json"), role_less).expect("fixture NPC writes");
    std::fs::write(
        root.join("vendors.json"),
        br#"{"schema_version":1,"vendors":[{"npc_id":9500000339,"item_ids":[8]}]}"#,
    )
    .expect("fixture vendor writes");
    std::fs::write(
        root.join("stores.json"),
        br#"{"schema_version":1,"items":[{"item_id":8,"purchase_price_coins":15,"sell_price_coins":7}]}"#,
    )
    .expect("fixture store writes");

    let error = match NpcInteraction::load(&root.join("npcs"), &root.join("stores.json")) {
        Err(error) => error,
        Ok(_) => panic!("a mapping without the vendor role must refuse the load"),
    };
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(
        error.to_string().contains("invalid vendor stock mapping"),
        "unexpected refusal: {error}"
    );

    // Positive control: the reviewed role+mapping rule still loads and opens the mapped stock.
    std::fs::write(root.join("npcs/fixture.json"), role_bearing).expect("fixture NPC writes");
    let mut service = NpcInteraction::load(&root.join("npcs"), &root.join("stores.json"))
        .expect("the role and the explicit mapping authorize stock");
    let menu = service
        .interact(VENDOR_ID, 0, [0.0; 3], None)
        .expect("the vendor opens its menu");
    let shop = menu
        .options
        .iter()
        .find(|option| option.action == NpcAction::Shop)
        .expect("the role adds the Shop action");
    let NpcSelection::Shop(window) = service
        .select(&request_selection(VENDOR_ID, shop.id), 0, [0.0; 3], None)
        .expect("the Shop choice opens the mapped stock")
    else {
        panic!("the Shop option selected a non-shop action");
    };
    assert_eq!(window.items.len(), 1);
    assert_eq!(window.items[0].item_id, 8);
    assert_eq!(window.items[0].purchase_price_coins, Some(15));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn dialogue_includes_goodbye_action_and_native_close_bytes() {
    let packet = encode_dialogue_26_3(&DialoguePacket26_3 {
        initial_dwords: [0; 4],
        initial_flag: false,
        initial_float_bits: 0,
        choices: vec![goodbye_dialogue_choice()],
        choice_vectors: [[0; 4]; 2],
        vector_flag: false,
        final_vector: [0; 4],
        float_after_vector_bits: 0,
        final_flags: [false; 3],
        final_float_bits: 0,
        final_dword: 0,
    })
    .unwrap();
    assert_eq!(packet.len(), 113);
    assert_eq!(&packet[..4], &[26, 0, 3, 0]);
    assert_eq!(
        &packet[29..49],
        &[
            0x2c, 1, 0, 0, 17, 0, 0, 0, 0x8b, 0x1f, 0, 0, 0, 0, 0, 0, 0x12, 0, 0, 0
        ]
    );
    assert_eq!(
        decode_dialogue_26_3(&packet).unwrap().choices[0],
        [300, 17, 8075, 0, 18]
    );
    assert_eq!(encode_dialogue_close_26_4(), [26, 0, 4, 0]);
    assert_eq!(
        decode_dialogue_response_26_6(&[26, 0, 6, 0, 0, 0, 0, 0]),
        Ok(0)
    );
}

#[test]
fn dialogue_close_releases_talk_but_keeps_an_open_shop() {
    let data = content_dir();
    let mut service = NpcInteraction::load(&data.join("npcs"), &data.join("stores.json"))
        .expect("NPC and store content load");
    let (zone, position) = service.location(VENDOR_ID).expect("content vendor exists");

    let menu = service.interact(VENDOR_ID, zone, position, None).unwrap();
    let talk = menu
        .options
        .iter()
        .find(|option| option.action == NpcAction::Talk)
        .unwrap();
    assert!(matches!(
        service
            .select(&request_selection(VENDOR_ID, talk.id), zone, position, None,)
            .unwrap(),
        NpcSelection::Talk(_)
    ));
    service.close_dialogue();
    assert!(matches!(
        service.select(&request_selection(VENDOR_ID, talk.id), zone, position, None,),
        Err(NpcInteractionError::NoInteraction)
    ));

    let menu = service.interact(VENDOR_ID, zone, position, None).unwrap();
    let shop = menu
        .options
        .iter()
        .find(|option| option.action == NpcAction::Shop)
        .unwrap();
    service
        .select(&request_selection(VENDOR_ID, shop.id), zone, position, None)
        .unwrap();
    service.close_dialogue();
    let request = encode_merchant_buy_66_43(&MerchantBuyRequestV2 {
        initial_dwords: [1, 2],
        context_dwords: [3, 4, 5],
        context_bytes: b"synthetic-context".to_vec(),
        trailing_dwords: [6, 7, 8],
    })
    .unwrap();
    assert!(service.handle_merchant_request_66_43(&request).is_ok());
}

#[test]
fn quest_reply_49_2_reads_the_client_id_and_accept_flag() {
    // Captured natively (run 143124): accept of quest 3000021.
    assert_eq!(
        decode_quest_reply_49_2(&[49, 0, 2, 0, 0, 0, 0xd5, 0xc6, 0x2d, 0, 1]),
        Ok((3_000_021, true))
    );
    assert_eq!(
        decode_quest_reply_49_2(&[49, 0, 2, 0, 0, 0, 3, 0, 0, 0, 2]),
        Err(NpcInteractionError::InvalidPacket)
    );
    assert_eq!(
        decode_quest_reply_49_2(&[49, 0, 2, 0, 3, 0, 0, 0, 1]),
        Err(NpcInteractionError::InvalidPacket)
    );
}

#[test]
fn quest_end_reply_49_14_uses_six_byte_header_and_quest_id() {
    let mut packet = vec![49, 0, 14, 0, 0, 0];
    packet.extend(3_000_021u32.to_le_bytes());
    assert_eq!(packet.len(), 10);
    assert_eq!(decode_quest_end_reply_49_14(&packet), Ok(3_000_021));
    assert_eq!(
        decode_quest_end_reply_49_14(&[49, 0, 14, 0, 0, 0, 1, 0, 0]),
        Err(NpcInteractionError::InvalidPacket)
    );
    assert_eq!(
        decode_quest_end_reply_49_14(&[49, 0, 14, 0, 1, 0, 1, 0, 0, 0]),
        Err(NpcInteractionError::InvalidPacket)
    );
}

#[test]
fn named_target_npc_matches_active_quest_content_and_starts_turnin() {
    let data = content_dir();
    let interaction = NpcInteraction::load(&data.join("npcs"), &data.join("stores.json"))
        .expect("NPC and store content load");
    let mut engine = QuestEngine::from_split_content(
        &data,
        7,
        CharacterEligibility {
            member: true,
            nonmember_quest_limit: 20,
            member_quest_limit: 30,
            ..Default::default()
        },
        None,
    )
    .unwrap();
    assert!(engine.offer(3_000_021, 9_000_000_189).changed);
    assert!(engine.accept(3_000_021).changed);
    assert_eq!(
        interaction.named_target_quest_for_talk(9_500_000_254, &engine),
        Some(3_000_021)
    );
    // W92 gave 3000021 an explicit talk objective for this target: the talk
    // completes it and starts the turn-in in the same outcome (QuestEnd), so a
    // second explicit begin is refused as already pending.
    let talk = engine
        .on_event(osfr_tools::quest_engine::QuestEvent::TalkToNpc {
            npc_id: 9_500_000_254,
        })
        .unwrap();
    assert!(talk.changed);
    assert!(talk.outbound.iter().any(|message| matches!(
        message,
        osfr_tools::quest_engine::OutboundMessage::Quest(
            osfr_tools::quest_engine::QuestMessage::QuestEnd {
                quest_id: 3_000_021,
                ..
            }
        )
    )));
    let reopened = engine.begin_named_target_turnin(3_000_021, 9_500_000_254);
    assert!(!reopened.changed);
    assert!(
        reopened.outbound.is_empty(),
        "one Talk must not queue a second 49/13"
    );
}

// ---------------------------------------------------------------------------------------------
// S16: one NPC router family, one private method. Each request below is a real client packet
// routed through `Gameplay::handle_in_world` (the production request path), which dispatches
// to `gameplay/npc.rs::handle_npc_packet`.
// ---------------------------------------------------------------------------------------------

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    CHARACTER, Fixture, QUEST_CHOICE, TALK_LABEL, accept, character_at_for, coins,
    content_position, dialogue_choice, has_family, interact, menu_option, npc_interact, npc_select,
    offer, pending_offer, pending_turnin, quest_abandon, quest_end_reply, quest_status, saved,
    send, talk,
};
use osfr_tools::{
    character_store::CharacterState,
    chat, game_store,
    gameplay::Gameplay,
    quest_engine::{QuestProgress, QuestStatus},
    quest_journal::BREADCRUMB_REFUSED,
};

/// The Cobblestone Village referral (`quests/adventurer.json`): giver Samantha, one Talk.
const INTRODUCE_YOURSELF: u32 = 3_000_021;
const SAMANTHA: u64 = 9_000_000_189;
const RICKY_DANGER: u64 = 9_500_000_254;
/// `npcs/cobblestone_village.json` places both NPCs in zone 3.
const COBBLESTONE_ZONE: u32 = 3;
/// A spawned (`legacy_data`) Blackspore vendor with shipped stock in `npcs/blackspore_swamp.json`
/// (zone 1). Only data-backed placements are actors the router can resolve from a runtime GUID.
const SWAMP_VENDOR: u64 = 9_500_000_007;
const SWAMP_ZONE: u32 = 1;
/// `npc_interaction::SHOP_STRING_ID`: the 26/9 label of the Shop choice.
const SHOP_LABEL: u32 = 432_156;
/// The shipped catalogue's one client-reported goal (Cooking 101's second step).
const CLIENT_GOAL_QUEST: u32 = 3_050_004;
const CLIENT_GOAL: u32 = 30_500_041;
const NOW: u64 = 30_000;

fn at_npc(npc: u64, file: &str, zone: u32) -> CharacterState {
    character_at_for(CHARACTER, zone, content_position(file, npc))
}

/// The 26/6 dialogue row choice reaches its own family and opens the 49/1 quest offer
/// (client reader `0x009f5890`: one i32 row).
#[test]
fn dialogue_26_6_choice_still_opens_the_quest_offer() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let samantha = at_npc(SAMANTHA, "cobblestone_village.json", COBBLESTONE_ZONE);
    let dialogue = talk(&mut game, &samantha, SAMANTHA, NOW);
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "the Talk must open the 26/3 dialogue"
    );
    let offered = send(
        &mut game,
        &samantha,
        &dialogue_choice(QUEST_CHOICE),
        NOW + 1,
    );
    assert_eq!(
        offered
            .replies
            .iter()
            .filter(|packet| has_family(packet, 49, 1, Some(INTRODUCE_YOURSELF)))
            .count(),
        1,
        "the 26/6 quest row must open one 49/1 offer"
    );
    assert_eq!(
        quest_status(&game, INTRODUCE_YOURSELF),
        QuestStatus::Offered
    );
    assert_eq!(pending_offer(&game), Some(INTRODUCE_YOURSELF));
}

/// 26/4 closes the open dialogue and nothing else: it answers no packet, refuses the stale
/// choice, and the NPC still opens a fresh menu afterwards (client handler `0x00aa0870`).
#[test]
fn dialogue_close_26_4_consumes_only_that_menu() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let samantha = at_npc(SAMANTHA, "cobblestone_village.json", COBBLESTONE_ZONE);
    let menu = interact(&mut game, &samantha, SAMANTHA, NOW);
    let choice = menu_option(&menu, TALK_LABEL);
    let before = saved(&game);
    let closed = send(&mut game, &samantha, &[26, 0, 4, 0], NOW + 1);
    assert!(
        closed.replies.is_empty(),
        "26/4 answers nothing: {:?}",
        closed.replies
    );
    assert!(!closed.changed);
    assert_eq!(saved(&game), before, "a 26/4 close changed gameplay state");
    let before_stale = saved(&game);
    assert!(
        game.handle_in_world(&npc_select(SAMANTHA, choice), &samantha, NOW + 2)
            .is_err(),
        "the closed menu still accepted its stale choice"
    );
    assert_eq!(
        saved(&game),
        before_stale,
        "a refused stale 26/10 changed gameplay state"
    );
    assert!(
        game.handle_in_world(&npc_select(SAMANTHA, choice), &samantha, NOW + 3)
            .is_err(),
        "a refused stale choice must not reopen the closed menu"
    );
    assert_eq!(
        saved(&game),
        before_stale,
        "a second refused stale 26/10 changed gameplay state"
    );
    let reopened = interact(&mut game, &samantha, SAMANTHA, NOW + 4);
    assert!(
        reopened
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 9, 0])),
        "26/4 consumed more than the menu: the NPC no longer answers"
    );
}

/// 45/7 only moves an open client-reported goal of the character's own active quest
/// (client writer `0x00cff9f0`: u8 subtype + one dword; the UI wrapper is `0x00cffba0`).
/// The catalogue's only such goal is Cooking 101's second step, so the first (visit) step is
/// marked done in the saved state.
#[test]
fn client_reported_45_7_only_advances_an_open_goal() {
    let fixture = Fixture::new();
    let mut state = fixture.game(None).state();
    state
        .npc_quests
        .as_mut()
        .expect("the fixture binds the split catalogue")
        .quests
        .insert(
            CLIENT_GOAL_QUEST,
            QuestProgress {
                status: QuestStatus::Active,
                counts: vec![1, 0],
            },
        );
    let mut game = fixture.game(Some(state));
    let at = character_at_for(CHARACTER, 0, [0.0; 3]);
    let before = saved(&game);
    // A forged id changes nothing.
    let mut forged = vec![45, 0, 7];
    forged.extend(999u32.to_le_bytes());
    let ignored = send(&mut game, &at, &forged, NOW);
    assert!(!ignored.changed);
    assert_eq!(saved(&game), before, "a forged 45/7 claim changed state");
    // The open goal advances, reaches the client, and the unrelated goal does not move.
    let mut claim = vec![45, 0, 7];
    claim.extend(CLIENT_GOAL.to_le_bytes());
    let advanced = send(&mut game, &at, &claim, NOW + 1);
    assert!(advanced.changed, "the open client goal did not advance");
    assert!(
        advanced
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 10, Some(CLIENT_GOAL_QUEST))),
        "the goal completion must reach the client: {:?}",
        advanced.replies
    );
    let state = game.state();
    let progress = &state.npc_quests.as_ref().unwrap().quests[&CLIENT_GOAL_QUEST];
    assert_eq!(progress.counts, vec![1, 1]);
    // A repeat changes nothing.
    let after = saved(&game);
    let repeat = send(&mut game, &at, &claim, NOW + 2);
    assert!(!repeat.changed);
    assert_eq!(saved(&game), after, "a replayed 45/7 claim changed state");
}

/// 49/15 for an id the crosswalk does not list answers one chat line and changes nothing
/// (client ctor `0x00a86580`, writer `0x00a91730`; the 47/12 and 47/13 requests share this
/// family; `tests/quest_journal.rs` pins them).
#[test]
fn journal_49_15_unlisted_id_is_refused_with_one_line() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let at = character_at_for(CHARACTER, 0, [0.0; 3]);
    let before = saved(&game);
    let mut request = vec![49, 0, 15, 0, 0, 0];
    request.extend(999_999u32.to_le_bytes());
    let refused = send(&mut game, &at, &request, NOW);
    assert!(!refused.changed);
    assert_eq!(refused.replies, vec![chat::system(BREADCRUMB_REFUSED)]);
    assert_eq!(saved(&game), before, "an unlisted 49/15 id changed state");
}

/// 49/2 accepts and 49/14 turns in through their own families (captured accept of quest
/// 3000021, run 143124; client constructors `0x00a86400` and `0x00a864c0`).
#[test]
fn quest_accept_49_2_and_turn_in_49_14_still_route() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let samantha = at_npc(SAMANTHA, "cobblestone_village.json", COBBLESTONE_ZONE);
    offer(&mut game, &samantha, SAMANTHA, INTRODUCE_YOURSELF, NOW);
    accept(&mut game, &samantha, INTRODUCE_YOURSELF, NOW + 2);
    assert_eq!(quest_status(&game, INTRODUCE_YOURSELF), QuestStatus::Active);
    let ricky = at_npc(RICKY_DANGER, "cobblestone_village.json", COBBLESTONE_ZONE);
    let talked = talk(&mut game, &ricky, RICKY_DANGER, NOW + 3);
    assert!(
        talked.changed,
        "the referral Talk did not complete the goal"
    );
    assert_eq!(pending_turnin(&game), Some(INTRODUCE_YOURSELF));
    let coins_before = coins(&game);
    let completed = send(
        &mut game,
        &ricky,
        &quest_end_reply(INTRODUCE_YOURSELF),
        NOW + 4,
    );
    assert!(completed.changed);
    assert!(
        completed
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 4, Some(INTRODUCE_YOURSELF))),
        "the turn-in must send its 49/4 completion: {:?}",
        completed.replies
    );
    assert_eq!(
        quest_status(&game, INTRODUCE_YOURSELF),
        QuestStatus::Completed
    );
    assert_eq!(pending_turnin(&game), None);
    assert_eq!(
        coins(&game),
        coins_before + 5,
        "the 49/14 turn-in must pay the content reward"
    );
}

/// The real save envelope: `game_store::save` -> `load_or_create` -> the login bind. The
/// snapshot comes from `load_or_create`, not a struct literal, so new envelope fields keep it.
fn store_reload(fixture: &Fixture, game: &Gameplay, at: &CharacterState) -> Gameplay {
    let path = fixture.path().join("abandon-save.json");
    let mut snapshot = game_store::load_or_create(&path, at.clone()).unwrap();
    snapshot.gameplay = Some(game.state());
    game_store::save(&path, &snapshot).unwrap();
    fixture.game(
        game_store::load_or_create(&path, at.clone())
            .unwrap()
            .gameplay,
    )
}

/// `guid` has accepted Introduce Yourself and stands at its giver.
fn active_quest(fixture: &Fixture, guid: u64) -> (Gameplay, CharacterState) {
    let mut game = fixture.game_as(guid, None);
    let position = content_position("cobblestone_village.json", SAMANTHA);
    let samantha = character_at_for(guid, COBBLESTONE_ZONE, position);
    offer(&mut game, &samantha, SAMANTHA, INTRODUCE_YOURSELF, NOW);
    accept(&mut game, &samantha, INTRODUCE_YOURSELF, NOW + 2);
    (game, samantha)
}

/// The 49/6 `QuestAbandoned` reply: client reader `0x00c7ae40` (exact length via
/// `0x00c7b500`) reads u16 49 + u32 6, the u32 id, then a bool.
fn abandoned_reply(quest_id: u32) -> Vec<u8> {
    [&[49u8, 0, 6, 0, 0, 0][..], &quest_id.to_le_bytes(), &[0]].concat()
}

/// 26/23 is the client's 8-byte body (`CommandPacketQuestAbandon`: ctor `0x00c09f60` stores
/// 26/23 via `0x008a2820` and the id at `this+0x10`; writer `0x00c0a350` runs the preamble
/// `0x0090ac70`, u16 26 + u16 23, then appends the u32 id; sender `0x00c0a3d0`, Lua
/// `Quest_AbandonQuest` `0x00c0a500`). It replays the native 49/6 and the save keeps it.
#[test]
fn a_real_client_eight_byte_abandon_is_decoded_and_removes_the_quest() {
    let fixture = Fixture::new();
    let (mut game, samantha) = active_quest(&fixture, CHARACTER);
    // Introduce Yourself is 3_000_021 = 0x002dc6d5.
    let request = [26, 0, 23, 0, 0xd5, 0xc6, 0x2d, 0x00];
    assert_eq!(quest_abandon(INTRODUCE_YOURSELF), request);
    let abandoned = send(&mut game, &samantha, &request, NOW + 3);
    assert_eq!(abandoned.replies, vec![abandoned_reply(INTRODUCE_YOURSELF)]);
    assert!(abandoned.changed);
    assert_eq!(
        quest_status(&game, INTRODUCE_YOURSELF),
        QuestStatus::Abandoned
    );
    let reloaded = store_reload(&fixture, &game, &samantha);
    assert_eq!(
        quest_status(&reloaded, INTRODUCE_YOURSELF),
        QuestStatus::Abandoned,
        "the saved character must keep the abandon"
    );
}

/// The old 10-byte shape was a test fixture, not a client shape: it and every other length
/// of family 26/23 are refused before anything changes, and the open NPC menu survives.
#[test]
fn the_old_ten_byte_abandon_and_other_lengths_are_refused_without_mutation() {
    let fixture = Fixture::new();
    let (mut game, samantha) = active_quest(&fixture, CHARACTER);
    let menu = interact(&mut game, &samantha, SAMANTHA, NOW + 3);
    let choice = menu_option(&menu, TALK_LABEL);
    let before = saved(&game);
    let real = quest_abandon(INTRODUCE_YOURSELF);
    let old = [
        &[26u8, 0, 23, 0, 0, 0][..],
        &INTRODUCE_YOURSELF.to_le_bytes(),
    ]
    .concat();
    for packet in [
        old,
        real[..6].to_vec(),
        real[..7].to_vec(),
        [&real[..], &[0]].concat(),
    ] {
        assert!(
            game.handle_in_world(&packet, &samantha, NOW + 4).is_err(),
            "{packet:?} must be refused"
        );
        assert_eq!(saved(&game), before, "{packet:?} changed state");
    }
    assert_eq!(quest_status(&game, INTRODUCE_YOURSELF), QuestStatus::Active);
    let talked = send(&mut game, &samantha, &npc_select(SAMANTHA, choice), NOW + 5);
    assert!(
        talked
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "a refused 26/23 consumed the open NPC menu"
    );
}

/// The request carries only a quest id; the engine reads the sender's own quests. A forged,
/// completed or other character's id answers nothing and changes neither save, and each
/// player's real abandon touches only their own quest.
#[test]
fn an_unknown_completed_or_other_characters_quest_is_refused_without_mutation() {
    let fixture = Fixture::new();
    let (mut other, other_at) = active_quest(&fixture, CHARACTER + 1);
    let other_before = saved(&other);
    let (mut game, samantha) = active_quest(&fixture, CHARACTER);
    let ricky = at_npc(RICKY_DANGER, "cobblestone_village.json", COBBLESTONE_ZONE);
    let talked = talk(&mut game, &ricky, RICKY_DANGER, NOW + 3);
    assert!(
        talked.changed,
        "the referral Talk did not complete the goal"
    );
    send(
        &mut game,
        &ricky,
        &quest_end_reply(INTRODUCE_YOURSELF),
        NOW + 5,
    );
    assert_eq!(
        quest_status(&game, INTRODUCE_YOURSELF),
        QuestStatus::Completed
    );
    for id in [INTRODUCE_YOURSELF, 0, u32::MAX, CLIENT_GOAL_QUEST] {
        let before = saved(&game);
        let refused = send(&mut game, &samantha, &quest_abandon(id), NOW + 6);
        assert!(refused.replies.is_empty(), "{id}: {:?}", refused.replies);
        assert!(!refused.changed, "{id} changed gameplay");
        assert_eq!(saved(&game), before, "{id} changed the save");
    }
    assert_eq!(saved(&other), other_before, "the other player changed");
    let before = saved(&game);
    let abandoned = send(
        &mut other,
        &other_at,
        &quest_abandon(INTRODUCE_YOURSELF),
        NOW + 7,
    );
    assert_eq!(abandoned.replies, vec![abandoned_reply(INTRODUCE_YOURSELF)]);
    assert_eq!(
        quest_status(&other, INTRODUCE_YOURSELF),
        QuestStatus::Abandoned
    );
    assert_eq!(saved(&game), before, "one abandon touched the other player");
}

/// A replayed 26/23, before or after a save round trip, sends no second 49/6.
#[test]
fn a_replayed_abandon_changes_nothing() {
    let fixture = Fixture::new();
    let (mut game, samantha) = active_quest(&fixture, CHARACTER);
    let request = quest_abandon(INTRODUCE_YOURSELF);
    assert!(send(&mut game, &samantha, &request, NOW + 3).changed);
    let mut reloaded = store_reload(&fixture, &game, &samantha);
    for game in [&mut game, &mut reloaded] {
        let before = saved(game);
        let replay = send(game, &samantha, &request, NOW + 4);
        assert!(replay.replies.is_empty(), "{:?}", replay.replies);
        assert!(!replay.changed);
        assert_eq!(saved(game), before);
    }
}

/// 26/8 opens the 26/9 menu and 26/10 selects from it (client constructors
/// `0x008b6bc0` and `0x00a36f80`); an unknown target and a malformed selection keep their
/// existing outcomes.
#[test]
fn interact_and_select_26_8_26_10_still_route() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let samantha = at_npc(SAMANTHA, "cobblestone_village.json", COBBLESTONE_ZONE);
    let menu = interact(&mut game, &samantha, SAMANTHA, NOW);
    let packet = menu
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 9, 0]))
        .expect("26/8 reaches the 26/9 menu builder");
    let count = i32::from_le_bytes(packet[13..17].try_into().unwrap());
    assert!(count > 0, "the 26/9 menu has no options");
    let talk_choice = menu_option(&menu, TALK_LABEL);
    let dialogue = send(
        &mut game,
        &samantha,
        &npc_select(SAMANTHA, talk_choice),
        NOW + 1,
    );
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0]))
    );
    // A target the content does not know belongs to no family: both fall through silently.
    let unknown = u64::MAX;
    let before = saved(&game);
    let missed = send(&mut game, &samantha, &npc_interact(unknown), NOW + 2);
    assert!(!missed.changed && missed.replies.is_empty());
    let missed = send(&mut game, &samantha, &npc_select(unknown, 1), NOW + 3);
    assert!(!missed.changed && missed.replies.is_empty());
    assert_eq!(saved(&game), before);
    let before_malformed = saved(&game);
    assert!(
        game.handle_in_world(&[26, 0, 10, 0, 0, 0], &samantha, NOW + 4)
            .is_err(),
        "a malformed selection must stay that family's refusal"
    );
    assert_eq!(
        saved(&game),
        before_malformed,
        "a malformed 26/10 changed gameplay state"
    );
    // The refused selection consumed nothing: the open menu still answers.
    let again = send(
        &mut game,
        &samantha,
        &npc_select(SAMANTHA, talk_choice),
        NOW + 5,
    );
    assert!(
        again
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "a malformed 26/10 consumed the open menu"
    );
}

/// 165/4 and 66/43 keep their shop binding checks: both reach their own family, answer the
/// client, and a refusal leaves the open shop and the inventory untouched.
#[test]
fn vendor_165_4_and_66_43_still_route() {
    let fixture = Fixture::new();
    // 100 coins covers the 15-coin price of item 8, so only the vendor's stock gate can
    // refuse the buy below (a stock-check regression would succeed).
    let mut state = fixture.game(None).state();
    state.inventory.coins = 100;
    let mut game = fixture.game(Some(state));
    let vendor = at_npc(SWAMP_VENDOR, "blackspore_swamp.json", SWAMP_ZONE);
    let menu = interact(&mut game, &vendor, SWAMP_VENDOR, NOW);
    let shop_choice = menu_option(&menu, SHOP_LABEL);
    let opened = send(
        &mut game,
        &vendor,
        &npc_select(SWAMP_VENDOR, shop_choice),
        NOW + 1,
    );
    assert!(
        opened
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[165, 0, 10, 0])),
        "the Shop choice must open the 165/10 coin merchant window"
    );
    let before = saved(&game);
    // 165/4 for an item this vendor does not stock: the family refuses and nothing moves.
    let buy = request_buy(SWAMP_VENDOR, 7, 8, 0, 1);
    assert!(
        game.handle_in_world(&buy, &vendor, NOW + 2).is_err(),
        "a non-stock 165/4 buy must be refused"
    );
    assert_eq!(saved(&game), before, "a refused 165/4 buy changed state");
    // 66/43 for the same item answers one 66/44 refusal (status dword at 48 = 0).
    let buy = encode_merchant_buy_66_43(&MerchantBuyRequestV2 {
        initial_dwords: [8, 0],
        context_dwords: [0; 3],
        context_bytes: Vec::new(),
        trailing_dwords: [1, 0, 7],
    })
    .unwrap();
    let refused = send(&mut game, &vendor, &buy, NOW + 3);
    assert!(!refused.changed);
    let status = refused
        .replies
        .last()
        .expect("66/43 answers a 66/44 result");
    assert_eq!(status.len(), 56, "the client reader needs the full record");
    assert_eq!(&status[..4], &[66, 0, 44, 0]);
    assert_ne!(
        u32::from_le_bytes(status[48..52].try_into().unwrap()),
        1,
        "a refused buy must not report success"
    );
    assert_eq!(saved(&game), before, "a refused 66/43 buy changed state");
    // A body that does not decode is answered by this family, not by the fall-through.
    let malformed = send(&mut game, &vendor, &[66, 0, 43, 0], NOW + 4);
    assert!(!malformed.changed);
    assert!(
        malformed
            .replies
            .last()
            .is_some_and(|packet| packet.starts_with(&[66, 0, 44, 0]))
    );
    assert_eq!(saved(&game), before);
}

/// Menus are per session: another player's refused choice consumes nothing and changes
/// neither player's saved state.
#[test]
fn a_two_session_refusal_does_not_change_the_other_players_state() {
    let fixture = Fixture::new();
    let mut first = fixture.game(None);
    let mut second = fixture.game(None);
    let samantha = at_npc(SAMANTHA, "cobblestone_village.json", COBBLESTONE_ZONE);
    let menu = interact(&mut first, &samantha, SAMANTHA, NOW);
    let choice = menu_option(&menu, TALK_LABEL);
    let first_before = saved(&first);
    let second_before = saved(&second);
    // The second session never opened this menu: its choice is refused.
    assert!(
        second
            .handle_in_world(&npc_select(SAMANTHA, choice), &samantha, NOW + 1)
            .is_err(),
        "a session without the menu must not read it"
    );
    assert_eq!(saved(&second), second_before);
    assert_eq!(
        saved(&first),
        first_before,
        "the other session's refusal changed this player's state"
    );
    // The first session's menu still answers.
    let talked = send(
        &mut first,
        &samantha,
        &npc_select(SAMANTHA, choice),
        NOW + 2,
    );
    assert!(
        talked
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0]))
    );
    assert_eq!(saved(&second), second_before);
}
