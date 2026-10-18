use super::*;
use crate::quest_engine::{
    CharacterEligibility, ObjectiveKind, QuestDefinition, QuestEngine, QuestObjective,
    QuestRequirements,
};
use crate::rewards::{RewardDisplay, RewardGrant};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
const FISHING_T0: u64 = 1_000_000;
const FISHING_ACTOR: crate::fishing::Handle = crate::fishing::Handle {
    id: 0x1122_3344,
    seq: 0x5566_7788,
};

fn reference() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-opening-cooking-original-assets")
}

fn game() -> Gameplay {
    let path = reference();
    assert!(
        path.join("opening.json").is_file(),
        "T013 requires the private opening reference at {}",
        path.display()
    );
    Gameplay::load(&path, None).expect("opening reference loads")
}

fn eligibility() -> CharacterEligibility {
    CharacterEligibility {
        member: true,
        member_quest_limit: 30,
        ..CharacterEligibility::default()
    }
}

fn definition(id: u32, kind: ObjectiveKind, required: u32) -> QuestDefinition {
    definition_many(id, vec![(1, kind, required)], true)
}

fn definition_many(
    id: u32,
    goals: Vec<(u32, ObjectiveKind, u32)>,
    ordered: bool,
) -> QuestDefinition {
    QuestDefinition {
        id,
        title: "Synthetic T013 objective (R)".into(),
        description: String::new(),
        giver_npc_id: 1,
        turnin_npc_id: 1,
        prerequisite_quest_ids: vec![],
        requirements: QuestRequirements::default(),
        ordered,
        objectives: goals
            .into_iter()
            .map(|(id, kind, required)| QuestObjective {
                id,
                required,
                description: "Synthetic T013 step (R)".into(),
                kind,
            })
            .collect(),
        reward: RewardGrant {
            coins: 0,
            items: vec![],
            job_xp: Default::default(),
            display: RewardDisplay {
                source_guid: 1,
                player_guid: 1,
                reward_kind: 0,
                icon_id: 0,
                name_id: 0,
                rows: vec![],
            },
        },
    }
}

fn install(game: &mut Gameplay, kind: ObjectiveKind, required: u32) {
    install_many(game, definition(1, kind, required));
}

fn install_many(game: &mut Gameplay, definition: QuestDefinition) {
    let mut engine =
        QuestEngine::new(1, vec![definition], eligibility(), None).expect("synthetic quest engine");
    assert!(engine.offer(1, 1).changed);
    assert!(engine.accept(1).changed);
    game.npc_quests = Some(engine);
}

struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-t013-{tag}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&directory).unwrap();
        Self(directory)
    }

    fn copy_reference_without_opening() -> Self {
        let scratch = Self::new("no-opening");
        copy_tree(&reference(), &scratch.0);
        for name in [
            "opening.json",
            "smelting.json",
            "mining.json",
            "harvesting.json",
            "forging-campaign.json",
            "cooking-campaign.json",
        ] {
            let path = scratch.0.join(name);
            if path.is_file() {
                fs::remove_file(path).unwrap();
            }
        }
        scratch
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn type8_board(goals: usize) -> crate::mining::Type8BoardSpec {
    use crate::mining::{SIDE, Type8BoardSpec, Type8Goal, Type8Kind, Type8Output};
    const LEFT: usize = 3;
    const TOP: usize = 3;
    const PLAYABLE: usize = 8;
    const BOTTOM: usize = TOP + PLAYABLE - 1;
    const OUTPUTS: [i32; 4] = [2739, 879, 65834, 2436];
    const VALUES: [u16; 4] = [601, 602, 603, 604];
    let values = &VALUES[..goals];
    let mut shape = vec![0_i32; SIDE * SIDE];
    for dy in 0..PLAYABLE {
        for dx in 0..PLAYABLE {
            shape[(TOP + dy) * SIDE + LEFT + dx] = 1;
        }
    }
    for (slot, value) in values.iter().enumerate() {
        shape[BOTTOM * SIDE + LEFT + slot] = -i32::from(*value);
    }
    for x in LEFT..LEFT + PLAYABLE {
        shape[(TOP + PLAYABLE) * SIDE + x] = 4;
    }
    Type8BoardSpec {
        width: SIDE,
        height: SIDE,
        shape,
        selectable: vec![401, 402, 403, 404],
        collectable: values.to_vec(),
        tiles: [401, 402, 403, 404]
            .iter()
            .map(|value| Type8Kind {
                value: *value,
                category: 400,
                icon_id: i32::from(*value) - 400,
            })
            .chain(values.iter().map(|value| Type8Kind {
                value: *value,
                category: 600,
                icon_id: 700 + i32::from(*value),
            }))
            .collect(),
        goals: values
            .iter()
            .enumerate()
            .map(|(slot, value)| Type8Goal {
                index: slot as i32,
                value: *value,
                quota: 1,
                per_reward: 1,
                icon_id: 700 + i32::from(*value),
            })
            .collect(),
        outputs: values
            .iter()
            .enumerate()
            .map(|(slot, value)| Type8Output {
                value: *value,
                definition_id: OUTPUTS[slot],
                cluster_quantity: 1,
            })
            .collect(),
        duration_seconds: 180,
        seed: 112,
        wildcard_percent: 0,
        collectable_percent: 0,
        blocker_percent: 0,
        powerup_percent: 0,
        meter_id: 0,
        reconstructed_rules: true,
        rule_notice: "Synthetic private F112 fixture; original level tables unrecovered.".into(),
    }
}

fn gathering_game(profile: i32, goals: usize, quest: QuestDefinition) -> (Gameplay, Scratch) {
    let scratch = Scratch::new("type8");
    let mining = json!({
        "schema_version": 1,
        "game_id": crate::mining::MINING_GAME_ID,
        "profile_id": crate::mining::MINER_PROFILE_ID,
        "name_id": 7910,
        "description_id": 7911,
        "icon_id": 909,
        "spec": type8_board(goals),
    });
    let harvesting = json!({
        "schema_version": 1,
        "game_id": HARVESTING_GAME_ID,
        "profile_id": harvesting::CHEF_PROFILE_ID,
        "name_id": 100834,
        "description_id": 94150,
        "icon_id": 908,
        "preload_file": harvesting::FARMING_PRELOAD_FILE,
        "launch_movie": harvesting::FARMING_LAUNCH_MOVIE,
        "spec": type8_board(goals),
    });
    fs::write(
        scratch.0.join(mining::CONTENT_FILE),
        serde_json::to_vec(&mining).unwrap(),
    )
    .unwrap();
    fs::write(
        scratch.0.join(harvesting::CONTENT_FILE),
        serde_json::to_vec(&harvesting).unwrap(),
    )
    .unwrap();
    let mut game = game();
    game.mining = Some(
        Type8Engine::load(
            &scratch.0,
            mining::CONTENT_FILE,
            mining::MINING_GAME_ID,
            mining::FIRST_MINING_INSTANCE,
            None,
        )
        .expect("synthetic mining content"),
    );
    game.harvesting = Some(
        Type8Engine::load(
            &scratch.0,
            harvesting::CONTENT_FILE,
            HARVESTING_GAME_ID,
            harvesting::FIRST_INSTANCE_ID,
            None,
        )
        .expect("synthetic harvesting content"),
    );
    install_many(&mut game, quest);
    select(&mut game, profile);
    (game, scratch)
}

fn gather_node(game: &Gameplay, name: &str, skip: usize) -> (u64, CharacterState) {
    let actor = game
        .gathering_nodes
        .as_ref()
        .expect("gathering nodes")
        .actors()
        .filter(|actor| actor.name == name)
        .nth(skip)
        .expect("gathering node")
        .clone();
    (
        actor.guid,
        CharacterState {
            guid: 1,
            zone_id: 0,
            position: actor.position,
            rotation: [0., 0., 1.],
        },
    )
}

fn node_click(guid: u64) -> Vec<u8> {
    let mut packet = vec![26, 0, 8, 0];
    packet.extend(guid.to_le_bytes());
    packet.push(0);
    packet
}

fn type8_frame(sub: u8, instance: i32) -> Vec<u8> {
    let mut packet = vec![39, 0, sub];
    packet.extend(instance.to_le_bytes());
    packet.extend((-1i32).to_le_bytes());
    packet.extend((-1i32).to_le_bytes());
    packet
}

fn type8_payload(instance: i32, text: &str) -> Vec<u8> {
    let mut packet = type8_frame(14, instance);
    packet.extend(((text.len() + 1) as i32).to_le_bytes());
    packet.extend(text.as_bytes());
    packet.push(0);
    packet
}

fn cooking_movie_request(sub: u8, instance: i32) -> Vec<u8> {
    let mut packet = vec![39, 0, sub];
    packet.extend(instance.to_le_bytes());
    packet.extend((-1i32).to_le_bytes());
    packet.extend((-1i32).to_le_bytes());
    packet
}

fn play_cooking(game: &mut Gameplay, character: &CharacterState, base_ms: u64) {
    let instance = game
        .cooking
        .as_ref()
        .unwrap()
        .state()
        .active
        .as_ref()
        .expect("cooking run")
        .instance_id;
    send(
        game,
        character,
        &cooking_movie_request(5, instance),
        base_ms,
    );
    send(
        game,
        character,
        &crate::cooking::payload_packet(instance, "FRServer_ReadyForGameData").unwrap(),
        base_ms + 1,
    );
    for stage in [1, 2, 3, 6, 17, 7] {
        for kind in 0..4 {
            let text = format!("FRServer_StatUpdate\t{stage}\t{kind}\t1)");
            send(
                game,
                character,
                &crate::cooking::payload_packet(instance, &text).unwrap(),
                base_ms + 2,
            );
        }
    }
    for kind in 4..8 {
        let text = format!("FRServer_StatUpdate\t-1\t{kind}\t6)");
        send(
            game,
            character,
            &crate::cooking::payload_packet(instance, &text).unwrap(),
            base_ms + 3,
        );
    }
    send(
        game,
        character,
        &crate::cooking::payload_packet(instance, "FRServer_GameWon\t900\t0").unwrap(),
        base_ms + 4,
    );
}

fn send(game: &mut Gameplay, character: &CharacterState, packet: &[u8], now: u64) -> Outcome {
    game.handle_in_world(packet, character, now)
        .expect("gameplay frame")
}

fn finish_type8(
    game: &mut Gameplay,
    character: &CharacterState,
    instance: i32,
    goals: usize,
    base_ms: u64,
) {
    for (step, packet) in [
        type8_frame(5, instance),
        type8_payload(instance, "OnConnectMsg"),
        type8_payload(instance, "OnStartGameMsg"),
        type8_payload(instance, "OnStartLevelMsg"),
    ]
    .iter()
    .enumerate()
    {
        send(game, character, packet, base_ms + step as u64 * 100);
    }
    for slot in 0..goals {
        let text = format!("OnLevelEventMsg\t50\t{}\t{}", 3 + slot, 10);
        send(
            game,
            character,
            &type8_payload(instance, &text),
            base_ms + 500 + slot as u64,
        );
    }
    send(
        game,
        character,
        &type8_payload(instance, "OnNextLevelMsg"),
        base_ms + 600,
    );
}

fn kart_character() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: [10., 20., 30.],
        rotation: [0., 0., 1.],
    }
}

