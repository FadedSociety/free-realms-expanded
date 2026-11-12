//! Mail in the player's own session, answered by the gateway before gameplay sees the packet
//! (`Gateway::staff_packet`): the chat commands, a town postbox click and the mail dialog's
//! buttons. Only "Mail Sorting" on a postbox is left to gameplay (`gameplay/npc/mailbox.rs`),
//! which starts it through the Games window's own gates.
//!
//! Chat. `realm mail [n]`, `realm mail claim <n>`, `realm mail delete <n>` for every player
//! (newest first; `n` counts from the newest), and the same words after `/mail` for a client
//! that sends a slash line: the stock client runs `/` lines in its own console and answers an
//! unknown one itself (`Console_ExecuteCommandLine` 0x00a9e480, `Console_ReportUnsupportedCommand`
//! 0x00a9e2d0; its registered words are only the chat channels, `attack`, `hit` and `report`,
//! `ClientChatProcessor` 0x00b15450 and `ClientCommandProcessor` 0x00aa06a0), so in the stock
//! client the `realm` form is the one that reaches us. Staff (admins and moderators,
//! `can_run_realm_command("mail")`) send with
//! `realm mail <to> | <subject> | <body> [| <attachments>]`.
//!
//! Dialog. The 26/3 window (`ShowDialogFromPacket` 0x009f5ae0) shows locale ids only, so the
//! letter is printed to chat and the window carries the stock buttons: Claim (386405), Delete
//! (118151), Next (8061) and Goodbye (8075), under "Mail" (21422). A postbox offers "Read mail"
//! (6100101, a `data/release/locale-additions.json` row: the stock locale has no such words)
//! and "Mail Sorting" (35595) under "Mailbox" (4095). A row's response id carries its action and
//! the message id (`response`), so no dialog state is kept: a forged id can only act on the
//! sender's own mailbox, as the chat words can.

use super::{LIMITS, Letter, Mailbox, MailError, Message, load, mailbox_path, unix_now};
use crate::account_store::AccountRole;
use crate::character_store::CharacterState;
use crate::gameplay::Gameplay;
use crate::npc_interaction::{self as dialog, DialoguePacket26_3};
use std::path::Path;

/// "Mail" (ucdt), the read dialog's body.
pub const MAIL_TEXT_ID: u32 = 21_422;
/// "Mailbox" (ucdt), the postbox dialog's body.
pub const MAILBOX_TEXT_ID: u32 = 4_095;
pub const CLAIM_TEXT_ID: u32 = 386_405;
pub const DELETE_TEXT_ID: u32 = 118_151;
pub const NEXT_TEXT_ID: u32 = 8_061;
/// "Mail Sorting" (ucdt).
pub const MAIL_SORTING_TEXT_ID: u32 = 35_595;
/// (R) "Read mail": the one new locale row, above every stock id and below the World Builder's
/// authored range (`world_text::FIRST_WORLD_TEXT_ID`).
pub const READ_MAIL_TEXT_ID: u32 = 6_100_101;
/// The locale rows this module adds through `locale-additions.json`.
pub const LOCALE_ROWS: [(u32, &str); 1] = [(READ_MAIL_TEXT_ID, "Read mail")];

/// The open world, the only zone the town postboxes stand in (`gm_menu_routing`'s rule).
const OPEN_WORLD_ZONE: u32 = 0;
/// (R) A dialog row id with 'M' in its top byte: clear of every other row (1..=5, 300, 301,
/// 310 and up for explainer pages). Bits 20..24 hold the action, the low 20 bits a message id
/// (or a Mail Sorting game).
const RESPONSE_TAG: i32 = 0x4D00_0000;
const VALUE_MASK: i32 = 0xF_FFFF;

const DENIED: &str =
    "Cannot do that: developer command requires --dev-tools and an authorized account role.";
const USAGE: &str = "Mail: realm mail [n] reads your mail, newest first; realm mail claim <n> \
    takes its attachments; realm mail delete <n> removes it.";
const SEND_USAGE: &str = "Usage: realm mail <First Last[, First Last...] | everyone | online | \
    role <role> | zone <id>> | <subject> | <body> [| <coins>c <item>x<count> ...]";
