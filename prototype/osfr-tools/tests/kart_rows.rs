//! L930: the shipped kart row table (`data/content/kart_rows.json`).
//!
//! Client facts: every row here is a TYPE 10 row of `Resources/MiniGameData.txt` (28 Wildwood Kart
//! Racing, 37 Frost Ridge, 38 Tangletrack, 567 Thunder Falls, 578 Sandstorm, 277 the Racing
//! Tutorial, 628-655 the speedway challenges). The shipped race package runs on one track scene
//! (`sg_kt_track`, Wildwood), so only the Wildwood rows (28 and 628-634) can run; the rest stay
//! listed and closed. Rules are reconstructed (R). The tests load the real files and drive the real
//! course; the per-row bookkeeping (choice, credit, save) is covered with synthetic packages in
//! `tests/kart_npc_entry.rs` and `src/gameplay/t013_objective_events.rs`.
use osfr_tools::{
    activity_portal::{self, Catalogue, Kind, Refusal, Viewer, game_row},
    character_store::CharacterState,
    kart_gateway::{KartGateway, KartGatewayContent},
    kart_service::RacePhase,
};
use std::{collections::BTreeSet, fs, path::PathBuf};

const PLAYER: u64 = 1;
/// Every Wildwood row that can run on the shipped track.
const WILDWOOD: [u32; 8] = [28, 628, 629, 630, 631, 632, 633, 634];
/// Every other row the client lists for the kart race.
const CLOSED: [u32; 5] = [37, 38, 277, 567, 578];

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
        .chain(&CLOSED)
        .copied()
        .chain(635..=655)
        .map(u64::from)
        .collect();
    assert_eq!(listed, expected, "28, 277, 37/38/567/578 and 628-655");
    for row in table["rows"].as_array().unwrap() {
        assert!(
            row["source"]
                .as_array()
                .is_some_and(|source| !source.is_empty()),
            "row {} carries its evidence",
            row["game_id"]
        );
    }
    // Only rows on the package's own track can run; the gates' row is one of them.
    let kart = shipped();
    assert_eq!(kart.minigame_ids(), WILDWOOD);
    assert_eq!(kart.minigame_id(), Some(28));
}

/// The four other speedways' Games window rows (63, 64, 228, 231) and the Racing Tutorial (1048)
/// stay closed: the client greys them with its own refusal, and a forged Play is refused too.
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
    let opened = catalogue.admit(62, viewer, false).unwrap();
    assert_eq!((opened.id, opened.kind), (28, Kind::Kart));
    for closed in [63, 64, 228, 231, 1048] {
        assert_eq!(
            catalogue.admit(closed, viewer, false),
            Err(Refusal::NotOpen),
            "Games window row {closed} has no track to run on"
        );
    }
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

/// The default race has no time limit; every Wildwood challenge ends unpaid when the driver is
/// too slow for it, and finishes for a fast one. The pace is far from every limit (40 m/s laps
/// the course in about 45 s, 20 m/s in about 90 s), so retuning a limit does not trip this.
#[test]
fn wildwood_rows_finish_for_a_fast_driver_and_the_challenges_end_for_a_slow_one() {
    for row in WILDWOOD {
        assert_eq!(drive(row, 40.0), RacePhase::Finished, "row {row}, fast");
    }
    assert_eq!(
        drive(28, 20.0),
        RacePhase::Finished,
        "the default race has no par"
    );
    for row in WILDWOOD.into_iter().filter(|row| *row != 28) {
        assert_eq!(drive(row, 20.0), RacePhase::Aborted, "row {row}, slow");
    }
}
