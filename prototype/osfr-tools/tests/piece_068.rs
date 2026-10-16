//! Work item 068: typed quest event routing for the restored Free Realms server.
//!
//! ORIGINAL evidence this test leans on (all outside this file):
//! * `lab/references/Sanctuary/src/Resources/Npcs.json` row 3156 `Hooligan Wolf`, model 3284,
//!   and the preserved locale row 5100200 `Hooligan Wolf`. The reconstructed cave encounter
//!   (`lab/references/opening-anvil-placement-v1/content/encounters.json`) spawns that same
//!   definition with the same model, so the local `defeat` trigger names a recovered enemy
//!   definition rather than a guess.
//! * Preserved locale row 5100532 `Encounter 411 Darkthorne Teleport` and the encounter document's
//!   dungeon id 411. The zone id 411 is a local assignment (recorded in the document's own
//!   provenance); only the story link is original.
//! * Supplied quest record 1617 `Harassing the Harassers`, objective 14096 `Defeat Petty Yetis`,
//!   required 20, reward item 1843 x1, and the catalogue's disabled claim
//!   (`notes/Research/Pieces/068 Quest event routing.md`). No verified Petty Yeti enemy definition
//!   exists in any pinned resource, so that objective must stay un-routable.
//!
//! RECONSTRUCTED policy exercised here: the typed trigger vocabulary, the exact-identifier match
//! rule, quantity clamping and the six-kind mapping. The 910xxx stage/objective ids, actor GUIDs,
//! resource ids 1001-1003, recipe ids 2001-2003 and zone id 0 are local reserved values from
//! `opening.json`, not original Free Realms identifiers.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers `pub mod piece_068;` under `src/lib.rs`.

#[path = "../src/restoration/piece_068.rs"]
#[allow(dead_code)]
mod piece_068;

use piece_068::{
    EventKind, ObjectiveCredit, QuestEvent, QuestEventRouter, QuestRoute, RouteError, RouteTrigger,
    SCHEMA_VERSION,
};

fn credit(quest_id: u32, objective_key: u32, required: u32, credit: u32) -> ObjectiveCredit {
    ObjectiveCredit {
        quest_id,
        objective_key,
        required,
        credit,
    }
}

fn assert_close(a: [f32; 3], b: [f32; 3]) {
    for (left, right) in a.iter().zip(b) {
        assert!(
            (left - right).abs() < 1e-4,
            "position mismatch: {a:?} vs {b:?}"
        );
    }
}

