//! Class switching wire - strict request parsing, the recovered failure command, and the
//! preservation certificate for health, gear and abilities (work item 044).
//!
//! ORIGINAL facts implemented here, with the recovered source of each:
//! * command sub-opcodes, from the decompiled original server enum
//!   `lab/decompiled/server/LibSOE/SOE/BaseCommandPackets.cs`:
//!   `CommandPacketSetProfile = 13`, `CommandPacketActivateProfileFailed = 26` and
//!   `CommandPacketSetProfileByItemDefinitionId = 36`, all riding the recovered command family
//!   `BaseCommandPacket = 26` (`BasePackets.cs`).
//! * client-update sub-opcodes, from `BaseClientUpdatePackets.cs`:
//!   `ClientUpdatePacketHitpoints = 1` and `ClientUpdatePacketActivateProfile = 21`, riding
//!   `BaseClientUpdatePacket = 38`.
//! * the supplied client's own constructor immediates in
//!   `lab/decompiled/client/FreeRealms/pseudocode.c`: `FUN_008a28e0` builds
//!   `CommandPacketSetProfile` with `0xd`, `FUN_008a2b20` builds
//!   `CommandPacketSetProfileByItemDefinitionId` with `0x24`, `FUN_008a2ca0` builds
//!   `CommandPacketActivateProfileFailed` with `0x1a`, and `FUN_0091c05c` builds
//!   `ClientUpdatePacketActivateProfile` with `0x15`. The shared base constructor `FUN_008a2820`
//!   stores the command family `0x1a` in `BasePacket` and the sub-opcode in `BaseCommandPacket`,
//!   so a command frame is `[26,0][sub,0][fields...]`.
//! * `FUN_008a2ca0` initialises exactly two `int` fields beyond the base command packet.
//!
//! COMMUNITY-RECOVERED, pinned under `lab/references`:
//! * `Sanctuary/src/Sanctuary.Packet/BaseCommandPacket/CommandPacketSetProfile.cs` reads one
//!   `int32` and then requires `RemainingLength == 0`, so the request payload is exactly four
//!   bytes. `Sanctuary.Packet.Common/Extensions/PacketReaderExtensions.cs` names the same three
//!   command sub-opcodes at the same values.
//! * `Sanctuary.Gateway/Handlers/BaseCommandPacket/CommandPacketSetProfileHandler.cs` is the
//!   recovered flow: a found profile serializes into the activate payload, attachments are sent,
//!   then `Animation = 3001` and `CompositeEffect = 4005`; an absent profile returns before the
//!   active profile is mutated, so there is no state change to roll back.
//! * `Sanctuary.Packet/BaseClientUpdatePacket/ClientUpdatePacketHitpoints.cs` serializes exactly
//!   `[38,0][1,0][current][max]`, which is why a class switch that preserves health emits no
//!   hitpoint frame at all.
//! * `Sanctuary.Packet.Common/ClientPcProfile.cs` keeps `Items` (gear) and the eight-slot ability
//!   list per profile, so gear and the ability bar move with the newly activated profile instead of
//!   being cleared.
//! * `freerealms-wiki/wiki.sqlite`, page "Game Interface": "Quick Job Change/Profile Info: Hitting
//!   the arrows will let you change your job without having to go to the About Me menu."
//!
//! RECONSTRUCTED here (policy, clearly not historical): the refusal choice, the
//! `CommandPacketActivateProfileFailed` payload shape, and the preservation guard itself. The
//! native handler is not recovered, no class-switch frame exists in the lab captures, and no
//! error-code vocabulary is recovered for this command, so the two failure-command fields are the
//! zeroes the recovered client constructor installs rather than an invented reason.
//!
//! Explicit gaps, do not imply these:
//! * which profiles a character may switch into is caller-supplied ownership, not a recovered
//!   rule; this module does not hard-code a class list, a rank gate or a level gate.
//! * the ActivateProfile payload bytes, its attachment list and its animation are built by the
//!   existing `profiles`/`inventory` code; this module only selects the profile and certifies what
//!   was preserved.
//! * `ClientUpdatePacketActivateProfile` may be followed by the equipped-items broadcast and the
//!   toolbar resync on the owned gameplay paths; neither is produced here.
//! * cooldown, in-combat and minigame restrictions on switching are unrecovered and are not
//!   invented; a caller that must refuse those still decides that itself.

