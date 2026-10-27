//! Work item 065 (dungeon native replay) contract tests.
//!
//! Evidence boundary, read before the assertions:
//!
//! ORIGINAL (first-party / pinned-community sources; hashes pinned by
//! the private recovery checker `piece_065.py`):
//! * the encounter family base opcode 41 and the `EncounterDetailsResponsePacket`
//!   (114), `EncounterZoneIsReadyPacket` (107), `EncounterStatePacket` (106) and
//!   `EncounterPacketPlayerEnter` (2) writers are pinned community sources under
//!   `lab/references/Sanctuary-combat-pr-research` (PR 76);
//! * `EncounterPacketIsFighting` (133) and the combat packets
//!   `CombatPacketAttackProcessed` (32/7) and `CombatPacketEnableBossDisplay`
//!   (32/9) are pinned community sources from PR 76/77;
//! * the minigame family base opcode 39 carries the start ack (17), the won/lost
//!   result (18), the state removal (19) and the score rows (47);
//! * player health 38/1, enemy health 35/5, the NPC animation 35/8 and the
//!   standalone movement opcode 125 are the layouts already decoded by
//!   `lab/reports/verify-native-cave-win.py` and implemented by
//!   `src/encounter.rs`;
//! * the native close request is `41/109 [0, instance]`
//!   (`notes/Research/Native Combat Results Lifecycle.md`).
//!
//! RECONSTRUCTED (labelled, not historical): the transcript schema, the six gate
//! rules and the synthetic reference replay. No assertion here is native
//! acceptance; the preserved runs are evaluated as evidence only.
//!
//! The module is not registered in `src/lib.rs` (lead-owned). Including the exact
//! source file keeps this target compiling against the real artifact. Integration
//! step for the lead: register the module and drop this `#[path]` include.
#[path = "../src/restoration/piece_065.rs"]
#[allow(dead_code)]
mod piece_065;

use piece_065::{
    AcceptanceGate, DUNGEON_ENCOUNTER_ID, DUNGEON_ZONE_ID, Direction, EncounterPhase, GateOutcome,
    REPLAY_WIRE, ReplayEvent, ReplayRow, ReplayTranscript, SELECTION_ATTACK, SELECTION_EXIT,
    SELECTION_START, SessionEvidence, WORLD_ZONE_ID, evaluate, synthetic_reference_replay,
};

fn outcome(report: &piece_065::ReplayReport, gate: AcceptanceGate) -> GateOutcome {
    report.gate(gate).outcome.clone()
}

fn detail(report: &piece_065::ReplayReport, gate: AcceptanceGate) -> String {
    report.gate(gate).outcome.detail().to_string()
}

fn session(label: &str, fingerprint: &str, rewarded: &[u32]) -> SessionEvidence {
    SessionEvidence {
        label: label.to_string(),
        identity_fingerprint: fingerprint.to_string(),
        start_zone: WORLD_ZONE_ID,
        end_zone: WORLD_ZONE_ID,
        start_player_health: 1000,
        end_player_health: 1000,
        end_encounter_active: false,
        end_encounter_phase: None,
        rewarded: rewarded.to_vec(),
    }
}

fn single_session_replay() -> ReplayTranscript {
    let mut replay = synthetic_reference_replay();
    replay.sessions.truncate(1);
    replay
}

