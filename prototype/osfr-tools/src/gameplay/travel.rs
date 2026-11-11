//! Travel: the Atlas table as content, the friend teleport and the rescues that keep a player
//! from being stranded. The wire codecs and the per-character rules stay in `piece_077`.
//!
//! Evidence (FreeRealms.exe by address; client scripts by name):
//! - The Atlas draws one marker per row of message 57 whose atlas name is the open page (Lua
//!   `Atlas:Populate`; reader `00b98b70`). Notification type 7 is a warpstone, 8 the greyed
//!   warpstone with no teleport, 3 a battle, 6 a minigame or shop (atlas movie
//!   `placeMapIndicator`, `rollOutTeleportConfirm`). "Teleport Now" sends op 90 with the row's
//!   id, and only for an id in the client's table (`00b991d0`). The client's own scripts
//!   teleport by id as well (ids 14, 16, 17 and 153), so the whole table has to be there.
//! - A friend, group or guild dot sends op 58 with that character's guid (`Ui.TeleportToFriend`
//!   `00c23b90`, packet ctor `00c1a5e0`). A refusal the client words itself is 74/8 with code 4
//!   (in combat), 10 (cannot teleport to that player) or 11 (playing a minigame), shown by
//!   `ShowFriendStatusMessage` `00cb3fd0`.
//! - "Teleport to Safety" sends op 122 with no body (`Ui.TeleportToSafePoint` `00c23da0`); the
//!   client sends it by itself below the world floor (`00bf9430`). The 2009 help article: outside
//!   it goes to the nearest warpstone already activated, inside a combat instance back to the
//!   beginning of the encounter.
//! - Pressing X on a warpstone (26/8 on the stone actor) opens the Atlas: the server sends the
//!   BaseUi 47/7 script `Atlas.Show`, as Sanctuary's `OpenAtlasInteraction` does. The client's
//!   case-7 reader (`BaseUiPacketDispatch` `00a91bf0`) splits the script at the dot and calls
//!   `Atlas:Show` as a method through `ClientLuaCallByName` (`009797a0`). With no argument the
//!   Lua `Atlas:Show` opens the player's current atlas page (`Ui.GetAtlasName`), and it does
//!   nothing in a minigame, a tutorial or the starting zone. A stone's AddNpc carries no prompt
//!   reach, so the client offers X within the session's 7 m (dispatch `0092f997`); the server
//!   keeps the NPC reach of 12 m.
//!
//! `data/content/travel.json` holds the rows and the tunable rules. A content set without the
//! file keeps the nine curated hubs of W142 (dormant default).
//!
//! (R) Reconstructed here: a house answers 122 by leaving the lot for the saved world pose (a
//! way out that never depends on the lot's own doors); a minigame scene answers it with the
//! client's "exit the current minigame" wording, since its own quit path and a reconnect both
//! return to the world; who counts as a friend for op 58 is decided by the host (mutual
//! friends, one group or one guild).
use super::*;
use piece_077::{
    FriendView, OPEN_WORLD_ZONE, PointOfInterestDefinition, Undiscovered, WARPSTONE_NOTIFICATION,
    WORLD_TELEPORT_REQUEST_OPCODE, WarpDestination, WarpRegistry, WarpRejection, WarpRules,
    WarpWorld, WorldTeleportRequest, ZONE_SAFE_TELEPORT_REQUEST_OPCODE,
};
use std::fs;

