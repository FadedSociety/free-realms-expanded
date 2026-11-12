//! C6: the kart race has a field. Seven AI racers join every race on a track `kart_ai.json` names,
//! drive the track's gate loop at tuned paces, and the race reports a real place, so "win the race"
//! credits only a first place and "place higher than <rival>" only a finish ahead of that rival.
//!
//! Client facts (retail capture, `lab/reports/original-minigame-packets`): the AI racers' `35/2`
//! AddNpc, `33/11` vehicle, `33/5` pose (slot in bits 3-5 of the control byte) and `33/14` progress
//! (place in `i32 @0x08`, the four leaders in `u64 @0x29..`); the vehicle block ends with the racer
//! guid, the grid pose and the slot. Reconstructed (R): the AI line and paces. These tests drive the
//! real Wildwood and Frost Ridge packages from the real content directory.
use osfr_tools::{
    character_store::CharacterState,
    content::MinigameRequirement,
    kart_ai::{self, KartAi},
    inventory::Inventory,
    kart_gateway::{KartGateway, KartGatewayContent, KartRewardClaim},
    profiles::Profiles,
    kart_service::{GateRule, KartServiceContent, RacePhase},
    kart_tracks,
};
use std::{collections::BTreeSet, fs, path::PathBuf};

const PLAYER: u64 = 1;

fn content() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn character() -> CharacterState {
    CharacterState {
        guid: PLAYER,
        zone_id: 0,
        position: [10., 20., 30.],
        rotation: [0., 0., 1.],
    }
}

fn shipped() -> KartGateway {
    KartGateway::load(&content(), None).unwrap().unwrap()
}

fn gate_hit(position: [f32; 3]) -> Vec<u8> {
    let mut packet = vec![33, 0, 7, 0];
    packet.extend(PLAYER.to_le_bytes());
    for value in position.into_iter().chain([1.0]) {
        packet.extend(value.to_le_bytes());
    }
    packet
}

fn sub(frame: &[u8]) -> Option<u16> {
    (frame.len() >= 4 && frame[..2] == [33, 0]).then(|| u16::from_le_bytes([frame[2], frame[3]]))
}

