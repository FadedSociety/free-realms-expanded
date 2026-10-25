//! The boombox catalogue (`boomboxes.json`, T459), one row per client boombox item, and the
//! boxes pressed from the item bar (T461): a [`Boomboxes`] set of live boxes. A press puts a box
//! beside the player with a poof and its song, never uses up the item, and removes the box
//! after the row's duration. Everything here is session-only; nothing is saved.
//!
//! # Recovered original facts
//!
//! * The client offers an item for the item bar (`BaseClient.Inventory.Consumables`, row
//!   predicate `00c31de0`) when its definition's type (+0x6c) is 1, its activatable ability
//!   (+0x8c) is positive, its gender usage fits (`00bae6a0`) and a unit is left. All 15 boombox
//!   definitions the server loads pass, so the client sends 42/6 for them.
//! * Every model, animation and song effect id is checked against the client's own tables by
//!   `tests/boombox_catalogue.rs`; each row's `source` cites where its values come from.
//! * The song is a composite effect attached to the box by tag, 35/41 (client dispatcher
//!   jump table `0092F540` case 41: ctor `008c4d50`, reader `008f82e0` -> `008d7ad0`: guid at
//!   +0x10, tag at +0x18) and stopped by 35/42 (case 42, ctor `008b51c0`: guid +0x10, tag
//!   +0x18). The box's own bounce is `amb_loop_01` (`AnimationTypes.xml:844`, id 2100) and the
//!   poof is effect 21 `PFX_smoke_black_explosion` (`ActorCompositeEffectDefinitions.xml:283`).
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
//! * The song is one random effect of the row's `song_effect_ids`, one pick per spawn (R: the
//!   rows and their six colour effects are Sanctuary's and the client's; no client spawn-time
//!   pick was located). It is attached by 35/41 with the owner as the effect's source guid, as
//!   `Placed::show` passes it; the reader (`008d7ad0`) calls that field the source, and what the
//!   client does with it for a song is unrecovered.
//! * A viewer who leaves the box's range is sent the removal (35/3) and sees the box again, with
//!   its song, on re-entry. One box per owner: a second press before the first ends replaces it
//!   (the default recast equals the duration, so this only happens if the two are tuned apart).
//! * A box outlives its owner's session until it expires.
//! * Dancing, each row kind by its own rule. Every tick a session within `range_m` of the box on
//!   X/Z, in its zone instance and on foot dances; overlapping boxes go to the higher priority,
//!   then the nearer (Sanctuary rotates by least served instead). `synchronized` rows (T463) need
//!   a model in `clip_ms`: one shared sequence, a step lasting the longest clip of any model
//!   minus `blend_ms`, never less than `MIN_STEP_MS`, everyone listed in one 35/63 per viewer.
//!   `independent` rows (T494) roll every dancer its own `clips` entry for its model, never the
//!   one just played, re-rolled `blend_ms` before its end; each clip is its own single-entry
//!   35/63 for the dancer and every viewer who knows them, and a model with no table stands on
//!   the row's `standing_animation_id` as a base 35/8. A player who starts dancing mid-step gets
//!   the current step at once (Sanctuary sends only an idle and waits for the next step). A
//!   dancer who leaves, mounts, or whose box ends or is replaced gets 35/8 idle, sent to them and
//!   their viewers. A viewer who walks into range sees the dancer idle until the next step. One
//!   synchronized packet carries at most `MAX_ENTRIES`.

