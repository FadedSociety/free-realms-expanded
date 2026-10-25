//! Harvest-node content imported from the pinned Legacy data tables and the community
//! collection tables (T523, dormant), and (W171) the ordinary way into mining and harvesting:
//! a node actor in the world that a player clicks.
//!
//! Recovered: a click on any interactable actor sends 26/8 (`PlayClient::RequestInteraction`,
//! `0092efe0`). Everything below that is reconstructed policy (R): the node is a plain actor,
//! 26/8 on it starts the Type8 run directly (as Legacy's collection nodes do, no menu), the
//! required job, the range, the per-player depletion and the respawn timers.

use crate::{
    harvesting::CHEF_PROFILE_ID,
    inventory::Inventory,
    mining::{MINER_PROFILE_ID, Type8BoardSpec},
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::Path,
    sync::Arc,
};

const MAX_FILE_BYTES: u64 = 1 << 20;
/// Node actor guids: "NOD", the node file's tag byte, then the content spawn id. The tag is
/// OR-ed into a zero byte, so every tag needs its own value; see `NODE_FILES`. Clear of NPC,
/// placement and pickup ids.
const NODE_GUID_BASE: u64 = 0x4e4f_4400_0000_0000;

/// Guid tag byte owned by each node content file. Tags share one spawn-id space, so no two
/// files may reuse one. `COLLECTION_NODE_TAG` names the dormant community collection file
/// (T523, `collection_nodes.json`), which nothing loads yet. T524 moves that file to tag 5 and
/// gives tag 4 to its per-node effect companions; no tag is reserved from another source, so
/// the live invariant is uniqueness (`tests/node_guid_namespace.rs`).
const HARVEST_NODE_TAG: u64 = 1;
const MINING_NODE_TAG: u64 = 2;
const QUEST_NODE_TAG: u64 = 3;
#[cfg(test)]
const COLLECTION_NODE_TAG: u64 = 4;

/// The node files `GatheringNodes::load` reads and the guid tag each one owns.
const NODE_FILES: [(&str, Option<Gather>, u64); 3] = [
    (
        "harvest_nodes.json",
        Some(Gather::Harvesting),
        HARVEST_NODE_TAG,
    ),
    ("mining_nodes.json", Some(Gather::Mining), MINING_NODE_TAG),
    ("quest_nodes.json", None, QUEST_NODE_TAG),
];

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct HarvestNodesContent {
    pub schema_version: u32,
    #[serde(default)]
    pub provenance: BTreeMap<String, String>,
    pub node_types: Vec<HarvestNodeType>,
    pub pools: Vec<HarvestNodePool>,
    pub spawns: Vec<HarvestNodeSpawn>,
    /// Generator findings (T523). Absent and empty for the Legacy files. Not read at runtime;
    /// collection-node-pickups owns it.
    #[serde(default)]
    pub gaps: HarvestNodeGaps,
}

/// Import gaps the generator records; nothing is refused for them.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct HarvestNodeGaps {
    /// Pool drop items that the collection catalogue (T447) does not link to exactly one entry.
    #[serde(default)]
    pub drop_items_without_entry: Vec<u32>,
}

