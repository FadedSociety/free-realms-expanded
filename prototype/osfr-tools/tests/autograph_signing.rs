//! P-autograph: "Can I Have Your Autograph", the Seaside autograph signing of Kart Driver quest
//! The Price of Fame (3080023), as `world_challenges.json` row 592.
//!
//! The client has text for the game (395139, the fan lines 394796-394881, the groups 394794 and
//! 394899) but no `MiniGameData.txt` row and no emitter, so 592 is a hand id (R) and the game runs
//! on the in-world challenge system: Roxy's menu offers it, Go lays the six fans' stops out, and
//! walking up to each fan in turn signs for them. A win sends the 39/18 that credits a
//! `finish_minigame` step naming 592 (`enabled_quest_reachability` pins the id).
//!
//! The gateway test copies the reference gameplay package and adds the shipped
//! `world_challenges.json`; the NPC content is the shipped `data/content/npcs` through the
//! production fallback (`content::npc_content`). The client packets are the 26/8 click
//! (`008b6bc0`), the 26/10 menu choice (`00a36f80`), the 39/5 Go (`MinigameRuns::handle_client`)
//! and opcode 125 positions (`00934a20`). All clocks here are test clocks.

#[path = "common/gateway_packets.rs"]
mod gateway_packets;

use gateway_packets::move_to;
use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState},
    gateway::{Gateway, GatewayOutcome},
    npc_interaction::NpcInteraction,
    world_challenge::{Challenges, FIRST_INSTANCE_ID, Kind},
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const CRATE: &str = env!("CARGO_MANIFEST_DIR");
const AUTOGRAPH: u32 = 592;
const ROXY: u64 = 9_500_000_620;
/// The NPC content's Seaside package label; the town itself lies in the open world, zone 0.
const SEASIDE: u32 = 11;
const KART_DRIVER: i32 = 48;
/// Client Global.Text: the quest's "Beat the ... minigame" line (the game's only client name),
/// the wiki's description, and the six "Sign an autograph for <fan>" goal lines in route order.
const NAME: u32 = 395_139;
const DESCRIPTION: u32 = 394_798;
const FANS: [(&str, u64, u32); 6] = [
    ("Cole", 9_500_000_567, 394_796),
    ("Brenna", 9_500_000_562, 394_843),
    ("Gaius", 9_500_000_585, 394_847),
    ("Sanford", 9_500_000_622, 394_857),
    ("Esther Summerspark", 9_500_000_582, 394_861),
    ("Banani", 9_500_000_554, 394_863),
];
/// What a win pays the Kart Driver in production: the job's `activity_xp` of
/// `data/release/job_xp_curve.json` (97 per race), not the row's fallback 35.
const KART_ACTIVITY_XP: u32 = 97;
const NOW: u64 = 10_000;
static NEXT: AtomicU64 = AtomicU64::new(0);

fn data() -> PathBuf {
    Path::new(CRATE).join("data/content")
}

fn reference_content() -> PathBuf {
    std::env::var_os("OSFR_TYPE8_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(CRATE).join("../../../lab/references/opening-journal-text-v2/content")
        })
}

