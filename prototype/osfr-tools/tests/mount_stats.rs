//! T451: a boarded ride sends its rider the speed, glide and jump stats (38/7) and every ride end
//! restores the on-foot values. Driven through `Gameplay::handle_in_world` / `settle_mounts` (and
//! the gateway's movement check for the mounted allowance) like `tests/mount_wired.rs`.
//!
//! RECOVERED (client): the 38/7 reader (`0091e1a0`, `0090a540`, `008fc4c0`, `008e7640`: u16 38,
//! u16 7, u64 target, i32 count, then i32 id, i32 kind (0 int, 1 float) and the value) and the
//! stat ids (name table `0x01af2008`: 2 MaxMovementSpeed, 59-63 Glide*, 64 GlideEnabled,
//! 67 GlideAccel, 68 JumpHeight). RECONSTRUCTED (R): the ride stat values (community data) and
//! the three-stat restore.
use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    chat,
    gameplay::{Gameplay, Outcome},
    gateway::Gateway,
    inventory::{InventoryItem, InventoryState},
    mounts::{MountCatalog, MountService, RideContext, RidePose, RideRestrictions, RideStats},
    movement::SpeedModifiers,
};
use serde_json::Value;
use std::{io, path::PathBuf};

const WOLF: i32 = 993;
const WOLF_ITEM: i32 = 14289;
const FIRE_HAWK: i32 = 13459;
const FIRE_HAWK_ITEM: i32 = 78397;
const POGO_STICK: i32 = 13567;
const POGO_STICK_ITEM: i32 = 79438;
const LAWN_CHAIR: i32 = 13460;
const LAWN_CHAIR_ITEM: i32 = 79410;
const NOW: u64 = 10_000;
const OWNER: u64 = 1;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Stat {
    Int(i32),
    Float(f32),
}

type Update = (u64, Vec<(i32, Stat)>);

/// Reader-faithful 38/7 parse: u16 38, u16 7, u64 target GUID, i32 count, then per entry i32 id,
/// i32 kind (0 int, 1 float; anything else is a client read error) and the value. No bytes may
/// be left over.
fn stat_update(packet: &[u8]) -> Option<Update> {
    if packet.get(..4)? != [38, 0, 7, 0] {
        return None;
    }
    let word = |at: usize| -> [u8; 4] { packet[at..at + 4].try_into().unwrap() };
    let target = u64::from_le_bytes(packet[4..12].try_into().unwrap());
    let count = i32::from_le_bytes(word(12));
    let mut stats = Vec::new();
    let mut at = 16;
    for _ in 0..count {
        let value = match i32::from_le_bytes(word(at + 4)) {
            0 => Stat::Int(i32::from_le_bytes(word(at + 8))),
            1 => Stat::Float(f32::from_le_bytes(word(at + 8))),
            kind => panic!("the client rejects stat kind {kind}"),
        };
        stats.push((i32::from_le_bytes(word(at)), value));
        at += 12;
    }
    assert_eq!(at, packet.len(), "38/7 has trailing bytes");
    Some((target, stats))
}

fn updates(packets: &[Vec<u8>]) -> Vec<Update> {
    packets.iter().filter_map(|p| stat_update(p)).collect()
}

/// The owner's 38/7 updates in one outcome; observers (`nearby`) never get one.
fn owner_updates(out: &Outcome) -> Vec<Update> {
    assert!(updates(&out.nearby).is_empty(), "a 38/7 reached observers");
    updates(&out.replies)
}

/// The boarded block in the order sent: speed, the glide block (59-63), GlideEnabled (int),
/// GlideAccel, JumpHeight.
fn ride_block(values: [f32; 9]) -> Vec<(i32, Stat)> {
    [2, 59, 60, 61, 62, 63, 64, 67, 68]
        .into_iter()
        .zip(values)
        .map(|(id, value)| match id {
            64 => (id, Stat::Int(value as i32)),
            _ => (id, Stat::Float(value)),
        })
        .collect()
}

