//! Work item 030: house native acceptance replay, gates and evidence validation.
//!
//! The housing family already has a bounded state machine (item 006), an entrance router (022), a
//! permission layer (023), a client-request codec (025), a durable inventory transaction (026) and
//! fixture limits (027). None of them answers the acceptance question for item 030: *did a real
//! client enter, place, move, remove, visit and restart successfully?* This module is that gate. It
//! cannot open a window, so it does the next best thing that is honest:
//!
//! * it replays the acceptance scenario against the reconstructed state machine, writes a real
//!   snapshot and a real transcript, and reports one gate per acceptance verb;
//! * it evaluates the same six gates over a transcript captured from the running client, refusing
//!   to pass a gate on missing, contradictory or unexplained evidence;
//! * it never reports native acceptance itself. Only a transcript whose origin is
//!   [`EvidenceOrigin::NativeCapture`] can be evidence-complete, and the lead still confirms it.
//!
//! ORIGINAL facts used here:
//! * housing is the inner-opcode family of base opcode 127 with the members in [`ACCEPTANCE_WIRE`]
//!   (`lab/decompiled/server/LibSOE/SOE/BaseHousingPackets.cs`); the shipped client constructs the
//!   same numbers with `FUN_00aa4990(<opcode>)` immediately before the matching `::vftable`
//!   (`lab/decompiled/client/FreeRealms/pseudocode.c`, e.g. enter request 19 at 1386220, zone data
//!   45 at 1393137, fixture update 40 at 1390143, update fixture position 51 at 1386478, fixture
//!   remove 41 at 1386050 and instance data 28 at 1384498);
//! * `HousingPacketHousingError` (46) carries one code word and code 2 is the permission denial
//!   (`notes/Research/Pieces/023 House permissions.md`, client handler `FUN_00ab9080`);
//! * the acceptance lot is `Houses.json` row 1 (`lab/references/FreeRealms-Legacy/src/Resources/
//!   Houses.json`): property 1, zone 1, name id 6514, build area
//!   x 322.0..=443.3, y -100.0..=60.0, z 392.1..=477.8.
//!
//! RECONSTRUCTED here: the acceptance transcript schema, the grouping of request and response rows,
//! the gate rules, the local fixture cap of the replay lot and every error string. No native client
//! has produced or consumed any row; [`ACCEPTANCE_WIRE`] carries recovered opcodes, but the packet
//! bodies beyond the client-request codec in item 025 are still unrecovered, so a transcript row
//! carries only the decoded facts a capture tool can supply.
//!
//! Deliberate boundaries:
//! * no packet is written: item 025 owns the codec, and its `UNRECOVERED_OPCODES` gate must close
//!   before a placement reply can reach the client;
//! * no UI, session, gateway or save path outside the two owned files is touched;
//! * collision, snap, landmarks, upkeep, visitor caps, invitations and interactive furniture stay
//!   with items 006/023/026/027/029 exactly as their notes describe.
//!
//! ponytail: one transcript type, one table, one pass per gate. No packet writer, no second state
//! machine, no timer.

#![allow(dead_code)]

use std::{
    collections::BTreeMap,
    fmt::Write as _,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[path = "../housing.rs"]
#[allow(dead_code)]
mod housing;

// INTEGRATION: when the lead registers `pub mod housing;` in `src/lib.rs`, replace the `#[path]`
// include above with `use crate::housing;` and drop this re-export block. Until then the owned test
// target includes the module by path exactly as `tests/housing.rs` and `tests/piece_026.rs` do.
pub use housing::{
    FixtureItemLedger, FixtureTransform, Housing, HousingError, PERMISSION_CAN_EDIT,
    PropertyDefinition, PropertyGrant,
};

/// ORIGINAL: every housing packet is a base opcode 127 datagram with an inner opcode.
pub const BASE_OPCODE: u16 = 127;
/// ORIGINAL: `HousingPacketHousingError` body code for a permission denial.
pub const HOUSING_ERROR_PERMISSION_DENIED: u32 = 2;
/// ORIGINAL: `Houses.json` row 1, used as the acceptance lot. The build area is flattened
/// `[min_x, min_y, min_z, max_x, max_y, max_z]` exactly as item 006 stores it.
pub const ACCEPTANCE_PROPERTY_ID: i32 = 1;
pub const ACCEPTANCE_ZONE_ID: i32 = 1;
pub const ACCEPTANCE_NAME_ID: i32 = 6514;
pub const ACCEPTANCE_BUILD_AREA: [f32; 6] = [322.0, -100.0, 392.1, 443.3, 60.0, 477.8];
/// RECONSTRUCTED local cap. No recovered field carries the real fixture limit; the only recovered
/// fact is the `HousingMaxFixtures` string, so the replay lot uses a small local cap and labels it.
pub const LOCAL_ACCEPTANCE_MAX_FIXTURES: u32 = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WireDirection {
    ClientToServer,
    ServerToClient,
}

impl WireDirection {
    pub fn name(self) -> &'static str {
        match self {
            Self::ClientToServer => "client to server",
            Self::ServerToClient => "server to client",
        }
    }
}

/// One recovered member of the housing family that the acceptance gates read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WireMember {
    pub opcode: u16,
    pub packet: &'static str,
    pub direction: WireDirection,
    /// Which recovered artifact establishes the entry. The Python checker requires a non-empty
    /// source for every row and cross-checks the client-request directions against item 025.
    pub source: &'static str,
}

