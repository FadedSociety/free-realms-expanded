//! Piece 058 - death and revive: the recovered respawn-window wire, the client revive entry
//! points and the once-only knockout/revive arbiter.
//!
//! Evidence labels follow the other restoration modules. ORIGINAL is read from the supplied
//! client binary, the decompiled client Lua and the original server enum dump; COMMUNITY is the
//! pinned Sanctuary / OSFR sources and the read-only wiki corpus; RECONSTRUCTED is a bounded local
//! policy the lead must replace with original server evidence; MISSING is an explicit gap.
//!
//! ORIGINAL facts:
//! * The encounter family is `BaseEncounterPacket` = outer opcode **41**, a `u16` sub-opcode and a
//!   12-byte header `[i32 encounter id][i32 instance id]`. `EncounterShowRespawnWindowPacket` is
//!   sub-opcode **125**: the client constructor `FUN_008a33b0`
//!   (`lab/decompiled/client/FreeRealms/pseudocode.c`) passes `0x7d` to the family base
//!   constructor `FUN_008a2fd0`, which writes outer `0x29` and the sub-opcode, and the class holds
//!   exactly **two** additional `i32` fields (`param_1[5]`, `param_1[6]`). The shared native reader
//!   `FUN_008d6690` consumes the four header words and `FUN_008d6730` consumes the two trailing
//!   `i32`s, which is the body this module encodes.
//! * The native dispatcher `FUN_0090d830` handles sub `0x7d` by calling `GroupHandler:SetKO` and
//!   then `RespawnWindow:DisplayRespawn`; sub `0x7e` (126) calls `RespawnWindow:Hide`.
//! * The display handler `FUN_00904300` uses its timer argument as milliseconds
//!   (`param_2 / 1000` reaches the label) and substitutes a client-side default when the value is
//!   below 1, so a zero timer means "use the client default".
//! * `ScriptsBase.lua` `RespawnWindow:DisplayRespawn(timer, isOverworld, reviveCost)` shows the
//!   `Revive here:\n %1 coins` dialog only when `isOverworld == 1` and otherwise calls
//!   `StartTimer(timer)`. Its revive entry points are `respawn` -> `Ui.RevivePlayer()` and
//!   `OnPaidRevive(b)` where the decompiled `revive_window.gfx` emits `OnPaidRevive( 1 )` for the
//!   paid button and `OnPaidRevive( 0 )` for the free "Revive at safe location" button (the GFX
//!   and label contract is recovered by work item 037, not duplicated here).
//! * The original server enum `LibSOE/SOE/BaseEncounterPackets.cs` carries
//!   `EncounterShowRespawnWindowPacket = 125`.
//! * The shipped client binary's Lua host-function name table (ASCII table at file offset
//!   21153752, `FreeRealms.exe` sha256 `26cc1c522a45e71fb57882da515f65bac45c8c8b8c7e740c0cea81c52a733351`)
//!   registers `RevivePlayer` and `OnPaidRevive` beside `IsInOverworldCombatArea`,
//!   `IsInOverworldCombat` and `IsInHub`, so the two revive entry points the Lua calls are real
//!   host calls and the dialog choice is a client-side overworld predicate rather than a field of
//!   the respawn window.
//!
//! COMMUNITY facts (never folded into the original section):
//! * The Sanctuary packet-name table maps family 41 sub 125 to `EncounterShowRespawnWindowPacket`
//!   and has no name for 126.
//! * `PacketZoneSafeTeleportRequest` is top-level opcode **122** with an empty body. Two independent
//!   community servers (pinned Sanctuary and the supplied OSFR server) answer it with
//!   `ClientUpdatePacketUpdateLocation` **38/12** and `Teleport = true` at a safe hub point of
//!   interest, and the pinned client expresses the same class (`PacketZoneSafeTeleportRequest`,
//!   `FUN_00bf7a50`). The revive link is inferred: the free client button is the "safe location"
//!   revive, and this is the recovered "take me to a safe spot" request.
//! * The wiki "Combat" page documents a ten-second recovery, a brief invulnerability and a return
//!   at full health and energy, and the "Sunstone Valley" page documents the free safe-zone revive
//!   against a 1,000-coin revive on the spot of defeat.
//!
//! RECONSTRUCTED policy (labelled at each call site): [`ReviveArbiter`] and [`TerminalLedger`].
//! The arbiter is the once-only knockout/revive state machine - a repeated KO message cannot
//! re-show the window, a replayed revive cannot charge twice or heal twice, and a paid revive below
//! cost leaves every value untouched. The ledger keeps one terminal verdict per encounter instance
//! so a replayed terminal message can never mint a reward. No health, cost or timer value is
//! invented here; the caller supplies them.
//!
//! MISSING (explicit gaps):
//! * No retail capture of 41/125 or 41/126 exists in `lab/reports/original-minigame-packets/`, so
//!   the two trailing field *names* follow the native handler's use of its arguments (timer first,
//!   cost second) rather than a captured frame.
//! * The client request that carries the *paid* revive is unrecovered.
//! * The recovery timer, the revive cost and the safe-location choice are server content that this
//!   module never fabricates.
//! * The writer that sets the client's overworld byte (client+0x80) is not recovered; only the
//!   client-side predicates that read that state are named.

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::fmt;

