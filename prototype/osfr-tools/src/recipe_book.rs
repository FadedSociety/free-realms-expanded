//! Recipe-book start (client request 26/24) at a real crafting station.
//!
//! Recovered client facts: the recipe window calls `Ui.RecipeStart(recipe)`
//! (`ScriptsBase.lua:29430,29543,43142`), native `RequestRecipeStart 00c23970`
//! builds `CommandPacketRecipeStart` via `00c14b30` (command sub-opcode 0x18,
//! recipe dword at +0x10) and sends it through `00c22460`. The serializer
//! `00c21290` writes the u16 at +4 (family 26), the u16 at +8 (sub 24) and the
//! dword at +0x10, and nothing else: the body is exactly
//! `26,0, 24,0, recipe:i32`. The client sends only the recipe id; job,
//! station, ingredients and rank are never client-supplied.
//!
//! Everything below the decode is reconstructed server policy (R): the
//! original station and learned-recipe rules were not recovered, so a start
//! needs a learned recipe, the matching active job, an idle crafting service,
//! a server-known station of that job within its interaction radius, and
//! `begin_recipe`'s own rank and exact-ingredient gate. Every refusal happens
//! before any state changes and carries a chat line for the player.
use crate::{
    chat,
    cooking::{Cooking, CookingOutcome},
    forging::{Forging, ForgingOutcome},
    opening::{OpeningActor, OpeningTrigger},
    recipes::{CraftingJob, RecipeCatalog},
    smelting::{Smelting, SmeltingOutcome},
};
use std::{collections::BTreeSet, io};

pub const FAMILY: u16 = 26;
pub const SUB: u16 = 24;
/// 26/24 is family, sub and one i32.
const LEN: usize = 8;

/// Decode a 26/24 body. `Ok(None)` means another message; a recognised header
/// with any other length is malformed (hostile or corrupt).
pub fn decode(inner: &[u8]) -> io::Result<Option<i32>> {
    if inner.len() < 4 || inner[..2] != FAMILY.to_le_bytes() || inner[2..4] != SUB.to_le_bytes() {
        return Ok(None);
    }
    if inner.len() != LEN {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "malformed recipe start request",
        ));
    }
    Ok(Some(i32::from_le_bytes(inner[4..8].try_into().unwrap())))
}

/// A crafting station the server knows about.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Station {
    pub job: CraftingJob,
    pub zone_id: u32,
    pub position: [f32; 3],
    pub radius: f32,
}

/// (R) The only stations the server knows are the opening world's craft
/// actors, whose objective ids 2001/2002/2003 are the stove, furnace and anvil
/// (the same binding `gameplay.rs` already uses for the tutorial crafts).
pub fn stations(actors: &[OpeningActor]) -> Vec<Station> {
    actors
        .iter()
        .filter_map(|actor| {
            let job = match actor.action {
                Some(OpeningTrigger::Craft { recipe_id: 2001 }) => CraftingJob::Chef,
                Some(OpeningTrigger::Craft { recipe_id: 2002 }) => CraftingJob::Miner,
                Some(OpeningTrigger::Craft { recipe_id: 2003 }) => CraftingJob::Blacksmith,
                _ => return None,
            };
            Some(Station {
                job,
                zone_id: actor.zone_id,
                position: actor.position,
                radius: actor.interaction_radius,
            })
        })
        .collect()
}

/// Server-owned facts about the requesting player.
pub struct Player<'a> {
    pub zone_id: u32,
    pub position: [f32; 3],
    pub active_profile: i32,
    /// Rank of `active_profile`.
    pub job_rank: u8,
    pub learned: &'a BTreeSet<u32>,
    /// Owned, non-rental item totals.
    pub available: &'a [(i32, u32)],
    pub in_encounter: bool,
}

/// The three crafting services; `None` when that movie is not loaded.
pub struct Services<'a> {
    pub cooking: Option<&'a mut Cooking>,
    pub smelting: Option<&'a mut Smelting>,
    pub forging: Option<&'a mut Forging>,
}

/// The started run. The caller applies it exactly like a tutorial start.
pub enum Started {
    Cooking(CookingOutcome),
    Smelting(SmeltingOutcome),
    Forging(ForgingOutcome),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Busy,
    UnknownRecipe,
    NotLearned,
    WrongJob(CraftingJob),
    NoStation(CraftingJob),
    RankTooLow,
    MissingIngredients,
    Unavailable,
}

