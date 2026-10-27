//! T533: pet treasure. A summoned, settled, non-vanity pet in a good mood sometimes sniffs, digs and
//! unearths a chest that only its owner sees and can open.
//!
//! The pet hunts while it stands settled beside a still owner (the T455 settle): after a random
//! wait it rolls `find_chance`, sniffs, digs, and the chest appears between the pet and its owner.
//! The tier (Common, Rare, Elite) and the contents are rolled when the chest appears, from the
//! service's own generator (OS entropy in production), never from anything the client sends. The
//! pet's cooldown starts at that moment and is saved with the roster (`Pet::treasure_found_at`, wall
//! clock), so a re-summon, a zone change or a reconnect rolls nothing again. The chest is session-only:
//! it lives `lifetime_ms`, a zone change removes it, a reconnect has none.
//!
//! Opening is two steps so a refusal changes nothing: `open_treasure` (read-only) checks the chest,
//! its owner, its lifetime and the range and returns the grant; the caller pays it through
//! `rewards::apply_grant` and only then calls `clear_treasure`, which removes the chest. The chest
//! is one `Option`, so a second open finds nothing.
//!
//! Client facts: model 151 (`Models.txt`, `sg_treasure_chest_01.adr`) declares the texture aliases
//! `Default1` to `Default3` (one chest texture per tier) and the `com_cast_01`/`com_cast_02` slots
//! (`sg_treasure_chest_open.gr2`/`sg_treasure_chest_close.gr2`). The brief does not ask for the open
//! animation, so opening only removes the actor (35/3); the original look would be that slot sent
//! before the removal. The locale names the tiers (140227 Common, 140234 Rare, 140235 Elite Pet
//! Treasure). `pet_sniff` 5520 and `pet_digging` 5525 are pet animation slots
//! (`AnimationTypes.xml`, type 4; 5525 is also the unrelated PLAYER_DRAW_EFFECT id in
//! `PetTrickGestureSettings.txt`); 16 of the 152 pet ADRs declare both, so the importer marks the
//! 21 model rows that use one of them (`PetModel::digs`) and every other pet hunts without the two
//! animations. A click on an interactable actor sends 26/8 (ctor `008b6bc0`: u64 guid, u8 flag;
//! `0093bff0` looks the actor up by guid and sends it through `0092efe0`); the AddNpc float
//! `unknown_float_51` is the client's X-prompt radius (T400). Every number and the reward tables
//! are reconstructed (R) from the Sanctuary reference (`src/Resources/Rewards.json` keys
//! `pet-treasure-*` and `Entities/PlayerPet.cs`; data only, no code) and live in the `treasure`
//! block of `pets.json` with their provenance.

use super::{
    ActivePet, PetService, PetWire, SourceTag, decayed, idle::PetAnimation, pet_mut,
    pet_removal_packet,
};
use crate::{
    power_ups::{Rng, pick_weighted},
    rewards::{ItemGrant, RewardDisplay, RewardGrant},
    startup_world, world,
    world_events::piece_052,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};

const MAX_TIERS: usize = 8;
const MAX_ROWS: usize = 64;
/// The longest cooldown the data may ask for: 30 days.
const MAX_COOLDOWN_MS: u64 = 30 * 24 * 3_600_000;
/// A chest's coins are a wire i32; this keeps a typo far from it.
const MAX_COINS: u32 = 1_000_000;
/// Chest actor guids: "PTCH", then a process-wide counter, so no two chests ever share a guid (not
/// across tiers, owners or sessions) and none aliases another actor family.
const CHEST_GUID_TAG: u64 = 0x5054_4348_0000_0000;
static NEXT_CHEST: AtomicU32 = AtomicU32::new(0);

/// One reward row of a tier: an item, how many, and its share of the tier's draw.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct TreasureItem {
    definition_id: i32,
    count: u32,
    weight: u32,
}

/// One chest quality: its draw weight, the name and look the client shows, and what it pays: the
/// coins always, plus one item drawn by weight.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct TreasureTier {
    key: String,
    weight: u32,
    /// The locale id the client shows as the chest's name (140227, 140234, 140235).
    name_id: i32,
    /// The ADR texture alias (`Default1` to `Default3`).
    texture_alias: String,
    coins: u32,
    items: Vec<TreasureItem>,
}

