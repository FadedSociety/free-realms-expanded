//! Housing decoration discovery (MMO work queue item 029).
//!
//! ORIGINAL evidence used by these vectors (all of it lives outside this test):
//! * `lab/runs/client-baseline-20261009T013155Z/client/Resources/StringHashToValue.txt`
//!   (sha256 `22003f3224dcc1cd5a189e6e818ab8c82711e713183f04dba13b76c255aeb570`) ships the only
//!   fixture-action name vocabulary the client knows:
//!   `Housing.ElevateFixtureActionName^Elevate^`, `Housing.MoveFixtureActionName^MoveFixture^`,
//!   `Housing.RemoveFixtureActionName^RemoveFixture^`.
//! * `lab/decompiled/client-ui/ScriptsBase.lua`
//!   (sha256 `c7c71c7cbb177c975a7cdb12ed0c2e1b74d662f5ca5ef6a4e9ff86d2d332a8f6`) declares the five
//!   housing radial interactions (`Housing.INTERACTION_PICK_UP = 1`, `INTERACTION_MOVE = 2`,
//!   `INTERACTION_ROTATE_LEFT = 3`, `INTERACTION_ROTATE_RIGHT = 4`, `INTERACTION_VERTICAL_MOVE = 5`),
//!   maps each of them to its `House.*` call inside `Housing:OnRadialMenuClick`, and asks
//!   `House.IsActionPermitted(currentItemGuid, "MoveFixture")` / `"RemoveFixture"` before the move
//!   and remove buttons are offered. The addressee is read from
//!   `BaseClient.Housing.Instance.SelectedFixture`, where `-1` means no fixture is selected.
//! * `.../Resources/Requirements.txt` (sha256 `1dc2f97fd35fcc20b55213ba8f08e93c43c84f442c139f08c0c816d5f7012644`)
//!   is the shipped requirement table (ids 1..=43, factories `CheckRankLastProfileByType`,
//!   `AlwaysFails`, `SourceVipRank`, `CheckCurrentProfile`, `CheckRenamePending`, `CheckNudge`) that a
//!   fixture action's `RequirementId` resolves against.
//! * the pinned community packet model
//!   (`lab/references/FreeRealms-Legacy/src/Sanctuary.Packet.Common/Housing/FixtureDefinition.cs`)
//!   carries `Category`, `LuaCall`, an `Actions` map of `FixtureAction { Id, Name, RequirementId }`
//!   and `CompositeEffectId`; the shipped client builds the same
//!   `SoeUtil::HashListMap<int, FixtureDefinition::FixtureAction, 16, -1>` container
//!   (`pseudocode.c:1402661`) and keys every fixture-scoped message by fixture guid
//!   (`SoeUtilSignatureFixtureGuid`, `HousingPacketFixtureRemove`, `Selected House Fixture: %lld`).
//!
//! This file is a `#[path]` include so it exercises the real module before the lead registers it in
//! `src/lib.rs`. Integration step for the lead: register the module, drop the include, and drive it
//! from the item 006/025/026 housing path.

#[path = "../src/restoration/piece_029.rs"]
#[allow(dead_code)]
mod piece_029;

use piece_029::{
    ClientArea, FIXTURE_ACTION_ELEVATE, FIXTURE_ACTION_MOVE, FIXTURE_ACTION_REMOVE,
    FixtureActionDefinition, FixtureDefinitionRow, HouseApiCall, INTERACTION_MOVE,
    INTERACTION_PICK_UP, INTERACTION_ROTATE_LEFT, INTERACTION_ROTATE_RIGHT,
    INTERACTION_VERTICAL_MOVE, InteractionRequest, NO_FIXTURE_SELECTED, PlacedFixtureRow, Refusal,
    Selection, fixture_action_names, plan_interaction, selection_from_guid,
};

const LOT_A: u64 = 0x0f00_0000_0000_0001;
const LOT_B: u64 = 0x0f00_0000_0000_0002;
const CHAIR_A: u64 = 0x0f00_0000_0000_0011;
const CHAIR_B: u64 = 0x0f00_0000_0000_0012;
const TABLE_A: u64 = 0x0f00_0000_0000_0021;

const CHAIR_ITEM: i32 = 100;
const TABLE_ITEM: i32 = 200;
const CHAIR_DEFINITION: i32 = 1;
const TABLE_DEFINITION: i32 = 2;

/// ORIGINAL requirement ids are 1..=43 in the shipped table; the vectors use three of them.
fn known_requirements() -> Vec<i32> {
    vec![3, 25, 31]
}

