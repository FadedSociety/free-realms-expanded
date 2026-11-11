//! G3: the gameplay half of `crate::character_admin`, compiled as a child of `gameplay`
//! (`gameplay/admin_move.rs` includes it). Every write runs on the character's staged gameplay
//! through the paths the game already uses: `grant_reward` (a quest reward's and `realm give`'s
//! grant: definition, stack and item-id rules, the inventory quest events, pet adoption),
//! `Inventory::consume` (unequip first, then 38/4), the 38/19 wallet, the quest engine's own
//! setter, and the job curve's 38/14 and 38/18. An error is the refusal: the caller drops the
//! staged clone, so nothing changes.
use super::super::*;
use crate::character_admin::{
    AdminError, Amount, Change, CharacterView, CoinOp, Edit, EquippedRow, InventoryView, ItemRow,
    JobRow, JobSet, MountRow, Pick, QuestRow, Removal, item_label,
};
use crate::inventory::EquippedItem;
use crate::quest_engine::{QuestProgress, QuestStatus};

fn refused(text: impl Into<String>) -> AdminError {
    AdminError::Refused(text.into())
}

fn stale(what: &str) -> AdminError {
    AdminError::Conflict(format!("{what} changed since that edit; nothing changed"))
}

/// The client's equip change (35/6) also goes to the players who can see this one.
fn share_equip_changes(replies: &[Vec<u8>], out: &mut Outcome) {
    out.nearby.extend(
        replies
            .iter()
            .filter(|reply| reply.starts_with(&[35, 0, 6, 0]))
            .cloned(),
    );
}

impl Gameplay {
    /// One staff write on this gameplay (a staged clone: the host commits it or drops it).
    pub fn admin_edit(&mut self, edit: &Edit, now_ms: u64) -> Result<Outcome, AdminError> {
        edit.check()?;
        // (R) A running activity can hold its own copy of the bag or the journal (a fishing
        // round's reward snapshot) and adopt it when it ends, which would drop this edit.
        if self.busy_outside_housing() {
            return Err(AdminError::Conflict(
                "the player is busy with an activity; try again when it ends".into(),
            ));
        }
        let mut out = Outcome::default();
        match edit {
            Edit::AddItems {
                item_id,
                count,
                equip,
            } => self.admin_add(*item_id, *count, *equip, now_ms, &mut out)?,
            Edit::RemoveItems(rows) => self.admin_remove(rows, &mut out)?,
            Edit::Coins(op, amount) => self.admin_coins(*op, *amount, &mut out)?,
            Edit::Quest(op, quest_id) => {
                let engine = self.admin_quests()?;
                let to = engine.admin_target(*quest_id, *op).map_err(refused)?;
                let from = engine.state().quests.get(quest_id).cloned();
                self.admin_quest(*quest_id, from, to, &mut out)?;
            }
            Edit::Job(job, set) => self.admin_job(*job, *set, &mut out)?,
            Edit::Apply(change) => self.admin_apply(change, &mut out)?,
        }
        out.changed = true;
        Ok(out)
    }

    /// The bag (every stack) and the worn items, with names from the item tables.
    pub fn admin_inventory(&self) -> InventoryView {
        let state = self.inventory.state();
        let label = |id| item_label(&self.inventory, id);
        let bags = state
            .items
            .iter()
            .map(|item| {
                let (name, icon) = label(item.definition_id);
                ItemRow {
                    slot: item.id,
                    item_id: item.definition_id,
                    name,
                    count: item.count,
                    icon,
                }
            })
            .collect();
        let equipped = state
            .equipment
            .iter()
            .filter_map(|row| {
                let item = state.items.iter().find(|i| i.id == row.item_id)?;
                let (name, icon) = label(item.definition_id);
                Some(EquippedRow {
                    profile_id: row.profile_id,
                    slot: row.slot,
                    instance: row.item_id,
                    item_id: item.definition_id,
                    name,
                    icon,
                })
            })
            .collect();
        InventoryView {
            coins: state.coins,
            bags,
            equipped,
        }
    }

