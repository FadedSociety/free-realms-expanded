//! P-games-window: the Marketplace panel on a minigame offer, through the real `Gameplay` route.
//! Play on a Games window row (167/1/2) makes the 39/16 offer; the commit tap
//! (`Gameplay::dress_offers`, called from the gateway's `commit_outcome`) adds 39/44; a Station
//! Cash order for an advertised booster is taken while the offer is open and used at once; the
//! run then pays the raised coins once. Runs on the repository's own `data/release` content.
use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, Outcome},
    offer_boosts::{StorePush, Table, Watch, ad_packet},
    station_cash::Wallet,
};
use serde_json::Value;
use std::path::PathBuf;

const SUPER_STAR: u32 = 78;
const SUPER_COIN: u32 = 81;
/// 200 SC at the shipped `sc_price_scale` of 0.25.
const PRICE: u32 = 50;
/// Spot the Difference's Games window category.
const SPOT: u64 = 24;

fn data() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

/// The release content: its `item-table.json` defines every item, the boosters among them.
fn content() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/release")
}

fn json(name: &str) -> Value {
    serde_json::from_slice(&std::fs::read(data().join(name)).unwrap()).unwrap()
}

fn who() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: [-733.952, 9.09, 945.973],
        rotation: [0.0, 0.0, 1.0],
    }
}

/// An ordinary player in the world with `balance` Station Cash.
fn game(balance: u32) -> Gameplay {
    let mut state = Gameplay::load(&content(), None).unwrap().state();
    state.sc_wallet = Some(Wallet { balance, ..Wallet::default() });
    let mut game = Gameplay::load(&content(), Some(state)).unwrap();
    game.initialize_world(&who(), 500).unwrap();
    game
}

fn word(packet: &[u8], at: usize) -> i32 {
    i32::from_le_bytes(packet[at..at + 4].try_into().unwrap())
}

fn find<'a>(packets: &'a [Vec<u8>], head: &[u8]) -> Option<&'a Vec<u8>> {
    packets.iter().find(|packet| packet.starts_with(head))
}

/// The gateway's tap, on inner packets, for a commit that entered no world.
fn dress(game: &mut Gameplay, out: &mut Outcome) {
    game.dress_offers(&mut out.replies, 0, <[u8]>::to_vec, false);
}

/// Play on the first Spot the Difference row; the dressed outcome and the offer's instance.
fn offer(game: &mut Gameplay) -> (Outcome, i32) {
    let catalogue = json("activity_catalogue.json");
    let row = (catalogue["rows"].as_array().unwrap().iter())
        .find(|row| row["category"] == SPOT)
        .unwrap();
    let mut play = vec![167, 0, 1, 2];
    play.extend((row["id"].as_u64().unwrap() as u32).to_le_bytes());
    play.extend(0u32.to_le_bytes());
    let mut out = game.handle_in_world(&play, &who(), 800).unwrap();
    dress(game, &mut out);
    let instance = word(find(&out.replies, &[39, 0, 16]).expect("the 39/16 offer"), 3);
    (out, instance)
}

