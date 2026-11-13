//! Every combat packet writer (combat design section 4). The first part is the writers the
//! engine sends today, moved out of `encounter.rs` by the combat seam without a byte changed;
//! the second part is the frozen interface the other combat pieces call. Piece P1
//! (`combat-wire`) owns the bodies.
use super::*;

#[path = "wire_templates.rs"]
pub mod templates;
pub use self::templates::{DefinitionPatch, HitEffects, Settings, WireData};

pub(super) fn header(op: u16, sub: u16) -> Vec<u8> {
    [op.to_le_bytes(), sub.to_le_bytes()].concat()
}
pub(super) fn encounter_header(sub: u16, id: u32, instance: i32) -> Vec<u8> {
    let mut b = header(41, sub);
    b.extend((id as i32).to_le_bytes());
    b.extend(instance.to_le_bytes());
    b
}
pub(super) fn encounter_state(id: u32, instance: i32, state: i32) -> Vec<u8> {
    let mut b = encounter_header(106, id, instance);
    b.extend(state.to_le_bytes());
    b
}
pub(super) fn combat_flag(sub: u16, id: u32, instance: i32, on: bool) -> Vec<u8> {
    let mut b = encounter_header(sub, id, instance);
    b.push(u8::from(on));
    b
}
pub(super) fn minigame(sub: u8, _instance: i32, id: u32) -> Vec<u8> {
    let mut b = 39_u16.to_le_bytes().to_vec();
    b.push(sub);
    // Native 41/114 creates its MiniGameState using the FIRST encounter-header
    // integer (encounter id). All 39 lifecycle messages must use that same key.
    for v in [id as i32, -1, id as i32] {
        b.extend(v.to_le_bytes());
    }
    b
}
/// A run left before its results still ends on the client: its game over (39/18), then the
/// remove (39/19, exact header only), as a finished run's end does. 39/19 alone left the client's
/// minigame manager open (it logged `LeaveMiniGame` with no `OnGameOver`) and every menu locked
/// (A36, owner 2026-11-13: leaving the hooligans mid-fight locked housing and the rest).
pub(super) fn left_unfinished(instance: i32, id: u32) -> [Vec<u8>; 2] {
    [game_over(false, instance, id), minigame(19, instance, id)]
}
pub(super) fn game_over(won: bool, instance: i32, id: u32) -> Vec<u8> {
    let mut b = minigame(18, instance, id);
    b.push(u8::from(won));
    for v in [0_i32, 0, 1] {
        b.extend(v.to_le_bytes());
    }
    b
}
pub(super) fn objective(sub: u8, id: i32, value: i32) -> Vec<u8> {
    let mut b = 45_u16.to_le_bytes().to_vec();
    b.push(sub);
    b.extend(id.to_le_bytes());
    b.extend(value.to_le_bytes());
    b
}
pub(super) fn objective_complete(id: i32) -> Vec<u8> {
    let mut b = objective(3, id, 0);
    b.extend(OBJECTIVE_AWARD_SCORE.to_le_bytes());
    b
}
pub(super) fn rules(combat: bool) -> Vec<u8> {
    let mut b = 62_u16.to_le_bytes().to_vec();
    b.extend([u8::from(combat); 11]);
    b.extend((-1_i32).to_le_bytes());
    b.extend(1_f32.to_le_bytes());
    for v in [3600_i32, 0, 0, 0, 0, 0] {
        b.extend(v.to_le_bytes());
    }
    b
}
/// Packet 62 with a fight's own values (capture fact C16; the two captured records are
/// `Settings::world` and `Settings::battle`). `rules` above stays the record the engine has
/// sent natively since October.
pub fn encounter_settings(s: &Settings) -> Vec<u8> {
    let mut b = 62_u16.to_le_bytes().to_vec();
    b.extend(s.flags);
    b.extend(s.int_a.to_le_bytes());
    b.extend(s.float_b.to_le_bytes());
    for v in s.ints {
        b.extend(v.to_le_bytes());
    }
    b
}
pub(super) fn player_hp(hp: u32, max: u32) -> Vec<u8> {
    let mut b = header(38, 1);
    b.extend((hp as i32).to_le_bytes());
    b.extend((max as i32).to_le_bytes());
    b
}
pub(super) fn enemy_hp(guid: u64, hp: u32, max: u32) -> Vec<u8> {
    let mut b = header(35, 5);
    b.extend(guid.to_le_bytes());
    for v in [hp as i32, max as i32, 0] {
        b.extend(v.to_le_bytes());
    }
    b
}
/// An enemy's health as the client must hold it: the 35/5 bar over its head, then the 35/35
/// health statement (show 0, delta 0: live sent that shape for every regen tick, C13). Only
/// 35/35 and 32/7 set the actor's current and maximum health (`PlayerHitpointEvent`,
/// `008ba940` writes +0x160 / +0x15c), and those two fields are what the boss bar reads
/// (`BaseClient.Bosses` cell getter `00a79eb0`, columns 2 and 3); the 35/5 handler (dispatch
/// `0092f460` case 5) only moves the nameplate bar. 32/7 alone subtracts its damage from the
/// client's copy (`00a2ba40`), which is zero until a statement sets it: every spawn, heal and
/// regen sends one; a hit sends its 32/7 (`player_hit`).
pub(super) fn enemy_health(guid: u64, hp: u32, max: u32) -> [Vec<u8>; 2] {
    let wire = |v: u32| i32::try_from(v).unwrap_or(i32::MAX);
    [
        enemy_hp(guid, hp, max),
        hitpoints(guid, guid, false, wire(max), wire(hp), 0, false),
    ]
}
pub(super) fn boss_display(guid: u64, on: bool) -> Vec<u8> {
    let mut b = header(32, 9);
    b.extend(guid.to_le_bytes());
    b.push(u8::from(on));
    b
}
pub(super) fn attack_processed(
    attacker: u64,
    target: u64,
    damage: u32,
    hp: u32,
    max: u32,
    critical: bool,
) -> Vec<u8> {
    attack(attacker, target, damage, hp, max, 0, [critical, false])
}
/// 32/7 AttackProcessed, 50 bytes (client reader `0x00a2a910`): the attacker twice, the
/// target, damage, the target's maximum, a composite effect id, two flag bytes, an int the
/// 83 captured rows leave 0, and the target's health.
fn attack(
    attacker: u64,
    target: u64,
    damage: u32,
    hp: u32,
    max: u32,
    effect: i32,
    flags: [bool; 2],
) -> Vec<u8> {
    let mut b = header(32, 7);
    for v in [attacker, attacker, target] {
        b.extend(v.to_le_bytes());
    }
    for v in [damage as i32, max as i32, effect] {
        b.extend(v.to_le_bytes());
    }
    // +0x2c critical-style flag (the client shows "-N!!"), +0x2d suppress reaction.
    b.extend(flags.map(u8::from));
    for v in [0_i32, hp as i32] {
        b.extend(v.to_le_bytes());
    }
    b
}
/// 35/8 SetAnimation (client reader `0x008e5dd0`); also world events' writer.
/// Public because the gateway binary (and its embedded test harnesses) is a
/// separate crate from this library and must not add a fourth copy (T507).
pub fn animation(guid: u64, id: i32, base: bool) -> Vec<u8> {
    let mut b = header(35, 8);
    b.extend(guid.to_le_bytes());
    b.extend(id.to_le_bytes());
    b.extend(0_i32.to_le_bytes());
    b.push(u8::from(base));
    b
}
pub(super) fn movement(guid: u64, p: [f32; 3], heading: [f32; 3]) -> Vec<u8> {
    // Movement is standalone opcode125, not a BasePlayerUpdate subpacket.
    let mut b = 125_u16.to_le_bytes().to_vec();
    b.extend(guid.to_le_bytes());
    for v in p.into_iter().chain(heading) {
        b.extend(v.to_le_bytes());
    }
    b.extend([1, 0]); // Community NPC movement state; position/heading are authoritative.
    b
}
pub(super) fn relevance(guids: &[u64], guide: bool) -> Vec<u8> {
    let mut b = header(35, 12);
    b.extend((guids.len() as i32).to_le_bytes());
    for (i, g) in guids.iter().enumerate() {
        b.extend(g.to_le_bytes());
        b.extend([1, if guide && i == 0 { 5 } else { 1 }, 0]);
    }
    b
}
pub(super) fn remove(guid: u64) -> Vec<u8> {
    crate::abilities::effect_packets::encode_remove_gracefully(guid, 0)
}
pub(super) fn replace_string(b: &mut Vec<u8>, offset: usize, s: &str) {
    b.splice(
        offset..offset + 4,
        (s.len() as u32).to_le_bytes().into_iter().chain(s.bytes()),
    );
}
pub(super) fn menu(guid: u64, name: &str, entries: &[(i32, i32)], auto: bool) -> Vec<u8> {
    let mut b = header(26, 9);
    b.extend(guid.to_le_bytes());
    b.push(u8::from(auto));
    b.extend((entries.len() as i32).to_le_bytes());
    for (id, label) in entries {
        for v in [*id, 0, *label, 0, 0, 0, 0] {
            b.extend(v.to_le_bytes());
        }
    }
    b.extend((name.len() as u32).to_le_bytes());
    b.extend(name.bytes());
    b.extend([1, 0]);
    b
}

