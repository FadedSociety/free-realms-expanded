//! T773: a town greeter's Talk dialog carries its explainer pages.
//!
//! Client evidence: the 26/3 reader `00a9ef10` stores the body text id at payload
//! `+0x10` and a count-prefixed linked list of 0x1c-byte response rows at `+0x20`;
//! `009f5ae0` passes `*(payload+0x10)` and each row's `+0x10` text id through
//! `0097c1e0` (`Global.Text.<id>`) into `HandlerDialog:SetDialogText`/`:SetChoiceText`,
//! hides unused rows, and keeps at most five (`wndDialog.xml` btn0..btn4). The client
//! sends the clicked row's id (`009f5890`) back in 26/6, so a second 26/3 re-renders
//! the open dialog with another body.
//!
//! The data rows live on the NPC record (`explainer_topics`); this suite walks the
//! real request path gateway -> Gameplay -> gameplay/npc.rs -> npc_interaction and
//! asserts the ids the client reads, plus that a refused row changes no saved state.

#[path = "common/npc_greetings.rs"]
mod npc_greetings;

use npc_greetings::{
    GREETING_HEADER, character_at, dialogue_choice, gameplay_dir, gameplay_talk_packet,
    locale_rows, npc_position, npc_record, select, service,
};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    npc_interaction::{
        DialoguePacket26_3, EXPLAIN_CHOICE_RESPONSE_BASE, MAX_EXPLAIN_TOPICS, NpcAction,
        NpcSelection, decode_dialogue_26_3, encode_dialogue_close_26_4, goodbye_dialogue_choice,
    },
    world_text::locale_hash,
};
use std::collections::{BTreeMap, BTreeSet};

/// The first explainer row's response id on the wire. The client echoes the clicked
/// row's id back in 26/6 (`009f5890`); 300 is Goodbye and 301 the quest row, so a tips
/// row must stay clear of both. Pinned here so the production id base cannot drift
/// without this suite noticing.
const FIRST_PAGE_ROW: i32 = 310;

/// One town greeter: the row that carries `greeting_string_id` and the explainer
/// pages in its `explainer_topics` order (the order fixes the response id).
struct Greeter {
    file: &'static str,
    npc_id: u64,
    name: &'static str,
    greeting: u32,
    topics: &'static [(u32, u32)],
}

const GREETERS: &[Greeter] = &[
    Greeter {
        file: "snowhill.json",
        npc_id: 9_000_000_093,
        name: "Frostpetal",
        greeting: 430_860,
        topics: &[(37933, 37936), (31328, 37942), (37945, 37948), (7910, 7911)],
    },
    Greeter {
        file: "cobblestone_village.json",
        npc_id: 9_000_000_189,
        name: "Samantha",
        greeting: 24_940,
        topics: &[(386_023, 20_520), (31023, 35603)],
    },
    Greeter {
        file: "highroad_junction.json",
        npc_id: 9_000_000_402,
        name: "Merigold",
        greeting: 80,
        topics: &[(386_023, 20_520), (104138, 115_776)],
    },
];

fn character(greeter: &Greeter) -> CharacterState {
    let record = npc_record(greeter.file, greeter.npc_id);
    character_at(npc_position(&record, greeter.npc_id))
}

fn new_game() -> Gameplay {
    Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads")
}

/// The dialogue rows a greeter's greeting should carry: one row per explainer page
/// in content order, then Goodbye.
fn expected_rows(greeter: &Greeter) -> Vec<[u32; 5]> {
    let mut rows: Vec<[u32; 5]> = greeter
        .topics
        .iter()
        .enumerate()
        .map(|(index, (label, _))| [(FIRST_PAGE_ROW + index as i32) as u32, 17, *label, 0, 18])
        .collect();
    rows.push(goodbye_dialogue_choice());
    rows
}

/// 26/8 opens the menu and 26/10 selects Talk; returns the decoded 26/3.
fn talk(game: &mut Gameplay, greeter: &Greeter) -> DialoguePacket26_3 {
    gameplay_talk_packet(game, greeter.file, greeter.npc_id)
}

/// The 26/3(body, rows) reply to one 26/6 row, if the server sent one.
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

