//! Piece 059 - dungeon definitions: the recovered roster, difficulty, entrances and maps.
//!
//! ORIGINAL facts implemented here (provenance in
//! `notes/Research/Pieces/059 Dungeon definitions.md`, evidence bundle
//! `lab/reports/piece-059-20261009T054317Z/`):
//!
//! * The read-only wiki corpus `Dungeons` page lists 43 combat dungeons in five ranked
//!   difficulty sections; `Hewey's Escape` and `King of the Ring!` are marked "(removed)".
//!   Difficulty 1 is open to anyone, 2 recommends level 3, and 3/4/5 require a combat job of
//!   level 5/9/13. That page is community documentation, not first-party data.
//! * The preserved locale export carries one contiguous achievement block, 30439-30475, of
//!   "Complete the <name> encounter." rows for 37 of the 43 names. The block order matches the
//!   roster's difficulty order, so each named dungeon's row number is recovered evidence.
//! * Each dungeon page carries the entrance sentence ("started by clicking on the ... in
//!   <place>"). `place` is kept verbatim; `region` is this module's inference from the wiki's
//!   own location vocabulary.
//! * The pinned pack catalogue holds a zone definition (`*.gzne`, header `GZNE` v2) for 14 of
//!   the roster - `zone` below - plus a 84-byte header-only `mv_den_of_secrets.gzne` stub for
//!   Den of Secrets. Zone and map-chunk byte counts are the catalogue's, not estimates.
//!
//! RECONSTRUCTED, labelled: nothing in this module changes gameplay. [`entry_rule`] restates
//! the wiki difficulty policy for the lead to enforce; it is not recovered server code.
//!
//! NOT CLAIMED: encounter scripts, spawn tables, wave/gate logic, rewards, per-dungeon player
//! counts and collision data. 29 dungeons still have no recovered zone definition, and Den of
//! Secrets has map chunks but only a stub definition. Both stay explicit gaps.

/// Zone definition sizes at or below this are header-only stubs, not recovered maps.
pub const ZONE_STUB_LIMIT: u32 = 84;
/// Lowest dungeon difficulty rank on the recovered roster.
pub const MIN_DIFFICULTY: u8 = 1;
/// Highest dungeon difficulty rank on the recovered roster.
pub const MAX_DIFFICULTY: u8 = 5;
/// First locale row of the recovered "Complete the <name> encounter." block.
pub const OBJECTIVE_LOCALE_FIRST: u32 = 30439;
/// Last locale row of the recovered "Complete the <name> encounter." block.
pub const OBJECTIVE_LOCALE_LAST: u32 = 30475;
/// Rosters per difficulty rank, in rank order, as listed by the wiki page.
pub const DIFFICULTY_MEMBERSHIP: [usize; 5] = [8, 8, 9, 9, 9];

/// A recovered zone definition and its map chunks from the pinned pack catalogue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZoneDefinition {
    /// Zone stem shared by `*.gzne` and `*_<col>_<row>.gcnk` entries.
    pub stem: &'static str,
    /// Bytes of the `*.gzne` zone definition.
    pub zone_bytes: u32,
    /// Map chunk (`*.gcnk`) count, excluding minimap tiles.
    pub map_chunks: u32,
    /// Total map chunk bytes.
    pub map_bytes: u32,
}

/// The wiki entrance sentence, split into its parts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DungeonEntrance {
    /// The object the player clicks, verbatim from the sentence.
    pub trigger: &'static str,
    /// Containing area, when the sentence names one ("located in ...").
    pub area: Option<&'static str>,
    /// Place text verbatim, minus the leading connector.
    pub place: &'static str,
    /// Inferred region from [`REGIONS`]; `None` when the sentence names no known region.
    pub region: Option<&'static str>,
}

