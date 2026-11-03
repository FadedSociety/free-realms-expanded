//! L942: soccer power-ups (`soccer_pickups`) with the shipped `soccer.json` values: a pickup appears
//! after the first wait on a spot, a player on the spot grabs it (76/10, then a kind-0 76/9), an
//! ungrabbed one clears after its lifetime, and the kinds take turns.

use osfr_tools::soccer::SoccerPacket;
use osfr_tools::soccer_pickups::{CHARGE, Pickups, SPEED, Settings, TOUGHNESS};
use std::path::PathBuf;

const STEP: f32 = 0.25;

fn shipped() -> Settings {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/soccer.json");
    let json: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    serde_json::from_value(json["pickups"].clone()).unwrap()
}

#[test]
fn a_pickup_appears_after_the_first_wait_and_a_player_on_its_spot_grabs_it() {
    let settings = shipped();
    let [x, z] = settings.spots[0];
    let mut pickups = Pickups::new(settings.clone()).unwrap();
    let away = [(1, [0.0, 0.0, 0.0])];

    let (records, _) = pickups.step(settings.first_after - STEP, &away);
    assert!(records.is_empty(), "nothing before the first wait");
    let (records, _) = pickups.step(STEP, &away);
    assert!(
        matches!(records.as_slice(),
            [SoccerPacket::SpawnPickUp { state: SPEED, marker: 0, position, .. }]
                if *position == [x, z, settings.lifetime]),
        "{records:?}"
    );

    let (records, grabs) = pickups.step(STEP, &[(1, [0.0; 3]), (7, [x, 0.0, z])]);
    assert_eq!(grabs.len(), 1);
    assert_eq!((grabs[0].player, grabs[0].kind), (7, SPEED));
    assert!(
        matches!(records.as_slice(), [
            SoccerPacket::AcquirePickUp { flag: 0, player: 7, value },
            SoccerPacket::SpawnPickUp { state: 0, marker: 0, .. },
        ] if *value == settings.duration),
        "{records:?}"
    );
}

#[test]
fn an_ungrabbed_pickup_clears_after_its_lifetime_and_the_kinds_take_turns() {
    let settings = shipped();
    let mut pickups = Pickups::new(settings.clone()).unwrap();
    let away = [(1, [0.0, 0.0, 0.0])];
    let (mut spawned, mut cleared, mut clock) = (Vec::new(), 0, 0.0);
    while clock < settings.first_after + settings.lifetime + STEP {
        let (records, grabs) = pickups.step(STEP, &away);
        clock += STEP;
        assert!(grabs.is_empty());
        for record in records {
            match record {
                SoccerPacket::SpawnPickUp { state: 0, .. } => cleared += 1,
                SoccerPacket::SpawnPickUp { state, .. } => spawned.push(state),
                other => panic!("unexpected {other:?}"),
            }
        }
    }
    assert_eq!(spawned[..3], [SPEED, TOUGHNESS, CHARGE]);
    assert!(
        spawned.len() <= settings.spots.len() + 1,
        "one pickup per free spot"
    );
    assert!(cleared >= 1, "the first pickup expired");
}
