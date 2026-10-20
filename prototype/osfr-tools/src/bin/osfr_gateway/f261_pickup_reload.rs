//! F261: every opcode-31 begin-zoning makes the client reload its scene, so
//! `Session::commit_outcome` passes that fact into `Gateway::bind_run_instance` and the
//! staged engine drops the old scene's ground pickups. The regression seeds the drop the
//! way the server does: a real 36/10 basic attack kills a content field enemy whose
//! `power_up_table` is the shipped guaranteed `boss` table, so the ground pickup is the
//! server's own roll and no test-only seed API exists. It then knocks the player out and
//! sends the client's free revive (41/122). The instance id does not change, so only the
//! commit's own `outcome.began_zoning` (client begin-zoning body reader `0x008e4ae0`) can
//! clear the drop. A fresh kill after the reload is still paid on the same spot, so the
//! "does not heal" assertion is about the reload, not an inert tick.
//! `Session::dispatch_message`, `Session::tick` and `Session::commit_outcome` are the same
//! staging/commit/save path a socket datagram takes.
use super::test_support::{self, Scene, Scratch, bin_test_target};
use super::*;
use osfr_tools::item_bar::use_wire::{Target, encode_client_request_start_ability};

/// The copied campaign plus two synthetic field enemies on the world checkpoint, each rolling
/// the shipped guaranteed `boss` table. `vendor_content` supplies the `npcs/` +
/// `npc_spawns_extra.json` marker, so the session loads this directory's field and drop tables
/// instead of the bundled `data/content` ones.
struct DropContent {
    path: PathBuf,
    at: [f32; 3],
}

fn drop_content(scratch: &Scratch, options: &Options) -> DropContent {
    let path = test_support::vendor_content(scratch);
    let initial = Gateway::reference_state(&options.reference).unwrap();
    // The same placement production uses: the copied encounters.json world position.
    let at = Gameplay::initial_character(&path, initial)
        .unwrap()
        .position;
    let enemy = |guid: u64| {
        json!({
            "guid": guid,
            "definition_id": 400,
            "model_id": 3,
            "name_id": 100,
            "name": "Synthetic wolf",
            "texture_alias": "",
            "position": at,
            "max_health": 1,
            "damage": 0,
            "boss": false,
            "power_up_table": "boss",
            "respawn_seconds": 3_600,
            "aggro_range": 8
        })
    };
    let enemies = json!({"schema_version": 1, "enemies": [enemy(9_001), enemy(9_002)]});
    fs::write(
        path.join("field_enemies.json"),
        serde_json::to_vec(&enemies).unwrap(),
    )
    .unwrap();
    fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/power_ups.json"),
        path.join("power_ups.json"),
    )
    .unwrap();
    DropContent { path, at }
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

fn active_profile(session: &Session) -> i32 {
    session
        .gateway
        .snapshot()
        .gameplay
        .unwrap()
        .profiles
        .unwrap()
        .active
}

/// The world instance the player's membership row names, so the revive can prove it stayed.
fn world_instance(scene: &Scene) -> i32 {
    scene
        .membership
        .player_location(scene.session.gateway.state().guid)
        .expect("the session is in the world membership")
        .instance_id
}

/// The client's 26/13 class selection: `PlayClient::ChangeProfile` (`0x00938e10`) builds
/// `CommandPacketSetProfile` (ctor `0x008a29f0`, family 26 / sub 13) and writes the one `i32`
/// profile id at packet `+0x10`. The shared command writer `0x0090ac70` serializes family and
/// sub as two `u16`s only, and the payload starts at `+0x10` (the 26/24 writer `0x00c21290`
/// writes its dword the same way), so the body is 8 bytes.
fn class_selection(job: i32) -> Vec<u8> {
    [vec![26, 0, 13, 0], job.to_le_bytes().to_vec()].concat()
}

/// The client's 36/10 targeted basic attack: bar 1, slot 0, one target GUID. The canonical
/// codec is the same one the server parses (client writer `0x00a342e0`).
fn basic_attack(target: u64) -> Vec<u8> {
    encode_client_request_start_ability(1, 0, Target::Guid(target))
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
    if !bin_test_target("f261") {
        return;
    }
    let scratch = Scratch::new("f261-pickup");
    let mut options = test_support::options(scratch.state());
    let content = drop_content(&scratch, &options);
    options.content = Some(content.path.clone());

    // The same seed `Server::new` installs before any session exists.
    let mut seed = load_gateway_template(&options).unwrap();
    let field = seed.take_shared_fields().expect("the field enemies load");
    let guids: Vec<u64> = field
        .interest_candidates(false)
        .iter()
        .map(|npc| npc.guid)
        .collect();
    assert_eq!(guids.len(), 2, "both fixture enemies are on the field");
    let (first, second) = (guids[0], guids[1]);

    let mut scene = Scene::with_field(&options, 261, field);
    scene.send(&options, &[13, 0], 1_000).unwrap();
    scene.send(&options, &[10, 0], 1_001).unwrap();
    assert!(scene.session.gateway.is_world_loaded());
    // The free revive returns to this checkpoint, where both enemies stand.
    assert_eq!(
        scene.session.gateway.state().position,
        content.at,
        "world entry uses the content checkpoint"
    );
    let instance = world_instance(&scene);

    // Brawler's equipped starter weapon is the basic attack this scenario uses.
    scene.send(&options, &class_selection(43), 1_100).unwrap();
    assert_eq!(
        active_profile(&scene.session),
        43,
        "the class selection is accepted"
    );

    // The server's own kill: the guaranteed boss table drops one ground heart.
    scene.session.known_npcs.insert(first);
    scene.send(&options, &basic_attack(first), 2_000).unwrap();
    assert_eq!(
        pickup_count(&scene.session),
        1,
        "the boss table drops a pickup"
    );

    let full = health(&scene.session);
    scene
        .session
        .gateway
        .receive_field_damage(full, 2_100, true)
        .unwrap();
    assert_eq!(health(&scene.session), 0, "the knockout holds");
    scene.send(&options, &free_revive(), 2_200).unwrap();
    assert_eq!(health(&scene.session), full, "the free revive works");
    assert_eq!(
        world_instance(&scene),
        instance,
        "the free revive stays in the same world instance"
    );
    assert_eq!(
        pickup_count(&scene.session),
        0,
        "a begin-zoning commit drops the old scene's ground pickups even into the same instance"
    );

    // The client reloaded its scene and sends the ready handshake. Wound the player first:
    // at full health a surviving big heart would heal zero and hide itself, but a wounded
    // player would be healed by it.
    scene.send(&options, &[13, 0], 2_300).unwrap();
    scene.send(&options, &[10, 0], 2_301).unwrap();
    let (full, wounded) = wound(&mut scene, 2_350);
    assert!(wounded < full, "the player is wounded before the tick");
    scene.tick(&options, 2_400);
    assert_eq!(
        health(&scene.session),
        wounded,
        "the stale big heart must not heal after the reload"
    );

    // Positive control: a fresh kill after the reload is paid by the same spot tick.
    scene.session.known_npcs.insert(second);
    scene.send(&options, &basic_attack(second), 3_000).unwrap();
    assert_eq!(
        pickup_count(&scene.session),
        1,
        "the live heart is on the ground"
    );
    let (full, wounded) = wound(&mut scene, 3_100);
    assert!(wounded < full);
    scene.tick(&options, 3_200);
    assert!(
        health(&scene.session) > wounded,
        "an ordinary tick collects the heart under the player"
    );
    assert_eq!(pickup_count(&scene.session), 0);
}
