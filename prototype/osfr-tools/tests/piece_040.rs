//! Work item 040 acceptance: Chef/Blacksmith recipe rank unlocks and job level rows.
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `lab/references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json`
//!   (pinned revision 9e082025391177d5a33ee18c714ddd06b457a695, AGPL-3.0, inspected as
//!   facts) - the 49 recoverable `Class 58`/`Class 59` `CategoryId 17` `Type 5` recipe
//!   records, their `Param1` recipe index, `NameId`, `Cost 540`, `NoSale`, `NoTrade`,
//!   single stack and `MinProfileRank` rank gate.
//! * The same tree's `ItemClasses.txt` - class 58 `Blacksmith Shards` is owned by
//!   profile name id 2614 (Blacksmith 16) and class 59 `Chef Shards` by 2856 (Chef 45).
//! * `lab/reports/job-catalogue-20261009T022023Z/locale-en_us.json` - the preserved
//!   85,932 row locale table; every string id below resolves there.
//! * `lab/working/OSFR.Client/OSFR Client/Resources/MiniGameData.txt` - rows 46 and 560
//!   `Cooking: Strong Arm Stew` difficulty 1 and row 186 `Cooking: Starnut Sundae`
//!   difficulty 3.
//! * `lab/references/freerealms-wiki/wiki.sqlite` - `List of Recipes (Chef)` revision
//!   45318 records `Where to find: Tutorial` for Strong Arm Stew and `Gain 10 levels in
//!   Chef` for Starnut Sundae; the Chef revision 46136 and Blacksmith revision 46062
//!   pages list the level gate quests.
//!
//! `MinProfileRank` is the recovered rank gate. The rule enforced by `recipe_access`
//! (same job owner, rank at or above the recovered gate, rank 0 below the floor of 1,
//! rank above 20 clamped to the recovered maximum) is reconstructed policy over that
//! recovered data. Nothing here grants items, coins or experience, and no native
//! acceptance is claimed.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before
//! the lead registers `pub mod piece_040;` under `src/restoration/mod.rs`/`src/lib.rs`.
#[path = "../src/restoration/piece_040.rs"]
#[allow(dead_code)]
mod piece_040;

use piece_040::{
    ACCEPTED_COOKING_PROFILE, ACCEPTED_FORGING_PROFILE, BLACKSMITH_PROFILE,
    BLACKSMITH_RECIPE_CLASS, CHEF_PROFILE, CHEF_RECIPE_CLASS, LEVEL_GATE_QUESTS, LEVEL_TITLES,
    MAX_JOB_RANK, RECIPE_ITEMS, RecipeAccess, WIKI_GATES, crafting_job, crafting_jobs,
    cross_class_recipes, level_gate_quest, level_title, rank_gated_recipes, recipe_access,
    recipe_item, recipes_for, shard_class, shard_class_for, wiki_gate, wiki_gate_access,
};

#[test]
fn crafting_jobs_are_the_recovered_chef_and_blacksmith_profiles() {
    assert_eq!(crafting_jobs().len(), 2);
    let chef = crafting_job(CHEF_PROFILE).unwrap();
    assert_eq!(
        (chef.profile_id, chef.name_id, chef.name),
        (45, 2856, "Chef")
    );
    let blacksmith = crafting_job(BLACKSMITH_PROFILE).unwrap();
    assert_eq!(
        (blacksmith.profile_id, blacksmith.name_id, blacksmith.name),
        (16, 2614, "Blacksmith")
    );
    assert!(crafting_job(43).is_none());

    assert_eq!((CHEF_RECIPE_CLASS, BLACKSMITH_RECIPE_CLASS), (59, 58));
    let chef_class = shard_class(CHEF_RECIPE_CLASS).unwrap();
    assert_eq!(
        (
            chef_class.name_id,
            chef_class.name,
            chef_class.owner_profile
        ),
        (2562, "Chef Shards", 45)
    );
    let blacksmith_class = shard_class(BLACKSMITH_RECIPE_CLASS).unwrap();
    assert_eq!(
        (
            blacksmith_class.name_id,
            blacksmith_class.name,
            blacksmith_class.owner_profile
        ),
        (2561, "Blacksmith Shards", 16)
    );
    assert_eq!(shard_class_for(CHEF_PROFILE).unwrap().class_id, 59);
    assert_eq!(shard_class_for(BLACKSMITH_PROFILE).unwrap().class_id, 58);
    assert!(shard_class_for(43).is_none());
    assert!(shard_class(17).is_none());
}

