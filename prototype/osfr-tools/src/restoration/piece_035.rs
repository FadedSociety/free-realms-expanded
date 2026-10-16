//! 035: Archer progression - the recovered ranged-action wire and a bounded reach gate.
//!
//! ORIGINAL, recovered (see this item's lab report `piece-035-20261009T033649Z/`):
//! * The ability family is packet family 36 and the projectile launch rides the PlayerUpdate
//!   family 35. The supplied client's own constructors carry the sub-opcode immediates:
//!   `AbilityPacketStartCasting` = 3 (`FUN_00a2ed30`), `AbilityPacketDetonateProjectile` = 14
//!   (`FUN_00a2ee30`), `PlayerUpdateLaunchProjectilePacket` = 62 (`FUN_008d0b10`,
//!   `0x3e`). The pinned community reader tables agree name for name.
//! * `AbilityPacketClientRequestStartAbility` (36/10) is the client's ranged request:
//!   `[int16 36][int16 10][int32 ActionBarId][int32 Slot][int32 Target]`, then a `float4`
//!   position when `Target == 1` and otherwise a `uint64` guid; the frame must be consumed
//!   exactly. Read from the pinned `Sanctuary`/`FreeRealms-Legacy` reader and cross-checked
//!   against the client's constructor constants for the same sub-opcode.
//! * `AbilityPacketSetDefinition` (36/5) is the toolbar that carries the ranged reach:
//!   `[int32 ProfileId][int32 SlotCount]` and eight positional slots. A slot is `int32 Type`,
//!   then `InstanceId/ManaCost` (types 1 and 3) or `ItemDefinitionId` (type 2), then
//!   `IconId, NameId, Unknown7, Range(float), Unknown9, AbilityDefinitionId, Unknown11` and the
//!   trailing `ForceDismount` byte. The client hardcodes eight slots, so a short run hides every
//!   later slot. PR 121 of the pinned community repository documents `Unknown8` as `float Range`,
//!   reverse-engineered from the client "which reads the field as a float and uses it as the
//!   auto-target radius".
//! * `AbilityPacketFailed` (36/1) carries one `int32` string id. The client's own
//!   `CodeStringMappings.txt` names that vocabulary: `NoCastKnockedOut` 280, `NoCastAsleep` 281,
//!   `NoCastSilenced` 282, `NoCastAfraid` 283, `NoCastStunned` 284, `NoCastInsufficientMana` 285,
//!   `NoCastDefault` 286, `NoCastRecastTimer` 287, `NoCastTargeting` 288 and `NoCastNotEnabled`
//!   3079. There is no out-of-range or line-of-sight message in that vocabulary.
//! * `AbilityPacketStartCasting` (36/3, server to client) is `[uint64 caster][uint64 target]
//!   [int32 CompositeEffectId][int32 Animation][int32 AbilityId][float ActionTime][bool
//!   HasActionProgress]`; the client's constructor default `-1` for the animation field lands on
//!   that exact dword.
//!
//! COMMUNITY, reverse-engineered but not original server data (`CombatJobs.json` /
//! `CombatAbilities.json` in the pinned community repository, PR 121): the Archer job kit is
//! profile 35, wield type 11, auto-target reach 30 units; the starter bow item 4266 maps to basic
//! ability 1010 (Barrage, 279 damage) and special ability 1011 (Volley, 506 damage, radius 10,
//! 100 energy). Melee kits are documented at reach 7 and the Wizard at 20, so the 30-unit reach is
//! what makes this kit ranged. Those numbers are community reconstruction; the original server's
//! balance values are not recovered.
//!
//! RECONSTRUCTED (ours, not historical): [`ReachGate`] (a squared-distance admission test in the
//! pinned helper's `float4` semantics), [`LineOfSight`] (a caller-supplied segment/box test that is
//! opt-in, because no retail line-of-sight predicate was recovered), [`admit_ranged_action`] (the
//! composed entry point that runs reach before geometry) and [`AmmoPolicy`] (arrows are never
//! charged, because the corpus contains no consumable-ammo evidence: the wiki names no ammo item,
//! the recovered item-definition export has zero non-zero `Range` rows, and a single
//! `Point Blank Archer Quiver` prize is a clothing item). Refusals reuse the recovered
//! `NoCastTargeting` string id because retail has no range or sight message; callers that need a
//! distinct client-visible reason must recover one first.
//!
//! GAPS: the payload layouts of 36/4 `LaunchAndLand`, 36/14 `DetonateProjectile` and 35/62
//! `LaunchProjectile` are not recovered - the pinned community build ships serializers for neither
//! and the lab holds no original ability capture - so [`ProjectilePayload`] reports them as
//! unrecovered instead of guessing frames. Nothing here grants currency, items or experience.