/// Pinned Sanctuary PacketClientBeginZoning field order. Fourth components are
/// 1 for a point/identity quaternion; saved XYZ fields are retained on return.
pub fn zone_begin_packet(zone: &ZoneTransition) -> Vec<u8> {
    let mut b = 31_u16.to_le_bytes().to_vec();
    b.extend((zone.name.len() as u32).to_le_bytes());
    b.extend(zone.name.bytes());
    b.extend(2_i32.to_le_bytes());
    for v in zone
        .position
        .into_iter()
        .chain([1_f32])
        .chain(zone.rotation)
        .chain([1_f32])
    {
        b.extend(v.to_le_bytes());
    }
    b.extend((zone.sky.len() as u32).to_le_bytes());
    b.extend(zone.sky.bytes());
    b.extend([u8::from(zone.tutorial), 0]);
    b.extend((zone.zone_id as i32).to_le_bytes());
    b.extend(0_f32.to_le_bytes());
    b.extend(zone.geometry_id.to_le_bytes());
    b.extend([1, 0]);
    b
}

/// AddNpc `interact_range` (+133) for an engine actor: the guide 12 m, an enemy 0, the rest
/// (the exit door, captives, talkers, pickups) 3 m. The client's press-X prompt picks the nearest
/// interactive actor whose range covers the player (`sub_BAF6E0` 00baf6e0 reads it at actor
/// +0x1818); the Kaboomers' 3 m drew it over them (owner, rc-0137), and so did every dungeon
/// enemy whose hit comes from its rank, not a `damage` of its own (owner 2026-11-13: "add an
/// identifier for enemies spawned in dungeons and inside instances"). `enemy` is that marker:
/// the caller knows what it spawns. (R) Live sent 100 to its hostiles (p12: all 52 Bixie
/// Soldier, Battlemage and wave-wolf AddNpc rows, with 35/12 cursor 11), so 0 is the owner's
/// rule, not the capture's. The enemy stays interactable (+224) with its 35/12 cursor, so a
/// click still targets it.
fn reach(disposition: i32, enemy: bool) -> i32 {
    match disposition {
        2 => 12,
        0 if enemy => 0,
        _ => 3,
    }
}

