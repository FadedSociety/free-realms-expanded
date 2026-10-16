//! Work item 042 acceptance: Soccer Star and Postman job progression coverage.
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `lab/reports/job-catalogue-20261009T022023Z/locale-en_us.json` - the preserved original
//!   locale export (85,932 rows). Job identity (3237/3238/4650/2920), the two hints
//!   (384184/416471), the level milestone quests (`Postman: Level 4/8/12/16/20` 34115-34119 and
//!   `Soccer Star: Level 4/8/12/16/20` 34120-34124 with objectives 34195-34199 / 34200-34204),
//!   the four Postman training gates (387931/387935/387939/387943 with objectives and trainer
//!   lines), the Soccer Star tutorial pages plus pickup icons (406669-406678, 416748) and the
//!   Postman mail-sorting tutorial (19323/19302) all resolve there byte for byte.
//! * `lab/working/OSFR.Client/OSFR Client/Resources/CodeStringMappings.txt` - the client's own
//!   name-to-string mapping. It resolves `SoccerTut0001..0007`, `SoccerTutIcon0001..0004` and
//!   `MailSortingTutorialOneTop/Bottom` to the ids above.
//! * `lab/decompiled/client-ui/ScriptsBase.lua` - `DS_SOCCER_TUTORIAL = 25` maps to
//!   `BaseClient.SoccerProcessor.SoccerPlayerActions`; `SoccerTutorial.OnDataChanged` advances the
//!   7-page tutorial only on a set column/value pair; `pageCount = 7`.
//! * `lab/references/Sanctuary/src/Resources/Profiles.json`, `ItemClasses.txt` and
//!   `ClientItemDefinitions.json` - community-pinned profile rows 4/52 and the creation kit item
//!   names (`Student Postman *` class 3, `Fast Forward *` class 134).
//! * `lab/reports/minigame-content-inventory.json` - the soccer activities (#42/#43/#44, type 12)
//!   and the Postman mail activities including `Can You Deliver?` (#238).
//! * `lab/references/freerealms-wiki/wiki.sqlite` - the `Postman`, `Soccer Star` and `Jobs` pages
//!   (unlock route, trainers, quest list, the level-20 cap and the level-4 non-member cap).
//!
//! BOUNDARIES: the module is recovery metadata plus two pure helpers (a tutorial page advance and
//! a level gate query). It grants no job, no item, no currency and no experience. The numeric XP
//! curve, the per-job unlock quest ids and the meaning of the recovered action columns were not
//! recovered and are reported as gaps instead of invented values.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers `pub mod piece_042;` under `src/restoration/mod.rs`/`src/lib.rs`.
#[path = "../src/restoration/piece_042.rs"]
#[allow(dead_code)]
mod piece_042;

use piece_042::{
    MAX_JOB_LEVEL, NON_MEMBER_JOB_LEVEL_CAP, POSTMAN_CREATION_KIT, POSTMAN_LEVEL_GATES,
    POSTMAN_LEVEL_MILESTONES, POSTMAN_MAIL_SORTING_TUTORIAL, POSTMAN_PROFILE_ID, POSTMAN_SKILLS,
    ProgressionError, SOCCER_STAR_CREATION_KIT, SOCCER_STAR_JOB_QUESTLINE,
    SOCCER_STAR_LEVEL_MILESTONES, SOCCER_STAR_PROFILE_ID,
    SOCCER_STAR_QUESTLINE_QUEST_IDS_RECOVERED, SOCCER_TUTORIAL_PAGE_COUNT, SOCCER_TUTORIAL_PAGES,
    SOCCER_TUTORIAL_PICKUP_ICONS, SOCCER_TUTORIAL_SOURCE_COLUMN_MEANING_RECOVERED, XpCurve,
    job_progression, level_gate, milestone_reached, soccer_tutorial_advance, unlocked_level_gates,
    validate, validate_locale,
};
use std::collections::BTreeMap;

