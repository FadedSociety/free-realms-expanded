//! T478: a style card through the gateway's real dispatch and commit path (`Session::
//! dispatch_message`): the player and the viewers in range get one 35/39, viewers out of range
//! none, a player who arrives later gets the new look in AddPc, and a failed character save
//! changes nothing and ends only that session. Cards and strings are the real catalogue rows and
//! client tables; each session has its own reference guid, as in T199.
use super::super::f120_fishing_hour_reconnect::drain_replies;
use super::super::t199_world_notice::{login, reference};
use super::test_support::{self, bin_test_target};
use super::*;
use osfr_tools::{game_store::Snapshot, gameplay::Gameplay, gateway::Gateway};

#[path = "../../../tests/common/style_card_fixture.rs"]
#[allow(dead_code)]
mod fixture;
#[path = "../../../tests/common/round70_content.rs"]
mod round70_content;
use fixture::*;

const NOW: u64 = 1_000;
const CARD: i32 = 900_001;
const BEARD_STRING: &[u8] = b"human_m_facialhair_beard.adr";

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

/// Where a session sits and what it carries.
#[derive(Clone, Copy)]
enum Seat {
    /// The player: gameplay content, the bearded boy's identity, the tables and one card.
    Player,
    /// No gameplay, at the player's spawn point: in range.
    Near,
    /// No gameplay, a kilometre away: out of range.
    Far,
}

struct Rig {
    scratch: PathBuf,
    content: PathBuf,
    world: SharedWorld,
    membership: World,
    sessions: SessionMap,
    keys: Vec<SessionKey>,
    options: Vec<Options>,
}

impl Drop for Rig {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.scratch);
    }
}

impl Rig {
    fn new(name: &str) -> Self {
        let scratch = std::env::temp_dir().join(format!("osfr-t478-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&scratch);
        fs::create_dir_all(&scratch).unwrap();
        Self {
            content: round70_content::build(&scratch, "content", &[]),
            scratch,
            world: SharedWorld::new(),
            membership: World::runtime(),
            sessions: SessionMap::new(),
            keys: Vec::new(),
            options: Vec::new(),
        }
    }

    /// Log a character in and send ready and loaded, like a client entering the world. `save`
    /// `earlier` is a previous save of this character, for a relog.
    fn add(&mut self, guid: u64, seat: Seat, earlier: Option<&[u8]>) -> usize {
        let id = 61 + self.keys.len() as u64;
        let mut options =
            test_support::options(self.scratch.join(format!("character-{guid}.json")));
        options.reference = reference(&self.scratch, guid);
        let mut character = Gateway::reference_state(&options.reference).unwrap();
        match seat {
            Seat::Player => {
                options.content = Some(self.content.clone());
                options.resources = Some(tables());
                options.identity = Some(self.scratch.join("identity.json"));
                osfr_tools::identity::save(options.identity.as_ref().unwrap(), &boy()).unwrap();
                let mut state = Gameplay::load(&self.content, None).unwrap().state();
                state.inventory.items.push(card_item(CARD, CLEAN_SHAVEN, 1));
                state.inventory.next_instance_id = Some(i64::from(CARD) + 1);
                test_support::save_character(&options.state, &character, &state);
            }
            Seat::Near | Seat::Far => {
                options.content = None;
                if matches!(seat, Seat::Far) {
                    character.position[0] += 1_000.0;
                }
                let snapshot = Snapshot {
                    character,
                    gameplay: None,
                    pets: None,
                };
                save(&options.state, &snapshot).unwrap();
            }
        }
        if let Some(bytes) = earlier {
            fs::write(&options.state, bytes).unwrap();
        }
        let mut session = Session::new(
            SessionId(id),
            format!("127.0.0.1:{}", 42_000 + id).parse().unwrap(),
            SessionRequest {
                id: id as u32,
                crc_length: 3,
                buffer_size: 1400,
            },
            &options,
            CRC_SEED,
            None,
            None,
        )
        .unwrap();
        let key = SessionKey {
            peer: session.peer,
            connection_id: session.request.id,
        };
        self.world.session_keys.insert(session.id, key);
        for (message, now) in [
            (login(guid), 0u64),
            (wrap(&[13, 0]), 100),
            (wrap(&[10, 0]), 200),
        ] {
            session
                .dispatch_message(
                    &message,
                    &options,
                    now,
                    &mut self.world,
                    &mut self.membership,
                    &mut self.sessions,
                    None,
                )
                .unwrap();
        }
        self.keys.push(key);
        self.options.push(options);
        self.sessions.insert(key, session);
        self.keys.len() - 1
    }

    fn try_send(&mut self, index: usize, inner: &[u8], now: u64) -> Result<(), String> {
        let key = self.keys[index];
        let mut session = self.sessions.remove(&key).unwrap();
        let result = session
            .dispatch_message(
                &wrap(inner),
                &self.options[index],
                now,
                &mut self.world,
                &mut self.membership,
                &mut self.sessions,
                None,
            )
            .map_err(|error| error.to_string());
        self.sessions.insert(key, session);
        result
    }

    fn drain(&mut self, index: usize) -> Vec<Vec<u8>> {
        let key = self.keys[index];
        let mut session = self.sessions.remove(&key).unwrap();
        let messages = drain_replies(&mut session, NOW);
        self.sessions.insert(key, session);
        messages
    }

    fn save(&self, index: usize) -> Vec<u8> {
        fs::read(&self.options[index].state).unwrap()
    }

    fn snapshot(&self, index: usize) -> Snapshot {
        self.sessions[&self.keys[index]].gateway.snapshot()
    }
}

/// The messages whose application body starts with `prefix` (after the 5/1 tunnel header).
fn of<'a>(messages: &'a [Vec<u8>], prefix: &[u8]) -> Vec<&'a [u8]> {
    messages
        .iter()
        .filter(|message| message.len() > 7 && message[7..].starts_with(prefix))
        .map(|message| &message[7..])
        .collect()
}

