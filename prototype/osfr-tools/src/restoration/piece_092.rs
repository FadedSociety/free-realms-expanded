//! Work item 092: mounts and vehicles.
//!
//! The module carries the recovered mount base packet surface (`OpCode` 168), the captured
//! character's mount rows, the mount speed/glide stat block and a bounded session contract that
//! gates spawning on ownership. It grants nothing and spawns no actor: the actor packets that
//! actually create and remove the visible mount are listed as integration boundaries below.
//!
//! ORIGINAL facts recovered from the pinned corpus (paths, hashes and `rg` anchors in
//! `notes/Research/Pieces/092 Mounts and vehicles.md`):
//! * `MountBasePacket` outer opcode `0xa8` (168) with the sub-opcode held in its second field
//!   (`FUN_0089f9c0` in the native client pseudocode). The native constructors register sub-opcodes
//!   3 (`PacketDismountRequest`), 6 (`PacketMountSpawn`), 8 (`PacketMountSpawnByItemDefinitionId`),
//!   9 (`PacketSetAutomount`), 10 (`PacketMountStartTrial`), 11 (`PacketMountEndTrial`),
//!   13 (`PacketMountTrialFailed`), 14 (`PacketMountUpgradeRequest`), 15
//!   (`PacketMountUpgradeResponse`), 2 (`PacketMountResponse`), 4 (`PacketDismountResponse`) and
//!   5 (`PacketMountList`); 1 is `PacketMountRequest`.
//! * The client Lua (`ScriptsBase.lua`) drives the surface: `Mount.Spawn(b)` sends the selected
//!   mount-list id, `Mount.Dismount()` sends the dismount request, and the dock disables the mount
//!   button inside a minigame or overworld combat (`GameDock:SetMountButtonEnabledState`).
//! * The captured self packet carries 34 `Mounts` rows: number, name id, icon id, guid,
//!   members-only, tint id, tint alias, fly-training, able-to-fly, model id and item definition id.
//! * `Mounts.json` row 13459 "The Fire Hawk" (`NameId` 442674, `ModelId` 4590, `ImageSetId` 8556)
//!   carries the nine glide values, and item 78397 binds to it through `Param1`.
//! * The stat block is `BaseClientUpdatePacket` 38 sub-opcode 7: actor guid, `i32` entry count,
//!   then per entry `i32` stat id, `i32` value kind (0 integer, 1 float) and the typed value.
//!
//! COMMUNITY reconstruction (pinned servers, not original binaries): the two server field orders
//! this module encodes, `MountBasePacket` 168/2 and 168/4, are identical in the supplied OSFR
//! server (`Gateway.NPC.MountSystem`) and in the pinned Sanctuary sources
//! (`Sanctuary.Packet.MountBasePacket`). The 12.5 / 8 / 2 / 18 / 0.75 / 4 glide numbers and the
//! Fire Hawk overrides appear in both.
//!
//! RECONSTRUCTED policy (labelled, local - not historical behaviour):
//! * `MountSession` refuses a spawn for a mount the captured rows do not own, replaces the current
//!   mount when a second spawn arrives and reports a dismount with no mount as a no-op. The two
//!   pinned servers both dismount before spawning; which response the original server sent for a
//!   duplicate spawn is not recovered.
//! * `item_bound_mount` resolves a duplicate item definition with the last captured row, mirroring
//!   Sanctuary's `LastOrDefault`. Rows 1002 and 1017 share item 11038, so the tie-break is load
//!   bearing today; the original server's rule is not recovered.
//! * `mount_stats` follows the OSFR branch order: Fire Hawk name id first, then able-to-fly, then
//!   mount 1023's unexplained zero-speed block, then the 12.5 default. The 1023 branch is a
//!   recovered server literal with no recovered meaning.
//!
//! NOT RECOVERED, deliberately absent: the mount list packet body (`PacketMountRequest` 1 and
//! `SetAutomount` 9 payloads), the trial/upgrade payloads, actor-spawn geometry, mount physics,
//! and any price or ownership-grant path.

