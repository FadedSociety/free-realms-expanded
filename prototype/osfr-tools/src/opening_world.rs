//! Native actors and interaction menus for the bounded opening content.
use crate::opening::OpeningActor;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    fs, io,
    path::{Component, Path},
    sync::Arc,
};

#[derive(Deserialize)]
struct Template {
    template_file: String,
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
        let filename = Path::new(&descriptor.template_file);
        if filename.components().count() != 1
            || !matches!(filename.components().next(), Some(Component::Normal(_)))
        {
            return Err(invalid("actor template must use a local filename"));
        }
        let bytes = fs::read(directory.join(filename))?;
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