/// Ability packet family (`BaseAbilityPacket.OpCode`).
pub const ABILITY_FAMILY: u16 = 36;
/// Player-update family that also carries projectile launches.
pub const PLAYER_UPDATE_FAMILY: u16 = 35;

pub const SUB_FAILED: u16 = 1;
pub const SUB_START_CASTING: u16 = 3;
pub const SUB_LAUNCH_AND_LAND: u16 = 4;
pub const SUB_SET_DEFINITION: u16 = 5;
pub const SUB_PURCHASE_ABILITY: u16 = 7;
pub const SUB_UPDATE_ABILITY_EXPERIENCE: u16 = 8;
pub const SUB_STOP_AURA: u16 = 9;
pub const SUB_CLIENT_REQUEST_START_ABILITY: u16 = 10;
pub const SUB_MELEE_REFRESH: u16 = 11;
pub const SUB_REQUEST_ABILITY_DEFINITION: u16 = 12;
pub const SUB_ABILITY_DEFINITION: u16 = 13;
pub const SUB_DETONATE_PROJECTILE: u16 = 14;
pub const SUB_PULSE_LOCATION_TARGETING: u16 = 15;
pub const SUB_RECEIVE_PULSE_LOCATION: u16 = 16;
pub const SUB_EXECUTE_CLIENT_LUA: u16 = 17;
pub const SUB_CAST_INTERRUPT: u16 = 18;
/// `PlayerUpdateLaunchProjectilePacket` inside family 35.
pub const SUB_LAUNCH_PROJECTILE: u16 = 62;

/// The client hardcodes eight toolbar slots; an empty slot hides every later one.
pub const ABILITY_SLOT_COUNT: usize = 8;

pub const ABILITY_SLOT_TYPE_EMPTY: i32 = 0;
pub const ABILITY_SLOT_TYPE_UNKNOWN: i32 = 1;
pub const ABILITY_SLOT_TYPE_ITEM: i32 = 2;
pub const ABILITY_SLOT_TYPE_ABILITY: i32 = 3;

/// Archer profile id.
pub const ARCHER_PROFILE_ID: i32 = 35;
/// Bow wield type: `ItemClasses.txt` row 18 is `WIELD_TYPE 11` for the archer bow class.
pub const ARCHER_WIELD_TYPE: i32 = 11;
/// Community kit reach ("Ranged; reach 30u").
pub const ARCHER_AUTO_TARGET_REACH: f32 = 30.0;
pub const ARCHER_STARTER_BOW_ITEM_ID: i32 = 4266;
pub const ARCHER_BASIC_ABILITY_ID: i32 = 1010;
pub const ARCHER_SPECIAL_ABILITY_ID: i32 = 1011;
pub const ARCHER_FALLBACK_BASIC_ABILITY_ID: i32 = 1001;
pub const ARCHER_BASIC_RECAST_MS: u32 = 600;
pub const ARCHER_ENERGY_MAX: i32 = 100;
pub const ARCHER_ENERGY_REGEN_PER_SECOND: i32 = 4;
pub const ARCHER_BASIC_DAMAGE: i32 = 279;
pub const ARCHER_SPECIAL_DAMAGE: i32 = 506;
pub const ARCHER_SPECIAL_AOE_RADIUS: f32 = 10.0;
pub const ARCHER_SPECIAL_ENERGY_COST: i32 = 100;