/// Original locale text for every string id this module claims.
fn recovered_locale() -> BTreeMap<u32, String> {
    [
        (3237u32, "Postman"),
        (3238, "Postman description"),
        (4650, "Soccer Star"),
        (2920, "Demolition Derby Driver Driver Description"),
        (384184, "Felipe needs to train more postmen to help out at Pixiewood."),
        (
            416471,
            "South of Snowhill, Manager Landon is helping teams recruit new soccer stars.",
        ),
        (34115, "Postman: Level 4"),
        (34116, "Postman: Level 8"),
        (34117, "Postman: Level 12"),
        (34118, "Postman: Level 16"),
        (34119, "Postman: Level 20"),
        (34195, "Reach Level 4 as a Postman."),
        (34196, "Reach Level 8 as a Postman."),
        (34197, "Reach Level 12 as a Postman."),
        (34198, "Reach Level 16 as a Postman."),
        (34199, "Reach Level 20 as a Postman."),
        (34120, "Soccer Star: Level 4"),
        (34121, "Soccer Star: Level 8"),
        (34122, "Soccer Star: Level 12"),
        (34123, "Soccer Star: Level 16"),
        (34124, "Soccer Star: Level 20"),
        (34200, "Reach Level 4 as a Soccer Star."),
        (34201, "Reach Level 8 as a Soccer Star."),
        (34202, "Reach Level 12 as a Soccer Star."),
        (34203, "Reach Level 16 as a Soccer Star."),
        (34204, "Reach Level 20 as a Soccer Star."),
        (387931, "Postman: Achieve Level 5"),
        (387932, "Your training will continue once you have reached level 5 as a postman. Return to Felipe at that time."),
        (387935, "Postman: Achieve Level 10"),
        (387936, "Your training will continue once you have reached level 10 as a postman. Return to Enve LeStamp at that time."),
        (387939, "Postman: Achieve Level 15"),
        (387940, "Your training will continue once you have reached level 15 as a postman. Return to Postchug Runalug at that time."),
        (387943, "Postman: Achieve Level 20"),
        (387944, "Your training will continue once you have reached level 20 as a postman. Return to Brin Galeglow at that time."),
        (5175, "Throw Bone"),
        (406669, "Soccer Basics"),
        (406670, "Kick"),
        (406671, "Super kick"),
        (406672, "Pass"),
        (406673, "Dodge"),
        (406674, "Slide Tackle and Steal"),
        (416748, "Power-up page"),
        (406675, "Charge Rate"),
        (406676, "Speed Boost"),
        (406677, "Tackle Power"),
        (406678, "Toughness"),
        (19323, "Mail Sort tutorial top"),
        (19302, "Mail Sort tutorial bottom"),
    ]
    .into_iter()
    .map(|(id, text)| (id, text.to_string()))
    .collect()
}

#[test]
fn job_identity_matches_the_recovered_locale_and_profile_rows() {
    let postman = job_progression(POSTMAN_PROFILE_ID).expect("Postman row");
    assert_eq!(postman.name, "Postman");
    assert_eq!(postman.name_id, 3237);
    assert_eq!(postman.description_id, 3238);
    assert_eq!(postman.profile_type, 3);
    assert_eq!(postman.hint_id, 384184);
    assert!(postman.trial);
    assert!(!postman.members_only);

    let soccer = job_progression(SOCCER_STAR_PROFILE_ID).expect("Soccer Star row");
    assert_eq!(soccer.name, "Soccer Star");
    assert_eq!(soccer.name_id, 4650);
    assert_eq!(soccer.profile_type, 8);
    assert_eq!(soccer.hint_id, 416471);
    assert!(soccer.trial);

    assert!(job_progression(1).is_none());
    assert!(job_progression(43).is_none());
}

#[test]
fn locale_validation_passes_on_the_recovered_locale_and_fails_on_drift() {
    let locale = recovered_locale();
    assert!(validate_locale(|id| locale.get(&id).cloned()).is_ok());

    let mut drifted = recovered_locale();
    drifted.insert(4650, "Football Star".to_string());
    let error = validate_locale(|id| drifted.get(&id).cloned()).unwrap_err();
    match error {
        ProgressionError::LocalizedTextDiffers {
            profile_id,
            string_id,
            ..
        } => {
            assert_eq!(profile_id, SOCCER_STAR_PROFILE_ID);
            assert_eq!(string_id, 4650);
        }
        other => panic!("unexpected error {other:?}"),
    }

    let mut missing = recovered_locale();
    missing.remove(&387932);
    let error = validate_locale(|id| missing.get(&id).cloned()).unwrap_err();
    assert!(matches!(
        error,
        ProgressionError::MissingLocalizedText { .. }
    ));
}

#[test]
fn level_milestones_keep_their_recovered_levels_and_ids() {
    let postman_levels: Vec<u8> = POSTMAN_LEVEL_MILESTONES
        .iter()
        .map(|row| row.level)
        .collect();
    assert_eq!(postman_levels, vec![4, 8, 12, 16, 20]);
    assert_eq!(POSTMAN_LEVEL_MILESTONES[0].name_id, 34115);
    assert_eq!(POSTMAN_LEVEL_MILESTONES[4].objective_id, 34199);

    let soccer_levels: Vec<u8> = SOCCER_STAR_LEVEL_MILESTONES
        .iter()
        .map(|row| row.level)
        .collect();
    assert_eq!(soccer_levels, vec![4, 8, 12, 16, 20]);
    assert_eq!(SOCCER_STAR_LEVEL_MILESTONES[0].name_id, 34120);
    assert_eq!(SOCCER_STAR_LEVEL_MILESTONES[4].objective_id, 34204);

    assert!(!milestone_reached(3, &POSTMAN_LEVEL_MILESTONES[0]));
    assert!(milestone_reached(4, &POSTMAN_LEVEL_MILESTONES[0]));
    assert!(!milestone_reached(19, &POSTMAN_LEVEL_MILESTONES[4]));
    assert!(milestone_reached(20, &POSTMAN_LEVEL_MILESTONES[4]));
}

