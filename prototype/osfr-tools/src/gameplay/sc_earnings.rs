//! T229: Station Cash earned by server-verified completions.
//!
//! Each activity's own completion record raises the event (kart finish claim, derby results,
//! soccer full time, chess settlement, quest turn-in, dungeon clear); this module turns it into one
//! `rewards::apply_grant` station-cash kind on the S5 reward snapshot, so the payout shares the
//! single applier, the data ceiling and staged rollback. A refused or absent payout returns before
//! committing: a missing Marketplace, disabled earning, a zero amount or a replay all leave the
//! staged clone unchanged.
use super::*;
use crate::{
    rewards::apply_grant,
    station_cash::{self, EarnSource, RideOffer},
};

impl Gameplay {
    /// T817: `realm sc <n>` for test realms: Station Cash through the same applier and data
    /// ceiling as earned SC, at most 1,000,000 per command like `realm coins`.
    pub(super) fn realm_sc(&mut self, n: &str, out: &mut Outcome) -> io::Result<bool> {
        let Ok(amount) = n.parse::<u32>() else {
            out.replies.push(chat::system("Usage: realm sc <amount>"));
            return Ok(true);
        };
        let Some(shop) = self.npc_coin_shop.as_ref() else {
            out.replies.push(chat::system("This realm has no Marketplace for Station Cash."));
            return Ok(true);
        };
        let amount = amount.min(1_000_000);
        let rewards = self.reward_snapshot().with_wallet(
            shop.sc_wallet().clone(),
            shop.sc_catalogue().config().max_balance,
        );
        let grant = station_cash::Earned {
            sc: amount,
            ..Default::default()
        }
        .grant();
        match apply_grant(&rewards, &grant, "realm-sc") {
            Ok((next, replies)) => {
                out.replies.extend(replies);
                self.adopt_reward_state(next, out)?;
                out.replies
                    .push(chat::system(&format!("Gave {amount} Station Cash.")));
            }
            Err(error) => out
                .replies
                .push(chat::system(&format!("Cannot give Station Cash: {error}"))),
        }
        Ok(true)
    }

    /// Profile state for a reward transaction; a default one without class content.
    pub(super) fn reward_profiles(&self) -> ProfileState {
        self.profiles
            .as_ref()
            .map(|profiles| profiles.state().clone())
            .unwrap_or_else(|| ProfileState {
                version: 1,
                active: 1,
                unlocked: BTreeSet::from([1]),
                experience: Default::default(),
            })
    }

    /// The reward bag capacity: 100 occupied stacks, or the current count when a legacy save
    /// already exceeds it.
    fn reward_bag_capacity(&self) -> usize {
        self.inventory.state().items.len().max(100)
    }

    /// Snapshot the inventory and profiles for a staged reward transaction. The Station Cash
    /// wallet is deliberately absent: a snapshot can outlive the request (fishing keeps one per
    /// round), and adopting a stale wallet would restore a spent balance. Only
    /// [`Self::pay_earned_station_cash`] stages the wallet, and it adopts in the same call.
    pub(super) fn reward_snapshot(&self) -> RewardState {
        RewardState::new(
            self.inventory.clone(),
            self.reward_profiles(),
            self.reward_bag_capacity(),
        )
    }

    /// Adopt the inventory, profiles and Station Cash wallet from a staged minigame transaction.
    ///
    /// Soccer settlements, fishing catches, chess results, the derby completion payout and the
    /// earned Station Cash payout use this path. Progression-extras claims and quest turn-in deliberately keep their
    /// existing payout implementations because their rollback and commit ordering differ; the
    /// quest turn-in commits its own grant and then calls [`Self::pay_earned_station_cash`].
    pub(super) fn adopt_reward_state(
        &mut self,
        next: RewardState,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let before = self.inventory.state().clone();
        if self
            .profiles
            .as_ref()
            .is_some_and(|profiles| *profiles.state() != next.profiles)
        {
            let directory = self
                .profile_content_directory
                .as_deref()
                .ok_or_else(|| invalid("minigame XP needs profile content"))?;
            self.profiles = Some(Profiles::adopt(&self.profiles, directory, next.profiles)?);
            self.npc_quest_profiles_changed(out)?;
        }
        // The wallet is part of the transaction: a refused grant never reaches this adopt.
        if let Some(wallet) = next.wallet {
            if let Some(shop) = self.npc_coin_shop.as_mut() {
                if shop.sc_wallet() != &wallet {
                    shop.set_sc_wallet(wallet);
                }
            }
        }
        self.inventory = next.inventory;
        out.changed = true;
        self.npc_inventory_events(&before, out)?;
        Ok(())
    }

    /// Pay the data-configured Station Cash for one server-verified completion through the single
    /// reward applier. The amount is never a client value. The one-shot guard is the caller's
    /// completion record; the grant key only satisfies the applier, because the wallet is staged
    /// fresh here and adopted before this returns.
    pub(super) fn pay_earned_station_cash(
        &mut self,
        source: EarnSource,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let Some(shop) = self.npc_coin_shop.as_ref() else {
            return Ok(());
        };
        let Some(table) = shop.sc_earn() else {
            return Ok(());
        };
        if !table.enabled || !shop.sc_catalogue().config().enabled {
            return Ok(());
        }
        // The first clear of the table's hardest dungeon grants its earn-only ride (R). The ride
        // must be in the validated catalogue; a content set without it pays the clear in SC only.
        // A full bag defers the ride (nothing recorded) and tells the player.
        let ride = table.first_clear_ride.and_then(|first| {
            if !matches!(source, EarnSource::DungeonClear { id } if id == first.dungeon_id) {
                return None;
            }
            let price = shop.sc_catalogue().earn_only_price(first.item_id)?;
            Some((first.item_id, price))
        });
        let owned =
            ride.is_some_and(|(item, _)| station_cash::owns_item(self.inventory.state(), item));
        let room = self.inventory.state().items.len() < self.reward_bag_capacity();
        let offer = ride.map(|(item_id, sc_equivalent)| RideOffer {
            item_id,
            sc_equivalent,
            owned,
            room,
        });
        let mut wallet = shop.sc_wallet().clone();
        let day = station_cash::activity_day(current_unix_seconds());
        let earned = table.earned(&mut wallet, source, day, offer);
        if earned.bag_full {
            out.replies.push(chat::system(
                "Make room in your bag to earn the ride from this dungeon: clear it again.",
            ));
        }
        if earned.sc == 0 && earned.ride.is_none() {
            return Ok(());
        }
        let rewards = self
            .reward_snapshot()
            .with_wallet(wallet, shop.sc_catalogue().config().max_balance);
        let (next, replies) = apply_grant(&rewards, &earned.grant(), "station-cash-earned")
            .map_err(|error| invalid(format!("Station Cash payout: {error}")))?;
        out.replies.extend(replies);
        self.adopt_reward_state(next, out)
    }
}