    /// Coins, jobs, regional quests, mounts, pets and collections.
    pub fn admin_profile(&self) -> CharacterView {
        let jobs = self.profiles.as_ref().map_or_else(Vec::new, |profiles| {
            let state = profiles.state();
            state
                .unlocked
                .iter()
                .map(|&job_id| JobRow {
                    job_id,
                    level: profiles.rank(job_id),
                    xp: state.experience.get(&job_id).copied().unwrap_or(0),
                    active: job_id == state.active,
                })
                .collect()
        });
        let quests = self.npc_quest_state();
        let quest_rows = |want: QuestStatus| {
            quests
                .iter()
                .flat_map(|state| &state.quests)
                .filter(|(_, progress)| progress.status == want)
                .map(|(&quest_id, _)| QuestRow {
                    quest_id,
                    title: self
                        .npc_quests
                        .as_ref()
                        .and_then(|engine| engine.admin_title(quest_id))
                        .unwrap_or_default()
                        .to_owned(),
                })
                .collect()
        };
        let mounts = self.mounts.as_ref().map_or_else(Vec::new, |mounts| {
            mounts
                .catalog()
                .owned_mounts(self.inventory.state())
                .into_iter()
                .map(|mount| MountRow {
                    mount_id: mount.id,
                    name: mount.name.clone(),
                })
                .collect()
        });
        let pets = self
            .pets_document()
            .and_then(|roster| {
                let pets = roster.get("pets")?.as_array()?;
                Some(
                    pets.iter()
                        .filter_map(|pet| Some(pet.get("name")?.as_str()?.to_owned()))
                        .collect(),
                )
            })
            .unwrap_or_default();
        let ledger = self.progression_extras.collections();
        CharacterView {
            coins: self.inventory.state().coins,
            jobs,
            quests_active: quest_rows(QuestStatus::Active),
            quests_done: quest_rows(QuestStatus::Completed),
            mounts,
            pets,
            collections_n: ledger.completed_count(),
            collection_entries: ledger.collected_count(),
        }
    }

    fn admin_quests(&self) -> Result<&crate::quest_engine::QuestEngine, AdminError> {
        self.npc_quests
            .as_ref()
            .ok_or_else(|| refused("this character has no regional quest data"))
    }

    /// A grant's rules are a quest reward's: a known definition, one stack's worth, and no new
    /// stack past the reward bag capacity (100 stacks, or more for an older full bag, R). An
    /// adoption item is refused past the pet roster's room, as `realm give` refuses it.
    fn admin_add(
        &mut self,
        item: i32,
        count: u32,
        equip: bool,
        now_ms: u64,
        out: &mut Outcome,
    ) -> Result<(), AdminError> {
        if !self.inventory.has_definition(item) {
            return Err(refused(format!("item {item} is not in the item tables")));
        }
        if !self.inventory.accepts_reward(item, count) {
            return Err(refused(format!(
                "{count} is more than one stack of item {item} holds"
            )));
        }
        let room = self.pets.room();
        if count as usize > room && self.pets.is_adoption_item(item) {
            return Err(refused(format!("the pet list has room for {room} more")));
        }
        let capacity = self.reward_snapshot().bag_capacity;
        let before = self.inventory.state().clone();
        self.grant_reward(0, &[(item, count)], out)?;
        if self.inventory.state().items.len() > capacity {
            return Err(refused(format!("the bag is full ({capacity} stacks)")));
        }
        self.settle_granted_pets(now_ms, out)?;
        if equip {
            let profile = self
                .active_profile()
                .ok_or_else(|| refused("this character has no job to wear it"))?;
            let worn = self
                .inventory
                .equip_owned_definition(item, profile)
                .map_err(|error| {
                    refused(format!("job {profile} cannot wear item {item}: {error}"))
                })?;
            share_equip_changes(&worn.replies, out);
            out.replies.extend(worn.replies);
            self.resend_bar_after_weapon_change(&before, out)?;
            self.sync_weapon();
        }
        let (name, _) = item_label(&self.inventory, item);
        out.replies
            .push(chat::system(&format!("A GM gave you {count} x {name}.")));
        Ok(())
    }