fn fixture(guid: u64, lot: u64, definition: i32, item: i32) -> PlacedFixtureRow {
    PlacedFixtureRow {
        fixture_guid: guid,
        lot_guid: lot,
        fixture_definition_id: definition,
        item_definition_id: item,
    }
}

fn action(name: &str, requirement_id: i32) -> FixtureActionDefinition {
    FixtureActionDefinition {
        name: name.to_owned(),
        requirement_id,
    }
}

fn definition(
    id: i32,
    item: i32,
    members_only: bool,
    actions: Vec<FixtureActionDefinition>,
) -> FixtureDefinitionRow {
    FixtureDefinitionRow {
        fixture_definition_id: id,
        item_definition_id: item,
        members_only,
        actions,
    }
}

/// A lot with two identical chairs, one table, and definitions for both.
fn lot_a() -> (Vec<PlacedFixtureRow>, Vec<FixtureDefinitionRow>) {
    let fixtures = vec![
        fixture(CHAIR_A, LOT_A, CHAIR_DEFINITION, CHAIR_ITEM),
        fixture(CHAIR_B, LOT_A, CHAIR_DEFINITION, CHAIR_ITEM),
        fixture(TABLE_A, LOT_A, TABLE_DEFINITION, TABLE_ITEM),
    ];
    let definitions = vec![
        definition(
            CHAIR_DEFINITION,
            CHAIR_ITEM,
            false,
            vec![
                action(FIXTURE_ACTION_MOVE, 0),
                action(FIXTURE_ACTION_REMOVE, 0),
                action(FIXTURE_ACTION_ELEVATE, 0),
            ],
        ),
        definition(
            TABLE_DEFINITION,
            TABLE_ITEM,
            false,
            vec![action(FIXTURE_ACTION_MOVE, 0)],
        ),
    ];
    (fixtures, definitions)
}

fn request<'a>(
    fixtures: &'a [PlacedFixtureRow],
    definitions: &'a [FixtureDefinitionRow],
    requirements: &'a [i32],
    selection: Selection,
    interaction: i32,
) -> InteractionRequest<'a> {
    InteractionRequest {
        lot_guid: LOT_A,
        selection,
        interaction,
        area: ClientArea::Housing,
        is_member: true,
        can_edit: true,
        fixtures,
        definitions,
        requirement_ids: requirements,
    }
}

#[test]
fn recovered_action_names_and_interaction_ids_match_the_client() {
    assert_eq!(
        fixture_action_names(),
        vec![
            FIXTURE_ACTION_ELEVATE,
            FIXTURE_ACTION_MOVE,
            FIXTURE_ACTION_REMOVE
        ]
    );
    assert_eq!(INTERACTION_PICK_UP, 1);
    assert_eq!(INTERACTION_MOVE, 2);
    assert_eq!(INTERACTION_ROTATE_LEFT, 3);
    assert_eq!(INTERACTION_ROTATE_RIGHT, 4);
    assert_eq!(INTERACTION_VERTICAL_MOVE, 5);
    assert_eq!(NO_FIXTURE_SELECTED, -1);
    assert_eq!(selection_from_guid(NO_FIXTURE_SELECTED), Selection::None);
    assert_eq!(
        selection_from_guid(CHAIR_A as i64),
        Selection::FixtureGuid(CHAIR_A)
    );
}

#[test]
fn radial_interaction_maps_to_the_recovered_house_call() {
    let (fixtures, definitions) = lot_a();
    let requirements = known_requirements();
    let expected = [
        (
            INTERACTION_PICK_UP,
            HouseApiCall::PickupFixture,
            "House.PickupFixture",
        ),
        (
            INTERACTION_MOVE,
            HouseApiCall::MoveFixture,
            "House.MoveFixture",
        ),
        (
            INTERACTION_ROTATE_LEFT,
            HouseApiCall::RotateFixtureCounterClockwise,
            "House.RotateFixtureCounterClockwise",
        ),
        (
            INTERACTION_ROTATE_RIGHT,
            HouseApiCall::RotateFixtureClockwise,
            "House.RotateFixtureClockwise",
        ),
        (
            INTERACTION_VERTICAL_MOVE,
            HouseApiCall::MoveFixtureVertically,
            "House.MoveFixtureVertically",
        ),
    ];
    for (interaction, api, lua_call) in expected {
        let plan = plan_interaction(&request(
            &fixtures,
            &definitions,
            &requirements,
            Selection::FixtureGuid(CHAIR_A),
            interaction,
        ))
        .expect("the chair declares every action the radial menu can ask for");
        assert_eq!(plan.api, api);
        assert_eq!(plan.api.lua_call(), lua_call);
        assert_eq!(plan.fixture_guid, CHAIR_A);
    }
}

