//! Piece 072 - starting-area historical coverage: the recovered client-objective
//! contract for the original Hero Journal close micro-tutorial.
//!
//! ORIGINAL facts implemented here (provenance in
//! `notes/Research/Pieces/072 Starting area historical coverage.md`):
//!
//! * `decompiled/client-ui/ScriptsBase.lua` `StoryBook:Hide` (line 14689) hides
//!   the book, re-enables movement and keyboard, and - only while
//!   `Ui.IsInNPXTutorial()` is true - kills the four `FtesNpxBook*` client timers
//!   and emits `Ui.OnObjectiveEvent("onStoryBookClose")` (line 14722, the only
//!   occurrence of that event string in the file). Closing the journal outside
//!   NPX tutorial mode emits nothing.
//! * `Resources/CodeStringMappings.txt` binds the directed-play code strings to
//!   locale ids: `FtesNpxBookExit^5100079^`, `FtesNpxDppCamera{1,2,3}` ->
//!   10258/10259/10260 and `FtesNpxDppMovement{1,2,3}` -> 10261/10262/10263.
//! * Locale rows: 5100079 "Click the X to close the Hero's Journal.", the three
//!   Magic Sphere look prompts and the three obstacle WASD/Space prompts.
//! * `FirstTimeEvent` callbacks: camera prompts play sound definitions
//!   18264/18265/18266, movement prompts 18267/18268/18269; `FtesNpxBookExit`
//!   plays no sound and sets `StoryBook.shouldAllowHide = true`.
//! * Wire shapes: S2C `45/10 UIEvent` =
//!   `u16(45), u8(10), i32 objectiveID, i32 string byte length, string bytes`;
//!   C2S `45/7 ClientComplete` = `u16(45), u8(7), i32 objectiveID`, exactly
//!   7 bytes when the client matches the registered event string. S2C
//!   `45/8 ClientClear` disables the registered objective by id and S2C
//!   `45/9 ClientCompleteFailed` clears its completed flag while it stays
//!   enabled, so the objective can be completed again.
//!
//! RECONSTRUCTED, labelled at each call site: the session-scoped registry, the
//! zone/phase guard, the once-only completion ledger, the local event-name
//! length bound, and the objective id itself. The original Hero Journal
//! objective id is unrecovered, so callers must supply their own explicitly
//! reconstructed id until a capture proves the real one.
//!
//! NOT RECOVERED, deliberately not implemented: the byte layout of the S2C
//! `45/8` and `45/9` messages (only their meaning is recovered, so this module
//! models their state effect and never encodes them), the camera globe and
//! movement barrier bindings, the original directed-play server ordering, and
//! every reward. The recovered prompts are exposed as evidence, not as an
//! approved sequence.
//!
//! This module grants nothing: no items, no coins, no XP and no quest progress.
//! It answers one question - may this exact client completion advance the one
//! registered tutorial objective, exactly once, for this player in this
//! zone/phase.

use std::collections::BTreeMap;

/// `UIEvent` packet family opcode (`u16` on the wire).
pub const UI_EVENT_OPCODE: u16 = 45;
/// S2C registration message type: `45/10 UIEvent`.
pub const UI_EVENT_REGISTER: u8 = 10;
/// C2S client-completion message type: `45/7 ClientComplete`.
pub const UI_EVENT_CLIENT_COMPLETE: u8 = 7;
/// S2C clear message type: `45/8 ClientClear`.
pub const UI_EVENT_CLIENT_CLEAR: u8 = 8;
/// S2C completion-failed message type: `45/9 ClientCompleteFailed`.
pub const UI_EVENT_CLIENT_COMPLETE_FAILED: u8 = 9;
/// Exact byte length of the recovered C2S client-completion packet.
pub const CLIENT_COMPLETE_BYTES: usize = 7;
/// Local safety bound on a registered event name.
///
/// The native `009b2860` reader bounds the string but its exact limit is not
/// recovered, so this is a local guard, not an original constant.
pub const MAX_EVENT_NAME_BYTES: usize = 64;

/// The objective event the original journal emits when the book is closed while
/// the native NPX tutorial flag is set.
pub const STORY_BOOK_CLOSE_EVENT: &str = "onStoryBookClose";
/// Locale id of the recovered journal-close instruction shown to the player.
pub const JOURNAL_CLOSE_INSTRUCTION_LOCALE: u32 = 5100079;
/// The original Hero Journal objective id is not recovered.
///
/// Keeping this as `None` is the point: an id must come from a capture or an
/// approved reconstruction, never from a guess dressed up as history.
pub const STORY_BOOK_CLOSE_OBJECTIVE_ID: Option<i32> = None;

