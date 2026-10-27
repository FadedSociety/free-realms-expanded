//! T546: the Warpwatcher brothers' interaction menus carry their recovered
//! client line.
//!
//! The two NPCs are spawned by `npc_spawns_extra.json` rows (Kellin 9500000502
//! near the Sanctuary warpstone, Vaelen 9500000778 near the Snowhill one) and
//! their content records had no `greeting_string_id`. `NpcInteraction::interact`
//! already offers Talk for every row, so the menu opened with a zero body
//! (`dialogue_packet_with` writes `body_string_id.unwrap_or(0)`); this slice binds
//! the line each brother speaks when he offers the warp test.
//!
//! Content only: no packet, handler, table or flag changes. The service path is
//! `NpcInteraction::interact` -> `select`; the dispatch path is
//! `Gameplay::handle_in_world` -> `gameplay/npc.rs` -> `dialogue_packet_with`.
//! The 26/3 body is the first dword after the four-byte header (client handler
//! `00aa12e0`, reader `00a9ef10`, display `009f5ae0`, pinned by
//! `tests/npc_interaction.rs::dialogue_26_3_synthetic_fixture_round_trips_exact_native_field_order`).
//! Locale ids, hashes and `en_us_data.dat` lines are in `notes/Autopilot/T546.md`.
//! The shared helpers live in `tests/common/npc_greetings.rs`.

#[path = "common/npc_greetings.rs"]
mod npc_greetings;

use npc_greetings::{
    gameplay_dir, gameplay_talk_body, greeting_bindings, locale_id_to_text, locale_rows,
    npc_record, select, service, service_talk_body,
};
use osfr_tools::{
    gameplay::Gameplay,
    npc_interaction::{NpcAction, NpcInteractionError, NpcSelection},
    world::NpcVisibility,
    world_text::locale_hash,
};
use std::collections::BTreeSet;

/// One Warpwatcher row: content key, file, speaker proof and the client line.
struct Warpwatcher {
    npc_id: u64,
    file: &'static str,
    name: &'static str,
    greeting: u32,
    hash: u32,
    dat_line: u32,
    guide: &'static str,
    guide_line: u32,
}

const WARPWATCHERS: [Warpwatcher; 2] = [
    Warpwatcher {
        npc_id: 9_500_000_502,
        file: "sanctuary.json",
        name: "Kellin Warpwatcher",
        // Snowhill Warp Tests offer line.
        greeting: 385_689,
        hash: 1_685_799_068,
        dat_line: 44_579,
        guide: "NPCs - Sanctuary.md",
        guide_line: 36,
    },
    Warpwatcher {
        npc_id: 9_500_000_778,
        file: "snowhill.json",
        name: "Vaelen Warpwatcher",
        // Sanctuary Warp Tests offer line.
        greeting: 385_679,
        hash: 652_874_222,
        dat_line: 17_287,
        guide: "NPCs - Snowhill.md",
        guide_line: 97,
    },
];

#[test]
fn the_warpwatchers_answer_talk_through_the_service_and_the_router() {
    let mut service = service();
    let visibility = NpcVisibility::load(&npc_greetings::content_dir())
        .expect("the production visibility loader accepts the content");
    let bindings = greeting_bindings();
    let mut seen = BTreeSet::new();
    for watcher in &WARPWATCHERS {
        assert!(
            seen.insert(watcher.greeting),
            "{}: greeting {} is reused",
            watcher.name,
            watcher.greeting
        );
        assert_eq!(
            bindings
                .values()
                .filter(|greeting| **greeting == watcher.greeting)
                .count(),
            1,
            "{}: the greeting must belong to this NPC only",
            watcher.name
        );
        // The spawned actor resolves to the row whose greeting this is.
        assert_eq!(
            visibility.content_key(watcher.npc_id),
            Some(watcher.npc_id),
            "{} is not spawned by NpcVisibility",
            watcher.name
        );
        assert_eq!(
            service_talk_body(&mut service, watcher.npc_id),
            Some(watcher.greeting),
            "{}: service Talk body",
            watcher.name
        );
    }

    // Dispatch path: gateway -> Gameplay -> gameplay/npc.rs -> 26/3.
    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    for watcher in &WARPWATCHERS {
        assert_eq!(
            gameplay_talk_body(&mut game, watcher.file, watcher.npc_id),
            watcher.greeting,
            "{}: 26/3 body through the router",
            watcher.name
        );
    }
}

