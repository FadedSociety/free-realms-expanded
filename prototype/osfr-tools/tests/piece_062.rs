//! Work item 062 acceptance: the recovered Necronomicus Unleashed boss phases,
//! abilities and authored attack timing.
//!
//! ORIGINAL evidence used by these vectors (all outside this test, re-read by
//! `scripts/pieces/piece_062.py`):
//! * `lab/reports/job-catalogue-20261009T022023Z/locale-en_us.json` - the preserved original
//!   locale export. Encounter 432 (`Encounter 432 - Necronomicus' Lair`, 5100662), the
//!   `Destroy Zombie Portal 1..8` objectives (5100669-5100683), the three barks (5100650,
//!   5100652, 5100653), the buff packages (5100686/5100688/5100690), the ability names and
//!   descriptions (5100693-5100696, 5100994-5101001, 5101015/5101016) and the `[dev]
//!   Necronomicus Phase 3 Basic Attack` row (5100995) all resolve there byte for byte.
//! * `lab/extracted/client-assets/pack_01/003350.adr` - `necronomicus_m.adr`, the original actor
//!   definition. Its animation bank binds 31 slots to clips (`com_cast_01` ->
//!   `necronomicus_com_ground_pound.gr2`, `com_cast_02` -> `necronomicus_com_360_attack.gr2`,
//!   `com_cast_03` -> `necronomicus_com_chest_laser.gr2`, `com_h2h_attack_01..04` -> clap /
//!   flick / kick / headbutt) and its `ap_contact` / `ap_interrupt` table carries the authored
//!   per-slot phase times.
//! * `lab/extracted/client-assets/catalog.tsv` - the supplied pack catalogue. Every clip the
//!   actor binds resolves to a row with a matching CRC32 except
//!   `necronomicus_spawn_stand_loop_02.gr2` and `necronomicus_spawn_stand_loop_03.gr2`.
//! * `lab/references/Sanctuary/src/Resources/Npcs.json` - the community NPC row 3283
//!   (name 440729, model 4220, `necronomicus_m.adr`).
//! * `lab/references/freerealms-wiki/wiki.sqlite` - the `Necronomicus` page (skeletal form,
//!   skeleton pieces and zombie portal on defeat, re-formed powered skeletal creature that
//!   smacks the ground) and the `Battles` page (difficulty 5, combat job level 13).
//!
//! BOUNDARIES: this module is recovery metadata plus one pure phase-order helper. It grants no
//! loot, damage, health or reward. Server-side cooldowns, phase health thresholds, portal entity
//! ids, the bark-to-cue pairing and any decoded GR2 clip duration were not recovered and are
//! reported as gaps.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers `pub mod piece_062;` under `src/restoration/mod.rs`/`src/lib.rs`.
#[path = "../src/restoration/piece_062.rs"]
#[allow(dead_code)]
mod piece_062;

use piece_062::{
    ATTACK_TIMINGS, BARK_RECORDINGS, BATTLE_DIFFICULTY, BATTLE_MIN_COMBAT_LEVEL, BossEvent,
    BossPhase, BossRefusal, CHEST_BEAM_END_SECONDS, CHEST_BEAM_HIT_FLASH_SECONDS,
    CUE_TO_RECORDING_RECOVERED, DIALOGUE_CUES, ENCOUNTER_ID, ENCOUNTER_NAME_ID, Evidence,
    GR2_CLIP_DURATION_DECODED, GROUND_SHAKE_CUE, GROUND_SHAKE_EFFECT, GROUND_SHAKE_SLOT,
    NECRONOMICUS_MODEL_FILE, NECRONOMICUS_NAME_ID, NECRONOMICUS_NPC_ID, OVERWORLD_ABILITY_TEXT_IDS,
    PHASE_TRANSITIONS, PORTAL_OBJECTIVES, SERVER_ABILITY_TIMING_RECOVERED, SLOT_EFFECT_BINDINGS,
    SPAWN_SMOKE_END_SECONDS, ability, advance, attack_timing, validate,
};

