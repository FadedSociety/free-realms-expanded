//! T016: Stone Heart's Hideaway through the production gateway path.
//!
//! Client shapes checked in the supplied 2014 client:
//! * `26/8`: family constructor `008b2820`, packet constructor `008b6bc0`
//!   (target guid and one byte), sender `0092efe0`.
//! * `36/10`: `ClientRequestStartAbility`, writer `00a342e0`; the basic slot
//!   is bar 1, slot 0, target mode 0 and the target guid.
//! * `41/108`: encounter entrance constructor `008b6e70`.
//! * `26/42`: results Close constructor `008a2eb0`, sender `0092f300`.

use osfr_tools::{
    account_store::AccountRole,
    chat,
    encounter::EncounterPhase,
    game_store,
    gateway::{Gateway, GatewayOutcome},
    startup_world::parse_add_npc,
};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const BATTLE: u32 = 12303;
/// The recovered Legacy entrance actor, reused by the field battle.
const ENTRANCE: u64 = 100_000_004_750;
const TITLE_ID: i32 = 426_971;
const OBJECTIVE_ID: i32 = 426_977;
const MATCH_MAKER_MALLET: i32 = 76_471;
const BRAWLER: i32 = 43;
const STONE_HEART: u64 = 100_001_230_301;
const HEARTBREAKERS: [u64; 3] = [100_001_230_302, 100_001_230_303, 100_001_230_304];
const PLAYER_POSITION: [f32; 3] = [-950.582, -34.42554, -648.0];

static SERIAL: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
    reference: PathBuf,
    content: PathBuf,
    version: String,
}

impl Fixture {
    fn new() -> Self {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "stone-heart-hideaway-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let reference = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../lab/references/rust-gateway-reference");
        let content = runtime_content(&root);
        let manifest: Value =
            serde_json::from_slice(&fs::read(reference.join("manifest.json")).unwrap()).unwrap();
        let version = manifest["client_version"].as_str().unwrap().to_owned();
        Self {
            root,
            reference,
            content,
            version,
        }
    }

    fn gateway(&self) -> Gateway {
        let mut character = Gateway::reference_state(&self.reference).unwrap();
        character.position = PLAYER_POSITION;
        character.rotation = [0.0, 0.0, 1.0];
        let mut gateway =
            Gateway::load_gameplay(&self.reference, character, &self.content, None).unwrap();
        gateway.set_dev_tools(true);
        gateway.set_role(AccountRole::Admin);
        self.login(&mut gateway);
        gateway
    }

    fn login(&self, gateway: &mut Gateway) -> GatewayOutcome {
        let mut login = 1_u16.to_le_bytes().to_vec();
        for (index, value) in ["local-private-test", self.version.as_str(), ""]
            .into_iter()
            .enumerate()
        {
            login.extend((value.len() as u32).to_le_bytes());
            login.extend(value.as_bytes());
            if index == 0 {
                login.extend(gateway.state().guid.to_le_bytes());
            }
        }
        gateway.handle_at(&login, 0).unwrap()
    }

    fn save(&self, gateway: &Gateway, name: &str) -> PathBuf {
        let path = self.root.join(name);
        game_store::save(&path, &gateway.snapshot()).unwrap();
        path
    }
}

fn runtime_content(root: &Path) -> PathBuf {
    if let Some(path) = std::env::var_os("OSFR_ACTIVITY_CONTENT") {
        return PathBuf::from(path);
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let base = manifest.join("../../../lab/references/opening-journal-text-v2/content");
    let catalogue =
        manifest.join("../../../lab/bundles/playtest-20261016T115811Z/content/item-catalogue.json");
    let content = root.join("content");
    let files = fs::read_dir(&base).unwrap();
    fs::create_dir_all(&content).unwrap();
    for entry in files {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            fs::copy(entry.path(), content.join(entry.file_name())).unwrap();
        }
    }
    let target = content.join("item-catalogue.json");
    if fs::hard_link(&catalogue, &target).is_err() {
        fs::copy(&catalogue, &target).unwrap();
    }
    // Gameplay derives NPC and field-battle content from the bundled package
    // when this directory has neither of these.
    let _ = fs::remove_file(content.join("npc_spawns_extra.json"));
    let _ = fs::remove_dir_all(content.join("npcs"));
    content
}

