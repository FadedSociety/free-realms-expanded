//! T247: the playable starting-road scavenger hunt (content-only).
//!
//! The tests drive the production in-world NPC router (`Gameplay::handle_in_world`
//! -> `gameplay/npc.rs` -> `QuestEngine`), the real `Gameplay::on_location`
//! movement path and the real accept/turn-in packets. They never call the quest
//! engine's private helpers as a shortcut.
//!
//! Client evidence (2014 client, checked with `lab/autopilot/ida_query.py`):
//! - 26/8 NPC interact constructor `008b6bc0`: u64 target + one bool.
//! - 26/10 menu select constructor `00a36f80`: two GUID dwords + i32 choice.
//! - 26/6 dialogue response `009f5890`: i32 row (301 is the quest row).
//! - 49/2 QuestReply constructor `00a86400`: quest id + accept byte.
//! - 49/14 QuestEndReply constructor `00a864c0`: one quest id.
//! - 49/1 QuestInfo reader `00c7bb60` / `00c7b7a0`: quest id, five screen dwords,
//!   bool, reward bundle, u64, float, two bools.
//! - 49/3 QuestAdd apply `00c7bd10`.
//! - 49/13 QuestEnd reader `00c7b990`: u64 actor, three dwords, reward bundle,
//!   trailing dword.
//! `visit_position` is a server-side objective; the client only receives the
//! ordinary 49/9/49/10 objective updates and the 49/8 activation, so there is no
//! new wire shape to match.

use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState, Outcome},
    npc_interaction::NpcInteraction,
    profiles::ProfileState,
    quest_engine::{CharacterEligibility, QuestEngine, QuestStatus},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const CHARACTER: u64 = 247_001;
const OTHER_CHARACTER: u64 = 247_002;
const QUEST: u32 = 3_000_200;
/// Samantha's existing Cobblestone referral; it is her first 49/1 offer, so the
/// tests accept it through the real menu before the hunt becomes the next offer.
const REFERRAL: u32 = 3_000_021;
const SAMANTHA: u64 = 9_000_000_189;
const PAPPY: u64 = 9_000_000_165;
const CARLOS: u64 = 9_000_000_042;
const MERIGOLD: u64 = 9_000_000_402;
/// A positioned Cobblestone vendor with no quest binding; a nearby non-target NPC.
const GLORIA: u64 = 9_500_000_246;
const TALK_LABEL: u32 = 3300;
const QUEST_CHOICE: i32 = 301;
const TITLE_ID: u32 = 429_342;
const HUNT_OBJECTIVE_BYTES: usize = 107;
static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

fn content_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn opening_content() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/opening-anvil-placement-v1/content")
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn eligibility() -> CharacterEligibility {
    CharacterEligibility {
        member: false,
        nonmember_quest_limit: 100,
        member_quest_limit: 100,
        ..CharacterEligibility::default()
    }
}

struct Fixture(PathBuf);

