//! T301: prove the first playtest hour end to end (test-only acceptance gate).
//!
//! One new character accepts a quest from a drawable giver, kills one real field enemy
//! with the ordinary basic attack, turns the quest in for coins/job XP/an item, saves
//! through `game_store`, and still has everything after a relog. No developer command,
//! no reward helper and no test copy of the logic is used; every step is a production
//! request:
//!
//! - `Gameplay::handle_in_world` for 26/8, 26/10, 26/6, 26/13, 49/2, 49/14, 45/7 and the
//!   forged 39/x packets;
//! - `Gameplay::handle_shared_field` for the parsed 36/10 basic attack (the shared-field
//!   request `Session::commit_outcome` runs), `Gameplay::take_shared_fields` for the
//!   server-owned field population;
//! - `Gameplay::load` + `bind_npc_quest_catalogue` (the production reconnect bind) and the
//!   real `game_store` envelope for the save/reload points.
//!
//! Synthetic content (a scratch copy built from the opening fixture + `data/content`, never
//! the shipped tree): quest 3999999 at giver 9_700_000_301, one `defeat` objective on field
//! definition 3156, count 1, reward 10 coins + 20 Brawler (profile 43) job XP + one
//! non-stackable item 8; a second quest 3999998 behind 3999999 for the "next offered
//! quest" check. The single wolf is a verbatim copy of the shipped T245
//! `starting_farnum_wolves` actor 131000102001 at the W102-F1 centre, its group carrying
//! one member. Quest ids, title/end-text locale ids and reward values are test content,
//! not claimed retail values.
//!
//! Client readers/writers (2014 client, `lab/autopilot/ida_query.py`, this run):
//! - 26/8 `008b6bc0`: two GUID dwords + one bool (13-byte body).
//! - 26/10 `00a36f80`: two GUID dwords + i32 choice (16-byte body).
//! - 26/6 `009f5890`: i32 dialogue row; 301 releases the held quest screen.
//! - 49/2 `00a86400`: quest id + accept byte; 49/14 `00a864c0`: one quest id.
//! - 49/1 reader `00c7bb60` -> `00c7b7a0`: quest at 6, intro at 10, title at 14/18,
//!   description at 22; the tests decode the offer with the production reader.
//! - 49/13 reader `00c7b990`: u64 actor at 6..14, quest at 14..18, end text at 18..22,
//!   animation at 22..26, then the reward bundle (success at 26, coins 27..31, stars
//!   31..35) consumed by `008e7930`.
//! - Quest base reader `00c7acd0`: u16 family + i32 subtype (the six-byte body header).
//! - 49/9 reader `00c7af60` and 49/10 reader `00c7b070`: quest, objective, current counter
//!   at 6..10/10..14/14..18.
//! - 36/10 `ClientRequestStartAbility`; writer `00a342e0` and action-bar request path
//!   `00a35150` (bar, slot, target mode, target GUID). The 24-byte body this suite sends
//!   is the same one `tests/shared_field_state.rs` pins.
//! - 35/2 AddNpc body reader `00921a10`; the drawable-giver check parses it through the
//!   production `startup_world::parse_add_npc`.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    CHARACTER, Fixture as ContentFixture, accept, active_job, character_at_for, coins, content_dir,
    counts, end_screen_reward, finish_quest, has_family, offer, pending_offer, pending_turnin,
    profile_xp, quest_end_reply, quest_reply, quest_status, read_json, release_end_screen, saved,
    select_job, send, talk,
};
use osfr_tools::{
    character_store::CharacterState,
    encounter::{SharedFieldAction, SharedFieldState},
    game_store,
    gameplay::{Gameplay, GameplayState, Outcome},
    inventory::InventoryItem,
    quest_engine::QuestStatus,
    startup_world,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs, io,
    path::{Path, PathBuf},
};

const QUEST: u32 = 3_999_999;
/// Only eligible after `QUEST`, so a second Talk can offer the next quest.
const NEXT_QUEST: u32 = 3_999_998;
const GIVER: u64 = 9_700_000_301;
/// A verified drawable NPC model (Samantha's row) so the giver's 35/2 is renderable.
const GIVER_MODEL: i32 = 958;
const GIVER_NAME_ID: u32 = 19_685;
/// T245 `starting_farnum_wolves` actor at the W102-F1 centre.
const WOLF_GUID: u64 = 131_000_102_001;
const WOLF_DEFINITION: u32 = 3_156;
const WOLF_CENTRE: [f32; 3] = [-1899.204, -50.59, 307.902];
const WOLF_HP: u32 = 100;
/// `SharedFieldAction::Ability { slot: 0, .. }` is the ordinary 36/10 basic attack.
const ATTACK_SLOT: i32 = 0;
const BRAWLER: i32 = 43;
const ARCHER: i32 = 35;
const ADVENTURER: i32 = 1;
const QUEST_COINS: u32 = 10;
const QUEST_XP: u32 = 20;
/// `bowl_basic.adr`, `max_stack: 1`, not in the opening inventory: the grant must add a
/// new stack, so the full-bag refusal is about the bag and not about a merge.
const REWARD_ITEM: i32 = 8;
/// `field_kill_rewards.json`: the level 1-2 band (and the 3156 override).
const FIELD_COINS: u32 = 2;
const FIELD_XP: u32 = 5;
const TITLE: u32 = 9_700_101;
const END_TEXT: u32 = 9_700_102;
/// (R) The pre-existing production turn-in floor in `gameplay/npc.rs`:
/// `items.len().max(100)`. A new stack at or above that count is refused.
const BAG_SLOTS: usize = 100;
const FILLER_ITEM: i32 = 75_060;
const GIVER_POSITION: [f32; 3] = [WOLF_CENTRE[0] + 6.0, WOLF_CENTRE[1], WOLF_CENTRE[2]];

