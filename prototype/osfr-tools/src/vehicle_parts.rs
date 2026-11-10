//! Vehicle loadouts: what the garage edits, the login self packet carries, and 137 and 139 move
//! (sheet `kart-garage` section 5). Field order is the client's: the record `008e8d30`, the whole
//! list `0092afb0`, family 137 `00944840`, family 139 `009d89c0`. A slot holds an item's instance
//! id, not its definition, and a row shows only while its chassis is in the bag.
use crate::kart_entry::ProxiedVehicle;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, io, path::Path, sync::Arc};

pub const SLOTS: usize = 16;
/// A look's attribute block plus the race's 33/11 tail (58 bytes) must fit the entry handler's
/// `MAX_ATTRIBUTE_BLOCK_BYTES`, or a saved race could not load; the biggest capture is 6,078.
const MAX_LOOK_BLOCK_BYTES: usize = crate::kart_entry::MAX_ATTRIBUTE_BLOCK_BYTES - 64;
/// Where the login self packet's tail holds the list (`post_effect_tail`).
pub const LOGIN_TAIL: std::ops::Range<usize> = 1116..1730;
/// How the template's list starts: next id 7, six loadouts.
const LOGIN_HEAD: [u8; 8] = [7, 0, 0, 0, 6, 0, 0, 0];
/// The slots a part can go in: `VehicleGarage.AllowedSlots`, 0-based. The chassis stays.
const FITTED: [u32; 7] = [1, 2, 3, 4, 5, 6, 8];
/// The slots with a None card: A, B and the decal.
const REMOVABLE: [u32; 3] = [5, 6, 8];
pub const DECAL: usize = 8;

/// One row of `data/content/vehicles.json` (`lab/tools/vehicle_table.py`).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Vehicle {
    pub item: i32,
    pub vehicle: u32,
    pub label: String,
    /// 9 a kart, 10 a derby car: the client's type filter.
    #[serde(rename = "type")]
    pub filter: u32,
    pub name: u32,
    pub description: u32,
    pub icon: u32,
    pub tint: u32,
    pub classes: Vec<i32>,
    /// The parts it comes with, the chassis first.
    pub kit: Vec<i32>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Vehicles {
    vehicles: Vec<Vehicle>,
}

impl Vehicles {
    /// `None` when the content set has no `vehicles.json`: logins keep the template's list.
    pub fn load(directory: &Path) -> io::Result<Option<Arc<Self>>> {
        let bytes = match fs::read(directory.join("vehicles.json")) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        let table: Self = serde_json::from_slice(&bytes).map_err(io::Error::other)?;
        if table
            .vehicles
            .iter()
            .any(|v| v.kit.is_empty() || !(9..=10).contains(&v.filter))
        {
            return Err(io::Error::other("a vehicle has no kit or an unknown type"));
        }
        Ok(Some(Arc::new(table)))
    }

    pub fn by_item(&self, item: i32) -> Option<&Vehicle> {
        self.vehicles.iter().find(|vehicle| vehicle.item == item)
    }

    /// Every vehicle item of the table.
    pub fn items(&self) -> impl Iterator<Item = i32> + '_ {
        self.vehicles.iter().map(|vehicle| vehicle.item)
    }
}

fn read_optional(path: &Path) -> io::Result<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

/// `data/content/vehicle_looks.json`: which captured car each vehicle item shows in the garage, a
/// race and a derby. A definition is a `ReadVehicleDefinition` blob (`009d7b50`): u32 name
/// length, the name, an f32, then the attribute block. The item rows are (R), the blobs captured.
#[derive(Debug, Deserialize)]
pub struct Looks {
    schema_version: u32,
    definitions: BTreeMap<String, Vec<u8>>,
    vehicles: Vec<LookRow>,
}

#[derive(Debug, Deserialize)]
struct LookRow {
    item: i32,
    look: String,
}

impl Looks {
    /// `None` without the file: every car keeps the race's or derby's default look.
    pub fn load(directory: &Path) -> io::Result<Option<Arc<Self>>> {
        let Some(bytes) = read_optional(&directory.join("vehicle_looks.json"))? else {
            return Ok(None);
        };
        let table: Self = serde_json::from_slice(&bytes).map_err(io::Error::other)?;
        let sound = |definition: &Vec<u8>| {
            split_definition(definition).is_some_and(|(_, value, block)| {
                value.is_finite() && !block.is_empty() && block.len() <= MAX_LOOK_BLOCK_BYTES
            })
        };
        if table.schema_version != 1
            || !table.definitions.values().all(sound)
            || (table.vehicles.iter()).any(|row| !table.definitions.contains_key(&row.look))
        {
            return Err(io::Error::other(
                "vehicle_looks.json names a look it does not hold",
            ));
        }
        Ok(Some(Arc::new(table)))
    }

