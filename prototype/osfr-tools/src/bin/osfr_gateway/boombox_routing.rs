//! T461: the gateway side of `osfr_tools::boombox`. A committed boombox press puts the owner's
//! box in the world and shows it to everyone in range; every tick shows live boxes to sessions
//! that came into range and ends the ones whose time is up. The rules (what a box looks like,
//! who has seen it, when it ends) live in the library; this only finds the owner's pose and
//! queues the packets. A refused queue is logged and skips only that recipient.
use super::*;
use osfr_tools::boombox::{Boomboxes, Catalogue, Sends};

/// T536: the world's permanent boxes (the extra NPC rows naming a boombox), read once at startup
/// from the NPC content set `Gameplay::load` binds. No content means none. A row naming an item
/// the catalogue lacks fails the start.
pub(super) fn load(options: &Options) -> io::Result<Boomboxes> {
    let Some(dir) = options.content.as_deref() else {
        return Ok(Boomboxes::default());
    };
    let dir = osfr_tools::content::npc_content(dir);
    Boomboxes::permanent(
        &Catalogue::load(&dir)?,
        &osfr_tools::world::boombox_props(&dir)?,
    )
}

/// The owner's pressed box: placed beside them at their registry pose, after the commit that
/// refused or accepted the press has already answered the owner (recast, 38/25).
pub(super) fn place(
    owner: &mut Session,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    now_ms: u64,
    definition: i32,
) {
    let Some(row) = owner
        .gateway
        .gameplay()
        .and_then(|game| game.boombox_row(definition))
    else {
        return;
    };
    let Some((zone, at, heading)) = world
        .players
        .get(owner.id)
        .filter(|_| world.joined.contains(&owner.id))
        .map(|p| (p.zone, p.position, p.heading))
    else {
        return;
    };
    let sends = world.boomboxes.spawn(
        row,
        owner.character_guid,
        zone,
        at,
        heading,
        now_ms,
        &world.players,
    );
    deliver(sends, Some(owner), sessions, world);
}

/// Once per simulation tick. With no live box the library returns at once. A panic in here must
/// not stop the server: the boxes are dropped (logged once) and the tick goes on without them.
pub(super) fn tick(world: &mut SharedWorld, sessions: &mut SessionMap, now_ms: u64) {
    let ticked = contain(|| {
        let sends = world.boomboxes.tick(now_ms, &mut world.players);
        deliver(sends, None, sessions, world);
        Ok(())
    });
    if let Err(error) = ticked {
        // The boxes are gone, so no AddPc may keep advertising a transform they applied. The world's
        // permanent boxes start again.
        world.boomboxes = world.boomboxes.restarted();
        world.players.clear_temporary_models();
        event(json!({"event": "boombox_tick_failed", "reason": error.to_string()}));
    }
}

