//! T467: the four Card Duelist NPCs and two card tables stand at their recovered
//! spawns, and Sam Potts teaches Card Duelist (profile 120).
//!
//! Content only: the regional rows of Sam Potts, Poe Tatters, Bry and Garrison Gold are
//! `legacy_data` placements from Sanctuary-main@10f129e (`FabledRealms.lua` spawns,
//! `Npcs.json` model and name ids), the tables are `npc_spawns_extra.json` rows, and
//! `job_trainers.json` has the profile-120 row (client 392269 'Become a Card Duelist';
//! 392273/383099 name Sam Potts as the trainer). No production Rust changed.
//!
//! Every packet goes through `Gameplay::handle_in_world`, the router the gateway session
//! calls. Client readers, unchanged by this slice: AddNpc 35/2 `0x00921a10` (u64 guid,
//! name id, model id first); 26/8 `0x008b6bc0`; 26/9 `0x00aa2360 -> 0x00aa2220 ->
//! 0x00aa1600 -> 0x00aa04d0` (count at 13, 28-byte rows, label at +8); 26/10 `0x00a36f80`
//! (guid, choice).

#[path = "support/card_duelist.rs"]
mod card_duelist;

use card_duelist::{
    BECOME_A_CARD_DUELIST, CARD_DUELIST, CHARACTER, Fixture, TALK_LABEL, active_job,
    assert_no_offer, character_at_for, content_dir, content_position, interact, learn_card_duelist,
    menu_option, npc_interact, npc_select, said, saved, seed_active_profile, send,
};
use osfr_tools::{
    account_store::AccountRole, character_store::CharacterState, chat, gameplay::Gameplay,
    gameplay::Outcome, mining, startup_world::parse_add_npc, world::NpcVisibility,
};
use std::collections::{BTreeMap, BTreeSet};

const SAM: u64 = 9_000_000_187;
const POE: u64 = 9_000_000_170;
const BRY: u64 = 9_000_000_035;
const GARRISON: u64 = 9_000_000_094;
const DUELISTS: [(u64, &str); 4] = [
    (SAM, "sanctuary.json"),
    (POE, "sanctuary.json"),
    (BRY, "lakeshore.json"),
    (GARRISON, "snowhill.json"),
];
const MINER: i32 = 14;

/// Sanctuary-main@10f129e spawns (`FabledRealms.lua:3570,3585,3762,3852,3586,3850`) and
/// `Npcs.json` ids: actor, position, heading, model, Global.Text name id. The card tables
/// draw no name: Sanctuary hides their nameplate (NameId 20704, HideNamePlate).
const SPAWNS: [(u64, [f64; 3], f64, i32, i32); 6] = [
    (
        SAM,
        [-1290.0687, -14.91131, 291.80383],
        1.08519005,
        646,
        74_093,
    ),
    (
        POE,
        [-1282.9117, -14.34194, 353.48657],
        -3.09744,
        648,
        38_111,
    ),
    (
        BRY,
        [-742.321, 1.05391, 844.33936],
        -1.71367001,
        643,
        38_107,
    ),
    (
        GARRISON,
        [192.64722, 23.345482, 353.1109],
        -0.00549800034,
        644,
        38_092,
    ),
    (
        100_000_033_356,
        [-1285.0726, -14.354, 352.8],
        0.0708599995,
        104,
        0,
    ),
    (
        100_000_033_631,
        [195.83235, 23.162956, 349.84732],
        -2.7494719,
        104,
        0,
    ),
];

fn at(npc: u64, file: &str) -> CharacterState {
    character_at_for(CHARACTER, 0, content_position(file, npc))
}

fn unlocked(game: &Gameplay) -> BTreeSet<i32> {
    game.state().profiles.expect("profiles").unlocked
}

/// Every option label of the 26/9 menu, in order (layout in the header).
fn labels(outcome: &Outcome) -> Vec<u32> {
    let menu = outcome
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 9, 0]))
        .expect("26/8 returns the 26/9 interaction menu");
    let count = u32::from_le_bytes(menu[13..17].try_into().unwrap()) as usize;
    (0..count)
        .map(|row| u32::from_le_bytes(menu[25 + row * 28..29 + row * 28].try_into().unwrap()))
        .collect()
}

