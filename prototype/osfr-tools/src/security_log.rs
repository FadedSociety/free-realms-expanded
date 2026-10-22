//! Bounded security audit (Security Model L9). All limits and aggregation are (R).
//!
//! Host-owned path, one writer per file. No IPs (including hashes), player names,
//! payloads or secrets are accepted. Numeric account/session ids are server-owned.
//! Keep this outside staged gameplay. The gateway calls `SecurityLog::record` on
//! refusal and handles I/O errors locally; a log error must not disconnect a peer.
//! Call `flush` on a host timer/shutdown to drain buffered counts. Logs inherit
//! the host directory's ACL; deployment must restrict it to service/admin users.

use crate::moderation::Applied;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// (R) Each of the current file and its `.previous` sibling is capped at 1 MiB.
pub const MAX_LOG_BYTES: u64 = 1024 * 1024;
const WRITE_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    AdmissionRefusal,
    AuthFailure,
    BudgetRefusal,
    DevCommandDenial,
    /// L042: one unbatched row per applied staff action (`record_moderation`).
    ModerationAction,
}

impl EventKind {
    /// Batched refusal kinds: (slot, action, reason). Moderation rows are never batched.
    fn refusal(self) -> Option<(usize, &'static str, &'static str)> {
        match self {
            Self::AdmissionRefusal => Some((0, "admission", "admission_policy")),
            Self::AuthFailure => Some((1, "authentication", "invalid_credentials")),
            Self::BudgetRefusal => Some((2, "social_action", "action_budget")),
            Self::DevCommandDenial => Some((3, "dev_command", "authorization")),
            Self::ModerationAction => None,
        }
    }
}

/// Closed schema: arbitrary input fields are discarded, never forwarded to JSONL.
/// `None` is appropriate before authentication. Never populate ids from a packet.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct SecurityEvent {
    pub kind: EventKind,
    pub account_id: Option<u64>,
    pub session_id: Option<u64>,
}

#[derive(Default)]
struct Pending {
    event: Option<SecurityEvent>,
    count: u64,
    last_write: Option<Instant>,
}

pub struct SecurityLog {
    path: PathBuf,
    previous: PathBuf,
    max_bytes: u64,
    pending: [Pending; 4],
}

