//! Spot the Difference's glue for `Gameplay`: Play on a Games window row or a telescope offers
//! that game, and its lifecycle and movie packets route to it. The rules live in
//! `crate::spot_game`.
use super::*;
use crate::{
    npc_interaction::{TableMenu, TableMenus},
    spot_game::{CONTENT_FILE, SpotContent, SpotGame},
};

/// The telescope's name (`npc_spawns_extra.json`).
const TELESCOPE_NAME: &str = "Spot the Difference";

/// The game when `spot_game.json` is present; without it no row is published.
pub(super) fn load_spot(content: &Path) -> io::Result<Option<Box<SpotGame>>> {
    if !content.join(CONTENT_FILE).is_file() {
        return Ok(None);
    }
    SpotGame::new(SpotContent::load(content)?).map(|game| Some(Box::new(game)))
}

/// Each telescope's menu: a game table with every game.
pub(super) fn telescope_menus(spot: &Option<Box<SpotGame>>) -> TableMenus {
    let Some(spot) = spot else {
        return TableMenus::new();
    };
    let (games, telescopes) = spot.telescopes();
    let menu = |position| TableMenu {
        name: TELESCOPE_NAME.to_owned(),
        position,
        games: games.clone(),
        lobbies: Vec::new(),
    };
    (telescopes.iter())
        .map(|telescope| (telescope.npc, menu(telescope.position)))
        .collect()
}

impl Gameplay {
    /// Play on a Spot the Difference row: the 39/16 offer, or one refusal line.
    pub(super) fn begin_spot(&mut self, id: u32) -> io::Result<Outcome> {
        let refused = |text: &str| Outcome {
            replies: vec![chat::system(text)],
            ..Outcome::default()
        };
        let game = i32::try_from(id).map_err(|_| invalid("spot game id"))?;
        let Some(spot) = self.spot.as_mut() else {
            return Ok(refused("Spot the Difference is not available."));
        };
        Ok(match spot.begin(game)? {
            Some(run) => Outcome {
                changed: run.changed,
                replies: run.replies,
                ..Outcome::default()
            },
            None => refused("Finish your game first."),
        })
    }

    /// One packet while a game is open (`None` otherwise). As for a song, only the lifecycle
    /// result (39/18) credits the minigame, once, and a win pays its coins first. The score is
    /// the client's own, so only the game id is quest authority (T787).
    pub(super) fn handle_spot(
        &mut self,
        inner: &[u8],
        player_id: u64,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        if !(self.spot.as_deref()).is_some_and(SpotGame::is_active) {
            return Ok(None);
        }
        let rewards = self.reward_snapshot();
        let Some(spot) = self.spot.as_mut() else {
            return Ok(None);
        };
        let Some(run) = spot.route_with(inner, now_ms, Some((player_id, &rewards)))? else {
            return Ok(None);
        };
        let mut out = Outcome {
            changed: run.changed,
            replies: run.replies.clone(),
            ..Outcome::default()
        };
        if let Some(next) = run.reward_state {
            self.adopt_reward_state(next, &mut out)?;
        }
        out.replies.extend(run.reward_packets);
        if run.replies.iter().any(|p| p.starts_with(&[39, 0, 18])) {
            self.npc_minigame_win_id_only(&run.replies, &mut out)?;
        }
        Ok(Some(out))
    }
}
