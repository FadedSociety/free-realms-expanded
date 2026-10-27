//! Piece 091 - pets and companions: recovered ownership, spawn/follow and interaction
//! definitions, plus the preserved pet profile state.
//!
//! Provenance lives in `notes/Research/Pieces/091 Pets and companions.md`; the corpus
//! cross-check is the private recovery checker `piece_091.py`. Raw/decompiled inputs stay in the lab.
//!
//! ORIGINAL facts implemented here:
//!
//! * Packet family base opcode 53 (0x35): the client ctor `FUN_00b50670` writes
//!   `param_1[1] = 0x35` and `param_1[2] = subtype`, and Sanctuary's reader dispatches
//!   `53 => reader.Read<byte>() switch { ... }` over the same 35 names the community
//!   `PetBasePackets` enum carries.
//! * Client request field layouts (Ghidra pseudocode, `lab/decompiled/client/FreeRealms`):
//!   * subtype 4 `PetSummonRecallPacket` - `FUN_00b60080` (pet id + action 1),
//!     `FUN_00b601a0` (pet id + action 0) and `FUN_00b60610` (action byte only, the
//!     forced-dismiss form that never assigns the id field);
//!   * subtype 32 `PetSummonByItemIdPacket` - `FUN_00b60110` writes two 32-bit values;
//!   * subtype 26 `PetCanGoLocationPacket` - `FUN_00b5fcf0` writes four 32-bit floats
//!     and a flag byte (its serializer `FUN_00b5a530` writes the 16-byte block at
//!     object offset 0x10 and a byte at 0x20);
//!   * subtype 28 `PetInWaterPacket` - `FUN_00b60350` (1) and `FUN_00b60420` (0).
//! * Preserved pet state: the recovered managed serializer `UnknownPetStruct::Serialize`
//!   with `PetTrickInfo`, `ItemGuid`, `ProfileItemClassData` and the 32-bit length
//!   prefixed UTF-8 string writer, matched against the recorded profile sample
//!   (`PacketSendSelfToClient.json`, 28 records, first is "Captain Zeus").
//! * Client surface: `Pet.Summon`, `Pet.Dismiss`, `Pet.GetActivePetId`, `Pet.RaiseNeed`,
//!   `Pet.ModeSwitch`, `Pet.StartFeed`/`StartGroom`, `Pet.PlayWithToy` and the
//!   `PetsHandler:OnForcedDismiss`/`OnSummonFailed`/`LockUtilities`/`UnlockUtilities`
//!   handlers in `decompiled/client-ui/ScriptsBase.lua`; the native utility gate is the
//!   `PetsHandler:UnlockUtilities` path reachable from `FUN_00b5f720`/`FUN_00b5f8c0`.
//!
//! RECONSTRUCTED (labelled at each call site, not historical behaviour):
//!
//! * The companion session rules: one active pet at a time, ownership checked against
//!   the roster, recall limited to the active pet, a non-zero actor guid required for a
//!   spawn, utility requests only while the pet is not busy, and a server-initiated
//!   forced dismiss.
//! * Treating the record's leading `Unknown` int as the pet id that `Pet.GetActivePetId`
//!   and the pet browser use. It is the only per-pet unique id in the recorded sample;
//!   the serializer does not name the field.
//!
//! NOT RECOVERED, deliberately absent: follow distance thresholds (the client compares
//! against `DAT_01b5d878` / `DAT_0176b1b0` / `DAT_017ed93c` / `DAT_017649d4`, none of
//! which is resolvable from the export), need decay or utility timers, trial timers,
//! trick effects, reward or purchase values, and any native run. No capture in the lab
//! decodes a pet-family frame, so no datagram-level claim is made here.

/// Client base opcode for every pet packet (`FUN_00b50670`, Sanctuary reader key 53).
pub const PET_PACKET_BASE_OPCODE: u32 = 0x35;

/// One named subtype of the recovered pet packet family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PetPacketSubtype {
    pub id: u8,
    pub name: &'static str,
}

const fn subtype(id: u8, name: &'static str) -> PetPacketSubtype {
    PetPacketSubtype { id, name }
}