pub const CONTENT_FILE: &str = "travel.json";
const MAX_FILE: u64 = 1 << 20;
/// The client keeps its rows in a 1000-bucket table (`HashList_PointOfInterestDefinition_1000_100`).
const MAX_POINTS: usize = 1_000;
const MAX_COORD: f32 = 100_000.0;
/// Client text 2019045072.
const EXIT_MINIGAME: &str = "You cannot teleport now. Exit the current minigame.";
/// The housing family's leave request (127/10, no body).
const LEAVE_HOUSE: [u8; 4] = [127, 0, 10, 0];
/// `sg_warpstone_01.adr`: the body of all 18 stone actors of `npc_spawns_extra.json`, and of
/// no other content actor.
const STONE_MODEL: u32 = 280;
/// The client's own Atlas opener (Lua `Atlas:Show`).
const ATLAS_SCRIPT: &str = "Atlas.Show";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ContentFile {
    schema_version: u32,
    #[serde(default, rename = "source")]
    _source: Vec<String>,
    rules: Rules,
    points: Vec<Point>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rules {
    #[serde(default, rename = "comment")]
    _comment: String,
    attune_radius_m: f32,
    cooldown_ms: u64,
    stuck_cooldown_ms: u64,
    below_world_m: f32,
    login_below_world_m: f32,
    #[serde(default)]
    undiscovered: Undiscovered,
}

/// One row of 57 plus where the server puts the character.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Point {
    id: i32,
    name: String,
    name_id: i32,
    location_id: i32,
    position: [f32; 3],
    /// Arrival; the row's own position when absent.
    spawn: Option<[f32; 3]>,
    /// Arrival facing, only where the original server's reply was captured.
    facing: Option<[f32; 3]>,
    /// (R) A marker that arrives at a warpstone travels only once that stone is found.
    requires_stone: Option<i32>,
    #[serde(default, rename = "spawn_source")]
    _spawn_source: String,
    heading: f32,
    icon_id: i32,
    notification_type: i32,
    sub_name_id: i32,
    unknown_11: i32,
    breadcrumb_quest_id: i32,
    teleport_location_id: i32,
    atlas: String,
    unknown_15: i32,
}

/// The travel registry for a content directory: the whole table from `travel.json`, or the
/// curated hubs when the content set has no such file. A malformed file refuses the load.
pub(super) fn registry(directory: &Path) -> io::Result<WarpRegistry> {
    let mut text = String::new();
    match fs::File::open(directory.join(CONTENT_FILE)) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(WarpRegistry::curated()),
        file => file?.take(MAX_FILE + 1).read_to_string(&mut text)?,
    };
    if text.len() as u64 > MAX_FILE {
        return Err(invalid("travel content exceeds 1 MiB"));
    }
    let file: ContentFile = serde_json::from_str(&text).map_err(|e| invalid(e.to_string()))?;
    from_content(file)
}

fn from_content(file: ContentFile) -> io::Result<WarpRegistry> {
    if file.schema_version != 1 {
        return Err(invalid("unsupported travel content version"));
    }
    if file.points.len() > MAX_POINTS {
        return Err(invalid(
            "travel content has more points than the client's table",
        ));
    }
    let on_map = |v: &[f32]| v.iter().all(|c| c.is_finite() && c.abs() <= MAX_COORD);
    let text_ok =
        |s: &str, max: usize| s.len() <= max && s.bytes().all(|b| (0x20..0x7f).contains(&b));
    let mut ids = BTreeSet::new();
    let mut requires = BTreeMap::new();
    let (mut destinations, mut definitions) = (Vec::new(), Vec::new());
    for point in file.points {
        let spawn = point.spawn.unwrap_or(point.position);
        let facing = point.facing.unwrap_or_default();
        if !ids.insert(point.id)
            || !on_map(&point.position)
            || !on_map(&spawn)
            || !on_map(&facing)
            || !point.heading.is_finite()
            || !text_ok(&point.name, 64)
            || !text_ok(&point.atlas, 32)
        {
            return Err(invalid(format!("travel point {} is invalid", point.id)));
        }
        if let Some(stone) = point.requires_stone {
            requires.insert(point.id, stone);
        }
        destinations.push(WarpDestination {
            id: point.id,
            name: point.name.into(),
            is_warpstone: point.notification_type == WARPSTONE_NOTIFICATION,
            position: [spawn[0], spawn[1], spawn[2], 1.0],
            rotation: [facing[0], facing[1], facing[2], 0.0],
            facing_known: point.facing.is_some(),
        });
        definitions.push(PointOfInterestDefinition {
            id: point.id,
            name_id: point.name_id,
            location_id: point.location_id,
            x: point.position[0],
            y: point.position[1],
            z: point.position[2],
            heading: point.heading,
            icon_id: point.icon_id,
            notification_type: point.notification_type,
            sub_name_id: point.sub_name_id,
            unknown_11: point.unknown_11,
            breadcrumb_quest_id: point.breadcrumb_quest_id,
            teleport_location_id: point.teleport_location_id,
            atlas_name: point.atlas,
            unknown_15: point.unknown_15,
        });
    }
    if (requires.values()).any(|s| !destinations.iter().any(|d| d.is_warpstone && d.id == *s)) {
        return Err(invalid(
            "travel content requires a stone that is not a warpstone row",
        ));
    }
    if !destinations.iter().any(|d| d.is_warpstone) {
        // The 122 rescue and the login rescue both need somewhere to go.
        return Err(invalid("travel content has no warpstone"));
    }
    let mut registry =
        WarpRegistry::atlas(destinations, definitions, file.rules.undiscovered, requires);
    registry
        .set_rules(WarpRules {
            attune_radius_m: file.rules.attune_radius_m,
            cooldown_ms: file.rules.cooldown_ms,
            stuck_cooldown_ms: file.rules.stuck_cooldown_ms,
            below_world_m: file.rules.below_world_m,
            login_below_world_m: file.rules.login_below_world_m,
        })
        .map_err(invalid)?;
    Ok(registry)
}

