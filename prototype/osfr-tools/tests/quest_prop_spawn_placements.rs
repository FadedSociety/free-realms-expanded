//! T739: the clickable quest props the resolved reference spawns name, drawn where the spawn says.
//!
//! `lab/references/quest-lines/wiring/spawn-objects.json` matched 23 quest objectives to one
//! reference spawn each. Sixteen already sit on a drawn NPC or prop body (listed in
//! `notes/Autopilot/T739.md` with the reason each was skipped); the tree stump's spawn is skipped
//! too, because its cited model is not shipped by this client and its position is a mining
//! cluster far from the Greenwood Forest lake camp the quest text names. This slice places the
//! remaining four props (Planter, Treasure Chest, Sanctuary Postboxes, 1 Way Back Way — six
//! objectives) as `legacy_data` regional rows, so the zone quest briefs can bind their
//! objectives.
//!
//! The wire path is unchanged: `NpcVisibility::load` draws a regional `legacy_data` row as its
//! own actor and `update` sends the 35/2 AddNpc the client reads (reader `0x00921a10`: u64
//! actor id, i32 name id, i32 model id first, then the transform parsed by `parse_add_npc`).
//! The click path is `gameplay/npc.rs::npc_interact_26_8` -> `NpcInteraction::interact`
//! (`check_range`, 12 m) -> the 26/9 menu; a refused click never opens one.
//!
//! Every value compared here is read from the content row, the cited `FabledRealms.lua` line,
//! that source's `Resources/Npcs.json`, the client locale table or the client `Models.txt`; the
//! only pinned literals are the brief's provenance marker and the 1 m duplicate rule.

#[path = "support/models.rs"]
mod models;
#[path = "common/npc_greetings.rs"]
mod npc_greetings;
#[path = "support/npc_visibility.rs"]
mod npc_visibility;
#[path = "support/placement.rs"]
mod placement;

use npc_greetings::{
    GREETING_HEADER, MENU_HEADER, character_at, content_dir, gameplay_dir, interact,
    locale_id_to_text, select, service,
};
use npc_visibility::seen_from;
use osfr_tools::{gameplay::Gameplay, npc_interaction::NpcInteractionError, world::NpcVisibility};
use placement::{
    cited_spawn, distance, has_source_kind, normalize_name, reference_npcs_row, regional_rows, xyz,
    xyz_f64,
};
use serde_json::Value;
use std::collections::BTreeSet;

/// The provenance phrase T739 requires on every row it places.
const MARKER: &str = "matched by spawnobj-infer (R)";
/// Two drawn actors closer than this are the same standing spot (the brief's 1 m rule).
const SAME_SPOT_METRES: f64 = 1.0;

/// The rows this brief placed: the regional rows whose `position_reason` carries its marker.
fn placed_props() -> Vec<Value> {
    regional_rows()
        .into_iter()
        .filter(|row| {
            row["position_reason"]
                .as_str()
                .is_some_and(|reason| reason.contains(MARKER))
        })
        .collect()
}

