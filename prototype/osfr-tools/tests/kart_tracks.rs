//! P-kart-racing: the four track packages beyond Wildwood (`data/content/kart_tracks/*.json`).
//!
//! Client facts: each package's gate curtains are the leaves of that track's own gates mesh
//! (`sh_kt_gates_bbec_004.cdt`, `sg_kt2_gates_bbec.cdt`, `bw_kt_gates_bbec_004.cdt`,
//! `ss_kt_gates_bbec.cdt`), placed as the zone's chunks place them; the road heights come from the
//! track collision meshes. Reconstructed (R): the start grid, the heading, the coverage lap rule, the
//! lap counts and the pickup spots (see each package's `source` and
//! `lab/tools/kart_track_build.py`). These tests load the real files, enter each track from its own
//! Games window row, gate and quest row, drive a coverage race to the finish, and check that the
//! offer, the scene frame and the grid pose follow the track.
use osfr_tools::{
    character_store::CharacterState,
    kart_gateway::{EntryDenied, KartGateway},
    kart_service::{GateRule, RacePhase},
    kart_tracks,
    vehicle_parts::{self, SLOTS},
};
use std::{collections::BTreeSet, path::PathBuf};

const PLAYER: u64 = 1;
/// (R) Laps per track: Frost Ridge five, Thunder Falls four, Tangletrack two, Sandstorm three.
const TRACKS: [(&str, u32, u32, u64); 4] = [
    ("sh_kt_track", 37, 5, 9500000759),
    ("sg_kt_track2", 567, 4, 9000000273),
    ("bw_kt_track", 38, 2, 9000000039),
    ("ss_kt_track", 578, 3, 9500000630),
];

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
    KartGateway::load(&content(), None)
        .expect("the shipped kart package loads")
        .expect("the shipped kart package is present")
}

fn gate_hit(position: [f32; 3]) -> Vec<u8> {
    let mut packet = vec![33, 0, 7, 0];
    packet.extend(PLAYER.to_le_bytes());
    for value in position.into_iter().chain([1.0]) {
        packet.extend(value.to_le_bytes());
    }
    packet
}

