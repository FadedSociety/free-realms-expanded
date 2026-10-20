//! T241: the two live Farnum starting quests, driven through the production paths.
//!
//! Fighting the Pack (3000009) counts five always-on Hooligan Wolves (definition 3156)
//! killed through the field Attack menu; Cleetus' Copper Collection (3000016) counts five
//! Copper Ore (item definition 2739) gathered from the Snowhill copper-vein nodes through
//! the ordinary Type8 mining round. Both are offered by Cleetus Farnum (9000000050) and
//! turned in through the real 26/8 -> 26/10 -> 26/6 -> 49/13 -> 49/14 sequence.
//!
//! Client packet layouts used here (IDA native map):
//! - 26/8 CommandPacketInteractRequest constructor `008b6bc0`: two u32 target words and a
//!   u8 trailing flag.
//! - 26/10 CommandPacketInteractionSelect constructor `00a36f80`: two u32 target words and
//!   an i32 choice.
//! - 26/6 PacketDialogResponse constructor `009f5890`: one i32 row (the quest row is 301).
//! - 49/2 QuestReply constructor `00a86400`: u32 quest id and a u8 accept flag.
//! - 49/1 QuestInfo reader `00c7bb60`; the field order is pinned by the T202 route test:
//!   quest at 6, intro at 10, title at 14/18, description at 22.
//! - 49/13 QuestEnd reader `00c7b990`: u64 actor, quest i32, end-text u32, a reserved u32,
//!   then the 69-byte reward bundle (success, coins, XP stars) and a trailing float.
//! - 49/14 QuestEndReply constructor `00a864c0`: one u32 quest id.
//! - 35/2 AddNpc body reader `00921a10` starts with the actor u64; 35/3 is the removal the
//!   hostile test below reads to prove the wolf died.

use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState, Outcome},
    harvest_nodes::GatheringNodes,
    mining::{self, Type8BoardSpec, Type8Goal, Type8Kind, Type8Output},
    profiles::ProfileState,
    quest_engine::QuestStatus,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

const CHARACTER: u64 = 241_017;
const CLEETUS: u64 = 9_000_000_050;
const ADVENTURER: i32 = 1;
const MINER: i32 = 14;
const BRAWLER: i32 = 43;
const FIGHTING_THE_PACK: u32 = 3_000_009;
const COPPER_COLLECTION: u32 = 3_000_016;
/// The authored predecessor; it stays disabled, so 3000009 is a playtest entry (R).
const YOUR_FIRST_WEAPON: u32 = 3_000_008;
const WOLF_DEFINITION: u32 = 3_156;
/// (R) The live copper-vein node emits the mined item definition, Copper Ore 2739.
const COPPER_ORE: u32 = 2_739;
const FIGHTING_TITLE: u32 = 4_420_074;
const FIGHTING_INTRO: u32 = 4_420_073;
const FIGHTING_DESCRIPTION: u32 = 4_420_075;
const FIGHTING_END_TEXT: u32 = 4_420_076;
const COPPER_TITLE: u32 = 5_100_483;
const COPPER_INTRO: u32 = 5_100_482;
const COPPER_DESCRIPTION: u32 = 5_100_484;
const COPPER_END_TEXT: u32 = 5_100_485;
const ATTACK_CHOICE: i32 = 103;
const QUEST_CHOICE: i32 = 301;
const TALK_LABEL: u32 = 3_300;
const MINING_XP_PER_WIN: u32 = mining::lifecycle::RECONSTRUCTED_MINER_XP_PER_WIN;
/// 39/6 Quit: leave the offered run without a payout.
const QUIT_FRAME_SUB: u8 = 6;
/// 26/42 closes the mining results window; the paid node hides on the next tick.
const CLOSE_GAME: [u8; 4] = [26, 0, 42, 0];
/// The authored `starting_farnum_wolves` group respawn timer.
const WOLF_RESPAWN_MS: u64 = 46_000;

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn gameplay_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/opening-anvil-placement-v1/content")
}

fn copy_dir(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
}

