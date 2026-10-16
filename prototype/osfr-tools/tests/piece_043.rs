//! Piece 043 (pet trainer and card progression) tests: recovered profile rows,
//! milestone unlock table and the job level-up notification contract.
//!
//! Evidence boundary, read before the assertions:
//!
//! ORIGINAL (first-party client inputs; hashes pinned in the lab bundle):
//! * `decompiled/client-ui/ScriptsBase.lua`, `NotificationHandler:JobLevelUp`:
//!   `goodIndices = {10,11,12,13,19,20,32,1783,1785,1999,46,4842,4843}`,
//!   `t[1783] = "levelup_tcg"`, `t[1785] = "levelup_pettrainer"`,
//!   `s[1783] = {[1]=16114,[4]=16115,[8]=16116,[12]=16117,[16]=16118}` and
//!   `s[1785] = {[1]=16119,[4]=16120,[8]=16121,[12]=16122,[16]=16123}`.
//!   Level 1 uses locale 919922, every other level uses 919923.
//! * Locale block 34070-34074 "Card Duelist: Level 4/8/12/16/20" with
//!   34150-34154 "Reach Level 4/8/12/16/20 as a Card Duelist." and the
//!   matching Pet Trainer block 34110-34114 / 34190-34194.
//! * `Customize/PacketSendSelfToClient.json` (the recorded profile sample):
//!   Card Duelist is job 120, name 2410, description 2411, level 20; the 16
//!   recorded rows match the pinned community `Profiles.json` catalogue 16/16.
//! * `FUN_0089b240` installs `PacketSendSelfToClient::vftable` and writes
//!   opcode 12, and the native `JobLevelUp` dispatch passes five values.
//! * Asset manifest: `levelup_pettrainer.gfx`, `levelup_tcg.gfx` and the
//!   per-job audio names `levelup_{pettrainer,cardduelist}_level{1,5,10,15,20,_misc}.mp3`.
//!
//! RECONSTRUCTED (labelled, not historical): the once-per-instance outcome
//! ledger, the caller-supplied XP curve, and the non-member level-4 cap that is
//! community-documented (fandom wiki "Jobs") rather than first-party.
//!
//! NOT CLAIMED: XP thresholds per level, a returned Pet Trainer profile row, the
//! Pet Trainer job id, or any native level-up run. Those stay gaps.
// The module is not registered in `src/lib.rs` (lead-owned). Including the exact
// source file keeps this target compiling against the real artifact. Integration
// step for the lead: register the module and drop this `#[path]` include.
#[path = "../src/restoration/piece_043.rs"]
#[allow(dead_code)]
mod piece_043;

use piece_043::{
    ACHIEVEMENT_MILESTONES, CARD_DUELIST_JOB_DESCRIPTION_LOCALE, CARD_DUELIST_JOB_ID,
    CARD_DUELIST_JOB_NAME_LOCALE, LEVEL_UP_MESSAGE_LEVEL_ONE, LEVEL_UP_MESSAGE_OTHER_LEVELS,
    LevelCurve, MAX_JOB_LEVEL, OutcomeResult, PET_TRAINER_BADGE_IMAGE_SET,
    PET_TRAINER_JOB_NAME_LOCALE, PET_TRAINER_PROFILE_JOB_ID, ProgressionError, ProgressionJob,
    ProgressionLedger, ProgressionSnapshot, SUPPORTED_LEVEL_UP_BADGE_SETS, ValidatedOutcome,
};

fn fixture_curve() -> LevelCurve {
    // Synthetic fixture curve: 100 XP per level, levels 1..=20. This is a test
    // fixture, not recovered data; the real thresholds stay a documented gap.
    LevelCurve::new(vec![100; MAX_JOB_LEVEL as usize - 1])
}

#[test]
fn recovered_job_ids_and_locale_rows() {
    assert_eq!(CARD_DUELIST_JOB_ID, 120);
    assert_eq!(CARD_DUELIST_JOB_NAME_LOCALE, 2410);
    assert_eq!(CARD_DUELIST_JOB_DESCRIPTION_LOCALE, 2411);
    assert_eq!(PET_TRAINER_JOB_NAME_LOCALE, 17419);
    assert_eq!(PET_TRAINER_BADGE_IMAGE_SET, 1785);
    // The recorded profile sample has no Pet Trainer row and no recovered job id.
    assert_eq!(PET_TRAINER_PROFILE_JOB_ID, None);
}