impl Fixture {
    /// A minimal gameplay package: the opening fixture's non-scene files only.
    /// The opening scene itself is omitted, so `Gameplay::on_location` advances
    /// the regional quest and the legacy checkpoint trial (far from every hunt
    /// stop) and never pays opening-story coins or discoveries during a hunt.
    fn new() -> Self {
        let source = opening_content();
        assert!(
            source.join("profiles.json").is_file() && source.join("inventory.json").is_file(),
            "T247 requires the preserved opening fixture at {}",
            source.display()
        );
        let root = std::env::temp_dir().join(format!(
            "osfr-t247-gameplay-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        // Everything except the opening scene and the opt-in campaigns that
        // refuse to load without it. This keeps the same profile, inventory,
        // combat and legacy-quest loaders while leaving `opening` None.
        for entry in fs::read_dir(&source).unwrap() {
            let path = entry.unwrap().path();
            if !path.is_file() {
                continue;
            }
            if matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some(
                    "opening.json"
                        | "opening-abilities.json"
                        | "smelting.json"
                        | "forging.json"
                        | "forging-campaign.json"
                        | "cooking.json"
                        | "cooking-campaign.json"
                        | "mining.json"
                        | "harvesting.json"
                )
            ) {
                continue;
            }
            fs::copy(&path, root.join(path.file_name().unwrap())).unwrap();
        }
        Self(root)
    }

    /// A fresh character with the Adventurer profile active, bound to the real
    /// bundled split quest content exactly as production binds it.
    fn fresh(&self, character_id: u64) -> Gameplay {
        let mut state = Gameplay::load(&self.0, None)
            .expect("minimal gameplay fixture loads")
            .state();
        state.profiles = Some(ProfileState {
            version: 1,
            active: 1,
            unlocked: BTreeSet::from([1]),
            experience: BTreeMap::new(),
        });
        self.load(character_id, Some(state))
    }

    fn load(&self, character_id: u64, state: Option<GameplayState>) -> Gameplay {
        let mut game = Gameplay::load(&self.0, state).expect("saved gameplay fixture loads");
        game.bind_npc_quest_catalogue(&content_root(), character_id)
            .expect("split quest content loads");
        game
    }

    /// The reconnect path: reload the saved `GameplayState` and rebind the catalogue.
    fn reload(&self, game: &Gameplay, character_id: u64) -> Gameplay {
        self.load(character_id, Some(game.state()))
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn npc_position(npc_id: u64) -> [f32; 3] {
    let content = content_root();
    NpcInteraction::load(&content.join("npcs"), &content.join("stores.json"))
        .expect("NPC content loads")
        .location(npc_id)
        .unwrap_or_else(|| panic!("NPC {npc_id} is a positioned content spawn"))
        .1
}

fn character(guid: u64, position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid,
        // The regional npcs/*.json zone ids are content labels inside the one
        // open-world continent; runtime movement is zone 0 (npc_interaction.rs
        // check_range and world::OPEN_WORLD_ZONE).
        zone_id: 0,
        position,
        rotation: [1.0, 0.0, 0.0],
    }
}

fn npc_interact(target: u64) -> Vec<u8> {
    // 26/8 constructor 0x008b6bc0: target GUID then the trailing bool.
    let mut packet = vec![26, 0, 8, 0];
    packet.extend(target.to_le_bytes());
    packet.push(0);
    packet
}

fn npc_select(target: u64, choice: i32) -> Vec<u8> {
    // 26/10 constructor 0x00a36f80: target GUID then i32 choice.
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(target.to_le_bytes());
    packet.extend(choice.to_le_bytes());
    packet
}

fn dialogue_choice(row: i32) -> Vec<u8> {
    // 26/6 dialogue response (constructor/reader 0x009f5890).
    let mut packet = vec![26, 0, 6, 0];
    packet.extend(row.to_le_bytes());
    packet
}

fn quest_reply(quest_id: u32, accept: bool) -> Vec<u8> {
    // 49/2 QuestReply constructor 0x00a86400: quest id + accept byte.
    let mut packet = vec![49, 0, 2, 0, 0, 0];
    packet.extend(quest_id.to_le_bytes());
    packet.push(u8::from(accept));
    packet
}

fn quest_end_reply(quest_id: u32) -> Vec<u8> {
    // 49/14 QuestEndReply constructor 0x00a864c0: one quest id.
    let mut packet = vec![49, 0, 14, 0, 0, 0];
    packet.extend(quest_id.to_le_bytes());
    packet
}

fn send(game: &mut Gameplay, character: &CharacterState, packet: &[u8], now: u64) -> Outcome {
    game.handle_in_world(packet, character, now)
        .unwrap_or_else(|error| {
            panic!(
                "packet {:?} was refused: {error}",
                &packet[..packet.len().min(4)]
            )
        })
}

fn has_family(packet: &[u8], family: u16, subtype: u16, id: Option<u32>) -> bool {
    if packet.len() < 6
        || packet[..2] != family.to_le_bytes()
        || packet[2..4] != subtype.to_le_bytes()
    {
        return false;
    }
    let Some(id) = id else { return true };
    // 49/13 names the turn-in speaker at 6..14; the other used packets carry
    // their quest id at 6..10.
    let offset = if (family, subtype) == (49, 13) { 14 } else { 6 };
    packet
        .get(offset..offset + 4)
        .is_some_and(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()) == id)
}

fn has_objective(replies: &[Vec<u8>], subtype: u16, objective_id: u32) -> bool {
    replies.iter().any(|packet| {
        packet.len() >= 14
            && packet[..6] == [49, 0, subtype as u8, 0, 0, 0]
            && packet[6..10] == QUEST.to_le_bytes()
            && packet[10..14] == objective_id.to_le_bytes()
    })
}

fn menu_option(outcome: &Outcome, label: u32) -> i32 {
    let menu = outcome
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 9, 0]))
        .expect("26/8 returns the 26/9 interaction menu");
    let count = i32::from_le_bytes(menu[13..17].try_into().unwrap());
    (0..count)
        .find_map(|index| {
            let offset = 17 + usize::try_from(index).unwrap() * 28;
            let id = i32::from_le_bytes(menu[offset..offset + 4].try_into().unwrap());
            let string_id = u32::from_le_bytes(menu[offset + 8..offset + 12].try_into().unwrap());
            (string_id == label).then_some(id)
        })
        .unwrap_or_else(|| panic!("menu label {label} is absent"))
}

