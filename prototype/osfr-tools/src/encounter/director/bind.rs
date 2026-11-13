//! The engine side of a fight: loading the files, turning a roster into engine rows, and
//! carrying out what a run asks for (spawns, the exit door, goal rows, the score, knockouts).
//!
//! A shared run (a group's battle) is advanced once, by the arena's driver
//! (`tick_shared_run`); every member's own step only tells that member's client what changed
//! (`tell`), so goal rows, quest facts and results reach each member once.
use super::fight::*;
use super::run::Life;
use super::*;
use serde_json::json;

/// (R) How near the open exit door a player leaves by walking into it.
const EXIT_REACH: f32 = 3.0;
/// (R) A reach goal's radius when its row gives none.
const REACH_RADIUS: f32 = 5.0;
/// (R) How near a player must be for an escorted actor to keep walking.
const FOLLOW_M: f32 = 8.0;
/// (R) A freed captive's run for the way out: how far, and how long until it is gone.
const FLEE_M: f32 = 6.0;
const FLEE_MS: u64 = 3_000;
/// Results rows (39/47): the client's names (code-string table: 117298, 9005, 117309, 16375,
/// 117302, 117296) and score types as `scorescreen.gfx` `ScoreItem.update` draws them: 0 a
/// count, 1 and 3 count / max, 2 the count as a clock (seconds), 4 the total. The client's own
/// goal rows (`009bb030`) are type 3 as well.
const SCORE_ROWS: [(&str, i32); 6] = [
    ("scoreEnemiesDefeated", 0),
    ("scoreTimeBonus", 2),
    ("scorePlayerKnockouts", 1),
    ("scoreObjectives", 3),
    ("scoreBonusObjectives", 3),
    ("scoreTotalScore", 4),
];

fn near(a: [f32; 3], b: [f32; 3], m: f32) -> bool {
    horizontal_distance(a, b) <= m
}

/// (health, damage, level) of one roster actor for a party of `group`.
fn numbers(fight: &Fight, kind: &KindRow, row: Option<&ActorRow>, group: u8) -> (u32, u32, u32) {
    let level = if kind.level > 0 {
        kind.level
    } else {
        fight.level()
    };
    let level = level.clamp(1, 20);
    let tier = scale::tier_of_level(level as u8);
    // A stated number takes the owner's multiplier here; a rank's has it already.
    let health = (row.and_then(|r| r.max_health).or(kind.max_health))
        .map_or_else(|| scale::enemy_health(kind.rank, tier, group), scale::live_health);
    let damage = (kind.damage).map_or_else(|| scale::enemy_hit(kind.rank, tier), scale::live_hit);
    (health.max(1), damage, level)
}

/// Roster slot `i` as an engine row. Built through the row's own reader, so columns the
/// engine adds later take their defaults.
fn actor_of(fight: &Fight, i: usize, group: u8) -> Option<Actor> {
    let slot = fight.roster.get(i)?;
    let kind = fight.resolved.get(&slot.kind)?;
    let row = fight.actors.get(&slot.key);
    let (health, damage, level) = numbers(fight, kind, row, group);
    serde_json::from_value(json!({
        "guid": fight.guid(Some(i)),
        "definition_id": kind.definition_id.max(0),
        "model_id": kind.model_id,
        "name_id": kind.name_id,
        "name": kind.name,
        "texture_alias": kind.texture_alias,
        "position": slot.spot,
        "max_health": health,
        "damage": damage,
        "level": level,
        "boss": slot.boss,
        // Not in the fight until its wave or a wake brings it in.
        "dormant": !slot.start,
        "rank": slot.rank,
        "tags": slot.tags,
        "scale": row.and_then(|r| r.scale).or(kind.scale),
        "behaviour": row.and_then(|r| r.behaviour.clone()).or(kind.behaviour.clone()),
        "power_up_table": (kind.power_up_table.clone())
            .or_else(|| crate::encounter::pickups::rank_table(slot.rank)),
    }))
    .ok()
}

/// The enemies of a fight as the engine's roster, in roster order.
fn compile(fight: &Fight) -> Vec<Actor> {
    (0..fight.roster.len())
        .filter(|&i| fight.roster[i].side == Side::Enemy)
        .filter_map(|i| actor_of(fight, i, 1))
        .collect()
}

fn exit_door(fight: &Fight) -> Option<Actor> {
    serde_json::from_value(json!({
        "guid": fight.guid(None),
        "definition_id": 0,
        "model_id": EXIT_MODEL_ID,
        "name_id": EXIT_NAME_ID,
        "name": "Exit",
        "texture_alias": "",
        "position": fight.exit_spot(),
        "max_health": 1,
        "damage": 0,
        "boss": false,
    }))
    .ok()
}

/// 39/47: the results card's rows (name, then icon, type, count, max, points).
fn score_packet(instance: i32, id: u32, card: &ScoreCard, primary: u32, bonus: u32) -> Vec<u8> {
    let rows = [
        (card.enemies as i32, -1, card.enemy_points),
        (card.seconds as i32, -1, card.time_bonus),
        (
            card.knockouts_left as i32,
            card.knockouts_allowed as i32,
            card.knockout_points,
        ),
        (card.goals as i32, primary as i32, card.goal_points),
        (card.bonus_goals as i32, bonus as i32, card.bonus_points),
        (-1, -1, card.total),
    ];
    let shown: Vec<_> = SCORE_ROWS
        .iter()
        .zip(rows)
        .filter(|((name, _), _)| bonus > 0 || *name != "scoreBonusObjectives")
        .collect();
    let mut b = minigame(47, instance, id);
    b.extend((shown.len() as i32).to_le_bytes());
    for ((name, kind), (count, max, points)) in shown {
        b.extend((name.len() as i32).to_le_bytes());
        b.extend(name.as_bytes());
        for field in [-1, *kind, count, max, points.min(i32::MAX as u32) as i32] {
            b.extend(field.to_le_bytes());
        }
    }
    b
}

