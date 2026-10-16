//! Piece 051 (enemy catalogue) tests: the recovered original enemy identity bindings.
//!
//! Evidence boundary, read before the assertions:
//!
//! ORIGINAL (first-party client files; sha256s re-verified by `scripts/pieces/piece_051.py`
//! and recorded in `notes/Research/Pieces/051 Enemy catalogue.md`):
//! * `Resources/Models.txt` declares 17 columns in this order: Id, ModelFileName, RaceId,
//!   Scale, Description, Gender, Age, Descriptor, MaterialType, WaterDisplacement,
//!   TeleportEffectId, CameraDistance, CameraAngle, IsValidForPc, NameplateOffset,
//!   CapsuleHeight, Radius. 3743 rows. 27 rows carry the combat marker
//!   (`Combat NPC`, `Combat NPC: Forest Giant`, `Combat NPC: Forest Giant Boss`,
//!   `Halloween Combat NPC`); `Combat training prop` and `Halloween Combat Powerup` are not
//!   combat actors.
//! * `Resources/RaceDefinitions.txt` maps race id -> original locale name id.
//! * The original locale table resolves the enemy name ids used by the local encounter
//!   (5100200 Hooligan Wolf, 5100344 Mac, 5101961 Gloam Creature, 441457 Gloamling,
//!   5101505 Gloam Overlord).
//!
//! COMMUNITY (recorded, not original): `lab/references/Sanctuary/src/Resources/Npcs.json`
//! binds npc definition id -> name id/model id, and `Scripts/Zone/FabledRealms.lua` places one
//! spawn per definition id. Both are cross-checked against the original files here.
//!
//! RECONSTRUCTED (labelled, not historical): the five definition ids the local restoration
//! applies for Gloam/Mac/Gloamling/Gloam Overlord are local; only Hooligan Wolf 3156 has a
//! community definition id. Health/damage stay out of this catalogue because no preserved
//! input carries an original enemy stat or ability table.
//!
//! The module is not registered in `src/lib.rs` (lead-owned). Including the exact source file
//! keeps this target compiling against the real artifact. Integration step for the lead:
//! register the module and drop this `#[path]` include.
#[path = "../src/restoration/piece_051.rs"]
#[allow(dead_code)]
mod piece_051;

use piece_051::{
    APPLIED_ENEMIES, AppliedEnemy, BindingGap, COMBAT_DESCRIPTOR, CatalogueError,
    DEFINITION_FIELDS, DefinitionBinding, EnemyEvidence, HALLOWEEN_COMBAT_DESCRIPTOR,
    MISSING_BINDINGS, MODELS_TXT_COLUMNS, PEER_DEFINITION_ID_BASE, binding_gap_evidence,
    definition_binding, is_enemy_model, parse_models_txt, validate_actor,
};
use std::collections::BTreeMap;

/// The exact original header line and a slice of unchanged original rows.
const MODELS_TXT: &str = "\
#ID^MODEL_FILE_NAME^RACE_ID^SCALE^DESCRIPTION^GENDER^AGE^DESCRIPTOR^MATERIAL_TYPE^WATER_DISPLACEMENT^TELEPORT_EFFECT_ID^CAMERA_DISTANCE^CAMERA_ANGLE^IS_VALID_FOR_PC^NAMEPLATE_OFFSET^CAPSULE_HEIGHT^RADIUS^
4^robgoblin_m_basic.adr^102^0.000000^Combat NPC^1^0^^3^0.000000^0^0.000000^0.000000^0^0.000000^1.900000^0.000000^
5^banshee_01.adr^9^0.000000^Combat NPC^2^0^^1^0.000000^0^0.000000^0.000000^0^0.000000^1.900000^0.000000^
62^bear_vinegolem.adr^105^0.000000^Combat NPC^0^0^^2^0.000000^0^0.000000^0.000000^0^0.000000^1.900000^0.000000^
88^maneatingplant.adr^105^0.000000^Combat NPC^0^0^^2^0.000000^0^0.000000^0.000000^0^0.000000^1.900000^0.000000^
314^troll_hogosh_m.adr^20^0.000000^Combat NPC^1^0^^1^0.000000^0^0.000000^0.000000^0^0.000000^1.900000^0.000000^
4011^dwarf_m_bikerthug_skullz.adr^0^0.000000^^0^0^^1^0.000000^0^0.000000^0.000000^0^0.000000^1.900000^0.000000^
3284^wolf_generic.adr^0^0.000000^^0^0^^1^0.000000^0^0.000000^0.000000^0^0.000000^1.900000^0.000000^
3933^gloam_m_construct.adr^0^0.000000^^0^0^^1^0.000000^0^0.000000^0.000000^0^0.000000^1.900000^0.000000^
3897^human_m_hooligan_boss_02.agr^0^0.000000^^0^0^^1^0.000000^0^0.000000^0.000000^0^0.000000^1.900000^0.000000^
4201^gloamling_m_default.adr^0^0.000000^^0^0^^1^0.000000^0^0.000000^0.000000^0^0.000000^1.900000^0.000000^
3956^gloam_m_boss.adr^0^0.000000^^0^0^^1^0.000000^0^0.000000^0.000000^0^0.000000^1.900000^0.000000^
353^fairy_darkthorne_f.adr^2^0.000000^Darkthorne^2^0^^1^0.000000^0^0.000000^0.000000^0^0.000000^1.900000^0.000000^
";

