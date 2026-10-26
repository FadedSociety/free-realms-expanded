//! The boombox catalogue (`boomboxes.json`, T459), one row per client boombox item, and the
//! boxes pressed from the item bar (T461): a [`Boomboxes`] set of live boxes. A press puts a box
//! beside the player with a poof and its song, never uses up the item, and removes the box
//! after the row's duration. Everything here is session-only; nothing is saved.
//!
//! T536: an NPC row of `npc_spawns_extra.json` with a `boombox_item_id` is a permanent box, a
//! dance spot that never expires ([`Boomboxes::permanent`]). It is the prop's own NPC, so it
//! sends no AddNpc, poof, song or removal (R: no song; Sanctuary's `BaseZone.cs:789` starts the
//! same permanent box with song tag 0). Its dancers are handled like any other box's.
//!
//! # Recovered original facts
//!
//! * The client offers an item for the item bar (`BaseClient.Inventory.Consumables`, row
//!   predicate `00c31de0`) when its definition's type (+0x6c) is 1, its activatable ability
//!   (+0x8c) is positive, its gender usage fits (`00bae6a0`) and a unit is left. All 15 boombox
//!   definitions the server loads pass, so the client sends 42/6 for them. That data source also
//!   fills the equipment window's Consumables tab, whose row click reaches
//!   `Ui.AssignToItemActionBar(slot - 18, id)` (binding `00c23ed0`, 42/6 ctor `00c14720`); the
//!   dock's item-bar picker reads the same source (`ScriptsBase.lua`
//!   `QuickItem.PopulateSelectors`). (R) Lead decision 2026-10-25: the Stuff detail pane is not
//!   a binding path for a type-1, slot-0 definition.
//! * Every model, animation, look alias and song effect id is checked against the client's own
//!   tables and actor definitions by `tests/boombox_catalogue.rs`; each row's `source` cites
//!   where its values come from.
//! * The placed box is 35/2 AddNpc. Reader `00921a10` stores the actor's `texture_alias` and
//!   `tint_alias` in two counted strings, then an i32 `tint_id` at `this+68` (decimal, `+0x44`);
//!   the actor definition's own `TextureAliasesRead` (`007e2500`) and `TintAliasesRead` (`007e2940`)
//!   parse the model `.adr`'s alias → texture and alias → colour tables. An alias the model does
//!   not define falls back to its default skin client-side (R, T508), which is why a row sends
//!   only its own model's aliases: the rainbow model defines no texture alias and its six tints
//!   are Red, Orange, Yellow, Green, Blue and Purple; its `.dma` defaults the body material red.
//! * The song is a composite effect attached to the box by tag, 35/41 (client dispatcher
//!   jump table `0092F540` case 41: ctor `008c4d50`, reader `008f82e0` -> `008d7ad0`: guid at
//!   +0x10, tag at +0x18) and stopped by 35/42 (case 42, ctor `008b51c0`: guid +0x10, tag
//!   +0x18). The six colour songs are effect ids 16448-16453, `PFX_music_boombox_red`, `_blue`,
//!   `_green`, `_orange`, `_purple`, `_yellow`. The box's own bounce is `amb_loop_01`
//!   (`AnimationTypes.xml:844`, id 2100) and the poof is effect 21
//!   `PFX_smoke_black_explosion` (`ActorCompositeEffectDefinitions.xml:283`).
//! * A transform row's dancers take its temporary appearance: 35/14 (dispatcher case 14 at
//!   `00930283`, ctor `008b4370`, reader `008d6890` -> header `008d6830`, i32 model, u64 guid;
//!   applier `ProxiedCharacter::SwitchToTemporaryAppearance` `0096bd90`) and revert with 35/15
//!   (case 15 at `00931452`, ctor `008b4310`, reader `008d7230`: u64 guid, i32 0; applier
//!   `0096a110(0)`). The dance itself is 35/8 with the base flag (encoder
//!   `world_events::piece_052::animation_packet`). The model is also the AddPc field at +0x144
//!   (client writer `0091add1`), so a later viewer sees it.
//! * A synchronized dance step is one 35/63 SetSynchronizedAnimations: u16 35, u16 63, i32 count,
//!   then `{u64 guid, i32 animation}` per entry. Client dispatcher case 63 (`009316a0`) builds the
//!   packet (`008eafc0`) and reads it with `00919c90`: header `008d6830` (two u16), `008fc2d0` (an
//!   i32 count, then per entry `008db2a0`: u64 then i32; no client-side cap), then plays each
//!   entity found by guid (`00984960`) through `0096c780(animation, -1, 0, 0, 1, 0)`, now and not
//!   as its base. A dance ends with 35/8 animation 1 (`loc_stand`, `AnimationTypes.xml:101`)
//!   with the base flag (reader `00908a50`).
//! * A model's animation set is its base actor definition, `Resources/<model>.adr`. It maps each
//!   animation slot name to the model's own clip, e.g. `emo_dance_kpop_01` to
//!   `human_m_emo_dance_kpop_A.gr2` and `emo_dance_balletboombox_01` to
//!   `human_m_emo_dance_waltz_A.gr2`. The 16 group-3550 slots with no `<model>_emo_dance_*` entry
//!   there (freestyle 02-05, zombie 05, the Realms Roll clips, totem 02-04, gramophone) are the
//!   ones Sanctuary's measured tables omit; `tests/boombox_catalogue.rs` rebuilds every row's
//!   clip set this way. A group id also plays: the client resolves a group to one weighted random
//!   slot when it starts the animation (`0096c780` calls `007829a0`, whose weight draw is the
//!   LCG `0071acb0`). A model with no table is therefore sent the row's standing group as its base
//!   animation (R: the base-flag path in `00908a50` only stores the id, so whether the client
//!   re-resolves the group on that path is still the lead's open native check).
//! * A live transform replaces the shown actor rather than its base model: the 35/14 case
//!   (`00930283`) applies it through `0096bd90` (`ProxiedCharacter::SwitchToTemporaryAppearance`,
//!   whose own error string names it), and a late viewer gets the same model in the AddPc
//!   temporary-appearance slot (reader `0091aa10` stores it at +324, T512). So a dance table is
//!   looked up by [`crate::players::PlayerPresentation::display_model`], the model the client shows.
//! * A dancer's summoned pet dances too (T537). While a session dances and has a pet out, the pet
//!   gets 35/8 with the play flag (reader `00908a50`, dispatcher case at `009315e7`: bit 0 set
//!   stores the id as the entity's base animation at +0x51C, clear calls `0096c780` and plays it
//!   now) of the `pets` block's animation on the pet's own actor guid. Animation 3201 is slot
//!   `emo_dance_basic` (`AnimationTypes.xml:1156`) and group `emo_dance_basic`
//!   (`AnimationGroups.xml:1972`), whose one member slot carries that name; every pet species
//!   model but `vulture_pet.adr` maps the slot to a creature clip in its own `.adr` (list in
//!   `notes/Autopilot/T537.md`). The guid is `PetPresence::actor_guid`, the id the pet's AddNpc
//!   spawned, never a second scheme.
//!
//! # Reconstructed (R)
//!
//! * Dances, clip lengths, songs, priority and transforms come from Sanctuary's measured rows;
//!   duration, range, spawn offset and recast are Sanctuary's defaults. All are tunable here.
//! * An independent clip id is a client fact (its model `.adr` plays that group slot); its length
//!   is Sanctuary's measurement (R). The clip-shorter-than-blend floor of `MIN_STEP_MS` is this
//!   repo's existing rule (R; Sanctuary would let it fall to 1 ms).
//! * The box stands `spawn_offset_m` to the owner's left (Sanctuary: the player's rotation applied
//!   to -X; here `(-cos h, 0, sin h)` for the repo's facing `(sin h, 0, cos h)`), at the owner's
//!   height, facing the way the owner faces. Not interactable, no nameplate, no interact range.
//! * The Rainbow Randomizer rolls one of its six `tints` per placement and plays that colour's
//!   song (the wiki: "it appears in one randomly chosen colored tint ... they all play a different
//!   tune"; the client effect names carry the same colours). The roll is this server's choice
//!   (R): no client spawn-time pick was located. The song is attached by 35/41 with the owner as
//!   the effect's source guid, as `Placed::show` passes it; the reader (`008d7ad0`) calls that
//!   field the source, and what the client does with it for a song is unrecovered.
//! * A viewer who leaves the box's range is sent the removal (35/3) and sees the box again, with
//!   its song, on re-entry. One box per owner: a second press before the first ends replaces it
//!   (the default recast equals the duration, so this only happens if the two are tuned apart).
//! * A box outlives its owner's session until it expires.
//! * Each tick a session within `range_m` of a box on X/Z, in its zone instance, takes that box:
//!   `synchronized` and `independent` boxes need it on foot (a synchronized one also needs a model
//!   in `clip_ms`, `independent` rows roll `clips` for any model and stand a model without a
//!   table on `standing_animation_id`); a `transform` box takes it mounted too. Overlapping boxes:
//!   the higher priority wins, then the nearer (Sanctuary rotates by least served instead).
//!   `synchronized` rows (T463) share one sequence, a step lasting the longest clip of any model
//!   minus `blend_ms`, never less than `MIN_STEP_MS`, everyone listed in one 35/63 per viewer.
//!   `independent` rows (T494) roll every dancer its own `clips` entry, never the one just
//!   played, re-rolled `blend_ms` before its end; each clip is its own single-entry 35/63 for the
//!   dancer and every viewer who knows them. A player who starts dancing mid-step gets the
//!   current clip at once (Sanctuary sends only an idle and waits for the next step).
//! * A transform row's player gets 35/14 (the row's `transform_model_id`) and then 35/8 base
//!   `dance_sequence[0]` (R: Sanctuary's `StartRealmsRollDance`), sent to the player and their
//!   viewers. A mounted player is transformed but gets no dance (R); on foot again it dances.
//!   When the player's mount state changes, 35/14 is re-sent once after `transform_reapply_ms`
//!   (the brief's 2 s; Sanctuary instead re-applies when the client reports no temporary
//!   appearance).
//! * A dancer who leaves, mounts, or whose box ends or is replaced gets 35/8 idle. A transformed
//!   player who leaves, zones, or whose box ends or is replaced gets 35/15 and then 35/8 idle;
//!   when another box takes them in the same tick the idle is skipped (a transform box sends
//!   35/14 instead, a dance box the current clip). A live consumable transform (T512's
//!   `PlayerPresentation::temporary_model`) is put back with 35/14 after the 35/15, because the
//!   client has only the one temporary-appearance slot. A viewer who walks into range sees the
//!   dancer idle until the next clip. One synchronized packet carries at most `MAX_ENTRIES`.
//! * (T537) A pet's dance is the `pets` block's animation for `pet_dance_ms`, then a random gap in
//!   `pet_gap_ms`, repeated while its owner dances. The first play is immediate; Sanctuary starts a
//!   pet that joins a dance up to 3 s later (`PetDanceJoinMaxDelayMs`) and also idles it while the
//!   owner moves, the pet does not face them, or it stands in water. Those are open gaps: the tick
//!   has no pet facing or water state.

use crate::{
    abilities::effect_packets,
    consumable_effects::{encode_remove_temporary_appearance, encode_temporary_appearance},
    encounter::SharedFieldState,
    pets::PetPresence,
    players::{PlayerRegistry, SessionId, ZoneInstance},
    power_ups::Rng,
    startup_world::{self, ActorLook, NpcSpawn},
    world,
    world_events::piece_052::animation_packet,
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, Read},
    path::Path,
    sync::Arc,
};

const FILE: &str = "boomboxes.json";
const MAX_FILE: u64 = 256 * 1024;
const MAX_ROWS: usize = 64;
const MAX_MS: u32 = 600_000;
const MIN_RECAST_MS: u32 = 3_000;
/// 'BOOM' in the high word of every box guid; the low word counts boxes. The other actor tags
/// are at `mounts`, `pets`, `encounter`, `harvest_nodes` and `housing`.
const GUID_TAG: u64 = 0x424f_4f4d_0000_0000;
/// The poof at a box's spawn and removal: client effect 21 `PFX_smoke_black_explosion`.
const POOF_EFFECT: i32 = 21;
/// The box's own bounce, `amb_loop_01`.
const BOX_ANIMATION: i32 = 2100;
/// 35/63 SetSynchronizedAnimations (client case 63, `009316a0`).
const SYNC_ANIMATIONS: u16 = 63;
/// Most entries in one 35/63 (R): the client reads any count, this bounds a packet.
const MAX_ENTRIES: usize = 64;
/// Most clips one player model may have in an independent box's table (sanity bound).
const MAX_CLIPS: usize = 256;
/// Most colour tints one box may roll (sanity bound; the Rainbow row has six).
const MAX_TINTS: usize = 16;
/// Longest texture or tint alias (the mount catalogue's bound; the AddNpc codec reads a counted
/// string, so this is a data sanity limit, not a client limit).
const MAX_ALIAS: usize = 4096;
/// The animation a dancer returns to: `loc_stand`.
const IDLE_ANIMATION: i32 = 1;
/// A dance step never lasts less than this (R), whatever the blend leaves of the clip.
const MIN_STEP_MS: u32 = 1_000;

/// How the dancers around a box move: all in step, each on its own clips, or transformed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Synchronized,
    Independent,
    /// Dancers take `transform_model_id`; only this kind has one. A transform row has no dance
    /// data: its `dance_sequence` and `clip_ms` are empty.
    Transform,
}

/// Where a row's values come from. Read by people and content gates, never by the server.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub kind: String,
    pub evidence: String,
}

/// One independent-dance clip: the client animation slot id a model plays for it, and the clip's
/// length. The id is a slot of the row's standing group (client `AnimationTypes.xml`); the length
/// is Sanctuary's measurement (R).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clip {
    pub animation_id: i32,
    pub ms: u32,
}

/// One look a placed box can take: the actor's tint alias (a name the model's `.adr` defines) and
/// the song effect that colour plays, 0 for no song. The Rainbow Randomizer has one per colour;
/// every other row has one. The colour pick is also the song pick, so a box's tune always matches
/// its tint.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tint {
    pub alias: String,
    pub song_effect_id: i32,
}

/// One boombox. Ids of 0 mean "none" for `standing_animation_id` and `transform_model_id`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Boombox {
    pub item_id: i32,
    pub model_id: i32,
    pub kind: Kind,
    /// The dance ids, one per step (R: Sanctuary's rows). A `Synchronized` row's sequence is its
    /// stepped dance; a `Transform` row carries its one base dance (`dance_sequence[0]`); an
    /// `Independent` row has no sequence (its ids are `clips`). The loader refuses an empty
    /// sequence on the first two kinds and one on `Independent`.
    #[serde(default)]
    pub dance_sequence: Vec<i32>,
    /// Player model id -> one clip length per dance step. Empty unless `kind` is `Synchronized`,
    /// the only kind whose dance uses it.
    pub clip_ms: BTreeMap<i32, Vec<u32>>,
    /// Player model id -> the clips that model may be rolled, in any order (independent rows only;
    /// empty otherwise). A model missing from the map stands on `standing_animation_id`.
    #[serde(default)]
    pub clips: BTreeMap<i32, Vec<Clip>>,
    pub blend_ms: u32,
    /// The looks a press may roll, chosen once per placed box. At least one; more than one makes
    /// the box a randomiser, and the pick fixes both the tint alias and the song effect.
    pub tints: Vec<Tint>,
    pub standing_animation_id: i32,
    pub priority: i32,
    pub transform_model_id: i32,
    /// 0 unless `kind` is `Transform`.
    pub transform_reapply_ms: u32,
    pub duration_ms: u32,
    pub range_m: f32,
    pub spawn_offset_m: f32,
    pub recast_ms: u32,
    /// The model `.adr`'s texture alias, empty when the model defines none. An alias the model
    /// does not define is ignored by the client and falls back to its default skin (T508).
    pub texture_alias: String,
    /// The item's dye tint id (0 when it has none). It rides beside the rolled tint alias; the
    /// shipped boxes all carry no dye tint, so the alias alone selects the colour.
    pub tint_id: i32,
    pub source: Vec<Source>,
}

impl Boombox {
    /// How long dance step `step` lasts: the longest clip of any model, less the blend (R: the
    /// blend is how far the next step overlaps), at least [`MIN_STEP_MS`].
    pub fn step_ms(&self, step: usize) -> u32 {
        let longest = self
            .clip_ms
            .values()
            .filter_map(|clips| clips.get(step))
            .max();
        match longest.map_or(0, |ms| ms.saturating_sub(self.blend_ms)) {
            0 => MIN_STEP_MS,
            ms => ms,
        }
    }

