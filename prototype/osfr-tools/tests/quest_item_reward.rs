//! T412: a quest item reward is granted exactly once, survives the save, and never pays
//! part of a reward.
//!
//! No enabled shipped quest authors an item reward (T318), so this suite builds one
//! synthetic enabled quest in a scratch content copy and drives it through the
//! production in-world router (`Gameplay::handle_in_world` -> `gameplay/npc.rs` ->
//! `QuestEngine::finish_turnin` -> `rewards::apply_grant`) with the packets the gateway
//! accepts. The catalogue is loaded by the production split loader
//! (`Gameplay::bind_npc_quest_catalogue` -> `QuestEngine::from_split_content`); no test
//! builds a `QuestDefinition` or calls the grant directly.
//!
//! Synthetic content (T301's drawable-giver pattern, `tests/playtest_first_hour.rs`):
//! quest 3999991 at giver 9_700_000_412 with one `talk_to_npc` objective at the giver
//! and the reward 7 coins + 2 of item 65822 + 3 Brawler (profile 43) job XP; quest
//! 3999992 at giver 9_700_000_413 is the slot the refusal cases rewrite. Item 65822 is
//! a captured opening-catalogue definition with `max_stack` 10, so two fit one stack;
//! item 8 (T301's reward item) has `max_stack` 1, so a count of 2 is refused by the
//! production stack rule and is pinned below as a refusal, not used as the grant.
//!
//! Client readers (2014 client): 26/8 `008b6bc0`, 26/10 `00a36f80`, 26/6 `009f5890`,
//! 49/2 `00a86400` (constructor), 49/14 `00a864c0` (constructor); 49/1 `QuestInfoPacket`
//! `00c7bb60` -> `ReadFields` `00c7b7a0`; 49/13 `QuestEndPacket::ReadFields`
//! `00c7b990`; both call the reward-bundle reader `008e7930`. 165/5 vendor sell
//! `00c76dd0` (writer `00c75a70`). No field is added: the item display row is the known
//! T318 gap (`definition_from_content` builds `reward.items` but no
//! `RewardDisplay.rows`), so both screens keep the 69-byte empty-row base.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    BAG_SLOTS, FILLER_ITEM, Fixture as ContentFixture, accept, active_job, character_at_for, coins,
    content_dir, dialogue_choice, end_screen_reward, fill_bag, finish_quest, has_family, interact,
    menu_option, npc_interact, npc_select, offer, pending_turnin, profile_xp, quest_end_reply,
    quest_reply, quest_status, read_json, release_end_screen, saved, select_job, send, talk,
};
use osfr_tools::{
    character_store::CharacterState,
    game_store,
    gameplay::{Gameplay, GameplayState, Outcome},
    inventory::{Inventory, InventoryItem},
    npc_interaction::{NpcInteraction, decode_quest_offer_49_1},
    quest_engine::{CharacterEligibility, QuestEngine, QuestStatus},
};
use serde_json::{Value, json};
use std::{fs, io, path::Path};

const A: u64 = 412_001;
const B: u64 = 412_002;
const QUEST: u32 = 3_999_991;
/// The second quest's reward is what the refusal cases rewrite.
const SECOND_QUEST: u32 = 3_999_992;
const GIVER: u64 = 9_700_000_412;
const SECOND_GIVER: u64 = 9_700_000_413;
const GIVER_POSITION: [f32; 3] = [-1893.204, -50.59, 307.902];
const SECOND_GIVER_POSITION: [f32; 3] = [-1893.204, -50.59, 327.902];
/// T301's synthetic giver model and name: test content, not a retail binding.
const GIVER_MODEL: i32 = 958;
const GIVER_NAME_ID: u32 = 19_685;
const TITLE: u32 = 9_741_201;
const END_TEXT: u32 = 9_741_202;
/// T301's switch target: unlocked in the seeded save, its outfit is in the opening bag.
const ARCHER: i32 = 35;
const BRAWLER: i32 = 43;
const COINS: u32 = 7;
const XP: u32 = 3;
const COUNT: u32 = 2;
/// Opening-catalogue definition with `max_stack` 10.
const ITEM: i32 = 65_822;
/// Opening-catalogue definition with `max_stack` 1.
const UNSTACKABLE: i32 = 8;
/// No opening-catalogue definition has this id.
const ABSENT: i32 = 999_999;
/// Melninster, a shipped vendor (`vendors.json`) whose shop buys any `stores.json` item.
const VENDOR: u64 = 9_500_000_045;
/// The 26/9 Shop option label (`npc_interaction::SHOP_STRING_ID`).
const SHOP_LABEL: u32 = 432_156;

