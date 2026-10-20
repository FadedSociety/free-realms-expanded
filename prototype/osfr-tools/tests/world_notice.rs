//! T199: `realm broadcast <text>` through the real gameplay request path.
//! The multi-session commit/fanout path lives in the bin rig
//! (`src/bin/osfr_gateway/t199_world_notice.rs`): it drives `Session::dispatch_message` and,
//! for the replay case, `Session::handle` with a real reliable datagram (not a socket read).
use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    chat,
    cooking::{CookingPhase, CookingRun},
    gameplay::{Gameplay, Outcome},
    world_notice::{self, MAX_BROADCAST_BYTES, MAX_BROADCAST_CHARS, NoticeError},
};
use std::path::PathBuf;

fn content(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references")
        .join(name)
}

fn game(directory: &str, role: AccountRole) -> (Gameplay, CharacterState) {
    let directory = content(directory);
    let character = Gameplay::initial_character(
        &directory,
        CharacterState {
            guid: 1,
            zone_id: 0,
            position: [0.0; 3],
            rotation: [0.0, 0.0, 1.0],
        },
    )
    .unwrap();
    let mut game = Gameplay::load(&directory, None).unwrap();
    game.set_dev_tools(true);
    game.set_role(role);
    (game, character)
}

/// The opening-stage pack takes `handle_opening_inner`; `rust-gameplay` has no
/// `opening.json`, so the same command reaches the opening-less `handle` path.
const OPENING: &str = "rust-opening-cooking-original-assets";
const PLAIN: &str = "rust-gameplay";

fn send(game: &mut Gameplay, character: &CharacterState, text: &str) -> Outcome {
    game.handle_in_world(&chat::local_request(text), character, 1_000)
        .unwrap()
}

/// The text of one 15/3 `ChatPacketDebugChat` inner packet.
fn system_text(packet: &[u8]) -> String {
    assert_eq!(&packet[..4], &[15, 0, 3, 0], "not a 15/3 system line");
    let length = u32::from_le_bytes(packet[4..8].try_into().unwrap()) as usize;
    assert_eq!(packet.len(), 8 + length + 1);
    String::from_utf8(packet[8..8 + length].to_vec()).unwrap()
}

fn reply_texts(out: &Outcome) -> Vec<String> {
    out.replies
        .iter()
        .map(|packet| system_text(packet))
        .collect()
}

#[test]
fn admin_broadcast_sets_one_system_line_without_character_change() {
    for pack in [OPENING, PLAIN] {
        let (mut game, character) = game(pack, AccountRole::Admin);
        let before = game.state();
        let out = send(
            &mut game,
            &character,
            "realm broadcast Scavenger hunt at noon",
        );
        assert_eq!(
            out.broadcast.as_deref(),
            Some("Scavenger hunt at noon"),
            "{pack}"
        );
        assert!(out.replies.is_empty(), "{pack}: {:?}", reply_texts(&out));
        assert!(!out.changed, "{pack}: a notice must not mark state changed");
        assert_eq!(game.state(), before, "{pack}: character state changed");
        let packet = world_notice::encode(out.broadcast.as_deref().unwrap());
        assert_eq!(&packet[..4], &[15, 0, 3, 0]);
        assert_eq!(system_text(&packet), "Scavenger hunt at noon");
    }
}

#[test]
fn player_and_builder_broadcasts_are_refused_without_a_notice() {
    for role in [AccountRole::Player, AccountRole::Builder] {
        for pack in [OPENING, PLAIN] {
            let (mut game, character) = game(pack, role);
            let before = game.state();
            let out = send(&mut game, &character, "realm broadcast hello");
            assert!(out.broadcast.is_none(), "{role:?}/{pack}");
            assert!(out.dev_command_denied, "{role:?}/{pack}");
            let texts = reply_texts(&out);
            assert_eq!(texts.len(), 1, "{role:?}/{pack}: {texts:?}");
            assert!(
                texts[0].contains("Cannot do that"),
                "{role:?}/{pack}: {texts:?}"
            );
            assert_eq!(game.state(), before, "{role:?}/{pack}: state changed");
        }
    }
}

