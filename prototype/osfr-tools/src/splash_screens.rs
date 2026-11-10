//! Loading (splash) screens: which screens exist, which each destination shows, and the two ways
//! the 2014 client takes them (P-games-window).
//!
//! Client evidence (2014 `FreeRealms.exe`):
//! * The client picks its own loading screen. `LoadingScreen.xml` in the install folder
//!   (`00caae20`) lists `LoadingScreen` rows (Id, Filename, Weight clamped to 1..=1000) and
//!   `LoadingScreenText` tips (StringId, Weight). A row counts when its member, Country, Locale,
//!   age and StartDate/EndDate tests pass (`00caa9e0`, `00caa1a0`) and its movie is held
//!   (`00caa4a0`). Each loading screen draws one row by weight (`00cab260`); only the launch
//!   arguments `SpecialLoadingScreenId` and `ShowMemberLoadingScreen` (`009161d0`) pin one, for
//!   the whole session. No packet names a screen for a zone or a minigame.
//! * The one movie the server names: ship combat's ZoneConfig carries a movie per dock, its last
//!   string (reader `00a5b660`, `TutorialMovieFilename` in the server loader), and `00a6cdf0`
//!   keeps the local player's dock's movie (`00a5c8b0`). It is the client's Tutorial-state movie,
//!   not a loading card: `00a6a4b0` draws it only while the tutorial flag of game state 7 is on
//!   (`00a6b0c0`). The install holds 18 such movies, `loadingscreen_pir<map>_<colour>_bbe`.
//!
//! So the table serves two outputs: [`Table::loading_screen_xml`], the file a launcher writes into
//! the install, and [`Table::dock_movies`] for ZoneConfig, which stays off (`dock_movies`) until
//! ship combat holds a Tutorial state. The per-destination rows beyond those are (R) and wait for
//! a client that can be told a screen.

use serde::Deserialize;
use std::{collections::BTreeMap, fmt::Write as _, io, path::Path};

