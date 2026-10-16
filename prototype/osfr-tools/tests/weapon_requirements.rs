//! W113: hostile casts must use the currently equipped job weapon.
//! Job-class requirements are reconstructed (R); client item classes/wield
//! families supply their identities, not recovered per-skill requirements.
use osfr_tools::{
    abilities::equipped_weapon_matches,
    account_store::AccountRole,
    character_store::CharacterState,
    chat,
    gameplay::{Gameplay, Outcome},
    inventory::Inventory,
};
use serde_json::Value;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static SERIAL: AtomicU64 = AtomicU64::new(0);
#[allow(dead_code)]
#[path = "../src/bin/osfr-bot.rs"]
mod bot_client;
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "w113-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn write(&self, name: &str, value: &Value) {
        fs::write(self.0.join(name), serde_json::to_vec(value).unwrap()).unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn current_owned_weapon_class_is_required_and_unequip_revokes_it() {
    let fixture = Fixture::new();
    let item = |id, definition| json!({"id":id,"definition_id":definition,"tint":0,"count":1,"consumed_count":0,"stored_count":0,"last_cast_time":0,"rental":null,"ability_count":0,"activate_enabled":false});
    let state = json!({"coins":0,"items":[item(81,100),item(82,101)],"equipment":[{"profile_id":12,"slot":7,"item_id":81,"category":7}]});
    let hash = |b: &[u8]| format!("{:x}", Sha256::digest(b));
    let definitions: Vec<_> = [(100_i32,10_i32),(101,27)].into_iter().map(|(id,class)| {
        let mut bytes = vec![0;56];
        for (offset,value) in [(0,id),(32,class),(40,7)] { bytes[offset..offset+4].copy_from_slice(&value.to_le_bytes()); }
        json!({"id":id,"slot":7,"class_id":class,"default_tint":0,"max_stack":1,"model":"synthetic.adr","texture":"","tint_alias":"","effect":0,"wield_type":0,"serialized_hex":bytes.iter().map(|b|format!("{b:02x}")).collect::<String>(),"sha256":hash(&bytes)})
    }).collect();
    let coins = vec![0; 4];
    let equipment: Vec<_> = [1_i32, 7, 81, 7]
        .into_iter()
        .flat_map(i32::to_le_bytes)
        .collect();
    let mut items = 2_i32.to_le_bytes().to_vec();
    for (id, def) in [(81_i32, 100_i32), (82, 101)] {
        for value in [def, 0, id, 1, 0, 0] {
            items.extend(value.to_le_bytes());
        }
        items.extend([0; 6]);
    }
    let span = |kind, offset, b: &[u8], profile| json!({"kind":kind,"offset":offset,"length":b.len(),"sha256":hash(b),"profile_id":profile});
    fixture.write("inventory.json", &json!({"schema_version":1,"player_guid":1,"default_state":state,"profiles":[{"id":12,"allowed_classes":[10,27],"slots":[7]}],"spans":[span("coins",13,&coins,0),span("equipment",17,&equipment,12),span("items",33,&items,0)],"definitions":definitions}));
    let mut inventory = Inventory::load(&fixture.0, None).unwrap();
    assert!(equipped_weapon_matches(&inventory, 12, Some(10)));
    assert!(
        !equipped_weapon_matches(&inventory, 11, Some(10)),
        "other profile's equipment is not authority"
    );
    assert!(!equipped_weapon_matches(&inventory, 12, None));
    assert!(!equipped_weapon_matches(&inventory, 12, Some(0)));
    inventory.equip_owned_definition(101, 12).unwrap();
    assert!(
        !equipped_weapon_matches(&inventory, 12, Some(10)),
        "owned wrong-class weapon refused"
    );
    inventory.equip_owned_definition(100, 12).unwrap();
    assert!(equipped_weapon_matches(&inventory, 12, Some(10)));
    inventory
        .handle(
            &[
                vec![42, 0, 2, 0],
                7_i32.to_le_bytes().to_vec(),
                12_i32.to_le_bytes().to_vec(),
            ]
            .concat(),
        )
        .unwrap();
    assert!(
        !equipped_weapon_matches(&inventory, 12, Some(10)),
        "live unequip revokes weapon"
    );
}

