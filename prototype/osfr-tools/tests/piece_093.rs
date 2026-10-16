//! Piece 093 (emotes and social actions) tests: the recovered emote command
//! table, the client quick-chat/animation rows, the animation-slot names and
//! the restriction flags the client data actually carries.
//!
//! Evidence boundary, read before the assertions:
//!
//! ORIGINAL (first-party inputs; hashes pinned in the lab bundle):
//! * `decompiled/client-ui/ScriptsBase.lua`, `EmoteHandler.emotes`: 72
//!   `{cmd = "...", id = N}` rows bound with `BindCommandToLua` in array order,
//!   so a repeated command is overwritten by the last row (`curious` = 2469
//!   then 148).
//! * `decompiled/server/Gateway/Gateway.MapChat/ChatSystem.cs`
//!   `SendQuickChatPacket`: the tunneled `BaseClient.QuickChatData` payload
//!   (opcode 67, subtype 1, 2470 rows of 10 little-endian u32 columns). The
//!   client Lua reads display id (column 2), say id (column 3) and animation id
//!   (column 4); the parent column (5) forms the menu tree rooted at row 13.
//! * `client/Resources/AnimationTypes.xml`: the `Emotes` slot block, including
//!   `emo_no` = 3317, `emo_dance_basic` = 3201 and `hideSlotType` /
//!   `freezeMovement` / `disableBoneMasking` / `priority` attributes.
//! * `client/Resources/AnimationTransitionRules.xml`: `Any -> Emote` plays once
//!   (`numLoops="1"`, easeIn/easeOut 0.4) and `Emote -> Fidgets` returns.
//! * Locale export text for the menu and phrase strings.
//!
//! RECONSTRUCTED, labelled at each call site: the request-validation policy
//! (mounted/moving rejection) built on the recovered button/motion data.
//!
//! NOT CLAIMED: animation id 3200 (the `Dance` phrase) has no `AnimationSlot`
//! row, so it stays an explicit gap instead of a guessed motion; no native
//! client run is claimed here.
// The module is not registered in `src/lib.rs` (lead-owned). Including the exact
// source file keeps this target compiling against the real artifact. Integration
// step for the lead: register the module and drop this `#[path]` include.
#[path = "../src/restoration/piece_093.rs"]
#[allow(dead_code)]
mod piece_093;

use piece_093::{
    DEAD_EMOTE_BINDINGS, DISABLE_BONE_MASKING_ANIMATIONS, EMOTE_ACTIONS, EMOTE_ANIMATIONS,
    EMOTE_CATEGORIES, EMOTE_COMMANDS, EMOTE_MENU_ROOT, EMOTE_TRANSITION, EmoteCategory,
    EmoteContext, EmoteRejection, FREEZE_MOVEMENT_ANIMATIONS, HIDE_SLOT_TYPE_EXCEPTIONS,
    PRIORITY_ONE_ANIMATIONS, TOP_LEVEL_PARENT, UNRECOVERED_ANIMATION_IDS, emote_action,
    emote_animation_name, emote_command_id, emote_restrictions, emote_visible_in_menu,
    validate_emote,
};

#[test]
fn recovered_command_table_matches_the_client_lua() {
    assert_eq!(EMOTE_COMMANDS.len(), 72);
    assert_eq!(emote_command_id("afraid"), Some(2462));
    assert_eq!(emote_command_id("dance"), Some(184));
    assert_eq!(emote_command_id("wave"), Some(143));
    assert_eq!(emote_command_id("whatsup"), Some(2487));
    assert_eq!(emote_command_id("NO_SUCH_COMMAND"), None);
    // `hi` and `hey` are separate commands sharing one quick-chat id.
    assert_eq!(emote_command_id("hi"), Some(2474));
    assert_eq!(emote_command_id("hey"), Some(2474));
    // `boo` and `scare` likewise share 219.
    assert_eq!(emote_command_id("boo"), Some(219));
    assert_eq!(emote_command_id("scare"), Some(219));
}

#[test]
fn repeated_command_binding_resolves_last_and_the_old_id_is_a_dead_binding() {
    // BindCommandToLua walks the array in order, so the second `curious` row
    // wins. The earlier id 2469 has no QuickChatData row and stays recorded.
    assert_eq!(emote_command_id("curious"), Some(148));
    assert_eq!(DEAD_EMOTE_BINDINGS, [(2469, "curious")]);
    assert!(emote_action(2469).is_none());
}

