//! Work item 029: housing decoration discovery — the fixture-action gate between the shipped
//! client's decoration radial menu and the restored housing state machine.
//!
//! ORIGINAL facts implemented or parsed here (every one is read from a preserved source; nothing in
//! this module invents a number, a price or an effect):
//!
//! * Fixture action names are a shipped vocabulary. `Resources/StringHashToValue.txt`
//!   (sha256 `22003f3224dcc1cd5a189e6e818ab8c82711e713183f04dba13b76c255aeb570`) carries exactly
//!   three `Housing.*FixtureActionName` entries: `Elevate`, `MoveFixture`, `RemoveFixture`.
//! * `client-ui/ScriptsBase.lua`
//!   (sha256 `c7c71c7cbb177c975a7cdb12ed0c2e1b74d662f5ca5ef6a4e9ff86d2d332a8f6`) declares the five
//!   radial interaction ids (`Housing.INTERACTION_PICK_UP = 1` … `INTERACTION_VERTICAL_MOVE = 5`),
//!   maps each to its `House.*` call in `Housing:OnRadialMenuClick`, reads the addressee from
//!   `BaseClient.Housing.Instance.SelectedFixture` (`-1` means nothing is selected), and asks
//!   `House.IsActionPermitted(currentItemGuid, "MoveFixture")` / `"RemoveFixture"` before offering
//!   the move and remove buttons. `MakeFarmingRadialMenu` gates its whole menu on `MoveFixture`;
//!   `MakeHousingRadialMenu` computes `allowRemove` from `RemoveFixture` and then never uses it —
//!   a shipped-branch quirk that this module records rather than copies.
//! * The pinned community packet model
//!   (`FreeRealms-Legacy/src/Sanctuary.Packet.Common/Housing/FixtureDefinition.cs`) carries the
//!   definition fields the action gate consumes: a fixture definition id, its item definition id,
//!   `Category`, `LuaCall`, a map of `FixtureAction { Id, Name, RequirementId }` and a
//!   `CompositeEffectId`. The shipped client builds the same
//!   `SoeUtil::HashListMap<int, FixtureDefinition::FixtureAction, 16, -1>` container
//!   (`lab/decompiled/client/FreeRealms/pseudocode.c:1402661`) and identifies a placed fixture by
//!   fixture guid (`SoeUtilSignatureFixtureGuid`, `HousingPacketFixtureRemove`,
//!   `Selected House Fixture: %lld` at `pseudocode.c:1397564`).
//! * `Resources/Requirements.txt`
//!   (sha256 `1dc2f97fd35fcc20b55213ba8f08e93c43c84f442c139f08c0c816d5f7012644`) is the shipped
//!   requirement table a fixture action's `RequirementId` resolves against: 43 rows over six
//!   factories (`CheckRankLastProfileByType`, `AlwaysFails`, `SourceVipRank`, `CheckCurrentProfile`,
//!   `CheckRenamePending`, `CheckNudge`).
//!
//! RECONSTRUCTED policy (this module's own bounded rules, not recovered server code):
//!
//! * an interaction is addressed by fixture guid; an item-definition address is accepted only when
//!   it resolves to exactly one instance of the lot, because two identical decorations share an item
//!   definition and "first match wins" would act on the wrong instance;
//! * `RequirementId` 0 means "no requirement declared"; any other id must exist in the shipped
//!   requirement table, otherwise the action is refused instead of allowed;
//! * requirement *evaluation* stays with the caller. The evaluator factories are client-side
//!   `CODE_FACTORY_NAME`s whose operator semantics are not recovered, so this module carries the id
//!   and refuses only what it cannot even name;
//! * the gate reports the most specific named gate of the request (`MoveFixture`, `RemoveFixture` or
//!   none) and never applies an effect itself.
//!
//! Explicit gaps, not implemented and not implied: the server-side fixture-definition table (which
//! definition declares which actions, and its `LuaCall` text) is unrecovered, so definitions are
//! caller input here; the `Elevate` name is shipped but no recovered Lua call site passes it to
//! `IsActionPermitted`, so vertical movement is ungated in this module; `CompositeEffectId` handling,
//! per-character `Fixture::EffectStorageNode` effects and seating are recovered as client-side
//! structures but are not simulated. This module does not write a snapshot, mint an item or grant
//! currency.
//!
//! ponytail: borrowed slices and one linear pass per lookup. Two scans over a lot's fixture list is
//! cheaper than an index that would need invalidation on every placement.