/// The original locale values for the names the local restoration spawns.
fn recovered_names() -> BTreeMap<u32, String> {
    [
        (5100200u32, "Hooligan Wolf"),
        (5100344, "Mac"),
        (5101961, "Gloam Creature"),
        (441457, "Gloamling"),
        (5101505, "Gloam Overlord"),
        (419307, "Crash Davis "),
    ]
    .into_iter()
    .map(|(id, text)| (id, text.to_string()))
    .collect()
}

fn models() -> BTreeMap<i32, piece_051::ModelRow> {
    parse_models_txt(MODELS_TXT).expect("the recovered original rows parse")
}

#[test]
fn models_schema_matches_the_recovered_original_header() {
    assert_eq!(MODELS_TXT_COLUMNS.len(), 17);
    assert_eq!(
        MODELS_TXT_COLUMNS,
        [
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
        ]
    );
    assert_eq!(DEFINITION_FIELDS.len(), 5);
}

#[test]
fn models_parser_binds_real_rows_and_rejects_malformed_input() {
    let parsed = models();
    assert_eq!(parsed.len(), 12);
    let wolf = &parsed[&3284];
    assert_eq!(wolf.file_name, "wolf_generic.adr");
    assert_eq!(wolf.race_id, 0);
    assert_eq!(wolf.description, "");
    assert_eq!(parsed[&4].race_id, 102);
    assert_eq!(parsed[&4].description, COMBAT_DESCRIPTOR);
    assert!((parsed[&4].capsule_height - 1.9).abs() < f32::EPSILON);
    assert!((parsed[&4].radius - 0.0).abs() < f32::EPSILON);

    let short = "#ID^MODEL_FILE_NAME^RACE_ID^SCALE^DESCRIPTION^GENDER^AGE^DESCRIPTOR^MATERIAL_TYPE^WATER_DISPLACEMENT^TELEPORT_EFFECT_ID^CAMERA_DISTANCE^CAMERA_ANGLE^IS_VALID_FOR_PC^NAMEPLATE_OFFSET^CAPSULE_HEIGHT^RADIUS^\n4^robgoblin_m_basic.adr^102^0.000000\n";
    assert!(matches!(
        parse_models_txt(short),
        Err(CatalogueError::MalformedRow { line: 2 })
    ));
}

#[test]
fn combat_descriptor_is_an_exact_original_enemy_marker() {
    assert!(is_enemy_model("Combat NPC"));
    assert!(is_enemy_model(HALLOWEEN_COMBAT_DESCRIPTOR));
    assert!(is_enemy_model("Combat NPC: Forest Giant"));
    assert!(is_enemy_model("Combat NPC: Forest Giant Boss"));
    assert!(!is_enemy_model(""));
    assert!(!is_enemy_model("Combat training prop"));
    assert!(!is_enemy_model("Halloween Combat Powerup"));
    assert!(!is_enemy_model("Not a Combat NPC"));

    let parsed = models();
    let flagged: Vec<i32> = parsed
        .values()
        .filter(|row| is_enemy_model(&row.description))
        .map(|row| row.id)
        .collect();
    assert_eq!(flagged, vec![4, 5, 62, 88, 314]);
}

#[test]
fn applied_enemies_match_original_models_and_locale() {
    let models = models();
    let names = recovered_names();
    assert_eq!(APPLIED_ENEMIES.len(), 7);
    for actor in APPLIED_ENEMIES {
        validate_actor(&actor, &models, &names).unwrap_or_else(|error| {
            panic!(
                "applied enemy {} failed validation: {error:?}",
                actor.definition_id
            )
        });
    }

    let wolf = APPLIED_ENEMIES
        .iter()
        .find(|actor| actor.definition_id == 3156)
        .expect("the recovered Hooligan Wolf definition is catalogued");
    assert_eq!(wolf.name, "Hooligan Wolf");
    assert_eq!(wolf.model_id, 3284);
    assert_eq!(wolf.evidence, EnemyEvidence::CommunityDefinition);
}