#[test]
fn recovered_table_carries_every_typed_kind_with_its_recovered_identifiers() {
    let router = QuestEventRouter::recovered().expect("embedded recovered routes");
    assert_eq!(SCHEMA_VERSION, 1);
    let routes = router.routes();
    assert_eq!(
        routes.len(),
        36,
        "35 opening routes plus one engine kill route"
    );

    let kinds: std::collections::BTreeSet<EventKind> =
        routes.iter().map(|route| route.trigger.kind()).collect();
    assert_eq!(
        kinds,
        [
            EventKind::Talk,
            EventKind::Interact,
            EventKind::Visit,
            EventKind::Kill,
            EventKind::Gather,
            EventKind::Craft,
            EventKind::Game,
        ]
        .into_iter()
        .collect()
    );

    let talk = routes
        .iter()
        .find(|route| route.objective_key == 91000401)
        .expect("Mae Farnum talk objective");
    assert_eq!(talk.quest_id, 910004);
    assert_eq!(talk.required, 1);
    assert!(matches!(
        talk.trigger,
        RouteTrigger::Talk {
            actor_guid: 5715144128442925060
        }
    ));

    let gather = routes
        .iter()
        .find(|route| route.objective_key == 91000404)
        .expect("Rainleaf objective");
    assert_eq!(gather.quest_id, 910004);
    assert_eq!(gather.required, 4);
    assert!(matches!(
        gather.trigger,
        RouteTrigger::Gather { resource_id: 1001 }
    ));

    let craft = routes
        .iter()
        .find(|route| route.objective_key == 91001502)
        .expect("smelting objective");
    assert_eq!(craft.required, 5);
    assert!(matches!(
        craft.trigger,
        RouteTrigger::Craft { recipe_id: 2002 }
    ));

    let visit = routes
        .iter()
        .find(|route| route.objective_key == 91001201)
        .expect("Queen road visit objective");
    assert_eq!(visit.quest_id, 910012);
    assert_eq!(visit.required, 1);
    match visit.trigger {
        RouteTrigger::Visit {
            zone_id,
            position,
            radius,
        } => {
            assert_eq!(zone_id, 0, "local sentinel world zone, recorded as a gap");
            assert_close(position, [-1054.135, 4.629629, 305.7209]);
            assert!((radius - 25.0).abs() < 1e-4);
        }
        ref other => panic!("unexpected trigger {other:?}"),
    }

    let kill = routes
        .iter()
        .find(|route| route.objective_key == 91000703)
        .expect("wolf objective");
    assert_eq!(kill.required, 5);
    assert!(matches!(
        kill.trigger,
        RouteTrigger::Kill {
            definition_id: 3156
        }
    ));

    let game = routes
        .iter()
        .find(|route| route.objective_key == 91000201)
        .expect("cave completion objective");
    assert_eq!(game.required, 1);
    assert!(matches!(
        game.trigger,
        RouteTrigger::Game { encounter_id: 411 }
    ));

    let engine_kill = routes
        .iter()
        .find(|route| route.objective_key == 1)
        .expect("local restoration kill route");
    assert_eq!(engine_kill.quest_id, 900001);
    assert_eq!(engine_kill.required, 3);
    assert!(matches!(
        engine_kill.trigger,
        RouteTrigger::Kill {
            definition_id: 1294
        }
    ));

    for route in routes {
        assert!(!route.origin.is_empty(), "route {route:?} has no origin");
    }
}

#[test]
fn every_typed_event_credits_only_its_matching_route() {
    let router = QuestEventRouter::recovered().unwrap();

    let cases: [(QuestEvent, Vec<ObjectiveCredit>); 7] = [
        (
            QuestEvent::Talk {
                actor_guid: 5715144128442925060,
            },
            vec![credit(910004, 91000401, 1, 1)],
        ),
        (
            QuestEvent::Interact {
                actor_guid: 5715144128442925058,
            },
            vec![credit(910001, 91000102, 1, 1)],
        ),
        (
            QuestEvent::Visit {
                zone_id: 0,
                position: [-1060.0, 4.0, 310.0],
            },
            vec![credit(910012, 91001201, 1, 1)],
        ),
        (
            QuestEvent::Kill {
                definition_id: 3156,
            },
            vec![credit(910007, 91000703, 5, 1)],
        ),
        (
            QuestEvent::Gather {
                resource_id: 1003,
                quantity: 2,
            },
            vec![
                credit(910005, 91000503, 5, 2),
                credit(910014, 91001402, 10, 2),
            ],
        ),
        (
            QuestEvent::Craft {
                recipe_id: 2003,
                quantity: 1,
            },
            vec![
                credit(910006, 91000602, 1, 1),
                credit(910015, 91001504, 1, 1),
            ],
        ),
        (
            QuestEvent::Game { encounter_id: 411 },
            vec![credit(910002, 91000201, 1, 1)],
        ),
    ];

    for (event, expected) in cases {
        let actual = router.route(&event).expect("valid event");
        assert_eq!(actual, expected, "event {event:?}");
    }
}

#[test]
fn unrelated_kills_never_credit_another_objective() {
    let router = QuestEventRouter::recovered().unwrap();

    let robgoblin = router
        .route(&QuestEvent::Kill {
            definition_id: 1294,
        })
        .unwrap();
    assert_eq!(robgoblin, vec![credit(900001, 1, 3, 1)]);

    let construct = router
        .route(&QuestEvent::Kill {
            definition_id: 920001,
        })
        .unwrap();
    assert_eq!(construct, vec![credit(910008, 91000802, 1, 1)]);

    let unknown = router
        .route(&QuestEvent::Kill {
            definition_id: 999_999,
        })
        .unwrap();
    assert!(unknown.is_empty(), "unknown enemy definitions earn nothing");

    // Quest 1617 / objective 14096 (`Defeat Petty Yetis`) has no recovered enemy definition, so
    // no recovered or unrelated enemy may ever advance it.
    for definition_id in [1294, 3156, 920001, 920002, 920100, 999_999] {
        let credits = router.route(&QuestEvent::Kill { definition_id }).unwrap();
        assert!(
            credits.iter().all(|row| row.objective_key != 14096),
            "definition {definition_id} credited the Petty Yeti objective"
        );
    }
}