/// ORIGINAL: `Housing.INTERACTION_PICK_UP` — the radial menu's remove/pick-up button.
pub const INTERACTION_PICK_UP: i32 = 1;
/// ORIGINAL: `Housing.INTERACTION_MOVE`.
pub const INTERACTION_MOVE: i32 = 2;
/// ORIGINAL: `Housing.INTERACTION_ROTATE_LEFT`.
pub const INTERACTION_ROTATE_LEFT: i32 = 3;
/// ORIGINAL: `Housing.INTERACTION_ROTATE_RIGHT`.
pub const INTERACTION_ROTATE_RIGHT: i32 = 4;
/// ORIGINAL: `Housing.INTERACTION_VERTICAL_MOVE`.
pub const INTERACTION_VERTICAL_MOVE: i32 = 5;

/// ORIGINAL: the value the client sends in `BaseClient.Housing.Instance.SelectedFixture` when no
/// fixture instance is selected (the radial menu is hidden at `-1`).
pub const NO_FIXTURE_SELECTED: i64 = -1;

/// ORIGINAL: `Housing.ElevateFixtureActionName` in the shipped `StringHashToValue.txt`.
pub const FIXTURE_ACTION_ELEVATE: &str = "Elevate";
/// ORIGINAL: `Housing.MoveFixtureActionName` in the shipped `StringHashToValue.txt`.
pub const FIXTURE_ACTION_MOVE: &str = "MoveFixture";
/// ORIGINAL: `Housing.RemoveFixtureActionName` in the shipped `StringHashToValue.txt`.
pub const FIXTURE_ACTION_REMOVE: &str = "RemoveFixture";

/// ORIGINAL: config key to action name, in the order the shipped file lists them.
pub const ACTION_NAME_KEYS: [(&str, &str); 3] = [
    ("Housing.ElevateFixtureActionName", FIXTURE_ACTION_ELEVATE),
    ("Housing.MoveFixtureActionName", FIXTURE_ACTION_MOVE),
    ("Housing.RemoveFixtureActionName", FIXTURE_ACTION_REMOVE),
];

/// ORIGINAL: the named fixture actions the shipped client knows, sorted for comparison.
pub fn fixture_action_names() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = ACTION_NAME_KEYS.iter().map(|(_, name)| *name).collect();
    names.sort_unstable();
    names
}

/// ORIGINAL: the `House.*` Lua call the client makes for one radial interaction
/// (`Housing:OnRadialMenuClick`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HouseApiCall {
    PickupFixture,
    MoveFixture,
    RotateFixtureClockwise,
    RotateFixtureCounterClockwise,
    MoveFixtureVertically,
}

impl HouseApiCall {
    /// The recovered Lua call name, spelled as the shipped `ScriptsBase.lua` spells it.
    pub fn lua_call(self) -> &'static str {
        match self {
            HouseApiCall::PickupFixture => "House.PickupFixture",
            HouseApiCall::MoveFixture => "House.MoveFixture",
            HouseApiCall::RotateFixtureClockwise => "House.RotateFixtureClockwise",
            HouseApiCall::RotateFixtureCounterClockwise => "House.RotateFixtureCounterClockwise",
            HouseApiCall::MoveFixtureVertically => "House.MoveFixtureVertically",
        }
    }

    /// The recovered interaction id that issues this call.
    pub fn interaction(self) -> i32 {
        match self {
            HouseApiCall::PickupFixture => INTERACTION_PICK_UP,
            HouseApiCall::MoveFixture => INTERACTION_MOVE,
            HouseApiCall::RotateFixtureCounterClockwise => INTERACTION_ROTATE_LEFT,
            HouseApiCall::RotateFixtureClockwise => INTERACTION_ROTATE_RIGHT,
            HouseApiCall::MoveFixtureVertically => INTERACTION_VERTICAL_MOVE,
        }
    }
}

