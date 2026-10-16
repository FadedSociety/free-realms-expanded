//! Work item 060: dungeon instance lifecycle (`op41` encounter + `op39` settlement) tests.
//!
//! This file `#[path]`-includes the module so the real acceptance runs before the lead registers
//! `pub mod piece_060;` in `src/restoration`. Integration step for the lead: register the module and
//! drop the include.
//!
//! ORIGINAL vectors embedded here are three retail capture frames from
//! `lab/reports/original-minigame-packets/`, each re-hashed in the first test:
//!
//! * `demoderby-23087-flow0-frame5078-op41-sub2.bin`, 28 bytes, sha256
//!   `ecf5eda4a3dcd73ea1ab492ed676ad235880d4f32d9acdb5b80e31f68603e7bf` - a real
//!   `EncounterPacketPlayerEnter` carrying the 12-byte encounter header, a `u64` player guid and a
//!   trailing byte.
//! * `demoderby-22789-flow0-frame119-op41-sub106.bin`, 23 bytes, sha256
//!   `625e25eaf19c75cf2d157bcff9c2d6ca5d0b9ff2995155375f326f9fde354b91` - `EncounterStatePacket`
//!   at state `2` (the offer/loading step).
//! * `demoderby-22822-flow0-frame153-op41-sub107.bin`, 19 bytes, sha256
//!   `95c4e6234252b93af4c520607e2cb77802434e800d356632191f4b9b9661c3c3` - the bare-header
//!   `EncounterZoneIsReadyPacket`.
//!
//! A capture frame starts with the 7-byte tunnel envelope, so the tests compare the encoder output
//! against the frame tail. The `op39` settlement bodies have no captured frame (piece 016 records the
//! same gap); their layout is asserted as bytes here and evidenced in the module header.
//!
//! GAP this test does not close: the caller-supplied capacity and grace values are reconstructed, and
//! no captured dungeon instance manager traffic exists. Native acceptance stays with the lead.

#[path = "../src/restoration/piece_060.rs"]
#[allow(dead_code)]
mod piece_060;

use piece_060::{
    Config, Lifecycle, Member, Phase, Registry, Reject, STATE_GO, STATE_OFFER, STATE_READY,
    STATE_RUNNING, encode_encounter_state, encode_game_over, encode_player_enter,
    encode_state_remove, encode_zone_ready, required_combat_level,
};
use sha2::{Digest, Sha256};

const ENCOUNTER: i32 = 411;
const INSTANCE: i32 = 1_000_001;
const FIRST: u64 = 0x0000_0040_0000_0001;
const SECOND: u64 = 0x0000_0060_0000_0002;
const THIRD: u64 = 0x0000_0060_0000_0003;

/// The three original retail frames, verbatim, including the 7-byte tunnel envelope.
const CAPTURE_PLAYER_ENTER: &str = "050001150000002900020041001400f0030000000000000000000000";
const CAPTURE_STATE_OFFER: &str = "0500011000000029006a0041001400f003000002000000";
const CAPTURE_ZONE_READY: &str = "0500010c00000029006b0041001400f0030000";
/// The captured encounter/instance header values of `demoderby-23087-flow0-frame5078`.
const CAPTURE_ENCOUNTER: i32 = 0x0014_0041;
const CAPTURE_INSTANCE: i32 = 0x0000_03f0;

