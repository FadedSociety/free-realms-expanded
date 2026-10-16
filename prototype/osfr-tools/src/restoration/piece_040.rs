//! Work item 040: Chef/Blacksmith crafting-job progression.
//!
//! Recovered original facts, all copied from pinned sources listed in
//! `notes/Research/Pieces/040 Crafting job progression.md`:
//!
//! * `CRAFTING_JOBS` are the two crafting job profiles: Chef 45 (name id 2856)
//!   and Blacksmith 16 (name id 2614). `src/cooking.rs` already requires
//!   profile 45 and `src/forging.rs` profile 16; the constants here mirror those
//!   accepted services and the piece checker binds them to the live source.
//! * `SHARD_CLASSES` come from the pinned `ItemClasses.txt`: class 58
//!   `Blacksmith Shards` is owned by the Blacksmith profile and class 59
//!   `Chef Shards` by Chef. Recipe items are `Class 58`/`Class 59`,
//!   `CategoryId 17`, `Type 5`, single stack, no-sale, no-trade, cost 540 with
//!   a 10% member discount and `Param1` as their own recipe index.
//! * `RECIPE_ITEMS` are the 49 recoverable recipe records and their recovered
//!   `MinProfileRank` job-rank gate. Rank 0 is the one absent JSON field
//!   (`23872`), which serializes as the default 0.
//! * `RECIPE_ITEMS` are ordered by shard class, then recovered rank, then id,
//!   so the rank ladder reads top to bottom.
//! * `LEVEL_TITLES` pair the recovered `Chef: Level 4..20` / `Blacksmith:
//!   Level 4..20` title strings with the matching `Reach Level 4..20 as a
//!   Chef/Blacksmith.` requirement strings.
//! * `LEVEL_GATE_QUESTS` are the recovered mainline rank quests at levels
//!   5/10/15/20 for both jobs.
//! * `WIKI_GATES` are the two recovered recipe acquisition rows: Strong Arm
//!   Stew is the tutorial recipe and Starnut Sundae needs 10 Chef levels. The
//!   recovered `MiniGameData` rows 46/560 (Strong Arm Stew, difficulty 1) and
//!   186 (Starnut Sundae, difficulty 3) bind those names to minigames.
//!
//! The rule enforced by `recipe_access`/`wiki_gate_access` is reconstructed
//! policy over the recovered data: the recipe's recovered shard class owner
//! must be the requesting job, rank 0 is below the recovered rank floor of 1,
//! and requests above the recovered maximum rank 20 clamp to 20. Recipes,
//! ranks and minigames with no recovered row are refused as `UnknownRecipe`
//! rather than assumed unlocked. Nothing here grants items, coins or
//! experience, and no native acceptance is claimed.

