//! T790: an `any_of` quest objective counts any one of several same-family targets.
//!
//! The split tree is the production quest path (`QuestEngine::from_split_content`, the
//! loader `Gameplay::bind_npc_quest_catalogue` calls); the flat tree drives the
//! `ContentDb::load` loader that owns the same authored JSON. The fishing test drives the
//! real `FishingService::route` request path from `tests/support/fishing_packets.rs`, then
//! hands the server-generated events to the engine, so a paid catch (not a synthetic
//! event) is what completes the goal.
//!
//! Client evidence: the objective counter is client-reader 49/9 (`00c7b5e0` body
//! `00c7af60`: quest, objective, current, progress, bool, dword) and 49/10
//! (`00c7b070`); neither carries a target id, so an any-of goal sends exactly the same
//! counter as a single-target goal. No wire change.

#[path = "support/fishing_packets.rs"]
mod fishing_packets;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use fishing_packets::{cast_at, lifecycle, reel, register};
use npc_quest_chain::content_dir;
use osfr_tools::{
    content::ContentDb,
    fishing::Handle,
    fishing_service::{FishingService, FishingSpots},
    quest_engine::{
        CharacterEligibility, ObjectiveKind, OutboundMessage, QuestDefinition, QuestEngine,
        QuestEvent, QuestMessage, QuestObjective, QuestRequirements,
    },
    quest_markers::{QuestMarker, marker_for},
    rewards::{RewardDisplay, RewardGrant},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const QUEST: u32 = 9_900_001;
const GIVER: u64 = 9_000_000_189;
const CHARACTER: u64 = 1;
const ITEM_A: i32 = 500;
const ITEM_B: i32 = 501;
const NPC_A: u64 = 101;
const NPC_B: u64 = 102;
const NPC_C: u64 = 103;
/// `tests/fixtures/content/npcs.json`'s one actor, present in the flat loader's NPC set.
const FIXTURE_NPC: u64 = 2_732_686_529_148_769_137;
const FIXTURE_QUEST: u32 = 101;
/// The three Cheery Salmon item definitions (`fishing_spots.json` `items.cheery_salmon`:
/// `[68062, 68063, 68064, 0]`), the variants the specs draft for 3070040.
const CHEERY_SALMON: [i32; 3] = [68_062, 68_063, 68_064];
const PLAYER: u64 = 0x0000_00a1_b2c3_d4e5;
/// Reed Stillwater, the Sacred Grove Shallows fishing guide (`fishing_spots.json`).
const REED: u64 = 9_000_000_181;
const ACTOR: Handle = Handle {
    id: 0x1122_3344,
    seq: 0x5566_7788,
};
const FISH_T0: u64 = 1_000_000;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "t790-{tag}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("quests")).unwrap();
        fs::create_dir_all(root.join("npcs")).unwrap();
        Self(root)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn source() -> Value {
    json!([{
        "kind": "reconstructed",
        "evidence": "(R) T790 synthetic any_of objective fixture"
    }])
}

fn write_json(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap())
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
}

fn any_of(id: u32, count: u32, alternatives: Value) -> Value {
    json!({
        "kind": "any_of",
        "id": id,
        "count": count,
        "alternatives": alternatives,
        "source": source(),
    })
}

fn collect(item_id: i32) -> Value {
    json!({"kind": "collect_item", "item_id": item_id})
}

fn gather(resource_id: u32) -> Value {
    json!({"kind": "gather_resource", "resource_id": resource_id})
}

fn talk(npc_id: u64) -> Value {
    json!({"kind": "talk_to_npc", "npc_id": npc_id})
}

fn talk_goal(id: u32, npc_id: u64) -> Value {
    json!({"kind": "talk_to_npc", "id": id, "npc_id": npc_id, "source": source()})
}