/// `BaseEncounterPacket` outer opcode, shared with `src/encounter.rs`.
pub const ENCOUNTER_OPCODE: u16 = 41;
/// `EncounterShowRespawnWindowPacket` sub-opcode (`FUN_008a33b0` passes `0x7d`).
pub const RESPAWN_WINDOW_SUB: u16 = 125;
/// Sub-opcode that makes the client run `RespawnWindow:Hide` (`FUN_0090d830`, branch `0x7e`).
pub const RESPAWN_HIDE_SUB: u16 = 126;
/// `PacketZoneSafeTeleportRequest` top-level opcode (community wire, community name).
pub const SAFE_TELEPORT_REQUEST_OPCODE: u16 = 122;
/// `ClientUpdatePacketUpdateLocation` opcode used for every relocation.
pub const UPDATE_LOCATION_OPCODE: u16 = 38;
/// `ClientUpdatePacketUpdateLocation` sub-opcode.
pub const UPDATE_LOCATION_SUB: u16 = 12;
/// Bytes of the recovered family header: outer, sub-opcode, encounter id, instance id.
pub const FAMILY_HEADER_LEN: usize = 12;
/// Bytes of a `41/125` respawn window: the family header plus its two `i32` fields.
pub const RESPAWN_WINDOW_LEN: usize = 20;
/// COMMUNITY default offer timer from the wiki "Combat" page: ten seconds of recovery.
/// The real per-encounter value is server content; this constant exists only for callers that
/// have no better value and is never applied by the arbiter on its own.
pub const RECOVER_DELAY_MS: u32 = 10_000;

/// Largest absolute coordinate accepted for a revive destination, matching the local engine's
/// position bound.
const MAX_COORDINATE: f32 = 1_000_000.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviveError {
    /// The character is not knocked out, so a revive message changes nothing.
    NotKnockedOut,
    /// The coin revive was requested below its cost; every value is untouched.
    NotEnoughCoins { wallet: u32, cost: u32 },
    /// The safe revive destination is not a finite, in-bounds position.
    InvalidPosition,
    /// The body does not carry the fields this packet needs.
    MalformedWire(&'static str),
    /// The body belongs to another packet in the same family.
    WrongPacket { expected: u16, found: u16 },
}

impl fmt::Display for ReviveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotKnockedOut => f.write_str("the character is not knocked out"),
            Self::NotEnoughCoins { wallet, cost } => {
                write!(f, "revive costs {cost} coins but the wallet holds {wallet}")
            }
            Self::InvalidPosition => f.write_str("the revive destination is not a finite position"),
            Self::MalformedWire(reason) => f.write_str(reason),
            Self::WrongPacket { expected, found } => {
                write!(f, "expected sub-opcode {expected}, found {found}")
            }
        }
    }
}

impl std::error::Error for ReviveError {}

