//! Kart reward registration and the one-shot payout seam for one finished race.
//!
//! ORIGINAL facts this module routes (evidence: `notes/Research/Kart Race Finish Protocol.md`,
//! `notes/Research/Reward Toast and Deferred Bank Findings.md`,
//! `notes/Research/Kart Racing Restoration.md`, `notes/Research/Mystery Chest Restoration.md`):
//! * `RaceSession::claim_reward` (`src/kart_racing.rs`) is the only grant authority: the award is
//!   derived from that session's own validated state, so no caller can supply or forge an amount;
//! * the reward family is base opcode `50` with a **byte** subtype (`FUN_00b88ad0`, reader
//!   `0x00b89190`); subtype 1 is the reward bundle, and the native reader `0x008e7930` places the
//!   coin award at inner offset 4, the XP/stars award at 8 and the reward kind at 16. The ordinary
//!   quest toast is that bundle with zero entries: a 72-byte inner packet;
//! * coins are granted through `Inventory::grant_coins` (the `38/19` reader) and class XP through
//!   `Profiles::add_experience` plus `Profiles::experience_packets` (native `FUN_008d86e0` 38/14 and
//!   `FUN_008d8910` 38/18);
//! * no captured frame reaches the lap limit, so the finish sequence and every payout number are
//!   RECONSTRUCTED. The starter registration below is labelled as such.
//!
//! Deliberate boundaries: this module never switches the active job, never edits `profiles.rs`,
//! `kart_racing.rs` or any shared file, and never writes a save document. It returns inner packets
//! for the lead's gateway to wrap and persist.

use crate::inventory::Inventory;
use crate::kart_racing::{KartRacingError, RaceSession, RewardProposal, RewardSchedule};
use crate::profiles::Profiles;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io};

/// ORIGINAL: the Kart Driver class (`kart_racing::KART_DRIVER_JOB_GUID`). Race XP is paid to this
/// job and to no other, so the seam refuses a payout while a different job is active instead of
/// moving experience onto it.
pub const KART_DRIVER_JOB_ID: i32 = 48;
/// ORIGINAL: `RewardBasePacket` base opcode.
pub const REWARD_FAMILY: u16 = 50;
/// ORIGINAL: `RewardBundlePacket` subtype, the only reward subtype with a recovered layout.
pub const REWARD_SUBTYPE_BUNDLE: u8 = 1;
/// RECONSTRUCTED: reward kind 16 selects the item-notification/Mystery-Chest panel, so an ordinary
/// coins/XP toast carries kind 0. The note records which values select the item path; the exact
/// currency value is not spelled out, so this default is labelled rather than asserted as original.
pub const REWARD_KIND_CURRENCY: i32 = 0;
/// RECONSTRUCTED: the starter prices the lead may replace with content rows.
pub const STARTER_REGISTRATION_PROVENANCE: &str =
    "RECONSTRUCTED starter price: no captured kart finish or payout exists";

/// One registered race payout. The registration is the price the lead allows for a speedway; the
/// session's own layout must agree with it before anything is paid.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RewardRegistration {
    pub speedway: u16,
    pub coins: u32,
    pub xp: u32,
    /// Where the numbers come from: a content row, or `STARTER_REGISTRATION_PROVENANCE`.
    pub provenance: String,
}

impl RewardRegistration {
    /// A labelled starter row. `coins` and `xp` may not both be zero.
    pub fn reconstructed(speedway: u16, coins: u32, xp: u32) -> Self {
        Self {
            speedway,
            coins,
            xp,
            provenance: STARTER_REGISTRATION_PROVENANCE.to_string(),
        }
    }

    /// The schedule a caller passes to `TrackLayout::new` when it opens this speedway's race, so the
    /// session price and the registration cannot drift by accident.
    pub fn schedule(&self) -> RewardSchedule {
        RewardSchedule {
            coins: self.coins,
            xp: self.xp,
        }
    }

    fn validate(&self) -> Result<(), RewardError> {
        if self.speedway == 0 {
            return Err(RewardError::ZeroSpeedway);
        }
        if self.coins == 0 && self.xp == 0 {
            return Err(RewardError::EmptySchedule {
                speedway: self.speedway,
            });
        }
        if self.coins > i32::MAX as u32 || self.xp > i32::MAX as u32 {
            return Err(RewardError::AwardTooLarge {
                coins: self.coins,
                xp: self.xp,
            });
        }
        Ok(())
    }
}

