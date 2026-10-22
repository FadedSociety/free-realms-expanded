//! W139 (W134 brief #15): emote acceptance on the quick-chat route.
//!
//! ORIGINAL facts (piece 093, recovered client data): the emote rows, their
//! animation ids, the `freezeMovement` slots and the client's mount gating of
//! the emotes button (`ChatLog:enableEmotesButton`). The native 67/3 packet
//! carries only the phrase id (`00b90a10` writer layout, see
//! `tests/social_chat.rs`), so the animation always comes from the server-held
//! startup row, never from the sender.
//!
//! RECONSTRUCTED (R): the server-side refusal policy (mounted, moving into a
//! `freezeMovement` slot, foreign/mismatched animation) and the 500 ms
//! "moving" window. Dance (3200) is the client's `emo_dance` animation group
//! (`AnimationGroups.xml:874`) and follows the same rules as every emote.
//!
//! Canaries: `canary_*` tests fail if the policy stops distinguishing rows.

use osfr_tools::social_chat::piece_093::{
    EMOTE_ACTIONS, EmoteContext, EmoteRejection, emote_command_id,
};
use osfr_tools::social_chat::{
    ChatChannel, ChatError, ChatSession, EMOTE_MOVING_WINDOW_MS, IgnoreLists, NameData,
    QuickChatCatalog, QuickChatRequest, decode_quick_chat_request, emote_context,
    encode_quick_chat_channel, encode_quick_chat_tell, resolve_quick_chat_recipients,
};

const PLAIN_PHRASE: i32 = 4101;

fn standing() -> EmoteContext {
    EmoteContext::standing()
}
fn mounted() -> EmoteContext {
    EmoteContext {
        mounted: true,
        moving: false,
    }
}
fn moving() -> EmoteContext {
    EmoteContext {
        mounted: false,
        moving: true,
    }
}

fn phrase(command: &str) -> i32 {
    emote_command_id(command).expect("recovered command") as i32
}

/// Every recovered animated row plus one plain phrase, as the startup table
/// would present them.
fn recovered_catalog() -> QuickChatCatalog {
    QuickChatCatalog::from_rows(
        EMOTE_ACTIONS
            .iter()
            .map(|row| (row.quick_chat_id as i32, row.animation_id as i32))
            .chain([(PLAIN_PHRASE, 0)]),
    )
}

fn name(first: &str) -> NameData {
    NameData {
        fields: [0; 3],
        first: first.to_owned(),
        last: String::new(),
    }
}

fn session(id: u64, guid: u64, first: &str, x: f32) -> ChatSession {
    ChatSession {
        session_id: id,
        character_guid: guid,
        name: name(first),
        zone_id: 1,
        position: [x, 0.0, 0.0],
        chat_area_id: Some(7),
        group_id: None,
        guild_id: None,
        is_member: true,
    }
}

#[test]
fn plain_phrases_are_never_restricted() {
    let catalog = recovered_catalog();
    for context in [standing(), mounted(), moving()] {
        assert_eq!(catalog.authorize_emote(PLAIN_PHRASE, context), Ok(()));
    }
}

#[test]
fn every_recovered_emote_is_accepted_standing() {
    let catalog = recovered_catalog();
    let refused: Vec<_> = EMOTE_ACTIONS
        .iter()
        .filter_map(|row| {
            let id = row.quick_chat_id as i32;
            catalog
                .authorize_emote(id, standing())
                .err()
                .map(|e| (id, e))
        })
        .collect();
    assert_eq!(refused, []);
    assert_eq!(phrase("dance"), 184);
    assert_eq!(catalog.authorize_emote(184, standing()), Ok(()));
}

#[test]
fn mounted_actors_cannot_emote_but_can_still_use_plain_chat() {
    let catalog = recovered_catalog();
    for row in EMOTE_ACTIONS.iter() {
        assert_eq!(
            catalog.authorize_emote(row.quick_chat_id as i32, mounted()),
            Err(ChatError::EmoteRefused(EmoteRejection::Mounted)),
            "phrase {}",
            row.quick_chat_id
        );
    }
    assert_eq!(catalog.authorize_emote(PLAIN_PHRASE, mounted()), Ok(()));
}

#[test]
fn moving_blocks_only_the_freeze_movement_slot() {
    let catalog = recovered_catalog();
    let outfit = phrase("outfitall");
    assert_eq!(
        catalog.authorize_emote(outfit, moving()),
        Err(ChatError::EmoteRefused(EmoteRejection::Moving {
            animation_id: 3001
        }))
    );
    assert_eq!(catalog.authorize_emote(outfit, standing()), Ok(()));
    // An ordinary emote plays while walking (the client allows it, 3354 has no freeze).
    assert_eq!(catalog.authorize_emote(phrase("wave"), moving()), Ok(()));
}

