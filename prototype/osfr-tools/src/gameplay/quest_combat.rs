//! What a fight's outcome means for quests (combat design section 12). The engine reports
//! facts; this module turns each one into quest events. Moved out of
//! `Gameplay::apply_encounter` by the combat seam; piece P6 (`combat-quests`) owns this file.
//!
//! | Fact | Becomes |
//! |---|---|
//! | `KillFact` | the opening story's `Defeat`, a quest-drop roll, then `QuestEvent::Kill` |
//! | `FightFact::Used` | `QuestEvent::UseAbility` |
//! | `FightFact::Entered` | `QuestEvent::EnterFight` |
//! | `FightFact::Goal` | `QuestEvent::FightGoal` |
//! | `FightFact::Freed` | `QuestEvent::Freed` |
//! | `FightFact::Won` | `QuestEvent::CompleteDungeon` and the clear's earnings |
//! | `FightFact::Lost` | nothing |
//!
//! The three older lists (`defeated`, `abilities_used`, `win_completed`) carry the same events
//! and are no longer read: the engine pushes each of them together with its fact
//! (`record_kill`, `job_ability`, `win_run`), so reading both would count twice.
use super::*;
use crate::encounter::{FightFact, KillFact};
use crate::quest_engine::QuestEvent;

/// (R) Local copy: no recovered client locale row names a quest drop the bag could not take.
const QUEST_DROP_BAG_FULL: &str =
    "Your bag is full. Make room, then defeat another to collect that quest item.";

impl Gameplay {
    /// Credit one encounter outcome's combat to the opening story, NPC quests and earnings.
    pub(super) fn credit_combat(
        &mut self,
        _defeated: Vec<i32>,
        _abilities_used: Vec<(i32, Option<u32>)>,
        _win_completed: Option<u32>,
        kills: Vec<KillFact>,
        facts: Vec<FightFact>,
        out: &mut Outcome,
    ) -> io::Result<()> {
        for kill in kills {
            self.credit_kill(kill, out)?;
        }
        for fact in facts {
            match fact {
                FightFact::Used { ability, on_kind } => {
                    self.emit_ability_uses(vec![(ability, on_kind)], out)?;
                }
                FightFact::Entered { fight } => {
                    self.npc_quest_event(QuestEvent::EnterFight { fight_id: fight }, out)?;
                }
                FightFact::Goal {
                    fight,
                    goal,
                    done,
                    of,
                } => {
                    // A goal with no size or no name counts nothing; never an error the
                    // session would see.
                    if of != 0 && crate::content::valid_fight_goal(&goal) {
                        self.npc_quest_event(
                            QuestEvent::FightGoal {
                                fight_id: fight,
                                goal,
                                done,
                                of,
                            },
                            out,
                        )?;
                    }
                }
                FightFact::Freed { kind, guid } => {
                    if kind != 0 || guid != 0 {
                        self.npc_quest_event(
                            QuestEvent::Freed {
                                actor_kind_id: kind,
                                guid,
                            },
                            out,
                        )?;
                    }
                }
                FightFact::Won { fight } => {
                    self.npc_quest_event(
                        QuestEvent::CompleteDungeon {
                            encounter_id: fight,
                        },
                        out,
                    )?;
                    // T229: the win transition (once per run, with the win grants) is the clear.
                    self.pay_earned_station_cash(EarnSource::DungeonClear { id: fight }, out)?;
                }
                FightFact::Lost { .. } => {}
            }
        }
        Ok(())
    }

    /// One credited kill. `record_kill` never reports kind 0.
    fn credit_kill(&mut self, kill: KillFact, out: &mut Outcome) -> io::Result<()> {
        if let Ok(definition_id) = i32::try_from(kill.kind) {
            let result = self
                .opening_mut()?
                .on_event(OpeningEvent::Defeat { definition_id })?;
            self.apply_opening(result, out)?;
        }
        // The drop is rolled before the kill is counted: a kill that finishes "defeat five"
        // and opens "collect five" was made before the collect step, so it drops nothing.
        self.quest_drops(kill.kind, out)?;
        // Ticks and requests both report kills here; `npc_quest_event` registers any goal
        // this defeat opened before the outcome is returned.
        self.npc_quest_event(QuestEvent::Kill { fact: kill }, out)
    }