struct Fixture {
    content: ContentFixture,
}

impl Fixture {
    /// The opening fixture plus the repository content, then the synthetic records.
    fn new() -> Self {
        let content = ContentFixture::for_character(A);
        content.overlay_content(&content_dir());
        // T256's field fixture rule (as T301): the opening inventory lacks optional
        // farming rewards, and no T412 step harvests.
        let _ = fs::remove_file(content.path().join("harvesting.json"));
        write_json(
            &content.path().join("npcs/quest_item_reward.json"),
            json!({"schema_version": 1, "records": [
                giver(GIVER, GIVER_POSITION),
                giver(SECOND_GIVER, SECOND_GIVER_POSITION),
            ]}),
        );
        let jobs_path = content.path().join("jobs.json");
        let mut jobs = read_json(&jobs_path);
        let brawler = jobs["records"]
            .as_array_mut()
            .expect("jobs.json records")
            .iter_mut()
            .find(|job| job["id"] == "brawler")
            .expect("the Brawler job row");
        let ids = brawler["quest_ids"].as_array_mut().expect("Brawler quests");
        ids.extend([json!(QUEST), json!(SECOND_QUEST)]);
        write_json(&jobs_path, jobs);
        let fixture = Self { content };
        fixture.write_second_reward(rewards(COINS, items(ITEM, COUNT)));
        fixture
    }

    fn path(&self) -> &Path {
        self.content.path()
    }

    /// Rewrite the second quest's reward rows; the first quest keeps the good reward.
    /// The next `game` binds the rewritten catalogue through the production loader.
    fn write_second_reward(&self, second: Value) {
        write_json(
            &self.path().join("quests/quest_item_reward.json"),
            json!({"schema_version": 1, "records": [
                quest(QUEST, GIVER, rewards(COINS, items(ITEM, COUNT))),
                quest(SECOND_QUEST, SECOND_GIVER, second),
            ]}),
        );
    }

    /// A session: the production loader, catalogue bind and world entry.
    fn game(&self, character_id: u64, state: Option<GameplayState>) -> Gameplay {
        let mut game = Gameplay::load(self.path(), state).expect("the fixture loads");
        game.bind_npc_quest_catalogue(self.path(), character_id)
            .expect("the split quest catalogue binds");
        game.initialize_world(&at(character_id, GIVER_POSITION), 0)
            .expect("the session world initializes");
        game
    }

    /// The opening state with one sellable filler stack, as a pre-login purchase.
    fn game_with_filler(&self, character_id: u64) -> Gameplay {
        let mut state = self.game(character_id, None).state();
        let id = 300_000;
        state.inventory.items.push(filler(id));
        state.inventory.next_instance_id = Some(i64::from(id) + 1);
        self.game(character_id, Some(state))
    }

    /// The real save envelope: `game_store::save` -> `load_or_create` -> the login bind.
    fn save_and_reload(&self, character_id: u64, game: &Gameplay) -> Gameplay {
        let path = self.path().join(format!("character-{character_id}.json"));
        let character = at(character_id, GIVER_POSITION);
        game_store::save(
            &path,
            &game_store::Snapshot {
                character: character.clone(),
                gameplay: Some(game.state()),
                pets: None,
            },
        )
        .expect("the character save writes");
        let snapshot =
            game_store::load_or_create(&path, character).expect("the character save loads");
        let reloaded = self.game(character_id, snapshot.gameplay);
        assert_eq!(
            saved(&reloaded),
            saved(game),
            "the save round trip kept the serialized state"
        );
        reloaded
    }
}

