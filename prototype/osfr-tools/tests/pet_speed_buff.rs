//! T532: a happy summoned pet gives its owner the Pet Happiness Speed Buff.
//!
//! Driven through the real route: `Gameplay::handle_in_world` (the gateway's 53/x dispatch into
//! `PetService::handle_at_ms`), `Gameplay::pet_tick` / `settle_pets` for the tick and zone paths,
//! and `Gateway::handle_at` for the movement allowance.
//!
//! Client facts the packets rest on: the buff-bar row is family 38 sub 16 (reader `008d8870`, blob
//! `008bfc00`: i32 tag id, i32 length, 85-byte row; the Lua `GroupHandler:BuffsUpdated` reads
//! instance/icon/duration/timeLeft/name and `AddAbilityEffect` returns early when icon is 0), and
//! the character speed stat is family 38 sub 7 stat id 2 `MaxMovementSpeed` (stat-name table
//! `0x01af2008`, entry `0x01af2010`; entry reader `008e7640`). The rule values are (R):
//! pets.json `happy_buff` (mood 25, +3 units/s, 10 s, 4 s refresh).

#[path = "common/gateway_packets.rs"]
mod gateway_packets;
#[path = "support/pet_inventory.rs"]
mod pet_inventory;
#[path = "../src/bin/osfr_gateway/pet_wire.rs"]
mod pet_wire;
#[path = "../src/restoration/piece_056.rs"]
#[allow(dead_code)]
mod piece_056;

use gateway_packets::{body, login, move_to, wrap};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    gateway::Gateway,
    pets::{PetBusy, PetContent, PetService},
};
use pet_inventory::InventoryFixture;
use pet_wire::{dismiss, summon};
use piece_056::{EffectTagPayload, decode_add_effect_tag, decode_remove_effect_tag};
use serde_json::{Value, json};
use std::path::PathBuf;

const OWNER: u64 = 1;
const NOW: u64 = 10_000;
/// Char: a non-vanity species with a model and an adoption item in the bundled `pets.json`.
const PET_SPECIES: u32 = 1_000_015;
/// Crab: a vanity Pet Pal (T454), the same row `tests/pet_care.rs` uses.
const CRAB_ITEM: i32 = 7885;
/// `mounts.json` `on_foot.max_movement_speed` (the captured world-entry stat id 2).
const ON_FOOT: f32 = 8.0;
/// `pets.json` `happy_buff.speed_bonus`.
const BONUS: f32 = 3.0;
/// The recovered locale row "Pet Happiness Speed Buff" (`Global.Text.116881`).
const NAME_ID: i32 = 116881;
/// `Resources/Images/Images.txt` row 3851 `icon_status_sprint_32.dds`, the status speed icon.
const ICON: i32 = 3851;

fn reference() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-opening-cooking-original-assets")
}

fn who() -> CharacterState {
    CharacterState {
        guid: OWNER,
        zone_id: 0,
        position: [-440.0, 0.8, 1292.0],
        rotation: [1.0, 0.0, 0.0],
    }
}

/// One saved pet with every need bar at `needs`, so its decayed mood is that value (R: the mood
/// formula is the average of the three bars).
fn roster(vanity: bool, needs: f32) -> Value {
    let content = PetContent::bundled().unwrap();
    let mut pets = PetService::new(content.clone()).unwrap();
    let definition = if vanity {
        content.definition_for_item(CRAB_ITEM).unwrap()
    } else {
        content.definition(PET_SPECIES).unwrap()
    };
    pets.adopt_vendor_grant(1, &definition, 0).unwrap();
    let mut document = pets.document_value().unwrap();
    for bar in ["food", "groom", "play", "mood"] {
        document["pets"][0]["needs"][bar] = json!(needs);
    }
    document
}

fn game(roster: Value) -> Gameplay {
    let mut game = Gameplay::load(&reference(), None).unwrap();
    game.adopt_pets(Some(roster)).unwrap();
    game.initialize_world(&who(), 0).unwrap();
    game
}

/// The buff-bar rows (38/16) in one outcome; `body` also strips the gateway's `005` tunnel.
fn rows(packets: &[Vec<u8>]) -> Vec<(i32, EffectTagPayload)> {
    packets
        .iter()
        .filter_map(|packet| decode_add_effect_tag(body(packet)).ok())
        .collect()
}