#[test]
fn replay_wire_rows_match_their_recovered_sources() {
    let row = |opcode: u16, subopcode: u16| {
        REPLAY_WIRE
            .iter()
            .find(|row| row.opcode == opcode && row.subopcode == subopcode)
            .unwrap_or_else(|| panic!("wire row {opcode}/{subopcode} missing"))
    };

    assert_eq!(row(41, 114).direction, Direction::ServerToClient);
    assert_eq!(row(41, 107).direction, Direction::ServerToClient);
    assert_eq!(row(41, 106).direction, Direction::ServerToClient);
    assert_eq!(row(41, 2).direction, Direction::ServerToClient);
    assert_eq!(row(41, 133).direction, Direction::ServerToClient);
    assert_eq!(row(41, 109).direction, Direction::ClientToServer);
    assert_eq!(row(39, 18).direction, Direction::ServerToClient);
    assert_eq!(row(39, 47).direction, Direction::ServerToClient);
    assert_eq!(row(32, 7).direction, Direction::ServerToClient);
    assert_eq!(row(32, 9).direction, Direction::ServerToClient);
    assert_eq!(row(35, 5).direction, Direction::ServerToClient);
    assert_eq!(row(125, 0).direction, Direction::ServerToClient);
    assert_eq!(row(38, 1).direction, Direction::ServerToClient);
    assert_eq!(row(31, 0).direction, Direction::ServerToClient);
    assert_eq!(row(10, 0).direction, Direction::ClientToServer);
    assert_eq!(row(13, 0).direction, Direction::ClientToServer);

    for entry in REPLAY_WIRE {
        assert!(
            !entry.source.trim().is_empty(),
            "{} carries no provenance source",
            entry.packet
        );
    }

    assert_eq!(DUNGEON_ENCOUNTER_ID, 411);
    assert_eq!(DUNGEON_ZONE_ID, 411);
    assert_eq!(SELECTION_EXIT, 101);
    assert_eq!(SELECTION_ATTACK, 103);
    assert_eq!(SELECTION_START, 104);
}

#[test]
fn synthetic_reference_replay_closes_every_gate() {
    let report = evaluate(&synthetic_reference_replay());
    assert!(report.accepted, "{:#?}", report.gates);
    assert_eq!(report.gates.len(), AcceptanceGate::ALL.len());
    for gate in AcceptanceGate::ALL {
        assert_eq!(
            outcome(&report, *gate),
            GateOutcome::Passed,
            "{}: {}",
            gate.label(),
            detail(&report, *gate)
        );
    }
}

#[test]
fn fresh_entry_requires_a_world_start_and_a_quiet_load_window() {
    let mut inside_cave = synthetic_reference_replay();
    inside_cave.sessions[0].start_zone = DUNGEON_ZONE_ID;
    let report = evaluate(&inside_cave);
    assert!(matches!(
        outcome(&report, AcceptanceGate::FreshEntry),
        GateOutcome::Failed(_)
    ));
    assert!(detail(&report, AcceptanceGate::FreshEntry).contains("already inside"));

    let mut damaged_before_start = synthetic_reference_replay();
    damaged_before_start.rows.insert(
        5,
        ReplayRow {
            frame: 55,
            event: ReplayEvent::PlayerHealth { hp: 940, max: 1000 },
        },
    );
    let report = evaluate(&damaged_before_start);
    assert!(matches!(
        outcome(&report, AcceptanceGate::FreshEntry),
        GateOutcome::Failed(_)
    ));
    assert!(detail(&report, AcceptanceGate::FreshEntry).contains("damage"));

    let mut enemy_moved_before_start = synthetic_reference_replay();
    enemy_moved_before_start.rows.insert(
        5,
        ReplayRow {
            frame: 56,
            event: ReplayEvent::EnemyMotion { guid: 2 },
        },
    );
    let report = evaluate(&enemy_moved_before_start);
    assert!(matches!(
        outcome(&report, AcceptanceGate::FreshEntry),
        GateOutcome::Failed(_)
    ));
    assert!(detail(&report, AcceptanceGate::FreshEntry).contains("moved before"));
}

#[test]
fn fresh_entry_is_not_evidenced_without_the_launch_sequence() {
    let mut replay = synthetic_reference_replay();
    replay
        .rows
        .retain(|row| !matches!(row.event, ReplayEvent::LaunchDetails { .. }));
    let report = evaluate(&replay);
    assert!(matches!(
        outcome(&report, AcceptanceGate::FreshEntry),
        GateOutcome::NotEvidenced(_)
    ));
    assert!(detail(&report, AcceptanceGate::FreshEntry).contains("41/114"));
}

