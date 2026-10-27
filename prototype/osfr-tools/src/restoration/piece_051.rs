//! Work item 051: enemy catalogue - recovered enemy identities, models and placements.
//!
//! ORIGINAL facts carried by this module (provenance in
//! `notes/Research/Pieces/051 Enemy catalogue.md`, re-verified against the preserved inputs by
//! the private recovery checker `piece_051.py`):
//!
//! * `Resources/Models.txt` (preserved original client table, 3743 rows) declares
//!   [`MODELS_TXT_COLUMNS`]. Exactly 27 of its rows carry a combat actor descriptor:
//!   `Combat NPC` (17), `Combat NPC: Forest Giant`, `Combat NPC: Forest Giant Boss` and
//!   `Halloween Combat NPC` (8). `Combat training prop` and `Halloween Combat Powerup` are
//!   props, so the marker is matched exactly, never by substring.
//! * The original locale table resolves every enemy name id this catalogue uses, including the
//!   names the local restoration spawns (5100200 Hooligan Wolf, 5100344 Mac, 5101961 Gloam
//!   Creature, 441457 Gloamling, 5101505 Gloam Overlord).
//! * `Resources/RaceDefinitions.txt` maps a model's race id to a locale name id, which is how a
//!   row is grouped by original creature race (Robgoblin Warrior 102, Ghost 9, Chugawug 7, ...).
//!
//! COMMUNITY facts (recorded, not original): the npc definition id -> name id/model id binding
//! comes from `lab/references/Sanctuary/src/Resources/Npcs.json` and its wider fork
//! `lab/references/FreeRealms-Legacy/src/Resources/Npcs.json`; the placement convention
//! ([`PEER_DEFINITION_ID_BASE`] + definition id) comes from
//! `Scripts/Zone/FabledRealms.lua`. The two copies agree on all 3419 shared definition ids
//! (model id and name), which is what makes the binding usable as evidence.
//!
//! RECONSTRUCTED, labelled at each row: the definition ids 920001 (Gloam Creature), 920002
//! (Mac), 920100-920102 (Gloamling) and 920110 (Gloam Overlord) are local. Their models and
//! names are original, their definition ids are not: no preserved NPC table contains them.
//! Health, damage, ability/skill tables, disposition per definition and the original patrol
//! routes were never captured and stay explicit gaps ([`MISSING_BINDINGS`]), never invented.
//!
//! NOT CLAIMED: any native spawn, rendering or combat behaviour. This module binds identities
//! and rejects fabricated ones; movement, aggro and melee are owned by items 052-054.

use std::collections::BTreeMap;
use std::fmt;

/// Recovered column order of the preserved original client model table.
pub const MODELS_TXT_COLUMNS: [&str; 17] = [
    "ID",
    "MODEL_FILE_NAME",
    "RACE_ID",
    "SCALE",
    "DESCRIPTION",
    "GENDER",
    "AGE",
    "DESCRIPTOR",
    "MATERIAL_TYPE",
    "WATER_DISPLACEMENT",
    "TELEPORT_EFFECT_ID",
    "CAMERA_DISTANCE",
    "CAMERA_ANGLE",
    "IS_VALID_FOR_PC",
    "NAMEPLATE_OFFSET",
    "CAPSULE_HEIGHT",
    "RADIUS",
];

/// Recovered field order of a community npc definition row.
pub const DEFINITION_FIELDS: [&str; 5] = ["Id", "NameId", "Name", "ModelId", "ModelFileName"];

/// The only original descriptor that marks a combat actor.
pub const COMBAT_DESCRIPTOR: &str = "Combat NPC";
/// Its prefix variant, used by the forest giant and forest giant boss rows.
pub const COMBAT_DESCRIPTOR_PREFIX: &str = "Combat NPC:";
/// The seasonal combat actor descriptor.
pub const HALLOWEEN_COMBAT_DESCRIPTOR: &str = "Halloween Combat NPC";

/// Community placement convention: every preserved spawn guid is this base plus the definition id.
pub const PEER_DEFINITION_ID_BASE: u64 = 100_000_000_000;

/// Where an applied enemy row's definition id comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefinitionBinding {
    /// A preserved community NPC table contains this definition id.
    CommunityDefinition,
    /// Only the local restoration defines this id; the model/name pair is still original.
    LocalOnly,
}

/// Which recovered channel put a row in the catalogue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyEvidence {
    /// The original model row carries a combat actor descriptor.
    OriginalCombatModel,
    /// A preserved community NPC table defines the definition id.
    CommunityDefinition,
    /// A wiki page filed under the enemy category documents the name.
    WikiEnemyPage,
    /// The local CartoonMMO encounter spawns this identity.
    LocalReconstruction,
}

/// A field the preserved inputs never carried. Each gap names the evidence that would close it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingGap {
    /// No original enemy ability/skill table exists in any preserved input.
    EnemySkills,
    /// No original enemy health/damage exists in any preserved input.
    EnemyStats,
    /// The client ships no faction field and the wiki faction row is empty.
    Faction,
    /// The definition id is local; no preserved table binds it.
    DefinitionId,
}