    /// The definition a vehicle item shows; `None` keeps the default car.
    pub fn definition(&self, item: i32) -> Option<&[u8]> {
        let row = self.vehicles.iter().find(|row| row.item == item)?;
        self.definitions.get(&row.look).map(Vec::as_slice)
    }
}

/// A definition's three parts: the asset name, the f32 after it (400.0 in every capture) and the
/// attribute block, as `ProxiedVehicle` holds them.
pub fn split_definition(definition: &[u8]) -> Option<(&str, f32, &[u8])> {
    let length = u32::from_le_bytes(definition.get(..4)?.try_into().ok()?) as usize;
    let name = std::str::from_utf8(definition.get(4..4 + length)?).ok()?;
    let value = f32::from_le_bytes(definition.get(4 + length..8 + length)?.try_into().ok()?);
    Some((name, value, definition.get(8 + length..)?))
}

/// Gives a race's entry vehicle another look: the definition replaces everything before the
/// 33/11 tail (owner guid on), which starts at `guid_offset`. Returns the tail's new offset.
pub fn apply_look(
    vehicle: &mut ProxiedVehicle,
    definition: &[u8],
    guid_offset: usize,
) -> Option<usize> {
    let (name, value, block) = split_definition(definition)?;
    if !value.is_finite() || block.is_empty() {
        return None;
    }
    let tail = vehicle.attribute_block.get(guid_offset..)?;
    vehicle.attribute_block = [block, tail].concat();
    vehicle.body_asset = name.to_owned();
    vehicle.first_attribute = value;
    Some(block.len())
}

/// `data/content/kart_garage.json`: the garage's tunable values. Without the file there are no
/// mechanic doors, no first cars and no room to zone into: the Games row refuses.
#[derive(Clone, Debug, Deserialize)]
pub struct GarageContent {
    schema_version: u32,
    pub menu_string_id: u32,
    /// (R) Where the kart stands: the room's lift (`prop_placements_2014.tsv`).
    pub spawn: [f32; 3],
    pub first_cars: Vec<FirstCar>,
    pub gate_npcs: Vec<GateNpc>,
}

/// (R) A driving job's first car: the capture's first two loadouts.
#[derive(Clone, Debug, Deserialize)]
pub struct FirstCar {
    pub job: i32,
    pub item: i32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GateNpc {
    pub npc_id: u64,
}

impl GarageContent {
    /// `None` without the file.
    pub fn load(directory: &Path) -> io::Result<Option<Arc<Self>>> {
        let Some(bytes) = read_optional(&directory.join("kart_garage.json"))? else {
            return Ok(None);
        };
        let content: Self = serde_json::from_slice(&bytes).map_err(io::Error::other)?;
        if content.schema_version != 1
            || content.spawn.iter().any(|value| !value.is_finite())
            || content.first_cars.len() > 8
            || content.gate_npcs.len() > 64
        {
            return Err(io::Error::other("kart_garage.json is out of bounds"));
        }
        Ok(Some(Arc::new(content)))
    }

    /// The first car of a driving job, when it has one.
    pub fn first_car(&self, job: i32) -> Option<i32> {
        (self.first_cars.iter()).find_map(|car| (car.job == job).then_some(car.item))
    }

    pub fn gate_ids(&self) -> Vec<u64> {
        self.gate_npcs.iter().map(|gate| gate.npc_id).collect()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Loadout {
    pub id: u32,
    /// The vehicle's item definition, its row in the table.
    pub item: i32,
    pub slots: [u32; SLOTS],
}

/// A character's vehicles. Nothing is saved until the first one.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Loadouts {
    pub next: u32,
    pub cars: Vec<Loadout>,
    /// The chosen loadout for each type, kart then derby.
    pub active: [u32; 2],
}

impl Default for Loadouts {
    fn default() -> Self {
        Self {
            next: 1,
            cars: Vec::new(),
            active: [0; 2],
        }
    }
}

impl Loadouts {
    pub fn is_empty(&self) -> bool {
        self.cars.is_empty()
    }

    /// Adds a car, chosen if it is the first of its type.
    pub fn add(&mut self, vehicle: &Vehicle, slots: [u32; SLOTS]) {
        let id = self.next;
        self.next += 1;
        self.cars.push(Loadout {
            id,
            item: vehicle.item,
            slots,
        });
        let active = &mut self.active[(vehicle.filter - 9) as usize];
        if *active == 0 {
            *active = id;
        }
    }

    /// The chosen car of a type (1 kart, 2 derby).
    pub fn chosen(&self, kind: u32) -> Option<&Loadout> {
        let id = *self.active.get(kind.checked_sub(1)? as usize)?;
        self.cars.iter().find(|car| car.id == id)
    }

