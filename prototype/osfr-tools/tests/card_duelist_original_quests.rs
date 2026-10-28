//! T505: Card Duelist quests 3040011 and 3040034 are bound to the recorded originals
//! 1304 'Poe's Best Deck' and 1875 'Esther, the Brains' (objectives, rewards, Dominic as
//! the giver of 1875) and stay disabled until a card duel can produce a result.
//!
//! Content only; no production Rust changed. The shipped tree loads through the production
//! `QuestEngine::from_split_content` (the loader checks the giver lists against each quest
//! record and `quests_accepted` picks the turn-in NPC). The enabled cases run on a private
//! copy of the content with the quest switched on, through `Gameplay::handle_in_world`
//! with the packets the client sends: 26/8 interact (`0x008b6bc0`), 26/10 menu choice
//! (`0x00a36f80`), 26/6 quest row (`0x009f5890`), 49/1 offer card (reader `0x00c7bb60`;
//! reward bundle reader `0x008e7930`: +1 coins, +5 XP, see `src/mystery_chest.rs`) and
//! 49/2 accept (`0x00a86400`). Recorded values: `PacketSendSelfToClient.json`
//! `ClientQuestData.ClientQuests[10]` (1304) and `[18]` (1875).

#[path = "support/card_duelist.rs"]
mod card_duelist;

use card_duelist::{
    CARD_DUELIST, CHARACTER, DOMINIC, ESTHER, ESTHER_THE_BRAINS, POE, POES_BEST_DECK, SAM, accept,
    activate_card_duelist, assert_no_offer, at, card_duelist, coins, content_dir, counts,
    fixture_with, learn_card_duelist, objective_complete, offer, pending_turnin, profile_xp,
    quest_status, talk,
};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    npc_interaction::decode_quest_offer_49_1,
    quest_engine::{CharacterEligibility, QuestDenial, QuestEngine, QuestStatus},
};

fn eligibility(jobs: &[(&str, u8)]) -> CharacterEligibility {
    CharacterEligibility {
        player_level: 20,
        job_levels: jobs
            .iter()
            .map(|(job, level)| (job.to_string(), *level))
            .collect(),
        member: true,
        nonmember_quest_limit: 30,
        member_quest_limit: 30,
    }
}

fn engine(content: &std::path::Path, jobs: &[(&str, u8)]) -> QuestEngine {
    QuestEngine::from_split_content(content, CHARACTER, eligibility(jobs), None)
        .expect("the Card Duelist content loads")
}

/// (coins, stars) of a 49/1 offer's reward bundle (offsets in the module header).
fn offered_reward(game: &mut Gameplay, who: &CharacterState, giver: u64, quest: u32) -> (u32, u32) {
    let offered = offer(game, who, giver, quest, 5_000);
    let card = offered
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[49, 0, 1, 0]))
        .expect("the 49/1 offer card");
    let body = decode_quest_offer_49_1(card).unwrap().reward_bundle_body;
    (
        u32::from_le_bytes(body[1..5].try_into().unwrap()),
        u32::from_le_bytes(body[5..9].try_into().unwrap()),
    )
}

#[test]
fn shipped_card_duelist_bindings_load_and_stay_disabled() {
    let mut engine = engine(&content_dir(), &[("card_duelist", 20)]);
    assert_eq!(engine.giver_npc_id(POES_BEST_DECK), Some(POE));
    assert_eq!(engine.turnin_npc_id(POES_BEST_DECK), Some(POE));
    assert_eq!(engine.giver_npc_id(ESTHER_THE_BRAINS), Some(DOMINIC));
    assert_eq!(engine.turnin_npc_id(ESTHER_THE_BRAINS), Some(ESTHER));

    // Disabled: every NPC refuses both quests and nothing changes.
    let before = engine.state().clone();
    for quest in [POES_BEST_DECK, ESTHER_THE_BRAINS] {
        assert!(!engine.is_enabled(quest), "{quest} stays disabled");
        for npc in [POE, DOMINIC, ESTHER] {
            let refused = engine.offer(quest, npc);
            assert!(
                refused.denial.is_some() && !refused.changed,
                "{quest} at {npc}"
            );
            assert!(refused.outbound.is_empty());
            assert_eq!(engine.state(), &before);
        }
    }

    // Through the real menu a Card Duelist is offered nothing by the three NPCs.
    let fixture = fixture_with(&[]);
    let mut game = card_duelist(&fixture);
    let mut now = 2_000;
    for npc in [POE, DOMINIC, ESTHER] {
        assert_no_offer(
            &mut game,
            &at(npc),
            npc,
            &mut now,
            "disabled Card Duelist quests",
        );
    }
}

