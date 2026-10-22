//! Crafting and gathering: the five movie gate, cancel and tick paths and
//! the server-side appliers for their staged outcomes.
use super::movie::{Movie, elapsed_if_due};
use super::*;

impl Gameplay {
    /// The running movie, first in the gate's intercept order. At most one can
    /// be active; the order is what the five separate blocks used to encode.
    pub(super) fn running_movie(&self) -> Option<Movie> {
        Movie::ALL
            .into_iter()
            .find(|movie| self.is_occupied(movie.occupation()))
    }

    /// Cancels the running movie: its engine's `cancel`, its applier, then the
    /// clock reset exactly as the five blocks did. Cooking and smelting have no
    /// tick clock.
    pub(super) fn cancel_movie(
        &mut self,
        movie: Movie,
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<()> {
        match movie {
            Movie::Forging => {
                let result = self.forging.as_mut().unwrap().cancel();
                self.apply_craft(CraftingJob::Blacksmith, result, character, out)?;
                self.forging_tick_ms = None;
            }
            Movie::Cooking => {
                let result = self.cooking.as_mut().unwrap().cancel();
                self.apply_craft(CraftingJob::Chef, result, character, out)?;
            }
            Movie::Smelting => {
                let result = self.smelting.as_mut().unwrap().cancel();
                self.apply_craft(CraftingJob::Miner, result, character, out)?;
            }
            Movie::Mining => {
                let result = self.type8_mut(Gather::Mining).unwrap().cancel();
                self.apply_type8(Gather::Mining, result, out)?;
                self.type8_tick_ms = None;
            }
            Movie::Harvesting => {
                let result = self.type8_mut(Gather::Harvesting).unwrap().cancel();
                self.apply_type8(Gather::Harvesting, result, out)?;
                self.type8_tick_ms = None;
            }
        }
        Ok(())
    }

    /// Routes a 39/x minigame frame to the running movie's engine and applies
    /// the outcome.
    pub(super) fn route_movie(
        &mut self,
        movie: Movie,
        inner: &[u8],
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<()> {
        match movie {
            Movie::Forging => {
                let result = self.forging.as_mut().unwrap().handle(inner)?;
                self.apply_craft(CraftingJob::Blacksmith, result, character, out)
            }
            Movie::Cooking => {
                let result = self.cooking.as_mut().unwrap().handle(inner)?;
                self.apply_craft(CraftingJob::Chef, result, character, out)
            }
            Movie::Smelting => {
                let result = self.smelting.as_mut().unwrap().handle(inner)?;
                self.apply_craft(CraftingJob::Miner, result, character, out)
            }
            Movie::Mining => {
                let result = self.type8_mut(Gather::Mining).unwrap().handle(inner)?;
                self.apply_type8(Gather::Mining, result, out)
            }
            Movie::Harvesting => {
                let result = self.type8_mut(Gather::Harvesting).unwrap().handle(inner)?;
                self.apply_type8(Gather::Harvesting, result, out)
            }
        }
    }

    /// One due Type8 tick; the clock is stamped only after the tick and its
    /// applier succeed.
    fn tick_gather(&mut self, gather: Gather, now_ms: u64, out: &mut Outcome) -> io::Result<()> {
        if let Some(elapsed) = elapsed_if_due(self.type8_tick_ms, now_ms) {
            let result = self.type8_mut(gather).unwrap().tick(elapsed)?;
            self.apply_type8(gather, result, out)?;
            self.type8_tick_ms = Some(now_ms);
        }
        Ok(())
    }

    /// Ticks the running movie once its 250 ms step is due. Cooking and
    /// smelting run on client input alone, so they only end the world tick.
    pub(super) fn movie_tick(
        &mut self,
        movie: Movie,
        character: &CharacterState,
        now_ms: u64,
        out: &mut Outcome,
    ) -> io::Result<()> {
        match movie {
            Movie::Forging => {
                if let Some(elapsed) = elapsed_if_due(self.forging_tick_ms, now_ms) {
                    let result = self.forging.as_mut().unwrap().tick(elapsed)?;
                    self.apply_craft(CraftingJob::Blacksmith, result, character, out)?;
                    self.forging_tick_ms = Some(now_ms);
                }
            }
            Movie::Mining => self.tick_gather(Gather::Mining, now_ms, out)?,
            Movie::Harvesting => self.tick_gather(Gather::Harvesting, now_ms, out)?,
            Movie::Cooking | Movie::Smelting => {}
        }
        Ok(())
    }

    /// One opening craft start: job, pending objective, exact inputs, begin,
    /// forging clock, applier and note. A refusal answers one chat line.
    pub(super) fn start_tutorial_craft(
        &mut self,
        job: CraftingJob,
        character: &CharacterState,
        now_ms: u64,
        out: &mut Outcome,
    ) -> io::Result<()> {
        struct Spec {
            switch: &'static str,
            pending: &'static str,
            shortage: &'static str,
            note: Option<&'static str>,
        }
        let spec = match job {
            CraftingJob::Chef => Spec {
                switch: "Switch to Chef before cooking.",
                pending: "Accept the cooking objective and finish its earlier steps first.",
                shortage: "StrongArm Stew requires 4 Bat Meat, 4 Gummyfruit, 4 Rainleaf and 4 Spice.",
                note: None,
            },
            CraftingJob::Blacksmith => Spec {
                switch: "Switch to Blacksmith before forging.",
                pending: "Accept the forging objective and finish its earlier steps first.",
                shortage: "Forging requires 10 Copper Bars, 5 Turquoise, 5 Wooden Planks and 5 Thorny Fasteners.",
                note: Some(
                    "Original Spunky Scrapper Hammer controls; board tuning, consumption and tint choice are reconstructed.",
                ),
            },
            CraftingJob::Miner => Spec {
                switch: "Switch to Miner before smelting.",
                pending: "Accept the smelting objective and finish its earlier steps first.",
                shortage: "Smelting requires 5 Copper Ore and 3 Singing Coal. Mine the accepted nearby ore objective first.",
                note: Some(
                    "Original four-stage smelting controls; phase tuning and this local recipe output are reconstructed.",
                ),
            },
        };
        let profile = job.profile_id();
        if self.profiles.as_ref().unwrap().active() != profile {
            out.replies.push(chat::system(spec.switch));
            return Ok(());
        }
        if !self.opening_craft_pending(job.opening_recipe_id()) {
            out.replies.push(chat::system(spec.pending));
            return Ok(());
        }
        let missing = match job {
            CraftingJob::Chef => self.cooking.as_ref().unwrap().requirements(),
            CraftingJob::Blacksmith => self.forging.as_ref().unwrap().requirements(),
            CraftingJob::Miner => self.smelting.as_ref().unwrap().requirements(),
        }
        .iter()
        .any(|(id, n)| self.material_count(*id) < u64::from(*n));
        if missing {
            out.replies.push(chat::system(spec.shortage));
            return Ok(());
        }
        let key = match job {
            CraftingJob::Chef => self.cooking.as_ref().unwrap().recipe_key(),
            CraftingJob::Blacksmith => self.forging.as_ref().unwrap().recipe_key(),
            CraftingJob::Miner => self.smelting.as_ref().unwrap().recipe_key(),
        }
        .to_owned();
        let rank = self.recipe_rank(profile);
        let learned = self.learned_recipe_for_key(&key);
        let available = self.available_ingredients();
        let result = match job {
            CraftingJob::Chef => self
                .cooking
                .as_mut()
                .unwrap()
                .begin_recipe(profile, &key, rank, learned, &available)?,
            CraftingJob::Blacksmith => self
                .forging
                .as_mut()
                .unwrap()
                .begin_recipe(profile, &key, rank, learned, &available)?,
            CraftingJob::Miner => self
                .smelting
                .as_mut()
                .unwrap()
                .begin_recipe(profile, &key, rank, learned, &available)?,
        };
        if job == CraftingJob::Blacksmith {
            self.forging_tick_ms = Some(now_ms);
        }
        self.apply_craft(job, result, character, out)?;
        if let Some(note) = spec.note {
            out.replies.push(chat::system(note));
        }
        Ok(())
    }

    fn opening_craft_pending(&self, recipe: u32) -> bool {
        self.opening.as_ref().is_some_and(|opening| {
            opening.active_stage().is_some_and(|stage| {
                let progress = &opening.state().stages[&stage.id];
                progress.accepted
                    && stage
                        .objectives
                        .iter()
                        .zip(&progress.counts)
                        .find(|(goal, count)| **count < goal.required)
                        .is_some_and(|(goal, _)| {
                            matches!(goal.trigger,
                        crate::opening::OpeningTrigger::Craft { recipe_id } if recipe_id == recipe)
                        })
            })
        })
    }

    /// Client 26/8 on a crafting station opens the recipe window for its job, or
    /// answers the 26/24 wrong-job / no-station line (`recipe_book::open_window`).
    /// Read-only, so a refusal changes nothing. Other targets and busy players
    /// fall through to the NPC, opening and movie paths.
    pub(super) fn station_interaction_26_8(
        &self,
        inner: &[u8],
        character: &CharacterState,
    ) -> io::Result<Option<Outcome>> {
        if !inner.starts_with(&[26, 0, 8, 0]) {
            return Ok(None);
        }
        let target = npc_interaction::decode_npc_interact_26_8(inner)
            .map_err(|_| invalid("invalid NPC interaction request"))?
            .target;
        let (Some(job), Some(profiles)) = (self.station_job(target), self.profiles.as_ref()) else {
            return Ok(None);
        };
        if self.activity_busy() {
            return Ok(None);
        }
        Ok(Some(Outcome {
            replies: vec![crate::recipe_book::open_window(
                job,
                profiles.active(),
                character.zone_id,
                character.position,
                &self.crafting_stations(),
            )],
            ..Outcome::default()
        }))
    }

    /// (R) The job of a station actor: a world model `crafting_stations.json`
    /// binds, or an opening craft actor once its tutorial craft is not pending
    /// (until then its menu starts the tutorial).
    fn station_job(&self, guid: u64) -> Option<CraftingJob> {
        let opening = self.opening.as_ref();
        if let Some(actor) = opening.and_then(|o| o.actors().iter().find(|a| a.guid == guid)) {
            let Some(OpeningTrigger::Craft { recipe_id }) = actor.action else {
                return None;
            };
            return crate::recipe_book::opening_job(recipe_id)
                .filter(|_| !self.opening_craft_pending(recipe_id));
        }
        let model = self.npc_visibility.as_ref()?.model_of(guid)?;
        self.world_station_rules
            .iter()
            .find(|rule| rule.model_id == model)
            .map(|rule| rule.job)
    }

    pub(super) fn near_craft_station(&self, character: &CharacterState, job: CraftingJob) -> bool {
        crate::recipe_book::near_any(
            &self.crafting_stations(),
            job,
            character.zone_id,
            character.position,
        )
    }

    /// Resolve the opening stations and every currently placed station model.
    /// Deriving this view at use keeps placement reloads and removals authoritative.
    pub(super) fn crafting_stations(&self) -> Vec<crate::recipe_book::Station> {
        let mut stations = self
            .opening
            .as_ref()
            .map(|engine| crate::recipe_book::stations(engine.actors()))
            .unwrap_or_default();
        if let Some(visibility) = &self.npc_visibility {
            crate::recipe_book::merge_stations(
                &mut stations,
                crate::recipe_book::world_stations(
                    &self.world_station_rules,
                    visibility.model_actors(),
                ),
            );
        }
        stations
    }

    pub(super) fn gather_count(&self, resource: u32) -> u32 {
        let opening = self.opening.as_ref().unwrap();
        opening.active_stage().map(|stage| {
            stage.objectives.iter().zip(&opening.state().stages[&stage.id].counts)
                .filter(|(goal, _)| matches!(goal.trigger,
                    crate::opening::OpeningTrigger::Gather { resource_id } if resource_id == resource))
                .map(|(_, count)| *count).sum()
        }).unwrap_or(0)
    }

    pub(super) fn material_count(&self, definition: i32) -> u64 {
        self.inventory
            .state()
            .items
            .iter()
            .filter(|item| item.definition_id == definition && item.rental.is_none())
            .map(|item| u64::from(item.count))
            .sum()
    }

    /// One completion for the three crafting movies. The recipe key is read
    /// only after a transaction arrives, and only the Miner tutorial counts the
    /// bars it grants.
    pub(super) fn apply_craft(
        &mut self,
        job: CraftingJob,
        result: CraftOutcome,
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<()> {
        if let Some(transaction) = result.transaction {
            // (R) A recipe-book run has no tutorial objective; the profile and
            // station checks still bind the completion.
            if self.profiles.as_ref().unwrap().active() != job.profile_id()
                || !self.near_craft_station(character, job)
            {
                return Err(invalid(format!(
                    "{} completion is outside a {} station",
                    job.noun(),
                    job.name()
                )));
            }
            if transaction
                .consumes
                .iter()
                .any(|(id, n)| self.material_count(*id) < u64::from(*n))
            {
                return Err(invalid(format!(
                    "{} ingredients changed before completion",
                    job.noun()
                )));
            }
            self.consume_materials(transaction.consumes, out)?;
            self.grant_reward(0, &transaction.grants, out)?;
            // (R) Only the Miner tutorial counts its Copper Bar (21785) grants;
            // cooking and forging advance their objective by one completion.
            let quantity: u32 = if job == CraftingJob::Miner {
                transaction
                    .grants
                    .iter()
                    .filter(|(id, _)| *id == 21785)
                    .map(|(_, n)| *n)
                    .sum()
            } else {
                1
            };
            let opening_recipe = job.opening_recipe_id();
            if self.opening_craft_pending(opening_recipe) {
                let progress = self
                    .opening
                    .as_mut()
                    .unwrap()
                    .on_event(OpeningEvent::Craft {
                        recipe_id: opening_recipe,
                        quantity,
                    })?;
                self.apply_opening(progress, out)?;
            }
            // (R) The opening binding is the fallback for the unnumbered sample.
            let recipe_id = self
                .recipe_catalog
                .by_key(match job {
                    CraftingJob::Chef => self.cooking.as_ref().unwrap().recipe_key(),
                    CraftingJob::Miner => self.smelting.as_ref().unwrap().recipe_key(),
                    CraftingJob::Blacksmith => self.forging.as_ref().unwrap().recipe_key(),
                })
                .and_then(|r| r.recipe_id)
                .unwrap_or(opening_recipe);
            self.npc_quest_event(
                crate::quest_engine::QuestEvent::CraftRecipe {
                    recipe_id,
                    quantity: 1,
                },
                out,
            )?;
            self.npc_minigame_win(&result.replies, out)?;
        }
        out.changed |= result.changed;
        out.replies.extend(result.replies);
        Ok(())
    }

    /// The Type8 engine for one gather game. Mining and Harvesting keep
    /// separate runs; `Gather` is the only key their shared paths need.
    pub(super) fn type8(&self, gather: Gather) -> Option<&Type8Engine> {
        match gather {
            Gather::Mining => self.mining.as_ref(),
            Gather::Harvesting => self.harvesting.as_ref(),
        }
    }

    pub(super) fn type8_mut(&mut self, gather: Gather) -> Option<&mut Type8Engine> {
        match gather {
            Gather::Mining => self.mining.as_mut(),
            Gather::Harvesting => self.harvesting.as_mut(),
        }
    }

    /// Stage a TYPE8 outcome (Mining18 or Harvesting111). The only reward path is
    /// the engine's server-side transaction derived from the configured
    /// `outputs` and the collected quota; client score or win reports never
    /// reach it. Grants commit inside the staged gameplay clone, which the
    /// gateway saves before sending.
    pub(super) fn apply_type8(
        &mut self,
        gather: Gather,
        result: Type8Outcome,
        out: &mut Outcome,
    ) -> io::Result<()> {
        if let Some(transaction) = result.transaction {
            let grant = match gather {
                Gather::Mining => crate::mining::lifecycle::reward_grant(Some(&transaction), 0)?,
                Gather::Harvesting => crate::harvesting::lifecycle::reward_grant(
                    Some(&transaction),
                    HARVESTING_GAME_ID,
                    0,
                )?,
            };
            if self.profiles.as_ref().map(Profiles::active) != Some(gather.job()) {
                return Err(invalid(match gather {
                    Gather::Mining => "mining completion is outside the accepted Miner job",
                    Gather::Harvesting => "harvesting completion is outside the accepted Chef job",
                }));
            }
            self.consume_materials(transaction.consumes, out)?;
            let grants: Vec<_> = grant
                .items
                .iter()
                .map(|item| (item.definition_id, item.count))
                .collect();
            self.grant_reward(0, &grants, out)?;
            // (R) TYPE8 resource keys use the configured output item definition;
            // no retail regional resource-to-quest table has been recovered.
            for item in &grant.items {
                self.npc_quest_event(
                    crate::quest_engine::QuestEvent::GatherResource {
                        resource_id: item.definition_id as u32,
                        quantity: item.count,
                    },
                    out,
                )?;
            }
            self.npc_minigame_win(&result.replies, out)?;
            for amount in grant.job_xp.values() {
                self.award_job_xp(*amount, out)?;
            }
            if let Some(nodes) = self.gathering_nodes.as_mut() {
                nodes.paid();
            }
        }
        out.changed |= result.changed;
        out.replies.extend(result.replies);
        Ok(())
    }

    fn consume_materials(
        &mut self,
        consumes: Vec<(i32, u32)>,
        out: &mut Outcome,
    ) -> io::Result<()> {
        for (definition, mut remaining) in consumes {
            let stacks: Vec<_> = self
                .inventory
                .state()
                .items
                .iter()
                .filter(|item| item.definition_id == definition && item.rental.is_none())
                .map(|item| (item.id, item.count))
                .collect();
            for (id, available) in stacks {
                if remaining == 0 {
                    break;
                }
                let quantity = available.min(remaining);
                let consumed = self.inventory.consume(id, quantity)?;
                out.changed |= consumed.changed;
                out.replies.extend(consumed.replies);
                remaining -= quantity;
            }
            if remaining != 0 {
                return Err(invalid("crafting material consumption incomplete"));
            }
        }
        Ok(())
    }
}