/// ORIGINAL: the client picks a different radial menu and a different gate set in a factory/farm
/// lot (`Housing:MakeFarmingRadialMenu`, driven by `House.GetIsInFactory`) than in a house lot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientArea {
    Housing,
    Farming,
}

/// ORIGINAL shape of `FixtureDefinition::FixtureAction.Name` plus its `RequirementId`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureActionDefinition {
    pub name: String,
    pub requirement_id: i32,
}

/// ORIGINAL fields of a fixture definition that the action gate consumes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureDefinitionRow {
    pub fixture_definition_id: i32,
    pub item_definition_id: i32,
    /// Caller input: the client asks `House.IsFixtureMembersOnly(guid)`; a non-member is offered
    /// only the removal button for such a fixture.
    pub members_only: bool,
    pub actions: Vec<FixtureActionDefinition>,
}

/// ORIGINAL identity of a placed fixture: the guid is the instance identity, the item definition id
/// is only the definition it was built from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlacedFixtureRow {
    pub fixture_guid: u64,
    pub lot_guid: u64,
    pub fixture_definition_id: i32,
    pub item_definition_id: i32,
}

/// How the client addressed the fixture it wants to act on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selection {
    FixtureGuid(u64),
    /// The wire also carries item definition ids (fixture item lists, preview requests); such an
    /// address is only usable when it names one instance.
    ItemDefinition(i32),
    None,
}

/// The client sends the selected fixture guid as a signed value; `-1` means "nothing selected".
pub fn selection_from_guid(guid: i64) -> Selection {
    if guid == NO_FIXTURE_SELECTED {
        Selection::None
    } else {
        Selection::FixtureGuid(guid as u64)
    }
}

/// A refusal, never an effect. Each variant is a distinct reason a decoration interaction does not
/// proceed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The client addressed `-1`: there is no instance to act on.
    NoSelection,
    /// No instance of this lot matches the address.
    UnknownFixture,
    /// More than one instance of this lot matches the address; acting would be a guess.
    AmbiguousSelection,
    /// The fixture's definition never arrived, so its actions are unknown.
    UnknownDefinition,
    /// The interaction id is not one of the five recovered radial interactions.
    UnknownInteraction,
    /// The fixture's definition does not declare the named action the client is asking about.
    ActionNotDeclared { action: &'static str },
    /// The declared action names a requirement the shipped table does not contain.
    UnknownRequirement { requirement_id: i32 },
    /// A non-member addressed a members-only fixture with anything but removal.
    MembersOnly,
    /// The lot is read-only for this character (`HousingInstance_DS.CanEdit`).
    NotAllowedToEdit,
}

/// The gate's output: which instance, which call, which named gate. It carries no effect and no
/// economy field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InteractionPlan {
    pub lot_guid: u64,
    pub fixture_guid: u64,
    pub fixture_definition_id: i32,
    pub item_definition_id: i32,
    pub interaction: i32,
    pub api: HouseApiCall,
    /// The most specific named gate of this request, when the recovered client names one.
    pub gated_action: Option<&'static str>,
    /// The requirement id carried by that gate's action, when it declares one.
    pub requirement_id: Option<i32>,
}

/// One interaction request, borrowed from the caller's lot, catalogue and requirement table.
#[derive(Clone, Debug)]
pub struct InteractionRequest<'a> {
    pub lot_guid: u64,
    pub selection: Selection,
    pub interaction: i32,
    pub area: ClientArea,
    pub is_member: bool,
    pub can_edit: bool,
    pub fixtures: &'a [PlacedFixtureRow],
    pub definitions: &'a [FixtureDefinitionRow],
    /// The requirement ids from the shipped `Requirements.txt` table (or the caller's equivalent).
    pub requirement_ids: &'a [i32],
}