fn on_foot() -> Vec<(i32, Stat)> {
    vec![
        (2, Stat::Float(8.0)),
        (64, Stat::Int(0)),
        (68, Stat::Float(0.0)),
    ]
}

const WOLF_BLOCK: [f32; 9] = [12.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];

// ------------------------------------------------------------------ gameplay fixture

fn lab() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab")
}
fn who() -> CharacterState {
    CharacterState {
        guid: OWNER,
        zone_id: 0,
        position: [-440.0, 0.8, 1292.0],
        rotation: [1.0, 0.0, 0.0],
    }
}
fn ride_guid() -> u64 {
    MountService::ride_guid(OWNER)
}
fn spawn(mount: i32) -> Vec<u8> {
    [vec![168, 0, 6], mount.to_le_bytes().to_vec()].concat()
}
fn board(actor: u64) -> Vec<u8> {
    [
        vec![168, 0, 1],
        actor.to_le_bytes().to_vec(),
        vec![1, 0, 0, 0, 0, 0],
    ]
    .concat()
}
const PUT_AWAY: [u8; 3] = [168, 0, 3];

fn item(definition_id: i32, id: i32) -> InventoryItem {
    InventoryItem {
        definition_id,
        tint: 0,
        id,
        count: 1,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: i64::from(i32::MAX),
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    }
}

/// A game that owns `items`, with world entry done (as `tests/mount_wired.rs::game_with`).
fn game_with(items: &[i32]) -> Gameplay {
    let reference = lab().join("references/rust-opening-cooking-original-assets");
    let mut state = Gameplay::load(&reference, None).unwrap().state();
    let next = state
        .inventory
        .items
        .iter()
        .map(|i| i.id)
        .max()
        .unwrap_or(0)
        + 1000;
    for (n, definition_id) in items.iter().enumerate() {
        state
            .inventory
            .items
            .push(item(*definition_id, next + n as i32));
    }
    let mut game = Gameplay::load(&reference, Some(state)).unwrap();
    game.initialize_world(&who(), 0).unwrap();
    game
}

/// Summon and board `mount`; returns the boarding outcome.
fn ride(game: &mut Gameplay, mount: i32, now: u64) -> Outcome {
    game.handle_in_world(&spawn(mount), &who(), now).unwrap();
    let boarded = game
        .handle_in_world(&board(ride_guid()), &who(), now + 10)
        .unwrap();
    assert!(game.is_mounted());
    boarded
}

fn mounted(items: &[i32], mount: i32) -> Gameplay {
    let mut game = game_with(items);
    ride(&mut game, mount, NOW);
    game
}

#[test]
fn boarding_sends_the_ride_stat_block_to_the_owner_only() {
    let mut game = game_with(&[WOLF_ITEM]);
    let summoned = game.handle_in_world(&spawn(WOLF), &who(), NOW).unwrap();
    assert!(
        owner_updates(&summoned).is_empty(),
        "summoning sends no stats"
    );

    // A refused board (another actor) sends no stats and changes nothing.
    let before = game.ride_view(OWNER);
    let refused = game
        .handle_in_world(&board(ride_guid() + 1), &who(), NOW + 5)
        .unwrap();
    assert!(
        refused
            .replies
            .iter()
            .any(|r| r.starts_with(&[15, 0, 3, 0]))
    );
    assert!(owner_updates(&refused).is_empty());
    assert!(!game.is_mounted());
    assert_eq!(game.ride_view(OWNER), before);

    let boarded = game
        .handle_in_world(&board(ride_guid()), &who(), NOW + 10)
        .unwrap();
    assert_eq!(
        owner_updates(&boarded),
        vec![(OWNER, ride_block(WOLF_BLOCK))]
    );
    // Boarding again is refused (already seated) and sends nothing more.
    let again = game
        .handle_in_world(&board(ride_guid()), &who(), NOW + 20)
        .unwrap();
    assert!(owner_updates(&again).is_empty());
}