/// The 38/17 tag ids in one outcome.
fn removes(packets: &[Vec<u8>]) -> Vec<i32> {
    packets
        .iter()
        .filter_map(|packet| decode_remove_effect_tag(body(packet)).ok())
        .collect()
}

/// Every 38/7 stat id 2 (`MaxMovementSpeed`) value in one outcome, reader-faithful: u16 38, u16
/// 7, u64 target, i32 count, then per entry i32 id, i32 kind (0 int, 1 float) and the value.
fn speeds(packets: &[Vec<u8>]) -> Vec<f32> {
    packets
        .iter()
        .filter_map(|packet| {
            let packet = body(packet);
            if packet.get(..4)? != [38, 0, 7, 0] {
                return None;
            }
            let count = i32::from_le_bytes(packet[12..16].try_into().unwrap());
            let mut at = 16;
            let mut speed = None;
            for _ in 0..count {
                let id = i32::from_le_bytes(packet[at..at + 4].try_into().unwrap());
                let kind = i32::from_le_bytes(packet[at + 4..at + 8].try_into().unwrap());
                match (id, kind) {
                    (2, 1) => {
                        speed = Some(f32::from_le_bytes(
                            packet[at + 8..at + 12].try_into().unwrap(),
                        ))
                    }
                    (2, other) => panic!("stat 2 must be a float, not kind {other}"),
                    _ => {}
                }
                at += 12;
            }
            assert_eq!(at, packet.len(), "38/7 has trailing bytes");
            speed
        })
        .collect()
}

/// The gateway's 38/12 location correction (the snap-back a refused move sends).
fn snap_back(packets: &[Vec<u8>]) -> bool {
    packets
        .iter()
        .any(|packet| body(packet).starts_with(&[38, 0, 12, 0]))
}

#[test]
fn a_happy_summoned_pet_sends_the_buff_row_and_the_boosted_speed_stat() {
    let mut game = game(roster(false, 100.0));
    let out = game.handle_in_world(&summon(1), &who(), NOW).unwrap();
    let sent = rows(&out.replies);
    assert_eq!(sent.len(), 1, "one buff-bar row per tag id");
    let (tag, row) = &sent[0];
    assert_eq!(row.name_id(), NAME_ID, "the recovered locale row");
    assert_eq!(row.icon_id(), ICON, "the client image table row");
    assert_eq!(row.duration_seconds(), 10, "the row's countdown");
    assert!(
        *tag > 1_500_000_000,
        "above the consumable and world tag ranges, so it can never alias one"
    );
    assert_eq!(
        speeds(&out.replies),
        vec![ON_FOOT + BONUS],
        "the client's max speed is the on-foot value plus the bonus"
    );
    // Inside the refresh window the buff repeats nothing.
    let quiet = game.pet_tick(&who(), NOW + 1_000).unwrap();
    assert!(rows(&quiet.replies).is_empty() && speeds(&quiet.replies).is_empty());
}

#[test]
fn a_refresh_replaces_the_one_row_and_never_stacks() {
    let mut game = game(roster(false, 100.0));
    let out = game.handle_in_world(&summon(1), &who(), NOW).unwrap();
    let tag = rows(&out.replies)[0].0;
    let refreshed = game.pet_tick(&who(), NOW + 4_000).unwrap();
    let rows = rows(&refreshed.replies);
    assert_eq!(rows.len(), 1, "a refresh re-sends the one row");
    assert_eq!(
        rows[0].0, tag,
        "the tag id never changes, so nothing stacks"
    );
    assert_eq!(speeds(&refreshed.replies), vec![ON_FOOT + BONUS]);
}

#[test]
fn a_vanity_pet_gives_no_buff() {
    let mut game = game(roster(true, 100.0));
    let out = game.handle_in_world(&summon(1), &who(), NOW).unwrap();
    assert!(rows(&out.replies).is_empty(), "vanity pets never buff");
    assert!(
        speeds(&out.replies).is_empty(),
        "the client's speed stays the on-foot value"
    );
    for step in 1..=12u64 {
        let tick = game.pet_tick(&who(), NOW + step * 1_000).unwrap();
        assert!(rows(&tick.replies).is_empty(), "step {step}");
        assert!(speeds(&tick.replies).is_empty(), "step {step}");
    }
}

