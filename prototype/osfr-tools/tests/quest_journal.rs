use osfr_tools::{
    quest_engine::{
        CharacterEligibility, ObjectiveKind, OutboundMessage, QuestDefinition, QuestDenial,
        QuestEngine, QuestMessage, QuestObjective, QuestOutcome, QuestRequirements, QuestState,
        QuestStatus,
    },
    quest_journal::{JournalRequest, Malformed, decode, handle},
    rewards::{RewardDisplay, RewardGrant},
};
use std::collections::BTreeMap;

const CHARACTER: u64 = 800;
const CANARY_COINS: u32 = 77_001;

fn quest(id: u32, giver: u64, objectives: usize) -> QuestDefinition {
    QuestDefinition {
        id,
        title: format!("Quest {id}"),
        description: String::new(),
        giver_npc_id: giver,
        turnin_npc_id: giver,
        prerequisite_quest_ids: Vec::new(),
        requirements: QuestRequirements::default(),
        ordered: false,
        objectives: (1..=objectives as u32)
            .map(|n| QuestObjective {
                id: n,
                required: 3,
                description: format!("Goal {n}"),
                kind: ObjectiveKind::KillActorKind { actor_kind_id: 5 },
            })
            .collect(),
        reward: RewardGrant {
            coins: CANARY_COINS,
            items: Vec::new(),
            job_xp: BTreeMap::new(),
            sc: 0,
            display: RewardDisplay {
                source_guid: 1,
                player_guid: CHARACTER,
                reward_kind: 0,
                icon_id: 0,
                name_id: 0,
                rows: Vec::new(),
            },
        },
    }
}

fn definitions() -> Vec<QuestDefinition> {
    let mut after_ten = quest(11, 2, 1);
    after_ten.prerequisite_quest_ids = vec![10];
    let mut high_level = quest(13, 5, 1);
    high_level.requirements.player_level = Some(50);
    vec![
        quest(10, 1, 1),
        after_ten,
        quest(12, 3, 0), // hollow: no objectives, like most generated job quests
        high_level,
        quest(14, 4, 1),
        quest(15, 6, 2),
    ]
}

fn eligibility(limit: usize) -> CharacterEligibility {
    CharacterEligibility {
        player_level: 10,
        job_levels: BTreeMap::new(),
        member: true,
        nonmember_quest_limit: limit,
        member_quest_limit: limit,
    }
}

fn engine(limit: usize) -> QuestEngine {
    QuestEngine::new(CHARACTER, definitions(), eligibility(limit), None).unwrap()
}

fn registry() -> BTreeMap<u32, u64> {
    BTreeMap::from([(10, 1), (11, 2), (12, 3), (13, 5), (14, 4)])
}

fn select(id: i32) -> Vec<u8> {
    [&[47, 0, 12][..], &id.to_le_bytes()].concat()
}

fn lock(byte: u8) -> Vec<u8> {
    vec![47, 0, 13, byte]
}

fn crumb(id: i32) -> Vec<u8> {
    [&[49, 0, 15, 0, 0, 0][..], &id.to_le_bytes()].concat()
}

/// Decode then handle, as the gateway route will.
fn run(engine: &mut QuestEngine, packet: &[u8]) -> QuestOutcome {
    let request = decode(packet).unwrap().expect("a journal request");
    handle(engine, request, &registry())
}

/// Offer + accept from the giver, the ordinary way a quest becomes active.
fn activate(engine: &mut QuestEngine, id: u32, giver: u64) {
    assert!(engine.offer(id, giver).changed);
    assert!(engine.accept(id).changed);
}

fn info_ids(outcome: &QuestOutcome) -> Vec<u32> {
    outcome
        .outbound
        .iter()
        .filter_map(|m| match m {
            OutboundMessage::Quest(QuestMessage::QuestInfo { quest_id }) => Some(*quest_id),
            _ => None,
        })
        .collect()
}

