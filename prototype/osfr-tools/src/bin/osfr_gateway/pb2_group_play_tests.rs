//! PB2 group play through the shared gateway with three sessions and no sockets: the player
//! wheel's Invite to Group, the member job on the group rows, and who a shared credit reaches.
use super::super::group_arena::group_play::eligible;
use super::super::t020_group_routing::{
    GUIDS, enter_member, inner, invite, invited_id, response, route_server, server_rig,
};
use super::*;
use osfr_tools::{
    group_play::GroupPlay,
    player_radial::{GROUP_ENTRY_ID, PublicLook},
    world::InstanceKind,
};

fn select(own: u64, target: u64) -> Vec<u8> {
    [
        &[26, 0, 19, 0][..],
        &own.to_le_bytes(),
        &target.to_le_bytes(),
    ]
    .concat()
}

fn choice(guid: u64, id: i32) -> Vec<u8> {
    [&[26, 0, 10, 0][..], &guid.to_le_bytes(), &id.to_le_bytes()].concat()
}

fn i32_at(bytes: &[u8], at: usize) -> i32 {
    i32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

/// Party A (leader) with B, through the family-40 path the client uses.
fn party(server: &mut Server, keys: &[SessionKey; 3], member: usize) {
    let sent = route_server(server, keys[0], &invite(0, member, 0));
    let id = invited_id(&sent, member);
    route_server(server, keys[member], &response(0, member, id, true));
}

fn queued(server: &mut Server, key: SessionKey) -> Vec<Vec<u8>> {
    let session = server.sessions.get_mut(&key).unwrap();
    super::f120_fishing_hour_reconnect::drain_replies(session, 5_000)
        .into_iter()
        .filter(|message| message.starts_with(&[5, 0, 1]))
        .map(|message| message[7..].to_vec())
        .collect()
}

#[test]
fn pb2_invite_to_group_from_the_player_wheel_forms_a_party() {
    if !super::test_support::bin_test_target("pb2") {
        return;
    }
    let (mut server, keys, scratch) = server_rig("pb2-wheel");
    let [a, b, _] = GUIDS;
    let menu = route_server(&mut server, keys[0], &select(a, b));
    assert_eq!(menu.len(), 1, "one wheel for the clicker");
    let wheel = inner(&menu[0]);
    assert_eq!(&wheel[..4], &[26, 0, 9, 0]);
    assert_eq!(
        i32_at(wheel, 13),
        1,
        "Invite to Group is the only entry that works here"
    );
    // The row: id, icon 136 (`icon_UI_context_group_add01_64.dds`), label 3372.
    assert_eq!(
        [i32_at(wheel, 17), i32_at(wheel, 21), i32_at(wheel, 25)],
        [GROUP_ENTRY_ID, 136, 3372]
    );

    let sent = route_server(&mut server, keys[0], &choice(b, GROUP_ENTRY_ID));
    let invite_id = invited_id(&sent, 1);
    assert!(
        sent.iter()
            .any(|d| d.recipient == server.sessions[&keys[0]].id
                && inner(d)[..4] == [40, 0, 2, 0]
                && i32_at(inner(d), 4) == 0),
        "the leader is told the invite went out"
    );
    route_server(&mut server, keys[1], &response(0, 1, invite_id, true));
    let group = server.world.social.group_of(a);
    assert!(
        group.is_some() && group == server.world.social.group_of(b),
        "one party"
    );

    // B is grouped now: the wheel offers nothing, and a replayed choice is refused silently.
    assert!(route_server(&mut server, keys[0], &select(a, b)).is_empty());
    assert!(route_server(&mut server, keys[0], &choice(b, GROUP_ENTRY_ID)).is_empty());
    drop(server);
    let _ = fs::remove_dir_all(scratch);
}

#[test]
fn pb2_a_members_job_shows_on_the_group_rows() {
    if !super::test_support::bin_test_target("pb2") {
        return;
    }
    let (mut server, keys, scratch) = server_rig("pb2-jobs");
    party(&mut server, &keys, 1);
    queued(&mut server, keys[0]);
    let b = GUIDS[1];
    let look = PublicLook {
        profiles: vec![(11, 7)],
        active_profile: 11,
        items: Vec::new(),
    };
    server.world.public_looks.insert(b, look);
    group_arena::tick(&mut server, 5_000);
    let rows: Vec<_> = (queued(&mut server, keys[0]).into_iter())
        .filter(|m| m.starts_with(&[40, 0, 9, 0]))
        .collect();
    assert_eq!(rows.len(), 1, "the leader gets B's new row once");
    // Header, two i16, guid, three name ints, "P302" and "", then +0x90..+0xa8.
    let at = 4 + 4 + 8 + 12 + 4 + 4 + 4;
    assert_eq!(&rows[0][8..16], &b.to_le_bytes());
    let fields: Vec<i32> = (0..7).map(|n| i32_at(&rows[0], at + n * 4)).collect();
    assert_eq!(
        fields,
        [0, 11, 0, 1340, 13, 7, 0],
        "Medic: id, icon, badge, rank"
    );
    // Nothing changed: the next tick sends nothing.
    group_arena::tick(&mut server, 5_050);
    assert!(
        !queued(&mut server, keys[0])
            .iter()
            .any(|m| m.starts_with(&[40, 0, 9, 0]))
    );
    drop(server);
    let _ = fs::remove_dir_all(scratch);
}

#[test]
fn pb2_a_credit_reaches_members_in_reach_or_in_the_same_battle_only() {
    if !super::test_support::bin_test_target("pb2") {
        return;
    }
    let (mut server, keys, scratch) = server_rig("pb2-reach");
    party(&mut server, &keys, 1);
    party(&mut server, &keys, 2);
    let [a, b, c] = GUIDS;
    let rules = GroupPlay::shipped();
    let content = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-opening-cooking-original-assets");
    let place = |server: &mut Server, index: usize, x: f32| {
        let mut character = server.sessions[&keys[index]].gateway.state().clone();
        character.position = [x, 0.0, 0.0];
        let state = Gameplay::load(&content, None).unwrap().state();
        let reference = &server.options.reference;
        let gateway = Gateway::load_gameplay(reference, character, &content, Some(state)).unwrap();
        server.sessions.get_mut(&keys[index]).unwrap().gateway = Box::new(gateway);
    };
    // Open world: B stands 30 m away, C 90 m away.
    for (guid, index, x) in [(a, 0, 0.0), (b, 1, 30.0), (c, 2, 90.0)] {
        enter_member(&mut server.membership, guid, None);
        place(&mut server, index, x);
    }
    let from = server.sessions[&keys[0]].gateway.state().clone();
    assert_eq!(
        eligible(&server, &rules, a, &from),
        [keys[1]],
        "only B is in reach"
    );

    // A battle instance: the whole instance counts, whoever stands where; C outside gets none.
    let run = server
        .membership
        .create_child_instance(0, 0, InstanceKind::Encounter { encounter_id: 96 }, 0)
        .unwrap();
    enter_member(&mut server.membership, a, Some(run));
    enter_member(&mut server.membership, b, Some(run));
    place(&mut server, 1, 500.0);
    assert_eq!(
        eligible(&server, &rules, a, &from),
        [keys[1]],
        "B is in the run"
    );
    enter_member(&mut server.membership, c, Some(run));
    let reached = eligible(&server, &rules, a, &from);
    assert_eq!(
        reached.len(),
        2,
        "C joined the run: both members, each once"
    );

    // A player who is not in the party is never reached.
    server.world.social.leave_group(c).unwrap();
    assert_eq!(eligible(&server, &rules, a, &from), [keys[1]]);
    drop(server);
    let _ = fs::remove_dir_all(scratch);
}

/// The social window's Invite to Group: Lua `Ui.InviteToGroup(name)` -> `SendGroupInvite`
/// (`00c24d20`) fills only the invitee's name, so the inviter pair arrives empty.
#[test]
fn pb2_the_social_windows_invite_by_name_reaches_the_target() {
    if !super::test_support::bin_test_target("pb2") {
        return;
    }
    let (mut server, keys, scratch) = server_rig("pb2-social");
    let name_data = |first: &str| {
        let mut out = vec![0; 12];
        out.extend((first.len() as i32).to_le_bytes());
        out.extend(first.as_bytes());
        out.extend(0_i32.to_le_bytes());
        out
    };
    let request = [
        &[40, 0, 1, 0][..],
        &0_u64.to_le_bytes(),
        &name_data(""),
        &0_u64.to_le_bytes(),
        &name_data("p302"),
        &[0; 24],
    ]
    .concat();
    let sent = route_server(&mut server, keys[0], &request);
    let invite_id = invited_id(&sent, 1);
    assert_ne!(invite_id, 0, "B is invited by the typed name, any case");
    route_server(&mut server, keys[1], &response(0, 1, invite_id, true));
    assert_eq!(
        server.world.social.group_of(GUIDS[0]),
        server.world.social.group_of(GUIDS[1])
    );
    drop(server);
    let _ = fs::remove_dir_all(scratch);
}