fn f32_at(bytes: &[u8], at: usize) -> f32 {
    f32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

fn phase(kart: &KartGateway) -> RacePhase {
    kart.state().race.as_ref().unwrap().phase
}

#[test]
fn every_track_package_loads_with_its_leaves_and_a_coverage_rule() {
    let tracks = kart_tracks::load(&content()).unwrap();
    let scenes: BTreeSet<&str> = tracks.keys().map(String::as_str).collect();
    assert_eq!(
        scenes,
        TRACKS
            .iter()
            .map(|(scene, ..)| *scene)
            .collect::<BTreeSet<_>>()
    );
    for (scene, _, laps, _) in TRACKS {
        let track = &tracks[scene];
        assert_eq!(track.scene.scene, scene);
        assert_eq!(track.race.laps, laps, "{scene} laps");
        assert_eq!(track.race.progress_template.lap_limit, laps as i32);
        assert!(track.race.gates.len() >= 16, "{scene} has a gate per leaf");
        assert_eq!(track.race.gate_curtains.len(), track.race.gates.len());
        assert!(!track.race.pickups.is_empty(), "{scene} has boxes");
        let Some(GateRule::Coverage {
            finish_gate,
            min_gates,
        }) = track.race.gate_rule
        else {
            panic!("{scene} races under the coverage rule until a lap is driven");
        };
        assert!((1..=track.race.gates.len() as u32).contains(&finish_gate));
        assert!(min_gates >= track.race.gates.len() as u32 / 2);
        // The grid is on the ground under the flags, not at the Wildwood grid.
        assert_ne!(track.race.start_position, [795.0, 86.190002, 365.0]);
        assert_eq!(
            &track.scene.position[..3],
            &track.race.start_position[..],
            "{scene} zones in at its grid"
        );
    }
}

/// Enters `row` (as the Games window would) and drives a coverage race at `speed` m/s: every leaf
/// but the finish in file order, then the finish, once per lap. Returns the gateway at the end.
fn drive(row: u32, speed: f32) -> KartGateway {
    let mut kart = shipped();
    kart.choose_row(&[], Some(row));
    kart.begin(&character()).unwrap();
    assert_eq!(kart.row(), Some(row));
    let scene = kart.scene().unwrap().scene.clone();
    let track = kart_tracks::load(&content())
        .unwrap()
        .remove(&scene)
        .unwrap();
    let race = &track.race;
    let Some(GateRule::Coverage { finish_gate, .. }) = race.gate_rule else {
        panic!("coverage rule");
    };
    let active = kart.state().active.clone().unwrap();
    let mut register = vec![33, 0, 9, 0];
    register.extend(PLAYER.to_le_bytes());
    register.extend([0; 8]);
    let replies = kart.handle(&register, 1_500).unwrap();
    // The 33/5 body puts the kart on this track's grid (basis then translation at 0x24).
    let body = replies
        .iter()
        .find(|reply| reply.starts_with(&[33, 0, 5, 0]))
        .expect("the register answer carries the 33/5 body");
    let payload = &body[4..];
    for axis in 0..3 {
        assert_eq!(
            f32_at(payload, 36 + axis * 4),
            race.start_position[axis],
            "{scene} grid axis {axis}"
        );
    }
    let mut go = vec![41, 0, 108, 0];
    go.extend(active.context.encounter_id.to_le_bytes());
    go.extend(active.context.instance_id.to_le_bytes());
    go.extend(PLAYER.to_le_bytes());
    kart.handle(&go, 2_000).unwrap();
    let mut now = 2_000 + race.schedule.last().unwrap().at_ms;
    kart.tick(now).unwrap();
    assert_eq!(
        phase(&kart),
        RacePhase::Racing,
        "{scene}: the light is green"
    );
    let mut last = race.start_position;
    // A racer drives the loop: the nearest unvisited leaf each time, the finish leaf last. (The
    // movement guard pays travel per claim, so a claim order that jumps across the track would
    // look like teleporting.)
    let distance = |a: [f32; 3], b: [f32; 3]| {
        a.iter()
            .zip(b)
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    };
    let mut order = Vec::new();
    let mut remaining: Vec<usize> = (0..race.gates.len())
        .filter(|index| *index as u32 + 1 != finish_gate)
        .collect();
    let mut at = race.start_position;
    while !remaining.is_empty() {
        let next = (0..remaining.len())
            .min_by(|a, b| {
                distance(at, race.gates[remaining[*a]].position)
                    .total_cmp(&distance(at, race.gates[remaining[*b]].position))
            })
            .unwrap();
        let index = remaining.swap_remove(next);
        at = race.gates[index].position;
        order.push(index);
    }
    order.push(finish_gate as usize - 1);
    for lap in 0..race.laps {
        for index in &order {
            let gate = &race.gates[*index];
            let meters = distance(last, gate.position);
            now += (meters / speed * 1_000.0) as u64 + 1;
            kart.tick(now).unwrap();
            kart.handle(&gate_hit(gate.position), now).unwrap();
            last = gate.position;
            if *index as u32 + 1 != finish_gate {
                assert!(
                    kart.state()
                        .race
                        .as_ref()
                        .unwrap()
                        .covered
                        .contains(&(*index as u32 + 1)),
                    "{scene}: a claim at leaf {} ({:?}) counts on lap {}",
                    index + 1,
                    gate.position,
                    lap + 1
                );
            }
        }
        let laps = kart.state().race.as_ref().unwrap().session.laps_completed;
        assert_eq!(
            laps,
            lap + 1,
            "{scene}: the finish leaf closes lap {}",
            lap + 1
        );
    }
    kart
}

#[test]
fn every_track_finishes_a_coverage_race_from_its_games_window_row() {
    for (scene, row, laps, _) in TRACKS {
        let kart = drive(row, 30.0);
        assert_eq!(phase(&kart), RacePhase::Finished, "{scene}");
        assert_eq!(kart.scene().unwrap().scene, scene);
        let session = &kart.state().race.as_ref().unwrap().session;
        assert_eq!(session.laps_completed, laps);
        assert!(kart.state().race.as_ref().unwrap().covered.is_empty());
    }
}

/// Crossing the finish leaf before the track is covered closes nothing, and a leaf counts once a
/// lap: the rule needs the drive, not a shortcut.
#[test]
fn the_finish_leaf_alone_does_not_close_a_lap() {
    let mut kart = shipped();
    kart.choose_row(&[], Some(37));
    kart.begin(&character()).unwrap();
    let track = kart_tracks::load(&content())
        .unwrap()
        .remove("sh_kt_track")
        .unwrap();
    let race = &track.race;
    let Some(GateRule::Coverage {
        finish_gate,
        min_gates,
    }) = race.gate_rule
    else {
        panic!("coverage rule");
    };
    let active = kart.state().active.clone().unwrap();
    let mut register = vec![33, 0, 9, 0];
    register.extend(PLAYER.to_le_bytes());
    register.extend([0; 8]);
    kart.handle(&register, 1_500).unwrap();
    let mut go = vec![41, 0, 108, 0];
    go.extend(active.context.encounter_id.to_le_bytes());
    go.extend(active.context.instance_id.to_le_bytes());
    go.extend(PLAYER.to_le_bytes());
    kart.handle(&go, 2_000).unwrap();
    let mut now = 2_000 + race.schedule.last().unwrap().at_ms;
    kart.tick(now).unwrap();
    let finish = race.gates[finish_gate as usize - 1].position;
    let other = race.gates[if finish_gate == 1 { 1 } else { 0 }].position;
    let laps = |kart: &KartGateway| kart.state().race.as_ref().unwrap().session.laps_completed;
    now += 30_000;
    kart.handle(&gate_hit(finish), now).unwrap();
    assert_eq!(laps(&kart), 0, "an uncovered finish is refused");
    for _ in 0..3 {
        now += 10_000;
        kart.handle(&gate_hit(other), now).unwrap();
    }
    let covered = &kart.state().race.as_ref().unwrap().covered;
    assert_eq!(covered.len(), 1, "a re-crossed leaf counts once");
    assert!((covered.len() as u32) < min_gates);
    now += 30_000;
    kart.handle(&gate_hit(finish), now).unwrap();
    assert_eq!(laps(&kart), 0);
}

/// Each speedway gate starts its own track (not Wildwood); the offer shows the row's own texts and
/// the track's spawn; a quest row on another track picks that track.
#[test]
fn gates_and_quest_rows_start_their_own_track_with_their_own_offer() {
    let rows = osfr_tools::kart_rows::KartRows::load(&content(), BTreeSet::new())
        .unwrap()
        .unwrap();
    for (scene, row, _, gate) in TRACKS {
        let mut kart = shipped();
        assert_eq!(kart.gate_row(gate), Some(row));
        assert_eq!(kart.admit_at(gate, false, 0), Ok(()));
        kart.choose_row(&[], kart.gate_row(gate));
        let (replies, zoning) = kart.begin(&character()).unwrap();
        assert_eq!(zoning.scene, scene);
        assert_eq!(kart.row(), Some(row));
        let texts = rows.texts(row).unwrap();
        let details = &replies[0];
        assert!(details.starts_with(&[41, 0, 114, 0]));
        for axis in 0..3 {
            assert_eq!(
                f32_at(details, 51 + axis * 4),
                zoning.position[axis],
                "{scene} spawn"
            );
        }
        let at = (0..details.len() - 15)
            .find(|at| u32_at(details, *at) == texts.name_id)
            .unwrap_or_else(|| panic!("{scene}: the offer carries name {}", texts.name_id));
        assert_eq!(
            [
                u32_at(details, at),
                u32_at(details, at + 4),
                u32_at(details, at + 8),
                u32_at(details, at + 12)
            ],
            [
                texts.name_id,
                texts.icon_id,
                texts.description_id,
                texts.difficulty
            ],
            "{scene}: title, picture, text, dots"
        );
        assert_eq!(
            u32_at(details, details.len() - 43),
            row,
            "{scene}: the row id"
        );
    }
    // The Wildwood gate and row keep the captured offer bytes but for the instance and session.
    let mut kart = shipped();
    let (replies, zoning) = kart.begin(&character()).unwrap();
    assert_eq!(zoning.scene, "sg_kt_track");
    assert_eq!(u32_at(&replies[0], replies[0].len() - 43), 28);
    assert!(replies[0].windows(4).any(|w| w == 17958u32.to_le_bytes()));
    // A Frost Ridge quest row chosen by an open objective races at Frost Ridge as that row.
    let mut kart = shipped();
    kart.choose_row(&[635], None);
    let (replies, zoning) = kart.begin(&character()).unwrap();
    assert_eq!(zoning.scene, "sh_kt_track");
    assert_eq!(kart.row(), Some(635));
    assert!(replies[0].windows(4).any(|w| w == 3322u32.to_le_bytes()));
    assert_eq!(u32_at(&replies[0], replies[0].len() - 43), 635);
}

/// A race on another track survives a save and reload with its scene and its progress.
#[test]
fn a_frost_ridge_race_restores_from_its_snapshot() {
    let kart = drive(37, 30.0);
    let saved = kart.state().clone();
    let restored = KartGateway::load(&content(), Some(saved)).unwrap().unwrap();
    assert_eq!(restored.scene().unwrap().scene, "sh_kt_track");
    assert_eq!(restored.row(), Some(37));
    assert_eq!(phase(&restored), RacePhase::Finished);
}

/// A gate bound to a row with no package refuses instead of starting another speedway's race.
#[test]
fn a_gate_whose_track_is_missing_refuses_to_race() {
    let dir = fixture("gate", false);
    let kart = KartGateway::load(&dir, None).unwrap().unwrap();
    assert_eq!(kart.scenes(), ["sg_kt_track"]);
    assert_eq!(kart.admit_at(9500001020, false, 0), Ok(()));
    assert_eq!(
        kart.admit_at(9500000759, false, 0),
        Err(EntryDenied::NoRace)
    );
    assert!(!kart.minigame_ids().contains(&37));
    std::fs::remove_dir_all(&dir).unwrap();
}

/// The shipped kart files (and, when asked, the track packages) in a scratch folder of their own.
fn fixture(name: &str, with_tracks: bool) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("kart-tracks-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for name in [
        "kart_entry.json",
        "kart_gates.json",
        "kart_rows.json",
        "job_xp_curve.json",
        "vehicles.json",
        "vehicle_looks.json",
        "kart_garage.json",
    ] {
        if content().join(name).is_file() {
            std::fs::copy(content().join(name), dir.join(name)).unwrap();
        }
    }
    if with_tracks {
        std::fs::create_dir_all(dir.join("kart_tracks")).unwrap();
        for entry in std::fs::read_dir(content().join("kart_tracks")).unwrap() {
            let path = entry.unwrap().path();
            std::fs::copy(
                &path,
                dir.join("kart_tracks").join(path.file_name().unwrap()),
            )
            .unwrap();
        }
    }
    dir
}

