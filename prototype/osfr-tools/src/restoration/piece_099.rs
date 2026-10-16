//! Multi-client integration harness: deterministic launch/log/state comparison for the local
//! server, driven by headless scenarios instead of two competing native windows.
//!
//! Piece 099, owned paths only. Evidence labels follow the other restoration modules: ORIGINAL is
//! read from the supplied client, the pinned community readers/serializers or the pinned older
//! server enum; RECONSTRUCTED is a bounded local policy; MISSING is an explicit gap.
//!
//! ORIGINAL facts used here:
//! * the trade wire family is `BaseTradePacket` = **59** (`0x3b`). The client constructor
//!   `FUN_00c6a7b0` stores `0x3b` in the packet's family slot and its argument in the 32-bit
//!   sub-opcode slot; each `Trade*Packet` constructor calls `FUN_00c6a7b0(<sub>)` immediately before
//!   its `Trade*Packet::vftable` store, so [`TradeMessage::CLASS_TABLE`] is read from the shipped
//!   binary rather than translated from a comment.
//! * the implemented community reader opens family 59 with a **one-byte** sub-opcode and names the
//!   same 18 rows; the pinned older server enum agrees on `BaseTradePacket = 59` and every
//!   sub-opcode. Six of those rows (`InviteReply`, `Accept`, `Cancel`, `Lock`, `OfferItem`,
//!   `CoinCount`) have recovered client-to-server field layouts in the pinned Sanctuary
//!   `BaseTradePacket.TryDeserialize`, which is what [`TradeCommand`] encodes.
//! * the grouping intents in [`GroupRequest`] reuse the recovered `ClientActivityLaunchBasePacket`
//!   = **175** sub-opcodes (`InviteMemberRequest` 20, `InviteResponse` 17, `MemberReady` 18,
//!   `KickMemberRequest` 21, `MemberQuit` 19, `OwnerLaunchRequest` 22) read from the client
//!   constructors and the community reader by `src/restoration/piece_020.rs`. The harness consumes
//!   that vocabulary; the grouping state machine itself belongs to piece 020.
//!
//! RECONSTRUCTED here (no recovered original for these):
//! * the scenario scripts for [`Case::Party`], [`Case::Trade`] and [`Case::Activity`], including
//!   the scripted trade session id and revisions. The bodies are recoverable wire shapes, but the
//!   session values are harness fixtures, not captured server assignments.
//! * the canonical launch/log/state record shape and the comparison policy. This is the harness's
//!   own contract; nothing here is a historical server behavior.
//!
//! MISSING / explicit gaps:
//! * there is no recovered client request for opening a group, cancelling a formation or entering
//!   an instance, so those intents carry [`None`] wire bytes and are harness-level steps only.
//! * the original server's trade session rules (two-sided lock/accept, revision checks,
//!   disconnect rollback, staleness) are unrecovered; the trade case therefore has no settlement
//!   policy here and must not gain one until the owning item recovers it.
//! * the local server does not dispatch family 59 or 175 at all today: `Gateway` without gameplay
//!   content drops application messages and `Gameplay::handle` ends in a silent `Ok(out)` for
//!   unknown families. The harness records that as `unhandled` instead of a passing step.
//! * two native clients on one desktop remain the last gate; this harness never touches the
//!   shared UI and is not native acceptance.
//!
//! ponytail: one ordered step list, one dispatcher closure, one canonical JSON document per run and
//! one recursive differ. No scheduler thread, no clock, no transport, no per-case framework. Add a
//! concurrent runner only when a real multi-session server exists to drive.

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

/// Harness version recorded in every launch record.
pub const HARNESS_VERSION: &str = "piece-099/1";
/// ORIGINAL: `BaseTradePacket` in the supplied client and the pinned community sources.
pub const TRADE_FAMILY: u16 = 59;
/// ORIGINAL: `ClientActivityLaunchBasePacket`, owned by `src/restoration/piece_020.rs`.
pub const GROUPING_FAMILY: u16 = 175;

/// How a fact in this module is supported.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Evidence {
    /// Read from the supplied client or the pinned community sources.
    Original,
    /// A bounded local policy with no recovered original.
    Reconstructed,
    /// Known to be missing; never presented as implemented.
    Missing,
}