#[test]
fn recipe_catalogue_is_the_recovered_class_58_and_59_records() {
    assert_eq!(RECIPE_ITEMS.len(), 49);
    assert_eq!(
        recipes_for(BLACKSMITH_PROFILE).count(),
        27,
        "class 58 recipe rows"
    );
    assert_eq!(
        recipes_for(CHEF_PROFILE).count(),
        22,
        "class 59 recipe rows"
    );
    for item in RECIPE_ITEMS {
        assert!(item.name.ends_with(" Recipe"), "{}", item.name);
        assert!([BLACKSMITH_RECIPE_CLASS, CHEF_RECIPE_CLASS].contains(&item.class_id));
        assert!(item.recipe_index > 0);
        assert!(item.min_profile_rank <= MAX_JOB_RANK);
    }
    assert_eq!(recipe_item(869).unwrap().recipe_index, 59);
    assert_eq!(
        recipe_item(2506).unwrap().name,
        "Spunky Scrapper Hammer Recipe"
    );
    assert_eq!(recipe_item(2506).unwrap().min_profile_rank, 1);
    assert_eq!(recipe_item(2523).unwrap().min_profile_rank, 20);
    assert!(recipe_item(1).is_none());

    for class_id in [BLACKSMITH_RECIPE_CLASS, CHEF_RECIPE_CLASS] {
        let mut indices: Vec<i32> = RECIPE_ITEMS
            .iter()
            .filter(|item| item.class_id == class_id)
            .map(|item| item.recipe_index)
            .collect();
        let count = indices.len();
        indices.sort_unstable();
        indices.dedup();
        assert_eq!(indices.len(), count, "recipe index {class_id}");
    }

    let unset: Vec<i32> = RECIPE_ITEMS
        .iter()
        .filter(|item| item.min_profile_rank == 0)
        .map(|item| item.id)
        .collect();
    assert_eq!(unset, vec![23872], "the one absent MinProfileRank field");
}

#[test]
fn two_recovered_records_cross_the_shard_classes() {
    let anomalies: Vec<(i32, i32, &str)> = cross_class_recipes()
        .map(|item| (item.id, item.class_id, item.name))
        .collect();
    assert_eq!(
        anomalies,
        vec![
            (869, 58, "Chocolate Bumbleberry Mousse Recipe"),
            (2518, 59, "Chivalrous Guardian Hammer Recipe"),
        ]
    );
}

#[test]
fn rank_ladder_is_the_recovered_min_profile_rank_data() {
    let blacksmith: Vec<(u8, i32)> = rank_gated_recipes(BLACKSMITH_PROFILE)
        .map(|item| (item.min_profile_rank, item.id))
        .collect();
    assert_eq!(
        blacksmith,
        vec![
            (4, 23882),
            (5, 2514),
            (10, 49385),
            (15, 2519),
            (15, 49382),
            (20, 2523),
            (20, 2525),
            (20, 49383),
            (20, 49384),
        ]
    );
    let chef: Vec<(u8, i32)> = rank_gated_recipes(CHEF_PROFILE)
        .map(|item| (item.min_profile_rank, item.id))
        .collect();
    assert_eq!(
        chef,
        vec![
            (5, 849),
            (5, 851),
            (10, 854),
            (10, 857),
            (15, 852),
            (15, 859),
            (15, 863),
            (20, 865),
            (20, 867),
        ]
    );
    assert!(rank_gated_recipes(43).next().is_none());
}

#[test]
fn recipe_access_uses_the_recovered_min_profile_rank() {
    assert_eq!(
        recipe_access(BLACKSMITH_PROFILE, 4, 2514),
        RecipeAccess::BelowRequiredRank {
            required_rank: 5,
            current_rank: 4,
        }
    );
    assert_eq!(
        recipe_access(BLACKSMITH_PROFILE, 5, 2514),
        RecipeAccess::Allowed
    );
    assert_eq!(
        recipe_access(CHEF_PROFILE, 19, 865),
        RecipeAccess::BelowRequiredRank {
            required_rank: 20,
            current_rank: 19,
        }
    );
    assert_eq!(recipe_access(CHEF_PROFILE, 20, 865), RecipeAccess::Allowed);
    assert_eq!(
        recipe_access(BLACKSMITH_PROFILE, 1, 2506),
        RecipeAccess::Allowed
    );
    // The recovered maximum job rank is 20; higher requests clamp to it.
    assert_eq!(
        recipe_access(BLACKSMITH_PROFILE, 200, 2523),
        RecipeAccess::Allowed
    );
    // Rank 0 is below the recovered rank floor of 1.
    assert_eq!(
        recipe_access(BLACKSMITH_PROFILE, 0, 2506),
        RecipeAccess::BelowRequiredRank {
            required_rank: 1,
            current_rank: 0,
        }
    );
    // The one absent MinProfileRank field defaults to 0 and still needs rank 1.
    assert_eq!(
        recipe_access(BLACKSMITH_PROFILE, 1, 23872),
        RecipeAccess::Allowed
    );
    assert_eq!(
        recipe_access(BLACKSMITH_PROFILE, 0, 23872),
        RecipeAccess::BelowRequiredRank {
            required_rank: 1,
            current_rank: 0,
        }
    );
}

