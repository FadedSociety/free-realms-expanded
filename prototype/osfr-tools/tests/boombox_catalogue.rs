//! T459: the boombox catalogue. The shipped `boomboxes.json` is gated against the client's own
//! tables, the loader keeps rows inside their bounds, and through the real `Gameplay` route an
//! owned box binds to the item bar and a press spends nothing (T461 places the box; see its
//! gateway tests for the world side).
#[path = "support/models.rs"]
mod models;

use osfr_tools::{
    boombox::Catalogue,
    character_store::CharacterState,
    gameplay::{Gameplay, Outcome},
    inventory::{Inventory, InventoryItem, InventoryState, RentalState},
    item_bar::{ItemBar, Player, use_wire},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

const TOTEM: i32 = 8185; // owned outright in the tests below
const REALMS_ROLL: i32 = 1180; // a rental in the captured inventory
const CLOCKWORK: i32 = 35759;
const CAMPFIRE: i32 = 9114; // Class 171 in the captured definitions, but not a boombox
const SANCTUARY_ONLY: [i32; 6] = [900059, 900060, 900061, 900063, 76877, 78075];

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn content() -> PathBuf {
    crate_root().join("data/content")
}
fn lab(path: &str) -> PathBuf {
    crate_root().join("../../../lab").join(path)
}
/// The captured opening content: its inventory defines the captured boombox rows.
fn reference() -> PathBuf {
    lab("references/rust-opening-cooking-original-assets")
}
fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}
fn rows() -> Vec<Value> {
    let file: Value = serde_json::from_str(&read(&content().join("boomboxes.json"))).unwrap();
    file["boomboxes"].as_array().unwrap().clone()
}
fn int(value: &Value) -> i32 {
    i32::try_from(value.as_i64().unwrap()).unwrap()
}
fn ints(value: &Value) -> Vec<i32> {
    value.as_array().unwrap().iter().map(int).collect()
}
/// Every `id="N"` of a `<tag ...>` element in a client resource file.
fn xml_ids(file: &str, tag: &str) -> HashSet<i32> {
    let text = read(&lab("working/OSFR.Client/OSFR Client/Resources").join(file));
    let open = format!("<{tag} ");
    text.split(&open)
        .skip(1)
        .filter_map(|element| {
            let element = &element[..element.find('>')?];
            let at = element.find(" id=\"")? + 5;
            element[at..].split('"').next()?.parse().ok()
        })
        .collect()
}
fn temp_dir() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "osfr-boombox-{}-{}",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn every_model_is_a_client_model() {
    let models = models::read_models();
    for row in rows() {
        let mut ids = vec![int(&row["model_id"])];
        ids.extend(
            [int(&row["transform_model_id"])]
                .into_iter()
                .filter(|id| *id != 0),
        );
        // clip_ms is keyed by the dancer's player model.
        ids.extend(
            row["clip_ms"]
                .as_object()
                .unwrap()
                .keys()
                .map(|k| k.parse::<i32>().unwrap()),
        );
        for id in ids {
            assert!(
                models.contains_key(&i64::from(id)),
                "{}: model {id}",
                row["item_id"]
            );
        }
    }
    assert!(models[&1941].contains("human_m_realmsroll.adr"));
    assert!(models[&1973].contains("mkt_boombox_realmsroll_01.adr"));
}

#[test]
fn every_dance_is_a_client_animation_slot_or_group() {
    let slots = xml_ids("AnimationTypes.xml", "AnimationSlot");
    let groups = xml_ids("AnimationGroups.xml", "AnimationGroup");
    assert!(
        slots.contains(&3531) && groups.contains(&3200),
        "the parser reads both tables"
    );
    for row in rows() {
        let standing = int(&row["standing_animation_id"]);
        let dances = ints(&row["dance_sequence"]);
        for id in dances
            .into_iter()
            .chain([standing].into_iter().filter(|id| *id != 0))
        {
            assert!(
                slots.contains(&id) || groups.contains(&id),
                "{}: {id}",
                row["item_id"]
            );
        }
    }
}