#[test]
fn empty_overlong_multibyte_or_control_text_is_refused() {
    let (mut game, character) = game(OPENING, AccountRole::Admin);
    // Literal bounds: a changed production constant must fail these pins.
    assert_eq!((MAX_BROADCAST_CHARS, MAX_BROADCAST_BYTES), (200, 2048));
    let before = game.state();
    let cases = [
        ("realm broadcast".to_owned(), "needs a message"),
        ("realm broadcast    ".to_owned(), "needs a message"),
        // 201 characters is one past the bound.
        (
            format!("realm broadcast {}", "a".repeat(201)),
            "200 characters maximum",
        ),
        // 201 four-byte characters are 804 bytes: still the character bound.
        (
            format!("realm broadcast {}", "😀".repeat(201)),
            "200 characters maximum",
        ),
        (
            "realm broadcast line\r\nbreak".to_owned(),
            "control characters",
        ),
        (
            "realm broadcast line\nbreak".to_owned(),
            "control characters",
        ),
        (
            "realm broadcast line\tbreak".to_owned(),
            "control characters",
        ),
        ("realm broadcast nul\0byte".to_owned(), "control characters"),
    ];
    for (request, expected) in cases {
        let out = send(&mut game, &character, &request);
        assert!(out.broadcast.is_none(), "{request:?}");
        assert!(!out.changed, "{request:?}");
        let texts = reply_texts(&out);
        assert_eq!(texts.len(), 1, "{request:?}: {texts:?}");
        assert!(texts[0].contains(expected), "{request:?}: {texts:?}");
        assert_eq!(game.state(), before, "{request:?}: refusal changed state");
    }
    // `chat::request` caps one message field at 2 KiB, so a larger wire line never reaches
    // gameplay (it is dropped, not answered); the module bound still refuses direct callers.
    let huge = format!("realm broadcast {}", "a".repeat(2049));
    assert_eq!(
        world_notice::parse(&huge["realm ".len()..]),
        Err(NoticeError::TooManyBytes)
    );
    assert!(
        game.handle_in_world(&chat::local_request(&huge), &character, 1_000)
            .is_err()
    );
    assert_eq!(game.state(), before);
    // The exact boundary is still a valid announcement whose packet fits one system line.
    let accepted = "a".repeat(200);
    let out = send(
        &mut game,
        &character,
        &format!("realm broadcast {accepted}"),
    );
    assert_eq!(out.broadcast.as_deref(), Some(accepted.as_str()));
    assert!(out.replies.is_empty());
    assert_eq!(game.state(), before);
    let multibyte = "😀".repeat(200);
    let out = send(
        &mut game,
        &character,
        &format!("realm broadcast {multibyte}"),
    );
    assert_eq!(out.broadcast.as_deref(), Some(multibyte.as_str()));
    let packet = world_notice::encode(out.broadcast.as_deref().unwrap());
    assert!(packet.len() <= 8 + 2048 + 1);
    assert_eq!(game.state(), before);
}

#[test]
fn announcement_text_is_trimmed_escaped_and_never_echoes_the_command() {
    let (mut game, character) = game(OPENING, AccountRole::Admin);
    let out = send(&mut game, &character, "realm broadcast   <hunt> & tips  ");
    assert_eq!(out.broadcast.as_deref(), Some("<hunt> & tips"));
    assert!(out.replies.is_empty());
    let packet = world_notice::encode(out.broadcast.as_deref().unwrap());
    assert_eq!(&packet[8..packet.len() - 1], b"&lt;hunt&gt; &amp; tips");
    assert!(!String::from_utf8_lossy(&packet).contains("realm"));
}

/// An active crafting movie owns the character: the opening path's intercept must refuse the
/// command before the `realm` parser sees it, so the notice can never be staged mid-movie. The
/// bin rig cannot reach a movie without the private cooking-station fixture, so the ordering is
/// pinned here with the same `GameplayState` shape the station fixture saves.
#[test]
fn an_active_cooking_movie_refuses_the_broadcast_before_staging_it() {
    let (game, character) = game(OPENING, AccountRole::Admin);
    let mut state = game.state();
    let cooking = state
        .cooking
        .as_mut()
        .expect("the opening pack loads cooking content");
    let instance = cooking.next_instance_id;
    cooking.next_instance_id += 1;
    cooking.active = Some(CookingRun {
        instance_id: instance,
        phase: CookingPhase::Playing,
        data_sent: false,
        round_stats: [[None; 4]; 6],
        game_stats: [None; 4],
        score: 0,
    });
    let mut movie = Gameplay::load(&content(OPENING), Some(state)).unwrap();
    movie.set_dev_tools(true);
    movie.set_role(AccountRole::Admin);
    let before = movie.state();
    let out = send(
        &mut movie,
        &character,
        "realm broadcast Scavenger hunt at noon",
    );
    assert!(
        out.broadcast.is_none(),
        "the movie intercept let the notice through"
    );
    let texts = reply_texts(&out);
    assert_eq!(texts.len(), 1, "{texts:?}");
    assert!(texts[0].contains("Close the cooking game"), "{texts:?}");
    assert_eq!(movie.state(), before, "the movie refusal changed state");
}
