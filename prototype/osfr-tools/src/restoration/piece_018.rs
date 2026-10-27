//! Work item 018: developer entry points for the restored minigames.
//!
//! `realm game` resolves a developer-only route for each supported minigame
//! and reports exactly what the current artifact can and cannot do:
//!
//! * `cooking`, `smelting` and `forging` are native-accepted movies that start
//!   from a loaded world actor. The route names that actor, its recovered
//!   profile and recipe id, and leaves the actual relocation to the caller,
//!   which reuses the reviewed `dev_teleport` standoff (this module never
//!   computes its own arrival and never sends a packet);
//! * `mining`, `harvesting`, `fishing`, `kart_racing` and `soccer` have
//!   code/test candidates that the gateway does not integrate yet. Their route
//!   reports the module, the driver command and the original wire gate instead
//!   of pretending a launch exists;
//! * `derby` has only the decoded opaque frame study, so its route reports the
//!   missing launch contract (queue item 007) and performs nothing.
//!
//! The whole surface is off unless the launcher passes `--dev-tools`:
//! [`handle`] returns `None` while the gate is closed, so every other chat
//! handler keeps its existing behavior byte for byte. Enabling it still grants
//! nothing: the route only reports a relocation target and hands it back.
//!
//! Integration (lead-owned): in the `dev_tools` branch next to
//! `developer_teleport`, call [`handle`] with the loaded actors converted to
//! [`ActorView`] and apply the returned [`Command::Route`] through
//! `dev_teleport::handle(&format!("goto {actor}"), actors, character)`.
//!
//! Evidence: `notes/Research/Pieces/018 Minigame native test routes.md`,
//! `notes/Research/original-activity-coverage.json` (item 015) and the bound
//! private content checked by the private recovery checker `piece_018.py`.

/// Chat-body keyword under `realm `, matching the `realm goto` convention.
pub const COMMAND: &str = "game";
/// The launcher flag that opens the developer surface. Off by default.
pub const GATE: &str = "--dev-tools";
/// The only zone these routes relocate inside, matching `dev_teleport`.
pub const WORLD_ZONE: u32 = 0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    NativeAccepted,
    CodeTestCandidate,
    WireStudyOnly,
}

impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Status::NativeAccepted => "native-accepted",
            Status::CodeTestCandidate => "code/test candidate",
            Status::WireStudyOnly => "wire study only",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    /// Started by ordinary interaction with a loaded world actor in zone 0.
    WorldActor {
        actor: &'static str,
        profile: i32,
        recipe: i32,
    },
    /// Exercised through the module's own bounded driver until integration.
    ModuleDriver {
        api: &'static str,
        driver: &'static str,
    },
    /// No launch contract recovered; the route reports the gap only.
    Unrecovered { missing: &'static str },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Route {
    pub slug: &'static str,
    pub aliases: &'static [&'static str],
    pub title: &'static str,
    pub family: &'static str,
    pub status: Status,
    pub original_games: &'static [i32],
    pub original_type: u16,
    pub module_path: &'static str,
    pub tests: &'static [&'static str],
    pub notes: &'static str,
    pub entry: Entry,
    pub native_gate: &'static str,
}

/// Minimal view of a loaded opening actor. The caller converts the real
/// `opening::OpeningActor` rows so this module stays independent of the
/// gateway types while it waits for lead registration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActorView<'a> {
    pub name: &'a str,
    pub guid: u64,
    pub zone_id: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Reply(String),
    Route {
        reply: String,
        actor: &'static str,
        guid: u64,
    },
}