impl HarvestNodesContent {
    pub fn load_from(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        if fs::metadata(path)?.len() > MAX_FILE_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "node file over 1 MiB",
            ));
        }
        let bytes = fs::read(path)?;
        serde_json::from_slice(&bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct HarvestNodeType {
    pub key: String,
    pub name: String,
    pub model_id: u32,
    pub scale: f32,
    pub composite_effect_id: u32,
    /// Sanctuary adds these on top of the main effect (sparkling water); Legacy has none. Not
    /// read at runtime; collection-node-pickups owns it.
    #[serde(default)]
    pub extra_composite_effect_ids: Vec<u32>,
    pub interact_range: u32,
    pub cursor_id: u32,
    pub placement_y_offset: f32,
    /// Sanctuary's rare-node subtitle locale row; absent on common types and Legacy nodes. Not
    /// read at runtime; collection-node-pickups owns it.
    #[serde(default)]
    pub sub_text_name_id: Option<i32>,
    /// Legacy drop table; unused by the Type8 payout, and absent for mining nodes.
    #[serde(default)]
    pub drop_table: Vec<HarvestNodeDrop>,
    #[serde(default)]
    pub source_tag: Option<String>,
    #[serde(default)]
    pub source_tags: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct HarvestNodeDrop {
    pub item_definition_id: u32,
    pub weight: u32,
    #[serde(default)]
    pub source_tags: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct HarvestNodePool {
    pub key: String,
    pub zone_definition_id: u32,
    pub node_type: String,
    pub max_active_nodes: u32,
    pub respawn_seconds: u32,
    /// Sanctuary keeps the loot table on the pool, Legacy on the type. Not read by W171, whose
    /// Type8 payout owns the reward.
    #[serde(default)]
    pub drop_table: Vec<HarvestNodeDrop>,
    #[serde(default)]
    pub source_tag: Option<String>,
    #[serde(default)]
    pub source_tags: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct HarvestNodeSpawn {
    pub id: u32,
    pub pool_key: String,
    pub zone_definition_id: u32,
    pub position: [f32; 3],
    pub heading: f32,
    #[serde(default)]
    pub source_tag: Option<String>,
    #[serde(default)]
    pub source_tags: BTreeMap<String, String>,
}

/// Which Type8 game a node starts. (R) Mushroom nodes start Harvesting111 (Chef), ore nodes Mining18.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Gather {
    Mining,
    Harvesting,
}

impl Gather {
    pub fn job(self) -> i32 {
        match self {
            Self::Mining => MINER_PROFILE_ID,
            Self::Harvesting => CHEF_PROFILE_ID,
        }
    }

    /// Lowercase noun for the developer-command sentences.
    pub(crate) fn noun(self) -> &'static str {
        match self {
            Self::Mining => "mining",
            Self::Harvesting => "harvesting",
        }
    }

    /// Capitalised noun for the start of a sentence.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Mining => "Mining",
            Self::Harvesting => "Harvesting",
        }
    }

    /// Capitalised job word used by the NPC refusal messages.
    pub(crate) fn word(self) -> &'static str {
        match self {
            Self::Mining => "Miner",
            Self::Harvesting => "Chef",
        }
    }
}

/// One node as the world draws it (AddNpc via `NpcVisibility::add_static_actor`).
#[derive(Clone, Debug, PartialEq)]
pub struct NodeActor {
    pub guid: u64,
    pub name: String,
    pub model_id: u32,
    pub position: [f32; 3],
}

pub enum NodeChange {
    Show(NodeActor),
    Hide(u64),
}

/// What a click on a placed node starts. A quest pickup names its `quest_nodes.json` node
/// type and grants no item (the fork's quest node types have an empty drop table, T503).
enum SiteKind {
    Gather(Gather),
    Pickup(String),
}

struct Site {
    actor: NodeActor,
    kind: SiteKind,
    range: f64,
    respawn_ms: u64,
}

impl Site {
    fn in_reach(&self, position: [f32; 3]) -> bool {
        let reach = self
            .actor
            .position
            .iter()
            .zip(position)
            .map(|(a, b)| (f64::from(*a) - f64::from(b)).powi(2))
            .sum::<f64>();
        reach.is_finite() && reach <= self.range * self.range
    }
}

/// Why a click did not start a run. Every variant leaves the player and the node untouched.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Denied {
    /// Hidden (gathered or picked, waiting to grow back) or not placed for this player.
    Gone,
    Busy,
    TooFar,
    NoJob(Gather),
    WrongJob(Gather),
    /// No active quest has an open collect step for this pickup's node type (T503).
    NotNeeded,
}

impl Denied {
    pub fn message(self) -> String {
        match self {
            Self::Gone => "That has already been gathered. It will grow back soon.".into(),
            Self::Busy => "Finish what you are doing first.".into(),
            Self::TooFar => "Move closer to use that.".into(),
            // (R) T503 wording; the client has no reader for this refusal.
            Self::NotNeeded => "You don't need that right now.".into(),
            // Client text 705975744.
            Self::NoJob(Gather::Mining) => {
                "You need to be a Miner to use that. Visit Sorin in Snowhill to become a Miner!"
                    .into()
            }
            Self::NoJob(g) => format!("You need to be a {} to gather that.", g.word()),
            Self::WrongJob(g) => format!("Switch to the {} job to gather that.", g.word()),
        }
    }
}

