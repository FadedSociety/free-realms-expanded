//! Shared setup for the gateway binary's `#[cfg(test)]` session modules: one scratch
//! directory, one `Options` initializer and one socket-less session fixture, so a new
//! option field or handshake step is added once instead of in every test copy. Test-only;
//! production never includes this file.
use super::*;
use osfr_tools::{game_store::Snapshot, gameplay::GameplayState};

#[path = "../../../tests/common/item_catalogue.rs"]
#[allow(dead_code)]
mod item_catalogue;

/// A clean temp directory; dropping it removes the directory and the saves in it.
pub struct Scratch(PathBuf);

/// Distinguishes directories created in the same clock tick: the protected-name alias of the
/// transfer regression runs the same body twice, in parallel, under the same tag.
static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl Scratch {
    pub fn new(tag: &str) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-{tag}-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();
        Self(directory)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn state(&self) -> PathBuf {
        self.0.join("character.json")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The prepared round59 content: `OSFR_APP_FUZZ_CONTENT` when set, else the lab report. Tests
/// that skip without the private fixtures check this directory.
pub fn lab_content() -> PathBuf {
    std::env::var_os("OSFR_APP_FUZZ_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/reports/round59-native-20261015T124443Z/content")
        })
}

/// The prepared round59 content plus one synthetic vendor NPC and its one-row store, in the
/// scratch directory. `npc_spawns_extra.json` and `npcs/` keep the content local so the bundled
/// `data/content` catalogue cannot leak into a test; the caller may add its own catalogue files.
pub fn vendor_content(scratch: &Scratch) -> PathBuf {
    let content = scratch.path().join("content");
    fs::create_dir_all(content.join("npcs")).unwrap();
    fs::create_dir_all(content.join("quests")).unwrap();
    let source = lab_content();
    for entry in fs::read_dir(&source).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            fs::copy(&path, content.join(path.file_name().unwrap())).unwrap();
        }
    }
    let write = |name: &str, value: serde_json::Value| {
        fs::write(content.join(name), serde_json::to_vec(&value).unwrap()).unwrap();
    };
    let provenance = json!([{"kind":"reconstructed","evidence":"F264 synthetic test policy"}]);
    write(
        "npcs/synthetic.json",
        json!({"schema_version":1,"records":[
            {"npc_id":700,"name":"Synthetic NPC","zone_id":0,"position":[0.,0.,0.],
             "placement":"legacy_data","model_id":1,"roles":["vendor"],"vendor_stock":"synthetic"}
        ]}),
    );
    write(
        "npc_spawns_extra.json",
        json!({"schema_version":1,"records":[]}),
    );
    write(
        "zones.json",
        json!({"schema_version":1,"records":[{"id":0,"name":"Synthetic zone"}]}),
    );
    write(
        "vendors.json",
        json!({"schema_version":1,"vendors":[{"npc_id":700,"item_ids":[8]}]}),
    );
    write(
        "stores.json",
        json!({"schema_version":1,"marketplace":{
            "enabled_on_handshake":false,"orders_enabled":false,
            "payment_sources_enabled":false,"not_available_message":"disabled"},
            "items":[{"item_id":8,"purchase_price_coins":15,"member_discount_percent":null,
                      "member_price_coins":null,"sell_price_coins":7,"no_sale":false}]}),
    );
    write(
        "quests/synthetic.json",
        json!({"schema_version":1,"records":[{
            "id":701,"title_string_id":1,"giver_npc_id":700,"prerequisite_quest_ids":[],
            "objectives":[{"kind":"talk_to_npc","id":702,"npc_id":700,"source":provenance}],
            "rewards":[{"coins":7,"items":[],"job_xp":{},"source":provenance}],"source":provenance
        }]}),
    );
    write(
        "jobs.json",
        json!({"schema_version":1,"records":[{
            "id":"synthetic","name":"Synthetic job","profile_id":null,"category":"freestyle",
            "max_level":20,"members_only":false,"trainer_npc_ids":[],"quest_ids":[701],
            "abilities":[],"source":provenance
        }]}),
    );
    content
}

/// Writes an item catalogue with one synthetic definition per `(id, serialized size)` row. Each
/// record starts with its id and repeats `0x5a` to `size`, so a caller controls the 165/2
/// definition reply sizes it exercises. The row and envelope builders are shared with the
/// integration `inventory_catalogue` fixture (`tests/common/item_catalogue.rs`).
pub fn item_catalogue(content: &Path, rows: &[(u32, usize)]) {
    let definitions: Vec<serde_json::Value> = rows
        .iter()
        .map(|(id, size)| item_catalogue::sized_row(*id, *size))
        .collect();
    item_catalogue::write(content, &definitions);
}