fn register() -> Vec<u8> {
    let mut register = vec![33, 0, 9, 0];
    register.extend(PLAYER.to_le_bytes());
    register.extend([0; 8]);
    register
}

/// A quest row stands in for a gate or Games row only on that speedway's track: with Wildwood
/// Sprint open, Mustang Alli's gate (row 37) still races Frost Ridge, a Frost Ridge quest row wins
/// there, the Wildwood row keeps the Sprint, and with no gate named the open objective leads.
#[test]
fn a_quest_row_stands_in_for_a_gate_only_on_its_own_track() {
    for (wanted, entered, row, scene) in [
        (&[631][..], Some(37), 37, "sh_kt_track"),
        (&[631, 635][..], Some(37), 635, "sh_kt_track"),
        (&[631][..], Some(28), 631, "sg_kt_track"),
        (&[635][..], None, 635, "sh_kt_track"),
        (&[631][..], Some(63), 631, "sg_kt_track"),
    ] {
        let mut kart = shipped();
        kart.choose_row(wanted, entered);
        let (_, zoning) = kart.begin(&character()).unwrap();
        assert_eq!(
            (kart.row(), zoning.scene.as_str()),
            (Some(row), scene),
            "wanted {wanted:?} entered {entered:?}"
        );
    }
}

/// The 33/11 grid pose is written after the owner guid, wherever the guid sits: a kart definition
/// of another length (the garage's looks) moves both together.
#[test]
fn the_grid_pose_follows_the_owner_guid_when_the_kart_block_grows() {
    let dir = fixture("longer-block", true);
    let path = dir.join("kart_entry.json");
    let mut doc: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let padding: Vec<serde_json::Value> = (0..16).map(|_| serde_json::json!(0)).collect();
    doc["entry"]["entry_vehicles"][0]["attribute_block"]
        .as_array_mut()
        .unwrap()
        .splice(0..0, padding);
    let guid_offset = doc["vehicle_guid_offset"].as_u64().unwrap() as usize + 16;
    doc["vehicle_guid_offset"] = serde_json::json!(guid_offset);
    std::fs::write(&path, serde_json::to_vec(&doc).unwrap()).unwrap();
    let mut kart = KartGateway::load(&dir, None).unwrap().unwrap();
    kart.choose_row(&[], Some(37));
    kart.begin(&character()).unwrap();
    let track = kart_tracks::load(&content())
        .unwrap()
        .remove("sh_kt_track")
        .unwrap();
    let replies = kart.handle(&register(), 1_500).unwrap();
    let (_, attributes) = created_kart(&replies);
    assert_pose_after_guid(&attributes, guid_offset, &track);
    std::fs::remove_dir_all(&dir).unwrap();
}