#[test]
fn guid_selection_resolves_the_addressed_instance() {
    let (fixtures, definitions) = lot_a();
    let requirements = known_requirements();
    let plan = plan_interaction(&request(
        &fixtures,
        &definitions,
        &requirements,
        Selection::FixtureGuid(CHAIR_B),
        INTERACTION_MOVE,
    ))
    .expect("second chair resolves by its own guid");
    assert_eq!(plan.fixture_guid, CHAIR_B);
    assert_eq!(plan.fixture_definition_id, CHAIR_DEFINITION);
    assert_eq!(plan.item_definition_id, CHAIR_ITEM);
}

#[test]
fn definition_only_selection_refuses_an_ambiguous_instance() {
    let (fixtures, definitions) = lot_a();
    let requirements = known_requirements();
    // Two identical chairs share item definition 100. Picking the first match would move or remove
    // the wrong instance, so the gate refuses to choose.
    let outcome = plan_interaction(&request(
        &fixtures,
        &definitions,
        &requirements,
        Selection::ItemDefinition(CHAIR_ITEM),
        INTERACTION_MOVE,
    ));
    assert_eq!(outcome, Err(Refusal::AmbiguousSelection));
}

#[test]
fn definition_only_selection_resolves_a_unique_instance() {
    let (fixtures, definitions) = lot_a();
    let requirements = known_requirements();
    let plan = plan_interaction(&request(
        &fixtures,
        &definitions,
        &requirements,
        Selection::ItemDefinition(TABLE_ITEM),
        INTERACTION_MOVE,
    ))
    .expect("the table is the only instance of its item definition");
    assert_eq!(plan.fixture_guid, TABLE_A);
}

#[test]
fn a_fixture_from_another_lot_is_not_selectable() {
    let (mut fixtures, definitions) = lot_a();
    fixtures.push(fixture(CHAIR_A, LOT_B, CHAIR_DEFINITION, CHAIR_ITEM));
    let requirements = known_requirements();
    // The same guid exists, but it belongs to lot B, so it is not an instance of the addressee lot.
    let outcome = plan_interaction(&request(
        &fixtures,
        &definitions,
        &requirements,
        Selection::FixtureGuid(0x0f00_0000_0000_00ff),
        INTERACTION_MOVE,
    ));
    assert_eq!(outcome, Err(Refusal::UnknownFixture));
}

#[test]
fn duplicate_guid_in_the_same_lot_is_refused() {
    let (mut fixtures, definitions) = lot_a();
    fixtures.push(fixture(CHAIR_A, LOT_A, CHAIR_DEFINITION, CHAIR_ITEM));
    let requirements = known_requirements();
    let outcome = plan_interaction(&request(
        &fixtures,
        &definitions,
        &requirements,
        Selection::FixtureGuid(CHAIR_A),
        INTERACTION_MOVE,
    ));
    assert_eq!(outcome, Err(Refusal::AmbiguousSelection));
}

#[test]
fn no_selection_refuses_the_interaction() {
    let (fixtures, definitions) = lot_a();
    let requirements = known_requirements();
    let outcome = plan_interaction(&request(
        &fixtures,
        &definitions,
        &requirements,
        Selection::None,
        INTERACTION_MOVE,
    ));
    assert_eq!(outcome, Err(Refusal::NoSelection));
}

#[test]
fn unknown_interaction_ids_are_refused() {
    let (fixtures, definitions) = lot_a();
    let requirements = known_requirements();
    for interaction in [0, 6, -1, 99] {
        let outcome = plan_interaction(&request(
            &fixtures,
            &definitions,
            &requirements,
            Selection::FixtureGuid(CHAIR_A),
            interaction,
        ));
        assert_eq!(outcome, Err(Refusal::UnknownInteraction));
    }
}

#[test]
fn a_fixture_whose_definition_never_arrived_is_refused() {
    let (fixtures, _) = lot_a();
    let definitions: Vec<FixtureDefinitionRow> = Vec::new();
    let requirements = known_requirements();
    let outcome = plan_interaction(&request(
        &fixtures,
        &definitions,
        &requirements,
        Selection::FixtureGuid(CHAIR_A),
        INTERACTION_MOVE,
    ));
    assert_eq!(outcome, Err(Refusal::UnknownDefinition));
}

