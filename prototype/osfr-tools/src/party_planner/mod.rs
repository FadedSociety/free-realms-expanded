//! Party planners and their venues (the early-2010 party update, PB3). A planner NPC books a
//! party instance of its venue for a host, private or for the host's friends and group; the host
//! invites guests, who arrive through the invite or through the planner while the party runs. A
//! party ends after a set time or when its host leaves, and everyone goes back where they came
//! from. This module is the venue table (`party_venues.json`) and a pure registry of live parties:
//! no I/O after the load, no clock, no sessions. The gateway (`party_routing.rs`) owns the
//! instances, the transfers and the props; `wire` holds the packets.
//!
//! # Evidence
//!
//! * freerealms.com article 763 (captured 2011-02-09): Preston Daly by the Sanctuary warpstone
//!   teleports to "the party area behind the Pixiewood sign"; Courtney Lakebash, on a Lakeshore
//!   walkway, into "the nearby tree house party area"; the Briarwood Party Planner by the
//!   Bristlewood warpstone, whose "party is located in the towers high above the Tanglewood Fort";
//!   only Members could throw a party, a non-Member friend could teleport in; the party cake gives
//!   "a temporary illusion". Article 937 names her Mandy Vice, as the client's own name row does
//!   (Global.Text 422676; 763's "Mandy Vide" is a typo), and she is already the Briarwood row
//!   9500000172, so no new NPC row is drawn for her.
//! * The client's own text: the planners' lines 422717 (Preston), 422818 (Courtney) and 422690
//!   (Mandy), titles 422710, 425220 and 422677, the ticket names 422707, 422705 and 422667.
//! * The world geometry still holds the 2010 party areas (`lab/reports/world-objects-20261015`):
//!   `evnt_private_party_ropes_01` around a clearing behind `sg_bixiewood_sign_01` at
//!   (-1198, 20, 494); two `evnt_world_private_party_tree_01` tree houses at (-700, -11, 522) and
//!   (-674, -8, 559) with tables, chairs and a cake on their decks; party ropes, rugs, tables and a
//!   Briarwood cake on two tower tops at y -30.6 and -30.3 above the fort (door at y -83).
//!
//! # Reconstructed (R)
//!
//! * Every player may host (the free economy); the 2010 ticket items are not needed.
//! * A party lasts `duration_ms` (60 minutes), holds `max_guests` (24, host included), and a guest
//!   farther than `leave_radius_m` from the venue's centre has left it (a warp or a walk away).
//! * "Friends and group" admits the host's mutual friends and the members of the host's group
//!   without an invite; a private party admits only the invited. An invite lasts as long as the
//!   party. One party per host, and nobody is in two parties at once.
//! * Guests arrive at the venue's spots in turn; a party guid is 'PRTY' in the high word with a
//!   counter in bits 8..32, and its props take the low byte (1..=255).

pub mod wire;

use crate::{
    startup_world::{self, NpcSpawn},
    world::{self, InstanceId},
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::Path,
    sync::Arc,
};

pub const FILE: &str = "party_venues.json";
const MAX_FILE: usize = 64 * 1024;
const MAX_VENUES: usize = 16;
const MAX_SPOTS: usize = 32;
const MAX_PROPS: usize = 32;
const MAX_DURATION_MS: u64 = 24 * 3_600_000;
const MAX_GUESTS: usize = 64;
/// Most outstanding invites one party keeps; a host cannot grow the table without bound.
const MAX_INVITES: usize = 128;
/// 'PRTY' in the high word of every party and prop guid.
pub const GUID_TAG: u64 = 0x5052_5459_0000_0000;
const GUID_TAG_MASK: u64 = 0xffff_ffff_0000_0000;

/// Whether `guid` names a party (the low byte clear), live or not.
pub fn is_party(guid: u64) -> bool {
    guid & GUID_TAG_MASK == GUID_TAG && guid & 0xff == 0
}

/// One placed thing: an item id (cake, boombox) or a model id (decoration) at `[x, y, z, heading]`.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prop {
    pub id: i32,
    pub at: [f32; 4],
}

/// One venue, booked at its planner.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Venue {
    pub key: String,
    /// How chat lines name the venue ("the Pixiewood Hills party").
    pub name: String,
    /// The planner's actor guid: the id a client click names.
    pub planner: u64,
    /// Where the planner stands (the drawn row's position); a click from farther is refused.
    pub planner_at: [f32; 3],
    /// The planner's own line, the booking dialog's body.
    pub greeting_id: u32,
    /// The private row's label (the venue's ticket name).
    pub private_label_id: u32,
    pub centre: [f32; 3],
    /// Arrival spots `[x, y, z, heading]` on the venue's floor.
    pub spots: Vec<[f32; 4]>,
    pub cakes: Vec<Prop>,
    pub boombox: Prop,
    pub decorations: Vec<Prop>,
    pub source: Vec<String>,
}

