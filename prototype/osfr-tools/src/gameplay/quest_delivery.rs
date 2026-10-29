//! T786 (R): a Talk step that hands items to an NPC.
//!
//! The hand-over is one transaction: the bag gives up `quantity` units of the step's item
//! definition and the quest step is credited together, or neither happens. The client never
//! names an item, a quantity or an NPC: `pending_delivery` reads all three from the active
//! ordered step of content the engine already validated, and the Talk's runtime actor only
//! chooses which NPC is being talked to.
use super::*;
use crate::quest_engine::{QuestEngine, QuestEvent, QuestOutcome};

/// The transaction behind one hand-over: consume on staged copies of the bag and the engine,
/// send the delivery event on the staged engine, and commit both only when every step
/// succeeded. `Err` leaves both values exactly as they were.
fn deliver_staged(
    inventory: &mut Inventory,
    engine: &mut QuestEngine,
    npc_id: u64,
    item_id: i32,
    quantity: u32,
) -> io::Result<(Vec<Vec<u8>>, QuestOutcome)> {
    let mut staged_inventory = inventory.clone();
    let consumed = staged_inventory.consume_definition(item_id, quantity)?;
    let mut staged_engine = engine.clone();
    let quest = staged_engine.on_event(QuestEvent::DeliverItem {
        npc_id,
        item_id,
        quantity,
    })?;
    // A valid event that credits no open step must not take the items either: the bag and the
    // progress move together or not at all.
    if !quest.changed {
        return Err(invalid("the delivery event credited no open step"));
    }
    *inventory = staged_inventory;
    *engine = staged_engine;
    Ok((consumed.replies, quest))
}

impl Gameplay {
    /// (R) T786: take this Talk step's items from the bag and credit the step together.
    /// A step that is not a delivery reads and changes nothing. A shortfall consumes nothing,
    /// sends no event, leaves one chat line and lets the Talk proceed as it does today.
    pub(super) fn take_delivery(&mut self, npc_id: u64, out: &mut Outcome) -> io::Result<()> {
        let Some((item_id, quantity)) = self
            .npc_quests
            .as_ref()
            .and_then(|engine| engine.pending_delivery(npc_id))
        else {
            return Ok(());
        };
        if self.material_count(item_id) < u64::from(quantity) {
            out.replies
                .push(chat::system("You do not have the items this step needs."));
            return Ok(());
        }
        let Self {
            inventory,
            npc_quests,
            ..
        } = self;
        let Some(engine) = npc_quests.as_mut() else {
            return Ok(());
        };
        let (replies, quest) = deliver_staged(inventory, engine, npc_id, item_id, quantity)?;
        out.replies.extend(replies);
        self.append_npc_quest_outcome_synced(quest, out)?;
        out.changed = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quest_engine::{
        CharacterEligibility, ObjectiveKind, QuestDefinition, QuestObjective, QuestProgress,
        QuestRequirements, QuestState, QuestStatus,
    };

    /// A real inventory from the opening reference content, the definition of its first
    /// non-rental stack (one unit), and an engine with one active quest per entry of
    /// `quantities`, each waiting on a hand-over of that many units to the same NPC.
    fn game_with_deliveries(quantities: &[u32]) -> (Gameplay, QuestEngine, i32) {
        let content = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../lab/references/opening-anvil-placement-v1/content");
        let game = Gameplay::load(&content, None).expect("the opening content loads");
        let state = game.state();
        let item_id = state
            .inventory
            .items
            .iter()
            .find(|item| item.rental.is_none())
            .expect("the opening bag holds a stack")
            .definition_id;
        let definitions = quantities
            .iter()
            .zip(10..)
            .map(|(&quantity, id)| QuestDefinition {
                id,
                title: "Delivery".into(),
                description: String::new(),
                giver_npc_id: 7,
                turnin_npc_id: 7,
                prerequisite_quest_ids: Vec::new(),
                requirements: QuestRequirements::default(),
                ordered: true,
                objectives: vec![QuestObjective {
                    id: 1,
                    required: 1,
                    description: "Objective 1".into(),
                    kind: ObjectiveKind::DeliverItem {
                        npc_id: 7,
                        item_id,
                        quantity,
                    },
                }],
                reward: crate::rewards::RewardGrant {
                    coins: 0,
                    items: Vec::new(),
                    job_xp: std::collections::BTreeMap::new(),
                    sc: 0,
                    display: crate::rewards::RewardDisplay {
                        source_guid: 7,
                        player_guid: 7,
                        reward_kind: 0,
                        icon_id: 0,
                        name_id: 0,
                        rows: Vec::new(),
                    },
                },
            });
        let engine = QuestEngine::new(
            7,
            definitions.collect(),
            CharacterEligibility::default(),
            Some(QuestState {
                version: 1,
                character_id: 7,
                quests: (10..)
                    .zip(quantities)
                    .map(|(id, _)| {
                        (
                            id,
                            QuestProgress {
                                status: QuestStatus::Active,
                                counts: vec![0],
                            },
                        )
                    })
                    .collect(),
                pending_offer: None,
                pending_turnin: None,
                pending_turnin_actor: None,
                selected_quest_id: None,
                auto_switch_selected: false,
            }),
        )
        .expect("the delivery quest definition loads");
        (game, engine, item_id)
    }

    /// T786 edge 2: the event is sent after the staged consume, so an event that is refused or
    /// credits nothing must leave the bag and the progress exactly as they were. The client
    /// cannot produce either event (the content loader bounds every field, and the engine builds
    /// the event from the open step), so the production transaction is called with the two events
    /// it defends against.
    #[test]
    fn failed_event_after_staged_consume_leaves_items_and_progress() {
        let (game, mut engine, item_id) = game_with_deliveries(&[1]);
        let mut inventory = game.inventory.clone();
        let before_items = inventory.state().clone();
        let before_quests = engine.state().clone();
        for (label, npc_id, quantity) in [
            ("an invalid event", 7, 0),
            ("a valid event for a step that is not open", 8, 1),
        ] {
            deliver_staged(&mut inventory, &mut engine, npc_id, item_id, quantity)
                .expect_err("the hand-over must not commit without a credit");
            assert_eq!(
                inventory.state(),
                &before_items,
                "{label}: the staged consume must not survive"
            );
            assert_eq!(
                engine.state(),
                &before_quests,
                "{label}: the event moved quest progress"
            );
        }
    }

    /// Review of attempt 2, finding 3: two open steps for the same NPC and item that differ in
    /// units. The Talk takes the first step's units and credits only the step with those units;
    /// the larger step stays open and becomes the next pending delivery.
    #[test]
    fn delivery_credits_only_the_step_with_the_units_taken() {
        let (game, mut engine, item_id) = game_with_deliveries(&[1, 2]);
        let mut inventory = game.inventory.clone();
        let before = inventory.state().items.iter().map(|i| i.count).sum::<u32>();
        assert_eq!(engine.pending_delivery(7), Some((item_id, 1)));

        deliver_staged(&mut inventory, &mut engine, 7, item_id, 1).expect("the 1-unit hand-over");

        let after = inventory.state().items.iter().map(|i| i.count).sum::<u32>();
        assert_eq!(before - after, 1, "only the first step's unit left the bag");
        assert_eq!(
            engine.state().quests[&10].counts,
            [1],
            "the 1-unit step is credited"
        );
        assert_eq!(
            engine.state().quests[&11].counts,
            [0],
            "the 2-unit step is not"
        );
        assert_eq!(engine.pending_delivery(7), Some((item_id, 2)));
    }
}