    /// Sanity bounds, not tuned values.
    fn valid(&self) -> bool {
        let ms = |v: u32| (1..=MAX_MS).contains(&v);
        let transform = self.kind == Kind::Transform;
        let synchronized = self.kind == Kind::Synchronized;
        let independent = self.kind == Kind::Independent;
        let sequence = self.dance_sequence.len();
        self.item_id > 0
            && self.model_id > 0
            && (sequence > 0) != independent
            && (!transform || sequence == 1)
            && self.dance_sequence.iter().all(|id| *id > 0)
            && (1..=MAX_TINTS).contains(&self.tints.len())
            && self.tints.iter().all(|tint| {
                tint.alias.len() <= MAX_ALIAS
                    && tint.song_effect_id >= 0
                    && self
                        .tints
                        .iter()
                        .filter(|other| other.alias == tint.alias)
                        .count()
                        == 1
            })
            && self.texture_alias.len() <= MAX_ALIAS
            && self.tint_id >= 0
            && self.standing_animation_id >= 0
            && synchronized == !self.clip_ms.is_empty()
            && self.clip_ms.iter().all(|(model, clips)| {
                *model > 0 && clips.len() == sequence && clips.iter().all(|v| ms(*v))
            })
            && independent == !self.clips.is_empty()
            && self.clips.iter().all(|(model, clips)| {
                *model > 0
                    && !clips.is_empty()
                    && clips.len() <= MAX_CLIPS
                    && clips
                        .iter()
                        .all(|clip| clip.animation_id > 0 && ms(clip.ms))
                    && clips.iter().enumerate().all(|(i, clip)| {
                        clips[..i]
                            .iter()
                            .all(|o| o.animation_id != clip.animation_id)
                    })
            })
            && (self.kind != Kind::Independent || self.standing_animation_id > 0)
            && ms(self.blend_ms)
            && self.transform_model_id >= 0
            && transform == (self.transform_model_id > 0)
            && if transform {
                ms(self.transform_reapply_ms)
            } else {
                self.transform_reapply_ms == 0
            }
            && ms(self.duration_ms)
            && (1.0..=50.0).contains(&self.range_m)
            && (0.0..=10.0).contains(&self.spawn_offset_m)
            && (MIN_RECAST_MS..=MAX_MS).contains(&self.recast_ms)
            && !self.source.is_empty()
    }
}

/// T537: the top-level `pets` block — a dancing owner's summoned pet dances too. A file without it
/// keeps the pet dance off: no packet is ever built.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PetDance {
    /// The animation a pet plays: 35/8 with the base flag clear, so it plays now
    /// (`emo_dance_basic`, 3201).
    pub pet_dance_animation_id: i32,
    /// One play's length, before the gap that follows it (R: Sanctuary's 4000 ms dance).
    pub pet_dance_ms: u32,
    /// The random gap after a play (R: Sanctuary's 500-2000 ms).
    pub pet_gap_ms: PetGap,
    pub source: Vec<Source>,
}

/// The random gap a pet waits after a dance, drawn uniformly from `min_ms..=max_ms`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PetGap {
    pub min_ms: u32,
    pub max_ms: u32,
}

impl PetDance {
    /// The wait after one play. The loader keeps `min_ms <= max_ms`; saturating arithmetic keeps a
    /// caller-built block from panicking the tick.
    fn gap_ms(&self, rng: &mut Rng) -> u32 {
        let span = u64::from(
            self.pet_gap_ms
                .max_ms
                .saturating_sub(self.pet_gap_ms.min_ms),
        ) + 1;
        self.pet_gap_ms
            .min_ms
            .saturating_add((rng.next_u64() % span) as u32)
    }

    /// Sanity bounds, not tuned values.
    fn valid(&self) -> bool {
        let ms = |value: u32| (1..=MAX_MS).contains(&value);
        self.pet_dance_animation_id > 0
            && ms(self.pet_dance_ms)
            && ms(self.pet_gap_ms.max_ms)
            && self.pet_gap_ms.min_ms <= self.pet_gap_ms.max_ms
            && !self.source.is_empty()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogueFile {
    schema_version: u32,
    boomboxes: Vec<Boombox>,
    /// T537: absent keeps the pet dance off (serde leaves a missing `Option` as `None`).
    pets: Option<PetDance>,
}

/// Every boombox row, read-only and shared. Nothing about it is saved. A press hands out the stored
/// row, so a live box points at the catalogue's clip table instead of copying it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Catalogue {
    rows: Vec<Arc<Boombox>>,
    /// T537: the top-level `pets` block, shared for the same reason as the rows.
    pets: Option<Arc<PetDance>>,
}

impl Catalogue {
    /// `boomboxes.json` from the content directory; a missing file is an empty catalogue.
    pub fn load(directory: &Path) -> io::Result<Self> {
        let file = match fs::File::open(directory.join(FILE)) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(error),
        };
        let mut bytes = Vec::new();
        file.take(MAX_FILE + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_FILE {
            return Err(invalid("boomboxes.json exceeds 256 KiB"));
        }
        let parsed: CatalogueFile = serde_json::from_slice(&bytes).map_err(invalid)?;
        if parsed.schema_version != 1 || parsed.boomboxes.len() > MAX_ROWS {
            return Err(invalid("unsupported boomboxes.json"));
        }
        let rows = parsed.boomboxes;
        for (index, row) in rows.iter().enumerate() {
            if !row.valid() || rows[..index].iter().any(|r| r.item_id == row.item_id) {
                return Err(invalid(format!("invalid boombox row {}", row.item_id)));
            }
        }
        if parsed.pets.as_ref().is_some_and(|pets| !pets.valid()) {
            return Err(invalid("invalid boomboxes.json pets block"));
        }
        Ok(Self {
            rows: rows.into_iter().map(Arc::new).collect(),
            pets: parsed.pets.map(Arc::new),
        })
    }

    /// The row for `item_id`, shared: callers clone the pointer, never the row.
    pub fn get(&self, item_id: i32) -> Option<&Arc<Boombox>> {
        self.rows.iter().find(|row| row.item_id == item_id)
    }

    /// T537: the `pets` block, shared; `None` when the file has none, which keeps the dance off.
    pub fn pet_dance(&self) -> Option<&Arc<PetDance>> {
        self.pets.as_ref()
    }
}

fn invalid(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

/// 35/63: every `(guid, animation)` plays at once on whoever the client finds. The caller keeps
/// a packet to [`MAX_ENTRIES`].
pub fn synchronized_animations(entries: &[(u64, i32)]) -> Vec<u8> {
    let mut bytes = 35_u16.to_le_bytes().to_vec();
    bytes.extend(SYNC_ANIMATIONS.to_le_bytes());
    bytes.extend((entries.len() as i32).to_le_bytes());
    for (guid, animation) in entries {
        bytes.extend(guid.to_le_bytes());
        bytes.extend(animation.to_le_bytes());
    }
    bytes
}

/// Who is told about a dancer: the dancer and every session near them that knows the character
/// (the rule of `ride_fanout`). Nobody if the dancer left the registry or the zone: their client
/// dropped the scene, and their viewers were told they were gone.
fn audience(
    players: &PlayerRegistry,
    dancer: SessionId,
    guid: u64,
    zone: ZoneInstance,
) -> Vec<SessionId> {
    let Some(player) = players.get(dancer).filter(|p| p.zone == zone) else {
        return Vec::new();
    };
    let viewers = players.nearby_sessions(zone, player.position);
    let knowing = viewers
        .into_iter()
        .filter(|v| *v != dancer && players.knows_character(*v, guid));
    std::iter::once(dancer).chain(knowing).collect()
}

/// A dancer's session and character guid.
type Dancers = BTreeMap<SessionId, u64>;

/// A session's winning box this tick: box index, priority, distance, character guid, shown model.
type Winner = (usize, i32, f32, u64, i32);

/// A participant a box let go of: session, character guid, the box's zone and the temporary
/// model it had (0 for a dance box).
#[derive(Clone, Debug)]
struct Released {
    session: SessionId,
    guid: u64,
    zone: ZoneInstance,
    model: i32,
}

/// Drop every dancer of `dancing` this box no longer owns (gone, mounted, out of range or taken by
/// a higher-priority box), for the next idle fanout. `guid` is the character the dancer was shown
/// as; the two dance kinds only differ in what they keep per dancer.
fn release_unowned<T>(
    dancing: &mut BTreeMap<SessionId, T>,
    owned: impl Fn(&SessionId) -> bool,
    guid: impl Fn(&T) -> u64,
    zone: ZoneInstance,
    released: &mut Vec<Released>,
) {
    let gone: Vec<SessionId> = dancing
        .keys()
        .filter(|session| !owned(session))
        .copied()
        .collect();
    for session in gone {
        if let Some(dancer) = dancing.remove(&session) {
            released.push(Released {
                session,
                guid: guid(&dancer),
                zone,
                model: 0,
            });
        }
    }
}

/// A synchronized box's dance: its steps, who may dance and the clock all dancers share.
#[derive(Clone, Debug)]
struct Synchronized {
    sequence: Vec<i32>,
    /// Each step's length, from [`Boombox::step_ms`].
    step_ms: Vec<u32>,
    /// Player models with clips (the keys of `clip_ms`).
    models: BTreeSet<i32>,
    /// The current step and when it ends; `None` until the first tick.
    clock: Option<(usize, u64)>,
    /// Who has been sent the current step.
    dancing: Dancers,
}

impl Synchronized {
    fn new(row: &Boombox) -> Self {
        let steps = row.dance_sequence.len();
        Self {
            sequence: row.dance_sequence.clone(),
            step_ms: (0..steps).map(|step| row.step_ms(step)).collect(),
            models: row.clip_ms.keys().copied().collect(),
            clock: None,
            dancing: Dancers::new(),
        }
    }

    /// Bring the clock to `now_ms`: the step playing and whether it began this tick.
    fn step(&mut self, now_ms: u64) -> (usize, bool) {
        let (step, began) = match self.clock {
            None => (0, true),
            Some((step, ends)) if now_ms >= ends => ((step + 1) % self.sequence.len(), true),
            Some((step, _)) => (step, false),
        };
        if began {
            let ends = now_ms.saturating_add(u64::from(self.step_ms[step]));
            self.clock = Some((step, ends));
        }
        (step, began)
    }
}

/// One independent dancer: the shown model it was rolled for, the clip playing (`None` while it
/// stands on the row's standing group) and when that clip is re-rolled.
#[derive(Clone, Debug)]
struct Solo {
    guid: u64,
    model: i32,
    clip: Option<i32>,
    ends_ms: u64,
}

/// An independent box's dance: every dancer rolls its own clips on its own clock.
#[derive(Clone, Debug)]
struct Independent {
    /// The catalogue row the box was pressed from, shared, not copied: a live box is cloned with
    /// the world on every application message and carries a whole clip table per player model.
    row: Arc<Boombox>,
    dancing: BTreeMap<SessionId, Solo>,
}

impl Independent {
    fn new(row: Arc<Boombox>) -> Self {
        Self {
            row,
            dancing: BTreeMap::new(),
        }
    }
}

/// A row's dance: one sequence for everyone, or every dancer on its own clips.
#[derive(Clone, Debug)]
enum Dance {
    Synchronized(Synchronized),
    Independent(Independent),
}

impl Dance {
    /// The dance of a row, sharing the catalogue row rather than copying its tables.
    fn new(row: Arc<Boombox>) -> Option<Self> {
        match row.kind {
            Kind::Synchronized => Some(Self::Synchronized(Synchronized::new(&row))),
            Kind::Independent => Some(Self::Independent(Independent::new(row))),
            Kind::Transform => None,
        }
    }

    /// Whether the shown `model` may dance: a synchronized step needs clips for it, an
    /// independent dancer without a table stands on the row's standing group.
    fn allows(&self, model: i32) -> bool {
        match self {
            Self::Synchronized(dance) => dance.models.contains(&model),
            Self::Independent(_) => true,
        }
    }

    /// Give up every dancer (the box ends or is replaced): their session and character guid.
    fn leave(&mut self) -> Vec<(SessionId, u64)> {
        match self {
            Self::Synchronized(dance) => std::mem::take(&mut dance.dancing).into_iter().collect(),
            Self::Independent(dance) => std::mem::take(&mut dance.dancing)
                .into_iter()
                .map(|(session, solo)| (session, solo.guid))
                .collect(),
        }
    }
}

/// One player a transform box has changed: its guid, the mount state it was applied in and the
/// dance it was sent.
#[derive(Clone, Debug)]
struct Transformed {
    guid: u64,
    mounted: bool,
    /// The client is playing the row's dance animation (35/8 base).
    dancing: bool,
    /// When to send 35/14 again after a mount change, if any.
    reapply_at: Option<u64>,
}

/// A transform box's live spell: one model for everyone in range and who has it.
#[derive(Clone, Debug)]
struct Transform {
    model: i32,
    reapply_ms: u32,
    /// The row's one dance, `dance_sequence[0]` (R: Sanctuary's `StartRealmsRollDance`).
    dance: i32,
    transformed: BTreeMap<SessionId, Transformed>,
}

impl Transform {
    fn new(row: &Boombox) -> Self {
        Self {
            model: row.transform_model_id,
            reapply_ms: row.transform_reapply_ms,
            // The catalogue requires exactly one; a bad caller gets idle, not a tick panic.
            dance: row
                .dance_sequence
                .first()
                .copied()
                .unwrap_or(IDLE_ANIMATION),
            transformed: BTreeMap::new(),
        }
    }
}

/// How a box stands in the world, pressed or permanent: bouncing, no nameplate, not interactable.
pub fn stand_as_box(spawn: &mut NpcSpawn) {
    spawn.animation_id = BOX_ANIMATION;
    spawn.hide_nameplate = 1;
    spawn.is_interactable = 0;
    spawn.interact_range = 0;
}

/// One of `items` at random, or `None` for an empty list.
fn pick<'a, T>(items: &'a [T], rng: &mut Rng) -> Option<&'a T> {
    if items.is_empty() {
        return None;
    }
    items.get((rng.next_u64() % items.len() as u64) as usize)
}

/// A clip from `clips`, never `previous` unless it is the only one. No allocation: the two passes
/// count and then walk to the chosen index. `None` only for an empty table (the loader refuses
/// one).
fn choose(clips: &[Clip], previous: i32, rng: &mut Rng) -> Option<Clip> {
    let candidates = clips
        .iter()
        .filter(|clip| clip.animation_id != previous)
        .count();
    let exclude = candidates > 0;
    let candidates = if exclude { candidates } else { clips.len() };
    if candidates == 0 {
        return None;
    }
    let mut pick = (rng.next_u64() % candidates as u64) as usize;
    for clip in clips {
        if exclude && clip.animation_id == previous {
            continue;
        }
        if pick == 0 {
            return Some(*clip);
        }
        pick -= 1;
    }
    None
}

/// When a clip is re-rolled: at its end less the blend, or [`MIN_STEP_MS`] when the blend leaves
/// nothing of it (R).
fn clip_ends(now_ms: u64, clip: Clip, blend_ms: u32) -> u64 {
    let play = if clip.ms > blend_ms {
        clip.ms - blend_ms
    } else {
        MIN_STEP_MS
    };
    now_ms.saturating_add(u64::from(play))
}

/// Inner packets to queue, each for one session. The gateway wraps and sends them.
pub type Sends = Vec<(SessionId, Vec<u8>)>;

/// One box in the world: what a viewer is sent, when it ends and who has been sent it.
#[derive(Clone, Debug)]
struct Placed {
    guid: u64,
    owner: u64,
    zone: ZoneInstance,
    position: [f32; 3],
    /// The song's effect tag: unique per live box, so a tag can never name two songs.
    tag: i32,
    song: Option<i32>,
    /// The box's AddNpc, built once. Empty for a permanent box, whose prop is already an NPC.
    add: Vec<u8>,
    /// When the box ends. `None` is a permanent box (T536): it never ends, and it is never shown,
    /// taken back or replaced, because its prop is the NPC list's. This one field decides it.
    ends_ms: Option<u64>,
    seen_by: BTreeSet<SessionId>,
    /// Who this box competes for, from the row. Equal priority goes to the nearer box.
    priority: i32,
    range_m: f32,
    /// `None` for `transform` rows, which do not dance.
    dance: Option<Dance>,
    /// `None` for rows that do not transform.
    transform: Option<Transform>,
}

impl Placed {
    /// A permanent box (see `ends_ms`).
    fn permanent(&self) -> bool {
        self.ends_ms.is_none()
    }