/// ORIGINAL: the acceptance-relevant housing members. The Python checker re-derives every row from
/// the supplied server enum and the shipped client's constructors and fails on any drift.
pub const ACCEPTANCE_WIRE: &[WireMember] = &[
    WireMember {
        opcode: 1,
        packet: "ClientHousingPacketPlaceFixtureRequest",
        direction: WireDirection::ClientToServer,
        source: "item 025 schema() client writer FUN_00abcb90",
    },
    WireMember {
        opcode: 2,
        packet: "ClientHousingPacketPlaceFixture",
        direction: WireDirection::ServerToClient,
        source: "item 006 placement flow answers opcode 1",
    },
    WireMember {
        opcode: 3,
        packet: "ClientHousingPacketPickupFixture",
        direction: WireDirection::ClientToServer,
        source: "item 025 schema() client writer FUN_00ac0510",
    },
    WireMember {
        opcode: 5,
        packet: "ClientHousingPacketSaveFixture",
        direction: WireDirection::ClientToServer,
        source: "item 025 schema() client writer FUN_00abf620",
    },
    WireMember {
        opcode: 6,
        packet: "ClientHousingPacketSetEditMode",
        direction: WireDirection::ClientToServer,
        source: "item 025 schema() client writer FUN_00abcb30",
    },
    WireMember {
        opcode: 19,
        packet: "ClientHousingPacketEnterRequest",
        direction: WireDirection::ClientToServer,
        source: "item 025 schema() client writer FUN_00abca80",
    },
    WireMember {
        opcode: 28,
        packet: "HousingPacketInstanceData",
        direction: WireDirection::ServerToClient,
        source: "community PlayerHousingInstanceData / item 006 snapshot",
    },
    WireMember {
        opcode: 40,
        packet: "HousingPacketFixtureUpdate",
        direction: WireDirection::ServerToClient,
        source: "item 006 / item 026 fixture record",
    },
    WireMember {
        opcode: 41,
        packet: "HousingPacketFixtureRemove",
        direction: WireDirection::ServerToClient,
        source: "item 006 pickup flow answers opcode 3",
    },
    WireMember {
        opcode: 45,
        packet: "HousingPacketZoneData",
        direction: WireDirection::ServerToClient,
        source: "item 022 entrance router zone data",
    },
    WireMember {
        opcode: 46,
        packet: "HousingPacketHousingError",
        direction: WireDirection::ServerToClient,
        source: "item 023 client handler FUN_00ab9080 code word",
    },
    WireMember {
        opcode: 51,
        packet: "HousingPacketUpdateFixturePosition",
        direction: WireDirection::ServerToClient,
        source: "client constructor FUN_00aa4990(0x33)",
    },
    WireMember {
        opcode: 53,
        packet: "HousingPacketFixturePlacementDenied",
        direction: WireDirection::ServerToClient,
        source: "client constructor FUN_00aa4990(0x35)",
    },
];

pub fn wire_member(opcode: u16) -> Option<&'static WireMember> {
    ACCEPTANCE_WIRE
        .iter()
        .find(|member| member.opcode == opcode)
}

/// The six verbs the acceptance run must demonstrate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AcceptanceVerb {
    Enter,
    Place,
    Move,
    Remove,
    Visit,
    Restart,
}

impl AcceptanceVerb {
    pub const ALL: [AcceptanceVerb; 6] = [
        AcceptanceVerb::Enter,
        AcceptanceVerb::Place,
        AcceptanceVerb::Move,
        AcceptanceVerb::Remove,
        AcceptanceVerb::Visit,
        AcceptanceVerb::Restart,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Enter => "enter",
            Self::Place => "place",
            Self::Move => "move",
            Self::Remove => "remove",
            Self::Visit => "visit",
            Self::Restart => "restart",
        }
    }

    /// RECONSTRUCTED: the recovered opcodes a native transcript draws on for this verb. The gate
    /// reads the rows it needs; the list documents the accepted surface, not a demand that every
    /// member appear.
    pub fn required_opcodes(self) -> &'static [u16] {
        match self {
            Self::Enter => &[19, 45],
            Self::Place => &[1, 2, 6],
            Self::Move => &[5, 40, 51],
            Self::Remove => &[3, 41],
            Self::Visit => &[19, 45, 46],
            Self::Restart => &[28],
        }
    }
}

/// Where a transcript came from. A local replay can never prove the client accepted anything.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceOrigin {
    LocalReplay,
    NativeCapture,
}

impl EvidenceOrigin {
    pub fn name(self) -> &'static str {
        match self {
            Self::LocalReplay => "local replay",
            Self::NativeCapture => "native capture",
        }
    }
}

/// ORIGINAL: one placed fixture in `HousingPacketInstanceData` (28).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureRow {
    pub fixture_guid: u64,
    pub position: [f32; 3],
}

/// One decoded fact a capture tool can supply. Every variant names the recovered opcode it belongs
/// to, so a mislabelled row is rejected instead of silently counted.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RowBody {
    EnterRequest {
        house_guid: u64,
    },
    ZoneData {
        house_guid: u64,
        zone_definition_id: i32,
    },
    SetEditMode {
        in_edit_mode: bool,
    },
    PlaceFixtureRequest {
        house_guid: u64,
    },
    PlaceFixture {
        house_guid: u64,
        fixture_guid: u64,
    },
    SaveFixtureRequest {
        house_guid: u64,
        fixture_guid: u64,
    },
    FixtureUpdate {
        fixture_guid: u64,
        position: Option<[f32; 3]>,
    },
    UpdateFixturePosition {
        fixture_guid: u64,
        position: [f32; 3],
    },
    PickupFixture {
        fixture_guid: u64,
    },
    FixtureRemove {
        fixture_guid: u64,
    },
    HousingError {
        code: u32,
    },
    InstanceData {
        house_guid: u64,
        fixtures: Vec<FixtureRow>,
    },
    /// Session marker, not a packet: the server process was stopped and started again. It carries
    /// opcode 0 because it has no inner opcode.
    ServerRestart,
}