pub const WIZARD_PROFILE_ID: i32 = 12;
pub const WIZARD_AUTO_TARGET_REACH: f32 = 20.0;
/// Every melee kit in the pinned community table: Ninja 2, Medic 11, Warrior 32, Brawler 43.
pub const MELEE_AUTO_TARGET_REACH: f32 = 7.0;

/// The recovered Archer kit. Values are community reconstruction, not original server balance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArcherKit {
    pub profile_id: i32,
    pub wield_type: i32,
    pub auto_target_reach: f32,
    pub starter_bow_item_id: i32,
    pub basic_ability_id: i32,
    pub special_ability_id: i32,
    pub fallback_basic_ability_id: i32,
    pub basic_recast_ms: u32,
    pub energy_max: i32,
    pub energy_regen_per_second: i32,
}

impl ArcherKit {
    pub const RECOVERED: Self = Self {
        profile_id: ARCHER_PROFILE_ID,
        wield_type: ARCHER_WIELD_TYPE,
        auto_target_reach: ARCHER_AUTO_TARGET_REACH,
        starter_bow_item_id: ARCHER_STARTER_BOW_ITEM_ID,
        basic_ability_id: ARCHER_BASIC_ABILITY_ID,
        special_ability_id: ARCHER_SPECIAL_ABILITY_ID,
        fallback_basic_ability_id: ARCHER_FALLBACK_BASIC_ABILITY_ID,
        basic_recast_ms: ARCHER_BASIC_RECAST_MS,
        energy_max: ARCHER_ENERGY_MAX,
        energy_regen_per_second: ARCHER_ENERGY_REGEN_PER_SECOND,
    };
}

/// Wire or policy failure. `WrongFamily`, `WrongSubOpcode`, `Truncated` and `TrailingBytes` are
/// strictness on recovered layouts; the rest guard reconstructed policy input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RangedError {
    WrongFamily,
    WrongSubOpcode,
    Truncated,
    TrailingBytes,
    BadProfile,
    BadSlotCount,
    UnknownSlotType,
    NonFinite,
}

impl std::fmt::Display for RangedError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for RangedError {}

/// The recovered `NoCast*` string ids an `AbilityPacketFailed` frame may carry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoCastReason {
    KnockedOut,
    Asleep,
    Silenced,
    Afraid,
    Stunned,
    InsufficientMana,
    Default,
    RecastTimer,
    Targeting,
    NotEnabled,
}

impl NoCastReason {
    /// String id from the client's own `CodeStringMappings.txt` vocabulary.
    pub fn string_id(self) -> i32 {
        match self {
            Self::KnockedOut => 280,
            Self::Asleep => 281,
            Self::Silenced => 282,
            Self::Afraid => 283,
            Self::Stunned => 284,
            Self::InsufficientMana => 285,
            Self::Default => 286,
            Self::RecastTimer => 287,
            Self::Targeting => 288,
            Self::NotEnabled => 3079,
        }
    }
}

/// Where a start-ability request points. `Target == 1` is a world position; anything else is a guid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RangedTarget {
    Position([f32; 4]),
    Guid(u64),
}

/// The recovered 36/10 client request.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StartAbilityRequest {
    pub action_bar_id: i32,
    pub slot: i32,
    pub target: RangedTarget,
}

impl StartAbilityRequest {
    pub fn decode(bytes: &[u8]) -> Result<Self, RangedError> {
        let mut reader = Reader::new(bytes, ABILITY_FAMILY, SUB_CLIENT_REQUEST_START_ABILITY)?;
        let action_bar_id = reader.i32()?;
        let slot = reader.i32()?;
        let target = reader.i32()?;
        let target = if target == 1 {
            let position = reader.f32x4()?;
            if !is_finite_position(position) {
                return Err(RangedError::NonFinite);
            }
            RangedTarget::Position(position)
        } else {
            RangedTarget::Guid(reader.u64()?)
        };
        reader.finish()?;
        Ok(Self {
            action_bar_id,
            slot,
            target,
        })
    }
}