fn write_json(path: &Path, value: Value) {
    fs::write(path, serde_json::to_vec(&value).unwrap())
        .unwrap_or_else(|error| panic!("{} writes: {error}", path.display()));
}

fn source() -> Value {
    json!([{
        "kind": "reconstructed",
        "evidence": "(R) T412 synthetic item-reward gate: 7 coins, 2 of item 65822 and 3 Brawler (43) XP. Test-only values, not recovered retail rewards."
    }])
}

fn items(definition_id: i32, count: u32) -> Value {
    json!([{"definition_id": definition_id, "count": count, "source": source()}])
}

fn rewards(coins: u32, items: Value) -> Value {
    json!([{"coins": coins, "items": items, "job_xp": {"43": XP}, "source": source()}])
}

fn quest(id: u32, giver: u64, rewards: Value) -> Value {
    json!({
        "id": id,
        "title_string_id": TITLE,
        "end_text_string_id": END_TEXT,
        "giver_npc_id": giver,
        "prerequisite_quest_ids": [],
        "objectives": [{"kind": "talk_to_npc", "id": 1, "npc_id": giver, "source": source()}],
        "rewards": rewards,
        "source": source(),
        "enabled": true
    })
}

/// T301's drawable regional giver row: `legacy_data` + a finite position draws it,
/// `quest_giver` offers, and `npc_id` is the runtime actor id.
fn giver(npc_id: u64, position: [f32; 3]) -> Value {
    json!({
        "npc_id": npc_id,
        "name": "Synthetic item-reward giver",
        "name_string_id": GIVER_NAME_ID,
        "zone_id": 0,
        "position": position,
        "placement": "legacy_data",
        "model_id": GIVER_MODEL,
        "roles": ["quest_giver"],
        "quest_ids_given": [],
        "quests_given": [],
        "quests_accepted": [],
        "source": source()
    })
}

fn filler(id: i32) -> InventoryItem {
    InventoryItem {
        definition_id: FILLER_ITEM,
        tint: 0,
        id,
        count: 1,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: 0,
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    }
}

fn at(character_id: u64, position: [f32; 3]) -> CharacterState {
    character_at_for(character_id, 0, position)
}

fn count_of(game: &Gameplay, definition_id: i32) -> u32 {
    game.inventory_state()
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id)
        .map(|item| item.count)
        .sum()
}

/// Everything a reward could move.
#[derive(Debug, PartialEq)]
struct Economy {
    coins: u32,
    item: u32,
    brawler: u32,
    /// Job XP summed over every profile except Brawler, which the reward names.
    other_xp: u64,
}

fn economy(game: &Gameplay) -> Economy {
    Economy {
        coins: coins(game),
        item: count_of(game, ITEM),
        brawler: profile_xp(game, BRAWLER),
        other_xp: game
            .state()
            .profiles
            .as_ref()
            .unwrap()
            .experience
            .iter()
            .filter(|(profile, _)| **profile != BRAWLER)
            .map(|(_, xp)| u64::from(*xp))
            .sum(),
    }
}

/// Offer, accept, complete the one Talk at the giver and release the held 49/13.
/// Returns the 49/1 offer and the 49/13 end screen as the client receives them.
fn open_end_screen(
    game: &mut Gameplay,
    character_id: u64,
    quest_id: u32,
    giver: u64,
    position: [f32; 3],
    now: &mut u64,
) -> (Vec<u8>, Vec<u8>) {
    let at = at(character_id, position);
    let offered = offer(game, &at, giver, quest_id, *now);
    let offer = offered
        .replies
        .iter()
        .find(|packet| has_family(packet, 49, 1, Some(quest_id)))
        .cloned()
        .expect("the Talk sent the 49/1 offer");
    *now += 10;
    accept(game, &at, quest_id, *now);
    *now += 10;
    assert!(talk(game, &at, giver, *now).changed, "the Talk objective");
    assert_eq!(pending_turnin(game), Some(quest_id));
    *now += 10;
    let end = release_end_screen(game, &at, quest_id, giver, END_TEXT, *now);
    *now += 10;
    (offer, end)
}