#[test]
fn interaction_without_the_declared_action_is_refused() {
    let (fixtures, definitions) = lot_a();
    let requirements = known_requirements();
    // The table declares only MoveFixture: the client may ask about RemoveFixture, and the answer
    // must be "no" rather than a silent removal.
    let outcome = plan_interaction(&request(
        &fixtures,
        &definitions,
        &requirements,
        Selection::FixtureGuid(TABLE_A),
        INTERACTION_PICK_UP,
    ));
    assert_eq!(
        outcome,
        Err(Refusal::ActionNotDeclared {
            action: FIXTURE_ACTION_REMOVE
        })
    );

    let plan = plan_interaction(&request(
        &fixtures,
        &definitions,
        &requirements,
        Selection::FixtureGuid(TABLE_A),
        INTERACTION_MOVE,
    ))
    .expect("the table declares MoveFixture");
    assert_eq!(plan.gated_action, Some(FIXTURE_ACTION_MOVE));
}

#[test]
fn farming_area_gates_every_interaction_on_move_fixture() {
    let fixtures = vec![fixture(CHAIR_A, LOT_A, CHAIR_DEFINITION, CHAIR_ITEM)];
    let definitions = vec![definition(
        CHAIR_DEFINITION,
        CHAIR_ITEM,
        false,
        vec![action(FIXTURE_ACTION_REMOVE, 0)],
    )];
    let requirements = known_requirements();

    let mut farming = request(
        &fixtures,
        &definitions,
        &requirements,
        Selection::FixtureGuid(CHAIR_A),
        INTERACTION_MOVE,
    );
    farming.area = ClientArea::Farming;
    assert_eq!(
        plan_interaction(&farming),
        Err(Refusal::ActionNotDeclared {
            action: FIXTURE_ACTION_MOVE
        })
    );

    let definitions = vec![definition(
        CHAIR_DEFINITION,
        CHAIR_ITEM,
        false,
        vec![
            action(FIXTURE_ACTION_MOVE, 0),
            action(FIXTURE_ACTION_REMOVE, 0),
        ],
    )];
    let mut farming = request(
        &fixtures,
        &definitions,
        &requirements,
        Selection::FixtureGuid(CHAIR_A),
        INTERACTION_PICK_UP,
    );
    farming.area = ClientArea::Farming;
    let plan = plan_interaction(&farming).expect("both farming gates pass");
    assert_eq!(plan.gated_action, Some(FIXTURE_ACTION_REMOVE));
}

#[test]
fn unknown_requirement_is_refused_instead_of_allowed() {
    let fixtures = vec![fixture(CHAIR_A, LOT_A, CHAIR_DEFINITION, CHAIR_ITEM)];
    let requirements = known_requirements();

    let definitions = vec![definition(
        CHAIR_DEFINITION,
        CHAIR_ITEM,
        false,
        vec![action(FIXTURE_ACTION_MOVE, 99)],
    )];
    assert_eq!(
        plan_interaction(&request(
            &fixtures,
            &definitions,
            &requirements,
            Selection::FixtureGuid(CHAIR_A),
            INTERACTION_MOVE,
        )),
        Err(Refusal::UnknownRequirement { requirement_id: 99 })
    );

    // RequirementId 0 means the shipped table declares no requirement for this action.
    let definitions = vec![definition(
        CHAIR_DEFINITION,
        CHAIR_ITEM,
        false,
        vec![action(FIXTURE_ACTION_MOVE, 0)],
    )];
    let plan = plan_interaction(&request(
        &fixtures,
        &definitions,
        &requirements,
        Selection::FixtureGuid(CHAIR_A),
        INTERACTION_MOVE,
    ))
    .expect("no requirement is declared");
    assert_eq!(plan.requirement_id, None);

    let definitions = vec![definition(
        CHAIR_DEFINITION,
        CHAIR_ITEM,
        false,
        vec![action(FIXTURE_ACTION_MOVE, 25)],
    )];
    let plan = plan_interaction(&request(
        &fixtures,
        &definitions,
        &requirements,
        Selection::FixtureGuid(CHAIR_A),
        INTERACTION_MOVE,
    ))
    .expect("requirement 25 is in the shipped table");
    assert_eq!(plan.requirement_id, Some(25));
}

