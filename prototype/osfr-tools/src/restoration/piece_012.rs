//! Piece 012: recovered Cooking recipe inventory and repeated-transaction audit.
//!
//! Original facts: wiki revision 45318 "List of Recipes (Chef)" lists Strong
//! Arm Stew (tutorial recipe) as Bat Meat x4, Bundle of Gummyfruit x4, Bundle
//! of Rainleaf x4 and Spice x4; the preserved item definitions resolve those
//! names to ids 2862/2875/2884/2900 and the output to definition 35.
//! Reconstructed policy: the fixed one-item output, the six-phase order and
//! all numeric phase tuning are explicit reconstructions carried by the
//! content file's own labels, not recovered server recipe rows.
//!
//! The audit drives the unmodified production `osfr_tools::cooking` service
//! through repeated completions and checks that every settlement is atomic
//! and once-only: one transaction per victory, four of each material consumed,
//! one output granted, no re-settlement on retransmission or reload, and no
//! partial consumption when the ledger is short.
use osfr_tools::cooking::{Cooking, CookingOutcome, CookingTransaction, payload_packet};
use std::{collections::BTreeMap, io, path::Path};

/// Recovered recipe materials: definition id and quantity per completion.
pub const RECOVERED_MATERIALS: [(i32, u32); 4] = [(2862, 4), (2875, 4), (2884, 4), (2900, 4)];
/// Reconstructed fixed output: one Strong Arm Stew (definition 35).
pub const RECOVERED_PRODUCT: (i32, u32) = (35, 1);
/// Recovered original stage types for the six ordered report groups.
pub const RECOVERED_STAGE_TYPES: [i32; 6] = [1, 2, 3, 6, 17, 7];
/// Mirrors the production service's first instance id for audit assertions.
pub const FIRST_INSTANCE_ID: i32 = 46_000_001;

const WIN_MESSAGE: &str = "FRServer_GameWon\t900\t0";
const MAX_CYCLES: u32 = 64;

/// Minimal material ledger used to check a settlement is all-or-nothing. It is
/// an audit model, not part of the authoritative gameplay economy.
#[derive(Clone, Debug, Default)]
pub struct RecipeLedger {
    stock: BTreeMap<i32, u64>,
    granted: BTreeMap<i32, u64>,
    applied: u64,
}

impl RecipeLedger {
    pub fn seeded(stock: impl IntoIterator<Item = (i32, u64)>) -> Self {
        let mut ledger = Self::default();
        for (id, count) in stock {
            *ledger.stock.entry(id).or_default() += count;
        }
        ledger
    }

    pub fn count(&self, id: i32) -> u64 {
        self.stock.get(&id).copied().unwrap_or(0)
    }

    pub fn granted(&self, id: i32) -> u64 {
        self.granted.get(&id).copied().unwrap_or(0)
    }

    pub fn applied(&self) -> u64 {
        self.applied
    }

    /// Applies one settlement atomically: the whole transaction is validated
    /// against aggregated per-material demand before any state changes.
    pub fn apply(&mut self, transaction: &CookingTransaction) -> io::Result<()> {
        let mut required: BTreeMap<i32, u64> = BTreeMap::new();
        for (id, count) in &transaction.consumes {
            let slot = required.entry(*id).or_default();
            *slot = slot
                .checked_add(u64::from(*count))
                .ok_or_else(|| invalid("recipe ledger consumption overflow"))?;
        }
        if required.iter().any(|(id, count)| self.count(*id) < *count) {
            return Err(invalid("recipe ledger lacks the required materials"));
        }
        for (id, count) in required {
            *self.stock.entry(id).or_default() -= count;
        }
        for (id, count) in &transaction.grants {
            let slot = self.granted.entry(*id).or_default();
            *slot = slot
                .checked_add(u64::from(*count))
                .ok_or_else(|| invalid("recipe ledger grant overflow"))?;
        }
        self.applied += 1;
        Ok(())
    }