impl RowBody {
    pub fn expected_opcode(&self) -> Option<u16> {
        match self {
            Self::EnterRequest { .. } => Some(19),
            Self::ZoneData { .. } => Some(45),
            Self::SetEditMode { .. } => Some(6),
            Self::PlaceFixtureRequest { .. } => Some(1),
            Self::PlaceFixture { .. } => Some(2),
            Self::SaveFixtureRequest { .. } => Some(5),
            Self::FixtureUpdate { .. } => Some(40),
            Self::UpdateFixturePosition { .. } => Some(51),
            Self::PickupFixture { .. } => Some(3),
            Self::FixtureRemove { .. } => Some(41),
            Self::HousingError { .. } => Some(46),
            Self::InstanceData { .. } => Some(28),
            Self::ServerRestart => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::EnterRequest { .. } => "enter request",
            Self::ZoneData { .. } => "zone data",
            Self::SetEditMode { .. } => "set edit mode",
            Self::PlaceFixtureRequest { .. } => "place fixture request",
            Self::PlaceFixture { .. } => "place fixture response",
            Self::SaveFixtureRequest { .. } => "save fixture request",
            Self::FixtureUpdate { .. } => "fixture update",
            Self::UpdateFixturePosition { .. } => "update fixture position",
            Self::PickupFixture { .. } => "pickup fixture",
            Self::FixtureRemove { .. } => "fixture remove",
            Self::HousingError { .. } => "housing error",
            Self::InstanceData { .. } => "instance data",
            Self::ServerRestart => "server restart marker",
        }
    }

    pub fn house_guid(&self) -> Option<u64> {
        match self {
            Self::EnterRequest { house_guid }
            | Self::ZoneData { house_guid, .. }
            | Self::PlaceFixtureRequest { house_guid }
            | Self::PlaceFixture { house_guid, .. }
            | Self::SaveFixtureRequest { house_guid, .. }
            | Self::InstanceData { house_guid, .. } => Some(*house_guid),
            _ => None,
        }
    }

    fn is_edit_request(&self) -> bool {
        matches!(
            self,
            Self::SetEditMode { .. }
                | Self::PlaceFixtureRequest { .. }
                | Self::SaveFixtureRequest { .. }
                | Self::PickupFixture { .. }
        )
    }
}

/// One row of a session transcript. `actor` is the character guid on the sending or receiving side.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionRow {
    pub seq: u32,
    pub actor: u64,
    pub opcode: u16,
    pub body: RowBody,
}

/// What the gates need to know about the lot under test.
#[derive(Clone, Debug, PartialEq)]
pub struct AcceptanceExpectation {
    pub house_guid: u64,
    pub owner_guid: u64,
    /// The guest the owner is expected to have admitted. `None` means no visitor evidence exists.
    pub permitted_guest_guid: Option<u64>,
    pub zone_definition_id: i32,
}

// A gate outcome is serialized for the lead's report; its `Unproven` reason borrows a static
// string, so only the writing direction is derived.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum GateStatus {
    Pass {
        evidence: String,
    },
    Fail {
        reason: String,
    },
    /// The transcript does not contain evidence the verb needs. Missing evidence is never a pass.
    Unproven {
        missing: &'static str,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GateOutcome {
    pub verb: AcceptanceVerb,
    pub status: GateStatus,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AcceptanceReport {
    pub origin: EvidenceOrigin,
    pub gates: Vec<GateOutcome>,
}

impl AcceptanceReport {
    pub fn gate(&self, verb: AcceptanceVerb) -> &GateOutcome {
        self.gates
            .iter()
            .find(|outcome| outcome.verb == verb)
            .expect("every verb receives an outcome")
    }

    pub fn is_evidence_complete(&self) -> bool {
        self.gates
            .iter()
            .all(|outcome| matches!(outcome.status, GateStatus::Pass { .. }))
    }

    /// The only route to a complete native claim: a captured session whose every gate passed. Even
    /// then this is evidence for the lead to confirm, not an acceptance granted by this module.
    pub fn native_evidence_complete(&self) -> bool {
        self.origin == EvidenceOrigin::NativeCapture && self.is_evidence_complete()
    }

    pub fn unproven(&self) -> Vec<AcceptanceVerb> {
        self.gates
            .iter()
            .filter(|outcome| matches!(outcome.status, GateStatus::Unproven { .. }))
            .map(|outcome| outcome.verb)
            .collect()
    }

    pub fn failures(&self) -> Vec<AcceptanceVerb> {
        self.gates
            .iter()
            .filter(|outcome| matches!(outcome.status, GateStatus::Fail { .. }))
            .map(|outcome| outcome.verb)
            .collect()
    }

    /// A compact text report for the lead's evidence directory.
    pub fn render(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "origin: {}", self.origin.name());
        for outcome in &self.gates {
            match &outcome.status {
                GateStatus::Pass { evidence } => {
                    let _ = writeln!(out, "{:<8} PASS     {evidence}", outcome.verb.name());
                }
                GateStatus::Fail { reason } => {
                    let _ = writeln!(out, "{:<8} FAIL     {reason}", outcome.verb.name());
                }
                GateStatus::Unproven { missing } => {
                    let _ = writeln!(out, "{:<8} UNPROVEN {missing}", outcome.verb.name());
                }
            }
        }
        let _ = writeln!(
            out,
            "native evidence complete: {}",
            self.native_evidence_complete()
        );
        out
    }
}

pub fn render_rows(rows: &[SessionRow]) -> String {
    let mut out = String::new();
    for row in rows {
        let _ = writeln!(
            out,
            "seq={:<4} actor={:<20} op={:<3} {}",
            row.seq,
            row.actor,
            row.opcode,
            row.body.name()
        );
    }
    out
}

/// A transcript that cannot be evaluated at all: a mislabelled opcode, an out-of-order row or
/// evidence for a different lot. These are rejected instead of scored.
#[derive(Clone, Debug, PartialEq)]
pub struct TranscriptError {
    pub index: usize,
    pub reason: String,
}

pub fn evaluate_transcript(
    origin: EvidenceOrigin,
    expectation: &AcceptanceExpectation,
    rows: &[SessionRow],
) -> Result<AcceptanceReport, TranscriptError> {
    validate_rows(expectation, rows)?;
    let gates = AcceptanceVerb::ALL
        .iter()
        .map(|verb| GateOutcome {
            verb: *verb,
            status: evaluate_gate(*verb, expectation, rows),
        })
        .collect();
    Ok(AcceptanceReport { origin, gates })
}