impl Venue {
    fn valid(&self) -> bool {
        let finite = |values: &[f32]| values.iter().all(|v| v.is_finite());
        let mut props = self
            .cakes
            .iter()
            .chain(&self.decorations)
            .chain([&self.boombox]);
        !self.key.is_empty()
            && !self.name.is_empty()
            && self.planner > 0
            && finite(&self.planner_at)
            && finite(&self.centre)
            && self.greeting_id > 0
            && self.private_label_id > 0
            && (1..=MAX_SPOTS).contains(&self.spots.len())
            && self.spots.iter().all(|spot| finite(spot))
            && self.cakes.len() + self.decorations.len() <= MAX_PROPS
            && props.all(|prop| prop.id > 0 && finite(&prop.at))
            && !self.source.is_empty()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    schema_version: u32,
    duration_ms: u64,
    max_guests: usize,
    leave_radius_m: f32,
    reach_m: f32,
    venues: Vec<Venue>,
}

/// The venue table, read-only and shared.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Venues {
    pub duration_ms: u64,
    pub max_guests: usize,
    pub leave_radius_m: f32,
    /// How near the planner a click must stand.
    pub reach_m: f32,
    venues: Vec<Arc<Venue>>,
}

impl Venues {
    /// `party_venues.json` from the content directory; a missing file is no venues.
    pub fn load(directory: &Path) -> io::Result<Self> {
        let bytes = match fs::read(directory.join(FILE)) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(error),
        };
        let invalid = |what: String| io::Error::new(io::ErrorKind::InvalidData, what);
        if bytes.len() > MAX_FILE {
            return Err(invalid(format!("{FILE} exceeds 64 KiB")));
        }
        let file: File = serde_json::from_slice(&bytes).map_err(|e| invalid(e.to_string()))?;
        if file.schema_version != 1
            || !(1..=MAX_DURATION_MS).contains(&file.duration_ms)
            || !(2..=MAX_GUESTS).contains(&file.max_guests)
            || !(10.0..=1000.0).contains(&file.leave_radius_m)
            || !(1.0..=50.0).contains(&file.reach_m)
            || file.venues.len() > MAX_VENUES
        {
            return Err(invalid(format!("unsupported {FILE}")));
        }
        for (index, venue) in file.venues.iter().enumerate() {
            let earlier = &file.venues[..index];
            if !venue.valid()
                || earlier
                    .iter()
                    .any(|v| v.key == venue.key || v.planner == venue.planner)
            {
                return Err(invalid(format!("invalid party venue {}", venue.key)));
            }
        }
        Ok(Self {
            duration_ms: file.duration_ms,
            max_guests: file.max_guests,
            leave_radius_m: file.leave_radius_m,
            reach_m: file.reach_m,
            venues: file.venues.into_iter().map(Arc::new).collect(),
        })
    }

    /// The venue a click on `planner` books.
    pub fn by_planner(&self, planner: u64) -> Option<&Arc<Venue>> {
        self.venues.iter().find(|venue| venue.planner == planner)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arc<Venue>> {
        self.venues.iter()
    }
}

/// Who a party lets in besides the host and the invited.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Privacy {
    Private,
    /// The host's mutual friends and group members come without an invite.
    FriendsAndGroup,
}

/// Where a guest came from: the instance and pose to go back to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Back {
    pub instance: InstanceId,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// This host already has a party.
    Hosting,
    /// This player is already in a party.
    InParty,
    /// Only a host invites.
    NotHosting,
    /// No live party has that guid.
    Unknown,
    /// A private party, and no invite.
    NotInvited,
    Full,
    Yourself,
    /// The party already holds its cap of invites.
    Invites,
}

/// One live party.
#[derive(Clone, Debug, PartialEq)]
pub struct Party {
    pub guid: u64,
    pub host: u64,
    pub venue: Arc<Venue>,
    pub instance: InstanceId,
    pub privacy: Privacy,
    pub ends_ms: u64,
    invited: BTreeSet<u64>,
    /// Everyone admitted (the host too), with where each came from.
    guests: BTreeMap<u64, Back>,
    /// The guests already sent the props in their current stay.
    shown: BTreeSet<u64>,
    next_spot: usize,
    /// What a guest is sent on arrival: the decorations' and cakes' AddNpc and effects.
    props: Vec<Vec<u8>>,
}

impl Party {
    pub fn props(&self) -> &[Vec<u8>] {
        &self.props
    }

    pub fn guests(&self) -> impl Iterator<Item = (u64, &Back)> {
        self.guests.iter().map(|(guid, back)| (*guid, back))
    }

    pub fn has_guest(&self, who: u64) -> bool {
        self.guests.contains_key(&who)
    }

