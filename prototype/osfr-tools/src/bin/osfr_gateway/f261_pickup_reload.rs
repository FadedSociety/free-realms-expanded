//! F261: every opcode-31 begin-zoning makes the client reload its scene, so
//! `Session::commit_outcome` passes that fact into `Gateway::bind_run_instance` and the
//! staged engine drops the old scene's ground pickups. The regression seeds a pickup
//! through the real drop roll, knocks the player out and sends the client's free revive
//! (41/122). The instance id does not change, so only the commit's own
//! `outcome.began_zoning` (client begin-zoning body reader `0x008e4ae0`) can clear the
//! drop. A tick on the revival spot afterwards must not pay the invisible heart.
//! `Session::dispatch_message`, `Session::tick` and `Session::commit_outcome` are the same
//! staging/commit/save path a socket datagram takes.
use super::test_support::{self, Scratch};
use super::*;

/// A logged-in session; `[13,0]`/`[10,0]` drive world entry as ordinary commits.
fn enter(options: &Options, id: u32) -> Session {
    let mut session = Session::new(
        SessionId(u64::from(id)),
        format!("127.0.0.1:{}", 23_300 + id).parse().unwrap(),
        SessionRequest {
            id,
            crc_length: 3,
            buffer_size: 512,
        },
        options,
        CRC_SEED,
        None,
        None,
    )
    .unwrap();
    let mut login = 1u16.to_le_bytes().to_vec();
    for (index, text) in ["f261-synthetic-ticket", "1.910.1.530630", ""]
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

/// One session and the containers a `Server` owns, without a listening socket.
struct Scene {
    session: Session,
    world: SharedWorld,
    membership: World,
    sessions: SessionMap,
}

impl Scene {
    fn new(options: &Options, id: u32) -> Self {
        Self {
            session: enter(options, id),
            world: SharedWorld::new(),
            membership: World::runtime(),
            sessions: SessionMap::new(),
        }
    }

    fn send(&mut self, options: &Options, inner: &[u8], now: u64) {
        self.session
            .dispatch_message(
                &wrap(inner),
                options,
                now,
                &mut self.world,
                &mut self.membership,
                &mut self.sessions,
                None,
            )
            .unwrap();
    }

    fn tick(&mut self, options: &Options, now: u64) {
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

fn pickup_count(session: &Session) -> usize {
    session
        .gateway
        .gameplay()
        .expect("a content session owns gameplay")
        .pickup_count()
}

fn health(session: &Session) -> u32 {
    session
        .gateway
        .snapshot()
        .gameplay
        .unwrap()
        .encounter
        .unwrap()
        .player_health
}

/// The client's free revive: family 41/sub 122, header ids zero, paid = false
/// (`0093bd80` free, `0093be30` paid; body 12-byte header + bool).
fn free_revive() -> Vec<u8> {
    [vec![41, 0, 122, 0], vec![0; 8], vec![0]].concat()
}

/// Hurt the player without knocking them out, so a collected big heart is visible.
fn wound(scene: &mut Scene, now: u64) -> (u32, u32) {
    let full = health(&scene.session);
    scene
        .session
        .gateway
        .receive_field_damage(full / 2, now, true)
        .unwrap();
    (full, health(&scene.session))
}

#[test]
fn f261_the_commit_clears_a_revive_reloads_ground_pickups() {
    // The gateway source is embedded by integration harnesses, which compile this
    // module too; the scenario runs once in the binary's own test target.
    if option_env!("CARGO_BIN_NAME").is_none() {
        return;
    }
    let scratch = Scratch::new("f261-pickup");
    let options = test_support::options(scratch.state());
    let mut scene = Scene::new(&options, 261);
    scene.send(&options, &[13, 0], 1_000);
    scene.send(&options, &[10, 0], 1_001);
    assert!(scene.session.gateway.is_world_loaded());
    // The free revive returns here (the world checkpoint from encounters.json).
    let checkpoint = scene.session.gateway.state().position;

    // Positive control: the tick pays a heart under the checkpoint, so "does not
    // heal" below is an assertion about the reload, not about an inert tick.
    scene
        .session
        .gateway
        .seed_power_up_drop("boss", checkpoint, 2_000);
    assert_eq!(
        pickup_count(&scene.session),
        1,
        "the boss table drops a pickup"
    );
    let (full, wounded) = wound(&mut scene, 2_100);
    assert!(wounded < full);
    scene.tick(&options, 2_200);
    assert!(
        health(&scene.session) > wounded,
        "an ordinary tick collects the heart under the player"
    );
    assert_eq!(pickup_count(&scene.session), 0);

    // A heart left from before the revive lies where the client comes back.
    scene
        .session
        .gateway
        .seed_power_up_drop("boss", checkpoint, 3_000);
    assert_eq!(pickup_count(&scene.session), 1);
    scene
        .session
        .gateway
        .receive_field_damage(full, 3_100, true)
        .unwrap();
    assert_eq!(health(&scene.session), 0, "the knockout holds");
    scene.send(&options, &free_revive(), 3_200);
    assert_eq!(health(&scene.session), full, "the free revive works");
    assert_eq!(
        pickup_count(&scene.session),
        0,
        "a begin-zoning commit drops the old scene's ground pickups even into the same instance"
    );

    // The client reloaded its scene; the ready handshake follows, then the tick on
    // the spot must not pay the heart the client no longer shows.
    scene.send(&options, &[13, 0], 3_300);
    scene.send(&options, &[10, 0], 3_301);
    let (full, wounded) = wound(&mut scene, 3_400);
    assert!(wounded < full);
    scene.tick(&options, 3_500);
    assert_eq!(
        health(&scene.session),
        wounded,
        "the stale big heart must not heal after the reload"
    );
    assert_eq!(pickup_count(&scene.session), 0);
}