/// Every subtype the community `PetBasePackets` enum names and Sanctuary's reader
/// dispatches. Subtypes 0, 27, 30 and 37 exist in neither source.
pub const PET_PACKET_SUBTYPES: [PetPacketSubtype; 35] = [
    subtype(1, "PetDecisionPacket"),
    subtype(2, "PetResponsePacket"),
    subtype(3, "PetStatusPacket"),
    subtype(4, "PetSummonRecallPacket"),
    subtype(5, "PetListPacket"),
    subtype(6, "PetRequestNamePacket"),
    subtype(7, "PetPerformTrickPacket"),
    subtype(8, "PetTrickGestureResult"),
    subtype(9, "PetActivePacket"),
    subtype(10, "PetSetNamePacket"),
    subtype(11, "PetEquipPacket"),
    subtype(12, "PetTrickSkillLevelPacket"),
    subtype(13, "PetTrickAddPacket"),
    subtype(14, "PetTrickFavoriteSetPacket"),
    subtype(15, "PetTrickNotifyPacket"),
    subtype(16, "PetUtilityPacket"),
    subtype(17, "PetDeletePacket"),
    subtype(18, "PetUtilityGroomPacket"),
    subtype(19, "PetUtilityFeedPacket"),
    subtype(20, "PetUtilityNotifyPacket"),
    subtype(21, "PetUpdateUtilityItemChangePacket"),
    subtype(22, "PetTactileNotify"),
    subtype(23, "PetExpireTimePacket"),
    subtype(24, "PetUiModePacket"),
    subtype(25, "PetBuyRequestPacket"),
    subtype(26, "PetCanGoLocationPacket"),
    subtype(28, "PetInWaterPacket"),
    subtype(29, "PetTempLockoutError"),
    subtype(31, "PetWorldToClientBuyResponse"),
    subtype(32, "PetSummonByItemIdPacket"),
    subtype(33, "PetPlayWithToyPacket"),
    subtype(34, "PetMoodListPacket"),
    subtype(35, "PetEquipByItemRecordPacket"),
    subtype(36, "PetPacketOfferUpsell"),
    subtype(38, "PetPreviewRequestPacket"),
];

/// The recovered name of a pet subtype, or `None` when neither reader names it.
pub fn pet_packet_name(id: u8) -> Option<&'static str> {
    PET_PACKET_SUBTYPES
        .iter()
        .find(|row| row.id == id)
        .map(|row| row.name)
}

/// The action byte of `PetSummonRecallPacket` (subtype 4).
///
/// Recovered from the two client senders: `FUN_00b60080` writes 1 before a summon,
/// `FUN_00b601a0` writes 0 for the recall path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PetSummonAction {
    Recall = 0,
    Summon = 1,
}

impl PetSummonAction {
    pub const fn from_byte(value: u8) -> Option<Self> {
        match value {
            0 => Some(PetSummonAction::Recall),
            1 => Some(PetSummonAction::Summon),
            _ => None,
        }
    }
}

/// A client-to-server pet request whose field layout the client senders pin.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PetRequest {
    /// Subtype 4 with the 32-bit pet id and action 1 (`FUN_00b60080`).
    Summon { pet_id: u32 },
    /// Subtype 4 with the 32-bit pet id and action 0 (`FUN_00b601a0`).
    Recall { pet_id: u32 },
    /// Subtype 4 carrying the action byte alone: the forced-dismiss sender
    /// `FUN_00b60610` writes its action byte and never assigns the id field, so the
    /// encoder emits that byte only instead of inventing an id.
    ForcedDismiss,
    /// Subtype 32, two 32-bit values (`FUN_00b60110`): an owned pet and the utility
    /// item record that summons it.
    SummonByItemId { pet_id: u32, item_id: u32 },
    /// Subtype 26, four 32-bit floats and a flag byte (`FUN_00b5fcf0`). The client
    /// uses the words as a position; the flag's meaning is unrecovered.
    CanGoLocation { position: [f32; 4], permitted: u8 },
    /// Subtype 28, one flag byte (`FUN_00b60350` / `FUN_00b60420`).
    InWater { in_water: u8 },
}

