//! PB3: the gateway side of `osfr_tools::party_planner`. Before gameplay sees a message, `route`
//! answers the planner click (26/8) with the booking dialog and its rows (26/6), an invite's
//! accept (127/19) and decline (127/16) that name a party, and the `realm party` words. A booking
//! makes the venue's own `InstanceKind::Party` child of the open world, sends the host there
//! (opcode-31 begin zoning, `Gateway::stage_instance_transfer`, committed through the host's own
//! session as a GM move is) and furnishes it: the decorations, the venue's cakes in the shared
//! cake registry (so the ordinary cake click gives its effect) and a boombox playing its song.
//! `tick` sends each arriving guest the props, ends a party whose time is over or whose host has
//! gone, and sends everyone back where they came from; a guest who walks or warps away from the
//! venue has left it and stays in the open world where they are.
//!
//! Everything answers to the session that acted, every refusal is one chat line to that player,
//! and nothing is saved: a restart ends every party, and a guest who logged out inside one comes
//! back in the open world at the venue (R).
use super::*;
use osfr_tools::chat;
use osfr_tools::gameplay::Presence;
use osfr_tools::npc_interaction;
use osfr_tools::party_planner::{
    Back, Party, Privacy, Refusal, Venue, Venues,
    wire::{self, Choice, Word},
};
use osfr_tools::world::{InstanceId, InstanceKind};

const OPEN_WORLD_ZONE: u32 = 0;
/// Only a message holding one of these is decoded; movement skips the route.
const HEADS: [[u8; 4]; 5] = [
    [26, 0, 8, 0],
    [26, 0, 6, 0],
    [127, 0, 19, 0],
    [127, 0, 16, 0],
    [15, 0, 1, 0],
];
const OVER: &str = "The party is over. Thanks for coming!";
const HOST_LEFT: &str = "The host has left, so the party is over.";
const LEFT: &str = "You have left the party.";

/// One client message, before gameplay. `true`: it was a party message and is answered.
pub(super) fn route(
    me: &mut Session,
    message: &[u8],
    now_ms: u64,
    world: &mut SharedWorld,
    membership: &mut World,
    sessions: &mut SessionMap,
) -> bool {
    let named = |w: &[u8]| HEADS.iter().any(|head| w == head);
    if !me.gateway.is_world_loaded() || !message.windows(4).any(named) {
        return false;
    }
    let Ok(requests) = me.gateway.application_requests(message) else {
        return false;
    };
    let [inner] = requests.as_slice() else {
        return false;
    };
    let Some(venues) = venues(me, membership) else {
        return false;
    };
    let mut me = Some(me);
    let at = Here {
        now_ms,
        world,
        membership,
        sessions,
    };
    match inner.get(..4) {
        Some([26, 0, 8, 0]) => click(&mut me, at, &venues, inner),
        Some([26, 0, 6, 0]) => answer(&mut me, at, inner),
        Some([127, 0, 19, 0]) => wire::accepted(inner).is_some_and(|party| {
            let reply = enter(&mut me, at, party);
            say(&mut me, &reply.unwrap_or_else(|refusal| refusal));
            true
        }),
        Some([127, 0, 16, 0]) => wire::declined(inner).is_some_and(|party| {
            decline(&mut me, at, party);
            true
        }),
        _ => chat(&mut me, at, inner),
    }
}

/// The shared state a route step works on.
struct Here<'a> {
    now_ms: u64,
    world: &'a mut SharedWorld,
    membership: &'a mut World,
    sessions: &'a mut SessionMap,
}

impl Here<'_> {
    fn reborrow(&mut self) -> Here<'_> {
        Here {
            now_ms: self.now_ms,
            world: self.world,
            membership: self.membership,
            sessions: self.sessions,
        }
    }
}

/// The venue table, read once from the content set the session's gameplay loaded (the same
/// `npc_content` rule as the cakes); a file that cannot be read is no venues and one log line.
fn venues(me: &Session, membership: &mut World) -> Option<Arc<Venues>> {
    if membership.parties().venues().is_none() {
        let dir = osfr_tools::content::npc_content(me.gateway.gameplay()?.content_dir());
        let venues = Venues::load(&dir).unwrap_or_else(|error| {
            event(json!({"event": "party_venues_unreadable", "reason": error.to_string()}));
            Venues::default()
        });
        membership.parties_mut().set_venues(venues);
    }
    membership.parties().venues().cloned()
}

