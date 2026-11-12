//! L930: the shipped kart row table (`data/content/kart_rows.json`).
//!
//! Client facts: every row here is a TYPE 10 row of `Resources/MiniGameData.txt` (28 Wildwood Kart
//! Racing, 37 Frost Ridge, 38 Tangletrack, 567 Thunder Falls, 578 Sandstorm, 277 the Racing
//! Tutorial, 628-655 the speedway challenges, 1103 Margin of Victory, 579/580 the Sandstorm
//! circuits). The shipped Wildwood package (`kart_entry.json`) and the four track packages
//! (`kart_tracks/*.json`, P-kart-racing) serve every speedway, so every row with a track runs;
//! the tutorials 277/278 stay listed and closed. Rules are reconstructed (R). The tests load the real
//! files and drive the real Wildwood course; the other tracks are driven in `tests/kart_tracks.rs`,
//! and the per-row bookkeeping (choice, credit, save) with synthetic packages in
//! `tests/kart_npc_entry.rs` and `src/gameplay/t013_objective_events.rs`.
use osfr_tools::{
    activity_portal::{self, Catalogue, Kind, Refusal, Viewer, game_row},
    character_store::CharacterState,
    kart_gateway::{KartGateway, KartGatewayContent},
    kart_service::RacePhase,
};
use std::{collections::BTreeSet, fs, path::PathBuf};

const PLAYER: u64 = 1;
/// Every Wildwood row that runs on the shipped package's track.
const WILDWOOD: [u32; 22] = [
    28, 628, 629, 630, 631, 632, 633, 634, 1103, // the free race and the Wildwood quest rows
    283, 284, 310, 561, 562, 565, 569, // C6: difficulty variants and copies (icon 9870 or the Wildwood text)
    1076, 1077, 1078, 1079, 1080, 1081, // C6: the Wildwood races of difficulty 1-4
];
/// The other speedways' free rows and the Sandstorm circuits (their quest rows are 635-655), and the
/// C6 difficulty variants the client's icons put on them.
const OTHER_TRACKS: [u32; 16] = [
    37, 38, 567, 578, 579, 580, 285, 286, 287, 288, 1105, 1106, 1107, 1108, 1110, 1112,
];
/// The tutorials and their copies: listed, no track recovered, closed.
const CLOSED: [u32; 6] = [277, 278, 279, 280, 281, 282];

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

