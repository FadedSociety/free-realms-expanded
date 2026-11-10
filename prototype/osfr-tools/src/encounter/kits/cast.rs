//! The engine side of a kit cast: refusals, the strike applied through the engine's own hit
//! path, energy, traits, helpers, and the facts quests read.
use super::status::ME;
use super::table::Trait;
use super::*;
use crate::abilities::{build_energy, build_failure};

/// The passive part of the job's traits at its level, folded into the damage, mitigation and
/// crit of `SkillEffects`.
#[derive(Clone, Debug, PartialEq)]
pub(in crate::encounter) struct Passives {
    pub dealt: f32,
    pub taken: f32,
    pub crit: u32,
    pub crit_damage: f32,
    pub energy_per_tick: i32,
}

impl Default for Passives {
    fn default() -> Self {
        Self {
            dealt: 1.0,
            taken: 1.0,
            crit: 0,
            crit_damage: 0.0,
            energy_per_tick: 0,
        }
    }
}

/// Summoned allies (Nurse!, Shadow Army): until the brain's summon can place real actors they
/// are strikes on the nearest enemy, each helper once per `every_ms`.
#[derive(Clone, Debug, PartialEq)]
pub(in crate::encounter) struct Helpers {
    pub until: u64,
    pub next: u64,
    pub every_ms: u64,
    pub damage: u32,
    pub count: u8,
    pub ability: i32,
}

/// Helpers reach this far from the player (R).
const HELPER_REACH: f32 = 15.0;
/// Key of the damage over time a status adds to hits, apart from any ability's own.
const ON_HIT_DOT: i32 = -7001;

impl EncounterEngine {
    fn refusal(id: i32) -> EncounterOutcome {
        EncounterOutcome {
            replies: vec![build_failure(id)],
            ..Default::default()
        }
    }

    /// Whether the kits decide `profile`'s casts (the table is loaded and knows the job).
    pub(in crate::encounter) fn kit_applies(&self, profile: i32) -> bool {
        match (&self.skills.kit.table, u32::try_from(profile)) {
            (Some(table), Ok(profile)) => table.supports(profile),
            _ => false,
        }
    }

    /// The kit and damage tier `profile` casts with right now.
    pub fn kit_of(&self, profile: i32) -> Option<(Kit, u8)> {
        let table = self.skills.kit.table.as_ref()?;
        let job = u32::try_from(profile).ok()?;
        let none = WeaponFacts::default();
        let weapon = self.kit_state.weapons.get(&profile).unwrap_or(&none);
        let level = self.job_level.clamp(1, 20) as u8;
        Some((table.kit_for(job, weapon)?.clone(), table.tier(job, level, weapon)))
    }

    fn refresh_passives(&mut self) {
        let mut passive = Passives::default();
        let mut threat = 0;
        if let (Some(table), Ok(job)) = (&self.skills.kit.table, u32::try_from(self.player_profile)) {
            let level = self.job_level.clamp(1, 20) as u8;
            for p in table.traits(job, level).filter_map(|t| t.passive.as_ref()) {
                passive.dealt *= p.dealt.unwrap_or(1.0);
                passive.taken *= p.taken.unwrap_or(1.0);
                passive.crit += p.crit;
                passive.crit_damage += p.crit_damage;
                passive.energy_per_tick += p.energy_per_tick;
            }
            let tier = usize::from(table::tier_of_level(level)) - 1;
            threat = (table.plain_basic[tier] as f32 * table.rider_defaults.taunt_threat_basics) as u32;
        }
        self.skills.kit.passive = passive;
        self.skills.kit.taunt_threat = threat;
    }