use crate::{
    abilities::effect_packets,
    encounter::SharedFieldState,
    players::{PlayerRegistry, SessionId, ZoneInstance},
    power_ups::Rng,
    startup_world::{self, NpcSpawn},
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
    /// Dancers take `transform_model_id`; only this kind has one.
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

/// One boombox. Ids of 0 mean "none" for `standing_animation_id` and `transform_model_id`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Boombox {
    pub item_id: i32,
    pub model_id: i32,
    pub kind: Kind,
    /// The synchronized sequence, one animation per step (R: Sanctuary's rows). Empty on an
    /// `Independent` row, whose only dance ids are its `clips`; the loader refuses a sequence on
    /// one and requires it on the others.
    #[serde(default)]
    pub dance_sequence: Vec<i32>,
    /// Player model id -> one clip length per dance step. Empty for `Independent` boxes, which use
    /// `clips` instead.
    pub clip_ms: BTreeMap<i32, Vec<u32>>,
    /// Player model id -> the clips that model may be rolled, in any order (independent rows only;
    /// empty otherwise). A model missing from the map stands on `standing_animation_id`.
    #[serde(default)]
    pub clips: BTreeMap<i32, Vec<Clip>>,
    pub blend_ms: u32,
    pub song_effect_ids: Vec<i32>,
    pub standing_animation_id: i32,
    pub priority: i32,
    pub transform_model_id: i32,
    /// 0 unless `kind` is `Transform`.
    pub transform_reapply_ms: u32,
    pub duration_ms: u32,
    pub range_m: f32,
    pub spawn_offset_m: f32,
    pub recast_ms: u32,
    pub texture_alias: String,
    pub tint_alias: String,
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
        self.item_id > 0
            && self.model_id > 0
            && (self.kind == Kind::Independent) == self.dance_sequence.is_empty()
            && self.dance_sequence.iter().all(|id| *id > 0)
            && self.song_effect_ids.iter().all(|id| *id > 0)
            && self.standing_animation_id >= 0
            && (self.kind == Kind::Independent) == self.clip_ms.is_empty()
            && self.clip_ms.iter().all(|(model, clips)| {
                *model > 0
                    && clips.len() == self.dance_sequence.len()
                    && clips.iter().all(|v| ms(*v))
            })
            && (self.kind == Kind::Independent) == !self.clips.is_empty()
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogueFile {
    schema_version: u32,
    boomboxes: Vec<Boombox>,
}

/// Every boombox row, read-only and shared. Nothing about it is saved.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Catalogue(Vec<Boombox>);

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
        Ok(Self(rows))
    }

    pub fn get(&self, item_id: i32) -> Option<&Boombox> {
        self.0.iter().find(|row| row.item_id == item_id)
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

/// A dancer a box let go of: session, character guid and the box's zone.
type Released = (SessionId, u64, ZoneInstance);

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
            released.push((session, guid(&dancer), zone));
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
    range_m: f32,
    priority: i32,
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
            range_m: row.range_m,
            priority: row.priority,
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
    /// Player model -> the clips it may play. A model missing here stands. Shared, not copied:
    /// a live box is cloned with the world on every application message.
    clips: Arc<BTreeMap<i32, Vec<Clip>>>,
    blend_ms: u32,
    range_m: f32,
    priority: i32,
    /// The row's `standing_animation_id`: the base 35/8 a model with no table dances.
    standing: i32,
    dancing: BTreeMap<SessionId, Solo>,
}