    /// Let every participant go (the box ends or is replaced); the next tick sends each idle (and
    /// a transform's 35/15) unless another box has taken them.
    fn release(&mut self) -> Vec<Released> {
        let zone = self.zone;
        let mut released = Vec::new();
        for (session, guid) in self.dance.as_mut().map(Dance::leave).unwrap_or_default() {
            released.push(Released {
                session,
                guid,
                zone,
                model: 0,
            });
        }
        if let Some(transform) = self.transform.as_mut() {
            let model = transform.model;
            for (session, changed) in std::mem::take(&mut transform.transformed) {
                released.push(Released {
                    session,
                    guid: changed.guid,
                    zone,
                    model,
                });
            }
        }
        released
    }

    /// The box, its poof (first sighting only) and its song, to one session.
    fn show(&mut self, session: SessionId, poof: bool, sends: &mut Sends) {
        self.seen_by.insert(session);
        sends.push((session, self.add.clone()));
        if poof {
            sends.push((
                session,
                effect_packets::encode_play_composite_effect(self.guid, POOF_EFFECT, self.position),
            ));
        }
        if let Some(song) = self.song {
            let packet =
                effect_packets::encode_add_composite_effect(self.guid, self.tag, song, self.owner);
            sends.push((session, packet));
        }
    }

    /// The song stops, the box poofs and is removed, for everyone who was shown it.
    fn end(self, sends: &mut Sends) {
        for session in self.seen_by {
            if self.song.is_some() {
                let stop = effect_packets::encode_remove_composite_effect(self.guid, self.tag);
                sends.push((session, stop));
            }
            sends.push((
                session,
                effect_packets::encode_play_composite_effect(self.guid, POOF_EFFECT, self.position),
            ));
            sends.push((session, SharedFieldState::removal(self.guid)));
        }
    }
}

/// The live boxes, shared by every session. Cloned with the world it lives in, so keep it small:
/// one box per owner, nothing when nobody has pressed one.
#[derive(Clone, Debug)]
pub struct Boomboxes {
    live: Vec<Placed>,
    next: u32,
    /// Participants of boxes that ended or were replaced, sent idle (and 35/15) by the next tick.
    released: Vec<Released>,
    /// Draws each box's colour (and with it the song) and every independent clip. One stream for
    /// the world, like Sanctuary's `Random.Shared`; the seed comes from the OS (0 if that fails).
    rng: Rng,
    /// The permanent boxes as they were built, so a restart after a fault can start them afresh.
    fixed: Vec<Placed>,
    /// T537: the catalogue's `pets` block, shared (the world is cloned per message). `None` — the
    /// block absent from `boomboxes.json` — keeps the pet dance off.
    pet_dance: Option<Arc<PetDance>>,
    /// T537: each dancing session's summoned pet, with the time its next dance is due.
    pet_dancers: BTreeMap<SessionId, u64>,
}

impl Default for Boomboxes {
    fn default() -> Self {
        Self {
            live: Vec::new(),
            next: 0,
            released: Vec::new(),
            rng: Rng::from_entropy().unwrap_or_else(|_| Rng::seeded(0)),
            fixed: Vec::new(),
            pet_dance: None,
            pet_dancers: BTreeMap::new(),
        }
    }
}

impl Boomboxes {
    /// A world whose `props` are permanent boxes of their catalogue rows (T536). A prop naming an
    /// item the catalogue lacks is an error naming its actor: the load fails, nothing is skipped
    /// (R: Sanctuary silently skips the id).
    pub fn permanent(catalogue: &Catalogue, props: &[world::BoomboxProp]) -> io::Result<Self> {
        let mut boxes = Self::default();
        for prop in props {
            let row = catalogue.get(prop.item_id).ok_or_else(|| {
                invalid(format!(
                    "NPC spawn {} names boombox item {}, which boomboxes.json does not define",
                    prop.spawn_guid, prop.item_id
                ))
            })?;
            boxes.fixed.push(Placed {
                guid: prop.spawn_guid,
                owner: 0,
                // (R) The extras are drawn in the public open world (as `world_events`).
                zone: ZoneInstance {
                    zone_id: world::OPEN_WORLD_ZONE,
                    instance_id: 0,
                },
                position: prop.position,
                tag: 0,
                song: None,
                add: Vec::new(),
                ends_ms: None,
                seen_by: BTreeSet::new(),
                priority: row.priority,
                range_m: row.range_m,
                transform: (row.kind == Kind::Transform).then(|| Transform::new(row)),
                dance: Dance::new(Arc::clone(row)),
            });
        }
        boxes.live = boxes.fixed.clone();
        boxes.pet_dance = catalogue.pet_dance().cloned();
        Ok(boxes)
    }

    /// Only the permanent boxes, each as freshly built: what the world runs on after a fault drops
    /// every pressed box and every dancer.
    pub fn restarted(&self) -> Self {
        Self {
            live: self.fixed.clone(),
            fixed: self.fixed.clone(),
            // The `pets` block outlives a fault; the dancers it had do not.
            pet_dance: self.pet_dance.clone(),
            ..Self::default()
        }
    }

    /// Put `owner`'s box beside them and show it to everyone in range. A box the owner already
    /// has is ended first. `at` and `heading` are the owner's pose. `row` is the catalogue's
    /// shared pointer, so the box never copies its dance table.
    pub fn spawn(
        &mut self,
        row: Arc<Boombox>,
        owner: u64,
        zone: ZoneInstance,
        at: [f32; 3],
        heading: f32,
        now_ms: u64,
        players: &PlayerRegistry,
    ) -> Sends {
        let mut sends = Sends::new();
        let reach = row.spawn_offset_m;
        let position = [
            at[0] - heading.cos() * reach,
            at[1],
            at[2] + heading.sin() * reach,
        ];
        let guid = self.next_guid();
        // One pick fixes both the colour and the song, so they cannot disagree.
        let tint = pick(&row.tints, &mut self.rng).cloned().unwrap_or_default();
        let mut npc = NpcSpawn {
            actor_id: guid,
            model_id: row.model_id,
            position: [position[0], position[1], position[2], 1.0],
            rotation: [heading.sin(), 0.0, heading.cos(), 0.0],
            ..world::content_spawn_template()
        };
        stand_as_box(&mut npc);
        ActorLook {
            texture_alias: row.texture_alias.clone(),
            tint_alias: tint.alias.clone(),
            tint_id: row.tint_id,
        }
        .apply(&mut npc);
        let Ok(add) = world::unframe_tunnel(&startup_world::build_add_npc(&npc)) else {
            return sends;
        };
        if let Some(old) = self
            .live
            .iter()
            .position(|b| !b.permanent() && b.owner == owner)
        {
            let mut old = self.live.remove(old);
            self.released.extend(old.release());
            old.end(&mut sends);
        }
        let mut placed = Placed {
            guid,
            owner,
            zone,
            position,
            tag: guid as u32 as i32,
            song: (tint.song_effect_id > 0).then_some(tint.song_effect_id),
            add,
            ends_ms: Some(now_ms.saturating_add(u64::from(row.duration_ms))),
            seen_by: BTreeSet::new(),
            priority: row.priority,
            range_m: row.range_m,
            transform: (row.kind == Kind::Transform).then(|| Transform::new(&row)),
            dance: Dance::new(row),
        };
        for session in players.nearby_sessions(zone, position) {
            placed.show(session, true, &mut sends);
        }
        self.live.push(placed);
        sends
    }

    /// Once per tick: end boxes whose time is up, show the rest to sessions that came into range
    /// and take it back from those that left, then take one box per session (transform or dance)
    /// and send it. No boxes means no work. `pets` is the world's summoned pets, keyed by their
    /// owner's character guid; the T537 pet dance needs it and nothing else does.
    pub fn tick_with_pets(
        &mut self,
        now_ms: u64,
        players: &mut PlayerRegistry,
        pets: &BTreeMap<u64, PetPresence>,
    ) -> Sends {
        let mut sends = Sends::new();
        if self.live.is_empty() {
            return sends;
        }
        let (ended, live): (Vec<_>, Vec<_>) = std::mem::take(&mut self.live)
            .into_iter()
            .partition(|b| b.ends_ms.is_some_and(|ends| now_ms >= ends));
        self.live = live;
        for mut placed in ended {
            self.released.extend(placed.release());
            placed.end(&mut sends);
        }
        let mut nears = Vec::with_capacity(self.live.len());
        for placed in &mut self.live {
            let near = players.nearby_sessions(placed.zone, placed.position);
            // A permanent box is never shown or taken back: its prop is the NPC list's.
            if !placed.permanent() {
                let left: Vec<_> = placed.seen_by.difference(&near).copied().collect();
                for session in left {
                    // A session that left the zone or the server is not told: its client already
                    // dropped the box with the scene.
                    if players.get(session).is_some_and(|p| p.zone == placed.zone) {
                        sends.push((session, SharedFieldState::removal(placed.guid)));
                    }
                    placed.seen_by.remove(&session);
                }
                for session in &near {
                    if !placed.seen_by.contains(session) {
                        placed.show(*session, false, &mut sends);
                    }
                }
            }
            nears.push(near);
        }
        self.dance(now_ms, players, pets, &nears, &mut sends);
        sends
    }

    /// [`Self::tick_with_pets`] for a world with no summoned pet.
    pub fn tick(&mut self, now_ms: u64, players: &mut PlayerRegistry) -> Sends {
        self.tick_with_pets(now_ms, players, &BTreeMap::new())
    }

    /// One box per session (a transform applies, a dance box steps). `nears[i]` are the sessions
    /// near live box `i`.
    fn dance(
        &mut self,
        now_ms: u64,
        players: &mut PlayerRegistry,
        pets: &BTreeMap<u64, PetPresence>,
        nears: &[BTreeSet<SessionId>],
        sends: &mut Sends,
    ) {
        // A session belongs to one box: the highest priority, then the nearest, then the
        // earliest placed.
        let mut owner: BTreeMap<SessionId, Winner> = BTreeMap::new();
        for (index, (placed, near)) in self.live.iter().zip(nears).enumerate() {
            if placed.dance.is_none() && placed.transform.is_none() {
                continue;
            }
            for session in near {
                let Some(player) = players.get(*session) else {
                    continue;
                };
                let dx = player.position[0] - placed.position[0];
                let dz = player.position[2] - placed.position[2];
                let distance = dx.hypot(dz);
                if distance > placed.range_m {
                    continue;
                }
                let model = player.presentation.display_model();
                if let Some(dance) = &placed.dance
                    && (players.is_mounted(*session) || !dance.allows(model))
                {
                    continue;
                }
                let wins = owner
                    .get(session)
                    .is_none_or(|&(_, priority, nearest, _, _)| {
                        placed.priority > priority
                            || (placed.priority == priority && distance < nearest)
                    });
                if wins {
                    owner.insert(
                        *session,
                        (
                            index,
                            placed.priority,
                            distance,
                            player.character_guid,
                            model,
                        ),
                    );
                }
            }
        }
        let mut released = std::mem::take(&mut self.released);
        let mut steps: BTreeMap<SessionId, Vec<(u64, i32)>> = BTreeMap::new();
        let rng = &mut self.rng;
        for (index, placed) in self.live.iter_mut().enumerate() {
            let zone = placed.zone;
            // Who this box owns this tick, straight from the shared winner map.
            let owned = |session: &SessionId| owner.get(session).is_some_and(|o| o.0 == index);
            if let Some(dance) = &mut placed.dance {
                match dance {
                    Dance::Synchronized(dance) => {
                        let (step, began) = dance.step(now_ms);
                        release_unowned(
                            &mut dance.dancing,
                            &owned,
                            |guid: &u64| *guid,
                            zone,
                            &mut released,
                        );
                        for (session, o) in owner.iter().filter(|(_, o)| o.0 == index) {
                            let guid = o.3;
                            // Everyone when the step begins; otherwise only who is new here.
                            if began || !dance.dancing.contains_key(session) {
                                for viewer in audience(players, *session, guid, zone) {
                                    steps
                                        .entry(viewer)
                                        .or_default()
                                        .push((guid, dance.sequence[step]));
                                }
                            }
                            dance.dancing.insert(*session, guid);
                        }
                    }
                    Dance::Independent(dance) => {
                        release_unowned(
                            &mut dance.dancing,
                            &owned,
                            |solo: &Solo| solo.guid,
                            zone,
                            &mut released,
                        );
                        for (session, o) in owner.iter().filter(|(_, o)| o.0 == index) {
                            let (guid, model) = (o.3, o.4);
                            let previous = dance.dancing.get(session).filter(|solo| {
                                solo.guid == guid && solo.model == model && now_ms < solo.ends_ms
                            });
                            // A clip still playing is kept as it is; no packet is sent for it.
                            if previous.and_then(|solo| solo.clip).is_some() {
                                continue;
                            }
                            let past = dance.dancing.get(session).and_then(|solo| solo.clip);
                            let mut solo = Solo {
                                guid,
                                model,
                                clip: None,
                                ends_ms: 0,
                            };
                            match dance.row.clips.get(&model) {
                                Some(clips) => {
                                    if let Some(clip) = choose(clips, past.unwrap_or(0), rng) {
                                        solo.clip = Some(clip.animation_id);
                                        solo.ends_ms = clip_ends(now_ms, clip, dance.row.blend_ms);
                                        // One single-entry 35/63 per viewer: an independent clip
                                        // is not a group step, so it never shares a packet.
                                        for viewer in audience(players, *session, guid, zone) {
                                            sends.push((
                                                viewer,
                                                synchronized_animations(&[(
                                                    guid,
                                                    clip.animation_id,
                                                )]),
                                            ));
                                        }
                                    }
                                }
                                // Standing: send the row's standing group as base 35/8 once.
                                None => {
                                    if past.is_some() || !dance.dancing.contains_key(session) {
                                        for viewer in audience(players, *session, guid, zone) {
                                            let packet = animation_packet(
                                                guid,
                                                dance.row.standing_animation_id,
                                                true,
                                            );
                                            sends.push((viewer, packet));
                                        }
                                    }
                                }
                            }
                            dance.dancing.insert(*session, solo);
                        }
                    }
                }
            } else if let Some(transform) = &mut placed.transform {
                let (model, reapply_ms, dance) =
                    (transform.model, transform.reapply_ms, transform.dance);
                let gone: Vec<SessionId> = transform
                    .transformed
                    .keys()
                    .filter(|session| !owned(session))
                    .copied()
                    .collect();
                for session in gone {
                    if let Some(changed) = transform.transformed.remove(&session) {
                        released.push(Released {
                            session,
                            guid: changed.guid,
                            zone,
                            model,
                        });
                    }
                }
                for (session, o) in owner.iter().filter(|(_, o)| o.0 == index) {
                    let guid = o.3;
                    let mounted = players.is_mounted(*session);
                    if let Some(changed) = transform.transformed.get_mut(session) {
                        if changed.mounted != mounted {
                            changed.mounted = mounted;
                            if changed.dancing {
                                changed.dancing = false;
                                for viewer in audience(players, *session, guid, zone) {
                                    sends.push((
                                        viewer,
                                        animation_packet(guid, IDLE_ANIMATION, true),
                                    ));
                                }
                            }
                            changed.reapply_at = Some(now_ms.saturating_add(u64::from(reapply_ms)));
                            continue;
                        }
                        if !changed.reapply_at.is_some_and(|due| now_ms >= due) {
                            continue;
                        }
                        changed.reapply_at = None;
                        players.set_temporary_model(*session, model);
                        let viewers = audience(players, *session, guid, zone);
                        for viewer in &viewers {
                            sends.push((*viewer, encode_temporary_appearance(model, guid)));
                        }
                        if !mounted {
                            changed.dancing = true;
                            for viewer in &viewers {
                                sends.push((*viewer, animation_packet(guid, dance, true)));
                            }
                        }
                    } else {
                        players.set_temporary_model(*session, model);
                        let viewers = audience(players, *session, guid, zone);
                        for viewer in &viewers {
                            sends.push((*viewer, encode_temporary_appearance(model, guid)));
                        }
                        if !mounted {
                            for viewer in &viewers {
                                sends.push((*viewer, animation_packet(guid, dance, true)));
                            }
                        }
                        transform.transformed.insert(
                            *session,
                            Transformed {
                                guid,
                                mounted,
                                dancing: !mounted,
                                reapply_at: None,
                            },
                        );
                    }
                }
            }
        }
        // Releases before the new steps: a transform-to-dance hand-over must revert the
        // temporary appearance before the new box's step starts on the base model. A player
        // another box took needs no idle: that box's packets already say what to play.
        for released in released {
            let next = owner.get(&released.session).map(|o| o.0);
            let next_model = next
                .and_then(|index| self.live[index].transform.as_ref())
                .map_or(0, |transform| transform.model);
            if released.model != 0 {
                players.set_temporary_model(released.session, next_model);
            }
            if next.is_some() && (released.model == 0 || next_model != 0) {
                continue;
            }
            let viewers = audience(players, released.session, released.guid, released.zone);
            if released.model != 0 && next_model == 0 {
                for viewer in &viewers {
                    sends.push((*viewer, encode_remove_temporary_appearance(released.guid)));
                }
                // The 35/15 dropped the appearance; a consumable transform (T512) the boombox
                // overrode is still the player's live look, so put it back before the idle.
                let restore = players
                    .get(released.session)
                    .map_or(0, |player| player.presentation.temporary_model);
                if restore > 0 {
                    for viewer in &viewers {
                        sends.push((*viewer, encode_temporary_appearance(restore, released.guid)));
                    }
                }
            }
            if next.is_none() {
                for viewer in viewers {
                    sends.push((
                        viewer,
                        animation_packet(released.guid, IDLE_ANIMATION, true),
                    ));
                }
            }
        }
        for (viewer, entries) in steps {
            for chunk in entries.chunks(MAX_ENTRIES) {
                sends.push((viewer, synchronized_animations(chunk)));
            }
        }
        self.pet_dances(now_ms, players, pets, &owner, sends);
    }