fn reopen(fixture: &Fixture, gateway: &Gateway, name: &str) -> Gateway {
    let path = fixture.save(gateway, name);
    let saved = game_store::load_or_create(&path, gateway.state().clone()).unwrap();
    Gateway::load_gameplay(
        &fixture.reference,
        saved.character,
        &fixture.content,
        saved.gameplay,
    )
    .unwrap()
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

fn unframe(reply: &[u8]) -> &[u8] {
    if reply.starts_with(&[5, 0, 1]) && reply.len() >= 7 {
        let length = u32::from_le_bytes(reply[3..7].try_into().unwrap()) as usize;
        if length == reply.len() - 7 {
            return &reply[7..];
        }
    }
    reply
}

fn send(gateway: &mut Gateway, inner: &[u8], now_ms: u64) -> GatewayOutcome {
    gateway
        .handle_at(&wrap(inner), now_ms)
        .expect("gateway frame")
}

fn interact(guid: u64) -> Vec<u8> {
    [vec![26, 0, 8, 0], guid.to_le_bytes().to_vec(), vec![0]].concat()
}

fn switch_job(profile: i32) -> Vec<u8> {
    [vec![26, 0, 13, 0], profile.to_le_bytes().to_vec()].concat()
}

fn play(activity: u32) -> Vec<u8> {
    let mut packet = vec![167, 0, 1, 2];
    packet.extend(activity.to_le_bytes());
    packet.extend(0_u32.to_le_bytes());
    packet
}

fn go(details: &[u8]) -> Vec<u8> {
    let mut packet = vec![41, 0, 108, 0];
    packet.extend(&details[4..12]); // encounter and launch instance
    packet.extend(0_u64.to_le_bytes()); // session, as in the native 41/108 frames
    packet
}

fn melee(guid: u64) -> Vec<u8> {
    [
        vec![36, 0, 10, 0],
        1_i32.to_le_bytes().to_vec(),
        0_i32.to_le_bytes().to_vec(),
        0_i32.to_le_bytes().to_vec(),
        guid.to_le_bytes().to_vec(),
    ]
    .concat()
}

fn move_to(guid: u64, position: [f32; 3]) -> Vec<u8> {
    let mut packet = 125_u16.to_le_bytes().to_vec();
    packet.extend(guid.to_le_bytes());
    for value in position.into_iter().chain([0.0, 0.0, 1.0]) {
        packet.extend(value.to_le_bytes());
    }
    packet.extend([0, 0]);
    packet
}

fn chat_text(outcome: &GatewayOutcome) -> String {
    outcome
        .replies
        .iter()
        .map(|packet| unframe(packet))
        .filter(|packet| packet.starts_with(&[15, 0, 3, 0]) && packet.len() >= 8)
        .map(|packet| {
            let length = u32::from_le_bytes(packet[4..8].try_into().unwrap()) as usize;
            String::from_utf8_lossy(&packet[8..8 + length]).into_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn entrance_actors(outcome: &GatewayOutcome) -> Vec<u64> {
    outcome
        .replies
        .iter()
        .map(|packet| unframe(packet))
        .filter(|packet| packet.starts_with(&[35, 0, 2, 0]))
        .filter_map(|packet| parse_add_npc(packet).ok())
        .filter(|npc| npc.name_string_id == TITLE_ID)
        .map(|npc| npc.actor_id)
        .collect()
}

fn details(outcome: &GatewayOutcome) -> Option<&[u8]> {
    outcome
        .replies
        .iter()
        .map(|packet| unframe(packet))
        .find(|packet| packet.starts_with(&[41, 0, 114, 0]))
}

fn arena_spawned(gateway: &mut Gateway, now_ms: u64) -> GatewayOutcome {
    let mut outcome = send(gateway, &13_u16.to_le_bytes(), now_ms);
    outcome.replies.extend(
        send(gateway, &10_u16.to_le_bytes(), now_ms + 100)
            .replies
            .into_iter(),
    );
    outcome
}

fn level(gateway: &Gateway, profile: i32) -> u32 {
    let state = gateway.snapshot().gameplay.unwrap();
    1 + state
        .profiles
        .unwrap()
        .experience
        .get(&profile)
        .copied()
        .unwrap_or(0)
        / 100
}

fn economy(gateway: &Gateway) -> (u32, u32, u32) {
    let state = gateway.snapshot().gameplay.unwrap();
    let item_count = state
        .inventory
        .items
        .iter()
        .filter(|item| item.definition_id == MATCH_MAKER_MALLET && item.rental.is_none())
        .map(|item| item.count)
        .sum();
    let experience = state
        .profiles
        .unwrap()
        .experience
        .get(&BRAWLER)
        .copied()
        .unwrap_or(0);
    (state.inventory.coins, item_count, experience)
}

fn active_enemy(
    gateway: &Gateway,
    guid: Option<u64>,
) -> Option<((u64, u32, [f32; 3]), EncounterPhase)> {
    let state = gateway.snapshot().gameplay?.encounter?;
    let active = state.active?;
    let enemy = guid
        .and_then(|guid| {
            active
                .enemies
                .iter()
                .find(|enemy| enemy.guid == guid && enemy.health > 0)
        })
        .or_else(|| {
            active
                .enemies
                .iter()
                .filter(|enemy| enemy.health > 0)
                .min_by_key(|enemy| enemy.guid == STONE_HEART)
        })?;
    Some(((enemy.guid, enemy.health, enemy.position), active.phase))
}

/// Run the gateway's production 20 Hz world tick across `duration_ms`.
fn tick_for(gateway: &mut Gateway, now_ms: &mut u64, duration_ms: u64) {
    let end = *now_ms + duration_ms;
    while *now_ms < end {
        *now_ms = (*now_ms + 50).min(end);
        gateway.tick_at(*now_ms).unwrap();
    }
}

fn player_health(gateway: &Gateway) -> u32 {
    gateway
        .snapshot()
        .gameplay
        .and_then(|state| state.encounter)
        .map_or(0, |state| state.player_health)
}

fn walk(gateway: &mut Gateway, guid: u64, to: [f32; 3], now_ms: &mut u64) {
    let from = gateway.state().position;
    let distance = ((to[0] - from[0]).powi(2) + (to[2] - from[2]).powi(2)).sqrt();
    let steps = (distance / 8.0).ceil().max(1.0) as usize;
    for step in 1..=steps {
        let amount = step as f32 / steps as f32;
        let position = [0, 1, 2].map(|axis| from[axis] + (to[axis] - from[axis]) * amount);
        *now_ms += 50;
        let moved = send(gateway, &move_to(guid, position), *now_ms);
        assert!(!moved.replies.iter().any(|reply| {
            let packet = unframe(reply);
            packet.starts_with(&[38, 0, 12, 0])
        }));
        assert_eq!(gateway.state().position, position);
        // The world keeps simulating between movement packets.
        tick_for(gateway, now_ms, 350);
    }
}

#[test]
fn stone_heart_hideaway_is_singular_level_gated_and_pays_through_gateway_once() {
    let fixture = Fixture::new();
    let mut gateway = fixture.gateway();
    let mut loaded = send(&mut gateway, &13_u16.to_le_bytes(), 100);
    loaded
        .replies
        .extend(send(&mut gateway, &10_u16.to_le_bytes(), 200).replies);
    assert_eq!(
        entrance_actors(&loaded),
        vec![ENTRANCE],
        "the field battle replaces the passive Legacy entrance instead of adding a second actor"
    );
    let switched = send(&mut gateway, &switch_job(BRAWLER), 300);
    assert!(switched.state_changed);
    let lowered = send(&mut gateway, &chat::local_request("realm level 12"), 400);
    assert!(
        chat_text(&lowered).contains("level 12"),
        "{:?}",
        lowered.replies
    );
    assert_eq!(level(&gateway, BRAWLER), 12);

    let screen = send(&mut gateway, &interact(ENTRANCE), 500);
    let details = details(&screen).expect("26/8 opens the recovered entrance screen");
    assert_eq!(
        i32::from_le_bytes(details[4..8].try_into().unwrap()),
        BATTLE as i32
    );
    assert_eq!(
        i32::from_le_bytes(details[47..51].try_into().unwrap()),
        TITLE_ID
    );

    let before_refusal = gateway.snapshot();
    let refused = send(&mut gateway, &go(details), 600);
    assert!(!refused.began_zoning);
    assert!(!refused.state_changed);
    assert!(
        chat_text(&refused).contains("level 13"),
        "{:?}",
        refused.replies
    );
    assert_eq!(
        gateway.snapshot(),
        before_refusal,
        "a low-level Go changes nothing"
    );

    // The same recovered battle is refused by the portal and the developer
    // start route through the one shared eligibility rule.
    let before_portal = gateway.snapshot();
    let portal = send(&mut gateway, &play(BATTLE), 650);
    assert!(
        chat_text(&portal).contains("level 13"),
        "the portal shares the marker's floor: {:?}",
        portal.replies
    );
    assert_eq!(
        gateway.snapshot(),
        before_portal,
        "a low-level portal Play changes nothing"
    );
    let before_developer = gateway.snapshot();
    let developer = send(
        &mut gateway,
        &chat::local_request("realm battle 12303"),
        660,
    );
    assert!(
        chat_text(&developer).contains("level 13"),
        "the developer start route shares the same floor: {:?}",
        developer.replies
    );
    assert_eq!(
        gateway.snapshot(),
        before_developer,
        "a low-level developer start changes nothing"
    );

    let raised = send(&mut gateway, &chat::local_request("realm level 13"), 700);
    assert!(
        chat_text(&raised).contains("level 13"),
        "{:?}",
        raised.replies
    );
    assert_eq!(level(&gateway, BRAWLER), 13);

    // The balance simulator starts every level at 1; it must skip the levels
    // this battle does not admit and still evaluate 15 and 20.
    let before_sim = gateway.snapshot();
    let sim = send(&mut gateway, &chat::local_request("realm sim 12303"), 750);
    let sim_text = chat_text(&sim);
    assert!(
        sim_text.contains("level 15") && sim_text.contains("level 20"),
        "eligible simulator levels still run: {sim_text}"
    );
    assert!(
        sim_text.contains("needs level 13"),
        "ineligible simulator levels report the production floor: {sim_text}"
    );
    assert!(
        !sim_text.contains("Cannot simulate"),
        "one ineligible level must not end the whole simulation: {sim_text}"
    );
    assert_eq!(
        gateway.snapshot(),
        before_sim,
        "the balance simulator changes nothing"
    );

    let go = send(&mut gateway, &go(details), 800);
    assert!(go.began_zoning, "the pending entry survives the refusal");
    assert_eq!(gateway.state().zone_id, 9600);

    let arena = arena_spawned(&mut gateway, 900);
    for guid in std::iter::once(STONE_HEART).chain(HEARTBREAKERS) {
        assert!(
            arena
                .replies
                .iter()
                .map(|packet| unframe(packet))
                .any(|packet| packet.starts_with(&[35, 0, 2, 0])
                    && packet[4..12] == guid.to_le_bytes()),
            "arena enemy {guid} spawned"
        );
    }
    assert!(
        arena
            .replies
            .iter()
            .map(|packet| unframe(packet))
            .any(|packet| packet
                .windows(4)
                .any(|field| field == OBJECTIVE_ID.to_le_bytes())),
        "the recovered objective reaches the client"
    );
    let (_, phase) = active_enemy(&gateway, None).expect("arena enemies spawned");
    assert_eq!(phase, EncounterPhase::Fighting);

    // The level-13 gate above is what this test pins; fight as a max-level
    // Brawler so the real enemy schedule is a survivable production fight
    // rather than a scripted immune one.
    let raised = send(&mut gateway, &chat::local_request("realm level 20"), 950);
    assert!(
        chat_text(&raised).contains("level 20"),
        "{:?}",
        raised.replies
    );
    assert_eq!(level(&gateway, BRAWLER), 20);
    let before_fight = economy(&gateway);

    let player_guid = gateway.state().guid;
    let mut now = 1_000;
    let mut target = None;
    let mut enemy_took_turns = false;
    for _ in 0..200 {
        let Some(((guid, _, position), phase)) = active_enemy(&gateway, target) else {
            break;
        };
        if phase != EncounterPhase::Fighting {
            break;
        }
        target = Some(guid);
        let approach = [position[0] + 1.5, position[1], position[2]];
        walk(&mut gateway, player_guid, approach, &mut now);
        // Fight this target on the server's swing cooldown, running the
        // production 20 Hz world tick between swings; the old scripted fight
        // skipped every enemy turn.
        while let Some(((current, _, _), phase)) = active_enemy(&gateway, target) {
            if phase != EncounterPhase::Fighting || current != guid {
                break;
            }
            send(&mut gateway, &melee(guid), now);
            let before_swing = player_health(&gateway);
            tick_for(&mut gateway, &mut now, 700);
            enemy_took_turns |= player_health(&gateway) < before_swing;
        }
    }
    assert!(
        enemy_took_turns,
        "the world tick must run the enemies' turns, not only the scripted attacks"
    );

    let won = gateway.snapshot().gameplay.unwrap().encounter.unwrap();
    assert_eq!(won.active.as_ref().unwrap().phase, EncounterPhase::Won);
    assert!(
        won.player_health > 0,
        "the player survived the real schedule"
    );
    assert_eq!(
        economy(&gateway),
        (before_fight.0, before_fight.1 + 1, before_fight.2 + 50),
        "the server-owned win grants the item and Brawler XP exactly once"
    );
    let paid = economy(&gateway);
    let repeated_attack = send(&mut gateway, &melee(STONE_HEART), now.saturating_add(700));
    assert!(!repeated_attack.state_changed);
    assert_eq!(economy(&gateway), paid);

    let reconnected = reopen(&fixture, &gateway, "won.json");
    assert_eq!(
        economy(&reconnected),
        paid,
        "the reward survives a save and reconnect"
    );
    assert!(
        reconnected
            .snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active
            .is_none(),
        "reconnect releases the connection-local encounter"
    );

    let closed = send(&mut gateway, &[26, 0, 42, 0], now + 800);
    assert_eq!(gateway.state().zone_id, 0);
    assert_eq!(economy(&gateway), paid);
    assert!(closed.began_zoning);
    let after_close = gateway.snapshot();
    let duplicate = send(&mut gateway, &[26, 0, 42, 0], now + 900);
    assert!(!duplicate.state_changed);
    assert!(!duplicate.began_zoning);
    assert_eq!(
        gateway.snapshot(),
        after_close,
        "a duplicate results Close changes no saved state"
    );

    let reloaded = reopen(&fixture, &gateway, "closed.json");
    assert_eq!(economy(&reloaded), paid);
    assert_eq!(reloaded.state().zone_id, 0);
}