#[test]
fn fresh_entry_rejects_damage_before_the_first_client_attack() {
    // The 073831 failure mode: no client start request, the enemies engage and the
    // player loses health before the first voluntary `26/10` attack.
    let mut replay = synthetic_reference_replay();
    replay.rows.retain(|row| {
        !matches!(row.event, ReplayEvent::ClientSelection { choice } if choice == SELECTION_START)
    });
    let trigger_frame = 70;
    replay.rows.push(ReplayRow {
        frame: trigger_frame,
        event: ReplayEvent::ClientSelection {
            choice: SELECTION_ATTACK,
        },
    });
    replay.rows.push(ReplayRow {
        frame: 65,
        event: ReplayEvent::EnemyMotion { guid: 2 },
    });
    replay.rows.push(ReplayRow {
        frame: 66,
        event: ReplayEvent::PlayerHealth { hp: 900, max: 1000 },
    });
    let report = evaluate(&replay);
    assert!(matches!(
        outcome(&report, AcceptanceGate::FreshEntry),
        GateOutcome::Failed(_)
    ));
    assert!(detail(&report, AcceptanceGate::FreshEntry).contains("damage"));
}

#[test]
fn an_earlier_loss_does_not_invalidate_the_winning_attempt() {
    // Runs with retries contain several `39/18` results; the gates grade the last one.
    let mut replay = synthetic_reference_replay();
    replay.rows.push(ReplayRow {
        frame: 73,
        event: ReplayEvent::Attack {
            attacker: 1,
            target: 2,
            damage: 50,
            hp: 100,
            max: 150,
        },
    });
    replay.rows.push(ReplayRow {
        frame: 74,
        event: ReplayEvent::PlayerHealth { hp: 0, max: 1000 },
    });
    replay.rows.push(ReplayRow {
        frame: 75,
        event: ReplayEvent::Result { won: false },
    });
    let report = evaluate(&replay);
    for gate in [
        AcceptanceGate::Combat,
        AcceptanceGate::Boss,
        AcceptanceGate::Rewards,
    ] {
        assert_eq!(
            outcome(&report, gate),
            GateOutcome::Passed,
            "{}: {}",
            gate.label(),
            detail(&report, gate)
        );
    }
}

#[test]
fn combat_gate_requires_player_damage_motion_and_survival() {
    let mut no_enemy_motion = synthetic_reference_replay();
    no_enemy_motion
        .rows
        .retain(|row| !matches!(row.event, ReplayEvent::EnemyMotion { .. }));
    let report = evaluate(&no_enemy_motion);
    assert!(matches!(
        outcome(&report, AcceptanceGate::Combat),
        GateOutcome::Failed(_)
    ));
    assert!(detail(&report, AcceptanceGate::Combat).contains("stationary"));

    let mut no_player_hit = synthetic_reference_replay();
    no_player_hit
        .rows
        .retain(|row| !matches!(row.event, ReplayEvent::Attack { .. }));
    let report = evaluate(&no_player_hit);
    assert!(matches!(
        outcome(&report, AcceptanceGate::Combat),
        GateOutcome::NotEvidenced(_)
    ));

    let mut defeated = synthetic_reference_replay();
    defeated.rows.push(ReplayRow {
        frame: 195,
        event: ReplayEvent::PlayerHealth { hp: 0, max: 1000 },
    });
    let report = evaluate(&defeated);
    assert!(matches!(
        outcome(&report, AcceptanceGate::Combat),
        GateOutcome::Failed(_)
    ));
    assert!(detail(&report, AcceptanceGate::Combat).contains("defeated"));
}

