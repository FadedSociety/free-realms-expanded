//! L942: soccer power-ups as the live server ran them (capture `packets_soccer_minigame.pcapng`;
//! client HUD switch at 0xb81414 and `Resources/CodeStringMappings.txt`). A 76/9 puts a pickup on
//! one of the pitch spots and a kind-0 76/9 clears it; a 76/10 names who grabbed it and for how
//! long. Kinds: 1 Speed, 2 Toughness, 4 Charge (the client's "grabbed a ... Increase!" text and
//! model). The values are content: `soccer.json` `pickups`.

use crate::soccer::SoccerPacket;
use serde::Deserialize;
use std::io;

/// Client kinds (76/9 state byte).
pub const SPEED: u8 = 1;
pub const TOUGHNESS: u8 = 2;
pub const CHARGE: u8 = 4;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    #[serde(default, rename = "source")]
    _source: String,
    /// Pitch spots (x, z); the live server used four, symmetric about the centre.
    pub spots: Vec<[f32; 2]>,
    /// Kinds in spawn order, with the live record's two ids.
    pub kinds: Vec<Kind>,
    /// Seconds of play before the first pickup, then between spawns.
    pub first_after: f32,
    pub every: f32,
    /// Seconds a pickup stays on the pitch, and a grabbed boost lasts.
    pub lifetime: f32,
    pub duration: f32,
    /// A player this close (metres, flat) grabs it.
    pub reach: f32,
    /// Speed boost run-speed factor; Toughness knockdown-time factor (R).
    pub speed_factor: f32,
    pub toughness_factor: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Kind {
    pub kind: u8,
    /// The live 76/9's u32 after the kind (852 Speed, 853 Toughness, 887 Charge; meaning R).
    pub definition: u32,
    /// The live 76/9's last u32 (1480 Speed, 1484 Toughness, 1482 Charge; meaning R).
    pub instance: u32,
}

impl Settings {
    pub fn validate(&self) -> io::Result<()> {
        let finite = |values: &[f32]| {
            values
                .iter()
                .all(|value| value.is_finite() && *value >= 0.0)
        };
        let spots_ok = !self.spots.is_empty()
            && self.spots.len() <= usize::from(u8::MAX)
            && self
                .spots
                .iter()
                .all(|spot| spot.iter().all(|v| v.is_finite()));
        let kinds_ok = !self.kinds.is_empty()
            && self
                .kinds
                .iter()
                .all(|k| matches!(k.kind, SPEED | TOUGHNESS | CHARGE));
        let times = [
            self.first_after,
            self.every,
            self.lifetime,
            self.duration,
            self.reach,
        ];
        if spots_ok
            && kinds_ok
            && finite(&times)
            && finite(&[self.speed_factor, self.toughness_factor])
            && self.every > 0.0
            && self.lifetime > 0.0
        {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "soccer pickups are invalid",
            ))
        }
    }
}

/// A grab: who, which kind, for how long.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grab {
    pub player: u64,
    pub kind: u8,
    pub duration: f32,
}

#[derive(Clone, Debug)]
pub struct Pickups {
    settings: Settings,
    /// Per spot: the pickup on it and when it expires (pickup clock).
    live: Vec<Option<(Kind, f32)>>,
    clock: f32,
    next_spawn: f32,
    turn: usize,
}

impl Pickups {
    pub fn new(settings: Settings) -> io::Result<Self> {
        settings.validate()?;
        Ok(Self {
            live: vec![None; settings.spots.len()],
            next_spawn: settings.first_after,
            clock: 0.0,
            turn: 0,
            settings,
        })
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Advance `dt` seconds of play with every placed player's position: expiries and grabs clear
    /// their spot, then a spawn is due every `every` seconds on the next free spot.
    pub fn step(&mut self, dt: f32, players: &[(u64, [f32; 3])]) -> (Vec<SoccerPacket>, Vec<Grab>) {
        self.clock += dt;
        let mut records = Vec::new();
        let mut grabs = Vec::new();
        for spot in 0..self.live.len() {
            let Some((kind, expires)) = self.live[spot] else {
                continue;
            };
            let [x, z] = self.settings.spots[spot];
            let grabber = players
                .iter()
                .find(|(_, at)| (at[0] - x).hypot(at[2] - z) <= self.settings.reach);
            if let Some(&(player, _)) = grabber {
                records.push(SoccerPacket::AcquirePickUp {
                    flag: spot as u8,
                    value: self.settings.duration,
                    player,
                });
                grabs.push(Grab {
                    player,
                    kind: kind.kind,
                    duration: self.settings.duration,
                });
            } else if self.clock < expires {
                continue;
            }
            records.push(record(0, kind, spot, [x, z], 0.0));
            self.live[spot] = None;
        }
        if self.clock >= self.next_spawn {
            self.next_spawn = self.clock + self.settings.every;
            let spots = self.live.len();
            if let Some(spot) = (0..spots)
                .map(|i| (self.turn + i) % spots)
                .find(|&i| self.live[i].is_none())
            {
                let kind = self.settings.kinds[self.turn % self.settings.kinds.len()];
                self.turn += 1;
                let lifetime = self.settings.lifetime;
                records.push(record(
                    kind.kind,
                    kind,
                    spot,
                    self.settings.spots[spot],
                    lifetime,
                ));
                self.live[spot] = Some((kind, self.clock + lifetime));
            }
        }
        (records, grabs)
    }
}

/// One 76/9: the live layout puts x, z and the seconds left in the codec's `position`.
fn record(state: u8, kind: Kind, spot: usize, [x, z]: [f32; 2], left: f32) -> SoccerPacket {
    SoccerPacket::SpawnPickUp {
        state,
        definition: kind.definition,
        marker: spot as u8,
        position: [x, z, left],
        scale: 1.0,
        instance: kind.instance,
    }
}