#[test]
fn boss_identity_matches_the_pinned_sources() {
    assert_eq!(NECRONOMICUS_NPC_ID, 3283);
    assert_eq!(NECRONOMICUS_NAME_ID, 440729);
    assert_eq!(NECRONOMICUS_MODEL_FILE, "necronomicus_m.adr");
    assert_eq!(ENCOUNTER_ID, 432);
    assert_eq!(ENCOUNTER_NAME_ID, 5100662);
    assert_eq!(BATTLE_DIFFICULTY, 5);
    assert_eq!(BATTLE_MIN_COMBAT_LEVEL, 13);
}

#[test]
fn phase_transitions_follow_the_documented_order() {
    assert_eq!(
        advance(BossPhase::Skeletal, BossEvent::SkeletalDefeated),
        Ok(BossPhase::Portals)
    );
    assert_eq!(
        advance(BossPhase::Portals, BossEvent::PortalsDestroyed),
        Ok(BossPhase::Empowered)
    );
    assert_eq!(
        advance(BossPhase::Empowered, BossEvent::EmpoweredDefeated),
        Ok(BossPhase::Defeated)
    );

    for (phase, event) in [
        (BossPhase::Skeletal, BossEvent::PortalsDestroyed),
        (BossPhase::Skeletal, BossEvent::EmpoweredDefeated),
        (BossPhase::Portals, BossEvent::SkeletalDefeated),
        (BossPhase::Portals, BossEvent::EmpoweredDefeated),
        (BossPhase::Empowered, BossEvent::SkeletalDefeated),
        (BossPhase::Empowered, BossEvent::PortalsDestroyed),
    ] {
        assert_eq!(
            advance(phase, event),
            Err(BossRefusal::EventNotValidInPhase { phase }),
            "{phase:?} + {event:?}"
        );
    }
    for event in [
        BossEvent::SkeletalDefeated,
        BossEvent::PortalsDestroyed,
        BossEvent::EmpoweredDefeated,
    ] {
        assert_eq!(
            advance(BossPhase::Defeated, event),
            Err(BossRefusal::AlreadyDefeated)
        );
    }

    assert_eq!(PHASE_TRANSITIONS.len(), 3);
    assert!(
        PHASE_TRANSITIONS
            .iter()
            .all(|row| row.evidence == Evidence::Wiki)
    );
}

#[test]
fn ability_rows_keep_their_recovered_slots_and_effects() {
    let ground_pound = ability(5100996).expect("Ground Pound row");
    assert_eq!(ground_pound.name, "Ground Pound");
    assert_eq!(ground_pound.description_id, Some(5100997));
    assert_eq!(ground_pound.slot, Some("com_cast_01"));
    assert_eq!(ground_pound.clip, Some("necronomicus_com_ground_pound.gr2"));
    assert_eq!(ground_pound.effect, Some("ground_pound_effect"));
    assert_eq!(ground_pound.evidence, Evidence::OriginalAdr);

    let hungering = ability(5100998).expect("Hungering Darkness row");
    assert_eq!(hungering.slot, Some("com_cast_02"));
    assert_eq!(hungering.clip, Some("necronomicus_com_360_attack.gr2"));
    assert_eq!(hungering.effect, Some("head_particles"));

    let heart = ability(5101000).expect("Heart of Darkness row");
    assert_eq!(heart.slot, Some("com_cast_03"));
    assert_eq!(heart.clip, Some("necronomicus_com_chest_laser.gr2"));
    assert_eq!(heart.effect, Some("magic_beam_shot"));

    // The signature blast and the splash are named in the original locale without a recovered
    // animation slot; the module keeps the slot empty instead of guessing one.
    let gloamfire = ability(5100693).expect("Necrotic Gloamfire row");
    assert_eq!(gloamfire.slot, None);
    assert_eq!(gloamfire.clip, None);
    assert_eq!(ability(5100695).unwrap().slot, None);
    assert_eq!(ability(5101015).unwrap().name, "Necronomicus Cave-in");
    assert_eq!(
        ability(5100995).unwrap().name,
        "[dev] Necronomicus Phase 3 Basic Attack"
    );
    assert!(ability(1).is_none());

    assert_eq!(
        OVERWORLD_ABILITY_TEXT_IDS,
        &[5101009, 5101011, 5101013, 5101015]
    );

    assert_eq!(SLOT_EFFECT_BINDINGS.len(), 3);
    assert_eq!(SLOT_EFFECT_BINDINGS[1].effect, "Head_trail");
    assert_eq!(SLOT_EFFECT_BINDINGS[1].instance, "head_particles");
    assert_eq!(SLOT_EFFECT_BINDINGS[2].instance, "magic_beam_shot");
    assert_eq!(GROUND_SHAKE_SLOT, "com_cast_01");
    assert_eq!(GROUND_SHAKE_EFFECT, "necronomicus_ground_shake");
    assert_eq!(GROUND_SHAKE_CUE, "ground_shake");
}