/// Every greeter answers Talk with its recovered greeting as the 26/3 body and its
/// explainer pages as the response rows the client can click.
#[test]
fn every_town_greeter_offers_its_explainer_rows_behind_the_greeting() {
    let mut game = new_game();
    let mut service = service();
    for greeter in GREETERS {
        // Service path: the context the router copies into the 26/3 rows.
        let (zone, position) = service
            .location(greeter.npc_id)
            .expect("the greeter has a location");
        let menu = service
            .interact(greeter.npc_id, zone, position, None)
            .expect("the greeter offers its menu");
        let talk_option = menu
            .options
            .iter()
            .find(|option| option.action == NpcAction::Talk)
            .expect("the greeter offers Talk")
            .id;
        let chosen = u32::try_from(talk_option).expect("menu option id fits u32");
        let NpcSelection::Talk(context) = service
            .select(&select(greeter.npc_id, chosen), zone, position, None)
            .expect("Talk returns the dialogue context")
        else {
            panic!(
                "NPC {}: Talk must not return another selection",
                greeter.npc_id
            );
        };
        assert_eq!(context.body_string_id, Some(greeter.greeting));
        let topics: Vec<(i32, u32, u32)> = context
            .explainer_topics
            .iter()
            .map(|topic| {
                (
                    topic.response_id,
                    topic.label_string_id,
                    topic.body_string_id,
                )
            })
            .collect();
        let expected: Vec<(i32, u32, u32)> = greeter
            .topics
            .iter()
            .enumerate()
            .map(|(index, (label, body))| (FIRST_PAGE_ROW + index as i32, *label, *body))
            .collect();
        assert_eq!(topics, expected, "NPC {}: the context rows", greeter.npc_id);
        assert_eq!(
            EXPLAIN_CHOICE_RESPONSE_BASE, FIRST_PAGE_ROW,
            "the production response-id base is the pinned wire id"
        );

        // Dispatch path: gateway -> Gameplay -> gameplay/npc.rs -> dialogue_packet_with.
        let dialogue = talk(&mut game, greeter);
        assert_eq!(
            dialogue.initial_dwords[0], greeter.greeting,
            "NPC {} ({}): the 26/3 body stays the recovered greeting",
            greeter.npc_id, greeter.name
        );
        assert_eq!(
            dialogue.choices,
            expected_rows(greeter),
            "NPC {} ({}): the response rows are the explainer pages then Goodbye",
            greeter.npc_id,
            greeter.name
        );
        assert!(
            dialogue.choices.len() <= 5,
            "NPC {} ({}): the client dialog window has five buttons",
            greeter.npc_id,
            greeter.name
        );
        assert!(
            !greeter.topics.is_empty() && greeter.topics.len() <= MAX_EXPLAIN_TOPICS,
            "NPC {} ({}): one to four explainer pages",
            greeter.npc_id,
            greeter.name
        );
    }
}

/// Choosing an explainer row replays the same dialogue with that page's body; the
/// rows stay clickable, so a player can read another page, and Goodbye still closes.
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
            assert_eq!(
                dialogue.choices[response_id as usize - FIRST_PAGE_ROW as usize][2],
                *label,
                "NPC {} ({}): the label the client reads back",
                greeter.npc_id,
                greeter.name
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

/// A response id the dialogue never offered changes no saved state and does not
/// leak a page: the server answers the ordinary close and the NPC reopens.
#[test]
fn a_refused_explainer_row_changes_nothing_and_keeps_the_npc_usable() {
    let mut game = new_game();
    let mut fresh = new_game();
    for greeter in GREETERS {
        talk(&mut game, greeter);
        let before = serde_json::to_value(game.state()).expect("state snapshot");
        let (packet, closed) = choose(&mut game, greeter, i32::MAX);
        assert!(packet.is_none(), "an unoffered row sends no dialogue");
        assert!(closed, "an unoffered row is answered with the 26/4 close");
        let after = serde_json::to_value(game.state()).expect("state snapshot");
        assert_eq!(
            before, after,
            "NPC {} ({}): a refused row changes no inventory, coin, quest or profile state",
            greeter.npc_id, greeter.name
        );
        // The refusal must not leave the NPC stuck: interact and Talk still work.
        let dialogue = talk(&mut game, greeter);
        assert_eq!(
            dialogue.initial_dwords[0], greeter.greeting,
            "NPC {} ({}): the NPC answers a fresh Talk after a refused row",
            greeter.npc_id, greeter.name
        );

        // A greeter row selected with no dialogue open is the same ordinary close.
        let character = character(greeter);
        let before = serde_json::to_value(fresh.state()).expect("state snapshot");
        let outcome = fresh
            .handle_in_world(&dialogue_choice(FIRST_PAGE_ROW), &character, 1_200)
            .expect("26/6 is accepted");
        assert!(
            outcome
                .replies
                .iter()
                .all(|packet| *packet == encode_dialogue_close_26_4()),
            "NPC {} ({}): a row with no open dialogue only closes",
            greeter.npc_id,
            greeter.name
        );
        assert_eq!(
            before,
            serde_json::to_value(fresh.state()).expect("state snapshot"),
            "NPC {} ({}): a row with no open dialogue changes no state",
            greeter.npc_id,
            greeter.name
        );
    }
}

/// A drawable NPC with no `explainer_topics` keeps the one-row Goodbye dialog, so the
/// feature cannot leak into every NPC.
#[test]
fn an_npc_without_explainer_rows_keeps_the_plain_dialogue() {
    // Bowawow shares Merigold's file and region but recovered no greeting and no tips.
    let blank = Greeter {
        file: "highroad_junction.json",
        npc_id: 9_500_000_328,
        name: "Bowawow",
        greeting: 0,
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
        vec![osfr_tools::npc_interaction::goodbye_dialogue_choice()],
        "NPC {}: no explainer rows leak into an unpatched NPC",
        blank.npc_id
    );
}