/// True when `options` names the private lab fixture pair a session rig loads: the
/// reference manifest plus the top-level content files `Server::new` opens. A clean
/// checkout has neither, and a partial copy must be recognised here before the rig panics
/// in setup. Files named *inside* the content (the actor template, the quest journal) are
/// data-driven and deliberately not pinned, so renaming one cannot cause a false skip.
pub fn fixtures_present(options: &Options) -> bool {
    const CONTENT: [&str; 8] = [
        "inventory.json",
        "categories.json",
        "profiles.json",
        "opening.json",
        "opening-abilities.json",
        "combat.json",
        "encounters.json",
        "quests.json",
    ];
    options.reference.join("manifest.json").is_file()
        && options
            .content
            .as_ref()
            .is_some_and(|content| CONTENT.iter().all(|file| content.join(file).is_file()))
}

/// Options for a synthetic session on the bundled lab content, without a listener.
pub fn options(state: PathBuf) -> Options {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
    Options {
        bind: "127.0.0.1:0".parse().unwrap(),
        reference: root.join("references/rust-gateway-reference"),
        state,
        content: Some(root.join("references/rust-opening-cooking-original-assets")),
        identity: None,
        account_store: None,
        gateway_ticket: login::DEFAULT_GATEWAY_TICKET.into(),
        handoff_key: Vec::new(),
        seconds: 1,
        dev_tools: false,
        studio_port: 0,
        client_logs: None,
        terrain: None,
        resources: None,
        world: None,
        stock_text: None,
        allow_lan: false,
        security_log: None,
        group_invite_ttl_ms: osfr_tools::social_lists::DEFAULT_GROUP_INVITE_TTL_MS,
        guild_invite_ttl_ms: osfr_tools::guilds::DEFAULT_INVITE_WINDOW_MS,
    }
}

/// The client's Play on an activity-portal row (167/1/2: the activity, then its trailing word),
/// the same bytes `tests/common/field_battle_fixture.rs` sends. The bin target cannot include
/// that integration fixture, so the builder lives here once for every bin test.
pub fn portal_play(activity: u32) -> Vec<u8> {
    let mut play = vec![167, 0, 1, 2];
    play.extend(activity.to_le_bytes());
    play.extend(0_u32.to_le_bytes());
    play
}

/// `Server::step` for a rig whose sessions have no client behind them. A real client keeps
/// the 60 s idle budget alive with its datagrams and KeepAlives; a rig's `dispatch_message`
/// calls and its test body do not, so a loaded host that stalls a test past the budget would
/// end every session as `idle_timeout` and look like a moderation, isolation or routing
/// failure (T628). Refresh the budget first, as a live client would; every other close path
/// (kick, error, panic) still runs unchanged.
pub fn step(server: &mut Server) -> Result<(), Box<dyn std::error::Error>> {
    for session in server.sessions.values_mut() {
        session.last_activity = Instant::now();
    }
    server.step()
}

/// A session before its login, with the fixture's own port and datagram size.
fn session(options: &Options, id: u32, port: u32, buffer_size: usize) -> Session {
    Session::new(
        SessionId(u64::from(id)),
        format!("127.0.0.1:{port}").parse().unwrap(),
        SessionRequest {
            id,
            crc_length: 3,
            buffer_size,
        },
        options,
        CRC_SEED,
        None,
        None,
    )
    .unwrap()
}

/// A logged-in session, before its world entry. The caller drives `[13,0]`/`[10,0]` either
/// through `Gateway::handle` (a direct handshake) or through `Scene::send` (the session commit
/// path), so each test keeps the entry route it is proving.
pub fn enter(options: &Options, id: u32) -> Session {
    let mut session = session(options, id, 23_000 + id, 512);
    let mut login = 1u16.to_le_bytes().to_vec();
    for (index, text) in ["synthetic-ticket", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
        login.extend((text.len() as u32).to_le_bytes());
        login.extend(text.as_bytes());
        if index == 0 {
            login.extend(1u64.to_le_bytes());
        }
    }
    session.gateway.handle(&login).unwrap();
    session
}

/// Write one character save envelope through the production `game_store::save`.
pub fn save_character(path: &Path, character: &CharacterState, gameplay: &GameplayState) {
    save(
        path,
        &Snapshot {
            character: character.clone(),
            gameplay: Some(gameplay.clone()),
            pets: None,
        },
    )
    .expect("fixture save writes");
}

