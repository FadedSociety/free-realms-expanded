//! Piece 093 - emotes and social actions: the recovered emote command table,
//! the client quick-chat rows that carry each animation id, the animation-slot
//! names and the restriction flags the shipped client data actually holds.
//!
//! ORIGINAL facts implemented here (provenance in
//! `notes/Research/Pieces/093 Emotes and social actions.md`):
//!
//! * `decompiled/client-ui/ScriptsBase.lua`, `EmoteHandler.emotes`: 72
//!   `{cmd = "...", id = N}` rows. `Init` binds them with `BindCommandToLua` in
//!   array order, so a repeated command is overwritten by the last row:
//!   `curious` is bound to 2469 first and to 148 second.
//! * `decompiled/server/Gateway/Gateway.MapChat/ChatSystem.cs`
//!   `SendQuickChatPacket`: the tunneled `BaseClient.QuickChatData` payload
//!   (opcode 67, subtype 1, 2470 rows x 10 little-endian u32 columns) that the
//!   client reads in `ChatLog:AddQuickChatPhrases`. Column 0 is the row id,
//!   column 2 the display string, column 3 the optional say string, column 4 the
//!   animation id and column 5 the menu parent.
//! * `client/Resources/AnimationTypes.xml` `Emotes` block: the animation slot
//!   names behind those ids and their `hideSlotType` / `freezeMovement` /
//!   `disableBoneMasking` / `priority` attributes.
//! * `client/Resources/AnimationTransitionRules.xml`: `Any -> Emote` plays once
//!   (`numLoops="1"`, easeIn/easeOut 0.4), `Emote -> Fidgets` hands back.
//! * `ChatLog:enableEmotesButton` is called with 0 while a mount data row exists
//!   and with 1 once the mount is gone.
//!
//! RECONSTRUCTED, labelled at the call site: the request-validation policy in
//! [`validate_emote`] mirrors the recovered client button/motion data on the
//! server side. It is a policy this restoration owns, not a first-party server
//! rule.
//!
//! GAP: animation id 3200, played by the `Dance` phrase (quick-chat id 184,
//! display locale 60), has no `AnimationSlot` row in `AnimationTypes.xml` and no
//! `CodeAnimationMappings.txt` row. It is reported as unrecovered and rejected
//! instead of inventing a motion.

/// Menu root row id of the Emotes panel (`BaseClient.QuickChatData` row 13).
pub const EMOTE_MENU_ROOT: u32 = 13;
/// Parent value of rows that sit outside the Emotes menu (all bits set).
pub const TOP_LEVEL_PARENT: u32 = u32::MAX;

/// Menu branch a quick-chat row belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmoteCategory {
    Basics,
    Dramatics,
    Instructions,
    Style,
    SweetMoves,
    Theatrics,
    /// Not inside the Emotes menu (`wave` is the only animated such row).
    TopLevel,
}

impl EmoteCategory {
    /// Locale id of the category label shown by the client; 0 for `TopLevel`.
    pub fn locale_id(self) -> u32 {
        match self {
            EmoteCategory::Basics => 36636,
            EmoteCategory::Dramatics => 36637,
            EmoteCategory::Instructions => 36638,
            EmoteCategory::Style => 36619,
            EmoteCategory::SweetMoves => 36614,
            EmoteCategory::Theatrics => 36612,
            EmoteCategory::TopLevel => 0,
        }
    }

    /// Parent row id these category entries hang off.
    pub fn menu_parent(self) -> u32 {
        match self {
            EmoteCategory::Basics => 129,
            EmoteCategory::Dramatics => 130,
            EmoteCategory::Instructions => 131,
            EmoteCategory::Style => 132,
            EmoteCategory::SweetMoves => 133,
            EmoteCategory::Theatrics => 134,
            EmoteCategory::TopLevel => TOP_LEVEL_PARENT,
        }
    }
}