/// Nothing in the state or the outcome may claim progress, completion or a reward.
fn assert_no_award(engine: &QuestEngine, outcome: &QuestOutcome) {
    assert!(outcome.rewards.is_empty());
    assert!(
        outcome
            .outbound
            .iter()
            .all(|m| !matches!(m, OutboundMessage::Reward(_)))
    );
    assert!(
        engine
            .state()
            .quests
            .values()
            .all(|p| p.status != QuestStatus::Completed && p.counts.iter().all(|c| *c == 0))
    );
    assert_eq!(engine.state().pending_turnin, None);
}

#[test]
fn decodes_exact_native_widths() {
    assert_eq!(
        decode(&select(10)),
        Ok(Some(JournalRequest::Select { quest_id: 10 }))
    );
    assert_eq!(
        decode(&select(i32::MAX)),
        Ok(Some(JournalRequest::Select {
            quest_id: i32::MAX as u32
        }))
    );
    assert_eq!(
        decode(&lock(0)),
        Ok(Some(JournalRequest::Lock { locked: false }))
    );
    assert_eq!(
        decode(&lock(1)),
        Ok(Some(JournalRequest::Lock { locked: true }))
    );
    assert_eq!(
        decode(&crumb(3_010_001)),
        Ok(Some(JournalRequest::Breadcrumb {
            quest_id: 3_010_001
        }))
    );
    // Byte-for-byte the client's wire sizes: 7, 4 and 10.
    assert_eq!(select(1).len(), 7);
    assert_eq!(lock(1).len(), 4);
    assert_eq!(crumb(1).len(), 10);
}

#[test]
fn other_packets_are_left_to_the_rest_of_the_dispatch() {
    for packet in [
        &[][..],
        &[47],
        &[47, 0],
        &[47, 0, 14, 0], // 47/14 is server to client
        &[47, 0, 6, 1, 0, 0, 0],
        &[49, 0, 2, 0, 0, 0, 1, 0, 0, 0, 1], // 49/2 accept stays with the gateway
        &[49, 0, 14, 0, 0, 0, 1, 0, 0, 0],
        &[49, 0, 15],                      // truncated header is not ours
        &[49, 0, 15, 1, 0, 0, 1, 0, 0, 0], // subtype 271
        &[49, 0, 15, 0, 1, 0, 1, 0, 0, 0],
        &[48, 0, 12, 1, 0, 0, 0],
        &[47, 1, 12, 1, 0, 0, 0], // family 303
        &[26, 0, 23, 0, 0, 0, 1, 0, 0, 0],
    ] {
        assert_eq!(decode(packet), Ok(None), "{packet:?}");
    }
}

#[test]
fn malformed_journal_packets_are_refused_without_panicking() {
    for packet in [
        vec![47, 0, 12],                // no body
        vec![47, 0, 12, 1, 0, 0],       // short id
        vec![47, 0, 12, 1, 0, 0, 0, 0], // trailing byte (canary)
        select(0),                      // id 0
        select(-1),                     // client's "no quest"
        select(i32::MIN),
        vec![47, 0, 13],    // no flag
        vec![47, 0, 13, 2], // not a bool
        vec![47, 0, 13, 0xff],
        vec![47, 0, 13, 1, 1],    // trailing byte
        vec![49, 0, 15, 0, 0, 0], // no id
        vec![49, 0, 15, 0, 0, 0, 1, 0, 0],
        [crumb(9), vec![0]].concat(),
    ] {
        assert_eq!(decode(&packet), Err(Malformed), "{packet:?}");
    }
    // Sweep every short run of edge bytes behind each header: never a panic,
    // and anything accepted carries a positive i32 id.
    for header in [&[47u8, 0, 12][..], &[47, 0, 13], &[49, 0, 15, 0, 0, 0]] {
        for len in 0..7 {
            for fill in [0u8, 1, 2, 0x7f, 0x80, 0xff] {
                let packet = [header, &vec![fill; len]].concat();
                // A 49/15 may decode to id 0 (see below); never to a negative id as u32.
                if let Ok(Some(
                    JournalRequest::Select { quest_id } | JournalRequest::Breadcrumb { quest_id },
                )) = decode(&packet)
                {
                    assert!(quest_id <= i32::MAX as u32, "{packet:?}");
                    assert!(quest_id > 0 || header[2] == 15, "{packet:?}");
                }
            }
        }
    }
}