pub mod sub_opcode {
    pub const MOUNT_REQUEST: u8 = 1;
    pub const MOUNT_RESPONSE: u8 = 2;
    pub const DISMOUNT_REQUEST: u8 = 3;
    pub const DISMOUNT_RESPONSE: u8 = 4;
    pub const MOUNT_LIST: u8 = 5;
    pub const MOUNT_SPAWN: u8 = 6;
    pub const MOUNT_SPAWN_BY_ITEM_DEFINITION_ID: u8 = 8;
    pub const SET_AUTOMOUNT: u8 = 9;
    pub const MOUNT_START_TRIAL: u8 = 10;
    pub const MOUNT_END_TRIAL: u8 = 11;
    pub const MOUNT_TRIAL_FAILED: u8 = 13;
    pub const MOUNT_UPGRADE_REQUEST: u8 = 14;
    pub const MOUNT_UPGRADE_RESPONSE: u8 = 15;
}

/// Tunneled inner opcode of the mount packet family (`MountBasePacket`, `0xa8`).
pub const MOUNT_BASE_OPCODE: u16 = 168;
/// Tunneled inner opcode of the client update family (`BaseClientUpdatePacket`).
pub const CLIENT_UPDATE_OPCODE: u16 = 38;
/// Tunneled inner opcode of the player update family (`BasePlayerUpdatePacket`).
pub const PLAYER_UPDATE_OPCODE: u16 = 35;
/// Sub-opcode of the character stat update inside the client update family.
pub const CLIENT_UPDATE_STAT_SUBOPCODE: u16 = 7;
/// Sub-opcode of `PlayerUpdatePacketAddNpc`, the actor packet that makes the mount visible.
pub const PLAYER_UPDATE_ADD_NPC_SUBOPCODE: u16 = 2;
/// Sub-opcode of the actor removal packet the dismount response pairs with.
pub const PLAYER_UPDATE_REMOVE_NPC_SUBOPCODE: u16 = 3;
/// Composite effect the pinned servers stamp on a mount response (Sanctuary: PFX_Teleport_Flash).
pub const MOUNT_TELEPORT_EFFECT_ID: i32 = 46;

/// Recovered character stat ids the mount stat block uses (`CharacterStatId`).
pub const MAX_MOVEMENT_SPEED_STAT: i32 = 2;
pub const GLIDE_DEFAULT_FORWARD_SPEED_STAT: i32 = 59;
pub const GLIDE_MIN_FORWARD_SPEED_STAT: i32 = 60;
pub const GLIDE_MAX_FORWARD_SPEED_STAT: i32 = 61;
pub const GLIDE_FALL_TIME_STAT: i32 = 62;
pub const GLIDE_FALL_SPEED_STAT: i32 = 63;
pub const GLIDE_ENABLED_STAT: i32 = 64;
pub const GLIDE_ACCEL_STAT: i32 = 67;
pub const JUMP_HEIGHT_STAT: i32 = 68;

/// The speed the dismount response restores: the pinned servers write 8.0 back to on-foot speed.
pub const DISMOUNT_SPEED: f32 = 8.0;
/// Default mounted speed written by both pinned servers and by every landed `Mounts.json` row.
pub const DEFAULT_MOUNT_SPEED: f32 = 12.5;
/// `Mounts.json` row 13459 and the captured mount row that resolve to this name id.
pub const FIRE_HAWK_NAME_ID: i32 = 442674;
/// Captured mount number of The Fire Hawk.
pub const FIRE_HAWK_MOUNT_NUMBER: i32 = 1030;
/// Item definition that binds to The Fire Hawk row through `Param1` (item 78397).
pub const FIRE_HAWK_ITEM_DEFINITION_ID: i32 = 78397;
/// OSFR's unexplained branch: this mount number receives a zero-speed block and nothing else.
pub const ZERO_SPEED_MOUNT_NUMBER: i32 = 1023;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StatValue {
    Int(i32),
    Float(f32),
}