/// ORIGINAL: every recovered trade sub-opcode, read from the client constructors that call
/// `FUN_00c6a7b0(<sub>)` before storing their `Trade*Packet::vftable`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TradeMessage {
    Invite = 2,
    InviteReply = 3,
    StartSession = 4,
    EndSession = 5,
    Accept = 6,
    Cancel = 7,
    Lock = 8,
    OfferItem = 9,
    CoinCount = 10,
    UpdateCoinCount = 11,
    UpdateAddItem = 12,
    UpdateItemCount = 13,
    UpdateAccept = 14,
    Message = 15,
    Rejected = 16,
    UpdateLock = 17,
    Confirm = 18,
    ItemRemoved = 19,
}

impl TradeMessage {
    /// Every recovered row, in ascending sub-opcode order.
    pub const ALL: [TradeMessage; 18] = [
        TradeMessage::Invite,
        TradeMessage::InviteReply,
        TradeMessage::StartSession,
        TradeMessage::EndSession,
        TradeMessage::Accept,
        TradeMessage::Cancel,
        TradeMessage::Lock,
        TradeMessage::OfferItem,
        TradeMessage::CoinCount,
        TradeMessage::UpdateCoinCount,
        TradeMessage::UpdateAddItem,
        TradeMessage::UpdateItemCount,
        TradeMessage::UpdateAccept,
        TradeMessage::Message,
        TradeMessage::Rejected,
        TradeMessage::UpdateLock,
        TradeMessage::Confirm,
        TradeMessage::ItemRemoved,
    ];

    /// ORIGINAL: `(sub-opcode, client class)` for every row, read from the shipped binary and
    /// cross-checked against the pinned community reader.
    pub const CLASS_TABLE: [(u8, &'static str); 18] = [
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

    pub fn sub_opcode(self) -> u8 {
        self as u8
    }

    pub fn class_name(self) -> &'static str {
        Self::CLASS_TABLE
            .into_iter()
            .find(|(sub, _)| *sub == self.sub_opcode())
            .map(|(_, name)| name)
            .expect("every variant has a recovered class row")
    }

    pub fn from_sub(sub: u8) -> Option<Self> {
        Self::ALL.into_iter().find(|row| row.sub_opcode() == sub)
    }

    pub fn evidence(self) -> Evidence {
        Evidence::Original
    }
}

/// ORIGINAL: family 59 opens with a one-byte sub-opcode in the implemented community reader.
pub fn trade_sub_width() -> usize {
    1
}

/// ORIGINAL: the six client-to-server trade commands with recovered field layouts
/// (pinned `BaseTradePacket.TryDeserialize`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TradeCommand {
    InviteReply {
        session: i32,
        accepted: bool,
    },
    Accept {
        session: i32,
        revision: i32,
    },
    Cancel {
        session: i32,
    },
    Lock {
        session: i32,
        revision: i32,
        locked: bool,
    },
    OfferItem {
        session: i32,
        item: i32,
        resulting_count: i32,
    },
    CoinCount {
        session: i32,
        resulting_coins: i32,
    },
}

impl TradeCommand {
    pub fn message(self) -> TradeMessage {
        match self {
            TradeCommand::InviteReply { .. } => TradeMessage::InviteReply,
            TradeCommand::Accept { .. } => TradeMessage::Accept,
            TradeCommand::Cancel { .. } => TradeMessage::Cancel,
            TradeCommand::Lock { .. } => TradeMessage::Lock,
            TradeCommand::OfferItem { .. } => TradeMessage::OfferItem,
            TradeCommand::CoinCount { .. } => TradeMessage::CoinCount,
        }
    }