fn kart_gateway(scratch: &Scratch) -> crate::kart_gateway::KartGateway {
    use crate::{
        kart_entry::{ProxiedVehicle, RaceConfig, RaceEntryContent, RaceEntryContext},
        kart_gateway::KartGatewayContent,
        kart_progress_native::ProgressPayload,
        kart_racing::RewardSchedule,
        kart_service::{GateWaypoint, KartServiceContent, RaceBounds, RaceStep, ScheduledFrame},
        zone_transition::ZoneTransition,
    };
    let context = RaceEntryContext {
        session: 123,
        instance_id: 1008,
        encounter_id: 917566,
        scene: ZoneTransition {
            envelope_flag: 1,
            scene: "sg_kt_track".into(),
            type_id: 2,
            position: [719., 72., 421., 1.],
            rotation: [0., 0., 1., 0.],
            sky: "sky.xml".into(),
            geometry_id: 101,
            ..Default::default()
        },
    };
    let details = |state: u32| {
        let mut bytes = vec![0u8; 391];
        bytes[..4].copy_from_slice(&[41, 0, 114, 0]);
        for (offset, word) in [
            (4, context.encounter_id),
            (8, context.instance_id),
            (12, context.encounter_id),
            (16, state),
        ] {
            bytes[offset..offset + 4].copy_from_slice(&word.to_le_bytes());
        }
        bytes[32..40].copy_from_slice(&context.session.to_le_bytes());
        for i in 0..3 {
            bytes[51 + i * 4..55 + i * 4].copy_from_slice(&context.scene.position[i].to_le_bytes());
        }
        bytes
    };
    let progress = ProgressPayload {
        racer_guid: 0,
        slot: 0,
        unknown_0c_bits: 0,
        unknown_10_bits: 0x7f7f_ffff,
        laps_completed: 0,
        local_racer: 1,
        lap_limit: 2,
        unknown_1d_bits: 0,
        best_lap_bits: 0,
        total_time_bits: 0,
        tail_0x29: 0,
        tail_0x31: 0,
        tail_0x39: 0,
        tail_0x41: 0,
    };
    let step = |at_ms, frame| RaceStep { at_ms, frame };
    let race = KartServiceContent {
        speedway: 31,
        laps: 2,
        reward: Some(RewardSchedule { coins: 40, xp: 60 }),
        gates: (1..=4)
            .map(|i| GateWaypoint {
                position: [i as f32 * 10.0, 0.0, 0.0],
                radius: 3.0,
            })
            .collect(),
        schedule: vec![
            step(3_026, ScheduledFrame::GameState(4)),
            step(3_026, ScheduledFrame::NpcCount(0)),
            step(3_353, ScheduledFrame::GameState(5)),
            step(8_246, ScheduledFrame::GameState(6)),
            step(8_694, ScheduledFrame::GameState(7)),
            step(9_290, ScheduledFrame::GameState(8)),
            step(9_290, ScheduledFrame::Countdown(4)),
            step(10_074, ScheduledFrame::Countdown(3)),
            step(10_529, ScheduledFrame::Countdown(2)),
            step(11_954, ScheduledFrame::Countdown(1)),
            step(13_241, ScheduledFrame::Countdown(0)),
            step(14_505, ScheduledFrame::GameState(9)),
        ],
        progress_template: ProgressPayload {
            racer_guid: 0,
            ..progress
        },
        start_position: [0.0, 0.0, 0.0],
        bounds: RaceBounds {
            max_speed_mps: 60.0,
            max_step_ms: 60_000,
            max_race_ms: 600_000,
            max_claims: 64,
            max_observations: 4_096,
        },
        gate_band: None,
        gate_curtains: Vec::new(),
        pickups: Vec::new(),
        pickup_respawn: None,
        weapon_effects: false,
    };
    let content = KartGatewayContent {
        schema_version: 1,
        details_loading: details(2),
        details_ready: details(4),
        vehicle_guid_offset: 0,
        race: Some(race),
        entry: RaceEntryContent {
            race_config: RaceConfig { words: vec![0; 21] },
            entry_vehicles: vec![ProxiedVehicle {
                body_asset: "test.cdt".into(),
                first_attribute: 400.,
                attribute_block: 123u64.to_le_bytes().to_vec(),
            }],
            client_data: vec![0; 85],
            player_count: 1,
            npc_count: 0,
            progress: vec![ProgressPayload {
                racer_guid: 123,
                slot: 0,
                unknown_0c_bits: 0,
                unknown_10_bits: 0,
                laps_completed: 0,
                local_racer: 0,
                lap_limit: 2,
                unknown_1d_bits: 0,
                best_lap_bits: 0,
                total_time_bits: 0,
                tail_0x29: 123,
                tail_0x31: 0,
                tail_0x39: 0,
                tail_0x41: 0,
            }],
        },
        context,
    };
    fs::write(
        scratch.0.join("kart_entry.json"),
        serde_json::to_vec(&content).unwrap(),
    )
    .unwrap();
    crate::kart_gateway::KartGateway::load(&scratch.0, None)
        .unwrap()
        .unwrap()
}

