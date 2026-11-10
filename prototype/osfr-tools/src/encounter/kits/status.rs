//! `status_effects.json`: one status table and one book for players and enemies alike, so a
//! stun from a kit ability and a stun from an enemy attack are the same code and packets.
use super::table::{RawDot, read_optional};
use crate::encounter::*;
use std::sync::OnceLock;

fn bad(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

/// The book's key for the engine's own player (enemy guids are never 0).
pub(in crate::encounter) const ME: u64 = 0;

/// One status row. A modifier of `None` or 0 changes nothing.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StatusRow {
    /// Bit of the client's 35/20 state word.
    #[serde(default)]
    pub bit: Option<u8>,
    /// Looping composite effect attached while the status lasts (35/41).
    #[serde(default)]
    pub effect_id: i32,
    /// Clip played when it lands.
    #[serde(default)]
    pub animation: i32,
    /// Buff-bar icon and name (38/16); 0 sends no icon.
    #[serde(default)]
    pub icon_id: i32,
    #[serde(default)]
    pub name_id: i32,
    /// Any of `move`, `act`, `cast`.
    #[serde(default)]
    pub blocks: Vec<String>,
    #[serde(default)]
    pub breaks_on_damage: bool,
    pub default_ms: u32,
    /// After it ends the target cannot get it again for this long.
    #[serde(default)]
    pub immune_ms: u32,
    #[serde(default)]
    pub immune_bosses: bool,
    #[serde(default)]
    pub helpful: bool,
    #[serde(default)]
    pub dealt: Option<f32>,
    #[serde(default)]
    pub taken: Option<f32>,
    #[serde(default)]
    pub crit: i32,
    #[serde(default)]
    pub crit_against: u32,
    #[serde(default)]
    pub crit_damage: f32,
    #[serde(default)]
    pub speed: Option<f32>,
    #[serde(default)]
    pub energy_per_tick: i32,
    /// Share of each hit taken that comes back as health.
    #[serde(default)]
    pub heal_on_damage: f32,
    /// Flat damage added to each hit, as its Power 5 value.
    #[serde(default)]
    pub dealt_add_power5: u32,
    #[serde(default)]
    pub(in crate::encounter) on_hit_dot: Option<RawDot>,
    #[serde(default)]
    #[allow(dead_code)]
    source: serde_json::Value,
}

