//! Work item 068: typed quest event routing.
//!
//! This module turns one recovered objective trigger table into a typed router:
//! an incoming [`QuestEvent`] (talk / interact / visit / kill / gather / craft /
//! game) credits exactly the objectives whose recovered identifier matches, and
//! credits nothing when the identifier was never recovered.
//!
//! ORIGINAL evidence the routes lean on:
//! * `lab/references/Sanctuary/src/Resources/Npcs.json` (pinned community mirror of the
//!   original NPC table, rev `4f028dcc274f628b65ef665fd59b184f5f3d3e3e`, AGPL-3.0) row
//!   3156 `Hooligan Wolf` model 3284, corroborated by the preserved locale row 5100200
//!   `Hooligan Wolf` and by the reconstructed encounter row that spawns the same model.
//! * Preserved locale row 5100532 `Encounter 411 Darkthorne Teleport` for the cave story
//!   link, and supplied quest record 1617 objective 14096 `Defeat Petty Yetis` (required 20)
//!   whose target definition was never recovered.
//!
//! RECONSTRUCTED, never presented as original: the typed vocabulary itself, the
//! rename `defeat` -> kill and `dungeon_completed` -> game, the exact-identifier match
//! rule, quantity clamping, and every 910xxx stage id, objective key, actor GUID,
//! resource id 1001-1003, recipe id 2001-2003 and the zone 0 visit sentinel. The
//! checker `scripts/pieces/piece_068.py` re-derives the table from the pinned sources
//! and records the evidence class of every identifier; the lead owns integration with
//! the journal writer and the gameplay engines.
//!
//! This module routes events. It grants no quest state, item, currency or experience,
//! and it never guesses a target for an objective such as 1617/14096.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

/// Schema version of the route document this module understands.
pub const SCHEMA_VERSION: u32 = 1;

/// Largest finite position component accepted from a local event or route.
const LOCAL_COMPONENT_BOUND: f32 = 1_000_000.0;