/// A save taken mid-race on a track whose package is gone still loads: the entry stays so the
/// login abandon returns the player, the race itself is dropped.
#[test]
fn a_saved_race_on_a_missing_track_loads_without_it_and_returns_the_player() {
    let dir = fixture("orphan", false);
    let mut kart = shipped();
    kart.choose_row(&[], Some(37));
    kart.begin(&character()).unwrap();
    kart.handle(&register(), 1_500).unwrap();
    let saved = kart.state().clone();
    assert!(saved.race.is_some());
    let mut loaded = KartGateway::load(&dir, Some(saved)).unwrap().unwrap();
    assert!(loaded.active());
    assert!(loaded.state().race.is_none());
    assert_eq!(loaded.scene().unwrap().scene, "sh_kt_track");
    assert_eq!(loaded.exit(), Some(character()));
    assert!(!loaded.active());
    std::fs::remove_dir_all(&dir).unwrap();
}

/// The kart created by the register answer: its body asset and the attribute block.
fn created_kart(replies: &[Vec<u8>]) -> (String, Vec<u8>) {
    let frame = replies
        .iter()
        .find(|reply| reply.starts_with(&[33, 0, 11, 0]))
        .expect("the register answer creates the kart");
    let name_len = u32_at(frame, 4) as usize;
    let name = String::from_utf8(frame[8..8 + name_len].to_vec()).unwrap();
    (name, frame[8 + name_len + 4..].to_vec())
}