/// One-goal Mining18 stand-in so a gather is exactly one Copper Ore. The board is the
/// established synthetic W140/W171 fixture; node entry, payout and the quest event are the
/// production path. The shipped `mining.json` pays a multi-ore batch per win, which cannot
/// exercise the five separate gathers this quest defines.
fn synthetic_mining() -> Value {
    const LEFT: usize = 3;
    const TOP: usize = 3;
    const PLAYABLE: usize = 8;
    const BOTTOM: usize = TOP + PLAYABLE - 1;
    let side = mining::SIDE;
    let mut shape = vec![0_i32; side * side];
    for dy in 0..PLAYABLE {
        for dx in 0..PLAYABLE {
            shape[(TOP + dy) * side + LEFT + dx] = 1;
        }
    }
    shape[BOTTOM * side + LEFT] = -601;
    for x in LEFT..LEFT + PLAYABLE {
        shape[(TOP + PLAYABLE) * side + x] = 4;
    }
    json!({
        "schema_version": 1,
        "game_id": mining::MINING_GAME_ID,
        "profile_id": mining::MINER_PROFILE_ID,
        "name_id": 7910,
        "description_id": 7911,
        "icon_id": 909,
        "spec": Type8BoardSpec {
            width: side,
            height: side,
            shape,
            selectable: vec![401, 402, 403, 404],
            collectable: vec![601],
            tiles: [401, 402, 403, 404]
                .iter()
                .map(|value| Type8Kind {
                    value: *value,
                    category: 400,
                    icon_id: i32::from(*value) - 400,
                })
                .chain([Type8Kind {
                    value: 601,
                    category: 600,
                    icon_id: 1301,
                }])
                .collect(),
            goals: vec![Type8Goal {
                index: 0,
                value: 601,
                quota: 1,
                per_reward: 1,
                icon_id: 1301,
            }],
            outputs: vec![Type8Output {
                value: 601,
                definition_id: COPPER_ORE as i32,
                cluster_quantity: 1,
            }],
            duration_seconds: 180,
            seed: 241,
            wildcard_percent: 0,
            collectable_percent: 0,
            blocker_percent: 0,
            powerup_percent: 0,
            meter_id: 0,
            reconstructed_rules: true,
            rule_notice: "Synthetic T241 fixture; original Mining18 level tables unrecovered.".into(),
        },
    })
}

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let source = gameplay_dir();
        assert!(
            source.join("opening.json").is_file() && source.join("inventory.json").is_file(),
            "the opening reference content is required at {}",
            source.display()
        );
        let root = std::env::temp_dir().join(format!(
            "osfr-t241-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        copy_dir(&source, &root);
        fs::write(
            root.join(mining::CONTENT_FILE),
            synthetic_mining().to_string(),
        )
        .unwrap();
        Self { root }
    }

    fn game(&self, active: i32, unlocked: &[i32]) -> Gameplay {
        let mut state = Gameplay::load(&self.root, None)
            .expect("opening reference loads")
            .state();
        state.profiles = Some(ProfileState {
            version: 1,
            active,
            unlocked: unlocked.iter().copied().collect(),
            experience: BTreeMap::new(),
        });
        self.load(state)
    }

    /// The shipped `Fixture::load` sequence: the saved character state plus the split quest
    /// catalogue exactly as `bind_npc_quest_catalogue` binds it in production.
    fn load(&self, state: GameplayState) -> Gameplay {
        let mut game = Gameplay::load(&self.root, Some(state)).expect("saved state loads");
        game.bind_npc_quest_catalogue(&content_dir(), CHARACTER)
            .expect("split quest content loads");
        game
    }

    fn save(&self, game: &Gameplay) -> Gameplay {
        let bytes = serde_json::to_vec(&game.state()).unwrap();
        self.load(serde_json::from_slice(&bytes).unwrap())
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn read_content(relative: &str) -> Value {
    serde_json::from_slice(&fs::read(content_dir().join(relative)).unwrap())
        .unwrap_or_else(|error| panic!("{relative} is valid JSON: {error}"))
}

fn cleetus_position() -> [f32; 3] {
    let document = read_content("npcs/farnum_farmstead.json");
    let npc = document["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["npc_id"].as_u64() == Some(CLEETUS))
        .expect("Cleetus Farnum is a split NPC record");
    assert_eq!(npc["name"], "Cleetus Farnum");
    // Only data-backed placements spawn; a POI anchor is not a drawable actor.
    assert_eq!(npc["placement"], "legacy_data");
    serde_json::from_value(npc["position"].clone()).expect("the giver has a spawn position")
}

/// The always-on Hooligan Wolves (definition 3156). Empty `quest_ids` is
/// production's "present without a quest" rule. Production credits every defeat
/// by definition id, so T245's level-5 `MID-F` camp is part of this population:
/// five live wolves, not three.
fn wolves() -> Vec<(u64, [f32; 3])> {
    let document = read_content("field_enemies.json");
    document["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|enemy| {
            enemy["definition_id"].as_u64() == Some(u64::from(WOLF_DEFINITION))
                && enemy["quest_ids"]
                    .as_array()
                    .is_some_and(|ids| ids.is_empty())
        })
        .map(|enemy| {
            (
                enemy["guid"].as_u64().expect("field GUID"),
                serde_json::from_value(enemy["position"].clone()).expect("field position"),
            )
        })
        .collect()
}

fn copper_nodes() -> Vec<(u64, [f32; 3])> {
    let nodes = GatheringNodes::load(&content_dir())
        .expect("node content loads")
        .expect("node content exists");
    let mut found: Vec<_> = nodes
        .actors()
        .filter(|actor| actor.name == "Copper Vein")
        .map(|actor| (actor.guid, actor.position))
        .collect();
    found.sort_by_key(|(guid, _)| *guid);
    found
}

fn character_at(position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid: CHARACTER,
        zone_id: 0,
        position,
        rotation: [0.0, 0.0, 1.0],
    }
}

fn saved(game: &Gameplay) -> Vec<u8> {
    serde_json::to_vec(&game.state()).unwrap()
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

fn copper_ore(game: &Gameplay) -> u32 {
    game.state()
        .inventory
        .items
        .iter()
        .filter(|item| item.definition_id == COPPER_ORE as i32 && item.rental.is_none())
        .map(|item| item.count)
        .sum()
}

fn quest(game: &Gameplay, quest_id: u32) -> Option<(QuestStatus, Vec<u32>)> {
    let state = game.state();
    let progress = state.npc_quests.as_ref()?.quests.get(&quest_id)?;
    Some((progress.status.clone(), progress.counts.clone()))
}

fn count(game: &Gameplay, quest_id: u32, objective: usize) -> u32 {
    quest(game, quest_id)
        .unwrap_or_else(|| panic!("quest {quest_id} has state"))
        .1[objective]
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

fn has_family(packet: &[u8], family: u16, subtype: u16) -> bool {
    packet.len() >= 4
        && packet[..2] == family.to_le_bytes()
        && packet[2..4] == subtype.to_le_bytes()
}

fn quest_packet(outcome: &Outcome, subtype: u16, quest_id: u32) -> Option<&Vec<u8>> {
    outcome.replies.iter().find(|packet| {
        has_family(packet, 49, subtype)
            && packet.get(6..10) == Some(quest_id.to_le_bytes().as_slice())
    })
}

fn word(packet: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(packet[offset..offset + 4].try_into().unwrap())
}

// Client constructors: 26/8 `008b6bc0`, 26/10 `00a36f80`, 26/6 `009f5890`, 49/2
// `00a86400`, 49/14 `00a864c0`.
fn npc_interact(target: u64) -> Vec<u8> {
    let mut packet = vec![26, 0, 8, 0];
    packet.extend(target.to_le_bytes());
    packet.push(0);
    packet
}

fn npc_select(target: u64, choice: i32) -> Vec<u8> {
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(target.to_le_bytes());
    packet.extend(choice.to_le_bytes());
    packet
}

fn dialogue_choice(row: i32) -> Vec<u8> {
    let mut packet = vec![26, 0, 6, 0];
    packet.extend(row.to_le_bytes());
    packet
}

fn quest_reply(quest_id: u32, accept: bool) -> Vec<u8> {
    let mut packet = vec![49, 0, 2, 0, 0, 0];
    packet.extend(quest_id.to_le_bytes());
    packet.push(u8::from(accept));
    packet
}

fn quest_end_reply(quest_id: u32) -> Vec<u8> {
    let mut packet = vec![49, 0, 14, 0, 0, 0];
    packet.extend(quest_id.to_le_bytes());
    packet
}

// Client 39/5 RequestStartGame `009be7d0` writes subtype, current instance and two -1 words;
// 39/14 SendMiniGameFlashPayload carries the movie's NUL-terminated text.
fn type8_frame(sub: u8, instance: i32) -> Vec<u8> {
    let mut packet = vec![39, 0, sub];
    packet.extend(instance.to_le_bytes());
    packet.extend((-1_i32).to_le_bytes());
    packet.extend((-1_i32).to_le_bytes());
    packet
}

fn type8_payload(instance: i32, text: &str) -> Vec<u8> {
    let mut packet = type8_frame(14, instance);
    packet.extend(((text.len() + 1) as i32).to_le_bytes());
    packet.extend(text.as_bytes());
    packet.push(0);
    packet
}

fn menu_option(outcome: &Outcome, label: u32) -> i32 {
    let menu = outcome
        .replies
        .iter()
        .find(|packet| has_family(packet, 26, 9))
        .expect("26/8 returns the 26/9 interaction menu");
    let count = i32::from_le_bytes(menu[13..17].try_into().unwrap());
    assert!(count > 0, "the interaction menu has options");
    (0..count)
        .find_map(|index| {
            let offset = 17 + usize::try_from(index).unwrap() * 28;
            let id = i32::from_le_bytes(menu[offset..offset + 4].try_into().unwrap());
            let string_id = word(menu, offset + 8);
            (string_id == label).then_some(id)
        })
        .unwrap_or_else(|| panic!("menu label {label} is absent"))
}

/// The Talk option. It also completes a quest's final talk objective and holds its end
/// screen in the same production request, exactly as the client does.
fn talk(game: &mut Gameplay, character: &CharacterState, npc: u64, now: u64) -> Outcome {
    let menu = send(game, character, &npc_interact(npc), now);
    let choice = menu_option(&menu, TALK_LABEL);
    send(game, character, &npc_select(npc, choice), now + 1)
}

/// Talk, then choose the quest row: the 49/1 offer.
fn offered(
    game: &mut Gameplay,
    character: &CharacterState,
    npc: u64,
    quest_id: u32,
    now: u64,
) -> Vec<u8> {
    let dialogue = talk(game, character, npc, now);
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| has_family(packet, 26, 3)),
        "Talk opens the 26/3 dialogue"
    );
    let replies = send(game, character, &dialogue_choice(QUEST_CHOICE), now + 2);
    quest_packet(&replies, 1, quest_id)
        .unwrap_or_else(|| panic!("quest {quest_id} was not offered"))
        .clone()
}

fn assert_offer_ids(offer: &[u8], quest_id: u32, intro: u32, title: u32, description: u32) {
    assert_eq!(word(offer, 6), quest_id);
    assert_eq!(word(offer, 10), intro, "49/1 intro id");
    assert_eq!(word(offer, 14), title, "49/1 title id");
    assert_eq!(word(offer, 18), title, "49/1 title id (card)");
    assert_eq!(word(offer, 22), description, "49/1 description id");
}

fn accept(game: &mut Gameplay, character: &CharacterState, quest_id: u32, now: u64) {
    let accepted = send(game, character, &quest_reply(quest_id, true), now);
    assert!(
        quest_packet(&accepted, 3, quest_id).is_some(),
        "49/2 accept answers QuestAdd"
    );
    assert_eq!(
        quest(game, quest_id).expect("accepted quest state").0,
        QuestStatus::Active
    );
}

/// Open the 49/13 end screen through Talk and the dialogue row; it must not arrive on the
/// first request because the client draws it over an open dialog.
fn open_end_screen(
    game: &mut Gameplay,
    character: &CharacterState,
    npc: u64,
    quest_id: u32,
    now: u64,
) -> Vec<u8> {
    let dialogue = talk(game, character, npc, now);
    assert!(
        !dialogue
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 13)),
        "the end screen opens only after the dialogue choice"
    );
    let released = send(game, character, &dialogue_choice(QUEST_CHOICE), now + 2);
    let end = released
        .replies
        .iter()
        .find(|packet| {
            has_family(packet, 49, 13)
                && packet.get(14..18) == Some(quest_id.to_le_bytes().as_slice())
        })
        .unwrap_or_else(|| panic!("quest {quest_id} did not release its 49/13 end screen"));
    end.clone()
}

