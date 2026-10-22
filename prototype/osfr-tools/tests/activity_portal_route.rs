//! W150 patch proof: the `gameplay.rs` wiring of `activity_portal`. Drop this file in as
//! `tests/activity_portal_route.rs` AFTER `apply_gameplay_patch.py` is applied; before the patch
//! the Refresh/Play/world-entry assertions fail (family 167 is ignored).
//!
//! It drives the real `Gameplay` as an ordinary player (no dev tools, role Player) with raw inner
//! packets. Needs the private opening content like `tests/warpstone_gameplay.rs`
//! (`OSFR_ACTIVITY_CONTENT` overrides the default directory).
use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    chat,
    gameplay::{Gameplay, Outcome},
};
use std::path::PathBuf;

#[path = "common/field_battle_fixture.rs"]
mod field_battle_fixture;
use field_battle_fixture::{battles, decode_list};

const LAKESHORE: [f32; 3] = [-733.952, 9.09, 945.973];
const BATTLE: u32 = 96;
/// Brawler, the combat job the opening gives every player.
const COMBAT_JOB: i32 = 43;

fn content() -> PathBuf {
    std::env::var_os("OSFR_ACTIVITY_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/rust-opening-cooking-original-assets")
        })
}

fn character() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: LAKESHORE,
        rotation: [0.0, 0.0, 1.0],
    }
}

/// An ordinary player: no dev tools, default role.
fn game() -> Gameplay {
    Gameplay::load(&content(), None).unwrap()
}

fn play(activity: u32) -> Vec<u8> {
    let mut inner = vec![167, 0, 1, 2];
    inner.extend(activity.to_le_bytes());
    inner.extend(0u32.to_le_bytes());
    inner
}

const REFRESH: [u8; 4] = [167, 0, 1, 4];

/// The class window's ordinary request (26/13), as an ordinary player sends it.
fn switch_to_combat(game: &mut Gameplay) {
    let mut inner = vec![26, 0, 13, 0];
    inner.extend(COMBAT_JOB.to_le_bytes());
    let out = game.handle_in_world(&inner, &character(), 600).unwrap();
    assert!(out.changed, "class switch accepted");
}

/// CanPlayerJoin of the first row (header 4 + key 4 + count 4 + ten dwords).
fn first_can_join(list: &[u8]) -> u8 {
    list[12 + 40]
}

fn lists(out: &Outcome) -> Vec<&Vec<u8>> {
    out.replies
        .iter()
        .filter(|reply| reply.starts_with(&[167, 0, 1, 1]))
        .collect()
}

fn ids(list: &[u8]) -> Vec<u32> {
    // header 4 + entry type 4 + count 4, then rows; ids are only readable for the first row here.
    let count = u32::from_le_bytes(list[8..12].try_into().unwrap()) as usize;
    assert!(count >= 1);
    vec![u32::from_le_bytes(list[12..16].try_into().unwrap())]
}

