//! T477 gates: the style card requests, the card-to-record rule and the 35/39 and 35/65
//! encoders, read back by decoders written from the client readers (`0091aff0`, `00925460`,
//! `00913230`, `008e9710`) that must consume every byte.
use osfr_tools::identity::{CharacterIdentity, NativeCreation};
use osfr_tools::style_cards::{
    CustomizationRecord, FACE_PAINT, HAIR, HEAD, Look, MODEL_CUSTOMIZATION, Refusal, Request,
    SKIN_TONE, StyleCard, Tables, customization_change, customization_data, decode_request,
};

/// Serialized definitions from our item catalogue (`lab/reports/item-catalogue-20261015`,
/// source legacy+client), by definition id.
const ROWS: &[(i32, &str)] = &[
    (
        77484,
        "ac2e01005c8e060000000000a21900000100000000000000000000000000000000000000000000000100000001010a000000536b696e546f6e65203100000000000000000c0000008d0000000000000000000000000000000000000000000000000000000000000000000001000000000700000064796574696e74000a000000000000000000000000000000000000000100000001000000000000000000000000000000",
    ),
    (
        6826,
        "aa1a000067bf010083bf01005c1200000000000000000000000000001c02000001000000000000000200000001000000000000000000010000000c0000008d00000001000000000000000000000000000100000000000000000000000000000001ffffffff000700000064796574696e74000a000000000000000000000000000000000000000200000001000000000000000000000000000000",
    ),
    (
        6834,
        "b21a00006fbf01008bbf01004a1200000000000000000000000000001c02000001000000000000000200000001000000000000000000020000000c0000008d00000001000000000000000000000000000100000000000000000000000000000001ffffffff000700000064796574696e74000a00000000000000000000000000000000000000020000000b000000000000000000000000000000",
    ),
    (
        35950,
        "6e8c00006751060087510600c0110000020000000000000000000000b400000001000000000000000300000001010000000000000000000000000c0000008e00000000000000000000000000000000000100000003000000000000000000000001ffffffff000700000064796574696e74000a000000000000000000000000000000000000000300000002000000000000000000000000000000",
    ),
    (
        35982,
        "8e8c0000da510600ef510600bf1100002c00000000000000000000005a00000001000000000000000400000001010000000000000000000000000c0000008f00000000000000000000000000000000000100000003000000000000000000000001ffffffff000700000064796574696e74000a00000000000000000000000000000000000000040000002c000000000000000000000000000000",
    ),
    (
        36022,
        "b68c0000ec530600f3530600611200000000000000000000000000000e01000001000000000000000500000001010000000000000000010000000c0000009000000000000000000000000000000000000100000003000000000000000000000001ffffffff000700000064796574696e74000a000000000000000000000000000000000000000500000000000000000000000000000000000000",
    ),
    (
        36023,
        "b78c0000ed530600f4530600841100000000000000000000000000000e01000001000000000000000500000001010000000000000000010000000c0000009000000000000000000000000000000000000100000003000000000000000000000001ffffffff000700000064796574696e74000a000000000000000000000000000000000000000500000001000000000000000000000000000000",
    ),
    (
        36030,
        "be8c0000e5550600f1550600701100000000000000000000000000000e01000001000000000000000700000001010000000000000000020000000c0000009200000000000000000000000000000000000100000003000000000000000000000001ffffffff000700000064796574696e74000a000000000000000000000000000000000000000500000007000000000000000000000000000000",
    ),
    (
        36004,
        "a48c00004d5306005f530600bc1100000000000000000000000000000e01000001000000000000000600000001010000000000000000010000000c0000009100000000000000000000000000000000000100000003000000000000000000000001ffffffff000700000064796574696e74000a000000000000000000000000000000000000000600000001000000000000000000000000000000",
    ),
    (
        77530,
        "da2e0100ff07000000000000a31900000000000000000000000000000000000000000000000000000700000001011b00000066616972795f665f77696e67735f627574746572666c792e61647200000000020000000c000000000000000000000000000000000000000000000000000000000000000000000000000001000000000700000064796574696e74000a000000000000000000000000000000000000000700000007000000000000000000000000000000",
    ),
    (
        77536,
        "e02e01003a8e060000000000a91900000000000000000000000000000000000000000000000000000700000001011800000066616972795f6d5f77696e67735f6372656570792e61647200000000010000000c000000000000000000000000000000000000000000000000000000000000000000000000000001000000000700000064796574696e74000a00000000000000000000000000000000000000070000000d000000000000000000000000000000",
    ),
    (
        77472,
        "a02e0100a48e06000000000065190000000000000000000000000000000000000000000000000000090000000101220000003c726163653e5f3c67656e6465723e5f686561645f63617563617369616e2e6164721a000000666163655f747970655f70665f63617563617369616e2e706e67010000000c000000000000000000000000000000000000000000000000000000000000000000000000000001000000000700000064796574696e74000a000000000000000000000000000000000000000900000001000000000000000000000000000000",
    ),
    (
        78086,
        "063101005da60600000000005c1200000000000000000000000000000000000001000000000000000000000001011600000066616972795f665f686561645f6368696e612e61647200000000010000000c0000008d0000000000000000000000000000000000000000000000000000000000000000000001000000000700000064796574696e74000a000000000000000000000000000000000000000000000005000000000000000000000000000000",
    ),
    (
        77641,
        "492f0100498e0600000000005f1900000000000000000000000000000000000000000000000000000800000000000000000000000000010000000c000000000000000000000000000000000000000000010000000000000000000000000000000001000000000700000064796574696e74000a000000000000000000000000000000000000000800000000000000000000000000000000000000",
    ),
    (
        1,
        "01000000ba0c0000bb0c0000be000000e7000000000000000000000005000000010000000100000000000000010000000000000000000000000001000000000000000000000000000000000000000000010000000000000000000000000000000101000000000700000064796574696e74000a000000000000000000000000000000000000000100000001000000000000000000000000000000",
    ),
];

