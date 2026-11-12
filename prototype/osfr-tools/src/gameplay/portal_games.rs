//! L095/L097: Games window glue for `Gameplay`: the live rows over the assembled catalogue, the
//! per-player lists, and Play for the kart race and the derby. The rows and their joins live in
//! `crate::activity_portal`; the race and derby entries are the speedway gates' own.
use super::*;
use crate::{
    activity_portal::{Activity, Catalogue, Kind, MISSING_JOB_TEXT, game_row, join_error},
    derby_gateway::{DerbyEntryDenied, DerbyGateway},
    fishing_service::FishingSpots,
    harvest_nodes::{Denied, payout_fits},
    kart_gateway::{EntryDenied, KartGateway},
    music_game::MusicGame,
    spot_game::SpotGame,
    sword_game::SwordGame,
    recipes::RecipeCatalog,
    tower_defense::TowerDefense,
};

/// The live Games window rows (from the first of `folders` that has them) over `catalogue`, with
/// the wheel and the service's own `games`.
pub(super) fn with_live(
    catalogue: Catalogue,
    folders: [&Path; 2],
    wheel: &crate::daily_wheel::DailyWheel,
    games: Vec<Activity>,
) -> io::Result<Catalogue> {
    let live = (folders.into_iter())
        .find(|base| base.join(crate::activity_portal::CATALOGUE_FILE).is_file())
        .map_or_else(|| Ok(Vec::new()), crate::activity_portal::load_live)?;
    catalogue.with_live(live, wheel.activity_id(), games)
}

/// The kart race, the default derby, (L098) every recipe with a client recipe id and a
/// MiniGameData row, (L100) every game the mining, harvesting and (L103) mail sorting engines
/// (`type8`, in that order) offer and (L102) every fishing hole, as Games window join rows keyed
/// by their MiniGameData rows, and (L104) the checkers table. (L961) Every Feeling the Beat!
/// game `music_game.json` carries, (L962) every Tower Defense game `tower_defense.json` carries and
/// (L963) Pirate's Plunder when `ship_combat.json` is present.
pub(super) fn games(
    kart: Option<&KartGateway>,
    derby: Option<&DerbyGateway>,
    recipes: &RecipeCatalog,
    type8: [Option<&Type8Engine>; 4],
    fishing: Option<&FishingSpots>,
    music: Option<&MusicGame>,
    (sword, spot): (Option<&SwordGame>, Option<&SpotGame>),
    tower_defense: Option<&TowerDefense>,
    pirates: bool,
) -> Vec<Activity> {
    let garage = kart.is_some().then(|| game_row(super::kart_garage::GARAGE_ROW, Kind::Garage));
    let kart = kart.map(KartGateway::minigame_ids).unwrap_or_default();
    // L931: every hosted derby row is a Games window join key, so Play starts the row chosen.
    let derby = derby.into_iter().flat_map(|derby| {
        (derby.rows().iter()).filter_map(|row| u32::try_from(row.game_id()).ok())
    });
    let mut seen = std::collections::BTreeSet::new();
    let crafts = [
        CraftingJob::Chef,
        CraftingJob::Miner,
        CraftingJob::Blacksmith,
    ]
    .into_iter()
    .flat_map(|job| recipes.by_job(job))
    .filter_map(|recipe| Some((recipe.minigame_id?, recipe.recipe_id?, recipe)))
    .filter(|(minigame, ..)| seen.insert(*minigame))
    .map(|(minigame, recipe_id, recipe)| Activity {
        required_level: recipe.required_job_rank.map_or(0, u32::from),
        ..game_row(
            minigame,
            Kind::Recipe {
                recipe_id,
                job: recipe.job,
            },
        )
    });
    let gathers = [Gather::Mining, Gather::Harvesting, Gather::Mail, Gather::Artifact]
        .into_iter()
        .zip(type8)
        .filter_map(|(gather, engine)| Some((gather, engine?)))
        .flat_map(|(gather, engine)| {
            (engine.games()).filter_map(move |(game, _)| {
                // L106/L934: a mail town's or an ore's row asks its job rank floor.
                Some(Activity {
                    required_level: gather.rank_floor(game),
                    ..game_row(u32::try_from(game).ok()?, Kind::Gather { gather })
                })
            })
        });
    let holes = (fishing.into_iter().flat_map(FishingSpots::games)).filter_map(|(guide, game)| {
        Some(game_row(u32::try_from(game).ok()?, Kind::Fishing { guide }))
    });
    let songs = (music.into_iter().flat_map(MusicGame::game_ids))
        .filter_map(|game| Some(game_row(u32::try_from(game).ok()?, Kind::Music)));
    let spots = (spot.into_iter().flat_map(SpotGame::game_ids))
        .filter_map(|game| Some(game_row(u32::try_from(game).ok()?, Kind::Spot)));
    let maps = (tower_defense.into_iter().flat_map(TowerDefense::game_ids))
        .filter_map(|game| Some(game_row(u32::try_from(game).ok()?, Kind::TowerDefense)));
    (kart.into_iter().map(|id| game_row(id, Kind::Kart)))
        .chain(derby.map(|id| game_row(id, Kind::Derby)))
        .chain(crafts)
        .chain(gathers)
        .chain(holes)
        .chain([game_row(crate::checkers::GAME_ID as u32, Kind::Checkers)])
        .chain(songs)
        .chain(spots)
        .chain(sword.and_then(|game| Some(game_row(u32::try_from(game.game_id()).ok()?, Kind::Sword))))
        .chain(maps)
        .chain(pirates.then(|| game_row(crate::ship_combat::GAME_ID, Kind::Pirates)))
        .chain(garage)
        .collect()
}