fn talk(game: &mut Gameplay, at: &CharacterState, target: u64, now: &mut u64) -> Outcome {
    let menu = send(game, at, &npc_interact(target), *now);
    assert!(
        menu.replies.iter().any(|p| p.starts_with(&[26, 0, 9, 0])),
        "26/8 must return the 26/9 menu"
    );
    let choice = menu_option(&menu, TALK_LABEL);
    *now += 1;
    send(game, at, &npc_select(target, choice), *now)
}

fn choose_quest_row(game: &mut Gameplay, at: &CharacterState, now: &mut u64) -> Outcome {
    *now += 1;
    send(game, at, &dialogue_choice(QUEST_CHOICE), *now)
}

fn status(game: &Gameplay, quest_id: u32) -> QuestStatus {
    game.state().npc_quests.as_ref().unwrap().quests[&quest_id]
        .status
        .clone()
}

fn counts(game: &Gameplay, quest_id: u32) -> Vec<u32> {
    game.state().npc_quests.as_ref().unwrap().quests[&quest_id]
        .counts
        .clone()
}

fn pending_offer(game: &Gameplay) -> Option<u32> {
    game.state().npc_quests.as_ref().unwrap().pending_offer
}

fn pending_turnin(game: &Gameplay) -> Option<u32> {
    game.state().npc_quests.as_ref().unwrap().pending_turnin
}

fn coins(game: &Gameplay) -> u32 {
    game.state().inventory.coins
}

fn profile_xp(game: &Gameplay, profile: i32) -> u32 {
    game.state()
        .profiles
        .as_ref()
        .unwrap()
        .experience
        .get(&profile)
        .copied()
        .unwrap_or(0)
}

fn saved(game: &Gameplay) -> Vec<u8> {
    serde_json::to_vec(&game.state()).unwrap()
}

/// T247 cannot change the player when a request is refused.
fn assert_request_changes_nothing(
    game: &mut Gameplay,
    at: &CharacterState,
    packet: &[u8],
    now: u64,
    label: &str,
) {
    let before_state = game.state();
    let before_bytes = saved(game);
    let outcome = send(game, at, packet, now);
    assert!(!outcome.changed, "{label} changed gameplay");
    assert_eq!(game.state(), before_state, "{label} changed the state");
    assert_eq!(saved(game), before_bytes, "{label} changed the save bytes");
}

fn assert_move_changes_nothing(
    game: &mut Gameplay,
    guid: u64,
    zone_id: u32,
    position: [f32; 3],
    now: u64,
    label: &str,
) {
    let before_state = game.state();
    let before_bytes = saved(game);
    let mut at = character(guid, position);
    at.zone_id = zone_id;
    let outcome = game.on_location(&at, now).expect("movement is accepted");
    assert!(!outcome.changed, "{label} changed gameplay");
    assert_eq!(game.state(), before_state, "{label} changed the state");
    assert_eq!(saved(game), before_bytes, "{label} changed the save bytes");
}

/// Track down Samantha's existing Cobblestone referral through the real menu.
/// The greeter offers it before the hunt (her `quest_ids_given` row is first),
/// so accepting it is what makes the hunt the next offer; this is the production
/// offer order, not a test-only shortcut.
fn accept_referral(game: &mut Gameplay, samantha: &CharacterState, now: &mut u64) {
    let dialogue = talk(game, samantha, SAMANTHA, now);
    assert!(
        dialogue
            .replies
            .iter()
            .any(|p| p.starts_with(&[26, 0, 3, 0])),
        "Talk must open the 26/3 dialogue"
    );
    let offered = choose_quest_row(game, samantha, now);
    assert!(
        offered
            .replies
            .iter()
            .any(|p| has_family(p, 49, 1, Some(REFERRAL))),
        "Samantha must offer her existing Cobblestone referral"
    );
    *now += 1;
    let accepted = send(game, samantha, &quest_reply(REFERRAL, true), *now);
    assert!(
        accepted
            .replies
            .iter()
            .any(|p| has_family(p, 49, 3, Some(REFERRAL))),
        "the referral must accept through 49/3"
    );
    assert_eq!(status(game, REFERRAL), QuestStatus::Active);
    *now += 1;
}

/// Accept the referral, then obtain the hunt's 49/1 offer and return that packet.
fn offer_hunt(game: &mut Gameplay, samantha: &CharacterState, now: &mut u64) -> Outcome {
    accept_referral(game, samantha, now);
    let dialogue = talk(game, samantha, SAMANTHA, now);
    assert!(
        dialogue
            .replies
            .iter()
            .any(|p| p.starts_with(&[26, 0, 3, 0])),
        "the hunt must be offered as a dialogue choice"
    );
    let offered = choose_quest_row(game, samantha, now);
    assert!(
        offered
            .replies
            .iter()
            .any(|p| has_family(p, 49, 1, Some(QUEST))),
        "Samantha must offer quest {QUEST}"
    );
    assert_eq!(pending_offer(game), Some(QUEST));
    *now += 1;
    offered
}

