//! Knockout and revive: the respawn window, the client's resume request and
//! the reconstructed revive rules (moved out of `encounter.rs` by T402).
//!
//! Client facts (`FreeRealms.exe`): 41/125 `EncounterShowRespawnWindowPacket`
//! and 41/126 `RespawnWindow:Hide` (below). 41/122
//! `EncounterParticipantResumePacket` is built by `008a3250` over the family
//! constructor `008a2fd0`, which zeroes both header ids, and carries one bool
//! at +0x14: 1 from the paid sender `0093be30` (Lua `Ui.OnPaidRevive(b)`), 0
//! from the free sender `0093bd80` (`Ui.RevivePlayer()`). Neither sender writes
//! a place or a price. `ScriptsBase.lua` `RespawnWindow:DisplayRespawn(timer,
//! isOverworld, reviveCost)` offers "Revive here: %1 coins" only in the
//! overworld and hides the window on either click (`OnUserEvent`).
//!
//! (R) Everything else: holding the knockout, the safe location, the grace,
//! the server fallback, the price and the paid revive in place.
use super::*;

/// (R) Overworld knockout window, in ms: the ten-second recovery of the wiki
/// Combat page (research piece 058). The client counts it down only when it
/// does not show the two-button dialog, and then sends the free revive itself.
const REVIVE_TIMER_MS: i32 = 10_000;
/// (R) Field enemies ignore a freshly revived or recovered player this long.
const FIELD_GRACE_MS: u64 = 8_000;
/// (R) Server fallback: an overworld knockout with no 41/122 by then is revived
/// at the safe location anyway, so a client that never shows the window (its
/// 41/125 field order is inferred, not captured) cannot leave a player down.
pub(super) const AUTO_REVIVE_MS: u64 = 15_000;
/// (R) The retail spot-revive price, 1,000 coins (the window's own default
/// `setReviveCost_lua(1000)` in `UI/revive_window.gfx`; the wiki's Sunstone
/// Valley page says 1,000). The shipped value lives in `field_enemies.json`
/// `paid_revive`; this is only the price of a package without that block.
pub(super) const RETAIL_REVIVE_COST: u32 = 1_000;
/// (R) Sanity ceiling for the data price (it also keeps the wire `i32`
/// positive), not a tuning pin.
const MAX_REVIVE_COST: u32 = 1_000_000;

/// `field_enemies.json` `paid_revive`: the "Revive here" price of an overworld
/// knockout and its provenance. The client greys the paid button while the
/// wallet is below it and enables it at or above (`showTwoButtons_lua`); it
/// never hides the button, so 0 makes the spot revive free.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PaidRevive {
    cost: u32,
    source: Vec<crate::content::SourceTag>,
}

impl PaidRevive {
    /// The price, or a load refusal for nonsense: above the ceiling, no
    /// provenance, or a reconstructed tag without its `(R)`.
    pub(super) fn cost(&self) -> io::Result<u32> {
        if self.cost > MAX_REVIVE_COST || field_rewards::validate_source(&self.source).is_err() {
            return Err(invalid("invalid paid revive content"));
        }
        Ok(self.cost)
    }
}