fn kart_register() -> Vec<u8> {
    let mut packet = vec![33, 0, 9, 0];
    packet.extend(1u64.to_le_bytes());
    packet.extend([0; 8]);
    packet
}

fn kart_go(encounter: u32, instance: u32) -> Vec<u8> {
    let mut packet = vec![41, 0, 108, 0];
    packet.extend(encounter.to_le_bytes());
    packet.extend(instance.to_le_bytes());
    packet.extend(1u64.to_le_bytes());
    packet
}

fn kart_gate(gate: f32) -> Vec<u8> {
    let mut packet = vec![33, 0, 7, 0];
    packet.extend(1u64.to_le_bytes());
    for value in [gate * 10.0, 0.0, 0.0, 1.0] {
        packet.extend(value.to_le_bytes());
    }
    packet
}

fn count(game: &Gameplay, expected: u32) {
    count_id(game, 1, expected);
}

fn count_id(game: &Gameplay, quest_id: u32, expected: u32) {
    let progress = game
        .npc_quests
        .as_ref()
        .expect("quest engine installed")
        .progress(quest_id)
        .expect("active quest exists");
    assert_eq!(progress.counts, vec![expected]);
}

fn select(game: &mut Gameplay, profile: i32) {
    let profiles = game.profiles.as_mut().expect("reference profiles");
    profiles.unlock(profile).expect("profile exists");
    profiles
        .select(profile, &game.inventory)
        .expect("profile can be selected");
}