const NO_STORE: &str = "Mail is not available on this realm (it needs the account store).";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Read = 1,
    Claim = 2,
    Delete = 3,
    Next = 4,
    Sort = 5,
}

/// A dialog row's response id.
pub fn response(action: Action, value: u32) -> i32 {
    RESPONSE_TAG | ((action as i32) << 20) | (value as i32 & VALUE_MASK)
}

/// The action and value of a mail row id; `None` for every other row.
pub fn decode_response(id: i32) -> Option<(Action, u32)> {
    if id & !0x00FF_FFFF != RESPONSE_TAG {
        return None;
    }
    let action = match (id >> 20) & 0xF {
        1 => Action::Read,
        2 => Action::Claim,
        3 => Action::Delete,
        4 => Action::Next,
        5 => Action::Sort,
        _ => return None,
    };
    Some((action, (id & VALUE_MASK) as u32))
}

/// The postbox Mail Sorting game a 26/6 row names, for gameplay.
pub fn sorting_game(response_id: i32) -> Option<i32> {
    let (Action::Sort, game) = decode_response(response_id)? else {
        return None;
    };
    let game = i32::try_from(game).ok()?;
    LIMITS
        .mailboxes
        .iter()
        .any(|postbox| postbox.game == game)
        .then_some(game)
}

/// Whom a staff chat send is for, before the host resolves names and the live roster.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Names(Vec<String>),
    Everyone,
    Online,
    Role(AccountRole),
    Zone(u32),
}

/// A staff send staged with the sender's transaction; the host starts it after the commit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StagedSend {
    pub to: Target,
    /// `from` is filled by the host with the sender's character name.
    pub letter: Letter,
}

/// This session's mail state on its gateway (a new gateway per login, so a new notice).
#[derive(Clone, Debug, Default)]
pub struct Slot {
    pub sends: Vec<StagedSend>,
    pub notified: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Read(usize),
    Claim(usize),
    Delete(usize),
    Help,
    Send(StagedSend),
}

/// What the gateway sends back: inner packets, whether the character changed (a claim), and
/// whether the role table refused the line.
#[derive(Debug, Default)]
pub struct Reply {
    pub packets: Vec<Vec<u8>>,
    pub changed: bool,
    pub denied: bool,
}

impl Reply {
    fn line(text: &str) -> Self {
        Self {
            packets: vec![crate::chat::system(text)],
            ..Self::default()
        }
    }
}

/// `mail ...` after `realm `, without a send's `|`: every player's words, so the role table's
/// refusal (`Gameplay::developer_command_denied`) never counts a reader as a denied command.
pub fn is_reader(command: &str) -> bool {
    (command == "mail" || command.starts_with("mail ")) && !command.contains('|')
}

/// `realm mail ...` or `/mail ...`; `None` for any other line. `Err` is the usage to show.
pub fn parse(text: &str) -> Option<Result<Command, &'static str>> {
    let text = text.trim();
    let lower = text.to_ascii_lowercase();
    let (realm, at) = ["realm mail", "/mail"]
        .iter()
        .find(|prefix| {
            lower
                .strip_prefix(**prefix)
                .is_some_and(|rest| rest.is_empty() || rest.starts_with(' '))
        })
        .map(|prefix| (prefix.starts_with('r'), prefix.len()))?;
    // ASCII lowercasing keeps byte offsets, so `raw` keeps the sender's case.
    let raw = text[at..].trim();
    if raw.contains('|') {
        return Some(if realm { parse_send(raw) } else { Err(SEND_USAGE) });
    }
    let words: Vec<String> = raw
        .split_whitespace()
        .map(str::to_ascii_lowercase)
        .collect();
    let number = |word: &str| word.parse::<usize>().ok().filter(|n| *n > 0);
    Some(match words.iter().map(String::as_str).collect::<Vec<_>>()[..] {
        [] => Ok(Command::Read(1)),
        ["help"] => Ok(Command::Help),
        [n] => number(n).map(Command::Read).ok_or(USAGE),
        ["claim", n] => number(n).map(Command::Claim).ok_or(USAGE),
        ["delete", n] => number(n).map(Command::Delete).ok_or(USAGE),
        _ => Err(USAGE),
    })
}