/// Exact extraction output of `scripts/pieces/piece_068.py`, embedded so the
/// router works without a private lab path. `tests/piece_068.rs` compares this
/// table against the independent extraction; never hand-edit a row.
pub const RECOVERED_ROUTES_JSON: &str = r#"{
  "schema_version": 1,
  "routes": [
    {"quest_id":900001,"objective_key":1,"required":3,"trigger":{"kind":"kill","definition_id":1294},"origin":"quests.json definition 900001 goal 1"},
    {"quest_id":910001,"objective_key":91000102,"required":1,"trigger":{"kind":"interact","actor_guid":5715144128442925058},"origin":"opening.json stage 910001 objective 91000102"},
    {"quest_id":910002,"objective_key":91000201,"required":1,"trigger":{"kind":"game","encounter_id":411},"origin":"opening.json stage 910002 objective 91000201"},
    {"quest_id":910002,"objective_key":91000202,"required":1,"trigger":{"kind":"talk","actor_guid":5715144128442925059},"origin":"opening.json stage 910002 objective 91000202"},
    {"quest_id":910003,"objective_key":91000301,"required":1,"trigger":{"kind":"interact","actor_guid":5715144128442925064},"origin":"opening.json stage 910003 objective 91000301"},
    {"quest_id":910003,"objective_key":91000302,"required":1,"trigger":{"kind":"interact","actor_guid":5715144128442925065},"origin":"opening.json stage 910003 objective 91000302"},
    {"quest_id":910003,"objective_key":91000303,"required":1,"trigger":{"kind":"interact","actor_guid":5715144128442925066},"origin":"opening.json stage 910003 objective 91000303"},
    {"quest_id":910003,"objective_key":91000304,"required":1,"trigger":{"kind":"interact","actor_guid":5715144128442925067},"origin":"opening.json stage 910003 objective 91000304"},
    {"quest_id":910004,"objective_key":91000401,"required":1,"trigger":{"kind":"talk","actor_guid":5715144128442925060},"origin":"opening.json stage 910004 objective 91000401"},
    {"quest_id":910004,"objective_key":91000404,"required":4,"trigger":{"kind":"gather","resource_id":1001},"origin":"opening.json stage 910004 objective 91000404"},
    {"quest_id":910004,"objective_key":91000405,"required":4,"trigger":{"kind":"gather","resource_id":1002},"origin":"opening.json stage 910004 objective 91000405"},
    {"quest_id":910004,"objective_key":91000406,"required":1,"trigger":{"kind":"craft","recipe_id":2001},"origin":"opening.json stage 910004 objective 91000406"},
    {"quest_id":910005,"objective_key":91000501,"required":1,"trigger":{"kind":"talk","actor_guid":5715144128442925061},"origin":"opening.json stage 910005 objective 91000501"},
    {"quest_id":910005,"objective_key":91000503,"required":5,"trigger":{"kind":"gather","resource_id":1003},"origin":"opening.json stage 910005 objective 91000503"},
    {"quest_id":910005,"objective_key":91000504,"required":5,"trigger":{"kind":"craft","recipe_id":2002},"origin":"opening.json stage 910005 objective 91000504"},
    {"quest_id":910006,"objective_key":91000602,"required":1,"trigger":{"kind":"craft","recipe_id":2003},"origin":"opening.json stage 910006 objective 91000602"},
    {"quest_id":910007,"objective_key":91000703,"required":5,"trigger":{"kind":"kill","definition_id":3156},"origin":"opening.json stage 910007 objective 91000703"},
    {"quest_id":910008,"objective_key":91000801,"required":1,"trigger":{"kind":"talk","actor_guid":5715144128442925059},"origin":"opening.json stage 910008 objective 91000801"},
    {"quest_id":910008,"objective_key":91000802,"required":1,"trigger":{"kind":"kill","definition_id":920001},"origin":"opening.json stage 910008 objective 91000802"},
    {"quest_id":910009,"objective_key":91000901,"required":1,"trigger":{"kind":"talk","actor_guid":5715144128442925062},"origin":"opening.json stage 910009 objective 91000901"},
    {"quest_id":910010,"objective_key":91001001,"required":1,"trigger":{"kind":"interact","actor_guid":5715144128442925077},"origin":"opening.json stage 910010 objective 91001001"},
    {"quest_id":910010,"objective_key":91001002,"required":1,"trigger":{"kind":"interact","actor_guid":5715144128442925078},"origin":"opening.json stage 910010 objective 91001002"},
    {"quest_id":910010,"objective_key":91001003,"required":1,"trigger":{"kind":"interact","actor_guid":5715144128442925079},"origin":"opening.json stage 910010 objective 91001003"},
    {"quest_id":910011,"objective_key":91001101,"required":1,"trigger":{"kind":"kill","definition_id":920002},"origin":"opening.json stage 910011 objective 91001101"},
    {"quest_id":910012,"objective_key":91001201,"required":1,"trigger":{"kind":"visit","zone_id":0,"position":[-1054.135,4.629629,305.7209],"radius":25.0},"origin":"opening.json stage 910012 objective 91001201"},
    {"quest_id":910012,"objective_key":91001202,"required":1,"trigger":{"kind":"talk","actor_guid":5715144128442925063},"origin":"opening.json stage 910012 objective 91001202"},
    {"quest_id":910013,"objective_key":91001302,"required":4,"trigger":{"kind":"gather","resource_id":1001},"origin":"opening.json stage 910013 objective 91001302"},
    {"quest_id":910013,"objective_key":91001303,"required":4,"trigger":{"kind":"gather","resource_id":1002},"origin":"opening.json stage 910013 objective 91001303"},
    {"quest_id":910013,"objective_key":91001304,"required":1,"trigger":{"kind":"craft","recipe_id":2001},"origin":"opening.json stage 910013 objective 91001304"},
    {"quest_id":910014,"objective_key":91001402,"required":10,"trigger":{"kind":"gather","resource_id":1003},"origin":"opening.json stage 910014 objective 91001402"},
    {"quest_id":910015,"objective_key":91001502,"required":5,"trigger":{"kind":"craft","recipe_id":2002},"origin":"opening.json stage 910015 objective 91001502"},
    {"quest_id":910015,"objective_key":91001504,"required":1,"trigger":{"kind":"craft","recipe_id":2003},"origin":"opening.json stage 910015 objective 91001504"},
    {"quest_id":910016,"objective_key":91001601,"required":1,"trigger":{"kind":"talk","actor_guid":5715144128442925080},"origin":"opening.json stage 910016 objective 91001601"},
    {"quest_id":910016,"objective_key":91001602,"required":1,"trigger":{"kind":"talk","actor_guid":5715144128442925081},"origin":"opening.json stage 910016 objective 91001602"},
    {"quest_id":910016,"objective_key":91001603,"required":1,"trigger":{"kind":"talk","actor_guid":5715144128442925082},"origin":"opening.json stage 910016 objective 91001603"},
    {"quest_id":910016,"objective_key":91001604,"required":1,"trigger":{"kind":"talk","actor_guid":5715144128442925083},"origin":"opening.json stage 910016 objective 91001604"}
  ]
}"#;

