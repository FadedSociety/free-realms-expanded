//! L039: a save whose open-world pose is off the map or deep under the terrain loads at the
//! nearest warpstone through the production load (`Session::new` -> `load_gateway` ->
//! `load_gateway_from`), the save on disk holds that pose at once, and the server's movement
//! check starts there. Without terrain the save loads and stays byte for byte as it was.
use super::test_support::{self, Scratch};
use super::*;
use osfr_tools::game_store::Snapshot;
use osfr_tools::terrain::{HeightGrid, Terrain};

/// Cobblestone Village's warpstone row (piece 077 hub 7), the one nearest to both poses below.
const COBBLESTONE: [f32; 3] = [-1904.46, -39.79, 413.316];

/// Flat ground at -40 m around Cobblestone Village, written as a real `zone_0.hgt`.
fn terrain(scratch: &Scratch) -> std::sync::Arc<Terrain> {
    let mut grid = HeightGrid::new([-2000.0, 300.0], [-1800.0, 500.0], 2.0).unwrap();
    let p = |x: f32, z: f32| [x, -40.0, z];
    grid.add_triangle([p(-2000.0, 300.0), p(-1800.0, 300.0), p(-1800.0, 500.0)]);
    grid.add_triangle([p(-2000.0, 300.0), p(-1800.0, 500.0), p(-2000.0, 500.0)]);
    let directory = scratch.path().join("terrain");
    fs::create_dir_all(&directory).unwrap();
    fs::write(directory.join("zone_0.hgt"), grid.to_bytes()).unwrap();
    std::sync::Arc::new(Terrain::load(&directory).unwrap())
}

/// Options on the lab fixture with one save at `position`; `None` when the fixture is absent.
fn saved_at(scratch: &Scratch, position: [f32; 3]) -> Option<Options> {
    let options = test_support::options(scratch.state());
    if !test_support::fixtures_present(&options) {
        eprintln!("skipped: lab gateway fixtures are not present");
        return None;
    }
    let mut character = Gateway::reference_state(&options.reference).unwrap();
    character.zone_id = 0;
    character.position = position;
    character.rotation = [0.0, 0.0, 1.0];
    // The opening content resets a save without opening progress; this one has it.
    let content = options.content.as_ref().unwrap();
    let gameplay = osfr_tools::gameplay::Gameplay::load(content, None)
        .unwrap()
        .state();
    assert!(gameplay.opening.is_some());
    test_support::save_character(&options.state, &character, &gameplay);
    Some(options)
}

fn saved_position(options: &Options) -> [f32; 3] {
    let initial = Gateway::reference_state(&options.reference).unwrap();
    let Snapshot { character, .. } = load_or_create(&options.state, initial).unwrap();
    character.position
}

/// One 125 position update from `position`, facing +z.
fn step(guid: u64, position: [f32; 3]) -> Vec<u8> {
    let mut inner = 125_u16.to_le_bytes().to_vec();
    inner.extend(guid.to_le_bytes());
    for value in position.into_iter().chain([0.0, 0.0, 1.0]) {
        inner.extend(value.to_le_bytes());
    }
    inner.extend([0, 0]);
    super::wrap(&inner)
}

#[test]
fn a_save_off_the_map_or_deep_under_it_enters_at_the_nearest_warpstone() {
    if !super::test_support::bin_test_target("l039") {
        return;
    }
    for (name, saved) in [
        ("off the map", [-2600.0, 0.0, 400.0]),
        ("300 m under", [-1900.0, -340.0, 400.0]),
    ] {
        let scratch = Scratch::new("l039-recover");
        let Some(mut options) = saved_at(&scratch, saved) else {
            return;
        };
        options.terrain = Some(terrain(&scratch));
        let mut session = test_support::enter(&options, 1);
        let arrival = session.gateway.state().position;
        assert_eq!(arrival[0], COBBLESTONE[0], "{name}");
        assert_eq!(arrival[2], COBBLESTONE[2], "{name}");
        assert!(
            (arrival[1] - -39.5).abs() < 0.01,
            "{name}: lifted onto the ground"
        );
        assert_eq!(saved_position(&options), arrival, "{name}: saved at once");
        assert_eq!(session.gateway.state().rotation, [0.0, 0.0, 1.0], "{name}");

        // Movement validation starts at the warpstone: a short step from it is accepted.
        let guid = session.gateway.state().guid;
        session
            .gateway
            .handle_at(&super::wrap(&[13, 0]), 100)
            .unwrap();
        session
            .gateway
            .handle_at(&super::wrap(&[10, 0]), 200)
            .unwrap();
        let next = [arrival[0] + 1.0, arrival[1], arrival[2]];
        let moved = session.gateway.handle_at(&step(guid, next), 1_000).unwrap();
        assert!(moved.state_changed, "{name}");
        assert_eq!(session.gateway.state().position, next, "{name}");

        // Idempotent: the next load finds a good pose and rewrites nothing.
        let bytes = fs::read(&options.state).unwrap();
        let again = test_support::enter(&options, 2);
        assert_eq!(again.gateway.state().position, arrival, "{name}");
        assert_eq!(fs::read(&options.state).unwrap(), bytes, "{name}");
    }
}

#[test]
fn without_terrain_or_on_the_ground_a_save_loads_unchanged() {
    if !super::test_support::bin_test_target("l039") {
        return;
    }
    for (name, saved, with_terrain) in [
        (
            "off the map, no terrain loaded",
            [-2600.0, 0.0, 400.0],
            false,
        ),
        (
            "300 m under, no terrain loaded",
            [-1900.0, -340.0, 400.0],
            false,
        ),
        ("on the ground", [-1900.0, -40.0, 400.0], true),
        ("a cave 60 m under", [-1900.0, -100.0, 400.0], true),
    ] {
        let scratch = Scratch::new("l039-keep");
        let Some(mut options) = saved_at(&scratch, saved) else {
            return;
        };
        if with_terrain {
            options.terrain = Some(terrain(&scratch));
        }
        let bytes = fs::read(&options.state).unwrap();
        let mut session = test_support::enter(&options, 1);
        assert_eq!(session.gateway.state().position, saved, "{name}");
        assert_eq!(fs::read(&options.state).unwrap(), bytes, "{name}");
        // Once logged in the pose is the client's: recovery never moves it without a 38/12.
        session.gateway.set_terrain(Some(terrain(&scratch)));
        assert!(session.gateway.recover_login_pose().is_none(), "{name}");
        assert_eq!(session.gateway.state().position, saved, "{name}");
    }
}