pub(super) fn spawn_actor(
    template: &[u8],
    a: &Actor,
    position: [f32; 3],
    disposition: i32,
    enemy: bool,
) -> Vec<u8> {
    let mut b = template.to_vec();
    b[4..12].copy_from_slice(&a.guid.to_le_bytes());
    b[12..16].copy_from_slice(&a.name_id.to_le_bytes());
    b[16..20].copy_from_slice(&a.model_id.to_le_bytes());
    if let Some(scale) = a.scale {
        // AddNpc: unknown flag (20), bubble colours and size (21..33), scale.
        b[33..37].copy_from_slice(&scale.to_le_bytes());
    }
    for (i, v) in position.into_iter().enumerate() {
        b[37 + i * 4..41 + i * 4].copy_from_slice(&v.to_le_bytes());
    }
    b[69..73].copy_from_slice(&0_i32.to_le_bytes());
    b[77..81].copy_from_slice(&disposition.to_le_bytes());
    b[128..132].copy_from_slice(&3.0_f32.to_le_bytes());
    b[133..137].copy_from_slice(&reach(disposition, enemy).to_le_bytes());
    b[137..141].copy_from_slice(&2_i32.to_le_bytes());
    b[141..145].copy_from_slice(&3_i32.to_le_bytes());
    b[145..149].copy_from_slice(&1_i32.to_le_bytes());
    b[233..237].copy_from_slice(&2_i32.to_le_bytes());
    replace_string(&mut b, 106, &a.name);
    replace_string(&mut b, 81, &a.texture_alias);
    b
}