impl PetRequest {
    pub const fn subtype(&self) -> u8 {
        match self {
            PetRequest::Summon { .. } | PetRequest::Recall { .. } | PetRequest::ForcedDismiss => 4,
            PetRequest::SummonByItemId { .. } => 32,
            PetRequest::CanGoLocation { .. } => 26,
            PetRequest::InWater { .. } => 28,
        }
    }

    /// The packet body without the SOE framing: exactly the bytes the client writes
    /// after the subtype slot.
    pub fn encode(&self) -> Vec<u8> {
        let mut body = Vec::new();
        match *self {
            PetRequest::Summon { pet_id } => {
                body.extend_from_slice(&pet_id.to_le_bytes());
                body.push(PetSummonAction::Summon as u8);
            }
            PetRequest::Recall { pet_id } => {
                body.extend_from_slice(&pet_id.to_le_bytes());
                body.push(PetSummonAction::Recall as u8);
            }
            PetRequest::ForcedDismiss => body.push(PetSummonAction::Recall as u8),
            PetRequest::SummonByItemId { pet_id, item_id } => {
                body.extend_from_slice(&pet_id.to_le_bytes());
                body.extend_from_slice(&item_id.to_le_bytes());
            }
            PetRequest::CanGoLocation {
                position,
                permitted,
            } => {
                for word in position {
                    body.extend_from_slice(&word.to_le_bytes());
                }
                body.push(permitted);
            }
            PetRequest::InWater { in_water } => body.push(in_water),
        }
        body
    }

    /// Decode one request body. Bodies of subtypes whose sender was not recovered are
    /// refused with `LayoutNotRecovered` instead of being guessed.
    pub fn decode(subtype: u8, body: &[u8]) -> Result<Self, PetError> {
        let word = |offset: usize| -> u32 {
            u32::from_le_bytes([
                body[offset],
                body[offset + 1],
                body[offset + 2],
                body[offset + 3],
            ])
        };
        match subtype {
            4 => match body.len() {
                5 => {
                    let pet_id = word(0);
                    match PetSummonAction::from_byte(body[4]) {
                        Some(PetSummonAction::Summon) => Ok(PetRequest::Summon { pet_id }),
                        Some(PetSummonAction::Recall) => Ok(PetRequest::Recall { pet_id }),
                        None => Err(PetError::UnknownAction(body[4])),
                    }
                }
                1 => match PetSummonAction::from_byte(body[0]) {
                    Some(PetSummonAction::Recall) => Ok(PetRequest::ForcedDismiss),
                    _ => Err(PetError::UnknownAction(body[0])),
                },
                other => Err(PetError::WrongLength {
                    subtype,
                    expected: 5,
                    got: other,
                }),
            },
            32 => {
                if body.len() != 8 {
                    return Err(PetError::WrongLength {
                        subtype,
                        expected: 8,
                        got: body.len(),
                    });
                }
                Ok(PetRequest::SummonByItemId {
                    pet_id: word(0),
                    item_id: word(4),
                })
            }
            26 => {
                if body.len() != 17 {
                    return Err(PetError::WrongLength {
                        subtype,
                        expected: 17,
                        got: body.len(),
                    });
                }
                let mut position = [0.0f32; 4];
                for (index, slot) in position.iter_mut().enumerate() {
                    *slot = f32::from_le_bytes([
                        body[index * 4],
                        body[index * 4 + 1],
                        body[index * 4 + 2],
                        body[index * 4 + 3],
                    ]);
                }
                Ok(PetRequest::CanGoLocation {
                    position,
                    permitted: body[16],
                })
            }
            28 => {
                if body.len() != 1 {
                    return Err(PetError::WrongLength {
                        subtype,
                        expected: 1,
                        got: body.len(),
                    });
                }
                Ok(PetRequest::InWater { in_water: body[0] })
            }
            other if pet_packet_name(other).is_some() => Err(PetError::LayoutNotRecovered(other)),
            other => Err(PetError::UnknownSubtype(other)),
        }
    }
}