/// Packet 62 with a fight's own values, in the captured order (C16): eleven switches, the
/// game time override, the objective multiplier, then the quick-completion time, value and
/// loss, the points per enemy, the knockout reward and the knockout penalty.
fn settings_packet(fight: &Fight) -> Vec<u8> {
    let (rules, s) = (&fight.rules, &fight.rules.score);
    let mut b = 62_u16.to_le_bytes().to_vec();
    let mut flags = [1_u8; 11];
    flags[2] = u8::from(rules.can_use_abilities);
    flags[3] = u8::from(rules.can_use_consumables);
    b.extend(flags);
    b.extend((-1_i32).to_le_bytes());
    b.extend(s.objective_multiplier.to_le_bytes());
    for v in [
        s.quick_time_s,
        s.quick_value,
        s.quick_loss_per_s,
        s.per_enemy,
        s.per_knockout_left,
        s.knockout_penalty,
    ] {
        b.extend((v.min(i32::MAX as u32) as i32).to_le_bytes());
    }
    b
}

/// 35/18 UpdateLivesRemaining: guid and two ints (client reader 0x008d7120). (R) that they are
/// the knockouts taken and allowed, which is what `Ui.GetPlayerKnockoutInfo` hands the badge.
fn lives_packet(guid: u64, used: u32, allowed: u32) -> Vec<u8> {
    let mut b = header(35, 18);
    b.extend(guid.to_le_bytes());
    b.extend((used as i32).to_le_bytes());
    b.extend((allowed as i32).to_le_bytes());
    b
}

impl EncounterEngine {
    /// Engine load: read `fights/*.json` (a directory without the folder has no fights). A
    /// fight that is not a `field_battles.json` row gets its own arena row here; one that is
    /// takes that row over when a run begins (`prepare_fights`), because the battle file
    /// loads after the engine.
    pub(in crate::encounter) fn load_fights(&mut self, directory: &Path) -> io::Result<()> {
        self.fight_state.pacing = Arc::new(super::super::pacing::PacingTable::load(directory)?);
        let fights = load_dir(directory)?;
        if fights.is_empty() {
            return Ok(());
        }
        let content = Arc::make_mut(&mut self.content);
        let (mut loaded, mut drops) = (BTreeMap::new(), BTreeMap::new());
        for fight in fights {
            let refuse = |why: &str| invalid(&format!("fights/{}.json: {why}", fight.id));
            if content.dungeons.iter().any(|d| d.id == fight.id) {
                return Err(refuse("its id is an encounter's"));
            }
            let wheel: Vec<Prize> = serde_json::from_value(json!(fight.rewards.wheel))
                .map_err(|e| refuse(&format!("/rewards/wheel: {e}")))?;
            let rows: Vec<battle_drops::BattleDrop> =
                serde_json::from_value(json!(fight.rewards.drops))
                    .map_err(|e| refuse(&format!("/rewards/drops: {e}")))?;
            battle_drops::validate(&rows, self.field_rewards.loot_tables())?;
            if !card::wheel_ok(&wheel) {
                return Err(refuse("/rewards/wheel: at most eight well-formed prizes a job"));
            }
            if compile(&fight).len()
                != fight
                    .roster
                    .iter()
                    .filter(|slot| slot.side == Side::Enemy)
                    .count()
            {
                return Err(refuse("an actor does not make an engine row"));
            }
            if !fight.is_enabled() {
                continue;
            }
            if fight.entry.battle_row {
                if !wheel.is_empty() || !rows.is_empty() {
                    return Err(refuse(
                        "/rewards: the field_battles.json row carries the wheel and the drops",
                    ));
                }
            } else if fight.instanced() {
                content.dungeons.push(Dungeon {
                    id: fight.id,
                    title_id: fight.title_id,
                    description_id: fight.description_id,
                    objective_id: goal_text(&fight),
                    icon_id: fight.icon_id.unwrap_or(-1),
                    zone: fight.zone(),
                    guide: None,
                    enemies: compile(&fight),
                    reward_coins: fight.rewards.coins,
                    reward_items: Vec::new(),
                    prizes: wheel,
                    wheel_coins: fight.rewards.wheel_coins,
                });
                drops.insert(fight.id, rows);
            }
            loaded.insert(fight.id, Arc::new(fight));
        }
        self.fight_state.fights = Arc::new(loaded);
        self.fight_state.drops = Arc::new(drops);
        Ok(())
    }

    /// The fights this engine plays, by id.
    pub fn fight_ids(&self) -> Vec<u32> {
        self.fight_state.fights.keys().copied().collect()
    }

    /// The run in progress, if it is a fight file's.
    pub fn fight_run(&self) -> Option<&Run> {
        self.run.run.as_ref().filter(|run| !run.legacy)
    }