/// The payout table, keyed by speedway. It is content, not save state: the lead may build it from a
/// package file, and the reconstructed starter rows exist so a local race can be paid at all.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KartRewardRegistry {
    rows: BTreeMap<u16, RewardRegistration>,
}

impl KartRewardRegistry {
    /// Build a table. Rows are validated whole: a zero speedway, an empty price, an unrepresentable
    /// award or a duplicate speedway rejects the entire table rather than silently dropping a row.
    pub fn new(rows: Vec<RewardRegistration>) -> Result<Self, RewardError> {
        let mut table = BTreeMap::new();
        for row in rows {
            row.validate()?;
            if table.contains_key(&row.speedway) {
                return Err(RewardError::DuplicateRegistration {
                    speedway: row.speedway,
                });
            }
            table.insert(row.speedway, row);
        }
        Ok(Self { rows: table })
    }

    /// RECONSTRUCTED starter rows over the five recovered speedway ids (Wildwood 31, Thunder Falls
    /// 32, Frost Ridge 33, Tangletrack 34, Sandstorm 39). No captured payout fixes these numbers.
    pub fn starter_defaults() -> Self {
        Self {
            rows: [
                (31u16, 50u32, 25u32),
                (32, 60, 30),
                (33, 75, 35),
                (34, 90, 40),
                (39, 100, 50),
            ]
            .into_iter()
            .map(|(speedway, coins, xp)| {
                (
                    speedway,
                    RewardRegistration::reconstructed(speedway, coins, xp),
                )
            })
            .collect(),
        }
    }

    pub fn registration(&self, speedway: u16) -> Option<&RewardRegistration> {
        self.rows.get(&speedway)
    }