/// The recovery offer the server shows for one knockout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RespawnOffer {
    pub encounter_id: u32,
    pub instance_id: i32,
    /// Milliseconds; zero asks the client for its own default (`FUN_00904300`).
    pub timer_ms: u32,
}

/// The `41/125` `EncounterShowRespawnWindowPacket` body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RespawnWindow {
    pub encounter_id: u32,
    pub instance_id: i32,
    /// Milliseconds; the native handler divides by 1000 for its label.
    pub timer_ms: u32,
    /// Coins the "Revive here" button charges; `0` disables the paid path in the client guard.
    pub revive_cost: u32,
}

impl RespawnWindow {
    /// Encodes the recovered family header plus the two trailing `i32` fields.
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = family_header(RESPAWN_WINDOW_SUB, self.encounter_id, self.instance_id);
        bytes.extend((self.timer_ms as i32).to_le_bytes());
        bytes.extend((self.revive_cost as i32).to_le_bytes());
        bytes
    }

    /// Builds the window for an offer with the arbiter's configured cost.
    pub fn from_offer(offer: &RespawnOffer, revive_cost: u32) -> Self {
        Self {
            encounter_id: offer.encounter_id,
            instance_id: offer.instance_id,
            timer_ms: offer.timer_ms,
            revive_cost,
        }
    }
}

fn family_header(sub: u16, encounter_id: u32, instance_id: i32) -> Vec<u8> {
    let mut bytes = ENCOUNTER_OPCODE.to_le_bytes().to_vec();
    bytes.extend(sub.to_le_bytes());
    bytes.extend((encounter_id as i32).to_le_bytes());
    bytes.extend(instance_id.to_le_bytes());
    bytes
}

fn read_i32(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

/// Reads a `41/125` body. The field order (encounter id, instance, timer, cost) follows the family
/// header plus the native handler's use of its two arguments; it is not proven by a capture.
pub fn decode_respawn_window(bytes: &[u8]) -> Result<RespawnWindow, ReviveError> {
    if bytes.len() < 4 {
        return Err(ReviveError::MalformedWire(
            "expected at least 4 header bytes",
        ));
    }
    let opcode = u16::from_le_bytes([bytes[0], bytes[1]]);
    if opcode != ENCOUNTER_OPCODE {
        return Err(ReviveError::WrongPacket {
            expected: ENCOUNTER_OPCODE,
            found: opcode,
        });
    }
    let sub = u16::from_le_bytes([bytes[2], bytes[3]]);
    if sub != RESPAWN_WINDOW_SUB {
        return Err(ReviveError::WrongPacket {
            expected: RESPAWN_WINDOW_SUB,
            found: sub,
        });
    }
    if bytes.len() < RESPAWN_WINDOW_LEN {
        return Err(ReviveError::MalformedWire(
            "expected the two respawn fields",
        ));
    }
    if bytes.len() > RESPAWN_WINDOW_LEN {
        return Err(ReviveError::MalformedWire("unexpected trailing bytes"));
    }
    Ok(RespawnWindow {
        encounter_id: read_i32(bytes, 4) as u32,
        instance_id: read_i32(bytes, 8),
        timer_ms: read_i32(bytes, 12) as u32,
        revive_cost: read_i32(bytes, 16) as u32,
    })
}

/// The bare `41/126` that makes the client run `RespawnWindow:Hide`. The native reader consumes the
/// standard 12-byte family header before it branches on the sub-opcode.
pub fn encode_respawn_hide(encounter_id: u32, instance_id: i32) -> Vec<u8> {
    family_header(RESPAWN_HIDE_SUB, encounter_id, instance_id)
}

/// Validates the bare top-level 122 `PacketZoneSafeTeleportRequest` the client sends from the
/// "Revive at safe location: Free" side (community wire; see the module header).
pub fn decode_safe_teleport_request(bytes: &[u8]) -> Result<(), ReviveError> {
    if bytes.len() < 2 {
        return Err(ReviveError::MalformedWire(
            "expected a 2-byte packet header",
        ));
    }
    let opcode = u16::from_le_bytes([bytes[0], bytes[1]]);
    if opcode != SAFE_TELEPORT_REQUEST_OPCODE {
        return Err(ReviveError::WrongPacket {
            expected: SAFE_TELEPORT_REQUEST_OPCODE,
            found: opcode,
        });
    }
    if bytes.len() > 2 {
        return Err(ReviveError::MalformedWire(
            "safe teleport request has no body",
        ));
    }
    Ok(())
}

/// The `38/12` `ClientUpdatePacketUpdateLocation` used for every relocation, byte-identical to the
/// pinned reviewed `dev_teleport::location_packet`: position `XYZ` with `W = 1`, rotation `XYZ`
/// with `W = 0`, the `Teleport` flag and the writer's default trailing byte.
pub fn encode_teleport_location(position: [f32; 3], rotation: [f32; 3]) -> Vec<u8> {
    let mut bytes = UPDATE_LOCATION_OPCODE.to_le_bytes().to_vec();
    bytes.extend(UPDATE_LOCATION_SUB.to_le_bytes());
    for value in position.into_iter().chain([1.0_f32]) {
        bytes.extend(value.to_le_bytes());
    }
    for value in rotation.into_iter().chain([0.0_f32]) {
        bytes.extend(value.to_le_bytes());
    }
    bytes.push(1);
    bytes.push(0);
    bytes
}

fn valid_vector(value: [f32; 3]) -> bool {
    value
        .iter()
        .all(|v| v.is_finite() && v.abs() <= MAX_COORDINATE)
}

/// How the character gets up.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReviveChoice {
    /// `Revive at safe location: Free` (`OnPaidRevive( 0 )`): free, and the character relocates to
    /// the caller-supplied safe destination.
    Safe {
        position: [f32; 3],
        rotation: [f32; 3],
    },
    /// `Revive here: N coins` (`OnPaidRevive( 1 )`): the character stays where it fell and pays the
    /// recovered cost once.
    Paid,
}

