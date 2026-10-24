//! Server-authoritative fishing service for native minigame type 21.
//!
//! Recovered layouts: fishing sub 2/5/6/7/15 client requests; sub 1 update,
//! sub 3 registration, sub 4 fish list, sub 8 bobber, sub 10 fish state,
//! sub 11 school, sub 12 despawn, sub 14 result and sub 18 fish run. Sources are the readers
//! and writers named in `notes/Build Guides/Gameplay/Fishing.md`, notably
//! `00b6a400`, `00b6a380`, `00b6a460`, `00b6a4d0`, `00b65880`, `00b67240`,
//! `00b673b0`, `00b66a00`, `00b66a80`, `00b69130`, `00b68810` and `00b66be0`.
//! Result presentation is read by `00cd0150`.
//!
//! The content file marks reconstructed values (R). No client capture proves
//! registration field meanings/values, school layouts, timing, catch odds,
//! fish type ids, icons, lure-hint ids, sizes/weights/rarity, or inventory
//! definition bindings. The client sends no bite, hook, tension or catch
//! outcome. Only this service selects and awards a catch.
//!
//! Payout rows and the optional paid-catch cap are operator content in
//! `fishing_spots.json`; [`FishingService::begin`] keeps the low-level
//! caller-owned payout, while the ordinary [`FishingService::enter`] path uses
//! both.
//!
//! # Ordinary entry (W160)
//!
//! [`FishingSpots`] (`fishing_spots.json`) lists the guide NPCs whose menu offers the trip, the
//! area each one takes the player to, and the inventory item behind every fish size. The caller
//! (the gameplay NPC menu) asks [`FishingService::enter`] to admit the player; every later
//! packet goes through [`FishingService::route`], which owns the whole round: lifecycle,
//! opcode 138, the server-owned payout and the player status flag that switches the native
//! fishing processor on and off.

use crate::{
    fishing::{
        self, ClientRequest, Error, FishRun, Handle, Outcome, Phase, ResultBody, RunFish, Sub, Vec4,
    },
    inventory::Inventory,
    minigame::{MinigameRuns, RunMode, RunSpec, RunState},
    profiles::ProfileState,
    quest_engine::QuestEvent,
    rewards::{ItemGrant, RewardDisplay, RewardGrant, RewardState, apply_grant},
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Read},
    path::Path,
    sync::LazyLock,
};

const CONTENT_JSON: &str = include_str!("../data/content/fishing.json");

pub const MAX_SESSIONS: usize = 4;

/// Player status flags (35/20). Bit 22 of the character status word turns the native fishing
/// processor on: `BaseClient::DispatchBasePlayerUpdate` (`0092f460`) shifts the word by 0x16 and
/// calls `FishingProcessor::SetActive` (`00b6bb90`) when it differs from the processor flag. The
/// baseline 1 is the value the kart exit restores (R).
const STATUS_FISHING: u32 = 0x40_0001;
const STATUS_NORMAL: u32 = 1;
/// The Fisherman job's profile id (`jobs.json`). Catches pay its XP when the job is unlocked.
pub const FISHERMAN_PROFILE: i32 = 137;
/// (R) How far from the spot a fisherman may wander before the round ends. Independent of the
/// cast reach (`MaxCastDistance`, player word 2 in `fishing.json`).
const LEASH_M: f32 = 50.0;
const SPOTS_FILE: &str = "fishing_spots.json";
const MAX_SPOTS_FILE: usize = 64 * 1024;
const MAX_SPOTS: usize = 16;
const MAX_CATCH_XP: u32 = 1_000;
/// Sanity ceilings for the optional payout/cap content (R). They are not gameplay values: the
/// shipped cap is disabled, and these only reject an operator file no live catch rate can reach.
const MAX_PAYOUT_COINS: u32 = 1_000_000;
const MAX_PAID_CATCHES_PER_SESSION: u32 = 10_000;
const MAX_PAID_CATCHES_PER_HOUR: u32 = 3_600;
/// (R) The rolling output-cap window exposed to the gateway's character ledger. Lead-owned
/// exception: the hub's `commit_outcome` sweep uses it while `catch_cap.window` selects the
/// policy and `fishing_spots.json` documents the 3,600-second duration; a data-driven window
/// needs a lead-approved hub change.
pub const HOUR_WINDOW_MS: u64 = 60 * 60 * 1_000;
/// (R) Player-readable line when the optional cap refuses to pay a catch.
const CATCH_CAP_NOTICE: &str = "The fishing reward limit has been reached, so the fish got away.";
/// The most new inventory stacks fishing can ever add: fish stack without limit, so a bag gains
/// at most one stack per fish definition (40 species x 4 sizes). The payout bag floor is the
/// player's current stacks plus this, so a catch never fails for want of a slot (R: the bag limit
/// itself is unrecovered, and this keeps a large existing inventory from blocking every catch).
const MAX_FISH_STACKS: usize = 160;

static CONTENT: LazyLock<Content> = LazyLock::new(|| {
    let content: Content =
        serde_json::from_str(CONTENT_JSON).expect("bundled fishing content must parse");
    assert!(
        content.models.valid(),
        "fishing.json models: ids must be positive and motion within 0..=100"
    );
    assert!(
        content.registration.valid(),
        "fishing.json registration: player word 6 and zone word 3 must be whole numbers"
    );
    content
});

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Change {
    /// Inner application packets, ready for the gateway's outer tunnel.
    pub replies: Vec<Vec<u8>>,
    /// Authoritative objective events for the caller's QuestEngine.
    pub quest_events: Vec<QuestEvent>,
    pub changed: bool,
    /// A catch was paid: the caller adopts [`FishingService::reward_state`] (W160).
    pub paid: bool,
    /// Player-readable lines the caller sends as chat (W160).
    pub notices: Vec<&'static str>,
}

#[derive(Clone)]
struct Tracked {
    runs: MinigameRuns,
    instance_id: i32,
    /// Stable minigame catalogue id used by quest completion events.
    game_id: i32,
    area_key: String,
    fisherman_level: u32,
    anchor: Vec4,
    /// The 138/3 zone block: the spot's scene values ([`Scene::zone_words`]) for `enter`, the
    /// bundled `fishing.json` words for `begin`.
    zone_words: [f32; 5],
    /// Zone of the spot; `None` for sessions started by `begin` (no zone supervision).
    zone: Option<u32>,
    /// Per-grade payout rows for this round (R, `fishing_spots.json`); `None` for `begin`,
    /// which keeps the low-level entry's caller-owned payout.
    payout: Option<PayoutTable>,
    /// Optional cap for this round (R, `fishing_spots.json`); `None` for `begin`.
    cap: Option<CatchCap>,
    rewards: RewardState,
    session: fishing::Session,
    attempt: Option<Attempt>,
    school_ids: Vec<u32>,
    lure_key: Option<String>,
    cast_sequence: u32,
    playing: bool,
}

/// Paid catches for one character. The service is already scoped to one player; the map key
/// keeps the accounting correct if several rounds are hosted together.
#[derive(Clone, Default)]
struct PaidCatches {
    /// Paid catches across every round since the login session began.
    session: u32,
    /// Paid-catch timestamps for the rolling hour window, oldest first. Unlike `session`, this
    /// survives a reconnect and is restored with the character's server-side cap ledger.
    hour: Vec<u64>,
}

#[derive(Clone)]
struct Attempt {
    stage: AttemptStage,
    cast_at_ms: u64,
    bite_at_ms: u64,
    hook_at_ms: u64,
    expires_at_ms: u64,
    fish_key: u32,
    species_key: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AttemptStage {
    Casting,
    Nibbling,
    Hooked,
}

#[derive(Clone, Default)]
pub struct FishingService {
    sessions: BTreeMap<u64, Tracked>,
    item_definitions: BTreeMap<(String, u8), i32>,
    next_instance_id: i32,
    /// Cap accounting keyed by character (R). Reconnect clears `session` only; the caller
    /// restores `hour` into a fresh service.
    paid: BTreeMap<u64, PaidCatches>,
}

impl FishingService {
    pub fn new() -> Self {
        Self {
            next_instance_id: 1,
            ..Self::default()
        }
    }

