//! T894: talker NPC rows whose own first-person client line is recovered carry it as
//! `greeting_string_id`. Content only: `NpcInteraction::dialogue_context` copies the id into
//! `DialogueContext.body_string_id` and `gameplay/npc.rs::dialogue_packet_with` writes it into
//! 26/3 (client handler `00aa12e0`, payload reader `00a9ef10`, display `009f5ae0`; see
//! `tests/common/npc_greetings.rs`). Speaker proofs are in each row's `field_sources` and in
//! `notes/Autopilot/T894.md`.

#[path = "common/npc_greetings.rs"]
mod npc_greetings;

use npc_greetings::{
    gameplay_dir, gameplay_talk_body, greeting_bindings, locale_id_to_text, npc_record, service,
    service_talk_body,
};
use osfr_tools::gameplay::Gameplay;

/// (content file, npc id, name, bound `Global.Text` id)
const ROWS: [(&str, u64, &str, u32); 7] = [
    ("merry_vale.json", 9500000422, "Rylan", 427591),
    ("sanctuary.json", 9500000435, "Abacus Coinmiser", 5102533),
    ("snowhill.json", 9500000729, "Bruce", 4419632),
    ("durango_zoo.json", 9500000257, "Glenn", 116073),
    ("wilds.json", 9500000953, "Tooga", 70733),
    ("snowhill.json", 9500000764, "Roland Sporeling", 432172),
    (
        "highroad_junction.json",
        9500000337,
        "Sammy \"The Shark\"",
        436638,
    ),
];

/// Lines the census listed for a talker but another speaker says: Archnos's lecture hinted to
/// Burpy (the toad) and a third-person message about Fizzi. Binding them would put the wrong
/// speaker's words in the NPC's dialogue body.
const NOT_THEIR_LINES: [u32; 2] = [24994, 71826];

#[test]
fn each_talker_row_shows_its_own_line_when_talked_to() {
    let mut service = service();
    for (_, npc_id, name, greeting) in ROWS {
        assert_eq!(
            service_talk_body(&mut service, npc_id),
            Some(greeting),
            "{name} ({npc_id}): Talk body"
        );
    }
}

#[test]
fn bound_ids_are_real_client_lines_with_sources_and_stay_unique() {
    let text = locale_id_to_text();
    let bindings = greeting_bindings();
    for (file, npc_id, name, greeting) in ROWS {
        assert!(
            text.get(&u64::from(greeting))
                .is_some_and(|t| !t.is_empty()),
            "{name}: Global.Text.{greeting} must be a stock client line"
        );
        let record = npc_record(file, npc_id);
        assert_eq!(record["name"], name);
        let sources = record["field_sources"]["greeting_string_id"]
            .as_array()
            .unwrap_or_else(|| panic!("{name}: greeting field_sources"));
        let kinds: Vec<&str> = sources.iter().filter_map(|s| s["kind"].as_str()).collect();
        assert_eq!(
            kinds,
            ["client", "reconstructed"],
            "{name}: provenance kinds"
        );
        assert!(
            sources[1]["evidence"]
                .as_str()
                .is_some_and(|e| e.starts_with("(R) T894")),
            "{name}: the reconstructed claim is marked (R)"
        );
        let users = bindings.values().filter(|id| **id == greeting).count();
        assert_eq!(users, 1, "{name}: greeting {greeting} is bound by one row");
    }
}

#[test]
fn other_speakers_lines_are_not_bound_to_the_talker_they_name() {
    let bound: Vec<u32> = greeting_bindings().into_values().collect();
    for line in NOT_THEIR_LINES {
        assert!(!bound.contains(&line), "{line} is another speaker's line");
    }
}

#[test]
fn the_drawn_row_sends_its_line_in_26_3() {
    // Dispatch path: gateway -> Gameplay -> gameplay/npc.rs -> dialogue_packet_with. Bruce is
    // the one edited row placed from legacy data, so the world draws it and the router answers.
    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    assert_eq!(
        gameplay_talk_body(&mut game, "snowhill.json", 9500000729),
        4419632,
        "26/3 body dword"
    );
}
