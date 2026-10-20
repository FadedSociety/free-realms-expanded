//! T007 route proof: field-battle matchmaking queues, zero queue statistics,
//! the empty in-progress lobby, and explicit refusal of every queue join.
//!
//! The wire layouts are tested in `tests/matchmaking.rs`. This file drives the
//! real non-developer `Gameplay` with raw application packets.
use osfr_tools::{
    activity_portal::{Catalogue, Kind},
    character_store::CharacterState,
    gameplay::{Gameplay, Outcome},
    matchmaking::{EmptyRunningListResponse, QueueListResponse, QueueStatsResponse},
};
use std::path::PathBuf;

const LAKESHORE: [f32; 3] = [-733.952, 9.09, 945.973];
const BATTLE: u32 = 96;
const SESSION_KEY: u64 = 0x0102_0304_0506_0708;

fn content() -> PathBuf {
    std::env::var_os("OSFR_ACTIVITY_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/rust-opening-cooking-original-assets")
        })
}

fn character() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: LAKESHORE,
        rotation: [0.0, 0.0, 1.0],
    }
}

fn game() -> Gameplay {
    let mut game = Gameplay::load(&content(), None).unwrap();
    game.initialize_world(&character(), 500).unwrap();
    game
}

fn list_request(key: u64) -> Vec<u8> {
    let mut request = vec![141, 0, 1, 0];
    request.extend(key.to_le_bytes());
    request
}

fn stats_request(key: u64) -> Vec<u8> {
    let mut request = vec![141, 0, 13, 0];
    request.extend(key.to_le_bytes());
    request
}

/// Native 00c08910 writes the 141/3 request subobject as u64, u32, queue id.
fn join_request(queue_id: u32) -> Vec<u8> {
    let mut request = vec![141, 0, 3, 0];
    request.extend(0x0102_0304_0506_0708u64.to_le_bytes());
    request.extend(0u32.to_le_bytes());
    request.extend(queue_id.to_le_bytes());
    request
}

/// Native 00a26890 writes the 12-byte encounter header and two u32 fields.
fn running_list_request() -> Vec<u8> {
    let mut request = vec![41, 0, 129, 0];
    request.extend([0; 16]);
    request
}

/// Native 00bb11b0 writes u16 family, u8 sub, then the u32 lobby type.
fn leave_lobby_request() -> Vec<u8> {
    vec![101, 0, 9, 3, 0, 0, 0]
}

