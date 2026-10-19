use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState, Outcome},
    npc_interaction::NpcInteraction,
    quest_engine::{QuestProgress, QuestState, QuestStatus},
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

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
const TALK_MENU_STRING_ID: u32 = 3300;
const QUEST_CHOICE_RESPONSE: u32 = 301;
const NOW: u64 = 10_000;

fn content_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn gameplay_content() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-opening-cooking-original-assets")
}

fn completed_quest() -> QuestProgress {
    QuestProgress {
        status: QuestStatus::Completed,
        counts: vec![1],
    }
}

fn active_stealth(counts: Vec<u32>) -> QuestProgress {
    QuestProgress {
        status: QuestStatus::Active,
        counts,
    }
}

fn completed_through_stolen_away(character_id: u64, stealth: Option<QuestProgress>) -> QuestState {
    let mut quests = BTreeMap::from([
        (NINJA_PATH, completed_quest()),
        (STOLEN_AWAY, completed_quest()),
    ]);
    if let Some(progress) = stealth {
        quests.insert(STEALTH_REFERRAL, progress);
    }
    QuestState {
        version: 1,
        character_id,
        quests,
        pending_offer: None,
        pending_turnin: None,
        pending_turnin_actor: None,
        selected_quest_id: None,
        auto_switch_selected: false,
    }
}

fn spawn_location(npc_id: u64) -> (u32, [f32; 3]) {
    let content = content_root();
    NpcInteraction::load(&content.join("npcs"), &content.join("stores.json"))
        .unwrap()
        .location(npc_id)
        .expect("spawned NPC")
}

struct Session {
    game: Gameplay,
    character: CharacterState,
}

impl Session {
    fn new(npc_id: u64, stealth: Option<QuestProgress>) -> Self {
        let mut character = CharacterState {
            guid: 700_107,
            zone_id: 0,
            position: [0.0; 3],
            rotation: [1.0, 0.0, 0.0],
        };
        let (zone_id, position) = spawn_location(npc_id);
        character.zone_id = zone_id;
        character.position = position;

        let mut state = Gameplay::load(&gameplay_content(), None).unwrap().state();
        state.npc_quests = Some(completed_through_stolen_away(character.guid, stealth));
        let mut game = Gameplay::load(&gameplay_content(), Some(state)).unwrap();
        game.bind_npc_quest_catalogue(&content_root(), character.guid)
            .unwrap();
        Self { game, character }
    }

    fn reconnect_at(self, npc_id: u64) -> Self {
        let encoded = serde_json::to_vec(&self.game.state()).unwrap();
        let state: GameplayState = serde_json::from_slice(&encoded).unwrap();
        let mut game = Gameplay::load(&gameplay_content(), Some(state)).unwrap();
        game.bind_npc_quest_catalogue(&content_root(), self.character.guid)
            .unwrap();
        let mut character = self.character;
        let (zone_id, position) = spawn_location(npc_id);
        character.zone_id = zone_id;
        character.position = position;
        Self { game, character }
    }

    fn send(&mut self, inner: &[u8]) -> Outcome {
        self.game
            .handle_in_world(inner, &self.character, NOW)
            .unwrap()
    }

    fn talk_option(&mut self, npc_id: u64) -> i32 {
        let replies = self.send(&npc_interact(npc_id)).replies;
        let menu = replies
            .iter()
            .find(|body| body.starts_with(&[26, 0, 9, 0]))
            .expect("26/8 returns the 26/9 interaction menu");
        let count = i32::from_le_bytes(menu[13..17].try_into().unwrap());
        let mut offset = 17;
        for _ in 0..count {
            let option_id = i32::from_le_bytes(menu[offset..offset + 4].try_into().unwrap());
            let label = u32::from_le_bytes(menu[offset + 8..offset + 12].try_into().unwrap());
            if label == TALK_MENU_STRING_ID {
                return option_id;
            }
            offset += 28;
        }
        panic!("NPC menu has no Talk option");
    }

    fn talk(&mut self, npc_id: u64) -> Outcome {
        let option = self.talk_option(npc_id);
        let outcome = self.send(&npc_select(npc_id, option));
        assert!(
            outcome
                .replies
                .iter()
                .any(|body| body.starts_with(&[26, 0, 3, 0])),
            "Talk must open the dialogue packet"
        );
        outcome
    }

    fn saved_quests(&self) -> QuestState {
        self.game.state().npc_quests.expect("saved regional quests")
    }

    fn stealth_counts(&self) -> Vec<u32> {
        self.saved_quests().quests[&STEALTH_REFERRAL].counts.clone()
    }
}

fn npc_interact(npc_id: u64) -> Vec<u8> {
    // Client 26/8 constructor 0x008b6bc0.
    let mut bytes = vec![26, 0, 8, 0];
    bytes.extend(npc_id.to_le_bytes());
    bytes.push(0);
    bytes
}

fn npc_select(npc_id: u64, option: i32) -> Vec<u8> {
    // Client 26/10 constructor 0x00a36f80: actor id, then menu entry.
    let mut bytes = vec![26, 0, 10, 0];
    bytes.extend(npc_id.to_le_bytes());
    bytes.extend(option.to_le_bytes());
    bytes
}