/// One decoded toolbar slot. Type 0 slots are `None`; every other field is the recovered layout.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AbilitySlot {
    pub slot_type: i32,
    pub instance_id: i32,
    pub mana_cost: i32,
    pub item_definition_id: i32,
    pub icon_id: i32,
    pub name_id: i32,
    pub unknown7: i32,
    /// The client's auto-target radius for this slot (PR 121).
    pub range: f32,
    pub unknown9: i32,
    pub ability_definition_id: i32,
    pub unknown11: i32,
    pub force_dismount: u8,
}

/// The recovered 36/5 toolbar.
#[derive(Clone, Debug, PartialEq)]
pub struct Toolbar {
    pub profile_id: i32,
    pub slots: Vec<Option<AbilitySlot>>,
}

impl Toolbar {
    pub fn decode(bytes: &[u8]) -> Result<Self, RangedError> {
        let mut reader = Reader::new(bytes, ABILITY_FAMILY, SUB_SET_DEFINITION)?;
        let profile_id = reader.i32()?;
        if profile_id <= 0 {
            return Err(RangedError::BadProfile);
        }
        let slot_count = reader.i32()?;
        if slot_count != ABILITY_SLOT_COUNT as i32 {
            return Err(RangedError::BadSlotCount);
        }
        let mut slots = Vec::with_capacity(ABILITY_SLOT_COUNT);
        for _ in 0..ABILITY_SLOT_COUNT {
            slots.push(Self::decode_slot(&mut reader)?);
        }
        reader.finish()?;
        Ok(Self { profile_id, slots })
    }

    fn decode_slot(reader: &mut Reader<'_>) -> Result<Option<AbilitySlot>, RangedError> {
        let slot_type = reader.i32()?;
        if slot_type == ABILITY_SLOT_TYPE_EMPTY {
            return Ok(None);
        }
        let mut slot = AbilitySlot {
            slot_type,
            instance_id: 0,
            mana_cost: 0,
            item_definition_id: 0,
            icon_id: 0,
            name_id: 0,
            unknown7: 0,
            range: 0.0,
            unknown9: 0,
            ability_definition_id: 0,
            unknown11: 0,
            force_dismount: 0,
        };
        match slot_type {
            ABILITY_SLOT_TYPE_UNKNOWN | ABILITY_SLOT_TYPE_ABILITY => {
                slot.instance_id = reader.i32()?;
                slot.mana_cost = reader.i32()?;
            }
            ABILITY_SLOT_TYPE_ITEM => slot.item_definition_id = reader.i32()?,
            _ => return Err(RangedError::UnknownSlotType),
        }
        slot.icon_id = reader.i32()?;
        slot.name_id = reader.i32()?;
        slot.unknown7 = reader.i32()?;
        slot.range = reader.f32()?;
        if !slot.range.is_finite() {
            return Err(RangedError::NonFinite);
        }
        slot.unknown9 = reader.i32()?;
        slot.ability_definition_id = reader.i32()?;
        slot.unknown11 = reader.i32()?;
        slot.force_dismount = reader.u8()?;
        Ok(Some(slot))
    }

    /// The reach of the slot the client would use first, if the toolbar carries one.
    pub fn reach(&self) -> Option<f32> {
        self.slots.iter().flatten().map(|slot| slot.range).next()
    }
}

/// The recovered 36/3 server-to-client cast frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StartCasting {
    pub caster_guid: u64,
    pub target_guid: u64,
    pub composite_effect_id: i32,
    /// Animation group id; the client's constructor default is `-1`.
    pub animation: i32,
    pub ability_id: i32,
    /// Slot-lock window in seconds.
    pub action_time: f32,
    pub has_action_progress: bool,
}

fn put_i32(bytes: &mut Vec<u8>, value: i32) {
    bytes.extend(value.to_le_bytes());
}

/// Encode the recovered `AbilityPacketFailed` (36/1).
pub fn encode_failure(reason: NoCastReason) -> Vec<u8> {
    let mut bytes = vec![ABILITY_FAMILY as u8, 0, SUB_FAILED as u8, 0];
    put_i32(&mut bytes, reason.string_id());
    bytes
}