fn f32_at(bytes: &[u8], at: usize) -> f32 {
    f32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

fn u64_at(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}

/// A decoded `33/14` (payload after the 4-byte inner header).
#[derive(Debug, Clone)]
struct Progress {
    guid: u64,
    place: i32,
    remaining: f32,
    clock: f32,
    laps: i32,
    local: bool,
    leaders: [u64; 4],
}

fn progress(frame: &[u8]) -> Progress {
    let p = &frame[4..];
    assert_eq!(p.len(), 73);
    Progress {
        guid: u64_at(p, 0),
        place: u32_at(p, 8) as i32,
        remaining: f32_at(p, 16),
        clock: f32_at(p, 29),
        laps: u32_at(p, 20) as i32,
        local: p[24] == 1,
        leaders: [u64_at(p, 41), u64_at(p, 49), u64_at(p, 57), u64_at(p, 65)],
    }
}

/// The package a row races on: the Wildwood race or a track's.
fn race_of(scene: &str) -> KartServiceContent {
    if scene == "sg_kt_track" {
        let cfg: KartGatewayContent =
            serde_json::from_slice(&fs::read(content().join("kart_entry.json")).unwrap()).unwrap();
        return cfg.race.unwrap();
    }
    kart_tracks::load(&content())
        .unwrap()
        .remove(scene)
        .unwrap()
        .race
}

struct Race {
    kart: KartGateway,
    register: Vec<Vec<u8>>,
    /// Everything the server sent from the green light on, in order.
    after_green: Vec<Vec<u8>>,
    race: KartServiceContent,
}

/// Enters `row`, registers, starts, lights green, then drives at `speed` m/s until the race ends
/// (gates in order on the ordered rule, nearest leaf first on a coverage track).
fn drive(row: u32, speed: f32) -> Race {
    let mut kart = shipped();
    kart.choose_row(&[], Some(row));
    kart.begin(&character()).unwrap();
    let scene = kart.scene().unwrap().scene.clone();
    let race = race_of(&scene);
    let active = kart.state().active.clone().unwrap();
    let mut register = vec![33, 0, 9, 0];
    register.extend(PLAYER.to_le_bytes());
    register.extend([0; 8]);
    let registered = kart.handle(&register, 1_500).unwrap();
    let mut go = vec![41, 0, 108, 0];
    go.extend(active.context.encounter_id.to_le_bytes());
    go.extend(active.context.instance_id.to_le_bytes());
    go.extend(PLAYER.to_le_bytes());
    // The client loads for a few seconds first: the schedule's states 4-6 and 33/4 go out then.
    let mut sent = kart.tick(5_000).unwrap();
    kart.handle(&go, 6_000).unwrap();
    let mut now = 6_000 + race.schedule.last().unwrap().at_ms;
    sent.extend(kart.tick(now).unwrap());
    let phase = |kart: &KartGateway| kart.state().race.as_ref().unwrap().phase;
    assert_eq!(phase(&kart), RacePhase::Racing, "{row}: green");
    let dist = |a: [f32; 3], b: [f32; 3]| {
        a.iter()
            .zip(b)
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    };
    let mut order: Vec<usize> = (0..race.gates.len()).collect();
    if let Some(GateRule::Coverage { finish_gate, .. }) = race.gate_rule {
        let mut remaining: Vec<usize> = order
            .iter()
            .copied()
            .filter(|i| *i as u32 + 1 != finish_gate)
            .collect();
        let mut at = race.start_position;
        order.clear();
        while !remaining.is_empty() {
            let next = (0..remaining.len())
                .min_by(|a, b| {
                    dist(at, race.gates[remaining[*a]].position)
                        .total_cmp(&dist(at, race.gates[remaining[*b]].position))
                })
                .unwrap();
            let index = remaining.swap_remove(next);
            at = race.gates[index].position;
            order.push(index);
        }
        order.push(finish_gate as usize - 1);
    }
    let mut last = race.start_position;
    'laps: for _lap in 0..race.laps {
        for index in &order {
            let gate = &race.gates[*index];
            // Tick the clock in 50 ms steps along the leg, as the gateway's tick does.
            let leg = (dist(last, gate.position) / speed * 1_000.0) as u64 + 1;
            let end = now + leg;
            while now + 50 < end {
                now += 50;
                sent.extend(kart.tick(now).unwrap());
            }
            now = end;
            sent.extend(kart.tick(now).unwrap());
            if phase(&kart) != RacePhase::Racing {
                break 'laps;
            }
            sent.extend(kart.handle(&gate_hit(gate.position), now).unwrap());
            last = gate.position;
            if phase(&kart) != RacePhase::Racing {
                break 'laps;
            }
        }
    }
    assert_eq!(phase(&kart), RacePhase::Finished, "row {row} at {speed} m/s");
    Race {
        kart,
        register: registered,
        after_green: sent,
        race,
    }
}

/// The 33/14 frames a finished race sent, in order.
fn finals(race: &Race) -> Vec<Progress> {
    let end = race
        .after_green
        .iter()
        .rposition(|f| sub(f) == Some(14) && progress(f).local)
        .expect("a final local 33/14");
    // The seven AI rows of the finish go out just before the local row.
    let rows: Vec<Progress> = race.after_green[end - 7..=end]
        .iter()
        .inspect(|f| assert_eq!(sub(f), Some(14)))
        .map(|f| progress(f))
        .collect();
    assert!(rows[..7].iter().all(|r| !r.local));
    rows
}