/// The six recovered Emotes-menu categories and their locale ids.
pub const EMOTE_CATEGORIES: [(EmoteCategory, u32); 6] = [
    (EmoteCategory::Basics, 36636),
    (EmoteCategory::Dramatics, 36637),
    (EmoteCategory::Instructions, 36638),
    (EmoteCategory::Style, 36619),
    (EmoteCategory::SweetMoves, 36614),
    (EmoteCategory::Theatrics, 36612),
];

/// One recovered `BaseClient.QuickChatData` row that carries an animation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EmoteActionRow {
    pub quick_chat_id: u32,
    pub animation_id: u32,
    pub display_locale: u32,
    /// Say-string locale id; 0 means the row only plays the animation.
    pub say_locale: u32,
    pub category: EmoteCategory,
}

impl EmoteActionRow {
    /// Recovered menu parent id (the row's column-5 value).
    pub fn parent_id(&self) -> u32 {
        self.category.menu_parent()
    }
}

/// `EmoteHandler.emotes` in client array order: command name and quick-chat id.
pub const EMOTE_COMMANDS: [(&str, u32); 72] = [
    ("afraid", 2462),
    ("airguitar", 183),
    ("angry", 2463),
    ("beg", 171),
    ("boo", 219),
    ("bow", 2465),
    ("call", 155),
    ("callpet", 156),
    ("charge", 2467),
    ("cheer", 145),
    ("clap", 2468),
    ("clickheel", 2473),
    ("confused", 146),
    ("congrats", 135),
    ("cry", 147),
    ("curious", 2469),
    ("curious", 148),
    ("dance", 184),
    ("flex", 169),
    ("flip", 2471),
    ("flirt", 172),
    ("frustrated", 149),
    ("giggle", 150),
    ("give", 158),
    ("goodbye", 2472),
    ("hey", 2474),
    ("hi", 2474),
    ("insult", 136),
    ("late", 2476),
    ("laugh", 137),
    ("no", 138),
    ("omg", 151),
    ("outfitall", 161),
    ("outfitchest", 162),
    ("outfitfeet", 163),
    ("outfithands", 164),
    ("outfithead", 165),
    ("outfitlegs", 166),
    ("outfitshoulders", 167),
    ("playtrick", 159),
    ("please", 2477),
    ("point", 139),
    ("ponder", 152),
    ("pose", 173),
    ("prank", 157),
    ("pray", 174),
    ("questing", 2478),
    ("reassure", 2479),
    ("receive", 160),
    ("rofl", 175),
    ("sad", 153),
    ("salute", 2480),
    ("scare", 219),
    ("shoo", 2481),
    ("shrug", 176),
    ("shy", 154),
    ("sleep", 140),
    ("sorry", 2482),
    ("startled", 2483),
    ("stink", 2485),
    ("talk", 141),
    ("tap", 2484),
    ("taunt", 177),
    ("thanks", 142),
    ("twirl", 170),
    ("twist", 168),
    ("wait", 2486),
    ("wave", 143),
    ("whatsup", 2487),
    ("wink", 2488),
    ("yawn", 178),
    ("yes", 144),
];

/// Quick-chat ids bound before the winning row for a repeated command. The
/// client binds `curious` to 2469 and then to 148; 2469 has no data row.
pub const DEAD_EMOTE_BINDINGS: [(u32, &str); 1] = [(2469, "curious")];