#[test]
fn boss_gate_requires_the_tallest_enemy_dead_and_displayed() {
    let mut alive = synthetic_reference_replay();
    alive.rows.push(ReplayRow {
        frame: 198,
        event: ReplayEvent::EnemyHealth {
            guid: 5,
            hp: 100,
            max: 500,
        },
    });
    let report = evaluate(&alive);
    assert!(matches!(
        outcome(&report, AcceptanceGate::Boss),
        GateOutcome::Failed(_)
    ));
    assert!(detail(&report, AcceptanceGate::Boss).contains("100 hp"));

    let mut no_display = synthetic_reference_replay();
    no_display
        .rows
        .retain(|row| !matches!(row.event, ReplayEvent::BossDisplay { .. }));
    let report = evaluate(&no_display);
    assert!(matches!(
        outcome(&report, AcceptanceGate::Boss),
        GateOutcome::NotEvidenced(_)
    ));
}

#[test]
fn rewards_gate_requires_a_win_real_defeat_counts_and_score_rows() {
    let mut loss = synthetic_reference_replay();
    for row in &mut loss.rows {
        if let ReplayEvent::Result { won } = &mut row.event {
            *won = false;
        }
    }
    let report = evaluate(&loss);
    assert!(matches!(
        outcome(&report, AcceptanceGate::Rewards),
        GateOutcome::Failed(_)
    ));
    assert!(detail(&report, AcceptanceGate::Rewards).contains("loss"));

    let mut no_scores = synthetic_reference_replay();
    no_scores
        .rows
        .retain(|row| !matches!(row.event, ReplayEvent::ScoreRows { .. }));
    let report = evaluate(&no_scores);
    assert!(matches!(
        outcome(&report, AcceptanceGate::Rewards),
        GateOutcome::Failed(_)
    ));
    assert!(detail(&report, AcceptanceGate::Rewards).contains("39/47"));

    let mut wrong_count = synthetic_reference_replay();
    for row in &mut wrong_count.rows {
        if let ReplayEvent::ScoreRows { defeated } = &mut row.event {
            *defeated = 1;
        }
    }
    let report = evaluate(&wrong_count);
    assert!(detail(&report, AcceptanceGate::Rewards).contains("defeats"));

    let mut unrewarded = synthetic_reference_replay();
    for session in &mut unrewarded.sessions {
        session.rewarded.clear();
    }
    let report = evaluate(&unrewarded);
    assert!(detail(&report, AcceptanceGate::Rewards).contains("reward"));
}

#[test]
fn exit_gate_requires_a_genuine_client_exit_then_the_world_return() {
    let mut no_exit = synthetic_reference_replay();
    no_exit.rows.retain(|row| {
        !matches!(
            row.event,
            ReplayEvent::ExitRequest { .. } | ReplayEvent::ClientSelection { .. }
        )
    });
    let report = evaluate(&no_exit);
    assert!(matches!(
        outcome(&report, AcceptanceGate::Exit),
        GateOutcome::Failed(_)
    ));
    assert!(detail(&report, AcceptanceGate::Exit).contains("close"));

    let mut no_return = synthetic_reference_replay();
    no_return.rows.retain(
        |row| !matches!(row.event, ReplayEvent::Zone { zone_id } if zone_id == WORLD_ZONE_ID),
    );
    let report = evaluate(&no_return);
    assert!(matches!(
        outcome(&report, AcceptanceGate::Exit),
        GateOutcome::Failed(_)
    ));
    assert!(detail(&report, AcceptanceGate::Exit).contains("world return"));
}

