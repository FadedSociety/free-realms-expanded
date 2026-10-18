//! What a player is doing right now, and what each activity blocks.
//!
//! One table replaces the hand-written busy lists that used to disagree (`activity_busy`, the
//! warp busy term, `ride_restrictions.in_minigame`, the pets busy flag and the location-report
//! silence). Adding an activity means adding a variant here: `gates` and `Gameplay::is_occupied`
//! are exhaustive, so the build fails until every gate and the detection are decided.
use super::*;

/// One activity a player can be engaged in. Several can be active at once (for example a
/// shared-field fight while a kart entry is queued), so every gate asks "is any active".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Occupation {
    Forging,
    Smelting,
    Cooking,
    Mining,
    Harvesting,
    Chess,
    Encounter,
    Derby,
    Fishing,
    Soccer,
    Kart,
    FieldEngaged,
}

/// What an occupation blocks while it is active.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Gates {
    /// Starting another activity, service or NPC action (`activity_busy`).
    pub(super) start: bool,
    /// Warpstone travel (knockout and encounter report their own message first).
    pub(super) warp: bool,
    /// Mounting or staying mounted.
    pub(super) ride: bool,
    /// Summoning or using pets.
    pub(super) pets: bool,
    /// Ordinary location reports are ignored because the activity owns movement.
    pub(super) location: bool,
}

impl Occupation {
    /// Every variant, in the order gates are evaluated. Keep in step with the enum
    /// (`all_lists_every_variant` fails otherwise).
    pub(super) const ALL: [Occupation; 12] = [
        Occupation::Forging,
        Occupation::Smelting,
        Occupation::Cooking,
        Occupation::Mining,
        Occupation::Harvesting,
        Occupation::Chess,
        Occupation::Encounter,
        Occupation::Derby,
        Occupation::Fishing,
        Occupation::Soccer,
        Occupation::Kart,
        Occupation::FieldEngaged,
    ];

    /// (R) Today's rules, reproduced exactly. Derby does not block warp and kart does not
    /// block starting another activity; both are known gaps kept until a tested change.
    pub(super) const fn gates(self) -> Gates {
        use Occupation::*;
        let (start, warp, ride, pets, location) = match self {
            Forging | Smelting | Cooking | Mining | Harvesting | Chess | Fishing => {
                (true, true, false, true, false)
            }
            Encounter => (true, true, true, true, false),
            Derby => (true, false, true, true, true),
            Soccer => (true, true, true, true, true),
            Kart => (false, true, true, true, true),
            FieldEngaged => (false, false, true, false, false),
        };
        Gates {
            start,
            warp,
            ride,
            pets,
            location,
        }
    }
}

impl Gameplay {
    /// Whether one occupation is active right now.
    pub(super) fn is_occupied(&self, occupation: Occupation) -> bool {
        match occupation {
            Occupation::Forging => self.forging.as_ref().is_some_and(Forging::active),
            Occupation::Smelting => self.smelting.as_ref().is_some_and(Smelting::active),
            Occupation::Cooking => self.cooking.as_ref().is_some_and(Cooking::active),
            Occupation::Mining => self.mining.as_ref().is_some_and(Type8Engine::active),
            Occupation::Harvesting => self.harvesting.as_ref().is_some_and(Type8Engine::active),
            Occupation::Chess => self.chess.as_ref().is_some_and(ChessService::is_active),
            Occupation::Encounter => self.encounter.as_ref().is_some_and(EncounterEngine::active),
            Occupation::Derby => self
                .derby
                .as_ref()
                .is_some_and(crate::derby_gateway::DerbyGateway::active),
            Occupation::Fishing => !self.fishing.is_empty(),
            Occupation::Soccer => self.soccer.is_some(),
            Occupation::Kart => self.kart_entry.as_ref().is_some_and(|k| k.active()),
            Occupation::FieldEngaged => self.field_engaged,
        }
    }

    /// Whether any active occupation closes this gate.
    pub(super) fn blocks(&self, gate: fn(Gates) -> bool) -> bool {
        Occupation::ALL
            .into_iter()
            .any(|occupation| gate(occupation.gates()) && self.is_occupied(occupation))
    }
}

#[cfg(test)]
mod tests {
    use super::Occupation::{self, *};

    #[test]
    fn all_lists_every_variant() {
        // Exhaustive: a new variant fails to compile here until it gets a position and ALL.
        fn position(occupation: Occupation) -> usize {
            match occupation {
                Forging => 0,
                Smelting => 1,
                Cooking => 2,
                Mining => 3,
                Harvesting => 4,
                Chess => 5,
                Encounter => 6,
                Derby => 7,
                Fishing => 8,
                Soccer => 9,
                Kart => 10,
                FieldEngaged => 11,
            }
        }
        for (index, occupation) in Occupation::ALL.into_iter().enumerate() {
            assert_eq!(position(occupation), index);
        }
    }

    /// Pins the table to the lists it replaced; changing a gate is a deliberate, tested edit.
    #[test]
    fn gates_reproduce_the_replaced_busy_lists() {
        let members = |gate: fn(super::Gates) -> bool| -> Vec<Occupation> {
            Occupation::ALL
                .into_iter()
                .filter(|occupation| gate(occupation.gates()))
                .collect()
        };
        assert_eq!(
            members(|g| g.start),
            [
                Forging, Smelting, Cooking, Mining, Harvesting, Chess, Encounter, Derby, Fishing,
                Soccer
            ]
        );
        assert_eq!(
            members(|g| g.warp),
            [
                Forging, Smelting, Cooking, Mining, Harvesting, Chess, Encounter, Fishing, Soccer,
                Kart
            ]
        );
        assert_eq!(
            members(|g| g.ride),
            [Encounter, Derby, Soccer, Kart, FieldEngaged]
        );
        assert_eq!(
            members(|g| g.pets),
            [
                Forging, Smelting, Cooking, Mining, Harvesting, Chess, Encounter, Derby, Fishing,
                Soccer, Kart
            ]
        );
        assert_eq!(members(|g| g.location), [Derby, Soccer, Kart]);
    }
}