/// Recovered Chef job profile, the requirement of the accepted cooking service.
pub const CHEF_PROFILE: i32 = 45;
/// Recovered Blacksmith job profile, the requirement of the accepted forging service.
pub const BLACKSMITH_PROFILE: i32 = 16;
/// Recovered Chef recipe shard class (`Chef Shards`).
pub const CHEF_RECIPE_CLASS: i32 = 59;
/// Recovered Blacksmith recipe shard class (`Blacksmith Shards`).
pub const BLACKSMITH_RECIPE_CLASS: i32 = 58;
/// Recovered maximum job rank; the level 20 title is the highest recovered row.
pub const MAX_JOB_RANK: u8 = 20;
/// Mirrors `PROFILE` in `src/cooking.rs`; the piece checker fails on drift.
pub const ACCEPTED_COOKING_PROFILE: i32 = CHEF_PROFILE;
/// Mirrors `PROFILE` in `src/forging.rs`; the piece checker fails on drift.
pub const ACCEPTED_FORGING_PROFILE: i32 = BLACKSMITH_PROFILE;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CraftingJob {
    pub profile_id: i32,
    pub name_id: i32,
    pub name: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShardClass {
    pub class_id: i32,
    pub name_id: i32,
    pub name: &'static str,
    pub owner_profile: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecipeItem {
    pub id: i32,
    pub name_id: i32,
    pub name: &'static str,
    pub recipe_index: i32,
    pub class_id: i32,
    /// Recovered `MinProfileRank`; 0 is the one absent field's serialized default.
    pub min_profile_rank: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevelTitle {
    pub profile_id: i32,
    pub level: u8,
    pub title_id: i32,
    pub title: &'static str,
    pub requirement_id: i32,
    pub requirement: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevelGateQuest {
    pub profile_id: i32,
    pub required_level: u8,
    pub quest_id: i32,
    pub quest: &'static str,
    pub objective_id: i32,
    pub objective: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WikiGate {
    pub name_id: i32,
    pub name: &'static str,
    pub recipe_name_id: i32,
    pub profile_id: i32,
    pub required_rank: Option<u8>,
    pub minigame_row: Option<i32>,
    pub minigame_difficulty: Option<u8>,
    pub evidence: &'static str,
}

/// Reconstructed access verdict over the recovered rank data. No variant grants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecipeAccess {
    Allowed,
    WrongJob { owner_profile: i32 },
    BelowRequiredRank { required_rank: u8, current_rank: u8 },
    UnknownRecipe,
}

pub const CRAFTING_JOBS: &[CraftingJob] = &[
    CraftingJob {
        profile_id: 16,
        name_id: 2614,
        name: "Blacksmith",
    },
    CraftingJob {
        profile_id: 45,
        name_id: 2856,
        name: "Chef",
    },
];

pub const SHARD_CLASSES: &[ShardClass] = &[
    ShardClass {
        class_id: 58,
        name_id: 2561,
        name: "Blacksmith Shards",
        owner_profile: 16,
    },
    ShardClass {
        class_id: 59,
        name_id: 2562,
        name: "Chef Shards",
        owner_profile: 45,
    },
];

/// Recovered recipe records ordered by shard class, recovered rank, then id.
pub const RECIPE_ITEMS: &[RecipeItem] = &[
    RecipeItem {
        id: 23872,
        name_id: 26636,
        name: "Dust Storm Flair Shard Recipe",
        recipe_index: 155,
        class_id: 58,
        min_profile_rank: 0,
    },
    RecipeItem {
        id: 869,
        name_id: 1536,
        name: "Chocolate Bumbleberry Mousse Recipe",
        recipe_index: 59,
        class_id: 58,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 2505,
        name_id: 1564,
        name: "Quick Shot Bow Recipe",
        recipe_index: 10,
        class_id: 58,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 2506,
        name_id: 1565,
        name: "Spunky Scrapper Hammer Recipe",
        recipe_index: 11,
        class_id: 58,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 2507,
        name_id: 1566,
        name: "First Aid Scalpel Recipe",
        recipe_index: 12,
        class_id: 58,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 2508,
        name_id: 1567,
        name: "Dancing Monkey Bokken Recipe",
        recipe_index: 13,
        class_id: 58,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 2509,
        name_id: 1568,
        name: "Power Charmed Wand Recipe",
        recipe_index: 14,
        class_id: 58,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 2510,
        name_id: 1569,
        name: "Fast Blast Bow Recipe",
        recipe_index: 15,
        class_id: 58,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 2512,
        name_id: 1571,
        name: "Field Support Hammer Recipe",
        recipe_index: 17,
        class_id: 58,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 2513,
        name_id: 1572,
        name: "Biting Mongoose Blade Recipe",
        recipe_index: 18,
        class_id: 58,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 2515,
        name_id: 1574,
        name: "Alley Fighter Hammer Recipe",
        recipe_index: 20,
        class_id: 58,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 2516,
        name_id: 1575,
        name: "Anguish Aid Bonesaw Recipe",
        recipe_index: 21,
        class_id: 58,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 2517,
        name_id: 1576,
        name: "Diving Hawk Scythe Recipe",
        recipe_index: 22,
        class_id: 58,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 2520,
        name_id: 1579,
        name: "Pain Relief Bonesaw Recipe",
        recipe_index: 25,
        class_id: 58,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 2521,
        name_id: 1580,
        name: "Stalking Panther Scythe Recipe",
        recipe_index: 26,
        class_id: 58,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 2522,
        name_id: 1581,
        name: "Stalwart Protector Hammer Recipe",
        recipe_index: 27,
        class_id: 58,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 2524,
        name_id: 1583,
        name: "Trauma Support Shockrod Recipe",
        recipe_index: 29,
        class_id: 58,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 2526,
        name_id: 1585,
        name: "Intrepid Protector Axe Recipe",
        recipe_index: 31,
        class_id: 58,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 23882,
        name_id: 26646,
        name: "Manguana Sunstone Recipe",
        recipe_index: 165,
        class_id: 58,
        min_profile_rank: 4,
    },
    RecipeItem {
        id: 2514,
        name_id: 1573,
        name: "Lunar Enchanted Wand Recipe",
        recipe_index: 19,
        class_id: 58,
        min_profile_rank: 5,
    },
    RecipeItem {
        id: 49385,
        name_id: 384393,
        name: "Eclipse Infused Wand Recipe",
        recipe_index: 64,
        class_id: 58,
        min_profile_rank: 10,
    },
    RecipeItem {
        id: 2519,
        name_id: 1578,
        name: "Strong Scrapper Hammer Recipe",
        recipe_index: 24,
        class_id: 58,
        min_profile_rank: 15,
    },
    RecipeItem {
        id: 49382,
        name_id: 384403,
        name: "Savage Sting Bow Recipe",
        recipe_index: 65,
        class_id: 58,
        min_profile_rank: 15,
    },
    RecipeItem {
        id: 2523,
        name_id: 1582,
        name: "Frenzy Fighter Hammer Recipe",
        recipe_index: 28,
        class_id: 58,
        min_profile_rank: 20,
    },
    RecipeItem {
        id: 2525,
        name_id: 1584,
        name: "Striking Serpent Sword Recipe",
        recipe_index: 30,
        class_id: 58,
        min_profile_rank: 20,
    },
    RecipeItem {
        id: 49383,
        name_id: 384404,
        name: "Brutal Blast Bow Recipe",
        recipe_index: 67,
        class_id: 58,
        min_profile_rank: 20,
    },
    RecipeItem {
        id: 49384,
        name_id: 384405,
        name: "Void Woven Wand Recipe",
        recipe_index: 68,
        class_id: 58,
        min_profile_rank: 20,
    },
    RecipeItem {
        id: 843,
        name_id: 1510,
        name: "Spiralmint Steak Recipe",
        recipe_index: 33,
        class_id: 59,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 844,
        name_id: 1511,
        name: "Savory Spiralmint Soup Recipe",
        recipe_index: 34,
        class_id: 59,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 845,
        name_id: 1512,
        name: "Gummyfruit Gumbo Recipe",
        recipe_index: 35,
        class_id: 59,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 847,
        name_id: 1514,
        name: "Tangy Tumtumnip Mishmash Recipe",
        recipe_index: 37,
        class_id: 59,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 848,
        name_id: 1515,
        name: "Burnetta's Curious Creation Recipe",
        recipe_index: 38,
        class_id: 59,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 853,
        name_id: 1520,
        name: "Snowberry Top`ped Sundae Recipe",
        recipe_index: 43,
        class_id: 59,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 855,
        name_id: 1522,
        name: "Fried Starnut Stick Recipe",
        recipe_index: 45,
        class_id: 59,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 858,
        name_id: 1525,
        name: "Rainleaf Wrapped Dumpling Recipe",
        recipe_index: 48,
        class_id: 59,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 860,
        name_id: 1527,
        name: "Royal Stew du Jour Recipe",
        recipe_index: 50,
        class_id: 59,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 861,
        name_id: 1528,
        name: "Seared Stainberry Recipe",
        recipe_index: 51,
        class_id: 59,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 862,
        name_id: 1529,
        name: "Spicy Vegetable Stirfry Recipe",
        recipe_index: 52,
        class_id: 59,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 864,
        name_id: 1531,
        name: "De'Flor Family Fried Dumplings Recipe",
        recipe_index: 54,
        class_id: 59,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 2518,
        name_id: 1577,
        name: "Chivalrous Guardian Hammer Recipe",
        recipe_index: 23,
        class_id: 59,
        min_profile_rank: 1,
    },
    RecipeItem {
        id: 849,
        name_id: 1516,
        name: "Crispy Choychoy Delight Recipe",
        recipe_index: 39,
        class_id: 59,
        min_profile_rank: 5,
    },
    RecipeItem {
        id: 851,
        name_id: 1518,
        name: "Slugmud Surprise Recipe",
        recipe_index: 41,
        class_id: 59,
        min_profile_rank: 5,
    },
    RecipeItem {
        id: 854,
        name_id: 1521,
        name: "Sizzling Snowberry Saute Recipe",
        recipe_index: 44,
        class_id: 59,
        min_profile_rank: 10,
    },
    RecipeItem {
        id: 857,
        name_id: 1524,
        name: "Stainberry Chocolate Gumdrop Recipe",
        recipe_index: 47,
        class_id: 59,
        min_profile_rank: 10,
    },
    RecipeItem {
        id: 852,
        name_id: 1519,
        name: "Glerga's Gourmet Goulash Recipe",
        recipe_index: 42,
        class_id: 59,
        min_profile_rank: 15,
    },
    RecipeItem {
        id: 859,
        name_id: 1526,
        name: "Merri's Marinated Mystery Meat Recipe",
        recipe_index: 49,
        class_id: 59,
        min_profile_rank: 15,
    },
    RecipeItem {
        id: 863,
        name_id: 1530,
        name: "Oddly Colored Omelette Recipe",
        recipe_index: 53,
        class_id: 59,
        min_profile_rank: 15,
    },
    RecipeItem {
        id: 865,
        name_id: 1532,
        name: "Honey Roasted Moonbean Recipe",
        recipe_index: 55,
        class_id: 59,
        min_profile_rank: 20,
    },
    RecipeItem {
        id: 867,
        name_id: 1534,
        name: "Candied Briarmelon Recipe",
        recipe_index: 57,
        class_id: 59,
        min_profile_rank: 20,
    },
];

/// The two recovered records stored under the other job's shard class.
pub const CROSS_CLASS_RECIPE_IDS: [i32; 2] = [869, 2518];

pub const LEVEL_TITLES: &[LevelTitle] = &[
    LevelTitle {
        profile_id: 16,
        level: 4,
        title_id: 34060,
        title: "Blacksmith: Level 4",
        requirement_id: 34140,
        requirement: "Reach Level 4 as a Blacksmith.",
    },
    LevelTitle {
        profile_id: 16,
        level: 8,
        title_id: 34061,
        title: "Blacksmith: Level 8",
        requirement_id: 34141,
        requirement: "Reach Level 8 as a Blacksmith.",
    },
    LevelTitle {
        profile_id: 16,
        level: 12,
        title_id: 34062,
        title: "Blacksmith: Level 12",
        requirement_id: 34142,
        requirement: "Reach Level 12 as a Blacksmith.",
    },
    LevelTitle {
        profile_id: 16,
        level: 16,
        title_id: 34063,
        title: "Blacksmith: Level 16",
        requirement_id: 34143,
        requirement: "Reach Level 16 as a Blacksmith.",
    },
    LevelTitle {
        profile_id: 16,
        level: 20,
        title_id: 34064,
        title: "Blacksmith: Level 20",
        requirement_id: 34144,
        requirement: "Reach Level 20 as a Blacksmith.",
    },
    LevelTitle {
        profile_id: 45,
        level: 4,
        title_id: 34075,
        title: "Chef: Level 4",
        requirement_id: 34155,
        requirement: "Reach Level 4 as a Chef.",
    },
    LevelTitle {
        profile_id: 45,
        level: 8,
        title_id: 34076,
        title: "Chef: Level 8",
        requirement_id: 34156,
        requirement: "Reach Level 8 as a Chef.",
    },
    LevelTitle {
        profile_id: 45,
        level: 12,
        title_id: 34077,
        title: "Chef: Level 12",
        requirement_id: 34157,
        requirement: "Reach Level 12 as a Chef.",
    },
    LevelTitle {
        profile_id: 45,
        level: 16,
        title_id: 34078,
        title: "Chef: Level 16",
        requirement_id: 34158,
        requirement: "Reach Level 16 as a Chef.",
    },
    LevelTitle {
        profile_id: 45,
        level: 20,
        title_id: 34079,
        title: "Chef: Level 20",
        requirement_id: 34159,
        requirement: "Reach Level 20 as a Chef.",
    },
];

pub const LEVEL_GATE_QUESTS: &[LevelGateQuest] = &[
    LevelGateQuest {
        profile_id: 16,
        required_level: 5,
        quest_id: 387551,
        quest: "Blacksmith: Reach Level 5",
        objective_id: 387615,
        objective: "Reach level 5 as a blacksmith, then return to Brody Sparfist.",
    },
    LevelGateQuest {
        profile_id: 16,
        required_level: 10,
        quest_id: 387555,
        quest: "Blacksmith: Reach Level 10",
        objective_id: 387625,
        objective: "Reach level 10 as a blacksmith and return to Derp Axemaker.",
    },
    LevelGateQuest {
        profile_id: 16,
        required_level: 15,
        quest_id: 387559,
        quest: "Blacksmith: Reach Level 15",
        objective_id: 387633,
        objective: "Level up to 15 as a blacksmith, then return to Mili Ironcall.",
    },
    LevelGateQuest {
        profile_id: 16,
        required_level: 20,
        quest_id: 387563,
        quest: "Blacksmith: Reach Level 20",
        objective_id: 387641,
        objective: "Level up to 20 as a blacksmith, then return to Silverray.",
    },
    LevelGateQuest {
        profile_id: 45,
        required_level: 5,
        quest_id: 387247,
        quest: "Chef: Reach Level 5",
        objective_id: 387267,
        objective: "Reach Level 5 as a Chef, then return to Bordon Cloo.",
    },
    LevelGateQuest {
        profile_id: 45,
        required_level: 10,
        quest_id: 387255,
        quest: "Chef: Reach Level 10",
        objective_id: 387273,
        objective: "Reach Level 10 as a Chef, then return to Chef Jarvie.",
    },
    LevelGateQuest {
        profile_id: 45,
        required_level: 15,
        quest_id: 387259,
        quest: "Chef: Reach Level 15",
        objective_id: 387279,
        objective: "Reach Level 15 as a Chef, then return to Lucca De'Flor.",
    },
    LevelGateQuest {
        profile_id: 45,
        required_level: 20,
        quest_id: 387263,
        quest: "Chef: Reach Level 20",
        objective_id: 387285,
        objective: "Reach Level 20 as a Chef, then return to Glerga.",
    },
];

pub const WIKI_GATES: &[WikiGate] = &[
    WikiGate {
        name_id: 1232,
        name: "Strong Arm Stew",
        recipe_name_id: 1232,
        profile_id: 45,
        required_rank: None,
        minigame_row: Some(46),
        minigame_difficulty: Some(1),
        evidence: "wiki List of Recipes (Chef) revision 45318: Where to find: Tutorial",
    },
    WikiGate {
        name_id: 38658,
        name: "Starnut Sundae",
        recipe_name_id: 387777,
        profile_id: 45,
        required_rank: Some(10),
        minigame_row: Some(186),
        minigame_difficulty: Some(3),
        evidence: "wiki List of Recipes (Chef) revision 45318: Where to find: Gain 10 levels in Chef",
    },
];

pub fn crafting_jobs() -> &'static [CraftingJob] {
    CRAFTING_JOBS
}

pub fn crafting_job(profile_id: i32) -> Option<&'static CraftingJob> {
    CRAFTING_JOBS
        .iter()
        .find(|job| job.profile_id == profile_id)
}

pub fn shard_class(class_id: i32) -> Option<&'static ShardClass> {
    SHARD_CLASSES
        .iter()
        .find(|class| class.class_id == class_id)
}

pub fn shard_class_for(profile_id: i32) -> Option<&'static ShardClass> {
    SHARD_CLASSES
        .iter()
        .find(|class| class.owner_profile == profile_id)
}