impl Gameplay {
    pub(super) fn activity_viewer(&self) -> crate::activity_portal::Viewer<'_> {
        static NO_JOBS: BTreeSet<i32> = BTreeSet::new();
        let profile = self.profiles.as_ref().map_or(0, Profiles::active);
        crate::activity_portal::Viewer {
            combat_job: self
                .abilities
                .as_ref()
                .is_some_and(|a| a.supports_basic(profile)),
            level: self.job_level(profile),
            crafts: CraftingJob::ALL.map(|job| self.job_level(job.profile_id())),
            postman: self.job_level(crate::mail_sorting::POSTMAN_PROFILE_ID),
            jobs: (self.profiles.as_ref()).map_or(&NO_JOBS, |p| &p.state().unlocked),
            learned: &self.learned_recipes,
        }
    }

    /// The facts a list's CanPlayerJoin rests on that the client never asks again for: a job
    /// unlocked or a recipe learned (it sends 167/1/4 on a rank change only).
    fn portal_basis(&self) -> (usize, usize) {
        let jobs = (self.profiles.as_ref()).map_or(0, |p| p.state().unlocked.len());
        (jobs, self.learned_recipes.len())
    }

    /// Push a fresh list after a packet that unlocked a job or taught a recipe, so the rows it
    /// opens stop showing greyed. Nothing before the first list.
    pub(super) fn sync_activity_list(&mut self, out: &mut Outcome) {
        if self
            .portal_listed
            .is_some_and(|basis| basis != self.portal_basis())
        {
            self.append_activity_list(out);
        }
    }

    /// W150: the portal lists for this player (167/1/1); the live minigame rows travel in their
    /// own entry-type list (L095).
    pub(super) fn append_activity_list(&mut self, out: &mut Outcome) {
        let viewer = self.activity_viewer();
        out.replies.push(self.activities.list_packet(viewer));
        out.replies
            .extend(self.activities.minigame_list_packet(viewer));
        self.portal_listed = Some(self.portal_basis());
    }

    /// L097: Play on the kart race's or the derby's Games window row: the gate's admission without
    /// the gate, then the gate's entry; L098: a recipe row through the recipe book; L100: game
    /// `id` of a gather row; L102: a fishing hole. A refusal answers with one line (B4: the
    /// client's own 147/2 alert for a missing job, else chat) and changes nothing.
    pub(super) fn begin_portal_game(
        &mut self,
        id: u32,
        kind: Kind,
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Outcome> {
        let busy = self.activity_busy();
        let refused = |text: &str| Outcome {
            replies: vec![chat::system(text)],
            ..Outcome::default()
        };
        let alert = |text_id: u32| Outcome {
            replies: vec![join_error(text_id)],
            ..Outcome::default()
        };
        match kind {
            Kind::Kart => match self.kart_entry.as_ref().map(|kart| kart.admit_play(busy)) {
                Some(Ok(())) => {
                    self.commit_if_changed(|staged| staged.begin_kart_row(character, Some(id)))
                }
                Some(Err(denied)) => Ok(refused(denied.message())),
                None => Ok(refused(EntryDenied::NoRace.message())),
            },
            Kind::Derby => match self.derby.as_ref().map(|derby| derby.admit_play(busy)) {
                Some(Ok(())) => self.commit_if_changed(|staged| {
                    staged.begin_derby(character, now_ms, i32::try_from(id).ok())
                }),
                Some(Err(denied)) => Ok(refused(denied.message())),
                None => Ok(refused(DerbyEntryDenied::Unavailable.message())),
            },
            // L098: a recipe row starts the recipe book anywhere (R).
            Kind::Recipe { recipe_id, .. } => {
                let recipe = i32::try_from(recipe_id).map_err(|_| invalid("recipe id"))?;
                let mut out = Outcome::default();
                self.start_recipe(recipe, true, character, now_ms, &mut out)?;
                Ok(out)
            }
            // L100 (R): a node's gates without the node: idle, the gather job unlocked, the game
            // offered and its whole payout fitting the bag. L936: the start switches into the job.
            Kind::Gather { gather } => {
                let game = i32::try_from(id).map_err(|_| invalid("gather game id"))?;
                let profiles = self
                    .profiles
                    .as_ref()
                    .ok_or_else(|| invalid("no profiles"))?;
                let admitted = if busy {
                    Err(Denied::Busy)
                } else {
                    gather.admit_job(&profiles.state().unlocked)
                };
                match admitted {
                    Err(Denied::NoJob(_)) => return Ok(alert(MISSING_JOB_TEXT)),
                    Err(denied) => return Ok(refused(&denied.message())),
                    Ok(()) => {}
                }
                if self.job_level(gather.job()) < gather.rank_floor(game) {
                    return Ok(refused(&format!(
                        "Your {} rank is too low for that game.",
                        gather.word()
                    )));
                }
                let Some(spec) = self.type8(gather).and_then(|e| e.spec_for(Some(game))) else {
                    return Ok(refused("Gathering is not available in this campaign."));
                };
                if !payout_fits(&self.inventory, spec) {
                    return Ok(refused("Your bag is full. Make room before gathering."));
                }
                self.commit_if_changed(|staged| {
                    let mut out = Outcome::default();
                    let guid = character.guid;
                    staged.begin_gather_in_job(gather, Some(game), guid, now_ms, &mut out)?;
                    Ok(out)
                })
            }
            // L102 (R): the guide's round without the guide; `begin_fishing` stages itself.
            Kind::Fishing { guide } => {
                self.commit_if_changed(|staged| staged.begin_fishing(guide, character))
            }
            // L961: the server's own movies start nothing during another activity (a groom session
            // keeps travel open, so the portal's travel gate alone lets them through).
            Kind::Checkers | Kind::Music | Kind::Sword | Kind::Spot | Kind::TowerDefense | Kind::Pirates | Kind::Garage
                if busy =>
            {
                Ok(refused(&Denied::Busy.message()))
            }
            // L104: the checkers table (it stages itself).
            Kind::Checkers => self.begin_checkers(),
            // L961: Feeling the Beat! game `id` (it stages itself).
            Kind::Music => self.begin_music(id),
            Kind::Sword => self.begin_sword(),
            Kind::Spot => self.begin_spot(id),
            // L962: Tower Defense game `id` (it stages itself).
            Kind::TowerDefense => self.begin_tower_defense(id),
            // L963: a solo match on any of the maps (R: the live game matched one to five players
            // through queue 5). A refusal answers with its line.
            Kind::Pirates => self.commit_if_changed(|staged| {
                let mut out = Outcome::default();
                let text = staged.begin_pirates(0, character, &mut out)?;
                if !out.changed {
                    out.replies.push(chat::system(&text));
                }
                Ok(out)
            }),
            // The garage zones in as a race does.
            Kind::Garage => self.commit_if_changed(|staged| {
                let mut out = Outcome::default();
                staged.begin_garage(character, &mut out)?;
                Ok(out)
            }),
            _ => Err(invalid("not a Games window game row")),
        }
    }
}