pub const FILE: &str = "splash_screens.json";
const MAX_FILE_BYTES: u64 = 256 * 1024;
/// The client clamps a weight to this range (`00caae20`).
const WEIGHTS: std::ops::RangeInclusive<u32> = 1..=1000;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Screen {
    /// The row's Id in `LoadingScreen.xml`; the client skips a repeated id.
    pub id: u32,
    /// The movie, as the client names it (`.swf`; it looks for the `.lst` and `.gfx` itself).
    pub file: String,
    #[serde(default)]
    pub about: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pick {
    pub screen: u32,
    pub weight: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tip {
    pub string_id: u32,
    pub weight: u32,
}

/// How a destination with several screens rotates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rotation {
    /// The client's own rule: a draw by weight each time.
    Weighted,
    /// One after another, in the listed order.
    Sequence,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Table {
    schema_version: u32,
    #[serde(default)]
    source: String,
    pub rotation: Rotation,
    /// Whether ZoneConfig names the dock movies ([`Table::dock_movies`]). Off: every name is empty.
    #[serde(default)]
    pub dock_movies: bool,
    #[serde(default)]
    dock_movies_note: String,
    pub screens: Vec<Screen>,
    /// What a destination with no row of its own shows, and the rows of `LoadingScreen.xml`.
    pub default: Vec<Pick>,
    /// `zone:<id>`, `category:<id>`, `game:<id>`, `event:<key>`, `dock:<scene>:<colour>`.
    pub destinations: BTreeMap<String, Vec<Pick>>,
    pub tips: Vec<Tip>,
}

impl Table {
    pub fn load(directory: &Path) -> io::Result<Option<Self>> {
        let path = directory.join(FILE);
        if !path.is_file() {
            return Ok(None);
        }
        if path.metadata()?.len() > MAX_FILE_BYTES {
            return Err(invalid("splash_screens.json is too large"));
        }
        let table: Self = serde_json::from_slice(&std::fs::read(path)?).map_err(invalid)?;
        table.validate()?;
        Ok(Some(table))
    }

    fn validate(&self) -> io::Result<()> {
        let _ = (&self.source, &self.dock_movies_note);
        let mut ids = std::collections::BTreeSet::new();
        let file = |name: &str| {
            name.ends_with(".swf")
                && name.len() <= 63
                && name.bytes().all(|b| b.is_ascii_alphanumeric() || b"_.".contains(&b))
        };
        let picks = (self.destinations.values().chain([&self.default])).flatten();
        if self.schema_version != 1
            || self.default.is_empty()
            || self.destinations.values().any(Vec::is_empty)
            || !self.screens.iter().all(|s| s.id > 0 && file(&s.file) && ids.insert(s.id))
            || !picks.into_iter().all(|p| ids.contains(&p.screen) && WEIGHTS.contains(&p.weight))
            || !self.tips.iter().all(|t| t.string_id > 0 && WEIGHTS.contains(&t.weight))
        {
            return Err(invalid("invalid splash_screens.json"));
        }
        Ok(())
    }

    fn screen(&self, id: u32) -> Option<&Screen> {
        self.screens.iter().find(|screen| screen.id == id)
    }

    /// The first of `keys` (most specific first) that has screens of its own.
    pub fn own(&self, keys: &[&str]) -> Option<&[Pick]> {
        keys.iter().find_map(|key| self.destinations.get(*key)).map(Vec::as_slice)
    }

    /// The screen `keys` shows on its `turn`th load: the destination's rows (else the default
    /// rotation), by the table's rule. A weighted draw uses `turn` as its roll.
    pub fn pick(&self, keys: &[&str], turn: u64) -> Option<&Screen> {
        let picks = self.own(keys).unwrap_or(&self.default);
        let total: u64 = picks.iter().map(|p| u64::from(p.weight)).sum();
        let chosen = match self.rotation {
            Rotation::Sequence => picks.get((turn % picks.len().max(1) as u64) as usize),
            Rotation::Weighted => {
                let mut roll = turn % total.max(1);
                picks.iter().find(|p| {
                    let hit = roll < u64::from(p.weight);
                    roll = roll.saturating_sub(u64::from(p.weight));
                    hit
                })
            }
        }?;
        self.screen(chosen.screen)
    }

    /// `LoadingScreen.xml` as the client reads it: the default rotation, then the screens of the
    /// `events` running now (`event:<key>` rows, the events framework's hook), then the tips. No
    /// row is limited by membership, country, locale or date: the server decides what is listed.
    pub fn loading_screen_xml(&self, events: &[String]) -> String {
        let live = events.iter().filter_map(|key| self.destinations.get(&format!("event:{key}")));
        let mut seen = Vec::new();
        let mut xml = String::from("<LoadingScreens>\n");
        for pick in self.default.iter().chain(live.flatten()) {
            let Some(screen) = self.screen(pick.screen).filter(|s| !seen.contains(&s.id)) else {
                continue;
            };
            seen.push(screen.id);
            let _ = writeln!(
                xml,
                "\t<LoadingScreen Id={} Filename=\"{}\" ShowToMembers=1 ShowToNonMembers=1 Weight={}/>",
                screen.id, screen.file, pick.weight
            );
        }
        xml.push_str("</LoadingScreens>\n\n<LoadingScreenText>\n");
        for tip in &self.tips {
            let _ = writeln!(
                xml,
                "\t<Text StringId={} ShowToMembers=1 ShowToNonMembers=1 Weight={}/>",
                tip.string_id, tip.weight
            );
        }
        xml.push_str("</LoadingScreenText>\n");
        xml
    }

    /// Ship combat's tutorial movie per dock slot for the arena `scene`: `dock:<scene>:<colour>`,
    /// an empty name where a dock has none or while the `dock_movies` switch is off. `turn`
    /// rotates a dock with several.
    pub fn dock_movies(&self, scene: &str, colours: &[String], turn: u64) -> Vec<String> {
        let movie = |colour: &String| {
            let key = format!("dock:{scene}:{colour}");
            (self.own(&[&key]).filter(|_| self.dock_movies)).and_then(|_| self.pick(&[&key], turn))
        };
        colours.iter().map(|c| movie(c).map(|s| s.file.clone()).unwrap_or_default()).collect()
    }
}

fn invalid(error: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}
