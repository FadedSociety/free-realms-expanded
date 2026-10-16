use osfr_tools::abilities;

use abilities::{
    AbilityCatalog, AbilityDefinition, AbilityEffect, AbilityFailure, AbilityTarget, ActorKind,
    CombatState, EffectKind, EffectRecipient, RespawnWindow, Sourced, TargetKind, use_ability,
};

const CASTER: u64 = 0x0102_0304_0506_0708;
const TARGET: u64 = 0x1112_1314_1516_1718;
const ABILITY_ID: i32 = 77;
const EFFECT_ID: i32 = 9001;

#[test]
fn job_effect_bounds_reject_zero_timers_unbounded_targets_and_nonfinite_numbers() {
    use abilities::JobEffect;
    for value in [
        serde_json::json!({"kind":"damage_over_time","multiplier":1,"duration_ms":1000,"interval_ms":0}),
        serde_json::json!({"kind":"damage_over_time","multiplier":1,"duration_ms":30001,"interval_ms":1000}),
        serde_json::json!({"kind":"area_damage","radius":6,"max_targets":17,"centered_on_caster":true}),
        serde_json::json!({"kind":"stun","duration_ms":10001}),
        serde_json::json!({"kind":"buff","recipient":"caster","duration_ms":1000,"damage_multiplier":100,"damage_taken_multiplier":1,"crit_bonus":0,"crit_damage_bonus":0}),
    ] {
        assert!(!serde_json::from_value::<JobEffect>(value).unwrap().valid());
    }
    assert!(
        !JobEffect::AreaDamage {
            radius: f32::NAN,
            max_targets: 1,
            centered_on_caster: true
        }
        .valid()
    );
    assert!(
        !JobEffect::DamageOverTime {
            multiplier: f32::INFINITY,
            duration_ms: 1000,
            interval_ms: 1000
        }
        .valid()
    );
    assert!(
        serde_json::from_value::<JobEffect>(
            serde_json::json!({"kind":"stun","duration_ms":1000,"extra":true})
        )
        .is_err()
    );
}

fn sourced<T>(value: T) -> Sourced<T> {
    Sourced {
        value,
        source: "(R) synthetic test definition".to_owned(),
    }
}

fn damage(amount: i32) -> AbilityEffect {
    AbilityEffect {
        kind: sourced(EffectKind::Damage),
        recipient: sourced(EffectRecipient::Target),
        amount: Some(sourced(amount)),
        effect_id: None,
        duration_ms: None,
        icon_id: None,
        name_id: None,
    }
}

fn apply_effect(duration_ms: u64) -> AbilityEffect {
    AbilityEffect {
        kind: sourced(EffectKind::ApplyEffect),
        recipient: sourced(EffectRecipient::Target),
        amount: None,
        effect_id: Some(sourced(EFFECT_ID)),
        duration_ms: Some(sourced(duration_ms)),
        icon_id: Some(sourced(0)),
        name_id: Some(sourced(0)),
    }
}

fn definition(target_kind: TargetKind, effects: Vec<AbilityEffect>) -> AbilityDefinition {
    AbilityDefinition {
        id: sourced(ABILITY_ID),
        name: sourced("Synthetic ability".to_owned()),
        job_profile: sourced(11),
        slot: sourced(0),
        target_kind: sourced(target_kind),
        range: sourced(5.0),
        cost: sourced(10),
        cooldown_ms: sourced(100),
        cast_time_ms: sourced(0),
        effects,
    }
}

fn caster() -> CombatState {
    let mut actor = CombatState::new(CASTER, ActorKind::Player, 11, [0.0; 3], 100, 100, 50, 100);
    actor.owned_abilities.insert(ABILITY_ID);
    actor.slots.insert(0, ABILITY_ID);
    actor
}

fn hostile() -> CombatState {
    CombatState::new(TARGET, ActorKind::Hostile, 0, [3.0, 0.0, 0.0], 40, 50, 0, 0)
}

fn packet(family: u16, sub: u16, body: &[u8]) -> Vec<u8> {
    let mut bytes = [family.to_le_bytes(), sub.to_le_bytes()].concat();
    bytes.extend(body);
    bytes
}

fn push_i32(bytes: &mut Vec<u8>, value: i32) {
    bytes.extend(value.to_le_bytes());
}

