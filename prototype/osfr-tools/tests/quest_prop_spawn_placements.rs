//! T739: the clickable quest props the resolved reference spawns name, drawn where the spawn says.
//!
//! `lab/references/quest-lines/wiring/spawn-objects.json` matched 23 quest objectives to one
//! reference spawn each. Sixteen already sit on a drawn NPC or prop body (listed in
//! `notes/Autopilot/T739.md` with the reason each was skipped); the tree stump's spawn is skipped
//! too, because its cited model is not shipped by this client and its position is a mining
//! cluster far from the Greenwood Forest lake camp the quest text names. This slice places the
//! remaining four props (Planter, Treasure Chest, Sanctuary Postboxes, 1 Way Back Way — six
//! objectives) as `legacy_data` regional rows, so the zone quest briefs can bind their objectives.
//!
//! The wire path is unchanged: `NpcVisibility::load` draws a regional `legacy_data` row as its
//! own actor and `update` sends the 35/2 AddNpc the client reads (reader `0x00921a10`: u64
//! actor id, i32 name id, i32 model id first, then the transform parsed by `parse_add_npc`).
//! The click path is `gameplay/npc.rs::npc_interact_26_8` -> `NpcInteraction::interact`
//! (`check_range`, 12 m) -> the 26/9 menu; a refused click never opens one.
//!
//! Every value compared here is read from the content row, the cited `FabledRealms.lua` line,
//! that source's `Resources/Npcs.json`, the client locale table or the client `Models.txt`; the
//! only pinned literals are the brief's provenance marker and the 1 m duplicate rule. The row
//! placement assertions are shared with T698 (`support/placement.rs`).

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
    locale_id_to_text, read_json, select, service,
};
use npc_visibility::seen_from;
use osfr_tools::{gameplay::Gameplay, npc_interaction::NpcInteractionError, world::NpcVisibility};
use placement::{assert_placed_row_matches_cited_spawn, distance, has_source_kind, xyz};
use serde_json::Value;
use std::collections::BTreeSet;

/// The provenance phrase T739 requires on every row it places.
const MARKER: &str = "matched by spawnobj-infer (R)";
/// Two drawn actors closer than this are the same standing spot (the brief's 1 m rule).
const SAME_SPOT_METRES: f64 = 1.0;
/// The zone files this brief's rows live in. A later spawnobj brief may use the same phrase in
/// another zone file, so the scan stays scoped here and the count pin covers only these two.
const BRIEF_ZONES: [&str; 2] = ["sanctuary", "snowhill"];

/// The rows this brief placed: in its zone files, `position_reason` carries its marker.
fn placed_props() -> Vec<Value> {
    let mut rows = Vec::new();
    for zone in BRIEF_ZONES {
        let path = content_dir().join("npcs").join(format!("{zone}.json"));
        for row in read_json(&path)["records"].as_array().unwrap() {
            if row["position_reason"]
                .as_str()
                .is_some_and(|reason| reason.contains(MARKER))
            {
                rows.push(row.clone());
            }
        }
    }
    rows
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
        assert_placed_row_matches_cited_spawn(row, &locale, &models, &base);

        // The objective ids the zone briefs bind are the lead's spawnobj-infer join, not the
        // spawn line: neither `FabledRealms.lua` nor `Npcs.json` carries an objective id, so
        // the field source must stay reconstructed and cite the join.
        let reason = row["position_reason"].as_str().unwrap();
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
        let sources = &row["field_sources"]["objective_ids"];
        assert!(
            has_source_kind(sources, "reconstructed"),
            "{owner}: objective_ids provenance is not the (R) spawnobj-infer join"
        );
        assert!(
            !has_source_kind(sources, "legacy-data"),
            "{owner}: objective_ids cites the spawn line, which carries no objective ids"
        );
        let evidence: String = sources
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|tag| tag["evidence"].as_str())
            .collect();
        assert!(
            evidence.contains("spawn-objects.json") && evidence.contains("spawnobj-infer"),
            "{owner}: objective_ids evidence does not cite the spawnobj-infer join"
        );

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
