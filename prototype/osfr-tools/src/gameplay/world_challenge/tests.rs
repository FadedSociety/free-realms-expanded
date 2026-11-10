//! L937: the in-world challenge through the real packet path: the giver's 26/8 menu, the 26/10
//! choice, the client's Go (39/5), clicks (26/8), the tick, and the end. The rules themselves are
//! tested in `crate::world_challenge`.
use super::super::t013_objective_events::{game, install};
use super::*;
use crate::{
    quest_engine::ObjectiveKind,
    startup_world::{NpcSpawn, parse_add_npc},
};

const LUGABOW: u64 = 9_000_000_387;
const FELIPE: u64 = 9_000_000_086;
const ARCHER: i32 = 35;
const POSTMAN: i32 = 4;
const ARCHER_TEST: u32 = 74;
const DELIVERY: u32 = 302;
/// P-quest-games: the Postman quest games. Honey's Bixie Bud Blitz (243: collect, then return)
/// and Loryn's Biting Back (301: catch the Tricksters).
const HONEY: u64 = 9_000_000_107;
const LORYN: u64 = 9_000_000_132;
const BIXIE_BUD_BLITZ: u32 = 243;
const BITING_BACK: u32 = 301;
const T0: u64 = 1_000_000;

fn where_is(game: &Gameplay, npc: u64) -> CharacterState {
    let (_, position) = game
        .npc_interaction
        .as_ref()
        .and_then(|npcs| npcs.location(npc))
        .expect("the giver is NPC content");
    at(position)
}

fn at(position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position,
        rotation: [0.0, 0.0, 1.0],
    }
}

fn click(guid: u64) -> Vec<u8> {
    let mut packet = vec![26, 0, 8, 0];
    packet.extend(guid.to_le_bytes());
    packet.push(0);
    packet
}

fn lifecycle(sub: u8, instance: i32) -> Vec<u8> {
    let mut packet = vec![39, 0, sub];
    packet.extend(instance.to_le_bytes());
    packet.extend((-1_i32).to_le_bytes());
    packet.extend((-1_i32).to_le_bytes());
    packet
}

/// The menu row id whose label is `label`: 26/9 rows are 28 bytes from byte 17, id first and the
/// label dword at +8 (reader `00a9dca0`).
fn option(menu: &[u8], label: u32) -> i32 {
    let count = i32::from_le_bytes(menu[13..17].try_into().unwrap());
    (0..count as usize)
        .find_map(|i| {
            let row = &menu[17 + i * 28..];
            (u32::from_le_bytes(row[8..12].try_into().unwrap()) == label)
                .then(|| i32::from_le_bytes(row[..4].try_into().unwrap()))
        })
        .unwrap_or_else(|| panic!("the giver's menu has no row {label}"))
}

fn send(game: &mut Gameplay, character: &CharacterState, packet: &[u8], now: u64) -> Outcome {
    game.handle_in_world(packet, character, now)
        .expect("the packet is handled")
}

fn find<'a>(replies: &'a [Vec<u8>], prefix: &[u8]) -> Option<&'a Vec<u8>> {
    replies.iter().find(|p| p.starts_with(prefix))
}

fn position_of(replies: &[Vec<u8>], prefix: &[u8]) -> Option<usize> {
    replies.iter().position(|p| p.starts_with(prefix))
}

/// Both packets are there, `first` ahead of `second`.
fn before(replies: &[Vec<u8>], first: &[u8], second: &[u8]) -> bool {
    matches!(
        (position_of(replies, first), position_of(replies, second)),
        (Some(a), Some(b)) if a < b
    )
}

fn spawns(replies: &[Vec<u8>]) -> Vec<NpcSpawn> {
    replies
        .iter()
        .filter(|p| p.starts_with(&[35, 0, 2, 0]))
        .map(|p| parse_add_npc(p).expect("a decodable AddNpc"))
        .collect()
}

/// A player who has unlocked `job`, standing at the giver, with a quest waiting for `challenge`.
fn ready(job: i32, challenge: u32) -> Gameplay {
    let mut game = game();
    game.profiles.as_mut().unwrap().unlock(job).unwrap();
    install(
        &mut game,
        ObjectiveKind::FinishMinigame {
            minigame_id: challenge,
        },
        1,
    );
    game
}

