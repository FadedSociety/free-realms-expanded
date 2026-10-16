//! 055: Ranged enemy attacks - the recovered cast/projectile vocabulary plus a bounded enemy cast
//! lifecycle.
//!
//! ORIGINAL, recovered (see this item's lab report `piece-055-20261009T043358Z/`):
//! * Families: `BaseCombatPacket` 32, `BasePlayerUpdatePacket` 35, `BaseAbilityPacket` 36
//!   (`lab/references/OpenSourceFreeRealms/.../PacketCodes.cs`, matching the supplied server's own
//!   enum files). The sub-opcode immediates in the client's constructors agree name for name:
//!   `AbilityPacketStartCasting` 3, `AbilityPacketDetonateProjectile` 14, and
//!   `PlayerUpdateLaunchProjectilePacket` 62 (`FUN_00a2ed30`, `FUN_00a2ee30`, `FUN_008d0b10`).
//! * `PlayerUpdateLaunchProjectilePacket` embeds the client's `ProjectileParameters` object. Its
//!   constructor (`FUN_008d00e0`) stores the `0x0175b420` dword - `0x3f800000`, 1.0f - at dword
//!   +0x20 and +0x24, i.e. [`PROJECTILE_PARAMETERS_DWORD_UNIT_A`] and
//!   [`PROJECTILE_PARAMETERS_DWORD_UNIT_B`]. That single default is the only literal recovered from
//!   the embedded struct; the remaining fields are initialised from runtime globals.
//! * `AbilityPacketDetonateProjectile`'s constructor initialises a 64-bit id pair followed by three
//!   zeroed 32-bit fields ([`DETONATE_ID_DWORD_COUNT`], [`DETONATE_INT32_FIELD_COUNT`]). The field
//!   *meanings* are not recovered; only the shape is.
//! * The client simulates projectiles with distinct motion models - `Projectile`, `ProjectileBeam`,
//!   `ProjectileArc`, `ProjectileThrowCatch`, `ProjectileSeek` - driven by a `ProjectileManager`,
//!   an `ActorProjectileEmitter` and the `PlayerSpawnProjectileEvent`/`PlayerProjectileEvent`
//!   pair, all named in the preserved client's RTTI. So travel is client-side; the server's cast
//!   (`36/3`), launch (`35/62`) and detonation (`36/14`) frames bracket it.
//! * Hit outcomes are their own family-32 frames: `AttackTargetDamage` 4, `AttackAttackerMissed` 5,
//!   `AttackTargetDodged` 6. They are the source-defined alternatives to a landed hit, which is why
//!   [`CombatResolution`] is an input to this module rather than a roll inside it.
//! * `AbilityPacketFailed` (36/1) carries one `int32` string id; the client's own
//!   `CodeStringMappings.txt` vocabulary has no out-of-range or line-of-sight message, so an
//!   over-reach cast is refused as `NoCastTargeting` 288, the same convention item 035 uses.
//!
//! RECONSTRUCTED (ours, not historical): [`RangedEnemyAttack`] and the [`EnemyCastProfile`] rules
//! around it. The lifecycle - cast for `ActionTime`, launch when flight time is positive, detonate
//! after the flight, resolve exactly once - is our composition of the recovered frames. Cast times,
//! projectile speeds and enemy reaches are **caller-supplied**: no retail enemy cast capture exists
//! in the lab, so no value is defaulted here and a zero-flight profile simply does not emit a
//! travelling launch. `AbilityPacketLaunchAndLand` (36/4) is a recovered frame name whose role is
//! still unrecovered, so this module deliberately never emits it.
//!
//! GAPS: the payload layouts of `35/62`, `36/14`, `36/4` and `36/18` are unrecovered. This module
//! therefore returns typed [`EnemyAttackIntent`]s and never fabricates a frame for them; a caller
//! that needs the bytes must first recover the layout. Nothing here applies damage, grants rewards
//! or claims that retail enemies ever cast.

/// Combat packet family (`BaseCombatPacket`).
pub const COMBAT_FAMILY: u16 = 32;
pub const COMBAT_SUB_AUTO_ATTACK_TARGET: u16 = 1;
pub const COMBAT_SUB_SINGLE_ATTACK_TARGET: u16 = 3;
pub const COMBAT_SUB_ATTACK_TARGET_DAMAGE: u16 = 4;
pub const COMBAT_SUB_ATTACKER_MISSED: u16 = 5;
pub const COMBAT_SUB_TARGET_DODGED: u16 = 6;
pub const COMBAT_SUB_ATTACK_PROCESSED: u16 = 7;
pub const COMBAT_SUB_ENABLE_BOSS_DISPLAY: u16 = 9;

