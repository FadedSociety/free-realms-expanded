//! The client's Claim window (`HandlerClaimSystem`, Marketplace > Claim) for gifts, answered by
//! the gateway before gameplay sees the packet (`Gateway::staff_packet`), and `realm redeem`.
//!
//! The claim family is opcode 108 with a u16 sub-opcode (`ClaimPacketDispatcher` 0x00a056a0):
//! - **108/1** `GetAllAvailableClaimItemsRequest` (header only, ctor 0x009ff3c0, sent by
//!   0x00a04a30 while the client's list is not valid). Answered with
//! - **108/2** `GetAllAvailableClaimItemsResponse` (reader 0x00a04950): a success byte, then the
//!   claim rows (0x00a04640 count, 0x00a040e0 each). The rows replace the client's whole list
//!   (the packet reads into the manager's own array) and mark it valid; for each row the client
//!   posts a `ClaimNotification` game action (`IterateArrayEntriesNotify` 0x00a02d30,
//!   `GaqItemClaimItem` 0x00a0e090), so the 108/2 sent at login and on arrival is the on-screen
//!   notice, beside a chat line.
//! - **108/3** `ClaimItemsRequest` (writer 0x00a045d0): tracker u32, count u32, claim ids u64
//!   (one per unit; `ClaimScreen.AddClaimItem` then `FinishClaim`, 0x00c00de0 -> 0x00a052e0).
//!   Answered with
//! - **108/4** `ClaimItemsResponse` (reader 0x00a049b0): tracker, success byte, then the rows,
//!   which again replace the list (ctor 0x00a00490 points the packet at the manager's array), so
//!   the answer carries what is still waiting. No tracker matches a plain claim, so the client
//!   fires `OnClaimItems(success)` (0x00a05490) and the Lua shows "claim success" or the error.
//! - **108/5** `ClaimItemsItemDefinitionsRequest` (writer 0x00a04520): tracker, then item ids,
//!   for a tooltip, a bundle's contents or "use it now". Answered with **108/6** (reader
//!   0x00a041b0): tracker, a byte, then a length-prefixed blob of `ClientItemDefinition`s.
//!
//! A row (0x00a040e0): u32 n + n u64 claim ids (the StackCount column), NameId, DescriptionId,
//! ImageSetId and tint (into +0x158, the AvailableItems columns 0x00c695a0), the contents (u32 n,
//! then per item a tint, the item id and the quantity, which `ClaimItemTooltip_BuildRows`
//! 0x00a77680 shows), and a u64 giver. `progression_extras::ClaimItem` encodes it.
//!
//! Character claims (achievement rewards, `progression_extras`) share the window: their rows are
//! listed beside the gifts, and a claim of only character rows goes on to gameplay as before.
use super::{Catalog, Events, GiftError, Pending, Rules, Stock, Toggles, box_path, load, take, unix_now};
use crate::character_store::CharacterState;
use crate::gameplay::Gameplay;
use crate::progression_extras::{ClaimItem, ClaimItemEntry};
use crate::redeem_codes::{self, Codes};
use std::io;
use std::path::Path;

/// (R) The most ids one claim request may name (the client sends one row's worth).
const MAX_CLAIM_IDS: usize = 256;

/// What the gateway sends back, and whether the character changed (a claim).
pub struct Reply {
    pub packets: Vec<Vec<u8>>,
    pub changed: bool,
}

impl Reply {
    fn of(packets: Vec<Vec<u8>>) -> Self {
        Self {
            packets,
            changed: false,
        }
    }

    fn line(text: &str) -> Self {
        Self::of(vec![crate::chat::system(text)])
    }
}

/// What this session's gifts are judged by, read once per request and owned, so a claim can
/// replace the gameplay it was read from: the catalog, the toggles, the realm's limited stock,
/// and the events running for this session (`EventCalendar::is_running` through gameplay's
/// season table and hand toggles).
pub struct Realm {
    pub catalog: Catalog,
    pub toggles: Toggles,
    pub stock: Stock,
    pub events: Events,
}

impl Realm {
    pub fn load(store: &Path, game: &Gameplay) -> io::Result<Self> {
        Ok(Self {
            catalog: Catalog::load(&game.gift_content_dir())?,
            toggles: Toggles::load(store),
            stock: Stock::read(store)?,
            events: game.gift_running_events(),
        })
    }