/// 41/125 EncounterShowRespawnWindowPacket: dispatcher `0090d830`, ctor
/// `008a33b0`, reader `008d6730`. Family header (ids zero: the overworld has
/// no instance) then two i32 handed to `DisplayRespawnWindow` `00904300`,
/// which takes (timer ms, revive cost). The field order is inferred from
/// that handler, not captured.
pub fn respawn_window(timer_ms: i32, cost: u32) -> Vec<u8> {
    let mut b = encounter_header(125, 0, 0);
    b.extend(timer_ms.to_le_bytes());
    // Bounded by MAX_REVIVE_COST at load, so it fits the wire i32.
    b.extend((cost as i32).to_le_bytes());
    b
}

/// 41/126: the dispatcher runs `RespawnWindow:Hide` (family header only).
pub(super) fn respawn_hide() -> Vec<u8> {
    encounter_header(126, 0, 0)
}

/// The engine field `wire_state`: templates and whatever else `load_wire` reads. P1 fills it;
/// no other piece looks inside.
#[derive(Clone, Debug, Default)]
pub struct WireState {
    /// `combat_wire.json`, or the built-in scalars with no templates.
    pub data: Arc<WireData>,
}

impl EncounterEngine {
    /// Engine load: read this piece's own content from `directory` (absent files leave the
    /// feature dormant).
    pub(super) fn load_wire(&mut self, directory: &Path) -> io::Result<()> {
        self.wire_state.data = WireData::load(directory)?;
        Ok(())
    }

    /// Every packet of one player hit; the engine's one call for it (`strike_damage`). The hit
    /// carries the cast in progress, and this is where `rules.player_hit_wire` and the
    /// templates in `wire_state` choose the shape.
    pub(super) fn player_hit_packets(&self, mut hit: PlayerHit) -> Vec<Vec<u8>> {
        if let Some(cast) = &self.kit_state.cast {
            hit.ability = Some(cast.ability);
            hit.effect = cast.effect;
        }
        if self.rules.player_hit_wire {
            self.wire_state.data.player_hit_live(&hit)
        } else {
            player_hit(&hit)
        }
    }

    /// The loaded wire data: templates, the death effect, the hit effects, packet 62 records.
    pub fn wire_data(&self) -> &WireData {
        &self.wire_state.data
    }
}

/// One player hit on an enemy, as the engine resolved it. Build it with
/// `..Default::default()`: fields may be added.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlayerHit {
    pub attacker: u64,
    pub target: u64,
    pub dealt: u32,
    /// The target's health after the hit, and its maximum.
    pub health: u32,
    pub max_health: u32,
    pub critical: bool,
    /// A basic attack: the attacker plays its swing.
    pub swing: bool,
    /// The ability that made the hit, and the effect it plays on the target; `None` for the
    /// plain attack of before the kits.
    pub ability: Option<i32>,
    pub effect: Option<i32>,
}

/// Every packet of one player hit, in order. Today: the swing (35/8 animation 1099), 32/7
/// carrying the kit's impact effect when the hit names one, and the enemy's health (35/5 and
/// the 35/35 statement the boss bar reads). The live shape is `WireData::player_hit_live`,
/// chosen in `player_hit_packets`.
pub fn player_hit(h: &PlayerHit) -> Vec<Vec<u8>> {
    let mut packets = Vec::with_capacity(4);
    if h.swing {
        packets.push(animation(h.attacker, 1099, false));
    }
    packets.push(attack(
        h.attacker,
        h.target,
        h.dealt,
        h.health,
        h.max_health,
        h.effect.unwrap_or(0),
        [h.critical, false],
    ));
    // The 32/7 itself moves the boss bar, in step with the swing (`00a2ba40` takes its damage
    // off the copy the spawn's statement set). A 35/35 here as well came on the enemy's own
    // animation clock (`008d3020` delays the event by its attacker's actor, `008ba940` keeps
    // only a newer one) and made the bar tick sporadically (owner, rc-0136 recheck).
    packets.push(enemy_hp(h.target, h.health, h.max_health));
    packets
}