fn accept_hunt(game: &mut Gameplay, samantha: &CharacterState, now: &mut u64) {
    let accepted = send(game, samantha, &quest_reply(QUEST, true), *now);
    let add = accepted
        .replies
        .iter()
        .find(|p| has_family(p, 49, 3, Some(QUEST)))
        .expect("accept sends the 49/3 quest record");
    let objective_count = i32::from_le_bytes(add[109..113].try_into().unwrap());
    assert_eq!(objective_count, 4, "the hunt has four objectives");
    let ids: Vec<u32> = (0..usize::try_from(objective_count).unwrap())
        .map(|index| {
            let offset = 113 + index * HUNT_OBJECTIVE_BYTES;
            u32::from_le_bytes(add[offset..offset + 4].try_into().unwrap())
        })
        .collect();
    assert_eq!(ids, [1, 2, 3, 4], "the objectives are in hunt order");
    assert!(
        has_objective(&accepted.replies, 8, 1),
        "accept activates the first ordered objective"
    );
    assert_eq!(status(game, QUEST), QuestStatus::Active);
    assert_eq!(counts(game, QUEST), [0, 0, 0, 0]);
    assert_eq!(pending_offer(game), None);
    *now += 1;
}

fn visit(
    game: &mut Gameplay,
    guid: u64,
    position: [f32; 3],
    now: &mut u64,
    expected: [u32; 4],
) -> Outcome {
    *now += 1;
    let outcome = game
        .on_location(&character(guid, position), *now)
        .expect("movement is accepted");
    assert!(outcome.changed, "the landmark must advance the hunt");
    assert_eq!(counts(game, QUEST), expected, "landmark counters");
    outcome
}

#[test]
fn samantha_offers_the_hunt_to_a_new_character() {
    let record =
        read_json(&content_root().join("quests/scavenger_hunt.json"))["records"][0].clone();
    assert_eq!(record["objectives"].as_array().unwrap().len(), 4);
    assert_eq!(record["rewards"][0]["coins"], json!(50));
    assert_eq!(record["rewards"][0]["job_xp"], json!({"1": 25}));

    let fixture = Fixture::new();
    let mut game = fixture.fresh(CHARACTER);
    let samantha = character(CHARACTER, npc_position(SAMANTHA));
    let mut now = 1_000;

    let offer = offer_hunt(&mut game, &samantha, &mut now);
    let offers: Vec<_> = offer
        .replies
        .iter()
        .filter(|p| has_family(p, 49, 1, Some(QUEST)))
        .collect();
    assert_eq!(offers.len(), 1, "the hunt offer appears once");
    let offer = offers[0];
    // 49/1 layout (reader 0x00c7b7a0): quest, intro, wire dword 2, name,
    // description, wire dword 5. The builder puts the stock title in 2/name/5.
    assert_eq!(u32::from_le_bytes(offer[6..10].try_into().unwrap()), QUEST);
    assert_eq!(u32::from_le_bytes(offer[10..14].try_into().unwrap()), 0);
    assert_eq!(
        u32::from_le_bytes(offer[14..18].try_into().unwrap()),
        TITLE_ID
    );
    assert_eq!(
        u32::from_le_bytes(offer[18..22].try_into().unwrap()),
        TITLE_ID
    );
    assert_eq!(
        u32::from_le_bytes(offer[22..26].try_into().unwrap()),
        0,
        "description text is not authored; it stays 0 under the hollow-text policy"
    );
    assert_eq!(pending_offer(&game), Some(QUEST));
    assert_eq!(status(&game, QUEST), QuestStatus::Offered);
    assert_eq!(counts(&game, QUEST), [0, 0, 0, 0]);

    // A second conversation before the accept must not repeat the offer.
    let held = game.state();
    let held_bytes = saved(&game);
    let dialogue = talk(&mut game, &samantha, SAMANTHA, &mut now);
    assert!(
        dialogue
            .replies
            .iter()
            .any(|p| p.starts_with(&[26, 0, 3, 0]))
    );
    let repeated = choose_quest_row(&mut game, &samantha, &mut now);
    assert!(
        !repeated
            .replies
            .iter()
            .any(|p| has_family(p, 49, 1, Some(QUEST))),
        "a pending offer must not be offered twice"
    );
    assert_eq!(game.state(), held);
    assert_eq!(saved(&game), held_bytes);

    accept_hunt(&mut game, &samantha, &mut now);
    let first = game.state();
    let first_bytes = saved(&game);

    // A second character independently resolves the same giver, referral and
    // hunt offer; the first character's state is untouched.
    let mut other = fixture.fresh(OTHER_CHARACTER);
    let other_samantha = character(OTHER_CHARACTER, npc_position(SAMANTHA));
    let mut other_now = 2_000;
    offer_hunt(&mut other, &other_samantha, &mut other_now);
    assert_eq!(pending_offer(&other), Some(QUEST));
    assert_eq!(status(&other, QUEST), QuestStatus::Offered);
    assert_eq!(
        other.state().npc_quests.as_ref().unwrap().character_id,
        OTHER_CHARACTER
    );
    assert_eq!(game.state(), first);
    assert_eq!(saved(&game), first_bytes);
}