/// One private gameplay content tree: the opening fixture's profile/inventory/opening
/// files plus the repository NPC/quest/field tree, then the T301 synthetic records.
struct Fixture {
    content: ContentFixture,
}

impl Fixture {
    fn new() -> Self {
        let content = ContentFixture::for_character(CHARACTER);
        content.overlay_content(&content_dir());
        // T256's field fixture rule: the opening inventory does not carry every optional
        // farming reward, and no T301 step uses harvesting.
        let _ = fs::remove_file(content.path().join("harvesting.json"));
        write_json(&content.path().join("field_enemies.json"), wolf_content());
        write_json(
            &content.path().join("quests/playtest_first_hour.json"),
            json!({
                "schema_version": 1,
                "records": [
                    quest_record(QUEST, &[], QUEST_COINS, true),
                    quest_record(NEXT_QUEST, &[QUEST], 1, false),
                ]
            }),
        );
        write_json(
            &content.path().join("npcs/playtest_first_hour.json"),
            json!({"schema_version": 1, "records": [giver_record()]}),
        );
        let jobs_path = content.path().join("jobs.json");
        let mut jobs = read_json(&jobs_path);
        let brawler = jobs["records"]
            .as_array_mut()
            .expect("jobs.json records")
            .iter_mut()
            .find(|job| job["id"] == "brawler")
            .expect("the Brawler job row");
        let ids = brawler["quest_ids"]
            .as_array_mut()
            .expect("the Brawler quest list");
        for id in [QUEST, NEXT_QUEST] {
            if !ids.contains(&json!(id)) {
                ids.push(json!(id));
            }
        }
        write_json(&jobs_path, jobs);
        Self { content }
    }

    fn path(&self) -> &Path {
        self.content.path()
    }

    fn giver_character(&self) -> CharacterState {
        character_at_for(CHARACTER, 0, GIVER_POSITION)
    }

    fn wolf_character(&self) -> CharacterState {
        character_at_for(
            CHARACTER,
            0,
            [WOLF_CENTRE[0] + 1.0, WOLF_CENTRE[1], WOLF_CENTRE[2]],
        )
    }

    /// A fresh session: the opening state with Archer unlocked as well, loaded through the
    /// production loader/bind and entered the way a world session enters.
    fn game(&self) -> Gameplay {
        let mut state = Gameplay::load(self.path(), None)
            .expect("the playtest fixture loads")
            .state();
        {
            let profiles = state
                .profiles
                .as_mut()
                .expect("the opening fixture supplies profiles");
            profiles.active = ADVENTURER;
            profiles.unlocked.insert(ARCHER);
            profiles.experience.clear();
        }
        self.game_at(Some(state), &self.giver_character())
    }

    fn game_at(&self, state: Option<GameplayState>, character: &CharacterState) -> Gameplay {
        let mut game = Gameplay::load(self.path(), state).expect("the playtest fixture loads");
        game.bind_npc_quest_catalogue(self.path(), CHARACTER)
            .expect("the split quest catalogue binds");
        game.initialize_world(character, 0)
            .expect("the session world initializes");
        game
    }

    /// A reconnect: the saved state through the real loader and catalogue bind. The session's
    /// `initialize_world` is deliberately not replayed here; the assertions are about saved
    /// quest/economy state, and this is the `Fixture::reload` shape the quest suites use.
    fn reload(&self, state: GameplayState) -> Gameplay {
        let mut game = Gameplay::load(self.path(), Some(state)).expect("the saved state loads");
        game.bind_npc_quest_catalogue(self.path(), CHARACTER)
            .expect("the saved quest catalogue binds");
        game
    }

    fn save_path(&self) -> PathBuf {
        self.path().join("character.json")
    }

    fn write_save_to(&self, path: &Path, game: &Gameplay, character: &CharacterState) {
        let snapshot = game_store::Snapshot {
            character: character.clone(),
            gameplay: Some(game.state()),
        };
        game_store::save(path, &snapshot).expect("the character save writes");
    }

    fn save_and_reload(&self, game: &Gameplay, character: &CharacterState) -> Gameplay {
        self.write_save_to(&self.save_path(), game, character);
        let loaded = game_store::load_or_create(&self.save_path(), character.clone())
            .expect("the character save loads");
        self.reload(loaded.gameplay.expect("the save envelope carries gameplay"))
    }
}

fn write_json(path: &Path, value: Value) {
    fs::write(
        path,
        serde_json::to_vec(&value).expect("fixture JSON serializes"),
    )
    .unwrap_or_else(|error| panic!("fixture {} writes: {error}", path.display()));
}

