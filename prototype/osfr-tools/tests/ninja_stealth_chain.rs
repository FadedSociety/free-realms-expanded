use osfr_tools::{
    inventory::Inventory,
    npc_interaction::{NpcAction, NpcInteraction, NpcSelection},
    profiles::ProfileState,
    quest_engine::{
        CharacterEligibility, OutboundMessage, QuestDenial, QuestEngine, QuestEvent, QuestMessage,
        QuestOutcome, QuestProgress, QuestState, QuestStatus,
    },
    rewards::RewardState,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const CHARACTER: u64 = 700_107;
const TY: u64 = 9_000_000_223;
const BRENDAN: u64 = 9_500_000_561;
const KOSHA: u64 = 9_500_000_697;
const CHLOE: u64 = 9_000_000_047;
const UNRELATED: u64 = 9_000_000_189;
const NINJA_PATH: u32 = 3_110_001;
const STOLEN_AWAY: u32 = 3_110_002;
const HALTED_MONGO_CHAIN: u32 = 3_110_003;
const STEALTH_REFERRAL: u32 = 3_110_004;
const STEALTH_REWARD_COINS: u32 = 5;

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn eligibility() -> CharacterEligibility {
    CharacterEligibility {
        player_level: 20,
        job_levels: BTreeMap::new(),
        member: true,
        nonmember_quest_limit: 30,
        member_quest_limit: 30,
    }
}

fn completed_through_stolen_away() -> QuestState {
    QuestState {
        version: 1,
        character_id: CHARACTER,
        quests: BTreeMap::from([
            (
                NINJA_PATH,
                QuestProgress {
                    status: QuestStatus::Completed,
                    counts: vec![1],
                },
            ),
            (
                STOLEN_AWAY,
                QuestProgress {
                    status: QuestStatus::Completed,
                    counts: vec![1],
                },
            ),
        ]),
        pending_offer: None,
        pending_turnin: None,
        selected_quest_id: None,
        auto_switch_selected: false,
    }
}

fn engine_with_history() -> QuestEngine {
    QuestEngine::from_split_content(
        &content_dir(),
        CHARACTER,
        eligibility(),
        Some(completed_through_stolen_away()),
    )
    .unwrap()
}

fn npc_interaction() -> NpcInteraction {
    let data = content_dir();
    NpcInteraction::load(&data.join("npcs"), &data.join("stores.json")).unwrap()
}

fn offer_and_accept_stealth(interaction: &NpcInteraction, engine: &mut QuestEngine) {
    assert_eq!(
        interaction.offerable_quest_for_talk(TY, engine),
        Some(STEALTH_REFERRAL)
    );
    assert!(engine.offer(STEALTH_REFERRAL, TY).changed);
    assert!(engine.accept(STEALTH_REFERRAL).changed);
}

fn select_talk(interaction: &mut NpcInteraction, engine: &QuestEngine, npc_id: u64) {
    let (zone_id, position) = interaction.location(npc_id).expect("spawned target");
    let menu = interaction
        .interact(npc_id, zone_id, position, Some(engine))
        .unwrap();
    let choice_id = menu
        .options
        .iter()
        .find(|option| option.action == NpcAction::Talk)
        .expect("normal Talk option")
        .id;
    let mut packet = vec![26, 0, 10, 0];
    packet.extend_from_slice(&npc_id.to_le_bytes());
    packet.extend_from_slice(&choice_id.to_le_bytes());
    assert!(matches!(
        interaction
            .select(&packet, zone_id, position, None)
            .unwrap(),
        NpcSelection::Talk(_)
    ));
}

fn talk(interaction: &mut NpcInteraction, engine: &mut QuestEngine, npc_id: u64) -> QuestOutcome {
    select_talk(interaction, engine, npc_id);
    engine.on_event(QuestEvent::TalkToNpc { npc_id }).unwrap()
}

fn assert_talk_changes_nothing(
    interaction: &mut NpcInteraction,
    engine: &mut QuestEngine,
    npc_id: u64,
) {
    let before = engine.state().clone();
    let outcome = talk(interaction, engine, npc_id);
    assert!(!outcome.changed);
    assert!(outcome.outbound.is_empty());
    assert_eq!(engine.state(), &before);
}

fn complete_three_mentors_and_return(interaction: &mut NpcInteraction, engine: &mut QuestEngine) {
    for npc_id in [BRENDAN, KOSHA, CHLOE] {
        assert!(talk(interaction, engine, npc_id).changed);
    }
    assert_eq!(engine.state().pending_turnin, None);

    let returned = talk(interaction, engine, TY);
    assert!(returned.changed);
    assert_eq!(engine.state().pending_turnin, Some(STEALTH_REFERRAL));
    assert!(returned.outbound.iter().any(|message| matches!(
        message,
        OutboundMessage::Quest(QuestMessage::QuestEnd {
            quest_id: STEALTH_REFERRAL
        })
    )));
}

struct RewardFixture(PathBuf);

impl RewardFixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-ninja-stealth-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        let zero = 0i32.to_le_bytes();
        let item = [500i32.to_le_bytes(), 8i32.to_le_bytes()].concat();
        let content = json!({
            "schema_version": 1,
            "player_guid": CHARACTER,
            "default_state": {
                "coins": 0,
                "items": [],
                "equipment": [],
                "next_instance_id": 1
            },
            "profiles": [],
            "spans": [
                {"kind":"coins","offset":13,"length":4,"profile_id":0,"sha256":digest(&zero)},
                {"kind":"items","offset":17,"length":4,"profile_id":0,"sha256":digest(&zero)}
            ],
            "definitions": [{
                "id": 500,
                "slot": 3,
                "class_id": 1,
                "default_tint": 0,
                "max_stack": 99,
                "model": "synthetic.adr",
                "texture": "synthetic",
                "tint_alias": "synthetic",
                "effect": 0,
                "wield_type": 0,
                "serialized_hex": hex(&item),
                "sha256": digest(&item)
            }]
        });
        fs::write(
            directory.join("inventory.json"),
            serde_json::to_vec(&content).unwrap(),
        )
        .unwrap();
        Self(directory)
    }

    fn reward_state(&self) -> RewardState {
        RewardState::new(
            Inventory::load(&self.0, None).unwrap(),
            ProfileState {
                version: 1,
                active: 1,
                unlocked: BTreeSet::new(),
                experience: BTreeMap::new(),
            },
            16,
        )
    }
}