    pub fn rows(&self) -> impl Iterator<Item = &RewardRegistration> {
        self.rows.values()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RewardError {
    /// The race's speedway has no registration, so nothing may be paid.
    Unregistered {
        speedway: u16,
    },
    /// The active job is not the Kart Driver, so race XP would land on another class.
    MissingKartDriverJob {
        active: i32,
    },
    /// The session carries no price at all.
    Unpriced {
        speedway: u16,
    },
    /// The session's own price is not the registered price for its speedway.
    ScheduleNotRegistered {
        speedway: u16,
        coins: u32,
        xp: u32,
    },
    EmptySchedule {
        speedway: u16,
    },
    DuplicateRegistration {
        speedway: u16,
    },
    ZeroSpeedway,
    AwardTooLarge {
        coins: u32,
        xp: u32,
    },
    CoinOverflow {
        balance: u32,
        award: u32,
    },
    ExperienceOverflow {
        balance: u32,
        award: u32,
    },
    /// `RaceSession` refused: not finished, already claimed, clock regression.
    Racing(KartRacingError),
    Inventory(io::ErrorKind),
    Profiles(io::ErrorKind),
    Toast(io::ErrorKind),
}

/// The accepted payout: real reply packets for the lead's gateway plus the exact amounts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardPayout {
    pub speedway: u16,
    pub coins: u32,
    pub xp: u32,
    /// The Kart Driver XP before this payout, as `experience_packets` needs it.
    pub previous_xp: u32,
    /// Inner packets, in wire order: coins, then profile XP, then the reward toast.
    pub replies: Vec<Vec<u8>>,
    pub changed: bool,
}

/// The recovered ordinary reward toast: `50/1`, zero entries, coins at inner offset 4 and XP at 8.
/// The body order and every fixed field mirror the compiled `mystery_chest::item_reward` writer for
/// the same subtype, so the two cannot drift apart on layout.
pub fn reward_bundle_toast(
    source_guid: u64,
    player_guid: u64,
    coins: u32,
    xp: u32,
) -> io::Result<Vec<u8>> {
    if coins == 0 && xp == 0 {
        return Err(invalid("reward toast must carry coins or XP"));
    }
    if coins > i32::MAX as u32 || xp > i32::MAX as u32 {
        return Err(invalid("reward toast award exceeds the wire field"));
    }
    let mut bytes = Vec::with_capacity(72);
    bytes.extend_from_slice(&REWARD_FAMILY.to_le_bytes());
    bytes.push(REWARD_SUBTYPE_BUNDLE);
    bytes.push(1); // success
    bytes.extend_from_slice(&(coins as i32).to_le_bytes()); // body offset 1, inner offset 4
    bytes.extend_from_slice(&(xp as i32).to_le_bytes()); // body offset 5, inner offset 8
    bytes.extend_from_slice(&0i32.to_le_bytes()); // unresolved, zero
    bytes.extend_from_slice(&REWARD_KIND_CURRENCY.to_le_bytes()); // body offset 13
    bytes.extend_from_slice(&0i32.to_le_bytes()); // unresolved, zero
    bytes.extend_from_slice(&0i32.to_le_bytes()); // unresolved, zero
    bytes.extend_from_slice(&1.0f32.to_le_bytes()); // multiplier
    bytes.extend_from_slice(&0i32.to_le_bytes()); // unresolved, zero
    bytes.extend_from_slice(&0i32.to_le_bytes()); // unresolved, zero
    bytes.extend_from_slice(&source_guid.to_le_bytes());
    bytes.extend_from_slice(&player_guid.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes()); // icon id
    bytes.extend_from_slice(&0i32.to_le_bytes()); // name id
    bytes.extend_from_slice(&0i32.to_le_bytes()); // entry count: none
    bytes.extend_from_slice(&0i32.to_le_bytes()); // appended final word
    Ok(bytes)
}

/// Pay one finished race exactly once.
///
/// Order of authority: the registry must price the speedway, the active job must be the Kart
/// Driver, and `RaceSession::claim_reward` must grant an amount that equals the registration. The
/// economy is then staged on clones and committed only if every grant succeeded, so a refusal
/// leaves the session, the wallet and the class XP exactly as they were.
pub fn claim_race_reward(
    registry: &KartRewardRegistry,
    session: &mut RaceSession,
    inventory: &mut Inventory,
    profiles: &mut Profiles,
    source_guid: u64,
    player_guid: u64,
) -> Result<RewardPayout, RewardError> {
    let snapshot = session.snapshot();
    let speedway = snapshot.speedway;
    let registration = registry
        .registration(speedway)
        .ok_or(RewardError::Unregistered { speedway })?;
    let active = profiles.active();
    if active != KART_DRIVER_JOB_ID {
        return Err(RewardError::MissingKartDriverJob { active });
    }

    // Stage the race first: an independent restore of this session's own state. Every refusal below
    // - not finished, already claimed, clock regression, an unpriced layout - therefore leaves the
    // live session untouched, including its one-shot `reward_claimed` marker.
    let mut staged_race = RaceSession::restore(snapshot).map_err(RewardError::Racing)?;
    let proposal = staged_race.claim_reward().map_err(RewardError::Racing)?;
    let (coins, xp) = match proposal {
        RewardProposal::Granted { coins, xp } => (coins, xp),
        RewardProposal::Unpriced => return Err(RewardError::Unpriced { speedway }),
    };
    if coins != registration.coins || xp != registration.xp {
        // The session's own price is not the registered price for its speedway: refuse instead of
        // paying an amount that no registration authorizes.
        return Err(RewardError::ScheduleNotRegistered {
            speedway,
            coins,
            xp,
        });
    }
    if coins == 0 && xp == 0 {
        return Err(RewardError::EmptySchedule { speedway });
    }

    // Stage the economy on clones. Nothing is committed until every grant and every reply exists.
    let mut staged_inventory = inventory.clone();
    let mut staged_profiles = profiles.clone();
    let previous_xp = staged_profiles
        .state()
        .experience
        .get(&KART_DRIVER_JOB_ID)
        .copied()
        .unwrap_or(0);
    let mut replies = Vec::new();
    if coins > 0 {
        let balance = staged_inventory.state().coins;
        balance
            .checked_add(coins)
            .filter(|total| *total <= i32::MAX as u32)
            .ok_or(RewardError::CoinOverflow {
                balance,
                award: coins,
            })?;
        let outcome = staged_inventory
            .grant_coins(coins)
            .map_err(|error| RewardError::Inventory(error.kind()))?;
        replies.extend(outcome.replies);
    }
    if xp > 0 {
        previous_xp
            .checked_add(xp)
            .ok_or(RewardError::ExperienceOverflow {
                balance: previous_xp,
                award: xp,
            })?;
        staged_profiles
            .add_experience(xp)
            .map_err(|error| RewardError::Profiles(error.kind()))?;
        replies.extend(staged_profiles.experience_packets(previous_xp));
    }
    replies.push(
        reward_bundle_toast(source_guid, player_guid, coins, xp)
            .map_err(|error| RewardError::Toast(error.kind()))?,
    );

    // Commit whole success: three infallible assignments, so a staged payout can never be half
    // applied.
    *inventory = staged_inventory;
    *profiles = staged_profiles;
    *session = staged_race;
    Ok(RewardPayout {
        speedway,
        coins,
        xp,
        previous_xp,
        replies,
        changed: true,
    })
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