fn split_tree(objectives: Value) -> Scratch {
    let scratch = Scratch::new("split");
    write_json(
        &scratch.path().join("quests/t790.json"),
        &json!({
            "schema_version": 1,
            "records": [{
                "id": QUEST,
                "title_string_id": 1,
                "giver_npc_id": GIVER,
                "prerequisite_quest_ids": [],
                "objectives": objectives,
                "rewards": [],
                "source": source(),
            }],
        }),
    );
    write_json(
        &scratch.path().join("npcs/bindings.json"),
        &json!({"schema_version": 1, "records": []}),
    );
    write_json(
        &scratch.path().join("jobs.json"),
        &json!({
            "schema_version": 1,
            "records": [{
                "id": "t790",
                "name": "T790 fixture",
                "profile_id": 4,
                "category": "combat",
                "max_level": 20,
                "members_only": false,
                "trainer_npc_ids": [],
                "quest_ids": [QUEST],
                "abilities": [],
                "source": source(),
            }],
        }),
    );
    scratch
}

fn eligibility() -> CharacterEligibility {
    CharacterEligibility {
        player_level: 20,
        member: true,
        nonmember_quest_limit: 10,
        member_quest_limit: 10,
        ..Default::default()
    }
}

fn split_load(scratch: &Scratch) -> std::io::Result<QuestEngine> {
    QuestEngine::from_split_content(scratch.path(), CHARACTER, eligibility(), None)
}

fn split_engine(scratch: &Scratch) -> QuestEngine {
    split_load(scratch).expect("the T790 split fixture loads")
}

fn activate(engine: &mut QuestEngine) {
    assert!(engine.offer(QUEST, GIVER).changed);
    assert!(engine.accept(QUEST).changed);
}

fn counts(engine: &QuestEngine) -> Vec<u32> {
    engine
        .progress(QUEST)
        .unwrap_or_else(|| panic!("quest {QUEST} is active"))
        .counts
        .clone()
}

fn objective_completed(outcome: &osfr_tools::quest_engine::QuestOutcome) -> bool {
    outcome.outbound.iter().any(|message| {
        matches!(
            message,
            OutboundMessage::Quest(QuestMessage::ObjectiveComplete { quest_id, .. })
                if *quest_id == QUEST
        )
    })
}

/// The flat `ContentDb` fixture: the shared `tests/fixtures/content` rows plus the quests
/// and items this suite needs. The NPC row already links quest 101.
fn flat_tree(objectives: Value, item_ids: &[i32]) -> Scratch {
    let scratch = Scratch::new("flat");
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content");
    for name in [
        "dungeons.json",
        "jobs.json",
        "minigames.json",
        "npcs.json",
        "pois.json",
        "zones.json",
    ] {
        fs::copy(fixture.join(name), scratch.path().join(name)).unwrap();
    }
    let records: Vec<Value> = item_ids
        .iter()
        .map(|id| {
            json!({
                "id": id,
                "name_string_id": 700_000 + id,
                "description_string_id": null,
                "category_id": null,
                "source": source(),
            })
        })
        .collect();
    write_json(
        &scratch.path().join("items.json"),
        &json!({"schema_version": 1, "records": records}),
    );
    write_json(
        &scratch.path().join("quests.json"),
        &json!({
            "schema_version": 1,
            "records": [{
                "id": FIXTURE_QUEST,
                "title_string_id": null,
                "giver_npc_id": FIXTURE_NPC,
                "prerequisite_quest_ids": [],
                "objectives": objectives,
                "rewards": [],
                "source": source(),
            }],
        }),
    );
    scratch
}

