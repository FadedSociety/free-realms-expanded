//! Style cards (item type 12), part 1: the 42/10-42/12 requests, the card-to-record rule and
//! the 35/39 and 35/65 customization packets. Pure: no I/O and no state. Routing, ownership,
//! consuming the card and saving the look are part 2.
use crate::identity::{CharacterIdentity, Reader, invalid, string};
use std::io;

/// Mapping tables in `identity::Resources` order, also the index of `Look::ids`.
pub const HEAD: usize = 0;
pub const HAIR: usize = 1;
pub const MODEL_CUSTOMIZATION: usize = 2;
pub const FACE_PAINT: usize = 3;
pub const SKIN_TONE: usize = 4;

/// The client tables a card resolves through: `Resources/Models.txt` and the five
/// `Resources/CharacterCreate/*Mappings.txt`, as `identity::Resources` loads them.
pub trait Tables {
    /// Gender (GENDER, 1 or 2) and file stem (MODEL_FILE_NAME without `.adr`, e.g. `human_m`)
    /// of a player model.
    fn model(&self, id: i32) -> Option<(i32, &str)>;
    /// Row `id` of mapping table `table` (`HEAD` .. `SKIN_TONE`).
    fn mapping(&self, table: usize, id: i32) -> Option<&str>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    /// 42/10 UseStyleCard: an inventory item id. Lua `EquipScreen.UseStyleCard` (`00c11b10`)
    /// passes inventory row column 0 (ScriptsBase.lua `StyleCardHandler:UseStyleCards`);
    /// writer `00c11600`.
    Use(i32),
    /// 42/11 PreviewStyleCard: a definition id, which `00bda400` looks up in the definition
    /// cache before sending; writer `00bd98c0`.
    Preview(i32),
    /// 42/12 UseStyleCardByItemRecord: a definition id. `00a1ec00` sends `ItemRecord+4` (the
    /// definition, ctor `00a84370`) and Lua `UseStyleCardByItemId` (`00bda370`) its argument;
    /// writer `00a1b9a0` writes only that dword, no tint.
    UseByDefinition(i32),
}

/// Each request is the inventory header (`00a1ada0`: u16 42, u16 opcode) and one i32: exactly
/// 8 bytes. Anything else is refused.
pub fn decode_request(inner: &[u8]) -> io::Result<Request> {
    let &[42, 0, opcode, 0, a, b, c, d] = inner else {
        return Err(invalid("invalid style card request"));
    };
    let value = i32::from_le_bytes([a, b, c, d]);
    match opcode {
        10 => Ok(Request::Use(value)),
        11 => Ok(Request::Preview(value)),
        12 => Ok(Request::UseByDefinition(value)),
        _ => Err(invalid("not a style card request")),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The row is not a readable type 12 definition.
    NotAStyleCard,
    /// PARAM1 0 (the China heads), 8 (model) or a code no record carries (R): no store sells them.
    UnsupportedSlot,
    /// The card's gender usage excludes the character.
    WrongGender,
    /// The look's model is not a player model, or the card's asset is for another model.
    WrongModel,
    /// PARAM2 names no row of its table, or the row breaks the identity string bound.
    UnknownMapping,
}

/// One change record as the client reads it (`008e9710`): i32 id, string, i32 param, i32 item.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CustomizationRecord {
    /// What `PlayerCustomization_ApplyChangeRecord` (`00bd8e00`) switches on: 0 head, 1 skin
    /// tone, 2 hair, 3 hair tint, 4 eye tint, 5 model customization (an empty string clears
    /// it, `0096b970`), 6 face paint, 7 wings, 8 model (`00958460` looks it up as a model id).
    pub id: i32,
    pub string_param: String,
    pub param: i32,
    /// Kept by the client per record id for its own player when not a preview (`00bd8e00`);
    /// the card's item id (R, OSFR #108).
    pub item_id: i32,
}

/// A type 12 definition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StyleCard {
    /// PARAM1, the customization slot code.
    pub slot: i32,
    /// PARAM2, a mapping row, or the tint for hair and eye colour.
    pub mapping: i32,
    /// Gender usage: 0 any, 1 male, 2 female.
    pub gender: i32,
}