#[test]
fn the_fire_hawk_glides_and_the_pogo_stick_jumps() {
    let mut game = game_with(&[FIRE_HAWK_ITEM]);
    assert_eq!(
        owner_updates(&ride(&mut game, FIRE_HAWK, NOW)),
        vec![(
            OWNER,
            ride_block([12.5, 12.0, 6.0, 22.0, 0.75, 3.0, 1.0, 4.0, 5.0])
        )]
    );
    // The gameplay fixture's captured item catalogue has no Pogo Stick item, so this one boards
    // through `MountService::handle` (the same `wire` the gameplay route uses).
    let mut service = MountService::new(MountCatalog::load(mounts_json()).unwrap());
    assert_eq!(
        board_with(&mut service, POGO_STICK, POGO_STICK_ITEM),
        vec![(
            OWNER,
            ride_block([12.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 2.5])
        )]
    );
}

#[test]
fn every_ride_end_restores_the_on_foot_stats() {
    let restored = vec![(OWNER, on_foot())];

    // 168/3; a repeat after the ride ended sends nothing.
    let mut game = mounted(&[WOLF_ITEM], WOLF);
    let away = game.handle_in_world(&PUT_AWAY, &who(), NOW + 100).unwrap();
    assert_eq!(owner_updates(&away), restored, "put away");
    let again = game.handle_in_world(&PUT_AWAY, &who(), NOW + 200).unwrap();
    assert!(owner_updates(&again).is_empty());

    // Zoning; a second settle sends nothing.
    let mut game = mounted(&[WOLF_ITEM], WOLF);
    assert_eq!(owner_updates(&game.settle_mounts(&who(), true)), restored);
    assert!(owner_updates(&game.settle_mounts(&who(), true)).is_empty());

    // Knockout.
    let mut game = mounted(&[WOLF_ITEM], WOLF);
    game.receive_field_damage(u32::MAX, NOW + 100, false)
        .unwrap();
    assert_eq!(
        owner_updates(&game.settle_mounts(&who(), false)),
        restored,
        "knockout"
    );

    // An instance battle starts (the minigame gate of the ride restrictions).
    let mut game = mounted(&[WOLF_ITEM], WOLF);
    game.set_dev_tools(true);
    game.set_role(AccountRole::Admin);
    game.handle_in_world(&chat::local_request("realm battle 411"), &who(), NOW + 100)
        .unwrap();
    assert_eq!(
        owner_updates(&game.settle_mounts(&who(), false)),
        restored,
        "battle"
    );
    assert!(!game.is_mounted());

    // The ride item traded away (T424 puts the ride away in `settle_mounts`).
    let mut game = mounted(&[WOLF_ITEM], WOLF);
    let mut after = game.inventory_state().clone();
    after.items.retain(|row| row.definition_id != WOLF_ITEM);
    game.adopt_trade_inventory(after).unwrap();
    assert_eq!(
        owner_updates(&game.settle_mounts(&who(), false)),
        restored,
        "item lost"
    );

    // A ride that was summoned but never boarded restores nothing, by 168/3 or by zoning.
    let mut game = game_with(&[WOLF_ITEM]);
    game.handle_in_world(&spawn(WOLF), &who(), NOW).unwrap();
    let away = game.handle_in_world(&PUT_AWAY, &who(), NOW + 100).unwrap();
    assert!(away.replies.iter().any(|r| r.starts_with(&[35, 0, 3, 0])));
    assert!(owner_updates(&away).is_empty());
    game.handle_in_world(&spawn(WOLF), &who(), NOW + 5_000)
        .unwrap();
    assert!(owner_updates(&game.settle_mounts(&who(), true)).is_empty());
}

