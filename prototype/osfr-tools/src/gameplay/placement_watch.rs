//! Per-session watch of the live-world placement table.
//!
//! `realm place`/`realm unplace` write `npc_placements.json` atomically and
//! update only the acting session's `NpcVisibility`; every other session read
//! its copy at login. Once per second a session stats the file and, when its
//! mtime changed, swaps the placed rows in; the existing solo `update` or the
//! gateway shared-view diff then sends the 35/2/35/3 packets. No new actor
//! packet, broadcast or save field is involved. The client readers are the
//! existing AddNpc 35/2 (0x00921a10) and removal 35/3 (0x008d69f0).
use super::*;
use std::{fs, time::SystemTime};

/// (R) `realm place` is human-paced; one stat per session per second is the
/// smallest poll that keeps a live event responsive without per-tick I/O.
const PLACEMENT_POLL_MS: u64 = 1_000;

/// What this session has already applied. `Unknown` makes the first check
/// apply, so rows written before the session started watching are never missed.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Stamp {
    #[default]
    Unknown,
    /// The last check found no file; its reappearance is a change.
    Absent,
    /// The file at this mtime was read and refused (malformed, oversized or not
    /// a regular file). The refusal is reported once, not once per poll, and is
    /// retried only when a writer changes the stamp.
    Rejected(SystemTime),
    Modified(SystemTime),
}

#[derive(Clone, Default)]
pub struct PlacementWatch {
    stamp: Stamp,
    next_check_ms: u64,
}

impl PlacementWatch {
    /// Record the file a session just loaded or wrote, so the next poll does not
    /// remove and re-add rows that are already in its view.
    pub fn observe(&mut self, path: &Path) {
        self.stamp = match fs::metadata(path) {
            Ok(metadata) => match metadata.modified() {
                Ok(modified) => Stamp::Modified(modified),
                Err(_) => Stamp::Unknown,
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => Stamp::Absent,
            Err(_) => Stamp::Unknown,
        };
    }

    /// Forget the applied stamp so the next poll re-applies the file. Used when
    /// a read succeeded but applying it failed, so a later tick retries.
    pub fn retry(&mut self) {
        self.stamp = Stamp::Unknown;
    }

    /// True when the one-second budget allows the next stat. The tick checks
    /// this before building the file path, so an idle tick allocates nothing.
    fn due(&self, now_ms: u64) -> bool {
        now_ms >= self.next_check_ms
    }

    /// Stat at most once per second. `Some(records)` is a changed table to
    /// apply (empty after a deletion); `None` means nothing changed. A refused
    /// file is reported once per stamp, so a broken table cannot log once per
    /// second per session; the caller keeps the view it already has.
    pub fn poll(
        &mut self,
        path: &Path,
        now_ms: u64,
    ) -> io::Result<Option<Vec<crate::world::Placement>>> {
        if !self.due(now_ms) {
            return Ok(None);
        }
        self.next_check_ms = now_ms.saturating_add(PLACEMENT_POLL_MS);
        let metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                if self.stamp == Stamp::Absent {
                    return Ok(None);
                }
                self.stamp = Stamp::Absent;
                return Ok(Some(Vec::new()));
            }
            Err(error) => return Err(error),
        };
        // A filesystem without mtime cannot signal a change; keep the rows.
        let Ok(modified) = metadata.modified() else {
            return Ok(None);
        };
        if self.stamp == Stamp::Modified(modified) || self.stamp == Stamp::Rejected(modified) {
            return Ok(None);
        }
        match crate::world::read_placements(path) {
            Ok(records) => {
                self.stamp = Stamp::Modified(modified);
                Ok(Some(records))
            }
            Err(error) => {
                self.stamp = Stamp::Rejected(modified);
                Err(error)
            }
        }
    }
}