fn tell(session: &mut Session, packet: &[u8]) {
    if let Err(error) = session.queue_shared_message(&wrap_application(packet)) {
        event(
            json!({"event": "party_send_refused", "session": session.id.0, "reason": error.to_string()}),
        );
    }
}

fn say(me: &mut Option<&mut Session>, text: &str) {
    if let Some(me) = me.as_deref_mut() {
        tell(me, &chat::system(text));
    }
}

/// Run `act` on character `who`'s session: `me` when it is the one being dispatched (it is not
/// in `sessions`), else the stored one, out of the map for the call.
fn with_session<R>(
    who: u64,
    me: &mut Option<&mut Session>,
    at: &mut Here,
    act: impl FnOnce(&mut Session, Here<'_>) -> R,
) -> Option<R> {
    if let Some(me) = me.as_deref_mut().filter(|me| me.character_guid == who) {
        return Some(act(me, at.reborrow()));
    }
    let key = *at
        .world
        .session_keys
        .get(at.world.active_sessions.get(&who)?)?;
    let mut session = at.sessions.remove(&key)?;
    let result = act(&mut session, at.reborrow());
    at.sessions.insert(key, session);
    Some(result)
}

fn session_of<'a>(
    world: &SharedWorld,
    sessions: &'a mut SessionMap,
    who: u64,
) -> Option<&'a mut Session> {
    let key = world.session_keys.get(world.active_sessions.get(&who)?)?;
    sessions.get_mut(key)
}

/// Mutual friends or one group, and neither ignores the other: who a friends-and-group party
/// lets in without an invite (R).
fn related(world: &SharedWorld, host: u64, who: u64) -> bool {
    let group = world.social.group_of(host);
    host != who
        && !world.social.ignores_either(host, who)
        && (world.social.house_friends(host, who)
            || (group.is_some() && group == world.social.group_of(who)))
}

fn name_of(world: &SharedWorld, who: u64) -> NameData {
    world.identities.get(&who).map_or_else(
        || NameData::new("", ""),
        |id| NameData::new(id.name.first.clone(), id.name.last.clone()),
    )
}

fn display(name: &NameData) -> String {
    format!("{} {}", name.first, name.last).trim().to_owned()
}

fn refusal_text(refusal: Refusal) -> String {
    match refusal {
        Refusal::Hosting => "You are already hosting a party.",
        Refusal::InParty => "You are already at a party.",
        Refusal::NotHosting => "You are not hosting a party.",
        Refusal::Unknown => "That party is over.",
        Refusal::NotInvited => "That party is private; ask its host for an invitation.",
        Refusal::Full => "Sorry, the party is full right now. Come back later!",
        Refusal::Yourself => "You are the host.",
        Refusal::Invites => "Your party has sent all the invitations it can.",
    }
    .to_owned()
}

/// Whether the session may go to a party: free, in the public open world.
fn may_travel(session: &Session, membership: &World) -> Result<Back, String> {
    let state = session.gateway.state();
    let here = membership
        .player_location(session.character_guid)
        .map(|row| row.instance_id);
    if state.zone_id != OPEN_WORLD_ZONE || here.is_none() || here != membership.public_instance(0) {
        return Err("You can only go to a party from the open world.".into());
    }
    let free = session
        .gateway
        .gameplay()
        .map(|game| game.travel_presence());
    if free != Some(Presence::Free) {
        return Err("Finish what you are doing first.".into());
    }
    Ok(Back {
        instance: here.unwrap_or_default(),
        position: state.position,
        rotation: state.rotation,
    })
}

/// Send `session` to `instance` of the open world at `at`, facing `rotation`.
fn transfer(
    session: &mut Session,
    at: Here,
    instance: InstanceId,
    position: [f32; 3],
    rotation: [f32; 3],
) -> Result<(), String> {
    let to = ZoneInstance::from_world(OPEN_WORLD_ZONE, instance).map_err(|e| e.to_string())?;
    let zone = session.gateway.runtime_zone().map_err(|e| e.to_string())?;
    let scene = gm_powers_routing::scene_at(zone, position, rotation)?;
    let now_ms = at.now_ms;
    gm_powers_routing::commit(
        session,
        at.sessions,
        at.world,
        at.membership,
        now_ms,
        |gateway| gateway.stage_instance_transfer(to, scene, now_ms),
    )
}

