//! The staff audit log: one JSON row per staff write (who, when, what, on whom, before and
//! after), in numbered segments `admin-audit/000001.jsonl`, ... beside the account store.
//!
//! Append-only: a row is never changed or removed and no segment is pruned. Each append rewrites
//! the current segment through the crate's atomic replace (`character_store::replace_bytes`), so
//! a crash leaves the old segment or the new one, never a torn row; a segment past
//! [`SEGMENT_BYTES`] closes and the next number starts. Unlike the security log (`security_log`,
//! ids only, rotated), these rows carry names and staff reasons: the panel reads them back.
use super::Staff;
use crate::character_store::replace_bytes;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

/// A segment closes once the next row would pass this many bytes (R).
pub const SEGMENT_BYTES: usize = 256 * 1024;
/// Rows per page on read (R).
pub const PAGE: usize = 50;
/// A longer row is refused rather than stored (R).
const MAX_ROW_BYTES: usize = 16 * 1024;
/// One writer at a time in this process: the HTTP threads and the world loop both append.
static WRITER: Mutex<()> = Mutex::new(());

/// Whom an action was on. Ids are server-owned; the name is for reading the log later.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Target {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Row {
    /// Unix seconds.
    pub time: u64,
    pub staff: Staff,
    pub action: String,
    #[serde(default)]
    pub target: Target,
    #[serde(default)]
    pub before: Value,
    #[serde(default)]
    pub after: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// The front end: `panel` or `chat`.
    pub via: String,
}

/// The log folder for an account store file: `admin-audit/` beside it.
pub fn dir_for(account_store: &Path) -> PathBuf {
    account_store
        .parent()
        .unwrap_or(Path::new("."))
        .join("admin-audit")
}

fn segment(dir: &Path, number: u32) -> PathBuf {
    dir.join(format!("{number:06}.jsonl"))
}

/// Segment numbers present, ascending; none when the folder does not exist yet.
fn segments(dir: &Path) -> io::Result<Vec<u32>> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut numbers = Vec::new();
    for entry in entries {
        let name = entry?.file_name();
        if let Some(number) = name
            .to_str()
            .and_then(|name| name.strip_suffix(".jsonl"))
            .filter(|stem| stem.len() == 6 && stem.bytes().all(|b| b.is_ascii_digit()))
            .and_then(|stem| stem.parse().ok())
        {
            numbers.push(number);
        }
    }
    numbers.sort_unstable();
    Ok(numbers)
}

/// Append one row. An error means the row is not stored; the caller reports it.
pub fn record(dir: &Path, row: &Row) -> io::Result<()> {
    let mut line = serde_json::to_vec(row)?;
    if line.len() > MAX_ROW_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "audit row too long",
        ));
    }
    line.push(b'\n');
    let _writer = WRITER.lock().unwrap_or_else(PoisonError::into_inner);
    fs::create_dir_all(dir)?;
    let (number, mut bytes) = match segments(dir)?.last() {
        Some(&number) => {
            let bytes = fs::read(segment(dir, number))?;
            if bytes.len() + line.len() > SEGMENT_BYTES {
                let next = number
                    .checked_add(1)
                    .filter(|next| *next <= 999_999)
                    .ok_or_else(|| io::Error::other("audit segments exhausted"))?;
                (next, Vec::new())
            } else {
                (number, bytes)
            }
        }
        None => (1, Vec::new()),
    };
    bytes.extend_from_slice(&line);
    replace_bytes(&segment(dir, number), &bytes)
}

/// Every row, newest first, until `visit` returns false. Unreadable lines are skipped.
pub fn scan(dir: &Path, mut visit: impl FnMut(Row) -> bool) -> io::Result<()> {
    // ponytail: reads whole segments newest first; an index per staff or player if the log
    // grows past what one read per page can afford.
    for number in segments(dir)?.into_iter().rev() {
        let bytes = fs::read(segment(dir, number))?;
        for line in bytes.split(|b| *b == b'\n').rev() {
            if let Ok(row) = serde_json::from_slice::<Row>(line)
                && !visit(row)
            {
                return Ok(());
            }
        }
    }
    Ok(())
}

