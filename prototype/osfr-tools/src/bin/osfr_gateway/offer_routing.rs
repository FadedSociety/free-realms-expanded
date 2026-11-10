//! P-games-window: the one place every outgoing reply passes (`commit_outcome`) gives each
//! minigame offer its Marketplace panel (`osfr_tools::offer_boosts`). Staged with the other
//! replies, so a refused commit sends none.
use super::*;

/// The bytes `wrap_application` puts ahead of an inner packet.
const WRAPPER: usize = 7;

pub(super) fn dress(outcome: &mut GatewayOutcome, gateway: &mut Gateway) {
    // The commit whose client just finished loading a scene, as `client_area_routing` reads it.
    let entered = outcome.entered_world && gateway.is_world_loaded();
    if let Some(game) = gateway.gameplay_mut() {
        game.dress_offers(&mut outcome.replies, WRAPPER, wrap_application, entered);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tests::test_support::{self, Scratch};
    use osfr_tools::offer_boosts::Watch;

    /// A reply as the sender queues it: `[5, 0, 1]`, the inner length, the inner packet.
    fn well_wrapped(reply: &[u8]) -> bool {
        reply.len() >= WRAPPER
            && reply[..3] == [5, 0, 1]
            && u32::from_le_bytes(reply[3..7].try_into().unwrap()) as usize == reply.len() - WRAPPER
    }

    /// The tap on wrapped replies: the Marketplace rows, the offer, then its 39/44, each wrapped.
    #[test]
    fn a_wrapped_offer_gains_its_wrapped_panel() {
        if !test_support::bin_test_target("offer_routing") {
            return;
        }
        // A 39/16 as `minigame::info_body` lays it out: instance 9, game 1040, type 2.
        let mut offer = vec![39, 0, 16];
        for value in [9, -1, 1040, 0, 0, 0, 1, 0, 2] {
            offer.extend(i32::to_le_bytes(value));
        }
        let chat = wrap_application(&[15, 0, 3, 0]);
        let staged = || vec![chat.clone(), wrap_application(&offer)];

        // The shipped table with the tap's own wrapper width and wrapper, every bundle listed.
        let content = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let mut watch = Watch::load(&content).unwrap();
        let mut replies = staged();
        let rows = || vec![vec![66, 0, 5, 0]];
        watch.dress(&mut replies, WRAPPER, wrap_application, false, |_| true, rows);
        let mut panel = vec![39, 0, 44];
        for value in [9, -1, 1040, 1040, 2, 78, 81] {
            panel.extend(i32::to_le_bytes(value));
        }
        assert_eq!(
            replies,
            [
                chat.clone(),
                wrap_application(&[66, 0, 5, 0]),
                wrap_application(&offer),
                wrap_application(&panel)
            ]
        );

        // `dress` itself on a real session (shipped content): whatever this rig's Marketplace
        // lists, the staged replies stay in order and every reply stays well wrapped.
        let scratch = Scratch::new("offer-routing");
        let mut options = test_support::options(scratch.state());
        options.reference = PathBuf::new();
        options.content = Some(content);
        let mut scene = test_support::Scene::new(&options, 1);
        let mut outcome = GatewayOutcome { replies: staged(), ..GatewayOutcome::default() };
        dress(&mut outcome, &mut scene.session.gateway);
        assert!(outcome.replies.iter().all(|reply| well_wrapped(reply)));
        assert_eq!(outcome.replies[0], chat);
        let at = outcome.replies.iter().position(|r| *r == wrap_application(&offer)).unwrap();
        if let Some(next) = outcome.replies.get(at + 1) {
            assert!(next[WRAPPER..].starts_with(&[39, 0, 44]), "only its panel follows an offer");
        }
        // A commit with no offer gains nothing.
        let mut quiet = GatewayOutcome { replies: vec![chat.clone()], ..GatewayOutcome::default() };
        dress(&mut quiet, &mut scene.session.gateway);
        assert_eq!(quiet.replies, [chat]);
    }
}