/// 49/13: actor u64 at 6, quest at 14, end text at 18, then the reward bundle: success at
/// 26, coins at 27, XP stars at 31 (client reader `00c7b990`, `008e7930`).
fn assert_end_screen(
    packet: &[u8],
    quest_id: u32,
    end_text: u32,
    expected_coins: u32,
    expected_xp: u32,
) {
    assert_eq!(
        u64::from_le_bytes(packet[6..14].try_into().unwrap()),
        CLEETUS,
        "the end screen names the runtime turn-in actor"
    );
    assert_eq!(word(packet, 14), quest_id);
    assert_eq!(word(packet, 18), end_text, "49/13 end-text id");
    assert_eq!(packet[26], 1, "reward bundle success flag");
    assert_eq!(
        i32::from_le_bytes(packet[27..31].try_into().unwrap()),
        expected_coins as i32,
        "49/13 displayed coins"
    );
    assert_eq!(
        i32::from_le_bytes(packet[31..35].try_into().unwrap()),
        expected_xp as i32,
        "49/13 displayed XP stars"
    );
}

fn finish_turnin(
    game: &mut Gameplay,
    character: &CharacterState,
    quest_id: u32,
    profile: i32,
    expected_coins: u32,
    expected_xp: u32,
    now: u64,
) {
    let coins_before = coins(game);
    let xp_before = profile_xp(game, profile);
    let completed = send(game, character, &quest_end_reply(quest_id), now);
    assert!(
        quest_packet(&completed, 4, quest_id).is_some(),
        "49/14 answers QuestComplete"
    );
    assert_eq!(coins(game), coins_before + expected_coins);
    assert_eq!(profile_xp(game, profile), xp_before + expected_xp);
    assert_eq!(
        quest(game, quest_id).expect("completed state").0,
        QuestStatus::Completed
    );
    assert_eq!(
        game.state().npc_quests.as_ref().unwrap().pending_turnin,
        None
    );
}