/// The first quest's 49/14 pays exactly the authored reward once.
fn turn_in(game: &mut Gameplay, character_id: u64, now: u64) -> Outcome {
    let before = economy(game);
    let paid = finish_quest(
        game,
        &at(character_id, GIVER_POSITION),
        QUEST,
        BRAWLER,
        COINS,
        XP,
        now,
    );
    assert_eq!(
        economy(game),
        Economy {
            coins: before.coins + COINS,
            item: before.item + COUNT,
            brawler: before.brawler + XP,
            other_xp: before.other_xp,
        },
        "the turn-in pays 7 coins, 2 items and 3 Brawler XP, nothing else"
    );
    paid
}

/// A refused 49/14 pays nothing and keeps the pending end screen for a retry. Today a
/// grant failure is an `Err` the gateway drops without installing the staged game
/// (`Gateway::handle_at`); T382 may turn it into a player-local `Ok` refusal, which
/// this accepts too as long as it changes nothing.
fn assert_turn_in_refused(
    game: &mut Gameplay,
    character_id: u64,
    quest_id: u32,
    packet: &[u8],
    now: u64,
    label: &str,
) {
    let before = saved(game);
    match game.handle_in_world(packet, &at(character_id, GIVER_POSITION), now) {
        Ok(outcome) => {
            assert!(!outcome.changed, "{label}: the refusal changed gameplay");
            assert!(
                !outcome
                    .replies
                    .iter()
                    .any(|reply| has_family(reply, 49, 4, None)),
                "{label}: the refusal completed the quest"
            );
        }
        Err(error) => assert_eq!(
            error.kind(),
            io::ErrorKind::InvalidData,
            "{label}: unexpected failure {error}"
        ),
    }
    assert_eq!(saved(game), before, "{label}: the save bytes changed");
    assert_eq!(quest_status(game, quest_id), QuestStatus::Active, "{label}");
    assert_eq!(
        pending_turnin(game),
        Some(quest_id),
        "{label}: the pending end screen was consumed"
    );
}

/// 165/5 sell of one unit: u64 merchant, u32 item instance, u32 quantity (reader
/// `00c76dd0`, writer `00c75a70`).
fn sell_packet(instance: i32) -> Vec<u8> {
    let mut packet = vec![165, 0, 5, 0];
    packet.extend(VENDOR.to_le_bytes());
    packet.extend(instance.to_le_bytes());
    packet.extend(1_u32.to_le_bytes());
    packet
}

/// Walk to the shipped vendor, open its shop through the ordinary 26/8 -> 26/10 menu,
/// and sell one filler stack.
fn sell_one_filler(game: &mut Gameplay, character_id: u64, now: u64) -> Outcome {
    let content = content_dir();
    let (zone_id, position) =
        NpcInteraction::load(&content.join("npcs"), &content.join("stores.json"))
            .expect("NPC content loads")
            .location(VENDOR)
            .expect("the vendor is a positioned content spawn");
    let vendor = character_at_for(character_id, zone_id, position);
    let menu = interact(game, &vendor, VENDOR, now);
    let shop = menu_option(&menu, SHOP_LABEL);
    send(game, &vendor, &npc_select(VENDOR, shop), now + 1);
    let instance = game
        .inventory_state()
        .items
        .iter()
        .find(|item| item.definition_id == FILLER_ITEM)
        .expect("a filler stack to sell")
        .id;
    let sold = send(game, &vendor, &sell_packet(instance), now + 2);
    assert!(sold.changed, "the vendor sale commits");
    sold
}

fn switch_job(game: &mut Gameplay, character_id: u64, profile: i32, now: u64) {
    let switched = send(
        game,
        &at(character_id, GIVER_POSITION),
        &select_job(profile),
        now,
    );
    assert!(switched.changed, "the 26/13 switch to {profile} commits");
    assert_eq!(active_job(game), profile);
}

