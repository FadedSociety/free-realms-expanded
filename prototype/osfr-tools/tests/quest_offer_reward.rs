//! T318: the 49/1 quest offer card carries the offered quest's authored reward.
//!
//! The offer card and the turn-in screen read the same `ClientRewardBundle` body:
//! - 49/1 `QuestInfoPacket` reader `00c7bb60` -> `ReadFields` `00c7b7a0` reads six
//!   dwords (quest id + five screen fields), a bool, then calls the shared bundle
//!   reader `sub_8E7930`: the body starts at packet offset 31 (6-byte header, quest id,
//!   five fields, bool), followed by an 8-byte actor, a float and two bools (the final
//!   14 bytes).
//! - 49/13 `QuestEndPacket::ReadFields` `00c7b990` reads the 8-byte actor, quest id,
//!   end text and animation dword, then calls the same `sub_8E7930` at packet offset
//!   26, then a trailing float.
//!
//! IDA xrefs to `0x008e7930` list both callers: `sub_C7B7A0+0x10c` and
//! `sub_C7B990+0xab`. The reader starts with the success byte (bundle+0) and the
//! coin dword (bundle+1..5); the next dword is the summed job-XP stars
//! (bundle+5..9); after 6 i32, a float, two i32 and two u64 the counted display rows
//! start at bundle+65 (row count at bundle+61..65). The 69-byte empty base is
//! 1 + 6*4 + 4 + 2*4 + 2*8 + 3*4 + 4, and a populated reward appends counted rows.
//!
//! These tests drive the production in-world router (`Gameplay::handle_in_world` ->
//! `gameplay/npc.rs` -> `QuestEngine`) with the same 26/8, 26/10, 26/6, 49/2 and
//! 49/14 packets the gateway accepts, so the bytes are the bytes a client receives.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    Fixture, accept, assert_packet_changes_nothing, character_at_for, coins, content_dir,
    end_screen_reward, finish_quest, offer, pending_turnin, profile_xp, quest_end_reply,
    release_end_screen, talk_step,
};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    npc_interaction::{NpcInteraction, decode_quest_offer_49_1},
    quest_engine::{CharacterEligibility, QuestEngine, QuestProgress, QuestStatus},
};

const CHARACTER: u64 = 318_001;
/// Adventurer, the profile the paying test quests use.
const ADVENTURER: i32 = 1;
/// Cobblestone Village giver 9000000189; gives quest 3000021 ("Introduce Yourself").
const COBBLESTONE_GIVER: u64 = 9_000_000_189;
/// Ricky Danger 9_500_000_254, the one-talk objective target of quest 3000021.
const COBBLESTONE_TARGET: u64 = 9_500_000_254;
/// Grumpore 9000000337; gives the no-reward quest 3100020.
const MINER_GIVER: u64 = 9_000_000_337;
/// 3100020's prerequisite. Later content disabled it, so the test restores the
/// completed-quest save state that `repair_counters` preserves for exactly this case.
const DISABLED_PREREQUISITE: u32 = 3_100_019;
/// One Talk, 5 coins and 20 Adventurer XP, no items and no display rows.
const REWARD_QUEST: u32 = 3_000_021;
/// Enabled record with empty `rewards`, so its bundle is the 69-byte base.
const EMPTY_REWARD_QUEST: u32 = 3_100_020;
/// Greenwood Forest giver 9000000444; quest 3000024 pays 13 coins and 25 Adventurer XP.
const SECOND_GIVER: u64 = 9_000_000_444;
const SECOND_REWARD_QUEST: u32 = 3_000_024;

fn character_at(guid: u64, npc_id: u64) -> CharacterState {
    let content = content_dir();
    let (zone_id, position) =
        NpcInteraction::load(&content.join("npcs"), &content.join("stores.json"))
            .expect("NPC content loads")
            .location(npc_id)
            .unwrap_or_else(|| panic!("NPC {npc_id} is a positioned content spawn"));
    character_at_for(guid, zone_id, position)
}

/// The production split catalogue for one character; the display GUIDs are
/// character-owned, so the accessor must be built with the same character id.
fn engine(character_id: u64) -> QuestEngine {
    QuestEngine::from_split_content(
        &content_dir(),
        character_id,
        CharacterEligibility::default(),
        None,
    )
    .expect("split quest content loads")
}

