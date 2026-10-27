//! Work item 062 - one original boss's phases and abilities: the recovered
//! `Necronomicus Unleashed` encounter (original encounter 432, "Necronomicus' Lair").
//!
//! ORIGINAL facts implemented here (provenance in
//! `notes/Research/Pieces/062 Boss behaviors.md` and re-read from the preserved lab evidence by
//! the private recovery checker `piece_062.py`):
//!
//! * Identity: the community NPC row 3283 carries name id 440729 `Necronomicus` and model
//!   `necronomicus_m.adr`; the client's own `Resources/Models.txt` maps model id 4220 to the same
//!   file. Locale 5100662 names the encounter `Encounter 432 - Necronomicus' Lair` and locale
//!   442596 names the battle `Necronomicus Unleashed`.
//! * Phases: the `Necronomicus` page says the boss fights as a skeletal form, collapses into
//!   skeleton pieces with a zombie portal that spawns necrowart zombies, and after the portal is
//!   destroyed re-forms as a "powered skeletal creature" that smacks the ground. The original
//!   locale independently carries eight `Destroy Zombie Portal N` objectives (5100669-5100683),
//!   three `Necronomicus Buff Package 1/2/3` rows (5100686/5100688/5100690) and the dev row
//!   `[dev] Necronomicus Phase 3 Basic Attack` (5100995).
//! * Abilities: locale names and descriptions for Ground Pound (5100996/5100997), Hungering
//!   Darkness (5100998/5100999), Heart of Darkness (5101000/5101001), Necrotic Gloamfire
//!   (5100693/5100694), Necrotic Gloamfire Splash (5100695/5100696), the basic attack rows
//!   (5100994/5100995/5101009) and `Necronomicus Cave-in` (5101015/5101016, "triggered from N's
//!   Ground Pound ability").
//! * Animation binding and timing: the original actor definition `necronomicus_m.adr` binds its
//!   attack slots to clips (`com_cast_01` -> `necronomicus_com_ground_pound.gr2`, `com_cast_02`
//!   -> `necronomicus_com_360_attack.gr2`, `com_cast_03` -> `necronomicus_com_chest_laser.gr2`,
//!   `com_h2h_attack_01..04` -> clap/flick/kick/headbutt) and stores authored `ap_contact` /
//!   `ap_interrupt` phase times per slot. The same file binds `necronomicus_ground_shake` to
//!   `com_cast_01`, `head_particles` to `com_cast_02` and `magic_beam_shot` to `com_cast_03`.
//!
//! RECONSTRUCTED, labelled at each call site: nothing in this module drives a live encounter.
//! `advance` re-derives the documented phase order as a pure lookup; the original server's own
//! scheduler was not recovered.
//!
//! BOUNDARIES / GAPS: server cooldowns and cast scheduling, phase health thresholds, portal
//! entity ids, the bark-to-recording pairing and any decoded Granny clip duration were not
//! recovered. `SERVER_ABILITY_TIMING_RECOVERED`, `PHASE_HEALTH_THRESHOLDS_RECOVERED`,
//! `PORTAL_ENTITY_IDS_RECOVERED`, `CUE_TO_RECORDING_RECOVERED` and `GR2_CLIP_DURATION_DECODED`
//! stay `false` and the checker fails if any of them is flipped.

/// Community NPC row id for Necronomicus (`Sanctuary/src/Resources/Npcs.json`).
pub const NECRONOMICUS_NPC_ID: i32 = 3283;
/// Locale id of the boss name, `Necronomicus`.
pub const NECRONOMICUS_NAME_ID: u32 = 440729;
/// Client model id (`Resources/Models.txt` row `4220`).
pub const NECRONOMICUS_MODEL_ID: i32 = 4220;
/// Actor definition the model row resolves to.
pub const NECRONOMICUS_MODEL_FILE: &str = "necronomicus_m.adr";

