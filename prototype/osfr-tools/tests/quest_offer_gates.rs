use osfr_tools::{
    npc_interaction::NpcInteraction,
    quest_engine::{
        CharacterEligibility, ObjectiveKind, OutboundMessage, QuestDefinition, QuestDenial,
        QuestEngine, QuestMessage, QuestObjective, QuestRequirements, QuestStatus,
    },
    rewards::{RewardDisplay, RewardGrant},
};
use std::{collections::BTreeMap, path::PathBuf};

const CHARACTER: u64 = 7;
const QUEST_GIVER: u64 = 9_000_000_047;
const EMPTY_QUEST: u32 = 3_110_005;
const VALID_QUEST: u32 = 3_110_006;

fn eligibility(player_level: u8) -> CharacterEligibility {
    CharacterEligibility {
        player_level,
        job_levels: BTreeMap::new(),
        member: true,
        nonmember_quest_limit: 20,
        member_quest_limit: 20,
    }
}

fn definition(
    id: u32,
    objectives: Vec<QuestObjective>,
    requirements: QuestRequirements,
) -> QuestDefinition {
    QuestDefinition {
        id,
        title: format!("Synthetic quest {id}"),
        description: "Synthetic quest body".into(),
        giver_npc_id: QUEST_GIVER,
        turnin_npc_id: QUEST_GIVER,
        prerequisite_quest_ids: Vec::new(),
        requirements,
        ordered: false,
        objectives,
        reward: RewardGrant {
            coins: 0,
            items: Vec::new(),
            job_xp: BTreeMap::new(),
            display: RewardDisplay {
                source_guid: QUEST_GIVER,
                player_guid: CHARACTER,
                reward_kind: 0,
                icon_id: 0,
                name_id: 1,
                rows: Vec::new(),
            },
        },
    }
}

fn valid_objective() -> Vec<QuestObjective> {
    vec![QuestObjective {
        id: 1,
        required: 1,
        description: "Synthetic talk objective".into(),
        kind: ObjectiveKind::TalkToNpc {
            npc_id: QUEST_GIVER,
        },
    }]
}

fn npc_interaction() -> NpcInteraction {
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
    NpcInteraction::load(&data.join("npcs"), &data.join("stores.json")).unwrap()
}

#[test]
fn objective_less_quest_is_hidden_and_skipped_by_the_talk_offer_path() {
    let interaction = npc_interaction();
    let empty = definition(EMPTY_QUEST, Vec::new(), QuestRequirements::default());
    let mut empty_only =
        QuestEngine::new(CHARACTER, vec![empty.clone()], eligibility(10), None).unwrap();

    assert!(!empty_only.is_enabled(EMPTY_QUEST));
    assert_eq!(
        interaction.offerable_quest_for_talk(QUEST_GIVER, &empty_only),
        None
    );
    let before = empty_only.state().clone();
    let refused = empty_only.offer(EMPTY_QUEST, QUEST_GIVER);
    assert_eq!(refused.denial, Some(QuestDenial::Unavailable));
    assert!(!refused.changed);
    assert!(refused.outbound.is_empty());
    assert_eq!(empty_only.state(), &before);

    let valid = definition(VALID_QUEST, valid_objective(), QuestRequirements::default());
    let mut mixed = QuestEngine::new(CHARACTER, vec![empty, valid], eligibility(10), None).unwrap();
    assert_eq!(
        interaction.offerable_quest_for_talk(QUEST_GIVER, &mixed),
        Some(VALID_QUEST),
        "the empty quest is skipped instead of consuming the talk choice"
    );
    let offered = mixed.offer(VALID_QUEST, QUEST_GIVER);
    assert!(offered.changed);
    assert!(matches!(
        offered.outbound.as_slice(),
        [OutboundMessage::Quest(QuestMessage::QuestInfo { quest_id })]
            if *quest_id == VALID_QUEST
    ));
    assert_eq!(mixed.state().pending_offer, Some(VALID_QUEST));
    assert_eq!(
        mixed.progress(VALID_QUEST).unwrap().status,
        QuestStatus::Offered
    );
}

#[test]
fn under_level_offer_is_refused_without_mutation_then_qualifies() {
    let mut requirements = QuestRequirements::default();
    requirements.player_level = Some(3);
    let mut engine = QuestEngine::new(
        CHARACTER,
        vec![definition(VALID_QUEST, valid_objective(), requirements)],
        eligibility(2),
        None,
    )
    .unwrap();

    let before = engine.state().clone();
    let refused = engine.offer(VALID_QUEST, QUEST_GIVER);
    assert_eq!(refused.denial, Some(QuestDenial::Level));
    assert!(!refused.changed);
    assert!(refused.outbound.is_empty());
    assert_eq!(engine.state(), &before);
    assert!(engine.progress(VALID_QUEST).is_none());

    engine.set_eligibility(eligibility(3));
    let offered = engine.offer(VALID_QUEST, QUEST_GIVER);
    assert!(offered.changed);
    assert!(matches!(
        offered.outbound.as_slice(),
        [OutboundMessage::Quest(QuestMessage::QuestInfo { quest_id })]
            if *quest_id == VALID_QUEST
    ));
    assert_eq!(engine.state().pending_offer, Some(VALID_QUEST));
}

#[test]
fn job_requirement_uses_the_matching_job_level_and_refusals_are_atomic() {
    let mut requirements = QuestRequirements::default();
    requirements.player_level = Some(3);
    requirements.job = Some(("brawler".into(), 3));
    let mut engine = QuestEngine::new(
        CHARACTER,
        vec![definition(VALID_QUEST, valid_objective(), requirements)],
        eligibility(20),
        None,
    )
    .unwrap();

    let before = engine.state().clone();
    let refused = engine.offer(VALID_QUEST, QUEST_GIVER);
    assert_eq!(refused.denial, Some(QuestDenial::Job));
    assert!(!refused.changed);
    assert!(refused.outbound.is_empty());
    assert_eq!(engine.state(), &before);

    let mut other_job = eligibility(20);
    other_job.job_levels.insert("chef".into(), 20);
    engine.set_eligibility(other_job);
    assert_eq!(
        engine.offer(VALID_QUEST, QUEST_GIVER).denial,
        Some(QuestDenial::Job)
    );
    assert_eq!(engine.state(), &before);

    let mut under_level_job = eligibility(20);
    under_level_job.job_levels.insert("brawler".into(), 2);
    engine.set_eligibility(under_level_job);
    assert_eq!(
        engine.offer(VALID_QUEST, QUEST_GIVER).denial,
        Some(QuestDenial::Job)
    );
    assert_eq!(engine.state(), &before);

    let mut qualified = eligibility(3);
    qualified.job_levels.insert("brawler".into(), 3);
    engine.set_eligibility(qualified);
    let offered = engine.offer(VALID_QUEST, QUEST_GIVER);
    assert!(offered.changed);
    assert_eq!(engine.state().pending_offer, Some(VALID_QUEST));
}
