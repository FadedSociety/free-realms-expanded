//! L095/L097: Games window glue for `Gameplay`: the live rows over the assembled catalogue, the
//! per-player lists, and Play for the kart race and the derby. The rows and their joins live in
//! `crate::activity_portal`; the race and derby entries are the speedway gates' own.
use super::*;
use crate::{
    activity_portal::{Activity, Catalogue, Kind, game_row},
    derby_gateway::{DerbyEntryDenied, DerbyGateway},
    kart_gateway::{EntryDenied, KartGateway},
    recipes::RecipeCatalog,
};

/// The live Games window rows (from the first of `folders` that has them) over `catalogue`, with
/// the wheel and the kart/derby rows the server runs.
pub(super) fn with_live(
    catalogue: Catalogue,
    folders: [&Path; 2],
    wheel: &crate::daily_wheel::DailyWheel,
    kart: Option<&KartGateway>,
    derby: Option<&DerbyGateway>,
    recipes: &RecipeCatalog,
) -> io::Result<Catalogue> {
    let live = (folders.into_iter())
        .find(|base| base.join(crate::activity_portal::CATALOGUE_FILE).is_file())
        .map_or_else(|| Ok(Vec::new()), crate::activity_portal::load_live)?;
    catalogue.with_live(live, wheel.activity_id(), games(kart, derby, recipes))
}

/// The kart race, the default derby and (L098) every recipe with a client recipe id and a
/// MiniGameData row, as Games window join rows keyed by their MiniGameData rows.
pub(super) fn games(
    kart: Option<&KartGateway>,
    derby: Option<&DerbyGateway>,
    recipes: &RecipeCatalog,
) -> Vec<Activity> {
    let kart = kart.and_then(KartGateway::minigame_id);
    let derby = derby.and_then(|derby| u32::try_from(derby.game_id()).ok());
    let mut seen = std::collections::BTreeSet::new();
    let crafts = [
        CraftingJob::Chef,
        CraftingJob::Miner,
        CraftingJob::Blacksmith,
    ]
    .into_iter()
    .flat_map(|job| recipes.by_job(job))
    .filter_map(|recipe| Some((recipe.minigame_id?, recipe.recipe_id?)))
    .filter(|(minigame, _)| seen.insert(*minigame))
    .map(|(minigame, recipe_id)| game_row(minigame, Kind::Recipe { recipe_id }));
    (kart.map(|id| game_row(id, Kind::Kart)).into_iter())
        .chain(derby.map(|id| game_row(id, Kind::Derby)))
        .chain(crafts)
        .collect()
}

impl Gameplay {
    /// W150: the portal lists for this player (167/1/1); the live minigame rows travel in their
    /// own entry-type list (L095).
    pub(super) fn append_activity_list(&self, out: &mut Outcome) {
        let viewer = self.activity_viewer();
        out.replies.push(self.activities.list_packet(viewer));
        out.replies
            .extend(self.activities.minigame_list_packet(viewer));
    }

    /// L097: Play on the kart race's or the derby's Games window row: the gate's admission without
    /// the gate, then the gate's entry; L098: a recipe row through the recipe book. A refusal
    /// answers with one chat line and changes nothing.
    pub(super) fn begin_portal_game(
        &mut self,
        kind: Kind,
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Outcome> {
        let busy = self.activity_busy();
        let refused = |text: &str| Outcome {
            replies: vec![chat::system(text)],
            ..Outcome::default()
        };
        match kind {
            Kind::Kart => match self.kart_entry.as_ref().map(|kart| kart.admit_play(busy)) {
                Some(Ok(())) => self.commit_if_changed(|staged| staged.begin_kart_race(character)),
                Some(Err(denied)) => Ok(refused(denied.message())),
                None => Ok(refused(EntryDenied::NoRace.message())),
            },
            Kind::Derby => match self.derby.as_ref().map(|derby| derby.admit_play(busy)) {
                Some(Ok(())) => {
                    self.commit_if_changed(|staged| staged.begin_derby(character, now_ms, None))
                }
                Some(Err(denied)) => Ok(refused(denied.message())),
                None => Ok(refused(DerbyEntryDenied::Unavailable.message())),
            },
            // L098: a recipe row starts the recipe book anywhere (R).
            Kind::Recipe { recipe_id } => {
                let mut out = Outcome::default();
                let recipe = i32::try_from(recipe_id).map_err(|_| invalid("recipe id"))?;
                self.start_recipe(recipe, true, character, now_ms, &mut out)?;
                Ok(out)
            }
            _ => Err(invalid("not a Games window game row")),
        }
    }
}