fn chat_text(out: &Outcome) -> String {
    out.replies
        .iter()
        .filter(|reply| reply.starts_with(&[15, 0, 3, 0]))
        .map(|reply| {
            let length = u32::from_le_bytes(reply[4..8].try_into().unwrap()) as usize;
            String::from_utf8_lossy(&reply[8..8 + length]).into_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn world_entry_and_refresh_publish_the_same_populated_list() {
    let mut game = game();
    let entry = game.initialize_world(&character(), 500).unwrap();
    let published = lists(&entry);
    assert_eq!(published.len(), 1, "one 167/1/1 at world entry");
    assert_eq!(ids(published[0]), [BATTLE]);
    assert_eq!(
        first_can_join(published[0]),
        0,
        "a non-combat job cannot join"
    );

    let refreshed = game.handle_in_world(&REFRESH, &character(), 1_000).unwrap();
    assert_eq!(lists(&refreshed), published, "refresh re-sends the list");
    assert!(!refreshed.changed && refreshed.zoning.is_none());

    // The client asks again after a class change (009392c0); the answer now says Join.
    switch_to_combat(&mut game);
    let after = game.handle_in_world(&REFRESH, &character(), 1_500).unwrap();
    assert_eq!(first_can_join(lists(&after)[0]), 1);
}

/// T399: Games > Battles opens the finder on `Activities.Category10` (client
/// `ActivityCategories.txt` row 10, "Battles"), and the client files each row under its
/// Category dword (`00cea720`). Every shipped battle must land there with its name and
/// icon, and a combat job gets PLAY! for every battle without a level floor.
#[test]
fn world_entry_lists_every_battle_under_battles() {
    const BATTLES_CATEGORY: u32 = 10;
    let mut game = game();
    let entry = game.initialize_world(&character(), 500).unwrap();
    let rows = decode_list(lists(&entry)[0]);
    switch_to_combat(&mut game);
    let refreshed =
        decode_list(lists(&game.handle_in_world(&REFRESH, &character(), 1_000).unwrap())[0]);
    let file = battles();
    let shipped = file["battles"].as_array().unwrap();
    assert!(!shipped.is_empty());
    for battle in shipped {
        let id = battle["id"].as_u64().unwrap() as u32;
        let field = |name: &str| battle[name].as_i64().unwrap_or(0);
        let mut listed = rows.iter().filter(|row| row.id == id);
        let row = listed
            .next()
            .unwrap_or_else(|| panic!("battle {id} is listed"));
        assert!(listed.next().is_none(), "battle {id} is listed once");
        assert_eq!(
            row.category, BATTLES_CATEGORY,
            "battle {id} is under Battles"
        );
        assert_eq!(i64::from(row.name), field("title_id"), "battle {id} name");
        assert_eq!(
            i64::from(row.icon),
            field("icon_id").max(0),
            "battle {id} icon"
        );
        assert!(!row.can_join, "battle {id}: no PLAY! before a combat job");
        let after = refreshed.iter().find(|row| row.id == id).unwrap();
        if field("required_combat_level") == 0 {
            assert!(after.can_join, "battle {id}: PLAY! for a combat job");
        }
    }
}

#[test]
fn play_needs_a_combat_job_then_starts_the_battle_once() {
    let mut game = game();
    game.initialize_world(&character(), 500).unwrap();
    let wrong_job = game
        .handle_in_world(&play(BATTLE), &character(), 1_000)
        .unwrap();
    assert!(wrong_job.zoning.is_none());
    assert!(chat_text(&wrong_job).contains("combat job"));
    switch_to_combat(&mut game);

    let started = game
        .handle_in_world(&play(BATTLE), &character(), 2_000)
        .unwrap();
    assert!(started.zoning.is_some(), "Play zones into the arena");
    assert!(started.changed);

    let again = game
        .handle_in_world(&play(BATTLE), &character(), 3_000)
        .unwrap();
    assert!(again.zoning.is_none(), "a second Play cannot restart");
    assert!(chat_text(&again).contains("Finish what you are doing"));
}

#[test]
fn forged_and_unlisted_ids_neither_zone_nor_change_anything() {
    let mut game = game();
    game.initialize_world(&character(), 500).unwrap();
    switch_to_combat(&mut game);
    // Canary: the listed id works on a fresh game, so the refusals below are the id's fault.
    let mut control = self::game();
    control.initialize_world(&character(), 500).unwrap();
    switch_to_combat(&mut control);
    assert!(
        control
            .handle_in_world(&play(BATTLE), &character(), 2_000)
            .unwrap()
            .zoning
            .is_some()
    );

    for id in [0, 1, BATTLE + 1, 411, 0x8000_0000, u32::MAX] {
        let out = game
            .handle_in_world(&play(id), &character(), 2_000)
            .unwrap();
        assert!(
            out.zoning.is_none() && out.scene_zoning.is_none(),
            "id {id}"
        );
        assert!(!out.changed, "id {id}");
        assert!(chat_text(&out).contains("not available"), "id {id}");
    }
    // Malformed Play is an error, not a start.
    assert!(
        game.handle_in_world(&[167, 0, 1, 2, 96, 0, 0, 0], &character(), 2_500)
            .is_err()
    );
    // And the real one still works afterwards: nothing above consumed or poisoned state.
    assert!(
        game.handle_in_world(&play(BATTLE), &character(), 3_000)
            .unwrap()
            .zoning
            .is_some()
    );
}

#[test]
fn play_is_refused_away_from_the_open_world() {
    let mut game = game();
    game.initialize_world(&character(), 500).unwrap();
    switch_to_combat(&mut game);
    let mut elsewhere = character();
    elsewhere.zone_id = 411;
    let out = game
        .handle_in_world(&play(BATTLE), &elsewhere, 2_000)
        .unwrap();
    assert!(out.zoning.is_none());
    assert!(chat_text(&out).contains("cannot start that activity here"));
}

#[test]
fn a_finished_battle_closes_and_play_works_again() {
    let mut game = game();
    game.initialize_world(&character(), 500).unwrap();
    switch_to_combat(&mut game);
    game.handle_in_world(&play(BATTLE), &character(), 2_000)
        .unwrap();
    // The fight itself is outside this proof: a developer win stands in for it, then the
    // ordinary end-screen Close (26/42) returns the player exactly like a real player's.
    game.initialize_world(&character(), 3_000).unwrap();
    game.set_dev_tools(true);
    game.set_role(AccountRole::Admin);
    game.handle_in_world(&chat::local_request("realm win"), &character(), 4_000)
        .unwrap();
    game.set_dev_tools(false);
    game.set_role(AccountRole::Player);
    let closed = game
        .handle_in_world(&[26, 0, 42, 0], &character(), 5_000)
        .unwrap();
    assert!(closed.zoning.is_some(), "Close returns to the open world");
    game.initialize_world(&character(), 6_000).unwrap();
    let again = game
        .handle_in_world(&play(BATTLE), &character(), 7_000)
        .unwrap();
    assert!(
        again.zoning.is_some(),
        "the portal works again after returning"
    );
}
