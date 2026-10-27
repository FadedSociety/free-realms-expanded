//! Work item 049: recovered character appearance coverage and invalid-asset exclusions.
//!
//! Recovered original facts, all copied from the preserved client tables under
//! `lab/working/OSFR.Client/OSFR Client/Resources` and re-checked by
//! the private recovery checker `piece_049.py`:
//!
//! * Four creator character types exist (`CharacterCreateCharacterTypes.txt`): Human/Pixie x
//!   Male/Female with locale name ids 430557-430560. They bind one-to-one to the four
//!   `Models.txt` rows with `IS_VALID_FOR_PC = 1` (1 `human_m.adr`, 2 `fairy_m.adr`,
//!   60 `human_f.adr`, 61 `fairy_f.adr`). RACE_ID 1 is `Human` and RACE_ID 2 is `Pixie`
//!   (locale 2230/2231) even though race 2's model files are named `fairy_*`.
//! * Nine creator categories (`CharacterCreateCategories.txt`) each reference exactly one
//!   `CharacterCreateItemMappings.txt` TYPE: FaceType 8, SkinTone 0, HairStyle 1, HairColor 2,
//!   EyeColor 3, WingsPixie 6, FaceHair 4, Clothes 7, FacePaint 5. `FaceHair` is scoped
//!   Human/Male and `WingsPixie` Pixie/All; every other category is All/All.
//! * The `CharacterCreateCustomization*.txt` tables hold 253 creator rows: 67 Pixie Female,
//!   61 Human Female, 63 Human Male and 62 Pixie Male. Each row references a mapping row, which
//!   resolves to a type-12 `ItemDefinitions.txt` appearance definition, except clothing rows
//!   which resolve through `CharacterCreateItemClothingSets.txt` to three item definitions
//!   (slots 5/4/3).
//! * Twelve of the 253 rows resolve into the class-1 `CATEGORY_ID 141` definition family
//!   (78086-78091): the two extra skin tones (definitions 78090/78091, appearance indexes 7/8)
//!   and the four China head variants (78086-78089). The public/community creator tables remove
//!   exactly those twelve rows, and `lab/reports/creator-appearance-availability.json` records
//!   that Human male Caucasian head textures 7/8 are absent from the supplied client and the
//!   scanned public packs while 1-6 match the original manifest CRC. Run
//!   `client-baseline-20261008T065152Z` failed natively on `human_m_head_caucasian_08.dds`.
//! * The mapping table holds 272 rows; 223 are reachable from the creator tables and 49 are
//!   not (type 0: 16, type 1: 30, type 3: 1, type 4: 1, type 5: 1). The 30 unreachable type-1
//!   rows resolve to the `FRC` hair family 78037-78066, which no creator row offers.
//!
//! BOUNDARIES: this module is recovery metadata plus pure lookups. It grants nothing, sends
//! nothing and rejects nothing in production. Login creation enforces the same sets in
//! `identity::Resources::resolve` (T444), derived from the pinned creator tables rather than
//! from these constants; `tests/creator_allow_list.rs` asserts the two agree, and head 4 is
//! refused there because no pinned row reaches it. The China head exclusion carries
//! community-table evidence only; the skins 7/8 exclusion additionally carries absent-asset and
//! native-failure evidence. `HeadMappings.txt` id 4 has no creator row in either table and stays
//! an explicit gap in this module's lookups rather than being accepted or rejected here.

/// One creator character type and the PC model it binds to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CharacterType {
    /// `CharacterCreateCharacterTypes.txt` CHARACTER_TYPE_ID.
    pub id: i32,
    /// Locale name id (`430557` Human Male .. `430560` Pixie Female).
    pub name_id: i32,
    /// `Models.txt` RACE_ID.
    pub race_id: i32,
    /// Race display name recovered from the locale export via `RaceDefinitions.txt`.
    pub race: &'static str,
    /// `GenderDefinitions.txt` ID.
    pub gender_id: i32,
    pub gender: &'static str,
    /// `Models.txt` row with `IS_VALID_FOR_PC = 1`.
    pub model_id: i32,
    pub model_file: &'static str,
}