/// Every binding this catalogue still owes, in reporting order.
pub const MISSING_BINDINGS: [BindingGap; 4] = [
    BindingGap::EnemySkills,
    BindingGap::EnemyStats,
    BindingGap::Faction,
    BindingGap::DefinitionId,
];

/// What would close a gap. Kept verbatim in the checker's findings.json.
pub fn binding_gap_evidence(gap: BindingGap) -> &'static str {
    match gap {
        BindingGap::EnemySkills => {
            "no preserved input carries an original enemy ability or skill table; closing this needs \
             the server-side NPC/encounter data source or a capture holding enemy ability cast \
             messages"
        }
        BindingGap::EnemyStats => {
            "no preserved input carries original enemy health or damage; the local encounter \
             values are reconstructed local rules, never historical stats"
        }
        BindingGap::Faction => {
            "the client ships no faction field and the wiki infobox faction row is empty; only a \
             wiki category on a documented group page names a faction"
        }
        BindingGap::DefinitionId => {
            "the definition id is local; only a preserved NPC definition table or a spawn capture \
             can bind this model/name pair to a community definition id"
        }
    }
}

/// One row of the preserved original client model table.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelRow {
    pub id: i32,
    pub file_name: String,
    pub race_id: i32,
    pub description: String,
    pub capsule_height: f32,
    pub radius: f32,
}

/// An enemy identity the local restoration can spawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AppliedEnemy {
    pub definition_id: i32,
    pub name_id: u32,
    pub name: &'static str,
    pub model_id: i32,
    pub model_file: &'static str,
    pub boss: bool,
    pub evidence: EnemyEvidence,
    pub binding: DefinitionBinding,
}

/// Why an enemy row cannot be bound to the preserved original tables.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CatalogueError {
    /// The model table header is not the recovered 17-column schema.
    UnexpectedSchema { line: usize },
    /// A model row does not have the recovered column count or a numeric id.
    MalformedRow { line: usize },
    /// The definition id repeats inside one table.
    DuplicateModelId { model_id: i32 },
    /// The original model table has no row for this model id.
    MissingModelBinding { definition_id: i32, model_id: i32 },
    /// The recorded model file name disagrees with the original table.
    ModelFileMismatch {
        definition_id: i32,
        model_id: i32,
        recorded: String,
        original: String,
    },
    /// The original locale has no text for this name id.
    MissingNameId { definition_id: i32, name_id: u32 },
    /// The recorded name disagrees with the original locale text.
    NameDrift {
        definition_id: i32,
        name_id: u32,
        recorded: String,
        original: String,
    },
}

impl fmt::Display for CatalogueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CatalogueError::UnexpectedSchema { line } => {
                write!(
                    f,
                    "model table header on line {line} is not the recovered schema"
                )
            }
            CatalogueError::MalformedRow { line } => write!(f, "model row {line} is malformed"),
            CatalogueError::DuplicateModelId { model_id } => {
                write!(f, "model id {model_id} appears twice")
            }
            CatalogueError::MissingModelBinding {
                definition_id,
                model_id,
            } => write!(
                f,
                "enemy {definition_id} references model {model_id}, which the original table does not define"
            ),
            CatalogueError::ModelFileMismatch {
                definition_id,
                model_id,
                recorded,
                original,
            } => write!(
                f,
                "enemy {definition_id} records model {model_id} as {recorded:?}, the original table says {original:?}"
            ),
            CatalogueError::MissingNameId {
                definition_id,
                name_id,
            } => write!(
                f,
                "enemy {definition_id} references name id {name_id}, which the original locale does not resolve"
            ),
            CatalogueError::NameDrift {
                definition_id,
                name_id,
                recorded,
                original,
            } => write!(
                f,
                "enemy {definition_id} records name id {name_id} as {recorded:?}, the original locale says {original:?}"
            ),
        }
    }
}

impl std::error::Error for CatalogueError {}

/// True only for the original combat actor descriptors, never for props or powerups.
pub fn is_enemy_model(description: &str) -> bool {
    description == COMBAT_DESCRIPTOR
        || description == HALLOWEEN_COMBAT_DESCRIPTOR
        || description.starts_with(COMBAT_DESCRIPTOR_PREFIX)
}