/// Original encounter id from locale 5100662 `Encounter 432 - Necronomicus' Lair`.
pub const ENCOUNTER_ID: u32 = 432;
/// Locale id of the encounter name above.
pub const ENCOUNTER_NAME_ID: u32 = 5100662;
/// Locale id of the battle name `Necronomicus Unleashed` (442596, also the quest line name).
pub const BATTLE_NAME_ID: u32 = 442596;
/// The battle name the locale resolves that id to.
pub const BATTLE_NAME: &str = "Necronomicus Unleashed";
/// `Battles` page difficulty rating for `Necronomicus Unleashed` (five is hardest).
pub const BATTLE_DIFFICULTY: u8 = 5;
/// `Battles` page entry requirement: difficulty 5 needs a combat job of level 13 or higher.
pub const BATTLE_MIN_COMBAT_LEVEL: u8 = 13;
/// Locale id of the Awakened/Overworld quest objective `Defeat the Awakened version ...` (442600).
pub const AWAKENED_OBJECTIVE_ID: u32 = 442600;
/// Recommended group-level the Awakened warning text states (locale 442598).
pub const AWAKENED_RECOMMENDED_LEVEL: u8 = 20;
/// Merry Vale terrain prefix the lair assets use (`mv_necronomicus*.gzne` / tiles).
pub const LAIR_ZONE_PREFIX: &str = "mv_necronomicus";
/// Animation slots the original actor definition binds.
pub const ADR_ANIMATION_SLOT_COUNT: usize = 31;

/// How a recovered row was obtained.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Evidence {
    /// Original client locale text or table preserved in the lab.
    OriginalLocale,
    /// Original actor definition (`necronomicus_m.adr`) data.
    OriginalAdr,
    /// Original particle/effect definition.
    OriginalEffect,
    /// Community mirror of an original table (`Npcs.json`, `Models.txt`).
    CommunityTable,
    /// Read-only wiki documentation.
    Wiki,
}

/// The documented boss forms, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BossPhase {
    /// The skeletal necromancer the fight starts with.
    Skeletal,
    /// The collapsed skeleton pieces and the zombie portals that spawn necrowart zombies.
    Portals,
    /// The re-formed powered skeletal creature.
    Empowered,
    /// Final defeat.
    Defeated,
}

/// The events the wiki documents at each phase boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BossEvent {
    /// The skeletal form is defeated and collapses.
    SkeletalDefeated,
    /// The zombie portals are destroyed.
    PortalsDestroyed,
    /// The powered form is defeated for good.
    EmpoweredDefeated,
}

/// Why a phase transition was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BossRefusal {
    /// The event does not belong to this phase (the documented order is strict).
    EventNotValidInPhase { phase: BossPhase },
    /// The encounter is already over.
    AlreadyDefeated,
}

/// One documented phase boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhaseTransition {
    pub from: BossPhase,
    pub event: BossEvent,
    pub to: BossPhase,
    pub evidence: Evidence,
}

/// The three phase boundaries the `Necronomicus` page documents, in order.
pub const PHASE_TRANSITIONS: &[PhaseTransition] = &[
    PhaseTransition {
        from: BossPhase::Skeletal,
        event: BossEvent::SkeletalDefeated,
        to: BossPhase::Portals,
        evidence: Evidence::Wiki,
    },
    PhaseTransition {
        from: BossPhase::Portals,
        event: BossEvent::PortalsDestroyed,
        to: BossPhase::Empowered,
        evidence: Evidence::Wiki,
    },
    PhaseTransition {
        from: BossPhase::Empowered,
        event: BossEvent::EmpoweredDefeated,
        to: BossPhase::Defeated,
        evidence: Evidence::Wiki,
    },
];

/// Authored attack phase timing from the actor definition's `ap_contact` / `ap_interrupt` rows.
///
/// The original file stores two big-endian `f32` values per attack slot. The unit is not stored
/// in the file; seconds inside the clip is the working interpretation and is not claimed as
/// proved. The server-side cast schedule these values fed was not recovered.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AttackTiming {
    pub slot: &'static str,
    pub clip: &'static str,
    pub contact_seconds: f32,
    pub interrupt_seconds: Option<f32>,
}

