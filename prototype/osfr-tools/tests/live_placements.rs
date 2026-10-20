//! Live placement watch: an NPC placed or removed by one online session
//! reaches the other sessions without a relog.
//!
//! Two `Gameplay` values share one temp live directory (the World Builder
//! `--world` layer); the fixture content is the shared private opening package
//! that `gameplay/developer_teleport.rs` also uses, because `realm place` is
//! routed only while opening content is loaded. The same file is the content
//! folder's `npc_placements.json` when the server has no `--world`, so the
//! watch also covers `realm place` in that mode. The session-side view is the
//! existing solo `update_npc_visibility` (35/2 and 35/3) and the gateway's
//! `shared_npc_view` (revision-driven), not a new packet path.
use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    chat,
    gameplay::{Gameplay, Outcome},
    world::{PLACEMENT_GUID_BASE, Placement, PlacementFile},
};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

const ZONE: u32 = 0;
/// A zone that owns no placed actors. `NpcVisibility` is zone-keyed, like every
/// content NPC; private-instance filtering is the gateway view loop
/// (`Server::sync_field_views` skips a session whose membership instance differs),
/// and production child instances take the dungeon's own zone id
/// (`World::enter_dungeon`), never the open-world zone. This fixture can only
/// move between zones; `shared_world_membership` covers instance membership.
const OTHER_ZONE: u32 = 5;

fn content_directory() -> PathBuf {
    std::env::var_os("OSFR_OPENING_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/rust-opening-cooking-original-assets")
        })
}

fn live_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "osfr-live-placements-{}-{name}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn character() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: ZONE,
        position: [-1063.60595703125, 4.418806552886963, 305.6822509765625],
        rotation: [1.0, 0.0, 0.0],
    }
}

fn fixture(live: &Path) -> (Gameplay, CharacterState) {
    let content = content_directory();
    assert!(
        content.join("opening.json").is_file(),
        "missing private opening content at {}",
        content.display()
    );
    let mut game = Gameplay::load(&content, None).unwrap();
    game.set_dev_tools(true);
    game.set_role(AccountRole::Admin);
    game.set_world_dir(Some(live.to_path_buf())).unwrap();
    let character = character();
    game.update_npc_visibility(&character).unwrap();
    (game, character)
}

fn say(game: &mut Gameplay, character: &CharacterState, text: &str, now: u64) -> Outcome {
    game.handle_in_world(&chat::local_request(text), character, now)
        .unwrap()
}

fn reply_text(out: &Outcome) -> String {
    out.replies
        .iter()
        .map(|reply| String::from_utf8_lossy(reply).into_owned())
        .collect()
}

fn added(packet: &[u8]) -> Option<u64> {
    // AddNpc body after the unframed 35/2 header starts with the actor id
    // (client reader 0x00921a10 reads the 64-bit guid first).
    (packet.starts_with(&[35, 0, 2, 0]) && packet.len() >= 12)
        .then(|| u64::from_le_bytes(packet[4..12].try_into().unwrap()))
}

fn removed(packet: &[u8]) -> Option<u64> {
    // 35/3: one u16 count, then the actor id (client reader 0x008d69f0 reads
    // the family/subtype pair, the count and the guid pair).
    (packet.starts_with(&[35, 0, 3, 0]) && packet.len() >= 14)
        .then(|| u64::from_le_bytes(packet[6..14].try_into().unwrap()))
}

fn adds(replies: &[Vec<u8>], guid: u64) -> usize {
    replies
        .iter()
        .filter(|packet| added(packet) == Some(guid))
        .count()
}

fn removes(replies: &[Vec<u8>], guid: u64) -> usize {
    replies
        .iter()
        .filter(|packet| removed(packet) == Some(guid))
        .count()
}

fn changes(replies: &[Vec<u8>], guid: u64) -> usize {
    adds(replies, guid) + removes(replies, guid)
}

fn file_records(live: &Path) -> Vec<Placement> {
    let bytes = fs::read(live.join("npc_placements.json")).unwrap();
    serde_json::from_slice::<PlacementFile>(&bytes)
        .unwrap()
        .records
}

fn file_stamp(live: &Path) -> Option<SystemTime> {
    fs::metadata(live.join("npc_placements.json"))
        .ok()
        .and_then(|metadata| metadata.modified().ok())
}

