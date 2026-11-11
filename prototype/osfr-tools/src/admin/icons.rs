//! Item icons for the GM panel (`GET /admin/icon/<item_id>`), read from the server operator's
//! client install and never stored in the repository.
//!
//! An item definition's fourth field is its icon *image set* ([`ICON_FIELD`]). `Resources/Images/ImageSetMappings.txt` gives the set's image id per kind
//! and `Images.txt` the image's file name, a `.dds` inside the `Assets_NNN.pack` files. The pack
//! directory and DDS readers follow this project's Rust client (`client-rs` `fr-formats`
//! `pack.rs` and `dds.rs`, AGPL-3.0), cut down to what an icon needs: the top mip, decoded to RGBA
//! and written as a PNG.
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

/// Byte offset of the icon image set in a serialized item definition (id, name, description,
/// icon: `gm_panel::item_rows`, the client reader `008e3760`).
const ICON_FIELD: usize = 12;
/// Image-set kinds tried in order: medium (the 64 px item icon), large, small (`ImageSetTypes.txt`).
const KINDS: [u32; 3] = [6, 7, 5];
/// (R) Bounds: an icon file, the PNGs kept in memory, a pack name.
const MAX_ICON_BYTES: u32 = 4 * 1024 * 1024;
const MAX_CACHED: usize = 4096;
const MAX_NAME: usize = 1024;

struct Entry {
    pack: usize,
    offset: u32,
    len: u32,
}

#[derive(Default)]
struct Index {
    /// Item definition id -> icon image set.
    items: HashMap<i32, u32>,
    /// Image id -> file name.
    files: HashMap<u32, String>,
    /// (image set, kind) -> image id.
    sets: HashMap<(u32, u32), u32>,
    packs: Vec<PathBuf>,
    /// Lower-case file name -> where it lives; the first pack that has a name wins.
    entries: HashMap<String, Entry>,
}

/// The install's icons, indexed on first use and cached as PNG bytes.
pub struct Icons {
    install: PathBuf,
    content: Option<PathBuf>,
    index: OnceLock<Index>,
    cache: Mutex<HashMap<i32, Option<Arc<[u8]>>>>,
    /// Zone name -> its whole map picture and the world box it covers.
    maps: Mutex<HashMap<String, Option<ZoneMap>>>,
}

/// A zone's map as one PNG, north up, and the world box `[min_x, min_z, max_x, max_z]` it covers.
pub type ZoneMap = (Arc<[u8]>, [f32; 4]);

impl Icons {
    /// `install` is the client folder (`FreeRealms.exe`, `Resources/`, the packs); `content` the
    /// gateway's content folder, whose item tables name each item's icon.
    pub fn new(install: PathBuf, content: Option<PathBuf>) -> Self {
        Self {
            install,
            content,
            index: OnceLock::new(),
            cache: Mutex::new(HashMap::new()),
            maps: Mutex::new(HashMap::new()),
        }
    }

    /// The item's icon as a PNG, or `None` when the item, its image or the file is missing.
    pub fn png(&self, item_id: i32) -> Option<Arc<[u8]>> {
        if let Some(cached) = self.lock().get(&item_id) {
            return cached.clone();
        }
        let index = self.index.get_or_init(|| self.build());
        let png = index.icon(item_id).map(Arc::from);
        let mut cache = self.lock();
        if cache.len() >= MAX_CACHED {
            cache.clear(); // ponytail: drop all at the cap; an LRU if the panel ever churns it
        }
        cache.insert(item_id, png.clone());
        png
    }