const fn character_type_row(
    id: i32,
    name_id: i32,
    race_id: i32,
    race: &'static str,
    gender_id: i32,
    gender: &'static str,
    model_id: i32,
    model_file: &'static str,
) -> CharacterType {
    CharacterType {
        id,
        name_id,
        race_id,
        race,
        gender_id,
        gender,
        model_id,
        model_file,
    }
}

/// The four recovered creator character types, in `CharacterCreateCharacterTypes.txt` order.
pub const CHARACTER_TYPES: [CharacterType; 4] = [
    character_type_row(1, 430560, 2, "Pixie", 2, "Female", 61, "fairy_f.adr"),
    character_type_row(2, 430558, 1, "Human", 2, "Female", 60, "human_f.adr"),
    character_type_row(3, 430557, 1, "Human", 1, "Male", 1, "human_m.adr"),
    character_type_row(4, 430559, 2, "Pixie", 1, "Male", 2, "fairy_m.adr"),
];

/// One creator customization category and its recovered option counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CreatorCategory {
    /// `CharacterCreateCategories.txt` CATEGORY_ID.
    pub id: i32,
    /// Category title locale id.
    pub name_id: i32,
    /// `DATA_SOURCE_NAME_SUFFIX`, also the `CharacterCreateCustomization<SUFFIX>.txt` infix.
    pub suffix: &'static str,
    /// Category scope (`All`, `Human` or `Pixie`).
    pub scope_race: &'static str,
    /// Category scope (`All`, `Male` or `Female`).
    pub scope_gender: &'static str,
    /// The single `CharacterCreateItemMappings.txt` TYPE every row of this category references.
    pub mapping_type: i32,
    /// Creator rows per `CHARACTER_TYPES` index.
    pub rows_per_type: [u8; 4],
    /// Excluded creator rows per `CHARACTER_TYPES` index (subset of `rows_per_type`).
    pub excluded_per_type: [u8; 4],
}

#[allow(clippy::too_many_arguments)]
const fn category_row(
    id: i32,
    name_id: i32,
    suffix: &'static str,
    scope_race: &'static str,
    scope_gender: &'static str,
    mapping_type: i32,
    rows_per_type: [u8; 4],
    excluded_per_type: [u8; 4],
) -> CreatorCategory {
    CreatorCategory {
        id,
        name_id,
        suffix,
        scope_race,
        scope_gender,
        mapping_type,
        rows_per_type,
        excluded_per_type,
    }
}

/// The nine recovered creator categories, in `CharacterCreateCategories.txt` id order.
pub const CREATOR_CATEGORIES: [CreatorCategory; 9] = [
    category_row(
        1,
        434053,
        "FaceType",
        "All",
        "All",
        8,
        [4, 4, 4, 4],
        [1, 1, 1, 1],
    ),
    category_row(
        2,
        434046,
        "SkinTone",
        "All",
        "All",
        0,
        [8, 8, 8, 8],
        [2, 2, 2, 2],
    ),
    category_row(
        3,
        434047,
        "HairStyle",
        "All",
        "All",
        1,
        [8, 8, 8, 8],
        [0, 0, 0, 0],
    ),
    category_row(
        4,
        434048,
        "HairColor",
        "All",
        "All",
        2,
        [10, 10, 10, 10],
        [0, 0, 0, 0],
    ),
    category_row(
        5,
        434049,
        "EyeColor",
        "All",
        "All",
        3,
        [10, 10, 10, 10],
        [0, 0, 0, 0],
    ),
    category_row(
        6,
        434051,
        "WingsPixie",
        "Pixie",
        "All",
        6,
        [6, 0, 0, 6],
        [0, 0, 0, 0],
    ),
    category_row(
        7,
        156816,
        "FaceHair",
        "Human",
        "Male",
        4,
        [0, 0, 7, 0],
        [0, 0, 0, 0],
    ),
    category_row(
        8,
        434052,
        "Clothes",
        "All",
        "All",
        7,
        [9, 9, 9, 9],
        [0, 0, 0, 0],
    ),
    category_row(
        9,
        434050,
        "FacePaint",
        "All",
        "All",
        5,
        [12, 12, 7, 7],
        [0, 0, 0, 0],
    ),
];