#[test]
fn seven_racers_join_the_grid_after_the_kart_and_the_client_waits_for_them() {
    let mut kart = shipped();
    kart.choose_row(&[], Some(28));
    kart.begin(&character()).unwrap();
    let mut register = vec![33, 0, 9, 0];
    register.extend(PLAYER.to_le_bytes());
    register.extend([0; 8]);
    let replies = kart.handle(&register, 1_500).unwrap();
    let vehicles: Vec<&Vec<u8>> = replies.iter().filter(|f| sub(f) == Some(11)).collect();
    assert_eq!(vehicles.len(), 8, "the kart and seven AI karts");
    let npcs = replies.iter().filter(|f| f.starts_with(&[35, 0, 2, 0])).count();
    assert_eq!(npcs, 7, "one AddNpc per AI driver");
    // The capture's order per racer: 35/2, 33/11, 33/5, 33/14.
    let first_npc = replies.iter().position(|f| f.starts_with(&[35, 0, 2, 0])).unwrap();
    for racer in 0..7 {
        let at = first_npc + racer * 4;
        assert!(replies[at].starts_with(&[35, 0, 2, 0]));
        assert_eq!(sub(&replies[at + 1]), Some(11));
        assert_eq!(sub(&replies[at + 2]), Some(5));
        assert_eq!(sub(&replies[at + 3]), Some(14));
    }
    let cfg = race_of("sg_kt_track");
    // The captured 35/2 bodies (frames 16794-17349), by slot: driver name id, model id and the grid.
    // Every other field is the captured template's.
    let roster = [
        (1u8, 19358, 871, [805.0, 365.0]),
        (2, 19359, 872, [785.0, 365.0]),
        (3, 19363, 891, [800.0, 390.0]),
        (4, 19361, 888, [790.0, 390.0]),
        (5, 19362, 888, [795.0, 415.0]),
        (6, 19364, 869, [805.0, 415.0]),
        (7, 19344, 891, [785.0, 415.0]),
    ];
    let drivers: Vec<&Vec<u8>> = replies.iter().filter(|f| f.starts_with(&[35, 0, 2, 0])).collect();
    for (frame, (slot, name, model, grid)) in drivers.iter().zip(roster) {
        assert_eq!(frame.len(), 334, "the captured body width");
        assert_eq!(u64_at(frame, 4), kart_ai::racer_guid(slot));
        assert_eq!(u32_at(frame, 12) as i32, name, "slot {slot} name");
        assert_eq!(u32_at(frame, 16) as i32, model, "slot {slot} model");
        assert_eq!(u32_at(frame, 21) as i32, 408_679, "chat foreground");
        assert_eq!(u32_at(frame, 25) as i32, 13_951_728, "chat background");
        assert_eq!(u32_at(frame, 29) as i32, 1, "chat size");
        let at = |i: usize| f32_at(frame, 37 + i * 4);
        assert_eq!([at(0), at(2)], grid, "slot {slot} stands on its captured grid spot");
        assert_eq!(at(1), cfg.start_position[1], "at the start height");
        assert_eq!([f32_at(frame, 53), f32_at(frame, 57), f32_at(frame, 61), f32_at(frame, 65)], [0.0, 0.0, 1.0, 0.0], "captured rotation");
    }
    let mut slots = BTreeSet::new();
    for vehicle in &vehicles[1..] {
        let payload = &vehicle[4..];
        let name = u32_at(payload, 0) as usize;
        let block = &payload[4 + name + 4..];
        let at = block.len() - 58;
        let guid = u64_at(block, at);
        let slot = block[block.len() - 1];
        assert_eq!(guid, kart_ai::racer_guid(slot), "the block's guid names its slot");
        assert_eq!(block[block.len() - 2], 0, "an AI racer is not the local one");
        assert!(slots.insert(slot));
        // The captured grid: ahead of the kart by 0, 25 or 50 m, 5 or 10 m across.
        let z = f32_at(block, at + 8 + 36 + 8);
        assert_eq!(f32_at(block, at + 8 + 36 + 4), cfg.start_position[1], "slot {slot} at the start height");
        let ahead = z - cfg.start_position[2];
        assert!(
            [0.0, 25.0, 50.0].iter().any(|a| (ahead - a).abs() < 0.01),
            "slot {slot} stands {ahead} m ahead"
        );
    }
    assert_eq!(slots, (1..=7).collect());
    // The register frames' racers are the AI's, with no duplicate guid.
    let guids: BTreeSet<u64> = replies
        .iter()
        .filter(|f| sub(f) == Some(14))
        .map(|f| progress(f).guid)
        .collect();
    assert_eq!(guids.len(), 8);
    // `33/4` names exactly the AI racers created, in the entry bundle and in the schedule.
    let mut join = vec![33, 0, 2, 0];
    join.extend(PLAYER.to_le_bytes());
    join.extend(2u32.to_le_bytes());
    kart.handle(&join, 1_600).unwrap();
    let ready = kart.race_ready().unwrap();
    let npc_count = ready.iter().find(|f| sub(f) == Some(4)).unwrap();
    assert_eq!(u32_at(npc_count, 4), 7);
}

