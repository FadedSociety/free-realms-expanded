//! Staff and system mail (gm-panel G2): one mailbox per character, sends to one, several or
//! everyone, the "You have N new messages" notice, and claim, read and delete.
//!
//! The original client has no mail code (no Mail class or string in the Hex-Rays export), so
//! every rule here is reconstructed (R) and every bound is data: `data/content/mail.json`,
//! compiled in like `moderation.json`. What the client does have is reused:
//! - the notice is its own locale row 3023177284 ("You have %1 new messages", text id 436387),
//!   sent as a 15/3 system line (`chat::system`) because no client packet binds `%1`;
//! - reading uses the 26/3 dialog window quests and NPCs use (`chat.rs`), with stock button
//!   texts, while the message itself goes to chat: the dialog shows locale ids only.
//!
//! Storage: `<account store dir>/mail/<character guid>.json`, beside the realm's character
//! registry, never inside the character save. Every write is a read-modify-write under one
//! process lock through the atomic `character_store::replace_bytes`, so a background job
//! writing an offline character's mailbox and that player's own claim or a login cannot lose
//! each other's update, and the character save itself is never touched by a send.

pub mod admin;
pub mod chat;

use serde::{Deserialize, Serialize};
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

/// Client text id of locale row 3023177284, `You have <font ...>%1</font> new messages`.
pub const NEW_MESSAGES_TEXT_ID: i32 = 436_387;
/// The same row's words, for a gateway started without `--client-locale`.
const NEW_MESSAGES_FALLBACK: &str = "You have %1 new messages";
const SCHEMA_VERSION: u32 = 1;
/// (R) A mailbox at its message cap is about 120 KB; this bounds a corrupt or foreign file.
const MAX_FILE_BYTES: u64 = 1 << 20;
/// Message ids fit the 20 bits a dialog row carries (`chat::response`); a mailbox would need a
/// million deliveries to run out, and then refuses instead of wrapping.
pub const MAX_MESSAGE_ID: u32 = 0xF_FFFF;

/// The tunable bounds and the town postboxes, from `data/content/mail.json`.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub schema_version: u32,
    #[serde(default)]
    pub notes: String,
    pub max_messages: usize,
    pub max_subject_chars: usize,
    pub max_body_chars: usize,
    pub max_attachment_items: usize,
    pub max_attachment_coins: u32,
    /// Deliveries per second a mass send writes (one mailbox file each).
    pub send_rate_per_second: u32,
    /// Metres from a postbox a click is answered at.
    pub click_range: f32,
    pub mailboxes: Vec<Postbox>,
}

/// One town postbox group: its world actor (an `npc_spawns_extra.json` spawn guid) and the
/// town's Mail Sorting game.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Postbox {
    pub guid: u64,
    pub town: String,
    pub game: i32,
    pub position: [f32; 3],
}

impl Limits {
    /// What a shipped file that does not parse leaves: every send refused, no postbox.
    fn closed() -> Self {
        Self {
            schema_version: 0,
            notes: String::new(),
            max_messages: 0,
            max_subject_chars: 0,
            max_body_chars: 0,
            max_attachment_items: 0,
            max_attachment_coins: 0,
            send_rate_per_second: 1,
            click_range: 0.0,
            mailboxes: Vec::new(),
        }
    }

    pub fn postbox(&self, guid: u64) -> Option<&Postbox> {
        self.mailboxes.iter().find(|postbox| postbox.guid == guid)
    }
}

/// The shipped bounds; `shipped_limits_are_usable` keeps the file parsing.
pub static LIMITS: LazyLock<Limits> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../data/content/mail.json"))
        .ok()
        .filter(|limits: &Limits| limits.schema_version == SCHEMA_VERSION)
        .unwrap_or_else(Limits::closed)
});

/// Items (`(definition id, count)`) and coins a message carries until it is claimed.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Attachments {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<(i32, u32)>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub coins: u32,
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}