/// T497: a 49/15 whose id is 0 or negative is a well-formed request for an id nothing maps
/// (the caller answers it with the refusal line), not a malformed packet that ends the session.
#[test]
fn a_breadcrumb_with_a_non_positive_id_is_a_request_for_id_zero() {
    for id in [0, -1, -5, i32::MIN] {
        assert_eq!(
            decode(&crumb(id)),
            Ok(Some(JournalRequest::Breadcrumb { quest_id: 0 })),
            "{id}"
        );
    }
    // The other requests keep refusing a non-positive id.
    for packet in [select(0), select(-1)] {
        assert_eq!(decode(&packet), Err(Malformed));
    }
    // Id 0 is never startable: no registry entry, no state change.
    let mut e = engine(20);
    let before = e.state().clone();
    let outcome = run(&mut e, &crumb(0));
    assert!(!outcome.changed && outcome.outbound.is_empty());
    assert_eq!(outcome.denial, Some(QuestDenial::Unavailable));
    assert_eq!(e.state(), &before);
}

#[test]
fn selecting_an_owned_active_quest_persists_and_survives_a_reload() {
    let mut e = engine(20);
    activate(&mut e, 10, 1);
    activate(&mut e, 14, 4);
    let first = run(&mut e, &select(10));
    assert!(first.changed && first.denial.is_none() && first.outbound.is_empty());
    assert_eq!(e.state().selected_quest_id, Some(10));
    // Same id again changes nothing.
    assert!(!run(&mut e, &select(10)).changed);
    // Switching works and the old one is no longer selected.
    assert!(run(&mut e, &select(14)).changed);
    assert_eq!(e.state().selected_quest_id, Some(14));
    // Selecting is not progress.
    assert_no_award(&e, &first);

    let saved: QuestState =
        serde_json::from_slice(&serde_json::to_vec(e.state()).unwrap()).unwrap();
    let resumed = QuestEngine::new(CHARACTER, definitions(), eligibility(20), Some(saved)).unwrap();
    assert_eq!(resumed.state().selected_quest_id, Some(14));
}

#[test]
fn forged_or_ineligible_selections_change_nothing() {
    let mut e = engine(20);
    activate(&mut e, 10, 1);
    assert!(run(&mut e, &select(10)).changed);
    // An offer that is pending but not accepted.
    assert!(e.offer(14, 4).changed);
    let before = e.state().clone();
    for id in [
        14,       // offered, not owned
        11,       // known, never offered
        999_999,  // unknown
        i32::MAX, // unknown, largest legal id
        12,       // hollow
    ] {
        let outcome = run(&mut e, &select(id));
        assert!(!outcome.changed, "{id}");
        assert_eq!(outcome.denial, Some(QuestDenial::Unavailable), "{id}");
        assert!(outcome.outbound.is_empty() && outcome.rewards.is_empty());
        assert_eq!(e.state(), &before, "state moved for forged select {id}");
    }
    // Completed, abandoned and removed quests cannot be re-selected either.
    assert!(e.decline(14).changed);
    activate(&mut e, 14, 4);
    assert!(e.abandon(14).changed);
    let after_abandon = e.state().clone();
    let outcome = run(&mut e, &select(14));
    assert_eq!(outcome.denial, Some(QuestDenial::Unavailable));
    assert_eq!(e.state(), &after_abandon);
}