#[test]
fn every_song_is_a_client_effect_definition() {
    let effects = xml_ids("ActorCompositeEffectDefinitions.xml", "EffectDefinition");
    let mut songs = 0;
    for row in rows() {
        for id in ints(&row["song_effect_ids"]) {
            assert!(effects.contains(&id), "{}: effect {id}", row["item_id"]);
            songs += 1;
        }
    }
    assert_eq!(songs, 6, "PFX_music_boombox_red.._yellow (16448-16453)");
}

/// One row of the reference item definitions (W84's item catalogue source).
#[derive(Deserialize)]
struct ItemRow {
    #[serde(rename = "Id")]
    id: i32,
    #[serde(rename = "Class", default)]
    class: i32,
    #[serde(rename = "ModelName", default)]
    model: String,
    #[serde(rename = "TextureAlias", default)]
    texture: String,
    #[serde(rename = "TintAlias", default)]
    tint: String,
}

#[test]
fn the_catalogue_is_exactly_the_client_boombox_items_with_their_model_and_aliases() {
    let path = lab("references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json");
    let items: Vec<ItemRow> = serde_json::from_str(read(&path).trim_start_matches('\u{feff}'))
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let mut boxes: Vec<_> = items
        .iter()
        .filter(|i| i.class == 171)
        .map(|i| i.id)
        .collect();
    boxes.sort_unstable();
    let rows = rows();
    let mut listed: Vec<_> = rows.iter().map(|row| int(&row["item_id"])).collect();
    listed.sort_unstable();
    assert_eq!(
        listed, boxes,
        "every Class 171 client item, and nothing else"
    );
    assert!(SANCTUARY_ONLY.iter().all(|id| !listed.contains(id)));
    let models = models::read_models();
    for row in &rows {
        let item = items.iter().find(|i| i.id == int(&row["item_id"])).unwrap();
        // Client evidence wins: the model is the client item's own model file, where it has one.
        if !item.model.is_empty() {
            let shown = &models[&row["model_id"].as_i64().unwrap()];
            assert!(
                shown.contains(&item.model.to_ascii_lowercase()),
                "{}",
                item.id
            );
        }
        assert_eq!(row["texture_alias"], item.texture.as_str(), "{}", item.id);
        assert_eq!(row["tint_alias"], item.tint.as_str(), "{}", item.id);
    }
}

/// The shipped file with one edit, loaded from a scratch content directory.
fn load_with(edit: fn(&mut Value)) -> io::Result<Catalogue> {
    let mut file: Value = serde_json::from_str(&read(&content().join("boomboxes.json"))).unwrap();
    edit(&mut file);
    let dir = temp_dir();
    fs::write(
        dir.join("boomboxes.json"),
        serde_json::to_vec(&file).unwrap(),
    )
    .unwrap();
    let loaded = Catalogue::load(&dir);
    let _ = fs::remove_dir_all(&dir);
    loaded
}