/// The seven attack slots the actor definition gives phase times for, in file order.
pub const ATTACK_TIMINGS: &[AttackTiming] = &[
    AttackTiming {
        slot: "com_h2h_attack_01",
        clip: "necronomicus_com_h2h_attack_clap.gr2",
        contact_seconds: 0.703_599_99,
        interrupt_seconds: Some(0.834_1),
    },
    AttackTiming {
        slot: "com_h2h_attack_02",
        clip: "necronomicus_com_h2h_attack_flick.gr2",
        contact_seconds: 0.782,
        interrupt_seconds: None,
    },
    AttackTiming {
        slot: "com_h2h_attack_03",
        clip: "necronomicus_com_h2h_attack_kick.gr2",
        contact_seconds: 1.025_3,
        interrupt_seconds: Some(1.272_4),
    },
    AttackTiming {
        slot: "com_h2h_attack_04",
        clip: "necronomicus_com_h2h_attack_headbutt.gr2",
        contact_seconds: 0.599_8,
        interrupt_seconds: Some(0.758_3),
    },
    AttackTiming {
        slot: "com_cast_01",
        clip: "necronomicus_com_ground_pound.gr2",
        contact_seconds: 1.195_5,
        interrupt_seconds: Some(1.459_3),
    },
    AttackTiming {
        slot: "com_cast_02",
        clip: "necronomicus_com_360_attack.gr2",
        contact_seconds: 1.597_8,
        interrupt_seconds: Some(3.164_2),
    },
    AttackTiming {
        slot: "com_cast_03",
        clip: "necronomicus_com_chest_laser.gr2",
        contact_seconds: 3.095_3,
        interrupt_seconds: Some(4.527_7),
    },
];

/// A named ability the original locale text documents.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ability {
    pub name: &'static str,
    pub name_id: u32,
    pub description_id: Option<u32>,
    /// Animation slot, when the actor definition or the original effect binding proves one.
    pub slot: Option<&'static str>,
    pub clip: Option<&'static str>,
    /// Effect definition the actor binds to that slot.
    pub effect: Option<&'static str>,
    pub evidence: Evidence,
}

/// Every battle-side ability row recovered from the original locale and actor definition.
///
/// The signature blast and its splash carry original names and descriptions but no recovered
/// animation slot, so their slot stays `None` instead of being guessed.
pub const ABILITIES: &[Ability] = &[
    Ability {
        name: "Necronomicus Basic Attack",
        name_id: 5100994,
        description_id: None,
        slot: None,
        clip: None,
        effect: None,
        evidence: Evidence::OriginalLocale,
    },
    Ability {
        name: "Ground Pound",
        name_id: 5100996,
        description_id: Some(5100997),
        slot: Some("com_cast_01"),
        clip: Some("necronomicus_com_ground_pound.gr2"),
        effect: Some("ground_pound_effect"),
        evidence: Evidence::OriginalAdr,
    },
    Ability {
        name: "Hungering Darkness",
        name_id: 5100998,
        description_id: Some(5100999),
        slot: Some("com_cast_02"),
        clip: Some("necronomicus_com_360_attack.gr2"),
        effect: Some("head_particles"),
        evidence: Evidence::OriginalAdr,
    },
    Ability {
        name: "Heart of Darkness",
        name_id: 5101000,
        description_id: Some(5101001),
        slot: Some("com_cast_03"),
        clip: Some("necronomicus_com_chest_laser.gr2"),
        effect: Some("magic_beam_shot"),
        evidence: Evidence::OriginalAdr,
    },
    Ability {
        name: "Necrotic Gloamfire",
        name_id: 5100693,
        description_id: Some(5100694),
        slot: None,
        clip: None,
        effect: None,
        evidence: Evidence::OriginalLocale,
    },
    Ability {
        name: "Necrotic Gloamfire Splash",
        name_id: 5100695,
        description_id: Some(5100696),
        slot: None,
        clip: None,
        effect: None,
        evidence: Evidence::OriginalLocale,
    },
    Ability {
        name: "Necronomicus Cave-in",
        name_id: 5101015,
        description_id: Some(5101016),
        slot: None,
        clip: None,
        effect: None,
        evidence: Evidence::OriginalLocale,
    },
    Ability {
        name: "[dev] Necronomicus Phase 3 Basic Attack",
        name_id: 5100995,
        description_id: None,
        slot: None,
        clip: None,
        effect: None,
        evidence: Evidence::OriginalLocale,
    },
];

/// Locale text ids the original data marks as the Overworld variant of the battle abilities.
pub const OVERWORLD_ABILITY_TEXT_IDS: &[u32] = &[5101009, 5101011, 5101013, 5101015];

/// One `slot -> (effect, instance)` row from the actor definition's effect binding table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotEffectBinding {
    pub slot: &'static str,
    /// Effect definition the actor names for the slot.
    pub effect: &'static str,
    /// Emitter/instance name inside that effect, which is the name the slot plays.
    pub instance: &'static str,
}