/// A planner click: the booking dialog, within reach of the planner. Any other actor's click
/// closes an open booking prompt and goes on to gameplay.
fn click(me: &mut Option<&mut Session>, at: Here, venues: &Venues, inner: &[u8]) -> bool {
    let Some(session) = me.as_deref_mut() else {
        return false;
    };
    let guid = session.character_guid;
    let target = npc_interaction::decode_npc_interact_26_8(inner).map(|r| r.target);
    let Some(venue) = target.ok().and_then(|target| venues.by_planner(target)) else {
        at.membership.parties_mut().close_prompt(guid);
        return false;
    };
    let pose = at
        .world
        .players
        .get(session.id)
        .map(|p| (p.zone, p.position));
    let near = pose.is_some_and(|(zone, position)| {
        let (dx, dz) = (
            position[0] - venue.planner_at[0],
            position[2] - venue.planner_at[2],
        );
        zone.zone_id == OPEN_WORLD_ZONE && dx.hypot(dz) <= venues.reach_m
    });
    if !near {
        return true; // A far click on the planner is silent, as a far NPC click is (R).
    }
    let choices = choices(at.world, at.membership, venue, guid);
    at.membership.parties_mut().open_prompt(guid, venue.planner);
    for packet in wire::dialog(venue, &choices) {
        tell(session, &packet);
    }
    true
}

/// The rows a planner offers `who`.
fn choices(world: &SharedWorld, membership: &World, venue: &Venue, who: u64) -> Vec<Choice> {
    let parties = membership.parties();
    let joinable = parties.joinable(&venue.key, who, |host| related(world, host, who));
    let mut rows = Vec::new();
    if parties.hosted_by(who).is_some() {
        rows.extend([Choice::Invite, Choice::End]);
    } else if parties.containing(who).is_none() {
        rows.extend([Choice::Private, Choice::FriendsAndGroup]);
        rows.extend(joinable.map(|_| Choice::Join));
    }
    rows.push(Choice::Goodbye);
    rows
}

/// A row of the open booking dialog. A 26/6 with no party prompt open, or a row the party dialog
/// never sends, goes on to gameplay.
fn answer(me: &mut Option<&mut Session>, mut at: Here, inner: &[u8]) -> bool {
    let Some(guid) = me.as_deref().map(|me| me.character_guid) else {
        return false;
    };
    let Some(planner) = at.membership.parties().prompt(guid) else {
        return false;
    };
    let response = npc_interaction::decode_dialogue_response_26_6(inner);
    let Some(choice) = response.ok().and_then(Choice::from_response) else {
        return false;
    };
    at.membership.parties_mut().close_prompt(guid);
    if let Some(me) = me.as_deref_mut() {
        tell(me, &wire::close());
    }
    let venues = at.membership.parties().venues().cloned();
    let Some(venue) = venues.as_ref().and_then(|v| v.by_planner(planner)).cloned() else {
        return true;
    };
    let reply = match choice {
        Choice::Private => book(me, at.reborrow(), venue, Privacy::Private),
        Choice::FriendsAndGroup => book(me, at.reborrow(), venue, Privacy::FriendsAndGroup),
        Choice::Join => {
            let world = &*at.world;
            let open = (at.membership.parties())
                .joinable(&venue.key, guid, |host| related(world, host, guid));
            match open {
                Some(party) => enter(me, at.reborrow(), party),
                None => Err("No party here is open to you. Ask its host for an invitation.".into()),
            }
        }
        Choice::Invite => Ok(invite_all(me, at.reborrow())),
        Choice::End => end_hosted(me, at.reborrow()),
        Choice::Goodbye => return true,
    };
    say(me, &reply.unwrap_or_else(|refusal| refusal));
    true
}