fn station(game: &Gameplay, recipe_id: u32) -> CharacterState {
    let actor = game
        .opening
        .as_ref()
        .expect("reference opening")
        .actors()
        .iter()
        .find(|actor| {
            matches!(
                actor.action,
                Some(crate::opening::OpeningTrigger::Craft { recipe_id: id })
                    if id == recipe_id
            )
        })
        .expect("reference crafting station")
        .clone();
    CharacterState {
        guid: 1,
        zone_id: actor.zone_id,
        position: actor.position,
        rotation: [0., 0., 1.],
    }
}

fn fishing_request(sub: u16, body: &[u8]) -> Vec<u8> {
    let mut packet = vec![138, 0];
    packet.extend(sub.to_le_bytes());
    packet.extend(body);
    packet
}

fn fishing_actor_bytes() -> Vec<u8> {
    [
        FISHING_ACTOR.id.to_le_bytes(),
        FISHING_ACTOR.seq.to_le_bytes(),
    ]
    .concat()
}

fn fishing_register() -> Vec<u8> {
    let mut body = fishing_actor_bytes();
    body.push(0);
    fishing_request(2, &body)
}

fn fishing_cast() -> Vec<u8> {
    let mut body = fishing_actor_bytes();
    for coordinate in [10.0f32, 0.0, 0.0, 1.0] {
        body.extend(coordinate.to_le_bytes());
    }
    body.push(0);
    fishing_request(6, &body)
}

fn fishing_reel() -> Vec<u8> {
    let mut body = fishing_actor_bytes();
    body.push(1);
    fishing_request(7, &body)
}