fn validate_rows(
    expectation: &AcceptanceExpectation,
    rows: &[SessionRow],
) -> Result<(), TranscriptError> {
    let mut previous_seq: Option<u32> = None;
    for (index, row) in rows.iter().enumerate() {
        if let Some(previous) = previous_seq {
            if row.seq <= previous {
                return Err(TranscriptError {
                    index,
                    reason: format!(
                        "rows must be in strictly increasing sequence order, but {} follows {}",
                        row.seq, previous
                    ),
                });
            }
        }
        previous_seq = Some(row.seq);

        match row.body.expected_opcode() {
            Some(expected) if expected != row.opcode => {
                return Err(TranscriptError {
                    index,
                    reason: format!(
                        "row {} declares opcode {} but its body is a {} (opcode {})",
                        row.seq,
                        row.opcode,
                        row.body.name(),
                        expected
                    ),
                });
            }
            None if row.opcode != 0 => {
                return Err(TranscriptError {
                    index,
                    reason: format!(
                        "row {} is a {} and carries no inner opcode",
                        row.seq,
                        row.body.name()
                    ),
                });
            }
            _ => {}
        }

        if let Some(house_guid) = row.body.house_guid() {
            if house_guid != expectation.house_guid {
                return Err(TranscriptError {
                    index,
                    reason: format!(
                        "row {} carries house guid {} but the acceptance house is {}",
                        row.seq, house_guid, expectation.house_guid
                    ),
                });
            }
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Default)]
struct FixtureFacts {
    place_seq: Option<u32>,
    remove_seq: Option<u32>,
    /// Known positions in ascending sequence order, from fixture updates and position updates.
    /// Each entry is `(sequence, opcode, position)` so a single server position update can be told
    /// apart from a placement echo.
    positions: Vec<(u32, u16, [f32; 3])>,
}

impl FixtureFacts {
    fn is_placed(&self) -> bool {
        self.place_seq.is_some()
    }

    fn is_removed(&self) -> bool {
        match (self.place_seq, self.remove_seq) {
            (Some(placed), Some(removed)) => removed > placed,
            _ => false,
        }
    }

    fn last_position(&self) -> Option<[f32; 3]> {
        self.positions.last().map(|(_, _, position)| *position)
    }
}

fn collect_fixtures(rows: &[SessionRow]) -> BTreeMap<u64, FixtureFacts> {
    let mut fixtures: BTreeMap<u64, FixtureFacts> = BTreeMap::new();
    for row in rows {
        match &row.body {
            RowBody::PlaceFixture { fixture_guid, .. } => {
                let facts = fixtures.entry(*fixture_guid).or_default();
                if facts.place_seq.is_none() {
                    facts.place_seq = Some(row.seq);
                }
            }
            RowBody::FixtureUpdate {
                fixture_guid,
                position: Some(position),
            }
            | RowBody::UpdateFixturePosition {
                fixture_guid,
                position,
            } => {
                fixtures
                    .entry(*fixture_guid)
                    .or_default()
                    .positions
                    .push((row.seq, row.opcode, *position));
            }
            RowBody::FixtureRemove { fixture_guid } => {
                let facts = fixtures.entry(*fixture_guid).or_default();
                if facts.remove_seq.is_none() {
                    facts.remove_seq = Some(row.seq);
                }
            }
            _ => {}
        }
    }
    fixtures
}

fn evaluate_gate(
    verb: AcceptanceVerb,
    expectation: &AcceptanceExpectation,
    rows: &[SessionRow],
) -> GateStatus {
    match verb {
        AcceptanceVerb::Enter => evaluate_enter(expectation, rows),
        AcceptanceVerb::Place => evaluate_place(expectation, rows),
        AcceptanceVerb::Move => evaluate_move(rows),
        AcceptanceVerb::Remove => evaluate_remove(rows),
        AcceptanceVerb::Visit => evaluate_visit(expectation, rows),
        AcceptanceVerb::Restart => evaluate_restart(expectation, rows),
    }
}

fn first_owner_enter<'a>(
    expectation: &AcceptanceExpectation,
    rows: &'a [SessionRow],
) -> Option<&'a SessionRow> {
    rows.iter().find(|row| {
        row.actor == expectation.owner_guid && matches!(row.body, RowBody::EnterRequest { .. })
    })
}

fn evaluate_enter(expectation: &AcceptanceExpectation, rows: &[SessionRow]) -> GateStatus {
    let Some(request) = first_owner_enter(expectation, rows) else {
        return GateStatus::Unproven {
            missing: "no owner enter request in the transcript",
        };
    };
    let zone = rows.iter().find(|row| {
        row.seq > request.seq
            && row.actor == expectation.owner_guid
            && matches!(row.body, RowBody::ZoneData { .. })
    });
    let Some(zone) = zone else {
        return GateStatus::Unproven {
            missing: "the owner enter request was not answered by zone data",
        };
    };
    if let RowBody::ZoneData {
        zone_definition_id, ..
    } = zone.body
    {
        if zone_definition_id != expectation.zone_definition_id {
            return GateStatus::Fail {
                reason: format!(
                    "zone data names zone {} but the acceptance lot is zone {}",
                    zone_definition_id, expectation.zone_definition_id
                ),
            };
        }
    }
    GateStatus::Pass {
        evidence: format!(
            "enter request at row {} answered by zone data at row {} for zone {}",
            request.seq, zone.seq, expectation.zone_definition_id
        ),
    }
}

