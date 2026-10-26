//! T316: Merry Vale and Stillwater Crossing drawable NPC rows carry recovered
//! locale greetings.
//!
//! Content only: `NpcInteraction::dialogue_context` already copies
//! `greeting_string_id` into `DialogueContext.body_string_id`, and
//! `gameplay/npc.rs::dialogue_packet_with` writes that body into 26/3. The
//! clicked row's own id is the body, never the nearest locale line.
//!
//! Client evidence re-checked with IDA for T316:
//! - 26/3 handler `0x00aa12e0` calls payload reader `0x00a9ef10` and then
//!   display routine `0x009f5ae0` (ShowDialogFromPacket).
//! - Reader `0x00a9ef10` stores the first payload dword at object `+16`
//!   (`*(_DWORD *)(this + 16) = *v3`) and advances the cursor by 4.
//! - Display `0x009f5ae0` passes `*(_DWORD *)(a2 + 16)` to the `Global.Text`
//!   lookup `0x0097c1e0` immediately before `:SetDialogText`.
//! The body is therefore the first dword after the four-byte family/subtype
//! header, matching `encode_dialogue_26_3`'s `initial_dwords[0]`. The byte
//! layout is pinned by
//! `tests/npc_interaction.rs::dialogue_26_3_synthetic_fixture_round_trips_exact_native_field_order`.
//! Locale ids, hashes, dat lines and speaker proofs are in
//! `notes/Autopilot/T316.md`.
//!
//! The service path (`NpcInteraction::interact` -> `select`) is the same one
//! `gameplay/npc.rs` uses; the dispatch path (`Gameplay::handle_in_world`)
//! drives the real router and `dialogue_packet_with`. The packet builders, the
//! locale line convention and the 26/3 body offset live in the shared
//! `tests/common/npc_greetings.rs` harness; this suite keeps only its own
//! content pins and the one-shot zero-greeting fixture.

#[path = "common/npc_greetings.rs"]
mod npc_greetings;

use npc_greetings::{
    assert_drawable, content_dir, gameplay_dir, gameplay_talk_body, greeting_bindings, locale_rows,
    npc_record, select, service, service_talk_body,
};
use osfr_tools::{
    gameplay::Gameplay,
    npc_interaction::{NpcAction, NpcInteraction, NpcInteractionError, NpcSelection},
    world::NpcVisibility,
    world_text::locale_hash,
};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

const MAX_GREETINGS: usize = 24;

const WARRIOR_GUIDE: &str = "notes/Build Guides/Content/Jobs/Warrior.md";
const POSTMAN_GUIDE: &str = "notes/Build Guides/Content/Jobs/Postman.md";
const CHEF_GUIDE: &str = "notes/Build Guides/Content/Jobs/Chef.md";
const WIZARD_GUIDE: &str = "notes/Build Guides/Content/Jobs/Wizard.md";
const BRAWLER_GUIDE: &str = "notes/Build Guides/Content/Jobs/Brawler.md";
const FISHERMAN_GUIDE: &str = "notes/Build Guides/Content/Jobs/Fisherman.md";
const MERRY_VALE_GUIDE: &str = "notes/Build Guides/Content/NPCs/NPCs - Merry Vale.md";
const STILLWATER_GUIDE: &str = "notes/Build Guides/Content/NPCs/NPCs - Stillwater Crossing.md";

/// The two NPC files this slice owns.
const OWNED_FILES: [&str; 2] = ["merry_vale.json", "stillwater_crossing.json"];
/// Repository-relative NPC directory; the changed-path guard works on it.
const NPC_CONTENT: &str = "prototype/osfr-tools/data/content/npcs";