/// Why a recovered creator option is not supported.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExclusionReason {
    /// The choice selects a head texture that is absent from the supplied client and the scanned
    /// public packs (skin tones 7/8; native run `client-baseline-20261008T065152Z` 404s on
    /// `human_m_head_caucasian_08.dds`).
    SkinTextureAbsent,
    /// The choice resolves into the class-1 `CATEGORY_ID 141` definition family that the public
    /// creator tables remove. For the four China head variants no missing asset was proven.
    NotOfferedByPublicTables,
}

/// One creator row that is present in the supplied tables but not supported.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExcludedCreatorRow {
    /// Category suffix (`SkinTone` or `FaceType`).
    pub category: &'static str,
    pub race: &'static str,
    pub gender: &'static str,
    /// `CharacterCreateCustomization*.txt` GUID.
    pub guid: i32,
    /// `CharacterCreateItemMappings.txt` row the creator row references.
    pub mapping_id: i32,
    /// `ItemDefinitions.txt` appearance definition the mapping row selects.
    pub definition_id: i32,
    /// Definition `PARAM2`, the appearance index the wire ultimately carries.
    pub appearance_index: i32,
    /// Definition `MODEL_NAME` (`SkinTone 8` or `<race>_<gender>_head_china.adr`).
    pub model_name: &'static str,
    pub reason: ExclusionReason,
}

#[allow(clippy::too_many_arguments)]
const fn excluded(
    category: &'static str,
    race: &'static str,
    gender: &'static str,
    guid: i32,
    mapping_id: i32,
    definition_id: i32,
    appearance_index: i32,
    model_name: &'static str,
    reason: ExclusionReason,
) -> ExcludedCreatorRow {
    ExcludedCreatorRow {
        category,
        race,
        gender,
        guid,
        mapping_id,
        definition_id,
        appearance_index,
        model_name,
        reason,
    }
}

/// The twelve creator rows the public tables remove, in character-type order
/// (Pixie Female, Human Female, Human Male, Pixie Male) then category order.
pub const EXCLUDED_CREATOR_ROWS: [ExcludedCreatorRow; 12] = [
    excluded(
        "SkinTone",
        "Pixie",
        "Female",
        1000001,
        272,
        78090,
        7,
        "SkinTone 7",
        ExclusionReason::SkinTextureAbsent,
    ),
    excluded(
        "SkinTone",
        "Pixie",
        "Female",
        1000002,
        1,
        78091,
        8,
        "SkinTone 8",
        ExclusionReason::SkinTextureAbsent,
    ),
    excluded(
        "SkinTone",
        "Human",
        "Female",
        1000007,
        272,
        78090,
        7,
        "SkinTone 7",
        ExclusionReason::SkinTextureAbsent,
    ),
    excluded(
        "SkinTone",
        "Human",
        "Female",
        1000008,
        1,
        78091,
        8,
        "SkinTone 8",
        ExclusionReason::SkinTextureAbsent,
    ),
    excluded(
        "SkinTone",
        "Human",
        "Male",
        1000013,
        272,
        78090,
        7,
        "SkinTone 7",
        ExclusionReason::SkinTextureAbsent,
    ),
    excluded(
        "SkinTone",
        "Human",
        "Male",
        1000014,
        1,
        78091,
        8,
        "SkinTone 8",
        ExclusionReason::SkinTextureAbsent,
    ),
    excluded(
        "SkinTone",
        "Pixie",
        "Male",
        1000019,
        272,
        78090,
        7,
        "SkinTone 7",
        ExclusionReason::SkinTextureAbsent,
    ),
    excluded(
        "SkinTone",
        "Pixie",
        "Male",
        1000020,
        1,
        78091,
        8,
        "SkinTone 8",
        ExclusionReason::SkinTextureAbsent,
    ),
    excluded(
        "FaceType",
        "Pixie",
        "Female",
        1000001,
        268,
        78086,
        5,
        "fairy_f_head_china.adr",
        ExclusionReason::NotOfferedByPublicTables,
    ),
    excluded(
        "FaceType",
        "Pixie",
        "Male",
        1000005,
        269,
        78087,
        6,
        "fairy_m_head_china.adr",
        ExclusionReason::NotOfferedByPublicTables,
    ),
    excluded(
        "FaceType",
        "Human",
        "Male",
        1000009,
        270,
        78088,
        7,
        "human_m_head_china.adr",
        ExclusionReason::NotOfferedByPublicTables,
    ),
    excluded(
        "FaceType",
        "Human",
        "Female",
        1000013,
        271,
        78089,
        8,
        "human_f_head_china.adr",
        ExclusionReason::NotOfferedByPublicTables,
    ),
];