/// What the host found for the target of one op 58, lent to the requester's gameplay before the
/// request is handled. The guid is the host's own lookup of the request body, never trusted here.
#[derive(Clone, Debug, PartialEq)]
pub enum FriendLend {
    /// The target may be joined: it stands at `position` in the open world.
    Go { guid: u64, position: [f32; 3] },
    /// Refused; `packet` is the 74/8 the client words itself, when the host could build one.
    Refuse { guid: u64, packet: Option<Vec<u8>> },
}

/// Why another player cannot be teleported to right now, for the host's refusal code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Presence {
    Free,
    /// Knocked out, in a battle or engaged in the field: code 4.
    Combat,
    /// A minigame, craft or gather owns the player: code 11.
    Minigame,
    /// Somewhere a visitor cannot be put (a house lot, the prize wheel): code 10.
    Away,
}

impl Gameplay {
    /// The host's answer for the next op 58 in this message (`None` clears it).
    pub fn lend_friend(&mut self, lend: Option<FriendLend>) {
        self.lent_friend = lend.map(Box::new);
    }

    /// The login side of a save in a zone nothing claims: the reference's open-world start for
    /// this character, remembered so `recover_login_pose` reports it and the host saves at once.
    pub fn rescue_unclaimed_zone(
        &mut self,
        lost: &CharacterState,
        start: &CharacterState,
    ) -> CharacterState {
        self.zone_rescued = true;
        CharacterState {
            guid: lost.guid,
            ..start.clone()
        }
    }

    /// (R) Whether a friend may arrive beside this player, by the occupation table's warp gate.
    pub fn travel_presence(&self) -> Presence {
        use occupation::Occupation as O;
        if matches!(self.warp_blocker(), Some(b) if b != piece_077::Blocker::Busy)
            || self.is_occupied(O::FieldEngaged)
        {
            return Presence::Combat;
        }
        match self.blocker(|gates| gates.warp) {
            None => Presence::Free,
            Some(O::Housing | O::Wheel | O::Garage) => Presence::Away,
            Some(_) => Presence::Minigame,
        }
    }

