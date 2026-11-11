//! rc-0136 F4: world dressing answers a click, through the socket path (`test_support::Table`,
//! the shipped `data/content` with `data/release` laid over it, as a release runs). A wild
//! Bumbleberry Plant (an extra spawn with no content NPC row, `unknown_runtime_guid` before)
//! gives its harvest once per regrow time; the Halloween Pumpkin Prince figure says its line.
#[path = "fishing_fixtures.rs"]
#[allow(dead_code)] // the shared fixtures module also ships builders this test does not use
mod fixtures;
#[path = "../../../tests/support/pickup_packets.rs"]
mod pickup_packets;
#[path = "../../../tests/common/round70_content.rs"]
#[allow(dead_code)] // this test uses the reference path, not the fixture builder
mod round70_content;
use super::test_support::{self, Scratch, Table};
use super::*;
use fixtures::npc;
use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState},
};
use pickup_packets::said;

const BUMBLEBERRY: u64 = 100_000_033_110;
const BUMBLEBERRY_AT: [f32; 3] = [-1383.2769, -29.53867, 214.16885];
/// Bundle of Bumbleberry, the plant's `npc_presentation.json` harvest.
const BUNDLE: i32 = 2889;
const PUMPKIN_PRINCE: u64 = 100_000_031_334;
const PUMPKIN_PRINCE_AT: [f32; 3] = [-2392.853, -33.08391, 1155.678];
/// "Foolish mortals! I can't be beaten; I am the Pumpkin Prince!"
const PRINCE_LINE: i32 = 432_082;
/// 2026-10-31 12:00 UTC, inside Halloween.
const HALLOWEEN_MS: u64 = 1_793_448_000_000;

fn content(scratch: &Scratch) -> PathBuf {
    let content = scratch.path().join("content");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fixtures::copy_tree(&root.join("data/content"), &content);
    fixtures::copy_tree(&root.join("data/release"), &content);
    content
}

/// A fresh character (guid 1) standing `at` in the open world.
fn player(content: &Path, at: [f32; 3]) -> (CharacterState, GameplayState) {
    let mut character = Gateway::reference_state(&round70_content::reference()).unwrap();
    character.guid = 1;
    character.zone_id = 0;
    character.position = at;
    let mut game = Gameplay::load(content, None).unwrap();
    game.initialize_world(&character, 0).unwrap();
    (character, game.state())
}

fn table(tag: &str, at: [f32; 3]) -> Option<Table> {
    if !test_support::bin_test_target("f4") {
        return None;
    }
    let scratch = Scratch::new(tag);
    let content = content(&scratch);
    if !round70_content::reference().join("manifest.json").is_file() {
        return None;
    }
    let players = [player(&content, at)];
    let mut table = Table::new(scratch, &content, &round70_content::reference(), &players)?;
    table.tick(1_000);
    Some(table)
}

fn held(table: &Table, item: i32) -> u32 {
    let session = &table.server.sessions[&table.keys[0]];
    let state = session.gateway.snapshot().gameplay.unwrap();
    (state.inventory.items.iter())
        .filter(|row| row.definition_id == item)
        .map(|row| row.count)
        .sum()
}

#[test]
fn a_wild_plant_gives_its_harvest_once_per_regrow() {
    let near = [BUMBLEBERRY_AT[0] + 1.0, BUMBLEBERRY_AT[1], BUMBLEBERRY_AT[2]];
    let Some(mut table) = table("f4-plant", near) else {
        return;
    };
    let before = held(&table, BUNDLE);
    let click = npc(8, BUMBLEBERRY, None);
    let picked = table.send(0, &click, 2_000);
    assert!(
        said(&picked, "You gathered from the Bumbleberry Plant."),
        "the click is answered with the harvest"
    );
    assert_eq!(held(&table, BUNDLE), before + 1, "a Bundle of Bumbleberry is in the bag");
    let spent = table.send(0, &click, 3_000);
    assert!(said(&spent, "grow back soon"), "a picked plant waits for its regrow");
    assert_eq!(held(&table, BUNDLE), before + 1, "and pays nothing meanwhile");
    let regrown = table.send(0, &click, 2_000 + 300_000);
    assert!(said(&regrown, "You gathered"), "after 300 s it gives again");
    assert_eq!(held(&table, BUNDLE), before + 2);
}

#[test]
fn a_plant_out_of_reach_answers_without_paying() {
    let far = [BUMBLEBERRY_AT[0] + 30.0, BUMBLEBERRY_AT[1], BUMBLEBERRY_AT[2]];
    let Some(mut table) = table("f4-far", far) else {
        return;
    };
    let before = held(&table, BUNDLE);
    let refused = table.send(0, &npc(8, BUMBLEBERRY, None), 2_000);
    assert!(said(&refused, "Move closer"), "30 m is out of reach");
    assert_eq!(held(&table, BUNDLE), before);
}

/// The Pumpkin Prince figure (Halloween only, so the clock is pinned): its line, a 15/4 from
/// its own guid, and nothing else changes.
#[test]
fn the_pumpkin_prince_figure_says_its_line() {
    let near = [PUMPKIN_PRINCE_AT[0] + 1.0, PUMPKIN_PRINCE_AT[1], PUMPKIN_PRINCE_AT[2]];
    let Some(mut table) = table("f4-prince", near) else {
        return;
    };
    let key = table.keys[0];
    let session = table.server.sessions.get_mut(&key).unwrap();
    session.gateway.gameplay_mut().unwrap().set_event_clock(Some(HALLOWEEN_MS));
    let line = osfr_tools::chat::string_say(PUMPKIN_PRINCE, PRINCE_LINE);
    let replies = table.send(0, &npc(8, PUMPKIN_PRINCE, None), 3_000);
    assert!(replies.contains(&line), "the figure says its line: {replies:?}");
}

/// rc-0136 F4 with F3's display shop: the Striped and Armored T-Rex displays carry their rides'
/// own name ids (5570, 6403), so a click takes F3's Station Cash path and opens the window that
/// sells that ride (76853, 76859); the Legacy ids they had matched no Marketplace item.
#[test]
fn the_t_rex_displays_open_their_station_cash_shop() {
    for (display, at, ride) in [
        (100_000_004_661_u64, [-908.5041_f32, -34.22184, -380.331], 76_853_i32),
        (100_000_004_662, [-894.3927, -33.65656, -385.3986], 76_859),
    ] {
        let near = [at[0] + 2.0, at[1], at[2]];
        let Some(mut table) = table("f4-trex", near) else {
            return;
        };
        let replies = table.send(0, &npc(8, display, None), 2_000);
        let window = replies
            .iter()
            .find(|reply| reply.starts_with(&[66, 0, 42, 0]))
            .unwrap_or_else(|| panic!("{display} opens the Station Cash window: {replies:?}"));
        let list = osfr_tools::npc_interaction::decode_merchant_list_66_42(window).unwrap();
        assert!(
            (list.group.entries.iter()).any(|row| row.first_item_id as i32 == ride),
            "{display} sells ride {ride}"
        );
    }
}
