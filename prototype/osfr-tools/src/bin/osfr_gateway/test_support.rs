//! Shared setup for the gateway binary's `#[cfg(test)]` session modules: one scratch
//! directory and one `Options` initializer, so a new option field is added once instead
//! of in every test copy. Test-only; production never includes this file.
use super::*;

/// A clean temp directory; dropping it removes the directory and the saves in it.
pub struct Scratch(PathBuf);

impl Scratch {
    pub fn new(tag: &str) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-{tag}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();
        Self(directory)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn state(&self) -> PathBuf {
        self.0.join("character.json")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The prepared round59 content plus one synthetic vendor NPC and its one-row store, in the
/// scratch directory. `npc_spawns_extra.json` and `npcs/` keep the content local so the bundled
/// `data/content` catalogue cannot leak into a test; the caller may add its own catalogue files.
pub fn vendor_content(scratch: &Scratch) -> PathBuf {
    let content = scratch.path().join("content");
    fs::create_dir_all(content.join("npcs")).unwrap();
    fs::create_dir_all(content.join("quests")).unwrap();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
    let source = std::env::var_os("OSFR_APP_FUZZ_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("reports/round59-native-20261015T124443Z/content"));
    for entry in fs::read_dir(&source).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            fs::copy(&path, content.join(path.file_name().unwrap())).unwrap();
        }
    }
    let write = |name: &str, value: serde_json::Value| {
        fs::write(content.join(name), serde_json::to_vec(&value).unwrap()).unwrap();
    };
    let provenance = json!([{"kind":"reconstructed","evidence":"F264 synthetic test policy"}]);
    write(
        "npcs/synthetic.json",
        json!({"schema_version":1,"records":[
            {"npc_id":700,"name":"Synthetic NPC","zone_id":0,"position":[0.,0.,0.],
             "placement":"legacy_data","model_id":1,"roles":["vendor"],"vendor_stock":"synthetic"}
        ]}),
    );
    write(
        "npc_spawns_extra.json",
        json!({"schema_version":1,"records":[]}),
    );
    write(
        "zones.json",
        json!({"schema_version":1,"records":[{"id":0,"name":"Synthetic zone"}]}),
    );
    write(
        "vendors.json",
        json!({"schema_version":1,"vendors":[{"npc_id":700,"item_ids":[8]}]}),
    );
    write(
        "stores.json",
        json!({"schema_version":1,"marketplace":{
            "enabled_on_handshake":false,"orders_enabled":false,
            "payment_sources_enabled":false,"not_available_message":"disabled"},
            "items":[{"item_id":8,"purchase_price_coins":15,"member_discount_percent":null,
                      "member_price_coins":null,"sell_price_coins":7,"no_sale":false}]}),
    );
    write(
        "quests/synthetic.json",
        json!({"schema_version":1,"records":[{
            "id":701,"title_string_id":1,"giver_npc_id":700,"prerequisite_quest_ids":[],
            "objectives":[{"kind":"talk_to_npc","id":702,"npc_id":700,"source":provenance}],
            "rewards":[{"coins":7,"items":[],"job_xp":{},"source":provenance}],"source":provenance
        }]}),
    );
    write(
        "jobs.json",
        json!({"schema_version":1,"records":[{
            "id":"synthetic","name":"Synthetic job","profile_id":null,"category":"freestyle",
            "max_level":20,"members_only":false,"trainer_npc_ids":[],"quest_ids":[701],
            "abilities":[],"source":provenance
        }]}),
    );
    content
}

/// Options for a synthetic session on the bundled lab content, without a listener.
pub fn options(state: PathBuf) -> Options {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
    Options {
        bind: "127.0.0.1:0".parse().unwrap(),
        reference: root.join("references/rust-gateway-reference"),
        state,
        content: Some(root.join("references/rust-opening-cooking-original-assets")),
        identity: None,
        account_store: None,
        gateway_ticket: login::DEFAULT_GATEWAY_TICKET.into(),
        handoff_key: Vec::new(),
        seconds: 1,
        dev_tools: false,
        studio_port: 0,
        client_logs: None,
        terrain: None,
        world: None,
        stock_text: None,
        allow_lan: false,
        security_log: None,
        group_invite_ttl_ms: osfr_tools::social_lists::DEFAULT_GROUP_INVITE_TTL_MS,
    }
}