impl StatusRow {
    pub fn blocks(&self, what: &str) -> bool {
        self.blocks.iter().any(|b| b == what)
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BaseBits {
    pub enemy: u32,
    pub player: u32,
}

/// Which status packets go out. The clip is always sent; the state word and the attached
/// effect wait for their native proof.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StatusWire {
    pub state_bits: bool,
    pub attached_effects: bool,
    pub base_bits: BaseBits,
    pub first_tag: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    schema_version: u32,
    #[allow(dead_code)]
    source: serde_json::Value,
    wire: StatusWire,
    statuses: BTreeMap<String, StatusRow>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StatusTable {
    pub wire: StatusWire,
    rows: BTreeMap<String, Arc<StatusRow>>,
}

impl StatusTable {
    pub fn builtin() -> &'static StatusTable {
        static TABLE: OnceLock<StatusTable> = OnceLock::new();
        TABLE.get_or_init(|| {
            Self::parse(include_bytes!("../../../data/content/status_effects.json")).unwrap_or_default()
        })
    }

    /// `status_effects.json` in `directory`; `None` when the file is absent.
    pub fn load(directory: &Path) -> io::Result<Option<StatusTable>> {
        read_optional(&directory.join("status_effects.json"))?
            .map(|bytes| Self::parse(&bytes))
            .transpose()
    }

    pub fn parse(bytes: &[u8]) -> io::Result<StatusTable> {
        let raw: Raw = serde_json::from_slice(bytes)
            .map_err(|e| bad(format!("status_effects.json: {e}")))?;
        if raw.schema_version != 1 || raw.statuses.len() > 256 || raw.wire.first_tag <= 0 {
            return Err(bad("status_effects.json: unsupported content"));
        }
        let factor = |v: Option<f32>| v.is_none_or(|v| (0.05..=10.0).contains(&v));
        for (id, row) in &raw.statuses {
            if row.bit.is_some_and(|bit| bit > 31)
                || row.effect_id < 0
                || row.animation < 0
                || row.icon_id < 0
                || row.name_id < 0
                || !(100..=600_000).contains(&row.default_ms)
                || row.immune_ms > 600_000
                || row.blocks.iter().any(|b| !matches!(b.as_str(), "move" | "act" | "cast"))
                || !factor(row.dealt)
                || !factor(row.taken)
                || !factor(row.speed)
                || !(-100..=100).contains(&row.crit)
                || row.crit_against > 100
                || !(-1.0..=5.0).contains(&row.crit_damage)
                || !(0..=100).contains(&row.energy_per_tick)
                || !(0.0..=1.0).contains(&row.heal_on_damage)
                || row.on_hit_dot.as_ref().is_some_and(|d| {
                    !(250..=60_000).contains(&d.every_ms) || !(250..=60_000).contains(&d.ms)
                })
            {
                return Err(bad(format!("status_effects.json: status {id} out of range")));
            }
        }
        Ok(StatusTable {
            wire: raw.wire,
            rows: raw.statuses.into_iter().map(|(id, row)| (id, Arc::new(row))).collect(),
        })
    }

    pub fn row(&self, id: &str) -> Option<&Arc<StatusRow>> {
        self.rows.get(id)
    }

    pub fn ids(&self) -> impl Iterator<Item = &String> {
        self.rows.keys()
    }
}

#[derive(Clone)]
pub(in crate::encounter) struct Applied {
    pub until: u64,
    pub row: Arc<StatusRow>,
    pub tag: i32,
    /// This entry also holds the enemy in the shared stun table.
    stuns: bool,
}

/// Who has which status, and who is immune for now. Session only.
#[derive(Clone, Default)]
pub(in crate::encounter) struct Book {
    active: BTreeMap<(u64, String), Applied>,
    immune: BTreeMap<(u64, String), u64>,
    /// Ended outside a tick (broken by damage): their packets go out on the next tick.
    ended: Vec<(u64, Applied)>,
    next_tag: i32,
}

impl Book {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn forget(&mut self, guid: u64) {
        self.active.retain(|(target, _), _| *target != guid);
        self.immune.retain(|(target, _), _| *target != guid);
    }

    fn of(&self, guid: u64) -> impl Iterator<Item = &Applied> {
        self.active
            .range((guid, String::new())..)
            .take_while(move |((target, _), _)| *target == guid)
            .map(|(_, applied)| applied)
    }

    pub fn has(&self, guid: u64, id: &str, now_ms: u64) -> bool {
        self.active
            .get(&(guid, id.to_string()))
            .is_some_and(|a| now_ms < a.until)
    }

    pub fn blocked(&self, guid: u64, what: &str, now_ms: u64) -> bool {
        self.of(guid).any(|a| now_ms < a.until && a.row.blocks(what))
    }

    /// Product of one factor over the statuses on `guid`.
    fn product(&self, guid: u64, pick: impl Fn(&StatusRow) -> Option<f32>) -> f32 {
        self.of(guid).filter_map(|a| pick(&a.row)).product()
    }

    pub fn dealt(&self, guid: u64) -> f32 {
        self.product(guid, |row| row.dealt)
    }

    pub fn taken(&self, guid: u64) -> f32 {
        self.product(guid, |row| row.taken)
    }

    pub fn speed(&self, guid: u64) -> f32 {
        self.product(guid, |row| row.speed)
    }

    pub fn crit(&self, guid: u64) -> i32 {
        self.of(guid).map(|a| a.row.crit).sum()
    }