/// Open the giver's menu, choose the challenge and return the 39/16's instance id.
fn offer(
    game: &mut Gameplay,
    giver: u64,
    name_id: u32,
    now: u64,
) -> (CharacterState, i32, Outcome) {
    let player = where_is(game, giver);
    let menu = send(game, &player, &click(giver), now);
    let menu = find(&menu.replies, &[26, 0, 9, 0])
        .expect("the giver's menu")
        .clone();
    let choice = option(&menu, name_id);
    let mut select = vec![26, 0, 10, 0];
    select.extend(giver.to_le_bytes());
    select.extend(choice.to_le_bytes());
    let out = send(game, &player, &select, now + 1);
    let info = find(&out.replies, &[39, 0, 16]).expect("the start screen (39/16)");
    let instance = i32::from_le_bytes(info[3..7].try_into().unwrap());
    (player, instance, out)
}

#[test]
fn a_use_challenge_plays_through_the_menu_pays_once_and_credits_the_quest() {
    let mut game = ready(ARCHER, ARCHER_TEST);
    let coins = game.inventory.state().coins;
    let (player, instance, offered) = offer(&mut game, LUGABOW, 22_102, T0);
    assert!(
        before(&offered.replies, &[26, 0, 4, 0], &[39, 0, 16]),
        "the dialog closes before the start screen"
    );
    assert!(
        game.activity_busy(),
        "the start screen blocks other activities"
    );

    let go = send(&mut game, &player, &lifecycle(5, instance), T0 + 100);
    let things = spawns(&go.replies);
    assert_eq!(things.len(), 6, "the wiki's six target dummies");
    assert!(
        before(&go.replies, &[35, 0, 2, 0], &[47, 0, 1]),
        "things, then the tracker row"
    );
    assert!(
        before(&go.replies, &[47, 0, 1], &[39, 0, 17]),
        "the tracker row, then Go's ack"
    );
    assert!(
        before(&go.replies, &[39, 0, 17], &[47, 0, 10]),
        "the ack, then the timer"
    );
    assert!(find(&go.replies, &[45, 0, 1]).is_some() && find(&go.replies, &[47, 0, 14]).is_some());

    let mut last = Outcome::default();
    for (n, thing) in things.iter().enumerate() {
        let at_it = at([thing.position[0], thing.position[1], thing.position[2]]);
        last = send(
            &mut game,
            &at_it,
            &click(thing.actor_id),
            T0 + 1_000 + n as u64,
        );
    }
    let won = find(&last.replies, &[39, 0, 18]).expect("the result");
    assert_eq!(won[15], 1, "won");
    assert!(before(&last.replies, &[39, 0, 68], &[39, 0, 18]));
    assert!(before(&last.replies, &[39, 0, 18], &[39, 0, 19]));
    assert!(
        find(&last.replies, &[47, 0, 11]).is_some(),
        "the timer stops"
    );
    assert!(last.changed, "the payout is a save");
    assert_eq!(game.inventory.state().coins, coins + 70, "the row's coins");
    let xp = game
        .profiles
        .as_ref()
        .unwrap()
        .state()
        .experience
        .get(&ARCHER)
        .copied();
    assert_eq!(
        xp,
        Some(41),
        "a combat job keeps the row's XP (its rate is per kill)"
    );
    let quest = game.npc_quests.as_ref().unwrap().progress(1).unwrap();
    assert_eq!(quest.counts, vec![1], "FinishMinigame(74) credited");

    // Settled: a repeat click, a tick and the automatic 39/7 change nothing and pay nothing.
    let again = send(&mut game, &player, &click(things[0].actor_id), T0 + 2_000);
    assert!(again.replies.is_empty() && !again.changed);
    game.pet_tick(&player, T0 + 500_000).unwrap_or_default();
    send(&mut game, &player, &lifecycle(7, instance), T0 + 2_100);
    assert_eq!(game.inventory.state().coins, coins + 70, "paid once");
    assert!(!game.world_challenge.is_active());
    send(&mut game, &player, &[26, 0, 42, 0], T0 + 2_200);
    assert!(
        game.world_challenge.offered.is_none(),
        "closed with the end screen"
    );
    assert!(!game.activity_busy());
}