    /// 56 destination list, 90 Atlas teleport, 122 Teleport to Safety, 58 friend teleport.
    /// Every request is validated against server state; no position or path comes from the
    /// client. Op 58 moves only with the host's [`FriendLend`]; without one it is refused.
    pub(super) fn handle_warp(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        let opcode = inner.get(..2).map(|b| u16::from_le_bytes([b[0], b[1]]));
        if opcode == Some(ZONE_SAFE_TELEPORT_REQUEST_OPCODE)
            && inner.len() == 2
            && character.zone_id != OPEN_WORLD_ZONE
        {
            return self.safe_teleport_in_scene(character, now_ms).map(Some);
        }
        let lend = (opcode == Some(WORLD_TELEPORT_REQUEST_OPCODE))
            .then(|| self.lent_friend.take().map(|lend| *lend))
            .flatten();
        let asked = WorldTeleportRequest::decode(inner.get(2..).unwrap_or_default())
            .map(|request| request.player_guid);
        if let (
            Some(FriendLend::Refuse {
                guid,
                packet: Some(packet),
            }),
            Ok(asked),
        ) = (&lend, asked)
            && *guid == asked
        {
            return Ok(Some(Outcome {
                replies: vec![packet.clone()],
                ..Outcome::default()
            }));
        }
        let mut context = self.warp_context(character);
        // (R) Op 122 stays open in a field fight: it is the rescue for a fall through the world,
        // which a fight does not prevent, and its own five-minute cooldown bounds it as an escape.
        if opcode != Some(ZONE_SAFE_TELEPORT_REQUEST_OPCODE) {
            context.blocked = self.atlas_blocker();
        }
        let terrain = self.terrain.clone();
        let zone = character.zone_id;
        let ground = move |x: f32, z: f32| terrain.as_ref()?.height_at(zone, x, z);
        let friend = |guid: u64| match &lend {
            Some(FriendLend::Go {
                guid: lent,
                position,
            }) if *lent == guid => Some(FriendView {
                zone_id: OPEN_WORLD_ZONE,
                position: *position,
                blocked: false,
            }),
            _ => None,
        };
        let world = WarpWorld {
            ground: &ground,
            friend: &friend,
        };
        let Some(reply) = self.warp.handle(inner, &context, now_ms, &world) else {
            return Ok(None);
        };
        if matches!(reply.rejection, Some(WarpRejection::MalformedRequest(_))) {
            return Err(invalid("invalid warpstone request"));
        }
        let mut out = Outcome {
            changed: reply.changed,
            replies: reply.replies,
            ..Outcome::default()
        };
        if let Some(target) = reply.relocation {
            self.apply_relocation(
                dev_teleport::Relocation {
                    position: target.position,
                    rotation: target.rotation,
                },
                character,
                &mut out,
            )?;
        }
        out.replies
            .extend(reply.notices.iter().map(|text| chat::system(text)));
        Ok(Some(out))
    }

    /// What closes the Atlas and its travel (ops 90 and 58): the warp blocker, and a field fight,
    /// as `travel_presence` reports it to others.
    fn atlas_blocker(&self) -> Option<piece_077::Blocker> {
        let fight = self.is_occupied(occupation::Occupation::FieldEngaged);
        (self.warp_blocker()).or(fight.then_some(piece_077::Blocker::Encounter))
    }

    /// A click (26/8) on any warpstone actor opens the Atlas. Out of reach or off the open world
    /// it answers nothing, as any NPC click (T836); where Atlas travel is closed it answers that
    /// refusal's line instead, so the map never opens on a teleport it would refuse.
    pub(super) fn warpstone_click_26_8(
        &self,
        inner: &[u8],
        character: &CharacterState,
    ) -> io::Result<Option<Outcome>> {
        if !inner.starts_with(&[26, 0, 8, 0]) {
            return Ok(None);
        }
        let target = npc_interaction::decode_npc_interact_26_8(inner)
            .map_err(|_| invalid("invalid NPC interaction request"))?
            .target;
        let Some(table) = (self.npc_visibility.as_ref())
            .filter(|table| table.model_of(target) == Some(STONE_MODEL))
        else {
            return Ok(None);
        };
        let in_reach = table.actor_position(target).is_some_and(|at| {
            let squared: f64 = (at.iter().zip(character.position))
                .map(|(a, b)| (f64::from(*a) - f64::from(b)).powi(2))
                .sum();
            squared <= npc_interaction::INTERACTION_RADIUS.powi(2)
        });
        let mut out = Outcome::default();
        let reason = if character.zone_id != OPEN_WORLD_ZONE {
            Some("wrong_zone")
        } else if !in_reach {
            Some("out_of_range")
        } else if let Some(blocker) = self.atlas_blocker() {
            (out.replies).extend(WarpRejection::Blocked(blocker).notice().map(chat::system));
            Some("travel_closed")
        } else {
            out.replies
                .push(crate::base_ui::execute_script(ATLAS_SCRIPT, &[])?);
            None
        };
        npc_event(serde_json::json!({
            "event": "npc_interact", "guid": target, "content_key": null,
            "outcome": if reason.is_some() { "refused" } else { "atlas" }, "reason": reason
        }));
        Ok(Some(out))
    }

