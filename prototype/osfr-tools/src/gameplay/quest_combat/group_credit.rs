//! PB2: quest credit a group shares. The one call is in `npc_quest_event` (`credit`): an event
//! this player earned that a group shares waits in the ledger until the host hands it to every
//! eligible member (`take_group_credits`; who is eligible is `group_play.rs`), and that member's
//! own quests count it through `receive_group_credit` exactly as if earned there. A received
//! credit is never offered again, and each member gets each offered event once.
//!
//! | Event | Shared |
//! |---|---|
//! | `Kill` in a battle or dungeon run (`fight` set) | always: the shared arena credits only the killer. An open-world kill is not offered: the shared field already credits group members in reach (`FieldSim::party_near`), so it would count twice |
//! | `CollectNode` | always: a quest-only node one member picked |
//! | `FightGoal`, `Freed` | always: a fight goal or a captive one member advanced |
//! | `VisitPosition` | only when it moved this player's own step ("reaches an objective"); every move raises one |
//!
//! A new objective kind joins by adding its event to `shared`.
use super::*;
use crate::quest_engine::{QuestEngine, QuestOutcome};

/// (R) Most credits that wait for one host tick; the oldest are kept.
const MAX_PENDING: usize = 32;

#[derive(Clone, Debug, Default)]
pub(in crate::gameplay) struct Ledger {
    pending: Vec<QuestEvent>,
    receiving: bool,
}

/// `Some(true)`: shared whenever earned; `Some(false)`: only when it moved this player's step.
fn shared(event: &QuestEvent) -> Option<bool> {
    match event {
        QuestEvent::Kill { fact } => fact.fight.map(|_| true),
        QuestEvent::CollectNode { .. }
        | QuestEvent::FightGoal { .. }
        | QuestEvent::Freed { .. } => Some(true),
        QuestEvent::VisitPosition { .. } => Some(false),
        _ => None,
    }
}

/// Count `event` for this player and keep a copy for the group when it is shared.
pub(in crate::gameplay) fn credit(
    engine: &mut QuestEngine,
    event: QuestEvent,
    ledger: &mut Option<Box<Ledger>>,
) -> io::Result<QuestOutcome> {
    let receiving = ledger.as_ref().is_some_and(|l| l.receiving);
    let copy = (!receiving)
        .then(|| shared(&event).map(|always| (always, event.clone())))
        .flatten();
    let result = engine.on_event(event)?;
    if let Some((_, event)) = copy.filter(|(always, _)| *always || result.changed) {
        let ledger = ledger.get_or_insert_with(Box::default);
        if ledger.pending.len() < MAX_PENDING {
            ledger.pending.push(event);
        }
    }
    Ok(result)
}

impl Gameplay {
    /// PB2: the shared credits this player earned since the last call, emptied (the host's tick).
    pub fn take_group_credits(&mut self) -> Vec<QuestEvent> {
        (self.group_credit.as_mut()).map_or_else(Vec::new, |l| std::mem::take(&mut l.pending))
    }

    /// PB2: another member's credit, counted by this player's own quests as if earned here. A
    /// kill is this player's: their job, never solo, without the killer's ability or item, and
    /// it rolls this player's own quest drops.
    pub fn receive_group_credit(&mut self, event: QuestEvent) -> io::Result<Outcome> {
        let mut out = Outcome::default();
        self.group_credit.get_or_insert_with(Box::default).receiving = true;
        let result = match event {
            QuestEvent::Kill { mut fact } => {
                fact.solo = false;
                fact.job = (self.active_profile())
                    .and_then(|p| u32::try_from(p).ok())
                    .unwrap_or(0);
                (fact.ability, fact.item) = (None, None);
                self.credit_kill(fact, &mut out)
            }
            other => self.npc_quest_event(other, &mut out),
        };
        if let Some(ledger) = &mut self.group_credit {
            ledger.receiving = false;
        }
        result.map(|()| out)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::t013_objective_events::{game, install};
    use super::*;
    use crate::encounter::{KillFact, Rank};
    use crate::quest_engine::ObjectiveKind;

    const TROLL: u32 = 940_203;

    fn kill(fight: Option<u32>) -> QuestEvent {
        QuestEvent::Kill {
            fact: KillFact {
                kind: TROLL,
                guid: 77,
                rank: Rank::Normal,
                tags: Vec::new(),
                zone: 0,
                area: None,
                fight,
                solo: true,
                job: 43,
                ability: Some(62_019),
                item: None,
            },
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

    fn with_kill_step() -> Gameplay {
        let mut game = game();
        install(
            &mut game,
            ObjectiveKind::KillActorKind {
                actor_kind_id: TROLL,
            },
            5,
        );
        game
    }

    #[test]
    fn a_battle_kill_is_offered_once_and_an_open_world_kill_is_not() {
        let mut game = with_kill_step();
        let mut out = Outcome::default();
        game.npc_quest_event(kill(Some(12_520)), &mut out).unwrap();
        game.npc_quest_event(kill(None), &mut out).unwrap();
        assert_eq!(count(&game), 2, "both count for the killer");
        let offered = game.take_group_credits();
        assert_eq!(
            offered,
            vec![kill(Some(12_520))],
            "only the battle kill is shared"
        );
        assert!(game.take_group_credits().is_empty(), "taken once");
    }

    #[test]
    fn a_received_credit_counts_for_the_member_and_is_never_offered_again() {
        let mut member = with_kill_step();
        member.receive_group_credit(kill(Some(12_520))).unwrap();
        assert_eq!(
            count(&member),
            1,
            "the member's step counts the shared kill"
        );
        assert!(
            member.take_group_credits().is_empty(),
            "no echo back to the group"
        );
        // The member's own later kill is offered as usual.
        let mut out = Outcome::default();
        member
            .npc_quest_event(kill(Some(12_520)), &mut out)
            .unwrap();
        assert_eq!(member.take_group_credits().len(), 1);
    }

    #[test]
    fn a_collected_quest_node_is_shared_and_counts_on_the_members_step() {
        let node = || QuestEvent::CollectNode {
            node_type: "frost_lily".into(),
        };
        let mut picker = game();
        install(
            &mut picker,
            ObjectiveKind::CollectNode {
                node_type: "frost_lily".into(),
            },
            3,
        );
        picker
            .npc_quest_event(node(), &mut Outcome::default())
            .unwrap();
        let offered = picker.take_group_credits();
        assert_eq!(offered, vec![node()]);
        let mut member = game();
        install(
            &mut member,
            ObjectiveKind::CollectNode {
                node_type: "frost_lily".into(),
            },
            3,
        );
        let out = member.receive_group_credit(offered[0].clone()).unwrap();
        assert!(out.changed, "the member's progress is saved");
        assert_eq!(count(&member), 1);
    }

    #[test]
    fn a_member_on_another_step_gets_nothing() {
        let mut member = game();
        install(
            &mut member,
            ObjectiveKind::KillActorKind {
                actor_kind_id: TROLL + 1,
            },
            5,
        );
        member.receive_group_credit(kill(Some(12_520))).unwrap();
        assert_eq!(count(&member), 0);
    }

    #[test]
    fn a_position_is_shared_only_when_it_moved_the_players_own_step() {
        let mut game = with_kill_step();
        let visit = QuestEvent::VisitPosition {
            zone_id: 1,
            position: [0.0; 3],
        };
        game.npc_quest_event(visit, &mut Outcome::default())
            .unwrap();
        assert!(
            game.take_group_credits().is_empty(),
            "a move that counts nothing stays here"
        );
    }
}