#[test]
fn the_three_landmarks_and_return_complete_the_hunt_once() {
    let fixture = Fixture::new();
    let mut game = fixture.fresh(CHARACTER);
    let samantha = character(CHARACTER, npc_position(SAMANTHA));
    let mut now = 3_000;
    offer_hunt(&mut game, &samantha, &mut now);
    accept_hunt(&mut game, &samantha, &mut now);

    // A Talk at Samantha before the three stops neither completes the final
    // objective nor opens the end screen.
    let held = game.state();
    let held_bytes = saved(&game);
    let early = talk(&mut game, &samantha, SAMANTHA, &mut now);
    assert!(!early.replies.iter().any(|p| has_family(p, 49, 13, None)));
    let closed = choose_quest_row(&mut game, &samantha, &mut now);
    assert!(!closed.replies.iter().any(|p| has_family(p, 49, 13, None)));
    assert_eq!(game.state(), held);
    assert_eq!(saved(&game), held_bytes);
    assert_eq!(counts(&game, QUEST), [0, 0, 0, 0]);

    // Stop 1: Pappy Farnum's farm.
    let pappy = visit(
        &mut game,
        CHARACTER,
        npc_position(PAPPY),
        &mut now,
        [1, 0, 0, 0],
    );
    assert!(has_objective(&pappy.replies, 10, 1), "goal 1 completes");
    assert!(has_objective(&pappy.replies, 8, 2), "goal 2 activates");
    // Stop 2: the Cobblestone brawling ring.
    let ring = visit(
        &mut game,
        CHARACTER,
        npc_position(CARLOS),
        &mut now,
        [1, 1, 0, 0],
    );
    assert!(has_objective(&ring.replies, 10, 2), "goal 2 completes");
    assert!(has_objective(&ring.replies, 8, 3), "goal 3 activates");
    // Stop 3: Highroad Junction / Merigold.
    let highroad = visit(
        &mut game,
        CHARACTER,
        npc_position(MERIGOLD),
        &mut now,
        [1, 1, 1, 0],
    );
    assert!(has_objective(&highroad.replies, 10, 3), "goal 3 completes");
    assert!(
        has_objective(&highroad.replies, 8, 4),
        "the return Talk activates"
    );

    // Return to Samantha: the final Talk completes goal 4 and holds the end
    // screen until the client chooses the dialogue row.
    let final_talk = talk(&mut game, &samantha, SAMANTHA, &mut now);
    assert!(has_objective(&final_talk.replies, 10, 4));
    assert!(
        !final_talk
            .replies
            .iter()
            .any(|p| has_family(p, 49, 13, None)),
        "the end screen is released by the 26/6 choice"
    );
    assert_eq!(counts(&game, QUEST), [1, 1, 1, 1]);
    assert_eq!(pending_turnin(&game), Some(QUEST));
    assert_eq!(status(&game, QUEST), QuestStatus::Active);

    let end = choose_quest_row(&mut game, &samantha, &mut now);
    let screen = end
        .replies
        .iter()
        .find(|p| has_family(p, 49, 13, Some(QUEST)))
        .expect("choosing the return row opens the 49/13 end screen");
    assert_eq!(
        u64::from_le_bytes(screen[6..14].try_into().unwrap()),
        SAMANTHA,
        "the end screen names the turn-in speaker"
    );
    assert_eq!(screen.len(), 99);
    assert_eq!(i32::from_le_bytes(screen[27..31].try_into().unwrap()), 50);
    assert_eq!(i32::from_le_bytes(screen[31..35].try_into().unwrap()), 25);

    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, 1);
    let completed = send(&mut game, &samantha, &quest_end_reply(QUEST), now);
    assert!(
        completed
            .replies
            .iter()
            .any(|p| has_family(p, 49, 4, Some(QUEST)))
    );
    let count = completed
        .replies
        .iter()
        .find(|p| has_family(p, 49, 12, None))
        .expect("the completed count follows");
    assert_eq!(i32::from_le_bytes(count[6..10].try_into().unwrap()), 1);
    assert_eq!(coins(&game), coins_before + 50);
    assert_eq!(profile_xp(&game, 1), xp_before + 25);
    assert_eq!(status(&game, QUEST), QuestStatus::Completed);
    assert_eq!(pending_turnin(&game), None);

    // A repeated 49/14 (or 49/2) after the payout changes nothing.
    let paid = game.state();
    let paid_bytes = saved(&game);
    assert_request_changes_nothing(
        &mut game,
        &samantha,
        &quest_end_reply(QUEST),
        now + 1,
        "repeated turn-in",
    );
    assert_request_changes_nothing(
        &mut game,
        &samantha,
        &quest_reply(QUEST, true),
        now + 2,
        "repeated accept",
    );
    assert_eq!(game.state(), paid);
    assert_eq!(saved(&game), paid_bytes);
    assert_eq!(coins(&game), coins_before + 50);
    assert_eq!(profile_xp(&game, 1), xp_before + 25);
}