/// The 73 recovered quick-chat rows that play an animation, in payload order.
pub const EMOTE_ACTIONS: [EmoteActionRow; 73] = [
    EmoteActionRow {
        quick_chat_id: 135,
        animation_id: 3308,
        display_locale: 23480,
        say_locale: 0,
        category: EmoteCategory::Basics,
    },
    EmoteActionRow {
        quick_chat_id: 136,
        animation_id: 3315,
        display_locale: 23479,
        say_locale: 0,
        category: EmoteCategory::Basics,
    },
    EmoteActionRow {
        quick_chat_id: 137,
        animation_id: 3316,
        display_locale: 23489,
        say_locale: 0,
        category: EmoteCategory::Basics,
    },
    EmoteActionRow {
        quick_chat_id: 138,
        animation_id: 3317,
        display_locale: 8072,
        say_locale: 0,
        category: EmoteCategory::Basics,
    },
    EmoteActionRow {
        quick_chat_id: 139,
        animation_id: 3319,
        display_locale: 36714,
        say_locale: 0,
        category: EmoteCategory::Basics,
    },
    EmoteActionRow {
        quick_chat_id: 140,
        animation_id: 3329,
        display_locale: 23477,
        say_locale: 0,
        category: EmoteCategory::Basics,
    },
    EmoteActionRow {
        quick_chat_id: 141,
        animation_id: 3101,
        display_locale: 3300,
        say_locale: 0,
        category: EmoteCategory::Basics,
    },
    EmoteActionRow {
        quick_chat_id: 142,
        animation_id: 3331,
        display_locale: 23476,
        say_locale: 0,
        category: EmoteCategory::Basics,
    },
    EmoteActionRow {
        quick_chat_id: 143,
        animation_id: 3354,
        display_locale: 23475,
        say_locale: 0,
        category: EmoteCategory::TopLevel,
    },
    EmoteActionRow {
        quick_chat_id: 144,
        animation_id: 3335,
        display_locale: 8073,
        say_locale: 0,
        category: EmoteCategory::Basics,
    },
    EmoteActionRow {
        quick_chat_id: 145,
        animation_id: 3306,
        display_locale: 23473,
        say_locale: 0,
        category: EmoteCategory::Dramatics,
    },
    EmoteActionRow {
        quick_chat_id: 146,
        animation_id: 3307,
        display_locale: 7934,
        say_locale: 0,
        category: EmoteCategory::Dramatics,
    },
    EmoteActionRow {
        quick_chat_id: 147,
        animation_id: 3309,
        display_locale: 23471,
        say_locale: 0,
        category: EmoteCategory::Dramatics,
    },
    EmoteActionRow {
        quick_chat_id: 148,
        animation_id: 3310,
        display_locale: 23470,
        say_locale: 0,
        category: EmoteCategory::Dramatics,
    },
    EmoteActionRow {
        quick_chat_id: 149,
        animation_id: 3336,
        display_locale: 23469,
        say_locale: 0,
        category: EmoteCategory::Dramatics,
    },
    EmoteActionRow {
        quick_chat_id: 150,
        animation_id: 3313,
        display_locale: 23490,
        say_locale: 0,
        category: EmoteCategory::Dramatics,
    },
    EmoteActionRow {
        quick_chat_id: 151,
        animation_id: 3318,
        display_locale: 23493,
        say_locale: 0,
        category: EmoteCategory::Dramatics,
    },
    EmoteActionRow {
        quick_chat_id: 152,
        animation_id: 3320,
        display_locale: 23468,
        say_locale: 0,
        category: EmoteCategory::Dramatics,
    },
    EmoteActionRow {
        quick_chat_id: 153,
        animation_id: 3325,
        display_locale: 8082,
        say_locale: 0,
        category: EmoteCategory::Dramatics,
    },
    EmoteActionRow {
        quick_chat_id: 154,
        animation_id: 3327,
        display_locale: 23484,
        say_locale: 0,
        category: EmoteCategory::Dramatics,
    },
    EmoteActionRow {
        quick_chat_id: 155,
        animation_id: 3304,
        display_locale: 36716,
        say_locale: 0,
        category: EmoteCategory::Instructions,
    },
    EmoteActionRow {
        quick_chat_id: 156,
        animation_id: 3305,
        display_locale: 23465,
        say_locale: 0,
        category: EmoteCategory::Instructions,
    },
    EmoteActionRow {
        quick_chat_id: 157,
        animation_id: 3337,
        display_locale: 23466,
        say_locale: 0,
        category: EmoteCategory::Instructions,
    },
    EmoteActionRow {
        quick_chat_id: 158,
        animation_id: 3314,
        display_locale: 36717,
        say_locale: 0,
        category: EmoteCategory::Instructions,
    },
    EmoteActionRow {
        quick_chat_id: 159,
        animation_id: 3338,
        display_locale: 23464,
        say_locale: 0,
        category: EmoteCategory::Instructions,
    },
    EmoteActionRow {
        quick_chat_id: 160,
        animation_id: 3323,
        display_locale: 36718,
        say_locale: 0,
        category: EmoteCategory::Instructions,
    },
    EmoteActionRow {
        quick_chat_id: 161,
        animation_id: 3001,
        display_locale: 23463,
        say_locale: 0,
        category: EmoteCategory::Style,
    },
    EmoteActionRow {
        quick_chat_id: 162,
        animation_id: 3004,
        display_locale: 23462,
        say_locale: 0,
        category: EmoteCategory::Style,
    },
    EmoteActionRow {
        quick_chat_id: 163,
        animation_id: 3007,
        display_locale: 23461,
        say_locale: 0,
        category: EmoteCategory::Style,
    },
    EmoteActionRow {
        quick_chat_id: 164,
        animation_id: 3005,
        display_locale: 23460,
        say_locale: 0,
        category: EmoteCategory::Style,
    },
    EmoteActionRow {
        quick_chat_id: 165,
        animation_id: 3002,
        display_locale: 23459,
        say_locale: 0,
        category: EmoteCategory::Style,
    },
    EmoteActionRow {
        quick_chat_id: 166,
        animation_id: 3006,
        display_locale: 23458,
        say_locale: 0,
        category: EmoteCategory::Style,
    },
    EmoteActionRow {
        quick_chat_id: 167,
        animation_id: 3003,
        display_locale: 23457,
        say_locale: 0,
        category: EmoteCategory::Style,
    },
    EmoteActionRow {
        quick_chat_id: 168,
        animation_id: 3201,
        display_locale: 23456,
        say_locale: 0,
        category: EmoteCategory::SweetMoves,
    },
    EmoteActionRow {
        quick_chat_id: 169,
        animation_id: 3311,
        display_locale: 23481,
        say_locale: 0,
        category: EmoteCategory::SweetMoves,
    },
    EmoteActionRow {
        quick_chat_id: 170,
        animation_id: 3332,
        display_locale: 23485,
        say_locale: 0,
        category: EmoteCategory::SweetMoves,
    },
    EmoteActionRow {
        quick_chat_id: 171,
        animation_id: 3303,
        display_locale: 6644,
        say_locale: 0,
        category: EmoteCategory::Theatrics,
    },
    EmoteActionRow {
        quick_chat_id: 172,
        animation_id: 3312,
        display_locale: 23482,
        say_locale: 0,
        category: EmoteCategory::Theatrics,
    },
    EmoteActionRow {
        quick_chat_id: 173,
        animation_id: 3321,
        display_locale: 23483,
        say_locale: 0,
        category: EmoteCategory::Theatrics,
    },
    EmoteActionRow {
        quick_chat_id: 174,
        animation_id: 3322,
        display_locale: 23486,
        say_locale: 0,
        category: EmoteCategory::Theatrics,
    },
    EmoteActionRow {
        quick_chat_id: 175,
        animation_id: 3324,
        display_locale: 7617,
        say_locale: 0,
        category: EmoteCategory::Theatrics,
    },
    EmoteActionRow {
        quick_chat_id: 176,
        animation_id: 3326,
        display_locale: 23455,
        say_locale: 0,
        category: EmoteCategory::Theatrics,
    },
    EmoteActionRow {
        quick_chat_id: 177,
        animation_id: 3330,
        display_locale: 23454,
        say_locale: 0,
        category: EmoteCategory::Theatrics,
    },
    EmoteActionRow {
        quick_chat_id: 178,
        animation_id: 3334,
        display_locale: 77707,
        say_locale: 0,
        category: EmoteCategory::Theatrics,
    },
    EmoteActionRow {
        quick_chat_id: 179,
        animation_id: 3211,
        display_locale: 139664,
        say_locale: 0,
        category: EmoteCategory::SweetMoves,
    },
    EmoteActionRow {
        quick_chat_id: 180,
        animation_id: 3212,
        display_locale: 139665,
        say_locale: 0,
        category: EmoteCategory::SweetMoves,
    },
    EmoteActionRow {
        quick_chat_id: 181,
        animation_id: 3213,
        display_locale: 139666,
        say_locale: 0,
        category: EmoteCategory::SweetMoves,
    },
    EmoteActionRow {
        quick_chat_id: 182,
        animation_id: 3214,
        display_locale: 139667,
        say_locale: 0,
        category: EmoteCategory::SweetMoves,
    },
    EmoteActionRow {
        quick_chat_id: 183,
        animation_id: 3350,
        display_locale: 139668,
        say_locale: 0,
        category: EmoteCategory::SweetMoves,
    },
    EmoteActionRow {
        quick_chat_id: 184,
        animation_id: 3200,
        display_locale: 60,
        say_locale: 0,
        category: EmoteCategory::SweetMoves,
    },
    EmoteActionRow {
        quick_chat_id: 219,
        animation_id: 3369,
        display_locale: 5719,
        say_locale: 0,
        category: EmoteCategory::Basics,
    },
    EmoteActionRow {
        quick_chat_id: 2462,
        animation_id: 3339,
        display_locale: 7928,
        say_locale: 0,
        category: EmoteCategory::Dramatics,
    },
    EmoteActionRow {
        quick_chat_id: 2463,
        animation_id: 3301,
        display_locale: 428871,
        say_locale: 0,
        category: EmoteCategory::Dramatics,
    },
    EmoteActionRow {
        quick_chat_id: 2465,
        animation_id: 3342,
        display_locale: 420181,
        say_locale: 0,
        category: EmoteCategory::Theatrics,
    },
    EmoteActionRow {
        quick_chat_id: 2467,
        animation_id: 3343,
        display_locale: 36324,
        say_locale: 0,
        category: EmoteCategory::Theatrics,
    },
    EmoteActionRow {
        quick_chat_id: 2468,
        animation_id: 3302,
        display_locale: 428876,
        say_locale: 0,
        category: EmoteCategory::Basics,
    },
    EmoteActionRow {
        quick_chat_id: 2471,
        animation_id: 3367,
        display_locale: 411985,
        say_locale: 0,
        category: EmoteCategory::Theatrics,
    },
    EmoteActionRow {
        quick_chat_id: 2472,
        animation_id: 3360,
        display_locale: 428880,
        say_locale: 422215,
        category: EmoteCategory::Basics,
    },
    EmoteActionRow {
        quick_chat_id: 2473,
        animation_id: 3366,
        display_locale: 428881,
        say_locale: 0,
        category: EmoteCategory::Theatrics,
    },
    EmoteActionRow {
        quick_chat_id: 2474,
        animation_id: 3354,
        display_locale: 428882,
        say_locale: 421660,
        category: EmoteCategory::Basics,
    },
    EmoteActionRow {
        quick_chat_id: 2476,
        animation_id: 3362,
        display_locale: 428884,
        say_locale: 428898,
        category: EmoteCategory::Instructions,
    },
    EmoteActionRow {
        quick_chat_id: 2477,
        animation_id: 3355,
        display_locale: 428885,
        say_locale: 140442,
        category: EmoteCategory::Instructions,
    },
    EmoteActionRow {
        quick_chat_id: 2478,
        animation_id: 3353,
        display_locale: 428886,
        say_locale: 428897,
        category: EmoteCategory::Instructions,
    },
    EmoteActionRow {
        quick_chat_id: 2479,
        animation_id: 3358,
        display_locale: 428887,
        say_locale: 421883,
        category: EmoteCategory::Instructions,
    },
    EmoteActionRow {
        quick_chat_id: 2480,
        animation_id: 3344,
        display_locale: 428888,
        say_locale: 0,
        category: EmoteCategory::Instructions,
    },
    EmoteActionRow {
        quick_chat_id: 2481,
        animation_id: 3345,
        display_locale: 428889,
        say_locale: 0,
        category: EmoteCategory::Instructions,
    },
    EmoteActionRow {
        quick_chat_id: 2482,
        animation_id: 3357,
        display_locale: 428890,
        say_locale: 422852,
        category: EmoteCategory::Basics,
    },
    EmoteActionRow {
        quick_chat_id: 2483,
        animation_id: 3368,
        display_locale: 428891,
        say_locale: 0,
        category: EmoteCategory::Theatrics,
    },
    EmoteActionRow {
        quick_chat_id: 2484,
        animation_id: 3346,
        display_locale: 428892,
        say_locale: 0,
        category: EmoteCategory::Theatrics,
    },
    EmoteActionRow {
        quick_chat_id: 2485,
        animation_id: 3359,
        display_locale: 428893,
        say_locale: 8153,
        category: EmoteCategory::Theatrics,
    },
    EmoteActionRow {
        quick_chat_id: 2486,
        animation_id: 3347,
        display_locale: 8079,
        say_locale: 422435,
        category: EmoteCategory::Instructions,
    },
    EmoteActionRow {
        quick_chat_id: 2487,
        animation_id: 3364,
        display_locale: 428895,
        say_locale: 422525,
        category: EmoteCategory::Basics,
    },
    EmoteActionRow {
        quick_chat_id: 2488,
        animation_id: 3348,
        display_locale: 428896,
        say_locale: 0,
        category: EmoteCategory::Theatrics,
    },
];

