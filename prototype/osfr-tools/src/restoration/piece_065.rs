//! Work item 065: dungeon native replay acceptance gates.
//!
//! Items 059-064 own the dungeon definitions, lifecycle, waves/gates, boss behaviour,
//! party rewards and collision evidence. None of them answers item 065's acceptance
//! question: *did one original client run enter the cave from the world, fight the
//! waves, kill the boss, receive the reward once, exit and reconnect?* This module is
//! that gate. It cannot drive the client, so it does the honest thing instead: it
//! grades a decoded replay transcript against six explicit gates and refuses to pass a
//! gate on missing, contradictory or unexplained rows.
//!
//! ORIGINAL facts used here (provenance in
//! `notes/Research/Pieces/065 Dungeon native replay.md`, hashes re-derived by
//! the private recovery checker `piece_065.py`):
//!
//! * the encounter family is base opcode 41 with `EncounterDetailsResponsePacket` 114,
//!   `EncounterZoneIsReadyPacket` 107, `EncounterStatePacket` 106,
//!   `EncounterPacketPlayerEnter` 2 and `EncounterPacketIsFighting` 133;
//! * the minigame family is base opcode 39 with the start ack 17, the won/lost result
//!   18, the state removal 19 and the score rows 47;
//! * combat feedback is opcode 32 subtype 7 (`AttackProcessed`) and subtype 9
//!   (boss display enable); enemy health is opcode 35 subtype 5; player health is
//!   38/1; standalone movement is opcode 125; zoning is opcode 31 and the client
//!   load gates are `ClientIsReady` 13 and `ClientFinishedLoading` 10;
//! * the native close request is `41/109 [0, instance]` and the guide exit is the
//!   recovered `26/10` selection 101 (`notes/Research/Native Combat Results
//!   Lifecycle.md`, `src/encounter.rs`).
//!
//! RECONSTRUCTED here (labelled): the replay transcript schema, the six gate rules and
//! the synthetic reference replay. The gates describe what the acceptance run must
//! show; they are not recovered original dungeon scripting, balance or rewards.
//!
//! Deliberate boundaries:
//! * this module never sends a packet, opens a window or writes a save; it grades a
//!   transcript produced from a preserved capture;
//! * a fully passing transcript is *evidence for the lead*, not native acceptance;
//! * unknown dungeon scripting, original waves, balance and reward tables stay with
//!   items 059-063 exactly as their notes describe.
//!
//! ponytail: one transcript type, one evaluator, six rules. No packet writer, no
//! second state machine, no live process control.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// ORIGINAL: the recovered story link for Darkthorne's teleport (locale 5100532).
/// The numeric value 411 doubles as the locally assigned encounter/zone id.
pub const DUNGEON_ENCOUNTER_ID: u32 = 411;
/// LOCAL assignment: the reconstructed cave zone id carried in `31` zoning rows.
pub const DUNGEON_ZONE_ID: i32 = 411;
/// The world zone the client returns to after the encounter.
pub const WORLD_ZONE_ID: i32 = 0;

/// Reconstructed interface values mirrored from `src/encounter.rs`'s `26/10`
/// selection handling: 101 exit, 103 attack, 104 start. Retry is 102.
pub const SELECTION_EXIT: i32 = 101;
pub const SELECTION_ATTACK: i32 = 103;
pub const SELECTION_START: i32 = 104;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    ClientToServer,
    ServerToClient,
}

/// One recovered wire member the replay gates read.
///
/// `subopcode: 0` marks an opcode that carries no subopcode on the wire (125, 31 and
/// 10); the second `u16` of those payloads is ordinary data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplayWire {
    pub opcode: u16,
    pub subopcode: u16,
    pub packet: &'static str,
    pub direction: Direction,
    /// Which preserved artifact establishes the row. The Python checker re-derives
    /// every row from that artifact; a row without a source is a checker failure.
    pub source: &'static str,
}