    /// Give every fight that is a `field_battles.json` row its roster, arena and goal on that
    /// row. Nothing changes for a row already taken over, or while a run is active. Every
    /// start in `gameplay/field_battle.rs` asks this first.
    pub fn prepare_fights(&mut self) -> io::Result<()> {
        if self.active() || self.fight_state.fights.is_empty() {
            return Ok(());
        }
        let (fights, doors) = (self.fight_state.fights.clone(), self.fight_state.doors.clone());
        let mut changed = false;
        for fight in fights.values().filter(|f| f.entry.battle_row) {
            let Some(at) = self.content.dungeons.iter().position(|d| d.id == fight.id) else {
                continue;
            };
            let enemies = compile(fight);
            let row = &self.content.dungeons[at];
            if row.enemies.len() == enemies.len()
                && row
                    .enemies
                    .iter()
                    .zip(&enemies)
                    .all(|(a, b)| a.guid == b.guid)
            {
                continue;
            }
            for actor in &enemies {
                behaviour::resolve_behaviour(actor, &self.behaviour_presets)
                    .map_err(|e| invalid(&e))?;
            }
            let row = &mut Arc::make_mut(&mut self.content).dungeons[at];
            row.enemies = enemies;
            row.zone = doors.complete(fight.zone());
            if goal_text(fight) > 0 {
                row.objective_id = goal_text(fight);
            }
            if fight.rewards.coins > 0 {
                row.reward_coins = fight.rewards.coins;
            }
            changed = true;
        }
        if changed {
            self.rebuild_behaviours()?;
        }
        Ok(())
    }

    /// Whether the arena row of `fight` holds its roster.
    fn compiled(&self, fight: &Fight) -> bool {
        let first = (0..fight.roster.len())
            .find(|&i| fight.roster[i].side == Side::Enemy)
            .map(|i| fight.guid(Some(i)));
        self.content
            .dungeons
            .iter()
            .find(|d| d.id == fight.id)
            .is_some_and(|d| d.enemies.first().map(|a| a.guid) == first)
    }

    /// The arena finished loading for a new run (not a reconnect into a running one).
    pub(in crate::encounter) fn run_begin(&mut self, now_ms: u64, out: &mut EncounterOutcome) {
        let Some(id) = self.state.active.as_ref().map(|a| a.encounter_id) else {
            return;
        };
        out.facts.push(FightFact::Entered {
            fight: self.fight_state.doors.fight(id),
        });
        self.fight_state.seen = Seen::default();
        self.fight_state.down_since = None;
        self.fight_state.saved = None;
        // A member who joins a group's run finds it in the arena loan.
        let joined =
            self.arena.acting.is_some() && self.run.run.as_ref().is_some_and(|run| run.fight == id);
        if !joined {
            let fight = self.fight_state.fights.get(&id).cloned();
            self.run.run = Some(match fight.filter(|f| self.compiled(f)) {
                Some(fight) => begin(&fight, &self.party()),
                None => Run::legacy(id, now_ms),
            });
        }
        let Some(run) = self.fight_run() else {
            return;
        };
        let def = run.def.clone();
        // What the others already see: the door and the allies.
        let shown: Vec<u64> = run.bind.shown.iter().copied().collect();
        for guid in shown {
            self.send_extra(&def, guid, out);
        }
        if self.held_power_up.is_none() {
            self.held_power_up.clone_from(&def.rules.power_up);
        }
        self.run_step(&[], now_ms, out);
    }

    /// Every engine tick, first thing: also while the player is down, so waves and clocks go on.
    pub(in crate::encounter) fn run_tick(
        &mut self,
        position: [f32; 3],
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) {
        let Some(active) = &self.state.active else {
            // The battle file loads after the engine; a refusal here keeps the old row.
            let _ = self.prepare_fights();
            return;
        };
        if active.phase == EncounterPhase::Fighting && self.armed {
            self.run_step(&[position], now_ms, out);
        }
    }

    /// Zone-ready is about to send the goal rows. `true`: a fight file covers this encounter,
    /// so its own rows go out instead of the engine's single goal: here for a run already in
    /// memory (a scene reload), with the door and the allies it shows; from `run_begin` or
    /// `run_resume` otherwise.
    pub(in crate::encounter) fn run_goals(&self, out: &mut EncounterOutcome) -> bool {
        let Some(id) = self.state.active.as_ref().map(|a| a.encounter_id) else {
            return false;
        };
        let Some(run) = self.run.run.as_ref().filter(|run| run.fight == id) else {
            return (self.fight_state.fights.get(&id)).is_some_and(|fight| self.compiled(fight));
        };
        if run.legacy {
            return false;
        }
        for (row, goal) in run.def.objectives.iter().zip(&run.goals) {
            if row.text_id > 0 && goal.state != GoalState::Waiting {
                let opened = card::goal_row(row.text_id, row.bonus, goal.of as i32);
                out.replies.extend(card::goal_opened(&opened));
                out.replies
                    .extend(card::goal_counted(row.text_id, goal.done as i32));
                if goal.state == GoalState::Done {
                    out.replies.extend(card::goal_met(row.text_id));
                }
            }
        }
        for guid in &run.bind.shown {
            self.send_extra(&run.def, *guid, out);
        }
        true
    }

    /// A relog into a running fight: rebuild the run from the save, or begin it afresh when
    /// the save does not fit the fight any more.
    pub(in crate::encounter) fn run_resume(&mut self, now_ms: u64, out: &mut EncounterOutcome) {
        let Some(active) = self.state.active.as_mut() else {
            return;
        };
        let id = active.encounter_id;
        let saved: Option<super::run::Saved> =
            (active.run_save.take()).and_then(|save| serde_json::from_value(save.0).ok());
        let fight = self.fight_state.fights.get(&id).cloned();
        let run = match fight.filter(|f| self.compiled(f)) {
            Some(fight) => saved
                .and_then(|saved| Run::restore(&fight, &saved, now_ms))
                .unwrap_or_else(|| begin(&fight, &self.party())),
            None => Run::legacy(id, now_ms),
        };
        self.run.run = Some(run);
        self.fight_state.seen = Seen::default();
        self.fight_state.saved = None;
        self.fight_state.down_since = (self.state.player_health == 0).then_some(now_ms);
        self.run_step(&[], now_ms, out);
    }