impl SecurityLog {
    /// Append to an existing file; never truncate it at startup. Refuse oversized
    /// preexisting files rather than silently discard or archive oversized data.
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_owned();
        let mut previous = path.as_os_str().to_owned();
        previous.push(".previous");
        let previous = PathBuf::from(previous);
        for file in [&path, &previous] {
            match fs::metadata(file) {
                Ok(meta) if !meta.is_file() || meta.len() > MAX_LOG_BYTES => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "invalid security log file",
                    ));
                }
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
        // Parent directory must already be host-provisioned with appropriate ACLs.
        OpenOptions::new().create(true).append(true).open(&path)?;
        Ok(Self {
            path,
            previous,
            max_bytes: MAX_LOG_BYTES,
            pending: std::array::from_fn(|_| Pending::default()),
        })
    }

    /// At most one immediate row per kind per second (four kinds, no actor map).
    /// Suppressed events are counted with saturating arithmetic. If a batch mixes
    /// actors, differing ids become null rather than attributing it to one actor.
    pub fn record(&mut self, event: SecurityEvent) -> io::Result<()> {
        self.record_at(event, Instant::now())
    }

    fn record_at(&mut self, event: SecurityEvent, now: Instant) -> io::Result<()> {
        let Some((index, ..)) = event.kind.refusal() else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "moderation rows use record_moderation",
            ));
        };
        let slot = &mut self.pending[index];
        if slot.count == 0 {
            slot.event = Some(event);
        } else if let Some(pending) = &mut slot.event {
            if pending.account_id != event.account_id {
                pending.account_id = None;
            }
            if pending.session_id != event.session_id {
                pending.session_id = None;
            }
        }
        slot.count = slot.count.saturating_add(1);
        if slot
            .last_write
            .is_none_or(|last| now.saturating_duration_since(last) >= WRITE_INTERVAL)
        {
            // Throttle failed I/O attempts too: an unwritable log must not cause
            // an open/rotation attempt (or host error notice) for every packet.
            slot.last_write = Some(now);
            self.write_pending(index, now)?;
        }
        Ok(())
    }

    /// Drain buffered counts on a trusted host timer (at most once per second)
    /// or clean shutdown. Abrupt process death can lose buffered counts (R).
    pub fn flush(&mut self) -> io::Result<()> {
        let now = Instant::now();
        for index in 0..self.pending.len() {
            self.write_pending(index, now)?;
        }
        Ok(())
    }

    fn write_pending(&mut self, index: usize, now: Instant) -> io::Result<()> {
        let slot = &self.pending[index];
        let Some(event) = slot.event.filter(|_| slot.count > 0) else {
            return Ok(());
        };
        let Some((_, action, reason)) = event.kind.refusal() else {
            return Ok(());
        };
        let count = slot.count;
        self.append(serde_json::json!({
            "timestamp_ms": timestamp_ms(), "kind": event.kind,
            "account_id": event.account_id, "session_id": event.session_id,
            "action": action, "outcome": "refused", "reason": reason, "count": count,
        }))?;
        let slot = &mut self.pending[index];
        slot.count = 0;
        slot.event = None;
        slot.last_write = Some(now);
        Ok(())
    }

    /// L043: one row per admin character move, written at once (never batched or merged like
    /// refusals). Numbers only: the row has no field for a name or chat text.
    pub fn record_admin_move(&mut self, row: AdminMoveRow) -> io::Result<()> {
        let pose = |(zone_id, position): (u32, [f32; 3])| serde_json::json!({"zone_id": zone_id, "position": position});
        self.append(serde_json::json!({
            "timestamp_ms": timestamp_ms(), "kind": "admin_move",
            "account_id": row.account_id, "session_id": row.session_id,
            "action": "move_character", "outcome": "applied", "target_guid": row.target_guid,
            "online": row.online, "from": pose(row.from), "to": pose(row.to),
        }))
    }

    /// L042: one row per applied moderation action, written at once and never batched, so
    /// every action keeps its own actor. Server-owned account ids, the closed action name and
    /// the minutes only; never names, chat or the command text.
    pub fn record_moderation(&mut self, actor_account_id: u64, applied: Applied) -> io::Result<()> {
        self.append(serde_json::json!({
            "timestamp_ms": timestamp_ms(), "kind": EventKind::ModerationAction,
            "account_id": actor_account_id, "target_account_id": applied.target_account_id,
            "action": applied.action, "minutes": applied.minutes, "outcome": "applied",
        }))
    }

    /// The one write path for every row (refusal batches, admin moves, moderation): append one
    /// JSON row, rotating to `.previous` first when the row would pass the cap.
    fn append(&mut self, row: serde_json::Value) -> io::Result<()> {
        let mut line = serde_json::to_vec(&row)?;
        line.push(b'\n');
        if line.len() as u64 > self.max_bytes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "security log row exceeds cap",
            ));
        }
        // ponytail: open on each bounded write; no background thread or logging crate.
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        if file.metadata()?.len().saturating_add(line.len() as u64) > self.max_bytes {
            drop(file); // Windows rotation must not hold the old file open.
            match fs::remove_file(&self.previous) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
            fs::rename(&self.path, &self.previous)?;
            file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)?;
        }
        file.write_all(&line)?;
        file.flush()
    }
}

/// L043: an applied admin move. Server-owned numbers: the acting account and session, the
/// moved character's GUID, whether it was online, and its zone and position before and after.
#[derive(Clone, Copy, Debug)]
pub struct AdminMoveRow {
    pub account_id: u64,
    pub session_id: u64,
    pub target_guid: u64,
    pub online: bool,
    pub from: (u32, [f32; 3]),
    pub to: (u32, [f32; 3]),
}