fn reference() -> PathBuf {
    std::env::var_os("OSFR_GATEWAY_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(CRATE).join("../../../lab/references/rust-gateway-reference"))
}

fn copy_dir(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// Reference gameplay content plus the shipped challenge file; no NPC package, so the production
/// fallback binds the shipped `data/content/npcs`.
struct Content(PathBuf);

impl Content {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "osfr-autograph-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        copy_dir(&reference_content(), &dir);
        fs::copy(
            data().join("world_challenges.json"),
            dir.join("world_challenges.json"),
        )
        .unwrap();
        // L927: the release curve file makes a win pay the Kart Driver's `activity_xp`, as the
        // shipped bundle does; without it `challenge_xp` falls back to the row's own 35.
        fs::copy(
            Path::new(CRATE).join("data/release/job_xp_curve.json"),
            dir.join("job_xp_curve.json"),
        )
        .unwrap();
        Self(dir)
    }

    /// A Kart Driver's save: the row pays and locks on profile 48.
    fn state(&self) -> GameplayState {
        let mut state = Gameplay::load(&self.0, None).unwrap().state();
        let profiles = state
            .profiles
            .as_mut()
            .expect("the reference save has jobs");
        profiles.unlocked.insert(KART_DRIVER);
        state
    }
}

impl Drop for Content {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn shipped_npcs() -> NpcInteraction {
    NpcInteraction::load(&data().join("npcs"), &data().join("stores.json")).unwrap()
}

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut packet = vec![5, 0, 1];
    packet.extend((inner.len() as u32).to_le_bytes());
    packet.extend(inner);
    packet
}

fn login(gateway: &mut Gateway) {
    let guid = gateway.state().guid;
    let mut packet = 1u16.to_le_bytes().to_vec();
    for (index, value) in ["p-autograph-test", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
        packet.extend((value.len() as u32).to_le_bytes());
        packet.extend(value.as_bytes());
        if index == 0 {
            packet.extend(guid.to_le_bytes());
        }
    }
    gateway.handle_at(&packet, 0).unwrap();
    for packet in [wrap(&[13, 0]), wrap(&[10, 0])] {
        gateway.handle_at(&packet, 1_000).unwrap();
    }
}

fn gateway_at(content: &Content, position: [f32; 3]) -> Gateway {
    let mut character: CharacterState = Gateway::reference_state(&reference()).unwrap();
    // Seaside is a content package label (11) inside the one open world, zone 0
    // (`npc_interaction::check_range`): the player stands in zone 0.
    character.zone_id = 0;
    character.position = position;
    let mut gateway =
        Gateway::load_gameplay(&reference(), character, &content.0, Some(content.state())).unwrap();
    login(&mut gateway);
    gateway
}

fn npc_click(target: u64) -> Vec<u8> {
    let mut packet = vec![26, 0, 8, 0];
    packet.extend(target.to_le_bytes());
    packet.push(0);
    packet
}

fn npc_select(target: u64, choice: i32) -> Vec<u8> {
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(target.to_le_bytes());
    packet.extend(choice.to_le_bytes());
    packet
}

/// The 39/x lifecycle header the client sends: instance, group -1, game -1.
fn lifecycle(sub: u8, instance: i32) -> Vec<u8> {
    let mut packet = vec![39, 0, sub];
    for value in [instance, -1, -1] {
        packet.extend(value.to_le_bytes());
    }
    packet
}

fn bodies(outcome: &GatewayOutcome) -> Vec<Vec<u8>> {
    outcome
        .replies
        .iter()
        .map(|packet| {
            assert!(packet.len() >= 7 && packet[..3] == [5, 0, 1]);
            packet[7..].to_vec()
        })
        .collect()
}

/// The option id whose 26/9 row carries `label`, from the client menu reader shape.
fn menu_option(menu: &[Vec<u8>], label: u32) -> Option<i32> {
    let packet = menu
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 9, 0]))?;
    let count = i32::from_le_bytes(packet[13..17].try_into().ok()?) as usize;
    (0..count).find_map(|index| {
        let at = 17 + index * 28;
        let id = i32::from_le_bytes(packet[at..at + 4].try_into().ok()?);
        let row_label = u32::from_le_bytes(packet[at + 8..at + 12].try_into().ok()?);
        (row_label == label).then_some(id)
    })
}

/// The `name_id` of a 47/1 goal tracker row (`minigame::goal_row_packet`).
fn goal_row(packet: &[u8]) -> Option<u32> {
    (packet.starts_with(&[47, 0, 1]) && packet.len() >= 15)
        .then(|| u32::from_le_bytes(packet[11..15].try_into().unwrap()))
}

/// A 39/18 result for `game`: `Some(won)`.
fn game_over(packets: &[Vec<u8>], game: u32) -> Option<bool> {
    packets.iter().find_map(|packet| {
        let header = packet.starts_with(&[39, 0, 18])
            && packet.len() >= 16
            && packet[11..15] == (game as i32).to_le_bytes();
        header.then(|| packet[15] == 1)
    })
}

