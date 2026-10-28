use osfr_tools::{
    quest_engine::{ObjectiveKind, QuestObjective},
    quest_journal::{Pin, Places, Tracker, locate, tracked_pin},
};

const NPC: u64 = 9_500_000_254;
const ENEMY_GUID: u64 = 100_000_004_154;
const ENEMY_KIND: u32 = 930_050;
const POI: u32 = 77;
/// Sheep Watch (`field_battles.json` 12350): marker guid and position, as shipped.
const BATTLE: u32 = 12_350;
const MARKER_GUID: u64 = 100_001_235_000;
const MARKER: [f32; 3] = [-2165.513, -32.96, 328.1112];
/// Battle 96's own kind: it fights only inside that battle, never in the field.
const BATTLE_KIND: u32 = 930_096;
const BATTLE_96_GUID: u64 = 100_000_096_000;
const BATTLE_96: [f32; 3] = [-440.0, 0.8, 1290.0];

fn objective(id: u32, kind: ObjectiveKind) -> QuestObjective {
    QuestObjective {
        id,
        required: 1,
        description: String::new(),
        kind,
    }
}

/// One NPC, one kind of field enemy, one POI and one battle exist; everything else is
/// unknown. The field kind also fights in that battle (930050 is in 12350's roster), and
/// one other kind fights only in battle 96.
fn with_places<R>(f: impl FnOnce(&Places) -> R) -> R {
    let npc = |id: u64| (id == NPC).then_some([-1909.5, -44.4, 462.8]);
    let enemy = |kind: u32| (kind == ENEMY_KIND).then_some((ENEMY_GUID, [-1971.4, -34.5, 539.8]));
    let poi = |id: u32| (id == POI).then_some((5, [10.0, 2.0, -30.0]));
    let battle = |id: u32| (id == BATTLE).then_some((MARKER_GUID, MARKER));
    let battle_enemy = |kind: u32| match kind {
        ENEMY_KIND => Some((MARKER_GUID, MARKER)),
        BATTLE_KIND => Some((BATTLE_96_GUID, BATTLE_96)),
        _ => None,
    };
    f(&Places {
        npc: &npc,
        enemy: &enemy,
        poi: &poi,
        battle: &battle,
        battle_enemy: &battle_enemy,
    })
}

fn pin(objective_id: u32, guid: u64, zone_id: u32, position: [f32; 3]) -> Pin {
    Pin {
        quest_id: 3_000_021,
        objective_id,
        guid,
        zone_id,
        position,
    }
}

fn locate_kind(kind: ObjectiveKind) -> Option<Pin> {
    with_places(|places| locate(3_000_021, &objective(4, kind), places))
}

#[test]
fn active_pin_has_the_recovered_47_14_layout() {
    let packet = Pin::encode(Some(&pin(9, 0x0102_0304_0506_0708, 5, [1.5, -2.5, 3.5])));
    // u16 47, u8 14, u8 active, 3 dwords, u64 guid, name id, vec4, zone = 3 + 45 bytes.
    assert_eq!(packet.len(), 48);
    assert_eq!(&packet[..4], &[47, 0, 14, 1]);
    let dword = |at: usize| u32::from_le_bytes(packet[at..at + 4].try_into().unwrap());
    let float = |at: usize| f32::from_le_bytes(packet[at..at + 4].try_into().unwrap());
    assert_eq!(dword(4), 3_000_021, "first task dword is the quest id (R)");
    assert_eq!(dword(8), 0);
    assert_eq!(dword(12), 9, "third task dword is the objective id (R)");
    assert_eq!(
        u64::from_le_bytes(packet[16..24].try_into().unwrap()),
        0x0102_0304_0506_0708
    );
    assert_eq!(
        dword(24),
        0,
        "name string id: the client shows its fallback"
    );
    assert_eq!(
        [float(28), float(32), float(36), float(40)],
        [1.5, -2.5, 3.5, 1.0],
        "x, y, z and the client's own w of 1.0"
    );
    assert_eq!(dword(44), 5);
}

#[test]
fn no_pin_is_the_four_byte_clear() {
    assert_eq!(Pin::encode(None), vec![47, 0, 14, 0]);
}

#[test]
fn talk_and_interact_objectives_point_at_the_npc() {
    let expected = pin(4, NPC, 0, [-1909.5, -44.4, 462.8]);
    assert_eq!(
        locate_kind(ObjectiveKind::TalkToNpc { npc_id: NPC }),
        Some(expected)
    );
    assert_eq!(
        locate_kind(ObjectiveKind::InteractNpc { npc_id: NPC }),
        Some(expected)
    );
}

#[test]
fn reach_poi_and_visit_position_use_their_own_geometry() {
    assert_eq!(
        locate_kind(ObjectiveKind::ReachPoi { poi_id: POI }),
        Some(pin(4, 0, 5, [10.0, 2.0, -30.0]))
    );
    assert_eq!(
        locate_kind(ObjectiveKind::VisitPosition {
            zone_id: 6,
            position: [1.0, 2.0, 3.0],
            radius: 4.0,
        }),
        Some(pin(4, 0, 6, [1.0, 2.0, 3.0]))
    );
}

#[test]
fn dungeon_objectives_point_at_their_battle_marker() {
    assert_eq!(
        locate_kind(ObjectiveKind::CompleteDungeon {
            encounter_id: BATTLE
        }),
        Some(pin(4, MARKER_GUID, 0, MARKER))
    );
}