fn command(family: u16, sub: u16, fields: &[i32]) -> Vec<u8> {
    [
        family.to_le_bytes().to_vec(),
        sub.to_le_bytes().to_vec(),
        fields.iter().flat_map(|v| v.to_le_bytes()).collect(),
    ]
    .concat()
}
fn basic(target: u64) -> Vec<u8> {
    [command(36, 10, &[1, 0, 0]), target.to_le_bytes().to_vec()].concat()
}
fn hit(out: &Outcome) -> bool {
    out.replies.iter().any(|p| p.starts_with(&[32, 0, 7, 0]))
}
fn encounter(g: &Gameplay) -> Value {
    serde_json::to_value(g.state().encounter).unwrap()
}

#[test]
#[ignore = "requires OSFR_PLAY_PACKAGE with W82 starters and full item catalogue"]
fn private_fresh_outfits_cast_wrong_or_missing_weapons_refuse_every_route() {
    let source = PathBuf::from(std::env::var_os("OSFR_PLAY_PACKAGE").unwrap()).join("content");
    let fixture = Fixture::new();
    for entry in fs::read_dir(&source).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file()
            && matches!(
                path.extension().and_then(|s| s.to_str()),
                Some("json" | "bin")
            )
        {
            fs::copy(&path, fixture.0.join(path.file_name().unwrap())).unwrap();
        }
    }
    // (R) Test-only relaxed equip permission isolates the cast gate from the
    // production inventory gate. Every definition retains its original class.
    let mut inventory: Value =
        serde_json::from_slice(&fs::read(fixture.0.join("inventory.json")).unwrap()).unwrap();
    for row in inventory["profiles"].as_array_mut().unwrap() {
        row["allowed_classes"]
            .as_array_mut()
            .unwrap()
            .extend([6, 10, 17, 18, 27, 71].map(|v| json!(v)));
        let mut classes = row["allowed_classes"].as_array().unwrap().clone();
        classes.sort_by_key(|v| v.as_i64());
        classes.dedup();
        row["allowed_classes"] = json!(classes);
    }
    fixture.write("inventory.json", &inventory);
    let mut content: Value =
        serde_json::from_slice(&fs::read(fixture.0.join("encounters.json")).unwrap()).unwrap();
    let cave = content["dungeons"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|d| d["id"] == 411)
        .unwrap();
    let enemy = cave["enemies"][0].clone();
    cave["enemies"][0]["max_health"] = json!(100_000);
    fixture.write("encounters.json", &content);
    let target = enemy["guid"].as_u64().unwrap();
    let position = serde_json::from_value(enemy["position"].clone()).unwrap();
    let mut seed = Gameplay::load(&fixture.0, None).unwrap().state();
    let profiles = seed.profiles.as_mut().unwrap();
    profiles.unlocked.extend([2, 11, 12, 32, 35, 43]);
    profiles.active = 1;
    // Simulate freshly created combat outfits, not a saved login snapshot.
    for e in &mut seed.inventory.equipment {
        if [2, 11, 12, 32, 35, 43].contains(&e.profile_id) {
            e.item_id = 0;
        }
    }
    let jobs: Value =
        serde_json::from_str(include_str!("../data/content/job_abilities.json")).unwrap();
    for profile in [2, 11, 12, 32, 35, 43] {
        let mut g = Gameplay::load(&fixture.0, Some(seed.clone())).unwrap();
        g.set_dev_tools(true);
        g.set_role(AccountRole::Admin);
        let mut location = CharacterState {
            guid: 1,
            zone_id: 0,
            position: [0.; 3],
            rotation: [0.; 3],
        };
        g.handle_in_world(&command(26, 13, &[profile]), &location, 100)
            .unwrap();
        assert!(
            g.state()
                .inventory
                .equipment
                .iter()
                .any(|e| e.profile_id == profile && e.slot == 7 && e.item_id != 0),
            "starter weapon {profile}"
        );
        g.handle_in_world(&chat::local_request("realm battle 411"), &location, 200)
            .unwrap();
        location.zone_id = 411;
        location.position = position;
        g.initialize_world(&location, 300).unwrap();
        let menu = [
            command(26, 10, &[]),
            target.to_le_bytes().to_vec(),
            103_i32.to_le_bytes().to_vec(),
        ]
        .concat();
        let requests = [basic(target), basic(1), menu];
        for request in &requests {
            let mut armed = g.clone();
            assert!(
                hit(&armed.handle_in_world(request, &location, 1000).unwrap()),
                "fresh job {profile} attacks"
            );
        }
        // Actual same-session unequip after a successful cast stops the next.
        assert!(hit(&g
            .handle_in_world(&basic(target), &location, 1000)
            .unwrap()));
        g.handle_in_world(&command(42, 2, &[7, profile]), &location, 1100)
            .unwrap();
        for request in &requests {
            let before = encounter(&g);
            assert!(
                !hit(&g.handle_in_world(request, &location, 2000).unwrap()),
                "unarmed {profile}"
            );
            assert_eq!(encounter(&g), before, "refusal mutates no combat state");
        }
        let wrong = if profile == 11 { 4914 } else { 4622 };
        g.handle_in_world(
            &chat::local_request(&format!("realm give {wrong}")),
            &location,
            2100,
        )
        .unwrap();
        let id = g
            .state()
            .inventory
            .items
            .iter()
            .find(|i| i.definition_id == wrong)
            .unwrap()
            .id;
        g.handle_in_world(&command(42, 3, &[id, profile, 7]), &location, 2200)
            .unwrap();
        for request in &requests {
            let before = encounter(&g);
            assert!(
                !hit(&g.handle_in_world(request, &location, 3000).unwrap()),
                "wrong weapon {profile}"
            );
            assert_eq!(encounter(&g), before);
        }
        // Every executable skill, including pure heals/buffs, refuses before
        // energy, cooldown or effects. Re-equipping permits that same request.
        g.handle_in_world(&chat::local_request("realm level 20"), &location, 3100)
            .unwrap();
        let row = jobs["jobs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["profile_id"] == profile)
            .unwrap();
        let starter = inventory_weapon(&g, profile, &fixture.0);
        for skill in row["slots"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|s| s["unavailable"] != true)
        {
            let request = [
                command(36, 10, &[1, skill["slot"].as_i64().unwrap() as i32, 0]),
                target.to_le_bytes().to_vec(),
            ]
            .concat();
            for unarmed in [false, true] {
                let mut refused = g.clone();
                if unarmed {
                    refused
                        .handle_in_world(&command(42, 2, &[7, profile]), &location, 3200)
                        .unwrap();
                }
                let before = encounter(&refused);
                let out = refused.handle_in_world(&request, &location, 4000).unwrap();
                assert!(
                    out.replies.is_empty(),
                    "refused skill {profile}/{}",
                    skill["slot"]
                );
                assert_eq!(encounter(&refused), before);
                refused
                    .handle_in_world(&command(42, 3, &[starter, profile, 7]), &location, 4100)
                    .unwrap();
                let out = refused.handle_in_world(&request, &location, 4200).unwrap();
                assert!(
                    out.replies.iter().any(|p| p.starts_with(&[35, 0, 8, 0])),
                    "valid skill {profile}/{}",
                    skill["slot"]
                );
                assert!(
                    !out.replies.iter().any(|p| p.starts_with(&[36, 0, 1, 0])),
                    "valid skill refused"
                );
            }
        }
        println!(
            "job {profile}: fresh starter basic; targeted/self/menu; wrong/no weapon; mid-fight unequip; all executable skills PASS"
        );
    }
}