#[test]
fn selecting_a_completed_quest_is_refused() {
    let mut e = engine(20);
    activate(&mut e, 10, 1);
    let state = e.state().clone();
    let mut done = state.clone();
    done.quests.get_mut(&10).unwrap().status = QuestStatus::Completed;
    done.selected_quest_id = None;
    let mut resumed = QuestEngine::new(
        CHARACTER,
        definitions(),
        eligibility(20),
        Some(done.clone()),
    )
    .unwrap();
    let loaded = resumed.state().clone();
    let outcome = run(&mut resumed, &select(10));
    assert_eq!(outcome.denial, Some(QuestDenial::Unavailable));
    assert_eq!(resumed.state(), &loaded);
}

#[test]
fn lock_flag_is_the_inverse_of_the_client_auto_switch_flag() {
    let mut e = engine(20);
    // Fresh state: auto-switch clear, i.e. the client already considers it locked.
    assert!(!e.state().auto_switch_selected);
    // 47/13 byte 0 = unlocked = client auto-switch set.
    assert!(run(&mut e, &lock(0)).changed);
    assert!(e.state().auto_switch_selected);
    assert!(!run(&mut e, &lock(0)).changed); // repeat is a no-op
    // 47/13 byte 1 = locked = auto-switch clear.
    assert!(run(&mut e, &lock(1)).changed);
    assert!(!e.state().auto_switch_selected);
    assert!(!run(&mut e, &lock(1)).changed);
    // Persists.
    run(&mut e, &lock(0));
    let saved: QuestState =
        serde_json::from_slice(&serde_json::to_vec(e.state()).unwrap()).unwrap();
    let resumed = QuestEngine::new(CHARACTER, definitions(), eligibility(20), Some(saved)).unwrap();
    assert!(resumed.state().auto_switch_selected);
}

#[test]
fn locking_touches_neither_selection_nor_progress() {
    let mut e = engine(20);
    activate(&mut e, 10, 1);
    run(&mut e, &select(10));
    let quests = e.state().quests.clone();
    let outcome = run(&mut e, &lock(0));
    assert!(outcome.outbound.is_empty() && outcome.rewards.is_empty());
    assert_eq!(e.state().selected_quest_id, Some(10));
    assert_eq!(e.state().quests, quests);
}

#[test]
fn breadcrumb_makes_the_normal_offer_and_never_completes_anything() {
    let mut e = engine(20);
    let outcome = run(&mut e, &crumb(10));
    assert!(outcome.changed && outcome.denial.is_none());
    assert_eq!(info_ids(&outcome), vec![10]);
    assert_eq!(outcome.outbound.len(), 1);
    assert_eq!(e.state().pending_offer, Some(10));
    assert_eq!(e.progress(10).unwrap().status, QuestStatus::Offered);
    assert_no_award(&e, &outcome);
    // Pressing it again while the offer is open re-shows the same offer.
    let again = run(&mut e, &crumb(10));
    assert!(!again.changed);
    assert_eq!(info_ids(&again), vec![10]);
    // The player still has to accept; then it is active with zero progress.
    assert!(e.accept(10).changed);
    assert_eq!(e.progress(10).unwrap().status, QuestStatus::Active);
    assert_eq!(e.progress(10).unwrap().counts, vec![0]);
    assert_no_award(&e, &outcome);
}

#[test]
fn breadcrumb_for_an_active_quest_just_selects_it() {
    let mut e = engine(20);
    activate(&mut e, 14, 4);
    let outcome = run(&mut e, &crumb(14));
    assert!(outcome.changed && outcome.denial.is_none() && outcome.outbound.is_empty());
    assert_eq!(e.state().selected_quest_id, Some(14));
    assert_no_award(&e, &outcome);
    // Works for a quest that is not in the breadcrumb registry too: it is owned.
    activate(&mut e, 15, 6);
    assert!(run(&mut e, &crumb(15)).changed);
    assert_eq!(e.state().selected_quest_id, Some(15));
}