fn source() -> Value {
    json!([{
        "kind": "reconstructed",
        "evidence": "(R) T301 synthetic first-hour acceptance content: one 3156 defeat, 10 coins, 20 Brawler (43) XP and item 8. Test-only values, not recovered retail rewards."
    }])
}

fn quest_record(id: u32, prerequisites: &[u32], coins: u32, item: bool) -> Value {
    let items = if item {
        json!([{"definition_id": REWARD_ITEM, "count": 1, "source": source()}])
    } else {
        json!([])
    };
    let job_xp = if item {
        json!({"43": QUEST_XP})
    } else {
        json!({})
    };
    json!({
        "id": id,
        "title_string_id": TITLE,
        "end_text_string_id": END_TEXT,
        "giver_npc_id": GIVER,
        "prerequisite_quest_ids": prerequisites,
        "objectives": [{
            "kind": "defeat",
            "id": 1,
            "model_id": WOLF_DEFINITION,
            "count": 1,
            "source": source()
        }],
        "rewards": [{"coins": coins, "items": items, "job_xp": job_xp, "source": source()}],
        "source": source(),
        "enabled": true
    })
}

/// A drawable regional giver: `placement: legacy_data` plus a finite position is the
/// `NpcVisibility` draw rule, `quest_giver` is the offer rule, and `npc_id` is the
/// runtime actor id for a regional row.
fn giver_record() -> Value {
    json!({
        "npc_id": GIVER,
        "name": "Synthetic first-hour giver",
        "name_string_id": GIVER_NAME_ID,
        "zone_id": 0,
        "position": GIVER_POSITION,
        "placement": "legacy_data",
        "model_id": GIVER_MODEL,
        "roles": ["quest_giver"],
        "quest_ids_given": [],
        "quests_given": [],
        "quests_accepted": [],
        "source": source()
    })
}

/// The shipped T245 group and actor, narrowed to one always-on wolf. Copying the verified
/// rows keeps the W102 centre, 100 HP/7 damage, 45 s respawn and `standard` power-up table.
fn wolf_content() -> Value {
    let shipped = read_json(&content_dir().join("field_enemies.json"));
    let enemy = shipped["enemies"]
        .as_array()
        .expect("shipped field enemies")
        .iter()
        .find(|enemy| enemy["guid"] == json!(WOLF_GUID))
        .expect("the T245 starting_farnum_wolves actor")
        .clone();
    assert_eq!(enemy["definition_id"], json!(WOLF_DEFINITION));
    assert_eq!(enemy["max_health"], json!(WOLF_HP));
    assert_eq!(enemy["power_up_table"], json!("standard"));
    let position: [f32; 3] =
        serde_json::from_value(enemy["position"].clone()).expect("the actor position");
    assert_eq!(position, WOLF_CENTRE);
    let mut group = shipped["spawn_groups"]
        .as_array()
        .expect("shipped spawn groups")
        .iter()
        .find(|group| group["id"] == json!("starting_farnum_wolves"))
        .expect("the T245 starting_farnum_wolves group")
        .clone();
    group["members"] = json!([WOLF_GUID]);
    json!({"schema_version": 1, "spawn_groups": [group], "enemies": [enemy]})
}

fn wolf_content_with_definition(definition_id: i32) -> Value {
    let mut content = wolf_content();
    content["enemies"][0]["definition_id"] = json!(definition_id);
    content
}

/// The first-hour suite sends no chat request (15/1), so it cannot carry a developer
/// `realm` command; the shared packet builders used here are the quest/NPC shapes above.
fn assert_not_chat(packet: &[u8]) {
    assert!(
        !packet.starts_with(&[15, 0]),
        "T301 must not send a chat/dev command"
    );
}

/// 26/13 job switch, offer, accept. Brawler is the combat job with the opening starter
/// weapon; the quest itself has no job gate.
fn start_quest(game: &mut Gameplay, at: &CharacterState, now: &mut u64) {
    *now += 10;
    let switch = select_job(BRAWLER);
    assert_not_chat(&switch);
    let switched = send(game, at, &switch, *now);
    assert!(switched.changed, "the Brawler job must activate");
    assert_eq!(active_job(game), BRAWLER);
    *now += 10;
    offer(game, at, GIVER, QUEST, *now);
    *now += 10;
    accept(game, at, QUEST, *now);
    assert_eq!(pending_offer(game), None);
}

// 49/9 QuestObjectiveUpdate reader 0x00c7af60: quest, objective, current counter.
fn objective_update(replies: &[Vec<u8>], quest: u32, objective: u32, current: u32) -> bool {
    replies.iter().any(|packet| {
        has_family(packet, 49, 9, None)
            && packet.len() >= 18
            && u32::from_le_bytes(packet[6..10].try_into().unwrap()) == quest
            && u32::from_le_bytes(packet[10..14].try_into().unwrap()) == objective
            && u32::from_le_bytes(packet[14..18].try_into().unwrap()) == current
    })
}

// 49/10 QuestObjectiveComplete reader 0x00c7b070: same prefix, quest + objective.
fn objective_complete(replies: &[Vec<u8>], quest: u32, objective: u32) -> bool {
    replies.iter().any(|packet| {
        has_family(packet, 49, 10, None)
            && packet.len() >= 14
            && u32::from_le_bytes(packet[6..10].try_into().unwrap()) == quest
            && u32::from_le_bytes(packet[10..14].try_into().unwrap()) == objective
    })
}