pub const ROUTES: &[Route] = &[
    Route {
        slug: "cooking",
        aliases: &["cook"],
        title: "Cooking (Strong Arm Stew)",
        family: "TYPE 9 mimic movie (Chef)",
        status: Status::NativeAccepted,
        original_games: &[46],
        original_type: 9,
        module_path: "prototype/osfr-tools/src/cooking.rs",
        tests: &[
            "prototype/osfr-tools/tests/cooking.rs",
            "prototype/osfr-tools/tests/cooking_gameplay.rs",
        ],
        notes: "notes/Research/Original Cooking.md",
        entry: Entry::WorldActor {
            actor: "Cooking Table",
            profile: 45,
            recipe: 2001,
        },
        native_gate: "lead gate: original movie entry, six rounds, atomic recipe reward and ordinary exit",
    },
    Route {
        slug: "smelting",
        aliases: &["smelt"],
        title: "Smelting (Copper Bar)",
        family: "TYPE 9 mimic movie (Miner)",
        status: Status::NativeAccepted,
        original_games: &[77],
        original_type: 9,
        module_path: "prototype/osfr-tools/src/smelting.rs",
        tests: &[
            "prototype/osfr-tools/tests/smelting.rs",
            "prototype/osfr-tools/tests/smelting_gameplay.rs",
        ],
        notes: "notes/Research/Original Smelting.md",
        entry: Entry::WorldActor {
            actor: "Smelt 5 Copper Bars",
            profile: 14,
            recipe: 2002,
        },
        native_gate: "lead gate: four-stage furnace controls, win, atomic bar reward and ordinary exit",
    },
    Route {
        slug: "forging",
        aliases: &["forge"],
        title: "Forging (Spunky Scrapper Hammer anvil)",
        family: "TYPE 8 MatchThree (Forging row 139)",
        status: Status::NativeAccepted,
        original_games: &[139],
        original_type: 8,
        module_path: "prototype/osfr-tools/src/forging.rs",
        tests: &[
            "prototype/osfr-tools/tests/forging.rs",
            "prototype/osfr-tools/tests/forging_gameplay.rs",
        ],
        notes: "notes/Research/Original Forging.md",
        entry: Entry::WorldActor {
            actor: "Anvil",
            profile: 16,
            recipe: 2003,
        },
        native_gate: "lead gate: original anvil entry, real drag chains without desync, one output and ordinary exit",
    },
    Route {
        slug: "mining",
        aliases: &["mine"],
        title: "Mining (TYPE 8 MatchThree board)",
        family: "TYPE 8 MatchThree board",
        status: Status::CodeTestCandidate,
        original_games: &[18, 91],
        original_type: 8,
        module_path: "prototype/osfr-tools/src/mining.rs",
        tests: &["prototype/osfr-tools/tests/mining.rs"],
        notes: "notes/Research/Mining Restoration.md",
        entry: Entry::ModuleDriver {
            api: "mining::Type8Engine::load(...) then begin(18); mining_spec() is the reconstructed default",
            driver: "scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test mining",
        },
        native_gate: "open gate: original game_mining.gfx board start, level and removal against the movie; the engine is not registered in the gateway and its boards are reconstructed",
    },
    Route {
        slug: "harvesting",
        aliases: &["harvest"],
        title: "Harvesting (Farnum's Farm)",
        family: "TYPE 8 MatchThree board",
        status: Status::CodeTestCandidate,
        original_games: &[58, 111],
        original_type: 8,
        module_path: "prototype/osfr-tools/src/harvesting.rs",
        tests: &["prototype/osfr-tools/tests/harvesting.rs"],
        notes: "notes/Research/Harvesting Restoration.md",
        entry: Entry::ModuleDriver {
            api: "harvesting::board_spec(111) and content(111) loaded into the shared type-8 engine",
            driver: "scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test harvesting",
        },
        native_gate: "open gate: original game_farming.gfx board against the shared engine; boards, quotas and drop cadence are reconstructed",
    },
    Route {
        slug: "fishing",
        aliases: &["fish"],
        title: "Fishing (Darklit Lagoon and the recovered lakes)",
        family: "TYPE 21 native world activity",
        status: Status::CodeTestCandidate,
        original_games: &[595, 596],
        original_type: 21,
        module_path: "prototype/osfr-tools/src/fishing.rs",
        tests: &["prototype/osfr-tools/tests/fishing.rs"],
        notes: "notes/Research/Fishing Restoration.md",
        entry: Entry::ModuleDriver {
            api: "fishing::Session::request(ClientRequest) with the recovered eight-byte actor handle",
            driver: "scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test fishing",
        },
        native_gate: "open gate: opcode 138 cast/catch lifecycle against the native client; phase labels and ordering rules are reconstructed",
    },
    Route {
        slug: "kart_racing",
        aliases: &["kart", "racing"],
        title: "Kart racing (Wildwood, Frost Ridge, Tangletrack)",
        family: "TYPE 10 native vehicle race",
        status: Status::CodeTestCandidate,
        original_games: &[28, 37, 38],
        original_type: 10,
        module_path: "prototype/osfr-tools/src/kart_racing.rs",
        tests: &["prototype/osfr-tools/tests/kart_racing.rs"],
        notes: "notes/Research/Kart Racing Restoration.md",
        entry: Entry::ModuleDriver {
            api: "kart_racing::RaceSession::open(...) over the captured subtype stream",
            driver: "scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test kart_racing",
        },
        native_gate: "open gate: original VehicleRace lobby/config stream and native checkpoint driving; lap counts, checkpoint ids and rewards are reconstructed",
    },
    Route {
        slug: "soccer",
        aliases: &["football"],
        title: "Soccer (Sacred Grove)",
        family: "TYPE 12 native match",
        status: Status::CodeTestCandidate,
        original_games: &[42, 61],
        original_type: 12,
        module_path: "prototype/osfr-tools/src/soccer.rs",
        tests: &["prototype/osfr-tools/tests/soccer.rs"],
        notes: "notes/Research/Soccer Restoration.md",
        entry: Entry::ModuleDriver {
            api: "soccer::Match::new plus the reassembled Sacred Grove opcode 76 stream",
            driver: "scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test soccer",
        },
        native_gate: "open gate: native Sacred Grove match start, roster and ball stream; the capture never shows match completion and the half-end transition is reconstructed",
    },
    Route {
        slug: "derby",
        aliases: &["demolition"],
        title: "Demolition derby",
        family: "TYPE 11 native arena (opaque frame study only)",
        status: Status::WireStudyOnly,
        original_games: &[39],
        original_type: 11,
        module_path: "prototype/osfr-tools/src/derby.rs",
        tests: &["prototype/osfr-tools/tests/derby.rs"],
        notes: "notes/Research/Kart Racing Restoration.md",
        entry: Entry::Unrecovered {
            missing: "no launch route: only the tunnel envelope, per-subtype lengths, participant keys and vehicle-config asset names are decoded; arena, damage, scoring and reward packets are unrecovered (queue item 007)",
        },
        native_gate: "open gate: a recovered launch and arena contract before any developer test route can exist",
    },
];