#[test]
fn the_schedule_announces_the_seven_racers_not_zero() {
    let race = drive(28, 40.0);
    let counts: Vec<u32> = race
        .after_green
        .iter()
        .filter(|f| sub(f) == Some(4))
        .map(|f| u32_at(f, 4))
        .collect();
    // The schedule's 33/4 sits before the green light (sent on the way to it), so check the
    // shipped race config the gateway built: no step says zero.
    assert!(!counts.is_empty() && counts.iter().all(|c| *c == 7), "{counts:?}");
    let ready = race.register.iter().filter(|f| sub(f) == Some(4)).count();
    assert_eq!(ready, 0, "33/4 follows the client's join, not the register");
}

#[test]
fn poses_name_their_slot_and_the_board_orders_the_whole_field() {
    let race = drive(28, 40.0);
    let poses: Vec<&Vec<u8>> = race.after_green.iter().filter(|f| sub(f) == Some(5)).collect();
    let slots: BTreeSet<u8> = poses.iter().map(|f| (f[4 + 76] >> 3) & 7).collect();
    assert_eq!(slots, (1..=7).collect(), "every AI racer is posed");
    for pose in &poses {
        assert_eq!(pose.len(), 4 + 85);
        assert!((0..18).all(|i| f32_at(pose, 4 + i * 4).is_finite()));
        assert_eq!(u32_at(pose, 4 + 81), 1);
        assert_eq!(pose[4 + 76] & 0x40, 0x40, "moving");
    }
    // An AI racer moves along the loop: its first and last pose are far apart.
    let of_slot = |slot: u8| -> Vec<[f32; 3]> {
        poses
            .iter()
            .filter(|f| (f[4 + 76] >> 3) & 7 == slot)
            .map(|f| [f32_at(f, 4 + 36), f32_at(f, 4 + 40), f32_at(f, 4 + 44)])
            .collect()
    };
    let line = of_slot(6);
    let moved = line
        .first()
        .zip(line.last())
        .map(|(a, b)| ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt())
        .unwrap();
    assert!(moved > 100.0, "slot 6 drove {moved} m");
    // Boards: places are a set 0..8, led by the leaders list, with the AI rows' distance to run
    // shrinking.
    let boards: Vec<Progress> = race
        .after_green
        .iter()
        .filter(|f| sub(f) == Some(14))
        .map(|f| progress(f))
        .collect();
    assert!(boards.len() >= 16, "several board updates");
    assert!(boards.iter().any(|p| p.local), "the local row rides the board");
    let ai_rows: Vec<&Progress> = boards.iter().filter(|p| !p.local).collect();
    let first: Vec<&Progress> = ai_rows.iter().copied().take(7).collect();
    let places: BTreeSet<i32> = first.iter().map(|p| p.place).collect();
    assert_eq!(places.len(), 7, "seven distinct places among the AI");
    assert!(places.iter().all(|p| (0..8).contains(p)));
    for p in &first {
        assert!(!p.local);
        assert!(p.leaders.iter().all(|g| *g != 0));
    }
}