    fn admin_remove(&mut self, rows: &[Removal], out: &mut Outcome) -> Result<(), AdminError> {
        let before = self.inventory.state().clone();
        for row in rows {
            let stacks: Vec<(i32, u32)> = self
                .inventory
                .state()
                .items
                .iter()
                .filter(|item| match row.pick {
                    Pick::Slot(id) => item.id == id,
                    Pick::Item(definition) => item.definition_id == definition,
                })
                .map(|item| (item.id, item.count))
                .collect();
            let held: u64 = stacks.iter().map(|&(_, n)| u64::from(n)).sum();
            let what = match row.pick {
                Pick::Slot(id) => format!("bag slot {id}"),
                Pick::Item(definition) => format!("item {definition}"),
            };
            if stacks.is_empty() {
                return Err(refused(format!("the bag holds no {what}")));
            }
            let removed = match (row.amount, row.pick) {
                (Amount::All, _) => stacks
                    .iter()
                    .map(|&(id, n)| self.inventory.consume(id, n))
                    .collect::<io::Result<Vec<_>>>()?,
                (Amount::Count(n), _) if u64::from(n) > held => {
                    return Err(refused(format!("the bag holds only {held} of {what}")));
                }
                (Amount::Count(n), Pick::Slot(id)) => vec![self.inventory.consume(id, n)?],
                (Amount::Count(n), Pick::Item(definition)) => {
                    vec![self.inventory.consume_definition(definition, n)?]
                }
            };
            out.replies
                .extend(removed.into_iter().flat_map(|outcome| outcome.replies));
        }
        self.npc_inventory_events(&before, out)?;
        self.resend_bar_after_weapon_change(&before, out)?;
        self.sync_weapon();
        out.replies
            .push(chat::system("A GM removed items from your bag."));
        Ok(())
    }

    /// The 38/19 wallet through `grant_coins` (refused past the i32 coin field) and
    /// `spend_coins` (refused below zero).
    fn admin_coins(
        &mut self,
        op: CoinOp,
        amount: u32,
        out: &mut Outcome,
    ) -> Result<(), AdminError> {
        let have = self.inventory.state().coins;
        let (give, take) = match op {
            CoinOp::Give => (amount, 0),
            CoinOp::Take => (0, amount),
            CoinOp::Set => (amount.saturating_sub(have), have.saturating_sub(amount)),
        };
        if take > have {
            return Err(refused(format!("the wallet holds only {have} coins")));
        }
        if give == 0 && take == 0 {
            return Err(refused(format!("the wallet already holds {have} coins")));
        }
        let before = self.inventory.state().clone();
        if give > 0 {
            out.replies
                .extend(self.inventory.grant_coins(give)?.replies);
        }
        if take > 0 {
            out.replies
                .extend(self.inventory.spend_coins(take)?.replies);
        }
        self.npc_inventory_events(&before, out)?;
        let coins = self.inventory.state().coins;
        out.replies
            .push(chat::system(&format!("A GM set your coins to {coins}.")));
        Ok(())
    }

    fn admin_quest(
        &mut self,
        quest_id: u32,
        from: Option<QuestProgress>,
        to: Option<QuestProgress>,
        out: &mut Outcome,
    ) -> Result<(), AdminError> {
        let engine = self
            .npc_quests
            .as_mut()
            .ok_or_else(|| refused("this character has no regional quest data"))?;
        let quest = engine
            .admin_set_progress(quest_id, from.as_ref(), to)
            .map_err(refused)?;
        self.append_npc_quest_outcome_synced(quest, out)?;
        self.refresh_npc_quest_eligibility(current_unix_seconds());
        out.replies
            .push(chat::system("A GM changed your quest journal."));
        Ok(())
    }

    fn admin_job(&mut self, job: i32, set: JobSet, out: &mut Outcome) -> Result<(), AdminError> {
        let profiles = self
            .profiles
            .as_ref()
            .ok_or_else(|| refused("this character has no jobs"))?;
        if !profiles.state().unlocked.contains(&job) {
            return Err(refused(format!(
                "job {job} is not unlocked for this character"
            )));
        }
        let curve = profiles.curve(job);
        let xp = match set {
            JobSet::Level(level) => curve.xp_for(level),
            JobSet::Xp(xp) => xp,
        };
        if profiles.state().experience.get(&job).copied().unwrap_or(0) == xp {
            return Err(refused(format!("job {job} already has {xp} XP")));
        }
        self.set_job_xp(job, xp, out)?;
        out.replies.push(chat::system(&format!(
            "A GM set job {job} to level {}.",
            curve.level(xp)
        )));
        Ok(())
    }