impl Refusal {
    pub fn message(&self) -> &'static str {
        match self {
            Self::Busy => "Finish what you are doing before you start crafting.",
            Self::UnknownRecipe => "That recipe is not available.",
            Self::NotLearned => "You have not learned that recipe yet.",
            Self::WrongJob(CraftingJob::Chef) => "Switch to Chef to make that recipe.",
            Self::WrongJob(CraftingJob::Blacksmith) => "Switch to Blacksmith to make that recipe.",
            Self::WrongJob(CraftingJob::Miner) => "Switch to Miner to make that recipe.",
            Self::NoStation(CraftingJob::Chef) => "Move closer to a cooking station.",
            Self::NoStation(CraftingJob::Blacksmith) => "Move closer to an anvil.",
            Self::NoStation(CraftingJob::Miner) => "Move closer to a furnace.",
            Self::RankTooLow => "Your job rank is too low for that recipe.",
            Self::MissingIngredients => "You do not have the ingredients for that recipe.",
            Self::Unavailable => "That recipe cannot be made right now.",
        }
    }

    /// Player-visible chat line (15/3 system message).
    pub fn feedback(&self) -> Vec<u8> {
        chat::system(self.message())
    }
}

fn near(station: &Station, player: &Player) -> bool {
    let distance_squared: f32 = station
        .position
        .iter()
        .zip(player.position)
        .map(|(a, b)| (a - b).powi(2))
        .sum();
    // NaN or infinite coordinates make this false.
    station.zone_id == player.zone_id && distance_squared <= station.radius.powi(2)
}

fn has_ingredients(required: &[(i32, u32)], available: &[(i32, u32)]) -> bool {
    required.iter().all(|(definition, needed)| {
        available
            .iter()
            .filter(|(id, _)| id == definition)
            .map(|(_, count)| u64::from(*count))
            .sum::<u64>()
            >= u64::from(*needed)
    })
}

/// Start the recipe the client named. Validates in the order that reveals the
/// least (unknown before unlearned before location), then delegates to the
/// job service's own `begin_recipe`, which re-checks learned, rank and exact
/// ingredients. No state changes on any `Err`.
pub fn start(
    recipe_id: i32,
    catalog: &RecipeCatalog,
    player: &Player,
    stations: &[Station],
    services: Services,
) -> Result<Started, Refusal> {
    if player.in_encounter {
        return Err(Refusal::Busy);
    }
    let recipe = u32::try_from(recipe_id)
        .ok()
        .and_then(|id| {
            catalog
                .records
                .iter()
                .find(|recipe| recipe.recipe_id == Some(id))
        })
        .ok_or(Refusal::UnknownRecipe)?;
    if !recipe
        .recipe_id
        .is_some_and(|id| player.learned.contains(&id))
    {
        return Err(Refusal::NotLearned);
    }
    if recipe.job.profile_id() != player.active_profile {
        return Err(Refusal::WrongJob(recipe.job));
    }
    let Services {
        cooking,
        smelting,
        forging,
    } = services;
    if cooking.as_deref().is_some_and(Cooking::active)
        || smelting.as_deref().is_some_and(Smelting::active)
        || forging.as_deref().is_some_and(Forging::active)
    {
        return Err(Refusal::Busy);
    }
    if !stations
        .iter()
        .any(|station| station.job == recipe.job && near(station, player))
    {
        return Err(Refusal::NoStation(recipe.job));
    }
    if recipe
        .required_job_rank
        .is_some_and(|required| player.job_rank < required)
    {
        return Err(Refusal::RankTooLow);
    }
    let required: Vec<(i32, u32)> = recipe
        .ingredients
        .iter()
        .map(|item| (item.definition_id, item.count))
        .collect();
    if !has_ingredients(&required, player.available) {
        return Err(Refusal::MissingIngredients);
    }
    let profile = recipe.job.profile_id();
    let key = recipe.key.as_str();
    let (rank, available) = (player.job_rank, player.available);
    // `learned` is true: checked above against the same server-owned set.
    // Any service-side rejection (a recipe it does not carry) is Unavailable.
    let started = match recipe.job {
        CraftingJob::Chef => cooking
            .ok_or(Refusal::Unavailable)?
            .begin_recipe(profile, key, rank, true, available)
            .map(Started::Cooking),
        CraftingJob::Miner => smelting
            .ok_or(Refusal::Unavailable)?
            .begin_recipe(profile, key, rank, true, available)
            .map(Started::Smelting),
        CraftingJob::Blacksmith => forging
            .ok_or(Refusal::Unavailable)?
            .begin_recipe(profile, key, rank, true, available)
            .map(Started::Forging),
    };
    started.map_err(|_| Refusal::Unavailable)
}