#[test]
fn recipe_access_rejects_the_wrong_job_from_the_recovered_class_owner() {
    assert_eq!(
        recipe_access(BLACKSMITH_PROFILE, 20, 843),
        RecipeAccess::WrongJob { owner_profile: 45 }
    );
    assert_eq!(
        recipe_access(CHEF_PROFILE, 20, 2505),
        RecipeAccess::WrongJob { owner_profile: 16 }
    );
    // The mousse sits in the Blacksmith shard class, so the recovered owner rule
    // gates it by Blacksmith rank and reports the mismatch instead of guessing.
    assert_eq!(
        recipe_access(CHEF_PROFILE, 20, 869),
        RecipeAccess::WrongJob { owner_profile: 16 }
    );
    assert_eq!(
        recipe_access(BLACKSMITH_PROFILE, 1, 869),
        RecipeAccess::Allowed
    );
    // The class 59 hammer is a Blacksmith name under the Chef shard class, so
    // the recovered owner rule gates it by Chef rank like the mousse above.
    assert_eq!(
        recipe_access(BLACKSMITH_PROFILE, 20, 2518),
        RecipeAccess::WrongJob { owner_profile: 45 }
    );
    assert_eq!(recipe_access(CHEF_PROFILE, 1, 2518), RecipeAccess::Allowed);
    // An unmapped profile is told which recovered job owns the recipe.
    assert_eq!(
        recipe_access(43, 20, 2505),
        RecipeAccess::WrongJob { owner_profile: 16 }
    );
}

#[test]
fn unknown_recipes_and_rows_fail_closed() {
    assert_eq!(
        recipe_access(CHEF_PROFILE, 20, 999_999),
        RecipeAccess::UnknownRecipe
    );
    assert_eq!(
        wiki_gate_access(CHEF_PROFILE, 20, 1234),
        RecipeAccess::UnknownRecipe
    );
    assert!(wiki_gate(1234).is_none());
}

#[test]
fn wiki_gates_carry_the_tutorial_and_starnut_sundae_rows() {
    assert_eq!(WIKI_GATES.len(), 2);
    let stew = wiki_gate(1232).unwrap();
    assert_eq!(stew.name, "Strong Arm Stew");
    assert_eq!(stew.recipe_name_id, 1232);
    assert_eq!(stew.profile_id, CHEF_PROFILE);
    assert_eq!(stew.required_rank, None);
    assert_eq!(stew.minigame_row, Some(46));
    assert_eq!(stew.minigame_difficulty, Some(1));

    let sundae = wiki_gate(38658).unwrap();
    assert_eq!(sundae.name, "Starnut Sundae");
    assert_eq!(sundae.recipe_name_id, 387777);
    assert_eq!(sundae.profile_id, CHEF_PROFILE);
    assert_eq!(sundae.required_rank, Some(10));
    assert_eq!(sundae.minigame_row, Some(186));
    assert_eq!(sundae.minigame_difficulty, Some(3));

    assert_eq!(
        wiki_gate_access(CHEF_PROFILE, 1, 1232),
        RecipeAccess::Allowed
    );
    assert_eq!(
        wiki_gate_access(CHEF_PROFILE, 0, 1232),
        RecipeAccess::BelowRequiredRank {
            required_rank: 1,
            current_rank: 0,
        }
    );
    assert_eq!(
        wiki_gate_access(CHEF_PROFILE, 9, 38658),
        RecipeAccess::BelowRequiredRank {
            required_rank: 10,
            current_rank: 9,
        }
    );
    assert_eq!(
        wiki_gate_access(CHEF_PROFILE, 10, 38658),
        RecipeAccess::Allowed
    );
    assert_eq!(
        wiki_gate_access(BLACKSMITH_PROFILE, 20, 38658),
        RecipeAccess::WrongJob { owner_profile: 45 }
    );
}