/// One dungeon roster row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DungeonDefinition {
    /// Roster index, 1-based, in the wiki's difficulty order.
    pub id: u32,
    pub name: &'static str,
    pub difficulty: u8,
    /// The wiki page marks the row "(removed)".
    pub removed: bool,
    /// "Complete the <name> encounter." locale row, when the achievement block covers it.
    pub objective_locale: Option<u32>,
    /// Recovered zone definition, when the catalogue holds a real one.
    pub zone: Option<ZoneDefinition>,
    /// Zone stem whose only catalogue definition is a header-only stub.
    pub stub_zone: Option<&'static str>,
    /// Recovered entrance sentence, when the page carries a usable one.
    pub entrance: Option<DungeonEntrance>,
}

/// Shorthand for a recovered zone definition inside [`DUNGEONS`].
#[allow(non_snake_case)]
const fn Z(
    stem: &'static str,
    zone_bytes: u32,
    map_chunks: u32,
    map_bytes: u32,
) -> Option<ZoneDefinition> {
    Some(ZoneDefinition {
        stem,
        zone_bytes,
        map_chunks,
        map_bytes,
    })
}

/// Shorthand for a recovered entrance sentence inside [`DUNGEONS`].
#[allow(non_snake_case)]
const fn E(
    trigger: &'static str,
    area: Option<&'static str>,
    place: &'static str,
    region: Option<&'static str>,
) -> Option<DungeonEntrance> {
    Some(DungeonEntrance {
        trigger,
        area,
        place,
        region,
    })
}

/// The wiki's own region vocabulary. `region` is the first of these a place text mentions.
pub const REGIONS: [&str; 13] = [
    "Sanctuary",
    "Snowhill",
    "Wilds",
    "Seaside",
    "Blackspore Swamp",
    "Briarwood",
    "Wugachug",
    "Merry Vale",
    "Shrouded Glade",
    "Sunstone Valley",
    "Crossroads",
    "Highroad Junction",
    "Stillwater Crossing",
];