/// Parse the preserved original client model table.
pub fn parse_models_txt(text: &str) -> Result<BTreeMap<i32, ModelRow>, CatalogueError> {
    let mut rows = BTreeMap::new();
    let mut seen_header = false;
    for (index, raw) in text.lines().enumerate() {
        let line_number = index + 1;
        let line = raw.trim_end_matches('\r');
        if line.is_empty() {
            continue;
        }
        if let Some(header) = line.strip_prefix('#') {
            let columns: Vec<&str> = header.trim_end_matches('^').split('^').collect();
            if columns != MODELS_TXT_COLUMNS {
                return Err(CatalogueError::UnexpectedSchema { line: line_number });
            }
            seen_header = true;
            continue;
        }
        let fields: Vec<&str> = line.split('^').collect();
        if fields.len() < MODELS_TXT_COLUMNS.len() {
            return Err(CatalogueError::MalformedRow { line: line_number });
        }
        let Ok(id) = fields[0].parse::<i32>() else {
            return Err(CatalogueError::MalformedRow { line: line_number });
        };
        let race_id = fields[2]
            .parse::<i32>()
            .map_err(|_| CatalogueError::MalformedRow { line: line_number })?;
        let capsule_height = fields[15]
            .parse::<f32>()
            .map_err(|_| CatalogueError::MalformedRow { line: line_number })?;
        let radius = fields[16]
            .parse::<f32>()
            .map_err(|_| CatalogueError::MalformedRow { line: line_number })?;
        let row = ModelRow {
            id,
            file_name: fields[1].to_string(),
            race_id,
            description: fields[4].to_string(),
            capsule_height,
            radius,
        };
        if rows.insert(id, row).is_some() {
            return Err(CatalogueError::DuplicateModelId { model_id: id });
        }
    }
    if !seen_header {
        return Err(CatalogueError::UnexpectedSchema { line: 0 });
    }
    Ok(rows)
}

/// Which channel defines a definition id.
pub fn definition_binding(definition_id: i32, community_ids: &[i32]) -> DefinitionBinding {
    if community_ids.contains(&definition_id) {
        DefinitionBinding::CommunityDefinition
    } else {
        DefinitionBinding::LocalOnly
    }
}

/// Bind one enemy identity to the preserved original model table and locale.
///
/// A trailing-space-only difference is accepted on either side: the original table keeps
/// `cray_king_lowtide.adr ` for model 4446 and the preserved locale keeps a trailing space on
/// four names, so the recovered text is compared trimmed and never rewritten.
pub fn validate_actor(
    actor: &AppliedEnemy,
    models: &BTreeMap<i32, ModelRow>,
    names: &BTreeMap<u32, String>,
) -> Result<(), CatalogueError> {
    let model = models
        .get(&actor.model_id)
        .ok_or(CatalogueError::MissingModelBinding {
            definition_id: actor.definition_id,
            model_id: actor.model_id,
        })?;
    if model.file_name.trim() != actor.model_file.trim() {
        return Err(CatalogueError::ModelFileMismatch {
            definition_id: actor.definition_id,
            model_id: actor.model_id,
            recorded: actor.model_file.to_string(),
            original: model.file_name.clone(),
        });
    }
    let text = names
        .get(&actor.name_id)
        .ok_or(CatalogueError::MissingNameId {
            definition_id: actor.definition_id,
            name_id: actor.name_id,
        })?;
    if text.trim() != actor.name.trim() {
        return Err(CatalogueError::NameDrift {
            definition_id: actor.definition_id,
            name_id: actor.name_id,
            recorded: actor.name.to_string(),
            original: text.clone(),
        });
    }
    Ok(())
}

const fn row(
    definition_id: i32,
    name_id: u32,
    name: &'static str,
    model_id: i32,
    model_file: &'static str,
    boss: bool,
    evidence: EnemyEvidence,
    binding: DefinitionBinding,
) -> AppliedEnemy {
    AppliedEnemy {
        definition_id,
        name_id,
        name,
        model_id,
        model_file,
        boss,
        evidence,
        binding,
    }
}

/// The enemy identities the local CartoonMMO opening actually spawns, in encounter order.
///
/// Hooligan Wolf 3156 is a preserved community definition; every Gloam actor is local-only and
/// says so. The three Gloamling rows are three spawn instances of one recovered identity.
pub const APPLIED_ENEMIES: [AppliedEnemy; 7] = [
    row(
        3156,
        5100200,
        "Hooligan Wolf",
        3284,
        "wolf_generic.adr",
        false,
        EnemyEvidence::CommunityDefinition,
        DefinitionBinding::CommunityDefinition,
    ),
    row(
        920001,
        5101961,
        "Gloam Creature",
        3933,
        "gloam_m_construct.adr",
        false,
        EnemyEvidence::LocalReconstruction,
        DefinitionBinding::LocalOnly,
    ),
    row(
        920002,
        5100344,
        "Mac",
        3897,
        "human_m_hooligan_boss_02.agr",
        true,
        EnemyEvidence::LocalReconstruction,
        DefinitionBinding::LocalOnly,
    ),
    row(
        920100,
        441457,
        "Gloamling",
        4201,
        "gloamling_m_default.adr",
        false,
        EnemyEvidence::LocalReconstruction,
        DefinitionBinding::LocalOnly,
    ),
    row(
        920101,
        441457,
        "Gloamling",
        4201,
        "gloamling_m_default.adr",
        false,
        EnemyEvidence::LocalReconstruction,
        DefinitionBinding::LocalOnly,
    ),
    row(
        920102,
        441457,
        "Gloamling",
        4201,
        "gloamling_m_default.adr",
        false,
        EnemyEvidence::LocalReconstruction,
        DefinitionBinding::LocalOnly,
    ),
    row(
        920110,
        5101505,
        "Gloam Overlord",
        3956,
        "gloam_m_boss.adr",
        true,
        EnemyEvidence::LocalReconstruction,
        DefinitionBinding::LocalOnly,
    ),
];
