//! Combat wire data: the values and captured packets that are bytes, not rules (combat design
//! section 4). They come from `combat_wire.json` in the content directory; a directory
//! without the file gets the built-in defaults, which hold the captured scalar values and no
//! templates. A file that is present but bad refuses the load. Piece P1 (`combat-wire`) owns
//! this file and the data file.
//!
//! A template is one packet the March 2014 server sent, kept whole, with the offsets of what
//! must change when we send it for our own actors. 36/4 LaunchAndLand (client reader
//! `0x00a31f30`) is replayed with its guids patched: across 21 captured rows of one ability
//! only the target guid and one counter byte differ. 36/13 AbilityDefinition (reader
//! `0x00a32930`, effect rows `0x00a30360`) is replayed with its ids and tuned values patched.
use super::*;

const FILE: &str = "combat_wire.json";
const MAX_FILE_BYTES: u64 = 1024 * 1024;

fn hex<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
    let text = String::deserialize(d)?;
    let digit = |c: u8| (c as char).to_digit(16).map(|v| v as u8);
    if text.len() % 2 != 0 {
        return Err(serde::de::Error::custom("odd hex length"));
    }
    text.as_bytes()
        .chunks(2)
        .map(|pair| Some(digit(pair[0])? << 4 | digit(pair[1])?))
        .collect::<Option<Vec<u8>>>()
        .ok_or_else(|| serde::de::Error::custom("not hex"))
}

/// The graceful remove (35/3) of a defeated actor, beyond its delay. Captured: animate 1,
/// effect 5017, duration 1000 ms, in the open world and in a battle alike.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Death {
    pub animate: bool,
    pub effect_id: i32,
    pub duration_ms: i32,
}

impl Default for Death {
    fn default() -> Self {
        Self {
            animate: true,
            effect_id: 5017,
            duration_ms: 1000,
        }
    }
}

/// The impact effect of an enemy's hit on a player (32/7): 39 + 41 normal rows carry 5409,
/// the three 1.5x hits 5622 and the one hit that knocked the player out 5631.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HitEffects {
    pub normal: i32,
    pub special: i32,
    pub knockout: i32,
}

impl Default for HitEffects {
    fn default() -> Self {
        Self {
            normal: 5409,
            special: 5622,
            knockout: 5631,
        }
    }
}

impl HitEffects {
    /// The effect id for one hit; the knockout wins over the special.
    pub fn pick(&self, special: bool, knockout: bool) -> i32 {
        if knockout {
            self.knockout
        } else if special {
            self.special
        } else {
            self.normal
        }
    }
}

/// Packet 62, the encounter settings record: eleven flag bytes, an int, a float and six
/// ints. The client's loader names the last three of a battle EncounterNpcKOValue,
/// EncounterKnockoutRewardValue and EncounterKnockoutPenalty; the other roles are not
/// decoded, so the fields keep their position as their name.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub flags: [u8; 11],
    pub int_a: i32,
    pub float_b: f32,
    pub ints: [i32; 6],
}

impl Settings {
    /// Outside a battle (capture p12 t=18.7).
    pub fn world() -> Self {
        Self {
            flags: [0; 11],
            int_a: -1,
            float_b: 1.0,
            ints: [3600, 0, 0, 1, 0, 0],
        }
    }
    /// The Frostfang Growlers battle (capture p12 t=546240.0): 300 a knockout dealt, 25000
    /// and 5000 the knockout reward and penalty.
    pub fn battle() -> Self {
        Self {
            flags: [1; 11],
            int_a: -1,
            float_b: 500.0,
            ints: [1, 100_000, 500, 300, 25_000, 5_000],
        }
    }
    /// The same record with a fight's own score values.
    pub fn with_score(mut self, npc_knockout: i32, reward: i32, penalty: i32) -> Self {
        self.ints[3..].copy_from_slice(&[npc_knockout, reward, penalty]);
        self
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SettingsPair {
    pub world: Settings,
    pub battle: Settings,
}

impl Default for SettingsPair {
    fn default() -> Self {
        Self {
            world: Settings::world(),
            battle: Settings::battle(),
        }
    }
}

/// One captured packet and where its actor ids sit.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Template {
    #[serde(deserialize_with = "hex")]
    pub hex: Vec<u8>,
    /// Offsets of the caster's (or source's) u64.
    pub caster_at: Vec<usize>,
    /// Offsets of every other actor's u64; all get the one target.
    pub target_at: Vec<usize>,
    /// Bytes seen to differ between two rows of the template beyond the ids (a cast counter);
    /// replayed as captured.
    #[serde(default)]
    pub counter_at: Vec<usize>,
    #[serde(default)]
    pub ability: i32,
    #[serde(default)]
    pub targets: u32,
    /// Evidence: the capture file and the seconds from its first packet.
    pub capture: String,
    pub t: f64,
}

impl Template {
    fn fits(&self, head: &[u8]) -> bool {
        self.hex.starts_with(head)
            && self
                .caster_at
                .iter()
                .chain(&self.target_at)
                .all(|at| at.checked_add(8).is_some_and(|end| end <= self.hex.len()))
    }
    /// The packet with our ids in place.
    pub fn patched(&self, caster: u64, target: u64) -> Vec<u8> {
        let mut b = self.hex.clone();
        for (offsets, guid) in [(&self.caster_at, caster), (&self.target_at, target)] {
            for at in offsets {
                if let Some(slot) = b.get_mut(*at..*at + 8) {
                    slot.copy_from_slice(&guid.to_le_bytes());
                }
            }
        }
        b
    }
}

/// One effect row of a captured definition: where its ability id, chance and tuned low and
/// high values sit in the packet.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EffectRow {
    pub at: usize,
    pub ability_at: usize,
    pub chance_at: usize,
    pub low_at: usize,
    pub high_at: usize,
    pub low: f32,
    pub high: f32,
}

