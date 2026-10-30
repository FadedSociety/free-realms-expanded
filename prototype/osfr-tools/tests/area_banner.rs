//! T900: the area-enter banner. `AreaBanners` (the session state the gateway tick polls)
//! against the shipped `regions.json`: 104 then 15/2 once, after the settle time, per new
//! named area. The gateway side is `client_area_routing` in the `osfr-gateway` bin tests.

#[path = "support/json.rs"]
mod json;
#[path = "support/node_content.rs"]
mod node_content;
#[path = "support/scratch.rs"]
mod scratch;

use osfr_tools::{
    area_banner::{AreaBanners, poi_change},
    chat::enter_area,
    world_regions::RegionTable,
};

const SESSION: u64 = 7;
/// World positions `[x, y, z]` inside Sanctuary (region 5), Snowhill (8), the Lakeshore place in
/// default region 1, and outside every region (sample points of `tests/world_regions.rs`).
const SANCTUARY: [f32; 3] = [-1239.0, 0.0, 533.0];
const SNOWHILL: [f32; 3] = [156.0, 0.0, 390.0];
const LAKESHORE: [f32; 3] = [-751.0, 0.0, 765.0];
const NOWHERE: [f32; 3] = [9000.0, 0.0, 9000.0];

fn table() -> RegionTable {
    RegionTable::load(&node_content::content_dir())
        .expect("regions.json loads")
        .expect("regions.json is shipped")
}

fn at(
    banners: &mut AreaBanners,
    table: &RegionTable,
    position: [f32; 3],
    now_ms: u64,
) -> Vec<Vec<u8>> {
    banners.poll(SESSION, Some((table, 0, position)), now_ms)
}

/// The two packets that announce an area: 104 (name, region as POI id, area 0), then 15/2.
fn announcement(name: u32, region: u32) -> Vec<Vec<u8>> {
    vec![poi_change(name, region, 0), enter_area(name)]
}

#[test]
fn the_wire_matches_the_client_readers() {
    // 104: i16 opcode, then u32 name id, u32 POI id, u32 area id (reader 0x008d5dd0).
    assert_eq!(
        poi_change(3961, 8, 0),
        [104, 0, 0x79, 0x0F, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0]
    );
    // 15/2: u16 family, u16 subtype, one u32 (readers 0x00b0f550, 0x00b10580).
    assert_eq!(enter_area(3961), [15, 0, 2, 0, 0x79, 0x0F, 0, 0]);
}

#[test]
fn a_new_area_is_announced_once_after_the_settle_time() {
    let (table, mut banners) = (table(), AreaBanners::default());
    assert_eq!(table.banner_settle_ms(), 1500);
    assert!(at(&mut banners, &table, SANCTUARY, 10_000).is_empty());
    assert!(at(&mut banners, &table, SANCTUARY, 11_499).is_empty());
    assert_eq!(
        at(&mut banners, &table, SANCTUARY, 11_500),
        announcement(3499, 5)
    );
    for now in [11_550, 20_000, 99_000] {
        assert!(at(&mut banners, &table, SANCTUARY, now).is_empty(), "{now}");
    }
    // The next region settles on its own clock and announces its own name and POI id.
    assert!(at(&mut banners, &table, SNOWHILL, 100_000).is_empty());
    assert_eq!(
        at(&mut banners, &table, SNOWHILL, 101_500),
        announcement(3961, 8)
    );
}

#[test]
fn leaving_before_the_settle_time_announces_nothing() {
    let (table, mut banners) = (table(), AreaBanners::default());
    at(&mut banners, &table, SANCTUARY, 0);
    assert_eq!(
        at(&mut banners, &table, SANCTUARY, 1_500),
        announcement(3499, 5)
    );
    // Into Snowhill and back to the announced Sanctuary inside the settle time.
    at(&mut banners, &table, SNOWHILL, 5_000);
    at(&mut banners, &table, SANCTUARY, 6_000);
    for now in [6_500, 8_000, 60_000] {
        assert!(at(&mut banners, &table, SANCTUARY, now).is_empty(), "{now}");
    }
    // A change of area restarts the wait: Snowhill, then the Lakeshore place.
    at(&mut banners, &table, SNOWHILL, 70_000);
    assert!(at(&mut banners, &table, LAKESHORE, 71_000).is_empty());
    assert!(at(&mut banners, &table, LAKESHORE, 72_499).is_empty());
    assert_eq!(
        at(&mut banners, &table, LAKESHORE, 72_500),
        announcement(18_899, 1)
    );
}

