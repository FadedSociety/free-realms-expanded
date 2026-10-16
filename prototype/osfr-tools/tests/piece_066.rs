//! Piece 066 (historical quest catalogue) tests: the recovered catalogue of
//! supplied, community, wiki, local and engine quest sources.
//!
//! Evidence boundary, read before the assertions:
//!
//! ORIGINAL (recovered first-party/community sources, hashes pinned in
//! `notes/Research/original-quest-coverage.json` and the lab report):
//! * supplied journal: 25 numeric quest records spliced from the recorded
//!   `PacketSendSelfToClient.json` character data (title/description locale ids,
//!   objectives with required counts, reward bundle item references);
//! * community journal `StartingZone.cs`: 13 region-1 hub quest registrations
//!   (hub ids 2513/2514, 2521-2526, 2527-2530, 2566) that carry no recovered
//!   title, objective, prerequisite or reward;
//! * community journal stickers: 6 region-1 stickers with locale titles
//!   (2563-2565, 2572, 2573, 2587);
//! * wiki: 52 revision-pinned quest pages; the two Farnum's Farm opening pages
//!   carry prerequisites, start NPC, goals and coin amounts.
//!
//! RECONSTRUCTED (labelled, not historical): the 17 reserved `910xxx` local
//! opening stages. Every one keeps `original_quest_id` null.
//!
//! NOT CLAIMED: any numeric binding between a local stage and an original quest,
//! any objective/reward for the 13 hub registrations, and any native run of an
//! original quest. Those stay explicit gaps; the catalogue must not invent them.
// The module is not registered in `src/lib.rs` (lead-owned). Including the exact
// source file keeps this target compiling against the real artifact. Integration
// step for the lead: register the module and drop this `#[path]` include.
#[path = "../src/restoration/piece_066.rs"]
#[allow(dead_code)]
mod piece_066;

use piece_066::{
    QUEST_SOURCE_COMMUNITY_JOURNAL, QUEST_SOURCE_LOCAL_OPENING, QUEST_SOURCE_SUPPLIED_JOURNAL,
    QUEST_SOURCE_WIKI, QuestCatalogue, QuestId,
};

fn catalogue() -> QuestCatalogue {
    QuestCatalogue::load_workspace().expect("workspace quest catalogue loads")
}

#[test]
fn catalogue_reports_the_recovered_corpus_totals() {
    let catalogue = catalogue();
    assert_eq!(catalogue.schema_version, 1);
    assert_eq!(catalogue.count(QUEST_SOURCE_SUPPLIED_JOURNAL), 25);
    assert_eq!(catalogue.objective_count(QUEST_SOURCE_SUPPLIED_JOURNAL), 42);
    assert_eq!(catalogue.opening_hub_quests().len(), 13);
    assert_eq!(catalogue.region_one_stickers().len(), 6);
    // The row list carries the two opening wiki pages; the 52-page corpus is a
    // counted, hash-pinned source that the community cross-checks cover.
    assert_eq!(catalogue.count(QUEST_SOURCE_WIKI), 2);
    assert_eq!(catalogue.corpus_totals.get("wiki-quest-pages"), Some(&52));
    assert_eq!(catalogue.wiki_opening_pages().len(), 2);
    assert_eq!(catalogue.count(QUEST_SOURCE_LOCAL_OPENING), 17);
    assert_eq!(catalogue.objective_count(QUEST_SOURCE_LOCAL_OPENING), 51);
    assert_eq!(catalogue.engine_definitions.len(), 2);
    // The catalogue's own totals block and the row scan agree.
    assert_eq!(catalogue.corpus_totals.get("supplied-journal"), Some(&25));
    assert_eq!(
        catalogue.corpus_totals.get("supplied-journal-objectives"),
        Some(&42)
    );
    assert_eq!(catalogue.cross_checks.len(), 2);
    assert!(
        catalogue
            .cross_checks
            .iter()
            .all(|check| check.status == "match")
    );
}

#[test]
fn supplied_records_expose_title_objective_and_reward_sources() {
    let catalogue = catalogue();
    let yetis = catalogue
        .quest(QUEST_SOURCE_SUPPLIED_JOURNAL, &QuestId::Int(1617))
        .expect("supplied record 1617");
    assert_eq!(yetis.title_locale_id, Some(389427));
    assert_eq!(
        yetis.title_locale.as_deref(),
        Some("Harassing the Harassers")
    );
    assert_eq!(yetis.objectives.len(), 1);
    assert_eq!(yetis.objectives[0].id, Some(QuestId::Int(14096)));
    assert_eq!(
        yetis.objectives[0].text.as_deref(),
        Some("Defeat Petty Yetis")
    );
    assert_eq!(yetis.objectives[0].required, Some(20));
    assert_eq!(yetis.reward_refs.len(), 1);
    assert_eq!(yetis.reward_refs[0].item_definition, Some(1843));
    assert_eq!(yetis.reward_refs[0].quantity, Some(1));

    // A locale-bearing record resolves its text from the locale table.
    let smoothie = catalogue
        .quest(QUEST_SOURCE_SUPPLIED_JOURNAL, &QuestId::Int(1029))
        .expect("supplied record 1029");
    assert_eq!(smoothie.title_locale_id, Some(90150));
    assert_eq!(smoothie.title_locale.as_deref(), Some("Smoothie Operator"));
}