fn expected_apply_effect(tag_id: i32, seconds: i32) -> Vec<u8> {
    let mut bytes = packet(38, 16, &[]);
    push_i32(&mut bytes, tag_id);
    for value in [0, tag_id, 0, 0, 0, 0, seconds] {
        push_i32(&mut bytes, value);
    }
    bytes.push(0);
    bytes.extend(0_i64.to_le_bytes());
    for _ in 0..4 {
        push_i32(&mut bytes, 0);
    }
    bytes.extend(0_i64.to_le_bytes());
    push_i32(&mut bytes, 0);
    push_i32(&mut bytes, 0);
    bytes.extend([0, 0, 0]);
    push_i32(&mut bytes, 0);
    bytes.push(0);
    push_i32(&mut bytes, 0);
    push_i32(&mut bytes, 0);
    bytes
}

fn assert_failure(
    mut caster: CombatState,
    mut target: Option<CombatState>,
    ability: &AbilityDefinition,
    now_ms: u64,
    failure: AbilityFailure,
    code: i32,
) {
    let caster_before = caster.clone();
    let target_before = target.clone();
    let result = match target.as_mut() {
        Some(actor) => use_ability(&mut caster, AbilityTarget::Actor(actor), ability, now_ms),
        None => use_ability(&mut caster, AbilityTarget::Missing, ability, now_ms),
    };
    assert_eq!(result.failure, Some(failure));
    assert_eq!(result.client_failure_code, Some(code));
    assert_eq!(result.packets, vec![packet(36, 1, &code.to_le_bytes())]);
    assert_eq!(caster, caster_before);
    assert_eq!(target, target_before);
}

#[test]
fn medic_fixture_loads_with_source_tags_and_client_profile() {
    let catalog = AbilityCatalog::from_json(include_bytes!("fixtures/abilities/medic.json"))
        .expect("Medic ability fixture");
    let fixture_path = std::path::Path::new(file!())
        .parent()
        .unwrap()
        .join("fixtures/abilities/medic.json");
    let loaded_from_path = AbilityCatalog::load(fixture_path).expect("load JSON fixture path");
    assert_eq!(loaded_from_path, catalog);
    assert_eq!(catalog.iter().count(), 3);
    let first_aid = catalog.get(10001).unwrap();
    assert_eq!(first_aid.job_profile.value, 11);
    assert_eq!(first_aid.name.value, "First Aid");
    assert_eq!(first_aid.effects[0].amount.as_ref().unwrap().value, 250);
    for ability in catalog.iter() {
        for source in [
            &ability.id.source,
            &ability.name.source,
            &ability.job_profile.source,
            &ability.slot.source,
            &ability.target_kind.source,
            &ability.range.source,
            &ability.cost.source,
            &ability.cooldown_ms.source,
            &ability.cast_time_ms.source,
        ] {
            assert!(!source.is_empty());
        }
        for effect in &ability.effects {
            assert!(!effect.kind.source.is_empty());
            assert!(!effect.recipient.source.is_empty());
            assert!(
                effect
                    .amount
                    .as_ref()
                    .is_none_or(|value| !value.source.is_empty())
            );
            assert!(
                effect
                    .effect_id
                    .as_ref()
                    .is_none_or(|value| !value.source.is_empty())
            );
            assert!(
                effect
                    .duration_ms
                    .as_ref()
                    .is_none_or(|value| !value.source.is_empty())
            );
            assert!(
                effect
                    .icon_id
                    .as_ref()
                    .is_none_or(|value| !value.source.is_empty())
            );
            assert!(
                effect
                    .name_id
                    .as_ref()
                    .is_none_or(|value| !value.source.is_empty())
            );
        }
    }
}

#[test]
fn damage_cast_updates_energy_health_and_emits_exact_reader_bytes() {
    let mut ability = definition(TargetKind::Hostile, vec![damage(25), apply_effect(2000)]);
    ability.cast_time_ms.value = 500;
    let mut caster = caster();
    let mut target = hostile();

    let result = use_ability(
        &mut caster,
        AbilityTarget::Actor(&mut target),
        &ability,
        100,
    );
    assert_eq!(result.failure, None);
    assert_eq!(caster.energy, 40);
    assert_eq!(target.health, 15);
    assert_eq!(caster.cooldown_until_ms.get(&0), Some(&200));

    let mut cast = packet(36, 3, &[]);
    cast.extend(CASTER.to_le_bytes());
    cast.extend(TARGET.to_le_bytes());
    for value in [ABILITY_ID, 0, 0] {
        push_i32(&mut cast, value);
    }
    cast.extend(0.5_f32.to_le_bytes());
    cast.push(0);

    let mut attack = packet(32, 7, &[]);
    for guid in [CASTER, CASTER, TARGET] {
        attack.extend(guid.to_le_bytes());
    }
    for value in [25, 50, 0] {
        push_i32(&mut attack, value);
    }
    attack.extend([0, 0]);
    push_i32(&mut attack, 0);
    push_i32(&mut attack, 15);

    let mut target_health = packet(35, 5, &[]);
    target_health.extend(TARGET.to_le_bytes());
    for value in [15, 50, 0] {
        push_i32(&mut target_health, value);
    }
    let mut energy = packet(38, 13, &[]);
    push_i32(&mut energy, 40);
    push_i32(&mut energy, 100);
    energy.push(0);

    assert_eq!(
        result.packets,
        vec![
            cast,
            attack,
            target_health,
            energy,
            expected_apply_effect(EFFECT_ID, 2)
        ]
    );
}

