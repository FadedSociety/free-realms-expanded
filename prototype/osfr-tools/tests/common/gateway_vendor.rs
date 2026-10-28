//! Shared gateway packet plumbing for the vendor suites (`vendor_sell`,
//! `vendor_stock_coverage`, `vendor_buy_sell_gateway`) and the collection-node suite.
//!
//! Client-derived bodies: 26/8 constructor `008b6bc0` (u64 target + bool),
//! 26/10 constructor `00a36f80` (u64 target + i32 choice), 66/43 serializer
//! `00b3ab20`, 66/44 reader `00b32430` (status at wire offset 48), the 165/5
//! sell body written by `00c75a70`/`00c76dd0` (Lua-facing `00c77030`), and the
//! 165/12 buy-back request written by `00c759c0` (caller `00c76600`).
//! The 7-byte tunnel prefix mirrors the captured client wrapper (`005`, subtype
//! `1`, payload length).
//!
//! Each include site uses a subset of the helpers.
#![allow(dead_code)]
//!
//! The 165/6 completion parser lives here too, so a layout correction lands in
//! one place.

#[path = "gateway_packets.rs"]
mod gateway_packets;
// Each `#[path]` suite uses a different subset of these request builders.
#[allow(unused_imports)]
pub use gateway_packets::{bundle, interact, login, select};

use osfr_tools::{
    character_store::CharacterState,
    gameplay::GameplayState,
    gateway::{Gateway, GatewayOutcome},
    inventory::InventoryState,
    npc_interaction::{MerchantBuyRequestV2, encode_merchant_buy_66_43},
};
use serde_json::Value;
use std::path::{Path, PathBuf};

pub const NOW: u64 = 10_000;

pub fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

pub fn inner_packets(outcome: &GatewayOutcome) -> Vec<Vec<u8>> {
    outcome
        .replies
        .iter()
        .map(|packet| packet[7..].to_vec())
        .collect()
}

pub fn send(gateway: &mut Gateway, packet: &[u8]) -> Vec<Vec<u8>> {
    inner_packets(
        &gateway
            .handle_at(&wrap(packet), NOW)
            .expect("gateway handles the request"),
    )
}

/// The integration suite's private build directory for the representative
/// vendor's content pack.
#[allow(dead_code)] // Only the suites that build that fixture call it.
pub fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab/build/w178-vendor-gateway")
}

/// Reconnect through the saved character half of the save envelope, not a fresh
/// reference character. It deliberately sets no role: `Gateway::load_gameplay`
/// mints the production default, so an assertion on `role()` is not the helper's
/// own setup.
#[allow(dead_code)] // Only the suites that reconnect through the fixture call it.
pub fn connect_with(
    reference: &Path,
    character: CharacterState,
    content: &Path,
    state: GameplayState,
) -> Gateway {
    let guid = character.guid;
    let mut gateway = Gateway::load_gameplay(reference, character, content, Some(state))
        .expect("gateway loads the vendor content");
    gateway.set_dev_tools(false);
    for packet in [login(guid), wrap(&[13, 0]), wrap(&[10, 0])] {
        gateway
            .handle_at(&packet, NOW)
            .expect("initialization message handles");
    }
    gateway
}

/// One request through the gateway: the inner replies and the outcome flags.
#[allow(dead_code)] // Only the suites that assert on the outcome flags use this.
pub fn handled(gateway: &mut Gateway, packet: &[u8]) -> (Vec<Vec<u8>>, GatewayOutcome) {
    let outcome = gateway
        .handle_at(&wrap(packet), NOW)
        .expect("gateway handles the request");
    (inner_packets(&outcome), outcome)
}

/// T772: the saved roster's pets, from the gateway's save document.
#[allow(dead_code)] // Only the pet suites read the roster.
pub fn roster(gateway: &Gateway) -> Vec<Value> {
    gateway
        .snapshot()
        .pets
        .and_then(|document| document["pets"].as_array().cloned())
        .unwrap_or_default()
}

/// A reconnect through the save document: the gameplay half and the pet roster reload into a
/// fresh gateway for the same content.
#[allow(dead_code)] // Only the pet suites reconnect.
pub fn reload_saved(reference: &Path, gateway: &Gateway, content: &Path) -> Gateway {
    let saved = gateway.snapshot();
    let mut restored =
        Gateway::load_gameplay(reference, gateway.state().clone(), content, saved.gameplay)
            .expect("the saved gameplay reloads");
    restored
        .adopt_pets(saved.pets)
        .expect("the saved roster reloads");
    restored
}

pub fn inventory_of(gateway: &Gateway) -> InventoryState {
    gateway
        .snapshot()
        .gameplay
        .expect("gameplay state exists")
        .inventory
}

pub fn count_definition(state: &InventoryState, definition_id: i32) -> u32 {
    state
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id)
        .map(|item| item.count)
        .sum()
}

