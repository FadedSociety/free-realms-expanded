//! Treasure War's glue for `Gameplay`: Play on the Games window row, the client's own card
//! doors (they send the same join) or the arena keeper offers a game against the computer, and
//! its lifecycle and movie packets route to it. A lobby reaches the two-player game through the
//! `*_hosted_treasure_war` functions, which mirror `hosted_chess.rs`. The rules live in
//! `crate::treasure_war`, the tables in `crate::treasure_war_table`.
use super::*;
use crate::{
    chess::ChessMessage,
    npc_interaction::{TableMenu, TableMenus},
    power_ups::Rng,
    treasure_war::{CONTENT_FILE, Content},
    treasure_war_table::{HostedMatch, TreasureWarTable},
};

/// The table when `treasure_war.json` is present; without it the row stays closed.
pub(super) fn load_treasure_war(content: &Path) -> io::Result<Option<Box<TreasureWarTable>>> {
    if !content.join(CONTENT_FILE).is_file() {
        return Ok(None);
    }
    let table = TreasureWarTable::new(Content::load(content)?, Rng::from_entropy()?)?;
    Ok(Some(Box::new(table)))
}

/// Whether players can start the game.
pub(super) fn is_open(table: &Option<Box<TreasureWarTable>>) -> bool {
    table.as_deref().is_some_and(TreasureWarTable::is_open)
}

/// The arena keeper's menu, while the game is open: a game table with the one game.
pub(super) fn arena_menus(table: &Option<Box<TreasureWarTable>>) -> TableMenus {
    let Some(content) = table
        .as_deref()
        .filter(|table| table.is_open())
        .map(|table| table.content())
    else {
        return TableMenus::new();
    };
    // `validate` holds both ids above 0.
    let menu = TableMenu {
        name: content.arena.name.clone(),
        position: content.arena.position,
        games: vec![(content.game_id as u32, content.name_id as u32)],
        lobbies: Vec::new(),
    };
    TableMenus::from([(content.arena.npc, menu)])
}

impl Gameplay {
    /// Whether `game` is the open Treasure War game (a game table's choice).
    pub(super) fn is_treasure_war(&self, game: u32) -> bool {
        (self.treasure_war.as_deref())
            .is_some_and(|table| table.is_open() && table.content().game_id as u32 == game)
    }

    /// Play: the 39/16 offer, or one refusal line. The caller has refused a busy player.
    pub(super) fn begin_treasure_war(&mut self) -> io::Result<Outcome> {
        let refused = |text: &str| Outcome {
            replies: vec![chat::system(text)],
            ..Outcome::default()
        };
        let Some(table) = self
            .treasure_war
            .as_deref_mut()
            .filter(|table| table.is_open())
        else {
            return Ok(refused(
                &crate::activity_portal::Refusal::TreasureWarClosed.message(),
            ));
        };
        Ok(match table.begin()? {
            Some(run) => Outcome {
                changed: run.changed,
                replies: run.replies,
                ..Outcome::default()
            },
            None => refused("Finish your game first."),
        })
    }

    /// One packet while a game is open (`None` otherwise). As for checkers, only the lifecycle
    /// result (39/18) credits the minigame, once, and a win pays its coins first.
    pub(super) fn handle_treasure_war(
        &mut self,
        inner: &[u8],
        player_id: u64,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        if !(self.treasure_war.as_deref()).is_some_and(TreasureWarTable::is_active) {
            return Ok(None);
        }
        let rewards = self.reward_snapshot();
        let Some(table) = self.treasure_war.as_deref_mut() else {
            return Ok(None);
        };
        let Some(run) = table.route_with(inner, now_ms, Some((player_id, &rewards)))? else {
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

    /// A lobby game's content, for the World to deal the shared game from; `None` while closed.
    pub fn treasure_war_content(&self) -> Option<Arc<Content>> {
        let table = self
            .treasure_war
            .as_deref()
            .filter(|table| table.is_open())?;
        Some(table.content().clone())
    }

    /// The lobby game's offer for this player, as `begin_hosted` is for chess and checkers: the
    /// run's instance, or `None` when they cannot take it.
    pub fn begin_hosted_treasure_war(
        &mut self,
        table_id: u64,
        now_ms: u64,
    ) -> io::Result<(Outcome, Option<i32>)> {
        let mut out = Outcome::default();
        let table = self
            .treasure_war
            .as_deref_mut()
            .filter(|table| table.is_open());
        let Some((run, instance)) = table
            .map(|t| t.begin_hosted(table_id, now_ms))
            .transpose()?
            .flatten()
        else {
            return Ok((out, None));
        };
        out.changed = run.changed;
        out.replies = run.replies;
        Ok((out, Some(instance)))
    }

    /// The lobby game this player's run belongs to.
    pub fn hosted_treasure_war(&self) -> Option<u64> {
        self.treasure_war
            .as_deref()
            .and_then(TreasureWarTable::hosted_table)
    }

    /// Lend the World's game for one message.
    pub fn lend_treasure_war(&mut self, game: HostedMatch) {
        if let Some(table) = self.treasure_war.as_deref_mut() {
            table.lend(game);
        }
    }

    /// The lent game back, with what the other seat is owed.
    pub fn reclaim_treasure_war(&mut self) -> Option<(HostedMatch, Vec<ChessMessage>)> {
        self.treasure_war
            .as_deref_mut()
            .and_then(TreasureWarTable::reclaim)
    }
}