/// `AnimationTypes.xml` slot names for the animation ids the quick-chat table
/// references. Id 3200 has no slot row and is listed in
/// [`UNRECOVERED_ANIMATION_IDS`].
pub const EMOTE_ANIMATIONS: [(u32, &str); 71] = [
    (3001, "emo_outfit_all"),
    (3002, "emo_outfit_head"),
    (3003, "emo_outfit_shoulders"),
    (3004, "emo_outfit_chest"),
    (3005, "emo_outfit_hands"),
    (3006, "emo_outfit_legs"),
    (3007, "emo_outfit_feet"),
    (3101, "emo_talk_basic"),
    (3201, "emo_dance_basic"),
    (3211, "emo_dance_01"),
    (3212, "emo_dance_02"),
    (3213, "emo_dance_03"),
    (3214, "emo_dance_04"),
    (3301, "emo_angry"),
    (3302, "emo_applaud"),
    (3303, "emo_beg"),
    (3304, "emo_call_person"),
    (3305, "emo_call_pet"),
    (3306, "emo_cheer"),
    (3307, "emo_confused"),
    (3308, "emo_congrats"),
    (3309, "emo_cry"),
    (3310, "emo_curious"),
    (3311, "emo_flex"),
    (3312, "emo_flirt"),
    (3313, "emo_giggle"),
    (3314, "emo_give"),
    (3315, "emo_insult"),
    (3316, "emo_laugh"),
    (3317, "emo_no"),
    (3318, "emo_omg"),
    (3319, "emo_point"),
    (3320, "emo_ponder"),
    (3321, "emo_posing"),
    (3322, "emo_pray"),
    (3323, "emo_receive"),
    (3324, "emo_rofl"),
    (3325, "emo_sad"),
    (3326, "emo_shrug"),
    (3327, "emo_shy"),
    (3329, "emo_sleep_standing"),
    (3330, "emo_taunt"),
    (3331, "emo_thanks"),
    (3332, "emo_twirl"),
    (3334, "emo_yawn"),
    (3335, "emo_yes"),
    (3336, "emo_frustrated"),
    (3337, "emo_trick_bad"),
    (3338, "emo_trick_good"),
    (3339, "emo_afraid"),
    (3342, "emo_bow"),
    (3343, "emo_charge"),
    (3344, "emo_salute"),
    (3345, "emo_shoo"),
    (3346, "emo_tap"),
    (3347, "emo_wait"),
    (3348, "emo_wink"),
    (3350, "emo_air_guitar"),
    (3353, "emo_going_on_quest"),
    (3354, "emo_hey"),
    (3355, "emo_please"),
    (3357, "emo_sorry"),
    (3358, "emo_dont_worry"),
    (3359, "emo_that_stinks"),
    (3360, "emo_bye_bye"),
    (3362, "emo_peace_out"),
    (3364, "emo_whats_up"),
    (3366, "emo_click_heels"),
    (3367, "emo_flip"),
    (3368, "emo_startled"),
    (3369, "emo_scare"),
];