/// ORIGINAL: every row here is either re-derived from a pinned community writer or
/// observed in a preserved native capture. the private recovery checker `piece_065.py` fails when
/// one of these drifts from its source.
pub const REPLAY_WIRE: &[ReplayWire] = &[
    ReplayWire {
        opcode: 41,
        subopcode: 114,
        packet: "EncounterDetailsResponsePacket",
        direction: Direction::ServerToClient,
        source: "sanctuary-combat-pr-76:src/Sanctuary.Packet/BaseEncounterPacket/EncounterDetailsResponsePacket.cs",
    },
    ReplayWire {
        opcode: 41,
        subopcode: 107,
        packet: "EncounterZoneIsReadyPacket",
        direction: Direction::ServerToClient,
        source: "sanctuary-combat-pr-76:src/Sanctuary.Packet/BaseEncounterPacket/EncounterZoneIsReadyPacket.cs",
    },
    ReplayWire {
        opcode: 41,
        subopcode: 106,
        packet: "EncounterStatePacket",
        direction: Direction::ServerToClient,
        source: "sanctuary-combat-pr-76:src/Sanctuary.Packet/BaseEncounterPacket/EncounterStatePacket.cs",
    },
    ReplayWire {
        opcode: 41,
        subopcode: 2,
        packet: "EncounterPacketPlayerEnter",
        direction: Direction::ServerToClient,
        source: "sanctuary-combat-pr-76:src/Sanctuary.Packet/BaseEncounterPacket/EncounterPacketPlayerEnter.cs",
    },
    ReplayWire {
        opcode: 41,
        subopcode: 133,
        packet: "EncounterPacketIsFighting",
        direction: Direction::ServerToClient,
        source: "sanctuary-combat-pr-76:src/Sanctuary.Packet/BaseEncounterPacket/EncounterPacketIsFighting.cs",
    },
    ReplayWire {
        opcode: 41,
        subopcode: 109,
        packet: "EncounterParticipantRequestExitPacket",
        direction: Direction::ClientToServer,
        source: "capture:client-baseline-20261008T154226Z",
    },
    ReplayWire {
        opcode: 39,
        subopcode: 17,
        packet: "MiniGameGameStartPacket",
        direction: Direction::ServerToClient,
        source: "sanctuary-combat-pr-76:src/Sanctuary.Packet/BaseMiniGamePacket/MiniGameGameStartPacket.cs",
    },
    ReplayWire {
        opcode: 39,
        subopcode: 18,
        packet: "MiniGameGameOverPacket",
        direction: Direction::ServerToClient,
        source: "sanctuary-combat-pr-76:src/Sanctuary.Packet/BaseMiniGamePacket/MiniGameGameOverPacket.cs",
    },
    ReplayWire {
        opcode: 39,
        subopcode: 19,
        packet: "MiniGameStateRemovePacket",
        direction: Direction::ServerToClient,
        source: "sanctuary-combat-pr-76:src/Sanctuary.Packet/BaseMiniGamePacket/MiniGameStateRemovePacket.cs",
    },
    ReplayWire {
        opcode: 39,
        subopcode: 47,
        packet: "MiniGameGameEndScorePacket",
        direction: Direction::ServerToClient,
        source: "sanctuary-combat-pr-76:src/Sanctuary.Packet/BaseMiniGamePacket/MiniGameLootWheelPackets.cs",
    },
    ReplayWire {
        opcode: 32,
        subopcode: 7,
        packet: "CombatPacketAttackProcessed",
        direction: Direction::ServerToClient,
        source: "sanctuary-combat-pr-77:src/Sanctuary.Packet/BaseCombatPacket/CombatPacketAttackProcessed.cs",
    },
    ReplayWire {
        opcode: 32,
        subopcode: 9,
        packet: "CombatPacketEnableBossDisplay",
        direction: Direction::ServerToClient,
        source: "sanctuary-combat-pr-77:src/Sanctuary.Packet/BaseCombatPacket/CombatPacketEnableBossDisplay.cs",
    },
    ReplayWire {
        opcode: 35,
        subopcode: 5,
        packet: "PlayerUpdatePacketUpdateHitpoints",
        direction: Direction::ServerToClient,
        source: "sanctuary-combat-pr-74:src/Sanctuary.Packet/BasePlayerUpdatePacket/PlayerUpdatePacketUpdateHitpoints.cs",
    },
    ReplayWire {
        opcode: 26,
        subopcode: 10,
        packet: "encounter interaction selection",
        direction: Direction::ClientToServer,
        source: "capture:client-baseline-20261008T081219Z",
    },
    ReplayWire {
        opcode: 125,
        subopcode: 0,
        packet: "standalone movement",
        direction: Direction::ServerToClient,
        source: "capture:client-baseline-20261008T081219Z",
    },
    ReplayWire {
        opcode: 38,
        subopcode: 1,
        packet: "player health",
        direction: Direction::ServerToClient,
        source: "capture:client-baseline-20261008T081219Z",
    },
    ReplayWire {
        opcode: 31,
        subopcode: 0,
        packet: "zone transition",
        direction: Direction::ServerToClient,
        source: "capture:client-baseline-20261008T073831Z",
    },
    ReplayWire {
        opcode: 10,
        subopcode: 0,
        packet: "ClientFinishedLoading",
        direction: Direction::ClientToServer,
        source: "sanctuary:src/Sanctuary.Packet/PacketClientFinishedLoading.cs",
    },
    ReplayWire {
        opcode: 13,
        subopcode: 0,
        packet: "ClientIsReady",
        direction: Direction::ClientToServer,
        source: "sanctuary:src/Sanctuary.Packet/PacketClientIsReady.cs",
    },
];

