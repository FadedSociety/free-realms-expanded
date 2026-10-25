//! T006: the family-129 route through the real session dispatch and save path.
//!
//! The packet layouts are pinned by the library tests; these tests prove the gateway label,
//! account budget, staged guild save, restart load, name lookup and guild chat wiring.

use super::t006_guilds::GuildWorld;
use super::*;
use osfr_tools::guilds::GuildStore;
use osfr_tools::players::PlayerPresentation;
use osfr_tools::social_lists::SocialName;

const GUIDS: [u64; 2] = [601, 602];
const NAMES: [&str; 2] = ["G601", "G602"];

struct Rig {
    sessions: [Session; 2],
    world: SharedWorld,
    options: Options,
    scratch: PathBuf,
}

impl Drop for Rig {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.scratch);
    }
}

fn options(scratch: &Path) -> Options {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
    Options {
        bind: "127.0.0.1:0".parse().unwrap(),
        reference: root.join("references/rust-gateway-reference"),
        state: scratch.join("character.json"),
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
        world: Some(scratch.join("world")),
        stock_text: None,
        allow_lan: false,
        security_log: None,
        group_invite_ttl_ms: osfr_tools::social_lists::DEFAULT_GROUP_INVITE_TTL_MS,
        guild_invite_ttl_ms: osfr_tools::guilds::DEFAULT_INVITE_WINDOW_MS,
    }
}

fn identity(index: usize) -> SocialIdentity {
    SocialIdentity {
        account_id: GUIDS[index] + 10_000,
        character_guid: GUIDS[index],
        name: SocialName::new(NAMES[index], "").unwrap(),
    }
}

fn session(options: &Options, index: usize) -> Session {
    let id = 17 + index as u64;
    let mut session = Session::new(
        SessionId(id),
        "127.0.0.1:12345".parse().unwrap(),
        SessionRequest {
            id: id as u32,
            crc_length: 3,
            buffer_size: 512,
        },
        options,
        CRC_SEED,
        None,
        None,
    )
    .unwrap();
    session.character_guid = GUIDS[index];
    session.account_guid = GUIDS[index] + 10_000;
    session.social_identity = Some(identity(index));
    session
}

fn session_key(session: &Session) -> SessionKey {
    SessionKey {
        peer: session.peer,
        connection_id: session.request.id,
    }
}

fn seed_world(world: &mut SharedWorld, sessions: &[Session; 2]) {
    for (index, session) in sessions.iter().enumerate() {
        let guid = GUIDS[index];
        world
            .players
            .join_with_presentation(
                session.id,
                guid + 10_000,
                guid,
                ZoneInstance {
                    zone_id: 1,
                    instance_id: 1,
                },
                [index as f32 * 5.0, 0.0, 0.0],
                0.0,
                PlayerPresentation {
                    character_guid: guid,
                    first_name: NAMES[index].to_owned(),
                    ..PlayerPresentation::default()
                },
            )
            .unwrap();
        let identity = identity(index);
        world.identities.insert(guid, identity.clone());
        world
            .social
            .register(SocialLists::new(identity).unwrap())
            .unwrap();
        world.social.login(guid).unwrap();
        world.joined.insert(session.id);
        world.active_sessions.insert(guid, session.id);
        world.session_keys.insert(session.id, session_key(session));
        world.chat_sessions.insert(
            session.id,
            ChatMember {
                name: NameData {
                    fields: [0; 3],
                    first: NAMES[index].to_owned(),
                    last: String::new(),
                },
                is_member: true,
            },
        );
        world.chat_settings.join(guid, None);
    }
}

fn seed_guild(path: &Path) -> u64 {
    let mut store = GuildStore::new();
    let guild = store
        .create_guild(
            GUIDS[0],
            "Lantern Guild",
            &NameData::new(NAMES[0], ""),
            true,
        )
        .unwrap();
    store.save(path).unwrap();
    guild
}

/// A saved character with one unlocked job at rank 16, so the real eligibility gate passes.
fn seed_eligible_character(options: &Options) {
    let initial = Gateway::reference_state(&options.reference).unwrap();
    let content = options.content.as_ref().unwrap();
    let character = Gameplay::initial_character(content, initial).unwrap();
    let mut snapshot = Gateway::load_gameplay(&options.reference, character, content, None)
        .unwrap()
        .snapshot();
    let profiles = snapshot
        .gameplay
        .as_mut()
        .and_then(|game| game.profiles.as_mut())
        .expect("reference content has job profiles");
    profiles.unlocked.insert(profiles.active);
    profiles.experience.insert(profiles.active, 1_500);
    save(&options.state, &snapshot).unwrap();
}

