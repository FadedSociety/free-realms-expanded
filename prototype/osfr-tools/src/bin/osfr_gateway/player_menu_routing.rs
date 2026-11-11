//! W143/W163: the player click wheel (radial menu) on the shared gateway. Moved out of
//! osfr-gateway.rs by L081, which adds the admin GM wheel here.
use super::*;
use osfr_tools::{account_store::AccountRole, player_gm_menu::Step};

impl Session {
    /// 26/19 select, 26/10 choice on an open player menu, 149/2 close (W143).
    /// A refusal is a silent drop (R): nothing is sent, so ids cannot be probed.
    pub(super) fn route_player_radial(
        &mut self,
        inner: &[u8],
        world: &mut SharedWorld,
        now_ms: u64,
    ) -> Vec<Delivery> {
        let looks = &world.public_looks;
        let me = self.character_guid;
        // L081: only an admin's wheel carries the GM entry, and only an admin's choice acts.
        let admin = self.gateway.role() == AccountRole::Admin;
        let replies = if inner == [149, 0, 2] {
            self.radial.stop_inspect(inner).map(|()| Vec::new())
        } else if inner.starts_with(&[26, 0, 19, 0]) {
            // W163: Trade is offered only where an invite would be admitted right now.
            let trades = &world.trades;
            self.radial
                .select_menu(&world.players, self.id, inner, |guid| MenuOffer {
                    inspect: looks.contains_key(&guid),
                    trade: trades.can_invite(&world.players, &world.social, me, guid),
                    gm: admin,
                    // PB2: only a player the 40/1 path can name (an identity row).
                    group: world.identities.contains_key(&guid)
                        && world.social.can_group_invite(me, guid),
                })
        } else {
            match self
                .radial
                .choose_entry(&world.players, self.id, inner, |guid| {
                    looks.get(&guid).cloned()
                }) {
                Ok(Some(Chosen::Trade(target))) => {
                    // The service re-checks everything; a refusal is a silent drop (R).
                    let invited =
                        world
                            .trades
                            .invite(&world.players, &world.social, me, target, now_ms);
                    return trade_deliveries(invited.unwrap_or_default());
                }
                Ok(Some(Chosen::GroupInvite(target))) => {
                    // PB2: the wheel's Invite to Group is the social window's 40/1 by name.
                    let Some(name) = world.identities.get(&target).map(|id| id.name.clone()) else {
                        return Vec::new();
                    };
                    let lookup = IdentityLookup(&world.identities);
                    let messages = world.social.invite_to_group(me, &name, &lookup, now_ms);
                    return (messages.unwrap_or_default().into_iter())
                        .filter_map(|message| world.social_delivery(message))
                        .collect();
                }
                Ok(Some(Chosen::Inspect(packet))) => Ok(vec![packet]),
                Ok(Some(Chosen::Gm(target, Step::Open(page)))) if admin => self
                    .radial
                    .open_gm_page(&world.players, self.id, target, page),
                Ok(Some(Chosen::Gm(target, Step::Do(action)))) if admin => {
                    // The post-commit host (gm_menu_routing) carries it out.
                    self.radial.queue_gm(target, action, now_ms);
                    Ok(Vec::new())
                }
                Ok(Some(Chosen::Gm(..))) => Ok(Vec::new()),
                Ok(None) => Ok(Vec::new()),
                Err(error) => Err(error),
            }
        };
        replies
            .unwrap_or_default()
            .iter()
            .map(|reply| Delivery {
                recipient: self.id,
                bytes: wrap_application(reply),
            })
            .collect()
    }
}
