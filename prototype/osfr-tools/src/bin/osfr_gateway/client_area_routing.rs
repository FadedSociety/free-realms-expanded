//! T457: the gateway side of `osfr_tools::client_areas`. A session that finishes loading the
//! open world gets every enabled area; season changes go to every session with the open world
//! loaded. Instances (other zones) get none: coming back loads zone 0 again and resends.
use super::*;
use osfr_tools::client_areas::{ClientAreas, unix_now_ms};

/// `world::OPEN_WORLD_ZONE`, which is crate-private.
const OPEN_WORLD_ZONE: u32 = 0;

/// `client_areas.json` from the content directory; no content means no areas.
pub(super) fn load(options: &Options) -> io::Result<ClientAreas> {
    options
        .content
        .as_deref()
        .map_or_else(|| Ok(ClientAreas::default()), ClientAreas::load)
}

fn in_open_world(gateway: &Gateway) -> bool {
    gateway.is_world_loaded() && gateway.state().zone_id == OPEN_WORLD_ZONE
}

/// On the commit whose client just finished loading the open world, every enabled area joins
/// that commit's replies (Sanctuary also sends them on ClientFinishedLoading). Staged with the
/// other replies, so a refused commit sends none.
pub(super) fn world_ready(outcome: &mut GatewayOutcome, gateway: &Gateway, world: &SharedWorld) {
    if outcome.entered_world && in_open_world(gateway) {
        let packets = world.client_areas.world_ready_packets(unix_now_ms());
        outcome
            .replies
            .extend(packets.iter().map(|packet| wrap_application(packet)));
    }
}

