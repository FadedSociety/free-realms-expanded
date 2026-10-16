use osfr_tools::{opening::OpeningActor, opening_world::OpeningWorld};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn vector(name: &str) -> Vec<u8> {
    let data: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/opening-world-community-vectors.json"
    ))
    .unwrap();
    data["vectors"][name]
        .as_str()
        .unwrap()
        .as_bytes()
        .chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn native_prompt_radius_with_utf8_name_matches_full_community_packet_and_preserves_menu() {
    let temp = Temp(std::env::temp_dir().join(format!(
        "osfr-opening-world-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir(&temp.0).unwrap();
    let template = vector("template");
    fs::write(temp.0.join("synthetic-template.bin"), &template).unwrap();
    fs::write(temp.0.join("combat.json"), serde_json::to_vec(&serde_json::json!({
        "template_file": "synthetic-template.bin", "template_sha256": format!("{:x}", Sha256::digest(&template))
    })).unwrap()).unwrap();
    let world = OpeningWorld::load(&temp.0).unwrap();
    let mut actor = OpeningActor {
        guid: 0x0102030405060708,
        definition_id: 303,
        name_id: 101,
        name: "Artéfact 猫".into(),
        model_id: 202,
        model_file_name: "synthetic.adr".into(),
        zone_id: 0,
        position: [-1.5, 2.25, -3.75],
        heading: 0.0,
        interaction_radius: 3.25,
        kind: "npc".into(),
        action: None,
        required_profile: None,
        provenance: "synthetic community serializer fixture".into(),
    };
    let spawn = world.spawn(&actor);
    assert_eq!(
        spawn,
        vector("spawn_after"),
        "entire packet must match unchanged community writer"
    );
    let before = vector("spawn_before");
    // Name lengths on the wire count UTF8 bytes; insertion shifts every later field.
    assert_eq!(
        u32::from_le_bytes(spawn[106..110].try_into().unwrap()) as usize,
        actor.name.len()
    );
    let radius = 237 + actor.name.len();
    assert_eq!(&spawn[..radius], &before[..radius]);
    assert_eq!(&spawn[radius + 4..], &before[radius + 4..]);
    assert_eq!(
        f32::from_le_bytes(spawn[radius..radius + 4].try_into().unwrap()),
        3.25
    );
    let legacy_range = 133 + actor.name.len();
    assert_eq!(
        i32::from_le_bytes(spawn[legacy_range..legacy_range + 4].try_into().unwrap()),
        4
    );
    assert_eq!(OpeningWorld::menu(&actor), vector("menu_npc"));
    actor.kind = "prop".into();
    assert_eq!(OpeningWorld::menu(&actor), vector("menu_prop"));
    let mut enemy = actor.clone();
    enemy.kind = "enemy".into();
    enemy.guid += 1;
    let mut other_zone = actor.clone();
    other_zone.zone_id = 411;
    other_zone.guid += 2;
    let packets = world.initialization(&[actor.clone(), enemy, other_zone], 0);
    let mut relevance = vec![35, 0, 12, 0, 1, 0, 0, 0];
    relevance.extend(actor.guid.to_le_bytes());
    relevance.extend([1, 5, 0]);
    assert_eq!(packets, vec![spawn, relevance]);
}