fn chat_text(out: &Outcome) -> String {
    out.replies
        .iter()
        .filter(|reply| reply.starts_with(&[15, 0, 3, 0]))
        .map(|reply| {
            let length = u32::from_le_bytes(reply[4..8].try_into().unwrap()) as usize;
            String::from_utf8_lossy(&reply[8..8 + length]).into_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn queue_list(game: &mut Gameplay) -> QueueListResponse {
    let out = game
        .handle_in_world(&list_request(SESSION_KEY), &character(), 1_000)
        .unwrap();
    let replies: Vec<_> = out
        .replies
        .iter()
        .filter(|reply| reply.starts_with(&[141, 0, 2, 0]))
        .collect();
    assert_eq!(replies.len(), 1, "exactly one 141/2 reply");
    QueueListResponse::decode(replies[0]).unwrap()
}

/// The queues the server must publish: every shipped field battle, and no
/// chess or soccer portal row.
fn battle_ids() -> Vec<u32> {
    Catalogue::load(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/field_battles.json"),
    )
    .unwrap()
    .activities()
    .iter()
    .filter(|activity| activity.kind == Kind::Battle)
    .map(|activity| activity.id)
    .collect()
}

fn assert_list_canary(game: &mut Gameplay) {
    let list = queue_list(game);
    assert_eq!(list.session_key, SESSION_KEY);
    assert_eq!(
        list.rows
            .iter()
            .map(|row| row.queue_id())
            .collect::<Vec<_>>(),
        battle_ids(),
        "the queue list still works and publishes only field battles"
    );
}

#[test]
fn list_and_stats_publish_only_field_battles_and_do_not_mutate() {
    let mut game = game();
    let before = serde_json::to_value(game.state()).unwrap();

    let list = queue_list(&mut game);
    assert_eq!(list.session_key, SESSION_KEY);
    let ids: Vec<u32> = list.rows.iter().map(|row| row.queue_id()).collect();
    assert_eq!(ids, battle_ids(), "one queue per shipped field battle");
    assert!(
        !ids.contains(&20) && !ids.contains(&42),
        "chess and soccer are portal rows, not matchmaking queues"
    );
    let row = &list.rows[0];
    assert_eq!(row.queue_id(), BATTLE);
    assert_eq!(row.wire_dwords[3..=4], [1, 1], "solo queue capacity");
    assert_eq!(row.wire_dwords[15], 42059, "battle description id");
    assert_eq!(row.wire_dwords[16], 4543, "battle icon id");
    assert_eq!(row.member_only, 0);

    let out = game
        .handle_in_world(&stats_request(SESSION_KEY), &character(), 1_100)
        .unwrap();
    let replies: Vec<_> = out
        .replies
        .iter()
        .filter(|reply| reply.starts_with(&[141, 0, 14, 0]))
        .collect();
    assert_eq!(replies.len(), 1, "exactly one 141/14 reply");
    let stats = QueueStatsResponse::decode(replies[0]).unwrap();
    assert_eq!(stats.session_key, SESSION_KEY);
    assert_eq!(stats.values_a, vec![0; ids.len()]);
    assert_eq!(stats.values_b, vec![0; ids.len()]);
    assert!(!out.changed);

    assert_eq!(serde_json::to_value(game.state()).unwrap(), before);
}

#[test]
fn every_join_is_refused_without_state_and_the_list_still_works() {
    let mut game = game();
    for queue_id in [BATTLE, 12301, 0, BATTLE + 1, 411, 0x8000_0000, u32::MAX] {
        let before = serde_json::to_value(game.state()).unwrap();
        let out = game
            .handle_in_world(&join_request(queue_id), &character(), 2_000)
            .unwrap();
        assert!(!out.changed, "queue {queue_id} changed gameplay");
        assert!(out.zoning.is_none() && out.scene_zoning.is_none());
        assert!(
            chat_text(&out).contains("not available"),
            "queue {queue_id}: {}",
            chat_text(&out)
        );
        assert_eq!(
            serde_json::to_value(game.state()).unwrap(),
            before,
            "queue {queue_id}"
        );
        assert_list_canary(&mut game);
    }

    // One byte short of the native queue-id prefix: malformed, not a join.
    let mut short = vec![141, 0, 3, 0];
    short.extend([0; 15]);
    assert!(
        game.handle_in_world(&short, &character(), 2_100).is_err(),
        "a truncated join is rejected rather than treated as idle"
    );
    assert_list_canary(&mut game);
}

#[test]
fn in_progress_games_are_empty_and_leave_lobby_is_a_known_noop() {
    let mut game = game();
    let before = serde_json::to_value(game.state()).unwrap();

    let running = game
        .handle_in_world(&running_list_request(), &character(), 3_000)
        .unwrap();
    let replies: Vec<_> = running
        .replies
        .iter()
        .filter(|reply| reply.starts_with(&[41, 0, 130, 0]))
        .collect();
    assert_eq!(replies.len(), 1);
    assert_eq!(
        replies[0].as_slice(),
        &[41, 0, 130, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        "41/130 clears the in-progress table with a zero count"
    );
    EmptyRunningListResponse::decode(replies[0]).unwrap();
    assert!(!running.changed);

    let leave = game
        .handle_in_world(&leave_lobby_request(), &character(), 3_100)
        .unwrap();
    assert!(leave.replies.is_empty());
    assert!(!leave.changed);
    assert_eq!(serde_json::to_value(game.state()).unwrap(), before);
}

#[test]
fn malformed_known_requests_are_rejected_and_activity_portal_still_works() {
    let mut game = game();
    let before = serde_json::to_value(game.state()).unwrap();
    for packet in [
        vec![141, 0, 1, 0],
        vec![141, 0, 13, 0, 0],
        vec![41, 0, 129, 0],
        vec![101, 0, 9, 0],
        vec![101, 0, 9, 0, 0, 0, 0, 0],
    ] {
        assert!(
            game.handle_in_world(&packet, &character(), 4_000).is_err(),
            "{packet:02x?}"
        );
        assert_eq!(serde_json::to_value(game.state()).unwrap(), before);
        assert_list_canary(&mut game);
    }

    let portal = game
        .handle_in_world(&[167, 0, 1, 4], &character(), 4_100)
        .unwrap();
    assert!(
        portal
            .replies
            .iter()
            .any(|reply| reply.starts_with(&[167, 0, 1, 1]))
    );
}