    pub fn is_invited(&self, who: u64) -> bool {
        self.invited.contains(&who)
    }

    /// Whether `who` may come in: the host, the invited, or (friends and group) someone related
    /// to the host. `related` is the caller's mutual-friend-or-group test.
    fn admits(&self, who: u64, related: bool) -> bool {
        who == self.host
            || self.invited.contains(&who)
            || (self.privacy == Privacy::FriendsAndGroup && related)
    }
}

/// The live parties of one server and the venue table they book from. Session-only: a restart
/// ends every party (its instances are gone too).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Parties {
    venues: Option<Arc<Venues>>,
    live: Vec<Party>,
    next: u32,
    /// Each character's open booking dialog: the planner it clicked.
    prompts: BTreeMap<u64, u64>,
}

impl Parties {
    pub fn venues(&self) -> Option<&Arc<Venues>> {
        self.venues.as_ref()
    }

    /// Install the venue table once; later calls keep the first.
    pub fn set_venues(&mut self, venues: Venues) {
        self.venues.get_or_insert_with(|| Arc::new(venues));
    }

    pub fn iter(&self) -> impl Iterator<Item = &Party> {
        self.live.iter()
    }

    pub fn get(&self, guid: u64) -> Option<&Party> {
        self.live.iter().find(|party| party.guid == guid)
    }

    fn get_mut(&mut self, guid: u64) -> Option<&mut Party> {
        self.live.iter_mut().find(|party| party.guid == guid)
    }

    pub fn hosted_by(&self, host: u64) -> Option<&Party> {
        self.live.iter().find(|party| party.host == host)
    }

    /// The party `who` is admitted to (as host or guest).
    pub fn containing(&self, who: u64) -> Option<&Party> {
        self.live.iter().find(|party| party.has_guest(who))
    }

    pub fn open_prompt(&mut self, who: u64, planner: u64) {
        self.prompts.insert(who, planner);
    }

    pub fn prompt(&self, who: u64) -> Option<u64> {
        self.prompts.get(&who).copied()
    }

    pub fn close_prompt(&mut self, who: u64) -> Option<u64> {
        self.prompts.remove(&who)
    }

    /// The next unused party guid; a full counter wraps past live guids.
    fn free_guid(&self) -> (u32, u64) {
        let mut counter = self.next;
        loop {
            counter = counter.wrapping_add(1) & 0x00ff_ffff;
            let guid = GUID_TAG | (u64::from(counter) << 8);
            if counter != 0 && self.get(guid).is_none() {
                return (counter, guid);
            }
        }
    }

    /// The guid `book` would give the next party, so the caller can key its instance by it.
    pub fn next_guid(&self) -> u64 {
        self.free_guid().1
    }

    /// Book `venue` for `host` in `instance` until `now_ms + duration_ms`. The host is admitted
    /// with `back`. A refusal changes nothing.
    pub fn book(
        &mut self,
        host: u64,
        venue: Arc<Venue>,
        privacy: Privacy,
        instance: InstanceId,
        back: Back,
        now_ms: u64,
    ) -> Result<u64, Refusal> {
        if self.hosted_by(host).is_some() {
            return Err(Refusal::Hosting);
        }
        if self.containing(host).is_some() {
            return Err(Refusal::InParty);
        }
        let duration = self.venues.as_ref().map_or(0, |venues| venues.duration_ms);
        let (counter, guid) = self.free_guid();
        self.next = counter;
        self.live.push(Party {
            guid,
            host,
            venue,
            instance,
            privacy,
            ends_ms: now_ms.saturating_add(duration),
            invited: BTreeSet::new(),
            guests: BTreeMap::from([(host, back)]),
            shown: BTreeSet::new(),
            // The host stands on the first spot (`host_spot`); guests take the rest in turn.
            next_spot: 1,
            props: Vec::new(),
        });
        Ok(guid)
    }

    /// The packets each arriving guest is sent (built by the host once the venue is set up).
    pub fn furnish(&mut self, guid: u64, props: Vec<Vec<u8>>) {
        if let Some(party) = self.get_mut(guid) {
            party.props = props;
        }
    }

    /// `host` invites `guest` to the party it hosts; the party's guid.
    pub fn invite(&mut self, host: u64, guest: u64) -> Result<u64, Refusal> {
        let party = (self.live.iter_mut())
            .find(|party| party.host == host)
            .ok_or(Refusal::NotHosting)?;
        if guest == host {
            return Err(Refusal::Yourself);
        }
        if !party.invited.contains(&guest) && party.invited.len() >= MAX_INVITES {
            return Err(Refusal::Invites);
        }
        party.invited.insert(guest);
        Ok(party.guid)
    }