fn evaluate_place(expectation: &AcceptanceExpectation, rows: &[SessionRow]) -> GateStatus {
    let Some(response) = rows
        .iter()
        .find(|row| matches!(row.body, RowBody::PlaceFixture { .. }))
    else {
        return GateStatus::Unproven {
            missing: "no place fixture response in the transcript",
        };
    };
    let (fixture_guid, house_guid) = match &response.body {
        RowBody::PlaceFixture {
            fixture_guid,
            house_guid,
        } => (*fixture_guid, *house_guid),
        _ => unreachable!("the row was filtered on its body"),
    };
    let request = rows.iter().find(|row| {
        row.seq < response.seq && matches!(row.body, RowBody::PlaceFixtureRequest { .. })
    });
    if request.is_none() {
        return GateStatus::Fail {
            reason: format!(
                "place fixture response for fixture {fixture_guid} at row {} has no client request before it",
                response.seq
            ),
        };
    }
    if house_guid != expectation.house_guid {
        return GateStatus::Fail {
            reason: format!(
                "place fixture response names house {house_guid} instead of {}",
                expectation.house_guid
            ),
        };
    }
    let mut earlier_place = false;
    let mut earlier_remove = false;
    for row in rows.iter().filter(|row| row.seq < response.seq) {
        match &row.body {
            RowBody::PlaceFixture {
                fixture_guid: guid, ..
            } if *guid == fixture_guid => earlier_place = true,
            RowBody::FixtureRemove {
                fixture_guid: guid, ..
            } if *guid == fixture_guid => earlier_remove = true,
            _ => {}
        }
    }
    if earlier_place {
        return GateStatus::Fail {
            reason: format!("fixture {fixture_guid} was placed twice"),
        };
    }
    if earlier_remove {
        return GateStatus::Fail {
            reason: format!(
                "fixture {fixture_guid} reuses the guid of a fixture that was already removed"
            ),
        };
    }
    GateStatus::Pass {
        evidence: format!(
            "place request at row {} answered by fixture {fixture_guid} at row {}",
            request.expect("the request was checked above").seq,
            response.seq
        ),
    }
}

fn evaluate_move(rows: &[SessionRow]) -> GateStatus {
    let fixtures = collect_fixtures(rows);
    let mut candidates = 0_usize;
    let mut demonstrated = 0_usize;
    for (fixture_guid, facts) in &fixtures {
        if facts.positions.is_empty() {
            continue;
        }
        if !facts.is_placed() {
            return GateStatus::Fail {
                reason: format!("fixture {fixture_guid} was never placed in this transcript"),
            };
        }
        if let Some(removed) = facts.remove_seq {
            if facts.positions.iter().any(|(seq, _, _)| *seq > removed) {
                return GateStatus::Fail {
                    reason: format!("fixture {fixture_guid} moved after it was removed"),
                };
            }
            // The echoes that arrived while the fixture was placed say nothing about a movement.
            continue;
        }
        candidates += 1;
        let after_placement: Vec<&(u32, u16, [f32; 3])> = facts
            .positions
            .iter()
            .filter(|(seq, _, _)| facts.place_seq.is_some_and(|placed| *seq > placed))
            .collect();
        let changed = after_placement
            .windows(2)
            .any(|pair| pair[0].2 != pair[1].2);
        // A single fixture update is the placement echo; a single position update is the server's
        // own confirmation that the fixture moved.
        let confirmed_by_server = after_placement.len() == 1 && after_placement[0].1 == 51;
        if changed || confirmed_by_server {
            demonstrated += 1;
        }
    }
    if candidates == 0 {
        return GateStatus::Unproven {
            missing: "no fixture movement evidence in the transcript",
        };
    }
    if demonstrated == 0 {
        return GateStatus::Fail {
            reason: "the recorded fixture movement repeated the previous transform".to_owned(),
        };
    }
    GateStatus::Pass {
        evidence: format!("{demonstrated} fixture movement(s) changed the transform"),
    }
}

fn evaluate_remove(rows: &[SessionRow]) -> GateStatus {
    let requests: Vec<&SessionRow> = rows
        .iter()
        .filter(|row| matches!(row.body, RowBody::PickupFixture { .. }))
        .collect();
    let Some(request) = requests.first() else {
        return GateStatus::Unproven {
            missing: "no pickup request in the transcript",
        };
    };
    let fixture_guid = match &request.body {
        RowBody::PickupFixture { fixture_guid } => *fixture_guid,
        _ => unreachable!("the rows were filtered on their body"),
    };
    let fixtures = collect_fixtures(rows);
    if let Some(facts) = fixtures.get(&fixture_guid) {
        if !facts.is_placed() {
            return GateStatus::Fail {
                reason: format!("pickup names fixture {fixture_guid}, which was never placed"),
            };
        }
        if let Some(removed) = facts.remove_seq {
            if removed < request.seq {
                return GateStatus::Fail {
                    reason: format!("fixture {fixture_guid} was removed twice"),
                };
            }
        }
    }
    let response = rows.iter().find(|row| {
        row.seq > request.seq
            && matches!(&row.body, RowBody::FixtureRemove { fixture_guid: guid } if *guid == fixture_guid)
    });
    let Some(response) = response else {
        return GateStatus::Unproven {
            missing: "the pickup request was not answered by a fixture remove",
        };
    };
    GateStatus::Pass {
        evidence: format!(
            "pickup request at row {} answered by fixture remove at row {}",
            request.seq, response.seq
        ),
    }
}

enum EditOutcome {
    Succeeded { seq: u32 },
    Refused { code: u32 },
    Unanswered,
}

fn next_edit_outcome(rows: &[SessionRow], request: &SessionRow) -> EditOutcome {
    for row in rows.iter().filter(|row| row.seq > request.seq) {
        match (&row.body, &request.body) {
            (RowBody::PlaceFixture { .. }, RowBody::PlaceFixtureRequest { .. }) => {
                return EditOutcome::Succeeded { seq: row.seq };
            }
            (
                RowBody::FixtureRemove {
                    fixture_guid: response_guid,
                },
                RowBody::PickupFixture {
                    fixture_guid: request_guid,
                },
            ) if response_guid == request_guid => {
                return EditOutcome::Succeeded { seq: row.seq };
            }
            (
                RowBody::FixtureUpdate {
                    fixture_guid: response_guid,
                    ..
                }
                | RowBody::UpdateFixturePosition {
                    fixture_guid: response_guid,
                    ..
                },
                RowBody::SaveFixtureRequest {
                    fixture_guid: request_guid,
                    ..
                },
            ) if response_guid == request_guid => {
                return EditOutcome::Succeeded { seq: row.seq };
            }
            (RowBody::HousingError { code }, _) => {
                return EditOutcome::Refused { code: *code };
            }
            _ => {}
        }
    }
    EditOutcome::Unanswered
}