/// Every recovered roster row, in the wiki's order.
#[rustfmt::skip]
pub const DUNGEONS: [DungeonDefinition; 43] = [
    row(1, "Bixie Hive", 1, false, None, Z("sg_bixie_hive", 1944, 9, 194_780), None, E("door", Some("Queensfields"), "Sanctuary", Some("Sanctuary"))),
    row(2, "Cracked Claw Caverns", 1, false, Some(30439), None, None, E("barrier", None, "Blackspore Swamp", Some("Blackspore Swamp"))),
    row(3, "Cursed Graveyard!", 1, false, None, None, None, E("gate", None, "Stillwater Crossing", Some("Stillwater Crossing"))),
    row(4, "Hewey's Escape", 1, true, None, None, None, E("barrier", None, "Highroad Junction", Some("Highroad Junction"))),
    row(5, "Hot Springs Haven", 1, false, Some(30440), None, None, E("barrier", None, "Snowhill", Some("Snowhill"))),
    row(6, "King of the Ring!", 1, true, None, None, None, E("gate", None, "Crossroads", Some("Crossroads"))),
    row(7, "Sheep Watch", 1, false, Some(30441), None, None, E("gate", None, "southwest Wilds, near Farnum's Farm", Some("Wilds"))),
    row(8, "Tavern Cellar", 1, false, Some(30442), None, None, E("cellar door", None, "northwest Wilds", Some("Wilds"))),
    row(9, "Arachnia's Lair", 2, false, Some(30443), Z("bw_spider_lair", 2431, 4, 2_697_568), None, E("spider web", None, "Bristlewood, Briarwood", Some("Briarwood"))),
    row(10, "Danger Peaks", 2, false, Some(30444), None, None, E("fence", None, "southwest Wilds", Some("Wilds"))),
    row(11, "Den of Secrets", 2, false, Some(30445), None, Some("mv_den_of_secrets"), E("barrier", None, "Merry Vale", Some("Merry Vale"))),
    row(12, "Highroad Hijinx", 2, false, Some(30446), Z("sg_highroad_hijinx", 3805, 25, 8_083_144), None, E("fence", None, "southwest Wilds, near Farnum's Farm", Some("Wilds"))),
    row(13, "Robgoblin Treasure Trove", 2, false, Some(30447), None, None, E("gate", None, "northeast Wilds, at the Robgoblin Camp", Some("Wilds"))),
    row(14, "Sweetwater Climb", 2, false, Some(30448), Z("wc_sweetwater_climb", 5438, 30, 6_251_162), None, E("gate", None, "Wugachug", Some("Wugachug"))),
    row(15, "The Bat Cave!", 2, false, Some(30449), None, None, E("barrier", Some("Singing Crystal Mines"), "Snowhill", Some("Snowhill"))),
    row(16, "Sandscale Oasis", 2, false, None, None, None, E("gate", None, "Sunstone Valley", Some("Sunstone Valley"))),
    row(17, "Bone Bog Cemetery", 3, false, Some(30450), Z("bs_bone_bog_cemetery", 7793, 30, 14_619_138), None, E("gate", None, "Blackspore Swamp", Some("Blackspore Swamp"))),
    row(18, "Bristlewood Glade", 3, false, Some(30451), None, None, E("barrier", None, "Bristlewood, Briarwood", Some("Briarwood"))),
    row(19, "Deep Mines", 3, false, Some(30452), None, None, E("barrier", Some("Singing Crystal Mines"), "Snowhill", Some("Snowhill"))),
    row(20, "Floren Forest", 3, false, Some(30453), Z("sg_floren_forest", 2027, 0, 0), None, E("gate", None, "northwest Wilds", Some("Wilds"))),
    row(21, "Forgotten Caves", 3, false, Some(30454), None, None, E("barrier", Some("Memorial Caverns"), "Sanctuary", Some("Sanctuary"))),
    row(22, "Howling Hills", 3, false, Some(30455), None, None, E("barrier", None, "Snowhill", Some("Snowhill"))),
    row(23, "Misty Mountain", 3, false, Some(30456), Z("gl_misty_mountain", 1940, 16, 15_959), None, E("gate", None, "Shrouded Glade", Some("Shrouded Glade"))),
    row(24, "Mushroom Cavern", 3, false, Some(30457), None, None, E("barrier", None, "Nettleseed, Briarwood", Some("Briarwood"))),
    row(25, "The Snarling Hedges", 3, false, Some(30458), None, None, E("gate", None, "Thistlerow, Briarwood", Some("Briarwood"))),
    row(26, "Bandit Hideout!", 4, false, Some(30459), Z("sg_bandit_hideout", 664, 2, 28_728), None, E("barrier", None, "Greenwood Camp, in the southwest Wilds", Some("Wilds"))),
    row(27, "Briar Patch", 4, false, Some(30460), None, None, E("thorn plants", None, "north end of Bristlewood, Briarwood", Some("Briarwood"))),
    row(28, "Croaking Vale", 4, false, Some(30461), None, None, E("barrier", None, "Nettleseed, Briarwood", Some("Briarwood"))),
    row(29, "Darvon's Descent", 4, false, Some(30462), None, None, E("barrier", Some("Memorial Caverns"), "Sanctuary", Some("Sanctuary"))),
    row(30, "Frostfang Caverns", 4, false, Some(30463), Z("sh_frostfang_cavern", 1656, 9, 20_288), None, E("barrier", None, "Snowhill", Some("Snowhill"))),
    row(31, "Grexan's Camp", 4, false, Some(30464), None, None, E("spider web", None, "northeast Wilds", Some("Wilds"))),
    row(32, "Mugwort's Hollow", 4, false, Some(30465), Z("sg_mugworts_hollow", 1552, 7, 7049), None, E("barrier", None, "northeast Wilds, along the mountain face between Snowhill and Shrouded Glade", Some("Wilds"))),
    row(33, "Treeleaf's Retreat", 4, false, Some(30466), Z("bw_treeleaf_retreat", 3356, 4, 2_678_705), None, None),
    row(34, "Vale of the Ancients", 4, false, Some(30467), None, None, E("barrier", None, "Bristlewood, Briarwood", Some("Briarwood"))),
    row(35, "Briarheart Caverns", 5, false, Some(30468), None, None, E("barrier", None, "Briarheart, Briarwood", Some("Briarwood"))),
    row(36, "Cray Caves", 5, false, Some(30469), Z("ss_cray_caves", 3056, 4, 123_790), None, E("barrier", None, "Seaside", Some("Seaside"))),
    row(37, "Dark Spore Depths", 5, false, Some(30470), None, None, E("barrier", None, "Nettleseed, Briarwood", Some("Briarwood"))),
    row(38, "Forest Troll Fort", 5, false, Some(30471), None, None, E("wood barrier", None, "northeast Wilds", Some("Wilds"))),
    row(39, "Haunted Mines", 5, false, Some(30472), None, None, E("barrier", None, "Blackspore Swamp", Some("Blackspore Swamp"))),
    row(40, "Precursor Ruins", 5, false, None, None, None, None),
    row(41, "Snowy Canyon", 5, false, Some(30473), None, None, E("barrier", None, "Snowhill", Some("Snowhill"))),
    row(42, "Tanglewood Fort", 5, false, Some(30474), Z("bw_tanglewood_fort", 3331, 9, 4_138_889), None, E("gate", None, "Nettleseed, Briarwood", Some("Briarwood"))),
    row(43, "Trail of Betrayal", 5, false, Some(30475), Z("bw_trail_of_betrayal", 4078, 35, 6_629_059), None, E("barrier", None, "Nettleseed, Briarwood", Some("Briarwood"))),
];