    /// Roll this kind's quest drops for the steps that are open right now. A catalogued item
    /// goes into the bag, where the bag change credits the step like any other pickup; an
    /// item id with no bag item credits the step directly.
    fn quest_drops(&mut self, kind: u32, out: &mut Outcome) -> io::Result<()> {
        let (Some(encounter), Some(quests)) = (&mut self.encounter, &self.npc_quests) else {
            return Ok(());
        };
        let dropped =
            encounter.roll_quest_drops(kind, |quest, item| quests.wants_quest_drop(quest, item));
        for item_id in dropped {
            if !self.inventory.has_definition(item_id) {
                self.npc_quest_event(
                    QuestEvent::CollectItem {
                        item_id,
                        quantity: 1,
                    },
                    out,
                )?;
            } else if self.inventory.accepts_reward_bundle(&[(item_id, 1)]) {
                self.grant_reward(0, &[(item_id, 1)], out)?;
            } else {
                out.replies.push(chat::system(QUEST_DROP_BAG_FULL));
            }
        }
        Ok(())
    }
}

/// Facts the fight engine does not report yet (`ability`, `item`, `area`, fight goals, freed
/// captives) have no request path to walk, so these tests hand them to `credit_combat`, the
/// one place every outcome's facts arrive. The kinds a request can reach today are walked in
/// `tests/quest_combat_steps.rs` and `tests/quest_drops.rs`.
#[cfg(test)]
mod tests {
    use super::super::t013_objective_events::{game, install};
    use super::*;
    use crate::encounter::Rank;
    use crate::quest_engine::{KillRule, ObjectiveKind};

    const TROLL: u32 = 940_203;
    const FIGHT: u32 = 12_520;

    fn kill(kind: u32) -> KillFact {
        KillFact {
            kind,
            guid: 77,
            rank: Rank::Normal,
            tags: Vec::new(),
            zone: 0,
            area: None,
            fight: None,
            solo: true,
            job: 43,
            ability: None,
            item: None,
        }
    }

    fn count(game: &Gameplay) -> u32 {
        game.npc_quests
            .as_ref()
            .unwrap()
            .progress(1)
            .unwrap()
            .counts[0]
    }

    fn credit(game: &mut Gameplay, kills: Vec<KillFact>, facts: Vec<FightFact>) -> u32 {
        game.credit_combat(
            Vec::new(),
            Vec::new(),
            None,
            kills,
            facts,
            &mut Outcome::default(),
        )
        .unwrap();
        count(game)
    }

    fn with_rule(rule: KillRule, required: u32) -> Gameplay {
        let mut game = game();
        install(&mut game, ObjectiveKind::Kill { rule }, required);
        game
    }

    #[test]
    fn a_kill_with_the_named_ability_item_or_area_counts_and_others_do_not() {
        let cases: [(KillRule, fn(&mut KillFact)); 3] = [
            (
                KillRule {
                    kinds: vec![TROLL],
                    ability: Some(62_019),
                    ..Default::default()
                },
                |fact| fact.ability = Some(62_019),
            ),
            (
                KillRule {
                    kinds: vec![TROLL],
                    item: Some(11_002),
                    ..Default::default()
                },
                |fact| fact.item = Some(11_002),
            ),
            (
                KillRule {
                    area: Some(314),
                    ..Default::default()
                },
                |fact| fact.area = Some(314),
            ),
        ];
        for (rule, meet) in cases {
            let mut game = with_rule(rule, 2);
            assert_eq!(
                credit(&mut game, vec![kill(TROLL)], vec![]),
                0,
                "plain kill"
            );
            let mut met = kill(TROLL);
            meet(&mut met);
            assert_eq!(credit(&mut game, vec![met], vec![]), 1, "the condition met");
        }
    }

    #[test]
    fn the_older_lists_are_not_read_so_nothing_counts_twice() {
        let mut game = game();
        install(
            &mut game,
            ObjectiveKind::KillActorKind {
                actor_kind_id: TROLL,
            },
            5,
        );
        // The engine fills `defeated` and `kills` together: one kill is one count.
        game.credit_combat(
            vec![TROLL as i32],
            Vec::new(),
            None,
            vec![kill(TROLL)],
            Vec::new(),
            &mut Outcome::default(),
        )
        .unwrap();
        assert_eq!(count(&game), 1);
        // A list entry without its fact is not a kill any more.
        game.credit_combat(
            vec![TROLL as i32],
            Vec::new(),
            None,
            Vec::new(),
            Vec::new(),
            &mut Outcome::default(),
        )
        .unwrap();
        assert_eq!(count(&game), 1);
    }