/// Windows file times advance in clock ticks (~15.6 ms), so two writes can
/// share an mtime. Placement edits are human-paced; tests wait one tick before
/// the next edit so the watch gets a distinct stamp.
fn rest() {
    std::thread::sleep(Duration::from_millis(100));
}

fn place(game: &mut Gameplay, character: &CharacterState, command: &str, now: u64) -> u64 {
    let out = say(game, character, command, now);
    assert!(
        reply_text(&out).contains("Placed NPC"),
        "{}",
        reply_text(&out)
    );
    file_records(&game.live_dir()).last().unwrap().guid
}

#[test]
fn another_session_sees_a_new_placement_without_reload() {
    let live = live_dir("new-placement");
    let (mut admin, character) = fixture(&live);
    let (mut friend, _) = fixture(&live);
    assert!(!live.join("npc_placements.json").exists());

    let out = say(&mut admin, &character, "realm place 168 named Scout", 900);
    let records = file_records(&live);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].model_id, 168);
    assert_eq!(records[0].name.as_deref(), Some("Scout"));
    let guid = records[0].guid;
    assert!(guid > PLACEMENT_GUID_BASE);
    assert_eq!(adds(&out.replies, guid), 1, "the admin sees it at once");

    // The acting session stamped its own write: no remove/re-add in a tick.
    let own = admin.tick(&character, 1_000).unwrap();
    assert_eq!(changes(&own.replies, guid), 0, "no self reload");

    // The friend logged in before the placement existed and never reloaded.
    let first = friend.tick(&character, 1_000).unwrap();
    assert_eq!(adds(&first.replies, guid), 1);
    assert_eq!(removes(&first.replies, guid), 0);
    // The bytes are the existing AddNpc writer's: actor, model, literal name
    // and transform all came from the placement row.
    let add = first
        .replies
        .iter()
        .find(|packet| added(packet) == Some(guid))
        .unwrap();
    let spawn = osfr_tools::startup_world::parse_add_npc(add).unwrap();
    assert_eq!(spawn.actor_id, guid);
    assert_eq!(spawn.model_id, 168);
    assert_eq!(spawn.npc_name, "Scout");
    assert_eq!(&spawn.position[..3], &character.position);

    let second = friend.tick(&character, 2_000).unwrap();
    assert_eq!(
        changes(&second.replies, guid),
        0,
        "the unchanged file sends no duplicate"
    );

    // A reconnect loads the file through the unchanged startup path.
    let mut reconnected = Gameplay::load(&content_directory(), None).unwrap();
    reconnected.set_world_dir(Some(live.clone())).unwrap();
    let logged_in = reconnected.update_npc_visibility(&character).unwrap();
    assert_eq!(adds(&logged_in, guid), 1, "a reconnect loads the file");
    let _ = fs::remove_dir_all(&live);
}

#[test]
fn unplace_removes_it_from_other_sessions() {
    let live = live_dir("unplace");
    let (mut admin, character) = fixture(&live);
    let (mut friend, _) = fixture(&live);
    let guid = place(&mut admin, &character, "realm place 168", 900);

    let first = friend.tick(&character, 1_000).unwrap();
    assert_eq!(adds(&first.replies, guid), 1);
    let stamp = file_stamp(&live);
    rest();
    let out = say(
        &mut admin,
        &character,
        &format!("realm unplace {guid}"),
        1_100,
    );
    assert!(
        reply_text(&out).contains(&format!("Removed placed NPC {guid}")),
        "{}",
        reply_text(&out)
    );
    assert!(file_records(&live).is_empty());
    assert_ne!(file_stamp(&live), stamp, "the watcher has a new stamp");

    let removal = friend.tick(&character, 2_000).unwrap();
    assert_eq!(removes(&removal.replies, guid), 1);
    assert_eq!(adds(&removal.replies, guid), 0);

    let again = friend.tick(&character, 3_000).unwrap();
    assert_eq!(
        changes(&again.replies, guid),
        0,
        "no further packets after the removal"
    );
    let _ = fs::remove_dir_all(&live);
}