/// Encounter phase as persisted by the rebuilt gateway (`41/106` state 4/5/6 and the
/// terminal snapshots).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncounterPhase {
    Loading,
    Fighting,
    Won,
    Lost,
}

/// One decoded replay row the gates read.
///
/// These are the only facts a capture tool has to supply; every row maps to one
/// recovered packet or one persisted snapshot field.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ReplayEvent {
    /// `31` zone transition into `zone_id`.
    Zone { zone_id: i32 },
    /// `41/114` encounter details; the trailing launch flag creates the state.
    LaunchDetails { instance: i32 },
    /// `41/107` the client finished loading the encounter zone.
    ZoneReady { instance: i32 },
    /// `41/2` the player actor entered the encounter.
    PlayerEnter,
    /// `41/106` encounter state machine step.
    EncounterState { phase: EncounterPhase },
    /// `38/1`.
    PlayerHealth { hp: i32, max: i32 },
    /// `35/5`.
    EnemyHealth { guid: u64, hp: i32, max: i32 },
    /// `32/7` processed attack.
    Attack {
        attacker: u64,
        target: u64,
        damage: i32,
        hp: i32,
        max: i32,
    },
    /// `32/9` boss display enable/disable.
    BossDisplay { guid: u64, on: bool },
    /// `125` server movement row for an enemy guid.
    EnemyMotion { guid: u64 },
    /// `26/10` client interaction selection (101 exit, 103 attack, 104 start).
    ClientSelection { choice: i32 },
    /// Client `10` finished-loading gate.
    LoadGate,
    /// `39/17` start acknowledgement.
    Start,
    /// `39/18` terminal result.
    Result { won: bool },
    /// `39/47` score rows; `defeated` is the real `scoreEnemiesDefeated` value.
    ScoreRows { defeated: i32 },
    /// `39/19` state removal.
    Remove,
    /// `41/109` native close request.
    ExitRequest { instance: i32 },
}

/// One decoded row with its capture frame index.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReplayRow {
    pub frame: u64,
    pub event: ReplayEvent,
}

/// Snapshot boundaries of one client session (one run directory, or the reconnect
/// session the lead captures next).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionEvidence {
    pub label: String,
    pub identity_fingerprint: String,
    pub start_zone: i32,
    pub end_zone: i32,
    pub start_player_health: i32,
    pub end_player_health: i32,
    pub end_encounter_active: bool,
    pub end_encounter_phase: Option<EncounterPhase>,
    /// Encounter ids recorded in the session's rewarded set.
    pub rewarded: Vec<u32>,
}

/// A decoded native replay: the wire rows plus the session snapshots around them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReplayTranscript {
    pub run: String,
    pub player_guid: u64,
    pub dungeon_zone: i32,
    pub rows: Vec<ReplayRow>,
    pub sessions: Vec<SessionEvidence>,
}

/// The six acceptance verbs of item 065.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AcceptanceGate {
    FreshEntry,
    Combat,
    Boss,
    Rewards,
    Exit,
    Reconnect,
}

impl AcceptanceGate {
    pub const ALL: &'static [AcceptanceGate] = &[
        AcceptanceGate::FreshEntry,
        AcceptanceGate::Combat,
        AcceptanceGate::Boss,
        AcceptanceGate::Rewards,
        AcceptanceGate::Exit,
        AcceptanceGate::Reconnect,
    ];

    pub fn label(self) -> &'static str {
        match self {
            AcceptanceGate::FreshEntry => "fresh entry",
            AcceptanceGate::Combat => "combat",
            AcceptanceGate::Boss => "boss",
            AcceptanceGate::Rewards => "rewards",
            AcceptanceGate::Exit => "exit",
            AcceptanceGate::Reconnect => "reconnect",
        }
    }
}

/// Outcome of one gate over one transcript.
#[derive(Clone, Debug, PartialEq)]
pub enum GateOutcome {
    Passed,
    /// The capture contains the rows and they contradict the rule.
    Failed(String),
    /// The capture does not contain the rows this rule needs.
    NotEvidenced(String),
}

impl GateOutcome {
    pub fn passed(&self) -> bool {
        matches!(self, GateOutcome::Passed)
    }