    pub fn content_json() -> &'static str {
        CONTENT_JSON
    }

    pub fn area_count() -> usize {
        CONTENT.areas.len()
    }

    /// Check the bundled tables' schema, client source tags and cross-links.
    pub fn validate_content() -> bool {
        CONTENT.schema_version == 1
            && CONTENT.minigame_type_id == 21
            && CONTENT
                .source_tags
                .get("minigame_type_id")
                .is_some_and(|tag| tag.contains("client"))
            && CONTENT.timing.bite_delay_ms > 0
            && CONTENT.timing.nibble_ms > 0
            && CONTENT.timing.reel_window_ms > 0
            && CONTENT.timing.source.contains("R:")
            && CONTENT.registration.source.contains("client")
            && CONTENT.registration.valid()
            && CONTENT.models.valid()
            && CONTENT.sizes.len() == 4
            && CONTENT.sizes.iter().enumerate().all(|(i, size)| {
                size.grade == i as u8 + 1 && !size.name.is_empty() && !size.source.is_empty()
            })
            && CONTENT.areas.len() == 6
            && CONTENT.areas.iter().all(|area| {
                !area.name.is_empty()
                    && !area.source.is_empty()
                    && !area.fish.is_empty()
                    && area.fish.iter().all(|key| species(key).is_some())
            })
            && CONTENT.species.iter().all(|fish| {
                !fish.name.is_empty()
                    && fish.source_tags.values().all(|tag| !tag.is_empty())
                    && fish.size_weights.iter().any(|weight| *weight > 0)
            })
            && CONTENT.lures.iter().all(|lure| {
                !lure.name.is_empty()
                    && !lure.source.is_empty()
                    && lure.fish.iter().all(|key| species(key).is_some())
            })
    }