    fn party(&self) -> Party {
        Party {
            members: vec![Member {
                guid: self.arena.acting.unwrap_or(1),
                profile: u32::try_from(self.player_profile).unwrap_or(0),
                level: self.job_level.min(255) as u8,
            }],
        }
    }

    /// Keep the save in step with a run this engine drives (a shared run is the arena's and
    /// is not saved).
    fn save_run(&mut self, run: &Run, out: &mut EncounterOutcome) {
        if self.arena.acting.is_some() {
            return;
        }
        let saved = run.save();
        if self.fight_state.saved.as_ref() == Some(&saved) {
            return;
        }
        if let (Some(active), Ok(value)) =
            (self.state.active.as_mut(), serde_json::to_value(&saved))
        {
            active.run_save = Some(Box::new(RunSave(value)));
            out.changed = true;
        }
        self.fight_state.saved = Some(saved);
    }

    /// One step of the run for this player: advance it when this engine is the one that
    /// drives it (a solo run), then tell this player's client what changed.
    pub(in crate::encounter) fn run_step(
        &mut self,
        players: &[[f32; 3]],
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) {
        let Some(mut run) = self.run.run.take() else {
            return;
        };
        if run.legacy {
            self.run.run = Some(run);
            return;
        }
        if self.arena.acting.is_none() {
            self.drive(&mut run, players, now_ms, out);
        }
        let recover = u64::from(run.def.rules.recover_s) * 1000;
        let ended = self.tell(&run, out);
        self.save_run(&run, out);
        self.run.run = Some(run);
        match ended {
            Some(true) => self.win_run(now_ms, out),
            Some(false) => self.lose_run(now_ms, out),
            None => {
                // The client's own timer asks to stand up; this is the fallback.
                let down = self.fight_state.down_since;
                if down.is_some_and(|at| now_ms >= at + recover + 2000) {
                    self.stand_up(now_ms, out);
                }
            }
        }
    }

    /// The arena driver's step of a shared run: the same advance a solo engine makes on its
    /// own tick. Returns what every member's client must see.
    pub fn tick_shared_run(&mut self, players: &[FieldParticipant], now_ms: u64) -> Vec<Vec<u8>> {
        let mut out = EncounterOutcome::default();
        if let Some(mut run) = self.run.run.take() {
            if !run.legacy && self.arena.acting.is_some() {
                let spots: Vec<[f32; 3]> = players.iter().map(|p| p.position).collect();
                self.drive(&mut run, &spots, now_ms, &mut out);
            }
            self.run.run = Some(run);
        }
        out.replies
    }

    /// Feed one fact to the run in this player's own action and carry out the answer.
    pub(in crate::encounter) fn run_fact(
        &mut self,
        fact: &Signal,
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) {
        let Some(mut run) = self.run.run.take() else {
            return;
        };
        if !run.legacy {
            self.feed(&mut run, fact, now_ms, out);
            let ended = self.tell(&run, out);
            self.save_run(&run, out);
            self.run.run = Some(run);
            match ended {
                Some(true) => self.win_run(now_ms, out),
                Some(false) => self.lose_run(now_ms, out),
                None => {}
            }
        } else {
            self.run.run = Some(run);
        }
    }

    fn feed(&mut self, run: &mut Run, fact: &Signal, now_ms: u64, out: &mut EncounterOutcome) {
        let mut plain = Vec::new();
        on_fact(run, fact, &mut plain);
        let mut acts: Vec<DirectorAct> = plain
            .into_iter()
            .map(|act| DirectorAct::Actor { guid: 0, act })
            .collect();
        tick(run, now_ms, &mut acts);
        self.apply_director(run, acts, now_ms, out);
    }

    /// Bring the run up to date with the engine (deaths, health, where players stand, wards
    /// and escorts), advance it to `now_ms` and carry out what it asks.
    fn drive(
        &mut self,
        run: &mut Run,
        players: &[[f32; 3]],
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) {
        let def = run.def.clone();
        let mut died = Vec::new();
        if let (Some(d), Some(active)) = (self.dungeon(), self.state.active.as_ref()) {
            for (a, s) in d.enemies.iter().zip(&active.enemies) {
                let Some(i) = run.slot_of(a.guid) else {
                    continue;
                };
                run.set_health(
                    a.guid,
                    (u64::from(s.health) * 100 / u64::from(a.max_health.max(1))) as u32,
                );
                match run.lives[i] {
                    Life::Alive if s.health == 0 => died.push(i),
                    // An enemy's own rule woke it.
                    Life::Waiting if s.health > 0 && !s.dormant => run.lives[i] = Life::Alive,
                    _ => {}
                }
            }
        }
        for i in died {
            let slot = &def.roster[i];
            let fact = Signal::Died {
                guid: def.guid(Some(i)),
                kind: slot.definition_id,
                tags: slot.tags.clone(),
            };
            self.feed(run, &fact, now_ms, out);
        }
        if !players.is_empty() {
            self.places(run, players, now_ms, out);
        }
        let elapsed = now_ms.saturating_sub(*run.bind.clocks.get("drive").unwrap_or(&now_ms));
        run.bind.clocks.insert("drive".into(), now_ms);
        self.wards(run, players, elapsed.min(250), now_ms, out);
        let mut acts = Vec::new();
        tick(run, now_ms, &mut acts);
        self.apply_director(run, acts, now_ms, out);
    }

