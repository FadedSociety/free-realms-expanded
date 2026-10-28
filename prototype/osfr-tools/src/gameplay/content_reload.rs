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
//! In particular `npc_interaction` is not reloaded: `NpcInteraction::load` runs only
//! in `Gameplay::load`, so a vendor role or `vendors.json` mapping fix needs a restart.
//! T753: `regions.json` belongs to `NpcVisibility`; `reload regions` and `reload all`
//! re-read it on the staged clone and publish it with the same commit.
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

    /// Loads in place with no rollback: only `reload_content` runs it on live
    /// state. `pub(super)` so the protected regression can read what a refusal drops.
    pub(super) fn reload_content_in_place(
        &mut self,
        what: &str,
        out: &mut Outcome,
    ) -> io::Result<String> {
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
                    self.activities = crate::activity_portal::Catalogue::assemble(
                        Some(&path),
                        self.chess.as_ref(),
                        self.soccer_content.as_ref(),
                        self.activities.cards(),
                    )?;
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
            // Same bounded reader and refusal as the watch and the commands.
            let records = self.load_placements()?;
            if let Some(visibility) = self.npc_visibility.as_mut() {
                out.replies.extend(visibility.replace_placements(&records)?);
                done.push("placed NPCs");
            }
        }
        if (all && self.npc_visibility.is_some()) || what == "regions" {
            // T753: `content_dir` is already the NPC content set `NpcVisibility::load`
            // bound, so a save directory's bundled fallback is not re-resolved here.
            let visibility = self
                .npc_visibility
                .as_mut()
                .ok_or_else(|| invalid("regions requires the NPC content set"))?;
            visibility.reload_regions(&dir)?;
            done.push("regions");
        }
        if done.is_empty() {
            return Err(invalid(
                "unknown target; use realm reload [all|battles|enemies|abilities|behaviours|power_ups|placements|regions]",
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
            granted_items: vec![(41, 2)],
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
            admin_move: Some(AdminMove {
                target: MoveTarget::Guid(9),
                position: [4.0, 5.0, 6.0],
                rotation: None,
            }),
            world_event: Some(crate::world_events::WorldEventCommand::Start("show".into())),
            boombox: Some(878),
            house_broadcast: Some(crate::housing_service::HouseBroadcast {
                house_guid: 12,
                lot: crate::housing::LotSnapshot {
                    owner_guid: 1,
                    property_id: 2,
                    lot_guid: 12,
                    zone_id: 3,
                    name_id: 4,
                    max_fixtures: 5,
                    max_landmarks: 6,
                    count: 0,
                    build_areas: Vec::new(),
                    fixtures: Vec::new(),
                    permissions: Vec::new(),
                    locked: false,
                },
                packets: vec![vec![5]],
            }),
        }
    }

    /// T872: two edits of one lot in a bundle reach its visitors as both edits' packets in order
    /// with the later snapshot; an edit of another lot replaces the earlier one.
    #[test]
    fn house_broadcasts_of_one_lot_merge_in_order() {
        use crate::housing_service::HouseBroadcast;
        let edit = |house_guid, packet: u8, count| {
            let mut broadcast = staged_outcome().house_broadcast.unwrap();
            broadcast.house_guid = house_guid;
            broadcast.lot.count = count;
            broadcast.packets = vec![vec![packet]];
            Some(broadcast)
        };
        let merged = HouseBroadcast::merge(edit(12, 1, 1), edit(12, 2, 2)).unwrap();
        assert_eq!(merged.packets, vec![vec![1], vec![2]]);
        assert_eq!(merged.lot.count, 2);
        let other = HouseBroadcast::merge(edit(12, 1, 1), edit(13, 2, 2)).unwrap();
        assert_eq!((other.house_guid, other.packets), (13, vec![vec![2]]));
        assert!(HouseBroadcast::merge(edit(12, 1, 1), None).is_some());
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
        assert_eq!(live.granted_items, vec![(41, 2)]);
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
        assert_eq!(live.admin_move.as_ref().unwrap().position, [4.0, 5.0, 6.0]);
        assert_eq!(
            live.world_event,
            Some(crate::world_events::WorldEventCommand::Start("show".into()))
        );
        assert_eq!(live.boombox, Some(878));
        assert_eq!(
            live.house_broadcast
                .as_ref()
                .map(|b| (b.house_guid, &b.packets)),
            Some((12, &vec![vec![5]]))
        );
        // Nothing staged never wipes the live state.
        live.absorb(Outcome::default());
        assert!(live.dev_command_denied && live.changed);
        assert!(live.shared_field.is_some() && live.zoning.is_some());
        assert!(live.scene_zoning.is_some() && live.relocation.is_some());
        assert!(live.group_heal.is_some());
        assert_eq!(live.granted_items, vec![(41, 2)]);
        assert!(
            live.broadcast.is_some()
                && live.admin_move.is_some()
                && live.world_event.is_some()
                && live.boombox.is_some(),
            "an empty outcome wiped the notice, the move, the world event or the boombox"
        );
        assert_eq!(live.replies, vec![vec![1], vec![3]]);
    }

    /// T753: a `regions` reload stages `regions.json` and publishes it only when the
    /// whole file validates; a malformed file leaves the loaded table in place.
    #[test]
    fn regions_reload_swaps_the_table_and_refuses_a_malformed_one() {
        struct Scratch(PathBuf);
        impl Scratch {
            fn new() -> Self {
                let path = std::env::temp_dir()
                    .join(format!("osfr-t753-regions-reload-{}", std::process::id()));
                let _ = std::fs::remove_dir_all(&path);
                std::fs::create_dir_all(&path).unwrap();
                Self(path)
            }
        }
        impl Drop for Scratch {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }

        let reference = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../lab/references/rust-opening-cooking-original-assets");
        assert!(
            reference.join("inventory.json").is_file(),
            "the opening reference is required at {}",
            reference.display()
        );
        let mut game = Gameplay::load(&reference, None).expect("opening reference loads");
        assert_eq!(
            game.region_table()
                .unwrap()
                .region(8)
                .unwrap()
                .name_string_id,
            3961
        );

        let scratch = Scratch::new();
        let mut value: serde_json::Value = serde_json::from_slice(
            &std::fs::read(crate::content::bundled_content().join("regions.json")).unwrap(),
        )
        .unwrap();
        value["regions"][7]["name_string_id"] = 3500.into();
        std::fs::write(
            scratch.0.join("regions.json"),
            serde_json::to_vec(&value).unwrap(),
        )
        .unwrap();
        game.content_dir = scratch.0.clone();
        let mut out = Outcome::default();
        assert_eq!(game.reload_content("regions", &mut out).unwrap(), "regions");
        assert_eq!(
            game.region_table()
                .unwrap()
                .region(8)
                .unwrap()
                .name_string_id,
            3500
        );

        std::fs::write(scratch.0.join("regions.json"), b"{ not json").unwrap();
        let error = game.reload_content("regions", &mut out).unwrap_err();
        assert!(error.to_string().contains("regions.json"), "{error}");
        assert_eq!(
            game.region_table()
                .unwrap()
                .region(8)
                .unwrap()
                .name_string_id,
            3500,
            "a refused reload left the previous table in place"
        );
    }
}
