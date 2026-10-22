//! Roosey's coin shop as a one-vendor content pack, shared by the T362 window suite
//! (`vendor_coin_window`) and the T434 Worth suite (`vendor_sell_worth`). The pack is the
//! round70 base (plus `extra` files from it) with the repo's stores, zones and Roosey's vendor
//! and NPC rows, built under `CARGO_TARGET_TMPDIR` and removed on drop. The including suite
//! declares `gateway_vendor` and `round70_content` at its root.

use crate::gateway_vendor::{NOW, interact, login, select, send, wrap};
use crate::round70_content;
use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState},
    gateway::Gateway,
    npc_interaction::{NpcAction, NpcInteraction},
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub const ROOSEY: u64 = 9_500_000_255;

static NEXT: AtomicU64 = AtomicU64::new(0);

pub fn data() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

pub struct Fixture {
    pub content: PathBuf,
    pub reference: PathBuf,
    pub option: u32,
    pub position: [f32; 3],
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.content);
    }
}

impl Fixture {
    pub fn new(extra: &[&str]) -> Self {
        let reference = round70_content::reference();
        let content = round70_content::build(
            &PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("roosey-shop"),
            &format!("roosey-{}", NEXT.fetch_add(1, Ordering::Relaxed)),
            extra,
        );
        let data = data();
        for file in ["stores.json", "zones.json"] {
            fs::copy(data.join(file), content.join(file)).expect("fixture content file copies");
        }
        let vendors = read_json(&data.join("vendors.json"));
        let vendor = vendors["vendors"]
            .as_array()
            .expect("vendor records are an array")
            .iter()
            .find(|vendor| vendor["npc_id"].as_u64() == Some(ROOSEY))
            .expect("Roosey's vendor row exists")
            .clone();
        write_json(
            &content.join("vendors.json"),
            &json!({"schema_version": 1, "vendors": [vendor]}),
        );
        let npcs = read_json(&data.join("npcs/cobblestone_village.json"));
        let npc = npcs["records"]
            .as_array()
            .expect("NPC records are an array")
            .iter()
            .find(|npc| npc["npc_id"].as_u64() == Some(ROOSEY))
            .expect("Roosey's NPC record exists")
            .clone();
        write_json(
            &content.join("npcs/roosey.json"),
            &json!({"schema_version": 1, "records": [npc]}),
        );
        write_json(
            &content.join("npc_spawns_extra.json"),
            &json!({"schema_version": 1, "records": []}),
        );

        let mut service = NpcInteraction::load(&content.join("npcs"), &content.join("stores.json"))
            .expect("fixture NPC content loads");
        let (zone, position) = service.location(ROOSEY).expect("Roosey has a location");
        let menu = service
            .interact(ROOSEY, zone, position, None)
            .expect("Roosey opens her menu");
        let option = u32::try_from(
            menu.options
                .iter()
                .find(|option| option.action == NpcAction::Shop)
                .expect("Roosey exposes the Shop option")
                .id,
        )
        .expect("the shop option id fits the wire field");
        Self {
            content,
            reference,
            option,
            position,
        }
    }

    pub fn character(&self) -> CharacterState {
        let mut character =
            Gateway::reference_state(&self.reference).expect("reference character loads");
        character.zone_id = 0;
        character.position = self.position;
        character.rotation = [0.0; 3];
        character
    }

    /// The pack's gameplay state with an empty bag, nothing worn and 1,000 coins.
    pub fn state(&self) -> GameplayState {
        let mut state = Gameplay::load(&self.content, None)
            .expect("fixture gameplay content loads")
            .state();
        state.inventory.items.clear();
        for entry in &mut state.inventory.equipment {
            entry.item_id = 0;
        }
        state.inventory.coins = 1_000;
        state.inventory.next_instance_id = Some(1);
        state
    }

    pub fn connect(&self, state: GameplayState) -> Gateway {
        let mut gateway = Gateway::load_gameplay(
            &self.reference,
            self.character(),
            &self.content,
            Some(state),
        )
        .expect("gateway loads the Roosey content");
        gateway.set_dev_tools(false);
        gateway.set_role(AccountRole::Player);
        let guid = gateway.state().guid;
        for packet in [login(guid), wrap(&[13, 0]), wrap(&[10, 0])] {
            gateway
                .handle_at(&packet, NOW)
                .expect("initialization message handles");
        }
        gateway
    }

    /// The production path: 26/8 opens the menu and 26/10 selects the Shop option.
    pub fn open_shop(&self, gateway: &mut Gateway) -> Vec<Vec<u8>> {
        let menu = send(gateway, &interact(ROOSEY));
        assert!(
            menu.iter().any(|packet| packet.starts_with(&[26, 0, 9, 0])),
            "26/8 must answer with the 26/9 menu"
        );
        send(gateway, &select(ROOSEY, self.option))
    }
}

pub fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("fixture JSON reads"))
        .expect("fixture JSON parses")
}

fn write_json(path: &Path, value: &Value) {
    fs::write(
        path,
        serde_json::to_vec(value).expect("fixture JSON serializes"),
    )
    .expect("fixture JSON writes");
}