/// Decoding failures. Every one of these refuses the message; nothing is defaulted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PetError {
    /// The subtype is named by neither reader.
    UnknownSubtype(u8),
    /// The body length does not match the recovered layout for that subtype.
    WrongLength {
        subtype: u8,
        expected: usize,
        got: usize,
    },
    /// The summon/recall action byte is neither 0 nor 1.
    UnknownAction(u8),
    /// The subtype is known, but no client sender pins a body for it yet.
    LayoutNotRecovered(u8),
    /// The body ended before a field the layout requires.
    Truncated,
    /// A count or string length is negative or larger than the remaining body.
    InvalidCount { offset: usize, value: i32 },
    /// A recovered string field is not valid UTF-8.
    InvalidUtf8,
}

/// One trained pet trick row: the 32-bit trick key followed by the serializer's eight
/// unnamed 32-bit fields and its flag (`Unknown`..`Unknown9`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PetTrickRecord {
    pub trick_id: i32,
    pub fields: [i32; 8],
    pub enabled: bool,
}

/// One `(int, ProfileItemClassData)` row of the recovered pet record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PetProfileItem {
    pub id: i32,
    pub unknown: i32,
    pub unknown_2: i32,
}

/// The preserved per-pet profile record. Field names follow the recovered community
/// serializer (`UnknownPetStruct`); where a name carries meaning it is stated in the
/// doc comment and marked as inferred.
#[derive(Clone, Debug, PartialEq)]
pub struct PetRecord {
    /// `Unknown`: the only per-pet unique id in the recorded sample. INFERRED to be the
    /// id `Pet.GetActivePetId` and the pet browser use; the serializer does not name it.
    pub unknown: i32,
    /// `Unknown2`: true for the sample's trained, renamed pets.
    pub unknown_2: bool,
    /// `Unknown3`: 652/653 in the sample, i.e. a species/family-shaped id.
    pub unknown_3: i32,
    /// `Food` need, 0..100 in the sample.
    pub food: f32,
    /// `Groom` need, 0..100 in the sample.
    pub groom: f32,
    /// `Exercise` need; the sample pairs it with the client's "play" progress column.
    pub exercise: f32,
    /// `Happiness`; the sample uses a small positive float rather than 0..100.
    pub happiness: f32,
    /// `Unknown8`: true on a few untrained rows.
    pub unknown_8: bool,
    pub pet_tricks: Vec<PetTrickRecord>,
    pub item_guids: Vec<i32>,
    pub profile_items: Vec<PetProfileItem>,
    pub pet_name: String,
    pub unknown_10: i32,
    pub unknown_11: i32,
    pub texture_alias: String,
    pub icon_id: i32,
    pub unknown_14: bool,
    pub unknown_15: i32,
    pub unknown_16: bool,
    /// `Unknown17`: four 32-bit slots; the sample holds the favourite trick ids.
    pub unknown_17: [i32; 4],
    /// `Unknown18`: eight 32-bit slots, all zero on some rows.
    pub unknown_18: [i32; 8],
}

impl PetRecord {
    /// INFERRED: the record's leading id doubles as the pet id used by the client's
    /// pet browser and `Pet.GetActivePetId`. Documented at the field.
    pub const fn pet_id(&self) -> i32 {
        self.unknown
    }

    /// Serialize in the recovered `UnknownPetStruct::Serialize` order.
    pub fn encode(&self) -> Vec<u8> {
        let mut body = Vec::new();
        body.extend_from_slice(&self.unknown.to_le_bytes());
        body.push(self.unknown_2 as u8);
        body.extend_from_slice(&self.unknown_3.to_le_bytes());
        for need in [self.food, self.groom, self.exercise, self.happiness] {
            body.extend_from_slice(&need.to_le_bytes());
        }
        body.push(self.unknown_8 as u8);
        body.extend_from_slice(&(self.pet_tricks.len() as i32).to_le_bytes());
        for trick in &self.pet_tricks {
            body.extend_from_slice(&trick.trick_id.to_le_bytes());
            for field in trick.fields {
                body.extend_from_slice(&field.to_le_bytes());
            }
            body.push(trick.enabled as u8);
        }
        body.extend_from_slice(&(self.item_guids.len() as i32).to_le_bytes());
        for guid in &self.item_guids {
            body.extend_from_slice(&guid.to_le_bytes());
        }
        body.extend_from_slice(&(self.profile_items.len() as i32).to_le_bytes());
        for item in &self.profile_items {
            body.extend_from_slice(&item.id.to_le_bytes());
            body.extend_from_slice(&item.unknown.to_le_bytes());
            body.extend_from_slice(&item.unknown_2.to_le_bytes());
        }
        write_string(&mut body, &self.pet_name);
        body.extend_from_slice(&self.unknown_10.to_le_bytes());
        body.extend_from_slice(&self.unknown_11.to_le_bytes());
        write_string(&mut body, &self.texture_alias);
        body.extend_from_slice(&self.icon_id.to_le_bytes());
        body.push(self.unknown_14 as u8);
        body.extend_from_slice(&self.unknown_15.to_le_bytes());
        body.push(self.unknown_16 as u8);
        for slot in self.unknown_17 {
            body.extend_from_slice(&slot.to_le_bytes());
        }
        for slot in self.unknown_18 {
            body.extend_from_slice(&slot.to_le_bytes());
        }
        body
    }

