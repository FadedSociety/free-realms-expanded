//! Builder tests for work item 057, "Loot and enemy rewards".
//!
//! The module is not registered in `src/lib.rs` (a shared file owned by the lead), so this test
//! target compiles the exact source with `#[path]`, as the other piece-NNN targets do. Lead
//! integration step: register `restoration::piece_057` and drop the include.
//!
//! These tests assert the recovered original evidence (client loot-event opcode, loot prop model
//! ids, loot toast locale ids, the five wiki-documented enemy drop lists with their explicit
//! gaps), the pinned community weighted-selection semantics, and the reconstructed once-only
//! kill/loot grant contract. They never claim native behaviour and never invent probabilities.

#[path = "../src/restoration/piece_057.rs"]
mod piece_057;

use piece_057::{
    COINS_NAME_ID, COMMUNITY_DOCUMENTED_DROPS, DocumentedKind, DropTable, KillId,
    LOOT_BAG_BIG_MODEL_ID, LOOT_BAG_SMALL_MODEL_ID, LOOT_CHEST_MODEL_ID, LOOT_COINS_MODEL_ID,
    LOOT_TOAST_EXPERIENCE_COINS_LOCALE_ID, LOOT_TOAST_ITEM_LOCALE_ID, LootGap, LootLedger,
    LootOutcome, LootProp, LootRequest, PLAYER_UPDATE_LOOT_EVENT, RewardTarget, TableError,
    WeightedDrop, loot_props, weighted_from_documented,
};

#[test]
fn loot_event_opcode_is_the_original_client_sub_packet() {
    // Original client `PlayerUpdatePacketLootEvent` constructor table: `FUN_0089fbb0(0x1d)`.
    assert_eq!(PLAYER_UPDATE_LOOT_EVENT, 29);
}

#[test]
fn loot_prop_models_are_recovered_asset_rows() {
    assert_eq!(LOOT_BAG_SMALL_MODEL_ID, 808);
    assert_eq!(LOOT_BAG_BIG_MODEL_ID, 807);
    assert_eq!(LOOT_COINS_MODEL_ID, 841);
    assert_eq!(LOOT_CHEST_MODEL_ID, 720);
    assert_eq!(LootProp::SmallBag.model_id(), LOOT_BAG_SMALL_MODEL_ID);
    assert_eq!(LootProp::BigBag.model_id(), LOOT_BAG_BIG_MODEL_ID);
    assert_eq!(LootProp::Coins.model_id(), LOOT_COINS_MODEL_ID);
    assert_eq!(LootProp::Chest.model_id(), LOOT_CHEST_MODEL_ID);
}

#[test]
fn loot_toast_locale_ids_are_recovered() {
    // Original locale rows 2 and 3 carry the "You receive ..." loot text.
    assert_eq!(LOOT_TOAST_ITEM_LOCALE_ID, 2);
    assert_eq!(LOOT_TOAST_EXPERIENCE_COINS_LOCALE_ID, 3);
    assert_eq!(COINS_NAME_ID, 4923);
}

#[test]
fn documented_drop_lists_are_recorded_verbatim_with_gaps() {
    assert_eq!(COMMUNITY_DOCUMENTED_DROPS.len(), 5);
    let floren = COMMUNITY_DOCUMENTED_DROPS[0];
    assert_eq!(floren.npc, "Floren Defender");
    assert_eq!(floren.source_page_id, 1035);
    assert_eq!(floren.source_revision_id, 48775);
    assert_eq!(floren.drops.len(), 18);
    assert_eq!(floren.drops[0].label, "Coins");
    assert_eq!(floren.drops[0].kind, DocumentedKind::Coins);
    assert_eq!(floren.drops[0].name_id, Some(COINS_NAME_ID));
    assert_eq!(floren.drops[0].item_definition_id, None);
    assert_eq!(floren.drops[1].name_id, Some(442929));
    assert_eq!(floren.drops[3].name_id, None, "no locale row for this name");
    assert_eq!(floren.drops[14].label, "Wilds Small Pillar");
    assert_eq!(floren.drops[14].name_id, Some(442067));
    assert_eq!(floren.drops[15].label, "Wilds Wall");
    assert_eq!(floren.drops[15].name_id, Some(442064));

    let magus = COMMUNITY_DOCUMENTED_DROPS[1];
    assert_eq!(magus.npc, "Sandscale Magus");
    assert_eq!(magus.source_revision_id, 45373);
    assert_eq!(magus.drops.len(), 2);
    assert_eq!(magus.drops[1].item_definition_id, Some(18172));

    let scavenger = COMMUNITY_DOCUMENTED_DROPS[2];
    assert_eq!(scavenger.npc, "Sandscale Scavenger");
    assert_eq!(scavenger.drops.len(), 5);
    assert_eq!(scavenger.drops[1].name_id, None);
    assert_eq!(scavenger.drops[4].name_id, Some(34006));

    let boss = COMMUNITY_DOCUMENTED_DROPS[3];
    assert_eq!(boss.npc, "Sandspear");
    assert_eq!(boss.source_revision_id, 50196);
    assert_eq!(boss.drops.len(), 5);

    let skullz = COMMUNITY_DOCUMENTED_DROPS[4];
    assert_eq!(skullz.npc, "Skullz Roughrider");
    assert_eq!(skullz.drops.len(), 5);
    assert_eq!(skullz.drops[2].item_definition_id, None);
    assert_eq!(skullz.drops[3].item_definition_id, Some(38822));
    assert_eq!(skullz.drops[4].item_definition_id, Some(38827));
}

