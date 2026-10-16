//! Server-authoritative fishing service for native minigame type 21.
//!
//! Recovered layouts: fishing sub 2/5/6/7/15 client requests; sub 1 update,
//! sub 3 registration, sub 4 fish list, sub 8 bobber, sub 10 fish state,
//! sub 11 school, sub 12 despawn and sub 14 result. Sources are the readers
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

use crate::{
    fishing::{self, ClientRequest, Error, Handle, Outcome, Phase, ResultBody, Sub, Vec4},
    minigame::{MinigameRuns, RunMode, RunSpec, RunState},
    quest_engine::QuestEvent,
    rewards::{ItemGrant, RewardDisplay, RewardGrant, RewardState, apply_grant},
};
use serde::Deserialize;
use std::{collections::BTreeMap, sync::LazyLock};

const CONTENT_JSON: &str = include_str!("../data/content/fishing.json");

pub const MAX_SESSIONS: usize = 4;

static CONTENT: LazyLock<Content> = LazyLock::new(|| {
    serde_json::from_str(CONTENT_JSON).expect("bundled fishing content must parse")
});

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Change {
    /// Inner application packets, ready for the gateway's outer tunnel.
    pub replies: Vec<Vec<u8>>,
    /// Authoritative objective events for the caller's QuestEngine.
    pub quest_events: Vec<QuestEvent>,
    pub changed: bool,
}

#[derive(Clone)]
struct Tracked {
    runs: MinigameRuns,
    instance_id: i32,
    area_key: String,
    fisherman_level: u32,
    anchor: Vec4,
    rewards: RewardState,
    session: fishing::Session,
    attempt: Option<Attempt>,
    school_ids: Vec<u32>,
    lure_key: Option<String>,
    cast_sequence: u32,
    playing: bool,
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
            && CONTENT.registration.source.contains("R:")
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
    /// is unresolved (R). The returned 39/16 offer must be sent before client
    /// start handling. Call `mark_started` after the encounter is ready.
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
        let mut runs = MinigameRuns::new(id).map_err(|_| Error::InvalidValue)?;
        let offered = runs
            .offer(RunSpec::empty(
                game_id,
                CONTENT.minigame_type_id,
                RunMode::Encounter {
                    encounter_id,
                    player_guid: guid,
                },
            ))
            .map_err(|_| Error::InvalidValue)?;
        if !offered.accepted {
            return Err(Error::OutOfOrder);
        }
        self.next_instance_id = id.checked_add(1).ok_or(Error::InvalidValue)?;
        self.sessions.insert(
            guid,
            Tracked {
                runs,
                instance_id: id,
                area_key: area_key.to_owned(),
                fisherman_level,
                anchor,
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
        self.sessions.remove(&guid).is_some()
    }

    pub fn reset_all(&mut self) -> usize {
        let count = self.sessions.len();
        self.sessions.clear();
        count
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
                tracked.session.request(&request)?;
                tracked.cast_sequence = tracked.cast_sequence.wrapping_add(1).max(1);
                let selected = select_species(tracked, tracked.cast_sequence)?;
                let now = now_ms;
                let bite_at = now.saturating_add(CONTENT.timing.bite_delay_ms);
                let hook_at = bite_at.saturating_add(CONTENT.timing.nibble_ms);
                let expires_at = hook_at.saturating_add(CONTENT.timing.reel_window_ms);
                let fish_key = ((guid as u32).rotate_left(9) ^ tracked.cast_sequence).max(1);
                tracked.attempt = Some(Attempt {
                    stage: AttemptStage::Casting,
                    cast_at_ms: now,
                    bite_at_ms: bite_at,
                    hook_at_ms: hook_at,
                    expires_at_ms: expires_at,
                    fish_key,
                    species_key: selected.key.clone(),
                });
                change.replies.push(bobber_reply(actor, fish_key, target));
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
                        settle_catch(tracked, &self.item_definitions, guid, now_ms, &mut change)?;
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

    let grade = choose_grade(fish, seed.rotate_left(17));
    let definition_id = *item_definitions
        .get(&(fish.key.clone(), grade))
        .ok_or(Error::InvalidValue)?;
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
    let grant = RewardGrant {
        coins: 0,
        items: vec![ItemGrant {
            definition_id,
            count: 1,
        }],
        job_xp: BTreeMap::new(),
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
    let (next_rewards, reward_packets) =
        apply_grant(&tracked.rewards, &grant, &grant_key).map_err(|_| Error::InvalidValue)?;

    change.replies.push(result_reply(&result));
    change.replies.extend(reward_packets);
    change.quest_events.push(QuestEvent::CollectItem {
        item_id: definition_id,
        quantity: 1,
    });
    change.changed = true;
    tracked.rewards = next_rewards;
    tracked.attempt = None;
    tracked.session.cancel();
    Ok(())
}

fn registration_reply(tracked: &Tracked) -> Result<Vec<u8>, Error> {
    let area = area(&tracked.area_key).ok_or(Error::InvalidValue)?;
    let mut out = packet(Sub::RegisterPlayerResponse, Vec::new());
    for (index, value) in CONTENT.registration.player_words.iter().enumerate() {
        if index == 6 {
            out.extend((*value as u32).to_le_bytes());
        } else {
            out.extend(value.to_le_bytes());
        }
    }
    push_string(&mut out, &CONTENT.registration.underwater_sky);
    for (index, value) in CONTENT.registration.zone_words.iter().enumerate() {
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

fn bobber_reply(actor: Handle, fish_key: u32, target: Vec4) -> Vec<u8> {
    let mut body = actor.encode().to_vec();
    body.extend(fish_key.to_le_bytes());
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

fn choose_grade(fish: &Species, seed: u64) -> u8 {
    let total = fish.size_weights.iter().map(|n| u64::from(*n)).sum::<u64>();
    let mut pick = mix64(seed) % total;
    for (index, weight) in fish.size_weights.iter().enumerate() {
        if pick < u64::from(*weight) {
            return index as u8 + 1;
        }
        pick -= u64::from(*weight);
    }
    4
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

#[derive(Deserialize)]
struct Content {
    schema_version: u32,
    minigame_type_id: i32,
    source_tags: BTreeMap<String, String>,
    timing: Timing,
    registration: Registration,
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
    player_words: [f32; 16],
    underwater_sky: String,
    zone_words: [f32; 5],
    integer_values: Vec<u32>,
    source: String,
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