#[test]
fn breadcrumb_refusals_leave_state_untouched() {
    let mut e = engine(20);
    let fresh = e.state().clone();
    let cases = [
        (12, QuestDenial::Unavailable),  // hollow content
        (11, QuestDenial::Prerequisite), // prerequisite quest not completed
        (13, QuestDenial::Level),        // level 50 needed, character is 10
        (15, QuestDenial::Unavailable),  // real quest, not a registered breadcrumb
        (999_999, QuestDenial::Unavailable),
        (i32::MAX, QuestDenial::Unavailable),
    ];
    for (id, denial) in cases {
        let outcome = run(&mut e, &crumb(id as i32));
        assert!(!outcome.changed, "{id}");
        assert_eq!(outcome.denial, Some(denial), "{id}");
        assert!(info_ids(&outcome).is_empty(), "{id} must not open an offer");
        assert_eq!(e.state(), &fresh, "state moved for breadcrumb {id}");
        assert_no_award(&e, &outcome);
    }
}

#[test]
fn breadcrumb_cannot_be_used_to_skip_an_open_offer_or_finished_quest() {
    let mut e = engine(20);
    assert!(e.offer(14, 4).changed);
    let pending = e.state().clone();
    // Another offer is open: a second breadcrumb is refused.
    let outcome = run(&mut e, &crumb(10));
    assert_eq!(outcome.denial, Some(QuestDenial::Unavailable));
    assert_eq!(e.state(), &pending);
    assert!(e.decline(14).changed);

    // A quest that is completed cannot be restarted.
    let mut done = e.state().clone();
    done.quests.insert(
        10,
        osfr_tools::quest_engine::QuestProgress {
            status: QuestStatus::Completed,
            counts: vec![3],
        },
    );
    let mut resumed = QuestEngine::new(
        CHARACTER,
        definitions(),
        eligibility(20),
        Some(done.clone()),
    )
    .unwrap();
    let outcome = run(&mut resumed, &crumb(10));
    assert_eq!(outcome.denial, Some(QuestDenial::Unavailable));
    assert_eq!(resumed.state(), &done);
    // With 10 done, 11's prerequisite is met and its breadcrumb now works.
    let unlocked = run(&mut resumed, &crumb(11));
    assert_eq!(info_ids(&unlocked), vec![11]);
}

#[test]
fn breadcrumb_respects_the_quest_limit_and_sends_the_limit_notice() {
    let mut e = engine(1);
    activate(&mut e, 14, 4);
    let state = e.state().clone();
    let outcome = run(&mut e, &crumb(10));
    assert_eq!(outcome.denial, Some(QuestDenial::Limit));
    assert!(info_ids(&outcome).is_empty());
    assert!(
        outcome
            .outbound
            .contains(&OutboundMessage::Quest(QuestMessage::QuestLimit {
                member: true
            }))
    );
    assert_eq!(e.state(), &state);
}

#[test]
fn a_replayed_breadcrumb_burst_makes_one_offer_and_one_quest() {
    let mut e = engine(20);
    let mut shown = 0;
    for _ in 0..50 {
        shown += info_ids(&run(&mut e, &crumb(10))).len();
    }
    assert_eq!(shown, 50); // the offer window is re-shown, nothing else happens
    assert_eq!(e.state().quests.len(), 1);
    assert_eq!(e.progress(10).unwrap().status, QuestStatus::Offered);
    assert!(e.accept(10).changed);
    // Once accepted, replays only re-select; they never add a second record.
    for _ in 0..50 {
        run(&mut e, &crumb(10));
    }
    assert_eq!(e.state().quests.len(), 1);
    assert_eq!(e.progress(10).unwrap().status, QuestStatus::Active);
    assert_eq!(e.progress(10).unwrap().counts, vec![0]);
}