#[allow(clippy::too_many_arguments)]
const fn row(
    id: u32,
    name: &'static str,
    difficulty: u8,
    removed: bool,
    objective_locale: Option<u32>,
    zone: Option<ZoneDefinition>,
    stub_zone: Option<&'static str>,
    entrance: Option<DungeonEntrance>,
) -> DungeonDefinition {
    DungeonDefinition {
        id,
        name,
        difficulty,
        removed,
        objective_locale,
        zone,
        stub_zone,
        entrance,
    }
}

/// The whole recovered roster.
pub fn all() -> &'static [DungeonDefinition; 43] {
    &DUNGEONS
}

/// Look up a roster row by its 1-based id.
pub fn by_id(id: u32) -> Option<&'static DungeonDefinition> {
    DUNGEONS.iter().find(|row| row.id == id)
}

/// Look up a roster row by name (ASCII case-insensitive, exact otherwise).
pub fn by_name(name: &str) -> Option<&'static DungeonDefinition> {
    DUNGEONS
        .iter()
        .find(|row| row.name.eq_ignore_ascii_case(name))
}

/// Rows at one difficulty rank, in roster order.
pub fn with_difficulty(difficulty: u8) -> Vec<&'static DungeonDefinition> {
    DUNGEONS
        .iter()
        .filter(|row| row.difficulty == difficulty)
        .collect()
}

/// Rows the wiki still lists as live content.
pub fn active() -> Vec<&'static DungeonDefinition> {
    DUNGEONS.iter().filter(|row| !row.removed).collect()
}

/// Rows the wiki marks "(removed)".
pub fn removed() -> Vec<&'static DungeonDefinition> {
    DUNGEONS.iter().filter(|row| row.removed).collect()
}

/// Rows with a real recovered zone definition.
pub fn recovered_zone_maps() -> Vec<&'static DungeonDefinition> {
    DUNGEONS.iter().filter(|row| row.zone.is_some()).collect()
}

/// The achievement locale row for a dungeon id, when the recovered block covers it.
pub fn objective_locale(id: u32) -> Option<u32> {
    by_id(id).and_then(|row| row.objective_locale)
}

