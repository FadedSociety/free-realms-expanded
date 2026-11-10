//! L961 (L123's engine): Feeling the Beat!'s glue for `Gameplay`: Play on a Games window row or a
//! song picked at a music stand offers that game, and its lifecycle and movie packets route to
//! it. The rules live in `crate::music_game`.
use super::*;
use crate::{
    music_game::{CONTENT_FILE, MusicContent, MusicGame, STAND_NAME},
    npc_interaction::{TableMenu, TableMenus},
};

/// The music game when `music_game.json` is present; without it no row is published.
pub(super) fn load_music(content: &Path) -> io::Result<Option<MusicGame>> {
    if !content.join(CONTENT_FILE).is_file() {
        return Ok(None);
    }
    MusicGame::new(MusicContent::load(content)?).map(Some)
}

/// Each music stand's menu: a game table listing every song (fan wiki: the game "can be started
/// at any Music Stand").
pub(super) fn stand_menus(music: &Option<MusicGame>) -> TableMenus {
    let Some(music) = music else {
        return TableMenus::new();
    };
    let (stands, games) = music.stands();
    let menu = |position| TableMenu {
        name: STAND_NAME.to_owned(),
        position,
        games: games.clone(),
        lobbies: Vec::new(),
    };
    (stands.iter())
        .map(|stand| (stand.npc, menu(stand.position)))
        .collect()
}

impl Gameplay {
    /// (R) A song picked at a music stand is the quests' "Click on the music stand": a stand's
    /// menu has no Talk row, so the pick is the click. Any other table's key changes nothing.
    pub(super) fn music_stand_picked(&mut self, key: u64, out: &mut Outcome) -> io::Result<()> {
        let stand = |music: &MusicGame| music.stands().0.iter().any(|stand| stand.npc == key);
        if !self.music.as_ref().is_some_and(stand) {
            return Ok(());
        }
        if let Some(engine) = &mut self.npc_quests {
            let quest =
                engine.on_event(crate::quest_engine::QuestEvent::TalkToNpc { npc_id: key })?;
            self.append_npc_quest_outcome_synced(quest, out)?;
        }
        Ok(())
    }

    /// Play on a Feeling the Beat! row: the 39/16 offer, or one refusal line.
    pub(super) fn begin_music(&mut self, id: u32) -> io::Result<Outcome> {
        let refused = |text: &str| Outcome {
            replies: vec![chat::system(text)],
            ..Outcome::default()
        };
        let game = i32::try_from(id).map_err(|_| invalid("music game id"))?;
        let Some(music) = self.music.as_mut() else {
            return Ok(refused("Feeling the Beat! is not available."));
        };
        Ok(match music.begin(game)? {
            Some(run) => Outcome {
                changed: run.changed,
                replies: run.replies,
                ..Outcome::default()
            },
            None => refused("Finish your song first."),
        })
    }

    /// One packet while a song is open (`None` otherwise). As for checkers, only the lifecycle
    /// result (39/18) credits the minigame, once, and a win pays its coins first. The 39/18 score
    /// is the client's own total, so only the game id is quest authority (T787).
    pub(super) fn handle_music(
        &mut self,
        inner: &[u8],
        player_id: u64,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        if !(self.music.as_ref()).is_some_and(MusicGame::is_active) {
            return Ok(None);
        }
        let rewards = self.reward_snapshot();
        let Some(music) = self.music.as_mut() else {
            return Ok(None);
        };
        let Some(run) = music.route_with(inner, now_ms, Some((player_id, &rewards)))? else {
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