    /// T537: while a session dances, its summoned pet dances beside it. The `pets` block fixes the
    /// animation, the play's length and the gap after it; without the block nothing is sent. The
    /// packet names the pet's own actor guid and goes to the owner and everyone near them who
    /// knows the owner, the same audience as the owner's own dance. `owner` is this tick's winner
    /// map, so this costs no allocation at all until a dancer has a pet out.
    fn pet_dances(
        &mut self,
        now_ms: u64,
        players: &PlayerRegistry,
        pets: &BTreeMap<u64, PetPresence>,
        owner: &BTreeMap<SessionId, Winner>,
        sends: &mut Sends,
    ) {
        let Some(block) = self.pet_dance.as_ref() else {
            return;
        };
        let (animation, dance_ms) = (block.pet_dance_animation_id, block.pet_dance_ms);
        // A dance box's winners all dance; a transform box's mounted rider stands (R).
        let dancing = |session: &SessionId| {
            owner.get(session).is_some_and(|winner| {
                self.live[winner.0].dance.is_some() || !players.is_mounted(*session)
            })
        };
        let mut clocks = std::mem::take(&mut self.pet_dancers);
        clocks.retain(|session, _| dancing(session));
        for session in owner.keys().filter(|session| dancing(session)) {
            // A dancer that left the registry, or whose pet was dismissed, is dropped without a
            // word: the client already removed that actor.
            let Some(player) = players.get(*session) else {
                clocks.remove(session);
                continue;
            };
            let Some(pet) = pets.get(&player.character_guid) else {
                clocks.remove(session);
                continue;
            };
            let next = clocks.entry(*session).or_insert(now_ms);
            if now_ms < *next {
                continue;
            }
            let gap = u64::from(block.gap_ms(&mut self.rng));
            *next = now_ms
                .saturating_add(u64::from(dance_ms))
                .saturating_add(gap);
            let packet = animation_packet(pet.actor_guid, animation, false);
            for viewer in audience(players, *session, player.character_guid, player.zone) {
                sends.push((viewer, packet.clone()));
            }
        }
        self.pet_dancers = clocks;
    }

