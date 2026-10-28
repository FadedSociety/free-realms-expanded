//! T773: a town greeter's Talk dialog carries its explainer pages.
//!
//! Client evidence: the 26/3 reader `00a9ef10` stores the body text id at payload
//! `+0x10` and a count-prefixed linked list of 0x1c-byte response rows at `+0x20`;
//! `009f5ae0` passes `*(payload+0x10)` and each row's `+0x10` text id through
//! `0097c1e0` (`Global.Text.<id>`) into `HandlerDialog:SetDialogText`/`:SetChoiceText`,
//! hides unused rows, and stops after five (`wndDialog.xml` btn0..btn4). The display
//! stores each row's first dword in the dialog's `this+4+index` table; `009f6aa0` reads
//! that slot for the clicked button and sends it in 26/6.
//!
//! T844: a 26/3 that arrives while the dialog is open does not re-show it. Lua
//! `DialogHandler:Show` (dialog.lua) only pushes the window context when `isDialogShown`
//! is false, and `Hide` is what clears it; the 26/4 handler `00aa0870` hides through
//! `009f6890`, which also sends the client's own 26/4. So a page row is answered with the
//! close first and then the page 26/3, and the server must keep the recorded rows across
//! that client close echo.
//!
//! The pages live on the NPC record (`explainer_topics`). A Talk that also offers a
//! quest sends the quest row first, then the pages, then Goodbye. This suite walks the
//! real request path gateway -> Gameplay -> gameplay/npc.rs -> npc_interaction with the
//! bundled quest catalogue bound, so the production quest givers are covered; it asserts
//! the ids the client reads, the close-then-page order, that a page keeps the same rows
//! answerable, and that a refused row leaves the saved character untouched.

#[path = "common/npc_greetings.rs"]
mod npc_greetings;

use npc_greetings::{
    GREETING_HEADER, character_at, content_dir, dialogue_choice, gameplay_dir,
    gameplay_talk_packet, locale_rows, npc_position, npc_record,
};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    npc_interaction::{
        DialoguePacket26_3, EXPLAIN_CHOICE_RESPONSE_BASE, QUEST_CHOICE_RESPONSE,
        decode_dialogue_26_3, decode_quest_offer_49_1, encode_dialogue_close_26_4,
        explainer_dialogue_choice, goodbye_dialogue_choice, quest_dialogue_choice,
    },
    world_text::locale_hash,
};
use std::collections::{BTreeMap, BTreeSet};

/// The first explainer row's response id on the wire. The client echoes the clicked
/// row's stored id (`009f6aa0`); 300 is Goodbye and 301 the quest row, so a tips row
/// must stay clear of both. Pinned here so the production id base cannot drift.
const FIRST_PAGE_ROW: i32 = 310;

/// One town greeter: the row that carries `greeting_string_id`, the quest its Talk can
/// offer (quest id + title locale id), and the explainer pages in `explainer_topics`
/// order (the order fixes the response id).
struct Greeter {
    file: &'static str,
    npc_id: u64,
    name: &'static str,
    greeting: u32,
    offer: Option<(u32, u32)>,
    topics: &'static [(u32, u32)],
}

const GREETERS: &[Greeter] = &[
    Greeter {
        file: "snowhill.json",
        npc_id: 9_000_000_093,
        name: "Frostpetal",
        greeting: 430_860,
        // Her quest 3110012 is defined but disabled with an unmet prerequisite
        // (data/content/quests/ninja.json), so this Talk carries no quest row.
        offer: None,
        topics: &[(37933, 37936), (31328, 37942), (37945, 37948), (7910, 7911)],
    },
    Greeter {
        file: "cobblestone_village.json",
        npc_id: 9_000_000_189,
        name: "Samantha",
        greeting: 24_940,
        offer: Some((3_000_021, 94_247)),
        topics: &[(386_023, 20_520), (31023, 35603)],
    },
    Greeter {
        file: "highroad_junction.json",
        npc_id: 9_000_000_402,
        name: "Merigold",
        greeting: 80,
        offer: Some((3_000_020, 94_247)),
        topics: &[(386_023, 20_520), (104138, 115_776)],
    },
];