impl TreasureTier {
    fn valid(&self) -> bool {
        !self.key.is_empty()
            && self.key.len() <= 32
            && self.weight > 0
            && self.name_id > 0
            && !self.texture_alias.is_empty()
            && self.texture_alias.len() <= 64
            && self.coins <= MAX_COINS
            && self.items.len() <= MAX_ROWS
            && (self.coins > 0 || !self.items.is_empty())
            && self.items.iter().all(|row| {
                row.definition_id > 0 && (1..=999).contains(&row.count) && row.weight > 0
            })
    }
}

/// A rarer prize: with `chance`, one of `items` (one of each) replaces the tier's item.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct TreasureBonus {
    chance: f32,
    items: Vec<i32>,
}

impl TreasureBonus {
    fn valid(&self) -> bool {
        self.chance.is_finite()
            && (0.0..=1.0).contains(&self.chance)
            && self.items.len() <= MAX_ROWS
            && (self.chance == 0.0 || !self.items.is_empty())
            && self.items.iter().all(|id| *id > 0)
            && self.items.iter().collect::<BTreeSet<_>>().len() == self.items.len()
    }
}

/// The `treasure` block of `pets.json`; `valid` keeps it sane at content load. Without the block a
/// pet never digs.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TreasureConfig {
    /// A pet finds nothing while its mood is under this.
    min_mood: f32,
    /// A settled, happy pet rolls for a find after a random wait in this range.
    find_min_ms: u64,
    find_max_ms: u64,
    /// The chance that a due roll finds something. The one number a treasure-hunting vitamin would
    /// raise (out of scope for T533); a roll that fails waits again.
    find_chance: f32,
    /// How long a pet rests from the moment its chest appears.
    cooldown_ms: u64,
    /// How long a chest stays before it vanishes.
    lifetime_ms: u64,
    chest_model_id: i32,
    /// The chest appears this far from the pet, toward its owner (never past the owner).
    spawn_distance: f32,
    /// The owner may open the chest from this far; also the client's X-prompt radius.
    open_range: f32,
    sniff: PetAnimation,
    dig: PetAnimation,
    tiers: Vec<TreasureTier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bonus: Option<TreasureBonus>,
    source: SourceTag,
}

impl TreasureConfig {
    pub(super) fn valid(&self) -> bool {
        let within =
            |value: f32, low: f32, high: f32| value.is_finite() && (low..=high).contains(&value);
        let mut keys = BTreeSet::new();
        within(self.min_mood, 0.0, 100.0)
            && (1_000..=3_600_000).contains(&self.find_min_ms)
            && self.find_min_ms <= self.find_max_ms
            && self.find_max_ms <= 3_600_000
            && within(self.find_chance, 0.0, 1.0)
            && self.find_chance > 0.0
            && self.cooldown_ms <= MAX_COOLDOWN_MS
            && (1_000..=3_600_000).contains(&self.lifetime_ms)
            && self.chest_model_id > 0
            && within(self.spawn_distance, 0.0, 20.0)
            && within(self.open_range, 1.0, 50.0)
            && self.sniff.valid()
            && self.dig.valid()
            && (1..=MAX_TIERS).contains(&self.tiers.len())
            && self
                .tiers
                .iter()
                .all(|tier| tier.valid() && keys.insert(tier.key.as_str()))
            && self.bonus.as_ref().is_none_or(TreasureBonus::valid)
    }
}

/// Where the summoned pet is in its hunt (session only: every summon starts `Idle`, so no timer
/// outlives its pet).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Hunt {
    /// Not hunting: a condition fails, or nothing is armed yet.
    #[default]
    Idle,
    /// Hunting: the find roll is due at this time (ms).
    Waiting { roll_at_ms: u64 },
    /// Sniffing: the dig starts at this time.
    Sniffing { dig_at_ms: u64 },
    /// Digging: the chest appears at this time.
    Digging { chest_at_ms: u64 },
}

impl Hunt {
    /// The pet is sniffing or digging, so no care action may start (`ActivePet::interacting`).
    pub(super) fn working(self) -> bool {
        matches!(self, Self::Sniffing { .. } | Self::Digging { .. })
    }
}