/// Book `venue` for the dispatching player and send them there.
fn book(
    me: &mut Option<&mut Session>,
    mut at: Here,
    venue: Arc<Venue>,
    privacy: Privacy,
) -> Result<String, String> {
    let session = me.as_deref_mut().ok_or("no session")?;
    let host = session.character_guid;
    let back = may_travel(session, at.membership)?;
    let parties = at.membership.parties();
    if parties.hosted_by(host).is_some() {
        return Err(refusal_text(Refusal::Hosting));
    }
    let party_guid = parties.next_guid();
    let kind = InstanceKind::Party { party_guid };
    let instance = (at.membership)
        .create_child_instance(back.instance, OPEN_WORLD_ZONE, kind, at.now_ms)
        .map_err(|error| error.to_string())?;
    let guid = (at.membership.parties_mut())
        .book(host, Arc::clone(&venue), privacy, instance, back, at.now_ms)
        .map_err(refusal_text)?;
    let spot = at.membership.parties().host_spot(guid).unwrap_or_default();
    let [x, y, z, heading] = spot;
    let rotation = osfr_tools::party_planner::facing(heading);
    if let Err(error) = transfer(session, at.reborrow(), instance, [x, y, z], rotation) {
        at.membership.parties_mut().end(guid);
        return Err(format!("The party could not start: {error}."));
    }
    furnish(session, at.reborrow(), guid);
    event(
        json!({"event": "party_booked", "party": guid, "host": host, "venue": venue.key, "instance": instance}),
    );
    let who = match privacy {
        Privacy::Private => "Only the players you invite can come",
        Privacy::FriendsAndGroup => "Your friends and group can come, and anyone you invite",
    };
    Ok(format!(
        "Welcome to {}! {who}: type realm party invite <name>, or realm party invite for your \
         friends and group.",
        venue.name
    ))
}

/// Set the venue up: its decorations, its cakes (in the shared registry, for the party's length)
/// and its boombox, then keep the packets an arriving guest is sent.
fn furnish(host: &Session, at: Here, guid: u64) {
    let Some(party) = at.membership.parties().get(guid).cloned() else {
        return;
    };
    let Ok(instance_id) = u64::try_from(party.instance) else {
        return;
    };
    let zone = ZoneInstance {
        zone_id: OPEN_WORLD_ZONE,
        instance_id,
    };
    let venue = &party.venue;
    let left = party.ends_ms.saturating_sub(at.now_ms).max(1);
    let mut props = osfr_tools::party_planner::decoration_packets(guid, venue).unwrap_or_default();
    for cake in &venue.cakes {
        let Some(row) = at.world.cakes.table.get(cake.id) else {
            continue;
        };
        // The venue's cake lasts the party and stands exactly on its spot (R).
        let row = Arc::new(osfr_tools::placed_cakes::Row {
            lifetime_ms: left,
            spawn_offset_m: 0.0,
            ..(**row).clone()
        });
        let [x, y, z, heading] = cake.at;
        let placed = (at.world.cakes.live).place(&row, guid, zone, [x, y, z], heading, at.now_ms);
        props.extend(placed.map(|placed| placed.spawn).unwrap_or_default());
    }
    let row = host
        .gateway
        .gameplay()
        .and_then(|g| g.boombox_row(venue.boombox.id));
    if let Some(row) = row {
        let row = Arc::new(osfr_tools::boombox::Boombox {
            duration_ms: u32::try_from(left).unwrap_or(u32::MAX),
            spawn_offset_m: 0.0,
            ..(*row).clone()
        });
        let [x, y, z, heading] = venue.boombox.at;
        // Nobody is in the instance yet: the boombox tick shows it to each guest in range.
        let _ = (at.world.boomboxes).spawn(
            row,
            guid,
            zone,
            [x, y, z],
            heading,
            at.now_ms,
            &at.world.players,
        );
    }
    at.membership.parties_mut().furnish(guid, props);
}

