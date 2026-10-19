//! Developer hot reload of authored content: battles, enemies, abilities.
//!
//! The whole request is staged on the `Gameplay` clone the gateway owns for the
//! request. Only the four subsystems a reload owns commit, and only after every
//! requested loader succeeds; a refusal publishes neither state nor packets.
use super::*;

impl Gameplay {
    pub fn reload_content(&mut self, what: &str, out: &mut Outcome) -> io::Result<String> {
        let mut staged = self.clone();
        let mut staged_out = Outcome::default();
        let done = staged.reload_content_in_place(what, &mut staged_out)?;
        self.abilities = staged.abilities;
        self.encounter = staged.encounter;
        self.activities = staged.activities;
        self.npc_visibility = staged.npc_visibility;
        out.dev_command_denied |= staged_out.dev_command_denied;
        out.shared_field = staged_out.shared_field.or(out.shared_field.take());
        out.replies.extend(staged_out.replies);
        out.changed |= staged_out.changed;
        out.zoning = staged_out.zoning.or(out.zoning.take());
        out.scene_zoning = staged_out.scene_zoning.or(out.scene_zoning.take());
        out.relocation = staged_out.relocation.or(out.relocation.take());
        out.group_heal = staged_out.group_heal.or(out.group_heal.take());
        out.nearby.extend(staged_out.nearby);
        out.pets_changed |= staged_out.pets_changed;
        Ok(done)
    }

    fn reload_content_in_place(&mut self, what: &str, out: &mut Outcome) -> io::Result<String> {
        let dir = self.content_dir.clone();
        let mut done = Vec::new();
        let all = what == "all";
        if all || what == "abilities" {
            let path = dir.join("job_abilities.json");
            if path.is_file() {
                if let Some(abilities) = self.abilities.as_mut() {
                    abilities.load_job_abilities(&path)?;
                    let profile = self.profiles.as_ref().unwrap().active();
                    let level = self.job_level(profile);
                    out.replies.extend(
                        self.abilities
                            .as_ref()
                            .unwrap()
                            .activation(profile, level)?,
                    );
                    done.push("job abilities");
                }
            }
        }
        if all || what == "battles" {
            let path = dir.join("field_battles.json");
            if path.is_file() {
                if let Some(encounter) = self.encounter.as_mut() {
                    encounter.reload_field_battles(&path)?;
                    self.activities = crate::activity_portal::Catalogue::load(&path)?;
                    if let Some(chess) = &self.chess {
                        self.activities = self
                            .activities
                            .clone()
                            .with_chess_rows(chess.activities())?;
                    }
                    if let Some(visibility) = self.npc_visibility.as_mut() {
                        for (guid, name, name_id, model_id, position, icon) in
                            encounter.battle_spawners()
                        {
                            visibility
                                .add_static_actor(guid, &name, name_id, model_id, position, icon);
                        }
                    }
                    done.push("battles");
                }
            }
        }
        if all || what == "behaviours" {
            let path = dir.join("behaviours.json");
            if path.is_file() {
                if let Some(encounter) = self.encounter.as_mut() {
                    encounter.load_behaviours(&path)?;
                    done.push("behaviours");
                }
            }
        }
        if all || what == "enemies" {
            let path = dir.join("field_enemies.json");
            if path.is_file() {
                if let Some(encounter) = self.encounter.as_mut() {
                    out.replies.extend(encounter.reload_field_enemies(&path)?);
                    done.push("field enemies");
                }
                self.sync_field_quests(out)?;
            }
        }
        if all || what == "power_ups" {
            let path = dir.join("power_ups.json");
            if path.is_file() {
                if let Some(encounter) = self.encounter.as_mut() {
                    encounter.load_power_ups(&path)?;
                    done.push("power-ups");
                }
            }
        }
        if all || what == "placements" {
            let path = self.placements_path();
            if let Some(visibility) = self.npc_visibility.as_mut() {
                let records = if path.is_file() {
                    let bytes = std::fs::read(&path)?;
                    serde_json::from_slice::<crate::world::PlacementFile>(&bytes)
                        .map_err(|e| invalid(format!("npc_placements.json: {e}")))?
                        .records
                } else {
                    Vec::new()
                };
                out.replies.extend(visibility.replace_placements(&records)?);
                done.push("placed NPCs");
            }
        }
        if done.is_empty() {
            return Err(invalid(
                "unknown target; use realm reload [all|battles|enemies|abilities|behaviours|power_ups|placements]",
            ));
        }
        Ok(done.join(", "))
    }
}