fn flat_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// Walk from `from` to `to` in legs the movement envelope accepts (one a second), ticking the
/// world after each, and return the last tick's packets with the clock.
fn walk(gateway: &mut Gateway, from: [f32; 3], to: [f32; 3], mut now: u64) -> (Vec<Vec<u8>>, u64) {
    let guid = gateway.state().guid;
    let legs = (flat_distance(from, to) / 40.0).ceil().max(1.0) as usize;
    let mut packets = Vec::new();
    for leg in 1..=legs {
        let t = leg as f32 / legs as f32;
        let at = [
            from[0] + (to[0] - from[0]) * t,
            from[1] + (to[1] - from[1]) * t,
            from[2] + (to[2] - from[2]) * t,
        ];
        now += 1_000;
        let moved = gateway.handle_at(&wrap(&move_to(guid, at)), now).unwrap();
        assert!(
            !moved
                .replies
                .iter()
                .any(|r| r.len() > 7 && r[7..].starts_with(&[38, 0, 12])),
            "the walk must not be snapped back"
        );
        assert_eq!(gateway.state().position, at, "the leg must be accepted");
        packets = bodies(&gateway.tick_at(now).unwrap());
    }
    (packets, now)
}

#[test]
fn the_shipped_row_is_roxys_six_stop_signing_in_seaside() {
    let content = Challenges::load(&data())
        .unwrap()
        .expect("the shipped file");
    let row = content.get(AUTOGRAPH).expect("row 592");
    assert_eq!(row.title, "Can I Have Your Autograph");
    assert_eq!(
        (row.giver, row.name_id, row.description_id),
        (ROXY, NAME, DESCRIPTION)
    );
    assert_eq!(
        (row.difficulty, row.profile),
        (1, KART_DRIVER),
        "Easy, pays Kart Driver"
    );
    assert!(
        row.source.iter().any(|s| s.contains("(R) id 592")),
        "the hand id is marked"
    );
    assert_eq!(row.stages.len(), FANS.len());
    let npcs = shipped_npcs();
    let (roxy_zone, roxy) = npcs.location(ROXY).expect("Roxy stands in the NPC content");
    assert_eq!(roxy_zone, SEASIDE);
    for (stage, (fan, key, line)) in row.stages.iter().zip(FANS) {
        assert_eq!(stage.kind, Kind::Deliver, "{fan}");
        assert_eq!(
            (stage.count, stage.name_id),
            (1, line),
            "{fan}: one stop, the fan's own line"
        );
        assert_eq!(stage.recipients, vec![key], "{fan}");
        let (zone, at) = npcs
            .location(key)
            .unwrap_or_else(|| panic!("{fan} has a body"));
        assert_eq!(zone, SEASIDE, "{fan} is in Seaside");
        let reach = flat_distance(roxy, at);
        assert!(
            (50.0..300.0).contains(&reach),
            "{fan} is {reach} m from Roxy"
        );
    }
}