fn parse_send(raw: &str) -> Result<Command, &'static str> {
    let parts: Vec<&str> = raw.split('|').map(str::trim).collect();
    let (to, subject, body, extras) = match parts[..] {
        [to, subject, body] => (to, subject, body, ""),
        [to, subject, body, extras] => (to, subject, body, extras),
        _ => return Err(SEND_USAGE),
    };
    let lower = to.to_ascii_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();
    let to = match words[..] {
        ["everyone"] | ["all"] => Target::Everyone,
        ["online"] => Target::Online,
        ["role", role] => Target::Role(AccountRole::parse(role).ok_or(SEND_USAGE)?),
        ["zone", zone] => Target::Zone(zone.parse().map_err(|_| SEND_USAGE)?),
        [] => return Err(SEND_USAGE),
        _ => Target::Names(
            to.split(',')
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(str::to_owned)
                .collect(),
        ),
    };
    let mut letter = Letter {
        subject: subject.to_owned(),
        body: body.to_owned(),
        ..Letter::default()
    };
    for token in extras.split(|c: char| c.is_whitespace() || c == ',') {
        let token = token.trim().to_ascii_lowercase();
        if token.is_empty() {
            continue;
        }
        if let Some(coins) = token.strip_suffix('c') {
            let coins: u32 = coins.parse().map_err(|_| SEND_USAGE)?;
            letter.attachments.coins = letter.attachments.coins.saturating_add(coins);
            continue;
        }
        let (id, count) = token.split_once('x').unwrap_or((&token, "1"));
        let (Ok(id), Ok(count)) = (id.parse::<i32>(), count.parse::<u32>()) else {
            return Err(SEND_USAGE);
        };
        letter.attachments.items.push((id, count));
    }
    Ok(Command::Send(StagedSend { to, letter }))
}

/// The gateway's mail stop for one application packet; `None` passes it on to gameplay.
/// `store` is the session's account store (mail is a realm feature: none, no mail).
pub fn intercept(
    slot: &mut Slot,
    store: Option<&Path>,
    game: &mut Gameplay,
    inner: &[u8],
    state: &CharacterState,
    now_ms: u64,
) -> Option<Reply> {
    if inner.starts_with(&[15, 0, 1, 0]) {
        let text = crate::chat::request(inner, state.guid).ok().flatten()?;
        let command = match parse(&text)? {
            Ok(command) => command,
            Err(usage) => return Some(Reply::line(usage)),
        };
        return Some(run(slot, store, game, command, state, now_ms));
    }
    if inner.starts_with(&[26, 0, 8, 0]) {
        return postbox_click(game, inner, state);
    }
    if inner.starts_with(&[26, 0, 6, 0]) {
        let id = dialog::decode_dialogue_response_26_6(inner).ok()?;
        let (action, value) = decode_response(id)?;
        if action == Action::Sort {
            return None; // gameplay starts it (`Gameplay::mailbox_sorting_26_6`)
        }
        let Some(store) = store else {
            return Some(Reply::line(NO_STORE));
        };
        let path = mailbox_path(store, state.guid);
        return Some(match action {
            Action::Read => show(&path, 1, game.mail_busy()),
            Action::Next => {
                let at = load(&path).ok().and_then(|mailbox| mailbox.position(value));
                show(&path, at.map_or(1, |at| at + 1), game.mail_busy())
            }
            Action::Claim => claim(&path, value, game, now_ms),
            Action::Delete => delete(&path, value, game.mail_busy()),
            Action::Sort => Reply::default(),
        });
    }
    None
}

