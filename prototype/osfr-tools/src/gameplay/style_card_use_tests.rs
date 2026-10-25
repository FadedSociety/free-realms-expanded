//! T478: 42/10 and 42/12 through `Gameplay::handle_in_world`, the router the gateway calls, on
//! both content shapes: the open world (`Gameplay::handle`) and the opening campaign
//! (`handle_opening_inner`). The real item catalogue supplies the cards and the client's own
//! mapping tables the strings.

use super::*;
use crate::{
    account_store::AccountRole,
    gateway::Gateway,
    style_cards::{CustomizationRecord, customization_change},
};
#[path = "../../tests/common/style_card_fixture.rs"]
mod fixture;
#[path = "../../tests/common/round70_content.rs"]
mod round70_content;
use fixture::*;
use std::{
    fs,
    sync::atomic::{AtomicU64, Ordering},
};

const NOW: u64 = 5_000;
const GUID: u64 = 1;
/// Instance ids of the cards a test puts in the bag.
const CARD: i32 = 900_001;
const OTHER: i32 = 900_002;
static NEXT: AtomicU64 = AtomicU64::new(0);

/// One content directory; the open world and the opening campaign share the item catalogue.
struct Content(PathBuf);

impl Content {
    fn new(opening: bool) -> Self {
        let root = std::env::temp_dir().join(format!(
            "osfr-t478-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        if !opening {
            return Self(round70_content::build(&root, "open", &[]));
        }
        let lab = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
        let campaign = lab.join("references/opening-journal-text-v2/content");
        for entry in fs::read_dir(&campaign).unwrap() {
            let file = entry.unwrap().path();
            if file.is_file() {
                fs::copy(&file, root.join(file.file_name().unwrap())).unwrap();
            }
        }
        let catalogue = round70_content::source().join("item-catalogue.json");
        if fs::hard_link(&catalogue, root.join("item-catalogue.json")).is_err() {
            fs::copy(&catalogue, root.join("item-catalogue.json")).unwrap();
        }
        Self(root)
    }

    /// A gameplay with `cards` (instance id, definition, count) in the bag, the client tables
    /// unless `tables` is off, and `identity` applied as a login does.
    fn game(
        &self,
        cards: &[(i32, i32, u32)],
        identity: &CharacterIdentity,
        tables: bool,
    ) -> Gameplay {
        let mut game = self.unidentified(cards, tables);
        game.apply_identity(identity).unwrap();
        game
    }

    /// The same before any identity is applied: a session that has not finished logging in.
    fn unidentified(&self, cards: &[(i32, i32, u32)], tables: bool) -> Gameplay {
        let mut state = Gameplay::load(&self.0, None).unwrap().state();
        for &(id, definition, count) in cards {
            state.inventory.items.push(card_item(id, definition, count));
        }
        let next = state
            .inventory
            .items
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap()
            + 1;
        state.inventory.next_instance_id = Some(i64::from(next));
        let mut game = Gameplay::load(&self.0, Some(state)).unwrap();
        game.set_style_tables(tables.then(fixture::tables));
        game
    }
}

impl Drop for Content {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn each_content(test: impl Fn(&Content)) {
    for opening in [false, true] {
        test(&Content::new(opening));
    }
}

fn character() -> CharacterState {
    let reference = round70_content::reference();
    let mut character = Gateway::reference_state(&reference).unwrap();
    character.guid = GUID;
    character
}

fn state_bytes(game: &Gameplay) -> Vec<u8> {
    serde_json::to_vec(&game.state()).unwrap()
}

fn use_card(game: &mut Gameplay, opcode: u8, value: i32) -> Outcome {
    game.handle_in_world(&request(opcode, value), &character(), NOW)
        .unwrap()
}

fn chat_lines(out: &Outcome) -> Vec<String> {
    let mut lines = Vec::new();
    for reply in out.replies.iter().filter(|r| r.starts_with(&[15, 0, 3, 0])) {
        let length = u32::from_le_bytes(reply[4..8].try_into().unwrap()) as usize;
        lines.push(String::from_utf8(reply[8..8 + length].to_vec()).unwrap());
    }
    lines
}

fn count(game: &Gameplay, id: i32) -> Option<u32> {
    let items = &game.state().inventory.items;
    items
        .iter()
        .find(|item| item.id == id)
        .map(|item| item.count)
}

/// The 35/39 a card makes: one record, preview 0 (reader `0091aff0`).
fn change(record: u8, text: &str, param: i32, item: i32) -> Vec<u8> {
    let record = CustomizationRecord {
        id: i32::from(record),
        string_param: text.to_owned(),
        param,
        item_id: item,
    };
    customization_change(GUID, false, &[record])
}

fn delete(id: i32) -> Vec<u8> {
    let mut packet = vec![38, 0, 4, 0];
    packet.extend(id.to_le_bytes());
    packet
}

#[test]
fn clean_shaven_removes_the_beard() {
    each_content(|content| {
        for opcode in [10, 12] {
            let mut game = content.game(&[(CARD, CLEAN_SHAVEN, 1)], &boy(), true);
            let by = if opcode == 10 { CARD } else { CLEAN_SHAVEN };
            let out = use_card(&mut game, opcode, by);
            let packet = change(5, "", 0, CARD);
            assert!(out.changed, "42/{opcode}");
            assert_eq!(out.replies.first(), Some(&delete(CARD)), "42/{opcode}");
            assert_eq!(
                out.replies.iter().filter(|r| **r == packet).count(),
                1,
                "42/{opcode}: one 35/39 to the player"
            );
            assert_eq!(
                out.nearby,
                [packet],
                "42/{opcode}: the same 35/39 to the others"
            );
            let look = game.look().unwrap();
            assert_eq!(look.model_customization, "", "42/{opcode}");
            assert_eq!(look.ids[crate::style_cards::MODEL_CUSTOMIZATION], 0);
            assert_eq!(game.state().appearance, Some(look), "42/{opcode}: saved");
            assert_eq!(count(&game, CARD), None, "42/{opcode}: the card is spent");
            // A duplicate request after the last card is gone is a chat refusal, not an error.
            let before = state_bytes(&game);
            let again = use_card(&mut game, opcode, by);
            assert_eq!(chat_lines(&again), ["You do not have that style card."]);
            assert!(!again.changed && again.nearby.is_empty());
            assert_eq!(state_bytes(&game), before, "42/{opcode}: duplicate");
        }
    });
}

#[test]
fn last_card_is_deleted_larger_stack_counts_down() {
    each_content(|content| {
        let mut single = content.game(&[(CARD, SKULL_FACE_PAINT, 1)], &boy(), true);
        let out = use_card(&mut single, 12, SKULL_FACE_PAINT);
        assert_eq!(
            out.replies.first(),
            Some(&delete(CARD)),
            "the last card sends 38/4"
        );

        let mut stack = content.game(&[(CARD, SKULL_FACE_PAINT, 3)], &boy(), true);
        let out = use_card(&mut stack, 10, CARD);
        assert_eq!(count(&stack, CARD), Some(2));
        assert!(out.replies.iter().all(|r| !r.starts_with(&[38, 0, 4, 0])));
        assert!(
            out.replies
                .first()
                .is_some_and(|r| r.starts_with(&[38, 0, 3, 0])),
            "a stack that keeps cards sends an item update: {:?}",
            out.replies
        );
        // The same card again would not change the look, so it is kept.
        let again = use_card(&mut stack, 10, CARD);
        assert_eq!(chat_lines(&again), ["You already look like that."]);
        assert_eq!(count(&stack, CARD), Some(2));
    });
}

#[test]
fn use_by_definition_takes_the_lowest_stack() {
    each_content(|content| {
        let cards = [(OTHER, SKULL_FACE_PAINT, 1), (CARD, SKULL_FACE_PAINT, 1)];
        let mut game = content.game(&cards, &boy(), true);
        let out = use_card(&mut game, 12, SKULL_FACE_PAINT);
        assert_eq!(
            out.replies.first(),
            Some(&delete(CARD)),
            "{:?}",
            out.replies
        );
        assert_eq!(count(&game, OTHER), Some(1));
        assert!(out.nearby.contains(&change(6, "skull", 1, CARD)));
    });
}

#[test]
fn refused_style_card_changes_nothing() {
    let cards = [
        (CARD, CLEAN_SHAVEN, 1),
        (OTHER, NOT_A_CARD, 1),
        (900_003, BEARD, 1),
        (900_004, CHINA_HEAD, 1),
        (900_005, MODEL_CARD, 1),
        (900_006, BUTTERFLY_WINGS, 1),
        (900_007, FAIRY_WINGS, 1),
    ];
    let boy_cases = [
        (request(10, 424_242), "You do not have that style card."),
        (request(12, FACE_TYPE), "You do not have that style card."),
        (request(10, OTHER), "That item is not a style card."),
        (request(10, 900_003), "You already look like that."),
        (request(10, 900_004), "That style card cannot be used."),
        (request(10, 900_005), "That style card cannot be used."),
        (
            request(12, BUTTERFLY_WINGS),
            "That style card is not made for your character.",
        ),
        (
            request(12, FAIRY_WINGS),
            "That style card does not fit your character.",
        ),
    ];
    each_content(|content| {
        let mut game = content.game(&cards, &boy(), true);
        for (request, text) in &boy_cases {
            let before = state_bytes(&game);
            let out = game.handle_in_world(request, &character(), NOW).unwrap();
            assert_eq!(chat_lines(&out), [*text], "{request:?}");
            assert_eq!(out.replies.len(), 1, "{request:?}: only the chat line");
            assert!(!out.changed && out.nearby.is_empty(), "{request:?}");
            assert_eq!(
                state_bytes(&game),
                before,
                "{request:?}: the save is unchanged"
            );
        }
        // A girl cannot use a boys' card; a server without the tables refuses everything.
        for (identity, tables, text) in [
            (
                girl(),
                true,
                "That style card is not made for your character.",
            ),
            (
                boy(),
                false,
                "Style cards are not available on this server.",
            ),
        ] {
            let mut game = content.game(&cards, &identity, tables);
            let before = state_bytes(&game);
            let out = use_card(&mut game, 10, CARD);
            assert_eq!(chat_lines(&out), [text]);
            assert!(!out.changed && out.nearby.is_empty());
            assert_eq!(state_bytes(&game), before);
        }
    });
}

/// Finding 5: a player with no look yet is told so, not that the server lacks style cards.
#[test]
fn a_player_with_no_look_gets_its_own_refusal() {
    let content = Content::new(false);
    let mut game = content.unidentified(&[(CARD, CLEAN_SHAVEN, 1)], true);
    assert!(game.look().is_none());
    let before = state_bytes(&game);
    let out = use_card(&mut game, 10, CARD);
    assert_eq!(
        chat_lines(&out),
        ["Your character's look is not ready yet."]
    );
    assert!(!out.changed && out.nearby.is_empty());
    assert_eq!(state_bytes(&game), before);
    assert_eq!(count(&game, CARD), Some(1));
}

/// Finding 4 (R): the player's own housing lot is where a card is used, so being inside it does
/// not count as busy; the card applies, is spent and is saved.
#[test]
fn a_card_works_inside_the_own_housing_lot() {
    let content = Content::new(false);
    // No starter kit: the round70 catalogue has no kit item definitions.
    let mut houses = crate::housing_service::HousingContent::bundled().unwrap();
    houses.starter_kit = None;
    fs::write(
        content.0.join("houses.json"),
        serde_json::to_vec(&houses).unwrap(),
    )
    .unwrap();
    let mut game = content.game(&[(CARD, CLEAN_SHAVEN, 1)], &boy(), true);
    let character = character();
    let grant = game
        .handle_in_world(&[127, 0, 23, 0], &character, NOW)
        .unwrap();
    let lot = crate::housing_native::InstanceList::decode(
        &grant
            .replies
            .iter()
            .find(|r| r.starts_with(&[127, 0, 39, 0]))
            .unwrap()[2..],
    )
    .unwrap()
    .records[0]
        .instance_guid;
    let mut enter = vec![127, 0, 19, 0];
    enter.extend(lot.to_le_bytes());
    enter.extend(0i32.to_le_bytes());
    enter.push(0);
    game.handle_in_world(&enter, &character, NOW).unwrap();
    assert!(
        game.house_return().is_some(),
        "the player is inside the lot"
    );

    let out = use_card(&mut game, 10, CARD);
    assert_eq!(chat_lines(&out), Vec::<String>::new(), "no refusal");
    assert!(out.changed);
    assert_eq!(out.nearby, [change(5, "", 0, CARD)]);
    assert_eq!(game.look().unwrap().model_customization, "");
    assert_eq!(count(&game, CARD), None, "the card is spent");
}

/// 26/13-style job changes and relogs: the look belongs to the character.
#[test]
fn look_survives_job_switch_and_relog() {
    let content = Content::new(true);
    let identity = boy();
    let mut game = content.game(&[(CARD, CLEAN_SHAVEN, 1)], &identity, true);
    use_card(&mut game, 10, CARD);
    let (look, fingerprint) = (game.look().unwrap(), game.identity_fingerprint.clone());
    assert!(fingerprint.is_some());
    // A job switch (the profile table is what 26/13 drives) leaves the look alone.
    let profiles = game.profiles.as_mut().unwrap();
    profiles.unlock(crate::mining::MINER_PROFILE_ID).unwrap();
    profiles
        .select(crate::mining::MINER_PROFILE_ID, &game.inventory)
        .unwrap();
    assert_eq!(game.look().unwrap(), look);
    assert_eq!(game.state().appearance, Some(look.clone()));

    let saved: GameplayState = serde_json::from_slice(&state_bytes(&game)).unwrap();
    let mut relogged = Gameplay::load(&content.0, Some(saved)).unwrap();
    assert_eq!(
        relogged.look().as_ref(),
        Some(&look),
        "the saved look needs no identity"
    );
    relogged.set_style_tables(Some(fixture::tables()));
    assert!(
        !relogged.apply_identity(&identity).unwrap(),
        "the identity still matches its fingerprint"
    );
    assert_eq!(relogged.look().unwrap(), look);
    assert_eq!(relogged.identity_fingerprint, fingerprint);
    assert_eq!(relogged.look().unwrap().model_customization, "");
    // The identity itself still has its beard.
    assert_eq!(
        crate::style_cards::Look::from_identity(&identity).model_customization,
        "human_m_facialhair_beard.adr"
    );
}

#[test]
fn a_save_without_a_style_card_look_is_unchanged() {
    let content = Content::new(false);
    let mut game = content.game(&[(CARD, CLEAN_SHAVEN, 1)], &boy(), true);
    let plain = state_bytes(&game);
    assert!(!String::from_utf8(plain).unwrap().contains("appearance"));
    use_card(&mut game, 10, CARD);
    assert!(
        String::from_utf8(state_bytes(&game))
            .unwrap()
            .contains("appearance")
    );
}

/// Fishing is not stopped by the router's earlier 42 gates, so this reaches the card's own
/// busy gate (`Gameplay::blocks`).
#[test]
fn a_busy_player_is_refused_without_spending_the_card() {
    let content = Content::new(true);
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    fs::copy(
        data.join("fishing_spots.json"),
        content.0.join("fishing_spots.json"),
    )
    .unwrap();
    let mut game = content.game(&[(CARD, CLEAN_SHAVEN, 1)], &boy(), true);
    let spots = fs::read(content.0.join("fishing_spots.json")).unwrap();
    game.fishing_spots = Some(Arc::new(
        crate::fishing_service::FishingSpots::parse(&spots).unwrap(),
    ));
    let character = character();
    game.begin_fishing(9_000_000_181, &character)
        .expect("fishing begins");
    assert!(!game.fishing.is_empty(), "a fishing round is live");
    let before = state_bytes(&game);
    let out = game
        .handle_in_world(&request(10, CARD), &character, NOW + 1)
        .unwrap();
    assert_eq!(
        chat_lines(&out),
        ["Finish what you are doing before using a style card."]
    );
    assert!(!out.changed && out.nearby.is_empty());
    assert_eq!(state_bytes(&game), before);
}

/// A live mining run is stopped by the router's own gate first; the card is still kept.
#[test]
fn a_mining_run_keeps_the_card() {
    let content = Content::new(true);
    crate::mining::write_content(&content.0, crate::mining::MINING_GAME_ID).unwrap();
    let mut game = content.game(&[(CARD, CLEAN_SHAVEN, 1)], &boy(), true);
    {
        let profiles = game.profiles.as_mut().unwrap();
        profiles.unlock(crate::mining::MINER_PROFILE_ID).unwrap();
        profiles
            .select(crate::mining::MINER_PROFILE_ID, &game.inventory)
            .unwrap();
    }
    game.set_dev_tools(true);
    game.set_role(AccountRole::Admin);
    let character = character();
    game.handle_in_world(&crate::chat::local_request("realm mine"), &character, NOW)
        .unwrap();
    assert!(
        game.mining.as_ref().is_some_and(Type8Engine::active),
        "mining runs"
    );
    let before = state_bytes(&game);
    let out = game
        .handle_in_world(&request(10, CARD), &character, NOW + 1)
        .unwrap();
    assert_eq!(
        out.replies.len(),
        1,
        "one chat line: {:?}",
        chat_lines(&out)
    );
    assert!(!out.changed && out.nearby.is_empty());
    assert_eq!(state_bytes(&game), before);
}