impl Drop for RewardFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[test]
fn ty_offers_the_stealth_referral_after_stolen_away() {
    let mut engine = engine_with_history();
    let mut interaction = npc_interaction();
    select_talk(&mut interaction, &engine, TY);
    assert_eq!(
        interaction.offerable_quest_for_talk(TY, &engine),
        Some(STEALTH_REFERRAL)
    );

    let offered = engine.offer(STEALTH_REFERRAL, TY);
    assert!(offered.changed);
    assert!(matches!(
        offered.outbound.as_slice(),
        [OutboundMessage::Quest(QuestMessage::QuestInfo {
            quest_id: STEALTH_REFERRAL
        })]
    ));
    assert_eq!(engine.state().pending_offer, Some(STEALTH_REFERRAL));
    assert!(engine.accept(STEALTH_REFERRAL).changed);
}

#[test]
fn three_mentors_are_required_in_order() {
    let mut engine = engine_with_history();
    let mut interaction = npc_interaction();
    offer_and_accept_stealth(&interaction, &mut engine);

    for npc_id in [KOSHA, CHLOE, TY, UNRELATED] {
        assert_talk_changes_nothing(&mut interaction, &mut engine, npc_id);
    }
    assert_eq!(
        engine.progress(STEALTH_REFERRAL).unwrap().counts,
        vec![0, 0, 0, 0]
    );

    assert!(talk(&mut interaction, &mut engine, BRENDAN).changed);
    assert_eq!(
        engine.progress(STEALTH_REFERRAL).unwrap().counts,
        vec![1, 0, 0, 0]
    );
    assert_talk_changes_nothing(&mut interaction, &mut engine, BRENDAN);
    assert_talk_changes_nothing(&mut interaction, &mut engine, CHLOE);

    assert!(talk(&mut interaction, &mut engine, KOSHA).changed);
    assert_eq!(
        engine.progress(STEALTH_REFERRAL).unwrap().counts,
        vec![1, 1, 0, 0]
    );
    assert_talk_changes_nothing(&mut interaction, &mut engine, KOSHA);

    assert!(talk(&mut interaction, &mut engine, CHLOE).changed);
    assert_eq!(
        engine.progress(STEALTH_REFERRAL).unwrap().counts,
        vec![1, 1, 1, 0]
    );
    assert_talk_changes_nothing(&mut interaction, &mut engine, CHLOE);
    assert_eq!(engine.state().pending_turnin, None);
}