/// PR #106 canary: a summon while seated replaces the ride; the old ride's glide must not stick.
#[test]
fn summoning_while_mounted_restores_then_replaces() {
    let mut game = mounted(&[FIRE_HAWK_ITEM, WOLF_ITEM], FIRE_HAWK);
    let replaced = game
        .handle_in_world(&spawn(WOLF), &who(), NOW + 2_000)
        .unwrap();
    let removal = [35, 0, 3, 0, 1, 0];
    assert!(replaced.replies.iter().any(|r| r.starts_with(&[168, 0, 4])));
    assert!(
        replaced
            .replies
            .iter()
            .any(|r| r.starts_with(&removal) && r[6..14] == ride_guid().to_le_bytes())
    );
    assert!(
        replaced
            .replies
            .iter()
            .any(|r| r.starts_with(&[35, 0, 2, 0]) && r[4..12] == ride_guid().to_le_bytes())
    );
    assert_eq!(owner_updates(&replaced), vec![(OWNER, on_foot())]);
    assert!(!game.is_mounted());

    let boarded = game
        .handle_in_world(&board(ride_guid()), &who(), NOW + 2_010)
        .unwrap();
    assert_eq!(
        owner_updates(&boarded),
        vec![(OWNER, ride_block(WOLF_BLOCK))]
    );
}

#[test]
fn on_foot_speed_matches_the_captured_snapshot() {
    let capture: Value = serde_json::from_slice(
        &std::fs::read(lab().join("content/startup-snapshot/startup.json")).unwrap(),
    )
    .unwrap();
    let speed = capture["character"]["stats"]
        .as_array()
        .unwrap()
        .iter()
        .find(|stat| stat["id"] == 2)
        .unwrap();
    assert_eq!(speed["kind"], 1, "captured as a float");
    let captured = f32::from_bits(speed["value_bits"].as_u64().unwrap() as u32);

    let mut game = mounted(&[WOLF_ITEM], WOLF);
    let away = game.handle_in_world(&PUT_AWAY, &who(), NOW + 100).unwrap();
    assert_eq!(
        owner_updates(&away)[0].1[0],
        (2, Stat::Float(captured)),
        "the restore sends the captured on-foot speed"
    );
}

// ------------------------------------------------------------------ catalogue level

fn mounts_json() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/mounts.json")
}

fn bundled() -> Value {
    serde_json::from_slice(&std::fs::read(mounts_json()).unwrap()).unwrap()
}