fn bytes(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn hex(data: &[u8]) -> String {
    data.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn config(capacity: usize) -> Config {
    Config {
        capacity,
        grace_ms: 30_000,
    }
}

/// A loading instance with one eligible member, ready to zone in.
fn ready_instance() -> Lifecycle {
    let mut life = Lifecycle::new(INSTANCE, ENCOUNTER, 3, config(4));
    life.enter(FIRST, 12).unwrap();
    life.zone_ready(FIRST).unwrap();
    life
}

#[test]
fn captured_op41_frames_re_encode_byte_for_byte() {
    for (name, frame, expected) in [
        (
            "op41 sub2 player enter",
            CAPTURE_PLAYER_ENTER,
            "ecf5eda4a3dcd73ea1ab492ed676ad235880d4f32d9acdb5b80e31f68603e7bf",
        ),
        (
            "op41 sub106 state",
            CAPTURE_STATE_OFFER,
            "625e25eaf19c75cf2d157bcff9c2d6ca5d0b9ff2995155375f326f9fde354b91",
        ),
        (
            "op41 sub107 zone ready",
            CAPTURE_ZONE_READY,
            "95c4e6234252b93af4c520607e2cb77802434e800d356632191f4b9b9661c3c3",
        ),
    ] {
        let frame = bytes(frame);
        assert_eq!(sha256_hex(&frame), expected, "{name} provenance hash");
    }

    let player_enter = bytes(CAPTURE_PLAYER_ENTER);
    assert_eq!(
        hex(&player_enter[7..]),
        hex(&encode_player_enter(CAPTURE_ENCOUNTER, CAPTURE_INSTANCE, 0)),
        "player-enter body"
    );
    let state = bytes(CAPTURE_STATE_OFFER);
    assert_eq!(
        hex(&state[7..]),
        hex(&encode_encounter_state(
            CAPTURE_ENCOUNTER,
            CAPTURE_INSTANCE,
            STATE_OFFER
        )),
        "encounter state body"
    );
    let ready = bytes(CAPTURE_ZONE_READY);
    assert_eq!(
        hex(&ready[7..]),
        hex(&encode_zone_ready(CAPTURE_ENCOUNTER, CAPTURE_INSTANCE)),
        "zone-ready body"
    );
}

#[test]
fn required_combat_level_matches_the_recovered_difficulty_table() {
    assert_eq!(required_combat_level(1), Some(0));
    assert_eq!(required_combat_level(2), Some(0));
    assert_eq!(required_combat_level(3), Some(5));
    assert_eq!(required_combat_level(4), Some(9));
    assert_eq!(required_combat_level(5), Some(13));
    assert_eq!(required_combat_level(0), None);
    assert_eq!(required_combat_level(6), None);
}

#[test]
fn entering_emits_the_recovered_player_enter_packet_and_records_membership() {
    let mut life = Lifecycle::new(INSTANCE, ENCOUNTER, 1, config(4));
    let packets = life.enter(FIRST, 0).unwrap();
    assert_eq!(
        hex(&packets[0]),
        "290002009b01000041420f00010000004000000000"
    );
    assert_eq!(life.phase(), Phase::Loading);
    assert_eq!(life.roster(), vec![FIRST]);
    assert_eq!(life.member(FIRST), Some(Member::Joined));
    assert_eq!(life.connected(), 1);
}

#[test]
fn entry_is_gated_by_difficulty_and_caller_supplied_capacity() {
    let mut life = Lifecycle::new(INSTANCE, ENCOUNTER, 3, config(2));
    assert_eq!(
        life.enter(FIRST, 4).unwrap_err(),
        Reject::NotEligible {
            required: 5,
            actual: 4
        }
    );
    life.enter(FIRST, 5).unwrap();
    life.enter(SECOND, 20).unwrap();
    assert_eq!(life.enter(THIRD, 20).unwrap_err(), Reject::InstanceFull);
    assert_eq!(life.roster(), vec![FIRST, SECOND]);
}

#[test]
fn duplicate_and_post_close_entry_are_rejected() {
    let mut life = ready_instance();
    assert_eq!(life.enter(SECOND, 20).unwrap_err(), Reject::RosterClosed);
    assert_eq!(life.enter(FIRST, 20).unwrap_err(), Reject::RosterClosed);
    life.start(FIRST).unwrap();
    life.spawn(&[0xaa]).unwrap();
    life.defeat(0xaa).unwrap();
    life.complete(FIRST).unwrap();
    assert_eq!(life.enter(THIRD, 20).unwrap_err(), Reject::RosterClosed);
    let mut closed = Lifecycle::new(INSTANCE, ENCOUNTER, 1, config(4));
    closed.enter(FIRST, 0).unwrap();
    closed.exit(FIRST).unwrap();
    assert_eq!(closed.phase(), Phase::Closed);
    assert_eq!(closed.enter(SECOND, 0).unwrap_err(), Reject::InstanceClosed);
}

#[test]
fn zone_ready_flips_the_go_packet_only_when_every_connected_member_is_ready() {
    let mut life = Lifecycle::new(INSTANCE, ENCOUNTER, 2, config(4));
    life.enter(FIRST, 3).unwrap();
    life.enter(SECOND, 3).unwrap();
    life.disconnect(SECOND, 1_000).unwrap();
    let packets = life.zone_ready(FIRST).unwrap();
    assert_eq!(
        packets.len(),
        1,
        "a disconnected member must not hold the loading gate closed"
    );
    assert_eq!(life.phase(), Phase::Ready);

    let mut waiting = Lifecycle::new(INSTANCE, ENCOUNTER, 2, config(4));
    waiting.enter(FIRST, 3).unwrap();
    waiting.enter(SECOND, 3).unwrap();
    assert!(waiting.zone_ready(FIRST).unwrap().is_empty());
    assert_eq!(waiting.phase(), Phase::Loading);
    let packets = waiting.zone_ready(SECOND).unwrap();
    assert_eq!(
        hex(&packets[0]),
        "29006b009b01000041420f00",
        "41/107 is the bare encounter header"
    );
    assert_eq!(waiting.phase(), Phase::Ready);
    assert_eq!(waiting.member(SECOND), Some(Member::Ready));
}

#[test]
fn start_walks_the_recovered_encounter_states() {
    let mut life = ready_instance();
    assert_eq!(life.phase(), Phase::Ready);
    let packets = life.start(FIRST).unwrap();
    assert_eq!(
        hex(&packets[0]),
        "29006a009b01000041420f0005000000",
        "41/106 state 5 is the GO step"
    );
    assert_eq!(
        hex(&packets[1]),
        "29006a009b01000041420f0006000000",
        "41/106 state 6 is running"
    );
    assert_eq!(life.phase(), Phase::Active);

    let mut loading = Lifecycle::new(INSTANCE, ENCOUNTER, 1, config(4));
    loading.enter(FIRST, 0).unwrap();
    assert_eq!(loading.start(FIRST).unwrap_err(), Reject::NotReady);
    assert_eq!(loading.start(SECOND).unwrap_err(), Reject::NotAMember);
    // The offer/ready walk is the recovered state sequence: 4 -> 5 -> 6.
    assert!(STATE_READY < STATE_GO && STATE_GO < STATE_RUNNING);
}

#[test]
fn spawn_is_once_per_generation_and_rejects_duplicate_or_reused_actor_ids() {
    let mut life = ready_instance();
    life.start(FIRST).unwrap();
    assert_eq!(life.spawn(&[]).unwrap_err(), Reject::EmptySpawn);
    assert_eq!(life.spawn(&[7, 7]).unwrap_err(), Reject::DuplicateActor(7));
    let spawn = life.spawn(&[7, 8]).unwrap();
    assert_eq!(spawn.generation, 1);
    assert_eq!(spawn.actors, 2);
    assert_eq!(spawn.defeated, 0);
    assert_eq!(life.spawn(&[9]).unwrap_err(), Reject::AlreadySpawned);
    assert_eq!(life.generation(), 1);
}

#[test]
fn reset_advances_the_generation_and_never_reuses_an_actor_id() {
    let mut life = ready_instance();
    life.start(FIRST).unwrap();
    assert_eq!(life.reset(FIRST).unwrap_err(), Reject::NothingToReset);
    life.spawn(&[7]).unwrap();
    let packets = life.reset(FIRST).unwrap();
    assert_eq!(hex(&packets[0]), "29006a009b01000041420f0006000000");
    assert_eq!(
        life.reset(FIRST).unwrap_err(),
        Reject::NothingToReset,
        "a reset clears the current generation"
    );
    let spawn = life.spawn(&[8]).unwrap();
    assert_eq!(spawn.generation, 2);
    assert_eq!(
        life.defeat(7).unwrap_err(),
        Reject::UnknownActor(7),
        "defeats do not carry across a reset generation"
    );
    life.defeat(8).unwrap();
    life.reset(FIRST).unwrap();
    assert_eq!(life.spawn(&[7]).unwrap_err(), Reject::ActorIdReused(7));
    assert_eq!(life.reset(SECOND).unwrap_err(), Reject::NotAMember);
}

#[test]
fn completion_requires_every_spawned_actor_defeated() {
    let mut life = ready_instance();
    life.start(FIRST).unwrap();
    assert_eq!(life.complete(FIRST).unwrap_err(), Reject::NotActive);
    life.spawn(&[7, 8]).unwrap();
    assert_eq!(life.complete(FIRST).unwrap_err(), Reject::EnemiesAlive(2));
    life.defeat(7).unwrap();
    assert_eq!(life.complete(FIRST).unwrap_err(), Reject::EnemiesAlive(1));
    assert_eq!(life.defeat(7).unwrap_err(), Reject::AlreadyDefeated(7));
    life.defeat(8).unwrap();
    assert_eq!(
        life.defeat(9).unwrap_err(),
        Reject::UnknownActor(9),
        "an actor that never spawned in this generation cannot be defeated"
    );
    assert!(life.complete(FIRST).is_ok());
}

#[test]
fn completion_is_once_only_and_emits_the_recovered_terminal_packets() {
    let mut life = ready_instance();
    life.start(FIRST).unwrap();
    life.spawn(&[7]).unwrap();
    life.defeat(7).unwrap();
    let completion = life.complete(FIRST).unwrap();
    assert_eq!(life.phase(), Phase::Completed);
    assert_eq!(completion.instance_id, INSTANCE);
    assert_eq!(completion.encounter_id, ENCOUNTER);
    assert_eq!(completion.roster, vec![FIRST]);
    assert_eq!(completion.generation, 1);
    assert_eq!(
        completion
            .packets
            .iter()
            .map(|packet| hex(packet))
            .collect::<Vec<_>>(),
        vec![
            // 39/18 | encounter 411 | current-state -1 | game 411 | Won true | 0, 0, 1
            concat!(
                "270012", "9b010000", "ffffffff", "9b010000", "01", "00000000", "00000000",
                "01000000"
            )
            .to_string(),
            // 39/19 | encounter 411 | current-state -1 | game 411
            "2700139b010000ffffffff9b010000".to_string(),
        ]
    );
    assert_eq!(life.complete(FIRST).unwrap_err(), Reject::AlreadyCompleted);
    assert_eq!(
        completion.packets,
        vec![
            encode_game_over(true, ENCOUNTER, -1, ENCOUNTER),
            encode_state_remove(ENCOUNTER, -1, ENCOUNTER),
        ]
    );
}

#[test]
fn completion_requires_a_connected_member_and_emits_no_reward_bytes() {
    let mut life = Lifecycle::new(INSTANCE, ENCOUNTER, 3, config(4));
    life.enter(FIRST, 9).unwrap();
    life.enter(SECOND, 9).unwrap();
    life.zone_ready(FIRST).unwrap();
    life.zone_ready(SECOND).unwrap();
    life.start(FIRST).unwrap();
    life.spawn(&[7]).unwrap();
    life.defeat(7).unwrap();
    assert_eq!(life.complete(THIRD).unwrap_err(), Reject::NotAMember);
    life.disconnect(FIRST, 1_000).unwrap();
    life.disconnect(SECOND, 1_001).unwrap();
    assert_eq!(life.complete(FIRST).unwrap_err(), Reject::NoConnectedMember);
    life.reconnect(FIRST, 2_000).unwrap();
    let completion = life.complete(FIRST).unwrap();
    // The settlement is exactly the recovered 39/18 + 39/19 pair: no bundle, loot or
    // currency packet is emitted here, so a regression cannot smuggle a grant into this path.
    assert_eq!(completion.packets.len(), 2);
    assert!(
        completion
            .packets
            .iter()
            .all(|packet| packet.starts_with(&[0x27, 0x00]))
    );
    assert_eq!(completion.roster, vec![FIRST, SECOND]);
}

#[test]
fn exit_keeps_the_instance_for_the_remaining_members_and_closes_on_the_last() {
    let mut life = Lifecycle::new(INSTANCE, ENCOUNTER, 2, config(4));
    life.enter(FIRST, 3).unwrap();
    life.enter(SECOND, 3).unwrap();
    assert!(life.exit(SECOND).unwrap().is_empty());
    assert_eq!(life.roster(), vec![FIRST]);
    assert_eq!(life.phase(), Phase::Loading);
    assert_eq!(life.exit(THIRD).unwrap_err(), Reject::NotAMember);
    let packets = life.exit(FIRST).unwrap();
    assert_eq!(hex(&packets[0]), "2700139b010000ffffffff9b010000");
    assert_eq!(life.phase(), Phase::Closed);
    assert_eq!(life.roster(), Vec::<u64>::new());
}

#[test]
fn disconnect_holds_the_slot_and_reconnect_preserves_the_spawn_generation() {
    let mut life = ready_instance();
    life.start(FIRST).unwrap();
    life.spawn(&[7]).unwrap();
    life.disconnect(FIRST, 5_000).unwrap();
    assert_eq!(
        life.member(FIRST),
        Some(Member::Disconnected {
            deadline_ms: 35_000
        })
    );
    assert_eq!(
        life.disconnect(FIRST, 5_100).unwrap_err(),
        Reject::NotConnected
    );
    assert_eq!(
        life.reconnect(SECOND, 5_100).unwrap_err(),
        Reject::NotAMember
    );
    assert_eq!(life.connected(), 0);
    assert_eq!(life.generation(), 1);
    assert_eq!(life.roster(), vec![FIRST], "a graced member keeps the slot");
    life.reconnect(FIRST, 20_000).unwrap();
    assert_eq!(life.member(FIRST), Some(Member::Ready));
    assert_eq!(life.connected(), 1);
    assert_eq!(
        life.reconnect(FIRST, 21_000).unwrap_err(),
        Reject::NotDisconnected
    );
    assert_eq!(life.generation(), 1, "reconnect must not respawn the run");
    assert_eq!(life.defeat(7).unwrap().defeated, 1);
}

#[test]
fn expired_membership_is_reaped_and_reconnect_after_the_deadline_is_rejected() {
    let mut life = Lifecycle::new(INSTANCE, ENCOUNTER, 2, config(4));
    life.enter(FIRST, 3).unwrap();
    life.enter(SECOND, 3).unwrap();
    life.disconnect(FIRST, 1_000).unwrap();
    assert_eq!(
        life.reconnect(FIRST, 31_001).unwrap_err(),
        Reject::MembershipExpired
    );
    assert_eq!(
        life.reap(1_000).unwrap_err(),
        Reject::NothingExpired,
        "the grace window is inclusive of its deadline"
    );
    let packets = life.reap(31_001).unwrap();
    assert!(
        packets.is_empty(),
        "a surviving member keeps the instance open"
    );
    assert_eq!(life.roster(), vec![SECOND]);
    assert_eq!(life.phase(), Phase::Loading);
    life.exit(SECOND).unwrap();
    let mut last = Lifecycle::new(INSTANCE, ENCOUNTER, 2, config(4));
    last.enter(FIRST, 3).unwrap();
    last.disconnect(FIRST, 10).unwrap();
    let packets = last.reap(30_010).unwrap();
    assert_eq!(hex(&packets[0]), "2700139b010000ffffffff9b010000");
    assert_eq!(last.phase(), Phase::Closed);
}

#[test]
fn registry_keeps_one_live_instance_per_player_and_never_reuses_ids() {
    let mut registry = Registry::new(INSTANCE);
    let first = registry.open(ENCOUNTER, 3, config(4)).unwrap();
    let second = registry.open(ENCOUNTER + 1, 3, config(4)).unwrap();
    assert_eq!((first, second), (INSTANCE, INSTANCE + 1));
    registry.enter(first, FIRST, 9).unwrap();
    assert_eq!(registry.instance_of(FIRST), Some(first));
    assert_eq!(
        registry.enter(second, FIRST, 9).unwrap_err(),
        Reject::OtherInstance(first)
    );
    assert_eq!(registry.get(second).unwrap().roster(), Vec::<u64>::new());
    assert_eq!(registry.enter(second, SECOND, 9).unwrap().len(), 1);
    registry.exit(first, FIRST).unwrap();
    assert_eq!(registry.instance_of(FIRST), None);
    assert_eq!(registry.get(first).unwrap().phase(), Phase::Closed);
    assert_eq!(registry.enter(second, FIRST, 9).unwrap().len(), 1);
    // Leaving a live instance frees the player for another run, even though it stays open.
    registry.exit(second, SECOND).unwrap();
    assert_eq!(registry.instance_of(SECOND), None);
    assert_eq!(registry.get(second).unwrap().phase(), Phase::Loading);
    assert_eq!(registry.get(second).unwrap().roster(), vec![FIRST]);
    assert_eq!(registry.live_instances(), 1);
    assert_eq!(registry.get(INSTANCE + 99).map(|life| life.phase()), None);
}