    /// Decode one record from the front of `body`, returning how many bytes it used.
    pub fn decode(body: &[u8]) -> Result<(Self, usize), PetError> {
        let mut reader = Reader::new(body);
        let unknown = reader.i32()?;
        let unknown_2 = reader.flag()?;
        let unknown_3 = reader.i32()?;
        let food = reader.f32()?;
        let groom = reader.f32()?;
        let exercise = reader.f32()?;
        let happiness = reader.f32()?;
        let unknown_8 = reader.flag()?;
        let trick_count = reader.count(37)?;
        let mut pet_tricks = Vec::with_capacity(trick_count);
        for _ in 0..trick_count {
            let trick_id = reader.i32()?;
            let mut fields = [0i32; 8];
            for field in &mut fields {
                *field = reader.i32()?;
            }
            let enabled = reader.flag()?;
            pet_tricks.push(PetTrickRecord {
                trick_id,
                fields,
                enabled,
            });
        }
        let guid_count = reader.count(4)?;
        let mut item_guids = Vec::with_capacity(guid_count);
        for _ in 0..guid_count {
            item_guids.push(reader.i32()?);
        }
        let item_count = reader.count(12)?;
        let mut profile_items = Vec::with_capacity(item_count);
        for _ in 0..item_count {
            profile_items.push(PetProfileItem {
                id: reader.i32()?,
                unknown: reader.i32()?,
                unknown_2: reader.i32()?,
            });
        }
        let pet_name = reader.string()?;
        let unknown_10 = reader.i32()?;
        let unknown_11 = reader.i32()?;
        let texture_alias = reader.string()?;
        let icon_id = reader.i32()?;
        let unknown_14 = reader.flag()?;
        let unknown_15 = reader.i32()?;
        let unknown_16 = reader.flag()?;
        let mut unknown_17 = [0i32; 4];
        for slot in &mut unknown_17 {
            *slot = reader.i32()?;
        }
        let mut unknown_18 = [0i32; 8];
        for slot in &mut unknown_18 {
            *slot = reader.i32()?;
        }
        Ok((
            PetRecord {
                unknown,
                unknown_2,
                unknown_3,
                food,
                groom,
                exercise,
                happiness,
                unknown_8,
                pet_tricks,
                item_guids,
                profile_items,
                pet_name,
                unknown_10,
                unknown_11,
                texture_alias,
                icon_id,
                unknown_14,
                unknown_15,
                unknown_16,
                unknown_17,
                unknown_18,
            },
            reader.position,
        ))
    }
}

fn write_string(body: &mut Vec<u8>, value: &str) {
    body.extend_from_slice(&(value.len() as i32).to_le_bytes());
    body.extend_from_slice(value.as_bytes());
}

struct Reader<'a> {
    body: &'a [u8],
    position: usize,
}

impl<'a> Reader<'a> {
    fn new(body: &'a [u8]) -> Self {
        Self { body, position: 0 }
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], PetError> {
        let end = self
            .position
            .checked_add(length)
            .ok_or(PetError::Truncated)?;
        if end > self.body.len() {
            return Err(PetError::Truncated);
        }
        let slice = &self.body[self.position..end];
        self.position = end;
        Ok(slice)
    }

