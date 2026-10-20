//! Decode and relay the native client's separate jump update (opcode 164).
//!
//! Client reader `008e5ba0` and handler `0090e470` establish the exact 40-byte
//! application body; writer `009127f0` serializes it field by field:
//! `u16 opcode, u64 guid, 3 x f32 position, 3 x f32 direction, u8 state,
//! u8 unknown, f32 vertical_velocity`. The client's sender `0092e670` stamps
//! the local actor id and its current jump motion. The GUID is server-bound
//! before relaying; position and direction still pass through
//! `movement::validate`, exactly like the opcode-125 position path.

use crate::movement::{MAX_POSITION_COMPONENT, MovementState, MovementUpdate};
use std::io;

/// Native packet id stored by constructor `008b3b70` (`[this+4] = 0xA4`).
pub const OPCODE: u16 = 164;
/// Native application body length from reader `008e5ba0` / writer `009127f0`.
pub const BODY_LEN: usize = 40;

/// (R) Relay bound for the client's vertical impulse, 10x the 10 u/s opcode-125
/// base speed. Sender `0092e670` supplies its own `+0x674` speed and the native
/// bounce path caps velocity itself (`BounceVelocityCap`, `sub_CEB120`); no
/// universal impulse was recovered, so finite input is clamped rather than
/// refused (a refused body would end the session).
pub const MAX_VERTICAL_SPEED: f32 = 100.0;

/// (R) Relay budget per character: about one relay per 100 ms, with a small burst
/// so a normal jump cadence (about one per 0.5 s) is never dropped. The gateway's
/// `Security` owns the `KeyedBuckets`; over budget only the extra 164 is dropped.
/// The native sender's 50 ms gate (`0092e670`) is client-owned, not a boundary.
pub const RELAY_RATE_PER_SECOND: f64 = 10.0;
/// (R) Allow a short burst of jumps before the budget drops extra relays.
pub const RELAY_BURST: f64 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JumpUpdate {
    /// Untrusted client position: checked finite and within `movement::MAX_POSITION_COMPONENT`;
    /// it then passes `movement::validate` exactly like opcode 125 before any relay.
    pub position: [f32; 3],
    /// Untrusted client direction; `movement::validate` normalizes it before relay.
    pub direction: [f32; 3],
    /// Client motion byte; relayed with its impulse, never replaced by a later
    /// opcode-125 pose state (native sender `0092e670` forces bit 2).
    pub state: u8,
    /// Preserved opaque client byte, relayed with its impulse.
    pub unknown: u8,
    /// Client vertical impulse; finite, clamped to `±MAX_VERTICAL_SPEED`, and
    /// relayed as the clamped value.
    pub vertical_velocity: f32,
}

/// The accepted 164's motion bytes travel with its impulse. Native sender
/// `0092e670` forces state bit 2 (`v22 |= 4`) and sets bit 0 when the horizontal
/// direction is zero; handler `0090e470` (at `0x0090e5f5`) applies the packet
/// velocity only when bit 2 is set. A later 125 in the same bundle supplies the
/// final position/direction but must not replace these bytes. A later application
/// child drops the impulse: it can relocate or zone the character.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JumpImpulse {
    pub vertical_velocity: f32,
    pub state: u8,
    pub unknown: u8,
}

impl JumpUpdate {
    pub fn movement(self) -> MovementUpdate {
        MovementUpdate {
            position: self.position,
            direction: self.direction,
            state: self.state,
            unknown: self.unknown,
        }
    }

    pub fn impulse(self) -> JumpImpulse {
        JumpImpulse {
            vertical_velocity: self.vertical_velocity,
            state: self.state,
            unknown: self.unknown,
        }
    }
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn checked_float(bytes: &[u8], offset: usize, what: &str) -> io::Result<f32> {
    let value = f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
    if !value.is_finite() || value.abs() > MAX_POSITION_COMPONENT {
        return Err(invalid(what));
    }
    Ok(value)
}

/// Decode exactly one untrusted opcode-164 body. `guid` is the session-bound
/// character; a body naming any other actor is refused, so a client cannot
/// publish motion on behalf of another player.
pub fn decode(body: &[u8], guid: u64) -> io::Result<JumpUpdate> {
    if body.len() != BODY_LEN {
        return Err(invalid("jump body must be exactly 40 bytes"));
    }
    if u16::from_le_bytes(body[..2].try_into().unwrap()) != OPCODE {
        return Err(invalid("jump body has the wrong opcode"));
    }
    if u64::from_le_bytes(body[2..10].try_into().unwrap()) != guid {
        return Err(invalid("jump selects another character"));
    }
    let mut position = [0.0; 3];
    let mut direction = [0.0; 3];
    for (index, value) in position.iter_mut().enumerate() {
        *value = checked_float(body, 10 + index * 4, "invalid jump position")?;
    }
    for (index, value) in direction.iter_mut().enumerate() {
        *value = checked_float(body, 22 + index * 4, "invalid jump direction")?;
    }
    let vertical_velocity = f32::from_le_bytes(body[36..40].try_into().unwrap());
    if !vertical_velocity.is_finite() {
        return Err(invalid("non-finite jump velocity"));
    }
    let vertical_velocity = vertical_velocity.clamp(-MAX_VERTICAL_SPEED, MAX_VERTICAL_SPEED);
    Ok(JumpUpdate {
        position,
        direction,
        state: body[34],
        unknown: body[35],
        vertical_velocity,
    })
}

/// Build the observer body from the server-bound GUID, the final validated pose
/// and the accepted 164's impulse. Position and direction are the pose's; `state`,
/// `unknown` and the velocity are the jump's, so the native handler's bit-2
/// velocity rule (`0090e470` at `0x0090e5f5`) survives a later 125 pose child.
/// The gateway builds it once per request, after the last child.
pub fn relay(guid: u64, movement: &MovementState, impulse: JumpImpulse) -> Vec<u8> {
    let mut body = Vec::with_capacity(BODY_LEN);
    body.extend_from_slice(&OPCODE.to_le_bytes());
    body.extend_from_slice(&guid.to_le_bytes());
    for value in movement.position.into_iter().chain(movement.direction) {
        body.extend_from_slice(&value.to_le_bytes());
    }
    body.extend_from_slice(&[impulse.state, impulse.unknown]);
    body.extend_from_slice(&impulse.vertical_velocity.to_le_bytes());
    debug_assert_eq!(body.len(), BODY_LEN);
    body
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net_policy::KeyedBuckets;
    use std::time::{Duration, Instant};

    /// Reader `008e5ba0`: opcode, u64 guid, three position floats, three
    /// direction floats (the reader's fourth slots are wire-absent constants),
    /// state byte, unknown byte, velocity. State 4 is what sender `0092e670`
    /// stamps (`v22 |= 4`, bit 2 clear would drop the velocity in `0090e470`).
    fn body(guid: u64, velocity: f32) -> Vec<u8> {
        let mut bytes = OPCODE.to_le_bytes().to_vec();
        bytes.extend_from_slice(&guid.to_le_bytes());
        for value in [1.0_f32, 2.0, 3.0, 0.0, 0.0, 1.0] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&[4, 255]);
        bytes.extend_from_slice(&velocity.to_le_bytes());
        assert_eq!(bytes.len(), BODY_LEN);
        bytes
    }