    /// 139/5, from the garage or a race's offer: only a car of that type can be chosen.
    pub fn select(&mut self, table: &Vehicles, id: u32, kind: u32) -> bool {
        if !matches!(kind, 1 | 2) {
            return false;
        }
        let fits = self.cars.iter().any(|car| {
            car.id == id
                && table
                    .by_item(car.item)
                    .is_some_and(|v| v.filter == kind + 8)
        });
        if fits {
            self.active[kind as usize - 1] = id;
        }
        fits
    }

    /// The list as the self packet and 137/3 carry it.
    pub fn encode(&self, table: &Vehicles) -> Vec<u8> {
        let cars: Vec<_> = (self.cars.iter())
            .filter_map(|car| Some((car, table.by_item(car.item)?)))
            .collect();
        let mut out = Vec::with_capacity(8 + cars.len() * 106);
        words(&mut out, &[self.next, cars.len() as u32]);
        for (car, vehicle) in cars {
            words(&mut out, &[car.id, car.id, vehicle.vehicle, SLOTS as u32]);
            words(&mut out, &car.slots);
            let item = vehicle.item as u32;
            words(
                &mut out,
                &[
                    item,
                    vehicle.name,
                    vehicle.description,
                    vehicle.icon,
                    vehicle.tint,
                ],
            );
            out.push(0);
        }
        out
    }
}

fn words(out: &mut Vec<u8>, values: &[u32]) {
    for value in values {
        out.extend(value.to_le_bytes());
    }
}

/// Whether a self-packet tail holds the template's list where [`LOGIN_TAIL`] says.
pub fn tail_holds_list(tail: &[u8]) -> bool {
    tail.len() >= LOGIN_TAIL.end && tail[LOGIN_TAIL.start..].starts_with(&LOGIN_HEAD)
}

/// Whether a part may be fitted in `slot` of a vehicle: the slot takes parts, and the part's Class
/// is one the vehicle takes (`00c2cb80`).
pub fn fits(vehicle: &Vehicle, slot: u32, class: i32) -> bool {
    FITTED.contains(&slot) && vehicle.classes.contains(&class)
}

pub fn removable(slot: u32) -> bool {
    REMOVABLE.contains(&slot)
}

/// What the client asks of family 139.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Request {
    Config,
    Equip(u32),
    Select { id: u32, kind: u32 },
    Remove(u32),
}

/// Decodes a 139 request of exactly its length (`00b87080`-`00b87230`).
pub fn request(inner: &[u8]) -> Option<Request> {
    let word = |at: usize| Some(u32::from_le_bytes(inner.get(at..at + 4)?.try_into().ok()?));
    match (inner.get(..4)?, inner.len()) {
        ([139, 0, 1, 0], 4) => Some(Request::Config),
        ([139, 0, 3, 0], 8) => Some(Request::Equip(word(4)?)),
        ([139, 0, 5, 0], 12) => Some(Request::Select {
            id: word(4)?,
            kind: word(8)?,
        }),
        ([139, 0, 6, 0], 8) => Some(Request::Remove(word(4)?)),
        _ => None,
    }
}

/// VehicleConfig (139/2; reader `009d88b0`, dropped unless it parses to the last byte): the
/// definition, no part textures, the chosen loadout and the type filter.
pub fn config(definition: &[u8], active: u32, filter: u32) -> Vec<u8> {
    let mut out = vec![139, 0, 2, 0];
    out.extend(definition);
    words(&mut out, &[0, active, filter]);
    out
}

/// VehicleUpdateDecal (139/4): the decal's texture alias, after the 139/2 that rebuilt the car.
pub fn decal(alias: &str) -> Vec<u8> {
    let mut out = vec![139, 0, 4, 0];
    words(&mut out, &[alias.len() as u32]);
    out.extend(alias.as_bytes());
    out
}

/// 137/2: one slot of one loadout; 0 empties it. Family 137's subtype is one byte.
pub fn slot_update(id: u32, slot: u32, item: u32) -> Vec<u8> {
    let mut out = vec![137, 0, 2];
    words(&mut out, &[id, slot, item]);
    out
}

/// 137/3: the whole list, as after a new car.
pub fn refresh(list: &[u8]) -> Vec<u8> {
    let mut out = vec![137, 0, 3];
    out.extend(list);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> Arc<Vehicles> {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
        Vehicles::load(&dir).unwrap().expect("vehicles.json")
    }

    /// The capture's list, as the login tail holds it.
    fn captured() -> Vec<u8> {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/content/startup/self_data.json"
        );
        let data: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        let tail = data["post_effect_tail"].as_str().unwrap();
        let bytes: Vec<u8> = (0..tail.len() / 2)
            .map(|at| u8::from_str_radix(&tail[at * 2..at * 2 + 2], 16).unwrap())
            .collect();
        bytes[LOGIN_TAIL].to_vec()
    }

