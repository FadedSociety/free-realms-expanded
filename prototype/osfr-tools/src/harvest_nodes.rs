//! Harvest-node content imported from the pinned Legacy data tables, and (W171) the ordinary
//! way into mining and harvesting: a node actor in the world that a player clicks.
//!
//! Recovered: a click on any interactable actor sends 26/8 (`PlayClient::RequestInteraction`,
//! `0092efe0`). Everything below that is reconstructed policy (R): the node is a plain actor,
//! 26/8 on it starts the Type8 run directly (as Legacy's collection nodes do, no menu), the
//! required job, the range, the per-player depletion and the respawn timers.

use crate::{
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

/// Chef profile id (`harvesting::CHEF_PROFILE_ID`).
const CHEF_PROFILE_ID: i32 = 45;
const MAX_FILE_BYTES: u64 = 1 << 20;
/// Node actor guids: "NODE" tag, game kind, content spawn id. Clear of NPC, placement and pickup ids.
const NODE_GUID_BASE: u64 = 0x4e4f_4445_0000_0000;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct HarvestNodesContent {
    pub schema_version: u32,
    #[serde(default)]
    pub provenance: BTreeMap<String, String>,
    pub node_types: Vec<HarvestNodeType>,
    pub pools: Vec<HarvestNodePool>,
    pub spawns: Vec<HarvestNodeSpawn>,
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
    pub interact_range: u32,
    pub cursor_id: u32,
    pub placement_y_offset: f32,
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

    fn word(self) -> &'static str {
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

struct Site {
    actor: NodeActor,
    gather: Gather,
    range: f64,
    respawn_ms: u64,
}

/// Why a click did not start a run. Every variant leaves the player and the node untouched.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Denied {
    /// Hidden (gathered, waiting to grow back) or not placed for this player.
    Gone,
    Busy,
    TooFar,
    NoJob(Gather),
    WrongJob(Gather),
}

impl Denied {
    pub fn message(self) -> String {
        match self {
            Self::Gone => "That has already been gathered. It will grow back soon.".into(),
            Self::Busy => "Finish what you are doing first.".into(),
            Self::TooFar => "Move closer to use that.".into(),
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

/// What the clicking player is doing, for [`GatheringNodes::admit`].
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
    /// Reads `harvest_nodes.json` (Chef) and `mining_nodes.json` (Miner) from `dir`; `None` when
    /// neither exists. Per pool only `max_active_nodes` spawn rows are placed, spread across the
    /// pool's rows (R: Legacy moves nodes between rows as they respawn; ours stay put). Nodes
    /// live in the open-world coordinate space whatever `zone_definition_id` says.
    pub fn load(dir: &Path) -> io::Result<Option<Self>> {
        let mut sites = BTreeMap::new();
        for (file, gather, tag) in [
            ("harvest_nodes.json", Gather::Harvesting, 1_u64),
            ("mining_nodes.json", Gather::Mining, 2),
        ] {
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

    /// Ordinary entry check for a click on node `guid` (the caller has checked `is_node`).
    pub fn admit(&self, guid: u64, who: &Visitor) -> Result<Gather, Denied> {
        let site = self.sites.get(&guid).ok_or(Denied::Gone)?;
        if !self.shown.contains(&guid) {
            return Err(Denied::Gone);
        }
        if who.busy {
            return Err(Denied::Busy);
        }
        let reach = site
            .actor
            .position
            .iter()
            .zip(who.position)
            .map(|(a, b)| (f64::from(*a) - f64::from(b)).powi(2))
            .sum::<f64>();
        if who.zone != 0 || !reach.is_finite() || reach > site.range * site.range {
            return Err(Denied::TooFar);
        }
        let job = site.gather.job();
        if !who.unlocked_jobs.contains(&job) {
            return Err(Denied::NoJob(site.gather));
        }
        if who.active_job != job {
            return Err(Denied::WrongJob(site.gather));
        }
        Ok(site.gather)
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

fn add_sites(
    sites: &mut BTreeMap<u64, Site>,
    content: &HarvestNodesContent,
    gather: Gather,
    tag: u64,
) -> io::Result<()> {
    let bad = |what: &str| io::Error::new(io::ErrorKind::InvalidData, what.to_owned());
    if content.schema_version != 1 {
        return Err(bad("unsupported node schema"));
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
            let guid = NODE_GUID_BASE | tag << 32 | u64::from(spawn.id);
            let [x, y, z] = spawn.position;
            let site = Site {
                actor: NodeActor {
                    guid,
                    name: kind.name.clone(),
                    model_id: kind.model_id,
                    position: [x, y + kind.placement_y_offset, z],
                },
                gather,
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