#[test]
fn milestone_rows_match_the_contiguous_locale_blocks() {
    let card = ProgressionJob::CardDuelist.milestones();
    let pet = ProgressionJob::PetTrainer.milestones();
    assert_eq!(ACHIEVEMENT_MILESTONES, [4, 8, 12, 16, 20]);
    assert_eq!(
        card.iter().map(|m| m.title_locale).collect::<Vec<_>>(),
        vec![34070, 34071, 34072, 34073, 34074]
    );
    assert_eq!(
        card.iter()
            .map(|m| m.description_locale)
            .collect::<Vec<_>>(),
        vec![34150, 34151, 34152, 34153, 34154]
    );
    assert_eq!(
        pet.iter().map(|m| m.title_locale).collect::<Vec<_>>(),
        vec![34110, 34111, 34112, 34113, 34114]
    );
    assert_eq!(
        pet.iter().map(|m| m.description_locale).collect::<Vec<_>>(),
        vec![34190, 34191, 34192, 34193, 34194]
    );
    assert!(
        card.iter()
            .all(|m| ACHIEVEMENT_MILESTONES.contains(&m.level))
    );
}

#[test]
fn level_up_presentation_rows_come_from_the_lua_table() {
    let tcg = ProgressionJob::CardDuelist.level_up();
    assert_eq!(tcg.badge_image_set, 1783);
    assert_eq!(tcg.movie, "levelup_tcg");
    assert_eq!(tcg.sounds, [16114, 16115, 16116, 16117, 16118]);

    let pet = ProgressionJob::PetTrainer.level_up();
    assert_eq!(pet.badge_image_set, 1785);
    assert_eq!(pet.movie, "levelup_pettrainer");
    assert_eq!(pet.sounds, [16119, 16120, 16121, 16122, 16123]);
}

#[test]
fn level_up_notification_requires_a_supported_badge_set() {
    assert!(SUPPORTED_LEVEL_UP_BADGE_SETS.contains(&1783));
    assert!(SUPPORTED_LEVEL_UP_BADGE_SETS.contains(&1785));
    // The Lua handler sets `isGood` only for the listed badge image sets, so an
    // unknown set shows nothing at all.
    assert!(piece_043::level_up_notification(9999, 4).is_none());
}

#[test]
fn level_up_notification_uses_the_recovered_message_locale() {
    let first = piece_043::level_up_notification(1785, 1).expect("first level is supported");
    assert_eq!(first.message_locale, LEVEL_UP_MESSAGE_LEVEL_ONE);
    assert_eq!(first.message_locale, 919922);
    assert_eq!(first.sound_id, Some(16119));

    let later = piece_043::level_up_notification(1785, 4).expect("band level is supported");
    assert_eq!(later.message_locale, LEVEL_UP_MESSAGE_OTHER_LEVELS);
    assert_eq!(later.message_locale, 919923);
}

#[test]
fn level_up_notification_movie_is_not_gated_by_the_sound_bands() {
    // The Lua handler always invokes the movie for a supported badge set but
    // indexes the sound table by the exact level, so a level outside
    // {1,4,8,12,16} shows the movie without a recovered sound id.
    let odd = piece_043::level_up_notification(1783, 7).expect("movie still plays");
    assert_eq!(odd.sound_id, None);
    assert_eq!(odd.movie, "levelup_tcg");
}

#[test]
fn ledger_grants_only_validated_completed_outcomes() {
    let mut ledger = ProgressionLedger::new(ProgressionJob::PetTrainer, true);
    let lost = ValidatedOutcome::pet_trick(11, OutcomeResult::Lost);
    let abandoned = ValidatedOutcome::pet_trick(12, OutcomeResult::Abandoned);
    assert_eq!(ledger.apply(&lost, &fixture_curve()).unwrap().xp, 0);
    assert_eq!(ledger.apply(&abandoned, &fixture_curve()).unwrap().xp, 0);
    assert_eq!(ledger.level(), 1);
    assert_eq!(ledger.xp_in_level(), 0);
}

#[test]
fn ledger_rejects_a_replayed_instance() {
    let mut ledger = ProgressionLedger::new(ProgressionJob::CardDuelist, true);
    let win = ValidatedOutcome::tcg_match(77, OutcomeResult::Completed { xp: 60 });
    assert_eq!(ledger.apply(&win, &fixture_curve()).unwrap().xp, 60);
    assert_eq!(
        ledger.apply(&win, &fixture_curve()).unwrap_err(),
        ProgressionError::DuplicateOutcome
    );
    assert_eq!(ledger.xp_in_level(), 60);
    assert_eq!(ledger.xp_total(), 60);
}