    /// Tell the run which named points and rooms hold a player, and the open exit door.
    fn places(
        &mut self,
        run: &mut Run,
        players: &[[f32; 3]],
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) {
        let def = run.def.clone();
        let mut places: Vec<(String, bool)> = Vec::new();
        let mut within = |list: &str, m: f32| {
            let inside = def.points.get(list).is_some_and(|points| {
                points
                    .iter()
                    .any(|point| players.iter().any(|p| near(*p, *point, m)))
            });
            places.push((list.to_owned(), inside));
        };
        let triggers =
            (def.waves.iter().map(|w| &w.when)).chain(def.script.iter().map(|r| &r.when));
        for when in triggers {
            if let Trigger::PlayerWithin { point, m } = when {
                within(point, *m);
            }
        }
        for goal in def.objectives.iter().filter(|o| o.kind == GoalKind::Reach) {
            let radius = if goal.radius > 0.0 {
                goal.radius
            } else {
                REACH_RADIUS
            };
            within(goal.target.point.as_deref().unwrap_or_default(), radius);
        }
        for room in &def.place.rooms {
            let b = room.bounds;
            let inside = players
                .iter()
                .any(|p| (0..3).all(|a| p[a] >= b[a] && p[a] <= b[a + 3]));
            places.push((room.id.clone(), inside));
        }
        for (place, inside) in places {
            if inside != run.is_inside(&place) {
                let place = if inside {
                    place
                } else {
                    format!("left:{place}")
                };
                self.feed(run, &Signal::Reached { place }, now_ms, out);
            }
        }
        if run.exit_is_open() {
            let at_door = players
                .iter()
                .any(|p| near(*p, def.exit_spot(), EXIT_REACH));
            if !at_door {
                run.bind.clear_of_exit = true;
            } else if run.bind.clear_of_exit {
                let place = "exit".to_owned();
                self.feed(run, &Signal::Reached { place }, now_ms, out);
            }
        }
    }

    /// Wards lose health to the enemies around them; escorted actors walk their route while a
    /// player keeps close.
    fn wards(
        &mut self,
        run: &mut Run,
        players: &[[f32; 3]],
        elapsed: u64,
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) {
        let def = run.def.clone();
        for (g, goal) in def.objectives.iter().enumerate() {
            if run.goals[g].state != GoalState::Active {
                continue;
            }
            let Some(i) = (0..def.roster.len())
                .find(|&i| goal.target.actor.as_ref() == Some(&def.roster[i].key))
            else {
                continue;
            };
            if run.lives[i] != Life::Alive {
                continue;
            }
            let guid = def.guid(Some(i));
            let died = Signal::Died {
                guid,
                kind: def.roster[i].definition_id,
                tags: def.roster[i].tags.clone(),
            };
            match (goal.kind, &goal.pressure) {
                (GoalKind::Defend, Some(pressure)) => {
                    let clock = format!("pressure:{g}");
                    let last = *run.bind.clocks.entry(clock.clone()).or_insert(now_ms);
                    if now_ms < last + pressure.every_ms {
                        continue;
                    }
                    run.bind.clocks.insert(clock, now_ms);
                    let spot = def.roster[i].spot;
                    let around = self.state.active.as_ref().map_or(0, |active| {
                        active
                            .enemies
                            .iter()
                            .filter(|s| {
                                s.health > 0
                                    && !s.dormant
                                    && near(s.position, spot, pressure.within)
                            })
                            .count() as u32
                    });
                    if around == 0 {
                        continue;
                    }
                    let max = def.resolved.get(&def.roster[i].kind).map_or(1, |kind| {
                        numbers(&def, kind, def.actors.get(&def.roster[i].key), 1).0
                    });
                    let lost = (u64::from(around.saturating_mul(pressure.damage)) * 100)
                        .div_ceil(u64::from(max));
                    let left = u64::from(run.health[i]).saturating_sub(lost.max(1)) as u32;
                    run.set_health(guid, left);
                    out.replies.extend(enemy_health(
                        guid,
                        (u64::from(max) * u64::from(left) / 100) as u32,
                        max,
                    ));
                    if left == 0 {
                        self.feed(run, &died, now_ms, out);
                    }
                }
                (GoalKind::Escort, _) => {
                    let route = goal
                        .route
                        .as_ref()
                        .and_then(|r| def.points.get(r))
                        .cloned()
                        .unwrap_or_default();
                    let (mut at, mut leg) =
                        *run.bind.spots.entry(i).or_insert((def.roster[i].spot, 0));
                    let follow = if goal.follow_m > 0.0 {
                        goal.follow_m
                    } else {
                        FOLLOW_M
                    };
                    let Some(to) = route.get(leg).copied() else {
                        continue;
                    };
                    if !players.iter().any(|p| near(*p, at, follow)) {
                        continue;
                    }
                    let gap = horizontal_distance(at, to);
                    let step = goal.speed * elapsed as f32 / 1000.0;
                    if gap <= step.max(0.05) {
                        at = to;
                        leg += 1;
                    } else {
                        for a in 0..3 {
                            at[a] += (to[a] - at[a]) / gap * step;
                        }
                    }
                    self.ground().snap(&mut at);
                    run.bind.spots.insert(i, (at, leg));
                    let heading = [
                        (to[0] - at[0]) / gap.max(0.01),
                        0.0,
                        (to[2] - at[2]) / gap.max(0.01),
                    ];
                    out.replies.push(movement(guid, at, heading));
                    if leg >= route.len() {
                        let arrived = Signal::Named(format!("arrived:{}", def.roster[i].key));
                        self.feed(run, &arrived, now_ms, out);
                    }
                }
                _ => {}
            }
        }
    }