fn enemy_hp(replies: &[Vec<u8>], guid: u64) -> Option<u32> {
    replies.iter().find_map(|packet| {
        (has_family(packet, 35, 5, None)
            && packet.get(4..12) == Some(guid.to_le_bytes().as_slice()))
        .then(|| u32::from_le_bytes(packet[12..16].try_into().unwrap()))
    })
}

/// The ordinary 36/10 basic attack (`ClientRequestStartAbility`; writer `0x00a342e0`)
/// through the production shared-field seam.
fn kill_wolf(
    game: &mut Gameplay,
    field: &mut SharedFieldState,
    at: &CharacterState,
    now: u64,
) -> Outcome {
    let known = BTreeSet::from([WOLF_GUID]);
    game.handle_shared_field(
        &SharedFieldAction::Ability {
            slot: ATTACK_SLOT,
            target: WOLF_GUID,
        },
        field,
        &known,
        at,
        now,
    )
    .expect("the shared-field basic attack is accepted")
}

fn count_definition(game: &Gameplay, definition_id: i32) -> usize {
    game.inventory_state()
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id && item.rental.is_none())
        .map(|item| usize::try_from(item.count).unwrap_or(usize::MAX))
        .sum()
}

/// Assert a refusal changes the whole gameplay state and its serialized bytes. An `Err`
/// and an unchanged `Ok` are both refusals; a changed or a quest-add/complete reply fails.
fn assert_refused(game: &mut Gameplay, at: &CharacterState, packet: &[u8], now: u64, label: &str) {
    assert_not_chat(packet);
    let before_state = game.state();
    let before_bytes = saved(game);
    match game.handle_in_world(packet, at, now) {
        Ok(outcome) => {
            assert!(!outcome.changed, "{label} changed gameplay");
            assert!(
                !outcome.replies.iter().any(|reply| {
                    has_family(reply, 49, 3, None) || has_family(reply, 49, 4, None)
                }),
                "{label} added or completed a quest"
            );
        }
        Err(error) => assert_eq!(
            error.kind(),
            io::ErrorKind::InvalidData,
            "{label} failed unexpectedly: {error}"
        ),
    }
    assert_eq!(
        game.state(),
        before_state,
        "{label} changed the gameplay state"
    );
    assert_eq!(
        saved(game),
        before_bytes,
        "{label} changed the serialized save bytes"
    );
}

/// Write the current state to a real game_store envelope and return its path and bytes;
/// the caller compares the bytes after a refusal.
fn save_bytes(
    fixture: &Fixture,
    game: &Gameplay,
    at: &CharacterState,
    name: &str,
) -> (PathBuf, Vec<u8>) {
    let path = fixture.path().join(name);
    fixture.write_save_to(&path, game, at);
    let bytes = fs::read(&path).expect("the save envelope is readable");
    (path, bytes)
}

/// Fill the bag with real filler stacks. The production turn-in floor is
/// `items.len().max(100)`, so a new reward stack cannot fit once the bag is at that size.
fn fill_bag(state: &mut GameplayState) {
    let base = 100_000;
    for offset in 0..BAG_SLOTS {
        state.inventory.items.push(InventoryItem {
            definition_id: FILLER_ITEM,
            tint: 0,
            id: base + offset as i32,
            count: 1,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: 0,
            rental: None,
            ability_count: 0,
            activate_enabled: false,
        });
    }
    state.inventory.next_instance_id = Some(i64::from(base) + BAG_SLOTS as i64);
}