/// Encode the recovered `AbilityPacketStartCasting` (36/3).
pub fn encode_start_casting(cast: &StartCasting) -> Vec<u8> {
    let mut bytes = vec![ABILITY_FAMILY as u8, 0, SUB_START_CASTING as u8, 0];
    bytes.extend(cast.caster_guid.to_le_bytes());
    bytes.extend(cast.target_guid.to_le_bytes());
    put_i32(&mut bytes, cast.composite_effect_id);
    put_i32(&mut bytes, cast.animation);
    put_i32(&mut bytes, cast.ability_id);
    bytes.extend(cast.action_time.to_le_bytes());
    bytes.push(u8::from(cast.has_action_progress));
    bytes
}

/// RECONSTRUCTED admission test: the pinned community helper compares `float4` distance squared
/// against the reach squared, and that is the only recovered range semantics available.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReachGate {
    reach: f32,
}

impl ReachGate {
    pub fn new(reach: f32) -> Result<Self, RangedError> {
        if !reach.is_finite() || reach <= 0.0 {
            return Err(RangedError::NonFinite);
        }
        Ok(Self { reach })
    }

    /// The documented community reach for the Archer kit.
    pub fn for_archer() -> Self {
        Self {
            reach: ARCHER_AUTO_TARGET_REACH,
        }
    }

    pub fn reach(&self) -> f32 {
        self.reach
    }

    pub fn admits(&self, origin: [f32; 4], target: [f32; 4]) -> bool {
        let squared = origin
            .iter()
            .zip(target.iter())
            .map(|(a, b)| (a - b) * (a - b))
            .sum::<f32>();
        squared <= self.reach * self.reach
    }

    /// Retail has no range message, so an over-reach request is refused as `NoCastTargeting`; a
    /// caller that recovers a distinct string id must replace this instead of inventing one.
    pub fn admit_position(&self, origin: [f32; 4], target: [f32; 4]) -> Result<(), NoCastReason> {
        if !is_finite_position(origin) || !is_finite_position(target) {
            return Err(NoCastReason::Targeting);
        }
        if self.admits(origin, target) {
            Ok(())
        } else {
            Err(NoCastReason::Targeting)
        }
    }
}

/// Axis-aligned blocker for [`LineOfSight::SegmentBlockers`]: `[minimum, maximum]`.
pub type Aabb = [[f32; 4]; 2];

/// RECONSTRUCTED visibility policy. The retail predicate is not recovered, so the default is
/// [`LineOfSight::Unchecked`] and a refusal only happens when the caller supplies geometry.
#[derive(Clone, Copy, Debug)]
pub enum LineOfSight<'a> {
    Unchecked,
    SegmentBlockers(&'a [Aabb]),
}

impl LineOfSight<'_> {
    pub fn admit(&self, origin: [f32; 4], target: [f32; 4]) -> Result<(), NoCastReason> {
        if !is_finite_position(origin) || !is_finite_position(target) {
            return Err(NoCastReason::Targeting);
        }
        match self {
            Self::Unchecked => Ok(()),
            Self::SegmentBlockers(blockers) => {
                for blocker in *blockers {
                    if segment_hits_box(origin, target, blocker) {
                        return Err(NoCastReason::Targeting);
                    }
                }
                Ok(())
            }
        }
    }
}

/// Compose the reconstructed gates in the order the server should evaluate them: the cheap reach
/// test first, then caller-supplied geometry. `Ok(())` only means the request may proceed to cast;
/// it grants nothing, and damage or rewards stay in the lead-owned layers.
pub fn admit_ranged_action(
    origin: [f32; 4],
    target: [f32; 4],
    gate: &ReachGate,
    sight: LineOfSight<'_>,
) -> Result<(), NoCastReason> {
    gate.admit_position(origin, target)?;
    sight.admit(origin, target)
}

fn is_finite_position(position: [f32; 4]) -> bool {
    position.iter().all(|value| value.is_finite())
}