impl Gameplay {
    /// Apply the acting session's saved table with the canonical swap, so a
    /// same-GUID model/name/position edit is not mistaken for an unchanged row.
    /// The file is already saved; the stamp is recorded only after the view
    /// applied, so a failed apply is retried by the next watch poll.
    pub(super) fn apply_own_placement_write(
        &mut self,
        records: &[crate::world::Placement],
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<()> {
        if let Some(mut visibility) = self.npc_visibility.clone() {
            let mut packets = visibility.replace_placements(records)?;
            packets.extend(visibility.update(character.zone_id, character.position)?);
            self.npc_visibility = Some(visibility);
            out.replies.extend(packets);
        }
        self.placement_watch.observe(&self.placements_path());
        Ok(())
    }

    /// Apply a changed live placement table to this session's `NpcVisibility`.
    ///
    /// The poll covers `placements_path()`, which is the `--world` layer or the
    /// content folder when no world is configured; `realm place` writes to that
    /// same path in both modes. Placement visibility is world state, not a
    /// character change: no save and no `Outcome::changed`. The reload runs on a
    /// clone so a malformed file or a failed packet build keeps the previous
    /// view, and one player's bad file is logged instead of ending the session.
    pub(super) fn tick_live_placements(&mut self, now_ms: u64, out: &mut Outcome) {
        if !self.placement_watch.due(now_ms) {
            return; // No path allocation and no stat while the window is closed.
        }
        let path = self.placements_path();
        let records = match self.placement_watch.poll(&path, now_ms) {
            Ok(Some(records)) => records,
            Ok(None) => return,
            Err(error) => return placement_watch_error(error),
        };
        let Some(mut visibility) = self.npc_visibility.clone() else {
            return;
        };
        match visibility.replace_placements(&records) {
            Ok(packets) => {
                self.npc_visibility = Some(visibility);
                out.replies.extend(packets);
            }
            Err(error) => {
                // The read succeeded, so the stamp advanced; make the next tick
                // try the file again instead of treating it as applied.
                self.placement_watch.retry();
                placement_watch_error(error);
            }
        }
    }
}

fn placement_watch_error(error: io::Error) {
    npc_event(serde_json::json!({
        "event": "placement_watch_error",
        "error": error.to_string(),
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{path::PathBuf, time::Duration};

    fn directory(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "osfr-placement-watch-{}-{name}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn content_directory() -> PathBuf {
        std::env::var_os("OSFR_OPENING_REFERENCE")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../../lab/references/rust-opening-cooking-original-assets")
            })
    }

    fn character() -> CharacterState {
        CharacterState {
            guid: 1,
            zone_id: crate::world::OPEN_WORLD_ZONE,
            position: [0.0; 3],
            rotation: [1.0, 0.0, 0.0],
        }
    }

    fn records() -> &'static str {
        r#"{"schema_version":1,"records":[{"guid":130000000001,"model_id":168,"position":[1.0,2.0,3.0]}]}"#
    }

    fn adds(replies: &[Vec<u8>], guid: u64) -> usize {
        replies
            .iter()
            .filter(|packet| {
                packet.starts_with(&[35, 0, 2, 0])
                    && packet.get(4..12) == Some(&guid.to_le_bytes()[..])
            })
            .count()
    }

    /// Rewrite the file until the watch sees a new stamp. Filesystem timestamps
    /// can share a clock tick, so one rewrite is not a deterministic signal.
    fn rewrite_until_seen(watch: &mut PlacementWatch, path: &Path) -> Vec<crate::world::Placement> {
        let mut at = watch.next_check_ms.max(1_000);
        for _ in 0..100 {
            fs::write(path, records()).unwrap();
            if let Some(records) = watch.poll(path, at).unwrap() {
                return records;
            }
            at += PLACEMENT_POLL_MS;
            std::thread::sleep(Duration::from_millis(5));
        }
        panic!("the rewritten file never registered");
    }

    #[test]
    fn one_stat_per_second_and_an_unchanged_file_is_ignored() {
        let dir = directory("rate");
        let path = dir.join("npc_placements.json");
        fs::write(&path, records()).unwrap();
        let mut watch = PlacementWatch::default();
        assert_eq!(watch.poll(&path, 1_000).unwrap().unwrap().len(), 1);
        assert_eq!(watch.next_check_ms, 2_000, "one stat per second");
        assert_eq!(watch.poll(&path, 1_500).unwrap(), None, "inside the window");
        assert_eq!(watch.poll(&path, 2_000).unwrap(), None, "same mtime");
        watch.retry();
        assert_eq!(
            watch.poll(&path, 3_000).unwrap().unwrap().len(),
            1,
            "a failed apply retries"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    /// Historical test id kept for the integration test inventory: a malformed
    /// table is refused and a later write applies. The refusal is now remembered
    /// as a `Rejected` stamp, so the second poll reports nothing instead of
    /// re-erroring (and re-logging) every second.
    #[test]
    fn malformed_file_errors_without_stamping_so_a_later_write_applies() {
        let dir = directory("malformed");
        let path = dir.join("npc_placements.json");
        fs::write(&path, "{\"records\":[").unwrap();
        let mut watch = PlacementWatch::default();
        assert!(watch.poll(&path, 1_000).is_err());
        assert!(
            matches!(watch.stamp, Stamp::Rejected(_)),
            "a refused file is stamped"
        );
        assert_eq!(
            watch.poll(&path, 2_000).unwrap(),
            None,
            "one report per stamp, not one per second"
        );
        assert_eq!(rewrite_until_seen(&mut watch, &path).len(), 1);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_non_regular_file_is_refused_once() {
        let dir = directory("non-regular");
        let path = dir.join("npc_placements.json");
        fs::create_dir(&path).unwrap();
        let mut watch = PlacementWatch::default();
        assert!(watch.poll(&path, 1_000).is_err());
        assert!(matches!(watch.stamp, Stamp::Rejected(_)));
        assert_eq!(watch.poll(&path, 2_000).unwrap(), None);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_deleted_file_clears_once_and_a_reappearing_file_applies() {
        let dir = directory("deleted");
        let path = dir.join("npc_placements.json");
        let mut watch = PlacementWatch::default();
        assert_eq!(watch.poll(&path, 1_000).unwrap(), Some(Vec::new()));
        assert_eq!(watch.poll(&path, 2_000).unwrap(), None, "still absent");
        assert_eq!(rewrite_until_seen(&mut watch, &path).len(), 1);
        fs::remove_file(&path).unwrap();
        let after = watch.next_check_ms;
        assert_eq!(watch.poll(&path, after).unwrap(), Some(Vec::new()));
        assert_eq!(watch.poll(&path, after + 1_000).unwrap(), None);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn observe_records_the_loaded_or_absent_file() {
        let dir = directory("observe");
        let path = dir.join("npc_placements.json");
        fs::write(&path, records()).unwrap();
        let mut watch = PlacementWatch::default();
        watch.observe(&path);
        assert_eq!(watch.poll(&path, 1_000).unwrap(), None, "already loaded");
        let absent_path = dir.join("absent.json");
        let mut absent = PlacementWatch::default();
        absent.observe(&absent_path);
        assert_eq!(
            absent.poll(&absent_path, 1_000).unwrap(),
            None,
            "known absent is not a deletion"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    /// The reload is world visibility only: the existing AddNpc path runs, but
    /// no character state changes and no save is requested.
    #[test]
    fn applying_a_changed_file_sends_packets_without_a_character_change() {
        let content = content_directory();
        assert!(
            content.join("opening.json").is_file(),
            "missing private opening content"
        );
        let dir = directory("apply");
        let mut game = Gameplay::load(&content, None).unwrap();
        game.set_world_dir(Some(dir.clone())).unwrap();
        let character = character();
        game.update_npc_visibility(&character).unwrap();
        let before = game.state();
        fs::write(dir.join("npc_placements.json"), records()).unwrap();
        let mut out = Outcome::default();
        game.tick_live_placements(1_000, &mut out);
        assert!(
            out.replies
                .iter()
                .any(|packet| packet.starts_with(&[35, 0, 2, 0])),
            "the existing AddNpc path carries the new placement"
        );
        assert!(!out.changed, "visibility is not a character change");
        assert_eq!(game.state(), before, "no character state is touched");
        let _ = fs::remove_dir_all(&dir);
    }

    /// With no `--world`, `live_dir()` is the content folder and `realm place`
    /// still writes there; another session must see that row without a relog.
    #[test]
    fn the_content_folder_fallback_is_watched_without_a_world_dir() {
        let content = content_directory();
        assert!(
            content.join("opening.json").is_file(),
            "missing private opening content"
        );
        let dir = directory("content-fallback");
        let session = || {
            let mut game = Gameplay::load(&content, None).unwrap();
            game.content_dir = dir.clone(); // no --world: live_dir() is this folder
            game.set_world_dir(None).unwrap();
            game
        };
        let character = character();
        let mut admin = session();
        admin.set_dev_tools(true);
        admin.set_role(crate::account_store::AccountRole::Admin);
        admin.update_npc_visibility(&character).unwrap();
        let mut friend = session();
        friend.update_npc_visibility(&character).unwrap();

        let out = admin
            .handle_in_world(&chat::local_request("realm place 168"), &character, 900)
            .unwrap();
        let guid = out
            .replies
            .iter()
            .find(|packet| packet.starts_with(&[35, 0, 2, 0]) && packet.len() >= 12)
            .map(|packet| u64::from_le_bytes(packet[4..12].try_into().unwrap()))
            .expect("the acting session sees its placement");
        assert!(
            dir.join("npc_placements.json").is_file(),
            "written beside the content"
        );
        let first = friend.tick(&character, 1_000).unwrap();
        assert_eq!(
            adds(&first.replies, guid),
            1,
            "another session's tick applies the content-folder write"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