#[test]
fn the_turn_in_grants_two_of_the_item_once_and_the_save_keeps_them() {
    let fixture = Fixture::new();
    let catalogue = Inventory::load(fixture.path(), None).expect("the catalogue loads");
    assert!(catalogue.accepts_reward(ITEM, COUNT), "two fit one stack");
    let mut state = fixture.game(A, None).state();
    state.profiles.as_mut().unwrap().unlocked.insert(ARCHER);
    let mut game = fixture.game(A, Some(state));
    let mut now = 1_000;
    assert_eq!(count_of(&game, ITEM), 0, "a new character owns none");
    switch_job(&mut game, A, BRAWLER, now);
    let before = economy(&game);

    open_end_screen(&mut game, A, QUEST, GIVER, GIVER_POSITION, &mut now);
    // The pending reward lives in the saved `QuestState` until 49/14.
    let mut game = fixture.save_and_reload(A, &game);
    assert_eq!(pending_turnin(&game), Some(QUEST));
    assert_eq!(economy(&game), before, "the pending screen paid nothing");
    // The reward map names Brawler; the job active at the turn-in does not matter.
    switch_job(&mut game, A, ARCHER, now);
    now += 10;

    turn_in(&mut game, A, now);
    now += 10;
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);
    let paid = economy(&game);

    let mut game = fixture.save_and_reload(A, &game);
    assert_eq!(economy(&game), paid, "the save keeps stack, coins and XP");
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);
    let after = saved(&game);
    for (label, packet) in [
        ("replayed 49/14", quest_end_reply(QUEST)),
        ("replayed 49/2", quest_reply(QUEST, true)),
    ] {
        let outcome = send(&mut game, &at(A, GIVER_POSITION), &packet, now);
        now += 10;
        assert!(!outcome.changed, "{label} changed gameplay");
        assert_eq!(saved(&game), after, "{label} changed the save");
    }
    assert_eq!(economy(&game), paid, "a replay pays nothing");
}

#[test]
fn the_offer_and_end_screens_keep_the_base_bundle_and_grant_nothing() {
    let fixture = Fixture::new();
    let mut game = fixture.game(A, None);
    let mut now = 2_000;
    let before = economy(&game);
    let inventory = game.inventory_state().clone();
    let (offer, end) = open_end_screen(&mut game, A, QUEST, GIVER, GIVER_POSITION, &mut now);
    assert_eq!(
        economy(&game),
        before,
        "offer, accept and end screen pay nothing"
    );
    assert_eq!(game.inventory_state(), &inventory, "no item moved");

    let bundle =
        QuestEngine::from_split_content(fixture.path(), A, CharacterEligibility::default(), None)
            .expect("the catalogue loads")
            .quest_reward_bundle(QUEST)
            .expect("the reward encodes");
    // T318 gap: the item builds `reward.items` but no display row, so the body is the
    // 69-byte base (row count 0 at bundle 61..65) with the coin and summed-XP words.
    assert_eq!(bundle.len(), 69);
    assert_eq!(u32::from_le_bytes(bundle[1..5].try_into().unwrap()), COINS);
    assert_eq!(u32::from_le_bytes(bundle[5..9].try_into().unwrap()), XP);
    assert_eq!(i32::from_le_bytes(bundle[61..65].try_into().unwrap()), 0);
    let decoded = decode_quest_offer_49_1(&offer).expect("the 49/1 decodes");
    assert_eq!(
        decoded.reward_bundle_body, bundle,
        "the offer card's bundle"
    );
    // 49/13: actor, quest, end text and animation precede the bundle at 26.
    assert_eq!(
        end.get(26..26 + 69),
        Some(bundle.as_slice()),
        "the end screen"
    );
    assert_eq!(end_screen_reward(&end), (COINS, XP));

    turn_in(&mut game, A, now);
}