impl Attachments {
    pub fn is_empty(&self) -> bool {
        self.items.is_empty() && self.coins == 0
    }

    /// "100 coins, 2 x item 1195".
    pub fn describe(&self) -> String {
        let coins = (self.coins > 0).then(|| format!("{} coins", self.coins));
        let items = self.items.iter().map(|(id, n)| format!("{n} x item {id}"));
        coins.into_iter().chain(items).collect::<Vec<_>>().join(", ")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Message {
    pub id: u32,
    /// A staff member's name, "Free Realms" or another system label.
    pub from: String,
    pub subject: String,
    pub body: String,
    /// Unix seconds.
    pub sent: u64,
    #[serde(default)]
    pub read: bool,
    #[serde(default, skip_serializing_if = "Attachments::is_empty")]
    pub attachments: Attachments,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub claimed: bool,
}

impl Message {
    /// Attachments still waiting for a claim.
    pub fn unclaimed(&self) -> bool {
        !self.claimed && !self.attachments.is_empty()
    }
}

/// One character's mail, oldest first on disk; readers show it newest first.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Mailbox {
    pub schema_version: u32,
    pub next_id: u32,
    pub messages: Vec<Message>,
}

impl Default for Mailbox {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            next_id: 1,
            messages: Vec::new(),
        }
    }
}

impl Mailbox {
    pub fn unread(&self) -> usize {
        self.messages.iter().filter(|message| !message.read).count()
    }

    /// The `n`th newest message (1 is the newest).
    pub fn nth_newest(&self, n: usize) -> Option<&Message> {
        n.checked_sub(1).and_then(|skip| self.messages.iter().rev().nth(skip))
    }

    /// 1-based newest-first position of message `id`.
    pub fn position(&self, id: u32) -> Option<usize> {
        let index = self.messages.iter().rposition(|message| message.id == id)?;
        Some(self.messages.len() - index)
    }

    fn get_mut(&mut self, id: u32) -> Option<&mut Message> {
        self.messages.iter_mut().find(|message| message.id == id)
    }
}

/// A refusal sentence (no trailing stop), or a storage failure.
#[derive(Debug)]
pub enum MailError {
    Refused(String),
    Io(io::Error),
}

impl fmt::Display for MailError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Refused(text) => f.write_str(text),
            Self::Io(error) => write!(f, "mail storage failed ({error})"),
        }
    }
}

impl std::error::Error for MailError {}

impl From<io::Error> for MailError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub(crate) fn refused(text: impl Into<String>) -> MailError {
    MailError::Refused(text.into())
}

pub fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

/// The realm's mail folder, beside its account store.
pub fn mail_dir(account_store: &Path) -> PathBuf {
    account_store.with_file_name("mail")
}

pub fn mailbox_path(account_store: &Path, guid: u64) -> PathBuf {
    mail_dir(account_store).join(format!("{guid}.json"))
}

// ponytail: one process-wide lock, because only the gateway process writes mail; a file lock
// like `AccountStoreLock` if a second process ever does.
static LOCK: Mutex<()> = Mutex::new(());

/// A mailbox as stored; a missing file is an empty mailbox.
pub fn load(path: &Path) -> io::Result<Mailbox> {
    let bytes = match fs::metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Mailbox::default()),
        Err(error) => return Err(error),
        Ok(meta) if !meta.is_file() || meta.len() > MAX_FILE_BYTES => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "mailbox is not a bounded regular file",
            ));
        }
        Ok(_) => fs::read(path)?,
    };
    let mailbox: Mailbox = serde_json::from_slice(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    if mailbox.schema_version != SCHEMA_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported mailbox schema",
        ));
    }
    Ok(mailbox)
}

/// Unread messages, 0 when the mailbox cannot be read (a notice is never worth an error).
pub fn unread(path: &Path) -> usize {
    load(path).map_or(0, |mailbox| mailbox.unread())
}