    fn remaining(&self) -> Vec<(i32, u64)> {
        self.stock
            .iter()
            .filter(|(_, count)| **count > 0)
            .map(|(id, count)| (*id, *count))
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepeatAudit {
    pub cycles: u32,
    pub consumed: Vec<(i32, u64)>,
    pub granted: Vec<(i32, u64)>,
    pub live_replays_ignored: u32,
    pub reload_replays_ignored: u32,
    pub completed: u32,
    pub next_instance_id: i32,
    pub remaining_stock: Vec<(i32, u64)>,
}

/// Plays `cycles` complete recipe sessions against the preserved content and
/// returns the settlement accounting. The caller supplies only a content
/// directory; the service itself is the unmodified production implementation.
pub fn audit_repeated_transactions(content_dir: &Path, cycles: u32) -> io::Result<RepeatAudit> {
    if cycles == 0 || cycles > MAX_CYCLES {
        return Err(invalid("cooking audit cycle count out of range"));
    }
    let mut game = Cooking::load(content_dir, None)?;
    if game.requirements() != RECOVERED_MATERIALS.as_slice() {
        return Err(invalid(
            "cooking content does not declare the recovered recipe materials",
        ));
    }
    let mut ledger = RecipeLedger::seeded(
        game.requirements()
            .iter()
            .map(|(id, count)| (*id, u64::from(*count) * u64::from(cycles))),
    );
    let mut consumed: BTreeMap<i32, u64> = BTreeMap::new();
    let mut granted: BTreeMap<i32, u64> = BTreeMap::new();
    let mut live_replays_ignored = 0;
    let mut reload_replays_ignored = 0;
    for _ in 0..cycles {
        let (instance_id, won) = play_completion(&mut game)?;
        let transaction = won
            .transaction
            .ok_or_else(|| invalid("cooking completion produced no transaction"))?;
        if transaction.consumes != RECOVERED_MATERIALS {
            return Err(invalid("cooking completion consumed an unexpected recipe"));
        }
        if transaction.grants != vec![RECOVERED_PRODUCT] {
            return Err(invalid("cooking completion granted an unexpected product"));
        }
        ledger.apply(&transaction)?;
        for (id, count) in &transaction.consumes {
            *consumed.entry(*id).or_default() += u64::from(*count);
        }
        for (id, count) in &transaction.grants {
            *granted.entry(*id).or_default() += u64::from(*count);
        }
        // Reliable retransmission of the same victory must not settle twice.
        let replay = game.handle(&native_message(instance_id, WIN_MESSAGE)?)?;
        if replay.transaction.is_some() || replay.changed {
            return Err(invalid("cooking victory replay changed settled state"));
        }
        live_replays_ignored += 1;
        // Neither must a restart from the persisted state.
        let restored_state = game.state().clone();
        let mut restored = Cooking::load(content_dir, Some(restored_state))?;
        let reloaded = restored.handle(&native_message(instance_id, WIN_MESSAGE)?)?;
        if reloaded.transaction.is_some() {
            return Err(invalid("cooking victory settled again after reload"));
        }
        reload_replays_ignored += 1;
        close_completion(&mut game, instance_id)?;
    }
    if game.state().completed != cycles {
        return Err(invalid("cooking completion count does not match the audit"));
    }
    Ok(RepeatAudit {
        cycles,
        consumed: consumed.into_iter().collect(),
        granted: granted.into_iter().collect(),
        live_replays_ignored,
        reload_replays_ignored,
        completed: game.state().completed,
        next_instance_id: game.state().next_instance_id,
        remaining_stock: ledger.remaining(),
    })
}

/// One session: offer, native start, phase data, six ordered round report
/// groups of four statistics, four final statistics and the win report.
fn play_completion(game: &mut Cooking) -> io::Result<(i32, CookingOutcome)> {
    game.begin(45)?;
    let instance_id = game
        .state()
        .active
        .as_ref()
        .ok_or_else(|| invalid("cooking run did not open"))?
        .instance_id;
    game.handle(&start_request(instance_id))?;
    game.handle(&native_message(instance_id, "FRServer_ReadyForGameData")?)?;
    for stage in RECOVERED_STAGE_TYPES {
        for kind in 0..4 {
            let report = format!("FRServer_StatUpdate\t{stage}\t{kind}\t1)");
            game.handle(&native_message(instance_id, &report)?)?;
        }
    }
    for kind in 4..8 {
        let report = format!("FRServer_StatUpdate\t-1\t{kind}\t6)");
        game.handle(&native_message(instance_id, &report)?)?;
    }
    let won = game.handle(&native_message(instance_id, WIN_MESSAGE)?)?;
    Ok((instance_id, won))
}

fn close_completion(game: &mut Cooking, instance_id: i32) -> io::Result<()> {
    let closed = game.handle(&native_message(instance_id, "FRServer_GameClose")?)?;
    if game.state().active.is_some() {
        return Err(invalid("cooking run stayed active after close"));
    }
    if closed.transaction.is_some() {
        return Err(invalid("closing a cooking run settled a transaction"));
    }
    Ok(())
}

fn start_request(instance_id: i32) -> Vec<u8> {
    let mut packet = vec![39, 0, 5];
    for value in [instance_id, -1, -1] {
        packet.extend(value.to_le_bytes());
    }
    packet
}

/// Native `SendPayload` writes the -1 game sentinel (proof in the private
/// client header report); the production guard admits it for live runs.
fn native_message(instance_id: i32, message: &str) -> io::Result<Vec<u8>> {
    let mut packet = payload_packet(instance_id, message)?;
    packet[11..15].copy_from_slice(&(-1i32).to_le_bytes());
    Ok(packet)
}

fn invalid(message: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_string())
}