    pub fn crit_against(&self, guid: u64) -> u32 {
        self.of(guid).map(|a| a.row.crit_against).sum()
    }

    pub fn crit_damage(&self, guid: u64) -> f32 {
        self.of(guid).map(|a| a.row.crit_damage).sum()
    }

    pub fn energy_per_tick(&self, guid: u64) -> i32 {
        self.of(guid).map(|a| a.row.energy_per_tick).sum()
    }

    pub fn heal_on_damage(&self, guid: u64) -> f32 {
        self.of(guid).map(|a| a.row.heal_on_damage).fold(0.0, f32::max)
    }

    pub fn dealt_add_power5(&self, guid: u64) -> u32 {
        self.of(guid).map(|a| a.row.dealt_add_power5).sum()
    }

    pub fn on_hit_dots(&self, guid: u64) -> Vec<RawDot> {
        self.of(guid).filter_map(|a| a.row.on_hit_dot.clone()).collect()
    }

    fn bits(&self, guid: u64) -> u32 {
        self.of(guid)
            .filter_map(|a| a.row.bit)
            .fold(0, |bits, bit| bits | 1 << bit)
    }

    /// A damaging hit ends every status on `guid` that breaks on damage. Returns whether one
    /// of them held the enemy in the stun table.
    pub fn broken_by_damage(&mut self, guid: u64) -> bool {
        let keys: Vec<_> = self
            .active
            .range((guid, String::new())..)
            .take_while(|((target, _), _)| *target == guid)
            .filter(|(_, a)| a.row.breaks_on_damage)
            .map(|(key, _)| key.clone())
            .collect();
        let mut stuns = false;
        for key in keys {
            if let Some(applied) = self.active.remove(&key) {
                stuns |= applied.stuns;
                self.ended.push((guid, applied));
            }
        }
        stuns && !self.of(guid).any(|a| a.stuns)
    }
}

impl EncounterEngine {
    fn status_wire_guid(&self, key: u64) -> u64 {
        if key == ME {
            self.shared_sender.unwrap_or(1)
        } else {
            key
        }
    }

    fn status_state_packet(&self, key: u64) -> Vec<u8> {
        let wire = &self.skills.kit.statuses.wire;
        let base = if key == ME { wire.base_bits.player } else { wire.base_bits.enemy };
        wire::state(self.status_wire_guid(key), base | self.skills.kit.book.bits(key))
    }

