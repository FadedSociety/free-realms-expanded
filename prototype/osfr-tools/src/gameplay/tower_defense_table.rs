//! L962: Tower Defense's glue for `Gameplay`: Play on a Games window row offers that game, and its
//! lifecycle and movie packets route to it. The rules live in `crate::tower_defense`.
use super::*;
use crate::tower_defense::{CONTENT_FILE, TdContent, TowerDefense};

/// The Tower Defense games when `tower_defense.json` is present; without it no row is published.
pub(super) fn load_tower_defense(content: &Path) -> io::Result<Option<Box<TowerDefense>>> {
    if !content.join(CONTENT_FILE).is_file() {
        return Ok(None);
    }
    TowerDefense::new(&TdContent::load(content)?).map(|td| Some(Box::new(td)))
}

/// Makes the world's game tables and the dock's book stands clickable: each answers as NPC content
/// under its own guid.
pub(super) fn bind_tables(
    content: &Path,
    td: &Option<Box<TowerDefense>>,
    ships: &Option<Arc<crate::ship_combat::Content>>,
    interaction: &mut Option<NpcInteraction>,
    visibility: &mut Option<crate::world::NpcVisibility>,
) -> io::Result<()> {
    let stands = ships.as_ref().map(|ships| ships.stand_menus()).unwrap_or_default();
    let boards = super::checkers_table::board_table_menus(content)?;
    if let Some(interaction) = interaction {
        interaction.set_pirates_stands(&stands);
        interaction.set_board_tables(&boards);
        if let Some(td) = td {
            interaction.set_tower_defense_tables(td.tables());
        }
    }
    if let Some(visibility) = visibility {
        visibility.bind_content_keys(stands.keys().copied());
        visibility.bind_content_keys(boards.keys().copied());
        if let Some(td) = td {
            visibility.bind_content_keys(td.tables().keys().copied());
        }
    }
    Ok(())
}

impl Gameplay {
    /// Play on a Tower Defense row: the 39/16 offer, or one refusal line.
    pub(super) fn begin_tower_defense(&mut self, id: u32) -> io::Result<Outcome> {
        let refused = |text: &str| Outcome {
            replies: vec![chat::system(text)],
            ..Outcome::default()
        };
        let game = i32::try_from(id).map_err(|_| invalid("tower defense game id"))?;
        let Some(td) = self.tower_defense.as_deref_mut() else {
            return Ok(refused("Tower Defense is not available."));
        };
        Ok(match td.begin(game)? {
            Some(run) => Outcome {
                changed: run.changed,
                replies: run.replies,
                ..Outcome::default()
            },
            None => refused("Finish your Tower Defense game first."),
        })
    }

    /// One packet while a game is open (`None` otherwise). As for the songs (L961), only the lifecycle
    /// result (39/18) credits the minigame, once, and a win pays its coins first. The 39/18 score is the
    /// client's own total, so only the game id is quest authority (T787).
    pub(super) fn handle_tower_defense(
        &mut self,
        inner: &[u8],
        player_id: u64,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        if !(self.tower_defense.as_deref()).is_some_and(TowerDefense::is_active) {
            return Ok(None);
        }
        let rewards = self.reward_snapshot();
        let Some(td) = self.tower_defense.as_deref_mut() else {
            return Ok(None);
        };
        let Some(run) = td.route_with(inner, now_ms, Some((player_id, &rewards)))? else {
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
