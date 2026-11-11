//! The mail facade every front end calls: G1's `/admin/api/mail*` routes and the `realm mail`
//! chat command (`chat.rs`, started by the gateway host). Plain types, serde-ready.
//!
//! A send resolves its recipients at once and then runs as a background job on its own thread,
//! paced at `send_rate_per_second`, so even "everyone" never blocks the 50 ms world tick: the
//! tick only drains the job's delivered list (`take_delivered`) to tell online recipients.
//! Every send writes one row to the staff audit log (`admin::audit`, the panel's Audit view):
//! who sent what to whom, from which front end.

use super::{Attachments, LIMITS, Letter, MailError, mailbox_path, refused, unix_now};
use crate::account_store::{AccountRole, AccountStore};
use crate::admin::audit;
/// The staff member a send is audited under: the admin core's own.
pub use crate::admin::Staff;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
    time::Duration,
};

/// Who a send is for. `Everyone` and `Role` cover offline characters too (every character in
/// the account store); `Online` and `Zone` are the sessions the caller lists.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum To {
    Characters(Vec<u64>),
    Everyone,
    Online,
    Role(AccountRole),
    Zone(u32),
}

/// One live session as the caller's roster sees it.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct OnlineCharacter {
    pub guid: u64,
    pub zone: u32,
    pub role: AccountRole,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Receipt {
    pub job_id: u64,
    pub recipients: usize,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Queued,
    Running,
    Done,
}

/// A send's progress: `done` delivered, `skipped` refused (a full mailbox) or failed.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct JobView {
    pub job_id: u64,
    pub to_summary: String,
    pub total: usize,
    pub done: usize,
    pub skipped: usize,
    pub state: JobState,
}

/// One message in a player's mailbox, for `GET /admin/api/players/{id}/mail`.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MailView {
    pub mail_id: u32,
    pub from: String,
    pub subject: String,
    pub body: String,
    pub sent: u64,
    pub read: bool,
    pub attachments: Attachments,
    pub claimed: bool,
}

struct Job {
    id: u64,
    realm: PathBuf,
    to_summary: String,
    total: usize,
    started: AtomicBool,
    finished: AtomicBool,
    done: AtomicUsize,
    skipped: AtomicUsize,
    /// `(character, unread after delivery)` not yet told by the host.
    delivered: Mutex<Vec<(u64, usize)>>,
}

impl Job {
    fn view(&self) -> JobView {
        let state = if self.finished.load(Ordering::Acquire) {
            JobState::Done
        } else if self.started.load(Ordering::Acquire) {
            JobState::Running
        } else {
            JobState::Queued
        };
        JobView {
            job_id: self.id,
            to_summary: self.to_summary.clone(),
            total: self.total,
            done: self.done.load(Ordering::Acquire),
            skipped: self.skipped.load(Ordering::Acquire),
            state,
        }
    }
}

/// (R) Finished jobs kept for `jobs()`; older finished ones are dropped first.
const KEPT_JOBS: usize = 64;
static NEXT_JOB: AtomicU64 = AtomicU64::new(1);
static JOBS: Mutex<Vec<Arc<Job>>> = Mutex::new(Vec::new());

fn jobs_lock() -> std::sync::MutexGuard<'static, Vec<Arc<Job>>> {
    JOBS.lock().unwrap_or_else(|error| error.into_inner())
}

/// Queue `letter` for `to`. `item_ok` validates each attachment row against the item tables
/// (the caller's: the sender's own inventory catalogue for chat, G3's item search for HTTP);
/// a refusal queues nothing. `via` is the front end (`chat` or `panel`) for the audit row,
/// which is best effort, like every security log row: a failure is printed and never undoes
/// the send.
pub fn send(
    account_store: &Path,
    staff: &Staff,
    via: &str,
    to: &To,
    letter: Letter,
    online: &[OnlineCharacter],
    item_ok: &dyn Fn(i32, u32) -> bool,
) -> Result<Receipt, MailError> {
    let limits = &*LIMITS;
    letter.validate(limits)?;
    if !letter
        .attachments
        .items
        .iter()
        .all(|&(id, count)| item_ok(id, count))
    {
        return Err(refused("an attached item id is not in the item tables"));
    }
    let store = AccountStore::load(account_store)?;
    let recipients = recipients(&store, to, online)?;
    let total = recipients.len();
    let summary = summary(&store, to, total);
    let job = Arc::new(Job {
        id: NEXT_JOB.fetch_add(1, Ordering::Relaxed),
        realm: account_store.to_owned(),
        to_summary: summary,
        total,
        started: AtomicBool::new(false),
        finished: AtomicBool::new(false),
        done: AtomicUsize::new(0),
        skipped: AtomicUsize::new(0),
        delivered: Mutex::new(Vec::new()),
    });
    let worker = Arc::clone(&job);
    let worker_letter = letter.clone();
    std::thread::Builder::new()
        .name(format!("mail-job-{}", job.id))
        .spawn(move || run(&worker, &recipients, &worker_letter))
        .map_err(MailError::Io)?;
    {
        let mut jobs = jobs_lock();
        while jobs.len() >= KEPT_JOBS {
            let Some(oldest) = jobs.iter().position(|job| job.finished.load(Ordering::Acquire))
            else {
                break;
            };
            jobs.remove(oldest);
        }
        jobs.push(Arc::clone(&job));
    }
    let target = match to {
        To::Characters(guids) if guids.len() == 1 => (store.character(guids[0])).map(|record| {
            audit::Target {
                character_id: Some(record.guid),
                account_id: Some(record.owner_account_id),
                name: Some(record.identity.display_name()),
            }
        }),
        _ => None,
    };
    audit::record_beside(account_store, &audit::Row {
        time: unix_now(),
        staff: staff.clone(),
        action: "mail".into(),
        target: target.unwrap_or_default(),
        before: serde_json::Value::Null,
        after: serde_json::json!({
            "to": job.to_summary, "recipients": total, "job": job.id, "from": letter.from,
            "subject": letter.subject, "attachments": letter.attachments,
        }),
        reason: None,
        via: via.to_owned(),
    });
    Ok(Receipt {
        job_id: job.id,
        recipients: total,
    })
}

