use super::*;
use fixtures::{copy_tree, fish_option, npc, wrap};
use osfr_tools::game_store::Snapshot;
use osfr_tools::gateway::Gateway;
use serde_json::{Value, json};

#[path = "fishing_fixtures.rs"]
mod fixtures;

const NOW: u64 = 10_000;
const REED: u64 = 9_000_000_181;
const GATE_POSITION: [f32; 3] = [-1319.765, -65.62, 898.9357];
const ACTOR: osfr_tools::fishing::Handle = osfr_tools::fishing::Handle {
    id: 0x1122_3344,
    seq: 0x5566_7788,
};

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn login() -> Vec<u8> {
    fixtures::login("f120-fixture")
}

fn lifecycle(instance: i32) -> Vec<u8> {
    let mut bytes = vec![39, 0, 5];
    bytes.extend(instance.to_le_bytes());
    bytes.extend((-1i32).to_le_bytes());
    bytes.extend((-1i32).to_le_bytes());
    bytes
}

fn request(sub: u16, body: &[u8]) -> Vec<u8> {
    let mut bytes = [138u16.to_le_bytes(), sub.to_le_bytes()].concat();
    bytes.extend(body);
    bytes
}

fn register() -> Vec<u8> {
    let mut body = [ACTOR.id.to_le_bytes(), ACTOR.seq.to_le_bytes()].concat();
    body.push(0);
    request(2, &body)
}

fn cast() -> Vec<u8> {
    let mut body = [ACTOR.id.to_le_bytes(), ACTOR.seq.to_le_bytes()].concat();
    for value in [
        GATE_POSITION[0] + 10.0,
        GATE_POSITION[1],
        GATE_POSITION[2],
        1.0,
    ] {
        body.extend(value.to_le_bytes());
    }
    body.push(0);
    request(6, &body)
}

fn reel() -> Vec<u8> {
    let mut body = [ACTOR.id.to_le_bytes(), ACTOR.seq.to_le_bytes()].concat();
    body.push(1);
    request(7, &body)
}

fn menu(gateway: &mut Gateway, target: u64, now: u64) -> Vec<Vec<u8>> {
    gateway
        .handle_at(&wrap(&npc(8, target, None)), now)
        .unwrap()
        .replies
        .into_iter()
        .map(|packet| packet[7..].to_vec())
        .collect()
}

fn send(
    session: &mut Session,
    options: &Options,
    world: &mut SharedWorld,
    membership: &mut World,
    sessions: &mut SessionMap,
    message: &[u8],
    now: u64,
) {
    session
        .dispatch_message(
            &wrap(message),
            options,
            now,
            world,
            membership,
            sessions,
            None,
        )
        .unwrap();
}

fn send_raw(
    session: &mut Session,
    options: &Options,
    world: &mut SharedWorld,
    membership: &mut World,
    sessions: &mut SessionMap,
    message: &[u8],
    now: u64,
) {
    session
        .dispatch_message(message, options, now, world, membership, sessions, None)
        .unwrap();
}

fn coins(snapshot: &Snapshot) -> u32 {
    snapshot.gameplay.as_ref().unwrap().inventory.coins
}

fn fisherman_xp(snapshot: &Snapshot) -> u32 {
    snapshot
        .gameplay
        .as_ref()
        .unwrap()
        .profiles
        .as_ref()
        .unwrap()
        .experience
        .get(&137)
        .copied()
        .unwrap_or(0)
}

fn try_catch(
    session: &mut Session,
    options: &Options,
    world: &mut SharedWorld,
    membership: &mut World,
    sessions: &mut SessionMap,
    at: u64,
) -> (GatewayOutcome, u64) {
    send(
        session,
        options,
        world,
        membership,
        sessions,
        &cast(),
        at + 10,
    );
    session.gateway.tick_at(at + 1_600).unwrap();
    session.gateway.tick_at(at + 2_600).unwrap();
    let mut probe = session.gateway.clone();
    let outcome = probe.handle_at(&wrap(&reel()), at + 2_700).unwrap();
    send(
        session,
        options,
        world,
        membership,
        sessions,
        &reel(),
        at + 2_700,
    );
    (outcome, at + 2_700)
}