/// What the wiki's difficulty rank means for entry. Community policy restated, not server code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntryRule {
    /// Combat job level required to enter, when entry is gated at all.
    pub min_combat_level: Option<u8>,
    /// True when the rank only recommends a level instead of requiring one.
    pub recommended_only: bool,
}

/// Entry policy for one difficulty rank.
pub fn entry_rule(difficulty: u8) -> Option<EntryRule> {
    match difficulty {
        1 => Some(EntryRule {
            min_combat_level: None,
            recommended_only: true,
        }),
        2 => Some(EntryRule {
            min_combat_level: Some(3),
            recommended_only: true,
        }),
        3 => Some(EntryRule {
            min_combat_level: Some(5),
            recommended_only: false,
        }),
        4 => Some(EntryRule {
            min_combat_level: Some(9),
            recommended_only: false,
        }),
        5 => Some(EntryRule {
            min_combat_level: Some(13),
            recommended_only: false,
        }),
        _ => None,
    }
}

/// Counted facts about a roster table, produced by [`validate`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CatalogSummary {
    pub dungeons: usize,
    pub active: usize,
    pub removed: usize,
    pub recovered_zone_maps: usize,
    pub stub_zone_definitions: usize,
    pub objective_locales: usize,
    pub missing_entrances: usize,
    pub missing_zone_maps: usize,
    pub difficulty_membership: [usize; 5],
}

