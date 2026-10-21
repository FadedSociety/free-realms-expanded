//! What a player is doing right now, and what each activity blocks.
//!
//! One table replaces the hand-written busy lists that used to disagree (`activity_busy`, the
//! warp busy term, `ride_restrictions.in_minigame`, the pets busy flag and the location-report
//! silence). Adding an activity means adding a variant here: `gates` and `Gameplay::is_occupied`
//! are exhaustive, so the build fails until every gate and the detection are decided.
use super::*;

/// Declares the enum and `Occupation::ALL` from one list, so a variant can never exist without
/// being consulted by `Gameplay::blocks`.
macro_rules! occupations {
    ($($variant:ident),+ $(,)?) => {
        /// One activity a player can be engaged in. Several can be active at once (for example a
        /// shared-field fight while a kart entry is queued), so every gate asks "is any active".
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub(super) enum Occupation {
            $($variant),+
        }

        impl Occupation {
            /// Every variant, in the order gates are evaluated.
            pub(super) const ALL: &[Occupation] = &[$(Occupation::$variant),+];
        }
    };
}

occupations!(
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
    // Lead housing slice 1: inside a house lot (a separate scene).
    Housing,
);

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
    /// (R) Reconstructed rules. the owner, 2026-10-19: a kart race blocks starting another activity
    /// and a derby blocks warpstone travel, like every other minigame (the old hand-written
    /// lists had left both open).
    pub(super) const fn gates(self) -> Gates {
        use Occupation::*;
        let (start, warp, ride, pets, location) = match self {
            Forging | Smelting | Cooking | Mining | Harvesting | Chess | Fishing => {
                (true, true, false, true, false)
            }
            Encounter => (true, true, true, true, false),
            Derby | Soccer | Kart => (true, true, true, true, true),
            FieldEngaged => (false, false, true, false, false),
            // (R) A lot is its own scene: leave it before travelling, riding or starting an
            // activity; pets may follow (the lot's pet autospawn flag); housing owns movement.
            Housing => (true, true, true, false, true),
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
            Occupation::Housing => self.housing.as_ref().is_some_and(|h| h.in_house()),
        }
    }

    /// Whether any active occupation closes this gate.
    pub(super) fn blocks(&self, gate: fn(Gates) -> bool) -> bool {
        Occupation::ALL
            .iter()
            .copied()
            .any(|occupation| gate(occupation.gates()) && self.is_occupied(occupation))
    }
}

#[cfg(test)]
mod tests {
    use super::Occupation::{self, *};

    /// Pins the table; changing a gate is a deliberate, tested edit. The start and warp rows
    /// include kart and derby since the owner closed those gaps (2026-10-19). Housing (lead,
    /// 2026-10-20) closes start, warp and ride, keeps pets, and owns location.
    #[test]
    fn gates_reproduce_the_replaced_busy_lists() {
        let members = |gate: fn(super::Gates) -> bool| -> Vec<Occupation> {
            Occupation::ALL
                .iter()
                .copied()
                .filter(|occupation| gate(occupation.gates()))
                .collect()
        };
        assert_eq!(
            members(|g| g.start),
            [
                Forging, Smelting, Cooking, Mining, Harvesting, Chess, Encounter, Derby, Fishing,
                Soccer, Kart, Housing
            ]
        );
        assert_eq!(
            members(|g| g.warp),
            [
                Forging, Smelting, Cooking, Mining, Harvesting, Chess, Encounter, Derby, Fishing,
                Soccer, Kart, Housing
            ]
        );
        assert_eq!(
            members(|g| g.ride),
            [Encounter, Derby, Soccer, Kart, FieldEngaged, Housing]
        );
        assert_eq!(
            members(|g| g.pets),
            [
                Forging, Smelting, Cooking, Mining, Harvesting, Chess, Encounter, Derby, Fishing,
                Soccer, Kart
            ]
        );
        assert_eq!(members(|g| g.location), [Derby, Soccer, Kart, Housing]);
    }
}