#[test]
fn a_battle_only_kill_points_at_its_battle_marker() {
    assert_eq!(
        locate_kind(ObjectiveKind::KillActorKind {
            actor_kind_id: BATTLE_KIND
        }),
        Some(pin(4, BATTLE_96_GUID, 0, BATTLE_96))
    );
}

#[test]
fn a_field_enemy_still_wins_over_a_battle_marker() {
    // 930050 has both a field home and a battle; the field home is where it roams (this is also
    // the plain field-enemy kill pin).
    assert_eq!(
        locate_kind(ObjectiveKind::KillActorKind {
            actor_kind_id: ENEMY_KIND
        }),
        Some(pin(4, ENEMY_GUID, 0, [-1971.4, -34.5, 539.8]))
    );
}

#[test]
fn unknown_battles_and_battle_kinds_pin_nothing() {
    for kind in [
        ObjectiveKind::CompleteDungeon { encounter_id: 0 },
        ObjectiveKind::CompleteDungeon {
            encounter_id: BATTLE + 1,
        },
        ObjectiveKind::KillActorKind { actor_kind_id: 0 },
        ObjectiveKind::KillActorKind {
            actor_kind_id: BATTLE_KIND + 1,
        },
    ] {
        assert_eq!(locate_kind(kind.clone()), None, "{kind:?}");
    }
}

#[test]
fn objectives_without_a_known_place_locate_nothing() {
    for kind in [
        ObjectiveKind::TalkToNpc { npc_id: 1 },
        ObjectiveKind::InteractNpc { npc_id: 1 },
        ObjectiveKind::KillActorKind { actor_kind_id: 1 },
        ObjectiveKind::ReachPoi { poi_id: 1 },
        ObjectiveKind::CollectItem { item_id: 5 },
        ObjectiveKind::EnterArea { area_id: 5 },
        ObjectiveKind::FinishMinigame { minigame_id: 5 },
        ObjectiveKind::UseItem { item_id: 5 },
        ObjectiveKind::GatherResource { resource_id: 5 },
        ObjectiveKind::CraftRecipe { recipe_id: 5 },
        ObjectiveKind::SelectJob { profile_id: 5 },
        ObjectiveKind::EquipItem { definition_id: 5 },
        ObjectiveKind::CompleteDungeon { encounter_id: 5 },
        ObjectiveKind::UseAbility {
            ability_id: 5,
            target_actor_kind_id: None,
        },
    ] {
        assert_eq!(locate_kind(kind.clone()), None, "{kind:?}");
    }
}

#[test]
fn a_position_that_is_not_finite_is_never_sent() {
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(
            locate_kind(ObjectiveKind::VisitPosition {
                zone_id: 0,
                position: [0.0, bad, 0.0],
                radius: 1.0,
            }),
            None
        );
    }
}

#[test]
fn only_the_selected_quests_open_objectives_are_tracked() {
    let done_first = objective(1, ObjectiveKind::CollectItem { item_id: 9 });
    let talk = objective(2, ObjectiveKind::TalkToNpc { npc_id: NPC });
    let open = [&done_first, &talk];
    with_places(|places| {
        // The first open objective with a place wins; the unlocated one is skipped.
        assert_eq!(
            tracked_pin(Some(3_000_021), &open, 0, places),
            Some(pin(2, NPC, 0, [-1909.5, -44.4, 462.8]))
        );
        // No selected quest, or a selection with no open objectives (forged, another
        // player's, not active, finished): nothing.
        assert_eq!(tracked_pin(None, &open, 0, places), None);
        assert_eq!(tracked_pin(Some(3_000_021), &[], 0, places), None);
        // Only an objective with no place at all: nothing, not a guess.
        assert_eq!(
            tracked_pin(Some(3_000_021), &[&done_first], 0, places),
            None
        );
    });
}

#[test]
fn a_place_in_another_zone_is_not_pinned() {
    let reach = objective(3, ObjectiveKind::ReachPoi { poi_id: POI });
    with_places(|places| {
        assert_eq!(tracked_pin(Some(9), &[&reach], 0, places), None);
        assert!(tracked_pin(Some(9), &[&reach], 5, places).is_some());
    });
}

#[test]
fn the_tracker_sends_changes_clears_and_nothing_else() {
    let a = pin(1, NPC, 0, [1.0, 2.0, 3.0]);
    let b = pin(2, NPC, 0, [4.0, 5.0, 6.0]);
    let mut tracker = Tracker::default();

    // Nothing wanted and nothing shown: silence, however often it is asked.
    assert_eq!(tracker.update(None), None);
    assert_eq!(tracker.update(None), None);

    // A new pin is sent once, not on every packet.
    assert_eq!(tracker.update(Some(a)), Some(Pin::encode(Some(&a))));
    assert_eq!(tracker.update(Some(a)), None);

    // The next objective replaces it; losing the target clears it once.
    assert_eq!(tracker.update(Some(b)), Some(Pin::encode(Some(&b))));
    assert_eq!(tracker.update(None), Some(vec![47, 0, 14, 0]));
    assert_eq!(tracker.update(None), None);

    // It comes back after a clear (a zone round trip, for instance).
    assert_eq!(tracker.update(Some(b)), Some(Pin::encode(Some(&b))));
}

#[test]
fn a_new_session_sends_the_pin_again() {
    let a = pin(1, NPC, 0, [1.0, 2.0, 3.0]);
    let mut first = Tracker::default();
    assert!(first.update(Some(a)).is_some());
    // The tracker is not saved, so a reconnect starts from nothing.
    let mut second = Tracker::default();
    assert!(second.update(Some(a)).is_some());
}