pub fn recipe_items() -> &'static [RecipeItem] {
    RECIPE_ITEMS
}

pub fn recipe_item(id: i32) -> Option<&'static RecipeItem> {
    RECIPE_ITEMS.iter().find(|item| item.id == id)
}

/// Every recipe item whose recovered shard class belongs to the job.
pub fn recipes_for(profile_id: i32) -> impl Iterator<Item = &'static RecipeItem> {
    RECIPE_ITEMS.iter().filter(move |item| {
        shard_class(item.class_id).is_some_and(|class| class.owner_profile == profile_id)
    })
}

/// The two recovered records stored under the other job's shard class, in id order.
pub fn cross_class_recipes() -> impl Iterator<Item = &'static RecipeItem> {
    CROSS_CLASS_RECIPE_IDS
        .iter()
        .filter_map(|id| recipe_item(*id))
}

/// Recipes of the job with a recovered rank gate above the rank 1 floor,
/// in ascending recovered-rank order.
pub fn rank_gated_recipes(profile_id: i32) -> impl Iterator<Item = &'static RecipeItem> {
    recipes_for(profile_id).filter(move |item| item.min_profile_rank > 1)
}

/// Reconstructed policy: same recovered job owner, rank floor 1, cap 20.
fn rank_access(profile_id: i32, rank: u8, owner_profile: i32, required_rank: u8) -> RecipeAccess {
    if profile_id != owner_profile {
        return RecipeAccess::WrongJob { owner_profile };
    }
    let required_rank = required_rank.max(1);
    let current_rank = rank.min(MAX_JOB_RANK);
    if current_rank < required_rank {
        return RecipeAccess::BelowRequiredRank {
            required_rank,
            current_rank,
        };
    }
    RecipeAccess::Allowed
}