#[test]
fn a_player_without_the_job_is_refused_before_the_start_screen() {
    let mut game = game();
    let player = where_is(&game, LUGABOW);
    let menu = send(&mut game, &player, &click(LUGABOW), T0);
    let menu = find(&menu.replies, &[26, 0, 9, 0]).unwrap().clone();
    let mut select = vec![26, 0, 10, 0];
    select.extend(LUGABOW.to_le_bytes());
    select.extend(option(&menu, 22_102).to_le_bytes());
    let refused = send(&mut game, &player, &select, T0 + 1);
    assert!(
        find(&refused.replies, &[39, 0, 16]).is_none(),
        "no start screen"
    );
    assert!(
        find(&refused.replies, &[15, 0, 3]).is_some(),
        "one chat line"
    );
    assert!(!game.world_challenge.is_active());
}

#[test]
fn running_out_of_time_loses_and_pays_nothing() {
    let mut game = ready(ARCHER, ARCHER_TEST);
    let coins = game.inventory.state().coins;
    let (player, instance, _) = offer(&mut game, LUGABOW, 22_102, T0);
    send(&mut game, &player, &lifecycle(5, instance), T0 + 100);
    let out = game
        .pet_tick(&player, T0 + 100 + 91_000)
        .unwrap_or_default();
    let over = find(&out.replies, &[39, 0, 18]).expect("the result");
    assert_eq!(over[15], 0, "lost");
    assert!(
        find(&out.replies, &[45, 0, 4]).is_some() && find(&out.replies, &[39, 0, 19]).is_some()
    );
    assert!(
        find(&out.replies, &[39, 0, 68]).is_none(),
        "no reward update"
    );
    assert_eq!(game.inventory.state().coins, coins);
    assert_eq!(
        game.npc_quests
            .as_ref()
            .unwrap()
            .progress(1)
            .unwrap()
            .counts,
        vec![0]
    );
    assert_eq!(
        find(&out.replies, &[35, 0, 3, 0]).map(|_| ()),
        Some(()),
        "the targets are taken away"
    );
}

#[test]
fn leaving_the_game_removes_the_course_without_a_verdict_or_pay() {
    let mut game = ready(ARCHER, ARCHER_TEST);
    let coins = game.inventory.state().coins;
    let (player, instance, _) = offer(&mut game, LUGABOW, 22_102, T0);
    let go = send(&mut game, &player, &lifecycle(5, instance), T0 + 100);
    let out = send(&mut game, &player, &lifecycle(6, instance), T0 + 200);
    let removed = out
        .replies
        .iter()
        .filter(|p| p.starts_with(&[35, 0, 3, 0]))
        .count();
    assert_eq!(removed, spawns(&go.replies).len());
    assert!(before(&out.replies, &[39, 0, 18], &[39, 0, 19]), "the HUD needs the verdict");
    assert_eq!(find(&out.replies, &[39, 0, 18]).unwrap()[15], 0, "lost");
    assert_eq!(game.inventory.state().coins, coins);
    assert!(!game.world_challenge.is_active());
}

#[test]
fn a_reconnect_drops_a_live_run_unpaid() {
    let mut game = ready(ARCHER, ARCHER_TEST);
    let coins = game.inventory.state().coins;
    let (player, instance, _) = offer(&mut game, LUGABOW, 22_102, T0);
    send(&mut game, &player, &lifecycle(5, instance), T0 + 100);
    assert!(game.world_challenge.is_active());
    game.reset_session();
    assert!(!game.world_challenge.is_active() && game.world_challenge.session.is_none());
    let out = game.pet_tick(&player, T0 + 500_000).unwrap_or_default();
    let ours =
        |p: &Vec<u8>| p.starts_with(&[39, 0]) || p.starts_with(&[45, 0]) || p.starts_with(&[47, 0]);
    assert!(
        !out.replies.iter().any(ours),
        "no verdict or HUD for a dropped run"
    );
    assert_eq!(game.inventory.state().coins, coins);
}

#[test]
fn pressing_go_away_from_the_giver_cancels_the_offer() {
    let mut game = ready(ARCHER, ARCHER_TEST);
    let (player, instance, _) = offer(&mut game, LUGABOW, 22_102, T0);
    let mut away = player.clone();
    away.position[0] += 200.0;
    let out = send(&mut game, &away, &lifecycle(5, instance), T0 + 100);
    assert!(
        find(&out.replies, &[35, 0, 2, 0]).is_none(),
        "nothing is laid out"
    );
    assert!(
        find(&out.replies, &[39, 0, 19]).is_some(),
        "the start screen closes"
    );
    assert!(!game.world_challenge.is_active());
}