/// Ability packet family (`BaseAbilityPacket`).
pub const ABILITY_FAMILY: u16 = 36;
pub const ABILITY_SUB_FAILED: u16 = 1;
pub const ABILITY_SUB_START_CASTING: u16 = 3;
/// Recovered frame name; its role could not be recovered, so this module never emits it.
pub const ABILITY_SUB_LAUNCH_AND_LAND: u16 = 4;
pub const ABILITY_SUB_DETONATE_PROJECTILE: u16 = 14;
pub const ABILITY_SUB_CAST_INTERRUPT: u16 = 18;

/// Player-update family that carries the travelling projectile launch.
pub const PLAYER_UPDATE_FAMILY: u16 = 35;
pub const PLAYER_UPDATE_SUB_LAUNCH_PROJECTILE: u16 = 62;

/// `NoCastTargeting` in the client's own string-id vocabulary.
pub const NO_CAST_TARGETING_ID: i32 = 288;

/// Dword offsets inside the client's `ProjectileParameters` that both default to 1.0f.
pub const PROJECTILE_PARAMETERS_DWORD_UNIT_A: usize = 8;
pub const PROJECTILE_PARAMETERS_DWORD_UNIT_B: usize = 9;
pub const PROJECTILE_PARAMETERS_UNIT_DEFAULT: f32 = 1.0;

/// Partial `AbilityPacketDetonateProjectile` shape: a 64-bit id pair, then three `int32`s.
pub const DETONATE_ID_DWORD_COUNT: usize = 2;
pub const DETONATE_INT32_FIELD_COUNT: usize = 3;

/// Everything the caller must decide because retail values were not recovered.
///
/// `range_units` is an admission radius, not a simulation; `flight_time_ms` is how long the
/// caller's own projectile simulation needs between [`PLAYER_UPDATE_SUB_LAUNCH_PROJECTILE`] and
/// [`ABILITY_SUB_DETONATE_PROJECTILE`]. A zero flight time means the profile has no travelling
/// projectile and only the detonation frame is emitted at cast end.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnemyCastProfile {
    pub ability_id: i32,
    pub cast_time_ms: u32,
    pub flight_time_ms: u32,
    pub range_units: f32,
}

/// Source-defined alternatives to a landed hit. The frame sub-opcode is recovered; which one
/// applies is the caller's resolution (this module does not roll dice or invent a hit table).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombatResolution {
    Damage,
    Missed,
    Dodged,
}

impl CombatResolution {
    pub fn sub_opcode(self) -> u16 {
        match self {
            Self::Damage => COMBAT_SUB_ATTACK_TARGET_DAMAGE,
            Self::Missed => COMBAT_SUB_ATTACKER_MISSED,
            Self::Dodged => COMBAT_SUB_TARGET_DODGED,
        }
    }
}

/// One recovered frame the lifecycle wants to send. Payload bytes are intentionally absent until
/// the layout is recovered; [`EnemyAttackIntent::frame`] returns the family/sub pair a caller can
/// dispatch on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EnemyAttackIntent {
    StartCasting {
        caster: u64,
        target: u64,
        ability_id: i32,
        action_time_ms: u32,
        has_action_progress: bool,
    },
    LaunchProjectile {
        caster: u64,
        target: u64,
    },
    DetonateProjectile {
        caster: u64,
        target: u64,
    },
    CastInterrupt {
        caster: u64,
    },
    AttackResolution {
        attacker: u64,
        target: u64,
        resolution: CombatResolution,
    },
}

impl EnemyAttackIntent {
    /// `(family, sub-opcode)` of the recovered frame this intent corresponds to.
    pub fn frame(self) -> (u16, u16) {
        match self {
            Self::StartCasting { .. } => (ABILITY_FAMILY, ABILITY_SUB_START_CASTING),
            Self::LaunchProjectile { .. } => {
                (PLAYER_UPDATE_FAMILY, PLAYER_UPDATE_SUB_LAUNCH_PROJECTILE)
            }
            Self::DetonateProjectile { .. } => (ABILITY_FAMILY, ABILITY_SUB_DETONATE_PROJECTILE),
            Self::CastInterrupt { .. } => (ABILITY_FAMILY, ABILITY_SUB_CAST_INTERRUPT),
            Self::AttackResolution { resolution, .. } => (COMBAT_FAMILY, resolution.sub_opcode()),
        }
    }
}

/// Where the caster is in the reconstructed lifecycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyCastState {
    Idle,
    Casting {
        ready_at_ms: u64,
    },
    InFlight {
        resolve_at_ms: u64,
    },
    /// The detonation frame went out; the caller now supplies the resolution.
    AwaitingResolution,
    Resolved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CastError {
    OutOfRange,
    Invalid,
    Busy,
    NotCasting,
    NotDetonated,
    AlreadyResolved,
}

impl std::fmt::Display for CastError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for CastError {}

