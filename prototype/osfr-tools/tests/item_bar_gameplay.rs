//! W146: the item bar through the real `Gameplay` route (needs the shared-file patch in
//! `lab/runs/claude/W146/shared-patch.diff`): assign, 38/25, use, heal, recast, reload.
use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, Outcome},
    inventory::InventoryItem,
};
use std::path::PathBuf;

const POTION: i32 = 11002; // Small Health Potion, in the captured catalogue
const HAT_LIKE: i32 = 1; // placeholder instance id that is not a potion (see `plain_item`)

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
/// A game that owns `count` Small Health Potions; returns it and the stack's instance id.
fn game_with_potions(count: u32) -> (Gameplay, i32) {
    let mut state = Gameplay::load(&reference(), None).unwrap().state();
    let id = state
        .inventory
        .items
        .iter()
        .map(|i| i.id)
        .max()
        .unwrap_or(0)
        + 1000;
    state.inventory.items.push(InventoryItem {
        definition_id: POTION,
        tint: 0,
        id,
        count,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: i64::from(i32::MAX),
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    });
    (Gameplay::load(&reference(), Some(state)).unwrap(), id)
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