    /// Start the shared TYPE 21 encounter. `game_id` and `encounter_id` are
    /// supplied by the trusted area/portal integration; their native pairing
    /// is unresolved (R). The returned 39/16 offer and its 41/107 ready must be
    /// sent before client start handling. Call `mark_started` after the
    /// client's 39/5 is answered (41/2, 39/17).
    pub fn begin(
        &mut self,
        guid: u64,
        game_id: i32,
        encounter_id: i32,
        area_key: &str,
        fisherman_level: u32,
        anchor: Vec4,
        rewards: RewardState,
    ) -> Result<Change, Error> {
        let spec = RunSpec::empty(
            game_id,
            CONTENT.minigame_type_id,
            RunMode::Encounter {
                encounter_id,
                player_guid: guid,
            },
        );
        self.begin_spec(
            guid,
            spec,
            area_key,
            fisherman_level,
            anchor,
            CONTENT.registration.zone_words,
            None,
            None,
            None,
            rewards,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn begin_spec(
        &mut self,
        guid: u64,
        spec: RunSpec,
        area_key: &str,
        fisherman_level: u32,
        anchor: Vec4,
        zone_words: [f32; 5],
        zone: Option<u32>,
        payout: Option<PayoutTable>,
        cap: Option<CatchCap>,
        rewards: RewardState,
    ) -> Result<Change, Error> {
        if guid == 0
            || fisherman_level == 0
            || !finite_vec(anchor)
            || area(area_key).is_none()
            || self.sessions.contains_key(&guid)
            || self.sessions.len() >= MAX_SESSIONS
        {
            return Err(if self.sessions.len() >= MAX_SESSIONS {
                Error::AtCapacity
            } else {
                Error::InvalidValue
            });
        }
        let id = self.next_instance_id.max(1);
        let game_id = spec.game_id;
        let mut runs = MinigameRuns::new(id).map_err(|_| Error::InvalidValue)?;
        let offered = runs.offer(spec).map_err(|_| Error::InvalidValue)?;
        if !offered.accepted {
            return Err(Error::OutOfOrder);
        }
        self.next_instance_id = id.checked_add(1).ok_or(Error::InvalidValue)?;
        self.sessions.insert(
            guid,
            Tracked {
                runs,
                instance_id: id,
                game_id,
                area_key: area_key.to_owned(),
                fisherman_level,
                anchor,
                zone_words,
                zone,
                payout,
                cap,
                rewards,
                session: fishing::Session::new(),
                attempt: None,
                school_ids: Vec::new(),
                lure_key: None,
                cast_sequence: 0,
                playing: false,
            },
        );
        Ok(Change {
            replies: offered.replies,
            changed: true,
            ..Change::default()
        })
    }

    pub fn instance_id(&self, guid: u64) -> Option<i32> {
        self.sessions.get(&guid).map(|s| s.instance_id)
    }

    pub fn run_state(&self, guid: u64) -> Option<RunState> {
        let tracked = self.sessions.get(&guid)?;
        tracked.runs.state(tracked.instance_id)
    }

    /// Handle 39/5, 39/6 and 41/109 lifecycle messages through MinigameRuns.
    /// Cancellation/leave removes schools and the session without granting.
    pub fn handle_minigame_lifecycle(&mut self, guid: u64, packet: &[u8]) -> Result<Change, Error> {
        let tracked = self.sessions.get_mut(&guid).ok_or(Error::OutOfOrder)?;
        let outcome = tracked
            .runs
            .handle_client(packet)
            .map_err(|_| Error::InvalidValue)?;
        let terminal = matches!(outcome.state, Some(RunState::Cancelled | RunState::Closed));
        let mut change = Change {
            replies: outcome.replies,
            changed: outcome.changed,
            ..Change::default()
        };
        if terminal {
            change
                .replies
                .extend(tracked.school_ids.iter().copied().map(despawn_school));
            self.sessions.remove(&guid);
        }
        Ok(change)
    }

    /// Complete the encounter's server-side readiness step after startup.
    pub fn mark_started(&mut self, guid: u64) -> bool {
        let Some(tracked) = self.sessions.get_mut(&guid) else {
            return false;
        };
        let outcome = tracked.runs.started(tracked.instance_id);
        tracked.playing = outcome.state == Some(RunState::Playing);
        outcome.changed
    }

    /// Convenience leave path matching encounter close (41/109).
    pub fn leave(&mut self, guid: u64) -> Result<Change, Error> {
        let id = self.instance_id(guid).ok_or(Error::OutOfOrder)?;
        let mut packet = vec![41, 0, 109, 0];
        packet.extend(0i32.to_le_bytes());
        packet.extend(id.to_le_bytes());
        self.handle_minigame_lifecycle(guid, &packet)
    }

    /// Disconnect has no socket on which to send cleanup packets. Drop all
    /// ephemeral state and explicitly produce no rewards or quest events.
    pub fn disconnect(&mut self, guid: u64) -> Change {
        self.clear_session_count(guid);
        let Some(mut tracked) = self.sessions.remove(&guid) else {
            return Change::default();
        };
        let _ = tracked.runs.disconnect();
        tracked.attempt = None;
        tracked.session.reset();
        Change {
            // The disconnected socket cannot receive these; the caller can
            // use them to remove the player's schools from nearby clients.
            replies: tracked.school_ids.into_iter().map(despawn_school).collect(),
            changed: true,
            ..Change::default()
        }
    }

    pub fn set_anchor(&mut self, guid: u64, anchor: Vec4) -> Result<(), Error> {
        if !finite_vec(anchor) {
            return Err(Error::InvalidValue);
        }
        self.sessions
            .get_mut(&guid)
            .ok_or(Error::OutOfOrder)?
            .anchor = anchor;
        Ok(())
    }

    /// Called by trusted inventory/action-bar code after validating possession.
    pub fn set_lure(&mut self, guid: u64, lure_key: Option<&str>) -> Result<(), Error> {
        let tracked = self.sessions.get_mut(&guid).ok_or(Error::OutOfOrder)?;
        if let Some(key) = lure_key {
            if !CONTENT.lures.iter().any(|l| l.key == key) {
                return Err(Error::InvalidValue);
            }
            tracked.lure_key = Some(key.to_owned());
        } else {
            tracked.lure_key = None;
        }
        Ok(())
    }

    /// Refresh the client's Fish Finder catchable flags after a level change.
    pub fn update_fisherman_level(
        &mut self,
        guid: u64,
        fisherman_level: u32,
    ) -> Result<Change, Error> {
        if fisherman_level == 0 {
            return Err(Error::InvalidValue);
        }
        let tracked = self.sessions.get_mut(&guid).ok_or(Error::OutOfOrder)?;
        if !tracked.playing || !tracked.session.registered() {
            return Err(Error::OutOfOrder);
        }
        if tracked.fisherman_level == fisherman_level {
            return Ok(Change::default());
        }
        tracked.fisherman_level = fisherman_level;
        Ok(Change {
            replies: vec![fish_info_reply(tracked)?],
            changed: true,
            ..Change::default()
        })
    }

    /// Bind a recovered item definition when the content/import layer supplies
    /// it. Fishing-to-inventory ids are not currently mapped by client evidence.
    pub fn bind_item_definition(
        &mut self,
        species_key: &str,
        size_grade: u8,
        definition_id: i32,
    ) -> Result<(), Error> {
        if species(species_key).is_none() || !(1..=4).contains(&size_grade) || definition_id <= 0 {
            return Err(Error::InvalidValue);
        }
        self.item_definitions
            .insert((species_key.to_owned(), size_grade), definition_id);
        Ok(())
    }

    pub fn reward_state(&self, guid: u64) -> Option<&RewardState> {
        self.sessions.get(&guid).map(|s| &s.rewards)
    }

    pub fn phase(&self, guid: u64) -> Option<Phase> {
        self.sessions.get(&guid).map(|s| s.session.phase())
    }

    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    pub fn reset(&mut self, guid: u64) -> bool {
        self.clear_session_count(guid);
        self.sessions.remove(&guid).is_some()
    }

    pub fn reset_all(&mut self) -> usize {
        let count = self.sessions.len();
        self.sessions.clear();
        for paid in self.paid.values_mut() {
            paid.session = 0;
        }
        self.paid.retain(|_, paid| !paid.hour.is_empty());
        count
    }

    /// The character's paid catches inside the rolling hour, oldest first. The gateway keeps
    /// this beside the character and passes it back to [`Self::restore_hour_history`] on
    /// reconnect; it deliberately stays out of the saved gameplay schema.
    pub fn hour_history(&self, guid: u64) -> Vec<u64> {
        self.paid
            .get(&guid)
            .map(|paid| paid.hour.clone())
            .unwrap_or_default()
    }

    /// Restore a character's rolling-hour history into a fresh service. The caller supplies
    /// server-owned timestamps; reject a malformed or oversized ledger rather than letting it
    /// strengthen or evade the configured cap.
    pub fn restore_hour_history(&mut self, guid: u64, mut history: Vec<u64>) -> Result<(), Error> {
        if guid == 0
            || history.len() > MAX_PAID_CATCHES_PER_HOUR as usize
            || !history.windows(2).all(|pair| pair[0] <= pair[1])
        {
            return Err(Error::InvalidValue);
        }
        history.dedup();
        if history.is_empty() {
            if let Some(paid) = self.paid.get_mut(&guid) {
                paid.hour.clear();
                if paid.session == 0 {
                    self.paid.remove(&guid);
                }
            }
        } else {
            self.paid.entry(guid).or_default().hour = history;
        }
        Ok(())
    }

    fn clear_session_count(&mut self, guid: u64) {
        if let Some(paid) = self.paid.get_mut(&guid) {
            paid.session = 0;
            if paid.hour.is_empty() {
                self.paid.remove(&guid);
            }
        }
    }

    /// Ordinary entry (W160): admit the player at a listed guide and offer the round. Staged:
    /// a refusal changes nothing, and the 39/16 offer and its 41/107 ready in the returned
    /// [`Change`] are the only effect of success. `busy` is true while another activity runs;
    /// range and "this exact option was offered" are already proven by the NPC menu service.
    #[allow(clippy::too_many_arguments)]
    pub fn enter(
        &mut self,
        spots: &FishingSpots,
        npc_id: u64,
        guid: u64,
        zone: u32,
        position: [f32; 3],
        busy: bool,
        fisherman_level: u32,
        rewards: RewardState,
    ) -> Result<Change, EntryDenied> {
        let spot = spots.spot(npc_id).ok_or(EntryDenied::NotAGate)?;
        if self.sessions.contains_key(&guid) {
            return Err(EntryDenied::AlreadyFishing);
        }
        if busy {
            return Err(EntryDenied::Busy);
        }
        if self.sessions.len() >= MAX_SESSIONS {
            return Err(EntryDenied::Full);
        }
        // (R) The encounter id is the area's MiniGameData id: its native pairing is unresolved.
        let mut spec = RunSpec::empty(
            spot.game_id,
            CONTENT.minigame_type_id,
            RunMode::Encounter {
                encounter_id: spot.game_id,
                player_guid: guid,
            },
        );
        spec.name_id = spot.name_id;
        spec.description_id = spot.description_id;
        spec.icon_id = spot.icon_id;
        let anchor = Vec4 {
            x: position[0],
            y: position[1],
            z: position[2],
            w: 1.0,
        };
        let change = self
            .begin_spec(
                guid,
                spec,
                &spot.area,
                fisherman_level,
                anchor,
                spot.scene.zone_words(),
                Some(zone),
                Some(spots.payout.clone()),
                Some(spots.catch_cap.clone()),
                rewards,
            )
            .map_err(|error| match error {
                Error::AtCapacity => EntryDenied::Full,
                _ => EntryDenied::Unavailable,
            })?;
        for (species, sizes) in &spots.items {
            for (index, definition) in sizes.iter().enumerate() {
                if *definition > 0 {
                    // Idempotent; the file validated every key and id.
                    let _ = self.bind_item_definition(species, index as u8 + 1, *definition);
                }
            }
        }
        Ok(change)
    }

    /// Route one in-world packet for a fishing player: the minigame lifecycle and opcode 138.
    /// `Ok(None)` means it is not a fishing packet (or there is no round), so the caller carries
    /// on with its other handlers. The player's live inventory and profiles are used for the
    /// payout, not the copy taken at entry; a paid [`Change`] sets `paid` and the caller adopts
    /// [`Self::reward_state`]. The status flag follows the round: on at start, off at any end.
    pub fn route(
        &mut self,
        guid: u64,
        inner: &[u8],
        now_ms: u64,
        inventory: &Inventory,
        profiles: &ProfileState,
    ) -> Result<Option<Change>, Error> {
        let Some(tracked) = self.sessions.get_mut(&guid) else {
            return Ok(None);
        };
        if inner.starts_with(&fishing::OPCODE.to_le_bytes()) {
            tracked.rewards.inventory = inventory.clone();
            tracked.rewards.profiles = profiles.clone();
            tracked.rewards.bag_capacity = inventory.state().items.len() + MAX_FISH_STACKS;
            return self.handle(inner, guid, now_ms).map(Some);
        }
        if !is_lifecycle(inner) {
            return Ok(None);
        }
        let mut change = self.handle_minigame_lifecycle(guid, inner)?;
        let starting = self
            .sessions
            .get(&guid)
            .map(|t| !t.playing && t.runs.state(t.instance_id) == Some(RunState::Starting));
        match starting {
            None => change.replies.push(player_status(guid, STATUS_NORMAL)),
            Some(true) => {
                self.mark_started(guid);
                change.replies.push(player_status(guid, STATUS_FISHING));
            }
            Some(false) => {}
        }
        Ok(Some(change))
    }

    /// Per-tick upkeep for a fishing player: fire the bite timers, and end the round when the
    /// player has left the spot's zone or walked past the leash (R). `None` without a round.
    pub fn supervise(
        &mut self,
        guid: u64,
        zone: u32,
        position: [f32; 3],
        now_ms: u64,
    ) -> Result<Option<Change>, Error> {
        let Some(tracked) = self.sessions.get(&guid) else {
            return Ok(None);
        };
        let away = tracked.zone.is_some_and(|spot_zone| {
            let (dx, dz) = (
                position[0] - tracked.anchor.x,
                position[2] - tracked.anchor.z,
            );
            spot_zone != zone
                || !(dx * dx + dz * dz).is_finite()
                || dx * dx + dz * dz > LEASH_M * LEASH_M
        });
        if away {
            return Ok(Some(self.end(guid)));
        }
        self.tick(guid, now_ms).map(Some)
    }

    /// End the round from the server side (the player left, or the slot is needed). The same
    /// ordered cancel a 39/6 produces: state removal, school despawns and the status flag off.
    /// Nothing is paid and the attempt in flight is dropped.
    pub fn end(&mut self, guid: u64) -> Change {
        let Some(id) = self.instance_id(guid) else {
            return Change::default();
        };
        let mut cancel = vec![39, 0, 6];
        cancel.extend(id.to_le_bytes());
        cancel.extend((-1i32).to_le_bytes());
        cancel.extend((-1i32).to_le_bytes());
        let mut change = self
            .handle_minigame_lifecycle(guid, &cancel)
            .unwrap_or_default();
        self.sessions.remove(&guid);
        change.replies.push(player_status(guid, STATUS_NORMAL));
        change.changed = true;
        change
    }

    /// Decode and handle one opcode 138 request for the server-owned player.
    pub fn handle(&mut self, inner: &[u8], guid: u64, now_ms: u64) -> Result<Change, Error> {
        if inner.len() > fishing::MAX_BODY + 4 {
            return Err(Error::Oversize);
        }
        let request = ClientRequest::decode(inner)?;
        let actor = request.actor();

        if let ClientRequest::RegisterPlayer { .. } = request {
            if self.bound_to(actor).is_some_and(|owner| owner != guid) {
                return Err(Error::ForeignActor);
            }
        } else if let Some(owner) = self.bound_to(actor) {
            if owner != guid {
                return Err(Error::ForeignActor);
            }
        }

        let tracked = self.sessions.get_mut(&guid).ok_or(Error::OutOfOrder)?;
        if !tracked.playing || tracked.runs.state(tracked.instance_id) != Some(RunState::Playing) {
            return Err(Error::OutOfOrder);
        }

        if matches!(request, ClientRequest::RegisterPlayer { .. }) {
            let outcome = tracked.session.request(&request)?;
            if outcome == Outcome::Duplicate {
                return Ok(Change::default());
            }
            let content_area = area(&tracked.area_key).ok_or(Error::InvalidValue)?;
            tracked.school_ids = school_ids(guid, content_area.schools.len());
            let mut replies = vec![
                registration_reply(tracked)?,
                update_data_reply(actor, tracked.anchor),
            ];
            replies.extend(
                content_area
                    .schools
                    .iter()
                    .zip(&tracked.school_ids)
                    .map(|(position, id)| school_reply(*id, tracked.anchor, *position)),
            );
            return Ok(Change {
                replies,
                changed: true,
                ..Change::default()
            });
        }

        if tracked.session.actor() != Some(actor) {
            return Err(if tracked.session.actor().is_some() {
                Error::ForeignActor
            } else {
                Error::OutOfOrder
            });
        }

        match &request {
            ClientRequest::Cast { target, .. } => validate_cast(*target, tracked.anchor)?,
            ClientRequest::CastAnim {
                distance, position, ..
            } if !distance.is_finite() || *distance < 0.0 || !finite_vec(*position) => {
                return Err(Error::InvalidValue);
            }
            ClientRequest::ReelIn { flag, .. } if *flag != 1 => {
                return Err(Error::InvalidValue);
            }
            _ => {}
        }

        let mut change = advance(tracked, now_ms)?;
        if matches!(request, ClientRequest::ReelIn { .. })
            && tracked.attempt.is_none()
            && change.changed
        {
            return Ok(change);
        }
        match request {
            ClientRequest::CastAnim { .. } => {
                // Client state 1 sends this before state 2 sends CastRequest.
                // It carries presentation only; the server does not echo it.
                if tracked.attempt.is_none() && tracked.session.phase() != Phase::Idle {
                    return Err(Error::OutOfOrder);
                }
                change.changed = true;
            }
            ClientRequest::Cast { target, .. } => {
                if tracked.attempt.is_some() {
                    return Err(Error::OutOfOrder);
                }
                // Select before any mutation, so a refused cast changes nothing.
                let sequence = tracked.cast_sequence.wrapping_add(1).max(1);
                let selected = select_species(tracked, sequence)?;
                tracked.session.request(&request)?;
                tracked.cast_sequence = sequence;
                let now = now_ms;
                let bite_at = now.saturating_add(CONTENT.timing.bite_delay_ms);
                let hook_at = bite_at.saturating_add(CONTENT.timing.nibble_ms);
                let expires_at = hook_at.saturating_add(CONTENT.timing.reel_window_ms);
                let fish_key = ((guid as u32).rotate_left(9) ^ sequence).max(1);
                tracked.attempt = Some(Attempt {
                    stage: AttemptStage::Casting,
                    cast_at_ms: now,
                    bite_at_ms: bite_at,
                    hook_at_ms: hook_at,
                    expires_at_ms: expires_at,
                    fish_key,
                    species_key: selected.key.clone(),
                });
                change.replies.push(bobber_reply(actor, target));
                change.replies.push(fish_run_reply(fish_key, selected));
                change.changed = true;
            }
            ClientRequest::ReelIn { flag, .. } => {
                debug_assert_eq!(flag, 1);
                match tracked.attempt.as_ref().map(|a| a.stage) {
                    Some(AttemptStage::Casting | AttemptStage::Nibbling) => {
                        change.replies.extend(escape(tracked, actor));
                        change.changed = true;
                    }
                    Some(AttemptStage::Hooked) if tracked.session.phase() == Phase::Reeling => {
                        tracked.session.request(&request)?;
                        settle_catch(
                            tracked,
                            &self.item_definitions,
                            &mut self.paid,
                            guid,
                            now_ms,
                            &mut change,
                        )?;
                    }
                    _ => return Err(Error::OutOfOrder),
                }
            }
            ClientRequest::Special { .. } => {
                // The native client has a handle-only writer but no sender.
            }
            ClientRequest::RegisterPlayer { .. } => unreachable!(),
        }
        Ok(change)
    }

    /// Fire server timers for one active attempt. All durations come from the
    /// reconstructed tuning file; client packets carry no equivalent clock.
    pub fn tick(&mut self, guid: u64, now_ms: u64) -> Result<Change, Error> {
        let Some(tracked) = self.sessions.get_mut(&guid) else {
            return Ok(Change::default());
        };
        if !tracked.playing {
            return Ok(Change::default());
        }
        advance(tracked, now_ms)
    }

    fn bound_to(&self, actor: Handle) -> Option<u64> {
        self.sessions
            .iter()
            .find(|(_, tracked)| tracked.session.actor() == Some(actor))
            .map(|(guid, _)| *guid)
    }
}

fn advance(tracked: &mut Tracked, now_ms: u64) -> Result<Change, Error> {
    let Some(attempt) = tracked.attempt.as_mut() else {
        return Ok(Change::default());
    };
    let actor = tracked.session.actor().ok_or(Error::OutOfOrder)?;
    let mut change = Change::default();
    if attempt.stage == AttemptStage::Casting && now_ms >= attempt.bite_at_ms {
        attempt.stage = AttemptStage::Nibbling;
        tracked.session.nibbling();
        change
            .replies
            .push(update_fish(actor, attempt.fish_key, false, true));
        change.changed = true;
    }
    if attempt.stage == AttemptStage::Nibbling && now_ms >= attempt.hook_at_ms {
        attempt.stage = AttemptStage::Hooked;
        tracked.session.hooked();
        change
            .replies
            .push(update_fish(actor, attempt.fish_key, true, false));
        change.changed = true;
    }
    if attempt.stage == AttemptStage::Hooked && now_ms >= attempt.expires_at_ms {
        change.replies.extend(escape(tracked, actor));
        change.changed = true;
    }
    Ok(change)
}

fn escape(tracked: &mut Tracked, actor: Handle) -> Vec<Vec<u8>> {
    let Some(attempt) = tracked.attempt.take() else {
        return Vec::new();
    };
    tracked.session.cancel();
    vec![
        update_fish(actor, attempt.fish_key, false, false),
        result_reply(&ResultBody {
            actor,
            at_18: 0,
            ..ResultBody::default()
        }),
    ]
}

fn settle_catch(
    tracked: &mut Tracked,
    item_definitions: &BTreeMap<(String, u8), i32>,
    paid: &mut BTreeMap<u64, PaidCatches>,
    guid: u64,
    now_ms: u64,
    change: &mut Change,
) -> Result<(), Error> {
    let attempt = tracked.attempt.clone().ok_or(Error::OutOfOrder)?;
    let fish = species(&attempt.species_key).ok_or(Error::InvalidValue)?;
    let seed = mix64(guid ^ u64::from(attempt.fish_key) ^ u64::from(now_ms as u32));
    let roll = unit(seed);
    if roll < CONTENT.timing.line_snap_chance
        || roll >= CONTENT.timing.line_snap_chance + CONTENT.timing.catch_chance
    {
        change
            .replies
            .extend(escape(tracked, tracked.session.actor().unwrap()));
        change.changed = true;
        return Ok(());
    }

    // Only sizes with a bound inventory item can be caught. A fish with none gets away instead
    // of a catch that would pay nothing.
    let bound = [1u8, 2, 3, 4].map(|g| item_definitions.contains_key(&(fish.key.clone(), g)));
    let Some(grade) = choose_grade(fish, bound, seed.rotate_left(17)) else {
        change
            .replies
            .extend(escape(tracked, tracked.session.actor().unwrap()));
        change.changed = true;
        return Ok(());
    };

    // The optional cap is checked only after the server has selected a size: a blocked catch
    // takes the same no-payout escape the bag-full path uses, and changes no reward state.
    let cap = tracked.cap.clone();
    let ledger = paid.entry(guid).or_default();
    if !cap_allows(cap.as_ref(), ledger, now_ms) {
        change
            .replies
            .extend(escape(tracked, tracked.session.actor().unwrap()));
        change.notices.push(CATCH_CAP_NOTICE);
        change.changed = true;
        return Ok(());
    }

    let payout = tracked
        .payout
        .as_ref()
        .and_then(|table| table.for_grade(grade))
        .copied();
    let definition_id = item_definitions[&(fish.key.clone(), grade)];
    let weight_roll = unit(seed.rotate_left(31));
    let weight = fish.weight_kg[0] + (fish.weight_kg[1] - fish.weight_kg[0]) * weight_roll;
    let elapsed = now_ms.saturating_sub(attempt.cast_at_ms) as f32 / 1000.0;
    let score = fish.difficulty.saturating_mul(fish.rarity);
    let result = ResultBody {
        actor: tracked.session.actor().ok_or(Error::OutOfOrder)?,
        at_18: 5,
        at_1c: false,
        at_20: fish.name_id,
        at_24: fish.icon_id,
        at_38: weight,
        at_3c: u32::from(grade),
        at_40: elapsed,
        at_44: fish.difficulty,
        at_48: fish.rarity,
        at_4c: score,
        ..ResultBody::default()
    };

    // apply_grant stages on a clone. Commit service state only after the full
    // inventory/display transaction succeeds; the caller persists it with the
    // fishing/minigame session before sending the returned packets.
    // (R) Fisherman XP only while the job is unlocked: the reward transaction refuses a locked one.
    let xp = payout.map_or(0, |row| row.fisherman_xp);
    let job_xp = if xp > 0
        && tracked
            .rewards
            .profiles
            .unlocked
            .contains(&FISHERMAN_PROFILE)
    {
        BTreeMap::from([(FISHERMAN_PROFILE, xp)])
    } else {
        BTreeMap::new()
    };
    let grant = RewardGrant {
        coins: payout.map_or(0, |row| row.coins),
        items: vec![ItemGrant {
            definition_id,
            count: 1,
        }],
        job_xp,
        sc: 0,
        display: RewardDisplay {
            source_guid: guid,
            player_guid: guid,
            reward_kind: 0,
            icon_id: fish.icon_id as i32,
            name_id: fish.name_id as i32,
            rows: Vec::new(),
        },
    };
    let grant_key = format!(
        "fishing:{guid}:{}:{}",
        tracked.instance_id, tracked.cast_sequence
    );
    let Ok((next_rewards, reward_packets)) = apply_grant(&tracked.rewards, &grant, &grant_key)
    else {
        // A full bag (or an item the catalogue lacks): the fish is released and nothing is paid.
        change
            .replies
            .extend(escape(tracked, tracked.session.actor().unwrap()));
        change
            .notices
            .push("Your bag is full, so the fish got away.");
        change.changed = true;
        return Ok(());
    };

    change.replies.push(result_reply(&result));
    change.replies.extend(reward_packets);
    change.quest_events.push(QuestEvent::CollectItem {
        item_id: definition_id,
        quantity: 1,
    });
    // (R) One paid catch is one quest-visible minigame completion.
    change.quest_events.push(QuestEvent::FinishMinigame {
        minigame_id: tracked.game_id as u32,
    });
    change.changed = true;
    change.paid = true;
    tracked.rewards = next_rewards;
    tracked.attempt = None;
    tracked.session.cancel();
    cap_record(cap.as_ref(), ledger, now_ms);
    Ok(())
}

/// Whether another paid catch fits the optional cap. A disabled or absent cap never blocks.
fn cap_allows(cap: Option<&CatchCap>, paid: &mut PaidCatches, now_ms: u64) -> bool {
    let Some(cap) = cap.filter(|cap| cap.enabled) else {
        return true;
    };
    let limit = cap.max_paid_catches.unwrap_or(0);
    match cap.window {
        CapWindow::Session => paid.session < limit,
        CapWindow::Hour => {
            let cutoff = now_ms.saturating_sub(HOUR_WINDOW_MS);
            paid.hour.retain(|at| *at > cutoff);
            paid.hour.len() < limit as usize
        }
    }
}

/// Count one paid catch. Only successful grants reach this call; escapes do not consume a slot.
fn cap_record(cap: Option<&CatchCap>, paid: &mut PaidCatches, now_ms: u64) {
    let Some(cap) = cap.filter(|cap| cap.enabled) else {
        return;
    };
    match cap.window {
        CapWindow::Session => paid.session = paid.session.saturating_add(1),
        CapWindow::Hour => paid.hour.push(now_ms),
    }
}

fn player_status(guid: u64, flags: u32) -> Vec<u8> {
    crate::kart_gateway::player_status(guid, flags)
}

/// The shared lifecycle messages `MinigameRuns` consumes: 39/5 start, 39/6 leave, 39/7 ack,
/// 39/46 wheel stop, 41/109 encounter close and 26/42 offer cancel (the length is checked there).
fn is_lifecycle(inner: &[u8]) -> bool {
    inner == [26, 0, 42, 0]
        || inner.starts_with(&[41, 0, 109, 0])
        || (inner.starts_with(&[39, 0]) && matches!(inner.get(2), Some(5 | 6 | 7 | 46)))
}

fn registration_reply(tracked: &Tracked) -> Result<Vec<u8>, Error> {
    let area = area(&tracked.area_key).ok_or(Error::InvalidValue)?;
    let mut out = packet(Sub::RegisterPlayerResponse, Vec::new());
    for (index, value) in CONTENT.registration.player_words.iter().enumerate() {
        if index == 6 {
            // BobberSplashCompositeEffectId: the client default is -1, so it goes out as FF FF FF FF.
            out.extend((*value as i32).to_le_bytes());
        } else {
            out.extend(value.to_le_bytes());
        }
    }
    push_string(&mut out, &CONTENT.registration.underwater_sky);
    for (index, value) in tracked.zone_words.iter().enumerate() {
        if index == 3 {
            out.extend((*value as u32).to_le_bytes());
        } else {
            out.extend(value.to_le_bytes());
        }
    }
    push_count(&mut out, CONTENT.registration.integer_values.len());
    for value in &CONTENT.registration.integer_values {
        out.extend(value.to_le_bytes());
    }
    let visible = area
        .fish
        .iter()
        .filter_map(|key| species(key))
        .collect::<Vec<_>>();
    push_count(&mut out, visible.len());
    for fish in visible {
        push_fish_entry(&mut out, fish, tracked.fisherman_level);
    }
    Ok(out)
}

fn fish_info_reply(tracked: &Tracked) -> Result<Vec<u8>, Error> {
    let area = area(&tracked.area_key).ok_or(Error::InvalidValue)?;
    let fish = area
        .fish
        .iter()
        .filter_map(|key| species(key))
        .collect::<Vec<_>>();
    let mut out = packet(Sub::FishInfoUpdate, Vec::new());
    push_count(&mut out, fish.len());
    for fish in fish {
        push_fish_entry(&mut out, fish, tracked.fisherman_level);
    }
    Ok(out)
}

fn push_fish_entry(out: &mut Vec<u8>, fish: &Species, fisherman_level: u32) {
    // Field widths/order are recovered. Which flag/text field drives each
    // Fish Finder column is unresolved; these assignments are R.
    out.extend(fish.type_id.to_le_bytes());
    out.extend(fish.name_id.to_le_bytes());
    out.extend(fish.icon_id.to_le_bytes());
    out.push(0); // R: unknown skip flag
    out.push(0); // R: FishSpecial candidate
    out.extend(0u32.to_le_bytes()); // R: unresolved lure-hint text id
    push_string(out, "");
    out.extend(0u32.to_le_bytes());
    out.push(u8::from(fisherman_level >= fish.minimum_level));
    out.extend(0u32.to_le_bytes());
}

fn update_data_reply(actor: Handle, position: Vec4) -> Vec<u8> {
    let mut body = actor.encode().to_vec();
    body.extend(position.encode());
    packet(Sub::UpdateData, body)
}

fn school_reply(school_id: u32, anchor: Vec4, position: [f32; 3]) -> Vec<u8> {
    let mut body = Vec::with_capacity(48);
    body.extend(school_id.to_le_bytes());
    body.extend(
        Vec4 {
            x: anchor.x + position[0],
            y: anchor.y + position[1],
            z: anchor.z + position[2],
            w: 1.0,
        }
        .encode(),
    );
    body.extend(
        Vec4 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
        .encode(),
    );
    push_count(&mut body, 0); // R: no recovered school model assignment
    push_count(&mut body, 0); // R: no recovered school path assignment
    body.extend(0u32.to_le_bytes());
    body.extend(0u32.to_le_bytes());
    packet(Sub::SpawnSchool, body)
}

fn bobber_reply(actor: Handle, target: Vec4) -> Vec<u8> {
    let mut body = actor.encode().to_vec();
    // The client keeps this u32 as the player's bobber model (+0xd8); `0x00ccfdb0` looks a
    // positive one up without a null check.
    body.extend(CONTENT.models.bobber.to_le_bytes());
    body.extend(target.encode());
    body.extend(
        Vec4 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
        .encode(),
    );
    packet(Sub::SpawnBobber, body)
}

/// 138/18 for one attempt: its fish alone, as the head record (138/7 needs one). Caster only:
/// case 18 has no handle. (R) The model follows the species' likeliest size (ties to the
/// smaller); the catch card keeps the size `settle_catch` rolls.
///
/// (R) `on_hook` = 1: the client starts the fish at the hook in state 6 (`0x00cd3a70`, +0x10),
/// and state 4 sends 138/7 past its +0xd9 gate (`0x00b6cbd5`). With 0 the fish starts away from
/// the hook (`0x00b6d5d4`) and the lure is drawn there, but our hook 138/10 sets +0xd9
/// (`0x00b6de7c`) and that gate then blocks the reel. Side effect: every escape (138/10 0,0)
/// shows 'Item Lost', not 'Fish Escaped' (`0x00b6df48`).
fn fish_run_reply(fish_key: u32, fish: &Species) -> Vec<u8> {
    let models = &CONTENT.models;
    let grade = (1..=4)
        .rev()
        .max_by_key(|grade| fish.size_weights[grade - 1])
        .unwrap_or(1);
    fishing::encode_spawn_fish_run(&FishRun {
        flag: true, // R
        model: models.run_header,
        texture: Vec::new(), // R
        fish: vec![RunFish {
            id: fish_key,
            model: models.fish_by_grade[grade - 1],
            size_grade: grade as u32,
            on_hook: true,
            motion: models.fish_motion,
            motion_tail: models.fish_motion_tail,
            ..RunFish::default()
        }],
    })
}

fn update_fish(actor: Handle, fish_key: u32, first: bool, second: bool) -> Vec<u8> {
    let mut body = actor.encode().to_vec();
    body.extend(fish_key.to_le_bytes());
    body.push(u8::from(first));
    body.push(u8::from(second));
    packet(Sub::UpdateBobber, body)
}

fn despawn_school(school_id: u32) -> Vec<u8> {
    packet(Sub::DespawnSchool, school_id.to_le_bytes().to_vec())
}

fn result_reply(body: &ResultBody) -> Vec<u8> {
    let mut out = packet(Sub::Result, Vec::new());
    out.extend(body.actor.encode());
    out.extend(body.at_18.to_le_bytes());
    out.push(u8::from(body.at_1c));
    out.extend(body.at_20.to_le_bytes());
    out.extend(body.at_24.to_le_bytes());
    push_string_bytes(&mut out, &body.blob_2c);
    out.extend(body.at_38.to_le_bytes());
    out.extend(body.at_3c.to_le_bytes());
    out.extend(body.at_40.to_le_bytes());
    out.extend(body.at_44.to_le_bytes());
    out.extend(body.at_48.to_le_bytes());
    out.extend(body.at_4c.to_le_bytes());
    out.extend(body.at_50.to_le_bytes());
    push_string_bytes(&mut out, &body.blob_58);
    push_string_bytes(&mut out, &body.blob_64);
    out.extend(body.at_74.to_le_bytes());
    out.extend(body.at_7c.to_le_bytes());
    out.push(u8::from(body.at_80));
    out.extend(body.at_78.to_le_bytes());
    out
}

fn packet(sub: Sub, body: Vec<u8>) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + body.len());
    out.extend(fishing::OPCODE.to_le_bytes());
    out.extend(sub.as_u16().to_le_bytes());
    out.extend(body);
    out
}

