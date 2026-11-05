//! Job selection, trainer learning and starter outfits.
//!
//! Everything here stages its changes on the `Gameplay` clone the gateway owns for the
//! request, so a refusal leaves the old job, inventory and open NPC menu untouched.
use super::*;

impl Gameplay {
    /// L926: the job's XP curve (`job_xp_curve.json`), or the local 100-XP curve without
    /// profile content. The startup self data reads it for each job record.
    pub fn job_curve(&self, profile: i32) -> crate::profiles::RankCurve {
        self.profiles
            .as_ref()
            .map_or_else(crate::profiles::RankCurve::flat, |profiles| {
                profiles.curve(profile)
            })
    }

    /// L927: what one unit of `profile`'s repeatable activity pays: the job's `activity_xp`
    /// (`job_xp_curve.json`), else `fallback`, the activity's own constant.
    pub(super) fn activity_xp(&self, profile: i32, fallback: u32) -> u32 {
        let profiles = self.profiles.as_ref();
        profiles
            .and_then(|profiles| profiles.activity_xp(profile))
            .unwrap_or(fallback)
    }

    /// L936: entering a job's game switches the player into that job, and the job stays after the
    /// game (client 101319: "When you start the minigame, you automatically switch to the right
    /// Job"; 76669 says the same for harvesting nodes). First entry unlocks the job and hands its
    /// starter outfit (everything is free). A job whose outfit cannot dress Chest and Legs is not
    /// switched, because the client's `ProcessProfileSwap` (`0x00bfb620`) rejects it: the game
    /// still starts, in the current job.
    pub(super) fn enter_activity_job(
        &mut self,
        job: i32,
        guid: u64,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let Some(profiles) = self.profiles.as_mut() else {
            return Ok(());
        };
        if profiles.active() == job {
            return Ok(());
        }
        // L953: a job unlocked bare (before every job had a kit) is dressed too, from what the
        // player owns first; nothing is granted twice.
        let first = profiles.unlock(job)?;
        if first || !self.inventory.has_required_profile_slots(job) {
            self.give_starter_outfit(job, !first, out)?;
        }
        let mut staged = self.clone();
        let mut switch = Outcome::default();
        staged.restore_job(job, guid, &mut switch)?;
        if !staged.inventory.has_required_profile_slots(job) {
            // Unlocked but not switched: the job still needs its client row, since a later rank-up
            // (38/18, `009392c0` case 0x12) reads the row without a null check.
            if first {
                out.replies
                    .push(self.profiles()?.add_packet(job, &self.inventory)?);
            }
            return Ok(());
        }
        *self = staged;
        out.changed = true;
        out.replies.extend(switch.replies);
        out.nearby.extend(switch.nearby);
        Ok(())
    }

    /// L936: leaving a race or match keeps the activity's job; only a race or match saved before
    /// L936 still names a prior job to go back to. Either way the on-foot action bar is sent for
    /// the job the player ends in, as the old exit's `restore_job` did.
    pub(super) fn leave_activity_job(
        &mut self,
        previous: Option<i32>,
        guid: u64,
        out: &mut Outcome,
    ) -> io::Result<()> {
        if let Some(job) = previous {
            return self.restore_job(job, guid, out);
        }
        if let (Some(_), Some(profiles)) = (&self.abilities, &self.profiles) {
            out.replies
                .extend(self.ability_activation_for(profiles.active())?);
        }
        Ok(())
    }

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

    /// A job unlocked without switching to it: its outfit, then its row on the client's list.
    pub(super) fn dress_new_job(&mut self, job: i32, out: &mut Outcome) -> io::Result<()> {
        self.give_starter_outfit(job, false, out)?;
        out.replies
            .push(self.profiles()?.add_packet(job, &self.inventory)?);
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
        for definition in items {
            // The inherited Chef defaults include the same hat that
            // this local story awards at Cooking101 turn-in. Defer only
            // that cosmetic in the versioned Cooking campaign; older
            // campaigns and original profile records stay compatible.
            if job == 45 && definition == 14794 && self.cooking.is_some() {
                continue;
            }
            // L953: a kit can list a boys' and a girls' piece for one slot; the other gender's
            // piece is skipped (an unknown gender keeps both, the last equipped wins).
            let need = self.inventory.definition_gender(definition).unwrap_or(0);
            if need != 0 && self.gender != 0 && need != self.gender {
                continue;
            }
            self.give_starter_piece(job, definition, reuse_owned, out)?;
        }
        Ok(())
    }