#[test]
fn members_only_fixture_offers_only_removal_to_a_guest() {
    let fixtures = vec![fixture(CHAIR_A, LOT_A, CHAIR_DEFINITION, CHAIR_ITEM)];
    let definitions = vec![definition(
        CHAIR_DEFINITION,
        CHAIR_ITEM,
        true,
        vec![
            action(FIXTURE_ACTION_MOVE, 0),
            action(FIXTURE_ACTION_REMOVE, 0),
        ],
    )];
    let requirements = known_requirements();

    let mut guest = request(
        &fixtures,
        &definitions,
        &requirements,
        Selection::FixtureGuid(CHAIR_A),
        INTERACTION_MOVE,
    );
    guest.is_member = false;
    assert_eq!(plan_interaction(&guest), Err(Refusal::MembersOnly));

    let mut guest_remove = guest.clone();
    guest_remove.interaction = INTERACTION_PICK_UP;
    let plan = plan_interaction(&guest_remove).expect("a guest may still pick the item up");
    assert_eq!(plan.api, HouseApiCall::PickupFixture);
}

#[test]
fn read_only_lot_requires_edit_permission() {
    let (fixtures, definitions) = lot_a();
    let requirements = known_requirements();
    let mut viewer = request(
        &fixtures,
        &definitions,
        &requirements,
        Selection::FixtureGuid(CHAIR_A),
        INTERACTION_ROTATE_LEFT,
    );
    viewer.can_edit = false;
    assert_eq!(plan_interaction(&viewer), Err(Refusal::NotAllowedToEdit));
}

#[test]
fn plans_are_pure_and_repeatable() {
    let (fixtures, definitions) = lot_a();
    let requirements = known_requirements();
    let fixtures_before = fixtures.clone();
    let definitions_before = definitions.clone();

    let first = plan_interaction(&request(
        &fixtures,
        &definitions,
        &requirements,
        Selection::FixtureGuid(CHAIR_A),
        INTERACTION_ROTATE_RIGHT,
    ))
    .expect("rotate right needs no named action");
    let second = plan_interaction(&request(
        &fixtures,
        &definitions,
        &requirements,
        Selection::FixtureGuid(CHAIR_A),
        INTERACTION_ROTATE_RIGHT,
    ))
    .expect("repeat");
    assert_eq!(first, second);
    assert_eq!(fixtures, fixtures_before, "the gate never mutates the lot");
    assert_eq!(definitions, definitions_before);
    assert_eq!(
        first.gated_action, None,
        "rotate right is not gated by a named action"
    );
}