pub fn routes() -> &'static [Route] {
    ROUTES
}

/// Exact slug lookup. Aliases and prefixes go through [`resolve`].
pub fn route(slug: &str) -> Option<&'static Route> {
    ROUTES.iter().find(|route| route.slug == slug)
}

/// Exact slug, then exact alias, then a unique prefix over slugs and aliases.
/// `Err` carries the candidate slugs, empty when nothing matches at all.
pub fn resolve(query: &str) -> Result<&'static Route, Vec<&'static str>> {
    let query = query.trim().to_ascii_lowercase();
    if query.is_empty() {
        return Err(Vec::new());
    }
    if let Some(route) = ROUTES
        .iter()
        .find(|route| route.slug == query || route.aliases.contains(&query.as_str()))
    {
        return Ok(route);
    }
    let matches: Vec<&'static Route> = ROUTES
        .iter()
        .filter(|route| {
            route.slug.starts_with(&query)
                || route.aliases.iter().any(|alias| alias.starts_with(&query))
        })
        .collect();
    match matches.as_slice() {
        [] => Err(Vec::new()),
        [only] => Ok(only),
        many => Err(many.iter().map(|route| route.slug).collect()),
    }
}

/// Resolve one lowercased `realm ` command body.
///
/// Returns `None` while the developer gate is closed and for anything that is
/// not the `game` keyword, so `realm goto`, the ordinary opening commands and
/// every unknown command keep their existing handling.
pub fn handle(enabled: bool, text: &str, actors: &[ActorView<'_>]) -> Option<Command> {
    if !enabled {
        return None;
    }
    let text = text.trim();
    let remainder = text.strip_prefix(COMMAND)?;
    // A keyword boundary, so `gamex` stays an ordinary unknown command.
    if remainder.chars().next().is_some_and(|c| !c.is_whitespace()) {
        return None;
    }
    let query = remainder.trim();
    if query.is_empty() || query.eq_ignore_ascii_case("list") {
        return Some(Command::Reply(listing()));
    }
    match resolve(query) {
        Ok(route) => Some(plan(route, actors)),
        Err(candidates) if candidates.is_empty() => Some(Command::Reply(usage())),
        Err(candidates) => Some(Command::Reply(format!(
            "Developer route '{query}' is ambiguous: {}. Nothing changed.",
            candidates.join(", ")
        ))),
    }
}

fn listing() -> String {
    let mut text = String::from(
        "Developer minigame routes (gate --dev-tools, off by default; nothing is granted):",
    );
    for route in ROUTES {
        let entry = match route.entry {
            Entry::WorldActor { actor, .. } => format!("world actor {actor}"),
            Entry::ModuleDriver { .. } => "module driver".to_string(),
            Entry::Unrecovered { .. } => "no launch route".to_string(),
        };
        text.push_str(&format!(
            "\n  {} [{}] {} -> {entry}",
            route.slug,
            route.status.label(),
            route.title
        ));
    }
    text.push_str("\nUsage: realm game <route>, realm game list.");
    text
}

fn usage() -> String {
    let slugs: Vec<&str> = ROUTES.iter().map(|route| route.slug).collect();
    format!(
        "Developer route usage: realm game, realm game list, or realm game <route>. Known routes: {}.",
        slugs.join(", ")
    )
}

fn plan(route: &Route, actors: &[ActorView<'_>]) -> Command {
    match route.entry {
        Entry::WorldActor {
            actor,
            profile,
            recipe,
        } => {
            let loaded: Vec<&ActorView<'_>> = actors
                .iter()
                .filter(|view| view.name == actor && view.zone_id == WORLD_ZONE)
                .collect();
            match loaded.as_slice() {
                [] => Command::Reply(format!(
                    "Developer route {} needs its entry actor '{actor}' in zone {WORLD_ZONE}, and the loaded content does not provide it. Nothing changed.",
                    route.slug
                )),
                [only] => Command::Route {
                    reply: format!(
                        "Developer route {}: {} [{}].\nEntry actor '{actor}' (GUID {}) in zone {WORLD_ZONE}; switch to profile {profile} and use the ordinary native interaction (recipe {recipe}).\n{}\nRelocation is position-only and the interaction still runs the ordinary rules.",
                        route.slug,
                        route.title,
                        route.status.label(),
                        only.guid,
                        route.native_gate
                    ),
                    actor,
                    guid: only.guid,
                },
                many => Command::Reply(format!(
                    "Developer route {} found {} actors named '{actor}' in zone {WORLD_ZONE}; the content is ambiguous. Nothing changed.",
                    route.slug,
                    many.len()
                )),
            }
        }
        Entry::ModuleDriver { api, driver } => Command::Reply(format!(
            "Developer route {}: {} [{}].\nModule {}; API {}.\nDriver: {driver}\n{}",
            route.slug,
            route.title,
            route.status.label(),
            route.module_path,
            api,
            route.native_gate
        )),
        Entry::Unrecovered { missing } => Command::Reply(format!(
            "Developer route {}: {} [{}].\n{missing}\n{}",
            route.slug,
            route.title,
            route.status.label(),
            route.native_gate
        )),
    }
}
