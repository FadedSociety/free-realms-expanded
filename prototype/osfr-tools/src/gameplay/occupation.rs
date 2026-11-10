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
    // L103: a Mail Sorting run (Postman).
    Mail,
    // An Artifact Hunt run (Adventurer).
    Artifact,
    Chess,
    // L104: a checkers game against the server; (R) it closes the chess gates.
    Checkers,
    // L961: a Feeling the Beat! song against the server; (R) it closes the checkers gates.
    Music,
    Sword,
    Spot,
    // L962: a Tower Defense game against the server; (R) it closes the checkers gates.
    TowerDefense,
    // L963: a Pirate's Plunder match in its arena; (R) the derby's gates.
    Pirates,
    // The Kart & Demo Derby Garage, from its offer on; (R) the derby's gates.
    Garage,
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
    // T560: the daily wheel's movie is open (Flash window over the world).
    Wheel,
    // L937: an in-world challenge's start screen or run (TYPE 1 minigames).
    Challenge,
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
    /// (R) Reconstructed rules. The owner, 2026-10-19: a kart race blocks starting another activity
    /// and a derby blocks warpstone travel, like every other minigame (the old hand-written
    /// lists had left both open).
    ///
    /// T481: a gather, craft, chess or fishing activity also closes the ride gate, so starting
    /// one ends the ride and no ride can be summoned during it. The client dismounts by itself
    /// only on an ability or item use (`SendDismountRequest` 0x00b06cc0, four callers), never on
    /// the 26/8 interaction (`PlayClient::RequestInteraction` 0x0092efe0), so the server ends it.
    /// Their row now matches Encounter's: start, warp, ride and pets; not location.
    /// L082: the activity as a refusal line names it ("... during a kart race").
    pub(super) const fn activity(self) -> &'static str {
        use Occupation::*;
        match self {
            Forging => "forging",
            Smelting => "smelting",
            Cooking => "cooking",
            Mining => "mining",
            Harvesting => "harvesting",
            Mail => "mail sorting",
            Artifact => "an artifact hunt",
            Chess => "a chess game",
            Checkers => "a checkers game",
            Music => "a Feeling the Beat! song",
            Sword => "a Hack 'n' Slash game",
            Spot => "a Spot the Difference game",
            TowerDefense => "a Tower Defense game",
            Pirates => "a Pirate's Plunder match",
            CardGame => "a card game",
            Encounter | FieldEngaged => "a fight",
            Derby => "a derby",
            Garage => "the garage",
            Fishing => "fishing",
            Soccer => "a soccer match",
            Kart => "a kart race",
            Housing => "a house visit",
            Groom => "grooming",
            Wheel => "the prize wheel",
            Challenge => "a challenge",
        }
    }

    pub(super) const fn gates(self) -> Gates {
        use Occupation::*;
        let (start, warp, ride, pets, location) = match self {
            // (R) A card run closes the same gates as chess (T765).
            Forging | Smelting | Cooking | Mining | Harvesting | Mail | Artifact | Chess | Checkers
            | Music | Sword | Spot | TowerDefense | CardGame | Fishing | Encounter => {
                (true, true, true, true, false)
            }
            Derby | Soccer | Kart | Pirates | Garage => (true, true, true, true, true),
            FieldEngaged => (false, false, true, false, false),
            // (R) A lot is its own scene: leave it before travelling, riding or starting an
            // activity; pets may follow (the lot's pet autospawn flag); housing owns movement.
            Housing => (true, true, true, false, true),
            // T488: a groom session closes the start gate, so no craft, gather, fight or minigame
            // begins mid-session. It keeps the pets gate open (its own 53/22 strokes are pet
            // packets, and a dismissed or wet pet already ends it with kind 4) and travel, riding
            // and movement open; zoning and water end it.
            Groom => (true, false, false, false, false),
            // T560 (R): the open wheel movie closes the start gate (no craft, gather, fight or other
            // minigame begins under it) and warpstone travel, like the other Flash movies; riding,
            // pets and movement stay open because the wheel is a window, not a scene.
            Wheel => (true, true, false, false, false),
            // L937 (R): a challenge closes the start gate and warpstone travel and ends a ride (the
            // run is on foot); pets and movement stay open, the player runs the course.
            Challenge => (true, true, true, false, false),
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
            Occupation::Mail => self.mail_sorting.as_ref().is_some_and(Type8Engine::active),
            Occupation::Artifact => self.artifact_hunt.as_ref().is_some_and(Type8Engine::active),
            Occupation::Chess => self.chess.as_ref().is_some_and(ChessService::is_active),
            Occupation::Checkers => self.checkers.is_active(),
            Occupation::Music => {
                (self.music.as_ref()).is_some_and(crate::music_game::MusicGame::is_active)
            }
            Occupation::Sword => {
                (self.sword.as_deref()).is_some_and(crate::sword_game::SwordGame::is_active)
            }
            Occupation::Spot => (self.spot.as_deref()).is_some_and(crate::spot_game::SpotGame::is_active),
            Occupation::TowerDefense => (self.tower_defense.as_deref())
                .is_some_and(crate::tower_defense::TowerDefense::is_active),
            Occupation::Pirates => self.pirates.is_some(),
            Occupation::Garage => self.garage.is_some(),
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
            Occupation::Wheel => self.daily_wheel.is_active(),
            Occupation::Challenge => self.world_challenge.is_active(),
        }
    }

    /// Whether any active occupation closes this gate.
    pub(super) fn blocks(&self, gate: fn(Gates) -> bool) -> bool {
        self.blocker(gate).is_some()
    }

    /// L082: the first running activity whose `gate` is closed, so a refusal can say why.
    pub(super) fn blocker(&self, gate: fn(Gates) -> bool) -> Option<Occupation> {
        Occupation::ALL
            .iter()
            .copied()
            .find(|occupation| gate(occupation.gates()) && self.is_occupied(*occupation))
    }

    /// True while a crafting, gathering or battle activity is running.
    pub(super) fn activity_busy(&self) -> bool {
        self.blocks(|gates| gates.start)
    }

    /// P-housing: `activity_busy` with the Housing variant alone exempted. Being inside a lot
    /// is an occupation that closes `start`, yet the lot's own NPC and a second lot must stay
    /// reachable from it; every other start-closing activity (a groom on the farm, a fight)
    /// still refuses, exactly as in the continent.
    pub(super) fn busy_outside_housing(&self) -> bool {
        Occupation::ALL
            .iter()
            .any(|o| *o != Occupation::Housing && o.gates().start && self.is_occupied(*o))
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
                Forging,
                Smelting,
                Cooking,
                Mining,
                Harvesting,
                Mail,
                Artifact,
                Chess,
                Checkers,
                Music,
                Sword,
                Spot,
                TowerDefense,
                Pirates,
                Garage,
                CardGame,
                Encounter,
                Derby,
                Fishing,
                Soccer,
                Kart,
                Housing,
                Groom,
                Wheel,
                Challenge
            ]
        );
        assert_eq!(
            members(|g| g.warp),
            [
                Forging,
                Smelting,
                Cooking,
                Mining,
                Harvesting,
                Mail,
                Artifact,
                Chess,
                Checkers,
                Music,
                Sword,
                Spot,
                TowerDefense,
                Pirates,
                Garage,
                CardGame,
                Encounter,
                Derby,
                Fishing,
                Soccer,
                Kart,
                Housing,
                Wheel,
                Challenge
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
                Mail,
                Artifact,
                Chess,
                Checkers,
                Music,
                Sword,
                Spot,
                TowerDefense,
                Pirates,
                Garage,
                CardGame,
                Encounter,
                Derby,
                Fishing,
                Soccer,
                Kart,
                FieldEngaged,
                Housing,
                Challenge
            ]
        );
        assert_eq!(
            members(|g| g.pets),
            [
                Forging,
                Smelting,
                Cooking,
                Mining,
                Harvesting,
                Mail,
                Artifact,
                Chess,
                Checkers,
                Music,
                Sword,
                Spot,
                TowerDefense,
                Pirates,
                Garage,
                CardGame,
                Encounter,
                Derby,
                Fishing,
                Soccer,
                Kart
            ]
        );
        assert_eq!(
            members(|g| g.location),
            [Pirates, Garage, Derby, Soccer, Kart, Housing]
        );
        // A style card is refused wherever starting an activity is, except at home.
        assert_eq!(
            members(|g| g.style_card),
            [
                Forging,
                Smelting,
                Cooking,
                Mining,
                Harvesting,
                Mail,
                Artifact,
                Chess,
                Checkers,
                Music,
                Sword,
                Spot,
                TowerDefense,
                Pirates,
                Garage,
                CardGame,
                Encounter,
                Derby,
                Fishing,
                Soccer,
                Kart,
                Groom,
                Wheel,
                Challenge
            ]
        );
    }
}