#[test]
fn wiki_pages_carry_pinned_revision_prerequisite_and_start_npc() {
    let catalogue = catalogue();
    let weapon = catalogue
        .quest(
            QUEST_SOURCE_WIKI,
            &QuestId::Text("Quest:Your First Weapon".into()),
        )
        .expect("wiki page Quest:Your First Weapon");
    assert_eq!(weapon.revision_id, Some(27815));
    assert_eq!(
        weapon.previous.as_deref(),
        Some("Quest:Learn to Mine & Smelt Ore")
    );
    assert_eq!(weapon.next.as_deref(), Some("Quest:Fighting off the Pack"));
    assert_eq!(weapon.coin_reward, Some(16));
    assert_eq!(weapon.quest_series.as_deref(), Some("Farnum's Farm"));
    assert_eq!(weapon.opening_arc, Some(true));
    let npc = weapon.npc.as_deref().expect("start NPC");
    assert!(
        npc.starts_with("Cleetus Farnum"),
        "unexpected NPC text {npc:?}"
    );
    assert!(!weapon.goals.is_empty());

    let pack = catalogue
        .quest(
            QUEST_SOURCE_WIKI,
            &QuestId::Text("Quest:Fighting off the Pack".into()),
        )
        .expect("wiki page Quest:Fighting off the Pack");
    assert_eq!(pack.revision_id, Some(28955));
    assert_eq!(pack.coin_reward, Some(18));
    assert_eq!(pack.previous.as_deref(), Some("Quest:Your First Weapon"));
    assert_eq!(pack.next.as_deref(), Some("Quest:Eviction Notice"));
}

#[test]
fn community_journal_rows_keep_unrecovered_fields_explicit() {
    let catalogue = catalogue();
    let opening: Vec<i64> = catalogue
        .opening_hub_quests()
        .iter()
        .map(|row| row.id.as_int().expect("numeric hub registration id"))
        .collect();
    assert_eq!(
        opening,
        vec![
            2513, 2514, 2521, 2522, 2523, 2524, 2525, 2526, 2527, 2528, 2529, 2530, 2566
        ]
    );
    for row in catalogue.opening_hub_quests() {
        assert!(row.title_locale_id.unwrap_or(0) == 0);
        assert!(row.title.is_none());
        assert!(row.objectives.is_empty());
        assert!(row.reward_refs.is_empty());
    }
    let sticker_ids: Vec<i64> = catalogue
        .region_one_stickers()
        .iter()
        .map(|row| row.id.as_int().expect("numeric sticker id"))
        .collect();
    assert_eq!(sticker_ids, vec![2563, 2564, 2565, 2572, 2573, 2587]);
    let stew = catalogue
        .quest(QUEST_SOURCE_COMMUNITY_JOURNAL, &QuestId::Int(2563))
        .expect("sticker 2563");
    assert_eq!(stew.title_locale_id, Some(5100479));
    assert_eq!(
        stew.title_locale.as_deref(),
        Some("Second Helping of Strongarm Stew")
    );
}

#[test]
fn local_stages_keep_original_bindings_explicit() {
    let catalogue = catalogue();
    assert_eq!(catalogue.local_binding(), (0, 17));
    for stage in catalogue.local_stages() {
        assert_eq!(
            stage.original_quest_id,
            None,
            "local stage {} must not claim an original binding",
            stage.id.display()
        );
    }
    let accepted = catalogue
        .local_stages()
        .iter()
        .filter(|stage| stage.implementation_state.as_deref() == Some("native-accepted"))
        .count();
    let code_only = catalogue
        .local_stages()
        .iter()
        .filter(|stage| stage.implementation_state.as_deref() == Some("code-only"))
        .count();
    assert_eq!((accepted, code_only), (12, 5));
    // Title similarity is recorded, never merged into an identity binding.
    let local = catalogue
        .quest(QUEST_SOURCE_LOCAL_OPENING, &QuestId::Int(910013))
        .expect("local stage 910013");
    let sticker = catalogue
        .quest(QUEST_SOURCE_COMMUNITY_JOURNAL, &QuestId::Int(2563))
        .expect("sticker 2563");
    assert_eq!(local.title_locale, sticker.title_locale);
    assert_ne!(local.source, sticker.source);
}