fn push_string(out: &mut Vec<u8>, value: &str) {
    push_string_bytes(out, value.as_bytes());
}

fn push_string_bytes(out: &mut Vec<u8>, value: &[u8]) {
    push_count(out, value.len());
    out.extend(value);
}

fn push_count(out: &mut Vec<u8>, count: usize) {
    out.extend((count as u32).to_le_bytes());
}

fn validate_cast(target: Vec4, anchor: Vec4) -> Result<(), Error> {
    if !finite_vec(target) {
        return Err(Error::InvalidValue);
    }
    let dx = target.x - anchor.x;
    let dy = target.y - anchor.y;
    let dz = target.z - anchor.z;
    let distance = (dx * dx + dy * dy + dz * dz).sqrt();
    let min = CONTENT.registration.player_words[1];
    let max = CONTENT.registration.player_words[2];
    if distance < min || distance > max {
        return Err(Error::InvalidValue);
    }
    Ok(())
}

fn select_species(tracked: &Tracked, seed: u32) -> Result<&'static Species, Error> {
    let fishing_area = area(&tracked.area_key).ok_or(Error::InvalidValue)?;
    let lure = tracked
        .lure_key
        .as_deref()
        .and_then(|key| CONTENT.lures.iter().find(|l| l.key == key));
    let mut choices = Vec::new();
    let mut total = 0u64;
    for key in &fishing_area.fish {
        let Some(fish) = species(key) else { continue };
        if fish.minimum_level > tracked.fisherman_level {
            continue;
        }
        let mut weight = u64::from(fish.selection_weight);
        if lure.is_some_and(|l| l.fish.iter().any(|boosted| boosted == key)) {
            let bonus = lure.unwrap().bonus_percent;
            weight += weight * u64::from(bonus) / 100;
        }
        total = total.checked_add(weight).ok_or(Error::InvalidValue)?;
        choices.push((fish, weight));
    }
    if choices.is_empty() || total == 0 {
        return Err(Error::InvalidValue);
    }
    let mut pick = u64::from(mix32(seed)) % total;
    for (fish, weight) in choices {
        if pick < weight {
            return Ok(fish);
        }
        pick -= weight;
    }
    Err(Error::InvalidValue)
}