#[test]
fn unrecovered_kill_targets_are_refused_instead_of_guessed() {
    let recovered = QuestEventRouter::recovered().unwrap();
    assert!(recovered.routes().iter().all(|route| match route.trigger {
        RouteTrigger::Kill { definition_id } => definition_id > 0,
        _ => true,
    }));

    let mut routes = recovered.routes().to_vec();
    routes.push(QuestRoute {
        quest_id: 1617,
        objective_key: 14096,
        required: 20,
        trigger: RouteTrigger::Kill { definition_id: 0 },
        origin: "supplied PacketSendSelfToClient.json quest 1617 (target unrecovered)".into(),
    });
    assert_eq!(
        QuestEventRouter::new(routes).unwrap_err(),
        RouteError::ZeroIdentifier {
            quest_id: 1617,
            objective_key: 14096,
        }
    );

    assert_eq!(
        recovered
            .route(&QuestEvent::Kill { definition_id: 0 })
            .unwrap_err(),
        RouteError::InvalidEventIdentifier
    );
}

#[test]
fn gather_and_craft_credit_clamps_to_the_required_count() {
    let router = QuestEventRouter::recovered().unwrap();
    let credits = router
        .route(&QuestEvent::Gather {
            resource_id: 1001,
            quantity: 10,
        })
        .unwrap();
    assert_eq!(
        credits,
        vec![
            credit(910004, 91000404, 4, 4),
            credit(910013, 91001302, 4, 4),
        ]
    );

    let partial = router
        .route(&QuestEvent::Craft {
            recipe_id: 2002,
            quantity: 2,
        })
        .unwrap();
    assert_eq!(
        partial,
        vec![
            credit(910005, 91000504, 5, 2),
            credit(910015, 91001502, 5, 2),
        ]
    );

    assert_eq!(
        router
            .route(&QuestEvent::Gather {
                resource_id: 1001,
                quantity: 0,
            })
            .unwrap_err(),
        RouteError::ZeroQuantity
    );
}

#[test]
fn visit_routing_requires_the_same_zone_and_position() {
    let router = QuestEventRouter::recovered().unwrap();

    let inside = router
        .route(&QuestEvent::Visit {
            zone_id: 0,
            position: [-1054.0, 4.6, 305.7],
        })
        .unwrap();
    assert_eq!(inside, vec![credit(910012, 91001201, 1, 1)]);

    let outside = router
        .route(&QuestEvent::Visit {
            zone_id: 0,
            position: [-1200.0, 4.6, 305.7],
        })
        .unwrap();
    assert!(
        outside.is_empty(),
        "outside the recovered radius earns nothing"
    );

    let wrong_zone = router
        .route(&QuestEvent::Visit {
            zone_id: 411,
            position: [-1054.0, 4.6, 305.7],
        })
        .unwrap();
    assert!(
        wrong_zone.is_empty(),
        "another zone never credits the visit"
    );

    assert_eq!(
        router
            .route(&QuestEvent::Visit {
                zone_id: 0,
                position: [f32::NAN, 0.0, 0.0],
            })
            .unwrap_err(),
        RouteError::InvalidEventPosition
    );
}

#[test]
fn malformed_events_are_refused_instead_of_credited() {
    let router = QuestEventRouter::recovered().unwrap();
    let events = [
        QuestEvent::Talk { actor_guid: 0 },
        QuestEvent::Interact { actor_guid: 0 },
        QuestEvent::Kill { definition_id: -4 },
        QuestEvent::Gather {
            resource_id: 0,
            quantity: 1,
        },
        QuestEvent::Craft {
            recipe_id: 0,
            quantity: 1,
        },
        QuestEvent::Game { encounter_id: 0 },
    ];
    for event in events {
        assert_eq!(
            router.route(&event).unwrap_err(),
            RouteError::InvalidEventIdentifier,
            "event {event:?}"
        );
    }
}

