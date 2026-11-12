//! Mail, gift and code writes staged while a gateway packet is evaluated. The host commits
//! them with the resulting character snapshot. A synced redo journal survives partial writes;
//! affected files refuse further writes until startup recovery finishes that journal.
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    fs::{self, File},
    io::{self, Read},
    marker::PhantomData,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Mutex,
};

const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_JOURNAL_BYTES: u64 = 32 * 1024 * 1024;
const EXTENSION: &str = "reward-journal";
thread_local! {
    static STAGED: RefCell<Option<Batch>> = const { RefCell::new(None) };
}
// Journal -> affected paths. Only recovery may write these after an interrupted commit.
static PENDING: Mutex<BTreeMap<PathBuf, Vec<PathBuf>>> = Mutex::new(BTreeMap::new());

#[derive(Clone, Debug, Default)]
pub struct Batch {
    reads: BTreeMap<PathBuf, Option<Vec<u8>>>,
    writes: BTreeMap<PathBuf, Vec<u8>>,
    notices: Vec<(PathBuf, u64, bool)>,
}

/// Scope restores the previous staging context on errors and unwinding too. Nested scopes
/// are savepoints: successful work merges; a failed operation discards only its own writes.
pub struct Scope {
    previous: Option<Option<Batch>>,
    _thread: PhantomData<Rc<()>>,
}
impl Scope {
    pub fn begin() -> Self {
        let previous = STAGED.with(|slot| {
            let next = slot.borrow().clone().unwrap_or_default();
            slot.replace(Some(next))
        });
        Self {
            previous: Some(previous),
            _thread: PhantomData,
        }
    }

    pub fn finish(mut self) -> Batch {
        self.previous
            .take()
            .and_then(|previous| STAGED.with(|slot| slot.replace(previous)))
            .unwrap_or_default()
    }

    pub(crate) fn savepoint() -> Self {
        if STAGED.with(|slot| slot.borrow().is_some()) {
            Self::begin()
        } else {
            Self {
                previous: None,
                _thread: PhantomData,
            }
        }
    }

    pub(crate) fn keep(mut self) {
        self.previous = None;
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.take() {
            STAGED.with(|slot| {
                slot.replace(previous);
            });
        }
    }
}

fn disk_read(path: &Path, max: u64) -> io::Result<Option<Vec<u8>>> {
    // Reject directories/devices/FIFOs before opening: opening a FIFO can block the world.
    match fs::metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
        Ok(meta) if !meta.is_file() || meta.len() > max => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "reward store is not a bounded regular file",
            ));
        }
        Ok(_) => {}
    }
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    if !file.metadata()?.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "reward store is not a regular file",
        ));
    }
    let mut bytes = Vec::new();
    file.take(max + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "reward store exceeds its size limit",
        ));
    }
    Ok(Some(bytes))
}

/// Bounded read through the packet's overlay, including writes from earlier bundle children.
pub(crate) fn read(path: &Path, max: u64) -> io::Result<Option<Vec<u8>>> {
    STAGED.with(|slot| {
        let mut staged = slot.borrow_mut();
        let Some(batch) = staged.as_mut() else {
            return disk_read(path, max);
        };
        if let Some(bytes) = batch.writes.get(path) {
            return if bytes.len() as u64 <= max {
                Ok(Some(bytes.clone()))
            } else {
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "staged reward store exceeds its size limit",
                ))
            };
        }
        let bytes = disk_read(path, max)?;
        batch
            .reads
            .entry(path.to_path_buf())
            .or_insert_with(|| bytes.clone());
        Ok(bytes)
    })
}

pub(crate) fn write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    STAGED.with(|slot| {
        let mut staged = slot.borrow_mut();
        if let Some(batch) = staged.as_mut() {
            if !batch.reads.contains_key(path) {
                batch
                    .reads
                    .insert(path.to_path_buf(), disk_read(path, MAX_FILE_BYTES)?);
            }
            batch.writes.insert(path.to_path_buf(), bytes.to_vec());
            Ok(())
        } else {
            crate::character_store::replace_bytes(path, bytes)
        }
    })
}

pub(crate) fn notice(store: &Path, account: u64, resend: bool) -> bool {
    STAGED.with(|slot| {
        let mut staged = slot.borrow_mut();
        let Some(batch) = staged.as_mut() else {
            return false;
        };
        batch.notices.push((store.to_path_buf(), account, resend));
        true
    })
}

fn absolute(path: &Path) -> io::Result<PathBuf> {
    std::path::absolute(path)
}