#[test]
fn a_new_character_accepts_a_quest_kills_a_field_enemy_turns_it_in_and_relogs_with_the_reward() {
    let fixture = Fixture::new();
    let mut game = fixture.game();
    let at = fixture.giver_character();
    let mut now = 1_000;

    // The giver is drawable through the production visibility batch, before the field
    // runtime takes over the shared interest view.
    let drawn = game
        .update_npc_visibility(&at)
        .expect("the relog visibility batch is built");
    let spawned = drawn
        .iter()
        .filter_map(|packet| startup_world::parse_add_npc(packet).ok())
        .find(|npc| npc.actor_id == GIVER)
        .expect("the giver's 35/2 AddNpc is in the production visibility batch");
    assert_eq!(spawned.model_id, GIVER_MODEL);

    start_quest(&mut game, &at, &mut now);

    // One ordinary 36/10 basic attack against the server-spawned actor.
    let wolf = fixture.wolf_character();
    let mut field = game.take_shared_fields().expect("the shared field exists");
    assert_eq!(field.health(WOLF_GUID), Some(WOLF_HP));
    let coins_before_kill = coins(&game);
    let xp_before_kill = profile_xp(&game, BRAWLER);
    let killed = kill_wolf(&mut game, &mut field, &wolf, now);
    now += 700;
    assert!(killed.changed, "the kill must change gameplay");
    assert_eq!(
        field.health(WOLF_GUID),
        Some(0),
        "the server's wolf is dead"
    );
    assert_eq!(
        enemy_hp(&killed.replies, WOLF_GUID),
        Some(0),
        "the killer sees the 35/5 zero-health update"
    );
    assert_eq!(counts(&game, QUEST), [1], "one kill is one objective count");
    assert!(
        objective_update(&killed.replies, QUEST, 1, 1),
        "49/9 reports the defeat objective at 1/1"
    );
    assert!(
        objective_complete(&killed.replies, QUEST, 1),
        "49/10 completes the only objective"
    );
    assert_eq!(
        coins(&game),
        coins_before_kill + FIELD_COINS,
        "the normal field reward still pays the killer"
    );
    assert_eq!(
        profile_xp(&game, BRAWLER),
        xp_before_kill + FIELD_XP,
        "the normal field reward pays the active job"
    );
    assert_eq!(
        quest_status(&game, QUEST),
        QuestStatus::Active,
        "the ready quest is still active until its turn-in"
    );

    // The active job changes between kill and turn-in; the reward map names profile 43.
    now += 10;
    let switched = send(&mut game, &at, &select_job(ARCHER), now);
    assert!(switched.changed, "the Archer job must activate");
    assert_eq!(active_job(&game), ARCHER);
    let archer_before = profile_xp(&game, ARCHER);
    let brawler_before = profile_xp(&game, BRAWLER);
    let coins_before = coins(&game);
    let items_before = count_definition(&game, REWARD_ITEM);
    now += 10;

    let dialogue = talk(&mut game, &at, GIVER, now);
    now += 10;
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "the completion Talk opens its dialogue"
    );
    assert!(
        !dialogue
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 13, None)),
        "the 49/13 waits for the client's 26/6 dialogue choice"
    );
    assert_eq!(pending_turnin(&game), Some(QUEST));
    let screen = release_end_screen(&mut game, &at, QUEST, GIVER, END_TEXT, now);
    now += 10;
    // 49/13 reader 0x00c7b990: actor 6..14, quest 14..18, end text 18..22, bundle at 26.
    assert_eq!(
        end_screen_reward(&screen),
        (QUEST_COINS, QUEST_XP),
        "49/13 carries the authored coin and star display"
    );

    finish_quest(&mut game, &at, QUEST, BRAWLER, QUEST_COINS, QUEST_XP, now);
    now += 10;
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);
    assert_eq!(pending_turnin(&game), None);
    assert_eq!(coins(&game), coins_before + QUEST_COINS, "exactly 10 coins");
    assert_eq!(
        profile_xp(&game, BRAWLER),
        brawler_before + QUEST_XP,
        "exactly 20 Brawler XP on the configured profile"
    );
    assert_eq!(
        profile_xp(&game, ARCHER),
        archer_before,
        "the active job is not the configured reward profile"
    );
    assert_eq!(
        count_definition(&game, REWARD_ITEM),
        items_before + 1,
        "exactly one reward item"
    );

    // A repeated 49/14 and a repeated 49/2 pay and change nothing.
    let paid_state = game.state();
    let paid_bytes = saved(&game);
    let save_path = fixture.save_path();
    fixture.write_save_to(&save_path, &game, &at);
    let paid_save = fs::read(&save_path).expect("the paid save is readable");
    assert_refused(
        &mut game,
        &at,
        &quest_end_reply(QUEST),
        now + 1,
        "replayed 49/14",
    );
    assert_refused(
        &mut game,
        &at,
        &quest_reply(QUEST, true),
        now + 2,
        "replayed 49/2",
    );
    assert_eq!(game.state(), paid_state);
    assert_eq!(saved(&game), paid_bytes);
    assert_eq!(
        fs::read(&save_path).expect("the save is readable"),
        paid_save,
        "a replayed packet rewrote the character save"
    );

    // Save through the real envelope and reload through the real loader/bind.
    let mut game = fixture.save_and_reload(&game, &at);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);
    assert_eq!(counts(&game, QUEST), [1], "the objective counter survives");
    assert_eq!(coins(&game), coins_before + QUEST_COINS, "coins survive");
    assert_eq!(
        profile_xp(&game, BRAWLER),
        brawler_before + QUEST_XP,
        "Brawler XP survives"
    );
    assert_eq!(
        count_definition(&game, REWARD_ITEM),
        items_before + 1,
        "the item survives"
    );
    assert_eq!(pending_turnin(&game), None);
    let restored_state = game.state();
    let restored_bytes = saved(&game);
    assert_refused(
        &mut game,
        &at,
        &quest_end_reply(QUEST),
        now + 100,
        "replayed 49/14 after relog",
    );
    assert_eq!(game.state(), restored_state);
    assert_eq!(saved(&game), restored_bytes);
    assert_eq!(coins(&game), coins_before + QUEST_COINS);
}