fn put_string(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend((value.len() as i32).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

/// The client's order (`00b3fc60`/`00b3af80`), as `tests/marketplace_owned_rides.rs` writes it.
fn buy(tracking: u32, bundle: u32) -> Vec<u8> {
    let mut bytes = vec![66, 0, 3, 0];
    put_string(&mut bytes, "");
    put_string(&mut bytes, "");
    bytes.extend(tracking.to_le_bytes());
    put_string(&mut bytes, "en_US");
    for _ in 0..3 {
        put_string(&mut bytes, "");
    }
    for value in [1, bundle, 1, 1] {
        bytes.extend(value.to_le_bytes());
    }
    put_string(&mut bytes, "0");
    put_string(&mut bytes, "");
    bytes
}

/// 66/4's result (1 is success).
fn order_result(out: &Outcome) -> i32 {
    word(find(&out.replies, &[66, 0, 4, 0]).expect("a 66/4 order response"), 8)
}

fn lifecycle(sub: u8, instance: i32) -> Vec<u8> {
    let mut packet = vec![39, 0, sub];
    for value in [instance, -1, -1] {
        packet.extend(value.to_le_bytes());
    }
    packet
}

/// The movie's 39/14 message.
fn say(game: &mut Gameplay, instance: i32, text: &str, now_ms: u64) -> Outcome {
    let mut packet = lifecycle(14, instance);
    packet.extend((text.len() as u32 + 1).to_le_bytes());
    packet.extend(text.as_bytes());
    packet.push(0);
    let mut out = game.handle_in_world(&packet, &who(), now_ms).unwrap();
    dress(game, &mut out);
    out
}

/// Go, a full game and the win; the coins it paid.
fn win(game: &mut Gameplay, instance: i32) -> u32 {
    let before = game.inventory_state().coins;
    game.handle_in_world(&lifecycle(5, instance), &who(), 1_000).unwrap();
    say(game, instance, "FRServer_RequestStageId", 2_000);
    for _ in 0..25 {
        say(game, instance, "FRServer_StatUpdate\t22\t1", 6_000);
    }
    let out = say(game, instance, "FRServer_GameWon\t90000", 61_000);
    assert!(find(&out.replies, &[39, 0, 18]).is_some(), "the result goes out");
    // A second win message pays nothing more.
    say(game, instance, "FRServer_GameWon\t90000", 62_000);
    u32::try_from(game.inventory_state().coins - before).unwrap()
}

#[test]
fn the_shipped_ads_are_marketplace_boosters() {
    let table = Table::load(&data()).unwrap().expect("offer_boosts.json ships");
    assert!(table.enabled && table.store_push == StorePush::FirstOffer);
    let ads = table.ads(1040, 2, &[]);
    assert_eq!(ads, [SUPER_STAR, SUPER_COIN]);
    let stores = json("stores.json");
    let consumables = json("consumables.json");
    for id in ads {
        let row = (stores["items"].as_array().unwrap().iter())
            .find(|row| row["item_id"] == id)
            .expect("an advertised item is a store row");
        assert!(row["sc_price"].as_u64() > Some(0) && row["earn_only"] == false);
        let effect = (consumables["consumables"].as_array().unwrap().iter())
            .find(|row| row["definition"] == id)
            .expect("an advertised booster has its effect");
        assert!(effect["effect"]["boost"]["percent"].as_u64() > Some(0));
    }
}

#[test]
fn the_advertisement_matches_the_client_reader() {
    // `009b5f10`: the 15-byte header, an i32, then `009b0700`'s count and ids.
    let packet = ad_packet(7, 1040, 1040, &[78, 81]);
    let mut expected = vec![39, 0, 44];
    for value in [7, -1, 1040, 1040, 2, 78, 81] {
        expected.extend(i32::to_le_bytes(value));
    }
    assert_eq!(packet, expected);
}

#[test]
fn an_event_brings_its_own_ads_and_a_filter_narrows_them() {
    let mut value = json("offer_boosts.json");
    value["event_ads"]["snow_days"] =
        serde_json::json!([{"item_id": 500, "game_types": [9]}, {"item_id": 81}]);
    let scratch = std::env::temp_dir().join(format!("osfr-offer-boosts-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    std::fs::write(scratch.join("offer_boosts.json"), value.to_string()).unwrap();
    let table = Table::load(&scratch).unwrap().unwrap();
    let live = ["snow_days".to_owned()];
    assert_eq!(table.ads(1, 9, &live), [500, 81, 78], "the event's ads lead, each id once");
    assert_eq!(table.ads(1, 2, &live), [81, 78], "a type filter leaves the other games out");
    assert_eq!(table.ads(1, 9, &[]), [78, 81], "no event, no event ads");

    // The watch: rows once and ahead of the first offer, the panel after each offer, unlisted
    // ids dropped, and the offer closed by its start.
    let mut first = serde_json::from_str::<Value>(&value.to_string()).unwrap();
    first["store_push"] = "first_offer".into();
    std::fs::write(scratch.join("offer_boosts.json"), first.to_string()).unwrap();
    let mut watch = Watch::load(&scratch).unwrap();
    std::fs::remove_dir_all(&scratch).unwrap();
    let offer = |instance: i32| {
        let mut packet = vec![39, 0, 16];
        for v in [instance, -1, 1040, 0, 0, 0, 1, 0, 2] {
            packet.extend(i32::to_le_bytes(v));
        }
        packet
    };
    let rows = || vec![vec![66, 0, 5, 0]];
    let mut replies = vec![vec![15, 0], offer(3)];
    watch.dress(&mut replies, 0, <[u8]>::to_vec, true, |id| id != 78, rows);
    assert_eq!(
        replies,
        [vec![15, 0], vec![66, 0, 5, 0], offer(3), ad_packet(3, 1040, 1040, &[81])]
    );
    assert!(watch.advertises([81].into_iter()) && !watch.advertises([78].into_iter()));
    let mut replies = vec![lifecycle(17, 3), offer(4)];
    watch.dress(&mut replies, 0, <[u8]>::to_vec, true, |_| true, rows);
    assert_eq!(replies.len(), 3, "the rows went out once");
    let mut replies = vec![lifecycle(17, 4)];
    watch.dress(&mut replies, 0, <[u8]>::to_vec, true, |_| true, rows);
    assert!(!watch.advertises([81].into_iter()), "a started run is no longer an offer");
}

#[test]
fn a_real_offer_carries_the_panel_after_the_marketplace_rows() {
    let mut game = game(0);
    // Shipped as `first_offer`: entering the world adds nothing to the login.
    let mut ready = Outcome::default();
    game.dress_offers(&mut ready.replies, 0, <[u8]>::to_vec, true);
    assert!(ready.replies.is_empty(), "no Marketplace rows at world entry");
    let (out, instance) = offer(&mut game);
    let at = out.replies.iter().position(|p| p.starts_with(&[39, 0, 16])).unwrap();
    let rows = out.replies.iter().position(|p| p.starts_with(&[66, 0, 5, 0])).expect("the store rows");
    assert!(rows < at, "the rows lead the first offer");
    println!("marketplace rows ahead of the first offer: {} bytes", out.replies[rows].len());
    let game_id = word(&out.replies[at], 11);
    assert_eq!(
        out.replies[at + 1],
        ad_packet(instance, game_id, game_id, &[SUPER_STAR, SUPER_COIN]),
        "39/44 follows its 39/16"
    );
}

#[test]
fn world_ready_pushes_the_rows_with_the_entry_commit_only() {
    let mut value = json("offer_boosts.json");
    value["store_push"] = "world_ready".into();
    let scratch = std::env::temp_dir().join(format!("osfr-offer-ready-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    std::fs::write(scratch.join("offer_boosts.json"), value.to_string()).unwrap();
    let mut watch = Watch::load(&scratch).unwrap();
    std::fs::remove_dir_all(&scratch).unwrap();
    let rows = || vec![vec![66, 0, 5, 0]];
    let mut replies = vec![vec![15, 0]];
    watch.dress(&mut replies, 0, <[u8]>::to_vec, false, |_| true, rows);
    assert_eq!(replies, [vec![15, 0]], "a commit that entered no world carries no rows");
    watch.dress(&mut replies, 0, <[u8]>::to_vec, true, |_| true, rows);
    assert_eq!(replies, [vec![15, 0], vec![66, 0, 5, 0]]);
    watch.dress(&mut replies, 0, <[u8]>::to_vec, true, |_| true, rows);
    assert_eq!(replies.len(), 2, "once per session");
    watch.reset();
    watch.dress(&mut replies, 0, <[u8]>::to_vec, true, |_| true, rows);
    assert_eq!(replies.len(), 3, "a new session gets them again");
}

#[test]
fn go_or_leave_from_the_client_closes_the_offer_without_any_reply_seen() {
    // Go, with the game's answer never passing the commit tap: the offer is closed all the same.
    let mut game = game(10_000);
    let (_, instance) = offer(&mut game);
    game.handle_in_world(&lifecycle(5, instance), &who(), 1_000).unwrap();
    let late = game.handle_in_world(&buy(1, SUPER_STAR), &who(), 1_100).unwrap();
    assert_ne!(order_result(&late), 1, "a started game sells nothing");
    assert_eq!(game.consumable_effects().boost().stars, 0);

    // Leave: whatever the order then does, it is no offer purchase, so nothing is used.
    let mut game = self::game(10_000);
    let (_, instance) = offer(&mut game);
    game.handle_in_world(&lifecycle(6, instance), &who(), 1_000).unwrap();
    let after = game.handle_in_world(&buy(1, SUPER_COIN), &who(), 1_100).unwrap();
    assert!(find(&after.replies, &[38, 0, 16]).is_none(), "no booster starts by itself");
    assert_eq!(game.consumable_effects().boost().coins, 0);
}

#[test]
fn a_booster_bought_at_the_offer_raises_that_run_once() {
    let mut plain = game(0);
    let (_, instance) = offer(&mut plain);
    let base = win(&mut plain, instance);
    assert!(base > 0, "a Spot the Difference win pays coins");

    let mut boosted = game(PRICE);
    let (_, instance) = offer(&mut boosted);
    let bought = boosted.handle_in_world(&buy(1, SUPER_COIN), &who(), 900).unwrap();
    assert_eq!(order_result(&bought), 1, "the offer's own booster sells while the offer is open");
    assert!(find(&bought.replies, &[38, 0, 16]).is_some(), "its buff row starts at once");
    assert_eq!(boosted.consumable_effects().boost().coins, 50);
    assert!(
        !(boosted.inventory_state().items.iter()).any(|item| item.definition_id == 81),
        "the scroll was used, not left in the bag"
    );
    assert!(bought.granted_items.is_empty(), "a scroll already used is not reported as granted");
    assert_eq!(boosted.state().sc_wallet.unwrap().balance, 0, "paid in Station Cash");
    assert_eq!(win(&mut boosted, instance), (base * 3 + 1) / 2, "the win pays half as much again, once");
}

#[test]
fn only_the_open_offers_own_bundles_sell_before_go() {
    // No offer: an idle player's order is an ordinary Marketplace order.
    let mut game = game(10_000);
    let (_, instance) = offer(&mut game);
    // A Marketplace row the offer does not advertise stays refused while the player is busy.
    let stores = json("stores.json");
    let other = (stores["items"].as_array().unwrap().iter())
        .find(|row| row["category_id"] == 408)
        .map(|row| row["item_id"].as_u64().unwrap() as u32)
        .unwrap();
    let refused = game.handle_in_world(&buy(1, other), &who(), 900).unwrap();
    assert_ne!(order_result(&refused), 1, "a busy player buys nothing else");
    // After Go the run is no longer an offer.
    let mut started = game.handle_in_world(&lifecycle(5, instance), &who(), 1_000).unwrap();
    dress(&mut game, &mut started);
    let late = game.handle_in_world(&buy(2, SUPER_STAR), &who(), 1_100).unwrap();
    assert_ne!(order_result(&late), 1, "nothing sells mid-game");
    assert_eq!(game.consumable_effects().boost().stars, 0);
}
