//! Owned tests for the piece-015 minigame launch catalogue module.
//!
//! Run with the private toolchain:
//!
//!   scripts/cargo.ps1 test --test piece_015
//!
//! The synthetic tables mirror the shipped caret grammar (header row starting
//! with '#', trailing delimiter, CRLF-tolerant). The real-table test is
//! read-only against the preserved lab corpus and skips when the lab is absent.

#[path = "../src/restoration/piece_015.rs"]
mod piece_015;

use piece_015::{CatalogueError, LaunchCatalogue, LaunchMode};
use std::path::PathBuf;

const GAME_TABLE: &str = concat!(
    "#ID^TYPE^NAME_ID^DESCRIPTION_ID^ICON_ID^DIFFICULTY^MEMBERS_ONLY^SHORTCUT^",
    "MIN_PLAYERS^MAX_PLAYERS^START_SOUND_DEFINITION^PRELOAD_FILE^LEADERBOARD_ID^\n",
    "1^8^10^11^12^1^0^^0^2^0^^3^\n",
    "2^9^20^21^22^2^0^^0^0^0^^0^\n",
);
const TYPE_TABLE: &str = concat!(
    "#TYPE^NAME_ID^DESCRIPTION_ID^IS_FLASH^IS_MICRO^PARAM1^HOWTO_SWF_NAME^ICON_ID^",
    "MEMBERS_ONLY^BACKGROUND_ID^IS_POPULAR^IS_HIGHLIGHTED^CATEGORY_ID^IS_ACTIVE^\n",
    "8^0^0^1^0^0^^0^0^0^0^0^0^1^\n",
    "9^0^0^0^0^0^^0^0^0^0^0^0^1^\n",
);
const GROUP_TABLE: &str = concat!(
    "#ID^NAME_ID^DESCRIPTION_ID^ICON_ID^BACKGROUND_SWF^STAGE_PROGRESSION^",
    "SHOW_START_SCREEN_ON_PLAY_NEXT^SETTINGS_ICON_ID^TYPE_ID^PURCHASE_ITEM_ID^",
    "PURCHASE_BUNDLE_ID^PURCHASE_PREREQ_ITEM_ID^\n",
    "5^0^0^0^^^0^0^0^0^0^0^\n",
);
const LINK_TABLE: &str = concat!(
    "#ID^GROUP_ID^PARENT_GAME_ID^LINK_GAME_ID^LINK_NAME^SHORT_NAME^ICON_ID^",
    "STAGE_NUMBER^LINK_GROUP_ID^\n",
    "1^5^0^1^^^0^0^0^\n",
    "2^5^0^2^^^0^0^1^\n",
);

fn build(game: &str, types: &str, groups: &str, links: &str) -> LaunchCatalogue {
    LaunchCatalogue::parse(game, types, groups, links).expect("fixture tables parse")
}

fn lab_resources() -> Option<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("lab")
        .join("working")
        .join("OSFR.Client")
        .join("OSFR Client")
        .join("Resources");
    path.is_dir().then_some(path)
}

#[test]
fn parses_rows_and_reports_launch_modes() {
    let catalogue = build(GAME_TABLE, TYPE_TABLE, GROUP_TABLE, LINK_TABLE);
    let counts = catalogue.counts();
    assert_eq!(
        (counts.games, counts.types, counts.groups, counts.links),
        (2, 2, 1, 2)
    );
    assert_eq!(catalogue.launch_mode(1), Some(LaunchMode::Flash));
    assert_eq!(catalogue.launch_mode(2), Some(LaunchMode::Native));
    assert_eq!(catalogue.launch_mode(3), None);
    let game = catalogue.game(1).expect("row 1");
    assert_eq!(
        (game.name_id, game.max_players, game.leaderboard_id),
        (10, 2, 3)
    );
}

#[test]
fn rows_with_an_undeclared_type_are_undeclared() {
    let games = format!("{GAME_TABLE}3^999^0^0^0^1^0^^0^0^0^^0^\n");
    let catalogue = build(&games, TYPE_TABLE, GROUP_TABLE, LINK_TABLE);
    assert_eq!(catalogue.launch_mode(3), Some(LaunchMode::Undeclared));
}

#[test]
fn malformed_row_reports_source_and_line() {
    let games = format!("{GAME_TABLE}9^9^\n");
    let error = LaunchCatalogue::parse(&games, TYPE_TABLE, GROUP_TABLE, LINK_TABLE)
        .expect_err("short row must fail");
    match error {
        CatalogueError::Row {
            source,
            line,
            fields,
            expected,
        } => {
            assert_eq!(source, "MiniGameData.txt");
            assert_eq!(line, 4);
            // "9^9^" splits into three fields: the shipped rows end in '^'.
            assert_eq!(fields, 3);
            assert_eq!(expected, 14);
        }
        other => panic!("unexpected error {other:?}"),
    }
}

#[test]
fn duplicate_game_id_is_rejected() {
    let games = format!("{GAME_TABLE}1^8^0^0^0^1^0^^0^0^0^^0^\n");
    let error = LaunchCatalogue::parse(&games, TYPE_TABLE, GROUP_TABLE, LINK_TABLE)
        .expect_err("duplicate id must fail");
    match error {
        CatalogueError::DuplicateId { source, id } => {
            assert_eq!(source, "MiniGameData.txt");
            assert_eq!(id, 1);
        }
        other => panic!("unexpected error {other:?}"),
    }
}

#[test]
fn group_links_resolve_to_game_rows() {
    let catalogue = build(GAME_TABLE, TYPE_TABLE, GROUP_TABLE, LINK_TABLE);
    let mut games = catalogue.games_in_group(5);
    games.sort_unstable();
    assert_eq!(games, vec![1, 2]);
    assert!(catalogue.games_in_group(99).is_empty());
}

#[test]
fn real_tables_match_the_recovered_catalogue_counts() {
    let Some(resources) = lab_resources() else {
        eprintln!("skipping: preserved lab resources are not present");
        return;
    };
    let read =
        |name: &str| std::fs::read_to_string(resources.join(name)).expect("lab resource readable");
    let catalogue = LaunchCatalogue::parse(
        &read("MiniGameData.txt"),
        &read("MiniGameTypeData.txt"),
        &read("MiniGameGroupData.txt"),
        &read("MiniGameGroupLinks.txt"),
    )
    .expect("preserved tables parse");
    let counts = catalogue.counts();
    assert_eq!(
        (counts.games, counts.types, counts.groups, counts.links),
        (783, 22, 20, 610)
    );
    assert_eq!(catalogue.launch_mode(139), Some(LaunchMode::Flash));
    assert_eq!(catalogue.launch_mode(802), Some(LaunchMode::Undeclared));
    for id in [18, 42, 46, 77, 91, 109, 217, 584] {
        assert!(catalogue.launch_mode(id).is_some(), "row {id} resolves");
    }
}