#[test]
fn below_the_threshold_there_is_no_row_and_no_refresh() {
    let mut game = game(roster(false, 5.0));
    let out = game.handle_in_world(&summon(1), &who(), NOW).unwrap();
    assert!(rows(&out.replies).is_empty());
    assert!(speeds(&out.replies).is_empty());
    for step in 1..=12u64 {
        let tick = game.pet_tick(&who(), NOW + step * 1_000).unwrap();
        assert!(rows(&tick.replies).is_empty(), "step {step}");
        assert!(speeds(&tick.replies).is_empty(), "step {step}");
    }
}

#[test]
fn a_mood_drop_below_the_threshold_ends_the_buff() {
    let mut game = game(roster(false, 100.0));
    game.handle_in_world(&summon(1), &who(), NOW).unwrap();
    // Summoned needs decay 0.02/s: 3,800 s takes the mood below 25.
    let ended = game.pet_tick(&who(), NOW + 3_800_000).unwrap();
    assert_eq!(removes(&ended.replies).len(), 1, "38/17");
    assert_eq!(speeds(&ended.replies), vec![ON_FOOT], "speed restored");
    let quiet = game.pet_tick(&who(), NOW + 3_801_000).unwrap();
    assert!(rows(&quiet.replies).is_empty());
    assert!(removes(&quiet.replies).is_empty());
    assert!(speeds(&quiet.replies).is_empty());
}

#[test]
fn dismissal_ends_the_buff_at_once_and_the_tick_stops() {
    let mut game = game(roster(false, 100.0));
    game.handle_in_world(&summon(1), &who(), NOW).unwrap();
    let out = game
        .handle_in_world(&dismiss(1), &who(), NOW + 1_000)
        .unwrap();
    assert_eq!(removes(&out.replies).len(), 1);
    assert_eq!(speeds(&out.replies), vec![ON_FOOT]);
    assert!(
        game.pet_tick(&who(), NOW + 2_000).is_none(),
        "no pet and no buff: the tick is idle"
    );
}

#[test]
fn zoning_ends_the_buff_with_the_pet() {
    let mut game = game(roster(false, 100.0));
    game.handle_in_world(&summon(1), &who(), NOW).unwrap();
    let ended = game.settle_pets(&who(), true, NOW + 1_000).unwrap();
    assert_eq!(removes(&ended.replies).len(), 1);
    assert_eq!(speeds(&ended.replies), vec![ON_FOOT]);
    assert!(rows(&ended.replies).is_empty(), "no row survives the zone");
    assert!(game.pet_view(OWNER).is_none(), "the pet left the zone");
    // T487 brings the pet back on the destination's next tick, so the buff starts again
    // from a fresh row there: the zone dropped the old one, it did not keep it running.
    let back = game
        .pet_tick(&who(), NOW + 2_000)
        .expect("the remembered pet comes back");
    assert_eq!(rows(&back.replies).len(), 1);
    assert_eq!(speeds(&back.replies), vec![ON_FOOT + BONUS]);
}

#[test]
fn a_refused_pet_request_leaves_the_buff_and_the_pet_alone() {
    let mut game = game(roster(false, 100.0));
    game.handle_in_world(&summon(1), &who(), NOW).unwrap();
    let state = game.state();
    let pets = game.pets_document();
    let refused = game
        .handle_in_world(&summon(999), &who(), NOW + 1_000)
        .unwrap();
    assert!(
        refused
            .replies
            .iter()
            .any(|p| p.starts_with(&[15, 0, 3, 0]))
    );
    assert!(!refused.changed);
    assert!(rows(&refused.replies).is_empty());
    assert!(removes(&refused.replies).is_empty());
    assert!(speeds(&refused.replies).is_empty());
    assert_eq!(game.state(), state, "a refusal changes no live state");
    assert_eq!(game.pets_document(), pets, "a refusal changes no roster");
    assert_eq!(game.pet_view(OWNER).unwrap().pet_id, 1, "the pet stays out");
    let tick = game.pet_tick(&who(), NOW + 2_000).unwrap();
    assert!(rows(&tick.replies).is_empty(), "still fresh");
}

