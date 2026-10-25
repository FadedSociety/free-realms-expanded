//! W146/T510: the item bar through the real `Gameplay` route: assign, 38/25, use, heal, recast,
//! reload, and the schema-2 food auras and transformation treats.
#[path = "../src/restoration/piece_056.rs"]
#[allow(dead_code)]
mod piece_056;

use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, Outcome},
    inventory::InventoryItem,
    item_bar::use_wire,
};
use std::path::PathBuf;

const POTION: i32 = 11002; // Small Health Potion, in the captured catalogue
const HAT_LIKE: i32 = 1; // placeholder instance id that is not a potion (see `plain_item`)
const VEAL: i32 = 2982; // Electric Veal Meat, an aura row in data/content/consumables.json
const BERRIES: i32 = 4042; // Bark Berries, a transform row

fn reference() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-opening-cooking-original-assets")
}
fn who() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: [-440.0, 0.8, 1292.0],
        rotation: [1.0, 0.0, 0.0],
    }
}
fn msg(main: u16, sub: u16, fields: &[i32]) -> Vec<u8> {
    let mut bytes = [main.to_le_bytes(), sub.to_le_bytes()].concat();
    for field in fields {
        bytes.extend(field.to_le_bytes());
    }
    bytes
}
fn cast(bar: i32, slot: i32, guid: u64) -> Vec<u8> {
    let mut bytes = msg(36, 10, &[bar, slot, 0]);
    bytes.extend(guid.to_le_bytes());
    bytes
}
fn item(definition_id: i32, id: i32, count: u32) -> InventoryItem {
    InventoryItem {
        definition_id,
        tint: 0,
        id,
        count,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: i64::from(i32::MAX),
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    }
}
/// A game that owns one stack of each `(definition, count)`; returns it and the instance ids.
fn game_with(items: &[(i32, u32)]) -> (Gameplay, Vec<i32>) {
    let mut state = Gameplay::load(&reference(), None).unwrap().state();
    let mut next = state
        .inventory
        .items
        .iter()
        .map(|i| i.id)
        .max()
        .unwrap_or(0)
        + 1000;
    let mut ids = Vec::new();
    for &(definition_id, count) in items {
        state.inventory.items.push(item(definition_id, next, count));
        ids.push(next);
        next += 1;
    }
    (Gameplay::load(&reference(), Some(state)).unwrap(), ids)
}
/// A game that owns `count` Small Health Potions; returns it and the stack's instance id.
fn game_with_potions(count: u32) -> (Gameplay, i32) {
    let (game, ids) = game_with(&[(POTION, count)]);
    (game, ids[0])
}
fn count(game: &Gameplay, definition: i32) -> u32 {
    game.state()
        .inventory
        .items
        .iter()
        .filter(|i| i.definition_id == definition)
        .map(|i| i.count)
        .sum()
}
/// The item-bar state and the live effect stack, for "a refusal changes nothing" snapshots.
fn snapshot(
    game: &Gameplay,
) -> (
    osfr_tools::inventory::InventoryState,
    Option<osfr_tools::item_bar::ItemBar>,
    osfr_tools::consumable_effects::EffectStack,
) {
    (
        game.state().inventory.clone(),
        game.state().item_bar.clone(),
        game.consumable_effects().clone(),
    )
}
fn slot(out: &Outcome, index: i32) -> Option<use_wire::ActionBarSlotUpdate> {
    out.replies
        .iter()
        .filter_map(|packet| use_wire::decode_slot_update(packet))
        .find(|update| update.bar == 2 && update.slot == index)
}
/// The composite effect id of a 35/41 (client reader `008d7ad0`: guid, tag, composite).
fn composite_effect(packet: &[u8]) -> i32 {
    i32::from_le_bytes(packet[16..20].try_into().unwrap())
}
fn has_aura(packets: &[Vec<u8>], composite: i32) -> bool {
    packets
        .iter()
        .any(|p| p.starts_with(&[35, 0, 41, 0]) && composite_effect(p) == composite)
}
/// The 38/16 buff row (client reader `008d8870`; payload at `008bfc00`).
fn buff_row(out: &Outcome) -> piece_056::EffectTagPayload {
    let packet = out
        .replies
        .iter()
        .find(|r| r.starts_with(&[38, 0, 16, 0]))
        .expect("a buff row");
    piece_056::decode_add_effect_tag(packet).unwrap().1
}
/// The model of a 35/14 (client reader `008d6890`: i32 model then u64 guid).
fn appearance_model(packet: &[u8]) -> i32 {
    i32::from_le_bytes(packet[4..8].try_into().unwrap())
}
fn has_appearance(packets: &[Vec<u8>], model: i32) -> bool {
    packets
        .iter()
        .any(|p| p.starts_with(&[35, 0, 14, 0]) && appearance_model(p) == model)
}

