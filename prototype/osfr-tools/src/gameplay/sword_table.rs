//! Hack 'n' Slash's glue for `Gameplay`: Play on the Games window row offers the game, and its
//! lifecycle and movie packets route to it. The rules live in `crate::sword_game`.
use super::*;
use crate::{
    npc_interaction::{TableMenu, TableMenus},
    sword_game::{CONTENT_FILE, SwordContent, SwordGame},
};

/// The Practice Dummy's name (`npc_spawns_extra.json`).
const DUMMY_NAME: &str = "Practice Dummy";

/// The game when `sword_game.json` is present; without it no row is published.
pub(super) fn load_sword(content: &Path) -> io::Result<Option<Box<SwordGame>>> {
    if !content.join(CONTENT_FILE).is_file() {
        return Ok(None);
    }
    SwordGame::new(SwordContent::load(content)?).map(|game| Some(Box::new(game)))
}

/// Each Practice Dummy's menu: a game table with one game.
pub(super) fn dummy_menus(sword: &Option<Box<SwordGame>>) -> TableMenus {
    let Some(sword) = sword else {
        return TableMenus::new();
    };
    let (label, dummies) = sword.dummies();
    let game = sword.game_id() as u32;
    let menu = |position| TableMenu {
        name: DUMMY_NAME.to_owned(),
        position,
        games: vec![(game, label)],
        lobbies: Vec::new(),
    };
    (dummies.iter())
        .map(|dummy| (dummy.npc, menu(dummy.position)))
        .collect()
}

impl Gameplay {
    /// Play on the Hack 'n' Slash row: the 39/16 offer, or one refusal line.
    pub(super) fn begin_sword(&mut self) -> io::Result<Outcome> {
        let refused = |text: &str| Outcome {
            replies: vec![chat::system(text)],
            ..Outcome::default()
        };
        let Some(sword) = self.sword.as_mut() else {
            return Ok(refused("Hack 'n' Slash is not available."));
        };
        Ok(match sword.begin()? {
            Some(run) => Outcome {
                changed: run.changed,
                replies: run.replies,
                ..Outcome::default()
            },
            None => refused("Finish your game first."),
        })
    }

    /// One packet while a game is open (`None` otherwise). As for a song, only the lifecycle
    /// result (39/18) credits the minigame, once, and a win pays its coins first. The 39/18 score
    /// is the client's own total, so only the game id is quest authority (T787).
    pub(super) fn handle_sword(
        &mut self,
        inner: &[u8],
        player_id: u64,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        if !(self.sword.as_deref()).is_some_and(SwordGame::is_active) {
            return Ok(None);
        }
        let rewards = self.reward_snapshot();
        let Some(sword) = self.sword.as_mut() else {
            return Ok(None);
        };
        let Some(run) = sword.route_with(inner, now_ms, Some((player_id, &rewards)))? else {
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
