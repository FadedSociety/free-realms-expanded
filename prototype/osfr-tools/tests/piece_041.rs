//! Builder tests for work item 041, "Racer and derby progression".
//!
//! The module is not registered in `src/lib.rs` (a shared file owned by the lead), so the test
//! target compiles the exact source with `#[path]`, as the other piece-NNN targets do. Lead
//! integration step: add `pub mod restoration;` (or the module directly) and drop the include.
//!
//! The tests assert the recovered definitions (job identity, rank gates, vehicle rows, part
//! classes) and the reconstructed once-only credit contract. They never claim native behaviour.

#[path = "../src/restoration/piece_041.rs"]
mod piece_041;

use piece_041::{
    CAPTURED_VEHICLES, CreditOutcome, DEMO_DERBY_DRIVER_GEAR, DEMO_DERBY_DRIVER_GEAR_CLASS,
    DEMO_DERBY_DRIVER_GEAR_PROFILE_NAME_ID, DEMO_DERBY_DRIVER_JOB, DEMO_DERBY_DRIVER_NAME_ID,
    DEMO_DERBY_KART_PART_CLASS, DEMO_DERBY_KART_PART_TYPE, DRIVER_JOBS, DriverActivity,
    DriverProgression, KART_DRIVER_GEAR, KART_DRIVER_GEAR_CLASS, KART_DRIVER_JOB,
    KART_DRIVER_NAME_ID, MAX_JOB_RANK, MAX_STARS_PER_OUTCOME, NON_MEMBER_RANK_CAP,
    ProgressionError, RACING_KART_PART_CLASS, RACING_KART_PART_TYPE, VEHICLE_CATEGORY_ID,
    VEHICLE_ITEM_TYPE, ValidatedOutcome, driver_job, rank_cap,
};

#[test]
fn recovered_job_identity() {
    let kart = driver_job(KART_DRIVER_JOB).expect("kart driver row");
    let derby = driver_job(DEMO_DERBY_DRIVER_JOB).expect("demo derby row");

    assert_eq!(kart.name_id, KART_DRIVER_NAME_ID);
    assert_eq!(kart.name_id, 2917);
    assert_eq!(kart.description_id, 2918);
    assert_eq!(derby.name_id, DEMO_DERBY_DRIVER_NAME_ID);
    assert_eq!(derby.name_id, 2919);
    assert_eq!(derby.description_id, 2920);

    assert_eq!(DRIVER_JOBS.len(), 2);
    assert_eq!(piece_041::validate(), Ok(()));
    assert_eq!(driver_job(1), None);
}

#[test]
fn activity_binds_job_and_original_wire() {
    assert_eq!(DriverActivity::KartRace.owning_job(), KART_DRIVER_JOB);
    assert_eq!(
        DriverActivity::DemoDerby.owning_job(),
        DEMO_DERBY_DRIVER_JOB
    );
    assert_eq!(DriverActivity::KartRace.lobby_game_type(), 1);
    assert_eq!(DriverActivity::DemoDerby.lobby_game_type(), 2);
    assert_eq!(DriverActivity::KartRace.tunnel_inner_opcode(), 33);
    assert_eq!(DriverActivity::DemoDerby.tunnel_inner_opcode(), 34);
}

#[test]
fn progression_rejects_non_driving_jobs() {
    assert!(matches!(
        DriverProgression::new(1),
        Err(ProgressionError::NotADrivingJob(1))
    ));
    assert!(DriverProgression::new(KART_DRIVER_JOB).is_ok());
    assert!(DriverProgression::new(DEMO_DERBY_DRIVER_JOB).is_ok());
    assert_eq!(
        DriverProgression::new(DEMO_DERBY_DRIVER_JOB).unwrap().job(),
        DEMO_DERBY_DRIVER_JOB
    );
}

#[test]
fn wrong_activity_is_rejected_without_mutation() {
    let mut kart = DriverProgression::new(KART_DRIVER_JOB).unwrap();
    let outcome = ValidatedOutcome::new(DriverActivity::DemoDerby, 7, 3);

    assert_eq!(
        kart.credit(outcome, 1),
        CreditOutcome::WrongJob {
            expected: KART_DRIVER_JOB,
            got: DEMO_DERBY_DRIVER_JOB,
        }
    );
    assert_eq!(kart.total_stars(), 0);
    assert_eq!(kart.credited_outcomes(), 0);
}

#[test]
fn validated_outcomes_credit_exactly_once() {
    let mut kart = DriverProgression::new(KART_DRIVER_JOB).unwrap();
    let outcome = ValidatedOutcome::new(DriverActivity::KartRace, 4242, 5);

    assert_eq!(
        kart.credit(outcome, 1),
        CreditOutcome::Credited {
            stars: 5,
            total_stars: 5,
        }
    );
    assert_eq!(kart.credit(outcome, 1), CreditOutcome::Duplicate);
    assert_eq!(kart.total_stars(), 5);
    assert_eq!(kart.credited_outcomes(), 1);
}

