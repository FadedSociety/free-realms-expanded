//! F3 (rc-0136): Station Cash for a new player, and the Station Cash displays in the world.
//!
//! - m-booster ("No sc to buy a scroll"): Station Cash is earned from activities, so a new
//!   character had none. The shipped earn table's `welcome_grant` pays once, at the first wallet
//!   read (66/10, sent on every world entry; reply 66/11 read by `00b3e120`), and never again.
//! - SC displays ("sellable sc mounts and pets aren't clickable to open the shop menu"): a click
//!   on a display (here the Gold-Tipped Dragon at the Seaside stables, a Legacy extra spawn with no
//!   NPC record) opens the Station Cash merchant window (66/42) on its Marketplace bundle, and the
//!   window's buy (66/43) is a paid Station Cash order.
//!
//! Driven through `Gameplay::handle_in_world` on the shipped content.
use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState},
    npc_interaction::{MerchantBuyRequestV2, decode_merchant_list_66_42, encode_merchant_buy_66_43},
    station_cash::{EarnTable, Wallet},
};
use std::path::PathBuf;

#[path = "support/scratch.rs"]
mod scratch;
use scratch::Scratch;

/// The Gold-Tipped Dragon display at the Seaside stables (`npc_spawns_extra.json`, Legacy 1738).
const DRAGON_DISPLAY: u64 = 100_000_001_738;
const DRAGON_AT: [f32; 3] = [-586.6817, 19.42487, -1049.762];
/// The ride it shows (mounts.json Gold-Tipped Dragon, 12671; its Marketplace item).
const DRAGON_RIDE: i32 = 76_855;

fn shipped_grant() -> u32 {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/sc_rewards.json");
    EarnTable::load(&path).unwrap().unwrap().welcome_grant
}

fn character(position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position,
        rotation: [0.0, 0.0, 1.0],
    }
}

fn game(scratch: &Scratch, saved: Option<GameplayState>) -> Gameplay {
    Gameplay::load(scratch.path(), saved).unwrap()
}

/// The Station Cash balance a 66/11 shows (`00b3e120`: status, a bool, then the balance).
fn balance(replies: &[Vec<u8>]) -> u32 {
    let wallet = replies
        .iter()
        .find(|reply| reply.starts_with(&[66, 0, 11, 0]))
        .expect("the wallet answer");
    u32::from_le_bytes(wallet[9..13].try_into().unwrap())
}

#[test]
fn a_new_character_is_welcomed_with_station_cash_once() {
    let grant = shipped_grant();
    assert!(grant > 0, "the shipped table pays a welcome grant");
    let scratch = Scratch::new("f3-welcome");
    let mut game = game(&scratch, None);
    let at = character([0.0; 3]);
    let first = game.handle_in_world(&[66, 0, 10, 0], &at, 1_000).unwrap();
    assert_eq!(balance(&first.replies), grant, "the first wallet read pays it");
    assert!(first.changed, "and the save keeps it");
    let again = game.handle_in_world(&[66, 0, 10, 0], &at, 2_000).unwrap();
    assert_eq!(balance(&again.replies), grant, "never twice");
    assert!(!again.changed);

    // Spent to nothing and relogged: still welcomed, so nothing is paid again.
    let mut saved = game.state();
    let wallet = saved.sc_wallet.take().expect("the welcomed wallet is saved");
    assert!(wallet.welcomed);
    saved.sc_wallet = Some(Wallet { balance: 0, ..wallet });
    let mut relog = self::game(&scratch, Some(saved));
    let read = relog.handle_in_world(&[66, 0, 10, 0], &at, 3_000).unwrap();
    assert_eq!(balance(&read.replies), 0);
    assert!(!read.changed);
}

#[test]
fn a_station_cash_display_opens_its_shop_and_sells_its_ride_for_station_cash() {
    // The released item tables (`data/release`), so the Marketplace lists its rides as it ships.
    let scratch = Scratch::new("f3-display");
    let release = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/release");
    for file in ["inventory.json", "item-table.json", "item-table-supplement.json"] {
        std::fs::copy(release.join(file), scratch.path().join(file)).unwrap();
    }
    let mut saved = game(&scratch, None).state();
    saved.sc_wallet = Some(Wallet { balance: 1_000, welcomed: true, ..Wallet::default() });
    let mut game = game(&scratch, Some(saved));
    let near = character([DRAGON_AT[0] + 2.0, DRAGON_AT[1], DRAGON_AT[2]]);
    game.initialize_world(&near, 500).unwrap();

    // Too far: nothing opens.
    let far = character([DRAGON_AT[0] + 40.0, DRAGON_AT[1], DRAGON_AT[2]]);
    let mut click = vec![26, 0, 8, 0];
    click.extend(DRAGON_DISPLAY.to_le_bytes());
    click.push(0);
    let refused = game.handle_in_world(&click, &far, 1_000).unwrap();
    assert!(!refused.replies.iter().any(|reply| reply.starts_with(&[66, 0, 42, 0])));

    let opened = game.handle_in_world(&click, &near, 1_100).unwrap();
    let window = opened
        .replies
        .iter()
        .find(|reply| reply.starts_with(&[66, 0, 42, 0]))
        .expect("the display opens the Station Cash window");
    let list = decode_merchant_list_66_42(window).unwrap();
    assert_eq!(list.initial_dword, u32::MAX, "no sale timer");
    let row = list
        .group
        .entries
        .iter()
        .find(|row| row.first_item_id as i32 == DRAGON_RIDE)
        .expect("the window sells the ride the display shows");
    assert!(row.can_buy);
    let merchant = u64::from(list.group.fields[3]) | (u64::from(list.group.fields[4]) << 32);
    assert_eq!(merchant, DRAGON_DISPLAY);

    let before = game.state();
    let buy = encode_merchant_buy_66_43(&MerchantBuyRequestV2 {
        initial_dwords: [row.item_id, 0],
        context_dwords: [0; 3],
        context_bytes: Vec::new(),
        trailing_dwords: [1, 0, 77],
    })
    .unwrap();
    let bought = game.handle_in_world(&buy, &near, 1_200).unwrap();
    let result = bought
        .replies
        .iter()
        .find(|reply| reply.starts_with(&[66, 0, 44, 0]))
        .expect("the window's own result");
    assert_eq!(u32::from_le_bytes(result[48..52].try_into().unwrap()), 1, "success");
    let after = game.state();
    let spent = before.sc_wallet.as_ref().unwrap().balance - after.sc_wallet.as_ref().unwrap().balance;
    assert!(spent > 0, "paid in Station Cash");
    assert_eq!(balance(&bought.replies), 1_000 - spent, "the window's balance follows");
    assert!(bought.changed);
}