fn quest_choice() -> Vec<u8> {
    // Client 26/6 reader 0x009f5890 stores one dialogue-row i32; the row value is (R).
    let mut bytes = vec![26, 0, 6, 0];
    bytes.extend(QUEST_CHOICE_RESPONSE.to_le_bytes());
    bytes
}

fn quest_reply(quest_id: u32) -> Vec<u8> {
    // Client QuestReply constructor 0x00a86400, body writer 0x00a91630, sender 0x00a97d70.
    let mut bytes = vec![49, 0, 2, 0, 0, 0];
    bytes.extend(quest_id.to_le_bytes());
    bytes.push(1);
    bytes
}

fn quest_end_reply(quest_id: u32) -> Vec<u8> {
    // Client QuestEndReply constructor 0x00a864c0 and sender 0x00a97ef0.
    let mut bytes = vec![49, 0, 14, 0, 0, 0];
    bytes.extend(quest_id.to_le_bytes());
    bytes
}

fn quest_id(body: &[u8]) -> u32 {
    u32::from_le_bytes(body[6..10].try_into().unwrap())
}

fn halted_mongo_chain_is_disabled() -> bool {
    let document: serde_json::Value =
        serde_json::from_slice(&std::fs::read(content_root().join("quests/ninja.json")).unwrap())
            .unwrap();
    document["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["id"] == HALTED_MONGO_CHAIN)
        .is_some_and(|record| record["enabled"] == false)
}

#[test]
fn ty_offers_the_stealth_referral_after_stolen_away() {
    let mut session = Session::new(TY, None);
    let talk_option = session.talk_option(TY);

    // A forged menu entry and a forged accept cannot consume the open menu.
    let before = session.game.state();
    assert!(
        session
            .game
            .handle_in_world(&npc_select(TY, i32::MAX), &session.character, NOW)
            .is_err()
    );
    assert_eq!(session.game.state(), before);

    let talked = session.send(&npc_select(TY, talk_option));
    assert!(
        talked
            .replies
            .iter()
            .any(|body| body.starts_with(&[26, 0, 3, 0]))
    );
    assert!(session.saved_quests().pending_offer.is_none());

    let before = session.game.state();
    let refused = session.send(&quest_reply(HALTED_MONGO_CHAIN));
    assert!(!refused.changed);
    assert_eq!(session.game.state(), before);

    let offered = session.send(&quest_choice());
    let offer = offered
        .replies
        .iter()
        .find(|body| body.starts_with(&[49, 0, 1, 0]))
        .expect("dialogue choice returns the 49/1 offer");
    assert_eq!(quest_id(offer), STEALTH_REFERRAL);
    assert_eq!(session.saved_quests().pending_offer, Some(STEALTH_REFERRAL));

    let accepted = session.send(&quest_reply(STEALTH_REFERRAL));
    assert!(accepted.changed);
    assert!(
        accepted
            .replies
            .iter()
            .any(|body| body.starts_with(&[49, 0, 3, 0]))
    );
    let saved = session.saved_quests();
    assert_eq!(saved.pending_offer, None);
    assert_eq!(saved.quests[&STEALTH_REFERRAL].status, QuestStatus::Active);
    assert_eq!(saved.quests[&STEALTH_REFERRAL].counts, [0, 0, 0, 0]);

    let before = session.game.state();
    let replay = session.send(&quest_reply(STEALTH_REFERRAL));
    assert!(!replay.changed);
    assert_eq!(session.game.state(), before);
}

#[test]
fn three_mentors_are_required_in_order() {
    let mut session = Session::new(KOSHA, Some(active_stealth(vec![0, 0, 0, 0])));
    for npc_id in [KOSHA, CHLOE, TY] {
        session = session.reconnect_at(npc_id);
        let before = session.game.state();
        let out_of_order = session.talk(npc_id);
        assert!(!out_of_order.changed, "NPC {npc_id} cannot skip the order");
        assert_eq!(session.game.state(), before);
        assert_eq!(session.stealth_counts(), [0, 0, 0, 0]);
    }

    session = session.reconnect_at(BRENDAN);
    assert!(session.talk(BRENDAN).changed);
    assert_eq!(session.stealth_counts(), [1, 0, 0, 0]);

    let before = session.game.state();
    let replay = session.talk(BRENDAN);
    assert!(!replay.changed);
    assert_eq!(session.game.state(), before);

    session = session.reconnect_at(KOSHA);
    assert!(session.talk(KOSHA).changed);
    assert_eq!(session.stealth_counts(), [1, 1, 0, 0]);

    let before = session.game.state();
    let replay = session.talk(KOSHA);
    assert!(!replay.changed);
    assert_eq!(session.game.state(), before);

    session = session.reconnect_at(CHLOE);
    assert!(session.talk(CHLOE).changed);
    assert_eq!(session.stealth_counts(), [1, 1, 1, 0]);
    assert_eq!(session.saved_quests().pending_turnin, None);

    session = session.reconnect_at(UNRELATED);
    let before = session.game.state();
    let unrelated = session.talk(UNRELATED);
    assert!(!unrelated.changed);
    assert_eq!(session.game.state(), before);
    assert_eq!(session.stealth_counts(), [1, 1, 1, 0]);
}

#[test]
fn return_to_ty_settles_once() {
    let mut session = Session::new(TY, Some(active_stealth(vec![1, 1, 1, 0])));
    let returned = session.talk(TY);
    assert!(returned.changed);
    assert_eq!(session.stealth_counts(), [1, 1, 1, 1]);
    assert_eq!(
        session.saved_quests().pending_turnin,
        Some(STEALTH_REFERRAL)
    );
    assert!(
        !returned
            .replies
            .iter()
            .any(|body| body.starts_with(&[49, 0, 13, 0])),
        "the end screen opens only after the client chooses Talk"
    );

    let end = session.send(&quest_choice());
    let end = end
        .replies
        .iter()
        .find(|body| body.starts_with(&[49, 0, 13, 0]))
        .expect("Talk choice opens the 49/13 end screen");
    assert_eq!(
        u64::from_le_bytes(end[6..14].try_into().unwrap()),
        TY,
        "the 49/13 names the NPC that opened the screen, not the quest id"
    );
    assert_eq!(
        u32::from_le_bytes(end[14..18].try_into().unwrap()),
        STEALTH_REFERRAL
    );

    let mut session = session.reconnect_at(TY);
    assert_eq!(
        session.saved_quests().pending_turnin,
        Some(STEALTH_REFERRAL)
    );
    let coins_before = session.game.state().inventory.coins;
    let before = session.game.state();
    let forged = session.send(&quest_end_reply(HALTED_MONGO_CHAIN));
    assert!(!forged.changed);
    assert_eq!(session.game.state(), before);

    let completed = session.send(&quest_end_reply(STEALTH_REFERRAL));
    assert!(completed.changed);
    assert!(
        completed
            .replies
            .iter()
            .any(|body| body.starts_with(&[49, 0, 4, 0]))
    );
    let after = session.game.state();
    assert_eq!(after.inventory.coins, coins_before + STEALTH_REWARD_COINS);
    assert_eq!(
        after.npc_quests.as_ref().unwrap().quests[&STEALTH_REFERRAL].status,
        QuestStatus::Completed
    );
    assert_eq!(after.npc_quests.as_ref().unwrap().pending_turnin, None);

    let before = session.game.state();
    let replay = session.send(&quest_end_reply(STEALTH_REFERRAL));
    assert!(!replay.changed);
    assert_eq!(session.game.state(), before);

    let session = session.reconnect_at(TY);
    let saved = session.game.state();
    assert_eq!(saved.inventory.coins, coins_before + STEALTH_REWARD_COINS);
    assert_eq!(
        saved.npc_quests.unwrap().quests[&STEALTH_REFERRAL].status,
        QuestStatus::Completed
    );
}

#[test]
fn halted_mongo_chain_stays_hidden_and_save_reloads() {
    assert!(halted_mongo_chain_is_disabled());

    let mut session = Session::new(TY, None);
    session.talk(TY);
    let before = session.game.state();
    let refused = session.send(&quest_reply(HALTED_MONGO_CHAIN));
    assert!(!refused.changed);
    assert_eq!(session.game.state(), before);

    let offered = session.send(&quest_choice());
    let offer = offered
        .replies
        .iter()
        .find(|body| body.starts_with(&[49, 0, 1, 0]))
        .expect("Ty offers the enabled referral, not the disabled chapter");
    assert_eq!(quest_id(offer), STEALTH_REFERRAL);

    assert!(session.send(&quest_reply(STEALTH_REFERRAL)).changed);
    for npc_id in [BRENDAN, KOSHA, CHLOE] {
        session = session.reconnect_at(npc_id);
        assert!(session.talk(npc_id).changed);
    }
    session = session.reconnect_at(TY);
    assert!(session.talk(TY).changed);
    let end = session.send(&quest_choice());
    assert!(
        end.replies
            .iter()
            .any(|body| body.starts_with(&[49, 0, 13, 0]))
    );

    session = session.reconnect_at(TY);
    let coins_before = session.game.state().inventory.coins;
    let completed = session.send(&quest_end_reply(STEALTH_REFERRAL));
    assert!(completed.changed);
    assert_eq!(
        session.game.state().inventory.coins,
        coins_before + STEALTH_REWARD_COINS
    );

    let reloaded = session.reconnect_at(TY);
    let saved = reloaded.game.state();
    let quests = saved.npc_quests.as_ref().unwrap();
    assert_eq!(
        quests.quests[&STEALTH_REFERRAL].status,
        QuestStatus::Completed
    );
    assert_eq!(quests.quests[&STEALTH_REFERRAL].counts, [1, 1, 1, 1]);
    assert!(!quests.quests.contains_key(&HALTED_MONGO_CHAIN));
    assert_eq!(quests.pending_turnin, None);
    assert_eq!(saved.inventory.coins, coins_before + STEALTH_REWARD_COINS);
    assert!(halted_mongo_chain_is_disabled());
}