/// Let the dispatching player into party `party` (an accept or the planner's Join row).
fn enter(me: &mut Option<&mut Session>, mut at: Here, party: u64) -> Result<String, String> {
    let session = me.as_deref_mut().ok_or("no session")?;
    let who = session.character_guid;
    let host = (at.membership.parties().get(party)).map(|p| p.host);
    let host = host.ok_or_else(|| refusal_text(Refusal::Unknown))?;
    let back = may_travel(session, at.membership)?;
    let related = related(at.world, host, who);
    (at.membership.parties().may_enter(party, who, related)).map_err(refusal_text)?;
    let parties = at.membership.parties_mut();
    let spot = parties
        .admit(party, who, back)
        .ok_or_else(|| refusal_text(Refusal::Unknown))?;
    let (instance, name) = (parties.get(party))
        .map(|p| (p.instance, p.venue.name.clone()))
        .ok_or_else(|| refusal_text(Refusal::Unknown))?;
    let [x, y, z, heading] = spot;
    let rotation = osfr_tools::party_planner::facing(heading);
    if let Err(error) = transfer(session, at.reborrow(), instance, [x, y, z], rotation) {
        at.membership.parties_mut().leave(party, who);
        return Err(format!("You could not get to the party: {error}."));
    }
    event(json!({"event": "party_entered", "party": party, "guest": who}));
    Ok(format!("Welcome to {name}!"))
}

/// A declined invite: forgotten, and the host told.
fn decline(me: &mut Option<&mut Session>, at: Here, party: u64) {
    let Some(guest) = me.as_deref().map(|me| me.character_guid) else {
        return;
    };
    let Some(host) = at.membership.parties_mut().decline(party, guest) else {
        return;
    };
    let line = format!(
        "{} declined your party invitation.",
        display(&name_of(at.world, guest))
    );
    if let Some(session) = session_of(at.world, at.sessions, host) {
        tell(session, &chat::system(&line));
    }
}

/// One invite from `host` to `guest`: the client's invite prompt and a line naming the party.
fn invite_one(at: &mut Here, host: u64, guest: u64) -> Result<(), Refusal> {
    if at.world.social.ignores_either(host, guest) || !at.world.active_sessions.contains_key(&guest)
    {
        return Err(Refusal::Unknown);
    }
    if at.membership.parties().containing(guest).is_some() {
        return Err(Refusal::InParty);
    }
    let party = at.membership.parties_mut().invite(host, guest)?;
    let venue = (at.membership.parties().get(party)).map(|p| p.venue.name.clone());
    let (host_name, guest_name) = (name_of(at.world, host), name_of(at.world, guest));
    let line = format!(
        "{} invites you to {}! Accept to go there now, or ask its party planner while it lasts.",
        display(&host_name),
        venue.unwrap_or_default()
    );
    let prompt = wire::invite(party, host, guest, &host_name, &guest_name);
    let session = session_of(at.world, at.sessions, guest).ok_or(Refusal::Unknown)?;
    tell(session, &prompt);
    tell(session, &chat::system(&line));
    Ok(())
}

/// The host's online mutual friends and group members, each invited.
fn invite_all(me: &mut Option<&mut Session>, mut at: Here) -> String {
    let Some(host) = me.as_deref().map(|me| me.character_guid) else {
        return String::new();
    };
    if at.membership.parties().hosted_by(host).is_none() {
        return refusal_text(Refusal::NotHosting);
    }
    let social = &at.world.social;
    let friends = social
        .profile(host)
        .map(|p| p.friends.keys().copied().collect::<Vec<_>>());
    let group =
        (social.group_of(host)).and_then(|id| social.group_members(id).map(<[u64]>::to_vec));
    let mut wanted: BTreeSet<u64> = friends.unwrap_or_default().into_iter().collect();
    wanted.retain(|friend| social.house_friends(host, *friend));
    wanted.extend(group.unwrap_or_default());
    wanted.remove(&host);
    let count = wanted
        .into_iter()
        .filter(|guest| invite_one(&mut at, host, *guest).is_ok())
        .count();
    match count {
        0 => "None of your friends or group members can be invited right now.".into(),
        1 => "Invited 1 friend to your party.".into(),
        n => format!("Invited {n} friends and group members to your party."),
    }
}

