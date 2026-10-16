//! Multi-client integration harness acceptance for `src/restoration/piece_099.rs`.
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `lab/decompiled/client/FreeRealms/pseudocode.c` sha256
//!   `f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3` - `FUN_00c6a7b0`
//!   stores `0x3b` (59) in the packet's family slot and `param_1[2]`, its second argument, is the
//!   sub-opcode. Each `Trade*Packet` constructor calls `FUN_00c6a7b0(<sub>)` immediately before its
//!   `Trade*Packet::vftable` store, which is where the 18-row trade table below is read from.
//! * `lab/references/FreeRealms-Legacy/src/Sanctuary.Packet.Common/Extensions/PacketReaderExtensions.cs`
//!   - the implemented community reader: family 59 opens with a one-byte sub-opcode and names every
//!   row of that table.
//! * `lab/references/FreeRealms-Legacy/src/Sanctuary.Packet/BaseTradePacket.cs` - the community
//!   reader/serializer that gives the six recovered client-to-server command layouts
//!   (`InviteReply`, `Accept`, `Cancel`, `Lock`, `OfferItem`, `CoinCount`).
//! * `lab/references/OpenSourceFreeRealms/ofrserver/Server/LibSOE/Core/PacketCodes.cs` -
//!   `BaseTradePacket = 59` and the same 18 sub-opcodes, cross-checked against the client binary.
//! * `src/restoration/piece_020.rs` - the candidate activity-launch module. This test pins the
//!   harness's grouping intents to that module's recovered family-175 sub-opcode table.
//!
//! GAP this test does not close: the original server's trade session/idempotency rules, the
//! grouping open/cancel/enter requests, and any native multi-client acceptance. The gateway path
//! is used here only to record that the recovered families are silently dropped today.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers `pub mod piece_099;` in `src/restoration`. Integration step for the lead: register the
//! module and drop the include.

#[path = "../src/restoration/piece_020.rs"]
#[allow(dead_code)]
mod piece_020;
#[path = "../src/restoration/piece_099.rs"]
#[allow(dead_code)]
mod piece_099;

