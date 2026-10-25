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
//!
//! # Reconstructed (R)
//!
//! * Dances, clip lengths, songs, priority and transforms come from Sanctuary's measured rows;
//!   duration, range, spawn offset and recast are Sanctuary's defaults. All are tunable here.
//! * The box stands `spawn_offset_m` to the owner's left (Sanctuary: the player's rotation applied
//!   to -X; here `(-cos h, 0, sin h)` for the repo's facing `(sin h, 0, cos h)`), at the owner's
//!   height, facing the way the owner faces. Not interactable, no nameplate, no interact range.
//! * The song is the row's first effect, attached by 35/41 with the owner as source. The pick
//!   among several songs belongs to the later random-dances slice.
//! * A viewer who leaves the box's range is sent the removal (35/3) and sees the box again, with
//!   its song, on re-entry. One box per owner: a second press before the first ends replaces it
//!   (the default recast equals the duration, so this only happens if the two are tuned apart).
//! * A box outlives its owner's session until it expires.

use crate::{
    abilities::effect_packets,
    encounter::SharedFieldState,
    players::{PlayerRegistry, SessionId, ZoneInstance},
    startup_world::{self, NpcSpawn},
    world,
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, Read},
    path::Path,
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

/// One boombox. Ids of 0 mean "none" for `standing_animation_id` and `transform_model_id`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Boombox {
    pub item_id: i32,
    pub model_id: i32,
    pub kind: Kind,
    pub dance_sequence: Vec<i32>,
    /// Player model id -> one clip length per dance step. Empty for `Independent` boxes, whose
    /// per-dancer random clips are a later slice.
    pub clip_ms: BTreeMap<i32, Vec<u32>>,
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
    /// Sanity bounds, not tuned values.
    fn valid(&self) -> bool {
        let ms = |v: u32| (1..=MAX_MS).contains(&v);
        let transform = self.kind == Kind::Transform;
        self.item_id > 0
            && self.model_id > 0
            && !self.dance_sequence.is_empty()
            && self.dance_sequence.iter().all(|id| *id > 0)
            && self.song_effect_ids.iter().all(|id| *id > 0)
            && self.standing_animation_id >= 0
            && (self.kind == Kind::Independent) == self.clip_ms.is_empty()
            && self.clip_ms.iter().all(|(model, clips)| {
                *model > 0
                    && clips.len() == self.dance_sequence.len()
                    && clips.iter().all(|v| ms(*v))
            })
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
}

impl Placed {
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
#[derive(Clone, Debug, Default)]
pub struct Boomboxes {
    live: Vec<Placed>,
    next: u32,
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
            self.live.remove(old).end(&mut sends);
        }
        let mut placed = Placed {
            guid,
            owner,
            zone,
            position,
            tag: guid as u32 as i32,
            song: row.song_effect_ids.first().copied(),
            add,
            ends_ms: now_ms.saturating_add(u64::from(row.duration_ms)),
            seen_by: BTreeSet::new(),
        };
        for session in players.nearby_sessions(zone, position) {
            placed.show(session, true, &mut sends);
        }
        self.live.push(placed);
        sends
    }

    /// Once per tick: end boxes whose time is up, show the rest to sessions that came into range
    /// and take it back from those that left. No boxes means no work.
    pub fn tick(&mut self, now_ms: u64, players: &PlayerRegistry) -> Sends {
        let mut sends = Sends::new();
        if self.live.is_empty() {
            return sends;
        }
        let (ended, live): (Vec<_>, Vec<_>) = std::mem::take(&mut self.live)
            .into_iter()
            .partition(|b| now_ms >= b.ends_ms);
        self.live = live;
        for placed in ended {
            placed.end(&mut sends);
        }
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
            for session in near {
                if !placed.seen_by.contains(&session) {
                    placed.show(session, false, &mut sends);
                }
            }
        }
        sends
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
            let mut song = vec![35, 0, 41, 0];
            song.extend(guid.to_le_bytes());
            song.extend((guid as u32 as i32).to_le_bytes());
            song.extend(16_448_i32.to_le_bytes());
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
        assert!(live.tick(1_000 + 179_999, &players).is_empty());
        let ended = live.tick(1_000 + 180_000, &players);
        let guid = GUID_TAG | 1;
        for session in [1, 2] {
            let packets = to(&ended, session);
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
        let packets = to(&shown, 3);
        assert_eq!(packets.len(), 2, "AddNpc and song, no poof");
        assert!(packets[0].starts_with(&[35, 0, 2, 0]) && packets[1].starts_with(&[35, 0, 41, 0]));
        assert!(live.tick(100, &players).is_empty(), "exactly once");
        // Walking out sends the removal; coming back shows the box again.
        players
            .update_pose_with_state(SessionId(3), ZONE, [900.0, 1.0, 0.0], 0.0, 0, 0)
            .unwrap();
        let gone = live.tick(150, &players);
        assert_eq!(to(&gone, 3), [SharedFieldState::removal(guid).as_slice()]);
        arrive(&mut players);
        assert_eq!(to(&live.tick(200, &players), 3).len(), 2);
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
}