#[test]
fn a_forged_or_premature_turn_in_changes_nothing() {
    let fixture = Fixture::new();
    let mut game = fixture.game();
    let at = fixture.giver_character();
    let mut now = 2_000;
    start_quest(&mut game, &at, &mut now);
    let (seed_path, seed_save) = save_bytes(&fixture, &game, &at, "refusals.json");

    // 49/14 before the defeat objective.
    assert_refused(
        &mut game,
        &at,
        &quest_end_reply(QUEST),
        now + 1,
        "premature 49/14",
    );

    // 49/2 without a server offer (the offer was consumed by the accept).
    assert_refused(
        &mut game,
        &at,
        &quest_reply(NEXT_QUEST, true),
        now + 2,
        "49/2 without an offer",
    );

    // Kill, then open the server's pending end screen.
    let wolf = fixture.wolf_character();
    let mut field = game.take_shared_fields().expect("the shared field exists");
    kill_wolf(&mut game, &mut field, &wolf, now + 3);
    now += 800;
    let dialogue = talk(&mut game, &at, GIVER, now);
    now += 10;
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "the completion Talk opens its dialogue"
    );
    assert_eq!(pending_turnin(&game), Some(QUEST));
    release_end_screen(&mut game, &at, QUEST, GIVER, END_TEXT, now);
    now += 10;

    // A turn-in for another quest neither pays nor consumes the pending screen.
    assert_refused(
        &mut game,
        &at,
        &quest_end_reply(NEXT_QUEST),
        now + 1,
        "49/14 for another quest",
    );
    assert_eq!(pending_turnin(&game), Some(QUEST));
    assert_eq!(counts(&game, QUEST), [1]);

    // The real turn-in pays once; every replay is a no-op.
    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, BRAWLER);
    finish_quest(&mut game, &at, QUEST, BRAWLER, QUEST_COINS, QUEST_XP, now);
    now += 10;
    assert_eq!(coins(&game), coins_before + QUEST_COINS);
    assert_eq!(profile_xp(&game, BRAWLER), xp_before + QUEST_XP);
    assert_refused(
        &mut game,
        &at,
        &quest_end_reply(QUEST),
        now + 1,
        "replayed 49/14",
    );
    assert_refused(
        &mut game,
        &at,
        &quest_reply(QUEST, true),
        now + 2,
        "replayed 49/2",
    );
    assert_eq!(coins(&game), coins_before + QUEST_COINS);
    assert_eq!(profile_xp(&game, BRAWLER), xp_before + QUEST_XP);
    assert_eq!(pending_turnin(&game), None);
    assert_eq!(
        fs::read(&seed_path).expect("the seed save is readable"),
        seed_save,
        "a refusal wrote the character save"
    );
}

#[test]
fn a_full_bag_refuses_the_item_reward_without_paying_coins_or_xp() {
    let fixture = Fixture::new();
    let mut game = fixture.game();
    let at = fixture.giver_character();
    let mut now = 3_000;
    start_quest(&mut game, &at, &mut now);

    let wolf = fixture.wolf_character();
    let mut field = game.take_shared_fields().expect("the shared field exists");
    kill_wolf(&mut game, &mut field, &wolf, now);
    now += 700;
    assert_eq!(counts(&game, QUEST), [1]);

    // Fill the bag through a saved-state reload, so only the turn-in sees it.
    let mut full = game.state();
    fill_bag(&mut full);
    let mut game = fixture.reload(full);
    assert_eq!(
        game.inventory_state().items.len(),
        BAG_SLOTS + 7,
        "fixture setup: the bag is at the production capacity floor"
    );

    let dialogue = talk(&mut game, &at, GIVER, now);
    now += 10;
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "the completion Talk opens its dialogue"
    );
    assert_eq!(pending_turnin(&game), Some(QUEST));
    release_end_screen(&mut game, &at, QUEST, GIVER, END_TEXT, now);
    now += 10;

    let before_state = game.state();
    let before_bytes = saved(&game);
    let before_coins = coins(&game);
    let before_xp = profile_xp(&game, BRAWLER);
    let before_item = count_definition(&game, REWARD_ITEM);
    let (save_path, before_save) = save_bytes(&fixture, &game, &at, "full-bag.json");
    let error = match game.handle_in_world(&quest_end_reply(QUEST), &at, now + 1) {
        Ok(_) => panic!("the full bag must refuse the reward"),
        Err(error) => error,
    };
    assert!(
        error.to_string().contains("bag capacity"),
        "the refusal is the production bag check: {error}"
    );
    assert_eq!(
        game.state(),
        before_state,
        "a full-bag refusal changed state"
    );
    assert_eq!(
        saved(&game),
        before_bytes,
        "a full-bag refusal changed the serialized save bytes"
    );
    assert_eq!(coins(&game), before_coins, "coins were paid");
    assert_eq!(profile_xp(&game, BRAWLER), before_xp, "job XP was paid");
    assert_eq!(
        count_definition(&game, REWARD_ITEM),
        before_item,
        "the item was granted"
    );
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(
        pending_turnin(&game),
        Some(QUEST),
        "the pending turn-in was consumed"
    );
    assert_eq!(
        fs::read(&save_path).expect("the save is readable"),
        before_save,
        "a full-bag refusal wrote the character save"
    );
}