    /// One starter piece: equipped from the bag when owned (`reuse_owned`), else granted and
    /// equipped. A piece the catalogue lacks is skipped.
    fn give_starter_piece(
        &mut self,
        job: i32,
        definition: i32,
        reuse_owned: bool,
        out: &mut Outcome,
    ) -> io::Result<()> {
        if !self.inventory.has_definition(definition) {
            return Ok(());
        }
        let inactive = job != self.profiles.as_ref().unwrap().active();
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
        Ok(())
    }

    /// L933: a job at its work is dressed and armed. An outfit without the Chest and Legs the
    /// client's profile swap needs gets the whole starter outfit (owned pieces equipped, none
    /// granted twice); a complete outfit is left alone, then the tool slot is checked. Inventory
    /// quest events are the request boundary's, as for the trainer path.
    pub(super) fn ensure_job_gear(&mut self, job: i32, out: &mut Outcome) -> io::Result<()> {
        if !self.inventory.has_required_profile_slots(job) {
            self.grant_starter_outfit(job, true, out)?;
        }
        self.ensure_tool(job, out)
    }

    /// L933: a job at its work has its tool in hand. The Fisherman's fishing aim runs only
    /// while the local player has an item in equipment slot 7 (`FishingProcessor::Update`
    /// `0x00b6c290`), so a Fisherman who sold or never got the rod could not cast. When
    /// `job` has nothing in slot 7, equip the starter tool from the content's starter table:
    /// an owned one, else a new grant. A job whose starter table has no slot-7 piece is left
    /// as it is.
    pub(super) fn ensure_tool(&mut self, job: i32, out: &mut Outcome) -> io::Result<()> {
        if self.inventory.state().equipped_weapon_item(job) != 0 {
            return Ok(());
        }
        // Slot 7 is the weapon/tool slot (`InventoryState::equipped_weapon_item`).
        let tool = self.profiles.as_ref().and_then(|profiles| {
            profiles
                .starter_items(job)
                .iter()
                .copied()
                .find(|&definition| self.inventory.definition_slot(definition) == Some(7))
        });
        match tool {
            Some(definition) => self.give_starter_piece(job, definition, true, out),
            None => Ok(()),
        }
    }

    /// (R) A job never stays bare: when a job with a completely empty outfit
    /// becomes active, it gets its starter outfit (The owner, 2026-10-15). Fixes
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
    /// Kart Driver stays only when chosen or required (The owner, 2026-10-15).
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
        // A job press must never cost the player the session: the client shows no disconnect.
        if let Err(error) = staged.ensure_outfit(id, &mut selection) {
            out.replies
                .push(chat::system(&format!("Cannot switch classes: {error}")));
            return Ok(None);
        }
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
        out.replies
            .push(self.profiles()?.add_packet(trainer.profile, &self.inventory)?);
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

    /// Developer `realm level <1-20>`: set the active job's saved XP to that level. Moved out of
    /// the gameplay hub (T545): the command owns the rank write, its XP packets, the ability-bar
    /// refresh and the job-level quest goals, and it stages on the caller's `Outcome`.
    pub(super) fn developer_level(&mut self, level: &str, out: &mut Outcome) -> io::Result<()> {
        let Ok(level) = level.parse::<u32>() else {
            out.replies.push(chat::system("Usage: realm level <1-20>"));
            return Ok(());
        };
        let level = level.clamp(1, 20);
        let profile = self.profiles.as_ref().unwrap().active();
        let before = self.job_level(profile);
        let profiles = self.profiles.as_mut().unwrap();
        let previous = profiles
            .state()
            .experience
            .get(&profile)
            .copied()
            .unwrap_or(0);
        let start = profiles.curve(profile).xp_for(level);
        out.changed |= profiles.set_experience(start);
        out.replies.extend(profiles.experience_packets(previous));
        if self.abilities.as_ref().is_some_and(|abilities| {
            abilities.unlocks_between(profile, before, level) || level < before
        }) {
            out.replies.extend(self.ability_activation_for(profile)?);
        }
        self.sync_player_level(out);
        self.npc_quest_profiles_changed(out)?;
        out.replies.push(chat::system(&format!(
            "Job {profile} is now level {level}."
        )));
        Ok(())
    }
}