/// The three cast-slot effect bindings the actor definition carries.
pub const SLOT_EFFECT_BINDINGS: &[SlotEffectBinding] = &[
    SlotEffectBinding {
        slot: "com_cast_01",
        effect: "ground_pound",
        instance: "ground_pound_effect",
    },
    SlotEffectBinding {
        slot: "com_cast_02",
        effect: "Head_trail",
        instance: "head_particles",
    },
    SlotEffectBinding {
        slot: "com_cast_03",
        effect: "magic_beam",
        instance: "magic_beam_shot",
    },
];

/// The actor definition binds this ground-shake effect and cue to the Ground Pound cast slot.
pub const GROUND_SHAKE_SLOT: &str = "com_cast_01";
/// Effect definition name of the ground shake.
pub const GROUND_SHAKE_EFFECT: &str = "necronomicus_ground_shake";
/// Cue name the same record carries.
pub const GROUND_SHAKE_CUE: &str = "ground_shake";
/// The two raw `f32` values that record carries; their meaning was not recovered.
pub const GROUND_SHAKE_RAW_VALUES: [f32; 2] = [1.176_6, 1.522_9];

/// The eight `Destroy Zombie Portal N` objectives, portals 1..8.
pub const PORTAL_OBJECTIVES: &[(u8, u32)] = &[
    (1, 5100669),
    (2, 5100671),
    (3, 5100673),
    (4, 5100675),
    (5, 5100677),
    (6, 5100679),
    (7, 5100681),
    (8, 5100683),
];

/// Locale 5100663 `Defeat the necromancer!`, carried next to the encounter name.
pub const DEFEAT_NECROMANCER_OBJECTIVE_ID: u32 = 5100663;
/// Locale 5100664 `Defeat Necronomicus!`, the final objective.
pub const DEFEAT_NECROMONICUS_OBJECTIVE_ID: u32 = 5100664;
/// Locale 5100666 `Destroy the Zombie Portals`, the portal wave objective.
pub const DESTROY_PORTALS_OBJECTIVE_ID: u32 = 5100666;

/// One dialogue cue name the original actor definition declares.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DialogueCue {
    /// Cue name exactly as the actor definition stores it.
    pub cue: &'static str,
    /// Locale id of the spoken line.
    pub text_id: u32,
}

/// The three Necronomicus dialogue cues from the actor definition.
pub const DIALOGUE_CUES: &[DialogueCue] = &[
    DialogueCue {
        cue: "DIA_Necronomicus_Friends",
        text_id: 5100650,
    },
    DialogueCue {
        cue: "DIA_Necronomicus_Not_So_Fast",
        text_id: 5100652,
    },
    DialogueCue {
        cue: "DIA_Necronomicus_Power_Eternal",
        text_id: 5100653,
    },
];

/// One measured original bark recording.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarkRecording {
    pub asset: &'static str,
    /// Duration in seconds, measured from the preserved pack payload's MPEG frames.
    pub seconds: f32,
}

/// The three `DX_NECRONOMICUS_*` recordings shipped in the supplied packs.
pub const BARK_RECORDINGS: &[BarkRecording] = &[
    BarkRecording {
        asset: "DX_NECRONOMICUS_BACK.mp3",
        seconds: 1.392,
    },
    BarkRecording {
        asset: "DX_NECRONOMICUS_NOT_SO_FAST.mp3",
        seconds: 2.136,
    },
    BarkRecording {
        asset: "DX_NECRONOMICUS_POWER.mp3",
        seconds: 3.552,
    },
];

/// Animation slots the actor definition binds to clips that have no supplied pack row.
pub const MISSING_ANIMATION_CLIPS: &[(&str, &str)] = &[
    (
        "amb_loop_01_fidget_01",
        "necronomicus_spawn_stand_loop_02.gr2",
    ),
    (
        "amb_loop_01_fidget_02",
        "necronomicus_spawn_stand_loop_03.gr2",
    ),
];

/// Original spawn smoke effect definition (`necronomicus_pixie_gloam_smoke.xml`).
pub const SPAWN_SMOKE_ASSET: &str = "necronomicus_pixie_gloam_smoke.xml";
/// Its authored end time in seconds.
pub const SPAWN_SMOKE_END_SECONDS: f32 = 3.0;
/// Original chest-beam effect definition (`necronomicus_chest_beam.xml`).
pub const CHEST_BEAM_ASSET: &str = "necronomicus_chest_beam.xml";
/// Its authored end time in seconds.
pub const CHEST_BEAM_END_SECONDS: f32 = 3.0;
/// The chest-beam hit flash instance start time in seconds.
pub const CHEST_BEAM_HIT_FLASH_SECONDS: f32 = 0.5;