#[test]
fn a_client_reported_result_cannot_pay() {
    let fixture = Fixture::new();
    let mut game = fixture.game();
    let at = fixture.giver_character();
    let mut now = 4_000;
    start_quest(&mut game, &at, &mut now);
    let coins_before = coins(&game);
    let brawler_before = profile_xp(&game, BRAWLER);
    let archer_before = profile_xp(&game, ARCHER);
    let item_before = count_definition(&game, REWARD_ITEM);

    // A forged "game won" lifecycle packet pair, a forged 45/7 goal claim naming the
    // defeat objective, and a forged server-only reward bundle.
    let mut won = vec![39, 0, 18, 0];
    won.extend([0u8; 12]);
    won[15] = 1;
    let mut flash = vec![39, 0, 14, 0];
    flash.extend([0u8; 15]);
    flash.extend(b"OnGameWonMsg\0");
    let mut claim = vec![45, 0, 7];
    claim.extend(1_i32.to_le_bytes());
    let mut reward = vec![50, 0, 1, 0];
    reward.extend([0u8; 32]);
    for (packet, label) in [
        (won.as_slice(), "forged 39/18 game won"),
        (flash.as_slice(), "forged 39/14 win flash"),
        (claim.as_slice(), "forged 45/7 objective claim"),
        (reward.as_slice(), "forged 50/1 reward bundle"),
    ] {
        now += 10;
        assert_refused(&mut game, &at, packet, now, label);
    }
    assert_eq!(
        counts(&game, QUEST),
        [0],
        "no client claim advanced the objective"
    );
    assert_eq!(coins(&game), coins_before, "no client claim paid coins");
    assert_eq!(profile_xp(&game, BRAWLER), brawler_before);
    assert_eq!(profile_xp(&game, ARCHER), archer_before);
    assert_eq!(count_definition(&game, REWARD_ITEM), item_before);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);

    // The same quest still advances on the server's own defeat event.
    let wolf = fixture.wolf_character();
    let mut field = game.take_shared_fields().expect("the shared field exists");
    let killed = kill_wolf(&mut game, &mut field, &wolf, now + 10);
    assert_eq!(counts(&game, QUEST), [1]);
    assert!(objective_update(&killed.replies, QUEST, 1, 1));
    assert_eq!(field.health(WOLF_GUID), Some(0));
}

#[test]
fn a_reconnect_mid_quest_keeps_progress_and_pays_once() {
    let fixture = Fixture::new();
    let mut game = fixture.game();
    let at = fixture.giver_character();
    let mut now = 5_000;
    start_quest(&mut game, &at, &mut now);

    let wolf = fixture.wolf_character();
    let mut field = game.take_shared_fields().expect("the shared field exists");
    kill_wolf(&mut game, &mut field, &wolf, now);
    now += 700;
    assert_eq!(counts(&game, QUEST), [1]);

    // Reload point 1: the objective survives, the turn-in is not open yet.
    let mut game = fixture.save_and_reload(&game, &at);
    assert_eq!(
        counts(&game, QUEST),
        [1],
        "the objective survives a reconnect"
    );
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(pending_turnin(&game), None);

    // The completion Talk opens the pending end screen for the reconnect test.
    let dialogue = talk(&mut game, &at, GIVER, now);
    now += 10;
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "the completion Talk opens its dialogue"
    );
    assert_eq!(pending_turnin(&game), Some(QUEST));
    release_end_screen(&mut game, &at, QUEST, GIVER, END_TEXT, now);
    now += 10;

    // Reload point 2: the pending turn-in exists only after the completion Talk, and the
    // relog visibility batch re-sends its 49/13 at the saved speaker.
    let mut game = fixture.save_and_reload(&game, &at);
    assert_eq!(
        pending_turnin(&game),
        Some(QUEST),
        "the pending turn-in survives a reconnect"
    );
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(counts(&game, QUEST), [1]);
    let resend = game.update_npc_visibility(&at).expect("relog visibility");
    assert!(
        resend
            .iter()
            .any(|packet| has_family(packet, 49, 13, Some(QUEST))),
        "the relog batch re-sends the held 49/13"
    );

    // Pay once, then reconnect again and replay: no second reward.
    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, BRAWLER);
    let item_before = count_definition(&game, REWARD_ITEM);
    finish_quest(&mut game, &at, QUEST, BRAWLER, QUEST_COINS, QUEST_XP, now);
    now += 10;
    assert_eq!(coins(&game), coins_before + QUEST_COINS);
    assert_eq!(profile_xp(&game, BRAWLER), xp_before + QUEST_XP);
    assert_eq!(count_definition(&game, REWARD_ITEM), item_before + 1);
    let mut game = fixture.save_and_reload(&game, &at);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);
    assert_eq!(pending_turnin(&game), None);
    assert_eq!(coins(&game), coins_before + QUEST_COINS);
    assert_eq!(profile_xp(&game, BRAWLER), xp_before + QUEST_XP);
    assert_eq!(count_definition(&game, REWARD_ITEM), item_before + 1);
    let paid_state = game.state();
    assert_refused(
        &mut game,
        &at,
        &quest_end_reply(QUEST),
        now + 100,
        "replayed 49/14 after the second reconnect",
    );
    assert_eq!(game.state(), paid_state);
}

