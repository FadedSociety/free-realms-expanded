//! T557: the gateway side of `osfr_tools::placed_cakes`. A committed cake press puts the cake in
//! the shared world at the placer's registry pose and shows it to every session near it; a
//! committed click on a live cake rolls an outcome for the clicker and applies it through the
//! consumable effect stack; every tick despawns old cakes and plays the boss cakes' animations
//! for the sessions near each one. The rules (the roll, the window, the lifetime) live in the
//! library; this finds poses and recipients and queues packets.
//!
//! (R) A session that comes into range after the spawn is not shown the cake (it lives 60 s), and
//! one that left range before the despawn keeps the actor until its next scene load. A refused
//! click is silent. The cap and the cake table are `placed_cakes.json`'s.
use super::*;
use osfr_tools::placed_cakes::{Action, Catalogue, PlacedCakes};

/// The world's live cakes and the table they are placed from. Session-only: never saved.
#[derive(Clone, Debug, Default)]
pub(super) struct Cakes {
    table: Catalogue,
    live: PlacedCakes,
}

/// `placed_cakes.json` from the NPC content set `Gameplay::load` binds (the item bar reads the
/// same file). No content means no cakes: nothing can name one.
pub(super) fn load(options: &Options) -> io::Result<Cakes> {
    let Some(dir) = options.content.as_deref() else {
        return Ok(Cakes::default());
    };
    let table = Catalogue::load(&osfr_tools::content::npc_content(dir))?;
    Ok(Cakes {
        live: PlacedCakes::new(table.max_live()),
        table,
    })
}

/// Queue one inner packet on `session`; a queue that refuses is logged and skips only it.
fn send(session: &mut Session, inner: &[u8]) {
    if let Err(error) = session.queue_shared_message(&wrap_application(inner)) {
        event(json!({
            "event": "cake_send_refused",
            "session": session.id.0,
            "reason": error.to_string()
        }));
    }
}

/// The session `id`: the one being dispatched (it is not in `sessions`) or a stored one.
fn session_of<'a>(
    owner: &'a mut Session,
    sessions: &'a mut SessionMap,
    world: &SharedWorld,
    id: SessionId,
) -> Option<&'a mut Session> {
    if owner.id == id {
        return Some(owner);
    }
    world
        .session_keys
        .get(&id)
        .and_then(|key| sessions.get_mut(key))
}

/// `owner`'s zone instance and position in the registry, never a packet's.
fn pose(owner: &Session, world: &SharedWorld) -> Option<(ZoneInstance, [f32; 3], f32)> {
    let player = world.players.get(owner.id);
    let player = player.filter(|_| world.joined.contains(&owner.id))?;
    Some((player.zone, player.position, player.heading))
}

/// A committed press or click. Everything here answers to the one session that acted; no error
/// reaches another session.
pub(super) fn apply(
    owner: &mut Session,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    now_ms: u64,
    action: Action,
) {
    match action {
        Action::Place { row } => place(owner, sessions, world, now_ms, row),
        Action::Click { guid } => click(owner, sessions, world, now_ms, guid),
    }
}

/// The press was committed and its recast stamped; a place the world cannot take (the cap, a
/// placer outside the registry) takes the recast back and answers with the bar's refusal.
fn place(
    owner: &mut Session,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    now_ms: u64,
    row: i32,
) {
    let cake = world.cakes.table.get(row).cloned();
    let at = pose(owner, world).zip(cake);
    let placed = at.and_then(|((zone, at, heading), cake)| {
        let placer = owner.character_guid;
        let placed = world
            .cakes
            .live
            .place(&cake, placer, zone, at, heading, now_ms);
        placed.ok().map(|placed| (zone, at, placed))
    });
    let Some((zone, at, placed)) = placed else {
        let refused = owner
            .gateway
            .gameplay_mut()
            .map(|game| game.refuse_cake_place(row, now_ms));
        for packet in refused.and_then(Result::ok).unwrap_or_default() {
            send(owner, &packet);
        }
        return;
    };
    for id in world.players.nearby_sessions(zone, at) {
        if let Some(session) = session_of(owner, sessions, world, id) {
            placed.spawn.iter().for_each(|packet| send(session, packet));
        }
    }
}