#[test]
fn far_away_placement_is_sent_only_when_in_range() {
    let live = live_dir("far");
    let (mut admin, near) = fixture(&live);
    let (mut friend, _) = fixture(&live);
    let far = CharacterState {
        position: [near.position[0] + 300.0, near.position[1], near.position[2]],
        ..near.clone()
    };
    friend.update_npc_visibility(&far).unwrap();

    let guid = place(&mut admin, &near, "realm place 168", 900);
    let ticked = friend.tick(&far, 1_000).unwrap();
    assert_eq!(changes(&ticked.replies, guid), 0, "out of range");
    assert_eq!(
        changes(&friend.update_npc_visibility(&far).unwrap(), guid),
        0
    );

    let arrived = friend.update_npc_visibility(&near).unwrap();
    assert_eq!(adds(&arrived, guid), 1, "moving in range adds it once");
    assert_eq!(
        changes(&friend.update_npc_visibility(&near).unwrap(), guid),
        0
    );

    let away = friend.update_npc_visibility(&far).unwrap();
    assert_eq!(removes(&away, guid), 1, "moving away removes it once");
    assert_eq!(
        changes(&friend.update_npc_visibility(&far).unwrap(), guid),
        0
    );
    let _ = fs::remove_dir_all(&live);
}

#[test]
fn malformed_file_keeps_the_previous_view() {
    let live = live_dir("malformed");
    let (mut admin, character) = fixture(&live);
    let (mut friend, _) = fixture(&live);
    let guid = place(&mut admin, &character, "realm place 168", 900);
    let first = friend.tick(&character, 1_000).unwrap();
    assert_eq!(adds(&first.replies, guid), 1);
    let revision = friend.npc_visibility_revision();
    let state_before = friend.state();

    rest();
    let path = live.join("npc_placements.json");
    fs::write(
        &path,
        b"{\"schema_version\":1,\"records\":[{\"guid\":130000000002",
    )
    .unwrap();
    let refused = friend.tick(&character, 2_000).unwrap();
    assert_eq!(
        changes(&refused.replies, guid),
        0,
        "a refused reload publishes nothing"
    );
    assert_eq!(
        friend.npc_visibility_revision(),
        revision,
        "the previous revision is kept"
    );
    assert_eq!(
        friend.state(),
        state_before,
        "a refused reload changes no character state"
    );
    assert_eq!(
        changes(&friend.update_npc_visibility(&character).unwrap(), guid),
        0,
        "the previous view still has the actor"
    );

    // A later valid write (here produced by a placer-shaped JSON write) is
    // applied: the old row goes and the new one comes in.
    rest();
    let next = guid + 1;
    fs::write(
        &path,
        format!(
            r#"{{"schema_version":1,"records":[{{"guid":{next},"model_id":182,"position":[{},{},{}]}}]}}"#,
            character.position[0], character.position[1], character.position[2]
        ),
    )
    .unwrap();
    let applied = friend.tick(&character, 3_000).unwrap();
    assert_eq!(removes(&applied.replies, guid), 1);
    assert_eq!(adds(&applied.replies, next), 1);
    assert_ne!(friend.npc_visibility_revision(), revision);
    assert_eq!(file_records(&live)[0].guid, next);
    let _ = fs::remove_dir_all(&live);
}

#[test]
fn poll_is_rate_limited_and_does_not_duplicate() {
    let live = live_dir("rate");
    let (mut admin, character) = fixture(&live);
    let (mut friend, _) = fixture(&live);
    let first = place(&mut admin, &character, "realm place 168", 900);
    let seen = friend.tick(&character, 1_000).unwrap();
    assert_eq!(adds(&seen.replies, first), 1);

    // A second edit inside the same one-second window is not applied yet.
    rest();
    let second = place(&mut admin, &character, "realm place 166", 1_100);
    assert_ne!(first, second, "each placement gets a unique GUID");
    assert_eq!(file_records(&live).len(), 2);
    let early = friend.tick(&character, 1_500).unwrap();
    assert_eq!(changes(&early.replies, second), 0, "rate limited");

    let applied = friend.tick(&character, 2_000).unwrap();
    assert_eq!(adds(&applied.replies, second), 1);

    // The same file mtime never re-adds either actor on later ticks.
    for now in [3_000, 4_000] {
        let settled = friend.tick(&character, now).unwrap();
        assert_eq!(changes(&settled.replies, first), 0);
        assert_eq!(changes(&settled.replies, second), 0);
    }
    let _ = fs::remove_dir_all(&live);
}