static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// The captured inventory catalogue plus 2982/4042 definitions, with no opening content: the
/// player has no encounter engine, which exercises the (R) "alive, but a heal needs health" rule.
struct TempContent {
    path: PathBuf,
}
impl TempContent {
    fn new() -> Self {
        use sha2::{Digest, Sha256};
        let path = std::env::temp_dir().join(format!(
            "osfr-t510-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        // Every campaign file except the opening gate and the files that require it: loading
        // this directory must leave the encounter engine absent.
        let skipped = [
            "opening.json",
            "smelting.json",
            "mining.json",
            "forging-campaign.json",
            "cooking-campaign.json",
        ];
        for entry in std::fs::read_dir(reference()).unwrap() {
            let entry = entry.unwrap();
            if !entry.file_type().unwrap().is_file()
                || skipped.iter().any(|name| entry.file_name() == *name)
            {
                continue;
            }
            std::fs::copy(entry.path(), path.join(entry.file_name())).unwrap();
        }
        let definitions: Vec<_> = [(VEAL, 277, 2797), (BERRIES, 1201, 2851)]
            .into_iter()
            .map(|(id, name, icon)| {
                let mut bytes = vec![0u8; 147];
                for (at, value) in [(0, id), (4, name), (12, icon), (32, 9)] {
                    bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
                }
                let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
                serde_json::json!({
                    "id": id,
                    "slot": 0,
                    "class_id": 9,
                    "default_tint": 0,
                    "max_stack": 0,
                    "model": "m.adr",
                    "texture": "t",
                    "tint_alias": "a",
                    "effect": 0,
                    "wield_type": 0,
                    "serialized_hex": hex,
                    "sha256": format!("{:x}", Sha256::digest(&bytes)),
                })
            })
            .collect();
        std::fs::write(
            path.join("item-catalogue.json"),
            serde_json::json!({"schema_version": 1, "definitions": definitions}).to_string(),
        )
        .unwrap();
        Self { path }
    }
}
impl Drop for TempContent {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
/// A game without an encounter engine that owns one stack of each `(definition, count)`.
fn game_without_encounter(items: &[(i32, u32)]) -> (Gameplay, Vec<i32>, TempContent) {
    let content = TempContent::new();
    let mut state = Gameplay::load(&content.path, None).unwrap().state();
    let mut next = state
        .inventory
        .items
        .iter()
        .map(|i| i.id)
        .max()
        .unwrap_or(0)
        + 1000;
    let mut ids = Vec::new();
    for &(definition_id, count) in items {
        state.inventory.items.push(item(definition_id, next, count));
        ids.push(next);
        next += 1;
    }
    (
        Gameplay::load(&content.path, Some(state)).unwrap(),
        ids,
        content,
    )
}
fn potions(game: &Gameplay) -> u32 {
    game.state()
        .inventory
        .items
        .iter()
        .filter(|i| i.definition_id == POTION)
        .map(|i| i.count)
        .sum()
}
fn health(game: &Gameplay) -> u32 {
    game.state().encounter.unwrap().player_health
}
fn slot_updates(out: &Outcome) -> usize {
    out.replies
        .iter()
        .filter(|r| r.len() == 63 && r.starts_with(&[38, 0, 25, 0]))
        .count()
}
/// The (health, maximum) of the last 38/1 in the replies.
fn health_packet(out: &Outcome) -> Option<(i32, i32)> {
    out.replies
        .iter()
        .rev()
        .find(|r| r.len() == 12 && r.starts_with(&[38, 0, 1, 0]))
        .map(|r| {
            (
                i32::from_le_bytes(r[4..8].try_into().unwrap()),
                i32::from_le_bytes(r[8..12].try_into().unwrap()),
            )
        })
}
fn failed(out: &Outcome) -> bool {
    out.replies.len() == 1 && out.replies[0].starts_with(&[36, 0, 1, 0])
}

#[test]
fn a_bound_potion_heals_once_per_recast_and_the_bar_survives_a_reload() {
    let (mut game, id) = game_with_potions(3);
    let login = game.initialize_world(&who(), 0).unwrap();
    assert_eq!(slot_updates(&login), 4, "the bar is sent at world entry");

    // Bind slot 0, as the native drag does.
    let bound = game
        .handle_in_world(&msg(42, 6, &[0, id]), &who(), 1_000)
        .unwrap();
    assert!(bound.changed);
    assert_eq!(slot_updates(&bound), 4);

    // Hurt the player, then use the slot.
    game.receive_field_damage(40, 2_000, true).unwrap();
    let hurt = health(&game);
    let used = game.handle_in_world(&cast(2, 0, 1), &who(), 3_000).unwrap();
    assert!(used.changed);
    assert_eq!(potions(&game), 2, "exactly one potion was spent");
    let (now, max) = health_packet(&used).expect("the HUD gets a 38/1");
    assert_eq!(now as u32, health(&game));
    assert_eq!(now as u32, (hurt + max as u32 * 35 / 100).min(max as u32));
    assert!(now as u32 > hurt);
    assert!(
        used.replies.iter().any(|r| r.starts_with(&[38, 0, 3, 0])),
        "38/3 count update"
    );
    assert_eq!(slot_updates(&used), 4);

    // The recast blocks a replay: nothing is spent or healed.
    game.receive_field_damage(10, 3_500, true).unwrap();
    let before = (health(&game), potions(&game));
    let replay = game.handle_in_world(&cast(2, 0, 1), &who(), 4_000).unwrap();
    assert!(failed(&replay) && !replay.changed);
    assert_eq!((health(&game), potions(&game)), before);

    // The bar and the stack persist; the recast does not.
    let mut game = Gameplay::load(&reference(), Some(game.state())).unwrap();
    let login = game.initialize_world(&who(), 0).unwrap();
    assert_eq!(slot_updates(&login), 4);
    assert_eq!(potions(&game), 2);
    let again = game.handle_in_world(&cast(2, 0, 1), &who(), 0).unwrap();
    assert!(again.changed && potions(&game) == 1);
}

#[test]
fn bar_use_refusals_leave_health_and_stack_alone() {
    let (mut game, id) = game_with_potions(2);
    game.handle_in_world(&msg(42, 6, &[1, id]), &who(), 0)
        .unwrap();
    game.receive_field_damage(40, 0, true).unwrap();
    let before = (health(&game), potions(&game));
    for (why, request) in [
        ("empty slot", cast(2, 0, 1)),
        ("slot past the last", cast(2, 4, 1)),
        ("someone else's guid", cast(2, 1, 2)),
        ("job bar slot with an item bar tail", cast(3, 1, 1)),
    ] {
        let out = game.handle_in_world(&request, &who(), 1_000).unwrap();
        assert!(failed(&out), "{why}");
        assert_eq!((health(&game), potions(&game)), before, "{why}");
    }
    // Canary: the bound slot works with the right request.
    let out = game.handle_in_world(&cast(2, 1, 1), &who(), 1_000).unwrap();
    assert!(!failed(&out) && potions(&game) == before.1 - 1 && health(&game) > before.0);
    // A knocked-out player can't use it.
    game.receive_field_damage(u32::MAX / 2, 1_500, true)
        .unwrap();
    assert_eq!(health(&game), 0);
    let left = potions(&game);
    let out = game
        .handle_in_world(&cast(2, 1, 1), &who(), 60_000)
        .unwrap();
    assert!(failed(&out) && potions(&game) == left && health(&game) == 0);
}

#[test]
fn other_inventory_requests_keep_their_routes() {
    let (mut game, _) = game_with_potions(1);
    // 42/16 for an instance that is not a bar consumable still reaches the old handling
    // (the Mystery Chest route when its package is loaded, otherwise the shared 42 branch).
    let out = game
        .handle_in_world(&msg(42, 16, &[HAT_LIKE]), &who(), 0)
        .unwrap();
    assert!(!out.changed);
    assert!(
        !out.replies
            .iter()
            .any(|r| r.starts_with(&[38, 0, 30, 0]) && r[24] == 1)
    );
}

#[test]
fn electric_veal_meat_consumes_one_and_shows_its_thirty_minute_aura() {
    // No opening content: the player has no encounter engine.
    let (mut game, ids, content) = game_without_encounter(&[(VEAL, 2), (POTION, 1)]);
    let (veal, potion) = (ids[0], ids[1]);
    game.handle_in_world(&msg(42, 6, &[0, veal]), &who(), 1_000)
        .unwrap();
    game.handle_in_world(&msg(42, 6, &[1, potion]), &who(), 1_000)
        .unwrap();
    // The 38/30 confirm text is the row's own name and description.
    let checked = game
        .handle_in_world(&msg(42, 16, &[veal]), &who(), 2_000)
        .unwrap();
    let packet = checked
        .replies
        .iter()
        .find(|r| r.starts_with(&[38, 0, 30, 0]))
        .expect("the confirm dialog");
    assert_eq!(
        (
            i32::from_le_bytes(packet[16..20].try_into().unwrap()),
            i32::from_le_bytes(packet[20..24].try_into().unwrap()),
        ),
        (277, 2707)
    );
    // Without health tracking a heal row is refused and spends nothing (R)...
    let before = snapshot(&game);
    let refused = game.handle_in_world(&cast(2, 1, 1), &who(), 2_500).unwrap();
    assert!(failed(&refused));
    assert_eq!(snapshot(&game), before);
    // ...while the aura row works through the effect stack.
    let used = game.handle_in_world(&cast(2, 0, 1), &who(), 3_000).unwrap();
    assert!(used.changed);
    assert_eq!(count(&game, VEAL), 1, "exactly one item is spent");
    assert!(
        has_aura(&used.replies, 5030) && has_aura(&used.nearby, 5030),
        "35/41 with composite 5030 for self and neighbours"
    );
    let row = buff_row(&used);
    assert_eq!((row.unknown_20, row.unknown_22), (3843, 277));
    assert_eq!(row.unknown_7, 1800, "the buff row counts thirty minutes");
    let update = slot(&used, 0).expect("the own slot's 38/25");
    assert_eq!(update.total_refresh_time, 1_800_000);
    assert_eq!((update.quantity, update.name, update.icon), (1, 277, 2797));
    // Inside thirty minutes the same use is refused and changes nothing.
    let before = snapshot(&game);
    let replay = game
        .handle_in_world(&cast(2, 0, 1), &who(), 1_802_999)
        .unwrap();
    assert!(failed(&replay) && !replay.changed);
    assert_eq!(snapshot(&game), before);
    // The binding survives a relog; the cooldown and the live effect do not (R).
    let mut relog = Gameplay::load(&content.path, Some(game.state())).unwrap();
    let login = relog.initialize_world(&who(), 0).unwrap();
    let update = slot(&login, 0).expect("the reloaded 38/25");
    assert_eq!((update.quantity, update.total_refresh_time), (1, 1_800_000));
    assert!(relog.consumable_effects().visual().auras.is_empty());
    let again = relog.handle_in_world(&cast(2, 0, 1), &who(), 0).unwrap();
    assert!(again.changed && count(&relog, VEAL) == 0);
    // Canary: exactly on the boundary the original session can use it again; the empty stack
    // unbinds its slot.
    let ready = game
        .handle_in_world(&cast(2, 0, 1), &who(), 1_803_000)
        .unwrap();
    assert!(ready.changed && count(&game, VEAL) == 0);
    assert!(slot(&ready, 0).unwrap().is_empty);
}

#[test]
fn bark_berries_transforms_and_a_live_transform_refuses_the_treat() {
    let (mut game, ids, _content) = game_without_encounter(&[(BERRIES, 2)]);
    let id = ids[0];
    game.handle_in_world(&msg(42, 6, &[0, id]), &who(), 0)
        .unwrap();
    let used = game.handle_in_world(&cast(2, 0, 1), &who(), 0).unwrap();
    assert!(used.changed && count(&game, BERRIES) == 1);
    assert!(
        has_appearance(&used.replies, 169) && has_appearance(&used.nearby, 169),
        "35/14 with model 169 for self and neighbours"
    );
    assert!(
        used.replies.iter().any(|p| p.starts_with(&[35, 0, 16, 0])),
        "the poof 21"
    );
    let row = buff_row(&used);
    assert_eq!((row.unknown_20, row.unknown_22), (3843, 1201));
    // At the 30-minute cooldown boundary the transform is still live (no tick yet): refused.
    let before = snapshot(&game);
    let refused = game
        .handle_in_world(&cast(2, 0, 1), &who(), 1_800_000)
        .unwrap();
    assert!(failed(&refused) && !refused.changed);
    assert_eq!(snapshot(&game), before);
    // The 42/16 path names the reason with the brief's chat line.
    let checked = game
        .handle_in_world(&msg(42, 16, &[id]), &who(), 1_800_000)
        .unwrap();
    let line = checked
        .replies
        .iter()
        .find(|r| r.starts_with(&[15, 0, 3, 0]))
        .expect("a chat refusal");
    assert_eq!(
        String::from_utf8(line[8..line.len() - 1].to_vec()).unwrap(),
        "You are already transformed."
    );
    assert_eq!(snapshot(&game), before);
    // The expiry tick reverts the model, then the same use works.
    let expired = game.tick(&who(), 1_800_000).unwrap();
    assert!(
        expired
            .replies
            .iter()
            .any(|p| p.starts_with(&[35, 0, 15, 0])),
        "the transform ended"
    );
    let again = game
        .handle_in_world(&cast(2, 0, 1), &who(), 1_800_001)
        .unwrap();
    assert!(again.changed && count(&game, BERRIES) == 0);
}