#[test]
fn the_table_lists_every_client_kart_row_and_only_wildwood_runs() {
    let text = fs::read(content().join("kart_rows.json")).unwrap();
    let table: serde_json::Value = serde_json::from_slice(&text).unwrap();
    let listed: BTreeSet<u64> = table["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["game_id"].as_u64().unwrap())
        .collect();
    let expected: BTreeSet<u64> = WILDWOOD
        .iter()
        .chain(&OTHER_TRACKS)
        .chain(&CLOSED)
        .copied()
        .chain(635..=655)
        .map(u64::from)
        .collect();
    assert_eq!(
        listed, expected,
        "28, 277/278, 37/38/567/578/579/580, 628-655 and 1103"
    );
    for row in table["rows"].as_array().unwrap() {
        assert!(
            row["source"]
                .as_array()
                .is_some_and(|source| !source.is_empty()),
            "row {} carries its evidence",
            row["game_id"]
        );
    }
    // Every row with a served track can run (the tutorials have none); the gates' row is one of them.
    let kart = shipped();
    let mut runs: Vec<u32> = WILDWOOD
        .into_iter()
        .chain(OTHER_TRACKS)
        .chain(635..=655)
        .collect();
    runs.sort_unstable();
    assert_eq!(kart.minigame_ids(), runs);
    assert_eq!(kart.minigame_id(), Some(28));
    assert_eq!(
        kart.scenes(),
        [
            "sg_kt_track",
            "bw_kt_track",
            "sg_kt_track2",
            "sh_kt_track",
            "ss_kt_track"
        ]
    );
}

/// Every speedway's Games window row (62, 63, 64, 228, 231) opens its own track; the Racing
/// Tutorial (1048) stays closed: the client greys it with its own refusal, and a forged Play is
/// refused too.
#[test]
fn games_window_rows_open_only_for_tracks_that_run() {
    let live = activity_portal::load_live(&content()).unwrap();
    let games: Vec<_> = shipped()
        .minigame_ids()
        .into_iter()
        .map(|id| game_row(id, Kind::Kart))
        .collect();
    let catalogue = Catalogue::new(Vec::new())
        .unwrap()
        .with_live(live, None, games)
        .unwrap();
    let viewer = Viewer {
        combat_job: false,
        level: 1,
        crafts: [0; 3],
        postman: 0,
        jobs: &std::collections::BTreeSet::new(),
        learned: &std::collections::BTreeSet::new(),
    };
    for (games_row, row) in [(62, 28), (63, 37), (64, 38), (228, 567), (231, 578)] {
        let opened = catalogue.admit(games_row, viewer, false).unwrap();
        assert_eq!((opened.id, opened.kind), (row, Kind::Kart));
    }
    assert_eq!(
        catalogue.admit(1048, viewer, false),
        Err(Refusal::NotOpen),
        "the Racing Tutorial has no track to run on"
    );
}

fn gate_hit(position: [f32; 3]) -> Vec<u8> {
    let mut packet = vec![33, 0, 7, 0];
    packet.extend(PLAYER.to_le_bytes());
    for value in position.into_iter().chain([1.0]) {
        packet.extend(value.to_le_bytes());
    }
    packet
}

/// Enters `row` on the real course and drives the gates in order at `speed` m/s, ticking the
/// clock as the gateway does, until the race ends. Returns the phase it ended in.
fn drive(row: u32, speed: f32) -> RacePhase {
    let cfg: KartGatewayContent =
        serde_json::from_slice(&fs::read(content().join("kart_entry.json")).unwrap()).unwrap();
    let race = cfg.race.unwrap();
    let mut kart = shipped();
    kart.choose_row(&[row], None);
    kart.begin(&character()).unwrap();
    assert_eq!(kart.row(), Some(row));
    let active = kart.state().active.clone().unwrap();
    let mut register = vec![33, 0, 9, 0];
    register.extend(PLAYER.to_le_bytes());
    register.extend([0; 8]);
    assert!(!kart.handle(&register, 1_500).unwrap().is_empty());
    let mut go = vec![41, 0, 108, 0];
    go.extend(active.context.encounter_id.to_le_bytes());
    go.extend(active.context.instance_id.to_le_bytes());
    go.extend(PLAYER.to_le_bytes());
    kart.handle(&go, 2_000).unwrap();
    let mut now = 2_000 + race.schedule.last().unwrap().at_ms;
    kart.tick(now).unwrap();
    let phase = |kart: &KartGateway| kart.state().race.as_ref().unwrap().phase;
    assert_eq!(phase(&kart), RacePhase::Racing, "{row}: the light is green");
    let mut last = race.start_position;
    for gate in race.gates.iter().cycle().take(race.gates.len() * 4) {
        let meters = last
            .iter()
            .zip(gate.position)
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f32>();
        now += (meters.sqrt() / speed * 1_000.0) as u64 + 1;
        kart.tick(now).unwrap();
        if phase(&kart) != RacePhase::Racing {
            break;
        }
        kart.handle(&gate_hit(gate.position), now).unwrap();
        last = gate.position;
        if phase(&kart) != RacePhase::Racing {
            break;
        }
    }
    phase(&kart)
}

/// No Wildwood row cuts a race short any more (sheet C13): a challenge's clock is its quest goal,
/// so a slow driver (20 m/s, about 90 s a lap, past every Sprint and Endurance par) still finishes
/// and sees the board, and a fast one (40 m/s) does too. The 600 s race window alone ends a race.
#[test]
fn wildwood_rows_finish_for_a_fast_driver_and_the_challenges_end_for_a_slow_one() {
    for row in WILDWOOD {
        assert_eq!(drive(row, 40.0), RacePhase::Finished, "row {row}, fast");
        assert_eq!(drive(row, 20.0), RacePhase::Finished, "row {row}, slow");
    }
}