#[test]
fn duplicate_alternative_is_rejected_at_load() {
    let control = split_tree(json!([any_of(
        1,
        1,
        json!([collect(ITEM_A), collect(ITEM_B)])
    )]));
    assert!(
        split_load(&control).is_ok(),
        "the same-family split control must load"
    );
    let duplicate = split_tree(json!([any_of(
        1,
        1,
        json!([collect(ITEM_A), collect(ITEM_A)])
    )]));
    assert!(
        split_load(&duplicate).is_err(),
        "the split loader accepted a repeated any_of target"
    );

    let flat_control = flat_tree(
        json!([any_of(1, 1, json!([collect(ITEM_A), collect(ITEM_B)]))]),
        &[ITEM_A, ITEM_B],
    );
    assert!(
        ContentDb::load(flat_control.path()).is_ok(),
        "the same-family flat control must load"
    );
    let flat_duplicate = flat_tree(
        json!([any_of(1, 1, json!([collect(ITEM_A), collect(ITEM_A)]))]),
        &[ITEM_A, ITEM_B],
    );
    assert!(
        ContentDb::load(flat_duplicate.path()).is_err(),
        "the flat loader accepted a repeated any_of target"
    );
}

#[test]
fn mixed_family_is_rejected_at_load() {
    // Both ids resolve in the flat fixture (its one NPC and item 500), so only the
    // same-family rule can refuse this list.
    let talk_and_collect = json!([any_of(1, 1, json!([talk(FIXTURE_NPC), collect(ITEM_A)]))]);
    let split = split_tree(talk_and_collect.clone());
    assert!(
        split_load(&split).is_err(),
        "the split loader accepted a mixed-family any_of"
    );
    let flat = flat_tree(talk_and_collect, &[ITEM_A]);
    assert!(
        ContentDb::load(flat.path()).is_err(),
        "the flat loader accepted a mixed-family any_of"
    );
}

#[test]
fn too_many_alternatives_is_rejected_at_load() {
    let eight: Vec<Value> = (0..8).map(|offset| collect(ITEM_A + offset)).collect();
    let nine: Vec<Value> = (0..9).map(|offset| collect(ITEM_A + offset)).collect();
    let split_control = split_tree(json!([any_of(1, 1, json!(eight))]));
    assert!(
        split_load(&split_control).is_ok(),
        "eight split alternatives are the bound and must load"
    );
    let split = split_tree(json!([any_of(1, 1, json!(nine))]));
    assert!(
        split_load(&split).is_err(),
        "the split loader accepted nine any_of alternatives"
    );

    let ids: Vec<i32> = (0..9).map(|offset| ITEM_A + offset).collect();
    let flat_control = flat_tree(
        json!([any_of(
            1,
            1,
            json!(ids[..8].iter().map(|id| collect(*id)).collect::<Vec<_>>())
        )]),
        &ids,
    );
    assert!(
        ContentDb::load(flat_control.path()).is_ok(),
        "eight flat alternatives must load"
    );
    let flat = flat_tree(
        json!([any_of(
            1,
            1,
            json!(ids.iter().map(|id| collect(*id)).collect::<Vec<_>>())
        )]),
        &ids,
    );
    assert!(
        ContentDb::load(flat.path()).is_err(),
        "the flat loader accepted nine any_of alternatives"
    );
}

#[test]
fn non_member_changes_nothing() {
    let scratch = split_tree(json!([any_of(
        1,
        2,
        json!([collect(ITEM_A), collect(ITEM_B)])
    )]));
    let mut engine = split_engine(&scratch);
    activate(&mut engine);
    let before = serde_json::to_vec(engine.state()).unwrap();
    for event in [
        // A collect id outside the alternatives.
        QuestEvent::CollectItem {
            item_id: 999,
            quantity: 5,
        },
        // A same-id event of another family cannot match a collect alternative.
        QuestEvent::GatherResource {
            resource_id: ITEM_A as u32,
            quantity: 5,
        },
        QuestEvent::TalkToNpc { npc_id: NPC_A },
    ] {
        let outcome = engine.on_event(event.clone()).unwrap();
        assert!(!outcome.changed, "{event:?} changed the quest");
        assert!(outcome.outbound.is_empty(), "{event:?} sent a message");
        assert!(outcome.messages.is_empty(), "{event:?} produced a line");
        assert_eq!(
            serde_json::to_vec(engine.state()).unwrap(),
            before,
            "{event:?} changed the saved state"
        );
    }
    assert_eq!(counts(&engine), [0]);
}