impl Independent {
    fn new(row: &Boombox) -> Self {
        Self {
            clips: Arc::new(row.clips.clone()),
            blend_ms: row.blend_ms,
            range_m: row.range_m,
            priority: row.priority,
            standing: row.standing_animation_id,
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
    fn new(row: &Boombox) -> Option<Self> {
        match row.kind {
            Kind::Synchronized => Some(Self::Synchronized(Synchronized::new(row))),
            Kind::Independent => Some(Self::Independent(Independent::new(row))),
            Kind::Transform => None,
        }
    }

    fn priority(&self) -> i32 {
        match self {
            Self::Synchronized(dance) => dance.priority,
            Self::Independent(dance) => dance.priority,
        }
    }

    fn range_m(&self) -> f32 {
        match self {
            Self::Synchronized(dance) => dance.range_m,
            Self::Independent(dance) => dance.range_m,
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
    /// The box's AddNpc, built once.
    add: Vec<u8>,
    ends_ms: u64,
    seen_by: BTreeSet<SessionId>,
    /// `None` for `transform` rows, which do not dance.
    dance: Option<Dance>,
}

impl Placed {
    /// Let every dancer go (the box ends or is replaced); the next tick sends each idle unless
    /// another box has taken them.
    fn release(&mut self) -> Vec<Released> {
        let zone = self.zone;
        let dancing = self.dance.as_mut().map(Dance::leave).unwrap_or_default();
        dancing
            .into_iter()
            .map(|(session, guid)| (session, guid, zone))
            .collect()
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
    /// Dancers of boxes that ended or were replaced, sent idle by the next tick.
    released: Vec<Released>,
    /// Draws the song of each box and every independent clip. One stream for the world, like
    /// Sanctuary's `Random.Shared`; the seed comes from the OS (0 if that fails).
    rng: Rng,
}

impl Default for Boomboxes {
    fn default() -> Self {
        Self {
            live: Vec::new(),
            next: 0,
            released: Vec::new(),
            rng: Rng::from_entropy().unwrap_or_else(|_| Rng::seeded(0)),
        }
    }
}

impl Boomboxes {
    /// Put `owner`'s box beside them and show it to everyone in range. A box the owner already
    /// has is ended first. `at` and `heading` are the owner's pose.
    pub fn spawn(
        &mut self,
        row: &Boombox,
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
        let npc = NpcSpawn {
            actor_id: guid,
            model_id: row.model_id,
            texture_alias: row.texture_alias.clone(),
            tint_alias: row.tint_alias.clone(),
            animation_id: BOX_ANIMATION,
            hide_nameplate: 1,
            is_interactable: 0,
            interact_range: 0,
            position: [position[0], position[1], position[2], 1.0],
            rotation: [heading.sin(), 0.0, heading.cos(), 0.0],
            ..world::content_spawn_template()
        };
        let Ok(add) = world::unframe_tunnel(&startup_world::build_add_npc(&npc)) else {
            return sends;
        };
        if let Some(old) = self.live.iter().position(|b| b.owner == owner) {
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
            song: pick(&row.song_effect_ids, &mut self.rng).copied(),
            add,
            ends_ms: now_ms.saturating_add(u64::from(row.duration_ms)),
            seen_by: BTreeSet::new(),
            dance: Dance::new(row),
        };
        for session in players.nearby_sessions(zone, position) {
            placed.show(session, true, &mut sends);
        }
        self.live.push(placed);
        sends
    }

    /// Once per tick: end boxes whose time is up, show the rest to sessions that came into range
    /// and take it back from those that left, then step the dances. No boxes means no work.
    pub fn tick(&mut self, now_ms: u64, players: &PlayerRegistry) -> Sends {
        let mut sends = Sends::new();
        if self.live.is_empty() {
            return sends;
        }
        let (ended, live): (Vec<_>, Vec<_>) = std::mem::take(&mut self.live)
            .into_iter()
            .partition(|b| now_ms >= b.ends_ms);
        self.live = live;
        for mut placed in ended {
            self.released.extend(placed.release());
            placed.end(&mut sends);
        }
        let mut nears = Vec::with_capacity(self.live.len());
        for placed in &mut self.live {
            let near = players.nearby_sessions(placed.zone, placed.position);
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
            nears.push(near);
        }
        self.dance(now_ms, players, &nears, &mut sends);
        sends
    }

    /// Step every box's dance. `nears[i]` are the sessions near live box `i`.
    fn dance(
        &mut self,
        now_ms: u64,
        players: &PlayerRegistry,
        nears: &[BTreeSet<SessionId>],
        sends: &mut Sends,
    ) {
        // A session dances for one box: the highest priority, then the nearest, then the
        // earliest placed. Value: (box index, priority, distance, character guid, shown model).
        let mut owner: BTreeMap<SessionId, (usize, i32, f32, u64, i32)> = BTreeMap::new();
        for (index, (placed, near)) in self.live.iter().zip(nears).enumerate() {
            let Some(dance) = &placed.dance else { continue };
            for session in near {
                let Some(player) = players.get(*session) else {
                    continue;
                };
                let dx = player.position[0] - placed.position[0];
                let dz = player.position[2] - placed.position[2];
                let distance = dx.hypot(dz);
                let model = player.presentation.display_model();
                if !dance.allows(model)
                    || players.is_mounted(*session)
                    || !(distance <= dance.range_m())
                {
                    continue;
                }
                let wins = owner
                    .get(session)
                    .is_none_or(|&(_, priority, nearest, _, _)| {
                        dance.priority() > priority
                            || (dance.priority() == priority && distance < nearest)
                    });
                if wins {
                    let guid = player.character_guid;
                    owner.insert(*session, (index, dance.priority(), distance, guid, model));
                }
            }
        }
        let mut released = std::mem::take(&mut self.released);
        let mut steps: BTreeMap<SessionId, Vec<(u64, i32)>> = BTreeMap::new();
        let rng = &mut self.rng;
        for (index, placed) in self.live.iter_mut().enumerate() {
            let zone = placed.zone;
            let Some(dance) = &mut placed.dance else {
                continue;
            };
            // Who this box owns this tick, straight from the shared winner map.
            let owned = |session: &SessionId| owner.get(session).is_some_and(|o| o.0 == index);
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
                        // Everyone when the step begins; otherwise only who is new to this box.
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
                        match dance.clips.get(&model) {
                            Some(clips) => {
                                if let Some(clip) = choose(clips, past.unwrap_or(0), rng) {
                                    solo.clip = Some(clip.animation_id);
                                    solo.ends_ms = clip_ends(now_ms, clip, dance.blend_ms);
                                    // One single-entry 35/63 per viewer: an independent clip is
                                    // not a group step, so it never shares a packet.
                                    for viewer in audience(players, *session, guid, zone) {
                                        sends.push((
                                            viewer,
                                            synchronized_animations(&[(guid, clip.animation_id)]),
                                        ));
                                    }
                                }
                            }
                            // Standing: send the row's standing group as the base 35/8 once.
                            None => {
                                if past.is_some() || !dance.dancing.contains_key(session) {
                                    for viewer in audience(players, *session, guid, zone) {
                                        let packet = animation_packet(guid, dance.standing, true);
                                        sends.push((viewer, packet));
                                    }
                                }
                            }
                        }
                        dance.dancing.insert(*session, solo);
                    }
                }
            }
        }
        for (viewer, entries) in steps {
            for chunk in entries.chunks(MAX_ENTRIES) {
                sends.push((viewer, synchronized_animations(chunk)));
            }
        }
        // A dancer another box took this tick is already dancing there.
        for (session, guid, zone) in released {
            if owner.contains_key(&session) {
                continue;
            }
            for viewer in audience(players, session, guid, zone) {
                sends.push((viewer, animation_packet(guid, IDLE_ANIMATION, true)));
            }
        }
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
    use crate::players::InterestPolicy;

    const RAINBOW: i32 = 1913; // the one shipped row with songs
    const TOTEM: i32 = 8185; // no songs
    const ZONE: ZoneInstance = ZoneInstance {
        zone_id: 0,
        instance_id: 0,
    };

    fn row(item: i32) -> Boombox {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let catalogue = Catalogue::load(&directory).unwrap();
        catalogue.get(item).unwrap().clone()
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

    #[test]
    fn a_press_shows_the_box_its_poof_and_song_to_those_in_range_only() {
        let mut live = Boomboxes::default();
        let sends = live.spawn(
            &row(RAINBOW),
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
            assert_eq!(npc.texture_alias, "boombox-rainbow-M");
            assert_eq!(
                (npc.hide_nameplate, npc.is_interactable, npc.interact_range),
                (1, 0, 0)
            );
            assert_eq!(packets[1], poof(guid, at));
            let effect = i32::from_le_bytes(packets[2][16..20].try_into().unwrap());
            assert!(row(RAINBOW).song_effect_ids.contains(&effect));
            let mut song = vec![35, 0, 41, 0];
            song.extend(guid.to_le_bytes());
            song.extend((guid as u32 as i32).to_le_bytes());
            song.extend(effect.to_le_bytes());
            song.extend(600_u64.to_le_bytes()); // the owner is the source
            song.extend([0; 8]);
            assert_eq!(packets[2], song);
        }
        assert!(to(&sends, 3).is_empty(), "the far session is not shown it");
        // A row with no songs sends no 35/41.
        let quiet =
            Boomboxes::default().spawn(&row(TOTEM), 600, ZONE, [0.0; 3], 0.0, 0, &registry());
        assert_eq!(to(&quiet, 1).len(), 2);
    }

    #[test]
    fn expiry_stops_the_song_poofs_and_removes_once_then_nothing() {
        let mut live = Boomboxes::default();
        let players = registry();
        live.spawn(
            &row(RAINBOW),
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
        let early = live.tick(1_000 + 179_999, &players);
        let standing = animation_packet(601, 3_550, true);
        assert_eq!(to(&early, 1), [standing.as_slice()]);
        assert_eq!(to(&early, 2), [standing.as_slice()]);
        assert!(to(&early, 3).is_empty(), "the far session is not shown it");
        assert!(early.iter().all(|(_, p)| p.starts_with(&[35, 0, 8, 0])));
        let ended = live.tick(1_000 + 180_000, &players);
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
        assert!(live.tick(1_000 + 999_999, &players).is_empty());
        assert!(live.live.is_empty());
    }

    #[test]
    fn a_session_in_range_later_is_shown_once_and_one_that_left_is_taken_back() {
        let mut live = Boomboxes::default();
        let mut players = registry();
        live.spawn(&row(RAINBOW), 600, ZONE, [0.0, 1.0, 0.0], 0.0, 0, &players);
        let guid = GUID_TAG | 1;
        let arrive = |players: &mut PlayerRegistry| {
            players
                .update_pose_with_state(SessionId(3), ZONE, [10.0, 1.0, 0.0], 0.0, 0, 0)
                .unwrap();
        };
        arrive(&mut players);
        let shown = live.tick(50, &players);
        let packets = scenery(&to(&shown, 3));
        assert_eq!(packets.len(), 2, "AddNpc and song, no poof");
        assert!(packets[0].starts_with(&[35, 0, 2, 0]) && packets[1].starts_with(&[35, 0, 41, 0]));
        assert!(live.tick(100, &players).is_empty(), "exactly once");
        // Walking out sends the removal; coming back shows the box again.
        players
            .update_pose_with_state(SessionId(3), ZONE, [900.0, 1.0, 0.0], 0.0, 0, 0)
            .unwrap();
        let gone = live.tick(150, &players);
        assert_eq!(
            scenery(&to(&gone, 3)),
            [SharedFieldState::removal(guid).as_slice()]
        );
        arrive(&mut players);
        assert_eq!(scenery(&to(&live.tick(200, &players), 3)).len(), 2);
        // A session that left the registry is dropped without a word.
        let _ = players.leave(SessionId(3));
        assert!(live.tick(250, &players).is_empty());
        assert!(live.live[0].seen_by.iter().all(|s| s.0 != 3));
    }

    #[test]
    fn a_second_box_from_one_owner_replaces_the_first() {
        let mut live = Boomboxes::default();
        let players = registry();
        live.spawn(&row(RAINBOW), 600, ZONE, [0.0, 1.0, 0.0], 0.0, 0, &players);
        let sends = live.spawn(&row(TOTEM), 600, ZONE, [0.0, 1.0, 0.0], 0.0, 10, &players);
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
            live.spawn(&row(TOTEM), owner, ZONE, [0.0; 3], 0.0, 0, &players);
        }
        // The counter is about to wrap onto the two live guids (low words 1 and 2).
        live.next = u32::MAX;
        live.spawn(&row(TOTEM), 3, ZONE, [0.0; 3], 0.0, 0, &players);
        live.spawn(&row(TOTEM), 4, ZONE, [0.0; 3], 0.0, 0, &players);
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

    /// T463: synchronized dances.
    mod dancing {
        use super::*;
        use crate::{mounts::MountTail, players::PlayerPresentation};

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
            live.spawn(row, owner, ZONE, [0.0, 1.0, 0.0], 0.0, now, p);
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
            let players = world(
                16,
                &[(1, HUMAN_M, 0.0, 0.0, ZONE), (2, HUMAN_F, 5.0, 3.0, ZONE)],
            );
            let mut live = Boomboxes::default();
            press(&mut live, &row(JINGLE), 600, 1_000, &players);
            let both = |anim| vec![vec![(601, anim), (602, anim)]];
            let first = live.tick(1_000, &players);
            assert_eq!(steps(&first, 1), both(3531));
            assert_eq!(steps(&first, 2), both(3531));
            for (now, anim) in [(5_234, 3532), (9_068, 3533), (15_135, 3534), (25_535, 3531)] {
                assert!(STEP_ENDS.contains(&now));
                // One tick before the boundary nothing moves; at it both get the next step.
                let before = live.tick(now - 1, &players);
                assert!(steps(&before, 1).is_empty(), "tick {}", now - 1);
                let sends = live.tick(now, &players);
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
            let sends = live.tick(10, &players);
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
            let sends = live.tick(20, &players);
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
            live.tick(0, &players);
            walk(&mut players, 1, 40.0);
            let sends = live.tick(50, &players);
            assert_eq!(idles(&sends, 1), [601]);
            assert_eq!(idles(&sends, 2), [601], "the viewer who knows them");
            assert!(steps(&sends, 1).is_empty());
            assert!(live.tick(100, &players).is_empty(), "once, then nothing");
        }

        #[test]
        fn mounting_stops_the_dance_and_stepping_off_rejoins_it_mid_step() {
            let mut players = world(
                16,
                &[(1, HUMAN_M, 0.0, 0.0, ZONE), (2, HUMAN_M, 4.0, 0.0, ZONE)],
            );
            let mut live = Boomboxes::default();
            press(&mut live, &row(JINGLE), 600, 1_000, &players);
            live.tick(1_000, &players);
            players.set_mount(SessionId(1), ride(7));
            let sends = live.tick(1_050, &players);
            assert_eq!((idles(&sends, 1), idles(&sends, 2)), (vec![601], vec![601]));
            assert!(live.tick(1_100, &players).is_empty());
            // Off the ride the dancer joins at the step now playing, as a single entry.
            players.set_mount(SessionId(1), MountTail::NONE);
            let sends = live.tick(1_150, &players);
            assert_eq!(steps(&sends, 1), [[(601, 3531)]]);
            assert_eq!(steps(&sends, 2), [[(601, 3531)]]);
        }

        #[test]
        fn expiry_sends_idle_to_every_dancer_and_then_nothing() {
            let players = world(
                16,
                &[(1, HUMAN_M, 0.0, 0.0, ZONE), (2, HUMAN_F, 4.0, 0.0, ZONE)],
            );
            let mut live = Boomboxes::default();
            press(&mut live, &row(JINGLE), 600, 0, &players);
            live.tick(0, &players);
            let ended = live.tick(180_000, &players);
            assert_eq!(idles(&ended, 1), [601, 602]);
            assert_eq!(idles(&ended, 2), [601, 602]);
            assert!(steps(&ended, 1).is_empty());
            assert!(live.tick(180_050, &players).is_empty());
        }

        #[test]
        fn a_replaced_box_lets_its_dancers_go() {
            let players = world(16, &[(1, HUMAN_M, 0.0, 0.0, ZONE)]);
            let mut live = Boomboxes::default();
            press(&mut live, &row(JINGLE), 600, 0, &players);
            live.tick(0, &players);
            // The owner presses again 500 m away: the first box ends under the dancer.
            let again = [500.0, 1.0, 0.0];
            live.spawn(&row(JINGLE), 600, ZONE, again, 0.0, 100, &players);
            let sends = live.tick(150, &players);
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
            let sends = live.tick(10, &players);
            assert_eq!(steps(&sends, 1), [[(601, 3551)]], "listed by one box only");
            // Equal priority: the nearer wins, whichever was placed first.
            loud.priority = 0;
            let mut live = Boomboxes::default();
            press(&mut live, &loud, 601, 0, &players);
            press(&mut live, &quiet, 600, 0, &players);
            let sends = live.tick(10, &players);
            assert_eq!(steps(&sends, 1), [[(601, 3531)]]);
            // Walking to the loud box's side hands the dancer over with no idle between.
            walk(&mut players, 1, -10.0);
            let sends = live.tick(20, &players);
            assert!(idles(&sends, 1).is_empty());
            assert_eq!(steps(&sends, 1), [[(601, 3551)]]);
        }

        #[test]
        fn a_viewer_is_told_only_about_the_dancers_it_knows() {
            // Each session sees only its nearest neighbour.
            let players = world(
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
            let sends = live.tick(0, &players);
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
            let players = world(200, &who);
            let mut live = Boomboxes::default();
            press(&mut live, &row(JINGLE), 600, 0, &players);
            let sends = live.tick(0, &players);
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
            live.tick(1_000, &players);
            let _ = players.leave(SessionId(1));
            let sends = live.tick(1_050, &players);
            assert!(sends.is_empty(), "the leaver's viewers were told it left");
            // The box and its remaining dancer carry on at the next step.
            let sends = live.tick(STEP_ENDS[0], &players);
            assert_eq!(steps(&sends, 2), [[(602, 3532)]]);
            // A reconnect is a new session with no dance: out of range it hears nothing, and
            // walking in makes it a joiner at the step now playing.
            join(&mut players, 11, HUMAN_M, [900.0, 1.0, 0.0], ZONE);
            assert!(live.tick(STEP_ENDS[0] + 50, &players).is_empty());
            walk(&mut players, 11, 3.0);
            let sends = live.tick(STEP_ENDS[0] + 100, &players);
            assert_eq!(steps(&sends, 11), [[(611, 3532)]]);
        }

        #[test]
        fn other_kinds_and_empty_worlds_send_nothing() {
            let players = world(16, &[(1, HUMAN_M, 0.0, 0.0, ZONE)]);
            let mut live = Boomboxes::default();
            assert!(live.tick(0, &players).is_empty(), "no live box");
            // Realms Roll transforms its dancers; only a synchronized or independent row dances.
            press(&mut live, &row(1180), 1180, 0, &players);
            let sends = live.tick(10, &players);
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
                let players = world(16, &[(1, HUMAN_M, 0.0, 0.0, ZONE)]);
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
                    let sends = live.tick(now, &players);
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
                let players = world(
                    16,
                    &[(1, HUMAN_M, 0.0, 0.0, ZONE), (2, HUMAN_M, 4.0, 0.0, ZONE)],
                );
                let mut live = seeded(1);
                press(&mut live, &row(RAINBOW), 600, 0, &players);
                let sends = live.tick(0, &players);
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
                let players = world(16, &[(1, HUMAN_M, 0.0, 0.0, ZONE)]);
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
                    let entries = rolled(&live.tick(now, &players), 1);
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
                let sends = live.tick(0, &players);
                assert!(
                    steps(&sends, 1).is_empty(),
                    "the shown model has no table: no 35/63"
                );
                assert_eq!(bases(&sends, 1), [(601, 3_550)]);
            }

            #[test]
            fn a_replaced_independent_box_lets_its_dancers_go() {
                let players = world(
                    16,
                    &[(1, HUMAN_M, 0.0, 0.0, ZONE), (2, HUMAN_M, 4.0, 0.0, ZONE)],
                );
                let mut live = seeded(6);
                press(&mut live, &row(RAINBOW), 600, 0, &players);
                live.tick(0, &players);
                // The owner presses again 500 m away: the first box ends under both dancers, who
                // get the idle and no further clip.
                let again = [500.0, 1.0, 0.0];
                live.spawn(&row(RAINBOW), 600, ZONE, again, 0.0, 100, &players);
                let sends = live.tick(150, &players);
                assert_eq!(idles(&sends, 1), [601, 602]);
                assert_eq!(idles(&sends, 2), [601, 602]);
                assert!(steps(&sends, 1).is_empty());
            }

            #[test]
            fn the_randomizer_song_is_always_one_of_its_six() {
                let players = world(16, &[(1, HUMAN_M, 0.0, 0.0, ZONE)]);
                let rainbow = row(RAINBOW);
                let songs = rainbow.song_effect_ids.clone();
                assert_eq!(songs.len(), 6);
                let mut live = seeded(11);
                let mut seen = BTreeSet::new();
                for n in 0..32 {
                    let sends =
                        live.spawn(&rainbow, 600 + n, ZONE, [0.0, 1.0, 0.0], 0.0, n, &players);
                    let song = to(&sends, 1)
                        .into_iter()
                        .find(|p| p.starts_with(&[35, 0, 41, 0]))
                        .expect("the box has a song");
                    let effect = i32::from_le_bytes(song[16..20].try_into().unwrap());
                    assert!(songs.contains(&effect), "spawn {n}: {effect}");
                    seen.insert(effect);
                }
                assert!(seen.len() > 1, "the pick varies: {seen:?}");
            }

            #[test]
            fn a_model_without_a_table_stands_on_the_row_group_as_a_base_animation() {
                let players = world(16, &[(1, TRANSFORMED, 0.0, 0.0, ZONE)]);
                let mut live = seeded(5);
                press(&mut live, &row(RAINBOW), 600, 0, &players);
                let sends = live.tick(0, &players);
                assert!(steps(&sends, 1).is_empty(), "no 35/63");
                assert_eq!(bases(&sends, 1), [(601, 3_550)]);
                // The base plays until the box ends: it is sent once, not every tick.
                let again = live.tick(5_000, &players);
                assert!(bases(&again, 1).is_empty() && steps(&again, 1).is_empty());
            }

            #[test]
            fn a_one_clip_table_repeats_it_instead_of_rerolling_forever() {
                let players = world(16, &[(1, HUMAN_M, 0.0, 0.0, ZONE)]);
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
                assert_eq!(rolled(&live.tick(0, &players), 1), [(601, 3_201)]);
                assert!(live.tick(4_399, &players).is_empty(), "still playing");
                assert_eq!(rolled(&live.tick(4_400, &players), 1), [(601, 3_201)]);
            }

            #[test]
            fn a_clip_shorter_than_the_blend_waits_a_second() {
                let players = world(16, &[(1, HUMAN_M, 0.0, 0.0, ZONE)]);
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
                let first = rolled(&live.tick(0, &players), 1);
                assert_eq!(first.len(), 1);
                assert!(live.tick(999, &players).is_empty(), "the 1000 ms floor");
                let second = rolled(&live.tick(1_000, &players), 1);
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
                live.tick(1_000, &players);
                // Walking out of range frees only the leaver, told to them and their viewer.
                walk(&mut players, 1, 40.0);
                let left = live.tick(1_100, &players);
                assert_eq!(idles(&left, 1), [601]);
                assert_eq!(idles(&left, 2), [601], "session 2 knows session 1");
                walk(&mut players, 1, 0.0);
                live.tick(1_200, &players);
                // Mounting frees the rider, then the box's own expiry frees the rest.
                players.set_mount(SessionId(1), ride(7));
                let mounted = live.tick(1_300, &players);
                assert_eq!(idles(&mounted, 1), [601]);
                players.set_mount(SessionId(1), MountTail::NONE);
                live.tick(1_400, &players);
                let ended = live.tick(1_000 + u64::from(rainbow.duration_ms), &players);
                assert_eq!(idles(&ended, 1), [601, 602]);
                assert_eq!(idles(&ended, 2), [601, 602]);
                assert!(steps(&ended, 1).is_empty());
                assert!(
                    live.tick(1_000 + u64::from(rainbow.duration_ms) + 50, &players)
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
                let first = rolled(&live.tick(1_000, &players), 2);
                let own = first
                    .iter()
                    .find(|(guid, _)| *guid == 602)
                    .expect("session 2 sees its own clip");
                let _ = players.leave(SessionId(1));
                assert!(
                    live.tick(1_050, &players).is_empty(),
                    "the leaver's viewers were told it left"
                );
                // The remaining dancer re-rolls on its own clock.
                let sends = live.tick(ends(RAINBOW, 1_000, own.1), &players);
                let second = rolled(&sends, 2);
                assert_eq!(second.len(), 1);
                assert_ne!(second[0].1, own.1);
                // A reconnect is a new session: walking in joins at a fresh clip.
                join(&mut players, 11, HUMAN_M, [900.0, 1.0, 0.0], ZONE);
                assert!(rolled(&live.tick(75_000, &players), 11).is_empty());
                walk(&mut players, 11, 3.0);
                assert_eq!(rolled(&live.tick(75_050, &players), 11).len(), 1);
            }

            #[test]
            fn the_realmshake_and_xvd_tables_are_the_clients_own() {
                // The two other independent rows keep their model tables (ids from their groups).
                assert_eq!(ids(REALMSHAKE, HUMAN_M).len(), 53);
                assert_eq!(ids(RAINBOW, HUMAN_F), ids(RAINBOW, HUMAN_M));
                // A model with a table but a different one still never rolls outside its own.
                let players = world(16, &[(1, HUMAN_F, 0.0, 0.0, ZONE)]);
                let table = ids(RAINBOW, HUMAN_F);
                let mut live = seeded(12);
                press(&mut live, &row(RAINBOW), 600, 0, &players);
                assert!(table.contains(&rolled(&live.tick(0, &players), 1)[0].1));
            }
        }
    }
}