#[test]
fn a_wrong_position_or_npc_changes_nothing() {
    let fixture = Fixture::new();
    let mut game = fixture.fresh(CHARACTER);
    let samantha = character(CHARACTER, npc_position(SAMANTHA));
    let mut now = 5_000;
    offer_hunt(&mut game, &samantha, &mut now);
    accept_hunt(&mut game, &samantha, &mut now);

    // Outside the 8 m search radius, same zone and landmark height.
    let mut outside = npc_position(PAPPY);
    outside[2] += 30.0;
    assert_move_changes_nothing(&mut game, CHARACTER, 0, outside, now + 1, "outside radius");

    // Inside the copied position but in a different zone.
    assert_move_changes_nothing(
        &mut game,
        CHARACTER,
        1,
        npc_position(PAPPY),
        now + 2,
        "wrong zone",
    );

    // A nearby positioned NPC that is not a hunt target and gives no quest:
    // the whole real conversation leaves the saved state equal.
    let gloria = character(CHARACTER, npc_position(GLORIA));
    let before_state = game.state();
    let before_bytes = saved(&game);
    let dialogue = talk(&mut game, &gloria, GLORIA, &mut now);
    assert!(
        dialogue
            .replies
            .iter()
            .any(|p| p.starts_with(&[26, 0, 3, 0])),
        "the non-target Talk must reach the dialogue"
    );
    let closed = choose_quest_row(&mut game, &gloria, &mut now);
    assert!(
        !closed.replies.iter().any(|p| has_family(p, 49, 1, None)),
        "the non-target NPC must offer no quest"
    );
    assert_eq!(game.state(), before_state);
    assert_eq!(saved(&game), before_bytes);

    // Forged accept without a new offer, and a forged 49/14 without a pending
    // end screen, are both refused without a state change.
    assert_request_changes_nothing(
        &mut game,
        &samantha,
        &quest_reply(QUEST, true),
        now + 1,
        "forged 49/2",
    );
    assert_request_changes_nothing(
        &mut game,
        &samantha,
        &quest_end_reply(QUEST),
        now + 2,
        "forged 49/14",
    );

    // Out-of-range interaction is refused and mutates nothing.
    let mut far = npc_position(SAMANTHA);
    far[2] += 100.0;
    let before_state = game.state();
    let before_bytes = saved(&game);
    assert!(
        game.handle_in_world(&npc_interact(SAMANTHA), &character(CHARACTER, far), now + 3)
            .is_err(),
        "an out-of-range 26/8 must be refused"
    );
    assert_eq!(game.state(), before_state);
    assert_eq!(saved(&game), before_bytes);

    // The hunt is still Active with no progress and no payout.
    assert_eq!(status(&game, QUEST), QuestStatus::Active);
    assert_eq!(counts(&game, QUEST), [0, 0, 0, 0]);
    assert_eq!(coins(&game), 0);
    assert_eq!(profile_xp(&game, 1), 0);
}