/// `SkinToneMappings.txt` ids the creator offers (the six standard skin tones).
pub const SUPPORTED_SKIN_TONE_IDS: [i32; 6] = [1, 2, 3, 4, 5, 6];
/// `SkinToneMappings.txt` ids the extra creator rows select; their head textures are absent.
pub const EXCLUDED_SKIN_TONE_IDS: [i32; 2] = [7, 8];
/// `HeadMappings.txt` ids the creator offers (Caucasian, African, Asian).
pub const SUPPORTED_HEAD_IDS: [i32; 3] = [1, 2, 3];
/// `HeadMappings.txt` China head ids the public creator tables no longer offer.
pub const EXCLUDED_HEAD_IDS: [i32; 4] = [5, 6, 7, 8];

/// Distinct `CharacterCreateItemMappings.txt` rows reachable from the creator tables.
pub const OFFERED_MAPPING_ROWS: usize = 223;
/// Mapping rows no creator row references, as `(mapping type, row count)`.
pub const UNOFFERED_MAPPING_ROWS: [(i32, usize); 5] = [(0, 16), (1, 30), (3, 1), (4, 1), (5, 1)];
/// The unreachable type-1 rows resolve to this `FRC` hair definition range (inclusive).
pub const FRC_HAIR_DEFINITION_RANGE: [i32; 2] = [78037, 78066];
/// Definitions with `ITEM_TYPE 12`, `CATEGORY_ID 141` and `ITEM_CLASS 1` in the supplied table.
pub const CREATOR_FAMILY_DEFINITION_COUNT: usize = 36;
/// The class-1 `CATEGORY_ID 141` definitions the twelve excluded rows resolve into.
pub const EXCLUDED_FAMILY_DEFINITION_IDS: [i32; 6] = [78086, 78087, 78088, 78089, 78090, 78091];

/// The character type carrying `id`, if any.
pub fn character_type(id: i32) -> Option<&'static CharacterType> {
    CHARACTER_TYPES.iter().find(|row| row.id == id)
}

/// The category carrying `suffix`, if any.
pub fn category(suffix: &str) -> Option<&'static CreatorCategory> {
    CREATOR_CATEGORIES.iter().find(|row| row.suffix == suffix)
}

/// Every excluded creator row of one category.
pub fn excluded_rows_for(category: &str) -> Vec<&'static ExcludedCreatorRow> {
    EXCLUDED_CREATOR_ROWS
        .iter()
        .filter(|row| row.category == category)
        .collect()
}

/// Whether a `SkinToneMappings.txt` id is supported by the supplied assets.
pub fn skin_tone_exclusion(id: i32) -> Option<ExclusionReason> {
    EXCLUDED_SKIN_TONE_IDS
        .contains(&id)
        .then_some(ExclusionReason::SkinTextureAbsent)
}

/// Whether a `HeadMappings.txt` id is offered by the recovered creator tables. Id 4 exists in
/// the mapping table but carries no creator row; it is a reported gap, not a rejection.
pub fn head_exclusion(id: i32) -> Option<ExclusionReason> {
    EXCLUDED_HEAD_IDS
        .contains(&id)
        .then_some(ExclusionReason::NotOfferedByPublicTables)
}