#[test]
fn postman_training_gates_name_the_recovered_trainers() {
    let levels: Vec<u8> = POSTMAN_LEVEL_GATES.iter().map(|row| row.level).collect();
    assert_eq!(levels, vec![5, 10, 15, 20]);
    assert_eq!(POSTMAN_LEVEL_GATES[0].trainer, "Felipe");
    assert_eq!(POSTMAN_LEVEL_GATES[1].trainer, "Enve LeStamp");
    assert_eq!(POSTMAN_LEVEL_GATES[2].trainer, "Postchug Runalug");
    assert_eq!(POSTMAN_LEVEL_GATES[3].trainer, "Brin Galeglow");
    assert_eq!(POSTMAN_LEVEL_GATES[0].name_id, 387931);
    assert_eq!(POSTMAN_LEVEL_GATES[3].objective_id, 387944);

    assert!(level_gate(4).is_none());
    assert_eq!(level_gate(5).unwrap().trainer, "Felipe");
    assert_eq!(level_gate(20).unwrap().name_id, 387943);

    assert_eq!(unlocked_level_gates(1), Vec::<u8>::new());
    assert_eq!(unlocked_level_gates(5), vec![5]);
    assert_eq!(unlocked_level_gates(15), vec![5, 10, 15]);
    assert_eq!(unlocked_level_gates(20), vec![5, 10, 15, 20]);
    assert_eq!(unlocked_level_gates(99), vec![5, 10, 15, 20]);
}

#[test]
fn soccer_tutorial_pages_and_pickups_keep_the_client_mapping() {
    assert_eq!(SOCCER_TUTORIAL_PAGE_COUNT, 7);
    assert_eq!(SOCCER_TUTORIAL_PAGES.len(), 7);
    let ids: Vec<u32> = SOCCER_TUTORIAL_PAGES
        .iter()
        .map(|page| page.string_id)
        .collect();
    assert_eq!(
        ids,
        vec![406669, 406670, 406671, 406672, 406673, 406674, 416748]
    );
    let keys: Vec<&str> = SOCCER_TUTORIAL_PAGES.iter().map(|page| page.key).collect();
    assert_eq!(
        keys,
        vec![
            "SoccerTut0001",
            "SoccerTut0002",
            "SoccerTut0003",
            "SoccerTut0004",
            "SoccerTut0005",
            "SoccerTut0006",
            "SoccerTut0007",
        ]
    );

    assert_eq!(SOCCER_TUTORIAL_PICKUP_ICONS.len(), 3);
    assert_eq!(SOCCER_TUTORIAL_PICKUP_ICONS[0].icon_id, 11542);
    assert_eq!(SOCCER_TUTORIAL_PICKUP_ICONS[1].icon_id, 17353);
    assert_eq!(SOCCER_TUTORIAL_PICKUP_ICONS[2].icon_id, 18855);
    assert_eq!(SOCCER_TUTORIAL_PICKUP_ICONS[2].string_id, 406678);
}

#[test]
fn soccer_tutorial_advance_reproduces_the_client_transition_table() {
    let cases: [(u8, u8, Option<u8>); 12] = [
        (1, 0, Some(2)),
        (1, 2, None),
        (2, 2, Some(3)),
        (2, 0, None),
        (3, 3, Some(4)),
        (4, 6, Some(5)),
        (4, 7, Some(5)),
        (4, 3, None),
        (5, 1, Some(6)),
        (6, 4, Some(7)),
        (6, 5, Some(7)),
        (7, 0, None),
    ];
    for (page, column, expected) in cases {
        assert_eq!(
            soccer_tutorial_advance(page, column),
            expected,
            "page {page} column {column}"
        );
    }
    assert_eq!(soccer_tutorial_advance(0, 0), None);
    assert_eq!(soccer_tutorial_advance(1, 9), None);
}

#[test]
fn progression_does_not_invent_experience_or_unlock_quest_ids() {
    let postman = job_progression(POSTMAN_PROFILE_ID).unwrap();
    let soccer = job_progression(SOCCER_STAR_PROFILE_ID).unwrap();
    assert_eq!(postman.xp_curve, XpCurve::Missing);
    assert_eq!(soccer.xp_curve, XpCurve::Missing);
    assert!(!postman.unlock_quest_ids_recovered);
    assert!(!SOCCER_STAR_QUESTLINE_QUEST_IDS_RECOVERED);
    assert!(!SOCCER_TUTORIAL_SOURCE_COLUMN_MEANING_RECOVERED);
    assert_eq!(MAX_JOB_LEVEL, 20);
    assert_eq!(NON_MEMBER_JOB_LEVEL_CAP, 4);
}

#[test]
fn creation_kits_and_questline_stay_on_their_recovered_classes() {
    for item in POSTMAN_CREATION_KIT {
        assert_eq!(item.class_id, 3, "{}", item.name);
    }
    for item in SOCCER_STAR_CREATION_KIT {
        assert_eq!(item.class_id, 134, "{}", item.name);
    }
    assert_eq!(SOCCER_STAR_JOB_QUESTLINE[0].name_id, 407407);
    assert_eq!(SOCCER_STAR_JOB_QUESTLINE[11].name_id, 407418);
    assert_eq!(POSTMAN_SKILLS[1].name_id, Some(5175));
    assert_eq!(POSTMAN_MAIL_SORTING_TUTORIAL[0].string_id, 19323);
    assert!(validate().is_ok());
}