    fn apply_director(
        &mut self,
        run: &mut Run,
        acts: Vec<DirectorAct>,
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) {
        let def = run.def.clone();
        for act in acts {
            match act {
                DirectorAct::OpenExit => {
                    let guid = def.guid(None);
                    if run.bind.shown.insert(guid) {
                        self.send_extra(&def, guid, out);
                    }
                }
                DirectorAct::Actor { guid: 0, act } => match act {
                    Act::Say(line) => out.replies.push(wire::say(0, &line)),
                    Act::Signal(name) => self.run.signals.push(name),
                    Act::Drop(table) => {
                        let at = def.place.arrive;
                        self.drop_power_ups(&table, (def.guid(None), now_ms), at, now_ms, 1, out);
                    }
                    // A wake by tag shows below; the rest need an actor.
                    _ => {}
                },
                DirectorAct::Actor { guid, act } => self.apply_act(guid, &act, now_ms, out),
                DirectorAct::Signal(name) => self.run.signals.push(name),
                DirectorAct::Fact(FightFact::Freed { kind, guid }) => {
                    out.facts.push(FightFact::Freed { kind, guid });
                }
                // Waves show below; goals and the end reach every member through `tell`.
                DirectorAct::Wave(_)
                | DirectorAct::Fact(_)
                | DirectorAct::Win
                | DirectorAct::Lose => {}
            }
        }
        self.show(run, now_ms, out);
    }

    /// Send the exit door or one ally to this player's client.
    fn send_extra(&self, def: &Fight, guid: u64, out: &mut EncounterOutcome) {
        let ground = self.ground();
        if guid == def.guid(None) {
            if let Some(door) = exit_door(def) {
                // As captured: added hostile, then made friendly at once.
                out.replies
                    .push(self.spawn_object(&door, ground.snapped(door.position), 0));
                out.replies.push(wire::disposition(guid, 1));
            }
        } else if let Some(i) = (0..def.roster.len()).find(|&i| def.guid(Some(i)) == guid) {
            if let Some(ally) = actor_of(def, i, 1) {
                out.replies
                    .push(self.spawn(&ally, ground.snapped(ally.position), 1));
                if let Some(interact) = def.interact(i) {
                    out.replies.push(wire::hover(guid, interact.cursor));
                }
            }
        }
    }

    /// Put on screen every actor the run holds alive that the client has not been sent, and
    /// take away the allies that are gone.
    fn show(&mut self, run: &mut Run, now_ms: u64, out: &mut EncounterOutcome) {
        let def = run.def.clone();
        let ground = self.ground();
        let wake: Vec<u64> = self.state.active.as_ref().map_or_else(Vec::new, |active| {
            active
                .enemies
                .iter()
                .filter(|s| s.dormant && s.health > 0 && run.life(s.guid) == Some(Life::Alive))
                .map(|s| s.guid)
                .collect()
        });
        if !wake.is_empty() {
            let actors: Vec<Actor> = self.dungeon().map_or_else(Vec::new, |d| {
                d.enemies
                    .iter()
                    .filter(|a| wake.contains(&a.guid))
                    .cloned()
                    .collect()
            });
            for a in &actors {
                let at = ground.snapped(a.position);
                if let Some(s) = self
                    .state
                    .active
                    .as_mut()
                    .and_then(|active| active.enemies.iter_mut().find(|s| s.guid == a.guid))
                {
                    s.dormant = false;
                    s.position = at;
                }
                out.replies.push(self.spawn(a, at, 0));
                out.replies
                    .extend(enemy_health(a.guid, a.max_health, a.max_health));
                if a.boss {
                    out.replies.push(boss_display(a.guid, true));
                }
            }
            out.replies.push(relevance(&wake, false));
            out.changed = true;
            self.enemies_spawned(&wake, now_ms, out);
            for guid in &wake {
                if let Some(memory) = self.arena.brains.get_mut(guid) {
                    memory.alert = true;
                }
            }
        }
        if run.exit_is_open() && run.bind.shown.insert(def.guid(None)) {
            self.send_extra(&def, def.guid(None), out);
        }
        for i in (0..def.roster.len()).filter(|&i| def.roster[i].side == Side::Ally) {
            let guid = def.guid(Some(i));
            if run.lives[i] == Life::Alive {
                if run.bind.shown.insert(guid) {
                    self.send_extra(&def, guid, out);
                }
            } else if run.bind.shown.contains(&guid) {
                // (R) A freed captive runs a few steps for the way in and is gone FLEE_MS later
                // (the guide: "tell them it is safe to leave"); any other ally goes at once.
                let since = *run.bind.clocks.entry(format!("fled:{i}")).or_insert(now_ms);
                let gone = run.lives[i] == Life::Gone;
                if gone && since == now_ms {
                    let (from, to) = (def.roster[i].spot, def.place.arrive);
                    let gap = horizontal_distance(from, to).max(0.01);
                    let way = [(to[0] - from[0]) / gap, 0.0, (to[2] - from[2]) / gap];
                    let step = FLEE_M.min(gap);
                    let at = [from[0] + way[0] * step, from[1], from[2] + way[2] * step];
                    out.replies.push(movement(guid, at, way));
                }
                if !gone || now_ms >= since + FLEE_MS {
                    run.bind.shown.remove(&guid);
                    out.replies.push(remove(guid));
                }
            }
        }
    }