#[test]
fn quick_chat_rows_carry_the_recovered_animation_ids() {
    assert_eq!(EMOTE_ACTIONS.len(), 73);
    assert_eq!(emote_action(138).unwrap().animation_id, 3317);
    assert_eq!(emote_action(138).unwrap().display_locale, 8072);
    assert_eq!(emote_action(168).unwrap().animation_id, 3201);
    assert_eq!(emote_action(175).unwrap().animation_id, 3324);
    assert_eq!(emote_action(175).unwrap().display_locale, 7617);
    // Social rows that also say a phrase.
    let goodbye = emote_action(2472).unwrap();
    assert_eq!((goodbye.animation_id, goodbye.say_locale), (3360, 422215));
    let wait = emote_action(2486).unwrap();
    assert_eq!((wait.animation_id, wait.say_locale), (3347, 422435));
    // The `wave` phrase is a top-level row, not an emote-menu child.
    assert_eq!(emote_action(143).unwrap().category, EmoteCategory::TopLevel);
    assert!(emote_action(9999).is_none());
}

#[test]
fn emote_menu_categories_match_the_recovered_locale_ids() {
    assert_eq!(EMOTE_MENU_ROOT, 13);
    assert_eq!(EMOTE_CATEGORIES.len(), 6);
    let pairs: Vec<(EmoteCategory, u32)> = EMOTE_CATEGORIES.to_vec();
    assert!(pairs.contains(&(EmoteCategory::Basics, 36636)));
    assert!(pairs.contains(&(EmoteCategory::Dramatics, 36637)));
    assert!(pairs.contains(&(EmoteCategory::Instructions, 36638)));
    assert!(pairs.contains(&(EmoteCategory::Style, 36619)));
    assert!(pairs.contains(&(EmoteCategory::SweetMoves, 36614)));
    assert!(pairs.contains(&(EmoteCategory::Theatrics, 36612)));
    assert_eq!(EmoteCategory::Basics.locale_id(), 36636);
    assert_eq!(EmoteCategory::TopLevel.locale_id(), 0);
    assert_eq!(
        emote_action(145).unwrap().category,
        EmoteCategory::Dramatics
    );
    assert_eq!(
        emote_action(155).unwrap().category,
        EmoteCategory::Instructions
    );
    assert_eq!(emote_action(161).unwrap().category, EmoteCategory::Style);
}

#[test]
fn animation_names_cover_every_referenced_id_except_the_recorded_gap() {
    // 73 actions reference 72 distinct animation ids; 3200 (the `Dance` phrase)
    // has no AnimationSlot row, so 71 ids carry a recovered slot name.
    assert_eq!(EMOTE_ANIMATIONS.len(), 71);
    assert_eq!(emote_animation_name(3301), Some("emo_angry"));
    assert_eq!(emote_animation_name(3317), Some("emo_no"));
    assert_eq!(emote_animation_name(3201), Some("emo_dance_basic"));
    assert_eq!(emote_animation_name(3369), Some("emo_scare"));
    assert_eq!(emote_animation_name(3001), Some("emo_outfit_all"));
    assert_eq!(emote_animation_name(3101), Some("emo_talk_basic"));
    // The `Dance` phrase points at 3200, which has no AnimationSlot row.
    assert_eq!(UNRECOVERED_ANIMATION_IDS, [3200]);
    assert_eq!(emote_action(184).unwrap().animation_id, 3200);
    assert_eq!(emote_animation_name(3200), None);
    // Every action resolves to a named slot or to the recorded gap; nothing else.
    for action in EMOTE_ACTIONS {
        assert!(
            emote_animation_name(action.animation_id).is_some()
                || UNRECOVERED_ANIMATION_IDS.contains(&action.animation_id),
            "action {} points at unexplained animation {}",
            action.quick_chat_id,
            action.animation_id
        );
    }
    // No duplicate names and no duplicate ids.
    let mut names: Vec<&str> = EMOTE_ANIMATIONS.iter().map(|(_, name)| *name).collect();
    names.sort_unstable();
    let unique = names.len();
    names.dedup();
    assert_eq!(names.len(), unique);
}