fn character(greeter: &Greeter) -> CharacterState {
    let record = npc_record(greeter.file, greeter.npc_id);
    character_at(npc_position(&record, greeter.npc_id))
}

/// The real request path with the bundled split quest content bound, so a quest_giver
/// greeter's Talk builds its quest row exactly as production does.
fn new_game() -> Gameplay {
    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    game.bind_npc_quest_catalogue(&content_dir(), 1)
        .expect("bundled quest content binds");
    game
}

/// The dialogue rows a greeter's Talk should carry: the quest row when the same Talk
/// offers one, then one row per explainer page in content order, then Goodbye.
fn expected_rows(greeter: &Greeter) -> Vec<[u32; 5]> {
    let mut rows: Vec<[u32; 5]> = greeter
        .offer
        .into_iter()
        .map(|(_, title)| quest_dialogue_choice(title))
        .collect();
    rows.extend(
        greeter
            .topics
            .iter()
            .enumerate()
            .map(|(index, (label, _))| {
                explainer_dialogue_choice(FIRST_PAGE_ROW + index as i32, *label)
            }),
    );
    rows.push(goodbye_dialogue_choice());
    rows
}

/// 26/8 opens the menu and 26/10 selects Talk; returns the decoded 26/3.
fn talk(game: &mut Gameplay, greeter: &Greeter) -> DialoguePacket26_3 {
    gameplay_talk_packet(game, greeter.file, greeter.npc_id)
}

/// The replies to one 26/6 row, in wire order.
fn choose(game: &mut Gameplay, greeter: &Greeter, response_id: i32) -> Vec<Vec<u8>> {
    let character = character(greeter);
    game.handle_in_world(&dialogue_choice(response_id), &character, 1_200)
        .expect("26/6 is accepted")
        .replies
}

/// The dialog-shaping replies among `replies`, in wire order: the 26/4 close and the 26/3
/// dialogue. Markers or pins the router appends after the dialog are not part of it.
fn dialogue_replies(replies: &[Vec<u8>]) -> Vec<&[u8]> {
    let close = encode_dialogue_close_26_4();
    replies
        .iter()
        .map(Vec::as_slice)
        .filter(|packet| packet.starts_with(&GREETING_HEADER) || *packet == close.as_slice())
        .collect()
}

/// One page row's answer: the 26/4 close that lets the client's `Show` run again, then the
/// page 26/3. Returns the decoded dialogue the client re-shows.
fn page_after_close(
    replies: &[Vec<u8>],
    greeter: &Greeter,
    response_id: i32,
) -> DialoguePacket26_3 {
    let dialog = dialogue_replies(replies);
    assert_eq!(
        dialog.len(),
        2,
        "NPC {} ({}): row {response_id} answers the close and the page, found {} dialog packets",
        greeter.npc_id,
        greeter.name,
        dialog.len()
    );
    assert_eq!(
        dialog[0],
        encode_dialogue_close_26_4().as_slice(),
        "NPC {} ({}): row {response_id} must close the dialog before the page so `Show` runs again",
        greeter.npc_id,
        greeter.name
    );
    decode_dialogue_26_3(dialog[1]).expect("the page 26/3 decodes")
}

/// The client's acknowledgement of the server's close: handler `00aa0870` destroys the
/// dialog record and `009f6890` sends its own 26/4 while Lua `Hide` clears `isDialogShown`.
/// It answers nothing, changes no saved state, and must not end the server interaction.
fn close_echo(game: &mut Gameplay, greeter: &Greeter) {
    let character = character(greeter);
    let before = saved(game);
    let echo = game
        .handle_in_world(&encode_dialogue_close_26_4(), &character, 1_300)
        .expect("the client's close echo is accepted");
    assert!(
        echo.replies.is_empty(),
        "NPC {} ({}): the close echo answers nothing: {:?}",
        greeter.npc_id,
        greeter.name,
        echo.replies
    );
    assert_eq!(
        before,
        saved(game),
        "NPC {} ({}): the close echo changes no saved state",
        greeter.npc_id,
        greeter.name
    );
}