/// The typed event kinds a route can answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Talk,
    Interact,
    Visit,
    Kill,
    Gather,
    Craft,
    Game,
}

/// One recovered objective trigger.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RouteTrigger {
    Talk {
        actor_guid: u64,
    },
    Interact {
        actor_guid: u64,
    },
    Visit {
        zone_id: u32,
        position: [f32; 3],
        radius: f32,
    },
    Kill {
        definition_id: i32,
    },
    Gather {
        resource_id: u32,
    },
    Craft {
        recipe_id: u32,
    },
    Game {
        encounter_id: u32,
    },
}

impl RouteTrigger {
    pub fn kind(&self) -> EventKind {
        match self {
            RouteTrigger::Talk { .. } => EventKind::Talk,
            RouteTrigger::Interact { .. } => EventKind::Interact,
            RouteTrigger::Visit { .. } => EventKind::Visit,
            RouteTrigger::Kill { .. } => EventKind::Kill,
            RouteTrigger::Gather { .. } => EventKind::Gather,
            RouteTrigger::Craft { .. } => EventKind::Craft,
            RouteTrigger::Game { .. } => EventKind::Game,
        }
    }
}

/// One objective route: the live journal addresses objectives by
/// [`QuestRoute::objective_key`] (the objective definition id), not by ordinal.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuestRoute {
    pub quest_id: u32,
    pub objective_key: u32,
    pub required: u32,
    pub trigger: RouteTrigger,
    /// Where this row was extracted from; kept so a reviewer can re-derive it.
    pub origin: String,
}

/// A typed gameplay event, already narrowed to recovered identifiers.
#[derive(Clone, Debug, PartialEq)]
pub enum QuestEvent {
    Talk { actor_guid: u64 },
    Interact { actor_guid: u64 },
    Visit { zone_id: u32, position: [f32; 3] },
    Kill { definition_id: i32 },
    Gather { resource_id: u32, quantity: u32 },
    Craft { recipe_id: u32, quantity: u32 },
    Game { encounter_id: u32 },
}

impl QuestEvent {
    pub fn kind(&self) -> EventKind {
        match self {
            QuestEvent::Talk { .. } => EventKind::Talk,
            QuestEvent::Interact { .. } => EventKind::Interact,
            QuestEvent::Visit { .. } => EventKind::Visit,
            QuestEvent::Kill { .. } => EventKind::Kill,
            QuestEvent::Gather { .. } => EventKind::Gather,
            QuestEvent::Craft { .. } => EventKind::Craft,
            QuestEvent::Game { .. } => EventKind::Game,
        }
    }

    fn validate(&self) -> Result<(), RouteError> {
        match self {
            QuestEvent::Talk { actor_guid } | QuestEvent::Interact { actor_guid }
                if *actor_guid == 0 =>
            {
                Err(RouteError::InvalidEventIdentifier)
            }
            QuestEvent::Kill { definition_id } if *definition_id <= 0 => {
                Err(RouteError::InvalidEventIdentifier)
            }
            QuestEvent::Gather {
                resource_id,
                quantity,
            } => {
                if *resource_id == 0 {
                    return Err(RouteError::InvalidEventIdentifier);
                }
                if *quantity == 0 {
                    return Err(RouteError::ZeroQuantity);
                }
                Ok(())
            }
            QuestEvent::Craft {
                recipe_id,
                quantity,
            } => {
                if *recipe_id == 0 {
                    return Err(RouteError::InvalidEventIdentifier);
                }
                if *quantity == 0 {
                    return Err(RouteError::ZeroQuantity);
                }
                Ok(())
            }
            QuestEvent::Game { encounter_id } if *encounter_id == 0 => {
                Err(RouteError::InvalidEventIdentifier)
            }
            QuestEvent::Visit { position, .. } if !valid_position(*position) => {
                Err(RouteError::InvalidEventPosition)
            }
            _ => Ok(()),
        }
    }
}