#[test]
fn a_mid_hunt_reconnect_keeps_progress_and_pays_no_extra() {
    let fixture = Fixture::new();
    let mut game = fixture.fresh(CHARACTER);
    let samantha = character(CHARACTER, npc_position(SAMANTHA));
    let mut now = 7_000;
    offer_hunt(&mut game, &samantha, &mut now);
    accept_hunt(&mut game, &samantha, &mut now);

    visit(
        &mut game,
        CHARACTER,
        npc_position(PAPPY),
        &mut now,
        [1, 0, 0, 0],
    );
    visit(
        &mut game,
        CHARACTER,
        npc_position(CARLOS),
        &mut now,
        [1, 1, 0, 0],
    );
    let mid = saved(&game);
    assert_eq!(coins(&game), 0);
    assert_eq!(profile_xp(&game, 1), 0);

    // Disconnect after two landmarks: the counters are in the save and are
    // restored without a payout.
    game = fixture.reload(&game, CHARACTER);
    assert_eq!(saved(&game), mid, "reload changed the mid-hunt save");
    assert_eq!(status(&game, QUEST), QuestStatus::Active);
    assert_eq!(counts(&game, QUEST), [1, 1, 0, 0]);
    assert_eq!(coins(&game), 0);
    assert_eq!(profile_xp(&game, 1), 0);

    visit(
        &mut game,
        CHARACTER,
        npc_position(MERIGOLD),
        &mut now,
        [1, 1, 1, 0],
    );
    let final_talk = talk(&mut game, &samantha, SAMANTHA, &mut now);
    assert!(has_objective(&final_talk.replies, 10, 4));
    choose_quest_row(&mut game, &samantha, &mut now);
    send(&mut game, &samantha, &quest_end_reply(QUEST), now);
    assert_eq!(status(&game, QUEST), QuestStatus::Completed);
    assert_eq!(coins(&game), 50);
    assert_eq!(profile_xp(&game, 1), 25);
    let paid = saved(&game);

    // Reconnect after the turn-in does not pay again and a replayed 49/14
    // changes nothing.
    game = fixture.reload(&game, CHARACTER);
    assert_eq!(saved(&game), paid, "reload changed the paid save");
    assert_eq!(status(&game, QUEST), QuestStatus::Completed);
    assert_eq!(coins(&game), 50);
    assert_eq!(profile_xp(&game, 1), 25);
    assert_request_changes_nothing(
        &mut game,
        &samantha,
        &quest_end_reply(QUEST),
        now + 1,
        "turn-in replay after reconnect",
    );
    assert_eq!(coins(&game), 50);
    assert_eq!(profile_xp(&game, 1), 25);
}