fn run(
    slot: &mut Slot,
    store: Option<&Path>,
    game: &mut Gameplay,
    command: Command,
    state: &CharacterState,
    now_ms: u64,
) -> Reply {
    if let Command::Send(send) = command {
        return stage(slot, store, game, send);
    }
    let Some(store) = store else {
        return Reply::line(NO_STORE);
    };
    let path = mailbox_path(store, state.guid);
    let id_at = |n: usize| -> Result<u32, Reply> {
        let mailbox = load(&path).map_err(|_| Reply::line("Your mailbox cannot be read."))?;
        mailbox
            .nth_newest(n)
            .map(|message| message.id)
            .ok_or_else(|| Reply::line(&format!("You have {} messages.", mailbox.messages.len())))
    };
    match command {
        Command::Help => Reply::line(USAGE),
        Command::Read(n) => show(&path, n, game.mail_busy()),
        Command::Claim(n) => id_at(n).map_or_else(|r| r, |id| claim(&path, id, game, now_ms)),
        Command::Delete(n) => id_at(n).map_or_else(|r| r, |id| delete(&path, id, game.mail_busy())),
        Command::Send(_) => Reply::default(),
    }
}

/// A staff send: role gate first (nothing about recipients is read for a refused sender), then
/// the letter's caps and its item ids against this session's item tables, then staged.
fn stage(slot: &mut Slot, store: Option<&Path>, game: &Gameplay, send: StagedSend) -> Reply {
    if !game.may_run("mail") {
        return Reply {
            denied: true,
            ..Reply::line(DENIED)
        };
    }
    if store.is_none() {
        return Reply::line(NO_STORE);
    }
    // The name is the host's to fill; a placeholder of the longest name keeps the cap honest.
    let probe = Letter {
        from: "Free Realms".into(),
        ..send.letter.clone()
    };
    if let Err(error) = probe.validate(&LIMITS) {
        return Reply::line(&format!("Cannot send mail: {error}."));
    }
    if !game.mail_items_known(&send.letter.attachments.items) {
        return Reply::line("Cannot send mail: an attached item id is not in the item tables.");
    }
    if matches!(&send.to, Target::Names(names) if names.is_empty()) {
        return Reply::line(SEND_USAGE);
    }
    slot.sends.push(send);
    Reply::default()
}

fn age(sent: u64) -> String {
    match unix_now().saturating_sub(sent) / 86_400 {
        0 => "today".into(),
        1 => "1 day ago".into(),
        days => format!("{days} days ago"),
    }
}

/// The letter's chat lines, newest-first position `n` of `total`.
fn letter_lines(message: &Message, n: usize, total: usize) -> Vec<Vec<u8>> {
    let new = if message.read { "" } else { " (new)" };
    let mut lines = vec![
        crate::chat::system(&format!(
            "Mail {n} of {total}{new}: \"{}\" from {}, {}.",
            message.subject,
            message.from,
            age(message.sent)
        )),
        crate::chat::system(&message.body),
    ];
    if !message.attachments.is_empty() {
        let state = if message.claimed {
            "claimed".to_owned()
        } else {
            format!("claim it with realm mail claim {n}")
        };
        lines.push(crate::chat::system(&format!(
            "Attached: {} ({state}).",
            message.attachments.describe()
        )));
    }
    lines
}

/// A 26/3 for the dialog window: `body` and `rows`, spoken by `actor` (0: no speaker). The
/// client rebuilds the window only while it is hidden (T844), so a 26/4 goes first.
fn dialog_packets(body: u32, rows: Vec<[u32; 5]>, actor: u64) -> Vec<Vec<u8>> {
    let packet = dialog::encode_dialogue_26_3(&DialoguePacket26_3 {
        initial_dwords: [body, 0, actor as u32, (actor >> 32) as u32],
        initial_flag: false,
        initial_float_bits: 0,
        choices: rows,
        choice_vectors: [[0; 4]; 2],
        vector_flag: false,
        final_vector: [0; 4],
        float_after_vector_bits: 0,
        final_flags: [false; 3],
        final_float_bits: 0,
        final_dword: 0,
    });
    let mut packets = vec![dialog::encode_dialogue_close_26_4()];
    packets.extend(packet.ok());
    packets
}

fn row(action: Action, value: u32, label: u32) -> [u32; 5] {
    dialog::dialogue_choice_row(response(action, value), label)
}

