//! W146: item bar assignment (42/6, 42/9), bar state (38/25), item-bar use (36/10) and the
//! immediate-use check (42/16 to 38/30, 42/15) for the Small Health Potion. Every refusal has a
//! canary: the same request succeeds once the one missing fact is supplied.
use osfr_tools::{
    inventory::{Inventory, InventoryItem, InventoryState},
    item_bar::{
        bar_wire::{self, SLOT_UPDATE_BYTES},
        Effect, Effects, ItemBar, Player, Reply,
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
const MEDIUM: i32 = 3021; // recovered consumable, no effect yet: assignable, not usable
const HAT: i32 = 200; // not a consumable
const ME: u64 = 77;
const ALIVE: Player = Player {
    guid: ME,
    alive: true,
};
const KNOCKED_OUT: Player = Player {
    guid: ME,
    alive: false,
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
    bytes.push(0); // no rental
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
    assert!(bar
        .handle(&assign(0, 10)[..11], &mut inventory, &ALIVE, 0)
        .is_err());
    assert!(bar
        .handle(
            &[assign(0, 10), vec![0]].concat(),
            &mut inventory,
            &ALIVE,
            0
        )
        .is_err());
    assert!(bar
        .handle(&cast_on(2, 0, ME)[..20], &mut inventory, &ALIVE, 0)
        .is_err());
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
    run(&mut bar, &mut inventory, &assign(1, 12), &ALIVE, 0).unwrap(); // Medium: no effect
    let mut position = msg(36, 10, &[2, 0, 1]);
    for axis in [1.0f32, 2.0, 3.0, 1.0] {
        position.extend(axis.to_le_bytes());
    }
    let refused: [(&str, Vec<u8>, &Player); 7] = [
        ("empty slot", cast(2), &ALIVE),
        ("slot past the last", cast(4), &ALIVE),
        ("negative slot", cast(-1), &ALIVE),
        ("someone else's guid", cast_on(2, 0, ME + 1), &ALIVE),
        ("ground target", position, &ALIVE),
        ("knocked out", cast(0), &KNOCKED_OUT),
        ("no effect recovered yet", cast(1), &ALIVE),
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
    // A medium potion can sit on the bar, greyed out.
    let reply = run(&mut bar, &mut inventory, &assign(1, 12), &ALIVE, 0).unwrap();
    assert!(!slot_updates(&reply)[1].server_enabled);
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
                                          // A use for a different instance neither works nor keeps the confirmation.
    let reply = run(&mut bar, &mut inventory, &use_item(11), &ALIVE, 1).unwrap();
    assert!(is_chat(&reply) && count(&inventory, 11) == 1);
    let reply = run(&mut bar, &mut inventory, &use_item(10), &ALIVE, 1).unwrap();
    assert!(is_chat(&reply) && count(&inventory, 10) == 3);
    // Canary: check, then use once spends one and heals.
    run(&mut bar, &mut inventory, &check(10), &ALIVE, 2).unwrap();
    let reply = run(&mut bar, &mut inventory, &use_item(10), &ALIVE, 2).unwrap();
    assert_eq!((reply.heal_percent, count(&inventory, 10)), (35, 2));
    // A replayed 42/15 needs a new check.
    let reply = run(&mut bar, &mut inventory, &use_item(10), &ALIVE, 20_000).unwrap();
    assert!(is_chat(&reply) && reply.heal_percent == 0 && count(&inventory, 10) == 2);
}

#[test]
fn the_check_refuses_with_a_chat_line_not_the_chest_dialog() {
    let fixture = Fixture::new();
    let (mut inventory, mut bar) = (fixture.inventory(), fixture.bar());
    for (why, id, player) in [
        ("no effect yet", 12, &ALIVE),
        ("exhausted", 15, &ALIVE),
        ("knocked out", 10, &KNOCKED_OUT),
    ] {
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
fn the_effect_table_is_tunable_from_data_within_sanity_bounds() {
    let fixture = Fixture::new();
    let write = |value: serde_json::Value| {
        fs::write(fixture.path.join("consumables.json"), value.to_string()).unwrap();
        ItemBar::load(&fixture.path, Default::default(), None)
    };
    let row = |definition, heal, cooldown| json!({"definition":definition,"heal_percent":heal,"cooldown_ms":cooldown});
    // Absent file: the built-in Small Health Potion rows.
    assert_eq!(
        Effects::load(&fixture.path).unwrap().get(SMALL),
        Some(Effect {
            definition: SMALL,
            heal_percent: 35,
            cooldown_ms: 10_000
        })
    );
    assert_eq!(Effects::load(&fixture.path).unwrap().get(MEDIUM), None);
    // A tuned file takes effect: Medium becomes usable, Small heals 50% with no recast.
    let mut bar =
        write(json!({"schema_version":1,"consumables":[row(SMALL, 50, 0), row(MEDIUM, 20, 1000)]}))
            .unwrap();
    let mut inventory = fixture.inventory();
    run(&mut bar, &mut inventory, &assign(0, 10), &ALIVE, 0).unwrap();
    run(&mut bar, &mut inventory, &assign(1, 12), &ALIVE, 0).unwrap();
    assert_eq!(
        run(&mut bar, &mut inventory, &cast(0), &ALIVE, 0)
            .unwrap()
            .heal_percent,
        50
    );
    assert_eq!(
        run(&mut bar, &mut inventory, &cast(0), &ALIVE, 0)
            .unwrap()
            .heal_percent,
        50
    );
    assert_eq!(
        run(&mut bar, &mut inventory, &cast(1), &ALIVE, 0)
            .unwrap()
            .heal_percent,
        20
    );
    assert!(is_ability_failure(
        &run(&mut bar, &mut inventory, &cast(1), &ALIVE, 999).unwrap()
    ));
    // Out-of-bounds or unrecovered rows are refused when the content loads.
    for (why, value) in [
        (
            "heal 0",
            json!({"schema_version":1,"consumables":[row(SMALL, 0, 0)]}),
        ),
        (
            "heal over 100",
            json!({"schema_version":1,"consumables":[row(SMALL, 101, 0)]}),
        ),
        (
            "recast over ten minutes",
            json!({"schema_version":1,"consumables":[row(SMALL, 10, 600_001)]}),
        ),
        (
            "not a recovered consumable",
            json!({"schema_version":1,"consumables":[row(HAT, 10, 0)]}),
        ),
        (
            "duplicate",
            json!({"schema_version":1,"consumables":[row(SMALL, 10, 0), row(SMALL, 20, 0)]}),
        ),
        ("schema", json!({"schema_version":2,"consumables":[]})),
        (
            "unknown field",
            json!({"schema_version":1,"consumables":[],"extra":1}),
        ),
    ] {
        assert!(write(value).is_err(), "{why}");
    }
    assert!(write(json!({"schema_version":1,"consumables":[row(SMALL, 100, 600_000)]})).is_ok());
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