#[test]
fn healing_emits_hitpoint_delta_and_health_energy_updates() {
    let catalog = AbilityCatalog::from_json(include_bytes!("fixtures/abilities/medic.json"))
        .expect("Medic ability fixture");
    let ability = catalog.get(10001).unwrap();
    let mut caster = CombatState::new(CASTER, ActorKind::Player, 11, [0.0; 3], 100, 100, 50, 100);
    caster.owned_abilities.insert(ability.id.value);
    caster.slots.insert(ability.slot.value, ability.id.value);
    let mut target = CombatState::new(
        TARGET,
        ActorKind::Player,
        11,
        [1.0, 0.0, 0.0],
        40,
        100,
        30,
        100,
    );

    let result = use_ability(&mut caster, AbilityTarget::Actor(&mut target), ability, 0);
    assert_eq!(target.health, 100);
    assert_eq!(caster.energy, 40);

    let mut delta = packet(35, 35, &[]);
    delta.extend(CASTER.to_le_bytes());
    delta.extend(TARGET.to_le_bytes());
    delta.push(1);
    push_i32(&mut delta, 60);
    push_i32(&mut delta, 0);
    push_i32(&mut delta, 0);
    delta.push(0);
    let mut hp = packet(38, 1, &[]);
    push_i32(&mut hp, 100);
    push_i32(&mut hp, 100);
    let mut energy = packet(38, 13, &[]);
    push_i32(&mut energy, 40);
    push_i32(&mut energy, 100);
    energy.push(0);
    assert_eq!(result.packets, vec![delta, hp, energy]);
}

#[test]
fn first_aid_can_select_the_caster_as_its_target() {
    let catalog = AbilityCatalog::from_json(include_bytes!("fixtures/abilities/medic.json"))
        .expect("Medic ability fixture");
    let ability = catalog.get(10001).unwrap();
    let mut caster = caster();
    caster.owned_abilities.insert(ability.id.value);
    caster.slots.insert(ability.slot.value, ability.id.value);
    caster.health = 40;

    let result = use_ability(&mut caster, AbilityTarget::SelfTarget, ability, 100);
    assert_eq!(result.failure, None);
    assert_eq!(caster.health, 100);
    assert_eq!(caster.energy, 40);
    let mut delta = packet(35, 35, &[]);
    delta.extend(CASTER.to_le_bytes());
    delta.extend(CASTER.to_le_bytes());
    delta.push(1);
    push_i32(&mut delta, 60);
    push_i32(&mut delta, 0);
    push_i32(&mut delta, 0);
    delta.push(0);
    let mut hp = packet(38, 1, &[]);
    push_i32(&mut hp, 100);
    push_i32(&mut hp, 100);
    let mut energy = packet(38, 13, &[]);
    push_i32(&mut energy, 40);
    push_i32(&mut energy, 100);
    energy.push(0);
    assert_eq!(result.packets, vec![delta, hp, energy]);
}