fn row(id: i32) -> Vec<u8> {
    let (_, hex) = ROWS.iter().find(|(row, _)| *row == id).unwrap();
    (0..hex.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).unwrap())
        .collect()
}

fn card(id: i32) -> StyleCard {
    StyleCard::from_row(&row(id)).unwrap()
}

/// Rows of the client's `Resources/Models.txt` and `Resources/CharacterCreate/*Mappings.txt`.
struct Client;

const MAPPINGS: &[(usize, i32, &str)] = &[
    (HEAD, 1, "<race>_<gender>_head_caucasian.adr"),
    (HEAD, 2, "<race>_<gender>_head_african.adr"),
    (HAIR, 1, "<race>_m_hair_afro_<hairtype>.adr"),
    (HAIR, 11, "<race>_f_hair_flame_<hairtype>.adr"),
    (HAIR, 54, "fairy_f_hair_FRCcurly_<hairtype>.adr"),
    (MODEL_CUSTOMIZATION, 1, "human_m_facialhair_beard.adr"),
    (MODEL_CUSTOMIZATION, 7, "fairy_f_wings_butterfly.adr"),
    (MODEL_CUSTOMIZATION, 13, "fairy_m_wings_creepy.adr"),
    (FACE_PAINT, 0, "none"),
    (FACE_PAINT, 1, "skull"),
    (SKIN_TONE, 1, "skintone1"),
    (SKIN_TONE, 3, "skintone3"),
];

impl Tables for Client {
    fn model(&self, id: i32) -> Option<(i32, &str)> {
        match id {
            1 => Some((1, "human_m")),
            2 => Some((1, "fairy_m")),
            60 => Some((2, "human_f")),
            61 => Some((2, "fairy_f")),
            _ => None,
        }
    }

    fn mapping(&self, table: usize, id: i32) -> Option<&str> {
        MAPPINGS
            .iter()
            .find(|(t, row, _)| (*t, *row) == (table, id))
            .map(|(_, _, text)| *text)
    }
}

const HUMAN_M: i32 = 1;
const FAIRY_M: i32 = 2;
const HUMAN_F: i32 = 60;
const FAIRY_F: i32 = 61;