/// `realm party invite <name>`: one online player whose name matches.
fn invite_named(me: &mut Option<&mut Session>, mut at: Here, name: &str) -> String {
    let Some(host) = me.as_deref().map(|me| me.character_guid) else {
        return String::new();
    };
    if at.membership.parties().hosted_by(host).is_none() {
        return refusal_text(Refusal::NotHosting);
    }
    let matches = |id: &&SocialIdentity| {
        let full = format!("{} {}", id.name.first, id.name.last);
        full.trim().eq_ignore_ascii_case(name.trim())
    };
    let guest = at
        .world
        .identities
        .values()
        .find(matches)
        .map(|id| id.character_guid);
    let missing = format!("No player named {name} can be invited right now.");
    let Some(guest) = guest else {
        return missing;
    };
    match invite_one(&mut at, host, guest) {
        Ok(()) => format!(
            "Invited {} to your party.",
            display(&name_of(at.world, guest))
        ),
        Err(Refusal::Unknown) => missing,
        Err(refusal) => refusal_text(refusal),
    }
}

/// The dispatching host ends their party.
fn end_hosted(me: &mut Option<&mut Session>, at: Here) -> Result<String, String> {
    let host = me
        .as_deref()
        .map(|me| me.character_guid)
        .ok_or("no session")?;
    let party = (at.membership.parties().hosted_by(host)).map(|p| p.guid);
    let party = party.ok_or_else(|| refusal_text(Refusal::NotHosting))?;
    end_party(me, at, party, OVER);
    Ok("You ended your party.".into())
}

/// End `party`: its cakes and boombox go, and every guest still inside goes back where they
/// came from (the zone's public instance if theirs has closed), told `why`.
fn end_party(me: &mut Option<&mut Session>, mut at: Here, party: u64, why: &str) {
    let Some(party) = at.membership.parties_mut().end(party) else {
        return;
    };
    event(json!({"event": "party_ended", "party": party.guid, "reason": why}));
    if let Ok(instance_id) = u64::try_from(party.instance) {
        let zone = ZoneInstance {
            zone_id: OPEN_WORLD_ZONE,
            instance_id,
        };
        at.world.cakes.live.remove_in(zone);
    }
    for (recipient, packet) in at.world.boomboxes.end_owner(party.guid) {
        let key = at.world.session_keys.get(&recipient).copied();
        if let Some(session) = key.and_then(|key| at.sessions.get_mut(&key)) {
            tell(session, &packet);
        }
    }
    for (who, back) in party.guests() {
        send_back(me, &mut at, &party, who, *back, why);
    }
}

/// Send guest `who` of `party` back to `back` if they are still inside it.
fn send_back(
    me: &mut Option<&mut Session>,
    at: &mut Here,
    party: &Party,
    who: u64,
    back: Back,
    why: &str,
) {
    let inside = at.membership.player_location(who).map(|l| l.instance_id) == Some(party.instance);
    if !inside {
        return;
    }
    let open = at
        .membership
        .instance(back.instance)
        .is_some_and(|i| i.zone_id == OPEN_WORLD_ZONE);
    let instance = if open {
        Some(back.instance)
    } else {
        at.membership.public_instance(0)
    };
    let Some(instance) = instance else {
        return;
    };
    with_session(who, me, at, |session, at| {
        tell(session, &chat::system(why));
        if let Err(error) = transfer(session, at, instance, back.position, back.rotation) {
            event(json!({"event": "party_return_failed", "guest": who, "reason": error}));
        }
    });
}

/// The `realm party` words.
fn chat(me: &mut Option<&mut Session>, mut at: Here, inner: &[u8]) -> bool {
    let Some(guid) = me.as_deref().map(|me| me.character_guid) else {
        return false;
    };
    let text = chat::request(inner, guid).ok().flatten();
    let Some(word) = text.as_deref().and_then(wire::word) else {
        return false;
    };
    let reply = match word {
        Word::Help => wire::HELP.to_owned(),
        Word::Invite(None) => invite_all(me, at.reborrow()),
        Word::Invite(Some(name)) => invite_named(me, at.reborrow(), &name),
        Word::End => end_hosted(me, at.reborrow()).unwrap_or_else(|refusal| refusal),
        Word::Leave => leave(me, at.reborrow()),
    };
    say(me, &reply);
    true
}

/// A guest leaves on their own and goes back where they came from.
fn leave(me: &mut Option<&mut Session>, mut at: Here) -> String {
    let Some(who) = me.as_deref().map(|me| me.character_guid) else {
        return String::new();
    };
    let Some(party) = at.membership.parties().containing(who).cloned() else {
        return "You are not at a party.".into();
    };
    if party.host == who {
        return "You are the host: realm party end ends the party.".into();
    }
    let Some(back) = party
        .guests()
        .find(|(guest, _)| *guest == who)
        .map(|(_, back)| *back)
    else {
        return "You are not at a party.".into();
    };
    send_back(me, &mut at, &party, who, back, LEFT);
    at.membership.parties_mut().leave(party.guid, who);
    String::new()
}