fn evaluate_visit(expectation: &AcceptanceExpectation, rows: &[SessionRow]) -> GateStatus {
    let Some(guest) = expectation.permitted_guest_guid else {
        return GateStatus::Unproven {
            missing: "no permitted guest identity was supplied",
        };
    };
    let Some(owner_enter) = first_owner_enter(expectation, rows) else {
        return GateStatus::Unproven {
            missing: "the owner has not entered the lot yet",
        };
    };
    let Some(guest_enter) = rows.iter().find(|row| {
        row.actor == guest
            && row.seq > owner_enter.seq
            && matches!(row.body, RowBody::EnterRequest { .. })
    }) else {
        return GateStatus::Unproven {
            missing: "no guest enter request in the transcript",
        };
    };
    let guest_zone = rows.iter().find(|row| {
        row.seq > guest_enter.seq
            && row.actor == guest
            && matches!(row.body, RowBody::ZoneData { .. })
    });
    if let Some(denial) = rows.iter().find(|row| {
        row.seq > guest_enter.seq
            && row.actor == guest
            && matches!(row.body, RowBody::HousingError { .. })
    }) {
        let denial_wins = match guest_zone {
            Some(zone) => denial.seq < zone.seq,
            None => true,
        };
        if denial_wins {
            let code = match denial.body {
                RowBody::HousingError { code } => code,
                _ => unreachable!("the row was filtered on its body"),
            };
            return GateStatus::Fail {
                reason: format!(
                    "the guest was refused with housing error {code} instead of entering"
                ),
            };
        }
    }
    let Some(guest_zone) = guest_zone else {
        return GateStatus::Unproven {
            missing: "the guest enter request was not answered by zone data",
        };
    };
    for request in rows.iter().filter(|row| row.body.is_edit_request()) {
        if request.actor == expectation.owner_guid || request.actor == guest {
            continue;
        }
        match next_edit_outcome(rows, request) {
            EditOutcome::Succeeded { seq } => {
                return GateStatus::Fail {
                    reason: format!(
                        "actor {} is not the owner or a permitted guest, yet its {} at row {seq} succeeded",
                        request.actor,
                        request.body.name()
                    ),
                };
            }
            EditOutcome::Refused { code } if code == HOUSING_ERROR_PERMISSION_DENIED => {}
            EditOutcome::Refused { code } => {
                return GateStatus::Fail {
                    reason: format!(
                        "actor {} was refused with housing error {code} instead of {HOUSING_ERROR_PERMISSION_DENIED}",
                        request.actor
                    ),
                };
            }
            EditOutcome::Unanswered => {
                return GateStatus::Unproven {
                    missing: "an unauthorised edit request was not answered by the server",
                };
            }
        }
    }
    GateStatus::Pass {
        evidence: format!(
            "guest {guest} entered at row {} and received zone data at row {}",
            guest_enter.seq, guest_zone.seq
        ),
    }
}

fn evaluate_restart(expectation: &AcceptanceExpectation, rows: &[SessionRow]) -> GateStatus {
    let Some(restart) = rows
        .iter()
        .find(|row| matches!(row.body, RowBody::ServerRestart))
    else {
        return GateStatus::Unproven {
            missing: "no server restart evidence in the transcript",
        };
    };
    let fixtures = collect_fixtures(rows);
    let placed = fixtures.values().filter(|facts| facts.is_placed()).count();
    if placed == 0 {
        return GateStatus::Unproven {
            missing: "no fixture was placed before the restart",
        };
    }
    let expected: BTreeMap<u64, Option<[f32; 3]>> = fixtures
        .iter()
        .filter(|(_, facts)| facts.is_placed() && !facts.is_removed())
        .map(|(guid, facts)| (*guid, facts.last_position()))
        .collect();
    let Some(instance) = rows.iter().find(|row| {
        row.seq > restart.seq
            && matches!(&row.body, RowBody::InstanceData { house_guid, .. } if *house_guid == expectation.house_guid)
    }) else {
        return GateStatus::Unproven {
            missing: "no instance data after the restart",
        };
    };
    let listed = match &instance.body {
        RowBody::InstanceData { fixtures, .. } => fixtures,
        _ => unreachable!("the row was filtered on its body"),
    };
    for guid in expected.keys() {
        if !listed.iter().any(|row| row.fixture_guid == *guid) {
            return GateStatus::Fail {
                reason: format!("fixture {guid} was not restored after the restart"),
            };
        }
    }
    for row in listed {
        if !expected.contains_key(&row.fixture_guid) {
            return GateStatus::Fail {
                reason: format!(
                    "fixture {} appeared after the restart but no placement explains it",
                    row.fixture_guid
                ),
            };
        }
        if let Some(Some(expected_position)) = expected.get(&row.fixture_guid) {
            if *expected_position != row.position {
                return GateStatus::Fail {
                    reason: format!(
                        "fixture {} was restored at a different transform than the transcript records",
                        row.fixture_guid
                    ),
                };
            }
        }
    }
    GateStatus::Pass {
        evidence: format!(
            "restart at row {} restored {} fixture(s) in instance data at row {}",
            restart.seq,
            listed.len(),
            instance.seq
        ),
    }
}