/// Animation ids referenced by the quick-chat table that have no recovered
/// `AnimationSlot` name. `Dance` (id 184) points at 3200.
pub const UNRECOVERED_ANIMATION_IDS: [u32; 1] = [3200];

/// Slots with `freezeMovement="1"`: the client holds the actor still while the
/// clip plays. `emo_spraycan` (3351) is in the Emotes block but no quick-chat
/// row reaches it yet.
pub const FREEZE_MOVEMENT_ANIMATIONS: [u32; 2] = [3001, 3351];

/// Slots with `disableBoneMasking="1"`.
pub const DISABLE_BONE_MASKING_ANIMATIONS: [u32; 1] = [3329];

/// Slots with `priority="1"` inside the Emotes block.
pub const PRIORITY_ONE_ANIMATIONS: [u32; 7] = [3001, 3101, 3201, 3211, 3212, 3213, 3214];

/// Every named emote slot carries `hideSlotType="1"` (the wielded item is
/// hidden for the clip) except these recovered exceptions.
pub const HIDE_SLOT_TYPE_EXCEPTIONS: [u32; 2] = [3001, 3200];

/// Recovered `Any -> Emote` transition rule.
pub struct EmoteTransition {
    /// `numLoops="1"`: an emote plays once.
    pub num_loops: i32,
    pub ease_in: f32,
    pub ease_out: f32,
}