/// The server-side result of one accepted revive.
#[derive(Clone, Debug, PartialEq)]
pub struct ReviveOutcome {
    /// `41/126` when the server had shown a window for this knockout; `None` when the knockout was
    /// engine-driven and no offer was recorded.
    pub hide: Option<Vec<u8>>,
    /// `38/12` with `Teleport` set, only for the free safe revive.
    pub relocation: Option<Vec<u8>>,
    /// Coins actually charged: zero for the safe revive, the cost for the coin revive.
    pub charged_coins: u32,
    /// Health after the revive. COMMUNITY: the recovery restores full health.
    pub health: u32,
}

/// Once-only knockout and revive bookkeeping for one character.
///
/// Every transition is idempotent under replayed terminal messages: a destructive hit that lands on
/// zero records the knockout and offers the window exactly once, a repeated knockout message changes
/// nothing, and a replayed revive is refused without charging coins or healing again.
#[derive(Clone, Debug)]
pub struct ReviveArbiter {
    max_health: u32,
    health: u32,
    coins: u32,
    revive_cost: u32,
    knocked_out: bool,
    window_emitted: bool,
    offer: Option<RespawnOffer>,
    knockouts: u32,
    revives: u32,
}

impl ReviveArbiter {
    /// `max_health`, the current wallet and the revived-at-cost come from the caller's content; no
    /// balance value is assumed here.
    pub fn new(max_health: u32, coins: u32, revive_cost: u32) -> Self {
        Self {
            max_health,
            health: max_health,
            coins,
            revive_cost,
            knocked_out: false,
            window_emitted: false,
            offer: None,
            knockouts: 0,
            revives: 0,
        }
    }

    pub fn max_health(&self) -> u32 {
        self.max_health
    }

    pub fn health(&self) -> u32 {
        self.health
    }

    pub fn coins(&self) -> u32 {
        self.coins
    }

    pub fn revive_cost(&self) -> u32 {
        self.revive_cost
    }

    pub fn knocked_out(&self) -> bool {
        self.knocked_out
    }

