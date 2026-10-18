use super::*;

fn reference() -> PathBuf {
    bundled_content()
}

fn bundled_content() -> PathBuf {
    PathBuf::from(
        std::env::var_os("OSFR_W95_TEST_CONTENT").expect("private vendor content required"),
    )
}

fn count_definition(game: &Gameplay, definition_id: i32) -> u32 {
    game.inventory
        .state()
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id)
        .map(|item| item.count)
        .sum()
}

#[test]
#[ignore = "requires OSFR_W95_TEST_CONTENT pointing at private play content"]
fn coin_store_and_vendor_buys_preserve_live_inventory_and_saved_state() {
    let reference = reference();
    let mut game = Gameplay::load(&reference, None).expect("gameplay loads");
    let mut state = game.state();
    state.inventory.items.clear();
    for equipped in &mut state.inventory.equipment {
        equipped.item_id = 0;
    }
    state.inventory.coins = 1_000_000;
    game = Gameplay::load(&reference, Some(state)).expect("prepared login state loads");

    // Simulate a reward granted after login while CoinShop still owns its login snapshot.
    game.inventory
        .grant(0, 8, 1)
        .expect("post-login item grant succeeds");
    assert_eq!(count_definition(&game, 8), 1);

    let content = bundled_content();
    let vendors: serde_json::Value = serde_json::from_slice(
        &std::fs::read(content.join("vendors.json")).expect("vendor content reads"),
    )
    .expect("vendor content parses");
    let stores: serde_json::Value = serde_json::from_slice(
        &std::fs::read(content.join("stores.json")).expect("store content reads"),
    )
    .expect("store content parses");
    let mut selected = None;
    for vendor in vendors["vendors"].as_array().expect("vendors are an array") {
        let npc_id = vendor["npc_id"].as_u64().expect("NPC id is an integer");
        let Some((zone_id, position)) = game
            .npc_interaction
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .location(npc_id)
        else {
            continue;
        };
        for item in vendor["item_ids"].as_array().expect("stock is an array") {
            let Some(item_id) = item.as_i64().and_then(|id| i32::try_from(id).ok()) else {
                continue;
            };
            if item_id == 8 || !game.inventory.has_definition(item_id) {
                continue;
            }
            let Some(price) = stores["items"]
                .as_array()
                .expect("store items are an array")
                .iter()
                .find(|row| row["item_id"].as_i64() == Some(i64::from(item_id)))
                .and_then(|row| row["purchase_price_coins"].as_u64())
            else {
                continue;
            };
            selected = Some((npc_id, zone_id, position, item_id, price as u32));
            break;
        }
        if selected.is_some() {
            break;
        }
    }
    let (npc_id, zone_id, position, item_id, price) =
        selected.expect("a priced, defined vendor item exists");

    let mut interaction = game.npc_interaction.as_ref().unwrap().lock().unwrap();
    let menu = interaction
        .interact(npc_id, zone_id, position, None)
        .expect("vendor interaction opens");
    let shop_option = menu
        .options
        .iter()
        .find(|option| option.action == npc_interaction::NpcAction::Shop)
        .expect("vendor has a shop option");
    let mut select = vec![26, 0, 10, 0];
    select.extend(npc_id.to_le_bytes());
    select.extend(shop_option.id.to_le_bytes());
    interaction
        .select(&select, zone_id, position, None)
        .expect("vendor shop opens");
    drop(interaction);

    let coins_before = game.inventory.state().coins;
    let purchased_before = count_definition(&game, item_id);
    let mut request = vec![165, 0, 4, 0];
    request.extend(npc_id.to_le_bytes());
    request.extend(99001u32.to_le_bytes());
    request.extend(item_id.to_le_bytes());
    request.extend(1u32.to_le_bytes());
    request.extend(0u32.to_le_bytes());
    let character = CharacterState {
        guid: 7,
        zone_id,
        position,
        rotation: [0.0; 3],
    };
    let outcome = game
        .handle_in_world(&request, &character, 1_000)
        .expect("vendor Coin Store request routes");

    assert!(outcome.changed, "successful purchase requests a save");
    assert_eq!(game.inventory.state().coins, coins_before - price);
    assert_eq!(count_definition(&game, item_id), purchased_before + 1);
    assert_eq!(count_definition(&game, 8), 1, "post-login reward survives");

    let vendor_request =
        npc_interaction::encode_merchant_buy_66_43(&npc_interaction::MerchantBuyRequestV2 {
            initial_dwords: [item_id as u32, 0],
            context_dwords: [0; 3],
            context_bytes: Vec::new(),
            trailing_dwords: [1, 0, 99002],
        })
        .expect("vendor request encodes");
    let vendor_outcome = game
        .handle_in_world(&vendor_request, &character, 2_000)
        .expect("66/43 vendor request routes");
    assert!(vendor_outcome.changed);
    let response = vendor_outcome
        .replies
        .last()
        .expect("vendor result reply exists");
    assert_eq!(response.len(), 56);
    assert_eq!(&response[..4], &[66, 0, 44, 0]);
    assert_eq!(u32::from_le_bytes(response[48..52].try_into().unwrap()), 1);
    assert_eq!(game.inventory.state().coins, coins_before - (price * 2));
    assert_eq!(count_definition(&game, item_id), purchased_before + 2);
    assert_eq!(
        count_definition(&game, 8),
        1,
        "post-login reward still survives"
    );

    let before_refusal = game.state();
    let unknown_request =
        npc_interaction::encode_merchant_buy_66_43(&npc_interaction::MerchantBuyRequestV2 {
            initial_dwords: [i32::MAX as u32, 0],
            context_dwords: [0; 3],
            context_bytes: Vec::new(),
            trailing_dwords: [1, 1, 99003],
        })
        .expect("unknown-item request encodes");
    let refusal = game
        .handle_in_world(&unknown_request, &character, 3_000)
        .expect("unknown vendor item gets a refusal");
    assert!(!refusal.changed);
    assert_eq!(game.state(), before_refusal);
    let response = refusal.replies.last().expect("refusal reply exists");
    assert_eq!(response.len(), 56);
    assert_eq!(&response[..4], &[66, 0, 44, 0]);
    assert_eq!(u32::from_le_bytes(response[48..52].try_into().unwrap()), 0);

    let saved = serde_json::to_vec(&game.state()).expect("changed state serializes");
    let saved: GameplayState = serde_json::from_slice(&saved).expect("saved state parses");
    let restored = Gameplay::load(&reference, Some(saved)).expect("saved state reloads");
    assert_eq!(restored.inventory.state().coins, coins_before - (price * 2));
    assert_eq!(count_definition(&restored, item_id), purchased_before + 2);
    assert_eq!(count_definition(&restored, 8), 1);
}