/// Why a roster table failed validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CatalogError {
    /// Ids must be the 1-based row index in roster order.
    IdNotContiguous {
        expected: u32,
        found: u32,
    },
    DifficultyOutOfRange {
        id: u32,
        difficulty: u8,
    },
    DuplicateName(&'static str),
    /// Two dungeons cannot claim the same zone stem.
    DuplicateZoneStem(&'static str),
    /// A recovered objective row must be the block row for that id.
    ObjectiveLocaleMisaligned {
        id: u32,
        expected: u32,
        found: u32,
    },
    /// Removed content carries no achievement row.
    RemovedDungeonWithObjective(&'static str),
    /// A row cannot carry both a real definition and a stub for the same dungeon.
    ZoneAndStubOnSameRow(&'static str),
    /// A zone stem must be a plain slash-free token.
    ZoneStemShape(&'static str),
    /// The per-rank roster counts drifted from the wiki page.
    DifficultyMembershipMismatch {
        expected: [usize; 5],
        found: [usize; 5],
    },
}

impl std::fmt::Display for CatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CatalogError::IdNotContiguous { expected, found } => {
                write!(formatter, "expected dungeon id {expected}, found {found}")
            }
            CatalogError::DifficultyOutOfRange { id, difficulty } => {
                write!(
                    formatter,
                    "dungeon {id} has difficulty {difficulty} outside 1..=5"
                )
            }
            CatalogError::DuplicateName(name) => write!(formatter, "dungeon {name} listed twice"),
            CatalogError::DuplicateZoneStem(stem) => {
                write!(formatter, "zone stem {stem} claimed by two dungeons")
            }
            CatalogError::ObjectiveLocaleMisaligned {
                id,
                expected,
                found,
            } => write!(
                formatter,
                "dungeon {id} expects objective locale {expected}, found {found}"
            ),
            CatalogError::RemovedDungeonWithObjective(name) => {
                write!(
                    formatter,
                    "removed dungeon {name} carries an achievement row"
                )
            }
            CatalogError::ZoneAndStubOnSameRow(name) => {
                write!(formatter, "dungeon {name} carries both a zone and a stub")
            }
            CatalogError::ZoneStemShape(stem) => {
                write!(formatter, "zone stem {stem:?} is not a plain token")
            }
            CatalogError::DifficultyMembershipMismatch { expected, found } => write!(
                formatter,
                "difficulty membership {found:?} does not match the wiki page {expected:?}"
            ),
        }
    }
}

impl std::error::Error for CatalogError {}

fn zone_stem_shaped(stem: &str) -> bool {
    !stem.is_empty()
        && stem.contains('_')
        && stem.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Validate a roster table and count what it covers.
///
/// Fails closed on an id, difficulty, name, objective row or zone stem that does not match the
/// recovered shape, so a hand-edited table cannot silently drift from the evidence.
pub fn validate(table: &[DungeonDefinition]) -> Result<CatalogSummary, CatalogError> {
    let mut seen_names = std::collections::BTreeSet::new();
    let mut seen_stems = std::collections::BTreeSet::new();
    // The achievement block is contiguous and ascending in roster order; rows the block does
    // not cover (three live dungeons, two removed rows and Bixie Hive) are simply skipped.
    let mut next_block_row = OBJECTIVE_LOCALE_FIRST;
    let mut summary = CatalogSummary {
        dungeons: table.len(),
        active: 0,
        removed: 0,
        recovered_zone_maps: 0,
        stub_zone_definitions: 0,
        objective_locales: 0,
        missing_entrances: 0,
        missing_zone_maps: 0,
        difficulty_membership: [0; MAX_DIFFICULTY as usize],
    };
    for (index, row) in table.iter().enumerate() {
        let expected = index as u32 + 1;
        if row.id != expected {
            return Err(CatalogError::IdNotContiguous {
                expected,
                found: row.id,
            });
        }
        if row.difficulty < MIN_DIFFICULTY || row.difficulty > MAX_DIFFICULTY {
            return Err(CatalogError::DifficultyOutOfRange {
                id: row.id,
                difficulty: row.difficulty,
            });
        }
        if !seen_names.insert(row.name) {
            return Err(CatalogError::DuplicateName(row.name));
        }
        if let Some(zone) = row.zone {
            if !zone_stem_shaped(zone.stem) {
                return Err(CatalogError::ZoneStemShape(zone.stem));
            }
            if zone.zone_bytes <= ZONE_STUB_LIMIT {
                return Err(CatalogError::ZoneStemShape(zone.stem));
            }
            if !seen_stems.insert(zone.stem) {
                return Err(CatalogError::DuplicateZoneStem(zone.stem));
            }
        }
        if let Some(stub) = row.stub_zone {
            if !zone_stem_shaped(stub) {
                return Err(CatalogError::ZoneStemShape(stub));
            }
            if row.zone.is_some() {
                return Err(CatalogError::ZoneAndStubOnSameRow(row.name));
            }
            if !seen_stems.insert(stub) {
                return Err(CatalogError::DuplicateZoneStem(stub));
            }
        }
        if let Some(locale) = row.objective_locale {
            if row.removed {
                return Err(CatalogError::RemovedDungeonWithObjective(row.name));
            }
            if locale != next_block_row || locale > OBJECTIVE_LOCALE_LAST {
                return Err(CatalogError::ObjectiveLocaleMisaligned {
                    id: row.id,
                    expected: next_block_row,
                    found: locale,
                });
            }
            next_block_row += 1;
        }
        summary.difficulty_membership[row.difficulty as usize - 1] += 1;
        if row.removed {
            summary.removed += 1;
        } else {
            summary.active += 1;
        }
        if row.zone.is_some() {
            summary.recovered_zone_maps += 1;
        } else {
            summary.missing_zone_maps += 1;
        }
        if row.stub_zone.is_some() {
            summary.stub_zone_definitions += 1;
        }
        if row.objective_locale.is_some() {
            summary.objective_locales += 1;
        }
        if row.entrance.is_none() {
            summary.missing_entrances += 1;
        }
    }
    if summary.difficulty_membership != DIFFICULTY_MEMBERSHIP {
        return Err(CatalogError::DifficultyMembershipMismatch {
            expected: DIFFICULTY_MEMBERSHIP,
            found: summary.difficulty_membership,
        });
    }
    Ok(summary)
}

/// Validate the recovered roster.
pub fn validate_recovered() -> Result<CatalogSummary, CatalogError> {
    validate(&DUNGEONS)
}