/// ORIGINAL property row 1 with a RECONSTRUCTED local fixture cap.
pub fn acceptance_property() -> PropertyDefinition {
    PropertyDefinition {
        property_id: ACCEPTANCE_PROPERTY_ID,
        zone_id: ACCEPTANCE_ZONE_ID,
        name_id: ACCEPTANCE_NAME_ID,
        max_fixtures: LOCAL_ACCEPTANCE_MAX_FIXTURES,
        max_landmarks: 0,
        build_areas: vec![ACCEPTANCE_BUILD_AREA],
    }
}

/// RECONSTRUCTED cast for the replay. The identities are local and collide with nothing real.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReplayFixture {
    pub fixture_definition_id: i32,
    pub item_instance_id: u32,
    pub tint_id: i32,
    pub position: [f32; 3],
}

/// RECONSTRUCTED acceptance scenario. The lot is the recovered row 1; everything else is a local
/// choice so the replay is deterministic.
#[derive(Clone, Debug, PartialEq)]
pub struct ReplayScenario {
    pub owner_guid: u64,
    pub guest_guid: u64,
    pub stranger_guid: u64,
    pub property: PropertyDefinition,
    /// Placed first, moved, then expected to survive the restart.
    pub surviving: ReplayFixture,
    /// Placed second and picked up, so the restart must not bring it back.
    pub removed: ReplayFixture,
    pub moved_position: [f32; 3],
}

impl ReplayScenario {
    pub fn recovered_acceptance_lot() -> Self {
        Self {
            owner_guid: 0x0000_1000_0000_0001,
            guest_guid: 0x0000_1000_0000_0002,
            stranger_guid: 0x0000_1000_0000_0003,
            property: acceptance_property(),
            surviving: ReplayFixture {
                fixture_definition_id: 3001,
                item_instance_id: 9001,
                tint_id: 5,
                position: [330.0, -10.0, 400.0],
            },
            removed: ReplayFixture {
                fixture_definition_id: 3002,
                item_instance_id: 9002,
                tint_id: 7,
                position: [350.0, 0.0, 420.0],
            },
            moved_position: [340.0, -5.0, 410.0],
        }
    }

    pub fn expectation(&self, house_guid: u64) -> AcceptanceExpectation {
        AcceptanceExpectation {
            house_guid,
            owner_guid: self.owner_guid,
            permitted_guest_guid: Some(self.guest_guid),
            zone_definition_id: self.property.zone_id,
        }
    }
}