    pub fn detail(&self) -> &str {
        match self {
            GateOutcome::Passed => "passed",
            GateOutcome::Failed(detail) | GateOutcome::NotEvidenced(detail) => detail,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GateReport {
    pub gate: AcceptanceGate,
    pub outcome: GateOutcome,
    /// Capture frame indices the rule read.
    pub evidence: Vec<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReplayReport {
    pub run: String,
    pub gates: Vec<GateReport>,
    pub accepted: bool,
}

impl ReplayReport {
    pub fn gate(&self, gate: AcceptanceGate) -> &GateReport {
        self.gates
            .iter()
            .find(|report| report.gate == gate)
            .expect("every gate is reported")
    }
}

fn passed(gate: AcceptanceGate, evidence: Vec<u64>) -> GateReport {
    GateReport {
        gate,
        outcome: GateOutcome::Passed,
        evidence,
    }
}

fn failed(gate: AcceptanceGate, detail: impl Into<String>, evidence: Vec<u64>) -> GateReport {
    GateReport {
        gate,
        outcome: GateOutcome::Failed(detail.into()),
        evidence,
    }
}

fn not_evidenced(
    gate: AcceptanceGate,
    detail: impl Into<String>,
    evidence: Vec<u64>,
) -> GateReport {
    GateReport {
        gate,
        outcome: GateOutcome::NotEvidenced(detail.into()),
        evidence,
    }
}

fn first_row(rows: &[ReplayRow], predicate: impl Fn(&ReplayEvent) -> bool) -> Option<&ReplayRow> {
    rows.iter().find(|row| predicate(&row.event))
}

fn first_after(
    rows: &[ReplayRow],
    frame: u64,
    predicate: impl Fn(&ReplayEvent) -> bool,
) -> Option<&ReplayRow> {
    rows.iter()
        .find(|row| row.frame > frame && predicate(&row.event))
}

/// Per-enemy `(max health, last health seen)` over the whole replay.
fn enemy_health(rows: &[ReplayRow]) -> Vec<(u64, i32, i32)> {
    let mut order: Vec<u64> = Vec::new();
    let mut max: std::collections::BTreeMap<u64, i32> = std::collections::BTreeMap::new();
    let mut last: std::collections::BTreeMap<u64, i32> = std::collections::BTreeMap::new();
    for row in rows {
        if let ReplayEvent::EnemyHealth {
            guid,
            hp,
            max: maximum,
        } = &row.event
        {
            let (guid, hp, maximum) = (*guid, *hp, *maximum);
            if !max.contains_key(&guid) {
                order.push(guid);
            }
            let entry = max.entry(guid).or_insert(maximum);
            *entry = (*entry).max(maximum);
            last.insert(guid, hp);
        }
    }
    order
        .into_iter()
        .filter_map(|guid| Some((guid, *max.get(&guid)?, *last.get(&guid)?)))
        .collect()
}

/// Last health reported for one enemy before `frame`.
fn health_before(rows: &[ReplayRow], guid: u64, frame: u64) -> Option<i32> {
    rows.iter()
        .filter(|row| row.frame < frame)
        .filter_map(|row| match &row.event {
            ReplayEvent::EnemyHealth { guid: g, hp, .. } if *g == guid => Some(*hp),
            _ => None,
        })
        .last()
}

/// The tallest enemy, when exactly one enemy carries the greatest maximum health.
fn boss_guid(enemies: &[(u64, i32, i32)]) -> Option<u64> {
    let tallest = enemies.iter().map(|(_, max, _)| *max).max()?;
    let mut candidates = enemies.iter().filter(|(_, max, _)| *max == tallest);
    let boss = candidates.next()?.0;
    if candidates.next().is_some() {
        return None;
    }
    Some(boss)
}

/// The terminal `39/18` result of the replay.
///
/// A reconstructed run may lose attempts before the winning one, so the gates grade the
/// *last* result, which is what the client left the encounter with.
fn terminal_result(replay: &ReplayTranscript) -> Option<&ReplayRow> {
    replay
        .rows
        .iter()
        .filter(|row| matches!(row.event, ReplayEvent::Result { .. }))
        .last()
}

fn terminal_frame(replay: &ReplayTranscript) -> Option<u64> {
    terminal_result(replay)
        .map(|row| row.frame)
        .or_else(|| replay.rows.last().map(|row| row.frame))
}

/// Gate 1: the client starts in the world, zones into the cave, launches the encounter
/// and neither side acts until the player's first voluntary input.
fn fresh_entry(replay: &ReplayTranscript) -> GateReport {
    let gate = AcceptanceGate::FreshEntry;
    let dungeon_zone = replay.dungeon_zone;
    match replay.sessions.first() {
        None => {
            return not_evidenced(
                gate,
                "no session snapshot accompanies the replay",
                Vec::new(),
            );
        }
        Some(session) if session.start_zone == dungeon_zone => {
            return failed(
                gate,
                format!("the run began already inside the dungeon (zone {dungeon_zone})"),
                Vec::new(),
            );
        }
        Some(_) => {}
    }
    // Recovered flow order in the preserved entry run (073831): the server zones the
    // client into the cave (31 at 105.344s), then sends the launch details (41/114 at
    // 106.75s), the zone-ready row (41/107) and the player enter (41/2).
    let Some(zone) = first_row(
        &replay.rows,
        |event| matches!(event, ReplayEvent::Zone { zone_id } if *zone_id == dungeon_zone),
    ) else {
        return not_evidenced(
            gate,
            format!("no 31 zone transition into the dungeon zone {dungeon_zone}"),
            Vec::new(),
        );
    };
    let Some(launch) = first_after(&replay.rows, zone.frame, |event| {
        matches!(event, ReplayEvent::LaunchDetails { .. })
    }) else {
        return not_evidenced(
            gate,
            "no 41/114 launch details row after the cave zoning",
            vec![zone.frame],
        );
    };
    let Some(ready) = first_after(&replay.rows, launch.frame, |event| {
        matches!(event, ReplayEvent::ZoneReady { .. })
    }) else {
        return not_evidenced(
            gate,
            "no 41/107 zone-ready row after the launch",
            vec![zone.frame, launch.frame],
        );
    };
    let Some(enter) = first_after(&replay.rows, ready.frame, |event| {
        matches!(event, ReplayEvent::PlayerEnter)
    }) else {
        return not_evidenced(
            gate,
            "no 41/2 player-enter row after the zoning",
            vec![launch.frame, ready.frame, zone.frame],
        );
    };
    // The voluntary trigger is a client `26/10` selection: the recovery prepended the
    // cave's preparation gate exactly for this reason. A server `39/17` ack is not
    // voluntary input.
    let trigger = first_after(&replay.rows, enter.frame, |event| {
        matches!(
            event,
            ReplayEvent::ClientSelection { choice }
                if *choice == SELECTION_START || *choice == SELECTION_ATTACK
        )
    });
    let Some(trigger) = trigger else {
        return failed(
            gate,
            "no voluntary start or attack followed the zone ready",
            vec![launch.frame, ready.frame, zone.frame, enter.frame],
        );
    };
    let window_start = ready.frame.max(enter.frame);
    if let Some(row) = first_after(
        &replay.rows,
        window_start,
        |event| matches!(event, ReplayEvent::PlayerHealth { hp, max } if hp < max),
    )
    .filter(|row| row.frame < trigger.frame)
    {
        return failed(
            gate,
            format!(
                "the player took damage before the first voluntary action (frame {})",
                row.frame
            ),
            vec![window_start, row.frame, trigger.frame],
        );
    }
    if let Some(row) = first_after(&replay.rows, window_start, |event| {
        matches!(event, ReplayEvent::EnemyMotion { .. })
    })
    .filter(|row| row.frame < trigger.frame)
    {
        return failed(
            gate,
            format!(
                "an enemy moved before the first voluntary action (frame {})",
                row.frame
            ),
            vec![window_start, row.frame, trigger.frame],
        );
    }
    passed(
        gate,
        vec![
            launch.frame,
            ready.frame,
            zone.frame,
            enter.frame,
            trigger.frame,
        ],
    )
}

/// Gate 2: the player's own attacks processed, an enemy actually moved, and the
/// player was still standing when the encounter ended.
fn combat(replay: &ReplayTranscript) -> GateReport {
    let gate = AcceptanceGate::Combat;
    let player = replay.player_guid;
    let attacks: Vec<&ReplayRow> = replay
        .rows
        .iter()
        .filter(|row| {
            matches!(row.event, ReplayEvent::Attack { attacker, damage, .. } if attacker == player && damage > 0)
        })
        .collect();
    if attacks.is_empty() {
        return not_evidenced(gate, "no processed player attack damage (32/7)", Vec::new());
    }
    let enemies = enemy_health(&replay.rows);
    if enemies.is_empty() {
        return not_evidenced(gate, "no enemy health rows (35/5)", Vec::new());
    }
    let damaged = enemies.iter().filter(|(_, max, last)| last < max).count();
    if damaged == 0 {
        return failed(
            gate,
            "player attacks never reduced any enemy health",
            attacks.iter().map(|row| row.frame).collect(),
        );
    }
    let Some(motion) = first_row(&replay.rows, |event| {
        matches!(event, ReplayEvent::EnemyMotion { .. })
    }) else {
        return failed(
            gate,
            "no enemy movement rows; the enemies would appear stationary",
            attacks.iter().map(|row| row.frame).collect(),
        );
    };
    let Some(terminal) = terminal_frame(replay) else {
        return not_evidenced(gate, "the replay has no rows", Vec::new());
    };
    let standing = replay
        .rows
        .iter()
        .filter(|row| row.frame < terminal)
        .filter_map(|row| match &row.event {
            ReplayEvent::PlayerHealth { hp, .. } => Some((row.frame, *hp)),
            _ => None,
        })
        .last();
    let Some((frame, hp)) = standing else {
        return not_evidenced(
            gate,
            "no player health rows (38/1) before the terminal result",
            Vec::new(),
        );
    };
    if hp <= 0 {
        return failed(
            gate,
            format!("the player was defeated before the encounter ended (frame {frame})"),
            vec![frame, terminal],
        );
    }
    passed(
        gate,
        attacks
            .iter()
            .map(|row| row.frame)
            .chain(std::iter::once(motion.frame))
            .chain(std::iter::once(frame))
            .collect(),
    )
}

/// Gate 3: the boss is displayed and the tallest enemy is dead by the terminal
/// result.
fn boss(replay: &ReplayTranscript) -> GateReport {
    let gate = AcceptanceGate::Boss;
    let enemies = enemy_health(&replay.rows);
    if enemies.is_empty() {
        return not_evidenced(gate, "no enemy health rows (35/5)", Vec::new());
    }
    let Some(boss) = boss_guid(&enemies) else {
        return failed(
            gate,
            "no single tallest enemy; the boss is ambiguous",
            Vec::new(),
        );
    };
    let Some(display) = first_row(
        &replay.rows,
        |event| matches!(event, ReplayEvent::BossDisplay { guid, on } if *guid == boss && *on),
    ) else {
        return not_evidenced(
            gate,
            "no 32/9 boss display row for the tallest enemy",
            Vec::new(),
        );
    };
    let Some(terminal) = terminal_frame(replay) else {
        return not_evidenced(gate, "the replay has no rows", Vec::new());
    };
    let Some(hp) = health_before(&replay.rows, boss, terminal) else {
        return not_evidenced(
            gate,
            "no enemy health row for the boss before the terminal result",
            vec![display.frame],
        );
    };
    if hp != 0 {
        return failed(
            gate,
            format!("the tallest enemy still had {hp} hp when the encounter ended"),
            vec![display.frame, terminal],
        );
    }
    passed(gate, vec![display.frame, terminal])
}

/// Gate 4: a win, every enemy dead, the real defeat counts in the score rows, and
/// the reward recorded exactly once.
fn rewards(replay: &ReplayTranscript) -> GateReport {
    let gate = AcceptanceGate::Rewards;
    let Some(result) = terminal_result(replay) else {
        return not_evidenced(gate, "no 39/18 terminal result row", Vec::new());
    };
    let ReplayEvent::Result { won } = &result.event else {
        unreachable!("the row matched the result pattern")
    };
    if !*won {
        return failed(
            gate,
            "the terminal 39/18 result was a loss, not a win",
            vec![result.frame],
        );
    }
    let enemies = enemy_health(&replay.rows);
    let alive: Vec<u64> = enemies
        .iter()
        .filter(|(guid, _, _)| health_before(&replay.rows, *guid, result.frame) != Some(0))
        .map(|(guid, _, _)| *guid)
        .collect();
    if !alive.is_empty() {
        return failed(
            gate,
            format!("{} enemies were still alive at the win", alive.len()),
            vec![result.frame],
        );
    }
    let Some(scores) = first_after(&replay.rows, result.frame, |event| {
        matches!(event, ReplayEvent::ScoreRows { .. })
    }) else {
        return failed(
            gate,
            "no 39/47 score rows; the native score page has no data",
            vec![result.frame],
        );
    };
    let ReplayEvent::ScoreRows { defeated } = &scores.event else {
        unreachable!("the row matched the score pattern")
    };
    if *defeated != enemies.len() as i32 {
        return failed(
            gate,
            format!(
                "39/47 reported {defeated} defeats, the capture shows {} defeated enemies",
                enemies.len()
            ),
            vec![result.frame, scores.frame],
        );
    }
    let Some(session) = replay.sessions.last() else {
        return not_evidenced(
            gate,
            "no session snapshot accompanies the replay",
            Vec::new(),
        );
    };
    if !session.rewarded.contains(&DUNGEON_ENCOUNTER_ID) {
        return failed(
            gate,
            format!("the encounter id {DUNGEON_ENCOUNTER_ID} is not in the persisted rewarded set"),
            vec![result.frame, scores.frame],
        );
    }
    passed(gate, vec![result.frame, scores.frame])
}

/// Gate 5: a genuine client close or guide exit after the results, then the world
/// return, with the persisted session ending outside the encounter.
fn exit(replay: &ReplayTranscript) -> GateReport {
    let gate = AcceptanceGate::Exit;
    let Some(terminal) = terminal_frame(replay) else {
        return not_evidenced(gate, "the replay has no rows", Vec::new());
    };
    let Some(close) = first_after(&replay.rows, terminal, |event| {
        matches!(event, ReplayEvent::ExitRequest { .. })
            || matches!(event, ReplayEvent::ClientSelection { choice } if *choice == SELECTION_EXIT)
    }) else {
        return failed(
            gate,
            "no native close request (41/109 or the guide exit) followed the results",
            vec![terminal],
        );
    };
    let Some(zone) = first_after(
        &replay.rows,
        close.frame,
        |event| matches!(event, ReplayEvent::Zone { zone_id } if *zone_id == WORLD_ZONE_ID),
    ) else {
        return failed(
            gate,
            "the native close was not followed by a world return zone transition",
            vec![terminal, close.frame],
        );
    };
    let Some(session) = replay.sessions.last() else {
        return not_evidenced(
            gate,
            "no session snapshot accompanies the replay",
            Vec::new(),
        );
    };
    if session.end_zone != WORLD_ZONE_ID || session.end_encounter_active {
        return failed(
            gate,
            format!(
                "the persisted session ended in zone {} with encounter_active={}",
                session.end_zone, session.end_encounter_active
            ),
            vec![terminal, close.frame, zone.frame],
        );
    }
    passed(gate, vec![terminal, close.frame, zone.frame])
}

/// Gate 6: a later session for the same character, outside the encounter, with the
/// reward set unchanged (a reconnect, not a replayed reward).
fn reconnect(replay: &ReplayTranscript) -> GateReport {
    let gate = AcceptanceGate::Reconnect;
    if replay.sessions.len() < 2 {
        return not_evidenced(
            gate,
            "no second session was captured after the return",
            Vec::new(),
        );
    }
    let Some(winner) = replay
        .sessions
        .iter()
        .position(|session| session.rewarded.contains(&DUNGEON_ENCOUNTER_ID))
    else {
        return not_evidenced(gate, "no session recorded the dungeon reward", Vec::new());
    };
    if winner + 1 >= replay.sessions.len() {
        return not_evidenced(
            gate,
            "the rewarded session is the last one; no reconnect followed it",
            Vec::new(),
        );
    }
    let winning = &replay.sessions[winner];
    for later in &replay.sessions[winner + 1..] {
        if later.identity_fingerprint != winning.identity_fingerprint {
            return failed(
                gate,
                "the later session belongs to a different identity fingerprint",
                Vec::new(),
            );
        }
        if later.rewarded != winning.rewarded {
            return failed(
                gate,
                format!(
                    "the rewarded set changed on reconnect: {:?} -> {:?}",
                    winning.rewarded, later.rewarded
                ),
                Vec::new(),
            );
        }
        if later.start_zone != WORLD_ZONE_ID {
            return failed(
                gate,
                format!(
                    "the later session started in zone {} instead of the world",
                    later.start_zone
                ),
                Vec::new(),
            );
        }
        if later.end_encounter_active {
            return failed(
                gate,
                "the encounter was still active after the reconnect".to_string(),
                Vec::new(),
            );
        }
    }
    passed(gate, Vec::new())
}

/// Grade one decoded replay against the six acceptance gates.
///
/// `replay.accepted` means the transcript contains evidence for every verb; it is not
/// native acceptance by itself. The lead confirms the capture origin before any
/// playable claim.
pub fn evaluate(replay: &ReplayTranscript) -> ReplayReport {
    // ponytail: the gates walk the rows in frame order, so grade a frame-sorted copy
    // instead of trusting every producer to emit sorted rows.
    let mut replay = replay.clone();
    replay.rows.sort_by_key(|row| row.frame);
    let replay = &replay;
    let gates = vec![
        fresh_entry(replay),
        combat(replay),
        boss(replay),
        rewards(replay),
        exit(replay),
        reconnect(replay),
    ];
    let accepted = gates.iter().all(|report| report.outcome.passed());
    ReplayReport {
        run: replay.run.clone(),
        gates,
        accepted,
    }
}

/// RECONSTRUCTED reference replay: the row sequence an acceptance run has to produce.
///
/// It is a fixture, not a capture. It exists so the gates are demonstrably reachable
/// and so a regression in any rule fails a test instead of silently passing.
pub fn synthetic_reference_replay() -> ReplayTranscript {
    let rows = vec![
        (
            10,
            ReplayEvent::Zone {
                zone_id: DUNGEON_ZONE_ID,
            },
        ),
        (20, ReplayEvent::LaunchDetails { instance: 1 }),
        (30, ReplayEvent::ZoneReady { instance: 1 }),
        (40, ReplayEvent::PlayerEnter),
        (50, ReplayEvent::LoadGate),
        (
            60,
            ReplayEvent::ClientSelection {
                choice: SELECTION_START,
            },
        ),
        (
            70,
            ReplayEvent::EncounterState {
                phase: EncounterPhase::Fighting,
            },
        ),
        (
            80,
            ReplayEvent::EnemyHealth {
                guid: 2,
                hp: 150,
                max: 150,
            },
        ),
        (
            82,
            ReplayEvent::EnemyHealth {
                guid: 3,
                hp: 150,
                max: 150,
            },
        ),
        (
            84,
            ReplayEvent::EnemyHealth {
                guid: 4,
                hp: 150,
                max: 150,
            },
        ),
        (
            86,
            ReplayEvent::EnemyHealth {
                guid: 5,
                hp: 500,
                max: 500,
            },
        ),
        (90, ReplayEvent::BossDisplay { guid: 5, on: true }),
        (100, ReplayEvent::EnemyMotion { guid: 2 }),
        (
            110,
            ReplayEvent::Attack {
                attacker: 1,
                target: 2,
                damage: 150,
                hp: 0,
                max: 150,
            },
        ),
        (
            120,
            ReplayEvent::EnemyHealth {
                guid: 2,
                hp: 0,
                max: 150,
            },
        ),
        (
            130,
            ReplayEvent::Attack {
                attacker: 1,
                target: 3,
                damage: 150,
                hp: 0,
                max: 150,
            },
        ),
        (
            140,
            ReplayEvent::EnemyHealth {
                guid: 3,
                hp: 0,
                max: 150,
            },
        ),
        (
            150,
            ReplayEvent::Attack {
                attacker: 1,
                target: 4,
                damage: 150,
                hp: 0,
                max: 150,
            },
        ),
        (
            160,
            ReplayEvent::EnemyHealth {
                guid: 4,
                hp: 0,
                max: 150,
            },
        ),
        (
            170,
            ReplayEvent::Attack {
                attacker: 1,
                target: 5,
                damage: 500,
                hp: 0,
                max: 500,
            },
        ),
        (
            180,
            ReplayEvent::EnemyHealth {
                guid: 5,
                hp: 0,
                max: 500,
            },
        ),
        (190, ReplayEvent::PlayerHealth { hp: 280, max: 1000 }),
        (200, ReplayEvent::Result { won: true }),
        (210, ReplayEvent::ScoreRows { defeated: 4 }),
        (220, ReplayEvent::Remove),
        (
            230,
            ReplayEvent::ClientSelection {
                choice: SELECTION_EXIT,
            },
        ),
        (
            240,
            ReplayEvent::Zone {
                zone_id: WORLD_ZONE_ID,
            },
        ),
        (
            250,
            ReplayEvent::PlayerHealth {
                hp: 1000,
                max: 1000,
            },
        ),
    ]
    .into_iter()
    .map(|(frame, event)| ReplayRow { frame, event })
    .collect();

    let session = |label: &str| SessionEvidence {
        label: label.to_string(),
        identity_fingerprint: "synthetic-reference".to_string(),
        start_zone: WORLD_ZONE_ID,
        end_zone: WORLD_ZONE_ID,
        start_player_health: 1000,
        end_player_health: 1000,
        end_encounter_active: false,
        end_encounter_phase: None,
        rewarded: vec![DUNGEON_ENCOUNTER_ID],
    };

    ReplayTranscript {
        run: "synthetic-reference".to_string(),
        player_guid: 1,
        dungeon_zone: DUNGEON_ZONE_ID,
        rows,
        sessions: vec![
            session("synthetic-reference"),
            session("synthetic-reference-reconnect"),
        ],
    }
}