#[test]
fn mixed_alternatives_share_one_total() {
    let scratch = split_tree(json!([any_of(
        1,
        3,
        json!([talk(NPC_A), talk(NPC_B), talk(NPC_C)])
    )]));
    let mut engine = split_engine(&scratch);
    activate(&mut engine);
    for (npc, expected) in [(NPC_A, 1), (NPC_B, 2), (NPC_C, 3)] {
        let outcome = engine
            .on_event(QuestEvent::TalkToNpc { npc_id: npc })
            .unwrap();
        assert!(outcome.changed);
        assert_eq!(counts(&engine), [expected]);
        assert_eq!(objective_completed(&outcome), expected == 3);
    }
    let again = engine
        .on_event(QuestEvent::TalkToNpc { npc_id: NPC_A })
        .unwrap();
    assert!(!again.changed, "a completed any_of counted again");
    assert_eq!(counts(&engine), [3]);
}

#[test]
fn one_alternative_repeated_completes() {
    let scratch = split_tree(json!([any_of(1, 3, json!([talk(NPC_A), talk(NPC_B)]))]));
    let mut engine = split_engine(&scratch);
    activate(&mut engine);
    for expected in [1, 2, 3] {
        let outcome = engine
            .on_event(QuestEvent::TalkToNpc { npc_id: NPC_A })
            .unwrap();
        assert!(outcome.changed);
        assert_eq!(counts(&engine), [expected]);
    }
    assert!(
        !engine
            .on_event(QuestEvent::TalkToNpc { npc_id: NPC_B })
            .unwrap()
            .changed,
        "the second alternative counted after completion"
    );
}

#[test]
fn collect_quantity_counts_fully() {
    let scratch = split_tree(json!([any_of(
        1,
        3,
        json!([collect(ITEM_A), collect(ITEM_B)])
    )]));
    let mut engine = split_engine(&scratch);
    activate(&mut engine);
    let outcome = engine
        .on_event(QuestEvent::CollectItem {
            item_id: ITEM_B,
            quantity: 3,
        })
        .unwrap();
    assert!(outcome.changed);
    assert_eq!(counts(&engine), [3], "a quantity of 3 must count 3");
    assert!(objective_completed(&outcome));
}

#[test]
fn gather_alternatives_count_their_quantity() {
    let scratch = split_tree(json!([any_of(1, 3, json!([gather(1_001), gather(2_889)]))]));
    let mut engine = split_engine(&scratch);
    activate(&mut engine);
    let first = engine
        .on_event(QuestEvent::GatherResource {
            resource_id: 2_889,
            quantity: 2,
        })
        .unwrap();
    assert!(first.changed);
    assert_eq!(counts(&engine), [2]);
    let second = engine
        .on_event(QuestEvent::GatherResource {
            resource_id: 1_001,
            quantity: 1,
        })
        .unwrap();
    assert!(second.changed);
    assert_eq!(counts(&engine), [3]);
    assert!(objective_completed(&second));
}

/// A repeated leaf is impossible in authored content (both loaders refuse it), but the
/// runtime type accepts one and must still count a single event once: `event_matches`
/// uses the first matching alternative, never a sum over every alternative.
#[test]
fn one_event_counts_once() {
    let mut engine = direct_engine(
        ObjectiveKind::AnyOf {
            alternatives: vec![
                ObjectiveKind::TalkToNpc { npc_id: NPC_A },
                ObjectiveKind::TalkToNpc { npc_id: NPC_A },
            ],
        },
        2,
    );
    let outcome = engine
        .on_event(QuestEvent::TalkToNpc { npc_id: NPC_A })
        .unwrap();
    assert!(outcome.changed);
    assert_eq!(
        counts(&engine),
        [1],
        "one Talk counted for two alternatives"
    );
    assert!(!objective_completed(&outcome));
}