/// One matched objective: the caller adds `credit` to `objective_key` and
/// completes it at `required`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectiveCredit {
    pub quest_id: u32,
    pub objective_key: u32,
    pub required: u32,
    pub credit: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteError {
    UnsupportedSchema(u32),
    MalformedTable,
    EmptyTable,
    ZeroQuestId,
    ZeroObjectiveKey,
    ZeroRequired { quest_id: u32, objective_key: u32 },
    ZeroIdentifier { quest_id: u32, objective_key: u32 },
    DuplicateObjectiveKey { objective_key: u32 },
    InvalidRoutePosition { objective_key: u32 },
    InvalidRadius { objective_key: u32 },
    InvalidEventIdentifier,
    InvalidEventPosition,
    ZeroQuantity,
}

impl fmt::Display for RouteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RouteError::UnsupportedSchema(version) => {
                write!(f, "unsupported route schema version {version}")
            }
            RouteError::MalformedTable => write!(f, "route table is not valid JSON"),
            RouteError::EmptyTable => write!(f, "route table has no routes"),
            RouteError::ZeroQuestId => write!(f, "route has no quest id"),
            RouteError::ZeroObjectiveKey => write!(f, "route has no objective key"),
            RouteError::ZeroRequired {
                quest_id,
                objective_key,
            } => write!(
                f,
                "route {quest_id}:{objective_key} has no positive required count"
            ),
            RouteError::ZeroIdentifier {
                quest_id,
                objective_key,
            } => write!(
                f,
                "route {quest_id}:{objective_key} has a zero or negative target identifier"
            ),
            RouteError::DuplicateObjectiveKey { objective_key } => {
                write!(f, "objective key {objective_key} appears twice")
            }
            RouteError::InvalidRoutePosition { objective_key } => {
                write!(f, "visit route {objective_key} has an invalid position")
            }
            RouteError::InvalidRadius { objective_key } => {
                write!(f, "visit route {objective_key} has an invalid radius")
            }
            RouteError::InvalidEventIdentifier => {
                write!(f, "event carries no usable identifier")
            }
            RouteError::InvalidEventPosition => write!(f, "event position is invalid"),
            RouteError::ZeroQuantity => write!(f, "event quantity is zero"),
        }
    }
}

impl std::error::Error for RouteError {}

#[derive(Deserialize)]
struct RouteDocument {
    schema_version: u32,
    routes: Vec<QuestRoute>,
}

/// Exact-identifier router over one recovered objective table.
#[derive(Clone, Debug)]
pub struct QuestEventRouter {
    routes: Vec<QuestRoute>,
}

impl QuestEventRouter {
    /// Validate and store a route table. Routes are kept sorted by
    /// `(quest_id, objective_key)` so results are deterministic.
    pub fn new(mut routes: Vec<QuestRoute>) -> Result<Self, RouteError> {
        if routes.is_empty() {
            return Err(RouteError::EmptyTable);
        }
        routes.sort_by_key(|route| (route.quest_id, route.objective_key));
        let mut keys = BTreeSet::new();
        for route in &routes {
            if route.quest_id == 0 {
                return Err(RouteError::ZeroQuestId);
            }
            if route.objective_key == 0 {
                return Err(RouteError::ZeroObjectiveKey);
            }
            if route.required == 0 || route.required > 1_000_000 {
                return Err(RouteError::ZeroRequired {
                    quest_id: route.quest_id,
                    objective_key: route.objective_key,
                });
            }
            if !keys.insert(route.objective_key) {
                return Err(RouteError::DuplicateObjectiveKey {
                    objective_key: route.objective_key,
                });
            }
            let zero_identifier = match &route.trigger {
                RouteTrigger::Talk { actor_guid } | RouteTrigger::Interact { actor_guid } => {
                    *actor_guid == 0
                }
                RouteTrigger::Kill { definition_id } => *definition_id <= 0,
                RouteTrigger::Gather { resource_id } => *resource_id == 0,
                RouteTrigger::Craft { recipe_id } => *recipe_id == 0,
                RouteTrigger::Game { encounter_id } => *encounter_id == 0,
                RouteTrigger::Visit { .. } => false,
            };
            if zero_identifier {
                return Err(RouteError::ZeroIdentifier {
                    quest_id: route.quest_id,
                    objective_key: route.objective_key,
                });
            }
            if let RouteTrigger::Visit {
                position, radius, ..
            } = &route.trigger
            {
                if !valid_position(*position) {
                    return Err(RouteError::InvalidRoutePosition {
                        objective_key: route.objective_key,
                    });
                }
                if !radius.is_finite() || !(0.1..=1_000.0).contains(radius) {
                    return Err(RouteError::InvalidRadius {
                        objective_key: route.objective_key,
                    });
                }
            }
        }
        Ok(Self { routes })
    }

