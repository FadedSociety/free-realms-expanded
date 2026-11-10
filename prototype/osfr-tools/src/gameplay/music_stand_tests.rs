//! Feeling the Beat! at a music stand, through the real packets: 26/8 click, 26/9 menu, 26/10
//! pick, 39/16 offer, 39/5 Go, the movie's 39/14 texts and the 39/18 result. The quest is put on
//! the player through the engine; everything after that is the client's own path.
use super::*;

/// The Wugachug stand's drawn actor (`npc_spawns_extra.json`) and the content key it answers as.
const STAND: u64 = 100_000_001_667;
const STAND_KEY: u64 = 9_500_003_432;
const STAND_AT: [f32; 3] = [75.55552, 24.75366, -404.7428];
/// The Snowhill stand has no quest row: it answers under its own guid.
const SNOWHILL_STAND: u64 = 100_000_003_570;
const SNOWHILL_AT: [f32; 3] = [332.6411, 27.24397, 483.7185];
const WOOTMOOT: u64 = 9_500_001_005;
/// Ready to Rock: click the stand, win on novice (201), return to Wootmoot.
const READY_TO_ROCK: u32 = 3_954_977;
const SONG_NAMES: [u32; 5] = [388_938, 388_939, 388_940, 388_941, 388_942];

fn shipped() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn game_at(position: [f32; 3]) -> (Gameplay, CharacterState) {
    let mut game = Gameplay::load(&shipped(), None).unwrap();
    game.bind_npc_quest_catalogue(&shipped(), 1).unwrap();
    let character = CharacterState {
        guid: 1,
        zone_id: 0,
        position,
        rotation: [1.0, 0.0, 0.0],
    };
    game.update_npc_visibility(&character).unwrap();
    (game, character)
}

fn click(target: u64) -> Vec<u8> {
    let mut packet = vec![26, 0, 8, 0];
    packet.extend(target.to_le_bytes());
    packet.push(0);
    packet
}

fn pick(target: u64, option: i32) -> Vec<u8> {
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(target.to_le_bytes());
    packet.extend(option.to_le_bytes());
    packet
}

/// The 26/9 menu's rows: option id and label, 28 bytes each from offset 17.
fn menu_rows(out: &Outcome) -> Vec<(i32, u32)> {
    let menu = (out.replies.iter())
        .find(|packet| packet.starts_with(&[26, 0, 9, 0]))
        .expect("the 26/9 menu");
    let count = i32::from_le_bytes(menu[13..17].try_into().unwrap());
    (0..usize::try_from(count).unwrap())
        .map(|index| {
            let at = 17 + index * 28;
            (
                i32::from_le_bytes(menu[at..at + 4].try_into().unwrap()),
                u32::from_le_bytes(menu[at + 8..at + 12].try_into().unwrap()),
            )
        })
        .collect()
}

/// The 39/16 offer in `out`: its run instance and game id.
fn offered(out: &Outcome) -> Option<(i32, i32)> {
    let offer = (out.replies.iter()).find(|packet| packet.starts_with(&[39, 0, 16]))?;
    let word = |at: usize| i32::from_le_bytes(offer[at..at + 4].try_into().unwrap());
    Some((word(3), word(11)))
}

fn movie_text(instance: i32, text: &str) -> Vec<u8> {
    crate::music_game::payload_packet(instance, -1, text).unwrap()
}

/// Click the stand and pick the song named `label`; the pick's outcome.
fn pick_song(
    game: &mut Gameplay,
    at: &CharacterState,
    stand: u64,
    label: u32,
    now: u64,
) -> Outcome {
    let menu = game.handle_in_world(&click(stand), at, now).unwrap();
    let rows = menu_rows(&menu);
    assert_eq!(
        rows.iter().map(|row| row.1).collect::<Vec<_>>(),
        SONG_NAMES,
        "a stand lists the five songs and nothing else"
    );
    let option = rows.iter().find(|row| row.1 == label).unwrap().0;
    game.handle_in_world(&pick(stand, option), at, now + 10)
        .unwrap()
}