/// The AddPc (35/1) a session received for `guid`.
fn add_pc<'a>(messages: &'a [Vec<u8>], guid: u64) -> Vec<&'a [u8]> {
    of(messages, &[35, 0, 1, 0])
        .into_iter()
        .filter(|inner| inner[4..12] == guid.to_le_bytes())
        .collect()
}

fn has(bytes: &[u8], needle: &[u8]) -> bool {
    bytes.windows(needle.len()).any(|window| window == needle)
}

#[test]
fn nearby_player_sees_style_card_change() {
    if !bin_test_target("t478") {
        return;
    }
    let mut rig = Rig::new("nearby");
    let player = rig.add(1, Seat::Player, None);
    let (viewer, far) = (rig.add(2, Seat::Near, None), rig.add(3, Seat::Far, None));
    // The viewer met the bearded player when it entered.
    let entered = rig.drain(viewer);
    let before = add_pc(&entered, 1);
    assert_eq!(before.len(), 1, "one AddPc for the player");
    assert!(has(before[0], BEARD_STRING), "the AddPc shows the beard");
    for index in 0..3 {
        rig.drain(index);
    }

    rig.try_send(player, &request(10, CARD), NOW).unwrap();

    let own = rig.drain(player);
    let changes = of(&own, &[35, 0, 39, 0]);
    assert_eq!(changes.len(), 1, "the player gets one 35/39");
    assert_eq!(changes[0][4..12], 1u64.to_le_bytes(), "naming the player");
    assert_eq!(changes[0][12], 0, "preview off");
    assert_eq!(of(&own, &[38, 0, 4, 0]).len(), 1, "and the card's delete");
    let seen = rig.drain(viewer);
    assert_eq!(
        of(&seen, &[35, 0, 39, 0]),
        changes,
        "the viewer gets the same 35/39 once"
    );
    assert!(
        rig.drain(far).is_empty(),
        "a player out of range hears nothing"
    );

    // Someone who arrives after the change gets the new look in AddPc.
    let late = rig.add(4, Seat::Near, None);
    let arrived = rig.drain(late);
    let after = add_pc(&arrived, 1);
    assert_eq!(after.len(), 1);
    assert!(!has(after[0], BEARD_STRING), "the late AddPc has no beard");
    assert!(
        has(after[0], b"skintone3"),
        "the rest of the look is the same"
    );
}

#[test]
fn style_card_save_failure_changes_nothing() {
    if !bin_test_target("t478") {
        return;
    }
    let mut rig = Rig::new("save-failure");
    let player = rig.add(1, Seat::Player, None);
    let viewer = rig.add(2, Seat::Near, None);
    for index in 0..2 {
        rig.drain(index);
    }
    let (snapshot, file) = (rig.snapshot(player), rig.save(player));
    let key = rig.keys[player];
    // The save path becomes a path under a file, so the staged commit's save cannot be written.
    rig.sessions.get_mut(&key).unwrap().state_path = rig.options[player].state.join("blocked.json");

    let error = rig.try_send(player, &request(10, CARD), NOW).unwrap_err();
    assert!(
        error.contains("os error"),
        "the commit failed in the save: {error}"
    );
    assert!(
        of(&rig.drain(player), &[35, 0, 39, 0]).is_empty(),
        "no look change reached the player"
    );
    assert!(rig.drain(viewer).is_empty(), "nor the viewer");
    assert_eq!(
        rig.snapshot(player),
        snapshot,
        "the card and the look are unchanged"
    );
    assert_eq!(rig.save(player), file, "so is the file");
    let state = snapshot.gameplay.unwrap();
    assert!(state.appearance.is_none());
    assert!(
        state
            .inventory
            .items
            .iter()
            .any(|item| item.id == CARD && item.count == 1)
    );
}