fn saved(game: &Gameplay) -> serde_json::Value {
    serde_json::to_value(game.state()).expect("state snapshot")
}

/// Every greeter answers Talk with its recovered greeting as the 26/3 body and its
/// explainer pages as the response rows the client can click. A greeter whose Talk also
/// offers a quest keeps that row, so the pages are not dropped for the offer window.
#[test]
fn every_town_greeter_offers_its_explainer_rows_behind_the_greeting() {
    let mut game = new_game();
    assert_eq!(
        EXPLAIN_CHOICE_RESPONSE_BASE, FIRST_PAGE_ROW,
        "the production response-id base is the pinned wire id"
    );
    for greeter in GREETERS {
        let dialogue = talk(&mut game, greeter);
        assert_eq!(
            dialogue.initial_dwords[0], greeter.greeting,
            "NPC {} ({}): the 26/3 body stays the recovered greeting",
            greeter.npc_id, greeter.name
        );
        assert_eq!(
            dialogue.choices,
            expected_rows(greeter),
            "NPC {} ({}): the quest row, the explainer pages, then Goodbye",
            greeter.npc_id,
            greeter.name
        );
    }
}

/// Choosing an explainer row closes the dialog and re-shows it with that page's body: the
/// client's `Show` only rebuilds the window when `isDialogShown` is false, and only `Hide`
/// clears that flag, so the client answers the 26/4 with its own close. The recorded rows
/// stay answerable across that echo (another page, then Goodbye), including a pending
/// quest row.
#[test]
fn choosing_an_explainer_row_replays_the_dialogue_with_that_page() {
    let mut game = new_game();
    for greeter in GREETERS {
        talk(&mut game, greeter);
        for (index, (label, body)) in greeter.topics.iter().enumerate() {
            let response_id = FIRST_PAGE_ROW + index as i32;
            let replies = choose(&mut game, greeter, response_id);
            let dialogue = page_after_close(&replies, greeter, response_id);
            assert_eq!(
                dialogue.initial_dwords[0], *body,
                "NPC {} ({}): row {response_id} body",
                greeter.npc_id, greeter.name
            );
            let row = dialogue
                .choices
                .iter()
                .find(|row| row[0] == response_id as u32)
                .unwrap_or_else(|| {
                    panic!(
                        "NPC {} ({}): the replay keeps row {response_id}",
                        greeter.npc_id, greeter.name
                    )
                });
            assert_eq!(
                row[2], *label,
                "NPC {} ({}): the label the client reads back",
                greeter.npc_id, greeter.name
            );
            assert_eq!(
                dialogue.choices,
                expected_rows(greeter),
                "NPC {} ({}): the page keeps the same rows",
                greeter.npc_id,
                greeter.name
            );
            // The client echoes the close before the re-shown dialog can answer.
            close_echo(&mut game, greeter);
        }
        // Goodbye (300) still closes from the re-shown dialog and leaves the greeting
        // reachable again.
        let replies = choose(&mut game, greeter, 300);
        assert_eq!(
            dialogue_replies(&replies),
            vec![encode_dialogue_close_26_4().as_slice()],
            "NPC {} ({}): Goodbye sends only the close",
            greeter.npc_id,
            greeter.name
        );
        assert_eq!(
            talk(&mut game, greeter).initial_dwords[0],
            greeter.greeting,
            "NPC {} ({}): a fresh Talk reopens the greeting",
            greeter.npc_id,
            greeter.name
        );
    }
}

