//! Item category reference data, using the community 44/2 wire description.
//! Private tables and serializer-oracle output remain outside the repository.
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::File,
    io::{self, Read},
    path::Path,
};

const MAX_CONTENT: usize = 256 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Category {
    id: i32,
    name_id: i32,
    icon_id: i32,
    unknown: i32,
    unknown2: bool,
    display: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Group {
    id: i32,
    category_id: i32,
}

#[derive(Deserialize)]
struct Content {
    schema_version: u32,
    categories: Vec<Category>,
    groups: Vec<Group>,
    packet_sha256: String,
}

#[derive(Clone)]
pub struct ItemCategories {
    packet: Vec<u8>,
}

fn invalid(message: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn put(bytes: &mut Vec<u8>, value: i32) {
    bytes.extend(value.to_le_bytes());
}

impl ItemCategories {
    pub fn load(directory: &Path) -> io::Result<Self> {
        let file = File::open(directory.join("categories.json"))?;
        if !file.metadata()?.is_file() {
            return Err(invalid("item categories must be a regular file"));
        }
        let mut bytes = Vec::new();
        file.take((MAX_CONTENT + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_CONTENT {
            return Err(invalid("item category content exceeds 256 KiB"));
        }
        let content: Content = serde_json::from_slice(&bytes).map_err(invalid)?;
        if content.schema_version != 1
            || content.categories.is_empty()
            || content.categories.len() > 512
            || content.groups.len() > 2048
        {
            return Err(invalid("unsupported item category content"));
        }
        let mut packet = [44u16.to_le_bytes(), 2u16.to_le_bytes()].concat();
        put(&mut packet, content.categories.len() as i32);
        let mut ids = BTreeSet::new();
        for category in content.categories {
            if category.id <= 0
                || category.name_id <= 0
                || category.icon_id < 0
                || category.unknown < 0
                || category.display < 0
                || !ids.insert(category.id)
            {
                return Err(invalid("invalid or duplicate item category"));
            }
            // PacketWriter.Write(dictionary) emits the dictionary key, then
            // the category value's own Id field. Both are present on the wire.
            put(&mut packet, category.id);
            for value in [
                category.id,
                category.name_id,
                category.icon_id,
                category.unknown,
            ] {
                put(&mut packet, value);
            }
            packet.push(u8::from(category.unknown2));
            put(&mut packet, category.display);
        }
        put(&mut packet, content.groups.len() as i32);
        let mut pairs = BTreeSet::new();
        // Dictionary iteration groups rows by group ID while retaining each
        // ID's first occurrence and the order of its category associations.
        let mut order = Vec::new();
        for group in &content.groups {
            if group.id <= 0
                || group.category_id <= 0
                || !pairs.insert((group.id, group.category_id))
            {
                return Err(invalid("invalid or duplicate item category group"));
            }
            if !order.contains(&group.id) {
                order.push(group.id);
            }
        }
        for id in order {
            for group in content.groups.iter().filter(|group| group.id == id) {
                // The pinned tables intentionally include several unmapped
                // category IDs; preserve those references as upstream does.
                put(&mut packet, id);
                put(&mut packet, group.id);
                put(&mut packet, group.category_id);
            }
        }
        if format!("{:x}", Sha256::digest(&packet)) != content.packet_sha256 {
            return Err(invalid("item category packet differs from prepared proof"));
        }
        Ok(Self { packet })
    }

    /// Inner application packet; the gateway supplies the tunnel envelope.
    pub fn initialization(&self) -> Vec<u8> {
        self.packet.clone()
    }
}