fn rig(name: &str, with_guild: bool) -> Rig {
    let scratch = std::env::temp_dir().join(format!("osfr-t006-{name}-{}", std::process::id()));
    fs::create_dir_all(&scratch).unwrap();
    let options = options(&scratch);
    if with_guild {
        seed_guild(&options.state.with_file_name("guilds.json"));
    }
    let sessions = std::array::from_fn(|index| session(&options, index));
    let mut world = SharedWorld::new();
    world.guilds = GuildWorld::load(&options.state, options.guild_invite_ttl_ms).unwrap();
    seed_world(&mut world, &sessions);
    Rig {
        sessions,
        world,
        options,
        scratch,
    }
}

fn eligible_rig(name: &str) -> Rig {
    let scratch = std::env::temp_dir().join(format!("osfr-t006-{name}-{}", std::process::id()));
    fs::create_dir_all(&scratch).unwrap();
    let options = options(&scratch);
    seed_eligible_character(&options);
    let sessions = std::array::from_fn(|index| session(&options, index));
    let mut world = SharedWorld::new();
    world.guilds = GuildWorld::load(&options.state, options.guild_invite_ttl_ms).unwrap();
    seed_world(&mut world, &sessions);
    Rig {
        sessions,
        world,
        options,
        scratch,
    }
}

fn route(rig: &mut Rig, index: usize, packet: &[u8]) -> Vec<Delivery> {
    rig.sessions[index].route_application(packet, &mut rig.world, 1_000)
}

fn wrap(inner: &[u8]) -> Vec<u8> {
    wrap_application(inner)
}

fn log_in(session: &mut Session, ticket: &str) {
    let mut login = 1u16.to_le_bytes().to_vec();
    for (index, text) in [ticket, "1.910.1.530630", ""].into_iter().enumerate() {
        login.extend((text.len() as u32).to_le_bytes());
        login.extend(text.as_bytes());
        if index == 0 {
            login.extend(1u64.to_le_bytes());
        }
    }
    session.gateway.handle(&login).unwrap();
    session.gateway.handle(&wrap(&[13, 0])).unwrap();
    session.gateway.handle(&wrap(&[10, 0])).unwrap();
}

/// The production `dispatch_message` path: gateway label, account budget, staging and commit.
fn dispatch_result(
    rig: &mut Rig,
    index: usize,
    inner: &[u8],
    now_ms: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut membership = World::runtime();
    let mut sessions = SessionMap::new();
    let options = &rig.options;
    rig.sessions[index].dispatch_message(
        &wrap(inner),
        options,
        now_ms,
        &mut rig.world,
        &mut membership,
        &mut sessions,
        None,
    )
}

fn dispatch(rig: &mut Rig, index: usize, inner: &[u8], now_ms: u64) {
    dispatch_result(rig, index, inner, now_ms).unwrap();
}

fn string(out: &mut Vec<u8>, value: &str) {
    out.extend((value.len() as i32).to_le_bytes());
    out.extend(value.as_bytes());
}

fn create(name: &str) -> Vec<u8> {
    let mut bytes = vec![129, 0, 1, 0];
    string(&mut bytes, name);
    string(&mut bytes, "en_US");
    string(&mut bytes, "");
    bytes
}

fn motd(guild: u64, text: &str) -> Vec<u8> {
    let mut bytes = vec![129, 0, 11, 0];
    bytes.extend(guild.to_le_bytes());
    string(&mut bytes, text);
    bytes
}

fn invite(guild: u64, target: u64, name: &str) -> Vec<u8> {
    let mut bytes = vec![129, 0, 3, 0];
    bytes.extend(guild.to_le_bytes());
    bytes.extend(target.to_le_bytes());
    string(&mut bytes, name);
    bytes
}

fn one_guid(sub: u16, guild: u64) -> Vec<u8> {
    let mut bytes = vec![129, 0];
    bytes.extend(sub.to_le_bytes());
    bytes.extend(guild.to_le_bytes());
    bytes
}

fn family_sub(delivery: &Delivery) -> (u16, u16) {
    let inner = &delivery.bytes[7..];
    (
        u16::from_le_bytes([inner[0], inner[1]]),
        u16::from_le_bytes([inner[2], inner[3]]),
    )
}

fn has(deliveries: &[Delivery], index: usize, sub: u16) -> bool {
    let recipient = SessionId(17 + index as u64);
    deliveries
        .iter()
        .any(|delivery| delivery.recipient == recipient && family_sub(delivery).1 == sub)
}