#[test]
fn any_of_inside_an_ordered_quest() {
    let scratch = split_tree(json!([
        any_of(1, 2, json!([talk(NPC_A), talk(NPC_B)])),
        talk_goal(2, GIVER),
    ]));
    let mut engine = split_engine(&scratch);
    activate(&mut engine);
    // Only the active ordered step counts: the second step is an ordinary Talk.
    assert!(
        !engine
            .on_event(QuestEvent::TalkToNpc { npc_id: GIVER })
            .unwrap()
            .changed,
        "a later ordered step counted before the any_of completed"
    );
    assert_eq!(counts(&engine), [0, 0]);
    assert!(
        engine
            .on_event(QuestEvent::TalkToNpc { npc_id: NPC_A })
            .unwrap()
            .changed
    );
    let second = engine
        .on_event(QuestEvent::TalkToNpc { npc_id: NPC_B })
        .unwrap();
    assert_eq!(counts(&engine), [2, 0]);
    assert!(
        second.outbound.iter().any(|message| matches!(
            message,
            OutboundMessage::Quest(QuestMessage::ObjectiveActivated {
                quest_id: QUEST,
                objective_id: 2,
                ..
            })
        )),
        "completing the any_of must activate the next ordered step"
    );
    let next = engine
        .on_event(QuestEvent::TalkToNpc { npc_id: GIVER })
        .unwrap();
    assert!(next.changed);
    assert_eq!(counts(&engine), [2, 1]);
}

#[test]
fn reload_keeps_any_of_progress() {
    let scratch = split_tree(json!([any_of(
        1,
        2,
        json!([collect(ITEM_A), collect(ITEM_B)])
    )]));
    let mut engine = split_engine(&scratch);
    activate(&mut engine);
    assert!(
        engine
            .on_event(QuestEvent::CollectItem {
                item_id: ITEM_A,
                quantity: 1,
            })
            .unwrap()
            .changed
    );
    assert_eq!(counts(&engine), [1]);
    let saved = serde_json::to_vec(engine.state()).unwrap();
    let state: osfr_tools::quest_engine::QuestState = serde_json::from_slice(&saved).unwrap();
    let mut reloaded =
        QuestEngine::from_split_content(scratch.path(), CHARACTER, eligibility(), Some(state))
            .expect("the saved any_of progress reloads");
    assert_eq!(counts(&reloaded), [1], "the reload lost the any_of count");
    assert_eq!(serde_json::to_vec(reloaded.state()).unwrap(), saved);
    assert!(
        reloaded
            .on_event(QuestEvent::CollectItem {
                item_id: ITEM_B,
                quantity: 1,
            })
            .unwrap()
            .changed
    );
    assert_eq!(counts(&reloaded), [2]);
}

#[test]
fn every_any_of_talk_alternative_marks_the_npc() {
    let scratch = split_tree(json!([any_of(
        1,
        1,
        json!([talk(NPC_A), talk(NPC_B), talk(NPC_C)])
    )]));
    let mut engine = split_engine(&scratch);
    activate(&mut engine);
    for npc in [NPC_A, NPC_B, NPC_C] {
        assert_eq!(
            marker_for(&engine, npc),
            Some(QuestMarker::TurnIn),
            "talk alternative {npc} must carry the turn-in marker"
        );
    }
    assert_eq!(
        marker_for(&engine, NPC_C + 1),
        None,
        "an NPC outside the alternatives must not carry a marker"
    );
}