#[test]
fn level_titles_are_the_recovered_locale_rows() {
    assert_eq!(LEVEL_TITLES.len(), 10);
    let rows: Vec<(i32, u8, i32, i32)> = LEVEL_TITLES
        .iter()
        .map(|row| (row.profile_id, row.level, row.title_id, row.requirement_id))
        .collect();
    assert_eq!(
        rows,
        vec![
            (16, 4, 34060, 34140),
            (16, 8, 34061, 34141),
            (16, 12, 34062, 34142),
            (16, 16, 34063, 34143),
            (16, 20, 34064, 34144),
            (45, 4, 34075, 34155),
            (45, 8, 34076, 34156),
            (45, 12, 34077, 34157),
            (45, 16, 34078, 34158),
            (45, 20, 34079, 34159),
        ]
    );
    assert_eq!(MAX_JOB_RANK, 20);
    assert!(level_title(CHEF_PROFILE, 3).is_none());
    assert_eq!(level_title(CHEF_PROFILE, 4).unwrap().title_id, 34075);
    assert_eq!(level_title(CHEF_PROFILE, 7).unwrap().title_id, 34075);
    assert_eq!(level_title(CHEF_PROFILE, 8).unwrap().title_id, 34076);
    assert_eq!(level_title(BLACKSMITH_PROFILE, 19).unwrap().title_id, 34063);
    assert_eq!(level_title(BLACKSMITH_PROFILE, 20).unwrap().title_id, 34064);
    // Requests above the recovered maximum stay at the level 20 title.
    assert_eq!(level_title(BLACKSMITH_PROFILE, 21).unwrap().title_id, 34064);
    assert_eq!(level_title(BLACKSMITH_PROFILE, 0), None);
    assert_eq!(level_title(43, 20), None);
}

#[test]
fn level_gate_quests_pair_the_chef_and_blacksmith_rank_quests() {
    assert_eq!(LEVEL_GATE_QUESTS.len(), 8);
    let chef = level_gate_quest(CHEF_PROFILE, 5).unwrap();
    assert_eq!((chef.quest_id, chef.quest), (387247, "Chef: Reach Level 5"));
    assert_eq!(chef.objective_id, 387267);
    assert_eq!(
        chef.objective,
        "Reach Level 5 as a Chef, then return to Bordon Cloo."
    );
    let blacksmith = level_gate_quest(BLACKSMITH_PROFILE, 20).unwrap();
    assert_eq!(
        (blacksmith.quest_id, blacksmith.quest),
        (387563, "Blacksmith: Reach Level 20")
    );
    assert_eq!(blacksmith.objective_id, 387641);
    for (profile_id, expected, level) in [
        (CHEF_PROFILE, 387255, 10),
        (CHEF_PROFILE, 387259, 15),
        (CHEF_PROFILE, 387263, 20),
        (BLACKSMITH_PROFILE, 387551, 5),
        (BLACKSMITH_PROFILE, 387555, 10),
        (BLACKSMITH_PROFILE, 387559, 15),
    ] {
        let row = level_gate_quest(profile_id, level).unwrap();
        assert_eq!(row.quest_id, expected);
        assert_eq!(row.required_level, level);
    }
    let levels: Vec<(i32, u8)> = LEVEL_GATE_QUESTS
        .iter()
        .map(|row| (row.profile_id, row.required_level))
        .collect();
    assert_eq!(
        levels,
        vec![
            (16, 5),
            (16, 10),
            (16, 15),
            (16, 20),
            (45, 5),
            (45, 10),
            (45, 15),
            (45, 20),
        ]
    );
    assert!(level_gate_quest(CHEF_PROFILE, 4).is_none());
    assert!(level_gate_quest(43, 5).is_none());
}

#[test]
fn accepted_crafting_profiles_mirror_the_live_services_without_grants() {
    assert_eq!(ACCEPTED_COOKING_PROFILE, CHEF_PROFILE);
    assert_eq!(ACCEPTED_FORGING_PROFILE, BLACKSMITH_PROFILE);
    assert_eq!(
        (ACCEPTED_COOKING_PROFILE, ACCEPTED_FORGING_PROFILE),
        (45, 16)
    );
    // The validator can only allow or refuse; a grant variant would not compile.
    for access in [
        recipe_access(CHEF_PROFILE, 20, 843),
        recipe_access(CHEF_PROFILE, 1, 843),
        wiki_gate_access(CHEF_PROFILE, 10, 38658),
    ] {
        match access {
            RecipeAccess::Allowed
            | RecipeAccess::WrongJob { .. }
            | RecipeAccess::BelowRequiredRank { .. }
            | RecipeAccess::UnknownRecipe => {}
        }
    }
}