#[derive(Debug)]
pub enum ReplayError {
    Housing(HousingError),
    Unexpected(&'static str),
}

/// The replay result: the transcript the gates scored plus the identities a caller needs.
#[derive(Clone, Debug)]
pub struct ReplayReport {
    pub report: AcceptanceReport,
    pub transcript: Vec<SessionRow>,
    pub snapshot_path: PathBuf,
    pub lot_guid: u64,
    pub owner_guid: u64,
    pub guest_guid: u64,
    pub stranger_guid: u64,
    pub placed_position: [f32; 3],
    pub moved_position: [f32; 3],
    pub surviving_fixture_guid: u64,
    pub removed_fixture_guid: u64,
}

#[derive(Default)]
struct Recorder {
    rows: Vec<SessionRow>,
    next_seq: u32,
}

impl Recorder {
    fn push(&mut self, actor: u64, body: RowBody) {
        self.next_seq += 1;
        let opcode = body.expected_opcode().unwrap_or(0);
        self.rows.push(SessionRow {
            seq: self.next_seq,
            actor,
            opcode,
            body,
        });
    }
}

fn transform(position: [f32; 3], tint_id: i32) -> FixtureTransform {
    FixtureTransform {
        position,
        tint_id,
        ..FixtureTransform::default()
    }
}

/// Drive the reconstructed housing state machine through the acceptance verbs, writing a real
/// snapshot so the restart gate is backed by a genuine save and load. The returned transcript is
/// the golden reference a native capture can be compared against; it is not native evidence.
pub fn replay_reconstructed_session(snapshot_path: &Path) -> Result<ReplayReport, ReplayError> {
    replay_scenario(snapshot_path, &ReplayScenario::recovered_acceptance_lot())
}

pub fn replay_scenario(
    snapshot_path: &Path,
    scenario: &ReplayScenario,
) -> Result<ReplayReport, ReplayError> {
    let property = scenario.property.clone();
    let mut server = Housing::new();
    let mut ledger = FixtureItemLedger::default();
    server
        .grant_property(PropertyGrant {
            owner_guid: scenario.owner_guid,
            property: property.clone(),
            lot_guid: 0,
        })
        .map_err(ReplayError::Housing)?;
    let lot_guid = server
        .lot_guid(scenario.owner_guid, property.property_id)
        .map_err(ReplayError::Housing)?;

    if !ledger.seed(
        scenario.surviving.fixture_definition_id,
        scenario.surviving.tint_id,
        [scenario.surviving.item_instance_id],
    ) || !ledger.seed(
        scenario.removed.fixture_definition_id,
        scenario.removed.tint_id,
        [scenario.removed.item_instance_id],
    ) {
        return Err(ReplayError::Unexpected(
            "the replay furniture could not be seeded",
        ));
    }

    let mut recorder = Recorder::default();

    // Enter: the owner asks for the recovered house guid and receives its zone.
    recorder.push(
        scenario.owner_guid,
        RowBody::EnterRequest {
            house_guid: lot_guid,
        },
    );
    recorder.push(
        scenario.owner_guid,
        RowBody::ZoneData {
            house_guid: lot_guid,
            zone_definition_id: property.zone_id,
        },
    );
    recorder.push(
        scenario.owner_guid,
        RowBody::SetEditMode { in_edit_mode: true },
    );

    // Place two fixtures through the reconstructed server.
    recorder.push(
        scenario.owner_guid,
        RowBody::PlaceFixtureRequest {
            house_guid: lot_guid,
        },
    );
    let placed_surviving = server
        .place_fixture(
            scenario.owner_guid,
            lot_guid,
            scenario.surviving.fixture_definition_id,
            scenario.surviving.item_instance_id,
            transform(scenario.surviving.position, scenario.surviving.tint_id),
            &mut ledger,
        )
        .map_err(ReplayError::Housing)?;
    let surviving_fixture_guid = placed_surviving.placed.fixture_guid;
    recorder.push(
        scenario.owner_guid,
        RowBody::PlaceFixture {
            house_guid: lot_guid,
            fixture_guid: surviving_fixture_guid,
        },
    );
    recorder.push(
        scenario.owner_guid,
        RowBody::FixtureUpdate {
            fixture_guid: surviving_fixture_guid,
            position: Some(scenario.surviving.position),
        },
    );
    recorder.push(
        scenario.owner_guid,
        RowBody::PlaceFixtureRequest {
            house_guid: lot_guid,
        },
    );
    let placed_removed = server
        .place_fixture(
            scenario.owner_guid,
            lot_guid,
            scenario.removed.fixture_definition_id,
            scenario.removed.item_instance_id,
            transform(scenario.removed.position, scenario.removed.tint_id),
            &mut ledger,
        )
        .map_err(ReplayError::Housing)?;
    let removed_fixture_guid = placed_removed.placed.fixture_guid;
    recorder.push(
        scenario.owner_guid,
        RowBody::PlaceFixture {
            house_guid: lot_guid,
            fixture_guid: removed_fixture_guid,
        },
    );
    recorder.push(
        scenario.owner_guid,
        RowBody::FixtureUpdate {
            fixture_guid: removed_fixture_guid,
            position: Some(scenario.removed.position),
        },
    );

    // Move: the client commits the placement it edited, and the server confirms the new position.
    recorder.push(
        scenario.owner_guid,
        RowBody::SaveFixtureRequest {
            house_guid: lot_guid,
            fixture_guid: surviving_fixture_guid,
        },
    );
    let moved = server
        .move_fixture(
            scenario.owner_guid,
            lot_guid,
            surviving_fixture_guid,
            transform(scenario.moved_position, scenario.surviving.tint_id),
            &mut ledger,
        )
        .map_err(ReplayError::Housing)?;
    recorder.push(
        scenario.owner_guid,
        RowBody::UpdateFixturePosition {
            fixture_guid: surviving_fixture_guid,
            position: moved.position,
        },
    );

    // Remove: the client picks a fixture up and the server answers with a fixture remove.
    recorder.push(
        scenario.owner_guid,
        RowBody::PickupFixture {
            fixture_guid: removed_fixture_guid,
        },
    );
    let picked_up = server
        .pickup_fixture(
            scenario.owner_guid,
            lot_guid,
            removed_fixture_guid,
            &mut ledger,
        )
        .map_err(ReplayError::Housing)?;
    recorder.push(
        scenario.owner_guid,
        RowBody::FixtureRemove {
            fixture_guid: picked_up.removed.fixture_guid,
        },
    );
    if ledger.available(
        scenario.removed.fixture_definition_id,
        scenario.removed.tint_id,
    ) == 0
    {
        return Err(ReplayError::Unexpected(
            "a picked-up fixture did not return to the ledger",
        ));
    }

    // Visit: a permitted guest enters, and a stranger's edit is refused with the recovered code.
    server
        .set_permission(
            scenario.owner_guid,
            lot_guid,
            scenario.guest_guid,
            PERMISSION_CAN_EDIT,
        )
        .map_err(ReplayError::Housing)?;
    recorder.push(
        scenario.guest_guid,
        RowBody::EnterRequest {
            house_guid: lot_guid,
        },
    );
    recorder.push(
        scenario.guest_guid,
        RowBody::ZoneData {
            house_guid: lot_guid,
            zone_definition_id: property.zone_id,
        },
    );
    recorder.push(
        scenario.stranger_guid,
        RowBody::SaveFixtureRequest {
            house_guid: lot_guid,
            fixture_guid: surviving_fixture_guid,
        },
    );
    match server.move_fixture(
        scenario.stranger_guid,
        lot_guid,
        surviving_fixture_guid,
        transform(scenario.moved_position, scenario.surviving.tint_id),
        &mut ledger,
    ) {
        Err(HousingError::NotPropertyOwner) => recorder.push(
            scenario.stranger_guid,
            RowBody::HousingError {
                code: HOUSING_ERROR_PERMISSION_DENIED,
            },
        ),
        Err(other) => return Err(ReplayError::Housing(other)),
        Ok(_) => {
            return Err(ReplayError::Unexpected(
                "a stranger was allowed to move a fixture",
            ));
        }
    }

    // Restart: save, drop the server, load a fresh one, and report what the lot now holds.
    server
        .save_state(snapshot_path)
        .map_err(ReplayError::Housing)?;
    let mut restarted = Housing::new();
    restarted
        .load_state(snapshot_path)
        .map_err(ReplayError::Housing)?;
    recorder.push(0, RowBody::ServerRestart);
    let restored = restarted
        .saved_fixtures(lot_guid)
        .map_err(ReplayError::Housing)?;
    recorder.push(
        scenario.owner_guid,
        RowBody::InstanceData {
            house_guid: lot_guid,
            fixtures: restored
                .fixtures
                .iter()
                .map(|fixture| FixtureRow {
                    fixture_guid: fixture.fixture_guid,
                    position: fixture.transform.position,
                })
                .collect(),
        },
    );

    let report = evaluate_transcript(
        EvidenceOrigin::LocalReplay,
        &scenario.expectation(lot_guid),
        &recorder.rows,
    )
    .map_err(|_| ReplayError::Unexpected("the replay produced an invalid transcript"))?;

    Ok(ReplayReport {
        report,
        transcript: recorder.rows,
        snapshot_path: snapshot_path.to_path_buf(),
        lot_guid,
        owner_guid: scenario.owner_guid,
        guest_guid: scenario.guest_guid,
        stranger_guid: scenario.stranger_guid,
        placed_position: scenario.surviving.position,
        moved_position: scenario.moved_position,
        surviving_fixture_guid,
        removed_fixture_guid,
    })
}