/// A session whose save already completed 3100020's prerequisite before content
/// disabled it: `repair_counters` preserves that historical progress, and the real
/// Talk path then offers the enabled no-reward quest. No other enabled quest has an
/// empty reward and an offerable giver on this content.
fn game_with_completed_prerequisite(fixture: &Fixture) -> Gameplay {
    let mut state = fixture.game(None).state();
    state
        .npc_quests
        .as_mut()
        .expect("the fixture binds the quest catalogue")
        .quests
        .insert(
            DISABLED_PREREQUISITE,
            QuestProgress {
                status: QuestStatus::Completed,
                counts: Vec::new(),
            },
        );
    fixture.game(Some(state))
}

fn offer_packet(offered: &osfr_tools::gameplay::Outcome) -> Vec<u8> {
    let packets: Vec<_> = offered
        .replies
        .iter()
        .filter(|packet| packet.starts_with(&[49, 0, 1, 0]))
        .cloned()
        .collect();
    assert_eq!(packets.len(), 1, "one Talk offers exactly one quest");
    packets.into_iter().next().unwrap()
}

/// The 49/1 fields after the bundle: the runtime actor the client clicked (8 bytes),
/// one unresolved dword (the float) and two (R) bools.
fn assert_trailing_fields(packet: &[u8], actor: u64) {
    assert!(packet.len() >= 14);
    let tail = &packet[packet.len() - 14..];
    assert_eq!(
        u64::from_le_bytes(tail[..8].try_into().unwrap()),
        actor,
        "the offer's trailing actor is the clicked NPC"
    );
    assert_eq!(&tail[8..], &[0; 6], "float bits and both bools are zero");
}

fn assert_no_grant(
    game: &Gameplay,
    inventory_before: &osfr_tools::inventory::InventoryState,
    coins_before: u32,
    xp_before: u32,
    label: &str,
) {
    assert_eq!(coins(game), coins_before, "{label}: coins changed");
    assert_eq!(
        &game.state().inventory,
        inventory_before,
        "{label}: inventory changed"
    );
    assert_eq!(
        profile_xp(game, ADVENTURER),
        xp_before,
        "{label}: job XP changed"
    );
}

#[test]
fn the_offer_card_carries_the_authored_reward() {
    let fixture = Fixture::for_character(CHARACTER);
    let mut game = fixture.game(None);
    let character = character_at(CHARACTER, COBBLESTONE_GIVER);
    let offered = offer(
        &mut game,
        &character,
        COBBLESTONE_GIVER,
        REWARD_QUEST,
        1_000,
    );
    let packet = offer_packet(&offered);
    let decoded = decode_quest_offer_49_1(&packet).expect("the offered 49/1 decodes");
    assert_eq!(decoded.quest_id, REWARD_QUEST);

    let bundle = engine(CHARACTER)
        .quest_reward_bundle(REWARD_QUEST)
        .expect("the authored reward encodes");
    assert_eq!(
        decoded.reward_bundle_body, bundle,
        "the card carries the same bundle the 49/13 turn-in builds"
    );
    // No enabled content record authors display rows, so the real cardinality is
    // the 69-byte base; the coin and star words are no longer zeros.
    assert_eq!(decoded.reward_bundle_body.len(), 69);
    assert_eq!(
        i32::from_le_bytes(bundle[1..5].try_into().unwrap()),
        5,
        "the card's coin word is the authored reward"
    );
    assert_eq!(
        i32::from_le_bytes(bundle[5..9].try_into().unwrap()),
        20,
        "the card's star word is the summed job XP"
    );
    assert_eq!(
        i32::from_le_bytes(bundle[61..65].try_into().unwrap()),
        0,
        "the record authors no display rows"
    );
    assert_ne!(
        bundle,
        vec![0; 69],
        "the offer no longer sends the all-zero placeholder"
    );
    assert_trailing_fields(&packet, COBBLESTONE_GIVER);
}