/// The unearthed chest and what it holds, rolled when it appeared. Session only; one at a time.
#[derive(Clone, Debug)]
struct Chest {
    guid: u64,
    owner: u64,
    position: [f32; 3],
    /// Gone from this time (ms, the session clock).
    expires_ms: u64,
    coins: u32,
    item: Option<ItemGrant>,
    name_id: i32,
}

/// The service's treasure state: the generator and the live chest (never saved).
#[derive(Clone)]
pub(super) struct State {
    rng: Rng,
    chest: Option<Chest>,
}

impl State {
    pub(super) fn new() -> Self {
        // The tier and the prize come from here: OS entropy, never a constant a client could learn.
        Self {
            rng: Rng::from_entropy().unwrap_or_else(|_| Rng::seeded(0)),
            chest: None,
        }
    }
}

/// Why `open_treasure` refused. Every refusal changes nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreasureRefusal {
    /// No live chest has this guid (never found, opened, expired or left behind).
    Gone,
    /// The chest belongs to another player.
    NotYours,
    TooFar,
}

impl TreasureRefusal {
    /// The one chat line the player gets.
    pub fn message(self) -> &'static str {
        match self {
            Self::Gone => "That treasure is gone.",
            Self::NotYours => "That treasure is not yours to open.",
            Self::TooFar => "You are too far away to open the treasure.",
        }
    }
}

impl Chest {
    fn grant(&self) -> RewardGrant {
        RewardGrant {
            coins: self.coins,
            items: self.item.iter().cloned().collect(),
            job_xp: BTreeMap::new(),
            sc: 0,
            display: RewardDisplay {
                source_guid: self.guid,
                player_guid: self.owner,
                reward_kind: 0,
                icon_id: 0,
                name_id: self.name_id,
                rows: Vec::new(),
            },
        }
    }
}

impl PetService {
    /// Replace the generator with a seeded one, so a test replays any session.
    pub fn set_treasure_seed(&mut self, seed: u64) {
        self.treasure.rng = Rng::seeded(seed);
    }

    /// Whether the owner has a live chest (the world tick keeps running for it after the pet is gone).
    pub fn has_treasure(&self) -> bool {
        self.treasure.chest.is_some()
    }

    /// Whether `guid` is this owner's live chest.
    pub fn is_treasure_chest(&self, guid: u64) -> bool {
        self.treasure
            .chest
            .as_ref()
            .is_some_and(|chest| chest.guid == guid)
    }

    /// The world tick of the treasure hunt: an expired chest is removed (to the owner only), and the
    /// summoned pet works through wait, find roll, sniff, dig and chest. `owner` is the owner's
    /// position (the chest appears toward it), `wall_secs` the wall clock the cooldown is kept in,
    /// `busy` the caller's start-activity gate (the one `open_treasure`'s caller refuses with, so
    /// the hunt cannot produce a chest the owner may not open). Only the moment the chest appears
    /// marks a save (the cooldown). Costs a few comparisons and up to two linear scans of the
    /// model rows (vanity, and the ADR's `digs` when an animation plays) per settled tick;
    /// nothing is rolled or allocated unless a timer is due.
    pub fn treasure_tick(
        &mut self,
        busy: bool,
        owner: [f32; 3],
        ground: impl Fn(f32, f32) -> Option<f32>,
        now_ms: u64,
        wall_secs: u64,
    ) -> PetWire {
        let mut wire = PetWire::default();
        // The content is shared (`Arc`): holding it costs no copy on this per-tick path.
        let content = Arc::clone(&self.content);
        let Some(config) = content.treasure.as_ref() else {
            return wire;
        };
        if self
            .treasure
            .chest
            .as_ref()
            .is_some_and(|chest| now_ms >= chest.expires_ms)
        {
            wire.private.extend(self.clear_treasure());
        }
        let Some(active) = self.active else {
            return wire;
        };
        let mut hunt = Hunt::Idle;
        if self.may_hunt(config, &active, busy, now_ms, wall_secs) {
            hunt = match active.treasure {
                Hunt::Idle => Hunt::Waiting {
                    roll_at_ms: self.next_roll(config, now_ms),
                },
                Hunt::Waiting { roll_at_ms } if now_ms >= roll_at_ms => {
                    // (A treasure-hunting vitamin would raise `find_chance` here: out of scope.)
                    if self.treasure.rng.next_f64() < f64::from(config.find_chance) {
                        self.animate(&active, config.sniff, &mut wire);
                        Hunt::Sniffing {
                            dig_at_ms: now_ms.saturating_add(u64::from(config.sniff.duration_ms)),
                        }
                    } else {
                        Hunt::Waiting {
                            roll_at_ms: self.next_roll(config, now_ms),
                        }
                    }
                }
                Hunt::Sniffing { dig_at_ms } if now_ms >= dig_at_ms => {
                    self.animate(&active, config.dig, &mut wire);
                    Hunt::Digging {
                        chest_at_ms: now_ms.saturating_add(u64::from(config.dig.duration_ms)),
                    }
                }
                Hunt::Digging { chest_at_ms } if now_ms >= chest_at_ms => {
                    if let Some(spawn) =
                        self.unearth(config, &active, owner, ground, now_ms, wall_secs)
                    {
                        wire.private.push(spawn);
                        wire.changed = true;
                    }
                    Hunt::Idle
                }
                unchanged => unchanged,
            };
        }
        if let Some(active) = self.active.as_mut() {
            active.treasure = hunt;
        }
        wire
    }