#[test]
fn attack_timing_table_matches_the_original_actor_definition() {
    assert_eq!(ATTACK_TIMINGS.len(), 7);
    let cases: [(&str, f32, Option<f32>); 7] = [
        ("com_h2h_attack_01", 0.7036, Some(0.8341)),
        ("com_h2h_attack_02", 0.7820, None),
        ("com_h2h_attack_03", 1.0253, Some(1.2724)),
        ("com_h2h_attack_04", 0.5998, Some(0.7583)),
        ("com_cast_01", 1.1955, Some(1.4593)),
        ("com_cast_02", 1.5978, Some(3.1642)),
        ("com_cast_03", 3.0953, Some(4.5277)),
    ];
    for (slot, contact, interrupt) in cases {
        let row = attack_timing(slot).unwrap_or_else(|| panic!("{slot} timing row"));
        assert!(
            (row.contact_seconds - contact).abs() < 1e-3,
            "{slot} contact {} != {contact}",
            row.contact_seconds
        );
        match (row.interrupt_seconds, interrupt) {
            (Some(found), Some(expected)) => assert!(
                (found - expected).abs() < 1e-3,
                "{slot} interrupt {found} != {expected}"
            ),
            (None, None) => {}
            other => panic!("{slot} interrupt mismatch {other:?}"),
        }
        if let Some(interrupt) = row.interrupt_seconds {
            assert!(
                row.contact_seconds < interrupt,
                "{slot} contact after interrupt"
            );
        }
    }
    assert!(attack_timing("loc_stand").is_none());
}

#[test]
fn portal_objectives_record_eight_original_targets() {
    assert_eq!(PORTAL_OBJECTIVES.len(), 8);
    assert_eq!(PORTAL_OBJECTIVES[0], (1, 5100669));
    assert_eq!(PORTAL_OBJECTIVES[7], (8, 5100683));
}

#[test]
fn barks_keep_their_measured_durations_without_inventing_a_cue_pairing() {
    assert_eq!(DIALOGUE_CUES.len(), 3);
    assert_eq!(DIALOGUE_CUES[0].cue, "DIA_Necronomicus_Friends");
    assert_eq!(DIALOGUE_CUES[0].text_id, 5100650);
    assert_eq!(DIALOGUE_CUES[2].cue, "DIA_Necronomicus_Power_Eternal");
    assert_eq!(DIALOGUE_CUES[2].text_id, 5100653);

    assert_eq!(BARK_RECORDINGS.len(), 3);
    assert!((BARK_RECORDINGS[0].seconds - 1.392).abs() < 5e-3);
    assert_eq!(BARK_RECORDINGS[0].asset, "DX_NECRONOMICUS_BACK.mp3");
    assert!((BARK_RECORDINGS[2].seconds - 3.552).abs() < 5e-3);
    assert!(!CUE_TO_RECORDING_RECOVERED);
}

#[test]
fn effect_timing_matches_the_original_particle_definitions() {
    assert!((SPAWN_SMOKE_END_SECONDS - 3.0).abs() < 1e-6);
    assert!((CHEST_BEAM_END_SECONDS - 3.0).abs() < 1e-6);
    assert!((CHEST_BEAM_HIT_FLASH_SECONDS - 0.5).abs() < 1e-6);
}

#[test]
fn unrecovered_server_timing_stays_a_gap() {
    assert!(!SERVER_ABILITY_TIMING_RECOVERED);
    assert!(!GR2_CLIP_DURATION_DECODED);
    assert!(validate().is_ok());
}