#[test]
fn esther_the_brains_starts_at_dominic_and_meeting_esther_advances_it() {
    let fixture = fixture_with(&[ESTHER_THE_BRAINS]);
    let mut game = card_duelist(&fixture);
    let mut now = 2_000;
    // The recorded giver is Dominic: Esther offers nothing.
    assert_no_offer(
        &mut game,
        &at(ESTHER),
        ESTHER,
        &mut now,
        "Esther before the offer",
    );

    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, CARD_DUELIST));
    offer(&mut game, &at(DOMINIC), DOMINIC, ESTHER_THE_BRAINS, now);
    accept(&mut game, &at(DOMINIC), ESTHER_THE_BRAINS, now + 10);
    assert_eq!(counts(&game, ESTHER_THE_BRAINS), [0, 0]);

    // Meeting Esther completes objective 1; the duel (minigame 668) is still open.
    let met = talk(&mut game, &at(ESTHER), ESTHER, now + 100);
    assert!(met.changed && objective_complete(&met, ESTHER_THE_BRAINS, 1));
    assert_eq!(counts(&game, ESTHER_THE_BRAINS), [1, 0]);
    assert_eq!(quest_status(&game, ESTHER_THE_BRAINS), QuestStatus::Active);
    assert_eq!(pending_turnin(&game), None);
    // Nothing is paid at the accept or the talk step.
    assert_eq!(coins(&game), coins_before);
    assert_eq!(profile_xp(&game, CARD_DUELIST), xp_before);
}

#[test]
fn poes_best_deck_is_offered_by_poe_with_its_xp() {
    let fixture = fixture_with(&[POES_BEST_DECK]);
    let mut game = card_duelist(&fixture);
    // Recorded 1304 bundle: 0 coins, 300 XP (the cap's item row is the T318/T412 gap).
    assert_eq!(
        offered_reward(&mut game, &at(POE), POE, POES_BEST_DECK),
        (0, 300)
    );
    assert_eq!(quest_status(&game, POES_BEST_DECK), QuestStatus::Offered);
}

#[test]
fn card_duelist_quests_need_the_job() {
    let fixture = fixture_with(&[POES_BEST_DECK, ESTHER_THE_BRAINS]);
    // Engine: without profile 120 both offers are refused with the job denial.
    let mut engine = engine(&fixture.quest_catalogue(), &[]);
    let before = engine.state().clone();
    for (quest, giver) in [(POES_BEST_DECK, POE), (ESTHER_THE_BRAINS, DOMINIC)] {
        let refused = engine.offer(quest, giver);
        assert_eq!(refused.denial, Some(QuestDenial::Job), "{quest}");
        assert!(!refused.changed && refused.outbound.is_empty());
        assert_eq!(engine.state(), &before);
    }
    // Real menu: a player who never learned the job gets no offer and no change.
    let mut game = fixture.game_as(CHARACTER, None);
    let mut now = 1_000;
    for npc in [POE, DOMINIC] {
        assert_no_offer(&mut game, &at(npc), npc, &mut now, "no Card Duelist job");
    }
    // Learned but not the active job: still refused. Activating is the only change needed.
    learn_card_duelist(&mut game, &at(SAM), 2_000);
    for npc in [POE, DOMINIC] {
        assert_no_offer(
            &mut game,
            &at(npc),
            npc,
            &mut now,
            "Card Duelist not active",
        );
    }
    let mut game = activate_card_duelist(&fixture, &game, CHARACTER);
    assert_eq!(
        offered_reward(&mut game, &at(POE), POE, POES_BEST_DECK),
        (0, 300)
    );
}
