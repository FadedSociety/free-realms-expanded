//! 081: inventory instance lifecycle. A monotonic per-character instance id counter.
//!
//! The prepared inventory already owns item rows: definition, tint, instance id, count, consumed
//! count, rental and ability fields (`src/inventory.rs`). Production allocates a new instance as
//! `max(live id) + 1`, so retiring the highest row frees its id for the next grant, and a restart
//! repeats the same rule. This module owns only the missing piece: the durable high-water mark, the
//! single allocator, and the invariant that a live row can never carry an id the counter will hand
//! out again.
//!
//! ORIGINAL facts used here (all outside Git, see
//! `notes/Research/Pieces/081 Inventory instance lifecycle.md`):
//! * `BaseItemDefinition.MaxStackSize` is an original definition field, written into the 35/37
//!   catalogue by `Gateway.Player.PlayerCode.SendPlayerUpdateItemDefinitions`
//!   (`lab/decompiled/server/Gateway/Gateway/BaseItemDefinition.cs` line 59).
//! * the client and the prototype gateway serialize the same item row, instance id included
//!   (`lab/decompiled/server/Gateway/Gateway.Player/PlayerCode.cs`,
//!   `lab/references/Sanctuary/src/Sanctuary.Packet.Common/ItemInstance.cs`).
//! * the preserved original character self holds 1386 live instance ids inside 2..=4595 with 3209
//!   gaps, evidence that the original id space was not compacted after removals
//!   (`lab/references/rust-gameplay/inventory.json`).
//!
//! RECONSTRUCTED here: the counter document and its schema, the floor-on-open reconciliation, the
//! write-before-hand-out ordering, every numeric bound and every error. No original server
//! allocation routine was recovered, so "never reuse an id" is a local invariant justified by the
//! sparse captured id space, not a native rule. The original client has not accepted anything this
//! module produces.
//!
//! Deliberate boundaries and one explicit gap:
//! * the counter does not read, rewrite or validate item rows; rows stay owned by `inventory.rs`;
//! * `open` floors the durable value to `max(stored, highest observed live id + 1)`. Observe every
//!   id the character still owns, and also the prepared catalogue's own ids, because the floor can
//!   only respect ids the caller supplies;
//! * if the counter document is lost, ids that were retired above every observed id are
//!   unrecoverable and can be handed out again. Only the durable document records them; this module
//!   does not fabricate a recovered rule to close that case;
//! * stack limits, merge, atomic add/remove and the 38/2, 38/3 and 38/4 replies already live in
//!   `inventory.rs` and are exercised, not reimplemented, by this item's tests.
//!
//! ponytail: one document holding one integer, one floor on open, one write per hand-out. No
//! journal, no per-definition counters, no reserved-id table. A failed grant burns its id and
//! leaves a permanent gap, which is what the captured id space already looks like.

use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