    /// One roll in 0..1 on the engine's own generator (the one crits use).
    fn kit_roll(&mut self) -> f32 {
        self.crit_seed = self
            .crit_seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.crit_seed >> 40) & 0xFF_FFFF) as f32 / (1u32 << 24) as f32
    }

    fn enemy_bodies(&self) -> Vec<Body> {
        self.living_enemies()
            .into_iter()
            .filter_map(|(guid, position)| {
                let (actor, state) = self.enemy(guid)?;
                Some(Body {
                    guid,
                    kind: u32::try_from(actor.definition_id).unwrap_or(0),
                    position,
                    health: state.health,
                    max_health: actor.max_health,
                })
            })
            .collect()
    }

    /// One hit of a kit ability, trait or helper through the engine's hit path (shield,
    /// defeat, rewards, win), with the target's own crit vulnerability counted.
    fn kit_hit(
        &mut self,
        guid: u64,
        damage: u32,
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) -> io::Result<()> {
        let Some(at) = self.enemy(guid).filter(|(_, s)| s.health > 0).map(|(_, s)| s.position) else {
            return Ok(());
        };
        self.skills.kit.crit_against = self.skills.kit.book.crit_against(guid);
        let hit = self.strike_damage(guid, at, f32::MAX, damage, now_ms, false);
        self.skills.kit.crit_against = 0;
        skills::merge_skill_outcome(out, hit?);
        Ok(())
    }

    fn kit_heal_self(&mut self, amount: u32, now_ms: u64, out: &mut EncounterOutcome) {
        if self.state.player_health == 0 || amount == 0 {
            return;
        }
        let before = self.state.player_health;
        self.state.player_health = before.saturating_add(amount).min(self.max_health);
        let healed = self.state.player_health - before;
        self.skills.kit.last_health = Some(self.state.player_health);
        self.log_fight(now_ms, "heal", "you", "you", i64::from(healed));
        let me = self.shared_sender.unwrap_or(1);
        out.replies.push(wire::hitpoints(
            me,
            me,
            true,
            self.max_health as i32,
            self.state.player_health as i32,
            healed as i32,
            false,
        ));
        out.replies.push(player_hp(self.state.player_health, self.max_health));
        out.changed = true;
    }

    fn kit_energy(&mut self, delta: i32, out: &mut EncounterOutcome) {
        let max = self.skills.kit.table.as_ref().map_or(self.rules.energy.max, |t| t.energy.max);
        let energy = (self.energy + delta).clamp(0, max);
        if energy != self.energy {
            self.energy = energy;
            out.replies.push(build_energy(energy, max));
        }
    }

    /// A press of bar 1 slot `slot` through the weapon kits, for callers that hold no bar
    /// row (tools, the simulator, tests). Refused with 286 when the kits are not live for the job.
    pub fn cast_slot(
        &mut self,
        slot: u8,
        target: u64,
        position: [f32; 3],
        profile: i32,
        now_ms: u64,
    ) -> io::Result<EncounterOutcome> {
        if !self.kit_applies(profile) {
            return Ok(Self::refusal(286));
        }
        self.kit_cast(slot, target, position, profile, now_ms)
    }

    /// A press of bar 1 slot `slot` on `target` (the player's own guid, or anything that is not
    /// an enemy, lets the server pick). Slot 0 fails silently, as the live server did; the
    /// others answer with the client's failure strings.
    pub(in crate::encounter) fn kit_cast(
        &mut self,
        slot: u8,
        target: u64,
        position: [f32; 3],
        profile: i32,
        now_ms: u64,
    ) -> io::Result<EncounterOutcome> {
        if !valid_position(position) {
            return Err(invalid("invalid kit cast position"));
        }
        let refuse = |id: i32| -> io::Result<EncounterOutcome> {
            Ok(if slot == 0 { EncounterOutcome::default() } else { Self::refusal(id) })
        };
        let Some((kit, tier)) = self.kit_of(profile) else {
            return refuse(286);
        };
        let Some(ability) = kit.slot(slot).cloned() else {
            return refuse(286);
        };
        let level = self.job_level.clamp(1, 20) as u8;
        let locked = ability.kind == Kind::Third
            && self.skills.kit.table.as_ref().is_some_and(|t| level < t.slots.third_min_level);
        if locked
            || self.state.player_health == 0
            || self.state.active.as_ref().is_some_and(|a| a.phase != EncounterPhase::Fighting)
        {
            return refuse(286);
        }
        self.skills.expire(now_ms);
        let book = &self.skills.kit.book;
        if book.blocked(ME, "act", now_ms) || book.blocked(ME, "cast", now_ms) {
            return refuse(286);
        }
        if ability.kind == Kind::Basic {
            // One basic per recast: the live server let one through and dropped the rest.
            if self
                .last_attack
                .is_some_and(|last| now_ms.saturating_sub(last) < u64::from(ability.recast_ms))
            {
                return Ok(EncounterOutcome::default());
            }
        } else if self
            .ability_ready
            .get(&ability.ability_id)
            .is_some_and(|ready| now_ms < *ready)
        {
            return refuse(287);
        }
        self.refresh_passives();
        self.skills.kit.position = Some(position);
        let me = self.shared_sender.unwrap_or(1);
        let enemies = self.enemy_bodies();
        let selected = enemies.iter().any(|b| b.guid == target).then_some(target);
        // A named target that is not a live enemy here is a forged or stale GUID: nothing
        // happens, as in `shared_job_ability`. Only the client's own "no target" (0) lets the
        // kit pick one (lead 2026-11-10, playtest_first_hour).
        if target != 0 && target != me && selected.is_none() {
            return Ok(EncounterOutcome::default());
        }
        let allies = [Body {
            guid: me,
            kind: 0,
            position,
            health: self.state.player_health,
            max_health: self.max_health,
        }];
        let caster = Caster {
            guid: me,
            position,
            heading: [0.0; 3],
            profile: u32::try_from(profile).unwrap_or(0),
            level,
            tier,
            energy: self.energy,
            now_ms,
        };
        // A shielded summoner is the last choice: put the unshielded in reach first.
        let mut open: Vec<Body> = enemies.iter().filter(|b| !self.shielded(b.guid)).cloned().collect();
        let targets = Targets { selected, enemies: &open, allies: &allies };
        let strike = match resolve(&kit, slot, &caster, &targets) {
            Err(288) if open.len() < enemies.len() => {
                open = enemies;
                let targets = Targets { selected, enemies: &open, allies: &allies };
                resolve(&kit, slot, &caster, &targets)
            }
            other => other,
        };
        let strike = match strike {
            Ok(strike) => strike,
            Err(id) => return refuse(id),
        };
        let mut out = EncounterOutcome::default();
        // The primary target's kind for a `use_ability` step, read before any hit lands: a
        // lethal blow on the last enemy ends the run and the actor no longer resolves.
        let on_kind = strike
            .primary
            .and_then(|guid| self.enemy(guid))
            .map(|(actor, _)| actor.definition_id)
            .filter(|definition| *definition > 0)
            .and_then(|definition| u32::try_from(definition).ok());
        if !strike.hits.is_empty() || !strike.dots.is_empty() {
            if self.active() && !self.armed {
                self.armed = true;
                self.last_tick = Some(now_ms);
            }
        }
        if ability.kind == Kind::Basic {
            self.last_attack = Some(now_ms);
        }
        out.replies.push(animation(
            me,
            if strike.animation > 0 { strike.animation } else { 1099 },
            false,
        ));
        if strike.effect > 0 {
            out.replies.push(composite_effect(me, strike.effect, position));
        }
        if let Some(launch) = wire::launch(&strike.wire, me, strike.primary.unwrap_or(me)) {
            out.replies.push(wire::cast(me, strike.primary.unwrap_or(me), ability.wire_id(tier)));
            out.replies.push(launch);
        }
        let (crits, kills) = (self.skills.kit.crits, 0usize);
        self.kit_state.cast = Some(Cast { ability: strike.ability_id, effect: None });
        let flat_add = {
            let add = self.skills.kit.book.dealt_add_power5(ME);
            let ladder = self.skills.kit.table.as_ref().map_or([0; 5], |t| t.plain_basic);
            let tier = usize::from(tier.clamp(1, 5)) - 1;
            (f64::from(add) * f64::from(ladder[tier]) / f64::from(ladder[4].max(1))).round() as u32
        };
        let mut landed = Vec::new();
        let applied = (|| -> io::Result<()> {
            for &(guid, damage) in &strike.hits {
                self.kit_hit(guid, damage.saturating_add(flat_add), now_ms, &mut out)?;
                if !landed.contains(&guid) {
                    landed.push(guid);
                }
            }
            for (guid, status, damage) in &strike.conditional {
                if self.skills.kit.book.has(*guid, status, now_ms) {
                    self.kit_hit(*guid, *damage, now_ms, &mut out)?;
                }
            }
            Ok(())
        })();
        self.kit_state.cast = None;
        applied?;
        for (guid, status, ms) in &strike.statuses {
            self.apply_status(*guid, status, *ms, now_ms, &mut out);
        }
        for dot in &strike.dots {
            self.apply_dot(
                &[dot.target],
                strike.ability_id,
                dot.damage,
                u64::from(dot.ms),
                u64::from(dot.every_ms),
                now_ms,
            );
            if !dot.status.is_empty() {
                self.apply_status(dot.target, &dot.status, dot.ms, now_ms, &mut out);
            }
        }
        // A status on the caster that adds damage over time to its hits (Poisoned Blades).
        for extra in self.skills.kit.book.on_hit_dots(ME) {
            let ladder = self.skills.kit.table.as_ref().map_or([0; 5], |t| t.plain_basic);
            let index = usize::from(tier.clamp(1, 5)) - 1;
            let damage =
                (f64::from(extra.power5) * f64::from(ladder[index]) / f64::from(ladder[4].max(1))).round() as u32;
            self.apply_dot(&landed, ON_HIT_DOT, damage, u64::from(extra.ms), u64::from(extra.every_ms), now_ms);
            for &guid in &landed {
                if !extra.status.is_empty() {
                    self.apply_status(guid, &extra.status, extra.ms, now_ms, &mut out);
                }
            }
        }
        if let Some(&(_, amount)) = strike.heals.iter().find(|(guid, _)| *guid == me) {
            self.kit_heal_self(amount, now_ms, &mut out);
        }
        if strike.group_heal > 0 {
            // The rest of the group heals on their own engines; the gateway routes it once.
            let percent = (u64::from(strike.group_heal) * 100 / u64::from(self.max_health.max(1))).clamp(1, 100);
            out.group_heal = Some(GroupHeal { percent: percent as u32, kill: (me, now_ms) });
        }
        if let Some((count, ms, damage, every_ms)) = strike.summon {
            self.skills.kit.helpers = Some(Helpers {
                until: now_ms.saturating_add(u64::from(ms)),
                next: now_ms.saturating_add(u64::from(every_ms)),
                every_ms: u64::from(every_ms),
                damage,
                count,
                ability: strike.ability_id,
            });
        }
        self.kit_energy(strike.energy_gain - strike.energy_cost, &mut out);
        if strike.energy_cost > 0 {
            // The bar starts to refill one full step later (p12 38/13: 0, then 4 a second on).
            let step = self.skills.kit.table.as_ref().map_or(1000, |t| t.energy.tick_ms);
            self.energy_regen_at = now_ms.saturating_add(step);
        }
        if ability.kind != Kind::Basic {
            self.ability_ready
                .insert(strike.ability_id, now_ms.saturating_add(u64::from(strike.recast_ms)));
        }
        for &id in &strike.credit_ids {
            out.abilities_used.push((id, on_kind));
            out.facts.push(FightFact::Used { ability: id, on_kind });
        }
        let focus = strike.primary;
        if !landed.is_empty() {
            self.fire_traits("hit", focus, now_ms, &mut out)?;
        }
        if self.skills.kit.crits != crits {
            self.fire_traits("crit", focus, now_ms, &mut out)?;
        }
        if out.kills.len() > kills {
            self.fire_traits("kill", focus, now_ms, &mut out)?;
        }
        out.changed = true;
        Ok(out)
    }

    /// Run the job's traits that trigger on `on`. `focus` is the enemy the event is about.
    fn fire_traits(
        &mut self,
        on: &str,
        focus: Option<u64>,
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) -> io::Result<()> {
        let (Some(table), Ok(job)) = (self.skills.kit.table.clone(), u32::try_from(self.player_profile))
        else {
            return Ok(());
        };
        let level = self.job_level.clamp(1, 20) as u8;
        let fired: Vec<Trait> = table.traits(job, level).filter(|t| t.on == on).cloned().collect();
        for t in fired {
            if t.chance < 1.0 && self.kit_roll() >= t.chance {
                continue;
            }
            if t.energy > 0 {
                self.kit_energy(t.energy, out);
            }
            if t.heal_pct > 0 {
                let amount = (u64::from(self.max_health) * u64::from(t.heal_pct) / 100) as u32;
                self.kit_heal_self(amount, now_ms, out);
                if t.group {
                    let me = self.shared_sender.unwrap_or(1);
                    out.group_heal = Some(GroupHeal { percent: t.heal_pct, kill: (me, now_ms) });
                }
            }
            let near = |engine: &Self, radius: f32, cap: usize| -> Vec<u64> {
                let Some(at) = engine.skills.kit.position else {
                    return focus.into_iter().collect();
                };
                engine.area_targets(at, radius, cap, focus)
            };
            if let Some(status) = &t.status {
                let targets = match status.to.as_str() {
                    "near" => near(self, 8.0, 8),
                    _ => focus.into_iter().collect(),
                };
                for guid in targets {
                    self.apply_status(guid, &status.id, status.ms, now_ms, out);
                }
            }
            if let Some(strike) = &t.strike {
                let tier = usize::from(table::tier_of_level(level)) - 1;
                let damage = table.class(&strike.class).map_or(0, |l| l[tier]) as f32 * strike.factor.unwrap_or(1.0);
                for guid in near(self, strike.radius, strike.targets) {
                    self.kit_hit(guid, damage.round() as u32, now_ms, out)?;
                }
            }
        }
        Ok(())
    }

    /// Each world tick: statuses that ran out, helpers, and the traits that answer a hit taken.
    pub(in crate::encounter) fn tick_kits(
        &mut self,
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) -> io::Result<()> {
        self.expire_statuses(now_ms, out);
        if !self.kits_live() {
            return Ok(());
        }
        let health = self.state.player_health;
        let lost = self.skills.kit.last_health.map_or(0, |before| before.saturating_sub(health));
        self.skills.kit.last_health = Some(health);
        if lost > 0 && health > 0 {
            let back = self.skills.kit.book.heal_on_damage(ME);
            if back > 0.0 {
                self.kit_heal_self((lost as f32 * back).round() as u32, now_ms, out);
            }
            // The attacker is not passed to this piece: the nearest living enemy stands in.
            let attacker = self
                .skills
                .kit
                .position
                .and_then(|at| self.area_targets(at, 30.0, 1, None).first().copied());
            self.fire_traits("struck", attacker, now_ms, out)?;
        }
        if let Some(mut helpers) = self.skills.kit.helpers.take() {
            self.kit_state.cast = Some(Cast { ability: helpers.ability, effect: None });
            let mut result = Ok(());
            while helpers.next <= now_ms && helpers.next <= helpers.until && result.is_ok() {
                let at = self.skills.kit.position;
                let targets = at.map_or_else(Vec::new, |at| {
                    self.area_targets(at, HELPER_REACH, usize::from(helpers.count), None)
                });
                for i in 0..usize::from(helpers.count) {
                    // Helpers spread over the enemies in reach, and double up when there are few.
                    if let Some(&guid) = targets.get(i % targets.len().max(1)) {
                        result = self.kit_hit(guid, helpers.damage, now_ms, out);
                    }
                }
                helpers.next = helpers.next.saturating_add(helpers.every_ms);
            }
            self.kit_state.cast = None;
            result?;
            if helpers.next <= helpers.until && self.state.player_health > 0 {
                self.skills.kit.helpers = Some(helpers);
            }
        }
        Ok(())
    }
}