/// One field kill through the real 26/8 -> Attack menu -> 26/10 path. The packet loop is
/// bounded and stops on the removal packet or on the counted objective.
fn kill_wolf(game: &mut Gameplay, position: [f32; 3], guid: u64, now: &mut u64) -> bool {
    let character = character_at(position);
    *now += 100;
    let menu = send(game, &character, &npc_interact(guid), *now);
    assert!(
        menu.replies.iter().any(|packet| has_family(packet, 26, 9)),
        "wolf {guid} exposes the field interaction menu"
    );
    let before = count(game, FIGHTING_THE_PACK, 0);
    for _ in 0..8 {
        *now += 700;
        let attack = send(game, &character, &npc_select(guid, ATTACK_CHOICE), *now);
        if attack.replies.iter().any(|packet| {
            has_family(packet, 35, 3) && packet.get(6..14) == Some(guid.to_le_bytes().as_slice())
        }) {
            return true;
        }
        if count(game, FIGHTING_THE_PACK, 0) > before {
            return true;
        }
    }
    false
}

fn respawn_wolves(game: &mut Gameplay, now: &mut u64) {
    let observer = character_at([10_000.0, 0.0, 10_000.0]);
    *now += WOLF_RESPAWN_MS;
    game.tick(&observer, *now).expect("field respawn tick");
}

fn active_mining(game: &Gameplay) -> Option<i32> {
    game.state()
        .mining
        .as_ref()?
        .active
        .as_ref()
        .map(|run| run.instance_id)
}