#[test]
fn a_fast_driver_wins_and_a_slow_one_comes_last_and_the_board_says_so() {
    for (speed, place) in [(40.0, 0), (20.0, 7)] {
        let race = drive(28, speed);
        let rows = finals(&race);
        let local = rows.last().unwrap();
        assert!(local.local);
        assert_eq!(local.place, place, "{speed} m/s");
        assert_eq!(local.remaining, 0.0);
        assert_eq!(local.laps, 3);
        assert_eq!(rows.len(), 8, "seven AI rows and the local row");
        let mut places: Vec<i32> = rows.iter().map(|r| r.place).collect();
        places.sort_unstable();
        assert_eq!(places, (0..8).collect::<Vec<_>>(), "{speed} m/s: every place once");
        for ai in &rows[..7] {
            assert_eq!((ai.laps, ai.remaining, ai.local), (3, 0.0, false));
            // `@0x1d` is the racer's lap clock, not the race clock: a last lap, not 150+ s.
            assert!((20.0..120.0).contains(&ai.clock), "lap clock {}", ai.clock);
        }
        // The leaders list names the first four places in order.
        let mut by_place: Vec<&Progress> = rows.iter().collect();
        by_place.sort_by_key(|r| r.place);
        let top: Vec<u64> = by_place.iter().take(4).map(|r| r.guid).collect();
        assert_eq!(local.leaders.to_vec(), top);
    }
}

#[test]
fn a_rival_row_names_its_rival_and_the_local_racer_is_placed_against_it() {
    let rival_slot = 4u8;
    let rival_guid = kart_ai::racer_guid(rival_slot);
    let verve = 101_902i32.to_le_bytes();
    for (speed, beats) in [(40.0, true), (20.0, false)] {
        let race = drive(628, speed);
        assert!(
            race.register
                .iter()
                .any(|f| f.starts_with(&[35, 0, 2, 0]) && f.windows(4).any(|w| w == verve)),
            "Verve McNichols is one of the drivers"
        );
        let rows = finals(&race);
        let local = rows.iter().find(|r| r.local).unwrap().place;
        let rival = rows.iter().find(|r| r.guid == rival_guid).unwrap().place;
        assert_eq!(local < rival, beats, "{speed} m/s: local {local}, rival {rival}");
    }
}

#[test]
fn a_frost_ridge_coverage_race_has_the_same_field_and_places() {
    let race = drive(37, 36.0);
    let rows = finals(&race);
    assert_eq!(rows.len(), 8);
    assert_eq!(rows.last().unwrap().laps, 5);
    let mut places: Vec<i32> = rows.iter().map(|r| r.place).collect();
    places.sort_unstable();
    assert_eq!(places, (0..8).collect::<Vec<_>>());
}

/// The quest side: the race result carries the place and, on a rival row, whether the rival was
/// beaten, and a `require` matches against both.
#[test]
fn the_result_credits_a_place_goal_only_for_a_place_that_meets_it() {
    let claim = |placement, beat_rival| KartRewardClaim {
        placement,
        beat_rival,
        elapsed_seconds: Some(120),
        ..Default::default()
    };
    let top3 = MinigameRequirement {
        max_placement: Some(3),
        ..Default::default()
    };
    for (place, credited) in [(1, true), (2, true), (3, true), (4, false), (8, false)] {
        assert_eq!(claim(Some(place), None).result().meets(&top3), credited, "place {place}");
    }
    assert!(
        !claim(None, None).result().meets(&top3),
        "a race with no field reports no place, so a place goal cannot be met"
    );
    let rival = MinigameRequirement {
        max_placement: Some(8),
        ..Default::default()
    };
    assert!(claim(Some(5), Some(true)).result().meets(&rival));
    assert!(!claim(Some(1), Some(false)).result().meets(&rival), "ahead of all but the rival");
}