/// One edited row: content key, the `Global.Text` id the client looks up, its
/// JenkinsLookup2 hash, the physical stock-locale line, the speaker proof and
/// the 49/1 quest intro when the line is that quest's offer speech.
struct Greeting {
    npc_id: u64,
    file: &'static str,
    greeting: u32,
    hash: u32,
    dat_line: u32,
    proof_guide: &'static str,
    proof_line: u32,
    region_line: u32,
    quest: Option<(&'static str, u32)>,
}

const NEW_GREETINGS: [Greeting; 17] = [
    Greeting {
        npc_id: 9000000222,
        file: "merry_vale.json",
        greeting: 71035,
        hash: 2377109724,
        dat_line: 62495,
        proof_guide: WARRIOR_GUIDE,
        proof_line: 94,
        region_line: 45,
        quest: Some(("warrior.json", 3150008)),
    },
    Greeting {
        npc_id: 9000000030,
        file: "merry_vale.json",
        greeting: 71067,
        hash: 2242122283,
        dat_line: 59014,
        proof_guide: WARRIOR_GUIDE,
        proof_line: 94,
        region_line: 46,
        quest: None,
    },
    Greeting {
        npc_id: 9000000174,
        file: "merry_vale.json",
        greeting: 117035,
        hash: 1942512514,
        dat_line: 51179,
        proof_guide: POSTMAN_GUIDE,
        proof_line: 88,
        region_line: 43,
        quest: Some(("postman.json", 3130012)),
    },
    Greeting {
        npc_id: 9500000371,
        file: "merry_vale.json",
        greeting: 427481,
        hash: 13348600,
        dat_line: 337,
        proof_guide: MERRY_VALE_GUIDE,
        proof_line: 115,
        region_line: 115,
        quest: None,
    },
    Greeting {
        npc_id: 9000000044,
        file: "merry_vale.json",
        greeting: 71783,
        hash: 217741357,
        dat_line: 5717,
        proof_guide: CHEF_GUIDE,
        proof_line: 118,
        region_line: 41,
        quest: Some(("chef.json", 3050022)),
    },
    Greeting {
        npc_id: 9500000417,
        file: "merry_vale.json",
        greeting: 387081,
        hash: 2145785937,
        dat_line: 56498,
        proof_guide: MERRY_VALE_GUIDE,
        proof_line: 90,
        region_line: 90,
        quest: None,
    },
    Greeting {
        npc_id: 9500000420,
        file: "merry_vale.json",
        greeting: 41743,
        hash: 3010630538,
        dat_line: 79507,
        proof_guide: MERRY_VALE_GUIDE,
        proof_line: 72,
        region_line: 72,
        quest: None,
    },
    Greeting {
        npc_id: 9500000424,
        file: "merry_vale.json",
        greeting: 441732,
        hash: 3419032289,
        dat_line: 90415,
        proof_guide: MERRY_VALE_GUIDE,
        proof_line: 196,
        region_line: 100,
        quest: None,
    },
    Greeting {
        npc_id: 9500000427,
        file: "merry_vale.json",
        greeting: 141684,
        hash: 1290681492,
        dat_line: 34188,
        proof_guide: WIZARD_GUIDE,
        proof_line: 92,
        region_line: 44,
        quest: Some(("wizard.json", 3160014)),
    },
    Greeting {
        npc_id: 9500000400,
        file: "merry_vale.json",
        greeting: 73922,
        hash: 1124293448,
        dat_line: 29803,
        proof_guide: MERRY_VALE_GUIDE,
        proof_line: 32,
        region_line: 32,
        quest: None,
    },
    Greeting {
        npc_id: 9500000381,
        file: "merry_vale.json",
        greeting: 387043,
        hash: 2082414646,
        dat_line: 54875,
        proof_guide: MERRY_VALE_GUIDE,
        proof_line: 76,
        region_line: 76,
        quest: None,
    },
    Greeting {
        npc_id: 9000000038,
        file: "stillwater_crossing.json",
        greeting: 116793,
        hash: 1220502472,
        dat_line: 32325,
        proof_guide: STILLWATER_GUIDE,
        proof_line: 44,
        region_line: 44,
        quest: None,
    },
    Greeting {
        npc_id: 9500000785,
        file: "stillwater_crossing.json",
        greeting: 388502,
        hash: 3566246703,
        dat_line: 94453,
        proof_guide: BRAWLER_GUIDE,
        proof_line: 125,
        region_line: 49,
        quest: Some(("brawler.json", 3030022)),
    },
    Greeting {
        npc_id: 9000000196,
        file: "stillwater_crossing.json",
        greeting: 116508,
        hash: 1942927184,
        dat_line: 51192,
        proof_guide: BRAWLER_GUIDE,
        proof_line: 113,
        region_line: 50,
        quest: Some(("brawler.json", 3030017)),
    },
    Greeting {
        npc_id: 9000000103,
        file: "stillwater_crossing.json",
        greeting: 116777,
        hash: 3160199473,
        dat_line: 83418,
        proof_guide: STILLWATER_GUIDE,
        proof_line: 45,
        region_line: 45,
        quest: None,
    },
    Greeting {
        npc_id: 9000000096,
        file: "stillwater_crossing.json",
        greeting: 116496,
        hash: 724072415,
        dat_line: 19001,
        proof_guide: CHEF_GUIDE,
        proof_line: 103,
        region_line: 46,
        quest: Some(("chef.json", 3050017)),
    },
    Greeting {
        npc_id: 9000000181,
        file: "stillwater_crossing.json",
        greeting: 408272,
        hash: 2663118808,
        dat_line: 70112,
        proof_guide: FISHERMAN_GUIDE,
        proof_line: 32,
        region_line: 47,
        quest: None,
    },
];

/// The two bindings that already existed in the two owned files (W126). The
/// other ninety-two pre-existing bindings are pinned by the suites that own
/// their regions; this suite proves their rows were not disturbed through the
/// non-target-row comparison below.
const OWNED_PRE_EXISTING: [(&str, u64, u32); 2] = [
    ("merry_vale.json", 9500000430, 41268),
    ("stillwater_crossing.json", 9500000786, 116463),
];

/// Drawable rows with no recovered first-person speaker line, one per owned
/// file. They stay blank and must not inherit another row's body. T381 bound
/// Mayor McFairy (9500000405), so the Merry Vale control moved to Madison.
const BLANK_MERRY_VALE: u64 = 9500000404;
const BLANK_STILLWATER: u64 = 9000000227;

/// A bound row carried into the zero-greeting fixture as the control: it must
/// keep sending its own id so the fixture is not a bypass.
const CONTROL_ROW: u64 = 9500000371;

fn region_guide(file: &str) -> &'static str {
    match file {
        "merry_vale.json" => MERRY_VALE_GUIDE,
        "stillwater_crossing.json" => STILLWATER_GUIDE,
        other => panic!("not an owned region file: {other}"),
    }
}