    fn goal(done: u32, of: u32) -> FightFact {
        FightFact::Goal {
            fight: FIGHT,
            goal: "hold".into(),
            done,
            of,
        }
    }

    fn with_fight_goal(required: u32) -> Gameplay {
        let mut game = game();
        install(
            &mut game,
            ObjectiveKind::FightGoal {
                fight_id: FIGHT,
                goal: "hold".into(),
            },
            required,
        );
        game
    }

    #[test]
    fn a_counted_fight_goal_follows_the_fights_own_counter() {
        // "Defend Darkthorne from the Gloam 0/3".
        let mut game = with_fight_goal(3);
        assert_eq!(credit(&mut game, vec![], vec![goal(1, 3)]), 1);
        assert_eq!(
            credit(&mut game, vec![], vec![goal(1, 3)]),
            1,
            "a repeat is no count"
        );
        let other = FightFact::Goal {
            fight: FIGHT,
            goal: "escort".into(),
            done: 3,
            of: 3,
        };
        assert_eq!(credit(&mut game, vec![], vec![other]), 1, "another goal");
        let elsewhere = FightFact::Goal {
            fight: FIGHT + 1,
            goal: "hold".into(),
            done: 3,
            of: 3,
        };
        assert_eq!(
            credit(&mut game, vec![], vec![elsewhere]),
            1,
            "another fight"
        );
        // A lost fight starts its counter again; the step keeps what it had.
        assert_eq!(
            credit(&mut game, vec![], vec![FightFact::Lost { fight: FIGHT }]),
            1
        );
        assert_eq!(credit(&mut game, vec![], vec![goal(1, 3)]), 1);
        assert_eq!(credit(&mut game, vec![], vec![goal(2, 3), goal(3, 3)]), 3);
    }

    #[test]
    fn a_small_fight_goal_counts_once_each_time_it_is_finished() {
        // "Survive the ambush" three times: the fight's goal is 1 of 1 per run.
        let mut game = with_fight_goal(3);
        assert_eq!(
            credit(&mut game, vec![], vec![goal(0, 1)]),
            0,
            "not finished"
        );
        assert_eq!(credit(&mut game, vec![], vec![goal(1, 1)]), 1);
        assert_eq!(credit(&mut game, vec![], vec![goal(1, 1)]), 2);
        // A malformed fact counts nothing and is not an error.
        assert_eq!(credit(&mut game, vec![], vec![goal(1, 0)]), 2);
    }

    #[test]
    fn a_guarded_captive_is_refused_on_a_click_and_credited_when_freed() {
        const CAPTIVE: u64 = 100_000_777_001;
        let mut game = game();
        install(
            &mut game,
            ObjectiveKind::FreeCaptive {
                guid: CAPTIVE,
                actor_kind_id: None,
            },
            1,
        );
        // The click itself (what an accepted menu interaction raises) frees nobody while the
        // guards stand: only the fight can say the captive is free.
        game.npc_quest_event(
            QuestEvent::InteractNpc { npc_id: CAPTIVE },
            &mut Outcome::default(),
        )
        .unwrap();
        assert_eq!(count(&game), 0, "a click on a guarded captive is refused");
        let other = FightFact::Freed {
            kind: TROLL,
            guid: CAPTIVE + 1,
        };
        assert_eq!(credit(&mut game, vec![], vec![other]), 0, "another captive");
        let freed = FightFact::Freed {
            kind: TROLL,
            guid: CAPTIVE,
        };
        assert_eq!(credit(&mut game, vec![], vec![freed]), 1);
    }

    #[test]
    fn captives_of_a_kind_are_counted_one_per_freed_actor() {
        let mut game = game();
        install(
            &mut game,
            ObjectiveKind::FreeCaptive {
                guid: 0,
                actor_kind_id: Some(TROLL),
            },
            2,
        );
        let freed = |kind, guid| FightFact::Freed { kind, guid };
        assert_eq!(credit(&mut game, vec![], vec![freed(TROLL + 1, 5)]), 0);
        assert_eq!(
            credit(&mut game, vec![], vec![freed(TROLL, 5), freed(TROLL, 6)]),
            2
        );
    }

    #[test]
    fn a_freed_content_actor_finishes_a_plain_interact_step_on_it() {
        let mut game = game();
        install(&mut game, ObjectiveKind::InteractNpc { npc_id: 4_242 }, 1);
        let freed = FightFact::Freed {
            kind: TROLL,
            guid: 4_242,
        };
        assert_eq!(credit(&mut game, vec![], vec![freed]), 1);
    }
}