/// ORIGINAL: the recovered `BaseCommandPacket` family opcode.
pub const BASE_COMMAND_OPCODE: u16 = 26;
/// ORIGINAL: the recovered `BaseClientUpdatePacket` family opcode.
pub const BASE_CLIENT_UPDATE_OPCODE: u16 = 38;
/// ORIGINAL: `CommandPacketActivateProfileFailed` sub-opcode, from the original server enum.
pub const COMMAND_ACTIVATE_PROFILE_FAILED: u16 = 26;
/// ORIGINAL: `ClientUpdatePacketActivateProfile` sub-opcode.
pub const CLIENT_UPDATE_ACTIVATE_PROFILE: u16 = 21;
/// ORIGINAL: `ClientUpdatePacketHitpoints` sub-opcode; the frame a class switch must not emit.
pub const CLIENT_UPDATE_HITPOINTS: u16 = 1;
/// COMMUNITY-RECOVERED: `emo_outfit_all`, the animation the switch feedback plays.
pub const ACTIVATE_PROFILE_ANIMATION: i32 = 3001;
/// COMMUNITY-RECOVERED: `PFX_Job_Swirl`, the composite effect the switch feedback plays.
pub const ACTIVATE_PROFILE_COMPOSITE_EFFECT: i32 = 4005;
/// COMMUNITY-RECOVERED: `ClientPcProfile.Abilities` is an eight-slot list.
pub const PROFILE_ABILITY_SLOTS: usize = 8;

/// ORIGINAL: the recovered command sub-opcodes this slice understands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// `CommandPacketSetProfile`: the client asks for a played profile.
    SetProfile,
    /// `CommandPacketSetProfileByItemDefinitionId`: the same intent keyed by a job item.
    SetProfileByItemDefinitionId,
}

impl Command {
    pub const fn wire(self) -> u16 {
        match self {
            Self::SetProfile => 13,
            Self::SetProfileByItemDefinitionId => 36,
        }
    }
}

/// ORIGINAL: the recovered client-update sub-opcodes this slice names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientUpdate {
    /// `ClientUpdatePacketHitpoints`, `[38,0][1,0][current][max]`.
    Hitpoints,
    /// `ClientUpdatePacketActivateProfile`, `[38,0][21,0][payload len][payload]...`.
    ActivateProfile,
}

impl ClientUpdate {
    pub const fn wire(self) -> u16 {
        match self {
            Self::Hitpoints => CLIENT_UPDATE_HITPOINTS,
            Self::ActivateProfile => CLIENT_UPDATE_ACTIVATE_PROFILE,
        }
    }
}

/// RECONSTRUCTED: the recovered failure command. The original enum pins the sub-opcode and the
/// supplied client's constructor pins two `int` fields; their meaning is not recovered, so both
/// are emitted as the zeroes that constructor installs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommandPacketActivateProfileFailed {
    /// The profile the client asked for, echoed so the client can drop the queued switch.
    pub profile_id: i32,
    /// Unrecovered; the client constructor's default, deliberately not an invented reason.
    pub reason: i32,
}

impl CommandPacketActivateProfileFailed {
    /// ORIGINAL: the recovered sub-opcode.
    pub const WIRE: u16 = COMMAND_ACTIVATE_PROFILE_FAILED;

    /// `[26,0][26,0][profile][reason]`.
    pub fn packet(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(12);
        bytes.extend(BASE_COMMAND_OPCODE.to_le_bytes());
        bytes.extend(Self::WIRE.to_le_bytes());
        bytes.extend(self.profile_id.to_le_bytes());
        bytes.extend(self.reason.to_le_bytes());
        bytes
    }
}

/// The decoded `CommandPacketSetProfile` request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SwitchRequest {
    /// ORIGINAL: the single `int32` the recovered reader consumes.
    pub profile_id: i32,
}

/// Why a switch request was rejected before any state was touched.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwitchRequestError {
    /// The frame is not part of the recovered command family.
    WrongCommandFamily,
    /// The frame is a different command in that family.
    WrongSubCommand,
    /// The recovered payload is exactly one `int32`; anything else is not this request.
    WrongLength,
    /// `ClientPcProfile.Id` values in the recovered content are positive.
    NonPositiveProfile,
}

/// Why a well-formed request was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileRefusal {
    /// The character does not own the requested profile.
    NotOwned,
}

impl ProfileRefusal {
    /// The recovered failure command for this refusal. The requested profile is echoed and the
    /// second field stays the client constructor's default, so no unrecovered reason is invented.
    pub fn packet(self, requested_profile_id: i32) -> Vec<u8> {
        let _ = self;
        CommandPacketActivateProfileFailed {
            profile_id: requested_profile_id,
            reason: 0,
        }
        .packet()
    }
}