#[test]
fn documented_lists_refuse_to_become_weighted_tables() {
    for list in COMMUNITY_DOCUMENTED_DROPS {
        assert_eq!(
            list.blocking_gaps(),
            &[
                LootGap::NoProbabilityData,
                LootGap::NoQuantityData,
                LootGap::NoAmountData,
            ]
        );
        assert_eq!(
            weighted_from_documented(&list),
            Err(LootGap::NoProbabilityData),
            "the wiki records no weights, so this module must not invent any"
        );
    }
}

#[test]
fn pinned_weighted_selection_matches_community_semantics() {
    let table = DropTable::new(
        "dev-test-reward",
        vec![WeightedDrop::item(205, 1, 1), WeightedDrop::coins(50, 1)],
    );
    assert_eq!(table.validate(), Ok(()));
    assert_eq!(table.total_weight(), Ok(2));
    assert_eq!(
        table.select(0).unwrap().target,
        RewardTarget::Item {
            item_definition_id: 205,
            quantity: 1
        }
    );
    assert_eq!(
        table.select(1).unwrap().target,
        RewardTarget::Coins { amount: 50 }
    );
    assert_eq!(
        table.select(2),
        Err(TableError::RollOutOfRange { roll: 2, total: 2 })
    );
}

#[test]
fn weighted_tables_reject_the_invalid_shapes_the_pinned_loader_rejects() {
    assert_eq!(
        DropTable::new("empty", Vec::new()).validate(),
        Err(TableError::Empty)
    );
    assert_eq!(
        DropTable::new("zero", vec![WeightedDrop::item(205, 1, 0)]).validate(),
        Err(TableError::ZeroWeight { index: 0 })
    );
    assert_eq!(
        DropTable::new("qty", vec![WeightedDrop::item(205, 0, 1)]).validate(),
        Err(TableError::NonPositiveQuantity { index: 0 })
    );
    assert_eq!(
        DropTable::new("coins", vec![WeightedDrop::coins(0, 1)]).validate(),
        Err(TableError::NonPositiveAmount { index: 0 })
    );
    // Keys are trimmed and lowercased exactly like the pinned community loader.
    assert_eq!(
        DropTable::new(" Dev-Test-Reward ", Vec::new()).key,
        "dev-test-reward"
    );
}

#[test]
fn a_kill_is_registered_once_and_granted_once() {
    let kill = KillId::new(7, 0x5255_5354_0000_0002, 1);
    let mut ledger = LootLedger::new();
    assert!(ledger.register_defeat(kill));
    assert!(
        !ledger.register_defeat(kill),
        "repeat defeat messages are ignored"
    );

    let table = DropTable::new(
        "dev-test-reward",
        vec![WeightedDrop::item(205, 1, 1), WeightedDrop::coins(50, 1)],
    );
    let request = LootRequest::new(kill, 1, 1, 0);
    assert_eq!(
        ledger.grant(request, &table),
        LootOutcome::Granted {
            drop: WeightedDrop::item(205, 1, 1)
        }
    );
    assert_eq!(ledger.grant(request, &table), LootOutcome::Duplicate);
    assert_eq!(ledger.granted_kills(), 1);
    assert_eq!(ledger.granted(&kill), Some(WeightedDrop::item(205, 1, 1)));

    // A second kill of the same enemy is a separate grant.
    let second = KillId::new(7, 0x5255_5354_0000_0002, 2);
    assert!(ledger.register_defeat(second));
    assert_eq!(
        ledger.grant(LootRequest::new(second, 1, 1, 1), &table),
        LootOutcome::Granted {
            drop: WeightedDrop::coins(50, 1)
        }
    );
    assert_eq!(ledger.granted_kills(), 2);
}

#[test]
fn grants_require_the_killer_and_a_registered_defeat() {
    let kill = KillId::new(7, 0x5255_5354_0000_0002, 1);
    let table = DropTable::new("dev-test-reward", vec![WeightedDrop::item(205, 1, 1)]);
    let mut ledger = LootLedger::new();

    assert_eq!(
        ledger.grant(LootRequest::new(kill, 1, 1, 0), &table),
        LootOutcome::UnregisteredKill
    );
    assert!(ledger.register_defeat(kill));
    assert_eq!(
        ledger.grant(LootRequest::new(kill, 1, 2, 0), &table),
        LootOutcome::NotEligible {
            killer: 1,
            claimant: 2
        }
    );
    assert_eq!(ledger.granted(&kill), None);
    assert_eq!(ledger.granted_kills(), 0);
    assert_eq!(
        ledger.grant(LootRequest::new(kill, 1, 1, 0), &table),
        LootOutcome::Granted {
            drop: WeightedDrop::item(205, 1, 1)
        }
    );
}

#[test]
fn invalid_tables_are_refused_without_consuming_the_kill() {
    let kill = KillId::new(7, 1, 1);
    let mut ledger = LootLedger::new();
    assert!(ledger.register_defeat(kill));
    let broken = DropTable::new("broken", vec![WeightedDrop::item(205, 1, 0)]);
    assert_eq!(
        ledger.grant(LootRequest::new(kill, 1, 1, 0), &broken),
        LootOutcome::InvalidTable(TableError::ZeroWeight { index: 0 })
    );
    assert_eq!(ledger.granted(&kill), None);
    assert_eq!(ledger.granted_kills(), 0);
}

#[test]
fn loot_prop_policy_is_labelled_reconstruction() {
    assert_eq!(loot_props(false), &[LootProp::SmallBag, LootProp::Coins]);
    assert_eq!(loot_props(true), &[LootProp::Chest]);
}