#[test]
fn distinct_outcomes_accumulate_and_unpriced_finish_credits_zero() {
    let mut derby = DriverProgression::new(DEMO_DERBY_DRIVER_JOB).unwrap();

    let unpriced = ValidatedOutcome::new(DriverActivity::DemoDerby, 1, 0);
    assert_eq!(
        derby.credit(unpriced, 1),
        CreditOutcome::Credited {
            stars: 0,
            total_stars: 0,
        }
    );

    let paid = ValidatedOutcome::new(DriverActivity::DemoDerby, 2, 4);
    assert_eq!(
        derby.credit(paid, 1),
        CreditOutcome::Credited {
            stars: 4,
            total_stars: 4,
        }
    );
    assert_eq!(derby.credited_outcomes(), 2);
}

#[test]
fn star_award_is_a_single_byte() {
    assert_eq!(MAX_STARS_PER_OUTCOME, 255);
    let mut kart = DriverProgression::new(KART_DRIVER_JOB).unwrap();
    let outcome = ValidatedOutcome::new(DriverActivity::KartRace, 9, MAX_STARS_PER_OUTCOME);
    assert_eq!(
        kart.credit(outcome, 1),
        CreditOutcome::Credited {
            stars: MAX_STARS_PER_OUTCOME,
            total_stars: u32::from(MAX_STARS_PER_OUTCOME),
        }
    );
}

#[test]
fn max_rank_credits_but_does_not_advance() {
    let mut kart = DriverProgression::new(KART_DRIVER_JOB).unwrap();
    let outcome = ValidatedOutcome::new(DriverActivity::KartRace, 11, 2);

    assert_eq!(
        kart.credit(outcome, MAX_JOB_RANK),
        CreditOutcome::AtMaxRank {
            rank: MAX_JOB_RANK,
            total_stars: 2,
        }
    );
    assert_eq!(kart.credit(outcome, MAX_JOB_RANK), CreditOutcome::Duplicate);
    assert_eq!(kart.credited_outcomes(), 1);
    assert_eq!(rank_cap(true), MAX_JOB_RANK);
    assert_eq!(rank_cap(false), NON_MEMBER_RANK_CAP);
}

#[test]
fn unlock_tables_cover_the_recovered_ranks() {
    let kart = DriverProgression::new(KART_DRIVER_JOB).unwrap();
    let derby = DriverProgression::new(DEMO_DERBY_DRIVER_JOB).unwrap();

    assert_eq!(kart.unlock_table().len(), 20);
    assert_eq!(
        kart.unlock_table()
            .iter()
            .map(|row| row.rank)
            .collect::<Vec<_>>(),
        (1..=20).collect::<Vec<u8>>()
    );

    assert_eq!(derby.unlock_table().len(), 19);
    assert!(
        derby.unlock_table().iter().all(|row| row.rank != 9),
        "the derby gear catalogue has no rank 9 row"
    );

    assert_eq!(kart.unlocked(1).len(), 1);
    assert_eq!(kart.unlocked(2).len(), 2);
    assert_eq!(kart.unlocked(MAX_JOB_RANK).len(), 20);
    assert_eq!(kart.unlocks_at(6).len(), 1);
    assert_eq!(kart.unlocks_at(6)[0].item_id, 76280);
    assert_eq!(derby.unlocks_at(9).len(), 0);
    assert_eq!(derby.unlocked(8).len(), 8);
}

#[test]
fn gear_class_and_part_class_constants_are_recovered() {
    assert_eq!(KART_DRIVER_GEAR_CLASS, 24);
    assert_eq!(DEMO_DERBY_DRIVER_GEAR_CLASS, 22);
    assert_eq!(DEMO_DERBY_DRIVER_GEAR_PROFILE_NAME_ID, 17401);
    assert_eq!(RACING_KART_PART_CLASS, 75);
    assert_eq!(DEMO_DERBY_KART_PART_CLASS, 76);
    assert_eq!(RACING_KART_PART_TYPE, 9);
    assert_eq!(DEMO_DERBY_KART_PART_TYPE, 10);
    assert_eq!(VEHICLE_ITEM_TYPE, 11);
    assert_eq!(VEHICLE_CATEGORY_ID, 127);

    assert_eq!(KART_DRIVER_GEAR[0].catalogue_rows, 450);
    assert_eq!(KART_DRIVER_GEAR[19].catalogue_rows, 609);
    assert_eq!(DEMO_DERBY_DRIVER_GEAR[0].catalogue_rows, 586);
    assert_eq!(DEMO_DERBY_DRIVER_GEAR[4].label, "Wonky Ram Helmet");
}

#[test]
fn captured_vehicles_match_the_recovered_catalogue_rows() {
    assert_eq!(CAPTURED_VEHICLES.len(), 6);
    let roadster = CAPTURED_VEHICLES[0];
    assert_eq!(roadster.item_id, 48316);
    assert_eq!(roadster.name_id, 141872);
    assert_eq!(roadster.description_id, 383250);
    assert_eq!(roadster.model_index, 6);
    assert_eq!(roadster.power_rating, 1);
    assert_eq!(roadster.label, "Roadster Racer");

    let zipper = CAPTURED_VEHICLES[4];
    assert_eq!(zipper.item_id, 61006);
    assert_eq!(zipper.model_index, 51);
    assert_eq!(zipper.power_rating, 5);
    assert_eq!(zipper.label, "Splash Zipper");

    assert!(
        CAPTURED_VEHICLES
            .iter()
            .all(|row| row.item_id > 0 && row.name_id > 0 && row.description_id > 0),
        "every captured loadout carries an item, name and description id"
    );
}