    /// Tell this player's client what changed in the run since it was last told: goal rows,
    /// the facts quests read, the knockout badge. Returns how the run ended, when this
    /// player is still fighting in it.
    fn tell(&mut self, run: &Run, out: &mut EncounterOutcome) -> Option<bool> {
        let def = &run.def;
        let me = self.arena.acting.unwrap_or(1);
        let seen = &mut self.fight_state.seen;
        if !seen.settings {
            seen.settings = true;
            if def.rules.wire.settings {
                out.replies.push(settings_packet(def));
            }
            if def.rules.wire.lives {
                out.replies
                    .push(lives_packet(me, run.knockouts, def.knockouts()));
            }
        }
        seen.goals
            .resize(run.goals.len(), (GoalState::Waiting, 0, 0));
        for (i, (row, goal)) in def.objectives.iter().zip(&run.goals).enumerate() {
            let (now, was) = ((goal.state, goal.done, goal.of), seen.goals[i]);
            if now == was {
                continue;
            }
            seen.goals[i] = now;
            if goal.state == GoalState::Waiting {
                continue;
            }
            out.changed = true;
            out.facts.push(FightFact::Goal {
                fight: run.fight,
                goal: row.id.clone(),
                done: goal.done,
                of: goal.of,
            });
            if row.text_id > 0 {
                if was.0 == GoalState::Waiting {
                    let opened = card::goal_row(row.text_id, row.bonus, goal.of as i32);
                    out.replies.extend(card::goal_opened(&opened));
                }
                out.replies
                    .extend(card::goal_counted(row.text_id, goal.done as i32));
                if goal.state == GoalState::Done {
                    out.replies.extend(card::goal_met(row.text_id));
                }
            }
        }
        if seen.knockouts != run.knockouts {
            seen.knockouts = run.knockouts;
            if def.rules.wire.lives {
                out.replies
                    .push(lives_packet(me, run.knockouts, def.knockouts()));
            }
        }
        let fighting = self
            .state
            .active
            .as_ref()
            .is_some_and(|a| a.phase == EncounterPhase::Fighting);
        run.outcome().filter(|_| fighting)
    }

    /// The client's leave (41/109). With the exit door open the fight is won and only the walk
    /// to the door is left, so leaving is going out through it (R): the win is paid and
    /// counted before the player goes home, never lost to the menu. The client has already
    /// closed its minigame when it sends the leave (native log "MiniGame:LeaveMiniGame"), so
    /// the results card it would answer is not shown.
    pub(in crate::encounter) fn leave(&mut self, now_ms: u64) -> io::Result<EncounterOutcome> {
        let mut out = EncounterOutcome::default();
        if self.fight_run().is_some_and(Run::exit_is_open) {
            let exit = Signal::Reached {
                place: "exit".to_owned(),
            };
            self.run_fact(&exit, now_ms, &mut out);
        }
        let left = self.exit()?;
        out.replies.extend(left.replies);
        out.changed |= left.changed;
        out.zoning = left.zoning;
        out.completed = left.completed;
        Ok(out)
    }

    /// A player talked to (clicked) an actor of the fight: the open exit door ends the run,
    /// a captive whose guards are down goes free, and a `talked` rule may answer.
    pub(in crate::encounter) fn actor_talked(
        &mut self,
        guid: u64,
        position: [f32; 3],
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) {
        let Some(run) = self.fight_run() else {
            return;
        };
        let def = run.def.clone();
        let at = if guid == def.guid(None) {
            Some(def.exit_spot())
        } else {
            run.slot_of(guid).map(|i| def.roster[i].spot)
        };
        // An actor whose `interact` is switched off answers no click.
        let off = |i: usize| def.roster[i].interact.as_ref().is_some_and(|x| !x.enabled);
        if run.slot_of(guid).is_some_and(off) {
            return;
        }
        // The door and the captives stand still: the click must come from beside them.
        let present = run.bind.shown.contains(&guid) || run.life(guid) == Some(Life::Alive);
        if !present || !at.is_some_and(|at| near(position, at, 12.0)) {
            return;
        }
        self.run_fact(&Signal::Talked { guid }, now_ms, out);
    }

    /// An enemy of the run reached zero health in this player's action. `true`: the run owns
    /// goal progress and the win, so the engine's single-goal lines stay silent.
    pub(in crate::encounter) fn run_kill(
        &mut self,
        _guid: u64,
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) -> bool {
        let Some(mut run) = self.run.run.take() else {
            return false;
        };
        if run.legacy {
            self.run.run = Some(run);
            return false;
        }
        // The same reading of the engine the tick makes, at once, so the kill's own answer
        // (the goal row, the next wave, the door) rides with the hit.
        self.drive(&mut run, &[], now_ms, out);
        let ended = self.tell(&run, out);
        self.save_run(&run, out);
        self.run.run = Some(run);
        match ended {
            Some(true) => self.win_run(now_ms, out),
            Some(false) => self.lose_run(now_ms, out),
            None => {}
        }
        true
    }

    /// This player was knocked out inside the run. `true`: the group still has knockouts
    /// left, so the player waits and stands up again; `false`: the fight is lost (or the run
    /// is a row with no fight file, where the first knockout always was).
    pub(in crate::encounter) fn player_down(
        &mut self,
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) -> bool {
        let Some(mut run) = self.run.run.take() else {
            return false;
        };
        if run.legacy {
            self.run.run = Some(run);
            return false;
        }
        let guid = self.arena.acting.unwrap_or(1);
        self.feed(&mut run, &Signal::PlayerDown { guid }, now_ms, out);
        let (lost, left, recover) = (
            run.outcome() == Some(false),
            run.knockouts_left(),
            run.def.rules.recover_s,
        );
        let _ = self.tell(&run, out);
        self.save_run(&run, out);
        self.run.run = Some(run);
        if lost {
            return false;
        }
        self.fight_state.down_since = Some(now_ms);
        out.replies
            .push(wire::respawn_window((recover * 1000) as i32, 0));
        out.feedback = Some(format!("Knocked out. Knockouts remaining: {left}."));
        true
    }

