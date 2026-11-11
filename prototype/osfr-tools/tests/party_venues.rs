//! PB3: the shipped party venue table (`data/content/party_venues.json`) against the client and
//! the world it names. Each planner is an NPC the open world already draws, where the venue
//! says; every row id is a client row (locale lines, cake and boombox items, decoration models in
//! the players' packs); the Pixiewood spots stand on level baked terrain; the two raised venues,
//! whose floors the baked collision does not hold, say so in their source (R). Reads the private
//! lab terrain, model table, asset catalogue and locale export.
#[path = "support/client_assets.rs"]
mod client_assets;
#[path = "support/models.rs"]
mod models;
#[path = "support/npc_visibility.rs"]
mod npc_visibility;
#[path = "support/placement.rs"]
mod placement;
#[path = "support/terrain.rs"]
mod terrain;

use osfr_tools::boombox;
use osfr_tools::party_planner::Venues;
use osfr_tools::placed_cakes;
use osfr_tools::world::NpcVisibility;
use std::collections::BTreeMap;

fn venues() -> Venues {
    Venues::load(&npc_visibility::content_dir()).unwrap()
}

fn locale() -> BTreeMap<u32, String> {
    let path = placement::project_parent().join("lab/reports/locale-id-to-text.json");
    let rows: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    (rows.as_array().unwrap().iter())
        .filter_map(|row| Some((row["id"].as_u64()? as u32, row["text"].as_str()?.to_owned())))
        .collect()
}

#[test]
fn pb3_each_planner_is_a_drawn_npc_where_its_venue_says() {
    let drawn =
        placement::drawn_npcs(&NpcVisibility::load(&npc_visibility::content_dir()).unwrap());
    for venue in venues().iter() {
        let body = drawn
            .iter()
            .find(|npc| npc.actor == venue.planner)
            .unwrap_or_else(|| panic!("{}: planner {} is not drawn", venue.key, venue.planner));
        let off = placement::distance(body.position, venue.planner_at);
        assert!(
            off < 0.01,
            "{}: the planner stands {off:.3} m away",
            venue.key
        );
        // Exactly one body: binding a click must not have drawn a second planner.
        assert_eq!(
            drawn
                .iter()
                .filter(|npc| npc.actor == venue.planner)
                .count(),
            1
        );
    }
}

#[test]
fn pb3_spots_stand_on_their_venue_floor() {
    let ground = terrain::load();
    let grid = ground.grid(0).expect("the open world grid");
    for venue in venues().iter() {
        let raised = venue.key != "pixiewood";
        for [x, y, z, heading] in &venue.spots {
            let at = [*x, *y, *z];
            assert!(
                placement::planar_distance(at, venue.centre) <= 12.0,
                "{}: {at:?}",
                venue.key
            );
            assert!(heading.is_finite());
            let floor = grid.height_at(*x, *z).expect("terrain under the spot");
            if raised {
                // A tree-house deck or a tower top, tens of metres above the ground under it.
                assert!(
                    y - floor > 10.0,
                    "{}: {at:?} is {:.1} m up",
                    venue.key,
                    y - floor
                );
                assert!(
                    (y - venue.centre[1]).abs() <= 0.5,
                    "{}: one floor",
                    venue.key
                );
            } else {
                assert!(
                    (y - floor).abs() <= 0.5,
                    "{}: {at:?} vs ground {floor:.2}",
                    venue.key
                );
                let slope = terrain::slope_degrees(grid, *x, *z);
                assert!(
                    slope <= 25.0,
                    "{}: {at:?} slopes {slope:.1} degrees",
                    venue.key
                );
            }
        }
        let marked = venue
            .source
            .iter()
            .any(|line| line.contains("(R) the floor height"));
        assert_eq!(
            marked, raised,
            "{}: a raised floor is marked (R)",
            venue.key
        );
        let props = venue
            .cakes
            .iter()
            .chain(&venue.decorations)
            .chain([&venue.boombox]);
        for prop in props {
            let at = [prop.at[0], prop.at[1], prop.at[2]];
            assert!(
                placement::planar_distance(at, venue.centre) <= 12.0,
                "{}: {at:?}",
                venue.key
            );
        }
    }
}

#[test]
fn pb3_every_venue_row_is_a_client_row() {
    let content = npc_visibility::content_dir();
    let cakes = placed_cakes::Catalogue::load(&content).unwrap();
    let boxes = boombox::Catalogue::load(&content).unwrap();
    let models = models::read_models();
    let assets = client_assets::Assets::open().expect("the extracted client assets");
    let names = assets.names();
    let text = locale();
    let lines = [
        ("pixiewood", 422_717, "best spot in all of Sanctuary"),
        (
            "lakeshore_treehouse",
            422_818,
            "hottest party in all of Lakeshore",
        ),
        ("tanglewood_towers", 422_690, "Cake will be served"),
    ];
    let tickets = [422_707, 422_705, 422_667];
    for (venue, ((key, line, words), ticket)) in venues().iter().zip(lines.iter().zip(tickets)) {
        assert_eq!(venue.key, *key);
        assert_eq!(venue.greeting_id, *line);
        assert!(text[line].contains(words), "{key}: {:?}", text[line]);
        assert_eq!(venue.private_label_id, ticket);
        assert!(
            text[&ticket].ends_with("Private Party Ticket"),
            "{key}: {:?}",
            text[&ticket]
        );
        for cake in &venue.cakes {
            assert!(cakes.get(cake.id).is_some(), "{key}: cake item {}", cake.id);
        }
        let row = boxes.get(venue.boombox.id).expect("a boombox item");
        assert!(
            row.tints.iter().any(|t| t.song_effect_id > 0),
            "{key}: the box plays music"
        );
        for prop in &venue.decorations {
            let files = models.get(&i64::from(prop.id)).expect("a Models.txt row");
            assert!(
                files.iter().any(|f| names.contains(f)),
                "{key}: model {} in the packs",
                prop.id
            );
        }
    }
    // The dialog's stock labels.
    for (id, label) in [
        (3_482, "Friends"),
        (385_979, "Join the Party"),
        (427_559, "Invite Friends"),
        (118_044, "End"),
    ] {
        assert_eq!(text[&id], label);
    }
}