#[test]
fn a_delivery_walks_to_the_named_people_and_pays_the_jobs_rate() {
    let mut game = ready(POSTMAN, DELIVERY);
    let coins = game.inventory.state().coins;
    let row = game
        .world_challenge
        .content
        .as_ref()
        .unwrap()
        .get(DELIVERY)
        .unwrap()
        .clone();
    let (player, instance, _) = offer(&mut game, FELIPE, row.name_id, T0);
    let go = send(&mut game, &player, &lifecycle(5, instance), T0 + 100);
    assert!(
        spawns(&go.replies).is_empty(),
        "the people are already in the world"
    );
    let mut last = Outcome::default();
    let stops = &row.stages[0].recipients;
    for (n, stop) in stops.iter().enumerate() {
        let there = where_is(&game, *stop);
        last = game
            .pet_tick(&there, T0 + 1_000 + 500 * n as u64)
            .unwrap_or_default();
    }
    assert_eq!(
        find(&last.replies, &[39, 0, 18]).expect("the result")[15],
        1
    );
    assert_eq!(game.inventory.state().coins, coins + row.coins);
    let xp = game.challenge_xp(&row);
    assert_eq!(
        game.profiles
            .as_ref()
            .unwrap()
            .state()
            .experience
            .get(&POSTMAN)
            .copied(),
        Some(xp),
        "the job's activity rate, or the row's own XP without one"
    );
    assert_eq!(
        game.npc_quests
            .as_ref()
            .unwrap()
            .progress(1)
            .unwrap()
            .counts,
        vec![1]
    );
}

#[test]
fn closing_the_start_screen_answers_on_the_leave_not_the_close() {
    let mut game = ready(ARCHER, ARCHER_TEST);
    let (player, instance, _) = offer(&mut game, LUGABOW, 22_102, T0);
    let close = send(&mut game, &player, &[26, 0, 42, 0], T0 + 100);
    assert!(find(&close.replies, &[39, 0, 19]).is_none());
    assert!(game.world_challenge.is_active(), "still offered");
    let leave = send(&mut game, &player, &lifecycle(6, instance), T0 + 200);
    assert!(find(&leave.replies, &[39, 0, 19]).is_some());
    assert!(!game.world_challenge.is_active());
}

#[test]
fn zoning_away_drops_the_offer() {
    let mut game = ready(ARCHER, ARCHER_TEST);
    let (mut player, _, _) = offer(&mut game, LUGABOW, 22_102, T0);
    player.zone_id = 7;
    let out = game.pet_tick(&player, T0 + 100).unwrap_or_default();
    assert!(find(&out.replies, &[39, 0, 19]).is_some());
    assert!(!game.world_challenge.is_active());
}

#[test]
fn a_finished_run_whose_end_screen_was_never_closed_gives_way_to_the_next_offer() {
    let mut game = ready(ARCHER, ARCHER_TEST);
    let (player, first, _) = offer(&mut game, LUGABOW, 22_102, T0);
    send(&mut game, &player, &lifecycle(5, first), T0 + 100);
    game.pet_tick(&player, T0 + 100 + 91_000);
    assert!(
        !game.world_challenge.is_active(),
        "lost, waiting for the Close click"
    );
    let (_, second, again) = offer(&mut game, LUGABOW, 22_102, T0 + 200_000);
    assert_ne!(first, second, "a new instance");
    assert!(find(&again.replies, &[39, 0, 16]).is_some());
}

/// The shipped row of `challenge`, as the player's quest and payout read it.
fn shipped_row(game: &Gameplay, challenge: u32) -> std::sync::Arc<crate::world_challenge::Challenge> {
    game.world_challenge
        .content
        .as_ref()
        .expect("world_challenges.json is shipped")
        .get(challenge)
        .expect("the row is shipped")
        .clone()
}

/// Click every thing laid out at Go, standing at each; the last click's outcome.
fn take_every_thing(game: &mut Gameplay, things: &[NpcSpawn], from: u64) -> Outcome {
    let mut last = Outcome::default();
    for (n, thing) in things.iter().enumerate() {
        let at_it = at([thing.position[0], thing.position[1], thing.position[2]]);
        last = send(game, &at_it, &click(thing.actor_id), from + n as u64);
    }
    last
}