#[test]
fn roxys_menu_offers_the_signing_and_walking_to_every_fan_wins_it() {
    let content = Content::new();
    let npcs = shipped_npcs();
    let (_, roxy) = npcs.location(ROXY).unwrap();
    let mut gateway = gateway_at(&content, roxy);
    let coins_before = gateway.snapshot().gameplay.unwrap().inventory.coins;

    // The click on Roxy lists the game under the quest line's own client text.
    let menu = bodies(&gateway.handle_at(&wrap(&npc_click(ROXY)), NOW).unwrap());
    let option = menu_option(&menu, NAME).expect("Roxy's menu offers the signing");
    let offered = bodies(
        &gateway
            .handle_at(&wrap(&npc_select(ROXY, option)), NOW + 100)
            .unwrap(),
    );
    let offer = offered
        .iter()
        .find(|p| p.starts_with(&[39, 0, 16]))
        .expect("the 39/16 start screen");
    assert_eq!(
        offer[11..15],
        (AUTOGRAPH as i32).to_le_bytes(),
        "the offer names 592"
    );

    // Go lays the first stop out: the first goal line is Cole's, the timer and the arrow follow.
    let started = bodies(
        &gateway
            .handle_at(&wrap(&lifecycle(5, FIRST_INSTANCE_ID)), NOW + 200)
            .unwrap(),
    );
    assert!(
        started.iter().any(|p| goal_row(p) == Some(FANS[0].2)),
        "the goal tracker row on Go is Cole's line: {started:?}"
    );
    assert!(
        !started.iter().any(|p| p.starts_with(&[35, 0, 2])),
        "a delivery spawns nothing: the fans already stand there"
    );

    // Walking up to each fan in turn signs for them; the last one wins the game.
    let mut at = roxy;
    let mut now = NOW + 200;
    let mut result = None;
    for (index, (fan, key, _)) in FANS.iter().enumerate() {
        let (_, stop) = npcs.location(*key).unwrap();
        let (packets, later) = walk(&mut gateway, at, stop, now);
        at = stop;
        now = later;
        result = game_over(&packets, AUTOGRAPH);
        assert!(
            packets.iter().any(|p| p.starts_with(&[47, 0, 3])),
            "{fan}'s goal is marked done when the player reaches them: {packets:?}"
        );
        if index + 1 < FANS.len() {
            assert_eq!(result, None, "{fan} is not the last stop");
            let next = FANS[index + 1].2;
            assert!(
                packets.iter().any(|p| goal_row(p) == Some(next)),
                "{fan}: the next fan's line opens"
            );
        }
    }
    assert_eq!(result, Some(true), "the sixth autograph wins the game");
    let after = gateway.snapshot().gameplay.unwrap();
    assert_eq!(
        after.inventory.coins,
        coins_before + 45,
        "the win pays the row's coins"
    );
    assert_eq!(
        after
            .profiles
            .as_ref()
            .unwrap()
            .experience
            .get(&KART_DRIVER)
            .copied(),
        Some(KART_ACTIVITY_XP),
        "the win pays the Kart Driver's activity XP, not the row's fallback"
    );
}

#[test]
fn a_fan_out_of_order_or_a_player_far_from_roxy_changes_nothing() {
    let content = Content::new();
    let npcs = shipped_npcs();
    let (_, roxy) = npcs.location(ROXY).unwrap();
    let (_, banani) = npcs.location(FANS[5].1).unwrap();

    // A click on Roxy from across town opens no menu (and a choice there starts nothing).
    let mut far = gateway_at(&content, banani);
    let before = serde_json::to_vec(&far.snapshot()).unwrap();
    let menu = far
        .handle_at(&wrap(&npc_click(ROXY)), NOW)
        .map(|out| bodies(&out))
        .unwrap_or_default();
    assert!(
        menu_option(&menu, NAME).is_none(),
        "Roxy is out of reach from Banani's spot, so the signing is not offered"
    );
    let _ = far.handle_at(&wrap(&npc_select(ROXY, 1)), NOW + 100);
    assert!(
        !bodies(&far.tick_at(NOW + 200).unwrap())
            .iter()
            .any(|p| p.starts_with(&[39, 0])),
        "nothing of the game runs"
    );
    assert_eq!(
        serde_json::to_vec(&far.snapshot()).unwrap(),
        before,
        "nothing changed"
    );

    // The first stop is Cole's: standing by Banani first signs nothing.
    let mut gateway = gateway_at(&content, roxy);
    let menu = bodies(&gateway.handle_at(&wrap(&npc_click(ROXY)), NOW).unwrap());
    let option = menu_option(&menu, NAME).unwrap();
    gateway
        .handle_at(&wrap(&npc_select(ROXY, option)), NOW + 100)
        .unwrap();
    gateway
        .handle_at(&wrap(&lifecycle(5, FIRST_INSTANCE_ID)), NOW + 200)
        .unwrap();
    let (packets, _) = walk(&mut gateway, roxy, banani, NOW + 200);
    assert!(
        !packets.iter().any(|p| p.starts_with(&[39, 0, 18])),
        "Banani before Cole ends nothing"
    );
    assert!(
        !packets.iter().any(|p| p.starts_with(&[47, 0, 3])),
        "no goal is marked done out of order"
    );
}