#[test]
fn a_relogged_player_shows_the_saved_look_to_viewers() {
    if !bin_test_target("t478") {
        return;
    }
    let mut first = Rig::new("relog-a");
    let player = first.add(1, Seat::Player, None);
    first
        .try_send(player, &request(12, CLEAN_SHAVEN), NOW)
        .unwrap();
    let saved = first.save(player);
    assert!(
        has(&saved, b"\"appearance\""),
        "the look is in the character save"
    );

    let mut second = Rig::new("relog-b");
    let viewer = second.add(2, Seat::Near, None);
    second.drain(viewer);
    second.add(1, Seat::Player, Some(&saved));
    let met = second.drain(viewer);
    let pc = add_pc(&met, 1);
    assert_eq!(pc.len(), 1);
    assert!(
        !has(pc[0], BEARD_STRING),
        "AddPc after relog shows the saved look"
    );
}

/// A viewer in another zone instance, even at the same spot, gets no 35/39: the fanout only asks
/// the player's own instance, and that viewer no longer knows the player.
#[test]
fn a_viewer_in_another_instance_hears_nothing() {
    if !bin_test_target("t478") {
        return;
    }
    let mut rig = Rig::new("other-instance");
    let player = rig.add(1, Seat::Player, None);
    let (same, other) = (rig.add(2, Seat::Near, None), rig.add(3, Seat::Near, None));
    // Move the third viewer into the next instance of the same zone, at the same position.
    let id = rig.sessions[&rig.keys[other]].id;
    let here = rig.world.players.get(id).unwrap().clone();
    let elsewhere = ZoneInstance {
        instance_id: here.zone.instance_id + 1,
        ..here.zone
    };
    rig.world
        .players
        .update_pose(id, elsewhere, here.position, here.heading)
        .unwrap();
    for index in 0..3 {
        rig.drain(index);
    }

    rig.try_send(player, &request(10, CARD), NOW).unwrap();

    assert_eq!(
        of(&rig.drain(same), &[35, 0, 39, 0]).len(),
        1,
        "a viewer in the player's instance gets it"
    );
    assert!(
        rig.drain(other).is_empty(),
        "a viewer in another instance hears nothing"
    );
}

/// Only viewers who already met the player may be told of the change (a stranger would get a
/// 35/39 for a guid it has no AddPc for): the fanout treats 35/39 like 35/6 and 35/7.
#[test]
fn a_look_change_is_a_live_outfit_packet() {
    use super::super::delivery::live_outfit_packet;
    assert!(live_outfit_packet(&[35, 0, 39, 0, 1]));
    assert!(!live_outfit_packet(&[35, 0, 65, 0]));
}

/// T520: the store try-on (42/11) goes to its sender only and changes nothing: no viewer
/// traffic, no card spent, no state for the commit to save, so the save file is byte-identical.
#[test]
fn a_preview_reaches_only_the_player_and_saves_nothing() {
    if !bin_test_target("t520") {
        return;
    }
    let mut rig = Rig::new("preview");
    let player = rig.add(1, Seat::Player, None);
    let viewer = rig.add(2, Seat::Near, None);
    for index in 0..2 {
        rig.drain(index);
    }
    let (snapshot, file) = (rig.snapshot(player), rig.save(player));

    rig.try_send(player, &request(11, SKULL_FACE_PAINT), NOW)
        .unwrap();

    let own = rig.drain(player);
    let previews = of(&own, &[35, 0, 39, 0]);
    assert_eq!(previews.len(), 1, "one preview to its sender");
    assert_eq!(previews[0][4..12], 1u64.to_le_bytes(), "naming the player");
    assert_eq!(previews[0][12], 1, "preview on");
    assert!(of(&own, &[38, 0, 4, 0]).is_empty(), "no card is spent");
    assert!(rig.drain(viewer).is_empty(), "the viewer hears nothing");
    assert_eq!(rig.snapshot(player), snapshot, "no state changed");
    assert_eq!(rig.save(player), file, "the save file is unchanged");
}