fn inventory_weapon(g: &Gameplay, profile: i32, content: &std::path::Path) -> i32 {
    let state = g.state().inventory;
    let inventory = Inventory::load(content, Some(state.clone())).unwrap();
    let class = match profile {
        2 => 6,
        11 => 27,
        12 => 10,
        32 => 17,
        35 => 18,
        43 => 71,
        _ => unreachable!(),
    };
    state
        .items
        .iter()
        .find_map(|item| {
            let mut candidate = inventory.clone();
            candidate
                .equip_owned_definition(item.definition_id, profile)
                .ok()?;
            equipped_weapon_matches(&candidate, profile, Some(class)).then_some(item.id)
        })
        .expect("starter of correct class")
}

#[test]
#[ignore = "requires W113 loopback scratch gateway, never a native session"]
fn scratch_wizard_bot_hits_with_wand_then_unequips_and_is_refused() {
    use std::time::Duration;
    let reference = PathBuf::from(std::env::var_os("OSFR_W113_REFERENCE").unwrap());
    let manifest: Value =
        serde_json::from_slice(&fs::read(reference.join("manifest.json")).unwrap()).unwrap();
    let gateway: std::net::SocketAddr =
        std::env::var("OSFR_W113_GATEWAY").unwrap().parse().unwrap();
    assert!(
        gateway.ip().is_loopback(),
        "scratch bot stays loopback-only"
    );
    let mut bot = bot_client::Bot::connect(bot_client::BotConfig {
        gateway,
        name: String::new(),
        connection_id: std::process::id(),
        guid: manifest["initial_state"]["guid"].as_u64().unwrap(),
        ticket: osfr_tools::login::DEFAULT_GATEWAY_TICKET.to_owned(),
        version: manifest["client_version"].as_str().unwrap().to_owned(),
        expect_timeout_ms: 2500,
        quiet: true,
    })
    .unwrap();
    bot.login().unwrap();
    for line in [
        "raw 1a000d000c000000",
        "expect 38 21",
        "expect 36 5",
        "chat realm battle 411",
        "expect 41 114",
        "wait 700",
        "move 291 28 289",
        "wait 100",
        "ability 0 5715144128442986753",
        "expect 32 7",
        "raw 2a000200070000000c000000",
        "expect 38 6",
        "wait 700",
    ] {
        bot.run_step(&bot_client::parse_step(line).unwrap().unwrap())
            .unwrap();
    }
    bot.run_step(
        &bot_client::parse_step("ability 0 5715144128442986753")
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    assert!(
        bot.wait_for(Duration::from_millis(900), |p| {
            p.starts_with(&[32, 0, 7, 0]) && p.get(4..12) == Some(1_u64.to_le_bytes().as_slice())
        })
        .is_err(),
        "unequipped Wizard must produce no player attack (enemy attacks may continue)"
    );
    println!(
        "Wizard bot PASS: wand 32/7 hit, live 42/2 unequip 38/6, next cast no player 32/7 for 900 ms"
    );
    bot.disconnect();
}

#[test]
fn shipped_combat_rows_require_each_jobs_weapon_class() {
    let basics: Value =
        serde_json::from_str(include_str!("../data/content/opening-abilities.json")).unwrap();
    let jobs: Value =
        serde_json::from_str(include_str!("../data/content/job_abilities.json")).unwrap();
    for (profile, class) in [(2, 6), (11, 27), (12, 10), (32, 17), (35, 18), (43, 71)] {
        let basic = basics["basics"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["profile_id"] == profile)
            .expect("combat basic");
        assert_eq!(basic["weapon_class"], class, "profile {profile} basic");
        let job = jobs["jobs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["profile_id"] == profile)
            .unwrap();
        for skill in job["slots"].as_array().unwrap() {
            assert_eq!(
                skill["weapon_class"], class,
                "profile {profile}, slot {}",
                skill["slot"]
            );
        }
    }
}