/// Straight-line segment against an axis-aligned box in world `(x, y, z)`, which is geometry
/// rather than simulation: no gravity, arc or travel time is inferred from it.
fn segment_hits_box(origin: [f32; 4], target: [f32; 4], blocker: &Aabb) -> bool {
    let mut enter = 0.0_f32;
    let mut exit = 1.0_f32;
    for axis in 0..3 {
        let delta = target[axis] - origin[axis];
        let (minimum, maximum) = (blocker[0][axis], blocker[1][axis]);
        if delta.abs() <= f32::EPSILON {
            if origin[axis] < minimum || origin[axis] > maximum {
                return false;
            }
            continue;
        }
        let first = (minimum - origin[axis]) / delta;
        let second = (maximum - origin[axis]) / delta;
        let (low, high) = if first <= second {
            (first, second)
        } else {
            (second, first)
        };
        enter = enter.max(low);
        exit = exit.min(high);
        if enter > exit {
            return false;
        }
    }
    true
}

/// RECONSTRUCTED ammo policy: nothing charges an item, because the corpus holds no consumable
/// ammunition for the Archer. The single `Point Blank Archer Quiver` entry is clothing and the
/// wiki names no ammo item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AmmoPolicy {
    NotRecovered,
}

impl AmmoPolicy {
    pub const RECOVERED: Self = Self::NotRecovered;

    pub fn charges_items(self) -> bool {
        false
    }

    pub fn requires_equipped_ammo(self) -> bool {
        false
    }
}

/// What is known about a projectile frame. The sub-opcodes are recovered; the payload layouts are
/// not, so this type exists to stop a caller from assuming a frame it can already build.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectilePayload {
    Unrecovered(u16),
}

impl ProjectilePayload {
    pub fn launch_and_land() -> Self {
        Self::Unrecovered(SUB_LAUNCH_AND_LAND)
    }

    pub fn detonate() -> Self {
        Self::Unrecovered(SUB_DETONATE_PROJECTILE)
    }

    pub fn player_update_launch() -> Self {
        Self::Unrecovered(SUB_LAUNCH_PROJECTILE)
    }

    pub fn has_recovered_layout(self) -> bool {
        false
    }
}

/// Minimal little-endian reader that enforces the recovered header and exact frame length.
struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8], family: u16, sub: u16) -> Result<Self, RangedError> {
        if bytes.len() < 4 {
            return Err(RangedError::Truncated);
        }
        if u16::from_le_bytes([bytes[0], bytes[1]]) != family {
            return Err(RangedError::WrongFamily);
        }
        if u16::from_le_bytes([bytes[2], bytes[3]]) != sub {
            return Err(RangedError::WrongSubOpcode);
        }
        Ok(Self { bytes, offset: 4 })
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], RangedError> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or(RangedError::Truncated)?;
        let slice = self
            .bytes
            .get(self.offset..end)
            .ok_or(RangedError::Truncated)?;
        self.offset = end;
        Ok(slice)
    }

    fn i32(&mut self) -> Result<i32, RangedError> {
        let raw: [u8; 4] = self.take(4)?.try_into().expect("4 bytes");
        Ok(i32::from_le_bytes(raw))
    }

    fn u64(&mut self) -> Result<u64, RangedError> {
        let raw: [u8; 8] = self.take(8)?.try_into().expect("8 bytes");
        Ok(u64::from_le_bytes(raw))
    }

    fn u8(&mut self) -> Result<u8, RangedError> {
        Ok(self.take(1)?[0])
    }

    fn f32(&mut self) -> Result<f32, RangedError> {
        let raw: [u8; 4] = self.take(4)?.try_into().expect("4 bytes");
        Ok(f32::from_le_bytes(raw))
    }

    fn f32x4(&mut self) -> Result<[f32; 4], RangedError> {
        Ok([self.f32()?, self.f32()?, self.f32()?, self.f32()?])
    }

    fn finish(&self) -> Result<(), RangedError> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(RangedError::TrailingBytes)
        }
    }
}