/// Go, the data request and a win reported `song_ms` after the row; the win's outcome.
fn play_to_a_win(
    game: &mut Gameplay,
    at: &CharacterState,
    instance: i32,
    total: u32,
    start: u64,
    song_ms: u64,
) -> Outcome {
    let mut go = vec![39, 0, 5];
    for word in [instance, -1, -1] {
        go.extend(word.to_le_bytes());
    }
    let started = game.handle_in_world(&go, at, start).unwrap();
    assert!(
        (started.replies.iter()).any(|packet| packet.windows(14).any(|w| w == b"game_music.gfx")),
        "the music movie launches"
    );
    let data = game
        .handle_in_world(
            &movie_text(instance, "FRServer_ReadyForGameData"),
            at,
            start + 100,
        )
        .unwrap();
    assert_eq!(data.replies.len(), 3, "game, level row, data complete");
    let won = format!("FRServer_GameWon\t{total}\t0");
    let out = game
        .handle_in_world(&movie_text(instance, &won), at, start + 200 + song_ms)
        .unwrap();
    game.handle_in_world(
        &movie_text(instance, "FRServer_GameOver"),
        at,
        start + 300 + song_ms,
    )
    .unwrap();
    game.handle_in_world(&[26, 0, 42, 0], at, start + 400 + song_ms)
        .unwrap();
    out
}

fn counts(game: &Gameplay, quest: u32) -> Vec<u32> {
    let engine = game.npc_quests.as_ref().unwrap();
    engine.progress(quest).unwrap().counts.clone()
}

fn coins(game: &Gameplay) -> u32 {
    game.state().inventory.coins
}

/// The stand with no quest row offers every song, Very Hard (205) among them, and a won Very
/// Hard song pays its five coins once.
#[test]
fn a_music_stand_offers_every_song() {
    let (mut game, at) = game_at(SNOWHILL_AT);
    let picked = pick_song(&mut game, &at, SNOWHILL_STAND, 388_942, 1_000);
    let (instance, game_id) = offered(&picked).expect("the pick offers the song");
    assert_eq!(game_id, 205);
    // The open song is an occupation: another click offers nothing new.
    let menu = game.handle_in_world(&click(SNOWHILL_STAND), &at, 1_100);
    assert!(!menu.is_ok_and(|out| offered(&out).is_some()));
    let before = coins(&game);
    let won = play_to_a_win(&mut game, &at, instance, 106_000, 2_000, 58_000);
    let result = (won.replies.iter())
        .find(|packet| packet.starts_with(&[39, 0, 18]))
        .expect("the result screen");
    assert_eq!(result[15], 1, "won");
    assert_eq!(coins(&game), before + 5);
    // A new song can be picked after the close.
    let again = pick_song(&mut game, &at, SNOWHILL_STAND, 388_938, 70_000);
    assert_eq!(offered(&again).map(|offer| offer.1), Some(201));
}

/// Ready to Rock: picking a song at the stand is the quest's "Click on the music stand", and only
/// the difficulty the quest names counts as its win.
#[test]
fn a_pick_is_the_quests_click_and_only_its_difficulty_credits() {
    let (mut game, at) = game_at(STAND_AT);
    let engine = game.npc_quests.as_mut().unwrap();
    engine.offer(READY_TO_ROCK, WOOTMOOT);
    engine.accept(READY_TO_ROCK);
    assert_eq!(counts(&game, READY_TO_ROCK), [0, 0, 0]);
    assert_eq!(
        game.npc_visibility.as_ref().unwrap().content_key(STAND),
        Some(STAND_KEY)
    );

    // Easy (202) is not what Wootmoot asked for: the click counts, the win does not.
    let picked = pick_song(&mut game, &at, STAND, 388_939, 1_000);
    let (instance, game_id) = offered(&picked).expect("the pick offers the song");
    assert_eq!(game_id, 202);
    assert_eq!(
        counts(&game, READY_TO_ROCK),
        [1, 0, 0],
        "the pick is the click"
    );
    play_to_a_win(&mut game, &at, instance, 1_000, 2_000, 40_000);
    assert_eq!(
        counts(&game, READY_TO_ROCK),
        [1, 0, 0],
        "Easy is not novice"
    );

    // Very Easy (201) is.
    let picked = pick_song(&mut game, &at, STAND, 388_938, 50_000);
    let (instance, game_id) = offered(&picked).expect("a new song after the close");
    assert_eq!(game_id, 201);
    play_to_a_win(&mut game, &at, instance, 1_000, 51_000, 37_000);
    assert_eq!(counts(&game, READY_TO_ROCK), [1, 1, 0], "the novice win");
}