fn timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Duration;

    static NEXT_FILE: AtomicU64 = AtomicU64::new(0);

    struct Scratch(PathBuf);

    impl Scratch {
        fn new() -> Self {
            // CARGO_TARGET_DIR is W91-owned; no game/save/log files are used.
            let root = std::env::var_os("CARGO_TARGET_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(std::env::temp_dir);
            let dir = root.join(format!(
                "security-log-test-{}-{}",
                std::process::id(),
                NEXT_FILE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
        fn path(&self) -> PathBuf {
            self.0.join("security.jsonl")
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    fn event(kind: EventKind) -> SecurityEvent {
        SecurityEvent {
            kind,
            account_id: Some(7),
            session_id: Some(11),
        }
    }

    fn rows(path: &Path) -> Vec<serde_json::Value> {
        fs::read_to_string(path)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    #[test]
    fn burst_is_counted_without_per_packet_writes_and_mixed_actors_are_not_misattributed() {
        let scratch = Scratch::new();
        let mut log = SecurityLog::open(scratch.path()).unwrap();
        let now = Instant::now();
        for _ in 0..100 {
            log.record_at(event(EventKind::BudgetRefusal), now).unwrap();
        }
        assert_eq!(rows(&scratch.path()).len(), 1);
        let mut other = event(EventKind::BudgetRefusal);
        other.account_id = Some(8);
        other.session_id = Some(12);
        log.record_at(other, now + Duration::from_secs(1)).unwrap();
        let rows = rows(&scratch.path());
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["count"], 1);
        assert_eq!(rows[1]["count"], 100);
        assert!(rows[1]["account_id"].is_null());
        assert!(rows[1]["session_id"].is_null());
    }

    #[test]
    fn append_reopen_and_rotation_keep_only_two_capped_jsonl_files() {
        let scratch = Scratch::new();
        let now = Instant::now();
        let mut log = SecurityLog::open(scratch.path()).unwrap();
        log.max_bytes = 600;
        for n in 0..16 {
            log.record_at(
                event(EventKind::AdmissionRefusal),
                now + Duration::from_secs(n),
            )
            .unwrap();
            assert!(fs::metadata(scratch.path()).unwrap().len() <= 600);
            if log.previous.exists() {
                assert!(fs::metadata(&log.previous).unwrap().len() <= 600);
                assert!(!rows(&log.previous).is_empty());
            }
        }
        assert_eq!(fs::read_dir(&scratch.0).unwrap().count(), 2);
        let before = fs::read(scratch.path()).unwrap();
        drop(log);
        let mut reopened = SecurityLog::open(scratch.path()).unwrap();
        reopened
            .record_at(event(EventKind::AuthFailure), now)
            .unwrap();
        assert!(fs::read(scratch.path()).unwrap().starts_with(&before));
    }

    #[test]
    fn untrusted_extra_name_chat_token_and_ip_fields_never_reach_disk() {
        let scratch = Scratch::new();
        let input = serde_json::json!({
            "kind": "dev_command_denial", "account_id": 7, "session_id": 11,
            "name": "REDACTION_NAME_SENTINEL", "chat": "REDACTION_CHAT_SENTINEL",
            "token": "REDACTION_TOKEN_SENTINEL", "ticket": "REDACTION_TICKET_SENTINEL",
            "ip": "192.0.2.77", "reason": "REDACTION_REASON_SENTINEL"
        });
        let safe: SecurityEvent = serde_json::from_value(input).unwrap();
        let mut log = SecurityLog::open(scratch.path()).unwrap();
        log.record(safe).unwrap();
        let bytes = fs::read_to_string(scratch.path()).unwrap();
        assert!(!bytes.contains("SENTINEL"));
        assert!(!bytes.contains("192.0.2.77"));
        let rows = rows(&scratch.path());
        assert_eq!(rows[0]["kind"], "dev_command_denial");
        assert_eq!(rows[0]["outcome"], "refused");
        assert_eq!(rows[0]["account_id"], 7);
        assert!(rows[0]["timestamp_ms"].is_u64());
    }

    #[test]
    fn all_four_kinds_have_separate_counts_and_pending_counts_flush() {
        let scratch = Scratch::new();
        let mut log = SecurityLog::open(scratch.path()).unwrap();
        let now = Instant::now();
        for kind in [
            EventKind::AdmissionRefusal,
            EventKind::AuthFailure,
            EventKind::BudgetRefusal,
            EventKind::DevCommandDenial,
        ] {
            log.record_at(event(kind), now).unwrap();
            log.record_at(event(kind), now).unwrap();
        }
        assert_eq!(rows(&scratch.path()).len(), 4);
        log.flush().unwrap();
        let rows = rows(&scratch.path());
        assert_eq!(rows.len(), 8);
        assert_eq!(
            rows.iter()
                .map(|row| row["count"].as_u64().unwrap())
                .sum::<u64>(),
            8
        );
    }

    #[test]
    fn failed_io_is_throttled_and_pending_counts_survive_until_retry() {
        let scratch = Scratch::new();
        let mut log = SecurityLog::open(scratch.path()).unwrap();
        fs::remove_file(scratch.path()).unwrap();
        fs::create_dir(scratch.path()).unwrap(); // Force an ordinary I/O failure.
        let now = Instant::now();
        assert!(log.record_at(event(EventKind::AuthFailure), now).is_err());
        fs::remove_dir(scratch.path()).unwrap();
        log.record_at(event(EventKind::AuthFailure), now).unwrap();
        assert!(!scratch.path().exists()); // No per-packet I/O retry.
        log.record_at(event(EventKind::AuthFailure), now + Duration::from_secs(1))
            .unwrap();
        assert_eq!(rows(&scratch.path())[0]["count"], 3);
    }

    /// L043: every admin move is its own row at once, even two in the same instant, and the
    /// row carries numbers only.
    #[test]
    fn admin_moves_are_written_at_once_with_numbers_only() {
        let scratch = Scratch::new();
        let mut log = SecurityLog::open(scratch.path()).unwrap();
        let row = AdminMoveRow {
            account_id: 7,
            session_id: 11,
            target_guid: 42,
            online: false,
            from: (0, [1.0, 2.0, 3.0]),
            to: (0, [-1904.5, -39.25, 413.5]),
        };
        log.record_admin_move(row).unwrap();
        log.record_admin_move(AdminMoveRow {
            online: true,
            ..row
        })
        .unwrap();
        let rows = rows(&scratch.path());
        assert_eq!(rows.len(), 2, "admin moves are never batched");
        assert_eq!(rows[0]["kind"], "admin_move");
        assert_eq!(rows[0]["outcome"], "applied");
        assert_eq!(rows[0]["account_id"], 7);
        assert_eq!(rows[0]["target_guid"], 42);
        assert_eq!(
            rows[0]["to"]["position"],
            serde_json::json!([-1904.5, -39.25, 413.5])
        );
        assert_eq!(rows[1]["online"], true);
        let keys: Vec<_> = rows[0].as_object().unwrap().keys().cloned().collect();
        assert_eq!(
            keys,
            [
                "account_id",
                "action",
                "from",
                "kind",
                "online",
                "outcome",
                "session_id",
                "target_guid",
                "timestamp_ms",
                "to"
            ]
        );
    }

    /// L042R2: refusal batches, admin moves and moderation rows share the one capped, rotating
    /// `append`, and a moderation kind is never taken as a batched refusal.
    #[test]
    fn every_row_kind_shares_the_capped_rotation() {
        let scratch = Scratch::new();
        let mut log = SecurityLog::open(scratch.path()).unwrap();
        log.max_bytes = 600;
        assert!(log.record(event(EventKind::ModerationAction)).is_err());
        assert!(fs::read(scratch.path()).unwrap().is_empty());
        let applied = Applied {
            action: crate::moderation::Action::Ban,
            target_account_id: 9,
            minutes: Some(60),
        };
        let moved = AdminMoveRow {
            account_id: 7,
            session_id: 11,
            target_guid: 42,
            online: true,
            from: (0, [1.0, 2.0, 3.0]),
            to: (0, [4.0, 5.0, 6.0]),
        };
        let now = Instant::now();
        for n in 0..12 {
            match n % 3 {
                0 => log.record_moderation(7, applied).unwrap(),
                1 => log.record_admin_move(moved).unwrap(),
                _ => log
                    .record_at(event(EventKind::AuthFailure), now + Duration::from_secs(n))
                    .unwrap(),
            }
            assert!(fs::metadata(scratch.path()).unwrap().len() <= 600);
            assert!(!log.previous.exists() || fs::metadata(&log.previous).unwrap().len() <= 600);
        }
        assert_eq!(fs::read_dir(&scratch.0).unwrap().count(), 2);
        let mut all = rows(&log.previous);
        all.extend(rows(&scratch.path()));
        for kind in ["moderation_action", "admin_move", "auth_failure"] {
            assert!(all.iter().any(|row| row["kind"] == kind), "{kind}");
        }
        let row = all
            .iter()
            .find(|row| row["kind"] == "moderation_action")
            .unwrap();
        let keys: Vec<_> = row.as_object().unwrap().keys().cloned().collect();
        assert_eq!(
            keys,
            [
                "account_id",
                "action",
                "kind",
                "minutes",
                "outcome",
                "target_account_id",
                "timestamp_ms"
            ]
        );
        assert_eq!(row["account_id"], 7);
        assert_eq!(row["target_account_id"], 9);
        assert_eq!(row["action"], "ban");
    }

    #[test]
    fn oversized_existing_log_is_rejected_without_truncation() {
        let scratch = Scratch::new();
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(scratch.path())
            .unwrap();
        file.set_len(MAX_LOG_BYTES + 1).unwrap();
        drop(file);
        assert!(SecurityLog::open(scratch.path()).is_err());
        assert_eq!(
            fs::metadata(scratch.path()).unwrap().len(),
            MAX_LOG_BYTES + 1
        );
    }
}