// The frozen writer interface (combat design section 4). Piece P1 (`combat-wire`) owns every
// body in this file and proves each against its capture row; the signatures do not change.
// Until P1 lands, a writer that had a body before the seam still writes those bytes, and a
// packet the server never sent before is written from the layout the design table gives,
// unproven. None of the new ones is sent by the engine yet.

/// One enemy hit on a player (32/7).
#[derive(Clone, Debug, PartialEq)]
pub struct EnemyHit {
    pub attacker: u64,
    pub target: u64,
    /// The raw hit, before the target's Defense.
    pub damage: u32,
    pub target_max: u32,
    pub target_current: u32,
    /// The impact effect (5409; 5622 special; 5631 knockout).
    pub effect: i32,
    /// A 1.5x hit: the first flag byte.
    pub special: bool,
    /// The hit that knocked the player out: the second flag byte.
    pub knockout: bool,
}

/// A buff or status icon on the group frame (38/16).
#[derive(Clone, Debug, PartialEq)]
pub struct Tag {
    pub id: i32,
    pub duration_ms: u64,
    pub icon_id: i32,
    pub name_id: i32,
}

/// How a spoken line shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum How {
    #[default]
    Bubble,
    Red,
    System,
}

/// One line an actor says: a client string id, or literal text when there is none.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Line {
    pub text_id: Option<i32>,
    pub text: Option<String>,
    pub how: How,
    /// L1189: several client lines, one said per firing (a pack of hooligans each taunts with
    /// its own line instead of chanting one).
    pub text_ids: Vec<i32>,
}

impl Line {
    /// The line `guid` says at `now_ms`: one of `text_ids`, picked by speaker and moment, or
    /// the line as written when it lists none.
    pub fn pick(&self, guid: u64, now_ms: u64) -> Line {
        let mut line = self.clone();
        if !self.text_ids.is_empty() {
            let pick = crate::hash::mix64(guid ^ now_ms.rotate_left(29)) % self.text_ids.len() as u64;
            line.text_id = Some(self.text_ids[pick as usize]);
        }
        line
    }
}

/// One 38/7 stat value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StatValue {
    Int(i32),
    Float(f32),
}

/// Status bits of 35/20, counted from bit 0 as the client's handler tests them (dispatcher
/// `0x0092f460` case 0x14). Live sent 0x1 on every actor at rest, 0x8001 while it fought,
/// 0x4001 while it ran home, 0x101, and 0x8080 then 0x81 on the player it knocked out.
pub mod bits {
    pub const ALIVE: u32 = 1 << 0;
    pub const AFRAID: u32 = 1 << 1;
    pub const ASLEEP: u32 = 1 << 2;
    pub const SILENCED: u32 = 1 << 3;
    pub const BOUND: u32 = 1 << 4;
    pub const ROOTED: u32 = 1 << 5;
    pub const STUNNED: u32 = 1 << 6;
    pub const KNOCKED_OUT: u32 = 1 << 7;
    pub const NONATTACKABLE: u32 = 1 << 8;
    pub const KNOCKED_BACK: u32 = 1 << 9;
    pub const CONFUSED: u32 = 1 << 13;
    pub const GOING_HOME: u32 = 1 << 14;
    /// Unnamed in the client; live set it from the moment an actor went for a player.
    pub const FIGHTING: u32 = 1 << 15;
    pub const FROZEN: u32 = 1 << 16;
    pub const BERSERK: u32 = 1 << 17;
    pub const POPPED_UP: u32 = 1 << 20;
    pub const IDLE: u32 = ALIVE;
    pub const IN_FIGHT: u32 = ALIVE | FIGHTING;
    pub const RETURNING: u32 = ALIVE | GOING_HOME;
}

