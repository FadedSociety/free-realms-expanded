//! L961 (L123's engine): Feeling the Beat!'s glue for `Gameplay`: Play on a Games window row
//! offers that game, and its lifecycle and movie packets route to it. The rules live in
//! `crate::music_game`.
use super::*;
use crate::music_game::{CONTENT_FILE, MusicContent, MusicGame};

/// The music game when `music_game.json` is present; without it no row is published.
pub(super) fn load_music(content: &Path) -> io::Result<Option<MusicGame>> {
    if !content.join(CONTENT_FILE).is_file() {
        return Ok(None);
    }
    MusicGame::new(MusicContent::load(content)?).map(Some)
}

impl Gameplay {
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