fn look(model: i32) -> Look {
    let (hair, model_customization) = match model {
        HUMAN_M => (
            (1, "<race>_m_hair_afro_<hairtype>.adr"),
            (1, "human_m_facialhair_beard.adr"),
        ),
        FAIRY_M => (
            (1, "<race>_m_hair_afro_<hairtype>.adr"),
            (13, "fairy_m_wings_creepy.adr"),
        ),
        FAIRY_F => (
            (11, "<race>_f_hair_flame_<hairtype>.adr"),
            (7, "fairy_f_wings_butterfly.adr"),
        ),
        _ => ((11, "<race>_f_hair_flame_<hairtype>.adr"), (0, "")),
    };
    Look {
        model,
        head: "<race>_<gender>_head_african.adr".into(),
        hair: hair.1.into(),
        hair_color: 5,
        eye_color: 6,
        skin_tone: "skintone3".into(),
        face_paint: "none".into(),
        model_customization: model_customization.1.into(),
        ids: [2, hair.0, model_customization.0, 0, 3],
    }
}

/// Bounds-checked cursor; a short read panics, so a test fails instead of guessing.
struct Wire<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Wire<'a> {
    fn take(&mut self, n: usize) -> &'a [u8] {
        let bytes = &self.bytes[self.at..self.at + n];
        self.at += n;
        bytes
    }

    fn i32(&mut self) -> i32 {
        i32::from_le_bytes(self.take(4).try_into().unwrap())
    }

    /// `00913230` count, then `008e9710` per record: i32 id, i32 length and bytes (a negative
    /// length is an error), i32 param, i32 item id.
    fn records(&mut self) -> Vec<CustomizationRecord> {
        (0..self.i32())
            .map(|_| {
                let id = self.i32();
                let length = usize::try_from(self.i32()).unwrap();
                let string_param = String::from_utf8(self.take(length).to_vec()).unwrap();
                CustomizationRecord {
                    id,
                    string_param,
                    param: self.i32(),
                    item_id: self.i32(),
                }
            })
            .collect()
    }

    fn done(&self) {
        assert_eq!(self.at, self.bytes.len(), "the reader consumes every byte");
    }
}

/// 35/39 as `0091aff0` reads it: u16 35, u16 39 (`008d6830`), u64 guid, u8 preview, records.
fn read_change(bytes: &[u8]) -> (u64, bool, Vec<CustomizationRecord>) {
    let mut wire = Wire { bytes, at: 0 };
    assert_eq!(wire.take(4), [35, 0, 39, 0]);
    let guid = u64::from_le_bytes(wire.take(8).try_into().unwrap());
    let preview = wire.take(1)[0] != 0;
    let records = wire.records();
    wire.done();
    (guid, preview, records)
}

/// 35/65 as `00925460` reads it: u16 35, u16 65, records; no guid and no preview.
fn read_data(bytes: &[u8]) -> Vec<CustomizationRecord> {
    let mut wire = Wire { bytes, at: 0 };
    assert_eq!(wire.take(4), [35, 0, 65, 0]);
    let records = wire.records();
    wire.done();
    records
}

/// Uses definition `id` on `look`, sends the record in a 35/39 and reads it back.
fn use_card(id: i32, look: &Look) -> CustomizationRecord {
    let record = card(id).record(look, &Client, 900).unwrap();
    let sent = customization_change(1, false, std::slice::from_ref(&record));
    assert_eq!(read_change(&sent), (1, false, vec![record.clone()]));
    record
}

fn expect(id: i32, text: &str, param: i32) -> CustomizationRecord {
    CustomizationRecord {
        id,
        string_param: text.into(),
        param,
        item_id: 900,
    }
}

#[test]
fn request_decoders_take_exact_lengths() {
    let body = |opcode: u8| vec![42, 0, opcode, 0, 0x2a, 0x8c, 0, 0];
    assert_eq!(decode_request(&body(10)).unwrap(), Request::Use(35882));
    assert_eq!(decode_request(&body(11)).unwrap(), Request::Preview(35882));
    assert_eq!(
        decode_request(&body(12)).unwrap(),
        Request::UseByDefinition(35882)
    );
    let mut long = body(12);
    long.extend([0; 4]); // an ItemRecord (definition, tint) is not what 00a1b9a0 writes
    for refused in [
        body(12)[..7].to_vec(),
        long,
        vec![42, 0, 12, 0],
        body(13),
        body(9),
        vec![35, 0, 12, 0, 1, 0, 0, 0],
        vec![42, 1, 12, 0, 1, 0, 0, 0],
    ] {
        assert!(decode_request(&refused).is_err(), "{refused:?}");
    }
}