#[test]
fn the_first_hour_loop_uses_no_dev_commands_and_leaves_the_character_free() {
    let fixture = Fixture::new();
    let mut game = fixture.game();
    let at = fixture.giver_character();
    let mut now = 6_000;

    // The session has no developer ability and the suite sends no chat/dev packet.
    for command in [
        "offer",
        "teleport",
        "here",
        "broadcast",
        "cheat",
        "give",
        "npx",
    ] {
        assert!(
            !game.may_run(command),
            "the first-hour character must not run the {command:?} developer command"
        );
    }

    start_quest(&mut game, &at, &mut now);
    let wolf = fixture.wolf_character();
    let mut field = game.take_shared_fields().expect("the shared field exists");
    kill_wolf(&mut game, &mut field, &wolf, now);
    now += 700;
    assert_eq!(counts(&game, QUEST), [1]);
    let dialogue = talk(&mut game, &at, GIVER, now);
    now += 10;
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "the completion Talk opens its dialogue"
    );
    release_end_screen(&mut game, &at, QUEST, GIVER, END_TEXT, now);
    now += 10;
    finish_quest(&mut game, &at, QUEST, BRAWLER, QUEST_COINS, QUEST_XP, now);
    now += 10;

    // Every server-owned pending/control flag is clear and the character is in a valid
    // place, so the next quest can be accepted.
    assert_eq!(pending_offer(&game), None);
    assert_eq!(pending_turnin(&game), None);
    assert!(!game.in_encounter(), "a field encounter is still active");
    assert!(!game.is_mounted(), "the character is still mounted");
    assert!(!game.is_soccer_active());
    assert!(!game.derby_active());
    assert!(at.zone_id == 0);
    assert!(at.position.iter().all(|value| value.is_finite()));
    assert!(
        game.supports_location(&at),
        "the character's saved position is a valid zone-0 place"
    );

    // A busy character would be refused before the menu; a second quest is offered and
    // accepted through the ordinary Talk path.
    offer(&mut game, &at, GIVER, NEXT_QUEST, now);
    now += 10;
    accept(&mut game, &at, NEXT_QUEST, now);
    assert_eq!(quest_status(&game, NEXT_QUEST), QuestStatus::Active);
    assert_eq!(pending_offer(&game), None);
}

#[test]
fn the_loop_uses_only_server_owned_ids_and_authoritative_hits() {
    let fixture = Fixture::new();
    let mut game = fixture.game();
    let at = fixture.giver_character();
    let mut now = 7_000;
    start_quest(&mut game, &at, &mut now);
    let clean = game.state();

    // A forged target GUID and an out-of-range attack hit nothing and credit nothing.
    let wolf = fixture.wolf_character();
    let mut field = game.take_shared_fields().expect("the shared field exists");
    let known = BTreeSet::from([WOLF_GUID]);
    let forged = game
        .handle_shared_field(
            &SharedFieldAction::Ability {
                slot: ATTACK_SLOT,
                target: WOLF_GUID + 1,
            },
            &mut field,
            &known,
            &wolf,
            now + 1,
        )
        .expect("a forged target is ignored, not an error");
    assert!(!forged.changed && forged.replies.is_empty());
    assert_eq!(field.health(WOLF_GUID), Some(WOLF_HP));
    let far = character_at_for(
        CHARACTER,
        0,
        [WOLF_CENTRE[0] + 100.0, WOLF_CENTRE[1], WOLF_CENTRE[2]],
    );
    let out_of_range = game
        .handle_shared_field(
            &SharedFieldAction::Ability {
                slot: ATTACK_SLOT,
                target: WOLF_GUID,
            },
            &mut field,
            &known,
            &far,
            now + 2,
        )
        .expect("an out-of-range attack is refused, not an error");
    assert!(!out_of_range.changed && out_of_range.replies.is_empty());
    assert_eq!(field.health(WOLF_GUID), Some(WOLF_HP));
    assert_eq!(counts(&game, QUEST), [0]);

    // A client-supplied reward bundle is not a grant.
    let mut reward = vec![50, 0, 1, 0];
    reward.extend([0u8; 32]);
    assert_refused(&mut game, &at, &reward, now + 3, "client-supplied reward");

    // Only the actor the server spawned is hittable; the defeat event credits it.
    let killed = kill_wolf(&mut game, &mut field, &wolf, now + 4);
    assert_eq!(field.health(WOLF_GUID), Some(0));
    assert_eq!(counts(&game, QUEST), [1]);
    assert!(objective_update(&killed.replies, QUEST, 1, 1));

    // A zero definition is killed by the server but cannot advance a 3156 objective.
    write_json(
        &fixture.path().join("field_enemies.json"),
        wolf_content_with_definition(0),
    );
    let mut zero = fixture.reload(clean);
    let mut zero_field = zero.take_shared_fields().expect("the shared field exists");
    let zero_kill = kill_wolf(&mut zero, &mut zero_field, &wolf, now + 100);
    assert_eq!(
        zero_field.health(WOLF_GUID),
        Some(0),
        "the server still kills it"
    );
    assert_eq!(
        counts(&zero, QUEST),
        [0],
        "a zero definition credits no quest"
    );
    assert!(
        !zero_kill
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 9, None) || has_family(packet, 49, 10, None)),
        "a zero-definition kill sends no quest objective packet"
    );

    // A negative definition is refused by the production content loader.
    write_json(
        &fixture.path().join("field_enemies.json"),
        wolf_content_with_definition(-1),
    );
    let error = Gameplay::load(fixture.path(), None)
        .err()
        .expect("a negative field definition is refused");
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
}