#[test]
fn the_quest_loads_with_real_targets_and_no_collision() {
    let content = content_root();
    let quest_dir = content.join("quests");
    let mut files: Vec<PathBuf> = fs::read_dir(&quest_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    files.sort();
    assert_eq!(files.len(), 18, "the hunt is one new quest file");

    let mut seen = BTreeSet::new();
    let mut total_records = 0usize;
    for path in &files {
        let document = read_json(path);
        for record in document["records"].as_array().unwrap() {
            let id = record["id"].as_u64().unwrap() as u32;
            assert!(id > 0, "{}: quest ids are positive", path.display());
            assert!(seen.insert(id), "duplicate quest id {id}");
            total_records += 1;
        }
    }
    assert_eq!(total_records, 654, "653 existing records plus the hunt");

    let hunt_path = quest_dir.join("scavenger_hunt.json");
    let hunt = read_json(&hunt_path);
    let records = hunt["records"].as_array().unwrap();
    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record["id"], json!(QUEST));
    assert_eq!(record["title_string_id"], json!(TITLE_ID));
    assert_eq!(record["giver_npc_id"], json!(SAMANTHA));
    assert_eq!(record["prerequisite_quest_ids"], json!([]));
    assert_ne!(record["enabled"], json!(false));

    // The loader requires the quest to be listed by a jobs.json quest_ids
    // list; the giver offer itself comes from the record's giver_npc_id.
    let jobs = read_json(&content.join("jobs.json"));
    let job_refs: Vec<&str> = jobs["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|job| {
            job["quest_ids"]
                .as_array()
                .is_some_and(|ids| ids.contains(&json!(QUEST)))
        })
        .map(|job| job["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        job_refs,
        vec!["adventurer"],
        "the hunt is listed once by a job"
    );

    // No NPC content row was edited to bind the quest; the giver is the quest
    // record's field and the menu adds QuestEngine::quest_ids_for_giver.
    for path in fs::read_dir(content.join("npcs")).unwrap() {
        let path = path.unwrap().path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let document = read_json(&path);
        for npc in document["records"].as_array().unwrap() {
            for field in ["quest_ids_given", "quests_given", "quests_accepted"] {
                let bound = npc[field].as_array().is_some_and(|rows| {
                    rows.iter()
                        .any(|row| row["quest_id"] == json!(QUEST) || row == &json!(QUEST))
                });
                assert!(!bound, "{}: NPC {field} binds the hunt", path.display());
            }
        }
    }

    // The giver is spawned and drawable in the bundled NPC content.
    let samantha = npc_row("cobblestone_village.json", SAMANTHA);
    assert!(
        samantha["roles"]
            .as_array()
            .unwrap()
            .contains(&json!("quest_giver")),
        "Samantha is a quest giver"
    );
    assert!(samantha["model_id"].as_u64().unwrap() > 0);
    let position: [f32; 3] = serde_json::from_value(samantha["position"].clone()).unwrap();
    assert!(position.iter().all(|value| value.is_finite()));
    assert!(
        NpcInteraction::load(&content.join("npcs"), &content.join("stores.json"))
            .unwrap()
            .location(SAMANTHA)
            .is_some(),
        "Samantha has a positioned content spawn"
    );

    // The three stops copy the cited rows verbatim and use the open-world zone.
    let objectives = record["objectives"].as_array().unwrap();
    assert_eq!(objectives.len(), 4);
    for (index, (file, npc_id, id)) in [
        ("farnum_farmstead.json", PAPPY, 1u32),
        ("cobblestone_village.json", CARLOS, 2),
        ("highroad_junction.json", MERIGOLD, 3),
    ]
    .into_iter()
    .enumerate()
    {
        let objective = &objectives[index];
        assert_eq!(objective["kind"], json!("visit_position"));
        assert_eq!(objective["id"], json!(id));
        assert_eq!(objective["zone_id"], json!(0));
        assert_eq!(objective["radius"], json!(8.0));
        let npc = npc_row(file, npc_id);
        assert_eq!(
            objective["position"], npc["position"],
            "stop {id} must copy the {file} row position verbatim"
        );
        let position: [f32; 3] = serde_json::from_value(objective["position"].clone()).unwrap();
        assert!(position.iter().all(|value| value.is_finite()));
        let radius = objective["radius"].as_f64().unwrap();
        assert!(
            (1.0..=50.0).contains(&radius),
            "the local hunt radius stays inside its authored sanity bounds"
        );
    }
    assert_eq!(objectives[3]["kind"], json!("talk_to_npc"));
    assert_eq!(objectives[3]["npc_id"], json!(SAMANTHA));

    // Every record/objective/reward carries non-empty, correctly labelled
    // provenance; a reconstructed tag starts with (R).
    let check_sources = |value: &Value, label: &str| {
        let tags = value["source"]
            .as_array()
            .unwrap_or_else(|| panic!("{label} has provenance"));
        assert!(!tags.is_empty(), "{label} has non-empty provenance");
        for tag in tags {
            let evidence = tag["evidence"].as_str().unwrap_or("");
            assert!(!evidence.trim().is_empty(), "{label} evidence is non-empty");
            if tag["kind"] == json!("reconstructed") {
                assert!(
                    evidence.starts_with("(R)"),
                    "{label} reconstructed evidence starts with (R)"
                );
            }
        }
    };
    check_sources(record, "record");
    for (index, objective) in objectives.iter().enumerate() {
        check_sources(objective, &format!("objective {index}"));
    }
    for (index, reward) in record["rewards"].as_array().unwrap().iter().enumerate() {
        check_sources(reward, &format!("reward {index}"));
    }
    let rewards = record["rewards"].as_array().unwrap();
    assert_eq!(rewards.len(), 1);
    assert_eq!(rewards[0]["coins"], json!(50));
    assert_eq!(rewards[0]["items"], json!([]));
    assert_eq!(rewards[0]["job_xp"], json!({"1": 25}));

    // A malformed synthetic copy is a bounded load error, never a panic; the
    // unmodified copy in the same scratch directory loads as a control.
    let scratch = Scratch::new("scavenger-hunt-malformed");
    copy_split_content(&scratch.0);
    assert!(
        QuestEngine::from_split_content(&scratch.0, 1, eligibility(), None).is_ok(),
        "the control copy must load"
    );
    let path = scratch.0.join("quests/scavenger_hunt.json");
    let mut corrupted = read_json(&path);
    corrupted["records"][0]["objectives"][0]["radius"] = json!(0.0);
    fs::write(&path, serde_json::to_vec(&corrupted).unwrap()).unwrap();
    let error = match QuestEngine::from_split_content(&scratch.0, 1, eligibility(), None) {
        Ok(_) => panic!("a zero hunt radius must be a load error"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
}

fn npc_row(file: &str, npc_id: u64) -> Value {
    let document = read_json(&content_root().join("npcs").join(file));
    document["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["npc_id"] == json!(npc_id))
        .unwrap_or_else(|| panic!("{file} has NPC {npc_id}"))
        .clone()
}

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-t247-{name}-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn copy_split_content(target: &Path) {
    let content = content_root();
    for folder in ["quests", "npcs"] {
        fs::create_dir_all(target.join(folder)).unwrap();
        for entry in fs::read_dir(content.join(folder)).unwrap() {
            let path = entry.unwrap().path();
            fs::copy(&path, target.join(folder).join(path.file_name().unwrap())).unwrap();
        }
    }
    fs::copy(content.join("jobs.json"), target.join("jobs.json")).unwrap();
}