/// `AnimationTransitionRules.xml`, `Rule fromType="Any" toType="Emote"`.
pub const EMOTE_TRANSITION: EmoteTransition = EmoteTransition {
    num_loops: 1,
    ease_in: 0.4,
    ease_out: 0.4,
};

/// Per-slot restriction flags from the Emotes animation block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EmoteRestrictions {
    pub hide_slot_type: bool,
    pub freeze_movement: bool,
    pub disable_bone_masking: bool,
    pub priority_one: bool,
}

/// Quick-chat id bound to `command`, or `None` when the client has no binding.
///
/// The client binds array entries in order, so the last row for a repeated
/// command wins (`curious` resolves to 148, not to the earlier 2469).
pub fn emote_command_id(command: &str) -> Option<u32> {
    EMOTE_COMMANDS
        .iter()
        .rev()
        .find(|(name, _)| *name == command)
        .map(|(_, id)| *id)
}

/// Recovered quick-chat row for a command id.
pub fn emote_action(quick_chat_id: u32) -> Option<EmoteActionRow> {
    EMOTE_ACTIONS
        .iter()
        .find(|row| row.quick_chat_id == quick_chat_id)
        .copied()
}

/// Recovered animation slot name, or `None` when the id is unknown or belongs
/// to the documented gap ([`UNRECOVERED_ANIMATION_IDS`]).
pub fn emote_animation_name(animation_id: u32) -> Option<&'static str> {
    EMOTE_ANIMATIONS
        .iter()
        .find(|(id, _)| *id == animation_id)
        .map(|(_, name)| *name)
}