/// What the clicking player is doing, for [`GatheringNodes::admit`]. A quest pickup has no
/// job gate, so [`GatheringNodes::admit_pickup`] takes only the shared reach and busy facts.
pub struct Visitor<'a> {
    pub zone: u32,
    pub position: [f32; 3],
    pub busy: bool,
    pub active_job: i32,
    pub unlocked_jobs: &'a BTreeSet<i32>,
}

/// Node actors and their state for one player (R: nodes are personal, so one player gathering
/// never hides a node for another, and nothing is shared between gateway sessions). Not saved:
/// a fresh login sees every node.
#[derive(Clone)]
pub struct GatheringNodes {
    sites: Arc<BTreeMap<u64, Site>>,
    shown: BTreeSet<u64>,
    /// Node of the running run, from `begin` until the run ends.
    current: Option<u64>,
    /// Paid nodes waiting for the run to close before they hide.
    paid: Vec<u64>,
    /// Hidden nodes and the time they return.
    regrow: BTreeMap<u64, u64>,
}

impl GatheringNodes {
    /// Reads `harvest_nodes.json` (Chef), `mining_nodes.json` (Miner) and `quest_nodes.json`
    /// (quest pickups, T503) from `dir`; `None` when none exists. Per pool only
    /// `max_active_nodes` spawn rows are placed, spread across the pool's rows (R: Legacy moves
    /// nodes between rows as they respawn; ours stay put). Nodes live in the open-world
    /// coordinate space whatever `zone_definition_id` says.
    pub fn load(dir: &Path) -> io::Result<Option<Self>> {
        let mut sites = BTreeMap::new();
        for (file, gather, tag) in NODE_FILES {
            let path = dir.join(file);
            if path.is_file() {
                let content = HarvestNodesContent::load_from(path)?;
                add_sites(&mut sites, &content, gather, tag)?;
            }
        }
        Ok((!sites.is_empty()).then(|| Self {
            sites: Arc::new(sites),
            shown: BTreeSet::new(),
            current: None,
            paid: Vec::new(),
            regrow: BTreeMap::new(),
        }))
    }

    /// Every node this player can ever see, shown or not.
    pub fn actors(&self) -> impl Iterator<Item = &NodeActor> {
        self.sites.values().map(|site| &site.actor)
    }

    pub fn is_node(&self, guid: u64) -> bool {
        self.sites.contains_key(&guid)
    }

    /// Ordinary entry check for a click on a gather node `guid` (the caller has checked
    /// `is_node`; a quest pickup never reaches here, see [`Self::admit_pickup`]).
    pub fn admit(&self, guid: u64, who: &Visitor) -> Result<Gather, Denied> {
        let site = self.admit_common(guid, who.zone, who.position, who.busy)?;
        let SiteKind::Gather(gather) = &site.kind else {
            return Err(Denied::NotNeeded);
        };
        let job = gather.job();
        if !who.unlocked_jobs.contains(&job) {
            return Err(Denied::NoJob(*gather));
        }
        if who.active_job != job {
            return Err(Denied::WrongJob(*gather));
        }
        Ok(*gather)
    }

    /// T503: the quest pickup admission rule (R). A pickup is admitted only while it is a
    /// placed pickup, shown, the player is not busy, the player is in zone 0 within the
    /// pool's range, and one of the player's active quests has an open collect step for its
    /// node type.
    pub fn admit_pickup(
        &self,
        guid: u64,
        zone: u32,
        position: [f32; 3],
        busy: bool,
        needed: bool,
    ) -> Result<(), Denied> {
        let site = self.admit_common(guid, zone, position, busy)?;
        if !matches!(site.kind, SiteKind::Pickup(_)) {
            return Err(Denied::Gone);
        }
        if !needed {
            return Err(Denied::NotNeeded);
        }
        Ok(())
    }

    /// The admission lines every node shares: placed, shown, not busy, in the open world and
    /// within the node's own interact range. The caller adds its kind rule.
    fn admit_common(
        &self,
        guid: u64,
        zone: u32,
        position: [f32; 3],
        busy: bool,
    ) -> Result<&Site, Denied> {
        let site = self.sites.get(&guid).ok_or(Denied::Gone)?;
        if !self.shown.contains(&guid) {
            return Err(Denied::Gone);
        }
        if busy {
            return Err(Denied::Busy);
        }
        if zone != 0 || !site.in_reach(position) {
            return Err(Denied::TooFar);
        }
        Ok(site)
    }

