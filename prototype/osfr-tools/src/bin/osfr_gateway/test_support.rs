//! Shared setup for the gateway binary's `#[cfg(test)]` session modules: one scratch
//! directory, one `Options` initializer and one socket-less session fixture, so a new
//! option field or handshake step is added once instead of in every test copy. Test-only;
//! production never includes this file.
use super::*;
use osfr_tools::item_bar::use_wire::{
    ActionBarSlotUpdate, Target, decode_slot_update, encode_client_request_start_ability,
};
use osfr_tools::players::PlayerPresentation;
use osfr_tools::{game_store::Snapshot, gameplay::GameplayState};

#[path = "../../../tests/common/item_catalogue.rs"]
#[allow(dead_code)]
mod item_catalogue;

#[path = "fishing_fixtures.rs"]
#[allow(dead_code)] // only `copy_tree` is used here
mod fixtures;

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

/// The repo's content copied into a scratch `content` directory, then the runtime pack
/// (`runtime`, e.g. `round70_content::source()`) over it, as the playtest host ran it: the repo
/// fills what the pack does not ship (zones, NPCs, field actors) and the pack wins elsewhere.
pub fn repo_and_runtime_content(scratch: &Scratch, runtime: &Path) -> PathBuf {
    let content = scratch.path().join("content");
    fixtures::copy_tree(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content"),
        &content,
    );
    fixtures::copy_tree(runtime, &content);
    content
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

// T461/T557: one owner in the open world with an item on bar slot 0, and any number of viewers.

/// The owner's character id in a [`Rig`].
pub const OWNER: u32 = 461;

pub fn sender(max_queued_bytes: usize) -> ReliableSender {
    ReliableSender::new(
        Codec::new(CRC_SEED, 3, true, MAX_MESSAGE).unwrap(),
        SenderConfig {
            // Roomy, so an AddNpc is one datagram the drain below can read back.
            datagram_limit: 4096,
            window: 32,
            max_message: MAX_MESSAGE,
            max_queued_bytes,
            retry_ms: 250,
            max_retries: 40,
        },
    )
    .unwrap()
}

pub fn msg(main: u16, sub: u16, fields: &[i32]) -> Vec<u8> {
    let mut bytes = [main.to_le_bytes(), sub.to_le_bytes()].concat();
    fields.iter().for_each(|f| bytes.extend(f.to_le_bytes()));
    bytes
}

/// The item bar's slot 0 as the owner's client is told it, from a drained batch.
pub fn slot_zero(messages: &[Vec<u8>]) -> ActionBarSlotUpdate {
    let updates = messages.iter().filter_map(|m| m.get(7..));
    let mut updates = updates.filter_map(decode_slot_update);
    updates.find(|s| s.bar == 2 && s.slot == 0).unwrap()
}

pub struct Rig {
    pub options: Options,
    pub scene: Scene,
    _scratch: Scratch,
    pub instance: i32,
    /// The options of each full `player`, for `send_as`.
    players: BTreeMap<SessionKey, Options>,
}

impl Rig {
    /// The owner has entered the open world holding `item`, bound to bar slot 0, with the boombox
    /// and ride catalogues (T461).
    pub fn new(tag: &str, item: i32) -> Option<Self> {
        Self::with_files(tag, item, &["boomboxes.json", "mounts.json"])
    }

    /// As `new`, with the bundled content `files` copied into the content directory (T557).
    pub fn with_files(tag: &str, item: i32, files: &[&str]) -> Option<Self> {
        let scratch = Scratch::new(tag);
        let mut options = options(scratch.state());
        if !bin_test_target("t461") || !fixtures_present(&options) {
            eprintln!("skip {tag}: not the binary's own target, or no lab fixtures");
            return None;
        }
        let content = vendor_content(&scratch);
        let shipped = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
        for file in files {
            fs::copy(shipped.join(file), content.join(file)).unwrap();
        }
        options.content = Some(content);
        options.dev_tools = true;
        let mut scene = Scene::new(&options, OWNER);
        scene.session.sender = sender(32 * 1024 * 1024);
        scene.send(&options, &[13, 0], 1_000).unwrap();
        scene.send(&options, &[10, 0], 1_001).unwrap();
        assert!(scene.session.gateway.is_world_loaded());
        // The server registers a session's key at accept; the scene does not.
        let (id, peer) = (scene.session.id, scene.session.peer);
        let connection_id = scene.session.request.id;
        let key = SessionKey {
            peer,
            connection_id,
        };
        scene.world.session_keys.insert(id, key);
        let mut rig = Self {
            options,
            scene,
            _scratch: scratch,
            instance: 0,
            players: BTreeMap::new(),
        };
        rig.give(item);
        rig.instance = rig.instance_of(item);
        rig.drain_owner(1_500);
        rig.exchange(&msg(42, 6, &[0, rig.instance]), 2_000);
        Some(rig)
    }

    pub fn give(&mut self, item: i32) {
        let command = osfr_tools::chat::local_request(&format!("realm give {item}"));
        self.scene.send(&self.options, &command, 1_100).unwrap();
    }

    pub fn instance_of(&self, item: i32) -> i32 {
        let state = self.scene.session.gateway.snapshot().gameplay.unwrap();
        let held = state.inventory.items.iter();
        held.filter(|i| i.definition_id == item).last().unwrap().id
    }

    pub fn exchange(&mut self, inner: &[u8], now: u64) -> Vec<Vec<u8>> {
        self.scene.exchange(&self.options, inner, now)
    }

    pub fn drain_owner(&mut self, now: u64) -> Vec<Vec<u8>> {
        super::super::f120_fishing_hour_reconnect::drain_replies(&mut self.scene.session, now)
    }

    /// 36/10 on the item bar, slot 0, aimed at the player: the owner's replies.
    pub fn press(&mut self, now: u64) -> Vec<Vec<u8>> {
        let target = Target::Guid(self.scene.session.character_guid);
        let packet = encode_client_request_start_ability(2, 0, target);
        self.scene.send(&self.options, &packet, now).unwrap();
        self.drain_owner(now)
    }

    /// The bar as the owner's client sees it now (a no-op rebind answers with the bar).
    pub fn bar(&mut self, now: u64) -> ActionBarSlotUpdate {
        let rebind = msg(42, 6, &[0, self.instance]);
        slot_zero(&self.exchange(&rebind, now))
    }

    /// A second session in the open world `dx` metres east of the owner.
    pub fn viewer(&mut self, id: u32, dx: f32) -> SessionKey {
        self.viewer_as(id, dx, 0).0
    }

    /// As `viewer`, wearing player model `model` (the boxes dance models 1, 2, 60 and 61).
    /// Also returns the AddPc/movement messages the registry built for the join, which the
    /// real gateway delivers after `join_player`.
    pub fn viewer_as(
        &mut self,
        id: u32,
        dx: f32,
        model: i32,
    ) -> (SessionKey, Vec<osfr_tools::players::OutboundMessage>) {
        let mut session = enter(&self.options, id);
        session.sender = sender(32 * 1024 * 1024);
        let me = self
            .scene
            .world
            .players
            .get(self.scene.session.id)
            .unwrap()
            .clone();
        let mut look = PlayerPresentation::default();
        look.character_guid = 7_000_000_000 + u64::from(id);
        look.model = model;
        let at = [me.position[0] + dx, me.position[1], me.position[2]];
        let world = &mut self.scene.world;
        let account = 8_000_000_000 + u64::from(id);
        let (character, zone) = (look.character_guid, me.zone);
        let joined = world
            .players
            .join_with_presentation(session.id, account, character, zone, at, 0.0, look);
        let joined = joined.unwrap();
        world.joined.insert(session.id);
        let key = SessionKey {
            peer: session.peer,
            connection_id: session.request.id,
        };
        world.session_keys.insert(session.id, key);
        self.scene.sessions.insert(key, session);
        (key, joined)
    }

    pub fn drain(&mut self, key: SessionKey, now: u64) -> Vec<Vec<u8>> {
        super::super::f120_fishing_hour_reconnect::drain_replies(
            self.scene.sessions.get_mut(&key).unwrap(),
            now,
        )
    }

    /// `key`'s player now shows the temporary model `model`, keeping its base model, as a
    /// live transform's AddPc snapshot does (`PlayerPresentation::set_visual`).
    pub fn transform(&mut self, key: SessionKey, model: i32) {
        let id = self.scene.sessions[&key].id;
        let players = &mut self.scene.world.players;
        let mut look = players.get(id).unwrap().presentation.clone();
        look.set_visual(osfr_tools::consumable_effects::Visual {
            temporary_model: model,
            scale: 1.0,
            auras: Vec::new(),
        });
        players.set_presentation(id, look).unwrap();
    }

    /// The owner is the scene's own session, so it joins the session map for a tick or for
    /// another player's request, as every session is in the real loop; `unlist_owner` takes it back.
    fn list_owner(&mut self) -> SessionKey {
        let stand_in = enter(&self.options, 9_999);
        let owner = std::mem::replace(&mut self.scene.session, stand_in);
        let key = SessionKey {
            peer: owner.peer,
            connection_id: owner.request.id,
        };
        self.scene.sessions.insert(key, owner);
        key
    }

    fn unlist_owner(&mut self, key: SessionKey) {
        self.scene.session = self.scene.sessions.remove(&key).unwrap();
    }

    /// T557: a second full player, character `guid` (not the owner's 1), logged in and in the
    /// world through the real messages and then `dx` metres east of the owner. The nearer
    /// `viewer` is only a registry row; this one has a gameplay and can act (`send_as`).
    pub fn player(&mut self, id: u32, guid: u64, dx: f32) -> SessionKey {
        let scratch = self._scratch.path().to_path_buf();
        let mut options = options(scratch.join(format!("character-{guid}.json")));
        options.reference = super::super::t199_world_notice::reference(&scratch, guid);
        options.content = self.options.content.clone();
        options.dev_tools = true;
        let mut session = session(&options, id, 24_000 + id, 1400);
        session.sender = sender(32 * 1024 * 1024);
        let key = SessionKey {
            peer: session.peer,
            connection_id: session.request.id,
        };
        let world = &mut self.scene.world;
        world.session_keys.insert(session.id, key);
        let login = super::super::t199_world_notice::login(guid);
        for (message, now) in [
            (login, 1_002),
            (wrap(&[13, 0]), 1_003),
            (wrap(&[10, 0]), 1_004),
        ] {
            let (membership, sessions) = (&mut self.scene.membership, &mut self.scene.sessions);
            session
                .dispatch_message(&message, &options, now, world, membership, sessions, None)
                .unwrap();
        }
        let at = world.players.get(self.scene.session.id).unwrap().clone();
        let to = [at.position[0] + dx, at.position[1], at.position[2]];
        let moved = world
            .players
            .update_pose_with_state(session.id, at.zone, to, 0.0, 0, 0);
        moved.unwrap();
        self.scene.sessions.insert(key, session);
        self.players.insert(key, options);
        key
    }

    /// One client application message from a `player`, through `Session::dispatch_message`.
    pub fn send_as(&mut self, key: SessionKey, inner: &[u8], now: u64) -> Result<(), String> {
        let owner = self.list_owner();
        let mut session = self.scene.sessions.remove(&key).unwrap();
        let scene = &mut self.scene;
        let result = session.dispatch_message(
            &wrap(inner),
            &self.players[&key],
            now,
            &mut scene.world,
            &mut scene.membership,
            &mut scene.sessions,
            None,
        );
        self.scene.sessions.insert(key, session);
        self.unlist_owner(owner);
        result.map_err(|error| error.to_string())
    }

    /// One server tick.
    pub fn tick(&mut self, now: u64) {
        let key = self.list_owner();
        let (world, sessions) = (&mut self.scene.world, &mut self.scene.sessions);
        super::super::boombox_routing::tick(world, sessions, now);
        super::super::cake_routing::tick(world, sessions, now);
        self.unlist_owner(key);
    }

    pub fn move_viewer(&mut self, key: SessionKey, dx: f32) {
        let players = &self.scene.world.players;
        let me = players.get(self.scene.session.id).unwrap().clone();
        let id = self.scene.sessions[&key].id;
        let at = [me.position[0] + dx, me.position[1], me.position[2]];
        let players = &mut self.scene.world.players;
        players
            .update_pose_with_state(id, me.zone, at, 0.0, 0, 0)
            .unwrap();
    }

    /// The live consumable effects of a `player` (session-only state, never saved).
    pub fn effects(&self, key: SessionKey) -> osfr_tools::consumable_effects::EffectStack {
        let gameplay = self.scene.sessions[&key].gateway.gameplay().unwrap();
        gameplay.consumable_effects().clone()
    }

    /// Everything the save holds about the box's owner, byte for byte.
    pub fn saved(&self) -> Vec<u8> {
        let state = self.scene.session.gateway.snapshot().gameplay.unwrap();
        serde_json::to_vec(&(&state.inventory, &state.item_bar)).unwrap()
    }

    /// The character's whole saved gameplay state, byte for byte.
    pub fn saved_state(&self) -> Vec<u8> {
        let state = self.scene.session.gateway.snapshot().gameplay.unwrap();
        serde_json::to_vec(&state).unwrap()
    }

    pub fn count(&self, item: i32) -> Option<u32> {
        let state = self.scene.session.gateway.snapshot().gameplay.unwrap();
        let held = state.inventory.items.iter();
        held.filter(|i| i.definition_id == item)
            .map(|i| i.count)
            .last()
    }
}

/// T889: a `Server` whose logged-in sessions share its world and field. Every packet goes through
/// `Session::dispatch_message` (staging, `commit_outcome`, the shared-field step), the path a
/// socket datagram takes, so a bug that only shows with two clients on one gateway can be pinned.
/// Session `index` is character guid `index + 1`; `players` holds each one's saved character.
#[allow(dead_code)] // the embedded integration copies of the gateway never build a table
pub struct Table {
    pub server: Server,
    pub keys: Vec<SessionKey>,
    /// Every application packet each session was sent so far, in order.
    pub seen: Vec<Vec<Vec<u8>>>,
    _scratch: Scratch,
}

#[allow(dead_code)]
impl Table {
    /// `content` is the content directory every session loads and `reference` the character
    /// directory; `None` if the private lab reference is absent, so a clean checkout skips
    /// instead of failing in setup.
    pub fn new(
        scratch: Scratch,
        content: &Path,
        reference: &Path,
        players: &[(CharacterState, osfr_tools::gameplay::GameplayState)],
    ) -> Option<Self> {
        if !reference.join("manifest.json").is_file() {
            eprintln!("skip: the private lab fixtures are absent");
            return None;
        }
        let make = |state: PathBuf| Options {
            content: Some(content.to_owned()),
            reference: reference.to_owned(),
            world: Some(scratch.path().join("world")),
            ..options(state)
        };
        let mut server = Server::new(make(scratch.state())).unwrap();
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(reference.join("manifest.json")).unwrap()).unwrap();
        let version = manifest["client_version"].as_str().unwrap().to_owned();
        let mut keys = Vec::new();
        for (index, (character, saved)) in players.iter().enumerate() {
            let id = index as u32 + 1;
            let key = SessionKey {
                peer: format!("127.0.0.1:{}", 45_000 + id).parse().unwrap(),
                connection_id: id,
            };
            let options = make(scratch.path().join(format!("character-{id}.json")));
            let mut session = session(&options, id, 45_000 + id, 1400);
            let mut gateway =
                Gateway::load_gameplay(reference, character.clone(), content, Some(saved.clone()))
                    .unwrap();
            gateway.take_shared_fields();
            session.gateway = gateway;
            session.character_guid = character.guid;
            session.account_guid = 10_000 + character.guid;
            session.state_path = options.state.clone();
            server.world.session_keys.insert(session.id, key);
            server.sessions.insert(key, session);
            keys.push(key);
        }
        let mut table = Self {
            server,
            seen: vec![Vec::new(); keys.len()],
            keys,
            _scratch: scratch,
        };
        for (index, (character, _)) in players.iter().enumerate() {
            for (step, message) in [
                f160_field_views::f213_login(character.guid, &version, "t889"),
                wrap(&[13, 0]),
                wrap(&[10, 0]),
            ]
            .iter()
            .enumerate()
            {
                table.dispatch(index, message, 100 + step as u64 * 100);
            }
            table.drain(index, 400);
        }
        Some(table)
    }

    fn dispatch(&mut self, index: usize, message: &[u8], now: u64) {
        let key = self.keys[index];
        let server = &mut self.server;
        let mut session = server.sessions.remove(&key).unwrap();
        let result = session.dispatch_message(
            message,
            &server.options,
            now,
            &mut server.world,
            &mut server.membership,
            &mut server.sessions,
            None,
        );
        server.sessions.insert(key, session);
        result.unwrap();
    }

    /// Everything queued for session `index` as inner application bodies.
    pub fn drain(&mut self, index: usize, now: u64) -> Vec<Vec<u8>> {
        let session = self.server.sessions.get_mut(&self.keys[index]).unwrap();
        let replies: Vec<_> = super::f120_fishing_hour_reconnect::drain_replies(session, now)
            .into_iter()
            .filter(|message| message.starts_with(&[5, 0, 1]))
            .map(|message| message[7..].to_vec())
            .collect();
        self.seen[index].extend(replies.iter().cloned());
        replies
    }

    /// One client packet from session `index`, then what that session was sent.
    pub fn send(&mut self, index: usize, inner: &[u8], now: u64) -> Vec<Vec<u8>> {
        self.dispatch(index, &wrap(inner), now);
        self.drain(index, now)
    }

    /// One shared-field tick (view sync, motion), then empty every session's queue.
    pub fn tick(&mut self, now: u64) {
        self.server.tick_shared_fields(now);
        for index in 0..self.keys.len() {
            self.drain(index, now);
        }
    }

    pub fn field(&self) -> &osfr_tools::encounter::SharedFieldState {
        self.server
            .membership
            .field(0)
            .expect("the field is installed")
    }
}