pub(crate) fn ensure_writable(path: &Path) -> io::Result<()> {
    let path = absolute(path)?;
    let pending = PENDING.lock().unwrap_or_else(|e| e.into_inner());
    if pending.values().any(|paths| paths.contains(&path)) {
        return Err(io::Error::other(
            "reward transaction requires gateway restart and recovery",
        ));
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Journal {
    version: u32,
    writes: Vec<(PathBuf, String)>,
}

impl Batch {
    pub fn is_empty(&self) -> bool {
        self.writes.is_empty()
    }

    /// The gateway is the single character writer. Reacquire the mail/gift locks and the
    /// code-file lock before comparing the stores read during staging with their live bytes.
    pub fn commit(
        mut self,
        character: &Path,
        snapshot: &crate::game_store::Snapshot,
    ) -> io::Result<()> {
        if self.is_empty() {
            self.publish_notices();
            return Ok(());
        }
        let mut code_locks = Vec::new();
        for path in self
            .writes
            .keys()
            .filter(|p| p.file_name().is_some_and(|n| n == "redeem-codes.json"))
        {
            code_locks.push(crate::account_store::AccountStoreLock::acquire_within(
                path,
                crate::account_store::STAFF_LOCK_WAIT,
            )?);
        }
        let _mail = crate::mail::LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let _gifts = crate::gifts::guard();
        let bytes = crate::game_store::snapshot_bytes(character, snapshot)?;
        self.reads.insert(
            character.to_path_buf(),
            disk_read(character, MAX_FILE_BYTES)?,
        );
        self.writes.insert(character.to_path_buf(), bytes);
        self.commit_with(character, |path, bytes| {
            crate::character_store::replace_bytes_unchecked(path, bytes)
        })?;
        self.publish_notices();
        Ok(())
    }

    fn publish_notices(self) {
        for (store, account, resend) in self.notices {
            crate::gifts::publish_notice(&store, account, resend);
        }
    }

    fn commit_with(
        &self,
        character: &Path,
        mut replace: impl FnMut(&Path, &[u8]) -> io::Result<()>,
    ) -> io::Result<()> {
        for (path, before) in &self.reads {
            crate::reward_transaction::ensure_writable(path)?;
            if disk_read(path, MAX_FILE_BYTES)? != *before {
                return Err(io::Error::other(
                    "reward store changed while staging; retry the request",
                ));
            }
        }
        let journal_path = absolute(&character.with_extension(EXTENSION))?;
        if journal_path.exists() {
            return Err(io::Error::other(
                "reward journal already exists; restart for recovery",
            ));
        }
        let writes = self
            .writes
            .iter()
            .map(|(path, bytes)| {
                Ok((
                    absolute(path)?,
                    String::from_utf8(bytes.clone()).map_err(io::Error::other)?,
                ))
            })
            .collect::<io::Result<Vec<_>>>()?;
        let journal = Journal { version: 1, writes };
        let bytes = serde_json::to_vec(&journal).map_err(io::Error::other)?;
        if bytes.len() as u64 > MAX_JOURNAL_BYTES {
            return Err(io::Error::other("reward journal exceeds its size limit"));
        }
        if let Some(parent) = journal_path.parent() {
            fs::create_dir_all(parent)?;
        }
        crate::character_store::replace_bytes(&journal_path, &bytes)?;
        block(&journal_path, &journal);
        sync_parent(&journal_path)?;
        apply(&journal_path, &journal, &mut replace)
    }
}

fn block(path: &Path, journal: &Journal) {
    PENDING.lock().unwrap_or_else(|e| e.into_inner()).insert(
        path.to_path_buf(),
        journal.writes.iter().map(|(p, _)| p.clone()).collect(),
    );
}

fn apply(
    path: &Path,
    journal: &Journal,
    replace: &mut impl FnMut(&Path, &[u8]) -> io::Result<()>,
) -> io::Result<()> {
    for (target, bytes) in &journal.writes {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        replace(target, bytes.as_bytes())?;
        sync_parent(target)?;
    }
    fs::remove_file(path)?;
    sync_parent(path)?;
    PENDING
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .remove(path);
    Ok(())
}

fn sync_parent(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    if let Some(parent) = path.parent() {
        File::open(parent)?.sync_all()?;
    }
    Ok(())
}

/// Run before loading any character, starting HTTP writers, or accepting clients. Recovery
/// is idempotent: an already-written destination is replaced with the same complete bytes.
pub fn recover(directory: &Path) -> io::Result<usize> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(error),
    };
    let mut count = 0;
    for entry in entries {
        let path = absolute(&entry?.path())?;
        if path.extension().is_none_or(|ext| ext != EXTENSION) {
            continue;
        }
        let bytes = disk_read(&path, MAX_JOURNAL_BYTES)?
            .ok_or_else(|| io::Error::other("missing reward journal"))?;
        let journal: Journal = serde_json::from_slice(&bytes).map_err(io::Error::other)?;
        if journal.version != 1
            || journal.writes.is_empty()
            || journal.writes.iter().any(|(p, _)| !p.is_absolute())
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unsupported reward journal",
            ));
        }
        block(&path, &journal);
        apply(
            &path,
            &journal,
            &mut crate::character_store::replace_bytes_unchecked,
        )?;
        count += 1;
    }
    Ok(count)
}

#[cfg(test)]
mod tests;