fn play_mining_round(game: &mut Gameplay, character: &CharacterState, instance: i32, base: u64) {
    for (step, packet) in [
        type8_frame(5, instance),
        type8_payload(instance, "OnConnectMsg"),
        type8_payload(instance, "OnStartGameMsg"),
        type8_payload(instance, "OnStartLevelMsg"),
    ]
    .iter()
    .enumerate()
    {
        send(game, character, packet, base + 100 * step as u64);
    }
    send(
        game,
        character,
        &type8_payload(instance, "OnLevelEventMsg\t50\t3\t10"),
        base + 600,
    );
    send(
        game,
        character,
        &type8_payload(instance, "OnNextLevelMsg"),
        base + 700,
    );
}

/// One ordinary copper-vein gather: click the node, play the Mining18 round, close the
/// results, and let the tick hide the paid node.
fn gather_copper(game: &mut Gameplay, position: [f32; 3], guid: u64, now: &mut u64) -> bool {
    let character = character_at(position);
    *now += 100;
    game.tick(&character, *now).expect("node tick");
    send(game, &character, &npc_interact(guid), *now);
    let Some(instance) = active_mining(game) else {
        return false;
    };
    *now += 100;
    play_mining_round(game, &character, instance, *now);
    *now += 1_000;
    send(game, &character, &CLOSE_GAME, *now);
    *now += 100;
    game.tick(&character, *now).expect("node sync tick");
    true
}

fn assert_refused_without_change(
    game: &mut Gameplay,
    character: &CharacterState,
    packet: &[u8],
    now: u64,
    label: &str,
) {
    let before = saved(game);
    match game.handle_in_world(packet, character, now) {
        Ok(outcome) => {
            assert!(!outcome.changed, "{label} changed gameplay");
            assert!(
                !outcome.replies.iter().any(|reply| {
                    has_family(reply, 49, 4)
                        || has_family(reply, 49, 13)
                        || has_family(reply, 50, 1)
                }),
                "{label} answered with a completion or reward"
            );
        }
        Err(_) => {}
    }
    assert_eq!(saved(game), before, "{label} changed the saved state");
}

#[test]
fn cleetus_offers_both_quests_to_the_right_job() {
    let fixture = Fixture::new();
    let cleetus = cleetus_position();
    let character = character_at(cleetus);

    // Two clients at the same giver: each keeps its own offer state.
    let mut brawler = fixture.game(BRAWLER, &[ADVENTURER, MINER, BRAWLER]);
    let mut miner = fixture.game(MINER, &[ADVENTURER, MINER, BRAWLER]);

    let offer = offered(&mut brawler, &character, CLEETUS, FIGHTING_THE_PACK, 1_000);
    assert_offer_ids(
        &offer,
        FIGHTING_THE_PACK,
        FIGHTING_INTRO,
        FIGHTING_TITLE,
        FIGHTING_DESCRIPTION,
    );
    assert_eq!(
        quest(&brawler, FIGHTING_THE_PACK).unwrap().0,
        QuestStatus::Offered
    );

    let offer = offered(&mut miner, &character, CLEETUS, COPPER_COLLECTION, 1_000);
    assert_offer_ids(
        &offer,
        COPPER_COLLECTION,
        COPPER_INTRO,
        COPPER_TITLE,
        COPPER_DESCRIPTION,
    );
    assert_eq!(
        quest(&miner, COPPER_COLLECTION).unwrap().0,
        QuestStatus::Offered
    );

    // The offers are independent: neither client sees the other quest, and a third
    // client on another active job is offered neither.
    assert!(quest(&brawler, COPPER_COLLECTION).is_none());
    assert!(quest(&miner, FIGHTING_THE_PACK).is_none());
    let mut adventurer = fixture.game(ADVENTURER, &[ADVENTURER, MINER, BRAWLER]);
    let before = saved(&adventurer);
    let dialogue = talk(&mut adventurer, &character, CLEETUS, 2_000);
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| has_family(packet, 26, 3))
    );
    let replies = send(
        &mut adventurer,
        &character,
        &dialogue_choice(QUEST_CHOICE),
        2_002,
    );
    assert!(
        !replies
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1)),
        "a non-Brawler, non-Miner is offered neither live quest"
    );
    assert_eq!(saved(&adventurer), before, "the refusal changed the save");

    accept(&mut brawler, &character, FIGHTING_THE_PACK, 3_000);
    accept(&mut miner, &character, COPPER_COLLECTION, 3_000);
    assert!(quest(&brawler, COPPER_COLLECTION).is_none());
    assert!(quest(&miner, FIGHTING_THE_PACK).is_none());
}