#[test]
fn two_admins_converge_to_the_final_rows() {
    let live = live_dir("two-admins");
    let (mut first_admin, character) = fixture(&live);
    let (mut second_admin, _) = fixture(&live);
    let (mut friend, _) = fixture(&live);

    let first = place(&mut first_admin, &character, "realm place 168", 900);
    rest();
    // The second writer loads the first writer's row, saves both and applies
    // both, so its own view does not lose the row another admin just wrote.
    let out = say(&mut second_admin, &character, "realm place 166", 1_100);
    assert!(
        reply_text(&out).contains("Placed NPC"),
        "{}",
        reply_text(&out)
    );
    let records = file_records(&live);
    assert_eq!(records.len(), 2);
    let guids: BTreeSet<_> = records.iter().map(|record| record.guid).collect();
    assert_eq!(guids.len(), 2, "no duplicate GUIDs");
    let second = records
        .iter()
        .map(|record| record.guid)
        .find(|guid| *guid != first)
        .unwrap();
    assert_eq!(
        adds(&out.replies, first),
        1,
        "the second writer sees row one"
    );
    assert_eq!(
        adds(&out.replies, second),
        1,
        "the second writer sees row two"
    );

    // The first writer polls once to pick up row two and then settles.
    let applied = first_admin.tick(&character, 2_000).unwrap();
    assert_eq!(adds(&applied.replies, second), 1);
    let settled = first_admin.tick(&character, 3_000).unwrap();
    assert_eq!(changes(&settled.replies, first), 0);
    assert_eq!(changes(&settled.replies, second), 0);

    // The second writer already applied both; its own poll adds nothing.
    let own = second_admin.tick(&character, 2_000).unwrap();
    assert_eq!(changes(&own.replies, first), 0);
    assert_eq!(changes(&own.replies, second), 0);

    // A friend sees both rows once and no duplicate on the same mtime.
    let seen = friend.tick(&character, 2_000).unwrap();
    assert_eq!(adds(&seen.replies, first), 1);
    assert_eq!(adds(&seen.replies, second), 1);
    let again = friend.tick(&character, 3_000).unwrap();
    assert_eq!(changes(&again.replies, first), 0);
    assert_eq!(changes(&again.replies, second), 0);
    let _ = fs::remove_dir_all(&live);
}

#[test]
fn sessions_in_another_zone_do_not_see_it() {
    let live = live_dir("zone");
    let (mut admin, character) = fixture(&live);
    let (mut friend, _) = fixture(&live);
    let elsewhere = CharacterState {
        zone_id: OTHER_ZONE,
        ..character.clone()
    };
    friend.update_npc_visibility(&elsewhere).unwrap();

    let guid = place(&mut admin, &character, "realm place 168", 900);
    let ticked = friend.tick(&elsewhere, 1_000).unwrap();
    assert_eq!(changes(&ticked.replies, guid), 0, "another zone");
    assert_eq!(
        changes(&friend.update_npc_visibility(&elsewhere).unwrap(), guid),
        0
    );

    // Transferring into the open world sends it once through the normal view.
    assert_eq!(
        adds(&friend.update_npc_visibility(&character).unwrap(), guid),
        1
    );
    assert_eq!(
        changes(&friend.update_npc_visibility(&character).unwrap(), guid),
        0
    );
    let _ = fs::remove_dir_all(&live);
}

/// World Builder edits keep the GUID and change the row fields. The watcher
/// and the acting session's own write must swap the stale actor, not skip it.
#[test]
fn a_same_guid_edit_replaces_the_actor_for_the_writer_and_a_poller() {
    let live = live_dir("same-guid");
    let (mut admin, character) = fixture(&live);
    let (mut friend, _) = fixture(&live);
    let guid = place(&mut admin, &character, "realm place 168", 900);
    let seen = friend.tick(&character, 1_000).unwrap();
    assert_eq!(adds(&seen.replies, guid), 1);

    // Another writer edits the existing row: same GUID, new fields.
    rest();
    let mut rows = file_records(&live);
    rows[0].model_id = 182;
    rows[0].name = Some("Edited".into());
    let moved = [
        character.position[0] + 3.0,
        character.position[1],
        character.position[2],
    ];
    rows[0].position = moved;
    fs::write(
        live.join("npc_placements.json"),
        serde_json::to_string(&PlacementFile {
            schema_version: 1,
            records: rows,
        })
        .unwrap(),
    )
    .unwrap();

    let applied = friend.tick(&character, 2_000).unwrap();
    assert_eq!(removes(&applied.replies, guid), 1, "the stale actor goes");
    assert_eq!(adds(&applied.replies, guid), 1, "the edited actor arrives");
    let add = applied
        .replies
        .iter()
        .find(|packet| added(packet) == Some(guid))
        .unwrap();
    let spawn = osfr_tools::startup_world::parse_add_npc(add).unwrap();
    assert_eq!(spawn.model_id, 182);
    assert_eq!(spawn.npc_name, "Edited");
    assert_eq!(&spawn.position[..3], &moved);

    // The acting admin has not polled; its own `realm place` must still apply
    // the edited row from the table it just saved.
    rest();
    let out = say(&mut admin, &character, "realm place 166", 2_100);
    assert!(
        reply_text(&out).contains("Placed NPC"),
        "{}",
        reply_text(&out)
    );
    assert_eq!(removes(&out.replies, guid), 1);
    let add = out
        .replies
        .iter()
        .find(|packet| added(packet) == Some(guid))
        .unwrap();
    let spawn = osfr_tools::startup_world::parse_add_npc(add).unwrap();
    assert_eq!(spawn.model_id, 182, "the same-GUID edit is not skipped");
    assert_eq!(spawn.npc_name, "Edited");
    assert_eq!(&spawn.position[..3], &moved);
    assert_eq!(file_records(&live).len(), 2, "the file keeps both rows");
    let _ = fs::remove_dir_all(&live);
}