/// The caller's decision after decoding a request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwitchOutcome {
    /// The caller should build the activate payload for this profile and send `[38,0][21,0]...`.
    Activate { profile_id: i32 },
    /// The caller should send the recovered failure command and change nothing else.
    Refuse(ProfileRefusal),
}

impl SwitchOutcome {
    /// An activation profile exists only when the caller should build activate feedback; a refusal
    /// hands back nothing, so a caller cannot mistake a refusal for a switch.
    pub const fn activated(&self) -> Option<i32> {
        match self {
            Self::Activate { profile_id } => Some(*profile_id),
            Self::Refuse(_) => None,
        }
    }
}

/// Parses `[26,0][13,0][int32 profile]`, the recovered shape of a class-switch request. Anything
/// else is an error rather than a guess, so a malformed frame can never be answered with a silent
/// activation.
pub fn decode_switch_request(bytes: &[u8]) -> Result<SwitchRequest, SwitchRequestError> {
    if bytes.len() < 2 {
        return Err(SwitchRequestError::WrongLength);
    }
    if u16::from_le_bytes([bytes[0], bytes[1]]) != BASE_COMMAND_OPCODE {
        return Err(SwitchRequestError::WrongCommandFamily);
    }
    if bytes.len() < 4 {
        return Err(SwitchRequestError::WrongLength);
    }
    if u16::from_le_bytes([bytes[2], bytes[3]]) != Command::SetProfile.wire() {
        return Err(SwitchRequestError::WrongSubCommand);
    }
    if bytes.len() != 8 {
        return Err(SwitchRequestError::WrongLength);
    }
    let profile_id = i32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    if profile_id <= 0 {
        return Err(SwitchRequestError::NonPositiveProfile);
    }
    Ok(SwitchRequest { profile_id })
}

/// Decides a switch without touching any state. `owned_profiles` is the caller's ownership set,
/// which this module deliberately does not infer, cache or widen.
pub fn decide_switch(
    bytes: &[u8],
    owned_profiles: &[i32],
) -> Result<SwitchOutcome, SwitchRequestError> {
    let request = decode_switch_request(bytes)?;
    if !owned_profiles.contains(&request.profile_id) {
        return Ok(SwitchOutcome::Refuse(ProfileRefusal::NotOwned));
    }
    Ok(SwitchOutcome::Activate {
        profile_id: request.profile_id,
    })
}

/// The three values the acceptance conditions require a switch to carry through unchanged,
/// alongside the profile that is now active. The caller samples these before and after its own
/// activation so the claim is checked against real state rather than asserted here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreservedValues {
    /// Current hitpoints.
    pub health: u32,
    /// The profile that is active.
    pub active_profile_id: i32,
    /// Equipped item definition ids, in slot order.
    pub equipment_definition_ids: Vec<i32>,
    /// Size of the ability set; the recovered `ClientPcProfile` list is eight slots.
    pub ability_count: usize,
}

/// A checked claim that health, gear and the ability set survived the switch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Preservation {
    /// The profile that became active.
    pub profile_id: i32,
    /// Carried through unchanged.
    pub health: u32,
    /// Carried through unchanged.
    pub equipment_definition_ids: Vec<i32>,
    /// Carried through unchanged.
    pub ability_count: usize,
}

/// Which preservation condition a completed switch broke.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreserveError {
    /// Current hitpoints moved; a switch must not heal, damage or respec the player.
    HealthChanged,
    /// The equipped set moved; gear belongs to the profiles, not to the switch itself.
    EquipmentChanged,
    /// The ability set moved; the toolbar is resynced from the newly active profile.
    AbilitiesChanged,
    /// The same profile is still active, so nothing was switched.
    ProfileUnchanged,
}

/// Checks a completed switch. Every field except the active profile must be identical, and the
/// active profile must actually have moved.
pub fn preserve(
    before: &PreservedValues,
    after: &PreservedValues,
) -> Result<Preservation, PreserveError> {
    if after.active_profile_id == before.active_profile_id {
        return Err(PreserveError::ProfileUnchanged);
    }
    if after.health != before.health {
        return Err(PreserveError::HealthChanged);
    }
    if after.equipment_definition_ids != before.equipment_definition_ids {
        return Err(PreserveError::EquipmentChanged);
    }
    if after.ability_count != before.ability_count {
        return Err(PreserveError::AbilitiesChanged);
    }
    Ok(Preservation {
        profile_id: after.active_profile_id,
        health: after.health,
        equipment_definition_ids: after.equipment_definition_ids.clone(),
        ability_count: after.ability_count,
    })
}