/// Recovered source of the client objective.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServerBinding {
    /// The objective event name is recovered; the numeric objective id is not.
    ObjectiveEventRecovered,
    /// The prompt text and audio are recovered, the server trigger is not.
    Unrecovered,
}

/// Player input the recovered prompt asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputHint {
    CloseJournal,
    LookWithRightMouse,
    Move,
    MoveAndJump,
}

/// One recovered original opening micro-tutorial prompt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TutorialPrompt {
    JournalClose,
    DirectedPlayCamera1,
    DirectedPlayCamera2,
    DirectedPlayCamera3,
    DirectedPlayMovement1,
    DirectedPlayMovement2,
    DirectedPlayMovement3,
}

/// A recovered prompt with its locale text, sound and binding status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OriginalTutorialGate {
    pub prompt: TutorialPrompt,
    /// Original Lua callback / code string name.
    pub callback: &'static str,
    /// Locale id resolved through `CodeStringMappings.txt`.
    pub text_locale_id: u32,
    /// Sound definition id played by the callback, when the callback plays one.
    pub sound_id: Option<u32>,
    pub input_hint: InputHint,
    /// Objective event the client emits for this prompt, when recovered.
    pub objective_event: Option<&'static str>,
    pub server_binding: ServerBinding,
}

/// The seven recovered opening micro-tutorial prompts, in the order the client
/// resource table lists them. The listing order is not recovered server
/// ordering; see [`OriginalTutorialGate::server_binding`] for each gate's state.
pub const ORIGINAL_TUTORIAL_GATES: [OriginalTutorialGate; 7] = [
    OriginalTutorialGate {
        prompt: TutorialPrompt::JournalClose,
        callback: "FtesNpxBookExit",
        text_locale_id: JOURNAL_CLOSE_INSTRUCTION_LOCALE,
        sound_id: None,
        input_hint: InputHint::CloseJournal,
        objective_event: Some(STORY_BOOK_CLOSE_EVENT),
        server_binding: ServerBinding::ObjectiveEventRecovered,
    },
    OriginalTutorialGate {
        prompt: TutorialPrompt::DirectedPlayCamera1,
        callback: "FtesNpxDppCamera1",
        text_locale_id: 10258,
        sound_id: Some(18264),
        input_hint: InputHint::LookWithRightMouse,
        objective_event: None,
        server_binding: ServerBinding::Unrecovered,
    },
    OriginalTutorialGate {
        prompt: TutorialPrompt::DirectedPlayCamera2,
        callback: "FtesNpxDppCamera2",
        text_locale_id: 10259,
        sound_id: Some(18265),
        input_hint: InputHint::LookWithRightMouse,
        objective_event: None,
        server_binding: ServerBinding::Unrecovered,
    },
    OriginalTutorialGate {
        prompt: TutorialPrompt::DirectedPlayCamera3,
        callback: "FtesNpxDppCamera3",
        text_locale_id: 10260,
        sound_id: Some(18266),
        input_hint: InputHint::LookWithRightMouse,
        objective_event: None,
        server_binding: ServerBinding::Unrecovered,
    },
    OriginalTutorialGate {
        prompt: TutorialPrompt::DirectedPlayMovement1,
        callback: "FtesNpxDppMovement1",
        text_locale_id: 10261,
        sound_id: Some(18267),
        input_hint: InputHint::Move,
        objective_event: None,
        server_binding: ServerBinding::Unrecovered,
    },
    OriginalTutorialGate {
        prompt: TutorialPrompt::DirectedPlayMovement2,
        callback: "FtesNpxDppMovement2",
        text_locale_id: 10262,
        sound_id: Some(18268),
        input_hint: InputHint::MoveAndJump,
        objective_event: None,
        server_binding: ServerBinding::Unrecovered,
    },
    OriginalTutorialGate {
        prompt: TutorialPrompt::DirectedPlayMovement3,
        callback: "FtesNpxDppMovement3",
        text_locale_id: 10263,
        sound_id: Some(18269),
        input_hint: InputHint::MoveAndJump,
        objective_event: None,
        server_binding: ServerBinding::Unrecovered,
    },
];

/// The player's current zone and phase. Client objectives are session state, so
/// this is the only scope the recovered completion packet can be judged against.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ZonePhase {
    pub zone_id: u32,
    pub phase: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObjectiveError {
    /// Client-completion packet is not exactly [`CLIENT_COMPLETE_BYTES`] bytes.
    WrongPacketLength(usize),
    /// Opcode or message type is not the recovered `45/7`.
    WrongMessageType,
    /// Objective ids are positive; zero and negatives are never real objectives.
    InvalidObjectiveId(i32),
    EmptyEventName,
    EventNameTooLong,
    /// Event names are printable ASCII; the client matches the exact `IString`.
    InvalidEventName,
    /// A different event is already registered under this id.
    ConflictingRegistration(i32),
    UnknownObjective(i32),
    WrongScope {
        expected: ZonePhase,
        observed: ZonePhase,
    },
    AlreadyCompleted(i32),
    NotCompleted(i32),
}