#[test]
fn unbound_definition_ids_are_reported_as_local_only() {
    let community_ids = [3156i32, 3157, 3158, 3159, 3160];
    assert_eq!(
        definition_binding(3156, &community_ids),
        DefinitionBinding::CommunityDefinition
    );
    assert_eq!(
        definition_binding(920001, &community_ids),
        DefinitionBinding::LocalOnly
    );

    let local_only: Vec<i32> = APPLIED_ENEMIES
        .iter()
        .filter(|actor| actor.binding == DefinitionBinding::LocalOnly)
        .map(|actor| actor.definition_id)
        .collect();
    // 920100-920102 are three spawn instances of the recovered Gloamling identity.
    assert_eq!(
        local_only,
        vec![920001, 920002, 920100, 920101, 920102, 920110]
    );
    assert!(PEER_DEFINITION_ID_BASE > 0);
}

#[test]
fn actor_validation_rejects_model_and_name_drift() {
    let models = models();
    let names = recovered_names();
    let wolf = APPLIED_ENEMIES[0];

    let mut wrong_file = wolf;
    wrong_file.model_file = "wolf_generic.dme";
    assert!(matches!(
        validate_actor(&wrong_file, &models, &names),
        Err(CatalogueError::ModelFileMismatch { model_id: 3284, .. })
    ));

    let mut wrong_name = wolf;
    wrong_name.name = "Hooligan Hound";
    assert!(matches!(
        validate_actor(&wrong_name, &models, &names),
        Err(CatalogueError::NameDrift {
            name_id: 5100200,
            ..
        })
    ));

    let mut unknown_name = wolf;
    unknown_name.name_id = 1;
    assert!(matches!(
        validate_actor(&unknown_name, &models, &names),
        Err(CatalogueError::MissingNameId { name_id: 1, .. })
    ));

    let mut unknown_model = wolf;
    unknown_model.model_id = 999_999;
    assert!(matches!(
        validate_actor(&unknown_model, &models, &names),
        Err(CatalogueError::MissingModelBinding {
            model_id: 999_999,
            ..
        })
    ));

    // A missing model row must not be silently accepted for an actor that carries no identity.
    let mut unbound = wolf;
    unbound.model_id = 0;
    assert!(matches!(
        validate_actor(&unbound, &models, &names),
        Err(CatalogueError::MissingModelBinding { model_id: 0, .. })
    ));
}

#[test]
fn name_lookup_tolerates_only_the_recovered_trailing_space() {
    let models = models();
    let names = recovered_names();
    let mut actor = AppliedEnemy {
        definition_id: 3043,
        name_id: 419307,
        name: "Crash Davis",
        model_id: 3284,
        model_file: "wolf_generic.adr",
        boss: false,
        evidence: EnemyEvidence::CommunityDefinition,
        binding: DefinitionBinding::CommunityDefinition,
    };
    // The preserved community row trims a trailing space the original locale keeps, so the
    // catalogue accepts the trimmed form and never rewrites either side.
    validate_actor(&actor, &models, &names).expect("trailing-space-only drift is tolerated");

    actor.name = "Crash Daviz";
    assert!(matches!(
        validate_actor(&actor, &models, &names),
        Err(CatalogueError::NameDrift {
            name_id: 419307,
            ..
        })
    ));
}

#[test]
fn missing_bindings_are_explicit_about_what_was_never_recovered() {
    assert_eq!(
        MISSING_BINDINGS,
        [
            BindingGap::EnemySkills,
            BindingGap::EnemyStats,
            BindingGap::Faction,
            BindingGap::DefinitionId,
        ]
    );
    let skills = binding_gap_evidence(BindingGap::EnemySkills);
    assert!(skills.contains("ability") || skills.contains("skill"));
    let stats = binding_gap_evidence(BindingGap::EnemyStats);
    assert!(stats.contains("health") || stats.contains("stat"));
    let faction = binding_gap_evidence(BindingGap::Faction);
    assert!(faction.contains("faction"));
    let definition = binding_gap_evidence(BindingGap::DefinitionId);
    assert!(definition.contains("definition"));
    for gap in MISSING_BINDINGS {
        assert!(!binding_gap_evidence(gap).is_empty());
    }
}