    /// A declined invite is gone; the guest may still be invited again.
    pub fn decline(&mut self, guid: u64, guest: u64) -> Option<u64> {
        let party = self.get_mut(guid)?;
        party.invited.remove(&guest).then_some(party.host)
    }

    /// Whether `who` may enter party `guid` now. `related` is the caller's mutual-friend-or-group
    /// test against the host.
    pub fn may_enter(&self, guid: u64, who: u64, related: bool) -> Result<(), Refusal> {
        let party = self.get(guid).ok_or(Refusal::Unknown)?;
        if self.containing(who).is_some() {
            return Err(Refusal::InParty);
        }
        if !party.admits(who, related) {
            return Err(Refusal::NotInvited);
        }
        let cap = self.venues.as_ref().map_or(0, |venues| venues.max_guests);
        if party.guests.len() >= cap {
            return Err(Refusal::Full);
        }
        Ok(())
    }

    /// A party at `venue` that `who` may enter: one they were invited to first, then one whose
    /// host `related` approves (friends and group).
    pub fn joinable(&self, venue: &str, who: u64, related: impl Fn(u64) -> bool) -> Option<u64> {
        let here: Vec<&Party> = self.live.iter().filter(|p| p.venue.key == venue).collect();
        let fits = |party: &Party, related: bool| self.may_enter(party.guid, who, related).is_ok();
        (here.iter())
            .find(|party| party.is_invited(who) && fits(party, false))
            .or_else(|| here.iter().find(|party| fits(party, related(party.host))))
            .map(|party| party.guid)
    }

    /// Admit `who` (already checked by `may_enter`) with where they came from; the spot they
    /// arrive at.
    pub fn admit(&mut self, guid: u64, who: u64, back: Back) -> Option<[f32; 4]> {
        let party = self.get_mut(guid)?;
        party.guests.insert(who, back);
        party.shown.remove(&who);
        Some(party.spot())
    }

    /// The host's own arrival spot (the first).
    pub fn host_spot(&self, guid: u64) -> Option<[f32; 4]> {
        self.get(guid)?.venue.spots.first().copied()
    }

    /// `who` left the party on their own: forgotten, nothing to send back.
    pub fn leave(&mut self, guid: u64, who: u64) -> Option<Back> {
        let party = self.get_mut(guid)?;
        party.shown.remove(&who);
        party.guests.remove(&who)
    }

    /// Record that `who` was sent the props; true the first time in this stay.
    pub fn mark_shown(&mut self, guid: u64, who: u64) -> bool {
        self.get_mut(guid)
            .is_some_and(|party| party.shown.insert(who))
    }

    /// The parties whose time is over at `now_ms`.
    pub fn due(&self, now_ms: u64) -> Vec<u64> {
        let over = self.live.iter().filter(|party| now_ms >= party.ends_ms);
        over.map(|party| party.guid).collect()
    }

    /// End party `guid`: it leaves the registry with its guests and their way back.
    pub fn end(&mut self, guid: u64) -> Option<Party> {
        let at = self.live.iter().position(|party| party.guid == guid)?;
        Some(self.live.remove(at))
    }
}

impl Party {
    /// The next arrival spot, in turn.
    fn spot(&mut self) -> [f32; 4] {
        let spots = &self.venue.spots;
        let at = spots
            .get(self.next_spot % spots.len().max(1))
            .copied()
            .unwrap_or([
                self.venue.centre[0],
                self.venue.centre[1],
                self.venue.centre[2],
                0.0,
            ]);
        self.next_spot = self.next_spot.wrapping_add(1);
        at
    }
}

/// The facing vector `(sin h, 0, cos h)` the character state and AddNpc both carry.
pub fn facing(heading: f32) -> [f32; 3] {
    [heading.sin(), 0.0, heading.cos()]
}

/// AddNpc 35/2 for each decoration of `venue`, guid `party | (index + 1)`: a still prop with no
/// nameplate and no interaction (the boombox's stance without its bounce).
pub fn decoration_packets(guid: u64, venue: &Venue) -> io::Result<Vec<Vec<u8>>> {
    (venue.decorations.iter().enumerate())
        .map(|(index, prop)| {
            let [x, y, z, heading] = prop.at;
            let [sin, _, cos] = facing(heading);
            // `valid` caps the props at 32, so the index always fits the low byte.
            let npc = NpcSpawn {
                actor_id: guid | (index as u64 + 1),
                model_id: prop.id,
                position: [x, y, z, 1.0],
                rotation: [sin, 0.0, cos, 0.0],
                hide_nameplate: 1,
                is_interactable: 0,
                interact_range: 0,
                ..world::content_spawn_template()
            };
            world::unframe_tunnel(&startup_world::build_add_npc(&npc))
        })
        .collect()
}

#[cfg(test)]
mod tests;