    /// Only the guids come from the capture: every other field of its six loadouts comes back
    /// from the table, byte for byte.
    #[test]
    fn the_table_rebuilds_the_captures_loadouts() {
        let bytes = captured();
        let word = |at: usize| u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
        let mut list = Loadouts {
            next: word(0),
            cars: Vec::new(),
            active: [3, 5],
        };
        let mut at = 8;
        for _ in 0..word(4) {
            let mut slots = [0; SLOTS];
            for (slot, value) in slots.iter_mut().enumerate() {
                *value = word(at + 16 + slot * 4);
            }
            list.cars.push(Loadout {
                id: word(at + 4),
                item: word(at + 16 + SLOTS * 4) as i32,
                slots,
            });
            at += 16 + SLOTS * 4 + 21;
        }
        assert_eq!(list.cars.len(), 6);
        assert_eq!(list.encode(&table()), bytes);
    }

    /// The two first cars, as the sheet reads them off the capture.
    #[test]
    fn the_first_cars_come_with_their_kits() {
        let table = table();
        let kart = table.by_item(48316).unwrap();
        assert_eq!((kart.vehicle, kart.filter), (6, 9));
        assert_eq!(kart.kit, [60649, 60820, 60821, 60822, 60823, 60824]);
        let derby = table.by_item(48317).unwrap();
        assert_eq!((derby.vehicle, derby.filter), (9, 10));
        assert_eq!(derby.kit, [60669, 60835, 60836]);
    }

    #[test]
    fn a_new_car_is_chosen_only_if_it_is_the_first_of_its_type() {
        let table = table();
        let mut list = Loadouts::default();
        list.add(table.by_item(48316).unwrap(), [0; SLOTS]);
        list.add(table.by_item(48318).unwrap(), [0; SLOTS]);
        list.add(table.by_item(48317).unwrap(), [0; SLOTS]);
        assert_eq!((list.active, list.next), ([1, 3], 4));
        assert!(list.select(&table, 2, 1));
        assert_eq!(list.chosen(1).unwrap().item, 48318);
        assert!(!list.select(&table, 3, 1), "a derby car is not a kart");
        assert!(!list.select(&table, 9, 1), "not owned");
        assert!(!list.select(&table, 2, 3), "no such type");
        assert!(!list.select(&table, 2, u32::MAX), "a client's word");
        assert_eq!(list.active, [2, 3]);
        assert_eq!(list.chosen(3), None);
    }

    #[test]
    fn requests_are_decoded_only_at_their_length() {
        assert_eq!(request(&[139, 0, 1, 0]), Some(Request::Config));
        assert_eq!(
            request(&[139, 0, 3, 0, 7, 0, 0, 0]),
            Some(Request::Equip(7))
        );
        let select = [139, 0, 5, 0, 3, 0, 0, 0, 2, 0, 0, 0];
        assert_eq!(request(&select), Some(Request::Select { id: 3, kind: 2 }));
        assert_eq!(
            request(&[139, 0, 6, 0, 5, 0, 0, 0]),
            Some(Request::Remove(5))
        );
        assert_eq!(request(&[139, 0, 3, 0, 7, 0, 0]), None);
        assert_eq!(request(&[139, 0, 1, 0, 0]), None);
        assert_eq!(request(&[139, 0, 2, 0]), None);
    }

    #[test]
    fn the_packets_carry_their_fields_in_the_readers_order() {
        let config = config(&[1, 2, 3], 4, 9);
        assert_eq!(config[..7], [139, 0, 2, 0, 1, 2, 3]);
        assert_eq!(config[7..], [0, 0, 0, 0, 4, 0, 0, 0, 9, 0, 0, 0]);
        assert_eq!(decal("ab"), [139, 0, 4, 0, 2, 0, 0, 0, b'a', b'b']);
        assert_eq!(
            slot_update(1, 2, 3),
            [137, 0, 2, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0]
        );
        assert_eq!(refresh(&[9]), [137, 0, 3, 9]);
    }

    #[test]
    fn parts_go_only_where_the_garage_offers_them() {
        let table = table();
        let kart = table.by_item(48316).unwrap();
        assert!(fits(kart, 1, 85), "its own engine");
        assert!(fits(kart, 8, 75), "a shared decal");
        assert!(!fits(kart, 0, 85), "the chassis stays");
        assert!(!fits(kart, 1, 76), "a derby part");
        assert!(removable(5) && !removable(1));
    }
}
