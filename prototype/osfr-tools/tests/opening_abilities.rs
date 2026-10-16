use osfr_tools::opening_abilities::OpeningAbilities;
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    path: PathBuf,
    content: Value,
}
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-abilities-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        let content = json!({"schema_version":1,"basics":[{"profile_id":43,"definition_id":1234,"name_id":2301,"description_id":2302,"icon_id":2303,"weapon_name_id":2401}]});
        let f = Self { path, content };
        f.write();
        f
    }
    fn write(&self) {
        fs::write(
            self.path.join("opening-abilities.json"),
            serde_json::to_vec(&self.content).unwrap(),
        )
        .unwrap();
    }
    fn load(&self) -> OpeningAbilities {
        OpeningAbilities::load(&self.path).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.path.join("opening-abilities.json"));
        let _ = fs::remove_dir(&self.path);
    }
}
fn vectors(name: &str) -> Vec<Vec<u8>> {
    let data: Value = serde_json::from_str(include_str!(
        "fixtures/opening-ability-community-vectors.json"
    ))
    .unwrap();
    data["vectors"][name]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            s.as_str()
                .unwrap()
                .as_bytes()
                .chunks_exact(2)
                .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
                .collect()
        })
        .collect()
}

#[test]
fn basic_definition_and_toolbar_equal_independent_community_serializers() {
    let f = Fixture::new();
    assert_eq!(f.load().activation(43, 1).unwrap(), vectors("synthetic_basic"));
}

#[test]
fn noncombat_class_clears_all_eight_slots_and_invalid_profile_fails() {
    let f = Fixture::new();
    let abilities = f.load();
    assert_eq!(abilities.activation(1, 1).unwrap(), vectors("synthetic_empty"));
    assert!(abilities.activation(0, 1).is_err());
    assert!(abilities.activation(-1, 1).is_err());
}

#[test]
fn loaded_metadata_rejects_invalid_fields_duplicates_and_oversized_content() {
    let mut f = Fixture::new();
    let original = f.content.clone();
    for field in ["profile_id", "definition_id", "icon_id", "weapon_name_id"] {
        f.content = original.clone();
        f.content["basics"][0][field] = json!(0);
        f.write();
        assert!(OpeningAbilities::load(&f.path).is_err(), "{field}");
    }
    f.content = original.clone();
    f.content["basics"]
        .as_array_mut()
        .unwrap()
        .push(original["basics"][0].clone());
    f.write();
    assert!(OpeningAbilities::load(&f.path).is_err());
    f.content = original;
    f.content["schema_version"] = json!(2);
    f.write();
    assert!(OpeningAbilities::load(&f.path).is_err());
    fs::write(f.path.join("opening-abilities.json"), vec![b' '; 65537]).unwrap();
    assert!(OpeningAbilities::load(&f.path).is_err());
}

#[test]
fn returned_packets_and_clones_do_not_mutate_shared_metadata() {
    let f = Fixture::new();
    let abilities = f.load();
    let clone = abilities.clone();
    let mut replies = clone.activation(43, 1).unwrap();
    replies[0][10] ^= 1;
    assert_eq!(
        abilities.activation(43, 1).unwrap(),
        vectors("synthetic_basic")
    );
    assert_eq!(clone.activation(43, 1).unwrap(), vectors("synthetic_basic"));
}

#[test]
#[ignore = "requires OSFR_OPENING_CONTENT and private compiled community serializer fixtures"]
fn private_opening_hotbars_equal_compiled_community_packets() {
    let directory = PathBuf::from(std::env::var("OSFR_OPENING_CONTENT").expect("opening content"));
    let abilities = OpeningAbilities::load(&directory).unwrap();
    for profile in [43, 11, 35] {
        let expected: Vec<_> = (0..2)
            .map(|i| {
                fs::read(
                    directory
                        .join("opening-ability-oracle")
                        .join(format!("{profile}-{i}.bin")),
                )
                .unwrap()
            })
            .collect();
        assert_eq!(
            abilities.activation(profile, 1).unwrap(),
            expected,
            "profile{profile}"
        );
    }
}