/// A page row leaves the recorded quest row answerable: after the page and the client's
/// close echo, row 301 still opens that greeter's 49/1 offer.
#[test]
fn a_page_row_then_the_quest_row_still_opens_the_offer() {
    for greeter in GREETERS {
        let Some((quest_id, _title)) = greeter.offer else {
            continue;
        };
        // A fresh game per greeter: a live offer counts against the eligibility limit, so
        // the second greeter's Talk must not inherit the first greeter's offer.
        let mut game = new_game();
        talk(&mut game, greeter);
        let replies = choose(&mut game, greeter, FIRST_PAGE_ROW);
        page_after_close(&replies, greeter, FIRST_PAGE_ROW);
        close_echo(&mut game, greeter);
        let replies = choose(&mut game, greeter, QUEST_CHOICE_RESPONSE);
        let offer = replies
            .iter()
            .find_map(|packet| decode_quest_offer_49_1(packet).ok())
            .unwrap_or_else(|| {
                panic!(
                    "NPC {} ({}): the quest row after a page must open 49/1",
                    greeter.npc_id, greeter.name
                )
            });
        assert_eq!(
            offer.quest_id, quest_id,
            "NPC {} ({}): the offer names the greeter's own quest",
            greeter.npc_id, greeter.name
        );
    }
}

/// Every label and body id is a real `Global.Text` key in the stock client locale,
/// and the record carries the client data row that names it.
#[test]
fn each_explainer_id_resolves_to_a_stock_client_line() {
    let wanted: BTreeSet<u32> = GREETERS
        .iter()
        .flat_map(|greeter| {
            greeter
                .topics
                .iter()
                .flat_map(|(label, body)| [*label, *body])
        })
        .map(|id| locale_hash(i32::try_from(id).expect("a stock locale id fits i32")))
        .collect();
    let locale: BTreeMap<u32, (usize, String)> = locale_rows(&wanted);
    let mut bodies = BTreeMap::new();
    for greeter in GREETERS {
        let record = npc_record(greeter.file, greeter.npc_id);
        assert_eq!(
            record["greeting_string_id"].as_u64(),
            Some(u64::from(greeter.greeting)),
            "NPC {} ({}): the greeting binding the rows hang off",
            greeter.npc_id,
            greeter.name
        );
        let topics = record["explainer_topics"]
            .as_array()
            .unwrap_or_else(|| panic!("NPC {}: explainer_topics", greeter.npc_id));
        let bound: Vec<(u32, u32)> = topics
            .iter()
            .map(|topic| {
                (
                    u32::try_from(topic["label_string_id"].as_u64().expect("label id"))
                        .expect("label fits u32"),
                    u32::try_from(topic["body_string_id"].as_u64().expect("body id"))
                        .expect("body fits u32"),
                )
            })
            .collect();
        assert_eq!(
            bound, greeter.topics,
            "NPC {} ({}) content order",
            greeter.npc_id, greeter.name
        );
        let sources = record["field_sources"]["explainer_topics"]
            .as_array()
            .unwrap_or_else(|| panic!("NPC {}: explainer_topics sources", greeter.npc_id));
        assert_eq!(
            sources.len(),
            2,
            "NPC {}: one client and one reconstructed source",
            greeter.npc_id
        );
        let client = sources
            .iter()
            .find(|source| source["kind"] == "client")
            .unwrap_or_else(|| panic!("NPC {}: client source", greeter.npc_id));
        let evidence = client["evidence"].as_str().expect("client evidence");
        assert!(
            evidence.contains("Resources/MiniGameData.txt"),
            "NPC {}: the client source names the data rows it read",
            greeter.npc_id
        );
        assert!(
            sources
                .iter()
                .any(|source| source["kind"] == "reconstructed"
                    && source["evidence"]
                        .as_str()
                        .is_some_and(|text| text.starts_with("(R)"))),
            "NPC {}: the greeter binding is marked reconstructed",
            greeter.npc_id
        );
    }
    for greeter in GREETERS {
        let mut labels = BTreeSet::new();
        for (label, body) in greeter.topics {
            assert!(
                labels.insert(*label),
                "NPC {}: label {label} repeats in one list",
                greeter.npc_id
            );
            assert_ne!(
                label, body,
                "NPC {}: a label is not its own page body",
                greeter.npc_id
            );
            for (kind, id) in [("label", *label), ("body", *body)] {
                let (line, text) = locale
                    .get(&locale_hash(i32::try_from(id).expect("id fits i32")))
                    .unwrap_or_else(|| {
                        panic!(
                            "NPC {}: {kind} {id} is not a stock Global.Text row",
                            greeter.npc_id
                        )
                    });
                assert!(
                    !text.trim().is_empty(),
                    "NPC {}: {kind} {id} (line {line}) is blank",
                    greeter.npc_id
                );
                if kind == "label" {
                    assert!(
                        text.chars().count() <= 64,
                        "NPC {}: label {id} must fit a dialog button: {text}",
                        greeter.npc_id
                    );
                } else {
                    bodies.insert(*body, greeter.npc_id);
                }
            }
        }
    }
    assert!(
        bodies.len() >= GREETERS.len(),
        "the pages are real, distinct explanations, not one shared string"
    );
}