#[test]
fn an_unknown_phrase_is_refused_and_supplies_no_animation() {
    let catalog = recovered_catalog();
    for id in [-1, 0, 1, 999_999, i32::MAX, i32::MIN] {
        assert_eq!(
            catalog.authorize_emote(id, standing()),
            Err(ChatError::UnknownQuickChatId(id))
        );
    }
}

#[test]
fn a_catalog_row_cannot_supply_arbitrary_animation_data() {
    let cheer = phrase("cheer");
    let catalog = QuickChatCatalog::from_rows([
        (5, 99_999),   // outside the Emotes slot block
        (6, -3306),    // negative id
        (184, 3899),   // Dance phrase, an id with neither slot nor group
        (cheer, 3317), // recovered phrase, another emote's animation
        (8, 3999),     // just past the block
        (9, 3000),     // just before the block
        (10, 3333),    // in-block id without a recovered row: allowed (R)
    ]);
    let refused = |id| catalog.authorize_emote(id, standing()).unwrap_err();
    assert_eq!(
        refused(5),
        ChatError::EmoteRefused(EmoteRejection::ForeignAnimation {
            animation_id: 99_999
        })
    );
    assert!(matches!(
        refused(6),
        ChatError::EmoteRefused(EmoteRejection::ForeignAnimation { .. })
    ));
    assert_eq!(
        refused(184),
        ChatError::EmoteRefused(EmoteRejection::AnimationMismatch { animation_id: 3899 })
    );
    assert_eq!(
        refused(cheer),
        ChatError::EmoteRefused(EmoteRejection::AnimationMismatch { animation_id: 3317 })
    );
    assert!(matches!(
        refused(8),
        ChatError::EmoteRefused(EmoteRejection::ForeignAnimation { .. })
    ));
    assert!(matches!(
        refused(9),
        ChatError::EmoteRefused(EmoteRejection::ForeignAnimation { .. })
    ));
    assert_eq!(catalog.authorize_emote(10, standing()), Ok(()));
}

#[test]
fn moving_window_is_inferred_from_the_last_accepted_movement() {
    let now = 10_000;
    assert!(!emote_context(false, None, now).moving);
    assert!(emote_context(false, Some(now), now).moving);
    assert!(emote_context(false, Some(now - EMOTE_MOVING_WINDOW_MS + 1), now).moving);
    assert!(!emote_context(false, Some(now - EMOTE_MOVING_WINDOW_MS), now).moving);
    // A last-movement time ahead of now saturates to zero elapsed: still inside the window.
    assert!(emote_context(false, Some(now + 5), now).moving);
    assert!(emote_context(true, None, now).mounted);
}

/// Mirrors the gateway order: decode, authorize, resolve recipients, encode.
fn route(
    catalog: &QuickChatCatalog,
    packet: &[u8],
    context: EmoteContext,
    sessions: &[ChatSession],
    ignores: &IgnoreLists,
) -> Result<(Vec<u64>, Vec<u8>), ChatError> {
    let request = decode_quick_chat_request(packet, catalog)?;
    let phrase_id = match &request {
        QuickChatRequest::Channel { phrase_id, .. } | QuickChatRequest::Tell { phrase_id, .. } => {
            *phrase_id
        }
    };
    catalog.authorize_emote(phrase_id, context)?;
    let recipients = resolve_quick_chat_recipients(&request, 1, sessions, ignores)?;
    let sender = &sessions[0];
    let echo = match &request {
        QuickChatRequest::Channel {
            phrase_id,
            channel,
            guild_guid,
        } => encode_quick_chat_channel(
            catalog,
            *phrase_id,
            *channel,
            sender.character_guid,
            &sender.name,
            *guild_guid,
        )?,
        QuickChatRequest::Tell {
            phrase_id,
            target_name,
        } => encode_quick_chat_tell(
            catalog,
            *phrase_id,
            sender.character_guid,
            &sender.name,
            target_name,
        )?,
    };
    Ok((recipients, echo))
}