/// Season changes (evaluated at most once a minute) to every session with the open world
/// loaded. A refused queue is logged and skips only that session. T775: a season boundary also
/// changes which seasonal NPCs are drawn, so it forces each loaded session's shared NPC view to
/// re-diff even when the season holds no client area (and therefore sent no packet).
pub(super) fn season_tick(world: &mut SharedWorld, sessions: &mut SessionMap, unix_ms: u64) {
    let packets = world.client_areas.season_changes(unix_ms);
    let seasons_changed = world.client_areas.season_changed();
    if packets.is_empty() && !seasons_changed {
        return;
    }
    let packets = packets
        .iter()
        .map(|packet| wrap_application(packet))
        .collect::<Vec<_>>();
    for session in sessions.values_mut().filter(|s| in_open_world(&s.gateway)) {
        if seasons_changed {
            session.npc_visibility_revision = None;
        }
        if packets.is_empty() {
            continue;
        }
        let queued = packets
            .iter()
            .try_for_each(|packet| session.sender.queue_message(packet));
        if let Err(error) = queued {
            event(
                json!({"event":"client_area_send_failed","session":session.id.0,"reason":error.to_string()}),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::f120_fishing_hour_reconnect::drain_replies;
    use super::super::tests::test_support::{self, Scene, Scratch};
    use super::*;

    /// Framed opcode-89 messages: 5/1 tunnel, u32 length 11, then the reader's fields.
    fn area_messages(messages: &[Vec<u8>]) -> Vec<Vec<u8>> {
        messages
            .iter()
            .filter(|message| message.get(7..9) == Some(&[89, 0]))
            .cloned()
            .collect()
    }

    fn fields(message: &[u8]) -> (i32, bool, i32) {
        assert_eq!(message.len(), 18, "{message:?}");
        assert_eq!(&message[..7], &[5, 0, 1, 11, 0, 0, 0]);
        (
            i32::from_le_bytes(message[9..13].try_into().unwrap()),
            message[13] != 0,
            i32::from_le_bytes(message[14..18].try_into().unwrap()),
        )
    }

    /// The pad always on, and 4285 switchable for an event.
    fn pad_content(scratch: &Scratch) -> ClientAreas {
        let content = json!({
            "schema_version": 1,
            "switchable": {"area_ids": [4173, 4285], "source": "test"},
            "always_on": [{"area_id": 4173, "source": "test"}],
        });
        fs::write(
            scratch.path().join("client_areas.json"),
            content.to_string(),
        )
        .unwrap();
        ClientAreas::load(scratch.path()).unwrap()
    }

    /// The pad always on, and an area-less season (T775: it gates NPCs, never a client area).
    fn quiet_season(scratch: &Scratch) -> ClientAreas {
        let content = json!({
            "schema_version": 1,
            "switchable": {"area_ids": [4173], "source": "test"},
            "always_on": [{"area_id": 4173, "source": "test"}],
            "seasons": [{"name": "Quiet", "start_month": 3, "start_day": 1,
                "end_month": 3, "end_day": 2, "area_ids": [], "source": "test"}],
        });
        fs::write(
            scratch.path().join("client_areas.json"),
            content.to_string(),
        )
        .unwrap();
        ClientAreas::load(scratch.path()).unwrap()
    }

    /// Login, Ready and Loaded through the real dispatch and commit; the area messages each
    /// step queued.
    fn enter_world(scene: &mut Scene, options: &Options) -> [Vec<Vec<u8>>; 2] {
        scene.send(options, &[13, 0], 100).unwrap();
        let ready = area_messages(&scene.drain(100));
        scene.send(options, &[10, 0], 200).unwrap();
        [ready, area_messages(&scene.drain(200))]
    }

    #[test]
    fn loading_the_open_world_sends_the_enabled_areas_and_a_new_session_gets_them_again() {
        if !super::super::tests::test_support::bin_test_target("client-area") {
            return;
        }
        let scratch = Scratch::new("t457-load");
        let options = test_support::options(scratch.state());
        if !test_support::fixtures_present(&options) {
            eprintln!("skip t457-load: the private lab fixtures are absent");
            return;
        }
        let mut areas = pad_content(&scratch);
        assert!(areas.set_area(4285, true, unix_now_ms() - 30_000).is_some());
        let mut scene = Scene::new(&options, 1);
        scene.world.client_areas = areas.clone();
        let [ready, loaded] = enter_world(&mut scene, &options);
        assert!(
            ready.is_empty(),
            "nothing before the client finished loading"
        );
        assert_eq!(loaded.len(), 2, "{loaded:?}");
        assert_eq!(
            loaded[0],
            [
                5, 0, 1, 11, 0, 0, 0, 0x59, 0, 0x4D, 0x10, 0, 0, 1, 0, 0, 0, 0
            ]
        );
        let (id, on, elapsed) = fields(&loaded[1]);
        assert!(
            id == 4285 && on && (30_000..90_000).contains(&elapsed),
            "{elapsed}"
        );
        // Nothing is saved with the character: a new session recomputes the same areas.
        let mut again = Scene::new(&options, 2);
        again.world.client_areas = areas;
        let [_, reloaded] = enter_world(&mut again, &options);
        assert_eq!(reloaded.len(), 2);
        assert_eq!(reloaded[0], loaded[0]);
        assert_eq!(fields(&reloaded[1]).0, 4285);
    }

    /// `realm race` is the developer entry to the recovered kart scene transfer (F286).
    #[test]
    fn a_session_loading_an_instance_zone_gets_no_areas() {
        if !super::super::tests::test_support::bin_test_target("client-area") {
            return;
        }
        let scratch = Scratch::new("t457-instance");
        let mut options = test_support::options(scratch.state());
        if !test_support::fixtures_present(&options) {
            eprintln!("skip t457-instance: the private lab fixtures are absent");
            return;
        }
        options.content = Some(test_support::vendor_content(&scratch));
        options.dev_tools = true;
        let mut scene = Scene::new(&options, 3);
        scene.world.client_areas = pad_content(&scratch);
        let [_, open_world] = enter_world(&mut scene, &options);
        assert_eq!(open_world.len(), 1, "the open world shows the pad");
        let race = osfr_tools::chat::local_request("realm race");
        scene.send(&options, &race, 1_000).unwrap();
        scene.send(&options, &[13, 0], 1_100).unwrap();
        scene.send(&options, &[10, 0], 1_200).unwrap();
        assert_ne!(scene.session.gateway.state().zone_id, OPEN_WORLD_ZONE);
        assert!(scene.session.gateway.is_world_loaded());
        let instance = area_messages(&scene.drain(1_200));
        assert!(instance.is_empty(), "{instance:?}");
    }

    fn full_sender() -> ReliableSender {
        ReliableSender::new(
            Codec::new(CRC_SEED, 3, true, MAX_MESSAGE).unwrap(),
            SenderConfig {
                datagram_limit: 512,
                window: 32,
                max_message: MAX_MESSAGE,
                max_queued_bytes: 1,
                retry_ms: 250,
                max_retries: 40,
            },
        )
        .unwrap()
    }

    /// Unix ms of 2026-11-12 00:00 UTC, the day after Halloween ends.
    const HALLOWEEN_OVER: u64 = 1_794_441_600_000;

    #[test]
    fn a_season_change_reaches_loaded_sessions_and_a_full_queue_skips_only_its_own() {
        if !super::super::tests::test_support::bin_test_target("client-area") {
            return;
        }
        let scratch = Scratch::new("t457-season");
        let options = test_support::options(scratch.state());
        if !test_support::fixtures_present(&options) {
            eprintln!("skip t457-season: the private lab fixtures are absent");
            return;
        }
        let mut world = SharedWorld::new();
        let shipped = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
        world.client_areas = ClientAreas::load(&shipped).unwrap();
        let mut sessions = SessionMap::new();
        let mut keys = Vec::new();
        for id in 11..15 {
            let mut session = test_support::enter(&options, id);
            if id != 14 {
                session.gateway.handle(&wrap_application(&[13, 0])).unwrap();
                session.gateway.handle(&wrap_application(&[10, 0])).unwrap();
            }
            if id == 12 {
                session.sender = full_sender();
            }
            let key = SessionKey {
                peer: session.peer,
                connection_id: session.request.id,
            };
            keys.push(key);
            sessions.insert(key, session);
        }
        season_tick(&mut world, &mut sessions, HALLOWEEN_OVER - 60_000);
        season_tick(&mut world, &mut sessions, HALLOWEEN_OVER);
        let mut received = keys
            .iter()
            .map(|key| area_messages(&drain_replies(sessions.get_mut(key).unwrap(), 1_000)))
            .collect::<Vec<_>>();
        let expected = (3992..=4004).map(|id| (id, false, 0)).collect::<Vec<_>>();
        for index in [0, 2] {
            let got = received[index]
                .iter()
                .map(|m| fields(m))
                .collect::<Vec<_>>();
            assert_eq!(got, expected, "loaded session {index}");
        }
        assert!(
            sessions[&keys[1]].sender.is_idle(),
            "the full queue took nothing"
        );
        assert!(
            received[3].is_empty(),
            "a session that never loaded the world"
        );
        // The same minute again sends nothing to anyone.
        season_tick(&mut world, &mut sessions, HALLOWEEN_OVER + 30_000);
        received = keys
            .iter()
            .map(|key| area_messages(&drain_replies(sessions.get_mut(key).unwrap(), 2_000)))
            .collect();
        assert!(received.iter().all(Vec::is_empty));
    }

    /// Unix ms of 2026-02-28 23:59 and 2026-03-01 00:00 UTC: the area-less Quiet season opens.
    const QUIET_BEFORE: u64 = 1_772_323_140_000;
    const QUIET_OPENS: u64 = 1_772_323_200_000;

    #[test]
    fn a_season_boundary_forces_the_npc_view_without_an_area_packet() {
        if !super::super::tests::test_support::bin_test_target("client-area") {
            return;
        }
        let scratch = Scratch::new("t775-season");
        let options = test_support::options(scratch.state());
        if !test_support::fixtures_present(&options) {
            eprintln!("skip t775-season: the private lab fixtures are absent");
            return;
        }
        let mut world = SharedWorld::new();
        world.client_areas = quiet_season(&scratch);
        let mut sessions = SessionMap::new();
        let mut loaded = test_support::enter(&options, 21);
        loaded.gateway.handle(&wrap_application(&[13, 0])).unwrap();
        loaded.gateway.handle(&wrap_application(&[10, 0])).unwrap();
        // Drop the world-entry replies so the assertion below sees the season tick alone.
        let _ = drain_replies(&mut loaded, 1_000);
        loaded.npc_visibility_revision = Some(7);
        let loaded_key = SessionKey {
            peer: loaded.peer,
            connection_id: loaded.request.id,
        };
        let mut waiting = test_support::enter(&options, 22);
        waiting.npc_visibility_revision = Some(7);
        let waiting_key = SessionKey {
            peer: waiting.peer,
            connection_id: waiting.request.id,
        };
        sessions.insert(loaded_key, loaded);
        sessions.insert(waiting_key, waiting);

        season_tick(&mut world, &mut sessions, QUIET_BEFORE);
        season_tick(&mut world, &mut sessions, QUIET_OPENS);
        assert_eq!(
            sessions[&loaded_key].npc_visibility_revision, None,
            "a season opening with no client area must still force the NPC view"
        );
        assert_eq!(
            sessions[&waiting_key].npc_visibility_revision,
            Some(7),
            "a session that never loaded the open world keeps its view"
        );
        let heard = drain_replies(sessions.get_mut(&loaded_key).unwrap(), 2_000);
        assert!(
            area_messages(&heard).is_empty(),
            "an area-less season sends no 89 packet: {heard:?}"
        );
    }
}
