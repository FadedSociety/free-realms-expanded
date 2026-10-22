//! One native-created local character. Protocol/resource facts were checked
//! against pinned Sanctuary and the supplied ClientPcData serializer; no account
//! service, moderation system or original progression is implied.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::Path,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCreation {
    pub first_name: String,
    pub last_name: String,
    pub temporary_first_name: String,
    pub temporary_last_name: String,
    /// Model, head, hair, customization, face, skin; six clothing definitions;
    /// six clothing tints; eye tint, hair tint, location, in native wire order
    /// (client writer 0x00954bd0, filled at 0x007186a4..0x007187a9).
    pub choices: [i32; 21],
}

impl NativeCreation {
    pub fn decode(bytes: &[u8]) -> io::Result<Self> {
        let mut r = Reader::new(bytes);
        let mut value = Self {
            first_name: r.string(128)?,
            last_name: r.string(128)?,
            temporary_first_name: r.string(128)?,
            temporary_last_name: r.string(128)?,
            choices: [0; 21],
        };
        for choice in &mut value.choices {
            *choice = r.i32()?;
        }
        r.end()?;
        Ok(value)
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for s in [
            &self.first_name,
            &self.last_name,
            &self.temporary_first_name,
            &self.temporary_last_name,
        ] {
            string(&mut out, s);
        }
        for choice in self.choices {
            out.extend(choice.to_le_bytes());
        }
        out
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attachment {
    pub model_name: String,
    pub texture_alias: String,
    pub tint_alias: String,
    pub tint_id: i32,
    pub composite_effect_id: i32,
    pub slot: i32,
    pub definition: i32,
}

impl Attachment {
    pub(crate) fn encode(&self, out: &mut Vec<u8>) {
        string(out, &self.model_name);
        string(out, &self.texture_alias);
        string(out, &self.tint_alias);
        for value in [self.tint_id, self.composite_effect_id, self.slot] {
            out.extend(value.to_le_bytes());
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterIdentity {
    pub guid: u64,
    pub first_name: String,
    pub last_name: String,
    pub model: i32,
    pub head: String,
    pub hair: String,
    pub model_customization: String,
    pub face_paint: String,
    pub skin_tone: String,
    pub eye_color: i32,
    pub hair_color: i32,
    pub gender: i32,
    pub creation: NativeCreation,
    pub attachments: Vec<Attachment>,
}

pub fn valid_name(first: &str, last: &str) -> bool {
    let valid = |s: &str| {
        s.chars().count() <= 32
            && s.len() <= 128
            && s.chars()
                .all(|c| c.is_alphabetic() || matches!(c, ' ' | '-' | '\''))
    };
    !first.trim().is_empty()
        && first == first.trim()
        && last == last.trim()
        && valid(first)
        && valid(last)
}

impl CharacterIdentity {
    pub fn validate(&self) -> io::Result<()> {
        if self.guid != 1
            || self.model <= 0
            || !matches!(self.gender, 1 | 2)
            || !valid_name(&self.first_name, &self.last_name)
            || self.attachments.len() > 6
        {
            return Err(invalid("invalid single-character identity"));
        }
        for s in [
            &self.head,
            &self.hair,
            &self.model_customization,
            &self.face_paint,
            &self.skin_tone,
        ] {
            if s.len() > 512 || s.contains('\0') {
                return Err(invalid("invalid appearance string"));
            }
        }
        if self.head.is_empty() || self.hair.is_empty() || self.skin_tone.is_empty() {
            return Err(invalid("incomplete character appearance"));
        }
        for a in &self.attachments {
            if a.definition <= 0
                || !(1..=32).contains(&a.slot)
                || a.tint_id < 0
                || [&a.model_name, &a.texture_alias, &a.tint_alias]
                    .iter()
                    .any(|s| s.len() > 512 || s.contains('\0'))
            {
                return Err(invalid("invalid character attachment"));
            }
        }
        Ok(())
    }

    pub fn display_name(&self) -> String {
        if self.last_name.is_empty() {
            self.first_name.clone()
        } else {
            format!("{} {}", self.first_name, self.last_name)
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    version: u32,
    character: CharacterIdentity,
}

/// Missing means the native creator should be offered. Corrupt files fail.
pub fn load(path: &Path) -> io::Result<Option<CharacterIdentity>> {
    use std::io::Read;
    let f = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    let mut bytes = Vec::new();
    f.take(65537).read_to_end(&mut bytes)?;
    if bytes.len() > 65536 {
        return Err(invalid("identity file exceeds 64 KiB"));
    }
    let d: Document = serde_json::from_slice(&bytes).map_err(invalid)?;
    if d.version != 1 {
        return Err(invalid("unsupported identity version"));
    }
    d.character.validate()?;
    Ok(Some(d.character))
}

pub fn save(path: &Path, identity: &CharacterIdentity) -> io::Result<()> {
    identity.validate()?;
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let bytes = serde_json::to_vec_pretty(&Document {
        version: 1,
        character: identity.clone(),
    })
    .map_err(invalid)?;
    if bytes.len() > 65536 {
        return Err(invalid("identity file exceeds 64 KiB"));
    }
    crate::character_store::replace_bytes(path, &bytes)
}

/// Private supplied text tables, kept outside the repository. Only models marked
/// valid for player characters and appearance/clothing IDs in these tables pass.
pub struct Resources {
    models: BTreeMap<i32, i32>,
    mappings: [BTreeMap<i32, String>; 5],
    items: BTreeMap<i32, (Attachment, i32)>,
    /// Skin tone and head ids the pinned creator tables offer; `None` only for a
    /// tree with none of the allow-list inputs (see `Resources::load`).
    pinned_looks: Option<[BTreeSet<i32>; 2]>,
}

/// Community creator tables of the client overlay (`OVERLAY` in
/// scripts/bundle/client_overlay.py), which the native harness installs and the
/// launcher ships (L024). Same paths and pins; tests/creator_allow_list.rs builds
/// its tree through that overlay, so re-pinning one side alone fails the test.
/// They drop the rows that select skin tone 7/8 and the four China heads. The
/// client reads them from Resources/ itself: 0x008ec05d passes "Resources/" to
/// 0x00982f10, which opens "%s%s.txt" (0x009830c9) with the name from 0x0097f570.
/// Order is skin tone, then face type: `resolve` reads them as `[skins, heads]`.
const PINNED_CREATOR_TABLES: [(&str, &str); 2] = [
    (
        "CharacterCreateCustomizationSkinTone.txt",
        "485d20174e96c53021d9644d3ba943f0a016fb0413d1be8fa8f728bea528dd9d",
    ),
    (
        "CharacterCreateCustomizationFaceType.txt",
        "9d4d7acd63c7926ed3d4c03f99116e929061eef4ae25e1a2b6208c738f2c8525",
    ),
];

const ITEM_MAPPINGS: &str = "CharacterCreate/CharacterCreateItemMappings.txt";

/// Each pinned row's ITEM_ID goes through the item mappings (ITEM_ID column 5,
/// row parser 0x007255e0) to an item definition whose PARAM2 (column 25, stored at
/// +0x218 by 0x0072e050) is the skin tone or head id: ChangeItem 0x0071f030 cases 1
/// and 9 look it up in SkinToneMappings/HeadMappings, and 0x0071875b/0x00718796
/// copy it into creation choices 1 and 5 for the writer 0x00954bd0.
fn pinned_looks(directory: &Path, param2: &BTreeMap<i32, i32>) -> io::Result<[BTreeSet<i32>; 2]> {
    let mut definitions = BTreeMap::new();
    for row in rows(&directory.join(ITEM_MAPPINGS))? {
        if row.len() >= 6 {
            if let (Ok(id), Ok(definition)) = (row[0].parse::<i32>(), row[5].parse::<i32>()) {
                definitions.insert(id, definition);
            }
        }
    }
    let mut looks: [BTreeSet<i32>; 2] = Default::default();
    for ((name, sha256), allowed) in PINNED_CREATOR_TABLES.iter().zip(&mut looks) {
        let bytes = fs::read(directory.join(name))?;
        if format!("{:x}", Sha256::digest(&bytes)) != *sha256 {
            return Err(invalid("creator table does not match its pin"));
        }
        for row in parse_rows(bytes)? {
            let id = row
                .get(3)
                .and_then(|item| item.parse::<i32>().ok())
                .and_then(|item| definitions.get(&item))
                .and_then(|definition| param2.get(definition))
                .ok_or_else(|| invalid("pinned creator row does not resolve"))?;
            allowed.insert(*id);
        }
    }
    Ok(looks)
}

fn rows(path: &Path) -> io::Result<Vec<Vec<String>>> {
    parse_rows(fs::read(path)?)
}

fn parse_rows(bytes: Vec<u8>) -> io::Result<Vec<Vec<String>>> {
    if bytes.len() > 32 * 1024 * 1024 {
        return Err(invalid("creation resource exceeds local bound"));
    }
    let text = String::from_utf8(bytes).map_err(invalid)?;
    Ok(text
        .lines()
        .filter(|s| !s.starts_with('#') && !s.is_empty())
        .map(|s| s.split('^').map(str::to_owned).collect())
        .collect())
}

impl Resources {
    pub fn load(directory: &Path) -> io::Result<Self> {
        let mut models = BTreeMap::new();
        for row in rows(&directory.join("Models.txt"))? {
            if row.len() >= 14 && row[13] == "1" {
                if let (Ok(id), Ok(gender)) = (row[0].parse(), row[5].parse()) {
                    models.insert(id, gender);
                }
            }
        }
        let mut mappings: [BTreeMap<i32, String>; 5] = Default::default();
        for (index, name) in [
            "HeadMappings.txt",
            "HairMappings.txt",
            "ModelCustomizationMappings.txt",
            "FacePaintMappings.txt",
            "SkinToneMappings.txt",
        ]
        .iter()
        .enumerate()
        {
            for row in rows(&directory.join("CharacterCreate").join(name))? {
                if row.len() >= 2 {
                    if let Ok(id) = row[0].parse() {
                        mappings[index].insert(id, row[1].clone());
                    }
                }
            }
        }
        let mut items = BTreeMap::new();
        let mut param2 = BTreeMap::new();
        for row in rows(&directory.join("CharacterCreate/ItemDefinitions.txt"))? {
            if row.len() < 42 {
                continue;
            }
            let int = |n: usize| row[n].parse::<i32>().map_err(invalid);
            let a = Attachment {
                definition: int(0)?,
                model_name: row[16].clone(),
                texture_alias: row[18].clone(),
                tint_alias: row[41].clone(),
                tint_id: int(19)?,
                composite_effect_id: int(36)?,
                slot: int(12)?,
            };
            param2.insert(a.definition, int(25)?);
            items.insert(a.definition, (a, int(17)?));
        }
        if models.is_empty() || mappings.iter().any(BTreeMap::is_empty) || items.is_empty() {
            return Err(invalid("missing native creation resources"));
        }
        // A tree holding any allow-list input must hold all of them, pinned: every
        // client tree does (a stock one needs the overlay installed). Trees with none
        // (synthetic fixtures, bundle login-resources) create unfiltered.
        let mut pinned = directory.join(ITEM_MAPPINGS).try_exists()?;
        for (name, _) in PINNED_CREATOR_TABLES {
            pinned |= directory.join(name).try_exists()?;
        }
        let pinned_looks = if pinned {
            Some(pinned_looks(directory, &param2)?)
        } else {
            None
        };
        Ok(Self {
            models,
            mappings,
            items,
            pinned_looks,
        })
    }

    pub fn resolve(&self, creation: NativeCreation) -> io::Result<CharacterIdentity> {
        let c = &creation.choices;
        let gender = *self
            .models
            .get(&c[0])
            .ok_or_else(|| invalid("invalid player model"))?;
        if let Some([skins, heads]) = &self.pinned_looks
            && (!skins.contains(&c[5]) || !heads.contains(&c[1]))
        {
            return Err(invalid("look hidden by the pinned creator tables"));
        }
        let mapping = |index: usize, id: i32| {
            self.mappings[index]
                .get(&id)
                .cloned()
                .ok_or_else(|| invalid("invalid appearance ID"))
        };
        let mut attachments = Vec::new();
        // Native order: hands, feet, legs, chest, head, shoulders.
        for n in 0..6 {
            let id = c[6 + n];
            if id == 0 && matches!(n, 0 | 4 | 5) {
                continue;
            }
            let (mut a, usage) = self
                .items
                .get(&id)
                .cloned()
                .ok_or_else(|| invalid("invalid creator clothing ID"))?;
            if usage != 0 && usage != gender {
                return Err(invalid("clothing does not match character gender"));
            }
            let expected_slot = [6, 5, 4, 3, 1, 2][n];
            if a.slot != expected_slot || c[12 + n] < 0 {
                return Err(invalid("invalid creator clothing slot or tint"));
            }
            a.tint_id = c[12 + n];
            attachments.push(a);
        }
        if c[18] < 0 || c[19] < 0 {
            return Err(invalid("invalid character tint"));
        }
        let first_name = if creation.first_name.is_empty() {
            creation.temporary_first_name.clone()
        } else {
            creation.first_name.clone()
        };
        let last_name = if creation.last_name.is_empty() {
            creation.temporary_last_name.clone()
        } else {
            creation.last_name.clone()
        };
        let identity = CharacterIdentity {
            guid: 1,
            first_name,
            last_name,
            model: c[0],
            head: mapping(0, c[1])?,
            hair: mapping(1, c[2])?,
            model_customization: if c[3] == 0 {
                String::new()
            } else {
                mapping(2, c[3])?
            },
            face_paint: mapping(3, c[4])?,
            skin_tone: mapping(4, c[5])?,
            eye_color: c[18],
            hair_color: c[19],
            gender,
            creation,
            attachments,
        };
        identity.validate()?;
        Ok(identity)
    }
}

/// Apply AFTER original-offset movement/gameplay splices. The supplied self
/// prefix is parsed rather than assuming equal string lengths. Preserve every
/// unowned field and repair the two enclosing lengths. Clothing/profile state
/// is owned by the gateway inventory service, not this prefix patch.
pub fn patch_self(bytes: &[u8], identity: &CharacterIdentity) -> io::Result<Vec<u8>> {
    identity.validate()?;
    if bytes.len() < 33
        || bytes[..3] != [5, 0, 1]
        || bytes[7..9] != [12, 0]
        || u32::from_le_bytes(bytes[3..7].try_into().unwrap()) as usize != bytes.len() - 7
        || u32::from_le_bytes(bytes[9..13].try_into().unwrap()) as usize != bytes.len() - 13
        || u64::from_le_bytes(bytes[21..29].try_into().unwrap()) != 1
    {
        return Err(invalid("unsupported native self packet"));
    }
    let mut r = Reader::new(&bytes[33..]);
    r.string(512)?;
    r.string(512)?;
    r.take(8)?;
    r.string(512)?;
    r.string(512)?;
    r.string(512)?;
    let appearance_end = 33 + r.offset;
    let names_start = appearance_end
        .checked_add(64)
        .ok_or_else(|| invalid("self prefix overflow"))?;
    let mut names = Reader::new(
        bytes
            .get(names_start..)
            .ok_or_else(|| invalid("short self prefix"))?,
    );
    names.string(128)?;
    names.string(128)?;
    let names_end = names_start + names.offset;
    if bytes.len() < names_end + 4 {
        return Err(invalid("short self tail"));
    }
    let mut out = bytes[..29].to_vec();
    out.extend(identity.model.to_le_bytes());
    string(&mut out, &identity.head);
    string(&mut out, &identity.hair);
    out.extend(identity.hair_color.to_le_bytes());
    out.extend(identity.eye_color.to_le_bytes());
    string(&mut out, &identity.skin_tone);
    string(&mut out, &identity.face_paint);
    string(&mut out, &identity.model_customization);
    // Independently corroborated as appearance IDs by the newer community
    // ClientPcData schema; recovered code had named these Unknown11..15.
    for index in [1, 2, 5, 4, 3] {
        out.extend(identity.creation.choices[index].to_le_bytes());
    }
    out.extend(&bytes[appearance_end + 20..names_start]);
    string(&mut out, &identity.first_name);
    string(&mut out, &identity.last_name);
    out.extend(&bytes[names_end..]);
    let outer = u32::try_from(out.len() - 7).map_err(invalid)?;
    let inner = u32::try_from(out.len() - 13).map_err(invalid)?;
    out[3..7].copy_from_slice(&outer.to_le_bytes());
    out[9..13].copy_from_slice(&inner.to_le_bytes());
    Ok(out)
}

pub(crate) fn invalid(e: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, e)
}
pub(crate) fn string(out: &mut Vec<u8>, value: &str) {
    out.extend((value.len() as u32).to_le_bytes());
    out.extend(value.as_bytes());
}
pub(crate) fn payload(out: &mut Vec<u8>, value: &[u8]) {
    out.extend((value.len() as u32).to_le_bytes());
    out.extend(value);
}
pub(crate) struct Reader<'a> {
    bytes: &'a [u8],
    pub(crate) offset: usize,
}
impl<'a> Reader<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }
    pub(crate) fn take(&mut self, n: usize) -> io::Result<&'a [u8]> {
        let end = self
            .offset
            .checked_add(n)
            .ok_or_else(|| invalid("wire length overflow"))?;
        let s = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| invalid("truncated login message"))?;
        self.offset = end;
        Ok(s)
    }
    pub(crate) fn u8(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }
    pub(crate) fn i32(&mut self) -> io::Result<i32> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    pub(crate) fn u64(&mut self) -> io::Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    pub(crate) fn string(&mut self, max: usize) -> io::Result<String> {
        let n = self.i32()?;
        if n < 0 || n as usize > max {
            return Err(invalid("login string exceeds bound"));
        }
        String::from_utf8(self.take(n as usize)?.to_vec()).map_err(invalid)
    }
    pub(crate) fn payload(&mut self, max: usize) -> io::Result<&'a [u8]> {
        let n = self.i32()?;
        if n < 0 || n as usize > max {
            return Err(invalid("login payload exceeds bound"));
        }
        self.take(n as usize)
    }
    pub(crate) fn end(&self) -> io::Result<()> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(invalid("trailing login bytes"))
        }
    }
}