    pub fn rules(&self) -> Rules<'_> {
        Rules {
            catalog: &self.catalog,
            toggles: &self.toggles,
            events: &self.events,
            stock: &self.stock,
        }
    }
}

fn header(sub: u16) -> Vec<u8> {
    [108u16.to_le_bytes(), sub.to_le_bytes()].concat()
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

/// One gift's row, shown as its first item. `None` for an item the tables do not know.
fn gift_row(game: &Gameplay, pending: &Pending) -> Option<Vec<u8>> {
    let &(first, _) = pending.items.first()?;
    let shown = game.gift_presentation(first)?;
    let entries = (pending.items.iter())
        .map(|&(item, count)| {
            Some(ClaimItemEntry {
                field_4: 0, // (R) the entry's ItemRecord tint
                item_id: u32::try_from(item).ok()?,
                field_14: count,
            })
        })
        .collect::<Option<Vec<_>>>()?;
    let as_u32 = |value: i32| u32::try_from(value).unwrap_or(0);
    ClaimItem {
        claim_ids: vec![pending.claim_id()],
        name_id: as_u32(shown.name_id),
        description_id: as_u32(shown.description_id),
        icon: [as_u32(shown.icon_id), as_u32(shown.icon_tint)],
        entries,
        field_150: [0; 2],
    }
    .encode()
    .ok()
}

/// Every row `character` may claim now: its own character claims, then the account's gifts.
pub fn rows(game: &Gameplay, waiting: &[Pending], rules: &Rules, character: u64) -> Vec<Vec<u8>> {
    let mut rows = game.gift_claim_rows();
    rows.extend(
        (waiting.iter())
            .filter(|pending| rules.shows(pending, character))
            .filter_map(|pending| gift_row(game, pending)),
    );
    rows
}

fn with_rows(mut packet: Vec<u8>, rows: &[Vec<u8>]) -> Vec<u8> {
    put_u32(&mut packet, u32::try_from(rows.len()).unwrap_or(0));
    for row in rows {
        packet.extend_from_slice(row);
    }
    packet
}

/// The 108/2 list.
pub fn list_packet(rows: &[Vec<u8>]) -> Vec<u8> {
    let mut packet = header(2);
    packet.push(1);
    with_rows(packet, rows)
}

fn claim_answer(tracker: u32, ok: bool, rows: &[Vec<u8>]) -> Vec<u8> {
    let mut packet = header(4);
    put_u32(&mut packet, tracker);
    packet.push(u8::from(ok));
    with_rows(packet, rows)
}

/// `(tracker, claim ids)` of a 108/3, or `None` for anything else.
fn claim_request(inner: &[u8]) -> Option<(u32, Vec<u64>)> {
    if !inner.starts_with(&[108, 0, 3, 0]) {
        return None;
    }
    let tracker = crate::le::u32(inner, 4)?;
    let count = usize::try_from(crate::le::u32(inner, 8)?).ok()?;
    if count == 0 || count > MAX_CLAIM_IDS || inner.len() != 12 + count * 8 {
        return None;
    }
    let ids = (0..count).map(|n| crate::le::u64(inner, 12 + n * 8)).collect::<Option<_>>()?;
    Some((tracker, ids))
}

/// `(tracker, item ids)` of a 108/5.
fn definitions_request(inner: &[u8]) -> Option<(u32, Vec<i32>)> {
    if !inner.starts_with(&[108, 0, 5, 0]) {
        return None;
    }
    let tracker = crate::le::u32(inner, 4)?;
    let count = usize::try_from(crate::le::u32(inner, 8)?).ok()?;
    if count == 0 || count > MAX_CLAIM_IDS || inner.len() != 12 + count * 4 {
        return None;
    }
    (0..count)
        .map(|n| crate::le::i32(inner, 12 + n * 4))
        .collect::<Option<Vec<_>>>()
        .map(|ids| (tracker, ids))
}

/// Answer a Claim window packet or a `realm redeem <code>` line for a gift; `None` sends the
/// packet on to gameplay (no account store, a character claim, any other packet).
pub fn intercept(
    account: Option<(&Path, u64)>,
    game: &mut Gameplay,
    inner: &[u8],
    state: &CharacterState,
    now_ms: u64,
) -> Option<Reply> {
    let (store, account) = account?;
    let character = state.guid;
    if inner == header(1).as_slice() {
        return Some(list(store, account, game, character));
    }
    if let Some((tracker, ids)) = claim_request(inner) {
        return claim(store, account, game, character, tracker, &ids, now_ms);
    }
    if let Some((tracker, ids)) = definitions_request(inner) {
        return definitions(store, account, game, character, tracker, &ids);
    }
    let text = crate::chat::request(inner, character).ok().flatten()?;
    let lower = text.trim().to_ascii_lowercase();
    // The panel's `!redeem <code>` (Sanctuary PR #163, `gm_panel.rs`) is the same words.
    let code = (lower.strip_prefix("realm redeem "))
        .or_else(|| lower.strip_prefix("!redeem "))?
        .trim();
    // `!redeem` alone (the panel's code box) and an empty code stay gameplay's.
    (!code.is_empty()).then(|| redeem(store, account, game, character, code, now_ms))
}

/// Sync the standing offers, then the whole list.
pub fn list(store: &Path, account: u64, game: &Gameplay, character: u64) -> Reply {
    Reply::of(vec![refresh(store, account, game, character).packet])
}

/// A synced list: the 108/2, how many gift rows it shows, and how many offers just arrived.
pub struct Listing {
    pub packet: Vec<u8>,
    pub gifts: usize,
    pub added: usize,
}

/// Add the due standing offers to the account's box, then build the whole list.
pub fn refresh(store: &Path, account: u64, game: &Gameplay, character: u64) -> Listing {
    let own = game.gift_claim_rows();
    let Ok(realm) = Realm::load(store, game) else {
        return Listing { packet: list_packet(&own), gifts: 0, added: 0 };
    };
    let rules = realm.rules();
    let added = super::sync_offers(store, account, character, &rules, unix_now())
        .unwrap_or_else(|error| {
            log("gift_offers_failed", account, &error.to_string());
            0
        });
    let waiting = load(&box_path(store, account)).map(|gifts| gifts.pending).unwrap_or_default();
    let all = rows(game, &waiting, &rules, character);
    Listing {
        gifts: all.len().saturating_sub(own.len()),
        packet: list_packet(&all),
        added,
    }
}

/// The calendar events running for this session, with their runs (the tick syncs when they move).
pub fn running_events(game: &Gameplay) -> Events {
    game.gift_running_events()
}

fn log(event: &str, account: u64, error: &str) {
    eprintln!("{}", serde_json::json!({"event": event, "account": account, "error": error}));
}

/// A claim of gift rows: the grant runs on a copy of the player's gameplay first (bag space,
/// stacking, the item tables: the reward path), then the box gives the gifts up under its lock,
/// and only then does the copy become the player's. A full bag or a second claim changes nothing.
fn claim(
    store: &Path,
    account: u64,
    game: &mut Gameplay,
    character: u64,
    tracker: u32,
    ids: &[u64],
    now_ms: u64,
) -> Option<Reply> {
    let gifts: Vec<u32> = (ids.iter())
        .filter(|&&id| id & !0xFFFF_FFFF == super::CLAIM_TAG)
        .filter_map(|&id| u32::try_from(id & 0xFFFF_FFFF).ok())
        .collect();
    if gifts.is_empty() {
        // Character claims are gameplay's; the gifts' rows go back to the list after its 108/4.
        super::mark_resend(store, account);
        return None;
    }
    let Ok(realm) = Realm::load(store, game) else {
        let rows = game.gift_claim_rows();
        let text = "Gifts cannot be read right now; try again.";
        return Some(Reply::of(vec![claim_answer(tracker, false, &rows), crate::chat::system(text)]));
    };
    let rules = realm.rules();
    let path = box_path(store, account);
    let waiting = load(&path).map(|gifts| gifts.pending).unwrap_or_default();
    let refuse = |text: &str| {
        let rows = rows(game, &waiting, &rules, character);
        Some(Reply::of(vec![claim_answer(tracker, false, &rows), crate::chat::system(text)]))
    };
    if gifts.len() != ids.len() {
        return refuse("Claim gifts and other rewards one at a time.");
    }
    if game.mail_busy() {
        return refuse("Finish what you are doing before you claim a gift.");
    }
    let mut picked = Vec::with_capacity(gifts.len());
    for id in &gifts {
        match waiting.iter().find(|p| p.id == *id && rules.shows(p, character)) {
            Some(pending) => picked.push(pending.clone()),
            None => return refuse("That gift is no longer waiting."),
        }
    }
    let mut trial = game.clone();
    let mut packets = Vec::new();
    for pending in &picked {
        match trial.gift_grant(pending.coins, &pending.items, now_ms) {
            Ok(granted) => packets.extend(granted),
            Err(error) => return refuse(&format!("Cannot claim {}: {error}.", pending.name)),
        }
    }
    let claimed = match take(store, account, &gifts, character, &rules, unix_now()) {
        Ok(claimed) => claimed,
        Err(GiftError::Refused(text)) => return refuse(&format!("Cannot claim: {text}.")),
        Err(GiftError::Io(_)) => return refuse("Your gifts cannot be saved right now; try again."),
    };
    *game = trial;
    let left = load(&path).map(|gifts| gifts.pending).unwrap_or_default();
    let rows = rows(game, &left, &rules, character);
    let names: Vec<&str> = claimed.iter().map(|p| p.name.as_str()).collect();
    let mut out = vec![claim_answer(tracker, true, &rows)];
    out.extend(packets);
    out.push(crate::chat::system(&format!("You claimed {}.", names.join(", "))));
    Some(Reply {
        packets: out,
        changed: true,
    })
}

/// Item definitions for a tooltip or a bundle's contents, when every id is an item of a gift
/// waiting for this character; otherwise gameplay's own claims answer.
fn definitions(
    store: &Path,
    account: u64,
    game: &Gameplay,
    character: u64,
    tracker: u32,
    ids: &[i32],
) -> Option<Reply> {
    let realm = Realm::load(store, game).ok()?;
    let rules = realm.rules();
    let waiting = load(&box_path(store, account)).ok()?.pending;
    let mine = |id: &i32| {
        (waiting.iter())
            .filter(|pending| rules.shows(pending, character))
            .any(|pending| pending.items.iter().any(|(item, _)| item == id))
    };
    if !ids.iter().all(mine) {
        return None;
    }
    let unique: std::collections::BTreeSet<i32> = ids.iter().copied().collect();
    let mut blob = Vec::new();
    put_u32(&mut blob, u32::try_from(unique.len()).ok()?);
    for id in unique {
        blob.extend_from_slice(&game.gift_definition(id)?);
    }
    let mut packet = header(6);
    put_u32(&mut packet, tracker);
    packet.push(1);
    packet.extend(i32::try_from(blob.len()).ok()?.to_le_bytes());
    packet.extend(blob);
    Some(Reply::of(vec![packet]))
}

/// `realm redeem <code>`: an admin code (`redeem-codes.json`) first, then a catalog row's
/// original code. A code with items puts its gift in the Claim window; a coins-only admin code
/// pays at once, as L081's did.
fn redeem(
    store: &Path,
    account: u64,
    game: &mut Gameplay,
    character: u64,
    code: &str,
    now_ms: u64,
) -> Reply {
    let name = redeem_codes::normalize(code);
    match admin_code(store, account, game, character, &name, now_ms) {
        Some(reply) => reply,
        None => classic(store, account, game, character, &name),
    }
}

/// `None` when no admin code has this name.
fn admin_code(
    store: &Path,
    account: u64,
    game: &mut Gameplay,
    character: u64,
    name: &str,
    now_ms: u64,
) -> Option<Reply> {
    use crate::account_store::{AccountStoreLock, STAFF_LOCK_WAIT};
    let path = super::codes_path(store);
    let Ok(_lock) = AccountStoreLock::acquire_within(&path, STAFF_LOCK_WAIT) else {
        return Some(Reply::line("Codes are busy; try again in a moment."));
    };
    let mut codes = Codes::load(&path).ok()?;
    codes.codes.get(name)?;
    let member = game.gift_member();
    let today = redeem_codes::today();
    let (coins, items) = match codes.redeem_for(name, character, account, member, today) {
        Ok(paid) => paid,
        Err(text) => return Some(Reply::line(text)),
    };
    let note = codes.codes.get(name).map(|c| c.note.clone()).unwrap_or_default();
    if items.is_empty() {
        // A coins-only code pays now: a Claim window row shows an item.
        let mut trial = game.clone();
        let granted = match trial.gift_grant(coins, &[], now_ms) {
            Ok(granted) => granted,
            Err(error) => return Some(Reply::line(&format!("Cannot redeem that code: {error}."))),
        };
        if codes.save(&path).is_err() {
            return Some(Reply::line("Codes cannot be saved right now; try again."));
        }
        *game = trial;
        let mut packets = granted;
        packets.push(crate::chat::system(&format!("Redeemed code {name}.")));
        return Some(Reply {
            packets,
            changed: true,
        });
    }
    let parcel = super::Parcel {
        key: None,
        name: if note.is_empty() { format!("Code {name}") } else { note },
        items,
        coins,
        from: "Free Realms".into(),
        note: String::new(),
        via: super::Via::Code,
        character: None,
    };
    // The gift lands before the use is saved: a failed save gives the gift without spending
    // the use (it can be redeemed again), never a spent use without the gift.
    if let Err(error) = super::deliver(store, account, parcel) {
        return Some(Reply::line(&format!("Cannot redeem that code: {error}.")));
    }
    if codes.save(&path).is_err() {
        log("gift_code_save_failed", account, name);
    }
    Some(redeemed(store, account, game, character, name))
}

fn classic(store: &Path, account: u64, game: &Gameplay, character: u64, name: &str) -> Reply {
    let Ok(realm) = Realm::load(store, game) else {
        return Reply::line("That code is not valid.");
    };
    let Some(gift) = realm.catalog.by_code(name) else {
        return Reply::line("That code is not valid.");
    };
    let rules = realm.rules();
    match super::redeem_classic(store, account, character, gift, &rules) {
        Ok(_) => redeemed(store, account, game, character, name),
        Err(error) => Reply::line(&error.to_string()),
    }
}

/// The new list (the gift's row is in it) and the line.
fn redeemed(store: &Path, account: u64, game: &Gameplay, character: u64, name: &str) -> Reply {
    let mut reply = list(store, account, game, character);
    // This session shows it now; the tick need not tell it again.
    let _ = super::take_delivered(store);
    reply.packets.push(crate::chat::system(&format!(
        "Redeemed code {name}: your gift is waiting in the Claim window (Marketplace)."
    )));
    reply
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_parse_from_the_clients_writers() {
        // 0x00a045d0: header, tracker, then 0x00a04090's count and u64 ids.
        let mut request = header(3);
        request.extend(7u32.to_le_bytes());
        request.extend(2u32.to_le_bytes());
        request.extend((super::super::CLAIM_TAG | 1).to_le_bytes());
        request.extend((super::super::CLAIM_TAG | 2).to_le_bytes());
        let (tracker, ids) = claim_request(&request).unwrap();
        assert_eq!((tracker, ids.len()), (7, 2));
        assert!(claim_request(&request[..request.len() - 1]).is_none());
        // 0x00a04520: header, tracker, then 0x00a03fc0's count and i32 ids.
        let mut defs = header(5);
        defs.extend(9u32.to_le_bytes());
        defs.extend(1u32.to_le_bytes());
        defs.extend(70430i32.to_le_bytes());
        assert_eq!(definitions_request(&defs), Some((9, vec![70430])));
        assert_eq!(list_packet(&[]), [108, 0, 2, 0, 1, 0, 0, 0, 0]);
        assert_eq!(claim_answer(5, false, &[]), [108, 0, 4, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    /// Every shipped gift grants from the release content alone: each item has a definition the
    /// inventory loads (the captured rows, the item table and the retired rows of its
    /// supplement), a 108/6 can describe it, and mail's reward path takes the whole gift into a
    /// fresh bag.
    #[test]
    fn every_shipped_gift_grants_from_the_release_content() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let catalog = Catalog::load(&root.join("data/content")).unwrap();
        let game = Gameplay::load(&root.join("data/release"), None).unwrap();
        for gift in &catalog.gifts {
            let items: Vec<(i32, u32)> = gift.items.iter().map(|i| (i.item_id, i.count)).collect();
            assert!(game.mail_items_known(&items), "{}: {items:?}", gift.key);
            assert!(items.iter().all(|&(item, _)| game.gift_definition(item).is_some()), "{}", gift.key);
            let mut trial = game.clone();
            let granted = trial.gift_grant(gift.coins, &items, 1_000);
            assert!(granted.is_ok(), "{}: {granted:?}", gift.key);
        }
    }
}