/// Pick a size grade (1..=4) by weight among the sizes that have an inventory item.
fn choose_grade(fish: &Species, bound: [bool; 4], seed: u64) -> Option<u8> {
    let weight = |i: usize| {
        if bound[i] {
            u64::from(fish.size_weights[i])
        } else {
            0
        }
    };
    let total = (0..4).map(weight).sum::<u64>();
    if total == 0 {
        return None;
    }
    let mut pick = mix64(seed) % total;
    for index in 0..4 {
        if pick < weight(index) {
            return Some(index as u8 + 1);
        }
        pick -= weight(index);
    }
    None
}

fn school_ids(guid: u64, count: usize) -> Vec<u32> {
    let base = mix32(guid as u32 ^ (guid >> 32) as u32) & 0x3fff_ffff;
    (0..count)
        .map(|index| base.wrapping_add(index as u32 + 1).max(1))
        .collect()
}

fn unit(seed: u64) -> f32 {
    (mix64(seed) >> 40) as f32 / (1u32 << 24) as f32
}

fn mix32(mut value: u32) -> u32 {
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846c_a68b);
    value ^ (value >> 16)
}

fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn finite_vec(value: Vec4) -> bool {
    [value.x, value.y, value.z, value.w]
        .into_iter()
        .all(f32::is_finite)
}