/// One session and the containers a `Server` owns, without a listening socket.
pub struct Scene {
    pub session: Session,
    pub world: SharedWorld,
    pub membership: World,
    pub sessions: SessionMap,
}

impl Scene {
    pub fn new(options: &Options, id: u32) -> Self {
        Self {
            session: enter(options, id),
            world: SharedWorld::new(),
            membership: World::runtime(),
            sessions: SessionMap::new(),
        }
    }

    /// Like `new`, but the field is on the membership before the session exists: the order
    /// `Server::new` installs its seed in, so a login/enter path cannot see an empty field.
    pub fn with_field(
        options: &Options,
        id: u32,
        field: osfr_tools::encounter::SharedFieldState,
    ) -> Self {
        let mut membership = World::runtime();
        membership.install_field_seed(0, field);
        Self {
            session: enter(options, id),
            world: SharedWorld::new(),
            membership,
            sessions: SessionMap::new(),
        }
    }

    /// Like `new`, but without the login: a test that drives the committed login itself keeps
    /// its own handshake while sharing this staging path. `port` and `buffer_size` are the
    /// fixture's, because the session codec reads the datagram limit.
    pub fn unentered(options: &Options, id: u32, port: u32, buffer_size: usize) -> Self {
        Self {
            session: session(options, id, port, buffer_size),
            world: SharedWorld::new(),
            membership: World::runtime(),
            sessions: SessionMap::new(),
        }
    }

    /// One already-framed client message through `Session::dispatch_message`. A login body is
    /// top-level and uses this directly; application bodies go through `send`.
    pub fn dispatch(&mut self, options: &Options, message: &[u8], now: u64) -> Result<(), String> {
        self.session
            .dispatch_message(
                message,
                options,
                now,
                &mut self.world,
                &mut self.membership,
                &mut self.sessions,
                None,
            )
            .map_err(|error| error.to_string())
    }

    /// One client application message through `Session::dispatch_message`, the same
    /// staging/commit/save path a socket datagram takes.
    pub fn send(&mut self, options: &Options, inner: &[u8], now: u64) -> Result<(), String> {
        self.dispatch(options, &wrap(inner), now)
    }

    /// `send` plus the replies it queued, for tests that assert on the bytes the client
    /// sees. A refusal fails the test: these cases drive requests that must commit.
    pub fn exchange(&mut self, options: &Options, inner: &[u8], now: u64) -> Vec<Vec<u8>> {
        self.send(options, inner, now).unwrap_or_else(|error| {
            panic!(
                "packet {:?} was refused: {error}",
                &inner[..inner.len().min(4)]
            )
        });
        self.drain(now)
    }

    pub fn drain(&mut self, now: u64) -> Vec<Vec<u8>> {
        super::super::f120_fishing_hour_reconnect::drain_replies(&mut self.session, now)
    }

    pub fn snapshot(&self) -> Snapshot {
        self.session.gateway.snapshot()
    }

    /// Break only the next save: the good save file is a component of the target path, so
    /// `save` cannot create its temporary file beside it. The good bytes stay put.
    pub fn break_next_save(&mut self, good_save: &Path) {
        self.session.state_path = good_save.join("blocked.json");
    }

    pub fn repair_save(&mut self, good_save: &Path) {
        self.session.state_path = good_save.to_owned();
    }

    pub fn tick(&mut self, options: &Options, now: u64) {
        self.session
            .tick(
                options,
                now,
                &mut self.world,
                &mut self.membership,
                &mut self.sessions,
            )
            .unwrap();
    }
}

/// Ready and Loaded, the two messages every world session sends after login, through the
/// production dispatch path (`Scene::new` already sent the login). Returns the entry replies.
pub fn enter_world(scene: &mut Scene, options: &Options) -> Vec<Vec<u8>> {
    for (message, now) in [([13, 0], 100), ([10, 0], 200)] {
        scene
            .send(options, &message, now)
            .expect("world entry commits");
    }
    scene.drain(200)
}

/// Whether this copy is the binary's own test target. The gateway source is embedded by five
/// integration harnesses, which compile these test modules too; an embedded copy says so on
/// stderr (libtest does not capture a direct fd write), so a green integration-target line is
/// not mistaken for a real run.
pub fn bin_test_target(tag: &str) -> bool {
    if option_env!("CARGO_BIN_NAME").is_some() {
        return true;
    }
    use std::io::Write as _;
    let _ = std::io::stderr().write_all(
        format!(
            "{tag}: embedded gateway copy skipped; run `cargo test --bin osfr-gateway {tag}_`\n"
        )
        .as_bytes(),
    );
    false
}
