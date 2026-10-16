use osfr_tools::{matchmaking, minigame, players};
use std::cell::Cell;
use std::rc::Rc;

use matchmaking::{
    Clock, InviteMemberNotice, JoinResult, Matchmaking, MatchmakingError, Participant,
    QueueDefinition, QueueEntry, QueueListResponse, QueueRow, QueueStatsResponse, RandomSource,
    SessionOutbound, TunnelFrame, decode_activity_launched, decode_cancel_match_request,
    decode_invite_response, decode_server_status, encode_activity_launched,
    encode_cancel_match_request, encode_invite_response, encode_member_state, encode_server_status,
};
use minigame::{RunMode, RunSpec, RunState};
use players::SessionId;

#[derive(Clone)]
struct FakeClock(Rc<Cell<u64>>);

impl FakeClock {
    fn new() -> Self {
        Self(Rc::new(Cell::new(0)))
    }

    fn set(&self, value: u64) {
        self.0.set(value);
    }
}

impl Clock for FakeClock {
    fn now_ms(&self) -> u64 {
        self.0.get()
    }
}

struct FakeRng(u64);

impl RandomSource for FakeRng {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(1);
        self.0
    }
}

fn spec() -> RunSpec {
    RunSpec::empty(
        46,
        9,
        RunMode::Flash {
            movie: "game_cooking.gfx".into(),
        },
    )
}

fn definition(queue_id: u32, min_players: usize, max_players: usize) -> QueueDefinition {
    QueueDefinition {
        queue_id,
        min_players,
        max_players,
        min_teams: 1,
        max_teams: max_players,
        team_sizes: Vec::new(),
        allow_solo: true,
        allow_groups: true,
        max_wait_ms: 0,
        ready_timeout_ms: 100,
        countdown_ms: 0,
        run_spec: spec(),
    }
}

fn service(defs: Vec<QueueDefinition>, clock: FakeClock) -> Matchmaking<FakeClock, FakeRng> {
    Matchmaking::new(defs, clock, FakeRng(400), 71).unwrap()
}

fn participant(session: u64) -> Participant {
    Participant {
        session_id: SessionId(session),
        character_guid: 10_000 + session,
    }
}

fn solo(session: u64) -> QueueEntry {
    QueueEntry {
        group_id: None,
        leader: SessionId(session),
        members: vec![participant(session)],
    }
}

fn group(id: u32, sessions: &[u64]) -> QueueEntry {
    QueueEntry {
        group_id: Some(id),
        leader: SessionId(sessions[0]),
        members: sessions.iter().copied().map(participant).collect(),
    }
}

fn family_sub(bytes: &[u8]) -> (u16, u16) {
    (
        u16::from_le_bytes([bytes[0], bytes[1]]),
        u16::from_le_bytes([bytes[2], bytes[3]]),
    )
}

fn family_sub_byte(bytes: &[u8]) -> (u16, u8) {
    (u16::from_le_bytes([bytes[0], bytes[1]]), bytes[2])
}

fn find_to(out: &[SessionOutbound], session: u64, sub: u16) -> &SessionOutbound {
    out.iter()
        .find(|message| {
            message.recipient == SessionId(session) && family_sub(&message.bytes) == (175, sub)
        })
        .expect("expected per-session group-launch packet")
}

#[test]
fn queue_row_and_queue_list_use_recovered_fixed_width_layout() {
    let row = QueueRow::new(
        5,
        100,
        1,
        2,
        4,
        1,
        2,
        30,
        [8, 9, 10, 11, 12, 13, 14],
        200,
        201,
        [0xfeed, 0xbeef],
        1,
        0,
    );
    let response = QueueListResponse {
        session_key: 0x4b52_cc6a_9573_ca91,
        rows: vec![row.clone()],
    };
    let bytes = response.encode().unwrap();
    assert_eq!(bytes.len(), 4 + 8 + 4 + 78);
    assert_eq!(&bytes[..4], &[141, 0, 2, 0]);
    assert_eq!(QueueListResponse::decode(&bytes).unwrap(), response);
    assert_eq!(row.queue_id(), 5);
    let request = matchmaking::ListQueuesRequest {
        session_key: response.session_key,
    };
    assert_eq!(
        matchmaking::ListQueuesRequest::decode(&request.encode()).unwrap(),
        request
    );
    let stats = QueueStatsResponse {
        session_key: response.session_key,
        values_a: vec![0, 1, 2],
        values_b: vec![5, 30, 60],
    };
    assert_eq!(
        QueueStatsResponse::decode(&stats.encode().unwrap()).unwrap(),
        stats
    );
}