#[test]
fn every_validation_failure_returns_client_code_without_mutation() {
    let ability = definition(TargetKind::Hostile, vec![damage(1)]);
    let base_caster = caster();
    let base_target = hostile();

    let mut value = base_caster.clone();
    value.owned_abilities.clear();
    assert_failure(
        value,
        Some(base_target.clone()),
        &ability,
        1000,
        AbilityFailure::NotOwned,
        286,
    );

    let mut value = base_caster.clone();
    value.slots.insert(0, 99);
    assert_failure(
        value,
        Some(base_target.clone()),
        &ability,
        1000,
        AbilityFailure::SlotMismatch,
        286,
    );

    let mut value = base_caster.clone();
    value.job_profile = 12;
    assert_failure(
        value,
        Some(base_target.clone()),
        &ability,
        1000,
        AbilityFailure::WrongJob,
        286,
    );

    assert_failure(
        base_caster.clone(),
        None,
        &ability,
        1000,
        AbilityFailure::MissingTarget,
        288,
    );

    let mut dead = base_target.clone();
    dead.health = 0;
    assert_failure(
        base_caster.clone(),
        Some(dead),
        &ability,
        1000,
        AbilityFailure::InvalidTarget,
        286,
    );

    let mut invalid_kind = base_target.clone();
    invalid_kind.kind = ActorKind::Neutral;
    assert_failure(
        base_caster.clone(),
        Some(invalid_kind),
        &ability,
        1000,
        AbilityFailure::InvalidTarget,
        286,
    );

    let mut far = base_target.clone();
    far.position = [10.0, 0.0, 0.0];
    assert_failure(
        base_caster.clone(),
        Some(far),
        &ability,
        1000,
        AbilityFailure::OutOfRange,
        286,
    );

    let mut low_energy = base_caster.clone();
    low_energy.energy = 9;
    assert_failure(
        low_energy,
        Some(base_target.clone()),
        &ability,
        1000,
        AbilityFailure::InsufficientEnergy,
        285,
    );

    let mut cooling = base_caster.clone();
    cooling.cooldown_until_ms.insert(0, 1001);
    assert_failure(
        cooling,
        Some(base_target.clone()),
        &ability,
        1000,
        AbilityFailure::Cooldown,
        287,
    );

    let mut casting = base_caster.clone();
    casting.casting_until_ms = Some(1001);
    assert_failure(
        casting,
        Some(base_target.clone()),
        &ability,
        1000,
        AbilityFailure::AlreadyCasting,
        286,
    );

    let mut knocked_out = base_caster;
    knocked_out.health = 0;
    assert_failure(
        knocked_out,
        Some(base_target),
        &ability,
        1000,
        AbilityFailure::CasterKnockedOut,
        286,
    );
}

#[test]
fn cooldown_is_rejected_until_the_exact_expiry_time() {
    let ability = definition(TargetKind::Hostile, vec![damage(1)]);
    let mut caster = caster();
    let mut target = hostile();
    assert!(
        use_ability(
            &mut caster,
            AbilityTarget::Actor(&mut target),
            &ability,
            100
        )
        .failure
        .is_none()
    );
    assert_eq!(
        use_ability(
            &mut caster,
            AbilityTarget::Actor(&mut target),
            &ability,
            199
        )
        .failure,
        Some(AbilityFailure::Cooldown)
    );
    assert!(
        use_ability(
            &mut caster,
            AbilityTarget::Actor(&mut target),
            &ability,
            200
        )
        .failure
        .is_none()
    );
}

#[test]
fn lethal_damage_emits_knockout_health_and_respawn_sequence() {
    let ability = definition(TargetKind::Any, vec![damage(10)]);
    let mut caster = caster();
    let mut target = CombatState::new(TARGET, ActorKind::Player, 11, [1.0, 0.0, 0.0], 10, 10, 0, 0);
    target.respawn_window = Some(RespawnWindow {
        timer_ms: 10_000,
        revive_cost: 0,
    });
    let result = use_ability(
        &mut caster,
        AbilityTarget::Actor(&mut target),
        &ability,
        100,
    );
    assert_eq!(target.health, 0);

    let mut attack = packet(32, 7, &[]);
    for guid in [CASTER, CASTER, TARGET] {
        attack.extend(guid.to_le_bytes());
    }
    for value in [10, 10, 0] {
        push_i32(&mut attack, value);
    }
    attack.extend([0, 0]);
    push_i32(&mut attack, 0);
    push_i32(&mut attack, 0);
    let mut state = packet(35, 20, &[]);
    state.extend(TARGET.to_le_bytes());
    push_i32(&mut state, 1 << 7);
    let mut hp = packet(38, 1, &[]);
    push_i32(&mut hp, 0);
    push_i32(&mut hp, 10);
    let mut respawn = packet(41, 125, &[]);
    push_i32(&mut respawn, 10_000);
    push_i32(&mut respawn, 0);
    let mut energy = packet(38, 13, &[]);
    push_i32(&mut energy, 40);
    push_i32(&mut energy, 100);
    energy.push(0);
    assert_eq!(result.packets, vec![attack, state, hp, respawn, energy]);
}

#[test]
fn applied_effect_expires_once_with_remove_tag_message() {
    let mut ability = definition(TargetKind::Hostile, vec![apply_effect(2000)]);
    ability.cost.value = 0;
    let mut caster = caster();
    let mut target = hostile();
    let result = use_ability(
        &mut caster,
        AbilityTarget::Actor(&mut target),
        &ability,
        100,
    );
    assert_eq!(result.packets, vec![expected_apply_effect(EFFECT_ID, 2)]);
    assert_eq!(
        target.active_effects.get(&EFFECT_ID).unwrap().expires_at_ms,
        2100
    );
    assert!(abilities::tick_effects(&mut target, 2099).is_empty());
    assert_eq!(
        abilities::tick_effects(&mut target, 2100),
        vec![packet(38, 17, &EFFECT_ID.to_le_bytes())]
    );
    assert!(target.active_effects.is_empty());
}