/// No server-side ability cooldown or cast schedule was recovered.
pub const SERVER_ABILITY_TIMING_RECOVERED: bool = false;
/// No phase health threshold was recovered.
pub const PHASE_HEALTH_THRESHOLDS_RECOVERED: bool = false;
/// No portal actor/entity id was recovered; only the eight objective strings are original.
pub const PORTAL_ENTITY_IDS_RECOVERED: bool = false;
/// The bark cue names and the recordings were both recovered, but not which cue plays which file.
pub const CUE_TO_RECORDING_RECOVERED: bool = false;
/// The clips were not decoded; their authored playback scalars are recorded, not durations proved.
pub const GR2_CLIP_DURATION_DECODED: bool = false;

/// Structural problems in the recovered tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BossError {
    DuplicateAbility(u32),
    EmptyAbilityName(u32),
    DuplicateAttackSlot(&'static str),
    InvalidAttackTiming(&'static str),
    MissingAttackTiming(&'static str),
    PortalObjectivesOutOfOrder,
    DuplicatePhaseTransition,
    InvalidBarkDuration(&'static str),
}

/// The documented phase the event moves the fight to, or a refusal.
///
/// The lookup is derived from [`PHASE_TRANSITIONS`]; it does not schedule abilities, spawn
/// portals, apply damage or grant anything.
pub fn advance(phase: BossPhase, event: BossEvent) -> Result<BossPhase, BossRefusal> {
    if phase == BossPhase::Defeated {
        return Err(BossRefusal::AlreadyDefeated);
    }
    PHASE_TRANSITIONS
        .iter()
        .find(|row| row.from == phase && row.event == event)
        .map(|row| row.to)
        .ok_or(BossRefusal::EventNotValidInPhase { phase })
}

/// The recovered ability row for a locale name id.
pub fn ability(name_id: u32) -> Option<&'static Ability> {
    ABILITIES.iter().find(|row| row.name_id == name_id)
}

/// The authored attack phase timing for an animation slot.
pub fn attack_timing(slot: &str) -> Option<&'static AttackTiming> {
    ATTACK_TIMINGS.iter().find(|row| row.slot == slot)
}

/// Structural invariants over the recovered tables.
pub fn validate() -> Result<(), BossError> {
    let mut seen_ids: Vec<u32> = Vec::new();
    for row in ABILITIES {
        if row.name.trim().is_empty() {
            return Err(BossError::EmptyAbilityName(row.name_id));
        }
        if seen_ids.contains(&row.name_id) {
            return Err(BossError::DuplicateAbility(row.name_id));
        }
        seen_ids.push(row.name_id);
        if let Some(slot) = row.slot {
            let timing = attack_timing(slot).ok_or(BossError::MissingAttackTiming(slot))?;
            if row.clip != Some(timing.clip) {
                return Err(BossError::InvalidAttackTiming(slot));
            }
        }
    }
    let mut seen_slots: Vec<&str> = Vec::new();
    for row in ATTACK_TIMINGS {
        if seen_slots.contains(&row.slot) {
            return Err(BossError::DuplicateAttackSlot(row.slot));
        }
        seen_slots.push(row.slot);
        if !row.contact_seconds.is_finite() || row.contact_seconds <= 0.0 {
            return Err(BossError::InvalidAttackTiming(row.slot));
        }
        if let Some(interrupt) = row.interrupt_seconds {
            if !interrupt.is_finite() || interrupt <= row.contact_seconds {
                return Err(BossError::InvalidAttackTiming(row.slot));
            }
        }
    }
    for (index, (portal, objective)) in PORTAL_OBJECTIVES.iter().enumerate() {
        if *portal as usize != index + 1 || *objective == 0 {
            return Err(BossError::PortalObjectivesOutOfOrder);
        }
    }
    for (index, row) in PHASE_TRANSITIONS.iter().enumerate() {
        if PHASE_TRANSITIONS[..index]
            .iter()
            .any(|other| other.from == row.from && other.event == row.event)
        {
            return Err(BossError::DuplicatePhaseTransition);
        }
    }
    for row in BARK_RECORDINGS {
        if !row.seconds.is_finite() || row.seconds <= 0.0 {
            return Err(BossError::InvalidBarkDuration(row.asset));
        }
    }
    Ok(())
}