    /// Parse a route document (`schema_version` plus `routes`; extra recovery
    /// fields produced by the checker are ignored).
    pub fn load_json(text: &str) -> Result<Self, RouteError> {
        let document: RouteDocument =
            serde_json::from_str(text).map_err(|_| RouteError::MalformedTable)?;
        if document.schema_version != SCHEMA_VERSION {
            return Err(RouteError::UnsupportedSchema(document.schema_version));
        }
        Self::new(document.routes)
    }

    /// The embedded extraction output, ready to route.
    pub fn recovered() -> Result<Self, RouteError> {
        Self::load_json(RECOVERED_ROUTES_JSON)
    }

    pub fn routes(&self) -> &[QuestRoute] {
        &self.routes
    }

    /// Credit every objective whose recovered identifier matches this event.
    /// A well-formed event that matches nothing returns an empty list: unknown
    /// or unrelated identifiers earn no progress.
    pub fn route(&self, event: &QuestEvent) -> Result<Vec<ObjectiveCredit>, RouteError> {
        event.validate()?;
        let mut credits = Vec::new();
        for route in &self.routes {
            let credit = match (&route.trigger, event) {
                (RouteTrigger::Talk { actor_guid }, QuestEvent::Talk { actor_guid: other })
                | (
                    RouteTrigger::Interact { actor_guid },
                    QuestEvent::Interact { actor_guid: other },
                ) if actor_guid == other => 1,
                (
                    RouteTrigger::Kill { definition_id },
                    QuestEvent::Kill {
                        definition_id: other,
                    },
                ) if definition_id == other => 1,
                (
                    RouteTrigger::Gather { resource_id },
                    QuestEvent::Gather {
                        resource_id: other,
                        quantity,
                    },
                ) if resource_id == other => (*quantity).min(route.required),
                (
                    RouteTrigger::Craft { recipe_id },
                    QuestEvent::Craft {
                        recipe_id: other,
                        quantity,
                    },
                ) if recipe_id == other => (*quantity).min(route.required),
                (
                    RouteTrigger::Game { encounter_id },
                    QuestEvent::Game {
                        encounter_id: other,
                    },
                ) if encounter_id == other => 1,
                (
                    RouteTrigger::Visit {
                        zone_id,
                        position,
                        radius,
                    },
                    QuestEvent::Visit {
                        zone_id: other_zone,
                        position: other_position,
                    },
                ) if zone_id == other_zone && within(*position, *other_position, *radius) => 1,
                _ => 0,
            };
            if credit > 0 {
                credits.push(ObjectiveCredit {
                    quest_id: route.quest_id,
                    objective_key: route.objective_key,
                    required: route.required,
                    credit,
                });
            }
        }
        Ok(credits)
    }
}

fn valid_position(position: [f32; 3]) -> bool {
    position
        .iter()
        .all(|value| value.is_finite() && value.abs() <= LOCAL_COMPONENT_BOUND)
}

fn within(a: [f32; 3], b: [f32; 3], radius: f32) -> bool {
    a.iter().zip(b).map(|(x, y)| (x - y).powi(2)).sum::<f32>() <= radius * radius
}
