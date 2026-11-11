//! The names the panel shows for ids the other routes give: jobs by profile id (`jobs.json`) and
//! regional quests by id (`quests/*.json`), each quest's title through the client locale
//! (`world_text::locale_text`, loaded with the gateway's `--client-locale`). A quest with no
//! locale text is left out, and the panel shows its id.
use serde_json::{Map, Value, json};
use std::path::Path;

fn records(path: &Path) -> Vec<Value> {
    std::fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        .and_then(|mut document| match document.get_mut("records")?.take() {
            Value::Array(rows) => Some(rows),
            _ => None,
        })
        .unwrap_or_default()
}

/// `{"jobs": {"<profile id>": name}, "quests": {"<quest id>": title}}` from `content`.
pub fn names(content: &Path) -> Value {
    let mut jobs = Map::new();
    for job in records(&content.join("jobs.json")) {
        if let (Some(profile), Some(name)) = (job["profile_id"].as_i64(), job["name"].as_str()) {
            jobs.insert(profile.to_string(), json!(name));
        }
    }
    let mut quests = Map::new();
    let files = std::fs::read_dir(content.join("quests"))
        .into_iter()
        .flatten()
        .flatten();
    for file in files {
        for quest in records(&file.path()) {
            let title = quest["title_string_id"]
                .as_i64()
                .and_then(|id| i32::try_from(id).ok())
                .and_then(crate::world_text::locale_text);
            if let (Some(id), Some(title)) = (quest["id"].as_u64(), title) {
                quests.insert(id.to_string(), json!(title));
            }
        }
    }
    json!({"jobs": jobs, "quests": quests})
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn shipped_jobs_are_named_by_profile_id() {
        let content = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join("content");
        let names = super::names(&content);
        assert_eq!(names["jobs"]["1"], "Adventurer");
        assert!(names["jobs"].as_object().is_some_and(|jobs| jobs.len() > 5));
        assert!(
            names["quests"].is_object(),
            "titles need the client locale; never an error"
        );
    }
}