#[test]
fn a_full_bag_refuses_the_turn_in_without_partial_coins_or_xp() {
    let fixture = Fixture::new();
    let mut game = fixture.game(A, None);
    let mut now = 3_000;
    open_end_screen(&mut game, A, QUEST, GIVER, GIVER_POSITION, &mut now);

    let mut full = game.state();
    fill_bag(&mut full);
    let mut game = fixture.game(A, Some(full));
    assert_eq!(game.inventory_state().items.len(), BAG_SLOTS);
    assert_eq!(pending_turnin(&game), Some(QUEST));

    assert_turn_in_refused(
        &mut game,
        A,
        QUEST,
        &quest_end_reply(QUEST),
        now,
        "full bag",
    );
    now += 10;
    // A malformed 49/14 at the same screen is refused the same way.
    assert_turn_in_refused(
        &mut game,
        A,
        QUEST,
        &quest_end_reply(QUEST)[..8],
        now,
        "truncated 49/14",
    );
    now += 10;

    // Free one slot through a real vendor sale, then the same turn-in pays once.
    sell_one_filler(&mut game, A, now);
    let sold = economy(&game);
    now += 10;
    assert_eq!(game.inventory_state().items.len(), BAG_SLOTS - 1);
    turn_in(&mut game, A, now);
    assert_eq!(coins(&game), sold.coins + COINS);
    now += 10;
    let paid = saved(&game);
    let outcome = send(
        &mut game,
        &at(A, GIVER_POSITION),
        &quest_end_reply(QUEST),
        now,
    );
    assert!(!outcome.changed);
    assert_eq!(saved(&game), paid, "the retried reward paid exactly once");
}

#[test]
fn an_unknown_item_definition_is_refused_without_partial_coins_or_xp() {
    let fixture = Fixture::new();
    let catalogue = Inventory::load(fixture.path(), None).expect("the catalogue loads");
    assert!(!catalogue.has_definition(ABSENT));
    assert!(
        !catalogue.accepts_reward(UNSTACKABLE, COUNT),
        "item 8 stacks to 1"
    );
    // The catalogue accepts these rows (only the inventory knows definitions and stack
    // limits), so the 49/14 grant is where they are refused.
    for (label, bad) in [
        ("absent definition", items(ABSENT, 1)),
        ("over the stack limit", items(UNSTACKABLE, COUNT)),
        (
            "a good row before a bad row",
            json!([
                {"definition_id": ITEM, "count": 1, "source": source()},
                {"definition_id": ABSENT, "count": 1, "source": source()},
            ]),
        ),
    ] {
        fixture.write_second_reward(rewards(COINS, bad));
        let mut first = fixture.game(A, None);
        let mut second = fixture.game(B, None);
        let mut now = 4_000;
        open_end_screen(
            &mut first,
            A,
            SECOND_QUEST,
            SECOND_GIVER,
            SECOND_GIVER_POSITION,
            &mut now,
        );
        assert_turn_in_refused(
            &mut first,
            A,
            SECOND_QUEST,
            &quest_end_reply(SECOND_QUEST),
            now,
            label,
        );
        // The refused player's session keeps ticking, and another player is paid.
        first
            .tick(&at(A, SECOND_GIVER_POSITION), now + 1)
            .unwrap_or_else(|error| panic!("{label}: the tick stopped: {error}"));
        open_end_screen(&mut second, B, QUEST, GIVER, GIVER_POSITION, &mut now);
        turn_in(&mut second, B, now);
    }
}

/// Rows the quest loader itself rejects (`definition_from_content` sums with checked
/// arithmetic; `QuestEngine::new` refuses a non-positive definition or a zero count), so
/// no such quest is ever offered and nothing can be partly paid.
#[test]
fn a_malformed_reward_is_refused_when_the_catalogue_loads() {
    let fixture = Fixture::new();
    for (label, rows) in [
        ("zero count", rewards(COINS, items(ITEM, 0))),
        ("negative definition", rewards(COINS, items(-ITEM, 1))),
        (
            "coins overflow",
            json!([
                {"coins": u32::MAX, "items": [], "job_xp": {}, "source": source()},
                {"coins": 1, "items": [], "job_xp": {}, "source": source()},
            ]),
        ),
        (
            "job XP overflow",
            json!([
                {"coins": 0, "items": [], "job_xp": {"43": u32::MAX}, "source": source()},
                {"coins": 0, "items": [], "job_xp": {"43": 1}, "source": source()},
            ]),
        ),
    ] {
        fixture.write_second_reward(rows);
        let mut game = Gameplay::load(fixture.path(), None).expect("the fixture loads");
        let error = game
            .bind_npc_quest_catalogue(fixture.path(), A)
            .expect_err(label);
        assert_eq!(error.kind(), io::ErrorKind::InvalidData, "{label}: {error}");
    }
}