    /// Any job's XP, as `realm level` sets the active one's: the progression snapshot adopted,
    /// the curve's 38/18 and 38/14, the ability bar when an active job's level crosses an
    /// unlock or drops, then the level-driven health and the quest level gates.
    fn set_job_xp(&mut self, job: i32, xp: u32, out: &mut Outcome) -> Result<(), AdminError> {
        let profiles = self
            .profiles
            .as_ref()
            .ok_or_else(|| refused("this character has no jobs"))?;
        let curve = profiles.curve(job);
        let previous = profiles.state().experience.get(&job).copied().unwrap_or(0);
        let mut state = profiles.state().clone();
        state.experience.insert(job, xp);
        let directory = self
            .profile_content_directory
            .clone()
            .ok_or_else(|| refused("this character has no job content"))?;
        self.profiles = Some(Profiles::adopt(&self.profiles, &directory, state)?);
        out.replies.extend(crate::rewards::experience_packets(
            job, previous, xp, &curve,
        ));
        let (was, now) = (curve.level(previous), curve.level(xp));
        if self.active_profile() == Some(job)
            && self
                .abilities
                .as_ref()
                .is_some_and(|abilities| abilities.unlocks_between(job, was, now) || now < was)
        {
            out.replies.extend(self.ability_activation_for(job)?);
        }
        self.sync_player_level(out);
        self.npc_quest_profiles_changed(out)?;
        Ok(())
    }

    /// An undo (or any recorded change): every touched part must still be at the change's
    /// before, then moves to its after.
    fn admin_apply(&mut self, change: &Change, out: &mut Outcome) -> Result<(), AdminError> {
        if change.touches_inventory() {
            self.restore_inventory(change, out)?;
        }
        for quest in &change.quests {
            let current = self.admin_quests()?.state().quests.get(&quest.quest_id);
            if current != quest.before.as_ref() {
                return Err(stale(&format!("quest {}", quest.quest_id)));
            }
            self.admin_quest(
                quest.quest_id,
                quest.before.clone(),
                quest.after.clone(),
                out,
            )?;
        }
        for job in &change.jobs {
            let have = self
                .profiles
                .as_ref()
                .and_then(|p| p.state().experience.get(&job.job_id).copied())
                .unwrap_or(0);
            if have != job.before {
                return Err(stale(&format!("job {}", job.job_id)));
            }
            self.set_job_xp(job.job_id, job.after, out)?;
        }
        out.replies.push(chat::system(
            "A GM undid an earlier change to your character.",
        ));
        Ok(())
    }

    /// The touched coins, stacks and worn rows put back through `adopt_state` (38/4, 38/2, the
    /// stack updates, 38/19). A row emptied gets the client's 38/6 first; a row worn again goes
    /// through the equip path, so the client and the players near it see it.
    fn restore_inventory(&mut self, change: &Change, out: &mut Outcome) -> Result<(), AdminError> {
        let before = self.inventory.state().clone();
        let mut state = before.clone();
        if let Some([from, to]) = change.coins {
            if state.coins != from {
                return Err(stale("the wallet"));
            }
            state.coins = to;
        }
        for stack in &change.stacks {
            let at = state.items.iter().position(|item| item.id == stack.id);
            if at.map(|i| &state.items[i]) != stack.before.as_ref() {
                return Err(stale(&format!("bag slot {}", stack.id)));
            }
            match (at, &stack.after) {
                (Some(i), Some(item)) => state.items[i] = item.clone(),
                (Some(i), None) => {
                    state.items.remove(i);
                }
                (None, Some(item)) => state.items.push(item.clone()),
                (None, None) => {}
            }
        }
        let (mut unequip, mut wear) = (Vec::new(), Vec::new());
        for [from, to] in &change.equipment {
            let row = state
                .equipment
                .iter_mut()
                .find(|row| row.profile_id == to.profile_id && row.slot == to.slot);
            if row.as_ref().map_or(0, |row| row.item_id) != from.item_id {
                return Err(stale(&format!("equipment slot {}", to.slot)));
            }
            match row {
                Some(row) => row.item_id = 0,
                None => state.equipment.push(EquippedItem {
                    item_id: 0,
                    ..to.clone()
                }),
            }
            if to.item_id == 0 {
                let mut packet = vec![38, 0, 6, 0];
                packet.extend(to.slot.to_le_bytes());
                packet.extend(to.profile_id.to_le_bytes());
                unequip.push(packet);
            } else {
                wear.push(to.clone());
            }
        }
        out.replies.extend(unequip);
        out.replies.extend(self.inventory.adopt_state(state)?);
        for row in wear {
            let definition = self
                .inventory
                .state()
                .items
                .iter()
                .find(|item| item.id == row.item_id)
                .map(|item| item.definition_id)
                .ok_or_else(|| stale(&format!("equipment slot {}", row.slot)))?;
            let worn = self
                .inventory
                .equip_owned_definition(definition, row.profile_id)?;
            share_equip_changes(&worn.replies, out);
            out.replies.extend(worn.replies);
        }
        self.npc_inventory_events(&before, out)?;
        self.resend_bar_after_weapon_change(&before, out)?;
        self.sync_weapon();
        Ok(())
    }
}
