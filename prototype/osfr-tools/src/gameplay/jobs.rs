//! Job selection, trainer learning and starter outfits.
//!
//! Everything here stages its changes on the `Gameplay` clone the gateway owns for the
//! request, so a refusal leaves the old job, inventory and open NPC menu untouched.
use super::*;

impl Gameplay {
    /// Grant and equip a job's starter outfit from the content's starter table.
    /// With `reuse_owned`, a piece the player already owns is equipped instead
    /// of granting a duplicate. (R) Pieces the catalogue lacks are skipped.
    /// Inventory quest events are emitted once for the whole call.
    pub(super) fn give_starter_outfit(
        &mut self,
        job: i32,
        reuse_owned: bool,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let before = self.inventory.state().clone();
        self.grant_starter_outfit(job, reuse_owned, out)?;
        self.npc_inventory_events(&before, out)?;
        Ok(())
    }

    /// The starter grant without inventory quest events. The caller emits them
    /// once over its committed transaction (the trainer path already diffs the
    /// whole request in `handle_in_world_inner`).
    pub(super) fn grant_starter_outfit(
        &mut self,
        job: i32,
        reuse_owned: bool,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let items = self.profiles.as_ref().unwrap().starter_items(job).to_vec();
        let inactive = job != self.profiles.as_ref().unwrap().active();
        for definition in items {
            // The inherited Chef defaults include the same hat that
            // this local story awards at Cooking101 turn-in. Defer only
            // that cosmetic in the versioned Cooking campaign; older
            // campaigns and original profile records stay compatible.
            if job == 45 && definition == 14794 && self.cooking.is_some() {
                continue;
            }
            if !self.inventory.has_definition(definition) {
                continue;
            }
            let owned = if reuse_owned {
                self.inventory.equip_owned_definition(definition, job).ok()
            } else {
                None
            };
            let result = match owned {
                Some(result) => result,
                None => {
                    out.replies
                        .extend(self.inventory.grant(0, definition, 1)?.replies);
                    self.inventory.equip_owned_definition(definition, job)?
                }
            };
            out.changed = true;
            out.replies.extend(
                result
                    .replies
                    .into_iter()
                    .filter(|r| !(inactive && r.starts_with(&[35, 0, 6, 0]))),
            );
        }
        Ok(())
    }

    /// (R) A job never stays bare: when a job with a completely empty outfit
    /// becomes active, it gets its starter outfit (the owner, 2026-10-15). Fixes
    /// saves whose job was unlocked by a path that granted no clothes.
    pub(super) fn ensure_outfit(&mut self, job: i32, out: &mut Outcome) -> io::Result<()> {
        if self
            .inventory
            .profile_attachments(job)?
            .starts_with(&0i32.to_le_bytes())
        {
            self.give_starter_outfit(job, true, out)?;
        }
        let has_weapon = self
            .inventory
            .state()
            .equipment
            .iter()
            .any(|entry| entry.profile_id == job && entry.slot == 7 && entry.item_id != 0);
        let starter = self
            .abilities
            .as_ref()
            .and_then(|abilities| abilities.starter_weapon(job));
        if !has_weapon
            && let Some(definition) = starter
            && self.inventory.has_definition(definition)
        {
            if !self
                .inventory
                .state()
                .items
                .iter()
                .any(|item| item.definition_id == definition)
            {
                out.replies
                    .extend(self.inventory.grant(0, definition, 1)?.replies);
            }
            let equipped = self.inventory.equip_owned_definition(definition, job)?;
            out.changed |= equipped.changed;
            out.replies.extend(equipped.replies);
        }
        Ok(())
    }

    /// (R) Leaving a race puts the player back in the job they had before it;
    /// Kart Driver stays only when chosen or required (the owner, 2026-10-15).
    /// The 38/21 activation is serialized from the repaired outfit, not from the
    /// inventory before `ensure_outfit` (client reader `0x0091b080`).
    pub(super) fn restore_job(&mut self, job: i32, guid: u64, out: &mut Outcome) -> io::Result<()> {
        if self.profiles.as_ref().unwrap().active() == job {
            return Ok(());
        }
        let at = out.replies.len();
        let outfit_before = self
            .inventory
            .profile_attachments(self.profiles.as_ref().unwrap().active())?;
        self.profiles.as_mut().unwrap().activate(job)?;
        // T499/F736: every job switch settles quest eligibility where it happens (see
        // `stage_job_selection`, `begin_kart_race`, `begin_derby`), so the marker pass that ends
        // this packet reads the new job.
        self.refresh_npc_quest_eligibility(current_unix_seconds());
        self.sync_player_level(out);
        // Repair the outfit and weapon before the bar: T521 labels the basic
        // slot from the live slot-7 weapon, so a job re-armed here must not
        // send the empty bar first.
        self.ensure_outfit(job, out)?;
        // The bar goes through the power-up seam: a held kind keeps its slot.
        if self.abilities.is_some() {
            out.replies.extend(self.ability_activation_for(job)?);
        }
        let activation = self
            .profiles
            .as_ref()
            .unwrap()
            .activation(job, &self.inventory)?;
        out.replies.insert(at, activation);
        self.announce_profile_outfit(guid, &outfit_before, true, out)
    }

