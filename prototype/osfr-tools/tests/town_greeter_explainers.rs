//! T773: a town greeter's Talk dialog carries its explainer pages.
//!
//! Client evidence: the 26/3 reader `00a9ef10` stores the body text id at payload
//! `+0x10` and a count-prefixed linked list of 0x1c-byte response rows at `+0x20`;
//! `009f5ae0` passes `*(payload+0x10)` and each row's `+0x10` text id through
//! `0097c1e0` (`Global.Text.<id>`) into `HandlerDialog:SetDialogText`/`:SetChoiceText`,
//! hides unused rows, and stops after five (`wndDialog.xml` btn0..btn4). The display
//! stores each row's first dword in the dialog's `this+4+index` table; `009f6aa0` reads
//! that slot for the clicked button and sends it in 26/6, so a second 26/3 re-renders
//! the open dialog with another body.
//!
//! The pages live on the NPC record (`explainer_topics`). A Talk that also offers a
//! quest sends the quest row first, then the pages, then Goodbye. This suite walks the
//! real request path gateway -> Gameplay -> gameplay/npc.rs -> npc_interaction with the
//! bundled quest catalogue bound, so the production quest givers are covered; it asserts
//! the ids the client reads, that a page replay keeps the same rows, and that a refused
//! row leaves the saved character untouched.

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
        DialoguePacket26_3, EXPLAIN_CHOICE_RESPONSE_BASE, decode_dialogue_26_3,
        encode_dialogue_close_26_4, explainer_dialogue_choice, goodbye_dialogue_choice,
        quest_dialogue_choice,
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

/// The 26/3(body, rows) reply to one 26/6 row, if the server sent one, and whether the
/// reply was the ordinary 26/4 close.
fn choose(game: &mut Gameplay, greeter: &Greeter, response_id: i32) -> (Option<Vec<u8>>, bool) {
    let character = character(greeter);
    let outcome = game
        .handle_in_world(&dialogue_choice(response_id), &character, 1_200)
        .expect("26/6 is accepted");
    let dialogue = outcome
        .replies
        .iter()
        .find(|packet| packet.starts_with(&GREETING_HEADER))
        .cloned();
    let closed = outcome
        .replies
        .iter()
        .any(|packet| *packet == encode_dialogue_close_26_4());
    (dialogue, closed)
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

/// Choosing an explainer row replays the same dialogue with that page's body; the rows
/// stay clickable (including a pending quest row), so a player can read another page,
/// and Goodbye still closes.
#[test]
fn choosing_an_explainer_row_replays_the_dialogue_with_that_page() {
    let mut game = new_game();
    for greeter in GREETERS {
        talk(&mut game, greeter);
        for (index, (label, body)) in greeter.topics.iter().enumerate() {
            let response_id = FIRST_PAGE_ROW + index as i32;
            let (packet, closed) = choose(&mut game, greeter, response_id);
            let packet = packet.unwrap_or_else(|| {
                panic!(
                    "NPC {} ({}): row {response_id} must answer 26/3",
                    greeter.npc_id, greeter.name
                )
            });
            assert!(!closed, "a replay keeps the dialog open");
            let dialogue = decode_dialogue_26_3(&packet).expect("the replay decodes");
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
        }
        // Goodbye (300) ends the conversation and leaves the greeting reachable again.
        let (packet, closed) = choose(&mut game, greeter, 300);
        assert!(packet.is_none(), "Goodbye sends no new dialogue");
        assert!(closed, "Goodbye answers the client's 26/4 close");
        assert_eq!(
            talk(&mut game, greeter).initial_dwords[0],
            greeter.greeting,
            "NPC {} ({}): a fresh Talk reopens the greeting",
            greeter.npc_id,
            greeter.name
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
        let (packet, closed) = choose(&mut game, greeter, i32::MAX);
        assert!(packet.is_none(), "an unoffered row sends no dialogue");
        assert!(
            closed,
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
    let (packet, closed) = choose(&mut fresh, &GREETERS[0], FIRST_PAGE_ROW);
    assert!(
        packet.is_none(),
        "a row with no open dialogue sends no 26/3"
    );
    assert!(closed, "a row with no open dialogue is the ordinary close");
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