#[test]
fn table_validation_refuses_damaged_routes() {
    let base = QuestRoute {
        quest_id: 910004,
        objective_key: 91000401,
        required: 1,
        trigger: RouteTrigger::Talk {
            actor_guid: 5715144128442925060,
        },
        origin: "synthetic".into(),
    };

    let mut duplicate = vec![base.clone(), base.clone()];
    assert_eq!(
        QuestEventRouter::new(duplicate.clone()).unwrap_err(),
        RouteError::DuplicateObjectiveKey {
            objective_key: 91000401
        }
    );
    duplicate[1].objective_key = 91000402;
    duplicate[1].quest_id = 0;
    assert_eq!(
        QuestEventRouter::new(duplicate).unwrap_err(),
        RouteError::ZeroQuestId
    );

    let mut zero_objective = vec![base.clone()];
    zero_objective[0].objective_key = 0;
    assert_eq!(
        QuestEventRouter::new(zero_objective).unwrap_err(),
        RouteError::ZeroObjectiveKey
    );

    let mut zero_required = vec![base.clone()];
    zero_required[0].required = 0;
    assert_eq!(
        QuestEventRouter::new(zero_required).unwrap_err(),
        RouteError::ZeroRequired {
            quest_id: 910004,
            objective_key: 91000401,
        }
    );

    let mut bad_radius = vec![base.clone()];
    bad_radius[0].trigger = RouteTrigger::Visit {
        zone_id: 0,
        position: [0.0, 0.0, 0.0],
        radius: f32::INFINITY,
    };
    assert_eq!(
        QuestEventRouter::new(bad_radius).unwrap_err(),
        RouteError::InvalidRadius {
            objective_key: 91000401
        }
    );

    assert_eq!(
        QuestEventRouter::new(Vec::new()).unwrap_err(),
        RouteError::EmptyTable
    );
}

#[test]
fn json_table_loads_extra_recovery_fields_and_rejects_a_foreign_schema() {
    let router = QuestEventRouter::load_json(
        r#"{
          "schema_version": 1,
          "sources": [{"path": "opening.json", "sha256": "00"}],
          "blocked": [{"quest_id": 1617, "objective_key": 14096}],
          "routes": [
            {
              "quest_id": 910007,
              "objective_key": 91000703,
              "required": 5,
              "trigger": {"kind": "kill", "definition_id": 3156},
              "origin": "opening.json stage 910007 objective 91000703"
            }
          ]
        }"#,
    )
    .unwrap();
    assert_eq!(router.routes().len(), 1);
    assert_eq!(
        router
            .route(&QuestEvent::Kill {
                definition_id: 3156
            })
            .unwrap(),
        vec![credit(910007, 91000703, 5, 1)]
    );

    assert_eq!(
        QuestEventRouter::load_json(r#"{"schema_version": 2, "routes": []}"#).unwrap_err(),
        RouteError::UnsupportedSchema(2)
    );
    assert_eq!(
        QuestEventRouter::load_json("not json").unwrap_err(),
        RouteError::MalformedTable
    );
}

/// Optional private cross-check: compare the embedded table with the checker's extraction output.
/// Run with `OSFR_PIECE_068_ROUTES=<lab>/reports/piece-068-<utc>/route-table.json`.
#[test]
#[ignore = "requires the piece 068 lab extraction output"]
fn embedded_table_matches_the_independent_extraction() {
    let path = std::env::var("OSFR_PIECE_068_ROUTES").expect("OSFR_PIECE_068_ROUTES");
    let text = std::fs::read_to_string(&path).expect("route table");
    let extracted = QuestEventRouter::load_json(&text).expect("extracted table");
    let embedded = QuestEventRouter::recovered().expect("embedded table");

    let mut left: Vec<_> = embedded.routes().to_vec();
    let mut right: Vec<_> = extracted.routes().to_vec();
    left.sort_by_key(|route| (route.quest_id, route.objective_key));
    right.sort_by_key(|route| (route.quest_id, route.objective_key));
    assert_eq!(left, right, "embedded routes drifted from the extraction");
}