#[test]
fn the_default_region_names_its_place_and_resets_the_client_without_a_banner() {
    let (table, mut banners) = (table(), AreaBanners::default());
    at(&mut banners, &table, SNOWHILL, 0);
    at(&mut banners, &table, SNOWHILL, 1_500);
    // POI id 1 reaches no `LoadingZone` call (Lua `1 < poiId < 11`) but changes the client's
    // stored pair, so walking back into Snowhill shows its banner again.
    at(&mut banners, &table, LAKESHORE, 10_000);
    let lakeshore = at(&mut banners, &table, LAKESHORE, 11_500);
    assert_eq!(lakeshore, announcement(18_899, 1));
    at(&mut banners, &table, SNOWHILL, 20_000);
    assert_eq!(
        at(&mut banners, &table, SNOWHILL, 21_500),
        announcement(3961, 8)
    );
}

#[test]
fn outside_every_region_keeps_the_announced_area() {
    let (table, mut banners) = (table(), AreaBanners::default());
    assert!(at(&mut banners, &table, NOWHERE, 0).is_empty());
    assert!(at(&mut banners, &table, NOWHERE, 5_000).is_empty());
    at(&mut banners, &table, SNOWHILL, 6_000);
    assert_eq!(
        at(&mut banners, &table, SNOWHILL, 7_500),
        announcement(3961, 8)
    );
    at(&mut banners, &table, NOWHERE, 9_000);
    assert!(at(&mut banners, &table, NOWHERE, 20_000).is_empty());
    // Back where the client already shows it: nothing to say.
    at(&mut banners, &table, SNOWHILL, 21_000);
    assert!(at(&mut banners, &table, SNOWHILL, 30_000).is_empty());
}

#[test]
fn a_session_outside_the_open_world_is_announced_again_on_return() {
    let (table, mut banners) = (table(), AreaBanners::default());
    at(&mut banners, &table, SNOWHILL, 0);
    assert_eq!(
        at(&mut banners, &table, SNOWHILL, 1_500),
        announcement(3961, 8)
    );
    // An instance (or a reconnect) holds no state; the zone's own lookup yields no region.
    assert!(banners.poll(SESSION, None, 2_000).is_empty());
    assert!(
        banners
            .poll(SESSION + 1, Some((&table, 0, SANCTUARY)), 2_000)
            .is_empty()
    );
    assert!(
        banners
            .poll(SESSION, Some((&table, 5, SNOWHILL)), 2_100)
            .is_empty()
    );
    assert!(
        banners
            .poll(SESSION, Some((&table, 5, SNOWHILL)), 9_000)
            .is_empty()
    );
    banners.poll(SESSION, None, 9_500);
    at(&mut banners, &table, SNOWHILL, 10_000);
    assert_eq!(
        at(&mut banners, &table, SNOWHILL, 11_500),
        announcement(3961, 8)
    );
    // Each session has its own clock: the other one settled in Sanctuary on its own.
    assert_eq!(
        banners.poll(SESSION + 1, Some((&table, 0, SANCTUARY)), 3_500),
        announcement(3499, 5)
    );
}

#[test]
fn the_settle_time_is_content_with_a_sanity_bound() {
    let shipped = json::read(&node_content::content_dir().join("regions.json"));
    let load = |settle: Option<u64>| {
        let dir = scratch::Scratch::new("t900-settle");
        let mut value = shipped.clone();
        match settle {
            Some(ms) => value["banner_settle_ms"] = ms.into(),
            None => {
                value.as_object_mut().unwrap().remove("banner_settle_ms");
            }
        }
        json::write_pretty(&dir.path().join("regions.json"), &value);
        RegionTable::load(dir.path())
    };
    let table = load(Some(0)).unwrap().unwrap();
    let mut banners = AreaBanners::default();
    assert_eq!(
        at(&mut banners, &table, SNOWHILL, 100),
        announcement(3961, 8),
        "0 ms announces on the first look"
    );
    assert_eq!(load(None).unwrap().unwrap().banner_settle_ms(), 1500);
    assert_eq!(
        load(Some(60_000)).unwrap().unwrap().banner_settle_ms(),
        60_000
    );
    let error = load(Some(60_001)).expect_err("above the bound is refused");
    assert!(error.to_string().contains("banner_settle_ms"), "{error}");
}