    /// The node type key of a quest pickup site, or `None` for a gather node.
    pub fn pickup_kind(&self, guid: u64) -> Option<&str> {
        match &self.sites.get(&guid)?.kind {
            SiteKind::Pickup(node_type) => Some(node_type),
            SiteKind::Gather(_) => None,
        }
    }

    /// T503: the node types of quest pickups that have at least one placed spawn. The
    /// enabled-quest reachability gate resolves `collect_node` targets against this set.
    pub fn pickup_kinds(&self) -> BTreeSet<&str> {
        self.sites
            .keys()
            .filter_map(|guid| self.pickup_kind(*guid))
            .collect()
    }

    /// T503: the player picked this quest node. It hides now and returns after the pool's
    /// respawn time, without the Type8 run or its paid bookkeeping. A node that is not shown
    /// changes nothing: no `Hide`, no new deadline.
    pub fn picked(&mut self, guid: u64, now_ms: u64) -> Option<NodeChange> {
        let site = self.sites.get(&guid)?;
        if !self.shown.remove(&guid) {
            return None;
        }
        self.regrow.insert(guid, now_ms + site.respawn_ms);
        Some(NodeChange::Hide(guid))
    }

    /// The run started from `guid` is under way.
    pub fn begin(&mut self, guid: u64) {
        self.current = Some(guid);
    }

    /// The run was paid: its node hides when the run closes and returns after the pool's respawn
    /// time. A payout with no node behind it (the developer commands) does nothing.
    pub fn paid(&mut self) {
        self.paid.extend(self.current.take());
    }

    /// Show every node not waiting to grow back, hide the paid ones once the player is idle, and
    /// bring back the ones whose time is up. Called each tick; returns what the world must draw.
    pub fn sync(&mut self, now_ms: u64, busy: bool) -> Vec<NodeChange> {
        if !busy {
            self.current = None; // quit, lost or dropped run: the node stays
            for guid in self.paid.drain(..) {
                if let Some(site) = self.sites.get(&guid) {
                    self.regrow.insert(guid, now_ms + site.respawn_ms);
                }
            }
        }
        self.regrow.retain(|_, ready| *ready > now_ms);
        let mut changes = Vec::new();
        for (guid, site) in self.sites.iter() {
            let want = !self.regrow.contains_key(guid);
            if want && self.shown.insert(*guid) {
                changes.push(NodeChange::Show(site.actor.clone()));
            } else if !want && self.shown.remove(guid) {
                changes.push(NodeChange::Hide(*guid));
            }
        }
        changes
    }
}

/// The world guid of one node: "NOD", the file's tag byte and the content spawn id. No two
/// live sources (files or T524's tag-4 effect companions) may share a tag, or two spawns with
/// the same id would get one guid.
fn node_guid(tag: u64, spawn_id: u32) -> u64 {
    NODE_GUID_BASE | tag << 32 | u64::from(spawn_id)
}