#[test]
fn skin_tone_card_is_record_1() {
    let record = use_card(77484, &look(HUMAN_M));
    assert_eq!(record, expect(1, "skintone1", 1));
    let after = look(HUMAN_M).apply(&record);
    assert_eq!(
        (after.skin_tone.as_str(), after.ids[SKIN_TONE]),
        ("skintone1", 1)
    );
}

#[test]
fn hair_card_is_record_2() {
    let record = use_card(6834, &look(FAIRY_F));
    assert_eq!(record, expect(2, "<race>_f_hair_flame_<hairtype>.adr", 11));
    let record = use_card(6826, &look(HUMAN_M));
    assert_eq!(record, expect(2, "<race>_m_hair_afro_<hairtype>.adr", 1));
    let after = look(HUMAN_F).apply(&record);
    assert_eq!(
        (after.hair.as_str(), after.ids[HAIR]),
        (record.string_param.as_str(), 1)
    );
}

#[test]
fn hair_colour_card_is_record_3_with_its_tint() {
    let record = use_card(35950, &look(FAIRY_M));
    assert_eq!(record, expect(3, "", 2));
    assert_eq!(look(FAIRY_M).apply(&record).hair_color, 2);
}

#[test]
fn eye_colour_card_is_record_4_with_its_tint() {
    let record = use_card(35982, &look(HUMAN_F));
    assert_eq!(record, expect(4, "", 44));
    assert_eq!(look(HUMAN_F).apply(&record).eye_color, 44);
}

#[test]
fn model_customization_card_is_record_5() {
    let record = use_card(36023, &look(HUMAN_M));
    assert_eq!(record, expect(5, "human_m_facialhair_beard.adr", 1));
    // Stores also sell pixie wings under PARAM1 5 (36030: fairy_f_wings_butterfly).
    let record = use_card(36030, &look(FAIRY_F));
    assert_eq!(record, expect(5, "fairy_f_wings_butterfly.adr", 7));
    let after = look(FAIRY_M).apply(&record);
    assert_eq!(
        (
            after.model_customization.as_str(),
            after.ids[MODEL_CUSTOMIZATION]
        ),
        ("fairy_f_wings_butterfly.adr", 7)
    );
}

#[test]
fn face_paint_card_is_record_6() {
    let record = use_card(36004, &look(HUMAN_M));
    assert_eq!(record, expect(6, "skull", 1));
    let after = look(HUMAN_M).apply(&record);
    assert_eq!(
        (after.face_paint.as_str(), after.ids[FACE_PAINT]),
        ("skull", 1)
    );
}

#[test]
fn wings_card_is_record_7() {
    let record = use_card(77536, &look(FAIRY_M));
    assert_eq!(record, expect(7, "fairy_m_wings_creepy.adr", 13));
    let after = look(FAIRY_F).apply(&record);
    assert_eq!(after.model_customization, "fairy_m_wings_creepy.adr");
}

#[test]
fn face_type_card_is_record_0() {
    let record = use_card(77472, &look(HUMAN_M));
    assert_eq!(record, expect(0, "<race>_<gender>_head_caucasian.adr", 1));
    let after = look(HUMAN_M).apply(&record);
    assert_eq!(
        (after.head.as_str(), after.ids[HEAD]),
        (record.string_param.as_str(), 1)
    );
}

#[test]
fn clean_shaven_is_empty_string_without_lookup() {
    assert_eq!(Client.mapping(MODEL_CUSTOMIZATION, 0), None);
    let clean_shaven = card(36022);
    assert_eq!(
        clean_shaven,
        StyleCard {
            slot: 5,
            mapping: 0,
            gender: 1
        }
    );
    let record = use_card(36022, &look(HUMAN_M));
    assert_eq!(record, expect(5, "", 0));
    let after = look(HUMAN_M).apply(&record);
    assert_eq!(
        (
            after.model_customization.as_str(),
            after.ids[MODEL_CUSTOMIZATION]
        ),
        ("", 0)
    );
}