    /// Stage an ordinary 26/13 job switch. The candidate job is committed only
    /// when its outfit carries the Chest and Legs attachments the client's
    /// `ProcessProfileSwap` (`0x00bfb620`) requires; otherwise the previous job
    /// stays active. The activation is serialized after the outfit repair, so
    /// its attachment list is never empty for a job `ensure_outfit` can dress.
    pub(super) fn stage_job_selection(
        &mut self,
        id: i32,
        out: &mut Outcome,
    ) -> io::Result<Option<bool>> {
        let Some(profiles) = &self.profiles else {
            return Ok(None);
        };
        if profiles.is_staff_job(id) {
            return Ok(self.wear_staff_job(id, out));
        }
        if !profiles.state().unlocked.contains(&id) {
            out.replies.push(chat::system(
                "Cannot switch classes: that class has not been unlocked",
            ));
            return Ok(None);
        }
        let mut staged = self.clone();
        let mut selection = Outcome::default();
        let changed = staged.profiles.as_ref().unwrap().shown() != id;
        if let Err(error) = staged.profiles.as_mut().unwrap().activate(id) {
            out.replies
                .push(chat::system(&format!("Cannot switch classes: {error}")));
            return Ok(None);
        }
        staged.ensure_outfit(id, &mut selection)?;
        // Client ProcessProfileSwap 0x00bfb620 rejects a group without these.
        if !staged.inventory.has_required_profile_slots(id) {
            out.replies.push(chat::system(
                "Cannot switch classes: that job's outfit is missing a required Chest or Legs item.",
            ));
            return Ok(None);
        }
        let activation = staged
            .profiles
            .as_ref()
            .unwrap()
            .activation(id, &staged.inventory)?;
        selection.replies.insert(0, activation);
        selection.changed |= changed;
        *self = staged;
        self.refresh_npc_quest_eligibility(current_unix_seconds());
        out.changed |= selection.changed;
        out.replies.extend(selection.replies);
        out.nearby.extend(selection.nearby);
        Ok(Some(changed))
    }

    /// L052 (R): wear a granted staff job over the real one. The client shows it (this 38/21,
    /// the callers' 35/7 to neighbours, AddPc for later viewers) while the active job, its XP,
    /// quests, inventory and save stay untouched. A refusal is one chat line, nothing else.
    fn wear_staff_job(&mut self, id: i32, out: &mut Outcome) -> Option<bool> {
        let profiles = self.profiles.as_mut()?;
        let changed = profiles.shown() != id;
        let Ok(activation) = profiles.wear_staff_job(id, &self.inventory) else {
            out.replies.push(chat::system(
                "Cannot switch classes: that job's outfit is unavailable.",
            ));
            return None;
        };
        out.replies.push(activation);
        out.changed |= changed;
        Some(changed)
    }

    /// One profile's outfit as the native attachment list (count + rows); a staff job's outfit.
    pub fn profile_attachments(&self, profile_id: i32) -> io::Result<Vec<u8>> {
        match &self.profiles {
            Some(profiles) => profiles.attachments(profile_id, &self.inventory),
            None => self.inventory.profile_attachments(profile_id),
        }
    }

    /// L052: grant (moderators and admins) or withhold this session's staff jobs; the gateway
    /// decides once at login from the account role.
    pub fn set_staff_jobs(&mut self, granted: bool) {
        if let Some(profiles) = self.profiles.as_mut() {
            profiles.set_staff_jobs(granted);
        }
    }

    /// The staff jobs' rows for the self data (opcode 12); none for a player.
    pub fn staff_job_rows(&self) -> Vec<crate::startup_selfdata::JobProfile> {
        self.profiles
            .as_ref()
            .map(Profiles::staff_rows)
            .unwrap_or_default()
    }

    /// W176 (R): teach the job the chosen trainer offers. The caller stages `self`, so a refusal
    /// (a chat line, `changed` false) leaves the player exactly as they were. Starter pieces the
    /// player already owns are equipped, never granted again, so training cannot inflate them.
    pub(super) fn learn_job(&mut self, trainer: &crate::profiles::Trainer) -> io::Result<Outcome> {
        use crate::profiles::LearnDenied;
        let mut out = Outcome::default();
        let denied = if self.activity_busy() {
            Some(LearnDenied::Busy)
        } else {
            match self.profiles.as_mut() {
                Some(profiles) => profiles.learn(trainer).err(),
                None => Some(LearnDenied::Unavailable),
            }
        };
        if let Some(denied) = denied {
            out.replies.push(chat::system(&denied.message()));
            return Ok(out);
        }
        // The job is not made active; the player picks it in the job list (R).
        // Inventory quest events are emitted once by the request boundary.
        self.grant_starter_outfit(trainer.profile, true, &mut out)?;
        out.changed = true;
        let message = if self.inventory.has_required_profile_slots(trainer.profile) {
            format!(
                "You learned the {} job. Pick it in your job list to switch.",
                trainer.job
            )
        } else {
            format!(
                "You learned the {} job. Its starter outfit is unavailable, so it cannot be selected yet.",
                trainer.job
            )
        };
        out.replies.push(chat::system(&message));
        Ok(out)
    }
}