/// Structural invariants over the recovered tables.
pub fn validate() -> Result<(), &'static str> {
    if CHARACTER_TYPES.len() != 4 {
        return Err("expected four creator character types");
    }
    for (index, row) in CHARACTER_TYPES.iter().enumerate() {
        if row.id <= 0 || row.name_id <= 0 || row.model_id <= 0 || row.model_file.is_empty() {
            return Err("incomplete character type row");
        }
        if !matches!(row.race_id, 1 | 2) || !matches!(row.gender_id, 1 | 2) {
            return Err("character type uses an unrecovered race or gender");
        }
        if CHARACTER_TYPES[..index].iter().any(|other| {
            other.id == row.id || other.model_id == row.model_id || other.name_id == row.name_id
        }) {
            return Err("duplicate character type identity");
        }
    }
    if CREATOR_CATEGORIES.len() != 9 {
        return Err("expected nine creator categories");
    }
    for (index, row) in CREATOR_CATEGORIES.iter().enumerate() {
        if row.id <= 0 || row.name_id <= 0 || row.suffix.is_empty() {
            return Err("incomplete creator category row");
        }
        if !(0..=8).contains(&row.mapping_type) {
            return Err("creator category carries an unrecovered mapping type");
        }
        if CREATOR_CATEGORIES[..index]
            .iter()
            .any(|other| other.id == row.id || other.mapping_type == row.mapping_type)
        {
            return Err("creator categories do not have unique ids and mapping types");
        }
        for (slot, (rows, excluded)) in row
            .rows_per_type
            .iter()
            .zip(row.excluded_per_type.iter())
            .enumerate()
        {
            if excluded > rows {
                return Err("more excluded rows than creator rows");
            }
            if *excluded > 0 && CHARACTER_TYPES[slot].race.is_empty() {
                return Err("excluded rows without a character type");
            }
        }
    }
    if EXCLUDED_CREATOR_ROWS.len() != 12 {
        return Err("expected twelve excluded creator rows");
    }
    for (index, row) in EXCLUDED_CREATOR_ROWS.iter().enumerate() {
        if row.guid <= 0 || row.mapping_id <= 0 || row.definition_id <= 0 {
            return Err("incomplete excluded creator row");
        }
        if row.model_name.is_empty() || row.race.is_empty() || row.gender.is_empty() {
            return Err("excluded creator row is missing recovered text");
        }
        if EXCLUDED_CREATOR_ROWS[..index]
            .iter()
            .any(|other| other.category == row.category && other.guid == row.guid)
        {
            return Err("duplicate excluded creator row");
        }
        match row.reason {
            ExclusionReason::SkinTextureAbsent if row.category != "SkinTone" => {
                return Err("skin texture exclusion on a non-skin category");
            }
            ExclusionReason::NotOfferedByPublicTables if row.category == "SkinTone" => {
                return Err("skin row without the absent-texture reason");
            }
            _ => {}
        }
    }
    let rows: usize = CREATOR_CATEGORIES
        .iter()
        .map(|row| row.rows_per_type.iter().map(|n| *n as usize).sum::<usize>())
        .sum();
    let excluded: usize = CREATOR_CATEGORIES
        .iter()
        .map(|row| {
            row.excluded_per_type
                .iter()
                .map(|n| *n as usize)
                .sum::<usize>()
        })
        .sum();
    if rows != 253 || excluded != EXCLUDED_CREATOR_ROWS.len() {
        return Err("creator row counts disagree with the excluded row table");
    }
    let unoffered: usize = UNOFFERED_MAPPING_ROWS.iter().map(|(_, count)| *count).sum();
    if OFFERED_MAPPING_ROWS + unoffered != 272 {
        return Err("mapping row counts disagree with the mapping table");
    }
    if FRC_HAIR_DEFINITION_RANGE[0] > FRC_HAIR_DEFINITION_RANGE[1] {
        return Err("FRC hair definition range is inverted");
    }
    Ok(())
}