#[test]
fn restriction_flags_come_from_the_animation_block() {
    // `freezeMovement="1"` on emo_outfit_all (3001) and emo_spraycan (3351);
    // only the first is reachable from a quick-chat row.
    assert_eq!(FREEZE_MOVEMENT_ANIMATIONS, [3001, 3351]);
    assert!(emote_restrictions(3001).freeze_movement);
    assert!(!emote_restrictions(3333).freeze_movement);
    // Every emote slot hides the wielded item except the recorded exceptions.
    assert_eq!(HIDE_SLOT_TYPE_EXCEPTIONS, [3001, 3200]);
    assert!(emote_restrictions(3317).hide_slot_type);
    assert!(!emote_restrictions(3001).hide_slot_type);
    assert!(!emote_restrictions(3200).hide_slot_type);
    assert_eq!(DISABLE_BONE_MASKING_ANIMATIONS, [3329]);
    assert!(emote_restrictions(3329).disable_bone_masking);
    assert_eq!(
        PRIORITY_ONE_ANIMATIONS,
        [3001, 3101, 3201, 3211, 3212, 3213, 3214]
    );
    assert!(emote_restrictions(3201).priority_one);
    assert!(!emote_restrictions(3317).priority_one);
    // `Any -> Emote` runs the clip once and blends 0.4s in/out.
    assert_eq!(EMOTE_TRANSITION.num_loops, 1);
    assert_eq!(EMOTE_TRANSITION.ease_in, 0.4);
    assert_eq!(EMOTE_TRANSITION.ease_out, 0.4);
}

#[test]
fn menu_gating_matches_the_client_lua_filters() {
    // `ChatLog:CreateQuickChatPhrase` recovered filters:
    //   emote mode:  create when parentId ~= 0 (parentId == 13 re-parents to 0)
    //   canned mode: create when parentId ~= 13 and id ~= 13
    assert_eq!(TOP_LEVEL_PARENT, u32::MAX);
    assert_eq!(emote_action(143).unwrap().parent_id(), TOP_LEVEL_PARENT);
    assert_eq!(
        emote_action(145).unwrap().parent_id(),
        EmoteCategory::Dramatics.menu_parent()
    );
    // The Emotes root row and the six category rows show only in emote mode.
    assert!(emote_visible_in_menu(
        EMOTE_MENU_ROOT,
        EMOTE_MENU_ROOT,
        true
    ));
    assert!(!emote_visible_in_menu(
        EMOTE_MENU_ROOT,
        EMOTE_MENU_ROOT,
        false
    ));
    assert!(emote_visible_in_menu(130, EMOTE_MENU_ROOT, true));
    assert!(!emote_visible_in_menu(130, EMOTE_MENU_ROOT, false));
    // Ordinary top-level phrases (parent 0) show only in canned-chat mode.
    assert!(!emote_visible_in_menu(1, 0, true));
    assert!(emote_visible_in_menu(1, 0, false));
    // Category children are created in both modes.
    assert!(emote_visible_in_menu(145, 130, true));
    assert!(emote_visible_in_menu(145, 130, false));
    // Top-level rows that carry an emote animation, like `wave`, show in both.
    assert!(emote_visible_in_menu(143, TOP_LEVEL_PARENT, true));
    assert!(emote_visible_in_menu(143, TOP_LEVEL_PARENT, false));
}

#[test]
fn validation_accepts_recovered_emotes_and_rejects_restricted_requests() {
    let standing = EmoteContext::standing();
    let wave = validate_emote("wave", standing).unwrap();
    assert_eq!(wave.animation_id, 3354);
    let cheer = validate_emote("cheer", standing).unwrap();
    assert_eq!(cheer.animation_id, 3306);

    // Mounted requests are blocked: the client disables the emotes button while
    // a mount data row exists. RECONSTRUCTED server-side mirror of that rule.
    let mounted = EmoteContext {
        mounted: true,
        moving: false,
    };
    assert_eq!(
        validate_emote("wave", mounted),
        Err(EmoteRejection::Mounted)
    );

    // Moving is rejected only where the recovered slot freezes movement.
    let moving = EmoteContext {
        mounted: false,
        moving: true,
    };
    assert_eq!(validate_emote("wave", moving).unwrap().animation_id, 3354);
    assert_eq!(
        validate_emote("outfitall", moving),
        Err(EmoteRejection::Moving { animation_id: 3001 })
    );

    assert_eq!(
        validate_emote("dance", standing).unwrap_err(),
        EmoteRejection::UnrecoveredAnimation { animation_id: 3200 }
    );
    assert_eq!(
        validate_emote("nope", standing),
        Err(EmoteRejection::UnknownCommand)
    );
}
