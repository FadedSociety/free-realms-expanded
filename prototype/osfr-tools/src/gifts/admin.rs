//! The GM panel's Gifts & Codes tab (`osfr_gateway/admin_gifts.rs` serves it): browse the
//! catalog, send a gift straight into a player's Claim window, make a code for an event's
//! winner (one use or many, an expiry, bound to one player or not), list and revoke codes, and
//! turn standing offers on or off. Plain types in and out; the routes check roles and write the
//! audit rows.
use super::{Catalog, Gift, GiftError, Parcel, Toggles, Via, codes_path, deliver, refused};
use crate::account_store::{AccountStore, AccountStoreLock, STAFF_LOCK_WAIT};
use crate::redeem_codes::{self, Code, Codes};
use serde_json::{Value, json};
use std::path::Path;

/// Rows of one catalog page.
pub const PAGE: usize = 50;
/// (R) The most uses one panel code may allow, and the longest expiry, in days.
pub const MAX_USES: u32 = 10_000;
pub const MAX_EXPIRY_DAYS: u32 = 3_650;

/// One catalog row as the panel shows it: the row, and whether it is on now (the toggle).
pub fn row(gift: &Gift, toggles: &Toggles) -> Value {
    let mut value = json!(gift);
    value["enabled_now"] = json!(toggles.enabled(gift));
    value
}

/// Rows whose name, key or code contains `query` (any case), a page at a time (1-based).
pub fn catalog_page(catalog: &Catalog, toggles: &Toggles, query: &str, page: usize) -> Value {
    let query = query.trim().to_lowercase();
    let found: Vec<&Gift> = (catalog.gifts.iter())
        .filter(|gift| {
            query.is_empty()
                || [Some(gift.name.as_str()), Some(gift.key.as_str()), gift.code.as_deref()]
                    .into_iter()
                    .flatten()
                    .any(|text| text.to_lowercase().contains(&query))
        })
        .collect();
    let rows: Vec<Value> = (found.iter())
        .skip(page.saturating_sub(1).saturating_mul(PAGE))
        .take(PAGE)
        .map(|gift| row(gift, toggles))
        .collect();
    json!({
        "total": found.len(), "page": page, "page_size": PAGE, "rows": rows,
        "unresolved": catalog.unresolved,
    })
}

/// The account that owns `character`, from the store.
pub fn account_of(store: &Path, character: u64) -> Result<u64, GiftError> {
    let accounts = AccountStore::load(store)?;
    accounts
        .character(character)
        .map(|record| record.owner_account_id)
        .ok_or_else(|| refused("no such character"))
}

/// Send catalog row `key` to `character`'s account (a per-character row to that character
/// only), with an optional staff note. Online or not, it waits in the Claim window; the tick
/// tells an online player. Returns `(account, gift id)`.
pub fn send(
    store: &Path,
    gift: &Gift,
    character: u64,
    from: &str,
    note: &str,
) -> Result<(u64, u32), GiftError> {
    let note = clean_note(note)?;
    let account = account_of(store, character)?;
    let mut parcel = Parcel::of(gift, from, Via::Staff, character);
    parcel.note = note;
    Ok((account, deliver(store, account, parcel)?))
}

fn clean_note(note: &str) -> Result<String, GiftError> {
    let note = note.trim();
    if note.chars().count() > super::MAX_NOTE_CHARS || note.chars().any(char::is_control) {
        return Err(refused(format!(
            "a note is at most {} characters with no control characters",
            super::MAX_NOTE_CHARS
        )));
    }
    Ok(note.to_owned())
}

/// A code to make: a catalog row's items or the items of several rows (a bundle).
pub struct NewCode<'a> {
    pub gifts: Vec<&'a Gift>,
    /// Total redemptions; 1 for an event winner's code.
    pub uses: u32,
    /// Days from today it still works (inclusive); `None` for no expiry.
    pub expires_days: Option<u32>,
    /// The one character whose account may redeem it.
    pub bound_character: Option<u64>,
    pub note: String,
}