    /// A guid no live box has, counting up through the low word and wrapping.
    fn next_guid(&mut self) -> u64 {
        loop {
            self.next = self.next.wrapping_add(1);
            let guid = GUID_TAG | u64::from(self.next);
            if !self.live.iter().any(|b| b.guid == guid) {
                return guid;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        mounts::MountTail,
        players::{InterestPolicy, PlayerPresentation},
    };

    const RAINBOW: i32 = 1913; // the one shipped row with songs
    const TOTEM: i32 = 8185; // no songs
    const ZONE: ZoneInstance = ZoneInstance {
        zone_id: 0,
        instance_id: 0,
    };

    fn row(item: i32) -> Boombox {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let catalogue = Catalogue::load(&directory).unwrap();
        (**catalogue.get(item).unwrap()).clone()
    }

    /// Sessions 1, 2 near the origin and 3 far away, all in `ZONE`.
    fn registry() -> PlayerRegistry {
        let mut players = PlayerRegistry::new(InterestPolicy::new(100.0, 16).unwrap());
        for (id, x) in [(1, 0.0), (2, 30.0), (3, 900.0)] {
            players
                .join(SessionId(id), id + 500, id + 600, ZONE, [x, 1.0, 0.0], 0.0)
                .unwrap();
        }
        players
    }

    fn to(sends: &Sends, session: u64) -> Vec<&[u8]> {
        let want = SessionId(session);
        let found = sends.iter().filter(|(who, _)| *who == want);
        found.map(|(_, packet)| packet.as_slice()).collect()
    }

    /// The lifecycle packets: everything but a dance step (35/8) or a synchronized step (35/63).
    fn scenery<'a>(packets: &[&'a [u8]]) -> Vec<&'a [u8]> {
        packets
            .iter()
            .copied()
            .filter(|p| !(p.starts_with(&[35, 0, 8, 0]) || p.starts_with(&[35, 0, 63, 0])))
            .collect()
    }

    /// 35/16 laid out from the community catalogue, independent of the helper under test.
    fn poof(guid: u64, at: [f32; 3]) -> Vec<u8> {
        let mut bytes = vec![35, 0, 16, 0];
        bytes.extend(guid.to_le_bytes());
        bytes.extend([0; 8]);
        bytes.extend(21_i32.to_le_bytes());
        bytes.extend([0; 8]);
        for value in [at[0], at[1], at[2], 1.0] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.push(0);
        bytes
    }

    /// Sessions `id` (character `id + 600`) as `(id, model, x, z, zone)`, all at y 1.
    fn world(cap: usize, who: &[(u64, i32, f32, f32, ZoneInstance)]) -> PlayerRegistry {
        let mut players = PlayerRegistry::new(InterestPolicy::new(100.0, cap).unwrap());
        for (id, model, x, z, zone) in who {
            join(&mut players, *id, *model, [*x, 1.0, *z], *zone);
        }
        players
    }

    fn join(
        players: &mut PlayerRegistry,
        id: u64,
        model: i32,
        at: [f32; 3],
        zone: ZoneInstance,
    ) -> Vec<crate::players::OutboundMessage> {
        let mut look = PlayerPresentation::default();
        look.character_guid = id + 600;
        look.model = model;
        let session = SessionId(id);
        let joined =
            players.join_with_presentation(session, id + 500, id + 600, zone, at, 0.0, look);
        joined.unwrap()
    }

    fn walk(players: &mut PlayerRegistry, id: u64, x: f32) {
        let zone = players.get(SessionId(id)).unwrap().zone;
        let at = [x, 1.0, 0.0];
        let moved = players.update_pose_with_state(SessionId(id), zone, at, 0.0, 0, 0);
        moved.unwrap();
    }

    /// A press at the origin: the box stands 2 m toward -x.
    fn press(live: &mut Boomboxes, row: &Boombox, owner: u64, now: u64, p: &PlayerRegistry) {
        live.spawn(
            Arc::new(row.clone()),
            owner,
            ZONE,
            [0.0, 1.0, 0.0],
            0.0,
            now,
            p,
        );
    }

    fn ride(guid: u64) -> MountTail {
        MountTail {
            ride_guid: guid,
            seat: 0,
            queue: 0,
            lift: 0.0,
        }
    }

    /// The `(guid, animation)` entries of each 35/63 a session was sent.
    fn steps(sends: &Sends, session: u64) -> Vec<Vec<(u64, i32)>> {
        let packets = to(sends, session);
        let sync = packets
            .into_iter()
            .filter(|p| p.starts_with(&[35, 0, 63, 0]));
        sync.map(|p| {
            let count = i32::from_le_bytes(p[4..8].try_into().unwrap()) as usize;
            assert_eq!(p.len(), 8 + 12 * count, "count matches the bytes");
            (0..count)
                .map(|i| {
                    let at = 8 + 12 * i;
                    let guid = u64::from_le_bytes(p[at..at + 8].try_into().unwrap());
                    let anim = i32::from_le_bytes(p[at + 8..at + 12].try_into().unwrap());
                    (guid, anim)
                })
                .collect()
        })
        .collect()
    }

    /// The characters a session was told to stand idle: 35/8, animation 1, base set.
    fn idles(sends: &Sends, session: u64) -> Vec<u64> {
        let packets = to(sends, session);
        let idle = packets
            .into_iter()
            .filter(|p| p.starts_with(&[35, 0, 8, 0]));
        idle.map(|p| {
            assert_eq!(p[12..16], 1_i32.to_le_bytes(), "loc_stand");
            assert_eq!(p[20], 1, "the base flag");
            u64::from_le_bytes(p[4..12].try_into().unwrap())
        })
        .collect()
    }

    #[test]
    fn a_press_shows_the_box_its_poof_and_song_to_those_in_range_only() {
        let rainbow = row(RAINBOW);
        let mut live = Boomboxes::default();
        let sends = live.spawn(
            Arc::new(rainbow.clone()),
            600,
            ZONE,
            [0.0, 1.0, 0.0],
            0.0,
            1_000,
            &registry(),
        );
        // Heading 0 faces +z, so the box stands 2 m toward -x at the owner's height.
        let at = [-2.0, 1.0, 0.0];
        let guid = GUID_TAG | 1;
        for session in [1, 2] {
            let packets = to(&sends, session);
            assert_eq!(packets.len(), 3, "AddNpc, poof, song for session {session}");
            let npc = startup_world::parse_add_npc(packets[0]).unwrap();
            assert_eq!(
                (npc.actor_id, npc.model_id, npc.animation_id),
                (guid, 1979, 2100)
            );
            assert_eq!(npc.position, [at[0], at[1], at[2], 1.0]);
            assert_eq!(npc.rotation, [0.0, 0.0, 1.0, 0.0]);
            assert_eq!(
                (npc.hide_nameplate, npc.is_interactable, npc.interact_range),
                (1, 0, 0)
            );
            assert_eq!(packets[1], poof(guid, at));
            let effect = i32::from_le_bytes(packets[2][16..20].try_into().unwrap());
            let tint = rainbow
                .tints
                .iter()
                .find(|tint| tint.song_effect_id == effect)
                .expect("the song belongs to one of the row's colour tints");
            // The AddNpc carries the same colour the song plays, not just the song.
            assert_eq!(
                (
                    npc.texture_alias.as_str(),
                    npc.tint_alias.as_str(),
                    npc.tint_id
                ),
                (
                    rainbow.texture_alias.as_str(),
                    tint.alias.as_str(),
                    rainbow.tint_id
                )
            );
            assert!(!tint.alias.is_empty(), "a rainbow colour has a tint alias");
            let mut song = vec![35, 0, 41, 0];
            song.extend(guid.to_le_bytes());
            song.extend((guid as u32 as i32).to_le_bytes());
            song.extend(effect.to_le_bytes());
            song.extend(600_u64.to_le_bytes()); // the owner is the source
            song.extend([0; 8]);
            assert_eq!(packets[2], song);
        }
        assert!(to(&sends, 3).is_empty(), "the far session is not shown it");
        // A row with no look keeps the model's default appearance (empty aliases, as the old
        // unresolved item aliases also rendered) and sends no 35/41.
        let totem = row(TOTEM);
        let quiet = Boomboxes::default().spawn(
            Arc::new(totem.clone()),
            600,
            ZONE,
            [0.0; 3],
            0.0,
            0,
            &registry(),
        );
        assert_eq!(to(&quiet, 1).len(), 2);
        let npc = startup_world::parse_add_npc(to(&quiet, 1)[0]).unwrap();
        assert_eq!(
            (
                npc.texture_alias.as_str(),
                npc.tint_alias.as_str(),
                npc.tint_id
            ),
            (
                totem.texture_alias.as_str(),
                totem.tints[0].alias.as_str(),
                totem.tint_id
            )
        );
    }

    #[test]
    fn expiry_stops_the_song_poofs_and_removes_once_then_nothing() {
        let mut live = Boomboxes::default();
        let mut players = registry();
        live.spawn(
            Arc::new(row(RAINBOW)),
            600,
            ZONE,
            [0.0, 1.0, 0.0],
            0.0,
            1_000,
            &players,
        );
        // Before its time is up the box only steps its dance: the near session (model 0, no
        // table) stands on the row's group as one base 35/8; the viewer that knows it hears the
        // same one packet, with no dancer of its own, and the far session hears nothing.
        let early = live.tick(1_000 + 179_999, &mut players);
        let standing = animation_packet(601, 3_550, true);
        assert_eq!(to(&early, 1), [standing.as_slice()]);
        assert_eq!(to(&early, 2), [standing.as_slice()]);
        assert!(to(&early, 3).is_empty(), "the far session is not shown it");
        assert!(early.iter().all(|(_, p)| p.starts_with(&[35, 0, 8, 0])));
        let ended = live.tick(1_000 + 180_000, &mut players);
        let guid = GUID_TAG | 1;
        for session in [1, 2] {
            let packets = scenery(&to(&ended, session));
            assert_eq!(packets.len(), 3);
            let mut stop = vec![35, 0, 42, 0];
            stop.extend(guid.to_le_bytes());
            stop.extend((guid as u32 as i32).to_le_bytes());
            assert_eq!(packets[0], stop);
            assert_eq!(packets[1], poof(guid, [-2.0, 1.0, 0.0]));
            assert_eq!(packets[2], SharedFieldState::removal(guid));
        }
        assert!(to(&ended, 3).is_empty());
        assert!(live.tick(1_000 + 999_999, &mut players).is_empty());
        assert!(live.live.is_empty());
    }

    #[test]
    fn a_session_in_range_later_is_shown_once_and_one_that_left_is_taken_back() {
        let mut live = Boomboxes::default();
        let mut players = registry();
        live.spawn(
            Arc::new(row(RAINBOW)),
            600,
            ZONE,
            [0.0, 1.0, 0.0],
            0.0,
            0,
            &players,
        );
        let guid = GUID_TAG | 1;
        let arrive = |players: &mut PlayerRegistry| {
            players
                .update_pose_with_state(SessionId(3), ZONE, [10.0, 1.0, 0.0], 0.0, 0, 0)
                .unwrap();
        };
        arrive(&mut players);
        let shown = live.tick(50, &mut players);
        let packets = scenery(&to(&shown, 3));
        assert_eq!(packets.len(), 2, "AddNpc and song, no poof");
        assert!(packets[0].starts_with(&[35, 0, 2, 0]) && packets[1].starts_with(&[35, 0, 41, 0]));
        assert!(live.tick(100, &mut players).is_empty(), "exactly once");
        // Walking out sends the removal; coming back shows the box again.
        players
            .update_pose_with_state(SessionId(3), ZONE, [900.0, 1.0, 0.0], 0.0, 0, 0)
            .unwrap();
        let gone = live.tick(150, &mut players);
        assert_eq!(
            scenery(&to(&gone, 3)),
            [SharedFieldState::removal(guid).as_slice()]
        );
        arrive(&mut players);
        assert_eq!(scenery(&to(&live.tick(200, &mut players), 3)).len(), 2);
        // A session that left the registry is dropped without a word.
        let _ = players.leave(SessionId(3));
        assert!(live.tick(250, &mut players).is_empty());
        assert!(live.live[0].seen_by.iter().all(|s| s.0 != 3));
    }

    #[test]
    fn a_second_box_from_one_owner_replaces_the_first() {
        let mut live = Boomboxes::default();
        let players = registry();
        live.spawn(
            Arc::new(row(RAINBOW)),
            600,
            ZONE,
            [0.0, 1.0, 0.0],
            0.0,
            0,
            &players,
        );
        let sends = live.spawn(
            Arc::new(row(TOTEM)),
            600,
            ZONE,
            [0.0, 1.0, 0.0],
            0.0,
            10,
            &players,
        );
        assert_eq!(live.live.len(), 1);
        let packets = to(&sends, 1);
        // The old box's stop, poof and removal, then the new box's AddNpc and poof.
        assert_eq!(packets.len(), 5);
        assert!(packets[2].starts_with(&[35, 0, 3, 0]));
        assert!(packets[3].starts_with(&[35, 0, 2, 0]));
    }

    #[test]
    fn a_guid_never_repeats_among_live_boxes_across_the_counter_wrap() {
        let mut live = Boomboxes::default();
        let players = registry();
        for owner in [1, 2] {
            live.spawn(
                Arc::new(row(TOTEM)),
                owner,
                ZONE,
                [0.0; 3],
                0.0,
                0,
                &players,
            );
        }
        // The counter is about to wrap onto the two live guids (low words 1 and 2).
        live.next = u32::MAX;
        live.spawn(Arc::new(row(TOTEM)), 3, ZONE, [0.0; 3], 0.0, 0, &players);
        live.spawn(Arc::new(row(TOTEM)), 4, ZONE, [0.0; 3], 0.0, 0, &players);
        let mut guids: Vec<u64> = live.live.iter().map(|b| b.guid).collect();
        assert_eq!(
            guids.iter().map(|g| *g as u32).collect::<Vec<_>>(),
            [1, 2, 0, 3],
            "the wrap skips the live low words"
        );
        guids.sort();
        guids.dedup();
        assert_eq!(guids.len(), 4);
    }

    #[test]
    fn a_live_box_points_at_the_pressed_catalogue_row_instead_of_copying_its_table() {
        let players = registry();
        let pressed = Arc::new(row(RAINBOW));
        let mut live = Boomboxes::default();
        live.spawn(Arc::clone(&pressed), 600, ZONE, [0.0; 3], 0.0, 0, &players);
        let Some(Dance::Independent(dance)) = &live.live[0].dance else {
            panic!("the Randomizer dances independently");
        };
        assert!(
            Arc::ptr_eq(&dance.row, &pressed),
            "the box shares the row's clip table, it does not copy it"
        );
    }

    /// T463: synchronized dances.
    mod dancing {
        use super::*;
        use crate::mounts::MountTail;

        const JINGLE: i32 = 878; // synchronized, four steps: 3531 to 3534, blend 600
        const HUMAN_M: i32 = 1;
        const HUMAN_F: i32 = 60;
        const CREATURE: i32 = 1941; // a transform model: no clips on any synchronized row
        const OTHER: ZoneInstance = ZoneInstance {
            zone_id: 0,
            instance_id: 1,
        };
        /// Step ends of Jingle Bell from its first tick at 1000: the longest clip of the four
        /// player models less the 600 ms blend, 4234, 3834, 6067 and 10400 ms.
        const STEP_ENDS: [u64; 4] = [5_234, 9_068, 15_135, 25_535];

        /// Sessions `id` (character `id + 600`) as `(id, model, x, z, zone)`, all at y 1.
        fn world(cap: usize, who: &[(u64, i32, f32, f32, ZoneInstance)]) -> PlayerRegistry {
            let mut players = PlayerRegistry::new(InterestPolicy::new(100.0, cap).unwrap());
            for (id, model, x, z, zone) in who {
                join(&mut players, *id, *model, [*x, 1.0, *z], *zone);
            }
            players
        }

        fn join(
            players: &mut PlayerRegistry,
            id: u64,
            model: i32,
            at: [f32; 3],
            zone: ZoneInstance,
        ) {
            let mut look = PlayerPresentation::default();
            look.character_guid = id + 600;
            look.model = model;
            let session = SessionId(id);
            let joined =
                players.join_with_presentation(session, id + 500, id + 600, zone, at, 0.0, look);
            joined.unwrap();
        }

        fn walk(players: &mut PlayerRegistry, id: u64, x: f32) {
            let zone = players.get(SessionId(id)).unwrap().zone;
            let at = [x, 1.0, 0.0];
            let moved = players.update_pose_with_state(SessionId(id), zone, at, 0.0, 0, 0);
            moved.unwrap();
        }

        /// A press at the origin: the box stands 2 m toward -x.
        fn press(live: &mut Boomboxes, row: &Boombox, owner: u64, now: u64, p: &PlayerRegistry) {
            live.spawn(
                Arc::new(row.clone()),
                owner,
                ZONE,
                [0.0, 1.0, 0.0],
                0.0,
                now,
                p,
            );
        }

        fn ride(guid: u64) -> MountTail {
            MountTail {
                ride_guid: guid,
                seat: 0,
                queue: 0,
                lift: 0.0,
            }
        }

        /// The `(guid, animation)` entries of each 35/63 a session was sent.
        fn steps(sends: &Sends, session: u64) -> Vec<Vec<(u64, i32)>> {
            let packets = to(sends, session);
            let sync = packets
                .into_iter()
                .filter(|p| p.starts_with(&[35, 0, 63, 0]));
            sync.map(|p| {
                let count = i32::from_le_bytes(p[4..8].try_into().unwrap()) as usize;
                assert_eq!(p.len(), 8 + 12 * count, "count matches the bytes");
                (0..count)
                    .map(|i| {
                        let at = 8 + 12 * i;
                        let guid = u64::from_le_bytes(p[at..at + 8].try_into().unwrap());
                        let anim = i32::from_le_bytes(p[at + 8..at + 12].try_into().unwrap());
                        (guid, anim)
                    })
                    .collect()
            })
            .collect()
        }

        /// The base 35/8 `(guid, animation)` pairs a session was sent.
        fn bases(sends: &Sends, session: u64) -> Vec<(u64, i32)> {
            let packets = to(sends, session);
            let base = packets
                .into_iter()
                .filter(|p| p.starts_with(&[35, 0, 8, 0]));
            base.map(|p| {
                assert_eq!(p[20], 1, "the base flag");
                let guid = u64::from_le_bytes(p[4..12].try_into().unwrap());
                let animation = i32::from_le_bytes(p[12..16].try_into().unwrap());
                (guid, animation)
            })
            .collect()
        }

        /// The characters a session was told to stand idle: 35/8, animation 1, base set.
        fn idles(sends: &Sends, session: u64) -> Vec<u64> {
            bases(sends, session)
                .into_iter()
                .filter(|(_, animation)| *animation == IDLE_ANIMATION)
                .map(|(guid, _)| guid)
                .collect()
        }

        #[test]
        fn the_packet_is_the_clients_layout() {
            let bytes = synchronized_animations(&[(0x0102_0304_0506_0708, 3531), (9, -2)]);
            let mut want = vec![35, 0, 63, 0, 2, 0, 0, 0];
            want.extend([8, 7, 6, 5, 4, 3, 2, 1, 0xcb, 0x0d, 0, 0]);
            want.extend([9, 0, 0, 0, 0, 0, 0, 0, 0xfe, 0xff, 0xff, 0xff]);
            assert_eq!(bytes, want);
            assert_eq!(synchronized_animations(&[]), [35, 0, 63, 0, 0, 0, 0, 0]);
        }

        #[test]
        fn a_step_is_the_longest_clip_less_the_blend_and_never_below_a_second() {
            let jingle = row(JINGLE);
            // Model 60's step 3 is 11000 ms, the others 10667: the longest wins.
            let lengths: Vec<_> = (0..4).map(|step| jingle.step_ms(step)).collect();
            assert_eq!(lengths, [4_234, 3_834, 6_067, 10_400]);
            let mut short = jingle.clone();
            short.blend_ms = 20_000; // more than any clip
            assert_eq!(short.step_ms(0), 1_000);
            short.blend_ms = 4_834; // exactly the clip: nothing left
            assert_eq!(short.step_ms(0), 1_000);
            assert_eq!(short.step_ms(9), 1_000, "no such step");
        }

        #[test]
        fn dancers_in_range_get_the_same_step_in_one_tick_and_it_advances_on_the_clock() {
            let mut players = world(
                16,
                &[(1, HUMAN_M, 0.0, 0.0, ZONE), (2, HUMAN_F, 5.0, 3.0, ZONE)],
            );
            let mut live = Boomboxes::default();
            press(&mut live, &row(JINGLE), 600, 1_000, &players);
            let both = |anim| vec![vec![(601, anim), (602, anim)]];
            let first = live.tick(1_000, &mut players);
            assert_eq!(steps(&first, 1), both(3531));
            assert_eq!(steps(&first, 2), both(3531));
            for (now, anim) in [(5_234, 3532), (9_068, 3533), (15_135, 3534), (25_535, 3531)] {
                assert!(STEP_ENDS.contains(&now));
                // One tick before the boundary nothing moves; at it both get the next step.
                let before = live.tick(now - 1, &mut players);
                assert!(steps(&before, 1).is_empty(), "tick {}", now - 1);
                let sends = live.tick(now, &mut players);
                assert_eq!(steps(&sends, 1), both(anim), "tick {now}");
                assert_eq!(steps(&sends, 2), both(anim), "tick {now}");
            }
        }

        #[test]
        fn only_sessions_in_range_on_foot_and_with_clips_dance() {
            let mut players = world(
                16,
                &[
                    (1, HUMAN_M, 0.0, 0.0, ZONE),   // dances
                    (2, HUMAN_M, 40.0, 0.0, ZONE),  // 42 m from the box: out of range
                    (3, HUMAN_M, 0.0, 0.0, OTHER),  // another zone instance
                    (4, HUMAN_F, 1.0, 0.0, ZONE),   // mounted below
                    (5, CREATURE, 2.0, 0.0, ZONE),  // no clips for this model
                    (6, 0, 3.0, 0.0, ZONE),         // no presentation model
                    (7, HUMAN_M, -2.0, 14.9, ZONE), // inside the edge, X/Z only
                    (8, HUMAN_M, -2.0, 15.1, ZONE), // just outside it
                ],
            );
            players.set_mount(SessionId(4), ride(1));
            let mut live = Boomboxes::default();
            press(&mut live, &row(JINGLE), 600, 0, &players);
            let sends = live.tick(10, &mut players);
            // The two dancers are listed, and only them, wherever they are heard.
            for session in [1, 2, 4, 5, 6, 7, 8] {
                for list in steps(&sends, session) {
                    let guids: Vec<_> = list.iter().map(|e| e.0).collect();
                    assert!(
                        guids.iter().all(|g| [601, 607].contains(g)),
                        "session {session}: {guids:?}"
                    );
                }
            }
            assert_eq!(steps(&sends, 1), [[(601, 3531), (607, 3531)]]);
            assert_eq!(steps(&sends, 7), [[(601, 3531), (607, 3531)]]);
            assert!(steps(&sends, 3).is_empty(), "another zone hears nothing");
            // Height does not count: range is on X/Z.
            let high = [-2.0, 40.0, 14.0];
            let moved = players.update_pose_with_state(SessionId(8), ZONE, high, 0.0, 0, 0);
            moved.unwrap();
            let sends = live.tick(20, &mut players);
            assert_eq!(steps(&sends, 8), [[(608, 3531)]], "a joiner, heard alone");
        }

        #[test]
        fn leaving_range_sends_one_idle_to_the_leaver_and_their_viewers() {
            let mut players = world(
                16,
                &[(1, HUMAN_M, 0.0, 0.0, ZONE), (2, HUMAN_M, 4.0, 0.0, ZONE)],
            );
            let mut live = Boomboxes::default();
            press(&mut live, &row(JINGLE), 600, 0, &players);
            live.tick(0, &mut players);
            walk(&mut players, 1, 40.0);
            let sends = live.tick(50, &mut players);
            assert_eq!(idles(&sends, 1), [601]);
            assert_eq!(idles(&sends, 2), [601], "the viewer who knows them");
            assert!(steps(&sends, 1).is_empty());
            assert!(
                live.tick(100, &mut players).is_empty(),
                "once, then nothing"
            );
        }

        #[test]
        fn mounting_stops_the_dance_and_stepping_off_rejoins_it_mid_step() {
            let mut players = world(
                16,
                &[(1, HUMAN_M, 0.0, 0.0, ZONE), (2, HUMAN_M, 4.0, 0.0, ZONE)],
            );
            let mut live = Boomboxes::default();
            press(&mut live, &row(JINGLE), 600, 1_000, &players);
            live.tick(1_000, &mut players);
            players.set_mount(SessionId(1), ride(7));
            let sends = live.tick(1_050, &mut players);
            assert_eq!((idles(&sends, 1), idles(&sends, 2)), (vec![601], vec![601]));
            assert!(live.tick(1_100, &mut players).is_empty());
            // Off the ride the dancer joins at the step now playing, as a single entry.
            players.set_mount(SessionId(1), MountTail::NONE);
            let sends = live.tick(1_150, &mut players);
            assert_eq!(steps(&sends, 1), [[(601, 3531)]]);
            assert_eq!(steps(&sends, 2), [[(601, 3531)]]);
        }

        #[test]
        fn expiry_sends_idle_to_every_dancer_and_then_nothing() {
            let mut players = world(
                16,
                &[(1, HUMAN_M, 0.0, 0.0, ZONE), (2, HUMAN_F, 4.0, 0.0, ZONE)],
            );
            let mut live = Boomboxes::default();
            press(&mut live, &row(JINGLE), 600, 0, &players);
            live.tick(0, &mut players);
            let ended = live.tick(180_000, &mut players);
            assert_eq!(idles(&ended, 1), [601, 602]);
            assert_eq!(idles(&ended, 2), [601, 602]);
            assert!(steps(&ended, 1).is_empty());
            assert!(live.tick(180_050, &mut players).is_empty());
        }

        #[test]
        fn a_replaced_box_lets_its_dancers_go() {
            let mut players = world(16, &[(1, HUMAN_M, 0.0, 0.0, ZONE)]);
            let mut live = Boomboxes::default();
            press(&mut live, &row(JINGLE), 600, 0, &players);
            live.tick(0, &mut players);
            // The owner presses again 500 m away: the first box ends under the dancer.
            let again = [500.0, 1.0, 0.0];
            live.spawn(Arc::new(row(JINGLE)), 600, ZONE, again, 0.0, 100, &players);
            let sends = live.tick(150, &mut players);
            assert_eq!(idles(&sends, 1), [601]);
            assert!(steps(&sends, 1).is_empty());
        }

        #[test]
        fn overlapping_boxes_the_higher_priority_wins_then_the_nearer() {
            let mut players = world(16, &[(1, HUMAN_M, 0.0, 0.0, ZONE)]);
            let mut loud = row(TOTEM); // 3551
            let mut quiet = row(JINGLE); // 3531, priority 0
            // The quiet box is nearer (2 m) than the loud one (4 m), but the loud one wins.
            loud.priority = 5;
            quiet.spawn_offset_m = 2.0;
            loud.spawn_offset_m = 4.0;
            let mut live = Boomboxes::default();
            press(&mut live, &quiet, 600, 0, &players);
            press(&mut live, &loud, 601, 0, &players);
            let sends = live.tick(10, &mut players);
            assert_eq!(steps(&sends, 1), [[(601, 3551)]], "listed by one box only");
            // Equal priority: the nearer wins, whichever was placed first.
            loud.priority = 0;
            let mut live = Boomboxes::default();
            press(&mut live, &loud, 601, 0, &players);
            press(&mut live, &quiet, 600, 0, &players);
            let sends = live.tick(10, &mut players);
            assert_eq!(steps(&sends, 1), [[(601, 3531)]]);
            // Walking to the loud box's side hands the dancer over with no idle between.
            walk(&mut players, 1, -10.0);
            let sends = live.tick(20, &mut players);
            assert!(idles(&sends, 1).is_empty());
            assert_eq!(steps(&sends, 1), [[(601, 3551)]]);
        }

        #[test]
        fn a_viewer_is_told_only_about_the_dancers_it_knows() {
            // Each session sees only its nearest neighbour.
            let mut players = world(
                1,
                &[
                    (1, HUMAN_M, 0.0, 0.0, ZONE),
                    (2, HUMAN_M, 5.0, 0.0, ZONE),
                    (3, 0, 6.0, 0.0, ZONE), // watches, does not dance
                ],
            );
            assert!(!players.knows_character(SessionId(3), 601));
            let mut live = Boomboxes::default();
            press(&mut live, &row(JINGLE), 600, 0, &players);
            let sends = live.tick(0, &mut players);
            assert_eq!(steps(&sends, 3), [[(602, 3531)]], "it does not know 601");
            assert_eq!(
                steps(&sends, 2),
                [[(602, 3531)]],
                "it sees only the watcher"
            );
        }

        #[test]
        fn a_crowd_is_split_into_packets_of_at_most_64() {
            let who: Vec<_> = (1..=70)
                .map(|id| (id, HUMAN_M, (id % 10) as f32, 0.0, ZONE))
                .collect();
            let mut players = world(200, &who);
            let mut live = Boomboxes::default();
            press(&mut live, &row(JINGLE), 600, 0, &players);
            let sends = live.tick(0, &mut players);
            let sizes: Vec<_> = steps(&sends, 1).iter().map(Vec::len).collect();
            assert_eq!(sizes, [64, 6]);
        }

        #[test]
        fn a_disconnect_mid_dance_tells_nobody_and_does_not_stall_the_box() {
            let mut players = world(
                16,
                &[(1, HUMAN_M, 0.0, 0.0, ZONE), (2, HUMAN_M, 4.0, 0.0, ZONE)],
            );
            let mut live = Boomboxes::default();
            press(&mut live, &row(JINGLE), 600, 1_000, &players);
            live.tick(1_000, &mut players);
            let _ = players.leave(SessionId(1));
            let sends = live.tick(1_050, &mut players);
            assert!(sends.is_empty(), "the leaver's viewers were told it left");
            // The box and its remaining dancer carry on at the next step.
            let sends = live.tick(STEP_ENDS[0], &mut players);
            assert_eq!(steps(&sends, 2), [[(602, 3532)]]);
            // A reconnect is a new session with no dance: out of range it hears nothing, and
            // walking in makes it a joiner at the step now playing.
            join(&mut players, 11, HUMAN_M, [900.0, 1.0, 0.0], ZONE);
            assert!(live.tick(STEP_ENDS[0] + 50, &mut players).is_empty());
            walk(&mut players, 11, 3.0);
            let sends = live.tick(STEP_ENDS[0] + 100, &mut players);
            assert_eq!(steps(&sends, 11), [[(611, 3532)]]);
        }

        #[test]
        fn other_kinds_and_empty_worlds_send_nothing() {
            let mut players = world(16, &[(1, HUMAN_M, 0.0, 0.0, ZONE)]);
            let mut live = Boomboxes::default();
            assert!(live.tick(0, &mut players).is_empty(), "no live box");
            // Realms Roll transforms its dancers; only a synchronized or independent row dances.
            press(&mut live, &row(1180), 1180, 0, &players);
            let sends = live.tick(10, &mut players);
            assert!(steps(&sends, 1).is_empty() && idles(&sends, 1).is_empty());
        }

        /// T494: independent dances (the Rainbow Randomizer, Realmshake and XVD-S).
        mod independent {
            use super::*;

            const REALMSHAKE: i32 = 76935;
            const TRANSFORMED: i32 = 1941; // the Realms Roll model: no table on any row

            /// A seeded world: every draw is reproducible.
            fn seeded(seed: u64) -> Boomboxes {
                let mut live = Boomboxes::default();
                live.rng = Rng::seeded(seed);
                live
            }

            fn ids(item: i32, model: i32) -> Vec<i32> {
                row(item).clips[&model]
                    .iter()
                    .map(|clip| clip.animation_id)
                    .collect()
            }

            /// Every entry a session was sent, of the single-entry 35/63 packets.
            fn rolled(sends: &Sends, session: u64) -> Vec<(u64, i32)> {
                steps(sends, session).into_iter().flatten().collect()
            }

            /// When the clip `animation` rolled at `now` re-rolls, by the row's own rule.
            fn ends(item: i32, now: u64, animation: i32) -> u64 {
                let row = row(item);
                let clip = row.clips[&HUMAN_M]
                    .iter()
                    .find(|clip| clip.animation_id == animation)
                    .unwrap();
                clip_ends(now, *clip, row.blend_ms)
            }

            #[test]
            fn every_roll_comes_from_the_model_table_and_never_repeats_the_last() {
                let mut players = world(16, &[(1, HUMAN_M, 0.0, 0.0, ZONE)]);
                let table = ids(RAINBOW, HUMAN_M);
                assert_eq!(table.len(), 73, "the Rainbow Randomizer's model-1 table");
                let mut live = seeded(7);
                // A long box so every roll fits inside its life.
                let mut rainbow = row(RAINBOW);
                rainbow.duration_ms = MAX_MS;
                press(&mut live, &rainbow, 600, 1_000, &players);
                let mut now = 1_000;
                let mut previous = None;
                for roll in 0..30 {
                    let sends = live.tick(now, &mut players);
                    let entries = rolled(&sends, 1);
                    assert_eq!(entries.len(), 1, "roll {roll} at {now}: {entries:?}");
                    let (guid, animation) = entries[0];
                    assert_eq!(guid, 601);
                    assert!(
                        table.contains(&animation),
                        "roll {roll}: {animation} is not in the model table"
                    );
                    assert_ne!(
                        Some(animation),
                        previous,
                        "roll {roll} repeated {animation}"
                    );
                    previous = Some(animation);
                    now = ends(RAINBOW, now, animation);
                }
            }

            #[test]
            fn two_dancers_can_hold_different_clips_at_the_same_moment() {
                let mut players = world(
                    16,
                    &[(1, HUMAN_M, 0.0, 0.0, ZONE), (2, HUMAN_M, 4.0, 0.0, ZONE)],
                );
                let mut live = seeded(1);
                press(&mut live, &row(RAINBOW), 600, 0, &players);
                let sends = live.tick(0, &mut players);
                // One single-entry 35/63 per dancer, not one packet with both entries.
                let packets = steps(&sends, 1);
                assert_eq!(packets.len(), 2, "one packet per dancer");
                assert!(packets.iter().all(|step| step.len() == 1), "{packets:?}");
                let entries: Vec<_> = packets.iter().flatten().copied().collect();
                assert_eq!((entries[0].0, entries[1].0), (601, 602));
                assert_ne!(entries[0].1, entries[1].1);
                let seen = steps(&sends, 2);
                assert_eq!(seen.len(), 2, "each viewer sees both");
                assert!(seen.iter().all(|step| step.len() == 1));
                assert_eq!(seen.into_iter().flatten().collect::<Vec<_>>(), entries);
            }

            #[test]
            fn a_two_clip_table_alternates_and_never_repeats() {
                let mut players = world(16, &[(1, HUMAN_M, 0.0, 0.0, ZONE)]);
                // With only two clips every roll must be the one not playing, so an "allow a
                // repeat" regression fails here; on the 73-clip table it could pass for a seed.
                let mut short = row(RAINBOW);
                short.duration_ms = MAX_MS; // every roll fits inside the box's life
                short.clips.insert(
                    HUMAN_M,
                    vec![
                        Clip {
                            animation_id: 3_201,
                            ms: 14_967,
                        },
                        Clip {
                            animation_id: 3_211,
                            ms: 8_000,
                        },
                    ],
                );
                let mut live = seeded(3);
                press(&mut live, &short, 600, 0, &players);
                let mut now = 0;
                let mut previous = None;
                for roll in 0..20 {
                    let entries = rolled(&live.tick(now, &mut players), 1);
                    assert_eq!(entries.len(), 1, "roll {roll} at {now}");
                    let animation = entries[0].1;
                    assert_ne!(
                        Some(animation),
                        previous,
                        "roll {roll} repeated {animation}"
                    );
                    previous = Some(animation);
                    now = ends(RAINBOW, now, animation);
                }
            }

            #[test]
            fn a_transformed_player_stands_by_its_temporary_model() {
                let mut players = world(16, &[(1, HUMAN_M, 0.0, 0.0, ZONE)]);
                // A live transform shows model 1941 (the Realms Roll model: no dance table) over
                // a base model that has one. The table belongs to the model the client shows.
                let mut look = players.get(SessionId(1)).unwrap().presentation.clone();
                look.set_visual(crate::consumable_effects::Visual {
                    temporary_model: TRANSFORMED,
                    scale: 1.0,
                    auras: Vec::new(),
                });
                players.set_presentation(SessionId(1), look).unwrap();
                assert_eq!(
                    players
                        .get(SessionId(1))
                        .unwrap()
                        .presentation
                        .display_model(),
                    TRANSFORMED
                );
                let mut live = seeded(5);
                press(&mut live, &row(RAINBOW), 600, 0, &players);
                let sends = live.tick(0, &mut players);
                assert!(
                    steps(&sends, 1).is_empty(),
                    "the shown model has no table: no 35/63"
                );
                assert_eq!(bases(&sends, 1), [(601, 3_550)]);
            }

            #[test]
            fn a_replaced_independent_box_lets_its_dancers_go() {
                let mut players = world(
                    16,
                    &[(1, HUMAN_M, 0.0, 0.0, ZONE), (2, HUMAN_M, 4.0, 0.0, ZONE)],
                );
                let mut live = seeded(6);
                press(&mut live, &row(RAINBOW), 600, 0, &players);
                live.tick(0, &mut players);
                // The owner presses again 500 m away: the first box ends under both dancers, who
                // get the idle and no further clip.
                let again = [500.0, 1.0, 0.0];
                live.spawn(Arc::new(row(RAINBOW)), 600, ZONE, again, 0.0, 100, &players);
                let sends = live.tick(150, &mut players);
                assert_eq!(idles(&sends, 1), [601, 602]);
                assert_eq!(idles(&sends, 2), [601, 602]);
                assert!(steps(&sends, 1).is_empty());
            }

            #[test]
            fn the_randomizer_colour_and_song_are_always_one_of_its_six() {
                let players = world(16, &[(1, HUMAN_M, 0.0, 0.0, ZONE)]);
                let rainbow = row(RAINBOW);
                assert_eq!(rainbow.tints.len(), 6);
                let mut live = seeded(11);
                let mut seen = BTreeSet::new();
                for n in 0..32 {
                    let sends = live.spawn(
                        Arc::new(rainbow.clone()),
                        600 + n,
                        ZONE,
                        [0.0, 1.0, 0.0],
                        0.0,
                        n,
                        &players,
                    );
                    let packets = to(&sends, 1);
                    let npc = startup_world::parse_add_npc(packets[0]).unwrap();
                    let song = packets
                        .into_iter()
                        .find(|p| p.starts_with(&[35, 0, 41, 0]))
                        .expect("the box has a song");
                    let effect = i32::from_le_bytes(song[16..20].try_into().unwrap());
                    let tint = rainbow
                        .tints
                        .iter()
                        .find(|tint| tint.song_effect_id == effect)
                        .unwrap_or_else(|| panic!("spawn {n}: {effect}"));
                    assert_eq!(
                        (npc.tint_alias.as_str(), npc.tint_id),
                        (tint.alias.as_str(), rainbow.tint_id),
                        "spawn {n}: the colour matches the song"
                    );
                    seen.insert((tint.alias.clone(), effect));
                }
                assert!(seen.len() > 1, "the pick varies: {seen:?}");
            }

            #[test]
            fn a_model_without_a_table_stands_on_the_row_group_as_a_base_animation() {
                let mut players = world(16, &[(1, TRANSFORMED, 0.0, 0.0, ZONE)]);
                let mut live = seeded(5);
                press(&mut live, &row(RAINBOW), 600, 0, &players);
                let sends = live.tick(0, &mut players);
                assert!(steps(&sends, 1).is_empty(), "no 35/63");
                assert_eq!(bases(&sends, 1), [(601, 3_550)]);
                // The base plays until the box ends: it is sent once, not every tick.
                let again = live.tick(5_000, &mut players);
                assert!(bases(&again, 1).is_empty() && steps(&again, 1).is_empty());
            }

            #[test]
            fn a_one_clip_table_repeats_it_instead_of_rerolling_forever() {
                let mut players = world(16, &[(1, HUMAN_M, 0.0, 0.0, ZONE)]);
                let mut single = row(RAINBOW);
                single.clips.insert(
                    HUMAN_M,
                    vec![Clip {
                        animation_id: 3_201,
                        ms: 5_000,
                    }],
                );
                let mut live = seeded(9);
                press(&mut live, &single, 600, 0, &players);
                assert_eq!(rolled(&live.tick(0, &mut players), 1), [(601, 3_201)]);
                assert!(live.tick(4_399, &mut players).is_empty(), "still playing");
                assert_eq!(rolled(&live.tick(4_400, &mut players), 1), [(601, 3_201)]);
            }

            #[test]
            fn a_clip_shorter_than_the_blend_waits_a_second() {
                let mut players = world(16, &[(1, HUMAN_M, 0.0, 0.0, ZONE)]);
                let mut short = row(RAINBOW);
                short.clips.insert(
                    HUMAN_M,
                    vec![
                        Clip {
                            animation_id: 3_201,
                            ms: 500,
                        },
                        Clip {
                            animation_id: 3_211,
                            ms: 500,
                        },
                    ],
                );
                let mut live = seeded(4);
                press(&mut live, &short, 600, 0, &players);
                let first = rolled(&live.tick(0, &mut players), 1);
                assert_eq!(first.len(), 1);
                assert!(live.tick(999, &mut players).is_empty(), "the 1000 ms floor");
                let second = rolled(&live.tick(1_000, &mut players), 1);
                assert_eq!(second.len(), 1);
                assert_ne!(first[0].1, second[0].1);
            }

            #[test]
            fn expiry_and_leaving_stop_every_independent_dancer() {
                let mut players = world(
                    16,
                    &[(1, HUMAN_M, 0.0, 0.0, ZONE), (2, HUMAN_F, 4.0, 0.0, ZONE)],
                );
                let mut live = seeded(6);
                let rainbow = row(RAINBOW);
                press(&mut live, &rainbow, 600, 1_000, &players);
                live.tick(1_000, &mut players);
                // Walking out of range frees only the leaver, told to them and their viewer.
                walk(&mut players, 1, 40.0);
                let left = live.tick(1_100, &mut players);
                assert_eq!(idles(&left, 1), [601]);
                assert_eq!(idles(&left, 2), [601], "session 2 knows session 1");
                walk(&mut players, 1, 0.0);
                live.tick(1_200, &mut players);
                // Mounting frees the rider, then the box's own expiry frees the rest.
                players.set_mount(SessionId(1), ride(7));
                let mounted = live.tick(1_300, &mut players);
                assert_eq!(idles(&mounted, 1), [601]);
                players.set_mount(SessionId(1), MountTail::NONE);
                live.tick(1_400, &mut players);
                let ended = live.tick(1_000 + u64::from(rainbow.duration_ms), &mut players);
                assert_eq!(idles(&ended, 1), [601, 602]);
                assert_eq!(idles(&ended, 2), [601, 602]);
                assert!(steps(&ended, 1).is_empty());
                assert!(
                    live.tick(1_000 + u64::from(rainbow.duration_ms) + 50, &mut players)
                        .is_empty()
                );
            }

            #[test]
            fn a_disconnect_mid_dance_tells_nobody_and_the_box_carries_on() {
                let mut players = world(
                    16,
                    &[(1, HUMAN_M, 0.0, 0.0, ZONE), (2, HUMAN_M, 4.0, 0.0, ZONE)],
                );
                let mut live = seeded(13);
                press(&mut live, &row(RAINBOW), 600, 1_000, &players);
                let first = rolled(&live.tick(1_000, &mut players), 2);
                let own = first
                    .iter()
                    .find(|(guid, _)| *guid == 602)
                    .expect("session 2 sees its own clip");
                let _ = players.leave(SessionId(1));
                assert!(
                    live.tick(1_050, &mut players).is_empty(),
                    "the leaver's viewers were told it left"
                );
                // The remaining dancer re-rolls on its own clock.
                let sends = live.tick(ends(RAINBOW, 1_000, own.1), &mut players);
                let second = rolled(&sends, 2);
                assert_eq!(second.len(), 1);
                assert_ne!(second[0].1, own.1);
                // A reconnect is a new session: walking in joins at a fresh clip.
                join(&mut players, 11, HUMAN_M, [900.0, 1.0, 0.0], ZONE);
                assert!(rolled(&live.tick(75_000, &mut players), 11).is_empty());
                walk(&mut players, 11, 3.0);
                assert_eq!(rolled(&live.tick(75_050, &mut players), 11).len(), 1);
            }

            #[test]
            fn the_realmshake_and_xvd_tables_are_the_clients_own() {
                // The two other independent rows keep their model tables (ids from their groups).
                assert_eq!(ids(REALMSHAKE, HUMAN_M).len(), 53);
                assert_eq!(ids(RAINBOW, HUMAN_F), ids(RAINBOW, HUMAN_M));
                // A model with a table but a different one still never rolls outside its own.
                let mut players = world(16, &[(1, HUMAN_F, 0.0, 0.0, ZONE)]);
                let table = ids(RAINBOW, HUMAN_F);
                let mut live = seeded(12);
                press(&mut live, &row(RAINBOW), 600, 0, &players);
                assert!(table.contains(&rolled(&live.tick(0, &mut players), 1)[0].1));
            }
        }
    }

    /// T492: the Realms Roll transform (row 1180, model 1941, dance 3541).
    mod transforms {
        use super::*;

        const REALMS_ROLL: i32 = 1180;
        const JINGLE: i32 = 878;
        const HUMAN_M: i32 = 1;
        const MODEL: i32 = 1941; // human_m_realmsroll.adr (Models.txt:1915)
        const DANCE: i32 = 3541; // emo_dance_rroll_01 (AnimationTypes.xml:1054)
        const OTHER: ZoneInstance = ZoneInstance {
            zone_id: 0,
            instance_id: 1,
        };

        /// 35/14, reader `008d6890`: header, i32 model, u64 guid.
        fn appearance(guid: u64, model: i32) -> Vec<u8> {
            let mut bytes = vec![35, 0, 14, 0];
            bytes.extend(model.to_le_bytes());
            bytes.extend(guid.to_le_bytes());
            bytes
        }

        /// 35/15, reader `008d7230`: header, u64 guid, i32 0.
        fn revert(guid: u64) -> Vec<u8> {
            let mut bytes = vec![35, 0, 15, 0];
            bytes.extend(guid.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes
        }

        /// The `(model, guid)` of every 35/14 a session was sent.
        fn appearances(sends: &Sends, session: u64) -> Vec<(i32, u64)> {
            let packets = to(sends, session);
            let appearance = packets
                .into_iter()
                .filter(|p| p.starts_with(&[35, 0, 14, 0]));
            appearance
                .map(|p| {
                    assert_eq!(p.len(), 16);
                    (
                        i32::from_le_bytes(p[4..8].try_into().unwrap()),
                        u64::from_le_bytes(p[8..16].try_into().unwrap()),
                    )
                })
                .collect()
        }

        /// The guids of every 35/15 a session was sent.
        fn reversions(sends: &Sends, session: u64) -> Vec<u64> {
            let packets = to(sends, session);
            let revert = packets
                .into_iter()
                .filter(|p| p.starts_with(&[35, 0, 15, 0]));
            revert
                .map(|p| {
                    assert_eq!(p.len(), 16);
                    assert_eq!(p[12..16], 0_i32.to_le_bytes());
                    u64::from_le_bytes(p[4..12].try_into().unwrap())
                })
                .collect()
        }

        /// The `(guid, animation, base)` of every 35/8 a session was sent.
        fn animations(sends: &Sends, session: u64) -> Vec<(u64, i32, bool)> {
            let packets = to(sends, session);
            let animation = packets
                .into_iter()
                .filter(|p| p.starts_with(&[35, 0, 8, 0]));
            animation
                .map(|p| {
                    (
                        u64::from_le_bytes(p[4..12].try_into().unwrap()),
                        i32::from_le_bytes(p[12..16].try_into().unwrap()),
                        p[20] == 1,
                    )
                })
                .collect()
        }

        /// The models sent for `guid`, however many other players share the audience.
        fn own(sends: &Sends, session: u64, guid: u64) -> Vec<i32> {
            appearances(sends, session)
                .into_iter()
                .filter(|(_, named)| *named == guid)
                .map(|(model, _)| model)
                .collect()
        }

        /// The animations sent for `guid`, as `(animation, base)`.
        fn own_animations(sends: &Sends, session: u64, guid: u64) -> Vec<(i32, bool)> {
            animations(sends, session)
                .into_iter()
                .filter(|(named, _, _)| *named == guid)
                .map(|(_, animation, base)| (animation, base))
                .collect()
        }

        /// The 35/8 dance a transform row plays: `dance_sequence[0]` with the base flag.
        fn dance(guid: u64, animation: i32) -> Vec<u8> {
            let mut bytes = vec![35, 0, 8, 0];
            bytes.extend(guid.to_le_bytes());
            bytes.extend(animation.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.push(1);
            bytes
        }

        /// The temporary model in the AddPc a viewer was sent for `guid`. The field is +0x144 and
        /// 68 bytes follow it, so it sits at `inner.len() - 72` for any presentation.
        fn add_pc_model(
            messages: &[crate::players::OutboundMessage],
            viewer: u64,
            guid: u64,
        ) -> Option<i32> {
            let message = messages.iter().find(|m| {
                m.recipient == SessionId(viewer)
                    && m.kind == crate::players::MessageKind::Spawn
                    && m.bytes.get(11..19) == Some(&guid.to_le_bytes()[..])
            })?;
            let inner = message.bytes.get(7..)?;
            assert!(inner.starts_with(&[35, 0, 1, 0]));
            let at = inner.len() - 72;
            Some(i32::from_le_bytes(inner[at..at + 4].try_into().unwrap()))
        }

        #[test]
        fn a_player_in_range_is_transformed_then_dances_to_the_player_and_a_viewer() {
            // Session 2 is 20 m away: inside the 100 m interest radius (so it knows session 1)
            // but outside the box's 15 m range, so it only watches.
            let mut players = world(
                16,
                &[(1, HUMAN_M, 0.0, 0.0, ZONE), (2, HUMAN_M, 20.0, 0.0, ZONE)],
            );
            let mut live = Boomboxes::default();
            press(&mut live, &row(REALMS_ROLL), 600, 0, &players);
            let sends = live.tick(10, &mut players);
            for session in [1, 2] {
                assert_eq!(
                    to(&sends, session),
                    [
                        appearance(601, MODEL).as_slice(),
                        dance(601, DANCE).as_slice()
                    ],
                    "35/14 then 35/8 base {DANCE} for session {session}"
                );
            }
            assert!(
                own(&sends, 2, 602).is_empty(),
                "the far viewer is not transformed itself"
            );
            // A later viewer gets the model in AddPc.
            let later = join(&mut players, 3, HUMAN_M, [20.0, 1.0, 0.0], ZONE);
            assert_eq!(add_pc_model(&later, 3, 601), Some(MODEL));
        }

        #[test]
        fn leaving_range_reverts_then_idles_once_and_a_disconnect_is_silent() {
            let mut players = world(
                16,
                &[(1, HUMAN_M, 0.0, 0.0, ZONE), (2, HUMAN_M, 20.0, 0.0, ZONE)],
            );
            let mut live = Boomboxes::default();
            press(&mut live, &row(REALMS_ROLL), 600, 0, &players);
            live.tick(10, &mut players);
            walk(&mut players, 1, 40.0);
            let sends = live.tick(50, &mut players);
            for session in [1, 2] {
                assert_eq!(reversions(&sends, session), [601]);
                assert_eq!(idles(&sends, session), [601], "then loc_stand");
                assert_eq!(
                    to(&sends, session),
                    [revert(601).as_slice(), dance(601, 1).as_slice()],
                    "the reader's 35/15 bytes, then idle"
                );
            }
            assert!(
                live.tick(100, &mut players).is_empty(),
                "once, then nothing"
            );
            // A disconnect is never told anything (the client dropped the actor already).
            walk(&mut players, 1, 0.0);
            live.tick(150, &mut players);
            let _ = players.leave(SessionId(1));
            let silent = live.tick(200, &mut players);
            assert!(to(&silent, 1).is_empty());
        }

        #[test]
        fn leaving_range_restores_a_live_consumable_transform_after_the_revert() {
            let mut players = world(
                16,
                &[(1, HUMAN_M, 0.0, 0.0, ZONE), (2, HUMAN_M, 20.0, 0.0, ZONE)],
            );
            // T512: the consumable effect stack has a live temporary appearance on session 1.
            let mut look = players.get(SessionId(1)).unwrap().presentation.clone();
            look.temporary_model = 169;
            players.set_presentation(SessionId(1), look).unwrap();
            let mut live = Boomboxes::default();
            press(&mut live, &row(REALMS_ROLL), 600, 0, &players);
            live.tick(0, &mut players);
            walk(&mut players, 1, 40.0);
            let sends = live.tick(50, &mut players);
            for session in [1, 2] {
                assert_eq!(
                    to(&sends, session),
                    [
                        revert(601).as_slice(),
                        appearance(601, 169).as_slice(),
                        dance(601, 1).as_slice(),
                    ],
                    "revert, the still-live consumable look, then idle"
                );
            }
        }

        #[test]
        fn the_transform_box_beats_an_overlapping_jingle_bell_and_neither_lists_them() {
            let mut players = world(
                16,
                &[(1, HUMAN_M, 0.0, 0.0, ZONE), (2, HUMAN_M, 4.0, 0.0, ZONE)],
            );
            let mut live = Boomboxes::default();
            press(&mut live, &row(JINGLE), 601, 0, &players);
            let first = live.tick(0, &mut players);
            assert_eq!(steps(&first, 1), [[(601, 3531), (602, 3531)]]);
            assert_eq!(steps(&first, 2), [[(601, 3531), (602, 3531)]]);
            // Realms Roll at the same spot: priority 100 against Jingle Bell's 0.
            press(&mut live, &row(REALMS_ROLL), 600, 10, &players);
            let overlap = live.tick(50, &mut players);
            for (session, guid) in [(1, 601), (2, 602)] {
                assert_eq!(own(&overlap, session, guid), [MODEL]);
                assert_eq!(own_animations(&overlap, session, guid), [(DANCE, true)]);
            }
            // Neither box's 35/63 names them, even when Jingle Bell's next step begins.
            for session in [1, 2] {
                assert!(steps(&overlap, session).is_empty());
            }
            let next = live.tick(4_234, &mut players);
            for session in [1, 2] {
                assert!(steps(&next, session).is_empty(), "not listed");
                assert!(appearances(&next, session).is_empty(), "not re-applied");
            }
        }

        #[test]
        fn an_expired_transform_hands_over_to_a_sync_box_after_the_revert() {
            // Session 2 is a viewer only: 20 m from the 15 m boxes.
            let mut players = world(
                16,
                &[(1, HUMAN_M, 0.0, 0.0, ZONE), (2, HUMAN_M, 20.0, 0.0, ZONE)],
            );
            let mut short = row(REALMS_ROLL);
            short.duration_ms = 50;
            let mut live = Boomboxes::default();
            press(&mut live, &short, 600, 0, &players);
            press(&mut live, &row(JINGLE), 601, 0, &players);
            // Both boxes are on the spot; priority 100 keeps the player transformed first.
            let transformed = live.tick(0, &mut players);
            assert_eq!(appearances(&transformed, 1), [(MODEL, 601)]);
            assert!(steps(&transformed, 1).is_empty());
            // The transform box expires and Jingle Bell takes over. The revert must precede the
            // new 35/63, or the dance starts on the temporary model.
            let sends = live.tick(60, &mut players);
            for session in [1, 2] {
                let packets: Vec<&[u8]> = to(&sends, session)
                    .into_iter()
                    .filter(|p| p.starts_with(&[35, 0, 15, 0]) || p.starts_with(&[35, 0, 63, 0]))
                    .collect();
                assert_eq!(packets.len(), 2, "{packets:?}");
                assert_eq!(packets[0], revert(601).as_slice(), "35/15 first");
                assert!(packets[1].starts_with(&[35, 0, 63, 0]), "then the step");
            }
        }

        #[test]
        fn a_mount_change_reapplies_the_model_once_after_two_seconds() {
            let mut players = world(
                16,
                &[(1, HUMAN_M, 0.0, 0.0, ZONE), (2, HUMAN_M, 20.0, 0.0, ZONE)],
            );
            let mut live = Boomboxes::default();
            press(&mut live, &row(REALMS_ROLL), 600, 0, &players);
            live.tick(1_000, &mut players);
            players.set_mount(SessionId(1), ride(9));
            let mounted = live.tick(1_050, &mut players);
            assert_eq!(idles(&mounted, 1), [601], "the dance stops");
            assert!(appearances(&mounted, 1).is_empty(), "not immediately");
            assert_eq!(to(&mounted, 2).len(), 1, "the viewer sees the idle");
            assert!(live.tick(3_049, &mut players).is_empty(), "2 s not yet");
            let due = live.tick(3_050, &mut players);
            assert_eq!(appearances(&due, 1), [(MODEL, 601)]);
            assert_eq!(appearances(&due, 2), [(MODEL, 601)]);
            assert!(
                animations(&due, 1).is_empty(),
                "mounted players get no dance"
            );
            assert!(live.tick(3_100, &mut players).is_empty(), "once");
            // Off the ride: the model is re-sent after the delay and the dance starts again.
            players.set_mount(SessionId(1), MountTail::NONE);
            assert!(live.tick(3_150, &mut players).is_empty());
            let ready = live.tick(5_150, &mut players);
            assert_eq!(appearances(&ready, 1), [(MODEL, 601)]);
            assert_eq!(animations(&ready, 1), [(601, DANCE, true)]);
        }

        #[test]
        fn a_mounted_player_is_transformed_but_does_not_dance() {
            let mut players = world(
                16,
                &[(1, HUMAN_M, 0.0, 0.0, ZONE), (2, HUMAN_M, 20.0, 0.0, ZONE)],
            );
            players.set_mount(SessionId(1), ride(9));
            let mut live = Boomboxes::default();
            press(&mut live, &row(REALMS_ROLL), 600, 0, &players);
            let sends = live.tick(10, &mut players);
            assert_eq!(appearances(&sends, 1), [(MODEL, 601)]);
            assert!(animations(&sends, 1).is_empty(), "no 3541 mounted");
            assert_eq!(appearances(&sends, 2), [(MODEL, 601)]);
        }

        #[test]
        fn expiry_reverts_then_idles_to_the_player_and_their_viewer() {
            let mut players = world(
                16,
                &[(1, HUMAN_M, 0.0, 0.0, ZONE), (2, HUMAN_M, 20.0, 0.0, ZONE)],
            );
            let mut live = Boomboxes::default();
            press(&mut live, &row(REALMS_ROLL), 600, 0, &players);
            live.tick(1_000, &mut players);
            let ended = live.tick(180_000, &mut players);
            for session in [1, 2] {
                assert_eq!(reversions(&ended, session), [601]);
                assert_eq!(idles(&ended, session), [601]);
            }
            assert!(live.tick(180_050, &mut players).is_empty(), "then nothing");
        }

        #[test]
        fn a_reconnect_starts_untransformed_and_the_next_tick_transforms_again() {
            let mut players = world(16, &[(1, HUMAN_M, 0.0, 0.0, ZONE)]);
            let mut live = Boomboxes::default();
            press(&mut live, &row(REALMS_ROLL), 600, 0, &players);
            live.tick(1_000, &mut players);
            let _ = players.leave(SessionId(1));
            let joined = join(&mut players, 11, HUMAN_M, [0.0, 1.0, 0.0], ZONE);
            assert!(joined.is_empty(), "nobody else is near the new session");
            let late = join(&mut players, 12, HUMAN_M, [20.0, 1.0, 0.0], ZONE);
            assert_eq!(add_pc_model(&late, 12, 611), Some(0), "starts plain");
            let sends = live.tick(1_100, &mut players);
            assert_eq!(appearances(&sends, 11), [(MODEL, 611)]);
            assert_eq!(animations(&sends, 11), [(611, DANCE, true)]);
            let later = join(&mut players, 13, HUMAN_M, [20.0, 1.0, 0.0], ZONE);
            assert_eq!(add_pc_model(&later, 13, 611), Some(MODEL));
        }

        #[test]
        fn a_replaced_transform_box_lets_its_players_go() {
            let mut players = world(16, &[(1, HUMAN_M, 0.0, 0.0, ZONE)]);
            let mut live = Boomboxes::default();
            press(&mut live, &row(REALMS_ROLL), 600, 0, &players);
            live.tick(0, &mut players);
            // The owner presses again 500 m away: the old box ends under the player.
            live.spawn(
                Arc::new(row(TOTEM)),
                600,
                ZONE,
                [500.0, 1.0, 0.0],
                0.0,
                1_000,
                &players,
            );
            let sends = live.tick(1_050, &mut players);
            assert_eq!(reversions(&sends, 1), [601]);
            assert_eq!(idles(&sends, 1), [601]);
        }

        #[test]
        fn zoning_out_is_silent_and_clears_the_add_pc_model() {
            let mut players = world(
                16,
                &[(1, HUMAN_M, 0.0, 0.0, ZONE), (2, HUMAN_M, 20.0, 0.0, ZONE)],
            );
            let mut live = Boomboxes::default();
            press(&mut live, &row(REALMS_ROLL), 600, 0, &players);
            live.tick(1_000, &mut players);
            players
                .update_pose_with_state(SessionId(1), OTHER, [0.0, 1.0, 0.0], 0.0, 0, 0)
                .unwrap();
            // The new zone's AddPc is already plain; the scene change itself ended the
            // transform, before the next box tick could notice.
            let early = join(&mut players, 11, HUMAN_M, [20.0, 1.0, 0.0], OTHER);
            assert_eq!(add_pc_model(&early, 11, 601), Some(0));
            let sends = live.tick(1_100, &mut players);
            assert!(
                to(&sends, 1).is_empty() && to(&sends, 2).is_empty(),
                "the client dropped the actor with the scene"
            );
            let late = join(&mut players, 12, HUMAN_M, [20.0, 1.0, 0.0], OTHER);
            assert_eq!(add_pc_model(&late, 12, 601), Some(0));
        }
    }

    /// T536: the world's permanent boxes, the radio props.
    mod permanent {
        use super::*;

        const XVD: i32 = 48244; // the radios' catalogue row: independent dances, no songs
        const RADIO: u64 = 100_000_033_617;
        const OTHER_RADIO: u64 = 100_000_033_838;
        const HUMAN_M: i32 = 1;

        /// A world with permanent XVD-S radios at `(actor guid, position)`; every draw is seeded.
        fn radios(at: &[(u64, [f32; 3])]) -> Boomboxes {
            let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
            let props: Vec<_> = at
                .iter()
                .map(|(spawn_guid, position)| world::BoomboxProp {
                    spawn_guid: *spawn_guid,
                    item_id: XVD,
                    position: *position,
                })
                .collect();
            let mut live =
                Boomboxes::permanent(&Catalogue::load(&directory).unwrap(), &props).unwrap();
            live.rng = Rng::seeded(3);
            live
        }

        /// The animation each single-entry 35/63 sent to `session` plays for character `guid`.
        fn rolled(sends: &Sends, session: u64, guid: u64) -> Vec<i32> {
            let found = steps(sends, session).into_iter().flatten();
            found
                .filter(|(who, _)| *who == guid)
                .map(|(_, a)| a)
                .collect()
        }

        fn xvd_ids(model: i32) -> Vec<i32> {
            row(XVD).clips[&model]
                .iter()
                .map(|c| c.animation_id)
                .collect()
        }

        #[test]
        fn a_permanent_box_never_expires_and_sends_no_scenery() {
            let mut players = world(16, &[(1, HUMAN_M, 0.0, 0.0, ZONE)]);
            let mut live = radios(&[(RADIO, [5.0, 1.0, 0.0])]);
            // Each tick is past the last clip's end, so each one re-rolls: a box that expired
            // would send its removal and then nothing, so no step would follow.
            for now in [0, 180_000, 600_000, 99_999_999_999] {
                let sends = live.tick(now, &mut players);
                assert!(
                    sends.iter().all(|(_, p)| p.starts_with(&[35, 0, 63, 0])),
                    "tick {now}: only dance steps, never an AddNpc, poof, song or removal"
                );
                let rolls = rolled(&sends, 1, 601);
                assert_eq!(rolls.len(), 1, "tick {now}");
                assert!(
                    xvd_ids(HUMAN_M).contains(&rolls[0]),
                    "a clip of the model's table"
                );
            }
            assert_eq!(live.live.len(), 1);
            assert!(live.live[0].permanent());
        }

        #[test]
        fn a_player_at_the_radio_dances_and_stops_on_leaving() {
            let mut players = world(
                16,
                &[(1, HUMAN_M, 0.0, 0.0, ZONE), (2, 1941, 1.0, 0.0, ZONE)],
            );
            let mut live = radios(&[(RADIO, [5.0, 1.0, 0.0])]);
            let sends = live.tick(1_000, &mut players);
            // A model with a table rolls one of its clips; a model without one stands on the
            // row's standing group, as a pressed XVD-S box does.
            assert!(xvd_ids(HUMAN_M).contains(&rolled(&sends, 1, 601)[0]));
            let standing = animation_packet(602, row(XVD).standing_animation_id, true);
            let bases = to(&sends, 2)
                .into_iter()
                .filter(|p| p.starts_with(&[35, 0, 8, 0]));
            assert_eq!(bases.collect::<Vec<_>>(), [standing.as_slice()]);
            // Out of its 15 m range: each dancer is told idle, and so is the one who sees it.
            walk(&mut players, 1, 40.0);
            walk(&mut players, 2, 40.0);
            let left = live.tick(1_100, &mut players);
            assert_eq!(idles(&left, 1), [601, 602]);
            assert_eq!(idles(&left, 2), [601, 602]);
            assert!(live.tick(1_200, &mut players).is_empty());
            // Back in range, the dance starts again.
            walk(&mut players, 1, 0.0);
            let back = live.tick(1_300, &mut players);
            assert_eq!(rolled(&back, 1, 601).len(), 1);
        }

        #[test]
        fn a_viewer_out_of_range_of_the_radio_sees_the_dancer_and_does_not_dance() {
            let mut players = world(
                16,
                &[(1, HUMAN_M, 0.0, 0.0, ZONE), (2, HUMAN_M, 60.0, 0.0, ZONE)],
            );
            let mut live = radios(&[(RADIO, [5.0, 1.0, 0.0])]);
            // Session 2 is 55 m from the radio, so it does not dance, but it knows session 1 and
            // is told its step like a viewer of any box.
            let sends = live.tick(1_000, &mut players);
            assert_eq!(rolled(&sends, 1, 601).len(), 1);
            assert_eq!(rolled(&sends, 2, 601), rolled(&sends, 1, 601));
            assert!(rolled(&sends, 2, 602).is_empty() && idles(&sends, 2).is_empty());
        }

        #[test]
        fn a_players_own_box_next_to_a_radio_follows_the_priority_rule() {
            let mut players = world(16, &[(1, HUMAN_M, 0.0, 0.0, ZONE)]);
            let own = row(TOTEM); // priority 0, stands 2 m away, steps 3551
            let mut loud = own.clone();
            loud.priority = 5;
            // The radio is 1 m away, nearer than the player's own box: equal priority, the
            // nearer one wins, and the dancer is listed by that box only.
            let mut live = radios(&[(RADIO, [1.0, 1.0, 0.0])]);
            press(&mut live, &own, 600, 0, &players);
            let sends = live.tick(10, &mut players);
            let rolls = rolled(&sends, 1, 601);
            assert_eq!(rolls.len(), 1);
            assert!(
                xvd_ids(HUMAN_M).contains(&rolls[0]),
                "the radio won: {rolls:?}"
            );
            // A higher-priority own box of the same player wins over the radio.
            let mut live = radios(&[(RADIO, [1.0, 1.0, 0.0])]);
            press(&mut live, &loud, 600, 0, &players);
            let sends = live.tick(10, &mut players);
            assert_eq!(rolled(&sends, 1, 601), [3551]);
            // When that box ends, the radio takes the player over, with no idle between.
            let after = live.tick(u64::from(loud.duration_ms) + 10, &mut players);
            assert!(idles(&after, 1).is_empty());
            assert!(xvd_ids(HUMAN_M).contains(&rolled(&after, 1, 601)[0]));
        }

        #[test]
        fn a_press_never_replaces_a_permanent_box() {
            let players = world(16, &[(1, HUMAN_M, 0.0, 0.0, ZONE)]);
            let mut live = radios(&[(RADIO, [5.0, 1.0, 0.0])]);
            // The radio's owner field is 0; a press by owner 0 must still leave it standing.
            press(&mut live, &row(TOTEM), 0, 0, &players);
            press(&mut live, &row(TOTEM), 0, 10, &players);
            assert_eq!(live.live.len(), 2);
            assert!(live.live[0].permanent() && live.live[0].guid == RADIO);
        }

        #[test]
        fn a_restart_keeps_the_radios_afresh_and_drops_pressed_boxes_and_dancers() {
            let mut players = world(16, &[(1, HUMAN_M, 0.0, 0.0, ZONE)]);
            let mut live = radios(&[(RADIO, [5.0, 1.0, 0.0]), (OTHER_RADIO, [900.0, 1.0, 0.0])]);
            press(&mut live, &row(TOTEM), 600, 0, &players);
            live.tick(10, &mut players);
            let mut again = live.restarted();
            assert_eq!(
                again.live.iter().map(|b| b.guid).collect::<Vec<_>>(),
                [RADIO, OTHER_RADIO]
            );
            // The dancer's clock was dropped: the first tick rolls again at once.
            let sends = again.tick(20, &mut players);
            assert_eq!(rolled(&sends, 1, 601).len(), 1);
        }

        #[test]
        fn an_unknown_item_fails_naming_the_actor_and_no_props_change_nothing() {
            let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
            let catalogue = Catalogue::load(&directory).unwrap();
            let prop = |spawn_guid, item_id| world::BoomboxProp {
                spawn_guid,
                item_id,
                position: [0.0; 3],
            };
            let error = Boomboxes::permanent(&catalogue, &[prop(RADIO, XVD), prop(77, 1)])
                .unwrap_err()
                .to_string();
            assert!(
                error.contains("77") && error.contains("boombox item 1"),
                "{error}"
            );
            let mut players = registry();
            let mut none = Boomboxes::permanent(&catalogue, &[]).unwrap();
            assert!(none.live.is_empty() && none.tick(0, &mut players).is_empty());
        }
    }

    /// T537: a dancing owner's summoned pet dances beside them.
    mod pets {
        use super::*;
        use crate::pets::PetPresence;

        const JINGLE: i32 = 878; // synchronized; model 1 has clips
        const HUMAN_M: i32 = 1;
        /// A pet actor id that is not derived from its owner, so a packet naming it proves the
        /// tick uses the presence's own `actor_guid` and never recomputes one.
        const PET_ACTOR: u64 = 0x4d49_4c4b_0000_0007;

        fn presence(owner: u64) -> PetPresence {
            PetPresence {
                owner_guid: owner,
                pet_id: 3,
                actor_guid: PET_ACTOR,
                species_id: 652,
                model: "wolf_coyote.adr".into(),
                model_id: 1,
                texture_alias: String::new(),
                name: String::new(),
                speed: 1.0,
                position: [0.0, 1.0, 0.0],
            }
        }

        /// The world's pet table: `owner`'s one summoned pet.
        fn pets_of(owner: u64) -> BTreeMap<u64, PetPresence> {
            BTreeMap::from([(owner, presence(owner))])
        }

        /// A service with the `pets` block (3201 after 4000 ms plus a gap in `min_ms..=max_ms`).
        fn service(seed: u64, min_ms: u32, max_ms: u32) -> Boomboxes {
            let mut live = Boomboxes::default();
            live.pet_dance = Some(Arc::new(PetDance {
                pet_dance_animation_id: 3_201,
                pet_dance_ms: 4_000,
                pet_gap_ms: PetGap { min_ms, max_ms },
                source: Vec::new(),
            }));
            live.rng = Rng::seeded(seed);
            live
        }

        /// The pet plays a session was sent: 35/8 `(guid, animation)` with the base flag clear.
        fn plays(sends: &Sends, session: u64) -> Vec<(u64, i32)> {
            let plays = to(sends, session)
                .into_iter()
                .filter(|packet| packet.starts_with(&[35, 0, 8, 0]) && packet[20] == 0);
            plays
                .map(|packet| {
                    (
                        u64::from_le_bytes(packet[4..12].try_into().unwrap()),
                        i32::from_le_bytes(packet[12..16].try_into().unwrap()),
                    )
                })
                .collect()
        }

        /// When each of the first four plays happens, from one tick a millisecond.
        fn schedule(seed: u64) -> Vec<u64> {
            let mut players = world(16, &[(1, HUMAN_M, 0.0, 0.0, ZONE)]);
            let mut live = service(seed, 500, 2_000);
            press(&mut live, &row(JINGLE), 600, 0, &players);
            let pets = pets_of(601);
            let mut at = Vec::new();
            let mut now = 0;
            // Bounded: four plays of at most 6 s each fit well inside 60 s.
            while at.len() < 4 && now < 60_000 {
                if !plays(&live.tick_with_pets(now, &mut players, &pets), 1).is_empty() {
                    at.push(now);
                }
                now += 1;
            }
            at
        }

        #[test]
        fn a_dancing_owners_pet_plays_on_its_own_actor_every_dance_plus_gap() {
            let mut players = world(
                16,
                &[
                    (1, HUMAN_M, 0.0, 0.0, ZONE),   // the dancer, and the pet's owner
                    (2, HUMAN_M, 4.0, 0.0, ZONE),   // a viewer near them
                    (3, HUMAN_M, 900.0, 0.0, ZONE), // out of the box's range
                ],
            );
            let mut live = service(7, 500, 500);
            press(&mut live, &row(JINGLE), 600, 1_000, &players);
            let pets = pets_of(601);
            // The client's own 21 bytes: 35/8, the pet's guid, 3201, an i32 0 and flags 0 (play now,
            // reader `00908a50`; bit 0 would store it as the entity's base animation instead).
            let mut want = vec![35, 0, 8, 0];
            want.extend(PET_ACTOR.to_le_bytes());
            want.extend(3_201_i32.to_le_bytes());
            want.extend([0; 5]);
            assert_eq!(want.len(), 21);
            let first = live.tick_with_pets(1_000, &mut players, &pets);
            for session in [1, 2] {
                assert!(
                    to(&first, session).contains(&want.as_slice()),
                    "session {session} hears the pet"
                );
                assert_eq!(plays(&first, session), [(PET_ACTOR, 3_201)]);
            }
            assert!(to(&first, 3).is_empty(), "the far session is not shown it");
            // Four seconds of dance and the block's 500 ms gap: nothing a millisecond early.
            assert!(plays(&live.tick_with_pets(5_499, &mut players, &pets), 1).is_empty());
            assert_eq!(
                plays(&live.tick_with_pets(5_500, &mut players, &pets), 1),
                [(PET_ACTOR, 3_201)]
            );
            assert!(plays(&live.tick_with_pets(5_501, &mut players, &pets), 1).is_empty());
        }

        #[test]
        fn the_gap_is_drawn_from_the_block_and_a_seed_repeats_the_schedule() {
            let first = schedule(11);
            assert_eq!(first.len(), 4);
            assert_eq!(first[0], 0, "the first play is immediate");
            for pair in first.windows(2) {
                let gap = pair[1] - pair[0] - 4_000;
                assert!(
                    (500..=2_000).contains(&gap),
                    "gap {gap} ms ending at {}",
                    pair[1]
                );
            }
            assert!(
                first.windows(3).any(|w| w[1] - w[0] != w[2] - w[1]),
                "the gap is drawn per play: {first:?}"
            );
            assert_eq!(first, schedule(11), "a seed repeats the schedule");
            assert_ne!(first, schedule(12), "another seed draws another one");
        }

        #[test]
        fn the_pet_dance_stops_with_the_dance_or_the_pet() {
            let mut players = world(16, &[(1, HUMAN_M, 0.0, 0.0, ZONE)]);
            let mut live = service(5, 500, 500);
            press(&mut live, &row(JINGLE), 600, 0, &players);
            // No summoned pet: the dancer dances alone.
            let none: BTreeMap<u64, PetPresence> = BTreeMap::new();
            assert!(plays(&live.tick_with_pets(0, &mut players, &none), 1).is_empty());
            // Summoned mid-dance: the pet plays at once.
            let mut pets = pets_of(601);
            assert_eq!(
                plays(&live.tick_with_pets(100, &mut players, &pets), 1),
                [(PET_ACTOR, 3_201)]
            );
            // Dismissed mid-dance: the world's table loses the owner and nothing more is sent.
            pets.remove(&601);
            assert!(plays(&live.tick_with_pets(4_600, &mut players, &pets), 1).is_empty());
            // Summoned again: it plays at once; then the owner walks out of the box's range.
            pets = pets_of(601);
            assert_eq!(
                plays(&live.tick_with_pets(5_000, &mut players, &pets), 1),
                [(PET_ACTOR, 3_201)]
            );
            walk(&mut players, 1, 40.0);
            assert!(plays(&live.tick_with_pets(9_500, &mut players, &pets), 1).is_empty());
            // Back in range the dance restarts and the pet plays again, then the box ends.
            walk(&mut players, 1, 0.0);
            assert_eq!(
                plays(&live.tick_with_pets(10_000, &mut players, &pets), 1),
                [(PET_ACTOR, 3_201)]
            );
            assert!(plays(&live.tick_with_pets(10_001, &mut players, &pets), 1).is_empty());
            // A disconnect mid-dance: nothing is sent to the leaver, and a reconnect starts the
            // dance (and the pet's clock) from scratch.
            let _ = players.leave(SessionId(1));
            assert!(plays(&live.tick_with_pets(20_000, &mut players, &pets), 1).is_empty());
            join(&mut players, 1, HUMAN_M, [0.0, 1.0, 0.0], ZONE);
            assert_eq!(
                plays(&live.tick_with_pets(20_050, &mut players, &pets), 1),
                [(PET_ACTOR, 3_201)]
            );
            let ended = live.tick_with_pets(180_000, &mut players, &pets);
            assert!(
                plays(&ended, 1).is_empty(),
                "the box's end stops the pet too"
            );
            assert!(plays(&live.tick_with_pets(180_050, &mut players, &pets), 1).is_empty());
        }

        #[test]
        fn without_the_pets_block_the_pet_never_dances() {
            let mut players = world(16, &[(1, HUMAN_M, 0.0, 0.0, ZONE)]);
            let mut live = Boomboxes::default();
            assert!(live.pet_dance.is_none(), "the block is absent");
            press(&mut live, &row(JINGLE), 600, 0, &players);
            let pets = pets_of(601);
            for now in [0, 4_500, 9_000, 13_500] {
                let sends = live.tick_with_pets(now, &mut players, &pets);
                assert!(plays(&sends, 1).is_empty(), "tick {now}");
                assert!(
                    !sends
                        .iter()
                        .any(|(_, packet)| packet.get(4..12) == Some(&PET_ACTOR.to_le_bytes()[..])),
                    "tick {now}: no packet names the pet"
                );
            }
        }

        #[test]
        fn a_restart_keeps_the_pets_block_and_forgets_the_clocks() {
            let mut live = service(5, 500, 500);
            live.pet_dancers.insert(SessionId(1), 9);
            let again = live.restarted();
            assert!(
                again.pet_dance.is_some(),
                "the block survives a fault restart"
            );
            assert!(again.pet_dancers.is_empty(), "the dancers do not");
        }
    }
}