#[test]
fn fighting_the_pack_counts_five_live_wolves_and_pays_once() {
    let fixture = Fixture::new();
    let mut game = fixture.game(BRAWLER, &[ADVENTURER, MINER, BRAWLER]);
    let cleetus = cleetus_position();
    let character = character_at(cleetus);
    let wolves = wolves();
    assert_eq!(
        wolves.len(),
        5,
        "five always-on wolves are live: three starter plus the two level-5 MID-F camp"
    );

    let offer = offered(&mut game, &character, CLEETUS, FIGHTING_THE_PACK, 1_000);
    assert_eq!(word(&offer, 14), FIGHTING_TITLE);
    accept(&mut game, &character, FIGHTING_THE_PACK, 1_100);
    assert_eq!(quest(&game, FIGHTING_THE_PACK).unwrap().1, [0, 0]);

    let mut now = 2_000;
    // All five are live at once. Production credits every 3156 defeat, so the
    // two level-5 MID-F camp wolves count and the quest completes without
    // waiting for the authored 45 s respawn: the production rule the previous
    // W102-area filter hid.
    for (index, (guid, position)) in wolves.iter().copied().enumerate() {
        assert!(
            kill_wolf(&mut game, position, guid, &mut now),
            "wolf {index} died through the production attack path"
        );
        assert_eq!(count(&game, FIGHTING_THE_PACK, 0), index as u32 + 1);
    }
    assert_eq!(count(&game, FIGHTING_THE_PACK, 0), 5);

    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, BRAWLER);
    let end = open_end_screen(&mut game, &character, CLEETUS, FIGHTING_THE_PACK, now + 100);
    assert_end_screen(&end, FIGHTING_THE_PACK, FIGHTING_END_TEXT, 18, 25);
    assert_eq!(coins(&game), coins_before, "the end screen pays nothing");
    finish_turnin(
        &mut game,
        &character,
        FIGHTING_THE_PACK,
        BRAWLER,
        18,
        25,
        now + 200,
    );
    assert_eq!(
        profile_xp(&game, BRAWLER),
        xp_before + 25,
        "the quest pays the Brawler profile once"
    );

    // Replayed 49/14 pays nothing.
    let finished = saved(&game);
    let finished_coins = coins(&game);
    assert_refused_without_change(
        &mut game,
        &character,
        &quest_end_reply(FIGHTING_THE_PACK),
        now + 300,
        "a replayed 49/14",
    );
    assert_eq!(saved(&game), finished);
    assert_eq!(coins(&game), finished_coins);

    // A sixth live kill after turn-in changes nothing.
    respawn_wolves(&mut game, &mut now);
    let after_turnin = saved(&game);
    let after_coins = coins(&game);
    let after_xp = profile_xp(&game, BRAWLER);
    let (guid, position) = wolves[0];
    assert!(
        kill_wolf(&mut game, position, guid, &mut now),
        "the respawned sixth wolf still dies"
    );
    assert_eq!(count(&game, FIGHTING_THE_PACK, 0), 5);
    assert_eq!(saved(&game), after_turnin, "a sixth kill changed the save");
    assert_eq!(coins(&game), after_coins);
    assert_eq!(profile_xp(&game, BRAWLER), after_xp);
}

#[test]
fn copper_collection_counts_five_gathers_from_the_node_path() {
    let fixture = Fixture::new();
    let mut game = fixture.game(MINER, &[ADVENTURER, MINER, BRAWLER]);
    let cleetus = cleetus_position();
    let character = character_at(cleetus);
    let nodes = copper_nodes();
    assert!(nodes.len() >= 2, "the copper pool has live spawns");

    offered(&mut game, &character, CLEETUS, COPPER_COLLECTION, 1_000);
    accept(&mut game, &character, COPPER_COLLECTION, 1_100);
    assert_eq!(quest(&game, COPPER_COLLECTION).unwrap().1, [0, 0]);

    // An abandoned round pays and counts nothing.
    let (first_guid, first_position) = nodes[0];
    let mut now = 2_000;
    let miner = character_at(first_position);
    game.tick(&miner, now).expect("node tick");
    send(&mut game, &miner, &npc_interact(first_guid), now);
    let instance = active_mining(&game).expect("the node starts a run");
    now += 100;
    send(
        &mut game,
        &miner,
        &type8_frame(QUIT_FRAME_SUB, instance),
        now,
    );
    now += 100;
    game.tick(&miner, now).expect("quit tick");
    assert_eq!(count(&game, COPPER_COLLECTION, 0), 0);
    assert_eq!(copper_ore(&game), 0, "an abandoned round pays no ore");
    assert_eq!(coins(&game), 0);

    for (index, (guid, position)) in nodes.iter().copied().take(4).enumerate() {
        assert!(
            gather_copper(&mut game, position, guid, &mut now),
            "node {index} started a round"
        );
        assert_eq!(count(&game, COPPER_COLLECTION, 0), index as u32 + 1);
        assert_eq!(copper_ore(&game), index as u32 + 1);
    }
    assert_eq!(count(&game, COPPER_COLLECTION, 0), 4);

    // Four of the five rows are placed at once; the first node grows back after 60 s.
    let (first_guid, first_position) = nodes[0];
    now += 61_000;
    game.tick(&character, now).expect("node regrow tick");
    assert!(gather_copper(
        &mut game,
        first_position,
        first_guid,
        &mut now
    ));
    assert_eq!(count(&game, COPPER_COLLECTION, 0), 5);
    assert_eq!(copper_ore(&game), 5);

    let xp_before = profile_xp(&game, MINER);
    assert_eq!(xp_before, 5 * MINING_XP_PER_WIN);
    let end = open_end_screen(&mut game, &character, CLEETUS, COPPER_COLLECTION, now + 100);
    assert_end_screen(&end, COPPER_COLLECTION, COPPER_END_TEXT, 5, 15);
    finish_turnin(
        &mut game,
        &character,
        COPPER_COLLECTION,
        MINER,
        5,
        15,
        now + 200,
    );
    assert_eq!(
        profile_xp(&game, MINER),
        xp_before + 15,
        "the quest pays the Miner profile once"
    );

    // A stale win payload for the last round cannot pay or advance anything.
    let finished = saved(&game);
    let _ = game.handle_in_world(
        &type8_payload(instance, "OnNextLevelMsg"),
        &character,
        now + 300,
    );
    assert_eq!(saved(&game), finished, "a replayed win changed the save");
    assert_refused_without_change(
        &mut game,
        &character,
        &quest_end_reply(COPPER_COLLECTION),
        now + 400,
        "a replayed 49/14",
    );
}