/// Restriction flags for a recovered emote animation slot.
pub fn emote_restrictions(animation_id: u32) -> EmoteRestrictions {
    EmoteRestrictions {
        hide_slot_type: emote_animation_name(animation_id).is_some()
            && !HIDE_SLOT_TYPE_EXCEPTIONS.contains(&animation_id),
        freeze_movement: FREEZE_MOVEMENT_ANIMATIONS.contains(&animation_id),
        disable_bone_masking: DISABLE_BONE_MASKING_ANIMATIONS.contains(&animation_id),
        priority_one: PRIORITY_ONE_ANIMATIONS.contains(&animation_id),
    }
}

/// Recovered `ChatLog:CreateQuickChatPhrase` filter for one quick-chat row.
///
/// Emote mode creates every row whose parent is not 0 (rows parented to the
/// Emotes root are re-parented to 0); canned-chat mode drops the root row and
/// its category rows.
pub fn emote_visible_in_menu(id: u32, parent_id: u32, emote_mode: bool) -> bool {
    if emote_mode {
        parent_id != 0
    } else {
        parent_id != EMOTE_MENU_ROOT && id != EMOTE_MENU_ROOT
    }
}

/// Actor state the reconstructed validation policy looks at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EmoteContext {
    /// A mount data row exists; the client disables the emotes button.
    pub mounted: bool,
    /// The actor is moving, which the recovered `freezeMovement` slots cannot
    /// start from.
    pub moving: bool,
}