#[test]
#[ignore = "requires preserved raw capture paths outside the repository"]
fn captured_kart_catalogue_frames_roundtrip_when_private_gate_is_enabled() {
    let request_path = std::env::var_os("OSFR_W52_KART_141_1")
        .expect("set OSFR_W52_KART_141_1 to the preserved capture");
    let response_path = std::env::var_os("OSFR_W52_KART_141_2")
        .expect("set OSFR_W52_KART_141_2 to the preserved capture");
    let request_bytes = std::fs::read(request_path).unwrap();
    let response_bytes = std::fs::read(response_path).unwrap();
    let request_frame = TunnelFrame::decode(&request_bytes).unwrap();
    let response_frame = TunnelFrame::decode(&response_bytes).unwrap();
    let request = matchmaking::ListQueuesRequest::decode(&request_frame.inner).unwrap();
    let response = QueueListResponse::decode(&response_frame.inner).unwrap();
    assert_eq!(request.session_key, 0x4b52_cc6a_9573_ca91);
    assert_eq!(response.session_key, request.session_key);
    assert_eq!(response.rows.len(), 5);
    assert_eq!(response.rows[0].queue_id(), 5);
    assert_eq!(request_frame.encode().unwrap(), request_bytes);
    assert_eq!(response_frame.encode().unwrap(), response_bytes);
}

#[test]
fn other_recovered_packet_families_have_checked_shapes_and_roundtrip() {
    let cancel = encode_cancel_match_request(0x0102_0304_0506_0708);
    assert_eq!(
        decode_cancel_match_request(&cancel).unwrap(),
        0x0102_0304_0506_0708
    );
    assert_eq!(
        decode_server_status(&encode_server_status(true)).unwrap(),
        true
    );

    let launched = encode_activity_launched(12, 5, &[0x0102_0304_0506_0708, 99]).unwrap();
    assert_eq!(
        decode_activity_launched(&launched).unwrap(),
        (12, 5, vec![0x0102_0304_0506_0708, 99])
    );
    assert_eq!(family_sub(&encode_member_state(12, 5, 2, 0x82)), (175, 9));

    let reply = encode_invite_response(12, 5, false, 3);
    assert_eq!(decode_invite_response(&reply).unwrap(), (12, 5, false, 3));

    let notice = InviteMemberNotice {
        sub_op: 3,
        request_word: 12,
        context_word: 5,
        character_guid: 99,
        label_bytes: Vec::new(),
        status: true,
        result_code: 7,
    };
    assert_eq!(
        InviteMemberNotice::decode(&notice.encode().unwrap()).unwrap(),
        notice
    );
}

#[test]
fn solos_and_group_join_as_indivisible_entries_and_duplicate_join_is_inert() {
    let clock = FakeClock::new();
    let mut service = service(vec![definition(5, 3, 3)], clock);
    let grouped = service.join(5, group(21, &[1, 2])).unwrap();
    assert_eq!(grouped.result, JoinResult::Joined);
    assert!(grouped.outbound.is_empty());
    assert_eq!(service.join(5, solo(3)).unwrap().result, JoinResult::Joined);
    assert_eq!(
        service.join(5, group(21, &[1, 2])).unwrap().result,
        JoinResult::Duplicate
    );
    let ready = service.tick().unwrap();
    assert_eq!(ready.len(), 3);
    assert!(
        ready
            .iter()
            .all(|message| family_sub(&message.bytes) == (175, 14))
    );

    for session in [1, 2, 3] {
        let state = service.set_ready(SessionId(session), true).unwrap();
        assert_eq!(state.len(), 3);
        assert!(
            state
                .iter()
                .all(|message| family_sub(&message.bytes) == (175, 9))
        );
    }
    let start = service.tick().unwrap();
    assert_eq!(start.len(), 9);
    let (_, _, roster) = decode_activity_launched(&find_to(&start, 1, 5).bytes).unwrap();
    assert_eq!(roster, vec![10_001, 10_002, 10_003]);
    assert!(
        start
            .iter()
            .any(|message| family_sub_byte(&message.bytes) == (39, 16))
    );
    assert_eq!(service.match_state(71), Some(RunState::Offered));
}

#[test]
fn fifo_prefix_forms_first_and_match_never_exceeds_configured_maximum() {
    let clock = FakeClock::new();
    let mut service = service(vec![definition(5, 2, 2)], clock);
    for session in [1, 2, 3] {
        service.join(5, solo(session)).unwrap();
    }
    service.tick().unwrap();
    service.set_ready(SessionId(1), true).unwrap();
    service.set_ready(SessionId(2), true).unwrap();
    let start = service.tick().unwrap();
    let (_, _, roster) = decode_activity_launched(&find_to(&start, 1, 5).bytes).unwrap();
    assert_eq!(roster, vec![10_001, 10_002]);
    assert_eq!(service.queued_sessions(5), Some(1));
}

#[test]
fn session_cannot_join_a_second_queue_while_reserved() {
    let clock = FakeClock::new();
    let mut service = service(vec![definition(5, 2, 4), definition(8, 2, 4)], clock);
    assert_eq!(service.join(5, solo(1)).unwrap().result, JoinResult::Joined);
    assert_eq!(
        service.join(5, solo(1)).unwrap().result,
        JoinResult::Duplicate
    );
    assert_eq!(service.join(8, solo(1)), Err(MatchmakingError::SessionBusy));
}

