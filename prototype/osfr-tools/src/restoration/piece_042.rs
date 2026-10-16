//! Work item 042: recovered Soccer Star and Postman job progression coverage.
//!
//! Recovered original facts, copied from the pinned sources listed in
//! `notes/Research/Pieces/042 Soccer and postman progression.md`:
//!
//! * Job identity: the preserved locale export resolves `3237`/`3238` to `Postman` and
//!   `4650` to `Soccer Star`; the community `Profiles.json` rows 4 and 52 carry profile type
//!   3/8, hints 384184/416471, trial `true` and members-only `false`. Row 52 reuses the Demo
//!   Derby description id 2920, an inherited placeholder rather than a real Soccer Star blurb.
//! * Level milestones: locale `Postman: Level 4/8/12/16/20` (34115-34119) and
//!   `Soccer Star: Level 4/8/12/16/20` (34120-34124) with the `Reach Level N as a ...`
//!   objectives at 34195-34199 and 34200-34204.
//! * Postman training gates: locale 387931/387935/387939/387943 name the four gate quests and
//!   387932/387936/387940/387944 name the trainer each one returns to. The wiki `Postman` page
//!   lists the same trainers, and the client profile row keeps the XP progress as
//!   `Rank`/`StarsEarned`/`RankPercent`, which the server computes.
//! * Soccer Star tutorial: `CodeStringMappings.txt` resolves `SoccerTut0001..0007` and
//!   `SoccerTutIcon0001..0004`; `ScriptsBase.lua` declares `DS_SOCCER_TUTORIAL = 25` for
//!   `BaseClient.SoccerProcessor.SoccerPlayerActions`, a seven page tutorial, and advances a page
//!   only on the recovered action-column pairs.
//! * Postman mail sorting tutorial: `MailSortingTutorialOneTop/Bottom` resolve to 19323/19302.
//! * Activities: the extracted minigame table holds the three type-12 soccer activities 42/43/44
//!   and the Postman mail activities, including `Can You Deliver?` (238).
//!
//! BOUNDARIES: this module is recovery metadata plus two pure helpers. It grants no job, item,
//! currency or experience. The numeric XP curve, the per-job unlock quest ids and the meaning of
//! the recovered Soccer action columns were not recovered; they are reported as gaps.

/// Postman profile id (community `Profiles.json`, also a captured profile payload).
pub const POSTMAN_PROFILE_ID: i32 = 4;
/// Soccer Star profile id (community `Profiles.json`, also a captured profile payload).
pub const SOCCER_STAR_PROFILE_ID: i32 = 52;

/// Highest job rank the wiki documents ("The maximum of a job's level is 20").
pub const MAX_JOB_LEVEL: u8 = 20;
/// Non-member job cap the wiki documents ("level 4 if you are not a member for most jobs").
pub const NON_MEMBER_JOB_LEVEL_CAP: u8 = 4;

/// `Profile_DS.Rank` - the job rank the server reports; the client never derives it.
pub const PROFILE_RANK_COLUMN: usize = 4;
/// `Profile_DS.StarsAvailable` - unspent ability points reported by the server.
pub const PROFILE_STARS_AVAILABLE_COLUMN: usize = 5;
/// `Profile_DS.StarsEarned` - lifetime stars reported by the server.
pub const PROFILE_STARS_EARNED_COLUMN: usize = 11;
/// `Profile_DS.RankPercent` - the 0..1 progress toward the next rank.
pub const PROFILE_RANK_PERCENT_COLUMN: usize = 12;

/// Where a recovered progression row comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgressionEvidence {
    /// Original client/server text or table preserved in the lab.
    Original,
    /// Community mirror of an original table (`Sanctuary`/`FreeRealms-Legacy`).
    CommunityTable,
    /// Read-only wiki documentation.
    Wiki,
}

/// The XP contract recovered from the client: rank, stars and a progress percentage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum XpCurve {
    /// The XP-to-rank thresholds live on the original server and were not captured.
    Missing,
}

/// A locale string the module quotes verbatim. `validate_locale` enforces equality.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextClaim {
    pub profile_id: i32,
    pub string_id: u32,
    pub text: &'static str,
}

