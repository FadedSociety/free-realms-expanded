//! The Kart & Demo Derby Garage, first slice (sheet `kart-garage`): the Games row shows the offer
//! with the encounter's Kind at 12, and Go zones the player into `sg_racing_garage`. Kind 12 makes
//! the client start its garage as the room is ready (`00936d00` -> `GarageManager__Start`), so the
//! offer has to be gone by then. That asks for the vehicle (139/1), and the answer builds the kart
//! on the spot. Loadouts and parts come later.
use super::*;
use crate::kart_gateway::GarageParts;
use std::sync::atomic::{AtomicI32, Ordering};

/// The Games window row: it joins by its own id, as ContentId 121 is a dead row.
pub(super) const GARAGE_ROW: u32 = 1079;
/// (R) The ids the row itself shows, and the race's icon.
const TEXT: [u32; 6] = [393_460, 9_870, 403_317, 1, 0, 0];
/// (R) Where the room's floor is, from the scene's bounds.
const SPAWN: [f32; 4] = [250.0, 8.67, 250.0, 1.0];
/// The type filter 139/2 ends on: 9 shows karts, 10 derby cars (`Ui.GetActiveVehicleLoadoutType`).
const KARTS: u32 = 9;
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

/// VehicleConfig (139/2; reader `009d88b0`, dropped unless it parses to the last byte): the
/// definition, no part textures, the selected loadout and the type filter.
fn vehicle_config(vehicle: &[u8]) -> Vec<u8> {
    let mut out = vec![139, 0, 2, 0];
    out.extend(vehicle);
    for word in [0, 0, KARTS] {
        out.extend(word.to_le_bytes());
    }
    out
}

impl Gameplay {
    /// Offers the garage to `character`, keeping the world pose for the way out.
    pub(super) fn begin_garage(
        &mut self,
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let parts = (self.kart_entry.as_ref().and_then(|kart| kart.garage_parts()))
            .ok_or_else(|| invalid("the garage needs the kart content"))?;
        let instance = NEXT_INSTANCE.fetch_add(1, Ordering::Relaxed);
        let mut scene = parts.scene.clone();
        scene.scene = "sg_racing_garage".to_owned();
        scene.position = SPAWN;
        scene.id = instance;
        let guid = character.guid;
        out.replies.push(garage_frame(&parts.loading, instance, guid)?);
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
        if inner == [139, 0, 1, 0] && garage.entered {
            out.replies.push(vehicle_config(&garage.parts.vehicle));
            return Ok(Some(out));
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
            let kind = (frame.windows(9).position(|w| w == [12, 0, 0, 0, 0, 0, 0, 0, 1]))
                .expect("Kind 12");
            assert_eq!(frame[kind + 19..kind + 23], 393_460_u32.to_le_bytes(), "{key}");
            assert_eq!(frame[kind + 39..kind + 43], 0_u32.to_le_bytes(), "no game type");
        }
        assert!(garage_frame(&[0; 40], 1, 1).is_err(), "no Kind to rewrite");
    }

    /// The config ends on the three words the reader wants after the definition.
    #[test]
    fn the_vehicle_config_ends_on_the_type_filter() {
        let out = vehicle_config(&[1, 2, 3]);
        assert_eq!(out[..7], [139, 0, 2, 0, 1, 2, 3]);
        assert_eq!(out.len(), 7 + 12);
        assert_eq!(out[15..], KARTS.to_le_bytes());
    }
}
