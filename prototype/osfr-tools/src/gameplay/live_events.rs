//! T418: live events an admin opens and closes while the world runs.
//!
//! A quest marked `live_event` in content (the scavenger hunt, 3000200) is
//! offered and accepted only while `<live_dir>/events.json`
//! (`{"schema_version":1,"open":[ids]}`, at most 4 KiB and 64 ids) lists it.
//! `realm event open <id>`, `realm event close <id>` and `realm event list` keep
//! that file; they are admin-only through `account_store::can_run_realm_command`'s
//! default (`event` is neither a builder nor an ordinary command). A session reads
//! the file when its quest catalogue binds or its live folder changes, then polls
//! it once a second with the placement watch, so another session's open needs no
//! relog. A missing, malformed or oversized file closes every event. A closed
//! event is never disabled: an active quest still advances and turns in (the
//! gates are `QuestEngine::offer_denial` and `accept`). No packet is added: the
//! offer is the existing 49/1 (client reader 0x00c7bb60) and each command answers
//! with one `chat::system` line.
use super::*;
use std::fs;

const EVENTS_FILE: &str = "events.json";
/// (R) Sanity bounds for an admin's hand-sized list, not tuning values.
const MAX_EVENTS_BYTES: u64 = 4 * 1024;
const MAX_OPEN_EVENTS: usize = 64;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct EventsFile {
    schema_version: u32,
    open: BTreeSet<u32>,
}