/// Once per world tick: end what is over, let go of guests who left, and show each arriving guest
/// the venue's props. With no party this returns at once.
pub(super) fn tick(server: &mut Server, now_ms: u64) {
    if server.membership.parties().iter().next().is_none() {
        return;
    }
    let mut at = Here {
        now_ms,
        world: &mut server.world,
        membership: &mut server.membership,
        sessions: &mut server.sessions,
    };
    let radius = (at.membership.parties().venues()).map_or(f32::INFINITY, |v| v.leave_radius_m);
    for party in at.membership.parties().due(now_ms) {
        end_party(&mut None, at.reborrow(), party, OVER);
    }
    let live: Vec<_> = (at.membership.parties().iter())
        .map(|p| {
            (
                p.guid,
                p.host,
                p.instance,
                p.venue.centre,
                p.guests().map(|(g, b)| (g, *b)).collect::<Vec<_>>(),
            )
        })
        .collect();
    for (guid, host, instance, centre, guests) in live {
        // (in the party's instance with a live session, within the venue's radius)
        let inside = |at: &Here, who: u64| {
            let here = at.membership.player_location(who).map(|l| l.instance_id);
            let session = session_ref(at.world, at.sessions, who);
            let near = session.is_some_and(|s| {
                let p = s.gateway.state().position;
                (p[0] - centre[0]).hypot(p[2] - centre[2]) <= radius
            });
            (here == Some(instance) && session.is_some(), near)
        };
        if inside(&at, host) != (true, true) {
            end_party(&mut None, at.reborrow(), guid, HOST_LEFT);
            continue;
        }
        for (who, back) in guests.into_iter().filter(|(who, _)| *who != host) {
            match inside(&at, who) {
                (false, _) => {
                    at.membership.parties_mut().leave(guid, who);
                }
                (true, false) => {
                    // A walk or a warp away: out of the party, into the open world where they are.
                    at.membership.parties_mut().leave(guid, who);
                    let Some(public) = at.membership.public_instance(0) else {
                        continue;
                    };
                    with_session(who, &mut None, &mut at, |session, at| {
                        let position = session.gateway.state().position;
                        tell(session, &chat::system(LEFT));
                        if let Err(error) = transfer(session, at, public, position, back.rotation) {
                            event(
                                json!({"event": "party_return_failed", "guest": who, "reason": error}),
                            );
                        }
                    });
                }
                (true, true) => {}
            }
        }
        show_props(&mut at, guid);
    }
}

fn session_ref<'a>(world: &SharedWorld, sessions: &'a SessionMap, who: u64) -> Option<&'a Session> {
    let key = world.session_keys.get(world.active_sessions.get(&who)?)?;
    sessions.get(key)
}

/// Each guest who has loaded into the venue and joined its registry gets the props once a stay.
fn show_props(at: &mut Here, guid: u64) {
    let Some(party) = at.membership.parties().get(guid) else {
        return;
    };
    let Ok(instance_id) = u64::try_from(party.instance) else {
        return;
    };
    let zone = ZoneInstance {
        zone_id: OPEN_WORLD_ZONE,
        instance_id,
    };
    let world = &*at.world;
    let arrived: Vec<u64> = (party.guests().map(|(who, _)| who))
        .filter(|who| {
            let id = world.active_sessions.get(who);
            id.is_some_and(|id| {
                world.joined.contains(id) && world.players.get(*id).is_some_and(|p| p.zone == zone)
            })
        })
        .collect();
    for who in arrived {
        if !at.membership.parties_mut().mark_shown(guid, who) {
            continue;
        }
        let props = at
            .membership
            .parties()
            .get(guid)
            .map(Party::props)
            .unwrap_or_default();
        if let Some(session) = session_of(at.world, at.sessions, who) {
            props.iter().for_each(|packet| tell(session, packet));
        }
    }
}

#[cfg(test)]
#[path = "party_routing_tests.rs"]
mod tests;