    /// The recovered client message: `u16 family | u8 sub-opcode | fields`.
    pub fn encode(self) -> Vec<u8> {
        let mut bytes = TRADE_FAMILY.to_le_bytes().to_vec();
        bytes.push(self.message().sub_opcode());
        match self {
            TradeCommand::InviteReply { session, accepted } => {
                bytes.extend_from_slice(&session.to_le_bytes());
                bytes.push(u8::from(accepted));
            }
            TradeCommand::Accept { session, revision } => {
                bytes.extend_from_slice(&session.to_le_bytes());
                bytes.extend_from_slice(&revision.to_le_bytes());
            }
            TradeCommand::Cancel { session } => bytes.extend_from_slice(&session.to_le_bytes()),
            TradeCommand::Lock {
                session,
                revision,
                locked,
            } => {
                bytes.extend_from_slice(&session.to_le_bytes());
                bytes.extend_from_slice(&revision.to_le_bytes());
                bytes.push(u8::from(locked));
            }
            TradeCommand::OfferItem {
                session,
                item,
                resulting_count,
            } => {
                bytes.extend_from_slice(&session.to_le_bytes());
                bytes.extend_from_slice(&item.to_le_bytes());
                bytes.extend_from_slice(&resulting_count.to_le_bytes());
            }
            TradeCommand::CoinCount {
                session,
                resulting_coins,
            } => {
                bytes.extend_from_slice(&session.to_le_bytes());
                bytes.extend_from_slice(&resulting_coins.to_le_bytes());
            }
        }
        bytes
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, TradeError> {
        let mut cursor = Cursor::new(bytes);
        if cursor.u16()? != TRADE_FAMILY {
            return Err(TradeError::WrongFamily);
        }
        let sub = cursor.u8()?;
        let message = TradeMessage::from_sub(sub).ok_or(TradeError::UnknownSubOpcode(sub))?;
        let command = match message {
            TradeMessage::InviteReply => TradeCommand::InviteReply {
                session: cursor.i32()?,
                accepted: cursor.boolean()?,
            },
            TradeMessage::Accept => TradeCommand::Accept {
                session: cursor.i32()?,
                revision: cursor.i32()?,
            },
            TradeMessage::Cancel => TradeCommand::Cancel {
                session: cursor.i32()?,
            },
            TradeMessage::Lock => TradeCommand::Lock {
                session: cursor.i32()?,
                revision: cursor.i32()?,
                locked: cursor.boolean()?,
            },
            TradeMessage::OfferItem => TradeCommand::OfferItem {
                session: cursor.i32()?,
                item: cursor.i32()?,
                resulting_count: cursor.i32()?,
            },
            TradeMessage::CoinCount => TradeCommand::CoinCount {
                session: cursor.i32()?,
                resulting_coins: cursor.i32()?,
            },
            other => {
                return Err(TradeError::UnknownSubOpcode(other.sub_opcode()));
            }
        };
        cursor.end()?;
        Ok(command)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TradeError {
    WrongFamily,
    UnknownSubOpcode(u8),
    Truncated,
    TrailingBytes,
    InvalidBoolean,
}

struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], TradeError> {
        let end = self.at.checked_add(count).ok_or(TradeError::Truncated)?;
        let part = self.bytes.get(self.at..end).ok_or(TradeError::Truncated)?;
        self.at = end;
        Ok(part)
    }

    fn u8(&mut self) -> Result<u8, TradeError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, TradeError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn i32(&mut self) -> Result<i32, TradeError> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn boolean(&mut self) -> Result<bool, TradeError> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(TradeError::InvalidBoolean),
        }
    }

    fn end(&self) -> Result<(), TradeError> {
        if self.at == self.bytes.len() {
            Ok(())
        } else {
            Err(TradeError::TrailingBytes)
        }
    }
}

/// The grouping intents a harness script can drive. The state machine belongs to piece 020; this
/// module only names the intents and pins the recovered client requests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupRequest {
    Open { group: i32, capacity: usize },
    Invite { group: i32, target: u64 },
    Accept { group: i32 },
    Ready { group: i32, ready: bool },
    Kick { group: i32, target: u64 },
    Quit { group: i32 },
    Launch { group: i32 },
    Cancel { group: i32 },
    EnterInstance { instance: i32 },
}