#[test]
fn reconnect_gate_requires_a_later_session_with_preserved_state() {
    let report = evaluate(&single_session_replay());
    assert!(matches!(
        outcome(&report, AcceptanceGate::Reconnect),
        GateOutcome::NotEvidenced(_)
    ));
    assert!(detail(&report, AcceptanceGate::Reconnect).contains("second session"));

    let mut repeated = synthetic_reference_replay();
    repeated
        .sessions
        .push(session("synthetic-reconnect-2", "reference", &[411, 411]));
    let report = evaluate(&repeated);
    assert!(matches!(
        outcome(&report, AcceptanceGate::Reconnect),
        GateOutcome::Failed(_)
    ));

    let mut other_character = synthetic_reference_replay();
    other_character
        .sessions
        .push(session("synthetic-reconnect-2", "someone-else", &[411]));
    let report = evaluate(&other_character);
    assert!(matches!(
        outcome(&report, AcceptanceGate::Reconnect),
        GateOutcome::Failed(_)
    ));
    assert!(detail(&report, AcceptanceGate::Reconnect).contains("identity"));

    let mut still_inside = synthetic_reference_replay();
    let mut later = session("synthetic-reconnect-2", "reference", &[411]);
    later.start_zone = DUNGEON_ZONE_ID;
    later.end_encounter_active = true;
    later.end_encounter_phase = Some(EncounterPhase::Fighting);
    still_inside.sessions.push(later);
    let report = evaluate(&still_inside);
    assert!(matches!(
        outcome(&report, AcceptanceGate::Reconnect),
        GateOutcome::Failed(_)
    ));

    // A transcript whose only rewarded session is the last one has no reconnect evidence,
    // even when an earlier session exists.
    let mut reward_only_later = synthetic_reference_replay();
    reward_only_later.sessions[0].rewarded.clear();
    reward_only_later.sessions.truncate(1);
    reward_only_later
        .sessions
        .push(session("synthetic-reconnect-2", "reference", &[411]));
    let report = evaluate(&reward_only_later);
    assert!(matches!(
        outcome(&report, AcceptanceGate::Reconnect),
        GateOutcome::NotEvidenced(_)
    ));
}

#[test]
fn preserved_native_runs_are_not_reported_as_native_acceptance() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab/reports");
    let root = root.as_path();
    let Ok(entries) = std::fs::read_dir(root) else {
        eprintln!("lab reports are absent; preserved-run grading skipped");
        return;
    };
    let mut newest: Option<(std::time::SystemTime, std::path::PathBuf)> = None;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("piece-065-") {
            continue;
        }
        let transcripts = entry.path().join("replay-transcripts");
        if !transcripts.is_dir() {
            continue;
        }
        let modified = entry
            .metadata()
            .and_then(|meta| meta.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        if newest.as_ref().is_none_or(|(time, _)| modified > *time) {
            newest = Some((modified, transcripts));
        }
    }
    let Some((_, transcripts)) = newest else {
        eprintln!("no piece-065 replay transcripts yet; preserved-run grading skipped");
        return;
    };

    let mut graded = 0;
    for entry in std::fs::read_dir(&transcripts)
        .expect("transcripts readable")
        .flatten()
    {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if path.extension().and_then(|value| value.to_str()) != Some("json")
            || name.ends_with(".facts.json")
        {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("transcript readable");
        let replay: ReplayTranscript = serde_json::from_str(&text)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let report = evaluate(&replay);
        println!(
            "{}: accepted={} entry={:?} combat={:?} boss={:?} rewards={:?} exit={:?} reconnect={:?}",
            replay.run,
            report.accepted,
            report.gate(AcceptanceGate::FreshEntry).outcome,
            report.gate(AcceptanceGate::Combat).outcome,
            report.gate(AcceptanceGate::Boss).outcome,
            report.gate(AcceptanceGate::Rewards).outcome,
            report.gate(AcceptanceGate::Exit).outcome,
            report.gate(AcceptanceGate::Reconnect).outcome,
        );
        // The preserved runs are historical evidence. None of them is a native
        // acceptance run: either it starts inside the cave, or it ends in a loss,
        // or it predates the score-row repair, or no second session was captured.
        assert!(
            !report.accepted,
            "{} unexpectedly satisfied every gate without lead confirmation",
            replay.run
        );
        graded += 1;
    }
    println!("graded {graded} preserved replay transcript(s)");
}