#[test]
fn wrong_job_wrong_target_and_replay_change_nothing() {
    let fixture = Fixture::new();
    let cleetus = cleetus_position();
    let character = character_at(cleetus);

    // A non-Brawler, non-Miner sees no offer and keeps an unchanged save.
    let mut adventurer = fixture.game(ADVENTURER, &[ADVENTURER, MINER, BRAWLER]);
    assert_refused_without_change(
        &mut adventurer,
        &character,
        &quest_reply(FIGHTING_THE_PACK, true),
        100,
        "a forged accept on the wrong job",
    );
    let before = saved(&adventurer);
    let dialogue = talk(&mut adventurer, &character, CLEETUS, 200);
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| has_family(packet, 26, 3))
    );
    let replies = send(
        &mut adventurer,
        &character,
        &dialogue_choice(QUEST_CHOICE),
        202,
    );
    assert!(
        !replies
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1))
    );
    assert_eq!(saved(&adventurer), before);

    // The disabled predecessor and an unopened end screen are both refused.
    assert_refused_without_change(
        &mut adventurer,
        &character,
        &quest_reply(YOUR_FIRST_WEAPON, true),
        300,
        "a forged 3000008 accept",
    );
    assert_refused_without_change(
        &mut adventurer,
        &character,
        &quest_end_reply(YOUR_FIRST_WEAPON),
        400,
        "a forged 3000008 turn-in",
    );
    assert_refused_without_change(
        &mut adventurer,
        &character,
        &quest_end_reply(FIGHTING_THE_PACK),
        500,
        "a forged 49/14 with no end screen",
    );

    // The real active quest is still not payable while its kill step is open, and neither
    // the menu nor the end reply reaches it from outside interaction range.
    let mut brawler = fixture.game(BRAWLER, &[ADVENTURER, MINER, BRAWLER]);
    let offer = offered(&mut brawler, &character, CLEETUS, FIGHTING_THE_PACK, 600);
    assert_eq!(word(&offer, 14), FIGHTING_TITLE);
    accept(&mut brawler, &character, FIGHTING_THE_PACK, 602);
    assert_refused_without_change(
        &mut brawler,
        &character,
        &quest_end_reply(FIGHTING_THE_PACK),
        700,
        "a turn-in before the kill step is complete",
    );

    let far = character_at([10_000.0, 0.0, 10_000.0]);
    let before = saved(&brawler);
    assert!(
        brawler
            .handle_in_world(&npc_interact(CLEETUS), &far, 800)
            .is_err(),
        "an out-of-range NPC interaction is refused"
    );
    assert_eq!(saved(&brawler), before);
    assert!(
        brawler
            .handle_in_world(&quest_end_reply(FIGHTING_THE_PACK), &far, 801)
            .is_ok(),
        "a 49/14 without a server-opened end screen is a clean refusal"
    );
    assert_eq!(saved(&brawler), before);

    // A disabled-record replay does not unlock the authored chain either.
    assert_refused_without_change(
        &mut brawler,
        &character,
        &quest_reply(YOUR_FIRST_WEAPON, true),
        900,
        "a 3000008 accept on a Brawler",
    );
}