/// A click on a live cake rolls the clicker's outcome. Every refusal (not a cake, out of range,
/// the window still closed, another instance) is silent, and the clicker's gameplay and the
/// registry change together or not at all: the only step after the registry's is the effect
/// stack's, whose check the cake table already passed at load.
fn click(
    owner: &mut Session,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    now_ms: u64,
    guid: u64,
) {
    let Some((zone, at, _)) = pose(owner, world) else {
        return;
    };
    let clicker = owner.character_guid;
    let Ok(spec) = world.cakes.live.click(guid, clicker, zone, at, now_ms) else {
        return;
    };
    let Some(Ok(out)) = (owner.gateway.gameplay_mut())
        .map(|game| game.apply_consumable_effect(clicker, at, spec, now_ms))
    else {
        return;
    };
    for packet in &out.replies {
        send(owner, packet);
    }
    for delivery in world.ride_fanout(owner.id, &out.nearby) {
        let session = world.session_keys.get(&delivery.recipient);
        if let Some(session) = session.and_then(|key| sessions.get_mut(key)) {
            if let Err(error) = session.queue_shared_message(&delivery.bytes) {
                event(json!({"event": "cake_send_refused", "reason": error.to_string()}));
            }
        }
    }
    // A late joiner's AddPc must show the new model and auras (as for any visible change).
    if out.changed {
        refresh_public_look(world, clicker, &owner.gateway);
        if let Ok(look) = owner.gateway.player_presentation(owner.is_member) {
            let _ = world.players.set_presentation(owner.id, look);
        }
    }
}

