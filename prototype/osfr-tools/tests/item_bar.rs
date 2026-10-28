//! W146: item bar assignment (42/6, 42/9), bar state (38/25), item-bar use (36/10) and the
//! immediate-use check (42/16 to 38/30, 42/15) for the Small Health Potion. Every refusal has a
//! canary: the same request succeeds once the one missing fact is supplied.
use osfr_tools::{
    consumable_effects::{EffectStack, Kind, Spec},
    inventory::{Inventory, InventoryItem, InventoryState, RentalState},
    item_bar::{
        Effect, EffectKind, Effects, ItemBar, Player, Reply,
        bar_wire::{self, SLOT_UPDATE_BYTES},
    },
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

const SMALL: i32 = 2173; // usable
const SMALL_ALT: i32 = 11002; // usable, second catalogue row of the same item
const MEDIUM: i32 = 3021; // recovered consumable with no row in the test content
const HAT: i32 = 200; // not a consumable
const VEAL: i32 = 2982; // Electric Veal Meat, an aura row
const BERRIES: i32 = 4042; // Bark Berries, a transform row
const VEAL_NAME: i32 = 277;
const VEAL_DESC: i32 = 2707;
const BUFF_ICON: i32 = 3843; // OSFR's shared change-form buff icon (R)
const ME: u64 = 77;
const ALIVE: Player = Player {
    guid: ME,
    alive: true,
    can_place_box: true,
    can_heal: true,
    transformed: false,
};
const KNOCKED_OUT: Player = Player {
    alive: false,
    ..ALIVE
};
const TRANSFORMED: Player = Player {
    transformed: true,
    ..ALIVE
};
const NO_HEALING: Player = Player {
    can_heal: false,
    ..ALIVE
};

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn item(id: i32, definition_id: i32, count: u32, tint: i32) -> InventoryItem {
    InventoryItem {
        id,
        definition_id,
        tint,
        count,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: i64::from(i32::MAX),
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    }
}
fn serialize_item(item: &InventoryItem) -> Vec<u8> {
    let mut bytes = Vec::new();
    for value in [
        item.definition_id,
        item.tint,
        item.id,
        item.count as i32,
        item.consumed_count as i32,
        i32::try_from(item.last_cast_time).unwrap(),
    ] {
        bytes.extend(value.to_le_bytes());
    }
    match &item.rental {
        Some(rental) => {
            bytes.push(1);
            bytes.extend(rental.expiration.to_le_bytes());
            bytes.extend(rental.upsells.to_le_bytes());
            bytes.extend(rental.bundle_id.to_le_bytes());
        }
        None => bytes.push(0),
    }
    bytes.extend(item.ability_count.to_le_bytes());
    bytes.push(u8::from(item.activate_enabled));
    bytes
}
/// 147-byte definition row: id, name id (+4), icon (+12), icon tint (+16), class (+32).
fn row(id: i32, name: i32, icon: i32, tint: i32, class: i32) -> Vec<u8> {
    let mut bytes = vec![0u8; 147];
    for (at, value) in [(0, id), (4, name), (12, icon), (16, tint), (32, class)] {
        bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    bytes
}
fn row_json(
    definition: i32,
    name_id: i32,
    description_id: i32,
    cooldown_ms: u32,
    single_use: bool,
    effect: serde_json::Value,
) -> serde_json::Value {
    json!({
        "definition": definition,
        "name_id": name_id,
        "description_id": description_id,
        "cooldown_ms": cooldown_ms,
        "single_use": single_use,
        "effect": effect,
        "source": "T510 test row",
    })
}
fn heal_row(definition: i32, percent: u32, cooldown_ms: u32) -> serde_json::Value {
    row_json(
        definition,
        422909,
        422916,
        cooldown_ms,
        true,
        json!({"heal": {"percent": percent}}),
    )
}
fn aura_row(
    definition: i32,
    duration_ms: u64,
    cooldown_ms: u32,
    composite_effect_id: i32,
    delay_ms: u64,
) -> serde_json::Value {
    row_json(
        definition,
        VEAL_NAME,
        VEAL_DESC,
        cooldown_ms,
        true,
        json!({"aura": {
            "composite_effect_id": composite_effect_id,
            "duration_ms": duration_ms,
            "delay_ms": delay_ms,
            "icon_id": BUFF_ICON,
        }}),
    )
}
fn transform_row(
    definition: i32,
    duration_ms: u64,
    cooldown_ms: u32,
    model_id: i32,
    poof_effect_id: i32,
) -> serde_json::Value {
    row_json(
        definition,
        1201,
        2772,
        cooldown_ms,
        true,
        json!({"transform": {
            "model_id": model_id,
            "poof_effect_id": poof_effect_id,
            "duration_ms": duration_ms,
            "icon_id": BUFF_ICON,
        }}),
    )
}

struct Fixture {
    path: PathBuf,
}
impl Fixture {
    /// Items: 10 Small x3, 11 Small-alt x1, 12 Medium x2, 13 hat, 14 Small x1 tint 7,
    /// 15 Small-alt with its only unit already consumed.
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-item-bar-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        let mut exhausted = item(15, SMALL_ALT, 1, 0);
        exhausted.consumed_count = 1;
        let mut rented = item(18, VEAL, 2, 0);
        rented.rental = Some(RentalState {
            expiration: 0,
            upsells: 0,
            bundle_id: 0,
        });
        let state = InventoryState {
            next_instance_id: None,
            coins: 0,
            items: vec![
                item(10, SMALL, 3, 0),
                item(11, SMALL_ALT, 1, 0),
                item(12, MEDIUM, 2, 0),
                item(13, HAT, 1, 0),
                item(14, SMALL, 1, 7),
                exhausted,
                item(16, VEAL, 2, 0),
                item(17, BERRIES, 2, 0),
                rented,
            ],
            equipment: vec![],
        };
        let mut offset = 13;
        let mut spans = Vec::new();
        let mut span = |kind: &str, profile_id: i32, bytes: Vec<u8>| {
            spans.push(json!({"kind":kind,"offset":offset,"length":bytes.len(),"profile_id":profile_id,"sha256":digest(&bytes)}));
            offset += bytes.len();
        };
        span("coins", 0, 0i32.to_le_bytes().to_vec());
        span("equipment", 1, 0i32.to_le_bytes().to_vec());
        let mut bytes = (state.items.len() as i32).to_le_bytes().to_vec();
        for item in &state.items {
            bytes.extend(serialize_item(item));
        }
        span("items", 0, bytes);
        let rows = [
            row(SMALL, 422909, 730, 5, 9),
            row(SMALL_ALT, 422909, 730, 5, 9),
            row(MEDIUM, 422910, 543, 6, 9),
            row(HAT, 1, 2, 0, 1),
            row(VEAL, VEAL_NAME, 2797, 0, 9),
            row(BERRIES, 1201, 2851, 0, 9),
        ];
        let definitions: Vec<_> = rows.iter().map(|bytes| {
            let id = i32::from_le_bytes(bytes[..4].try_into().unwrap());
            let class = i32::from_le_bytes(bytes[32..36].try_into().unwrap());
            let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
            json!({"id":id,"slot":0,"class_id":class,"default_tint":0,"max_stack":0,"model":"m.adr","texture":"t","tint_alias":"a","effect":0,"wield_type":0,"serialized_hex":hex,"sha256":digest(bytes)})
        }).collect();
        let profiles = json!([{"id":1,"allowed_classes":[1],"slots":[3]}]);
        let content = json!({"schema_version":1,"player_guid":1,"default_state":state,"profiles":profiles,"spans":spans,"definitions":definitions});
        fs::write(
            path.join("inventory.json"),
            serde_json::to_vec(&content).unwrap(),
        )
        .unwrap();
        Self { path }
    }
    fn inventory(&self) -> Inventory {
        Inventory::load(&self.path, None).unwrap()
    }
    fn bar(&self) -> ItemBar {
        ItemBar::load(&self.path, Default::default(), None).unwrap()
    }
    /// Write a `consumables.json` into the fixture's content directory.
    fn write_consumables(&self, value: &serde_json::Value) {
        fs::write(self.path.join("consumables.json"), value.to_string()).unwrap();
    }
    /// Schema 2 content: the two potion rows plus the Electric Veal Meat aura and Bark Berries
    /// transform rows from data/content.
    fn write_schema2(&self) {
        self.write_consumables(&json!({
            "schema_version": 2,
            "consumables": [
                heal_row(SMALL, 35, 10_000),
                heal_row(SMALL_ALT, 35, 10_000),
                aura_row(VEAL, 1_800_000, 1_800_000, 5030, 0),
                transform_row(BERRIES, 1_800_000, 1_800_000, 169, 21),
            ],
        }));
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn msg(main: u16, sub: u16, fields: &[i32]) -> Vec<u8> {
    let mut bytes = [main.to_le_bytes(), sub.to_le_bytes()].concat();
    for field in fields {
        bytes.extend(field.to_le_bytes());
    }
    bytes
}
fn assign(slot: i32, instance: i32) -> Vec<u8> {
    msg(42, 6, &[slot, instance])
}
fn assign_record(slot: i32, definition: i32, tint: i32) -> Vec<u8> {
    msg(42, 9, &[slot, definition, tint])
}
/// 36/10 on `bar`, slot, target mode 0 with `guid`.
fn cast_on(bar: i32, slot: i32, guid: u64) -> Vec<u8> {
    let mut bytes = msg(36, 10, &[bar, slot, 0]);
    bytes.extend(guid.to_le_bytes());
    bytes
}
fn cast(slot: i32) -> Vec<u8> {
    cast_on(2, slot, ME)
}
fn check(id: i32) -> Vec<u8> {
    msg(42, 16, &[id])
}
fn use_item(id: i32) -> Vec<u8> {
    msg(42, 15, &[id])
}

fn run(
    bar: &mut ItemBar,
    inventory: &mut Inventory,
    inner: &[u8],
    player: &Player,
    now_ms: u64,
) -> Option<Reply> {
    bar.handle(inner, inventory, player, now_ms).unwrap()
}
fn slot_updates(reply: &Reply) -> Vec<osfr_tools::item_bar::use_wire::ActionBarSlotUpdate> {
    reply
        .replies
        .iter()
        .filter(|r| r.len() == SLOT_UPDATE_BYTES && r.starts_with(&[38, 0, 25, 0]))
        .map(|r| osfr_tools::item_bar::use_wire::decode_slot_update(r).unwrap())
        .collect()
}
fn count(inventory: &Inventory, id: i32) -> u32 {
    inventory
        .state()
        .items
        .iter()
        .find(|i| i.id == id)
        .map_or(0, |i| i.count)
}
fn is_ability_failure(reply: &Reply) -> bool {
    reply.replies.len() == 1 && reply.replies[0].starts_with(&[36, 0, 1, 0])
}
fn is_chat(reply: &Reply) -> bool {
    reply.replies.len() == 1 && reply.replies[0].starts_with(&[15, 0, 3, 0])
}
/// The 15/3 system line of a chat reply (15/3 header, u32 length, text, flag).
fn chat_text(reply: &Reply) -> String {
    let packet = &reply.replies[0];
    String::from_utf8(packet[8..packet.len() - 1].to_vec()).unwrap()
}
fn bound(bar: &ItemBar) -> Vec<(i32, i32)> {
    bar.bindings()
        .iter()
        .map(|b| (b.slot, b.item_instance_id))
        .collect()
}

#[test]
fn assigning_an_owned_potion_binds_the_slot_and_renders_all_four_slots() {
    let fixture = Fixture::new();
    let (mut inventory, mut bar) = (fixture.inventory(), fixture.bar());
    assert!(bar.is_empty());
    let reply = run(&mut bar, &mut inventory, &assign(1, 10), &ALIVE, 0).unwrap();
    assert!(reply.changed);
    assert_eq!(reply.heal_percent, 0);
    assert_eq!(bound(&bar), [(1, 10)]);
    let slots = slot_updates(&reply);
    assert_eq!(slots.len(), 4);
    for (index, update) in slots.iter().enumerate() {
        assert_eq!((update.bar, update.slot), (2, index as i32));
        assert_eq!(update.is_empty, index != 1);
    }
    let potion = slots[1];
    assert_eq!(
        (potion.icon, potion.icon_tint, potion.name),
        (730, 5, 422909)
    );
    assert_eq!(potion.quantity, 3);
    assert!(potion.server_enabled);
    assert_eq!((potion.selector_a, potion.target_selector), (1, 2));
    assert_eq!(potion.total_refresh_time, 10_000);
    assert_eq!(potion.cast_age_ms, i32::MAX); // never cast
    // The instance tint overrides the definition tint (14 carries tint 7).
    let reply = run(&mut bar, &mut inventory, &assign(2, 14), &ALIVE, 0).unwrap();
    assert_eq!(slot_updates(&reply)[2].icon_tint, 7);
    assert_eq!(bound(&bar), [(1, 10), (2, 14)]);
}

#[test]
fn a_refused_assignment_changes_nothing_and_snaps_the_bar_back() {
    let fixture = Fixture::new();
    let (mut inventory, mut bar) = (fixture.inventory(), fixture.bar());
    run(&mut bar, &mut inventory, &assign(0, 10), &ALIVE, 0).unwrap();
    let before = bar.clone();
    for (slot, instance, why) in [
        (4, 10, "slot past the last"),
        (-1, 10, "negative slot"),
        (1, 999, "not owned"),
        (1, -5, "negative instance"),
        (1, 13, "not a consumable"),
        (1, 15, "only unit already consumed"),
    ] {
        let reply = run(&mut bar, &mut inventory, &assign(slot, instance), &ALIVE, 0).unwrap();
        assert!(!reply.changed, "{why}");
        assert_eq!(bar, before, "{why}");
        // A chat line, then the authoritative bar (slot 0 still holds the potion).
        assert!(reply.replies[0].starts_with(&[15, 0, 3, 0]), "{why}");
        let slots = slot_updates(&reply);
        assert_eq!(slots.len(), 4, "{why}");
        assert!(!slots[0].is_empty && slots[1].is_empty, "{why}");
    }
    // Canary: the same slot takes a valid, owned potion.
    assert!(
        run(&mut bar, &mut inventory, &assign(1, 11), &ALIVE, 0)
            .unwrap()
            .changed
    );
}

#[test]
fn assigning_zero_clears_and_clearing_an_empty_slot_is_a_no_op() {
    let fixture = Fixture::new();
    let (mut inventory, mut bar) = (fixture.inventory(), fixture.bar());
    let reply = run(&mut bar, &mut inventory, &assign(3, 0), &ALIVE, 0).unwrap();
    assert!(!reply.changed);
    run(&mut bar, &mut inventory, &assign(3, 10), &ALIVE, 0).unwrap();
    let reply = run(&mut bar, &mut inventory, &assign(3, 0), &ALIVE, 0).unwrap();
    assert!(reply.changed && bar.is_empty());
    assert!(slot_updates(&reply).iter().all(|s| s.is_empty));
}

#[test]
fn assigning_by_record_resolves_definition_and_tint_to_an_owned_stack() {
    let fixture = Fixture::new();
    let (mut inventory, mut bar) = (fixture.inventory(), fixture.bar());
    run(
        &mut bar,
        &mut inventory,
        &assign_record(0, SMALL, 0),
        &ALIVE,
        0,
    )
    .unwrap();
    run(
        &mut bar,
        &mut inventory,
        &assign_record(1, SMALL, 7),
        &ALIVE,
        0,
    )
    .unwrap();
    assert_eq!(bound(&bar), [(0, 10), (1, 14)]);
    let before = bar.clone();
    for (slot, definition, tint, why) in [
        (2, SMALL, 99, "no stack with that tint"),
        (2, 424242, 0, "no such definition"),
        (2, HAT, 0, "not a consumable"),
        (9, SMALL, 0, "slot out of range"),
    ] {
        let reply = run(
            &mut bar,
            &mut inventory,
            &assign_record(slot, definition, tint),
            &ALIVE,
            0,
        )
        .unwrap();
        assert!(!reply.changed, "{why}");
        assert_eq!(bar, before, "{why}");
    }
    // Canary: the exhausted stack never matches, but a live one with the same definition does.
    assert!(
        run(
            &mut bar,
            &mut inventory,
            &assign_record(2, SMALL_ALT, 0),
            &ALIVE,
            0
        )
        .unwrap()
        .changed
    );
    assert_eq!(bound(&bar), [(0, 10), (1, 14), (2, 11)]);
    // 42/9 is exactly 16 bytes.
    for bad in [
        &assign_record(0, SMALL, 0)[..15],
        &[assign_record(0, SMALL, 0), vec![0]].concat()[..],
    ] {
        assert!(bar.handle(bad, &mut inventory, &ALIVE, 0).is_err());
    }
}

#[test]
fn malformed_and_foreign_messages_are_not_taken() {
    let fixture = Fixture::new();
    let (mut inventory, mut bar) = (fixture.inventory(), fixture.bar());
    assert!(
        bar.handle(&assign(0, 10)[..11], &mut inventory, &ALIVE, 0)
            .is_err()
    );
    assert!(
        bar.handle(
            &[assign(0, 10), vec![0]].concat(),
            &mut inventory,
            &ALIVE,
            0
        )
        .is_err()
    );
    assert!(
        bar.handle(&cast_on(2, 0, ME)[..20], &mut inventory, &ALIVE, 0)
            .is_err()
    );
    for foreign in [
        msg(42, 8, &[1, 0, 1, 3]), // equip by record
        msg(42, 3, &[10, 1, 3]),   // equip
        cast_on(1, 0, ME),         // the job bar stays with its existing route
        cast_on(3, 0, ME),         // other bars too
        check(13),                 // a hat is not ours: the chest route keeps 42/16
        use_item(13),
        check(999), // unknown instance: not ours either
        msg(26, 10, &[1, 2, 3]),
    ] {
        assert!(run(&mut bar, &mut inventory, &foreign, &ALIVE, 0).is_none());
    }
}

#[test]
fn using_a_bar_slot_spends_exactly_one_potion_and_asks_for_the_heal() {
    let fixture = Fixture::new();
    let (mut inventory, mut bar) = (fixture.inventory(), fixture.bar());
    run(&mut bar, &mut inventory, &assign(0, 10), &ALIVE, 0).unwrap();
    let reply = run(&mut bar, &mut inventory, &cast(0), &ALIVE, 5_000).unwrap();
    assert!(reply.changed);
    assert_eq!(reply.heal_percent, 35);
    assert_eq!(count(&inventory, 10), 2);
    assert!(reply.replies[0].starts_with(&[38, 0, 3, 0])); // item update with the new count
    let slot = slot_updates(&reply)[0];
    assert_eq!(slot.quantity, 2);
    assert_eq!((slot.total_refresh_time, slot.cast_age_ms), (10_000, 0));
    // Guid 0 ("no target") is the same as self.
    let mut inventory2 = fixture.inventory();
    let mut bar2 = fixture.bar();
    run(&mut bar2, &mut inventory2, &assign(0, 10), &ALIVE, 0).unwrap();
    assert_eq!(
        run(&mut bar2, &mut inventory2, &cast_on(2, 0, 0), &ALIVE, 0)
            .unwrap()
            .heal_percent,
        35
    );
}

#[test]
fn the_recast_window_blocks_replays_and_ends_on_its_boundary() {
    let fixture = Fixture::new();
    let (mut inventory, mut bar) = (fixture.inventory(), fixture.bar());
    run(&mut bar, &mut inventory, &assign(0, 10), &ALIVE, 0).unwrap();
    run(&mut bar, &mut inventory, &assign(1, 14), &ALIVE, 0).unwrap(); // same definition
    assert_eq!(
        run(&mut bar, &mut inventory, &cast(0), &ALIVE, 1_000)
            .unwrap()
            .heal_percent,
        35
    );
    for now in [1_000, 1_001, 10_999] {
        for slot in [0, 1] {
            let reply = run(&mut bar, &mut inventory, &cast(slot), &ALIVE, now).unwrap();
            assert!(is_ability_failure(&reply), "{now} slot {slot}");
            assert!(!reply.changed && reply.heal_percent == 0);
        }
    }
    assert_eq!(count(&inventory, 10), 2);
    assert_eq!(count(&inventory, 14), 1);
    // The next slot has to wait for the same item's window, then it works.
    let reply = run(&mut bar, &mut inventory, &cast(1), &ALIVE, 11_000).unwrap();
    assert_eq!(reply.heal_percent, 35);
    // The slot shows the running recast to the client.
    let slot = slot_updates(&reply)[0];
    assert_eq!(slot.cast_age_ms, 0);
    // A clock that restarted counts as ready instead of blocking forever.
    assert_eq!(
        run(&mut bar, &mut inventory, &cast(0), &ALIVE, 0)
            .unwrap()
            .heal_percent,
        35
    );
}

#[test]
fn the_last_potion_unbinds_its_slot() {
    let fixture = Fixture::new();
    let (mut inventory, mut bar) = (fixture.inventory(), fixture.bar());
    run(&mut bar, &mut inventory, &assign(2, 11), &ALIVE, 0).unwrap();
    let reply = run(&mut bar, &mut inventory, &cast(2), &ALIVE, 0).unwrap();
    assert_eq!(reply.heal_percent, 35);
    assert!(reply.replies[0].starts_with(&[38, 0, 4, 0])); // item removed
    assert!(inventory.state().items.iter().all(|i| i.id != 11));
    assert!(bar.is_empty());
    assert!(slot_updates(&reply).iter().all(|s| s.is_empty));
    // Nothing is left to use, and a new request can't spend what is gone.
    let reply = run(&mut bar, &mut inventory, &cast(2), &ALIVE, 60_000).unwrap();
    assert!(is_ability_failure(&reply) && reply.heal_percent == 0);
}

#[test]
fn bar_use_refusals_spend_nothing_and_each_has_a_canary() {
    let fixture = Fixture::new();
    let (mut inventory, mut bar) = (fixture.inventory(), fixture.bar());
    run(&mut bar, &mut inventory, &assign(0, 10), &ALIVE, 0).unwrap();
    let mut position = msg(36, 10, &[2, 0, 1]);
    for axis in [1.0f32, 2.0, 3.0, 1.0] {
        position.extend(axis.to_le_bytes());
    }
    let refused: [(&str, Vec<u8>, &Player); 6] = [
        ("empty slot", cast(2), &ALIVE),
        ("slot past the last", cast(4), &ALIVE),
        ("negative slot", cast(-1), &ALIVE),
        ("someone else's guid", cast_on(2, 0, ME + 1), &ALIVE),
        ("ground target", position, &ALIVE),
        ("knocked out", cast(0), &KNOCKED_OUT),
    ];
    for (why, request, player) in refused {
        let reply = run(&mut bar, &mut inventory, &request, player, 0).unwrap();
        assert!(is_ability_failure(&reply), "{why}");
        assert!(!reply.changed && reply.heal_percent == 0, "{why}");
        assert_eq!(
            (count(&inventory, 10), count(&inventory, 12)),
            (3, 2),
            "{why}"
        );
    }
    // Canary: the unmodified request works for the same slot and player.
    assert_eq!(
        run(&mut bar, &mut inventory, &cast(0), &ALIVE, 0)
            .unwrap()
            .heal_percent,
        35
    );
    // A definition without a row can no longer take a slot (T510 replaces the catalogue gate).
    let reply = run(&mut bar, &mut inventory, &assign(1, 12), &ALIVE, 0).unwrap();
    assert!(!reply.changed && bound(&bar) == [(0, 10)]);
    assert!(slot_updates(&reply)[1].is_empty);
}

#[test]
fn the_check_then_use_flow_confirms_once_and_only_for_the_checked_instance() {
    let fixture = Fixture::new();
    let (mut inventory, mut bar) = (fixture.inventory(), fixture.bar());
    // Use without a check is refused.
    let reply = run(&mut bar, &mut inventory, &use_item(10), &ALIVE, 0).unwrap();
    assert!(is_chat(&reply) && !reply.changed && count(&inventory, 10) == 3);
    // The check answers with the item's own name and description, success set.
    let reply = run(&mut bar, &mut inventory, &check(10), &ALIVE, 0).unwrap();
    let packet = &reply.replies[0];
    assert_eq!(packet.len(), 25);
    assert_eq!(&packet[..4], &[38, 0, 30, 0]);
    assert_eq!(i32::from_le_bytes(packet[4..8].try_into().unwrap()), 10);
    assert_eq!(
        i32::from_le_bytes(packet[16..20].try_into().unwrap()),
        422909
    );
    assert_eq!(
        i32::from_le_bytes(packet[20..24].try_into().unwrap()),
        422916
    );
    assert_eq!(packet[24], 1);
    assert_eq!(count(&inventory, 10), 3); // a check never spends
    // A use for a different instance is refused, and a refusal changes nothing: the checked
    // instance's confirmation stays pending, so its own 42/15 still confirms.
    let reply = run(&mut bar, &mut inventory, &use_item(11), &ALIVE, 1).unwrap();
    assert!(is_chat(&reply) && count(&inventory, 11) == 1);
    let reply = run(&mut bar, &mut inventory, &use_item(10), &ALIVE, 1).unwrap();
    assert_eq!((reply.heal_percent, count(&inventory, 10)), (35, 2));
    // Canary: after the recast a new check and use spend one and heal again.
    run(&mut bar, &mut inventory, &check(10), &ALIVE, 20_000).unwrap();
    let reply = run(&mut bar, &mut inventory, &use_item(10), &ALIVE, 20_000).unwrap();
    assert_eq!((reply.heal_percent, count(&inventory, 10)), (35, 1));
    // A replayed 42/15 needs a new check.
    let reply = run(&mut bar, &mut inventory, &use_item(10), &ALIVE, 30_000).unwrap();
    assert!(is_chat(&reply) && reply.heal_percent == 0 && count(&inventory, 10) == 1);
}

#[test]
fn the_check_refuses_with_a_chat_line_not_the_chest_dialog() {
    let fixture = Fixture::new();
    let (mut inventory, mut bar) = (fixture.inventory(), fixture.bar());
    for (why, id, player) in [("exhausted", 15, &ALIVE), ("knocked out", 10, &KNOCKED_OUT)] {
        let reply = run(&mut bar, &mut inventory, &check(id), player, 0).unwrap();
        assert!(is_chat(&reply), "{why}"); // never a 38/30 whose Okay opens the marketplace
        let reply = run(&mut bar, &mut inventory, &use_item(id), player, 0).unwrap();
        assert!(is_chat(&reply) && reply.heal_percent == 0, "{why}");
    }
    // During the recast the check is refused; once it ends the same check succeeds.
    run(&mut bar, &mut inventory, &check(10), &ALIVE, 0).unwrap();
    run(&mut bar, &mut inventory, &use_item(10), &ALIVE, 0).unwrap();
    let reply = run(&mut bar, &mut inventory, &check(10), &ALIVE, 9_999).unwrap();
    assert!(is_chat(&reply));
    let reply = run(&mut bar, &mut inventory, &check(10), &ALIVE, 10_000).unwrap();
    assert!(reply.replies[0].starts_with(&[38, 0, 30, 0]));
}

#[test]
fn the_bar_persists_and_runtime_state_does_not() {
    let fixture = Fixture::new();
    let (mut inventory, mut bar) = (fixture.inventory(), fixture.bar());
    run(&mut bar, &mut inventory, &assign(0, 10), &ALIVE, 0).unwrap();
    run(&mut bar, &mut inventory, &assign(3, 14), &ALIVE, 0).unwrap();
    run(&mut bar, &mut inventory, &cast(0), &ALIVE, 0).unwrap();
    let saved = serde_json::to_string(&bar).unwrap();
    let mut loaded = ItemBar::load(
        &fixture.path,
        Default::default(),
        Some(serde_json::from_str(&saved).unwrap()),
    )
    .unwrap();
    assert_eq!(bound(&loaded), [(0, 10), (3, 14)]);
    // The recast is not saved: a relog can use again at once.
    assert_eq!(
        run(&mut loaded, &mut inventory, &cast(0), &ALIVE, 0)
            .unwrap()
            .heal_percent,
        35
    );
    // The first refresh after load shows every slot and drops a binding whose stack is gone.
    let mut gone = fixture.inventory();
    gone.consume(14, 1).unwrap();
    let updates = loaded.refresh(&gone, 0).unwrap();
    assert_eq!(updates.len(), 4);
    assert_eq!(bound(&loaded), [(0, 10)]);
    // An empty bar adds nothing to be saved.
    assert!(fixture.bar().is_empty());
}

#[test]
fn a_tampered_saved_bar_is_refused_at_load() {
    let fixture = Fixture::new();
    let good = json!({"bar":{"version":1,"bindings":[{"slot":0,"item_instance_id":10,"definition_id":2173}]}});
    let loaded: ItemBar = serde_json::from_value(good.clone()).unwrap();
    assert!(ItemBar::load(&fixture.path, Default::default(), Some(loaded)).is_ok()); // canary
    for (why, value) in [
        (
            "slot out of range",
            json!({"bar":{"version":1,"bindings":[{"slot":4,"item_instance_id":10,"definition_id":2173}]}}),
        ),
        ("version", json!({"bar":{"version":2,"bindings":[]}})),
        (
            "five bindings",
            json!({"bar":{"version":1,"bindings":[
            {"slot":0,"item_instance_id":1,"definition_id":1},{"slot":1,"item_instance_id":1,"definition_id":1},
            {"slot":2,"item_instance_id":1,"definition_id":1},{"slot":3,"item_instance_id":1,"definition_id":1},
            {"slot":3,"item_instance_id":1,"definition_id":1}]}}),
        ),
        (
            "duplicate slot",
            json!({"bar":{"version":1,"bindings":[
            {"slot":1,"item_instance_id":10,"definition_id":2173},{"slot":1,"item_instance_id":11,"definition_id":2173}]}}),
        ),
        (
            "instance",
            json!({"bar":{"version":1,"bindings":[{"slot":0,"item_instance_id":0,"definition_id":2173}]}}),
        ),
    ] {
        let loaded: ItemBar = serde_json::from_value(value).unwrap();
        assert!(
            ItemBar::load(&fixture.path, Default::default(), Some(loaded)).is_err(),
            "{why}"
        );
    }
    let mut unknown = good;
    unknown["extra"] = json!(1);
    assert!(serde_json::from_value::<ItemBar>(unknown).is_err());
}

#[test]
fn schema_one_still_loads_the_potion_rule() {
    let fixture = Fixture::new();
    // Absent file: the built-in Small Health Potion rows, unchanged.
    let builtin = Effects::load(&fixture.path).unwrap();
    let small = builtin.get(SMALL).expect("built-in potion row");
    assert!(matches!(small.effect, EffectKind::Heal { percent: 35 }));
    assert_eq!(builtin.get(MEDIUM), None);
    // Schema 1 (the old potion-only shape) still loads and its values still apply.
    fixture.write_consumables(&json!({
        "schema_version": 1,
        "consumables": [
            {"definition": SMALL, "heal_percent": 50, "cooldown_ms": 0},
            {"definition": MEDIUM, "heal_percent": 20, "cooldown_ms": 1000},
        ],
    }));
    let mut bar = ItemBar::load(&fixture.path, Default::default(), None).unwrap();
    let mut inventory = fixture.inventory();
    run(&mut bar, &mut inventory, &assign(0, 10), &ALIVE, 0).unwrap();
    run(&mut bar, &mut inventory, &assign(1, 12), &ALIVE, 0).unwrap();
    let used = run(&mut bar, &mut inventory, &cast(0), &ALIVE, 3_000).unwrap();
    assert_eq!(used.heal_percent, 50);
    let used = run(&mut bar, &mut inventory, &cast(1), &ALIVE, 3_000).unwrap();
    assert_eq!(used.heal_percent, 20);
    // The 3 s floor applies to schema-1 rows too (the row says 1000 ms).
    assert!(is_ability_failure(
        &run(&mut bar, &mut inventory, &cast(1), &ALIVE, 5_999).unwrap()
    ));
    assert_eq!(
        run(&mut bar, &mut inventory, &cast(1), &ALIVE, 6_000)
            .unwrap()
            .heal_percent,
        20
    );
    // Schema 1 still refuses a definition outside the recovered catalogue.
    fixture.write_consumables(&json!({
        "schema_version": 1,
        "consumables": [{"definition": HAT, "heal_percent": 10, "cooldown_ms": 0}],
    }));
    assert!(Effects::load(&fixture.path).is_err());
}

#[test]
fn schema_two_rows_load_within_sanity_bounds() {
    let fixture = Fixture::new();
    let write = |value: serde_json::Value| {
        fixture.write_consumables(&value);
        Effects::load(&fixture.path)
    };
    fixture.write_schema2();
    let effects = Effects::load(&fixture.path).unwrap();
    assert_eq!(effects.rows().len(), 4);
    assert!(matches!(
        effects.get(VEAL).unwrap().effect,
        EffectKind::Aura {
            composite_effect_id: 5030,
            duration_ms: 1_800_000,
            delay_ms: 0,
            icon_id: BUFF_ICON,
        }
    ));
    assert!(matches!(
        effects.get(BERRIES).unwrap().effect,
        EffectKind::Transform {
            model_id: 169,
            poof_effect_id: 21,
            duration_ms: 1_800_000,
            icon_id: BUFF_ICON,
        }
    ));
    for definition in [SMALL, SMALL_ALT] {
        let row = effects.get(definition).unwrap();
        assert!(matches!(row.effect, EffectKind::Heal { percent: 35 }));
        assert!(row.single_use);
    }
    // Every loaded row is admissible for the effect stack, so an `apply` after a spend cannot
    // fail (the only stack refusal is this pure check, and content bounds are stricter).
    for row in effects.rows() {
        if let Some(spec) = row.spec() {
            assert_eq!(
                EffectStack::default().admits(&spec),
                Ok(()),
                "definition {}",
                row.definition
            );
        }
    }
    // Each refused shape names its reason; the canary at the end is the same shape accepted.
    let row = |effect: serde_json::Value| {
        json!([{
            "definition": VEAL,
            "name_id": VEAL_NAME,
            "description_id": VEAL_DESC,
            "cooldown_ms": 1000,
            "single_use": true,
            "effect": effect,
            "source": "T510 bounds",
        }])
    };
    for (why, value) in [
        ("cooldown over 24 h", {
            let mut row = row(json!({"heal": {"percent": 10}}));
            row[0]["cooldown_ms"] = json!(86_400_001);
            json!({"schema_version": 2, "consumables": row})
        }),
        (
            "heal 0",
            json!({"schema_version": 2, "consumables": row(json!({"heal": {"percent": 0}}))}),
        ),
        (
            "heal over 100",
            json!({"schema_version": 2, "consumables": row(json!({"heal": {"percent": 101}}))}),
        ),
        (
            "unknown effect field",
            json!({"schema_version": 2, "consumables": row(json!({"heal": {"percent": 10, "extra": 1}}))}),
        ),
        ("definition 0", {
            let mut rows = row(json!({"heal": {"percent": 10}}));
            rows[0]["definition"] = json!(0);
            json!({"schema_version": 2, "consumables": rows})
        }),
        ("name 0", {
            let mut rows = row(json!({"heal": {"percent": 10}}));
            rows[0]["name_id"] = json!(0);
            json!({"schema_version": 2, "consumables": rows})
        }),
        ("description 0", {
            let mut rows = row(json!({"heal": {"percent": 10}}));
            rows[0]["description_id"] = json!(0);
            json!({"schema_version": 2, "consumables": rows})
        }),
        ("missing source", {
            let mut rows = row(json!({"heal": {"percent": 10}}));
            rows[0].as_object_mut().unwrap().remove("source");
            json!({"schema_version": 2, "consumables": rows})
        }),
        ("unknown row field", {
            let mut rows = row(json!({"heal": {"percent": 10}}));
            rows[0]["extra"] = json!(1);
            json!({"schema_version": 2, "consumables": rows})
        }),
        (
            "aura composite 0",
            json!({"schema_version": 2, "consumables": row(json!({"aura": {"composite_effect_id": 0, "duration_ms": 1, "delay_ms": 0, "icon_id": BUFF_ICON}}))}),
        ),
        (
            "aura icon 0",
            json!({"schema_version": 2, "consumables": row(json!({"aura": {"composite_effect_id": 5030, "duration_ms": 1, "delay_ms": 0, "icon_id": 0}}))}),
        ),
        (
            "aura duration 0",
            json!({"schema_version": 2, "consumables": row(json!({"aura": {"composite_effect_id": 5030, "duration_ms": 0, "delay_ms": 0, "icon_id": BUFF_ICON}}))}),
        ),
        (
            "aura duration over 24 h",
            json!({"schema_version": 2, "consumables": row(json!({"aura": {"composite_effect_id": 5030, "duration_ms": 86_400_001, "delay_ms": 0, "icon_id": BUFF_ICON}}))}),
        ),
        (
            "aura delay over the stack bound",
            json!({"schema_version": 2, "consumables": row(json!({"aura": {"composite_effect_id": 5030, "duration_ms": 1, "delay_ms": 10_001, "icon_id": BUFF_ICON}}))}),
        ),
        (
            "transform model 0",
            json!({"schema_version": 2, "consumables": row(json!({"transform": {"model_id": 0, "poof_effect_id": 21, "duration_ms": 1, "icon_id": BUFF_ICON}}))}),
        ),
        (
            "transform negative poof",
            json!({"schema_version": 2, "consumables": row(json!({"transform": {"model_id": 169, "poof_effect_id": -1, "duration_ms": 1, "icon_id": BUFF_ICON}}))}),
        ),
        (
            "transform icon 0",
            json!({"schema_version": 2, "consumables": row(json!({"transform": {"model_id": 169, "poof_effect_id": 21, "duration_ms": 1, "icon_id": 0}}))}),
        ),
        (
            "scale under the stack bound",
            json!({"schema_version": 2, "consumables": row(json!({"scale": {"scale": 0.24, "duration_ms": 1, "icon_id": BUFF_ICON}}))}),
        ),
        (
            "scale over the stack bound",
            json!({"schema_version": 2, "consumables": row(json!({"scale": {"scale": 5.01, "duration_ms": 1, "icon_id": BUFF_ICON}}))}),
        ),
        (
            "scale duration 0",
            json!({"schema_version": 2, "consumables": row(json!({"scale": {"scale": 2.0, "duration_ms": 0, "icon_id": BUFF_ICON}}))}),
        ),
        (
            "unknown effect",
            json!({"schema_version": 2, "consumables": row(json!({"teleport": {}}))}),
        ),
        (
            "duplicate",
            json!({"schema_version": 2, "consumables": [
                row(json!({"heal": {"percent": 10}}))[0].clone(),
                row(json!({"heal": {"percent": 20}}))[0].clone(),
            ]}),
        ),
        ("schema 3", json!({"schema_version": 3, "consumables": []})),
        (
            "unknown file field",
            json!({"schema_version": 2, "consumables": [], "extra": 1}),
        ),
    ] {
        assert!(write(value).is_err(), "{why}");
    }
    // Canary: the same row shape at the exact bounds loads.
    assert!(
        write(
            json!({"schema_version": 2, "consumables": row(json!({"aura": {
                "composite_effect_id": 5030,
                "duration_ms": 86_400_000,
                "delay_ms": 10_000,
                "icon_id": BUFF_ICON,
            }}))})
        )
        .is_ok()
    );
    assert!(
        write(json!({"schema_version": 2, "consumables": [{
            "definition": VEAL,
            "name_id": VEAL_NAME,
            "description_id": VEAL_DESC,
            "cooldown_ms": 86_400_000,
            "single_use": true,
            "effect": {"heal": {"percent": 100}},
            "source": "T510 bounds",
        }]}))
        .is_ok()
    );
    // Row count and file size bounds. 512 is accepted, 513 is not.
    let rows = |count: i32| (1..=count).map(effect).collect::<Vec<_>>();
    assert!(Effects::new(rows(512)).is_ok());
    assert!(Effects::new(rows(513)).is_err());
    fs::write(
        fixture.path.join("consumables.json"),
        vec![b'x'; 256 * 1024 + 1],
    )
    .unwrap();
    assert!(Effects::load(&fixture.path).is_err());
}

/// A valid schema-2 row for the row-count bound.
fn effect(definition: i32) -> Effect {
    Effect {
        definition,
        name_id: VEAL_NAME,
        description_id: VEAL_DESC,
        cooldown_ms: 0,
        single_use: true,
        effect: EffectKind::Heal { percent: 10 },
        source: "T510 bounds".into(),
    }
}

#[test]
fn the_shipped_rows_all_fit_the_effect_stack() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let effects = Effects::load(&directory).unwrap();
    assert!(effects.get(VEAL).is_some() && effects.get(BERRIES).is_some());
    for row in effects.rows() {
        if let Some(spec) = row.spec() {
            assert_eq!(
                EffectStack::default().admits(&spec),
                Ok(()),
                "definition {}",
                row.definition
            );
        }
    }
}

#[test]
fn a_schema_two_aura_row_is_used_and_sweeps_its_thirty_minute_recast() {
    let fixture = Fixture::new();
    fixture.write_schema2();
    let (mut inventory, mut bar) = (fixture.inventory(), fixture.bar());
    run(&mut bar, &mut inventory, &assign(0, 16), &ALIVE, 0).unwrap();
    let used = run(&mut bar, &mut inventory, &cast(0), &ALIVE, 3_000).unwrap();
    assert!(used.changed && used.heal_percent == 0);
    assert_eq!(
        used.effect,
        Some(Spec {
            kind: Kind::Aura {
                composite_effect_id: 5030,
                delay_ms: 0,
            },
            duration_ms: 1_800_000,
            icon_id: BUFF_ICON,
            name_id: VEAL_NAME,
        })
    );
    assert_eq!(count(&inventory, 16), 1);
    let slot = slot_updates(&used)[0];
    assert_eq!((slot.quantity, slot.total_refresh_time), (1, 1_800_000));
    assert_eq!((slot.icon, slot.name), (2797, VEAL_NAME));
    // The 30-minute window refuses the bar and the 42/16 check alike, changing nothing.
    let before = (bar.clone(), count(&inventory, 16));
    for request in [cast(0), check(16)] {
        let refused = run(&mut bar, &mut inventory, &request, &ALIVE, 1_802_999).unwrap();
        assert!(refused.effect.is_none() && !refused.changed);
        assert_eq!((bar.clone(), count(&inventory, 16)), before);
    }
    // Canary: exactly on the boundary the same bar request works.
    let ready = run(&mut bar, &mut inventory, &cast(0), &ALIVE, 1_803_000).unwrap();
    assert!(ready.changed && ready.effect.is_some() && count(&inventory, 16) == 0);
}

#[test]
fn a_transform_row_is_refused_while_a_transform_is_live() {
    let fixture = Fixture::new();
    fixture.write_consumables(&json!({
        "schema_version": 2,
        "consumables": [
            heal_row(SMALL, 35, 10_000),
            transform_row(BERRIES, 60_000, 0, 169, 21),
        ],
    }));
    let (mut inventory, mut bar) = (fixture.inventory(), fixture.bar());
    run(&mut bar, &mut inventory, &assign(1, 17), &ALIVE, 0).unwrap();
    let used = run(&mut bar, &mut inventory, &cast(1), &ALIVE, 0).unwrap();
    assert_eq!(
        used.effect,
        Some(Spec {
            kind: Kind::Transform {
                model_id: 169,
                poof_effect_id: 21,
            },
            duration_ms: 60_000,
            icon_id: BUFF_ICON,
            name_id: 1201,
        })
    );
    assert_eq!(count(&inventory, 17), 1);
    // At the 3 s floor the recast is over, but a live transform still refuses the treat.
    let before = (bar.clone(), count(&inventory, 17));
    let refused = run(&mut bar, &mut inventory, &cast(1), &TRANSFORMED, 3_000).unwrap();
    assert!(is_ability_failure(&refused) && refused.effect.is_none());
    assert_eq!((bar.clone(), count(&inventory, 17)), before);
    // The 42/16 check answers the brief's chat line and also changes nothing.
    let refused = run(&mut bar, &mut inventory, &check(17), &TRANSFORMED, 3_000).unwrap();
    assert!(is_chat(&refused));
    assert_eq!(chat_text(&refused), "You are already transformed.");
    assert_eq!((bar.clone(), count(&inventory, 17)), before);
    // Canary: the same request with no live transform succeeds.
    let ready = run(&mut bar, &mut inventory, &cast(1), &ALIVE, 3_000).unwrap();
    assert!(ready.changed && ready.effect.is_some() && count(&inventory, 17) == 0);
}

#[test]
fn a_refused_check_keeps_the_pending_confirmation() {
    let fixture = Fixture::new();
    fixture.write_consumables(&json!({
        "schema_version": 2,
        "consumables": [
            heal_row(SMALL, 35, 10_000),
            transform_row(BERRIES, 60_000, 0, 169, 21),
        ],
    }));
    let (mut inventory, mut bar) = (fixture.inventory(), fixture.bar());
    let checked = run(&mut bar, &mut inventory, &check(10), &ALIVE, 0).unwrap();
    assert!(checked.replies[0].starts_with(&[38, 0, 30, 0]));
    // A refused check for another item is a refusal, so it must not drop that confirmation.
    let refused = run(&mut bar, &mut inventory, &check(17), &TRANSFORMED, 0).unwrap();
    assert!(is_chat(&refused));
    let used = run(&mut bar, &mut inventory, &use_item(10), &ALIVE, 0).unwrap();
    assert_eq!((used.heal_percent, count(&inventory, 10)), (35, 2));
}

#[test]
fn a_heal_row_needs_health_tracking_and_a_visual_row_does_not() {
    let fixture = Fixture::new();
    let (mut inventory, mut bar) = (fixture.inventory(), fixture.bar());
    run(&mut bar, &mut inventory, &assign(0, 10), &ALIVE, 0).unwrap();
    let refused = run(&mut bar, &mut inventory, &cast(0), &NO_HEALING, 0).unwrap();
    assert!(is_ability_failure(&refused) && count(&inventory, 10) == 3);
    // Aura rows apply from the row alone; only a heal needs the encounter engine (R).
    fixture.write_schema2();
    let mut bar = fixture.bar();
    let mut inventory = fixture.inventory();
    run(&mut bar, &mut inventory, &assign(0, 16), &NO_HEALING, 0).unwrap();
    let used = run(&mut bar, &mut inventory, &cast(0), &NO_HEALING, 0).unwrap();
    assert!(used.effect.is_some() && used.heal_percent == 0 && count(&inventory, 16) == 1);
}

#[test]
fn a_rental_item_with_a_row_stays_unusable() {
    let fixture = Fixture::new();
    fixture.write_schema2();
    let (mut inventory, mut bar) = (fixture.inventory(), fixture.bar());
    run(&mut bar, &mut inventory, &assign(3, 18), &ALIVE, 0).unwrap();
    let before = (bar.clone(), count(&inventory, 18));
    let refused = run(&mut bar, &mut inventory, &cast(3), &ALIVE, 0).unwrap();
    assert!(is_ability_failure(&refused) && refused.effect.is_none());
    assert_eq!((bar, count(&inventory, 18)), before);
}

#[test]
fn hostile_bytes_never_panic() {
    let fixture = Fixture::new();
    let (mut inventory, mut bar) = (fixture.inventory(), fixture.bar());
    run(&mut bar, &mut inventory, &assign(0, 10), &ALIVE, 0).unwrap();
    let mut seed = 0x9E37_79B9_7F4A_7C15u64;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let prefixes: [&[u8]; 6] = [
        &[42, 0, 6, 0],
        &[42, 0, 9, 0],
        &[42, 0, 15, 0],
        &[42, 0, 16, 0],
        &[36, 0, 10, 0, 2, 0, 0, 0],
        &[36, 0, 10, 0],
    ];
    for prefix in prefixes {
        for length in 0..48 {
            for _ in 0..40 {
                let mut bytes = prefix.to_vec();
                while bytes.len() < length {
                    bytes.push(next() as u8);
                }
                bytes.truncate(length);
                let _ = bar.handle(&bytes, &mut inventory, &ALIVE, next() % 100_000);
            }
        }
    }
    // Extreme values are bounds-checked, not trusted.
    for slot in [i32::MIN, i32::MAX] {
        for instance in [i32::MIN, i32::MAX, 0] {
            let _ = bar.handle(&assign(slot, instance), &mut inventory, &ALIVE, u64::MAX);
        }
        let _ = bar.handle(
            &assign_record(slot, i32::MAX, i32::MIN),
            &mut inventory,
            &ALIVE,
            u64::MAX,
        );
        let _ = bar.handle(
            &cast_on(2, slot, u64::MAX),
            &mut inventory,
            &ALIVE,
            u64::MAX,
        );
    }
    assert!(bar_wire::ITEM_BAR_SLOTS == 4);
    assert_eq!(count(&inventory, 10), 3); // nothing hostile spent a potion
}