#[test]
fn return_to_ty_settles_once() {
    let mut engine = engine_with_history();
    let mut interaction = npc_interaction();
    offer_and_accept_stealth(&interaction, &mut engine);
    complete_three_mentors_and_return(&mut interaction, &mut engine);

    let rewards = RewardFixture::new();
    let initial = rewards.reward_state();
    assert_eq!(initial.inventory.state().coins, 0);

    let (settled, paid) = engine.finish_turnin(STEALTH_REFERRAL, &initial).unwrap();
    assert!(settled.changed);
    assert_eq!(settled.rewards.len(), 1);
    assert_eq!(settled.rewards[0].quest_id, STEALTH_REFERRAL);
    assert_eq!(settled.rewards[0].coins, STEALTH_REWARD_COINS);
    assert_eq!(paid.inventory.state().coins, STEALTH_REWARD_COINS);
    assert!(
        paid.applied_grant_keys
            .contains(&format!("quest:{CHARACTER}:{STEALTH_REFERRAL}"))
    );
    assert_eq!(
        engine.progress(STEALTH_REFERRAL).unwrap().status,
        QuestStatus::Completed
    );

    let before = engine.state().clone();
    let (replay, replay_paid) = engine.finish_turnin(STEALTH_REFERRAL, &paid).unwrap();
    assert!(!replay.changed);
    assert!(replay.outbound.is_empty());
    assert!(replay.rewards.is_empty());
    assert_eq!(engine.state(), &before);
    assert_eq!(
        replay_paid.inventory.state().coins,
        paid.inventory.state().coins
    );
    assert_eq!(replay_paid.applied_grant_keys, paid.applied_grant_keys);
}

#[test]
fn halted_mongo_chain_stays_hidden_and_save_reloads() {
    let mut engine = engine_with_history();
    let mut interaction = npc_interaction();
    assert!(!engine.is_enabled(HALTED_MONGO_CHAIN));
    assert!(!engine.can_offer(HALTED_MONGO_CHAIN, TY));

    let before = engine.state().clone();
    let refused = engine.offer(HALTED_MONGO_CHAIN, TY);
    assert_eq!(refused.denial, Some(QuestDenial::Unavailable));
    assert!(!refused.changed);
    assert!(refused.outbound.is_empty());
    assert_eq!(engine.state(), &before);

    offer_and_accept_stealth(&interaction, &mut engine);
    complete_three_mentors_and_return(&mut interaction, &mut engine);
    let rewards = RewardFixture::new();
    engine
        .finish_turnin(STEALTH_REFERRAL, &rewards.reward_state())
        .unwrap();

    let encoded = serde_json::to_vec(engine.state()).unwrap();
    let saved: QuestState = serde_json::from_slice(&encoded).unwrap();
    let mut reloaded =
        QuestEngine::from_split_content(&content_dir(), CHARACTER, eligibility(), Some(saved))
            .unwrap();
    assert_eq!(
        reloaded.progress(STEALTH_REFERRAL).unwrap().status,
        QuestStatus::Completed
    );
    assert_eq!(
        reloaded.progress(STEALTH_REFERRAL).unwrap().counts,
        vec![1, 1, 1, 1]
    );
    assert!(reloaded.progress(HALTED_MONGO_CHAIN).is_none());
    assert!(!reloaded.is_enabled(HALTED_MONGO_CHAIN));

    let before = reloaded.state().clone();
    let refused = reloaded.offer(HALTED_MONGO_CHAIN, TY);
    assert_eq!(refused.denial, Some(QuestDenial::Unavailable));
    assert!(!refused.changed);
    assert_eq!(reloaded.state(), &before);
}