/// The open ids. A non-regular, oversized or malformed file is an error, which
/// every caller treats as "everything closed"; `NotFound` passes through so the
/// watch and the commands can tell a missing file apart.
fn read_events(path: &Path) -> io::Result<BTreeSet<u32>> {
    let file = fs::File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > MAX_EVENTS_BYTES {
        return Err(invalid(
            "events.json must be a regular file of at most 4 KiB",
        ));
    }
    let mut bytes = Vec::new();
    file.take(MAX_EVENTS_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_EVENTS_BYTES {
        return Err(invalid(
            "events.json must be a regular file of at most 4 KiB",
        ));
    }
    let events: EventsFile = serde_json::from_slice(&bytes).map_err(invalid)?;
    if events.schema_version != 1 || events.open.len() > MAX_OPEN_EVENTS {
        return Err(invalid(
            "events.json needs schema_version 1 and at most 64 ids",
        ));
    }
    Ok(events.open)
}

impl Gameplay {
    fn events_path(&self) -> PathBuf {
        self.live_dir().join(EVENTS_FILE)
    }

    /// Read the open list now (catalogue bind, live folder change, own write),
    /// so an NPC Talk never waits for the first tick to see the file.
    pub(super) fn load_live_events(&mut self) {
        self.event_watch = placement_watch::PlacementWatch::default();
        self.tick_live_events(0);
    }

    /// Apply a changed `events.json` to the quest engine, at most one stat per
    /// second. A refused file closes every event; it and any id that is not a
    /// live event are logged once per file version. World state only: no
    /// packet, no save, no `Outcome::changed`; an idle tick is one comparison.
    pub(super) fn tick_live_events(&mut self, now_ms: u64) {
        if !self.event_watch.due(now_ms) {
            return;
        }
        let path = self.events_path();
        let open = match self.event_watch.poll_with(&path, now_ms, read_events) {
            Ok(Some(open)) => open,
            Ok(None) => return,
            Err(error) => {
                npc_event(serde_json::json!({
                    "event": "live_events_refused",
                    "error": error.to_string(),
                }));
                BTreeSet::new()
            }
        };
        if let Some(engine) = self.npc_quests.as_mut() {
            let ignored = engine.set_open_events(&open);
            if !ignored.is_empty() {
                npc_event(serde_json::json!({"event": "live_events_ignored", "ids": ignored}));
            }
        }
    }

    /// `realm event open <id>`, `realm event close <id>` and `realm event list`.
    /// One chat line each; a refusal writes nothing and changes nothing.
    pub(super) fn realm_event(&mut self, args: &[&str]) -> io::Result<String> {
        let Some(live) = self.npc_quests.as_ref().map(|q| q.live_events().clone()) else {
            return Ok("Live events need the quest catalogue.".into());
        };
        let (opening, id) = match args {
            ["list"] => {
                let ids = |open: bool| {
                    let ids: Vec<String> = live
                        .iter()
                        .filter(|(_, is_open)| **is_open == open)
                        .map(|(id, _)| id.to_string())
                        .collect();
                    if ids.is_empty() {
                        "none".to_owned()
                    } else {
                        ids.join(", ")
                    }
                };
                return Ok(format!(
                    "Open events: {}. Closed events: {}.",
                    ids(true),
                    ids(false)
                ));
            }
            [verb @ ("open" | "close"), id] => (*verb == "open", *id),
            _ => {
                return Ok(
                    "Usage: realm event open <quest id>, realm event close <quest id> or realm event list."
                        .into(),
                );
            }
        };
        let Some(id) = id.parse::<u32>().ok().filter(|id| live.contains_key(id)) else {
            return Ok(format!(
                "{id} is not a live event quest; realm event list names them."
            ));
        };
        let path = self.events_path();
        let mut open = match read_events(&path) {
            Ok(open) => open,
            Err(error) if error.kind() == io::ErrorKind::NotFound => BTreeSet::new(),
            Err(error) => {
                return Ok(format!(
                    "Could not read {EVENTS_FILE}: {error}. Nothing changed."
                ));
            }
        };
        let state = if opening { "open" } else { "closed" };
        let changed = if opening {
            open.insert(id)
        } else {
            open.remove(&id)
        };
        if !changed {
            return Ok(format!("Event {id} is already {state}."));
        }
        if open.len() > MAX_OPEN_EVENTS {
            return Ok(format!(
                "At most {MAX_OPEN_EVENTS} events can be open. Nothing changed."
            ));
        }
        let file = EventsFile {
            schema_version: 1,
            open,
        };
        let text = serde_json::to_string_pretty(&file).map_err(invalid)? + "\n";
        if let Err(error) = crate::character_store::replace_bytes(&path, text.as_bytes()) {
            return Ok(format!(
                "Could not save {EVENTS_FILE}: {error}. Nothing changed."
            ));
        }
        self.load_live_events(); // the acting session applies its own write at once
        Ok(format!("Event {id} is {state}."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn directory(name: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("osfr-live-events-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn the_reader_refuses_everything_outside_the_bounds() {
        let dir = directory("bounds");
        let path = dir.join(EVENTS_FILE);
        let ids = |count: u32| (1..=count).map(|id| id.to_string()).collect::<Vec<_>>();
        let cases = [
            (r#"{"schema_version":1,"open":[3000200]}"#.to_owned(), true),
            (
                format!(r#"{{"schema_version":1,"open":[{}]}}"#, ids(64).join(",")),
                true,
            ),
            (
                format!(r#"{{"schema_version":1,"open":[{}]}}"#, ids(65).join(",")),
                false,
            ),
            (r#"{"schema_version":2,"open":[3000200]}"#.to_owned(), false),
            (
                r#"{"schema_version":1,"open":[3000200],"extra":1}"#.to_owned(),
                false,
            ),
            (r#"{"schema_version":1,"open":[-1]}"#.to_owned(), false),
            (r#"{"schema_version":1,"open":["#.to_owned(), false),
            (
                format!(
                    r#"{{"schema_version":1,"open":[3000200]}}{}"#,
                    " ".repeat(4096)
                ),
                false,
            ),
        ];
        for (text, accepted) in cases {
            fs::write(&path, &text).unwrap();
            assert_eq!(
                read_events(&path).is_ok(),
                accepted,
                "{}",
                &text[..text.len().min(60)]
            );
        }
        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap();
        assert!(read_events(&path).is_err(), "a directory is refused");
        let _ = fs::remove_dir_all(&dir);
    }

    /// The tick logs a refused file when the poll errors; the watch's
    /// `Rejected` stamp makes that once per file version, not once per second.
    #[test]
    fn a_refused_file_is_reported_once_per_version() {
        let dir = directory("once");
        let path = dir.join(EVENTS_FILE);
        fs::write(&path, "{").unwrap();
        let mut watch = placement_watch::PlacementWatch::default();
        assert!(watch.poll_with(&path, 1_000, read_events).is_err());
        assert_eq!(watch.poll_with(&path, 2_000, read_events).unwrap(), None);
        assert_eq!(watch.poll_with(&path, 3_000, read_events).unwrap(), None);
        fs::remove_file(&path).unwrap();
        assert_eq!(
            watch.poll_with(&path, 4_000, read_events).unwrap(),
            Some(BTreeSet::new()),
            "a deleted file is a change to everything closed"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
