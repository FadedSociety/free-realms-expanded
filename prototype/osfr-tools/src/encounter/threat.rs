//! L083 (Sanctuary port P2): a field enemy's threat table. Adapted from Sanctuary's `MobBrain`
//! (BubbaJoeX fork of Open-Source-Free-Realms/Sanctuary @1ff876a, `src/Sanctuary.Game/Entities/MobBrain.cs`,
//! AGPL-3.0-or-later). Who hurt the enemy and by how much: the one that hurt it most is its target, the current
//! target keeping a tie, so a player who attacks from beyond the aggro range is fought instead of ignored. A
//! landed hit adds the attacker; `field.rs` drops a player who is no longer a valid target or is beyond the
//! leash, and clears the table when the enemy evades home, dies or respawns. Plain data: no packets here.
use std::collections::BTreeMap;

/// Damage taken per player character (the shared sender, or 1 on a solo field).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Threat(BTreeMap<u64, u32>);

impl Threat {
    /// A landed hit (0 still counts: a shielded summoner was attacked).
    pub(super) fn add(&mut self, player: u64, amount: u32) {
        let total = self.0.entry(player).or_default();
        *total = total.saturating_add(amount);
    }

    pub(super) fn holds(&self, player: u64) -> bool {
        self.0.contains_key(&player)
    }

    pub(super) fn retain(&mut self, keep: impl Fn(u64) -> bool) {
        self.0.retain(|player, _| keep(*player));
    }

    pub(super) fn clear(&mut self) {
        self.0.clear();
    }

    /// The candidate on the table with the most threat; the current target keeps a tie, then the lower guid.
    pub(super) fn pick(
        &self,
        candidates: impl IntoIterator<Item = u64>,
        current: Option<u64>,
    ) -> Option<u64> {
        candidates
            .into_iter()
            .filter_map(|player| {
                let threat = *self.0.get(&player)?;
                Some((threat, Some(player) == current, std::cmp::Reverse(player)))
            })
            .max()
            .map(|(.., std::cmp::Reverse(player))| player)
    }
}

#[cfg(test)]
mod tests {
    use super::Threat;

    #[test]
    fn the_most_hurt_wins_the_current_target_keeps_a_tie_and_strangers_never_win() {
        let mut threat = Threat::default();
        assert_eq!(
            threat.pick([1, 2], None),
            None,
            "an empty table picks nobody"
        );
        threat.add(7, 10);
        threat.add(9, 10);
        assert_eq!(
            threat.pick([7, 9, 3], None),
            Some(7),
            "a tie goes to the lower guid"
        );
        assert_eq!(
            threat.pick([7, 9], Some(9)),
            Some(9),
            "the current target keeps a tie"
        );
        threat.add(9, 1);
        assert_eq!(
            threat.pick([7, 9], Some(7)),
            Some(9),
            "more threat beats the current target"
        );
        assert_eq!(
            threat.pick([7, 3], Some(3)),
            Some(7),
            "only players on the table are picked"
        );
        threat.add(7, u32::MAX);
        assert_eq!(
            threat.pick([7, 9], None),
            Some(7),
            "threat saturates instead of wrapping"
        );
        threat.retain(|player| player != 7);
        assert!(!threat.holds(7) && threat.holds(9));
        threat.clear();
        assert_eq!(threat.pick([9], None), None);
    }
}