/// The bundled document with `edit` applied, loaded through `MountCatalog::load`.
fn load_edited(name: &str, edit: impl FnOnce(&mut Value)) -> io::Result<MountCatalog> {
    let mut document = bundled();
    edit(&mut document);
    let dir = std::env::temp_dir().join(format!("osfr-t451-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("mounts.json");
    std::fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
    let loaded = MountCatalog::load(&path);
    std::fs::remove_dir_all(&dir).unwrap();
    loaded
}

fn wolf_row(document: &mut Value) -> &mut Value {
    document["mounts"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["id"] == WOLF)
        .unwrap()
}

fn inventory(definitions: &[i32]) -> InventoryState {
    InventoryState {
        coins: 0,
        items: definitions
            .iter()
            .enumerate()
            .map(|(n, definition_id)| item(*definition_id, n as i32 + 1))
            .collect(),
        equipment: Vec::new(),
        next_instance_id: None,
    }
}

fn request(
    service: &mut MountService,
    inner: &[u8],
    owned: &InventoryState,
    now: u64,
) -> Vec<Update> {
    let context = RideContext {
        guid: OWNER,
        inventory: owned,
        restrictions: RideRestrictions {
            zone_allows_riding: true,
            in_minigame: false,
            ability_active: false,
            knocked_out: false,
        },
        pose: RidePose {
            position: [0.0; 3],
            heading: 0.0,
        },
        now_ms: now,
    };
    let wire = service.handle(inner, &context).unwrap().unwrap();
    assert!(updates(&wire.shared).is_empty(), "a 38/7 reached observers");
    updates(&wire.private)
}

/// Summon and board `mount` through `MountService::handle`; the boarding's 38/7 updates.
fn board_with(service: &mut MountService, mount: i32, item_id: i32) -> Vec<Update> {
    let owned = inventory(&[item_id]);
    request(service, &spawn(mount), &owned, NOW);
    request(service, &board(ride_guid()), &owned, NOW + 10)
}

#[test]
fn an_upgradable_ride_glides_only_when_trained() {
    for (trained, glide) in [
        (false, WOLF_BLOCK),
        (true, [12.5, 8.0, 2.0, 18.0, 0.75, 4.0, 1.0, 4.0, 0.0]),
    ] {
        let catalog = load_edited("upgradable", |document| {
            let row = wolf_row(document);
            row["is_upgradable"] = true.into();
            row["is_upgraded"] = trained.into();
        })
        .unwrap();
        let mut service = MountService::new(catalog);
        assert_eq!(
            board_with(&mut service, WOLF, WOLF_ITEM),
            vec![(OWNER, ride_block(glide))],
            "trained: {trained}"
        );
    }
}

#[test]
fn mount_stats_bounds_are_enforced() {
    assert!(load_edited("bundled", |_| {}).is_ok());
    let stats = |field: &str, value: Value| {
        let field = field.to_owned();
        move |document: &mut Value| {
            wolf_row(document)["stats"] = [(field, value)].into_iter().collect()
        }
    };
    let refused: Vec<(&str, Box<dyn FnOnce(&mut Value)>)> = vec![
        (
            "negative-jump",
            Box::new(stats("jump_height", (-1.0).into())),
        ),
        ("high-jump", Box::new(stats("jump_height", 33.0.into()))),
        (
            "fast-glide",
            Box::new(stats("glide_max_forward_speed", 65.0.into())),
        ),
        ("long-fall", Box::new(stats("glide_fall_time", 10.5.into()))),
        ("infinite", Box::new(stats("glide_accel", 1e39.into()))),
        ("unknown", Box::new(stats("glide_turbo", 1.0.into()))),
        ("nan-text", Box::new(stats("jump_height", f64::NAN.into()))),
        (
            "inverted-glide",
            Box::new(|document: &mut Value| {
                wolf_row(document)["stats"] = serde_json::json!({
                    "glide_enabled": true,
                    "glide_min_forward_speed": 9.0,
                    "glide_default_forward_speed": 8.0,
                    "glide_max_forward_speed": 18.0,
                })
            }),
        ),
        (
            "fast-ride",
            Box::new(|document: &mut Value| wolf_row(document)["max_movement_speed"] = 65.0.into()),
        ),
        (
            "still-on-foot",
            Box::new(|document: &mut Value| document["on_foot"]["max_movement_speed"] = 0.0.into()),
        ),
        (
            "trained-negative",
            Box::new(|document: &mut Value| {
                document["trained_glide"]["stats"]["glide_fall_speed"] = (-4.0).into()
            }),
        ),
        (
            "trained-unsourced",
            Box::new(|document: &mut Value| {
                document["trained_glide"]["source"] = Value::Array(vec![])
            }),
        ),
    ];
    for (name, edit) in refused {
        assert!(load_edited(name, edit).is_err(), "{name} loaded");
    }

    // NaN cannot be written in JSON, so the bounds are also checked where the values enter.
    let catalog = load_edited("source", |_| {}).unwrap();
    let types: Vec<_> = catalog.mount_types().cloned().collect();
    let wolf = catalog.mount(WOLF).unwrap().clone();
    let nan = RideStats {
        jump_height: f32::NAN,
        ..RideStats::default()
    };
    let with_stats = osfr_tools::mounts::MountDefinition {
        stats: nan,
        ..wolf.clone()
    };
    assert!(MountCatalog::new(types.clone(), vec![with_stats], 8.0, RideStats::default()).is_err());
    assert!(MountCatalog::new(types.clone(), vec![wolf.clone()], 8.0, nan).is_err());
    assert!(
        MountCatalog::new(
            types.clone(),
            vec![wolf.clone()],
            f32::NAN,
            RideStats::default()
        )
        .is_err()
    );
    assert!(MountCatalog::new(types, vec![wolf], 8.0, RideStats::default()).is_ok());
}

// ------------------------------------------------------------------ movement allowance

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}
fn login() -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    for (i, text) in ["t451-synthetic-ticket", "1.910.1.530630", ""]
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
fn position(guid: u64, xyz: [f32; 3]) -> Vec<u8> {
    let mut bytes = 125u16.to_le_bytes().to_vec();
    bytes.extend(guid.to_le_bytes());
    for value in xyz.into_iter().chain([0.0, 0.0, 1.0]) {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend([0, 0]);
    bytes
}

/// How far east the player ends up after asking to move 450 units in one second, on `mount`
/// (or on foot). On foot the soft envelope is 10 units/s x 40 = 400, so the report is clamped;
/// on a ride at 1.25x it is 500 and the report is accepted.
fn one_second_dash(mount: Option<(i32, i32)>) -> f32 {
    let content = lab().join("reports/round59-native-20261015T124443Z/content");
    let reference = lab().join("references/rust-gateway-reference");
    let mut character = Gateway::reference_state(&reference).unwrap();
    character.position = [0.0; 3];
    let mut state = Gameplay::load(&content, None).unwrap().state();
    if let Some((_, item_id)) = mount {
        state.inventory.items.push(item(item_id, 900_001));
    }
    let mut gateway = Gateway::load_gameplay(&reference, character, &content, Some(state)).unwrap();
    gateway.set_dev_tools(false);
    gateway.set_role(AccountRole::Player);
    gateway.handle_at(&login(), NOW).unwrap();
    gateway.handle_at(&wrap(&[13, 0]), NOW).unwrap();
    gateway.handle_at(&wrap(&[10, 0]), NOW).unwrap();
    let guid = gateway.state().guid;
    if let Some((mount_id, _)) = mount {
        gateway.handle_at(&wrap(&spawn(mount_id)), NOW + 1).unwrap();
        let actor = MountService::ride_guid(guid);
        gateway.handle_at(&wrap(&board(actor)), NOW + 2).unwrap();
        assert!(gateway.emote_context(NOW + 3).mounted);
    }
    gateway
        .handle_at(&wrap(&position(guid, [0.0; 3])), NOW + 10)
        .unwrap();
    assert_eq!(gateway.state().position, [0.0; 3]);
    gateway
        .handle_at(&wrap(&position(guid, [450.0, 0.0, 0.0])), NOW + 1_010)
        .unwrap();
    gateway.state().position[0]
}

#[test]
fn mounted_allowance_uses_the_ride_speed() {
    let multiplier =
        |game: &Gameplay| SpeedModifiers::riding(game.mounted_speed()).mount_multiplier;
    let mut game = game_with(&[WOLF_ITEM]);
    assert_eq!(multiplier(&game), 1.0, "on foot");
    game.handle_in_world(&spawn(WOLF), &who(), NOW).unwrap();
    assert_eq!(multiplier(&game), 1.0, "summoned, not boarded");
    game.handle_in_world(&board(ride_guid()), &who(), NOW + 10)
        .unwrap();
    assert_eq!(multiplier(&game), 1.25, "Wolf, 12.5 over the base 10");
    game.handle_in_world(&PUT_AWAY, &who(), NOW + 20).unwrap();
    assert_eq!(multiplier(&game), 1.0, "on foot again");
    // No Lawn Chair item in the gameplay fixture's captured catalogue: board it on the service.
    let mut service = MountService::new(MountCatalog::load(mounts_json()).unwrap());
    board_with(&mut service, LAWN_CHAIR, LAWN_CHAIR_ITEM);
    assert_eq!(service.mounted_speed(), Some(0.0));
    assert_eq!(
        SpeedModifiers::riding(service.mounted_speed()).mount_multiplier,
        1.0,
        "the Lawn Chair stands still"
    );

    // Through the gateway's movement check.
    assert_eq!(
        one_second_dash(Some((WOLF, WOLF_ITEM))),
        450.0,
        "accepted on the Wolf"
    );
    assert!(one_second_dash(None) < 450.0, "clamped on foot");
}