impl StatValue {
    /// Recovered wire kind: 0 serializes an `i32`, 1 serializes an `f32`.
    pub const fn kind(self) -> i32 {
        match self {
            StatValue::Int(_) => 0,
            StatValue::Float(_) => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MountStat {
    pub id: i32,
    pub value: StatValue,
}

const fn float_stat(id: i32, value: f32) -> MountStat {
    MountStat {
        id,
        value: StatValue::Float(value),
    }
}

const fn int_stat(id: i32, value: i32) -> MountStat {
    MountStat {
        id,
        value: StatValue::Int(value),
    }
}

/// One captured `Mounts` row from the supplied character packet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MountOwnership {
    pub number: i32,
    pub name_id: i32,
    pub icon_id: i32,
    pub model_id: i32,
    pub item_definition_id: i32,
    pub tint_id: i32,
    pub tint_alias: &'static str,
    pub members_only: bool,
    pub fly_training: bool,
    pub able_to_fly: bool,
}

const fn mount(
    number: i32,
    name_id: i32,
    icon_id: i32,
    model_id: i32,
    item_definition_id: i32,
    tint_id: i32,
    tint_alias: &'static str,
    members_only: bool,
    fly_training: bool,
    able_to_fly: bool,
) -> MountOwnership {
    MountOwnership {
        number,
        name_id,
        icon_id,
        model_id,
        item_definition_id,
        tint_id,
        tint_alias,
        members_only,
        fly_training,
        able_to_fly,
    }
}

/// The 34 `Mounts` rows of the captured character, in capture order.
pub const CAPTURED_MOUNTS: [MountOwnership; 34] = [
    mount(
        1033, 9911, 8035, 1529, 81256176, 0, "dyetint", false, false, false,
    ),
    mount(
        1032, 416632, 6939, 3338, 163632473, 0, "dyetint", false, false, false,
    ),
    mount(
        1031, 646, 7799, 728, 35463452, 0, "dyetint", false, false, false,
    ),
    mount(1030, 442674, 8556, 4590, 78397, 0, "", false, false, true),
    mount(1029, 385, 7700, 3882, 11763, 0, "", false, false, false),
    mount(
        1028, 852, 7928, 3989, 17598, 249, "dyetint", false, false, false,
    ),
    mount(
        1027, 378, 7665, 3870, 11584, 553, "dyetint", false, false, false,
    ),
    mount(
        1026, 374, 6873, 3849, 14289, 249, "dyetint", false, false, false,
    ),
    mount(
        1025, 372, 6937, 3848, 14288, 249, "dyetint", false, false, true,
    ),
    mount(
        1024, 435557, 7484, 3782, 78070, 0, "dyetint", false, false, true,
    ),
    mount(
        1023, 442689, 8456, 4439, 6346231, 541, "dyetint", false, false, false,
    ),
    mount(1022, 435431, 7401, 3776, 78036, 0, "", false, false, false),
    mount(
        1021, 6764, 6977, 3384, 76759, 249, "dyetint", false, false, false,
    ),
    mount(
        1020, 6966, 7296, 3478, 76903, 249, "dyetint", false, false, false,
    ),
    mount(
        1019, 434170, 7056, 3480, 76905, 0, "dyetint", false, true, false,
    ),
    mount(
        1018, 6806, 7015, 3409, 11317, 0, "dyetint", false, false, false,
    ),
    mount(
        1017, 4897, 6294, 2064, 11038, 246, "dyetint", true, true, false,
    ),
    mount(
        1016, 22959, 6978, 3376, 17179, 117, "snow", false, false, false,
    ),
    mount(1015, 6605, 6922, 3314, 10424, 0, "", false, false, false),
    mount(
        1014, 6403, 6838, 3209, 13686, 0, "sapphire", false, false, false,
    ),
    mount(1013, 6230, 6820, 3162, 10381, 0, "", false, false, false),
    mount(
        1012, 6408, 6854, 3210, 14316, 0, "skyblue", false, false, false,
    ),
    mount(
        1011, 6305, 6821, 3208, 16376, 0, "snow", false, false, false,
    ),
    mount(
        1010, 6369, 6825, 3199, 76906, 0, "blizzard", false, false, false,
    ),
    mount(1009, 6231, 6802, 3163, 10382, 0, "", false, false, false),
    mount(
        1008, 5635, 6658, 3033, 15738, 0, "skyblue", false, false, false,
    ),
    mount(
        1007, 6182, 6703, 3026, 15902, 0, "pumpkin", false, false, false,
    ),
    mount(
        1006, 6847, 6459, 503, 10549, 0, "skyblue", false, false, false,
    ),
    mount(1005, 5218, 6457, 522, 7975, 0, "", false, false, false),
    mount(1004, 5219, 6456, 523, 7976, 0, "", false, false, false),
    mount(
        1003, 4831, 6355, 2108, 16167, 0, "deepred", false, false, false,
    ),
    mount(
        1002,
        4897,
        6294,
        2064,
        11038,
        0,
        "scales-blue",
        false,
        false,
        true,
    ),
    mount(
        1001,
        4893,
        6075,
        2064,
        7954,
        0,
        "goldtips-black",
        false,
        true,
        false,
    ),
    mount(
        1000,
        5570,
        6074,
        2040,
        7942,
        0,
        "trexbasic-blue",
        false,
        false,
        false,
    ),
];

/// The captured ownership row for `number`, if the character owns it.
pub fn captured_mount(number: i32) -> Option<&'static MountOwnership> {
    CAPTURED_MOUNTS.iter().find(|row| row.number == number)
}

/// The captured row bound to an item definition id, using the last match in capture order.
///
/// RECONSTRUCTED tie-break: Sanctuary resolves the same way (`LastOrDefault`); the original rule
/// is unrecovered and rows 1002/1017 share item 11038 today.
pub fn item_bound_mount(item_definition_id: i32) -> Option<&'static MountOwnership> {
    CAPTURED_MOUNTS
        .iter()
        .filter(|row| row.item_definition_id == item_definition_id)
        .next_back()
}