/// A row the dialogue never sent is answered with the ordinary 26/4 close: it changes no
/// saved state, ends only this session's dialog, and the NPC reopens with its rows. No
/// page, quest, item or coin is invented by a refused row.
#[test]
fn a_refused_row_changes_no_saved_state_and_the_npc_reopens() {
    let mut game = new_game();
    for greeter in GREETERS {
        talk(&mut game, greeter);
        let before = saved(&game);
        let replies = choose(&mut game, greeter, i32::MAX);
        assert_eq!(
            dialogue_replies(&replies),
            vec![encode_dialogue_close_26_4().as_slice()],
            "an unoffered row is answered with the ordinary 26/4 close"
        );
        assert_eq!(
            before,
            saved(&game),
            "NPC {} ({}): a refused row changes no saved state",
            greeter.npc_id,
            greeter.name
        );
        // The close leaves nothing stuck: a fresh Talk reopens the greeting and rows.
        let dialogue = talk(&mut game, greeter);
        assert_eq!(
            dialogue.initial_dwords[0], greeter.greeting,
            "NPC {} ({}): a fresh Talk answers after a refused row",
            greeter.npc_id, greeter.name
        );
        assert_eq!(
            dialogue.choices,
            expected_rows(greeter),
            "NPC {} ({}): the refused row did not consume a page",
            greeter.npc_id,
            greeter.name
        );
    }

    // A row sent with no dialogue open is the same ordinary close and changes nothing.
    let mut fresh = new_game();
    let before = saved(&fresh);
    let replies = choose(&mut fresh, &GREETERS[0], FIRST_PAGE_ROW);
    assert_eq!(
        dialogue_replies(&replies),
        vec![encode_dialogue_close_26_4().as_slice()],
        "a row with no open dialogue is the ordinary close"
    );
    assert_eq!(
        before,
        saved(&fresh),
        "a row with no open dialogue changes no saved state"
    );
}

/// A drawable NPC with no `explainer_topics` keeps the one-row Goodbye dialog, so the
/// feature cannot leak into every NPC.
#[test]
fn an_npc_without_explainer_rows_keeps_the_plain_dialogue() {
    // Bowawow shares Merigold's file and region but recovered no greeting, no tips and
    // no quest.
    let blank = Greeter {
        file: "highroad_junction.json",
        npc_id: 9_500_000_328,
        name: "Bowawow",
        greeting: 0,
        offer: None,
        topics: &[],
    };
    let record = npc_record(blank.file, blank.npc_id);
    assert!(
        record.get("explainer_topics").is_none(),
        "NPC {}: the control row must stay unpatched",
        blank.npc_id
    );
    assert!(
        record.get("greeting_string_id").is_none(),
        "NPC {}: the control row has no greeting",
        blank.npc_id
    );
    let mut game = new_game();
    let dialogue = talk(&mut game, &blank);
    assert_eq!(dialogue.initial_dwords[0], 0, "no body without a greeting");
    assert_eq!(
        dialogue.choices,
        vec![goodbye_dialogue_choice()],
        "NPC {}: no explainer rows leak into an unpatched NPC",
        blank.npc_id
    );
}