#[test]
fn a_reconnect_starts_without_the_buff() {
    let content = PetContent::bundled().unwrap();
    let mut fixture = InventoryFixture::new();
    let mut pets = PetService::new(content).unwrap();
    let id = pets.adopt_vendor_grant(1, &sample_pet(), 0).unwrap();
    pets.handle_at_ms(
        &summon(id),
        OWNER,
        [0.0; 3],
        &mut fixture.inventory,
        PetBusy::default(),
        NOW,
    )
    .unwrap();
    assert!(pets.happy_buff_tick(NOW).is_some(), "the row is sent");
    assert_eq!(pets.happy_speed_bonus(), Some(BONUS));
    pets.reset_session();
    assert_eq!(pets.happy_speed_bonus(), None, "session-only state");
    assert_eq!(
        pets.happy_buff_tick(NOW + 1),
        None,
        "a fresh session sends nothing"
    );
}

/// The non-vanity definition `roster` adopts.
fn sample_pet() -> osfr_tools::pets::PetDefinition {
    PetContent::bundled()
        .unwrap()
        .definition(PET_SPECIES)
        .unwrap()
}

/// A gateway that has logged in and loaded the world with `roster`'s pets.
fn session(roster: Value) -> Gateway {
    let content = reference();
    let state = Gameplay::load(&content, None).unwrap().state();
    let reference = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-gateway-reference");
    let mut gateway = Gateway::load_gameplay(&reference, who(), &content, Some(state)).unwrap();
    gateway.adopt_pets(Some(roster)).unwrap();
    gateway.handle_at(&login(OWNER), 0).unwrap();
    gateway.handle_at(&wrap(&[13, 0]), 1).unwrap();
    gateway.handle_at(&wrap(&[10, 0]), 2).unwrap();
    gateway
}

/// 90 units in 100 ms: over the unbuffed hard limit (10 * 0.1 * 80 = 80, so a refusal) and over
/// the buffed soft limit (13 * 0.1 * 40 = 52, so a clamp), but under the buffed hard limit
/// (13 * 0.1 * 80 = 104), so only the buff's allowance admits it.
fn far_move() -> Vec<u8> {
    let mut far = who().position;
    far[0] += 90.0;
    wrap(&move_to(OWNER, far))
}

#[test]
fn the_gateway_accepts_a_move_only_the_buff_admits() {
    let mut gateway = session(roster(false, 100.0));
    let buffed = gateway.handle_at(&wrap(&summon(1)), NOW).expect("summon");
    assert_eq!(
        rows(&buffed.replies).len(),
        1,
        "the buff row reached the client"
    );
    assert_eq!(speeds(&buffed.replies), vec![ON_FOOT + BONUS]);

    // Anchor the movement clock with a no-op report, then the far move 100 ms later.
    gateway
        .handle_at(&wrap(&move_to(OWNER, who().position)), NOW + 10)
        .expect("anchor");
    let moved = gateway.handle_at(&far_move(), NOW + 110).expect("move");
    assert!(snap_back(&moved.replies), "the clamp corrects the client");
    // Clamped to the buff's soft limit: (10 + 3) * 0.1 s * 40 = 52 units, not the 40 an
    // unbuffed player gets, and not a rejection (which keeps the previous pose).
    let advance = gateway.state().position[0] - who().position[0];
    assert!((advance - 52.0).abs() < 0.5, "advance {advance}");
}

#[test]
fn without_the_buff_the_same_move_is_rejected() {
    let mut gateway = session(roster(true, 100.0));
    let vanity = gateway.handle_at(&wrap(&summon(1)), NOW).expect("summon");
    assert!(rows(&vanity.replies).is_empty());
    gateway
        .handle_at(&wrap(&move_to(OWNER, who().position)), NOW + 10)
        .expect("anchor");
    let moved = gateway.handle_at(&far_move(), NOW + 110).expect("move");
    assert!(snap_back(&moved.replies), "no buff, no allowance");
    assert_eq!(gateway.state().position, who().position);
}