/// Structure checks over the captured rows. Corpus cross-checks live in the private recovery checker `piece_092.py`.
pub fn validate() -> Result<(), &'static str> {
    let mut seen = [0i32; CAPTURED_MOUNTS.len()];
    for (index, row) in CAPTURED_MOUNTS.iter().enumerate() {
        if row.number <= 0 {
            return Err("captured mount numbers must be positive");
        }
        if row.name_id <= 0 || row.icon_id <= 0 || row.model_id <= 0 || row.item_definition_id <= 0
        {
            return Err("captured mount rows must carry positive ids");
        }
        if seen[..index].contains(&row.number) {
            return Err("captured mount numbers must be unique");
        }
        seen[index] = row.number;
    }
    if CAPTURED_MOUNTS
        .iter()
        .all(|row| row.name_id != FIRE_HAWK_NAME_ID)
    {
        return Err("the Fire Hawk name id must appear in the captured rows");
    }
    Ok(())
}

/// A decoded client-to-server mount base request (bytes after the tunneled inner opcode).
#[derive(Clone, Debug, PartialEq)]
pub enum MountClientRequest {
    /// Sub-opcode 6: the client selected a mount-list entry.
    MountSpawn { mount_number: i32 },
    /// Sub-opcode 8: the Lua/automount path named an item definition instead of a mount number.
    MountSpawnByItemDefinitionId {
        item_definition_id: i32,
        unknown: bool,
    },
    /// Sub-opcode 3: no payload.
    DismountRequest,
    /// A client-to-server sub-opcode whose body is not recovered.
    Unrecovered { sub_opcode: u8, payload: Vec<u8> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MountRequestError {
    Empty,
    Truncated,
    TrailingBytes { expected: usize, got: usize },
    UnknownSubOpcode(u8),
    InvalidMountNumber(i32),
}

/// Decode a mount base request. `bytes` starts at the sub-opcode byte: the gateway has already
/// dispatched on `MOUNT_BASE_OPCODE`.
pub fn decode_client_request(bytes: &[u8]) -> Result<MountClientRequest, MountRequestError> {
    let (sub_opcode, payload) = bytes.split_first().ok_or(MountRequestError::Empty)?;
    match *sub_opcode {
        sub_opcode::MOUNT_SPAWN => {
            let mount_number = read_i32(payload)?;
            if mount_number <= 0 {
                return Err(MountRequestError::InvalidMountNumber(mount_number));
            }
            Ok(MountClientRequest::MountSpawn { mount_number })
        }
        sub_opcode::MOUNT_SPAWN_BY_ITEM_DEFINITION_ID => {
            if payload.len() < 5 {
                return Err(MountRequestError::Truncated);
            }
            if payload.len() > 5 {
                return Err(MountRequestError::TrailingBytes {
                    expected: 5,
                    got: payload.len(),
                });
            }
            let item_definition_id =
                i32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
            Ok(MountClientRequest::MountSpawnByItemDefinitionId {
                item_definition_id,
                unknown: payload[4] != 0,
            })
        }
        sub_opcode::DISMOUNT_REQUEST => {
            if !payload.is_empty() {
                return Err(MountRequestError::TrailingBytes {
                    expected: 0,
                    got: payload.len(),
                });
            }
            Ok(MountClientRequest::DismountRequest)
        }
        other if is_recovered_sub_opcode(other) => Ok(MountClientRequest::Unrecovered {
            sub_opcode: other,
            payload: payload.to_vec(),
        }),
        other => Err(MountRequestError::UnknownSubOpcode(other)),
    }
}

const fn is_recovered_sub_opcode(sub_opcode: u8) -> bool {
    matches!(
        sub_opcode,
        sub_opcode::MOUNT_REQUEST
            | sub_opcode::MOUNT_RESPONSE
            | sub_opcode::DISMOUNT_RESPONSE
            | sub_opcode::MOUNT_LIST
            | sub_opcode::SET_AUTOMOUNT
            | sub_opcode::MOUNT_START_TRIAL
            | sub_opcode::MOUNT_END_TRIAL
            | sub_opcode::MOUNT_TRIAL_FAILED
            | sub_opcode::MOUNT_UPGRADE_REQUEST
            | sub_opcode::MOUNT_UPGRADE_RESPONSE
    )
}

fn read_i32(payload: &[u8]) -> Result<i32, MountRequestError> {
    if payload.len() < 4 {
        return Err(MountRequestError::Truncated);
    }
    if payload.len() > 4 {
        return Err(MountRequestError::TrailingBytes {
            expected: 4,
            got: payload.len(),
        });
    }
    Ok(i32::from_le_bytes([
        payload[0], payload[1], payload[2], payload[3],
    ]))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MountRejection {
    /// The captured rows do not contain this mount number.
    UnownedMount(i32),
    /// No captured row is bound to this item definition id.
    UnknownItemDefinition(i32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MountTransition {
    Mounted { number: i32, replaced: Option<i32> },
    Dismounted { number: i32 },
    IgnoredNotMounted,
    Rejected(MountRejection),
    Unsupported { sub_opcode: u8 },
}

/// The bounded session contract: which mount is out, and what a request does to it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MountSession {
    mounted: Option<i32>,
}

impl MountSession {
    pub const fn new() -> Self {
        Self { mounted: None }
    }

    pub const fn mounted_mount(&self) -> Option<i32> {
        self.mounted
    }

    pub fn apply(&mut self, request: MountClientRequest) -> MountTransition {
        match request {
            MountClientRequest::MountSpawn { mount_number } => self.spawn(
                captured_mount(mount_number),
                MountRejection::UnownedMount(mount_number),
            ),
            MountClientRequest::MountSpawnByItemDefinitionId {
                item_definition_id, ..
            } => self.spawn(
                item_bound_mount(item_definition_id),
                MountRejection::UnknownItemDefinition(item_definition_id),
            ),
            MountClientRequest::DismountRequest => match self.mounted.take() {
                Some(number) => MountTransition::Dismounted { number },
                None => MountTransition::IgnoredNotMounted,
            },
            MountClientRequest::Unrecovered { sub_opcode, .. } => {
                MountTransition::Unsupported { sub_opcode }
            }
        }
    }

    fn spawn(
        &mut self,
        row: Option<&'static MountOwnership>,
        rejection: MountRejection,
    ) -> MountTransition {
        let Some(row) = row else {
            return MountTransition::Rejected(rejection);
        };
        let replaced = self.mounted.replace(row.number);
        MountTransition::Mounted {
            number: row.number,
            replaced,
        }
    }
}

/// The stat block a mount applies on spawn, exactly as the OSFR branch order writes it.
pub fn mount_stats(row: &MountOwnership) -> Vec<MountStat> {
    if row.name_id == FIRE_HAWK_NAME_ID {
        return vec![
            float_stat(MAX_MOVEMENT_SPEED_STAT, DEFAULT_MOUNT_SPEED),
            float_stat(GLIDE_DEFAULT_FORWARD_SPEED_STAT, 12.0),
            float_stat(GLIDE_MIN_FORWARD_SPEED_STAT, 6.0),
            float_stat(GLIDE_MAX_FORWARD_SPEED_STAT, 22.0),
            float_stat(GLIDE_FALL_TIME_STAT, 0.75),
            float_stat(GLIDE_FALL_SPEED_STAT, 3.0),
            int_stat(GLIDE_ENABLED_STAT, 1),
            float_stat(GLIDE_ACCEL_STAT, 4.0),
            float_stat(JUMP_HEIGHT_STAT, 5.0),
        ];
    }
    if row.able_to_fly {
        return vec![
            float_stat(MAX_MOVEMENT_SPEED_STAT, DEFAULT_MOUNT_SPEED),
            float_stat(GLIDE_DEFAULT_FORWARD_SPEED_STAT, 8.0),
            float_stat(GLIDE_MIN_FORWARD_SPEED_STAT, 2.0),
            float_stat(GLIDE_MAX_FORWARD_SPEED_STAT, 18.0),
            float_stat(GLIDE_FALL_TIME_STAT, 0.75),
            float_stat(GLIDE_FALL_SPEED_STAT, 4.0),
            int_stat(GLIDE_ENABLED_STAT, 1),
            float_stat(GLIDE_ACCEL_STAT, 4.0),
        ];
    }
    if row.number == ZERO_SPEED_MOUNT_NUMBER {
        return vec![float_stat(MAX_MOVEMENT_SPEED_STAT, 0.0)];
    }
    vec![float_stat(MAX_MOVEMENT_SPEED_STAT, DEFAULT_MOUNT_SPEED)]
}

/// The stat block the dismount response applies: on-foot speed, glide off, jump reset.
pub fn dismount_stats() -> Vec<MountStat> {
    vec![
        float_stat(MAX_MOVEMENT_SPEED_STAT, DISMOUNT_SPEED),
        int_stat(GLIDE_ENABLED_STAT, 0),
        float_stat(JUMP_HEIGHT_STAT, 0.0),
    ]
}

/// Encode a client update stat block (opcode 38, sub-opcode 7).
pub fn encode_stat_update(actor_guid: u64, stats: &[MountStat]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(16 + stats.len() * 12);
    bytes.extend_from_slice(&CLIENT_UPDATE_OPCODE.to_le_bytes());
    bytes.extend_from_slice(&CLIENT_UPDATE_STAT_SUBOPCODE.to_le_bytes());
    bytes.extend_from_slice(&actor_guid.to_le_bytes());
    bytes.extend_from_slice(&(stats.len() as i32).to_le_bytes());
    for stat in stats {
        bytes.extend_from_slice(&stat.id.to_le_bytes());
        bytes.extend_from_slice(&stat.value.kind().to_le_bytes());
        match stat.value {
            StatValue::Int(value) => bytes.extend_from_slice(&value.to_le_bytes()),
            StatValue::Float(value) => bytes.extend_from_slice(&value.to_le_bytes()),
        }
    }
    bytes
}

/// Encode the mount response (opcode 168, sub-opcode 2) in the recovered field order.
pub fn encode_mount_response(
    rider_guid: u64,
    mount_guid: u64,
    seat: i32,
    queue_position: i32,
    unknown: i32,
    composite_effect_id: i32,
    name_vertical_offset: f32,
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(33);
    bytes.extend_from_slice(&MOUNT_BASE_OPCODE.to_le_bytes());
    bytes.push(sub_opcode::MOUNT_RESPONSE);
    bytes.extend_from_slice(&rider_guid.to_le_bytes());
    bytes.extend_from_slice(&mount_guid.to_le_bytes());
    bytes.extend_from_slice(&seat.to_le_bytes());
    bytes.extend_from_slice(&queue_position.to_le_bytes());
    bytes.extend_from_slice(&unknown.to_le_bytes());
    bytes.extend_from_slice(&composite_effect_id.to_le_bytes());
    bytes.extend_from_slice(&name_vertical_offset.to_le_bytes());
    bytes
}

/// Encode the dismount response (opcode 168, sub-opcode 4) in the recovered field order.
pub fn encode_dismount_response(rider_guid: u64, composite_effect_id: i32) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(15);
    bytes.extend_from_slice(&MOUNT_BASE_OPCODE.to_le_bytes());
    bytes.push(sub_opcode::DISMOUNT_RESPONSE);
    bytes.extend_from_slice(&rider_guid.to_le_bytes());
    bytes.extend_from_slice(&composite_effect_id.to_le_bytes());
    bytes
}