#[test]
fn create_then_invite_accept_reaches_the_online_founder() {
    let mut rig = eligible_rig("founder-online");
    log_in(&mut rig.sessions[0], "t006-founder");
    // The real gateway path creates the guild; the creator is a connected session, so the
    // store marks it online before any roster is built (129/22 member bool +180).
    dispatch(&mut rig, 0, &create("Lantern Guild"), 1_000);
    // The reference save carries template guid 1, so the committed dispatch adopts it for the
    // session; the guild under test belongs to the seeded social identity. Production saves
    // always carry the session's own guid; restore it for the rest of this fixture.
    rig.sessions[0].character_guid = GUIDS[0];
    let guild = rig
        .world
        .guilds
        .store()
        .guild_guid_for(GUIDS[0])
        .expect("the founder owns the new guild");
    assert!(
        rig.world.guilds.store().is_online(GUIDS[0]),
        "the founder is online"
    );

    let invited = route(&mut rig, 0, &invite(guild, 0, NAMES[1]));
    assert!(has(&invited, 1, 24), "invitee pop-up");
    let accepted = route(&mut rig, 1, &one_guid(8, guild));
    assert!(has(&accepted, 0, 27), "founder accept notice");
    // The founder now receives the directed MemberJoined status and the joiner its roster;
    // before the online fix the broadcast filtered the founder out.
    assert!(has(&accepted, 0, 17), "founder sees the join status");
    assert!(has(&accepted, 1, 22), "joiner roster");
}

#[test]
fn motd_routes_through_dispatch_is_saved_and_survives_a_restart() {
    let mut rig = rig("persist", true);
    let guild = rig.world.guilds.store().guild_guid_for(GUIDS[0]).unwrap();
    for (index, session) in rig.sessions.iter_mut().enumerate() {
        log_in(session, &format!("t006-persist-{index}"));
    }
    dispatch(&mut rig, 0, &motd(guild, "Synthetic message"), 1_000);
    assert_eq!(
        rig.world.guilds.store().guild(guild).unwrap().motd,
        "Synthetic message"
    );
    let guild_file = rig.options.state.with_file_name("guilds.json");
    let saved = fs::read_to_string(&guild_file).unwrap();
    assert!(saved.contains("Synthetic message"), "{saved}");

    // A restart loads the guild file before any session exists.
    let options = options(&rig.scratch);
    let restarted = Server::new(options).unwrap();
    assert_eq!(
        restarted.world.guilds.store().guild(guild).unwrap().motd,
        "Synthetic message"
    );
    assert_eq!(
        restarted.world.guilds.store().guild_guid_for(GUIDS[0]),
        Some(guild)
    );
}

#[test]
fn create_below_job_rank_15_changes_nothing() {
    let mut rig = rig("eligibility", false);
    log_in(&mut rig.sessions[0], "t006-eligibility");
    dispatch(&mut rig, 0, &create("Lantern Guild"), 1_000);
    assert_eq!(rig.world.guilds.store().guild_guid_for(GUIDS[0]), None);
    assert!(
        !rig.options.state.with_file_name("guilds.json").exists(),
        "a refused create wrote a guild file"
    );
}

#[test]
fn create_at_job_rank_16_routes_through_dispatch_and_persists() {
    let mut rig = eligible_rig("create");
    log_in(&mut rig.sessions[0], "t006-create");
    dispatch(&mut rig, 0, &create("Lantern Guild"), 1_000);
    let guild = rig
        .world
        .guilds
        .store()
        .guild_guid_for(GUIDS[0])
        .expect("an eligible creator joins the new guild");
    let file = rig.options.state.with_file_name("guilds.json");
    let saved = fs::read_to_string(&file).unwrap();
    assert!(saved.contains("Lantern Guild"), "{saved}");

    // The committed guild survives a restart, like every other guild request.
    let restarted = Server::new(options(&rig.scratch)).unwrap();
    assert_eq!(
        restarted.world.guilds.store().guild_guid_for(GUIDS[0]),
        Some(guild)
    );
}