    pub fn knockouts(&self) -> u32 {
        self.knockouts
    }

    pub fn revives(&self) -> u32 {
        self.revives
    }

    /// Applies one damage instance. Returns `true` only for the hit that reaches zero health, which
    /// is the point where the caller stages the knockout. Damage received while already knocked out
    /// is discarded, so a replayed terminal message cannot move health.
    pub fn apply_damage(&mut self, damage: u32) -> bool {
        if self.knocked_out {
            return false;
        }
        self.health = self.health.saturating_sub(damage);
        if self.health == 0 {
            self.knocked_out = true;
            self.window_emitted = false;
            self.knockouts = self.knockouts.saturating_add(1);
            return true;
        }
        false
    }

    /// Records the knockout and returns the `41/125` body the first time it is asked for.
    ///
    /// The first call for a knockout records the offer and answers with the encoded window; every
    /// later call for the same knockout answers `None`, so a duplicated knockout message cannot
    /// re-open the window or extend the offer.
    pub fn knock_out(&mut self, offer: &RespawnOffer) -> Option<Vec<u8>> {
        if !self.knocked_out {
            self.health = 0;
            self.knocked_out = true;
            self.knockouts = self.knockouts.saturating_add(1);
            self.window_emitted = false;
        }
        if self.window_emitted {
            return None;
        }
        self.window_emitted = true;
        self.offer = Some(*offer);
        Some(RespawnWindow::from_offer(offer, self.revive_cost).encode())
    }

    /// Applies one revive request. A revive while standing, a repeated revive and a coin revive
    /// below cost are refused with no state change at all.
    pub fn revive(&mut self, choice: ReviveChoice) -> Result<ReviveOutcome, ReviveError> {
        if !self.knocked_out {
            return Err(ReviveError::NotKnockedOut);
        }
        let relocation = match choice {
            ReviveChoice::Safe { position, rotation } => {
                if !valid_vector(position) || !valid_vector(rotation) {
                    return Err(ReviveError::InvalidPosition);
                }
                Some(encode_teleport_location(position, rotation))
            }
            ReviveChoice::Paid => {
                if self.coins < self.revive_cost {
                    return Err(ReviveError::NotEnoughCoins {
                        wallet: self.coins,
                        cost: self.revive_cost,
                    });
                }
                None
            }
        };
        let charged_coins = if relocation.is_some() {
            0
        } else {
            self.revive_cost
        };
        self.coins -= charged_coins;
        self.health = self.max_health;
        self.knocked_out = false;
        self.window_emitted = false;
        self.revives = self.revives.saturating_add(1);
        let hide = self
            .offer
            .take()
            .map(|offer| encode_respawn_hide(offer.encounter_id, offer.instance_id));
        Ok(ReviveOutcome {
            hide,
            relocation,
            charged_coins,
            health: self.health,
        })
    }
}

/// One terminal verdict per encounter instance.
///
/// RECONSTRUCTED policy for the "no reward exploit through repeated terminal messages" gate: the
/// first terminal message of an instance decides whether it pays out, and every replay of that
/// message - or a replay that claims a loss was a win - returns `false`. A knocked-out instance can
/// therefore never be re-settled into a reward.
#[derive(Clone, Debug, Default)]
pub struct TerminalLedger {
    settled: BTreeMap<i32, bool>,
}

impl TerminalLedger {
    /// Records the terminal outcome of `instance_id`. Returns `true` only for the first settlement
    /// of a won instance; replays and losses never grant.
    pub fn settle(&mut self, instance_id: i32, won: bool) -> bool {
        match self.settled.entry(instance_id) {
            Entry::Vacant(slot) => {
                slot.insert(won);
                won
            }
            Entry::Occupied(_) => false,
        }
    }

    /// The recorded verdict for an instance, if its terminal message already arrived.
    pub fn settled(&self, instance_id: i32) -> Option<bool> {
        self.settled.get(&instance_id).copied()
    }

    pub fn len(&self) -> usize {
        self.settled.len()
    }

    pub fn is_empty(&self) -> bool {
        self.settled.is_empty()
    }
}