#[test]
fn the_warpwatcher_greetings_resolve_as_global_text_and_cite_their_speaker() {
    let wanted: BTreeSet<u32> = WARPWATCHERS.iter().map(|watcher| watcher.hash).collect();
    let locale = locale_rows(&wanted);
    let id_to_text = locale_id_to_text();
    for watcher in &WARPWATCHERS {
        let record = npc_record(watcher.file, watcher.npc_id);
        assert_eq!(record["name"].as_str(), Some(watcher.name));
        assert_eq!(
            record["greeting_string_id"].as_u64(),
            Some(u64::from(watcher.greeting)),
            "{}: greeting binding",
            watcher.name
        );
        let sources = record["field_sources"]["greeting_string_id"]
            .as_array()
            .unwrap_or_else(|| panic!("{}: greeting sources", watcher.name));
        let client = sources
            .iter()
            .find(|source| source["kind"] == "client")
            .unwrap_or_else(|| panic!("{}: client greeting source", watcher.name));
        let evidence = client["evidence"].as_str().unwrap_or_default();
        for needle in [
            format!("Global.Text.{}", watcher.greeting),
            format!("hash {}", watcher.hash),
            format!("line {}", watcher.dat_line),
            format!("{}:{}", watcher.guide, watcher.guide_line),
        ] {
            assert!(
                evidence.contains(&needle),
                "{}: source must name {needle:?}",
                watcher.name
            );
        }
        assert!(
            sources.iter().any(|source| {
                source["kind"] == "reconstructed"
                    && source["evidence"]
                        .as_str()
                        .is_some_and(|text| text.starts_with("(R)"))
            }),
            "{}: the reconstruction must be labelled (R)",
            watcher.name
        );

        // Resolve the id the way the client does: hash Global.Text.<id>, then find
        // that hash on the cited physical locale line.
        assert_eq!(
            locale_hash(watcher.greeting as i32),
            watcher.hash,
            "{}: greeting id must be a Global.Text key, not a raw hash",
            watcher.name
        );
        let (line, text) = locale
            .get(&watcher.hash)
            .unwrap_or_else(|| panic!("{}: the locale hash is missing", watcher.name));
        assert_eq!(
            *line, watcher.dat_line as usize,
            "{}: locale line",
            watcher.name
        );
        assert_eq!(
            Some(text.clone()),
            id_to_text.get(&u64::from(watcher.greeting)).cloned(),
            "{}: the locale text must match the id-to-text report",
            watcher.name
        );
        assert!(
            !text.trim().is_empty(),
            "{}: empty locale text",
            watcher.name
        );
    }
}

#[test]
fn a_refused_warpwatcher_choice_leaves_the_menu_and_binding_unchanged() {
    let watcher = &WARPWATCHERS[0];
    let record_before = npc_record(watcher.file, watcher.npc_id);
    let mut service = service();
    let (zone, position) = service
        .location(watcher.npc_id)
        .expect("the Warpwatcher has a location");
    let menu = service
        .interact(watcher.npc_id, zone, position, None)
        .expect("the Warpwatcher menu opens");
    let talk = menu
        .options
        .iter()
        .find(|option| option.action == NpcAction::Talk)
        .expect("Talk is offered")
        .id;
    assert!(matches!(
        service.select(
            &select(watcher.npc_id, i32::MAX as u32),
            zone,
            position,
            None
        ),
        Err(NpcInteractionError::InvalidOption)
    ));
    match service.select(&select(watcher.npc_id, talk as u32), zone, position, None) {
        Ok(NpcSelection::Talk(context)) => assert_eq!(
            context.body_string_id,
            Some(watcher.greeting),
            "the refused choice must not consume the menu"
        ),
        _ => panic!("Talk must still return dialogue"),
    }
    // A refusal is not a content edit: the row still carries the same binding.
    assert_eq!(
        npc_record(watcher.file, watcher.npc_id)["greeting_string_id"],
        record_before["greeting_string_id"]
    );
}
