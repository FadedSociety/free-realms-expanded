use osfr_tools::npc_interaction::{
    DialoguePacket26_3, MerchantBuyRequestV2, MerchantEntryV2, MerchantGroupV2, MerchantListV2,
    NpcAction, NpcInteraction, NpcInteractionError, NpcSelection, QuestOfferPacket49_1,
    decode_dialogue_26_3, decode_dialogue_response_26_6, decode_merchant_list_66_42,
    decode_npc_interact_26_8, decode_quest_end_reply_49_14, decode_quest_offer_49_1,
    decode_quest_reply_49_2, encode_dialogue_26_3, encode_dialogue_close_26_4,
    encode_merchant_buy_66_43, encode_merchant_list_66_42, encode_quest_offer_49_1,
    goodbye_dialogue_choice,
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
    CoinShop::new(directory, inventory, catalogue, usize::MAX).expect("coin shop loads")
}

fn count_definition(state: &InventoryState, definition_id: i32) -> u32 {
    state
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id)
        .map(|item| item.count)
        .sum()
}

fn request_buy(merchant_id: u64) -> Vec<u8> {
    let mut packet = vec![165, 0, 4, 0];
    packet.extend(merchant_id.to_le_bytes());
    packet.extend(77001u32.to_le_bytes());
    packet.extend(8i32.to_le_bytes());
    packet.extend(1u32.to_le_bytes());
    packet.extend(0u32.to_le_bytes());
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
        .handle_coin_store(&mut shop, &request_buy(VENDOR_ID), zone, position)
        .unwrap();
    assert_eq!(bought.result_code, coin_shop::RESULT_SUCCESS);
    assert_eq!(shop.inventory().state().coins, 85);
    assert_eq!(
        count_definition(shop.inventory().state(), 8),
        original_count + 1
    );
    let after_buy = shop.inventory().state().clone();
    let retry = service
        .handle_coin_store(&mut shop, &request_buy(VENDOR_ID), zone, position)
        .unwrap();
    assert!(!retry.changed);
    assert_eq!(shop.inventory().state(), &after_buy);

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
    assert_eq!(shop.inventory().state().coins, 92);
    assert_eq!(
        count_definition(shop.inventory().state(), 8),
        original_count
    );

    assert_eq!(service.close(VENDOR_ID).unwrap(), [66, 0, 46, 0]);
    assert!(matches!(
        service.handle_coin_store(&mut shop, &request_buy(VENDOR_ID), zone, position),
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
    assert_eq!(dialogue.body_string_id, None);
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

#[test]
fn merchant_list_66_42_matches_the_client_reader_layout() {
    // Client reader (`00b3d5f0`): the header, one dword, a six-dword record (`00b33020`), the
    // entry count (`00b2e620`), then 38-byte rows. Synthetic values, no captured frame exists.
    let message = MerchantListV2 {
        initial_i16s: [0, 0],
        initial_dword: 99,
        group: MerchantGroupV2 {
            fields: [1, 2, 3, 4, 5, 6, 0],
            entries: vec![MerchantEntryV2 {
                first_six_dwords: [8, 9, 10, 11, 12, 13],
                flag_after_six: true,
                final_three_dwords: [14, 15, 16],
                final_flag: false,
            }],
        },
    };
    let encoded = encode_merchant_list_66_42(&message).unwrap();
    assert_eq!(&encoded[..8], &[66, 0, 42, 0, 99, 0, 0, 0]);
    assert_eq!(encoded.len(), 4 + 4 + 24 + 4 + 38);
    assert_eq!(&encoded[32..36], &1u32.to_le_bytes());
    assert_eq!(decode_merchant_list_66_42(&encoded).unwrap(), message);
    // Off-wire legacy fields cannot be smuggled in, and the old 8-byte-longer shape is refused.
    let mut legacy = message.clone();
    legacy.initial_i16s = [1, 0];
    assert!(encode_merchant_list_66_42(&legacy).is_err());
    let mut legacy = message.clone();
    legacy.group.fields[6] = 1;
    assert!(encode_merchant_list_66_42(&legacy).is_err());
    let mut extended = encoded.clone();
    extended.splice(4..4, [0u8; 4]);
    assert!(decode_merchant_list_66_42(&extended).is_err());
    assert!(decode_merchant_list_66_42(&encoded[..encoded.len() - 1]).is_err());
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
                quest_id: 3_000_021
            }
        )
    )));
    assert!(!engine.begin_named_target_turnin(3_000_021).changed);
}