#[test]
fn every_placed_prop_is_drawn_at_its_cited_spawn_and_clickable() {
    let rows = placed_props();
    assert_eq!(
        rows.len(),
        4,
        "T739 places four clickable props; a row lost its spawnobj marker"
    );
    let locale = locale_id_to_text();
    let models = models::read_models();
    let base = NpcVisibility::load(&content_dir()).unwrap();
    let mut service = service();
    let mut ids = BTreeSet::new();
    for row in &rows {
        let id = row["npc_id"].as_u64().unwrap();
        let owner = format!("{id} ({})", row["name"]);
        assert!(ids.insert(id), "{owner}: one placed prop per actor id");
        assert_eq!(row["placement"], "legacy_data", "{owner}: placement");
        assert_eq!(
            row["placement_source"], "legacy_data_spawn",
            "{owner}: placement source"
        );
        assert_eq!(
            row["placement_confidence"], "medium",
            "{owner}: placement confidence"
        );
        assert!(
            row["placement_candidates"].as_array().unwrap().is_empty(),
            "{owner}: placement candidates"
        );

        // The row's own citation is the independent witness: it must name the transform and the
        // reference body the row stores, or the row was moved away from the evidence it cites.
        let reason = row["position_reason"].as_str().unwrap();
        let spawn = cited_spawn(reason);
        assert!(
            reason.contains(&format!("{}:{}", spawn.source, spawn.line)),
            "{owner}: reason does not cite its line"
        );
        let authored = xyz_f64(&row["position"]);
        for axis in 0..3 {
            assert!(
                (authored[axis] - spawn.position[axis]).abs() <= 1e-9,
                "{owner}: position axis {axis} differs from {}:{}",
                spawn.source,
                spawn.line
            );
        }
        let position = xyz(&row["position"]);
        let heading = row["heading"].as_f64().unwrap();
        assert!(
            (heading - spawn.heading).abs() <= 1e-9,
            "{owner}: heading differs from {}:{}",
            spawn.source,
            spawn.line
        );

        // The cited reference row and the client's own model and locale tables.
        let reference = reference_npcs_row(&spawn.source, spawn.ref_id);
        assert_eq!(
            row["model_id"].as_u64(),
            reference["ModelId"].as_u64(),
            "{owner}: model id"
        );
        assert_eq!(
            row["model_file_name"].as_str(),
            reference["ModelFileName"].as_str(),
            "{owner}: model file"
        );
        let name_id = reference["NameId"].as_u64().unwrap();
        let locale_name = locale
            .get(&name_id)
            .unwrap_or_else(|| panic!("{owner}: spawn NameId {name_id} does not resolve"));
        assert_eq!(
            normalize_name(row["name"].as_str().unwrap()),
            normalize_name(locale_name),
            "{owner}: spawn NameId {name_id} does not name this row"
        );
        assert_eq!(
            row["name_string_id"].as_u64(),
            Some(name_id),
            "{owner}: the row binds a name id that is not the spawn's NameId"
        );
        let model_id = row["model_id"].as_u64().unwrap();
        let model_file = row["model_file_name"].as_str().unwrap();
        assert!(
            models
                .get(&i64::try_from(model_id).unwrap())
                .is_some_and(|files| files.contains(&model_file.to_ascii_lowercase())),
            "{owner}: Models.txt does not map model {model_id} to {model_file}"
        );

        // The content rotation is the spawn heading as a Y-axis quaternion.
        let rotation = row["rotation"].as_array().unwrap();
        let yaw = 2.0
            * rotation[1]
                .as_f64()
                .unwrap()
                .atan2(rotation[3].as_f64().unwrap());
        assert!(
            (yaw - heading).abs() <= 1e-9,
            "{owner}: rotation is not the spawn heading"
        );
        assert!(
            has_source_kind(&row["field_sources"]["position"], "legacy-data"),
            "{owner}: position lacks Legacy provenance"
        );
        assert!(
            has_source_kind(&row["source"], "legacy-data"),
            "{owner}: source lacks a Legacy tag"
        );

        // The objective ids the zone briefs bind, noted on the row itself.
        let objectives = row["objective_ids"]
            .as_array()
            .unwrap_or_else(|| panic!("{owner}: objective_ids is missing"));
        assert!(!objectives.is_empty(), "{owner}: no objective id is noted");
        for objective in objectives {
            let objective = objective.as_str().unwrap();
            assert!(
                objective.starts_with('q') && objective.contains("-o"),
                "{owner}: {objective:?} is not a quest objective id"
            );
            assert!(
                reason.contains(objective),
                "{owner}: position reason does not name {objective}"
            );
        }

        // The production loader draws the row, and the 35/2 body the client receives carries
        // the row's own identity, model and place.
        assert_eq!(base.content_key(id), Some(id), "{owner}: content key");
        assert_eq!(
            base.actor_position(id),
            Some(position),
            "{owner}: loader position"
        );
        let seen = seen_from(&base, position);
        let drawn: Vec<_> = seen.iter().filter(|npc| npc.actor_id == id).collect();
        assert_eq!(drawn.len(), 1, "{owner}: exactly one drawn actor");
        assert_eq!(
            drawn[0].model_id,
            i32::try_from(model_id).unwrap(),
            "{owner}: AddNpc model"
        );
        assert_eq!(
            drawn[0].name_string_id,
            i32::try_from(name_id).unwrap(),
            "{owner}: AddNpc name id"
        );
        for axis in 0..3 {
            assert!(
                (drawn[0].position[axis] - position[axis]).abs() <= 1e-3,
                "{owner}: AddNpc position axis {axis}"
            );
        }

        // A click on the drawn actor opens the menu the router sends, and a click from outside
        // the range is refused without disturbing it.
        let (zone, row_position) = service
            .location(id)
            .unwrap_or_else(|| panic!("{owner}: the NPC service does not load this row"));
        let menu = service
            .interact(id, zone, row_position, None)
            .unwrap_or_else(|error| panic!("{owner}: a click is refused: {error:?}"));
        assert!(
            menu.packet.starts_with(&MENU_HEADER),
            "{owner}: a click must answer with the 26/9 menu"
        );
        let far = [row_position[0] + 20.0, row_position[1], row_position[2]];
        assert_eq!(
            service.interact(id, zone, far, None).unwrap_err(),
            NpcInteractionError::OutOfRange,
            "{owner}: an out-of-range click is not refused"
        );
        let again = service.interact(id, zone, row_position, None).unwrap();
        assert_eq!(
            again, menu,
            "{owner}: a refused click changed the open menu"
        );
    }
}