    fn i32(&mut self) -> Result<i32, PetError> {
        let bytes = self.take(4)?;
        Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn f32(&mut self) -> Result<f32, PetError> {
        let bytes = self.take(4)?;
        Ok(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn flag(&mut self) -> Result<bool, PetError> {
        Ok(self.take(1)?[0] != 0)
    }

    fn count(&mut self, element_size: usize) -> Result<usize, PetError> {
        let offset = self.position;
        let value = self.i32()?;
        let remaining = self.body.len() - self.position;
        if value < 0 || (value as usize) * element_size > remaining {
            return Err(PetError::InvalidCount { offset, value });
        }
        Ok(value as usize)
    }

    fn string(&mut self) -> Result<String, PetError> {
        let offset = self.position;
        let length = self.i32()?;
        if length < 0 || length as usize > self.body.len() - self.position {
            return Err(PetError::InvalidCount {
                offset,
                value: length,
            });
        }
        let bytes = self.take(length as usize)?;
        std::str::from_utf8(bytes)
            .map(|text| text.to_string())
            .map_err(|_| PetError::InvalidUtf8)
    }
}

/// A spawned companion. The guid is the actor id the server gave the spawned pet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActivePet {
    pub pet_id: i32,
    pub guid: u64,
}

/// The last follow update the client sent for the active pet. The four position words
/// are kept as floats because the client computes them as floats; the flag byte's
/// meaning is unrecovered.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PetFollowState {
    pub position: [f32; 4],
    pub permitted: u8,
    pub in_water: u8,
}

/// An interaction the client can request for the active pet (the 18/19/33 paths).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PetUtilityAction {
    Feed,
    Groom,
    Play,
}

/// What a companion request did. Events describe state transitions only; no reward,
/// need value or timer is produced here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompanionEvent {
    Summoned {
        pet_id: i32,
        guid: u64,
    },
    AlreadyActive {
        pet_id: i32,
    },
    Recalled {
        pet_id: i32,
    },
    ForcedDismiss,
    FollowStateChanged {
        pet_id: i32,
    },
    UtilityRequested {
        pet_id: i32,
        utility: PetUtilityAction,
    },
}

/// Refused companion requests. Validation is server-side: the client's own checks are
/// cosmetic (member gate, controller lock) and are not trusted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionError {
    UnknownPet(i32),
    InvalidGuid(i32),
    NotActive(i32),
    NoActivePet,
    GuidMismatch { active: u64, got: u64 },
    PetBusy,
}

/// RECONSTRUCTED companion session: one active pet, ownership checked against the
/// roster. It owns no economy, no need decay and no timers.
#[derive(Clone, Debug, Default)]
pub struct CompanionSession {
    roster: Vec<PetRecord>,
    active: Option<ActivePet>,
    follow: Option<PetFollowState>,
    busy: bool,
}

impl CompanionSession {
    /// Build a session from the profile's owned pets. Duplicate ids collapse to their
    /// first record so the roster behaves as an ownership set.
    pub fn new(roster: Vec<PetRecord>) -> Self {
        let mut owned: Vec<PetRecord> = Vec::with_capacity(roster.len());
        for record in roster {
            if !owned.iter().any(|kept| kept.pet_id() == record.pet_id()) {
                owned.push(record);
            }
        }
        Self {
            roster: owned,
            active: None,
            follow: None,
            busy: false,
        }
    }

    pub fn roster(&self) -> &[PetRecord] {
        &self.roster
    }

    pub fn roster_len(&self) -> usize {
        self.roster.len()
    }

    pub fn pet_ids(&self) -> Vec<i32> {
        self.roster.iter().map(PetRecord::pet_id).collect()
    }

    pub fn active_pet(&self) -> Option<&ActivePet> {
        self.active.as_ref()
    }

    pub fn follow_state(&self) -> Option<&PetFollowState> {
        self.follow.as_ref()
    }