/// Repository root: the manifest directory is `prototype/osfr-tools`.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn git(args: &[&str]) -> std::process::Output {
    Command::new("git")
        .arg("-C")
        .arg(repo_root())
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("git is required for the changed-content gate: {error}"))
}

fn git_text(args: &[&str]) -> String {
    let output = git(args);
    assert!(
        output.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn is_owned(path: &str) -> bool {
    OWNED_FILES
        .iter()
        .any(|file| path == format!("{NPC_CONTENT}/{file}"))
}

/// Repository-relative `npcs/*.json` paths this worktree has uncommitted
/// changes for (tracked or untracked). A builder workspace is private, so any
/// such path outside the two owned files is this slice's spill.
fn uncommitted_npc_paths() -> BTreeSet<String> {
    git_text(&["status", "--porcelain", "--", NPC_CONTENT])
        .lines()
        .filter(|line| line.len() > 3)
        .map(|line| {
            line[3..]
                .split(" -> ")
                .last()
                .unwrap_or_default()
                .to_owned()
        })
        .collect()
}

/// `npcs/*.json` paths the HEAD commit itself changed.
fn head_npc_paths() -> BTreeSet<String> {
    git_text(&[
        "diff-tree",
        "--no-commit-id",
        "--name-only",
        "-r",
        "HEAD",
        "--",
        NPC_CONTENT,
    ])
    .lines()
    .map(str::to_owned)
    .collect()
}

/// Where the change under test is read from.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ChangeBase {
    /// A builder workspace: the working tree against HEAD.
    WorkingTree,
    /// A committed review state: the HEAD commit against its parent. Comparing
    /// the commit's own diff, not HEAD against an identical working tree, is what
    /// makes this guard work after the integrator commits.
    Commit,
}

/// The NPC paths and revision pair of the change that must be checked: the
/// uncommitted NPC edit in a builder workspace, or the HEAD commit when it
/// touched one of the two owned files (the committed review state). A tip whose
/// change does not touch the owned files has no content change for this suite to
/// check, and each other region is pinned by the suite that owns it.
fn change_under_test() -> Option<(ChangeBase, BTreeSet<String>)> {
    let uncommitted = uncommitted_npc_paths();
    if !uncommitted.is_empty() {
        return Some((ChangeBase::WorkingTree, uncommitted));
    }
    let head = head_npc_paths();
    head.iter()
        .any(|path| is_owned(path))
        .then_some((ChangeBase::Commit, head))
}

/// One revision of `path` in the repository, or `None` when git or JSON fails.
fn document_at(revision: &str, path: &str) -> Option<Value> {
    let output = git(&["show", &format!("{revision}:{path}")]);
    if !output.status.success() {
        return None;
    }
    serde_json::from_slice(&output.stdout).ok()
}

/// The working-tree version of `path`, or `None` when it is missing or not JSON.
fn working_document(path: &str) -> Option<Value> {
    serde_json::from_str(&fs::read_to_string(repo_root().join(path)).ok()?).ok()
}

/// The fields a quest-binding slice may rewrite on a row it does not own, plus
/// the `field_sources` evidence that moves with each field.
const QUEST_BINDING_FIELDS: [&str; 4] = [
    "quest_ids_given",
    "quests_accepted",
    "quests_accepted_reason",
    "roles",
];

/// T502's `legacy_data` conversion of the existing Merk row: his spawn fields and
/// the diagnostics that explain them. The zone-quest suite
/// (`zone_quests_wugachug_briarwood::aiding_our_allies_ends_at_merk`) pins the
/// resulting model, name and drawable row, so this guard allows that one row's
/// conversion and still rejects every other foreign change.
const MERK_LEGACY_FIELDS: [&str; 12] = [
    "heading",
    "model_file_name",
    "model_id",
    "name_string_id",
    "placement",
    "placement_candidates",
    "placement_confidence",
    "placement_source",
    "position",
    "position_reason",
    "rotation",
    "source",
];

/// `row` with `fields` and their `field_sources` evidence removed.
fn without_fields(row: &Value, fields: &[&str]) -> Value {
    let mut row = row.clone();
    let Some(object) = row.as_object_mut() else {
        return row;
    };
    for field in fields {
        object.remove(*field);
    }
    if let Some(sources) = object
        .get_mut("field_sources")
        .and_then(Value::as_object_mut)
    {
        for field in fields {
            sources.remove(*field);
        }
    }
    row
}

/// True when `new` differs from `old` only by rows `allowed` accepts: the same
/// top-level metadata and the HEAD rows in the same order. `appended_ok` keeps
/// the greeting allowance that another slice may append a row; a quest-binding
/// allowance never accepts an unreviewed new row.
fn changed_rows_are(
    old: &Value,
    new: &Value,
    appended_ok: bool,
    allowed: impl Fn(&Value, &Value) -> bool,
) -> bool {
    let (Some(old_records), Some(new_records)) =
        (old["records"].as_array(), new["records"].as_array())
    else {
        return false;
    };
    if new_records.len() < old_records.len()
        || (!appended_ok && new_records.len() != old_records.len())
    {
        return false;
    }
    let mut old_document = old.clone();
    let mut new_document = new.clone();
    for document in [&mut old_document, &mut new_document] {
        let Some(object) = document.as_object_mut() else {
            return false;
        };
        object.remove("records");
    }
    if old_document != new_document {
        return false;
    }
    old_records
        .iter()
        .zip(new_records)
        .all(|(old, new)| old["npc_id"] == new["npc_id"] && (old == new || allowed(old, new)))
}

/// True when `new` differs from `old` only by greeting bindings and appended
/// rows: the HEAD records in the same order, each unchanged apart from
/// `greeting_string_id` and its `field_sources` entry, then any new rows, and
/// the same top-level metadata. A builder workspace may carry another slice's
/// uncommitted binding or appended row (T379's repair guys); that is that
/// slice's change, pinned by its own suite, not this suite's spill.
fn is_greeting_only_change_between(old: &Value, new: &Value) -> bool {
    changed_rows_are(old, new, true, |old, new| {
        without_fields(old, &["greeting_string_id"]) == without_fields(new, &["greeting_string_id"])
    })
}

/// T698's provenance marker: only rows a name-matched reference spawn placed
/// carry it, so a foreign NPC file may carry exactly those rows' placement change.
const REFERENCE_SPAWN_MARKER: &str = "matched by the spawn's client name (talk_spawns.py) (R)";

/// The fields a follow-up may bind on a row T698 already placed. The rest of the row
/// (position, model, greeting, quests) must stay byte-for-byte equal.
const NAME_BINDING_FIELDS: [&str; 2] = ["name_string_id", "name_string_id_reason"];

/// T698 (`data/content/npcs/*.json`): either the row's own `poi_anchor`/`unanchored`
/// -> `legacy_data` placement change, or a later name-only binding of the spawn `NameId`
/// on a row T698 already placed. In both cases the row's identity, zone, quest and
/// greeting fields stay put; the placement values are pinned by
/// `content_npc_placement.rs`, `content_npc_spawns.rs` and
/// `npc_reference_spawn_placements.rs`.
fn is_reference_spawn_row(old: &Value, new: &Value) -> bool {
    let marked = new["position_reason"]
        .as_str()
        .is_some_and(|reason| reason.contains(REFERENCE_SPAWN_MARKER));
    if !marked || old["greeting_string_id"] != new["greeting_string_id"] {
        return false;
    }
    let newly_placed = matches!(old["placement"].as_str(), Some("poi_anchor" | "unanchored"))
        && new["placement"] == "legacy_data"
        && new["placement_source"] == "legacy_data_spawn"
        && new["placement_confidence"] == "medium"
        && old["npc_id"] == new["npc_id"]
        && old["name"] == new["name"]
        && old["zone"] == new["zone"]
        && old["zone_id"] == new["zone_id"]
        && old["roles"] == new["roles"]
        && old["quest_ids_given"] == new["quest_ids_given"]
        && old["quests_accepted"] == new["quests_accepted"];
    newly_placed
        || (old["placement"] == "legacy_data"
            && old["placement_source"] == "legacy_data_spawn"
            && without_fields(old, &NAME_BINDING_FIELDS)
                == without_fields(new, &NAME_BINDING_FIELDS))
}

/// True when `new` differs from `old` only by quest bindings and their
/// provenance on the HEAD rows, plus T502's `legacy_data` conversion of Merk
/// and T698's reference-spawn placements. The quest suites pin those fields; a
/// greeting, a moved row or any other foreign content change does not pass.
fn is_quest_binding_change_between(path: &str, old: &Value, new: &Value) -> bool {
    changed_rows_are(old, new, false, |old, new| {
        let quest_old = without_fields(old, &QUEST_BINDING_FIELDS);
        let quest_new = without_fields(new, &QUEST_BINDING_FIELDS);
        if quest_old == quest_new || is_reference_spawn_row(old, new) {
            return true;
        }
        path.ends_with("/shrouded_glade.json")
            && old["npc_id"].as_u64() == Some(9_500_000_690)
            && old["placement"].as_str() != Some("legacy_data")
            && new["placement"].as_str() == Some("legacy_data")
            && without_fields(&quest_old, &MERK_LEGACY_FIELDS)
                == without_fields(&quest_new, &MERK_LEGACY_FIELDS)
    })
}

/// A throwaway copy of the shipped gameplay content plus a one-row-region NPC
/// fixture: the blank control with an explicit `greeting_string_id: 0` and one
/// bound control row. Removed when the test ends.
struct FixtureDir(PathBuf);

impl Drop for FixtureDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn zero_greeting_fixture() -> FixtureDir {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after the epoch")
        .as_nanos();
    let target =
        std::env::temp_dir().join(format!("osfr-t316-zero-{}-{stamp}", std::process::id()));
    fs::create_dir_all(target.join("npcs")).expect("fixture npcs directory");
    for entry in fs::read_dir(gameplay_dir()).expect("gameplay content directory") {
        let entry = entry.expect("gameplay content entry");
        if entry.path().is_file() {
            fs::copy(entry.path(), target.join(entry.file_name()))
                .expect("copied gameplay content");
        }
    }
    // The Coin Shop validates the full store schema, so the real catalogue is
    // copied; the fixture sells nothing, and `NpcInteraction::load` requires
    // the empty vendor list next to `npcs/`.
    fs::copy(
        content_dir().join("stores.json"),
        target.join("stores.json"),
    )
    .expect("copied store catalogue");
    fs::write(
        target.join("vendors.json"),
        br#"{"schema_version":1,"vendors":[]}"#,
    )
    .expect("fixture vendors");
    fs::write(
        target.join("zones.json"),
        br#"{"schema_version":1,"records":[]}"#,
    )
    .expect("fixture zones");
    fs::write(
        target.join("npc_spawns_extra.json"),
        br#"{"schema_version":1,"records":[]}"#,
    )
    .expect("fixture extra spawns");
    let mut record = npc_record("merry_vale.json", BLANK_MERRY_VALE);
    record["greeting_string_id"] = Value::from(0);
    let region = serde_json::json!({
        "schema_version": 1,
        "region_page": "Merry Vale",
        "region_zone_id": 9,
        "records": [record, npc_record("merry_vale.json", CONTROL_ROW)],
    });
    fs::write(
        target.join("npcs/merry_vale.json"),
        serde_json::to_vec(&region).expect("fixture NPC region"),
    )
    .expect("fixture NPC rows");
    FixtureDir(target)
}

/// Every edited row: one client source and one `(R)` policy source, a drawable
/// spawned row, and a locale id that resolves on its cited physical line.
/// `edited.len() <= 24` is the cap; the note lists the 37 drawable rows left
/// blank.
#[test]
fn every_edited_row_has_resolved_client_provenance() {
    assert!(
        NEW_GREETINGS.len() <= MAX_GREETINGS,
        "the slice binds at most {MAX_GREETINGS} rows"
    );
    let wanted: BTreeSet<u32> = NEW_GREETINGS.iter().map(|target| target.hash).collect();
    let locale = locale_rows(&wanted);
    let visibility =
        NpcVisibility::load(&content_dir()).expect("the production visibility loader accepts it");
    let mut greeting_ids = BTreeSet::new();
    let mut targets = BTreeSet::new();
    for target in &NEW_GREETINGS {
        assert!(
            target.greeting > 0,
            "NPC {}: greeting must be positive",
            target.npc_id
        );
        assert!(
            greeting_ids.insert(target.greeting),
            "NPC {}: greeting {} reused",
            target.npc_id,
            target.greeting
        );
        assert!(
            targets.insert((target.file.to_owned(), target.npc_id)),
            "NPC {}: duplicate target row",
            target.npc_id
        );

        let record = npc_record(target.file, target.npc_id);
        assert_eq!(
            record["greeting_string_id"].as_u64(),
            Some(u64::from(target.greeting)),
            "NPC {}: greeting binding",
            target.npc_id
        );
        assert_drawable(&record, target.npc_id);
        assert_eq!(
            visibility.content_key(target.npc_id),
            Some(target.npc_id),
            "NPC {} is not spawned by NpcVisibility",
            target.npc_id
        );

        let sources = record["field_sources"]["greeting_string_id"]
            .as_array()
            .unwrap_or_else(|| panic!("NPC {}: greeting sources", target.npc_id));
        assert_eq!(
            sources.len(),
            2,
            "NPC {}: one client and one reconstructed source",
            target.npc_id
        );
        let client = sources
            .iter()
            .find(|source| source["kind"] == "client")
            .unwrap_or_else(|| panic!("NPC {}: client source", target.npc_id));
        let reconstructed = sources
            .iter()
            .find(|source| source["kind"] == "reconstructed")
            .unwrap_or_else(|| panic!("NPC {}: reconstructed source", target.npc_id));
        assert!(
            reconstructed["evidence"].as_str().is_some_and(|evidence| {
                evidence.starts_with("(R)") && evidence.contains("fixed default Talk greeting")
            }),
            "NPC {}: the reconstruction must be labelled (R) fixed-default policy",
            target.npc_id
        );
        let evidence = client["evidence"].as_str().unwrap_or_default();
        for expected in [
            format!("Global.Text.{}", target.greeting),
            format!("JenkinsLookup2 hash {}", target.hash),
            format!("line {}", target.dat_line),
            format!("{} line {}", target.proof_guide, target.proof_line),
            format!("{} line {}", region_guide(target.file), target.region_line),
        ] {
            assert!(
                evidence.contains(&expected),
                "NPC {}: source must name {expected:?}",
                target.npc_id
            );
        }
        if let Some((quest_file, quest_id)) = target.quest {
            assert!(
                evidence.contains(&format!(
                    "data/content/quests/{quest_file} quest {quest_id}"
                )),
                "NPC {}: source must name its quest record",
                target.npc_id
            );
        }

        // Resolve the id the way the client does: hash Global.Text.<id>, then
        // find that hash on the cited physical locale line.
        assert_eq!(
            locale_hash(i32::try_from(target.greeting).expect("stock locale id fits i32")),
            target.hash,
            "NPC {}: greeting id must be a Global.Text key, not a raw hash",
            target.npc_id
        );
        let (line, text) = locale
            .get(&target.hash)
            .unwrap_or_else(|| panic!("NPC {}: locale hash missing", target.npc_id));
        assert_eq!(
            *line, target.dat_line as usize,
            "NPC {}: en_us_data.dat line",
            target.npc_id
        );
        assert!(
            !text.trim().is_empty(),
            "NPC {}: empty locale text",
            target.npc_id
        );
    }

    // Every other binding in `npcs/*.json` must stay disjoint from these ids;
    // this is the cross-file check the exact-set comparison could not make.
    let pre_existing: BTreeSet<u32> = greeting_bindings()
        .into_iter()
        .filter(|((file, npc_id), _)| !targets.contains(&(file.clone(), *npc_id)))
        .map(|(_, greeting)| greeting)
        .collect();
    assert!(!pre_existing.is_empty(), "the NPC content has greetings");
    assert!(
        greeting_ids.is_disjoint(&pre_existing),
        "a new greeting reuses an existing binding"
    );
}

#[test]
fn merry_vale_and_stillwater_rows_send_their_greeting_in_26_3() {
    // Service path: the context the router copies into the 26/3 body.
    let mut interaction = service();
    for target in &NEW_GREETINGS {
        assert_eq!(
            service_talk_body(&mut interaction, target.npc_id),
            Some(target.greeting),
            "NPC {}: service Talk body",
            target.npc_id
        );
    }

    // Dispatch path: gateway -> Gameplay -> gameplay/npc.rs -> dialogue_packet_with.
    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    for target in &NEW_GREETINGS {
        assert_eq!(
            gameplay_talk_body(&mut game, target.file, target.npc_id),
            target.greeting,
            "NPC {}: in-world 26/3 body",
            target.npc_id
        );
    }

    // A reconnect rebuilds the same static content; the body is not session state.
    let first = &NEW_GREETINGS[0];
    assert_eq!(
        service_talk_body(&mut service(), first.npc_id),
        Some(first.greeting),
        "NPC {}: a fresh loader sends the same body",
        first.npc_id
    );
}

#[test]
fn a_zero_or_missing_greeting_falls_back_to_no_body() {
    let mut interaction = service();
    let blanks = [
        ("merry_vale.json", BLANK_MERRY_VALE),
        ("stillwater_crossing.json", BLANK_STILLWATER),
    ];
    for (file, npc_id) in blanks {
        let record = npc_record(file, npc_id);
        assert!(
            record.get("greeting_string_id").is_none()
                || record["greeting_string_id"].as_u64() == Some(0),
            "NPC {npc_id}: the blank control must stay blank"
        );
        assert_drawable(&record, npc_id);
        assert_eq!(
            service_talk_body(&mut interaction, npc_id),
            None,
            "NPC {npc_id}: an NPC without a greeting has no body"
        );

        // A refused choice must leave the menu usable (F271): select Talk
        // directly on the still-open menu, with no second `interact`, so this
        // fails if `select` consumes the open menu on `InvalidOption`.
        let (zone, position) = interaction.location(npc_id).unwrap();
        let menu = interaction.interact(npc_id, zone, position, None).unwrap();
        let talk = menu
            .options
            .iter()
            .find(|option| option.action == NpcAction::Talk)
            .expect("the blank NPC offers Talk")
            .id;
        assert!(matches!(
            interaction.select(&select(npc_id, i32::MAX as u32), zone, position, None),
            Err(NpcInteractionError::InvalidOption)
        ));
        match interaction.select(&select(npc_id, talk as u32), zone, position, None) {
            Ok(NpcSelection::Talk(context)) => assert_eq!(
                context.body_string_id, None,
                "the refused choice must not consume the menu or inject a body"
            ),
            _ => panic!("the refused choice must leave the menu usable"),
        }
        // An unknown NPC id resolves to no row at all; a client cannot choose a body.
        assert!(matches!(
            interaction.interact(123, zone, position, None),
            Err(NpcInteractionError::UnknownNpc)
        ));
    }
    assert!(
        NEW_GREETINGS.iter().all(|target| target.greeting != 0),
        "zero is not a recovered greeting id"
    );

    // The real router still encodes 26/3 with a zero body and no borrowed id.
    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    for (file, npc_id) in blanks {
        assert_eq!(
            gameplay_talk_body(&mut game, file, npc_id),
            0,
            "NPC {npc_id}: no invented locale id"
        );
    }

    // An explicit `greeting_string_id: 0` is the `Some(0)` case, not the missing
    // block. The fixture keeps the blank control and one bound control row so a
    // fallback to another row's id would show up as the wrong body.
    let fixture = zero_greeting_fixture();
    let mut fixture_interaction =
        NpcInteraction::load(&fixture.0.join("npcs"), &fixture.0.join("stores.json"))
            .expect("the zero-greeting fixture loads through the real NPC loader");
    let control = NEW_GREETINGS
        .iter()
        .find(|target| target.npc_id == CONTROL_ROW)
        .expect("the bound control row is one of the edited rows");
    assert_eq!(
        service_talk_body(&mut fixture_interaction, BLANK_MERRY_VALE),
        Some(0),
        "an explicit zero greeting is carried as Some(0), not dropped or borrowed"
    );
    assert_eq!(
        service_talk_body(&mut fixture_interaction, CONTROL_ROW),
        Some(control.greeting),
        "the fixture still sends a bound row's own id"
    );
    let mut fixture_game =
        Gameplay::load(&fixture.0, None).expect("the fixture gameplay content loads");
    assert_eq!(
        gameplay_talk_body(&mut fixture_game, "merry_vale.json", BLANK_MERRY_VALE),
        0,
        "an explicit zero greeting encodes a zero 26/3 body"
    );
    assert_eq!(
        gameplay_talk_body(&mut fixture_game, "merry_vale.json", CONTROL_ROW),
        control.greeting,
        "the fixture's bound control row still sends its own 26/3 body"
    );
}

#[test]
fn the_existing_greetings_and_other_regions_are_unchanged() {
    for (file, npc_id, greeting) in OWNED_PRE_EXISTING {
        assert_eq!(
            npc_record(file, npc_id)["greeting_string_id"].as_u64(),
            Some(u64::from(greeting)),
            "existing binding {npc_id} in {file} changed"
        );
    }

    // The change under test (the uncommitted NPC edit in a builder workspace,
    // or the HEAD commit against its parent) may change only the two owned NPC
    // files. A foreign file may carry a quest-binding slice's fields (T502's
    // change, pinned by the quest suites) and, while uncommitted, another
    // slice's greeting-only binding or appended row. The complete greeting set
    // of the two regions with no greeting suite is pinned by
    // `npc_greetings_shrouded_glade_wugachug`; any other edit outside the two
    // owned files fails.
    let Some((base, changed)) = change_under_test() else {
        return;
    };
    for path in &changed {
        if is_owned(path) {
            continue;
        }
        let (old, new) = match base {
            ChangeBase::WorkingTree => (document_at("HEAD", path), working_document(path)),
            ChangeBase::Commit => (document_at("HEAD^", path), document_at("HEAD", path)),
        };
        let (Some(old), Some(new)) = (old, new) else {
            panic!("{path} changed: the change under test is not readable JSON");
        };
        assert!(
            is_quest_binding_change_between(path, &old, &new)
                || (base == ChangeBase::WorkingTree && is_greeting_only_change_between(&old, &new)),
            "{path} changed: this slice owns only merry_vale.json and stillwater_crossing.json"
        );
    }
}

#[test]
fn the_change_guard_allows_quest_bindings_and_rejects_a_foreign_greeting() {
    // A small NPC region behind the two T502 changes the guard must allow: a
    // quest binding on an existing row, and Merk's `poi_anchor` -> `legacy_data`
    // conversion. The top level and row ids are the guard's own inputs.
    let old = serde_json::json!({
        "schema_version": 1,
        "region_page": "Briarwood",
        "region_zone_id": 2,
        "records": [
            {
                "npc_id": 9500000213_u64,
                "name": "Smoggo",
                "roles": ["ambient"],
                "quest_ids_given": [],
                "quests_accepted": [],
                "greeting_string_id": 0,
                "field_sources": {
                    "roles": [{"kind": "wiki", "evidence": "old"}],
                    "greeting_string_id": [{"kind": "client", "evidence": "old"}]
                }
            },
            {
                "npc_id": 9500000690_u64,
                "name": "Headmaster Merk",
                "placement": "poi_anchor",
                "position": [900.0, 2.0, 2236.0],
                "greeting_string_id": 0,
                "field_sources": {
                    "placement": [{"kind": "reconstructed", "evidence": "old"}],
                    "position": [{"kind": "client", "evidence": "old"}]
                }
            }
        ]
    });
    let briarwood = "prototype/osfr-tools/data/content/npcs/briarwood.json";
    let shrouded_glade = "prototype/osfr-tools/data/content/npcs/shrouded_glade.json";

    // T502's foreign changes: a quest binding on an existing row, and Merk's
    // conversion (the zone-quest suite pins his drawn model and name).
    let mut quest = old.clone();
    quest["records"][0]["roles"] = serde_json::json!(["ambient", "quest_giver"]);
    quest["records"][0]["quest_ids_given"] = serde_json::json!([3000392]);
    quest["records"][0]["quests_accepted"] = serde_json::json!([{"quest_id": 3000391}]);
    quest["records"][0]["quests_accepted_reason"] = Value::from("T502 hand-in");
    quest["records"][0]["field_sources"] = serde_json::json!({
        "roles": [{"kind": "wiki", "evidence": "new"}],
        "quest_ids_given": [{"kind": "legacy-data", "evidence": "new"}],
        "quests_accepted": [{"kind": "legacy-data", "evidence": "new"}],
        "quests_accepted_reason": [{"kind": "legacy-data", "evidence": "new"}],
        "greeting_string_id": [{"kind": "client", "evidence": "old"}]
    });
    assert!(is_quest_binding_change_between(briarwood, &old, &quest));

    let mut merk = old.clone();
    let row = &mut merk["records"][1];
    row["placement"] = Value::from("legacy_data");
    row["position"] = serde_json::json!([482.1476, 65.42416, 1747.054]);
    row["name_string_id"] = Value::from(45_386);
    row["model_id"] = Value::from(3_952);
    row["quests_accepted"] = serde_json::json!([{"quest_id": 3000380}]);
    row["field_sources"] = serde_json::json!({
        "placement": [{"kind": "legacy-data", "evidence": "new"}],
        "position": [{"kind": "legacy-data", "evidence": "new"}],
        "name_string_id": [{"kind": "legacy-data", "evidence": "new"}],
        "model_id": [{"kind": "legacy-data", "evidence": "new"}],
        "quests_accepted": [{"kind": "legacy-data", "evidence": "new"}]
    });
    assert!(is_quest_binding_change_between(shrouded_glade, &old, &merk));

    // T698's reference-spawn placement is the one foreign NPC change allowed on
    // every region file; it is recognised by the row's own provenance marker.
    let mut reference_old = old.clone();
    reference_old["records"][0]["placement"] = Value::from("poi_anchor");
    let mut reference_new = reference_old.clone();
    let row = &mut reference_new["records"][0];
    row["placement"] = Value::from("legacy_data");
    row["placement_source"] = Value::from("legacy_data_spawn");
    row["placement_confidence"] = Value::from("medium");
    row["position_reason"] = Value::from(format!("reference spawn: {REFERENCE_SPAWN_MARKER}"));
    assert!(is_quest_binding_change_between(
        briarwood,
        &reference_old,
        &reference_new
    ));
    let mut unmarked = reference_new.clone();
    unmarked["records"][0]["position_reason"] = Value::from("moved without a spawn citation");
    assert!(!is_quest_binding_change_between(
        briarwood,
        &reference_old,
        &unmarked
    ));
    // The allowance covers placement fields only: a reference-spawn row that also
    // gains a greeting is a foreign greeting change, even though the marker matches.
    let mut cited_greeting = reference_new.clone();
    cited_greeting["records"][0]["greeting_string_id"] = Value::from(440_465);
    assert!(!is_quest_binding_change_between(
        briarwood,
        &reference_old,
        &cited_greeting
    ));
    // A follow-up may bind the spawn NameId on a row T698 already placed, and nothing
    // else; the same greeting change is still rejected there.
    let mut bound = reference_new.clone();
    bound["records"][0]["name_string_id"] = Value::from(101_924);
    bound["records"][0]["name_string_id_reason"] = Value::from("spawn NameId 101924");
    bound["records"][0]["field_sources"]["name_string_id"] =
        serde_json::json!([{"kind": "legacy-data", "evidence": "spawn"}]);
    bound["records"][0]["field_sources"]["name_string_id_reason"] =
        serde_json::json!([{"kind": "legacy-data", "evidence": "spawn"}]);
    assert!(is_quest_binding_change_between(
        briarwood,
        &reference_new,
        &bound
    ));
    let mut bound_greeting = bound.clone();
    bound_greeting["records"][0]["greeting_string_id"] = Value::from(440_465);
    assert!(!is_quest_binding_change_between(
        briarwood,
        &reference_new,
        &bound_greeting
    ));

    // A foreign greeting binding is not a quest binding. The committed guard
    // rejects it; the builder-workspace allowance still names it a greeting
    // change, so the message points at the right suite.
    let mut greeting = old.clone();
    greeting["records"][0]["greeting_string_id"] = Value::from(440_465);
    greeting["records"][0]["field_sources"]["greeting_string_id"] =
        serde_json::json!([{"kind": "client", "evidence": "new"}]);
    assert!(!is_quest_binding_change_between(briarwood, &old, &greeting));
    assert!(is_greeting_only_change_between(&old, &greeting));

    // Any other foreign content change is still rejected, and a quest-binding
    // allowance never accepts an unreviewed appended row.
    let mut moved = old.clone();
    moved["records"][0]["position"] = serde_json::json!([1.0, 2.0, 3.0]);
    assert!(!is_quest_binding_change_between(briarwood, &old, &moved));

    let mut appended = quest.clone();
    appended["records"]
        .as_array_mut()
        .expect("records")
        .push(serde_json::json!({"npc_id": 1, "name": "new"}));
    assert!(!is_quest_binding_change_between(briarwood, &old, &appended));
}

#[test]
fn the_content_stays_within_the_loader_bounds() {
    // `NpcInteraction::load` is the bounds gate: it enforces the 8 MiB per
    // file, 32 MiB total and 20,000-record limits over every `npcs/*.json`.
    let _ = service();

    for target in &NEW_GREETINGS {
        let record = npc_record(target.file, target.npc_id);
        assert!(
            record["greeting_string_id"]
                .as_u64()
                .is_some_and(|greeting| greeting > 0 && greeting <= u64::from(u32::MAX)),
            "NPC {}: greeting id",
            target.npc_id
        );
        for source in record["field_sources"]["greeting_string_id"]
            .as_array()
            .unwrap()
        {
            let evidence = source["evidence"].as_str().unwrap_or_default();
            assert!(
                !evidence.trim().is_empty(),
                "NPC {}: empty source evidence",
                target.npc_id
            );
            assert!(
                !evidence.chars().any(char::is_control),
                "NPC {}: control character in source",
                target.npc_id
            );
            assert!(
                evidence.len() <= 4096,
                "NPC {}: source evidence too long",
                target.npc_id
            );
            if source["kind"] == "reconstructed" {
                assert!(
                    evidence.starts_with("(R)"),
                    "NPC {}: (R) sources must start with (R)",
                    target.npc_id
                );
            }
        }
    }
}