/// RECONSTRUCTED schema version of the counter document.
pub const COUNTER_SCHEMA_VERSION: u32 = 1;
/// RECONSTRUCTED document bound; one integer is not expected to grow.
pub const MAX_COUNTER_BYTES: usize = 4 * 1024;
/// The first instance id the counter can hand out. The captured character starts at id 2.
pub const FIRST_INSTANCE_ID: i32 = 1;

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Allocate inside an atomic character snapshot. The caller persists the returned
/// high-water mark together with the granted row; legacy snapshots lack it.
pub fn allocate_from_snapshot(
    stored: Option<i64>,
    live_ids: &[i32],
) -> Result<(i32, i64), CounterError> {
    let highest = InstanceCounter::highest(live_ids)?;
    let stored = stored.unwrap_or(i64::from(FIRST_INSTANCE_ID));
    if !(1..=i64::from(i32::MAX) + 1).contains(&stored) {
        return Err(CounterError::Corrupt("invalid snapshot instance counter"));
    }
    let next = stored.max(i64::from(highest) + 1);
    let id = i32::try_from(next).map_err(|_| CounterError::Exhausted)?;
    Ok((id, next + 1))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CounterError {
    /// The document could not be read or replaced.
    Io(io::ErrorKind),
    /// The document is not a bounded, valid, supported counter.
    Corrupt(&'static str),
    /// Every representable instance id has been handed out.
    Exhausted,
    /// A live row carries an id that is not a positive instance id.
    InvalidInstanceId { id: i32 },
    /// The same live id appears twice, so the highest id is ambiguous.
    DuplicateInstanceId { id: i32 },
    /// The counter is not above a live row; the next hand-out would collide.
    BehindHighestInstance { next: i64, highest: i32 },
}

#[derive(Clone, Debug)]
pub struct InstanceCounter {
    next_instance_id: i64,
    path: PathBuf,
}

impl InstanceCounter {
    /// Read the durable counter and floor it above every observed live instance id. A missing
    /// document is a fresh character; a repaired floor is written before the counter is returned,
    /// so a reconciled value cannot be lost by a later failed hand-out.
    pub fn open(path: impl Into<PathBuf>, live_instance_ids: &[i32]) -> Result<Self, CounterError> {
        let path = path.into();
        let highest = Self::highest(live_instance_ids)?;
        let stored = match read_bounded(&path) {
            Ok(Some(bytes)) => CounterDocument::from_bytes(&bytes)?.next_instance_id,
            Ok(None) => i64::from(FIRST_INSTANCE_ID),
            Err(error) if error.kind() == io::ErrorKind::InvalidData => {
                return Err(CounterError::Corrupt(
                    "counter document is not a bounded regular file",
                ));
            }
            Err(error) => return Err(CounterError::Io(error.kind())),
        };
        let floor = stored
            .max(i64::from(highest) + 1)
            .max(i64::from(FIRST_INSTANCE_ID));
        let counter = Self {
            next_instance_id: floor,
            path,
        };
        if floor != stored {
            counter.persist(floor)?;
        }
        Ok(counter)
    }

    /// The id the next successful hand-out returns.
    pub fn next_instance_id(&self) -> i64 {
        self.next_instance_id
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Hand out the next instance id, durably recording the following value first. The id is burned
    /// even when the caller's grant fails afterwards; that is what keeps ids unique.
    pub fn allocate(&mut self) -> Result<i32, CounterError> {
        let id = i32::try_from(self.next_instance_id).map_err(|_| CounterError::Exhausted)?;
        let next = self.next_instance_id + 1;
        self.persist(next)?;
        self.next_instance_id = next;
        Ok(id)
    }

    /// Independent check: every live id is usable and unique, and the counter leads them all.
    /// Returns the highest live id.
    pub fn verify(&self, live_instance_ids: &[i32]) -> Result<i32, CounterError> {
        let highest = Self::highest(live_instance_ids)?;
        if self.next_instance_id <= i64::from(highest) {
            return Err(CounterError::BehindHighestInstance {
                next: self.next_instance_id,
                highest,
            });
        }
        Ok(highest)
    }

    fn persist(&self, next_instance_id: i64) -> Result<(), CounterError> {
        let document = CounterDocument {
            schema_version: COUNTER_SCHEMA_VERSION,
            next_instance_id,
        };
        let bytes = document.to_bytes()?;
        atomic_write(&self.path, &bytes).map_err(|error| CounterError::Io(error.kind()))
    }

    fn highest(live_instance_ids: &[i32]) -> Result<i32, CounterError> {
        let mut highest = 0i32;
        let mut seen = BTreeSet::new();
        for id in live_instance_ids {
            if *id < FIRST_INSTANCE_ID {
                return Err(CounterError::InvalidInstanceId { id: *id });
            }
            if !seen.insert(*id) {
                return Err(CounterError::DuplicateInstanceId { id: *id });
            }
            highest = highest.max(*id);
        }
        Ok(highest)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CounterDocument {
    schema_version: u32,
    next_instance_id: i64,
}

impl CounterDocument {
    fn to_bytes(&self) -> Result<Vec<u8>, CounterError> {
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|_| CounterError::Corrupt("counter document does not serialize"))?;
        if bytes.len() > MAX_COUNTER_BYTES {
            return Err(CounterError::Corrupt("counter document exceeds its bound"));
        }
        Ok(bytes)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, CounterError> {
        let document: Self = serde_json::from_slice(bytes)
            .map_err(|_| CounterError::Corrupt("counter document is not a valid document"))?;
        if document.schema_version != COUNTER_SCHEMA_VERSION {
            return Err(CounterError::Corrupt(
                "unsupported counter document version",
            ));
        }
        if document.next_instance_id < i64::from(FIRST_INSTANCE_ID) {
            return Err(CounterError::Corrupt(
                "counter is below the first instance id",
            ));
        }
        Ok(document)
    }
}

fn read_bounded(path: &Path) -> io::Result<Option<Vec<u8>>> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    if !file.metadata()?.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "counter document must be a regular file",
        ));
    }
    let mut bytes = Vec::new();
    file.take((MAX_COUNTER_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_COUNTER_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "counter document exceeds its bound",
        ));
    }
    Ok(Some(bytes))
}

/// Same-directory atomic replacement, matching `character_store::replace_bytes`. That helper is
/// `pub(crate)` and unreachable from this module's `#[path]`-included test target, so this keeps the
/// smallest equivalent: write a synced temporary file, then rename it over the destination with a
/// bounded Windows retry for a transient sharing violation.
fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let mut temporary = None;
    for _ in 0..64 {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let name = format!(
            ".{}.{}.{}.tmp",
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("instance-counter"),
            std::process::id(),
            sequence
        );
        let candidate = parent.join(name);
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(file) => {
                temporary = Some((candidate, file));
                break;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    let (temporary_path, mut file) = temporary.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::AlreadyExists,
            "could not create a counter temporary file",
        )
    })?;
    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        let mut retries = 0;
        loop {
            match fs::rename(&temporary_path, path) {
                Ok(()) => return Ok(()),
                Err(error)
                    if cfg!(windows)
                        && matches!(error.raw_os_error(), Some(5 | 32))
                        && retries < 20 =>
                {
                    retries += 1;
                    std::thread::sleep(std::time::Duration::from_millis(25));
                }
                Err(error) => return Err(error),
            }
        }
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary_path);
    }
    result
}
