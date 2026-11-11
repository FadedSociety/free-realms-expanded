//! gm-panel integration: the panel routes G1 registered, each wired to the facade its builder
//! made, with one staff audit row (`admin::audit`) for every applied write.
//!
//! - **Character edits** (inventory, coins, quests, jobs, undo): G3's
//!   `gm_menu_routing::character_admin` on the world loop, with a `Host` whose `me` is `None`: an
//!   online character is edited through its own session, an offline save after a backup.
//! - **Items**: G3's item search, on this thread (it reads the item tables, not the world).
//! - **Mail**: G2's `mail::admin` with the gateway's own account store and G1's online list, so
//!   online recipients are told as the letters land (`mail_routing::tick`).
//! - **Moves and powers**: G5's facade checks the power, then `gm_powers_routing::run` carries it
//!   out through the staff member's own in-game session. With no character in the world, only
//!   the powers that act on another player alone run (`run_without_character`).
//! - **Jail and release**: `gm_powers_routing::jail`, to the spot in `moderation.json`.
//! - **Map**: the online players of a zone; the zone's map art from the client install.
//! - **Names**: job and quest names for the ids the other routes give.
use super::super::gm_menu_routing::character_admin::{self as edits, Host};
use super::super::gm_powers_routing::{self as powers, Actor};
use super::*;
use osfr_tools::character_admin::{
    self as admin_edits, AdminError, Amount, Change, CoinOp, Edit, EditRecord, JobSet, Pick,
    QuestOp, Removal,
};
use osfr_tools::gm_powers::{self, Power, WaypointOp, Who};
use osfr_tools::mail::{self, admin::OnlineCharacter, admin::To};

/// (R) The panel's whole-zone map picture, pixels on its longer side.
const MAP_SIDE: u32 = 2048;
/// (R) With no map art, the box around a zone's players reaches this far past them.
const MAP_MARGIN_M: f32 = 100.0;
/// (R) How long a request waits for a world job that edits, moves or loads a character (a debug
/// build under load has taken more than G1's 3 s); the job runs to its end either way.
const EDIT_WAIT: Duration = Duration::from_secs(30);

fn now_ms(server: &Server) -> u64 {
    u64::try_from(server.start.elapsed().as_millis()).unwrap_or(u64::MAX)
}

/// G3's host for one request on the world loop: no dispatching session.
fn host(server: &mut Server) -> Host<'_> {
    let now_ms = now_ms(server);
    Host {
        options: &server.options,
        sessions: &mut server.sessions,
        world: &mut server.world,
        membership: &mut server.membership,
        now_ms,
        me: None,
    }
}

fn editor(caller: &Caller) -> admin_edits::Actor {
    admin_edits::Actor {
        account_id: caller.account,
        role: caller.role,
    }
}

fn edit_error(failure: AdminError) -> Reply {
    let status = match failure {
        AdminError::Forbidden => 403,
        AdminError::NotFound(_) => 404,
        AdminError::Refused(_) => 422,
        AdminError::Conflict(_) => 409,
        AdminError::Failed(_) => 500,
    };
    error(status, &failure.to_string())
}

/// The item tables, loaded once from the content folder.
fn catalogue(shared: &Shared) -> Option<&osfr_tools::inventory::Inventory> {
    let content = shared.content.as_deref()?;
    (shared.catalogue)
        .get_or_init(|| osfr_tools::inventory::Inventory::load(content, None).ok())
        .as_ref()
}

/// Names for the item rows of an edit's audit row, so the log reads without the tables.
fn name_items(shared: &Shared, row: &mut audit::Row) {
    let Some(tables) = catalogue(shared) else {
        return;
    };
    for side in [&mut row.before, &mut row.after] {
        for item in side
            .get_mut("items")
            .and_then(Value::as_array_mut)
            .into_iter()
            .flatten()
        {
            if let Some(id) = item["item_id"]
                .as_i64()
                .and_then(|id| i32::try_from(id).ok())
            {
                item["name"] = json!(admin_edits::item_label(tables, id).0);
            }
        }
    }
}

