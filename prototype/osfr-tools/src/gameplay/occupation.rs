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
    // T765: a Trading Card Game run (the client's card DLL window is open).
    CardGame,
    Encounter,
    Derby,
    Fishing,
    Soccer,
    Kart,
    FieldEngaged,
    // Lead housing slice 1: inside a house lot (a separate scene).
    Housing,
    // T488: a pet grooming session (53/20 kind 3 until kind 4), a timed activity.
    Groom,
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
    /// Using a style card (a look change that starts nothing). Everything that closes `start`
    /// closes it, except a house lot: the player's own home is where cards are used.
    pub(super) style_card: bool,
}

impl Occupation {
    /// (R) Reconstructed rules. the owner, 2026-10-19: a kart race blocks starting another activity
    /// and a derby blocks warpstone travel, like every other minigame (the old hand-written
    /// lists had left both open).
    ///
    /// T481: a gather, craft, chess or fishing activity also closes the ride gate, so starting
    /// one ends the ride and no ride can be summoned during it. The client dismounts by itself
    /// only on an ability or item use (`SendDismountRequest` 0x00b06cc0, four callers), never on
    /// the 26/8 interaction (`PlayClient::RequestInteraction` 0x0092efe0), so the server ends it.
    /// Their row now matches Encounter's: start, warp, ride and pets; not location.
    pub(super) const fn gates(self) -> Gates {
        use Occupation::*;
        let (start, warp, ride, pets, location) = match self {
            // (R) A card run closes the same gates as chess (T765).
            Forging | Smelting | Cooking | Mining | Harvesting | Chess | CardGame | Fishing
            | Encounter => (true, true, true, true, false),
            Derby | Soccer | Kart => (true, true, true, true, true),
            FieldEngaged => (false, false, true, false, false),
            // (R) A lot is its own scene: leave it before travelling, riding or starting an
            // activity; pets may follow (the lot's pet autospawn flag); housing owns movement.
            Housing => (true, true, true, false, true),
            // T488: a groom session closes the start gate, so no craft, gather, fight or minigame
            // begins mid-session. It keeps the pets gate open (its own 53/22 strokes are pet
            // packets, and a dismissed or wet pet already ends it with kind 4) and travel, riding
            // and movement open; zoning and water end it.
            Groom => (true, false, false, false, false),
        };
        Gates {
            start,
            warp,
            ride,
            pets,
            location,
            // (R) the owner/lead 2026-10-25: a card works inside the player's own lot.
            style_card: start && !matches!(self, Housing),
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
            Occupation::CardGame => self
                .card_launches
                .as_ref()
                .is_some_and(crate::card_game::Launches::is_active),
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
            Occupation::Groom => self.pets.is_grooming(),
        }
    }

    /// Whether any active occupation closes this gate.
    pub(super) fn blocks(&self, gate: fn(Gates) -> bool) -> bool {
        Occupation::ALL
            .iter()
            .copied()
            .any(|occupation| gate(occupation.gates()) && self.is_occupied(occupation))
    }

    /// True while a crafting, gathering or battle activity is running.
    pub(super) fn activity_busy(&self) -> bool {
        self.blocks(|gates| gates.start)
    }
}

#[cfg(test)]
mod tests {
    use super::Occupation::{self, *};

    /// Pins the table; changing a gate is a deliberate, tested edit. The start and warp rows
    /// include kart and derby since the owner closed those gaps (2026-10-19). Housing (lead,
    /// 2026-10-20) closes start, warp and ride, keeps pets, and owns location. T488/F796: a groom
    /// session closes start (and so style cards) but keeps pets, travel, riding and movement open.
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
                Forging, Smelting, Cooking, Mining, Harvesting, Chess, CardGame, Encounter, Derby,
                Fishing, Soccer, Kart, Housing, Groom
            ]
        );
        assert_eq!(
            members(|g| g.warp),
            [
                Forging, Smelting, Cooking, Mining, Harvesting, Chess, CardGame, Encounter, Derby,
                Fishing, Soccer, Kart, Housing
            ]
        );
        assert_eq!(
            members(|g| g.ride),
            [
                Forging,
                Smelting,
                Cooking,
                Mining,
                Harvesting,
                Chess,
                CardGame,
                Encounter,
                Derby,
                Fishing,
                Soccer,
                Kart,
                FieldEngaged,
                Housing
            ]
        );
        assert_eq!(
            members(|g| g.pets),
            [
                Forging, Smelting, Cooking, Mining, Harvesting, Chess, CardGame, Encounter, Derby,
                Fishing, Soccer, Kart
            ]
        );
        assert_eq!(members(|g| g.location), [Derby, Soccer, Kart, Housing]);
        // A style card is refused wherever starting an activity is, except at home.
        assert_eq!(
            members(|g| g.style_card),
            [
                Forging, Smelting, Cooking, Mining, Harvesting, Chess, CardGame, Encounter, Derby,
                Fishing, Soccer, Kart, Groom
            ]
        );
    }
}