/// 35/35 HitPointModification, 34 bytes (client reader `0x008d6c50`, handler `0x008d3020`;
/// capture p12 t=408.1): source, target, show-number, the target's maximum and current
/// health, the signed delta and the crit-style byte. The client acts only when `target` is
/// an actor it knows: it prints the delta when `show` is set and either id is the local
/// player, and always sets the target's bar to current * 100 / max, so `max` must be real.
/// Live did not clamp `current` at zero.
pub fn hitpoints(
    source: u64,
    target: u64,
    show: bool,
    max: i32,
    current: i32,
    delta: i32,
    crit: bool,
) -> Vec<u8> {
    let mut b = header(35, 35);
    b.extend(source.to_le_bytes());
    b.extend(target.to_le_bytes());
    b.push(u8::from(show));
    for v in [max, current, delta] {
        b.extend(v.to_le_bytes());
    }
    b.push(u8::from(crit));
    b
}

/// 35/20 UpdateCharacterState, 16 bytes (reader `0x008d7650`): an actor's state and status
/// bits, see `bits`.
pub fn state(guid: u64, bits: u32) -> Vec<u8> {
    let mut b = header(35, 20);
    b.extend(guid.to_le_bytes());
    b.extend(bits.to_le_bytes());
    b
}

/// 35/23 ExpectedSpeed, 16 bytes (reader `0x008d6b40`; the handler stores the float on any
/// actor but the local player).
pub fn speed(guid: u64, metres_per_s: f32) -> Vec<u8> {
    let mut b = header(35, 23);
    b.extend(guid.to_le_bytes());
    b.extend(metres_per_s.to_le_bytes());
    b
}

/// 35/3 in its graceful form, 31 bytes (reader `0x008d6a60`, capture p12 t=408.8): the
/// variant word 1, the guid, animate, the delay, an effect delay, the effect and its
/// duration. Built-in values; an engine with content calls `WireData::death`.
pub fn death(guid: u64, delay_ms: u32) -> Vec<u8> {
    death_with(&templates::Death::default(), guid, delay_ms)
}

pub(super) fn death_with(d: &templates::Death, guid: u64, delay_ms: u32) -> Vec<u8> {
    let mut b = header(35, 3);
    b.extend(1_u16.to_le_bytes());
    b.extend(guid.to_le_bytes());
    b.push(u8::from(d.animate));
    for v in [delay_ms as i32, 0, d.effect_id, d.duration_ms] {
        b.extend(v.to_le_bytes());
    }
    b
}

/// 32/7 for an enemy's hit on a player, as live sent all 83 (capture p12 t=384.8): the raw
/// damage, the impact effect (`HitEffects::pick`), the first flag byte on a 1.5x hit and
/// the second on the hit that knocks the player out.
pub fn enemy_hit(h: &EnemyHit) -> Vec<u8> {
    attack(
        h.attacker,
        h.target,
        h.damage,
        h.target_current,
        h.target_max,
        h.effect,
        [h.special, h.knockout],
    )
}

/// 32/6, 20 bytes (reader `0x00a2a820`): a hit that did no damage. Live sent it three times.
pub fn dodge(attacker: u64, target: u64) -> Vec<u8> {
    let mut b = header(32, 6);
    b.extend(attacker.to_le_bytes());
    b.extend(target.to_le_bytes());
    b
}

/// 36/3 StartCasting, 37 bytes (reader `0x00a2fc70`), as all 281 live rows have it: no
/// effect, no animation, the ability id, cast time 0 and no bar. On a player's own cast live
/// put the caster in the second id as well; on an enemy's cast, the player it was aimed at.
pub fn cast(caster: u64, target: u64, ability: i32) -> Vec<u8> {
    let mut b = header(36, 3);
    b.extend(caster.to_le_bytes());
    b.extend(target.to_le_bytes());
    for v in [0, 0, ability] {
        b.extend(v.to_le_bytes());
    }
    b.extend(0_f32.to_le_bytes());
    b.push(0);
    b
}

/// 36/4 LaunchAndLand needs a captured template, which is content: an engine calls
/// `WireData::launch`. This free function has no content to read and answers `None`.
pub fn launch(_template: &str, _caster: u64, _target: u64) -> Option<Vec<u8>> {
    None
}

/// 36/14, 24 bytes (reader `0x00a2fe20`): a hit reaction, animation first. Live sent 1120
/// (`com_recoil`, with 3.0 or 0.0) and 1402 (`com_knock_down`), with some abilities only.
pub fn reaction(guid: u64, animation: i32, seconds: f32) -> Vec<u8> {
    let mut b = header(36, 14);
    b.extend(guid.to_le_bytes());
    b.extend(animation.to_le_bytes());
    b.extend(0_i32.to_le_bytes());
    b.extend(seconds.to_le_bytes());
    b
}