#[test]
fn an_hour_cap_survives_reconnect_save_reload_and_a_blocked_catch_writes_nothing() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
    let content_source = std::env::var_os("OSFR_OPENING_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("references/rust-opening-cooking-original-assets"));
    let reference = std::env::var_os("OSFR_GATEWAY_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("references/rust-gateway-reference"));
    assert!(
        content_source.join("inventory.json").is_file(),
        "private opening fixture is missing"
    );
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let scratch = Scratch(
        std::env::temp_dir().join(format!("osfr-f120-fishing-{}-{nonce}", std::process::id())),
    );
    fs::create_dir_all(&scratch.0).unwrap();
    let content = scratch.0.join("content");
    copy_tree(&content_source, &content);
    let mut spots: Value =
        serde_json::from_str(include_str!("../../../data/content/fishing_spots.json")).unwrap();
    spots["catch_cap"]["enabled"] = json!(true);
    spots["catch_cap"]["window"] = json!("hour");
    spots["catch_cap"]["max_paid_catches"] = json!(1);
    for row in spots["payout"]["by_grade"].as_array_mut().unwrap() {
        row["coins"] = json!(25);
    }
    // Use one existing stackable item for every synthetic fixture size so the private
    // opening inventory can contain the catch without importing the full item catalogue.
    for sizes in spots["items"].as_object_mut().unwrap().values_mut() {
        *sizes = json!([68_001, 0, 0, 0]);
    }
    fs::write(
        content.join("fishing_spots.json"),
        serde_json::to_vec(&spots).unwrap(),
    )
    .unwrap();

    let state_path = scratch.0.join("character.json");
    let mut character = Gateway::reference_state(&reference).unwrap();
    character.position = GATE_POSITION;
    let initial = Gateway::load_gameplay(&reference, character, &content, None).unwrap();
    save(&state_path, &initial.snapshot()).unwrap();
    let options = Options {
        bind: "127.0.0.1:0".parse().unwrap(),
        reference: reference.clone(),
        state: state_path.clone(),
        content: Some(content.clone()),
        identity: None,
        account_store: None,
        gateway_ticket: login::DEFAULT_GATEWAY_TICKET.into(),
        handoff_key: Vec::new(),
        seconds: 1,
        dev_tools: false,
        studio_port: 0,
        client_logs: None,
        terrain: None,
        world: Some(scratch.0.join("world")),
        stock_text: None,
        allow_lan: false,
        security_log: None,
        group_invite_ttl_ms: osfr_tools::social_lists::DEFAULT_GROUP_INVITE_TTL_MS,
    };
    let new_session = |id: u64| {
        Session::new(
            SessionId(id),
            format!("127.0.0.1:{}", 30_000 + id).parse().unwrap(),
            SessionRequest {
                id: id as u32,
                crc_length: 3,
                buffer_size: 512,
            },
            &options,
            CRC_SEED,
            None,
            None,
        )
        .unwrap()
    };
    let mut world = SharedWorld::new();
    let mut membership = World::runtime();
    let mut sessions = SessionMap::new();
    let mut first = new_session(41);
    send_raw(
        &mut first,
        &options,
        &mut world,
        &mut membership,
        &mut sessions,
        &login(),
        0,
    );
    send(
        &mut first,
        &options,
        &mut world,
        &mut membership,
        &mut sessions,
        &[13, 0],
        100,
    );
    send(
        &mut first,
        &options,
        &mut world,
        &mut membership,
        &mut sessions,
        &[10, 0],
        200,
    );
    let option = fish_option(&menu(&mut first.gateway, REED, 300), 73_552).expect("fishing option");
    send(
        &mut first,
        &options,
        &mut world,
        &mut membership,
        &mut sessions,
        &npc(10, REED, Some(option)),
        400,
    );
    send(
        &mut first,
        &options,
        &mut world,
        &mut membership,
        &mut sessions,
        &lifecycle(1),
        500,
    );
    send(
        &mut first,
        &options,
        &mut world,
        &mut membership,
        &mut sessions,
        &register(),
        600,
    );
    assert_eq!(coins(&first.gateway.snapshot()), 0);
    assert_eq!(fisherman_xp(&first.gateway.snapshot()), 0);

    let mut at = NOW;
    let caught_at = loop {
        let (outcome, when) = try_catch(
            &mut first,
            &options,
            &mut world,
            &mut membership,
            &mut sessions,
            at,
        );
        if outcome.state_changed {
            break when;
        }
        at = when + 10_000;
        assert!(at < NOW + 3_000_000, "no deterministic catch before expiry");
    };
    let paid = first.gateway.snapshot();
    assert_eq!(coins(&paid), 25);
    assert_eq!(fisherman_xp(&paid), 10);
    let history = first.gateway.fishing_hour_history().unwrap();
    assert_eq!(history, vec![caught_at]);
    assert_eq!(world.fishing_hours.history(first.character_guid), history);
    // Lead (security): a handshake session that never logged in still carries the template
    // guid, which equals this character's. Its tick must not erase the hour-cap ledger.
    let mut stranger = new_session(77);
    assert_eq!(stranger.gateway.state().guid, first.character_guid);
    // A non-empty forged history: only the owner check (not the empty-history rule) stops it.
    stranger
        .gateway
        .restore_fishing_hours(vec![caught_at + 400])
        .unwrap();
    stranger
        .tick(
            &options,
            caught_at + 500,
            &mut world,
            &mut membership,
            &mut sessions,
        )
        .unwrap();
    assert_eq!(
        world.fishing_hours.history(first.character_guid),
        history,
        "an unauthenticated tick erased the fishing-hour ledger"
    );
    let saved_after_payout = fs::read(&state_path).unwrap();
    let guid = first.character_guid;
    world.leave_player(first.id);
    world.active_sessions.remove(&guid);
    membership.leave(guid, caught_at + 1_000);
    drop(first);

    let mut second = new_session(42);
    // The reconnect session ticks before its login message. That tick must not erase or replace
    // the ledger the login will restore (finding 1's failing path).
    second
        .tick(
            &options,
            caught_at + 1_000,
            &mut world,
            &mut membership,
            &mut sessions,
        )
        .unwrap();
    assert_eq!(world.fishing_hours.history(guid), history);
    send_raw(
        &mut second,
        &options,
        &mut world,
        &mut membership,
        &mut sessions,
        &login(),
        caught_at + 1_000,
    );
    send(
        &mut second,
        &options,
        &mut world,
        &mut membership,
        &mut sessions,
        &[13, 0],
        caught_at + 1_100,
    );
    send(
        &mut second,
        &options,
        &mut world,
        &mut membership,
        &mut sessions,
        &[10, 0],
        caught_at + 1_200,
    );
    assert_eq!(second.gateway.fishing_hour_history().unwrap(), history);
    assert_eq!(second.gateway.snapshot(), paid);
    assert_eq!(fs::read(&state_path).unwrap(), saved_after_payout);

    let option = fish_option(&menu(&mut second.gateway, REED, caught_at + 1_300), 73_552)
        .expect("fishing option after reload");
    send(
        &mut second,
        &options,
        &mut world,
        &mut membership,
        &mut sessions,
        &npc(10, REED, Some(option)),
        caught_at + 1_400,
    );
    send(
        &mut second,
        &options,
        &mut world,
        &mut membership,
        &mut sessions,
        &lifecycle(1),
        caught_at + 1_500,
    );
    send(
        &mut second,
        &options,
        &mut world,
        &mut membership,
        &mut sessions,
        &register(),
        caught_at + 1_600,
    );
    let before_refusal = second.gateway.snapshot();
    let before_refusal_file = fs::read(&state_path).unwrap();
    let (blocked, blocked_at) = try_catch(
        &mut second,
        &options,
        &mut world,
        &mut membership,
        &mut sessions,
        caught_at + 60_000,
    );
    assert!(!blocked.state_changed, "a capped catch requests no save");
    assert!(!blocked.gameplay_changed);
    let replies = blocked.replies.concat();
    assert!(
        String::from_utf8_lossy(&replies).contains("reward limit"),
        "the capped catch takes the refusal path"
    );
    assert_eq!(second.gateway.snapshot(), before_refusal);
    assert_eq!(coins(&second.gateway.snapshot()), 25);
    assert_eq!(fisherman_xp(&second.gateway.snapshot()), 10);
    assert_eq!(fs::read(&state_path).unwrap(), before_refusal_file);
    assert_eq!(second.gateway.fishing_hour_history().unwrap(), history);
    assert_eq!(world.fishing_hours.history(guid), history);

    let mut after_expiry = caught_at + 3_602_000;
    let paid_again = loop {
        let (outcome, when) = try_catch(
            &mut second,
            &options,
            &mut world,
            &mut membership,
            &mut sessions,
            after_expiry,
        );
        if outcome.state_changed {
            break when;
        }
        after_expiry = when + 10_000;
        assert!(
            after_expiry < blocked_at + 3_700_000,
            "no catch after the hour window expired"
        );
    };
    assert!(paid_again > caught_at + 3_600_000);
    assert_eq!(coins(&second.gateway.snapshot()), 50);
    assert_eq!(fisherman_xp(&second.gateway.snapshot()), 20);
    assert_eq!(
        second.gateway.fishing_hour_history().unwrap(),
        vec![paid_again]
    );
    assert_eq!(world.fishing_hours.history(guid), vec![paid_again]);
    // An idle owner's tick after the window closes must drop the expired timestamp rather than
    // republish it from the session's in-memory history (finding 5).
    second
        .tick(
            &options,
            paid_again + osfr_tools::fishing_service::HOUR_WINDOW_MS + 1,
            &mut world,
            &mut membership,
            &mut sessions,
        )
        .unwrap();
    assert!(
        world.fishing_hours.history(guid).is_empty(),
        "an expired history was republished"
    );
    assert_ne!(fs::read(&state_path).unwrap(), before_refusal_file);
}