    /// A zone's map tiles from `<zone>_TileInfo.txt` in the packs, read the way this project's
    /// Rust client reads them (`client-rs` `fr-ui` `minimap.rs`); empty for a zone with no map.
    pub fn map_tiles(&self, zone: &str) -> Vec<MapTile> {
        if zone.is_empty() || !zone.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
            return Vec::new();
        }
        let index = self.index.get_or_init(|| self.build());
        let Some(list) = index.read(&format!("{zone}_TileInfo.txt")) else {
            return Vec::new();
        };
        String::from_utf8_lossy(&list)
            .lines()
            .filter_map(|line| {
                let mut fields = line.split('\t').map(str::trim);
                let name = fields.next().filter(|name| !name.is_empty())?.to_owned();
                let mut number = || fields.next()?.parse::<f32>().ok().filter(|n| n.is_finite());
                let (x, _y, z, width, height) = (number()?, number()?, number()?, number()?, number()?);
                (width > 0.0 && height > 0.0).then_some(MapTile { name, x, z, width, height })
            })
            .collect()
    }

    /// The zone's whole map in one picture, north (+x) up and east (+z) right as the game's map
    /// lays its tiles, at most `side` pixels on its longer edge, each pixel sampled from the tile
    /// under it; made on first use and kept. `None` for a zone with no map.
    pub fn zone_map(&self, zone: &str, side: u32) -> Option<ZoneMap> {
        let key = zone.to_ascii_lowercase();
        // ponytail: the lock is held while a map is made (seconds, once per zone), so the panel's
        // 2 s polls wait for the one picture instead of each making it; per-zone slots if that ever matters.
        let mut maps = self.maps.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(made) = maps.get(&key) {
            return made.clone();
        }
        let made = self.compose(zone, side);
        maps.insert(key, made.clone());
        made
    }

    fn compose(&self, zone: &str, side: u32) -> Option<ZoneMap> {
        let tiles = self.map_tiles(zone);
        let index = self.index.get_or_init(|| self.build());
        let low = |pick: fn(&MapTile) -> f32| tiles.iter().map(pick).reduce(f32::min);
        let high = |pick: fn(&MapTile) -> f32| tiles.iter().map(pick).reduce(f32::max);
        let (min_x, max_x) = (low(|t| t.x)?, high(|t| t.x + t.height)?);
        let (min_z, max_z) = (low(|t| t.z)?, high(|t| t.z + t.width)?);
        let per_metre = side as f32 / (max_x - min_x).max(max_z - min_z);
        let (width, height) = (
            ((max_z - min_z) * per_metre).ceil().clamp(1.0, side as f32) as usize,
            ((max_x - min_x) * per_metre).ceil().clamp(1.0, side as f32) as usize,
        );
        let mut rgba = vec![0u8; width * height * 4];
        for tile in &tiles {
            let Some((tw, th, pixels)) = index.read(&tile.name).and_then(|bytes| decode_dds(&bytes)) else {
                continue;
            };
            let (tw, th) = (tw as usize, th as usize);
            if tw == 0 || th == 0 || pixels.len() < tw * th * 4 {
                continue;
            }
            // Picture rows and columns whose middles fall in the tile.
            let span = |from: f32, to: f32, end: usize| {
                let first = (from * per_metre - 0.5).ceil().max(0.0) as usize;
                first..((to * per_metre - 0.5).ceil().max(0.0) as usize).min(end)
            };
            let rows = span(max_x - (tile.x + tile.height), max_x - tile.x, height);
            for row in rows {
                let wx = max_x - (row as f32 + 0.5) / per_metre;
                let ty = (((tile.x + tile.height - wx) / tile.height * th as f32) as usize).min(th - 1);
                for col in span(tile.z - min_z, tile.z + tile.width - min_z, width) {
                    let wz = min_z + (col as f32 + 0.5) / per_metre;
                    let tx = (((wz - tile.z) / tile.width * tw as f32) as usize).min(tw - 1);
                    let (from, to) = ((ty * tw + tx) * 4, (row * width + col) * 4);
                    rgba[to..to + 4].copy_from_slice(&pixels[from..from + 4]);
                }
            }
        }
        let png = encode_png(width as u32, height as u32, &rgba);
        Some((Arc::from(png), [min_x, min_z, max_x, max_z]))
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<i32, Option<Arc<[u8]>>>> {
        self.cache.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Everything the lookups need. A missing or damaged part leaves its lookups empty, so the
    /// route answers 404 rather than failing the panel.
    fn build(&self) -> Index {
        let mut index = Index::default();
        if let Some(inventory) = self
            .content
            .as_deref()
            .and_then(|dir| crate::inventory::Inventory::load(dir, None).ok())
        {
            for id in inventory.definition_ids() {
                if let Some(icon) = inventory
                    .definition_record(id)
                    .and_then(|record| crate::le::u32(record, ICON_FIELD))
                {
                    index.items.insert(id, icon);
                }
            }
        }
        let images = self.install.join("Resources").join("Images");
        for row in table(&images.join("Images.txt")) {
            if let [id, file, ..] = row.as_slice()
                && let Ok(id) = id.parse()
            {
                index.files.insert(id, file.clone());
            }
        }
        for row in table(&images.join("ImageSetMappings.txt")) {
            if let [set, kind, image, ..] = row.as_slice()
                && let (Ok(set), Ok(kind), Ok(image)) = (set.parse(), kind.parse(), image.parse())
            {
                index.sets.insert((set, kind), image);
            }
        }
        for prefix in ["Assets", "AssetsW"] {
            for number in 0..1000 {
                let path = self.install.join(format!("{prefix}_{number:03}.pack"));
                let Ok(mut file) = File::open(&path) else { break };
                let pack = index.packs.len();
                let entries = &mut index.entries;
                let read = read_directory(&mut file, |name, offset, len| {
                    entries.entry(name).or_insert(Entry { pack, offset, len });
                });
                if let Err(error) = read {
                    eprintln!("admin icons: {}: {error}", path.display());
                }
                index.packs.push(path);
            }
        }
        index
    }
}

impl Index {
    fn icon(&self, item_id: i32) -> Option<Vec<u8>> {
        let set = *self.items.get(&item_id)?;
        let file = KINDS
            .iter()
            .find_map(|kind| self.files.get(self.sets.get(&(set, *kind))?))?;
        let (width, height, rgba) = decode_dds(&self.read(file)?)?;
        Some(encode_png(width, height, &rgba))
    }

    /// One file of the packs by name (any case), up to the size bound.
    fn read(&self, file: &str) -> Option<Vec<u8>> {
        let entry = self.entries.get(&file.to_ascii_lowercase())?;
        if entry.len > MAX_ICON_BYTES {
            return None;
        }
        let mut pack = File::open(self.packs.get(entry.pack)?).ok()?;
        pack.seek(SeekFrom::Start(u64::from(entry.offset))).ok()?;
        let mut bytes = vec![0; entry.len as usize];
        pack.read_exact(&mut bytes).ok()?;
        Some(bytes)
    }
}

/// One picture of a zone's map and the world square it covers: its south-west corner at world
/// (`x`, `z`), `height` metres along +x (north) and `width` along +z (east).
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct MapTile {
    pub name: String,
    pub x: f32,
    pub z: f32,
    pub width: f32,
    pub height: f32,
}

/// A caret table (`#` header lines skipped); empty when the file cannot be read.
fn table(path: &Path) -> Vec<Vec<String>> {
    let Ok(bytes) = std::fs::read(path) else {
        return Vec::new();
    };
    String::from_utf8_lossy(&bytes)
        .lines()
        .filter(|line| !line.starts_with('#'))
        .map(|line| line.split('^').map(str::to_owned).collect())
        .collect()
}

fn be32(reader: &mut impl Read) -> io::Result<u32> {
    let mut raw = [0; 4];
    reader.read_exact(&mut raw)?;
    Ok(u32::from_be_bytes(raw))
}

/// One pack's directory: a forward chain of blocks (next block offset, entry count, then per
/// entry a length-prefixed name, offset, length and CRC-32, all big-endian).
fn read_directory(
    source: &mut (impl Read + Seek),
    mut found: impl FnMut(String, u32, u32),
) -> io::Result<()> {
    let invalid = |what| io::Error::new(io::ErrorKind::InvalidData, what);
    let len = source.seek(SeekFrom::End(0))?;
    let mut block = 0u64;
    for _ in 0..1 << 20 {
        source.seek(SeekFrom::Start(block))?;
        let mut reader = BufReader::with_capacity(16 << 10, &mut *source);
        let next = u64::from(be32(&mut reader)?);
        for _ in 0..be32(&mut reader)? {
            let name_len = be32(&mut reader)? as usize;
            if name_len > MAX_NAME {
                return Err(invalid("pack entry name length"));
            }
            let mut name = vec![0; name_len];
            reader.read_exact(&mut name)?;
            let (offset, size, _crc) = (be32(&mut reader)?, be32(&mut reader)?, be32(&mut reader)?);
            if u64::from(offset) + u64::from(size) > len {
                return Err(invalid("pack entry extent"));
            }
            found(String::from_utf8_lossy(&name).to_lowercase(), offset, size);
        }
        if next == 0 {
            return Ok(());
        }
        if next <= block || next >= len {
            return Err(invalid("pack block chain"));
        }
        block = next;
    }
    Err(invalid("pack block chain"))
}

fn rgb565(c: u16) -> [u8; 3] {
    let (r, g, b) = (u32::from(c >> 11), u32::from((c >> 5) & 63), u32::from(c & 31));
    [((r * 527 + 23) >> 6) as u8, ((g * 259 + 33) >> 6) as u8, ((b * 527 + 23) >> 6) as u8]
}

/// The top mip of a DDS file as RGBA: DXT1, DXT3, DXT5 or plain 32-bit BGRA/RGBA by masks.
fn decode_dds(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    if bytes.get(..4) != Some(b"DDS ") || crate::le::u32(bytes, 4) != Some(124) {
        return None;
    }
    let (height, width) = (crate::le::u32(bytes, 12)?, crate::le::u32(bytes, 16)?);
    if !(1..=1024).contains(&width) || !(1..=1024).contains(&height) {
        return None;
    }
    let (flags, four_cc, bits) = (crate::le::u32(bytes, 80)?, bytes.get(84..88)?, crate::le::u32(bytes, 88)?);
    let data = bytes.get(128..)?;
    let (w, h) = (width as usize, height as usize);
    let mut out = vec![0u8; w * h * 4];
    if flags & 4 == 0 {
        // Plain pixels: only the 32-bit layouts icons use.
        let masks = [crate::le::u32(bytes, 92)?, crate::le::u32(bytes, 96)?, crate::le::u32(bytes, 100)?, crate::le::u32(bytes, 104)?];
        if bits != 32 {
            return None;
        }
        let channel = |value: u32, mask: u32, absent: u8| {
            if mask == 0 {
                return absent;
            }
            let shift = mask.trailing_zeros();
            (u64::from((value & mask) >> shift) * 255 / u64::from(mask >> shift)) as u8
        };
        for (pixel, out) in data.get(..w * h * 4)?.chunks_exact(4).zip(out.chunks_exact_mut(4)) {
            let value = u32::from_le_bytes([pixel[0], pixel[1], pixel[2], pixel[3]]);
            out.copy_from_slice(&[
                channel(value, masks[0], 0),
                channel(value, masks[1], 0),
                channel(value, masks[2], 0),
                channel(value, masks[3], 255),
            ]);
        }
        return Some((width, height, out));
    }
    let block_len = match four_cc {
        b"DXT1" => 8,
        b"DXT3" | b"DXT5" => 16,
        _ => return None,
    };
    let across = w.div_ceil(4);
    let blocks = data.get(..across * h.div_ceil(4) * block_len)?;
    for (index, block) in blocks.chunks_exact(block_len).enumerate() {
        let (bx, by) = (index % across * 4, index / across * 4);
        let colour = &block[block_len - 8..];
        let (c0, c1) = (
            u16::from_le_bytes([colour[0], colour[1]]),
            u16::from_le_bytes([colour[2], colour[3]]),
        );
        let (a, b) = (rgb565(c0), rgb565(c1));
        let mix = |wa: u16, wb: u16| -> [u8; 4] {
            let m = |x: u8, y: u8| ((u16::from(x) * wa + u16::from(y) * wb) / (wa + wb)) as u8;
            [m(a[0], b[0]), m(a[1], b[1]), m(a[2], b[2]), 255]
        };
        let palette = if block_len == 8 && c0 <= c1 {
            [[a[0], a[1], a[2], 255], [b[0], b[1], b[2], 255], mix(1, 1), [0; 4]]
        } else {
            [[a[0], a[1], a[2], 255], [b[0], b[1], b[2], 255], mix(2, 1), mix(1, 2)]
        };
        let picks = u32::from_le_bytes([colour[4], colour[5], colour[6], colour[7]]);
        let (a0, a1) = (u16::from(block[0]), u16::from(block[1]));
        let mut ramp = [block[0], block[1], 0, 0, 0, 0, 0, 255];
        let steps = if a0 > a1 { 7 } else { 5 };
        for i in 1..steps {
            ramp[i as usize + 1] = (((steps - i) * a0 + i * a1) / steps) as u8;
        }
        let alpha_bits = block[2..8].iter().rev().fold(0u64, |bits, byte| (bits << 8) | u64::from(*byte));
        for k in 0..16 {
            let (x, y) = (bx + k % 4, by + k / 4);
            if x >= w || y >= h {
                continue;
            }
            let mut pixel = palette[((picks >> (2 * k)) & 3) as usize];
            match four_cc {
                b"DXT3" => pixel[3] = ((block[k / 2] >> (4 * (k % 2))) & 15) * 17,
                b"DXT5" => pixel[3] = ramp[((alpha_bits >> (3 * k)) & 7) as usize],
                _ => {}
            }
            out[(y * w + x) * 4..][..4].copy_from_slice(&pixel);
        }
    }
    Some((width, height, out))
}

fn png_chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend((data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend(kind);
    out.extend(data);
    let crc = crate::portrait_upload::crc32(&out[start..]);
    out.extend(crc.to_be_bytes());
}

/// An 8-bit RGBA PNG, every row unfiltered.
fn encode_png(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    let stride = width as usize * 4;
    let mut raw = Vec::with_capacity((stride + 1) * height as usize);
    for row in rgba.chunks_exact(stride) {
        raw.push(0);
        raw.extend_from_slice(row);
    }
    let mut header = width.to_be_bytes().to_vec();
    header.extend(height.to_be_bytes());
    header.extend([8, 6, 0, 0, 0]);
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    png_chunk(&mut out, b"IHDR", &header);
    png_chunk(&mut out, b"IDAT", &miniz_oxide::deflate::compress_to_vec_zlib(&raw, 6));
    png_chunk(&mut out, b"IEND", &[]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 4x4 DXT1 file: red and blue endpoints, picks 0..3 along the first row.
    fn dxt1() -> Vec<u8> {
        let mut file = b"DDS ".to_vec();
        for value in [124, 0, 4, 4, 0, 0, 1] {
            file.extend(u32::to_le_bytes(value));
        }
        file.resize(76, 0);
        file.extend(32u32.to_le_bytes());
        file.extend(4u32.to_le_bytes());
        file.extend(b"DXT1");
        file.resize(128, 0);
        file.extend([0x00, 0xf8, 0x1f, 0x00, 0b1110_0100, 0, 0, 0]);
        file
    }

    /// An install with one pack holding `icon.dds` for image set 7 (`Images.txt` id 70, kind 6).
    fn install(dir: &Path) {
        let images = dir.join("Resources").join("Images");
        std::fs::create_dir_all(&images).unwrap();
        std::fs::write(images.join("Images.txt"), "#ID^FILE^\r\n70^Icon.dds^0^\r\n").unwrap();
        std::fs::write(images.join("ImageSetMappings.txt"), "#SET^TYPE^IMAGE^\r\n7^6^70^\r\n").unwrap();
        let name = b"icon.dds";
        let mut pack = 0u32.to_be_bytes().to_vec();
        pack.extend(1u32.to_be_bytes());
        pack.extend((name.len() as u32).to_be_bytes());
        pack.extend(name);
        let offset = pack.len() as u32 + 12;
        let file = dxt1();
        pack.extend(offset.to_be_bytes());
        pack.extend((file.len() as u32).to_be_bytes());
        pack.extend(0u32.to_be_bytes());
        pack.extend(file);
        std::fs::write(dir.join("Assets_000.pack"), pack).unwrap();
    }

    #[test]
    fn a_pack_icon_decodes_to_a_valid_png() {
        let root = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join(format!("admin-icons-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        install(&root);
        let icons = Icons::new(root.clone(), None);
        let mut index = icons.build();
        assert_eq!(index.entries.len(), 1);
        index.items.insert(5, 7);
        let png = index.icon(5).expect("the item's icon");
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(&png[16..24], [0, 0, 0, 4, 0, 0, 0, 4], "4 by 4");
        // IDAT: inflate and check the first row's four pixels.
        let idat = png.windows(4).position(|w| w == b"IDAT").unwrap();
        let length = u32::from_be_bytes(png[idat - 4..idat].try_into().unwrap()) as usize;
        let raw = miniz_oxide::inflate::decompress_to_vec_zlib(&png[idat + 4..idat + 4 + length]).unwrap();
        assert_eq!(raw.len(), 4 * 17);
        assert_eq!(&raw[..17], [0, 255, 0, 0, 255, 0, 0, 255, 255, 170, 0, 85, 255, 85, 0, 170, 255]);
        let crc = crate::portrait_upload::crc32(&png[idat..idat + 4 + length]);
        assert_eq!(png[idat + 4 + length..idat + 8 + length], crc.to_be_bytes());
        assert!(index.icon(6).is_none(), "an item without an icon");
        let _ = std::fs::remove_dir_all(&root);
    }
}