impl EmoteContext {
    /// Stationary, unmounted actor.
    pub fn standing() -> Self {
        EmoteContext {
            mounted: false,
            moving: false,
        }
    }
}

/// Why an emote request is refused. RECONSTRUCTED policy; see [`validate_emote`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmoteRejection {
    /// No `EmoteHandler` binding for the command.
    UnknownCommand,
    /// The row points at an animation id with no recovered slot (id 3200).
    UnrecoveredAnimation { animation_id: u32 },
    /// The animation id is outside the recovered `Emotes` slot block.
    ForeignAnimation { animation_id: u32 },
    /// The quick-chat row's animation differs from the recovered row for that id.
    AnimationMismatch { animation_id: u32 },
    /// Blocked while mounted, mirroring the recovered button state.
    Mounted,
    /// The slot freezes movement and the actor is moving.
    Moving { animation_id: u32 },
}

/// `AnimationTypes.xml` id block of the `Emotes` branch (3001-3900).
pub const EMOTE_ANIMATION_BLOCK: std::ops::RangeInclusive<u32> = 3001..=3900;

/// Resolve an emote command and apply the recovered restrictions.
pub fn validate_emote(
    command: &str,
    context: EmoteContext,
) -> Result<EmoteActionRow, EmoteRejection> {
    let Some(quick_chat_id) = emote_command_id(command) else {
        return Err(EmoteRejection::UnknownCommand);
    };
    let Some(action) = emote_action(quick_chat_id) else {
        // Only reachable for a command whose winning binding has no data row.
        return Err(EmoteRejection::UnknownCommand);
    };
    validate_emote_phrase(action.quick_chat_id, action.animation_id, context).map(|()| action)
}

/// Apply the recovered restrictions to one quick-chat row that carries
/// `animation_id`. RECONSTRUCTED policy (R): the server mirrors the client's
/// mount gating and `freezeMovement` data; the checks are ordered data first
/// (the row can never play), then actor state.
pub fn validate_emote_phrase(
    quick_chat_id: u32,
    animation_id: u32,
    context: EmoteContext,
) -> Result<(), EmoteRejection> {
    if UNRECOVERED_ANIMATION_IDS.contains(&animation_id) {
        return Err(EmoteRejection::UnrecoveredAnimation { animation_id });
    }
    if !EMOTE_ANIMATION_BLOCK.contains(&animation_id) {
        return Err(EmoteRejection::ForeignAnimation { animation_id });
    }
    if emote_action(quick_chat_id).is_some_and(|row| row.animation_id != animation_id) {
        return Err(EmoteRejection::AnimationMismatch { animation_id });
    }
    if context.mounted {
        return Err(EmoteRejection::Mounted);
    }
    if emote_restrictions(animation_id).freeze_movement && context.moving {
        return Err(EmoteRejection::Moving { animation_id });
    }
    Ok(())
}