/// Reconstructed policy: the recipe's recovered shard class owner must match
/// the requesting job and the rank must meet the recovered `MinProfileRank`.
pub fn recipe_access(profile_id: i32, rank: u8, item_id: i32) -> RecipeAccess {
    let Some(item) = recipe_item(item_id) else {
        return RecipeAccess::UnknownRecipe;
    };
    let Some(class) = shard_class(item.class_id) else {
        return RecipeAccess::UnknownRecipe;
    };
    rank_access(profile_id, rank, class.owner_profile, item.min_profile_rank)
}

/// The highest recovered level title at or below `level`; levels above the
/// recovered maximum stay at the level 20 title.
pub fn level_title(profile_id: i32, level: u8) -> Option<&'static LevelTitle> {
    let level = level.min(MAX_JOB_RANK);
    LEVEL_TITLES
        .iter()
        .filter(|row| row.profile_id == profile_id && row.level <= level)
        .max_by_key(|row| row.level)
}

pub fn level_gate_quest(profile_id: i32, required_level: u8) -> Option<&'static LevelGateQuest> {
    LEVEL_GATE_QUESTS
        .iter()
        .find(|row| row.profile_id == profile_id && row.required_level == required_level)
}

pub fn wiki_gates() -> &'static [WikiGate] {
    WIKI_GATES
}

pub fn wiki_gate(name_id: i32) -> Option<&'static WikiGate> {
    WIKI_GATES.iter().find(|row| row.name_id == name_id)
}

/// Reconstructed policy for the recovered wiki acquisition rows: no recovered
/// rank requirement means the rank 1 floor, otherwise the recovered level.
pub fn wiki_gate_access(profile_id: i32, rank: u8, name_id: i32) -> RecipeAccess {
    let Some(gate) = wiki_gate(name_id) else {
        return RecipeAccess::UnknownRecipe;
    };
    rank_access(
        profile_id,
        rank,
        gate.profile_id,
        gate.required_rank.unwrap_or(0),
    )
}