/// A long table row for the identity string bound.
struct Long(String);

impl Tables for Long {
    fn model(&self, _: i32) -> Option<(i32, &str)> {
        Some((1, "human_m"))
    }

    fn mapping(&self, _: usize, _: i32) -> Option<&str> {
        Some(&self.0)
    }
}

#[test]
fn ineligible_cards_are_refused() {
    let refused = [
        (card(36022), HUMAN_F, Refusal::WrongGender),
        (card(36022), FAIRY_M, Refusal::WrongModel),
        (card(77530), HUMAN_F, Refusal::WrongModel),
        (card(36030), HUMAN_F, Refusal::WrongModel),
        (card(77536), HUMAN_M, Refusal::WrongModel),
        (card(6826), HUMAN_F, Refusal::WrongGender),
        (card(78086), HUMAN_M, Refusal::UnsupportedSlot),
        (card(77641), HUMAN_M, Refusal::UnsupportedSlot),
        (card(77484), 3, Refusal::WrongModel),
    ];
    let any = |slot, mapping| StyleCard {
        slot,
        mapping,
        gender: 0,
    };
    let synthetic = [
        (any(2, 999), HUMAN_M, Refusal::UnknownMapping),
        (any(5, 46), HUMAN_M, Refusal::UnknownMapping),
        (any(7, 0), FAIRY_F, Refusal::UnknownMapping),
        (any(2, 1), HUMAN_F, Refusal::WrongModel),
        (any(2, 54), HUMAN_F, Refusal::WrongModel),
        (any(5, 1), FAIRY_M, Refusal::WrongModel),
        (any(10, 1), HUMAN_M, Refusal::UnsupportedSlot),
    ];
    for (card, model, refusal) in refused.into_iter().chain(synthetic) {
        let before = look(model);
        let seen = before.clone();
        assert_eq!(card.record(&seen, &Client, 900), Err(refusal), "{card:?}");
        assert_eq!(seen, before, "a refusal changes nothing");
    }
    assert_eq!(
        card(36004).record(&look(HUMAN_M), &Long("x".repeat(513)), 900),
        Err(Refusal::UnknownMapping)
    );
    assert!(
        card(36004)
            .record(&look(HUMAN_M), &Long("x".repeat(512)), 900)
            .is_ok()
    );
    let not_a_card = row(1);
    assert_eq!(
        StyleCard::from_row(&not_a_card),
        Err(Refusal::NotAStyleCard)
    );
    let short = row(36022);
    assert_eq!(
        StyleCard::from_row(&short[..short.len() - 13]),
        Err(Refusal::NotAStyleCard)
    );
}

#[test]
fn customization_data_round_trips() {
    let choices = [
        HUMAN_M, 2, 1, 1, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 6, 5, 1,
    ];
    let identity = CharacterIdentity {
        guid: 1,
        first_name: "A".into(),
        last_name: String::new(),
        model: HUMAN_M,
        head: "<race>_<gender>_head_african.adr".into(),
        hair: "<race>_m_hair_afro_<hairtype>.adr".into(),
        model_customization: "human_m_facialhair_beard.adr".into(),
        face_paint: "none".into(),
        skin_tone: "skintone3".into(),
        eye_color: 6,
        hair_color: 5,
        gender: 1,
        creation: NativeCreation {
            first_name: "A".into(),
            last_name: String::new(),
            temporary_first_name: String::new(),
            temporary_last_name: String::new(),
            choices,
        },
        attachments: Vec::new(),
    };
    assert_eq!(Look::from_identity(&identity), look(HUMAN_M));
    for (model, wings) in [(HUMAN_M, 5), (HUMAN_F, 5), (FAIRY_M, 7), (FAIRY_F, 7)] {
        let records = read_data(&customization_data(&look(model), &Client));
        let ids: Vec<_> = records.iter().map(|record| record.id).collect();
        assert_eq!(ids, [0, 1, 2, 3, 4, wings, 6, 8]);
        let rebuilt = records
            .iter()
            .fold(Look::default(), |look, r| look.apply(r));
        assert_eq!(rebuilt, look(model), "every field is sent");
    }
}