/// The recovered `House.*` call for a radial interaction id.
pub fn interaction_api(interaction: i32) -> Option<HouseApiCall> {
    match interaction {
        INTERACTION_PICK_UP => Some(HouseApiCall::PickupFixture),
        INTERACTION_MOVE => Some(HouseApiCall::MoveFixture),
        INTERACTION_ROTATE_LEFT => Some(HouseApiCall::RotateFixtureCounterClockwise),
        INTERACTION_ROTATE_RIGHT => Some(HouseApiCall::RotateFixtureClockwise),
        INTERACTION_VERTICAL_MOVE => Some(HouseApiCall::MoveFixtureVertically),
        _ => None,
    }
}

/// The named actions this request must be permitted for, least specific first.
///
/// ORIGINAL: `MakeFarmingRadialMenu` wraps its whole menu in `IsActionPermitted(guid,
/// "MoveFixture")` and adds the remove button only behind `RemoveFixture`; `MakeHousingRadialMenu`
/// asks about `RemoveFixture` only, and the shipped build then drops that answer.
///
/// RECONSTRUCTED: this gate keeps the stricter reading for both areas — an interaction whose action
/// would change the lot requires the fixture definition to declare that action, even where the
/// shipped UI branch forgot to ask. `VERTICAL_MOVE` stays ungated because no recovered call site
/// passes `Elevate` to `IsActionPermitted`.
fn required_actions(interaction: i32, area: ClientArea) -> Vec<&'static str> {
    match (area, interaction) {
        (ClientArea::Farming, INTERACTION_PICK_UP) => {
            vec![FIXTURE_ACTION_MOVE, FIXTURE_ACTION_REMOVE]
        }
        (ClientArea::Farming, INTERACTION_MOVE)
        | (ClientArea::Farming, INTERACTION_ROTATE_LEFT)
        | (ClientArea::Farming, INTERACTION_ROTATE_RIGHT) => vec![FIXTURE_ACTION_MOVE],
        (ClientArea::Housing, INTERACTION_PICK_UP) => vec![FIXTURE_ACTION_REMOVE],
        (ClientArea::Housing, INTERACTION_MOVE) => vec![FIXTURE_ACTION_MOVE],
        _ => Vec::new(),
    }
}

/// Resolves the addressed instance inside `lot_guid`.
fn resolve<'a>(
    lot_guid: u64,
    selection: Selection,
    fixtures: &'a [PlacedFixtureRow],
) -> Result<&'a PlacedFixtureRow, Refusal> {
    match selection {
        Selection::None => Err(Refusal::NoSelection),
        Selection::FixtureGuid(guid) => {
            let mut matches = fixtures
                .iter()
                .filter(|row| row.lot_guid == lot_guid && row.fixture_guid == guid);
            let first = matches.next().ok_or(Refusal::UnknownFixture)?;
            if matches.next().is_some() {
                return Err(Refusal::AmbiguousSelection);
            }
            Ok(first)
        }
        Selection::ItemDefinition(item_definition_id) => {
            let mut matches = fixtures.iter().filter(|row| {
                row.lot_guid == lot_guid && row.item_definition_id == item_definition_id
            });
            let first = matches.next().ok_or(Refusal::UnknownFixture)?;
            if matches.next().is_some() {
                return Err(Refusal::AmbiguousSelection);
            }
            Ok(first)
        }
    }
}