/// `GET /admin/api/audit` filters. Text matches ignore case; a number matches an id exactly.
#[derive(Clone, Debug, Default)]
pub struct Filter {
    pub staff: Option<String>,
    pub player: Option<String>,
    pub action: Option<String>,
    pub since: Option<u64>,
}

fn names_match(wanted: &str, id: Option<u64>, other_id: Option<u64>, name: Option<&str>) -> bool {
    if let Ok(number) = wanted.parse::<u64>() {
        return id == Some(number) || other_id == Some(number);
    }
    let wanted = wanted.to_lowercase();
    name.is_some_and(|name| name.to_lowercase().contains(&wanted))
}

impl Filter {
    pub fn matches(&self, row: &Row) -> bool {
        let staff = &row.staff;
        let target = &row.target;
        self.since.is_none_or(|since| row.time >= since)
            && self.action.as_deref().is_none_or(|action| row.action == action)
            && self.staff.as_deref().is_none_or(|wanted| {
                names_match(wanted, Some(staff.account_id), None, Some(&staff.name))
            })
            && self.player.as_deref().is_none_or(|wanted| {
                names_match(
                    wanted,
                    target.character_id,
                    target.account_id,
                    target.name.as_deref(),
                )
            })
    }
}

/// One page (1-based) of matching rows, newest first, and how many match in all.
pub fn page(dir: &Path, filter: &Filter, page: usize) -> io::Result<(usize, Vec<Row>)> {
    let skip = page.max(1).saturating_sub(1).saturating_mul(PAGE);
    let (mut total, mut rows) = (0, Vec::new());
    scan(dir, |row| {
        if filter.matches(&row) {
            if total >= skip && rows.len() < PAGE {
                rows.push(row);
            }
            total += 1;
        }
        true
    })?;
    Ok((total, rows))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(time: u64, action: &str, staff: &str, target: u64) -> Row {
        Row {
            time,
            staff: Staff {
                account_id: 1,
                name: staff.into(),
            },
            action: action.into(),
            target: Target {
                character_id: Some(target),
                account_id: Some(target - 100),
                name: Some(format!("Pip {target}")),
            },
            before: Value::Null,
            after: serde_json::json!({"muted": true}),
            reason: Some("spam".into()),
            via: "panel".into(),
        }
    }

    fn scratch(tag: &str) -> PathBuf {
        let root = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let dir = root.join(format!("admin-audit-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn rows_append_across_segments_and_page_newest_first() {
        let dir = scratch("segments");
        for n in 0..120u64 {
            record(&dir, &row(n, if n % 2 == 0 { "mute" } else { "kick" }, "Ada", 101 + n % 3)).unwrap();
        }
        // Force a second segment: the next row would pass the bound.
        let first = segment(&dir, 1);
        let mut big = fs::read(&first).unwrap();
        big.resize(SEGMENT_BYTES - 10, b'\n');
        fs::write(&first, &big).unwrap();
        record(&dir, &row(500, "ban", "Mo", 101)).unwrap();
        assert_eq!(segments(&dir).unwrap(), [1, 2]);

        let all = Filter::default();
        let (total, rows) = page(&dir, &all, 1).unwrap();
        assert_eq!((total, rows.len()), (121, PAGE));
        assert_eq!(rows[0].time, 500, "newest first, across segments");
        assert_eq!(rows[1].time, 119);
        let (_, third) = page(&dir, &all, 3).unwrap();
        assert_eq!(third.len(), 21);
        assert_eq!(third.last().unwrap().time, 0);

        let mutes = Filter {
            action: Some("mute".into()),
            since: Some(100),
            ..Filter::default()
        };
        assert_eq!(page(&dir, &mutes, 1).unwrap().0, 10);
        let by_mo = Filter {
            staff: Some("mo".into()),
            ..Filter::default()
        };
        assert_eq!(page(&dir, &by_mo, 1).unwrap().0, 1);
        let pip = Filter {
            player: Some("102".into()),
            ..Filter::default()
        };
        assert_eq!(page(&dir, &pip, 1).unwrap().0, 40);
        let _ = fs::remove_dir_all(&dir);
    }
}