#[test]
fn the_two_new_records_load_and_their_targets_exist() {
    let document = read_content("quests/adventurer.json");
    let records = document["records"].as_array().unwrap();
    let record = |id: u32| {
        records
            .iter()
            .find(|record| record["id"].as_u64() == Some(u64::from(id)))
            .unwrap_or_else(|| panic!("quest {id} exists"))
    };
    let sources = |value: &Value| {
        value["source"]
            .as_array()
            .is_some_and(|source| !source.is_empty())
    };
    for (id, title, coins, profile, xp) in [
        (FIGHTING_THE_PACK, FIGHTING_TITLE, 18, BRAWLER, 25),
        (COPPER_COLLECTION, COPPER_TITLE, 5, MINER, 15),
    ] {
        let quest = record(id);
        assert_eq!(quest["enabled"].as_bool(), Some(true), "quest {id} enabled");
        assert_eq!(quest["giver_npc_id"].as_u64(), Some(CLEETUS));
        assert_eq!(quest["title_string_id"].as_u64(), Some(u64::from(title)));
        assert_eq!(quest["job"].as_i64(), Some(i64::from(profile)));
        assert_eq!(quest["prerequisite_quest_ids"], json!([]));
        assert!(
            quest["objectives"].as_array().is_some_and(|goals| {
                !goals.is_empty() && goals.iter().all(|goal| sources(goal))
            })
        );
        let rewards = quest["rewards"].as_array().unwrap();
        assert!(!rewards.is_empty());
        assert_eq!(rewards[0]["coins"].as_u64(), Some(coins));
        assert_eq!(rewards[0]["items"], json!([]));
        assert_eq!(rewards[0]["job_xp"], json!({ (profile.to_string()): xp }));
        assert!(sources(&rewards[0]));
        assert!(
            quest["source"]
                .as_array()
                .is_some_and(|source| source.iter().any(|tag| {
                    tag["kind"] == "reconstructed"
                        && tag["evidence"]
                            .as_str()
                            .is_some_and(|text| text.starts_with("(R) T241:"))
                })),
            "quest {id} carries its T241 (R) decision"
        );
    }
    // The crafting records and every 3000020+ row stay out of this slice.
    for id in [3_000_006, 3_000_007, YOUR_FIRST_WEAPON] {
        assert_eq!(
            record(id)["enabled"],
            json!(false),
            "quest {id} stays hollow"
        );
    }

    // The kill target is a real always-on field population. T245's two level-5
    // MID-F camp wolves share the definition, so production counts five.
    assert_eq!(
        wolves().len(),
        5,
        "five always-on 3156 wolves: three starter plus the MID-F camp"
    );

    // The gather target exists in the Snowhill copper pool.
    let mining = read_content("mining_nodes.json");
    let pool = mining["pools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|pool| pool["key"] == "snowhill-copper")
        .expect("the snowhill-copper pool");
    assert_eq!(pool["zone_definition_id"].as_u64(), Some(13));
    assert!(
        mining["spawns"]
            .as_array()
            .unwrap()
            .iter()
            .any(|spawn| spawn["pool_key"] == "snowhill-copper")
    );

    // The giver is spawned and drawn by the production visibility loader.
    let position = cleetus_position();
    let mut visibility = osfr_tools::world::NpcVisibility::load(&content_dir())
        .expect("NPC visibility content loads");
    let drawn = visibility.update(0, position).expect("visibility update");
    assert!(
        drawn.iter().any(|packet| {
            packet
                .windows(8)
                .any(|window| u64::from_le_bytes(window.try_into().unwrap()) == CLEETUS)
        }),
        "Cleetus Farnum is drawn at his spawn"
    );
}

#[test]
fn a_mid_quest_reconnect_keeps_progress_and_pays_no_extra() {
    let fixture = Fixture::new();
    let mut game = fixture.game(BRAWLER, &[ADVENTURER, MINER, BRAWLER]);
    let cleetus = cleetus_position();
    let character = character_at(cleetus);
    let wolves = wolves();
    assert_eq!(wolves.len(), 5);

    offered(&mut game, &character, CLEETUS, FIGHTING_THE_PACK, 1_000);
    accept(&mut game, &character, FIGHTING_THE_PACK, 1_100);
    let mut now = 2_000;
    for (guid, position) in wolves.iter().copied().take(2) {
        assert!(kill_wolf(&mut game, position, guid, &mut now));
    }
    assert_eq!(quest(&game, FIGHTING_THE_PACK).unwrap().1, [2, 0]);
    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, BRAWLER);
    let save_bytes = saved(&game);

    // Disconnect and reload the same character document.
    let mut game = fixture.save(&game);
    assert_eq!(
        quest(&game, FIGHTING_THE_PACK).unwrap(),
        (QuestStatus::Active, vec![2, 0]),
        "the kill progress survives the reload"
    );
    assert_eq!(
        game.state().npc_quests.as_ref().unwrap().pending_turnin,
        None
    );
    assert_eq!(
        game.state().npc_quests.as_ref().unwrap().pending_offer,
        None
    );
    assert_eq!(coins(&game), coins_before, "the reload paid nothing");
    assert_eq!(profile_xp(&game, BRAWLER), xp_before);
    assert_eq!(
        serde_json::to_vec(&game.state()).unwrap(),
        save_bytes,
        "the reload round-trips the quest save"
    );

    // Finish the hunt after the reload; the panel still starts from the restored
    // wolves. The first two kills were credited before the reconnect, so three
    // more reach the five-goal.
    for (guid, position) in wolves.iter().copied().take(3) {
        assert!(kill_wolf(&mut game, position, guid, &mut now));
    }
    assert_eq!(count(&game, FIGHTING_THE_PACK, 0), 5);
    let end = open_end_screen(&mut game, &character, CLEETUS, FIGHTING_THE_PACK, now + 100);
    assert_end_screen(&end, FIGHTING_THE_PACK, FIGHTING_END_TEXT, 18, 25);
    finish_turnin(
        &mut game,
        &character,
        FIGHTING_THE_PACK,
        BRAWLER,
        18,
        25,
        now + 200,
    );

    // A reconnect after the turn-in does not pay again.
    let mut game = fixture.save(&game);
    assert_eq!(
        quest(&game, FIGHTING_THE_PACK).unwrap().0,
        QuestStatus::Completed
    );
    let after_turnin = saved(&game);
    let after_coins = coins(&game);
    assert_refused_without_change(
        &mut game,
        &character,
        &quest_end_reply(FIGHTING_THE_PACK),
        now + 300,
        "a turn-in replayed after reconnect",
    );
    assert_eq!(saved(&game), after_turnin);
    assert_eq!(coins(&game), after_coins);
}