    /// The hunt may run: the pet stands settled and out of water, no care action runs, it is not
    /// lying down, it is not a vanity pet, its mood is up, its cooldown is over, no chest is
    /// waiting and the owner is not busy. A clock that ran backwards hunts nothing.
    fn may_hunt(
        &self,
        config: &TreasureConfig,
        active: &ActivePet,
        busy: bool,
        now_ms: u64,
        wall_secs: u64,
    ) -> bool {
        let Some(pet) = self.pet(active.pet_id) else {
            return false;
        };
        // A stamp dated further ahead than a whole cooldown cannot be real (the clock was set
        // back), so it never freezes the pet; one dated just ahead still counts.
        let cooldown = config.cooldown_ms.div_ceil(1_000);
        let rested = pet.treasure_found_at.is_none_or(|found| {
            wall_secs >= found.saturating_add(cooldown)
                || found > wall_secs.saturating_add(cooldown)
        });
        !busy
            && self.treasure.chest.is_none()
            && active.pose.settled()
            && !active.in_water
            && !active.caring()
            && !active.idle.resting()
            && rested
            && !self.content.is_vanity(self.content.model_id(&pet.model))
            && decayed(&self.content.need_policy, pet, true, now_ms / 1_000)
                .is_some_and(|needs| needs.mood >= config.min_mood)
    }

    fn next_roll(&mut self, config: &TreasureConfig, now_ms: u64) -> u64 {
        let span = config.find_max_ms - config.find_min_ms + 1;
        let wait = config.find_min_ms + self.treasure.rng.next_u64() % span;
        now_ms.saturating_add(wait)
    }

    /// Play a hunt animation on the pet actor for the owner and the viewers, when the model's ADR
    /// declares the slot (`PetModel::digs`).
    fn animate(&self, active: &ActivePet, animation: PetAnimation, wire: &mut PetWire) {
        let digs = self
            .pet(active.pet_id)
            .and_then(|pet| self.content.model(self.content.model_id(&pet.model)))
            .is_some_and(|model| model.digs);
        if digs {
            let packet =
                piece_052::animation_packet(active.actor_guid, animation.animation_id, false);
            wire.private.push(packet.clone());
            wire.shared.push(packet);
        }
    }

    /// The chest appears: roll the tier and the prize, start the pet's cooldown, keep the chest.
    /// Returns the AddNpc for the owner alone; `None` (changing nothing) when nothing can be built.
    fn unearth(
        &mut self,
        config: &TreasureConfig,
        active: &ActivePet,
        owner: [f32; 3],
        ground: impl Fn(f32, f32) -> Option<f32>,
        now_ms: u64,
        wall_secs: u64,
    ) -> Option<Vec<u8>> {
        let tier = pick_weighted(
            config.tiers.iter().map(|tier| (tier, tier.weight)),
            &mut self.treasure.rng,
        )?;
        let item = self.roll_item(config, tier);
        let guid = CHEST_GUID_TAG | u64::from(NEXT_CHEST.fetch_add(1, Ordering::Relaxed));
        let position = chest_position(active.position, owner, config.spawn_distance, ground);
        let spawn = spawn_packet(guid, tier, config, position)?;
        pet_mut(&mut self.roster, active.pet_id)
            .ok()?
            .treasure_found_at = Some(wall_secs);
        self.treasure.chest = Some(Chest {
            guid,
            owner: active.owner_guid,
            position,
            expires_ms: now_ms.saturating_add(config.lifetime_ms),
            coins: tier.coins,
            item,
            name_id: tier.name_id,
        });
        Some(spawn)
    }