impl std::fmt::Display for ObjectiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectiveError::WrongPacketLength(len) => {
                write!(
                    f,
                    "client completion must be exactly {CLIENT_COMPLETE_BYTES} bytes, got {len}"
                )
            }
            ObjectiveError::WrongMessageType => write!(f, "not a 45/7 client completion"),
            ObjectiveError::InvalidObjectiveId(id) => write!(f, "invalid objective id {id}"),
            ObjectiveError::EmptyEventName => write!(f, "event name is empty"),
            ObjectiveError::EventNameTooLong => {
                write!(f, "event name exceeds {MAX_EVENT_NAME_BYTES} bytes")
            }
            ObjectiveError::InvalidEventName => write!(f, "event name is not printable ASCII"),
            ObjectiveError::ConflictingRegistration(id) => {
                write!(f, "objective {id} is already registered for another event")
            }
            ObjectiveError::UnknownObjective(id) => write!(f, "objective {id} is not registered"),
            ObjectiveError::WrongScope { expected, observed } => write!(
                f,
                "completion came from zone {} phase {}, expected zone {} phase {}",
                observed.zone_id, observed.phase, expected.zone_id, expected.phase
            ),
            ObjectiveError::AlreadyCompleted(id) => {
                write!(f, "objective {id} is already completed")
            }
            ObjectiveError::NotCompleted(id) => write!(f, "objective {id} is not completed"),
        }
    }
}

impl std::error::Error for ObjectiveError {}

/// One objective the server has registered for this session and scope.
#[derive(Clone, Debug, PartialEq, Eq)]
struct ClientObjective {
    event: String,
    completed: bool,
}

/// One accepted completion, as reported to the caller.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletedObjective {
    pub objective_id: i32,
    /// The registered event string, so the caller can route the transition.
    pub event: String,
}

/// Session-scoped registry of client objectives this server has registered.
///
/// Reconstructed policy, not recovered server code: objectives live for one
/// session and one zone/phase, a completion is accepted once per objective, and
/// neither `45/8` nor `45/9` is re-encoded here because their byte layout is not
/// recovered.
#[derive(Clone, Debug)]
pub struct ClientObjectiveRegistry {
    scope: ZonePhase,
    objectives: BTreeMap<i32, ClientObjective>,
}

impl ClientObjectiveRegistry {
    /// A fresh session starts with nothing registered; this is also the restart
    /// path, because client objectives are not persisted server state.
    pub fn new(scope: ZonePhase) -> Self {
        Self {
            scope,
            objectives: BTreeMap::new(),
        }
    }

    pub fn scope(&self) -> ZonePhase {
        self.scope
    }

    /// Registered objective ids in ascending order.
    pub fn outstanding(&self) -> Vec<i32> {
        self.objectives.keys().copied().collect()
    }

    /// Register one objective for this scope.
    ///
    /// Re-registering the same objective with the same event is a no-op, so a
    /// retried registration packet cannot corrupt the session. A different event
    /// for the same id is refused.
    pub fn register(
        &mut self,
        objective_id: i32,
        event: &str,
        scope: ZonePhase,
    ) -> Result<(), ObjectiveError> {
        validate_objective_id(objective_id)?;
        validate_event_name(event)?;
        if scope != self.scope {
            return Err(ObjectiveError::WrongScope {
                expected: self.scope,
                observed: scope,
            });
        }
        if let Some(existing) = self.objectives.get_mut(&objective_id) {
            if existing.event != event {
                return Err(ObjectiveError::ConflictingRegistration(objective_id));
            }
            return Ok(());
        }
        self.objectives.insert(
            objective_id,
            ClientObjective {
                event: event.to_string(),
                completed: false,
            },
        );
        Ok(())
    }

    /// The `45/10` packet that registers a registered objective.
    pub fn registration_packet(&self, objective_id: i32) -> Result<Vec<u8>, ObjectiveError> {
        let objective = self
            .objectives
            .get(&objective_id)
            .ok_or(ObjectiveError::UnknownObjective(objective_id))?;
        encode_ui_event_registration(objective_id, &objective.event)
    }