/// One enemy caster. `now_ms` is a monotonic millisecond clock owned by the caller, so the same
/// struct can be driven from a test, a replay or the live encounter.
#[derive(Clone, Copy, Debug)]
pub struct RangedEnemyAttack {
    caster: u64,
    target: u64,
    profile: EnemyCastProfile,
    state: EnemyCastState,
}

impl RangedEnemyAttack {
    pub fn new(caster: u64, target: u64, profile: EnemyCastProfile) -> Self {
        Self {
            caster,
            target,
            profile,
            state: EnemyCastState::Idle,
        }
    }

    pub fn state(&self) -> EnemyCastState {
        self.state
    }

    pub fn profile(&self) -> EnemyCastProfile {
        self.profile
    }

    pub fn caster(&self) -> u64 {
        self.caster
    }

    pub fn target(&self) -> u64 {
        self.target
    }

    /// Admit the cast. `distance_units` is the caller's current caster-to-target distance; an
    /// over-reach request is refused with [`CastError::OutOfRange`] and the caller answers with
    /// [`NO_CAST_TARGETING_ID`]. The cast never starts on a refusal.
    pub fn begin_cast(
        &mut self,
        now_ms: u64,
        distance_units: f32,
    ) -> Result<EnemyAttackIntent, CastError> {
        if !distance_units.is_finite()
            || distance_units < 0.0
            || !self.profile.range_units.is_finite()
            || self.profile.range_units <= 0.0
            || self.profile.ability_id <= 0
        {
            return Err(CastError::Invalid);
        }
        if self.state != EnemyCastState::Idle {
            return Err(CastError::Busy);
        }
        if distance_units > self.profile.range_units {
            return Err(CastError::OutOfRange);
        }
        let ready_at_ms = now_ms
            .checked_add(u64::from(self.profile.cast_time_ms))
            .ok_or(CastError::Invalid)?;
        self.state = EnemyCastState::Casting { ready_at_ms };
        Ok(EnemyAttackIntent::StartCasting {
            caster: self.caster,
            target: self.target,
            ability_id: self.profile.ability_id,
            action_time_ms: self.profile.cast_time_ms,
            // Reconstructed: a timed cast shows the client's action-progress bar; an instant one
            // has nothing to show. The recovered bool field itself is what the client reads.
            has_action_progress: self.profile.cast_time_ms > 0,
        })
    }

    /// Advance the lifecycle. Returns the frame to send when a milestone is crossed, otherwise
    /// `None`. Milestones are only crossed once, so repeated calls at the same timestamp are safe.
    pub fn advance(&mut self, now_ms: u64) -> Result<Option<EnemyAttackIntent>, CastError> {
        match self.state {
            EnemyCastState::Casting { ready_at_ms } if now_ms >= ready_at_ms => {
                if self.profile.flight_time_ms == 0 {
                    self.state = EnemyCastState::AwaitingResolution;
                    Ok(Some(EnemyAttackIntent::DetonateProjectile {
                        caster: self.caster,
                        target: self.target,
                    }))
                } else {
                    let resolve_at_ms = ready_at_ms
                        .checked_add(u64::from(self.profile.flight_time_ms))
                        .ok_or(CastError::Invalid)?;
                    self.state = EnemyCastState::InFlight { resolve_at_ms };
                    Ok(Some(EnemyAttackIntent::LaunchProjectile {
                        caster: self.caster,
                        target: self.target,
                    }))
                }
            }
            EnemyCastState::InFlight { resolve_at_ms } if now_ms >= resolve_at_ms => {
                self.state = EnemyCastState::AwaitingResolution;
                Ok(Some(EnemyAttackIntent::DetonateProjectile {
                    caster: self.caster,
                    target: self.target,
                }))
            }
            _ => Ok(None),
        }
    }

    /// Only a cast can be interrupted; in-flight projectiles and resolved attacks have no
    /// recovered cancellation frame.
    pub fn interrupt(&mut self) -> Result<EnemyAttackIntent, CastError> {
        if matches!(self.state, EnemyCastState::Casting { .. }) {
            self.state = EnemyCastState::Idle;
            Ok(EnemyAttackIntent::CastInterrupt {
                caster: self.caster,
            })
        } else {
            Err(CastError::NotCasting)
        }
    }

    /// Apply the caller's resolution exactly once, after the detonation frame.
    pub fn resolve(
        &mut self,
        resolution: CombatResolution,
    ) -> Result<EnemyAttackIntent, CastError> {
        match self.state {
            EnemyCastState::AwaitingResolution => {
                self.state = EnemyCastState::Resolved;
                Ok(EnemyAttackIntent::AttackResolution {
                    attacker: self.caster,
                    target: self.target,
                    resolution,
                })
            }
            EnemyCastState::Resolved => Err(CastError::AlreadyResolved),
            _ => Err(CastError::NotDetonated),
        }
    }
}
