//! World Builder text: strings authored in-game get stable locale ids.
//!
//! The client shows quest titles, descriptions, objective text and NPC names
//! by locale id (`locale/en_us_data.dat`), so new text needs new locale rows.
//! The server hands out ids from a reserved range and keeps them in the live
//! world's `text.json`; `scripts/build_world_locale.py` writes them into a
//! client's locale pair with the pinned MIT FreeRealmsLocaleTools. Players see
//! new text after their client restarts with the rebuilt pair.
//!
//! A locale row is keyed by JenkinsLookup2("Global.Text.<id>") (Bob Jenkins'
//! public-domain lookup2, as FreeRealmsLocaleTools `Preimaging.GetHash`), so an
//! id is usable when its hash is not already a row in the stock file.
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    io::{self, BufRead},
    path::{Path, PathBuf},
};

/// (R) Authored ids start here, above every stock id (max 5,103,267) and the
/// opening journal labels (619..1236).
pub const FIRST_WORLD_TEXT_ID: i32 = 7_000_000;
const MAX_TEXT_CHARS: usize = 1000;

fn mix(mut a: u32, mut b: u32, mut c: u32) -> (u32, u32, u32) {
    a = a.wrapping_sub(b).wrapping_sub(c) ^ (c >> 13);
    b = b.wrapping_sub(c).wrapping_sub(a) ^ (a << 8);
    c = c.wrapping_sub(a).wrapping_sub(b) ^ (b >> 13);
    a = a.wrapping_sub(b).wrapping_sub(c) ^ (c >> 12);
    b = b.wrapping_sub(c).wrapping_sub(a) ^ (a << 16);
    c = c.wrapping_sub(a).wrapping_sub(b) ^ (b >> 5);
    a = a.wrapping_sub(b).wrapping_sub(c) ^ (c >> 3);
    b = b.wrapping_sub(c).wrapping_sub(a) ^ (a << 10);
    c = c.wrapping_sub(a).wrapping_sub(b) ^ (b >> 15);
    (a, b, c)
}

/// Bob Jenkins' lookup2 with an initial value of 0.
pub fn jenkins_lookup2(key: &[u8]) -> u32 {
    let word = |p: &[u8]| u32::from_le_bytes(p[..4].try_into().unwrap());
    let (mut a, mut b, mut c) = (0x9e37_79b9_u32, 0x9e37_79b9_u32, 0_u32);
    let mut rest = key;
    while rest.len() >= 12 {
        a = a.wrapping_add(word(&rest[0..]));
        b = b.wrapping_add(word(&rest[4..]));
        c = c.wrapping_add(word(&rest[8..]));
        (a, b, c) = mix(a, b, c);
        rest = &rest[12..];
    }
    c = c.wrapping_add(key.len() as u32);
    for (i, shift) in [(10, 24), (9, 16), (8, 8)] {
        if rest.len() > i {
            c = c.wrapping_add(u32::from(rest[i]) << shift);
        }
    }
    for (i, shift) in [(7, 24), (6, 16), (5, 8), (4, 0)] {
        if rest.len() > i {
            b = b.wrapping_add(u32::from(rest[i]) << shift);
        }
    }
    for (i, shift) in [(3, 24), (2, 16), (1, 8), (0, 0)] {
        if rest.len() > i {
            a = a.wrapping_add(u32::from(rest[i]) << shift);
        }
    }
    mix(a, b, c).2
}

/// The locale row key of a text id.
pub fn locale_hash(id: i32) -> u32 {
    jenkins_lookup2(format!("Global.Text.{id}").as_bytes())
}