    /// Put status `id` on `target` for `ms` (0 = the status's own default). `target` is an
    /// enemy's guid; any other guid is this engine's player. Returns whether it landed: a dead,
    /// shielded or immune target takes none. Kit abilities, traits, battle items and enemy
    /// attacks all come through here.
    pub fn apply_status(
        &mut self,
        target: u64,
        id: &str,
        ms: u32,
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) -> bool {
        let Some(row) = self.skills.kit.statuses.row(id).cloned() else {
            return false;
        };
        let enemy = self.enemy(target).map(|(a, s)| (a.boss, s.health));
        let key = match enemy {
            Some((boss, health)) => {
                if health == 0 || self.shielded(target) || boss && row.immune_bosses {
                    return false;
                }
                target
            }
            None if self.state.player_health == 0 => return false,
            None => ME,
        };
        let book_key = (key, id.to_string());
        if self
            .skills
            .kit
            .book
            .immune
            .get(&book_key)
            .is_some_and(|until| now_ms < *until)
        {
            return false;
        }
        let ms = if ms == 0 { row.default_ms } else { ms };
        let until = now_ms.saturating_add(u64::from(ms));
        let stuns = key != ME && row.blocks("act");
        let wire_guid = self.status_wire_guid(key);
        let wire = self.skills.kit.statuses.wire.clone();
        let book = &mut self.skills.kit.book;
        // A second application while it runs refreshes it and keeps the longer end.
        let running = book.active.get(&book_key).filter(|a| now_ms < a.until);
        let until = running.map_or(until, |a| a.until.max(until));
        let refresh = running.map(|a| a.tag);
        let tag = refresh.unwrap_or_else(|| {
            book.next_tag = book.next_tag.wrapping_add(1).rem_euclid(1_000_000);
            wire.first_tag.saturating_add(book.next_tag)
        });
        book.active.insert(book_key, Applied { until, row: row.clone(), tag, stuns });
        if stuns {
            // The shared stun table is what both enemy ticks read, and what a shared field
            // carries between players.
            self.skills
                .stuns
                .entry(target)
                .and_modify(|current| *current = (*current).max(until))
                .or_insert(until);
        }
        if key != ME && row.blocks("move") {
            self.arena.moving.remove(&target);
            self.field_sim.moving.remove(&target);
        }
        if id == "taunt" && key != ME {
            let threat = self.skills.kit.taunt_threat;
            self.add_threat(target, threat);
        }
        if row.animation > 0 {
            if key != ME {
                out.replies.push(animation(target, 1, true));
            }
            out.replies.push(animation(wire_guid, row.animation, false));
        }
        if wire.state_bits && row.bit.is_some() {
            out.replies.push(self.status_state_packet(key));
        }
        if refresh.is_none() && wire.attached_effects && row.effect_id > 0 {
            let source = self.shared_sender.unwrap_or(1);
            out.replies.push(wire::attach(wire_guid, tag, row.effect_id, source));
        }
        if key == ME && row.icon_id > 0 {
            out.replies.push(wire::tag_add(&wire::Tag {
                id: tag,
                duration_ms: u64::from(ms),
                icon_id: row.icon_id,
                name_id: row.name_id,
            }));
        }
        out.changed = true;
        true
    }

    /// Whether `guid` (an enemy, or any other guid for the player) has status `id` now.
    pub fn has_status(&self, guid: u64, id: &str, now_ms: u64) -> bool {
        let key = if self.enemy(guid).is_some() { guid } else { ME };
        self.skills.kit.book.has(key, id, now_ms)
    }

    /// An enemy's damage factor from its statuses (attack down), for the enemy ticks.
    pub fn enemy_dealt_factor(&self, guid: u64) -> f32 {
        self.skills.kit.book.dealt(guid)
    }

    /// An enemy's speed factor from its statuses (slow), for the enemy ticks.
    pub fn enemy_speed_factor(&self, guid: u64) -> f32 {
        self.skills.kit.book.speed(guid)
    }

    /// End what has run out, and send the end of everything that ended since the last tick.
    pub(in crate::encounter) fn expire_statuses(&mut self, now_ms: u64, out: &mut EncounterOutcome) {
        let book = &mut self.skills.kit.book;
        book.immune.retain(|_, until| now_ms < *until);
        let over: Vec<_> = book
            .active
            .iter()
            .filter(|(_, a)| a.until <= now_ms)
            .map(|(key, _)| key.clone())
            .collect();
        let mut ended = std::mem::take(&mut book.ended);
        for key in over {
            if let Some(applied) = book.active.remove(&key) {
                if applied.row.immune_ms > 0 {
                    book.immune
                        .insert(key.clone(), now_ms.saturating_add(u64::from(applied.row.immune_ms)));
                }
                ended.push((key.0, applied));
            }
        }
        let wire = self.skills.kit.statuses.wire.clone();
        for (key, applied) in ended {
            let guid = self.status_wire_guid(key);
            if key != ME && self.enemy(key).is_none() {
                continue; // gone with its actor
            }
            if wire.state_bits && applied.row.bit.is_some() {
                out.replies.push(self.status_state_packet(key));
            }
            if wire.attached_effects && applied.row.effect_id > 0 {
                out.replies.push(wire::detach(guid, applied.tag));
            }
            if key == ME && applied.row.icon_id > 0 {
                out.replies.push(wire::tag_remove(applied.tag));
            }
        }
    }
}