/// A refusal may answer with a chat line or an error; either way the save is unchanged.
fn assert_refused(game: &mut Gameplay, who: &CharacterState, packet: &[u8], now: u64, label: &str) {
    let before = saved(game);
    let outcome = game.handle_in_world(packet, who, now);
    assert!(!outcome.is_ok_and(|outcome| outcome.changed), "{label}");
    assert_eq!(saved(game), before, "{label} changed the save");
}

#[test]
fn card_duelist_npcs_are_drawn_at_their_recovered_spawns() {
    let mut visibility = NpcVisibility::load(&content_dir()).expect("NPC visibility loads");
    let spawned: BTreeMap<u64, [f32; 3]> = visibility
        .teleport_targets(0)
        .into_iter()
        .map(|target| (target.guid, target.position))
        .collect();
    let mut added = BTreeMap::new();
    for (actor, position, ..) in SPAWNS {
        let position = position.map(|value| value as f32);
        assert_eq!(
            spawned.get(&actor),
            Some(&position),
            "{actor}: in the production spawn list at its recovered spawn"
        );
        for packet in visibility.update(0, position).unwrap() {
            if let Ok(npc) = parse_add_npc(&packet) {
                added.insert(npc.actor_id, npc);
            }
        }
    }
    for (actor, position, heading, model, name) in SPAWNS {
        let npc = added
            .get(&actor)
            .unwrap_or_else(|| panic!("{actor}: AddNpc for a player at the spawn"));
        assert_eq!(npc.position[..3], position.map(|value| value as f32));
        let heading = heading as f32;
        assert_eq!(npc.rotation, [heading.sin(), 0.0, heading.cos(), 0.0]);
        assert_eq!((npc.model_id, npc.name_string_id), (model, name), "{actor}");
        // The four NPCs resolve to their content rows; a card table is scenery.
        let key = (name != 0).then_some(actor);
        assert_eq!(visibility.content_key(actor), key, "{actor}");
    }
}

#[test]
fn sam_offers_become_a_card_duelist_through_the_real_menu() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 1_000;
    for (npc, file) in DUELISTS {
        let at = at(npc, file);
        let menu = interact(&mut game, &at, npc, now);
        let expected = if npc == SAM {
            vec![TALK_LABEL, BECOME_A_CARD_DUELIST]
        } else {
            vec![TALK_LABEL]
        };
        assert_eq!(labels(&menu), expected, "{npc}: only Sam teaches the job");
        // All 63 Card Duelist quests stay disabled: Talk offers nothing.
        assert_no_offer(&mut game, &at, npc, &mut now, "Card Duelist NPC");
    }
}

/// The learn unlocks profile 120 and nothing else. As for every trainer row it does not
/// switch jobs (`gameplay/jobs.rs::learn_job`), and profile 120 has no starter outfit
/// (Sanctuary `Profiles.json` Id 120 `DefaultItems: []`), so nothing is granted.
#[test]
fn learning_card_duelist_unlocks_profile_120_once() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let sam = at(SAM, "sanctuary.json");
    let before = game.state();
    assert!(!unlocked(&game).contains(&CARD_DUELIST));

    let learned = learn_card_duelist(&mut game, &sam, 1_000);
    assert!(learned.changed && said(&learned, "You learned the Card Duelist job"));
    assert!(said(&learned, "starter outfit is unavailable"));
    let mut expected = before.profiles.clone().unwrap().unlocked;
    expected.insert(CARD_DUELIST);
    assert_eq!(unlocked(&game), expected, "only Card Duelist is unlocked");
    assert_eq!(active_job(&game), before.profiles.unwrap().active);
    assert_eq!(
        game.state().inventory,
        before.inventory,
        "no outfit granted"
    );

    let learned_save = saved(&game);
    let again = learn_card_duelist(&mut game, &sam, 2_000);
    assert!(!again.changed && said(&again, "already know"));
    assert_eq!(
        saved(&game),
        learned_save,
        "a second lesson changed the save"
    );

    let mut game = fixture.reload_serialized(&game);
    assert_eq!(saved(&game), learned_save, "reload changed the save");
    let again = learn_card_duelist(&mut game, &sam, 3_000);
    assert!(!again.changed && said(&again, "already know"));
    assert_eq!(saved(&game), learned_save);
}