#[test]
fn two_participants_see_one_ordinary_emote_and_the_echo_carries_only_the_phrase_id() {
    let catalog = recovered_catalog();
    let sessions = [
        session(1, 101, "fixture-sender", 0.0),
        session(2, 202, "fixture-near", 5.0),
        session(3, 303, "fixture-far", 500.0),
    ];
    let sender = name("fixture-sender");
    let cheer = phrase("cheer");
    let packet =
        encode_quick_chat_channel(&catalog, cheer, ChatChannel::Say, 101, &sender, 0).unwrap();
    let (recipients, echo) = route(
        &catalog,
        &packet,
        standing(),
        &sessions,
        &IgnoreLists::default(),
    )
    .unwrap();
    assert_eq!(recipients, [1, 2]);
    // Same layout as a plain phrase: only the phrase id differs, so the
    // receiving client resolves the animation from its own table.
    let plain =
        encode_quick_chat_channel(&catalog, PLAIN_PHRASE, ChatChannel::Say, 101, &sender, 0)
            .unwrap();
    assert_eq!(echo.len(), plain.len());
    assert_eq!(echo[..4], plain[..4]);
    assert_eq!(echo[8..], plain[8..]);
    assert_eq!(echo[4..8], cheer.to_le_bytes());
}

#[test]
fn normal_quick_chat_and_tells_still_work_while_mounted_or_moving() {
    let catalog = recovered_catalog();
    let sessions = [
        session(1, 101, "fixture-sender", 0.0),
        session(2, 202, "fixture-target", 5.0),
    ];
    let sender = name("fixture-sender");
    let say = encode_quick_chat_channel(&catalog, PLAIN_PHRASE, ChatChannel::Say, 101, &sender, 0)
        .unwrap();
    let tell =
        encode_quick_chat_tell(&catalog, PLAIN_PHRASE, 101, &sender, "fixture-target").unwrap();
    for context in [standing(), mounted(), moving()] {
        let ignores = IgnoreLists::default();
        assert_eq!(
            route(&catalog, &say, context, &sessions, &ignores)
                .unwrap()
                .0,
            [1, 2]
        );
        assert_eq!(
            route(&catalog, &tell, context, &sessions, &ignores)
                .unwrap()
                .0,
            [2]
        );
    }
}

#[test]
fn forbidden_emote_fails_the_same_way_on_every_channel_and_reaches_nobody() {
    let catalog = recovered_catalog();
    let sessions = [
        session(1, 101, "fixture-sender", 0.0),
        session(2, 202, "fixture-target", 5.0),
    ];
    let sender = name("fixture-sender");
    let cheer = phrase("cheer");
    let say =
        encode_quick_chat_channel(&catalog, cheer, ChatChannel::Say, 101, &sender, 0).unwrap();
    let tell = encode_quick_chat_tell(&catalog, cheer, 101, &sender, "fixture-target").unwrap();
    for packet in [&say, &tell] {
        assert_eq!(
            route(
                &catalog,
                packet,
                mounted(),
                &sessions,
                &IgnoreLists::default()
            )
            .unwrap_err(),
            ChatError::EmoteRefused(EmoteRejection::Mounted)
        );
    }
    // A refusal is a silent drop (R): no client reply bytes are built.
    assert_eq!(
        ChatError::EmoteRefused(EmoteRejection::Mounted).client_reply(),
        None
    );
}

#[test]
fn ignored_sender_does_not_reach_an_ignoring_player_with_an_emote() {
    let catalog = recovered_catalog();
    let sessions = [
        session(1, 101, "fixture-sender", 0.0),
        session(2, 202, "fixture-near", 5.0),
    ];
    let mut ignores = IgnoreLists::default();
    ignores.add(202, 101);
    let packet = encode_quick_chat_channel(
        &catalog,
        phrase("cheer"),
        ChatChannel::Say,
        101,
        &name("fixture-sender"),
        0,
    )
    .unwrap();
    assert_eq!(
        route(&catalog, &packet, standing(), &sessions, &ignores)
            .unwrap()
            .0,
        [1]
    );
}

/// Canary: the gateway's old catalog (`from_ids`, built before this change)
/// treats every row as plain, so mounted emotes, Dance included, slip through.
/// If the policy ever stops reading the animation column these asserts fail.
#[test]
fn canary_catalog_without_animation_data_accepts_what_the_real_one_refuses() {
    let blind = QuickChatCatalog::from_ids([184, phrase("cheer")]);
    assert_eq!(blind.authorize_emote(184, mounted()), Ok(()));
    let real = recovered_catalog();
    assert!(real.authorize_emote(184, mounted()).is_err());
    assert!(real.authorize_emote(phrase("cheer"), mounted()).is_err());
}