/// A locale string the module only refers to by id. The checker requires it to resolve.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferenceId {
    pub profile_id: i32,
    pub string_id: u32,
    pub role: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevelMilestone {
    pub level: u8,
    pub name_id: u32,
    pub objective_id: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevelGate {
    pub level: u8,
    pub name_id: u32,
    pub objective_id: u32,
    pub gate_dialog_id: u32,
    pub success_dialog_id: u32,
    pub trainer: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProgressionAbility {
    pub name: &'static str,
    pub name_id: Option<u32>,
    pub effect: &'static str,
    pub evidence: ProgressionEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KitItem {
    pub id: i32,
    pub name: &'static str,
    pub name_id: u32,
    pub class_id: i32,
    pub slot: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JobQuest {
    pub name: &'static str,
    pub name_id: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TutorialPage {
    pub page: u8,
    pub key: &'static str,
    pub string_id: u32,
    pub label: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PickupTutorial {
    pub key: &'static str,
    pub string_id: u32,
    pub icon_id: i32,
    pub label: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TutorialText {
    pub key: &'static str,
    pub string_id: u32,
    pub label: &'static str,
}

/// The client tutorial advance rule, recovered from `SoccerTutorial.OnDataChanged`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TutorialAdvance {
    pub page: u8,
    pub columns: &'static [u8],
    pub next_page: u8,
}

/// Everything recovered for one job in this item.
#[derive(Clone, Copy, Debug)]
pub struct JobProgression {
    pub profile_id: i32,
    pub name: &'static str,
    pub name_id: u32,
    pub description_id: u32,
    pub profile_type: i32,
    pub hint_id: u32,
    pub icon_id: i32,
    pub members_only: bool,
    pub trial: bool,
    pub classification: &'static str,
    pub kit_class_id: i32,
    pub kit_class_name_id: u32,
    pub creation_kit: &'static [KitItem],
    pub milestones: &'static [LevelMilestone],
    pub gates: &'static [LevelGate],
    pub questline: &'static [JobQuest],
    pub xp_curve: XpCurve,
    pub unlock_quest_ids_recovered: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProgressionError {
    DuplicateStringId(u32),
    EmptyText {
        string_id: u32,
    },
    MilestoneNotAscending {
        profile_id: i32,
    },
    GateNotAscending {
        profile_id: i32,
    },
    KitClassMismatch {
        item: i32,
        expected: i32,
    },
    TutorialPageGap {
        page: u8,
    },
    TutorialTargetOutOfRange {
        page: u8,
        next_page: u8,
    },
    MissingLocalizedText {
        profile_id: i32,
        string_id: u32,
    },
    LocalizedTextDiffers {
        profile_id: i32,
        string_id: u32,
        expected: &'static str,
        found: String,
    },
}

const fn claim(profile_id: i32, string_id: u32, text: &'static str) -> TextClaim {
    TextClaim {
        profile_id,
        string_id,
        text,
    }
}

const fn reference(profile_id: i32, string_id: u32, role: &'static str) -> ReferenceId {
    ReferenceId {
        profile_id,
        string_id,
        role,
    }
}

const fn milestone(level: u8, name_id: u32, objective_id: u32) -> LevelMilestone {
    LevelMilestone {
        level,
        name_id,
        objective_id,
    }
}

const fn gate(
    level: u8,
    name_id: u32,
    objective_id: u32,
    gate_dialog_id: u32,
    success_dialog_id: u32,
    trainer: &'static str,
) -> LevelGate {
    LevelGate {
        level,
        name_id,
        objective_id,
        gate_dialog_id,
        success_dialog_id,
        trainer,
    }
}

const fn kit_item(id: i32, name: &'static str, name_id: u32, class_id: i32, slot: i32) -> KitItem {
    KitItem {
        id,
        name,
        name_id,
        class_id,
        slot,
    }
}

const fn quest(name: &'static str, name_id: u32) -> JobQuest {
    JobQuest { name, name_id }
}

const fn page(page: u8, key: &'static str, string_id: u32, label: &'static str) -> TutorialPage {
    TutorialPage {
        page,
        key,
        string_id,
        label,
    }
}

const fn pickup(
    key: &'static str,
    string_id: u32,
    icon_id: i32,
    label: &'static str,
) -> PickupTutorial {
    PickupTutorial {
        key,
        string_id,
        icon_id,
        label,
    }
}

/// Locale strings quoted verbatim by this module. The Python checker re-reads these lines and
/// diffs them against the preserved locale export.
pub const TEXT_CLAIMS: &[TextClaim] = &[
    claim(4, 3237, "Postman"),
    claim(4, 3238, "Postman description"),
    claim(52, 4650, "Soccer Star"),
    claim(52, 2920, "Demolition Derby Driver Driver Description"),
    claim(
        4,
        384184,
        "Felipe needs to train more postmen to help out at Pixiewood.",
    ),
    claim(
        52,
        416471,
        "South of Snowhill, Manager Landon is helping teams recruit new soccer stars.",
    ),
    claim(4, 34115, "Postman: Level 4"),
    claim(4, 34116, "Postman: Level 8"),
    claim(4, 34117, "Postman: Level 12"),
    claim(4, 34118, "Postman: Level 16"),
    claim(4, 34119, "Postman: Level 20"),
    claim(4, 34195, "Reach Level 4 as a Postman."),
    claim(4, 34196, "Reach Level 8 as a Postman."),
    claim(4, 34197, "Reach Level 12 as a Postman."),
    claim(4, 34198, "Reach Level 16 as a Postman."),
    claim(4, 34199, "Reach Level 20 as a Postman."),
    claim(52, 34120, "Soccer Star: Level 4"),
    claim(52, 34121, "Soccer Star: Level 8"),
    claim(52, 34122, "Soccer Star: Level 12"),
    claim(52, 34123, "Soccer Star: Level 16"),
    claim(52, 34124, "Soccer Star: Level 20"),
    claim(52, 34200, "Reach Level 4 as a Soccer Star."),
    claim(52, 34201, "Reach Level 8 as a Soccer Star."),
    claim(52, 34202, "Reach Level 12 as a Soccer Star."),
    claim(52, 34203, "Reach Level 16 as a Soccer Star."),
    claim(52, 34204, "Reach Level 20 as a Soccer Star."),
    claim(4, 387931, "Postman: Achieve Level 5"),
    claim(
        4,
        387932,
        "Your training will continue once you have reached level 5 as a postman. Return to Felipe at that time.",
    ),
    claim(4, 387935, "Postman: Achieve Level 10"),
    claim(
        4,
        387936,
        "Your training will continue once you have reached level 10 as a postman. Return to Enve LeStamp at that time.",
    ),
    claim(4, 387939, "Postman: Achieve Level 15"),
    claim(
        4,
        387940,
        "Your training will continue once you have reached level 15 as a postman. Return to Postchug Runalug at that time.",
    ),
    claim(4, 387943, "Postman: Achieve Level 20"),
    claim(
        4,
        387944,
        "Your training will continue once you have reached level 20 as a postman. Return to Brin Galeglow at that time.",
    ),
    claim(4, 5175, "Throw Bone"),
];

/// Locale strings this module refers to by id only. The checker requires a non-empty value.
pub const REFERENCE_IDS: &[ReferenceId] = &[
    reference(4, 387930, "level 5 gate dialog"),
    reference(4, 387933, "level 5 gate success"),
    reference(4, 387934, "level 10 gate dialog"),
    reference(4, 387937, "level 10 gate success"),
    reference(4, 387938, "level 15 gate dialog"),
    reference(4, 387941, "level 15 gate success"),
    reference(4, 387942, "level 20 gate dialog"),
    reference(4, 387945, "level 20 gate success"),
    reference(4, 387947, "level 5 achievement line"),
    reference(4, 387948, "level 5 quest title"),
    reference(4, 387949, "level 5 quest objective"),
    reference(4, 93381, "Can You Deliver? minigame name"),
    reference(4, 93382, "Can You Deliver? minigame description"),
    reference(4, 103829, "unlock objective: deliver the mail"),
    reference(4, 103839, "unlock objective: complete Can You Deliver?"),
    reference(4, 103840, "unlock objective: click the mailboxes"),
    reference(4, 103843, "unlock objective: return to Felipe"),
    reference(4, 103830, "unlock grant line: starting uniform"),
    reference(4, 103831, "unlock grant line: rewarded with the job"),
    reference(4, 9737, "unlock completion line"),
    reference(4, 19323, "mail sorting tutorial top"),
    reference(4, 19302, "mail sorting tutorial bottom"),
    reference(4, 35595, "mail sorting leaderboard category"),
    reference(4, 35596, "mail sorting leaderboard description"),
    reference(4, 6361, "Special Delivery minigame name"),
    reference(4, 6362, "Special Delivery minigame objective"),
    reference(4, 16443, "Ernie's Lost Letters minigame name"),
    reference(4, 115976, "What a Mess! minigame name"),
    reference(4, 15674, "Postal Route: Sanctuary minigame name"),
    reference(52, 407110, "Snowhill Soccer Field POI name"),
    reference(52, 416504, "soccer score tooltip"),
    reference(52, 406669, "soccer tutorial page 1"),
    reference(52, 406670, "soccer tutorial page 2"),
    reference(52, 406671, "soccer tutorial page 3"),
    reference(52, 406672, "soccer tutorial page 4"),
    reference(52, 406673, "soccer tutorial page 5"),
    reference(52, 406674, "soccer tutorial page 6"),
    reference(52, 416748, "soccer tutorial page 7"),
    reference(52, 406675, "soccer pickup: Charge Rate"),
    reference(52, 406676, "soccer pickup: Speed Boost"),
    reference(52, 406677, "soccer pickup: Tackle Power"),
    reference(52, 406678, "soccer pickup: Toughness"),
];

/// `Postman: Level 4/8/12/16/20` from locale 34115-34119 with objectives 34195-34199.
pub const POSTMAN_LEVEL_MILESTONES: &[LevelMilestone] = &[
    milestone(4, 34115, 34195),
    milestone(8, 34116, 34196),
    milestone(12, 34117, 34197),
    milestone(16, 34118, 34198),
    milestone(20, 34119, 34199),
];

/// `Soccer Star: Level 4/8/12/16/20` from locale 34120-34124 with objectives 34200-34204.
pub const SOCCER_STAR_LEVEL_MILESTONES: &[LevelMilestone] = &[
    milestone(4, 34120, 34200),
    milestone(8, 34121, 34201),
    milestone(12, 34122, 34202),
    milestone(16, 34123, 34203),
    milestone(20, 34124, 34204),
];

/// The four recovered Postman training gates. Trainer names come from the objective text and the
/// wiki trainer list; the gate and success dialogs are the neighbouring locale rows.
pub const POSTMAN_LEVEL_GATES: &[LevelGate] = &[
    gate(5, 387931, 387932, 387930, 387933, "Felipe"),
    gate(10, 387935, 387936, 387934, 387937, "Enve LeStamp"),
    gate(15, 387939, 387940, 387938, 387941, "Postchug Runalug"),
    gate(20, 387943, 387944, 387942, 387945, "Brin Galeglow"),
];

/// Postman abilities. `Mail Throw` is documented by the wiki without a locale row of its own;
/// `Throw Bone` resolves to locale 5175.
pub const POSTMAN_SKILLS: &[ProgressionAbility] = &[
    ProgressionAbility {
        name: "Mail Throw",
        name_id: None,
        effect: "Throws the mail into the targeted mailbox",
        evidence: ProgressionEvidence::Wiki,
    },
    ProgressionAbility {
        name: "Throw Bone",
        name_id: Some(5175),
        effect: "Tosses a bone to distract dogs from knocking the postman over",
        evidence: ProgressionEvidence::Wiki,
    },
];

/// The recovered `DefaultItems` of profile 4, resolved against `ClientItemDefinitions.json`.
pub const POSTMAN_CREATION_KIT: &[KitItem] = &[
    kit_item(13137, "Student Postman Hat", 27019, 3, 1),
    kit_item(12827, "Student Postman Shirt", 26718, 3, 3),
    kit_item(13197, "Student Postman Shorts", 27077, 3, 4),
    kit_item(13317, "Student Postman Short Shorts", 27194, 3, 4),
    kit_item(12957, "Student Postman Shoes", 26844, 3, 5),
];

/// The recovered `DefaultItems` of profile 52, resolved against `ClientItemDefinitions.json`.
pub const SOCCER_STAR_CREATION_KIT: &[KitItem] = &[
    kit_item(61907, "Fast Forward Bandana", 404315, 134, 1),
    kit_item(61353, "Fast Forward Jersey", 403755, 134, 3),
    kit_item(62169, "Fast Forward Shorts", 404595, 134, 4),
    kit_item(61609, "Fast Forward Cleats", 404035, 134, 5),
    kit_item(62473, "Fast Forward Shin Guards", 404875, 134, 6),
];

/// The recovered Soccer Star job questline names (locale 407407-407418). Quest ids are missing.
pub const SOCCER_STAR_JOB_QUESTLINE: &[JobQuest] = &[
    quest("NOT LIVE - Soccer Star: Part of the Team", 407407),
    quest("Soccer Star: Snowy Scrimmage", 407408),
    quest("Soccer Star: Goal Stopper", 407409),
    quest("Soccer Star: Briarwood's Fancy Feet", 407410),
    quest("Soccer Star: Returning The Trophy", 407411),
    quest("Soccer Star: Sidekicks Steal", 407412),
    quest("Soccer Star: Mascot Madness", 407413),
    quest("Soccer Star: Slowing The Snowspurs", 407414),
    quest("Soccer Star: Making The Finals", 407415),
    quest("Soccer Star: Ants In The Pants", 407416),
    quest("Soccer Star: Tournament Champion", 407417),
    quest("Soccer Star: Robgoblin Troublemakers", 407418),
];

/// The seven Soccer tutorial pages from `CodeStringMappings.txt` / the locale export.
pub const SOCCER_TUTORIAL_PAGES: &[TutorialPage] = &[
    page(1, "SoccerTut0001", 406669, "Soccer Basics"),
    page(2, "SoccerTut0002", 406670, "Kick"),
    page(3, "SoccerTut0003", 406671, "Super kick"),
    page(4, "SoccerTut0004", 406672, "Pass"),
    page(5, "SoccerTut0005", 406673, "Dodge"),
    page(6, "SoccerTut0006", 406674, "Slide Tackle and Steal"),
    page(7, "SoccerTut0007", 416748, "Power-ups and exit"),
];

/// `Ui.OnSoccerSpawnTutorialPickups` spawns these pickups; icons come from the client Lua.
pub const SOCCER_TUTORIAL_PICKUP_ICONS: &[PickupTutorial] = &[
    pickup("SoccerTutIcon0001", 406675, 11542, "Charge Rate"),
    pickup("SoccerTutIcon0002", 406676, 17353, "Speed Boost"),
    pickup("SoccerTutIcon0004", 406678, 18855, "Toughness"),
];

/// The Postman mail-sorting minigame tutorial strings.
pub const POSTMAN_MAIL_SORTING_TUTORIAL: &[TutorialText] = &[
    TutorialText {
        key: "MailSortingTutorialOneTop",
        string_id: 19323,
        label: "Mail Sorting objective",
    },
    TutorialText {
        key: "MailSortingTutorialOneBottom",
        string_id: 19302,
        label: "Mail Sorting timer warning",
    },
];

/// The client tutorial page count (`pageCount = 7`).
pub const SOCCER_TUTORIAL_PAGE_COUNT: u8 = 7;

/// `DS_SOCCER_TUTORIAL` index and datasource name from the decompiled client Lua.
pub const SOCCER_TUTORIAL_DATASOURCE_INDEX: u8 = 25;
pub const SOCCER_TUTORIAL_DATASOURCE: &str = "BaseClient.SoccerProcessor.SoccerPlayerActions";

/// The tutorial image ids the client loads while the Soccer tutorial runs.
pub const SOCCER_TUTORIAL_IMAGE_IDS: &[i32] = &[20999, 20998, 20248];

/// `SoccerTutorial.OnDataChanged` never advances an unauthenticated column/value pair.
pub const SOCCER_TUTORIAL_ADVANCE: &[TutorialAdvance] = &[
    TutorialAdvance {
        page: 1,
        columns: &[0],
        next_page: 2,
    },
    TutorialAdvance {
        page: 2,
        columns: &[2],
        next_page: 3,
    },
    TutorialAdvance {
        page: 3,
        columns: &[3],
        next_page: 4,
    },
    TutorialAdvance {
        page: 4,
        columns: &[6, 7],
        next_page: 5,
    },
    TutorialAdvance {
        page: 5,
        columns: &[1],
        next_page: 6,
    },
    TutorialAdvance {
        page: 6,
        columns: &[4, 5],
        next_page: 7,
    },
];

/// The `Snowhill Soccer Field` point of interest (community table `PointOfInterests.json`).
pub const SOCCER_FIELD_POI_ID: i32 = 117;
pub const SOCCER_FIELD_POI_LOCATION_ID: i32 = 207771;
pub const SOCCER_FIELD_POI_POSITION: [f32; 3] = [-145.95377, -6.12704, 259.3836];
pub const SOCCER_FIELD_POI_HEADING: f32 = -3.01945;
/// Community breadcrumb quest id; no recovered quest row carries this id.
pub const SOCCER_FIELD_POI_BREADCRUMB_QUEST_ID: i32 = 1881;

/// The three type-12 soccer activities in the extracted minigame table.
pub const SOCCER_ACTIVITY_IDS: &[i32] = &[42, 43, 44];
/// Postman mail activities in the same table, including `Can You Deliver?` (238).
pub const POSTMAN_ACTIVITY_IDS: &[i32] = &[3, 5, 6, 12, 13, 14, 16, 24, 238];

/// No numeric XP curve was recovered. The client only renders server-reported progress.
pub const XP_CURVE_RECOVERED: bool = false;
/// The Soccer tutorial datasource column meaning was not recovered; only the transitions are.
pub const SOCCER_TUTORIAL_SOURCE_COLUMN_MEANING_RECOVERED: bool = false;
/// The Soccer Star questline names were recovered without their quest ids.
pub const SOCCER_STAR_QUESTLINE_QUEST_IDS_RECOVERED: bool = false;
/// No per-job unlock quest id was recovered for either job.
pub const UNLOCK_QUEST_IDS_RECOVERED: bool = false;

/// Postman progression aggregate.
pub const POSTMAN: JobProgression = JobProgression {
    profile_id: POSTMAN_PROFILE_ID,
    name: "Postman",
    name_id: 3237,
    description_id: 3238,
    profile_type: 3,
    hint_id: 384184,
    icon_id: 1339,
    members_only: false,
    trial: true,
    classification: "Freestyle",
    kit_class_id: 3,
    kit_class_name_id: 2923,
    creation_kit: POSTMAN_CREATION_KIT,
    milestones: POSTMAN_LEVEL_MILESTONES,
    gates: POSTMAN_LEVEL_GATES,
    questline: &[],
    xp_curve: XpCurve::Missing,
    unlock_quest_ids_recovered: UNLOCK_QUEST_IDS_RECOVERED,
};

/// Soccer Star progression aggregate. Soccer Star has no recovered per-level training gate; its
/// questline rows carry no ids.
pub const SOCCER_STAR: JobProgression = JobProgression {
    profile_id: SOCCER_STAR_PROFILE_ID,
    name: "Soccer Star",
    name_id: 4650,
    description_id: 2920,
    profile_type: 8,
    hint_id: 416471,
    icon_id: 20743,
    members_only: false,
    trial: true,
    classification: "Freestyle",
    kit_class_id: 134,
    kit_class_name_id: 3922,
    creation_kit: SOCCER_STAR_CREATION_KIT,
    milestones: SOCCER_STAR_LEVEL_MILESTONES,
    gates: &[],
    questline: SOCCER_STAR_JOB_QUESTLINE,
    xp_curve: XpCurve::Missing,
    unlock_quest_ids_recovered: UNLOCK_QUEST_IDS_RECOVERED,
};

/// The two job aggregates this item owns.
pub const JOBS: &[JobProgression] = &[POSTMAN, SOCCER_STAR];

pub fn job_progression(profile_id: i32) -> Option<&'static JobProgression> {
    JOBS.iter().find(|row| row.profile_id == profile_id)
}

pub fn level_milestones(profile_id: i32) -> &'static [LevelMilestone] {
    job_progression(profile_id).map_or(&[], |row| row.milestones)
}

pub fn level_gates(profile_id: i32) -> &'static [LevelGate] {
    job_progression(profile_id).map_or(&[], |row| row.gates)
}

/// The Postman training gate a job level opens, if any. Gates are not cumulative: level 10 opens
/// the level 10 gate and leaves the level 5 gate completed behind it.
pub fn level_gate(level: u8) -> Option<&'static LevelGate> {
    POSTMAN_LEVEL_GATES.iter().find(|row| row.level == level)
}

/// Every Postman gate whose level is at or below `level`, in table order.
pub fn unlocked_level_gates(level: u8) -> Vec<u8> {
    POSTMAN_LEVEL_GATES
        .iter()
        .filter(|row| row.level <= level)
        .map(|row| row.level)
        .collect()
}

/// A milestone quest is offered once the job reaches its level.
pub fn milestone_reached(level: u8, milestone: &LevelMilestone) -> bool {
    level >= milestone.level
}

/// The page the Soccer tutorial moves to when the client reports `action_column` while on `page`.
/// Only the captured client pairing advances the tutorial; anything else returns `None`.
pub fn soccer_tutorial_advance(page: u8, action_column: u8) -> Option<u8> {
    SOCCER_TUTORIAL_ADVANCE
        .iter()
        .find(|rule| rule.page == page && rule.columns.contains(&action_column))
        .map(|rule| rule.next_page)
}

/// Structural invariants over the recovered tables. Text is checked by `validate_locale`.
pub fn validate() -> Result<(), ProgressionError> {
    let mut seen: Vec<u32> = Vec::new();
    for row in TEXT_CLAIMS {
        if seen.contains(&row.string_id) {
            return Err(ProgressionError::DuplicateStringId(row.string_id));
        }
        if row.text.trim().is_empty() {
            return Err(ProgressionError::EmptyText {
                string_id: row.string_id,
            });
        }
        seen.push(row.string_id);
    }
    for row in REFERENCE_IDS {
        if seen.contains(&row.string_id) {
            return Err(ProgressionError::DuplicateStringId(row.string_id));
        }
        seen.push(row.string_id);
    }
    for job in JOBS {
        let mut last = 0u8;
        for row in job.milestones {
            if row.level <= last {
                return Err(ProgressionError::MilestoneNotAscending {
                    profile_id: job.profile_id,
                });
            }
            last = row.level;
        }
        let mut last_gate = 0u8;
        for row in job.gates {
            if row.level <= last_gate {
                return Err(ProgressionError::GateNotAscending {
                    profile_id: job.profile_id,
                });
            }
            last_gate = row.level;
        }
        for item in job.creation_kit {
            if item.class_id != job.kit_class_id {
                return Err(ProgressionError::KitClassMismatch {
                    item: item.id,
                    expected: job.kit_class_id,
                });
            }
        }
    }
    for (index, row) in SOCCER_TUTORIAL_PAGES.iter().enumerate() {
        if row.page != (index + 1) as u8 {
            return Err(ProgressionError::TutorialPageGap { page: row.page });
        }
    }
    for rule in SOCCER_TUTORIAL_ADVANCE {
        if rule.next_page > SOCCER_TUTORIAL_PAGE_COUNT || rule.columns.is_empty() {
            return Err(ProgressionError::TutorialTargetOutOfRange {
                page: rule.page,
                next_page: rule.next_page,
            });
        }
    }
    Ok(())
}

/// Every quoted string must equal the preserved locale, and every referenced id must resolve.
pub fn validate_locale(lookup: impl Fn(u32) -> Option<String>) -> Result<(), ProgressionError> {
    for row in TEXT_CLAIMS {
        match lookup(row.string_id) {
            None => {
                return Err(ProgressionError::MissingLocalizedText {
                    profile_id: row.profile_id,
                    string_id: row.string_id,
                });
            }
            Some(found) if found != row.text => {
                return Err(ProgressionError::LocalizedTextDiffers {
                    profile_id: row.profile_id,
                    string_id: row.string_id,
                    expected: row.text,
                    found,
                });
            }
            Some(_) => {}
        }
    }
    Ok(())
}