/// Queue each packet on its recipient's session (`owner` is the session being dispatched, which
/// is not in `sessions`). A recipient that left is skipped; one whose queue refuses gets no more.
fn deliver(
    sends: Sends,
    mut owner: Option<&mut Session>,
    sessions: &mut SessionMap,
    world: &SharedWorld,
) {
    let mut refused = BTreeSet::new();
    for (recipient, inner) in sends {
        if refused.contains(&recipient) {
            continue;
        }
        let session = match owner.as_deref_mut().filter(|o| o.id == recipient) {
            Some(owner) => Some(owner),
            None => world
                .session_keys
                .get(&recipient)
                .and_then(|key| sessions.get_mut(key)),
        };
        let Some(session) = session else { continue };
        if let Err(error) = session.queue_shared_message(&wrap_application(&inner)) {
            refused.insert(recipient);
            event(json!({
                "event": "boombox_send_refused",
                "session": recipient.0,
                "reason": error.to_string()
            }));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::f120_fishing_hour_reconnect::drain_replies;
    use super::super::tests::test_support::{self, Scene, Scratch, bin_test_target};
    use super::*;
    use osfr_tools::item_bar::use_wire::{
        ActionBarSlotUpdate, NEVER_CAST_SENTINEL, Target, decode_slot_update,
        encode_client_request_start_ability,
    };
    use osfr_tools::players::PlayerPresentation;

    const RAINBOW: i32 = 1913; // the one shipped row with a song
    const TOTEM: i32 = 8185; // no song
    const REALMS_ROLL: i32 = 1180; // the transform row
    const RROLL_MODEL: i32 = 1941; // human_m_realmsroll.adr
    const RROLL_DANCE: i32 = 3541; // emo_dance_rroll_01
    const WOLF_ITEM: i32 = 14289; // a ride item (bundled `mounts.json`, Wolf 993)
    const OWNER: u32 = 461;
    /// The high word of every box guid.
    const BOX_TAG: u64 = 0x424f_4f4d;
    const DURATION: u64 = 180_000;

    fn sender(max_queued_bytes: usize) -> ReliableSender {
        ReliableSender::new(
            Codec::new(CRC_SEED, 3, true, MAX_MESSAGE).unwrap(),
            SenderConfig {
                // Roomy, so an AddNpc is one datagram the drain below can read back.
                datagram_limit: 4096,
                window: 32,
                max_message: MAX_MESSAGE,
                max_queued_bytes,
                retry_ms: 250,
                max_retries: 40,
            },
        )
        .unwrap()
    }

    fn msg(main: u16, sub: u16, fields: &[i32]) -> Vec<u8> {
        let mut bytes = [main.to_le_bytes(), sub.to_le_bytes()].concat();
        fields.iter().for_each(|f| bytes.extend(f.to_le_bytes()));
        bytes
    }

    /// The box's packets in a drained batch, as inner packets: AddNpc (35/2), poof (35/16),
    /// song (35/41), song stop (35/42) and removal (35/3), each naming a box guid.
    fn about_box(messages: &[Vec<u8>]) -> Vec<Vec<u8>> {
        let named = |inner: &[u8]| {
            let at = match inner.get(..4) {
                Some([35, 0, 3, 0]) => 6,
                Some([35, 0, 2 | 16 | 41 | 42, 0]) => 4,
                _ => return false,
            };
            let guid = inner
                .get(at..at + 8)
                .map(|g| u64::from_le_bytes(g.try_into().unwrap()));
            guid.is_some_and(|guid| guid >> 32 == BOX_TAG)
        };
        let inners = messages.iter().filter_map(|m| m.get(7..));
        inners
            .filter(|inner| named(inner))
            .map(<[u8]>::to_vec)
            .collect()
    }

    fn subs(packets: &[Vec<u8>]) -> Vec<u16> {
        packets
            .iter()
            .map(|p| u16::from_le_bytes([p[2], p[3]]))
            .collect()
    }

    fn failed(messages: &[Vec<u8>]) -> usize {
        let inners = messages.iter().filter_map(|m| m.get(7..));
        inners
            .filter(|inner| inner.starts_with(&[36, 0, 1, 0]))
            .count()
    }

    /// The item bar's slot 0 as the owner's client is told it, from a drained batch.
    fn slot_zero(messages: &[Vec<u8>]) -> ActionBarSlotUpdate {
        let updates = messages.iter().filter_map(|m| m.get(7..));
        let mut updates = updates.filter_map(decode_slot_update);
        updates.find(|s| s.bar == 2 && s.slot == 0).unwrap()
    }

    /// When the slot was last used, as the client would work it out at `now`; `None` while the
    /// slot is ready (the server shows no cast age then).
    fn last_use(slot: &ActionBarSlotUpdate, now: u64) -> Option<i64> {
        let used = slot.cast_age_ms != NEVER_CAST_SENTINEL;
        used.then(|| now as i64 - i64::from(slot.cast_age_ms))
    }

    struct Rig {
        options: Options,
        scene: Scene,
        _scratch: Scratch,
        instance: i32,
    }

    impl Rig {
        /// The owner has entered the open world holding `item`, bound to bar slot 0.
        fn new(tag: &str, item: i32) -> Option<Self> {
            let scratch = Scratch::new(tag);
            let mut options = test_support::options(scratch.state());
            if !bin_test_target("t461") || !test_support::fixtures_present(&options) {
                eprintln!("skip {tag}: not the binary's own target, or no lab fixtures");
                return None;
            }
            let content = test_support::vendor_content(&scratch);
            let shipped = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
            for file in ["boomboxes.json", "mounts.json"] {
                fs::copy(shipped.join(file), content.join(file)).unwrap();
            }
            options.content = Some(content);
            options.dev_tools = true;
            let mut scene = Scene::new(&options, OWNER);
            scene.session.sender = sender(32 * 1024 * 1024);
            scene.send(&options, &[13, 0], 1_000).unwrap();
            scene.send(&options, &[10, 0], 1_001).unwrap();
            assert!(scene.session.gateway.is_world_loaded());
            // The server registers a session's key at accept; the scene does not.
            let (id, peer) = (scene.session.id, scene.session.peer);
            let connection_id = scene.session.request.id;
            let key = SessionKey {
                peer,
                connection_id,
            };
            scene.world.session_keys.insert(id, key);
            let mut rig = Self {
                options,
                scene,
                _scratch: scratch,
                instance: 0,
            };
            rig.give(item);
            rig.instance = rig.instance_of(item);
            rig.drain_owner(1_500);
            rig.exchange(&msg(42, 6, &[0, rig.instance]), 2_000);
            Some(rig)
        }

        fn give(&mut self, item: i32) {
            let command = osfr_tools::chat::local_request(&format!("realm give {item}"));
            self.scene.send(&self.options, &command, 1_100).unwrap();
        }

        fn instance_of(&self, item: i32) -> i32 {
            let state = self.scene.session.gateway.snapshot().gameplay.unwrap();
            let held = state.inventory.items.iter();
            held.filter(|i| i.definition_id == item).last().unwrap().id
        }

        fn exchange(&mut self, inner: &[u8], now: u64) -> Vec<Vec<u8>> {
            self.scene.exchange(&self.options, inner, now)
        }

        fn drain_owner(&mut self, now: u64) -> Vec<Vec<u8>> {
            drain_replies(&mut self.scene.session, now)
        }

        /// 36/10 on the item bar, slot 0, aimed at the player: the owner's replies.
        fn press(&mut self, now: u64) -> Vec<Vec<u8>> {
            let target = Target::Guid(self.scene.session.character_guid);
            let packet = encode_client_request_start_ability(2, 0, target);
            self.scene.send(&self.options, &packet, now).unwrap();
            self.drain_owner(now)
        }

        /// The bar as the owner's client sees it now (a no-op rebind answers with the bar).
        fn bar(&mut self, now: u64) -> ActionBarSlotUpdate {
            let rebind = msg(42, 6, &[0, self.instance]);
            slot_zero(&self.exchange(&rebind, now))
        }

        /// A second session in the open world `dx` metres east of the owner.
        fn viewer(&mut self, id: u32, dx: f32) -> SessionKey {
            self.viewer_as(id, dx, 0).0
        }

        /// As `viewer`, wearing player model `model` (the boxes dance models 1, 2, 60 and 61).
        /// Also returns the AddPc/movement messages the registry built for the join, which the
        /// real gateway delivers after `join_player`.
        fn viewer_as(
            &mut self,
            id: u32,
            dx: f32,
            model: i32,
        ) -> (SessionKey, Vec<osfr_tools::players::OutboundMessage>) {
            let mut session = test_support::enter(&self.options, id);
            session.sender = sender(32 * 1024 * 1024);
            let me = self
                .scene
                .world
                .players
                .get(self.scene.session.id)
                .unwrap()
                .clone();
            let mut look = PlayerPresentation::default();
            look.character_guid = 7_000_000_000 + u64::from(id);
            look.model = model;
            let at = [me.position[0] + dx, me.position[1], me.position[2]];
            let world = &mut self.scene.world;
            let account = 8_000_000_000 + u64::from(id);
            let (character, zone) = (look.character_guid, me.zone);
            let joined = world
                .players
                .join_with_presentation(session.id, account, character, zone, at, 0.0, look);
            let joined = joined.unwrap();
            world.joined.insert(session.id);
            let key = SessionKey {
                peer: session.peer,
                connection_id: session.request.id,
            };
            world.session_keys.insert(session.id, key);
            self.scene.sessions.insert(key, session);
            (key, joined)
        }

        fn drain(&mut self, key: SessionKey, now: u64) -> Vec<Vec<u8>> {
            drain_replies(self.scene.sessions.get_mut(&key).unwrap(), now)
        }

        /// `key`'s player now shows the temporary model `model`, keeping its base model, as a
        /// live transform's AddPc snapshot does (`PlayerPresentation::set_visual`).
        fn transform(&mut self, key: SessionKey, model: i32) {
            let id = self.scene.sessions[&key].id;
            let players = &mut self.scene.world.players;
            let mut look = players.get(id).unwrap().presentation.clone();
            look.set_visual(osfr_tools::consumable_effects::Visual {
                temporary_model: model,
                scale: 1.0,
                auras: Vec::new(),
            });
            players.set_presentation(id, look).unwrap();
        }

        /// One server tick. The owner is the scene's own session, so it joins the session map
        /// for the tick, as every session is in the real loop.
        fn tick(&mut self, now: u64) {
            let stand_in = test_support::enter(&self.options, 9_999);
            let owner = std::mem::replace(&mut self.scene.session, stand_in);
            let key = SessionKey {
                peer: owner.peer,
                connection_id: owner.request.id,
            };
            self.scene.sessions.insert(key, owner);
            tick(&mut self.scene.world, &mut self.scene.sessions, now);
            self.scene.session = self.scene.sessions.remove(&key).unwrap();
        }

        fn move_viewer(&mut self, key: SessionKey, dx: f32) {
            let players = &self.scene.world.players;
            let me = players.get(self.scene.session.id).unwrap().clone();
            let id = self.scene.sessions[&key].id;
            let at = [me.position[0] + dx, me.position[1], me.position[2]];
            let players = &mut self.scene.world.players;
            players
                .update_pose_with_state(id, me.zone, at, 0.0, 0, 0)
                .unwrap();
        }

        /// Everything the save holds about the box's owner, byte for byte.
        fn saved(&self) -> Vec<u8> {
            let state = self.scene.session.gateway.snapshot().gameplay.unwrap();
            serde_json::to_vec(&(&state.inventory, &state.item_bar)).unwrap()
        }

        /// The character's whole saved gameplay state, byte for byte.
        fn saved_state(&self) -> Vec<u8> {
            let state = self.scene.session.gateway.snapshot().gameplay.unwrap();
            serde_json::to_vec(&state).unwrap()
        }

        fn count(&self, item: i32) -> Option<u32> {
            let state = self.scene.session.gateway.snapshot().gameplay.unwrap();
            let held = state.inventory.items.iter();
            held.filter(|i| i.definition_id == item)
                .map(|i| i.count)
                .last()
        }
    }

    #[test]
    fn t461_a_press_shows_the_box_to_the_owner_and_a_near_session_and_spends_nothing() {
        let Some(mut rig) = Rig::new("t461-press", RAINBOW) else {
            return;
        };
        let near = rig.viewer(2, 10.0);
        let far = rig.viewer(3, 900.0);
        let saved = rig.saved();
        let before = rig.bar(2_100);

        let owner = rig.press(3_000);
        let near_got = about_box(&rig.drain(near, 3_000));
        // One AddNpc, one poof and one song, in that order, to the owner and the near session.
        assert_eq!(subs(&about_box(&owner)), [2, 16, 41]);
        assert_eq!(subs(&near_got), [2, 16, 41]);
        let poof = &near_got[1];
        assert_eq!(
            poof[20..24],
            21_i32.to_le_bytes(),
            "effect 21, the black smoke"
        );
        assert_eq!(failed(&owner), 0);
        assert!(
            about_box(&rig.drain(far, 3_000)).is_empty(),
            "out of range: nothing"
        );
        // Nothing was spent or saved; only the recast shows.
        assert_eq!(rig.saved(), saved);
        assert_eq!(rig.count(RAINBOW), Some(1));
        let after = rig.bar(3_100);
        assert_eq!(last_use(&before, 2_100), None, "ready before the press");
        assert_eq!(
            last_use(&after, 3_100),
            Some(3_000),
            "the press stamped the recast"
        );
        assert_eq!(after.total_refresh_time, DURATION as i32);
    }

    #[test]
    fn t461_a_second_press_inside_the_recast_is_refused_and_one_after_it_works() {
        let Some(mut rig) = Rig::new("t461-recast", TOTEM) else {
            return;
        };
        let near = rig.viewer(2, 10.0);
        let first = rig.press(3_000);
        assert_eq!(subs(&about_box(&first)), [2, 16], "no song, so no 35/41");
        rig.drain(near, 3_000);

        for now in [3_501, 3_000 + DURATION - 1] {
            let refused = rig.press(now);
            assert_eq!(failed(&refused), 1, "one 36/1 at {now}");
            assert!(about_box(&refused).is_empty());
            assert!(about_box(&rig.drain(near, now)).is_empty());
        }
        let bar = rig.bar(3_000 + DURATION - 1);
        let stamp = last_use(&bar, 3_000 + DURATION - 1);
        assert_eq!(stamp, Some(3_000), "refusals never restamp");

        // Exactly at the recast it works again. No tick has ended the first box, so the second
        // press replaces it: the old box's poof and removal, then the new box and its poof.
        let again = about_box(&rig.press(3_000 + DURATION));
        assert_eq!(subs(&again), [16, 3, 2, 16]);
        assert_eq!(
            subs(&about_box(&rig.drain(near, 3_000 + DURATION))),
            [16, 3, 2, 16]
        );
    }

    #[test]
    fn t461_a_press_is_refused_when_mounted_or_knocked_out_and_changes_nothing() {
        for case in ["mounted", "knocked out"] {
            let Some(mut rig) = Rig::new("t461-refuse", RAINBOW) else {
                return;
            };
            let near = rig.viewer(2, 10.0);
            let now = 5_000;
            match case {
                "mounted" => {
                    rig.give(WOLF_ITEM);
                    // The client echoes the row id of the 168/5 roster the server sent it.
                    let roster = rig.drain_owner(now - 10);
                    let list = roster.iter().filter_map(|m| m.get(7..));
                    let list = list.filter(|inner| inner.starts_with(&[168, 0, 5])).last();
                    let row = list.expect("the roster after the grant")[7..11].to_vec();
                    let spawn = [vec![168, 0, 6], row].concat();
                    rig.exchange(&spawn, now);
                    let guid = rig.scene.session.character_guid;
                    let ride = osfr_tools::mounts::MountService::ride_guid(guid);
                    let board = [
                        vec![168, 0, 1],
                        ride.to_le_bytes().to_vec(),
                        vec![1, 0, 0, 0, 0, 0],
                    ];
                    rig.exchange(&board.concat(), now + 10);
                    assert!(rig.scene.session.gateway.gameplay().unwrap().is_mounted());
                }
                "knocked out" => {
                    let state = rig.scene.session.gateway.snapshot().gameplay.unwrap();
                    let full = state.encounter.unwrap().player_health;
                    let gateway = &mut rig.scene.session.gateway;
                    gateway.receive_field_damage(full, now, true).unwrap();
                }
                _ => unreachable!(),
            }
            rig.drain_owner(now + 30);
            rig.drain(near, now + 30);
            let saved = rig.saved();
            let refused = rig.press(now + 50);
            assert_eq!(failed(&refused), 1, "{case}: one 36/1");
            assert!(about_box(&refused).is_empty(), "{case}");
            assert!(about_box(&rig.drain(near, now + 50)).is_empty(), "{case}");
            assert_eq!(rig.saved(), saved, "{case}: inventory and bar");
            let bar = rig.bar(now + 60);
            assert_eq!(
                last_use(&bar, now + 60),
                None,
                "{case}: the recast is untouched"
            );
        }
    }

    #[test]
    fn t461_a_session_that_comes_into_range_later_is_shown_the_box_once() {
        let Some(mut rig) = Rig::new("t461-late", RAINBOW) else {
            return;
        };
        let late = rig.viewer(2, 900.0);
        rig.press(3_000);
        rig.tick(3_050);
        assert!(about_box(&rig.drain(late, 3_050)).is_empty());
        rig.move_viewer(late, 12.0);
        rig.tick(3_100);
        let shown = about_box(&rig.drain(late, 3_100));
        assert_eq!(subs(&shown), [2, 41], "the box and its song, no poof");
        rig.tick(3_150);
        rig.tick(3_200);
        assert!(
            about_box(&rig.drain(late, 3_200)).is_empty(),
            "once, not every tick"
        );
    }

    #[test]
    fn t461_expiry_ends_the_box_for_every_viewer_once_and_a_gone_viewer_gets_nothing() {
        let Some(mut rig) = Rig::new("t461-expiry", RAINBOW) else {
            return;
        };
        let near = rig.viewer(2, 10.0);
        let gone = rig.viewer(3, 12.0);
        rig.press(3_000);
        rig.tick(3_000);
        rig.drain_owner(3_000);
        rig.drain(near, 3_000);
        rig.drain(gone, 3_000);
        // The third viewer disconnects before the box ends.
        let gone_id = rig.scene.sessions[&gone].id;
        rig.scene.sessions.remove(&gone);
        rig.scene.world.session_keys.remove(&gone_id);
        rig.scene.world.joined.remove(&gone_id);
        let _ = rig.scene.world.players.leave(gone_id);

        let end = 3_000 + DURATION;
        rig.tick(end - 1);
        assert!(
            about_box(&rig.drain_owner(end - 1)).is_empty(),
            "not before its time"
        );
        rig.tick(end);
        assert_eq!(subs(&about_box(&rig.drain_owner(end))), [42, 16, 3]);
        assert_eq!(subs(&about_box(&rig.drain(near, end))), [42, 16, 3]);
        rig.tick(end + 50);
        assert!(
            about_box(&rig.drain_owner(end + 50)).is_empty(),
            "then nothing"
        );
        assert!(about_box(&rig.drain(near, end + 50)).is_empty());
    }

    #[test]
    fn t461_the_box_outlives_its_owners_session_and_a_new_session_presses_at_once() {
        let Some(mut rig) = Rig::new("t461-relog", TOTEM) else {
            return;
        };
        let near = rig.viewer(2, 10.0);
        rig.press(3_000);
        rig.drain(near, 3_000);
        // The owner disconnects: the box stays for the viewer until it ends.
        let id = rig.scene.session.id;
        rig.scene.world.session_keys.remove(&id);
        rig.scene.world.joined.remove(&id);
        let _ = rig.scene.world.players.leave(id);
        rig.tick(3_000 + DURATION - 1);
        assert!(about_box(&rig.drain(near, 3_000 + DURATION - 1)).is_empty());
        rig.tick(3_000 + DURATION);
        assert_eq!(
            subs(&about_box(&rig.drain(near, 3_000 + DURATION))),
            [16, 3]
        );

        // A new session of the same character (the bar is saved, the recast is not) can press.
        let mut again = Scene::new(&rig.options, OWNER + 1);
        again.session.sender = sender(32 * 1024 * 1024);
        again.send(&rig.options, &[13, 0], 9_000).unwrap();
        again.send(&rig.options, &[10, 0], 9_001).unwrap();
        let target = Target::Guid(again.session.character_guid);
        let packet = encode_client_request_start_ability(2, 0, target);
        again.send(&rig.options, &packet, 9_002).unwrap();
        let sent = drain_replies(&mut again.session, 9_002);
        assert_eq!(failed(&sent), 0);
        assert_eq!(
            subs(&about_box(&sent)),
            [2, 16],
            "the reloaded session's press works"
        );
    }

    #[test]
    fn t461_a_refusing_queue_skips_only_its_own_viewer_and_the_tick_goes_on() {
        let Some(mut rig) = Rig::new("t461-queue", RAINBOW) else {
            return;
        };
        let stuck = rig.viewer(2, 10.0);
        rig.press(3_000);
        // The stuck viewer's queue refuses everything from here on.
        rig.scene.sessions.get_mut(&stuck).unwrap().sender = sender(1);
        let healthy = rig.viewer(3, 12.0);
        rig.tick(3_050);
        let shown = about_box(&rig.drain(healthy, 3_050));
        assert_eq!(subs(&shown), [2, 41], "the other viewer is still served");
        rig.tick(3_000 + DURATION);
        let ended = about_box(&rig.drain(healthy, 3_000 + DURATION));
        assert_eq!(subs(&ended), [42, 16, 3]);
    }

    /// The `(model, guid)` of every 35/14 in a drained batch (client reader `008d6890`).
    fn temporary_models(messages: &[Vec<u8>]) -> Vec<(i32, u64)> {
        let inners = messages.iter().filter_map(|m| m.get(7..));
        let models = inners.filter(|inner| inner.starts_with(&[35, 0, 14, 0]));
        models
            .map(|p| {
                assert_eq!(p.len(), 16);
                (
                    i32::from_le_bytes(p[4..8].try_into().unwrap()),
                    u64::from_le_bytes(p[8..16].try_into().unwrap()),
                )
            })
            .collect()
    }

    /// The guids of every 35/15 in a drained batch (client reader `008d7230`).
    fn reverts(messages: &[Vec<u8>]) -> Vec<u64> {
        let inners = messages.iter().filter_map(|m| m.get(7..));
        let reverts = inners.filter(|inner| inner.starts_with(&[35, 0, 15, 0]));
        reverts
            .map(|p| {
                assert_eq!(p.len(), 16);
                assert_eq!(p[12..16], 0_i32.to_le_bytes());
                u64::from_le_bytes(p[4..12].try_into().unwrap())
            })
            .collect()
    }

    /// The `(guid, animation, base)` of every 35/8 in a drained batch.
    fn animations(messages: &[Vec<u8>]) -> Vec<(u64, i32, bool)> {
        let inners = messages.iter().filter_map(|m| m.get(7..));
        let animations = inners.filter(|inner| inner.starts_with(&[35, 0, 8, 0]));
        animations
            .map(|p| {
                (
                    u64::from_le_bytes(p[4..12].try_into().unwrap()),
                    i32::from_le_bytes(p[12..16].try_into().unwrap()),
                    p[20] == 1,
                )
            })
            .collect()
    }

    /// The AddPc a joining viewer was sent for `guid`.
    fn add_pc_of(messages: &[osfr_tools::players::OutboundMessage], guid: u64) -> Option<&[u8]> {
        messages
            .iter()
            .filter(|m| m.kind == osfr_tools::players::MessageKind::Spawn)
            .map(|m| m.bytes.as_slice())
            .find(|bytes| bytes.get(11..19) == Some(&guid.to_le_bytes()[..]))
    }

    /// The temporary appearance at +0x144 (client writer `0091adcf`): 68 bytes follow it (guild,
    /// active profile, four title dwords, effects, the mount block, wield and VIP), so it is at
    /// `inner.len() - 72` for every presentation.
    fn add_pc_model(frame: &[u8]) -> i32 {
        let inner = &frame[7..];
        assert!(inner.starts_with(&[35, 0, 1, 0]));
        let at = inner.len() - 72;
        i32::from_le_bytes(inner[at..at + 4].try_into().unwrap())
    }

    /// The `(guid, animation)` entries of every 35/63 in a drained batch.
    fn sync_steps(messages: &[Vec<u8>]) -> Vec<Vec<(u64, i32)>> {
        let inners = messages.iter().filter_map(|m| m.get(7..));
        let sync = inners.filter(|inner| inner.starts_with(&[35, 0, 63, 0]));
        sync.map(|p| {
            let count = i32::from_le_bytes(p[4..8].try_into().unwrap()) as usize;
            let entry = |i: usize| {
                let at = 8 + 12 * i;
                let guid = u64::from_le_bytes(p[at..at + 8].try_into().unwrap());
                (
                    guid,
                    i32::from_le_bytes(p[at + 8..at + 12].try_into().unwrap()),
                )
            };
            (0..count).map(entry).collect()
        })
        .collect()
    }

    /// The characters a drained batch tells to stand idle (35/8, animation 1).
    fn idled(messages: &[Vec<u8>]) -> Vec<u64> {
        let inners = messages.iter().filter_map(|m| m.get(7..));
        let idle = inners.filter(|inner| inner.starts_with(&[35, 0, 8, 0]));
        idle.map(|p| u64::from_le_bytes(p[4..12].try_into().unwrap()))
            .collect()
    }

    #[test]
    fn t463_two_dancers_get_the_step_through_the_real_tick_and_expiry_stops_them() {
        let Some(mut rig) = Rig::new("t463-dance", TOTEM) else {
            return;
        };
        let near = rig.viewer_as(2, 6.0, 60).0;
        let bystander = rig.viewer_as(3, 8.0, 0).0; // no model with clips: watches only
        let dancer = 7_000_000_002;
        rig.press(3_000);
        rig.drain(near, 3_000);
        rig.drain(bystander, 3_000);
        rig.tick(3_050);
        let first = sync_steps(&rig.drain(near, 3_050));
        // TOTEM is a one-step sequence: 3551, 17567 ms clip less the 600 ms blend.
        assert_eq!(first.len(), 1);
        assert!(first[0].contains(&(dancer, 3_551)), "{first:?}");
        assert!(first[0].iter().all(|(_, anim)| *anim == 3_551));
        let watched = sync_steps(&rig.drain(bystander, 3_050));
        assert!(
            watched
                .iter()
                .flatten()
                .all(|(guid, _)| *guid != 7_000_000_003)
        );
        rig.tick(3_100);
        assert!(
            sync_steps(&rig.drain(near, 3_100)).is_empty(),
            "once per step"
        );
        // The step repeats when it ends (the box lasts 180 s, the loop 16967 ms).
        rig.tick(3_050 + 16_967);
        let again = sync_steps(&rig.drain(near, 3_050 + 16_967));
        assert!(again[0].contains(&(dancer, 3_551)));
        // The box ends: the dancer is sent idle.
        rig.drain(near, 3_000 + DURATION);
        rig.tick(3_000 + DURATION);
        assert_eq!(idled(&rig.drain(near, 3_000 + DURATION)), [dancer]);
        rig.tick(3_000 + DURATION + 50);
        assert!(idled(&rig.drain(near, 3_000 + DURATION + 50)).is_empty());
    }

    /// The base 35/8 `(guid, animation)` pairs a drained batch carries.
    fn based(messages: &[Vec<u8>]) -> Vec<(u64, i32)> {
        let inners = messages.iter().filter_map(|m| m.get(7..));
        let base = inners.filter(|inner| inner.starts_with(&[35, 0, 8, 0]));
        base.map(|p| {
            (
                u64::from_le_bytes(p[4..12].try_into().unwrap()),
                i32::from_le_bytes(p[12..16].try_into().unwrap()),
            )
        })
        .collect()
    }

    #[test]
    fn t494_a_randomizer_rolls_a_model_clip_and_a_model_without_one_stands() {
        let Some(mut rig) = Rig::new("t494-random", RAINBOW) else {
            return;
        };
        let dancer = rig.viewer_as(2, 6.0, 60).0; // human_f: has a Rainbow table
        let second = rig.viewer_as(4, 10.0, 1).0; // human_m: has one too
        let standing = rig.viewer_as(3, 8.0, 1).0;
        // Base model 1 has a table, but a live transform shows the Realms Roll model 1941: no
        // table, so the client shows a model that must stand.
        rig.transform(standing, 1941);
        let owner = rig.press(3_000);
        // The box, its poof and one of the six colour songs: red..yellow (16448-16453).
        assert_eq!(subs(&about_box(&owner)), [2, 16, 41]);
        let song = about_box(&owner)
            .into_iter()
            .find(|inner| inner.starts_with(&[35, 0, 41, 0]))
            .unwrap();
        let effect = i32::from_le_bytes(song[16..20].try_into().unwrap());
        assert!((16_448..=16_453).contains(&effect), "{effect}");
        rig.drain(dancer, 3_000);
        rig.drain(second, 3_000);
        rig.drain(standing, 3_000);
        rig.tick(3_050);
        // Each table dancer gets its own single-entry 35/63; never one packet with both entries.
        let rolled = sync_steps(&rig.drain(dancer, 3_050));
        assert_eq!(rolled.len(), 2, "one single-entry step per dancer");
        assert!(rolled.iter().all(|step| step.len() == 1), "{rolled:?}");
        let entries: Vec<_> = rolled.iter().flatten().copied().collect();
        assert_eq!(
            entries.iter().map(|(guid, _)| *guid).collect::<Vec<_>>(),
            [7_000_000_002, 7_000_000_004]
        );
        assert!(entries.iter().all(|(_, animation)| *animation > 0));
        let seen = sync_steps(&rig.drain(second, 3_050));
        assert_eq!(seen.len(), 2, "the second dancer sees both, singly");
        assert!(seen.iter().all(|step| step.len() == 1));
        assert_eq!(seen.into_iter().flatten().collect::<Vec<_>>(), entries);
        // The transformed model stands on the row's group (3550) as a base animation, and is
        // never listed in a 35/63. The owner (default model, no table) stands too.
        let standing_got = rig.drain(standing, 3_050);
        assert!(
            sync_steps(&standing_got)
                .iter()
                .flatten()
                .all(|(guid, _)| *guid != 7_000_000_003)
        );
        let bases = based(&standing_got);
        assert!(bases.contains(&(7_000_000_003, 3_550)), "{bases:?}");
        assert!(bases.contains(&(1, 3_550)), "the owner stands: {bases:?}");
        assert!(bases.iter().all(|(_, animation)| *animation == 3_550));
    }

    #[test]
    fn t492_a_pressed_realms_roll_transforms_and_a_late_viewer_sees_the_model() {
        let Some(mut rig) = Rig::new("t492-roll", REALMS_ROLL) else {
            return;
        };
        let owner = rig.scene.session.character_guid;
        // 20 m away: in the registry's interest radius (so it is a directed viewer) but out of
        // the box's 15 m range, so it is not transformed itself.
        let (near, joined) = rig.viewer_as(2, 20.0, 0);
        let near_character = 7_000_000_002;
        let before = add_pc_of(&joined, owner).expect("the owner's AddPc");
        assert_eq!(add_pc_model(before), 0, "plain before the press");
        let saved = rig.saved_state();

        rig.press(3_000);
        rig.drain(near, 3_000);
        rig.tick(3_050);
        let owner_packets = rig.drain_owner(3_050);
        let near_packets = rig.drain(near, 3_050);
        for messages in [&owner_packets, &near_packets] {
            assert_eq!(temporary_models(messages), [(RROLL_MODEL, owner)]);
            assert_eq!(animations(messages), [(owner, RROLL_DANCE, true)]);
            let at = |sub: u8| {
                messages
                    .iter()
                    .position(|m| m.get(7..).is_some_and(|i| i.starts_with(&[35, 0, sub, 0])))
                    .unwrap()
            };
            assert!(at(14) < at(8), "35/14 before the base animation");
        }
        assert!(
            temporary_models(&near_packets)
                .iter()
                .all(|(_, guid)| *guid != near_character),
            "the 20 m viewer is outside the box's range"
        );
        // A viewer who joins now sees the model in the owner's AddPc.
        let (_later, later_join) = rig.viewer_as(3, 25.0, 0);
        let frame = add_pc_of(&later_join, owner).expect("the owner's AddPc");
        assert_eq!(add_pc_model(frame), RROLL_MODEL);
        assert_eq!(
            rig.saved_state(),
            saved,
            "the transform is session-only: the save is unchanged"
        );
    }

    #[test]
    fn t492_a_refusing_viewer_queue_skips_only_that_viewer_and_the_tick_goes_on() {
        let Some(mut rig) = Rig::new("t492-queue", REALMS_ROLL) else {
            return;
        };
        let owner = rig.scene.session.character_guid;
        let stuck = rig.viewer(2, 6.0);
        let healthy = rig.viewer(3, 8.0);
        // The stuck viewer's queue refuses everything from here on.
        rig.scene.sessions.get_mut(&stuck).unwrap().sender = sender(1);
        rig.press(3_000);
        rig.drain(healthy, 3_000);
        rig.tick(3_050);
        let served = temporary_models(&rig.drain(healthy, 3_050));
        // All three in-range players are transformed; the healthy viewer hears about all of
        // them, including the viewer whose own queue refused.
        assert_eq!(served.len(), 3, "{served:?}");
        assert!(
            served.iter().all(|(model, _)| *model == RROLL_MODEL),
            "{served:?}"
        );
        for guid in [owner, 7_000_000_002, 7_000_000_003] {
            assert!(served.iter().any(|(_, named)| *named == guid), "{guid}");
        }
        // The box and the other players carry on: expiry still reverts all three.
        rig.tick(3_000 + DURATION);
        assert_eq!(
            reverts(&rig.drain(healthy, 3_000 + DURATION)).len(),
            3,
            "the shared tick kept running"
        );
    }

    #[test]
    fn t492_leaving_range_reverts_and_a_later_viewer_sees_them_plain() {
        let Some(mut rig) = Rig::new("t492-leave", REALMS_ROLL) else {
            return;
        };
        let (viewer, _) = rig.viewer_as(2, 6.0, 0);
        let me = 7_000_000_002;
        rig.press(3_000);
        rig.drain(viewer, 3_000);
        rig.tick(3_050);
        let mut shown = Vec::new();
        shown.extend(temporary_models(&rig.drain(viewer, 3_050)));
        shown.extend(temporary_models(&rig.drain_owner(3_050)));
        assert!(shown.contains(&(RROLL_MODEL, me)), "{shown:?}");
        rig.move_viewer(viewer, 40.0);
        rig.tick(3_100);
        let viewer_packets = rig.drain(viewer, 3_100);
        let owner_packets = rig.drain_owner(3_100);
        for messages in [&viewer_packets, &owner_packets] {
            assert_eq!(reverts(messages), [me]);
            assert_eq!(idled(messages), [me], "then loc_stand");
        }
        // A viewer who joins now gets the plain model.
        let (_later, later_join) = rig.viewer_as(3, 45.0, 0);
        let frame = add_pc_of(&later_join, me).expect("the player's AddPc");
        assert_eq!(add_pc_model(frame), 0);
    }

    /// T536: a catalogue-bound radio prop at `at`, as the startup load builds one.
    fn radio_at(at: [f32; 3]) -> osfr_tools::boombox::Boomboxes {
        let shipped = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let prop = osfr_tools::world::BoomboxProp {
            spawn_guid: 100_000_033_617,
            item_id: 48244, // Boombox XVD-S
            position: at,
        };
        let catalogue = osfr_tools::boombox::Catalogue::load(&shipped).unwrap();
        osfr_tools::boombox::Boomboxes::permanent(&catalogue, &[prop]).unwrap()
    }

    #[test]
    fn t536_a_radio_dances_a_player_through_the_real_tick_and_never_sends_the_box_or_expires() {
        let Some(mut rig) = Rig::new("t536-radio", TOTEM) else {
            return;
        };
        let (viewer, _) = rig.viewer_as(2, 4.0, 60);
        let dancer = 7_000_000_002;
        let me = rig.scene.world.players.get(rig.scene.session.id).unwrap();
        let at = [me.position[0] + 3.0, me.position[1], me.position[2]];
        rig.scene.world.boomboxes = radio_at(at);
        rig.drain(viewer, 3_000);
        rig.drain_owner(3_000);
        // One tick: the model-60 player gets a clip of the XVD-S table as a single-entry 35/63.
        rig.tick(3_050);
        let table = {
            let shipped = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
            let row = osfr_tools::boombox::Catalogue::load(&shipped).unwrap();
            let row = row.get(48244).unwrap().clone();
            row.clips[&60]
                .iter()
                .map(|c| c.animation_id)
                .collect::<Vec<_>>()
        };
        let first = rig.drain(viewer, 3_050);
        let steps = sync_steps(&first);
        assert_eq!(steps.len(), 1, "{steps:?}");
        assert_eq!(steps[0].len(), 1);
        assert_eq!(steps[0][0].0, dancer);
        assert!(table.contains(&steps[0][0].1), "{steps:?}");
        // Ten days later the radio is still there: it dances the player again, and neither
        // session was ever sent an AddNpc, poof, song or removal for it.
        let later = 3_050 + 10 * 24 * 3_600 * 1_000;
        rig.drain_owner(3_050);
        rig.tick(later);
        let again = rig.drain(viewer, later);
        assert_eq!(sync_steps(&again).len(), 1, "still dancing: {again:?}");
        for messages in [&first, &again, &rig.drain_owner(later)] {
            assert!(about_box(messages).is_empty(), "no box packets");
        }
        // Walking out of range stands the dancer down once; walking back, the radio dances them.
        rig.move_viewer(viewer, 40.0);
        rig.tick(later + 100);
        assert_eq!(idled(&rig.drain(viewer, later + 100)), [dancer]);
        rig.move_viewer(viewer, 4.0);
        rig.tick(later + 200);
        let back = rig.drain(viewer, later + 200);
        assert_eq!(
            sync_steps(&back).len(),
            1,
            "the radio is still there: {back:?}"
        );
    }

    /// A content directory carrying an NPC package: an `npcs` folder and `rows` as the extras.
    fn npc_package(scratch: &Scratch, rows: Vec<serde_json::Value>) -> PathBuf {
        let dir = scratch.path().join("content");
        fs::create_dir_all(dir.join("npcs")).unwrap();
        let file = json!({"schema_version": 1, "records": rows});
        fs::write(dir.join("npc_spawns_extra.json"), file.to_string()).unwrap();
        dir
    }

    #[test]
    fn t536_startup_seeds_the_shipped_radios_and_a_missing_item_fails_the_load() {
        let scratch = Scratch::new("t536-load");
        let mut options = test_support::options(scratch.state());
        // A content directory without an NPC package binds the bundled one, which has the radios.
        options.content = Some(scratch.path().to_path_buf());
        let mut world = SharedWorld::new();
        world.boomboxes = load(&options).unwrap();
        let zone = ZoneInstance {
            zone_id: 0,
            instance_id: 0,
        };
        // Session 1 stands at the first radio, session 2 at the second, session 3 far from both.
        for (id, at) in [
            (1, [88.9, 23.5, 410.6]),
            (2, [-63.0, 5.9, 327.7]),
            (3, [5000.0, 0.0, 5000.0]),
        ] {
            let joined = world
                .players
                .join(SessionId(id), id + 500, id + 600, zone, at, 0.0);
            joined.unwrap();
        }
        let sends = world.boomboxes.tick(1_000, &mut world.players);
        let dancing: BTreeSet<_> = sends.iter().map(|(session, _)| session.0).collect();
        assert_eq!(
            dancing,
            BTreeSet::from([1, 2]),
            "both radios, and only near them"
        );
        // No content means no radios.
        options.content = None;
        let mut bare = SharedWorld::new();
        bare.boomboxes = load(&options).unwrap();
        bare.players
            .join(SessionId(1), 501, 601, zone, [88.9, 23.5, 410.6], 0.0)
            .unwrap();
        assert!(bare.boomboxes.tick(1_000, &mut bare.players).is_empty());
        // A row naming an item the catalogue lacks fails the start, naming the row.
        let row = json!({"spawn_guid": 42, "model_id": 185, "position": [0.0, 0.0, 0.0],
            "boombox_item_id": 999_999});
        options.content = Some(npc_package(&scratch, vec![row]));
        let error = load(&options).unwrap_err().to_string();
        assert!(
            error.contains("NPC spawn 42") && error.contains("999999"),
            "{error}"
        );
    }
}