    /// RECONSTRUCTED spawn rule: the pet must be owned and the server must supply a
    /// non-zero actor guid. Re-summoning the active pair is not a new spawn.
    pub fn summon(&mut self, pet_id: i32, guid: u64) -> Result<CompanionEvent, SessionError> {
        if !self.roster.iter().any(|pet| pet.pet_id() == pet_id) {
            return Err(SessionError::UnknownPet(pet_id));
        }
        if guid == 0 {
            return Err(SessionError::InvalidGuid(pet_id));
        }
        if let Some(active) = self.active {
            if active.pet_id == pet_id && active.guid == guid {
                return Ok(CompanionEvent::AlreadyActive { pet_id });
            }
        }
        self.active = Some(ActivePet { pet_id, guid });
        self.follow = None;
        Ok(CompanionEvent::Summoned { pet_id, guid })
    }

    /// RECONSTRUCTED recall rule: only the pet that is currently active can be recalled.
    pub fn recall(&mut self, pet_id: i32) -> Result<CompanionEvent, SessionError> {
        match self.active {
            None => Err(SessionError::NoActivePet),
            Some(active) if active.pet_id != pet_id => Err(SessionError::NotActive(pet_id)),
            Some(_) => {
                self.active = None;
                self.follow = None;
                self.busy = false;
                Ok(CompanionEvent::Recalled { pet_id })
            }
        }
    }

    /// RECONSTRUCTED: the server can despawn the active pet at any time
    /// (`PetsHandler:OnForcedDismiss` exists for exactly that client path).
    pub fn forced_dismiss(&mut self) -> Result<CompanionEvent, SessionError> {
        if self.active.is_none() {
            return Err(SessionError::NoActivePet);
        }
        self.active = None;
        self.follow = None;
        self.busy = false;
        Ok(CompanionEvent::ForcedDismiss)
    }

    /// Apply one decoded client request. The guid is the actor the request came from,
    /// which must be the spawned pet for follow and interaction traffic.
    pub fn apply(
        &mut self,
        request: PetRequest,
        guid: u64,
    ) -> Result<CompanionEvent, SessionError> {
        match request {
            PetRequest::Summon { pet_id } => self.summon(pet_id as i32, guid),
            PetRequest::SummonByItemId { pet_id, .. } => self.summon(pet_id as i32, guid),
            PetRequest::Recall { pet_id } => self.recall(pet_id as i32),
            PetRequest::ForcedDismiss => self.forced_dismiss(),
            PetRequest::CanGoLocation {
                position,
                permitted,
            } => {
                let pet_id = self.check_active_guid(guid)?;
                let state = self.follow.get_or_insert(PetFollowState {
                    position,
                    permitted,
                    in_water: 0,
                });
                state.position = position;
                state.permitted = permitted;
                Ok(CompanionEvent::FollowStateChanged { pet_id })
            }
            PetRequest::InWater { in_water } => {
                let pet_id = self.check_active_guid(guid)?;
                let state = self.follow.get_or_insert(PetFollowState {
                    position: [0.0; 4],
                    permitted: 0,
                    in_water,
                });
                state.in_water = in_water;
                Ok(CompanionEvent::FollowStateChanged { pet_id })
            }
        }
    }

    /// RECONSTRUCTED interaction rule: an interaction needs the active pet and is
    /// refused while the pet is busy (the client locks its utility buttons through
    /// `PetsHandler:LockUtilities`, which is not trusted here).
    pub fn interact(&mut self, utility: PetUtilityAction) -> Result<CompanionEvent, SessionError> {
        let pet_id = match self.active {
            None => return Err(SessionError::NoActivePet),
            Some(active) => active.pet_id,
        };
        if self.busy {
            return Err(SessionError::PetBusy);
        }
        Ok(CompanionEvent::UtilityRequested { pet_id, utility })
    }

    pub fn set_busy(&mut self, busy: bool) {
        self.busy = busy;
    }

    pub fn is_busy(&self) -> bool {
        self.busy
    }

    fn check_active_guid(&self, guid: u64) -> Result<i32, SessionError> {
        match self.active {
            None => Err(SessionError::NoActivePet),
            Some(active) if active.guid != guid => Err(SessionError::GuidMismatch {
                active: active.guid,
                got: guid,
            }),
            Some(active) => Ok(active.pet_id),
        }
    }
}
