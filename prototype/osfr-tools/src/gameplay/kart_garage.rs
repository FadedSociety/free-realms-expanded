//! The Kart & Demo Derby Garage, first slice (sheet `kart-garage`): the Games row shows the offer
//! with the encounter's Kind at 12, and Go zones the player into `sg_racing_garage`. Kind 12 makes
//! the client start its garage as the room is ready (`00936d00` -> `GarageManager__Start`), so the
//! offer has to be gone by then. That asks for the vehicle (139/1), and the answer builds the kart
//! on the spot. Every car on show looks like the race's one kart or the derby's one car: those are
//! the only definitions we hold. Parts are bookkeeping for now, as a race still sends its own kart.
use super::*;
use crate::inventory::Inventory;
use crate::kart_gateway::GarageParts;
use crate::startup_selfdata::SelfData;
use crate::vehicle_parts::{self, Request, Vehicle, DECAL, LOGIN_TAIL, SLOTS};
use std::sync::atomic::{AtomicI32, Ordering};

/// The Games window row: it joins by its own id, as ContentId 121 is a dead row.
pub(super) const GARAGE_ROW: u32 = 1079;
/// (R) The ids the row itself shows, and the race's icon.
const TEXT: [u32; 6] = [393_460, 9_870, 403_317, 1, 0, 0];
/// (R) Where the room's floor is, from the scene's bounds.
const SPAWN: [f32; 4] = [250.0, 8.67, 250.0, 1.0];
/// Locale 407742, for a player with no car.
const NO_CAR: &str = "Entering the garage isn't going to do you much good without a car now, is \
                      it! Come back once you've become a Kart Driver or a Demolition Derby Driver \
                      and the garage will be all yours.";
/// (R) The first car of each driving job: the capture's first two, guids 66-72 and 77-80.
const FIRST_CARS: [(i32, i32); 2] = [(48, 48316), (49, 48317)];
/// The instance ids a visit takes, out of the other games' ranges.
static NEXT_INSTANCE: AtomicI32 = AtomicI32::new(64_000_001);

#[derive(Clone, Debug)]
pub(super) struct Garage {
    pub(super) return_to: CharacterState,
    pub(super) scene: crate::zone_transition::ZoneTransition,
    parts: GarageParts,
    instance: i32,
    /// The zone-ready packets, once the room has loaded.
    pending: Option<Vec<Vec<u8>>>,
    /// Go was pressed: the player is in the room, not on the offer.
    pub(super) entered: bool,
    /// The type on show, 1 kart or 2 derby; the 139/2 filter is this plus 8.
    showing: u32,
    /// The derby car's definition; empty without derby content, and then there is no derby tab.
    derby: Vec<u8>,
}