/// Parses the shipped text files and re-derives the vocabulary the vectors above are built on.
///
/// Run with:
/// `OSFR_PIECE_029_CONFIG=<StringHashToValue.txt> OSFR_PIECE_029_LUA=<ScriptsBase.lua>
///  OSFR_PIECE_029_REQUIREMENTS=<Requirements.txt> OSFR_PIECE_029_REPORT=<dir>
///  cargo test --test piece_029 -- --ignored --exact private_decoration_vocabulary_matches_shipped_client_files`
#[test]
#[ignore = "requires preserved client Resources and the decompiled client Lua"]
fn private_decoration_vocabulary_matches_shipped_client_files() {
    use piece_029::{
        extract_action_name_entries, extract_interaction_ids, extract_lua_action_requests,
        requirement_factories, requirement_ids,
    };

    let config_path = std::env::var_os("OSFR_PIECE_029_CONFIG")
        .expect("OSFR_PIECE_029_CONFIG must point at StringHashToValue.txt");
    let lua_path = std::env::var_os("OSFR_PIECE_029_LUA")
        .expect("OSFR_PIECE_029_LUA must point at ScriptsBase.lua");
    let requirements_path = std::env::var_os("OSFR_PIECE_029_REQUIREMENTS")
        .expect("OSFR_PIECE_029_REQUIREMENTS must point at Requirements.txt");
    let report_dir = std::env::var_os("OSFR_PIECE_029_REPORT")
        .expect("OSFR_PIECE_029_REPORT must point at a directory");

    let config = std::fs::read_to_string(&config_path).expect("readable StringHashToValue.txt");
    let lua = std::fs::read_to_string(&lua_path).expect("readable ScriptsBase.lua");
    let requirements_text =
        std::fs::read_to_string(&requirements_path).expect("readable Requirements.txt");

    let entries = extract_action_name_entries(&config);
    assert_eq!(
        entries,
        vec![
            (
                "Housing.ElevateFixtureActionName".to_owned(),
                FIXTURE_ACTION_ELEVATE.to_owned()
            ),
            (
                "Housing.MoveFixtureActionName".to_owned(),
                FIXTURE_ACTION_MOVE.to_owned()
            ),
            (
                "Housing.RemoveFixtureActionName".to_owned(),
                FIXTURE_ACTION_REMOVE.to_owned()
            ),
        ]
    );

    let ids = extract_interaction_ids(&lua);
    assert_eq!(
        ids,
        vec![
            ("INTERACTION_PICK_UP".to_owned(), INTERACTION_PICK_UP),
            ("INTERACTION_MOVE".to_owned(), INTERACTION_MOVE),
            (
                "INTERACTION_ROTATE_LEFT".to_owned(),
                INTERACTION_ROTATE_LEFT
            ),
            (
                "INTERACTION_ROTATE_RIGHT".to_owned(),
                INTERACTION_ROTATE_RIGHT
            ),
            (
                "INTERACTION_VERTICAL_MOVE".to_owned(),
                INTERACTION_VERTICAL_MOVE
            ),
        ]
    );

    let asked = extract_lua_action_requests(&lua);
    assert!(asked.contains(&FIXTURE_ACTION_MOVE.to_owned()));
    assert!(asked.contains(&FIXTURE_ACTION_REMOVE.to_owned()));
    assert!(
        lua.contains("BaseClient.Housing.Instance.SelectedFixture"),
        "the selected instance data source is the interaction addressee"
    );
    assert!(lua.contains("tonumber(currentItemGuid) == -1"));
    assert!(lua.contains("House.IsFixtureMembersOnly"));

    let table = requirement_ids(&requirements_text);
    assert_eq!(table.first(), Some(&1));
    assert_eq!(table.last(), Some(&43));
    assert_eq!(table.len(), 43);
    let factories = requirement_factories(&requirements_text);
    for factory in [
        "CheckRankLastProfileByType",
        "AlwaysFails",
        "SourceVipRank",
        "CheckCurrentProfile",
        "CheckRenamePending",
        "CheckNudge",
    ] {
        assert!(
            factories.contains(&factory.to_owned()),
            "{factory} is shipped"
        );
    }

    // Export the vectors and this implementation's decisions so the independent Python checker can
    // re-evaluate the same inputs in another language and compare.
    let (mut fixtures, mut definitions) = lot_a();
    const LOCKED_CHAIR: u64 = 0x0f00_0000_0000_0031;
    const GATED_CHAIR: u64 = 0x0f00_0000_0000_0032;
    const REMOVE_ONLY: u64 = 0x0f00_0000_0000_0033;
    const MEMBERS_CHAIR: u64 = 0x0f00_0000_0000_0034;
    const ORPHAN_CHAIR: u64 = 0x0f00_0000_0000_0035;
    const LOCKED_DEFINITION: i32 = 3;
    const GATED_DEFINITION: i32 = 4;
    const REMOVE_ONLY_DEFINITION: i32 = 5;
    const MEMBERS_DEFINITION: i32 = 6;
    const ORPHAN_DEFINITION: i32 = 7;
    const REQUIREMENT_ROW: i32 = 25;
    const ABSENT_REQUIREMENT: i32 = 99;

    fixtures.push(fixture(LOCKED_CHAIR, LOT_B, LOCKED_DEFINITION, 300));
    fixtures.push(fixture(GATED_CHAIR, LOT_B, GATED_DEFINITION, 301));
    fixtures.push(fixture(REMOVE_ONLY, LOT_B, REMOVE_ONLY_DEFINITION, 302));
    fixtures.push(fixture(MEMBERS_CHAIR, LOT_B, MEMBERS_DEFINITION, 303));
    fixtures.push(fixture(ORPHAN_CHAIR, LOT_B, ORPHAN_DEFINITION, 304));
    definitions.push(definition(
        LOCKED_DEFINITION,
        300,
        false,
        vec![action(FIXTURE_ACTION_MOVE, REQUIREMENT_ROW)],
    ));
    definitions.push(definition(
        GATED_DEFINITION,
        301,
        false,
        vec![action(FIXTURE_ACTION_MOVE, ABSENT_REQUIREMENT)],
    ));
    definitions.push(definition(
        REMOVE_ONLY_DEFINITION,
        302,
        false,
        vec![action(FIXTURE_ACTION_REMOVE, 0)],
    ));
    definitions.push(definition(
        MEMBERS_DEFINITION,
        303,
        true,
        vec![
            action(FIXTURE_ACTION_MOVE, 0),
            action(FIXTURE_ACTION_REMOVE, 0),
        ],
    ));

    let vectors: Vec<(&str, Selection, i32, ClientArea, bool, bool)> = vec![
        (
            "guid_resolves_second_chair",
            Selection::FixtureGuid(CHAIR_B),
            INTERACTION_MOVE,
            ClientArea::Housing,
            true,
            true,
        ),
        (
            "guid_resolves_table",
            Selection::FixtureGuid(TABLE_A),
            INTERACTION_ROTATE_RIGHT,
            ClientArea::Housing,
            true,
            true,
        ),
        (
            "ambiguous_item_definition",
            Selection::ItemDefinition(CHAIR_ITEM),
            INTERACTION_MOVE,
            ClientArea::Housing,
            true,
            true,
        ),
        (
            "unique_item_definition",
            Selection::ItemDefinition(TABLE_ITEM),
            INTERACTION_MOVE,
            ClientArea::Housing,
            true,
            true,
        ),
        (
            "unknown_item_definition",
            Selection::ItemDefinition(999),
            INTERACTION_MOVE,
            ClientArea::Housing,
            true,
            true,
        ),
        (
            "foreign_lot_guid",
            Selection::FixtureGuid(0x0f00_0000_0000_00ff),
            INTERACTION_MOVE,
            ClientArea::Housing,
            true,
            true,
        ),
        (
            "no_selection",
            Selection::None,
            INTERACTION_MOVE,
            ClientArea::Housing,
            true,
            true,
        ),
        (
            "table_has_no_remove",
            Selection::FixtureGuid(TABLE_A),
            INTERACTION_PICK_UP,
            ClientArea::Housing,
            true,
            true,
        ),
        (
            "table_has_move",
            Selection::FixtureGuid(TABLE_A),
            INTERACTION_MOVE,
            ClientArea::Housing,
            true,
            true,
        ),
        (
            "rotate_right_is_ungated",
            Selection::FixtureGuid(TABLE_A),
            INTERACTION_ROTATE_RIGHT,
            ClientArea::Housing,
            true,
            true,
        ),
        (
            "read_only_lot",
            Selection::FixtureGuid(TABLE_A),
            INTERACTION_MOVE,
            ClientArea::Housing,
            true,
            false,
        ),
        (
            "unknown_interaction",
            Selection::FixtureGuid(CHAIR_A),
            9,
            ClientArea::Housing,
            true,
            true,
        ),
        (
            "chair_removal_housing",
            Selection::FixtureGuid(CHAIR_A),
            INTERACTION_PICK_UP,
            ClientArea::Housing,
            true,
            true,
        ),
        (
            "chair_removal_farming",
            Selection::FixtureGuid(CHAIR_A),
            INTERACTION_PICK_UP,
            ClientArea::Farming,
            true,
            true,
        ),
        (
            "requirement_known",
            Selection::FixtureGuid(LOCKED_CHAIR),
            INTERACTION_MOVE,
            ClientArea::Housing,
            true,
            true,
        ),
        (
            "requirement_absent",
            Selection::FixtureGuid(GATED_CHAIR),
            INTERACTION_MOVE,
            ClientArea::Housing,
            true,
            true,
        ),
        (
            "remove_only_house_move",
            Selection::FixtureGuid(REMOVE_ONLY),
            INTERACTION_MOVE,
            ClientArea::Housing,
            true,
            true,
        ),
        (
            "remove_only_farming_move",
            Selection::FixtureGuid(REMOVE_ONLY),
            INTERACTION_MOVE,
            ClientArea::Farming,
            true,
            true,
        ),
        (
            "remove_only_farming_remove",
            Selection::FixtureGuid(REMOVE_ONLY),
            INTERACTION_PICK_UP,
            ClientArea::Farming,
            true,
            true,
        ),
        (
            "members_only_guest_move",
            Selection::FixtureGuid(MEMBERS_CHAIR),
            INTERACTION_MOVE,
            ClientArea::Housing,
            false,
            true,
        ),
        (
            "members_only_guest_remove",
            Selection::FixtureGuid(MEMBERS_CHAIR),
            INTERACTION_PICK_UP,
            ClientArea::Housing,
            false,
            true,
        ),
        (
            "orphan_definition",
            Selection::FixtureGuid(ORPHAN_CHAIR),
            INTERACTION_MOVE,
            ClientArea::Housing,
            true,
            true,
        ),
    ];

    let mut exported = Vec::new();
    for (name, selection, interaction, area, is_member, can_edit) in vectors {
        let mut request = request(&fixtures, &definitions, &table, selection, interaction);
        request.area = area;
        request.is_member = is_member;
        request.can_edit = can_edit;
        let outcome = plan_interaction(&request);
        exported.push(serde_json::json!({
            "name": name,
            "lot": request.lot_guid,
            "selection": selection_json(selection),
            "interaction": interaction,
            "area": area_name(area),
            "is_member": is_member,
            "can_edit": can_edit,
            "rust": outcome_json(&outcome),
        }));
    }

    let vectors = serde_json::json!({
        "requirements": table,
        "vectors": exported,
        "fixtures": fixtures.iter().map(|row| serde_json::json!({
            "fixture_guid": row.fixture_guid,
            "lot_guid": row.lot_guid,
            "fixture_definition_id": row.fixture_definition_id,
            "item_definition_id": row.item_definition_id,
        })).collect::<Vec<_>>(),
        "definitions": definitions.iter().map(|row| serde_json::json!({
            "fixture_definition_id": row.fixture_definition_id,
            "item_definition_id": row.item_definition_id,
            "members_only": row.members_only,
            "actions": row.actions.iter().map(|entry| serde_json::json!({
                "name": entry.name,
                "requirement_id": entry.requirement_id,
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    });

    assert!(table.contains(&REQUIREMENT_ROW));
    assert!(!table.contains(&ABSENT_REQUIREMENT));

    let report = std::path::PathBuf::from(&report_dir);
    std::fs::create_dir_all(&report).expect("report directory");
    let payload = serde_json::to_vec_pretty(&vectors).expect("vectors serialize");
    std::fs::write(report.join("interaction-vectors.json"), payload).expect("vectors written");
}

fn selection_json(selection: Selection) -> serde_json::Value {
    match selection {
        Selection::FixtureGuid(guid) => serde_json::json!({ "kind": "guid", "value": guid }),
        Selection::ItemDefinition(item) => serde_json::json!({ "kind": "item", "value": item }),
        Selection::None => serde_json::json!({ "kind": "none", "value": NO_FIXTURE_SELECTED }),
    }
}

fn area_name(area: ClientArea) -> &'static str {
    match area {
        ClientArea::Housing => "Housing",
        ClientArea::Farming => "Farming",
    }
}

fn api_name(api: HouseApiCall) -> &'static str {
    match api {
        HouseApiCall::PickupFixture => "PickupFixture",
        HouseApiCall::MoveFixture => "MoveFixture",
        HouseApiCall::RotateFixtureClockwise => "RotateFixtureClockwise",
        HouseApiCall::RotateFixtureCounterClockwise => "RotateFixtureCounterClockwise",
        HouseApiCall::MoveFixtureVertically => "MoveFixtureVertically",
    }
}

fn outcome_json(outcome: &Result<piece_029::InteractionPlan, Refusal>) -> serde_json::Value {
    match outcome {
        Ok(plan) => serde_json::json!({
            "outcome": "plan",
            "fixture_guid": plan.fixture_guid,
            "fixture_definition_id": plan.fixture_definition_id,
            "item_definition_id": plan.item_definition_id,
            "api": api_name(plan.api),
            "gated_action": plan.gated_action,
            "requirement_id": plan.requirement_id,
        }),
        Err(refusal) => {
            let (name, action, requirement_id) = match refusal {
                Refusal::NoSelection => ("NoSelection", None, None),
                Refusal::UnknownFixture => ("UnknownFixture", None, None),
                Refusal::AmbiguousSelection => ("AmbiguousSelection", None, None),
                Refusal::UnknownDefinition => ("UnknownDefinition", None, None),
                Refusal::UnknownInteraction => ("UnknownInteraction", None, None),
                Refusal::ActionNotDeclared { action } => ("ActionNotDeclared", Some(*action), None),
                Refusal::UnknownRequirement { requirement_id } => {
                    ("UnknownRequirement", None, Some(*requirement_id))
                }
                Refusal::MembersOnly => ("MembersOnly", None, None),
                Refusal::NotAllowedToEdit => ("NotAllowedToEdit", None, None),
            };
            serde_json::json!({
                "outcome": "refusal",
                "refusal": name,
                "action": action,
                "requirement_id": requirement_id,
            })
        }
    }
}
