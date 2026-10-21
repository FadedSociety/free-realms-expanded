//! Scratch split-content tree for the enabled quest catalogue audit.
//!
//! Test-only: the fixture is built under `CARGO_TARGET_TMPDIR` and removed on
//! drop, so the shipped tree is never touched. The audit loads it through the
//! production `QuestEngine::from_split_content` and `world::NpcVisibility::load`;
//! this module only writes the minimal authored rows those loaders need.

use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const FIXTURE_DEFECT: u32 = 9_100_002;
const FIXTURE_GIVER: u64 = 9_000_100_001;
const FIXTURE_TARGET: u64 = 9_000_100_002;
const FIXTURE_MISSING_GIVER: u64 = 9_000_100_099;
const FIXTURE_MISSING_PREREQUISITE: u32 = 4_000_100_098;
const FIXTURE_QUEST: u32 = 9_100_001;
const FIXTURE_BLOCKER: u32 = 9_100_003;
const FIXTURE_TITLE: u32 = 700_001;

#[derive(Clone, Copy)]
pub enum Defect {
    None,
    MissingGiver,
    EmptyReward,
    DisabledPrerequisite,
    MissingPrerequisite,
    WrappedTitle,
    /// A `visit_position` objective with the given transform, so the production
    /// split loader's geometry bound can be exercised directly.
    VisitGeometry {
        position: [f64; 3],
        radius: f64,
    },
}