impl StyleCard {
    /// Walks a serialized definition the way the client reads it. `008e3760`: 46 fixed bytes,
    /// model and texture strings, gender usage (+0x68), type (+0x6c), 40 bytes of flags and
    /// dwords, a third string (`008e2600`), then a byte and four dwords (+0x9d..+0xac).
    /// `00919e10` then reads four dwords into +0xf0..+0xfc: PARAM1 is +0xf4 and PARAM2 +0xf8,
    /// the pair `00bd5980` compares with the stored slot values.
    pub fn from_row(row: &[u8]) -> Result<Self, Refusal> {
        let mut r = Reader::new(row);
        let mut walk = || -> io::Result<(i32, Self)> {
            r.take(46)?;
            r.payload(row.len())?;
            r.payload(row.len())?;
            let gender = r.i32()?;
            let kind = r.i32()?;
            r.take(40)?;
            r.payload(row.len())?;
            r.take(21)?;
            let (slot, mapping) = (r.i32()?, r.i32()?);
            Ok((
                kind,
                Self {
                    slot,
                    mapping,
                    gender,
                },
            ))
        };
        match walk() {
            Ok((12, card)) => Ok(card),
            _ => Err(Refusal::NotAStyleCard),
        }
    }

    /// The record this card makes for `look`. Every value comes from the card's row and the
    /// client tables; the request only names the item. A refusal changes nothing.
    pub fn record(
        &self,
        look: &Look,
        tables: &impl Tables,
        item_id: i32,
    ) -> Result<CustomizationRecord, Refusal> {
        let (gender, stem) = tables.model(look.model).ok_or(Refusal::WrongModel)?;
        if self.gender != 0 && self.gender != gender {
            return Err(Refusal::WrongGender);
        }
        // PARAM1 1-7 are record ids (`00bd5980` compares them as such); 9 is the creator's head
        // slot (`0071f030` case 9 reads HeadMappings) and becomes record 0 (R).
        let (id, table) = match self.slot {
            1 => (1, Some(SKIN_TONE)),
            2 => (2, Some(HAIR)),
            3 => (3, None),
            4 => (4, None),
            5 => (5, Some(MODEL_CUSTOMIZATION)),
            6 => (6, Some(FACE_PAINT)),
            7 => (7, Some(MODEL_CUSTOMIZATION)),
            9 => (0, Some(HEAD)),
            _ => return Err(Refusal::UnsupportedSlot),
        };
        let text = match table {
            None => "",
            // Clean Shaven: PARAM2 0 is no row; the empty string clears the model
            // customization, so only a facial hair model may use it, never a pixie (wings).
            Some(MODEL_CUSTOMIZATION) if id == 5 && self.mapping == 0 => {
                if stem != "human_m" {
                    return Err(Refusal::WrongModel);
                }
                ""
            }
            Some(table) => {
                let text = tables
                    .mapping(table, self.mapping)
                    .filter(|text| text.len() <= 512 && !text.contains('\0'))
                    .ok_or(Refusal::UnknownMapping)?;
                if matches!(table, HEAD | HAIR | MODEL_CUSTOMIZATION) && !fits(stem, text) {
                    return Err(Refusal::WrongModel);
                }
                text
            }
        };
        Ok(CustomizationRecord {
            id,
            string_param: text.to_owned(),
            param: self.mapping,
            item_id,
        })
    }
}

/// Whether an asset name is for this model. A fixed prefix must be the model's stem
/// (`human_m_facialhair_*`, `fairy_f_wings_*`, `fairy_f_hair_FRC*`); after the `<race>`
/// placeholder a fixed gender letter must still match (`<race>_m_hair_*`, R) unless it is
/// `<gender>`. The client fills both placeholders from the model row (`0070ca40`).
fn fits(stem: &str, asset: &str) -> bool {
    let Some((_, gender)) = stem.split_once('_') else {
        return false;
    };
    let rest = match asset.strip_prefix("<race>_") {
        Some(rest) => rest
            .strip_prefix("<gender>")
            .or_else(|| rest.strip_prefix(gender)),
        None => asset.strip_prefix(stem),
    };
    rest.is_some_and(|rest| rest.starts_with('_'))
}