/// Show the `n`th newest letter: chat lines, the dialog (not during an activity, whose own
/// window it would cover) and the read mark.
fn show(path: &Path, n: usize, busy: bool) -> Reply {
    let mailbox = match load(path) {
        Ok(mailbox) => mailbox,
        Err(_) => return Reply::line("Your mailbox cannot be read."),
    };
    let total = mailbox.messages.len();
    let Some(message) = mailbox.nth_newest(n).cloned() else {
        let mut reply = Reply::line(&if total == 0 {
            "You have no mail.".to_owned()
        } else {
            format!("You have {total} messages; that was the last.")
        });
        reply.packets.push(dialog::encode_dialogue_close_26_4());
        return reply;
    };
    let mut packets = letter_lines(&message, n, total);
    if !busy {
        packets.extend(dialog_packets(MAIL_TEXT_ID, read_rows(&mailbox, &message, n), 0));
    }
    if !message.read
        && let Err(error) = super::mark_read(path, message.id)
    {
        eprintln!("mail: read mark for {}: {error}", path.display());
    }
    Reply {
        packets,
        ..Reply::default()
    }
}

fn read_rows(mailbox: &Mailbox, message: &Message, n: usize) -> Vec<[u32; 5]> {
    let mut rows = Vec::new();
    if message.unclaimed() {
        rows.push(row(Action::Claim, message.id, CLAIM_TEXT_ID));
    } else {
        rows.push(row(Action::Delete, message.id, DELETE_TEXT_ID));
    }
    if n < mailbox.messages.len() {
        rows.push(row(Action::Next, message.id, NEXT_TEXT_ID));
    }
    rows.push(dialog::goodbye_dialogue_choice());
    rows
}

/// Claim message `id`: the grant runs on a copy of the player's gameplay first (bag space,
/// stacking, catalogue: the reward path), then the mailbox marks it claimed under the mail
/// lock, and only then does the copy become the player's. A full bag or a second claim
/// changes nothing. The gateway journals the staged mailbox write with the character save,
/// so a failed packet cannot consume attachments without granting them.
fn claim(path: &Path, id: u32, game: &mut Gameplay, now_ms: u64) -> Reply {
    if game.mail_busy() {
        return Reply::line("Finish what you are doing before you claim mail.");
    }
    let mailbox = match load(path) {
        Ok(mailbox) => mailbox,
        Err(_) => return Reply::line("Your mailbox cannot be read."),
    };
    let Some(message) = mailbox.messages.iter().find(|m| m.id == id).cloned() else {
        return Reply::line("That message is gone.");
    };
    if !message.unclaimed() {
        return Reply::line("That message has nothing to claim.");
    }
    let mut trial = game.clone();
    let attachments = &message.attachments;
    // The reward path's own refusal: "inventory is full", or a row the catalogue lost.
    let granted = match trial.mail_grant(attachments.coins, &attachments.items, now_ms) {
        Ok(granted) => granted,
        Err(error) => return Reply::line(&format!("Cannot claim this mail: {error}.")),
    };
    match super::mark_claimed(path, id) {
        Ok(_) => {}
        Err(MailError::Refused(text)) => return Reply::line(&format!("Cannot claim: {text}.")),
        Err(MailError::Io(_)) => return Reply::line("Your mailbox cannot be saved; try again."),
    }
    *game = trial;
    let mut packets = granted.replies;
    packets.push(crate::chat::system(&format!(
        "You claimed {}.",
        attachments.describe()
    )));
    let n = mailbox.position(id).unwrap_or(1);
    let mut claimed = message;
    claimed.claimed = true;
    claimed.read = true;
    packets.extend(dialog_packets(
        MAIL_TEXT_ID,
        read_rows(&mailbox, &claimed, n),
        0,
    ));
    Reply {
        packets,
        changed: true,
        denied: false,
    }
}

fn delete(path: &Path, id: u32, busy: bool) -> Reply {
    let n = load(path)
        .ok()
        .and_then(|mailbox| mailbox.position(id))
        .unwrap_or(1);
    if let Err(error) = super::delete(path, id) {
        return Reply::line(&format!("Cannot delete: {error}."));
    }
    let mut reply = show(path, n, busy);
    reply.packets.insert(0, crate::chat::system("Message deleted."));
    reply
}