impl GroupRequest {
    /// ORIGINAL: `(sub-opcode, client class)` for the recovered client-to-server launch requests.
    /// The full 20-row table is owned by `src/restoration/piece_020.rs`; this is the subset the
    /// harness scripts use.
    pub const RECOVERED_TABLE: [(u8, &'static str); 6] = [
        (20, "ClientActivityLaunchPacketInviteMemberRequest"),
        (17, "ClientActivityLaunchPacketInviteResponse"),
        (18, "ClientActivityLaunchPacketMemberReady"),
        (21, "ClientActivityLaunchPacketKickMemberRequest"),
        (19, "ClientActivityLaunchPacketMemberQuit"),
        (22, "ClientActivityLaunchPacketOwnerLaunchRequest"),
    ];

    /// The recovered request behind this intent, or `None` when no original request exists.
    pub fn sub_opcode(&self) -> Option<u8> {
        match self {
            GroupRequest::Invite { .. } => Some(20),
            GroupRequest::Accept { .. } => Some(17),
            GroupRequest::Ready { .. } => Some(18),
            GroupRequest::Kick { .. } => Some(21),
            GroupRequest::Quit { .. } => Some(19),
            GroupRequest::Launch { .. } => Some(22),
            GroupRequest::Open { .. }
            | GroupRequest::Cancel { .. }
            | GroupRequest::EnterInstance { .. } => None,
        }
    }

    /// `u16 family | u8 sub-opcode`, or `None` for the harness-level intents in the gap list.
    pub fn wire_bytes(&self) -> Option<Vec<u8>> {
        self.sub_opcode().map(|sub| {
            let mut bytes = GROUPING_FAMILY.to_le_bytes().to_vec();
            bytes.push(sub);
            bytes
        })
    }

    pub fn label(&self) -> String {
        match self {
            GroupRequest::Open { group, capacity } => format!("group.open({group},{capacity})"),
            GroupRequest::Invite { group, target } => {
                format!("group.invite({group},{target:#018x})")
            }
            GroupRequest::Accept { group } => format!("group.accept({group})"),
            GroupRequest::Ready { group, ready } => format!("group.ready({group},{ready})"),
            GroupRequest::Kick { group, target } => format!("group.kick({group},{target:#018x})"),
            GroupRequest::Quit { group } => format!("group.quit({group})"),
            GroupRequest::Launch { group } => format!("group.launch({group})"),
            GroupRequest::Cancel { group } => format!("group.cancel({group})"),
            GroupRequest::EnterInstance { instance } => format!("group.enter_instance({instance})"),
        }
    }
}

/// One step of a scripted client action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Group(GroupRequest),
    Trade(TradeCommand),
    /// A recovered family/sub-opcode pair with an opaque body.
    Wire {
        family: u16,
        sub: u8,
        body: Vec<u8>,
    },
}

impl Action {
    pub fn label(&self) -> String {
        match self {
            Action::Group(request) => request.label(),
            Action::Trade(command) => match command {
                TradeCommand::InviteReply { session, accepted } => {
                    format!("trade.invite_reply({session},{accepted})")
                }
                TradeCommand::Accept { session, revision } => {
                    format!("trade.accept({session},{revision})")
                }
                TradeCommand::Cancel { session } => format!("trade.cancel({session})"),
                TradeCommand::Lock {
                    session,
                    revision,
                    locked,
                } => format!("trade.lock({session},{revision},{locked})"),
                TradeCommand::OfferItem {
                    session,
                    item,
                    resulting_count,
                } => format!("trade.offer_item({session},{item},{resulting_count})"),
                TradeCommand::CoinCount {
                    session,
                    resulting_coins,
                } => format!("trade.coin_count({session},{resulting_coins})"),
            },
            Action::Wire { family, sub, body } => {
                format!("wire.{family}.{sub}.{}", hex(body))
            }
        }
    }