/// Row keys already used by a locale `.dat` (first column of each row).
pub fn stock_hashes(dat: &Path) -> io::Result<HashSet<u32>> {
    let file = fs::File::open(dat)?;
    let mut hashes = HashSet::new();
    for line in io::BufReader::new(file).split(b'\n') {
        let line = line?;
        let line = line.strip_prefix(b"\xef\xbb\xbf").unwrap_or(&line);
        let key = line.split(|b| *b == b'\t').next().unwrap_or_default();
        if let Some(hash) = std::str::from_utf8(key).ok().and_then(|k| k.parse::<u32>().ok()) {
            hashes.insert(hash);
        }
    }
    Ok(hashes)
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TextEntry {
    pub id: i32,
    pub text: String,
    #[serde(default)]
    pub author: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TextFile {
    schema_version: u32,
    next_id: i32,
    entries: Vec<TextEntry>,
}

/// Authored strings of one live world (`text.json`).
#[derive(Debug)]
pub struct TextPool {
    path: PathBuf,
    next_id: i32,
    pub entries: Vec<TextEntry>,
}

impl TextPool {
    pub fn load(dir: &Path) -> io::Result<Self> {
        let path = dir.join("text.json");
        if !path.is_file() {
            return Ok(Self {
                path,
                next_id: FIRST_WORLD_TEXT_ID,
                entries: Vec::new(),
            });
        }
        let file: TextFile = serde_json::from_slice(&fs::read(&path)?)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("text.json: {e}")))?;
        Ok(Self {
            path,
            next_id: file.next_id.max(FIRST_WORLD_TEXT_ID),
            entries: file.entries,
        })
    }

    pub fn get(&self, id: i32) -> Option<&str> {
        self.entries.iter().find(|e| e.id == id).map(|e| e.text.as_str())
    }

    /// The id for `text`: an existing one for identical text, else the next
    /// id whose locale hash no stock row uses. Saves the file.
    pub fn add(
        &mut self,
        text: &str,
        author: Option<&str>,
        stock: Option<&HashSet<u32>>,
    ) -> io::Result<i32> {
        let text = text.trim();
        if text.is_empty() || text.chars().count() > MAX_TEXT_CHARS || text.contains(['\t', '\r', '\n']) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("text must be 1..{MAX_TEXT_CHARS} characters on one line"),
            ));
        }
        if let Some(entry) = self.entries.iter().find(|e| e.text == text) {
            return Ok(entry.id);
        }
        let mut id = self.next_id;
        while stock.is_some_and(|hashes| hashes.contains(&locale_hash(id))) {
            id += 1;
        }
        self.entries.push(TextEntry {
            id,
            text: text.to_owned(),
            author: author.map(str::to_owned),
        });
        self.next_id = id + 1;
        self.save()?;
        Ok(id)
    }

    fn save(&self) -> io::Result<()> {
        let file = TextFile {
            schema_version: 1,
            next_id: self.next_id,
            entries: self.entries.clone(),
        };
        let text = serde_json::to_string_pretty(&file)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        fs::write(&self.path, text + "\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_matches_the_python_and_community_tool_values() {
        assert_eq!(jenkins_lookup2(b""), 3_175_731_469);
        assert_eq!(locale_hash(7_000_000), 9_863_440);
        assert_eq!(locale_hash(15_623), 984_541_795);
    }

    #[test]
    fn pool_reuses_text_skips_stock_hashes_and_persists() {
        let dir = std::env::temp_dir().join(format!("osfr-text-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let stock = HashSet::from([locale_hash(FIRST_WORLD_TEXT_ID)]);
        let mut pool = TextPool::load(&dir).unwrap();
        let hunt = pool.add("Find Bob!", Some("the owner"), Some(&stock)).unwrap();
        assert_eq!(hunt, FIRST_WORLD_TEXT_ID + 1, "skips an id whose hash is a stock row");
        assert_eq!(pool.add("  Find Bob! ", None, Some(&stock)).unwrap(), hunt, "same text, same id");
        let other = pool.add("Bob is hiding near the bridge.", None, None).unwrap();
        assert_eq!(other, hunt + 1);
        assert!(pool.add("two\nlines", None, None).is_err());
        let reloaded = TextPool::load(&dir).unwrap();
        assert_eq!(reloaded.get(hunt), Some("Find Bob!"));
        assert_eq!(reloaded.entries[0].author.as_deref(), Some("the owner"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn stock_hashes_read_the_first_column() {
        let dir = std::env::temp_dir().join(format!("osfr-dat-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let dat = dir.join("en_us_data.dat");
        fs::write(&dat, "\u{feff}42093\tugdt\tLevel 15\n63491\tucdt\tBryan\nbad row\n").unwrap();
        assert_eq!(stock_hashes(&dat).unwrap(), HashSet::from([42_093, 63_491]));
        fs::remove_dir_all(&dir).unwrap();
    }
}