/// A race's captured 41/114 as the garage's: Kind 12 and the row's own text. The Kind follows the
/// record's two lists (`EncounterDetailsCommon__ReadRecord`): the race's reads 2, a zero word, then
/// bytes 1, 0, 0; two words on come title, icon, description, difficulty, profile and game type.
fn garage_frame(captured: &[u8], instance: i32, guid: u64) -> io::Result<Vec<u8>> {
    const RACE: [u8; 9] = [2, 0, 0, 0, 0, 0, 0, 0, 1];
    let mut frame = captured.to_vec();
    let kind = (frame.windows(RACE.len()).position(|bytes| bytes == RACE))
        .ok_or_else(|| invalid("the race's details lost their Kind"))?;
    if frame.len() < kind + 19 + 4 * TEXT.len() {
        return Err(invalid("the race's details are cut short"));
    }
    frame[8..12].copy_from_slice(&instance.to_le_bytes());
    frame[32..40].copy_from_slice(&guid.to_le_bytes());
    frame[kind..kind + 4].copy_from_slice(&12_u32.to_le_bytes());
    for (slot, value) in TEXT.iter().enumerate() {
        let at = kind + 19 + slot * 4;
        frame[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    Ok(frame)
}

/// Grants a vehicle and its kit; returns the packets and its slots.
fn grant_kit(
    inventory: &mut Inventory,
    vehicle: &Vehicle,
) -> io::Result<(Vec<Vec<u8>>, [u32; SLOTS])> {
    let mut replies = inventory.grant(0, vehicle.item, 1)?.replies;
    let mut slots = [0; SLOTS];
    for &part in &vehicle.kit {
        let grant = inventory.grant(0, part, 1)?;
        let slot = (inventory.definition_slot(part).map(|slot| slot as usize))
            .filter(|slot| *slot < SLOTS)
            .ok_or_else(|| invalid("a kit part has no vehicle slot"))?;
        let (id, _) = *grant
            .granted
            .first()
            .ok_or_else(|| invalid("a kit part was not granted"))?;
        slots[slot] = id as u32;
        replies.extend(grant.replies);
    }
    Ok((replies, slots))
}

impl Gameplay {
    /// A driving job's first car, unless the player has a car of that type. A full bag leaves it
    /// for the next login's repair: a job must not fail on it.
    pub(super) fn grant_first_car(&mut self, job: i32, out: &mut Outcome) {
        let Some(&(_, item)) = FIRST_CARS.iter().find(|(driver, _)| *driver == job) else {
            return;
        };
        let Some((table, list)) = self.kart_entry.as_mut().and_then(|kart| kart.garage()) else {
            return;
        };
        let Some(vehicle) = table.by_item(item) else {
            return;
        };
        let owned = |car: &vehicle_parts::Loadout| {
            table
                .by_item(car.item)
                .is_some_and(|mine| mine.filter == vehicle.filter)
        };
        if list.cars.iter().any(owned) {
            return;
        }
        let mut staged = self.inventory.clone();
        let (replies, slots) = match grant_kit(&mut staged, vehicle) {
            Ok(granted) => granted,
            Err(error) => {
                eprintln!("first car {item} not granted: {error}");
                return;
            }
        };
        self.inventory = staged;
        list.add(vehicle, slots);
        out.replies.extend(replies);
        out.replies
            .push(vehicle_parts::refresh(&list.encode(&table)));
        out.changed = true;
    }

    /// On entering the world, for a driver from before cars or with a bag that was full.
    pub(super) fn repair_first_cars(&mut self, out: &mut Outcome) {
        for (job, _) in FIRST_CARS {
            if (self.profiles.as_ref()).is_some_and(|p| p.state().unlocked.contains(&job)) {
                self.grant_first_car(job, out);
            }
        }
    }

    /// The character's vehicles in the self packet, in place of the capture's. Without a vehicle
    /// table, or on a template that does not hold them there, the template stands.
    pub fn write_self_vehicles(&self, data: &mut SelfData) {
        let vehicles = self
            .kart_entry
            .as_ref()
            .and_then(|kart| kart.login_vehicles());
        if let Some((list, active)) =
            vehicles.filter(|_| vehicle_parts::tail_holds_list(&data.post_effect_tail))
        {
            data.post_effect_tail.splice(LOGIN_TAIL, list);
            data.account.active_vehicle_loadouts = active;
        }
    }

    /// The car on show: 139/2, then its decal, as the rebuild drops it.
    fn vehicle_config(&self) -> Vec<Vec<u8>> {
        let Some(garage) = self.garage.as_ref() else {
            return Vec::new();
        };
        let list = self.kart_entry.as_ref().map(|kart| &kart.state().loadouts);
        let chosen = list.and_then(|list| list.chosen(garage.showing));
        let definition = match garage.showing {
            2 => &garage.derby,
            _ => &garage.parts.vehicle,
        };
        let id = chosen.map_or(0, |car| car.id);
        let mut out = vec![vehicle_parts::config(definition, id, garage.showing + 8)];
        let decal = chosen.map_or(0, |car| car.slots[DECAL]);
        let item = self
            .inventory
            .state()
            .items
            .iter()
            .find(|item| item.id as u32 == decal);
        if let Some(look) = item.and_then(|item| self.inventory.look(item.definition_id)) {
            out.push(vehicle_parts::decal(&look.texture));
        }
        out
    }

    /// 139/5: every race and derby offer's picker sends this too, so it is saved wherever it comes
    /// from. In the garage it also changes what is on show; the client then asks with 139/1.
    fn select_vehicle(&mut self, id: u32, kind: u32) -> Outcome {
        let mut out = Outcome::default();
        let shown =
            (self.garage.as_ref()).is_none_or(|garage| kind != 2 || !garage.derby.is_empty());
        let Some((table, list)) = self.kart_entry.as_mut().and_then(|kart| kart.garage()) else {
            return out;
        };
        if shown && list.select(&table, id, kind) {
            out.changed = true;
            if let Some(garage) = self.garage.as_mut() {
                garage.showing = kind;
            }
        }
        out
    }

    /// Writes one slot of the chosen car: 137/2, then the rebuilt car (R: that order).
    fn set_slot(&mut self, slot: u32, item: u32) -> Option<Outcome> {
        let kind = self.garage.as_ref()?.showing;
        let (_, list) = self.kart_entry.as_mut()?.garage()?;
        let id = list.chosen(kind)?.id;
        let car = list.cars.iter_mut().find(|car| car.id == id)?;
        *car.slots.get_mut(slot as usize)? = item;
        let mut out = Outcome {
            changed: true,
            ..Outcome::default()
        };
        out.replies.push(vehicle_parts::slot_update(id, slot, item));
        out.replies.extend(self.vehicle_config());
        Some(out)
    }

    /// 139/3: a held part, in the slot its definition names, on a vehicle whose classes take it.
    fn fit_part(&mut self, guid: u32) -> Option<Outcome> {
        let items = &self.inventory.state().items;
        let definition = items
            .iter()
            .find(|item| item.id as u32 == guid)?
            .definition_id;
        let slot = u32::try_from(self.inventory.definition_slot(definition)?).ok()?;
        let class = self.inventory.definition_class(definition)?;
        let kind = self.garage.as_ref()?.showing;
        let (table, list) = self.kart_entry.as_mut()?.garage()?;
        let vehicle = table.by_item(list.chosen(kind)?.item)?;
        vehicle_parts::fits(vehicle, slot, class).then(|| self.set_slot(slot, guid))?
    }

    /// A 139 request from inside the garage; a refused one is answered with nothing.
    fn garage_request(&mut self, request: Request) -> Outcome {
        let answer = match request {
            Request::Config => Some(Outcome {
                replies: self.vehicle_config(),
                ..Outcome::default()
            }),
            Request::Equip(guid) => self.fit_part(guid),
            Request::Remove(slot) if vehicle_parts::removable(slot) => self.set_slot(slot, 0),
            Request::Remove(_) | Request::Select { .. } => None,
        };
        answer.unwrap_or_default()
    }

    /// Offers the garage to `character`, keeping the world pose for the way out.
    pub(super) fn begin_garage(
        &mut self,
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let parts = (self
            .kart_entry
            .as_ref()
            .and_then(|kart| kart.garage_parts()))
        .ok_or_else(|| invalid("the garage needs the kart content"))?;
        let list = self.kart_entry.as_mut().and_then(|kart| kart.garage());
        if list.as_ref().is_some_and(|(_, list)| list.is_empty()) {
            out.replies.push(chat::system(NO_CAR));
            return Ok(());
        }
        let showing = if list.is_some_and(|(_, list)| list.chosen(1).is_none()) {
            2
        } else {
            1
        };
        let instance = NEXT_INSTANCE.fetch_add(1, Ordering::Relaxed);
        let mut scene = parts.scene.clone();
        scene.scene = "sg_racing_garage".to_owned();
        scene.position = SPAWN;
        scene.id = instance;
        let guid = character.guid;
        out.replies
            .push(garage_frame(&parts.loading, instance, guid)?);
        let header = |sub: u8| {
            let mut packet = vec![41, 0, sub, 0];
            packet.extend(parts.encounter_id.to_le_bytes());
            packet.extend(instance.to_le_bytes());
            packet
        };
        // The visit has to outlive the staging the Games row runs this in.
        out.changed = true;
        // Go stays on "Loading" until the zone is called ready, so that comes with the offer, and
        // again once the room has loaded. 41/2 waits for Go: sent here, the client quits at once.
        out.replies.push(header(107));
        let pending = Some(vec![header(107)]);
        self.garage = Some(Box::new(Garage {
            return_to: character.clone(),
            scene,
            parts,
            instance,
            pending,
            entered: false,
            showing,
            derby: self
                .derby
                .as_ref()
                .map(|derby| derby.garage_vehicle())
                .unwrap_or_default(),
        }));
        Ok(())
    }

    /// After the room loads: the zone-ready packets, once.
    pub(super) fn garage_loaded(&mut self) -> Option<Vec<Vec<u8>>> {
        let garage = self.garage.as_mut()?;
        garage.entered.then(|| garage.pending.take()).flatten()
    }

    /// A packet while the garage is open (`None` otherwise). Go sends the race's launch pair (the
    /// second clears the activity id) and zones in, 139/1 the vehicle, and the way out zones back
    /// from the room; off the offer there is nowhere to go back from.
    pub(super) fn handle_garage(&mut self, inner: &[u8]) -> io::Result<Option<Outcome>> {
        let request = vehicle_parts::request(inner);
        if let Some(Request::Select { id, kind }) = request {
            return Ok(Some(self.select_vehicle(id, kind)));
        }
        let Some(garage) = self.garage.as_mut() else {
            return Ok(None);
        };
        let mut out = Outcome::default();
        // A race's encounter packets are not ours: each names its instance.
        let ours = |at: usize| inner.get(at..at + 4) == Some(&garage.instance.to_le_bytes()[..]);
        if inner.starts_with(&[41, 0, 108, 0]) {
            if !ours(8) {
                return Ok(None);
            }
            // A repeated Go would zone in again, with no zone-ready left to follow.
            if garage.entered {
                return Ok(Some(out));
            }
            let guid = garage.return_to.guid;
            let launch = garage_frame(&garage.parts.ready, garage.instance, guid)?;
            let mut enter = vec![41, 0, 2, 0];
            enter.extend(garage.parts.encounter_id.to_le_bytes());
            enter.extend(garage.instance.to_le_bytes());
            enter.extend([0; 9]);
            let mut ready = launch.clone();
            let activity = ready.len() - 15;
            ready[activity..activity + 4].fill(0);
            out.replies.extend([launch, enter, ready]);
            out.scene_zoning = Some(garage.scene.clone());
            out.changed = true;
            garage.entered = true;
            return Ok(Some(out));
        }
        if let Some(request) = request.filter(|_| garage.entered) {
            return Ok(Some(self.garage_request(request)));
        }
        if inner.starts_with(&[139, 0]) {
            return Ok(Some(out));
        }
        if (inner.starts_with(&[41, 0, 109, 0]) && ours(8)) || inner == [26, 0, 42, 0] {
            let mut leave = vec![41, 0, 121, 0];
            leave.extend(garage.parts.encounter_id.to_le_bytes());
            leave.extend(garage.instance.to_le_bytes());
            leave.push(0);
            let back = garage.entered.then(|| garage.return_to.clone());
            out.replies.push(leave);
            if let Some(back) = back {
                out.scene_zoning = Some(self.world_zoning(&back)?);
                out.changed = true;
            }
            self.garage = None;
            return Ok(Some(out));
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn captured(key: &str) -> Vec<u8> {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/content/kart_entry.json");
        let content: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        serde_json::from_value(content[key].clone()).unwrap()
    }

    /// Both of the race's captured offers take the garage's Kind and text in the same place.
    #[test]
    fn the_garage_rewrites_the_races_offer() {
        for key in ["details_loading", "details_ready"] {
            let race = captured(key);
            let frame = garage_frame(&race, 77, 0x1234).unwrap();
            assert_eq!(frame.len(), race.len(), "{key}");
            assert_eq!(frame[8..12], 77_i32.to_le_bytes());
            assert_eq!(frame[32..40], 0x1234_u64.to_le_bytes());
            let kind = (frame
                .windows(9)
                .position(|w| w == [12, 0, 0, 0, 0, 0, 0, 0, 1]))
            .expect("Kind 12");
            assert_eq!(
                frame[kind + 19..kind + 23],
                393_460_u32.to_le_bytes(),
                "{key}"
            );
            assert_eq!(
                frame[kind + 39..kind + 43],
                0_u32.to_le_bytes(),
                "no game type"
            );
        }
        assert!(garage_frame(&[0; 40], 1, 1).is_err(), "no Kind to rewrite");
    }

    /// A scratch content set: the reference campaign, the release's items and our kart, derby
    /// and vehicle files.
    struct Fixture(std::path::PathBuf);

    impl Fixture {
        fn new(name: &str) -> Self {
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
            let path = std::env::temp_dir().join(format!("osfr-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).unwrap();
            let reference =
                root.join("../../../lab/references/rust-opening-cooking-original-assets");
            for entry in std::fs::read_dir(reference).unwrap() {
                let file = entry.unwrap().path();
                if file.is_file() {
                    std::fs::copy(&file, path.join(file.file_name().unwrap())).unwrap();
                }
            }
            for name in [
                "inventory.json",
                "item-table.json",
                "item-table-supplement.json",
            ] {
                std::fs::copy(root.join("data/release").join(name), path.join(name)).unwrap();
            }
            for name in ["kart_entry.json", "vehicles.json"] {
                std::fs::copy(root.join("data/content").join(name), path.join(name)).unwrap();
            }
            Self(path)
        }

        /// A character who has unlocked `jobs`.
        fn game(&self, jobs: &[i32]) -> Gameplay {
            let mut saved = Gameplay::load(&self.0, None).unwrap().state();
            let profiles = saved.profiles.as_mut().unwrap();
            profiles.unlocked.extend(jobs);
            let mut game = Gameplay::load(&self.0, Some(saved)).unwrap();
            game.repair_first_cars(&mut Outcome::default());
            game
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn who() -> CharacterState {
        CharacterState {
            guid: 9,
            zone_id: 1,
            position: [1.0, 2.0, 3.0],
            rotation: [0.0; 3],
        }
    }

    fn list(game: &Gameplay) -> &vehicle_parts::Loadouts {
        &game.kart_entry.as_ref().unwrap().state().loadouts
    }

    fn enter(game: &mut Gameplay) {
        game.begin_garage(&who(), &mut Outcome::default()).unwrap();
        let instance = game.garage.as_ref().unwrap().instance;
        let mut go = vec![41, 0, 108, 0, 0, 0, 0, 0];
        go.extend(instance.to_le_bytes());
        game.handle_garage(&go).unwrap().unwrap();
    }

    fn equip(guid: u32) -> Vec<u8> {
        [&[139, 0, 3, 0], &guid.to_le_bytes()[..]].concat()
    }

    /// A driver gets the car and kit once, however often the repair runs, and a new character
    /// gets nothing and no way in.
    #[test]
    fn a_driver_owns_a_car_and_nobody_else_gets_in() {
        let fixture = Fixture::new("first-car");
        let mut game = fixture.game(&[48]);
        let car = list(&game).cars[0].clone();
        assert_eq!(car.item, 48316);
        assert_eq!(list(&game).active, [car.id, 0]);
        let filled = car.slots.iter().filter(|slot| **slot != 0).count();
        assert_eq!(filled, 6, "chassis to springs");
        let held = |game: &Gameplay, guid: u32| {
            game.inventory
                .state()
                .items
                .iter()
                .any(|item| item.id as u32 == guid)
        };
        assert!(car.slots[..6].iter().all(|guid| held(&game, *guid)));
        let items = game.inventory.state().items.len();
        let mut again = Outcome::default();
        game.repair_first_cars(&mut again);
        assert!(!again.changed && again.replies.is_empty());
        assert_eq!(
            (list(&game).cars.len(), game.inventory.state().items.len()),
            (1, items)
        );

        let mut stranger = fixture.game(&[]);
        assert!(list(&stranger).is_empty());
        let mut out = Outcome::default();
        stranger.begin_garage(&who(), &mut out).unwrap();
        assert!(stranger.garage.is_none() && !out.changed);
        assert_eq!(out.replies.len(), 1, "the refusal");
    }

    /// In the room: 139/1 shows the chosen car, a part of the wrong class or the chassis is
    /// refused, and a shared part goes in and comes out again.
    #[test]
    fn parts_go_in_and_come_out_of_the_chosen_car() {
        let fixture = Fixture::new("parts");
        let mut game = fixture.game(&[48, 49]);
        enter(&mut game);
        let id = list(&game).active[0];
        let config = game
            .handle_garage(&[139, 0, 1, 0])
            .unwrap()
            .unwrap()
            .replies;
        let tail = &config[0][config[0].len() - 8..];
        assert_eq!(tail[..4], id.to_le_bytes());
        assert_eq!(tail[4..], 9_u32.to_le_bytes());

        let chassis = list(&game).cars[0].slots[0];
        assert!(game
            .handle_garage(&equip(chassis))
            .unwrap()
            .unwrap()
            .replies
            .is_empty());
        let derby_part = list(&game).cars[1].slots[1];
        assert!(game
            .handle_garage(&equip(derby_part))
            .unwrap()
            .unwrap()
            .replies
            .is_empty());
        assert!(game
            .handle_garage(&equip(999_999))
            .unwrap()
            .unwrap()
            .replies
            .is_empty());

        let decal = game.inventory.grant(0, 9240, 1).unwrap().granted[0].0 as u32;
        let out = game.handle_garage(&equip(decal)).unwrap().unwrap();
        assert!(out.changed);
        assert_eq!(out.replies[0], vehicle_parts::slot_update(id, 8, decal));
        assert_eq!(out.replies[1][..4], [139, 0, 2, 0]);
        assert_eq!(out.replies[2], vehicle_parts::decal("catfood"));
        assert_eq!(list(&game).cars[0].slots[DECAL], decal);

        let remove = |slot: u32| [&[139, 0, 6, 0], &slot.to_le_bytes()[..]].concat();
        assert!(
            game.handle_garage(&remove(1))
                .unwrap()
                .unwrap()
                .replies
                .is_empty(),
            "no None card"
        );
        let out = game.handle_garage(&remove(8)).unwrap().unwrap();
        assert_eq!(out.replies[0], vehicle_parts::slot_update(id, 8, 0));
        assert_eq!(out.replies.len(), 2, "no decal left");
        assert_eq!(list(&game).cars[0].slots[DECAL], 0);
    }

    /// A race's picker chooses a car outside the garage; only a car of that type counts.
    #[test]
    fn a_choice_is_saved_from_anywhere() {
        let fixture = Fixture::new("select");
        let mut game = fixture.game(&[48, 49]);
        let derby = list(&game).active[1];
        let select = |id: u32, kind: u32| {
            [
                &[139, 0, 5, 0],
                &id.to_le_bytes()[..],
                &kind.to_le_bytes()[..],
            ]
            .concat()
        };
        assert!(
            !game
                .handle_garage(&select(derby, 1))
                .unwrap()
                .unwrap()
                .changed
        );
        assert!(
            game.handle_garage(&select(derby, 2))
                .unwrap()
                .unwrap()
                .changed
        );
        assert!(
            game.handle_garage(&[139, 0, 1, 0]).unwrap().is_none(),
            "no garage, no config"
        );
    }

    /// The login list has the character's car, in the place the template's six were.
    #[test]
    fn the_self_packet_carries_the_characters_list() {
        let fixture = Fixture::new("login");
        let game = fixture.game(&[48]);
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/content/startup/self_data.json"
        );
        let mut data: SelfData =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let template = data.post_effect_tail.clone();
        game.write_self_vehicles(&mut data);
        let (list, active) = game.kart_entry.as_ref().unwrap().login_vehicles().unwrap();
        assert_eq!(data.account.active_vehicle_loadouts, active);
        let tail = &data.post_effect_tail;
        let end = LOGIN_TAIL.start + list.len();
        assert_eq!(tail[..LOGIN_TAIL.start], template[..LOGIN_TAIL.start]);
        assert_eq!(tail[LOGIN_TAIL.start..end], list);
        assert_eq!(tail[end..], template[LOGIN_TAIL.end..]);
    }
}