#[test]
fn card_duelist_refusals_change_nothing() {
    let fixture = Fixture::new();
    mining::write_content(fixture.path(), mining::MINING_GAME_ID).unwrap();
    let sam = at(SAM, "sanctuary.json");
    let [x, y, z] = sam.position;
    let far = character_at_for(CHARACTER, 0, [x + 500.0, y, z]);
    let mut game = fixture.game(None);

    assert_refused(&mut game, &far, &npc_interact(SAM), 1_000, "far 26/8");
    let menu = interact(&mut game, &sam, SAM, 1_000);
    let choice = menu_option(&menu, BECOME_A_CARD_DUELIST);
    assert_refused(
        &mut game,
        &far,
        &npc_select(SAM, choice),
        1_001,
        "far 26/10",
    );
    let forged = npc_select(SAM, choice + 50);
    assert_refused(&mut game, &sam, &forged, 1_002, "unoffered 26/10");
    let unknown = npc_select(9_999_999_999, choice);
    assert_refused(&mut game, &sam, &unknown, 1_003, "unknown target");
    assert!(!unlocked(&game).contains(&CARD_DUELIST));
    // The refusals left the open menu usable.
    let learned = send(&mut game, &sam, &npc_select(SAM, choice), 1_004);
    assert!(learned.changed && unlocked(&game).contains(&CARD_DUELIST));

    // Busy: a live Mining run closes the trainer, even a menu opened before it.
    let mut state = fixture.game(None).state();
    seed_active_profile(&mut state, MINER, &[]);
    let mut game = fixture.game(Some(state));
    game.set_dev_tools(true);
    game.set_role(AccountRole::Admin);
    let menu = interact(&mut game, &sam, SAM, 2_000);
    let choice = menu_option(&menu, BECOME_A_CARD_DUELIST);
    send(&mut game, &sam, &chat::local_request("realm mine"), 2_001);
    assert!(
        game.state()
            .mining
            .is_some_and(|mining| mining.active.is_some()),
        "the Mining run is active"
    );
    assert_refused(&mut game, &sam, &npc_interact(SAM), 2_002, "busy 26/8");
    assert_refused(
        &mut game,
        &sam,
        &npc_select(SAM, choice),
        2_003,
        "busy 26/10",
    );
    assert!(!unlocked(&game).contains(&CARD_DUELIST));
    send(&mut game, &sam, &chat::local_request("realm exit"), 2_004);
    let learned = learn_card_duelist(&mut game, &sam, 2_005);
    assert!(learned.changed && unlocked(&game).contains(&CARD_DUELIST));
}

#[test]
fn two_players_learn_independently() {
    const OTHER: u64 = CHARACTER + 1;
    let fixture = Fixture::new();
    let mut first = fixture.game(None);
    let mut second = fixture.game_as(OTHER, None);
    let one = at(SAM, "sanctuary.json");
    let two = character_at_for(OTHER, 0, one.position);

    let menu = interact(&mut first, &one, SAM, 1_000);
    let choice = menu_option(&menu, BECOME_A_CARD_DUELIST);
    // The first player's open menu is not the second player's.
    let borrowed = npc_select(SAM, choice);
    assert_refused(&mut second, &two, &borrowed, 1_001, "another player's menu");
    send(&mut first, &one, &npc_select(SAM, choice), 1_002);
    assert!(unlocked(&first).contains(&CARD_DUELIST));
    assert!(!unlocked(&second).contains(&CARD_DUELIST));

    let first_save = saved(&first);
    let learned = learn_card_duelist(&mut second, &two, 1_010);
    assert!(learned.changed && unlocked(&second).contains(&CARD_DUELIST));
    assert_eq!(
        saved(&first),
        first_save,
        "the second lesson touched the first"
    );
}