fn recipients(
    store: &AccountStore,
    to: &To,
    online: &[OnlineCharacter],
) -> Result<Vec<u64>, MailError> {
    let role_of = |guid: u64| {
        store
            .character(guid)
            .and_then(|record| store.account_role(record.owner_account_id))
            .unwrap_or(AccountRole::Player)
    };
    let guids: BTreeSet<u64> = match to {
        To::Characters(guids) => {
            if let Some(missing) = guids.iter().find(|guid| store.character(**guid).is_none()) {
                return Err(refused(format!("there is no character {missing}")));
            }
            guids.iter().copied().collect()
        }
        To::Everyone => store.characters().map(|record| record.guid).collect(),
        To::Role(role) => (store.characters())
            .map(|record| record.guid)
            .filter(|guid| role_of(*guid) == *role)
            .collect(),
        To::Online => online.iter().map(|character| character.guid).collect(),
        To::Zone(zone) => (online.iter())
            .filter(|character| character.zone == *zone)
            .map(|character| character.guid)
            .collect(),
    };
    // A roster row the store does not know (a solo or stale session) gets no mailbox.
    let guids: Vec<u64> = guids
        .into_iter()
        .filter(|guid| store.character(*guid).is_some())
        .collect();
    if guids.is_empty() {
        return Err(refused("nobody matches those recipients"));
    }
    Ok(guids)
}

fn summary(store: &AccountStore, to: &To, total: usize) -> String {
    match to {
        To::Characters(guids) if guids.len() == 1 => store
            .character(guids[0])
            .map_or_else(|| guids[0].to_string(), |r| r.identity.display_name()),
        To::Characters(_) => format!("{total} characters"),
        To::Everyone => "everyone".into(),
        To::Online => "everyone online".into(),
        To::Role(role) => format!("role {}", format!("{role:?}").to_lowercase()),
        To::Zone(zone) => format!("zone {zone}"),
    }
}

/// The job thread: one mailbox write per recipient, paced, never touching a character save.
fn run(job: &Job, recipients: &[u64], letter: &Letter) {
    job.started.store(true, Ordering::Release);
    let rate = LIMITS.send_rate_per_second.max(1);
    let pause = Duration::from_secs(1) / rate;
    for (index, guid) in recipients.iter().enumerate() {
        if index > 0 {
            std::thread::sleep(pause);
        }
        match super::deliver(&mailbox_path(&job.realm, *guid), letter, &LIMITS) {
            Ok(unread) => {
                job.delivered
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .push((*guid, unread));
                job.done.fetch_add(1, Ordering::AcqRel);
            }
            Err(error) => {
                eprintln!("mail job {}: character {guid}: {error}", job.id);
                job.skipped.fetch_add(1, Ordering::AcqRel);
            }
        }
    }
    job.finished.store(true, Ordering::Release);
}

/// Every known job, newest first.
pub fn jobs() -> Vec<JobView> {
    jobs_lock().iter().rev().map(|job| job.view()).collect()
}

/// One job's progress.
pub fn job(job_id: u64) -> Option<JobView> {
    jobs_lock()
        .iter()
        .find(|job| job.id == job_id)
        .map(|job| job.view())
}

/// The host's half of a delivery: `(character, unread)` for each letter this realm's jobs put
/// down since the last call. Jobs of another account store (another server in the same
/// process, as in tests) are left for their own host.
pub fn take_delivered(account_store: &Path) -> Vec<(u64, usize)> {
    let jobs: Vec<Arc<Job>> = jobs_lock()
        .iter()
        .filter(|job| job.realm == account_store)
        .cloned()
        .collect();
    jobs.iter()
        .flat_map(|job| {
            std::mem::take(&mut *job.delivered.lock().unwrap_or_else(|e| e.into_inner()))
        })
        .collect()
}

/// A character's mail, newest first.
pub fn list_for(account_store: &Path, character: u64) -> Result<Vec<MailView>, MailError> {
    let mailbox = super::load(&mailbox_path(account_store, character))?;
    Ok(mailbox
        .messages
        .into_iter()
        .rev()
        .map(|message| MailView {
            mail_id: message.id,
            from: message.from,
            subject: message.subject,
            body: message.body,
            sent: message.sent,
            read: message.read,
            attachments: message.attachments,
            claimed: message.claimed,
        })
        .collect())
}