fn reach_engine(position: [f32; 3], radius: f32) -> QuestEngine {
    let directory = std::env::temp_dir().join(format!(
        "osfr-t013-reach-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(directory.join("quests")).unwrap();
    fs::create_dir_all(directory.join("npcs")).unwrap();
    let source = json!([
        {"kind": "reconstructed", "evidence": "(R) T013 synthetic reach objective"}
    ]);
    fs::write(
        directory.join("quests/synthetic.json"),
        serde_json::to_vec(&json!({
            "schema_version": 1,
            "records": [{
                "id": 7001,
                "title_string_id": 1,
                "giver_npc_id": 700,
                "prerequisite_quest_ids": [],
                "objectives": [{
                    "kind": "reach_poi",
                    "id": 1,
                    "poi_id": 9,
                    "zone_id": 0,
                    "position": position,
                    "radius": radius,
                    "source": source.clone()
                }],
                "rewards": [],
                "source": source.clone()
            }]
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        directory.join("jobs.json"),
        serde_json::to_vec(&json!({
            "schema_version": 1,
            "records": [{
                "id": "synthetic",
                "name": "Synthetic",
                "profile_id": null,
                "category": "freestyle",
                "max_level": 20,
                "members_only": false,
                "trainer_npc_ids": [],
                "quest_ids": [7001],
                "abilities": [],
                "source": source
            }]
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        directory.join("npcs/synthetic.json"),
        serde_json::to_vec(&json!({
            "schema_version": 1,
            "records": [{
                "npc_id": 700,
                "quest_ids_given": [7001],
                "quests_given": [],
                "quests_accepted": [{"quest_id": 7001}]
            }]
        }))
        .unwrap(),
    )
    .unwrap();
    let mut engine =
        QuestEngine::from_split_content(&directory, 1, eligibility(), None).expect("reach quest");
    assert!(engine.offer(7001, 700).changed);
    assert!(engine.accept(7001).changed);
    let _ = fs::remove_dir_all(directory);
    engine
}

#[test]
fn collect_reward_grant_advances_the_active_collect_objective() {
    let mut game = game();
    install(&mut game, ObjectiveKind::CollectItem { item_id: 1195 }, 2);
    let mut out = Outcome::default();
    game.grant_reward(0, &[(1195, 1)], &mut out).unwrap();
    count(&game, 1);

    let before = game.state();
    assert!(game.grant_reward(0, &[(i32::MAX, 1)], &mut out).is_err());
    assert_eq!(game.state(), before);
    count(&game, 1);

    game.grant_reward(0, &[(1195, 1)], &mut out).unwrap();
    count(&game, 2);
}

#[test]
fn ordinary_harvest_nodes_advance_gather_twice_through_gameplay() {
    let (mut game, _scratch) = gathering_game(
        harvesting::CHEF_PROFILE_ID,
        4,
        definition(1, ObjectiveKind::GatherResource { resource_id: 2739 }, 2),
    );
    let (first_guid, first) = gather_node(&game, "Mushrooms", 0);
    game.tick(&first, 1_000).unwrap();
    let clicked = send(&mut game, &first, &node_click(first_guid), 1_100);
    assert!(clicked.changed);
    let first_id = game
        .harvesting
        .as_ref()
        .unwrap()
        .state()
        .active
        .as_ref()
        .expect("first harvest run")
        .instance_id;
    finish_type8(&mut game, &first, first_id, 4, 2_000);
    count(&game, 1);

    send(&mut game, &first, &[26, 0, 42, 0], 3_000);
    let (second_guid, second) = gather_node(&game, "Mushrooms", 1);
    send(&mut game, &second, &node_click(second_guid), 3_100);
    let second_id = game
        .harvesting
        .as_ref()
        .unwrap()
        .state()
        .active
        .as_ref()
        .expect("second harvest run")
        .instance_id;
    finish_type8(&mut game, &second, second_id, 4, 4_000);
    count(&game, 2);
}

#[test]
fn recipe_book_cooking_completes_twice_through_gameplay() {
    let mut game = game();
    let recipe_id = 70;
    install(&mut game, ObjectiveKind::CraftRecipe { recipe_id }, 2);
    select(&mut game, 45);
    let character = station(&game, 2001);
    for definition in [2862, 2875, 2884, 2900] {
        game.inventory.grant(0, definition, 8).unwrap();
    }

    for round in 0..2u64 {
        let mut start = vec![26, 0, 24, 0];
        start.extend(recipe_id.to_le_bytes());
        send(&mut game, &character, &start, 1_000 + round * 2_000);
        assert!(game.cooking.as_ref().unwrap().active());
        let instance = game
            .cooking
            .as_ref()
            .unwrap()
            .state()
            .active
            .as_ref()
            .unwrap()
            .instance_id;
        if round == 0 {
            let before = game.state();
            assert!(
                game.handle_in_world(
                    &crate::cooking::payload_packet(instance, "FRServer_GameWon\t900\t0").unwrap(),
                    &character,
                    1_010,
                )
                .is_err(),
                "an early win is refused"
            );
            assert_eq!(game.state(), before);
            count(&game, 0);
        }
        play_cooking(&mut game, &character, 2_000 + round * 2_000);
        assert_eq!(
            game.npc_quests
                .as_ref()
                .unwrap()
                .progress(1)
                .unwrap()
                .counts,
            vec![round as u32 + 1]
        );
        send(
            &mut game,
            &character,
            &[26, 0, 42, 0],
            3_000 + round * 2_000,
        );
    }
    count(&game, 2);
}

#[test]
fn ordinary_mining_node_advances_gather_and_minigame_through_gameplay() {
    let (mut game, _scratch) = gathering_game(
        crate::mining::MINER_PROFILE_ID,
        1,
        definition_many(
            1,
            vec![
                (1, ObjectiveKind::GatherResource { resource_id: 2739 }, 1),
                (2, ObjectiveKind::FinishMinigame { minigame_id: 18 }, 1),
            ],
            false,
        ),
    );
    let (node_guid, character) = gather_node(&game, "Copper Vein", 0);
    game.tick(&character, 1_000).unwrap();
    game.profiles
        .as_mut()
        .unwrap()
        .select(1, &game.inventory)
        .unwrap();
    let before = game.state();
    let refused = send(&mut game, &character, &node_click(node_guid), 1_100);
    assert!(
        refused
            .replies
            .iter()
            .any(|packet| { String::from_utf8_lossy(packet).contains("Switch to the Miner job") })
    );
    assert_eq!(game.state(), before);
    assert_eq!(
        game.npc_quests
            .as_ref()
            .unwrap()
            .progress(1)
            .unwrap()
            .counts,
        vec![0, 0]
    );

    select(&mut game, crate::mining::MINER_PROFILE_ID);
    send(&mut game, &character, &node_click(node_guid), 1_200);
    let instance = game
        .mining
        .as_ref()
        .unwrap()
        .state()
        .active
        .as_ref()
        .expect("mining run")
        .instance_id;
    finish_type8(&mut game, &character, instance, 1, 2_000);
    assert_eq!(
        game.npc_quests
            .as_ref()
            .unwrap()
            .progress(1)
            .unwrap()
            .counts,
        vec![1, 1]
    );
}

#[test]
fn kart_completion_reaches_quests_only_after_a_paid_claim() {
    let scratch = Scratch::new("kart");
    let character = kart_character();
    let mut kart = kart_gateway(&scratch);
    kart.begin(&character).unwrap();
    kart.set_previous_job(43);
    let mut game = game();
    select(&mut game, 48);
    install(
        &mut game,
        ObjectiveKind::FinishMinigame { minigame_id: 31 },
        1,
    );
    game.kart_entry = Some(kart);

    send(&mut game, &character, &kart_register(), 100_000);
    let active = game
        .kart_entry
        .as_ref()
        .unwrap()
        .state()
        .active
        .as_ref()
        .unwrap()
        .context
        .clone();
    send(
        &mut game,
        &character,
        &kart_go(active.encounter_id, active.instance_id),
        100_500,
    );
    let green = 106_500;
    game.tick(&character, green).unwrap();
    let mut now = green;
    for gate in [1., 2., 3., 4., 1., 2., 3.] {
        now += 1_000;
        send(&mut game, &character, &kart_gate(gate), now);
    }

    game.profiles
        .as_mut()
        .unwrap()
        .select(1, &game.inventory)
        .unwrap();
    let before_refusal = (
        game.inventory.state().clone(),
        game.inventory.state().coins,
        game.profiles.as_ref().unwrap().state().clone(),
        game.npc_quests.as_ref().unwrap().state().clone(),
    );
    now += 1_000;
    send(&mut game, &character, &kart_gate(4.0), now);
    assert_eq!(game.inventory.state().coins, 0, "no unpaid reward");
    assert_eq!(
        game.profiles
            .as_ref()
            .unwrap()
            .state()
            .experience
            .get(&48)
            .copied()
            .unwrap_or(0),
        0
    );
    assert_eq!(
        game.npc_quests
            .as_ref()
            .unwrap()
            .progress(1)
            .unwrap()
            .counts,
        vec![0],
        "a refused payout does not complete the quest"
    );
    assert_eq!(
        (
            game.inventory.state().clone(),
            game.inventory.state().coins,
            game.profiles.as_ref().unwrap().state().clone(),
            game.npc_quests.as_ref().unwrap().state().clone(),
        ),
        before_refusal
    );

    select(&mut game, 48);
    now += 1_000;
    send(&mut game, &character, &kart_gate(4.0), now);
    assert_eq!(game.inventory.state().coins, 40);
    assert_eq!(
        game.profiles
            .as_ref()
            .unwrap()
            .state()
            .experience
            .get(&48)
            .copied()
            .unwrap_or(0),
        60
    );
    count(&game, 1);

    let paid = (
        game.inventory.state().clone(),
        game.profiles.as_ref().unwrap().state().clone(),
        game.npc_quests.as_ref().unwrap().state().clone(),
    );
    now += 1_000;
    send(&mut game, &character, &kart_gate(4.0), now);
    assert_eq!(
        (
            game.inventory.state().clone(),
            game.profiles.as_ref().unwrap().state().clone(),
            game.npc_quests.as_ref().unwrap().state().clone(),
        ),
        paid,
        "a replayed finish neither pays nor advances the quest"
    );
    count(&game, 1);
}

#[test]
fn fishing_pays_both_objectives_twice_and_replay_changes_nothing() {
    let mut game = game();
    install_many(
        &mut game,
        definition_many(
            1,
            vec![
                (1, ObjectiveKind::CollectItem { item_id: 2884 }, 3),
                (2, ObjectiveKind::FinishMinigame { minigame_id: 595 }, 3),
            ],
            false,
        ),
    );
    let content: serde_json::Value =
        serde_json::from_str(crate::fishing_service::FishingService::content_json()).unwrap();
    for fish in content["species"].as_array().unwrap() {
        let key = fish["key"].as_str().unwrap();
        for grade in 1..=4 {
            game.fishing.bind_item_definition(key, grade, 2884).unwrap();
        }
    }
    let rewards = crate::rewards::RewardState::new(
        game.inventory.clone(),
        game.profiles.as_ref().unwrap().state().clone(),
        100,
    );
    game.fishing
        .begin(
            1,
            595,
            9001,
            "sacred_grove_shallows",
            1,
            crate::fishing::Vec4 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 1.0,
            },
            rewards,
        )
        .unwrap();
    let instance = game.fishing.instance_id(1).unwrap();
    let mut start = vec![39, 0, 5];
    start.extend(instance.to_le_bytes());
    start.extend((-1i32).to_le_bytes());
    start.extend((-1i32).to_le_bytes());

    let character = CharacterState {
        guid: 1,
        zone_id: 0,
        position: [0.; 3],
        rotation: [0., 0., 1.],
    };
    let started = send(&mut game, &character, &start, FISHING_T0);
    assert!(!started.replies.is_empty(), "the lifecycle start routes");
    send(&mut game, &character, &fishing_register(), FISHING_T0 + 1);

    let economy = |game: &Gameplay| {
        (
            game.inventory.state().clone(),
            game.inventory.state().coins,
            game.profiles.as_ref().unwrap().state().clone(),
            game.npc_quests.as_ref().unwrap().state().clone(),
        )
    };
    for caught in 1..=3u32 {
        let at = FISHING_T0 + u64::from(caught) * 10_000;
        send(&mut game, &character, &fishing_cast(), at + 10);
        game.tick(&character, at + 1_510).unwrap();
        game.tick(&character, at + 2_410).unwrap();
        let outcome = send(&mut game, &character, &fishing_reel(), at + 2_420);
        assert!(outcome.changed);
        assert_eq!(
            game.npc_quests
                .as_ref()
                .unwrap()
                .progress(1)
                .unwrap()
                .counts,
            vec![caught, caught]
        );
        if caught == 2 {
            let before = economy(&game);
            let replay = send(&mut game, &character, &fishing_reel(), at + 2_430);
            assert!(!replay.changed, "a replayed reel is not a catch");
            assert_eq!(economy(&game), before);
            assert_eq!(
                game.npc_quests
                    .as_ref()
                    .unwrap()
                    .progress(1)
                    .unwrap()
                    .counts,
                vec![2, 2],
                "the refusal leaves both live objectives unchanged"
            );
        }
    }
    assert_eq!(
        game.npc_quests
            .as_ref()
            .unwrap()
            .progress(1)
            .unwrap()
            .counts,
        vec![3, 3]
    );
}

#[test]
fn scene_entry_advances_the_active_area_objective() {
    let mut game = game();
    install(&mut game, ObjectiveKind::EnterArea { area_id: 0 }, 1);
    let character = CharacterState {
        guid: 1,
        zone_id: 0,
        position: [0.; 3],
        rotation: [0., 0., 1.],
    };
    game.initialize_world(&character, 1_000).unwrap();
    count(&game, 1);

    let mut refused = game_without_job(ObjectiveKind::EnterArea { area_id: 0 });
    let before = refused.npc_quests.as_ref().unwrap().state().clone();
    refused
        .on_location(
            &CharacterState {
                zone_id: 1,
                ..character
            },
            2_000,
        )
        .unwrap();
    assert_eq!(refused.npc_quests.as_ref().unwrap().state(), &before);
    count(&refused, 0);
}

#[test]
fn scene_entry_advances_the_active_reach_objective() {
    let mut game = game();
    game.npc_quests = Some(reach_engine([0.; 3], 2.0));
    let character = CharacterState {
        guid: 1,
        zone_id: 0,
        position: [1.; 3],
        rotation: [0., 0., 1.],
    };
    game.initialize_world(&character, 1_000).unwrap();
    count_id(&game, 7001, 1);

    let mut refused = game_without_job(ObjectiveKind::EnterArea { area_id: 0 });
    refused.npc_quests = Some(reach_engine([0.; 3], 2.0));
    let before = refused.npc_quests.as_ref().unwrap().state().clone();
    refused
        .on_location(
            &CharacterState {
                position: [10.; 3],
                ..character
            },
            2_000,
        )
        .unwrap();
    assert_eq!(refused.npc_quests.as_ref().unwrap().state(), &before);
    count_id(&refused, 7001, 0);
}

#[test]
fn scene_entry_advances_area_and_reach_without_opening_content() {
    let scratch = Scratch::copy_reference_without_opening();
    let mut game = Gameplay::load(&scratch.0, None).expect("package without opening.json");
    assert!(game.opening.is_none());
    install_many(
        &mut game,
        definition_many(
            1,
            vec![
                (1, ObjectiveKind::EnterArea { area_id: 0 }, 1),
                (
                    2,
                    ObjectiveKind::VisitPosition {
                        zone_id: 0,
                        position: [0.; 3],
                        radius: 2.,
                    },
                    1,
                ),
            ],
            false,
        ),
    );
    let character = CharacterState {
        guid: 1,
        zone_id: 0,
        position: [1.; 3],
        rotation: [0., 0., 1.],
    };
    game.initialize_world(&character, 1_000).unwrap();
    assert_eq!(
        game.npc_quests
            .as_ref()
            .unwrap()
            .progress(1)
            .unwrap()
            .counts,
        vec![1, 1]
    );
}

fn game_without_job(kind: ObjectiveKind) -> Gameplay {
    let mut game = game();
    install(&mut game, kind, 1);
    game
}