/// A real paid catch from the production fishing request path. All shipped fish items are
/// bound and stored, so a non-Cheery-Salmon catch is a real, non-matching CollectItem; the
/// loop stops at the first Cheery Salmon size variant, and that one event completes the
/// any-of over the three variants.
#[test]
fn fish_variant_catch_credits_any_of() {
    let scratch = split_tree(json!([any_of(
        1,
        1,
        json!([
            collect(CHEERY_SALMON[0]),
            collect(CHEERY_SALMON[1]),
            collect(CHEERY_SALMON[2]),
        ])
    )]));
    let mut engine = split_engine(&scratch);
    activate(&mut engine);

    let inventory_dir = Scratch::new("fish-inventory");
    let mut rewards = fishing_packets::fish_inventory_rewards(inventory_dir.path());
    let spots = FishingSpots::load(&content_dir())
        .unwrap()
        .expect("the shipped fishing spots load");
    let mut service = FishingService::new();
    service
        .enter(
            &spots,
            REED,
            PLAYER,
            0,
            [0.0; 3],
            false,
            10,
            rewards.clone(),
        )
        .expect("the shipped fishing entry commits");
    let instance = service.instance_id(PLAYER).unwrap();
    service
        .handle_minigame_lifecycle(PLAYER, &lifecycle(5, instance))
        .unwrap();
    assert!(service.mark_started(PLAYER));

    let mut now = FISH_T0;
    let registered = service
        .route(
            PLAYER,
            &register(ACTOR),
            now,
            &rewards.inventory,
            &rewards.profiles,
        )
        .unwrap()
        .expect("the register request belongs to the round");
    assert!(!registered.replies.is_empty());
    for _ in 0..500 {
        now += 10_000;
        service
            .route(
                PLAYER,
                &cast_at(ACTOR, [0.0, 0.0, 0.0], 10.0),
                now,
                &rewards.inventory,
                &rewards.profiles,
            )
            .unwrap()
            .expect("the cast belongs to the round");
        service.tick(PLAYER, now + 1_510).unwrap();
        service.tick(PLAYER, now + 2_410).unwrap();
        let caught = service
            .route(
                PLAYER,
                &reel(ACTOR),
                now + 2_420,
                &rewards.inventory,
                &rewards.profiles,
            )
            .unwrap()
            .expect("the reel belongs to the round");
        if caught.paid {
            rewards = service.reward_state(PLAYER).unwrap().clone();
        }
        for event in caught.quest_events {
            let matched = match &event {
                QuestEvent::CollectItem { item_id, .. } => CHEERY_SALMON.contains(item_id),
                _ => false,
            };
            let outcome = engine.on_event(event).unwrap();
            if matched {
                assert!(outcome.changed);
                assert_eq!(counts(&engine), [1]);
                assert!(objective_completed(&outcome));
                return;
            }
            assert!(!outcome.changed, "a non-member catch credited the goal");
            assert!(
                outcome.outbound.is_empty(),
                "a non-member catch sent a message"
            );
        }
        assert_eq!(counts(&engine), [0], "a non-member catch credited the goal");
    }
    panic!("no Cheery Salmon size variant was caught in 500 attempts");
}

/// The direct (non-content) engine used for the repeated-leaf runtime rule.
fn direct_engine(kind: ObjectiveKind, required: u32) -> QuestEngine {
    let definition = QuestDefinition {
        id: QUEST,
        title: "T790".into(),
        description: String::new(),
        giver_npc_id: GIVER,
        turnin_npc_id: GIVER,
        prerequisite_quest_ids: Vec::new(),
        requirements: QuestRequirements::default(),
        ordered: true,
        objectives: vec![QuestObjective {
            id: 1,
            required,
            description: "T790".into(),
            kind,
        }],
        reward: RewardGrant {
            coins: 0,
            items: Vec::new(),
            job_xp: BTreeMap::new(),
            sc: 0,
            display: RewardDisplay {
                source_guid: GIVER,
                player_guid: CHARACTER,
                reward_kind: 0,
                icon_id: 0,
                name_id: 0,
                rows: Vec::new(),
            },
        },
    };
    let mut engine = QuestEngine::new(CHARACTER, vec![definition], eligibility(), None).unwrap();
    assert!(engine.offer(QUEST, GIVER).changed);
    assert!(engine.accept(QUEST).changed);
    engine
}