fn area(key: &str) -> Option<&'static Area> {
    CONTENT.areas.iter().find(|area| area.key == key)
}

fn species(key: &str) -> Option<&'static Species> {
    CONTENT.species.iter().find(|fish| fish.key == key)
}

/// Why an ordinary entry was refused. Nothing moved; the player is told with [`Self::message`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryDenied {
    /// The chosen NPC is not a listed guide (a forged choice, or the list changed).
    NotAGate,
    AlreadyFishing,
    /// Another activity (crafting, gathering, a race, a battle) is running.
    Busy,
    Full,
    /// The offer could not be built (bad content).
    Unavailable,
}

impl EntryDenied {
    pub fn message(self) -> &'static str {
        match self {
            Self::NotAGate => "Nobody here can take you fishing.",
            Self::AlreadyFishing => "You are already fishing.",
            Self::Busy => "Finish the active activity before fishing.",
            Self::Full => "The fishing spots are full. Try again soon.",
            Self::Unavailable => "Fishing isn't open yet.",
        }
    }
}

/// The optional `fishing_spots.json`: which guide NPCs offer a fishing trip (R), the area and
/// client MiniGameData row and scene each one stands for, the locale line of the menu option, the
/// per-grade payout table and the optional catch cap (R), and the inventory item behind every
/// fish size. Absent file = nobody offers fishing. The file is operator content: bounded,
/// regular-file only, strict.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FishingSpots {
    schema_version: u32,
    /// Locale id of the menu line (R: "Go fish!").
    menu_string_id: u32,
    payout: PayoutTable,
    catch_cap: CatchCap,
    spots: Vec<Spot>,
    /// Species key to the item for sizes small, medium, large, extra large; 0 = no such size.
    items: BTreeMap<String, [i32; 4]>,
}

