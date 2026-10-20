//! Developer hot reload of authored content: battles, enemies, abilities.
//!
//! (R) `realm reload` is a developer operation, not recovered client behavior;
//! its all-or-nothing rule is local reconstruction.
//!
//! The whole request is staged on the `Gameplay` clone the gateway owns for the
//! request. Only the four subsystems a reload owns commit -- `abilities`,
//! `encounter`, `activities` and `npc_visibility` -- and only after every
//! requested loader succeeds; a refusal publishes neither state nor packets.
//! A new `Gameplay` field is not reload content until it is added to that list.
//!
//! The regression `refused_reload_all_keeps_behaviour_and_shared_field_state`
//! is in `gameplay/w64_regional_quest_tests.rs`: `lab/autopilot/protected-pass.txt`
//! pins its full path, so it cannot move here yet.
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
        // The file was just read: stamp it so the placement watch does not
        // repeat the same removal/addition one tick later.
        if what == "placements" || what == "all" {
            self.placement_watch.observe(&self.placements_path());
        }
        out.absorb(staged_out);
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
                    out.replies.extend(self.ability_activation_for(profile)?);
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A rehearsed staged outcome that names every `Outcome` field explicitly:
    /// adding a field breaks this test until `Outcome::absorb` publishes it too.
    fn staged_outcome() -> Outcome {
        Outcome {
            dev_command_denied: true,
            shared_field: Some(crate::encounter::SharedFieldAction::Ability { slot: 3, target: 9 }),
            replies: vec![vec![3]],
            changed: true,
            zoning: Some(crate::encounter::ZoneTransition {
                zone_id: 7,
                name: "Seven".into(),
                sky: String::new(),
                geometry_id: 0,
                position: [0.0; 3],
                rotation: [0.0; 3],
                tutorial: false,
            }),
            scene_zoning: Some(crate::zone_transition::ZoneTransition {
                scene: "Seven".into(),
                ..Default::default()
            }),
            relocation: Some(crate::dev_teleport::Relocation {
                position: [1.0, 2.0, 3.0],
                rotation: [0.0; 3],
            }),
            group_heal: Some(crate::encounter::GroupHeal {
                percent: 50,
                kill: (7, 9),
            }),
            nearby: vec![vec![4]],
            broadcast: Some("staged notice".into()),
            pets_changed: true,
        }
    }

    #[test]
    fn absorb_publishes_every_staged_outcome_field() {
        let mut live = Outcome::default();
        live.replies.push(vec![1]);
        live.nearby.push(vec![2]);
        live.absorb(staged_outcome());
        assert!(live.dev_command_denied);
        assert!(matches!(
            live.shared_field,
            Some(crate::encounter::SharedFieldAction::Ability { slot: 3, target: 9 })
        ));
        assert_eq!(live.replies, vec![vec![1], vec![3]]);
        assert!(live.changed);
        assert_eq!(live.zoning.as_ref().unwrap().zone_id, 7);
        assert_eq!(live.scene_zoning.as_ref().unwrap().scene, "Seven");
        assert_eq!(live.relocation.as_ref().unwrap().position, [1.0, 2.0, 3.0]);
        assert_eq!(
            live.group_heal,
            Some(crate::encounter::GroupHeal {
                percent: 50,
                kill: (7, 9)
            })
        );
        assert_eq!(live.nearby, vec![vec![2], vec![4]]);
        assert_eq!(live.broadcast.as_deref(), Some("staged notice"));
        assert!(live.pets_changed);
        // Nothing staged never wipes the live state.
        live.absorb(Outcome::default());
        assert!(live.dev_command_denied && live.changed && live.pets_changed);
        assert!(live.shared_field.is_some() && live.zoning.is_some());
        assert!(
            live.broadcast.is_some(),
            "an empty outcome wiped the notice"
        );
        assert_eq!(live.replies, vec![vec![1], vec![3]]);
    }
}
