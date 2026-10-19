//! Shared gateway packet plumbing for the vendor suites (`vendor_sell`,
//! `vendor_stock_coverage`).
//!
//! Client-derived bodies: 26/8 constructor `008b6bc0` (u64 target + bool),
//! 26/10 constructor `00a36f80` (u64 target + i32 choice), 66/43 serializer
//! `00b3ab20`, 66/44 reader `00b32430` (status at wire offset 48), and the
//! 165/5 sell body read by the Coin Store handler. The 7-byte tunnel prefix
//! mirrors the captured client wrapper (`005`, subtype `1`, payload length).

use osfr_tools::{
    gateway::{Gateway, GatewayOutcome},
    inventory::InventoryState,
    npc_interaction::{MerchantBuyRequestV2, encode_merchant_buy_66_43},
};

pub const NOW: u64 = 10_000;

pub fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

pub fn login() -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    for (i, text) in ["synthetic-ticket", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
        bytes.extend((text.len() as u32).to_le_bytes());
        bytes.extend(text.as_bytes());
        if i == 0 {
            bytes.extend(1u64.to_le_bytes());
        }
    }
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

/// 165/5: u64 merchant (0 = the open shop) + i32 item instance + u32 quantity.
pub fn sell_packet(merchant: u64, instance: i32, quantity: u32) -> Vec<u8> {
    let mut packet = vec![165, 0, 5, 0];
    packet.extend(merchant.to_le_bytes());
    packet.extend(instance.to_le_bytes());
    packet.extend(quantity.to_le_bytes());
    packet
}

pub fn interact(npc_id: u64) -> Vec<u8> {
    let mut packet = vec![26, 0, 8, 0];
    packet.extend(npc_id.to_le_bytes());
    packet.push(0);
    packet
}

pub fn select(npc_id: u64, option: u32) -> Vec<u8> {
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(npc_id.to_le_bytes());
    packet.extend(option.to_le_bytes());
    packet
}