use osfr_tools::character_store::CharacterState;
use osfr_tools::gateway::Gateway;
use osfr_tools::startup_snapshot::StartupSnapshot;
use piece_099::{
    Action, Case, Client, Comparison, GroupRequest, Outcome, Scenario, ScenarioError, Step,
    TRADE_FAMILY, TradeCommand, TradeError, TradeMessage, compare, run, scenario,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[path = "common/startup_selfdata_fixture.rs"]
mod startup_selfdata_fixture;

const OWNER: u64 = 0x0000_0040_0000_0001;
const MEMBER: u64 = 0x0000_0060_0000_0002;
const THIRD: u64 = 0x0000_0060_0000_0003;
const FIRST_INSTANCE: i32 = 90_000_001;

/// A dispatcher over the candidate grouping module (piece 020).
struct Grouping {
    registry: piece_020::ActivityGroupRegistry,
    last: Value,
}

impl Grouping {
    fn new(seed: i32) -> Self {
        Self {
            registry: piece_020::ActivityGroupRegistry::new(seed),
            last: json!(null),
        }
    }

    fn dispatch(&mut self, step: &Step) -> Outcome {
        let result: Result<(), piece_020::GroupError> = match &step.action {
            Action::Group(GroupRequest::Open { group, capacity }) => self
                .registry
                .open(*group, step.client, *capacity)
                .map(|_| ()),
            Action::Group(GroupRequest::Invite { group, target }) => self
                .registry
                .invite(*group, step.client, *target)
                .map(|_| ()),
            Action::Group(GroupRequest::Accept { group }) => {
                self.registry.accept(*group, step.client).map(|_| ())
            }
            Action::Group(GroupRequest::Ready { group, ready }) => self
                .registry
                .set_ready(*group, step.client, *ready)
                .map(|_| ()),
            Action::Group(GroupRequest::Kick { group, target }) => {
                self.registry.kick(*group, step.client, *target).map(|_| ())
            }
            Action::Group(GroupRequest::Quit { group }) => {
                self.registry.quit(*group, step.client).map(|_| ())
            }
            Action::Group(GroupRequest::Launch { group }) => {
                self.registry.launch(*group, step.client).map(|_| ())
            }
            Action::Group(GroupRequest::Cancel { group }) => {
                self.registry.cancel(*group, step.client).map(|_| ())
            }
            Action::Group(GroupRequest::EnterInstance { instance }) => {
                self.registry.authorize_instance(*instance, step.client)
            }
            other => panic!("{} is not a grouping step", other.label()),
        };
        let state = self.registry.state();
        let changed = state != self.last;
        self.last = state.clone();
        match result {
            Ok(()) => Outcome::accepted(state).with_changed(changed),
            Err(error) => Outcome::refused(format!("{error:?}"), state).with_changed(changed),
        }
    }
}

fn grouping_run(case: Case) -> piece_099::Run {
    let scenario = scenario(case);
    let mut dispatcher = Grouping::new(scenario.seed);
    run(&scenario, |step| dispatcher.dispatch(step)).unwrap()
}

fn trade_wire_bytes(action: &Action) -> Vec<u8> {
    action
        .wire_bytes()
        .expect("every trade step carries recovered wire bytes")
}

#[test]
fn trade_vocabulary_is_pinned_to_the_supplied_client_and_community_sources() {
    let expected: [(u8, &str); 18] = [
        (2, "TradeInvitePacket"),
        (3, "TradeInviteReplyPacket"),
        (4, "TradeStartSessionPacket"),
        (5, "TradeEndSessionPacket"),
        (6, "TradeAcceptPacket"),
        (7, "TradeCancelPacket"),
        (8, "TradeLockPacket"),
        (9, "TradeOfferItemPacket"),
        (10, "TradeCoinCountPacket"),
        (11, "TradeUpdateCoinCountPacket"),
        (12, "TradeUpdateAddItemPacket"),
        (13, "TradeUpdateItemCountPacket"),
        (14, "TradeUpdateAcceptPacket"),
        (15, "TradeMessagePacket"),
        (16, "TradeRejectedPacket"),
        (17, "TradeUpdateLockPacket"),
        (18, "TradeConfirmPacket"),
        (19, "TradeItemRemovedPacket"),
    ];
    assert_eq!(TRADE_FAMILY, 59);
    assert_eq!(piece_099::trade_sub_width(), 1);
    assert_eq!(TradeMessage::CLASS_TABLE, expected);
    assert_eq!(TradeMessage::ALL.len(), expected.len());
    for (index, (sub, name)) in expected.iter().enumerate() {
        let message = TradeMessage::ALL[index];
        assert_eq!(message.sub_opcode(), *sub, "{name}");
        assert_eq!(message.class_name(), *name);
        assert_eq!(TradeMessage::from_sub(*sub), Some(message));
    }
    assert_eq!(TradeMessage::from_sub(0), None);
    assert_eq!(TradeMessage::from_sub(1), None);
    assert_eq!(TradeMessage::from_sub(20), None);
}

#[test]
fn trade_commands_round_trip_the_recovered_client_wire() {
    let cases = [
        (
            TradeCommand::InviteReply {
                session: 7,
                accepted: true,
            },
            "3b00030700000001",
        ),
        (
            TradeCommand::Accept {
                session: 7,
                revision: 3,
            },
            "3b00060700000003000000",
        ),
        (TradeCommand::Cancel { session: 7 }, "3b000707000000"),
        (
            TradeCommand::Lock {
                session: 7,
                revision: 3,
                locked: true,
            },
            "3b0008070000000300000001",
        ),
        (
            TradeCommand::OfferItem {
                session: 7,
                item: 1195,
                resulting_count: 5,
            },
            "3b000907000000ab04000005000000",
        ),
        (
            TradeCommand::CoinCount {
                session: 7,
                resulting_coins: 70,
            },
            "3b000a0700000046000000",
        ),
    ];
    assert_eq!(cases.len(), 6, "the recovered client-to-server command set");
    for (command, hex) in cases {
        let encoded = command.clone().encode();
        assert_eq!(hex_of(&encoded), hex, "{command:?}");
        assert_eq!(TradeCommand::parse(&encoded), Ok(command));
    }

    // Malformed wire is refused rather than guessed at.
    assert_eq!(
        TradeCommand::parse(&[0x3a, 0x00, 0x08]),
        Err(TradeError::WrongFamily)
    );
    assert_eq!(
        TradeCommand::parse(&[0x3b, 0x00, 0x01]),
        Err(TradeError::UnknownSubOpcode(1))
    );
    assert_eq!(
        TradeCommand::parse(&[0x3b, 0x00, 0x08, 0x07]),
        Err(TradeError::Truncated)
    );
    assert_eq!(
        TradeCommand::parse(&[0x3b, 0x00, 0x07, 0x07, 0x00, 0x00, 0x00, 0x00]),
        Err(TradeError::TrailingBytes)
    );
    let mut bad_bool = TradeCommand::InviteReply {
        session: 7,
        accepted: true,
    }
    .encode();
    *bad_bool.last_mut().unwrap() = 2;
    assert_eq!(
        TradeCommand::parse(&bad_bool),
        Err(TradeError::InvalidBoolean)
    );
}

#[test]
fn grouping_intents_match_the_recovered_activity_launch_table() {
    // The recovered client-to-server intents, pinned to piece 020's family-175 table.
    let pinned: [(GroupRequest, &str); 6] = [
        (
            GroupRequest::Invite {
                group: 12,
                target: MEMBER,
            },
            "ClientActivityLaunchPacketInviteMemberRequest",
        ),
        (
            GroupRequest::Accept { group: 12 },
            "ClientActivityLaunchPacketInviteResponse",
        ),
        (
            GroupRequest::Ready {
                group: 12,
                ready: true,
            },
            "ClientActivityLaunchPacketMemberReady",
        ),
        (
            GroupRequest::Kick {
                group: 12,
                target: MEMBER,
            },
            "ClientActivityLaunchPacketKickMemberRequest",
        ),
        (
            GroupRequest::Quit { group: 12 },
            "ClientActivityLaunchPacketMemberQuit",
        ),
        (
            GroupRequest::Launch { group: 12 },
            "ClientActivityLaunchPacketOwnerLaunchRequest",
        ),
    ];
    assert_eq!(pinned.len(), GroupRequest::RECOVERED_TABLE.len());
    for (index, (request, name)) in pinned.iter().enumerate() {
        let sub = request
            .sub_opcode()
            .unwrap_or_else(|| panic!("{} must carry a recovered sub-opcode", request.label()));
        let message = piece_020::LaunchMessage::from_sub(i32::from(sub))
            .unwrap_or_else(|| panic!("sub-opcode {sub} must exist in piece 020"));
        let wire = request
            .wire_bytes()
            .expect("a recovered intent carries wire bytes");
        assert_eq!(wire[0..2], 175u16.to_le_bytes(), "{}", request.label());
        assert_eq!(wire[2], sub);
        assert_eq!(message.sub_opcode(), i32::from(sub), "{}", request.label());
        assert_eq!(message.class_name(), *name, "{}", request.label());
        assert_eq!(
            GroupRequest::RECOVERED_TABLE[index],
            (sub, *name),
            "the shipped table must match the recovered intent"
        );
    }

    // Intents with no recovered request stay explicitly unmapped.
    for request in [
        GroupRequest::Open {
            group: 12,
            capacity: 4,
        },
        GroupRequest::Cancel { group: 12 },
        GroupRequest::EnterInstance {
            instance: FIRST_INSTANCE,
        },
    ] {
        assert_eq!(request.sub_opcode(), None, "{}", request.label());
        assert_eq!(request.wire_bytes(), None, "{}", request.label());
    }
}

#[test]
fn party_case_is_deterministic_and_compares_identical() {
    let first = grouping_run(Case::Party);
    let second = grouping_run(Case::Party);
    assert_eq!(first, second);
    assert_eq!(
        compare(&first, &second),
        Comparison::Identical {
            digest: first.digest()
        }
    );
    assert_eq!(first.digest().len(), 64);

    let scenario = scenario(Case::Party);
    assert_eq!(first.log.len(), scenario.steps.len());
    assert_eq!(first.launch.case, "party");
    assert_eq!(first.launch.steps, scenario.steps.len());
    assert_eq!(first.log[0].action, "group.open(12,4)");
    assert_eq!(first.log[0].verdict, "accepted");
    assert_eq!(first.log[0].state_changed, true);
    assert_eq!(first.log[0].wire, None, "no recovered open request exists");
    assert_eq!(first.log[1].wire.as_deref(), Some("af0014"));
    assert_eq!(first.log[8].verdict, "accepted");
    assert!(
        first
            .log
            .iter()
            .all(|record| record.index < first.log.len())
    );
    assert_eq!(first.log[2].wire.as_deref(), Some("af0011"));
    assert_eq!(
        first.log[5].wire.as_deref(),
        Some("af0015"),
        "the owner kick carries the recovered kick request"
    );
    assert_eq!(
        first.state["groups"][0]["members"].as_array().map(Vec::len),
        Some(1),
        "the invited member quits and only the owner remains"
    );
}

#[test]
fn activity_case_launches_an_instance_and_refuses_a_non_member() {
    let run = grouping_run(Case::Activity);
    let launch = run
        .log
        .iter()
        .find(|record| record.action == "group.launch(21)")
        .expect("the activity case launches one instance");
    assert_eq!(launch.verdict, "accepted");
    assert!(launch.state_changed);
    assert_eq!(launch.wire.as_deref(), Some("af0016"));
    assert_eq!(
        run.state["instances"][0]["members"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
    assert_eq!(run.state["instances"][0]["activity_id"], 21);

    let stranger = run
        .log
        .iter()
        .find(|record| record.action.starts_with("group.enter_instance("))
        .expect("the activity case probes a non-member");
    assert_eq!(stranger.verdict, "refused");
    assert_eq!(stranger.detail.as_deref(), Some("NotInInstance"));
    assert!(!stranger.state_changed);
    assert_eq!(stranger.wire, None, "no recovered enter request exists");

    // Cancelling after the launch is refused: the instance owns the roster now.
    let cancel = run
        .log
        .iter()
        .find(|record| record.action == "group.cancel(21)")
        .expect("the activity case attempts a late cancel");
    assert_eq!(cancel.verdict, "refused");
    assert_eq!(cancel.detail.as_deref(), Some("AlreadyLaunched"));
    assert!(!cancel.state_changed);
}

#[test]
fn comparison_reports_the_first_divergence_for_each_stream() {
    let baseline = grouping_run(Case::Activity);

    // A log divergence: the same script with one step replayed by the wrong client.
    let mut changed = scenario(Case::Activity);
    changed.steps[2].client = THIRD;
    let mut dispatcher = Grouping::new(changed.seed);
    let changed_run = run(&changed, |step| dispatcher.dispatch(step)).unwrap();
    let Comparison::Diverged {
        stream,
        path,
        left,
        right,
    } = compare(&baseline, &changed_run)
    else {
        panic!("a wrong-client replay must diverge");
    };
    assert_eq!(stream, "log");
    assert_eq!(
        path, "log[2].client",
        "the first differing field in canonical order"
    );
    assert_eq!(left, MEMBER.to_string());
    assert_eq!(right, THIRD.to_string());
    assert_eq!(
        changed_run.log[2].verdict, "refused",
        "the wrong client cannot accept another player's invitation"
    );

    // A launch divergence: the same log and state, a different client roster.
    let mut renamed = baseline.clone();
    renamed.launch.clients[0].name = "Other Tester".into();
    let Comparison::Diverged { stream, path, .. } = compare(&baseline, &renamed) else {
        panic!("a changed launch record must diverge");
    };
    assert_eq!(stream, "launch");
    assert_eq!(path, "launch.clients[0].name");

    // A state divergence: identical launch and log records, different final state.
    let mut mutated = baseline.clone();
    mutated.state["groups"][0]["capacity"] = json!(9);
    let Comparison::Diverged {
        stream,
        path,
        left,
        right,
    } = compare(&baseline, &mutated)
    else {
        panic!("a changed snapshot must diverge");
    };
    assert_eq!(stream, "state");
    assert_eq!(path, "state.groups[0].capacity");
    assert_eq!(left, "4");
    assert_eq!(right, "9");
}

#[test]
fn trade_case_records_the_silent_drop_on_the_real_gateway() {
    let reference = Reference::new();
    let mut gateway = Gateway::load_with_startup_snapshot(
        &reference.directory,
        initial_state(),
        StartupSnapshot::empty(),
    )
    .unwrap();
    let initial = serde_json::to_value(gateway.state()).unwrap();
    let scenario = scenario(Case::Trade);
    assert_eq!(scenario.clients.len(), 2);
    assert_eq!(scenario.steps.len(), 8);
    assert!(matches!(scenario.case, Case::Trade));

    let report = run(&scenario, |step| {
        let message = wrapped(5, &trade_wire_bytes(&step.action));
        let outcome = gateway.handle(&message);
        let state = serde_json::to_value(gateway.state()).unwrap();
        match outcome {
            Ok(outcome)
                if outcome.replies.is_empty()
                    && !outcome.state_changed
                    && !outcome.gameplay_changed
                    && !outcome.entered_world =>
            {
                Outcome::unhandled(state)
            }
            Ok(_) => Outcome::accepted(state),
            Err(error) => Outcome::refused(error.to_string(), state),
        }
    })
    .unwrap();

    // Today the recovered trade family is neither refused nor answered: it is dropped. The harness
    // must make that visible instead of reporting a passing trade.
    assert!(
        report
            .log
            .iter()
            .all(|record| record.verdict == "unhandled")
    );
    assert!(report.log.iter().all(|record| !record.state_changed));
    assert!(report.log.iter().all(|record| record.reply_count == 0));
    assert_eq!(report.log[0].wire.as_deref(), Some("3b00030700000001"));
    assert_eq!(report.state, initial, "a dropped trade never moves state");
    assert_eq!(
        compare(&report, &report.clone()),
        Comparison::Identical {
            digest: report.digest()
        }
    );
}

#[test]
fn scenario_validation_rejects_malformed_scripts() {
    let base = scenario(Case::Party);

    let mut empty_clients = base.clone();
    empty_clients.clients.clear();
    assert_eq!(empty_clients.validate(), Err(ScenarioError::NoClients));

    let mut empty_steps = base.clone();
    empty_steps.steps.clear();
    assert_eq!(empty_steps.validate(), Err(ScenarioError::NoSteps));

    let mut unknown = base.clone();
    unknown.steps[1].client = 0xdead_beef;
    assert_eq!(
        unknown.validate(),
        Err(ScenarioError::UnknownClient(0xdead_beef))
    );

    let mut duplicate = base.clone();
    duplicate.clients[1].guid = OWNER;
    assert_eq!(
        duplicate.validate(),
        Err(ScenarioError::DuplicateClient(OWNER))
    );

    let mut repeated = base.clone();
    let mut extra = repeated.steps[0].clone();
    extra.action = Action::Group(GroupRequest::Ready {
        group: 12,
        ready: true,
    });
    repeated.steps.insert(1, extra);
    assert_eq!(
        repeated.validate(),
        Err(ScenarioError::ScheduleNotIncreasing {
            previous: (1, OWNER),
            next: (1, OWNER),
        })
    );

    let mut backwards = base.clone();
    backwards.steps[1].tick = 0;
    assert_eq!(
        backwards.validate(),
        Err(ScenarioError::ScheduleNotIncreasing {
            previous: (1, OWNER),
            next: (0, OWNER),
        })
    );

    // Every shipped case is a valid, client-addressable script.
    for case in [Case::Party, Case::Trade, Case::Activity] {
        let scenario = scenario(case);
        assert_eq!(scenario.validate(), Ok(()), "{case:?}");
        assert!(
            scenario
                .clients
                .iter()
                .all(|client| !client.name.is_empty())
        );
    }
}

#[test]
fn replies_belong_to_the_log_stream_not_just_the_state() {
    // A one-step wire script whose server replies, to pin the reply half of the log record.
    let script = Scenario {
        case: Case::Trade,
        seed: 1,
        clients: vec![Client {
            guid: OWNER,
            name: "Tester One".into(),
        }],
        steps: vec![Step {
            tick: 1,
            client: OWNER,
            action: Action::Wire {
                family: TRADE_FAMILY,
                sub: 2,
                body: vec![0xaa],
            },
        }],
    };
    assert_eq!(script.validate(), Ok(()));
    assert_eq!(script.steps[0].action.label(), "wire.59.2.aa");

    let answered = run(&script, |_| {
        Outcome::unhandled(json!({ "step": 1 })).with_replies(vec![vec![1, 2, 3]])
    })
    .unwrap();
    let silent = run(&script, |_| Outcome::unhandled(json!({ "step": 1 }))).unwrap();
    assert_eq!(answered.log[0].reply_count, 1);
    assert_eq!(silent.log[0].reply_count, 0);
    assert_ne!(answered.log[0].reply_digest, silent.log[0].reply_digest);
    assert_eq!(answered.log[0].wire.as_deref(), Some("3b0002aa"));
    let Comparison::Diverged { stream, path, .. } = compare(&answered, &silent) else {
        panic!("a different reply set must diverge");
    };
    assert_eq!(stream, "log");
    assert_eq!(path, "log[0].reply_count");
}

fn hex_of(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn initial_state() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: [1.0, 2.0, 3.0],
        rotation: [0.1, 0.2, 0.3],
    }
}

fn string(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u32).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

fn wrapped(outer: u16, inner: &[u8]) -> Vec<u8> {
    let mut bytes = outer.to_le_bytes().to_vec();
    bytes.push(1);
    bytes.extend_from_slice(&(inner.len() as u32).to_le_bytes());
    bytes.extend_from_slice(inner);
    bytes
}

fn packet(opcode: u16, fields: &[u8]) -> Vec<u8> {
    let mut inner = opcode.to_le_bytes().to_vec();
    inner.extend_from_slice(fields);
    wrapped(5, &inner)
}

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);
const VERSION: &str = "synthetic.1";

/// A minimal verified gateway reference. This mirrors `tests/gateway.rs`; the harness uses the real
/// gateway only to record what the current server does with recovered trade traffic.
struct Reference {
    directory: PathBuf,
    manifest: Value,
}

impl Reference {
    fn new() -> Self {
        let serial = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "osfr-piece-099-{}-{time}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&directory).unwrap();
        let state = initial_state();
        let (self_packet, position_offset, rotation_offset) =
            startup_selfdata_fixture::self_data_packet(state.guid, state.position, state.rotation);
        let mut zone = Vec::new();
        string(&mut zone, "test zone");
        zone.extend_from_slice(&2u32.to_le_bytes());
        zone.extend_from_slice(&[0, 0]);
        string(&mut zone, "sky");
        zone.push(0);
        let zone_id_offset = 9 + zone.len();
        zone.extend_from_slice(&state.zone_id.to_le_bytes());
        zone.extend_from_slice(&5u32.to_le_bytes());
        zone.extend_from_slice(&[0, 0]);
        let zone_packet = packet(43, &zone);
        let mut templates = Vec::new();
        for index in 0..46 {
            let opcode = if index < 8 {
                [41, 2, 166, 43, 144, 35, 193, 12][index]
            } else if index == 8 {
                67
            } else if index == 45 {
                14
            } else {
                200
            };
            let bytes = match index {
                3 => zone_packet.clone(),
                7 => self_packet.clone(),
                _ => packet(opcode, &[]),
            };
            let name = format!("{index:04}.bin");
            fs::write(directory.join(&name), &bytes).unwrap();
            templates.push(json!({"kind":if index < 8 {"login"} else {"ready"},
                "file":name,"length":bytes.len(),"sha256":format!("{:x}",Sha256::digest(&bytes)),
                "opcode":opcode}));
        }
        let manifest = json!({"schema_version":1,"client_version":VERSION,"initial_state":state,
            "self_layout":{"guid_offset":21,"position_offset":position_offset,
                "rotation_offset":rotation_offset},
            "zone_layout":{"zone_id_offset":zone_id_offset},"templates":templates});
        fs::write(
            directory.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        Self {
            directory,
            manifest,
        }
    }
}

impl Drop for Reference {
    fn drop(&mut self) {
        if !self.manifest.is_null()
            && self.directory.parent() == Some(std::env::temp_dir().as_path())
        {
            let _ = fs::remove_dir_all(&self.directory);
        }
    }
}