fn store_target(shared: &Shared, guid: u64) -> audit::Target {
    audit_target(&shared.store, Some(guid))
}

pub(super) fn inventory(shared: &Shared, caller: &Caller, id: &str) -> Answer {
    caller.need("view")?;
    let guid = character_id(id)?;
    let actor = editor(caller);
    let view = world_within(shared, EDIT_WAIT, move |server| {
        edits::inventory(&host(server), &actor, guid)
    })?
    .map_err(edit_error)?;
    Ok(json_reply(200, json!(view)))
}

/// What an edit answers with, read after it: the inventory, the coins, the quests or the jobs.
#[derive(Clone, Copy)]
enum View {
    Inventory,
    Coins,
    Quests,
    Jobs,
}

fn view(
    host: &Host,
    actor: &admin_edits::Actor,
    guid: u64,
    kind: View,
) -> Result<Value, AdminError> {
    Ok(match kind {
        View::Inventory => json!(edits::inventory(host, actor, guid)?),
        View::Coins => json!({ "coins": edits::inventory(host, actor, guid)?.coins }),
        View::Quests => {
            let profile = edits::profile(host, actor, guid)?;
            json!({ "active": profile.quests_active, "done": profile.quests_done })
        }
        View::Jobs => json!({ "jobs": edits::profile(host, actor, guid)?.jobs }),
    })
}