/// Read, change and atomically replace one mailbox under the mail lock. A refusal or a failed
/// write leaves the file as it was.
pub fn update<T>(
    path: &Path,
    change: impl FnOnce(&mut Mailbox) -> Result<T, MailError>,
) -> Result<T, MailError> {
    let _guard = LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let mut mailbox = load(path)?;
    let result = change(&mut mailbox)?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let text = serde_json::to_string_pretty(&mailbox).map_err(io::Error::other)? + "\n";
    crate::character_store::replace_bytes(path, text.as_bytes())?;
    Ok(result)
}

/// The fields a sender writes; the mailbox assigns the id and keeps `read`/`claimed`.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Letter {
    pub from: String,
    pub subject: String,
    pub body: String,
    #[serde(default)]
    pub attachments: Attachments,
}

impl Letter {
    /// The caps of `limits`, before anything is queued.
    pub fn validate(&self, limits: &Limits) -> Result<(), MailError> {
        let chars = |text: &str| text.chars().count();
        let control = |text: &str| text.chars().any(char::is_control);
        if self.subject.trim().is_empty() || self.body.trim().is_empty() {
            return Err(refused("a letter needs a subject and a body"));
        }
        if chars(&self.subject) > limits.max_subject_chars
            || chars(&self.body) > limits.max_body_chars
            || chars(&self.from) > limits.max_subject_chars
        {
            return Err(refused(format!(
                "a subject has at most {} characters and a body {}",
                limits.max_subject_chars, limits.max_body_chars
            )));
        }
        if [&self.from, &self.subject, &self.body].iter().any(|text| control(text)) {
            return Err(refused("a letter cannot contain control characters"));
        }
        let items = &self.attachments.items;
        if items.len() > limits.max_attachment_items
            || items.iter().any(|&(id, count)| id <= 0 || count == 0)
            || self.attachments.coins > limits.max_attachment_coins
        {
            return Err(refused(format!(
                "a letter carries at most {} item rows and {} coins",
                limits.max_attachment_items, limits.max_attachment_coins
            )));
        }
        Ok(())
    }
}

/// Put one letter in a mailbox and return its unread count. A full mailbox first drops its
/// oldest read message that holds nothing unclaimed (R); with none to drop it refuses.
pub fn deliver(path: &Path, letter: &Letter, limits: &Limits) -> Result<usize, MailError> {
    update(path, |mailbox| {
        if mailbox.messages.len() >= limits.max_messages {
            let spare = mailbox
                .messages
                .iter()
                .position(|message| message.read && !message.unclaimed())
                .ok_or_else(|| refused("that mailbox is full"))?;
            mailbox.messages.remove(spare);
        }
        let id = mailbox.next_id;
        if id == 0 || id > MAX_MESSAGE_ID {
            return Err(refused("that mailbox has no message ids left"));
        }
        mailbox.next_id = id + 1;
        mailbox.messages.push(Message {
            id,
            from: letter.from.clone(),
            subject: letter.subject.clone(),
            body: letter.body.clone(),
            sent: unix_now(),
            read: false,
            attachments: letter.attachments.clone(),
            claimed: false,
        });
        Ok(mailbox.unread())
    })
}

pub fn mark_read(path: &Path, id: u32) -> Result<(), MailError> {
    update(path, |mailbox| {
        let message = mailbox
            .get_mut(id)
            .ok_or_else(|| refused("that message is gone"))?;
        message.read = true;
        Ok(())
    })
}

/// Mark message `id`'s attachments claimed, once; the caller grants them.
pub fn mark_claimed(path: &Path, id: u32) -> Result<Attachments, MailError> {
    update(path, |mailbox| {
        let message = mailbox
            .get_mut(id)
            .ok_or_else(|| refused("that message is gone"))?;
        if !message.unclaimed() {
            return Err(refused("that message has nothing to claim"));
        }
        message.claimed = true;
        message.read = true;
        Ok(message.attachments.clone())
    })
}