    /// Accept the recovered `45/7` client completion.
    ///
    /// The caller passes the player's actual zone/phase instead of trusting the
    /// packet, which carries only the objective id.
    pub fn complete(
        &mut self,
        packet: &[u8],
        actor_scope: ZonePhase,
    ) -> Result<CompletedObjective, ObjectiveError> {
        let objective_id = decode_client_completion(packet)?;
        if actor_scope != self.scope {
            return Err(ObjectiveError::WrongScope {
                expected: self.scope,
                observed: actor_scope,
            });
        }
        let objective = self
            .objectives
            .get_mut(&objective_id)
            .ok_or(ObjectiveError::UnknownObjective(objective_id))?;
        if objective.completed {
            return Err(ObjectiveError::AlreadyCompleted(objective_id));
        }
        objective.completed = true;
        Ok(CompletedObjective {
            objective_id,
            event: objective.event.clone(),
        })
    }

    /// `45/9 ClientCompleteFailed`: clear the completed flag, keep the objective.
    pub fn complete_failed(&mut self, objective_id: i32) -> Result<(), ObjectiveError> {
        let objective = self
            .objectives
            .get_mut(&objective_id)
            .ok_or(ObjectiveError::UnknownObjective(objective_id))?;
        if !objective.completed {
            return Err(ObjectiveError::NotCompleted(objective_id));
        }
        objective.completed = false;
        Ok(())
    }

    /// `45/8 ClientClear`: disable and drop a registered objective.
    pub fn clear(&mut self, objective_id: i32) -> Result<(), ObjectiveError> {
        self.objectives
            .remove(&objective_id)
            .map(|_| ())
            .ok_or(ObjectiveError::UnknownObjective(objective_id))
    }

    /// Zone or phase change: every registered objective goes stale.
    ///
    /// Returns the dropped ids in ascending order so the caller can send the
    /// matching `45/8` clear messages.
    pub fn change_scope(&mut self, scope: ZonePhase) -> Vec<i32> {
        if scope == self.scope {
            return Vec::new();
        }
        let dropped = self.outstanding();
        self.objectives.clear();
        self.scope = scope;
        dropped
    }
}

fn validate_objective_id(objective_id: i32) -> Result<(), ObjectiveError> {
    if objective_id <= 0 {
        return Err(ObjectiveError::InvalidObjectiveId(objective_id));
    }
    Ok(())
}

fn validate_event_name(event: &str) -> Result<(), ObjectiveError> {
    if event.is_empty() {
        return Err(ObjectiveError::EmptyEventName);
    }
    if event.len() > MAX_EVENT_NAME_BYTES {
        return Err(ObjectiveError::EventNameTooLong);
    }
    if !event.bytes().all(|byte| byte.is_ascii_graphic()) {
        return Err(ObjectiveError::InvalidEventName);
    }
    Ok(())
}

/// Recovered S2C `45/10 UIEvent` registration:
/// `u16(45), u8(10), i32 objectiveID, i32 string byte length, string bytes`.
pub fn encode_ui_event_registration(
    objective_id: i32,
    event: &str,
) -> Result<Vec<u8>, ObjectiveError> {
    validate_objective_id(objective_id)?;
    validate_event_name(event)?;
    let bytes = event.as_bytes();
    let mut packet = Vec::with_capacity(2 + 1 + 4 + 4 + bytes.len());
    packet.extend(UI_EVENT_OPCODE.to_le_bytes());
    packet.push(UI_EVENT_REGISTER);
    packet.extend(objective_id.to_le_bytes());
    packet.extend((bytes.len() as i32).to_le_bytes());
    packet.extend(bytes);
    Ok(packet)
}

/// Recovered C2S `45/7 ClientComplete`: `u16(45), u8(7), i32 objectiveID`.
///
/// Anything else - a different opcode or message type, a truncated packet, a
/// trailing byte, or a non-positive id - is refused rather than trimmed. The
/// packet carries no event string, so the caller can never infer which UI action
/// produced it; only the registry can decide whether it counts.
pub fn decode_client_completion(packet: &[u8]) -> Result<i32, ObjectiveError> {
    if packet.len() != CLIENT_COMPLETE_BYTES {
        return Err(ObjectiveError::WrongPacketLength(packet.len()));
    }
    let opcode = u16::from_le_bytes([packet[0], packet[1]]);
    if opcode != UI_EVENT_OPCODE || packet[2] != UI_EVENT_CLIENT_COMPLETE {
        return Err(ObjectiveError::WrongMessageType);
    }
    let objective_id = i32::from_le_bytes([packet[3], packet[4], packet[5], packet[6]]);
    validate_objective_id(objective_id)?;
    Ok(objective_id)
}