/// P-quest-games: Postman: Bixie Buds's game, through Honey's menu. Ten buds are laid out around
/// her, each is taken by a click, and the walk back to Honey wins the run, pays the row once and
/// credits `FinishMinigame(243)`.
#[test]
fn bixie_bud_blitz_collects_the_buds_and_returns_to_honey_for_the_postman_quest() {
    let mut game = ready(POSTMAN, BIXIE_BUD_BLITZ);
    let coins = game.inventory.state().coins;
    let row = shipped_row(&game, BIXIE_BUD_BLITZ);
    assert_eq!(row.giver, HONEY, "Honey offers it");
    let (player, instance, _) = offer(&mut game, HONEY, row.name_id, T0);
    let go = send(&mut game, &player, &lifecycle(5, instance), T0 + 100);
    let buds = spawns(&go.replies);
    assert_eq!(buds.len(), row.stages[0].count as usize, "the row's buds are laid out");
    assert!(
        buds.iter().all(|bud| bud.model_id == row.stages[0].model_id),
        "every bud wears the row's model"
    );

    let last = take_every_thing(&mut game, &buds, T0 + 1_000);
    assert!(
        find(&last.replies, &[39, 0, 18]).is_none(),
        "the buds alone do not win: the return stage is still open"
    );
    assert!(game.world_challenge.is_active());

    // Back at Honey: the return stage closes on the tick, which is the win.
    let back = game.pet_tick(&player, T0 + 5_000).unwrap_or_default();
    let won = find(&back.replies, &[39, 0, 18]).expect("the result at Honey");
    assert_eq!(won[15], 1, "won");
    assert!(back.changed, "the payout is a save");
    assert_eq!(game.inventory.state().coins, coins + row.coins, "the row's coins, once");
    let quest = game.npc_quests.as_ref().unwrap().progress(1).unwrap();
    assert_eq!(quest.counts, vec![1], "FinishMinigame(243) credited");

    let _ = game.pet_tick(&player, T0 + 6_000);
    assert_eq!(game.inventory.state().coins, coins + row.coins, "paid once");
    assert!(!game.world_challenge.is_active());
}

/// P-quest-games: Postman: Biting Back's game, through Loryn's menu at the Snowhill post office.
/// Eight Tricksters stand about; clicking each (the Mail Morph forms are not modelled, (R)) wins,
/// pays the row to the Postman and credits `FinishMinigame(301)`.
#[test]
fn biting_back_catches_the_tricksters_for_the_postman_quest() {
    let mut game = ready(POSTMAN, BITING_BACK);
    let coins = game.inventory.state().coins;
    let row = shipped_row(&game, BITING_BACK);
    assert_eq!(row.giver, LORYN, "Loryn offers it");
    assert_eq!(row.profile, POSTMAN, "a Postman game");
    let (player, instance, _) = offer(&mut game, LORYN, row.name_id, T0);
    let go = send(&mut game, &player, &lifecycle(5, instance), T0 + 100);
    let tricksters = spawns(&go.replies);
    assert_eq!(tricksters.len(), row.stages[0].count as usize, "the row's Tricksters");
    assert!(
        tricksters.iter().all(|t| t.model_id == row.stages[0].model_id),
        "every Trickster wears the row's model"
    );

    let last = take_every_thing(&mut game, &tricksters, T0 + 1_000);
    let won = find(&last.replies, &[39, 0, 18]).expect("the result");
    assert_eq!(won[15], 1, "won");
    assert!(last.changed, "the payout is a save");
    assert_eq!(game.inventory.state().coins, coins + row.coins, "the row's coins");
    assert_eq!(row.coins, 50, "(R) by difficulty 2");
    // The reference campaign has no `job_xp_curve.json`, so the row's own 35 XP is paid; a shipped
    // campaign pays the Postman's 93 per round instead (L927).
    assert_eq!(
        game.profiles
            .as_ref()
            .unwrap()
            .state()
            .experience
            .get(&POSTMAN)
            .copied(),
        Some(35),
        "the row's own XP to the Postman"
    );
    let quest = game.npc_quests.as_ref().unwrap().progress(1).unwrap();
    assert_eq!(quest.counts, vec![1], "FinishMinigame(301) credited");
    assert!(!game.world_challenge.is_active());
}