/// Once per simulation tick. With no live cake the library returns at once.
pub(super) fn tick(world: &mut SharedWorld, sessions: &mut SessionMap, now_ms: u64) {
    for (zone, at, packets) in world.cakes.live.tick(now_ms) {
        for id in world.players.nearby_sessions(zone, at) {
            let session = world.session_keys.get(&id);
            if let Some(session) = session.and_then(|key| sessions.get_mut(key)) {
                packets.iter().for_each(|packet| send(session, packet));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::test_support::Rig;
    use super::*;
    use osfr_tools::item_bar::use_wire::NEVER_CAST_SENTINEL;
    use osfr_tools::npc_interaction::NpcInteractRequest;
    use osfr_tools::placed_cakes::{self, is_cake};

    const BOSS: i32 = 7974;
    /// The bundled files the rig's content directory needs for a cake: the item rows and the table.
    const FILES: [&str; 2] = ["consumables.json", "placed_cakes.json"];
    /// The boss cake's three outcomes (`placed_cakes.json`): models 2055, 1944 and 1702.
    const BOSSES: [i32; 3] = [2055, 1944, 1702];
    const CLICKER: u64 = 2;

    /// The owner holds the boss cake on bar slot 0, and the world has the table the startup
    /// load builds from the same content directory.
    fn rig(tag: &str) -> Option<Rig> {
        let mut rig = Rig::with_files(tag, BOSS, &FILES)?;
        rig.scene.world.cakes = load(&rig.options).unwrap();
        Some(rig)
    }

    /// The inner packets of a drained batch.
    fn inners(messages: &[Vec<u8>]) -> Vec<&[u8]> {
        messages.iter().filter_map(|m| m.get(7..)).collect()
    }

    fn sub(inner: &[u8]) -> u16 {
        u16::from_le_bytes([inner[2], inner[3]])
    }

    fn guid_at(inner: &[u8], at: usize) -> u64 {
        u64::from_le_bytes(inner[at..at + 8].try_into().unwrap())
    }

    /// The packets about a placed cake, as `(subtype, inner)`: AddNpc 35/2, its cursor entry
    /// 35/12, a composite effect 35/16 or an animation 35/8 naming a cake, and a removal 35/3.
    fn about_cake(messages: &[Vec<u8>]) -> Vec<(u16, Vec<u8>)> {
        let cake = |inner: &&[u8]| {
            if !inner.starts_with(&[35, 0]) || inner.len() < 16 {
                return false;
            }
            let at = match sub(inner) {
                2 | 16 | 8 => 4,
                12 => 8,
                3 => 6,
                _ => return false,
            };
            is_cake(guid_at(inner, at))
        };
        let mut found = inners(messages);
        found.retain(cake);
        found.into_iter().map(|p| (sub(p), p.to_vec())).collect()
    }

    fn subs(packets: &[(u16, Vec<u8>)]) -> Vec<u16> {
        packets.iter().map(|(sub, _)| *sub).collect()
    }

    /// The composite effect id of a 35/16 (client reader `008e5f50`; the writer puts it at 20).
    fn effect_id(inner: &[u8]) -> i32 {
        i32::from_le_bytes(inner[20..24].try_into().unwrap())
    }

    /// The `(model, guid)` of each 35/14 temporary appearance (reader `008d6890`).
    fn models(messages: &[Vec<u8>]) -> Vec<(i32, u64)> {
        let mut found = inners(messages);
        found.retain(|p| p.starts_with(&[35, 0, 14, 0]) && p.len() >= 16);
        let model = |p: &[u8]| i32::from_le_bytes(p[4..8].try_into().unwrap());
        found
            .into_iter()
            .map(|p| (model(p), guid_at(p, 8)))
            .collect()
    }

    fn count_of(messages: &[Vec<u8>], main: u8, sub: u8) -> usize {
        let mut found = inners(messages);
        found.retain(|p| p.starts_with(&[main, 0, sub, 0]));
        found.len()
    }

    fn click(guid: u64) -> Vec<u8> {
        NpcInteractRequest {
            target: guid,
            trailing_flag: false,
        }
        .encode()
    }

    /// The live cake's guid, from the placer's own AddNpc.
    fn cake_guid(placer: &[Vec<u8>]) -> u64 {
        let spawned = about_cake(placer);
        guid_at(&spawned.iter().find(|(sub, _)| *sub == 2).unwrap().1, 4)
    }

    #[test]
    fn t557_a_press_shows_the_cake_to_the_placer_and_everyone_near_and_spends_nothing() {
        let Some(mut rig) = rig("t557-place") else {
            return;
        };
        let near = rig.player(2, CLICKER, 2.0);
        let far = rig.viewer(3, 900.0);
        let (saved, before) = (rig.saved(), rig.bar(2_100));
        rig.drain_owner(2_500);
        rig.drain(near, 2_500);

        let owner = rig.press(3_000);
        let neighbour = rig.drain(near, 3_000);
        for (who, got) in [("placer", &owner), ("neighbour", &neighbour)] {
            let cake = about_cake(got);
            // AddNpc, its cursor entry, then the two spawn effects of the boss cake.
            assert_eq!(subs(&cake), [2, 12, 16, 16], "{who}");
            assert_eq!(effect_id(&cake[2].1), 16372, "{who}");
            assert_eq!(effect_id(&cake[3].1), 16441, "{who}");
        }
        assert!(
            about_cake(&rig.drain(far, 3_000)).is_empty(),
            "out of range"
        );
        assert_eq!(count_of(&owner, 36, 1), 0, "no refusal");
        // The cake is not consumed and the item bar changed only by its recast.
        assert_eq!(rig.count(BOSS), Some(1));
        assert_eq!(rig.saved(), saved);
        assert_eq!(rig.scene.world.cakes.live.len(), 1);
        let after = rig.bar(3_100);
        assert_eq!(after.total_refresh_time, 60_000, "the cake row's recast");
        assert_ne!(after.cast_age_ms, before.cast_age_ms, "and it is running");

        // A press inside the recast is refused and changes nothing.
        let live = rig.scene.world.cakes.live.clone();
        let refused = rig.press(3_500);
        assert_eq!(count_of(&refused, 36, 1), 1);
        assert!(about_cake(&refused).is_empty());
        assert!(about_cake(&rig.drain(near, 3_500)).is_empty());
        assert_eq!(rig.scene.world.cakes.live, live);
        assert_eq!(rig.count(BOSS), Some(1));

        // (R) A session that arrives after the spawn is not shown the cake.
        let (late, _) = rig.viewer_as(4, 3.0, 0);
        rig.tick(3_600);
        assert!(about_cake(&rig.drain(late, 3_600)).is_empty());
    }

    #[test]
    fn t557_a_neighbour_clicks_for_an_outcome_and_a_second_click_inside_five_seconds_does_nothing()
    {
        let Some(mut rig) = rig("t557-click") else {
            return;
        };
        let near = rig.player(2, CLICKER, 2.0);
        let owner = rig.press(3_000);
        let guid = cake_guid(&owner);
        rig.drain_owner(3_000);
        rig.drain(near, 3_000);

        rig.send_as(near, &click(guid), 4_000).unwrap();
        let got = rig.drain(near, 4_000);
        let first = models(&got);
        assert_eq!(first.len(), 1, "{first:?}");
        assert!(
            BOSSES.contains(&first[0].0) && first[0].1 == CLICKER,
            "{first:?}"
        );
        // The poof on the clicker and the buff row, as for any transformation treat.
        let poof = inners(&got).into_iter().find(|p| {
            p.starts_with(&[35, 0, 16, 0]) && guid_at(p, 4) == CLICKER && effect_id(p) == 21
        });
        assert!(
            poof.is_some(),
            "the 35/16 poof (composite 21) on the clicker"
        );
        assert_eq!(count_of(&got, 38, 16), 1, "one buff row");
        // The placer sees the clicker change.
        assert_eq!(models(&rig.drain_owner(4_000)), first);

        // A second click inside the 5 s window: nothing is sent and nothing changes.
        let (effects, live) = (rig.effects(near), rig.scene.world.cakes.live.clone());
        rig.send_as(near, &click(guid), 8_999).unwrap();
        assert!(rig.drain(near, 8_999).is_empty());
        assert!(rig.drain_owner(8_999).is_empty());
        assert_eq!(rig.effects(near), effects);
        assert_eq!(rig.scene.world.cakes.live, live);

        // Once the window is over the roll excludes the last outcome, and a new transformation
        // replaces the live one.
        rig.send_as(near, &click(guid), 9_000).unwrap();
        let second = models(&rig.drain(near, 9_000));
        assert_eq!(second.len(), 1, "{second:?}");
        assert!(
            BOSSES.contains(&second[0].0) && second[0].0 != first[0].0,
            "{second:?}"
        );
    }

    #[test]
    fn t557_a_click_out_of_range_or_on_no_cake_is_silent_and_changes_nothing() {
        let Some(mut rig) = rig("t557-refused") else {
            return;
        };
        let near = rig.player(2, CLICKER, 2.0);
        let away = rig.player(3, 3, 40.0);
        let guid = cake_guid(&rig.press(3_000));
        rig.drain(near, 3_000);
        rig.drain(away, 3_000);
        let live = rig.scene.world.cakes.live.clone();
        let effects = rig.effects(away);

        // 40 m from the cake, a guid nobody placed, and a guid that is not a cake at all.
        for target in [guid, placed_cakes::GUID_TAG | 0x7fff, 1_234] {
            rig.send_as(away, &click(target), 4_000).unwrap();
            assert!(rig.drain(away, 4_000).is_empty(), "{target:x}");
        }
        assert_eq!(rig.scene.world.cakes.live, live);
        assert_eq!(rig.effects(away), effects);
        // The refused click did not use up the window: an in-range click at once works.
        rig.send_as(near, &click(guid), 4_001).unwrap();
        assert_eq!(models(&rig.drain(near, 4_001)).len(), 1);
    }

    #[test]
    fn t557_the_cake_goes_after_sixty_seconds_for_everyone_once_even_if_its_placer_left() {
        let Some(mut rig) = rig("t557-despawn") else {
            return;
        };
        let near = rig.player(2, CLICKER, 2.0);
        let owner = rig.press(3_000);
        let guid = cake_guid(&owner);
        rig.drain_owner(3_000);
        rig.drain(near, 3_000);
        // The one-shot animation may play on the way; the removal is the only 35/3.
        rig.tick(62_900);
        assert_eq!(count_of(&rig.drain(near, 62_900), 35, 3), 0, "not yet");
        rig.tick(63_000);
        for got in [rig.drain_owner(63_000), rig.drain(near, 63_000)] {
            let cake = about_cake(&got);
            let removed: Vec<_> = cake.iter().filter(|(s, _)| *s == 3).collect();
            assert_eq!(removed.len(), 1);
            assert_eq!(guid_at(&removed[0].1, 6), guid);
        }
        rig.tick(63_100);
        assert_eq!(count_of(&rig.drain(near, 63_100), 35, 3), 0, "only once");
        assert!(rig.scene.world.cakes.live.is_empty());

        // The placer logging out does not take the cake away: the neighbour still sees it go.
        rig.drain_owner(63_100);
        let guid = cake_guid(&rig.press(70_000));
        let id = rig.scene.session.id;
        rig.scene.world.leave_player(id);
        rig.drain(near, 70_000);
        rig.tick(130_000);
        let gone = about_cake(&rig.drain(near, 130_000));
        assert!(gone.iter().any(|(s, p)| *s == 3 && guid_at(p, 6) == guid));
    }

    #[test]
    fn t557_a_world_at_its_cap_refuses_the_place_and_the_recast_is_not_started() {
        let Some(mut rig) = rig("t557-full") else {
            return;
        };
        let near = rig.player(2, CLICKER, 2.0);
        let cap = rig.scene.world.cakes.live.clone();
        rig.scene.world.cakes.live = PlacedCakes::new(0);
        rig.drain_owner(2_500);
        rig.drain(near, 2_500);
        let saved = rig.saved();

        let refused = rig.press(3_000);
        assert_eq!(count_of(&refused, 36, 1), 1, "the bar's refusal");
        assert!(about_cake(&refused).is_empty());
        assert!(about_cake(&rig.drain(near, 3_000)).is_empty());
        assert_eq!(rig.saved(), saved);
        assert_eq!(rig.bar(3_050).cast_age_ms, NEVER_CAST_SENTINEL);
        // Canary: with room again the very next press places it.
        rig.scene.world.cakes.live = cap;
        let placed = rig.press(3_100);
        assert_eq!(subs(&about_cake(&placed)), [2, 12, 16, 16]);
    }
}
