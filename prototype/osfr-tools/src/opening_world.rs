//! Native actors and interaction menus for the bounded opening content.
use crate::opening::{OpeningActor, OpeningEngine};
use crate::world::NpcVisibility;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, io, path::Path, sync::Arc};

/// Some opening actors stand exactly where a world NPC does, so both were drawn on one spot (owner
/// playtest 2026-11-05: two Bartles). The world NPC holds the quests, shops and trainers, the actor
/// the campaign's steps. A guide takes the world NPC's GUID so one drawn body serves both, and
/// `content_npc_key` gives a click to the campaign while it wants that actor. Resource actors stay
/// as they are: a world table and its tutorial table both work. Returns the GUIDs now shared.
pub fn share_world_npcs(
    opening: &mut OpeningEngine,
    visibility: &NpcVisibility,
) -> BTreeSet<u64> {
    let twins: Vec<_> = opening
        .actors()
        .iter()
        .filter(|actor| actor.kind == "guide")
        .filter_map(|actor| {
            let world = visibility.actor_at(actor.model_id, actor.position)?;
            Some((actor.guid, world))
        })
        .collect();
    for &(guid, world) in &twins {
        opening.rebind_actor(guid, world);
    }
    twins.into_iter().map(|(_, world)| world).collect()
}

#[derive(Deserialize)]
struct Template {
    /// T853: a local filename used by existing content; the release overlay leaves this empty.
    #[serde(default)]
    template_file: String,
    /// T853: the AddNpc skeleton as hex; when non-empty, `template_file` is not read.
    /// `template_sha256` pins the bytes either way.
    #[serde(default)]
    template_hex: String,
    template_sha256: String,
}
#[derive(Clone)]
pub struct OpeningWorld {
    template: Arc<Vec<u8>>,
}
fn invalid(text: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, text)
}

impl OpeningWorld {
    pub fn load(directory: &Path) -> io::Result<Self> {
        let descriptor: Template =
            serde_json::from_slice(&fs::read(directory.join("combat.json"))?)
                .map_err(|_| invalid("invalid actor template descriptor"))?;
        // T853/F860: one rule for the local file or the release's typed JSON hex.
        let bytes = crate::startup_snapshot::embedded_bytes(
            directory,
            &descriptor.template_file,
            &descriptor.template_hex,
            4096,
        )?;
        if bytes.len() != 334
            || bytes[..4] != [35, 0, 2, 0]
            || format!("{:x}", Sha256::digest(&bytes)) != descriptor.template_sha256
        {
            return Err(invalid("actor template differs from verified evidence"));
        }
        Ok(Self {
            template: Arc::new(bytes),
        })
    }

    pub fn initialization(&self, actors: &[OpeningActor], zone_id: u32) -> Vec<Vec<u8>> {
        let actors: Vec<_> = actors
            .iter()
            .filter(|a| a.zone_id == zone_id && a.kind != "enemy")
            .collect();
        let mut packets: Vec<_> = actors.iter().map(|a| self.spawn(a)).collect();
        let mut relevance = vec![35, 0, 12, 0];
        relevance.extend((actors.len() as u32).to_le_bytes());
        for actor in actors {
            relevance.extend(actor.guid.to_le_bytes());
            relevance.extend([1, 5, 0]);
        }
        packets.push(relevance);
        packets
    }

    pub fn spawn(&self, actor: &OpeningActor) -> Vec<u8> {
        let mut bytes = self.template.as_ref().clone();
        bytes[4..12].copy_from_slice(&actor.guid.to_le_bytes());
        bytes[12..16].copy_from_slice(&actor.name_id.to_le_bytes());
        bytes[16..20].copy_from_slice(&actor.model_id.to_le_bytes());
        for (i, value) in actor.position.iter().enumerate() {
            bytes[37 + i * 4..41 + i * 4].copy_from_slice(&value.to_le_bytes());
        }
        bytes[69..73].copy_from_slice(&0i32.to_le_bytes());
        bytes[77..81].copy_from_slice(&2i32.to_le_bytes());
        bytes[133..137].copy_from_slice(&(actor.interaction_radius.ceil() as i32).to_le_bytes());
        // Native proximity prompts read this separate floating-point range.
        bytes[237..241].copy_from_slice(&actor.interaction_radius.to_le_bytes());
        bytes.splice(
            106..110,
            (actor.name.len() as u32)
                .to_le_bytes()
                .into_iter()
                .chain(actor.name.bytes()),
        );
        bytes
    }

    pub fn menu(actor: &OpeningActor) -> Vec<u8> {
        let mut bytes = vec![26, 0, 9, 0];
        bytes.extend(actor.guid.to_le_bytes());
        bytes.push(0);
        bytes.extend(1i32.to_le_bytes());
        let label = if actor.kind == "npc" { 3300 } else { 432156 }; // verified English locale: Talk / Interact
        for value in [1i32, 0, label, 0, 0, 0, 0] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend((actor.name.len() as u32).to_le_bytes());
        bytes.extend(actor.name.bytes());
        bytes.extend([1, 0]);
        bytes
    }
}