#[test]
fn the_loader_refuses_rows_out_of_bounds_and_a_missing_file_is_empty() {
    // Canary: the shipped file loads as is, and so does an untouched copy.
    let shipped = Catalogue::load(&content()).unwrap();
    assert!(shipped.get(TOTEM).is_some() && shipped.get(CAMPFIRE).is_none());
    assert_eq!(load_with(|_| {}).unwrap(), shipped);
    let cases: [(&str, fn(&mut Value)); 18] = [
        ("range 0", |f| f["boomboxes"][0]["range_m"] = json!(0)),
        ("range 51", |f| f["boomboxes"][0]["range_m"] = json!(51)),
        ("spawn offset 11", |f| {
            f["boomboxes"][0]["spawn_offset_m"] = json!(11)
        }),
        ("recast 2999", |f| {
            f["boomboxes"][0]["recast_ms"] = json!(2999)
        }),
        ("duration 0", |f| {
            f["boomboxes"][0]["duration_ms"] = json!(0)
        }),
        ("duration 600001", |f| {
            f["boomboxes"][0]["duration_ms"] = json!(600_001)
        }),
        ("clip 0", |f| {
            f["boomboxes"][0]["clip_ms"]["1"][0] = json!(0)
        }),
        ("clip per step missing", |f| {
            f["boomboxes"][0]["clip_ms"]["1"] = json!([4834])
        }),
        ("item id 0", |f| f["boomboxes"][0]["item_id"] = json!(0)),
        ("model id 0", |f| f["boomboxes"][0]["model_id"] = json!(0)),
        ("dance id 0", |f| {
            f["boomboxes"][0]["dance_sequence"][0] = json!(0)
        }),
        ("song effect 0", |f| {
            f["boomboxes"][2]["song_effect_ids"][0] = json!(0)
        }),
        ("transform without a model", |f| {
            f["boomboxes"][1]["transform_model_id"] = json!(0)
        }),
        ("no provenance", |f| f["boomboxes"][0]["source"] = json!([])),
        ("unknown field", |f| f["boomboxes"][0]["price"] = json!(1)),
        ("duplicate item id", |f| {
            let copy = f["boomboxes"][0].clone();
            f["boomboxes"].as_array_mut().unwrap().push(copy);
        }),
        ("65 rows", |f| {
            let rows = f["boomboxes"].as_array_mut().unwrap();
            let template = rows[0].clone();
            for n in rows.len()..65 {
                let mut row = template.clone();
                row["item_id"] = json!(100_000 + n);
                rows.push(row);
            }
        }),
        ("over 256 KiB", |f| {
            f["boomboxes"][0]["source"][0]["evidence"] = json!("x".repeat(256 * 1024));
        }),
    ];
    for (why, edit) in cases {
        assert!(load_with(edit).is_err(), "{why}");
    }
    let empty = temp_dir();
    assert_eq!(Catalogue::load(&empty).unwrap(), Catalogue::default());
    let _ = fs::remove_dir_all(&empty);
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
fn assign(slot: i32, instance: i32) -> Vec<u8> {
    msg(42, 6, &[slot, instance])
}
/// 36/10 on the item bar, aimed at the player (guid 1).
fn cast(slot: i32) -> Vec<u8> {
    let mut bytes = msg(36, 10, &[2, slot, 0]);
    bytes.extend(1u64.to_le_bytes());
    bytes
}
fn owned(id: i32, definition_id: i32, consumed_count: u32, rented: bool) -> InventoryItem {
    InventoryItem {
        definition_id,
        tint: 0,
        id,
        count: 1,
        consumed_count,
        stored_count: 0,
        last_cast_time: i64::from(i32::MAX),
        rental: rented.then_some(RentalState {
            expiration: 0,
            upsells: 0,
            bundle_id: 3761,
        }),
        ability_count: 0,
        activate_enabled: false,
    }
}
/// The captured opening inventory plus `items` as (definition, consumed, rented), with fresh ids.
fn inventory_with(items: &[(i32, u32, bool)]) -> (InventoryState, Vec<i32>) {
    let mut state = Inventory::load(&reference(), None).unwrap().state().clone();
    let first = state.items.iter().map(|i| i.id).max().unwrap_or(0) + 1000;
    let ids: Vec<i32> = (first..).take(items.len()).collect();
    for (&(definition, consumed, rented), &id) in items.iter().zip(&ids) {
        state.items.push(owned(id, definition, consumed, rented));
    }
    (state, ids)
}
fn slots(replies: &[Vec<u8>]) -> Vec<use_wire::ActionBarSlotUpdate> {
    replies
        .iter()
        .filter(|r| r.starts_with(&[38, 0, 25, 0]))
        .filter_map(|r| use_wire::decode_slot_update(r))
        .filter(|s| s.bar == 2)
        .collect()
}
fn failed(out: &Outcome) -> bool {
    out.replies.len() == 1 && out.replies[0].starts_with(&[36, 0, 1, 0])
}

#[test]
fn an_owned_boombox_binds_and_a_press_spends_nothing() {
    let (state, ids) = inventory_with(&[
        (TOTEM, 0, false),
        (REALMS_ROLL, 0, true),
        (CLOCKWORK, 1, false),
        (CAMPFIRE, 0, false),
    ]);
    let mut saved = Gameplay::load(&reference(), None).unwrap().state();
    saved.inventory = state;
    let mut game = Gameplay::load(&reference(), Some(saved)).unwrap();
    game.initialize_world(&who(), 0).unwrap();

    // A rented box, a used-up box and a Class 171 item that is no boombox are refused, and the
    // bar stays byte-identical.
    for (why, id) in [
        ("rental", ids[1]),
        ("none left", ids[2]),
        ("not a boombox", ids[3]),
    ] {
        let before = game.state().item_bar;
        let out = game.handle_in_world(&assign(0, id), &who(), 500).unwrap();
        assert!(!out.changed, "{why}");
        assert!(
            out.replies[0].starts_with(&[15, 0, 3, 0]),
            "{why}: chat line first"
        );
        assert!(slots(&out.replies).iter().all(|s| s.is_empty), "{why}");
        let after = game.state().item_bar;
        assert_eq!(after, before, "{why}");
        assert_eq!(
            serde_json::to_vec(&after).unwrap(),
            serde_json::to_vec(&before).unwrap()
        );
    }

    // The owned box binds, shown enabled with its count and no recast yet.
    let bound = game
        .handle_in_world(&assign(0, ids[0]), &who(), 1_000)
        .unwrap();
    assert!(bound.changed && bound.boombox.is_none());
    let slot = slots(&bound.replies)[0];
    assert!(!slot.is_empty && slot.server_enabled);
    assert_eq!(slot.quantity, 1);

    // A press hands the definition to the host and spends nothing: the inventory and the saved
    // bar are byte-identical, the box is still counted, and the slot shows the row's recast.
    let before = game.state();
    let used = game.handle_in_world(&cast(0), &who(), 2_000).unwrap();
    assert_eq!(used.boombox, Some(TOTEM));
    assert!(!used.changed && !failed(&used));
    let slot = slots(&used.replies)[0];
    assert_eq!((slot.quantity, slot.server_enabled), (1, true));
    assert_eq!(slot.total_refresh_time, 180_000, "the row's recast_ms");
    let after = game.state();
    assert_eq!(after.inventory, before.inventory);
    // The recast lives in memory only: the saved bar is byte-identical.
    assert_eq!(
        serde_json::to_vec(&after.item_bar).unwrap(),
        serde_json::to_vec(&before.item_bar).unwrap()
    );
    // The recast is session-only: a save and reload shows the slot ready, and a press works.
    let saved = serde_json::from_slice(&serde_json::to_vec(&after).unwrap()).unwrap();
    let mut game = Gameplay::load(&reference(), Some(saved)).unwrap();
    let login = game.initialize_world(&who(), 0).unwrap();
    let slot = slots(&login.replies)[0];
    assert!(slot.slot == 0 && !slot.is_empty && slot.server_enabled);
    assert_eq!(slot.total_refresh_time, 180_000);
    let again = game.handle_in_world(&cast(0), &who(), 2_100).unwrap();
    assert_eq!(again.boombox, Some(TOTEM));
    // 42/16 on a box stays outside the item bar: no confirm dialog.
    let check = game
        .handle_in_world(&msg(42, 16, &[ids[0]]), &who(), 3_000)
        .unwrap();
    assert!(!check.replies.iter().any(|r| r.starts_with(&[38, 0, 30, 0])));
}

/// The real item bar with the real inventory, against `boomboxes` as the content.
fn bar_and_inventory(boomboxes: Catalogue, state: InventoryState) -> (ItemBar, Inventory) {
    let bar = ItemBar::load(&reference(), Arc::new(boomboxes), None).unwrap();
    (bar, Inventory::load(&reference(), Some(state)).unwrap())
}
const ME: Player = Player {
    guid: 1,
    alive: true,
    can_place_box: true,
};

#[test]
fn without_boomboxes_json_no_class_171_item_can_bind_or_be_used() {
    // Every Class 171 definition the captured content loads, plus the catalogued boxes it defines.
    let probe = Inventory::load(&reference(), None).unwrap();
    let shipped = Catalogue::load(&content()).unwrap();
    let candidates: Vec<i32> = probe
        .definition_ids()
        .filter(|id| {
            let class = probe.definition_record(*id).and_then(|row| row.get(32..36));
            class == Some(&171i32.to_le_bytes()[..]) || shipped.get(*id).is_some()
        })
        .collect();
    assert!(candidates.len() >= 7, "{candidates:?}");
    let items: Vec<_> = candidates.iter().map(|id| (*id, 0, false)).collect();
    let (state, ids) = inventory_with(&items);

    let (mut bar, mut inventory) = bar_and_inventory(Catalogue::default(), state.clone());
    for id in &ids {
        let out = bar
            .handle(&assign(0, *id), &mut inventory, &ME, 0)
            .unwrap()
            .unwrap();
        assert!(!out.changed && bar.is_empty(), "{id}");
        let used = bar
            .handle(&cast(0), &mut inventory, &ME, 0)
            .unwrap()
            .unwrap();
        assert!(used.replies[0].starts_with(&[36, 0, 1, 0]) && !used.changed);
    }
    assert_eq!(inventory.state(), &state);

    // Canary: with the shipped catalogue the catalogued boxes bind.
    let (mut bar, mut inventory) = bar_and_inventory(shipped.clone(), state);
    for (id, definition) in ids.iter().zip(&candidates) {
        let out = bar
            .handle(&assign(0, *id), &mut inventory, &ME, 0)
            .unwrap()
            .unwrap();
        assert_eq!(
            out.changed,
            shipped.get(*definition).is_some(),
            "{definition}"
        );
    }
}

#[test]
fn a_saved_binding_whose_row_left_the_content_is_dropped_at_load() {
    let (state, ids) = inventory_with(&[(TOTEM, 0, false)]);
    let (mut bar, mut inventory) = bar_and_inventory(Catalogue::load(&content()).unwrap(), state);
    assert!(
        bar.handle(&assign(2, ids[0]), &mut inventory, &ME, 0)
            .unwrap()
            .unwrap()
            .changed
    );
    let saved = serde_json::from_slice(&serde_json::to_vec(&bar).unwrap()).unwrap();

    let mut reloaded = ItemBar::load(&reference(), Arc::default(), Some(saved)).unwrap();
    assert!(!reloaded.is_empty(), "the binding is restored as saved");
    let updates = slots(&reloaded.refresh(&inventory, 0).unwrap());
    assert_eq!(updates.len(), 4);
    assert!(updates.iter().all(|s| s.is_empty) && reloaded.is_empty());
}

#[test]
fn a_box_press_runs_the_item_checks_and_only_a_ready_valid_press_stamps_the_recast() {
    let (state, ids) = inventory_with(&[(TOTEM, 0, false), (REALMS_ROLL, 0, true)]);
    let (mut bar, mut inventory) = bar_and_inventory(Catalogue::load(&content()).unwrap(), state);
    bar.handle(&assign(0, ids[0]), &mut inventory, &ME, 0)
        .unwrap();
    let recast = Catalogue::load(&content())
        .unwrap()
        .get(TOTEM)
        .unwrap()
        .recast_ms;
    let press = |bar: &mut ItemBar, inventory: &mut Inventory, player: &Player, now: u64| {
        bar.handle(&cast(0), inventory, player, now)
            .unwrap()
            .unwrap()
    };
    // Each refusal is one 36/1 and stamps nothing: the press right after still works.
    let away = Player {
        can_place_box: false,
        ..ME
    };
    let knocked_out = Player { alive: false, ..ME };
    for (why, player) in [("away or busy", away), ("knocked out", knocked_out)] {
        let before = inventory.state().clone();
        let out = press(&mut bar, &mut inventory, &player, 1_000);
        assert!(failed_reply(&out) && out.boombox.is_none(), "{why}");
        assert_eq!(inventory.state(), &before, "{why}");
    }
    let out = press(&mut bar, &mut inventory, &ME, 1_000);
    assert_eq!(out.boombox, Some(TOTEM));
    assert!(!out.changed, "nothing is saved");
    // Inside the recast: refused. Exactly at it: ready again.
    let inside = press(&mut bar, &mut inventory, &ME, 1_000 + u64::from(recast) - 1);
    assert!(failed_reply(&inside) && inside.boombox.is_none());
    let ready = press(&mut bar, &mut inventory, &ME, 1_000 + u64::from(recast));
    assert_eq!(ready.boombox, Some(TOTEM));
    // A rented box cannot be bound, so a rental never reaches a press.
    let out = bar
        .handle(&assign(1, ids[1]), &mut inventory, &ME, 0)
        .unwrap()
        .unwrap();
    assert!(!out.changed && out.boombox.is_none());
}

fn failed_reply(reply: &osfr_tools::item_bar::Reply) -> bool {
    reply.replies.len() == 1 && reply.replies[0].starts_with(&[36, 0, 1, 0])
}