#[test]
fn an_empty_reward_quest_still_sends_the_69_byte_base() {
    let fixture = Fixture::for_character(CHARACTER);
    let mut game = game_with_completed_prerequisite(&fixture);
    let character = character_at(CHARACTER, MINER_GIVER);
    let offered = offer(
        &mut game,
        &character,
        MINER_GIVER,
        EMPTY_REWARD_QUEST,
        2_000,
    );
    let packet = offer_packet(&offered);
    let decoded = decode_quest_offer_49_1(&packet).expect("the offered 49/1 decodes");
    assert_eq!(decoded.quest_id, EMPTY_REWARD_QUEST);

    let bundle = engine(CHARACTER)
        .quest_reward_bundle(EMPTY_REWARD_QUEST)
        .expect("an empty reward still encodes");
    assert_eq!(decoded.reward_bundle_body, bundle);
    assert_eq!(decoded.reward_bundle_body.len(), 69);
    assert_eq!(
        i32::from_le_bytes(bundle[1..5].try_into().unwrap()),
        0,
        "no coins"
    );
    assert_eq!(
        i32::from_le_bytes(bundle[5..9].try_into().unwrap()),
        0,
        "no stars"
    );
    assert_eq!(
        u64::from_le_bytes(bundle[37..45].try_into().unwrap()),
        MINER_GIVER,
        "the display source GUID is the quest's giver"
    );
    assert_eq!(
        u64::from_le_bytes(bundle[45..53].try_into().unwrap()),
        CHARACTER,
        "the display player GUID is this character"
    );
    assert_trailing_fields(&packet, MINER_GIVER);
}

#[test]
fn the_offer_grants_nothing_and_turn_in_pays_once() {
    let fixture = Fixture::for_character(CHARACTER);
    let mut game = fixture.game(None);
    let giver = character_at(CHARACTER, COBBLESTONE_GIVER);
    let target = character_at(CHARACTER, COBBLESTONE_TARGET);
    let mut now = 3_000;
    let inventory_before = game.state().inventory.clone();
    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, ADVENTURER);

    offer(&mut game, &giver, COBBLESTONE_GIVER, REWARD_QUEST, now);
    assert_no_grant(
        &game,
        &inventory_before,
        coins_before,
        xp_before,
        "the 49/1 offer",
    );
    now += 10;
    accept(&mut game, &giver, REWARD_QUEST, now);
    assert_no_grant(
        &game,
        &inventory_before,
        coins_before,
        xp_before,
        "the 49/2 accept",
    );

    now += 10;
    talk_step(
        &mut game,
        &target,
        COBBLESTONE_TARGET,
        now,
        "Introduce Yourself",
    );
    assert_eq!(pending_turnin(&game), Some(REWARD_QUEST));
    now += 10;
    let end_text = engine(CHARACTER)
        .end_text_string_id(REWARD_QUEST)
        .unwrap_or(0);
    let end = release_end_screen(
        &mut game,
        &target,
        REWARD_QUEST,
        COBBLESTONE_TARGET,
        end_text,
        now,
    );
    assert_no_grant(
        &game,
        &inventory_before,
        coins_before,
        xp_before,
        "the 49/13 display",
    );
    let (displayed_coins, displayed_xp) = end_screen_reward(&end);
    assert_eq!((displayed_coins, displayed_xp), (5, 20));

    now += 10;
    finish_quest(&mut game, &target, REWARD_QUEST, ADVENTURER, 5, 20, now);
    assert_eq!(
        coins(&game) - coins_before,
        displayed_coins,
        "the turn-in pays exactly the displayed coins"
    );
    assert_eq!(
        profile_xp(&game, ADVENTURER) - xp_before,
        displayed_xp,
        "the turn-in pays exactly the displayed XP"
    );

    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &target,
        &quest_end_reply(REWARD_QUEST),
        now,
        "replayed 49/14",
    );
}