/// One reconstructed per-catch payout row, selected by the server-chosen size grade.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GradePayout {
    grade: u8,
    coins: u32,
    fisherman_xp: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PayoutTable {
    /// Provenance; every value is reconstructed because no original payout table was recovered.
    source: String,
    by_grade: Vec<GradePayout>,
}

impl PayoutTable {
    fn valid(&self) -> bool {
        self.source.contains("R:")
            && self.by_grade.len() == 4
            && self.by_grade.iter().enumerate().all(|(index, row)| {
                row.grade == index as u8 + 1
                    && row.coins <= MAX_PAYOUT_COINS
                    && row.fisherman_xp <= MAX_CATCH_XP
            })
    }

    fn for_grade(&self, grade: u8) -> Option<&GradePayout> {
        self.by_grade.iter().find(|row| row.grade == grade)
    }
}

/// The operator-selectable window. Both are reconstructed policies; neither is client-visible.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum CapWindow {
    /// Paid catches across trips in this login session.
    Session,
    /// Paid catches in a rolling 3600-second window, retained across reconnects by character.
    Hour,
}

/// Optional per-player cap on **paid** catches. A blocked catch escapes and changes no reward.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CatchCap {
    /// Provenance; the cap is an operator choice, not a recovered original value.
    source: String,
    enabled: bool,
    window: CapWindow,
    /// Required (1..=window maximum) when enabled. Null keeps the shipped file off.
    #[serde(default)]
    max_paid_catches: Option<u32>,
}

impl CatchCap {
    fn maximum(&self) -> u32 {
        match self.window {
            CapWindow::Session => MAX_PAID_CATCHES_PER_SESSION,
            CapWindow::Hour => MAX_PAID_CATCHES_PER_HOUR,
        }
    }