/// A table the bounded reader refuses must refuse every command that reads it,
/// with no save, no view change and no rewritten file.
#[test]
fn an_unreadable_table_refuses_the_commands_without_changing_anything() {
    let live = live_dir("refuse");
    let (mut admin, character) = fixture(&live);
    let path = live.join("npc_placements.json");
    fs::write(&path, b"{\"schema_version\":1,\"records\":[").unwrap();
    let before = fs::read(&path).unwrap();
    let state = admin.state();
    let revision = admin.npc_visibility_revision();

    for command in [
        "realm place 168",
        "realm unplace nearest",
        "realm placements",
    ] {
        let out = say(&mut admin, &character, command, 1_000);
        let text = reply_text(&out);
        assert!(
            text.contains("Could not read the placements"),
            "{command}: {text}"
        );
        assert_eq!(
            fs::read(&path).unwrap(),
            before,
            "{command} rewrote the file"
        );
        assert_eq!(admin.state(), state, "{command} changed character state");
        assert_eq!(
            admin.npc_visibility_revision(),
            revision,
            "{command} changed the view"
        );
        assert!(!out.changed, "{command} requested a save");
        assert!(
            out.replies
                .iter()
                .all(|packet| !(packet.starts_with(&[35, 0, 2, 0])
                    || packet.starts_with(&[35, 0, 3, 0]))),
            "{command} published placement packets"
        );
    }

    // Oversized is refused by the same bounded reader, before any parse.
    fs::write(&path, vec![b' '; 16 * 1024 * 1024 + 1]).unwrap();
    let before = fs::read(&path).unwrap();
    let out = say(&mut admin, &character, "realm place 168", 2_000);
    assert!(
        reply_text(&out).contains("Could not read the placements"),
        "{}",
        reply_text(&out)
    );
    assert_eq!(fs::read(&path).unwrap(), before);
    let _ = fs::remove_dir_all(&live);
}

#[test]
fn shared_view_sends_the_new_placement_once_through_the_revision() {
    let live = live_dir("shared");
    let (mut admin, character) = fixture(&live);
    let (mut friend, _) = fixture(&live);
    assert!(
        friend.take_shared_fields().is_some(),
        "the opening content has a shared field"
    );
    let revision = friend.npc_visibility_revision();
    let (known, _) = friend
        .shared_npc_view(&character, &[], &BTreeSet::new())
        .unwrap();

    let guid = place(&mut admin, &character, "realm place 168", 900);
    assert!(!known.contains(&guid));

    // Shared mode publishes no direct packets; the revision is the signal the
    // gateway's 20 Hz view loop already consumes.
    let ticked = friend.tick(&character, 1_000).unwrap();
    assert_eq!(changes(&ticked.replies, guid), 0);
    assert_ne!(friend.npc_visibility_revision(), revision);

    let (wanted, packets) = friend.shared_npc_view(&character, &[], &known).unwrap();
    assert_eq!(adds(&packets, guid), 1);
    assert!(wanted.contains(&guid));
    let (again, none) = friend.shared_npc_view(&character, &[], &wanted).unwrap();
    assert_eq!(again, wanted);
    assert_eq!(changes(&none, guid), 0, "no duplicate after the first diff");
    let _ = fs::remove_dir_all(&live);
}
