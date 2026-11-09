//! The HUD and actor packets of an in-world challenge.
//!
//! Layouts follow the Sanctuary community server fork (BubbaJoeX), `Sanctuary.Packet`
//! (`BaseUiPacket/*`, `BaseObjectivePacket/*`, `BaseMiniGamePacket.cs`; AGPL-3.0, credited in
//! NOTICE). The client side is partly recovered: 47/10 is `FlashTimerSetTimer` `00a8eff0` (seconds),
//! the 47/14 pin is `quest_journal::Pin`, and the 39/21 handler is `009b8880` (id, new star count,
//! flag). No capture shows any of these in a retail type-1 run, so every body below is (R).

use crate::{minigame::family39_header, startup_world, world};

const FAMILY_OBJECTIVE: u16 = 45;
const FAMILY_UI: u16 = 47;

fn ui(sub: u8, ints: &[i32]) -> Vec<u8> {
    let mut bytes = FAMILY_UI.to_le_bytes().to_vec();
    bytes.push(sub);
    for value in ints {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

fn objective(sub: u8, ints: &[i32]) -> Vec<u8> {
    let mut bytes = FAMILY_OBJECTIVE.to_le_bytes().to_vec();
    bytes.push(sub);
    for value in ints {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

/// 47/2 count, 47/3 complete, 47/4 fail.
pub(super) fn task_update(task: i32, count: i32) -> Vec<u8> {
    ui(2, &[task, count])
}

pub(super) fn task_complete(task: i32) -> Vec<u8> {
    ui(3, &[task])
}

pub(super) fn task_fail(task: i32) -> Vec<u8> {
    ui(4, &[task])
}

/// 47/10: the HUD countdown. The client sends nothing when it runs out, so the server keeps time.
pub(super) fn start_timer(seconds: i32) -> Vec<u8> {
    ui(10, &[seconds])
}

pub(super) fn stop_timer() -> Vec<u8> {
    ui(11, &[])
}

pub(super) fn objective_update(id: i32, count: i32) -> Vec<u8> {
    objective(2, &[id, count])
}

pub(super) fn objective_complete(id: i32, points: i32) -> Vec<u8> {
    objective(3, &[id, 0, points])
}

pub(super) fn objective_fail(id: i32) -> Vec<u8> {
    objective(4, &[id])
}

/// 39/21: the star counter's new total. The header's middle dword carries the total (it is -1 in
/// every other minigame packet); the last byte is "no animation".
pub(crate) fn stars_earned(instance_id: i32, game_id: i32, stars: i32) -> Vec<u8> {
    let mut bytes = family39_header(21, instance_id, game_id);
    bytes[7..11].copy_from_slice(&stars.to_le_bytes());
    bytes.push(0);
    bytes
}

/// One 69-byte reward bundle body (the client's `RewardBundleBase`) with `coins` and `stars`.
pub(crate) fn bundle(success: bool, coins: u32, stars: u32) -> Vec<u8> {
    let mut bytes = vec![u8::from(success)];
    for value in [coins, stars, 0, 0, 0, 0] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(1.0f32.to_le_bytes());
    // Two unknown dwords, the source and player guids, icon, name, entry count, description.
    bytes.extend([0; 8 + 16 + 16]);
    bytes
}

/// The AddNpc (35/2) of one challenge thing: a clickable, nameless prop for its player alone.
pub(super) fn spawn(
    guid: u64,
    model_id: i32,
    label_id: i32,
    position: [f32; 3],
    yaw: f32,
    range: f32,
) -> Option<Vec<u8>> {
    let npc = startup_world::NpcSpawn {
        actor_id: guid,
        model_id,
        position: [position[0], position[1], position[2], 1.0],
        rotation: [yaw.sin(), 0.0, yaw.cos(), 0.0],
        name_string_id: label_id,
        hide_nameplate: u8::from(label_id == 0),
        is_interactable: 1,
        interact_range: range.round() as i32,
        unknown_float_51: range,
        ..world::content_spawn_template()
    };
    world::unframe_tunnel(&startup_world::build_add_npc(&npc)).ok()
}

/// Offsets in the 35/3 RemovePlayerGracefully body: bool animate, i32 delay, i32 effect delay,
/// i32 composite effect, i32 duration (the Sanctuary fork's `PlayerUpdatePacketRemovePlayerGracefully`).
const REMOVE_DELAY_AT: usize = 15;
const REMOVE_EFFECT_AT: usize = 23;
/// `PFX_sparkles-swirl_gold_treasure-reward`: the fork's pickup effect.
const SPARKLE_EFFECT: i32 = 5386;

/// 35/3: take a thing away after `delay_ms`.
pub(super) fn remove_after(guid: u64, delay_ms: i32) -> Vec<u8> {
    let mut packet = crate::pets::pet_removal_packet(guid);
    packet[REMOVE_DELAY_AT..REMOVE_DELAY_AT + 4].copy_from_slice(&delay_ms.to_le_bytes());
    packet
}

/// 35/3: a thing picked up goes with a sparkle.
pub(super) fn remove_with_sparkle(guid: u64) -> Vec<u8> {
    let mut packet = crate::pets::pet_removal_packet(guid);
    packet[REMOVE_EFFECT_AT..REMOVE_EFFECT_AT + 4].copy_from_slice(&SPARKLE_EFFECT.to_le_bytes());
    packet
}

/// A composite effect on a thing (a crate of fireworks going off).
pub(super) fn play_effect(guid: u64, effect_id: i32, position: [f32; 3]) -> Vec<u8> {
    crate::abilities::effect_packets::encode_play_composite_effect(guid, effect_id, position)
}