/// A display row the client entry reader cannot instantiate is refused by the
/// shared encoder on both screens. The offer builder's own refusal is pinned by
/// `gameplay::npc::tests::a_malformed_reward_row_is_refused_by_the_offer_builder`:
/// content cannot author display rows, so no player path can reach it.
#[test]
fn a_malformed_reward_is_refused_without_mutation() {
    use osfr_tools::quest_engine::{QuestDefinition, QuestRequirements};
    use osfr_tools::rewards::{RewardBundleRow, RewardDisplay, RewardGrant};

    let definition = QuestDefinition {
        id: 7,
        title: "Quest 7".into(),
        description: String::new(),
        giver_npc_id: 11,
        turnin_npc_id: 11,
        prerequisite_quest_ids: Vec::new(),
        requirements: QuestRequirements::default(),
        ordered: false,
        objectives: Vec::new(),
        reward: RewardGrant {
            coins: 5,
            items: Vec::new(),
            job_xp: std::collections::BTreeMap::new(),
            sc: 0,
            display: RewardDisplay {
                source_guid: 11,
                player_guid: 3,
                reward_kind: 0,
                icon_id: 0,
                name_id: 0,
                rows: vec![RewardBundleRow {
                    type_id: 2,
                    body: vec![0; 34],
                }],
            },
        },
    };
    let engine = QuestEngine::new(3, vec![definition], CharacterEligibility::default(), None)
        .expect("display rows are not validated at engine construction");
    let state_before = serde_json::to_vec(engine.state()).unwrap();
    let bundle = engine.quest_reward_bundle(7).unwrap_err();
    assert_eq!(bundle.kind(), std::io::ErrorKind::InvalidData);
    let end = engine.quest_end_packet(7, 1).unwrap_err();
    assert_eq!(
        end.kind(),
        std::io::ErrorKind::InvalidData,
        "the offer and turn-in refuse the same row"
    );
    assert!(engine.quest_reward_bundle(8).is_err(), "unknown quest");
    assert_eq!(
        serde_json::to_vec(engine.state()).unwrap(),
        state_before,
        "a refused bundle changed the engine state"
    );
}

/// A Talk that can offer nothing must not touch the character and must leave the
/// open 26/9 menu usable; the shared helper asserts both and the byte-equal save.
#[test]
fn a_refused_offer_changes_nothing() {
    let fixture = Fixture::for_character(CHARACTER);
    let mut game = fixture.game(None);
    let character = character_at(CHARACTER, COBBLESTONE_GIVER);
    let mut now = 4_000;
    offer(&mut game, &character, COBBLESTONE_GIVER, REWARD_QUEST, now);

    // The same quest is already offered, so the giver may not offer it again.
    now += 10;
    npc_quest_chain::assert_no_offer(
        &mut game,
        &character,
        COBBLESTONE_GIVER,
        &mut now,
        "an already-offered quest at its giver",
    );
}

#[test]
fn two_players_see_their_own_offer_reward() {
    let first_fixture = Fixture::for_character(318_101);
    let second_fixture = Fixture::for_character(318_102);
    let mut first = first_fixture.game(None);
    let mut second = second_fixture.game(None);
    let first_character = character_at(318_101, COBBLESTONE_GIVER);
    let second_character = character_at(318_102, SECOND_GIVER);

    let first_offer = offer(
        &mut first,
        &first_character,
        COBBLESTONE_GIVER,
        REWARD_QUEST,
        1_000,
    );
    let second_offer = offer(
        &mut second,
        &second_character,
        SECOND_GIVER,
        SECOND_REWARD_QUEST,
        1_000,
    );
    let first_body = decode_quest_offer_49_1(&offer_packet(&first_offer))
        .expect("A's offer decodes")
        .reward_bundle_body;
    let second_body = decode_quest_offer_49_1(&offer_packet(&second_offer))
        .expect("B's offer decodes")
        .reward_bundle_body;

    assert_eq!(
        first_body,
        engine(318_101).quest_reward_bundle(REWARD_QUEST).unwrap()
    );
    assert_eq!(
        second_body,
        engine(318_102)
            .quest_reward_bundle(SECOND_REWARD_QUEST)
            .unwrap()
    );
    assert_eq!(
        i32::from_le_bytes(first_body[1..5].try_into().unwrap()),
        5,
        "A's card carries A's quest reward"
    );
    assert_eq!(
        i32::from_le_bytes(second_body[1..5].try_into().unwrap()),
        13,
        "B's card carries B's quest reward"
    );
    assert_eq!(
        i32::from_le_bytes(second_body[5..9].try_into().unwrap()),
        25,
        "B's star total is B's job XP"
    );
    assert_eq!(
        u64::from_le_bytes(first_body[37..45].try_into().unwrap()),
        COBBLESTONE_GIVER,
        "A's bundle names A's giver"
    );
    assert_eq!(
        u64::from_le_bytes(second_body[37..45].try_into().unwrap()),
        SECOND_GIVER,
        "B's bundle names B's giver"
    );
    assert_eq!(
        u64::from_le_bytes(first_body[45..53].try_into().unwrap()),
        318_101,
        "A's bundle names A"
    );
    assert_eq!(
        u64::from_le_bytes(second_body[45..53].try_into().unwrap()),
        318_102,
        "B's bundle names B"
    );
    assert_ne!(first_body, second_body, "no cached or shared bundle");
}