impl EncounterEngine {
    /// 41/122 ParticipantResume: header, then a bool, 1 for the paid revive
    /// (`0093be30`, Lua `OnPaidRevive`) and 0 for the free one (`0093bd80`,
    /// `RevivePlayer`, and the safe-location button). Only the session's own
    /// knocked-out state is revived; the client sends no place or price (R).
    pub(super) fn resume(&mut self, inner: &[u8], now_ms: u64) -> io::Result<EncounterOutcome> {
        if inner.len() != 13 || inner[12] > 1 {
            return Err(invalid("malformed encounter resume"));
        }
        if inner[12] == 1 {
            // The wallet lives in Gameplay, which answers a well-formed paid
            // request with `paid_revive` before this handler sees it. A caller
            // without a wallet has no coins to pay with.
            return Ok(self.paid_revive(0, now_ms).0);
        }
        if self.state.player_health != 0 {
            return Ok(EncounterOutcome::default());
        }
        if let Some(a) = &self.state.active {
            // A lost instance revives through its existing exit (return
            // location, full health). A header naming another instance is a
            // replay from an earlier one.
            let encounter = i32::from_le_bytes(inner[4..8].try_into().unwrap());
            let instance = i32::from_le_bytes(inner[8..12].try_into().unwrap());
            if a.phase == EncounterPhase::Lost
                && (encounter == 0 || encounter == a.encounter_id as i32)
                && (instance == 0 || instance == a.instance_id)
            {
                return self.exit();
            }
            return Ok(EncounterOutcome::default());
        }
        Ok(self.revive_at_safe_location(now_ms))
    }

    /// (R) Paid revive (41/122 with the flag set): the free revive's health,
    /// grace and packets where the player fell, without zoning. `wallet` is the
    /// live coin balance; the second value is the price the caller charges in
    /// the same commit (`Gameplay::handle_paid_revive`). A refusal changes
    /// nothing, charges 0 and says why in one line; below the price the window
    /// comes back, because the click hid it.
    pub fn paid_revive(&mut self, wallet: u32, now_ms: u64) -> (EncounterOutcome, u32) {
        let refuse = |text: String, replies: Vec<Vec<u8>>| {
            let out = EncounterOutcome {
                replies,
                feedback: Some(text),
                ..Default::default()
            };
            (out, 0)
        };
        if self.state.player_health != 0 {
            return refuse("You are not knocked out.".into(), Vec::new());
        }
        if self.state.active.is_some() {
            // The client offers the paid button only in the overworld.
            return refuse(
                "Paying to revive is not available. Choose the free revive.".into(),
                Vec::new(),
            );
        }
        let cost = self.paid_revive_cost;
        if wallet < cost {
            return refuse(
                format!("You need {cost} coins to revive here."),
                vec![self.respawn_window()],
            );
        }
        let mut out = self.revive(now_ms);
        out.feedback = Some("You revive where you fell.".into());
        (out, cost)
    }

    /// (R) Safe location: the world checkpoint the opening already uses.
    pub(super) fn revive_at_safe_location(&mut self, now_ms: u64) -> EncounterOutcome {
        EncounterOutcome {
            zoning: Some(self.content.world.clone()),
            feedback: Some("You revive at the safe location.".into()),
            ..self.revive(now_ms)
        }
    }

    /// Both revives: full health, field grace, the knockout cleared and the
    /// window hidden.
    fn revive(&mut self, now_ms: u64) -> EncounterOutcome {
        self.state.player_health = self.max_health;
        self.field_grace_until = now_ms.saturating_add(FIELD_GRACE_MS);
        self.knocked_out_at = None;
        EncounterOutcome {
            changed: true,
            replies: vec![player_hp(self.max_health, self.max_health), respawn_hide()],
            ..Default::default()
        }
    }

    /// 41/125 EncounterShowRespawnWindowPacket: dispatcher `0090d830`, ctor
    /// `008a33b0`, reader `008d6730`. Family header (ids zero: the overworld has
    /// no instance) then two i32 handed to `DisplayRespawnWindow` `00904300`,
    /// which takes (timer ms, revive cost). The field order is inferred from
    /// that handler, not captured.
    pub(super) fn respawn_window(&self) -> Vec<u8> {
        let mut b = encounter_header(125, 0, 0);
        b.extend(REVIVE_TIMER_MS.to_le_bytes());
        // Bounded by MAX_REVIVE_COST at load, so it fits the wire i32.
        b.extend((self.paid_revive_cost as i32).to_le_bytes());
        b
    }
}

/// 41/126: the dispatcher runs `RespawnWindow:Hide` (family header only).
pub(super) fn respawn_hide() -> Vec<u8> {
    encounter_header(126, 0, 0)
}