/// Delete message `id`. A message still holding attachments is kept (R): claim them first.
pub fn delete(path: &Path, id: u32) -> Result<(), MailError> {
    update(path, |mailbox| {
        let index = mailbox
            .messages
            .iter()
            .position(|message| message.id == id)
            .ok_or_else(|| refused("that message is gone"))?;
        if mailbox.messages[index].unclaimed() {
            return Err(refused("claim that message's attachments before you delete it"));
        }
        mailbox.messages.remove(index);
        Ok(())
    })
}

/// (R) The line after the notice: the stock client has no mail window, so it says how to read.
pub const READ_HINT: &str =
    "Type realm mail to read your mail, or click a town postbox. realm mail claim 1 takes its gifts.";

/// The "You have N new messages" notice: the client's own row when the gateway has its locale,
/// else the same words. rc-0136 F4: it is the 77/2 broadcast (`chat::broadcast`, on screen and
/// in chat); a chat line alone was missed. The encoder escapes markup, so the row's font tag is
/// stripped rather than shown as text.
pub fn notice(unread: usize) -> Vec<u8> {
    let template = crate::world_text::locale_text(NEW_MESSAGES_TEXT_ID)
        .unwrap_or_else(|| NEW_MESSAGES_FALLBACK.to_owned());
    let mut plain = String::with_capacity(template.len());
    let mut in_tag = false;
    for c in template.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => plain.push(c),
            _ => {}
        }
    }
    crate::chat::broadcast(&plain.replace("%1", &unread.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("osfr-mail-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir.join("accounts.json")
    }

    fn letter(subject: &str) -> Letter {
        Letter {
            from: "Free Realms".into(),
            subject: subject.into(),
            body: "Hello.".into(),
            attachments: Attachments::default(),
        }
    }

    #[test]
    fn shipped_limits_are_usable() {
        let limits = &*LIMITS;
        assert_eq!(limits.schema_version, SCHEMA_VERSION);
        assert!(limits.max_messages > 0 && limits.max_body_chars > 0);
        assert!(limits.send_rate_per_second > 0 && limits.click_range > 0.0);
        assert_eq!(limits.mailboxes.len(), 5);
        for postbox in &limits.mailboxes {
            assert!((376..=381).contains(&postbox.game), "{}", postbox.town);
        }
    }

    #[test]
    fn delivery_counts_unread_and_newest_comes_first() {
        let store = scratch("order");
        let path = mailbox_path(&store, 7);
        assert_eq!(deliver(&path, &letter("one"), &LIMITS).unwrap(), 1);
        assert_eq!(deliver(&path, &letter("two"), &LIMITS).unwrap(), 2);
        let mailbox = load(&path).unwrap();
        assert_eq!(mailbox.nth_newest(1).unwrap().subject, "two");
        assert_eq!(mailbox.nth_newest(2).unwrap().subject, "one");
        assert!(mailbox.nth_newest(0).is_none() && mailbox.nth_newest(3).is_none());
        mark_read(&path, mailbox.nth_newest(1).unwrap().id).unwrap();
        assert_eq!(unread(&path), 1);
        let _ = fs::remove_dir_all(mail_dir(&store));
    }

    #[test]
    fn a_full_mailbox_drops_only_a_read_empty_message() {
        let store = scratch("full");
        let path = mailbox_path(&store, 8);
        let limits = Limits {
            max_messages: 2,
            ..LIMITS.clone()
        };
        let mut gift = letter("gift");
        gift.attachments.coins = 5;
        deliver(&path, &gift, &limits).unwrap();
        deliver(&path, &letter("b"), &limits).unwrap();
        // Nothing read: refused, the file unchanged.
        assert!(matches!(
            deliver(&path, &letter("c"), &limits),
            Err(MailError::Refused(_))
        ));
        mark_read(&path, 1).unwrap(); // the gift: read but unclaimed, so it stays
        assert!(deliver(&path, &letter("c"), &limits).is_err());
        mark_read(&path, 2).unwrap();
        deliver(&path, &letter("c"), &limits).unwrap();
        let subjects: Vec<_> = load(&path)
            .unwrap()
            .messages
            .into_iter()
            .map(|message| message.subject)
            .collect();
        assert_eq!(subjects, ["gift", "c"]);
        let _ = fs::remove_dir_all(mail_dir(&store));
    }

    #[test]
    fn attachments_are_claimed_once_and_guard_delete() {
        let store = scratch("claim");
        let path = mailbox_path(&store, 9);
        let mut gift = letter("gift");
        gift.attachments = Attachments {
            items: vec![(1195, 2)],
            coins: 10,
        };
        deliver(&path, &gift, &LIMITS).unwrap();
        assert!(delete(&path, 1).is_err(), "unclaimed attachments are kept");
        assert_eq!(mark_claimed(&path, 1).unwrap(), gift.attachments);
        assert!(mark_claimed(&path, 1).is_err(), "a second claim is refused");
        delete(&path, 1).unwrap();
        assert!(load(&path).unwrap().messages.is_empty());
        let _ = fs::remove_dir_all(mail_dir(&store));
    }

    #[test]
    fn letters_are_bounded_before_queueing() {
        let ok = letter("Hi");
        ok.validate(&LIMITS).unwrap();
        let long = Letter {
            body: "x".repeat(LIMITS.max_body_chars + 1),
            ..letter("Hi")
        };
        assert!(long.validate(&LIMITS).is_err());
        assert!(letter("").validate(&LIMITS).is_err());
        assert!(letter("a\u{7}b").validate(&LIMITS).is_err());
        let mut rich = letter("Hi");
        rich.attachments.items = vec![(0, 1)];
        assert!(rich.validate(&LIMITS).is_err());
        rich.attachments.items = vec![(1, 1); LIMITS.max_attachment_items + 1];
        assert!(rich.validate(&LIMITS).is_err());
    }

    #[test]
    fn a_corrupt_mailbox_is_an_error_not_an_empty_box() {
        let store = scratch("corrupt");
        let path = mailbox_path(&store, 10);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        crate::character_store::replace_bytes(&path, b"{not json").unwrap();
        assert!(load(&path).is_err());
        assert!(deliver(&path, &letter("x"), &LIMITS).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"{not json");
        let _ = fs::remove_dir_all(mail_dir(&store));
    }

    /// A job thread delivering to a mailbox while its owner marks and deletes letters (a login
    /// mid-job): every write is a locked read-modify-write, so no update is lost.
    #[test]
    fn concurrent_writers_lose_no_update() {
        let store = scratch("race");
        let path = mailbox_path(&store, 11);
        for _ in 0..20 {
            deliver(&path, &letter("old"), &LIMITS).unwrap();
        }
        let job = {
            let path = path.clone();
            std::thread::spawn(move || {
                for _ in 0..20 {
                    deliver(&path, &letter("new"), &LIMITS).unwrap();
                }
            })
        };
        for id in 1..=20 {
            mark_read(&path, id).unwrap();
            if id % 2 == 0 {
                delete(&path, id).unwrap();
            }
        }
        job.join().unwrap();
        let mailbox = load(&path).unwrap();
        let count = |subject: &str| mailbox.messages.iter().filter(|m| m.subject == subject).count();
        assert_eq!((count("old"), count("new")), (10, 20));
        assert!(mailbox.messages.iter().filter(|m| m.subject == "old").all(|m| m.read));
        assert_eq!(mailbox.unread(), 20);
        let _ = fs::remove_dir_all(mail_dir(&store));
    }

    #[test]
    fn the_notice_is_the_client_row_without_markup() {
        let packet = notice(3);
        let text = crate::chat::broadcast_text(&packet).unwrap_or_default();
        assert_eq!(text, "You have 3 new messages");
    }
}