/// Make a fresh random code; returns it (`FR-XXXX-XXXX`) and the stored row.
pub fn make_code(store: &Path, new: &NewCode) -> Result<(String, Code), GiftError> {
    if new.gifts.is_empty() {
        return Err(refused("a code gives at least one catalog gift"));
    }
    if new.uses == 0 || new.uses > MAX_USES {
        return Err(refused(format!("uses is 1 to {MAX_USES}")));
    }
    if new.expires_days.is_some_and(|days| days > MAX_EXPIRY_DAYS) {
        return Err(refused(format!("expires_days is at most {MAX_EXPIRY_DAYS}")));
    }
    let mut items: Vec<(i32, u32)> = Vec::new();
    let coins = (new.gifts.iter())
        .fold(0u32, |sum, gift| sum.saturating_add(gift.coins))
        .min(super::MAX_GIFT_COINS);
    for gift in &new.gifts {
        for (item, count) in gift.items() {
            match items.iter_mut().find(|(id, _)| *id == item) {
                Some(row) => row.1 = row.1.saturating_add(count),
                None => items.push((item, count)),
            }
        }
    }
    if items.len() > redeem_codes::MAX_ITEMS {
        return Err(refused(format!("a code holds at most {} items", redeem_codes::MAX_ITEMS)));
    }
    let bound = match new.bound_character {
        Some(character) => Some(account_of(store, character)?),
        None => None,
    };
    let names: Vec<&str> = new.gifts.iter().map(|gift| gift.name.as_str()).collect();
    let note = match clean_note(&new.note)? {
        note if note.is_empty() => names.join(", "),
        note => note,
    };
    let path = codes_path(store);
    let _lock = AccountStoreLock::acquire_within(&path, STAFF_LOCK_WAIT)?;
    let mut codes = Codes::load(&path)?;
    let today = redeem_codes::today();
    let name = (0..8)
        .map(|_| redeem_codes::random_code())
        .find(|name| !codes.codes.contains_key(name))
        .ok_or_else(|| refused("no fresh code could be drawn; try again"))?;
    let code = Code {
        coins,
        items,
        max_uses: new.uses,
        // Each character redeems once; `uses` bounds the code as a whole.
        per_character: 1,
        expires: new.expires_days.map(|days| today + i64::from(days)),
        enabled: true,
        note: note.chars().take(200).collect(),
        bound,
        ..Code::default()
    };
    codes.codes.insert(name.clone(), code.clone());
    codes.save(&path)?;
    Ok((name, code))
}

/// One code as the panel lists it.
pub fn code_row(name: &str, code: &Code) -> Value {
    json!({
        "code": name, "enabled": code.enabled, "uses": code.uses, "max_uses": code.max_uses,
        "per_character": code.per_character, "expires": code.expires.map(redeem_codes::date),
        "bound_account": code.bound, "note": code.note, "coins": code.coins,
        "items": code.items.iter().map(|(item_id, count)| json!({"item_id": item_id, "count": count})).collect::<Vec<_>>(),
        "redeemed_by": code.redeemed.len(),
    })
}

/// Every admin code, newest names last (the file's order is by name).
pub fn codes(store: &Path) -> Result<Vec<Value>, GiftError> {
    let codes = Codes::load(&codes_path(store))?;
    Ok(codes.codes.iter().map(|(name, code)| code_row(name, code)).collect())
}

/// Turn an admin code off (kept, with its uses, for the record). Returns the row before.
pub fn revoke(store: &Path, name: &str) -> Result<Value, GiftError> {
    let path = codes_path(store);
    let _lock = AccountStoreLock::acquire_within(&path, STAFF_LOCK_WAIT)?;
    let mut codes = Codes::load(&path)?;
    let name = redeem_codes::normalize(name);
    let code = codes.codes.get_mut(&name).ok_or_else(|| refused("no such code"))?;
    let before = code_row(&name, code);
    if !code.enabled {
        return Err(refused("that code is already revoked"));
    }
    code.enabled = false;
    codes.save(&path)?;
    Ok(before)
}

/// Turn standing offer (or classic code) `gift` on or off; returns whether it was on.
pub fn toggle(store: &Path, gift: &Gift, enabled: bool) -> Result<bool, GiftError> {
    Ok(Toggles::set(store, gift, enabled)?)
}

#[cfg(test)]
mod tests {
    use super::super::{GiftItem, Per, Source};
    use super::*;

    fn gift(key: &str, item: i32) -> Gift {
        Gift {
            key: key.into(),
            name: key.into(),
            items: vec![GiftItem { item_id: item, count: 1, name_id: 1 }],
            source: Source::Promotional,
            code: None,
            per: Per::Account,
            repeatable: false,
            repeat_hours: None,
            live_event: None,
            tradeable: false,
            enabled: true,
            coins: 0,
            stock_per_event: None,
            per_player: None,
            stock_group: None,
            note: String::new(),
            evidence: String::new(),
        }
    }

    #[test]
    fn a_made_code_bundles_rows_and_a_revoked_one_stays_listed() {
        let dir = std::env::temp_dir().join(format!("osfr-gift-admin-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let store = dir.join("accounts.json");
        let (a, b) = (gift("a", 10), gift("b", 11));
        let new = NewCode {
            gifts: vec![&a, &b, &a],
            uses: 1,
            expires_days: Some(7),
            bound_character: None,
            note: String::new(),
        };
        let (name, code) = make_code(&store, &new).unwrap();
        assert!(name.starts_with("FR-") && code.items == [(10, 2), (11, 1)]);
        assert_eq!((code.max_uses, code.note.as_str()), (1, "a, b, a"));
        assert!(make_code(&store, &NewCode { uses: 0, ..new }).is_err());
        revoke(&store, &name.to_lowercase()).unwrap();
        assert!(revoke(&store, &name).is_err(), "revoked once");
        let listed = codes(&store).unwrap();
        assert_eq!((listed.len(), listed[0]["enabled"].as_bool()), (1, Some(false)));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