pub struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("t314-{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("quests")).expect("fixture quests dir");
        fs::create_dir_all(dir.join("npcs")).expect("fixture npcs dir");
        Self(dir)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn fixture_source() -> Value {
    json!([{"kind": "reconstructed", "evidence": "(R) T314 scratch fixture"}])
}

fn fixture_quest(
    id: u32,
    title_string_id: u32,
    giver_npc_id: u64,
    objectives: Value,
    rewards: Value,
    prerequisite_quest_ids: Value,
) -> Value {
    json!({
        "id": id,
        "title_string_id": title_string_id,
        "giver_npc_id": giver_npc_id,
        "prerequisite_quest_ids": prerequisite_quest_ids,
        "objectives": objectives,
        "rewards": rewards,
        "source": fixture_source(),
    })
}

fn fixture_talk_objective() -> Value {
    json!([{
        "kind": "talk_to_npc",
        "id": 1,
        "npc_id": FIXTURE_TARGET,
        "source": fixture_source(),
    }])
}

fn fixture_visit_objective(position: [f64; 3], radius: f64) -> Value {
    json!([{
        "kind": "visit_position",
        "id": 1,
        "zone_id": 1,
        "position": position,
        "radius": radius,
        "source": fixture_source(),
    }])
}

fn fixture_reward() -> Value {
    json!([{"coins": 5, "items": [], "job_xp": {"4": 5}, "source": fixture_source()}])
}

fn fixture_npc(npc_id: u64, name: &str, name_string_id: u32, model_id: u32, x: f64) -> Value {
    json!({
        "npc_id": npc_id,
        "name": name,
        "name_string_id": name_string_id,
        "zone_id": 1,
        "position": [x, 0.0, 0.0],
        "placement": "legacy_data",
        "model_id": model_id,
        "roles": ["quest_giver"],
        "quest_ids_given": [],
        "quests_given": [],
        "quests_accepted": [],
    })
}

fn write_json(path: &Path, value: &Value) {
    fs::write(
        path,
        serde_json::to_vec_pretty(value).expect("fixture JSON"),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
}

pub fn fixture(name: &str, defect: Defect) -> Scratch {
    let scratch = Scratch::new(name);
    let root = scratch.path();
    let mut quests = vec![fixture_quest(
        FIXTURE_QUEST,
        FIXTURE_TITLE,
        FIXTURE_GIVER,
        fixture_talk_objective(),
        fixture_reward(),
        json!([]),
    )];
    let mut quest_ids = vec![FIXTURE_QUEST];
    let mut blocker = None;
    match defect {
        Defect::None => {}
        Defect::MissingGiver => {
            quests.push(fixture_quest(
                FIXTURE_DEFECT,
                FIXTURE_TITLE,
                FIXTURE_MISSING_GIVER,
                fixture_talk_objective(),
                fixture_reward(),
                json!([]),
            ));
            quest_ids.push(FIXTURE_DEFECT);
        }
        Defect::WrappedTitle => {
            quests.push(fixture_quest(
                FIXTURE_DEFECT,
                u32::MAX,
                FIXTURE_GIVER,
                fixture_talk_objective(),
                fixture_reward(),
                json!([]),
            ));
            quest_ids.push(FIXTURE_DEFECT);
        }
        Defect::EmptyReward => {
            quests.push(fixture_quest(
                FIXTURE_DEFECT,
                FIXTURE_TITLE,
                FIXTURE_GIVER,
                fixture_talk_objective(),
                json!([]),
                json!([]),
            ));
            quest_ids.push(FIXTURE_DEFECT);
        }
        Defect::DisabledPrerequisite => {
            // The defect quest would fail the reward gate if it were graded, so
            // an empty reward proves the audit classified it unreachable instead.
            quests.push(fixture_quest(
                FIXTURE_DEFECT,
                FIXTURE_TITLE,
                FIXTURE_GIVER,
                fixture_talk_objective(),
                json!([]),
                json!([FIXTURE_BLOCKER]),
            ));
            blocker = Some(json!({
                "id": FIXTURE_BLOCKER,
                "title_string_id": FIXTURE_TITLE,
                "giver_npc_id": FIXTURE_GIVER,
                "prerequisite_quest_ids": [],
                "objectives": [],
                "rewards": [],
                "source": fixture_source(),
                "enabled": false,
            }));
            quest_ids.extend([FIXTURE_DEFECT, FIXTURE_BLOCKER]);
        }
        Defect::MissingPrerequisite => {
            quests.push(fixture_quest(
                FIXTURE_DEFECT,
                FIXTURE_TITLE,
                FIXTURE_GIVER,
                fixture_talk_objective(),
                fixture_reward(),
                json!([FIXTURE_MISSING_PREREQUISITE]),
            ));
            quest_ids.push(FIXTURE_DEFECT);
        }
        Defect::VisitGeometry { position, radius } => {
            quests.push(fixture_quest(
                FIXTURE_DEFECT,
                FIXTURE_TITLE,
                FIXTURE_GIVER,
                fixture_visit_objective(position, radius),
                fixture_reward(),
                json!([]),
            ));
            quest_ids.push(FIXTURE_DEFECT);
        }
    }
    if let Some(blocker) = blocker {
        quests.push(blocker);
    }
    let jobs = json!({
        "schema_version": 1,
        "records": [{
            "id": "fixture_job",
            "name": "Fixture Job",
            "profile_id": 4,
            "category": "combat",
            "max_level": 20,
            "members_only": false,
            "trainer_npc_ids": [],
            "quest_ids": quest_ids,
            "abilities": [],
            "source": fixture_source(),
        }],
    });
    write_json(
        &root.join("zones.json"),
        &json!({"schema_version": 1, "records": [{"id": 1, "name": "Fixture Zone"}]}),
    );
    write_json(&root.join("npc_spawns_extra.json"), &json!({"records": []}));
    write_json(&root.join("jobs.json"), &jobs);
    write_json(
        &root.join("quests/fixture.json"),
        &json!({"schema_version": 1, "records": quests}),
    );
    write_json(
        &root.join("npcs/fixture.json"),
        &json!({"schema_version": 1, "records": [
            fixture_npc(FIXTURE_GIVER, "Fixture giver", 700_002, 101, 10.0),
            fixture_npc(FIXTURE_TARGET, "Fixture target", 700_003, 102, 12.0),
        ]}),
    );
    scratch
}