/// A click on a town postbox: Read mail and the town's Mail Sorting. Out of reach or outside
/// the open world it answers nothing, like an NPC click (T836).
fn postbox_click(game: &Gameplay, inner: &[u8], state: &CharacterState) -> Option<Reply> {
    let request = dialog::decode_npc_interact_26_8(inner).ok()?;
    let postbox = LIMITS.postbox(request.target)?;
    let reach = state
        .position
        .iter()
        .zip(postbox.position)
        .map(|(a, b)| (a - b) * (a - b))
        .sum::<f32>()
        .sqrt();
    if state.zone_id != OPEN_WORLD_ZONE || !(reach <= LIMITS.click_range) {
        return Some(Reply::default());
    }
    if game.mail_busy() {
        return Some(Reply::line("Finish what you are doing first."));
    }
    let rows = vec![
        row(Action::Read, 0, READ_MAIL_TEXT_ID),
        row(Action::Sort, postbox.game as u32, MAIL_SORTING_TEXT_ID),
        dialog::goodbye_dialogue_choice(),
    ];
    Some(Reply {
        packets: dialog_packets(MAILBOX_TEXT_ID, rows, postbox.guid),
        ..Reply::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_ids_round_trip_and_stay_clear_of_other_rows() {
        for action in [Action::Read, Action::Claim, Action::Delete, Action::Next, Action::Sort] {
            for value in [0, 1, 376, super::super::MAX_MESSAGE_ID] {
                assert_eq!(decode_response(response(action, value)), Some((action, value)));
            }
        }
        for other in [0, 1, 5, 300, 301, 310, 315, -1, i32::MAX, 0x4D00_0000] {
            assert_eq!(decode_response(other), None, "{other}");
        }
        assert_eq!(sorting_game(response(Action::Sort, 377)), Some(377));
        assert_eq!(sorting_game(response(Action::Sort, 999)), None);
        assert_eq!(sorting_game(response(Action::Read, 377)), None);
    }

    #[test]
    fn player_words_parse_and_anything_else_passes() {
        assert_eq!(parse("realm mail"), Some(Ok(Command::Read(1))));
        assert_eq!(parse("  /MAIL 2 "), Some(Ok(Command::Read(2))));
        assert_eq!(parse("realm mail claim 3"), Some(Ok(Command::Claim(3))));
        assert_eq!(parse("/mail delete 1"), Some(Ok(Command::Delete(1))));
        assert_eq!(parse("realm mail 0"), Some(Err(USAGE)));
        assert_eq!(parse("realm mail claim"), Some(Err(USAGE)));
        assert_eq!(parse("/mail Pip | a | b"), Some(Err(SEND_USAGE)));
        for other in ["realm mailbox", "realm mine", "mail", "hello /mail", "realm help"] {
            assert_eq!(parse(other), None, "{other}");
        }
    }

    #[test]
    fn staff_sends_parse_targets_and_attachments() {
        let Some(Ok(Command::Send(send))) =
            parse("realm mail Pip Player, Ann Lee Moss | Hello | Body text | 50c 1195x2 2862")
        else {
            panic!("a send");
        };
        assert_eq!(
            send.to,
            Target::Names(vec!["Pip Player".into(), "Ann Lee Moss".into()])
        );
        assert_eq!((send.letter.subject.as_str(), send.letter.body.as_str()), ("Hello", "Body text"));
        assert_eq!(send.letter.attachments.coins, 50);
        assert_eq!(send.letter.attachments.items, vec![(1195, 2), (2862, 1)]);
        let target = |text: &str| match parse(text) {
            Some(Ok(Command::Send(send))) => Some(send.to),
            _ => None,
        };
        assert_eq!(target("realm mail everyone | a | b"), Some(Target::Everyone));
        assert_eq!(target("realm mail ONLINE | a | b"), Some(Target::Online));
        assert_eq!(
            target("realm mail role moderator | a | b"),
            Some(Target::Role(AccountRole::Moderator))
        );
        assert_eq!(target("realm mail zone 0 | a | b"), Some(Target::Zone(0)));
        for bad in [
            "realm mail role king | a | b",
            "realm mail zone x | a | b",
            "realm mail Pip | a",
            "realm mail Pip | a | b | c | d",
            "realm mail Pip | a | b | lots",
            "realm mail | a | b",
        ] {
            assert_eq!(parse(bad), Some(Err(SEND_USAGE)), "{bad}");
        }
    }
}
