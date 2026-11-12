//! gifts-codes: the panel's Gifts & Codes routes over `osfr_tools::gifts::admin`. Staff
//! (`gifts`, moderators and admins) browse the catalog, send gifts and list codes; admins
//! (`gift_codes`) make and revoke codes and turn standing offers on or off. Every applied write
//! is one audit row (`gift_send`, `gift_code`, `gift_code_revoke`, `gift_offer`).
//!
//! ```text
//! GET  /admin/api/gifts?q=&page=          catalog rows (+ enabled_now) and the unresolved list
//! POST /admin/api/gifts/send               {"character_id", "key", "note"?} -> {"account", "gift_id", "key"}
//! GET  /admin/api/gifts/codes              [{"code", "enabled", "uses", "max_uses", "expires", "bound_account", ...}]
//! POST /admin/api/gifts/codes              {"keys": [key...], "uses"?: 1, "expires_days"?: n, "character_id"?: id, "note"?}
//!                                          -> {"code", "row"}
//! POST /admin/api/gifts/codes/revoke       {"code"} -> {"row"}
//! POST /admin/api/gifts/offers             {"key", "enabled"} -> {"key", "enabled", "was"}
//! ```
use super::*;
use osfr_tools::gifts::{self, Catalog, GiftError, Toggles, admin as gift_admin};

fn gift_error(failure: GiftError) -> Reply {
    match failure {
        GiftError::Refused(text) if text.contains("no such") => error(404, &text),
        GiftError::Refused(text) => error(422, &text),
        GiftError::Io(_) => error(503, "the gift store is busy or unavailable"),
    }
}

fn catalog(shared: &Shared) -> Result<Catalog, Reply> {
    let content = shared
        .content
        .as_deref()
        .ok_or_else(|| error(503, "this server has no content folder"))?;
    // The folder the sessions read it from (`Gameplay::gift_content_dir`).
    Catalog::load(&osfr_tools::content::npc_content(content))
        .map_err(|failure| error(500, &failure.to_string()))
}

fn gift<'a>(catalog: &'a Catalog, key: &str) -> Result<&'a gifts::Gift, Reply> {
    catalog.get(key).ok_or_else(|| error(404, "no such gift"))
}

fn write(shared: &Shared, caller: &Caller, action: &str, target: Option<u64>, before: Value, after: Value) -> Result<(), Reply> {
    record(shared, &audit::Row {
        time: unix_now(),
        staff: caller.staff(),
        action: action.into(),
        target: audit_target(&shared.store, target),
        before,
        after,
        reason: None,
        via: "panel".into(),
    })
}

pub(super) fn browse(shared: &Shared, caller: &Caller, request: &Request) -> Answer {
    caller.need("gifts")?;
    let catalog = catalog(shared)?;
    let page = page_param(request)?;
    let query = request.param("q").unwrap_or_default();
    let toggles = Toggles::load(&shared.store);
    Ok(json_reply(200, gift_admin::catalog_page(&catalog, &toggles, &query, page)))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SendBody {
    character_id: u64,
    key: String,
    #[serde(default)]
    note: String,
}

pub(super) fn send(shared: &Shared, caller: &Caller, request: &Request) -> Answer {
    caller.need("gifts")?;
    let body: SendBody = parse(request)?;
    let catalog = catalog(shared)?;
    let gift = gift(&catalog, &body.key)?;
    let (account, gift_id) = gift_admin::send(&shared.store, gift, body.character_id, &caller.name, &body.note)
        .map_err(gift_error)?;
    write(shared, caller, "gift_send", Some(body.character_id), Value::Null, json!({
        "key": gift.key, "name": gift.name, "items": gift_admin::row(gift, &Toggles::default())["items"],
        "account": account, "gift_id": gift_id, "note": body.note,
    }))?;
    Ok(json_reply(200, json!({"account": account, "gift_id": gift_id, "key": gift.key})))
}

pub(super) fn codes(shared: &Shared, caller: &Caller) -> Answer {
    caller.need("gifts")?;
    let rows = gift_admin::codes(&shared.store).map_err(gift_error)?;
    Ok(json_reply(200, json!(rows)))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CodeBody {
    keys: Vec<String>,
    #[serde(default)]
    uses: Option<u32>,
    #[serde(default)]
    expires_days: Option<u32>,
    #[serde(default)]
    character_id: Option<u64>,
    #[serde(default)]
    note: String,
}

pub(super) fn make_code(shared: &Shared, caller: &Caller, request: &Request) -> Answer {
    caller.need("gift_codes")?;
    let body: CodeBody = parse(request)?;
    if body.keys.is_empty() || body.keys.len() > gifts::MAX_GIFT_ITEMS {
        return Err(error(400, "keys names 1 to 8 catalog gifts"));
    }
    let catalog = catalog(shared)?;
    let picked = (body.keys.iter())
        .map(|key| gift(&catalog, key))
        .collect::<Result<Vec<_>, _>>()?;
    let new = gift_admin::NewCode {
        gifts: picked,
        uses: body.uses.unwrap_or(1),
        expires_days: body.expires_days,
        bound_character: body.character_id,
        note: body.note,
    };
    let (code, row) = gift_admin::make_code(&shared.store, &new).map_err(gift_error)?;
    let row = gift_admin::code_row(&code, &row);
    write(shared, caller, "gift_code", body.character_id, Value::Null, json!({"keys": body.keys, "row": row}))?;
    Ok(json_reply(200, json!({"code": code, "row": row})))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RevokeBody {
    code: String,
}

pub(super) fn revoke(shared: &Shared, caller: &Caller, request: &Request) -> Answer {
    caller.need("gift_codes")?;
    let body: RevokeBody = parse(request)?;
    let before = gift_admin::revoke(&shared.store, &body.code).map_err(gift_error)?;
    let mut after = before.clone();
    after["enabled"] = json!(false);
    write(shared, caller, "gift_code_revoke", None, before, after.clone())?;
    Ok(json_reply(200, json!({"row": after})))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OfferBody {
    key: String,
    enabled: bool,
}

pub(super) fn offer(shared: &Shared, caller: &Caller, request: &Request) -> Answer {
    caller.need("gift_codes")?;
    let body: OfferBody = parse(request)?;
    let catalog = catalog(shared)?;
    let gift = gift(&catalog, &body.key)?;
    let was = gift_admin::toggle(&shared.store, gift, body.enabled).map_err(gift_error)?;
    write(shared, caller, "gift_offer", None, json!({"key": gift.key, "enabled": was}), json!({"key": gift.key, "enabled": body.enabled}))?;
    Ok(json_reply(200, json!({"key": gift.key, "enabled": body.enabled, "was": was})))
}