#[test]
fn two_players_each_get_their_own_item_reward() {
    let fixture = Fixture::new();
    let mut first = fixture.game(A, None);
    let mut second = fixture.game(B, None);
    let mut now = 5_000;
    open_end_screen(&mut first, A, QUEST, GIVER, GIVER_POSITION, &mut now);

    // B has no pending screen: B's 49/14 cannot pay A's reward, and A's screen stays.
    let second_before = saved(&second);
    let outcome = send(
        &mut second,
        &at(B, GIVER_POSITION),
        &quest_end_reply(QUEST),
        now,
    );
    assert!(!outcome.changed);
    assert_eq!(saved(&second), second_before);
    assert_eq!(pending_turnin(&first), Some(QUEST));

    // A full bag on B does not block A's turn-in.
    open_end_screen(&mut second, B, QUEST, GIVER, GIVER_POSITION, &mut now);
    let mut full = second.state();
    fill_bag(&mut full);
    let mut second = fixture.game(B, Some(full));
    assert_turn_in_refused(
        &mut second,
        B,
        QUEST,
        &quest_end_reply(QUEST),
        now,
        "B full",
    );
    turn_in(&mut first, A, now);
    assert_eq!(count_of(&first, ITEM), COUNT);
    assert_eq!(count_of(&second, ITEM), 0, "A's grant never reaches B");

    // B frees a slot and is paid its own two.
    sell_one_filler(&mut second, B, now + 10);
    turn_in(&mut second, B, now + 20);
    assert_eq!(count_of(&second, ITEM), COUNT);
    assert_eq!(count_of(&first, ITEM), COUNT, "B's grant never reaches A");
}

#[test]
fn a_live_inventory_change_before_the_turn_in_survives_the_grant() {
    let fixture = Fixture::new();
    let mut game = fixture.game_with_filler(A);
    let mut now = 6_000;
    open_end_screen(&mut game, A, QUEST, GIVER, GIVER_POSITION, &mut now);
    let opened = economy(&game);

    // After the end screen opens, sell the filler: an item leaves and coins arrive.
    sell_one_filler(&mut game, A, now);
    now += 10;
    let live = economy(&game);
    assert!(live.coins > opened.coins, "the sale paid coins");
    assert_eq!(count_of(&game, FILLER_ITEM), 0, "the sale took the item");

    turn_in(&mut game, A, now);
    assert_eq!(
        coins(&game),
        live.coins + COINS,
        "the reward lands on the live coins, not the end-screen snapshot"
    );
    assert_eq!(count_of(&game, FILLER_ITEM), 0, "the sold item stays sold");
}

#[test]
fn the_gate_uses_no_dev_commands() {
    let fixture = Fixture::new();
    let mut game = fixture.game_with_filler(A);
    for command in ["teleport", "here", "give", "coins", "item", "offer"] {
        assert!(
            !game.may_run(command),
            "the session may not run {command:?}"
        );
    }
    // Every request shape this suite and the shared helpers send (26/8, 26/10, 26/6,
    // 26/13, 49/2, 49/14, 165/5): none is a 15/x chat request, so none can carry `realm`.
    for packet in [
        npc_interact(GIVER),
        npc_select(GIVER, 1),
        dialogue_choice(301),
        select_job(BRAWLER),
        quest_reply(QUEST, true),
        quest_end_reply(QUEST),
        sell_packet(1),
    ] {
        assert!(!packet.starts_with(&[15, 0]), "a chat request: {packet:?}");
    }
    let mut now = 7_000;
    open_end_screen(&mut game, A, QUEST, GIVER, GIVER_POSITION, &mut now);
    sell_one_filler(&mut game, A, now);
    let paid = turn_in(&mut game, A, now + 10);
    assert!(
        !paid
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[15, 0])),
        "the turn-in answered with a chat/system line"
    );
    assert_eq!(pending_turnin(&game), None, "the character is free again");
}