#[test]
fn configured_team_capacity_assigns_whole_groups_without_oversizing_a_team() {
    let clock = FakeClock::new();
    let mut queue = definition(5, 4, 4);
    queue.min_teams = 2;
    queue.max_teams = 2;
    queue.team_sizes = vec![2, 2];
    let mut service = service(vec![queue], clock);
    service.join(5, solo(1)).unwrap();
    service.join(5, solo(2)).unwrap();
    service.join(5, group(9, &[3, 4])).unwrap();
    let ready = service.tick().unwrap();
    assert_eq!(ready.len(), 4);
    for session in [1, 2, 3, 4] {
        service.set_ready(SessionId(session), true).unwrap();
    }
    let launched = service.tick().unwrap();
    let (_, _, roster) = decode_activity_launched(&find_to(&launched, 1, 5).bytes).unwrap();
    assert_eq!(roster, vec![10_001, 10_002, 10_003, 10_004]);
}

#[test]
fn leave_and_disconnect_remove_only_the_departing_queued_member() {
    let clock = FakeClock::new();
    let mut service = service(vec![definition(5, 2, 4)], clock);
    service.join(5, group(7, &[1, 2, 3])).unwrap();
    service.leave(SessionId(2)).unwrap();
    assert_eq!(service.queued_sessions(5), Some(2));
    service.disconnect(SessionId(1)).unwrap();
    assert_eq!(service.queued_sessions(5), Some(1));
    assert_eq!(
        service.join(5, solo(3)).unwrap().result,
        JoinResult::Duplicate
    );
}

#[test]
fn ready_timeout_requeues_only_ready_players_and_frees_other_reservations() {
    let clock = FakeClock::new();
    let mut timed = definition(5, 2, 2);
    timed.ready_timeout_ms = 10;
    let mut service = service(vec![timed, definition(8, 1, 2)], clock.clone());
    service.join(5, solo(1)).unwrap();
    service.join(5, solo(2)).unwrap();
    service.tick().unwrap();
    service.set_ready(SessionId(1), true).unwrap();
    clock.set(11);
    let canceled = service.tick().unwrap();
    assert_eq!(family_sub(&find_to(&canceled, 1, 16).bytes), (175, 16));
    assert_eq!(service.queued_sessions(5), Some(1));
    assert_eq!(service.join(8, solo(1)), Err(MatchmakingError::SessionBusy));
    assert_eq!(service.join(8, solo(2)).unwrap().result, JoinResult::Joined);
}

#[test]
fn ready_disconnect_cancels_check_and_requeues_only_members_marked_ready() {
    let clock = FakeClock::new();
    let mut service = service(vec![definition(5, 2, 3)], clock);
    service.join(5, solo(1)).unwrap();
    service.join(5, solo(2)).unwrap();
    service.tick().unwrap();
    service.set_ready(SessionId(1), true).unwrap();
    let canceled = service.disconnect(SessionId(2)).unwrap();
    assert_eq!(canceled.len(), 1);
    assert_eq!(family_sub(&canceled[0].bytes), (175, 16));
    assert_eq!(service.queued_sessions(5), Some(1));
    assert_eq!(
        service.join(5, solo(1)).unwrap().result,
        JoinResult::Duplicate
    );
    assert_eq!(service.join(5, solo(2)).unwrap().result, JoinResult::Joined);
}

#[test]
fn leaving_during_ready_check_releases_unready_peers_for_rejoin() {
    let clock = FakeClock::new();
    let mut service = service(vec![definition(5, 2, 3), definition(8, 1, 3)], clock);
    service.join(5, solo(1)).unwrap();
    service.join(5, solo(2)).unwrap();
    service.tick().unwrap();
    let canceled = service.leave(SessionId(2)).unwrap();
    assert_eq!(canceled.len(), 1);
    assert_eq!(service.queued_sessions(5), Some(0));
    assert_eq!(service.join(8, solo(1)).unwrap().result, JoinResult::Joined);
}

#[test]
fn disconnect_after_match_start_keeps_remaining_member_and_one_shared_instance() {
    let clock = FakeClock::new();
    let mut service = service(vec![definition(5, 2, 3)], clock);
    service.join(5, solo(1)).unwrap();
    service.join(5, solo(2)).unwrap();
    service.join(5, solo(3)).unwrap();
    service.tick().unwrap();
    service.set_ready(SessionId(1), true).unwrap();
    service.set_ready(SessionId(2), true).unwrap();
    service.set_ready(SessionId(3), true).unwrap();
    let launched = service.tick().unwrap();
    assert_eq!(
        launched
            .iter()
            .filter(|m| family_sub_byte(&m.bytes) == (39, 16))
            .count(),
        3
    );
    assert_eq!(service.active_match_count(), 1);
    assert_eq!(service.leave(SessionId(1)).unwrap(), Vec::new());
    assert_eq!(service.active_match_count(), 1);
    assert_eq!(
        service.join(5, solo(3)).unwrap().result,
        JoinResult::Duplicate
    );
    service.disconnect(SessionId(2)).unwrap();
    assert_eq!(service.active_match_count(), 1);
    service.disconnect(SessionId(3)).unwrap();
    assert_eq!(service.active_match_count(), 0);
    assert_eq!(service.close_match(71), Err(MatchmakingError::UnknownMatch));
}