/// The brief's duplicate rule for the rows it placed: no other drawn actor (regional row,
/// extra body or placement) stands within 1 m of a placed prop. The global regional-row
/// pairwise check is `npc_reference_spawn_placements::no_two_drawn_regional_npc_rows_share_a_position`.
#[test]
fn no_placed_prop_shares_a_spot_with_another_drawn_actor() {
    let rows = placed_props();
    assert!(!rows.is_empty(), "no row carries the spawnobj marker");
    let base = NpcVisibility::load(&content_dir()).unwrap();
    for row in &rows {
        let id = row["npc_id"].as_u64().unwrap();
        let position = xyz(&row["position"]);
        let seen = seen_from(&base, position);
        assert!(
            seen.iter().any(|npc| npc.actor_id == id),
            "placed prop {id} is not drawn"
        );
        for other in seen.iter().filter(|npc| npc.actor_id != id) {
            let apart = distance(
                position,
                [other.position[0], other.position[1], other.position[2]],
            );
            assert!(
                apart > SAME_SPOT_METRES,
                "prop {id} shares its spot with actor {} ({apart:.2} m)",
                other.actor_id
            );
        }
    }
}

/// The real dispatch: `Gameplay::load` reads these bundled rows when the content package has no
/// `npcs/` of its own (`crate::content::npc_content`), and `Gameplay::handle_in_world` routes the
/// client's 26/8 click to `gameplay/npc.rs::npc_interact_26_8` -> `NpcInteraction`. A placed prop
/// answers with the 26/9 menu, Talk answers with 26/3, and a click from 20 m is refused without
/// changing the saved state.
#[test]
fn a_click_on_a_placed_prop_reaches_the_npc_router() {
    let rows = placed_props();
    assert!(!rows.is_empty(), "no row carries the spawnobj marker");
    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    for row in &rows {
        let id = row["npc_id"].as_u64().unwrap();
        let owner = format!("{id} ({})", row["name"]);
        let position = xyz(&row["position"]);
        let menu = game
            .handle_in_world(&interact(id), &character_at(position), 1_000)
            .unwrap_or_else(|error| panic!("{owner}: the router refused a click: {error}"));
        assert!(
            menu.replies
                .iter()
                .any(|packet| packet.starts_with(&MENU_HEADER)),
            "{owner}: a click must answer with the 26/9 menu"
        );

        // A refused click leaves the saved state alone (no menu, quest or counter consumed).
        let before = game.state();
        let far = character_at([position[0] + 20.0, position[1], position[2]]);
        assert!(
            game.handle_in_world(&interact(id), &far, 1_100).is_err(),
            "{owner}: a click from 20 m is accepted"
        );
        assert_eq!(
            game.state(),
            before,
            "{owner}: a refused click changed the saved state"
        );

        let talk = game
            .handle_in_world(&select(id, 1), &character_at(position), 1_200)
            .unwrap_or_else(|error| panic!("{owner}: Talk was refused: {error}"));
        assert!(
            talk.replies
                .iter()
                .any(|packet| packet.starts_with(&GREETING_HEADER)),
            "{owner}: Talk must answer with 26/3"
        );
    }
}