#[test]
fn ledger_is_job_scoped_and_rejects_cross_job_outcomes() {
    let curve = fixture_curve();
    let mut pet = ProgressionLedger::new(ProgressionJob::PetTrainer, true);
    let mut card = ProgressionLedger::new(ProgressionJob::CardDuelist, true);
    let trick = ValidatedOutcome::pet_trick(5, OutcomeResult::Completed { xp: 20 });
    let duel = ValidatedOutcome::tcg_match(5, OutcomeResult::Completed { xp: 20 });
    // The two instance spaces are separate, so instance 5 is valid for both jobs.
    assert_eq!(pet.apply(&trick, &curve).unwrap().xp, 20);
    assert_eq!(card.apply(&duel, &curve).unwrap().xp, 20);
    assert_eq!(
        pet.apply(&duel, &curve).unwrap_err(),
        ProgressionError::JobMismatch
    );
}

#[test]
fn ledger_refuses_a_zero_instance_id() {
    let mut ledger = ProgressionLedger::new(ProgressionJob::CardDuelist, true);
    let bogus = ValidatedOutcome::tcg_match(0, OutcomeResult::Completed { xp: 100 });
    assert_eq!(
        ledger.apply(&bogus, &fixture_curve()).unwrap_err(),
        ProgressionError::InvalidInstanceId
    );
    assert_eq!(ledger.level(), 1);
}

#[test]
fn ledger_unlocks_each_milestone_once_and_respects_the_cap() {
    let curve = fixture_curve();
    let mut ledger = ProgressionLedger::new(ProgressionJob::PetTrainer, true);
    let mut granted_levels = Vec::new();
    let mut total = 0u32;
    for instance in 1..=25u64 {
        let outcome = ValidatedOutcome::pet_trick(instance, OutcomeResult::Completed { xp: 100 });
        let applied = ledger.apply(&outcome, &curve).unwrap();
        total += applied.xp;
        granted_levels.extend(applied.unlocks.iter().map(|u| u.level));
    }
    assert_eq!(ledger.level(), MAX_JOB_LEVEL);
    assert_eq!(granted_levels, vec![4, 8, 12, 16, 20]);
    assert_eq!(total, 100 * 25);
    // XP past the cap is reported, but the level stays at the maximum.
    assert_eq!(ledger.level(), 20);
}

#[test]
fn non_member_cap_is_a_labelled_community_rule() {
    let curve = fixture_curve();
    let mut ledger = ProgressionLedger::new(ProgressionJob::CardDuelist, false);
    for instance in 1..=6u64 {
        let outcome = ValidatedOutcome::tcg_match(instance, OutcomeResult::Completed { xp: 100 });
        ledger.apply(&outcome, &curve).unwrap();
    }
    assert_eq!(ledger.member_level_cap(), 4);
    assert_eq!(ledger.level(), 4);
}

#[test]
fn snapshot_round_trip_validates_saved_progress() {
    let snapshot = ProgressionSnapshot {
        job: ProgressionJob::CardDuelist,
        level: 12,
        xp_in_level: 40,
        xp_total: 2400,
        unlocked_levels: vec![4, 8, 12],
    };
    let ledger = ProgressionLedger::from_snapshot(snapshot.clone(), true).unwrap();
    assert_eq!(ledger.snapshot(), snapshot);

    let too_high = ProgressionSnapshot {
        level: 21,
        ..snapshot.clone()
    };
    assert_eq!(
        ProgressionLedger::from_snapshot(too_high, true).unwrap_err(),
        ProgressionError::InvalidLevel(21)
    );
    let zero = ProgressionSnapshot {
        level: 0,
        ..snapshot.clone()
    };
    assert_eq!(
        ProgressionLedger::from_snapshot(zero, true).unwrap_err(),
        ProgressionError::InvalidLevel(0)
    );
    let non_member = ProgressionLedger::from_snapshot(snapshot.clone(), false);
    assert_eq!(
        non_member.unwrap_err(),
        ProgressionError::InvalidLevelForMembership(12)
    );
    let bad_unlock = ProgressionSnapshot {
        unlocked_levels: vec![5],
        ..snapshot
    };
    assert_eq!(
        ProgressionLedger::from_snapshot(bad_unlock, true).unwrap_err(),
        ProgressionError::UnknownMilestone(5)
    );
}