    fn valid(&self) -> bool {
        if !self.source.contains("R:") {
            return false;
        }
        match (self.enabled, self.max_paid_catches) {
            (true, Some(limit)) => (1..=self.maximum()).contains(&limit),
            // A pre-set value while disabled must still be sane, so flipping the switch cannot
            // enable an absurd cap without another edit.
            (false, Some(limit)) => (1..=self.maximum()).contains(&limit),
            (false, None) => true,
            (true, None) => false,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Spot {
    npc: u64,
    area: String,
    game_id: i32,
    name_id: i32,
    description_id: i32,
    icon_id: i32,
    scene: Scene,
    /// Provenance only: why this NPC stands for this area.
    #[allow(dead_code)]
    source: String,
}

/// The fishing hole's own client scene. The client draws underwater fish at fixed coordinates
/// of the current scene (`0x00b640e0`: X = processor +0x140 + |aim - +0x240| x +0x13C, Z = 485),
/// and the 138/3 handler copies the zone block to processor +0xFC (`0x00b6cf19`), so +0x13C and
/// +0x140 are UnderwaterRunScale and UnderwaterRunOffset. Each scene's Underwater_Bed sits near
/// Z 485, so the round must send its own scene's bed X.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Scene {
    /// Client zone name, e.g. `bw_fishing_medpond`.
    name: String,
    /// Zone (activity) id paired with the spot's MiniGameData row.
    zone_id: u32,
    spawn_position: [f32; 3],
    spawn_rotation: [f32; 4],
    /// Water surface height, for the spawn bound and fishing-hole-zoning. Not a zone word:
    /// BobberGroundCheckMinY (word 4) is a Y offset from the bobber ([`Scene::zone_words`]).
    water_y: f32,
    /// (R) X of the scene's Underwater_Bed area, sent as zone word 1 (UnderwaterRunOffset). A
    /// reconstruction: `bbe_FishingZoneConfig.txt` is lost. With RunScale 1 and word 4 at 0 the
    /// fish X runs from bed X to bed X + (cast distance - BobberMinDistanceFromPlayer).
    underwater_bed_x: f32,
    /// Provenance only: the scene's .gzne and Areas.xml are in our extracted client catalogue.
    #[allow(dead_code)]
    client_scene_cached: bool,
    source: String,
}

impl Scene {
    /// (R) Sanity bounds: every number finite, the bed X inside 0..=200 (the five client beds
    /// read 55..=74), and the spawn at most half a metre under the water (one bank spawn sits
    /// 4 cm below it).
    fn valid(&self) -> bool {
        self.spawn_position
            .iter()
            .chain(&self.spawn_rotation)
            .chain([&self.water_y, &self.underwater_bed_x])
            .all(|value| value.is_finite())
            && (0.0..=200.0).contains(&self.underwater_bed_x)
            && self.spawn_position[1] >= self.water_y - 0.5
            && self.zone_id != 0
            && !self.name.is_empty()
            && !self.source.is_empty()
    }

    /// The 138/3 zone block (`0x00b67240`) for this scene: `fishing.json`'s words with this
    /// scene's bed X as word 1. Word 4 is not the water height: processor +0x250 (`0x00b6cfaa`)
    /// is added to the aim Y for the cast point (`0x00b6ca39`) and reaches bobber +0xE4
    /// (`0x00b6d337`, `0x00ccff06`), the ground ray's Y offset from the bobber (`0x00cd3e89`).
    fn zone_words(&self) -> [f32; 5] {
        let mut words = CONTENT.registration.zone_words;
        words[1] = self.underwater_bed_x;
        words
    }
}

impl FishingSpots {
    /// Read `fishing_spots.json` from the content directory. `Ok(None)` when it is absent.
    pub fn load(directory: &Path) -> io::Result<Option<Self>> {
        let path = directory.join(SPOTS_FILE);
        let metadata = match std::fs::symlink_metadata(&path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
            Ok(m) => m,
        };
        if !metadata.is_file() {
            return Err(invalid("fishing spots must be a regular file"));
        }
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(MAX_SPOTS_FILE as u64 + 1)
            .read_to_end(&mut bytes)?;
        Self::parse(&bytes).map(Some)
    }

    pub fn parse(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() > MAX_SPOTS_FILE {
            return Err(invalid("fishing spots exceed 64 KiB"));
        }
        let spots: Self = serde_json::from_slice(bytes).map_err(invalid)?;
        let (mut seen, mut zones) = (BTreeSet::new(), BTreeSet::new());
        if spots.schema_version != 1
            || spots.menu_string_id == 0
            || !spots.payout.valid()
            || !spots.catch_cap.valid()
            || spots.spots.len() > MAX_SPOTS
            || !spots.spots.iter().all(|s| {
                s.npc != 0
                    && seen.insert(s.npc)
                    && area(&s.area).is_some()
                    && s.game_id > 0
                    && s.name_id >= 0
                    && s.description_id >= 0
                    && s.icon_id >= 0
                    && s.scene.valid()
                    && zones.insert(s.scene.zone_id)
            })
            || !spots.items.iter().all(|(key, sizes)| {
                species(key).is_some()
                    && sizes.iter().all(|id| *id >= 0)
                    && sizes.iter().any(|id| *id > 0)
            })
        {
            return Err(invalid("invalid fishing spots"));
        }
        Ok(spots)
    }

    /// The NPCs whose menu offers fishing.
    pub fn gate_npcs(&self) -> Vec<u64> {
        self.spots.iter().map(|s| s.npc).collect()
    }

    pub fn menu_string_id(&self) -> u32 {
        self.menu_string_id
    }

    fn spot(&self, npc: u64) -> Option<&Spot> {
        self.spots.iter().find(|s| s.npc == npc)
    }
}

fn invalid(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

#[derive(Deserialize)]
struct Content {
    schema_version: u32,
    minigame_type_id: i32,
    source_tags: BTreeMap<String, String>,
    timing: Timing,
    registration: Registration,
    models: Models,
    sizes: Vec<Size>,
    areas: Vec<Area>,
    species: Vec<Species>,
    lures: Vec<Lure>,
}

#[derive(Deserialize)]
struct Timing {
    bite_delay_ms: u64,
    nibble_ms: u64,
    reel_window_ms: u64,
    catch_chance: f32,
    line_snap_chance: f32,
    source: String,
}

#[derive(Deserialize)]
struct Registration {
    /// The 138/3 player block in client reader order (`0x00b65880`). Word 6
    /// (BobberSplashCompositeEffectId) goes out as an i32, the rest as f32 (`registration_reply`).
    player_words: [f32; 16],
    underwater_sky: String,
    /// The 138/3 zone block (`0x00b67240`). Word 3 (TensionMeterAmplifier) goes out as a u32.
    /// A spot's scene replaces word 1 ([`Scene::zone_words`]).
    zone_words: [f32; 5],
    integer_values: Vec<u32>,
    source: String,
}

impl Registration {
    /// The integer words must be whole and in range, so `registration_reply`'s casts are exact.
    /// The round trip also refuses NaN and infinities.
    fn valid(&self) -> bool {
        let (splash, amplifier) = (self.player_words[6], self.zone_words[3]);
        splash as i32 as f32 == splash && amplifier as u32 as f32 == amplifier
    }
}

/// Client model ids and fish motion for 138/8 and 138/18; provenance in `fishing.json`.
#[derive(Deserialize)]
struct Models {
    bobber: i32,
    run_header: i32,
    fish_by_grade: [i32; 4],
    fish_motion: [f32; 10],
    fish_motion_tail: f32,
}

impl Models {
    /// The client looks every id up unchecked, so each must be positive; motion is a sanity
    /// bound (R), which also refuses NaN and infinities.
    fn valid(&self) -> bool {
        [self.bobber, self.run_header]
            .iter()
            .chain(&self.fish_by_grade)
            .all(|id| *id > 0)
            && self
                .fish_motion
                .iter()
                .chain([&self.fish_motion_tail])
                .all(|value| (0.0..=100.0).contains(value))
    }
}

#[derive(Deserialize)]
struct Size {
    grade: u8,
    name: String,
    source: String,
}

#[derive(Deserialize)]
struct Area {
    key: String,
    name: String,
    fish: Vec<String>,
    schools: Vec<[f32; 3]>,
    source: String,
}

#[derive(Deserialize)]
struct Species {
    key: String,
    name: String,
    name_id: u32,
    type_id: u32,
    minimum_level: u32,
    rarity: u32,
    difficulty: u32,
    selection_weight: u32,
    weight_kg: [f32; 2],
    size_weights: [u32; 4],
    #[serde(default)]
    icon_id: u32,
    source_tags: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct Lure {
    key: String,
    name: String,
    bonus_percent: u32,
    fish: Vec<String>,
    source: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_bounds_refuse_a_zero_id_and_wild_motion() {
        let mut json: serde_json::Value = serde_json::from_str(CONTENT_JSON).unwrap();
        let valid = |json: &serde_json::Value| {
            serde_json::from_value::<Models>(json["models"].clone())
                .unwrap()
                .valid()
        };
        assert!(valid(&json));
        json["models"]["fish_by_grade"][2] = 0.into();
        assert!(!valid(&json));
        json["models"]["fish_by_grade"][2] = 1672.into();
        json["models"]["fish_motion_tail"] = 100.5.into();
        assert!(!valid(&json));
    }

    #[test]
    fn registration_refuses_fractional_integer_words() {
        let mut json: serde_json::Value = serde_json::from_str(CONTENT_JSON).unwrap();
        let valid = |json: &serde_json::Value| {
            serde_json::from_value::<Registration>(json["registration"].clone())
                .unwrap()
                .valid()
        };
        assert!(valid(&json));
        json["registration"]["player_words"][6] = 0.5.into();
        assert!(!valid(&json));
        json["registration"]["player_words"][6] = (-1.0).into();
        json["registration"]["zone_words"][3] = 1.5.into();
        assert!(!valid(&json));
        json["registration"]["zone_words"][3] = (-1.0).into();
        assert!(!valid(&json), "zone word 3 is a u32");
    }
}