fn add_sites(
    sites: &mut BTreeMap<u64, Site>,
    content: &HarvestNodesContent,
    gather: Option<Gather>,
    tag: u64,
) -> io::Result<()> {
    let bad = |what: &str| io::Error::new(io::ErrorKind::InvalidData, what.to_owned());
    if content.schema_version != 1 {
        return Err(bad("unsupported node schema"));
    }
    // T503: the whole quest node file is quest-only; a drop table would be a second,
    // unaudited reward path beside the counted collect goal.
    if gather.is_none()
        && content
            .node_types
            .iter()
            .any(|kind| !kind.drop_table.is_empty())
    {
        return Err(bad("a quest node type has a drop table"));
    }
    if content
        .spawns
        .iter()
        .any(|spawn| !content.pools.iter().any(|pool| pool.key == spawn.pool_key))
    {
        return Err(bad("a node spawn names an unknown pool"));
    }
    for pool in &content.pools {
        let kind = content
            .node_types
            .iter()
            .find(|kind| kind.key == pool.node_type)
            .ok_or_else(|| bad("pool names an unknown node type"))?;
        if !(1..=50).contains(&kind.interact_range) || !(1..=86_400).contains(&pool.respawn_seconds)
        {
            return Err(bad("node range or respawn out of bounds"));
        }
        let rows: Vec<_> = content
            .spawns
            .iter()
            .filter(|spawn| spawn.pool_key == pool.key)
            .collect();
        let placed = (pool.max_active_nodes as usize).min(rows.len());
        for slot in 0..placed {
            let spawn = rows[slot * rows.len() / placed];
            if !spawn.position.iter().all(|v| v.is_finite()) {
                return Err(bad("node position is not finite"));
            }
            let guid = node_guid(tag, spawn.id);
            let [x, y, z] = spawn.position;
            let site = Site {
                actor: NodeActor {
                    guid,
                    name: kind.name.clone(),
                    model_id: kind.model_id,
                    position: [x, y + kind.placement_y_offset, z],
                },
                kind: match gather {
                    Some(gather) => SiteKind::Gather(gather),
                    None => SiteKind::Pickup(pool.node_type.clone()),
                },
                range: f64::from(kind.interact_range),
                respawn_ms: u64::from(pool.respawn_seconds) * 1000,
            };
            if sites.insert(guid, site).is_some() {
                return Err(bad("duplicate node spawn id"));
            }
        }
    }
    Ok(())
}

/// Start gate for the bag (R): a win that cannot be stored rolls back and strands the run, so
/// refuse the start unless a full-quota payout would fit now.
pub fn payout_fits(inventory: &Inventory, spec: &Type8BoardSpec) -> bool {
    let grants: Vec<(i32, u32)> = spec
        .outputs
        .iter()
        .filter_map(|output| {
            let goal = spec.goals.iter().find(|goal| goal.value == output.value)?;
            let clusters = (goal.quota / goal.per_reward.max(1)).max(1);
            Some((
                output.definition_id,
                clusters.saturating_mul(output.cluster_quantity),
            ))
        })
        .collect();
    inventory.clone().grant_bundle(0, &grants).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// T523 review: five collection spawns (100..=104) reuse quest pickup ids, and the old
    /// tag-3 plan would have made `add_sites` refuse the load as a duplicate spawn id. Every
    /// loader tag and the collection tag must stay in its own guid space; T524's tag-4 effect
    /// companions share that space, which `tests/node_guid_namespace.rs` checks on the loader.
    #[test]
    fn node_file_tags_give_distinct_guids_for_the_same_spawn_id() {
        let mut tags: Vec<u64> = NODE_FILES.iter().map(|(_, _, tag)| *tag).collect();
        // T524 wires the collection file into NODE_FILES at tag 5; only add the dormant file's
        // tag while the loader leaves it out.
        if !tags.contains(&COLLECTION_NODE_TAG) {
            tags.push(COLLECTION_NODE_TAG);
        }
        assert!(
            tags.iter().all(|tag| *tag < 256),
            "a tag must fit the guid's tag byte"
        );
        let guids: BTreeSet<u64> = tags.iter().map(|tag| node_guid(*tag, 100)).collect();
        assert_eq!(guids.len(), tags.len(), "two node files share a guid space");
    }

    /// The committed collection file loads beside the quest file on its own tag. The pool
    /// caps place quest spawn 103 and collection spawn 103, the one overlapping placed id.
    #[test]
    fn the_collection_file_loads_beside_the_quest_file() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let mut sites = BTreeMap::new();
        let quest = HarvestNodesContent::load_from(dir.join("quest_nodes.json")).unwrap();
        add_sites(&mut sites, &quest, None, QUEST_NODE_TAG).unwrap();
        let collection = HarvestNodesContent::load_from(dir.join("collection_nodes.json")).unwrap();
        // The site kind is irrelevant to the guid space; this runs `add_sites`' bounds.
        add_sites(
            &mut sites,
            &collection,
            Some(Gather::Harvesting),
            COLLECTION_NODE_TAG,
        )
        .expect("tag 4 keeps collection spawn 103 clear of quest spawn 103");
        assert!(sites.contains_key(&node_guid(QUEST_NODE_TAG, 103)));
        assert!(sites.contains_key(&node_guid(COLLECTION_NODE_TAG, 103)));
    }
}