/// One staff write (its permission checked by the caller): G3's edit where the character
/// lives, the answer read after it, and the audit row with its undo key.
fn edit(shared: &Shared, caller: &Caller, guid: u64, change: Edit, kind: View) -> Answer {
    let actor = editor(caller);
    let staged = change.clone();
    let (receipt, mut answer) = world_within(shared, EDIT_WAIT, move |server| {
        let mut host = host(server);
        let host = &mut host;
        let receipt = match staged {
            Edit::AddItems {
                item_id,
                count,
                equip,
            } => edits::add_items(host, &actor, guid, item_id, count, equip),
            Edit::RemoveItems(rows) => edits::remove_items(host, &actor, guid, rows),
            Edit::Coins(op, amount) => edits::coins(host, &actor, guid, op, amount),
            Edit::Quest(op, quest) => edits::quests(host, &actor, guid, op, quest),
            Edit::Job(job, set) => edits::jobs(host, &actor, guid, job, set),
            Edit::Apply(_) => Err(AdminError::Refused("an undo goes through its key".into())),
        }?;
        // The edit stands even if the read after it fails; the panel reloads what it shows.
        let answer = view(&host, &actor, guid, kind).unwrap_or_else(|_| json!({}));
        Ok::<_, AdminError>((receipt, answer))
    })?
    .map_err(edit_error)?;
    let mut row = edits::audit_row(
        caller.staff(),
        "panel",
        store_target(shared, guid),
        &change,
        &receipt,
    );
    name_items(shared, &mut row);
    record(shared, &row)?;
    answer["undo"] = row.after.get("undo").cloned().unwrap_or(Value::Null);
    answer["online"] = json!(receipt.online);
    answer["summary"] = json!(receipt.record.summary);
    Ok(json_reply(200, answer))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AddBody {
    item_id: i32,
    count: u32,
    #[serde(default)]
    equip: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RemoveRow {
    #[serde(default)]
    slot: Option<i32>,
    #[serde(default)]
    item_id: Option<i32>,
    count: Value,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RemoveBody {
    items: Vec<RemoveRow>,
}

fn removal(row: &RemoveRow) -> Result<Removal, Reply> {
    let pick = match (row.slot, row.item_id) {
        (Some(slot), None) => Pick::Slot(slot),
        (None, Some(item)) => Pick::Item(item),
        _ => return Err(error(400, "each row names a slot or an item_id")),
    };
    let amount = match &row.count {
        Value::String(all) if all == "all" => Amount::All,
        count => Amount::Count(
            count
                .as_u64()
                .and_then(|count| u32::try_from(count).ok())
                .ok_or_else(|| error(400, "count is a number or \"all\""))?,
        ),
    };
    Ok(Removal { pick, amount })
}

pub(super) fn inventory_edit(
    shared: &Shared,
    caller: &Caller,
    request: &Request,
    id: &str,
    op: &str,
) -> Answer {
    caller.need("inventory")?;
    let guid = character_id(id)?;
    let change = if op == "add" {
        let body: AddBody = parse(request)?;
        Edit::AddItems {
            item_id: body.item_id,
            count: body.count,
            equip: body.equip,
        }
    } else {
        let body: RemoveBody = parse(request)?;
        Edit::RemoveItems(body.items.iter().map(removal).collect::<Result<_, _>>()?)
    };
    edit(shared, caller, guid, change, View::Inventory)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CoinBody {
    op: String,
    amount: u32,
}

pub(super) fn coins(shared: &Shared, caller: &Caller, request: &Request, id: &str) -> Answer {
    caller.need("coins")?;
    let guid = character_id(id)?;
    let body: CoinBody = parse(request)?;
    let op = match body.op.as_str() {
        "give" => CoinOp::Give,
        "take" => CoinOp::Take,
        "set" => CoinOp::Set,
        _ => return Err(error(400, "op is give, take or set")),
    };
    edit(
        shared,
        caller,
        guid,
        Edit::Coins(op, body.amount),
        View::Coins,
    )
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct QuestBody {
    op: String,
    quest_id: u32,
}

pub(super) fn quests(shared: &Shared, caller: &Caller, request: &Request, id: &str) -> Answer {
    caller.need("quests")?;
    let guid = character_id(id)?;
    let body: QuestBody = parse(request)?;
    let op = match body.op.as_str() {
        "grant" => QuestOp::Grant,
        "complete" => QuestOp::Complete,
        "reset" => QuestOp::Reset,
        "abandon" => QuestOp::Abandon,
        _ => return Err(error(400, "op is grant, complete, reset or abandon")),
    };
    edit(
        shared,
        caller,
        guid,
        Edit::Quest(op, body.quest_id),
        View::Quests,
    )
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct JobBody {
    job_id: i32,
    #[serde(default)]
    level: Option<u32>,
    #[serde(default)]
    xp: Option<u32>,
    /// `learn` gives the job as a trainer does (`active` also switches to it); `activate`
    /// switches to a job the character knows, as the job list does.
    #[serde(default)]
    op: Option<String>,
    #[serde(default)]
    active: bool,
}

pub(super) fn jobs(shared: &Shared, caller: &Caller, request: &Request, id: &str) -> Answer {
    caller.need("jobs")?;
    let guid = character_id(id)?;
    let body: JobBody = parse(request)?;
    let set = match (body.op.as_deref(), body.level, body.xp) {
        (Some("learn"), None, None) => JobSet::Learn {
            active: body.active,
        },
        (Some("activate"), None, None) => JobSet::Activate,
        (Some(_), ..) => return Err(error(400, "op is learn or activate, without a level or xp")),
        (None, Some(level), None) => JobSet::Level(level),
        (None, None, Some(xp)) => JobSet::Xp(xp),
        _ => return Err(error(400, "give a level or an xp, not both")),
    };
    edit(
        shared,
        caller,
        guid,
        Edit::Job(body.job_id, set),
        View::Jobs,
    )
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UndoBody {
    undo: String,
}

/// Reverse one edit by the undo key its audit row holds (a panel or chat edit). Refused when the
/// key was used already, or (by G3) when the character moved on since; the undo is itself a row
/// with its own key.
pub(super) fn undo(shared: &Shared, caller: &Caller, request: &Request) -> Answer {
    let body: UndoBody = parse(request)?;
    let key = body.undo;
    if key.len() != 32 || !key.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(error(400, "undo is the key an edit answered with"));
    }
    let (mut found, mut used) = (None, false);
    audit::scan(&shared.audit, |row| {
        let named =
            |field: &str| row.after.get(field).and_then(Value::as_str) == Some(key.as_str());
        used = named("undid");
        if !used && named("undo") {
            found = Some(row);
        }
        !used && found.is_none()
    })
    .map_err(|_| error(500, "the audit log could not be read"))?;
    if used {
        return Err(error(409, "that edit was undone already"));
    }
    let row = found.ok_or_else(|| error(404, "no edit has that undo key"))?;
    let permission = match row.action.as_str() {
        "coins" => "coins",
        "quests" => "quests",
        "jobs" => "jobs",
        _ => "inventory",
    };
    caller.need(permission)?;
    let change: Change = serde_json::from_value(row.after["change"].clone())
        .map_err(|_| error(500, "that audit row's change is unreadable"))?;
    let guid = row
        .target
        .character_id
        .ok_or_else(|| error(500, "that audit row names no character"))?;
    let undone = EditRecord {
        target: guid,
        actor_account: caller.account,
        at_unix: row.time,
        summary: row.after["summary"].as_str().unwrap_or_default().to_owned(),
        change,
    };
    let actor = editor(caller);
    let inverse = Edit::Apply(undone.change.inverse());
    let (receipt, mut answer) = world_within(shared, EDIT_WAIT, move |server| {
        let mut host = host(server);
        let receipt = edits::undo(&mut host, &actor, &undone)?;
        let answer = view(&host, &actor, guid, View::Inventory).unwrap_or_else(|_| json!({}));
        Ok::<_, AdminError>((receipt, answer))
    })?
    .map_err(edit_error)?;
    let mut row = edits::audit_row(
        caller.staff(),
        "panel",
        store_target(shared, guid),
        &inverse,
        &receipt,
    );
    row.after["undid"] = json!(key);
    name_items(shared, &mut row);
    record(shared, &row)?;
    answer["undo"] = row.after.get("undo").cloned().unwrap_or(Value::Null);
    answer["online"] = json!(receipt.online);
    answer["character_id"] = json!(guid);
    Ok(json_reply(200, answer))
}

pub(super) fn items(shared: &Shared, caller: &Caller, request: &Request) -> Answer {
    caller.need("view")?;
    let tables = catalogue(shared).ok_or_else(|| error(503, "this server has no item tables"))?;
    let page = page_param(request)?;
    let query = request.param("q").unwrap_or_default();
    let found = admin_edits::search_in(tables, &query, page - 1)
        .map_err(|failure| error(400, &failure.to_string()))?;
    Ok(json_reply(200, json!(found.rows)))
}

pub(super) fn names(shared: &Shared, caller: &Caller) -> Answer {
    caller.need("view")?;
    let content = shared
        .content
        .as_deref()
        .ok_or_else(|| error(503, "this server has no content folder"))?;
    Ok(json_reply(200, admin::names::names(content)))
}

/// A letter's attachments as the panel reads and sends them.
fn attachments(items: &[(i32, u32)], coins: u32) -> Value {
    let items: Vec<Value> = items
        .iter()
        .map(|(item_id, count)| json!({"item_id": item_id, "count": count}))
        .collect();
    json!({"items": items, "coins": coins})
}

pub(super) fn mailbox(shared: &Shared, caller: &Caller, id: &str) -> Answer {
    caller.need("view")?;
    let guid = character_id(id)?;
    if load_store(shared)?.character(guid).is_none() {
        return Err(error(404, "no such character"));
    }
    let letters = mail::admin::list_for(&shared.store, guid)
        .map_err(|_| error(500, "that mailbox could not be read"))?;
    let rows: Vec<Value> = letters
        .iter()
        .map(|letter| {
            let mut row = json!(letter);
            row["attachments"] = attachments(&letter.attachments.items, letter.attachments.coins);
            row
        })
        .collect();
    Ok(json_reply(200, json!(rows)))
}

pub(super) fn mail_jobs(caller: &Caller) -> Answer {
    caller.need("mail")?;
    Ok(json_reply(200, json!(mail::admin::jobs())))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ItemCount {
    item_id: i32,
    count: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AttachBody {
    #[serde(default)]
    items: Vec<ItemCount>,
    #[serde(default)]
    coins: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MailBody {
    to: Value,
    subject: String,
    body: String,
    #[serde(default)]
    attachments: Option<AttachBody>,
}

fn recipients(to: &Value) -> Option<To> {
    let flag = |name| to.get(name).and_then(Value::as_bool) == Some(true);
    if let Some(ids) = to.get("characters") {
        let ids = ids
            .as_array()?
            .iter()
            .map(Value::as_u64)
            .collect::<Option<Vec<u64>>>()?;
        return (!ids.is_empty() && ids.len() <= 1000).then_some(To::Characters(ids));
    }
    if flag("everyone") {
        return Some(To::Everyone);
    }
    if flag("online") {
        return Some(To::Online);
    }
    if let Some(zone) = to.get("zone") {
        return Some(To::Zone(u32::try_from(zone.as_u64()?).ok()?));
    }
    AccountRole::parse(to.get("role")?.as_str()?).map(To::Role)
}

/// Send a letter from the panel: G2's facade with the gateway's account store and the online
/// list, so the world tick tells online recipients. Attachments make items and coins, so they
/// need those permissions too (a moderator mails, but cannot write items or coins).
pub(super) fn send_mail(shared: &Shared, caller: &Caller, request: &Request) -> Answer {
    caller.need("mail")?;
    let body: MailBody = parse(request)?;
    let to = recipients(&body.to).ok_or_else(|| {
        error(400, "to is {\"characters\": [ids]}, {\"everyone\": true}, {\"online\": true}, {\"role\": name} or {\"zone\": id}")
    })?;
    let attached = body.attachments.unwrap_or(AttachBody {
        items: Vec::new(),
        coins: 0,
    });
    if !attached.items.is_empty() {
        caller.need("inventory")?;
    }
    if attached.coins > 0 {
        caller.need("coins")?;
    }
    let letter = mail::Letter {
        from: "Free Realms".into(),
        subject: body.subject,
        body: body.body,
        attachments: mail::Attachments {
            items: attached
                .items
                .iter()
                .map(|row| (row.item_id, row.count))
                .collect(),
            coins: attached.coins,
        },
    };
    let tables = match letter.attachments.items.is_empty() {
        true => None,
        false => {
            Some(catalogue(shared).ok_or_else(|| error(503, "this server has no item tables"))?)
        }
    };
    let online = world(shared, |server| {
        online_now(server)
            .into_iter()
            .map(|row| OnlineCharacter {
                guid: row.character_id,
                zone: row.zone.unwrap_or(u32::MAX),
                role: row
                    .role
                    .and_then(AccountRole::parse)
                    .unwrap_or(AccountRole::Player),
            })
            .collect::<Vec<_>>()
    })?;
    let item_ok = |item: i32, count: u32| {
        count > 0 && tables.is_some_and(|tables| tables.has_definition(item))
    };
    let receipt = mail::admin::send(
        &shared.store,
        &caller.staff(),
        "panel",
        &to,
        letter,
        &online,
        &item_ok,
    )
    .map_err(|failure| match failure {
        mail::MailError::Refused(text) => error(422, &text),
        mail::MailError::Io(_) => error(500, "mail storage failed"),
    })?;
    Ok(json_reply(200, json!(receipt)))
}

/// The staff member's own selected session, if a character of that account is in the world.
fn staff_session(sessions: &SessionMap, account: u64) -> Option<SessionKey> {
    sessions
        .iter()
        .find(|(_, session)| session.selected_character && session.account_guid == account)
        .map(|(key, _)| *key)
}

fn panel_actor(caller: &Caller) -> Actor {
    Actor {
        account: caller.account,
        character: 0,
        staff: caller.staff(),
        via: "panel",
    }
}

/// Carry out an authorized power: through the staff member's own session when one is in the
/// world (it is told in chat too), else for the powers that need none. The reply or refusal, and
/// the staff character it ran as.
fn run_power(
    server: &mut Server,
    mut actor: Actor,
    role: AccountRole,
    power: Power,
) -> (Result<String, String>, Option<u64>) {
    let now_ms = now_ms(server);
    let Some(key) = staff_session(&server.sessions, actor.account) else {
        let done = powers::run_without_character(
            &actor,
            role,
            &server.options,
            &mut server.sessions,
            &mut server.world,
            &mut server.membership,
            now_ms,
            power,
        );
        return (done, None);
    };
    let Some(mut staff) = server.sessions.remove(&key) else {
        return (Err("your session is gone".into()), None);
    };
    actor.character = staff.character_guid;
    let done = powers::run(
        &mut staff,
        &server.options,
        &mut server.sessions,
        &mut server.world,
        &mut server.membership,
        now_ms,
        power,
        &actor,
    );
    let me = staff.character_guid;
    server.sessions.insert(key, staff);
    let done =
        done.unwrap_or_else(|failure| Err(format!("your session could not be told: {failure}")));
    (done, Some(me))
}

fn refused(text: &str) -> Reply {
    let status = if text == gm_powers::NOT_ALLOWED {
        403
    } else {
        409
    };
    error(status, text)
}

fn online_session(server: &Server, guid: u64) -> Option<&Session> {
    (server.sessions.values())
        .find(|session| session.selected_character && session.character_guid == guid)
}

/// Where a character stands now, if online: `{zone, zone_name, position}`.
fn location(server: &Server, guid: u64) -> Option<Value> {
    let session = online_session(server, guid)?;
    let state = session.gateway.state();
    let zone_name = session.gateway.runtime_zone().ok().map(|zone| zone.name);
    Some(json!({"zone": state.zone_id, "zone_name": zone_name, "position": state.position}))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MoveBody {
    op: String,
    #[serde(default)]
    to: Option<Value>,
}

/// Where a `send` goes: `{"position": [x, y, z]}` in the open world (`zone` 0 if given), or
/// `{"character_id": id}`, beside an online player.
enum Destination {
    At([f32; 3]),
    Beside(u64),
}

fn destination(to: Option<&Value>) -> Result<Destination, Reply> {
    let bad = || {
        error(
            400,
            "to is {\"zone\": 0, \"position\": [x, y, z]} or {\"character_id\": id}",
        )
    };
    let to = to.ok_or_else(bad)?;
    if let Some(guid) = to.get("character_id") {
        return guid.as_u64().map(Destination::Beside).ok_or_else(bad);
    }
    if to.get("zone").is_some_and(|zone| zone.as_u64() != Some(0)) {
        return Err(error(
            422,
            "send moves a player within the open world (zone 0) only",
        ));
    }
    let at = to
        .get("position")
        .and_then(Value::as_array)
        .ok_or_else(bad)?;
    let mut spot = [0f32; 3];
    if at.len() != 3 {
        return Err(bad());
    }
    for (slot, value) in spot.iter_mut().zip(at) {
        *slot = value.as_f64().ok_or_else(bad)? as f32;
    }
    Ok(Destination::At(spot))
}

/// Goto and bring move the staff member's own character or to it; send moves the player alone.
pub(super) fn move_player(shared: &Shared, caller: &Caller, request: &Request, id: &str) -> Answer {
    caller.need("move")?;
    let guid = character_id(id)?;
    let body: MoveBody = parse(request)?;
    let (role, actor) = (caller.role, panel_actor(caller));
    let op = body.op;
    let destination = match op.as_str() {
        "goto" | "bring" => None,
        "send" => Some(destination(body.to.as_ref())?),
        _ => return Err(error(400, "op is goto, bring or send")),
    };
    let (done, at) = world_within(shared, EDIT_WAIT, move |server| {
        let power = match (op.as_str(), destination) {
            ("goto", _) => gm_powers::goto(role, Who::Guid(guid)),
            ("bring", _) => gm_powers::bring(role, Who::Guid(guid)),
            (_, Some(Destination::Beside(other))) => match online_session(server, other) {
                Some(there) => {
                    gm_powers::send(role, Who::Guid(guid), there.gateway.state().position)
                }
                None => Err("that player is not online"),
            },
            (_, Some(Destination::At(position))) => {
                gm_powers::send(role, Who::Guid(guid), position)
            }
            _ => Err("op is goto, bring or send"),
        };
        let power = match power {
            Ok(power) => power,
            Err(text) => return (Err(text.to_owned()), None),
        };
        let (done, me) = run_power(server, actor, role, power);
        let moved = if op == "goto" { me } else { Some(guid) };
        (done, moved.and_then(|guid| location(server, guid)))
    })?;
    let message = done.map_err(|text| refused(&text))?;
    let mut answer = at.unwrap_or_else(|| json!({"zone": 0, "zone_name": null, "position": null}));
    answer["message"] = json!(message);
    Ok(json_reply(200, answer))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PowerBody {
    op: String,
    #[serde(default)]
    target: Option<u64>,
    #[serde(default)]
    metres: Option<f32>,
    #[serde(default)]
    label: Option<String>,
}

fn power_for(role: AccountRole, body: &PowerBody) -> Result<Power, Reply> {
    let target = || {
        body.target
            .map(Who::Guid)
            .ok_or_else(|| error(400, "this power needs a target"))
    };
    let label = || {
        body.label
            .clone()
            .ok_or_else(|| error(400, "this power needs a label"))
    };
    let power = match body.op.as_str() {
        "spectate" => gm_powers::spectate(role, target()?),
        "unspectate" => gm_powers::unspectate(role),
        "invisible" => gm_powers::invisible(role, true),
        "visible" => gm_powers::invisible(role, false),
        "freeze" => gm_powers::freeze(role, target()?, true),
        "unfreeze" => gm_powers::freeze(role, target()?, false),
        "follow" => gm_powers::follow(
            role,
            target()?,
            body.metres.unwrap_or(gm_powers::FOLLOW_DEFAULT_M),
        ),
        "unfollow" => gm_powers::authorize(role, Power::Unfollow),
        "fly" => gm_powers::fly(role, true),
        "land" => gm_powers::fly(role, false),
        "waypoint_save" => gm_powers::waypoint(role, WaypointOp::Save(label()?)),
        "waypoint_go" => gm_powers::waypoint(role, WaypointOp::Go(label()?)),
        "waypoint_delete" => gm_powers::waypoint(role, WaypointOp::Delete(label()?)),
        "waypoint_list" => gm_powers::waypoint(role, WaypointOp::List),
        _ => {
            return Err(error(
                400,
                "op is spectate, unspectate, invisible, visible, freeze, unfreeze, follow, unfollow, fly, land or waypoint_save|go|delete|list",
            ));
        }
    };
    power.map_err(refused)
}

/// The powers' state for the staff member: its own character's (when in the world) and every
/// frozen or jailed player.
fn power_state(server: &Server, account: u64) -> Value {
    let me = staff_session(&server.sessions, account)
        .and_then(|key| server.sessions.get(&key))
        .map(|session| session.character_guid);
    powers::state(&server.world, me)
}

pub(super) fn powers_now(shared: &Shared, caller: &Caller) -> Answer {
    caller.need("view")?;
    let account = caller.account;
    let state = world(shared, move |server| power_state(server, account))?;
    Ok(json_reply(200, json!({ "state": state })))
}

pub(super) fn power(shared: &Shared, caller: &Caller, request: &Request) -> Answer {
    caller.need("powers")?;
    let body: PowerBody = parse(request)?;
    let power = power_for(caller.role, &body)?;
    let (role, actor, account) = (caller.role, panel_actor(caller), caller.account);
    let (done, state) = world_within(shared, EDIT_WAIT, move |server| {
        let (done, _) = run_power(server, actor, role, power);
        (done, power_state(server, account))
    })?;
    let message = done.map_err(|text| refused(&text))?;
    Ok(json_reply(
        200,
        json!({ "state": state, "message": message }),
    ))
}

/// Jail or release for the panel's moderation: the role table and hierarchy as for a sanction,
/// then G5's hold. The audit row for the caller to record.
pub(super) fn jail(
    shared: &Shared,
    caller: &Caller,
    guid: u64,
    on: bool,
    reason: Option<String>,
) -> Result<audit::Row, Reply> {
    let op = if on { "jail" } else { "release" };
    let store = load_store(shared)?;
    admin::moderation::may_act(&store, caller.account, guid, op).map_err(refusal)?;
    let spot = admin::moderation::jail_spot()
        .ok_or_else(|| error(503, "this server has no jail spot (moderation.json)"))?;
    let account = caller.account;
    let message = world_within(shared, EDIT_WAIT, move |server| {
        let now_ms = now_ms(server);
        powers::jail(
            account,
            &server.options,
            &mut server.sessions,
            &mut server.world,
            &mut server.membership,
            now_ms,
            guid,
            on,
            spot,
        )
    })?
    .map_err(|text| error(409, &text))?;
    Ok(audit::Row {
        time: unix_now(),
        staff: caller.staff(),
        action: op.into(),
        target: target(&store, guid).unwrap_or_default(),
        before: json!({ "jailed": !on }),
        after: json!({ "jailed": on, "result": message }),
        reason,
        via: "panel".into(),
    })
}

/// One zone for the live map: its online players, its map art's URL and the world box the art
/// (or, without art, the players) covers, `[min_x, min_z, max_x, max_z]`, north (+x) up.
pub(super) fn map(shared: &Shared, caller: &Caller, zone: &str) -> Answer {
    caller.need("view")?;
    let zone: u32 = zone
        .parse()
        .map_err(|_| error(400, "the zone is a number"))?;
    let (players, name) = world(shared, move |server| {
        let players: Vec<Value> = online_rows(&server.world)
            .into_iter()
            .filter(|row| row.zone == Some(zone))
            .filter_map(|row| Some(json!({"character_id": row.character_id, "name": row.name, "position": row.position?})))
            .collect();
        let name = (server.sessions.values())
            .filter(|session| session.selected_character && session.gateway.state().zone_id == zone)
            .find_map(|session| session.gateway.runtime_zone().ok())
            .or_else(|| {
                server
                    .template
                    .as_deref()
                    .and_then(|gateway| gateway.runtime_zone().ok())
            })
            .filter(|found| found.id == zone)
            .map(|found| found.name);
        (players, name)
    })?;
    let art = match (&shared.icons, &name) {
        (Some(icons), Some(name)) => icons.zone_map(name, MAP_SIDE).map(|(_, bounds)| bounds),
        _ => None,
    };
    let image = art
        .and(name.as_ref())
        .map(|name| format!("/admin/map-image/{name}"));
    let bounds = art.or_else(|| {
        let at = |axis: usize| {
            players
                .iter()
                .filter_map(move |p| p["position"][axis].as_f64())
                .map(|v| v as f32)
        };
        let (xs, zs): (Vec<f32>, Vec<f32>) = (at(0).collect(), at(2).collect());
        let low = |v: &[f32]| v.iter().copied().reduce(f32::min);
        let high = |v: &[f32]| v.iter().copied().reduce(f32::max);
        Some([
            low(&xs)? - MAP_MARGIN_M,
            low(&zs)? - MAP_MARGIN_M,
            high(&xs)? + MAP_MARGIN_M,
            high(&zs)? + MAP_MARGIN_M,
        ])
    });
    Ok(json_reply(
        200,
        json!({
            "zone": zone, "zone_name": name, "image": image, "bounds": bounds, "players": players,
        }),
    ))
}

/// `GET /admin/map-image/<zone name>`: the zone's whole map, from the client install, cached.
pub(super) fn map_image(shared: &Shared, request: &Request, name: &str) -> Answer {
    if request.method != "GET" {
        return Err(error(405, "map pictures are GET only"));
    }
    authenticate(shared, request)?.need("view")?;
    let (png, _) = shared
        .icons
        .as_ref()
        .and_then(|icons| icons.zone_map(name, MAP_SIDE))
        .ok_or_else(|| error(404, "no map for that zone in the client install"))?;
    Ok(Reply {
        status: 200,
        kind: "image/png",
        body: png.to_vec(),
        cache: "private, max-age=86400",
        headers: Vec::new(),
    })
}