/// Validates one decoration interaction and reports the plan the caller may execute.
///
/// Order of refusal is: edit permission, instance selection, interaction id, definition presence,
/// members-only policy, declared action, known requirement.
pub fn plan_interaction(request: &InteractionRequest<'_>) -> Result<InteractionPlan, Refusal> {
    if !request.can_edit {
        return Err(Refusal::NotAllowedToEdit);
    }

    let fixture = resolve(request.lot_guid, request.selection, request.fixtures)?;

    let api = interaction_api(request.interaction).ok_or(Refusal::UnknownInteraction)?;

    let definition = request
        .definitions
        .iter()
        .find(|row| row.fixture_definition_id == fixture.fixture_definition_id)
        .ok_or(Refusal::UnknownDefinition)?;

    if definition.members_only && !request.is_member && request.interaction != INTERACTION_PICK_UP {
        return Err(Refusal::MembersOnly);
    }

    let required = required_actions(request.interaction, request.area);
    let mut gated_action = None;
    let mut requirement_id = None;
    for action_name in required {
        let declared = definition
            .actions
            .iter()
            .find(|entry| entry.name == action_name)
            .ok_or(Refusal::ActionNotDeclared {
                action: action_name,
            })?;
        if declared.requirement_id < 0
            || (declared.requirement_id > 0
                && !request.requirement_ids.contains(&declared.requirement_id))
        {
            return Err(Refusal::UnknownRequirement {
                requirement_id: declared.requirement_id,
            });
        }
        gated_action = Some(action_name);
        requirement_id = if declared.requirement_id > 0 {
            Some(declared.requirement_id)
        } else {
            None
        };
    }

    Ok(InteractionPlan {
        lot_guid: request.lot_guid,
        fixture_guid: fixture.fixture_guid,
        fixture_definition_id: fixture.fixture_definition_id,
        item_definition_id: fixture.item_definition_id,
        interaction: request.interaction,
        api,
        gated_action,
        requirement_id,
    })
}

/// ORIGINAL: `Housing.*FixtureActionName` rows of `Resources/StringHashToValue.txt`, sorted by key.
///
/// The shipped file is `#STRING^VALUE^` per row; only the three fixture-action keys are kept.
pub fn extract_action_name_entries(text: &str) -> Vec<(String, String)> {
    let mut entries: Vec<(String, String)> = text
        .lines()
        .filter_map(|line| {
            let mut parts = line.trim().split('^');
            let key = parts.next()?.to_owned();
            let value = parts.next()?.to_owned();
            (key.ends_with("FixtureActionName") && !value.is_empty()).then_some((key, value))
        })
        .collect();
    entries.sort();
    entries
}

/// ORIGINAL: the `INTERACTION_*` constants of the shipped `Housing` table in `ScriptsBase.lua`,
/// sorted by id.
///
/// The table is written as `Housing = { INTERACTION_PICK_UP = 1, … }`, so the row text carries no
/// `Housing.` prefix; the neighbouring `INTERACTION_OPTION_NAME` row is a string and is skipped.
pub fn extract_interaction_ids(lua: &str) -> Vec<(String, i32)> {
    let mut found: Vec<(String, i32)> = lua
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let rest = line.strip_prefix("INTERACTION_")?;
            let (name, value) = rest.split_once('=')?;
            let value: i32 = value.trim().trim_end_matches(',').trim().parse().ok()?;
            Some((format!("INTERACTION_{}", name.trim()), value))
        })
        .collect();
    found.sort_by_key(|(_, value)| *value);
    found
}

/// ORIGINAL action names the shipped Lua passes to `House.IsActionPermitted`, sorted and unique.
pub fn extract_lua_action_requests(lua: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for chunk in lua.split("IsActionPermitted").skip(1) {
        let Some(open) = chunk.find('"') else {
            continue;
        };
        let Some(close) = chunk[open + 1..].find('"') else {
            continue;
        };
        let name = &chunk[open + 1..open + 1 + close];
        if !names.iter().any(|existing| existing == name) {
            names.push(name.to_owned());
        }
    }
    names.sort();
    names
}

/// ORIGINAL: requirement ids of `Resources/Requirements.txt`, sorted; header row ignored.
pub fn requirement_ids(text: &str) -> Vec<i32> {
    let mut ids: Vec<i32> = text
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .filter_map(|line| line.split('^').next()?.trim().parse().ok())
        .collect();
    ids.sort_unstable();
    ids
}

/// ORIGINAL: `CODE_FACTORY_NAME` column of `Resources/Requirements.txt`, sorted and unique.
pub fn requirement_factories(text: &str) -> Vec<String> {
    let mut factories: Vec<String> = Vec::new();
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('^').collect();
        let Some(factory) = fields.get(2) else {
            continue;
        };
        if factory.is_empty() {
            continue;
        }
        let factory = (*factory).to_owned();
        if !factories.contains(&factory) {
            factories.push(factory);
        }
    }
    factories.sort();
    factories
}