    /// The prize: the bonus item with its chance, else one item of the tier by weight.
    fn roll_item(&mut self, config: &TreasureConfig, tier: &TreasureTier) -> Option<ItemGrant> {
        let rng = &mut self.treasure.rng;
        if let Some(bonus) = &config.bonus {
            if rng.next_f64() < f64::from(bonus.chance) {
                let at = rng.next_u64() % bonus.items.len().max(1) as u64;
                let definition_id = *bonus.items.get(at as usize)?;
                return Some(ItemGrant {
                    definition_id,
                    count: 1,
                });
            }
        }
        pick_weighted(tier.items.iter().map(|row| (row, row.weight)), rng).map(|row| ItemGrant {
            definition_id: row.definition_id,
            count: row.count,
        })
    }

    /// What the owner gets for opening chest `guid` from `position`, or why not. Read-only, so a
    /// refusal changes nothing; the caller pays the grant and then calls `clear_treasure`.
    pub fn open_treasure(
        &self,
        guid: u64,
        owner: u64,
        position: [f32; 3],
        now_ms: u64,
    ) -> Result<RewardGrant, TreasureRefusal> {
        let chest = self
            .treasure
            .chest
            .as_ref()
            .filter(|chest| chest.guid == guid && now_ms < chest.expires_ms)
            .ok_or(TreasureRefusal::Gone)?;
        if chest.owner != owner {
            return Err(TreasureRefusal::NotYours);
        }
        let range = self
            .content
            .treasure
            .as_ref()
            .map_or(0.0, |config| f64::from(config.open_range));
        let reach: f64 = chest
            .position
            .iter()
            .zip(position)
            .map(|(at, player)| (f64::from(*at) - f64::from(player)).powi(2))
            .sum();
        // A non-finite position is too far.
        if !reach.is_finite() || reach > range * range {
            return Err(TreasureRefusal::TooFar);
        }
        Ok(chest.grant())
    }

    /// Remove the live chest, if any (an open that was paid, an expiry or a zone change): its 35/3,
    /// for the owner alone. A second open then finds nothing. A pet's cooldown is untouched.
    pub fn clear_treasure(&mut self) -> Vec<Vec<u8>> {
        self.treasure
            .chest
            .take()
            .map(|chest| pet_removal_packet(chest.guid))
            .into_iter()
            .collect()
    }
}

/// The chest spot: `distance` from the pet toward its owner, at most the owner's own spot, on the
/// ground there (the pet's height where the terrain has none). An owner on the pet gets +x.
fn chest_position(
    pet: [f32; 3],
    owner: [f32; 3],
    distance: f32,
    ground: impl Fn(f32, f32) -> Option<f32>,
) -> [f32; 3] {
    let (dx, dz) = (owner[0] - pet[0], owner[2] - pet[2]);
    let length = dx.hypot(dz);
    let (x, z) = if length > 1e-3 {
        let step = distance.min(length) / length;
        (pet[0] + dx * step, pet[2] + dz * step)
    } else {
        (pet[0] + distance, pet[2])
    };
    [x, ground(x, z).unwrap_or(pet[1]), z]
}

/// The chest's AddNpc (35/2, reader `00921a10`) for the owner alone: the tier's name and texture,
/// clickable, with the open range as the X-prompt radius.
fn spawn_packet(
    guid: u64,
    tier: &TreasureTier,
    config: &TreasureConfig,
    position: [f32; 3],
) -> Option<Vec<u8>> {
    let npc = startup_world::NpcSpawn {
        actor_id: guid,
        name_string_id: tier.name_id,
        model_id: config.chest_model_id,
        texture_alias: tier.texture_alias.clone(),
        position: [position[0], position[1], position[2], 1.0],
        rotation: [0.0, 0.0, 1.0, 0.0],
        is_interactable: 1,
        interact_range: config.open_range.round() as i32,
        unknown_float_51: config.open_range,
        ..world::content_spawn_template()
    };
    world::unframe_tunnel(&startup_world::build_add_npc(&npc)).ok()
}