    /// Whether this player is down inside a run that goes on.
    pub(in crate::encounter) fn down_in_fight(&self) -> bool {
        self.fight_state.down_since.is_some()
            && self.state.player_health == 0
            && self
                .state
                .active
                .as_ref()
                .is_some_and(|a| a.phase == EncounterPhase::Fighting)
    }

    /// Back on their feet where they fell, at full health (wiki combat guide), and out of the
    /// enemies' reach for the fight grace (owner 2026-11-13).
    pub(in crate::encounter) fn stand_up(&mut self, now_ms: u64, out: &mut EncounterOutcome) {
        if !self.down_in_fight() {
            return;
        }
        self.fight_state.down_since = None;
        self.fight_state.guarded_until = now_ms.saturating_add(self.rules.revive.fight_grace_ms);
        self.state.player_health = self.max_health;
        out.changed = true;
        out.replies
            .push(player_hp(self.max_health, self.max_health));
        out.replies.push(respawn_hide());
    }

    /// The results card of a fight run, in place of the engine's two fixed rows.
    pub(in crate::encounter) fn fight_results(&mut self, now_ms: u64, out: &mut EncounterOutcome) {
        let (Some(run), Some(active)) = (self.fight_run(), self.state.active.as_ref()) else {
            return;
        };
        if self.results_presented {
            return;
        }
        if active.phase == EncounterPhase::Lost {
            if let Some(d) = self.dungeon() {
                for (actor, state) in d.enemies.iter().zip(&active.enemies) {
                    if actor.boss && state.health > 0 && !state.dormant {
                        out.replies.push(boss_display(actor.guid, false));
                    }
                }
            }
        }
        let count = |bonus: bool| {
            run.def
                .objectives
                .iter()
                .filter(|o| o.bonus == bonus)
                .count()
        };
        out.replies.push(score_packet(
            active.instance_id,
            active.encounter_id,
            &run.score(now_ms),
            count(false) as u32,
            count(true) as u32,
        ));
        out.replies
            .push(minigame(19, active.instance_id, active.encounter_id));
        self.results_presented = true;
    }

    /// The results card of a row with no fight file (a `field_battles.json` battle, the cave):
    /// the rows and arithmetic of a fight file's card, with the captured battle record's values
    /// (packet 62, C16). Such a run is lost at its first knockout, so a won one has its one left.
    pub(in crate::encounter) fn results_card(&self, active: &ActiveEncounter, now_ms: u64) -> Vec<u8> {
        let won = active.phase == EncounterPhase::Won;
        let started = (self.run.run.as_ref()).filter(|run| run.legacy).and_then(|run| run.started);
        let card = ScoreCard {
            enemies: active.enemies.iter().filter(|e| e.health == 0).count() as u32,
            seconds: started.map_or(0, |at| (now_ms.saturating_sub(at) / 1000) as u32),
            knockouts_left: u32::from(won),
            knockouts_allowed: 1,
            goals: u32::from(won),
            ..ScoreCard::default()
        }
        .scored(&Score::from(&self.wire_data().settings.battle), won);
        score_packet(active.instance_id, active.encounter_id, &card, 1, 0)
    }

    /// The run was just won or lost (the results follow).
    pub(in crate::encounter) fn run_end(
        &mut self,
        _won: bool,
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) {
        self.fight_state.down_since = None;
        self.fight_results(now_ms, out);
    }

    /// The player left the run (exit) or it starts over (retry).
    pub(in crate::encounter) fn run_closed(&mut self) {
        self.run = RunSlot::default();
        self.fight_state.seen = Seen::default();
        self.fight_state.down_since = None;
        self.fight_state.saved = None;
        if let Some(active) = &mut self.state.active {
            active.run_save = None;
        }
    }

    /// Kill-fact fields this piece knows: the actor's tags and rank in its fight.
    pub(in crate::encounter) fn stamp_kill_run(&self, fact: &mut KillFact) {
        if let Some((run, i)) = self
            .fight_run()
            .and_then(|run| Some((run, run.slot_of(fact.guid)?)))
        {
            let slot = &run.def.roster[i];
            fact.tags.clone_from(&slot.tags);
            fact.rank = slot.rank;
        }
    }

    /// What a won fight pays on top of its arena row: (job XP, bonus purse when every bonus
    /// goal is done). `None` for a row with no fight file.
    pub(in crate::encounter) fn fight_pay(&self) -> Option<(u32, Purse)> {
        let run = self.fight_run()?;
        let all_bonus = run
            .def
            .objectives
            .iter()
            .zip(&run.goals)
            .all(|(o, g)| !o.bonus || g.state == GoalState::Done);
        let bonus = if all_bonus {
            run.def.rewards.bonus.clone()
        } else {
            Purse::default()
        };
        Some((run.def.rewards.job_xp.unwrap_or(BATTLE_JOB_XP), bonus))
    }

    /// Living actors per tag in the run, for an enemy's rules (`brain::View::tags_alive`).
    #[allow(dead_code)] // for the brain's view
    pub(in crate::encounter) fn fight_tags_alive(&self) -> BTreeMap<String, u32> {
        self.fight_run().map(Run::tags_alive).unwrap_or_default()
    }

    /// A rule of an enemy raised a signal: the fight's script hears it.
    #[allow(dead_code)] // for the brain's `signal` act
    pub(in crate::encounter) fn fight_signal(
        &mut self,
        name: &str,
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) {
        self.run_fact(&Signal::Named(name.to_owned()), now_ms, out);
    }
}

/// The goal text of the start screen and the engine's row: the first primary goal's.
fn goal_text(fight: &Fight) -> i32 {
    fight
        .objectives
        .iter()
        .find(|o| !o.bonus)
        .map_or(0, |o| o.text_id)
}