    /// The recovered wire bytes for this step, when a recovered message exists.
    pub fn wire_bytes(&self) -> Option<Vec<u8>> {
        match self {
            Action::Group(request) => request.wire_bytes(),
            Action::Trade(command) => Some(command.encode()),
            Action::Wire { family, sub, body } => {
                let mut bytes = family.to_le_bytes().to_vec();
                bytes.push(*sub);
                bytes.extend_from_slice(body);
                Some(bytes)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Client {
    pub guid: u64,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub tick: u32,
    pub client: u64,
    pub action: Action,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Case {
    Party,
    Trade,
    Activity,
}

impl Case {
    pub fn name(self) -> &'static str {
        match self {
            Case::Party => "party",
            Case::Trade => "trade",
            Case::Activity => "activity",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scenario {
    pub case: Case,
    /// The seeded first instance id for the activity case; the grouping registry owns the rest.
    pub seed: i32,
    pub clients: Vec<Client>,
    pub steps: Vec<Step>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScenarioError {
    NoClients,
    NoSteps,
    DuplicateClient(u64),
    UnknownClient(u64),
    ScheduleNotIncreasing {
        previous: (u32, u64),
        next: (u32, u64),
    },
}

impl Scenario {
    /// A script is valid only if every step names a known client and the `(tick, client)` keys are
    /// strictly increasing. That removes every hidden ordering rule from a run.
    pub fn validate(&self) -> Result<(), ScenarioError> {
        if self.clients.is_empty() {
            return Err(ScenarioError::NoClients);
        }
        if self.steps.is_empty() {
            return Err(ScenarioError::NoSteps);
        }
        let mut guids = BTreeSet::new();
        for client in &self.clients {
            if !guids.insert(client.guid) {
                return Err(ScenarioError::DuplicateClient(client.guid));
            }
        }
        let mut previous: Option<(u32, u64)> = None;
        for step in &self.steps {
            if !guids.contains(&step.client) {
                return Err(ScenarioError::UnknownClient(step.client));
            }
            let key = (step.tick, step.client);
            if let Some(previous) = previous
                && key <= previous
            {
                return Err(ScenarioError::ScheduleNotIncreasing {
                    previous,
                    next: key,
                });
            }
            previous = Some(key);
        }
        Ok(())
    }
}

/// The three headless acceptance cases. Every script is fixed data: two runs of the same case
/// produce the same sequence of client actions.
pub fn scenario(case: Case) -> Scenario {
    let owner = Client {
        guid: 0x0000_0040_0000_0001,
        name: "Tester One".into(),
    };
    let member = Client {
        guid: 0x0000_0060_0000_0002,
        name: "Tester Two".into(),
    };
    let third = Client {
        guid: 0x0000_0060_0000_0003,
        name: "Tester Three".into(),
    };
    let first_instance = 90_000_001;
    let step = |tick: u32, client: u64, action: Action| Step {
        tick,
        client,
        action,
    };
    let group = |request: GroupRequest| Action::Group(request);
    match case {
        Case::Party => Scenario {
            case,
            seed: first_instance,
            clients: vec![owner.clone(), member.clone()],
            steps: vec![
                step(
                    1,
                    owner.guid,
                    group(GroupRequest::Open {
                        group: 12,
                        capacity: 4,
                    }),
                ),
                step(
                    2,
                    owner.guid,
                    group(GroupRequest::Invite {
                        group: 12,
                        target: member.guid,
                    }),
                ),
                step(3, member.guid, group(GroupRequest::Accept { group: 12 })),
                step(
                    4,
                    member.guid,
                    group(GroupRequest::Ready {
                        group: 12,
                        ready: true,
                    }),
                ),
                step(
                    5,
                    owner.guid,
                    group(GroupRequest::Ready {
                        group: 12,
                        ready: true,
                    }),
                ),
                step(
                    6,
                    owner.guid,
                    group(GroupRequest::Kick {
                        group: 12,
                        target: member.guid,
                    }),
                ),
                step(
                    7,
                    owner.guid,
                    group(GroupRequest::Invite {
                        group: 12,
                        target: member.guid,
                    }),
                ),
                step(8, member.guid, group(GroupRequest::Accept { group: 12 })),
                step(9, member.guid, group(GroupRequest::Quit { group: 12 })),
            ],
        },
        Case::Trade => Scenario {
            case,
            seed: first_instance,
            clients: vec![owner.clone(), member.clone()],
            steps: vec![
                step(
                    1,
                    member.guid,
                    Action::Trade(TradeCommand::InviteReply {
                        session: 7,
                        accepted: true,
                    }),
                ),
                step(
                    2,
                    owner.guid,
                    Action::Trade(TradeCommand::Lock {
                        session: 7,
                        revision: 0,
                        locked: true,
                    }),
                ),
                step(
                    3,
                    member.guid,
                    Action::Trade(TradeCommand::Lock {
                        session: 7,
                        revision: 0,
                        locked: true,
                    }),
                ),
                step(
                    4,
                    owner.guid,
                    Action::Trade(TradeCommand::OfferItem {
                        session: 7,
                        item: 1077,
                        resulting_count: 0,
                    }),
                ),
                step(
                    5,
                    member.guid,
                    Action::Trade(TradeCommand::OfferItem {
                        session: 7,
                        item: 1195,
                        resulting_count: 5,
                    }),
                ),
                step(
                    6,
                    owner.guid,
                    Action::Trade(TradeCommand::Accept {
                        session: 7,
                        revision: 1,
                    }),
                ),
                step(
                    7,
                    member.guid,
                    Action::Trade(TradeCommand::Accept {
                        session: 7,
                        revision: 1,
                    }),
                ),
                step(
                    8,
                    owner.guid,
                    Action::Trade(TradeCommand::Cancel { session: 7 }),
                ),
            ],
        },
        Case::Activity => Scenario {
            case,
            seed: first_instance,
            clients: vec![owner.clone(), member.clone(), third.clone()],
            steps: vec![
                step(
                    1,
                    owner.guid,
                    group(GroupRequest::Open {
                        group: 21,
                        capacity: 4,
                    }),
                ),
                step(
                    2,
                    owner.guid,
                    group(GroupRequest::Invite {
                        group: 21,
                        target: member.guid,
                    }),
                ),
                step(3, member.guid, group(GroupRequest::Accept { group: 21 })),
                step(
                    4,
                    member.guid,
                    group(GroupRequest::Ready {
                        group: 21,
                        ready: true,
                    }),
                ),
                step(
                    5,
                    owner.guid,
                    group(GroupRequest::Ready {
                        group: 21,
                        ready: true,
                    }),
                ),
                step(6, owner.guid, group(GroupRequest::Launch { group: 21 })),
                step(
                    7,
                    third.guid,
                    group(GroupRequest::EnterInstance {
                        instance: first_instance,
                    }),
                ),
                step(8, owner.guid, group(GroupRequest::Cancel { group: 21 })),
            ],
        },
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Accepted,
    Refused,
    Unhandled,
}

impl Verdict {
    pub fn name(self) -> &'static str {
        match self {
            Verdict::Accepted => "accepted",
            Verdict::Refused => "refused",
            Verdict::Unhandled => "unhandled",
        }
    }
}

/// What the server under test did with one step. `state` is the caller's canonical snapshot; the
/// harness never inspects it beyond recording and comparing it.
#[derive(Clone, Debug, PartialEq)]
pub struct Outcome {
    pub verdict: Verdict,
    pub detail: Option<String>,
    pub replies: Vec<Vec<u8>>,
    pub state: Value,
    pub changed: bool,
}

impl Outcome {
    pub fn accepted(state: Value) -> Self {
        Self {
            verdict: Verdict::Accepted,
            detail: None,
            replies: Vec::new(),
            state,
            changed: false,
        }
    }

    pub fn refused(detail: impl Into<String>, state: Value) -> Self {
        Self {
            verdict: Verdict::Refused,
            detail: Some(detail.into()),
            replies: Vec::new(),
            state,
            changed: false,
        }
    }

    pub fn unhandled(state: Value) -> Self {
        Self {
            verdict: Verdict::Unhandled,
            detail: None,
            replies: Vec::new(),
            state,
            changed: false,
        }
    }

    pub fn with_replies(mut self, replies: Vec<Vec<u8>>) -> Self {
        self.replies = replies;
        self
    }

    pub fn with_changed(mut self, changed: bool) -> Self {
        self.changed = changed;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaunchRecord {
    pub harness: &'static str,
    pub case: &'static str,
    pub seed: i32,
    pub clients: Vec<Client>,
    pub steps: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogRecord {
    pub index: usize,
    pub tick: u32,
    pub client: u64,
    pub action: String,
    pub verdict: &'static str,
    pub detail: Option<String>,
    /// Hex of the recovered wire bytes for this step, or `None` when no recovered request exists.
    pub wire: Option<String>,
    pub reply_count: usize,
    pub reply_digest: String,
    pub state_digest: String,
    pub state_changed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Run {
    pub launch: LaunchRecord,
    pub log: Vec<LogRecord>,
    pub state: Value,
}

impl Run {
    /// The canonical run document. Key order and array order are fixed, so the bytes are stable.
    pub fn to_json(&self) -> Value {
        json!({
            "launch": {
                "harness": self.launch.harness,
                "case": self.launch.case,
                "seed": self.launch.seed,
                "clients": self
                    .launch
                    .clients
                    .iter()
                    .map(|client| json!({"guid": client.guid, "name": client.name}))
                    .collect::<Vec<_>>(),
                "steps": self.launch.steps,
            },
            "log": self
                .log
                .iter()
                .map(|record| {
                    json!({
                        "index": record.index,
                        "tick": record.tick,
                        "client": record.client,
                        "action": record.action,
                        "verdict": record.verdict,
                        "detail": record.detail,
                        "wire": record.wire,
                        "reply_count": record.reply_count,
                        "reply_digest": record.reply_digest,
                        "state_digest": record.state_digest,
                        "state_changed": record.state_changed,
                    })
                })
                .collect::<Vec<_>>(),
            "state": self.state,
        })
    }

    pub fn digest(&self) -> String {
        hex(&Sha256::digest(canonical(&self.to_json())))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Comparison {
    Identical {
        digest: String,
    },
    Diverged {
        /// `launch`, `log` or `state`.
        stream: &'static str,
        /// A concrete JSON path such as `log[2].verdict`.
        path: String,
        left: String,
        right: String,
    },
}

/// Run one script in order. The dispatcher is the server under test: it receives each step and
/// returns what happened plus its canonical snapshot. No clock, no threads, no shared state.
pub fn run<F>(scenario: &Scenario, mut dispatch: F) -> Result<Run, ScenarioError>
where
    F: FnMut(&Step) -> Outcome,
{
    scenario.validate()?;
    let mut log = Vec::with_capacity(scenario.steps.len());
    let mut state = Value::Null;
    for (index, step) in scenario.steps.iter().enumerate() {
        let outcome = dispatch(step);
        state = outcome.state.clone();
        log.push(LogRecord {
            index,
            tick: step.tick,
            client: step.client,
            action: step.action.label(),
            verdict: outcome.verdict.name(),
            detail: outcome.detail.clone(),
            wire: step.action.wire_bytes().map(|bytes| hex(&bytes)),
            reply_count: outcome.replies.len(),
            reply_digest: digest_replies(&outcome.replies),
            state_digest: hex(&Sha256::digest(canonical(&outcome.state))),
            state_changed: outcome.changed,
        });
    }
    Ok(Run {
        launch: LaunchRecord {
            harness: HARNESS_VERSION,
            case: scenario.case.name(),
            seed: scenario.seed,
            clients: scenario.clients.clone(),
            steps: scenario.steps.len(),
        },
        log,
        state,
    })
}

/// Compare two runs across all three streams: launch record, ordered log and final state.
pub fn compare(left: &Run, right: &Run) -> Comparison {
    match first_difference(&left.to_json(), &right.to_json(), "") {
        None => Comparison::Identical {
            digest: left.digest(),
        },
        Some((path, left, right)) => Comparison::Diverged {
            stream: stream_of(&path),
            path,
            left,
            right,
        },
    }
}

fn stream_of(path: &str) -> &'static str {
    if path.starts_with("launch") {
        "launch"
    } else if path.starts_with("log") {
        "log"
    } else if path.starts_with("state") {
        "state"
    } else {
        "run"
    }
}

fn first_difference(left: &Value, right: &Value, path: &str) -> Option<(String, String, String)> {
    if left == right {
        return None;
    }
    match (left, right) {
        (Value::Object(left), Value::Object(right)) => {
            let keys: BTreeSet<&String> = left.keys().chain(right.keys()).collect();
            for key in keys {
                let next = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                match (left.get(key), right.get(key)) {
                    (Some(left), Some(right)) => {
                        if let Some(found) = first_difference(left, right, &next) {
                            return Some(found);
                        }
                    }
                    (Some(value), None) => return Some((next, compact(value), "(missing)".into())),
                    (None, Some(value)) => return Some((next, "(missing)".into(), compact(value))),
                    (None, None) => {}
                }
            }
            None
        }
        (Value::Array(left), Value::Array(right)) => {
            for index in 0..left.len().min(right.len()) {
                let next = format!("{path}[{index}]");
                if let Some(found) = first_difference(&left[index], &right[index], &next) {
                    return Some(found);
                }
            }
            if left.len() != right.len() {
                return Some((
                    format!("{path}.length"),
                    left.len().to_string(),
                    right.len().to_string(),
                ));
            }
            None
        }
        (left, right) => Some((path.to_string(), compact(left), compact(right))),
    }
}

fn compact(value: &Value) -> String {
    serde_json::to_string(value).expect("a JSON value serializes")
}

/// Canonical bytes: `serde_json` maps are key ordered, so equal documents have equal bytes.
fn canonical(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).expect("a JSON value serializes")
}

fn digest_replies(replies: &[Vec<u8>]) -> String {
    let mut bytes = Vec::new();
    for reply in replies {
        bytes.extend_from_slice(&(reply.len() as u64).to_le_bytes());
        bytes.extend_from_slice(reply);
    }
    hex(&Sha256::digest(&bytes))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