/// The shipped quests: every race goal that names a place carries the bound, and every rival the
/// quests name is a rival of the row's field.
#[test]
fn the_shipped_place_goals_carry_a_placement_requirement() {
    let quests: serde_json::Value =
        serde_json::from_slice(&fs::read(content().join("quests/kart_driver.json")).unwrap()).unwrap();
    let ai: serde_json::Value =
        serde_json::from_slice(&fs::read(content().join("kart_ai.json")).unwrap()).unwrap();
    let rivals: BTreeSet<u64> = ai["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r.get("rival").is_some())
        .map(|r| r["game_id"].as_u64().unwrap())
        .collect();
    let wins: BTreeSet<u64> = [633, 640, 647, 654].into();
    let mut checked = BTreeSet::new();
    for record in quests["records"].as_array().unwrap() {
        for objective in record["objectives"].as_array().into_iter().flatten() {
            if objective["kind"] != "finish_minigame" {
                continue;
            }
            let row = objective["minigame_id"].as_u64().unwrap();
            let placement = objective["require"]["max_placement"].as_u64();
            if wins.contains(&row) {
                assert_eq!(placement, Some(1), "row {row}: win the race");
                checked.insert(row);
            } else if rivals.contains(&row) {
                assert_eq!(placement, Some(8), "row {row}: place higher than the rival");
                checked.insert(row);
            }
        }
    }
    // Every goal live today is gated; a later quest on 651 or 654 is covered by the loop above.
    for row in [628, 629, 630, 633, 635, 636, 637, 640, 642, 643, 644, 647, 649, 650] {
        assert!(checked.contains(&row), "row {row} has a gated goal");
    }
}

#[test]
fn every_track_with_a_pace_builds_a_field_and_the_rows_agree_with_the_rival_table() {
    let ai = KartAi::load(&content()).unwrap().expect("kart_ai.json ships");
    let tracks = kart_tracks::load(&content()).unwrap();
    for scene in ["sg_kt_track", "sh_kt_track", "sg_kt_track2", "bw_kt_track", "ss_kt_track"] {
        let (race, basis) = match tracks.get(scene) {
            Some(track) => (track.race.clone(), track.start_basis()),
            None => (race_of(scene), [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]),
        };
        let field = ai
            .field(None, scene, &race, basis)
            .unwrap_or_else(|| panic!("{scene} has a field"));
        assert_eq!(field.count(), 7);
        assert_eq!(field.intro_frames().len(), 28);
    }
    assert!(
        ai.field(None, "no_such_track", &race_of("sg_kt_track"), [1.0; 9]).is_none(),
        "a track the file does not name races solo"
    );
}

fn banks() -> (Inventory, Profiles) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/release");
    let profiles = Profiles::load(&dir, None).unwrap();
    let mut state = profiles.state().clone();
    state.active = 48;
    state.unlocked.insert(48);
    (
        Inventory::load(&dir, None).unwrap(),
        Profiles::load(&dir, Some(state)).unwrap(),
    )
}

/// Drives `row` and claims the paid finish the way the gameplay tick does.
fn claim(row: u32, speed: f32) -> KartRewardClaim {
    let mut race = drive(row, speed);
    let (mut inventory, mut profiles) = banks();
    let claim = race
        .kart
        .claim_reward_with_event(&mut inventory, &mut profiles, Default::default())
        .unwrap();
    assert_eq!(claim.minigame_id, Some(row), "the claim credits the row it ran as");
    claim
}

/// The link that credits a quest: a driven race's claim carries the place, and the result the quest
/// engine matches a `require` against meets "win" only for a first place.
#[test]
fn a_driven_races_claim_carries_the_place_a_win_goal_is_matched_against() {
    let win = MinigameRequirement {
        max_placement: Some(1),
        ..Default::default()
    };
    let (fast, slow) = (claim(633, 40.0), claim(633, 20.0));
    assert_eq!(fast.placement, Some(1));
    assert_eq!(slow.placement, Some(8));
    assert_eq!(fast.beat_rival, None, "a win row names no rival");
    assert!(fast.result().meets(&win));
    assert!(!slow.result().meets(&win), "last place does not win the race");
}

#[test]
fn a_rival_rows_claim_says_whether_the_rival_was_beaten() {
    let any_place = MinigameRequirement {
        max_placement: Some(8),
        ..Default::default()
    };
    let (fast, slow) = (claim(628, 40.0), claim(628, 20.0));
    assert_eq!(fast.beat_rival, Some(true));
    assert_eq!(slow.beat_rival, Some(false));
    assert!(fast.result().meets(&any_place));
    assert!(!slow.result().meets(&any_place), "behind Verve McNichols the goal stays open");
}
