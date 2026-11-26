use osfr_tools::progression_extras::{parse_collection_content, parse_title_content};
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

#[derive(Deserialize)]
struct LocaleRow {
    id: u32,
    text: String,
}

#[derive(Deserialize)]
struct ItemDefinitionRow {
    #[serde(rename = "Id")]
    id: u32,
}

fn project_path(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn lab_path(path: &str) -> PathBuf {
    project_path("../../../lab").join(path)
}

fn locale() -> HashMap<u32, String> {
    let bytes = fs::read(lab_path("reports/locale-id-to-text.json")).unwrap();
    serde_json::from_slice::<Vec<LocaleRow>>(&bytes)
        .unwrap()
        .into_iter()
        .map(|row| (row.id, row.text))
        .collect()
}

fn client_images() -> HashSet<u32> {
    fs::read_to_string(lab_path(
        "working/OSFR.Client/OSFR Client/Resources/Images/Images.txt",
    ))
    .unwrap()
    .lines()
    .filter_map(|line| line.split('^').next()?.parse().ok())
    .collect()
}

fn client_item_definitions() -> HashSet<u32> {
    let bytes = fs::read(lab_path(
        "references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json",
    ))
    .unwrap();
    serde_json::from_slice::<Vec<ItemDefinitionRow>>(&bytes)
        .unwrap()
        .into_iter()
        .map(|row| row.id)
        .collect()
}

#[test]
fn title_and_collection_ids_resolve_to_client_data() {
    let locale = locale();
    let images = client_images();
    let items = client_item_definitions();
    let titles =
        parse_title_content(&fs::read(project_path("data/content/titles.json")).unwrap()).unwrap();
    let collections =
        parse_collection_content(&fs::read(project_path("data/content/collections.json")).unwrap())
            .unwrap();

    assert_eq!(titles.definitions.len(), 90);
    let title_ids: HashSet<u32> = titles
        .definitions
        .iter()
        .map(|title| title.client_id.unwrap())
        .collect();
    assert_eq!(title_ids.len(), titles.definitions.len());
    for title in &titles.definitions {
        let name = &locale[&title.name_id.unwrap()];
        assert!(!name.trim().is_empty());
        if let Some(id) = title.description_id {
            assert!(!locale[&id].trim().is_empty());
        }
        if let Some(image_id) = title.image_id {
            assert!(
                images.contains(&image_id),
                "unresolved title image {image_id}"
            );
        }
        assert!(title.source_tags.contains_key("client_id"));
    }

    assert_eq!(collections.definitions.len(), 241);
    let collection_ids: HashSet<u32> = collections
        .definitions
        .iter()
        .map(|collection| collection.client_id.unwrap())
        .collect();
    assert_eq!(collection_ids.len(), collections.definitions.len());
    for collection in &collections.definitions {
        for id in [collection.name_id, collection.description_id]
            .into_iter()
            .flatten()
        {
            assert!(
                !locale[&id].trim().is_empty(),
                "unresolved collection locale id {id}"
            );
        }
        if let Some(image_id) = collection.image_id {
            assert!(
                images.contains(&image_id),
                "unresolved collection image {image_id}"
            );
        }
        for entry in &collection.entries {
            if let Some(id) = entry.name_id {
                assert!(
                    !locale[&id].trim().is_empty(),
                    "unresolved entry locale id {id}"
                );
            }
            if let Some(id) = entry.item_definition_id {
                assert!(
                    items.contains(&id),
                    "unknown collection item definition {id}"
                );
            }
            if let Some(id) = entry.icon_id {
                assert!(images.contains(&id), "unresolved entry image {id}");
            }
        }
    }

    let briarwood = collections
        .definitions
        .iter()
        .find(|collection| collection.client_id == Some(17054))
        .unwrap();
    assert_eq!(briarwood.category_id, Some(10));
    assert_eq!(briarwood.wiki_category_id, Some(3));
    assert_eq!(briarwood.description_id, Some(17055));
    assert_eq!(briarwood.entries.len(), 8);
    assert!(
        briarwood
            .entries
            .iter()
            .all(|entry| entry.item_definition_id.is_some())
    );
}

/// A53: on 150/3 the client looks the record's trophy id up in `AchievementTrophies.txt` and reads
/// the row without a null check (`00badec0` at `00bae04c`), so an achievement whose trophy is not a
/// client row crashes the client when it completes.
#[test]
fn every_achievement_sent_to_the_client_names_a_client_trophy() {
    let trophies: HashSet<u32> = fs::read_to_string(lab_path(
        "working/OSFR.Client/OSFR Client/Resources/AchievementTrophies.txt",
    ))
    .unwrap()
    .lines()
    .filter_map(|line| line.split('^').next()?.parse().ok())
    .collect();
    let bindings =
        osfr_tools::progression_extras::AchievementBindings::load(&project_path("data/content"))
            .unwrap();
    assert!(!bindings.client_rows().is_empty());
    let missing: Vec<_> = bindings
        .client_rows()
        .iter()
        .filter(|row| !trophies.contains(&row.trophy_id))
        .map(|row| (row.key.as_str(), row.trophy_id))
        .collect();
    assert!(missing.is_empty(), "no client trophy row: {missing:?}");
}

/// A67: an icon of 0 or a set with no Large image draws the client's "OOPS!" placeholder, and
/// a category that is not an AchievementCategories.txt row leaves the row out of the window.
#[test]
fn every_achievement_sent_to_the_client_has_a_client_icon_and_category() {
    let first_ids = |path: &str| -> Vec<Vec<String>> {
        fs::read_to_string(lab_path(path))
            .unwrap()
            .lines()
            .filter(|line| !line.starts_with('#'))
            .map(|line| line.split('^').map(str::to_owned).collect())
            .collect()
    };
    let large_sets: HashSet<u32> =
        first_ids("working/OSFR.Client/OSFR Client/Resources/Images/ImageSetMappings.txt")
            .into_iter()
            .filter(|row| row.get(1).map(String::as_str) == Some("7"))
            .filter_map(|row| row[0].parse().ok())
            .collect();
    let categories: HashSet<u32> =
        first_ids("working/OSFR.Client/OSFR Client/Resources/AchievementCategories.txt")
            .into_iter()
            .filter_map(|row| row[0].parse().ok())
            .collect();
    let bindings =
        osfr_tools::progression_extras::AchievementBindings::load(&project_path("data/content"))
            .unwrap();
    assert!(!bindings.client_rows().is_empty());
    let bad: Vec<_> = bindings
        .client_rows()
        .iter()
        .filter(|row| !large_sets.contains(&row.icon_id) || !categories.contains(&row.category_id))
        .map(|row| (row.key.as_str(), row.icon_id, row.category_id))
        .collect();
    assert!(bad.is_empty(), "no client icon or category: {bad:?}");
}