/// The customizable part of a character's appearance. Every field is persisted (part 2 writes
/// it back to `CharacterIdentity`), compared (an equal look is no change) and sent (35/65).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Look {
    pub model: i32,
    pub head: String,
    pub hair: String,
    pub hair_color: i32,
    pub eye_color: i32,
    pub skin_tone: String,
    pub face_paint: String,
    pub model_customization: String,
    /// Mapping ids of head, hair, model customization, face paint and skin tone, indexed
    /// `HEAD` .. `SKIN_TONE` (`NativeCreation::choices[1..=5]`).
    pub ids: [i32; 5],
}

impl Look {
    pub fn from_identity(identity: &CharacterIdentity) -> Self {
        let c = &identity.creation.choices;
        Self {
            model: identity.model,
            head: identity.head.clone(),
            hair: identity.hair.clone(),
            hair_color: identity.hair_color,
            eye_color: identity.eye_color,
            skin_tone: identity.skin_tone.clone(),
            face_paint: identity.face_paint.clone(),
            model_customization: identity.model_customization.clone(),
            ids: [c[1], c[2], c[3], c[4], c[5]],
        }
    }

    /// The look after one record: the one place that decides which field each record id
    /// writes (the `00bd8e00` switch). Unknown ids change nothing.
    pub fn apply(&self, record: &CustomizationRecord) -> Self {
        let mut look = self.clone();
        let (text, param) = (record.string_param.clone(), record.param);
        match record.id {
            0 => (look.head, look.ids[HEAD]) = (text, param),
            1 => (look.skin_tone, look.ids[SKIN_TONE]) = (text, param),
            2 => (look.hair, look.ids[HAIR]) = (text, param),
            3 => look.hair_color = param,
            4 => look.eye_color = param,
            5 | 7 => (look.model_customization, look.ids[MODEL_CUSTOMIZATION]) = (text, param),
            6 => (look.face_paint, look.ids[FACE_PAINT]) = (text, param),
            8 => look.model = param,
            _ => {}
        }
        look
    }
}

/// 35/39 PlayerUpdatePacketCustomizationChange (ctor `0091d5f0`, reader `0091aff0`): u64
/// character guid, u8 preview, then the records.
pub fn customization_change(guid: u64, preview: bool, records: &[CustomizationRecord]) -> Vec<u8> {
    let mut out = vec![35, 0, 39, 0];
    out.extend(guid.to_le_bytes());
    out.push(preview.into());
    put_records(&mut out, records);
    out
}

/// 35/65 PlayerUpdatePacketCustomizationData (ctor `0091d4d0`, reader `00925460`): the records
/// only, which replace the client's whole stored set (`00bd9730`). Records 0-6 and 8; a
/// pixie's model customization is its wings, record 7 instead of 5 (R). Item ids are 0 (R):
/// the look does not keep which card set a slot.
pub fn customization_data(look: &Look, tables: &impl Tables) -> Vec<u8> {
    let wings = tables
        .model(look.model)
        .is_some_and(|(_, stem)| stem.starts_with("fairy_"));
    let row = |id, text: &str, param| CustomizationRecord {
        id,
        string_param: text.to_owned(),
        param,
        item_id: 0,
    };
    let records = [
        row(0, &look.head, look.ids[HEAD]),
        row(1, &look.skin_tone, look.ids[SKIN_TONE]),
        row(2, &look.hair, look.ids[HAIR]),
        row(3, "", look.hair_color),
        row(4, "", look.eye_color),
        row(
            if wings { 7 } else { 5 },
            &look.model_customization,
            look.ids[MODEL_CUSTOMIZATION],
        ),
        row(6, &look.face_paint, look.ids[FACE_PAINT]),
        row(8, "", look.model),
    ];
    let mut out = vec![35, 0, 65, 0];
    put_records(&mut out, &records);
    out
}

/// `00913230`: i32 count, then each record as `008e9710` reads it.
fn put_records(out: &mut Vec<u8>, records: &[CustomizationRecord]) {
    out.extend((records.len() as u32).to_le_bytes());
    for record in records {
        out.extend(record.id.to_le_bytes());
        string(out, &record.string_param);
        out.extend(record.param.to_le_bytes());
        out.extend(record.item_id.to_le_bytes());
    }
}