/// Asserts the kart's attribute block carries the owner guid at `guid_at` and this track's grid
/// pose right after it.
fn assert_pose_after_guid(block: &[u8], guid_at: usize, track: &kart_tracks::TrackPackage) {
    assert_eq!(
        u64::from_le_bytes(block[guid_at..guid_at + 8].try_into().unwrap()),
        PLAYER,
        "owner guid at {guid_at}"
    );
    let pose = guid_at + 8;
    for (index, value) in track.start_basis().iter().enumerate() {
        assert_eq!(f32_at(block, pose + index * 4), *value, "basis {index}");
    }
    for axis in 0..3 {
        assert_eq!(
            f32_at(block, pose + 36 + axis * 4),
            track.race.start_position[axis],
            "axis {axis}"
        );
    }
}

/// The garage's chosen look replaces the kart definition before the owner guid, so the guid and
/// the grid pose move by the look's length: a shipped shorter look (the Buccaneer Racer's
/// zipster) and a padded longer look of the race kart both carry the pose at `start + 8`.
#[test]
fn a_chosen_look_of_another_length_moves_the_grid_pose_with_the_guid() {
    let dir = fixture("looks", true);
    let path = dir.join("vehicle_looks.json");
    let mut doc: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let padding: Vec<serde_json::Value> = (0..64).map(|_| serde_json::json!(0)).collect();
    doc["definitions"]["kt_kart08"]
        .as_array_mut()
        .unwrap()
        .extend(padding);
    std::fs::write(&path, serde_json::to_vec(&doc).unwrap()).unwrap();
    let track = kart_tracks::load(&content())
        .unwrap()
        .remove("sh_kt_track")
        .unwrap();
    let base = serde_json::from_slice::<serde_json::Value>(
        &std::fs::read(content().join("kart_entry.json")).unwrap(),
    )
    .unwrap()["vehicle_guid_offset"]
        .as_u64()
        .unwrap() as usize;
    // 56972 Buccaneer Racer wears the zipster look (shorter); 48308 The Volt Racer the padded one.
    for (item, longer) in [(56972, false), (48308, true)] {
        let mut kart = KartGateway::load(&dir, None).unwrap().unwrap();
        {
            let (table, loadouts) = kart.garage().unwrap();
            let vehicle = table.by_item(item).unwrap().clone();
            loadouts.add(&vehicle, [0; SLOTS]);
            let id = loadouts.cars.last().unwrap().id;
            assert!(loadouts.select(&table, id, 1));
        }
        let look = kart.chosen_look(1).expect("the chosen kart has a look");
        let (name, _, block) = vehicle_parts::split_definition(&look).unwrap();
        assert_eq!(block.len() > base, longer, "{item}: the look's length");
        kart.choose_row(&[], Some(37));
        kart.begin(&character()).unwrap();
        let replies = kart.handle(&register(), 1_500).unwrap();
        let (created, attributes) = created_kart(&replies);
        assert_eq!(created, name, "{item}: the chosen body rides");
        assert_pose_after_guid(&attributes, block.len(), &track);
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

/// Distance from a point to a vertical leaf (ring order: two low corners, two high).
fn leaf_distance(point: [f32; 3], leaf: &[[f32; 3]; 4]) -> f32 {
    let (a, b) = (leaf[0], leaf[1]);
    let (ux, uz) = (b[0] - a[0], b[2] - a[2]);
    let length = (ux * ux + uz * uz).sqrt();
    let (ux, uz) = (ux / length, uz / length);
    let (rx, rz) = (point[0] - a[0], point[2] - a[2]);
    let along = rx * ux + rz * uz;
    let across = rx * -uz + rz * ux;
    let low = leaf.iter().map(|v| v[1]).fold(f32::INFINITY, f32::min);
    let high = leaf.iter().map(|v| v[1]).fold(f32::NEG_INFINITY, f32::max);
    let out_along = (-along).max(along - length).max(0.0);
    let out_up = (low - point[1]).max(point[1] - high).max(0.0);
    (out_along * out_along + out_up * out_up + across * across).sqrt()
}

/// The coverage rule can only close a lap if enough leaves can be crossed from the ground: every
/// track has at least `min_gates` leaves whose own gate position lies inside the leaf's volume.
#[test]
fn every_track_has_at_least_min_gates_coverable_leaves() {
    for (scene, track) in kart_tracks::load(&content()).unwrap() {
        let race = &track.race;
        let Some(GateRule::Coverage { min_gates, .. }) = race.gate_rule else {
            panic!("{scene}: coverage rule");
        };
        let coverable = race
            .gates
            .iter()
            .zip(&race.gate_curtains)
            .filter(|(gate, leaves)| {
                leaves
                    .iter()
                    .any(|leaf| leaf_distance(gate.position, &leaf.vertices) <= leaf.half_depth)
            })
            .count() as u32;
        assert!(
            coverable >= min_gates,
            "{scene}: {coverable} coverable leaves for a {min_gates}-leaf lap"
        );
        assert_eq!(
            coverable,
            race.gates.len() as u32,
            "{scene}: every leaf meets the ground"
        );
    }
}