/// One captured 36/13 definition.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    #[serde(deserialize_with = "hex")]
    pub hex: Vec<u8>,
    pub name_id: i32,
    pub description_id: i32,
    pub rows: Vec<EffectRow>,
    pub t: f64,
}

/// Offsets in the fixed part of 36/13, read off the 33 captured definitions with the field
/// order of the client reader `0x00a32930`: the swing animation at 42 holds 1099 on every
/// basic and the reaction at 46 holds 1120 (`com_recoil`) or 1402 (`com_knock_down`).
pub const DEFINITION_ID_AT: usize = 4;
pub const DEFINITION_NAME_AT: usize = 10;
pub const DEFINITION_SUBTITLE_AT: usize = 14;
pub const DEFINITION_EFFECT_AT: usize = 34;
pub const DEFINITION_ANIMATION_AT: usize = 42;
pub const DEFINITION_REACTION_AT: usize = 46;
pub const DEFINITION_RANGE_AT: [usize; 2] = [62, 66];
pub const DEFINITION_DESCRIPTION_AT: usize = 103;
pub const DEFINITION_ROW_COUNT_AT: usize = 155;

/// What to change in a captured definition before sending it as another ability.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DefinitionPatch {
    pub id: i32,
    pub name_id: Option<i32>,
    pub description_id: Option<i32>,
    /// Tuned (low, high) per effect row, in row order; rows beyond the list keep the capture.
    pub values: Vec<(f32, f32)>,
}