#[test]
fn invite_accept_presence_and_guild_chat_reach_the_real_router() {
    let mut rig = rig("flow", true);
    let guild = rig.world.guilds.store().guild_guid_for(GUIDS[0]).unwrap();

    // The leader invites by display name (the client sends guid 0 and a name).
    let invited = route(&mut rig, 0, &invite(guild, 0, "G602"));
    assert!(has(&invited, 1, 24), "invitee pop-up");

    // The target accepts: 27 to the inviter, 17 and 22 to the joiner.
    let accepted = route(&mut rig, 1, &one_guid(8, guild));
    assert!(has(&accepted, 0, 27), "inviter accept notice");
    assert!(has(&accepted, 1, 17), "joiner member status");
    assert!(has(&accepted, 1, 22), "joiner roster");
    assert_eq!(
        rig.world.guilds.store().guild_guid_for(GUIDS[1]),
        Some(guild)
    );

    // The existing chat router now finds the guild because `chat_session` derives it from
    // the store. The claimed guild id must match the authenticated membership.
    let chat = social_chat::encode_packet_chat(&social_chat::PacketChat {
        channel: social_chat::ChatChannel::Guild,
        sender_guid: 0,
        target_guid: 0,
        sender_name: NameData::default(),
        target_name: NameData::default(),
        message: "synthetic line".to_owned(),
        position: [0.0; 4],
        guild_guid: guild,
        language: 0,
        area_id: None,
    })
    .unwrap();
    let deliveries = route(&mut rig, 1, &chat);
    let mut recipients: Vec<u64> = deliveries.iter().map(|d| d.recipient.0).collect();
    recipients.sort();
    assert_eq!(recipients, vec![17, 18]);
    let forged = route(&mut rig, 1, &chat_with_guild(&chat, guild, guild + 1));
    assert!(
        forged.iter().all(|d| d.recipient != SessionId(17)),
        "a forged guild id must not reach another member"
    );

    // World entry (join_player -> guild_login) publishes the roster, and the second entry
    // sees the first member online; disconnect (end_session -> guild_logout) clears it.
    let first = rig.world.guild_login(&rig.sessions[0]);
    assert!(
        first
            .iter()
            .any(|d| d.recipient == SessionId(17) && family_sub(d).1 == 22)
    );
    let second = rig.world.guild_login(&rig.sessions[1]);
    assert!(
        second
            .iter()
            .any(|d| d.recipient == SessionId(18) && family_sub(d).1 == 17)
    );
    let offline = rig.world.guild_logout(GUIDS[0]);
    assert!(
        offline
            .iter()
            .any(|d| d.recipient == SessionId(18) && family_sub(d).1 == 17)
    );
}

#[test]
fn guild_request_refused_by_the_account_budget_changes_nothing() {
    let mut rig = rig("budget", true);
    let guild = rig.world.guilds.store().guild_guid_for(GUIDS[0]).unwrap();
    log_in(&mut rig.sessions[0], "t006-budget");
    let file = rig.options.state.with_file_name("guilds.json");
    let saved = fs::read(&file).unwrap();
    {
        let later = Instant::now() + Duration::from_secs(5);
        let account = rig.sessions[0].account_guid;
        let mut security = rig.world.security.lock().unwrap();
        while security.budgets.allow(account, &[15, 0, 1, 0], later) {}
    }
    rig.sessions[0].recent_on = true;
    rig.sessions[0].recent.clear();
    dispatch(&mut rig, 0, &motd(guild, "blocked"), 1_000);
    assert_eq!(rig.world.guilds.store().guild(guild).unwrap().motd, "");
    assert_eq!(fs::read(&file).unwrap(), saved);
    assert!(rig.sessions[0].recent.iter().all(|row| row["dir"] != "out"));
}

#[test]
fn a_guildless_member_request_is_silent_and_writes_nothing() {
    let mut rig = rig("guildless", true);
    log_in(&mut rig.sessions[1], "t006-guildless");
    rig.sessions[1].recent_on = true;
    rig.sessions[1].recent.clear();
    let file = rig.options.state.with_file_name("guilds.json");
    let saved = fs::read(&file).unwrap();
    // Quit and name-request both require a guild reference; the store must not be
    // touched (or cloned) and the player must stay guildless.
    dispatch(&mut rig, 1, &one_guid(7, 999), 1_000);
    dispatch(&mut rig, 1, &one_guid(14, 601), 1_100);
    assert_eq!(rig.world.guilds.store().guild_guid_for(GUIDS[1]), None);
    assert_eq!(fs::read(&file).unwrap(), saved);
    assert!(rig.sessions[1].recent.iter().all(|row| row["dir"] != "out"));
}

#[test]
fn a_guild_save_failure_rolls_back_the_staged_motd() {
    let mut rig = rig("save-failure", true);
    let guild = rig.world.guilds.store().guild_guid_for(GUIDS[0]).unwrap();
    log_in(&mut rig.sessions[0], "t006-save");
    let file = rig.options.state.with_file_name("guilds.json");
    fs::remove_file(&file).unwrap();
    fs::create_dir(&file).unwrap();
    let before = rig.world.guilds.store().guild(guild).unwrap().clone();
    assert!(dispatch_result(&mut rig, 0, &motd(guild, "lost"), 1_000).is_err());
    assert_eq!(rig.world.guilds.store().guild(guild).unwrap(), &before);
}

/// `PacketChat` ends with the guild guid and language (channel 9 has no area id).
fn chat_with_guild(chat: &[u8], original: u64, forged: u64) -> Vec<u8> {
    let mut bytes = chat.to_vec();
    let at = bytes.len() - 12;
    assert_eq!(
        u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap()),
        original
    );
    bytes[at..at + 8].copy_from_slice(&forged.to_le_bytes());
    bytes
}