/// 38/16: show a buff or status icon.
pub fn tag_add(t: &Tag) -> Vec<u8> {
    crate::abilities::build_effect_tag(t.id, t.duration_ms, t.icon_id, t.name_id)
}

/// 38/17, 8 bytes (reader `0x008e4010`): remove the icon.
pub fn tag_remove(id: i32) -> Vec<u8> {
    let mut b = header(38, 17);
    b.extend(id.to_le_bytes());
    b
}

/// 35/41 AddEffectTagCompositeEffect, 36 bytes (reader `0x008d7ad0`; the parse is strict, a
/// shorter body is dropped): the actor, the tag, the effect, the source and eight bytes all
/// 53 live rows leave zero. Live: tag 2, effect 5 on an enemy running home.
pub fn attach(guid: u64, tag: i32, effect: i32, source: u64) -> Vec<u8> {
    let mut b = header(35, 41);
    b.extend(guid.to_le_bytes());
    b.extend(tag.to_le_bytes());
    b.extend(effect.to_le_bytes());
    b.extend(source.to_le_bytes());
    b.extend([0; 8]);
    b
}

/// 35/42, 16 bytes (reader `0x008d7bc0`): end that visual.
pub fn detach(guid: u64, tag: i32) -> Vec<u8> {
    let mut b = header(35, 42);
    b.extend(guid.to_le_bytes());
    b.extend(tag.to_le_bytes());
    b
}

/// A line from an actor: 15/4 by string id, else the text as a system line. Live sent an
/// enemy's line as 15/4 from its guid with all three flag bytes clear and the listener in
/// the first trailing id (capture p12 t=264.3); `chat::string_say` sets the chat-log byte
/// and no listener, the shape our server has already shown natively.
pub fn say(guid: u64, line: &Line) -> Vec<u8> {
    match (line.text_id, &line.text) {
        (Some(id), _) => crate::chat::string_say(guid, id),
        (None, text) => crate::chat::system(text.as_deref().unwrap_or_default()),
    }
}

/// 35/28 SetDisposition, 16 bytes (reader `0x008e6070`). Live: 1 on the battle's Exit door.
pub fn disposition(guid: u64, value: i32) -> Vec<u8> {
    let mut b = header(35, 28);
    b.extend(guid.to_le_bytes());
    b.extend(value.to_le_bytes());
    b
}

/// 38/7 UpdateStat (reader `0x0090a540`, rows `0x008e7640`): the actor, a count, then rows
/// of id, kind and value; kind 0 is an int and kind 1 a float.
pub fn stats(guid: u64, rows: &[(i32, StatValue)]) -> Vec<u8> {
    let mut b = header(38, 7);
    b.extend(guid.to_le_bytes());
    b.extend((rows.len() as i32).to_le_bytes());
    for (id, value) in rows {
        b.extend(id.to_le_bytes());
        match value {
            StatValue::Int(v) => {
                b.extend(0_i32.to_le_bytes());
                b.extend(v.to_le_bytes());
            }
            StatValue::Float(v) => {
                b.extend(1_i32.to_le_bytes());
                b.extend(v.to_le_bytes());
            }
        }
    }
    b
}

#[cfg(test)]
mod line_tests {
    use super::*;

    /// L1189: a rule that lists several lines says one of them, chosen per speaker and moment,
    /// so a pack does not chant one taunt; a plain line is said as written.
    #[test]
    fn a_line_with_several_ids_says_one_of_them_per_speaker() {
        let line = Line {
            text_ids: vec![37156, 37157, 37158, 37163],
            ..Line::default()
        };
        let mut said = std::collections::BTreeSet::new();
        for guid in 0..64 {
            let picked = line.pick(guid, 1_000).text_id.expect("a picked id");
            assert!(line.text_ids.contains(&picked));
            said.insert(picked);
        }
        assert!(said.len() > 1, "different speakers say different lines");
        assert_eq!(line.pick(7, 1_000), line.pick(7, 1_000), "the pick is stable");
        let plain = Line {
            text_id: Some(8130),
            ..Line::default()
        };
        assert_eq!(plain.pick(7, 1_000), plain);
    }
}