impl Definition {
    fn fits(&self) -> bool {
        let len = self.hex.len();
        let inside = |at: usize| at.checked_add(4).is_some_and(|end| end <= len);
        self.hex.starts_with(&[36, 0, 13, 0])
            && len >= 160
            && self.rows.iter().all(|r| {
                [r.at, r.ability_at, r.chance_at, r.low_at, r.high_at]
                    .into_iter()
                    .all(inside)
            })
    }
    /// The definition as another ability: every copy of the id, the names and the tuned values.
    pub fn patched(&self, patch: &DefinitionPatch) -> Vec<u8> {
        let mut b = self.hex.clone();
        let mut put = |at: usize, value: [u8; 4]| {
            if let Some(slot) = b.get_mut(at..at + 4) {
                slot.copy_from_slice(&value);
            }
        };
        put(DEFINITION_ID_AT, patch.id.to_le_bytes());
        if let Some(name) = patch.name_id {
            put(DEFINITION_NAME_AT, name.to_le_bytes());
        }
        if let Some(description) = patch.description_id {
            put(DEFINITION_DESCRIPTION_AT, description.to_le_bytes());
        }
        for (i, row) in self.rows.iter().enumerate() {
            put(row.ability_at, patch.id.to_le_bytes());
            if let Some((low, high)) = patch.values.get(i) {
                put(row.low_at, low.to_le_bytes());
                put(row.high_at, high.to_le_bytes());
            }
        }
        b
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct WireData {
    pub schema_version: u32,
    pub death: Death,
    pub hit_effects: HitEffects,
    pub settings: SettingsPair,
    /// The live player-hit shape, when it has no kit to name its cast: the 36/3 ability and
    /// the launch template of a basic swing. Absent = the swing stays animation 1099.
    pub live_swing: Option<String>,
    /// 36/4 by template name: the ability id, with `/<targets>` when not one target.
    pub launches: BTreeMap<String, Template>,
    /// 50/1, the kill reward toast: caster = the defeated actor, target = the player.
    pub kill_toast: Option<Template>,
    /// 36/13 by the captured ability id.
    pub definitions: BTreeMap<String, Definition>,
    /// Evidence rows for the content checker.
    pub source: Vec<serde_json::Value>,
}

impl Default for WireData {
    fn default() -> Self {
        Self {
            schema_version: 1,
            death: Death::default(),
            hit_effects: HitEffects::default(),
            settings: SettingsPair::default(),
            live_swing: None,
            launches: BTreeMap::new(),
            kill_toast: None,
            definitions: BTreeMap::new(),
            source: Vec::new(),
        }
    }
}

impl WireData {
    /// The content directory's file, or the defaults when it has none.
    pub fn load(directory: &Path) -> io::Result<Arc<Self>> {
        let path = directory.join(FILE);
        if !path.is_file() {
            return Ok(Arc::new(Self::default()));
        }
        let refuse = |e: &dyn std::fmt::Display| invalid(&format!("{}: {e}", path.display()));
        if fs::metadata(&path).map_err(|e| refuse(&e))?.len() > MAX_FILE_BYTES {
            return Err(refuse(&"file too large"));
        }
        let data: Self = serde_json::from_slice(&fs::read(&path).map_err(|e| refuse(&e))?)
            .map_err(|e| refuse(&e))?;
        let minute = 60_000;
        if data.schema_version != 1
            || !(0..=minute).contains(&data.death.duration_ms)
            || data.death.effect_id < 0
            || [
                data.hit_effects.normal,
                data.hit_effects.special,
                data.hit_effects.knockout,
            ]
            .iter()
            .any(|id| *id < 0)
            || [&data.settings.world, &data.settings.battle]
                .iter()
                .any(|s| !s.float_b.is_finite() || s.flags.iter().any(|f| *f > 1))
        {
            return Err(refuse(&"value outside its sanity bounds"));
        }
        if let Some((name, _)) = data
            .launches
            .iter()
            .find(|(_, t)| !t.fits(&[36, 0, 4, 0]))
        {
            return Err(refuse(&format!("launch template {name} does not fit")));
        }
        if data
            .kill_toast
            .as_ref()
            .is_some_and(|t| !t.fits(&[50, 0, 1]))
        {
            return Err(refuse(&"the kill toast template does not fit"));
        }
        if let Some((name, _)) = data.definitions.iter().find(|(_, d)| !d.fits()) {
            return Err(refuse(&format!("definition {name} does not fit")));
        }
        if data
            .live_swing
            .as_ref()
            .is_some_and(|name| !data.launches.contains_key(name))
        {
            return Err(refuse(&"live_swing names no launch template"));
        }
        Ok(Arc::new(data))
    }

    /// 36/4 LaunchAndLand from the named captured template, with our ids.
    pub fn launch(&self, template: &str, caster: u64, target: u64) -> Option<Vec<u8>> {
        Some(self.launches.get(template)?.patched(caster, target))
    }

    /// 50/1: the reward toast of one kill.
    pub fn kill_toast(&self, defeated: u64, player: u64) -> Option<Vec<u8>> {
        Some(self.kill_toast.as_ref()?.patched(defeated, player))
    }

    /// 36/13 built from the captured definition of `template`.
    pub fn definition(&self, template: i32, patch: &DefinitionPatch) -> Option<Vec<u8>> {
        Some(self.definitions.get(&template.to_string())?.patched(patch))
    }

    /// 35/3: the graceful remove of a defeated actor after `delay_ms`.
    pub fn death(&self, guid: u64, delay_ms: u32) -> Vec<u8> {
        super::death_with(&self.death, guid, delay_ms)
    }

    /// Every packet of one player hit in the live shape (capture facts C1, C9): the cast
    /// (36/3, the caster in both ids, ability id only) and its 36/4 when the hit names an
    /// ability with a template here, then 35/35 on the enemy and its bar (35/5). The 36/14
    /// reaction is not part of every hit: live sent it with some abilities only.
    pub fn player_hit_live(&self, h: &super::PlayerHit) -> Vec<Vec<u8>> {
        let mut packets = Vec::with_capacity(4);
        let name = h
            .ability
            .map(|id| id.to_string())
            .or_else(|| h.swing.then(|| self.live_swing.clone()).flatten());
        match name.as_ref().and_then(|n| Some((n, self.launches.get(n)?))) {
            Some((_, template)) => {
                packets.push(super::cast(h.attacker, h.attacker, template.ability));
                packets.push(template.patched(h.attacker, h.target));
            }
            None if h.swing => packets.push(super::animation(h.attacker, 1099, false)),
            None => {}
        }
        let dealt = i32::try_from(h.dealt).unwrap_or(i32::MAX);
        packets.push(super::hitpoints(
            h.attacker,
            h.target,
            true,
            i32::try_from(h.max_health).unwrap_or(i32::MAX),
            i32::try_from(h.health).unwrap_or(i32::MAX),
            -dealt,
            h.critical,
        ));
        packets.push(super::enemy_hp(h.target, h.health, h.max_health));
        packets
    }
}
