//! One synthetic, already-accepted "collect `quantity` of `item`" quest.
//!
//! F110 wrote this setup in `tests/job_trainers.rs`; T794 copied it into
//! `tests/common/vendor_fixture.rs`. F803 keeps the one copy and both callers delegate here.
//! `#[path]`-included from `tests/common/vendor_fixture.rs` (also by the gateway binary's L011
//! test) and from `tests/job_trainers.rs`, so it uses the `osfr_tools` crate path rather than
//! `crate::`, and it writes only under the caller's content directory.

use osfr_tools::quest_engine::{CharacterEligibility, QuestEngine, QuestState};
use serde_json::json;
use std::{fs, path::Path};

/// Add a `jobs.json` row, the quest and (when `npc_row`) a minimal giver NPC row to `content`,
/// then offer and accept the quest through the production engine. `evidence` labels the generated
/// rows; `guid` is the character the returned save is built for. The returned save holds the
/// quest active: callers put it in their `GameplayState::npc_quests`.
pub fn install(
    content: &Path,
    quest: u32,
    giver: u64,
    item: i32,
    quantity: u32,
    guid: u64,
    npc_row: bool,
    evidence: &str,
) -> QuestState {
    let source = json!([{"kind": "reconstructed", "evidence": evidence}]);
    let write = |path: &str, document: serde_json::Value| {
        fs::write(
            content.join(path),
            serde_json::to_vec(&document).expect("fixture quest content serializes"),
        )
        .expect("fixture quest content writes");
    };
    fs::create_dir_all(content.join("quests")).expect("fixture quests dir creates");
    write(
        &format!("quests/collect_{quest}.json"),
        json!({"schema_version": 1, "records": [{
            "id": quest,
            "title_string_id": 1,
            "giver_npc_id": giver,
            "prerequisite_quest_ids": [],
            "objectives": [{
                "kind": "collect_item", "id": 1, "item_id": item,
                "quantity": quantity, "source": source.clone()
            }],
            "rewards": [],
            "source": source.clone()
        }]}),
    );
    write(
        "jobs.json",
        json!({"schema_version": 1, "records": [{
            "id": format!("synthetic_{quest}"), "name": "Synthetic", "profile_id": null,
            "category": "freestyle", "max_level": 20, "members_only": false,
            "trainer_npc_ids": [], "quest_ids": [quest], "abilities": [], "source": source
        }]}),
    );
    if npc_row {
        fs::create_dir_all(content.join("npcs")).expect("fixture npcs dir creates");
        write(
            &format!("npcs/collect_{giver}.json"),
            json!({"schema_version": 1, "records": [{
                "npc_id": giver, "quest_ids_given": [quest]
            }]}),
        );
    }
    let mut engine = QuestEngine::from_split_content(
        content,
        guid,
        CharacterEligibility {
            player_level: 1,
            member: true,
            member_quest_limit: 10,
            ..CharacterEligibility::default()
        },
        None,
    )
    .expect("fixture quest catalogue loads");
    assert!(engine.offer(quest, giver).changed);
    assert!(engine.accept(quest).changed);
    engine.state().clone()
}