/// 66/44 result; the client reader `00b32430` compares the dword at offset 48
/// with 1.
pub fn merchant_status(packet: &[u8]) -> u32 {
    assert_eq!(packet.len(), 56);
    assert_eq!(&packet[..4], &[66, 0, 44, 0]);
    u32::from_le_bytes(packet[48..52].try_into().unwrap())
}

pub fn vendor_buy_packet(transaction: u32, item: i32, quantity: u32) -> Vec<u8> {
    encode_merchant_buy_66_43(&MerchantBuyRequestV2 {
        initial_dwords: [item as u32, 0],
        context_dwords: [0; 3],
        context_bytes: Vec::new(),
        trailing_dwords: [quantity, 0, transaction],
    })
    .expect("66/43 request encodes")
}

/// 165/5 body: u64 merchant (0 = the open shop) + u32 item instance + u32
/// quantity, in `00c76dd0`'s field order and written by `00c75a70` (the Lua
/// entry point is `00c77030`).
pub fn sell_packet(merchant: u64, instance: i32, quantity: u32) -> Vec<u8> {
    let mut packet = vec![165, 0, 5, 0];
    packet.extend(merchant.to_le_bytes());
    packet.extend(instance.to_le_bytes());
    packet.extend(quantity.to_le_bytes());
    packet
}

/// 165/12 body: the sale transaction id plus a context dword, written by
/// `00c759c0` (the caller `00c76600` passes the record's dwords).
#[allow(dead_code)] // Only the L011 save-failure test sends a buy-back request.
pub fn buy_back_packet(transaction: u32, context: u32) -> Vec<u8> {
    let mut packet = vec![165, 0, 12, 0];
    packet.extend(transaction.to_le_bytes());
    packet.extend(context.to_le_bytes());
    packet
}

/// The result dword of the 165/13 answer, which the client reads as three dwords after the
/// header (`00c71e40`: transaction, result, context).
#[allow(dead_code)] // Only the suites that probe a buy-back answer use this parser.
pub fn buy_back_result(replies: &[Vec<u8>]) -> u32 {
    let packet = replies
        .iter()
        .find(|packet| packet.starts_with(&[165, 0, 13, 0]))
        .expect("a 165/12 buy-back answers 165/13");
    assert_eq!(packet.len(), 16, "165/13 is the header and three dwords");
    u32::from_le_bytes(packet[8..12].try_into().unwrap())
}

/// 165/4 body (client writer `00c75ed0`, request `00c77230`): u64 merchant, the merchant-record
/// context dword, the definition id, tint, then quantity.
#[allow(dead_code)] // Only the suites that send a direct 165/4 buy use this builder.
pub fn coin_buy_packet(merchant: u64, context: u32, item: i32, quantity: u32) -> Vec<u8> {
    let mut packet = vec![165, 0, 4, 0];
    packet.extend(merchant.to_le_bytes());
    packet.extend(context.to_le_bytes());
    packet.extend(item.to_le_bytes());
    packet.extend(0i32.to_le_bytes()); // tint
    packet.extend(quantity.to_le_bytes());
    packet
}

/// The 165/6 record as `00c74ef0`/`00c71f30` read it: header, result, leading item,
/// then kind, u64 merchant, transaction, u64 timestamp, item pair, quantity, unit
/// price and remaining. `remaining` (`this+13`, the last wire dword) is the buy-back
/// quantity a successful sale leaves.
#[allow(dead_code)] // Only the suites that read a 165/6 completion use this parser.
#[derive(Debug, PartialEq)]
pub struct SellCompletion {
    pub result: u32,
    pub kind: u32,
    pub merchant: u64,
    pub transaction: u32,
    pub definition: i32,
    pub quantity: u32,
    pub price: u32,
    pub remaining: u32,
}

#[allow(dead_code)]
pub fn sell_completion(packet: &[u8]) -> SellCompletion {
    assert_eq!(
        packet.len(),
        56,
        "165/6 must match the 0x00c74ef0/0x00c71f30 reader size"
    );
    assert_eq!(&packet[..4], &[165, 0, 6, 0]);
    let u32_at = |at: usize| u32::from_le_bytes(packet[at..at + 4].try_into().unwrap());
    let u64_at = |at: usize| u64::from_le_bytes(packet[at..at + 8].try_into().unwrap());
    SellCompletion {
        result: u32_at(4),
        kind: u32_at(12),
        merchant: u64_at(16),
        transaction: u32_at(24),
        definition: u32_at(36) as i32,
        quantity: u32_at(44),
        price: u32_at(48),
        remaining: u32_at(52),
    }
}

#[allow(dead_code)]
pub fn completions(replies: &[Vec<u8>]) -> Vec<SellCompletion> {
    replies
        .iter()
        .filter(|packet| packet.starts_with(&[165, 0, 6, 0]))
        .map(|packet| sell_completion(packet))
        .collect()
}