    #[test]
    fn decodes_the_native_layout_and_rebuilds_it_byte_for_byte() {
        // The pose state deliberately differs from the jump state: the relay body
        // must use the 164's bytes, not the final pose's.
        let movement = MovementState {
            zone_id: 8,
            position: [1.0, 2.0, 3.0],
            direction: [0.0, 0.0, 1.0],
            state: 2,
            unknown: 255,
        };
        let decoded = decode(&body(777, 4.5), 777).unwrap();
        assert_eq!(decoded.position, [1.0, 2.0, 3.0]);
        assert_eq!(decoded.direction, [0.0, 0.0, 1.0]);
        assert_eq!(decoded.state, 4);
        assert_eq!(decoded.unknown, 255);
        assert_eq!(decoded.vertical_velocity, 4.5);
        assert_eq!(
            decoded.movement(),
            MovementUpdate {
                position: [1.0, 2.0, 3.0],
                direction: [0.0, 0.0, 1.0],
                state: 4,
                unknown: 255,
            }
        );
        // The relay body is the wire layout above with the server-bound GUID.
        assert_eq!(relay(777, &movement, decoded.impulse()), body(777, 4.5));
        assert_eq!(relay(777, &movement, decoded.impulse())[34] & 4, 4);
    }

    #[test]
    fn relay_budget_drops_only_extra_jumps_and_is_per_character() {
        let start = Instant::now();
        let mut limiter = KeyedBuckets::new(RELAY_RATE_PER_SECOND, RELAY_BURST, 8);
        let burst = (0..16).filter(|_| limiter.allow(1001, start)).count();
        assert_eq!(burst, RELAY_BURST as usize);
        // A normal cadence (one per 0.5 s) is never dropped.
        for step in 1..=20 {
            assert!(limiter.allow(1001, start + Duration::from_millis(step * 500)));
        }
        // The budget is keyed by character...
        assert!(limiter.allow(1002, start + Duration::from_millis(10_500)));
        // ...and leaving forgets it, starting a fresh burst.
        limiter.remove(&1001);
        assert!(limiter.allow(1001, start + Duration::from_millis(10_500)));
    }

    #[test]
    fn refuses_foreign_guid_bad_lengths_and_nonfinite_values() {
        assert!(decode(&body(1, 1.0)[..39], 1).is_err());
        assert!(decode(&[body(1, 1.0), vec![0]].concat(), 1).is_err());
        let mut wrong = body(1, 1.0);
        wrong[0] = 0;
        assert!(decode(&wrong, 1).is_err());
        assert!(decode(&body(1, 1.0), 2).is_err());

        for offset in [10, 14, 18, 22, 26, 30, 36] {
            let mut nan = body(1, 1.0);
            nan[offset..offset + 4].copy_from_slice(&f32::NAN.to_le_bytes());
            assert!(decode(&nan, 1).is_err());
        }
        assert!(decode(&body(1, f32::INFINITY), 1).is_err());
        assert_eq!(
            decode(&body(1, MAX_VERTICAL_SPEED + 1.0), 1)
                .unwrap()
                .vertical_velocity,
            MAX_VERTICAL_SPEED
        );
        assert_eq!(
            decode(&body(1, -MAX_VERTICAL_SPEED - 1.0), 1)
                .unwrap()
                .vertical_velocity,
            -MAX_VERTICAL_SPEED
        );

        let mut huge = body(1, 1.0);
        huge[10..14].copy_from_slice(&(MAX_POSITION_COMPONENT + 1.0).to_le_bytes());
        assert!(decode(&huge, 1).is_err());
    }
}