#[test]
fn validator_reports_gaps_without_fabricating_errors() {
    let report = catalogue().validate();
    assert!(
        report.errors.is_empty(),
        "unexpected errors: {:?}",
        report.errors
    );
    let joined = report.warnings.join("\n");
    for expected in [
        "opening-hub-main-quest 2514",
        "wiki-chain-link Quest:Eviction Notice",
        "opening-optional-sticker 2572",
        "disabled blocked claim: quest 1617",
        "shared quest id 2513",
        "unresolved wiki prerequisite \"Quest:Learn\"",
    ] {
        assert!(
            joined.contains(expected),
            "missing warning {expected:?} in {joined}"
        );
    }
}

#[test]
fn blocked_claim_1617_stays_disabled() {
    let catalogue = catalogue();
    assert_eq!(catalogue.blocked_claims.len(), 1);
    let claim = &catalogue.blocked_claims[0];
    assert_eq!(claim.quest_id, 1617);
    assert_eq!(claim.status, "disabled");
    assert_eq!(claim.objective.as_deref(), Some("Defeat Petty Yetis"));
    assert_eq!(claim.required, Some(20));
    // The engine definition keeps enemy_definition 0, so generic kills cannot
    // advance the quest.
    let definition = catalogue
        .engine_definitions
        .iter()
        .find(|definition| definition.id == 1617)
        .expect("engine definition 1617");
    assert!(definition.original);
    assert_eq!(definition.enemy_definitions, vec![0]);
}

#[test]
fn objective_and_reward_rows_aggregate_with_source_labels() {
    let catalogue = catalogue();
    let objective = catalogue
        .objective_rows()
        .into_iter()
        .find(|row| row.quest_id == "1617")
        .expect("supplied objective row");
    assert_eq!(objective.source, QUEST_SOURCE_SUPPLIED_JOURNAL);
    assert_eq!(objective.text.as_deref(), Some("Defeat Petty Yetis"));
    assert_eq!(objective.required, Some(20));

    let rewards = catalogue.reward_rows();
    assert!(rewards.iter().any(|row| {
        row.source == QUEST_SOURCE_SUPPLIED_JOURNAL
            && row.quest_id == "1617"
            && row.reward.item_definition == Some(1843)
            && row.reward.quantity == Some(1)
    }));
    assert!(rewards.iter().any(|row| {
        row.source == QUEST_SOURCE_LOCAL_OPENING
            && row.quest_id == "910013"
            && row.reward.kind == "coins"
            && row.reward.amount == Some(10)
    }));
}

#[test]
fn validator_rejects_a_fabricated_original_binding() {
    let text = std::fs::read_to_string(piece_066::catalogue_path()).expect("read catalogue");
    let mut value: serde_json::Value = serde_json::from_str(&text).expect("parse catalogue");
    let stages = value["quests"].as_array_mut().expect("quest rows");
    let stage = stages
        .iter_mut()
        .find(|row| row["source"] == QUEST_SOURCE_LOCAL_OPENING && row["id"] == 910001)
        .expect("local stage 910001");
    stage["original_quest_id"] = serde_json::Value::from(2513);
    let mutated = QuestCatalogue::from_json_str(&value.to_string()).expect("parse mutated");
    let report = mutated.validate();
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("910001") && error.contains("2513")),
        "errors: {:?}",
        report.errors
    );
    // The mutated row also disagrees with the coverage block, which is reported.
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("coverage.local_binding"))
    );
}

#[test]
fn validator_rejects_duplicate_quest_ids() {
    let text = std::fs::read_to_string(piece_066::catalogue_path()).expect("read catalogue");
    let mut value: serde_json::Value = serde_json::from_str(&text).expect("parse catalogue");
    let rows = value["quests"].as_array().expect("quest rows");
    let duplicate = rows
        .iter()
        .find(|row| row["source"] == QUEST_SOURCE_SUPPLIED_JOURNAL)
        .expect("supplied row")
        .clone();
    value["quests"]
        .as_array_mut()
        .expect("quest rows")
        .push(duplicate);
    let mutated = QuestCatalogue::from_json_str(&value.to_string()).expect("parse mutated");
    let report = mutated.validate();
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("duplicate")),
        "errors: {:?}",
        report.errors
    );
}

#[test]
fn prerequisite_links_expose_the_wiki_chain() {
    let catalogue = catalogue();
    let links = catalogue.prerequisite_links();
    assert!(links.iter().any(|link| {
        link.page == "Quest:Fighting off the Pack" && link.prerequisite == "Quest:Your"
    }));
    let npcs = catalogue.start_npcs();
    assert!(
        npcs.iter()
            .any(|(page, npc)| *page == "Quest:Fighting off the Pack"
                && npc.starts_with("Cleetus Farnum"))
    );
}