    /// Op 122 outside the open world, where no warpstone applies.
    fn safe_teleport_in_scene(
        &mut self,
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Outcome> {
        if self.is_occupied(occupation::Occupation::Housing)
            && let Some(out) = self.handle_housing(&LEAVE_HOUSE, character)?
        {
            return Ok(out);
        }
        let mut out = Outcome::default();
        let start = (self.encounter.as_ref())
            .and_then(EncounterEngine::zone)
            .filter(|zone| zone.zone_id == character.zone_id);
        let Some(zone) = start else {
            out.replies.push(chat::system(EXIT_MINIGAME));
            return Ok(out);
        };
        // "Inside a combat instance, you will teleport back to the beginning", under the
        // ordinary travel cooldown; a knocked-out player is revived by the battle, not moved.
        let refusal = match self.warp_blocker() {
            Some(piece_077::Blocker::KnockedOut) => {
                Some(WarpRejection::Blocked(piece_077::Blocker::KnockedOut))
            }
            _ => self.warp.charge(now_ms).err(),
        };
        match refusal {
            Some(refusal) => (out.replies).extend(refusal.notice().map(chat::system)),
            None => self.apply_relocation(
                dev_teleport::Relocation {
                    position: zone.position,
                    rotation: zone.rotation,
                },
                character,
                &mut out,
            )?,
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shipped() -> WarpRegistry {
        registry(&Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")).unwrap()
    }

    fn context(position: [f32; 3]) -> piece_077::TravelContext {
        piece_077::TravelContext {
            guid: 1,
            zone_id: OPEN_WORLD_ZONE,
            position,
            rotation: [0.0, 0.0, 1.0],
            blocked: None,
        }
    }

    fn rows(registry: &WarpRegistry) -> Vec<PointOfInterestDefinition> {
        piece_077::decode_point_of_interest_message(&registry.definitions_message()).unwrap()
    }

    fn travel(id: i32) -> Vec<u8> {
        [90u16.to_le_bytes().as_slice(), &id.to_le_bytes()].concat()
    }

    fn no_world() -> WarpWorld<'static> {
        WarpWorld {
            ground: &|_, _| None,
            friend: &|_| None,
        }
    }

    #[test]
    fn the_shipped_table_lists_every_row_and_greys_unfound_stones() {
        let mut registry = shipped();
        let listed = rows(&registry);
        assert_eq!(listed.len(), 137, "136 reference rows and the comic alias");
        let kind = |rows: &[PointOfInterestDefinition], t: i32| {
            rows.iter().filter(|r| r.notification_type == t).count()
        };
        assert_eq!(
            (kind(&listed, 7), kind(&listed, 8)),
            (0, 18),
            "18 stones, none found"
        );
        assert_eq!((kind(&listed, 3), kind(&listed, 6)), (42, 75));
        // Standing at the Seaside stone (row 13) finds it: 57 again, now as type 7.
        let found = registry.visit(&context([-636.96, 9.31, -1041.615]));
        assert!(
            found.changed && found.notices[0].contains("Seaside"),
            "{found:?}"
        );
        let listed = rows(&registry);
        assert_eq!((kind(&listed, 7), kind(&listed, 8)), (1, 17));
        assert_eq!(
            listed
                .iter()
                .find(|r| r.id == 13)
                .unwrap()
                .notification_type,
            7
        );
    }

    #[test]
    fn a_marker_teleports_at_once_and_a_stone_only_once_found() {
        let mut registry = shipped();
        let here = context([0.0, 0.0, 0.0]);
        // Mysterious Merchant (155): the original server's own arrival and facing.
        let reply = registry
            .handle(&travel(155), &here, 0, &no_world())
            .unwrap();
        let to = reply.relocation.expect("a marker needs no finding");
        assert_eq!(to.position, [129.655, 28.593, 296.775]);
        assert_eq!(to.rotation, [0.27307939529418945, 0.0, -0.9619914889335632]);
        let locked = registry
            .handle(&travel(16), &here, 60_000, &no_world())
            .unwrap();
        assert_eq!(locked.rejection, Some(WarpRejection::LockedDestination(16)));
        registry.attune(16);
        let open = registry
            .handle(&travel(16), &here, 60_000, &no_world())
            .unwrap();
        assert!(open.relocation.is_some(), "{open:?}");
        // Precursor Ruins (147, type 10): the Atlas offers no teleport, so neither does the server.
        let ruins = registry
            .handle(&travel(147), &here, 90_000, &no_world())
            .unwrap();
        assert_eq!(ruins.rejection, Some(WarpRejection::NotAWarpstone(147)));
        // The comic book's id 153 arrives at the Sunstone Valley stone, so it needs that stone.
        let early = registry
            .handle(&travel(153), &here, 120_000, &no_world())
            .unwrap();
        assert_eq!(early.rejection, Some(WarpRejection::LockedDestination(153)));
        registry.attune(152);
        let comic = registry
            .handle(&travel(153), &here, 120_000, &no_world())
            .unwrap();
        assert_eq!(comic.relocation.unwrap().position[0], 908.765);
    }

    #[test]
    fn a_cave_mouth_keeps_its_height_and_an_open_marker_is_lifted() {
        let mut registry = shipped();
        let here = context([0.0, 0.0, 0.0]);
        // The grid keeps the highest surface: 100 m of rock over Deep Mines (61, y 47.29).
        let world = WarpWorld {
            ground: &|_, _| Some(147.0),
            friend: &|_| None,
        };
        let cave = registry.handle(&travel(61), &here, 0, &world).unwrap();
        assert_eq!(cave.relocation.unwrap().position[1], 47.290_89);
        // Half a metre of ground over a farm's row lifts the arrival onto it.
        let world = WarpWorld {
            ground: &|_, _| Some(11.5),
            friend: &|_| None,
        };
        let farm = registry.handle(&travel(20), &here, 60_000, &world).unwrap();
        assert_eq!(farm.relocation.unwrap().position[1], 12.0);
    }

    #[test]
    fn safety_goes_to_the_nearest_found_stone_and_to_any_stone_before_the_first() {
        let mut registry = shipped();
        // Beside Blackspore (16) with nothing found: the nearest stone at all.
        let here = context([-2140.0, -32.0, 1040.0]);
        let first = registry.handle(&[122, 0], &here, 0, &no_world()).unwrap();
        assert_eq!(first.relocation.unwrap().position[0], -2140.764);
        // With only Snowhill (3) found, the same spot goes there instead.
        registry.attune(3);
        let later = registry
            .handle(&[122, 0], &here, 3_600_000, &no_world())
            .unwrap();
        assert_eq!(later.relocation.unwrap().position[0], 51.391);
    }

    #[test]
    fn a_friend_in_a_cave_is_joined_in_the_cave_not_on_the_mountain() {
        let mut registry = shipped();
        let friend = |y: f32| FriendView {
            zone_id: OPEN_WORLD_ZONE,
            position: [490.0, y, 273.7],
            blocked: false,
        };
        let here = context([0.0, 0.0, 0.0]);
        let join = [58, 0, 7, 0, 0, 0, 0, 0, 0, 0];
        // 100 m of rock over the friend: the friend's own height.
        let world = WarpWorld {
            ground: &|_, _| Some(147.0),
            friend: &|_| Some(friend(47.3)),
        };
        let cave = registry.handle(&join, &here, 0, &world).unwrap();
        assert_eq!(cave.relocation.unwrap().position[1], 47.3);
        // A friend half a metre into the ground is joined on it.
        let world = WarpWorld {
            ground: &|_, _| Some(147.0),
            friend: &|_| Some(friend(146.5)),
        };
        let open = registry.handle(&join, &here, 60_000, &world).unwrap();
        assert_eq!(open.relocation.unwrap().position[1], 147.5);
    }

    #[test]
    fn a_field_fight_closes_atlas_and_friend_travel_but_not_the_rescue() {
        let content = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let mut game = Gameplay::load(&content, None).unwrap();
        let here = CharacterState {
            guid: 1,
            zone_id: OPEN_WORLD_ZONE,
            position: [-1904.0, -42.0, 420.0],
            rotation: [0.0, 0.0, 1.0],
        };
        let join = [58, 0, 7, 0, 0, 0, 0, 0, 0, 0];
        let lend = || {
            Some(FriendLend::Go {
                guid: 7,
                position: [-1900.0, -42.0, 420.0],
            })
        };
        game.field_engaged = true;
        let out = game
            .handle_warp(&travel(155), &here, 10_000)
            .unwrap()
            .unwrap();
        assert!(out.relocation.is_none(), "no Atlas teleport out of a fight");
        game.lend_friend(lend());
        let out = game.handle_warp(&join, &here, 20_000).unwrap().unwrap();
        assert!(
            out.relocation.is_none(),
            "no friend teleport out of a fight"
        );
        let out = game.handle_warp(&[122, 0], &here, 30_000).unwrap().unwrap();
        assert!(out.relocation.is_some(), "the fall rescue stays open");
        game.field_engaged = false;
        game.lend_friend(lend());
        let out = game.handle_warp(&join, &here, 40_000).unwrap().unwrap();
        assert!(
            out.relocation.is_some(),
            "after the fight the same request moves"
        );
    }

    #[test]
    fn a_stone_click_in_a_field_fight_answers_the_atlas_refusal_and_opens_after_it() {
        let content = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let mut game = Gameplay::load(&content, None).unwrap();
        // 3 m from the Sanctuary Warpstone actor (`npc_spawns_extra.json` 100000033167).
        let here = CharacterState {
            guid: 1,
            zone_id: OPEN_WORLD_ZONE,
            position: [-1415.3, -27.0, 347.8],
            rotation: [0.0, 0.0, 1.0],
        };
        let click = [&[26, 0, 8, 0][..], &100000033167u64.to_le_bytes(), &[0]].concat();
        game.field_engaged = true;
        let out = game.handle_in_world(&click, &here, 10_000).unwrap();
        let line = WarpRejection::Blocked(piece_077::Blocker::Encounter).notice();
        assert_eq!(
            out.replies,
            [chat::system(line.unwrap())],
            "the Atlas teleport's own line"
        );
        let out = game
            .handle_warp(&travel(155), &here, 10_000)
            .unwrap()
            .unwrap();
        assert!(
            out.relocation.is_none(),
            "and the teleport itself stays closed"
        );
        game.field_engaged = false;
        let out = game.handle_in_world(&click, &here, 20_000).unwrap();
        let atlas = crate::base_ui::execute_script(ATLAS_SCRIPT, &[]).unwrap();
        assert_eq!(
            out.replies,
            [atlas],
            "after the fight the same click opens the Atlas"
        );
    }

    #[test]
    fn bad_content_refuses_the_load_and_a_missing_file_keeps_the_curated_hubs() {
        let empty = std::env::temp_dir().join(format!("osfr-travel-{}", std::process::id()));
        fs::create_dir_all(&empty).unwrap();
        let curated = registry(&empty).unwrap();
        assert!(
            rows(&curated).is_empty(),
            "the curated hubs list only found stones"
        );
        let parse = |text: &str| serde_json::from_str::<ContentFile>(text).map(from_content);
        let rules = r#""rules":{"attune_radius_m":30.0,"cooldown_ms":0,"stuck_cooldown_ms":0,"below_world_m":3.0,"login_below_world_m":250.0}"#;
        let point = |id: i32, kind: i32, x: &str| {
            format!(
                r#"{{"id":{id},"name":"A","name_id":1,"location_id":1,"position":[{x},0.0,0.0],"heading":0.0,"icon_id":1,"notification_type":{kind},"sub_name_id":1,"unknown_11":0,"breadcrumb_quest_id":0,"teleport_location_id":1,"atlas":"sacredgrove","unknown_15":0}}"#
            )
        };
        let file =
            |points: String| format!(r#"{{"schema_version":1,{rules},"points":[{points}]}}"#);
        assert!(parse(&file(point(1, 7, "1.0"))).unwrap().is_ok());
        for bad in [
            file(point(1, 6, "1.0")), // no warpstone
            file(format!("{},{}", point(1, 7, "1.0"), point(1, 7, "2.0"))), // duplicate id
            file(point(1, 7, "1e9")), // off the map
            file(point(1, 7, "1.0")).replace("30.0", "0.1"), // rule out of bounds
            file(point(1, 7, "1.0")).replace(r#""schema_version":1"#, r#""schema_version":2"#),
        ] {
            assert!(parse(&bad).is_ok_and(|r| r.is_err()), "{bad}");
        }
        fs::remove_dir_all(&empty).unwrap();
    }
}
